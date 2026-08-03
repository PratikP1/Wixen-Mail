//! Grouping messages into conversations.
//!
//! Threading is done from the `References` and `In-Reply-To` headers the
//! envelope already carries, so it costs no extra fetch. This is the JWZ
//! approach, minus the parts that trade correctness for tidiness.
//!
//! Subject matching is deliberately not used. "Re: lunch" collides across years
//! and strangers, and a thread that quietly merges two unrelated conversations
//! is worse than two threads: someone reading by ear has no way to see that the
//! sender changed halfway down.
//!
//! Assignment is incremental, because mail arrives one message at a time. The
//! case worth testing is a late message that references two messages already in
//! separate trees, which merges them.

use std::collections::HashMap;

/// What threading needs from a message. Nothing else is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadInput {
    /// The row this describes.
    pub id: i64,
    /// The `Message-ID` header.
    pub message_id: String,
    /// `References`, oldest ancestor first, with `In-Reply-To` appended.
    pub references: Vec<String>,
    /// A thread id the server supplied, such as Gmail's `X-GM-THRID`.
    ///
    /// Preferred over anything computed here, because it matches what that
    /// provider shows the same user in its own interface, and disagreeing with
    /// the web client about what a conversation is helps nobody.
    pub server_thread_id: Option<String>,
}

/// Where a message sits in its conversation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadPlacement {
    pub id: i64,
    /// The conversation this belongs to. Every message has one, including
    /// messages that are alone.
    pub thread_id: String,
    /// The message this replies to, where that message is present.
    pub parent_id: Option<i64>,
    /// How far down the conversation this sits. Zero is the root.
    pub depth: usize,
}

/// Group messages into conversations.
///
/// Every message comes back with a placement, including ones with no relatives.
/// A message alone is a conversation of one rather than a special case, so the
/// caller has one shape to handle instead of two.
pub fn thread_messages(messages: &[ThreadInput]) -> Vec<ThreadPlacement> {
    // Message-ID to row, so a reference can find what it points at. Duplicate
    // ids keep the first: a forged or repeated Message-ID must not let a later
    // message capture an earlier one's children.
    let mut by_message_id: HashMap<&str, i64> = HashMap::new();
    for message in messages {
        let key = message.message_id.trim();
        if !key.is_empty() {
            by_message_id.entry(key).or_insert(message.id);
        }
    }

    let mut union = DisjointSet::new();
    let mut parents: HashMap<i64, Option<i64>> = HashMap::new();

    for message in messages {
        union.add(message.id);
        // The nearest reference that is actually present is the parent. The
        // list runs oldest first, so the last match is the closest ancestor.
        let parent = message
            .references
            .iter()
            .rev()
            .filter_map(|reference| by_message_id.get(reference.trim()).copied())
            .find(|candidate| *candidate != message.id);
        parents.insert(message.id, parent);

        // Every reference joins the same conversation, present or not. That is
        // what merges two trees when a late message references both.
        for reference in &message.references {
            if let Some(other) = by_message_id.get(reference.trim()) {
                union.union(message.id, *other);
            }
        }
    }

    // A server thread id overrides everything computed: messages sharing one
    // are one conversation whatever the headers say.
    let mut by_server_id: HashMap<&str, i64> = HashMap::new();
    for message in messages {
        let Some(server_id) = message.server_thread_id.as_deref() else {
            continue;
        };
        let server_id = server_id.trim();
        if server_id.is_empty() {
            continue;
        }
        match by_server_id.get(server_id) {
            Some(first) => union.union(message.id, *first),
            None => {
                by_server_id.insert(server_id, message.id);
            }
        }
    }

    // The conversation is named after the least Message-ID it holds, in plain
    // string order, rather than a generated value. Least rather than oldest,
    // because a date can be missing, wrong or in another timezone, and the
    // point of the rule is that the same mailbox threads the same way on every
    // machine and after every restart.
    let mut root_name: HashMap<i64, String> = HashMap::new();
    for message in messages {
        let root = union.find(message.id);
        root_name
            .entry(root)
            .and_modify(|name| {
                if message.message_id.as_str() < name.as_str() {
                    *name = message.message_id.clone();
                }
            })
            .or_insert_with(|| message.message_id.clone());
    }

    messages
        .iter()
        .map(|message| {
            let root = union.find(message.id);
            let thread_id = root_name
                .get(&root)
                .cloned()
                .unwrap_or_else(|| message.message_id.clone());
            ThreadPlacement {
                id: message.id,
                thread_id,
                parent_id: parents.get(&message.id).copied().flatten(),
                depth: depth_of(message.id, &parents),
            }
        })
        .collect()
}

/// How far a message sits from its root.
///
/// Bounded by the number of links followed rather than trusting the data:
/// mail with a reference cycle exists, whether through a broken client or a
/// deliberately malformed message, and a cycle here would hang the list.
fn depth_of(id: i64, parents: &HashMap<i64, Option<i64>>) -> usize {
    let mut depth = 0;
    let mut current = id;
    for _ in 0..parents.len() {
        match parents.get(&current).copied().flatten() {
            Some(parent) if parent != current => {
                depth += 1;
                current = parent;
            }
            _ => break,
        }
    }
    depth
}

/// Union-find over message row ids.
#[derive(Debug, Default)]
struct DisjointSet {
    parent: HashMap<i64, i64>,
}

impl DisjointSet {
    pub fn new() -> Self {
        Self::default()
    }

    fn add(&mut self, id: i64) {
        self.parent.entry(id).or_insert(id);
    }

    fn find(&mut self, id: i64) -> i64 {
        let mut root = id;
        while let Some(&parent) = self.parent.get(&root) {
            if parent == root {
                break;
            }
            root = parent;
        }
        // Path compression, so a long chain does not cost the same walk again
        // on the next lookup.
        let mut current = id;
        while let Some(&parent) = self.parent.get(&current) {
            if parent == root {
                break;
            }
            self.parent.insert(current, root);
            current = parent;
        }
        root
    }

    fn union(&mut self, a: i64, b: i64) {
        self.add(a);
        self.add(b);
        let (root_a, root_b) = (self.find(a), self.find(b));
        if root_a != root_b {
            // Smaller id wins, so the result does not depend on the order the
            // messages happened to arrive in.
            let (keep, merge) = if root_a < root_b {
                (root_a, root_b)
            } else {
                (root_b, root_a)
            };
            self.parent.insert(merge, keep);
        }
    }
}

/// The heading level a message renders at in a combined thread document.
///
/// Capped at six and never skipping, because skipping a heading level is a
/// structure violation in its own right and conversations go deeper than six.
/// The real depth goes in the heading text instead, so nothing is lost.
pub fn heading_level(depth: usize) -> usize {
    (depth + 1).min(6)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(id: i64, message_id: &str, references: &[&str]) -> ThreadInput {
        ThreadInput {
            id,
            message_id: message_id.to_string(),
            references: references.iter().map(|r| r.to_string()).collect(),
            server_thread_id: None,
        }
    }

    fn placement(placements: &[ThreadPlacement], id: i64) -> &ThreadPlacement {
        placements
            .iter()
            .find(|p| p.id == id)
            .expect("every message gets a placement")
    }

    #[test]
    fn test_a_message_with_no_relatives_is_a_conversation_of_one() {
        // One shape for the caller to handle rather than two.
        let placements = thread_messages(&[message(1, "<a@x>", &[])]);
        assert_eq!(placements.len(), 1);
        assert_eq!(placements[0].depth, 0);
        assert_eq!(placements[0].parent_id, None);
        assert_eq!(placements[0].thread_id, "<a@x>");
    }

    #[test]
    fn test_a_reply_joins_its_parent() {
        let placements = thread_messages(&[
            message(1, "<a@x>", &[]),
            message(2, "<b@x>", &["<a@x>"]),
            message(3, "<c@x>", &["<a@x>", "<b@x>"]),
        ]);
        assert_eq!(placement(&placements, 2).parent_id, Some(1));
        assert_eq!(placement(&placements, 2).depth, 1);
        // The nearest present reference is the parent, not the oldest.
        assert_eq!(placement(&placements, 3).parent_id, Some(2));
        assert_eq!(placement(&placements, 3).depth, 2);
        let thread = &placement(&placements, 1).thread_id;
        assert!(placements.iter().all(|p| &p.thread_id == thread));
    }

    #[test]
    fn test_a_conversation_is_named_after_the_least_identifier_it_holds() {
        // Which one is picked is the whole reason the name is stable. Any rule
        // that depends on the order the rows came back names the conversation
        // differently on another machine, and a saved search or an expanded
        // thread stops matching after a restart.
        let placements = thread_messages(&[
            message(1, "<m@x>", &[]),
            message(2, "<a@x>", &["<m@x>"]),
            message(3, "<z@x>", &["<m@x>"]),
        ]);

        // Neither first nor last in the input, so taking either would fail here.
        assert_eq!(placement(&placements, 1).thread_id, "<a@x>");
        assert_eq!(placement(&placements, 2).thread_id, "<a@x>");
        assert_eq!(placement(&placements, 3).thread_id, "<a@x>");
    }

    #[test]
    fn test_a_late_message_referencing_two_trees_merges_them() {
        // The case that separates real threading from a lookup table.
        let placements = thread_messages(&[
            message(1, "<a@x>", &[]),
            message(2, "<b@x>", &[]),
            message(3, "<c@x>", &["<a@x>", "<b@x>"]),
        ]);
        let thread = &placement(&placements, 3).thread_id;
        assert_eq!(&placement(&placements, 1).thread_id, thread);
        assert_eq!(&placement(&placements, 2).thread_id, thread);
    }

    #[test]
    fn test_a_missing_ancestor_does_not_break_the_chain() {
        // Someone was added to a conversation halfway through, so the messages
        // referenced above them are not in this mailbox at all.
        let placements = thread_messages(&[
            message(1, "<b@x>", &["<missing@x>"]),
            message(2, "<c@x>", &["<missing@x>", "<b@x>"]),
        ]);
        assert_eq!(placement(&placements, 1).parent_id, None);
        assert_eq!(placement(&placements, 1).depth, 0);
        assert_eq!(placement(&placements, 2).parent_id, Some(1));
        assert_eq!(
            placement(&placements, 1).thread_id,
            placement(&placements, 2).thread_id
        );
    }

    #[test]
    fn test_subjects_are_never_used_to_thread() {
        // "Re: lunch" collides across years and strangers, and a thread that
        // silently merges two conversations is worse than two threads.
        let placements = thread_messages(&[message(1, "<a@x>", &[]), message(2, "<b@x>", &[])]);
        assert_ne!(
            placement(&placements, 1).thread_id,
            placement(&placements, 2).thread_id
        );
    }

    #[test]
    fn test_a_server_thread_id_wins_over_the_headers() {
        // Disagreeing with the provider's own web client about what a
        // conversation is helps nobody.
        let mut a = message(1, "<a@x>", &[]);
        let mut b = message(2, "<b@x>", &[]);
        a.server_thread_id = Some("thr-1".to_string());
        b.server_thread_id = Some("thr-1".to_string());
        let placements = thread_messages(&[a, b]);
        assert_eq!(
            placement(&placements, 1).thread_id,
            placement(&placements, 2).thread_id
        );
    }

    #[test]
    fn test_a_reference_cycle_does_not_hang() {
        // Mail with a reference cycle exists, from broken clients and from
        // deliberately malformed messages. A cycle here would freeze the list,
        // and a freeze is an accessibility failure before it is a performance
        // one.
        let placements = thread_messages(&[
            message(1, "<a@x>", &["<b@x>"]),
            message(2, "<b@x>", &["<a@x>"]),
        ]);
        assert_eq!(placements.len(), 2);
        for p in &placements {
            assert!(p.depth <= 2, "depth ran away: {:?}", p);
        }
    }

    #[test]
    fn test_a_message_referencing_itself_is_not_its_own_parent() {
        let placements = thread_messages(&[message(1, "<a@x>", &["<a@x>"])]);
        assert_eq!(placements[0].parent_id, None);
        assert_eq!(placements[0].depth, 0);
    }

    #[test]
    fn test_a_duplicate_message_id_does_not_let_one_message_capture_another() {
        // A repeated or forged Message-ID must not let a later message adopt
        // an earlier one's replies.
        let placements = thread_messages(&[
            message(1, "<a@x>", &[]),
            message(2, "<a@x>", &[]),
            message(3, "<c@x>", &["<a@x>"]),
        ]);
        assert_eq!(placement(&placements, 3).parent_id, Some(1));
    }

    #[test]
    fn test_threading_does_not_depend_on_arrival_order() {
        // The same mailbox has to thread the same way on every machine.
        let forwards = thread_messages(&[
            message(1, "<a@x>", &[]),
            message(2, "<b@x>", &["<a@x>"]),
            message(3, "<c@x>", &["<a@x>", "<b@x>"]),
        ]);
        let backwards = thread_messages(&[
            message(3, "<c@x>", &["<a@x>", "<b@x>"]),
            message(2, "<b@x>", &["<a@x>"]),
            message(1, "<a@x>", &[]),
        ]);
        for id in [1, 2, 3] {
            assert_eq!(
                placement(&forwards, id).thread_id,
                placement(&backwards, id).thread_id,
                "message {} threaded differently",
                id
            );
        }
    }

    #[test]
    fn test_whitespace_around_a_message_id_does_not_break_matching() {
        // Headers arrive folded and padded, and a reference that fails to
        // match silently produces a message that is alone for no reason.
        let placements = thread_messages(&[
            message(1, " <a@x> ", &[]),
            message(2, "<b@x>", &["  <a@x>  "]),
        ]);
        assert_eq!(placement(&placements, 2).parent_id, Some(1));
    }

    #[test]
    fn test_heading_levels_cap_at_six_and_never_skip() {
        // Skipping a heading level is a structure violation in its own right,
        // and conversations go deeper than six.
        assert_eq!(heading_level(0), 1);
        assert_eq!(heading_level(4), 5);
        assert_eq!(heading_level(5), 6);
        assert_eq!(heading_level(8), 6);
        for depth in 0..20 {
            let level = heading_level(depth);
            assert!((1..=6).contains(&level));
            if depth > 0 {
                assert!(level - heading_level(depth - 1) <= 1, "skipped a level");
            }
        }
    }

    #[test]
    fn test_an_empty_mailbox_threads_to_nothing() {
        assert!(thread_messages(&[]).is_empty());
    }

    #[test]
    fn test_a_missing_message_id_still_gets_a_placement() {
        // Malformed mail exists. A message with no Message-ID must still
        // appear in the list rather than vanishing from it.
        let placements = thread_messages(&[message(1, "", &[]), message(2, "  ", &[])]);
        assert_eq!(placements.len(), 2);
    }
}

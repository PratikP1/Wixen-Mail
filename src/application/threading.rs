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
                // Whichever of `<` or `<=` guards the assignment above,
                // `name` ends up the lexicographic minimum of its previous
                // value and this message's id: at equality the two are the
                // same string, so replacing one with the other changes
                // nothing observable. That is why cargo-mutants cannot tell
                // `<` from `<=` on the line above.
                debug_assert!(*name <= message.message_id);
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

    fn find(&mut self, id: i64) -> i64 {
        // Bounded by the number of entries rather than trusting the chain to
        // end on its own, the same reasoning `depth_of` already applies to
        // this same shape of data: `union` is the only thing supposed to
        // keep this structure acyclic, and a chain can never need more than
        // one step per entry, so that bound is never too tight for a real
        // chain and always enough to stop a corrupted one cold rather than
        // spin on it.
        let mut root = id;
        for _ in 0..self.parent.len() {
            match self.parent.get(&root) {
                Some(&parent) if parent != root => root = parent,
                _ => break,
            }
        }
        // Path compression, so a long chain does not cost the same walk again
        // on the next lookup.
        let mut current = id;
        for _ in 0..self.parent.len() {
            match self.parent.get(&current) {
                Some(&parent) if parent != root => {
                    self.parent.insert(current, root);
                    current = parent;
                }
                _ => break,
            }
        }
        root
    }

    fn union(&mut self, a: i64, b: i64) {
        let (root_a, root_b) = (self.find(a), self.find(b));
        if root_a != root_b {
            // Smaller id wins. This is a choice of representative and nothing
            // more: the grouping is the same set of conversations whichever of
            // the two survives, and what makes the answer independent of
            // arrival order is that the name comes from the least Message-ID
            // in the group rather than from the representative. The rule is
            // here so that `find` gives the same answer twice running, which
            // is worth having while anything debugs against it.
            //
            // `root_a < root_b` cannot be swapped for `<=` and be told
            // apart: the `if` above already rules out root_a == root_b, so
            // the two comparisons agree on every value that reaches here.
            debug_assert_ne!(root_a, root_b);
            let (keep, merge) = if root_a < root_b {
                (root_a, root_b)
            } else {
                (root_b, root_a)
            };
            self.parent.insert(merge, keep);
        }
    }
}

/// The two headers that put a reply in the conversation it answers.
///
/// Finished header values, angle brackets and all, so nothing downstream has
/// to know the bracket rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Continuing {
    /// The message being answered.
    pub in_reply_to: String,
    /// Everything before it, oldest first, then it.
    pub references: String,
}

/// The headers a reply to this message should carry.
///
/// The mirror of [`crate::application::mail_sync`]'s reading of the same two
/// headers, and the two have to agree or a reply this program sends will not
/// thread in the client that receives it, with nothing here able to notice.
///
/// `References` is a chain, not one value: the parent's chain, then the parent.
/// Sending only the parent puts every reply directly under the top of the
/// thread, and sending only the chain loses the message being answered.
///
/// `None` where the parent has no identifier of its own. `In-Reply-To: <>` is
/// worse than no header: it is a malformed value that some servers refuse and
/// every client ignores, so such a reply starts a new conversation openly
/// rather than a broken one.
///
/// # The brackets are ours to add
///
/// The cache stores identifiers bare, because the parser unwraps them, while a
/// draft's own identifier is stored already wrapped. The mail library writes
/// whichever string it is handed, verbatim. So both spellings arrive here and
/// exactly one pair of brackets leaves.
pub fn continuing(parent_message_id: &str, parent_references: Option<&str>) -> Option<Continuing> {
    let parent = parent_message_id.trim();
    if parent.is_empty() {
        return None;
    }
    let parent = bracketed(parent);

    let mut chain: Vec<String> = parent_references
        .unwrap_or_default()
        .split_whitespace()
        .map(bracketed)
        .collect();
    if !chain.contains(&parent) {
        chain.push(parent.clone());
    }

    Some(Continuing {
        in_reply_to: parent,
        references: chain.join(" "),
    })
}

/// The `References` column as the cache stores it, or nothing to store.
///
/// Every ancestor the sender named, oldest first, then the message answered
/// where that is not already among them. Identifiers go in as they arrive,
/// bare, because that is how both parsers hand them over and how
/// [`continuing`] expects to read them back.
///
/// One rule for both ways a message reaches the cache: downloaded by a sync,
/// and filed here after it has gone out. Two copies of it would mean the same
/// message threading one way when it arrives and another when it is sent.
pub fn as_stored(references: &[String], in_reply_to: Option<&str>) -> Option<String> {
    let mut chain: Vec<&str> = references.iter().map(String::as_str).collect();
    if let Some(parent) = in_reply_to
        && !chain.contains(&parent)
    {
        chain.push(parent);
    }
    if chain.is_empty() {
        return None;
    }
    Some(chain.join(" "))
}

/// One identifier with exactly one pair of angle brackets around it.
fn bracketed(id: &str) -> String {
    let bare = id.trim().trim_start_matches('<').trim_end_matches('>');
    format!("<{bare}>")
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

    #[test]
    fn test_a_reply_names_its_parent_and_the_whole_chain_before_it() {
        // References is a chain, not one value. The parent's own chain plus the
        // parent, in that order, which is what puts the reply under the right
        // message rather than under the top of the thread.
        let chain = continuing("c@x", Some("a@x b@x")).expect("a chain");
        assert_eq!(chain.in_reply_to, "<c@x>");
        assert_eq!(chain.references, "<a@x> <b@x> <c@x>");
    }

    #[test]
    fn test_a_reply_to_the_first_message_of_a_thread_still_names_it() {
        // Nothing came before it, so the chain is just the message answered.
        // Leaving References off here is the mistake that starts a new
        // conversation on the second message of every thread.
        let chain = continuing("c@x", None).expect("a chain");
        assert_eq!(chain.in_reply_to, "<c@x>");
        assert_eq!(chain.references, "<c@x>");
    }

    #[test]
    fn test_a_parent_that_already_named_itself_is_not_named_twice() {
        // Some senders put their own identifier in their References.
        let chain = continuing("c@x", Some("a@x c@x")).expect("a chain");
        assert_eq!(chain.references, "<a@x> <c@x>");
    }

    #[test]
    fn test_a_message_with_no_identifier_starts_a_new_conversation_rather_than_a_broken_one() {
        // `In-Reply-To: <>` is worse than no header: it is a malformed value
        // some servers refuse and every client ignores.
        assert!(continuing("", Some("a@x")).is_none());
        assert!(continuing("   ", None).is_none());
    }

    #[test]
    fn test_an_identifier_that_already_has_its_brackets_does_not_get_a_second_pair() {
        // The cache stores identifiers bare and a saved draft stores its own
        // already wrapped, so both spellings reach this.
        let chain = continuing("<c@x>", Some("<a@x>")).expect("a chain");
        assert_eq!(chain.in_reply_to, "<c@x>");
        assert_eq!(chain.references, "<a@x> <c@x>");
    }

    #[test]
    fn test_an_empty_references_column_reads_as_no_chain() {
        // An old row stores an empty string where a newer one stores nothing.
        for stored in ["", "   "] {
            let chain = continuing("c@x", Some(stored)).expect("a chain");
            assert_eq!(chain.references, "<c@x>", "from {stored:?}");
        }
    }

    #[test]
    fn test_a_chain_is_split_on_whatever_whitespace_it_was_stored_with() {
        // Stored space separated, and a header can carry line breaks after
        // folding, so anything blank between identifiers separates them.
        let chain = continuing("c@x", Some("a@x\r\n  b@x")).expect("a chain");
        assert_eq!(chain.references, "<a@x> <b@x> <c@x>");
    }

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
    fn test_depth_of_treats_a_parent_pointing_to_itself_as_a_root() {
        // No real caller can produce this: `thread_messages` never inserts an
        // entry whose parent is its own id (the `find` there filters out
        // `candidate == message.id`). Built directly because a self-loop is
        // exactly the shape the `parent != current` guard exists to stop
        // cold rather than count as depth.
        let parents = HashMap::from([(1, Some(1)), (2, Some(3))]);
        assert_eq!(depth_of(1, &parents), 0);
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
    fn test_find_stops_at_a_parent_pointing_to_itself_rather_than_spinning() {
        // `union` can never insert an entry that maps a key to itself: the
        // `if root_a != root_b` guard there rules it out before any insert
        // happens. This shape only exists here because it is built by hand,
        // which is the only way to drive `find` at the exact state where a
        // comparison-operator slip on either of its two loops turns into a
        // hang instead of a wrong answer.
        let mut set = DisjointSet::new();
        set.parent.insert(1, 1);

        assert_eq!(set.find(1), 1);
    }

    #[test]
    fn test_find_compresses_a_two_hop_chain_straight_to_the_root() {
        // `find`'s return value cannot tell this apart on its own: the first
        // loop in `find` always walks the current chain to the true root
        // whether or not the second (compression) loop ever ran. Only the
        // stored parent map shows whether the walk was shortened for the
        // next lookup, which is the entire point of doing it, so this reads
        // the map directly rather than only asserting what `find` returns.
        let mut set = DisjointSet::new();
        set.union(2, 1); // parent: {2 -> 1}
        set.union(1, 0); // parent: {2 -> 1, 1 -> 0}

        assert_eq!(set.find(2), 0);
        assert_eq!(
            set.parent.get(&2),
            Some(&0),
            "the two-hop chain 2 -> 1 -> 0 should now read 2 -> 0 directly"
        );
    }

    #[test]
    fn test_merging_two_conversations_keeps_the_smaller_row_as_the_root() {
        // Which of the two representatives survives is not visible in what
        // `thread_messages` returns, so this asks the set directly. Both
        // orders, because a rule that only holds when the larger id is named
        // first is not a rule.
        let mut set = DisjointSet::new();

        set.union(5, 3);
        assert_eq!(set.find(5), 3);
        assert_eq!(set.find(3), 3);

        set.union(2, 9);
        assert_eq!(set.find(9), 2);
        assert_eq!(set.find(2), 2);
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

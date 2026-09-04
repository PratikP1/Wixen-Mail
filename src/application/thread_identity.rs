//! Which conversation a message belongs to, answered from that message alone.
//!
//! # What was wrong
//!
//! `messages.thread_id` shipped as a column nothing ever wrote. Its only
//! mention in the whole data layer was the `ensure_column_exists` that creates
//! it, so every row held NULL, while `message_columns.rs` sorted the Thread
//! column on `m.thread_id` and therefore sorted every row by the same nothing
//! without failing. A column with a name, a sort expression and no writer reads
//! at every call site like working infrastructure.
//!
//! The other half of the failure was that the conversation id we did compute
//! moved. [`crate::application::threading::thread_messages`] names a
//! conversation after the least `Message-ID` among the messages it was handed,
//! which is stable for one batch and only for that batch: the same conversation
//! is named one thing in Inbox and another in Archive, and a message arriving
//! with a smaller identifier renames a conversation under somebody who is
//! reading it. Anything storing state against that id disagrees with the next
//! recompute.
//!
//! # The rule
//!
//! A conversation is named by the root its `References` chain points at. The
//! chain is stored oldest first, which is what
//! [`crate::application::threading::as_stored`] writes and what both syncs and
//! the filing path put there, so the root is its first identifier. A message
//! with no chain is its own root, which is how a conversation of one gets an id
//! like every other.
//!
//! This function is handed one message and cannot see a batch. That is the
//! whole design: batch independence is not a property somebody has to remember
//! to preserve, it is a thing the signature makes unavailable to get wrong.
//!
//! # Two answers coexist, and this one is authoritative for what is stored
//!
//! [`crate::application::threading::thread_messages`] still runs, in memory,
//! over one folder's loaded page, and still builds the parent links and depths
//! the conversation view reads. It is untouched. What it must not be used for
//! is a value that gets written down or compared across folders. The stored
//! `thread_id` comes from here and only from here.

/// The conversation this message belongs to.
///
/// The first identifier of the `References` chain, which is the root the
/// sender said this descends from, or the message's own identifier when it
/// names no ancestors. Identifiers come back bare, the way the cache stores
/// them.
///
/// Empty when the message has neither a chain nor an identifier of its own.
/// Whether to store a row like that is the caller's decision, and returning an
/// empty string leaves it in one place rather than making every caller unwrap
/// an `Option` that is almost never `None`.
pub fn conversation_root(message_id: &str, refs_header: Option<&str>) -> String {
    let named_root = refs_header
        .unwrap_or_default()
        .split_whitespace()
        .map(bare)
        .find(|identifier| !identifier.is_empty());

    match named_root {
        Some(root) => root.to_string(),
        // Skipping empty identifiers above rather than taking the chain's
        // first token whatever it is: `In-Reply-To: <>` is a real thing
        // senders write, and `threading::continuing` already refuses to write
        // it for the same reason. A chain of nothing but those names no root.
        None => bare(message_id).to_string(),
    }
}

/// One identifier with any angle brackets taken off.
///
/// **This is the one spelling a message identifier is stored in.** Every writer
/// goes through it, because `MessageCache::upsert_message` applies it on the
/// way into the parameters rather than each caller applying it first, and the
/// one lookup that asks the column by value applies it to the question. A
/// second writer added later gets the rule without knowing about it, and that
/// is the point: for a while the column held two spellings, because mail
/// arriving through `mail_parser` had the brackets stripped for it and a draft
/// this program filed did not.
///
/// The brackets are not wrong, they are just not part of the identifier. They
/// are the header's syntax, which is why
/// [`crate::application::draft_message::message_id_for`] and
/// [`crate::application::message_id::derived`] both still build them: those
/// values go on the wire. This is what goes in the column.
///
/// The same trimming [`crate::application::threading::bracketed`] does before
/// it puts one pair back on. Spelled again here rather than shared because
/// that function is private and returns the wrapped form, which is the
/// opposite of what a stored identifier needs, and because the module it lives
/// in is deliberately not touched by this change.
pub(crate) fn bare(identifier: &str) -> &str {
    identifier
        .trim()
        .trim_start_matches('<')
        .trim_end_matches('>')
}

/// How many identifiers out of one message are ever asked about at once.
///
/// T-01-57. A `References` chain is written by a stranger and has no length
/// limit, and the lookup this bounds sends one bound parameter per identifier.
/// SQLite refuses a statement with more than 999 of those, so an unbounded
/// chain is not a slow query, it is a query that fails, and the message it
/// arrived with would fail to store at all.
///
/// Sixty-four rather than the limit itself, because the useful identifiers are
/// few: the root, which names the conversation, and the nearest ancestors,
/// which are the ones likely to be held already. Everything between them is
/// covered by whichever of those two is stored.
pub const MOST_IDENTIFIERS_ASKED_ABOUT: usize = 64;

/// Which identifiers to ask the cache about when this message arrives.
///
/// The message's own identifier and its chain, bare, with empties dropped and
/// each named once. The order is the chain's, oldest first, with the message
/// itself last, so a reader can see that the root comes first.
///
/// Empty identifiers are dropped rather than passed on, and that is not
/// tidiness. The cache stores "this message had no identifier" as an empty
/// string, so asking about one would match every such message in the account
/// and merge a stranger's unrelated mail into this conversation.
/// [`crate::data::message_cache::MessageCache::message_ids_in_folder`] already
/// excludes them for the same reason.
///
/// Never more than [`MOST_IDENTIFIERS_ASKED_ABOUT`]. When a chain is longer
/// than that, the root is kept and the tail is kept, and the middle is
/// dropped: the root is what names the conversation and the tail is the
/// nearest ancestors, so those are the two ends worth asking about.
pub fn identifiers_worth_asking_about(message_id: &str, refs_header: Option<&str>) -> Vec<String> {
    let mut asking: Vec<String> = Vec::new();
    let named = refs_header
        .unwrap_or_default()
        .split_whitespace()
        .chain(std::iter::once(message_id));
    for identifier in named.map(bare).filter(|id| !id.is_empty()) {
        if !asking.iter().any(|held| held == identifier) {
            asking.push(identifier.to_string());
        }
    }

    if asking.len() > MOST_IDENTIFIERS_ASKED_ABOUT {
        // The root, then the tail, with the middle dropped. Drained in place
        // rather than rebuilt, so a chain of a hundred thousand identifiers
        // costs one pass and no second allocation of its own size.
        asking.drain(1..=asking.len() - MOST_IDENTIFIERS_ASKED_ABOUT);
    }
    asking
}

/// What an arriving message reveals about conversations already stored.
///
/// `winning_root` is the conversation everything settles on and
/// `roots_to_rewrite` are the conversations whose messages must be moved onto
/// it. Never empty, and never holds the winner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rerooting {
    /// The conversation the arriving message names, which is the one that wins.
    pub winning_root: String,
    /// The conversations revealed to be the same one, to be rewritten onto it.
    pub roots_to_rewrite: Vec<String>,
}

/// Which stored conversations an arriving message reveals to be one, or `None`
/// when it reveals nothing.
///
/// `the_arriving_conversation` is [`conversation_root`]'s answer for this
/// message. `conversations_found` are the conversations the cache already
/// files the identifiers of [`identifiers_worth_asking_about`] under.
///
/// # Why this is handed the answer rather than working it out
///
/// Pitfall 6, and D-39 before it. The defect this whole module exists against
/// is an incremental join that adopts a found conversation's id, because that
/// id came from somewhere else and disagrees with what the derivation says.
/// Taking `the_arriving_conversation` as a parameter is what makes adoption
/// unavailable: this function is never shown a candidate it could prefer over
/// the derived one, so the winner cannot be anything else. That is the same
/// trick [`conversation_root`] plays with the batch, in the same place, for the
/// same reason: a property the signature enforces is not one somebody has to
/// remember.
///
/// The chain is oldest first, so the conversation the arriving message names is
/// the chain's earliest identifier, and the earliest is the winner. Merging is
/// therefore not a new rule; it is [`conversation_root`]'s rule applied where
/// more than one candidate turns out to exist.
///
/// # What a merge costs, said plainly
///
/// One of the two conversations is renamed, and there is no way to merge two
/// trees without that. D-39 forbids a conversation being renamed by an arrival
/// that reveals nothing, which is what the rejected batch rule did every time a
/// smaller identifier turned up. It cannot forbid the rename that *is* the
/// merge.
pub fn rejoin(
    the_arriving_conversation: &str,
    conversations_found: &[String],
) -> Option<Rerooting> {
    // A message with no conversation of its own names no winner, and rewriting
    // other people's conversations onto nothing would empty them.
    if the_arriving_conversation.is_empty() {
        return None;
    }

    let mut roots_to_rewrite: Vec<String> = Vec::new();
    for found in conversations_found {
        let worth_rewriting = !found.is_empty()
            && found != the_arriving_conversation
            && !roots_to_rewrite.iter().any(|held| held == found);
        if worth_rewriting {
            roots_to_rewrite.push(found.clone());
        }
    }

    (!roots_to_rewrite.is_empty()).then(|| Rerooting {
        winning_root: the_arriving_conversation.to_string(),
        roots_to_rewrite,
    })
}

#[cfg(test)]
mod tests {
    use super::{Rerooting, conversation_root, identifiers_worth_asking_about, rejoin};

    #[test]
    fn a_message_with_no_chain_is_its_own_conversation() {
        assert_eq!(conversation_root("a@x", None), "a@x");
    }

    #[test]
    fn a_reply_is_named_after_the_root_of_its_chain() {
        assert_eq!(conversation_root("c@x", Some("a@x b@x")), "a@x");
    }

    #[test]
    fn the_answer_does_not_depend_on_which_other_messages_are_present() {
        // The conversation: a started it, b answered a, c answered b. Feeding
        // the same three in three different orders must produce the same three
        // answers, because the function is never shown the other two.
        let conversation = [
            ("a@x", None),
            ("b@x", Some("a@x")),
            ("c@x", Some("a@x b@x")),
        ];

        let orders = [[0, 1, 2], [2, 1, 0], [1, 2, 0]];
        let mut seen: Vec<Vec<(&str, String)>> = Vec::new();
        for order in orders {
            let mut answers: Vec<(&str, String)> = order
                .iter()
                .map(|&i| {
                    let (id, chain) = conversation[i];
                    (id, conversation_root(id, chain))
                })
                .collect();
            answers.sort();
            seen.push(answers);
        }

        assert_eq!(seen[0], seen[1]);
        assert_eq!(seen[1], seen[2]);
        // And all three are the conversation's actual root, not the least id
        // in whichever order it was fed.
        for (_, root) in &seen[0] {
            assert_eq!(root, "a@x");
        }
    }

    #[test]
    fn an_arriving_message_with_a_smaller_identifier_renames_nothing() {
        // This is the failure of the batch rule, written as a test. Under
        // "least Message-ID in the batch", aaa@x arriving would rename the
        // conversation from m@x to aaa@x. Here it joins m@x's conversation and
        // m@x's own answer is not consulted, let alone changed.
        assert_eq!(conversation_root("aaa@x", Some("m@x")), "m@x");
        assert_eq!(conversation_root("m@x", None), "m@x");
    }

    #[test]
    fn a_chain_that_arrived_wrapped_in_angle_brackets_still_names_a_bare_root() {
        assert_eq!(conversation_root("c@x", Some("<a@x> <b@x>")), "a@x");
    }

    #[test]
    fn an_empty_chain_falls_back_to_the_messages_own_identifier() {
        assert_eq!(conversation_root("a@x", Some("")), "a@x");
        assert_eq!(conversation_root("a@x", Some("   \t \n ")), "a@x");
        // A chain of nothing but empty brackets is malformed, not a root.
        assert_eq!(conversation_root("a@x", Some("<>")), "a@x");
    }

    #[test]
    fn a_message_with_no_identifier_at_all_answers_empty_rather_than_panicking() {
        // Deciding not to store this is the caller's, not ours. Returning an
        // empty string keeps that decision in one place instead of making
        // every caller handle an Option that is almost never None.
        assert_eq!(conversation_root("", None), "");
        assert_eq!(conversation_root("", Some("")), "");
        assert_eq!(conversation_root("   ", Some("  ")), "");
    }

    #[test]
    fn the_identifier_is_returned_bare_and_trimmed_the_way_the_cache_stores_it() {
        assert_eq!(conversation_root("  a@x  ", None), "a@x");
        assert_eq!(conversation_root("<a@x>", None), "a@x");
    }

    #[test]
    fn a_hostile_chain_is_carried_through_rather_than_interpreted() {
        // T-01-05. A sender writes the `References` header, so this is
        // attacker-controlled for any message a stranger can send. The id is
        // only ever a grouping key, never a permission, a path or a filename,
        // and it reaches SQL as a bound parameter. So the rule here is that
        // nothing is interpreted: whatever the first identifier is, that is
        // the answer, and no quoting, escaping or truncation happens on the
        // way.
        assert_eq!(
            conversation_root("c@x", Some("'; DROP TABLE messages;--")),
            "';"
        );
        assert_eq!(conversation_root("c@x", Some("a\"b@x c@x")), "a\"b@x");
        // A newline is whitespace, so it separates rather than smuggling.
        assert_eq!(conversation_root("c@x", Some("a@x\nb@x")), "a@x");
    }

    #[test]
    fn only_the_first_identifier_of_a_very_long_chain_is_read() {
        // T-01-06. Chain length must not drive work: an unbounded
        // `References` header is free for a sender to write.
        let mut chain = String::from("root@x");
        for i in 0..100_000 {
            chain.push(' ');
            chain.push_str(&format!("filler{i}@x"));
        }
        assert_eq!(conversation_root("last@x", Some(&chain)), "root@x");
    }

    // ── Which identifiers get asked about ───────────────────────────────────

    #[test]
    fn the_message_and_every_identifier_of_its_chain_are_asked_about() {
        assert_eq!(
            identifiers_worth_asking_about("c@x", Some("a@x b@x")),
            vec!["a@x", "b@x", "c@x"]
        );
    }

    #[test]
    fn a_message_with_no_chain_is_asked_about_by_itself() {
        assert_eq!(identifiers_worth_asking_about("a@x", None), vec!["a@x"]);
    }

    #[test]
    fn the_root_comes_first_so_the_conversation_it_names_is_readable() {
        let asking = identifiers_worth_asking_about("c@x", Some("<a@x> <b@x>"));
        assert_eq!(asking.first().map(String::as_str), Some("a@x"));
        assert_eq!(
            asking.first().map(String::as_str),
            Some(conversation_root("c@x", Some("<a@x> <b@x>")).as_str()),
            "the first identifier asked about must be the one conversation_root names, \
             or the two halves of this module disagree about which is the root"
        );
    }

    #[test]
    fn brackets_come_off_the_way_the_cache_stores_them() {
        assert_eq!(
            identifiers_worth_asking_about("<c@x>", Some("<a@x>")),
            vec!["a@x", "c@x"]
        );
    }

    #[test]
    fn an_identifier_named_twice_is_asked_about_once() {
        // A chain that repeats its root, which senders do write. Two copies
        // would be two bound parameters answering the same question.
        assert_eq!(
            identifiers_worth_asking_about("a@x", Some("a@x b@x a@x")),
            vec!["a@x", "b@x"]
        );
    }

    #[test]
    fn an_empty_identifier_is_never_asked_about_and_a_real_one_beside_it_still_is() {
        // The presence half is what makes the absence half mean anything: a
        // body returning nothing would satisfy "no empty identifier" for free.
        let asking = identifiers_worth_asking_about("c@x", Some("<> a@x <>"));
        assert!(
            asking.iter().any(|id| id == "a@x"),
            "the real identifier must still be asked about: {asking:?}"
        );
        assert!(
            !asking.iter().any(|id| id.is_empty()),
            "an empty identifier matches every message the cache holds with no \
             identifier of its own, so asking about one merges strangers' mail: {asking:?}"
        );
    }

    #[test]
    fn a_message_with_nothing_to_identify_it_asks_about_nothing() {
        assert!(identifiers_worth_asking_about("", None).is_empty());
        assert!(identifiers_worth_asking_about("  ", Some("<> <>")).is_empty());
    }

    #[test]
    fn a_chain_longer_than_the_cap_is_asked_about_at_both_ends_and_not_the_middle() {
        // T-01-57. The cap is what keeps the lookup inside SQLite's limit on
        // bound parameters, and the two ends are the useful identifiers.
        let mut chain = String::from("root@x");
        for i in 0..5_000 {
            chain.push(' ');
            chain.push_str(&format!("filler{i}@x"));
        }
        let asking = identifiers_worth_asking_about("last@x", Some(&chain));

        assert_eq!(
            asking.len(),
            super::MOST_IDENTIFIERS_ASKED_ABOUT,
            "the lookup must stay inside SQLite's limit on bound parameters"
        );
        assert_eq!(
            asking.first().map(String::as_str),
            Some("root@x"),
            "the root names the conversation, so it is the one identifier that \
             must survive any capping"
        );
        assert_eq!(
            asking.last().map(String::as_str),
            Some("last@x"),
            "the message itself is the nearest thing there is to a near ancestor"
        );
        assert!(
            asking.iter().any(|id| id == "filler4999@x"),
            "the tail is the nearest ancestors and is the half most likely to be held"
        );
        assert!(
            !asking.iter().any(|id| id == "filler0@x"),
            "the middle is what gets dropped: {} identifiers",
            asking.len()
        );
    }

    // ── The merge ───────────────────────────────────────────────────────────

    #[test]
    fn a_message_naming_no_stored_conversation_merges_nothing() {
        assert_eq!(rejoin("a@x", &[]), None);
    }

    #[test]
    fn a_message_joining_the_conversation_it_already_names_rewrites_nothing() {
        assert_eq!(rejoin("a@x", &["a@x".to_string()]), None);
    }

    #[test]
    fn a_chain_naming_two_stored_conversations_merges_them_onto_the_one_it_names_first() {
        // THREAD-02's own case. `a@x` and `c@x` each started a conversation;
        // the arriving message says they are one, and its chain says `a@x` is
        // the root because the chain is oldest first.
        assert_eq!(
            rejoin("a@x", &["a@x".to_string(), "c@x".to_string()]),
            Some(Rerooting {
                winning_root: "a@x".to_string(),
                roots_to_rewrite: vec!["c@x".to_string()],
            })
        );
    }

    #[test]
    fn three_conversations_connected_by_one_message_leave_two_to_rewrite() {
        let merged = rejoin(
            "a@x",
            &["c@x".to_string(), "a@x".to_string(), "d@x".to_string()],
        )
        .expect("three conversations named, so two of them move");
        assert_eq!(merged.winning_root, "a@x");
        assert_eq!(merged.roots_to_rewrite, vec!["c@x", "d@x"]);
    }

    #[test]
    fn the_winner_is_the_earliest_identifier_the_merging_conversations_know_between_them() {
        // This asserted the opposite until 03-05: that the arriving message's
        // own conversation wins whatever it is merged with. The argument for
        // that was that a chain is oldest first, so the conversation an
        // arriving message names is the chain's earliest identifier. True of a
        // message carrying a full chain, and not true of one naming only its
        // parent, where the arriving conversation is a message in the middle.
        // The old rule was therefore right about the common case, had no answer
        // for the other, and gave different answers depending on which message
        // arrived first. See `rejoin`'s comment.
        let merged = rejoin("m@x", &["aaa@x".to_string(), "m@x".to_string()])
            .expect("two conversations named, so one moves");
        assert_eq!(merged.winning_root, "aaa@x");
        assert_eq!(merged.roots_to_rewrite, vec!["m@x"]);
    }

    #[test]
    fn a_conversation_found_twice_is_rewritten_once() {
        // Two of the chain's identifiers are stored under the same
        // conversation, which is the ordinary case for a thread of any length.
        let merged = rejoin("a@x", &["c@x".to_string(), "c@x".to_string()])
            .expect("one other conversation named");
        assert_eq!(merged.roots_to_rewrite, vec!["c@x"]);
    }

    #[test]
    fn an_empty_conversation_found_is_not_something_to_rewrite() {
        // The presence half again: without it, a body returning nothing would
        // pass the absence assertion on its own.
        let merged = rejoin("a@x", &[String::new(), "c@x".to_string()])
            .expect("c@x is a real conversation and must still be merged");
        assert_eq!(merged.roots_to_rewrite, vec!["c@x"]);
    }

    #[test]
    fn a_message_with_no_conversation_of_its_own_merges_nothing() {
        // conversation_root answers empty for a message with neither a chain
        // nor an identifier. Rewriting other people's conversations onto
        // nothing would empty them.
        assert_eq!(rejoin("", &["a@x".to_string(), "c@x".to_string()]), None);
    }

    #[test]
    fn applying_the_answer_and_asking_again_leaves_nothing_to_do() {
        let first = rejoin("a@x", &["a@x".to_string(), "c@x".to_string()])
            .expect("the merge this is the second half of");
        // What the cache holds after the rewrite: everything that was `c@x` is
        // now `a@x`, so the same lookup comes back naming one conversation.
        let after: Vec<String> = ["a@x", "c@x"]
            .iter()
            .map(|found| {
                if first.roots_to_rewrite.iter().any(|old| old == found) {
                    first.winning_root.clone()
                } else {
                    (*found).to_string()
                }
            })
            .collect();
        assert_eq!(rejoin("a@x", &after), None);
    }

    // ── Order independence ──────────────────────────────────────────────────

    /// The storage path, in memory: store each message in turn, giving it
    /// [`conversation_root`]'s answer, writing down every identifier it names,
    /// and applying [`rejoin`] against the conversations those names are held
    /// under.
    ///
    /// Deliberately the same shape as
    /// [`crate::data::message_cache::MessageCache::upsert_message`], including
    /// what it can and cannot see. The third field is
    /// `identifiers_a_message_names`: without it the only question that could
    /// be asked was whether a stored message's own identifier was one of these,
    /// which is what left a conversation root arriving late out of the merge.
    ///
    /// The row is pushed before the lookup, which is the order the real path
    /// runs in: the message is inserted, its names are recorded, and only then
    /// is the cache asked.
    fn assign_in_this_order(
        conversation: &[(&str, Option<&str>)],
        order: &[usize],
    ) -> Vec<(String, String)> {
        // The message, the conversation it is in, and every identifier it named.
        let mut held: Vec<(String, String, Vec<String>)> = Vec::new();
        for &i in order {
            let (message_id, refs_header) = conversation[i];
            let mine = conversation_root(message_id, refs_header);
            let asking = identifiers_worth_asking_about(message_id, refs_header);
            held.push((message_id.trim().to_string(), mine.clone(), asking.clone()));
            let found: Vec<String> = held
                .iter()
                .filter(|(_, _, named)| named.iter().any(|name| asking.contains(name)))
                .map(|(_, thread, _)| thread.clone())
                .collect();
            if let Some(merge) = rejoin(&mine, &found) {
                for (_, thread, _) in held.iter_mut() {
                    if merge.roots_to_rewrite.iter().any(|old| old == thread) {
                        *thread = merge.winning_root.clone();
                    }
                }
            }
        }
        let mut rows: Vec<(String, String)> = held
            .into_iter()
            .map(|(message, thread, _)| (message, thread))
            .collect();
        rows.sort();
        rows
    }

    #[test]
    fn every_arrival_order_agrees_when_each_message_names_its_own_root() {
        // The ordinary conversation: a started it, b answered a, c answered b,
        // and a late message names both a and b. Every order of arrival must
        // end with the same four rows in the same conversation.
        let conversation = [
            ("a@x", None),
            ("b@x", Some("a@x")),
            ("c@x", Some("a@x b@x")),
            ("d@x", Some("a@x b@x c@x")),
        ];
        let orders = [
            [0, 1, 2, 3],
            [3, 2, 1, 0],
            [2, 0, 3, 1],
            [1, 3, 0, 2],
            [3, 0, 2, 1],
        ];

        let first = assign_in_this_order(&conversation, &orders[0]);
        assert_eq!(first.len(), 4);
        for (_, thread) in &first {
            assert_eq!(thread, "a@x", "the whole conversation is named after a@x");
        }
        for order in &orders[1..] {
            assert_eq!(
                assign_in_this_order(&conversation, order),
                first,
                "arrival order {order:?} disagreed with {:?}",
                orders[0]
            );
        }
    }

    #[test]
    fn a_late_message_merging_two_conversations_agrees_in_every_arrival_order() {
        // THREAD-02's merge and the gap phase 1 deferred, in one test. `a@x`
        // and `c@x` each started a conversation and `x@x` says they are one.
        //
        // Three of these six used to end with the conversation still split,
        // because nothing recorded what a message named and the only question
        // that could be asked was what a stored message was called. The other
        // three merged, and then settled under two different names depending on
        // which message arrived last, so this used to be written as the three
        // orders that could see it and a companion pinning the gap.
        let conversation = [("a@x", None), ("c@x", None), ("x@x", Some("a@x c@x"))];
        let orders = [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ];

        let first = assign_in_this_order(&conversation, &orders[0]);
        assert_eq!(first.len(), 3);
        for (message, thread) in &first {
            assert_eq!(
                thread, "a@x",
                "{message} was left out of the merged conversation"
            );
        }
        for order in &orders[1..] {
            assert_eq!(
                assign_in_this_order(&conversation, order),
                first,
                "arrival order {order:?} disagreed with {:?}",
                orders[0]
            );
        }
    }

    #[test]
    fn a_chain_naming_only_its_parent_agrees_in_every_arrival_order() {
        // The case the old winner rule had no answer for, and the reason it had
        // to change. `r@x` started a conversation, `p@x` answered it carrying
        // the whole chain, and `n@x` answers `p@x` naming only its parent,
        // which is what a client sending `In-Reply-To` and no `References`
        // produces.
        //
        // So `n@x`'s own root is `p@x`, a message in the middle rather than the
        // conversation's head. Under "the arriving message's conversation
        // wins", `n@x` arriving last renamed the whole conversation to `p@x`
        // and `n@x` arriving first left it as `r@x`, and both are the same
        // three messages.
        let conversation = [("r@x", None), ("p@x", Some("r@x")), ("n@x", Some("p@x"))];
        let orders = [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ];

        let first = assign_in_this_order(&conversation, &orders[0]);
        assert_eq!(first.len(), 3);
        let names: std::collections::BTreeSet<&str> =
            first.iter().map(|(_, thread)| thread.as_str()).collect();
        assert_eq!(
            names.len(),
            1,
            "the three belong in one conversation and are in {names:?}"
        );
        for order in &orders[1..] {
            assert_eq!(
                assign_in_this_order(&conversation, order),
                first,
                "arrival order {order:?} disagreed with {:?}",
                orders[0]
            );
        }
    }

    #[test]
    fn an_arrival_that_reveals_nothing_renames_nothing_however_its_identifier_sorts() {
        // D-39, and what stops the winner rule being the rejected batch rule
        // wearing a different hat. `aaa@x` answers `m@x` and sorts before it,
        // so a rule applying "earliest" to anything other than conversations an
        // arrival has PROVED to be one would rename a conversation somebody is
        // reading and reveal nothing by doing it.
        assert_eq!(conversation_root("aaa@x", Some("m@x")), "m@x");
        assert_eq!(rejoin("m@x", &["m@x".to_string()]), None);

        // The same thing through the storing path, which is where it would
        // really happen.
        let conversation = [("m@x", None), ("aaa@x", Some("m@x"))];
        assert_eq!(
            assign_in_this_order(&conversation, &[0, 1]),
            vec![
                ("aaa@x".to_string(), "m@x".to_string()),
                ("m@x".to_string(), "m@x".to_string()),
            ],
            "an arrival naming the conversation it is already in renamed it"
        );
    }
}

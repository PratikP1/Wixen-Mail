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
/// The same trimming [`crate::application::threading::bracketed`] does before
/// it puts one pair back on. Spelled again here rather than shared because
/// that function is private and returns the wrapped form, which is the
/// opposite of what a stored identifier needs, and because the module it lives
/// in is deliberately not touched by this change.
fn bare(identifier: &str) -> &str {
    identifier
        .trim()
        .trim_start_matches('<')
        .trim_end_matches('>')
}

#[cfg(test)]
mod tests {
    use super::conversation_root;

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
}

//! Which folders somebody has pinned to the top of their tree, and what a pin
//! means.
//!
//! FOLDER-03, and D-28 to D-32. A pin makes a **copy**: the folder stays in its
//! account's branch and appears in the group at the top as well. Unpinning can
//! then lose nothing, and the tree somebody learned does not change under them.
//!
//! # A pin and a subscription are two questions, and each has one answer
//!
//! FOLDER-03 asks for this to be written down **before** the second half is
//! built, rather than settled by whichever code path runs last, and it names the
//! reason: two answers to one question is a shape this project has been bitten
//! by. So, plainly, for a later phase to obey rather than to invent:
//!
//! > A pin says where a folder sits in this program's tree, on this computer. A
//! > subscription says which mailboxes an account asks its server to list.
//! > Pinning never writes a subscription, and a subscription changing never adds
//! > or removes a pin.
//!
//! Neither has to beat the other, because they are not one question:
//!
//! - **Pinning cannot write a subscription.** FOLDER-03 forbids reaching a
//!   server here at all, and a POP account has no subscriptions to write in the
//!   first place. A preference that sometimes opened a connection would cost
//!   somebody a round trip on a metered or watched network for rearranging their
//!   own sidebar.
//! - **A subscription cannot take a pin away.** It is shared with every other
//!   mail program that account is opened in, so letting it remove a pin is
//!   another program editing somebody's sidebar while they are not looking. A
//!   pinned folder that stops being subscribed keeps its pin and its row says
//!   so, which is the answer D-32 already gives for a folder the server has
//!   stopped listing.
//!
//! If a later phase wants the two joined, that is a setting somebody turns on,
//! gated like every other server write, and never a side effect of pinning.
//!
//! # Why nothing has to move when the server half arrives
//!
//! A pin is stored against `(account_id, path)`. That is the pair `folders` is
//! already unique on, the pair `imap::set_subscribed` names a mailbox by, and
//! the row `folders.subscribed` already sits on. So the local answer and the
//! server answer are one row apart from the day this is written, and a later
//! phase joins them without moving anything. Storing a folder's row id instead
//! would have worked as well for this program and left that join spelling out an
//! id nothing on the wire has ever heard of.
//!
//! # Nothing here reaches a server
//!
//! Not by intention but by measurement: the check at the bottom of this file
//! reads the shipping half of every file on the pinning path and fails on a call
//! that leaves this machine, and its companion proves the reading can see one.
//! That is the same shape `account_order` uses for the same reason, and for the
//! reason it gives: what is being defended is an absence, and an absence leaves
//! nothing for a behaviour test to count.

/// What the group of pinned folders is called.
///
/// One spelling, defined here and read by the tree. Two spellings of one group
/// is a heading that stops matching the sentence somebody heard a moment ago.
///
/// The British spelling, because every other word this program says to a person
/// is British: "Favorites" appears in the contacts module because that is a
/// vCard field name from the format, which is not a word anybody is shown.
pub const FAVOURITES: &str = "Favourites";

/// One pinned folder.
///
/// `account` and `path` together are the folder, and are exactly the pair D-25
/// makes a folder's stable identity and `folders` is unique on. Never a label:
/// a label carries a name somebody can change and an unread count that changes
/// on its own, and both are the moments a pin is supposed to stay put.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    pub account: String,
    pub path: String,
    /// Where this pin sits among its own account's pins, counting from nought.
    ///
    /// Per account rather than across the whole group, because the group is
    /// arranged by account (D-29) and a number spanning every account would
    /// renumber one account's pins when another account gained one.
    pub position: i64,
}

#[cfg(test)]
mod nothing_here_reaches_a_server {
    /// Every file on the path that pins a folder.
    ///
    /// The whole path rather than this module alone. This module decides
    /// nothing on its own; a call that left this machine would be written where
    /// the pin is stored or where the command is answered, and a check reading
    /// only the module named after the feature would never look at either.
    ///
    /// `wx_app.rs` is read for one function rather than whole, and joins this
    /// list in plan 01-08's third task, where the command is written.
    const THE_WHOLE_PATH: [&str; 2] = [
        "src/application/favourites.rs",
        "src/data/message_cache/folders.rs",
    ];

    /// How a call out of this machine is spelled, in call syntax.
    ///
    /// Call syntax rather than bare words, because the paragraphs above name
    /// every one of the things this forbids. A check that fires on the
    /// explanation of its own rule is a check somebody switches off.
    const A_CALL_THAT_LEAVES_THIS_MACHINE: [&str; 5] = [
        "crate::service::",
        "reqwest::",
        "spawn_mail_sync(",
        "a_session_at(",
        "super::service::",
    ];

    fn calls_out_of_this_machine(text: &str) -> Vec<String> {
        text.lines()
            .enumerate()
            .filter(|(_, line)| {
                A_CALL_THAT_LEAVES_THIS_MACHINE
                    .iter()
                    .any(|call| line.contains(call))
            })
            .map(|(number, line)| format!("{}: {}", number + 1, line.trim()))
            .collect()
    }

    #[test]
    fn test_pinning_a_folder_makes_no_call_that_leaves_this_machine() {
        let found: Vec<String> = THE_WHOLE_PATH
            .iter()
            .flat_map(|path| {
                let source = std::fs::read_to_string(path)
                    .unwrap_or_else(|_| panic!("{path} to be readable"));
                calls_out_of_this_machine(&crate::common::what_ships::what_ships(&source))
                    .into_iter()
                    .map(move |found| format!("{path} {found}"))
            })
            .collect();

        assert!(
            found.is_empty(),
            "{} call(s) that leave this machine sit on the path that pins a \
             folder. Pinning is a preference about this program's own sidebar: \
             FOLDER-03 forbids it reaching a server, a POP account has no \
             subscriptions to write, and a preference that opened a connection \
             would cost somebody a round trip for rearranging their own \
             tree:\n  {}",
            found.len(),
            found.join("\n  ")
        );
    }

    #[test]
    fn test_the_reading_can_see_such_a_call_when_there_is_one() {
        // Without this the test above passes just as well once the reading has
        // stopped reading anything, and a check that fails two ways without
        // saying which is worse than no check.
        assert_eq!(
            calls_out_of_this_machine(
                "pub fn pin_row(&self) {\n    \
                 crate::service::protocols::imap::a_session_at(host);\n}"
            )
            .len(),
            1
        );
        // And it does not fire on the paragraphs that explain the rule, which
        // name every one of these things in prose.
        assert!(
            calls_out_of_this_machine(
                "//! Pinning never reaches the service layer, never uses \
                 reqwest, and opens no imap, smtp or pop3 session."
            )
            .is_empty()
        );
    }

    #[test]
    fn test_the_files_this_reads_are_all_still_there() {
        // A file renamed or moved makes the reading above find nothing and
        // report a clean result over a path it never looked at.
        for path in THE_WHOLE_PATH {
            assert!(
                std::path::Path::new(path).exists(),
                "{path} has moved, so the check that reads it is reading nothing"
            );
        }
    }
}

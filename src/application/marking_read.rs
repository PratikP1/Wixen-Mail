//! What Mark as Read says, and what the letter M says.
//!
//! The tester on 2026-09-15 (#27): "The mark read command in the action menu
//! and the corresponding context menu should reflect the current status of
//! the message ... Use 'm' bound to message lists to toggle the state and
//! announcement, 'read'/'unread'." Until 2026-09-18 the command said "Mark as
//! Read" on every surface whatever the message's state, and toggled: on a
//! read message it marked unread while saying it would mark read. The words
//! for the two states are decided here, once, so the Action menu, the context
//! menu and the toolbar cannot come to say different things, and so what the
//! item says after the key is what the state is.
//!
//! One rule per word rather than four literals at four sites, for the reason
//! [`crate::application::context_menu`] gives for being data: strings that
//! drift drift invisibly, and the test that holds the mnemonic letter the same
//! whichever way the command goes can only be written where both forms are.

/// The words one state of the command is offered with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wording {
    /// The Action menu's item, carrying its mnemonic.
    pub menu: &'static str,
    /// The context menu's entry, in that menu's own case and carrying its own
    /// mnemonic, which is the letter it has always had there.
    pub context: &'static str,
    /// What the toolbar button says, and what the menu item is heard as: the
    /// menu word with no ampersand.
    pub spoken: &'static str,
    /// The sentence the tool's tip and the item's help carry.
    pub help: &'static str,
}

/// What the command is offered as, given whether the message it would act on
/// is unread.
///
/// `any_unread` is the message's own flag for a message row, and whether the
/// conversation holds any unread message for a conversation row, so the
/// command on a conversation row says what it will do to the messages that
/// need it.
pub const fn what_the_command_says(any_unread: bool) -> Wording {
    // The mnemonic stays on the e in both, so somebody who learned Alt+A, E
    // keeps it whichever way the command goes; the context menu keeps its M.
    if any_unread {
        Wording {
            menu: "Mark as R&ead",
            context: "&Mark as read",
            spoken: "Mark as Read",
            help: "Mark the selected message as read",
        }
    } else {
        Wording {
            menu: "Mark as Unr&ead",
            context: "&Mark as unread",
            spoken: "Mark as Unread",
            help: "Mark the selected message as unread",
        }
    }
}

/// The one word M says once it has toggled: the state the message is in now.
///
/// The tester's two words, and nothing else, because the row is still under
/// the cursor and a screen reader has already said whose message it is.
pub const fn what_the_key_says(now_read: bool) -> &'static str {
    if now_read { "read" } else { "unread" }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn without_the_ampersand(word: &str) -> String {
        word.replace('&', "")
    }

    fn the_mnemonic_of(word: &str) -> char {
        word.split('&')
            .nth(1)
            .and_then(|rest| rest.chars().next())
            .unwrap_or_else(|| panic!("{word:?} carries no mnemonic"))
            .to_ascii_lowercase()
    }

    #[test]
    fn test_an_unread_message_is_offered_mark_as_read() {
        let wording = what_the_command_says(true);
        assert_eq!(wording.spoken, "Mark as Read");
        assert_eq!(without_the_ampersand(wording.menu), "Mark as Read");
        assert_eq!(wording.help, "Mark the selected message as read");
    }

    #[test]
    fn test_a_read_message_is_offered_mark_as_unread() {
        let wording = what_the_command_says(false);
        assert_eq!(wording.spoken, "Mark as Unread");
        assert_eq!(without_the_ampersand(wording.menu), "Mark as Unread");
        assert_eq!(wording.help, "Mark the selected message as unread");
    }

    #[test]
    fn test_the_mnemonic_letter_is_the_same_whichever_way_the_command_goes() {
        // Somebody who learned the letter on the menu keeps it when the
        // message under the cursor happens to be read; a letter that moves
        // with the state is a letter that has to be listened for every time.
        assert_eq!(
            the_mnemonic_of(what_the_command_says(true).menu),
            the_mnemonic_of(what_the_command_says(false).menu)
        );
        assert_eq!(
            the_mnemonic_of(what_the_command_says(true).context),
            the_mnemonic_of(what_the_command_says(false).context)
        );
    }

    #[test]
    fn test_the_context_entry_says_the_same_words_in_its_own_case() {
        // The context menu writes its entries in sentence case and the Action
        // menu in title case, and that is all that may differ between them.
        for any_unread in [true, false] {
            let wording = what_the_command_says(any_unread);
            assert_eq!(
                without_the_ampersand(wording.context).to_lowercase(),
                without_the_ampersand(wording.menu).to_lowercase(),
                "the context entry and the menu item say different things"
            );
        }
    }

    #[test]
    fn test_the_key_says_the_state_the_message_is_in_now() {
        assert_eq!(what_the_key_says(true), "read");
        assert_eq!(what_the_key_says(false), "unread");
    }
}

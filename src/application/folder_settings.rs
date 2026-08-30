//! What the settings screen calls the group holding the folder and message
//! list settings.
//!
//! One string rather than one per screen, for the reason
//! [`crate::application::allowed::SETTINGS_SECTION`] gives: a sentence that
//! sends somebody to a section has to say the name they will then read on the
//! screen. That went wrong once already, where a sync said "Allow Changes" and
//! the section was headed "Allowed Changes", which is near enough to look like
//! the right place and far enough to make somebody stop and check.
//!
//! Nothing outside the settings screen names this group in a sentence today.
//! The constant is here so that the first thing which does reads it, rather
//! than typing the name a second time and starting the same drift, and
//! `test_no_settings_screen_writes_a_section_name_out_itself` in
//! `tests/house_style.rs` is what keeps the screen itself reading it.

/// What the settings screen calls the group of folder and message list
/// settings, on the Reading page.
///
/// The five settings under it are all about how the folder tree and the
/// message list behave, so they sit in one group in one place rather than
/// scattered down the page.
pub const SETTINGS_SECTION: &str = "Folders and Message Lists";

/// How a row that holds other rows says what is unread.
///
/// A folder with folders under it, an account branch, and the group of things
/// kept on this computer all have two numbers to give: what is unread in the
/// row itself, and what is unread in everything beneath it. Somebody who keeps
/// their branches closed can see only the second one, so a closed branch that
/// gave its own number alone would report nothing new while holding forty
/// unread messages.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum UnreadOnAParent {
    /// Both numbers, open or closed.
    ///
    /// The default, and the reason is that a row then means the same thing
    /// every time it is read. Somebody arrowing down a tree hears rows one
    /// after another, and a row whose wording depends on a state they have to
    /// remember is a row they have to stop and check.
    #[default]
    BothAlways,
    /// Both numbers while the row is closed, its own while it is open.
    ///
    /// For somebody who reads the children anyway when a branch is open, and
    /// would rather not hear the total repeated on the way past.
    BothWhenClosed,
}

impl UnreadOnAParent {
    /// How the setting stores itself, and reads back.
    pub const fn as_str(self) -> &'static str {
        match self {
            UnreadOnAParent::BothAlways => "both_always",
            UnreadOnAParent::BothWhenClosed => "both_when_closed",
        }
    }

    /// Read a stored setting.
    ///
    /// Anything unrecognised is both numbers always, which is the default. A
    /// settings file written by hand, or by a later version, should fall to the
    /// answer that says the most rather than to whichever branch happens to be
    /// written first.
    pub fn from_stored(stored: &str) -> Self {
        match stored.trim().to_ascii_lowercase().as_str() {
            "both_when_closed" => UnreadOnAParent::BothWhenClosed,
            _ => UnreadOnAParent::BothAlways,
        }
    }

    /// What the choice says on the settings screen.
    pub const fn words(self) -> &'static str {
        match self {
            UnreadOnAParent::BothAlways => "Both numbers, always",
            UnreadOnAParent::BothWhenClosed => "Both numbers only while it is closed",
        }
    }

    /// Read back what somebody chose, by the words they were shown.
    ///
    /// By the words rather than by the row number, for the reason `font_family`
    /// gives on the same screen: a row number means nothing without the list it
    /// counts into, and a list that differed between showing and saving would
    /// store a different answer than the one chosen.
    pub fn from_words(words: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|option| option.words() == words)
            .unwrap_or_default()
    }

    /// Both, so a chooser and its tests cover the set.
    pub const ALL: [UnreadOnAParent; 2] =
        [UnreadOnAParent::BothAlways, UnreadOnAParent::BothWhenClosed];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_settings_file_that_has_never_heard_of_this_gets_both_numbers() {
        // The control case: an empty string is what a settings file written
        // before this existed produces, and it must land on the default rather
        // than on whichever branch is written first.
        assert_eq!(
            UnreadOnAParent::from_stored(""),
            UnreadOnAParent::BothAlways
        );
        assert_eq!(UnreadOnAParent::default(), UnreadOnAParent::BothAlways);
    }

    #[test]
    fn test_the_other_option_reads_back_as_itself() {
        assert_eq!(
            UnreadOnAParent::from_stored("both_when_closed"),
            UnreadOnAParent::BothWhenClosed
        );
    }

    #[test]
    fn test_a_value_nobody_recognises_falls_to_both_numbers_rather_than_anywhere_else() {
        // Somebody hand-editing the settings file, or a file from a version
        // that had a third option. The answer that says the most is the safe
        // one to fall to.
        for written in ["sometimes", "BOTH_WHEN_CLOSE", "1", "  ", "never"] {
            assert_eq!(
                UnreadOnAParent::from_stored(written),
                UnreadOnAParent::BothAlways,
                "{written} should have fallen to the default"
            );
        }
    }

    #[test]
    fn test_what_is_stored_reads_back_as_what_was_stored() {
        for option in UnreadOnAParent::ALL {
            assert_eq!(UnreadOnAParent::from_stored(option.as_str()), option);
        }
    }

    #[test]
    fn test_the_words_shown_read_back_as_what_they_were_shown_for() {
        for option in UnreadOnAParent::ALL {
            assert_eq!(UnreadOnAParent::from_words(option.words()), option);
        }
    }

    #[test]
    fn test_words_from_a_list_that_is_not_this_one_fall_to_the_default() {
        assert_eq!(
            UnreadOnAParent::from_words("Date (Newest First)"),
            UnreadOnAParent::BothAlways
        );
    }

    #[test]
    fn test_the_two_options_do_not_say_the_same_thing() {
        // A chooser whose two rows read alike is a chooser nobody can use, and
        // `from_words` would answer the first of them for both.
        assert_ne!(
            UnreadOnAParent::BothAlways.words(),
            UnreadOnAParent::BothWhenClosed.words()
        );
        assert_ne!(
            UnreadOnAParent::BothAlways.as_str(),
            UnreadOnAParent::BothWhenClosed.as_str()
        );
    }
}

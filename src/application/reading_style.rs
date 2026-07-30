//! Which surface a message opens into.
//!
//! Two ways of reading the same message, and they are not equivalent.
//!
//! Formatted is a page. The sender's headings are headings, so `H` moves
//! between them; their links are links, so a screen reader can list them; a
//! table is a table with rows and columns. It is how the message was written.
//!
//! Plain is a text control. It has no structure at all: no headings, no links,
//! a table flattened into lines. What it has instead is a caret. Arrow keys
//! move by character, word and line, text can be selected and copied, and the
//! screen reader reports position continuously, which a rendered page does not
//! do in the same way.
//!
//! # Why formatted is the default
//!
//! Because it is what the message is. Flattening it throws away structure the
//! sender put there, and the person most affected by that is the one who cannot
//! see the layout that would otherwise stand in for it: without headings there
//! is no way to skim, and without links there is no way to list what a message
//! points at.
//!
//! Plain stays one setting away, because the caret is a real advantage for some
//! people and some messages, and neither surface is right for everybody.

/// How messages open.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Style {
    /// A page, with the sender's headings, links and tables intact.
    #[default]
    Formatted,
    /// A text control, with a caret and no structure.
    Plain,
}

impl Style {
    /// How the setting stores itself, and reads back.
    pub const fn as_str(self) -> &'static str {
        match self {
            Style::Formatted => "formatted",
            Style::Plain => "plain",
        }
    }

    /// Read a stored setting.
    ///
    /// Anything unrecognised is formatted, which is the default and the one
    /// that keeps what the sender wrote. A settings file from a later version
    /// should not quietly take structure away.
    pub fn from_stored(stored: &str) -> Self {
        match stored.trim().to_ascii_lowercase().as_str() {
            "plain" | "text" => Style::Plain,
            _ => Style::Formatted,
        }
    }

    /// What the choice says in the settings screen.
    ///
    /// Each says what it costs. "Recommended" on its own is something to click
    /// past, and the trade here is real in both directions.
    pub const fn spoken(self) -> &'static str {
        match self {
            Style::Formatted => {
                "Formatted, keeping the sender's headings, links and tables. \
                 Press H to move between headings"
            }
            Style::Plain => {
                "Plain text, with a caret you can move through the message \
                 character by character. Headings, links and tables are flattened"
            }
        }
    }

    /// Both, so a chooser and its tests cover the set.
    pub const ALL: [Style; 2] = [Style::Formatted, Style::Plain];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_messages_open_formatted_unless_somebody_says_otherwise() {
        // Flattening a message throws away structure the sender put there, and
        // the person most affected is the one who cannot see the layout that
        // would otherwise stand in for it.
        assert_eq!(Style::default(), Style::Formatted);
        assert_eq!(Style::from_stored(""), Style::Formatted);
        assert_eq!(Style::from_stored("something else"), Style::Formatted);
    }

    #[test]
    fn test_a_stored_choice_reads_back_as_itself() {
        for style in Style::ALL {
            assert_eq!(Style::from_stored(style.as_str()), style);
        }
    }

    #[test]
    fn test_plain_is_recognised_however_it_was_written() {
        assert_eq!(Style::from_stored("Plain"), Style::Plain);
        assert_eq!(Style::from_stored(" text "), Style::Plain);
    }

    #[test]
    fn test_each_choice_says_what_it_costs_rather_than_which_is_better() {
        // The trade is real in both directions: structure against a caret.
        assert!(Style::Formatted.spoken().contains("headings"));
        assert!(Style::Plain.spoken().contains("caret"));
        assert!(Style::Plain.spoken().contains("flattened"));
        for style in Style::ALL {
            assert!(!style.spoken().contains("ecommended"), "{style:?}");
        }
    }
}

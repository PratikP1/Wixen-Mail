//! Markdown, typed into the message and turned into structure as it is typed.
//!
//! # Why this is here at all
//!
//! Headings and lists are on a menu and on keys, and both of those ask somebody
//! to stop writing. Markdown does not. Typing `## ` and carrying on is the only
//! way to put a heading in a message without leaving the keyboard or leaving
//! the sentence, and it is how a great many people who cannot see a toolbar
//! already write.
//!
//! So this is not a convenience layered on top of the real editor. For the
//! people this application is for, it is the primary way structure gets into a
//! message, and the menu is the fallback.
//!
//! # What is deliberately not here
//!
//! No Markdown parser. Nothing here parses a document; it recognises what
//! somebody has just typed, one marker at a time, at the moment they finish
//! typing it. A parser would be the wrong shape: it would have to run on every
//! keystroke over the whole message and decide again what it already decided.
//!
//! # Why the rules are in Rust when the editor is a web page
//!
//! The same reason the keys are. The page is generated from these tables, so
//! what the editor recognises and what the documentation promises come from one
//! definition. The recognising is testable here; only the applying is JavaScript.

use crate::presentation::editor_document::Format;

/// A marker typed at the start of a line, and the block it makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockRule {
    /// What is typed before the space that triggers it.
    pub marker: &'static str,
    /// What the line becomes.
    pub format: Format,
}

/// The markers recognised at the start of a line.
///
/// Longest first, so `###` is not read as `#` followed by two more.
pub static BLOCK_RULES: [BlockRule; 6] = [
    BlockRule {
        marker: "###",
        format: Format::Heading3,
    },
    BlockRule {
        marker: "##",
        format: Format::Heading2,
    },
    BlockRule {
        marker: "#",
        format: Format::Heading1,
    },
    BlockRule {
        marker: "-",
        format: Format::BulletList,
    },
    BlockRule {
        marker: "*",
        format: Format::BulletList,
    },
    BlockRule {
        marker: ">",
        format: Format::Quote,
    },
];

/// The marker that starts a numbered list, once a digit run and a dot are seen.
///
/// Not in [`BLOCK_RULES`] because it is not one string: Markdown accepts any
/// number, and people who resume a list after a paragraph type the number they
/// are up to rather than 1.
const NUMBERED_FORMAT: Format = Format::NumberedList;

/// What a line becomes, given everything typed on it before the caret.
///
/// The space that finishes the marker is not included: this is asked at the
/// moment the space is typed, so it decides whether that space finishes a
/// marker or is just a space.
///
/// Leading whitespace is ignored, because an indented `- ` is still a bullet
/// and because somebody may be continuing a list the editor already indented.
pub fn block_rule_for(typed_on_line: &str) -> Option<Format> {
    let typed = typed_on_line.trim_start();
    if typed.is_empty() {
        return None;
    }
    if let Some(rule) = BLOCK_RULES.iter().find(|rule| rule.marker == typed) {
        return Some(rule.format);
    }
    is_numbered_marker(typed).then_some(NUMBERED_FORMAT)
}

/// Whether the text is a list number: digits and then a dot, and nothing else.
fn is_numbered_marker(typed: &str) -> bool {
    let Some(digits) = typed.strip_suffix('.') else {
        return false;
    };
    !digits.is_empty() && digits.chars().all(|character| character.is_ascii_digit())
}

/// A pair of delimiters typed around some words, and what they make of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InlineStyle {
    /// Typed on both sides of the words.
    pub delimiter: &'static str,
    /// The element the words end up in.
    pub tag: &'static str,
    /// What is announced when it happens.
    pub spoken: &'static str,
}

/// The inline markers, longest first.
///
/// Order is the whole correctness argument: `**` has to be tried before `*`, or
/// every bold ends up as an italic wrapping a stray asterisk.
pub static INLINE_STYLES: [InlineStyle; 5] = [
    InlineStyle {
        delimiter: "**",
        tag: "strong",
        spoken: "Bold",
    },
    InlineStyle {
        delimiter: "__",
        tag: "strong",
        spoken: "Bold",
    },
    InlineStyle {
        delimiter: "*",
        tag: "em",
        spoken: "Italic",
    },
    InlineStyle {
        delimiter: "_",
        tag: "em",
        spoken: "Italic",
    },
    InlineStyle {
        delimiter: "`",
        tag: "code",
        spoken: "Code",
    },
];

/// The style at a given index, for a message that says one was applied.
pub fn inline_style(index: usize) -> Option<&'static InlineStyle> {
    INLINE_STYLES.get(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashes_make_headings() {
        assert_eq!(block_rule_for("#"), Some(Format::Heading1));
        assert_eq!(block_rule_for("##"), Some(Format::Heading2));
        assert_eq!(block_rule_for("###"), Some(Format::Heading3));
    }

    #[test]
    fn test_the_longer_marker_wins() {
        // Read shortest-first, "###" is a heading 1 with two spare hashes, and
        // somebody typing a third-level heading silently gets a first-level
        // one. Which is the sort of thing nobody notices until the message has
        // gone.
        assert_eq!(block_rule_for("###"), Some(Format::Heading3));
        assert_ne!(block_rule_for("###"), Some(Format::Heading1));
    }

    #[test]
    fn test_dashes_and_stars_make_bullets() {
        assert_eq!(block_rule_for("-"), Some(Format::BulletList));
        assert_eq!(block_rule_for("*"), Some(Format::BulletList));
    }

    #[test]
    fn test_any_number_starts_a_numbered_list() {
        // Markdown accepts any number, and somebody resuming a list after a
        // paragraph types the one they are up to.
        assert_eq!(block_rule_for("1."), Some(Format::NumberedList));
        assert_eq!(block_rule_for("7."), Some(Format::NumberedList));
        assert_eq!(block_rule_for("12."), Some(Format::NumberedList));
    }

    #[test]
    fn test_a_dot_on_its_own_is_not_a_list() {
        assert_eq!(block_rule_for("."), None);
        assert_eq!(block_rule_for("a."), None);
        assert_eq!(block_rule_for("1.2."), None);
    }

    #[test]
    fn test_an_angle_bracket_quotes() {
        assert_eq!(block_rule_for(">"), Some(Format::Quote));
    }

    #[test]
    fn test_a_marker_has_to_be_the_whole_line_so_far() {
        // Otherwise a sentence ending in a hyphen turns into a bullet at the
        // next space, in the middle of writing.
        assert_eq!(block_rule_for("well -"), None);
        assert_eq!(block_rule_for("C#"), None);
        assert_eq!(block_rule_for("2 + 2 ="), None);
    }

    #[test]
    fn test_an_indented_marker_still_counts() {
        assert_eq!(block_rule_for("  -"), Some(Format::BulletList));
        assert_eq!(block_rule_for("\t#"), Some(Format::Heading1));
    }

    #[test]
    fn test_nothing_typed_is_not_a_marker() {
        assert_eq!(block_rule_for(""), None);
        assert_eq!(block_rule_for("   "), None);
    }

    #[test]
    fn test_the_double_delimiters_come_before_the_single_ones() {
        // The same argument as the headings, and the one that decides whether
        // "**word**" is bold or an italic wrapped around two asterisks.
        let first_star = INLINE_STYLES
            .iter()
            .position(|style| style.delimiter == "*")
            .expect("* is a delimiter");
        let first_double = INLINE_STYLES
            .iter()
            .position(|style| style.delimiter == "**")
            .expect("** is a delimiter");
        assert!(first_double < first_star);
        let first_underscore = INLINE_STYLES
            .iter()
            .position(|style| style.delimiter == "_")
            .expect("_ is a delimiter");
        let first_double_underscore = INLINE_STYLES
            .iter()
            .position(|style| style.delimiter == "__")
            .expect("__ is a delimiter");
        assert!(first_double_underscore < first_underscore);
    }

    #[test]
    fn test_every_inline_style_says_what_it_did() {
        for style in &INLINE_STYLES {
            assert!(!style.spoken.is_empty(), "{style:?}");
            assert!(!style.tag.is_empty(), "{style:?}");
        }
    }
}

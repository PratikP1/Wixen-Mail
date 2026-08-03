//! What help exists, and which key reaches it.
//!
//! The documents were written, they ship beside the program, and there was no
//! way in: no contents, no `F1`, and the only button that opened one was on the
//! first-run screen, which somebody sees once. Written and unreachable is the
//! same as unwritten for anybody who needs it in the middle of doing something.
//!
//! # Why F1 goes somewhere useful rather than somewhere general
//!
//! `F1` opened the About box. About is a version number and a licence; it is
//! not what somebody presses `F1` for. Now `F1` opens the page for whatever is
//! open, so pressing it in the calendar gives the calendar, and the contents
//! are one step further on for anybody who wants the whole list.
//!
//! Context-sensitive help matters more by ear than by eye. Somebody who can see
//! a page can skim it for the part they need in a second or two; somebody
//! listening reads it in order, so landing on the right page is the difference
//! between an answer and four minutes of reading.
//!
//! # Why this is data
//!
//! The same reason the item forms and the context menus are. The list is what
//! the menu shows and what `F1` looks up, and two hand-written copies of one
//! list drift apart quietly.
//!
//! The Help menu is the contents. A separate contents page was written and
//! taken out again: its links would have pointed at Markdown files, and opening
//! one of those hands somebody a text editor full of hash signs, which is the
//! whole reason `help_page` exists.

use crate::presentation::ui_types::PimModule;

/// One page of help.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Topic {
    /// What it is called, in the menu and on the contents page.
    ///
    /// Carries its own mnemonic, as every other menu label here does.
    pub title: &'static str,
    /// The document that ships beside the program.
    pub file: &'static str,
    /// One line saying what is in it, for the contents page and for the status
    /// bar hint the menu shows.
    pub covers: &'static str,
    /// Which module this answers questions about, when it answers one.
    ///
    /// `None` for the pages that are about the whole application, which are
    /// reachable from the menu and are never what `F1` lands on.
    pub about: Option<PimModule>,
}

/// Every page, in the order the menu offers them.
///
/// Getting started first, because somebody opening help for the first time is
/// usually at the beginning. The keyboard next, because it is the page this
/// application's readers come back to most.
pub const TOPICS: &[Topic] = &[
    Topic {
        title: "&Getting started",
        file: "getting-started.md",
        covers: "Setting up an account and finding your way around",
        about: Some(PimModule::Mail),
    },
    Topic {
        title: "&Keyboard shortcuts",
        file: "KEYBOARD_SHORTCUTS.md",
        covers: "Every key, by what it does",
        about: None,
    },
    Topic {
        title: "&Using Wixen Mail",
        file: "USER_GUIDE.md",
        covers: "Mail, the calendar, contacts, tasks and notes in detail",
        about: Some(PimModule::Calendar),
    },
    Topic {
        title: "Setting up a &provider",
        file: "PROVIDER_SETUP.md",
        covers: "Gmail, Outlook and other providers, including app passwords",
        about: None,
    },
    Topic {
        title: "When something goes &wrong",
        file: "TROUBLESHOOTING.md",
        covers: "What to try, and how to send a useful report",
        about: None,
    },
    Topic {
        title: "&Accessibility",
        file: "accessibility.md",
        covers: "What is built for screen readers, and what is not done yet",
        about: None,
    },
    Topic {
        title: "Pri&vacy",
        file: "privacy.md",
        covers: "What is stored, where, and what leaves this computer",
        about: None,
    },
    Topic {
        title: "What &changed",
        file: "changelog.md",
        covers: "Every change, newest first, including known limitations",
        about: None,
    },
];

/// The page `F1` opens for whichever module is showing.
///
/// Falls back to getting started, because a page that is roughly right beats
/// a keystroke that does nothing. Somebody pressing `F1` wants an answer, and
/// the contents are one step from any page.
pub fn for_module(module: PimModule) -> &'static Topic {
    TOPICS
        .iter()
        .find(|topic| topic.about == Some(module))
        .unwrap_or(&TOPICS[0])
}

/// A label without its mnemonic marker.
///
/// The ampersand is for a menu. On a page it is a stray character, and read
/// aloud it is the word "and" in the middle of a title.
pub fn plain(label: &str) -> String {
    label.replace('&', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_topic_names_a_document_that_ships() {
        // A menu entry pointing at a missing page is a menu entry that opens
        // an error, which is worse than one that is not there.
        for topic in TOPICS {
            let path = std::path::Path::new("docs").join(topic.file);
            assert!(
                path.exists(),
                "{} points at {}, which is not in docs/",
                topic.title,
                topic.file
            );
        }
    }

    #[test]
    fn test_every_topic_has_a_mnemonic() {
        // Every other menu in this application has them, and a menu where
        // some entries answer a letter and others do not is worse than one
        // where none do.
        for topic in TOPICS {
            assert!(topic.title.contains('&'), "{} has no mnemonic", topic.title);
        }
    }

    #[test]
    fn test_no_two_topics_share_a_mnemonic() {
        // Two entries answering the same letter means one of them cannot be
        // reached by it, and which one is not something anybody can predict.
        let mut letters: Vec<char> = TOPICS
            .iter()
            .filter_map(|topic| {
                topic
                    .title
                    .split('&')
                    .nth(1)
                    .and_then(|rest| rest.chars().next())
                    .map(|c| c.to_ascii_lowercase())
            })
            .collect();
        let before = letters.len();
        letters.sort_unstable();
        letters.dedup();
        assert_eq!(before, letters.len(), "two topics share a mnemonic");
    }

    #[test]
    fn test_f1_always_lands_on_a_page() {
        // A key that does nothing is indistinguishable from a broken one, and
        // this one is pressed by somebody who is already stuck.
        for module in PimModule::ALL {
            let topic = for_module(module);
            assert!(!topic.file.is_empty(), "{module:?} has no help page");
        }
    }

    #[test]
    fn test_f1_opens_the_page_for_the_module_you_are_in() {
        // Landing on a page is not the same as landing on the right one. For
        // somebody listening, the wrong page is read in order from the top,
        // which is the difference between an answer and four minutes of
        // reading.
        assert_eq!(for_module(PimModule::Mail).file, "getting-started.md");
        assert_eq!(for_module(PimModule::Calendar).file, "USER_GUIDE.md");
    }

    #[test]
    fn test_a_title_on_a_page_carries_no_menu_marker() {
        // Read aloud, an ampersand in the middle of a title is the word "and".
        assert_eq!(plain("&Getting started"), "Getting started");
        assert_eq!(plain("Pri&vacy"), "Privacy");
    }
}

//! Working the compose window's toolbar from the keyboard.
//!
//! The toolbar has nine buttons in four groups. Reaching them by Tab would put
//! nine stops between the subject line and the message, on the way somebody
//! takes every time they write anything, so the buttons are out of the tab
//! order and the toolbar is entered on purpose with Ctrl+backslash.
//!
//! Inside it, the arrows move along a group and Ctrl with an arrow moves to the
//! next group. That is the toolbar pattern every desktop and every screen
//! reader already knows, and it means the nine buttons cost one keystroke to
//! reach and two arrows to cross rather than nine presses of Tab.
//!
//! # Entering the toolbar is not built, and the reason is a trade
//!
//! Seven of the nine buttons are given `set_can_focus(false)` so that writing
//! a message does not cost nine tab stops between the subject line and the
//! body. A control that cannot take focus cannot be given it, so nothing
//! could ever move the keyboard onto them: Ctrl+backslash moved focus nowhere
//! and said nothing, which reads exactly like a shortcut nobody bound.
//!
//! The two ways out are a trade nobody has chosen. Make all nine focusable
//! and accept the tab stops, or drop this and keep the shortcut each button
//! already has. Both need a keyboard to settle and one is a judgement about
//! how people write mail, so the shortcut says it is not available rather
//! than pretending, and the movement rules that were written for it are gone
//! rather than sitting here reached by nothing.
//!
//! What is left is what still runs: which button a position is, and what is
//! said when one of the two still-reachable buttons takes focus.

/// One button on the toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    /// What a screen reader says. Not the visible label: "B" is a letter.
    pub spoken: &'static str,
    /// The key that does the same thing without the toolbar, so somebody who
    /// wants it twice learns the shortcut the second time.
    pub shortcut: &'static str,
}

/// A run of related buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Group {
    pub name: &'static str,
    pub keys: &'static [Key],
}

/// The whole toolbar, in the order it is laid out.
///
/// Grouped by what the buttons do rather than by where they fit, because the
/// group name is read aloud when it is crossed and "Formatting" tells somebody
/// where they are in a way that "second group" does not.
pub const GROUPS: &[Group] = &[
    Group {
        name: "Message",
        keys: &[Key {
            spoken: "Send",
            shortcut: "Ctrl+Enter",
        }],
    },
    Group {
        name: "Edit",
        keys: &[
            Key {
                spoken: "Undo",
                shortcut: "Ctrl+Z",
            },
            Key {
                spoken: "Redo",
                shortcut: "Ctrl+Y",
            },
        ],
    },
    Group {
        name: "Formatting",
        keys: &[
            Key {
                spoken: "Bold",
                shortcut: "Ctrl+B",
            },
            Key {
                spoken: "Italic",
                shortcut: "Ctrl+I",
            },
            Key {
                spoken: "Underline",
                shortcut: "Ctrl+U",
            },
            Key {
                spoken: "More formatting, opens a menu",
                shortcut: "Alt+O",
            },
        ],
    },
    Group {
        name: "Tools",
        keys: &[
            Key {
                spoken: "Check spelling",
                shortcut: "F7",
            },
            Key {
                spoken: "Attach a file",
                shortcut: "Alt+A",
            },
        ],
    },
];

/// Where the toolbar cursor is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct At {
    pub group: usize,
    pub key: usize,
}

impl At {
    /// The button this points at, if the toolbar still has one there.
    pub fn key(self) -> Option<&'static Key> {
        GROUPS.get(self.group)?.keys.get(self.key)
    }

    /// Its position among all nine, which is what a button is wired by.
    pub fn position(self) -> Option<usize> {
        self.key()?;
        Some(
            GROUPS
                .iter()
                .take(self.group)
                .map(|group| group.keys.len())
                .sum::<usize>()
                + self.key,
        )
    }

    /// The cursor for a button, given where it sits among all of them.
    ///
    /// The other direction from [`At::position`]. Focus arrives at a button
    /// rather than at a group and a place in it, because the arrows belong to
    /// the window, so what is known is the position and what has to be worked
    /// out is which group it is in.
    pub fn at_position(position: usize) -> Option<Self> {
        let mut left = position;
        for (group, definition) in GROUPS.iter().enumerate() {
            if left < definition.keys.len() {
                return Some(Self { group, key: left });
            }
            left -= definition.keys.len();
        }
        None
    }

    /// What is said when the cursor arrives here.
    ///
    /// The group name comes first only when the group has changed, because
    /// hearing "Formatting" before every one of its four buttons is three
    /// repetitions of something already known. Everything else is always said:
    /// the button, its key, and where it sits, because "3 of 4" is how somebody
    /// knows there is a fourth.
    pub fn spoken(self, from: Option<At>) -> String {
        let Some(key) = self.key() else {
            return String::new();
        };
        let group = GROUPS[self.group];
        let position = format!("{} of {}", self.key + 1, group.keys.len());
        let same_group = from.is_some_and(|previous| previous.group == self.group);
        if same_group {
            format!("{}, {}, {position}", key.spoken, key.shortcut)
        } else {
            format!(
                "{}, {}, {}, {position}",
                group.name, key.spoken, key.shortcut
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arriving_says_the_group_the_button_and_the_key() {
        // Somebody who has just arrived knows none of the three.
        let arrived = At { group: 2, key: 0 };

        assert_eq!(arrived.spoken(None), "Formatting, Bold, Ctrl+B, 1 of 4");
    }

    #[test]
    fn test_every_button_has_a_position_and_they_are_all_different() {
        // The position is what a button is wired by, so two buttons sharing one
        // would make a key press operate the wrong command.
        let mut seen = Vec::new();
        for (group, definition) in GROUPS.iter().enumerate() {
            for key in 0..definition.keys.len() {
                let position = At { group, key }
                    .position()
                    .expect("a button that exists has a position");
                assert!(!seen.contains(&position), "{position} is used twice");
                seen.push(position);
            }
        }
        assert_eq!(
            seen.len(),
            GROUPS.iter().map(|g| g.keys.len()).sum::<usize>()
        );
    }

    #[test]
    fn test_a_position_finds_its_way_back_to_a_group() {
        // Focus arrives at a button rather than at a group and a place in it,
        // because the arrows belong to the window. These two have to agree or
        // the announcement names the wrong button.
        for (group, definition) in GROUPS.iter().enumerate() {
            for key in 0..definition.keys.len() {
                let at = At { group, key };
                let position = at.position().expect("a button that exists");
                assert_eq!(At::at_position(position), Some(at));
            }
        }
    }

    #[test]
    fn test_a_position_past_the_end_is_no_button() {
        let all: usize = GROUPS.iter().map(|group| group.keys.len()).sum();

        assert_eq!(At::at_position(all), None);
        assert_eq!(At::at_position(all + 5), None);
    }

    #[test]
    fn test_every_button_says_a_word_rather_than_a_letter() {
        // The visible labels are B, I and U, which a screen reader reads as
        // letters. Nothing here may be one character long.
        for group in GROUPS {
            assert!(!group.name.trim().is_empty());
            for key in group.keys {
                assert!(
                    key.spoken.trim().chars().count() > 1,
                    "{:?} is not a word",
                    key.spoken
                );
                assert!(!key.shortcut.trim().is_empty(), "{:?}", key.spoken);
            }
        }
    }
}

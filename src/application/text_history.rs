//! Several steps of undo for a text box, because Windows' own box keeps one.
//!
//! # Why the history is ours
//!
//! A plain Windows edit box remembers one change, and Undo pressed twice
//! undoes the undo (`src/msw/textentry.cpp`). The tester asked for "a
//! multi-step history where the native control gives one step" (#47).
//! Switching every box to a rich edit control, which keeps several steps of
//! its own, was weighed and rejected: it changes the window class of about
//! 35 controls and with it what MSAA and UI Automation say about each. So the
//! history is kept here, one per box, and the box is told what to show.
//!
//! # What a step is
//!
//! Something a person would call one thing: a run of typing up to the end of
//! a word, a paste, a cut, a run of deleting in one direction. A space or a
//! punctuation mark ends the step it is typed in. A change of more than one
//! character at once is a step of its own. Moving the caret somewhere else
//! between two changes starts a new step, and so does an undo or a redo.
//!
//! # Positions
//!
//! The box counts positions the way Windows does, a character outside the
//! basic plane as two, and a line break as one because wxWidgets hands back
//! `\n` alone (`tests/text_selection_offsets.rs` measured both). The caret
//! this is given and the selection it hands back are counted that way; inside,
//! everything is counted in characters, so no string is ever cut through the
//! middle of one.

/// At most this many steps are kept for one box; the oldest goes first.
pub const MOST_STEPS: usize = 100;

/// What the box should show after an undo or a redo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Restore {
    pub value: String,
    /// From and to, counted the way the box counts. Equal when it is a caret.
    pub selection: (usize, usize),
}

/// One box's steps, back and forward.
#[derive(Debug, Clone, Default)]
pub struct History {}

impl History {
    /// A history for a box holding `value`, with nothing to undo.
    pub fn new(_value: &str) -> Self {
        Self {}
    }

    /// The box now holds `after`, with the caret at `caret_after`.
    pub fn record(&mut self, _after: &str, _caret_after: usize) {}

    /// Take the last step back.
    pub fn undo(&mut self) -> Option<Restore> {
        None
    }

    /// Put the last step undone back.
    pub fn redo(&mut self) -> Option<Restore> {
        None
    }

    pub fn can_undo(&self) -> bool {
        false
    }

    pub fn can_redo(&self) -> bool {
        false
    }

    /// The program wrote `value` into the box: a new start, nothing to undo.
    pub fn set_anew(&mut self, _value: &str) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How the box counts `text`'s length.
    fn units(text: &str) -> usize {
        text.encode_utf16().count()
    }

    /// Each character of `words` typed at the end of `before`, one key at a
    /// time, as the box reports each change. Returns what the box holds.
    fn typed(history: &mut History, before: &str, words: &str) -> String {
        let mut value = before.to_string();
        for character in words.chars() {
            value.push(character);
            history.record(&value, units(&value));
        }
        value
    }

    fn undone_to(history: &mut History) -> Option<String> {
        history.undo().map(|restore| restore.value)
    }

    fn redone_to(history: &mut History) -> Option<String> {
        history.redo().map(|restore| restore.value)
    }

    #[test]
    fn test_three_words_typed_come_back_one_word_at_a_time() {
        let mut history = History::new("");
        typed(&mut history, "", "one two three");
        assert_eq!(undone_to(&mut history).as_deref(), Some("one two "));
        assert_eq!(undone_to(&mut history).as_deref(), Some("one "));
        assert_eq!(undone_to(&mut history).as_deref(), Some(""));
        assert_eq!(undone_to(&mut history), None);
    }

    #[test]
    fn test_redo_puts_back_each_word_in_turn() {
        let mut history = History::new("");
        typed(&mut history, "", "one two three");
        while history.undo().is_some() {}
        assert!(history.can_redo());
        assert_eq!(redone_to(&mut history).as_deref(), Some("one "));
        assert_eq!(redone_to(&mut history).as_deref(), Some("one two "));
        assert_eq!(redone_to(&mut history).as_deref(), Some("one two three"));
        assert_eq!(redone_to(&mut history), None);
        assert!(!history.can_redo());
    }

    #[test]
    fn test_typing_after_an_undo_leaves_nothing_to_redo() {
        let mut history = History::new("");
        typed(&mut history, "", "one two");
        assert_eq!(undone_to(&mut history).as_deref(), Some("one "));
        assert!(history.can_redo());
        typed(&mut history, "one ", "x");
        assert!(!history.can_redo());
        assert_eq!(redone_to(&mut history), None);
        assert_eq!(undone_to(&mut history).as_deref(), Some("one "));
    }

    #[test]
    fn test_a_paste_is_one_step() {
        let mut history = History::new("");
        typed(&mut history, "", "one ");
        let pasted = "one pasted words";
        history.record(pasted, units(pasted));
        assert_eq!(undone_to(&mut history).as_deref(), Some("one "));
        assert_eq!(undone_to(&mut history).as_deref(), Some(""));
        assert_eq!(undone_to(&mut history), None);
    }

    #[test]
    fn test_deleting_backwards_is_one_step_until_the_direction_changes() {
        // "alpha beta ga|mma": Backspace twice takes "ga", then Delete twice
        // takes "mm" from after the caret.
        let mut history = History::new("alpha beta gamma");
        history.record("alpha beta gmma", 12);
        history.record("alpha beta mma", 11);
        history.record("alpha beta ma", 11);
        history.record("alpha beta a", 11);
        let forward = history.undo();
        assert_eq!(
            forward,
            Some(Restore {
                value: "alpha beta mma".to_string(),
                selection: (11, 13),
            })
        );
        let backward = history.undo();
        assert_eq!(
            backward,
            Some(Restore {
                value: "alpha beta gamma".to_string(),
                selection: (11, 13),
            })
        );
        assert!(!history.can_undo());
    }

    #[test]
    fn test_moving_the_caret_starts_a_new_step() {
        let mut history = History::new("");
        typed(&mut history, "", "one");
        // The caret moved to the start, and a letter typed there.
        history.record("Xone", 1);
        assert_eq!(undone_to(&mut history).as_deref(), Some("one"));
        assert_eq!(undone_to(&mut history).as_deref(), Some(""));
    }

    #[test]
    fn test_the_selection_comes_back_with_the_step() {
        // "beta" chosen in "alpha beta", and "x" typed over it.
        let mut history = History::new("alpha beta");
        history.record("alpha x", 7);
        assert_eq!(
            history.undo(),
            Some(Restore {
                value: "alpha beta".to_string(),
                selection: (6, 10),
            })
        );
        assert_eq!(
            history.redo(),
            Some(Restore {
                value: "alpha x".to_string(),
                selection: (7, 7),
            })
        );
    }

    #[test]
    fn test_a_value_set_anew_leaves_nothing_to_undo() {
        let mut history = History::new("");
        typed(&mut history, "", "one two");
        assert!(history.undo().is_some());
        history.set_anew("a note");
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo(), None);
        // What is typed next is undone back to the value set, not past it.
        typed(&mut history, "a note", "s");
        assert!(history.can_undo());
        assert_eq!(undone_to(&mut history).as_deref(), Some("a note"));
    }

    #[test]
    fn test_the_hundred_and_first_step_drops_the_first() {
        let mut history = History::new("");
        let mut value = String::new();
        for _ in 0..=MOST_STEPS {
            value.push_str("ab");
            history.record(&value, units(&value));
        }
        let mut undone = 0;
        let mut left = None;
        while let Some(restore) = history.undo() {
            undone += 1;
            left = Some(restore.value);
        }
        assert_eq!(undone, MOST_STEPS);
        // The first "ab" was the step that went, so it stays.
        assert_eq!(left.as_deref(), Some("ab"));
    }

    #[test]
    fn test_a_restore_after_an_emoji_lands_on_the_right_characters() {
        // The emoji is two of the box's positions, so "beta" is 5 to 9 there
        // and 4 to 8 counted in characters. "beta" cut, then undone.
        let mut history = History::new("ab\u{1F600} beta");
        history.record("ab\u{1F600} ", 5);
        assert_eq!(
            history.undo(),
            Some(Restore {
                value: "ab\u{1F600} beta".to_string(),
                selection: (5, 9),
            })
        );
    }
}

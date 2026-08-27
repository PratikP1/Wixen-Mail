//! Cut, copy, paste and select all, and what each means where the cursor is.
//!
//! # Why this is a decision rather than four calls
//!
//! The main window has a text box, six lists and two trees in it, and the same
//! four keys mean different things in each. Copy in a note's body is the words
//! somebody selected; copy in the message list is the row they are on; copy in
//! the folder tree is the folder's name. Cut and paste mean nothing in any of
//! the lists at all.
//!
//! Spreading that across the places that handle each command would put the same
//! question in nine places. Here it is asked once and the answer is testable
//! without a window.
//!
//! # Why nothing ever happens silently
//!
//! A key that does nothing cannot be told apart from a key that does not work.
//! Somebody who presses Ctrl+V in a message list and gets silence has learned
//! nothing about why, and if they cannot see the screen they may not even know
//! the focus is where it is. So every command that cannot act says what it
//! would have needed.
//!
//! # Why selecting everything has a limit
//!
//! The message list is virtual and holds as many rows as the mailbox does; the
//! sample mailbox alone is two hundred thousand. Selecting them means that many
//! calls into the control on the interface thread, which stops the window
//! answering, and a window that stops answering takes the screen reader with
//! it. That has happened here before and is written up in `CLAUDE.md`. So above
//! a limit the answer is a sentence rather than a freeze.

/// One of the four.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditCommand {
    Cut,
    Copy,
    Paste,
    SelectAll,
}

impl EditCommand {
    /// The word for it, for a sentence that has to name it.
    pub fn name(self) -> &'static str {
        match self {
            EditCommand::Cut => "Cut",
            EditCommand::Copy => "Copy",
            EditCommand::Paste => "Paste",
            EditCommand::SelectAll => "Select All",
        }
    }
}

/// What the cursor is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Where {
    /// A text box that can be typed into.
    AText,
    /// A text box that can be read and not changed.
    AReadOnlyText,
    /// One of the item lists, holding this many rows.
    AList { rows: usize },
    /// One of the sidebar trees.
    ATree,
    /// Focus is somewhere none of this applies, or nowhere known.
    SomewhereElse,
}

/// What to do about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Doing {
    /// Let the text box do it to its own selection.
    ToTheText,
    /// Put the chosen row or node on the clipboard as text.
    CopyWhatIsChosen,
    /// Choose every row.
    ChooseEveryRow,
    /// Nothing, and this is what to say about it.
    NotHere(String),
}

/// How many rows may be selected at once before the answer is a sentence.
///
/// Chosen to be far above any list somebody reads through and far below the
/// number that stops the window answering. The sample mailbox is two hundred
/// thousand rows, and a real one can be larger.
pub const MOST_ROWS_WORTH_SELECTING: usize = 5_000;

/// What a command should do, given where the cursor is.
pub fn what_to_do(command: EditCommand, place: Where) -> Doing {
    match (command, place) {
        // A text box does its own work for all four, which is what every other
        // Windows program does and what the native control already knows how
        // to do.
        (_, Where::AText) => Doing::ToTheText,
        (EditCommand::Copy | EditCommand::SelectAll, Where::AReadOnlyText) => Doing::ToTheText,
        (EditCommand::Cut | EditCommand::Paste, Where::AReadOnlyText) => Doing::NotHere(format!(
            "{} needs a box you can type in, and this one can only be read.",
            command.name()
        )),

        (EditCommand::Copy, Where::AList { .. } | Where::ATree) => Doing::CopyWhatIsChosen,
        (EditCommand::SelectAll, Where::AList { rows }) if rows <= MOST_ROWS_WORTH_SELECTING => {
            Doing::ChooseEveryRow
        }
        (EditCommand::SelectAll, Where::AList { rows }) => Doing::NotHere(format!(
            "There are {rows} rows here, which is more than Select All will take at once. \
             Selecting them would stop the window answering."
        )),
        (EditCommand::SelectAll, Where::ATree) => Doing::NotHere(
            "Select All works in a list or a box you can type in. This is the sidebar.".to_string(),
        ),
        (EditCommand::Cut | EditCommand::Paste, Where::AList { .. } | Where::ATree) => {
            Doing::NotHere(format!(
                "{} needs a box you can type in. Copy works here.",
                command.name()
            ))
        }

        (_, Where::SomewhereElse) => Doing::NotHere(format!(
            "{} needs a box you can type in, a list, or the sidebar. \
             Tab or F6 moves between them.",
            command.name()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EVERY_COMMAND: [EditCommand; 4] = [
        EditCommand::Cut,
        EditCommand::Copy,
        EditCommand::Paste,
        EditCommand::SelectAll,
    ];

    #[test]
    fn test_a_text_box_does_all_four_itself() {
        // What every other Windows program does, and what the native control
        // already knows how to do. Anything cleverer here would be this
        // application reimplementing an edit box.
        for command in EVERY_COMMAND {
            assert_eq!(
                what_to_do(command, Where::AText),
                Doing::ToTheText,
                "{command:?} does not reach the text box"
            );
        }
    }

    #[test]
    fn test_a_box_that_can_only_be_read_still_copies_and_selects() {
        // Reading a message and copying a line out of it is the ordinary
        // reason somebody is in a read-only box at all.
        assert_eq!(
            what_to_do(EditCommand::Copy, Where::AReadOnlyText),
            Doing::ToTheText
        );
        assert_eq!(
            what_to_do(EditCommand::SelectAll, Where::AReadOnlyText),
            Doing::ToTheText
        );
    }

    #[test]
    fn test_changing_a_box_that_can_only_be_read_says_why_not() {
        // Rather than doing nothing, which cannot be told apart from a key
        // that is broken.
        let Doing::NotHere(said) = what_to_do(EditCommand::Paste, Where::AReadOnlyText) else {
            panic!("paste into a read-only box was allowed");
        };

        assert!(said.contains("Paste"), "{said}");
        assert!(said.contains("only be read"), "{said}");
    }

    #[test]
    fn test_copy_in_a_list_or_the_sidebar_takes_what_is_chosen() {
        assert_eq!(
            what_to_do(EditCommand::Copy, Where::AList { rows: 40 }),
            Doing::CopyWhatIsChosen
        );
        assert_eq!(
            what_to_do(EditCommand::Copy, Where::ATree),
            Doing::CopyWhatIsChosen
        );
    }

    #[test]
    fn test_selecting_every_row_of_an_ordinary_list_is_allowed() {
        assert_eq!(
            what_to_do(EditCommand::SelectAll, Where::AList { rows: 40 }),
            Doing::ChooseEveryRow
        );
        // Right up to the limit, so the boundary is where it says it is.
        assert_eq!(
            what_to_do(
                EditCommand::SelectAll,
                Where::AList {
                    rows: MOST_ROWS_WORTH_SELECTING
                }
            ),
            Doing::ChooseEveryRow
        );
    }

    #[test]
    fn test_selecting_every_row_of_an_enormous_list_says_so_rather_than_freezing() {
        // The sample mailbox is two hundred thousand rows. Selecting them is
        // that many calls into the control on the interface thread, and a
        // window that stops answering takes the screen reader with it.
        let Doing::NotHere(said) = what_to_do(
            EditCommand::SelectAll,
            Where::AList {
                rows: MOST_ROWS_WORTH_SELECTING + 1,
            },
        ) else {
            panic!("select all was allowed on a list large enough to hang the window");
        };

        assert!(said.contains("stop the window answering"), "{said}");
        assert!(
            said.contains(&(MOST_ROWS_WORTH_SELECTING + 1).to_string()),
            "the sentence does not say how many there are: {said}"
        );
    }

    #[test]
    fn test_cut_and_paste_in_a_list_say_what_does_work_there() {
        // Naming what does work is the difference between a refusal somebody
        // learns from and one they only find annoying.
        for place in [Where::AList { rows: 10 }, Where::ATree] {
            let Doing::NotHere(said) = what_to_do(EditCommand::Cut, place) else {
                panic!("cut was allowed in {place:?}");
            };
            assert!(said.contains("Copy works here"), "{said}");
        }
    }

    #[test]
    fn test_focus_somewhere_unknown_says_where_the_commands_do_work() {
        // Somebody who cannot see the screen may not know where focus is, so
        // the sentence says how to get somewhere these work.
        let Doing::NotHere(said) = what_to_do(EditCommand::Copy, Where::SomewhereElse) else {
            panic!("copy claimed to work with focus nowhere known");
        };

        assert!(said.contains("F6"), "the way out is not named: {said}");
    }

    #[test]
    fn test_nothing_ever_happens_without_something_being_said() {
        // Over every combination, because the failure that matters is a key
        // that does nothing quietly, and it only has to slip through once.
        for command in EVERY_COMMAND {
            for place in [
                Where::AText,
                Where::AReadOnlyText,
                Where::AList { rows: 10 },
                Where::AList {
                    rows: MOST_ROWS_WORTH_SELECTING + 1,
                },
                Where::ATree,
                Where::SomewhereElse,
            ] {
                if let Doing::NotHere(said) = what_to_do(command, place) {
                    assert!(
                        said.len() > 20 && said.ends_with('.'),
                        "{command:?} in {place:?} refuses with {said:?}, which is not a sentence"
                    );
                }
            }
        }
    }
}

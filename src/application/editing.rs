//! Cut, copy, paste, select all, undo and redo, and what each means where the
//! cursor is.
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

/// One of the six on the Edit menu that act on what has focus.
///
/// Undo and Redo act on a text box alone. The box's history of several steps
/// is what they take back and put back, which `presentation::text_undo`
/// reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditCommand {
    Cut,
    Copy,
    Paste,
    SelectAll,
    Undo,
    Redo,
}

impl EditCommand {
    /// The word for it, for a sentence that has to name it.
    pub fn name(self) -> &'static str {
        match self {
            EditCommand::Cut => "Cut",
            EditCommand::Copy => "Copy",
            EditCommand::Paste => "Paste",
            EditCommand::SelectAll => "Select All",
            EditCommand::Undo => "Undo",
            EditCommand::Redo => "Redo",
        }
    }
}

/// What Undo says in a box that has nothing to take back.
///
/// Said rather than doing nothing, for the reason at the head of this module.
pub const NOTHING_TO_UNDO: &str = "There is nothing to undo in this box.";

/// What Redo says when there is no Undo to put back.
///
/// It says when Redo does work, because a box remembers one step: Redo is
/// only offered right after an Undo, and somebody who pressed it after typing
/// needs to know why it refused.
pub const NOTHING_TO_REDO: &str =
    "There is nothing to redo in this box. Redo puts back what Undo just took away.";

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
    /// Undo or Redo the last action on the list's items. Which lists have one
    /// is the window's to say, since only it knows what each list last did;
    /// a list with none says so in a sentence (`application::undoing`).
    TheLastAction,
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
        // A text box does its own work for all six, which is what every other
        // Windows program does and what the native control already knows how
        // to do.
        (_, Where::AText) => Doing::ToTheText,
        (EditCommand::Copy | EditCommand::SelectAll, Where::AReadOnlyText) => Doing::ToTheText,
        (
            EditCommand::Cut | EditCommand::Paste | EditCommand::Undo | EditCommand::Redo,
            Where::AReadOnlyText,
        ) => Doing::NotHere(format!(
            "{} needs a box you can type in, and this one can only be read.",
            command.name()
        )),

        // Undo and Redo take back and put back a change somebody typed, and a
        // list or the sidebar has none to offer. Nor does the way out Cut and
        // Copy give below, which would send somebody to a list for Undo.
        (EditCommand::Undo | EditCommand::Redo, Where::AList { .. } | Where::ATree) => {
            Doing::NotHere(format!(
                "{} works in a box you can type in, not in a list or the sidebar.",
                command.name()
            ))
        }
        (EditCommand::Undo | EditCommand::Redo, Where::SomewhereElse) => Doing::NotHere(format!(
            "{} works in a box you can type in. Tab or F6 moves between the parts of the window.",
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

    const EVERY_COMMAND: [EditCommand; 6] = [
        EditCommand::Cut,
        EditCommand::Copy,
        EditCommand::Paste,
        EditCommand::SelectAll,
        EditCommand::Undo,
        EditCommand::Redo,
    ];

    /// The two that take a change back or put it back.
    const TAKING_BACK: [EditCommand; 2] = [EditCommand::Undo, EditCommand::Redo];

    #[test]
    fn test_a_text_box_does_every_command_itself() {
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
    fn test_undo_and_redo_in_a_box_that_can_only_be_read_say_it_can_only_be_read() {
        // The same refusal Cut and Paste give there, naming the command the
        // way the menu does.
        assert_eq!(
            what_to_do(EditCommand::Undo, Where::AReadOnlyText),
            Doing::NotHere(
                "Undo needs a box you can type in, and this one can only be read.".to_string()
            )
        );
        assert_eq!(
            what_to_do(EditCommand::Redo, Where::AReadOnlyText),
            Doing::NotHere(
                "Redo needs a box you can type in, and this one can only be read.".to_string()
            )
        );
    }

    #[test]
    fn test_undo_and_redo_in_the_sidebar_say_they_work_in_a_box() {
        // Not Cut's "Copy works here", which answers a different question,
        // and no promise of something that does not exist yet. A list has
        // its own meaning for them since 13-07; the sidebar changes nothing
        // anybody could take back.
        for command in TAKING_BACK {
            let Doing::NotHere(said) = what_to_do(command, Where::ATree) else {
                panic!("{command:?} claimed to work in the sidebar");
            };
            assert!(said.starts_with(command.name()), "{said}");
            assert!(said.contains("a box you can type in"), "{said}");
            assert!(!said.contains("Copy works here"), "{said}");
            assert!(!said.contains("later"), "{said}");
        }
    }

    #[test]
    fn test_undo_in_a_list_is_the_last_action() {
        // What was last done to the list's items, a mark or a star or a
        // label, which the window holds and the list names (#47's second
        // level). However many rows the list has.
        for command in TAKING_BACK {
            for rows in [0, 10, MOST_ROWS_WORTH_SELECTING + 1] {
                assert_eq!(
                    what_to_do(command, Where::AList { rows }),
                    Doing::TheLastAction,
                    "{command:?} in a list of {rows}"
                );
            }
        }
    }

    #[test]
    fn test_undo_and_redo_with_focus_nowhere_known_say_how_to_reach_a_box() {
        // The way out Cut and Copy give names a list and the sidebar as well,
        // and sending somebody to a list for Undo sends them to a refusal.
        for command in TAKING_BACK {
            let Doing::NotHere(said) = what_to_do(command, Where::SomewhereElse) else {
                panic!("{command:?} claimed to work with focus nowhere known");
            };
            assert!(said.starts_with(command.name()), "{said}");
            assert!(said.contains("a box you can type in"), "{said}");
            assert!(said.contains("F6"), "the way out is not named: {said}");
            assert!(!said.contains("a list"), "{said}");
        }
    }

    #[test]
    fn test_having_nothing_to_undo_or_redo_is_said_as_a_persons_sentence() {
        // The two sentences the window speaks when the box has nothing to
        // take back or put back. Here beside the rule so the window cannot
        // word them differently.
        use crate::application::status_sentences::{Voice, reads_as_a_persons_sentence};
        for sentence in [NOTHING_TO_UNDO, NOTHING_TO_REDO] {
            reads_as_a_persons_sentence(sentence, Voice::Answer)
                .unwrap_or_else(|why| panic!("{sentence:?}: {why}"));
        }
        assert!(
            NOTHING_TO_UNDO.contains("nothing to undo"),
            "{NOTHING_TO_UNDO}"
        );
        assert!(
            NOTHING_TO_REDO.contains("nothing to redo"),
            "{NOTHING_TO_REDO}"
        );
        assert!(
            NOTHING_TO_REDO.contains("Redo puts back what Undo just took away"),
            "the sentence does not say when Redo works: {NOTHING_TO_REDO}"
        );
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

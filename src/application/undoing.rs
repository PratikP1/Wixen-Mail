//! Undo and Redo of the last thing somebody did to messages: a mark as read
//! or unread, a star, or a label, with the message named (#47, GAP-02).
//!
//! The tester on 2026-09-15 asked for "the last delete, move, copy, mark as
//! read or unread, star ... as 'Undo delete' or 'Undo move' with the item
//! named, for a bounded time or until the next action; and Redo of that."
//! This is the marks. Moves, deletes and copies are 13-08's.
//!
//! # Each message's own state, never the opposite
//!
//! Mark as Read over a set where some were read and some were not sets them
//! all read. Undoing it by marking them all unread would unread the ones that
//! were read before anybody touched them. So the action keeps each message as
//! it was, `Before`, and the undo puts each back to that, leaving out the ones
//! the action did not change.
//!
//! # One step, and no timer
//!
//! Only the last action is kept, and it lasts until the next action on
//! messages replaces it. Nothing here holds a time, because a limit on how
//! long somebody has to find the menu is a timing trap (WCAG 2.2.1). After an
//! Undo only Redo is offered and after a Redo only Undo, so the one step goes
//! back and forth. Decision 9 of phase 13.
//!
//! This knows nothing of the window. It says what an undo changes and what it
//! is called; `presentation::wx_app` carries it out through the same path the
//! action took, so a server that refuses puts it back and says so.

use crate::service::caldav::how_many;

/// A label as an undo needs it: which one it is here, what it is called, and
/// what it travels as to the server, when it travels at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub id: String,
    pub name: String,
    pub keyword: Option<String>,
}

/// One message as it was before the action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Before {
    pub row_id: i64,
    pub uid: u32,
    pub subject: String,
    pub read: bool,
    pub starred: bool,
    /// The labels it carried. Kept for a label action only; empty otherwise.
    pub labels: Vec<Label>,
}

impl Before {
    fn has(&self, label: &Label) -> bool {
        self.labels.iter().any(|carried| carried.id == label.id)
    }
}

/// A mark going on or coming off one message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mark {
    Read(bool),
    Starred(bool),
    Label { label: Label, on: bool },
}

impl Mark {
    /// Whether the message already carried this mark before the action, so
    /// the action did not change it.
    fn was_already_on(&self, message: &Before) -> bool {
        match self {
            Mark::Read(read) => message.read == *read,
            Mark::Starred(starred) => message.starred == *starred,
            Mark::Label { label, on } => message.has(label) == *on,
        }
    }

    /// The same kind of mark, set to what the message held before.
    fn as_it_was_on(&self, message: &Before) -> Mark {
        match self {
            Mark::Read(_) => Mark::Read(message.read),
            Mark::Starred(_) => Mark::Starred(message.starred),
            Mark::Label { label, .. } => Mark::Label {
                label: label.clone(),
                on: message.has(label),
            },
        }
    }
}

/// How long a subject may run on the Edit menu before it is cut short, so
/// the item stays readable and the menu does not stretch across the screen.
const MOST_OF_A_SUBJECT_ON_THE_MENU: usize = 60;

/// The last action somebody took on messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LastAction {
    /// One mark, the same for every message.
    Marked { mark: Mark, before: Vec<Before> },
    /// Every label taken off every message.
    LabelsRemoved { before: Vec<Before> },
}

impl LastAction {
    /// What the action is called, the way the menu that does it would say
    /// it: "Mark as Read", "Label Work", and "Remove every label" as the Label
    /// menu words it. A label taken off one message has no item of its own,
    /// since the label's item toggles, so it is "Remove label Work".
    pub fn name(&self) -> String {
        let mark = match self {
            LastAction::LabelsRemoved { .. } => return "Remove every label".to_string(),
            LastAction::Marked { mark, .. } => mark,
        };
        match mark {
            Mark::Read(true) => "Mark as Read".to_string(),
            Mark::Read(false) => "Mark as Unread".to_string(),
            Mark::Starred(true) => "Star".to_string(),
            Mark::Starred(false) => "Unstar".to_string(),
            Mark::Label { label, on: true } => format!("Label {}", label.name),
            Mark::Label { label, on: false } => format!("Remove label {}", label.name),
        }
    }

    fn before(&self) -> &[Before] {
        match self {
            LastAction::Marked { before, .. } | LastAction::LabelsRemoved { before } => before,
        }
    }

    /// The message's subject when there is one message with a subject, and
    /// the count otherwise, the way a command over the set names it.
    fn what_it_was_done_to(&self) -> String {
        match self.before() {
            [only] if !only.subject.trim().is_empty() => only.subject.trim().to_string(),
            all => how_many(all.len(), "message"),
        }
    }
}

/// Which way the one step goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Undo,
    Redo,
}

/// The one step kept, and which way it last went.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneStep {
    action: LastAction,
    undone: bool,
}

impl OneStep {
    /// The action just taken, not yet undone.
    pub fn new(action: LastAction) -> Self {
        Self {
            action,
            undone: false,
        }
    }

    pub fn action(&self) -> &LastAction {
        &self.action
    }

    /// Whether the step can go this way now: Undo until it has been undone,
    /// then Redo until it has been done again.
    pub fn offers(&self, direction: Direction) -> bool {
        match direction {
            Direction::Undo => !self.undone,
            Direction::Redo => self.undone,
        }
    }

    /// The step went this way.
    pub fn went(&mut self, direction: Direction) {
        self.undone = direction == Direction::Undo;
    }
}

/// What an undo changes: each message back to its own state before, leaving
/// out the ones the action did not change. Labels removed all at once come
/// back one label at a time, each on the message that carried it.
pub fn what_undo_does(action: &LastAction) -> Vec<(Before, Mark)> {
    match action {
        LastAction::Marked { mark, before } => before
            .iter()
            .filter(|message| !mark.was_already_on(message))
            .map(|message| (message.clone(), mark.as_it_was_on(message)))
            .collect(),
        LastAction::LabelsRemoved { before } => each_label_carried(before, true),
    }
}

/// What a redo changes: the action again, over the messages it changed.
pub fn what_redo_does(action: &LastAction) -> Vec<(Before, Mark)> {
    match action {
        LastAction::Marked { mark, before } => before
            .iter()
            .filter(|message| !mark.was_already_on(message))
            .map(|message| (message.clone(), mark.clone()))
            .collect(),
        LastAction::LabelsRemoved { before } => each_label_carried(before, false),
    }
}

fn each_label_carried(before: &[Before], on: bool) -> Vec<(Before, Mark)> {
    before
        .iter()
        .flat_map(|message| {
            message.labels.iter().map(move |label| {
                (
                    message.clone(),
                    Mark::Label {
                        label: label.clone(),
                        on,
                    },
                )
            })
        })
        .collect()
}

/// The Edit menu's item for the step, "&Undo Mark as Read: Quarterly
/// report\tCtrl+Z", keeping Undo's letter and key.
pub fn menu_label(action: &LastAction, direction: Direction) -> String {
    let (command, key) = match direction {
        Direction::Undo => ("&Undo", "Ctrl+Z"),
        Direction::Redo => ("&Redo", "Ctrl+Y"),
    };
    format!(
        "{command} {}: {}\t{key}",
        on_a_menu(&action.name()),
        on_a_menu(&action.what_it_was_done_to())
    )
}

/// Words from a message or a label's name, made safe to sit in a menu item.
/// An ampersand would give the next letter the item's key and a tab would
/// start its shortcut, so the first is doubled and the second made a space,
/// and a long subject is cut short.
fn on_a_menu(words: &str) -> String {
    let flat: String = words
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let short = match flat.chars().count() > MOST_OF_A_SUBJECT_ON_THE_MENU {
        true => {
            let kept: String = flat
                .chars()
                .take(MOST_OF_A_SUBJECT_ON_THE_MENU - 3)
                .collect();
            format!("{}...", kept.trim_end())
        }
        false => flat,
    };
    short.replace('&', "&&")
}

/// The one sentence after an undo.
pub fn undone(action: &LastAction) -> String {
    format!(
        "Undid {} on {}.",
        action.name(),
        action.what_it_was_done_to()
    )
}

/// The one sentence after a redo.
pub fn redone(action: &LastAction) -> String {
    format!(
        "Redid {} on {}.",
        action.name(),
        action.what_it_was_done_to()
    )
}

/// What Undo says in the message list when there is nothing to take back:
/// nothing done yet, or the one step already undone.
pub fn nothing_to_undo(step: Option<&OneStep>) -> &'static str {
    match step {
        None => "There is nothing to undo in this list yet.",
        Some(_) => "That was undone already. Redo does it again.",
    }
}

/// What Redo says in the message list when there is nothing to put back.
pub fn nothing_to_redo(_step: Option<&OneStep>) -> &'static str {
    "There is nothing to redo in this list. Redo puts back what Undo just took away."
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::status_sentences::{Voice, reads_as_a_persons_sentence};

    fn message(row_id: i64, subject: &str, read: bool, starred: bool) -> Before {
        Before {
            row_id,
            uid: row_id as u32 + 100,
            subject: subject.to_string(),
            read,
            starred,
            labels: Vec::new(),
        }
    }

    fn label(id: &str, name: &str) -> Label {
        Label {
            id: id.to_string(),
            name: name.to_string(),
            keyword: Some(name.to_lowercase()),
        }
    }

    fn labelled(mut before: Before, labels: &[Label]) -> Before {
        before.labels = labels.to_vec();
        before
    }

    fn rows_and_marks(changes: &[(Before, Mark)]) -> Vec<(i64, Mark)> {
        changes
            .iter()
            .map(|(message, mark)| (message.row_id, mark.clone()))
            .collect()
    }

    #[test]
    fn test_undoing_mark_as_read_over_a_mixed_set_puts_back_each_messages_own_state() {
        // Two unread and one read, all marked read. The undo unreads the two
        // and leaves the one that was read before anybody touched it.
        let action = LastAction::Marked {
            mark: Mark::Read(true),
            before: vec![
                message(1, "Quarterly report", false, false),
                message(2, "Lunch", true, false),
                message(3, "Invoice", false, true),
            ],
        };
        assert_eq!(
            rows_and_marks(&what_undo_does(&action)),
            vec![(1, Mark::Read(false)), (3, Mark::Read(false))]
        );
    }

    #[test]
    fn test_undoing_a_star_on_one_message_unstars_only_that_one() {
        let action = LastAction::Marked {
            mark: Mark::Starred(true),
            before: vec![message(7, "Quarterly report", true, false)],
        };
        let changes = what_undo_does(&action);
        assert_eq!(rows_and_marks(&changes), vec![(7, Mark::Starred(false))]);
        // The message is carried whole, because the server is told by its
        // number in its folder and the refusal names its subject.
        assert_eq!(changes[0].0.uid, 107);
        assert_eq!(changes[0].0.subject, "Quarterly report");
    }

    #[test]
    fn test_undoing_a_label_put_on_takes_it_off_only_where_it_was_not() {
        let work = label("w", "Work");
        let action = LastAction::Marked {
            mark: Mark::Label {
                label: work.clone(),
                on: true,
            },
            before: vec![
                labelled(
                    message(1, "Had it", false, false),
                    std::slice::from_ref(&work),
                ),
                message(2, "Lacked it", false, false),
            ],
        };
        assert_eq!(
            rows_and_marks(&what_undo_does(&action)),
            vec![(
                2,
                Mark::Label {
                    label: work,
                    on: false
                }
            )]
        );
    }

    #[test]
    fn test_undoing_every_label_removed_puts_each_back() {
        // One label at a time per message, each the one it carried.
        let work = label("w", "Work");
        let home = label("h", "Home");
        let action = LastAction::LabelsRemoved {
            before: vec![
                labelled(
                    message(1, "Both", false, false),
                    &[work.clone(), home.clone()],
                ),
                labelled(message(2, "None", false, false), &[]),
                labelled(
                    message(3, "Home only", false, false),
                    std::slice::from_ref(&home),
                ),
            ],
        };
        let on = |label: &Label| Mark::Label {
            label: label.clone(),
            on: true,
        };
        assert_eq!(
            rows_and_marks(&what_undo_does(&action)),
            vec![(1, on(&work)), (1, on(&home)), (3, on(&home))]
        );
    }

    #[test]
    fn test_redo_does_the_action_again() {
        let action = LastAction::Marked {
            mark: Mark::Read(true),
            before: vec![
                message(1, "Quarterly report", false, false),
                message(2, "Lunch", true, false),
            ],
        };
        assert_eq!(
            rows_and_marks(&what_redo_does(&action)),
            vec![(1, Mark::Read(true))]
        );

        let work = label("w", "Work");
        let removed = LastAction::LabelsRemoved {
            before: vec![labelled(
                message(4, "Kept", false, false),
                std::slice::from_ref(&work),
            )],
        };
        assert_eq!(
            rows_and_marks(&what_redo_does(&removed)),
            vec![(
                4,
                Mark::Label {
                    label: work,
                    on: false
                }
            )]
        );

        // The one step goes back and forth: after an Undo only Redo, after a
        // Redo only Undo.
        let mut step = OneStep::new(action);
        assert!(step.offers(Direction::Undo) && !step.offers(Direction::Redo));
        step.went(Direction::Undo);
        assert!(!step.offers(Direction::Undo) && step.offers(Direction::Redo));
        step.went(Direction::Redo);
        assert!(step.offers(Direction::Undo) && !step.offers(Direction::Redo));
    }

    #[test]
    fn test_the_menu_names_the_message_or_the_count() {
        let one = LastAction::Marked {
            mark: Mark::Read(true),
            before: vec![message(1, "Quarterly report", false, false)],
        };
        assert_eq!(
            menu_label(&one, Direction::Undo),
            "&Undo Mark as Read: Quarterly report\tCtrl+Z"
        );
        assert_eq!(
            menu_label(&one, Direction::Redo),
            "&Redo Mark as Read: Quarterly report\tCtrl+Y"
        );

        let four = LastAction::Marked {
            mark: Mark::Starred(false),
            before: (1..=4)
                .map(|row| message(row, "First of four", false, true))
                .collect(),
        };
        assert_eq!(
            menu_label(&four, Direction::Undo),
            "&Undo Unstar: 4 messages\tCtrl+Z"
        );

        let work = label("w", "Work");
        let names = [
            (Mark::Read(false), "Mark as Unread"),
            (Mark::Starred(true), "Star"),
            (
                Mark::Label {
                    label: work.clone(),
                    on: true,
                },
                "Label Work",
            ),
            (
                Mark::Label {
                    label: work,
                    on: false,
                },
                "Remove label Work",
            ),
        ];
        for (mark, name) in names {
            let action = LastAction::Marked {
                mark,
                before: vec![message(1, "A", false, false)],
            };
            assert_eq!(action.name(), name);
        }
        assert_eq!(
            LastAction::LabelsRemoved { before: vec![] }.name(),
            "Remove every label"
        );
    }

    #[test]
    fn test_a_subject_on_the_menu_cannot_take_a_letter_or_the_key() {
        // An ampersand in a menu label marks the next letter as the item's
        // key, and a tab starts the shortcut. A subject is a stranger's words.
        let action = LastAction::Marked {
            mark: Mark::Starred(true),
            before: vec![message(1, "Q&A\tnotes", false, false)],
        };
        assert_eq!(
            menu_label(&action, Direction::Undo),
            "&Undo Star: Q&&A notes\tCtrl+Z"
        );

        let long = "a".repeat(200);
        let label = menu_label(
            &LastAction::Marked {
                mark: Mark::Starred(true),
                before: vec![message(1, &long, false, false)],
            },
            Direction::Undo,
        );
        let (item, key) = label.split_once('\t').unwrap_or_default();
        assert_eq!(key, "Ctrl+Z");
        assert!(item.chars().count() <= 80, "{item}");
        assert!(item.ends_with("..."), "{item}");
    }

    #[test]
    fn test_every_undo_sentence_reads_as_a_persons_sentence() {
        let one = LastAction::Marked {
            mark: Mark::Read(true),
            before: vec![message(1, "Quarterly report", false, false)],
        };
        let several = LastAction::LabelsRemoved {
            before: vec![message(1, "A", false, false), message(2, "B", false, false)],
        };
        assert_eq!(undone(&one), "Undid Mark as Read on Quarterly report.");
        assert_eq!(redone(&several), "Redid Remove every label on 2 messages.");
        let undone_step = {
            let mut step = OneStep::new(one.clone());
            step.went(Direction::Undo);
            step
        };
        let sentences = [
            undone(&one),
            undone(&several),
            redone(&one),
            redone(&several),
            nothing_to_undo(None).to_string(),
            nothing_to_undo(Some(&undone_step)).to_string(),
            nothing_to_redo(None).to_string(),
            nothing_to_redo(Some(&OneStep::new(one))).to_string(),
        ];
        for sentence in sentences {
            reads_as_a_persons_sentence(&sentence, Voice::Answer)
                .unwrap_or_else(|why| panic!("{sentence:?}: {why}"));
        }
        assert_eq!(
            nothing_to_undo(None),
            "There is nothing to undo in this list yet."
        );
    }
}

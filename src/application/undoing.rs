//! Undo and Redo of the last thing somebody did to messages: a mark as read
//! or unread, a star, a label, a move, a delete or a copy, with the message
//! named (#47, GAP-02).
//!
//! The tester on 2026-09-15 asked for "the last delete, move, copy, mark as
//! read or unread, star ... as 'Undo delete' or 'Undo move' with the item
//! named, for a bounded time or until the next action; and Redo of that."
//! The marks came first (13-07); moves, deletes and copies are below them
//! (13-08).
//!
//! # A move is undone from where the store says it is now
//!
//! A move made here first waits for its server (#86). While the waiting row
//! is there the server has heard nothing, so the undo ends the row and the
//! message is back where the server still has it, and the server is never
//! asked anything. Answering a waiting move with a move back instead would
//! send the server a move into the folder it already holds the message in.
//! Once the server has carried it out, the row carries the folder and the
//! number the server gave it, and the undo is a new move back, here first
//! and then at the server, the way the action went. What cannot come back
//! is refused in a sentence naming the message: a delete the server took for
//! good, a move to another account, a message this computer has not read back
//! yet, and one whose move the server is being told about at that moment.
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

use crate::data::message_cache::moves_waiting::AWaitingMove;
use crate::service::caldav::how_many;

/// A label as an undo needs it: which one it is here, what it is called, and
/// what it travels as to the server, when it travels at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub id: String,
    pub name: String,
    pub keyword: Option<String>,
}

impl From<&crate::data::message_cache::Tag> for Label {
    fn from(tag: &crate::data::message_cache::Tag) -> Self {
        Self {
            id: tag.id.clone(),
            name: tag.name.clone(),
            keyword: tag.keyword.clone(),
        }
    }
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
    /// A move, a delete or a copy, and where each message was before it.
    Moved {
        moving: Moving,
        went: Vec<WhereItWas>,
    },
}

impl LastAction {
    /// What the action is called, the way the menu that does it would say
    /// it: "Mark as Read", "Label Work", and "Remove every label" as the Label
    /// menu words it. A label taken off one message has no item of its own,
    /// since the label's item toggles, so it is "Remove label Work".
    pub fn name(&self) -> String {
        let mark = match self {
            LastAction::LabelsRemoved { .. } => return "Remove every label".to_string(),
            LastAction::Moved { moving, .. } => return moving.name(),
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

    /// The subject of each message the action took, in order.
    fn subjects(&self) -> Vec<&str> {
        match self {
            LastAction::Marked { before, .. } | LastAction::LabelsRemoved { before } => before
                .iter()
                .map(|message| message.subject.as_str())
                .collect(),
            LastAction::Moved { went, .. } => went
                .iter()
                .map(|message| message.subject.as_str())
                .collect(),
        }
    }

    /// The message's subject when there is one message with a subject, and
    /// the count otherwise, the way a command over the set names it.
    fn what_it_was_done_to(&self) -> String {
        match self.subjects().as_slice() {
            [only] if !only.trim().is_empty() => only.trim().to_string(),
            all => how_many(all.len(), "message"),
        }
    }
}

/// What a move, a delete or a copy did to the set, which is what names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Moving {
    Move { to: String },
    Delete,
    DeletePermanently,
    Copy { to: String },
}

/// How one message went, which decides what its undo can do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WentBy {
    /// Made here first and told to its server afterwards (#86).
    ItsServer,
    /// Between two folders on this computer, with no server to tell: mail
    /// kept here, such as a POP account's.
    ThisComputerOnly,
    /// To a folder of another account.
    AnotherAccount,
}

/// One message as it was before a move, a delete or a copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhereItWas {
    /// The message's row. A move or a delete keeps it wherever it goes.
    pub row_id: i64,
    pub account_id: String,
    /// The folder it was in here, which is where an undo puts it back.
    pub folder_path: String,
    pub uid: u32,
    pub subject: String,
    /// Where the action sent it: the folder moved or copied to, the trash,
    /// or nothing for a delete that took it off the server.
    pub sent_to: Option<String>,
    pub went_by: WentBy,
    /// For a copy, the copy's row, which is what an undo takes away. A redo
    /// makes a new one, so this is kept per message rather than beside the
    /// set, where it could fall out of step.
    pub copy_row: Option<i64>,
}

/// A message's row as the store holds it now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhereItIsHere {
    pub row_id: i64,
    pub folder_path: String,
    pub uid: u32,
    /// Marked deleted, which is what a delete off the server leaves until
    /// the next read of its folder forgets it.
    pub deleted: bool,
}

/// What the store says about one message's row at the moment of an undo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatTheStoreSays {
    /// A change made here is waiting and its server has heard nothing.
    StillWaiting {
        waiting: AWaitingMove,
        here: WhereItIsHere,
    },
    /// Nothing is waiting: the row is where it is.
    Settled(WhereItIsHere),
    /// The row is not on this computer: the server moved the message and
    /// could not say where it landed, so the next read of that folder brings
    /// it.
    Gone,
    /// Its account's server is being told about waiting changes at this
    /// moment, so what the row says may change under an undo.
    BeingToldNow,
}

/// What one message's undo or redo does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OneChange {
    /// End the waiting change here: the server never hears of either.
    EndTheWaitingRow(AWaitingMove),
    /// Move it from where it is now, here first and then at the server.
    Move { from: WhereItIsHere, to: String },
    /// Delete it the way the Delete key does, or Delete Permanently.
    Delete {
        row: WhereItIsHere,
        permanently: bool,
    },
    /// Copy it again.
    Copy { from: WhereItIsHere, to: String },
    /// Send a copy to the trash, and never off the server outright, so a
    /// number that turned out wrong cannot destroy a message.
    TrashTheCopy(WhereItIsHere),
    /// Move the row between two folders on this computer.
    MoveOnThisComputer { row_id: i64, to: String },
    /// Nothing is done, and this is said.
    Refused(String),
}

impl Moving {
    /// What the action is called, the way the menu that does it says it.
    fn name(&self) -> String {
        match self {
            Moving::Move { to } => format!("Move to {to}"),
            Moving::Delete => "Delete".to_string(),
            Moving::DeletePermanently => "Delete Permanently".to_string(),
            Moving::Copy { to } => format!("Copy to {to}"),
        }
    }
}

/// The row an undo or a redo reads for one message: the copy's own for the
/// undo of a copy, the message's otherwise.
pub fn the_row_to_read(message: &WhereItWas, direction: Direction) -> i64 {
    match (direction, message.copy_row) {
        (Direction::Undo, Some(copy)) => copy,
        _ => message.row_id,
    }
}

/// What undoing a move, a delete or a copy does to one message, given what
/// the store says about it now.
///
/// A waiting change the server has heard nothing of is ended here, never
/// answered with a move back, because the server still holds the message
/// where it was and a move back would name the folder it is already in. The
/// exception is a message moved twice before the server heard either: ending
/// the row would undo both, so the second is taken back with a new ask.
pub fn what_undo_does_to(
    message: &WhereItWas,
    moving: &Moving,
    store: WhatTheStoreSays,
) -> OneChange {
    use WhatTheStoreSays::{BeingToldNow, Gone, Settled, StillWaiting};
    let who = named(&message.subject);
    if message.went_by == WentBy::AnotherAccount {
        return OneChange::Refused(went_to_another_account(who));
    }
    let copying = matches!(moving, Moving::Copy { .. });
    let permanently = *moving == Moving::DeletePermanently;
    let back = &message.folder_path;
    match store {
        BeingToldNow => OneChange::Refused(being_told_now(who)),
        Gone if permanently => OneChange::Refused(cannot_come_back(who)),
        Gone => OneChange::Refused(not_read_back_yet(who)),
        StillWaiting { waiting, .. } if copying => OneChange::EndTheWaitingRow(waiting),
        Settled(copy) if copying => OneChange::TrashTheCopy(copy),
        StillWaiting { waiting, .. } if waiting.from_folder_path == *back => {
            OneChange::EndTheWaitingRow(waiting)
        }
        StillWaiting { .. } if permanently => OneChange::Refused(moved_and_then_deleted(who)),
        StillWaiting { here, .. } => OneChange::Move {
            from: here,
            to: back.clone(),
        },
        Settled(here) if here.deleted => OneChange::Refused(cannot_come_back(who)),
        Settled(here) if here.folder_path == *back => OneChange::Refused(already_in(who, back)),
        Settled(here) if message.went_by == WentBy::ThisComputerOnly => {
            OneChange::MoveOnThisComputer {
                row_id: here.row_id,
                to: back.clone(),
            }
        }
        Settled(here) => OneChange::Move {
            from: here,
            to: back.clone(),
        },
    }
}

/// What redoing a move, a delete or a copy does to one message: the action
/// again, from where the undo left it.
///
/// An undo that moved a message back and is still waiting for the server
/// left the server holding it where the action sent it, so the redo ends
/// that wait rather than asking for a move into the folder the server holds
/// it in.
pub fn what_redo_does_to(
    message: &WhereItWas,
    moving: &Moving,
    store: WhatTheStoreSays,
) -> OneChange {
    use WhatTheStoreSays::{BeingToldNow, Gone, Settled, StillWaiting};
    let who = named(&message.subject);
    if message.went_by == WentBy::AnotherAccount {
        return OneChange::Refused(went_to_another_account(who));
    }
    let copying = matches!(moving, Moving::Copy { .. });
    let here = match store {
        BeingToldNow => return OneChange::Refused(being_told_now(who)),
        Gone => return OneChange::Refused(not_read_back_yet(who)),
        StillWaiting { waiting, .. }
            if !copying && message.sent_to.as_ref() == Some(&waiting.from_folder_path) =>
        {
            return OneChange::EndTheWaitingRow(waiting);
        }
        StillWaiting { here, .. } | Settled(here) => here,
    };
    match moving {
        Moving::Move { to } if here.folder_path == *to => OneChange::Refused(already_in(who, to)),
        Moving::Move { to } => OneChange::Move {
            from: here,
            to: to.clone(),
        },
        Moving::Delete => OneChange::Delete {
            row: here,
            permanently: false,
        },
        Moving::DeletePermanently => OneChange::Delete {
            row: here,
            permanently: true,
        },
        Moving::Copy { to } => OneChange::Copy {
            from: here,
            to: to.clone(),
        },
    }
}

/// The message's subject, or words standing in for one it does not have.
fn named(subject: &str) -> &str {
    match subject.trim() {
        "" => "That message",
        subject => subject,
    }
}

fn went_to_another_account(who: &str) -> String {
    format!(
        "{who} went to another account, so it cannot be taken back from here. Move it from \
         that account instead."
    )
}

fn being_told_now(who: &str) -> String {
    format!("{who} is being changed at the server right now. Try again shortly.")
}

fn not_read_back_yet(who: &str) -> String {
    format!(
        "{who} was moved at the server and this computer has not read where yet. Refresh the \
         folder it went to and move it back from there."
    )
}

fn cannot_come_back(who: &str) -> String {
    format!("{who} was deleted permanently, so it cannot come back.")
}

fn moved_and_then_deleted(who: &str) -> String {
    format!(
        "{who} was moved and then deleted before the server heard of either, so it cannot \
         come back from here."
    )
}

fn already_in(who: &str, folder: &str) -> String {
    format!("{who} is already in {folder}.")
}

/// The one sentence after an undo or a redo of a move, a delete or a copy:
/// what came back, and what could not with the first reason, once for the
/// set.
pub fn after_moving_back(
    action: &LastAction,
    direction: Direction,
    done: usize,
    refused: &[String],
) -> String {
    let (did, nothing, all_of_it) = match direction {
        Direction::Undo => ("Undid", "Nothing was undone.", undone(action)),
        Direction::Redo => ("Redid", "Nothing was redone.", redone(action)),
    };
    match (done, refused) {
        (_, []) => all_of_it,
        (0, [only]) => only.clone(),
        (0, [first, ..]) => format!("{nothing} {first}"),
        (_, [first, ..]) => format!(
            "{did} {} on {}, not on {}. {first}",
            action.name(),
            how_many(done, "message"),
            how_many(refused.len(), "message")
        ),
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
        // No mark: each message's undo is decided from the store, by
        // `what_undo_does_to`.
        LastAction::Moved { .. } => Vec::new(),
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
        LastAction::Moved { .. } => Vec::new(),
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

    // ── A move, a delete or a copy ─────────────────────────────────────────

    use crate::data::message_cache::moves_waiting::WhatAWaitingMoveDoes;

    /// Quarterly report, row 7, in the Inbox under 42, moved to Archive.
    fn quarterly_report(went_by: WentBy, sent_to: Option<&str>) -> WhereItWas {
        WhereItWas {
            row_id: 7,
            account_id: "an account".to_string(),
            folder_path: "INBOX".to_string(),
            uid: 42,
            subject: "Quarterly report".to_string(),
            sent_to: sent_to.map(str::to_string),
            went_by,
            copy_row: None,
        }
    }

    fn here(row_id: i64, folder_path: &str, uid: u32) -> WhereItIsHere {
        WhereItIsHere {
            row_id,
            folder_path: folder_path.to_string(),
            uid,
            deleted: false,
        }
    }

    /// A waiting row asking the server to do `what`, from where it still
    /// holds the message.
    fn waiting(row: i64, from: &str, uid: u32, what: WhatAWaitingMoveDoes) -> AWaitingMove {
        AWaitingMove {
            message_row_id: row,
            account_id: "an account".to_string(),
            from_folder_path: from.to_string(),
            uid,
            what,
            asked_at: "2026-09-25T09:00:00Z".to_string(),
        }
    }

    fn to_archive() -> Moving {
        Moving::Move {
            to: "Archive".to_string(),
        }
    }

    fn into_archive() -> WhatAWaitingMoveDoes {
        WhatAWaitingMoveDoes::Move {
            into_folder_path: "Archive".to_string(),
        }
    }

    #[test]
    fn test_undoing_a_move_the_server_has_not_heard_ends_the_waiting_row_rather_than_moving_back() {
        // The server still has it in the Inbox. A move back would ask it to
        // move a message into the folder it already holds it in.
        let message = quarterly_report(WentBy::ItsServer, Some("Archive"));
        let asked = waiting(7, "INBOX", 42, into_archive());
        let store = WhatTheStoreSays::StillWaiting {
            waiting: asked.clone(),
            here: here(7, "Archive", 4_000_000_001),
        };
        assert_eq!(
            what_undo_does_to(&message, &to_archive(), store),
            OneChange::EndTheWaitingRow(asked)
        );
    }

    #[test]
    fn test_undoing_a_move_the_server_made_moves_it_back_from_where_it_is_now() {
        // Carried out: the row carries Archive and the number the server gave
        // it there, and the undo is a move from there to the Inbox.
        let message = quarterly_report(WentBy::ItsServer, Some("Archive"));
        let store = WhatTheStoreSays::Settled(here(7, "Archive", 310));
        assert_eq!(
            what_undo_does_to(&message, &to_archive(), store),
            OneChange::Move {
                from: here(7, "Archive", 310),
                to: "INBOX".to_string(),
            }
        );

        // Moved twice before the server heard either, Inbox to Archive and
        // then Archive to Work: undoing the second puts it back in Archive,
        // which is a new ask, since ending the row would put it in the Inbox.
        let second = WhereItWas {
            folder_path: "Archive".to_string(),
            sent_to: Some("Work".to_string()),
            ..message
        };
        let store = WhatTheStoreSays::StillWaiting {
            waiting: waiting(
                7,
                "INBOX",
                42,
                WhatAWaitingMoveDoes::Move {
                    into_folder_path: "Work".to_string(),
                },
            ),
            here: here(7, "Work", 4_000_000_002),
        };
        assert_eq!(
            what_undo_does_to(
                &second,
                &Moving::Move {
                    to: "Work".to_string()
                },
                store
            ),
            OneChange::Move {
                from: here(7, "Work", 4_000_000_002),
                to: "Archive".to_string(),
            }
        );

        // Already back where it was, because the server refused and the
        // refusal put it back: nothing to do, and said.
        let message = quarterly_report(WentBy::ItsServer, Some("Archive"));
        assert_eq!(
            what_undo_does_to(
                &message,
                &to_archive(),
                WhatTheStoreSays::Settled(here(7, "INBOX", 42))
            ),
            OneChange::Refused("Quarterly report is already in INBOX.".to_string())
        );
    }

    #[test]
    fn test_undoing_a_delete_to_the_trash_moves_it_back_from_the_trash() {
        let message = quarterly_report(WentBy::ItsServer, Some("Trash"));
        let store = WhatTheStoreSays::Settled(here(7, "Trash", 88));
        assert_eq!(
            what_undo_does_to(&message, &Moving::Delete, store),
            OneChange::Move {
                from: here(7, "Trash", 88),
                to: "INBOX".to_string(),
            }
        );
        // Still waiting, it ends here like a move.
        let asked = waiting(
            7,
            "INBOX",
            42,
            WhatAWaitingMoveDoes::DeleteToTrash {
                trash_path: "Trash".to_string(),
            },
        );
        let store = WhatTheStoreSays::StillWaiting {
            waiting: asked.clone(),
            here: here(7, "Trash", 4_000_000_003),
        };
        assert_eq!(
            what_undo_does_to(&message, &Moving::Delete, store),
            OneChange::EndTheWaitingRow(asked)
        );
    }

    #[test]
    fn test_delete_permanently_still_waiting_comes_back() {
        let message = quarterly_report(WentBy::ItsServer, None);
        let asked = waiting(7, "INBOX", 42, WhatAWaitingMoveDoes::DeleteOutright);
        let store = WhatTheStoreSays::StillWaiting {
            waiting: asked.clone(),
            here: WhereItIsHere {
                deleted: true,
                ..here(7, "INBOX", 42)
            },
        };
        assert_eq!(
            what_undo_does_to(&message, &Moving::DeletePermanently, store),
            OneChange::EndTheWaitingRow(asked)
        );

        // Moved and then deleted permanently before the server heard either:
        // ending the row would lose the move, so it is refused.
        let moved_first = WhereItWas {
            folder_path: "Archive".to_string(),
            ..quarterly_report(WentBy::ItsServer, None)
        };
        let store = WhatTheStoreSays::StillWaiting {
            waiting: waiting(7, "INBOX", 42, WhatAWaitingMoveDoes::DeleteOutright),
            here: WhereItIsHere {
                deleted: true,
                ..here(7, "Archive", 4_000_000_004)
            },
        };
        assert_eq!(
            what_undo_does_to(&moved_first, &Moving::DeletePermanently, store),
            OneChange::Refused(
                "Quarterly report was moved and then deleted before the server heard of \
                 either, so it cannot come back from here."
                    .to_string()
            )
        );
    }

    #[test]
    fn test_delete_permanently_the_server_took_is_refused_saying_it_cannot_come_back() {
        let message = quarterly_report(WentBy::ItsServer, None);
        let cannot = OneChange::Refused(
            "Quarterly report was deleted permanently, so it cannot come back.".to_string(),
        );
        let taken = WhatTheStoreSays::Settled(WhereItIsHere {
            deleted: true,
            ..here(7, "INBOX", 42)
        });
        assert_eq!(
            what_undo_does_to(&message, &Moving::DeletePermanently, taken.clone()),
            cannot
        );
        // Forgotten by the next read of its folder: the same answer.
        assert_eq!(
            what_undo_does_to(&message, &Moving::DeletePermanently, WhatTheStoreSays::Gone),
            cannot
        );
        // The ordinary Delete inside the trash takes it off the server too.
        let in_the_trash = WhereItWas {
            folder_path: "Trash".to_string(),
            ..message
        };
        assert_eq!(
            what_undo_does_to(&in_the_trash, &Moving::Delete, taken),
            cannot
        );
    }

    #[test]
    fn test_a_crossing_is_refused_with_a_sentence() {
        let message = quarterly_report(WentBy::AnotherAccount, Some("Work"));
        let refused = OneChange::Refused(
            "Quarterly report went to another account, so it cannot be taken back from here. \
             Move it from that account instead."
                .to_string(),
        );
        for store in [
            WhatTheStoreSays::Settled(here(7, "Work", 5)),
            WhatTheStoreSays::StillWaiting {
                waiting: waiting(7, "INBOX", 42, into_archive()),
                here: here(7, "Work", 5),
            },
        ] {
            assert_eq!(
                what_undo_does_to(&message, &to_archive(), store.clone()),
                refused
            );
            assert_eq!(what_redo_does_to(&message, &to_archive(), store), refused);
        }
    }

    #[test]
    fn test_undoing_a_copy_sends_the_copy_to_the_trash() {
        let copy = WhereItWas {
            copy_row: Some(70),
            ..quarterly_report(WentBy::ItsServer, Some("Archive"))
        };
        let copying = Moving::Copy {
            to: "Archive".to_string(),
        };
        // The undo reads the copy's row, the redo the original's.
        assert_eq!(the_row_to_read(&copy, Direction::Undo), 70);
        assert_eq!(the_row_to_read(&copy, Direction::Redo), 7);
        // At the server: to the trash, never outright.
        assert_eq!(
            what_undo_does_to(
                &copy,
                &copying,
                WhatTheStoreSays::Settled(here(70, "Archive", 311))
            ),
            OneChange::TrashTheCopy(here(70, "Archive", 311))
        );
        // Not yet at the server: the copy made here goes and nothing is sent.
        let asked = waiting(
            70,
            "INBOX",
            42,
            WhatAWaitingMoveDoes::Copy {
                into_folder_path: "Archive".to_string(),
            },
        );
        assert_eq!(
            what_undo_does_to(
                &copy,
                &copying,
                WhatTheStoreSays::StillWaiting {
                    waiting: asked.clone(),
                    here: here(70, "Archive", 4_000_000_005),
                }
            ),
            OneChange::EndTheWaitingRow(asked)
        );
        // A redo copies the original again.
        assert_eq!(
            what_redo_does_to(
                &copy,
                &copying,
                WhatTheStoreSays::Settled(here(7, "INBOX", 42))
            ),
            OneChange::Copy {
                from: here(7, "INBOX", 42),
                to: "Archive".to_string(),
            }
        );
    }

    #[test]
    fn test_a_message_the_server_moved_and_this_computer_dropped_is_refused_saying_refresh() {
        let message = quarterly_report(WentBy::ItsServer, Some("Archive"));
        let refresh = OneChange::Refused(
            "Quarterly report was moved at the server and this computer has not read where \
             yet. Refresh the folder it went to and move it back from there."
                .to_string(),
        );
        assert_eq!(
            what_undo_does_to(&message, &to_archive(), WhatTheStoreSays::Gone),
            refresh
        );
        assert_eq!(
            what_undo_does_to(&message, &Moving::Delete, WhatTheStoreSays::Gone),
            refresh
        );
    }

    #[test]
    fn test_a_move_being_told_to_the_server_now_is_refused_saying_try_again() {
        let message = quarterly_report(WentBy::ItsServer, Some("Archive"));
        let again = OneChange::Refused(
            "Quarterly report is being changed at the server right now. Try again shortly."
                .to_string(),
        );
        for moving in [to_archive(), Moving::Delete, Moving::DeletePermanently] {
            assert_eq!(
                what_undo_does_to(&message, &moving, WhatTheStoreSays::BeingToldNow),
                again
            );
            assert_eq!(
                what_redo_does_to(&message, &moving, WhatTheStoreSays::BeingToldNow),
                again
            );
        }
    }

    #[test]
    fn test_a_local_delete_moves_back_between_local_folders() {
        // A POP account's message moved to the Trash folder on this
        // computer: moved back here, and nothing is asked of any server.
        let message = quarterly_report(WentBy::ThisComputerOnly, Some("Trash"));
        assert_eq!(
            what_undo_does_to(
                &message,
                &Moving::Delete,
                WhatTheStoreSays::Settled(here(7, "Trash", 42))
            ),
            OneChange::MoveOnThisComputer {
                row_id: 7,
                to: "INBOX".to_string(),
            }
        );
        // Taken off this computer, it is refused like any delete for good.
        assert_eq!(
            what_undo_does_to(
                &message,
                &Moving::Delete,
                WhatTheStoreSays::Settled(WhereItIsHere {
                    deleted: true,
                    ..here(7, "INBOX", 42)
                })
            ),
            OneChange::Refused(
                "Quarterly report was deleted permanently, so it cannot come back.".to_string()
            )
        );
    }

    #[test]
    fn test_the_menu_names_a_move_and_its_folder() {
        let one = LastAction::Moved {
            moving: to_archive(),
            went: vec![quarterly_report(WentBy::ItsServer, Some("Archive"))],
        };
        assert_eq!(
            menu_label(&one, Direction::Undo),
            "&Undo Move to Archive: Quarterly report\tCtrl+Z"
        );
        let three = LastAction::Moved {
            moving: Moving::Delete,
            went: vec![quarterly_report(WentBy::ItsServer, Some("Trash")); 3],
        };
        assert_eq!(
            menu_label(&three, Direction::Redo),
            "&Redo Delete: 3 messages\tCtrl+Y"
        );
        for (moving, name) in [
            (Moving::DeletePermanently, "Delete Permanently"),
            (
                Moving::Copy {
                    to: "Work & play".to_string(),
                },
                "Copy to Work & play",
            ),
        ] {
            let action = LastAction::Moved {
                moving,
                went: Vec::new(),
            };
            assert_eq!(action.name(), name);
        }
        assert_eq!(undone(&one), "Undid Move to Archive on Quarterly report.");
    }

    #[test]
    fn test_redo_of_a_move_moves_it_again() {
        let message = quarterly_report(WentBy::ItsServer, Some("Archive"));
        // Undone here before the server heard: the row is back in the Inbox
        // and nothing waits, so the redo is the move again.
        assert_eq!(
            what_redo_does_to(
                &message,
                &to_archive(),
                WhatTheStoreSays::Settled(here(7, "INBOX", 42))
            ),
            OneChange::Move {
                from: here(7, "INBOX", 42),
                to: "Archive".to_string(),
            }
        );
        // Undone by a move back the server has not heard yet: the server
        // still has it in Archive, so the redo ends that wait rather than
        // asking for a move into the folder the server holds it in.
        let back = waiting(
            7,
            "Archive",
            310,
            WhatAWaitingMoveDoes::Move {
                into_folder_path: "INBOX".to_string(),
            },
        );
        assert_eq!(
            what_redo_does_to(
                &message,
                &to_archive(),
                WhatTheStoreSays::StillWaiting {
                    waiting: back.clone(),
                    here: here(7, "INBOX", 4_000_000_006),
                }
            ),
            OneChange::EndTheWaitingRow(back)
        );
        // A delete done again goes the way the Delete key does.
        let deleted = quarterly_report(WentBy::ItsServer, Some("Trash"));
        assert_eq!(
            what_redo_does_to(
                &deleted,
                &Moving::Delete,
                WhatTheStoreSays::Settled(here(7, "INBOX", 42))
            ),
            OneChange::Delete {
                row: here(7, "INBOX", 42),
                permanently: false,
            }
        );
        assert_eq!(
            what_redo_does_to(
                &quarterly_report(WentBy::ItsServer, None),
                &Moving::DeletePermanently,
                WhatTheStoreSays::Settled(here(7, "INBOX", 42))
            ),
            OneChange::Delete {
                row: here(7, "INBOX", 42),
                permanently: true,
            }
        );
    }

    #[test]
    fn test_what_could_not_come_back_is_said_once_with_the_count() {
        let three = LastAction::Moved {
            moving: Moving::Delete,
            went: vec![quarterly_report(WentBy::ItsServer, Some("Trash")); 3],
        };
        let reason =
            "Quarterly report was deleted permanently, so it cannot come back.".to_string();
        assert_eq!(
            after_moving_back(&three, Direction::Undo, 3, &[]),
            "Undid Delete on 3 messages."
        );
        assert_eq!(
            after_moving_back(&three, Direction::Undo, 2, std::slice::from_ref(&reason)),
            "Undid Delete on 2 messages, not on 1 message. Quarterly report was deleted \
             permanently, so it cannot come back."
        );
        assert_eq!(
            after_moving_back(&three, Direction::Undo, 0, std::slice::from_ref(&reason)),
            reason
        );
        assert_eq!(
            after_moving_back(
                &three,
                Direction::Redo,
                0,
                &[reason.clone(), reason.clone()]
            ),
            "Nothing was redone. Quarterly report was deleted permanently, so it cannot come \
             back."
        );
        // Every refusal names the message, with a stand-in when it has no
        // subject, and reads as a person's sentence.
        let nameless = WhereItWas {
            subject: "  ".to_string(),
            ..quarterly_report(WentBy::AnotherAccount, Some("Work"))
        };
        let OneChange::Refused(sentence) =
            what_undo_does_to(&nameless, &to_archive(), WhatTheStoreSays::Gone)
        else {
            panic!("a crossing is refused");
        };
        assert!(sentence.starts_with("That message went"), "{sentence}");
        let message = quarterly_report(WentBy::ItsServer, Some("Archive"));
        let sentences = [
            WhatTheStoreSays::Gone,
            WhatTheStoreSays::BeingToldNow,
            WhatTheStoreSays::Settled(here(7, "INBOX", 42)),
        ]
        .into_iter()
        .filter_map(
            |store| match what_undo_does_to(&message, &to_archive(), store) {
                OneChange::Refused(sentence) => Some(sentence),
                _ => None,
            },
        )
        .chain([
            sentence,
            after_moving_back(&three, Direction::Undo, 2, std::slice::from_ref(&reason)),
            after_moving_back(&three, Direction::Redo, 0, &[reason.clone(), reason]),
        ])
        .collect::<Vec<_>>();
        assert_eq!(sentences.len(), 6);
        for sentence in sentences {
            reads_as_a_persons_sentence(&sentence, Voice::Answer)
                .unwrap_or_else(|why| panic!("{sentence:?}: {why}"));
        }
    }
}

//! Taking back a deletion somebody made on this computer, because they asked
//! for the thing back with Edit, Undo (#47, 13-09).
//!
//! The decision about what an undo means is
//! [`crate::application::undoing`]'s, which has no database in it. This holds
//! what a deleted item was, kept by the undo for the one step, in the store's
//! own shape: a copy of its fields written out again beside the entry types
//! would drift the first time a column was added.

use super::{CalendarEventEntry, ContactEntry, NoteEntry, ReminderEntry, TaskEntry};

/// Everything a deleted contact, event, task, note or reminder was, as the
/// store held it the moment before the delete.
///
/// Held in memory by the one step only, replaced by the next action and never
/// written anywhere, because it is the whole of somebody's item and the undo
/// is the only thing that needs it.
#[derive(Debug, Clone, PartialEq)]
pub enum Record {
    Contact(ContactEntry),
    Event(CalendarEventEntry),
    Task(TaskEntry),
    Note(NoteEntry),
    Reminder(ReminderEntry),
}

impl Record {
    /// The identifier the row had.
    pub fn id(&self) -> &str {
        match self {
            Record::Contact(contact) => &contact.id,
            Record::Event(event) => &event.id,
            Record::Task(task) => &task.id,
            Record::Note(note) => &note.id,
            Record::Reminder(reminder) => &reminder.id,
        }
    }
}

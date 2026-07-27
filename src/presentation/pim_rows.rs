//! Cell text for the contacts, calendar, reminders, tasks, and notes lists.
//!
//! These lists run in virtual mode for the same reason the message list does: a
//! native list control filled row by row stops being usable somewhere around
//! ten thousand items, and an address book or a task history reaches that. In
//! virtual mode the control asks for the text of a cell while it is painting,
//! so memory is proportional to what is on screen rather than to what exists,
//! and UI Automation still reports the real set size. A screen reader says
//! "row 12 of 40,000" and means it.
//!
//! Everything here is therefore a pure function over data already in memory.
//! The paint callback cannot query the database, cannot block, and has nowhere
//! to report an error to.
//!
//! Cells are also what a screen reader reads for a row, so a flag column says
//! "Yes" rather than repeating its own heading: a report row is announced as
//! "heading, value", and a cell holding the word "Done" under a heading of
//! "Done" comes out twice on every completed row.

use super::ui_types::{CalendarEventItem, ContactItem, NoteItem, ReminderItem, TaskItem};

/// Shown in a cell whose row is not loaded.
pub const PLACEHOLDER: &str = "Loading";

/// Contacts: name, email, phone, company.
pub fn contact_cell(contact: &ContactItem, column: i32) -> String {
    match column {
        0 => non_empty(&contact.name, "No name"),
        1 => contact.email.clone(),
        2 => contact.phone.clone(),
        3 => contact.company.clone(),
        _ => String::new(),
    }
}

/// Calendar: time, summary, calendar, location, status.
pub fn event_cell(event: &CalendarEventItem, column: i32) -> String {
    match column {
        0 => {
            if event.is_all_day {
                "All day".to_string()
            } else {
                event.start.clone()
            }
        }
        1 => non_empty(&event.summary, "No title"),
        2 => event.calendar_name.clone().unwrap_or_default(),
        3 => event.location.clone(),
        4 => event.status.clone(),
        _ => String::new(),
    }
}

/// Reminders: done, title, due, priority.
pub fn reminder_cell(reminder: &ReminderItem, column: i32) -> String {
    match column {
        0 => flag(reminder.is_completed),
        1 => non_empty(&reminder.title, "No title"),
        2 => reminder.due_datetime.clone().unwrap_or_default(),
        3 => reminder.priority.clone(),
        _ => String::new(),
    }
}

/// Tasks: done, title, due, priority.
pub fn task_cell(task: &TaskItem, column: i32) -> String {
    match column {
        0 => flag(task.is_completed),
        1 => non_empty(&task.title, "No title"),
        2 => task.due_date.clone().unwrap_or_default(),
        3 => task.priority.clone(),
        _ => String::new(),
    }
}

/// Notes: title, last modified.
pub fn note_cell(note: &NoteItem, column: i32) -> String {
    match column {
        0 => {
            let title = non_empty(&note.title, "Untitled");
            // The pin is part of the title cell rather than a column of its
            // own, because a whole column that is empty on almost every row
            // costs listening time on all of them to say nothing.
            if note.pinned {
                format!("Pinned. {}", title)
            } else {
                title
            }
        }
        1 => note.updated_at.clone(),
        _ => String::new(),
    }
}

/// "Yes" or nothing, never the column's own word.
fn flag(value: bool) -> String {
    if value { "Yes" } else { "" }.to_string()
}

/// Fall back to a stated absence rather than an empty cell.
///
/// An empty cell in the first column reads as nothing at all, which sounds
/// exactly like a row that failed to load.
fn non_empty(value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contact() -> ContactItem {
        ContactItem {
            id: "c1".to_string(),
            name: "Grace Hopper".to_string(),
            email: "grace@example.com".to_string(),
            phone: "555 0100".to_string(),
            company: "Navy".to_string(),
            favorite: false,
        }
    }

    fn task() -> TaskItem {
        TaskItem {
            id: "t1".to_string(),
            title: "File the report".to_string(),
            description: None,
            due_date: Some("2026-07-30".to_string()),
            is_completed: false,
            priority: "normal".to_string(),
            task_list_id: None,
            parent_task_id: None,
        }
    }

    fn reminder() -> ReminderItem {
        ReminderItem {
            id: "r1".to_string(),
            title: "Call the dentist".to_string(),
            description: None,
            due_datetime: None,
            is_completed: true,
            priority: "high".to_string(),
        }
    }

    fn note() -> NoteItem {
        NoteItem {
            id: "n1".to_string(),
            title: "Shopping".to_string(),
            body_preview: "Milk".to_string(),
            pinned: true,
            updated_at: "2026-07-26".to_string(),
            folder_id: None,
        }
    }

    fn event() -> CalendarEventItem {
        CalendarEventItem {
            id: "e1".to_string(),
            summary: "Standup".to_string(),
            start: "2026-07-27 09:00".to_string(),
            end: "2026-07-27 09:15".to_string(),
            location: "Room 2".to_string(),
            is_all_day: false,
            status: "confirmed".to_string(),
            provider: "local".to_string(),
            calendar_id: None,
            calendar_name: Some("Work".to_string()),
            calendar_color: None,
        }
    }

    #[test]
    fn test_a_done_cell_does_not_repeat_its_own_heading() {
        // A report row is read as "heading, value", so a cell holding "Done"
        // under a heading of "Done" is announced twice on every finished row.
        assert_eq!(reminder_cell(&reminder(), 0), "Yes");
        assert_eq!(task_cell(&task(), 0), "");
    }

    #[test]
    fn test_an_empty_first_column_says_what_is_missing() {
        // An empty cell reads as nothing at all, which sounds exactly like a
        // row that failed to load.
        let mut c = contact();
        c.name = "   ".to_string();
        assert_eq!(contact_cell(&c, 0), "No name");

        let mut t = task();
        t.title = String::new();
        assert_eq!(task_cell(&t, 1), "No title");

        let mut n = note();
        n.title = String::new();
        assert!(note_cell(&n, 0).ends_with("Untitled"));

        let mut e = event();
        e.summary = String::new();
        assert_eq!(event_cell(&e, 1), "No title");
    }

    #[test]
    fn test_an_all_day_event_says_so_instead_of_a_time() {
        let mut e = event();
        e.is_all_day = true;
        assert_eq!(event_cell(&e, 0), "All day");
        assert_eq!(event_cell(&event(), 0), "2026-07-27 09:00");
    }

    #[test]
    fn test_a_pin_rides_on_the_title_rather_than_taking_a_column() {
        // A column that is empty on almost every row still costs listening
        // time on all of them.
        assert_eq!(note_cell(&note(), 0), "Pinned. Shopping");
        let mut plain = note();
        plain.pinned = false;
        assert_eq!(note_cell(&plain, 0), "Shopping");
    }

    #[test]
    fn test_a_missing_due_date_is_blank_not_the_word_none() {
        // "None" on every undated row is a syllable per row that carries
        // nothing, and silence already means the same thing.
        assert_eq!(reminder_cell(&reminder(), 2), "");
        assert_eq!(task_cell(&task(), 2), "2026-07-30");
    }

    #[test]
    fn test_every_cell_is_printable_and_no_column_panics() {
        // The paint callback has nowhere to report a failure, so nothing here
        // may panic and nothing may return a control character. Out-of-range
        // columns are asked for whenever a layout changes under the control.
        for column in -2..10 {
            for text in [
                contact_cell(&contact(), column),
                event_cell(&event(), column),
                reminder_cell(&reminder(), column),
                task_cell(&task(), column),
                note_cell(&note(), column),
            ] {
                assert!(
                    !text.chars().any(|c| c.is_control()),
                    "column {} produced a control character",
                    column
                );
            }
        }
    }
}

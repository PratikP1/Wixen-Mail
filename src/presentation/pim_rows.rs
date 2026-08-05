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
//! Cells are also what a screen reader reads for a row, and a cell has to stand
//! on its own. The headings are not reliably announced, so a cell saying "Yes"
//! is a word with nothing attached to it: it has to say "Done".

use crate::application::reading_habits::WorkingDay;

use super::date_display::{DateSettings, spoken};
use super::read_aloud::{priority_worth_saying, status_worth_saying};
use super::ui_types::{CalendarEventItem, ContactItem, NoteItem, ReminderItem, TaskItem};

/// One stored date, written the way this reader asked for it.
///
/// Every date in every one of these lists goes through here. They used to print
/// the column, so a task due "2026-07-30" was read out as a run of digits while
/// the mail list beside it said "July 30, 2026".
///
/// `now` is passed in rather than read, because the paint callback runs for
/// every visible cell and asking the clock per cell is work done thousands of
/// times for an answer that does not change within a repaint.
fn date(stored: &str, dates: DateSettings, now: chrono::DateTime<chrono::Local>) -> String {
    spoken(stored, now, dates)
}

/// The hour of the day a stored time falls in.
///
/// Read out of the text rather than parsed into a moment, because that is all
/// this needs, and because a stored value with no time in it should answer
/// nothing rather than answering midnight.
fn hour_of(stored: &str) -> Option<u8> {
    let (_, time) = stored.trim().split_once([' ', 'T'])?;
    time.split(':').next()?.parse().ok()
}

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
pub fn event_cell(
    event: &CalendarEventItem,
    column: i32,
    dates: DateSettings,
    now: chrono::DateTime<chrono::Local>,
    day: WorkingDay,
) -> String {
    match column {
        0 => {
            if event.is_all_day {
                "All day".to_string()
            } else {
                // The time, and whether it falls outside the working day.
                // Words rather than a colour, because a colour is not
                // available to the person this is for, and nothing at all
                // inside the day, so most rows in most calendars cost nothing
                // to hear.
                let when = date(&event.start, dates, now);
                match hour_of(&event.start).map(|hour| day.note_for(hour)) {
                    Some(note) if !note.is_empty() => format!("{when}, {note}"),
                    _ => when,
                }
            }
        }
        1 => non_empty(&event.summary, "No title"),
        2 => event.calendar_name.clone().unwrap_or_default(),
        3 => event.location.clone(),
        4 => status_cell(&event.status),
        _ => String::new(),
    }
}

/// Reminders: done, title, due, priority.
pub fn reminder_cell(
    reminder: &ReminderItem,
    column: i32,
    dates: DateSettings,
    now: chrono::DateTime<chrono::Local>,
) -> String {
    match column {
        0 => flag(reminder.is_completed, "Done"),
        1 => non_empty(&reminder.title, "No title"),
        2 => date(
            reminder.due_datetime.as_deref().unwrap_or_default(),
            dates,
            now,
        ),
        3 => priority_cell(&reminder.priority),
        _ => String::new(),
    }
}

/// Tasks: done, title, due, priority.
pub fn task_cell(
    task: &TaskItem,
    column: i32,
    dates: DateSettings,
    now: chrono::DateTime<chrono::Local>,
) -> String {
    match column {
        0 => flag(task.is_completed, "Done"),
        1 => non_empty(&task.title, "No title"),
        2 => date(task.due_date.as_deref().unwrap_or_default(), dates, now),
        3 => priority_cell(&task.priority),
        _ => String::new(),
    }
}

/// Notes: title, last modified.
pub fn note_cell(
    note: &NoteItem,
    column: i32,
    dates: DateSettings,
    now: chrono::DateTime<chrono::Local>,
) -> String {
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
        1 => date(&note.updated_at, dates, now),
        _ => String::new(),
    }
}

/// What the flag means, or nothing.
///
/// The word itself rather than "Yes", because the column heading is not
/// reliably read and "Yes" on its own carries nothing. Silence for the
/// negative case, which costs no listening time.
fn flag(value: bool, meaning: &str) -> String {
    if value { meaning } else { "" }.to_string()
}

/// How an event's status reads, or nothing when it is going ahead.
///
/// The same rule the flag columns follow. Almost every event in almost every
/// calendar is confirmed, so a column that said so would spend a word on every
/// row that never varies, and would bury the cancelled one in a run of
/// identical ones. Which values are worth hearing is decided in one place and
/// shared with the reading of an event, so the list and the reading agree.
fn status_cell(status: &str) -> String {
    as_a_word(status_worth_saying(status))
}

/// How a priority reads, or nothing when it is the ordinary one.
///
/// "Normal" is what this program writes when a provider has no notion of
/// priority at all, so it is on nearly every task and reminder. The cell is
/// read without its heading, so what is left says what it is a level of.
fn priority_cell(priority: &str) -> String {
    let level = as_a_word(priority_worth_saying(priority));
    if level.is_empty() {
        level
    } else {
        format!("{level} priority")
    }
}

/// A stored token said as a word.
///
/// The values arrive lowercased from every provider, and a cell is read on its
/// own rather than after a heading, so it starts the way a sentence does.
fn as_a_word(token: &str) -> String {
    match token.trim().chars().next() {
        Some(first) => first.to_uppercase().to_string() + &token.trim()[first.len_utf8()..],
        None => String::new(),
    }
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

    /// Fixed rather than read from the machine, so these read the same
    /// wherever they run.
    fn at_a_desk() -> DateSettings {
        DateSettings {
            style: crate::presentation::date_display::DateStyle::Absolute,
            order: crate::presentation::date_display::DateOrder::MonthFirst,
            wording: crate::presentation::date_display::DateWording::Verbal,
            clock: crate::presentation::date_display::Clock::TwelveHour,
        }
    }

    fn midday() -> chrono::DateTime<chrono::Local> {
        use chrono::TimeZone;
        chrono::Local
            .with_ymd_and_hms(2026, 7, 26, 12, 0, 0)
            .single()
            .expect("a real moment")
    }

    #[test]
    fn test_an_event_outside_the_working_day_says_so() {
        // A meeting at seven in the evening is a fact somebody wants to
        // notice, and every hour used to read the same way.
        let mut evening = event();
        evening.start = "2026-07-27 19:00".to_string();

        let cell = event_cell(&evening, 0, at_a_desk(), midday(), WorkingDay::default());

        assert!(cell.contains("after the working day"), "{cell}");
    }

    #[test]
    fn test_an_event_inside_the_working_day_costs_nothing_to_hear() {
        // Most rows in most calendars, so a word on every one of them is a
        // word paid for on all of them.
        let mut morning = event();
        morning.start = "2026-07-27 10:00".to_string();

        let cell = event_cell(&morning, 0, at_a_desk(), midday(), WorkingDay::default());

        assert!(!cell.contains("working day"), "{cell}");
    }

    #[test]
    fn test_an_all_day_event_is_not_told_it_is_out_of_hours() {
        // It has no hour to be outside of.
        let mut all_day = event();
        all_day.is_all_day = true;

        assert_eq!(
            event_cell(&all_day, 0, at_a_desk(), midday(), WorkingDay::default()),
            "All day"
        );
    }

    #[test]
    fn test_an_event_with_no_time_in_it_says_nothing_about_hours() {
        // Rather than being treated as midnight, which every working day is
        // outside of, so every one of them would carry the note.
        let mut dateless = event();
        dateless.start = "2026-07-27".to_string();

        let cell = event_cell(&dateless, 0, at_a_desk(), midday(), WorkingDay::default());

        assert!(!cell.contains("working day"), "{cell}");
    }

    fn contact() -> ContactItem {
        ContactItem {
            id: "c1".to_string(),
            name: "Grace Hopper".to_string(),
            email: "grace@example.com".to_string(),
            phone: "555 0100".to_string(),
            company: "Navy".to_string(),
            birthday: String::new(),
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
            body: "Milk".to_string(),
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
            description: String::new(),
            start: "2026-07-27 09:00".to_string(),
            end: "2026-07-27 09:15".to_string(),
            location: "Room 2".to_string(),
            is_all_day: false,
            status: "confirmed".to_string(),
            provider: "local".to_string(),
            calendar_id: None,
            calendar_name: Some("Work".to_string()),
            calendar_color: None,
            reminder_minutes: None,
            repeats: String::new(),
            categories: String::new(),
        }
    }

    #[test]
    fn test_a_done_cell_says_done_rather_than_yes() {
        // The heading is not reliably announced, so "Yes" is a word with
        // nothing attached to it.
        assert_eq!(reminder_cell(&reminder(), 0, at_a_desk(), midday()), "Done");
        assert_eq!(task_cell(&task(), 0, at_a_desk(), midday()), "");
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
        assert_eq!(task_cell(&t, 1, at_a_desk(), midday()), "No title");

        let mut n = note();
        n.title = String::new();
        assert!(note_cell(&n, 0, at_a_desk(), midday()).ends_with("Untitled"));

        let mut e = event();
        e.summary = String::new();
        assert_eq!(
            event_cell(&e, 1, at_a_desk(), midday(), WorkingDay::default()),
            "No title"
        );
    }

    #[test]
    fn test_an_all_day_event_says_so_instead_of_a_time() {
        let mut e = event();
        e.is_all_day = true;
        assert_eq!(
            event_cell(&e, 0, at_a_desk(), midday(), WorkingDay::default()),
            "All day"
        );
        // Not "2026-07-27 09:00", which a screen reader reads as a run of
        // digits. Every list says a date the way the mail list always did.
        assert_eq!(
            event_cell(&event(), 0, at_a_desk(), midday(), WorkingDay::default()),
            "July 27, 2026 at 9:00 AM"
        );
    }

    #[test]
    fn test_a_pin_rides_on_the_title_rather_than_taking_a_column() {
        // A column that is empty on almost every row still costs listening
        // time on all of them.
        assert_eq!(
            note_cell(&note(), 0, at_a_desk(), midday()),
            "Pinned. Shopping"
        );
        let mut plain = note();
        plain.pinned = false;
        assert_eq!(note_cell(&plain, 0, at_a_desk(), midday()), "Shopping");
    }

    #[test]
    fn test_a_missing_due_date_is_blank_not_the_word_none() {
        // "None" on every undated row is a syllable per row that carries
        // nothing, and silence already means the same thing.
        assert_eq!(reminder_cell(&reminder(), 2, at_a_desk(), midday()), "");
        // Due on a day, so no midnight is invented for it.
        assert_eq!(
            task_cell(&task(), 2, at_a_desk(), midday()),
            "July 30, 2026"
        );
    }

    #[test]
    fn test_a_contact_row_reads_the_email_phone_and_company_it_holds() {
        // Three columns whose headings promise three different fields. A row
        // that went blank in all three would read as a name and three
        // silences, which is what a row that failed to load sounds like.
        assert_eq!(contact_cell(&contact(), 1), "grace@example.com");
        assert_eq!(contact_cell(&contact(), 2), "555 0100");
        assert_eq!(contact_cell(&contact(), 3), "Navy");
    }

    #[test]
    fn test_an_event_row_reads_the_calendar_and_the_location_it_holds() {
        assert_eq!(
            event_cell(&event(), 2, at_a_desk(), midday(), WorkingDay::default()),
            "Work"
        );
        assert_eq!(
            event_cell(&event(), 3, at_a_desk(), midday(), WorkingDay::default()),
            "Room 2"
        );
    }

    #[test]
    fn test_an_event_in_no_named_calendar_says_nothing_rather_than_none() {
        let mut loose = event();
        loose.calendar_name = None;

        assert_eq!(
            event_cell(&loose, 2, at_a_desk(), midday(), WorkingDay::default()),
            ""
        );
    }

    #[test]
    fn test_a_reminder_row_reads_the_title_it_holds() {
        assert_eq!(
            reminder_cell(&reminder(), 1, at_a_desk(), midday()),
            "Call the dentist"
        );
    }

    #[test]
    fn test_a_reminder_with_a_due_date_says_the_date_rather_than_nothing() {
        // The sibling of the test above it. Silence is right for a reminder
        // with no due date and wrong for one that has it, and only having both
        // tells the two apart.
        let mut due = reminder();
        due.due_datetime = Some("2026-07-30 14:30".to_string());

        assert_eq!(
            reminder_cell(&due, 2, at_a_desk(), midday()),
            "July 30, 2026 at 2:30 PM"
        );
    }

    #[test]
    fn test_a_completed_task_says_done_rather_than_falling_silent() {
        // The sibling of the silence a task that is not finished keeps.
        let mut finished = task();
        finished.is_completed = true;

        assert_eq!(task_cell(&finished, 0, at_a_desk(), midday()), "Done");
    }

    #[test]
    fn test_a_note_row_says_when_it_was_last_changed() {
        // As a date rather than as the stored string, which a screen reader
        // reads as a run of digits.
        assert_eq!(
            note_cell(&note(), 1, at_a_desk(), midday()),
            "July 26, 2026"
        );
    }

    #[test]
    fn test_an_event_that_is_going_ahead_costs_nothing_to_hear() {
        // Almost every event in almost every calendar is confirmed, and this
        // column used to speak that word on every one of them. It is the same
        // rule the Done column and the reading of an event already follow.
        assert_eq!(
            event_cell(&event(), 4, at_a_desk(), midday(), WorkingDay::default()),
            ""
        );
    }

    #[test]
    fn test_a_cancelled_event_says_so_rather_than_falling_silent() {
        // The one value in this column somebody has to notice, and it used to
        // sit in a run of identical words.
        let mut called_off = event();
        called_off.status = "cancelled".to_string();

        assert_eq!(
            event_cell(&called_off, 4, at_a_desk(), midday(), WorkingDay::default()),
            "Cancelled"
        );
    }

    #[test]
    fn test_an_ordinary_priority_costs_nothing_to_hear() {
        // "normal" is what this program writes when a provider has no notion
        // of priority at all, so it is on nearly every task and reminder.
        let mut plain = reminder();
        plain.priority = "normal".to_string();

        assert_eq!(reminder_cell(&plain, 3, at_a_desk(), midday()), "");
        assert_eq!(task_cell(&task(), 3, at_a_desk(), midday()), "");
    }

    #[test]
    fn test_a_reminder_of_an_unusual_priority_says_what_the_level_is_a_level_of() {
        // The cell is read without its heading, so "high" on its own is a word
        // with nothing attached to it.
        assert_eq!(
            reminder_cell(&reminder(), 3, at_a_desk(), midday()),
            "High priority"
        );
    }

    #[test]
    fn test_a_task_of_an_unusual_priority_says_what_the_level_is_a_level_of() {
        let mut urgent = task();
        urgent.priority = "high".to_string();

        assert_eq!(
            task_cell(&urgent, 3, at_a_desk(), midday()),
            "High priority"
        );
    }

    #[test]
    fn test_every_cell_is_printable_and_no_column_panics() {
        // The paint callback has nowhere to report a failure, so nothing here
        // may panic and nothing may return a control character. Out-of-range
        // columns are asked for whenever a layout changes under the control.
        for column in -2..10 {
            for text in [
                contact_cell(&contact(), column),
                event_cell(
                    &event(),
                    column,
                    at_a_desk(),
                    midday(),
                    WorkingDay::default(),
                ),
                reminder_cell(&reminder(), column, at_a_desk(), midday()),
                task_cell(&task(), column, at_a_desk(), midday()),
                note_cell(&note(), column, at_a_desk(), midday()),
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

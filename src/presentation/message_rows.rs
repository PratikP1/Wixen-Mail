//! Turning a message into the cells a virtual list asks for.
//!
//! The list runs in virtual mode, so wxWidgets calls back for cell text while
//! it is painting. That callback has to be quick and it must never fail: it
//! cannot query the database, cannot block, and has nowhere to report an error
//! to. Everything here is therefore a pure function over data already in memory.
//!
//! Cells are also what a screen reader reads for a row, so the text is written
//! to be heard rather than seen. "Unread" beats a bullet, and a date reads as a
//! date rather than a timestamp.

use super::date_display::{format_for_list, DateOrder, DateStyle};
use super::message_columns::MessageColumn;
use super::ui_types::MessageItem;

/// How dates are being shown, passed in so the cell function stays pure and
/// the preference is read once rather than per row.
#[derive(Debug, Clone, Copy)]
pub struct DateSettings {
    pub style: DateStyle,
    pub order: DateOrder,
}

impl Default for DateSettings {
    fn default() -> Self {
        Self {
            style: DateStyle::RelativeWithinWeek,
            order: DateOrder::from_system(),
        }
    }
}

/// Shown in a cell whose page has not been loaded yet.
///
/// Virtual mode asks for rows the instant they scroll into view, and the answer
/// has to be immediate. A row that is still being fetched says so rather than
/// appearing blank, which would read as an empty message.
pub const PLACEHOLDER: &str = "Loading";

/// The text for one cell.
pub fn cell_text(
    message: &MessageItem,
    column: MessageColumn,
    dates: DateSettings,
    now: chrono::DateTime<chrono::Local>,
) -> String {
    match column {
        // "Yes" rather than the column's own word. A screen reader reads a
        // report row as "heading, value", so a cell repeating its heading came
        // out as "Attachment attachment" on every row that had one. Empty for
        // the negative case, which costs no listening time at all.
        MessageColumn::Unread => if message.read { "" } else { "Yes" }.to_string(),
        MessageColumn::Attachment => if message.has_attachments { "Yes" } else { "" }.to_string(),
        MessageColumn::Subject => {
            if message.subject.trim().is_empty() {
                "No subject".to_string()
            } else {
                message.subject.clone()
            }
        }
        MessageColumn::Correspondent => display_address(&message.from),
        MessageColumn::Received | MessageColumn::Sent => {
            format_for_list(&message.date, now, dates.style, dates.order)
        }
        MessageColumn::Snippet => message.snippet.clone(),
        MessageColumn::Thread => thread_cell(message),
        MessageColumn::Size => message.size_bytes.map(size_cell).unwrap_or_default(),
        MessageColumn::Flagged => if message.starred { "Yes" } else { "" }.to_string(),
        MessageColumn::To => display_address(&message.to),
        MessageColumn::Cc => display_address(&message.cc),
    }
}

/// A size in units rather than raw bytes.
///
/// "2 KB" is one word to hear. "2048" is four digits a listener has to
/// assemble into a number and then into a size, on every row.
fn size_cell(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    match bytes {
        b if b < 0 => String::new(),
        1 => "1 byte".to_string(),
        b if b < KB => format!("{} bytes", b),
        b if b < MB => format!("{} KB", b / KB),
        b => {
            let megabytes = b as f64 / MB as f64;
            // One decimal place only when it says something: "1.5 MB" is
            // useful, "1.0 MB" is a syllable that carries nothing.
            if (megabytes - megabytes.round()).abs() < 0.05 {
                format!("{} MB", megabytes.round() as i64)
            } else {
                format!("{:.1} MB", megabytes)
            }
        }
    }
}

/// How a thread is described in its column.
///
/// Position rather than size, because in the message list you are standing on
/// one message and what matters is where you are in the conversation.
fn thread_cell(message: &MessageItem) -> String {
    match &message.thread_id {
        Some(_) if message.thread_depth > 0 => {
            format!("Reply {}", message.thread_depth)
        }
        Some(_) => "Thread start".to_string(),
        None => String::new(),
    }
}

/// Prefer a display name over a raw address.
///
/// "Ada Lovelace" is quicker to hear than "ada dot lovelace at example dot com",
/// and the address is still available in the details read with Shift+Space.
fn display_address(address: &str) -> String {
    let trimmed = address.trim();
    if let Some(open) = trimmed.find('<') {
        let name = trimmed[..open].trim().trim_matches('"').trim();
        if !name.is_empty() {
            return name.to_string();
        }
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_snippet_column_reads_the_stored_snippet() {
        assert_eq!(
            cell_text(
                &message(),
                MessageColumn::Snippet,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "The numbers are attached."
        );
    }

    #[test]
    fn test_size_is_spoken_in_units_not_raw_bytes() {
        // "2 KB" is one word to hear; "2048" is four digits to assemble.
        let mut m = message();
        for (bytes, expected) in [
            (0i64, "0 bytes"),
            (1, "1 byte"),
            (999, "999 bytes"),
            (2048, "2 KB"),
            (1_572_864, "1.5 MB"),
        ] {
            m.size_bytes = Some(bytes);
            assert_eq!(
                cell_text(
                    &m,
                    MessageColumn::Size,
                    DateSettings::default(),
                    chrono::Local::now()
                ),
                expected,
                "for {} bytes",
                bytes
            );
        }
    }

    #[test]
    fn test_an_unknown_size_says_nothing_rather_than_zero() {
        // We do not know the size until the envelope has been fetched, and
        // "0 bytes" is a claim, not an absence.
        let mut m = message();
        m.size_bytes = None;
        assert_eq!(
            cell_text(
                &m,
                MessageColumn::Size,
                DateSettings::default(),
                chrono::Local::now()
            ),
            ""
        );
    }

    #[test]
    fn test_to_and_cc_prefer_display_names_like_correspondent_does() {
        let mut m = message();
        m.to = "Grace Hopper <grace@example.com>".to_string();
        m.cc = "Alan Turing <alan@example.com>".to_string();
        assert_eq!(
            cell_text(
                &m,
                MessageColumn::To,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "Grace Hopper"
        );
        assert_eq!(
            cell_text(
                &m,
                MessageColumn::Cc,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "Alan Turing"
        );
    }

    fn message() -> MessageItem {
        MessageItem {
            uid: 1,
            message_id: 1,
            subject: "Quarterly report".to_string(),
            from: "Ada Lovelace <ada@example.com>".to_string(),
            date: "2026-07-26".to_string(),
            read: false,
            starred: false,
            has_attachments: false,
            attachments: Vec::new(),
            thread_depth: 0,
            is_thread_parent: false,
            thread_id: None,
            snippet: "The numbers are attached.".to_string(),
            size_bytes: Some(2048),
            to: "me@example.com".to_string(),
            cc: String::new(),
        }
    }

    #[test]
    fn test_a_flag_cell_does_not_repeat_its_own_heading() {
        // A report row is read as "heading, value", so a cell holding the word
        // "Attachment" under a heading of "Attachment" was announced twice on
        // every row that had one.
        let mut m = message();
        m.has_attachments = true;
        m.starred = true;
        assert_eq!(
            cell_text(
                &m,
                MessageColumn::Unread,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "Yes"
        );
        assert_eq!(
            cell_text(
                &m,
                MessageColumn::Attachment,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "Yes"
        );
        assert_eq!(
            cell_text(
                &m,
                MessageColumn::Flagged,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "Yes"
        );
    }

    #[test]
    fn test_a_read_message_says_nothing_in_that_column() {
        // Silence is the point: the column only costs listening time when it
        // has something to say.
        let mut m = message();
        m.read = true;
        assert_eq!(
            cell_text(
                &m,
                MessageColumn::Unread,
                DateSettings::default(),
                chrono::Local::now()
            ),
            ""
        );
    }

    #[test]
    fn test_a_missing_subject_says_so() {
        // An empty cell in the subject column reads as nothing at all, which is
        // indistinguishable from a row that failed to load.
        let mut m = message();
        m.subject = "   ".to_string();
        assert_eq!(
            cell_text(
                &m,
                MessageColumn::Subject,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "No subject"
        );
    }

    #[test]
    fn test_correspondent_prefers_the_display_name() {
        assert_eq!(
            cell_text(
                &message(),
                MessageColumn::Correspondent,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "Ada Lovelace"
        );
    }

    #[test]
    fn test_correspondent_falls_back_to_the_address() {
        let mut m = message();
        m.from = "ada@example.com".to_string();
        assert_eq!(
            cell_text(
                &m,
                MessageColumn::Correspondent,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "ada@example.com"
        );
    }

    #[test]
    fn test_correspondent_strips_quotes_around_a_name() {
        let mut m = message();
        m.from = "\"Lovelace, Ada\" <ada@example.com>".to_string();
        assert_eq!(
            cell_text(
                &m,
                MessageColumn::Correspondent,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "Lovelace, Ada"
        );
    }

    #[test]
    fn test_an_empty_display_name_falls_back_rather_than_reading_blank() {
        let mut m = message();
        m.from = "\"\" <ada@example.com>".to_string();
        assert_eq!(
            cell_text(
                &m,
                MessageColumn::Correspondent,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "\"\" <ada@example.com>"
        );
    }

    #[test]
    fn test_thread_column_describes_position_not_size() {
        let mut m = message();
        m.thread_id = Some("t1".to_string());
        assert_eq!(
            cell_text(
                &m,
                MessageColumn::Thread,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "Thread start"
        );
        m.thread_depth = 3;
        assert_eq!(
            cell_text(
                &m,
                MessageColumn::Thread,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "Reply 3"
        );
    }

    #[test]
    fn test_a_message_in_no_thread_says_nothing() {
        assert_eq!(
            cell_text(
                &message(),
                MessageColumn::Thread,
                DateSettings::default(),
                chrono::Local::now()
            ),
            ""
        );
    }

    #[test]
    fn test_every_column_returns_something_printable() {
        // The paint callback has nowhere to report a failure, so no column may
        // panic and none may return a control character.
        for column in MessageColumn::ALL {
            let text = cell_text(
                &message(),
                column,
                DateSettings::default(),
                chrono::Local::now(),
            );
            assert!(
                !text.chars().any(|c| c.is_control()),
                "{:?} produced a control character",
                column
            );
        }
    }
}

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

use super::message_columns::MessageColumn;
use super::ui_types::MessageItem;

/// Shown in a cell whose page has not been loaded yet.
///
/// Virtual mode asks for rows the instant they scroll into view, and the answer
/// has to be immediate. A row that is still being fetched says so rather than
/// appearing blank, which would read as an empty message.
pub const PLACEHOLDER: &str = "Loading";

/// The text for one cell.
pub fn cell_text(message: &MessageItem, column: MessageColumn) -> String {
    match column {
        // Spoken words rather than symbols. A screen reader announces "unread"
        // usefully and a bullet character either not at all or as "bullet".
        MessageColumn::Unread => if message.read { "" } else { "Unread" }.to_string(),
        MessageColumn::Attachment => if message.has_attachments {
            "Attachment"
        } else {
            ""
        }
        .to_string(),
        MessageColumn::Subject => {
            if message.subject.trim().is_empty() {
                "No subject".to_string()
            } else {
                message.subject.clone()
            }
        }
        MessageColumn::Correspondent => display_address(&message.from),
        MessageColumn::Received | MessageColumn::Sent => message.date.clone(),
        MessageColumn::Snippet => String::new(),
        MessageColumn::Thread => thread_cell(message),
        MessageColumn::Size => String::new(),
        MessageColumn::Flagged => if message.starred { "Flagged" } else { "" }.to_string(),
        MessageColumn::Answered | MessageColumn::Draft => String::new(),
        MessageColumn::To | MessageColumn::Cc | MessageColumn::Tags => String::new(),
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
        }
    }

    #[test]
    fn test_unread_reads_as_a_word() {
        // A bullet or an asterisk is either skipped by a screen reader or read
        // as "bullet", neither of which tells anyone the message is unread.
        assert_eq!(cell_text(&message(), MessageColumn::Unread), "Unread");
    }

    #[test]
    fn test_a_read_message_says_nothing_in_that_column() {
        // Silence is the point: the column only costs listening time when it
        // has something to say.
        let mut m = message();
        m.read = true;
        assert_eq!(cell_text(&m, MessageColumn::Unread), "");
    }

    #[test]
    fn test_attachment_and_flag_columns_are_words_too() {
        let mut m = message();
        m.has_attachments = true;
        m.starred = true;
        assert_eq!(cell_text(&m, MessageColumn::Attachment), "Attachment");
        assert_eq!(cell_text(&m, MessageColumn::Flagged), "Flagged");
    }

    #[test]
    fn test_a_missing_subject_says_so() {
        // An empty cell in the subject column reads as nothing at all, which is
        // indistinguishable from a row that failed to load.
        let mut m = message();
        m.subject = "   ".to_string();
        assert_eq!(cell_text(&m, MessageColumn::Subject), "No subject");
    }

    #[test]
    fn test_correspondent_prefers_the_display_name() {
        assert_eq!(
            cell_text(&message(), MessageColumn::Correspondent),
            "Ada Lovelace"
        );
    }

    #[test]
    fn test_correspondent_falls_back_to_the_address() {
        let mut m = message();
        m.from = "ada@example.com".to_string();
        assert_eq!(
            cell_text(&m, MessageColumn::Correspondent),
            "ada@example.com"
        );
    }

    #[test]
    fn test_correspondent_strips_quotes_around_a_name() {
        let mut m = message();
        m.from = "\"Lovelace, Ada\" <ada@example.com>".to_string();
        assert_eq!(cell_text(&m, MessageColumn::Correspondent), "Lovelace, Ada");
    }

    #[test]
    fn test_an_empty_display_name_falls_back_rather_than_reading_blank() {
        let mut m = message();
        m.from = "\"\" <ada@example.com>".to_string();
        assert_eq!(
            cell_text(&m, MessageColumn::Correspondent),
            "\"\" <ada@example.com>"
        );
    }

    #[test]
    fn test_thread_column_describes_position_not_size() {
        let mut m = message();
        m.thread_id = Some("t1".to_string());
        assert_eq!(cell_text(&m, MessageColumn::Thread), "Thread start");
        m.thread_depth = 3;
        assert_eq!(cell_text(&m, MessageColumn::Thread), "Reply 3");
    }

    #[test]
    fn test_a_message_in_no_thread_says_nothing() {
        assert_eq!(cell_text(&message(), MessageColumn::Thread), "");
    }

    #[test]
    fn test_every_column_returns_something_printable() {
        // The paint callback has nowhere to report a failure, so no column may
        // panic and none may return a control character.
        for column in MessageColumn::ALL {
            let text = cell_text(&message(), column);
            assert!(
                !text.chars().any(|c| c.is_control()),
                "{:?} produced a control character",
                column
            );
        }
    }
}

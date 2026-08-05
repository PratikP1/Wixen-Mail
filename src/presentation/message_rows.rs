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

use super::date_display::{DateSettings, format_for_list};
use super::message_columns::MessageColumn;
use super::ui_types::MessageItem;

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
        // The cell says what it means rather than "Yes".
        //
        // This was "Yes" for a while, on the reasoning that a screen reader
        // reads a report row as "heading, value" and a cell repeating its
        // heading would be said twice. In practice the headings are not being
        // read here, so "Yes" was a word with nothing attached to it and the
        // unread state was never spoken at all. A cell that stands on its own
        // is worth more than one that depends on a heading being announced.
        //
        // Still empty for the negative case, which costs no listening time.
        MessageColumn::Unread => if message.read { "" } else { "Unread" }.to_string(),
        MessageColumn::Attachment => if message.has_attachments {
            "Has attachment"
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
        MessageColumn::Received | MessageColumn::Sent => format_for_list(&message.date, now, dates),
        MessageColumn::Snippet => message.snippet.clone(),
        MessageColumn::Thread => thread_cell(message),
        MessageColumn::Size => message.size_bytes.map(size_cell).unwrap_or_default(),
        MessageColumn::Flagged => if message.starred { "Flagged" } else { "" }.to_string(),
        // Self-describing like the other flag cells: the column heading is not
        // announced with the cell, so "Yes" in a row would be a word with
        // nothing attached to it.
        MessageColumn::Answered => if message.answered { "Answered" } else { "" }.to_string(),
        MessageColumn::Draft => if message.draft { "Draft" } else { "" }.to_string(),
        // "Spam", "Phishing", "Suspicious", or nothing at all. Like the other
        // flag cells it costs listening time only when it has something to say,
        // which for most of a mailbox is never.
        MessageColumn::Safety => message.safety.label().to_string(),
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

/// How big the conversation this row belongs to is, when it is one.
///
/// `None` for a message that stands alone, and then nothing is said. Saying
/// "1 message" on every ordinary message is a word on every row that carries no
/// information, and the rows that are conversations are the ones where the
/// count changes what somebody does next.
///
/// Counted across the loaded list rather than asked of the server, because the
/// list is what Space is reading and the answer has to arrive with the key
/// rather than after it.
pub fn conversation_size(rows: &[MessageItem], index: usize) -> Option<usize> {
    let thread = rows.get(index)?.thread_id.as_deref()?;
    if thread.trim().is_empty() {
        return None;
    }
    let size = rows
        .iter()
        .filter(|row| row.thread_id.as_deref() == Some(thread))
        .count();
    (size > 1).then_some(size)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn in_thread(id: Option<&str>) -> MessageItem {
        let mut row = message();
        row.thread_id = id.map(str::to_string);
        row
    }

    #[test]
    fn test_a_conversation_says_how_many_messages_it_holds() {
        let rows = vec![
            in_thread(Some("t1")),
            in_thread(Some("t1")),
            in_thread(Some("t1")),
        ];

        assert_eq!(conversation_size(&rows, 0), Some(3));
    }

    #[test]
    fn test_a_message_that_stands_alone_says_nothing_about_conversations() {
        // "1 message" on every ordinary row is a word that carries nothing,
        // said on the key somebody presses most.
        let rows = vec![in_thread(Some("t1")), in_thread(Some("t2"))];

        assert_eq!(conversation_size(&rows, 0), None);
        assert_eq!(conversation_size(&[in_thread(None)], 0), None);
    }

    #[test]
    fn test_only_the_messages_in_this_conversation_are_counted() {
        let rows = vec![
            in_thread(Some("t1")),
            in_thread(Some("t2")),
            in_thread(Some("t1")),
            in_thread(None),
        ];

        assert_eq!(conversation_size(&rows, 1), None);
        assert_eq!(conversation_size(&rows, 2), Some(2));
    }

    #[test]
    fn test_a_row_that_is_not_there_is_not_a_conversation() {
        assert_eq!(conversation_size(&[], 0), None);
        assert_eq!(conversation_size(&[in_thread(Some("t1"))], 9), None);
    }

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
    fn test_a_size_of_exactly_one_kilobyte_reads_as_one_kilobyte() {
        // The boundary itself, which is the value the rule is written around.
        // A kilobyte read out as four digits is the thing this column exists
        // to stop.
        let mut m = message();
        m.size_bytes = Some(1024);

        assert_eq!(
            cell_text(
                &m,
                MessageColumn::Size,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "1 KB"
        );
    }

    #[test]
    fn test_a_whole_number_of_megabytes_drops_the_decimal_point() {
        // "1.0 MB" is a syllable that carries nothing, said on a column read
        // for every message.
        let mut m = message();
        m.size_bytes = Some(1_048_576);

        assert_eq!(
            cell_text(
                &m,
                MessageColumn::Size,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "1 MB"
        );
    }

    #[test]
    fn test_a_size_that_cannot_be_right_says_nothing_rather_than_a_negative_number() {
        // Read back off disk, so nothing in this layer can promise it is
        // sensible. Nothing written here can produce it today; the guard is
        // what stops "minus one bytes" if anything ever does.
        let mut m = message();
        m.size_bytes = Some(-1);

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
            answered: false,
            draft: false,
            has_attachments: false,
            attachments: Vec::new(),
            thread_depth: 0,
            is_thread_parent: false,
            thread_id: None,
            snippet: "The numbers are attached.".to_string(),
            size_bytes: Some(2048),
            to: "me@example.com".to_string(),
            cc: String::new(),
            reply_to: String::new(),
            header_message_id: String::new(),
            refs_header: None,
            safety: crate::service::safety::Safety::Ordinary,
            safety_reasons: Vec::new(),
            receipt_to: None,
            account_id: String::new(),
            labels: Vec::new(),
        }
    }

    #[test]
    fn test_the_safety_cell_is_empty_unless_there_is_something_to_say() {
        // Most of a mailbox is ordinary mail. A column that read "Safe" on
        // every row would be a word read past a thousand times a day.
        let ordinary = message();

        assert_eq!(
            cell_text(
                &ordinary,
                MessageColumn::Safety,
                DateSettings::default(),
                chrono::Local::now()
            ),
            ""
        );
    }

    #[test]
    fn test_the_safety_cell_names_the_verdict() {
        use crate::service::safety::Safety;
        for (level, expected) in [
            (Safety::Suspicious, "Suspicious"),
            (Safety::Spam, "Spam"),
            (Safety::Phishing, "Phishing"),
        ] {
            let mut flagged = message();
            flagged.safety = level;

            assert_eq!(
                cell_text(
                    &flagged,
                    MessageColumn::Safety,
                    DateSettings::default(),
                    chrono::Local::now()
                ),
                expected
            );
        }
    }

    #[test]
    fn test_a_flag_cell_stands_on_its_own_without_its_heading() {
        // "Yes" is a word with nothing attached to it when the heading is not
        // being read, and the unread state was never spoken at all.
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
            "Unread"
        );
        assert_eq!(
            cell_text(
                &m,
                MessageColumn::Attachment,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "Has attachment"
        );
        assert_eq!(
            cell_text(
                &m,
                MessageColumn::Flagged,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "Flagged"
        );
    }

    #[test]
    fn test_the_answered_and_draft_cells_stand_on_their_own_too() {
        // Both columns were withdrawn because nothing could fill them. The
        // server's flags fill them now, and they follow the same rule as the
        // others: the cell says what it means, or says nothing.
        let mut replied = message();
        replied.answered = true;
        assert_eq!(
            cell_text(
                &replied,
                MessageColumn::Answered,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "Answered"
        );
        assert_eq!(
            cell_text(
                &message(),
                MessageColumn::Answered,
                DateSettings::default(),
                chrono::Local::now()
            ),
            ""
        );

        let mut unsent = message();
        unsent.draft = true;
        assert_eq!(
            cell_text(
                &unsent,
                MessageColumn::Draft,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "Draft"
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

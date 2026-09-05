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
use crate::application::conversations::ConversationItem;

/// Shown in a cell whose snippet is absent because nobody has fetched it.
///
/// A blank cell here reads as a message with nothing in it, which is a claim
/// about somebody's mail rather than about this program, and it was wrong for
/// most of a large mailbox: a folder that has not been through the message text
/// fetch has no snippet for any of its rows.
///
/// Words rather than a placeholder character, because this is read aloud on
/// every row somebody arrows onto and a dash says nothing at all.
pub const TEXT_NOT_DOWNLOADED: &str = "Message text not downloaded";

/// Shown in a cell for a message whose text really is empty.
///
/// A calendar invitation, or a message that is nothing but an attachment. Kept
/// apart from [`TEXT_NOT_DOWNLOADED`] because they ask different things of the
/// person hearing them: one is worth fetching and the other is not there to
/// fetch.
pub const NO_MESSAGE_TEXT: &str = "No message text";

/// Shown in a cell whose page has not been loaded yet.
///
/// Virtual mode asks for rows the instant they scroll into view, and the answer
/// has to be immediate. A row that is still being fetched says so rather than
/// appearing blank, which would read as an empty message.
pub const PLACEHOLDER: &str = "Loading";

/// What the snippet column says, given what is stored.
///
/// One function for both row types, because the two views ask the same question
/// about the same three states and answering it twice is how they come to
/// disagree. `None` is text nobody has fetched, an empty string is a message
/// whose text was fetched and holds nothing, and anything else is the line
/// itself.
fn the_first_line(stored: Option<&str>) -> String {
    match stored {
        None => TEXT_NOT_DOWNLOADED.to_string(),
        Some(line) if line.trim().is_empty() => NO_MESSAGE_TEXT.to_string(),
        Some(line) => line.to_string(),
    }
}

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
        MessageColumn::Snippet => the_first_line(message.snippet.as_deref()),
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

/// The text for one cell of a conversation row.
///
/// The sibling of [`cell_text`], one arm per column, over a whole conversation
/// instead of one message. D-02: every arm answers about the conversation, and
/// the value each one reads was filled by the very SQL expression the list is
/// ordered by, so what a row says and what it sorts by cannot come apart.
///
/// The module's two rules hold here as they do above. A cell is self-describing,
/// because the headings are not being read and "Yes" would be a word with
/// nothing attached to it. And the negative case is the empty string, which
/// costs no listening time.
///
/// Pure over data already in memory, like everything else in this module: the
/// virtual list calls back for cell text while it is painting, so this cannot
/// query the database, cannot block, and has nowhere to report an error to.
pub fn conversation_cell_text(
    conversation: &ConversationItem,
    column: MessageColumn,
    dates: DateSettings,
    now: chrono::DateTime<chrono::Local>,
) -> String {
    match column {
        // Unread if any message in it is, which is what the count being more
        // than none means.
        MessageColumn::Unread => if conversation.unread > 0 {
            "Unread"
        } else {
            ""
        }
        .to_string(),
        MessageColumn::Attachment => if conversation.any_attachment {
            "Has attachment"
        } else {
            ""
        }
        .to_string(),
        // Already the conversation's name: the rule that strips the markers is
        // the one the query ordered by, so it has run before this. The fallback
        // is for a conversation whose oldest message had no subject stored at
        // all, which is a row somebody would otherwise arrow onto and hear
        // silence from.
        MessageColumn::Subject => {
            if conversation.subject.trim().is_empty() {
                crate::application::conversations::NO_SUBJECT.to_string()
            } else {
                conversation.subject.clone()
            }
        }
        MessageColumn::Correspondent => everyone_in(&conversation.senders),
        MessageColumn::Received => format_for_list(&conversation.newest_received, now, dates),
        MessageColumn::Sent => format_for_list(&conversation.newest_sent, now, dates),
        MessageColumn::Snippet => the_first_line(conversation.snippet.as_deref()),
        // D-03. A conversation identifier is a mail server's angle-bracketed
        // nonsense and no use at all to anybody hearing it, and there is no
        // per-item accessible name on this control, so the cell text is exactly
        // what a screen reader reads for the row.
        MessageColumn::Thread => crate::application::conversations::counts_read_as(
            conversation.messages,
            conversation.unread,
        ),
        MessageColumn::Size => conversation.size_bytes.map(size_cell).unwrap_or_default(),
        MessageColumn::Flagged => if conversation.any_flagged {
            "Flagged"
        } else {
            ""
        }
        .to_string(),
        MessageColumn::Answered => if conversation.any_answered {
            "Answered"
        } else {
            ""
        }
        .to_string(),
        MessageColumn::Draft => if conversation.any_draft { "Draft" } else { "" }.to_string(),
        // The worst in the conversation, already picked by the ranking in the
        // column's own expression. Empty for an ordinary one, like the other
        // flag cells.
        MessageColumn::Safety => conversation.worst_safety.label().to_string(),
        MessageColumn::To => everyone_in(&conversation.to),
        MessageColumn::Cc => everyone_in(&conversation.cc),
    }
}

/// A conversation's distinct people, as a sentence.
///
/// They arrive a line apiece, because SQLite joins with a comma and a display
/// name is allowed to contain one: `"Smith, John" <john@example.com>` split on
/// commas is two people who do not exist. Each is then read the way one
/// message's Correspondent cell already reads a sender, so a conversation of
/// one and the message in it say the same thing.
///
/// Empty for a conversation nobody was copied in on, which is most of them, and
/// an empty cell costs no listening time.
fn everyone_in(stored: &str) -> String {
    stored
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(display_address)
        .collect::<Vec<_>>()
        .join(", ")
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
    let name = crate::service::mime::parse_addresses(trimmed)
        .into_iter()
        .next()
        .and_then(|parsed| parsed.name)
        .filter(|name| !name.trim().is_empty());
    name.unwrap_or_else(|| trimmed.to_string())
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
    use crate::application::conversations;

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
    fn test_a_row_whose_text_has_not_been_fetched_says_so_rather_than_looking_empty() {
        // The column said the same thing about two different situations, and
        // one of the two things it said was untrue. A row whose text is not on
        // this computer read as a message with nothing in it, on every row of
        // every folder that has not been through the message text fetch, which
        // is most of a large mailbox.
        let mut not_fetched = message();
        not_fetched.snippet = None;

        assert_eq!(
            cell_text(
                &not_fetched,
                MessageColumn::Snippet,
                DateSettings::default(),
                chrono::Local::now()
            ),
            TEXT_NOT_DOWNLOADED
        );
    }

    #[test]
    fn test_a_message_that_really_has_no_text_is_not_said_the_same_way() {
        // The other half, and the half that makes the first one worth
        // anything. A calendar invitation or a message that is nothing but an
        // attachment really has no text, and telling somebody it has not been
        // downloaded would send them to fetch something that is not there.
        let mut nothing_in_it = message();
        nothing_in_it.snippet = Some(String::new());

        let said = cell_text(
            &nothing_in_it,
            MessageColumn::Snippet,
            DateSettings::default(),
            chrono::Local::now(),
        );

        assert_eq!(said, NO_MESSAGE_TEXT);
        assert_ne!(
            said, TEXT_NOT_DOWNLOADED,
            "a message with no text is described as one nobody has fetched"
        );
    }

    #[test]
    fn test_a_conversation_row_tells_the_two_apart_as_well() {
        // The same column in the other view, and the same lie until now. A
        // conversation whose newest message has not been fetched is not a
        // conversation about nothing.
        let mut not_fetched = conversation();
        not_fetched.snippet = None;
        let mut nothing_in_it = conversation();
        nothing_in_it.snippet = Some(String::new());

        let says = |item: &ConversationItem| {
            conversation_cell_text(
                item,
                MessageColumn::Snippet,
                DateSettings::default(),
                chrono::Local::now(),
            )
        };

        assert_eq!(says(&not_fetched), TEXT_NOT_DOWNLOADED);
        assert_eq!(says(&nothing_in_it), NO_MESSAGE_TEXT);
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
            snippet: Some("The numbers are attached.".to_string()),
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
    fn test_correspondent_recovers_the_whole_name_when_it_contains_a_literal_angle_bracket() {
        // Built from `EmailAddress::new(...).to_string()`, the shape this
        // program's own storage produces, not a hand-quoted stand-in. The
        // naive first-'<' search this replaces cut the name off at the
        // bracket inside it and read back only "Bob".
        let stored = crate::common::types::EmailAddress::new(
            "bob@example.com".to_string(),
            Some("Bob <VIP>".to_string()),
        )
        .to_string();
        let mut m = message();
        m.from = stored;
        assert_eq!(
            cell_text(
                &m,
                MessageColumn::Correspondent,
                DateSettings::default(),
                chrono::Local::now()
            ),
            "Bob <VIP>"
        );
    }

    #[test]
    fn test_display_address_never_panics_on_a_malformed_header() {
        // This module's own contract says the paint callback must never
        // fail. The same malformed-input battery reply.rs and mime.rs both
        // already run through their own address readers.
        for value in [
            "",
            ",",
            ";;;",
            "<",
            ">",
            "<<>>",
            "a>b<c",
            ">x<",
            "\"unclosed <a@example.com>",
            "a@example.com,",
            ",a@example.com",
            "\u{4f60}\u{597d} <ni@example.com>",
        ] {
            let mut m = message();
            m.from = value.to_string();
            m.to = value.to_string();
            m.cc = value.to_string();
            for column in [
                MessageColumn::Correspondent,
                MessageColumn::To,
                MessageColumn::Cc,
            ] {
                let text = cell_text(&m, column, DateSettings::default(), chrono::Local::now());
                assert!(
                    !text.chars().any(|c| c.is_control()),
                    "{:?} produced a control character for {value:?}: {text:?}",
                    column
                );
            }
        }
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

    // -- Conversation rows: one rule per column family, D-02 and D-03 --

    /// A conversation with something to say in every column.
    ///
    /// The ordinary case, and every test below starts from it and changes one
    /// thing. A fixture made only of edge cases cannot tell a working rule from
    /// one that does nothing.
    fn conversation() -> ConversationItem {
        ConversationItem {
            thread_id: "root@example.com".to_string(),
            subject: "Quarterly report".to_string(),
            messages: 5,
            unread: 2,
            newest_received: "2026-07-26T10:00:05+00:00".to_string(),
            newest_sent: "2026-07-26T10:00:00+00:00".to_string(),
            snippet: Some("The figures are attached".to_string()),
            senders: "\nAda Lovelace <ada@example.com>\nBob <bob@example.com>".to_string(),
            to: "\nme@example.com".to_string(),
            cc: "\nChris <chris@example.com>".to_string(),
            size_bytes: Some(4096),
            any_attachment: true,
            any_flagged: true,
            any_answered: true,
            any_draft: true,
            worst_safety: crate::service::safety::Safety::Phishing,
        }
    }

    fn conversation_cell(conversation: &ConversationItem, column: MessageColumn) -> String {
        conversation_cell_text(
            conversation,
            column,
            DateSettings::default(),
            chrono::Local::now(),
        )
    }

    #[test]
    fn test_the_dates_and_the_snippet_take_the_newest_message() {
        // D-02's first family. The row is about the conversation, and what a
        // conversation's date means is when something last happened in it.
        let conversation = conversation();
        let newest = ConversationItem {
            newest_received: "2026-07-28T09:00:00+00:00".to_string(),
            newest_sent: "2026-07-28T08:00:00+00:00".to_string(),
            snippet: Some("One more thing".to_string()),
            ..conversation.clone()
        };

        assert_eq!(
            conversation_cell(&conversation, MessageColumn::Snippet),
            "The figures are attached"
        );
        assert_eq!(
            conversation_cell(&newest, MessageColumn::Snippet),
            "One more thing"
        );
        assert_ne!(
            conversation_cell(&conversation, MessageColumn::Received),
            conversation_cell(&newest, MessageColumn::Received),
            "the Received cell said the same thing for two different days"
        );
        assert_ne!(
            conversation_cell(&conversation, MessageColumn::Sent),
            conversation_cell(&newest, MessageColumn::Sent)
        );
        // The two dates are two rules, not one: Received is when the server
        // took delivery and Sent is what the sender claims.
        let claimed = ConversationItem {
            newest_sent: "2031-01-01T00:00:00+00:00".to_string(),
            ..conversation.clone()
        };
        assert_eq!(
            conversation_cell(&claimed, MessageColumn::Received),
            conversation_cell(&conversation, MessageColumn::Received),
            "a forged sender date moved the arrival time"
        );
    }

    #[test]
    fn test_the_state_columns_are_true_if_any_message_is() {
        // D-02's second family: Attachment, Flagged, Answered and Draft. Four
        // members, and each one gets asked both ways, because a family with one
        // member tested and the rest untested is what mutation testing found
        // here before.
        let any = conversation();
        let none = ConversationItem {
            any_attachment: false,
            any_flagged: false,
            any_answered: false,
            any_draft: false,
            ..any.clone()
        };

        for (column, said) in [
            (MessageColumn::Attachment, "Has attachment"),
            (MessageColumn::Flagged, "Flagged"),
            (MessageColumn::Answered, "Answered"),
            (MessageColumn::Draft, "Draft"),
        ] {
            assert_eq!(conversation_cell(&any, column), said);
            assert_eq!(
                conversation_cell(&none, column),
                "",
                "{column:?} said something about a conversation that has none"
            );
        }
    }

    #[test]
    fn test_a_negative_state_is_an_empty_cell_rather_than_a_word() {
        // The module's own rule, stated for conversations. A cell reading "No"
        // on every row of a mailbox is a word per row that carries nothing, and
        // it is heard rather than seen.
        let none = ConversationItem {
            any_attachment: false,
            ..conversation()
        };
        assert_eq!(conversation_cell(&none, MessageColumn::Attachment), "");
        // Paired with the positive in the same fixture: a cell rule that
        // returned nothing for everything would satisfy the line above.
        assert_eq!(
            conversation_cell(&conversation(), MessageColumn::Attachment),
            "Has attachment"
        );
    }

    #[test]
    fn test_unread_is_true_if_any_message_in_it_is_unread() {
        let some = conversation();
        let all_read = ConversationItem {
            unread: 0,
            ..some.clone()
        };
        assert_eq!(conversation_cell(&some, MessageColumn::Unread), "Unread");
        assert_eq!(conversation_cell(&all_read, MessageColumn::Unread), "");
    }

    #[test]
    fn test_safety_is_the_worst_verdict_in_the_conversation() {
        // D-02's third family, and the one where the worst is not the largest.
        // Safety is stored as words so a person can read the database, and in
        // that alphabet "suspicious" sorts last while being the mildest of the
        // three.
        use crate::service::safety::Safety;

        for verdict in [
            Safety::Ordinary,
            Safety::Suspicious,
            Safety::Spam,
            Safety::Phishing,
        ] {
            let conversation = ConversationItem {
                worst_safety: verdict,
                ..conversation()
            };
            assert_eq!(
                conversation_cell(&conversation, MessageColumn::Safety),
                verdict.label()
            );
        }
        // Ordinary is the empty cell, like the other flag columns.
        assert_eq!(
            conversation_cell(
                &ConversationItem {
                    worst_safety: Safety::Ordinary,
                    ..conversation()
                },
                MessageColumn::Safety
            ),
            ""
        );
    }

    #[test]
    fn test_size_is_the_sum_and_reads_in_units() {
        // D-02's fourth family. The same wording a message's size gets, so a
        // conversation of one and the message in it read the same.
        let conversation = ConversationItem {
            size_bytes: Some(4096),
            ..conversation()
        };
        assert_eq!(
            conversation_cell(&conversation, MessageColumn::Size),
            "4 KB"
        );

        let unknown = ConversationItem {
            size_bytes: None,
            ..conversation.clone()
        };
        assert_eq!(
            conversation_cell(&unknown, MessageColumn::Size),
            "",
            "blank rather than 0 bytes, which is a claim we cannot make"
        );
    }

    #[test]
    fn test_correspondent_lists_the_distinct_senders_by_name() {
        // D-02's fifth family. Names rather than addresses, the same way one
        // message's Correspondent cell already reads, because "Ada Lovelace" is
        // quicker to hear than "ada dot lovelace at example dot com".
        let conversation = conversation();
        assert_eq!(
            conversation_cell(&conversation, MessageColumn::Correspondent),
            "Ada Lovelace, Bob"
        );
        assert_eq!(
            conversation_cell(&conversation, MessageColumn::To),
            "me@example.com"
        );
        assert_eq!(conversation_cell(&conversation, MessageColumn::Cc), "Chris");
        // Nobody copied in is an empty cell, not a stray separator: the list
        // arrives with a leading line break whether or not anybody is on it.
        let nobody = ConversationItem {
            cc: "\n".to_string(),
            ..conversation
        };
        assert_eq!(conversation_cell(&nobody, MessageColumn::Cc), "");
    }

    #[test]
    fn test_a_sender_whose_name_holds_a_comma_is_still_one_person() {
        // "Smith, John" is an ordinary way to write a name and the reason the
        // senders arrive a line apiece rather than comma separated: SQLite's
        // own separator is a comma, so splitting on one would make two people
        // out of one.
        let conversation = ConversationItem {
            senders: "\n\"Smith, John\" <john@example.com>\nAda <ada@example.com>".to_string(),
            ..conversation()
        };
        assert_eq!(
            conversation_cell(&conversation, MessageColumn::Correspondent),
            "Smith, John, Ada"
        );
    }

    #[test]
    fn test_the_subject_cell_is_the_conversations_name() {
        // D-02's sixth family, which is D-04. The name arrives already worked
        // out, because the same rule ordered the list.
        let conversation = ConversationItem {
            subject: "Quarterly report".to_string(),
            ..conversation()
        };
        assert_eq!(
            conversation_cell(&conversation, MessageColumn::Subject),
            "Quarterly report"
        );
        // A conversation whose oldest message had no subject at all still has
        // a name, because a row reading nothing is a row somebody arrows onto
        // and hears silence from.
        let unnamed = ConversationItem {
            subject: String::new(),
            ..conversation
        };
        assert_eq!(
            conversation_cell(&unnamed, MessageColumn::Subject),
            conversations::NO_SUBJECT
        );
    }

    #[test]
    fn test_the_thread_cell_says_the_counts_rather_than_an_identifier() {
        // D-03, and the whole of what makes THREAD-01's announcement possible:
        // there is no per-item accessible name on this list control, so what a
        // screen reader reads for a conversation row is exactly the cell text.
        let conversation = conversation();
        assert_eq!(
            conversation_cell(&conversation, MessageColumn::Thread),
            "5 messages, 2 unread"
        );

        let said = conversation_cell(&conversation, MessageColumn::Thread);
        assert!(
            !said.contains(&conversation.thread_id),
            "the conversation identifier reached the words: {said}"
        );
        assert!(
            !said.contains('@') && !said.contains('<'),
            "something identifier-shaped reached the words: {said}"
        );
    }

    #[test]
    fn test_every_column_has_both_a_conversation_rule_and_a_way_to_sort_by_it() {
        // The mechanism D-02 turns on. `MessageColumn::ALL` is a fixed-size
        // array with a match arm apiece, so a new column cannot arrive with a
        // display rule and no sort rule without the compiler saying so. This
        // says the other half: neither answer is empty, and no two columns
        // share a sort expression, which is what a copied arm looks like.
        let mut seen: Vec<&str> = Vec::new();
        for column in MessageColumn::ALL {
            let expression = column.conversation_sort_expression();
            assert!(
                !expression.trim().is_empty(),
                "{column:?} has no conversation sort rule"
            );
            assert!(
                !seen.contains(&expression),
                "{column:?} shares its conversation sort rule with another column: {expression}"
            );
            seen.push(expression);
        }
        assert_eq!(seen.len(), MessageColumn::ALL.len());

        // Every column answers with something for a conversation that has
        // something to say in all of them. Proving the measurement: the loop
        // would pass over an empty array.
        assert!(MessageColumn::ALL.len() > 10);
        for column in MessageColumn::ALL {
            let said = conversation_cell(&conversation(), column);
            assert!(
                !said.trim().is_empty(),
                "{column:?} said nothing about a conversation that has something \
                 to say in every column"
            );
        }
    }

    #[test]
    fn test_no_conversation_sort_rule_is_built_out_of_the_message_one() {
        // The named anti-pattern. Deriving the aggregate by wrapping the
        // message expression is how D-02's two halves come apart, and it would
        // also give up the property that keeps a sort expression safe to
        // interpolate: a fixed string chosen by matching on the enum.
        for column in MessageColumn::ALL {
            let conversation_rule = column.conversation_sort_expression();
            assert!(
                conversation_rule.contains('(') || conversation_rule.contains("COUNT"),
                "{column:?} takes no aggregate over the conversation: {conversation_rule}"
            );
        }
    }
}

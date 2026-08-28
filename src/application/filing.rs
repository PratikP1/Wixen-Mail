//! Writing a message this program already has in hand into a folder here.
//!
//! Two things do that and they are the same operation: keeping a copy of a
//! message that has just been sent, and bringing in a message read out of a
//! file. Neither came from a server, so both have to be filed rather than
//! stored, and both need a number no server will ever issue.
//!
//! # Why this is one place
//!
//! Getting either half wrong loses the message, and quietly. A row written
//! without the marker is one the next check for mail deletes, along with its
//! text, which for these rows is the only copy there was. A row numbered
//! upward in a folder a server also fills takes the number that server is
//! about to hand out, which hides a real message forever and lets a later
//! fetch write over this one.
//!
//! Both answers were written out twice before this existed, once for sent
//! copies and once for imports, and the numbering a third time. That is the
//! shape every data-losing defect in this program has had.

use crate::data::message_cache::IncomingMessage;
use crate::service::mime::ParsedMessage;

/// Whether the message being filed has been read.
///
/// A copy of something just sent has been: it was written by the person whose
/// folder it lands in, and filing it unread puts a bold row in Sent and a
/// count on the folder for mail they wrote themselves. A message out of a file
/// has not, unless the file said so.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlreadyRead {
    /// Filed read.
    Yes,
    /// Filed unread.
    No,
}

/// The row for a message being filed here.
///
/// The one place that decides what each column holds, so a sent copy and an
/// imported message cannot come to differ about the parts they share.
pub fn a_row_filed_here(
    parsed: &ParsedMessage,
    folder_id: i64,
    uid: u32,
    how_big: usize,
    read: AlreadyRead,
    filed_at: &str,
) -> IncomingMessage {
    IncomingMessage {
        folder_id,
        uid,
        message_id: parsed.message_id.clone().unwrap_or_default(),
        subject: parsed.subject.clone(),
        from_addr: addresses(&parsed.from),
        to_addr: addresses(&parsed.to),
        cc: Some(addresses(&parsed.cc)).filter(|cc| !cc.is_empty()),
        // Where the sender asked replies to go. Dropping it meant replying to
        // your own sent copy went to your own address.
        reply_to: Some(addresses(&parsed.reply_to)).filter(|to| !to.is_empty()),
        // A message with no usable date of its own sorts to whichever end of
        // the folder nobody looks at, which is the same as losing it for
        // anybody reading by ear.
        date: parsed.date.clone().unwrap_or_else(|| filed_at.to_string()),
        internal_date: Some(filed_at.to_string()),
        size_bytes: Some(how_big as i64),
        // The same chain the sync would store, through the same function, so a
        // filed message sits in the conversation it belongs to rather than
        // starting a new one in this program's own list.
        refs_header: crate::application::threading::as_stored(
            &parsed.references,
            parsed.in_reply_to.as_deref(),
        ),
        read: read == AlreadyRead::Yes,
        starred: false,
        answered: false,
        draft: false,
        deleted: false,
        has_attachments: !parsed.attachments.is_empty(),
        safety: crate::service::safety::Verdict::ordinary(),
        gmail_message_id: None,
        labels: None,
        // Deliberately dropped, for both callers and for the same reason. On a
        // message somebody sent, that header is them asking their recipient
        // for a receipt, so carrying it onto their own copy would have the
        // program offer to tell them they had read their own mail. On a
        // message read out of an archive it is a stranger's request from years
        // ago, and answering it now would tell them the address is still live.
        receipt_to: None,
        pop_uidl: None,
    }
}

/// Addresses written out the way a stored row holds them.
///
/// Through the address's own wording, which keeps the name somebody wrote
/// under and quotes it when it carries punctuation that would otherwise read
/// as the separator between two people. Taking the address field alone turns a
/// folder of names into a folder of addresses.
fn addresses(people: &[crate::common::types::EmailAddress]) -> String {
    people
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(raw: &str) -> ParsedMessage {
        crate::service::mime::parse(raw.as_bytes()).expect("a message to parse")
    }

    fn one_message() -> ParsedMessage {
        parsed(concat!(
            "From: Ada Lovelace <ada@example.com>\r\n",
            "To: Charles Babbage <charles@example.com>\r\n",
            "Subject: Notes\r\n",
            "Date: Mon, 20 Jul 2026 10:00:00 +0000\r\n",
            "Message-ID: <note-1@example.com>\r\n",
            "\r\n",
            "The engine weaves patterns.\r\n",
        ))
    }

    #[test]
    fn test_a_filed_row_carries_what_the_message_said() {
        let row = a_row_filed_here(
            &one_message(),
            7,
            42,
            120,
            AlreadyRead::No,
            "2026-08-28T09:00:00Z",
        );

        assert_eq!(row.folder_id, 7);
        assert_eq!(row.uid, 42);
        assert_eq!(row.subject, "Notes");
        assert_eq!(row.from_addr, "Ada Lovelace <ada@example.com>");
        // Without the angle brackets, which is how the reader hands it over
        // and how every other row in the database holds one.
        assert_eq!(row.message_id, "note-1@example.com");
    }

    #[test]
    fn test_the_name_somebody_wrote_under_survives_being_filed() {
        // Kept, because it is what the message list shows and what a screen
        // reader says on every row. Writing the address alone turns a folder
        // of names into a folder of addresses, and it goes unnoticed: pulling
        // this out of the sent copy path dropped it, and all twenty-two tests
        // there passed, because not one of them looked at a display name.
        let row = a_row_filed_here(
            &one_message(),
            1,
            1,
            10,
            AlreadyRead::No,
            "2026-08-28T09:00:00Z",
        );

        assert_eq!(row.from_addr, "Ada Lovelace <ada@example.com>");
        assert_eq!(row.to_addr, "Charles Babbage <charles@example.com>");
    }

    #[test]
    fn test_a_message_read_out_of_a_file_is_never_asked_to_send_a_receipt() {
        // A receipt request in an archive is a stranger's from years ago, and
        // answering it now tells them the address is still live. The sent copy
        // path dropped this for its own reason and the two agree here.
        let asking = parsed(concat!(
            "From: Someone <someone@example.com>\r\n",
            "To: Kit <kit@example.com>\r\n",
            "Subject: Old mail\r\n",
            "Disposition-Notification-To: someone@example.com\r\n",
            "\r\n",
            "Body\r\n",
        ));

        let row = a_row_filed_here(&asking, 1, 1, 10, AlreadyRead::No, "2026-08-28T09:00:00Z");

        assert_eq!(row.receipt_to, None);
    }

    #[test]
    fn test_a_message_with_no_date_of_its_own_takes_the_moment_it_was_filed() {
        // An empty date sorts it to whichever end of the folder nobody looks
        // at, which for somebody reading by ear is the same as losing it.
        let undated = parsed("From: a@example.com\r\nSubject: No date\r\n\r\nBody\r\n");

        let row = a_row_filed_here(&undated, 1, 1, 10, AlreadyRead::No, "2026-08-28T09:00:00Z");

        assert_eq!(row.date, "2026-08-28T09:00:00Z");
    }

    #[test]
    fn test_whether_it_is_read_is_the_callers_answer_and_not_a_default() {
        // The two callers differ here and nowhere else: a copy of something
        // just sent has been read by the person whose folder it lands in, and
        // a message out of an archive has not.
        let read = a_row_filed_here(
            &one_message(),
            1,
            1,
            10,
            AlreadyRead::Yes,
            "2026-08-28T09:00:00Z",
        );
        let unread = a_row_filed_here(
            &one_message(),
            1,
            1,
            10,
            AlreadyRead::No,
            "2026-08-28T09:00:00Z",
        );

        assert!(read.read);
        assert!(!unread.read);
    }
}

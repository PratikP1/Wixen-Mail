//! A message whose parts the download of everything left behind, fetched
//! once more when it is selected.
//!
//! Until 13-10's build the download kept a message's text and nothing about
//! its parts, and the reader fetches nothing for a message whose text is
//! here, so such a message lists no attachments and its meeting is never
//! said (ledger 633). Pratik's answer of 2026-09-26: the reader fetches the
//! whole message once, when its row says it has attachments and none are
//! stored, and keeps what it learns; the download itself goes on keeping
//! names and calendar documents only, because of the 512 MB budget.
//!
//! This module is the part of that which can be asked without a window: the
//! question, the keeping, and which account's server to ask. The fetch
//! itself is the preview's, in `presentation::wx_app`.
//!
//! Nothing here reads what opening an envelope or OpenPGP found, and nothing
//! records the form a message arrived in: whether a message is fetched
//! depends on the stored bit and the stored part rows and on nothing else.

use crate::application::answering;
use crate::common::Result;
use crate::data::account::Account;
use crate::data::message_cache::MessageCache;
use crate::data::message_cache::attachment_content::AttachmentWithContent;
use crate::service::mime::{self, AttachmentInfo};

/// What keeping a fetched message's parts did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeptParts {
    /// How many parts were recorded.
    pub count: usize,
    /// Whether one of them is a calendar document, which is the one kept
    /// part that changes what the preview says.
    pub carries_a_calendar_part: bool,
}

/// Whether this message's parts were left behind and a server can be asked
/// for them.
///
/// No for a store that cannot be read, since nothing can be kept there
/// either; the error is logged by row id and never by anything the message
/// says.
pub fn were_left_behind(cache: Option<&MessageCache>, message_row_id: i64) -> bool {
    let Some(cache) = cache else {
        return false;
    };
    cache
        .parts_were_left_behind(message_row_id)
        .unwrap_or_else(|e| {
            tracing::warn!("Could not ask whether message {message_row_id}'s parts are here: {e}");
            false
        })
}

/// Keep every part of a message fetched whole, each with its file.
///
/// The reader's rule for a message it fetched on selection, under the same
/// limits: a file over 25 MB is listed and not kept, and the store as a whole
/// stays under its 512 MB budget. A message whose own parse finds no
/// attachment is recorded as carrying none, so it is not fetched again.
pub fn keep_every_part(
    cache: &MessageCache,
    message_row_id: i64,
    parts: &[AttachmentInfo],
    raw: &[u8],
) -> Result<KeptParts> {
    // One walk of the message for every file rather than one per part, and
    // by position, which the walk and the parse that named the parts share.
    // A walk that fails keeps the names without the files, which still lists
    // them.
    let files = mime::attachments_with_bytes(raw).unwrap_or_else(|e| {
        tracing::warn!("Could not read the files of message {message_row_id}: {e}");
        Vec::new()
    });
    let kept: Vec<AttachmentWithContent> = parts
        .iter()
        .enumerate()
        .map(|(at, part)| {
            AttachmentWithContent::from_a_parsed_part(
                message_row_id,
                part,
                // every part with its file, not only the calendar's
                files.get(at).map(|file| file.bytes.clone()),
            )
        })
        .collect();
    // Replaced rather than added to, so a body downloaded again does not list
    // every part twice.
    cache.replace_attachments_with_content(message_row_id, &kept)?;
    if kept.is_empty() {
        cache.record_that_it_carries_no_attachments(message_row_id)?;
    }
    Ok(KeptParts {
        count: kept.len(),
        carries_a_calendar_part: parts
            .iter()
            .any(|part| answering::is_a_calendar_part(&part.mime_type)),
    })
}

/// The account a message is fetched through: the one it is filed under.
///
/// `None` when that names no account here, and never the account last
/// opened or the first one: asking another account's server for this
/// folder and number would store whatever message it holds there under this
/// row.
pub fn the_account_to_fetch_through(
    accounts: &[Account],
    filed_under: Option<&str>,
) -> Option<Account> {
    let filed_under = filed_under?;
    // the message's own account, or none
    accounts
        .iter()
        .find(|account| account.id == filed_under)
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{CachedFolder, IncomingMessage};
    use crate::service::mime;

    /// A message carrying a PDF, which the download keeps the name of and
    /// not the file.
    const WITH_A_PDF: &str = "From: Ada Lovelace <ada@example.com>\r\n\
To: me@example.com\r\n\
Subject: The agenda\r\n\
Message-ID: <agenda@example.com>\r\n\
MIME-Version: 1.0\r\n\
Content-Type: multipart/mixed; boundary=\"mix\"\r\n\
\r\n\
--mix\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
The agenda is attached.\r\n\
--mix\r\n\
Content-Type: application/pdf; name=\"agenda.pdf\"\r\n\
Content-Disposition: attachment; filename=\"agenda.pdf\"\r\n\
Content-Transfer-Encoding: base64\r\n\
\r\n\
JVBERi0xLjQKJSBhZ2VuZGEK\r\n\
--mix--\r\n";

    /// A meeting sent as a file.
    const WITH_A_MEETING: &str = "From: Ada Lovelace <ada@example.com>\r\n\
To: me@example.com\r\n\
Subject: Invitation: Quarterly review\r\n\
Message-ID: <meeting@example.com>\r\n\
MIME-Version: 1.0\r\n\
Content-Type: multipart/mixed; boundary=\"mix\"\r\n\
\r\n\
--mix\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
Are you free?\r\n\
--mix\r\n\
Content-Type: application/ics; name=\"invite.ics\"\r\n\
Content-Disposition: attachment; filename=\"invite.ics\"\r\n\
\r\n\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
METHOD:REQUEST\r\n\
BEGIN:VEVENT\r\n\
UID:m-9@example.com\r\n\
SUMMARY:Quarterly review\r\n\
DTSTART:20260305T090000\r\n\
DTEND:20260305T100000\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n\
--mix--\r\n";

    const PLAIN: &str = "From: Ada Lovelace <ada@example.com>\r\n\
To: me@example.com\r\n\
Subject: Nothing attached\r\n\
Message-ID: <plain@example.com>\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
Nothing is attached.\r\n";

    fn a_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_keeping_left_behind_", |dir| {
            let cache = MessageCache::new(dir.to_path_buf(), None).expect("cache");
            cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: "acc-1".to_string(),
                    name: "INBOX".to_string(),
                    path: "INBOX".to_string(),
                    folder_type: "Inbox".to_string(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("a folder");
            cache
        })
    }

    /// A row as the header sync writes it, with the bit it takes from the
    /// server's description, and nothing about its parts.
    fn a_row(cache: &MessageCache, uid: u32, has_attachments: bool) -> i64 {
        cache
            .upsert_message(&IncomingMessage {
                folder_id: 1,
                uid,
                message_id: format!("<{uid}@example.com>"),
                subject: "Something".to_string(),
                from_addr: "ada@example.com".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                reply_to: None,
                date: "2026-09-06".to_string(),
                internal_date: None,
                size_bytes: None,
                refs_header: None,
                read: false,
                starred: false,
                answered: false,
                draft: false,
                deleted: false,
                has_attachments,
                safety: crate::service::safety::Verdict::ordinary(),
                gmail_message_id: None,
                server_thread_id: None,
                labels: None,
                receipt_to: None,
                list_unsubscribe: None,
                pop_uidl: None,
            })
            .expect("the header sync's write")
    }

    fn kept(cache: &MessageCache, row: i64, raw: &str) -> KeptParts {
        let parsed = mime::parse(raw.as_bytes()).expect("parsed");
        keep_every_part(cache, row, &parsed.attachments, raw.as_bytes()).expect("kept")
    }

    fn an_account(id: &str) -> Account {
        Account {
            id: id.to_string(),
            imap_server: format!("imap.{id}.example.com"),
            ..Account::default()
        }
    }

    #[test]
    fn test_no_store_means_nothing_was_left_behind() {
        assert!(!were_left_behind(None, 1));
    }

    #[test]
    fn test_a_stored_message_saying_it_carries_attachments_with_none_recorded_is_left_behind() {
        let cache = a_cache();
        let says_none = a_row(&cache, 1, false);
        let says_some = a_row(&cache, 2, true);

        assert!(were_left_behind(Some(&cache), says_some));
        assert!(!were_left_behind(Some(&cache), says_none));
    }

    #[test]
    fn test_every_part_is_kept_with_its_file_where_the_download_keeps_none() {
        // The download keeps a PDF's name and not its bytes; a message
        // fetched on selection keeps them, as the reader always has.
        let cache = a_cache();
        let row = a_row(&cache, 1, true);

        let answer = kept(&cache, row, WITH_A_PDF);

        let held = cache.attachments_with_content(row).expect("read");
        assert_eq!(answer.count, 1);
        assert_eq!(held.len(), 1);
        assert_eq!(held[0].described.filename, "agenda.pdf");
        assert_eq!(
            held[0].content.as_deref(),
            Some(b"%PDF-1.4\n% agenda\n".as_slice()),
            "the PDF was listed and its file not kept"
        );
        assert!(!answer.carries_a_calendar_part);
    }

    #[test]
    fn test_keeping_replaces_what_was_recorded_rather_than_adding_to_it() {
        let cache = a_cache();
        let row = a_row(&cache, 1, true);

        kept(&cache, row, WITH_A_PDF);
        kept(&cache, row, WITH_A_PDF);

        assert_eq!(cache.attachments_with_content(row).expect("read").len(), 1);
    }

    #[test]
    fn test_a_kept_calendar_part_is_reported() {
        let cache = a_cache();
        let meeting = a_row(&cache, 1, true);
        let pdf = a_row(&cache, 2, true);

        assert!(kept(&cache, meeting, WITH_A_MEETING).carries_a_calendar_part);
        assert!(!kept(&cache, pdf, WITH_A_PDF).carries_a_calendar_part);
    }

    #[test]
    fn test_a_message_whose_parse_finds_no_attachment_is_not_left_behind_after_keeping() {
        let cache = a_cache();
        let row = a_row(&cache, 1, true);
        assert!(were_left_behind(Some(&cache), row));

        let answer = kept(&cache, row, PLAIN);

        assert_eq!(answer.count, 0);
        assert!(
            !were_left_behind(Some(&cache), row),
            "a message with nothing to keep would be fetched on every selection"
        );
    }

    #[test]
    fn test_the_account_fetched_through_is_the_one_the_message_is_filed_under() {
        let accounts = [an_account("first"), an_account("second")];

        let chosen = the_account_to_fetch_through(&accounts, Some("second"));

        assert_eq!(chosen.map(|account| account.id), Some("second".to_string()));
    }

    #[test]
    fn test_an_account_not_known_here_is_fetched_through_nothing() {
        let accounts = [an_account("first"), an_account("second")];

        assert!(the_account_to_fetch_through(&accounts, Some("gone")).is_none());
        assert!(the_account_to_fetch_through(&accounts, None).is_none());
        assert!(
            the_account_to_fetch_through(&accounts, Some("first")).is_some(),
            "the answer is always no"
        );
    }
}

//! A message the download of everything brought before 13-10's build, fetched
//! once for its parts when it is selected.
//!
//! Until 13-10 the download kept a message's text and nothing about its
//! parts, and the reader fetches nothing for a message whose text is here. So
//! such a message listed no attachments and its meeting was never said
//! (ledger 633). Pratik answered on 2026-09-26: fetch the whole message once
//! when the row says it has attachments and none are stored, keep what it
//! carries, and do not announce it twice.
//!
//! Each case builds a store the old way: the row as the header sync writes
//! it, with the bit the server's description gives that shape, then the text
//! and the form it arrived in, the text pass's two steps before 13-10, and
//! nothing else. What it cannot see: a real server's answer, which is phase
//! 14's, and how any of it sounds.

use wixen_mail::application::parts_left_behind::{self, KeptParts};
use wixen_mail::application::reading_a_message;
use wixen_mail::common::types::MessageBody;
use wixen_mail::data::message_cache::{CachedFolder, IncomingMessage, MessageCache};
use wixen_mail::presentation::date_display::{
    Clock, DateOrder, DateSettings, DateStyle, DateWording,
};
use wixen_mail::service::mime;

/// An invitation shaped the way Google Calendar sends one, from the
/// standard rather than from a real message: the covering note and the
/// meeting as three alternatives, and the same document again as a file
/// with an attachment disposition. The file is what makes the server count
/// an attachment.
const AN_INVITATION_WITH_ITS_FILE: &str = "From: Ada Lovelace <ada@example.com>\r\n\
To: me@example.com\r\n\
Subject: Invitation: Quarterly review\r\n\
Message-ID: <invite-google@example.com>\r\n\
Date: Mon, 2 Mar 2026 10:00:00 +0000\r\n\
MIME-Version: 1.0\r\n\
Content-Type: multipart/mixed; boundary=\"mix\"\r\n\
\r\n\
--mix\r\n\
Content-Type: multipart/alternative; boundary=\"alt\"\r\n\
\r\n\
--alt\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
Are you free for the quarterly review?\r\n\
--alt\r\n\
Content-Type: text/html; charset=utf-8\r\n\
\r\n\
<p>Are you free for the quarterly review?</p>\r\n\
--alt\r\n\
Content-Type: text/calendar; charset=utf-8; method=REQUEST\r\n\
\r\n\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example//EN\r\n\
METHOD:REQUEST\r\n\
BEGIN:VEVENT\r\n\
UID:m-google@example.com\r\n\
SEQUENCE:0\r\n\
SUMMARY:Quarterly review\r\n\
LOCATION:Room 4\r\n\
DTSTART:20260305T090000\r\n\
DTEND:20260305T100000\r\n\
ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n\
ATTENDEE;CN=Me;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:me@example.com\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n\
--alt--\r\n\
--mix\r\n\
Content-Type: application/ics; name=\"invite.ics\"\r\n\
Content-Disposition: attachment; filename=\"invite.ics\"\r\n\
\r\n\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example//EN\r\n\
METHOD:REQUEST\r\n\
BEGIN:VEVENT\r\n\
UID:m-google@example.com\r\n\
SEQUENCE:0\r\n\
SUMMARY:Quarterly review\r\n\
LOCATION:Room 4\r\n\
DTSTART:20260305T090000\r\n\
DTEND:20260305T100000\r\n\
ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n\
ATTENDEE;CN=Me;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:me@example.com\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n\
--mix--\r\n";

/// The shape the audit of 2026-09-15 found: the meeting only as a third
/// alternative, with no disposition. The server's description counts a text
/// part as an attachment only with one, so the header sync stores no bit.
const AN_INVITATION_ONLY_AS_AN_ALTERNATIVE: &str = "From: Ada Lovelace <ada@example.com>\r\n\
To: me@example.com\r\n\
Subject: Invitation: Quarterly review\r\n\
Message-ID: <invite-alternative@example.com>\r\n\
Date: Mon, 2 Mar 2026 10:00:00 +0000\r\n\
MIME-Version: 1.0\r\n\
Content-Type: multipart/alternative; boundary=\"alt\"\r\n\
\r\n\
--alt\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
Are you free for the quarterly review?\r\n\
--alt\r\n\
Content-Type: text/calendar; charset=utf-8; method=REQUEST\r\n\
\r\n\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
METHOD:REQUEST\r\n\
BEGIN:VEVENT\r\n\
UID:m-alternative@example.com\r\n\
SUMMARY:Quarterly review\r\n\
DTSTART:20260305T090000\r\n\
DTEND:20260305T100000\r\n\
ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n\
--alt--\r\n";

/// A note the server counts an attachment in and the parse reads as text: a
/// plain message stands for the two disagreeing.
const A_NOTE_WITH_NOTHING_TO_KEEP: &str = "From: Ada Lovelace <ada@example.com>\r\n\
To: me@example.com\r\n\
Subject: Nothing attached after all\r\n\
Message-ID: <nothing@example.com>\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
Nothing to keep here.\r\n";

const THE_SENDER: &str = "Ada Lovelace <ada@example.com>";

fn written_out_in_full() -> DateSettings {
    DateSettings {
        style: DateStyle::Absolute,
        order: DateOrder::DayFirst,
        wording: DateWording::Numeric,
        clock: Clock::TwentyFourHour,
    }
}

/// A store with one account's inbox in it.
fn a_store(dir: &tempfile::TempDir) -> MessageCache {
    let cache = MessageCache::new(dir.path().join("earlier.db"), None).expect("a store");
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
        .expect("an inbox");
    cache
}

/// How a message reached the store.
struct Arrived {
    uid: u32,
    /// The bit the header sync takes from the server's description.
    says_it_carries_attachments: bool,
    /// The identifier a POP server gave it, for a message collected that way.
    pop_uidl: Option<String>,
}

impl Arrived {
    fn over_imap(uid: u32, says_it_carries_attachments: bool) -> Self {
        Self {
            uid,
            says_it_carries_attachments,
            pop_uidl: None,
        }
    }
}

/// A message stored the way the download of everything stored one before
/// 13-10: the header sync's row, then the text and the form it arrived in,
/// and nothing about its parts.
fn brought_the_old_way(cache: &MessageCache, arrived: Arrived, raw: &str) -> i64 {
    let parsed = mime::parse(raw.as_bytes()).expect("the message to parse");
    let row = cache
        .upsert_message(&IncomingMessage {
            folder_id: 1,
            uid: arrived.uid,
            message_id: parsed.message_id.clone().unwrap_or_default(),
            subject: parsed.subject.clone(),
            from_addr: THE_SENDER.to_string(),
            to_addr: "me@example.com".to_string(),
            cc: None,
            reply_to: None,
            date: "2026-03-02T10:00:00+00:00".to_string(),
            internal_date: None,
            size_bytes: Some(raw.len() as i64),
            refs_header: None,
            read: false,
            starred: false,
            answered: false,
            draft: false,
            deleted: false,
            has_attachments: arrived.says_it_carries_attachments,
            safety: wixen_mail::service::safety::Verdict::ordinary(),
            gmail_message_id: None,
            server_thread_id: None,
            labels: None,
            receipt_to: None,
            list_unsubscribe: None,
            pop_uidl: arrived.pop_uidl,
        })
        .expect("the header sync's row");
    cache
        .save_message_body(
            row,
            parsed.body_plain.as_deref(),
            parsed.body_html.as_deref(),
        )
        .expect("the text kept");
    cache
        .note_the_form_it_arrived_in(row, raw.as_bytes())
        .expect("the form noted");
    row
}

/// The whole message fetched again and its parts kept, the way the reader
/// keeps them for a message it fetched on selection.
fn fetched_again(cache: &MessageCache, row: i64, raw: &str) -> KeptParts {
    let parsed = mime::parse(raw.as_bytes()).expect("the message to parse");
    parts_left_behind::keep_every_part(cache, row, &parsed.attachments, raw.as_bytes())
        .expect("the parts kept")
}

fn left_behind(cache: &MessageCache, row: i64) -> bool {
    parts_left_behind::were_left_behind(Some(cache), row)
}

/// What every surface asks about the message, the way they ask it.
fn the_meeting_said(cache: &MessageCache, row: i64) -> Option<String> {
    let body = cache
        .get_message_body(row)
        .expect("the body read")
        .and_then(|body| body.body_plain)
        .map(MessageBody::Plain)
        .unwrap_or_else(|| MessageBody::Plain(String::new()));
    reading_a_message::for_message(
        Some(cache),
        row,
        THE_SENDER,
        body,
        written_out_in_full,
        |_| reading_a_message::AnsweringAs {
            address: "me@example.com".to_string(),
            allowed: wixen_mail::application::allowed::Allowed::EVERYTHING,
        },
    )
    .said
    .invitation
    .said()
}

#[test]
fn test_a_message_brought_before_its_parts_were_kept_lists_them_and_says_its_meeting_once_fetched()
{
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    let row = brought_the_old_way(
        &cache,
        Arrived::over_imap(1, true),
        AN_INVITATION_WITH_ITS_FILE,
    );
    assert!(
        left_behind(&cache, row),
        "a message whose row says it has attachments and has none stored"
    );
    assert_eq!(
        the_meeting_said(&cache, row),
        None,
        "said before it was fetched"
    );

    let kept = fetched_again(&cache, row, AN_INVITATION_WITH_ITS_FILE);

    let parts = cache.attachments_with_content(row).expect("the parts read");
    let named: Vec<(&str, bool)> = parts
        .iter()
        .map(|part| (part.described.filename.as_str(), part.content.is_some()))
        .collect();
    assert!(
        !parts.is_empty() && parts.iter().all(|part| part.content.is_some()),
        "every part listed and each with its file: {named:?}"
    );
    assert!(
        named.iter().any(|(name, _)| *name == "invite.ics"),
        "{named:?}"
    );
    assert_eq!(kept.count, parts.len());
    assert!(kept.carries_a_calendar_part);
    assert!(
        the_meeting_said(&cache, row)
            .is_some_and(|said| said.starts_with("Meeting invitation: Quarterly review")),
        "{:?}",
        the_meeting_said(&cache, row)
    );
    assert!(
        !left_behind(&cache, row),
        "a message whose parts are kept would be fetched again"
    );
}

#[test]
fn test_a_message_the_download_kept_the_new_way_is_not_fetched_again() {
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    let old_way = brought_the_old_way(
        &cache,
        Arrived::over_imap(1, true),
        AN_INVITATION_WITH_ITS_FILE,
    );
    let new_way = brought_the_old_way(
        &cache,
        Arrived::over_imap(2, true),
        AN_INVITATION_WITH_ITS_FILE,
    );
    let parsed = mime::parse(AN_INVITATION_WITH_ITS_FILE.as_bytes()).expect("parsed");
    reading_a_message::keep_what_a_download_carried(
        &cache,
        new_way,
        &parsed.attachments,
        AN_INVITATION_WITH_ITS_FILE.as_bytes(),
    )
    .expect("kept the way the download keeps since 13-10");

    assert!(
        !left_behind(&cache, new_way),
        "a message the download kept since 13-10 would be fetched again"
    );
    assert!(left_behind(&cache, old_way));
}

#[test]
fn test_a_message_collected_over_pop_is_not_fetched_again() {
    // A POP server is not asked again: this computer has the only copy
    // (ledger 92).
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    let collected = brought_the_old_way(
        &cache,
        Arrived {
            uid: 1,
            says_it_carries_attachments: true,
            pop_uidl: Some("uidl-1".to_string()),
        },
        AN_INVITATION_WITH_ITS_FILE,
    );
    let over_imap = brought_the_old_way(
        &cache,
        Arrived::over_imap(2, true),
        AN_INVITATION_WITH_ITS_FILE,
    );

    assert!(
        !left_behind(&cache, collected),
        "a message collected over POP would be asked of a server that no longer has it"
    );
    assert!(left_behind(&cache, over_imap));
}

#[test]
fn test_a_message_whose_parse_finds_no_attachment_is_fetched_once_and_not_again() {
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    let row = brought_the_old_way(
        &cache,
        Arrived::over_imap(1, true),
        A_NOTE_WITH_NOTHING_TO_KEEP,
    );
    assert!(left_behind(&cache, row));

    let kept = fetched_again(&cache, row, A_NOTE_WITH_NOTHING_TO_KEEP);

    assert_eq!(kept.count, 0);
    assert!(
        !left_behind(&cache, row),
        "the server and the parse disagree, and the message would be fetched on every selection"
    );
}

#[test]
fn test_an_invitation_carried_only_as_an_alternative_is_not_reached_while_its_row_says_none() {
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    let alternative_only = brought_the_old_way(
        &cache,
        Arrived::over_imap(1, false),
        AN_INVITATION_ONLY_AS_AN_ALTERNATIVE,
    );
    let with_its_file = brought_the_old_way(
        &cache,
        Arrived::over_imap(2, true),
        AN_INVITATION_WITH_ITS_FILE,
    );

    assert!(
        !left_behind(&cache, alternative_only),
        "premise 6 of 13-21.3: the accepted rule does not reach an invitation the header \
         sync stored with no attachment bit. Reaching it is a wider rule, a fetch of every \
         earlier download once, which is Pratik's to choose (the ledger todo 13-21.3 \
         opened). Change this case on purpose, with that answer."
    );
    assert!(left_behind(&cache, with_its_file));
}

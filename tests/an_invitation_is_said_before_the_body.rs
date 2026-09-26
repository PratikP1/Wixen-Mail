//! A meeting invitation, from a whole raw message to the words said before
//! its body.
//!
//! The pieces each have their own tests: the decision in
//! `application::invitations`, the finder in `application::answering`, the
//! fold in `presentation::reader_text`. What none of them can show is that a
//! message as a server sends it gets there. Until 2026-09-25 no fixture in the
//! tree carried a calendar part at all (#50 point 5), so nothing said whether
//! the parser lists one, whether the store keeps its bytes, or whether the
//! reader finds it again.
//!
//! So each case here starts from the bytes: parsed by `service::mime::parse`,
//! its parts stored the way opening a message stores them, asked through
//! `reading_a_message::for_message` as every surface asks, and composed the
//! way the text reader, the formatted window and the preview compose it.
//!
//! # What this cannot see
//!
//! Nothing here was written by Outlook, Google Calendar or a CalDAV server.
//! The two messages are shaped the way those send one, from the standards and
//! from what the audit of 2026-09-15 found, and a real organiser's message is
//! phase 14's to read. Nor does anything here say how the sentence sounds.

use wixen_mail::application::checking_signatures::SignatureCheck;
use wixen_mail::application::reading_a_message::{self, WhatAMessageShowsAndSays};
use wixen_mail::common::types::MessageBody;
use wixen_mail::data::message_cache::attachment_content::AttachmentWithContent;
use wixen_mail::data::message_cache::{CachedFolder, CachedMessage, MessageCache};
use wixen_mail::presentation::date_display::{
    Clock, DateOrder, DateSettings, DateStyle, DateWording,
};
use wixen_mail::presentation::read_aloud::Reading;
use wixen_mail::presentation::reader_text::{self, ConversationPart, ReaderDocument};
use wixen_mail::presentation::ui_types::{AttachmentItem, MessageItem};
use wixen_mail::service::mime;

/// An invitation the way a calendar server sends one: the covering note as
/// text and as HTML, and the meeting as a third alternative, at nine on the
/// clock so what is said does not depend on the zone the test runs in.
const AN_INVITATION: &str = "From: Ada Lovelace <ada@example.com>\r\n\
To: me@example.com\r\n\
Subject: Invitation: Quarterly review\r\n\
Message-ID: <invite-1@example.com>\r\n\
Date: Mon, 2 Mar 2026 10:00:00 +0000\r\n\
MIME-Version: 1.0\r\n\
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
UID:m-1@example.com\r\n\
SEQUENCE:0\r\n\
SUMMARY:Quarterly review\r\n\
LOCATION:Room 4\r\n\
DTSTART:20260305T090000\r\n\
DTEND:20260305T100000\r\n\
ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n\
ATTENDEE;CN=Me;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:me@example.com\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n\
--alt--\r\n";

/// A cancellation sent as a file attachment of type `application/ics`, which
/// some senders write instead of `text/calendar` (#50 point 5).
const A_CANCELLATION_AS_A_FILE: &str = "From: Ada Lovelace <ada@example.com>\r\n\
To: me@example.com\r\n\
Subject: Cancelled: Quarterly review\r\n\
Message-ID: <cancel-1@example.com>\r\n\
Date: Tue, 3 Mar 2026 10:00:00 +0000\r\n\
MIME-Version: 1.0\r\n\
Content-Type: multipart/mixed; boundary=\"mix\"\r\n\
\r\n\
--mix\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
The quarterly review is off.\r\n\
--mix\r\n\
Content-Type: application/ics; method=CANCEL; name=\"cancel.ics\"\r\n\
Content-Disposition: attachment; filename=\"cancel.ics\"\r\n\
\r\n\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
METHOD:CANCEL\r\n\
BEGIN:VEVENT\r\n\
UID:m-1@example.com\r\n\
SEQUENCE:1\r\n\
SUMMARY:Quarterly review\r\n\
DTSTART:20260305T090000\r\n\
DTEND:20260305T100000\r\n\
ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n\
STATUS:CANCELLED\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n\
--mix--\r\n";

const THE_SENDER: &str = "Ada Lovelace <ada@example.com>";

/// Dates written out in full, so the sentence is the same on any machine.
fn written_out_in_full() -> DateSettings {
    DateSettings {
        style: DateStyle::Absolute,
        order: DateOrder::DayFirst,
        wording: DateWording::Numeric,
        clock: Clock::TwentyFourHour,
    }
}

fn reading() -> Reading {
    Reading {
        dates: written_out_in_full(),
        now: chrono::Local::now(),
    }
}

/// A store with one account's inbox in it.
fn a_store(dir: &tempfile::TempDir) -> MessageCache {
    let cache = MessageCache::new(dir.path().join("invitations.db"), None).expect("a store");
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

/// A raw message opened: parsed, its row saved, its parts stored with their
/// bytes the way the reader's open path stores them, and the row the reader
/// is handed built from what was stored.
fn opened(cache: &MessageCache, uid: u32, raw: &str) -> (MessageItem, MessageBody) {
    let parsed = mime::parse(raw.as_bytes()).expect("the message to parse");
    let row = cache
        .save_message(&CachedMessage {
            id: 0,
            uid,
            folder_id: 1,
            message_id: parsed.message_id.clone().unwrap_or_default(),
            subject: parsed.subject.clone(),
            from_addr: THE_SENDER.to_string(),
            to_addr: "me@example.com".to_string(),
            cc: None,
            date: parsed.date.clone().unwrap_or_default(),
            body_plain: parsed.body_plain.clone(),
            body_html: parsed.body_html.clone(),
            read: false,
            starred: false,
            deleted: false,
            safety: wixen_mail::service::safety::Safety::Ordinary,
        })
        .expect("the row saved");
    let files = mime::attachments_with_bytes(raw.as_bytes()).expect("the parts to read");
    let records: Vec<AttachmentWithContent> = parsed
        .attachments
        .iter()
        .enumerate()
        .map(|(at, part)| {
            AttachmentWithContent::from_a_parsed_part(
                row,
                part,
                files.get(at).map(|file| file.bytes.clone()),
            )
        })
        .collect();
    cache
        .replace_attachments_with_content(row, &records)
        .expect("the parts stored");

    let item = MessageItem {
        message_id: row,
        uid,
        subject: parsed.subject.clone(),
        from: THE_SENDER.to_string(),
        has_attachments: !parsed.attachments.is_empty(),
        attachments: parsed
            .attachments
            .iter()
            .map(|part| AttachmentItem {
                filename: part.display_name(),
                mime_type: part.mime_type.clone(),
                size: part.size,
                description: part.description.clone(),
            })
            .collect(),
        ..Default::default()
    };
    let body = match (parsed.body_html, parsed.body_plain) {
        (Some(html), _) => MessageBody::Html(html),
        (None, Some(plain)) => MessageBody::Plain(plain),
        (None, None) => MessageBody::Plain(String::new()),
    };
    (item, body)
}

/// What every surface asks about the message, the way they ask it.
fn asked(cache: &MessageCache, item: &MessageItem, body: MessageBody) -> WhatAMessageShowsAndSays {
    reading_a_message::for_message(
        Some(cache),
        item.message_id,
        &item.from,
        body,
        written_out_in_full,
        // The guest the invitations here name, with sending switched on, so
        // the bar holds the meeting's sentence and nothing about answering:
        // why a meeting cannot be answered is 13-11's target's to read.
        |_| reading_a_message::AnsweringAs {
            address: "me@example.com".to_string(),
            allowed: wixen_mail::application::allowed::Allowed::EVERYTHING,
        },
    )
}

/// The message as the text reader shows it.
fn in_the_text_reader(item: &MessageItem, shown: &WhatAMessageShowsAndSays) -> ReaderDocument {
    reader_text::single_message(item, &shown.body, reading()).with_what_is_said(&shown.said)
}

/// The top of the bar, which is what is spoken as the message opens.
fn spoken_as_it_opens(document: &ReaderDocument) -> String {
    reader_text::said_before_the_message(
        document
            .warning
            .as_deref()
            .expect("a message carrying a meeting has a bar"),
    )
    .to_string()
}

/// The calendar part's row in a list of attachment rows.
fn the_calendar_row(rows: &[reader_text::ReaderAttachment]) -> String {
    rows.iter()
        .find(|row| row.name.ends_with(".ics") || row.mime_type.contains("calendar"))
        .expect("the calendar part is listed")
        .label()
}

const THE_INVITATION_SAID: &str = "Meeting invitation: Quarterly review, 05/03/2026 at 09:00 \
     to 10:00, in Room 4, from Ada Lovelace, and it is new to your calendar.";

#[test]
fn test_a_raw_invitation_is_said_at_the_top_of_the_bar_and_the_body() {
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    let (item, body) = opened(&cache, 1, AN_INVITATION);

    let document = in_the_text_reader(&item, &asked(&cache, &item, body));

    assert_eq!(spoken_as_it_opens(&document), THE_INVITATION_SAID);
    let said_at = document
        .text
        .find(THE_INVITATION_SAID)
        .expect("the sentence is at the top of the body too");
    let covering_note = document
        .text
        .find("Are you free")
        .expect("the covering note is still there");
    assert!(
        document.text.starts_with("Subject:") && said_at < covering_note,
        "the sentence is not between the header lines and the body:\n{}",
        document.text
    );
    assert!(
        the_calendar_row(&document.attachments).contains(", meeting invitation,"),
        "{}",
        the_calendar_row(&document.attachments)
    );
}

#[test]
fn test_a_raw_cancellation_sent_as_application_ics_is_said_as_one() {
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    let (item, body) = opened(&cache, 2, A_CANCELLATION_AS_A_FILE);

    let document = in_the_text_reader(&item, &asked(&cache, &item, body));

    assert_eq!(
        spoken_as_it_opens(&document),
        "Meeting cancelled: Quarterly review. It is not on your calendar."
    );
    assert!(
        the_calendar_row(&document.attachments).contains(", meeting cancellation,"),
        "{}",
        the_calendar_row(&document.attachments)
    );
}

#[test]
fn test_the_formatted_window_and_the_preview_say_the_invitation_before_the_message() {
    // The formatted window puts the bar the composer answers above its page,
    // and the preview renders it into the page, before the title.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    let (item, body) = opened(&cache, 1, AN_INVITATION);
    let shown = asked(&cache, &item, body);
    let parts = [ConversationPart {
        message: item.clone(),
        body: shown.body.clone(),
        said: shown.said.clone(),
        depth: 0,
    }];

    let formatted = reader_text::conversation(&item.subject, &parts);
    let preview = reader_text::preview_html(&item.subject, &parts);

    assert_eq!(spoken_as_it_opens(&formatted), THE_INVITATION_SAID);
    let said_at = preview
        .find("Meeting invitation: Quarterly review")
        .expect("the preview says the invitation");
    let titled_at = preview.find("<h1>").expect("the preview has its title");
    assert!(said_at < titled_at, "{preview}");
    assert!(
        the_calendar_row(&reader_text::attachments_in(&parts)).contains(", meeting invitation,"),
        "the formatted window's list calls the part something else"
    );
}

#[test]
fn test_a_signed_invitation_says_the_meeting_above_the_account_of_the_signature() {
    // A signature verdict puts "More about this signature:" in the bar and the
    // reader speaks only what is above it, so the meeting is folded in first.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    let (item, body) = opened(&cache, 1, AN_INVITATION);
    let mut shown = asked(&cache, &item, body);
    shown.said.signature = SignatureCheck::NotKept;

    let document = in_the_text_reader(&item, &shown);

    assert!(
        document
            .warning
            .as_deref()
            .is_some_and(|bar| bar.contains("More about this signature:")),
        "the fixture did not produce the boundary this is about"
    );
    assert!(
        spoken_as_it_opens(&document).contains("Meeting invitation: Quarterly review"),
        "the meeting is below the signature's account, where nothing speaks it"
    );
}

// ── A message whose text the download of everything brought ──────────────

/// A meeting with its agenda attached: a file the download keeps the name of
/// and not the bytes, and the calendar part it keeps whole.
const A_MEETING_WITH_ITS_AGENDA: &str = "From: Ada Lovelace <ada@example.com>\r\n\
To: me@example.com\r\n\
Subject: Invitation: Quarterly review\r\n\
Message-ID: <invite-2@example.com>\r\n\
Date: Mon, 2 Mar 2026 10:00:00 +0000\r\n\
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
--mix\r\n\
Content-Type: text/calendar; charset=utf-8; method=REQUEST\r\n\
Content-Disposition: attachment; filename=\"invite.ics\"\r\n\
\r\n\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
METHOD:REQUEST\r\n\
BEGIN:VEVENT\r\n\
UID:m-2@example.com\r\n\
SUMMARY:Quarterly review\r\n\
DTSTART:20260305T090000\r\n\
DTEND:20260305T100000\r\n\
ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n\
ATTENDEE;CN=Me;PARTSTAT=NEEDS-ACTION:mailto:me@example.com\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n\
--mix--\r\n";

/// The row the download of everything leaves: the message and its text, and
/// nothing about its parts, because the reader will never fetch it again.
fn downloaded(cache: &MessageCache, uid: u32, raw: &str) -> (i64, mime::ParsedMessage) {
    let parsed = mime::parse(raw.as_bytes()).expect("the message to parse");
    let row = cache
        .save_message(&CachedMessage {
            id: 0,
            uid,
            folder_id: 1,
            message_id: parsed.message_id.clone().unwrap_or_default(),
            subject: parsed.subject.clone(),
            from_addr: THE_SENDER.to_string(),
            to_addr: "me@example.com".to_string(),
            cc: None,
            date: parsed.date.clone().unwrap_or_default(),
            body_plain: parsed.body_plain.clone(),
            body_html: parsed.body_html.clone(),
            read: false,
            starred: false,
            deleted: false,
            safety: wixen_mail::service::safety::Safety::Ordinary,
        })
        .expect("the row saved");
    (row, parsed)
}

#[test]
fn test_a_meeting_the_download_of_everything_brought_is_said_when_it_opens() {
    // The download fetches each whole message for its text, and the reader
    // fetches nothing for a message whose text is here. Before this, neither
    // stored the parts, so a meeting that came down that way was never said.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    let (row, parsed) = downloaded(&cache, 3, A_MEETING_WITH_ITS_AGENDA);

    reading_a_message::keep_what_a_download_carried(
        &cache,
        row,
        &parsed.attachments,
        A_MEETING_WITH_ITS_AGENDA.as_bytes(),
    )
    .expect("the parts kept");

    let said = reading_a_message::invitation_check_for(Some(&cache), row, written_out_in_full);
    assert!(
        said.said()
            .is_some_and(|said| said.starts_with("Meeting invitation: Quarterly review")),
        "{said:?}"
    );
}

#[test]
fn test_a_download_keeps_every_parts_name_and_only_the_calendar_parts_file() {
    // The names are what the reader lists; the calendar document is a few
    // kilobytes and is what the meeting is read from. Every other file stays
    // on the server until somebody opens it, as it did.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    let (row, parsed) = downloaded(&cache, 3, A_MEETING_WITH_ITS_AGENDA);

    reading_a_message::keep_what_a_download_carried(
        &cache,
        row,
        &parsed.attachments,
        A_MEETING_WITH_ITS_AGENDA.as_bytes(),
    )
    .expect("the parts kept");

    let kept = cache.attachments_with_content(row).expect("the parts read");
    let named: Vec<(&str, bool)> = kept
        .iter()
        .map(|part| (part.described.filename.as_str(), part.content.is_some()))
        .collect();
    assert_eq!(named, [("agenda.pdf", false), ("invite.ics", true)]);
}

#[test]
fn test_a_download_leaves_the_files_the_reader_kept_where_they_are() {
    // A message somebody opened has every file kept; its text going and
    // coming back through the download must not throw them away.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    let (item, _) = opened(&cache, 3, A_MEETING_WITH_ITS_AGENDA);
    let parsed = mime::parse(A_MEETING_WITH_ITS_AGENDA.as_bytes()).expect("parsed");

    reading_a_message::keep_what_a_download_carried(
        &cache,
        item.message_id,
        &parsed.attachments,
        A_MEETING_WITH_ITS_AGENDA.as_bytes(),
    )
    .expect("nothing refused");

    let kept = cache
        .attachments_with_content(item.message_id)
        .expect("the parts read");
    assert!(
        kept.iter().all(|part| part.content.is_some()),
        "a file the reader kept was dropped"
    );
}

/// The one function the download of everything stores a message through.
const THE_DOWNLOAD: &str = "src/application/mail_sync.rs";
const WHERE_IT_STORES: &str = "async fn fetch_and_store_one<";
const WHAT_IT_MUST_CALL: &str = "keep_what_a_download_carried(";

/// One top-level function's lines, from its opening to the `}` that closes it
/// in the first column, with comments cut off each line.
fn the_code_of<'a>(source: &'a str, opening: &str) -> Vec<&'a str> {
    source
        .lines()
        .skip_while(|line| !line.contains(opening))
        .take_while(|line| *line != "}")
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code))
        .collect()
}

#[test]
fn test_the_download_of_everything_keeps_what_the_message_carried() {
    // What makes the case above run: the text pass itself asks, rather than
    // a test calling the function on its behalf.
    let source = std::fs::read_to_string(THE_DOWNLOAD)
        .expect("the download to be readable")
        .replace("\r\n", "\n");
    let code = the_code_of(&source, WHERE_IT_STORES);

    assert!(
        !code.is_empty(),
        "{THE_DOWNLOAD} no longer has {WHERE_IT_STORES}"
    );
    assert!(
        code.iter().any(|line| line.contains(WHAT_IT_MUST_CALL)),
        "the text pass stores a message's text and never keeps its parts, so a \
         meeting it brought is never said"
    );
}

#[test]
fn test_the_reading_of_the_download_sees_a_call_that_is_missing_or_only_mentioned() {
    // The companion: the reading answers no for a body without the call and
    // for one that names it only in a comment.
    let without = "async fn fetch_and_store_one<M>() {\n    store_text();\n}\n";
    let mentioned =
        "async fn fetch_and_store_one<M>() {\n    // keep_what_a_download_carried(\n}\n";

    for source in [without, mentioned] {
        assert!(
            !the_code_of(source, WHERE_IT_STORES)
                .iter()
                .any(|line| line.contains(WHAT_IT_MUST_CALL)),
            "{source}"
        );
    }
}

#[test]
fn test_a_fold_that_put_the_meeting_after_the_signature_would_leave_it_unspoken() {
    // The companion, which says the case above can fail: the same answers
    // folded with the meeting after the signature put the sentence in the bar
    // and below the cut.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    let (item, body) = opened(&cache, 1, AN_INVITATION);
    let mut shown = asked(&cache, &item, body);
    shown.said.signature = SignatureCheck::NotKept;

    let folded_late = reader_text::single_message(&item, &shown.body, reading())
        .with_signature(&shown.said.signature)
        .with_invitation(&shown.said.invitation);

    assert!(
        folded_late
            .warning
            .as_deref()
            .is_some_and(|bar| bar.contains("Meeting invitation: Quarterly review")),
        "the late fold did not put the sentence in the bar at all"
    );
    assert!(
        !spoken_as_it_opens(&folded_late).contains("Meeting invitation"),
        "a sentence folded after the signature was still above the cut"
    );
}

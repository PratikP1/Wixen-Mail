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

// ── The readings over the window ───────────────────────────────────────────
//
// The stored half of the path is the cases above. What they cannot show is
// that selecting a row reaches it, that the fetch says nothing, and that the
// preview is not loaded twice for nothing, so these read the source that
// ships. Each has a companion below that plants the violation and is refused.

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";
const THE_DOWNLOAD: &str = "src/application/mail_sync.rs";

const THE_CURSOR_HANDLER: (&str, &str) =
    ("msg_list.on_item_focused({", "msg_list.on_column_click({");
const A_STORED_BODY: (&str, &str) = ("Some((id, body)) => {", "None => {");
const THE_PARTS_FETCH: &str = "fn spawn_parts_fetch(";
const THE_BODY_FETCH: &str = "fn spawn_body_fetch(";
const THE_WHOLE_MESSAGE: &str = "fn the_whole_message_from_its_own_account(";
const THE_ARM: (&str, &str) = (
    "UIUpdate::PartsKept { message_row_id } => {",
    "UIUpdate::ConnectionStatusChanged(",
);
const SHOWING_AGAIN: &str = "fn show_again_if_changed(";
const WHERE_THE_DOWNLOAD_STORES: &str = "async fn fetch_and_store_one<";

const ASKED: &str = "were_left_behind(";
const STARTED: &str = "spawn_parts_fetch(";
const KEPT: &str = "keep_every_part(";
const SENT: &str = "UIUpdate::PartsKept";
const THE_OPEN_ACCOUNT: &str = "active_account_id";
const CHOSEN: &str = "the_account_to_fetch_through(";
const COMPOSED: &str = "the_preview_of(";
const SHOWN_AGAIN: &str = "show_again_if_changed(";
const LOADED: &str = "show_the_message(";
const HELD: &str = "self.message.borrow()";

/// What the parts fetch must never do: announce the message a second time,
/// write its text, or say anything.
const NEVER_IN_THE_PARTS_FETCH: [&str; 3] = ["MessageBodyLoaded", "save_message_body(", "a11y."];
/// What the arm must never do: load regardless, say anything, or replace the
/// body a reply quotes.
const NEVER_IN_THE_ARM: [&str; 3] = [LOADED, "a11y.", "message_preview ="];

fn what_ships_in(path: &str) -> String {
    let source = std::fs::read_to_string(path)
        .unwrap_or_else(|why| panic!("{path}: {why}"))
        .replace("\r\n", "\n");
    wixen_mail::common::what_ships::what_ships(&source)
}

/// One top-level function's text, from its signature to the closing brace at
/// column nought, or a complaint when the signature is gone.
fn body_of(source: &str, signature: &str) -> Result<String, String> {
    one_block(source, signature, "\n}\n")
}

/// One method's text, from its signature to the closing brace one level in.
fn method_body_of(source: &str, signature: &str) -> Result<String, String> {
    one_block(source, signature, "\n    }\n")
}

fn one_block(source: &str, signature: &str, closing: &str) -> Result<String, String> {
    let at = source.find(signature).ok_or(format!(
        "{signature} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let ends = rest
        .find(closing)
        .map_or(rest.len(), |end| end + closing.len());
    Ok(rest[..ends].to_string())
}

/// The text between two anchors, or a complaint naming the one that is gone.
fn between<'a>(text: &'a str, from: &str, to: &str) -> Result<&'a str, String> {
    let start = text
        .find(from)
        .ok_or(format!("{from:?} is no longer here, so this reads nothing"))?
        + from.len();
    let rest = &text[start..];
    let end = rest
        .find(to)
        .ok_or(format!("{to:?} is no longer here, so this reads nothing"))?;
    Ok(&rest[..end])
}

/// Whether `first` is in the text before `second`, or a complaint naming
/// which is missing.
fn comes_before(text: &str, first: &str, second: &str) -> Result<bool, String> {
    let a = text.find(first).ok_or(format!(
        "{first:?} is no longer here, so this reads nothing"
    ))?;
    let b = text.find(second).ok_or(format!(
        "{second:?} is no longer here, so this reads nothing"
    ))?;
    Ok(a < b)
}

fn reaches(text: &str, whose: &str, needed: &[&str]) -> Result<(), String> {
    match needed.iter().find(|needed| !text.contains(**needed)) {
        Some(missing) => Err(format!("{whose} does not reach {missing}")),
        None => Ok(()),
    }
}

fn holds_none_of(text: &str, whose: &str, forbidden: &[&str]) -> Result<(), String> {
    match forbidden
        .iter()
        .find(|forbidden| text.contains(**forbidden))
    {
        Some(found) => Err(format!("{whose} holds {found}")),
        None => Ok(()),
    }
}

/// Selecting a message whose text is here asks whether its parts were left
/// behind, and starts their fetch when they were, in the arm that sends the
/// stored body; the arm for a body not here fetches the whole message
/// already.
fn the_cursor_handler_fetches_the_parts_left_behind(app: &str) -> Result<(), String> {
    let handler = between(app, THE_CURSOR_HANDLER.0, THE_CURSOR_HANDLER.1)?;
    let stored = between(handler, A_STORED_BODY.0, A_STORED_BODY.1)?;
    if comes_before(stored, ASKED, STARTED)? {
        Ok(())
    } else {
        Err(format!(
            "the cursor handler starts {STARTED} before it asks {ASKED}, so every message \
             with its text here is fetched again"
        ))
    }
}

/// The parts fetch keeps what it fetched and tells the preview only through
/// its own update, and never announces, stores text or speaks.
fn the_parts_fetch_keeps_them_and_says_nothing(app: &str) -> Result<(), String> {
    let fetch = body_of(app, THE_PARTS_FETCH)?;
    reaches(&fetch, THE_PARTS_FETCH, &[KEPT, SENT])?;
    holds_none_of(&fetch, THE_PARTS_FETCH, &NEVER_IN_THE_PARTS_FETCH)
}

/// Both fetches on selection take the account from the message's own
/// folder, through the one helper, and neither reads the account last
/// opened.
fn both_fetches_go_through_the_messages_own_account(app: &str) -> Result<(), String> {
    let helper = THE_WHOLE_MESSAGE.trim_start_matches("fn ");
    for fetch in [THE_BODY_FETCH, THE_PARTS_FETCH] {
        let body = body_of(app, fetch)?;
        holds_none_of(&body, fetch, &[THE_OPEN_ACCOUNT])?;
        reaches(&body, fetch, &[helper])?;
    }
    reaches(
        &body_of(app, THE_WHOLE_MESSAGE)?,
        THE_WHOLE_MESSAGE,
        &[CHOSEN],
    )
}

/// The arm composes the preview again and hands it to the one method that
/// loads only when the page changed; it never loads directly, speaks, or
/// replaces the body a reply quotes.
fn the_arm_loads_the_preview_again_only_when_it_changed(app: &str) -> Result<(), String> {
    let arm = between(app, THE_ARM.0, THE_ARM.1)?;
    reaches(arm, THE_ARM.0, &[COMPOSED, SHOWN_AGAIN])?;
    holds_none_of(arm, THE_ARM.0, &NEVER_IN_THE_ARM)
}

/// Showing again compares with the page held before it loads anything.
fn showing_again_reads_the_page_it_holds_first(app: &str) -> Result<(), String> {
    let method = method_body_of(app, SHOWING_AGAIN)?;
    if comes_before(&method, HELD, LOADED)? {
        Ok(())
    } else {
        Err(format!(
            "{SHOWING_AGAIN} loads before it reads the page it holds, so the preview is \
             loaded again whether or not anything changed"
        ))
    }
}

/// The download of everything keeps names and calendar documents only
/// (Pratik, 2026-09-26: the 512 MB budget), never every file.
fn the_download_never_keeps_every_file(sync: &str) -> Result<(), String> {
    let download = body_of(sync, WHERE_THE_DOWNLOAD_STORES)?;
    holds_none_of(&download, WHERE_THE_DOWNLOAD_STORES, &[KEPT])
}

#[test]
fn test_selecting_a_message_whose_parts_were_left_behind_starts_their_fetch() {
    the_cursor_handler_fetches_the_parts_left_behind(&what_ships_in(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_parts_fetch_keeps_them_and_says_nothing() {
    the_parts_fetch_keeps_them_and_says_nothing(&what_ships_in(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_both_fetches_go_through_the_messages_own_account() {
    both_fetches_go_through_the_messages_own_account(&what_ships_in(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_parts_kept_arm_loads_the_preview_again_only_when_it_changed() {
    the_arm_loads_the_preview_again_only_when_it_changed(&what_ships_in(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_showing_again_reads_the_page_it_holds_before_loading() {
    showing_again_reads_the_page_it_holds_first(&what_ships_in(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_download_of_everything_never_keeps_every_file() {
    the_download_never_keeps_every_file(&what_ships_in(THE_DOWNLOAD))
        .unwrap_or_else(|why| panic!("{why}"));
}

// ── The companions ─────────────────────────────────────────────────────────

/// A window shaped as it should be, so a reading that stopped finding its
/// anchors cannot pass by finding nothing.
const A_WINDOW_AS_IT_SHOULD_BE: &str = "            msg_list.on_item_focused({
                move |event| {
                    match body {
                        Some((id, body)) => {
                            let _ = ui_tx.try_send(UIUpdate::MessageBodyLoaded(body));
                            if parts_left_behind::were_left_behind(body_cache.as_deref(), id) {
                                spawn_parts_fetch(app, id, uid);
                            }
                        }
                        None => {
                            spawn_body_fetch(app, id, uid);
                        }
                    }
                }
            });
            msg_list.on_column_click({
            });

impl PageHost {
    fn show_again_if_changed(&self, html: &str, before_the_load: impl FnOnce()) -> bool {
        if *self.message.borrow() == html {
            return false;
        }
        before_the_load();
        self.show_the_message(html);
        true
    }
}

fn the_whole_message_from_its_own_account(cache: &MessageCache) -> Option<Vec<u8>> {
    let account = parts_left_behind::the_account_to_fetch_through(accounts, filed_under)?;
    None
}

fn spawn_body_fetch(app: AppHandles<'_>, message_row_id: i64, uid: u32) {
    let raw = the_whole_message_from_its_own_account(&cache);
    let _ = tx.send(UIUpdate::MessageBodyLoaded(body));
}

fn spawn_parts_fetch(app: AppHandles<'_>, message_row_id: i64, uid: u32) {
    let raw = the_whole_message_from_its_own_account(&cache);
    let kept = parts_left_behind::keep_every_part(&cache, message_row_id, &parts, &raw);
    let _ = tx.send(UIUpdate::PartsKept { message_row_id });
}

fn handle_update() {
    match update {
        UIUpdate::PartsKept { message_row_id } => {
            let html = the_preview_of(message_cache, showing, &body);
            preview.show_again_if_changed(&html, || {});
        }
        UIUpdate::ConnectionStatusChanged(status) => {}
    }
}
";

const A_DOWNLOAD_AS_IT_SHOULD_BE: &str = "async fn fetch_and_store_one<M: Mailbox>(mailbox: &M) {
    reading_a_message::keep_what_a_download_carried(cache, row, &parts, &raw);
}
";

fn every_reading_over(app: &str, sync: &str) -> Result<(), String> {
    the_cursor_handler_fetches_the_parts_left_behind(app)?;
    the_parts_fetch_keeps_them_and_says_nothing(app)?;
    both_fetches_go_through_the_messages_own_account(app)?;
    the_arm_loads_the_preview_again_only_when_it_changed(app)?;
    showing_again_reads_the_page_it_holds_first(app)?;
    the_download_never_keeps_every_file(sync)
}

/// The window with one line replaced, refusing a plant that finds nothing to
/// replace so a companion cannot pass over an unchanged window.
fn planted(original: &str, what: &str, with: &str) -> String {
    assert!(original.contains(what), "nothing to plant over: {what:?}");
    original.replacen(what, with, 1)
}

#[test]
fn test_the_readings_pass_a_window_shaped_as_it_should_be() {
    every_reading_over(A_WINDOW_AS_IT_SHOULD_BE, A_DOWNLOAD_AS_IT_SHOULD_BE)
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_readings_complain_when_the_fetch_is_not_started_or_speaks() {
    let not_started = planted(
        A_WINDOW_AS_IT_SHOULD_BE,
        "                                spawn_parts_fetch(app, id, uid);\n",
        "",
    );
    let why = the_cursor_handler_fetches_the_parts_left_behind(&not_started)
        .expect_err("selecting starts no fetch");
    assert!(
        why.contains("\"spawn_parts_fetch(\" is no longer here"),
        "{why}"
    );

    let started_unasked = planted(
        A_WINDOW_AS_IT_SHOULD_BE,
        "                            let _ = ui_tx.try_send(UIUpdate::MessageBodyLoaded(body));\n",
        "                            spawn_parts_fetch(app, id, uid);\n",
    );
    let why = the_cursor_handler_fetches_the_parts_left_behind(&started_unasked)
        .expect_err("the fetch started before the question");
    assert!(why.contains("before it asks were_left_behind("), "{why}");

    let speaks = planted(
        A_WINDOW_AS_IT_SHOULD_BE,
        "    let _ = tx.send(UIUpdate::PartsKept { message_row_id });\n",
        "    let _ = tx.send(UIUpdate::PartsKept { message_row_id });\n    let _ = a11y.announce(said);\n",
    );
    let why =
        the_parts_fetch_keeps_them_and_says_nothing(&speaks).expect_err("a fetch that speaks");
    assert!(why.contains("holds a11y."), "{why}");

    let announced_twice = planted(
        A_WINDOW_AS_IT_SHOULD_BE,
        "    let _ = tx.send(UIUpdate::PartsKept { message_row_id });\n",
        "    let _ = tx.send(UIUpdate::PartsKept { message_row_id });\n    let _ = tx.send(UIUpdate::MessageBodyLoaded(body));\n",
    );
    let why = the_parts_fetch_keeps_them_and_says_nothing(&announced_twice)
        .expect_err("the message announced a second time");
    assert!(why.contains("holds MessageBodyLoaded"), "{why}");

    let the_download_keeps_everything = planted(
        A_DOWNLOAD_AS_IT_SHOULD_BE,
        "reading_a_message::keep_what_a_download_carried(",
        "parts_left_behind::keep_every_part(",
    );
    let why = the_download_never_keeps_every_file(&the_download_keeps_everything)
        .expect_err("the download keeping every file");
    assert!(why.contains("holds keep_every_part("), "{why}");
}

#[test]
fn test_the_readings_complain_when_a_fetch_goes_through_another_account() {
    let the_open_account = planted(
        A_WINDOW_AS_IT_SHOULD_BE,
        "fn spawn_body_fetch(app: AppHandles<'_>, message_row_id: i64, uid: u32) {\n",
        "fn spawn_body_fetch(app: AppHandles<'_>, message_row_id: i64, uid: u32) {\n    let account_id = s.active_account_id.clone();\n",
    );
    let why = both_fetches_go_through_the_messages_own_account(&the_open_account)
        .expect_err("the body fetch through the account last opened");
    assert!(
        why.contains("fn spawn_body_fetch( holds active_account_id"),
        "{why}"
    );

    let the_first_account = planted(
        A_WINDOW_AS_IT_SHOULD_BE,
        "parts_left_behind::the_account_to_fetch_through(accounts, filed_under)?",
        "accounts.first().cloned()?",
    );
    let why = both_fetches_go_through_the_messages_own_account(&the_first_account)
        .expect_err("the helper taking the first account");
    assert!(
        why.contains("does not reach the_account_to_fetch_through("),
        "{why}"
    );
}

#[test]
fn test_the_readings_complain_when_the_preview_is_loaded_regardless() {
    let loaded_directly = planted(
        A_WINDOW_AS_IT_SHOULD_BE,
        "            preview.show_again_if_changed(&html, || {});\n",
        "            preview.show_the_message(&html);\n",
    );
    let why = the_arm_loads_the_preview_again_only_when_it_changed(&loaded_directly)
        .expect_err("the arm loading whatever changed");
    assert!(
        why.contains("does not reach show_again_if_changed("),
        "{why}"
    );

    let loaded_and_kept_quiet = planted(
        A_WINDOW_AS_IT_SHOULD_BE,
        "            preview.show_again_if_changed(&html, || {});\n",
        "            preview.show_again_if_changed(&html, || {});\n            preview.show_the_message(&html);\n",
    );
    let why = the_arm_loads_the_preview_again_only_when_it_changed(&loaded_and_kept_quiet)
        .expect_err("the arm loading beside the method");
    assert!(why.contains("holds show_the_message("), "{why}");

    let loads_first = planted(
        A_WINDOW_AS_IT_SHOULD_BE,
        "        if *self.message.borrow() == html {\n",
        "        self.show_the_message(html);\n        if *self.message.borrow() == html {\n",
    );
    let why = showing_again_reads_the_page_it_holds_first(&loads_first)
        .expect_err("loaded before compared");
    assert!(
        why.contains("loads before it reads the page it holds"),
        "{why}"
    );
}

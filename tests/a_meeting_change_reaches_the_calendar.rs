//! An organiser's update moves the meeting when the message is opened in a
//! reader window, and a cancellation offers Remove from Calendar (GAP-04, #50
//! points 2 and 3, 13-13).
//!
//! Two halves. The first starts from the bytes, the way
//! `an_invitation_is_said_before_the_body.rs` does: a raw message parsed, its
//! parts stored the way opening a message stores them, the calendar holding
//! the meeting with Ada recorded as its organiser, and then the reader's own
//! application call, `what_opening_it_in_a_reader_changed`, followed by the
//! composition every surface asks. The calendar row is read back after each.
//!
//! The second builds the real reader window and the formatted window, opens a
//! cancellation from the organiser in each, and reads the one button over
//! MSAA at its own handle, the way NVDA reads a native control: its name,
//! description, role and state, and where Tab reaches it. Alt+R is posted to
//! the message's text through the window's own loop and the presses counted,
//! as 13-11's target counts Alt+C.
//!
//! Source readings cover the rest: the page's Alt+R reaching the button's
//! handler in the formatted window, the preview never applying anything, and
//! the handler saying its outcome once.
//!
//! # What this cannot see
//!
//! No real organiser's update or cancellation has been through this, and a
//! change pushed to Google or Microsoft after they applied the same update
//! themselves is untried. Both are phase 14's, on the ledger. How the
//! sentences and the button sound is the tester's ear.

#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::{OsStr, c_void};
use std::fmt;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::AsRawHandle;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::allowed::Allowed;
use wixen_mail::application::meeting_changes::{MeetingChange, Removal};
use wixen_mail::application::reading_a_message::{self, AnsweringAs, WhatIsSaidAboutIt};
use wixen_mail::common::types::MessageBody;
use wixen_mail::data::message_cache::attachment_content::AttachmentWithContent;
use wixen_mail::data::message_cache::{
    CachedFolder, CachedMessage, CalendarEventEntry, MessageCache,
};
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::date_display::{
    Clock, DateOrder, DateSettings, DateStyle, DateWording,
};
use wixen_mail::presentation::read_aloud::Reading;
use wixen_mail::presentation::reader_text::{self, ConversationPart, ReaderDocument};
use wixen_mail::presentation::ui_types::MessageItem;
use wixen_mail::presentation::wx_app;
use wixen_mail::presentation::wx_reader::ReaderWindow;
use wixen_mail::service::mime;
use wxdragon::prelude::*;

// ── The messages ──────────────────────────────────────────────────────────

const ADA: &str = "Ada Lovelace <ada@example.com>";
const GRACE: &str = "Grace Hopper <grace@example.com>";

/// Version 3 of the meeting, moved from Thursday at nine to Friday at two, as
/// a calendar server sends an update: a covering note and the meeting as a
/// second alternative. At an hour on the clock, so what is said does not
/// depend on the zone the test runs in.
fn an_update_from(sender: &str) -> String {
    format!(
        "From: {sender}\r\n\
To: sam@example.com\r\n\
Subject: Updated invitation: Quarterly review\r\n\
Message-ID: <update-1@example.com>\r\n\
Date: Tue, 3 Mar 2026 10:00:00 +0000\r\n\
MIME-Version: 1.0\r\n\
Content-Type: multipart/alternative; boundary=\"alt\"\r\n\
\r\n\
--alt\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
The quarterly review has moved to Friday.\r\n\
--alt\r\n\
Content-Type: text/calendar; charset=utf-8; method=REQUEST\r\n\
\r\n\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example//EN\r\n\
METHOD:REQUEST\r\n\
BEGIN:VEVENT\r\n\
UID:m-1@example.com\r\n\
SEQUENCE:3\r\n\
SUMMARY:Quarterly review\r\n\
LOCATION:Room 3\r\n\
DTSTART:20260306T140000\r\n\
DTEND:20260306T150000\r\n\
ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n\
ATTENDEE;CN=Sam;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:sam@example.com\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n\
--alt--\r\n"
    )
}

/// The same meeting called off by whoever `sender` is.
fn a_cancellation_from(sender: &str) -> String {
    an_update_from(sender)
        .replace("method=REQUEST", "method=CANCEL")
        .replace("METHOD:REQUEST", "METHOD:CANCEL")
        .replace("Updated invitation", "Cancelled")
}

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

/// Sam, whom the invitation names, with changes allowed.
fn sam(_account: &str) -> AnsweringAs {
    AnsweringAs {
        address: "sam@example.com".to_string(),
        allowed: Allowed::EVERYTHING,
    }
}

// ── The store ─────────────────────────────────────────────────────────────

/// A store with one account's inbox, and the meeting on that account's
/// calendar at Thursday nine to ten: accepted here at version 2, with Ada
/// recorded as who called it, the way 13-12's answer path leaves it.
fn a_store_holding_the_meeting(dir: &tempfile::TempDir) -> MessageCache {
    a_store_holding(dir, &the_meeting())
}

/// The same store, holding `copy` as the calendar's copy of the meeting.
fn a_store_holding(dir: &tempfile::TempDir, copy: &CalendarEventEntry) -> MessageCache {
    let cache = MessageCache::new(dir.path().join("meetings.db"), None).expect("a store");
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
    cache.save_calendar_event(copy).expect("the meeting filed");
    cache
        .remember_the_answer(
            "evt-1",
            2,
            wixen_mail::application::invitations::Answer::Accepted,
        )
        .expect("the answer remembered");
    cache
        .remember_where_it_came_from("evt-1", Some("m-1@example.com"), Some("ada@example.com"))
        .expect("the organiser remembered");
    cache
}

fn the_meeting() -> CalendarEventEntry {
    CalendarEventEntry {
        id: "evt-1".to_string(),
        account_id: "acc-1".to_string(),
        provider_event_id: Some("m-1@example.com".to_string()),
        calendar_id: Some("cal".to_string()),
        summary: "Quarterly review".to_string(),
        description: None,
        location: Some("Room 3".to_string()),
        start_datetime: "2026-03-05T09:00:00".to_string(),
        end_datetime: "2026-03-05T10:00:00".to_string(),
        start_date: None,
        end_date: None,
        is_all_day: false,
        time_zone: None,
        status: "confirmed".to_string(),
        recurrence_rule: None,
        categories: String::new(),
        source_provider: None,
        etag: None,
        web_link: None,
        show_as: "busy".to_string(),
        last_modified_remote: None,
        last_synced_at: None,
        attendees_json: None,
        reminders_json: None,
        created_at: "2026-03-01T00:00:00Z".to_string(),
        updated_at: "2026-03-01T00:00:00Z".to_string(),
        pending: false,
        exception_dates: None,
        cut_from_event_id: None,
        provider_recurrence_id: None,
    }
}

/// The meeting as the calendar holds it now.
fn the_meeting_now(cache: &MessageCache) -> CalendarEventEntry {
    cache
        .get_event_by_id("evt-1")
        .expect("the calendar to be readable")
        .expect("the meeting to be on the calendar")
}

/// A raw message opened: parsed, its row saved, its parts stored with their
/// bytes the way the reader's open path stores them.
fn opened(cache: &MessageCache, uid: u32, raw: &str, sender: &str) -> (MessageItem, MessageBody) {
    let parsed = mime::parse(raw.as_bytes()).expect("the message to parse");
    let row = cache
        .save_message(&CachedMessage {
            id: 0,
            uid,
            folder_id: 1,
            message_id: parsed.message_id.clone().unwrap_or_default(),
            subject: parsed.subject.clone(),
            from_addr: sender.to_string(),
            to_addr: "sam@example.com".to_string(),
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
        from: sender.to_string(),
        ..Default::default()
    };
    let body = MessageBody::Plain(parsed.body_plain.unwrap_or_default());
    (item, body)
}

/// Opened in a reader window: what opening it changes is asked and applied
/// first, then the message is composed the way every surface composes one,
/// with the change folded in, which is the order the reader's open path takes.
fn opened_in_a_reader(
    cache: &MessageCache,
    item: &MessageItem,
    body: MessageBody,
) -> ReaderDocument {
    let change = reading_a_message::what_opening_it_in_a_reader_changed(
        Some(cache),
        item.message_id,
        &item.from,
        written_out_in_full,
        sam,
    );
    let shown = reading_a_message::for_message(
        Some(cache),
        item.message_id,
        &item.from,
        body,
        written_out_in_full,
        sam,
    );
    reader_text::single_message(item, &shown.body, reading()).with_what_is_said(
        &WhatIsSaidAboutIt {
            change,
            ..shown.said
        },
    )
}

/// The lines of the bar spoken as the message opens.
fn spoken_as_it_opens(document: &ReaderDocument) -> Vec<String> {
    reader_text::said_before_the_message(
        document
            .warning
            .as_deref()
            .expect("a message carrying a meeting has a bar"),
    )
    .lines()
    .map(str::to_string)
    .collect()
}

#[test]
fn test_an_update_from_the_organiser_opened_in_a_reader_moves_the_meeting_and_says_so() {
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store_holding_the_meeting(&dir);
    let (item, body) = opened(&cache, 1, &an_update_from(ADA), ADA);

    let document = opened_in_a_reader(&cache, &item, body);

    let moved = the_meeting_now(&cache);
    assert_eq!(moved.start_datetime, "2026-03-06T14:00:00");
    assert_eq!(moved.end_datetime, "2026-03-06T15:00:00");
    assert!(
        moved.pending,
        "a move nobody sends never reaches the provider"
    );
    let spoken = spoken_as_it_opens(&document);
    assert!(
        spoken.iter().any(|line| line
            == "Moved on your calendar from 05/03/2026 at 09:00 to 06/03/2026 at 14:00."),
        "{spoken:?}"
    );
    // Said against the calendar as it now is, so the meeting's own sentence
    // does not describe the move as one still to come.
    assert!(
        spoken
            .iter()
            .any(|line| line.ends_with(", and it is already on your calendar.")),
        "{spoken:?}"
    );
}

/// The instant a stored time names in the zone stored beside it, in
/// universal time.
fn in_universal_time(stored: &str, zone: Option<&str>) -> String {
    use wixen_mail::common::moment::{self, Moment};
    match moment::read_in(stored, zone) {
        Some(Moment::Fixed(at)) => at.with_timezone(&chrono::Utc).to_rfc3339(),
        other => panic!("{stored} beside {zone:?} names no instant: {other:?}"),
    }
}

#[test]
fn test_a_move_keeps_the_zone_the_update_was_written_in() {
    // The calendar's copy as Microsoft Graph stores one, a clock face in
    // universal time with "UTC" beside it, and Ada's update written for ten
    // in Los Angeles, an hour later. Stored as ten beside "UTC", the move
    // would put the meeting eight hours early here and send that hour back
    // (13-21.1, premise 9). America/Los_Angeles rather than Windows' name, so
    // the instant does not depend on Windows' ICU.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let graphs_copy = CalendarEventEntry {
        start_datetime: "2026-03-05T17:00:00.0000000".to_string(),
        end_datetime: "2026-03-05T18:00:00.0000000".to_string(),
        time_zone: Some("UTC".to_string()),
        ..the_meeting()
    };
    let cache = a_store_holding(&dir, &graphs_copy);
    let update = an_update_from(ADA)
        .replace(
            "DTSTART:20260306T140000",
            "DTSTART;TZID=America/Los_Angeles:20260305T100000",
        )
        .replace(
            "DTEND:20260306T150000",
            "DTEND;TZID=America/Los_Angeles:20260305T110000",
        );
    let (item, body) = opened(&cache, 1, &update, ADA);

    let document = opened_in_a_reader(&cache, &item, body);

    let spoken = spoken_as_it_opens(&document);
    assert!(
        spoken
            .iter()
            .any(|line| line.starts_with("Moved on your calendar")),
        "{spoken:?}"
    );
    let moved = the_meeting_now(&cache);
    let zone = moved.time_zone.as_deref();
    assert_eq!(
        (zone, moved.start_datetime.as_str()),
        (Some("America/Los_Angeles"), "2026-03-05T10:00:00"),
        "premise 9: the move keeps the copy's zone beside the update's clock face"
    );
    assert_eq!(
        (
            in_universal_time(&moved.start_datetime, zone),
            in_universal_time(&moved.end_datetime, zone)
        ),
        (
            "2026-03-05T18:00:00+00:00".to_string(),
            "2026-03-05T19:00:00+00:00".to_string()
        ),
        "premise 9: the moved meeting is not at the instant the organiser wrote"
    );
}

#[test]
fn test_the_same_update_opened_again_moves_nothing_and_says_nothing_more() {
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store_holding_the_meeting(&dir);
    let (item, body) = opened(&cache, 1, &an_update_from(ADA), ADA);
    opened_in_a_reader(&cache, &item, body.clone());

    let again = opened_in_a_reader(&cache, &item, body);

    let spoken = spoken_as_it_opens(&again);
    assert!(
        !spoken
            .iter()
            .any(|line| line.starts_with("Moved on your calendar")),
        "{spoken:?}"
    );
}

#[test]
fn test_an_update_from_somebody_else_moves_nothing_and_says_why() {
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store_holding_the_meeting(&dir);
    let (item, body) = opened(&cache, 1, &an_update_from(GRACE), GRACE);

    let document = opened_in_a_reader(&cache, &item, body);

    let held = the_meeting_now(&cache);
    assert_eq!(
        (
            held.start_datetime.as_str(),
            held.end_datetime.as_str(),
            held.pending
        ),
        ("2026-03-05T09:00:00", "2026-03-05T10:00:00", false),
        "a stranger's update moved the meeting"
    );
    let spoken = spoken_as_it_opens(&document);
    assert!(
        spoken.iter().any(|line| line
            == "This says the meeting changed, and it comes from grace@example.com rather than \
                the organiser, ada@example.com, so your calendar was not changed."),
        "{spoken:?}"
    );
}

#[test]
fn test_the_preview_says_the_invitation_and_changes_nothing() {
    // The preview opens a message as the cursor moves over it, and a calendar
    // change made by arrowing past a message is a change nobody asked for. So
    // it asks what every surface asks and nothing more.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store_holding_the_meeting(&dir);
    let (item, body) = opened(&cache, 1, &an_update_from(ADA), ADA);

    let shown = reading_a_message::for_message(
        Some(&cache),
        item.message_id,
        &item.from,
        body,
        written_out_in_full,
        sam,
    );
    let page = reader_text::preview_html(
        &item.subject,
        &[ConversationPart {
            message: item.clone(),
            body: shown.body,
            said: shown.said,
            depth: 0,
        }],
    );

    assert_eq!(
        the_meeting_now(&cache).start_datetime,
        "2026-03-05T09:00:00"
    );
    assert!(
        page.contains("a change to the meeting on your calendar, which was 05/03/2026 at 09:00"),
        "{page}"
    );
    assert!(!page.contains("Moved on your calendar"), "{page}");
}

#[test]
fn test_a_cancellation_from_the_organiser_is_offered_for_removal_and_nothing_is_marked_yet() {
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store_holding_the_meeting(&dir);
    let (item, body) = opened(&cache, 1, &a_cancellation_from(ADA), ADA);

    let document = opened_in_a_reader(&cache, &item, body);

    assert_eq!(document.removal, Some(the_meeting_removed()));
    assert_eq!(document.answering, None);
    assert_eq!(the_meeting_now(&cache).status, "confirmed");
    let spoken = spoken_as_it_opens(&document);
    assert!(
        spoken.iter().any(|line| line
            == "The organiser has called this meeting off. Remove from Calendar takes it off \
                yours."),
        "{spoken:?}"
    );
}

#[test]
fn test_a_cancellation_from_somebody_else_offers_nothing() {
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store_holding_the_meeting(&dir);
    let (item, body) = opened(&cache, 1, &a_cancellation_from(GRACE), GRACE);

    let document = opened_in_a_reader(&cache, &item, body);

    assert_eq!(document.removal, None);
    assert_eq!(the_meeting_now(&cache).status, "confirmed");
}

#[test]
fn test_removing_leaves_the_meeting_cancelled_free_present_and_waiting_to_be_sent() {
    // Marked, never deleted: a provider's row taken away on a message's word
    // would be gone at the provider too on the next push.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store_holding_the_meeting(&dir);

    let marked = cache
        .mark_the_meeting_called_off("evt-1")
        .expect("the calendar to be written");

    assert!(marked);
    let removed = the_meeting_now(&cache);
    assert_eq!(removed.status, "cancelled");
    assert_eq!(removed.show_as, "free");
    assert!(
        removed.pending,
        "a removal nobody sends never reaches the provider"
    );
    assert_eq!(removed.summary, "Quarterly review");
    assert!(
        cache
            .deleted_calendar_events("acc-1")
            .expect("the deletions to be readable")
            .is_empty(),
        "the meeting was noted as deleted"
    );
}

// ── The windows ───────────────────────────────────────────────────────────

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const VT_I4: u16 = 3;
const CHILDID_SELF: i64 = 0;

/// MSAA roles and states (oleacc.h).
const ROLE_SYSTEM_PUSHBUTTON: i64 = 0x2b;
const STATE_SYSTEM_UNAVAILABLE: i64 = 0x1;

const WM_SYSKEYDOWN: u32 = 0x0104;
const WM_SYSKEYUP: u32 = 0x0105;
const VK_R: usize = 0x52;
/// winuser.h: Alt.
const VK_MENU: usize = 0x12;
/// The context code, bit 29, which is what makes a key an Alt chord.
const ALT_HELD: isize = 0x2000_0001;
/// The same with the previous-state and transition bits a release carries.
const ALT_RELEASED: isize = 0xE000_0001_u32 as i32 as isize;

const TICK_MS: i32 = 30;
/// Ticks for the loop to translate the key, hand it to the dialog manager and
/// run the button's handler.
const TICKS_FOR_THE_KEY: u32 = 10;

#[repr(C)]
#[derive(Clone, Copy)]
struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

/// {618736E0-3C3D-11CF-810C-00AA00389B71}
const IID_IACCESSIBLE: Guid = Guid {
    data1: 0x618736E0,
    data2: 0x3C3D,
    data3: 0x11CF,
    data4: [0x81, 0x0C, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71],
};

/// A VARIANT as the 64-bit ABI lays it out: 24 bytes, the type at offset 0
/// and the payload at offset 8.
#[repr(C)]
#[derive(Clone, Copy)]
struct Variant {
    vt: u16,
    reserved1: u16,
    reserved2: u16,
    reserved3: u16,
    val: i64,
    extra: u64,
}

impl Variant {
    fn child(id: i64) -> Self {
        Variant {
            vt: VT_I4,
            reserved1: 0,
            reserved2: 0,
            reserved3: 0,
            val: id,
            extra: 0,
        }
    }

    fn empty() -> Self {
        Variant {
            vt: 0,
            reserved1: 0,
            reserved2: 0,
            reserved3: 0,
            val: 0,
            extra: 0,
        }
    }
}

type Hresult = i32;
type ReleaseFn = unsafe extern "system" fn(*mut c_void) -> u32;
type GetVariantFn = unsafe extern "system" fn(*mut c_void, Variant, *mut Variant) -> Hresult;
type GetBstrFn = unsafe extern "system" fn(*mut c_void, Variant, *mut *mut u16) -> Hresult;

// IAccessible's vtable: IUnknown (3), IDispatch (4), then get_accParent (7),
// get_accChildCount (8), get_accChild (9), get_accName (10), get_accValue
// (11), get_accDescription (12), get_accRole (13), get_accState (14).
const VTBL_RELEASE: usize = 2;
const VTBL_GET_ACC_NAME: usize = 10;
const VTBL_GET_ACC_DESCRIPTION: usize = 12;
const VTBL_GET_ACC_ROLE: usize = 13;
const VTBL_GET_ACC_STATE: usize = 14;

#[link(name = "user32")]
unsafe extern "system" {
    fn EnumChildWindows(
        parent: isize,
        callback: extern "system" fn(isize, isize) -> i32,
        lparam: isize,
    ) -> i32;
    fn GetClassNameW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
    fn GetWindowTextW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
    fn PostMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> i32;
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
    fn SetFocus(hwnd: isize) -> isize;
    fn GetDlgCtrlID(hwnd: isize) -> i32;
    fn EnumThreadWindows(
        thread: u32,
        callback: extern "system" fn(isize, isize) -> i32,
        lparam: isize,
    ) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentThreadId() -> u32;
}

/// winuser.h: Shift, Control and Alt, each with its left and right key.
const MODIFIERS: [usize; 9] = [0x10, 0x11, 0x12, 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5];

/// Set this thread's keyboard state with exactly `down` of the modifier keys
/// held and every other key as it was.
///
/// The dialog manager reads a key's modifiers from the keyboard state, and a
/// desktop of the run's own keeps out the keys a person presses but not the
/// ones they hold: measured 2026-10-02 by 13-44.6.2, every one of eight failed
/// runs of two key targets came while the person at the machine held Shift.
/// The key here is posted and read when the loop takes it, so the state is
/// set before it is posted and cleared once it has been read.
fn only_these_modifiers_down(down: &[usize]) {
    let mut held = [0u8; 256];
    // SAFETY: the buffer is the 256 bytes the call writes.
    unsafe { GetKeyboardState(held.as_mut_ptr()) };
    for key in MODIFIERS {
        held[key] = 0;
    }
    for &key in down {
        held[key] = 0x80;
    }
    // SAFETY: the buffer is the 256 bytes the call reads, for this thread.
    unsafe { SetKeyboardState(held.as_ptr()) };
}

// ── The child on a desktop of its own ─────────────────────────────────────
//
// The formatted window holds a browser, and a browser's process starts on its
// process's desktop, not on the desktop of the thread that asks for it, so
// this reading cannot move its window thread alone: measured 2026-10-02, a
// copy with its thread on a desktop of its own took 36 s, waiting out a
// browser that never came up. So the window tests run in a child of this same
// executable started on a desktop made for the run, where nothing a person
// types arrives, inside the one turn, and the parent passes each only on the
// child's own `ok` line for it. Started whole that way it passed 20 of 20.
// The helper is a copy of the one in
// `tests/a_marker_counts_at_the_start_of_any_line.rs`, where its sentence
// cases live.

/// Set in the child's environment to the desktop it runs on; a window test
/// that finds it set runs its body.
const ON_A_DESKTOP_OF_THEIR_OWN: &str = "WIXEN_TESTS_ON_A_DESKTOP_OF_THEIR_OWN";

/// How long the child may run before it is stopped.
const THE_CHILDS_BOUND_MS: u32 = 5 * 60 * 1000;

/// The turn every run whose window holds a browser takes around its child,
/// and that the real paste target takes at the logon's test clipboard: one
/// kernel object for both, since neither may overlap a second run.
const THE_TURN: &str = "Local\\wixen-mail-tests-one-turn-at-what-runs-share";

/// How long a parent waits for the turn: the bound a run holding it is
/// stopped at, and a minute more.
const LONGEST_WAIT_FOR_THE_TURN_MS: u32 = THE_CHILDS_BOUND_MS + 60_000;

const WAIT_TIMEOUT: u32 = 0x0000_0102;
const WAIT_FAILED: u32 = 0xFFFF_FFFF;
const GENERIC_ALL: u32 = 0x1000_0000;
const UOI_NAME: i32 = 2;
const HANDLE_FLAG_INHERIT: u32 = 0x0000_0001;
const STARTF_USESTDHANDLES: u32 = 0x0000_0100;
const CREATE_UNICODE_ENVIRONMENT: u32 = 0x0000_0400;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const WTS_CURRENT_SESSION: u32 = 0xFFFF_FFFF;
const WTS_SESSION_INFO_EX: u32 = 25;
const WTS_SESSIONSTATE_LOCK: u32 = 0;
const WTS_SESSIONSTATE_UNLOCK: u32 = 1;

/// processthreadsapi.h: `STARTUPINFOW`.
#[repr(C)]
struct StartupInfo {
    size: u32,
    reserved: *mut u16,
    desktop: *mut u16,
    title: *mut u16,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    columns: u32,
    rows: u32,
    fill_attribute: u32,
    flags: u32,
    show_window: u16,
    reserved2_size: u16,
    reserved2: *mut u8,
    std_input: isize,
    std_output: isize,
    std_error: isize,
}

/// processthreadsapi.h: `PROCESS_INFORMATION`.
#[repr(C)]
#[derive(Default)]
struct ProcessInformation {
    process: isize,
    thread: isize,
    process_id: u32,
    thread_id: u32,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn CreateDesktopW(
        name: *const u16,
        device: *const u16,
        mode: *const c_void,
        flags: u32,
        access: u32,
        attributes: *const c_void,
    ) -> isize;
    fn CloseDesktop(desktop: isize) -> i32;
    fn GetProcessWindowStation() -> isize;
    fn GetUserObjectInformationW(
        object: isize,
        index: i32,
        info: *mut u16,
        length: u32,
        needed: *mut u32,
    ) -> i32;
    fn GetKeyboardState(state: *mut u8) -> i32;
    fn SetKeyboardState(state: *const u8) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateMutexW(attributes: *const c_void, initial_owner: i32, name: *const u16) -> isize;
    fn WaitForSingleObject(handle: isize, milliseconds: u32) -> u32;
    fn ReleaseMutex(handle: isize) -> i32;
    fn CloseHandle(handle: isize) -> i32;
    fn GetLastError() -> u32;
    fn SetHandleInformation(handle: isize, mask: u32, flags: u32) -> i32;
    fn CreateProcessW(
        application: *const u16,
        command_line: *mut u16,
        process_attributes: *const c_void,
        thread_attributes: *const c_void,
        inherit_handles: i32,
        creation_flags: u32,
        environment: *const c_void,
        current_directory: *const u16,
        startup: *const StartupInfo,
        information: *mut ProcessInformation,
    ) -> i32;
    fn GetExitCodeProcess(process: isize, code: *mut u32) -> i32;
    fn TerminateProcess(process: isize, code: u32) -> i32;
}

#[link(name = "wtsapi32")]
unsafe extern "system" {
    fn WTSQuerySessionInformationW(
        server: isize,
        session: u32,
        class: u32,
        answer: *mut *mut c_void,
        bytes: *mut u32,
    ) -> i32;
    fn WTSFreeMemory(memory: *mut c_void);
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// What a failed call says, with Windows' error for it.
fn failed(call: &str) -> String {
    // SAFETY: reads this thread's last error and nothing else.
    format!("{call} failed with error {}", unsafe { GetLastError() })
}

/// Whether Windows says this session is locked, asked of Windows and never
/// read from the process list.
#[derive(Debug, Clone, PartialEq)]
enum SessionLock {
    Locked,
    Unlocked,
    NotSaid(String),
}

/// Whether this session is locked, as Windows answers it.
fn this_sessions_lock() -> SessionLock {
    let mut answer: *mut c_void = std::ptr::null_mut();
    let mut bytes = 0u32;
    // SAFETY: Windows writes a buffer it owns into `answer` and its length
    // into `bytes`; the buffer is copied and then freed once.
    unsafe {
        let asked = WTSQuerySessionInformationW(
            0,
            WTS_CURRENT_SESSION,
            WTS_SESSION_INFO_EX,
            &mut answer,
            &mut bytes,
        );
        if asked == 0 || answer.is_null() {
            return SessionLock::NotSaid(format!(
                "WTSQuerySessionInformationW failed with error {}",
                GetLastError()
            ));
        }
        let copied = std::slice::from_raw_parts(answer as *const u8, bytes as usize).to_vec();
        WTSFreeMemory(answer);
        the_lock_in(&copied)
    }
}

/// The lock state in a `WTSINFOEXW` answer: its level in the first four
/// bytes, and the session's flags at byte 16.
fn the_lock_in(answer: &[u8]) -> SessionLock {
    let word = |at: usize| {
        answer
            .get(at..at + 4)
            .map(|four| u32::from_le_bytes([four[0], four[1], four[2], four[3]]))
    };
    match (word(0), word(16)) {
        (Some(1), Some(WTS_SESSIONSTATE_LOCK)) => SessionLock::Locked,
        (Some(1), Some(WTS_SESSIONSTATE_UNLOCK)) => SessionLock::Unlocked,
        (level, flags) => SessionLock::NotSaid(format!(
            "{} bytes, level {level:?}, session flags {flags:?}",
            answer.len()
        )),
    }
}

/// The one turn, held by the parent from before its child starts until the
/// child has ended, and released on the thread that took it.
///
/// WebView2 runs one browser process per user data folder, and the folder is
/// named for the executable, so two runs of this target at once would share
/// one browser, even on two desktops (ledger 761). It is taken in the parent
/// and never in the body the child runs, since the child would otherwise wait
/// on a turn its own parent holds.
struct TheOneTurn(isize);

impl TheOneTurn {
    fn take() -> Result<Self, String> {
        let name = wide(THE_TURN);
        // SAFETY: a null-terminated name and no security attributes.
        let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
        if handle == 0 {
            return Err(failed("CreateMutexW for the one turn"));
        }
        // SAFETY: a live mutex handle. An abandoned one, left by a run that
        // ended while holding it, is taken like a released one.
        let waited = unsafe { WaitForSingleObject(handle, LONGEST_WAIT_FOR_THE_TURN_MS) };
        if waited == WAIT_TIMEOUT || waited == WAIT_FAILED {
            // SAFETY: the handle made above, closed once.
            unsafe { CloseHandle(handle) };
            return Err(format!(
                "another run held the one turn for {} s",
                LONGEST_WAIT_FOR_THE_TURN_MS / 1000
            ));
        }
        Ok(TheOneTurn(handle))
    }
}

impl Drop for TheOneTurn {
    fn drop(&mut self) {
        // SAFETY: the handle this turn took, released on the thread that took it.
        unsafe {
            ReleaseMutex(self.0);
            CloseHandle(self.0);
        }
    }
}

/// A desktop made for this run on the window station the process is on,
/// closed when dropped.
struct ADesktopOfItsOwn {
    handle: isize,
    /// As `CreateProcessW` takes it: `station\desktop`.
    full_name: String,
}

impl ADesktopOfItsOwn {
    fn make(short: &str) -> Result<Self, String> {
        let station = the_window_stations_name()?;
        let name = format!("wixen-{short}-{}", std::process::id());
        // SAFETY: the name is null-terminated and every other pointer is
        // null, which CreateDesktopW takes as "none".
        let handle = unsafe {
            CreateDesktopW(
                wide(&name).as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                GENERIC_ALL,
                std::ptr::null(),
            )
        };
        if handle == 0 {
            return Err(failed("CreateDesktopW"));
        }
        Ok(ADesktopOfItsOwn {
            handle,
            full_name: format!("{station}\\{name}"),
        })
    }
}

impl Drop for ADesktopOfItsOwn {
    fn drop(&mut self) {
        // SAFETY: the handle made in `make`, closed once.
        unsafe { CloseDesktop(self.handle) };
    }
}

/// The name of the window station this process is on, read rather than
/// written, since a runner may not be on `WinSta0`.
fn the_window_stations_name() -> Result<String, String> {
    let mut buffer = [0u16; 256];
    let mut needed = 0u32;
    // SAFETY: the buffer's length is passed in bytes, and the station handle
    // is the process's own, which Windows owns and is never closed here.
    let read = unsafe {
        GetUserObjectInformationW(
            GetProcessWindowStation(),
            UOI_NAME,
            buffer.as_mut_ptr(),
            (buffer.len() * 2) as u32,
            &mut needed,
        )
    };
    if read == 0 {
        return Err(failed("GetUserObjectInformationW for the window station"));
    }
    let end = buffer
        .iter()
        .position(|&unit| unit == 0)
        .unwrap_or(buffer.len());
    Ok(String::from_utf16_lossy(&buffer[..end]))
}

/// This process's environment with `ON_A_DESKTOP_OF_THEIR_OWN` set to
/// `desktop`, as the block `CreateProcessW` takes.
fn the_childs_environment(desktop: &str) -> Vec<u16> {
    let mut block = Vec::new();
    let ours = OsStr::new(ON_A_DESKTOP_OF_THEIR_OWN);
    let theirs = std::env::vars_os().filter(|(key, _)| key.as_os_str() != ours);
    for (key, value) in theirs.chain(std::iter::once((ours.into(), desktop.into()))) {
        block.extend(key.encode_wide());
        block.push(u16::from(b'='));
        block.extend(value.encode_wide());
        block.push(0);
    }
    block.push(0);
    block
}

/// Run this executable whole, with no filter, on a desktop made for the run,
/// and say how it ended.
fn the_child_run(short: &str) -> ChildRun {
    let started = std::time::Instant::now();
    let desktop = ADesktopOfItsOwn::make(short);
    let full_name = desktop
        .as_ref()
        .map_or_else(|_| format!("wixen-{short}"), |it| it.full_name.clone());
    let (end, output) = desktop
        .and_then(|desktop| run_in_the_one_turn(&desktop))
        .unwrap_or_else(|why| (ChildEnd::NeverStarted(why), String::new()));
    println!(
        "the child ran on {full_name} for {} s and {end}",
        started.elapsed().as_secs()
    );
    ChildRun {
        desktop: full_name,
        end,
        output,
        lock: this_sessions_lock(),
    }
}

/// Take the one turn, run the child on `desktop` with its output written to
/// a file, let the turn go once it has ended, and hand back how it ended and
/// what it wrote.
fn run_in_the_one_turn(desktop: &ADesktopOfItsOwn) -> Result<(ChildEnd, String), String> {
    let folder = tempfile::tempdir().map_err(|why| format!("the child's output folder: {why}"))?;
    let output = folder.path().join("child.txt");
    let turn = TheOneTurn::take()?;
    let end = run_and_wait(desktop, &output);
    drop(turn);
    let said = std::fs::read(&output)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default();
    Ok((end?, said))
}

/// Start the child on `desktop` with its output written to `output`, wait at
/// most `THE_CHILDS_BOUND_MS`, and say how it ended.
fn run_and_wait(desktop: &ADesktopOfItsOwn, output: &Path) -> Result<ChildEnd, String> {
    let file = std::fs::File::create(output).map_err(|why| format!("the child's output: {why}"))?;
    let handle = file.as_raw_handle() as isize;
    // SAFETY: a live file handle this function owns; only its flag changes.
    if unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) } == 0 {
        return Err(failed("SetHandleInformation"));
    }
    let me = std::env::current_exe().map_err(|why| format!("this test's own path: {why}"))?;
    let mut command_line = wide(&format!("\"{}\"", me.display()));
    let mut desktop_name = wide(&desktop.full_name);
    let environment = the_childs_environment(&desktop.full_name);
    let startup = StartupInfo {
        size: std::mem::size_of::<StartupInfo>() as u32,
        reserved: std::ptr::null_mut(),
        desktop: desktop_name.as_mut_ptr(),
        title: std::ptr::null_mut(),
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        columns: 0,
        rows: 0,
        fill_attribute: 0,
        flags: STARTF_USESTDHANDLES,
        show_window: 0,
        reserved2_size: 0,
        reserved2: std::ptr::null_mut(),
        std_input: 0,
        std_output: handle,
        std_error: handle,
    };
    let mut child = ProcessInformation::default();
    // SAFETY: every string is null-terminated and outlives the call, the
    // environment block ends in two nulls, and Windows writes `child`.
    let made = unsafe {
        CreateProcessW(
            std::ptr::null(),
            command_line.as_mut_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1,
            CREATE_NO_WINDOW | CREATE_UNICODE_ENVIRONMENT,
            environment.as_ptr() as *const c_void,
            std::ptr::null(),
            &startup,
            &mut child,
        )
    };
    if made == 0 {
        return Err(failed("CreateProcessW"));
    }
    drop(file);
    let ended = the_end_of(&child);
    // SAFETY: the two handles CreateProcessW handed back, closed once.
    unsafe {
        CloseHandle(child.thread);
        CloseHandle(child.process);
    }
    ended
}

/// Wait for the child at most `THE_CHILDS_BOUND_MS`, stopping it there.
fn the_end_of(child: &ProcessInformation) -> Result<ChildEnd, String> {
    // SAFETY: a live process handle.
    match unsafe { WaitForSingleObject(child.process, THE_CHILDS_BOUND_MS) } {
        WAIT_TIMEOUT => {
            // SAFETY: a live process handle; the wait lets it finish going.
            unsafe {
                TerminateProcess(child.process, 1);
                WaitForSingleObject(child.process, 10_000);
            }
            Ok(ChildEnd::StoppedAtTheBound)
        }
        WAIT_FAILED => Err(failed("WaitForSingleObject on the child")),
        _ => {
            let mut code = 0u32;
            // SAFETY: a live process handle that has ended.
            if unsafe { GetExitCodeProcess(child.process, &mut code) } == 0 {
                return Err(failed("GetExitCodeProcess"));
            }
            Ok(ChildEnd::Exited(code))
        }
    }
}

/// Whether this window test was run in a child on a desktop made for the
/// run, which is so in the parent; there it fails, with one sentence saying
/// why, unless the child's line for it says ok. In the child it answers
/// false and the test runs its body.
fn ran_in_a_child_on_a_desktop_of_its_own(test: &str) -> bool {
    if std::env::var_os(ON_A_DESKTOP_OF_THEIR_OWN).is_some() {
        return false;
    }
    static THE_CHILD: OnceLock<ChildRun> = OnceLock::new();
    let run = THE_CHILD.get_or_init(|| the_child_run("meeting"));
    if let Err(why) = what_the_child_said(test, run) {
        panic!("{why}");
    }
    true
}

/// How the child run ended.
#[derive(Debug, Clone, PartialEq)]
enum ChildEnd {
    /// It ended by itself with this exit code.
    Exited(u32),
    /// It ran past its five minutes and was stopped.
    StoppedAtTheBound,
    /// It never started, for this reason.
    NeverStarted(String),
}

impl fmt::Display for ChildEnd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChildEnd::Exited(code) => write!(f, "exited with exit code {code:#x}"),
            ChildEnd::StoppedAtTheBound => {
                write!(f, "did not finish in five minutes and was stopped")
            }
            ChildEnd::NeverStarted(why) => write!(f, "never started: {why}"),
        }
    }
}

impl fmt::Display for SessionLock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionLock::Locked => write!(f, "Windows says the session is locked."),
            SessionLock::Unlocked => write!(f, "Windows says the session is unlocked."),
            SessionLock::NotSaid(answer) => write!(
                f,
                "Windows did not say whether the session is locked: asking answered {answer}."
            ),
        }
    }
}

/// What the child run left for the parent to judge each window test by.
#[derive(Debug, Clone, PartialEq)]
struct ChildRun {
    /// The desktop it ran on, as `station\desktop`.
    desktop: String,
    end: ChildEnd,
    /// Its standard output and error, as libtest wrote them.
    output: String,
    /// What Windows said about the session lock once the child had ended.
    lock: SessionLock,
}

/// How many lines of the child's output a sentence quotes when the test's
/// own failure block is not there to quote.
const LINES_QUOTED_FROM_THE_END: usize = 20;

/// Whether the child's run says `test` passed, and if not, one sentence
/// saying what happened instead, ending with what Windows says about the
/// lock.
fn what_the_child_said(test: &str, run: &ChildRun) -> Result<(), String> {
    let line = |verdict: &str| format!("test {test} ... {verdict}");
    let said = |line: &str| run.output.lines().any(|it| it.trim_end() == line);
    let where_ = format!("the child run on {}", run.desktop);
    let lock = &run.lock;
    if said(&line("ok")) {
        return Ok(());
    }
    if said(&line("FAILED")) {
        let block = the_failure_block(test, &run.output).unwrap_or_else(|| {
            format!(
                "(no failure block for it; its output ends: {})",
                the_end_of_the_output(&run.output)
            )
        });
        return Err(format!("{test} failed in {where_}:\n{block}\n{lock}"));
    }
    match &run.end {
        ChildEnd::StoppedAtTheBound => Err(format!(
            "{where_} {}, so {test} has no result; its output ends:\n{}\n{lock}",
            run.end,
            the_end_of_the_output(&run.output)
        )),
        end => Err(format!(
            "{where_} {end} and never reported {test}; its output ends:\n{}\n{lock}",
            the_end_of_the_output(&run.output)
        )),
    }
}

/// The block libtest writes for a failed test: from its `---- NAME stdout
/// ----` line to the next such line or the list of failures after them.
fn the_failure_block(test: &str, output: &str) -> Option<String> {
    let header = format!("---- {test} stdout ----");
    let mut lines = output.lines().skip_while(|line| line.trim_end() != header);
    let first = lines.next()?;
    let rest =
        lines.take_while(|line| !line.starts_with("---- ") && line.trim_end() != "failures:");
    let block: Vec<&str> = std::iter::once(first).chain(rest).collect();
    Some(block.join("\n").trim_end().to_string())
}

/// The last lines of the child's output, or a sentence saying it wrote none.
fn the_end_of_the_output(output: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    if lines.is_empty() {
        return "(it wrote nothing)".to_string();
    }
    let from = lines.len().saturating_sub(LINES_QUOTED_FROM_THE_END);
    lines[from..].join("\n")
}

/// What a button sends its parent when it is clicked, sent to the parent
/// directly: `BM_CLICK` is refused by a window that is not in front, and a
/// window a test builds is not.
const WM_COMMAND: u32 = 0x0111;
const BN_CLICKED: usize = 0;

#[link(name = "oleacc")]
unsafe extern "system" {
    fn AccessibleObjectFromWindow(
        hwnd: isize,
        id_object: u32,
        riid: *const Guid,
        out: *mut *mut c_void,
    ) -> Hresult;
}

#[link(name = "oleaut32")]
unsafe extern "system" {
    fn SysStringLen(s: *mut u16) -> u32;
    fn SysFreeString(s: *mut u16);
}

thread_local! {
    static FOUND: RefCell<Vec<isize>> = const { RefCell::new(Vec::new()) };
}

extern "system" fn collect(hwnd: isize, _lparam: isize) -> i32 {
    FOUND.with(|found| found.borrow_mut().push(hwnd));
    1
}

fn descendants_of(parent: isize) -> Vec<isize> {
    FOUND.with(|found| found.borrow_mut().clear());
    // SAFETY: the callback only pushes to this thread's local.
    unsafe { EnumChildWindows(parent, collect, 0) };
    FOUND.with(|found| found.borrow().clone())
}

/// Every top-level window the calling thread owns.
fn this_threads_windows() -> Vec<isize> {
    FOUND.with(|found| found.borrow_mut().clear());
    // SAFETY: the callback only pushes to this thread's local.
    unsafe { EnumThreadWindows(GetCurrentThreadId(), collect, 0) };
    FOUND.with(|found| found.borrow().clone())
}

/// Press a button the way a click does: its command, sent to its parent.
fn press(parent: isize, button: isize) {
    // SAFETY: two live windows of this thread; the command is what the
    // button sends on a click, so the handler that runs is the click's.
    unsafe {
        let id = GetDlgCtrlID(button) as u16 as usize;
        SendMessageW(parent, WM_COMMAND, id | (BN_CLICKED << 16), button);
    }
}

fn class_name(hwnd: isize) -> String {
    let mut buffer = [0u16; 256];
    // SAFETY: the buffer is as long as the count says.
    let len = unsafe { GetClassNameW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

fn window_text(hwnd: isize) -> String {
    let mut buffer = [0u16; 4096];
    // SAFETY: the buffer is as long as the count says.
    let len = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize]).replace("\r\n", "\n")
}

unsafe fn vtable_entry(object: *mut c_void, index: usize) -> *const c_void {
    // SAFETY: a COM object is a pointer to its vtable.
    unsafe {
        let vtable = *(object as *const *const *const c_void);
        *vtable.add(index)
    }
}

unsafe fn take_bstr(s: *mut u16) -> String {
    if s.is_null() {
        return String::new();
    }
    // SAFETY: a BSTR carries its length; it is freed once, here.
    unsafe {
        let len = SysStringLen(s) as usize;
        let text = String::from_utf16_lossy(std::slice::from_raw_parts(s, len));
        SysFreeString(s);
        text
    }
}

/// What a window's own object answers over MSAA.
#[derive(Debug, Clone, PartialEq)]
struct Msaa {
    name: String,
    description: String,
    role: i64,
    state: i64,
}

fn msaa_of(hwnd: isize) -> Result<Msaa, String> {
    let mut object: *mut c_void = std::ptr::null_mut();
    // SAFETY: a live window handle; the object is released before returning.
    let hr =
        unsafe { AccessibleObjectFromWindow(hwnd, OBJID_CLIENT, &IID_IACCESSIBLE, &mut object) };
    if hr < 0 || object.is_null() {
        return Err(format!("AccessibleObjectFromWindow failed 0x{hr:x}"));
    }
    // SAFETY: `object` is a live IAccessible; the slots are IAccessible's.
    unsafe {
        let get_name: GetBstrFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_NAME));
        let get_description: GetBstrFn =
            std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_DESCRIPTION));
        let get_role: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_ROLE));
        let get_state: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_STATE));
        let text = |getter: GetBstrFn| {
            let mut s: *mut u16 = std::ptr::null_mut();
            match getter(object, Variant::child(CHILDID_SELF), &mut s) >= 0 {
                true => take_bstr(s),
                false => String::new(),
            }
        };
        let name = text(get_name);
        let description = text(get_description);
        let mut role = Variant::empty();
        let hr_role = get_role(object, Variant::child(CHILDID_SELF), &mut role);
        let mut state = Variant::empty();
        let hr_state = get_state(object, Variant::child(CHILDID_SELF), &mut state);
        let release: ReleaseFn = std::mem::transmute(vtable_entry(object, VTBL_RELEASE));
        release(object);
        let read = |hr: Hresult, v: Variant| match hr >= 0 && v.vt == VT_I4 {
            true => v.val & 0xFFFF_FFFF,
            false => -1,
        };
        Ok(Msaa {
            name,
            description,
            role: read(hr_role, role),
            state: read(hr_state, state),
        })
    }
}

/// One control of a window, as Windows holds it.
#[derive(Debug, Clone, PartialEq)]
struct Control {
    hwnd: isize,
    class: String,
    text: String,
    msaa: Msaa,
}

fn read_the_controls(parent: isize) -> Result<Vec<Control>, String> {
    descendants_of(parent)
        .into_iter()
        .map(|hwnd| {
            Ok(Control {
                hwnd,
                class: class_name(hwnd),
                text: window_text(hwnd),
                msaa: msaa_of(hwnd)?,
            })
        })
        .collect()
}

fn the_buttons(controls: &[Control]) -> Vec<&Control> {
    controls
        .iter()
        .filter(|control| control.class == "Button")
        .collect()
}

/// A cancellation from the organiser, composed the way the reader composes
/// one after asking what opening it changes: offered for removal.
fn a_cancellation_offered_for_removal() -> WhatIsSaidAboutIt {
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store_holding_the_meeting(&dir);
    let (item, body) = opened(&cache, 1, &a_cancellation_from(ADA), ADA);
    let change = reading_a_message::what_opening_it_in_a_reader_changed(
        Some(&cache),
        item.message_id,
        &item.from,
        written_out_in_full,
        sam,
    );
    let shown = reading_a_message::for_message(
        Some(&cache),
        item.message_id,
        &item.from,
        body,
        written_out_in_full,
        sam,
    );
    WhatIsSaidAboutIt {
        change,
        ..shown.said
    }
}

fn a_message_from_ada(row: i64) -> MessageItem {
    MessageItem {
        message_id: row,
        subject: "Cancelled: Quarterly review".to_string(),
        from: ADA.to_string(),
        ..Default::default()
    }
}

#[derive(Debug, Default)]
struct Harvest {
    /// The cancellation's tab in the text reader.
    text_tab: Vec<Control>,
    /// What the removal handler was handed after Alt+R in the message.
    pressed_by_alt_r: Vec<Removal>,
    /// The formatted window opened on the cancellation alone.
    formatted_window: Vec<Control>,
    /// What the removal handler was handed when its button was pressed there.
    pressed_in_the_formatted_window: Vec<Removal>,
}

/// Remove from Calendar for the whole meeting on row `evt-1`.
fn the_meeting_removed() -> Removal {
    Removal::TheMeeting {
        event_id: "evt-1".to_string(),
    }
}

struct Gathered {
    harvest: Result<Harvest, String>,
    page_windows: Vec<isize>,
}

/// The classes WebView2 gives the windows it puts under a host control.
const CHROMIUMS_CLASSES: [&str; 2] = ["Chrome_WidgetWin_1", "Chrome_RenderWidgetHostHWND"];

fn the_browser_is_there(page_window: isize) -> bool {
    descendants_of(page_window)
        .into_iter()
        .any(|hwnd| CHROMIUMS_CLASSES.contains(&class_name(hwnd).as_str()))
}

/// Ticks before the session stops waiting for a browser: half a minute.
const GIVE_UP_AFTER_TICKS: u32 = 1000;

fn take_the_harvest() -> Result<Harvest, String> {
    if std::mem::size_of::<Variant>() != 24 {
        return Err("VARIANT is not 24 bytes here, so the reader's layout is wrong".to_string());
    }
    let said = a_cancellation_offered_for_removal();
    if said.change.offered_removal().is_none() {
        return Err(format!(
            "the fixture is not offered for removal, so there is no button to read: {:?}",
            said.change
        ));
    }
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let settle = move |taken: Result<Harvest, String>| {
                if let Ok(mut slot) = outcome.lock() {
                    *slot = Some(taken);
                }
                wxdragon::call_after(Box::new(move || app.exit_main_loop()));
            };
            let a11y = match Accessibility::new() {
                Ok(a11y) => Arc::new(a11y),
                Err(why) => {
                    settle(Err(format!("no accessibility layer: {why}")));
                    return;
                }
            };
            let parent = Frame::builder().build();
            let reader = Rc::new(ReaderWindow::new(&parent, &a11y));
            reader.wire_menu();
            let pressed: Rc<RefCell<Vec<Removal>>> = Rc::default();
            reader.on_remove({
                let pressed = pressed.clone();
                move |removal| pressed.borrow_mut().push(removal.clone())
            });

            let document = reader_text::single_message(
                &a_message_from_ada(42),
                &MessageBody::Plain("The quarterly review is off.".to_string()),
                reading(),
            )
            .with_what_is_said(&said);
            let tab = reader.open(document);
            let panel = tab.panel.get_handle() as isize;
            let text_tab = match read_the_controls(panel) {
                Ok(read) => read,
                Err(why) => {
                    settle(Err(why));
                    return;
                }
            };
            let text = tab.text.get_handle() as isize;
            // Alt alone down while the loop translates the key, and nothing
            // down once it has been read.
            only_these_modifiers_down(&[VK_MENU]);
            // SAFETY: a live window of this thread; the key goes through the
            // window's own loop, which is what a real key does.
            unsafe {
                SetFocus(text);
                PostMessageW(text, WM_SYSKEYDOWN, VK_R, ALT_HELD);
                PostMessageW(text, WM_SYSKEYUP, VK_R, ALT_RELEASED);
            }

            let ticks = Rc::new(std::cell::Cell::new(0u32));
            let gathered: Rc<RefCell<Option<Gathered>>> = Rc::default();
            let ticker = Rc::new(Timer::new(&parent));
            ticker.on_tick({
                let ticker = ticker.clone();
                let reader = reader.clone();
                move |_| {
                    ticks.set(ticks.get() + 1);
                    if ticks.get() < TICKS_FOR_THE_KEY {
                        return;
                    }
                    let waiting = match gathered.borrow().as_ref() {
                        Some(already) => already
                            .page_windows
                            .iter()
                            .any(|window| !the_browser_is_there(*window)),
                        None => false,
                    };
                    if gathered.borrow().is_some() {
                        if waiting && ticks.get() < GIVE_UP_AFTER_TICKS {
                            return;
                        }
                        ticker.stop();
                        if let Some(done) = gathered.borrow_mut().take() {
                            settle(done.harvest);
                        }
                        return;
                    }
                    only_these_modifiers_down(&[]);
                    let pressed_by_alt_r = pressed.borrow().clone();

                    let subject = "A meeting called off";
                    wx_app::show_conversation_as_page(
                        &parent,
                        &reader,
                        &a11y,
                        subject,
                        &[ConversationPart {
                            message: a_message_from_ada(44),
                            body: MessageBody::Plain("The quarterly review is off.".to_string()),
                            said: said.clone(),
                            depth: 0,
                        }],
                        None,
                    );
                    let title = format!("{subject} - headings - Wixen Mail");
                    let formatted = this_threads_windows()
                        .into_iter()
                        .find(|hwnd| window_text(*hwnd) == title)
                        .ok_or(format!("no window titled {title:?} was opened"))
                        .and_then(|window| Ok((window, read_the_controls(window)?)));
                    let before = pressed.borrow().len();
                    if let Ok((window, controls)) = &formatted
                        && let Some(remove) = the_buttons(controls).first()
                    {
                        press(*window, remove.hwnd);
                    }
                    let pressed_in_the_formatted_window = pressed.borrow()[before..].to_vec();

                    let page_windows: Vec<isize> = formatted
                        .as_ref()
                        .ok()
                        .map(|(window, _)| *window)
                        .into_iter()
                        .collect();
                    let harvest = formatted.map(|(_, formatted_window)| Harvest {
                        text_tab: text_tab.clone(),
                        pressed_by_alt_r,
                        formatted_window,
                        pressed_in_the_formatted_window,
                    });
                    *gathered.borrow_mut() = Some(Gathered {
                        harvest,
                        page_windows,
                    });
                }
            });
            ticker.start(TICK_MS, false);
            std::mem::forget(ticker);
            std::mem::forget(reader);
        })
    };
    if let Err(why) = result {
        return Err(format!("wxdragon::main returned {why:?}"));
    }
    let taken = outcome
        .lock()
        .map_err(|_| "the harvest's lock was poisoned".to_string())?
        .take();
    taken.unwrap_or_else(|| Err("the window session ended without a harvest".to_string()))
}

/// The one harvest of this process, taken by whichever test asks first.
fn the_harvest() -> &'static Harvest {
    static HARVEST: OnceLock<Result<Harvest, String>> = OnceLock::new();
    match HARVEST.get_or_init(take_the_harvest) {
        Ok(harvest) => harvest,
        Err(why) => panic!("the window session could not be read: {why}"),
    }
}

/// What is wrong with a window's buttons for a cancellation: exactly one, the
/// push button Remove from Calendar, with its letter, described by what
/// pressing it does, and not greyed.
fn what_is_wrong_with_the_button(controls: &[Control]) -> Vec<String> {
    let buttons = the_buttons(controls);
    let [button] = buttons.as_slice() else {
        return vec![format!(
            "{} buttons, not the one Remove from Calendar: {:?}",
            buttons.len(),
            buttons
                .iter()
                .map(|button| &button.msaa.name)
                .collect::<Vec<_>>()
        )];
    };
    let mut wrong = Vec::new();
    if button.msaa.name != "Remove from Calendar" {
        wrong.push(format!("the button is named {:?}", button.msaa.name));
    }
    if button.text != "&Remove from Calendar" {
        wrong.push(format!("the button is labelled {:?}", button.text));
    }
    if button.msaa.description
        != "Marks this meeting cancelled on your calendar. Nothing is sent to the organiser."
    {
        wrong.push(format!(
            "the button is described as {:?}",
            button.msaa.description
        ));
    }
    if button.msaa.role != ROLE_SYSTEM_PUSHBUTTON {
        wrong.push("the button is not read as a push button".to_string());
    }
    if button.msaa.state & STATE_SYSTEM_UNAVAILABLE != 0 {
        wrong.push("the button is greyed, and Tab passes a greyed button by".to_string());
    }
    wrong
}

fn complain(what: &str, wrong: &[String]) {
    assert!(
        wrong.is_empty(),
        "{} thing(s) wrong, {what}:\n  {}",
        wrong.len(),
        wrong.join("\n  ")
    );
}

#[test]
fn test_a_cancellation_in_the_text_reader_has_one_named_described_button_and_no_answers() {
    if ran_in_a_child_on_a_desktop_of_its_own(
        "test_a_cancellation_in_the_text_reader_has_one_named_described_button_and_no_answers",
    ) {
        return;
    }
    complain(
        "on the text reader's button",
        &what_is_wrong_with_the_button(&the_harvest().text_tab),
    );
}

#[test]
fn test_the_button_comes_after_the_bar_and_before_the_message() {
    if ran_in_a_child_on_a_desktop_of_its_own(
        "test_the_button_comes_after_the_bar_and_before_the_message",
    ) {
        return;
    }
    let controls = &the_harvest().text_tab;
    let at = |wanted: &dyn Fn(&Control) -> bool| controls.iter().position(wanted);
    let bar = at(&|control| control.msaa.name == "Security warning");
    let button = at(&|control| control.class == "Button");
    let message = at(&|control| control.class.starts_with("RICHEDIT"));
    assert!(
        matches!((bar, button, message), (Some(b), Some(u), Some(m)) if b < u && u < m),
        "the bar, the button and the message are at {:?}",
        (bar, button, message)
    );
}

#[test]
fn test_alt_r_in_the_message_presses_remove_once_for_the_meeting_offered() {
    if ran_in_a_child_on_a_desktop_of_its_own(
        "test_alt_r_in_the_message_presses_remove_once_for_the_meeting_offered",
    ) {
        return;
    }
    assert_eq!(
        the_harvest().pressed_by_alt_r,
        vec![the_meeting_removed()],
        "Alt+R in the message's text did not press Remove from Calendar exactly once"
    );
}

#[test]
fn test_the_formatted_window_has_the_button_and_it_reaches_the_same_handler() {
    if ran_in_a_child_on_a_desktop_of_its_own(
        "test_the_formatted_window_has_the_button_and_it_reaches_the_same_handler",
    ) {
        return;
    }
    let harvest = the_harvest();
    complain(
        "on the formatted window's button",
        &what_is_wrong_with_the_button(&harvest.formatted_window),
    );
    assert_eq!(
        harvest.pressed_in_the_formatted_window,
        vec![the_meeting_removed()],
        "Remove from Calendar in the formatted window did not reach the handler once"
    );
}

#[test]
fn test_the_reading_refuses_a_second_button_or_a_wrong_name() {
    if ran_in_a_child_on_a_desktop_of_its_own(
        "test_the_reading_refuses_a_second_button_or_a_wrong_name",
    ) {
        return;
    }
    // The reading above can see the two ways a button goes wrong.
    let mut controls = the_harvest().text_tab.clone();
    let button = controls
        .iter()
        .find(|control| control.class == "Button")
        .cloned()
        .expect("the tab has a button");
    let mut renamed = button.clone();
    renamed.msaa.name = "Remove".to_string();
    let wrong_name = controls
        .iter()
        .map(|control| match control.class == "Button" {
            true => renamed.clone(),
            false => control.clone(),
        })
        .collect::<Vec<_>>();
    controls.push(button);

    assert!(
        what_is_wrong_with_the_button(&controls)
            .iter()
            .any(|why| why.contains("2 buttons")),
        "two buttons were passed over"
    );
    assert!(
        what_is_wrong_with_the_button(&wrong_name)
            .iter()
            .any(|why| why.contains("named \"Remove\"")),
        "a wrong name was passed over"
    );
}

// ── Read from the source ──────────────────────────────────────────────────

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

fn shipped(path: &str) -> String {
    wixen_mail::common::what_ships::what_ships(
        &std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("{path}: {e}"))
            .replace("\r\n", "\n"),
    )
}

/// One function's text, from its signature to the closing brace at column
/// nought, or a complaint when the signature is gone.
fn body_of(source: &str, signature: &str) -> Result<String, String> {
    let at = source.find(signature).ok_or(format!(
        "{signature} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let ends = rest.find("\n}\n").map_or(rest.len(), |end| end + 2);
    Ok(rest[..ends].to_string())
}

/// The call that applies what opening a message changes.
const APPLYING: &str = "what_opening_it_changes_on_the_calendar(";

fn only_a_reader_window_applies_a_change(app: &str) -> Result<(), String> {
    if !body_of(app, "fn open_single_message(")?.contains(APPLYING) {
        return Err(
            "opening a message in a reader window never asks what it changes, so an \
             organiser's update never moves the meeting"
                .to_string(),
        );
    }
    if body_of(app, "fn the_preview_of(")?.contains(APPLYING) {
        return Err(
            "the preview applies what a message changes, so arrowing past an update moves \
             a meeting nobody asked to move"
                .to_string(),
        );
    }
    Ok(())
}

#[test]
fn test_only_a_reader_window_applies_what_opening_a_message_changes() {
    only_a_reader_window_applies_a_change(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_reading_complains_when_the_preview_applies_a_change() {
    let app = shipped(THE_MAIN_WINDOW);
    let preview = body_of(&app, "fn the_preview_of(").expect("the preview");
    let planted = preview.replacen(
        "    let Some(message) = showing else {",
        &format!(
            "    let _ = {APPLYING}cache, &message);\n    let Some(message) = showing else {{"
        ),
        1,
    );
    assert_ne!(planted, preview, "the companion planted nothing");
    let why = only_a_reader_window_applies_a_change(&app.replacen(&preview, &planted, 1))
        .expect_err("a preview that applies a change was passed over");
    assert!(why.contains("arrowing past"), "{why}");
}

fn the_page_removes_through_the_buttons_handler(app: &str) -> Result<(), String> {
    let body = body_of(app, "pub fn show_conversation_as_page(")?;
    const THE_ARM: &str = "Some(page_jumps::Jump::Remove) =>";
    let at = body
        .find(THE_ARM)
        .ok_or("the page window has no arm for the page's Alt+R")?;
    let arm = &body[at..];
    let arm = &arm[..arm.find("\n                None =>").unwrap_or(arm.len())];
    if !arm.contains("reader.remove_now(") {
        return Err(
            "the page's Alt+R does not reach the handler the button presses, so Alt+R in the \
             message removes nothing"
                .to_string(),
        );
    }
    if !arm.contains("\"There is no cancelled meeting here to remove.\"") {
        return Err(
            "the page's Alt+R says nothing on a message with no meeting to remove".to_string(),
        );
    }
    Ok(())
}

#[test]
fn test_the_pages_alt_r_presses_the_buttons_handler() {
    the_page_removes_through_the_buttons_handler(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_reading_complains_when_alt_r_reaches_nothing() {
    let app = shipped(THE_MAIN_WINDOW);
    let body = body_of(&app, "pub fn show_conversation_as_page(").expect("the page window");
    let at = body
        .find("Some(page_jumps::Jump::Remove) =>")
        .expect("the companion's anchor, the page's remove arm, is not in the page window");
    let (before, the_arm_onwards) = body.split_at(at);
    let planted_body = format!(
        "{before}{}",
        the_arm_onwards.replacen("reader.remove_now(", "drop((", 1)
    );
    assert_ne!(planted_body, body, "the companion planted nothing");
    let why = the_page_removes_through_the_buttons_handler(&app.replacen(&body, &planted_body, 1))
        .expect_err("Alt+R reaching nothing was passed over");
    assert!(why.contains("removes nothing"), "{why}");
}

fn removing_marks_and_says_it_once(app: &str) -> Result<(), String> {
    let body = body_of(app, "fn take_the_called_off_meeting_off_the_calendar(")?;
    if !body.contains("mark_the_meeting_called_off(") {
        return Err("Remove from Calendar marks nothing".to_string());
    }
    if body.contains("delete_calendar_event(") {
        return Err("Remove from Calendar deletes the meeting rather than marking it".to_string());
    }
    if body.contains(".announce(") {
        return Err(
            "Remove from Calendar announces beside the status line, which speaks it too, so \
             it is heard twice"
                .to_string(),
        );
    }
    if !body.contains("\"Removed from your calendar.\"") {
        return Err("Remove from Calendar does not say it removed the meeting".to_string());
    }
    Ok(())
}

#[test]
fn test_remove_from_calendar_marks_the_meeting_and_says_so_once() {
    removing_marks_and_says_it_once(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_reading_complains_when_removing_deletes() {
    let app = shipped(THE_MAIN_WINDOW);
    let body = body_of(&app, "fn take_the_called_off_meeting_off_the_calendar(")
        .expect("the removal handler");
    let planted = body.replacen("mark_the_meeting_called_off(", "delete_calendar_event(", 1);
    assert_ne!(planted, body, "the companion planted nothing");
    let why = removing_marks_and_says_it_once(&app.replacen(&body, &planted, 1))
        .expect_err("a removal that deletes was passed over");
    assert!(
        why.contains("marks nothing") || why.contains("deletes"),
        "{why}"
    );
}

/// The change the reader applies is the one `MeetingChange` names, so a
/// fixture that forgot to ask is not read as a button that is missing.
#[test]
fn test_the_windows_fixture_is_the_organisers_cancellation_offered_for_removal() {
    assert_eq!(
        a_cancellation_offered_for_removal().change,
        MeetingChange::OfferRemoval {
            event_id: "evt-1".to_string()
        }
    );
}

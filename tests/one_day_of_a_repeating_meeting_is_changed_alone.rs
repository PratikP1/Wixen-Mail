//! An organiser's update or cancellation for one day of a repeating meeting
//! changes that day and no other, from the bytes of the message to the rows
//! the calendar shows (GAP-04, ledger 638, 13-36.3).
//!
//! The way `a_meeting_change_reaches_the_calendar.rs` does it: a raw message
//! parsed, its parts stored the way opening a message stores them, the
//! calendar holding a weekly series with Ada recorded as its organiser, then
//! the reader's own application call, `what_opening_it_in_a_reader_changed`,
//! and the composition every surface asks. The calendar is read back the way
//! its panel reads it: the events that could fall in a window, expanded
//! across the days each lands on, counted by date.
//!
//! The messages are shaped the way RFC 5546 section 3.2.2 describes an
//! update to one instance, which is the shape Outlook sends: the series' UID,
//! a `RECURRENCE-ID` naming the day with its zone, and the day's new start
//! and end. They are written from the standard and not taken from a real
//! Outlook; no real organiser's message for one day has been read here,
//! which is phase 14's, on the ledger.
//!
//! The last part builds the real text reader on a one-day cancellation, reads
//! its one button over MSAA at its own handle, the way NVDA reads a native
//! control, presses it through the reader's own `remove_now` into the main
//! window's real handler, and reads the calendar back. A source reading holds
//! the handler's one-day arm to calling the day off and nothing else.
//!
//! # What this cannot see
//!
//! Whether Google or Outlook apply an organiser's change to one day to the
//! guest's copy themselves, and what a calendar server makes of the pair a
//! day kept apart sends. How the sentences sound is the tester's ear.

#![cfg(windows)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::allowed::Allowed;
use wixen_mail::application::invitations;
use wixen_mail::application::meeting_changes::{MeetingChange, Removal};
use wixen_mail::application::reading_a_message::{self, AnsweringAs, WhatIsSaidAboutIt};
use wixen_mail::common::types::MessageBody;
use wixen_mail::data::message_cache::attachment_content::AttachmentWithContent;
use wixen_mail::data::message_cache::{
    CachedFolder, CachedMessage, CalendarContainer, CalendarEventEntry, MessageCache,
};
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::date_display::{
    Clock, DateOrder, DateSettings, DateStyle, DateWording,
};
use wixen_mail::presentation::read_aloud::Reading;
use wixen_mail::presentation::reader_text::{self, ReaderDocument};
use wixen_mail::presentation::ui_types::{CalendarEventItem, MessageItem, UIUpdate};
use wixen_mail::presentation::wx_app;
use wixen_mail::presentation::wx_reader::ReaderWindow;
use wixen_mail::service::mime;
use wxdragon::prelude::*;

// ── The messages ──────────────────────────────────────────────────────────

const ADA: &str = "Ada Lovelace <ada@example.com>";
const THE_MEETING: &str = "s-1@example.com";
const ACCOUNT: &str = "acc-1";
const LONDON: &str = "Europe/London";

/// A message from Ada carrying `lines`, the meeting's own lines, under
/// `method`, as a calendar program sends one: a covering note and the meeting
/// as a second alternative.
fn a_message(method: &str, number: u32, lines: &str) -> String {
    format!(
        "From: {ADA}\r\n\
To: sam@example.com\r\n\
Subject: Weekly sync\r\n\
Message-ID: <change-{number}@example.com>\r\n\
Date: Tue, 10 Mar 2026 10:00:00 +0000\r\n\
MIME-Version: 1.0\r\n\
Content-Type: multipart/alternative; boundary=\"alt\"\r\n\
\r\n\
--alt\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
A change to the weekly sync.\r\n\
--alt\r\n\
Content-Type: text/calendar; charset=utf-8; method={method}\r\n\
\r\n\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example//EN\r\n\
METHOD:{method}\r\n\
BEGIN:VEVENT\r\n\
UID:{THE_MEETING}\r\n\
SUMMARY:Weekly sync\r\n\
{lines}\
ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n\
ATTENDEE;CN=Sam;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:sam@example.com\r\n\
END:VEVENT\r\n\
END:VCALENDAR\r\n\
--alt--\r\n"
    )
}

/// Version `version` of the Thursday of 12 March, moved to Friday the
/// thirteenth at `hour` o'clock for an hour, in London.
fn one_day_moved(version: u32, hour: u32) -> String {
    a_message(
        "REQUEST",
        version,
        &format!(
            "SEQUENCE:{version}\r\n\
RECURRENCE-ID;TZID=Europe/London:20260312T090000\r\n\
DTSTART;TZID=Europe/London:20260313T{hour:02}0000\r\n\
DTEND;TZID=Europe/London:20260313T{end:02}0000\r\n",
            end = hour + 1
        ),
    )
}

/// The Thursday of 12 March called off.
fn one_day_called_off() -> String {
    a_message(
        "CANCEL",
        9,
        "SEQUENCE:3\r\n\
RECURRENCE-ID;TZID=Europe/London:20260312T090000\r\n\
DTSTART;TZID=Europe/London:20260312T090000\r\n\
DTEND;TZID=Europe/London:20260312T100000\r\n",
    )
}

/// Every day of the series moved from nine to ten o'clock, still weekly.
fn every_day_moved() -> String {
    a_message(
        "REQUEST",
        20,
        "SEQUENCE:4\r\n\
DTSTART;TZID=Europe/London:20260305T100000\r\n\
DTEND;TZID=Europe/London:20260305T110000\r\n\
RRULE:FREQ=WEEKLY;COUNT=10\r\n",
    )
}

fn written_out_in_full() -> DateSettings {
    DateSettings {
        style: DateStyle::Absolute,
        order: DateOrder::DayFirst,
        wording: DateWording::Numeric,
        clock: Clock::TwentyFourHour,
    }
}

/// How a start in London is said on this computer's clock, the words the
/// sentences use, so the cases read the same wherever they run.
fn said_at(starts: &str) -> String {
    invitations::when_it_starts(starts, false, Some(LONDON), written_out_in_full())
}

fn reading() -> Reading {
    Reading {
        dates: written_out_in_full(),
        now: chrono::Local::now(),
    }
}

/// Sam, whom the messages name, with changes allowed.
fn sam(_account: &str) -> AnsweringAs {
    AnsweringAs {
        address: "sam@example.com".to_string(),
        allowed: Allowed::EVERYTHING,
    }
}

// ── The store ─────────────────────────────────────────────────────────────

/// The weekly sync, Thursdays at nine to ten in London, ten times from 5
/// March, filed in `calendar`.
fn the_series(calendar: &str) -> CalendarEventEntry {
    CalendarEventEntry {
        id: "evt-series".to_string(),
        account_id: ACCOUNT.to_string(),
        provider_event_id: Some(THE_MEETING.to_string()),
        calendar_id: Some(calendar.to_string()),
        summary: "Weekly sync".to_string(),
        description: None,
        location: None,
        start_datetime: "2026-03-05T09:00:00".to_string(),
        end_datetime: "2026-03-05T10:00:00".to_string(),
        start_date: None,
        end_date: None,
        is_all_day: false,
        time_zone: Some(LONDON.to_string()),
        status: "confirmed".to_string(),
        recurrence_rule: Some("FREQ=WEEKLY;COUNT=10".to_string()),
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

/// A store with one account's inbox, and the series on a calendar made on
/// this computer.
fn a_store_holding_the_series(dir: &tempfile::TempDir) -> MessageCache {
    a_store_holding(dir, &the_series("cal"))
}

/// A store with one account's inbox and `series` on its calendar, Ada
/// recorded as who called it.
fn a_store_holding(dir: &tempfile::TempDir, series: &CalendarEventEntry) -> MessageCache {
    let cache = MessageCache::new(dir.path().join("series.db"), None).expect("a store");
    cache
        .save_folder(&CachedFolder {
            id: 0,
            account_id: ACCOUNT.to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "Inbox".to_string(),
            unread_count: 0,
            total_count: 0,
        })
        .expect("an inbox");
    cache.save_calendar_event(series).expect("the series filed");
    cache
        .remember_where_it_came_from("evt-series", Some(THE_MEETING), Some("ada@example.com"))
        .expect("the organiser remembered");
    cache
}

/// A Google calendar on the account.
fn a_google_calendar() -> CalendarContainer {
    CalendarContainer {
        id: "cal-g".to_string(),
        account_id: ACCOUNT.to_string(),
        name: "Sam".to_string(),
        color: String::new(),
        source_provider: Some("gmail".to_string()),
        caldav_url: None,
        subscription_url: None,
        is_default: true,
        is_visible: true,
        is_read_only: false,
        display_order: 0,
        etag: None,
        ctag: None,
        sync_token: None,
        refresh_interval_minutes: None,
        created_at: String::new(),
        updated_at: String::new(),
    }
}

fn the_series_now(cache: &MessageCache) -> CalendarEventEntry {
    cache
        .get_event_by_id("evt-series")
        .expect("the calendar to be readable")
        .expect("the series to be on the calendar")
}

/// The row standing for the twelfth, when there is one.
fn the_twelfth_kept_apart(cache: &MessageCache) -> Option<CalendarEventEntry> {
    cache
        .the_day_of_a_meeting(ACCOUNT, THE_MEETING, "2026-03-12T09:00:00")
        .expect("the calendar to be readable")
}

/// A raw message opened: parsed, its row saved, its parts stored with their
/// bytes the way the reader's open path stores them.
fn opened(cache: &MessageCache, uid: u32, raw: &str) -> (MessageItem, MessageBody) {
    let parsed = mime::parse(raw.as_bytes()).expect("the message to parse");
    let row = cache
        .save_message(&CachedMessage {
            id: 0,
            uid,
            folder_id: 1,
            message_id: parsed.message_id.clone().unwrap_or_default(),
            subject: parsed.subject.clone(),
            from_addr: ADA.to_string(),
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
        from: ADA.to_string(),
        ..Default::default()
    };
    let body = MessageBody::Plain(parsed.body_plain.unwrap_or_default());
    (item, body)
}

/// Opened in a reader window: the change asked and applied first, then the
/// message composed with it folded in, the reader's own order.
fn opened_in_a_reader(
    cache: &MessageCache,
    item: &MessageItem,
    body: MessageBody,
) -> (MeetingChange, ReaderDocument) {
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
    let document = reader_text::single_message(item, &shown.body, reading()).with_what_is_said(
        &WhatIsSaidAboutIt {
            change: change.clone(),
            ..shown.said
        },
    );
    (change, document)
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

// ── The calendar, as its panel reads it ──────────────────────────────────

/// Every row the calendar panel would show from March to May, by the date it
/// starts on: the two calls the panel makes, in its order.
fn rows_by_date(cache: &MessageCache) -> HashMap<String, usize> {
    let entries = cache
        .events_that_could_fall_between(ACCOUNT, "2026-03-01T00:00:00Z", "2026-05-31T23:59:59Z")
        .expect("the calendar to be readable");
    let from = chrono::NaiveDate::from_ymd_opt(2026, 3, 1).expect("a date");
    let to = chrono::NaiveDate::from_ymd_opt(2026, 5, 31).expect("a date");
    let mut found = HashMap::new();
    for row in CalendarEventItem::every_day_shown(&entries, from, to) {
        *found.entry(row.start[..10].to_string()).or_insert(0) += 1;
    }
    found
}

fn on(found: &HashMap<String, usize>, date: &str) -> usize {
    found.get(date).copied().unwrap_or(0)
}

/// The Thursdays of the series other than the twelfth.
const THE_OTHER_THURSDAYS: [&str; 9] = [
    "2026-03-05",
    "2026-03-19",
    "2026-03-26",
    "2026-04-02",
    "2026-04-09",
    "2026-04-16",
    "2026-04-23",
    "2026-04-30",
    "2026-05-07",
];

fn every_other_thursday_is_shown_once(found: &HashMap<String, usize>) {
    for thursday in THE_OTHER_THURSDAYS {
        assert_eq!(on(found, thursday), 1, "{thursday}: {found:?}");
    }
}

/// How many times the series names the twelfth among its called-off days.
fn times_the_twelfth_is_called_off(series: &CalendarEventEntry) -> usize {
    series
        .exception_dates
        .as_deref()
        .unwrap_or_default()
        .split(',')
        .filter(|day| day.trim() == "20260312T090000")
        .count()
}

// ── The cases ─────────────────────────────────────────────────────────────

#[test]
fn test_one_day_moved_by_the_organiser_leaves_the_other_days_where_they_were() {
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store_holding_the_series(&dir);
    let (item, body) = opened(&cache, 1, &one_day_moved(1, 14));

    let (_, document) = opened_in_a_reader(&cache, &item, body);

    let found = rows_by_date(&cache);
    assert_eq!(on(&found, "2026-03-12"), 0, "{found:?}");
    assert_eq!(on(&found, "2026-03-13"), 1, "{found:?}");
    every_other_thursday_is_shown_once(&found);
    let that_day = the_twelfth_kept_apart(&cache).expect("the twelfth kept apart");
    assert_eq!(that_day.cut_from_event_id.as_deref(), Some("evt-series"));
    assert_eq!(that_day.start_datetime, "2026-03-13T14:00:00");
    assert_eq!(that_day.recurrence_rule, None);
    let series = the_series_now(&cache);
    assert_eq!(times_the_twelfth_is_called_off(&series), 1, "{series:?}");
    assert!(
        series.pending && that_day.pending,
        "a change nobody sends never leaves"
    );
    let spoken = spoken_as_it_opens(&document);
    let moved = format!(
        "Moved one day of this repeating meeting on your calendar, from {} to {}.",
        said_at("2026-03-12T09:00:00"),
        said_at("2026-03-13T14:00:00")
    );
    assert!(spoken.contains(&moved), "{spoken:?}");
}

#[test]
fn test_the_same_update_opened_again_moves_nothing_more() {
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store_holding_the_series(&dir);
    let (item, body) = opened(&cache, 1, &one_day_moved(1, 14));
    opened_in_a_reader(&cache, &item, body.clone());
    let kept_first = the_twelfth_kept_apart(&cache).expect("the twelfth kept apart");

    let (again, _) = opened_in_a_reader(&cache, &item, body);

    assert_eq!(again, MeetingChange::Nothing);
    let found = rows_by_date(&cache);
    assert_eq!(on(&found, "2026-03-13"), 1, "{found:?}");
    assert_eq!(
        the_twelfth_kept_apart(&cache).map(|row| row.id),
        Some(kept_first.id)
    );
    assert_eq!(times_the_twelfth_is_called_off(&the_series_now(&cache)), 1);
}

#[test]
fn test_a_second_update_for_that_day_moves_its_row_and_cuts_nothing_twice() {
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store_holding_the_series(&dir);
    let (first, body) = opened(&cache, 1, &one_day_moved(1, 14));
    opened_in_a_reader(&cache, &first, body);
    let kept_first = the_twelfth_kept_apart(&cache).expect("the twelfth kept apart");
    let (second, body) = opened(&cache, 2, &one_day_moved(2, 16));

    let (change, _) = opened_in_a_reader(&cache, &second, body);

    let kept = the_twelfth_kept_apart(&cache).expect("the twelfth still kept apart");
    assert_eq!(kept.id, kept_first.id, "the same row moves");
    assert_eq!(kept.start_datetime, "2026-03-13T16:00:00");
    assert!(
        matches!(&change, MeetingChange::Move { event_id, .. } if *event_id == kept.id),
        "{change:?}"
    );
    let found = rows_by_date(&cache);
    assert_eq!(on(&found, "2026-03-12"), 0, "{found:?}");
    assert_eq!(on(&found, "2026-03-13"), 1, "{found:?}");
    every_other_thursday_is_shown_once(&found);
    assert_eq!(times_the_twelfth_is_called_off(&the_series_now(&cache)), 1);
}

#[test]
fn test_a_cancellation_of_a_moved_day_offers_that_days_row() {
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store_holding_the_series(&dir);
    let (moved, body) = opened(&cache, 1, &one_day_moved(1, 14));
    opened_in_a_reader(&cache, &moved, body);
    let kept = the_twelfth_kept_apart(&cache).expect("the twelfth kept apart");
    let (called_off, body) = opened(&cache, 2, &one_day_called_off());

    let (change, _) = opened_in_a_reader(&cache, &called_off, body);

    assert_eq!(change, MeetingChange::OfferRemoval { event_id: kept.id });
}

#[test]
fn test_one_day_on_a_google_series_changes_nothing_and_says_so() {
    // Google is never told a repeat on a change, so the day kept apart would
    // reach it as an extra meeting while the day called off never would.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store_holding(&dir, &the_series("cal-g"));
    cache
        .save_calendar(&a_google_calendar())
        .expect("a Google calendar");
    let (item, body) = opened(&cache, 1, &one_day_moved(1, 14));

    let (_, document) = opened_in_a_reader(&cache, &item, body);

    let found = rows_by_date(&cache);
    assert_eq!(on(&found, "2026-03-12"), 1, "{found:?}");
    assert_eq!(on(&found, "2026-03-13"), 0, "{found:?}");
    assert_eq!(the_twelfth_kept_apart(&cache), None);
    assert!(!the_series_now(&cache).pending, "nothing to send");
    let spoken = spoken_as_it_opens(&document);
    assert!(
        spoken.iter().any(|line| line
            == "Your calendar was not changed, because one day of a repeating meeting cannot \
                be changed on its own in your Google calendar from here."),
        "{spoken:?}"
    );
}

#[test]
fn test_an_update_to_every_day_moves_the_series_and_keeps_it_repeating() {
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store_holding_the_series(&dir);
    let (item, body) = opened(&cache, 1, &every_day_moved());

    let (change, _) = opened_in_a_reader(&cache, &item, body);

    assert!(matches!(change, MeetingChange::Move { .. }), "{change:?}");
    let series = the_series_now(&cache);
    assert_eq!(series.start_datetime, "2026-03-05T10:00:00");
    assert_eq!(
        series.recurrence_rule.as_deref(),
        Some("FREQ=WEEKLY;COUNT=10")
    );
    let found = rows_by_date(&cache);
    assert_eq!(on(&found, "2026-03-12"), 1, "{found:?}");
    every_other_thursday_is_shown_once(&found);
}

#[test]
fn test_the_preview_changes_nothing_for_one_day() {
    // The preview composes a message as the cursor moves over it and asks
    // nothing of the calendar: arrowing past a message changes nothing.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store_holding_the_series(&dir);
    let (item, body) = opened(&cache, 1, &one_day_moved(1, 14));
    let before = the_series_now(&cache);

    reading_a_message::for_message(
        Some(&cache),
        item.message_id,
        &item.from,
        body,
        written_out_in_full,
        sam,
    );

    assert_eq!(the_series_now(&cache), before);
    assert_eq!(the_twelfth_kept_apart(&cache), None);
}

#[test]
fn test_the_invitations_sentence_names_the_day() {
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store_holding_the_series(&dir);
    let (item, body) = opened(&cache, 1, &one_day_called_off());

    let (_, document) = opened_in_a_reader(&cache, &item, body);

    let spoken = spoken_as_it_opens(&document);
    let cancelled = format!(
        "Meeting cancelled: Weekly sync, one day of a repeating meeting, {}. It is on your \
         calendar.",
        said_at("2026-03-12T09:00:00")
    );
    assert!(spoken.contains(&cancelled), "{spoken:?}");
}

// ── The window ────────────────────────────────────────────────────────────
//
// Another copy of the MSAA reading `a_meeting_change_reaches_the_calendar.rs`
// holds, for the reason that file gives for its own copies: nothing here
// reaches into another target.

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const VT_I4: u16 = 3;
const CHILDID_SELF: i64 = 0;
const ROLE_SYSTEM_PUSHBUTTON: i64 = 0x2b;

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
        Variant::child(0)
    }
}

type Hresult = i32;
type ReleaseFn = unsafe extern "system" fn(*mut c_void) -> u32;
type GetVariantFn = unsafe extern "system" fn(*mut c_void, Variant, *mut Variant) -> Hresult;
type GetBstrFn = unsafe extern "system" fn(*mut c_void, Variant, *mut *mut u16) -> Hresult;

// IAccessible's vtable: IUnknown (3), IDispatch (4), then get_accName (10),
// get_accDescription (12), get_accRole (13).
const VTBL_RELEASE: usize = 2;
const VTBL_GET_ACC_NAME: usize = 10;
const VTBL_GET_ACC_DESCRIPTION: usize = 12;
const VTBL_GET_ACC_ROLE: usize = 13;

#[link(name = "user32")]
unsafe extern "system" {
    fn EnumChildWindows(
        parent: isize,
        callback: extern "system" fn(isize, isize) -> i32,
        lparam: isize,
    ) -> i32;
    fn GetClassNameW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
    fn GetWindowTextW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
}

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
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
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
        let release: ReleaseFn = std::mem::transmute(vtable_entry(object, VTBL_RELEASE));
        release(object);
        Ok(Msaa {
            name,
            description,
            role: match hr_role >= 0 && role.vt == VT_I4 {
                true => role.val & 0xFFFF_FFFF,
                false => -1,
            },
        })
    }
}

/// One button of the tab, as Windows holds it.
#[derive(Debug, Clone, PartialEq)]
struct AButton {
    text: String,
    msaa: Msaa,
}

fn the_buttons_under(parent: isize) -> Result<Vec<AButton>, String> {
    descendants_of(parent)
        .into_iter()
        .filter(|hwnd| class_name(*hwnd) == "Button")
        .map(|hwnd| {
            Ok(AButton {
                text: window_text(hwnd),
                msaa: msaa_of(hwnd)?,
            })
        })
        .collect()
}

/// What the one window session of this process saw.
#[derive(Debug)]
struct Harvest {
    /// What opening the cancellation offered for removal.
    offered: Option<Removal>,
    /// The buttons of the cancellation's tab in the text reader.
    buttons: Vec<AButton>,
    /// What the status line was told after Remove from Calendar was pressed.
    said_after_the_press: Vec<String>,
    /// The series after the press.
    series_after_the_press: CalendarEventEntry,
    /// The calendar, by date, after the press.
    rows_after_the_press: HashMap<String, usize>,
}

/// The buttons of the tab, or why they could not be read.
type TheButtonsRead = Result<Vec<AButton>, String>;

fn take_the_harvest() -> Result<Harvest, String> {
    if std::mem::size_of::<Variant>() != 24 {
        return Err("VARIANT is not 24 bytes here, so the reader's layout is wrong".to_string());
    }
    let dir = tempfile::tempdir().map_err(|e| format!("no temporary folder: {e}"))?;
    // Shared the way the main window shares its store with the handler, which
    // takes it as that; it never leaves this thread.
    let cache: Arc<MessageCache> = a_store_holding_the_series(&dir).into();
    let (item, body) = opened(&cache, 1, &one_day_called_off());
    let (_, document) = opened_in_a_reader(&cache, &item, body);
    let offered = document.removal.clone();
    let runtime = Arc::new(tokio::runtime::Runtime::new().map_err(|e| format!("no runtime: {e}"))?);
    let (ui_tx, ui_rx) = async_channel::unbounded::<UIUpdate>();

    let outcome: Arc<Mutex<Option<TheButtonsRead>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        let cache = Some(cache.clone());
        let runtime = runtime.clone();
        wxdragon::main(move |app| {
            let settle = move |taken: TheButtonsRead| {
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
            let reader = ReaderWindow::new(&parent, &a11y);
            reader.wire_menu();
            // The main window's own handler, as the reader window is wired
            // with it when the program runs.
            reader.on_remove(move |removal| {
                wx_app::take_the_called_off_meeting_off_the_calendar(
                    &ui_tx, &runtime, &cache, removal,
                );
            });
            let offered = document.removal.clone();
            let tab = reader.open(document);
            let buttons = the_buttons_under(tab.panel.get_handle() as isize);
            if let Some(removal) = &offered {
                reader.remove_now(removal);
            }
            settle(buttons);
            std::mem::forget(reader);
        })
    };
    if let Err(why) = result {
        return Err(format!("wxdragon::main returned {why:?}"));
    }
    let buttons = outcome
        .lock()
        .map_err(|_| "the harvest's lock was poisoned".to_string())?
        .take()
        .unwrap_or_else(|| Err("the window session ended without a harvest".to_string()))?;
    // What the handler said, sent from the runtime; a handler that says
    // nothing leaves this empty rather than waiting for ever.
    let said_after_the_press = runtime.block_on(async {
        let mut said = Vec::new();
        while let Ok(Ok(update)) =
            tokio::time::timeout(std::time::Duration::from_secs(2), ui_rx.recv()).await
        {
            match update {
                UIUpdate::StatusUpdated(line) => said.push(line),
                UIUpdate::CommandRefused(line) => said.push(format!("refused: {line}")),
                _ => {}
            }
        }
        said
    });
    Ok(Harvest {
        offered,
        buttons,
        said_after_the_press,
        series_after_the_press: the_series_now(&cache),
        rows_after_the_press: rows_by_date(&cache),
    })
}

/// The one window session of this process, taken by whichever case asks first.
fn the_harvest() -> &'static Harvest {
    static HARVEST: OnceLock<Result<Harvest, String>> = OnceLock::new();
    match HARVEST.get_or_init(take_the_harvest) {
        Ok(harvest) => harvest,
        Err(why) => panic!("the window session could not be read: {why}"),
    }
}

#[test]
fn test_a_cancelled_day_is_offered_one_button_described_as_that_day() {
    let harvest = the_harvest();
    let Some(Removal::OneDay { when, .. }) = &harvest.offered else {
        panic!(
            "the cancellation offered {:?}, not one day",
            harvest.offered
        );
    };
    let [button] = harvest.buttons.as_slice() else {
        panic!(
            "{} buttons, not the one: {:?}",
            harvest.buttons.len(),
            harvest.buttons
        );
    };

    assert_eq!(button.msaa.name, "Remove from Calendar");
    assert_eq!(button.text, "&Remove from Calendar");
    assert_eq!(button.msaa.role, ROLE_SYSTEM_PUSHBUTTON);
    assert_eq!(
        button.msaa.description,
        format!(
            "Takes {when} off this repeating meeting on your calendar. Nothing is sent to the \
             organiser."
        )
    );
}

#[test]
fn test_pressing_it_calls_that_day_off_and_keeps_the_series() {
    let harvest = the_harvest();
    let series = &harvest.series_after_the_press;

    assert_eq!(
        series.status, "confirmed",
        "the series itself is not called off"
    );
    assert_eq!(times_the_twelfth_is_called_off(series), 1, "{series:?}");
    assert!(series.pending, "a day called off nobody sends never leaves");
    assert_eq!(on(&harvest.rows_after_the_press, "2026-03-12"), 0);
    every_other_thursday_is_shown_once(&harvest.rows_after_the_press);
    assert_eq!(
        harvest.said_after_the_press,
        vec!["Weekly sync: that one day is taken off. The other days are unchanged.".to_string()]
    );
}

// ── Read from the source ──────────────────────────────────────────────────

/// The main window's source as it ships, without its tests.
fn the_main_window() -> String {
    const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";
    wixen_mail::common::what_ships::what_ships(
        &std::fs::read_to_string(THE_MAIN_WINDOW)
            .unwrap_or_else(|e| panic!("{THE_MAIN_WINDOW}: {e}"))
            .replace("\r\n", "\n"),
    )
}

/// The handler's arm for one day: from the last `Removal::OneDay` in the
/// handler to the handler's end.
fn the_one_day_arm(app: &str) -> Result<String, String> {
    let signature = "fn take_the_called_off_meeting_off_the_calendar(";
    let at = app
        .find(signature)
        .ok_or(format!("{signature} is gone, so this reads nothing"))?;
    let rest = &app[at..];
    let body = &rest[..rest.find("\n}\n").map_or(rest.len(), |end| end + 2)];
    let arm = body
        .rfind("Removal::OneDay")
        .ok_or("the removal handler has no arm for one day")?;
    Ok(body[arm..].to_string())
}

/// What is wrong with the one-day arm: it must call the day off the series,
/// and never delete a row or mark the whole meeting cancelled.
fn what_the_one_day_arm_does_wrong(arm: &str) -> Vec<&'static str> {
    let mut wrong = Vec::new();
    if !arm.contains("one_day_called_off(") {
        wrong.push("the one-day arm does not call the day off its series");
    }
    if arm.contains("delete_calendar_event(") {
        wrong.push("the one-day arm deletes a row");
    }
    if arm.contains("mark_the_meeting_called_off(") {
        wrong.push("the one-day arm marks the whole meeting cancelled");
    }
    wrong
}

#[test]
fn test_the_one_day_arm_calls_the_day_off_and_deletes_nothing() {
    let arm = the_one_day_arm(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));

    assert_eq!(what_the_one_day_arm_does_wrong(&arm), Vec::<&str>::new());
    // The reading can see both ways the arm goes wrong.
    for planted in ["delete_calendar_event(", "mark_the_meeting_called_off("] {
        let with_it = format!("{arm}\n    cache.{planted}&series_id);");
        assert!(
            !what_the_one_day_arm_does_wrong(&with_it).is_empty(),
            "{planted} planted in the arm was passed over"
        );
    }
}

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
//! # What this cannot see
//!
//! Whether Google or Outlook apply an organiser's change to one day to the
//! guest's copy themselves, and what a calendar server makes of the pair a
//! day kept apart sends. How the sentences sound is the tester's ear.

#![cfg(windows)]

use std::collections::HashMap;
use wixen_mail::application::allowed::Allowed;
use wixen_mail::application::invitations;
use wixen_mail::application::meeting_changes::MeetingChange;
use wixen_mail::application::reading_a_message::{self, AnsweringAs, WhatIsSaidAboutIt};
use wixen_mail::common::types::MessageBody;
use wixen_mail::data::message_cache::attachment_content::AttachmentWithContent;
use wixen_mail::data::message_cache::{
    CachedFolder, CachedMessage, CalendarContainer, CalendarEventEntry, MessageCache,
};
use wixen_mail::presentation::date_display::{
    Clock, DateOrder, DateSettings, DateStyle, DateWording,
};
use wixen_mail::presentation::read_aloud::Reading;
use wixen_mail::presentation::reader_text::{self, ReaderDocument};
use wixen_mail::presentation::ui_types::{CalendarEventItem, MessageItem};
use wixen_mail::service::mime;

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

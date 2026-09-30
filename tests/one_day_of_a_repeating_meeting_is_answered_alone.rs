//! Answering one day of a repeating meeting answers that day and no other,
//! from the bytes of the invitation to the reply and the rows the calendar
//! shows (GAP-04, ledger 638, 13-36.4).
//!
//! The way `one_day_of_a_repeating_meeting_is_changed_alone.rs` does it: a raw
//! message parsed and its parts stored the way opening a message stores them,
//! then the calls the Accept, Tentative and Decline buttons make, in the
//! handler's order: `whether_it_can_be_answered`, `the_answer_to_send`, and
//! `file_the_answer`. The reply is read as `the_answer_to_send` builds it,
//! which is the document the handler writes into the outbox; the queue itself
//! lives in the main window's state and is not started here. The calendar is
//! read back the way its panel reads it, by date.
//!
//! The invitation for one day is shaped the way RFC 5546 section 3.2.2
//! describes a request for one instance, which is the shape Outlook sends:
//! the series' UID, a `RECURRENCE-ID` naming the day with its zone, and the
//! day's own start and end. It is written from the standard and not taken
//! from a real Outlook.
//!
//! # What this cannot see
//!
//! Whether the organiser's program reads the reply as an answer to that day
//! alone, and what a calendar server makes of the day kept apart. Both are
//! phase 14's, on the ledger. How the sentences sound is the tester's ear.

use std::collections::HashMap;
use wixen_mail::application::allowed::Allowed;
use wixen_mail::application::answered_meetings::{self, Filed};
use wixen_mail::application::answering::{self, AnswerButtons, HowItWent};
use wixen_mail::application::invitations::{self, Answer};
use wixen_mail::application::reading_a_message::{self, AnsweringAs};
use wixen_mail::application::sending_later::{GoAfter, Hold, WhenItGoes};
use wixen_mail::data::message_cache::attachment_content::AttachmentWithContent;
use wixen_mail::data::message_cache::{
    CachedFolder, CachedMessage, CalendarContainer, CalendarEventEntry, MessageCache,
};
use wixen_mail::presentation::date_display::{
    Clock, DateOrder, DateSettings, DateStyle, DateWording,
};
use wixen_mail::presentation::ui_types::CalendarEventItem;
use wixen_mail::service::mime;

// ── The messages ──────────────────────────────────────────────────────────

const ADA: &str = "Ada Lovelace <ada@example.com>";
const THE_MEETING: &str = "s-1@example.com";
const ACCOUNT: &str = "acc-1";
const THE_TWELFTH: &str = "2026-03-12T09:00:00";

/// A message from Ada asking Sam, carrying `lines`, the meeting's own lines,
/// as a calendar program sends an invitation: a covering note and the meeting
/// as a second alternative.
fn an_invitation(number: u32, lines: &str) -> String {
    format!(
        "From: {ADA}\r\n\
To: sam@example.com\r\n\
Subject: Weekly sync\r\n\
Message-ID: <invite-{number}@example.com>\r\n\
Date: Tue, 3 Mar 2026 10:00:00 +0000\r\n\
MIME-Version: 1.0\r\n\
Content-Type: multipart/alternative; boundary=\"alt\"\r\n\
\r\n\
--alt\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
Are you free?\r\n\
--alt\r\n\
Content-Type: text/calendar; charset=utf-8; method=REQUEST\r\n\
\r\n\
BEGIN:VCALENDAR\r\n\
VERSION:2.0\r\n\
PRODID:-//Example//EN\r\n\
METHOD:REQUEST\r\n\
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

/// The weekly sync, Thursdays at nine to ten in London, ten times from 5
/// March, at version nought.
fn the_series_invitation() -> String {
    an_invitation(
        1,
        "SEQUENCE:0\r\n\
DTSTART;TZID=Europe/London:20260305T090000\r\n\
DTEND;TZID=Europe/London:20260305T100000\r\n\
RRULE:FREQ=WEEKLY;COUNT=10\r\n",
    )
}

/// Ada's invitation to the Thursday of 12 March alone, at its own time.
fn the_invitation_for_the_twelfth() -> String {
    an_invitation(
        2,
        "SEQUENCE:1\r\n\
RECURRENCE-ID;TZID=Europe/London:20260312T090000\r\n\
DTSTART;TZID=Europe/London:20260312T090000\r\n\
DTEND;TZID=Europe/London:20260312T100000\r\n",
    )
}

/// The calendar document a raw message carries.
fn the_document_in(raw: &str) -> String {
    let at = raw.find("BEGIN:VCALENDAR").expect("a calendar document");
    let ends = raw.find("END:VCALENDAR").expect("the document to end") + "END:VCALENDAR\r\n".len();
    raw[at..ends].to_string()
}

fn written_out_in_full() -> DateSettings {
    DateSettings {
        style: DateStyle::Absolute,
        order: DateOrder::DayFirst,
        wording: DateWording::Numeric,
        clock: Clock::TwentyFourHour,
    }
}

/// Sam, whom the invitations name, allowed to send.
fn sam(_account: &str) -> AnsweringAs {
    AnsweringAs {
        address: "sam@example.com".to_string(),
        allowed: Allowed::EVERYTHING,
    }
}

// ── The store ─────────────────────────────────────────────────────────────

/// A store with one account's inbox and nothing on its calendar.
fn an_empty_store(dir: &tempfile::TempDir) -> MessageCache {
    let cache = MessageCache::new(dir.path().join("answers.db"), None).expect("a store");
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
    cache
}

/// A store whose calendar kept here holds the weekly sync, put there the way
/// it gets there: its invitation answered Accept for every day.
fn a_store_holding_the_answered_series(dir: &tempfile::TempDir) -> MessageCache {
    let cache = an_empty_store(dir);
    let series = opened(&cache, 1, &the_series_invitation());
    answered(&cache, series, Answer::Accepted);
    cache
}

/// A Google calendar on the account, holding the weekly sync, with Ada
/// recorded as who called it.
fn a_store_with_the_series_at_google(dir: &tempfile::TempDir) -> MessageCache {
    let cache = an_empty_store(dir);
    cache
        .save_calendar(&CalendarContainer {
            id: "cal-g".to_string(),
            account_id: ACCOUNT.to_string(),
            name: "Sam".to_string(),
            color: String::new(),
            source_provider: Some(wixen_mail::application::calendar::GOOGLE.to_string()),
            caldav_url: None,
            subscription_url: None,
            is_default: false,
            is_visible: true,
            is_read_only: false,
            display_order: 0,
            etag: None,
            ctag: None,
            sync_token: None,
            refresh_interval_minutes: None,
            created_at: String::new(),
            updated_at: String::new(),
        })
        .expect("the Google calendar kept");
    cache
        .save_calendar_event(&CalendarEventEntry {
            id: "evt-series".to_string(),
            account_id: ACCOUNT.to_string(),
            provider_event_id: Some("google-series".to_string()),
            calendar_id: Some("cal-g".to_string()),
            summary: "Weekly sync".to_string(),
            description: None,
            location: None,
            start_datetime: "2026-03-05T09:00:00".to_string(),
            end_datetime: "2026-03-05T10:00:00".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: Some("Europe/London".to_string()),
            status: "confirmed".to_string(),
            recurrence_rule: Some("FREQ=WEEKLY;COUNT=10".to_string()),
            categories: String::new(),
            source_provider: Some(wixen_mail::application::calendar::GOOGLE.to_string()),
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
        })
        .expect("Google's copy of the series filed");
    cache
        .remember_where_it_came_from("evt-series", Some(THE_MEETING), Some("ada@example.com"))
        .expect("where it came from remembered");
    cache
}

/// A raw message opened: parsed, its row saved, its parts stored with their
/// bytes the way the reader's open path stores them. Its row is returned.
fn opened(cache: &MessageCache, uid: u32, raw: &str) -> i64 {
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
    row
}

/// What answering did: the reply the handler would queue, and what filing
/// it on the calendar answered.
struct Answered {
    reply: String,
    filed: Filed,
}

/// The message on `row` answered the way pressing a button answers it: the
/// invitation read from the message's own row, every reason not to answer
/// asked, the reply built, and the answer filed while it is held.
fn answered(cache: &MessageCache, row: i64, answer: Answer) -> Answered {
    let found = answered_meetings::the_invitation_on(cache, row)
        .expect("the message to be readable")
        .expect("the message to carry an invitation");
    let ready = answering::whether_it_can_be_answered(
        &found.document,
        "sam@example.com",
        Allowed::EVERYTHING,
    )
    .unwrap_or_else(|why| panic!("the invitation could not be answered: {}", why.why()));
    let sending = ready
        .the_answer_to_send(
            answer,
            chrono::Utc::now(),
            &found.message_id,
            found.references.as_deref(),
        )
        .expect("the answer to be built");
    let filed = answered_meetings::file_the_answer(
        cache,
        &found.account,
        &ready,
        answer,
        &HowItWent::Queued {
            goes: WhenItGoes::WhenItsTimeComes,
            waiting_on: GoAfter::held(Hold::DEFAULT, chrono::Local::now()),
        },
    )
    .expect("the answer to be filed");
    Answered {
        reply: sending.calendar_document,
        filed,
    }
}

/// The buttons the message on `row` is offered, the way either reader window
/// asks for them.
fn the_buttons_on(cache: &MessageCache, row: i64) -> answering::TheButtons {
    match reading_a_message::answer_buttons_among(cache, row, &[], written_out_in_full, sam) {
        AnswerButtons::Offered(buttons) => buttons,
        other => panic!("the invitation was not offered its buttons: {other:?}"),
    }
}

/// The row standing for the twelfth, when there is one.
fn the_twelfth(cache: &MessageCache) -> Option<CalendarEventEntry> {
    cache
        .the_day_of_a_meeting(ACCOUNT, THE_MEETING, THE_TWELFTH)
        .expect("the calendar to be readable")
}

/// The whole meeting, the row found by the meeting's UID.
fn the_series_now(cache: &MessageCache) -> CalendarEventEntry {
    cache
        .get_event_by_ical_uid(ACCOUNT, THE_MEETING)
        .expect("the calendar to be readable")
        .expect("the series to be on the calendar")
}

/// The lines of a reply that name a day.
fn the_days_named_in(reply: &str) -> Vec<&str> {
    reply
        .lines()
        .filter(|line| line.starts_with("RECURRENCE-ID"))
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

/// The ten Thursdays of the series.
const EVERY_THURSDAY: [&str; 10] = [
    "2026-03-05",
    "2026-03-12",
    "2026-03-19",
    "2026-03-26",
    "2026-04-02",
    "2026-04-09",
    "2026-04-16",
    "2026-04-23",
    "2026-04-30",
    "2026-05-07",
];

// ── The cases ─────────────────────────────────────────────────────────────

#[test]
fn test_one_day_is_offered_the_three_buttons_each_saying_one_day() {
    // The tracer: the invitation for one day reaches the three buttons, and
    // each says that it answers one day, before anything is sent.
    let dir = tempfile::tempdir().expect("a folder");
    let cache = a_store_holding_the_answered_series(&dir);
    let row = opened(&cache, 2, &the_invitation_for_the_twelfth());
    let when = invitations::when_the_invitation_is(
        &invitations::read_the_invitation(&the_document_in(&the_invitation_for_the_twelfth()))
            .expect("the invitation to read"),
        written_out_in_full(),
    );

    let buttons = the_buttons_on(&cache, row);

    assert_eq!(
        buttons.accept,
        format!("Accept one day of Weekly sync, {when}. Ada Lovelace will be told.")
    );
    assert_eq!(
        buttons.tentative,
        format!("Say you might come to one day of Weekly sync, {when}. Ada Lovelace will be told.")
    );
    assert_eq!(
        buttons.decline,
        format!("Decline one day of Weekly sync, {when}. Ada Lovelace will be told.")
    );
}

#[test]
fn test_the_answer_to_one_day_goes_out_naming_that_day() {
    // RFC 5546 section 3.2.3: a reply about one instance carries its
    // RECURRENCE-ID, or the organiser reads it for every day of the series.
    let dir = tempfile::tempdir().expect("a folder");
    let cache = a_store_holding_the_answered_series(&dir);
    let row = opened(&cache, 2, &the_invitation_for_the_twelfth());

    let went = answered(&cache, row, Answer::Declined);

    assert_eq!(
        the_days_named_in(&went.reply),
        vec!["RECURRENCE-ID;TZID=Europe/London:20260312T090000"],
        "{}",
        went.reply
    );
    assert_eq!(
        invitations::what_it_asks(&went.reply),
        invitations::WhatItAsks::SomebodysAnswer
    );
    assert!(went.reply.contains("PARTSTAT=DECLINED"), "{}", went.reply);
}

#[test]
fn test_declining_one_day_keeps_that_day_free_and_the_series_busy() {
    let dir = tempfile::tempdir().expect("a folder");
    let cache = a_store_holding_the_answered_series(&dir);
    let row = opened(&cache, 2, &the_invitation_for_the_twelfth());

    let went = answered(&cache, row, Answer::Declined);

    assert_eq!(went.filed, Filed::OnTheCalendar);
    let found = rows_by_date(&cache);
    for thursday in EVERY_THURSDAY {
        assert_eq!(on(&found, thursday), 1, "{thursday}: {found:?}");
    }
    let the_day = the_twelfth(&cache).expect("the twelfth kept apart with the answer on it");
    assert_eq!(
        the_day.show_as, "free",
        "a declined day still takes up the time"
    );
    assert_eq!(the_day.recurrence_rule, None);
    assert_eq!(
        the_series_now(&cache).show_as,
        "busy",
        "declining one day freed every day of the series"
    );
}

#[test]
fn test_answering_that_day_again_is_said_to_replace_the_first() {
    let dir = tempfile::tempdir().expect("a folder");
    let cache = a_store_holding_the_answered_series(&dir);
    let row = opened(&cache, 2, &the_invitation_for_the_twelfth());
    answered(&cache, row, Answer::Declined);

    let buttons = the_buttons_on(&cache, row);

    assert!(
        buttons
            .accept
            .ends_with("You have already declined this, and this replaces that answer."),
        "{}",
        buttons.accept
    );
    assert!(
        buttons
            .decline
            .ends_with("You have already declined this, and this says the same again."),
        "{}",
        buttons.decline
    );
}

#[test]
fn test_one_day_answered_on_a_google_series_leaves_the_calendar_and_says_so() {
    // A day kept apart there would reach Google as an extra meeting, since
    // Google is not told how a meeting repeats when it changes.
    let dir = tempfile::tempdir().expect("a folder");
    let cache = a_store_with_the_series_at_google(&dir);
    let row = opened(&cache, 2, &the_invitation_for_the_twelfth());

    let went = answered(&cache, row, Answer::Accepted);

    assert_eq!(
        went.filed,
        Filed::LeftAsItIs(
            "That day on your calendar was left as it was, because one day of a repeating \
             meeting cannot be changed on its own in your Google calendar from here."
                .to_string()
        )
    );
    assert_eq!(the_twelfth(&cache), None, "a day was kept apart on Google");
    let series = the_series_now(&cache);
    assert_eq!(
        series.exception_dates, None,
        "the series at Google lost a day"
    );
    assert!(
        !series.pending,
        "the series at Google was left waiting to be sent"
    );
    assert_eq!(
        the_days_named_in(&went.reply).len(),
        1,
        "the answer itself still names the day: {}",
        went.reply
    );
}

#[test]
fn test_one_day_answered_with_no_series_here_is_filed_alone_and_found_again() {
    let dir = tempfile::tempdir().expect("a folder");
    let cache = an_empty_store(&dir);
    let row = opened(&cache, 2, &the_invitation_for_the_twelfth());

    let went = answered(&cache, row, Answer::Accepted);

    assert_eq!(went.filed, Filed::OnTheCalendar);
    let alone = the_twelfth(&cache).expect("the day filed on its own and linked to its day");
    assert_eq!(alone.recurrence_rule, None);
    assert_eq!(alone.provider_event_id, None);
    assert_eq!(alone.show_as, "busy");
    assert_eq!(on(&rows_by_date(&cache), "2026-03-12"), 1);
    assert!(
        the_buttons_on(&cache, row)
            .accept
            .ends_with("You have already accepted this, and this says the same again."),
        "the answer filed on its own is not found again for that day"
    );
}

#[test]
fn test_a_day_filed_alone_is_never_taken_for_an_answers_whole_meeting() {
    // 13-36.2's read, which a calendar check asks before filing a provider's
    // copy of the whole meeting: a day filed alone is one day of it.
    let dir = tempfile::tempdir().expect("a folder");
    let cache = an_empty_store(&dir);
    let row = opened(&cache, 2, &the_invitation_for_the_twelfth());
    answered(&cache, row, Answer::Accepted);
    assert!(the_twelfth(&cache).is_some(), "nothing was filed to read");

    assert_eq!(
        cache
            .the_meeting_only_an_answer_filed(ACCOUNT, THE_MEETING)
            .expect("the calendar to be readable"),
        None
    );
}

#[test]
fn test_an_answered_series_still_repeats_on_the_calendar() {
    let dir = tempfile::tempdir().expect("a folder");
    let cache = a_store_holding_the_answered_series(&dir);

    let found = rows_by_date(&cache);

    for thursday in EVERY_THURSDAY {
        assert_eq!(on(&found, thursday), 1, "{thursday}: {found:?}");
    }
}

#[test]
fn test_a_calendar_left_as_it_was_is_said_after_the_answer_and_nothing_else_is() {
    // The sentence after answering is the answer's own, and the calendar's
    // only when it was left as it was; every other outcome is what pressing
    // the button promised, and saying it again is noise.
    let did = "Accepted one day of Weekly sync. Sending in 10 seconds.".to_string();
    let why = "That day on your calendar was left as it was, because one day of a repeating \
               meeting cannot be changed on its own in your Google calendar from here.";

    assert_eq!(
        answered_meetings::what_answering_said(
            did.clone(),
            Some(&Filed::LeftAsItIs(why.to_string()))
        ),
        format!("{did} {why}")
    );
    for filed in [Filed::OnTheCalendar, Filed::AlreadyAnswered, Filed::NotSent] {
        assert_eq!(
            answered_meetings::what_answering_said(did.clone(), Some(&filed)),
            did,
            "{filed:?}"
        );
    }
    assert_eq!(
        answered_meetings::what_answering_said(did.clone(), None),
        did
    );
}

// ── What the handler says ─────────────────────────────────────────────────
//
// Read from the source because the handler sends mail from an account, which
// this file does not start.

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

/// Whether the handler says what answering did and what the calendar did in
/// one status line, and nowhere else.
fn the_calendar_is_said_in_the_one_status_line(app: &str) -> Result<(), String> {
    let body = body_of(app, "fn answer_the_invitation(")?;
    let said = body.matches("send_status(").count();
    if said != 1 {
        return Err(format!(
            "answer_the_invitation says its outcome through send_status {said} times, so it is \
             heard more than once or not at all"
        ));
    }
    let status = &body[body.find("send_status(").unwrap_or_default()..];
    if !status.contains("what_answering_said(") {
        return Err(
            "the status line after answering does not carry what the calendar did, so a \
             calendar left as it was is never said"
                .to_string(),
        );
    }
    Ok(())
}

#[test]
fn test_the_handler_says_what_the_calendar_did_in_its_one_status_line() {
    the_calendar_is_said_in_the_one_status_line(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_reading_complains_when_the_calendar_is_said_a_second_time() {
    let app = shipped(THE_MAIN_WINDOW);
    let body = body_of(&app, "fn answer_the_invitation(").expect("the answer path");
    let planted_body = body.replacen(
        "send_status(",
        "send_status(ui_tx, runtime, \"That day was left as it was.\");\n    send_status(",
        1,
    );
    assert_ne!(planted_body, body, "the companion planted nothing");
    let why = the_calendar_is_said_in_the_one_status_line(&app.replacen(&body, &planted_body, 1))
        .expect_err("a second status line was passed over");
    assert!(why.contains("more than once"), "{why}");
}

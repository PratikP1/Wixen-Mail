//! A day of a repeating event that was moved, read back the way the panel reads it.
//!
//! `05-RESEARCH.md`'s assumption A1 says a moved occurrence is on the calendar
//! twice: once expanded from the series and once as the row it was cut out
//! into. It says plainly that this came from reading the code rather than from
//! running it. Reading the same code again on 2026-09-06 gave the opposite
//! answer, with a reason: every path that stores a moved day also writes an
//! EXDATE that takes that day off the series.
//!
//! Neither reading is evidence. This file runs it.
//!
//! It is an integration test because the question spans three layers and no
//! unit test can hold all three at once. `application::calendar` writes the
//! pair, `data::message_cache::calendar` asks which events could fall in the
//! window, and `presentation::ui_types` expands each of them across its days.
//! The de-duplication is not in any one of them: the writer puts the EXDATE on
//! and the expansion honours it, and only the two together decide how many rows
//! land on a date.
//!
//! What this cannot see. It stores a series and a moved day through the local
//! path, so it says nothing about a moved day arriving from Google or from
//! Outlook carrying a repeat rule of its own. Both of those build the row from
//! the payload, so no local round trip can reach either. It says nothing about
//! the case the running program already counts and announces: where a provider
//! does not say which day of the series a moved appointment stands in for,
//! there is no day to take off, and the appointment really is shown twice.

use wixen_mail::application::calendar::{
    WhoTookTheDayOut, one_day_called_off, one_day_kept_out_of_the_series,
};
use wixen_mail::data::message_cache::{CalendarEventEntry, MessageCache};
use wixen_mail::presentation::ui_types::CalendarEventItem;

/// The account every fixture here stores under.
const ACCOUNT: &str = "acct";

/// The window the rows are read over, wide enough to hold several weeks of the
/// series either side of the day that moves.
///
/// Fixed dates rather than a window counted from today, so this reads the same
/// in a year as it does now. `CalendarEventItem::the_window_now` is what the
/// panel uses and it is the clock; the arithmetic it does is already held to an
/// exact pair of dates by a test of its own.
const WINDOW_OPENS: &str = "2026-07-01T00:00:00Z";
const WINDOW_CLOSES: &str = "2026-09-30T23:59:59Z";

/// A weekly series starting on Monday 3 August 2026.
fn a_weekly_series() -> CalendarEventEntry {
    CalendarEventEntry {
        id: "series-1".to_string(),
        account_id: ACCOUNT.to_string(),
        provider_event_id: Some("uid-1".to_string()),
        calendar_id: Some("cal-1".to_string()),
        summary: "Stand-up".to_string(),
        description: None,
        location: None,
        start_datetime: "2026-08-03T09:00:00Z".to_string(),
        end_datetime: "2026-08-03T09:15:00Z".to_string(),
        start_date: None,
        end_date: None,
        is_all_day: false,
        time_zone: Some("Etc/UTC".to_string()),
        status: "confirmed".to_string(),
        recurrence_rule: Some("FREQ=WEEKLY".to_string()),
        categories: String::new(),
        source_provider: Some("caldav".to_string()),
        etag: None,
        web_link: None,
        show_as: "busy".to_string(),
        last_modified_remote: None,
        last_synced_at: None,
        attendees_json: None,
        reminders_json: None,
        created_at: String::new(),
        updated_at: String::new(),
        pending: false,
        exception_dates: None,
        cut_from_event_id: None,
        provider_recurrence_id: None,
    }
}

/// The one day of that series somebody moved, built the way
/// `presentation::managers::the_day_kept_on_its_own` builds one.
///
/// Its own identifier and its own provider identifier, because the store treats
/// an account, a calendar and a provider identifier as one row: sharing the
/// series' would write over the series rather than sit beside it.
fn the_day_it_moved_to(series: &CalendarEventEntry, start: &str, end: &str) -> CalendarEventEntry {
    CalendarEventEntry {
        id: "that-day".to_string(),
        provider_event_id: Some("uid-1-on-the-third".to_string()),
        start_datetime: start.to_string(),
        end_datetime: end.to_string(),
        recurrence_rule: None,
        cut_from_event_id: Some(series.id.clone()),
        ..series.clone()
    }
}

/// Every row the calendar panel would show, in the window, by the date it is on.
///
/// The same two calls the panel makes and in the same order: ask the store
/// which events could fall in the window, then expand each of them across the
/// days it lands on. Counting by date rather than by event, because how many
/// rows land on a date is the whole question and an event count cannot answer
/// it.
fn rows_by_date(cache: &MessageCache) -> std::collections::HashMap<String, usize> {
    let entries = cache
        .events_that_could_fall_between(ACCOUNT, WINDOW_OPENS, WINDOW_CLOSES)
        .expect("the calendar to be readable");
    let from = chrono::NaiveDate::parse_from_str(&WINDOW_OPENS[..10], "%Y-%m-%d")
        .expect("the window to open on a date");
    let to = chrono::NaiveDate::parse_from_str(&WINDOW_CLOSES[..10], "%Y-%m-%d")
        .expect("the window to close on a date");
    let mut found = std::collections::HashMap::new();
    for row in CalendarEventItem::every_day_shown(&entries, from, to) {
        *found.entry(row.start[..10].to_string()).or_insert(0) += 1;
    }
    found
}

/// How many rows landed on one date, said as a number so a missing date and a
/// date with no rows are the same answer.
fn on(found: &std::collections::HashMap<String, usize>, date: &str) -> usize {
    found.get(date).copied().unwrap_or(0)
}

#[test]
fn test_a_day_moved_out_of_a_series_is_on_the_calendar_once() {
    // Both answers to who moved it, because the only thing that argument
    // decides is whether the series is left waiting to be sent, and a day
    // moved by either has to leave one row on each date.
    for who in [
        WhoTookTheDayOut::SomebodyHere,
        WhoTookTheDayOut::TheProviderItself,
    ] {
        let dir = tempfile::tempdir().expect("a directory to work in");
        let cache = MessageCache::new(dir.path().join("calendar.db"), None)
            .expect("a calendar to store into");

        let series = a_weekly_series();
        cache
            .save_calendar_event(&series)
            .expect("the series to be stored");

        // The third Monday, not the first, so there are ordinary days of the
        // series either side of the one that moves. A day taken off the front
        // cannot tell a calendar that lost one day from one that lost every
        // day up to a point.
        //
        // It moves to a Wednesday, which the weekly rule never falls on, so a
        // row on that date can only be the moved day and a row on the Monday
        // can only be the series.
        let that_day = the_day_it_moved_to(&series, "2026-08-19T14:00:00Z", "2026-08-19T14:15:00Z");
        one_day_kept_out_of_the_series(&cache, &series, &that_day, "2026-08-17T09:00:00Z", who)
            .expect("the day and the series to be stored");

        let found = rows_by_date(&cache);

        assert_eq!(
            on(&found, "2026-08-17"),
            0,
            "the day it used to be on still has a row, moved by {who:?}: {found:?}"
        );
        assert_eq!(
            on(&found, "2026-08-19"),
            1,
            "the day it moved to does not have exactly one row, moved by {who:?}: {found:?}"
        );
        // The EXDATE took one day off the series and not the series itself.
        // Without this the two assertions above are satisfied by a calendar
        // that lost the whole series, which is a worse fault than the one this
        // file is about. Two dates before the day that moved and two after, so
        // a prefix or a suffix going missing is caught as well.
        for still_there in ["2026-08-03", "2026-08-10", "2026-08-24", "2026-08-31"] {
            assert_eq!(
                on(&found, still_there),
                1,
                "an ordinary day of the series is not on the calendar once, \
                 moved by {who:?}: {found:?}"
            );
        }
    }
}

#[test]
fn test_a_day_the_series_calls_off_is_on_no_date_at_all() {
    let dir = tempfile::tempdir().expect("a directory to work in");
    let cache =
        MessageCache::new(dir.path().join("calendar.db"), None).expect("a calendar to store into");

    let series = a_weekly_series();
    cache
        .save_calendar_event(&one_day_called_off(&series, "2026-08-10T09:00:00Z"))
        .expect("the series to be stored");

    let found = rows_by_date(&cache);

    assert_eq!(
        on(&found, "2026-08-10"),
        0,
        "a day the series called off is still on the calendar: {found:?}"
    );
    // The same guard as above, and for the same reason: a calendar that shows
    // nothing at all satisfies the assertion above and is not what was asked
    // for.
    for still_there in ["2026-08-03", "2026-08-17", "2026-08-24"] {
        assert_eq!(
            on(&found, still_there),
            1,
            "calling one day off took another day with it: {found:?}"
        );
    }
}

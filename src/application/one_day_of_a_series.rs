//! One day of a repeating meeting, as an organiser's message about that day
//! names it, set against the series the calendar holds.
//!
//! An organiser who moves or calls off one Thursday of a weekly meeting sends
//! a message naming that Thursday with `RECURRENCE-ID` (RFC 5545 section
//! 3.8.4.4, RFC 5546 section 3.2). The calendar holds the series as one row
//! and a rule, so the day has to be found on the series before anything is
//! done to it, and a day kept apart from its series already is a row of its
//! own that answers for that day from then on.
//!
//! # One spelling of the day
//!
//! The message writes the day in whatever zone the organiser's program
//! chose, and the series is stored in its own. The days a series calls off
//! are compared by clock face and zone, so a day named in universal time
//! against a series kept in London would be a second spelling of the same
//! Thursday, and calling it off would leave the real one on the calendar.
//! So the day is written once, on the series' own clock, and that one
//! spelling is what is linked, called off and compared everywhere.

use crate::application::invitations::{Invitation, OneDay};
use crate::data::message_cache::CalendarEventEntry;

/// The day, written the way the series writes its own start, or nothing when
/// either zone cannot be placed and they differ.
pub fn the_day_on_the_series_clock(_day: &OneDay, _series: &CalendarEventEntry) -> Option<String> {
    None
}

/// The series' row moved to that day, as long as the series' own meeting is,
/// and repeating nothing: the day as the series holds it.
pub fn as_the_series_holds_it(series: &CalendarEventEntry, _the_day: &str) -> CalendarEventEntry {
    series.clone()
}

/// Whether the series still holds that day, rather than calling it off.
pub fn still_on_the_series(_series: &CalendarEventEntry, _the_day: &str) -> bool {
    false
}

/// The day kept apart from its series as an appointment of its own, at the
/// time the organiser's update gives it.
pub fn the_day_kept_apart(
    series: &CalendarEventEntry,
    _invitation: &Invitation,
) -> CalendarEventEntry {
    series.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The weekly sync, Thursdays at nine to ten from 5 March, in London.
    fn the_series() -> CalendarEventEntry {
        CalendarEventEntry {
            id: "evt-series".to_string(),
            account_id: "acct".to_string(),
            provider_event_id: Some("s-1@example.com".to_string()),
            calendar_id: Some("cal".to_string()),
            summary: "Weekly sync".to_string(),
            description: Some("Standing agenda".to_string()),
            location: Some("Room 3".to_string()),
            start_datetime: "2026-03-05T09:00:00".to_string(),
            end_datetime: "2026-03-05T10:00:00".to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: Some("Europe/London".to_string()),
            status: "confirmed".to_string(),
            recurrence_rule: Some("FREQ=WEEKLY;COUNT=10".to_string()),
            categories: "Work".to_string(),
            source_provider: Some("caldav".to_string()),
            etag: Some("tag-1".to_string()),
            web_link: Some("https://dav.example.com/cal/s-1.ics".to_string()),
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            created_at: String::new(),
            updated_at: String::new(),
            pending: false,
            exception_dates: Some("20260319T090000".to_string()),
            cut_from_event_id: None,
            provider_recurrence_id: None,
        }
    }

    fn a_day(at: &str, zone: Option<&str>) -> OneDay {
        OneDay {
            at: at.to_string(),
            zone: zone.map(str::to_string),
            from_then_on: false,
        }
    }

    #[test]
    fn test_the_day_is_written_on_the_series_clock() {
        let london = the_series();
        let in_universal_time = CalendarEventEntry {
            start_datetime: "2026-03-05T09:00:00Z".to_string(),
            end_datetime: "2026-03-05T10:00:00Z".to_string(),
            time_zone: None,
            ..the_series()
        };
        let whole_days = CalendarEventEntry {
            start_datetime: "2026-03-05".to_string(),
            end_datetime: "2026-03-06".to_string(),
            start_date: Some("2026-03-05".to_string()),
            end_date: Some("2026-03-06".to_string()),
            is_all_day: true,
            ..the_series()
        };
        let los_angeles = CalendarEventEntry {
            time_zone: Some("America/Los_Angeles".to_string()),
            ..the_series()
        };
        let rows: [(&str, &CalendarEventEntry, OneDay, Option<&str>); 8] = [
            (
                "the series' own zone",
                &london,
                a_day("2026-03-12T09:00:00", Some("Europe/London")),
                Some("2026-03-12T09:00:00"),
            ),
            (
                "universal time in March, when London keeps it",
                &london,
                a_day("2026-03-12T09:00:00Z", None),
                Some("2026-03-12T09:00:00"),
            ),
            (
                "universal time in July, an hour behind London's summer clock",
                &london,
                a_day("2026-07-16T09:00:00Z", None),
                Some("2026-07-16T10:00:00"),
            ),
            (
                "London's summer clock against a series kept in universal time",
                &in_universal_time,
                a_day("2026-07-16T10:00:00", Some("Europe/London")),
                Some("2026-07-16T09:00:00Z"),
            ),
            (
                "a date against a series of whole days",
                &whole_days,
                a_day("2026-03-12", None),
                Some("2026-03-12"),
            ),
            (
                "a zone nothing here can place",
                &london,
                a_day("2026-03-12T09:00:00", Some("Nowhere/Atlantis")),
                None,
            ),
            (
                "no zone at all, which is the series' own",
                &london,
                a_day("2026-03-12T09:00:00", None),
                Some("2026-03-12T09:00:00"),
            ),
            (
                "the Windows name Outlook writes for the series' own zone",
                &los_angeles,
                a_day("2026-03-12T09:00:00", Some("Pacific Standard Time")),
                Some("2026-03-12T09:00:00"),
            ),
        ];

        for (case, series, day, expected) in rows {
            assert_eq!(
                the_day_on_the_series_clock(&day, series).as_deref(),
                expected,
                "{case}"
            );
        }
    }

    #[test]
    fn test_that_day_as_the_series_holds_it_keeps_the_series_length_and_repeats_nothing() {
        let that_day = as_the_series_holds_it(&the_series(), "2026-03-12T09:00:00");

        assert_eq!(that_day.start_datetime, "2026-03-12T09:00:00");
        assert_eq!(that_day.end_datetime, "2026-03-12T10:00:00");
        assert_eq!(that_day.recurrence_rule, None);
        assert_eq!(that_day.exception_dates, None);
        assert_eq!(that_day.id, "evt-series");
        assert_eq!(that_day.time_zone.as_deref(), Some("Europe/London"));
    }

    #[test]
    fn test_a_day_the_series_already_calls_off_is_not_on_it() {
        let series = the_series();

        assert!(!still_on_the_series(&series, "2026-03-19T09:00:00"));
        assert!(still_on_the_series(&series, "2026-03-12T09:00:00"));
    }

    #[test]
    fn test_the_day_kept_apart_names_its_series_and_carries_none_of_its_identity() {
        let invitation = crate::application::invitations::read_the_invitation(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\nMETHOD:REQUEST\r\n\
             BEGIN:VEVENT\r\nUID:s-1@example.com\r\nSEQUENCE:1\r\nSUMMARY:Weekly sync, moved\r\n\
             LOCATION:Room 5\r\nRECURRENCE-ID;TZID=Europe/London:20260312T090000\r\n\
             DTSTART;TZID=Europe/London:20260313T140000\r\n\
             DTEND;TZID=Europe/London:20260313T150000\r\n\
             ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n\
             END:VEVENT\r\nEND:VCALENDAR\r\n",
        )
        .expect("the update to read");
        let series = the_series();

        let kept = the_day_kept_apart(&series, &invitation);

        assert_ne!(kept.id, series.id);
        assert_eq!(kept.cut_from_event_id.as_deref(), Some("evt-series"));
        assert_eq!(kept.account_id, "acct");
        assert_eq!(kept.calendar_id.as_deref(), Some("cal"));
        assert_eq!(kept.status, "confirmed");
        assert_eq!(kept.categories, "Work");
        assert_eq!(kept.summary, "Weekly sync, moved");
        assert_eq!(kept.location.as_deref(), Some("Room 5"));
        assert_eq!(kept.start_datetime, "2026-03-13T14:00:00");
        assert_eq!(kept.end_datetime, "2026-03-13T15:00:00");
        assert_eq!(kept.time_zone.as_deref(), Some("Europe/London"));
        assert!(!kept.is_all_day);
        // None of the series' identity: a day kept apart that carried the
        // rule would be a second series, and one carrying the series' address
        // or version would be sent as the series.
        assert_eq!(kept.recurrence_rule, None);
        assert_eq!(kept.exception_dates, None);
        assert_eq!(kept.provider_event_id, None);
        assert_eq!(kept.source_provider, None);
        assert_eq!(kept.etag, None);
        assert_eq!(kept.web_link, None);
        assert_eq!(kept.provider_recurrence_id, None);
        assert!(kept.pending, "a day nobody sends never reaches the server");
    }
}

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

use chrono::{DateTime, FixedOffset};

use crate::application::calendar::{ADayWent, with_one_more_day_called_off};
use crate::application::invitations::{Invitation, OneDay};
use crate::common::Result;
use crate::common::moment::{self, Moment, WHOLE_DAY};
use crate::data::message_cache::{CalendarEventEntry, MessageCache};

/// What the calendar holds for one day of a meeting a message names.
///
/// One answer that the change a message makes, the sentence said about it and
/// answering that one day all read, so none of them can come to find the day
/// somewhere the others do not.
#[derive(Debug, Clone, PartialEq)]
pub enum ThatDay {
    /// Nothing on the calendar goes by the meeting's name.
    NotHeld,
    /// The day has a row of its own, kept apart here before or split out by a
    /// calendar server, with the series it came from when that is held.
    ItsOwnRow {
        row: CalendarEventEntry,
        series: Option<CalendarEventEntry>,
    },
    /// The series holds the day, which is `the_day` on its own clock and falls
    /// as `as_it_falls`.
    OnTheSeries {
        series: CalendarEventEntry,
        the_day: String,
        as_it_falls: CalendarEventEntry,
    },
    /// The series calls the day off already.
    OffTheSeries {
        series: CalendarEventEntry,
        the_day: String,
    },
    /// The calendar holds the meeting once, as a single appointment.
    OneAppointment(CalendarEventEntry),
    /// The day is on a clock this computer cannot place against the series'.
    CannotBePlaced { series: CalendarEventEntry },
}

/// What the calendar holds for `day` of the meeting `uid` names, on `account`.
///
/// The whole meeting is looked up first, only for its clock: the day's own
/// row is found by the day on the series' clock, or as written when no
/// meeting is held, and answers for the day whenever there is one. Then the
/// whole meeting says the rest: a single appointment, or the series holding
/// the day or calling it off.
pub fn what_the_calendar_holds_for_that_day(
    cache: &MessageCache,
    account: &str,
    uid: &str,
    day: &OneDay,
) -> Result<ThatDay> {
    let copy = cache.get_event_by_ical_uid(account, uid)?;
    let the_day = match &copy {
        None => day.at.clone(),
        Some(copy) => match the_day_on_the_series_clock(day, copy) {
            Some(the_day) => the_day,
            None => {
                return Ok(ThatDay::CannotBePlaced {
                    series: copy.clone(),
                });
            }
        },
    };
    if let Some(row) = cache.the_day_of_a_meeting(account, uid, &the_day)? {
        let series = copy.filter(|copy| copy.id != row.id);
        return Ok(ThatDay::ItsOwnRow { row, series });
    }
    let Some(copy) = copy else {
        return Ok(ThatDay::NotHeld);
    };
    if !repeats(&copy) {
        return Ok(ThatDay::OneAppointment(copy));
    }
    Ok(if still_on_the_series(&copy, &the_day) {
        ThatDay::OnTheSeries {
            as_it_falls: as_the_series_holds_it(&copy, &the_day),
            series: copy,
            the_day,
        }
    } else {
        ThatDay::OffTheSeries {
            series: copy,
            the_day,
        }
    })
}

/// Whether a row repeats.
pub fn repeats(row: &CalendarEventEntry) -> bool {
    row.recurrence_rule
        .as_deref()
        .is_some_and(|rule| !rule.trim().is_empty())
}

/// A clock face with no zone on it, the way the calendar stores one.
const A_CLOCK_FACE: &str = "%Y-%m-%dT%H:%M:%S";
/// An instant in universal time, the way a calendar kept in it stores one.
const IN_UNIVERSAL_TIME: &str = "%Y-%m-%dT%H:%M:%SZ";

/// The day, written the way the series writes its own start, or nothing when
/// either zone cannot be placed and they differ.
///
/// A day naming no zone, or exactly the series' zone, is on the series' clock
/// already and is taken as written. Any other is read in its own zone, its
/// `TZID` or universal time for a `Z`, through the reader that places Windows
/// names too, and written as the series' start is written: a clock face in
/// the series' zone, a `Z` moment for a series kept in universal time, a date
/// for a series of whole days.
pub fn the_day_on_the_series_clock(day: &OneDay, series: &CalendarEventEntry) -> Option<String> {
    let written = moment::read(&day.at)?;
    if series.is_all_day {
        return Some(written.the_day().format(WHOLE_DAY).to_string());
    }
    let its_zone = moment::the_zone_named(day.zone.as_deref());
    let the_series_zone = moment::the_zone_named(series.time_zone.as_deref());
    if matches!(written, Moment::ClockFace(_))
        && (its_zone.is_none() || its_zone == the_series_zone)
    {
        return Some(day.at.clone());
    }
    let Moment::Fixed(instant) = moment::read_in(&day.at, its_zone)? else {
        // A clock face in a zone nothing here can place.
        return None;
    };
    match moment::read(the_series_start(series))? {
        Moment::Fixed(_) => Some(
            instant
                .with_timezone(&chrono::Utc)
                .format(IN_UNIVERSAL_TIME)
                .to_string(),
        ),
        Moment::ClockFace(_) => the_clock_face_in(instant, the_series_zone),
        Moment::WholeDay(_) => Some(instant.date_naive().format(WHOLE_DAY).to_string()),
    }
}

/// Where the series starts, in whichever column holds it.
fn the_series_start(series: &CalendarEventEntry) -> &str {
    series
        .start_date
        .as_deref()
        .unwrap_or(&series.start_datetime)
}

/// The clock face an instant shows in a named zone, or on this computer's
/// clock for a series naming none, which is how a clock face beside no zone
/// is read everywhere else here.
fn the_clock_face_in(instant: DateTime<FixedOffset>, zone: Option<&str>) -> Option<String> {
    let face = match zone {
        None => instant.with_timezone(&chrono::Local).naive_local(),
        Some(named) => instant
            .with_timezone(&crate::common::zones::the_zone_called(named)?)
            .naive_local(),
    };
    Some(face.format(A_CLOCK_FACE).to_string())
}

/// The series' row moved to that day, as long as the series' own meeting is,
/// and repeating nothing: the day as the series holds it.
///
/// What the calendar shows on that day, which is what a change to the day is
/// compared with and what its sentence says it was.
pub fn as_the_series_holds_it(series: &CalendarEventEntry, the_day: &str) -> CalendarEventEntry {
    let ends = series.end_date.as_deref().unwrap_or(&series.end_datetime);
    let the_end = the_same_length_after(the_series_start(series), ends, the_day)
        .unwrap_or_else(|| the_day.to_string());
    CalendarEventEntry {
        start_datetime: the_day.to_string(),
        end_datetime: the_end.clone(),
        start_date: series.is_all_day.then(|| the_day.to_string()),
        end_date: series.is_all_day.then_some(the_end),
        recurrence_rule: None,
        exception_dates: None,
        ..series.clone()
    }
}

/// The end of a meeting starting on `the_day` that lasts as long as one from
/// `starts` to `ends`, written the way `the_day` is, or nothing when the three
/// are not one shape.
fn the_same_length_after(starts: &str, ends: &str, the_day: &str) -> Option<String> {
    match (
        moment::read(starts)?,
        moment::read(ends)?,
        moment::read(the_day)?,
    ) {
        (Moment::ClockFace(from), Moment::ClockFace(to), Moment::ClockFace(day)) => {
            Some((day + (to - from)).format(A_CLOCK_FACE).to_string())
        }
        (Moment::Fixed(from), Moment::Fixed(to), Moment::Fixed(day)) => Some(
            (day + (to - from))
                .with_timezone(&chrono::Utc)
                .format(IN_UNIVERSAL_TIME)
                .to_string(),
        ),
        (Moment::WholeDay(from), Moment::WholeDay(to), Moment::WholeDay(day)) => {
            Some((day + (to - from)).format(WHOLE_DAY).to_string())
        }
        _ => None,
    }
}

/// Whether the series still holds that day, rather than calling it off.
///
/// By the rule that decides whether calling a day off adds anything, so the
/// question and the write cannot come to two answers about the same day.
pub fn still_on_the_series(series: &CalendarEventEntry, the_day: &str) -> bool {
    let called_off = crate::service::caldav::the_called_off_value_for(the_day, series.is_all_day);
    with_one_more_day_called_off(series, &called_off).1 == ADayWent::OffTheSeries
}

/// Whether a single appointment is at that day's time, which is when a
/// message about one day of a meeting is about it.
pub fn is_at_that_day(appointment: &CalendarEventEntry, the_day: &str) -> bool {
    let zone = appointment.time_zone.as_deref();
    crate::application::invitations::the_same_instant(
        the_series_start(appointment),
        zone,
        the_day,
        zone,
    )
}

/// Whether a single appointment is the one a message about `day` means: at
/// that day's time, read on the appointment's own clock.
pub fn is_the_appointment_for(appointment: &CalendarEventEntry, day: &OneDay) -> bool {
    the_day_on_the_series_clock(day, appointment)
        .is_some_and(|the_day| is_at_that_day(appointment, &the_day))
}

/// The day kept apart from its series as an appointment of its own, at the
/// time the organiser's update gives it.
///
/// It names its series, and carries the series' account, calendar, status and
/// categories and nothing of its identity: no repeat, no called-off days, no
/// identifier or address or version at any provider. A day kept apart that
/// carried the rule would be a second series nobody asked for, and one
/// carrying the series' address would be sent as the series. The same shape
/// the editor's own one-day change leaves, so the push already knows the
/// pair.
pub fn the_day_kept_apart(
    series: &CalendarEventEntry,
    invitation: &Invitation,
) -> CalendarEventEntry {
    let ends = crate::application::caldav_sync::the_end_a_calendar_did_not_give(
        &invitation.starts,
        invitation.ends.as_deref(),
        invitation.is_all_day,
    );
    CalendarEventEntry {
        id: uuid::Uuid::new_v4().to_string(),
        account_id: series.account_id.clone(),
        provider_event_id: None,
        calendar_id: series.calendar_id.clone(),
        summary: invitation.summary.clone(),
        description: None,
        location: invitation.location.clone(),
        start_datetime: invitation.starts.clone(),
        end_datetime: ends.clone(),
        start_date: invitation.is_all_day.then(|| invitation.starts.clone()),
        end_date: invitation.is_all_day.then_some(ends),
        is_all_day: invitation.is_all_day,
        time_zone: invitation.time_zone.clone(),
        status: series.status.clone(),
        recurrence_rule: None,
        categories: series.categories.clone(),
        source_provider: None,
        etag: None,
        web_link: None,
        show_as: series.show_as.clone(),
        last_modified_remote: None,
        last_synced_at: None,
        attendees_json: None,
        reminders_json: None,
        created_at: String::new(),
        updated_at: String::new(),
        // A change made here, which is what puts it in front of the push.
        pending: true,
        exception_dates: None,
        cut_from_event_id: Some(series.id.clone()),
        provider_recurrence_id: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;

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
            // The calendar format's own spelling of the same moment, the way
            // an invitation writes it.
            as_written: at.replace(['-', ':'], ""),
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

    // ── What the calendar holds for that day ──────────────────────────────

    const THE_MEETING: &str = "s-1@example.com";

    /// A store holding `rows`, each filed under the meeting's UID when it has
    /// that as its provider identifier, the way a calendar server's read and
    /// an answer here file one.
    fn a_store_holding(label: &str, rows: &[CalendarEventEntry]) -> TempHome<MessageCache> {
        let cache = TempHome::named(label, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a store")
        });
        for row in rows {
            cache.save_calendar_event(row).expect("the row filed");
        }
        cache
    }

    fn what_it_holds(cache: &MessageCache, day: &OneDay) -> ThatDay {
        what_the_calendar_holds_for_that_day(cache, "acct", THE_MEETING, day)
            .expect("the calendar to be readable")
    }

    fn the_twelfth() -> OneDay {
        a_day("2026-03-12T09:00:00", Some("Europe/London"))
    }

    #[test]
    fn test_a_meeting_the_calendar_does_not_hold_is_not_held() {
        let cache = a_store_holding("that_day_not_held", &[]);

        assert_eq!(what_it_holds(&cache, &the_twelfth()), ThatDay::NotHeld);
    }

    #[test]
    fn test_a_day_kept_apart_is_its_own_row_beside_its_series() {
        let kept = CalendarEventEntry {
            id: "evt-day".to_string(),
            provider_event_id: None,
            recurrence_rule: None,
            exception_dates: None,
            cut_from_event_id: Some("evt-series".to_string()),
            start_datetime: "2026-03-13T14:00:00".to_string(),
            end_datetime: "2026-03-13T15:00:00".to_string(),
            ..the_series()
        };
        let cache = a_store_holding("that_day_its_own_row", &[the_series(), kept.clone()]);
        cache
            .remember_the_day_it_stands_for("evt-day", THE_MEETING, "2026-03-12T09:00:00")
            .expect("the day linked");

        let ThatDay::ItsOwnRow { row, series } = what_it_holds(&cache, &the_twelfth()) else {
            panic!("the day's own row");
        };
        assert_eq!(row.id, "evt-day");
        assert_eq!(
            series.map(|series| series.id).as_deref(),
            Some("evt-series")
        );
    }

    #[test]
    fn test_a_day_still_on_the_series_is_that_day_as_the_series_holds_it() {
        let series = CalendarEventEntry {
            exception_dates: None,
            ..the_series()
        };
        let cache = a_store_holding("that_day_on_the_series", &[series]);

        let ThatDay::OnTheSeries {
            series,
            the_day,
            as_it_falls,
        } = what_it_holds(&cache, &the_twelfth())
        else {
            panic!("the day on the series");
        };
        assert_eq!(series.id, "evt-series");
        assert_eq!(the_day, "2026-03-12T09:00:00");
        assert_eq!(as_it_falls.start_datetime, "2026-03-12T09:00:00");
        assert_eq!(as_it_falls.end_datetime, "2026-03-12T10:00:00");
    }

    #[test]
    fn test_a_day_the_series_calls_off_is_off_it() {
        let cache = a_store_holding("that_day_off_the_series", &[the_series()]);

        assert!(
            matches!(
                what_it_holds(&cache, &a_day("2026-03-19T09:00:00", Some("Europe/London"))),
                ThatDay::OffTheSeries { ref the_day, .. } if the_day == "2026-03-19T09:00:00"
            ),
            "the series calls the nineteenth off"
        );
    }

    #[test]
    fn test_a_meeting_held_once_is_one_appointment() {
        let once = CalendarEventEntry {
            recurrence_rule: None,
            exception_dates: None,
            ..the_series()
        };
        let cache = a_store_holding("that_day_one_appointment", std::slice::from_ref(&once));

        assert!(
            matches!(what_it_holds(&cache, &the_twelfth()), ThatDay::OneAppointment(ref row) if row.id == once.id),
            "a single appointment"
        );
    }

    #[test]
    fn test_a_day_on_no_clock_here_cannot_be_placed() {
        let cache = a_store_holding("that_day_cannot_be_placed", &[the_series()]);

        assert!(
            matches!(
                what_it_holds(
                    &cache,
                    &a_day("2026-03-12T09:00:00", Some("Nowhere/Atlantis"))
                ),
                ThatDay::CannotBePlaced { .. }
            ),
            "a zone nothing here can place"
        );
    }
}

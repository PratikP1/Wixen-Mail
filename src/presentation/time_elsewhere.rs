//! The clock a meeting was written on, said once when it differs from this one.
//!
//! Every time is said on this computer's clock, the way Google shows a meeting
//! to everybody it invites. An invitation's covering note usually carries the
//! organiser's own hour in their own zone, so somebody who hears five in the
//! afternoon in the sentence and nine in the morning in the body needs to know
//! why. So the sentence before the message adds one clause with the other
//! clock, and only there: never in a list row, a button's description or an
//! alert, so it is heard once per message and a syncing calendar stays quiet.
//!
//! Nothing is said for a whole day, which is on its day wherever somebody is;
//! for a zone that is only a way of writing universal time, which says how a
//! server delivered the time and not where anybody is; or for a zone whose
//! clock agrees with this one's at that moment. A zone nothing here can place
//! is said to be one, because the hour beside it was kept as it was written.

use chrono::{DateTime, FixedOffset, NaiveDateTime, Offset, TimeZone};
use chrono_tz::Tz;

use crate::common::moment::{self, Moment};
use crate::common::zones;
use crate::presentation::date_display::{self, DateSettings};

/// The clause saying the meeting's own clock, or nothing when it would say
/// nothing this computer's clock has not.
///
/// `starts` and `ends` are stored as the meeting's own clock face, `zone` is
/// the zone stored beside them, and `dates` words a date the way this reader
/// does.
pub fn where_it_was_set(
    starts: &str,
    ends: Option<&str>,
    zone: Option<&str>,
    dates: DateSettings,
) -> Option<String> {
    seen_from(starts, ends, zone, &chrono::Local, dates)
}

/// The clause as seen by somebody whose clock is `here`, so what it says can
/// be asked of any zone rather than of the machine a test runs on.
fn seen_from<Here: TimeZone>(
    starts: &str,
    ends: Option<&str>,
    zone: Option<&str>,
    here: &Here,
    dates: DateSettings,
) -> Option<String> {
    let named = moment::the_zone_named(zone)?;
    let Some(its_zone) = zones::the_zone_called(named) else {
        // The hour was kept as it was written, so say that it was, rather
        // than let it pass as this computer's.
        return is_a_clock_face(starts)
            .then(|| format!("as written in {named}, a time zone this computer cannot place"));
    };
    if !zones::names_a_place(its_zone) {
        return None;
    }
    let start = an_instant(starts, zone)?;
    let there = start.with_timezone(&its_zone);
    let seen_here = start.with_timezone(here);
    if there.offset().fix() == seen_here.offset().fix() {
        return None;
    }
    let begins = if there.date_naive() == seen_here.date_naive() {
        date_display::the_clock_of(there.naive_local(), dates)
    } else {
        date_display::a_clock_face(there.naive_local(), dates)
    };
    let ending = ends
        .and_then(|ends| an_instant(ends, zone))
        .map(|end| {
            until(
                there.naive_local(),
                end.with_timezone(&its_zone).naive_local(),
                dates,
            )
        })
        .unwrap_or_default();
    Some(format!(
        "which is {begins}{ending} {}",
        the_zone_said(named)
    ))
}

/// " to 10:00", or with the whole date again when the meeting ends on another
/// day on its own clock.
fn until(start: NaiveDateTime, end: NaiveDateTime, dates: DateSettings) -> String {
    if end.date() == start.date() {
        format!(" to {}", date_display::the_clock_of(end, dates))
    } else {
        format!(" to {}", date_display::a_clock_face(end, dates))
    }
}

/// Whether a stored time is an hour on a clock, which a zone beside it would
/// have placed.
fn is_a_clock_face(stored: &str) -> bool {
    matches!(moment::read(stored), Some(Moment::ClockFace(_)))
}

/// The instant a stored time names in the zone beside it, when it names one.
fn an_instant(stored: &str, zone: Option<&str>) -> Option<DateTime<FixedOffset>> {
    match moment::read_in(stored, zone)? {
        Moment::Fixed(at) => Some(at),
        Moment::ClockFace(_) | Moment::WholeDay(_) => None,
    }
}

/// A stored zone's name as it is said: a Windows name as Windows writes it,
/// "Tokyo Standard Time", and a zone database name by its place,
/// "America/Los_Angeles" as "Los Angeles time".
///
/// The one place a stored zone name becomes words.
pub fn the_zone_said(name: &str) -> String {
    let name = name.trim();
    let the_databases = name.starts_with('/') || name.parse::<Tz>().is_ok();
    if !the_databases {
        return name.to_string();
    }
    let place = name.rsplit('/').next().unwrap_or(name).replace('_', " ");
    format!("{place} time")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::date_display::{Clock, DateOrder, DateStyle, DateWording};

    fn written_out_in_full() -> DateSettings {
        DateSettings {
            style: DateStyle::Absolute,
            order: DateOrder::DayFirst,
            wording: DateWording::Numeric,
            clock: Clock::TwentyFourHour,
        }
    }

    /// The clause as somebody in London hears it, whatever this machine is.
    fn from_london(starts: &str, ends: &str, zone: Option<&str>) -> Option<String> {
        seen_from(
            starts,
            Some(ends),
            zone,
            &chrono_tz::Europe::London,
            written_out_in_full(),
        )
    }

    #[test]
    fn test_a_meeting_set_on_another_clock_says_that_clock_once() {
        // Nine in Los Angeles in March is five in the afternoon in London, on
        // the same day, so the day is not said again.
        assert_eq!(
            from_london(
                "2026-03-05T09:00:00",
                "2026-03-05T10:00:00",
                Some("America/Los_Angeles")
            )
            .as_deref(),
            Some("which is 09:00 to 10:00 Los Angeles time")
        );
    }

    #[test]
    fn test_a_meeting_on_another_day_there_names_that_day() {
        // Eight in Tokyo on the fifth is eleven at night in London on the
        // fourth, so the clause says which day it is in Tokyo.
        assert_eq!(
            from_london(
                "2026-03-05T08:00:00",
                "2026-03-05T09:00:00",
                Some("Asia/Tokyo")
            )
            .as_deref(),
            Some("which is 05/03/2026 at 08:00 to 09:00 Tokyo time")
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_zone_windows_names_is_said_by_its_windows_name() {
        assert_eq!(
            from_london(
                "2026-03-05T08:00:00",
                "2026-03-05T09:00:00",
                Some("Tokyo Standard Time")
            )
            .as_deref(),
            Some("which is 05/03/2026 at 08:00 to 09:00 Tokyo Standard Time")
        );
    }

    #[test]
    fn test_a_meeting_on_the_clock_here_says_nothing_more() {
        assert_eq!(
            from_london(
                "2026-03-05T09:00:00",
                "2026-03-05T10:00:00",
                Some("Europe/London")
            ),
            None
        );
    }

    #[test]
    fn test_a_time_written_in_universal_time_says_nothing_more() {
        // Seen from Tokyo, so the clocks differ and only the zone's being no
        // place keeps the clause out.
        for universal in ["UTC", "Etc/UTC", "Etc/GMT+5"] {
            assert_eq!(
                seen_from(
                    "2026-03-05T14:00:00.0000000",
                    Some("2026-03-05T15:00:00.0000000"),
                    Some(universal),
                    &chrono_tz::Asia::Tokyo,
                    written_out_in_full(),
                ),
                None,
                "{universal}"
            );
        }
    }

    #[test]
    fn test_a_whole_day_says_nothing_more() {
        assert_eq!(
            from_london("2026-03-05", "2026-03-06", Some("Asia/Tokyo")),
            None
        );
    }

    #[test]
    fn test_a_zone_nothing_here_can_place_is_said_to_be_one() {
        assert_eq!(
            from_london(
                "2026-03-05T09:00:00",
                "2026-03-05T10:00:00",
                Some("Customized Time Zone")
            )
            .as_deref(),
            Some("as written in Customized Time Zone, a time zone this computer cannot place")
        );
    }

    #[test]
    fn test_a_zone_named_by_nothing_but_space_says_nothing() {
        for blank in [None, Some(""), Some("   ")] {
            assert_eq!(
                from_london("2026-03-05T09:00:00", "2026-03-05T10:00:00", blank),
                None,
                "{blank:?}"
            );
        }
    }

    #[test]
    fn test_a_zones_name_is_said_by_its_place() {
        assert_eq!(the_zone_said("America/Los_Angeles"), "Los Angeles time");
        assert_eq!(
            the_zone_said("/mozilla.org/20050126_1/America/New_York"),
            "New York time"
        );
        assert_eq!(the_zone_said("Tokyo Standard Time"), "Tokyo Standard Time");
    }
}

//! The shapes a stored moment takes, and the one place that reads them.
//!
//! A moment reaches the cache from four places that each write it differently.
//! Microsoft Graph sends `2026-07-27T09:00:00.0000000`, seven digits of
//! fraction and no offset. Google and IMAP send RFC 3339, with an offset or a
//! `Z`. This program's own event editor writes `2026-07-27T09:00:00`, and older
//! rows and the reminder code write `2026-07-27 09:00` with a space. A whole
//! day is a bare date with no clock face at all.
//!
//! Every one of those is one column in one table, so anything that reads that
//! column has to know all of them. Two modules each keeping their own list of
//! the shapes is the defect this module exists to make impossible: the reader
//! knew three of them and the writer five, and the two Graph shapes were in the
//! writer's list and not the reader's. What the reader does with a shape it
//! cannot read is hand the stored string back, so the words announced for an
//! Outlook event were "2026-07-27T09:00:00". When the event editor started
//! writing the same shape, events made here read that way too.
//!
//! This lives in `common` because the layer above it does not have a single
//! answer. `presentation::date_display` reads these strings to say them aloud,
//! `application::calendar` and `application::caldav_sync` read them to send
//! them to a provider, and `application::due` reads them to decide whether a
//! reminder is late. Presentation must not reach into application, so the one
//! place all four can share is the layer underneath both.

use std::borrow::Cow;

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime};

/// A clock face with no zone on the end, in the shapes the cache holds one.
///
/// `%.f` matches a fraction and matches its absence, so one entry covers both
/// what Graph sends and what the editor writes. The two separators are both
/// here because a value written with a space and the same value written with a
/// `T` are the same moment, and the only reason there are two is history.
///
/// Not public. The list is the thing that went wrong by being copied, so the
/// way to use it is [`read`], which answers the question every caller was
/// really asking and cannot be half-copied.
const CLOCK_FACES: [&str; 4] = [
    "%Y-%m-%dT%H:%M:%S%.f",
    "%Y-%m-%dT%H:%M",
    "%Y-%m-%d %H:%M:%S%.f",
    "%Y-%m-%d %H:%M",
];

/// A day with no clock face on it.
pub const WHOLE_DAY: &str = "%Y-%m-%d";

/// What a stored moment turned out to be.
///
/// Three cases rather than one parsed instant, because every caller needs to
/// tell them apart and each would otherwise work it out again. A value carrying
/// its own offset names one instant everywhere; a clock face names an hour and
/// leaves the zone to whatever is stored beside it; a whole day has no hour at
/// all, and reading it as midnight is a claim the stored value never made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Moment {
    /// Carried its own offset or a `Z`, so it says which instant it means.
    Fixed(DateTime<FixedOffset>),
    /// An hour, in whatever zone is stored beside it.
    ClockFace(NaiveDateTime),
    /// A day, with nothing said about the time of day.
    WholeDay(NaiveDate),
}

impl Moment {
    /// The day this moment falls on.
    ///
    /// A moment carrying an offset falls on the day it names in that offset,
    /// which is the only day it can be said to fall on without knowing where
    /// somebody is standing. A clock face and a whole day each carry their own
    /// day already.
    pub fn the_day(self) -> NaiveDate {
        match self {
            Moment::Fixed(moment) => moment.date_naive(),
            Moment::ClockFace(clock) => clock.date(),
            Moment::WholeDay(day) => day,
        }
    }

    /// Where this falls on this computer's clock.
    ///
    /// One answer for all three shapes, so the two readers that had a copy of
    /// this each and the calendar's own ordering all agree. A whole day starts
    /// at midnight, which is what a day with nothing said about the time of
    /// day means everywhere else in this program.
    pub fn on_this_computer(self) -> Option<chrono::DateTime<chrono::Local>> {
        match self {
            Moment::Fixed(at) => Some(at.with_timezone(&chrono::Local)),
            Moment::ClockFace(clock) => on_this_computer(clock),
            Moment::WholeDay(day) => on_this_computer(day.and_hms_opt(0, 0, 0)?),
        }
    }
}

/// Read a stored moment, in every shape the cache holds one.
///
/// Nothing is returned for a value that is none of them. That is not the same
/// as an error: every caller has an answer for a moment it cannot read, and
/// they differ. The reader says the stored characters as they stand, which is
/// poor to listen to and better than silence or an invented date. The two
/// provider writers refuse to send it, rather than send an hour nobody meant.
pub fn read(stored: &str) -> Option<Moment> {
    let trimmed = stored.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(fixed) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(Moment::Fixed(fixed));
    }
    if let Some(clock) = clock_face(trimmed) {
        return Some(Moment::ClockFace(clock));
    }
    NaiveDate::parse_from_str(trimmed, WHOLE_DAY)
        .ok()
        .map(Moment::WholeDay)
}

/// The zone a stored clock face is named in, when the name names one.
///
/// A blank name is not a name. It says nothing about which hour a clock face
/// beside it means, so the answer is the same as no name at all, and every
/// writer that reads this column has to give that answer or one event goes to
/// two places at two different hours. Four of them each decided separately:
/// the Graph writer trimmed and the Google writer did not, so a name of one
/// space sent Graph an hour on this computer turned into universal time and
/// sent Google a clock face in a zone called " ". The calendar-server writer
/// asked neither question and wrote `DTSTART;TZID=:20260305T090000`, which is
/// not a calendar document at all.
///
/// Trimmed rather than refused, so a name with a space in front of it is still
/// the zone it names.
pub fn the_zone_named(stored: Option<&str>) -> Option<&str> {
    stored.map(str::trim).filter(|named| !named.is_empty())
}

/// Read a stored moment in the zone stored beside it.
///
/// The one place a stored time meets its zone. A clock face is stored as the
/// provider or the document wrote it, with the zone it was written in beside
/// it, because three writers send exactly that back and a series repeats on
/// its clock face; converting it where it arrives would send a different hour
/// back. So the conversion happens here, where it is read to be shown or said:
/// a clock face beside a zone this computer can place becomes the instant it
/// names there.
///
/// A moment carrying its own offset already names its instant, and a whole
/// day is on its day wherever somebody is, so both come back as read. A clock
/// face beside no zone is an hour on this computer's clock, and one beside a
/// zone nothing can place stays a clock face too, which the caller says rather
/// than presenting it as this computer's hour.
pub fn read_in(stored: &str, zone: Option<&str>) -> Option<Moment> {
    let moment = read(stored)?;
    let Moment::ClockFace(clock) = moment else {
        return Some(moment);
    };
    let placed = the_zone_named(zone)
        .and_then(super::zones::the_zone_called)
        .and_then(|its_zone| placed_in(clock, &its_zone));
    Some(placed.map_or(moment, |at| Moment::Fixed(at.fixed_offset())))
}

/// A stored moment written again as the instant its zone makes it, for a
/// reader that takes stored text.
///
/// The stored text itself when reading it in its zone changed nothing, and an
/// RFC 3339 instant when it placed a clock face, which [`read`] reads back as
/// the same instant.
pub fn written_in_its_zone<'a>(stored: &'a str, zone: Option<&str>) -> Cow<'a, str> {
    match read_in(stored, zone) {
        Some(Moment::Fixed(at)) if clock_face(stored).is_some() => Cow::Owned(at.to_rfc3339()),
        _ => Cow::Borrowed(stored),
    }
}

/// The clock face a stored value holds, when it holds one and no offset.
fn clock_face(stored: &str) -> Option<NaiveDateTime> {
    let trimmed = stored.trim();
    CLOCK_FACES
        .iter()
        .find_map(|shape| NaiveDateTime::parse_from_str(trimmed, shape).ok())
}

/// Where a clock face falls on this computer's clock, on every day of the year.
///
/// [`placed_in`] this computer's zone, by the one rule for the two days a year
/// that have no single answer.
pub fn on_this_computer(clock: NaiveDateTime) -> Option<chrono::DateTime<chrono::Local>> {
    placed_in(clock, &chrono::Local)
}

/// Where a clock face falls in a zone, on every day of the year.
///
/// Two days a year have no single answer and both used to come back as no
/// answer at all.
///
/// On the day the clocks go back, an hour of clock face happens twice. Asking
/// for "the" instant that matches it has two answers, and a reminder set for
/// one of them was filtered out of what is due and never went off, while the
/// same question asked for reading it aloud fell through to speaking the raw
/// stored text. The earlier of the two is taken, which is the one somebody
/// setting a reminder that morning meant.
///
/// On the day they go forward, an hour of clock face does not happen at all.
/// A reminder already set for it, or one carried in from a calendar in
/// another zone, is due when the clock reaches the hour it jumped to, rather
/// than never.
///
/// One rule for this computer's zone and for the zone a meeting was written
/// in, so a time read in either is placed the same way.
pub fn placed_in<Z: chrono::TimeZone>(clock: NaiveDateTime, zone: &Z) -> Option<DateTime<Z>> {
    if let Some(found) = zone.from_local_datetime(&clock).earliest() {
        return Some(found);
    }
    // Skipped by the clocks going forward. The jump is an hour almost
    // everywhere and half an hour in a few places, so this walks forward in
    // small steps rather than assuming which.
    (1..=8).find_map(|quarters| {
        zone.from_local_datetime(&(clock + chrono::Duration::minutes(15 * quarters)))
            .earliest()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clock(text: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S").expect("a real clock face")
    }

    /// The shapes both of the lists this replaced knew, plus the one neither
    /// did.
    #[test]
    fn test_every_zoneless_shape_the_cache_holds_is_read_as_a_clock_face() {
        for stored in [
            "2026-07-27T09:00:00",
            "2026-07-27T09:00",
            "2026-07-27 09:00:00",
            "2026-07-27 09:00",
        ] {
            assert_eq!(
                read(stored),
                Some(Moment::ClockFace(clock("2026-07-27 09:00:00"))),
                "stored as {stored}"
            );
        }
    }

    /// Seven digits of fraction is what Graph sends on every event, and the
    /// space form is here because a shape should not depend on the separator.
    #[test]
    fn test_a_fraction_of_a_second_is_read_whichever_separator_it_uses() {
        for stored in ["2026-07-27T09:00:00.0000000", "2026-07-27 09:00:00.0000000"] {
            assert_eq!(
                read(stored),
                Some(Moment::ClockFace(clock("2026-07-27 09:00:00"))),
                "stored as {stored}"
            );
        }
    }

    #[test]
    fn test_a_moment_carrying_an_offset_or_a_z_keeps_the_instant_it_named() {
        let Some(Moment::Fixed(east)) = read("2026-07-27T09:00:00+05:30") else {
            panic!("an offset should be read as a fixed moment");
        };
        assert_eq!(east.to_rfc3339(), "2026-07-27T09:00:00+05:30");

        let Some(Moment::Fixed(zulu)) = read("2026-07-27T09:00:00Z") else {
            panic!("a Z should be read as a fixed moment");
        };
        assert_eq!(zulu.to_rfc3339(), "2026-07-27T09:00:00+00:00");
    }

    #[test]
    fn test_a_whole_day_is_a_day_and_not_midnight() {
        assert_eq!(
            read("2026-07-27"),
            Some(Moment::WholeDay(
                NaiveDate::from_ymd_opt(2026, 7, 27).expect("a real day")
            ))
        );
    }

    #[test]
    fn test_nothing_is_returned_for_a_value_that_is_none_of_the_shapes() {
        for stored in [
            "",
            "   ",
            "not a moment",
            "2026-13-45T99:99:99",
            "27/07/2026",
            "--07-27",
        ] {
            assert_eq!(read(stored), None, "stored as {stored}");
        }
    }

    #[test]
    fn test_a_value_padded_with_spaces_is_still_the_moment_it_names() {
        assert_eq!(
            read("  2026-07-27T09:00:00  "),
            Some(Moment::ClockFace(clock("2026-07-27 09:00:00")))
        );
    }

    #[test]
    fn test_the_day_a_moment_falls_on_is_read_from_every_shape() {
        let day = NaiveDate::from_ymd_opt(2026, 7, 27).expect("a real day");
        for stored in [
            "2026-07-27T00:00:00Z",
            "2026-07-27T00:00:00.0000000",
            "2026-07-27 09:00",
            "2026-07-27",
            "2026-07-27T23:59:59+05:30",
        ] {
            assert_eq!(
                read(stored).expect("a moment").the_day(),
                day,
                "stored as {stored}"
            );
        }
    }

    #[test]
    fn test_a_zone_name_of_nothing_but_space_names_no_zone() {
        for naming_nothing in ["", " ", "   ", "\t", "\r\n"] {
            assert_eq!(
                the_zone_named(Some(naming_nothing)),
                None,
                "for a zone stored as {naming_nothing:?}"
            );
        }
        assert_eq!(the_zone_named(None), None);
    }

    #[test]
    fn test_a_zone_name_with_space_round_it_is_still_the_zone_it_names() {
        assert_eq!(
            the_zone_named(Some("  Europe/London  ")),
            Some("Europe/London")
        );
        assert_eq!(the_zone_named(Some("UTC")), Some("UTC"));
    }

    /// A moment carrying an offset is not a clock face, so a caller asking
    /// that question gets no for it rather than the hour with the offset
    /// quietly dropped.
    #[test]
    fn test_an_offset_is_not_mistaken_for_a_clock_face() {
        assert_eq!(clock_face("2026-07-27T09:00:00+05:30"), None);
        assert_eq!(clock_face("2026-07-27T09:00:00Z"), None);
        assert_eq!(clock_face("2026-07-27"), None);
        assert_eq!(
            clock_face("2026-07-27T09:00:00"),
            Some(clock("2026-07-27 09:00:00"))
        );
    }

    /// The instant a stored clock face names in `zone`, with the offset that
    /// zone had then, written the way RFC 3339 writes both.
    fn placed(stored: &str, zone: &str) -> String {
        match read_in(stored, Some(zone)) {
            Some(Moment::Fixed(at)) => at.to_rfc3339(),
            other => panic!("{stored} beside {zone} was not placed: {other:?}"),
        }
    }

    #[test]
    fn test_a_clock_face_beside_a_zone_is_the_instant_that_zone_puts_it_at() {
        // Nine in Los Angeles is five in the afternoon in universal time in
        // March, and four in July, when Los Angeles is on summer time.
        assert_eq!(
            placed("2026-03-05T09:00:00", "America/Los_Angeles"),
            "2026-03-05T09:00:00-08:00"
        );
        assert_eq!(
            placed("2026-07-09T09:00:00", "America/Los_Angeles"),
            "2026-07-09T09:00:00-07:00"
        );
    }

    #[test]
    fn test_the_hour_a_zone_goes_through_twice_is_the_earlier_one() {
        // 1 November 2026, when Los Angeles goes back to standard time at two
        // in the morning, so half past one happens once at -07:00 and again at
        // -08:00. The rule this computer's own clock follows, for the same
        // reason.
        assert_eq!(
            placed("2026-11-01T01:30:00", "America/Los_Angeles"),
            "2026-11-01T01:30:00-07:00"
        );
    }

    #[test]
    fn test_the_hour_a_zone_skips_is_the_first_that_happens_after_it() {
        // 8 March 2026, when Los Angeles goes from two to three in the
        // morning, so half past two never happens there.
        assert_eq!(
            placed("2026-03-08T02:30:00", "America/Los_Angeles"),
            "2026-03-08T03:00:00-07:00"
        );
    }

    #[test]
    fn test_a_clock_face_graph_wrote_beside_utc_is_that_instant() {
        assert_eq!(
            placed("2026-03-05T14:00:00.0000000", "UTC"),
            "2026-03-05T14:00:00+00:00"
        );
    }

    #[test]
    fn test_a_moment_carrying_its_own_offset_keeps_it_whatever_zone_is_beside_it() {
        assert_eq!(
            placed("2026-03-05T09:00:00Z", "Asia/Tokyo"),
            "2026-03-05T09:00:00+00:00"
        );
    }

    #[test]
    fn test_a_whole_day_is_not_moved_by_the_zone_beside_it() {
        let day = NaiveDate::from_ymd_opt(2026, 3, 5).expect("a real day");
        for zone in ["Asia/Tokyo", "America/Los_Angeles", "UTC"] {
            assert_eq!(
                read_in("2026-03-05", Some(zone)),
                Some(Moment::WholeDay(day)),
                "beside {zone}"
            );
        }
    }

    #[test]
    fn test_a_clock_face_beside_no_zone_or_one_nobody_can_place_stays_a_clock_face() {
        let nine = Some(Moment::ClockFace(clock("2026-03-05 09:00:00")));
        for zone in [None, Some(" "), Some("Customized Time Zone")] {
            assert_eq!(
                read_in("2026-03-05T09:00:00", zone),
                nine,
                "beside {zone:?}"
            );
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_clock_face_beside_a_windows_zone_name_is_placed_by_windows() {
        assert_eq!(
            placed("2026-03-05T09:00:00", "Tokyo Standard Time"),
            "2026-03-05T09:00:00+09:00"
        );
    }

    #[test]
    fn test_a_placed_clock_face_is_written_as_an_instant_that_reads_back_the_same() {
        let written = written_in_its_zone("2026-03-05T09:00:00", Some("Asia/Tokyo"));
        assert_eq!(written, "2026-03-05T09:00:00+09:00");
        assert_eq!(
            read(&written),
            read_in("2026-03-05T09:00:00", Some("Asia/Tokyo"))
        );
    }

    #[test]
    fn test_a_moment_its_zone_changes_nothing_about_is_handed_back_as_it_was_stored() {
        for (stored, zone) in [
            ("2026-03-05T09:00:00", None),
            ("2026-03-05T09:00:00", Some("Customized Time Zone")),
            ("2026-03-05T09:00:00Z", Some("Asia/Tokyo")),
            ("2026-03-05", Some("Asia/Tokyo")),
            ("not a moment", Some("Asia/Tokyo")),
        ] {
            assert!(
                matches!(written_in_its_zone(stored, zone), Cow::Borrowed(same) if same == stored),
                "{stored} beside {zone:?}"
            );
        }
    }
}

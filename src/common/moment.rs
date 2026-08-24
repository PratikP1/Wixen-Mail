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

/// The clock face a stored value holds, when it holds one and no offset.
fn clock_face(stored: &str) -> Option<NaiveDateTime> {
    let trimmed = stored.trim();
    CLOCK_FACES
        .iter()
        .find_map(|shape| NaiveDateTime::parse_from_str(trimmed, shape).ok())
}

/// Where a clock face falls on this computer's clock, on every day of the year.
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
pub fn on_this_computer(clock: NaiveDateTime) -> Option<chrono::DateTime<chrono::Local>> {
    use chrono::TimeZone;

    if let Some(found) = chrono::Local.from_local_datetime(&clock).earliest() {
        return Some(found);
    }
    // Skipped by the clocks going forward. The jump is an hour almost
    // everywhere and half an hour in a few places, so this walks forward in
    // small steps rather than assuming which.
    (1..=8).find_map(|quarters| {
        chrono::Local
            .from_local_datetime(&(clock + chrono::Duration::minutes(15 * quarters)))
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
}

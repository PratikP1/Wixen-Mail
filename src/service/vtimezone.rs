//! The rules of a named time zone, written as a calendar document component.
//!
//! RFC 5545 section 3.6 requires it: every date and time in a document that
//! names a zone must find that zone's rules in the same document, as a
//! `VTIMEZONE` component. A document that names a zone and defines it nowhere
//! is refused whole by a strict server, and a lenient one guesses, which puts
//! a meeting at whatever hour the guess lands on.
//!
//! The rules come from the IANA time zone database that ships inside
//! `chrono_tz`, the same database every other reading of a zone name here
//! goes through, so the rules written into a document and the conversions
//! made from stored values cannot disagree about what a zone means.
//!
//! What is written is derived by observation rather than copied out of a
//! table: the zone's offset is sampled across a window of years around the
//! event, every change of offset is pinned to the second, and the changes are
//! then described. A zone changing twice a year on a rule that holds across
//! the window gets the two blocks and two yearly rules a calendar program
//! expects. A zone that never changes gets one block and no rule. A zone
//! whose changes follow no yearly rule, which is real (Morocco moves its
//! clocks by the lunar calendar), gets its change days listed one by one,
//! because a wrong rule misplaces a meeting silently and a listed day cannot.
//!
//! The window is the event's own year, one back and four forward. That is a
//! stated tradeoff: a series read decades past its start leans on the
//! server's own copy of the same database beyond the window, and a listed
//! description goes stale sooner than a ruled one. Neither invents an
//! instant.

use chrono::{Datelike, NaiveDate, NaiveDateTime, Offset, TimeZone};
use chrono_tz::{OffsetComponents, Tz};

use crate::service::caldav::WIRE_CLOCK_FACE;

/// The `VTIMEZONE` component for a named zone, as document lines, or nothing
/// when the name is not in the time zone database.
///
/// Nothing rather than a guess, and the caller decides what a missing answer
/// means: the create path refuses to send the document and says which name
/// stopped it, and the change path keeps the server's own document as it was.
pub fn timezone_rules_for(tzid: &str, anchor_year: i32) -> Option<Vec<String>> {
    let zone: Tz = tzid.parse().ok()?;
    let changes = changes_within(zone, anchor_year - 1, anchor_year + 4)?;

    let mut lines = vec!["BEGIN:VTIMEZONE".to_string(), format!("TZID:{tzid}")];
    if changes.is_empty() {
        lines.extend(one_steady_block(zone, anchor_year));
        lines.push("END:VTIMEZONE".to_string());
        return Some(lines);
    }

    let (summer, winter): (Vec<&Change>, Vec<&Change>) =
        changes.iter().partition(|change| change.into_summer_time);
    let rules = (
        one_yearly_rule(&summer, anchor_year - 1, anchor_year + 4),
        one_yearly_rule(&winter, anchor_year - 1, anchor_year + 4),
    );
    match rules {
        // Two changes a year, each on a rule that holds across the whole
        // window: the shape almost every zone that changes at all has.
        (Some(into_summer), Some(into_winter)) => {
            lines.extend(ruled_block("DAYLIGHT", &summer, &into_summer, anchor_year));
            lines.extend(ruled_block("STANDARD", &winter, &into_winter, anchor_year));
        }
        // Anything else: list the days. A guessed rule misplaces a meeting
        // silently; a listed day cannot.
        _ => {
            lines.extend(listed_blocks("DAYLIGHT", &summer));
            lines.extend(listed_blocks("STANDARD", &winter));
        }
    }
    lines.push("END:VTIMEZONE".to_string());
    Some(lines)
}

/// One change of offset: the first second kept under the new offset, in UTC,
/// with the offset either side of it.
struct Change {
    first_second_utc: NaiveDateTime,
    from_seconds: i32,
    to_seconds: i32,
    /// Whether the side being changed into keeps summer time.
    into_summer_time: bool,
}

impl Change {
    /// The local wall time at which the clocks change, told in the offset
    /// that held until then, which is how a calendar document dates an onset.
    fn local_onset(&self) -> NaiveDateTime {
        self.first_second_utc + chrono::Duration::seconds(i64::from(self.from_seconds))
    }
}

/// The zone's offset from UTC at one instant, and whether it is summer time.
fn offset_at(zone: Tz, utc: NaiveDateTime) -> (i32, bool) {
    let offset = zone.offset_from_utc_datetime(&utc);
    (
        offset.fix().local_minus_utc(),
        !offset.dst_offset().is_zero(),
    )
}

/// Every change of offset between the start of one year and the end of
/// another, pinned to the second.
///
/// Sampled every seven days and searched between two samples that disagree.
/// No zone in the database changes its offset twice inside one week, so the
/// sampling cannot step over a change and back. Nothing is handed back only
/// for a year so far out the calendar itself cannot hold it.
fn changes_within(zone: Tz, first_year: i32, last_year: i32) -> Option<Vec<Change>> {
    let start = NaiveDate::from_ymd_opt(first_year, 1, 1)?.and_hms_opt(0, 0, 0)?;
    let end = NaiveDate::from_ymd_opt(last_year + 1, 1, 1)?.and_hms_opt(0, 0, 0)?;
    let mut changes = Vec::new();
    let mut at = start;
    while at < end {
        let next = (at + chrono::Duration::days(7)).min(end);
        if offset_at(zone, at).0 != offset_at(zone, next).0 {
            changes.push(the_change_between(zone, at, next));
        }
        at = next;
    }
    Some(changes)
}

/// The change of offset known to sit between two instants, to the second.
fn the_change_between(
    zone: Tz,
    mut still_before: NaiveDateTime,
    mut after: NaiveDateTime,
) -> Change {
    let before = offset_at(zone, still_before);
    while after - still_before > chrono::Duration::seconds(1) {
        let middle = still_before + (after - still_before) / 2;
        if offset_at(zone, middle).0 == before.0 {
            still_before = middle;
        } else {
            after = middle;
        }
    }
    let changed = offset_at(zone, after);
    Change {
        first_second_utc: after,
        from_seconds: before.0,
        to_seconds: changed.0,
        into_summer_time: changed.1,
    }
}

/// The one block of a zone that never changes: both offsets the same, no
/// rule, dated from before any calendar program because it has always held.
fn one_steady_block(zone: Tz, anchor_year: i32) -> Vec<String> {
    let steady = NaiveDate::from_ymd_opt(anchor_year, 1, 1)
        .and_then(|day| day.and_hms_opt(0, 0, 0))
        .map_or(0, |at| offset_at(zone, at).0);
    vec![
        "BEGIN:STANDARD".to_string(),
        "DTSTART:19700101T000000".to_string(),
        format!("TZOFFSETFROM:{}", offset_written(steady)),
        format!("TZOFFSETTO:{}", offset_written(steady)),
        "END:STANDARD".to_string(),
    ]
}

/// The description a yearly change rule carries: which month, and which
/// weekday of it, counted forward or as the last.
struct YearlyRule {
    month: u32,
    by_day: String,
}

/// The one yearly rule a group of changes all follow, or nothing when they
/// follow none.
///
/// A rule holds only when the window's every year has exactly one change in
/// this direction and they agree on month, weekday, local clock, both
/// offsets, and on being the same numbered weekday of the month, counted
/// forward or as the last. Anything less is no rule at all: a rule derived
/// from most of the years misplaces the other years' meetings silently.
fn one_yearly_rule(group: &[&Change], first_year: i32, last_year: i32) -> Option<YearlyRule> {
    let years: Vec<i32> = (first_year..=last_year).collect();
    if group.len() != years.len() {
        return None;
    }
    let onsets: Vec<NaiveDateTime> = group.iter().map(|change| change.local_onset()).collect();
    let first = *onsets.first()?;
    for (onset, year) in onsets.iter().zip(&years) {
        if onset.year() != *year
            || onset.month() != first.month()
            || onset.time() != first.time()
            || onset.weekday() != first.weekday()
        {
            return None;
        }
    }
    if group.iter().any(|change| {
        change.from_seconds != group[0].from_seconds || change.to_seconds != group[0].to_seconds
    }) {
        return None;
    }
    let nth_of = |onset: &NaiveDateTime| (onset.day() - 1) / 7 + 1;
    let is_last =
        |onset: &NaiveDateTime| onset.day() + 7 > days_in_month(onset.year(), onset.month());
    let by_day = if onsets.iter().all(|onset| nth_of(onset) == nth_of(&first)) {
        format!("{}{}", nth_of(&first), weekday_code(first.weekday()))
    } else if onsets.iter().all(is_last) {
        format!("-1{}", weekday_code(first.weekday()))
    } else {
        return None;
    };
    Some(YearlyRule {
        month: first.month(),
        by_day,
    })
}

/// One observance block described by a yearly rule, dated at the anchor
/// year's own onset.
fn ruled_block(kind: &str, group: &[&Change], rule: &YearlyRule, anchor_year: i32) -> Vec<String> {
    let onset = group
        .iter()
        .find(|change| change.local_onset().year() == anchor_year)
        .unwrap_or(&group[0]);
    vec![
        format!("BEGIN:{kind}"),
        format!("DTSTART:{}", onset.local_onset().format(WIRE_CLOCK_FACE)),
        format!("TZOFFSETFROM:{}", offset_written(onset.from_seconds)),
        format!("TZOFFSETTO:{}", offset_written(onset.to_seconds)),
        format!(
            "RRULE:FREQ=YEARLY;BYMONTH={};BYDAY={}",
            rule.month, rule.by_day
        ),
        format!("END:{kind}"),
    ]
}

/// Observance blocks that list their days one by one, for changes that
/// follow no yearly rule. One block per pair of offsets, naming every onset.
fn listed_blocks(kind: &str, group: &[&Change]) -> Vec<String> {
    let mut pairs: Vec<(i32, i32)> = Vec::new();
    for change in group {
        let pair = (change.from_seconds, change.to_seconds);
        if !pairs.contains(&pair) {
            pairs.push(pair);
        }
    }
    let mut lines = Vec::new();
    for (from, to) in pairs {
        let onsets: Vec<String> = group
            .iter()
            .filter(|change| change.from_seconds == from && change.to_seconds == to)
            .map(|change| change.local_onset().format(WIRE_CLOCK_FACE).to_string())
            .collect();
        lines.extend([
            format!("BEGIN:{kind}"),
            format!("DTSTART:{}", onsets[0]),
            format!("TZOFFSETFROM:{}", offset_written(from)),
            format!("TZOFFSETTO:{}", offset_written(to)),
            format!("RDATE:{}", onsets.join(",")),
            format!("END:{kind}"),
        ]);
    }
    lines
}

/// An offset written the way a calendar document writes one: `+0100`,
/// `-0430`.
fn offset_written(seconds: i32) -> String {
    let sign = if seconds < 0 { '-' } else { '+' };
    let magnitude = seconds.abs();
    format!(
        "{sign}{:02}{:02}",
        magnitude / 3600,
        (magnitude % 3600) / 60
    )
}

/// How many days the month holds, for telling a last weekday from a numbered
/// one.
fn days_in_month(year: i32, month: u32) -> u32 {
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    next_month
        .and_then(|first| first.pred_opt())
        .map_or(31, |last| last.day())
}

/// The two letters a calendar document writes a weekday as.
fn weekday_code(day: chrono::Weekday) -> &'static str {
    match day {
        chrono::Weekday::Mon => "MO",
        chrono::Weekday::Tue => "TU",
        chrono::Weekday::Wed => "WE",
        chrono::Weekday::Thu => "TH",
        chrono::Weekday::Fri => "FR",
        chrono::Weekday::Sat => "SA",
        chrono::Weekday::Sun => "SU",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The lines between `BEGIN:<kind>` and its `END:<kind>`, first block only.
    fn block(lines: &[String], kind: &str) -> Vec<String> {
        let opens = format!("BEGIN:{kind}");
        let closes = format!("END:{kind}");
        lines
            .iter()
            .skip_while(|line| **line != opens)
            .skip(1)
            .take_while(|line| **line != closes)
            .cloned()
            .collect()
    }

    /// The value of the one line in a block naming that property.
    fn value_of(block: &[String], name: &str) -> String {
        let head = format!("{name}:");
        block
            .iter()
            .find_map(|line| line.strip_prefix(&head))
            .unwrap_or_else(|| panic!("no {name} in {block:?}"))
            .to_string()
    }

    #[test]
    fn test_the_rules_for_a_zone_that_changes_carry_both_offsets_and_both_change_rules() {
        // London: last Sunday of March an hour forward at one in the morning,
        // last Sunday of October an hour back at two. Both offsets and both
        // change rules have to be in what a document defines, or a server
        // reading the definition puts half the year's meetings an hour off.
        let lines = timezone_rules_for("Europe/London", 2026).expect("a zone the database knows");

        assert_eq!(lines.first().map(String::as_str), Some("BEGIN:VTIMEZONE"));
        assert_eq!(lines.get(1).map(String::as_str), Some("TZID:Europe/London"));
        assert_eq!(lines.last().map(String::as_str), Some("END:VTIMEZONE"));
        let summer = block(&lines, "DAYLIGHT");
        assert_eq!(value_of(&summer, "TZOFFSETFROM"), "+0000");
        assert_eq!(value_of(&summer, "TZOFFSETTO"), "+0100");
        assert_eq!(
            value_of(&summer, "RRULE"),
            "FREQ=YEARLY;BYMONTH=3;BYDAY=-1SU"
        );
        assert_eq!(value_of(&summer, "DTSTART"), "20260329T010000");
        let winter = block(&lines, "STANDARD");
        assert_eq!(value_of(&winter, "TZOFFSETFROM"), "+0100");
        assert_eq!(value_of(&winter, "TZOFFSETTO"), "+0000");
        assert_eq!(
            value_of(&winter, "RRULE"),
            "FREQ=YEARLY;BYMONTH=10;BYDAY=-1SU"
        );
        assert_eq!(value_of(&winter, "DTSTART"), "20261025T020000");
    }

    #[test]
    fn test_the_rules_for_a_zone_that_counts_weeks_forward() {
        // New York counts its Sundays from the front of the month, the second
        // in March and the first in November, so this is the derivation the
        // London test cannot see: a numbered weekday rather than the last.
        let lines =
            timezone_rules_for("America/New_York", 2026).expect("a zone the database knows");

        let summer = block(&lines, "DAYLIGHT");
        assert_eq!(value_of(&summer, "TZOFFSETFROM"), "-0500");
        assert_eq!(value_of(&summer, "TZOFFSETTO"), "-0400");
        assert_eq!(
            value_of(&summer, "RRULE"),
            "FREQ=YEARLY;BYMONTH=3;BYDAY=2SU"
        );
        assert_eq!(value_of(&summer, "DTSTART"), "20260308T020000");
        let winter = block(&lines, "STANDARD");
        assert_eq!(value_of(&winter, "TZOFFSETFROM"), "-0400");
        assert_eq!(value_of(&winter, "TZOFFSETTO"), "-0500");
        assert_eq!(
            value_of(&winter, "RRULE"),
            "FREQ=YEARLY;BYMONTH=11;BYDAY=1SU"
        );
        assert_eq!(value_of(&winter, "DTSTART"), "20261101T020000");
    }

    #[test]
    fn test_a_zone_that_never_changes_writes_one_block_and_no_rule() {
        // Kolkata has kept one offset since before any calendar program. One
        // block, the same offset on both sides, and no change rule, because a
        // rule for a change that never comes describes a zone that does not
        // exist.
        let lines = timezone_rules_for("Asia/Kolkata", 2026).expect("a zone the database knows");

        assert!(
            !lines.iter().any(|line| line == "BEGIN:DAYLIGHT"),
            "{lines:?}"
        );
        assert!(
            !lines.iter().any(|line| line.starts_with("RRULE")),
            "{lines:?}"
        );
        let steady = block(&lines, "STANDARD");
        assert_eq!(value_of(&steady, "TZOFFSETFROM"), "+0530");
        assert_eq!(value_of(&steady, "TZOFFSETTO"), "+0530");
    }

    #[test]
    fn test_a_name_that_names_no_zone_writes_nothing() {
        // Nothing rather than a guess. The caller decides what a missing
        // answer means: the create path refuses to send and names the zone,
        // the change path keeps the server's document as it was.
        assert_eq!(timezone_rules_for("Middle-earth/Hobbiton", 2026), None);
        assert_eq!(timezone_rules_for("Pacific Standard Time", 2026), None);
    }

    #[test]
    fn test_a_zone_whose_changes_follow_no_yearly_rule_lists_its_days_rather_than_guessing() {
        // Casablanca moves its clocks by the lunar calendar, a different pair
        // of days every year, so no yearly rule describes it. A guessed rule
        // misplaces a meeting silently; a listed day cannot. Every listed
        // onset has to be a well-formed clock face.
        let lines =
            timezone_rules_for("Africa/Casablanca", 2026).expect("a zone the database knows");

        assert!(
            !lines.iter().any(|line| line.starts_with("RRULE")),
            "a yearly rule was guessed for a zone that follows none: {lines:?}"
        );
        let listed: Vec<&String> = lines
            .iter()
            .filter(|line| line.starts_with("RDATE:"))
            .collect();
        assert!(!listed.is_empty(), "no days listed at all: {lines:?}");
        for line in listed {
            for onset in line["RDATE:".len()..].split(',') {
                assert!(
                    chrono::NaiveDateTime::parse_from_str(onset, "%Y%m%dT%H%M%S").is_ok(),
                    "an onset that is not a clock face: {onset}"
                );
            }
        }
    }
}

//! Which days a repeating event actually falls on.

use crate::data::message_cache::CalendarEventEntry;

/// What is said about a series written in a way this cannot work out.
///
/// Said rather than guessed at. Showing nothing hides the event, and guessing
/// puts it on days it is not on: `repeating::Repeat::from_rule` answers "every
/// week" to an hourly rule, on purpose, because that is the least surprising
/// thing to show in a form control. In somebody's ear it is a lie.
pub const CANNOT_BE_READ: &str = "repeats, in a way this cannot work out, so it is shown once only";

/// One day a series falls on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Day {
    pub start: String,
    pub end: String,
}

/// Every day an event falls on inside a window, and what is said about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FallsOn {
    pub days: Vec<Day>,
    /// Empty for an event that does not repeat, so most events cost nothing.
    pub how_often: String,
}

/// How far either side of today a series is worked out.
///
/// The same window the syncs already ask the providers for, so a series from a
/// calendar server and one from Google reach as far as each other and somebody
/// with both accounts meets one behaviour rather than two.
pub const HOW_FAR_BACK: i64 = 180;
/// How far ahead a series is worked out. See [`HOW_FAR_BACK`].
pub const HOW_FAR_FORWARD: i64 = 365;

/// The most days one series is ever shown on.
///
/// The window bounds this already. It is here for the rule nobody thought of:
/// a list that grows without end is a calendar panel that never finishes
/// loading, and somebody would meet that as the application hanging.
const MOST_DAYS_ONE_SERIES_SHOWS: usize = 800;

/// The most times a rule is stepped forward before giving up.
///
/// A series set up years ago is stepped from its first day, so the count has to
/// cover the years before the window as well as the window itself. It is a
/// guard against a step that fails to move rather than a limit anybody meets.
const MOST_STEPS: usize = 40_000;

/// The days an event falls on between two dates.
///
/// An event with no rule falls on one day and costs nothing. An event with a
/// rule this cannot work out also falls on one day, and says so, because
/// showing nothing hides it and guessing puts it on days it is not on.
pub fn falls_on(
    event: &CalendarEventEntry,
    from: chrono::NaiveDate,
    to: chrono::NaiveDate,
) -> FallsOn {
    let (start, end) = when(event);
    let once = || FallsOn {
        days: vec![Day {
            start: start.clone(),
            end: end.clone(),
        }],
        how_often: String::new(),
    };

    let Some(written) = event
        .recurrence_rule
        .as_deref()
        .map(str::trim)
        .map(without_the_property_name)
    else {
        return once();
    };
    if written.is_empty() {
        return once();
    }
    let (Some(first), Some(rule)) = (date_in(&start), Rule::read(written)) else {
        return FallsOn {
            how_often: CANNOT_BE_READ.to_string(),
            ..once()
        };
    };

    let lasts = match (date_in(&start), date_in(&end)) {
        (Some(opens), Some(closes)) => (closes - opens).num_days(),
        _ => 0,
    };
    let called_off = days_called_off(event.exception_dates.as_deref());
    let days = rule
        .days_between(first, from, to)
        .into_iter()
        .filter(|day| !called_off.contains(day))
        .map(|day| Day {
            start: same_day_as(&start, day),
            end: same_day_as(&end, day + chrono::Duration::days(lasts)),
        })
        .collect();

    FallsOn {
        days,
        how_often: rule.spoken(written, &start),
    }
}

/// A stored rule without the property name some sources keep on the front.
///
/// Two shapes reach the one column: a calendar server's reader takes the name
/// off, and Google sends whole property lines and the converter keeps one. Both
/// mean the same rule, so both are read the same way.
fn without_the_property_name(written: &str) -> &str {
    let start = written
        .get(..6)
        .filter(|head| head.eq_ignore_ascii_case("RRULE:"))
        .map_or(0, str::len);
    written[start..].trim()
}

/// The date part of a stored when, whatever shape it was stored in.
///
/// A whole-day event holds a bare date, the editor here writes a date and a
/// clock face separated by a space, and both providers and a calendar server
/// write one separated by a T. All three start with the same ten characters.
fn date_in(stored: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(stored.get(..10)?, "%Y-%m-%d").ok()
}

/// The same stored when, moved to another day.
///
/// The clock face is carried across as the characters it was stored as, never
/// re-derived. That is what the calendar standard asks for, and it is why a
/// nine o'clock meeting is still at nine o'clock the week the clocks change:
/// adding seven days to an instant would move it an hour, for ever after.
fn same_day_as(stored: &str, day: chrono::NaiveDate) -> String {
    match stored.get(10..) {
        Some(clock) => format!("{day}{clock}"),
        None => day.to_string(),
    }
}

/// The days a series has called off.
///
/// Written as whole date and times, `20261225T090000Z`, or as bare dates, and
/// several of them separated by commas. Only the day is kept, because a day is
/// what the calendar shows and matching to the second would leave a cancelled
/// meeting on somebody's calendar over a server that wrote the seconds
/// differently.
///
/// A value that cannot be read is passed over rather than taken as calling off
/// nothing or calling off everything. Dropping the whole series over one bad
/// exclusion is how a calendar goes empty.
fn days_called_off(written: Option<&str>) -> std::collections::HashSet<chrono::NaiveDate> {
    let Some(written) = written else {
        return std::collections::HashSet::new();
    };
    written
        .split(',')
        .filter_map(|one| {
            let stamp: String = one.trim().chars().filter(char::is_ascii_digit).collect();
            chrono::NaiveDate::parse_from_str(stamp.get(..8)?, "%Y%m%d").ok()
        })
        .collect()
}

/// How often a rule comes round, at the coarseness a day can show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum How {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

/// A repeat rule this code is prepared to work out day by day.
///
/// Anything it is not prepared for is refused rather than approximated. The
/// event is then shown once and says so, which is a smaller wrong answer than
/// showing it on days it does not fall on.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Rule {
    how: How,
    every: u32,
    /// The weekdays a weekly rule names, in week order.
    weekdays: Vec<chrono::Weekday>,
    /// Which weekday of the month a monthly rule names, -1 being the last.
    nth_weekday: Option<(i32, chrono::Weekday)>,
    stops: crate::application::repeating::Until,
}

impl Rule {
    /// Read a stored rule, or refuse it.
    fn read(written: &str) -> Option<Self> {
        let text = written.trim().to_ascii_uppercase();
        let mut how = None;
        let mut every = 1u32;
        let mut byday = None;
        for part in text.split(';').map(str::trim).filter(|p| !p.is_empty()) {
            let (key, value) = part.split_once('=')?;
            match key {
                "FREQ" => {
                    how = Some(match value {
                        "DAILY" => How::Daily,
                        "WEEKLY" => How::Weekly,
                        "MONTHLY" => How::Monthly,
                        "YEARLY" => How::Yearly,
                        // Hourly and finer. A day cannot show one of those as a
                        // day, so there is nothing honest to draw.
                        _ => return None,
                    });
                }
                "INTERVAL" => {
                    // Nought is not a rule, it is a loop that never advances.
                    every = value.parse().ok().filter(|n| *n > 0)?;
                }
                "BYDAY" => byday = Some(value.to_string()),
                // Read whole by `repeating::Until`, which is exact.
                "COUNT" | "UNTIL" => {}
                // Which day a week starts on only changes the answer for a rule
                // counting weeks in a way this does not do, so any other answer
                // is a rule this has no business guessing at.
                "WKST" => {
                    if value != "MO" {
                        return None;
                    }
                }
                _ => return None,
            }
        }
        let how = how?;
        let (weekdays, nth_weekday) = match (how, byday.as_deref()) {
            (_, None) => (Vec::new(), None),
            (How::Weekly, Some(named)) => (every_weekday_in(named)?, None),
            (How::Monthly, Some(named)) => (Vec::new(), Some(one_weekday_of_the_month(named)?)),
            // A weekday named on a daily or yearly rule narrows it in a way
            // this does not work out.
            _ => return None,
        };
        Some(Rule {
            how,
            every,
            weekdays,
            nth_weekday,
            stops: crate::application::repeating::Until::from_rule(&text),
        })
    }

    /// Every day the series falls on inside the window.
    ///
    /// Stepped from the day the series starts rather than from the window,
    /// because a count counts from the start: counted from the window, a series
    /// that ran out last month would go on showing.
    fn days_between(
        &self,
        first: chrono::NaiveDate,
        from: chrono::NaiveDate,
        to: chrono::NaiveDate,
    ) -> Vec<chrono::NaiveDate> {
        use crate::application::repeating::Until;

        let stop_on = match &self.stops {
            Until::OnDate(date) => chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").ok(),
            _ => None,
        };
        let most_times = match self.stops {
            Until::AfterTimes(times) => Some(times as usize),
            _ => None,
        };

        let mut found = Vec::new();
        let mut counted = 0usize;
        let mut cursor = self.opening(first);
        for _ in 0..MOST_STEPS {
            for day in self.days_at(cursor, first) {
                if day < first {
                    continue;
                }
                if stop_on.is_some_and(|end| day > end) {
                    return found;
                }
                counted += 1;
                if most_times.is_some_and(|most| counted > most) {
                    return found;
                }
                if day > to {
                    return found;
                }
                if day >= from {
                    found.push(day);
                    if found.len() >= MOST_DAYS_ONE_SERIES_SHOWS {
                        return found;
                    }
                }
            }
            let Some(next) = self.step(cursor) else {
                return found;
            };
            // A step that does not move is a rule that would be worked out for
            // ever. Nothing known produces one; this is what stops the one
            // nobody thought of from hanging the panel.
            if next <= cursor {
                return found;
            }
            cursor = next;
        }
        found
    }

    /// Where the stepping starts, which is the block the first day sits in
    /// rather than the day itself: a weekly rule naming several weekdays has to
    /// look at the whole week, and a monthly rule at the whole month.
    fn opening(&self, first: chrono::NaiveDate) -> chrono::NaiveDate {
        use chrono::Datelike;
        match self.how {
            How::Daily => first,
            How::Weekly if self.weekdays.is_empty() => first,
            How::Weekly => {
                first - chrono::Duration::days(first.weekday().num_days_from_monday().into())
            }
            How::Monthly => first.with_day(1).unwrap_or(first),
            How::Yearly => first
                .with_day(1)
                .and_then(|d| d.with_month(1))
                .unwrap_or(first),
        }
    }

    /// The days one step of the rule lands on, in order.
    fn days_at(
        &self,
        cursor: chrono::NaiveDate,
        first: chrono::NaiveDate,
    ) -> Vec<chrono::NaiveDate> {
        use chrono::Datelike;
        match self.how {
            How::Daily => vec![cursor],
            How::Weekly if self.weekdays.is_empty() => vec![cursor],
            How::Weekly => self
                .weekdays
                .iter()
                .map(|day| cursor + chrono::Duration::days(day.num_days_from_monday().into()))
                .collect(),
            How::Monthly => match self.nth_weekday {
                Some((nth, weekday)) => nth_weekday_of(cursor.year(), cursor.month(), nth, weekday)
                    .into_iter()
                    .collect(),
                // A month with no such date is skipped, never clamped. Clamping
                // puts a month end meeting on a day it was never on.
                None => chrono::NaiveDate::from_ymd_opt(cursor.year(), cursor.month(), first.day())
                    .into_iter()
                    .collect(),
            },
            // The 29th of February is skipped in the years that have none, for
            // the same reason.
            How::Yearly => {
                chrono::NaiveDate::from_ymd_opt(cursor.year(), first.month(), first.day())
                    .into_iter()
                    .collect()
            }
        }
    }

    /// The next block after this one.
    ///
    /// The gap is worked out wider than the interval was read in. INTERVAL has
    /// no upper limit in the standard and none in the parser, and at the top of
    /// its own width a week count overflows. A month count that wrapped would
    /// be worse than a crash: it is no longer a whole number of years, so a
    /// yearly rule would step to a month it never named, and every day worked
    /// out from there belongs to a rule nobody wrote.
    fn step(&self, cursor: chrono::NaiveDate) -> Option<chrono::NaiveDate> {
        let every = i64::from(self.every);
        match self.how {
            How::Daily => cursor.checked_add_signed(chrono::Duration::days(every)),
            How::Weekly => cursor.checked_add_signed(chrono::Duration::days(every * 7)),
            How::Monthly => months_after(cursor, every),
            How::Yearly => months_after(cursor, every * 12),
        }
    }
}

impl Rule {
    /// What is said about this rule when the event is read out.
    ///
    /// Where the rule is one this application writes itself, the words come
    /// from `repeating::spoken`, so a series reads the same in the item form
    /// and in the calendar. That is decided by writing every choice out and
    /// comparing, which cannot drift and cannot flatter a rule it half
    /// recognises. Anything else is described here from what was parsed.
    fn spoken(&self, written: &str, start: &str) -> String {
        use crate::application::repeating::{self, Repeat};

        let weekday = repeating::weekday_of_month(start);
        for repeat in Repeat::ALL {
            if repeating::rule(repeat, &self.stops, weekday.as_deref())
                .is_some_and(|offered| offered.eq_ignore_ascii_case(written.trim()))
            {
                return repeating::spoken(repeat, &self.stops);
            }
        }
        with_the_ending(&self.how_often(), &self.stops)
    }

    /// How often this comes round, in words, without the ending.
    fn how_often(&self) -> String {
        let unit = match self.how {
            How::Daily => "day",
            How::Weekly => "week",
            How::Monthly => "month",
            How::Yearly => "year",
        };
        let how_often = if self.every == 1 {
            format!("every {unit}")
        } else {
            format!("every {} {unit}s", self.every)
        };
        if self.weekdays.is_empty() {
            return how_often;
        }
        let named: Vec<&str> = self.weekdays.iter().map(weekday_called).collect();
        format!("{how_often} on {}", one_after_another(&named))
    }
}

/// A sentence with when it stops added, in the words `repeating::spoken` uses.
fn with_the_ending(how_often: &str, stops: &crate::application::repeating::Until) -> String {
    use crate::application::repeating::Until;
    match stops {
        Until::Forever => how_often.to_string(),
        Until::OnDate(date) => format!("{how_often}, until {date}"),
        Until::AfterTimes(1) => format!("{how_often}, once"),
        Until::AfterTimes(times) => format!("{how_often}, {times} times"),
    }
}

/// Several things said as a list somebody would say out loud.
fn one_after_another(names: &[&str]) -> String {
    match names {
        [] => String::new(),
        [only] => (*only).to_string(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// What a weekday is called when it is read out, said in full.
fn weekday_called(day: &chrono::Weekday) -> &'static str {
    match day {
        chrono::Weekday::Mon => "Monday",
        chrono::Weekday::Tue => "Tuesday",
        chrono::Weekday::Wed => "Wednesday",
        chrono::Weekday::Thu => "Thursday",
        chrono::Weekday::Fri => "Friday",
        chrono::Weekday::Sat => "Saturday",
        chrono::Weekday::Sun => "Sunday",
    }
}

/// The first of the month a number of months after this one.
fn months_after(date: chrono::NaiveDate, months: i64) -> Option<chrono::NaiveDate> {
    use chrono::Datelike;
    let counted = date.year() as i64 * 12 + i64::from(date.month()) - 1 + months;
    let year = i32::try_from(counted.div_euclid(12)).ok()?;
    let month = u32::try_from(counted.rem_euclid(12)).ok()? + 1;
    chrono::NaiveDate::from_ymd_opt(year, month, 1)
}

/// The nth weekday of a month, counting back from the end when nth is negative.
///
/// `None` for a month that has no such day, which is what makes a fifth Tuesday
/// skip the months without one instead of landing in the next month.
fn nth_weekday_of(
    year: i32,
    month: u32,
    nth: i32,
    weekday: chrono::Weekday,
) -> Option<chrono::NaiveDate> {
    use chrono::Datelike;
    let first = chrono::NaiveDate::from_ymd_opt(year, month, 1)?;
    let forward = (7 + weekday.num_days_from_monday() as i64
        - first.weekday().num_days_from_monday() as i64)
        % 7;
    let earliest = first + chrono::Duration::days(forward);
    let found = if nth > 0 {
        earliest + chrono::Duration::days(7 * i64::from(nth - 1))
    } else {
        let last_of_month = months_after(first, 1)?.pred_opt()?;
        let back = (7 + last_of_month.weekday().num_days_from_monday() as i64
            - weekday.num_days_from_monday() as i64)
            % 7;
        last_of_month - chrono::Duration::days(back + 7 * i64::from(-nth - 1))
    };
    (found.month() == month && found.year() == year).then_some(found)
}

/// The weekdays a rule names, in week order, or nothing if any of them is not
/// a weekday this understands.
fn every_weekday_in(named: &str) -> Option<Vec<chrono::Weekday>> {
    let mut found = Vec::new();
    for name in named.split(',').map(str::trim) {
        found.push(weekday_named(name)?);
    }
    if found.is_empty() {
        return None;
    }
    found.sort_by_key(chrono::Weekday::num_days_from_monday);
    found.dedup();
    Some(found)
}

/// The one weekday of the month a rule names, as in `2TU` or `-1FR`.
fn one_weekday_of_the_month(named: &str) -> Option<(i32, chrono::Weekday)> {
    let named = named.trim();
    if named.contains(',') {
        return None;
    }
    let split = named.len().checked_sub(2)?;
    let (ordinal, day) = (named.get(..split)?, named.get(split..)?);
    let weekday = weekday_named(day)?;
    // A bare weekday on a monthly rule means every one of them in the month,
    // which is a different rule and not one this works out.
    //
    // Five is as many of one weekday as any month can hold, counted from either
    // end, so anything outside that names a day no month has. It was let
    // through, and it went two ways: a sixth Monday fell on nothing, every
    // month, so the event was shown on no day at all, and a number near the top
    // of its own width ran the date arithmetic off the end of the calendar and
    // took the calendar list down with it. Refused here, the event is shown once
    // and says why, which is what this module does with every rule it cannot
    // work out.
    let nth: i32 = ordinal
        .parse()
        .ok()
        .filter(|n| (-5..=5).contains(n) && *n != 0)?;
    Some((nth, weekday))
}

/// The two letters a rule writes a weekday as.
fn weekday_named(name: &str) -> Option<chrono::Weekday> {
    Some(match name {
        "MO" => chrono::Weekday::Mon,
        "TU" => chrono::Weekday::Tue,
        "WE" => chrono::Weekday::Wed,
        "TH" => chrono::Weekday::Thu,
        "FR" => chrono::Weekday::Fri,
        "SA" => chrono::Weekday::Sat,
        "SU" => chrono::Weekday::Sun,
        _ => return None,
    })
}

/// Which pair of stored columns says when an event is.
///
/// A whole-day event keeps its dates in their own columns, and reading the
/// datetime pair for one announces a time it does not have.
pub fn when(event: &CalendarEventEntry) -> (String, String) {
    if event.is_all_day {
        (
            event
                .start_date
                .clone()
                .unwrap_or_else(|| event.start_datetime.clone()),
            event
                .end_date
                .clone()
                .unwrap_or_else(|| event.end_datetime.clone()),
        )
    } else {
        (event.start_datetime.clone(), event.end_datetime.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::message_cache::CalendarEventEntry;

    fn an_event(start: &str, end: &str, rule: Option<&str>) -> CalendarEventEntry {
        CalendarEventEntry {
            id: "e1".to_string(),
            account_id: "a1".to_string(),
            provider_event_id: None,
            calendar_id: None,
            summary: "Standup".to_string(),
            description: None,
            location: None,
            start_datetime: start.to_string(),
            end_datetime: end.to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: None,
            status: "confirmed".to_string(),
            recurrence_rule: rule.map(str::to_string),
            categories: String::new(),
            source_provider: Some("local".to_string()),
            etag: None,
            web_link: None,
            show_as: "busy".to_string(),
            last_modified_remote: None,
            last_synced_at: None,
            attendees_json: None,
            reminders_json: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            pending: false,
            exception_dates: None,
        }
    }

    fn window() -> (chrono::NaiveDate, chrono::NaiveDate) {
        between("2026-03-01", "2026-04-05")
    }

    fn between(from: &str, to: &str) -> (chrono::NaiveDate, chrono::NaiveDate) {
        let read = |d: &str| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").expect("a date");
        (read(from), read(to))
    }

    fn starts(shown: &FallsOn) -> Vec<String> {
        shown.days.iter().map(|d| d.start.clone()).collect()
    }

    #[test]
    fn test_an_event_that_does_not_repeat_falls_on_one_day() {
        // The commonest event there is, and it must cost nothing: one day, and
        // nothing said about repeating.
        let event = an_event("2026-03-05 09:00", "2026-03-05 09:15", None);
        let (from, to) = window();

        let shown = falls_on(&event, from, to);

        assert_eq!(shown.days.len(), 1);
        assert_eq!(shown.days[0].start, "2026-03-05 09:00");
        assert_eq!(shown.days[0].end, "2026-03-05 09:15");
        assert_eq!(shown.how_often, "");
    }

    #[test]
    fn test_a_weekly_meeting_falls_on_every_week_inside_the_window() {
        // The whole point of the unit. A weekly meeting used to appear once.
        let event = an_event("2026-03-05 09:00", "2026-03-05 09:15", Some("FREQ=WEEKLY"));
        let (from, to) = window();

        let shown = falls_on(&event, from, to);

        assert_eq!(
            starts(&shown),
            [
                "2026-03-05 09:00",
                "2026-03-12 09:00",
                "2026-03-19 09:00",
                "2026-03-26 09:00",
                "2026-04-02 09:00",
            ]
        );
        // The end moves with the start, or every day after the first is an
        // appointment that ended before it began.
        assert_eq!(shown.days[1].end, "2026-03-12 09:15");
    }

    #[test]
    fn test_a_series_is_not_expanded_past_the_window() {
        // Without a bound a rule with no ending expands for ever.
        let event = an_event("2026-03-05 09:00", "2026-03-05 09:15", Some("FREQ=DAILY"));
        let (from, to) = between("2026-03-01", "2026-03-08");

        let shown = falls_on(&event, from, to);

        assert_eq!(shown.days.len(), 4, "{:?}", starts(&shown));
        assert_eq!(shown.days[3].start, "2026-03-08 09:00");
    }

    #[test]
    fn test_a_series_that_began_before_the_window_still_shows_inside_it() {
        // The commonest real series there is: a standing meeting set up years
        // ago. Starting the arithmetic at the window rather than at the series
        // would put it on the wrong weekday.
        let event = an_event("2020-01-06 09:00", "2020-01-06 09:15", Some("FREQ=WEEKLY"));
        let (from, to) = between("2026-03-01", "2026-03-20");

        let shown = falls_on(&event, from, to);

        // The 6th of January 2020 was a Monday, so every occurrence is one.
        assert_eq!(
            starts(&shown),
            ["2026-03-02 09:00", "2026-03-09 09:00", "2026-03-16 09:00"]
        );
    }

    #[test]
    fn test_a_rule_naming_two_weekdays_falls_on_both_of_them() {
        // The case `repeating::Repeat::from_rule` answers "every week" to, and
        // the reason the reading must not be built out of it.
        let event = an_event(
            "2026-03-03 09:00",
            "2026-03-03 09:15",
            Some("FREQ=WEEKLY;BYDAY=TU,TH"),
        );
        let (from, to) = between("2026-03-01", "2026-03-15");

        let shown = falls_on(&event, from, to);

        assert_eq!(
            starts(&shown),
            [
                "2026-03-03 09:00",
                "2026-03-05 09:00",
                "2026-03-10 09:00",
                "2026-03-12 09:00",
            ]
        );
    }

    #[test]
    fn test_every_weekday_a_rule_can_name_is_read_as_the_day_it_names() {
        // Every one of the seven, both the day it lands on and the sentence.
        // The start is the Monday for all seven rows on purpose: a weekday this
        // stopped recognising makes the whole rule unreadable, and the event
        // then falls back to its own start day, so a row asserting only the day
        // would still pass for Monday. Asserting the sentence as well is what
        // makes a dropped weekday visible.
        for (named, day, said) in [
            ("MO", "2026-03-02 09:00", "every week on Monday"),
            ("TU", "2026-03-03 09:00", "every week on Tuesday"),
            ("WE", "2026-03-04 09:00", "every week on Wednesday"),
            ("TH", "2026-03-05 09:00", "every week on Thursday"),
            ("FR", "2026-03-06 09:00", "every week on Friday"),
            ("SA", "2026-03-07 09:00", "every week on Saturday"),
            ("SU", "2026-03-08 09:00", "every week on Sunday"),
        ] {
            let event = an_event(
                "2026-03-02 09:00",
                "2026-03-02 09:15",
                Some(&format!("FREQ=WEEKLY;BYDAY={named}")),
            );
            let (from, to) = between("2026-03-02", "2026-03-08");

            let shown = falls_on(&event, from, to);

            assert_eq!(starts(&shown), [day], "for {named}");
            assert_eq!(shown.how_often, said, "for {named}");
        }
    }

    #[test]
    fn test_every_other_week_skips_the_week_between() {
        let event = an_event(
            "2026-03-05 09:00",
            "2026-03-05 09:15",
            Some("FREQ=WEEKLY;INTERVAL=2"),
        );
        let (from, to) = window();

        let shown = falls_on(&event, from, to);

        assert_eq!(
            starts(&shown),
            ["2026-03-05 09:00", "2026-03-19 09:00", "2026-04-02 09:00"]
        );
    }

    #[test]
    fn test_an_interval_no_calendar_could_mean_shows_the_first_day_and_stops() {
        // INTERVAL has no upper limit in the standard and none in the parser,
        // and the step used to work the gap out in the width the interval was
        // read in. A week count that wide overflows, and a month count that
        // wraps is no longer a whole number of years, which lands the cursor in
        // a month the rule never named. The first day is still the first day,
        // so it is shown, and the step then runs off the end of the calendar
        // and stops.
        for absurd in [
            "FREQ=WEEKLY;INTERVAL=4294967295",
            "FREQ=YEARLY;INTERVAL=400000000",
        ] {
            let event = an_event("2026-03-05 09:00", "2026-03-05 09:15", Some(absurd));
            let (from, to) = window();

            let shown = falls_on(&event, from, to);

            assert_eq!(starts(&shown), ["2026-03-05 09:00"], "for {absurd}");
        }
    }

    #[test]
    fn test_a_monthly_series_on_the_thirty_first_skips_the_months_that_have_no_thirty_first() {
        // Clamping to the 28th or the 30th would put somebody's month end
        // meeting on a day it was never on, every other month, for ever.
        let event = an_event("2026-01-31 09:00", "2026-01-31 10:00", Some("FREQ=MONTHLY"));
        let (from, to) = between("2026-01-01", "2026-06-30");

        let shown = falls_on(&event, from, to);

        assert_eq!(
            starts(&shown),
            ["2026-01-31 09:00", "2026-03-31 09:00", "2026-05-31 09:00"]
        );
    }

    #[test]
    fn test_a_monthly_series_on_a_weekday_of_the_month_keeps_that_weekday() {
        // The shape this application writes for "every month on this weekday".
        // The 9th of July 2026 is the second Thursday.
        let event = an_event(
            "2026-07-09 09:00",
            "2026-07-09 10:00",
            Some("FREQ=MONTHLY;BYDAY=2TH"),
        );
        let (from, to) = between("2026-07-01", "2026-09-30");

        let shown = falls_on(&event, from, to);

        assert_eq!(
            starts(&shown),
            ["2026-07-09 09:00", "2026-08-13 09:00", "2026-09-10 09:00"]
        );
    }

    #[test]
    fn test_a_monthly_series_on_the_last_weekday_of_the_month_is_the_last_one() {
        // Written as -1 rather than 5 by `repeating::weekday_of_month`, so a
        // month with four of that weekday is not skipped.
        let event = an_event(
            "2026-07-30 09:00",
            "2026-07-30 10:00",
            Some("FREQ=MONTHLY;BYDAY=-1TH"),
        );
        let (from, to) = between("2026-07-01", "2026-09-30");

        let shown = falls_on(&event, from, to);

        assert_eq!(
            starts(&shown),
            ["2026-07-30 09:00", "2026-08-27 09:00", "2026-09-24 09:00"]
        );
    }

    #[test]
    fn test_a_fifth_weekday_rule_skips_the_months_that_have_no_fifth_one() {
        // Only three months of 2026 have a fifth Tuesday. The months without
        // one are skipped, never allowed to run on into the next month, or a
        // meeting set for the fifth Tuesday turns up on the first one instead.
        let event = an_event(
            "2026-03-31 09:00",
            "2026-03-31 10:00",
            Some("FREQ=MONTHLY;BYDAY=5TU"),
        );
        let (from, to) = between("2026-03-01", "2026-09-30");

        let shown = falls_on(&event, from, to);

        assert_eq!(
            starts(&shown),
            ["2026-03-31 09:00", "2026-06-30 09:00", "2026-09-29 09:00"]
        );
    }

    #[test]
    fn test_the_second_to_last_weekday_of_the_month_is_counted_back_from_the_last_one() {
        // Second to last rather than last, because counting back from the last
        // one is arithmetically the same as landing on it when the ordinal is
        // -1: nothing is added either way. Only an ordinal of -2 or beyond can
        // tell the counting back from the landing.
        let event = an_event(
            "2026-06-23 09:00",
            "2026-06-23 10:00",
            Some("FREQ=MONTHLY;BYDAY=-2TU"),
        );
        let (from, to) = between("2026-06-01", "2026-09-30");

        let shown = falls_on(&event, from, to);

        assert_eq!(
            starts(&shown),
            [
                "2026-06-23 09:00",
                "2026-07-21 09:00",
                "2026-08-18 09:00",
                "2026-09-22 09:00",
            ]
        );
    }

    #[test]
    fn test_a_monthly_rule_naming_a_weekday_no_month_has_is_shown_once_and_says_so() {
        // No month has a sixth Monday, so a rule asking for one falls on no day
        // ever, and the event was shown on none. The two large ordinals are
        // worse than that: the date arithmetic ran off the end of the calendar
        // and took the whole calendar list down with it. Both arrive here as
        // they were written, because nothing between a calendar server and this
        // code looks at what BYDAY says.
        for impossible in [
            "FREQ=MONTHLY;BYDAY=6MO",
            "FREQ=MONTHLY;BYDAY=-6MO",
            "FREQ=MONTHLY;BYDAY=2147483647MO",
            "FREQ=MONTHLY;BYDAY=-2147483648MO",
        ] {
            let event = an_event("2026-03-05 09:00", "2026-03-05 09:15", Some(impossible));
            let (from, to) = window();

            let shown = falls_on(&event, from, to);

            assert_eq!(starts(&shown), ["2026-03-05 09:00"], "for {impossible}");
            assert_eq!(shown.how_often, CANNOT_BE_READ, "for {impossible}");
        }
    }

    #[test]
    fn test_a_weekly_series_keeps_its_clock_face_across_a_change_of_the_clocks() {
        // British clocks go forward on the 29th of March 2026. A nine o'clock
        // meeting is at nine o'clock on the 1st of April, which is what the
        // calendar standard says and what adding seven days to an instant
        // would get wrong by an hour for ever after.
        let mut event = an_event(
            "2026-03-25T09:00:00",
            "2026-03-25T09:30:00",
            Some("FREQ=WEEKLY"),
        );
        event.time_zone = Some("Europe/London".to_string());
        let (from, to) = between("2026-03-01", "2026-04-10");

        let shown = falls_on(&event, from, to);

        assert_eq!(
            starts(&shown),
            [
                "2026-03-25T09:00:00",
                "2026-04-01T09:00:00",
                "2026-04-08T09:00:00",
            ]
        );
    }

    #[test]
    fn test_a_series_that_stops_after_a_count_stops_there() {
        let event = an_event(
            "2026-03-05 09:00",
            "2026-03-05 09:15",
            Some("FREQ=DAILY;COUNT=3"),
        );
        let (from, to) = window();

        let shown = falls_on(&event, from, to);

        assert_eq!(
            starts(&shown),
            ["2026-03-05 09:00", "2026-03-06 09:00", "2026-03-07 09:00"]
        );
    }

    #[test]
    fn test_a_count_is_counted_from_the_start_of_the_series_not_from_the_window() {
        // Counted from the window, a series that ran out last month would go on
        // showing for another three days.
        let event = an_event(
            "2026-02-01 09:00",
            "2026-02-01 09:15",
            Some("FREQ=DAILY;COUNT=3"),
        );
        let (from, to) = window();

        let shown = falls_on(&event, from, to);

        assert!(shown.days.is_empty(), "{:?}", starts(&shown));
    }

    #[test]
    fn test_a_series_that_stops_on_a_date_includes_that_date() {
        // `repeating::Until::ending` writes T235959Z for exactly this reason:
        // "until the tenth" means the tenth counts.
        let event = an_event(
            "2026-03-05 09:00",
            "2026-03-05 09:15",
            Some("FREQ=DAILY;UNTIL=20260310T235959Z"),
        );
        let (from, to) = window();

        let shown = falls_on(&event, from, to);

        assert_eq!(shown.days.len(), 6, "{:?}", starts(&shown));
        assert_eq!(shown.days[5].start, "2026-03-10 09:00");
    }

    #[test]
    fn test_an_all_day_series_stays_all_day_on_every_day_it_falls_on() {
        // Reading the datetime columns would turn every one after the first
        // into a midnight appointment.
        let mut event = an_event(
            "2026-03-05T00:00:00Z",
            "2026-03-06T00:00:00Z",
            Some("FREQ=WEEKLY"),
        );
        event.is_all_day = true;
        event.start_date = Some("2026-03-05".to_string());
        event.end_date = Some("2026-03-06".to_string());
        let (from, to) = between("2026-03-01", "2026-03-20");

        let shown = falls_on(&event, from, to);

        assert_eq!(
            starts(&shown),
            ["2026-03-05", "2026-03-12", "2026-03-19"],
            "a whole day must not gain a time"
        );
        assert_eq!(shown.days[1].end, "2026-03-13");
    }

    #[test]
    fn test_a_series_lasting_more_than_one_day_keeps_its_length() {
        let event = an_event("2026-03-05 09:00", "2026-03-07 17:00", Some("FREQ=WEEKLY"));
        let (from, to) = between("2026-03-01", "2026-03-20");

        let shown = falls_on(&event, from, to);

        assert_eq!(shown.days[1].start, "2026-03-12 09:00");
        assert_eq!(shown.days[1].end, "2026-03-14 17:00");
    }

    #[test]
    fn test_a_rule_this_cannot_read_shows_the_event_once_and_says_so() {
        // Showing nothing hides the event. Guessing puts it on days it is not
        // on: `repeating::Repeat::from_rule` answers "every week" to this one.
        for unreadable in [
            "FREQ=HOURLY;INTERVAL=6",
            "FREQ=MINUTELY",
            "FREQ=WEEKLY;INTERVAL=0",
            "FREQ=MONTHLY;BYSETPOS=2;BYDAY=TU",
            "FREQ=YEARLY;BYMONTH=3;BYMONTHDAY=15",
            "not a rule at all",
            // A weekday written the way somebody says it rather than the two
            // letters a rule uses, and a list with one good name in it. Taking
            // either as a weekday would put the series on days of somebody
            // else's choosing.
            "FREQ=WEEKLY;BYDAY=SUN",
            "FREQ=WEEKLY;BYDAY=MO,XX",
        ] {
            let event = an_event("2026-03-05 09:00", "2026-03-05 09:15", Some(unreadable));
            let (from, to) = window();

            let shown = falls_on(&event, from, to);

            assert_eq!(shown.days.len(), 1, "for {unreadable}");
            assert_eq!(shown.days[0].start, "2026-03-05 09:00", "for {unreadable}");
            assert_eq!(shown.how_often, CANNOT_BE_READ, "for {unreadable}");
        }
    }

    #[test]
    fn test_a_rule_that_still_carries_its_property_name_is_read_the_same() {
        // Two shapes reach this one column. A calendar server's rule arrives
        // bare, because the reader strips the property name; Google sends a
        // list of whole property lines and the converter keeps one of them. A
        // reader that only knows the bare shape shows every Google series once
        // and says it could not be read.
        let event = an_event(
            "2026-03-05 09:00",
            "2026-03-05 09:15",
            Some("RRULE:FREQ=WEEKLY"),
        );
        let (from, to) = window();

        let shown = falls_on(&event, from, to);

        assert_eq!(shown.days.len(), 5, "{:?}", starts(&shown));
        assert_eq!(shown.how_often, "every week");
    }

    #[test]
    fn test_a_day_the_series_excludes_is_not_shown() {
        // A cancelled day of a series. Shown, it is a meeting somebody turns up
        // to that is not happening, which is worse than not expanding at all.
        let mut event = an_event("2026-03-05 09:00", "2026-03-05 09:15", Some("FREQ=WEEKLY"));
        event.exception_dates = Some("20260312T090000Z,20260326T090000Z".to_string());
        let (from, to) = window();

        let shown = falls_on(&event, from, to);

        assert_eq!(
            starts(&shown),
            ["2026-03-05 09:00", "2026-03-19 09:00", "2026-04-02 09:00"]
        );
    }

    #[test]
    fn test_an_excluded_day_written_as_a_bare_date_is_still_excluded() {
        // What a whole-day series carries, and what several servers send for a
        // timed one as well.
        let mut event = an_event("2026-03-05 09:00", "2026-03-05 09:15", Some("FREQ=WEEKLY"));
        event.exception_dates = Some("20260312".to_string());
        let (from, to) = between("2026-03-01", "2026-03-20");

        assert_eq!(
            starts(&falls_on(&event, from, to)),
            ["2026-03-05 09:00", "2026-03-19 09:00"]
        );
    }

    #[test]
    fn test_an_excluded_day_nobody_can_read_leaves_the_series_alone() {
        // Dropping every day of a series because one exclusion would not parse
        // is how a calendar goes empty.
        let mut event = an_event("2026-03-05 09:00", "2026-03-05 09:15", Some("FREQ=WEEKLY"));
        event.exception_dates = Some("not a date at all".to_string());
        let (from, to) = between("2026-03-01", "2026-03-20");

        assert_eq!(falls_on(&event, from, to).days.len(), 3);
    }

    #[test]
    fn test_a_series_says_how_often_it_repeats_and_when_it_stops() {
        // Nothing said an event repeated at all. Where the rule is one this
        // application writes, the words are the ones the item form shows, so
        // one series is described one way wherever it is met (WCAG 3.2.4).
        for (rule, expected) in [
            ("FREQ=WEEKLY", "every week"),
            ("FREQ=WEEKLY;INTERVAL=2", "every two weeks"),
            (
                "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR",
                "every weekday, monday to friday",
            ),
            ("FREQ=WEEKLY;COUNT=6", "every week, 6 times"),
            (
                "FREQ=WEEKLY;UNTIL=20260930T235959Z",
                "every week, until 2026-09-30",
            ),
            ("FREQ=MONTHLY;INTERVAL=3", "every three months"),
            ("FREQ=YEARLY", "every year"),
            // Rules no choice in the item form can express, described here.
            (
                "FREQ=WEEKLY;BYDAY=TU,TH",
                "every week on Tuesday and Thursday",
            ),
            (
                "FREQ=WEEKLY;BYDAY=MO,WE,FR;COUNT=6",
                "every week on Monday, Wednesday and Friday, 6 times",
            ),
            ("FREQ=DAILY;INTERVAL=3", "every 3 days"),
        ] {
            let event = an_event("2026-03-05 09:00", "2026-03-05 09:15", Some(rule));
            let (from, to) = window();

            assert_eq!(falls_on(&event, from, to).how_often, expected, "for {rule}");
        }
    }

    #[test]
    fn test_a_monthly_series_on_a_weekday_is_described_the_way_the_form_describes_it() {
        // The weekday comes out of the date the series starts, so this one can
        // only be recognised with the start in hand.
        let event = an_event(
            "2026-07-09 09:00",
            "2026-07-09 10:00",
            Some("FREQ=MONTHLY;BYDAY=2TH"),
        );
        let (from, to) = between("2026-07-01", "2026-09-30");

        assert_eq!(
            falls_on(&event, from, to).how_often,
            "every month, on this weekday"
        );
    }

    #[test]
    fn test_the_sentence_for_a_rule_nobody_can_read_says_what_to_expect() {
        // It is the one case where somebody has to be told without asking, so
        // the words have to carry the whole story on their own.
        assert!(CANNOT_BE_READ.contains("repeats"), "{CANNOT_BE_READ}");
        assert!(CANNOT_BE_READ.contains("once"), "{CANNOT_BE_READ}");
        assert!(
            !CANNOT_BE_READ.contains("RRULE") && !CANNOT_BE_READ.contains("rule"),
            "no machine words at somebody: {CANNOT_BE_READ}"
        );
    }
}

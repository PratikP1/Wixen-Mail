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
/// The window ends the stepping for anything that reaches it, and the stepping
/// now starts at the block the window opens in, so a rule that falls on a day
/// takes about as many steps as the window is wide. This is the backstop for
/// what neither of those covers: a rule that falls on no day at all, and a
/// month or year rule counting its way to an ending, which is still walked from
/// its own first day because how many days it has fallen on cannot be worked
/// out any other way. Forty thousand months is three thousand three hundred
/// years and forty thousand years is forty thousand years, so no first date a
/// calendar carries reaches this.
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
    let called_off = days_called_off(event.exception_dates.as_deref(), the_offset_on(&start));
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

/// The days a series has called off, as days in the event's own zone.
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
fn days_called_off(
    written: Option<&str>,
    the_events_offset: Option<chrono::FixedOffset>,
) -> std::collections::HashSet<chrono::NaiveDate> {
    let Some(written) = written else {
        return std::collections::HashSet::new();
    };
    written
        .split(',')
        .filter_map(|one| {
            the_day_called_off(
                without_the_name_and_parameters(one.trim()),
                the_events_offset,
            )
        })
        .collect()
}

/// The day one called-off value names, counted in the zone the event's own
/// times are written in.
///
/// A value ending in the letter for universal time names an instant, and the
/// day an instant falls on depends on where you are standing. A meeting at one
/// in the morning in Kolkata is half past seven the evening before in
/// Greenwich, so a server writing the cancellation as `20260809T193000Z` and
/// this reader taking the first eight digits compared 9 August with 10 August,
/// called nothing off, and announced the cancelled meeting. It works the other
/// way round too: an evening meeting in New York is cancelled by a value dated
/// the following morning.
///
/// The event's own offset is what the instant is turned into a day with,
/// because the days the series is shown on are that event's clock face carried
/// from day to day. Without one there is nothing to convert into, which is a
/// timed event stored as a clock face and a whole-day event stored as a bare
/// date. Both leave the digits standing, which is what they meant before this
/// and what a cancellation written in the event's own zone means anyway.
///
/// A value carrying a numeric offset rather than the letter is not something
/// the calendar standard allows in this property and nothing writes one, so it
/// is read as a clock face. Named so it is not mistaken for handled.
fn the_day_called_off(
    one: &str,
    the_events_offset: Option<chrono::FixedOffset>,
) -> Option<chrono::NaiveDate> {
    let stamp: String = one.chars().filter(char::is_ascii_digit).collect();
    let written = chrono::NaiveDate::parse_from_str(stamp.get(..8)?, "%Y%m%d").ok()?;
    let (Some(offset), true) = (the_events_offset, says_universal_time(one)) else {
        return Some(written);
    };
    let Some(clock) = the_clock_face_in(stamp.get(8..)?) else {
        return Some(written);
    };
    use chrono::TimeZone;
    Some(
        chrono::Utc
            .from_utc_datetime(&written.and_time(clock))
            .with_timezone(&offset)
            .date_naive(),
    )
}

/// Whether a called-off value says it is already in universal time.
///
/// The letter means the same in either case, the same as every other name a
/// calendar document carries. Read as a capital only, a server writing in small
/// letters would have every cancellation counted in the wrong zone.
fn says_universal_time(one: &str) -> bool {
    one.trim_end().ends_with(['Z', 'z'])
}

/// The clock face six digits name, or nothing when there are not six of them.
///
/// A cancellation with no time on it is a bare date, and there is no instant to
/// convert. Anything past the sixth digit is a fraction of a second, which
/// cannot move the day.
fn the_clock_face_in(digits: &str) -> Option<chrono::NaiveTime> {
    let figures = |from: usize, to: usize| digits.get(from..to)?.parse::<u32>().ok();
    chrono::NaiveTime::from_hms_opt(figures(0, 2)?, figures(2, 4)?, figures(4, 6)?)
}

/// The offset a stored moment carries, when it carries one.
///
/// Read through `common::moment` rather than by looking for a sign, so this
/// asks the same question of the column as everything else that reads it.
fn the_offset_on(stored: &str) -> Option<chrono::FixedOffset> {
    use crate::common::moment::Moment;
    match crate::common::moment::read(stored)? {
        Moment::Fixed(moment) => Some(*moment.offset()),
        Moment::ClockFace(_) | Moment::WholeDay(_) => None,
    }
}

/// One called-off day without the property name and parameters, if it has them.
///
/// Both readers that fill this column take them off, so most stored values
/// arrive bare. This does not rely on that. The digits are read out of whatever
/// is left, and a parameter is allowed to contain a digit: `Etc/GMT+5` is an
/// ordinary zone name, and its 5 was read as the first figure of the date. That
/// gave "52026031", which is not a date, so the day was quietly not called off
/// and the cancelled meeting was announced on the day it was cancelled for.
///
/// The name and its parameters end at the first colon, which a date never
/// contains, so a value that arrived bare is handed back untouched.
fn without_the_name_and_parameters(one: &str) -> &str {
    one.split_once(':').map_or(one, |(_, value)| value)
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
    /// The days are the series' own, never the window's: the arithmetic is
    /// always done on the rule's own blocks, so a weekly meeting keeps its
    /// weekday and a monthly one its date whatever window it is asked about.
    fn days_between(
        &self,
        first: chrono::NaiveDate,
        from: chrono::NaiveDate,
        to: chrono::NaiveDate,
    ) -> Vec<chrono::NaiveDate> {
        self.days_and_blocks(first, from, to).0
    }

    /// The same days, and how many blocks were looked at to find them.
    ///
    /// The count is here so a test can watch it. What has to hold about this
    /// loop is a bound on the work it does, and a bound nothing can see is a
    /// bound nothing would notice the loss of. Timing it instead would be a
    /// flaky check measuring the machine rather than the code.
    fn days_and_blocks(
        &self,
        first: chrono::NaiveDate,
        from: chrono::NaiveDate,
        to: chrono::NaiveDate,
    ) -> (Vec<chrono::NaiveDate>, usize) {
        let (start, gone_by) = self.where_to_start_stepping(first, from);
        self.days_stepping_from(start, gone_by, first, from, to)
    }

    /// The same again, from a block named by the caller, with the days the
    /// series had already fallen on before that block.
    ///
    /// Split out so a test can hold the answer from the block the window opens
    /// in against the answer from the series' own first day, which is where
    /// this used to start. Those two have to name the same days for every
    /// series the old way could still reach, and that is the whole claim: the
    /// fix is only allowed to end the truncation, never to shift a day.
    ///
    /// `gone_by` is nought when the walk starts at the series' own first day,
    /// and it only means anything to a series that stops after so many times,
    /// which is counted from that day whatever block the walk begins at.
    fn days_stepping_from(
        &self,
        start: chrono::NaiveDate,
        gone_by: usize,
        first: chrono::NaiveDate,
        from: chrono::NaiveDate,
        to: chrono::NaiveDate,
    ) -> (Vec<chrono::NaiveDate>, usize) {
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
        let mut counted = gone_by;
        let mut blocks = 0usize;
        let mut cursor = start;
        'stepping: for _ in 0..MOST_STEPS {
            // Every day a block falls on is at or after the block's own
            // opening: a weekly block opens on its Monday and reaches the
            // Sunday, a monthly one opens on the first and stays inside the
            // month, a yearly one opens on the first of January and stays
            // inside the year. So once the cursor is past the end of the
            // window, nothing still to come can be inside it. Without this the
            // only thing that ends a rule falling on no day is MOST_STEPS.
            if cursor > to {
                break;
            }
            for day in self.days_at(cursor, first) {
                if day < first {
                    continue;
                }
                if stop_on.is_some_and(|end| day > end) {
                    break 'stepping;
                }
                counted += 1;
                if most_times.is_some_and(|most| counted > most) {
                    break 'stepping;
                }
                if day > to {
                    break 'stepping;
                }
                if day >= from {
                    found.push(day);
                    if found.len() >= MOST_DAYS_ONE_SERIES_SHOWS {
                        break 'stepping;
                    }
                }
            }
            blocks += 1;
            let Some(next) = self.blocks_on(cursor, 1) else {
                break;
            };
            // A step that does not move is a rule that would be worked out for
            // ever. Nothing known produces one; this is what stops the one
            // nobody thought of from hanging the panel.
            if next <= cursor {
                break;
            }
            cursor = next;
        }
        (found, blocks)
    }

    /// Which block the stepping starts at, and how many days the series has
    /// already fallen on before it, so a series of any age reaches the window
    /// in about as many steps as the window is wide.
    ///
    /// Walked from its own first day, a series costs one step per interval
    /// since that day. Forty thousand daily steps is a hundred and nine years,
    /// so a daily series first dated 1917 ran out of steps five months into a
    /// window eighteen months wide, and one dated 1900 never reached the window
    /// at all. The half-shown one is the worse of the two, because nothing
    /// looks wrong: somebody scrolls forward and their standing appointment has
    /// quietly stopped existing.
    ///
    /// Blocks are evenly spaced, so the one the window opens in is worked out
    /// rather than counted to. Every day a block falls on is at or after that
    /// block's own opening and before the next block's, so the last block
    /// opening at or before `from` is the earliest one that can hold a day
    /// inside the window, and starting there leaves the days shown exactly as
    /// they were for every series the walking could already reach.
    ///
    /// A series that stops after so many times has to carry its count across
    /// that jump, because the count is of days from the series' own first one.
    /// Where the days already gone by cannot be worked out, the walk starts at
    /// that first day as it always did, so the count is never begun part way
    /// through and a series that ran out in 1918 never goes on showing today.
    fn where_to_start_stepping(
        &self,
        first: chrono::NaiveDate,
        from: chrono::NaiveDate,
    ) -> (chrono::NaiveDate, usize) {
        let opening = self.opening(first);
        let blocks = self.blocks_between(opening, from);
        let gone_by = match self.stops {
            crate::application::repeating::Until::AfterTimes(_) => {
                self.days_before_block(first, blocks)
            }
            _ => Some(0),
        };
        match (self.blocks_on(opening, blocks), gone_by) {
            (Some(start), Some(gone_by)) => (start, gone_by),
            // Either the count of this frequency cannot be worked out, or the
            // arithmetic for the block itself ran off the end of the calendar.
            // Both mean walking from the series' own first day, which is right
            // and slow rather than fast and wrong.
            _ => (opening, 0),
        }
    }

    /// How many whole intervals of this rule lie between two dates.
    ///
    /// Never negative: a series that has not begun by `from` starts at its own
    /// first block rather than at a block before it.
    fn blocks_between(&self, opening: chrono::NaiveDate, from: chrono::NaiveDate) -> i64 {
        // Worked out at an interval of one and divided by the interval after,
        // which is the same answer as dividing by the two together and keeps
        // each unit in one place.
        let apart = match self.how {
            How::Daily => (from - opening).num_days(),
            How::Weekly => (from - opening).num_days().div_euclid(7),
            How::Monthly => months_apart(opening, from),
            How::Yearly => months_apart(opening, from).div_euclid(12),
        };
        apart.max(0) / i64::from(self.every)
    }

    /// How many days the series falls on before a block, or nothing where that
    /// cannot be worked out without walking them.
    ///
    /// A day rule and a week rule land on the same number of days in every
    /// block, so the total is arithmetic however many blocks there are. A month
    /// rule does not: February has no 31st, and only three months of a year
    /// hold a fifth Tuesday. Nor does a year rule, because three years in four
    /// have no 29th of February. Those two are answered `None` and are walked
    /// from the series' own first day, which reaches three thousand three
    /// hundred years of months and forty thousand years of years before it
    /// meets `MOST_STEPS`, so no date a calendar carries is beyond it.
    ///
    /// The first block is the one that can hold days before the series' own
    /// first day, and those are not days of the series: a weekly rule naming
    /// Monday and Friday, first dated on a Friday, falls on one day in its
    /// opening week and two in every week after. How many days a block holds is
    /// asked of `days_at`, the same as the walking asks, so the two cannot
    /// disagree about it.
    fn days_before_block(&self, first: chrono::NaiveDate, blocks: i64) -> Option<usize> {
        if matches!(self.how, How::Monthly | How::Yearly) {
            return None;
        }
        let blocks = usize::try_from(blocks).ok()?;
        let opening = self.opening(first);
        let each_block = self.days_at(opening, first);
        let in_the_opening = each_block.iter().filter(|day| **day >= first).count();
        let Some(after_the_opening) = blocks.checked_sub(1) else {
            return Some(0);
        };
        // Saturating rather than checked: a count is at most a u32, so a total
        // this cannot hold is a series that ran out long ago whatever the exact
        // figure, and saying "more than the count" is the right answer for it.
        Some(in_the_opening.saturating_add(after_the_opening.saturating_mul(each_block.len())))
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

    /// The block a number of steps on from this one, one step being the
    /// interval the rule names.
    ///
    /// One place knows how far apart a rule's blocks are, and both the stepping
    /// and the working out of where to start it come here, so the two cannot
    /// disagree about where a block lands.
    ///
    /// Days for the rules that count in days and months for the ones that count
    /// in months, because a month is not a fixed number of days and counting a
    /// monthly rule in days would land it on the wrong month.
    ///
    /// The gap is worked out wider than the interval was read in. INTERVAL has
    /// no upper limit in the standard and none in the parser, and at the top of
    /// its own width a week count overflows. A month count that wrapped would
    /// be worse than a crash: it is no longer a whole number of years, so a
    /// yearly rule would step to a month it never named, and every day worked
    /// out from there belongs to a rule nobody wrote.
    fn blocks_on(&self, block: chrono::NaiveDate, steps: i64) -> Option<chrono::NaiveDate> {
        let moved = steps.checked_mul(i64::from(self.every))?;
        match self.how {
            How::Daily => block.checked_add_signed(chrono::TimeDelta::try_days(moved)?),
            How::Weekly => {
                block.checked_add_signed(chrono::TimeDelta::try_days(moved.checked_mul(7)?)?)
            }
            How::Monthly => months_after(block, moved),
            How::Yearly => months_after(block, moved.checked_mul(12)?),
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

/// How many whole months lie between the months two dates fall in.
///
/// Negative when the later date is the first one, which is what says a series
/// starts after the window rather than before it.
fn months_apart(from: chrono::NaiveDate, to: chrono::NaiveDate) -> i64 {
    use chrono::Datelike;
    (i64::from(to.year()) - i64::from(from.year())) * 12 + i64::from(to.month())
        - i64::from(from.month())
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
    fn test_a_rule_that_falls_on_no_day_is_not_stepped_past_the_window() {
        // A rule that lands on nothing gives the inner loop nothing to notice,
        // so the stepping used to run to its own backstop: forty thousand
        // months, three thousand years past a five week window, every time the
        // calendar panel was built.
        //
        // Built by hand rather than parsed, because the parser now refuses a
        // sixth Monday, and what is under test here is the stepping rather than
        // the parse. Any rule that falls on nothing would do.
        let rule = Rule {
            how: How::Monthly,
            every: 1,
            weekdays: Vec::new(),
            nth_weekday: Some((6, chrono::Weekday::Mon)),
            stops: crate::application::repeating::Until::Forever,
        };
        let first = between("2026-03-05", "2026-03-05").0;
        let (from, to) = window();

        let (days, blocks) = rule.days_and_blocks(first, from, to);

        assert!(days.is_empty(), "{days:?}");
        assert!(
            blocks <= 3,
            "stepped {blocks} blocks to cover a window five weeks wide"
        );
    }

    /// The real window, 180 days back and 365 forward, held still at a date
    /// rather than read from the clock so the numbers in these tests stay put.
    ///
    /// Written out as two dates and then checked against the two constants,
    /// because the counts asserted against it are written out as well. Widen
    /// the window and this says so, instead of these tests going on measuring
    /// a window the product no longer shows.
    fn the_real_window_width() -> (chrono::NaiveDate, chrono::NaiveDate) {
        let (from, to) = between("2026-02-07", "2027-08-06");
        assert_eq!(
            (to - from).num_days(),
            HOW_FAR_BACK + HOW_FAR_FORWARD,
            "{from} to {to} was the real window when it was written down"
        );
        (from, to)
    }

    /// Where two answers first disagree, said in a line.
    ///
    /// Printing both lists whole is eighteen months of dates twice over, which
    /// nobody reads. The day they part company is the thing worth seeing.
    fn where_they_differ(one: &[chrono::NaiveDate], other: &[chrono::NaiveDate]) -> Option<String> {
        if one == other {
            return None;
        }
        let at = one
            .iter()
            .zip(other)
            .position(|(here, there)| here != there)
            .unwrap_or_else(|| one.len().min(other.len()));
        Some(format!(
            "{} days against {}, first parting at number {at}, {:?} against {:?}",
            one.len(),
            other.len(),
            one.get(at),
            other.get(at)
        ))
    }

    #[test]
    fn test_the_old_dates_a_real_calendar_carries_still_come_round() {
        // A daily series first dated 1917 is nobody's diary. These are: a
        // birthday or an anniversary imported with a year of birth on it, and
        // an import that lost its start and left the epoch behind. Whatever
        // year the first date carries, the next day the series falls on is
        // still worked out.
        let (from, to) = the_real_window_width();
        for (what, start, rule, expected) in [
            (
                "a birthday kept with a date of birth in the thirties",
                "1935-06-15 00:00",
                "FREQ=YEARLY",
                vec!["2026-06-15 00:00", "2027-06-15 00:00"],
            ),
            (
                "a wedding anniversary from 1952",
                "1952-09-27 00:00",
                "FREQ=YEARLY",
                vec!["2026-09-27 00:00"],
            ),
            (
                "a monthly reminder whose import left the epoch behind",
                "1970-01-01 08:00",
                "FREQ=MONTHLY",
                vec![
                    "2026-03-01 08:00",
                    "2026-04-01 08:00",
                    "2026-05-01 08:00",
                    "2026-06-01 08:00",
                    "2026-07-01 08:00",
                    "2026-08-01 08:00",
                    "2026-09-01 08:00",
                    "2026-10-01 08:00",
                    "2026-11-01 08:00",
                    "2026-12-01 08:00",
                    "2027-01-01 08:00",
                    "2027-02-01 08:00",
                    "2027-03-01 08:00",
                    "2027-04-01 08:00",
                    "2027-05-01 08:00",
                    "2027-06-01 08:00",
                    "2027-07-01 08:00",
                    "2027-08-01 08:00",
                ],
            ),
        ] {
            let event = an_event(start, start, Some(rule));

            let shown = falls_on(&event, from, to);

            assert_eq!(starts(&shown), expected, "{what}");
        }
    }

    #[test]
    fn test_a_daily_series_first_dated_a_century_ago_is_shown_across_the_whole_window() {
        // The window holds 546 days and a daily series falls on all of them,
        // whatever year it was first dated. Carried to the window one step at a
        // time, a series dated 1917 ran out of steps partway through: it showed
        // for the first five months and then stopped, and a series dated 1900
        // never reached the window at all.
        //
        // Half a window shown is worse than none, because nothing looks wrong.
        // Somebody scrolls forward and their standing appointment has quietly
        // stopped existing.
        let (from, to) = the_real_window_width();
        for year in [1900, 1917, 1930, 1970, 1990, 2015] {
            let event = an_event(
                &format!("{year}-01-01 09:00"),
                &format!("{year}-01-01 09:15"),
                Some("FREQ=DAILY"),
            );

            let shown = falls_on(&event, from, to);

            assert_eq!(shown.days.len(), 546, "a daily series first dated {year}");
            assert_eq!(
                shown.days.first().map(|day| day.start.as_str()),
                Some("2026-02-07 09:00"),
                "a daily series first dated {year}"
            );
            assert_eq!(
                shown.days.last().map(|day| day.start.as_str()),
                Some("2027-08-06 09:00"),
                "a daily series first dated {year}"
            );
        }
    }

    #[test]
    fn test_an_old_series_is_not_carried_to_the_window_one_step_at_a_time() {
        // The days are only half of it. Even where the stepping did reach the
        // window it walked every day in between, so opening the calendar panel
        // stepped a series dated 1970 twenty thousand times, per event, per
        // build. The work has to be bounded by the width of the window rather
        // than by the age of the series.
        let rule = Rule::read("FREQ=DAILY").expect("a rule this can read");
        let (from, to) = the_real_window_width();

        for year in [1900, 1917, 1970, 2015] {
            let first = between(&format!("{year}-01-01"), "2026-01-01").0;

            let (days, blocks) = rule.days_and_blocks(first, from, to);

            assert_eq!(days.len(), 546, "a daily series first dated {year}");
            assert!(
                blocks <= 600,
                "stepped {blocks} blocks over a window 546 days wide, for a series first dated {year}"
            );
        }
    }

    #[test]
    fn test_a_counted_series_first_dated_a_century_ago_is_shown_across_the_whole_window() {
        // The same fault as the test above, on a series that stops after so
        // many times rather than one that runs for ever, and it was left open
        // when that one was closed. A count is counted from the series' own
        // first day, so a counted series was walked from there whatever window
        // it was asked about: a daily one first dated 1900 showed on none of
        // the 546 days the window holds, and one dated 1917 on 151 of them.
        //
        // Fifty thousand days is a hundred and thirty-six years, so every one
        // of these is still running through the whole of the window.
        let (from, to) = the_real_window_width();
        for year in [1900, 1917, 1930, 1970, 1990, 2015] {
            let event = an_event(
                &format!("{year}-01-01 09:00"),
                &format!("{year}-01-01 09:15"),
                Some("FREQ=DAILY;COUNT=50000"),
            );

            let shown = falls_on(&event, from, to);

            assert_eq!(shown.days.len(), 546, "a counted series first dated {year}");
            assert_eq!(
                shown.days.first().map(|day| day.start.as_str()),
                Some("2026-02-07 09:00"),
                "a counted series first dated {year}"
            );
            assert_eq!(
                shown.days.last().map(|day| day.start.as_str()),
                Some("2027-08-06 09:00"),
                "a counted series first dated {year}"
            );
        }
    }

    #[test]
    fn test_a_counted_series_from_long_ago_still_stops_on_the_day_its_count_runs_out() {
        // Starting inside the window is worth nothing if the count then starts
        // over from there: a series that ran out in 1918 would go on showing
        // today. The days already gone by have to be worked out as exactly as
        // walking them would have counted them.
        //
        // The 15th of June 2026 is 46,186 days after the 1st of January 1900,
        // so it is occurrence number 46,187 and the last day of a series
        // counting that far. The window opens on the 7th of February 2026,
        // which leaves 129 days of it to show.
        let (from, to) = the_real_window_width();
        let event = an_event(
            "1900-01-01 09:00",
            "1900-01-01 09:15",
            Some("FREQ=DAILY;COUNT=46187"),
        );

        let shown = falls_on(&event, from, to);

        assert_eq!(shown.days.len(), 129, "{:?}", shown.days.last());
        assert_eq!(
            shown.days.last().map(|day| day.start.as_str()),
            Some("2026-06-15 09:00")
        );
    }

    #[test]
    fn test_a_counted_series_that_ran_out_before_the_window_shows_no_day_of_it() {
        // The other side of the same arithmetic, and the reason the count
        // cannot simply be started again at the window.
        let (from, to) = the_real_window_width();
        let event = an_event(
            "1900-01-01 09:00",
            "1900-01-01 09:15",
            Some("FREQ=DAILY;COUNT=46000"),
        );

        let shown = falls_on(&event, from, to);

        assert!(shown.days.is_empty(), "{:?}", starts(&shown));
    }

    #[test]
    fn test_a_counted_series_is_not_carried_to_the_window_one_step_at_a_time() {
        // The days are only half of it, the same as for a series with no
        // ending. A counted daily series first dated 1900 was stepped forty
        // thousand times, per event, per build of the calendar panel, and
        // showed nothing for the trouble. A counted weekly one first dated 1900
        // did reach the window, so its days were right, and it still walked six
        // and a half thousand blocks to get there.
        let (from, to) = the_real_window_width();

        for (written, days) in [
            ("FREQ=DAILY;COUNT=50000", 546),
            ("FREQ=DAILY;INTERVAL=2;COUNT=50000", 273),
            ("FREQ=WEEKLY;COUNT=30000", 78),
            ("FREQ=WEEKLY;BYDAY=MO,WE,FR;COUNT=30000", 234),
        ] {
            let rule = Rule::read(written).expect("a rule this can read");
            let first = between("1900-01-03", "1900-01-03").0;

            let (found, blocks) = rule.days_and_blocks(first, from, to);

            assert_eq!(found.len(), days, "{written}");
            assert!(
                blocks <= 600,
                "stepped {blocks} blocks over a window 546 days wide, for {written}"
            );
        }
    }

    #[test]
    fn test_a_counted_month_series_counts_the_days_it_falls_on_not_the_months() {
        // Why a month rule is still walked, said as the measurement that
        // decided it rather than as an assertion in a comment. Only seven
        // months of twelve have a 31st, so counting a day per month would run
        // such a series out years early and take the rest of it off the
        // calendar.
        //
        // The 31st of January 2005 is the 147th 31st of the month counting from
        // itself through the end of 2025, so a series stopping after 150 has its
        // last day on the 31st of May 2026, and two of them fall inside the
        // window. Counted by months it would be 253 months in and long finished.
        let (from, to) = the_real_window_width();
        let event = an_event(
            "2005-01-31 09:00",
            "2005-01-31 10:00",
            Some("FREQ=MONTHLY;COUNT=150"),
        );

        assert_eq!(
            starts(&falls_on(&event, from, to)),
            ["2026-03-31 09:00", "2026-05-31 09:00"]
        );
    }

    #[test]
    fn test_a_counted_year_series_counts_the_days_it_falls_on_not_the_years() {
        // The same again for a year rule, which falls on the 29th of February
        // one year in four.
        //
        // A window of its own, further ahead than the one the product shows,
        // because the product's window holds no 29th of February this year. The
        // 29th of February 2020 is the first day of this series, 2024 the
        // second and 2028 the third, so one stopping after three ends there.
        // Counted by years it would be eight years in and finished.
        let event = an_event(
            "2020-02-29 09:00",
            "2020-02-29 10:00",
            Some("FREQ=YEARLY;COUNT=3"),
        );
        let (from, to) = between("2028-01-01", "2028-12-31");

        assert_eq!(starts(&falls_on(&event, from, to)), ["2028-02-29 09:00"]);
    }

    #[test]
    fn test_a_counted_monthly_or_yearly_series_reaches_the_window_from_any_real_first_date() {
        // How many days a month rule or a year rule has fallen on cannot be
        // worked out without walking them, so those two are still walked from
        // the series' own first day when they carry a count. This is what that
        // leaves, said as a measurement rather than as a promise: the backstop
        // is forty thousand steps, which is three thousand three hundred years
        // of months and forty thousand years of years, so a first date of the
        // year 1 still arrives with room to spare.
        let (from, to) = the_real_window_width();

        for (written, days, last) in [
            ("FREQ=MONTHLY;COUNT=40000", 18, "2027-08-01 08:00"),
            ("FREQ=YEARLY;COUNT=40000", 1, "2027-01-01 08:00"),
        ] {
            let event = an_event("0001-01-01 08:00", "0001-01-01 08:00", Some(written));

            let shown = falls_on(&event, from, to);

            assert_eq!(shown.days.len(), days, "{written}: {:?}", starts(&shown));
            assert_eq!(
                shown.days.last().map(|day| day.start.as_str()),
                Some(last),
                "{written}"
            );
        }
    }

    #[test]
    fn test_a_series_that_has_not_begun_yet_is_not_stepped_at_all() {
        // The other end of the same arithmetic. Working out where the window is
        // has to answer "at the series' own first block" for a series that
        // starts after the window, not step backwards to meet it: the days
        // would come out the same, because a day before the series began is
        // passed over anyway, but the panel would walk the whole window to
        // find nothing.
        let (from, to) = the_real_window_width();

        for written in [
            "FREQ=DAILY",
            "FREQ=WEEKLY;BYDAY=TU",
            "FREQ=MONTHLY",
            "FREQ=YEARLY",
        ] {
            let rule = Rule::read(written).expect("a rule this can read");
            let first = between("2031-05-04", "2031-05-04").0;

            let (days, blocks) = rule.days_and_blocks(first, from, to);

            assert!(days.is_empty(), "{written}: {days:?}");
            assert_eq!(blocks, 0, "stepped {blocks} blocks for {written}");
        }
    }

    #[test]
    fn test_starting_at_the_window_names_the_same_days_as_walking_from_the_first_day() {
        // The test that matters most here, and the one the fix had to earn.
        // Ending the truncation is worth nothing if the entry point shifts a
        // series that already worked, and a shifted series is the worse fault
        // of the two: every occurrence lands on a day nobody wrote, and the
        // calendar looks perfectly healthy while it does it.
        //
        // So the two are held against each other, over every shape where the
        // step is not a fixed number of days: weekday lists, weekday ordinals
        // counted from both ends, monthly dates no month has, intervals, and
        // the two endings. Each row also proves the walk it is held against
        // was a fair one, by asserting it never reached the backstop.
        let rules = [
            "FREQ=DAILY",
            "FREQ=DAILY;INTERVAL=2",
            "FREQ=DAILY;INTERVAL=7",
            "FREQ=WEEKLY",
            "FREQ=WEEKLY;INTERVAL=2",
            "FREQ=WEEKLY;INTERVAL=3",
            "FREQ=WEEKLY;BYDAY=MO,WE,FR",
            "FREQ=WEEKLY;BYDAY=SA,SU",
            "FREQ=WEEKLY;BYDAY=TU;INTERVAL=2",
            "FREQ=MONTHLY",
            "FREQ=MONTHLY;INTERVAL=3",
            "FREQ=MONTHLY;BYDAY=1MO",
            "FREQ=MONTHLY;BYDAY=3WE",
            "FREQ=MONTHLY;BYDAY=5MO",
            "FREQ=MONTHLY;BYDAY=-1FR",
            "FREQ=MONTHLY;BYDAY=-2TH",
            "FREQ=YEARLY",
            "FREQ=YEARLY;INTERVAL=4",
            "FREQ=DAILY;COUNT=10",
            "FREQ=WEEKLY;COUNT=500",
            "FREQ=DAILY;UNTIL=20261231T235959Z",
            "FREQ=MONTHLY;BYDAY=2TU;UNTIL=20270101T000000Z",
            // Counts big enough that the walk crosses decades to reach the
            // window, which is where the two used to part company: the walk was
            // the only way of knowing how many days had gone by, and it is now
            // arithmetic. The figures are chosen so that the day the count runs
            // out lands inside one of the windows for several of the first
            // dates below, because a count that never runs out inside a window
            // proves only that the jump did not lose the series.
            "FREQ=DAILY;COUNT=50000",
            "FREQ=DAILY;COUNT=15500",
            "FREQ=DAILY;COUNT=7700",
            "FREQ=DAILY;COUNT=2200",
            "FREQ=DAILY;INTERVAL=2;COUNT=3850",
            "FREQ=WEEKLY;COUNT=2200",
            "FREQ=WEEKLY;INTERVAL=2;COUNT=1100",
            "FREQ=WEEKLY;BYDAY=MO,WE,FR;COUNT=6600",
            "FREQ=WEEKLY;BYDAY=SA,SU;COUNT=4400",
            "FREQ=WEEKLY;BYDAY=TU;INTERVAL=2;COUNT=1100",
            // A month rule and a year rule are still walked from the first day
            // when they carry a count, so these two hold that path against
            // itself and would notice it being changed to the arithmetic one.
            "FREQ=MONTHLY;COUNT=520",
            "FREQ=YEARLY;COUNT=43",
        ];
        let windows = [
            the_real_window_width(),
            // Opening part way through a week and part way through a month, so
            // the block it opens in holds days on both sides of the opening.
            between("2026-02-18", "2026-05-20"),
            // Narrower than one monthly block, and narrower than one week.
            between("2026-03-10", "2026-03-14"),
        ];
        // Two leap years among them, and days no month has, so a rule that
        // skips a block meets the skipping on both sides of the comparison.
        let first_days = [
            (1984, 1, 1),
            (1984, 2, 29),
            (1984, 8, 30),
            (2005, 1, 31),
            (2005, 5, 29),
            (2005, 11, 4),
            (2020, 2, 29),
            (2020, 3, 15),
            (2020, 12, 31),
            (2026, 1, 1),
            (2026, 3, 31),
            (2026, 7, 4),
            (2031, 2, 28),
            (2031, 9, 30),
        ];

        for written in rules {
            let rule = Rule::read(written).expect("a rule this can read");
            for (year, month, day) in first_days {
                let first =
                    chrono::NaiveDate::from_ymd_opt(year, month, day).expect("a day of the year");
                for (from, to) in windows {
                    let (walked, blocks) =
                        rule.days_stepping_from(rule.opening(first), 0, first, from, to);

                    assert!(
                        blocks < MOST_STEPS,
                        "{written} first dated {first} reached the backstop, \
                         so there is nothing here to hold the answer against"
                    );
                    let started_at_the_window = rule.days_between(first, from, to);
                    if let Some(difference) = where_they_differ(&started_at_the_window, &walked) {
                        panic!("{written}, first dated {first}, over {from} to {to}: {difference}");
                    }
                }
            }
        }
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
    fn test_a_day_called_off_in_universal_time_is_still_called_off_here() {
        // What many servers write: the cancellation as the instant it names,
        // in universal time, whatever zone the event's own times are in. This
        // meeting is at one in the morning in Kolkata, so the same instant is
        // half past seven the evening before in Greenwich, and comparing the
        // first eight digits of the two strings compares 9 August with 10
        // August and calls nothing off. The cancelled meeting was announced on
        // the day it had been cancelled for.
        let mut event = an_event(
            "2026-08-10T01:00:00+05:30",
            "2026-08-10T01:30:00+05:30",
            Some("FREQ=WEEKLY"),
        );
        event.exception_dates = Some("20260809T193000Z".to_string());
        let (from, to) = between("2026-08-01", "2026-08-20");

        assert_eq!(
            starts(&falls_on(&event, from, to)),
            ["2026-08-17T01:00:00+05:30"]
        );
    }

    #[test]
    fn test_the_same_cancellation_written_in_the_events_own_zone_is_still_honoured() {
        // The other spelling of the same cancellation, and the one that has
        // always worked. Both have to, because which one arrives is the
        // server's choice and neither is more correct than the other.
        let mut event = an_event(
            "2026-08-10T01:00:00+05:30",
            "2026-08-10T01:30:00+05:30",
            Some("FREQ=WEEKLY"),
        );
        event.exception_dates = Some("20260810T010000".to_string());
        let (from, to) = between("2026-08-01", "2026-08-20");

        assert_eq!(
            starts(&falls_on(&event, from, to)),
            ["2026-08-17T01:00:00+05:30"]
        );
    }

    #[test]
    fn test_a_day_called_off_in_universal_time_does_not_call_off_the_day_before() {
        // The other direction, and what a change that simply shifted the date
        // would break. An evening meeting in New York is the same evening in
        // New York whichever zone the cancellation is written in, and turning
        // the instant into a day in the wrong zone moves it to the next
        // morning and leaves the cancelled meeting on the calendar.
        let mut event = an_event(
            "2026-08-10T20:00:00-04:00",
            "2026-08-10T21:00:00-04:00",
            Some("FREQ=WEEKLY"),
        );
        event.exception_dates = Some("20260811T000000Z".to_string());
        let (from, to) = between("2026-08-01", "2026-08-20");

        assert_eq!(
            starts(&falls_on(&event, from, to)),
            ["2026-08-17T20:00:00-04:00"]
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
    fn test_an_excluded_day_still_carrying_its_property_name_is_still_excluded() {
        // Google sends whole property lines, and what is stored is whichever
        // shape the source used, so this reader has to cope with the name and
        // its parameters still being on the front. Reading the digits out of
        // the whole line works only while no parameter has a digit in it.
        let mut event = an_event("2026-03-05 09:00", "2026-03-05 09:15", Some("FREQ=WEEKLY"));
        event.exception_dates = Some("EXDATE;TZID=Europe/London:20260312T090000".to_string());
        let (from, to) = between("2026-03-01", "2026-03-20");

        assert_eq!(
            starts(&falls_on(&event, from, to)),
            ["2026-03-05 09:00", "2026-03-19 09:00"]
        );
    }

    #[test]
    fn test_a_zone_with_a_digit_in_its_name_does_not_lose_the_called_off_day() {
        // `Etc/GMT+5` is an ordinary zone name. Its digit was read as part of
        // the date, "52026031" is not a date, and the exclusion was dropped
        // without a word: the cancelled meeting came back and was announced on
        // the day it had been called off for.
        let mut event = an_event("2026-03-05 09:00", "2026-03-05 09:15", Some("FREQ=WEEKLY"));
        event.exception_dates = Some("EXDATE;TZID=Etc/GMT+5:20260312T090000".to_string());
        let (from, to) = between("2026-03-01", "2026-03-20");

        assert_eq!(
            starts(&falls_on(&event, from, to)),
            ["2026-03-05 09:00", "2026-03-19 09:00"]
        );
    }

    #[test]
    fn test_every_called_off_day_on_one_line_is_read_when_the_name_is_still_there() {
        // The name sits on the front of the first value only. The rest follow
        // the commas bare, so a reader that strips the front once and a reader
        // that strips it per value have to agree on both.
        let mut event = an_event("2026-03-05 09:00", "2026-03-05 09:15", Some("FREQ=WEEKLY"));
        event.exception_dates =
            Some("EXDATE;TZID=Etc/GMT+5:20260312T090000,20260319T090000".to_string());
        let (from, to) = between("2026-03-01", "2026-03-20");

        assert_eq!(starts(&falls_on(&event, from, to)), ["2026-03-05 09:00"]);
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

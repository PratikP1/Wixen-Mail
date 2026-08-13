//! How often something comes round again, and when it stops.
//!
//! The item forms offered five answers: never, daily, weekly, monthly, yearly.
//! Nothing said every weekday, nothing said every other week, and nothing at
//! all could say when a series ends, so anything set to repeat repeated for
//! ever. A course that runs for six weeks had to be entered six times or left
//! wrong.
//!
//! What comes out of here is an RFC 5545 recurrence rule, which is what both
//! Google and Microsoft take and what an `.ics` file carries, so a series made
//! here means the same thing everywhere it is read.

/// How often something repeats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Repeat {
    #[default]
    Never,
    Daily,
    /// Monday to Friday. The most asked-for pattern that "daily" gets wrong,
    /// and the reason somebody ends up with a reminder going off on Sunday.
    Weekdays,
    Weekly,
    Fortnightly,
    Monthly,
    /// The same weekday of the month rather than the same date: the second
    /// Tuesday rather than the ninth. Which one is meant depends on what the
    /// thing is, and a meeting is almost always the weekday.
    MonthlyByWeekday,
    Quarterly,
    Yearly,
}

impl Repeat {
    /// Every choice, in the order they are offered.
    pub const ALL: [Repeat; 9] = [
        Repeat::Never,
        Repeat::Daily,
        Repeat::Weekdays,
        Repeat::Weekly,
        Repeat::Fortnightly,
        Repeat::Monthly,
        Repeat::MonthlyByWeekday,
        Repeat::Quarterly,
        Repeat::Yearly,
    ];

    /// What the choice is called.
    ///
    /// Said in full rather than abbreviated, because this is read aloud from a
    /// list where every entry sounds like the one before it otherwise.
    pub fn label(self) -> &'static str {
        match self {
            Repeat::Never => "Does not repeat",
            Repeat::Daily => "Every day",
            Repeat::Weekdays => "Every weekday, Monday to Friday",
            Repeat::Weekly => "Every week",
            Repeat::Fortnightly => "Every two weeks",
            Repeat::Monthly => "Every month, on this date",
            Repeat::MonthlyByWeekday => "Every month, on this weekday",
            Repeat::Quarterly => "Every three months",
            Repeat::Yearly => "Every year",
        }
    }

    /// The rule itself, without the ending.
    ///
    /// `None` for something that does not repeat, which is no rule at all
    /// rather than a rule saying so.
    fn frequency(self) -> Option<&'static str> {
        match self {
            Repeat::Never => None,
            Repeat::Daily => Some("FREQ=DAILY"),
            Repeat::Weekdays => Some("FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR"),
            Repeat::Weekly => Some("FREQ=WEEKLY"),
            Repeat::Fortnightly => Some("FREQ=WEEKLY;INTERVAL=2"),
            Repeat::Monthly => Some("FREQ=MONTHLY"),
            Repeat::MonthlyByWeekday => Some("FREQ=MONTHLY;BYDAY="),
            Repeat::Quarterly => Some("FREQ=MONTHLY;INTERVAL=3"),
            Repeat::Yearly => Some("FREQ=YEARLY"),
        }
    }

    /// Match what a form control is showing.
    ///
    /// The words it offers, and also the shorter ones the forms used before
    /// this: "Weekly" as well as "Every week". Nothing stores those any more,
    /// but a lookup that answers "does not repeat" to a word it does not know
    /// would turn a series into a single item without saying so, and that is
    /// the failure worth spending five lines on.
    pub fn from_label(shown: &str) -> Self {
        let shown = shown.trim();
        if let Some(found) = Repeat::ALL.iter().find(|choice| choice.label() == shown) {
            return *found;
        }
        match shown {
            "Daily" => Repeat::Daily,
            "Weekly" => Repeat::Weekly,
            "Monthly" => Repeat::Monthly,
            "Yearly" => Repeat::Yearly,
            _ => Repeat::Never,
        }
    }

    /// Read a stored rule back into a choice.
    ///
    /// A rule this does not recognise is still a repeat, and answering "does
    /// not repeat" would quietly turn a series into a single item the next
    /// time somebody saved it. It comes back as the nearest thing that shares
    /// its frequency, so what is offered is at least true about how often.
    pub fn from_rule(rule: &str) -> Self {
        let rule = rule.trim().to_ascii_uppercase();
        if rule.is_empty() {
            return Repeat::Never;
        }
        let has = |part: &str| rule.contains(part);
        if has("FREQ=DAILY") {
            return Repeat::Daily;
        }
        if has("FREQ=WEEKLY") {
            if has("BYDAY=MO,TU,WE,TH,FR") {
                return Repeat::Weekdays;
            }
            if has("INTERVAL=2") {
                return Repeat::Fortnightly;
            }
            return Repeat::Weekly;
        }
        if has("FREQ=MONTHLY") {
            if has("INTERVAL=3") {
                return Repeat::Quarterly;
            }
            if has("BYDAY=") {
                return Repeat::MonthlyByWeekday;
            }
            return Repeat::Monthly;
        }
        if has("FREQ=YEARLY") {
            return Repeat::Yearly;
        }
        // Something with a frequency nothing here offers. Weekly is the
        // commonest and the least surprising thing to show.
        Repeat::Weekly
    }
}

/// When a repeating series stops.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Until {
    /// It does not. What every series made before this could only be.
    #[default]
    Forever,
    /// After this date, written as it is stored: `YYYY-MM-DD`.
    OnDate(String),
    /// After this many occurrences.
    AfterTimes(u32),
}

impl Until {
    /// The part of the rule that ends it, if anything does.
    ///
    /// The one writer of it. Every `UNTIL` and every `COUNT` this program puts
    /// in a rule comes from here, so a series read back from a provider and one
    /// typed into the form cannot disagree about whether the last day counts.
    pub(crate) fn ending(&self) -> Option<String> {
        match self {
            Until::Forever => None,
            // UNTIL is a date-time in UTC in the form RFC 5545 wants, and the
            // end of the day rather than the start, so a series ending "on the
            // thirtieth" includes the thirtieth.
            Until::OnDate(date) => {
                let digits: String = date.chars().filter(char::is_ascii_digit).collect();
                (digits.len() == 8).then(|| format!("UNTIL={digits}T235959Z"))
            }
            Until::AfterTimes(0) => None,
            Until::AfterTimes(times) => Some(format!("COUNT={times}")),
        }
    }

    /// Read the ending out of a stored rule.
    pub fn from_rule(rule: &str) -> Self {
        let rule = rule.trim().to_ascii_uppercase();
        for part in rule.split(';') {
            if let Some(value) = part.strip_prefix("UNTIL=") {
                let digits: String = value.chars().filter(char::is_ascii_digit).collect();
                if digits.len() >= 8 {
                    return Until::OnDate(format!(
                        "{}-{}-{}",
                        &digits[0..4],
                        &digits[4..6],
                        &digits[6..8]
                    ));
                }
            }
            if let Some(value) = part.strip_prefix("COUNT=")
                && let Ok(times) = value.parse::<u32>()
                && times > 0
            {
                return Until::AfterTimes(times);
            }
        }
        Until::Forever
    }
}

/// The whole rule, as it is stored and sent.
///
/// `None` for something that does not repeat. An ending on its own is not a
/// rule: "until the thirtieth" says nothing about how often, so it is dropped
/// rather than written out as a rule that means nothing.
pub fn rule(repeat: Repeat, until: &Until, weekday: Option<&str>) -> Option<String> {
    let frequency = repeat.frequency()?;
    let mut parts = match (repeat, weekday) {
        // The weekday has to come from the date the series starts, because
        // "the second Tuesday" is a fact about that date rather than a choice.
        (Repeat::MonthlyByWeekday, Some(day)) => format!("{frequency}{day}"),
        // Without one it cannot say which weekday, and a rule ending in BYDAY=
        // is not a rule. The same date each month is the honest fallback.
        (Repeat::MonthlyByWeekday, None) => "FREQ=MONTHLY".to_string(),
        _ => frequency.to_string(),
    };
    if let Some(ending) = until.ending() {
        parts.push(';');
        parts.push_str(&ending);
    }
    Some(parts)
}

/// Which weekday of the month a date falls on, in the form a rule wants.
///
/// "2TU" for the second Tuesday. The last week of the month is written as -1
/// rather than 5, because a month with four Tuesdays has no fifth one and a
/// series set on the last Tuesday should not skip those months.
pub fn weekday_of_month(date: &str) -> Option<String> {
    use chrono::Datelike;
    let parsed = chrono::NaiveDate::parse_from_str(date.trim().get(..10)?, "%Y-%m-%d").ok()?;
    let day = match parsed.weekday() {
        chrono::Weekday::Mon => "MO",
        chrono::Weekday::Tue => "TU",
        chrono::Weekday::Wed => "WE",
        chrono::Weekday::Thu => "TH",
        chrono::Weekday::Fri => "FR",
        chrono::Weekday::Sat => "SA",
        chrono::Weekday::Sun => "SU",
    };
    let week = (parsed.day() - 1) / 7 + 1;
    let last = parsed.day() + 7 > days_in_month(parsed.year(), parsed.month());
    Some(if last {
        format!("-1{day}")
    } else {
        format!("{week}{day}")
    })
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .and_then(|first| first.pred_opt())
        .map(|last| chrono::Datelike::day(&last))
        .unwrap_or(28)
}

// ── The same series, said the way Outlook says one ───────────────────────
//
// Outlook has no rules. It has a shape and a stretch of time, and the two
// spellings have to be held against each other rather than each against a
// hand-written expectation, or one of them changes and the other does not.
// That is the failure this project has had more often than any other.

use crate::service::microsoft_graph::{
    MsDateTimeTimeZone, MsPatternedRecurrence, MsRecurrencePattern, MsRecurrenceRange,
};

/// The day Outlook is told a meeting begins.
///
/// Read off the start in the body rather than off the stored column, and both
/// directions of this conversion ask it. The stored column is a clock face in
/// whatever zone the row happens to carry, and the start in the body has
/// already been worked out from that into the hour Outlook is given, so the two
/// are not the same day for a meeting near midnight in a place hours from
/// Greenwich. Asked once here, the rule and the meeting cannot say different
/// days.
///
/// The shapes and the day itself both come from the one reader of a stored
/// moment this program has. A fourth hand-cut date off the front of a string is
/// how this kept coming back.
pub fn the_day_a_graph_start_names(start: &MsDateTimeTimeZone) -> Option<chrono::NaiveDate> {
    crate::common::moment::read(&start.date_time).map(crate::common::moment::Moment::the_day)
}

/// The seven weekdays, spelled both ways.
///
/// One table read in both directions, so a day cannot be written out under one
/// name and read back as another.
const THE_WEEKDAYS: [(chrono::Weekday, &str, &str); 7] = [
    (chrono::Weekday::Mon, "monday", "MO"),
    (chrono::Weekday::Tue, "tuesday", "TU"),
    (chrono::Weekday::Wed, "wednesday", "WE"),
    (chrono::Weekday::Thu, "thursday", "TH"),
    (chrono::Weekday::Fri, "friday", "FR"),
    (chrono::Weekday::Sat, "saturday", "SA"),
    (chrono::Weekday::Sun, "sunday", "SU"),
];

/// Which of a weekday's occurrences in a month, spelled both ways.
///
/// A rule can name the fifth of a weekday and count from either end of the
/// month. Outlook offers the first four and the last, and nothing else, so
/// anything outside this table is refused rather than rounded to a day the
/// series is not on.
const THE_WEEKS_OF_A_MONTH: [(i32, &str); 5] = [
    (1, "first"),
    (2, "second"),
    (3, "third"),
    (4, "fourth"),
    (-1, "last"),
];

/// Which day a week is counted from, which a rule leaves unsaid and means
/// Monday by.
const A_WEEK_STARTS_ON: &str = "monday";

/// The shape and stretch Outlook needs to make this series, or nothing when
/// Outlook has no way to say it.
///
/// The rule is read by the one reader that refuses what it cannot work out day
/// by day, never by the lenient one the item form uses. That reader answers
/// "every week" to anything it does not recognise, which is right for a form
/// and would file a Tuesday and Thursday meeting at Outlook as a Tuesday one,
/// losing every Thursday without a word.
///
/// The day the series starts fills in what a rule leaves to the start date: a
/// weekly rule naming no weekday, and the date or month a monthly or yearly one
/// keeps.
pub fn as_outlook_says_it(
    rule: &str,
    starts_on: chrono::NaiveDate,
) -> Option<MsPatternedRecurrence> {
    use crate::application::occurrences::{How, Rule};
    use chrono::Datelike;

    let read = Rule::read(crate::service::caldav::without_the_property_name(rule))?;
    // Refuses rather than sending a short list: a weekly meeting that lost one
    // of its days would go on quietly, on fewer days than somebody chose.
    let named_days = |days: &[chrono::Weekday]| -> Option<Vec<String>> {
        days.iter().map(|day| outlook_calls_the_day(*day)).collect()
    };
    let pattern = match read.how {
        How::Daily => MsRecurrencePattern {
            pattern_type: "daily".to_string(),
            interval: read.every,
            ..Default::default()
        },
        How::Weekly => MsRecurrencePattern {
            pattern_type: "weekly".to_string(),
            interval: read.every,
            // Outlook has no weekly shape without a day on it, so a rule that
            // leaves the day to the date it starts has that day written out.
            days_of_week: if read.weekdays.is_empty() {
                named_days(&[starts_on.weekday()])?
            } else {
                named_days(&read.weekdays)?
            },
            first_day_of_week: Some(A_WEEK_STARTS_ON.to_string()),
            ..Default::default()
        },
        How::Monthly => match read.nth_weekday {
            Some((nth, weekday)) => MsRecurrencePattern {
                pattern_type: "relativeMonthly".to_string(),
                interval: read.every,
                days_of_week: named_days(&[weekday])?,
                index: Some(outlook_calls_the_week(nth)?.to_string()),
                ..Default::default()
            },
            None => MsRecurrencePattern {
                pattern_type: "absoluteMonthly".to_string(),
                interval: read.every,
                day_of_month: Some(starts_on.day()),
                ..Default::default()
            },
        },
        How::Yearly => MsRecurrencePattern {
            pattern_type: "absoluteYearly".to_string(),
            interval: read.every,
            day_of_month: Some(starts_on.day()),
            month: Some(starts_on.month()),
            ..Default::default()
        },
    };
    let range = match &read.stops {
        Until::Forever => MsRecurrenceRange {
            range_type: "noEnd".to_string(),
            start_date: starts_on.to_string(),
            ..Default::default()
        },
        Until::OnDate(last_day) => MsRecurrenceRange {
            range_type: "endDate".to_string(),
            start_date: starts_on.to_string(),
            end_date: Some(last_day.clone()),
            ..Default::default()
        },
        Until::AfterTimes(times) => MsRecurrenceRange {
            range_type: "numbered".to_string(),
            start_date: starts_on.to_string(),
            number_of_occurrences: Some(*times),
            ..Default::default()
        },
    };
    Some(MsPatternedRecurrence { pattern, range })
}

/// What Outlook calls a weekday, or nothing for one it has no name for.
fn outlook_calls_the_day(day: chrono::Weekday) -> Option<String> {
    THE_WEEKDAYS
        .iter()
        .find(|(weekday, _, _)| *weekday == day)
        .map(|(_, outlook, _)| (*outlook).to_string())
}

/// What a rule calls a weekday Outlook has named, or nothing for a name it does
/// not use.
fn the_day_outlook_named(said: &str) -> Option<&'static str> {
    let said = said.trim();
    THE_WEEKDAYS
        .iter()
        .find(|(_, outlook, _)| outlook.eq_ignore_ascii_case(said))
        .map(|(_, _, in_a_rule)| *in_a_rule)
}

/// What Outlook calls the nth weekday of a month, or nothing for an ordinal it
/// cannot say.
fn outlook_calls_the_week(nth: i32) -> Option<&'static str> {
    THE_WEEKS_OF_A_MONTH
        .iter()
        .find(|(ordinal, _)| *ordinal == nth)
        .map(|(_, outlook)| *outlook)
}

/// Which occurrence of a weekday in the month Outlook has named, or nothing for
/// a word it does not use.
fn the_week_outlook_named(said: &str) -> Option<i32> {
    let said = said.trim();
    THE_WEEKS_OF_A_MONTH
        .iter()
        .find(|(_, outlook)| outlook.eq_ignore_ascii_case(said))
        .map(|(ordinal, _)| *ordinal)
}

/// The rule an Outlook shape means, or nothing when this program cannot say it.
///
/// Refuses rather than approximates, for the same reason the reading of a rule
/// does: a series drawn on days it does not fall on is worse than one shown
/// once and saying so.
///
/// The day the series starts is needed because a rule leaves the date to the
/// start and Outlook writes it out. Where the two disagree the shape means days
/// the rule would not, so it is refused.
pub fn what_outlook_said(
    said: &MsPatternedRecurrence,
    starts_on: chrono::NaiveDate,
) -> Option<String> {
    use chrono::Datelike;

    let shape = &said.pattern;
    // Nought is not a shape, it is a series that never advances. The reading
    // of a rule refuses one for exactly that reason, so this refuses it too
    // rather than quietly deciding it meant every one. Outlook always says how
    // often, so a nought is an answer that did not.
    let every = (shape.interval > 0).then_some(shape.interval)?;
    let the_only_day = || match shape.days_of_week.as_slice() {
        [only] => the_day_outlook_named(only),
        _ => None,
    };
    let how_often = match shape.pattern_type.to_ascii_lowercase().as_str() {
        "daily" => "FREQ=DAILY".to_string(),
        "weekly" => {
            // Which day a week is counted from decides which weeks a series
            // skipping weeks lands in, and a rule can only mean Monday. A
            // series that comes round every week lands in all of them, so the
            // question does not arise there.
            let counted_from = shape
                .first_day_of_week
                .as_deref()
                .unwrap_or(A_WEEK_STARTS_ON);
            if every > 1 && !counted_from.eq_ignore_ascii_case(A_WEEK_STARTS_ON) {
                return None;
            }
            let named: Option<Vec<&str>> = shape
                .days_of_week
                .iter()
                .map(|day| the_day_outlook_named(day))
                .collect();
            let named = named.filter(|days| !days.is_empty())?;
            format!("FREQ=WEEKLY;BYDAY={}", named.join(","))
        }
        "absolutemonthly" => {
            // The date it lands on comes from the day the series starts, which
            // is the only place a rule can keep it. A shape naming another date
            // means other days, so it is refused rather than filed as this one.
            if shape.day_of_month? != starts_on.day() {
                return None;
            }
            "FREQ=MONTHLY".to_string()
        }
        "relativemonthly" => {
            let which = the_week_outlook_named(shape.index.as_deref()?)?;
            format!("FREQ=MONTHLY;BYDAY={which}{}", the_only_day()?)
        }
        "absoluteyearly" => {
            if shape.day_of_month? != starts_on.day() || shape.month? != starts_on.month() {
                return None;
            }
            "FREQ=YEARLY".to_string()
        }
        // The same weekday of the same month every year, and anything Outlook
        // adds later. Neither is something a rule this program reads can say.
        _ => return None,
    };
    let stops = match said.range.range_type.to_ascii_lowercase().as_str() {
        "noend" => Until::Forever,
        // Outlook fills the end in on a series that never ends too, so the kind
        // of range is what decides whether it says anything.
        "enddate" => Until::OnDate(said.range.end_date.clone()?),
        "numbered" => Until::AfterTimes(said.range.number_of_occurrences?),
        _ => return None,
    };

    let mut written = how_often;
    if every > 1 {
        // After the frequency and before the days, which is where a rule
        // written here puts it.
        let (frequency, rest) = written.split_once(';').unwrap_or((&written, ""));
        written = match rest {
            "" => format!("{frequency};INTERVAL={every}"),
            days => format!("{frequency};INTERVAL={every};{days}"),
        };
    }
    if let Some(ending) = stops.ending() {
        written.push(';');
        written.push_str(&ending);
    }
    Some(written)
}

/// What is said about a series when the item is read out.
///
/// Empty for something that does not repeat, so most items cost nothing.
pub fn spoken(repeat: Repeat, until: &Until) -> String {
    if repeat == Repeat::Never {
        return String::new();
    }
    let how_often = repeat.label().to_ascii_lowercase();
    match until {
        Until::Forever => how_often,
        Until::OnDate(date) => format!("{how_often}, until {date}"),
        Until::AfterTimes(1) => format!("{how_often}, once"),
        Until::AfterTimes(times) => format!("{how_often}, {times} times"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_repeating_is_no_rule_at_all() {
        // Rather than a rule that says it does not repeat, which every reader
        // of an .ics file would then have to interpret.
        assert_eq!(rule(Repeat::Never, &Until::Forever, None), None);
    }

    #[test]
    fn test_every_weekday_is_a_pattern_daily_gets_wrong() {
        // This is the one that makes a reminder go off on Sunday.
        assert_eq!(
            rule(Repeat::Weekdays, &Until::Forever, None),
            Some("FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR".to_string())
        );
    }

    #[test]
    fn test_a_series_can_be_told_when_to_stop() {
        // Nothing could say this before, so anything set to repeat repeated
        // for ever and a six week course had to be entered six times.
        assert_eq!(
            rule(Repeat::Weekly, &Until::OnDate("2026-09-30".into()), None),
            Some("FREQ=WEEKLY;UNTIL=20260930T235959Z".to_string())
        );
        assert_eq!(
            rule(Repeat::Weekly, &Until::AfterTimes(6), None),
            Some("FREQ=WEEKLY;COUNT=6".to_string())
        );
    }

    #[test]
    fn test_the_last_day_is_included_in_the_series() {
        // Until the thirtieth means the thirtieth counts. Ending at midnight
        // would quietly drop the last one.
        let written = rule(Repeat::Daily, &Until::OnDate("2026-09-30".into()), None).unwrap();

        assert!(written.contains("T235959Z"), "{written}");
    }

    #[test]
    fn test_every_ending_survives_the_trip() {
        for ending in [
            Until::Forever,
            Until::OnDate("2026-09-30".to_string()),
            Until::AfterTimes(6),
        ] {
            let written = rule(Repeat::Weekly, &ending, None).expect("a rule");

            assert_eq!(Until::from_rule(&written), ending);
        }
    }

    #[test]
    fn test_every_frequency_survives_the_trip() {
        for repeat in Repeat::ALL {
            let Some(written) = rule(repeat, &Until::Forever, Some("2TU")) else {
                assert_eq!(repeat, Repeat::Never);
                continue;
            };
            assert_eq!(Repeat::from_rule(&written), repeat, "for {written}");
        }
    }

    #[test]
    fn test_a_rule_nothing_here_offers_stays_a_repeat() {
        // Answering "does not repeat" would turn somebody's series into a
        // single item the next time they saved it, silently.
        assert_ne!(Repeat::from_rule("FREQ=HOURLY;INTERVAL=6"), Repeat::Never);
        assert_eq!(Repeat::from_rule(""), Repeat::Never);
        assert_eq!(Repeat::from_rule("   "), Repeat::Never);
    }

    #[test]
    fn test_the_monthly_weekday_comes_from_the_date_it_starts() {
        // The ninth of July 2026 is the second Thursday.
        assert_eq!(weekday_of_month("2026-07-09"), Some("2TH".to_string()));
        assert_eq!(
            rule(
                Repeat::MonthlyByWeekday,
                &Until::Forever,
                weekday_of_month("2026-07-09").as_deref()
            ),
            Some("FREQ=MONTHLY;BYDAY=2TH".to_string())
        );
    }

    #[test]
    fn test_which_week_of_the_month_a_date_falls_in() {
        // "2TU" is the second Tuesday. Counting the week wrongly moves a
        // monthly series a week either way for good, and somebody finds out by
        // missing the meeting.
        //
        // July 2026: the 2nd is a Thursday, so the 1st is a Wednesday.
        assert_eq!(weekday_of_month("2026-07-01"), Some("1WE".to_string()));
        assert_eq!(weekday_of_month("2026-07-07"), Some("1TU".to_string()));
        assert_eq!(weekday_of_month("2026-07-08"), Some("2WE".to_string()));
        assert_eq!(weekday_of_month("2026-07-15"), Some("3WE".to_string()));
        assert_eq!(weekday_of_month("2026-07-22"), Some("4WE".to_string()));
    }

    #[test]
    fn test_a_weekday_seven_days_before_the_month_ends_is_not_the_last_one() {
        // The boundary the other cases step over. July 2026 has 31 days and
        // the 1st is a Wednesday, so the 24th is the fourth Friday and the
        // 31st is the last one. Seven days later is still inside the month, so
        // the 24th keeps its ordinal. Calling it the last Friday would move a
        // monthly series onto a different date for good, and in a short month
        // onto a week that is not there.
        assert_eq!(weekday_of_month("2026-07-24"), Some("4FR".to_string()));
        assert_eq!(weekday_of_month("2026-07-31"), Some("-1FR".to_string()));
    }

    #[test]
    fn test_how_many_days_a_month_has() {
        // The last week is worked out from this, so a month the wrong length
        // makes the last Tuesday the second to last, or the other way round.
        assert_eq!(days_in_month(2026, 1), 31);
        assert_eq!(days_in_month(2026, 4), 30);
        assert_eq!(days_in_month(2026, 2), 28);
        // A leap year, and the century rule either side of it.
        assert_eq!(days_in_month(2028, 2), 29);
        assert_eq!(days_in_month(1900, 2), 28);
        assert_eq!(days_in_month(2000, 2), 29);
        // December has to roll the year over to find its own length.
        assert_eq!(days_in_month(2026, 12), 31);
    }

    #[test]
    fn test_a_series_that_repeats_no_times_is_not_a_series() {
        // A count of nought is not a series that stops immediately, it is a
        // rule that says nothing. Reading it as a real count leaves an item
        // that repeats zero times, which is an item nobody ever sees again.
        assert_eq!(Until::from_rule("FREQ=DAILY;COUNT=0"), Until::Forever);
        assert_eq!(Until::from_rule("FREQ=DAILY;COUNT=1"), Until::AfterTimes(1));
        assert_eq!(
            Until::from_rule("FREQ=DAILY;COUNT=12"),
            Until::AfterTimes(12)
        );
    }

    #[test]
    fn test_the_short_name_for_a_repeat_is_understood_as_well_as_the_phrase() {
        // The phrase is what the settings show; the short word is what older
        // stored items hold. Not understanding one turns a series into a
        // single item, silently, which is the failure this function was
        // written to prevent.
        for (written, expected) in [
            ("Daily", Repeat::Daily),
            ("Weekly", Repeat::Weekly),
            ("Monthly", Repeat::Monthly),
            ("Yearly", Repeat::Yearly),
            ("Every day", Repeat::Daily),
            ("Every week", Repeat::Weekly),
            ("Every year", Repeat::Yearly),
            ("  Daily  ", Repeat::Daily),
            ("something nobody offers", Repeat::Never),
        ] {
            assert_eq!(Repeat::from_label(written), expected, "for {written:?}");
        }
    }

    #[test]
    fn test_the_last_weekday_of_a_month_is_written_as_the_last() {
        // The thirtieth of July 2026 is the last Thursday, and writing it as
        // the fifth would skip every month that has only four.
        assert_eq!(weekday_of_month("2026-07-30"), Some("-1TH".to_string()));
    }

    #[test]
    fn test_a_monthly_weekday_with_no_date_falls_back_rather_than_writing_nonsense() {
        // A rule ending in BYDAY= is not a rule.
        let written = rule(Repeat::MonthlyByWeekday, &Until::Forever, None).unwrap();

        assert_eq!(written, "FREQ=MONTHLY");
        assert!(!written.ends_with('='), "{written}");
    }

    #[test]
    fn test_a_date_that_is_not_one_says_nothing() {
        assert_eq!(weekday_of_month("not a date"), None);
        assert_eq!(weekday_of_month(""), None);
        assert_eq!(
            rule(Repeat::Weekly, &Until::OnDate("nonsense".into()), None),
            Some("FREQ=WEEKLY".to_string())
        );
    }

    #[test]
    fn test_an_item_that_repeats_says_so_and_says_when_it_stops() {
        assert_eq!(spoken(Repeat::Never, &Until::Forever), "");
        assert_eq!(spoken(Repeat::Daily, &Until::Forever), "every day");
        assert_eq!(
            spoken(Repeat::Weekly, &Until::OnDate("2026-09-30".into())),
            "every week, until 2026-09-30"
        );
        assert_eq!(
            spoken(Repeat::Weekly, &Until::AfterTimes(6)),
            "every week, 6 times"
        );
    }

    // ── Outlook says the same series ────────────────────────────────

    /// The day every series in these tests starts on: a Tuesday, the second
    /// Tuesday of its month, in a month with a thirty-first.
    const A_TUESDAY: &str = "2026-03-10";

    /// A meeting on the Wednesday after that Tuesday, two hours into the
    /// morning, in a place five and a half hours ahead of Greenwich.
    ///
    /// Stored, it reads as the Wednesday. In the body that goes to Outlook it
    /// is half past eight on the Tuesday evening. The offset is written into
    /// the value so it means the same moment wherever this runs; a clock face
    /// with no zone would be read as a time on the computer running the test
    /// and would convert to itself at Greenwich, which is where the checks run.
    const A_LATE_EVENING_IN_INDIA: (&str, &str) =
        ("2026-03-11T02:00:00+05:30", "2026-03-11T03:00:00+05:30");

    /// The day this program draws a series on, from the rule alone.
    ///
    /// The production reader, not a second one written for the test. Comparing
    /// the days two rules fall on is the only comparison that would notice a
    /// conversion that kept the words and changed the meaning.
    fn the_days_it_falls_on(rule: &str) -> Vec<String> {
        let event = a_stored_event(
            &format!("{A_TUESDAY} 09:00"),
            &format!("{A_TUESDAY} 10:00"),
            rule,
        );
        let from = chrono::NaiveDate::parse_from_str(A_TUESDAY, "%Y-%m-%d").expect("a Tuesday");
        crate::application::occurrences::falls_on(&event, from, from + chrono::Duration::days(900))
            .days
            .into_iter()
            .map(|day| day.start)
            .collect()
    }

    /// A repeating meeting in the diary, starting when it is told to.
    ///
    /// One fixture for both the days a rule falls on and the trip out to
    /// Outlook and back, so the round trip runs on a row the rest of this
    /// program would recognise rather than on a start invented beside it.
    fn a_stored_event(
        start_datetime: &str,
        end_datetime: &str,
        rule: &str,
    ) -> crate::data::message_cache::CalendarEventEntry {
        crate::data::message_cache::CalendarEventEntry {
            id: "e1".to_string(),
            account_id: "a1".to_string(),
            provider_event_id: None,
            calendar_id: None,
            summary: "Standup".to_string(),
            description: None,
            location: None,
            start_datetime: start_datetime.to_string(),
            end_datetime: end_datetime.to_string(),
            start_date: None,
            end_date: None,
            is_all_day: false,
            time_zone: None,
            status: "confirmed".to_string(),
            recurrence_rule: Some(rule.to_string()),
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
            cut_from_event_id: None,
        }
    }

    #[test]
    fn test_every_repeat_this_program_offers_survives_the_trip_to_outlook_and_back() {
        // The two conversions held against each other rather than each against
        // a hand-written expectation, because a hand-written expectation is
        // written twice by the same person on the same afternoon and agrees
        // with itself. This is the check that notices when one direction
        // changes and the other does not, which is the way this project has
        // lost data more often than any other.
        //
        // Compared on the days the series falls on, not on the text. Outlook
        // has to be told a weekday a rule can leave to the start date, so the
        // rule that comes back says more than the one that went out and means
        // exactly the same thing.
        //
        // The day the series begins is taken out of the body that goes to
        // Outlook, not written here and handed to both directions. Handed to
        // both, it agreed with itself whatever the two sides did, so it could
        // not see them differ, which is exactly what they were doing. The start
        // used here is two in the morning in India, which is the evening of the
        // day before once it is in the body, so the two answers are different
        // days and a test that cannot tell them apart goes green while the
        // meeting and its repeat are filed on separate days.
        let mut checked = 0;
        for repeat in Repeat::ALL {
            for ending in [
                Until::Forever,
                Until::OnDate("2026-09-30".to_string()),
                Until::AfterTimes(6),
            ] {
                let Some(written) = rule(repeat, &ending, weekday_of_month(A_TUESDAY).as_deref())
                else {
                    assert_eq!(repeat, Repeat::Never, "{repeat:?} has no rule");
                    continue;
                };
                let event = a_stored_event(
                    A_LATE_EVENING_IN_INDIA.0,
                    A_LATE_EVENING_IN_INDIA.1,
                    &written,
                );
                let body = crate::application::calendar::local_to_ms_event(
                    &event,
                    crate::application::calendar::TheBodyIsFor::MakingIt,
                )
                .expect("an Outlook body");
                let said = body.recurrence.expect("a new series carries its repeat");
                let starts_on = the_day_a_graph_start_names(body.start.as_ref().expect("a start"))
                    .expect("a day");
                assert_eq!(
                    starts_on.to_string(),
                    A_TUESDAY,
                    "the stored clock face reads as the Wednesday after, and the body says the \
                     Tuesday before; the Tuesday is what Outlook is told the meeting is on, so \
                     it is what the repeat has to be counted from"
                );
                let back = what_outlook_said(&said, starts_on)
                    .unwrap_or_else(|| panic!("{written} came back from Outlook unreadable"));

                assert_eq!(
                    the_days_it_falls_on(&back),
                    the_days_it_falls_on(&written),
                    "{written:?} became {back:?}, which falls on other days"
                );
                assert_eq!(
                    Until::from_rule(&back),
                    ending,
                    "{written:?} became {back:?}, which stops somewhere else"
                );
                checked += 1;
            }
        }
        assert_eq!(
            checked, 24,
            "eight repeats that are one, times three endings"
        );
    }

    /// A shape Outlook could have sent, to be spoiled one field at a time.
    fn a_weekly_shape_from_outlook() -> MsPatternedRecurrence {
        MsPatternedRecurrence {
            pattern: MsRecurrencePattern {
                pattern_type: "weekly".to_string(),
                interval: 2,
                days_of_week: vec!["tuesday".to_string()],
                first_day_of_week: Some(A_WEEK_STARTS_ON.to_string()),
                ..Default::default()
            },
            range: MsRecurrenceRange {
                range_type: "noEnd".to_string(),
                start_date: A_TUESDAY.to_string(),
                ..Default::default()
            },
        }
    }

    #[test]
    fn test_a_shape_outlook_sends_that_means_other_days_is_refused_rather_than_read() {
        // Every refusal in the reading, taken. The round trip above only ever
        // hands this shapes that came out of the writer beside it, so it walks
        // the happy arm of each of these and would go on passing with every one
        // of them removed. What is refused here is a series arriving from
        // Outlook that falls on days a rule written here would not.
        let starts_on = chrono::NaiveDate::parse_from_str(A_TUESDAY, "%Y-%m-%d").expect("a day");
        let said = |shape: &MsPatternedRecurrence| what_outlook_said(shape, starts_on);

        // The one it is all about: a fortnightly series counted from Sunday
        // lands in different weeks from one counted from Monday, and a rule can
        // only mean Monday.
        let mut counted_from_sunday = a_weekly_shape_from_outlook();
        counted_from_sunday.pattern.first_day_of_week = Some("sunday".to_string());
        assert_eq!(said(&counted_from_sunday), None);

        // Every week, and the question does not arise, because a series in
        // every week is in all of them however the weeks are counted.
        let mut every_week = counted_from_sunday.clone();
        every_week.pattern.interval = 1;
        assert_eq!(said(&every_week).as_deref(), Some("FREQ=WEEKLY;BYDAY=TU"));

        // A weekly shape naming no day at all is not a shape.
        let mut no_days = a_weekly_shape_from_outlook();
        no_days.pattern.days_of_week.clear();
        assert_eq!(said(&no_days), None);

        // A day Outlook spells in a way this does not know.
        let mut a_strange_day = a_weekly_shape_from_outlook();
        a_strange_day.pattern.days_of_week = vec!["thorsday".to_string()];
        assert_eq!(said(&a_strange_day), None);

        // Nought is a series that never advances, and the reading of a rule
        // refuses one, so this refuses it too rather than deciding on its own
        // that it meant every one.
        let mut never_advances = a_weekly_shape_from_outlook();
        never_advances.pattern.interval = 0;
        assert_eq!(said(&never_advances), None);

        // A date in the month that is not the day the series starts means other
        // days than the rule would, and a rule has nowhere else to keep it.
        let a_month = |day_of_month, month| MsPatternedRecurrence {
            pattern: MsRecurrencePattern {
                pattern_type: "absoluteMonthly".to_string(),
                interval: 1,
                day_of_month,
                month,
                ..Default::default()
            },
            range: a_weekly_shape_from_outlook().range,
        };
        assert_eq!(
            said(&a_month(Some(10), None)).as_deref(),
            Some("FREQ=MONTHLY")
        );
        assert_eq!(said(&a_month(Some(11), None)), None);
        assert_eq!(said(&a_month(None, None)), None);

        let a_year = |day_of_month, month| MsPatternedRecurrence {
            pattern: MsRecurrencePattern {
                pattern_type: "absoluteYearly".to_string(),
                interval: 1,
                day_of_month,
                month,
                ..Default::default()
            },
            range: a_weekly_shape_from_outlook().range,
        };
        assert_eq!(
            said(&a_year(Some(10), Some(3))).as_deref(),
            Some("FREQ=YEARLY")
        );
        assert_eq!(said(&a_year(Some(10), Some(4))), None, "another month");
        assert_eq!(said(&a_year(Some(11), Some(3))), None, "another date");
        assert_eq!(said(&a_year(Some(10), None)), None, "no month at all");

        // The same weekday of the month: the ordinal has to be one this reads,
        // and there has to be exactly one day named.
        let a_weekday_of_the_month =
            |index: Option<&str>, days: Vec<String>| MsPatternedRecurrence {
                pattern: MsRecurrencePattern {
                    pattern_type: "relativeMonthly".to_string(),
                    interval: 1,
                    index: index.map(str::to_string),
                    days_of_week: days,
                    ..Default::default()
                },
                range: a_weekly_shape_from_outlook().range,
            };
        assert_eq!(
            said(&a_weekday_of_the_month(
                Some("second"),
                vec!["tuesday".to_string()]
            ))
            .as_deref(),
            Some("FREQ=MONTHLY;BYDAY=2TU")
        );
        assert_eq!(
            said(&a_weekday_of_the_month(None, vec!["tuesday".to_string()])),
            None,
            "no ordinal"
        );
        assert_eq!(
            said(&a_weekday_of_the_month(
                Some("fifth"),
                vec!["tuesday".to_string()]
            )),
            None,
            "an ordinal Outlook does not have"
        );
        assert_eq!(
            said(&a_weekday_of_the_month(
                Some("second"),
                vec!["tuesday".to_string(), "thursday".to_string()]
            )),
            None,
            "two days, which is a rule this cannot write as one weekday"
        );

        // A shape of a kind this has no rule for at all.
        let mut yearly_by_weekday = a_weekly_shape_from_outlook();
        yearly_by_weekday.pattern.pattern_type = "relativeYearly".to_string();
        assert_eq!(said(&yearly_by_weekday), None);

        // And every ending, including the two that carry a value and the one
        // that is not an ending at all.
        for (range_type, end_date, times, expected) in [
            ("noEnd", None, None, Some("FREQ=WEEKLY;INTERVAL=2;BYDAY=TU")),
            (
                "endDate",
                Some("2026-09-30"),
                None,
                Some("FREQ=WEEKLY;INTERVAL=2;BYDAY=TU;UNTIL=20260930T235959Z"),
            ),
            ("endDate", None, None, None),
            (
                "numbered",
                None,
                Some(6),
                Some("FREQ=WEEKLY;INTERVAL=2;BYDAY=TU;COUNT=6"),
            ),
            ("numbered", None, None, None),
            ("everyOtherTuesday", None, None, None),
        ] {
            let mut shape = a_weekly_shape_from_outlook();
            shape.range.range_type = range_type.to_string();
            shape.range.end_date = end_date.map(str::to_string);
            shape.range.number_of_occurrences = times;
            assert_eq!(said(&shape).as_deref(), expected, "{range_type}");
        }
    }

    #[test]
    fn test_both_spellings_of_every_weekday_are_in_the_one_table() {
        // The conversions refuse a weekday the table has no name for, and that
        // refusal is only ever right while the table is whole. A row lost from
        // it would turn "Outlook cannot say this" into the answer for a
        // perfectly ordinary Wednesday meeting, and the refusal reads as
        // deliberate either way.
        for day in [
            chrono::Weekday::Mon,
            chrono::Weekday::Tue,
            chrono::Weekday::Wed,
            chrono::Weekday::Thu,
            chrono::Weekday::Fri,
            chrono::Weekday::Sat,
            chrono::Weekday::Sun,
        ] {
            let outlook = outlook_calls_the_day(day).unwrap_or_else(|| panic!("{day} at Outlook"));
            let in_a_rule =
                the_day_outlook_named(&outlook).unwrap_or_else(|| panic!("{outlook} in a rule"));
            assert_eq!(
                in_a_rule.len(),
                2,
                "{day} is written {in_a_rule} in a rule, and a rule names a day in two letters"
            );
        }

        // And back again for every week of the month Outlook offers, which is
        // the other table the refusals are read against.
        for (nth, outlook) in THE_WEEKS_OF_A_MONTH {
            assert_eq!(outlook_calls_the_week(nth), Some(outlook));
            assert_eq!(the_week_outlook_named(outlook), Some(nth));
        }
        assert_eq!(
            the_week_outlook_named("fifth"),
            None,
            "Outlook has no fifth week, so nothing may read one"
        );
    }

    #[test]
    fn test_a_repeat_outlook_cannot_say_is_refused_rather_than_rounded() {
        // Refused, because the alternative is a meeting filed on days nobody
        // chose. The lenient reader the item form uses answers "every week" to
        // all four of these, so routing the conversion through it would send
        // Outlook a plain weekly meeting and drop the rest without a word.
        let starts_on = chrono::NaiveDate::parse_from_str(A_TUESDAY, "%Y-%m-%d").expect("a day");
        for beyond in [
            // The second-to-last Tuesday, and the fifth: countable here,
            // and Outlook offers the first four and the last only.
            "FREQ=MONTHLY;BYDAY=-2TU",
            "FREQ=MONTHLY;BYDAY=5TU",
            // Finer than a day, which nothing here draws either.
            "FREQ=HOURLY",
            // A part of the standard nothing here reads.
            "FREQ=MONTHLY;BYSETPOS=2",
        ] {
            assert_eq!(
                as_outlook_says_it(beyond, starts_on),
                None,
                "{beyond} is not something Outlook can be told"
            );
        }

        // And the one that made this worth doing exactly: two named days stay
        // two named days.
        let both = as_outlook_says_it("FREQ=WEEKLY;BYDAY=TU,TH", starts_on).expect("a pattern");
        assert_eq!(both.pattern.days_of_week, ["tuesday", "thursday"]);
    }

    #[test]
    fn test_every_choice_reads_as_a_phrase_rather_than_a_word() {
        // Read from a list where every entry otherwise sounds like the one
        // before it.
        for repeat in Repeat::ALL {
            assert!(
                repeat.label().split_whitespace().count() >= 2,
                "{:?}",
                repeat
            );
        }
    }
}

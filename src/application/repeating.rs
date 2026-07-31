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
    fn ending(&self) -> Option<String> {
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

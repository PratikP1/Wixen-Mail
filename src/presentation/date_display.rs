//! Presenting dates and times so they can be listened to.
//!
//! A timestamp like "2026-07-26 14:30" is fine to look at and poor to hear: a
//! screen reader reads it as a run of digits, and the listener has to assemble
//! a date from it. Spelling the month out and using a twelve hour clock costs
//! nothing visually and saves that work every single row.
//!
//! Relative wording goes further. Most of the time nobody wants the date, they
//! want to know whether it is recent, and "2 days ago" answers that in three
//! syllables where "July 24, 2026 at 9:15 AM" takes a dozen.

use chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone, Timelike};

/// Which way round the day and month are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateOrder {
    /// July 26, 2026
    MonthFirst,
    /// 26 July 2026
    DayFirst,
}

/// How much of a date to say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateStyle {
    /// Always the full date and time.
    Absolute,
    /// "3 hours ago" within the last week, the full date before that.
    RelativeWithinWeek,
}

/// Whether the month is a word or a number.
///
/// Spelled out is easier to hear and longer to read, and which of those matters
/// depends on the person and on whether they are listening or looking. So it is
/// a choice, defaulted from the machine rather than decided for everybody.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateWording {
    /// 07/26/2026, in whichever order the date is written.
    Numeric,
    /// July 26, 2026
    Verbal,
}

/// Whether the clock runs to twelve or to twenty-four.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clock {
    TwelveHour,
    TwentyFourHour,
}

/// Everything that decides how a date is written, in one value.
///
/// One value rather than four parameters, because it is threaded through every
/// list and every reading, and a fifth would otherwise mean touching all of
/// them. Lives here rather than beside the message list, which is where it
/// started, because dates are read in six modules and only one of them is mail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateSettings {
    pub style: DateStyle,
    pub order: DateOrder,
    pub wording: DateWording,
    pub clock: Clock,
}

impl Default for DateSettings {
    /// What the machine already does, so nothing has to be set to get it.
    fn default() -> Self {
        Self {
            style: DateStyle::RelativeWithinWeek,
            order: DateOrder::from_system(),
            wording: DateWording::Verbal,
            clock: Clock::from_system(),
        }
    }
}

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

impl DateStyle {
    /// Read the stored preference, defaulting to relative.
    pub fn from_setting(value: &str) -> Self {
        match value {
            "absolute" => DateStyle::Absolute,
            _ => DateStyle::RelativeWithinWeek,
        }
    }
}

impl DateOrder {
    /// Read the stored preference.
    ///
    /// "auto" follows the system's own date order rather than assuming one, so
    /// the application reads the way the rest of the machine does.
    pub fn from_setting(value: &str) -> Self {
        match value {
            "month_first" => DateOrder::MonthFirst,
            "day_first" => DateOrder::DayFirst,
            _ => Self::from_system(),
        }
    }

    /// The order this machine uses.
    #[cfg(target_os = "windows")]
    pub fn from_system() -> Self {
        // LOCALE_USER_DEFAULT, LOCALE_IDATE: 0 means month first.
        const LOCALE_USER_DEFAULT: u32 = 0x0400;
        const LOCALE_IDATE: u32 = 0x00000021;

        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetLocaleInfoW(locale: u32, lctype: u32, data: *mut u16, size: i32) -> i32;
        }

        let mut buffer = [0u16; 8];
        let written = unsafe {
            GetLocaleInfoW(
                LOCALE_USER_DEFAULT,
                LOCALE_IDATE,
                buffer.as_mut_ptr(),
                buffer.len() as i32,
            )
        };
        if written > 0 && buffer[0] == b'0' as u16 {
            DateOrder::MonthFirst
        } else if written > 0 {
            DateOrder::DayFirst
        } else {
            DateOrder::MonthFirst
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn from_system() -> Self {
        DateOrder::DayFirst
    }
}

impl DateWording {
    /// Read the stored preference.
    pub fn from_setting(value: &str) -> Self {
        match value {
            "numeric" => DateWording::Numeric,
            _ => DateWording::Verbal,
        }
    }
}

impl Clock {
    /// Read the stored preference.
    ///
    /// "auto" follows the machine, so nothing has to be set to get the clock
    /// the rest of the computer already keeps.
    pub fn from_setting(value: &str) -> Self {
        match value {
            "12" => Clock::TwelveHour,
            "24" => Clock::TwentyFourHour,
            _ => Self::from_system(),
        }
    }

    /// The clock this machine keeps.
    #[cfg(target_os = "windows")]
    pub fn from_system() -> Self {
        // LOCALE_ITIME: 0 means the twelve hour clock.
        const LOCALE_USER_DEFAULT: u32 = 0x0400;
        const LOCALE_ITIME: u32 = 0x00000023;

        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetLocaleInfoW(locale: u32, lctype: u32, data: *mut u16, size: i32) -> i32;
        }

        let mut buffer = [0u16; 8];
        let written = unsafe {
            GetLocaleInfoW(
                LOCALE_USER_DEFAULT,
                LOCALE_ITIME,
                buffer.as_mut_ptr(),
                buffer.len() as i32,
            )
        };
        // Anything unreadable falls to twelve, which is what this application
        // did before there was a choice, so an unreadable locale changes
        // nothing rather than changing every row.
        if written > 0 && buffer[0] == b'1' as u16 {
            Clock::TwentyFourHour
        } else {
            Clock::TwelveHour
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn from_system() -> Self {
        Clock::TwentyFourHour
    }
}

/// Format a stored timestamp for a list cell.
///
/// Anything that cannot be parsed is returned unchanged. A date that is not
/// understood is still better shown as it was stored than replaced with a
/// guess or an empty cell.
pub fn format_for_list(stored: &str, now: DateTime<Local>, settings: DateSettings) -> String {
    let Some(when) = parse(stored) else {
        return stored.to_string();
    };

    if settings.style == DateStyle::RelativeWithinWeek
        && let Some(relative) = relative_to(when, now)
    {
        return relative;
    }
    absolute(when, settings)
}

/// One stored date, written the way this reader asked for it.
///
/// The one function everything outside the mail list calls. Before it, the mail
/// list read a date properly and every other module printed the column, so a
/// task due "2026-07-30" was read out as a run of digits, in the place where a
/// date matters most.
///
/// A date stored without a time keeps it that way. A task due on a day is due
/// on that day, and "at 12:00 AM" is a claim the stored value never made, heard
/// on every row.
///
/// Nothing stored is nothing said, rather than the word "none" or today's date.
pub fn spoken(stored: &str, now: DateTime<Local>, settings: DateSettings) -> String {
    if stored.trim().is_empty() {
        return String::new();
    }
    let Some(when) = parse(stored) else {
        return stored.to_string();
    };
    if settings.style == DateStyle::RelativeWithinWeek
        && let Some(relative) = relative_to(when, now)
    {
        return relative;
    }
    if stored_carries_a_time(stored) {
        absolute(when, settings)
    } else {
        date_part(when, settings)
    }
}

/// The full date and time.
pub fn absolute(when: DateTime<Local>, settings: DateSettings) -> String {
    format!("{} at {}", date_part(when, settings), clock(when, settings))
}

/// The date, without the time.
fn date_part(when: DateTime<Local>, settings: DateSettings) -> String {
    let month = MONTHS[(when.month() - 1) as usize];
    match (settings.wording, settings.order) {
        (DateWording::Verbal, DateOrder::MonthFirst) => {
            format!("{} {}, {}", month, when.day(), when.year())
        }
        (DateWording::Verbal, DateOrder::DayFirst) => {
            format!("{} {} {}", when.day(), month, when.year())
        }
        // Padded, because an unpadded numeric date is harder to scan in a
        // column and no shorter to hear.
        (DateWording::Numeric, DateOrder::MonthFirst) => {
            format!("{:02}/{:02}/{}", when.month(), when.day(), when.year())
        }
        (DateWording::Numeric, DateOrder::DayFirst) => {
            format!("{:02}/{:02}/{}", when.day(), when.month(), when.year())
        }
    }
}

/// The clock reading, on whichever clock this reader keeps.
fn clock(when: DateTime<Local>, settings: DateSettings) -> String {
    match settings.clock {
        // No leading zero on the hour: "09:07 AM" is a zero read out for
        // nothing, on every row.
        Clock::TwelveHour => {
            let (is_pm, hour) = when.hour12();
            format!(
                "{}:{:02} {}",
                hour,
                when.minute(),
                if is_pm { "PM" } else { "AM" }
            )
        }
        // Padded, because that is how a twenty-four hour clock is written.
        Clock::TwentyFourHour => format!("{:02}:{:02}", when.hour(), when.minute()),
    }
}

/// Whether the stored value said anything about the time of day.
///
/// By what is written rather than by what it parsed to, because everything
/// parses to a moment: a date alone becomes midnight, and once it has, midnight
/// is indistinguishable from something genuinely due at midnight.
fn stored_carries_a_time(stored: &str) -> bool {
    stored.contains(':')
}

/// How long ago, if that is within the last week.
///
/// Returns `None` beyond a week, where "9 days ago" stops being easier to place
/// than the date itself, and for anything in the future, where a message dated
/// ahead of now is either a clock difference or a forgery and saying "in 3
/// days" would dress that up as normal.
fn relative_to(when: DateTime<Local>, now: DateTime<Local>) -> Option<String> {
    let elapsed = now.signed_duration_since(when);
    if elapsed.num_seconds() < 0 {
        return None;
    }

    let minutes = elapsed.num_minutes();
    if minutes < 1 {
        return Some("just now".to_string());
    }
    if minutes < 60 {
        return Some(plural(minutes, "minute"));
    }

    let hours = elapsed.num_hours();
    if hours < 24 {
        return Some(plural(hours, "hour"));
    }

    let days = elapsed.num_days();
    if days <= 7 {
        return Some(plural(days, "day"));
    }
    None
}

fn plural(count: i64, unit: &str) -> String {
    if count == 1 {
        format!("1 {} ago", unit)
    } else {
        format!("{} {}s ago", count, unit)
    }
}

/// Read a stored timestamp.
///
/// Accepts RFC 3339, which is what sync writes, and the plainer forms that
/// reach the cache from other places.
fn parse(stored: &str) -> Option<DateTime<Local>> {
    let trimmed = stored.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(parsed) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(parsed.with_timezone(&Local));
    }
    for format in ["%Y-%m-%d %H:%M:%S", "%Y-%m-%d %H:%M", "%Y-%m-%d"] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, format) {
            return Local.from_local_datetime(&naive).single();
        }
        if let Ok(date) = chrono::NaiveDate::parse_from_str(trimmed, format) {
            return Local
                .from_local_datetime(&date.and_hms_opt(0, 0, 0)?)
                .single();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> DateTime<Local> {
        parse(text).expect("test timestamp should parse")
    }

    /// Fixed rather than [`DateSettings::default`], which asks the machine, so
    /// these read the same on every machine they run on.
    fn settings() -> DateSettings {
        DateSettings {
            style: DateStyle::Absolute,
            order: DateOrder::MonthFirst,
            wording: DateWording::Verbal,
            clock: Clock::TwelveHour,
        }
    }

    #[test]
    fn test_the_words_can_be_had_as_numbers_instead() {
        // Spelled out is easier to hear and longer to read. Which is better
        // depends on the person and on whether they are listening or looking,
        // so it is a choice rather than a decision made for everybody.
        let numeric = DateSettings {
            wording: DateWording::Numeric,
            ..settings()
        };

        assert_eq!(
            absolute(at("2026-07-26 14:30"), numeric),
            "07/26/2026 at 2:30 PM"
        );
        assert_eq!(
            absolute(
                at("2026-07-26 14:30"),
                DateSettings {
                    order: DateOrder::DayFirst,
                    ..numeric
                }
            ),
            "26/07/2026 at 2:30 PM"
        );
    }

    #[test]
    fn test_the_clock_can_run_to_twenty_four() {
        // Most of the world writes 14:30, and reading it back as "2:30 PM" to
        // somebody who wrote 14:30 is the application arguing with them.
        let day = DateSettings {
            clock: Clock::TwentyFourHour,
            ..settings()
        };

        assert_eq!(
            absolute(at("2026-07-26 14:30"), day),
            "July 26, 2026 at 14:30"
        );
        assert_eq!(
            absolute(at("2026-07-26 00:05"), day),
            "July 26, 2026 at 00:05"
        );
    }

    #[test]
    fn test_the_defaults_come_from_the_machine() {
        // Nobody should have to set this to get what the rest of their computer
        // already does. Whatever it answers has to be one of the two.
        let clock = Clock::from_setting("auto");
        assert!(clock == Clock::TwelveHour || clock == Clock::TwentyFourHour);

        assert_eq!(Clock::from_setting("24"), Clock::TwentyFourHour);
        assert_eq!(Clock::from_setting("12"), Clock::TwelveHour);
        assert_eq!(DateWording::from_setting("numeric"), DateWording::Numeric);
        assert_eq!(DateWording::from_setting("verbal"), DateWording::Verbal);
        // Anything else falls back rather than refusing to show a date.
        assert_eq!(DateWording::from_setting("nonsense"), DateWording::Verbal);
    }

    #[test]
    fn test_one_stored_date_is_spoken_the_same_way_wherever_it_appears() {
        // The mail list read a date properly and everything else printed what
        // was in the column, so a task due "2026-07-30" was read as a run of
        // digits. One function, so a date sounds the same in every module.
        let now = at("2026-07-26 12:00");

        assert_eq!(spoken("2026-07-30", now, settings()), "July 30, 2026");
        assert_eq!(
            spoken("2026-07-30 09:15", now, settings()),
            "July 30, 2026 at 9:15 AM"
        );
    }

    #[test]
    fn test_a_date_with_no_time_does_not_gain_a_midnight() {
        // A task due on a day is due on that day. "at 12:00 AM" is a claim the
        // stored value never made, and it is heard on every row.
        assert!(!spoken("2026-07-30", at("2026-07-26 12:00"), settings()).contains("AM"));
    }

    #[test]
    fn test_nothing_stored_is_nothing_said() {
        // An empty due date is blank, not the word "none" and not today's date.
        for stored in ["", "   "] {
            assert_eq!(spoken(stored, at("2026-07-26 12:00"), settings()), "");
        }
    }

    #[test]
    fn test_month_first_order() {
        assert_eq!(
            absolute(at("2026-07-26 14:30"), settings()),
            "July 26, 2026 at 2:30 PM"
        );
    }

    #[test]
    fn test_day_first_order() {
        assert_eq!(
            absolute(
                at("2026-07-26 14:30"),
                DateSettings {
                    order: DateOrder::DayFirst,
                    ..settings()
                }
            ),
            "26 July 2026 at 2:30 PM"
        );
    }

    #[test]
    fn test_morning_and_midnight_and_noon() {
        // The three that a twelve hour clock gets wrong when hand rolled.
        assert!(absolute(at("2026-07-26 00:05"), settings()).ends_with("12:05 AM"));
        assert!(absolute(at("2026-07-26 12:00"), settings()).ends_with("12:00 PM"));
        assert!(absolute(at("2026-07-26 09:07"), settings()).ends_with("9:07 AM"));
    }

    #[test]
    fn test_relative_wording_within_the_week() {
        let now = at("2026-07-26 12:00");
        let cases = [
            ("2026-07-26 11:59:30", "just now"),
            ("2026-07-26 11:59", "1 minute ago"),
            ("2026-07-26 11:30", "30 minutes ago"),
            ("2026-07-26 11:00", "1 hour ago"),
            ("2026-07-26 00:00", "12 hours ago"),
            ("2026-07-25 12:00", "1 day ago"),
            ("2026-07-24 12:00", "2 days ago"),
            ("2026-07-19 12:00", "7 days ago"),
        ];
        for (stored, expected) in cases {
            assert_eq!(
                format_for_list(
                    stored,
                    now,
                    DateSettings {
                        style: DateStyle::RelativeWithinWeek,
                        ..settings()
                    }
                ),
                expected,
                "for {}",
                stored
            );
        }
    }

    #[test]
    fn test_beyond_a_week_gives_the_date() {
        let now = at("2026-07-26 12:00");
        let shown = format_for_list(
            "2026-07-18 12:00",
            now,
            DateSettings {
                style: DateStyle::RelativeWithinWeek,
                ..settings()
            },
        );
        assert_eq!(shown, "July 18, 2026 at 12:00 PM");
    }

    #[test]
    fn test_a_future_timestamp_is_never_dressed_up_as_relative() {
        // A message dated ahead of now is a clock difference or a forgery.
        // "in 3 days" would present either as ordinary.
        let now = at("2026-07-26 12:00");
        let shown = format_for_list(
            "2026-07-29 12:00",
            now,
            DateSettings {
                style: DateStyle::RelativeWithinWeek,
                ..settings()
            },
        );
        assert_eq!(shown, "July 29, 2026 at 12:00 PM");
    }

    #[test]
    fn test_absolute_style_never_goes_relative() {
        let now = at("2026-07-26 12:00");
        let shown = format_for_list("2026-07-26 11:00", now, settings());
        assert_eq!(shown, "July 26, 2026 at 11:00 AM");
    }

    #[test]
    fn test_rfc3339_is_understood() {
        assert!(parse("2026-07-26T14:30:00+00:00").is_some());
    }

    #[test]
    fn test_a_date_with_no_time_is_understood() {
        assert!(parse("2026-07-26").is_some());
    }

    #[test]
    fn test_an_unparseable_value_is_shown_as_stored() {
        // Better to show what is there than to invent a date or leave a cell
        // that sounds like a row which failed to load.
        let now = at("2026-07-26 12:00");
        for stored in ["not a date", "", "   ", "\u{4f60}\u{597d}"] {
            assert_eq!(
                format_for_list(
                    stored,
                    now,
                    DateSettings {
                        style: DateStyle::RelativeWithinWeek,
                        ..settings()
                    }
                ),
                stored
            );
        }
    }

    #[test]
    fn test_settings_map_to_styles() {
        assert_eq!(DateStyle::from_setting("absolute"), DateStyle::Absolute);
        assert_eq!(
            DateStyle::from_setting("relative"),
            DateStyle::RelativeWithinWeek
        );
        // An unrecognised value falls back rather than refusing to show a date.
        assert_eq!(
            DateStyle::from_setting("nonsense"),
            DateStyle::RelativeWithinWeek
        );
        assert_eq!(DateOrder::from_setting("day_first"), DateOrder::DayFirst);
        assert_eq!(
            DateOrder::from_setting("month_first"),
            DateOrder::MonthFirst
        );
    }

    #[test]
    fn test_auto_order_answers_something() {
        // Whatever the machine says, it has to be one of the two.
        let order = DateOrder::from_setting("auto");
        assert!(order == DateOrder::MonthFirst || order == DateOrder::DayFirst);
    }

    #[test]
    fn test_singular_and_plural_agree() {
        assert_eq!(plural(1, "day"), "1 day ago");
        assert_eq!(plural(2, "day"), "2 days ago");
        assert_eq!(plural(0, "minute"), "0 minutes ago");
    }
}

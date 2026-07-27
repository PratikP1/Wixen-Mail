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
        extern "system" {
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

/// Format a stored timestamp for a list cell.
///
/// Anything that cannot be parsed is returned unchanged. A date that is not
/// understood is still better shown as it was stored than replaced with a
/// guess or an empty cell.
pub fn format_for_list(
    stored: &str,
    now: DateTime<Local>,
    style: DateStyle,
    order: DateOrder,
) -> String {
    let Some(when) = parse(stored) else {
        return stored.to_string();
    };

    if style == DateStyle::RelativeWithinWeek {
        if let Some(relative) = relative_to(when, now) {
            return relative;
        }
    }
    absolute(when, order)
}

/// The full date and time, spelled out.
pub fn absolute(when: DateTime<Local>, order: DateOrder) -> String {
    let month = MONTHS[(when.month() - 1) as usize];
    let date = match order {
        DateOrder::MonthFirst => format!("{} {}, {}", month, when.day(), when.year()),
        DateOrder::DayFirst => format!("{} {} {}", when.day(), month, when.year()),
    };
    format!("{} at {}", date, clock(when))
}

/// A twelve hour clock reading, without a leading zero on the hour.
fn clock(when: DateTime<Local>) -> String {
    let (is_pm, hour) = when.hour12();
    format!(
        "{}:{:02} {}",
        hour,
        when.minute(),
        if is_pm { "PM" } else { "AM" }
    )
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

    #[test]
    fn test_month_first_order() {
        assert_eq!(
            absolute(at("2026-07-26 14:30"), DateOrder::MonthFirst),
            "July 26, 2026 at 2:30 PM"
        );
    }

    #[test]
    fn test_day_first_order() {
        assert_eq!(
            absolute(at("2026-07-26 14:30"), DateOrder::DayFirst),
            "26 July 2026 at 2:30 PM"
        );
    }

    #[test]
    fn test_morning_and_midnight_and_noon() {
        // The three that a twelve hour clock gets wrong when hand rolled.
        assert!(absolute(at("2026-07-26 00:05"), DateOrder::MonthFirst).ends_with("12:05 AM"));
        assert!(absolute(at("2026-07-26 12:00"), DateOrder::MonthFirst).ends_with("12:00 PM"));
        assert!(absolute(at("2026-07-26 09:07"), DateOrder::MonthFirst).ends_with("9:07 AM"));
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
                    DateStyle::RelativeWithinWeek,
                    DateOrder::MonthFirst
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
            DateStyle::RelativeWithinWeek,
            DateOrder::MonthFirst,
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
            DateStyle::RelativeWithinWeek,
            DateOrder::MonthFirst,
        );
        assert_eq!(shown, "July 29, 2026 at 12:00 PM");
    }

    #[test]
    fn test_absolute_style_never_goes_relative() {
        let now = at("2026-07-26 12:00");
        let shown = format_for_list(
            "2026-07-26 11:00",
            now,
            DateStyle::Absolute,
            DateOrder::MonthFirst,
        );
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
                    DateStyle::RelativeWithinWeek,
                    DateOrder::MonthFirst
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

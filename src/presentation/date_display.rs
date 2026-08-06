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

/// What these settings do not reach, said where somebody meets it.
///
/// The order of the day and month and the clock follow this machine, so the
/// dates look as though they follow its language too. They do not: the month
/// names and the relative wording below are English and only English. On a
/// French machine that puts an English word in the middle of every date, read
/// with French pronunciation rules, in every list in every module. It sounds
/// like the screen reader misbehaving rather than like this application
/// speaking one language.
///
/// This is a statement, not a fix. Localising dates means the month names, the
/// relative wording, the examples in the settings screen and eventually every
/// other string in the application, which is a piece of work with a build cost.
/// Until then the limitation is written on the settings screen and in
/// `docs/accessibility.md` rather than left to be discovered.
pub const ENGLISH_ONLY: &str = "Dates are written in English. The order of the day and month \
     and the clock follow this computer, but the month names and wording such as \"2 days ago\" \
     stay in English whatever language this computer is set to.";

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

/// Which way round this machine writes a date. 0 means month first, 1 day
/// first, 2 year first.
///
/// Named here rather than inside the one function that reads it so a test can
/// ask Windows the same question the application asks, rather than a question
/// of its own that happens to have the same answer.
#[cfg(target_os = "windows")]
const LOCALE_IDATE: u32 = 0x0000_0021;

/// Which clock this machine keeps. 0 means twelve hour, 1 means twenty four.
/// Named here for the same reason as [`LOCALE_IDATE`].
#[cfg(target_os = "windows")]
const LOCALE_ITIME: u32 = 0x0000_0023;

/// Ask Windows one thing about the user's locale.
///
/// Answers how many characters were written and the first of them, which is
/// all any of these settings turn on. Holds no decision of its own, so the
/// decisions below can be read without a locale, a window or a machine set
/// any particular way.
#[cfg(target_os = "windows")]
fn read_locale(lctype: u32) -> (i32, u16) {
    const LOCALE_USER_DEFAULT: u32 = 0x0400;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetLocaleInfoW(locale: u32, lctype: u32, data: *mut u16, size: i32) -> i32;
    }

    let mut buffer = [0u16; 8];
    let written = unsafe {
        GetLocaleInfoW(
            LOCALE_USER_DEFAULT,
            lctype,
            buffer.as_mut_ptr(),
            buffer.len() as i32,
        )
    };
    (written, buffer[0])
}

/// What the machine's answer about date order means.
///
/// A locale that cannot be read at all falls back to month first rather than
/// to whatever the untouched buffer happens to hold.
///
/// One condition, the same shape as [`clock_from_locale`]. It used to be three
/// arms, two of which answered month first, and the second `written > 0` in
/// the middle of them could never decide anything: everything it turned away
/// reached the fallback and got the same answer. That is not a style point.
/// A redundant test is one no test can hold to account, and the sweep found it
/// exactly that way, as a comparison that could be loosened with nothing going
/// red.
#[cfg(target_os = "windows")]
fn order_from_locale(written: i32, first: u16) -> DateOrder {
    if written > 0 && first != b'0' as u16 {
        DateOrder::DayFirst
    } else {
        DateOrder::MonthFirst
    }
}

/// What the machine's answer about the clock means.
///
/// Anything unreadable falls to twelve, which is what this application did
/// before there was a choice, so an unreadable locale changes nothing rather
/// than changing every row.
#[cfg(target_os = "windows")]
fn clock_from_locale(written: i32, first: u16) -> Clock {
    if written > 0 && first == b'1' as u16 {
        Clock::TwentyFourHour
    } else {
        Clock::TwelveHour
    }
}

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
        Self::from_setting_or(value, Self::from_system())
    }

    /// The stored preference, with the machine's own order to fall back on.
    ///
    /// Split out from [`DateOrder::from_setting`] so the decision can be read
    /// without asking the machine. With the two joined, a test of the stored
    /// values passes on a machine whose locale happens to agree and says
    /// nothing about whether the stored value was obeyed.
    fn from_setting_or(value: &str, system: Self) -> Self {
        match value {
            "month_first" => DateOrder::MonthFirst,
            "day_first" => DateOrder::DayFirst,
            _ => system,
        }
    }

    /// The order this machine uses.
    #[cfg(target_os = "windows")]
    pub fn from_system() -> Self {
        let (written, first) = read_locale(LOCALE_IDATE);
        order_from_locale(written, first)
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
        Self::from_setting_or(value, Self::from_system())
    }

    /// The stored preference, with the machine's own clock to fall back on.
    ///
    /// Split out for the same reason as [`DateOrder::from_setting_or`]: joined
    /// to the machine reading, a test of the stored values proves only that
    /// this machine agrees with them.
    fn from_setting_or(value: &str, system: Self) -> Self {
        match value {
            "12" => Clock::TwelveHour,
            "24" => Clock::TwentyFourHour,
            _ => system,
        }
    }

    /// The clock this machine keeps.
    #[cfg(target_os = "windows")]
    pub fn from_system() -> Self {
        let (written, first) = read_locale(LOCALE_ITIME);
        clock_from_locale(written, first)
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
    // A day with no year names no moment, so nothing below can measure it or
    // put a clock on it. Handled here rather than left to `parse`, which
    // answers nothing for it and hands the reader "--03-14" character by
    // character.
    if stored.trim().starts_with(YEAR_LEFT_OUT) {
        return a_day_in_words(stored, settings);
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

/// How a stored date whose year nobody gave is written: "--03-14".
///
/// That is what a contact card writes and what a card reader expects, so it is
/// the right thing to keep. It is also the wrong thing to show anybody: a
/// screen reader says it one character at a time, which is why
/// [`a_day_in_words`] exists.
pub const YEAR_LEFT_OUT: &str = "--";

/// One stored day read as words, for a date that may name no year.
///
/// A birthday is the reason this exists. It is never written as how long ago
/// it was, because a birthday every year is not an event three days back, and
/// it never carries a clock reading, because midnight is a claim the stored
/// value never made.
///
/// The month and the day go in whichever order this reader writes a date, the
/// same rule the rest of this module follows. A value that names a year is
/// written the ordinary way. A value that cannot be read comes back exactly as
/// it was stored, so nothing is invented from a run of characters nobody here
/// understands.
pub fn a_day_in_words(stored: &str, settings: DateSettings) -> String {
    let trimmed = stored.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let Some(after_the_missing_year) = trimmed.strip_prefix(YEAR_LEFT_OUT) else {
        return match parse(trimmed) {
            Some(when) => date_part(when, settings),
            None => stored.to_string(),
        };
    };
    let Some((month, day)) = a_month_and_a_day(after_the_missing_year) else {
        return stored.to_string();
    };
    let month_name = MONTHS[(month - 1) as usize];
    match (settings.wording, settings.order) {
        (DateWording::Verbal, DateOrder::MonthFirst) => {
            format!("{} {}", month_name, ordinal(day))
        }
        (DateWording::Verbal, DateOrder::DayFirst) => {
            format!("{} {}", ordinal(day), month_name)
        }
        (DateWording::Numeric, DateOrder::MonthFirst) => format!("{:02}/{:02}", month, day),
        (DateWording::Numeric, DateOrder::DayFirst) => format!("{:02}/{:02}", day, month),
    }
}

/// The two numbers in "03-14", when both name a real month and a real day.
///
/// The range check is not politeness: the month is used to index the month
/// names, so a thirteenth month read out of a corrupt record would take the
/// whole application down rather than show one odd birthday.
fn a_month_and_a_day(written: &str) -> Option<(u32, u32)> {
    let (month, day) = written.split_once('-')?;
    let month: u32 = month.parse().ok()?;
    let day: u32 = day.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some((month, day))
}

/// A day of the month as it is said: "1st", "2nd", "3rd", "4th".
///
/// The eleventh, twelfth and thirteenth take "th" although they end in one,
/// two and three, which is the whole reason this is a function rather than a
/// look at the last digit.
fn ordinal(day: u32) -> String {
    let ending = match (day % 100, day % 10) {
        (11..=13, _) => "th",
        (_, 1) => "st",
        (_, 2) => "nd",
        (_, 3) => "rd",
        _ => "th",
    };
    format!("{}{}", day, ending)
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
    fn test_a_birthday_with_no_year_is_read_as_a_day_and_a_month() {
        assert_eq!(a_day_in_words("--03-14", settings()), "March 14th");
    }

    #[test]
    fn test_a_birthday_with_no_year_follows_the_order_the_locale_uses() {
        let day_first = DateSettings {
            order: DateOrder::DayFirst,
            ..settings()
        };

        assert_eq!(a_day_in_words("--03-14", day_first), "14th March");
    }

    #[test]
    fn test_a_birthday_with_no_year_can_be_had_as_numbers() {
        let month_first = DateSettings {
            wording: DateWording::Numeric,
            ..settings()
        };
        let day_first = DateSettings {
            order: DateOrder::DayFirst,
            wording: DateWording::Numeric,
            ..settings()
        };

        assert_eq!(a_day_in_words("--03-14", month_first), "03/14");
        assert_eq!(a_day_in_words("--03-14", day_first), "14/03");
    }

    #[test]
    fn test_the_eleventh_the_twelfth_and_the_thirteenth_are_not_first_second_and_third() {
        let read = |day: &str| a_day_in_words(&format!("--03-{day}"), settings());

        assert_eq!(read("01"), "March 1st");
        assert_eq!(read("02"), "March 2nd");
        assert_eq!(read("03"), "March 3rd");
        assert_eq!(read("04"), "March 4th");
        assert_eq!(read("11"), "March 11th");
        assert_eq!(read("12"), "March 12th");
        assert_eq!(read("13"), "March 13th");
        assert_eq!(read("21"), "March 21st");
        assert_eq!(read("22"), "March 22nd");
        assert_eq!(read("23"), "March 23rd");
        assert_eq!(read("31"), "March 31st");
    }

    #[test]
    fn test_a_birthday_that_names_no_real_month_is_left_as_it_was_stored() {
        assert_eq!(a_day_in_words("--13-14", settings()), "--13-14");
        assert_eq!(a_day_in_words("--00-14", settings()), "--00-14");
        assert_eq!(a_day_in_words("--03-00", settings()), "--03-00");
        assert_eq!(a_day_in_words("--03-32", settings()), "--03-32");
        assert_eq!(a_day_in_words("--ab-14", settings()), "--ab-14");
        assert_eq!(a_day_in_words("--0314", settings()), "--0314");
    }

    #[test]
    fn test_nothing_stored_is_nothing_said_for_a_day_in_words() {
        assert_eq!(a_day_in_words("", settings()), "");
        assert_eq!(a_day_in_words("   ", settings()), "");
    }

    #[test]
    fn test_a_birthday_with_a_year_is_still_read_as_a_whole_date() {
        assert_eq!(a_day_in_words("1906-12-09", settings()), "December 9, 1906");

        let relative = DateSettings {
            style: DateStyle::RelativeWithinWeek,
            ..settings()
        };
        assert_eq!(
            a_day_in_words("1906-12-09", relative),
            "December 9, 1906",
            "a birthday is never how long ago it was"
        );
    }

    #[test]
    fn test_a_date_with_no_year_reaching_the_ordinary_reading_is_read_as_words() {
        assert_eq!(
            spoken("--03-14", at("2026-07-26 09:15:00"), settings()),
            "March 14th"
        );
    }

    #[test]
    fn test_the_month_is_spelled_in_english_whatever_the_order_says() {
        // A pin, green from the first run, and it is here to go red one day.
        // The settings screen and `docs/accessibility.md` now say out loud that
        // dates are English only. When localisation lands this test fails, and
        // that is the reminder to take those two statements back out rather
        // than leave the application claiming a limitation it no longer has.
        let day_first = DateSettings {
            style: DateStyle::Absolute,
            order: DateOrder::DayFirst,
            wording: DateWording::Verbal,
            clock: Clock::TwentyFourHour,
        };

        let written = date_part(at("2026-07-26 09:15:00"), day_first);

        assert!(written.contains("July"), "{written}");
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
    fn test_a_stored_date_order_is_obeyed_even_when_the_machine_disagrees() {
        // The machine's own order is passed in as the opposite of the stored
        // one on purpose. Asking the machine for it, which is what the whole
        // function used to do, makes the answer depend on the machine the
        // test runs on: "month_first" would pass here and prove nothing on a
        // computer that already writes the month first.
        assert_eq!(
            DateOrder::from_setting_or("month_first", DateOrder::DayFirst),
            DateOrder::MonthFirst
        );
        assert_eq!(
            DateOrder::from_setting_or("day_first", DateOrder::MonthFirst),
            DateOrder::DayFirst
        );
        // Only "auto", and anything unrecognised, follows the machine.
        assert_eq!(
            DateOrder::from_setting_or("auto", DateOrder::DayFirst),
            DateOrder::DayFirst
        );
        assert_eq!(
            DateOrder::from_setting_or("nonsense", DateOrder::MonthFirst),
            DateOrder::MonthFirst
        );
    }

    #[test]
    fn test_a_stored_clock_is_obeyed_even_when_the_machine_disagrees() {
        assert_eq!(
            Clock::from_setting_or("12", Clock::TwentyFourHour),
            Clock::TwelveHour
        );
        assert_eq!(
            Clock::from_setting_or("24", Clock::TwelveHour),
            Clock::TwentyFourHour
        );
        assert_eq!(
            Clock::from_setting_or("auto", Clock::TwentyFourHour),
            Clock::TwentyFourHour
        );
        assert_eq!(
            Clock::from_setting_or("nonsense", Clock::TwelveHour),
            Clock::TwelveHour
        );
    }

    /// Reads what the machine's locale answer means, not what this machine
    /// answers. The call itself is the one part that still needs a real
    /// Windows locale behind it.
    #[cfg(target_os = "windows")]
    #[test]
    fn test_an_unreadable_locale_falls_back_rather_than_guessing_the_date_order() {
        // Nothing written means the buffer was never touched, so reading a
        // character out of it decides the date order by whatever was in
        // memory. Month first instead, on purpose.
        assert_eq!(order_from_locale(0, 0), DateOrder::MonthFirst);
        assert_eq!(order_from_locale(0, b'0' as u16), DateOrder::MonthFirst);
        // And a locale that does answer is obeyed both ways.
        assert_eq!(order_from_locale(1, b'0' as u16), DateOrder::MonthFirst);
        assert_eq!(order_from_locale(1, b'1' as u16), DateOrder::DayFirst);
    }

    /// The other half: that the call really reaches Windows and comes back
    /// with a locale digit in it.
    ///
    /// What it does not prove. Not that the digit agrees with what is set in
    /// Region settings, because nothing here can read that independently. Not
    /// that any date was spoken correctly, which only a screen reader pass
    /// answers. What it does prove is that `GetLocaleInfoW` was asked the
    /// question this application means to ask and answered it, which is what
    /// nothing checked: an application that silently gets no answer falls back
    /// to month first and a twelve hour clock everywhere, and looks like a
    /// choice rather than a broken call.
    ///
    /// The assertions are what Microsoft documents, not what this machine
    /// answers, so a future build image cannot turn this red without the code
    /// changing. The count includes the terminator, so one digit is 2.
    /// `LOCALE_IDATE` is documented as 0, 1 or 2 and `LOCALE_ITIME` as 0 or 1.
    #[cfg(target_os = "windows")]
    #[test]
    fn test_the_locale_answer_is_a_digit_windows_actually_wrote() {
        let (written, first) = read_locale(LOCALE_IDATE);
        assert!(
            written >= 2,
            "Windows wrote {written} characters for the date order"
        );
        assert!(
            (b'0' as u16..=b'2' as u16).contains(&first),
            "the date order came back as {first}, which is not a documented one"
        );

        let (written, first) = read_locale(LOCALE_ITIME);
        assert!(
            written >= 2,
            "Windows wrote {written} characters for the clock"
        );
        assert!(
            (b'0' as u16..=b'1' as u16).contains(&first),
            "the clock came back as {first}, which is not a documented one"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_an_unreadable_locale_leaves_the_clock_where_it_was() {
        // Twelve is what this application did before the setting existed, so
        // a locale that cannot be read changes nothing rather than changing
        // every row in every module.
        assert_eq!(clock_from_locale(0, b'1' as u16), Clock::TwelveHour);
        assert_eq!(clock_from_locale(1, b'1' as u16), Clock::TwentyFourHour);
        assert_eq!(clock_from_locale(1, b'0' as u16), Clock::TwelveHour);
    }

    #[test]
    fn test_a_moment_that_has_only_just_passed_is_just_now() {
        // Nothing elapsed at all is a real case: a message that arrives in the
        // second the list repaints. It is "just now", not the full date.
        let now = at("2026-07-26 12:00");
        assert_eq!(
            format_for_list(
                "2026-07-26 12:00",
                now,
                DateSettings {
                    style: DateStyle::RelativeWithinWeek,
                    ..settings()
                }
            ),
            "just now"
        );
    }

    #[test]
    fn test_singular_and_plural_agree() {
        assert_eq!(plural(1, "day"), "1 day ago");
        assert_eq!(plural(2, "day"), "2 days ago");
        assert_eq!(plural(0, "minute"), "0 minutes ago");
    }
}

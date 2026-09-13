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

use crate::common::how_the_machine_writes_dates as the_machine;
use crate::common::how_the_machine_writes_dates::WhichLocale;
use chrono::{DateTime, Datelike, Local, Timelike};

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
/// The month names now come from this computer, along with the order of the day
/// and month and the clock. What is left in English is the relative wording,
/// "2 days ago", which no Windows API answers and which needs real plural rules
/// in most languages rather than the English one form for 1 and one for
/// everything else.
///
/// **Narrowed rather than removed, and it will narrow again.** The day names in
/// a repeating appointment and the date in a signature outcome are still
/// English too. Saying so is the point: a settings screen that stops mentioning
/// a limitation the moment part of it is fixed is worse than one that never
/// mentioned it, because somebody reading it now believes the rest is done.
///
/// **The comment and the sentence have to name the same things, and they did
/// not.** The paragraph above named the signature date from the first draft of
/// this rewording; the string underneath it, which is the only half anybody
/// reads, said the month names follow this computer and stopped. That is not a
/// smaller claim than the truth, it is a larger one:
/// `src/service/signed_mail.rs` writes `%B` into eight sentences a person hears
/// about a signature, so a month name there is still English. A doc comment is
/// read by whoever is changing this file and the constant is read by whoever is
/// using the program, and the second is the one a disclosure is for.
///
/// No count of what is left, deliberately. The list shortens as each site is
/// done, and a number in the text is one more thing to remember to change.
///
/// Nothing here has been heard. No date written in any language has been read
/// by a screen reader in that language, which is the part no test can settle.
pub const ENGLISH_ONLY: &str = "The month names in a date, the order of the day and month, \
     and the clock all follow this computer. Some wording stays in English whatever language \
     this computer is set to: phrases such as \"2 days ago\", the day names in a repeating \
     appointment, and the date in a message about a signature.";

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
    format_for_list_asking(WhichLocale::ThisComputer, stored, now, settings)
}

/// Every public reading here has one of these beside it, and they exist for the
/// reason [`DateOrder::from_setting_or`] does.
///
/// Joined to the machine's own locale, a test asserting "July 26, 2026" is a
/// statement about the computer it ran on. It would pass here, pass in CI, and
/// say nothing at all about whether the stored preferences were obeyed, because
/// this machine and the assertion happen to agree. Split, a test forces `en-US`
/// and the assertion is about this code.
///
/// The locale is threaded rather than put on [`DateSettings`], and that was a
/// measurement rather than a preference: `DateSettings` is built as a literal
/// at 50 places in 13 files, 27 of them without a `..` spread, so a new field
/// would be 27 edits across files this task has no other business in. These
/// wrappers are one file.
fn format_for_list_asking(
    which: WhichLocale<'_>,
    stored: &str,
    now: DateTime<Local>,
    settings: DateSettings,
) -> String {
    let Some(when) = parse(stored) else {
        return stored.to_string();
    };

    if settings.style == DateStyle::RelativeWithinWeek
        && let Some(relative) = relative_to(when, now)
    {
        return relative;
    }
    absolute_asking(which, when, settings)
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
    spoken_asking(WhichLocale::ThisComputer, stored, now, settings)
}

fn spoken_asking(
    which: WhichLocale<'_>,
    stored: &str,
    now: DateTime<Local>,
    settings: DateSettings,
) -> String {
    use crate::common::moment::Moment;

    if stored.trim().is_empty() {
        return String::new();
    }
    // A day with no year names no moment, so nothing below can measure it or
    // put a clock on it. Handled here rather than left to the moment reader,
    // which answers nothing for it and hands the reader "--03-14" character by
    // character.
    if stored.trim().starts_with(YEAR_LEFT_OUT) {
        return a_day_in_words_asking(which, stored, settings);
    }
    let Some(moment) = crate::common::moment::read(stored) else {
        return stored.to_string();
    };
    // A whole day is a date under every style. Measured against now it would
    // be read from its midnight, and "12 hours ago" at noon about a task due
    // today is the reading calling it overdue. The birthday reading already
    // refuses that; the rule is the same for every stored day.
    if let Moment::WholeDay(day) = moment {
        return date_part_asking(which, day, settings);
    }
    let Some(when) = local_instant(moment) else {
        return stored.to_string();
    };
    if settings.style == DateStyle::RelativeWithinWeek
        && let Some(relative) = relative_to(when, now)
    {
        return relative;
    }
    absolute_asking(which, when, settings)
}

/// How a stored date whose year nobody gave is written: "--03-14".
///
/// That is what a contact card writes and what a card reader expects, so it is
/// the right thing to keep. It is also the wrong thing to show anybody: a
/// screen reader says it one character at a time, which is why
/// [`a_day_in_words`] exists.
///
/// A storage-format constant that belongs to `common`, not to this module;
/// re-exported here because everything below still recognises the format by
/// this name.
pub use crate::common::types::YEAR_LEFT_OUT;

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
    a_day_in_words_asking(WhichLocale::ThisComputer, stored, settings)
}

fn a_day_in_words_asking(which: WhichLocale<'_>, stored: &str, settings: DateSettings) -> String {
    let trimmed = stored.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let Some(after_the_missing_year) = trimmed.strip_prefix(YEAR_LEFT_OUT) else {
        return match parse(trimmed) {
            Some(when) => date_part_asking(which, when, settings),
            None => stored.to_string(),
        };
    };
    let Some((month, day)) = a_month_and_a_day(after_the_missing_year) else {
        return stored.to_string();
    };
    match (settings.wording, settings.order) {
        // A day sits beside the month, so the machine is asked for a date and
        // not for a month name. The year is the one thing this reading has not
        // got, and the module borrows a leap year to stand the date on so that
        // somebody born on the 29th of February can still have their birthday
        // written; nothing of the borrowed year is said.
        (DateWording::Verbal, DateOrder::MonthFirst) => {
            the_machine::a_date(which, the_machine::Shape::MonthDay, month, day)
        }
        (DateWording::Verbal, DateOrder::DayFirst) => {
            the_machine::a_date(which, the_machine::Shape::DayMonth, month, day)
        }
        (DateWording::Numeric, DateOrder::MonthFirst) => format!("{:02}/{:02}", month, day),
        (DateWording::Numeric, DateOrder::DayFirst) => format!("{:02}/{:02}", day, month),
    }
}

/// A month and its year as they are said: "July 2026", or "07/2026".
///
/// Here rather than beside the calendar heading that wants it, because how a
/// month is written is this module's rule and a second answer to it somewhere
/// else is a second thing to keep in step. `month` is 1 for January; anything
/// outside 1 to 12 comes back as the year alone rather than taking the
/// application down on a corrupt record, which is the same care
/// [`a_month_and_a_day`] takes for the same reason.
///
/// The order of the day and month does not arise: there is no day. So the two
/// orders answer alike and only the wording decides.
pub fn a_month_in_words(year: i32, month: u32, settings: DateSettings) -> String {
    a_month_in_words_asking(WhichLocale::ThisComputer, year, month, settings)
}

/// The one reading that really does want the standalone form, because there is
/// no day beside the month for it to agree with.
fn a_month_in_words_asking(
    which: WhichLocale<'_>,
    year: i32,
    month: u32,
    settings: DateSettings,
) -> String {
    if !(1..=12).contains(&month) {
        return year.to_string();
    }
    match settings.wording {
        DateWording::Verbal => {
            let named = the_machine::the_twelve_month_names(which);
            format!("{} {year}", named[(month - 1) as usize])
        }
        DateWording::Numeric => format!("{month:02}/{year}"),
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

/// The hour of the day a stored moment is spoken at, when it names one.
///
/// Built on the same reading and the same zone conversion as [`spoken`], so
/// anything judged by this hour, such as the calendar's out-of-hours note,
/// judges the hour the cell beside it says. A whole day answers nothing: it
/// names no hour, and midnight would put every one of them outside the working
/// day. A value that is not a stored moment answers nothing too, rather than
/// whatever number its characters happen to hold.
pub fn the_hour_spoken(stored: &str) -> Option<u32> {
    use crate::common::moment::Moment;

    match crate::common::moment::read(stored)? {
        Moment::WholeDay(_) => None,
        names_an_hour => local_instant(names_an_hour).map(|when| when.hour()),
    }
}

/// The full date and time.
pub fn absolute(when: DateTime<Local>, settings: DateSettings) -> String {
    absolute_asking(WhichLocale::ThisComputer, when, settings)
}

fn absolute_asking(
    which: WhichLocale<'_>,
    when: DateTime<Local>,
    settings: DateSettings,
) -> String {
    format!(
        "{} at {}",
        date_part_asking(which, when, settings),
        clock(when, settings)
    )
}

/// The date, without the time.
///
/// Over anything with a calendar date on it rather than a full moment, so a
/// whole day is written straight from the day it names and no midnight is
/// constructed just to be left unsaid.
fn date_part_asking(which: WhichLocale<'_>, when: impl Datelike, settings: DateSettings) -> String {
    match (settings.wording, settings.order) {
        // The person's stored wording and order choose which of these two the
        // machine is asked for, and the machine chooses only the words inside
        // it. Asking Windows for its own idea of a long date instead would
        // have written a French date the French way and thrown away the order
        // this person picked, on every machine whose locale disagrees with
        // them.
        (DateWording::Verbal, DateOrder::MonthFirst) => the_machine::a_date(
            which,
            the_machine::Shape::MonthDayYear(when.year()),
            when.month(),
            when.day(),
        ),
        (DateWording::Verbal, DateOrder::DayFirst) => the_machine::a_date(
            which,
            the_machine::Shape::DayMonthYear(when.year()),
            when.month(),
            when.day(),
        ),
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
/// The shapes are `common::moment`'s rather than a list kept here. This module
/// had its own, which knew RFC 3339 and three space-separated forms and neither
/// of the two Microsoft Graph writes. Nothing answered for those, so every
/// reading of an Outlook event was the stored string itself,
/// "2026-07-27T09:00:00", handed to the announcement as the words to say.
///
/// A moment carrying its own offset is moved to this computer's zone, because
/// what somebody wants to hear is the hour they will be sitting down at. A
/// clock face names an hour and nothing else, so it is read as an hour here. A
/// whole day is midnight here; [`spoken`] answers a whole day before it gets
/// this far, so no reading treats that midnight as an hour somebody named.
fn parse(stored: &str) -> Option<DateTime<Local>> {
    crate::common::moment::read(stored).and_then(local_instant)
}

/// Where a parsed moment falls on this computer's clock.
fn local_instant(moment: crate::common::moment::Moment) -> Option<DateTime<Local>> {
    // One answer, in `common::moment`, shared with what decides whether a
    // reminder is due and with the order a calendar lists its days in.
    moment.on_this_computer()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> DateTime<Local> {
        parse(text).expect("test timestamp should parse")
    }

    /// Forced, so every English assertion below is about this code rather than
    /// about the computer that ran it.
    ///
    /// Before the month names came from the machine, every one of these could
    /// be written against the public reading and be right anywhere, because
    /// the answer was English wherever you asked. That is no longer true: the
    /// same assertion through [`absolute`] passes here and fails on a French
    /// machine, and it would be the test that was wrong, not the code.
    fn english() -> WhichLocale<'static> {
        WhichLocale::NamedInATest("en-US")
    }

    /// The four readings under a forced `en-US`. Named apart from the public
    /// ones on purpose: a test module that shadowed `absolute` with a local
    /// `absolute` would read as testing the public function while testing
    /// something else.
    fn as_english_says_it(when: DateTime<Local>, settings: DateSettings) -> String {
        absolute_asking(english(), when, settings)
    }

    fn an_english_day(stored: &str, settings: DateSettings) -> String {
        a_day_in_words_asking(english(), stored, settings)
    }

    fn an_english_reading(stored: &str, now: DateTime<Local>, settings: DateSettings) -> String {
        spoken_asking(english(), stored, now, settings)
    }

    fn an_english_cell(stored: &str, now: DateTime<Local>, settings: DateSettings) -> String {
        format_for_list_asking(english(), stored, now, settings)
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
        // No ordinal. `date_part` has always written "9 December 1906" without
        // one, so this reading used to disagree with its own module about the
        // same day; a date picture cannot produce "14th"; and no language
        // other than English wants one.
        assert_eq!(an_english_day("--03-14", settings()), "March 14");
    }

    #[test]
    fn test_a_birthday_with_no_year_follows_the_order_the_locale_uses() {
        let day_first = DateSettings {
            order: DateOrder::DayFirst,
            ..settings()
        };

        assert_eq!(an_english_day("--03-14", day_first), "14 March");
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

        assert_eq!(an_english_day("--03-14", month_first), "03/14");
        assert_eq!(an_english_day("--03-14", day_first), "14/03");
    }

    /// A birthday on a day the calendar only has every four years is a real
    /// person's, and it is the case that breaks if a year-less date is stood
    /// on any year at all: Windows checks the whole date and refuses the 29th
    /// of February in a year that has none.
    #[test]
    fn test_a_birthday_on_the_twenty_ninth_of_february_is_still_read_as_a_day() {
        assert_eq!(an_english_day("--02-29", settings()), "February 29");
    }

    #[test]
    fn test_a_birthday_that_names_no_real_month_is_left_as_it_was_stored() {
        assert_eq!(an_english_day("--13-14", settings()), "--13-14");
        assert_eq!(an_english_day("--00-14", settings()), "--00-14");
        assert_eq!(an_english_day("--03-00", settings()), "--03-00");
        assert_eq!(an_english_day("--03-32", settings()), "--03-32");
        assert_eq!(an_english_day("--ab-14", settings()), "--ab-14");
        assert_eq!(an_english_day("--0314", settings()), "--0314");
    }

    #[test]
    fn test_nothing_stored_is_nothing_said_for_a_day_in_words() {
        assert_eq!(an_english_day("", settings()), "");
        assert_eq!(an_english_day("   ", settings()), "");
    }

    #[test]
    fn test_a_birthday_with_a_year_is_still_read_as_a_whole_date() {
        assert_eq!(an_english_day("1906-12-09", settings()), "December 9, 1906");

        let relative = DateSettings {
            style: DateStyle::RelativeWithinWeek,
            ..settings()
        };
        assert_eq!(
            an_english_day("1906-12-09", relative),
            "December 9, 1906",
            "a birthday is never how long ago it was"
        );
    }

    #[test]
    fn test_a_date_with_no_year_reaching_the_ordinary_reading_is_read_as_words() {
        assert_eq!(
            an_english_reading("--03-14", at("2026-07-26 09:15:00"), settings()),
            "March 14"
        );
    }

    #[test]
    fn test_the_month_is_spelled_the_way_this_computer_spells_it() {
        // This replaces a pin that was written to be green and to go red the
        // day localisation landed, as the reminder to take the English-only
        // statements back out. That day is this one, and the pin could not
        // have kept its promise: it asked the machine's own locale, so on an
        // English machine it would have gone on passing through the whole
        // change. What it was really pinning is asserted here instead, on both
        // sides, with the locale forced rather than read.
        let day_first = DateSettings {
            style: DateStyle::Absolute,
            order: DateOrder::DayFirst,
            wording: DateWording::Verbal,
            clock: Clock::TwentyFourHour,
        };
        let when = at("2026-07-26 09:15:00");

        assert_eq!(date_part_asking(english(), when, day_first), "26 July 2026");
    }

    /// The month name follows the machine, and the shape does not.
    ///
    /// Both halves matter and the second is the one that is easy to lose.
    /// Asking Windows for its own idea of a long date would have written a
    /// French date the French way and thrown away the order and wording this
    /// person chose, on every machine whose locale disagrees with them.
    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_french_machine_gets_a_french_month_in_the_order_the_person_chose() {
        let french = WhichLocale::NamedInATest("fr-FR");
        let when = at("2026-07-26 09:15:00");
        let day_first = DateSettings {
            order: DateOrder::DayFirst,
            ..settings()
        };

        assert_eq!(date_part_asking(french, when, day_first), "26 juillet 2026");
        // Month first is what this person chose, and it is obeyed even though
        // no French machine writes a date that way on its own.
        assert_eq!(
            date_part_asking(french, when, settings()),
            "juillet 26, 2026"
        );
    }

    /// The two mechanisms, seen from up here rather than inside the wrapper.
    ///
    /// A date has a day in it and wants the genitive; a month heading has no
    /// day and wants the standalone form. If these two ever answer alike, one
    /// of the two readings is using the other's mechanism.
    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_date_and_a_month_heading_name_the_same_month_differently() {
        let russian = WhichLocale::NamedInATest("ru-RU");
        let in_a_date = date_part_asking(russian, at("2026-01-02 09:15:00"), settings());
        let on_its_own = a_month_in_words_asking(russian, 2026, 1, settings());

        assert!(in_a_date.contains("января"), "{in_a_date}");
        assert!(on_its_own.starts_with("Январь"), "{on_its_own}");
    }

    /// The four stored combinations, each written out whole.
    ///
    /// This is the criterion's own test: the same code asked for `en-US` still
    /// produces what this program always produced, so nothing that worked
    /// stops working. The two numeric ones never go near the machine, because
    /// a numeric date has no month name in it to localise and letting a locale
    /// decide its separators would change a shape the person chose.
    #[test]
    fn test_the_four_stored_combinations_still_read_as_they_always_did() {
        let when = at("2026-03-14 09:15:00");
        let verbal_month_first = settings();
        let verbal_day_first = DateSettings {
            order: DateOrder::DayFirst,
            ..settings()
        };
        let numeric_month_first = DateSettings {
            wording: DateWording::Numeric,
            ..settings()
        };
        let numeric_day_first = DateSettings {
            order: DateOrder::DayFirst,
            wording: DateWording::Numeric,
            ..settings()
        };

        assert_eq!(
            date_part_asking(english(), when, verbal_month_first),
            "March 14, 2026"
        );
        assert_eq!(
            date_part_asking(english(), when, verbal_day_first),
            "14 March 2026"
        );
        assert_eq!(
            date_part_asking(english(), when, numeric_month_first),
            "03/14/2026"
        );
        assert_eq!(
            date_part_asking(english(), when, numeric_day_first),
            "14/03/2026"
        );
    }

    #[test]
    fn test_a_month_heading_outside_the_twelve_is_the_year_on_its_own() {
        // A corrupt stored row must not index past the end of the twelve.
        assert_eq!(
            a_month_in_words_asking(english(), 2026, 0, settings()),
            "2026"
        );
        assert_eq!(
            a_month_in_words_asking(english(), 2026, 13, settings()),
            "2026"
        );
        assert_eq!(
            a_month_in_words_asking(english(), 2026, 7, settings()),
            "July 2026"
        );
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
            as_english_says_it(at("2026-07-26 14:30"), day),
            "July 26, 2026 at 14:30"
        );
        assert_eq!(
            as_english_says_it(at("2026-07-26 00:05"), day),
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

        assert_eq!(
            an_english_reading("2026-07-30", now, settings()),
            "July 30, 2026"
        );
        assert_eq!(
            an_english_reading("2026-07-30 09:15", now, settings()),
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
    fn test_a_whole_day_under_the_relative_style_is_a_date_and_never_an_hour_count() {
        // The shipped default. A task due "2026-07-26" was measured from its
        // midnight and announced as "12 hours ago" at noon, which calls a task
        // due today overdue. A day is a date under every style, the same rule
        // the birthday reading already keeps.
        let relative = DateSettings {
            style: DateStyle::RelativeWithinWeek,
            ..settings()
        };
        let noon = at("2026-07-26 12:00");

        assert_eq!(
            an_english_reading("2026-07-26", noon, relative),
            "July 26, 2026"
        );
        assert_eq!(
            an_english_reading("2026-07-25", noon, relative),
            "July 25, 2026"
        );
    }

    #[test]
    fn test_a_whole_day_in_the_future_stays_a_date_under_the_relative_style() {
        // The boundary pin: the future side never went relative, and this
        // holds it there.
        let relative = DateSettings {
            style: DateStyle::RelativeWithinWeek,
            ..settings()
        };

        assert_eq!(
            an_english_reading("2026-07-30", at("2026-07-26 12:00"), relative),
            "July 30, 2026"
        );
    }

    #[test]
    fn test_nothing_stored_is_nothing_said() {
        // An empty due date is blank, not the word "none" and not today's date.
        for stored in ["", "   "] {
            assert_eq!(spoken(stored, at("2026-07-26 12:00"), settings()), "");
        }
    }

    #[test]
    fn test_the_hour_spoken_comes_from_the_parsed_moment_not_the_characters() {
        // The calendar's out-of-hours note judges the hour this answers, so it
        // has to be the hour of the same parsed, locally-converted moment the
        // cell speaks, not whatever digits sit after the first separator.
        assert_eq!(the_hour_spoken("2026-07-27 19:00"), Some(19));
        assert_eq!(the_hour_spoken("2026-07-27T09:00:00.0000000"), Some(9));
        // A whole day names no hour, so it earns no note.
        assert_eq!(the_hour_spoken("2026-07-27"), None);
        // Garbage used to answer ninety-nine.
        assert_eq!(the_hour_spoken("junk 99:00"), None);
        assert_eq!(the_hour_spoken(""), None);
        // An offset lands on this computer's own hour, so only the shape is
        // asserted or the test would read differently machine to machine.
        assert!(the_hour_spoken("2026-07-27T09:00:00Z").is_some());
    }

    #[test]
    fn test_month_first_order() {
        assert_eq!(
            as_english_says_it(at("2026-07-26 14:30"), settings()),
            "July 26, 2026 at 2:30 PM"
        );
    }

    #[test]
    fn test_day_first_order() {
        assert_eq!(
            as_english_says_it(
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
        let shown = an_english_cell(
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
        let shown = an_english_cell(
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
        let shown = an_english_cell("2026-07-26 11:00", now, settings());
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

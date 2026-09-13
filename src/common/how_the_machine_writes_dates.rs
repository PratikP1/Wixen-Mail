//! Ask this computer how it writes a date, rather than writing it in English.
//!
//! The order of the day and the month, and the twelve or twenty-four hour
//! clock, have followed this machine for a while. The month names did not: they
//! came from a hardcoded English array, so a French machine put an English word
//! in the middle of every date and a French voice read it with French
//! pronunciation. That sounds like the screen reader misbehaving rather than
//! like this application speaking one language.
//!
//! # Why there are two ways in and not one
//!
//! A month name inside a date and a month name on its own are different words
//! in several languages, and asking the wrong way is not untidy, it is
//! ungrammatical.
//!
//! Microsoft's page for `LOCALE_SMONTHNAME1` says a month name looked up on its
//! own is "the standalone, or nominative, form", and that to get the genitive an
//! application "calls GetDateFormat or GetDateFormatEx with a date picture of
//! ddMMMM". Its `GetDateFormatEx` page says the genitive form is retrieved "when
//! the date picture contains both a numeric form of the day (either d or dd) and
//! the full month name (MMMM)".
//!
//! Measured on this machine on 2026-09-13 rather than taken on the documents'
//! word, because a claim about an API is worth a minute to check:
//!
//! | asked | Russian | Polish |
//! |---|---|---|
//! | [`a_date`] with a day beside the month | `2 января` | `2 stycznia` |
//! | [`the_twelve_month_names`], no day | `Январь` | `styczeń` |
//!
//! Those are different words, not different capitalisation. So [`a_date`] is for
//! a date that has a day in it, and [`the_twelve_month_names`] is for a list
//! where there is no day, such as the month choice in the item form. Using
//! either for the other's job writes bad Russian and bad Polish.
//!
//! # What decides the shape, and what decides the words
//!
//! The person's stored preferences decide the shape and only the shape. They
//! chose whether the day comes before the month and whether the month is a word
//! or a number, and this module must not take that away by asking Windows for
//! its own idea of a long date. So the caller picks a [`Shape`], which is the
//! preference, and the machine supplies the names inside it.
//!
//! # The one untrusted-input surface
//!
//! `GetDateFormatEx` takes a locale name, and this program reads strings out of
//! messages and files all day. No locale name here comes from one.
//! [`WhichLocale::ThisComputer`] is the only value shipping code passes, and
//! [`WhichLocale::NamedInATest`] takes a literal written in a test, which is
//! what lets a test assert English on a machine set to anything.
//!
//! # Where this lives, and why it is not in `presentation`
//!
//! `src/service/signed_mail.rs` writes a date into a sentence a person hears,
//! and `src/service/` reaches `presentation` nowhere today. Putting the wrapper
//! beside the other date code would have made this the plan that started that
//! direction. `src/common/` imports nothing from the other three layers, which
//! is the same reason `common::moment` is where it is.
//!
//! # Where there is no Windows locale API
//!
//! The crate still builds, and every answer is English. That is not a way of
//! skipping the question: it is the documented fallback, and it is the same code
//! path a Windows machine takes when the call fails, so it is exercised by tests
//! on both platforms rather than only on the one nobody builds.

use crate::common::Result;

/// Only the half that talks to Windows ever builds one, so only that half
/// imports it. Measured on 2026-09-13 by compiling the other half on this
/// machine: an import the compiled arm does not use is a warning, and warnings
/// are build failures here, so an unconditional import would have broken the
/// Linux and macOS builds while every Windows check stayed green.
#[cfg(target_os = "windows")]
use crate::common::Error;

/// The twelve month names in English, January first.
///
/// Not a preference and not a default anybody chose. It is the answer where
/// there is no Windows locale API to ask and the answer where the ask failed,
/// which is the silent English fallback the success criterion asks for in as
/// many words.
const IN_ENGLISH: [&str; 12] = [
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

/// A year to stand a year-less date on, so Windows will look at it.
///
/// A birthday stored as "--02-29" belongs to somebody, and Windows checks the
/// whole date against the calendar before it writes a word of it: the 29th of
/// February in a year that has none is refused outright with error 87,
/// measured on 2026-09-13. So a day with no year of its own has to borrow one,
/// and it has to be a leap year or that person's birthday cannot be written.
///
/// Nothing ever sees it. The two year-less shapes write no year, checked the
/// same day across 1601, 1900, 2026, 2400 and 9999, every one of which wrote
/// the same "March 14".
///
/// Windows only, along with the two methods that read it, because a year to
/// stand a date on is a thing Windows asks for and the English fallback never
/// needs one.
#[cfg(target_os = "windows")]
const A_YEAR_WITH_A_TWENTY_NINTH_OF_FEBRUARY: i32 = 2024;

/// Which locale to ask.
///
/// Three variants rather than a bare string, because each has its own rule
/// about where the value may come from and a string cannot say which one it
/// is. None of the three is ever built from a message, a file, or anything a
/// person typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhichLocale<'a> {
    /// Whatever this computer is set to. What shipping code passes when it
    /// wants a date.
    ThisComputer,
    /// A named locale, for a test that has to read the same on every machine
    /// it runs on. A literal written in a test.
    NamedInATest(&'a str),
    /// The language a translation catalogue declares itself to be written in.
    /// A literal in this tree beside the catalogue it names, or in a test
    /// beside a resource written there, and it reaches here through the
    /// bundle that speaks it: a number inside a Russian sentence is written
    /// the Russian way whatever this computer is set to, because the sentence
    /// around it is Russian.
    ACatalogueIsWrittenIn(&'a str),
}

/// The shapes this program writes a date in.
///
/// Four rather than a free-text picture. A picture is a string the caller can
/// get wrong, and Windows does not refuse a wrong one: asked for `ZZZZ` it
/// answers `ZZZZ`, measured on 2026-09-13. An enum cannot be got wrong, and it
/// carries its year where the year appears, so there is no year to pass for a
/// shape that writes none.
///
/// The two that name a year come from the person's stored wording and order.
/// The two that do not are for a birthday nobody gave a year for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// "14 March 2026"
    DayMonthYear(i32),
    /// "March 14, 2026"
    MonthDayYear(i32),
    /// "14 March"
    DayMonth,
    /// "March 14"
    MonthDay,
}

impl Shape {
    /// The date picture Windows is given for this shape.
    ///
    /// Every one of the four carries both a numeric day and `MMMM`, which is
    /// the condition Microsoft states for the genitive month form. That is not
    /// a coincidence to be tidied away later: a picture here that lost its day
    /// would go on writing correct English and start writing wrong Russian.
    #[cfg(target_os = "windows")]
    fn picture(self) -> &'static str {
        match self {
            Shape::DayMonthYear(_) => "d MMMM yyyy",
            Shape::MonthDayYear(_) => "MMMM d, yyyy",
            Shape::DayMonth => "d MMMM",
            Shape::MonthDay => "MMMM d",
        }
    }

    /// The year this shape is asked about, which for the two that write none
    /// is only there to make the date one Windows will look at.
    #[cfg(target_os = "windows")]
    fn year(self) -> i32 {
        match self {
            Shape::DayMonthYear(year) | Shape::MonthDayYear(year) => year,
            Shape::DayMonth | Shape::MonthDay => A_YEAR_WITH_A_TWENTY_NINTH_OF_FEBRUARY,
        }
    }

    /// This shape written in English, asking nothing.
    ///
    /// A month outside 1 to 12 has no name, so the date is written without one
    /// rather than indexing past the end of the twelve. A stored row can hold
    /// anything, and an odd-looking birthday is a better answer than the
    /// application going down while reading a contact card.
    fn in_english(self, month: u32, day: u32) -> String {
        let name = IN_ENGLISH
            .get(month.wrapping_sub(1) as usize)
            .copied()
            .unwrap_or_default();
        match self {
            Shape::DayMonthYear(year) => format!("{day} {name} {year}"),
            Shape::MonthDayYear(year) => format!("{name} {day}, {year}"),
            Shape::DayMonth => format!("{day} {name}"),
            Shape::MonthDay => format!("{name} {day}"),
        }
        .trim()
        .to_string()
    }
}

/// One date, written the way this computer writes one in that shape.
///
/// `month` is 1 for January. Anything this computer will not write, a locale
/// name that is not one, a day no month has, a year outside what a Windows
/// date can hold, comes back in English rather than empty or missing. A
/// corrupt stored row should show an odd date, not take the reading down.
pub fn a_date(which: WhichLocale<'_>, shape: Shape, month: u32, day: u32) -> String {
    ask_for_a_date(which, shape, month, day).unwrap_or_else(|_| shape.in_english(month, day))
}

/// The twelve month names this computer uses, January first, each on its own.
///
/// On its own is the point: these go in a list where no day sits beside them,
/// so they want the standalone form and not the one a date puts a month in.
pub fn the_twelve_month_names(which: WhichLocale<'_>) -> [String; 12] {
    ask_for_the_twelve_month_names(which).unwrap_or_else(|_| IN_ENGLISH.map(str::to_string))
}

/// What this computer calls its own locale: "en-US", "fr-FR", "en-GB".
///
/// The one question here whose answer is a locale name rather than a word in
/// one. It is what chooses a translation catalogue, and the caller decides
/// what a read that fails means; this does not fall back to English on its
/// own, because "this computer could not say" and "this computer said
/// English" are different answers and the caller's fallback is the same
/// either way.
///
/// `src/service/spellcheck/mod.rs` reads the same constant through the older
/// `GetLocaleInfoW`, for the spelling dictionary. Two readers of one fact;
/// measured agreeing on this machine on 2026-09-13, both answering `en-US`.
/// Retiring one is a version 2 job, because that file is fingerprinted by
/// thirty guard records and this layer cannot reach it.
pub fn this_computers_locale_name() -> Result<String> {
    ask_this_computers_locale_name()
}

/// One number, written the way that locale writes one: "1,234" in English,
/// "1.234" in German, "1 234" with a no-break space in Russian.
///
/// For the four counts a date reading passes today, every one under sixty,
/// this and the plain digits agree in every locale there is. It exists so
/// that the day a catalogue message carries a larger number, or a fraction,
/// the number inside a sentence is written by the same Windows that writes
/// the date beside it, rather than becoming a second opinion.
///
/// `None` when Windows will not write it, or where there is no Windows, and
/// the caller writes the digits itself. Never an empty string: a sentence
/// with a hole where its number was is worse than one with plain digits.
pub fn a_number(which: WhichLocale<'_>, value: f64) -> Option<String> {
    ask_for_a_number(which, value)
}

/// What Windows writes into a caller's buffer, as it is laid out in memory.
///
/// `day_of_week` is left at zero and Windows works the real one out: asked for
/// `dddd` on the 14th of March 2026 with this field zero it answered
/// "Saturday", measured on 2026-09-13. Worth knowing rather than relying on
/// silently, because a day name is the next thing that wants asking.
#[cfg(target_os = "windows")]
#[repr(C)]
struct WindowsDate {
    year: u16,
    month: u16,
    day_of_week: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    milliseconds: u16,
}

/// The first of the twelve standalone month names.
///
/// The other eleven follow it one at a time. That they are consecutive was
/// measured rather than assumed, across four locales on 2026-09-13, because a
/// wrong constant here would answer a real month name for the wrong month and
/// nothing would look broken.
#[cfg(target_os = "windows")]
const FIRST_MONTH_NAMED_ON_ITS_OWN: u32 = 0x0000_0038;

/// A string as Windows wants one: its characters, then a zero.
#[cfg(target_os = "windows")]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// The characters Windows wrote, without the zero it counted.
///
/// Both calls here answer the number of characters written **including** the
/// terminating zero: "14 March 2026" is thirteen characters and the call
/// answers fourteen, measured. Taking that answer at face value puts a zero on
/// the end of every date this program speaks, which a screen reader then has
/// to decide what to do with.
#[cfg(target_os = "windows")]
fn without_the_zero(written: &[u16]) -> String {
    String::from_utf16_lossy(written.strip_suffix(&[0]).unwrap_or(written))
}

/// Whatever Windows last complained about.
#[cfg(target_os = "windows")]
fn what_went_wrong() -> u32 {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetLastError() -> u32;
    }

    unsafe { GetLastError() }
}

/// The locale name as a pointer, and the buffer keeping it alive.
///
/// Null rather than an empty string for this computer, and that is the trap
/// worth naming out loud. An empty name is `LOCALE_NAME_INVARIANT`, which
/// answers English on every machine there is. Passed where the user's own
/// locale was meant, every date stays English, every test stays green, and the
/// whole of this module does nothing. Measured on 2026-09-13: on this en-US
/// machine the two are indistinguishable, which is exactly why the mistake
/// would have survived being tested here.
#[cfg(target_os = "windows")]
fn as_windows_wants_it(which: WhichLocale<'_>) -> Option<Vec<u16>> {
    match which {
        WhichLocale::ThisComputer => None,
        WhichLocale::NamedInATest(name) | WhichLocale::ACatalogueIsWrittenIn(name) => {
            Some(wide(name))
        }
    }
}

#[cfg(target_os = "windows")]
fn ask_for_a_date(which: WhichLocale<'_>, shape: Shape, month: u32, day: u32) -> Result<String> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetDateFormatEx(
            locale: *const u16,
            flags: u32,
            date: *const WindowsDate,
            picture: *const u16,
            into: *mut u16,
            how_many: i32,
            calendar: *const u16,
        ) -> i32;
    }

    let year = shape.year();
    let refused = |what: &str| {
        Error::Other(format!(
            "no Windows date has a {what}: {year}-{month}-{day}"
        ))
    };
    let when = WindowsDate {
        year: u16::try_from(year).map_err(|_| refused("year like that"))?,
        month: u16::try_from(month).map_err(|_| refused("month like that"))?,
        day: u16::try_from(day).map_err(|_| refused("day like that"))?,
        day_of_week: 0,
        hour: 0,
        minute: 0,
        second: 0,
        milliseconds: 0,
    };

    let named = as_windows_wants_it(which);
    let locale = named.as_ref().map_or(std::ptr::null(), Vec::as_ptr);
    let picture = wide(shape.picture());

    // Asked how long the answer is before being given anywhere to put it,
    // rather than guessing a length. A buffer too small is refused outright
    // with error 122 and nothing is written into it, measured, so the guess
    // that went wrong would show up as a date that vanished.
    let how_many = unsafe {
        GetDateFormatEx(
            locale,
            0,
            &when,
            picture.as_ptr(),
            std::ptr::null_mut(),
            0,
            std::ptr::null(),
        )
    };
    if how_many <= 0 {
        return Err(Error::Other(format!(
            "Windows would not size a date: error {}",
            what_went_wrong()
        )));
    }

    let mut buffer = vec![0u16; how_many as usize];
    let written = unsafe {
        GetDateFormatEx(
            locale,
            0,
            &when,
            picture.as_ptr(),
            buffer.as_mut_ptr(),
            how_many,
            std::ptr::null(),
        )
    };
    if written <= 0 {
        return Err(Error::Other(format!(
            "Windows would not write a date: error {}",
            what_went_wrong()
        )));
    }

    Ok(without_the_zero(&buffer[..written as usize]))
}

/// One thing Windows knows about a locale, by name.
///
/// `GetLocaleInfoEx` rather than `GetLocaleInfoW`, and the reason is the one
/// that chose `GetDateFormatEx` too: the `Ex` forms take a locale *name*, so a
/// test can force one and assert the same answer on every machine. The older
/// form takes a numeric locale id, and the only id shipping code has is
/// "whatever this machine is", which makes every test a statement about the
/// machine that ran it.
#[cfg(target_os = "windows")]
fn ask_windows_about(which: WhichLocale<'_>, wanted: u32) -> Result<String> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetLocaleInfoEx(locale: *const u16, wanted: u32, into: *mut u16, how_many: i32) -> i32;
    }

    let named = as_windows_wants_it(which);
    let locale = named.as_ref().map_or(std::ptr::null(), Vec::as_ptr);

    let how_many = unsafe { GetLocaleInfoEx(locale, wanted, std::ptr::null_mut(), 0) };
    if how_many <= 0 {
        return Err(Error::Other(format!(
            "Windows would not size what it calls {wanted:#x}: error {}",
            what_went_wrong()
        )));
    }

    let mut buffer = vec![0u16; how_many as usize];
    let written = unsafe { GetLocaleInfoEx(locale, wanted, buffer.as_mut_ptr(), how_many) };
    if written <= 0 {
        return Err(Error::Other(format!(
            "Windows would not say what it calls {wanted:#x}: error {}",
            what_went_wrong()
        )));
    }

    Ok(without_the_zero(&buffer[..written as usize]))
}

#[cfg(target_os = "windows")]
fn ask_for_the_twelve_month_names(which: WhichLocale<'_>) -> Result<[String; 12]> {
    let mut names: [String; 12] = std::array::from_fn(|_| String::new());
    for (months_along, name) in names.iter_mut().enumerate() {
        *name = ask_windows_about(which, FIRST_MONTH_NAMED_ON_ITS_OWN + months_along as u32)?;
    }
    Ok(names)
}

/// The name of a locale, as Windows spells it: "en-US". `LOCALE_SNAME`.
#[cfg(target_os = "windows")]
const THE_NAME_OF_THE_LOCALE: u32 = 0x0000_005C;

#[cfg(target_os = "windows")]
fn ask_this_computers_locale_name() -> Result<String> {
    ask_windows_about(WhichLocale::ThisComputer, THE_NAME_OF_THE_LOCALE)
}

#[cfg(target_os = "windows")]
fn ask_for_a_number(_which: WhichLocale<'_>, _value: f64) -> Option<String> {
    None
}

/// Where there is no Windows locale API, English, and nothing said about it.
///
/// This is the same path a Windows machine takes when the call fails, so it is
/// exercised by tests on both platforms rather than only on the one nobody
/// builds. It is not a way of skipping the question: every caller still calls,
/// and what it gets back is the documented answer rather than a silence.
#[cfg(not(target_os = "windows"))]
fn ask_for_a_date(_which: WhichLocale<'_>, shape: Shape, month: u32, day: u32) -> Result<String> {
    Ok(shape.in_english(month, day))
}

#[cfg(not(target_os = "windows"))]
fn ask_for_the_twelve_month_names(_which: WhichLocale<'_>) -> Result<[String; 12]> {
    Ok(IN_ENGLISH.map(str::to_string))
}

/// A locale name is the one question with no English answer to give: "this
/// computer could not say" is the truthful one, and the caller's fallback
/// turns it into English the same way a failed read on Windows is.
#[cfg(not(target_os = "windows"))]
fn ask_this_computers_locale_name() -> Result<String> {
    Err(crate::common::Error::Other(
        "there is no Windows locale API to ask what this computer's locale is called".to_string(),
    ))
}

/// The caller writes the digits itself, which is the same path a Windows
/// machine takes when the call fails.
#[cfg(not(target_os = "windows"))]
fn ask_for_a_number(_which: WhichLocale<'_>, _value: f64) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The assertions that must hold on every machine, whatever it is set to.
    ///
    /// This is why the two entry points take a locale name at all. Reading the
    /// machine's own locale would make every one of these a statement about
    /// the machine that ran them.
    #[test]
    fn test_an_english_machine_still_gets_the_english_this_program_always_wrote() {
        let english = WhichLocale::NamedInATest("en-US");

        assert_eq!(
            a_date(english, Shape::DayMonthYear(2026), 3, 14),
            "14 March 2026"
        );
        assert_eq!(
            a_date(english, Shape::MonthDayYear(2026), 3, 14),
            "March 14, 2026"
        );
        assert_eq!(a_date(english, Shape::DayMonth, 3, 14), "14 March");
        assert_eq!(a_date(english, Shape::MonthDay, 3, 14), "March 14");
    }

    /// Windows only, and this is not a way of skipping it.
    ///
    /// Where there is no Windows locale API every answer is English, by
    /// design, so asking a machine like that for French and asserting French
    /// would be asserting something this module never promised. The English
    /// assertions above run on both and are what holds this to its word there.
    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_french_machine_gets_a_french_month() {
        let french = WhichLocale::NamedInATest("fr-FR");

        assert_eq!(
            a_date(french, Shape::DayMonthYear(2026), 3, 14),
            "14 mars 2026"
        );
        assert_eq!(a_date(french, Shape::DayMonth, 3, 14), "14 mars");
    }

    /// The whole reason this module has two ways in.
    ///
    /// Russian and Polish put a month into a different case when a day stands
    /// beside it. If these two ever answer alike, one of the two mechanisms has
    /// been used for the other's job and a date somewhere is ungrammatical.
    ///
    /// Windows only, for the reason given above the French one.
    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_month_inside_a_date_is_a_different_word_from_the_month_named_alone() {
        let russian = WhichLocale::NamedInATest("ru-RU");
        let polish = WhichLocale::NamedInATest("pl-PL");

        assert_eq!(a_date(russian, Shape::DayMonth, 1, 2), "2 января");
        assert_eq!(the_twelve_month_names(russian)[0], "Январь");

        assert_eq!(a_date(polish, Shape::DayMonth, 1, 2), "2 stycznia");
        assert_eq!(the_twelve_month_names(polish)[0], "styczeń");
    }

    /// Windows only, for the reason given above the French one.
    #[cfg(target_os = "windows")]
    #[test]
    fn test_the_twelve_names_for_a_list_come_from_the_machine() {
        assert_eq!(
            the_twelve_month_names(WhichLocale::NamedInATest("fr-FR")),
            [
                "janvier",
                "février",
                "mars",
                "avril",
                "mai",
                "juin",
                "juillet",
                "août",
                "septembre",
                "octobre",
                "novembre",
                "décembre",
            ]
            .map(str::to_string)
        );
    }

    /// Windows refuses a locale name that is not one, with error 87, measured
    /// on 2026-09-13. Nothing here may hand back a half-written buffer or an
    /// empty string for it.
    ///
    /// A well-formed name the machine has no data for, such as `xx-XX`, is a
    /// different case and is not tested: Windows answers English for it without
    /// reporting anything, so a test of it would pass whether this code handled
    /// it or not.
    #[test]
    fn test_a_locale_name_that_is_not_one_falls_back_to_english() {
        let nonsense = WhichLocale::NamedInATest("not a locale");

        assert_eq!(
            a_date(nonsense, Shape::DayMonthYear(2026), 3, 14),
            "14 March 2026"
        );
        assert_eq!(the_twelve_month_names(nonsense)[0], "January");
    }

    /// A stored birthday of "--02-29" is a real person's, and it is the reason
    /// a year-less shape cannot be given just any year to stand on.
    ///
    /// Windows checks the whole date against the calendar, so the 29th of
    /// February in a year that has none is refused outright, measured. A day
    /// with no year of its own therefore has to borrow a leap year.
    #[test]
    fn test_a_birthday_on_the_twenty_ninth_of_february_is_written_rather_than_refused() {
        let english = WhichLocale::NamedInATest("en-US");

        assert_eq!(a_date(english, Shape::DayMonth, 2, 29), "29 February");
        assert_eq!(a_date(english, Shape::MonthDay, 2, 29), "February 29");
    }

    /// A day the calendar does not have at all, which a stored row can still
    /// hold, comes back in English rather than as nothing.
    #[test]
    fn test_a_day_no_month_has_is_still_written_rather_than_lost() {
        let english = WhichLocale::NamedInATest("en-US");

        assert_eq!(a_date(english, Shape::DayMonth, 2, 30), "30 February");
        assert_eq!(a_date(english, Shape::DayMonth, 4, 31), "31 April");
    }

    /// Windows writes dates from 1601 to 30827 and refuses everything outside
    /// that, measured. A year beyond it must fall back rather than wrap: the
    /// year goes into a sixteen bit field, and 70000 truncated is 4464, which
    /// is a date Windows would happily write and nobody stored.
    #[test]
    fn test_a_year_outside_what_the_machine_will_write_falls_back_rather_than_wrapping() {
        let english = WhichLocale::NamedInATest("en-US");

        assert_eq!(
            a_date(english, Shape::DayMonthYear(70000), 3, 14),
            "14 March 70000"
        );
        assert_eq!(
            a_date(english, Shape::DayMonthYear(1500), 3, 14),
            "14 March 1500"
        );
    }

    /// A month outside 1 to 12 indexes nothing, here or in the fallback.
    #[test]
    fn test_a_month_that_is_not_one_is_written_without_a_name_rather_than_panicking() {
        let english = WhichLocale::NamedInATest("en-US");

        assert_eq!(a_date(english, Shape::DayMonth, 13, 14), "14");
        assert_eq!(a_date(english, Shape::DayMonth, 0, 14), "14");
    }

    /// Windows only, because where there is no locale API the honest answer
    /// is that the question cannot be asked, and that arm says so.
    ///
    /// A weak assertion on purpose: whatever this computer is called, the
    /// name has a language and a region in it. Asserting `en-US` would be a
    /// statement about the machine the test ran on.
    #[cfg(target_os = "windows")]
    #[test]
    fn test_this_computer_can_say_what_its_locale_is_called() {
        let name = this_computers_locale_name().expect("a locale name from Windows");

        let (language, region) = name.split_once('-').expect("a language and a region");
        assert!(language.chars().all(|c| c.is_ascii_lowercase()), "{name}");
        assert!(!region.is_empty(), "{name}");
    }

    /// The four counts a date reading passes today never reach a thousand,
    /// so a value under sixty proves nothing: Windows and plain digits agree
    /// on "5" in every locale there is. 1234 is where they part.
    ///
    /// Windows only, for the reason given above the French month test.
    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_number_is_written_the_way_that_locale_writes_one() {
        assert_eq!(
            a_number(WhichLocale::ACatalogueIsWrittenIn("en-US"), 1234.0).as_deref(),
            Some("1,234")
        );
        assert_eq!(
            a_number(WhichLocale::ACatalogueIsWrittenIn("de-DE"), 1234.0).as_deref(),
            Some("1.234")
        );
        // Russian groups with U+00A0, a no-break space, measured on this
        // machine on 2026-09-13. A test asserting an ordinary space would be
        // wrong in a way nobody can see on a screen.
        assert_eq!(
            a_number(WhichLocale::ACatalogueIsWrittenIn("ru-RU"), 1234.0).as_deref(),
            Some("1\u{a0}234")
        );
        // Nothing appended: a null format pointer makes Windows write "5.00",
        // measured, and a count is not a price.
        assert_eq!(
            a_number(WhichLocale::ACatalogueIsWrittenIn("en-US"), 5.0).as_deref(),
            Some("5")
        );
    }

    /// A value Windows will not write comes back as nothing, so the caller
    /// writes the digits itself, rather than as an empty string with a hole
    /// in the sentence where the number was.
    #[test]
    fn test_a_number_windows_cannot_write_is_left_to_the_caller() {
        assert_eq!(a_number(WhichLocale::NamedInATest("en-US"), f64::NAN), None);
        assert_eq!(
            a_number(WhichLocale::NamedInATest("not a locale"), 1234.0),
            None
        );
    }
}

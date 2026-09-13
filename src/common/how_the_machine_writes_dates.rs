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

/// Which locale to ask.
///
/// Two variants rather than a bare string, because the two have different
/// rules about where the value may come from and a string cannot say which
/// one it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhichLocale<'a> {
    /// Whatever this computer is set to. The only one shipping code uses.
    ThisComputer,
    /// A named locale, for a test that has to read the same on every machine
    /// it runs on. A literal written in a test, never a value out of a message,
    /// a file, or anything a person typed.
    NamedInATest(&'a str),
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

/// One date, written the way this computer writes one in that shape.
///
/// `month` is 1 for January. A month or a day that names no real date comes
/// back in English rather than empty or missing, because a corrupt stored row
/// should show an odd date and not take the reading down.
pub fn a_date(which: WhichLocale<'_>, shape: Shape, month: u32, day: u32) -> String {
    // A body that is present and wrong, so the tests below fail on what they
    // assert rather than on a name the compiler cannot find. Replaced at green.
    format!("{which:?} {shape:?} {month} {day}")
}

/// The twelve month names this computer uses, January first, each on its own.
///
/// On its own is the point: these go in a list where no day sits beside them,
/// so they want the standalone form and not the one a date puts a month in.
pub fn the_twelve_month_names(which: WhichLocale<'_>) -> [String; 12] {
    // Wrong on purpose, for the same reason as [`a_date`].
    std::array::from_fn(|which_month| format!("{which:?} {which_month}"))
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
    #[test]
    fn test_a_month_inside_a_date_is_a_different_word_from_the_month_named_alone() {
        let russian = WhichLocale::NamedInATest("ru-RU");
        let polish = WhichLocale::NamedInATest("pl-PL");

        assert_eq!(a_date(russian, Shape::DayMonth, 1, 2), "2 января");
        assert_eq!(the_twelve_month_names(russian)[0], "Январь");

        assert_eq!(a_date(polish, Shape::DayMonth, 1, 2), "2 stycznia");
        assert_eq!(the_twelve_month_names(polish)[0], "styczeń");
    }

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
}

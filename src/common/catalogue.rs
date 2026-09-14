//! The sentences this program speaks, read out of a translation catalogue.
//!
//! Four messages today, the relative wording a date is read with: "just now",
//! "5 minutes ago", "3 hours ago", "2 days ago". They are the first piece of
//! version 2, which translates the interface and the screen reader's speech,
//! and this module is shaped for the five thousand sentences that follow
//! rather than for the four that are here.
//!
//! # Why a catalogue and not a plural function
//!
//! "2 days ago" was written by a function with two arms, one for 1 and one for
//! everything else, which is English. Russian has four forms, Arabic six, and
//! the rule for which form a number takes belongs to the language, not to the
//! sentence. Project Fluent carries Unicode's plural rules for every language
//! and lets a translator write the variants their language distinguishes and
//! no more. The catalogue itself is `locales/en-US/dates.ftl`, whose header
//! holds the conventions.
//!
//! # The bundle's locale is the catalogue's language, never the machine's
//!
//! Plural rules select on the bundle's locale. English text under Russian
//! rules writes "21 day ago", because Russian puts 21 in the singular form,
//! and under French rules "0 minute ago". So the machine's locale chooses
//! *which* catalogue, and the chosen catalogue's own language is what the
//! bundle is built with. With one catalogue compiled in, every machine gets
//! English, silently, which is what the success criterion asks for where there
//! is no translation.
//!
//! # Three settings, applied in one place
//!
//! Every bundle, shipped or built in a test, goes through [`a_bundle_speaking`]
//! and gets the same three settings, so a test of a Russian resource is a test
//! of the shipping path's settings and not of a bundle configured some other
//! way:
//!
//! 1. Isolation off. Fluent's default wraps every placeable in U+2068 and
//!    U+2069, the Unicode bidi isolates, so that a right-to-left name inside a
//!    left-to-right sentence keeps its direction. Microsoft's text stack has
//!    been seen rendering them as junk in a window title, and a screen reader
//!    given them reads whatever its synthesiser makes of two invisible
//!    characters. Firefox ships with them off for the same reason. A string
//!    holding `\u{2068}2\u{2069} days ago` does not contain `2 days ago`, so
//!    every exact-string test here is a second witness to this setting.
//! 2. Numbers written by Windows. Fluent writes a number with `f64`'s own
//!    `to_string`, which is "1234" in every language. The date beside it is
//!    written by `GetDateFormatEx`, so the number goes through
//!    `GetNumberFormatEx` too, and the sentence has one opinion about how
//!    this locale writes. The formatter is a plain `fn` and learns which
//!    locale it is writing through [`TheLocaleThisBundleSpeaks`], which is
//!    why `intl-memoizer` is a direct dependency.
//! 3. Counts as numbers, and the error list read. A count passed as a string
//!    matches no plural category and falls through to `*[other]` with no error,
//!    so `1` as `"1"` writes "1 days ago" and nothing says so. A missing
//!    argument writes `{$days}` into the sentence and pushes one error. Both
//!    are silent unless the list `format_pattern` fills is read, and it is
//!    read at the one place that calls it.
//!
//! # Where the catalogue comes from
//!
//! Compiled in with `include_str!`, on the precedent of the spelling
//! dictionary, so a missing file is a build failure and the installer carries
//! nothing extra. Nothing under `locales/` is read by the running program.
//! Parsed once per process. The bundle is the concurrent one, because the
//! date readings are called from six files and the ordinary bundle's memoizer
//! is `RefCell`-backed and not `Sync`.

use std::sync::OnceLock;

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use fluent_langneg::{NegotiationStrategy, negotiate_languages};
use intl_memoizer::Memoizable;
use intl_memoizer::concurrent::IntlLangMemoizer;
use unic_langid::LanguageIdentifier;

use crate::common::how_the_machine_writes_dates::{self as the_machine, WhichLocale};
use crate::common::{Error, Result};

/// One declaration, three products: the typed ids the code asks with, the
/// string each is written as in the catalogue, and the `ALL` list the
/// completeness check walks. A message that is declared here and missing
/// from the catalogue fails a test, and so does one in the catalogue that is
/// declared nowhere, because a check that asks only one of those passes on an
/// orphaned message forever.
///
/// `counting` names the variable a message takes, in the catalogue's own
/// terms, so a caller passes a number and never a name. A message without
/// one takes no argument.
macro_rules! messages {
    ($($(#[$doc:meta])* $name:ident = $id:literal $(counting $counts:literal)?),* $(,)?) => {
        /// A sentence the code can ask the catalogue for.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Message {
            $($(#[$doc])* $name,)*
        }

        impl Message {
            /// Every message, for a check to walk.
            pub const ALL: [Message; messages!(@count $($name)*)] = [$(Message::$name,)*];

            /// The id this message has in the catalogue.
            pub fn id(self) -> &'static str {
                match self {
                    $(Message::$name => $id,)*
                }
            }

            /// The variable this message counts with, or nothing.
            pub fn counts(self) -> Option<&'static str> {
                match self {
                    $(Message::$name => messages!(@counts $($counts)?),)*
                }
            }
        }
    };
    (@count) => { 0 };
    (@count $head:ident $($tail:ident)*) => { 1 + messages!(@count $($tail)*) };
    (@counts) => { None };
    (@counts $counts:literal) => { Some($counts) };
}

messages! {
    /// A moment under a minute old.
    JustNow = "dates-just-now",
    /// Within the hour.
    MinutesAgo = "dates-minutes-ago" counting "minutes",
    /// Within the day.
    HoursAgo = "dates-hours-ago" counting "hours",
    /// Within the week.
    DaysAgo = "dates-days-ago" counting "days",
    /// Within the hour ahead.
    InMinutes = "dates-in-minutes" counting "minutes",
    /// Within the day ahead.
    InHours = "dates-in-hours" counting "hours",
    /// Within the week ahead.
    InDays = "dates-in-days" counting "days",
}

/// A catalogue compiled into this program, and the bundle built from it once.
struct Shipped {
    /// The language it is written in, which is the bundle's locale.
    locale: &'static str,
    dates: &'static str,
    built: OnceLock<Result<Catalogue>>,
}

/// The catalogues this program ships. One, and the day a second arrives it
/// is one more entry here and one more directory under `locales/`.
static SHIPPED: [Shipped; 1] = [Shipped {
    locale: "en-US",
    dates: include_str!("../../locales/en-US/dates.ftl"),
    built: OnceLock::new(),
}];

/// The catalogue every machine gets when none matches its language.
const WHERE_THERE_IS_NO_TRANSLATION: &str = "en-US";

/// A catalogue in one language: the sentences, and the rules for that
/// language's plurals and numbers.
pub struct Catalogue {
    bundle: FluentBundle<FluentResource>,
}

/// The catalogue for a locale, chosen from the ones compiled in.
///
/// The choice is language negotiation, RFC 4647 through `fluent-langneg`,
/// against the list of shipped catalogues, with English as the answer when
/// nothing matches. A locale name this computer answers that is not one, and
/// a read that fails, both ask for English too, and nothing is said about
/// either, because that is the documented fallback and not a fault.
pub fn for_this(which: WhichLocale<'_>) -> Result<&'static Catalogue> {
    let requested = match which {
        WhichLocale::ThisComputer => requested_locale(the_machine::this_computers_locale_name()),
        WhichLocale::NamedInATest(name) | WhichLocale::ACatalogueIsWrittenIn(name) => {
            requested_locale(Ok(name.to_string()))
        }
    };
    let shipped = SHIPPED
        .iter()
        .find(|catalogue| catalogue.locale == chosen_for(&requested))
        .ok_or_else(|| Error::Other(format!("no catalogue is compiled in for {requested}")))?;
    shipped
        .built
        .get_or_init(|| {
            a_bundle_speaking(shipped.locale, shipped.dates).map(|bundle| Catalogue { bundle })
        })
        .as_ref()
        .map_err(|why| Error::Other(why.to_string()))
}

/// The locale a name asks for, and English when the name cannot be read.
///
/// English by name rather than `LanguageIdentifier::default()`, which is
/// `und`, the undetermined language. Negotiation would turn `und` into
/// English today because English is the default catalogue, so the two look
/// alike with one catalogue compiled in; they stop looking alike the day a
/// second arrives and `und` is negotiated against a list rather than sent
/// straight to the fallback. The fallback clause is "English", so ask for it.
fn requested_locale(name: Result<String>) -> LanguageIdentifier {
    name.ok()
        .and_then(|name| name.parse().ok())
        .unwrap_or_else(english)
}

/// The catalogue every machine gets when none matches its language.
fn english() -> LanguageIdentifier {
    // Parsed from a literal that is also the name of a shipped catalogue,
    // and the test that walks `SHIPPED` holds that it parses; the fallback
    // is only for the type's sake.
    WHERE_THERE_IS_NO_TRANSLATION.parse().unwrap_or_default()
}

/// Which shipped catalogue serves a requested locale.
fn chosen_for(requested: &LanguageIdentifier) -> &'static str {
    let available: Vec<LanguageIdentifier> = SHIPPED
        .iter()
        .filter_map(|catalogue| catalogue.locale.parse().ok())
        .collect();
    let english = english();
    let chosen = negotiate_languages(
        std::slice::from_ref(requested),
        &available,
        Some(&english),
        NegotiationStrategy::Filtering,
    );
    let chosen = chosen
        .first()
        .map_or_else(|| english.clone(), |locale| (*locale).clone());
    SHIPPED
        .iter()
        .map(|catalogue| catalogue.locale)
        .find(|locale| locale.parse::<LanguageIdentifier>().ok().as_ref() == Some(&chosen))
        .unwrap_or(WHERE_THERE_IS_NO_TRANSLATION)
}

/// The one function that builds a bundle, shipped or test, and the one place
/// the three settings are applied.
fn a_bundle_speaking(locale: &str, source: &str) -> Result<FluentBundle<FluentResource>> {
    let locale: LanguageIdentifier = locale.parse().map_err(|why| {
        Error::Other(format!(
            "{locale} is not a locale a catalogue can be in: {why}"
        ))
    })?;
    let resource = FluentResource::try_new(source.to_string()).map_err(|(_, errors)| {
        Error::Other(format!(
            "the catalogue for {locale} does not parse: {errors:?}"
        ))
    })?;
    let mut bundle = FluentBundle::new_concurrent(vec![locale.clone()]);
    // Setting 1. Off, as Firefox ships it; the module comment says what the
    // marks do to a Windows title bar and a screen reader.
    bundle.set_use_isolating(false);
    // Setting 2. Numbers by Windows, in the bundle's own language.
    bundle.set_formatter(Some(as_this_locale_writes_a_number));
    bundle.add_resource(resource).map_err(|errors| {
        Error::Other(format!(
            "the catalogue for {locale} repeats itself: {errors:?}"
        ))
    })?;
    Ok(bundle)
}

/// What language a bundle is writing, learned the only way a plain `fn` can.
///
/// `set_formatter` takes a function pointer, so the formatter has no state of
/// its own, and the memoizer's locale is private. What the memoizer will do
/// is construct a value of any type that implements `Memoizable`, once per
/// bundle, handing it the locale. This is that type, and the locale is all it
/// keeps.
struct TheLocaleThisBundleSpeaks(String);

impl Memoizable for TheLocaleThisBundleSpeaks {
    type Args = ();
    type Error = ();

    fn construct(lang: LanguageIdentifier, (): ()) -> std::result::Result<Self, ()> {
        Ok(Self(lang.to_string()))
    }
}

/// Numbers written the way the bundle's language writes them, by Windows.
///
/// `None` hands the value back to Fluent, which writes the digits itself;
/// that is the answer for anything that is not a number and for a number
/// Windows will not write. Never an empty string.
fn as_this_locale_writes_a_number(
    value: &FluentValue<'_>,
    memoizer: &IntlLangMemoizer,
) -> Option<String> {
    let FluentValue::Number(number) = value else {
        return None;
    };
    memoizer
        .with_try_get::<TheLocaleThisBundleSpeaks, _, _>((), |locale| {
            the_machine::a_number(WhichLocale::ACatalogueIsWrittenIn(&locale.0), number.value)
        })
        .ok()
        .flatten()
}

impl Catalogue {
    /// A sentence that takes no argument.
    pub fn say(&self, message: Message) -> Result<String> {
        self.format(message, FluentArgs::new())
    }

    /// A sentence with a count in it, the count going in as a number so the
    /// language's plural rules can look at it.
    pub fn say_how_many(&self, message: Message, count: i64) -> Result<String> {
        let counts = message
            .counts()
            .ok_or_else(|| Error::Other(format!("{} takes no count", message.id())))?;
        let mut args = FluentArgs::new();
        // Setting 3, first half. As a number: a string matches no plural
        // category and falls to `*[other]` with no error, so "1 days ago".
        args.set(counts, count);
        self.format(message, args)
    }

    fn format(&self, message: Message, args: FluentArgs<'_>) -> Result<String> {
        let found = self
            .bundle
            .get_message(message.id())
            .ok_or_else(|| Error::Other(format!("{} is not in the catalogue", message.id())))?;
        let pattern = found
            .value()
            .ok_or_else(|| Error::Other(format!("{} has no text", message.id())))?;
        let mut errors = Vec::new();
        let text = self
            .bundle
            .format_pattern(pattern, Some(&args), &mut errors);
        // Setting 3, second half. A missing argument is written into the
        // sentence as `{$days}` and reported here, and only here.
        if !errors.is_empty() {
            return Err(Error::Other(format!(
                "{} could not be said: {errors:?}",
                message.id()
            )));
        }
        Ok(text.into_owned())
    }

    /// A catalogue built from a resource written in a test, in the language
    /// the test says, through the same builder as the shipped one.
    #[cfg(test)]
    fn from_a_resource_written_in_a_test(locale: &str, source: &str) -> Result<Self> {
        a_bundle_speaking(locale, source).map(|bundle| Self { bundle })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every id the code can ask for that the source does not hold.
    fn missing_from(source: &str) -> Vec<&'static str> {
        let catalogue = Catalogue::from_a_resource_written_in_a_test("en-US", source)
            .expect("a resource that parses");
        Message::ALL
            .iter()
            .map(|message| message.id())
            .filter(|id| !catalogue.bundle.has_message(id))
            .collect()
    }

    /// Every message the source holds that the code cannot ask for.
    ///
    /// Read off the text rather than off `FluentResource::entries()`, and that
    /// is a decision about dependencies rather than about parsing: the entry
    /// type belongs to `fluent-syntax`, naming it means a fifth direct
    /// dependency, and the plan was four. A message in Fluent starts at column
    /// zero with a letter, then `=`; a term starts with `-`, a comment with
    /// `#`, an attribute and a continued line with whitespace. So a line with
    /// a letter at column zero is a message start, in a resource the parser
    /// has already accepted without error, which is asserted first.
    fn nobody_asks_for_in(source: &str) -> Vec<&str> {
        source
            .lines()
            .filter(|line| line.chars().next().is_some_and(|c| c.is_ascii_alphabetic()))
            .filter_map(|line| line.split_once('=').map(|(id, _)| id.trim()))
            .filter(|id| !Message::ALL.iter().any(|message| message.id() == *id))
            .collect()
    }

    fn english() -> &'static Catalogue {
        for_this(WhichLocale::NamedInATest("en-US")).expect("the shipped English catalogue")
    }

    #[test]
    fn test_the_english_catalogue_is_complete_in_both_directions() {
        let source = SHIPPED[0].dates;

        assert_eq!(missing_from(source), Vec::<&str>::new());
        assert_eq!(nobody_asks_for_in(source), Vec::<&str>::new());
    }

    /// The reading above has to be able to see a violation, in each
    /// direction, or a catalogue that drifted would pass forever.
    #[test]
    fn test_the_completeness_reading_sees_a_message_missing_and_a_message_nobody_asks_for() {
        let short = "dates-just-now = just now\n";
        assert_eq!(
            missing_from(short),
            vec![
                "dates-minutes-ago",
                "dates-hours-ago",
                "dates-days-ago",
                "dates-in-minutes",
                "dates-in-hours",
                "dates-in-days"
            ]
        );

        let orphaned = format!("{}\ndates-nobody-asks = orphaned\n", SHIPPED[0].dates);
        assert_eq!(nobody_asks_for_in(&orphaned), vec!["dates-nobody-asks"]);
    }

    #[test]
    fn test_the_four_english_sentences_are_the_ones_this_program_always_said() {
        let catalogue = english();

        assert_eq!(catalogue.say(Message::JustNow).unwrap(), "just now");
        assert_eq!(
            catalogue.say_how_many(Message::MinutesAgo, 1).unwrap(),
            "1 minute ago"
        );
        assert_eq!(
            catalogue.say_how_many(Message::MinutesAgo, 30).unwrap(),
            "30 minutes ago"
        );
        assert_eq!(
            catalogue.say_how_many(Message::HoursAgo, 1).unwrap(),
            "1 hour ago"
        );
        assert_eq!(
            catalogue.say_how_many(Message::HoursAgo, 12).unwrap(),
            "12 hours ago"
        );
        assert_eq!(
            catalogue.say_how_many(Message::DaysAgo, 1).unwrap(),
            "1 day ago"
        );
        assert_eq!(
            catalogue.say_how_many(Message::DaysAgo, 2).unwrap(),
            "2 days ago"
        );
    }

    /// The three in the other direction, for a thing that has not happened
    /// yet, on the same plural rule as the three behind.
    #[test]
    fn test_the_three_future_sentences_say_in() {
        let catalogue = english();

        assert_eq!(
            catalogue.say_how_many(Message::InMinutes, 1).unwrap(),
            "in 1 minute"
        );
        assert_eq!(
            catalogue.say_how_many(Message::InMinutes, 15).unwrap(),
            "in 15 minutes"
        );
        assert_eq!(
            catalogue.say_how_many(Message::InHours, 1).unwrap(),
            "in 1 hour"
        );
        assert_eq!(
            catalogue.say_how_many(Message::InHours, 2).unwrap(),
            "in 2 hours"
        );
        assert_eq!(
            catalogue.say_how_many(Message::InDays, 1).unwrap(),
            "in 1 day"
        );
        assert_eq!(
            catalogue.say_how_many(Message::InDays, 3).unwrap(),
            "in 3 days"
        );
    }

    /// Condition 1: no isolation marks. Checked by code point over every
    /// message, because an exact-string assertion that fails on them says
    /// "expected 2 days ago, got 2 days ago" and leaves somebody staring.
    #[test]
    fn test_no_sentence_out_of_the_catalogue_carries_an_isolation_mark() {
        let catalogue = english();
        for message in Message::ALL {
            let said = match message.counts() {
                None => catalogue.say(message),
                Some(_) => catalogue.say_how_many(message, 2),
            }
            .unwrap();
            assert!(
                !said.contains(['\u{2068}', '\u{2069}']),
                "{} carries an isolation mark: {:?}",
                message.id(),
                said.chars().map(|c| c as u32).collect::<Vec<_>>()
            );
        }
    }

    /// Condition 3: counts as numbers. A count passed as a string matches no
    /// plural category, falls through to `*[other]` with no error, and only
    /// the singular can see it.
    #[test]
    fn test_a_count_of_one_takes_the_singular_form() {
        let catalogue = english();

        assert_eq!(
            catalogue.say_how_many(Message::MinutesAgo, 1).unwrap(),
            "1 minute ago"
        );
        assert_eq!(
            catalogue.say_how_many(Message::HoursAgo, 1).unwrap(),
            "1 hour ago"
        );
        assert_eq!(
            catalogue.say_how_many(Message::DaysAgo, 1).unwrap(),
            "1 day ago"
        );
    }

    /// Condition 3, the other half: the error list is read. Each message
    /// formats with its argument and no error, and a message asked for
    /// without its argument is an error rather than a sentence with a brace
    /// in it.
    #[test]
    fn test_every_message_formats_with_its_argument_and_no_error() {
        let catalogue = english();
        for message in Message::ALL {
            let said = match message.counts() {
                None => catalogue.say(message),
                Some(_) => catalogue.say_how_many(message, 3),
            };
            assert!(said.is_ok(), "{}: {said:?}", message.id());
        }
    }

    #[test]
    fn test_a_missing_argument_is_an_error_and_not_a_sentence_with_a_brace_in_it() {
        let said = english().say(Message::MinutesAgo);

        assert!(said.is_err(), "{said:?}");
    }

    /// Condition 2: numbers written by Windows. Under sixty the formatter and
    /// the plain digits agree in every locale, so this uses 1234, where
    /// German and English part.
    ///
    /// Windows only: where there is no Windows the formatter hands every
    /// number back and Fluent writes the digits, by design.
    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_number_inside_a_sentence_is_written_the_way_the_language_writes_it() {
        let source =
            "dates-days-ago = { $days ->\n    [one] { $days } Tag\n   *[other] { $days } Tage\n}\n";
        let german = Catalogue::from_a_resource_written_in_a_test("de-DE", source).unwrap();
        assert_eq!(
            german.say_how_many(Message::DaysAgo, 1234).unwrap(),
            "1.234 Tage"
        );

        assert_eq!(
            english().say_how_many(Message::DaysAgo, 1234).unwrap(),
            "1,234 days ago"
        );
    }

    /// The whole reason for the catalogue: a language with four forms gets
    /// four forms, on a machine that speaks none of them. The resource is
    /// written here and ships nowhere; nobody who wrote it reads Russian, and
    /// a translation nobody can read is not a translation to ship.
    #[test]
    fn test_a_russian_resource_produces_the_four_russian_forms() {
        let source = "dates-days-ago = { $days ->\n    [one] { $days } день назад\n    [few] { $days } дня назад\n    [many] { $days } дней назад\n   *[other] { $days } дня назад\n}\n";
        let russian = Catalogue::from_a_resource_written_in_a_test("ru-RU", source).unwrap();

        assert_eq!(
            russian.say_how_many(Message::DaysAgo, 1).unwrap(),
            "1 день назад"
        );
        assert_eq!(
            russian.say_how_many(Message::DaysAgo, 2).unwrap(),
            "2 дня назад"
        );
        assert_eq!(
            russian.say_how_many(Message::DaysAgo, 5).unwrap(),
            "5 дней назад"
        );
        assert_eq!(
            russian.say_how_many(Message::DaysAgo, 21).unwrap(),
            "21 день назад"
        );
    }

    #[test]
    fn test_a_polish_resource_produces_the_polish_forms() {
        let source = "dates-days-ago = { $days ->\n    [one] { $days } dzień temu\n    [few] { $days } dni temu\n    [many] { $days } dni temu\n   *[other] { $days } dnia temu\n}\n";
        let polish = Catalogue::from_a_resource_written_in_a_test("pl-PL", source).unwrap();

        assert_eq!(
            polish.say_how_many(Message::DaysAgo, 1).unwrap(),
            "1 dzień temu"
        );
        assert_eq!(
            polish.say_how_many(Message::DaysAgo, 2).unwrap(),
            "2 dni temu"
        );
        assert_eq!(
            polish.say_how_many(Message::DaysAgo, 5).unwrap(),
            "5 dni temu"
        );
        assert_eq!(
            polish.say_how_many(Message::DaysAgo, 22).unwrap(),
            "22 dni temu"
        );
    }

    /// The fallback the error list cannot see. A bundle whose locale nobody
    /// has plural rules for selects English rules with zero errors, because
    /// `fluent-bundle` negotiates its plural rules with `en` as the default.
    /// That default is a line in somebody else's crate, and a later version
    /// could move it, so it is held here.
    #[test]
    fn test_a_locale_nobody_has_rules_for_takes_english_plurals_silently() {
        let unknown =
            Catalogue::from_a_resource_written_in_a_test("xx-YY", SHIPPED[0].dates).unwrap();

        assert_eq!(
            unknown.say_how_many(Message::DaysAgo, 1).unwrap(),
            "1 day ago"
        );
        assert_eq!(
            unknown.say_how_many(Message::DaysAgo, 2).unwrap(),
            "2 days ago"
        );
    }

    /// Which catalogue a machine gets: the English one, whatever it asks for,
    /// while English is the only one compiled in.
    #[test]
    fn test_a_machine_with_no_catalogue_in_its_language_gets_english() {
        for name in ["fr-FR", "ru-RU", "en-GB", "xx-YY"] {
            let catalogue = for_this(WhichLocale::NamedInATest(name)).expect(name);
            assert_eq!(
                catalogue.say_how_many(Message::DaysAgo, 2).unwrap(),
                "2 days ago",
                "for {name}"
            );
        }
    }

    #[test]
    fn test_a_locale_name_that_is_not_one_asks_for_english() {
        let english: LanguageIdentifier = "en-US".parse().unwrap();

        assert_eq!(requested_locale(Ok("not a locale".to_string())), english);
    }

    #[test]
    fn test_a_locale_this_computer_could_not_name_asks_for_english() {
        let english: LanguageIdentifier = "en-US".parse().unwrap();

        assert_eq!(
            requested_locale(Err(Error::Other("Windows would not say".to_string()))),
            english
        );
    }
}

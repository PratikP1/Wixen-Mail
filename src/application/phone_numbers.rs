//! A phone number read against its own country's numbering plan (#40).
//!
//! Pratik answered on 2026-09-23 that a number is checked and formatted per
//! country through a phone-number library, because internationalisation is
//! planned and anything chosen now is the first piece of version 2's
//! system. This module is the reading, and it is the one file that names
//! the library, so nothing else in the tree grows uses of it.
//!
//! The library is `phonenumber` 0.3.10, a port of Google's libphonenumber,
//! confirmed by Pratik on 2026-09-24 with "Use phonenumber. See if you can
//! get around the bug." Four defects were measured that day against a second
//! port, `rlibphonenumber` 2.2.12, over 102 numbers; its plain calls agreed on
//! 46 and the reading below on 101 (12-07's plan, "The defects, measured").
//! Every number goes through all four routes, and each can come out, with
//! its cases kept, when upstream ships the fix:
//!
//! 1. Digits in any script become ASCII digits first, because the library's
//!    parser matches ASCII digits only. Windows does the folding.
//! 2. The chosen country is never the reference for a number that carries
//!    its own code. The library strips the chosen country's trunk digits from
//!    such a number (upstream pull request 110), so a number whose code came
//!    from a `+` is read again with no country, and one whose code came after
//!    the chosen country's international prefix has that prefix turned into
//!    a `+` first.
//! 3. When the library took the chosen country's own code out of a national
//!    number's digits (upstream issue 68), the whole number is read as well
//!    and kept when it is the valid one, which is Google's own preference.
//! 4. The region is read here with the significant number's leading zeros
//!    kept, because the library's own lookup drops them and loses every
//!    Italian number's region. This module never asks the library for a
//!    number's region or type.
//!
//! A number the library doubts is never refused. Its numbering data is
//! Google's of 2026-06-17 and trails the world's, so the editor stops a
//! doubted number once and keeps it exactly as typed on a second OK. Only
//! text with no digit in it is refused.

use crate::service::this_machine;
use phonenumber::country::{Id, Source};
use phonenumber::metadata::DATABASE;
use phonenumber::{Metadata, Mode, ParseError, PhoneNumber, Type};
use std::str::FromStr;

/// A region, as the two-letter ISO 3166 code a stored number's country is
/// kept as. Only a region the numbering data knows can be made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Region(&'static str);

impl Region {
    /// The region a two-letter code names, when the numbering data knows it.
    pub fn from_code(code: &str) -> Option<Region> {
        let is_two_capitals = code.len() == 2 && code.chars().all(|c| c.is_ascii_uppercase());
        if !is_two_capitals || Id::from_str(code).is_err() {
            return None;
        }
        DATABASE.by_id(code).map(|metadata| Region(metadata.id()))
    }

    /// The two-letter code, such as "GB".
    pub fn as_str(&self) -> &'static str {
        self.0
    }

    fn id(self) -> Option<Id> {
        Id::from_str(self.0).ok()
    }

    fn metadata(self) -> Option<&'static Metadata> {
        DATABASE.by_id(self.0)
    }
}

/// Why a number is doubted. A doubt stops a save once and never refuses: the
/// numbering data trails the world's, so a real number can be doubted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Doubt {
    /// No country was chosen and the number carries no country code.
    NoCountry,
    /// The number begins with a country code no country uses.
    UnknownCallingCode,
    /// Fewer digits than any number in its country has.
    TooShort,
    /// More digits than any number in its country has.
    TooLong,
    /// The right length and still not a number its country's plan holds.
    NotInItsPlan,
    /// The text could not be read as a number at all.
    Unreadable,
}

/// What a typed number was read as.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reading {
    /// A number its country's plan holds, in the form it is saved in.
    Valid {
        stored: String,
        region: Option<Region>,
    },
    /// A number with something wrong with it, and the region it was judged
    /// against when there was one.
    Doubtful {
        doubt: Doubt,
        region: Option<Region>,
    },
    /// Text with no digit in it, which cannot be a phone number.
    NoDigit,
}

/// Reads what somebody typed as a phone number, against the country chosen
/// beside it when the number does not carry its own code.
pub fn read(typed: &str, chosen: Option<Region>) -> Reading {
    if !typed.chars().any(char::is_numeric) {
        return Reading::NoDigit;
    }
    let text = this_machine::fold_digits(typed);
    match parse_around_the_defects(&text, chosen) {
        Ok(number) => judge(&number, &text),
        Err(refused) => {
            let doubt = doubt_for_a_refusal(&refused, &text, chosen);
            let region = match doubt {
                Doubt::NoCountry | Doubt::UnknownCallingCode => None,
                _ => chosen,
            };
            Reading::Doubtful { doubt, region }
        }
    }
}

/// Every region the numbering data knows, with its calling code, in the
/// order of their codes. The country list beside a number is built from
/// this, so no table of countries lives in the tree.
pub fn every_region() -> Vec<(Region, u16)> {
    let mut regions: Vec<(Region, u16)> = DATABASE
        .iter()
        .filter_map(|metadata| {
            Region::from_code(metadata.id()).map(|region| (region, metadata.country_code()))
        })
        .collect();
    regions.sort_unstable();
    regions
}

/// The sentence said for a doubted number: the number, what is wrong with
/// it, and how to keep it anyway. `country` is the name of the country it
/// was judged against, in the language the person reads.
///
/// Composed here and nowhere else, which is the seam version 2's catalogue
/// replaces. The library's own messages are English and never shown.
pub fn sentence(typed: &str, doubt: Doubt, country: Option<&str>) -> String {
    let country = country.unwrap_or("its country");
    let what = match doubt {
        Doubt::NoCountry => format!(
            "{typed} has no country code and no country is chosen, so it cannot be checked. \
             Choose its country, or type it with + and its country code."
        ),
        Doubt::UnknownCallingCode => {
            format!("{typed} begins with a country code that no country uses.")
        }
        Doubt::TooShort => format!("{typed} is too short for a phone number in {country}."),
        Doubt::TooLong => format!("{typed} is too long for a phone number in {country}."),
        Doubt::NotInItsPlan => format!(
            "{typed} is not a phone number used in {country}. \
             A number from another country is typed with + and its country code."
        ),
        Doubt::Unreadable => format!("{typed} could not be read as a phone number."),
    };
    format!("{what} Press OK again to keep it exactly as typed.")
}

/// The one refusal: text with no digit in it.
pub fn no_digit_sentence(typed: &str) -> String {
    format!("{typed} has no digits in it, so it cannot be a phone number.")
}

/// The library's reading, routed around defects 2 and 3 of the module
/// comment.
fn parse_around_the_defects(text: &str, chosen: Option<Region>) -> Result<PhoneNumber, ParseError> {
    let Some(chosen) = chosen.and_then(Region::id) else {
        return phonenumber::parse(None, text);
    };
    match phonenumber::parse(Some(chosen), text) {
        Ok(number) => match number.code().source() {
            Source::Plus => phonenumber::parse(None, text),
            Source::Idd => match after_the_international_prefix(text, chosen) {
                Some(with_a_plus) => phonenumber::parse(None, with_a_plus),
                None => Ok(number),
            },
            Source::Number => Ok(the_whole_number_when_it_is_valid(text, number)),
            Source::Default => Ok(number),
        },
        Err(refused) if begins_with_a_plus(text) => {
            phonenumber::parse(None, text).map_err(|_| refused)
        }
        Err(refused) => Err(refused),
    }
}

/// The number after the chosen country's international prefix, with a `+`
/// in the prefix's place, when the number begins with that prefix.
fn after_the_international_prefix(text: &str, chosen: Id) -> Option<String> {
    let prefix = DATABASE.by_id(chosen.as_ref())?.international_prefix()?;
    let digits: String = text.chars().filter(char::is_ascii_digit).collect();
    let found = prefix.find(&digits).filter(|found| found.start() == 0)?;
    let (at, _) = text
        .char_indices()
        .filter(|(_, letter)| letter.is_ascii_digit())
        .nth(found.end())?;
    Some(format!("+{}", &text[at..]))
}

/// When the library took the chosen country's code out of the digits, the
/// digits read whole as a national number of that country, if that is the
/// valid reading.
fn the_whole_number_when_it_is_valid(text: &str, stripped: PhoneNumber) -> PhoneNumber {
    match phonenumber::parse(None, format!("+{} {text}", stripped.code().value())) {
        Ok(whole) if phonenumber::is_valid(&whole) => whole,
        _ => stripped,
    }
}

fn begins_with_a_plus(text: &str) -> bool {
    text.trim_start().starts_with(['+', '\u{FF0B}'])
}

fn judge(number: &PhoneNumber, text: &str) -> Reading {
    let region = region_of(number);
    if phonenumber::is_valid(number) {
        return Reading::Valid {
            stored: number.format().mode(Mode::International).to_string(),
            region,
        };
    }
    let judged_by = region.or_else(|| main_region(number.code().value()));
    Reading::Doubtful {
        doubt: why_it_is_not_valid(number, text, judged_by),
        region: judged_by,
    }
}

/// A foreign number typed with its code and no `+` first, because that one
/// has an answer the person can act on; then the length against the
/// lengths its country's numbers have.
fn why_it_is_not_valid(number: &PhoneNumber, text: &str, judged_by: Option<Region>) -> Doubt {
    let typed_its_code_without_a_plus = number.code().source() == Source::Default
        && phonenumber::parse(None, format!("+{}", text.trim()))
            .is_ok_and(|with_a_plus| phonenumber::is_valid(&with_a_plus));
    if typed_its_code_without_a_plus {
        return Doubt::NotInItsPlan;
    }
    let length = number.national().to_string().len();
    let lengths = judged_by
        .and_then(Region::metadata)
        .map(possible_lengths)
        .unwrap_or_default();
    match (lengths.iter().min(), lengths.iter().max()) {
        (Some(&shortest), _) if length < usize::from(shortest) => Doubt::TooShort,
        (_, Some(&longest)) if length > usize::from(longest) => Doubt::TooLong,
        _ => Doubt::NotInItsPlan,
    }
}

/// The lengths a region's numbers have. The library leaves the general
/// description's list empty (measured on 2026-09-24: GB's is `[]`), and
/// Google defines it as every kind's lengths together, so they are gathered
/// here.
fn possible_lengths(metadata: &Metadata) -> Vec<u16> {
    let descriptors = metadata.descriptors();
    std::iter::once(descriptors.general())
        .chain(KINDS.iter().filter_map(|kind| descriptors.get(*kind)))
        .flat_map(|descriptor| descriptor.possible_length().iter().copied())
        .collect()
}

fn doubt_for_a_refusal(refused: &ParseError, text: &str, chosen: Option<Region>) -> Doubt {
    match refused {
        ParseError::InvalidCountryCode if chosen.is_none() && !begins_with_a_plus(text) => {
            Doubt::NoCountry
        }
        ParseError::InvalidCountryCode => Doubt::UnknownCallingCode,
        ParseError::TooShortAfterIdd | ParseError::TooShortNsn => Doubt::TooShort,
        ParseError::TooLong => Doubt::TooLong,
        ParseError::NoNumber | ParseError::MalformedInteger(_) => Doubt::Unreadable,
    }
}

/// The kinds of number a region's plan describes, any of which makes the
/// significant number one of that region's.
const KINDS: [Type; 10] = [
    Type::FixedLine,
    Type::Mobile,
    Type::TollFree,
    Type::PremiumRate,
    Type::SharedCost,
    Type::Voip,
    Type::PersonalNumber,
    Type::Pager,
    Type::Uan,
    Type::Voicemail,
];

/// The region a number belongs to, read with its significant number's
/// leading zeros kept (defect 4). Among the regions sharing a code, a
/// region's leading digits decide where it has them, and otherwise a kind of
/// number its plan holds, in the order the library and Google both use.
fn region_of(number: &PhoneNumber) -> Option<Region> {
    let significant = number.national().to_string();
    let regions = DATABASE.region(&number.code().value())?;
    let shared = regions.len() > 1;
    regions
        .into_iter()
        .filter_map(Region::from_code)
        .find(|region| {
            let Some(metadata) = region.metadata() else {
                return false;
            };
            match metadata.leading_digits() {
                Some(leading) if shared => leading
                    .find(&significant)
                    .is_some_and(|found| found.start() == 0),
                _ => !shared || is_a_kind_its_plan_holds(metadata, &significant),
            }
        })
}

fn is_a_kind_its_plan_holds(metadata: &Metadata, significant: &str) -> bool {
    let descriptors = metadata.descriptors();
    descriptors.general().is_match(significant)
        && KINDS.iter().any(|kind| {
            descriptors
                .get(*kind)
                .is_some_and(|descriptor| descriptor.is_match(significant))
        })
}

/// The region a calling code belongs to first, such as GB for 44.
fn main_region(code: u16) -> Option<Region> {
    let sharing = DATABASE.by_code(&code)?;
    sharing
        .iter()
        .find(|metadata| metadata.is_main_country_for_code())
        .or_else(|| sharing.first())
        .and_then(|metadata| Region::from_code(metadata.id()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn region(code: &str) -> Option<Region> {
        Region::from_code(code)
    }

    /// Reads the number and requires it valid, in that region, saved in
    /// that form. The rows are the audit's of 2026-09-24, where the
    /// workaround and a second port of Google's library agreed.
    fn reads_valid(typed: &str, chosen: &str, expected_region: &str, stored: &str) {
        let chosen = if chosen.is_empty() {
            None
        } else {
            region(chosen)
        };
        assert_eq!(
            read(typed, chosen),
            Reading::Valid {
                stored: stored.to_string(),
                region: region(expected_region),
            },
            "{typed:?} with {chosen:?}"
        );
    }

    fn reads_doubtful(typed: &str, chosen: &str, doubt: Doubt, judged_by: &str) {
        let chosen = if chosen.is_empty() {
            None
        } else {
            region(chosen)
        };
        assert_eq!(
            read(typed, chosen),
            Reading::Doubtful {
                doubt,
                region: region(judged_by),
            },
            "{typed:?} with {chosen:?}"
        );
    }

    #[test]
    fn test_row_1_a_british_number_typed_nationally() {
        reads_valid("0121 234 5678", "GB", "GB", "+44 121 234 5678");
    }

    #[test]
    fn test_row_2_an_american_number_typed_nationally() {
        reads_valid("(201) 555-0123", "US", "US", "+1 201-555-0123");
    }

    #[test]
    fn test_row_8_an_italian_number_keeps_its_leading_zero_and_its_region() {
        reads_valid("06 3551 1397", "IT", "IT", "+39 06 3551 1397");
    }

    #[test]
    fn test_row_9_a_russian_number_after_its_trunk_eight() {
        reads_valid("8 301 123 45 67", "RU", "RU", "+7 301 123-45-67");
    }

    #[test]
    fn test_row_10_a_hungarian_number_after_its_trunk_prefix() {
        reads_valid("06 1 234 5678", "HU", "HU", "+36 1 234 5678");
    }

    #[test]
    fn test_row_12_a_british_number_too_short() {
        reads_doubtful("0121 234", "GB", Doubt::TooShort, "GB");
    }

    #[test]
    fn test_row_13_a_british_number_too_long() {
        reads_doubtful("0121 234 5678 9999", "GB", Doubt::TooLong, "GB");
    }

    #[test]
    fn test_row_14_an_italian_number_with_its_code_and_britain_chosen() {
        reads_valid("+39 06 3551 1397", "GB", "IT", "+39 06 3551 1397");
    }

    #[test]
    fn test_row_22_an_italian_number_with_its_code_and_hungary_chosen() {
        reads_valid("+39 06 3551 1397", "HU", "IT", "+39 06 3551 1397");
    }

    #[test]
    fn test_row_24_a_vatican_number_is_found_by_its_leading_digits() {
        reads_valid("+39 06 6981 2345", "GB", "VA", "+39 06 6981 2345");
    }

    #[test]
    fn test_row_25_a_british_number_with_its_code_and_the_us_chosen() {
        reads_valid("+44 121 234 5678", "US", "GB", "+44 121 234 5678");
    }

    #[test]
    fn test_row_28_a_jersey_number_shares_britains_code() {
        reads_valid("+44 1534 456789", "US", "JE", "+44 1534 456789");
    }

    #[test]
    fn test_row_30_a_german_mobile_is_not_made_into_another_german_number() {
        reads_valid("+49 1512 3456789", "US", "DE", "+49 1512 3456789");
    }

    #[test]
    fn test_row_31_a_japanese_free_number_with_the_us_chosen() {
        reads_valid("+81 120 123 456", "US", "JP", "+81 120-123-456");
    }

    #[test]
    fn test_row_37_a_canadian_number_shares_the_us_code() {
        reads_valid("+1 506 234 5678", "GB", "CA", "+1 506-234-5678");
    }

    #[test]
    fn test_row_47_a_trunk_zero_in_brackets_after_the_code() {
        reads_valid("+44 (0) 121 234 5678", "US", "GB", "+44 121 234 5678");
    }

    #[test]
    fn test_row_49_britains_international_prefix_before_an_italian_number() {
        reads_valid("00 39 06 3551 1397", "GB", "IT", "+39 06 3551 1397");
    }

    #[test]
    fn test_row_53_the_us_international_prefix_before_a_british_number() {
        reads_valid("011 44 121 234 5678", "US", "GB", "+44 121 234 5678");
    }

    #[test]
    fn test_row_55_australias_international_prefix() {
        reads_valid("0011 44 121 234 5678", "AU", "GB", "+44 121 234 5678");
    }

    #[test]
    fn test_row_56_japans_international_prefix() {
        reads_valid("010 39 06 3551 1397", "JP", "IT", "+39 06 3551 1397");
    }

    #[test]
    fn test_row_57_russias_international_prefix() {
        reads_valid("810 39 06 3551 1397", "RU", "IT", "+39 06 3551 1397");
    }

    #[test]
    fn test_row_58_new_zealands_longer_international_prefix() {
        reads_valid("0161 39 06 3551 1397", "NZ", "IT", "+39 06 3551 1397");
    }

    #[test]
    fn test_row_62_the_chosen_countrys_own_code_without_a_plus() {
        reads_valid("44 121 234 5678", "GB", "GB", "+44 121 234 5678");
    }

    #[test]
    fn test_row_64_a_foreign_code_without_a_plus_is_doubted_not_guessed() {
        reads_doubtful("39 06 3551 1397", "GB", Doubt::NotInItsPlan, "GB");
    }

    #[test]
    fn test_row_66_a_tel_address_with_a_plus() {
        reads_valid("tel:+39-06-3551-1397", "GB", "IT", "+39 06 3551 1397");
    }

    #[test]
    fn test_row_68_a_tel_address_with_its_country_as_context() {
        reads_valid(
            "tel:06-3551-1397;phone-context=+39",
            "GB",
            "IT",
            "+39 06 3551 1397",
        );
    }

    #[test]
    fn test_row_71_an_extension_is_kept_in_its_countrys_style() {
        reads_valid(
            "+44 121 234 5678 ext. 12",
            "US",
            "GB",
            "+44 121 234 5678 x12",
        );
    }

    #[test]
    fn test_row_74_an_extension_after_an_international_prefix() {
        reads_valid(
            "00 39 06 3551 1397 ext 5",
            "GB",
            "IT",
            "+39 06 3551 1397 ext. 5",
        );
    }

    #[test]
    fn test_row_75_letters_on_the_keypad_become_their_digits() {
        reads_valid("1-800-FLOWERS", "US", "US", "+1 800-356-9377");
    }

    #[test]
    fn test_row_79_a_national_number_with_no_country_chosen() {
        reads_doubtful("0121 234 5678", "", Doubt::NoCountry, "");
    }

    #[test]
    fn test_row_80_an_international_prefix_with_no_country_chosen() {
        reads_doubtful("00 39 06 3551 1397", "", Doubt::NoCountry, "");
    }

    #[test]
    fn test_row_83_an_ivorian_number_keeps_its_leading_zero() {
        reads_valid("+225 01 23 45 67 89", "US", "CI", "+225 01 23 45 6789");
    }

    #[test]
    fn test_row_87_a_san_marino_number_with_italy_chosen() {
        reads_valid("+378 0549 886377", "IT", "SM", "+378 0549 886377");
    }

    #[test]
    fn test_row_90_an_italian_mobile_that_begins_with_italys_code() {
        reads_valid("391 231 2312", "IT", "IT", "+39 391 231 2312");
    }

    #[test]
    fn test_row_93_a_german_number_that_begins_with_germanys_code() {
        reads_valid("4921 123456", "DE", "DE", "+49 4921 123456");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_row_100_full_width_digits() {
        reads_valid(
            "\u{FF10}\u{FF11}\u{FF12}\u{FF11} 234 5678",
            "GB",
            "GB",
            "+44 121 234 5678",
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_row_101_arabic_indic_digits() {
        reads_valid(
            "\u{0660}\u{0661}\u{0662}\u{0661} \u{0662}\u{0663}\u{0664} \u{0665}\u{0666}\u{0667}\u{0668}",
            "GB",
            "GB",
            "+44 121 234 5678",
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_row_102_a_full_width_international_prefix() {
        reads_valid(
            "\u{FF10}\u{FF10} 39 06 3551 1397",
            "GB",
            "IT",
            "+39 06 3551 1397",
        );
    }

    #[test]
    fn test_text_with_no_digit_is_not_a_number() {
        assert_eq!(read("call reception", region("GB")), Reading::NoDigit);
        assert_eq!(read("", None), Reading::NoDigit);
    }

    #[test]
    fn test_a_code_no_country_uses_is_named_as_that() {
        reads_doubtful("+999 1234", "GB", Doubt::UnknownCallingCode, "");
        reads_doubtful("+999 1234", "", Doubt::UnknownCallingCode, "");
    }

    #[test]
    fn test_the_published_crafted_inputs_holding_a_digit_are_read_without_a_panic() {
        // RUSTSEC-2023-0082 and RUSTSEC-2024-0369 panicked on inputs like
        // these. The crate carries the fixes; this holds the reading to
        // answering rather than taking the interface thread down.
        for crafted in [
            "tel:0;phone-context=+\u{10000}",
            "+1;phone-context=\u{e9}\u{e9}\u{e9}",
        ] {
            let _ = read(crafted, region("GB"));
            let _ = read(crafted, None);
        }
    }

    #[test]
    fn test_every_region_is_listed_once_with_its_calling_code() {
        let regions = every_region();

        assert!(
            regions.contains(&(Region::from_code("GB").expect("GB"), 44)),
            "{} regions",
            regions.len()
        );
        assert!(regions.contains(&(Region::from_code("JE").expect("JE"), 44)));
        let mut codes: Vec<&str> = regions.iter().map(|(r, _)| r.as_str()).collect();
        let listed = codes.len();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), listed, "a region listed twice");
        assert!(listed > 200, "{listed}");
    }

    #[test]
    fn test_a_code_the_numbering_data_does_not_know_makes_no_region() {
        assert_eq!(Region::from_code("ZZ"), None);
        assert_eq!(Region::from_code("gb"), None);
        assert_eq!(Region::from_code("GBR"), None);
        assert_eq!(
            Region::from_code("GB").map(|r| r.as_str().to_string()),
            Some("GB".into())
        );
    }

    #[test]
    fn test_each_doubt_names_the_number_and_how_to_keep_it() {
        for doubt in [
            Doubt::NoCountry,
            Doubt::UnknownCallingCode,
            Doubt::TooShort,
            Doubt::TooLong,
            Doubt::NotInItsPlan,
            Doubt::Unreadable,
        ] {
            let said = sentence("0121 234", doubt, Some("United Kingdom"));
            assert!(said.contains("0121 234"), "{doubt:?}: {said}");
            assert!(
                said.ends_with("Press OK again to keep it exactly as typed."),
                "{doubt:?}: {said}"
            );
        }
    }

    #[test]
    fn test_the_doubts_are_told_apart_and_name_the_country() {
        let short = sentence("0121 234", Doubt::TooShort, Some("United Kingdom"));
        let long = sentence("0121 234", Doubt::TooLong, Some("United Kingdom"));
        let foreign = sentence(
            "39 06 3551 1397",
            Doubt::NotInItsPlan,
            Some("United Kingdom"),
        );
        let unnamed = sentence("0121 234", Doubt::TooShort, None);

        assert!(
            short.contains("too short") && short.contains("United Kingdom"),
            "{short}"
        );
        assert!(long.contains("too long"), "{long}");
        assert!(foreign.contains("+ and its country code"), "{foreign}");
        assert!(!unnamed.contains("None"), "{unnamed}");
        assert_ne!(short, long);
    }

    #[test]
    fn test_text_with_no_digit_is_refused_in_a_sentence_naming_it() {
        assert_eq!(
            no_digit_sentence("call reception"),
            "call reception has no digits in it, so it cannot be a phone number."
        );
    }
}

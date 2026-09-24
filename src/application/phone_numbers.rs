//! A phone number read against its own country's numbering plan (#40).
//!
//! Pratik answered on 2026-09-23 that a number is checked and formatted per
//! country through a phone-number library, because internationalisation is
//! planned and anything chosen now is the first piece of version 2's
//! system. This module is the reading, and it is the one file that names
//! the library, so nothing else in the tree grows uses of it.

/// A region, as the two-letter ISO 3166 code a stored number's country is
/// kept as. Only a region the numbering data knows can be made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Region([u8; 2]);

impl Region {
    /// The region a two-letter code names.
    pub fn from_code(code: &str) -> Option<Region> {
        match code.as_bytes() {
            [first, second] if first.is_ascii_uppercase() && second.is_ascii_uppercase() => {
                Some(Region([*first, *second]))
            }
            _ => None,
        }
    }

    /// The two-letter code, such as "GB".
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap_or("??")
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

pub fn read(_typed: &str, _chosen: Option<Region>) -> Reading {
    Reading::Doubtful {
        doubt: Doubt::Unreadable,
        region: None,
    }
}

pub fn every_region() -> Vec<(Region, u16)> {
    Vec::new()
}

pub fn sentence(_typed: &str, _doubt: Doubt, _country: Option<&str>) -> String {
    String::new()
}

pub fn no_digit_sentence(_typed: &str) -> String {
    String::new()
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

//! A whole name and its five parts, each guessed from the other (#40).
//!
//! The contact editor asks for a name and for its parts: a prefix, a given
//! name, a middle name, a family name and a suffix. A person who types the
//! whole name gets the parts filled in as a first guess, and one who fills
//! in the parts gets the whole name composed. The editor decides which
//! fields may be written; this module only guesses.
//!
//! The guess is the editor's and nobody else's. The sync stopped splitting
//! names on 2026-08 because a split at the last space sent "Grace Brewster
//! Murray Hopper" to Google with the given name "Grace Brewster Murray" and
//! brought "van der Berg" back as "Berg" (`contacts_sync.rs`). A guess a
//! person can see and correct before saving is a different thing from one
//! made silently on the way to a server, and those two names are the first
//! cases below.

/// The five parts of a person's name, each absent until somebody gives it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NameParts {
    pub prefix: Option<String>,
    pub given: Option<String>,
    pub middle: Option<String>,
    pub family: Option<String>,
    pub suffix: Option<String>,
}

/// The titles the Prefix field offers, and the words a whole name is
/// recognised as starting with.
pub const TITLES: [&str; 18] = [
    "Mr", "Mrs", "Ms", "Mx", "Dr", "Prof", "Sir", "Dame", "Rev", "Mr.", "Mrs.", "Ms.", "Mx.",
    "Dr.", "Prof.", "Rev.", "Lord", "Lady",
];

/// The suffixes the Suffix field offers, and the words a whole name is
/// recognised as ending with.
pub const SUFFIXES: [&str; 14] = [
    "Jr", "Sr", "II", "III", "IV", "PhD", "MD", "Esq", "Jr.", "Sr.", "Ph.D.", "M.D.", "Esq.", "V",
];

pub fn guess_parts(_full: &str) -> NameParts {
    NameParts::default()
}

pub fn compose(_parts: &NameParts) -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn part(text: &str) -> Option<String> {
        Some(text.to_string())
    }

    #[test]
    fn test_hopper_keeps_both_middle_names_and_her_family_name() {
        let parts = guess_parts("Grace Brewster Murray Hopper");

        assert_eq!(
            parts,
            NameParts {
                given: part("Grace"),
                middle: part("Brewster Murray"),
                family: part("Hopper"),
                ..NameParts::default()
            }
        );
    }

    #[test]
    fn test_van_der_berg_stays_one_family_name() {
        let parts = guess_parts("Anna van der Berg");

        assert_eq!(
            parts,
            NameParts {
                given: part("Anna"),
                family: part("van der Berg"),
                ..NameParts::default()
            }
        );
    }

    #[test]
    fn test_a_title_and_a_degree_become_the_prefix_and_the_suffix() {
        let parts = guess_parts("Dr. Jane Q. Public, PhD");

        assert_eq!(
            parts,
            NameParts {
                prefix: part("Dr."),
                given: part("Jane"),
                middle: part("Q."),
                family: part("Public"),
                suffix: part("PhD"),
            }
        );
    }

    #[test]
    fn test_a_suffix_after_a_comma_is_still_the_suffix() {
        let parts = guess_parts("Martin Luther King, Jr.");

        assert_eq!(
            parts,
            NameParts {
                given: part("Martin"),
                middle: part("Luther"),
                family: part("King"),
                suffix: part("Jr."),
                ..NameParts::default()
            }
        );
    }

    #[test]
    fn test_one_word_is_the_given_name_alone() {
        assert_eq!(
            guess_parts("Cher"),
            NameParts {
                given: part("Cher"),
                ..NameParts::default()
            }
        );
    }

    #[test]
    fn test_an_empty_name_gives_no_part() {
        assert_eq!(guess_parts("   "), NameParts::default());
    }

    #[test]
    fn test_a_lower_case_word_before_the_last_joins_the_family_name() {
        // "dos" is on no list; being written in lower case is what joins it.
        let parts = guess_parts("Ana Maria dos Santos");

        assert_eq!(parts.given, part("Ana"));
        assert_eq!(parts.middle, part("Maria"));
        assert_eq!(parts.family, part("dos Santos"));
    }

    #[test]
    fn test_a_particle_written_with_a_capital_still_joins_the_family_name() {
        let parts = guess_parts("Juan Carlos De La Cruz");

        assert_eq!(parts.given, part("Juan"));
        assert_eq!(parts.middle, part("Carlos"));
        assert_eq!(parts.family, part("De La Cruz"));
    }

    #[test]
    fn test_a_title_is_recognised_without_its_full_stop_and_in_any_case() {
        let parts = guess_parts("mrs Anna Smith");

        assert_eq!(parts.prefix, part("mrs"));
        assert_eq!(parts.given, part("Anna"));
        assert_eq!(parts.family, part("Smith"));
    }

    #[test]
    fn test_spaces_around_and_between_words_are_not_parts() {
        let parts = guess_parts("  Grace    Hopper ");

        assert_eq!(parts.given, part("Grace"));
        assert_eq!(parts.middle, None);
        assert_eq!(parts.family, part("Hopper"));
    }

    #[test]
    fn test_composing_the_guess_gives_back_the_name_that_was_typed() {
        for typed in [
            "Grace Brewster Murray Hopper",
            "Anna van der Berg",
            "Dr. Jane Q. Public, PhD",
            "Martin Luther King Jr.",
            "Cher",
        ] {
            assert_eq!(compose(&guess_parts(typed)), typed);
        }
    }

    #[test]
    fn test_composing_leaves_out_a_part_nobody_gave() {
        let parts = NameParts {
            given: part("Anna"),
            family: part("Berg"),
            ..NameParts::default()
        };

        assert_eq!(compose(&parts), "Anna Berg");
        assert_eq!(compose(&NameParts::default()), "");
    }

    #[test]
    fn test_composing_puts_a_comma_before_a_degree_and_none_before_jr() {
        let degree = NameParts {
            given: part("Jane"),
            family: part("Public"),
            suffix: part("MD"),
            ..NameParts::default()
        };
        let generation = NameParts {
            given: part("Martin"),
            family: part("King"),
            suffix: part("Jr."),
            ..NameParts::default()
        };

        assert_eq!(compose(&degree), "Jane Public, MD");
        assert_eq!(compose(&generation), "Martin King Jr.");
    }
}

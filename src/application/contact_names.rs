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

/// The words that join the family name when they come before its last word,
/// in whatever case they are written: "Anna van der Berg", "De La Cruz".
/// A word written in lower case joins it too, on the list or not.
pub const PARTICLES: [&str; 26] = [
    "van", "der", "den", "ter", "ten", "de", "del", "della", "dei", "di", "da", "das", "dos", "do",
    "du", "des", "von", "zu", "la", "le", "bin", "ibn", "al", "el", "mac", "mc",
];

/// The suffixes a comma is written before when a name is composed, because
/// they are a qualification rather than part of the name.
const DEGREES: [&str; 6] = ["PhD", "Ph.D.", "MD", "M.D.", "Esq", "Esq."];

/// A first guess at the five parts of a whole name, for the person to
/// correct: a leading title is the prefix, a trailing suffix the suffix, the
/// first word left the given name, the last word with the particles before it
/// the family name, and anything between them the middle name.
pub fn guess_parts(full: &str) -> NameParts {
    let mut words: Vec<&str> = full.split_whitespace().collect();
    let suffix = take_last_if(&mut words, |word| is_one_of(&SUFFIXES, word));
    if let Some(last) = words.last_mut() {
        *last = last.trim_end_matches(',');
    }
    let prefix = match words.first() {
        Some(first) if words.len() > 1 && is_one_of(&TITLES, first) => Some(words.remove(0)),
        _ => None,
    };
    let given = (!words.is_empty()).then(|| words.remove(0));
    let family_starts = where_the_family_name_starts(&words);
    NameParts {
        prefix: prefix.map(str::to_string),
        given: given.map(str::to_string),
        middle: joined(&words[..family_starts]),
        family: joined(&words[family_starts..]),
        suffix: suffix.map(|word| word.trim_end_matches(',').to_string()),
    }
}

/// The whole name the parts make, with single spaces and a comma before a
/// suffix that is a degree.
pub fn compose(parts: &NameParts) -> String {
    let name = [&parts.prefix, &parts.given, &parts.middle, &parts.family]
        .into_iter()
        .filter_map(filled)
        .collect::<Vec<_>>()
        .join(" ");
    match filled(&parts.suffix) {
        None => name,
        Some(suffix) if name.is_empty() => suffix.to_string(),
        Some(suffix) if is_one_of(&DEGREES, suffix) => format!("{name}, {suffix}"),
        Some(suffix) => format!("{name} {suffix}"),
    }
}

fn is_one_of(list: &[&str], word: &str) -> bool {
    let word = word.trim_end_matches(',');
    list.iter().any(|entry| entry.eq_ignore_ascii_case(word))
}

/// Takes the last word off when there is more than one and it matches.
fn take_last_if<'a>(words: &mut Vec<&'a str>, matches: impl Fn(&str) -> bool) -> Option<&'a str> {
    match words.last() {
        Some(last) if words.len() > 1 && matches(last) => words.pop(),
        _ => None,
    }
}

/// Where the family name begins among the words after the given name: its
/// last word, and every particle or lower-case word right before it.
fn where_the_family_name_starts(words: &[&str]) -> usize {
    let Some(last) = words.len().checked_sub(1) else {
        return 0;
    };
    let joins = |word: &str| {
        is_one_of(&PARTICLES, word) || word.chars().next().is_some_and(char::is_lowercase)
    };
    last - words[..last]
        .iter()
        .rev()
        .take_while(|word| joins(word))
        .count()
}

fn joined(words: &[&str]) -> Option<String> {
    (!words.is_empty()).then(|| words.join(" "))
}

fn filled(part: &Option<String>) -> Option<&str> {
    part.as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
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

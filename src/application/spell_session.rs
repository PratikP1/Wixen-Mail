//! A pass through the spelling of a message, one word at a time.
//!
//! The engine already marks misspellings in the editor and every screen reader
//! announces them as the caret crosses them. What no engine exposes is the
//! *list*, so there is no way to ask "take me to the next one", and reading a
//! message end to end to find three wrong words is not a way to check spelling.
//!
//! That is what this is for. It decides which words to stop on, in what order,
//! and what "ignore all" and "change all" mean afterwards. Everything here is
//! ordinary data and arithmetic: no dictionary, no widgets, no page. The
//! speller is passed in as a pair of closures so this can be tested against a
//! made-up one, and the composer is what holds the real one.
//!
//! # Word positions, and why they are the page's business
//!
//! Offsets into a string cannot be mapped back into a document: what the
//! editor shows is a tree, and the text somebody reads has line breaks between
//! blocks that the tree does not contain. So the page enumerates the words and
//! everything here is expressed as an index into that list. Nothing in this
//! file knows how long a word is or where it sits.

use std::collections::HashSet;

/// What is wrong with a word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Problem {
    /// The dictionary does not have it.
    Misspelled,
    /// It is the same word as the one before it.
    ///
    /// Worth separating because "the the" is two correctly spelled words, and
    /// calling it a misspelling sends somebody looking for a typo that is not
    /// there.
    Repeated,
}

impl Problem {
    /// How the word is described when the check stops on it.
    pub const fn spoken(self) -> &'static str {
        match self {
            Self::Misspelled => "not in the dictionary",
            Self::Repeated => "repeated word",
        }
    }
}

/// A word the check will stop on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Where the word is in the list the page enumerated.
    pub index: usize,
    pub word: String,
    pub problem: Problem,
    pub suggestions: Vec<String>,
}

impl Finding {
    /// What is said when the check arrives at this word.
    ///
    /// The word and what is wrong with it, then the first suggestion, because
    /// most of the time the first suggestion is the answer and hearing it saves
    /// arrowing into a list to find out.
    pub fn spoken(&self) -> String {
        let mut said = format!("{}, {}", self.word, self.problem.spoken());
        if let Some(first) = self.suggestions.first() {
            said.push_str(&format!(". First suggestion, {first}"));
        } else {
            said.push_str(". No suggestions");
        }
        said
    }
}

/// The words worth stopping on, in the order they appear.
///
/// `wrong` and `suggest` are the speller. Passing them in rather than taking a
/// `Speller` keeps this testable without a dictionary, and keeps the decision
/// about *which* speller in one place in the composer.
///
/// A repeated word is reported even when both spellings are fine, and a word
/// that is both repeated and misspelled is reported once, as a misspelling,
/// because that is the thing to fix first.
pub fn findings(
    words: &[String],
    wrong: impl Fn(&str) -> bool,
    suggest: impl Fn(&str) -> Vec<String>,
) -> Vec<Finding> {
    let mut found = Vec::new();
    for (index, word) in words.iter().enumerate() {
        if wrong(word) {
            found.push(Finding {
                index,
                word: word.clone(),
                problem: Problem::Misspelled,
                suggestions: suggest(word),
            });
            continue;
        }
        let repeated = index > 0 && same_word(&words[index - 1], word);
        if repeated {
            found.push(Finding {
                index,
                word: word.clone(),
                problem: Problem::Repeated,
                // Deleting it is the fix, and the dialog offers that as the
                // suggestion rather than leaving somebody to work it out.
                suggestions: Vec::new(),
            });
        }
    }
    found
}

/// Whether two words are the same word, ignoring how they are capitalised.
///
/// Case-insensitively, because "The the" is the same mistake as "the the" and
/// happens more often: it is what a sentence break plus a duplicated word looks
/// like.
fn same_word(first: &str, second: &str) -> bool {
    first.to_lowercase() == second.to_lowercase()
}

/// The words this pass has been told to stop stopping on.
#[derive(Debug, Default)]
pub struct Ignored {
    words: HashSet<String>,
}

impl Ignored {
    /// Stop offering this word for the rest of the pass.
    pub fn add(&mut self, word: &str) {
        self.words.insert(word.to_lowercase());
    }

    /// Whether this word has been ignored.
    pub fn skips(&self, word: &str) -> bool {
        self.words.contains(&word.to_lowercase())
    }
}

/// How many words a replacement is.
///
/// Not always one: "alot" is corrected to "a lot", and a correction that adds a
/// word moves every word after it along by one.
pub fn words_in(replacement: &str) -> usize {
    replacement.split_whitespace().count().max(1)
}

/// Where to carry on from after a word has been replaced.
///
/// `changed` is where the word was before anything moved. `earlier` is how many
/// occurrences before it were replaced in the same action, which is how Change
/// All can move the word it is standing on. Getting this wrong does not fail
/// loudly: it silently skips a misspelling, or offers the same one forever.
pub fn resume_after(changed: usize, earlier: usize, replacement: &str) -> usize {
    let words = words_in(replacement);
    changed + earlier * (words - 1) + words
}

/// The next word to stop on, given everything already dealt with.
pub fn next_finding<'a>(
    found: &'a [Finding],
    ignored: &Ignored,
    resume_from: usize,
) -> Option<&'a Finding> {
    found
        .iter()
        .find(|finding| finding.index >= resume_from && !ignored.skips(&finding.word))
}

/// Every place the same word appears, for Change All.
///
/// Highest index first, so replacing them in order never moves one that has not
/// been replaced yet.
pub fn same_word_indexes(found: &[Finding], word: &str) -> Vec<usize> {
    let mut indexes: Vec<usize> = found
        .iter()
        .filter(|finding| same_word(&finding.word, word))
        .map(|finding| finding.index)
        .collect();
    indexes.sort_unstable_by(|left, right| right.cmp(left));
    indexes
}

/// What is said when a pass finds nothing, or reaches the end.
///
/// A message with nothing wrong is never interrupted by a dialog, so this is
/// the only thing that happens, and it has to say which of the two it is: "no
/// mistakes" after checking a whole message means something different from "no
/// more mistakes" after correcting four.
pub fn finished(corrected: usize) -> String {
    match corrected {
        0 => "Spelling checked. Nothing to correct.".to_string(),
        1 => "Spelling checked. One word corrected.".to_string(),
        many => format!("Spelling checked. {many} words corrected."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(text: &str) -> Vec<String> {
        text.split_whitespace().map(str::to_string).collect()
    }

    fn nothing_wrong(_: &str) -> bool {
        false
    }

    fn no_suggestions(_: &str) -> Vec<String> {
        Vec::new()
    }

    #[test]
    fn test_a_clean_message_has_nothing_to_stop_on() {
        let found = findings(
            &words("all of these are fine"),
            nothing_wrong,
            no_suggestions,
        );
        assert!(found.is_empty());
    }

    #[test]
    fn test_a_misspelling_is_found_where_it_is() {
        let found = findings(
            &words("the wrold turns"),
            |word| word == "wrold",
            |_| vec!["world".to_string()],
        );
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].index, 1);
        assert_eq!(found[0].word, "wrold");
        assert_eq!(found[0].problem, Problem::Misspelled);
        assert_eq!(found[0].suggestions, vec!["world".to_string()]);
    }

    #[test]
    fn test_a_repeated_word_is_not_called_a_misspelling() {
        // "the" is spelled correctly. Calling this a misspelling sends somebody
        // looking for a typo that is not there.
        let found = findings(&words("and the the end"), nothing_wrong, no_suggestions);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].problem, Problem::Repeated);
        assert_eq!(found[0].index, 2);
    }

    #[test]
    fn test_a_repeat_across_a_capital_still_counts() {
        // What a duplicated word looks like when a sentence break falls between
        // the two, which is how it usually happens.
        let found = findings(&words("stop The the end"), nothing_wrong, no_suggestions);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].problem, Problem::Repeated);
    }

    #[test]
    fn test_a_word_that_is_both_is_reported_once() {
        let found = findings(
            &words("wrold wrold"),
            |word| word == "wrold",
            no_suggestions,
        );
        assert_eq!(found.len(), 2, "both are misspelled");
        assert!(found.iter().all(|f| f.problem == Problem::Misspelled));
    }

    #[test]
    fn test_the_first_word_is_never_a_repeat_of_nothing() {
        let found = findings(&words("hello"), nothing_wrong, no_suggestions);
        assert!(found.is_empty());
    }

    #[test]
    fn test_a_word_says_what_is_wrong_and_what_to_try() {
        let finding = Finding {
            index: 0,
            word: "wrold".to_string(),
            problem: Problem::Misspelled,
            suggestions: vec!["world".to_string(), "wold".to_string()],
        };
        let said = finding.spoken();
        assert!(said.contains("wrold"), "{said}");
        assert!(said.contains("not in the dictionary"), "{said}");
        assert!(said.contains("world"), "{said}");
    }

    #[test]
    fn test_a_word_with_no_suggestions_says_so() {
        // Rather than trailing off, which sounds like the announcement was cut
        // short and leaves somebody waiting for the rest.
        let finding = Finding {
            index: 0,
            word: "Kowalczyk".to_string(),
            problem: Problem::Misspelled,
            suggestions: Vec::new(),
        };
        assert!(finding.spoken().contains("No suggestions"));
    }

    #[test]
    fn test_ignoring_a_word_ignores_it_however_it_is_capitalised() {
        let mut ignored = Ignored::default();
        ignored.add("Kowalczyk");
        assert!(ignored.skips("kowalczyk"));
        assert!(ignored.skips("KOWALCZYK"));
        assert!(!ignored.skips("Kowalski"));
    }

    #[test]
    fn test_the_next_word_is_the_next_one_not_ignored() {
        let found = findings(
            &words("aaa bbb ccc"),
            |_| true,
            |_| vec!["fixed".to_string()],
        );
        let mut ignored = Ignored::default();
        ignored.add("bbb");
        let next = next_finding(&found, &ignored, 1).expect("ccc is still to come");
        assert_eq!(next.word, "ccc");
    }

    #[test]
    fn test_nothing_is_offered_twice() {
        let found = findings(&words("aaa bbb"), |_| true, no_suggestions);
        let ignored = Ignored::default();
        assert!(next_finding(&found, &ignored, 2).is_none());
    }

    #[test]
    fn test_a_replacement_of_one_word_moves_on_by_one() {
        assert_eq!(resume_after(3, 0, "world"), 4);
    }

    #[test]
    fn test_a_replacement_that_is_two_words_moves_on_by_two() {
        // "alot" becomes "a lot", and everything after it shifts. Getting this
        // wrong silently skips the next misspelling.
        assert_eq!(resume_after(3, 0, "a lot"), 5);
        assert_eq!(words_in("a lot"), 2);
    }

    #[test]
    fn test_changing_earlier_copies_moves_the_one_being_stood_on() {
        // Change All replaces occurrences before this one too, and each of
        // those that gains a word pushes this one along.
        assert_eq!(resume_after(5, 2, "a lot"), 5 + 2 + 2);
        assert_eq!(resume_after(5, 2, "world"), 5 + 1);
    }

    #[test]
    fn test_an_empty_replacement_still_counts_as_one_word() {
        // Deleting a repeated word replaces it with nothing, and the pass still
        // has to move past where it was.
        assert_eq!(words_in(""), 1);
        assert_eq!(resume_after(2, 0, ""), 3);
    }

    #[test]
    fn test_change_all_replaces_from_the_end_backwards() {
        // Otherwise replacing the first occurrence with two words moves every
        // later index, and the second replacement lands in the wrong place.
        let found = findings(
            &words("teh cat teh dog teh"),
            |w| w == "teh",
            no_suggestions,
        );
        assert_eq!(same_word_indexes(&found, "teh"), vec![4, 2, 0]);
    }

    #[test]
    fn test_change_all_matches_however_it_was_capitalised() {
        let found = findings(
            &words("Teh cat teh"),
            |w| w.to_lowercase() == "teh",
            no_suggestions,
        );
        assert_eq!(same_word_indexes(&found, "teh"), vec![2, 0]);
    }

    #[test]
    fn test_the_end_of_a_pass_says_which_end_it_was() {
        assert!(finished(0).contains("Nothing to correct"));
        assert!(finished(1).contains("One word"));
        assert!(finished(4).contains('4'));
    }
}

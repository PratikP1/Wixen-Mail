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
//! # Where the words are
//!
//! `application::words` finds them and says where each one sits, as a text
//! node and an offset into it. This works in those coordinates rather than in
//! positions of its own, so carrying on after a correction is a comparison
//! against a place the page reported rather than a sum over how many words a
//! replacement turned out to be.

use crate::application::words::{Position, Word};
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
    /// Where the word is, in the page's own coordinates.
    pub at: Position,
    /// Where it ends, so it can be selected and replaced.
    pub end: usize,
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
    words: &[Word],
    wrong: impl Fn(&str) -> bool,
    suggest: impl Fn(&str) -> Vec<String>,
) -> Vec<Finding> {
    let mut found = Vec::new();
    for (index, word) in words.iter().enumerate() {
        if wrong(&word.text) {
            found.push(Finding {
                at: word.at(),
                end: word.end,
                word: word.text.clone(),
                problem: Problem::Misspelled,
                suggestions: suggest(&word.text),
            });
            continue;
        }
        // Only when the two are actually next to each other. A full stop or a
        // paragraph between them means "the end. The next" is two sentences
        // rather than a duplicate, and the fix offered for a duplicate is to
        // delete it, so reporting one wrongly removes a word that was right.
        let repeated =
            index > 0 && !word.after_break && same_word(&words[index - 1].text, &word.text);
        if repeated {
            found.push(Finding {
                at: word.at(),
                end: word.end,
                word: word.text.clone(),
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

/// The next word to stop on, given everything already dealt with.
///
/// `resume_from` is where the last correction left the caret, which the page
/// reports after making it. There is no arithmetic here on purpose: the
/// version this replaced worked out how far a replacement had moved the words
/// after it, and getting that wrong skipped a misspelling silently rather than
/// reporting anything.
pub fn next_finding<'a>(
    found: &'a [Finding],
    ignored: &Ignored,
    resume_from: Option<Position>,
) -> Option<&'a Finding> {
    found.iter().find(|finding| {
        resume_from.is_none_or(|from| finding.at >= from) && !ignored.skips(&finding.word)
    })
}

/// Every place the same word appears from here on, for Change All.
///
/// Last in the message first, so replacing them in order never moves one that
/// has not been replaced yet.
///
/// From `from` rather than from the beginning, because an occurrence before it
/// is one somebody has already been asked about and chosen to leave. Going
/// back and rewriting it overrides that decision without saying so, and Ignore
/// means "leave this one".
pub fn same_word_places(found: &[Finding], word: &str, from: Position) -> Vec<Position> {
    let mut places: Vec<Position> = found
        .iter()
        .filter(|finding| finding.at >= from && same_word(&finding.word, word))
        .map(|finding| finding.at)
        .collect();
    places.sort_unstable_by(|left, right| right.cmp(left));
    places
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

    use crate::application::words::{TextNode, words_in};

    /// The words of a message, segmented the way the real one is.
    ///
    /// Through `words_in` rather than splitting on spaces, so these run
    /// against the boundaries that ship rather than against a simpler rule
    /// that agrees with them only for ASCII prose.
    fn words(text: &str) -> Vec<Word> {
        words_in(&[TextNode {
            text: text.to_string(),
            block: 0,
        }])
    }

    /// A place in a message that is one text node.
    fn place(offset: usize) -> Position {
        Position { node: 0, offset }
    }

    fn finding_at(word: &str, offset: usize, suggestions: Vec<String>) -> Finding {
        Finding {
            at: place(offset),
            end: offset + word.len(),
            word: word.to_string(),
            problem: Problem::Misspelled,
            suggestions,
        }
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
        assert_eq!(found[0].at, place(4), "after \"the \"");
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
        assert_eq!(found[0].at, place(8), "the second one");
    }

    #[test]
    fn test_a_repeat_across_a_capital_still_counts() {
        // A duplicated word that happens to be capitalised is still a
        // duplicate, as long as nothing separates the two.
        let found = findings(&words("stop The the end"), nothing_wrong, no_suggestions);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].problem, Problem::Repeated);
    }

    #[test]
    fn test_the_same_word_either_side_of_a_full_stop_is_not_a_repeat() {
        // "the end. The next" is two sentences. The fix offered for a repeated
        // word is to delete it, so calling this one takes away a word that was
        // right. The old rule could not see the full stop at all: the page
        // threw the punctuation away before any of this was reached.
        let found = findings(&words("the end. The next"), nothing_wrong, no_suggestions);

        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn test_the_same_word_across_a_paragraph_is_not_a_repeat() {
        let found = findings(
            &words_in(&[
                TextNode {
                    text: "draft".to_string(),
                    block: 0,
                },
                TextNode {
                    text: "draft".to_string(),
                    block: 1,
                },
            ]),
            nothing_wrong,
            no_suggestions,
        );

        assert!(found.is_empty(), "{found:?}");
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
        let finding = finding_at("wrold", 0, vec!["world".to_string(), "wold".to_string()]);
        let said = finding.spoken();
        assert!(said.contains("wrold"), "{said}");
        assert!(said.contains("not in the dictionary"), "{said}");
        assert!(said.contains("world"), "{said}");
    }

    #[test]
    fn test_a_word_with_no_suggestions_says_so() {
        // Rather than trailing off, which sounds like the announcement was cut
        // short and leaves somebody waiting for the rest.
        let finding = finding_at("Kowalczyk", 0, Vec::new());
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
        let next = next_finding(&found, &ignored, Some(place(4))).expect("ccc is still to come");
        assert_eq!(next.word, "ccc");
    }

    #[test]
    fn test_nothing_is_offered_twice() {
        let found = findings(&words("aaa bbb"), |_| true, no_suggestions);
        let ignored = Ignored::default();
        assert!(next_finding(&found, &ignored, Some(place(8))).is_none());
    }

    #[test]
    fn test_a_replacement_carries_on_from_where_it_left_the_caret() {
        // What replaced the arithmetic. "alot" corrected to "a lot" used to
        // move every word after it, so the pass had to work out by how much,
        // and getting that wrong skipped a misspelling without reporting
        // anything. The page now says where the caret ended up, and this
        // compares rather than adds.
        let found = findings(&words("teh alot end"), |w| w != "end", no_suggestions);
        let ignored = Ignored::default();

        // "alot" starts at 4 and became "a lot", five units, so the caret is
        // at 9. Nothing is left to stop on.
        let next = next_finding(&found, &ignored, Some(place(9)));

        assert!(
            next.is_none(),
            "it went back over what it just corrected: {next:?}"
        );
    }

    #[test]
    fn test_a_pass_that_has_corrected_nothing_starts_at_the_beginning() {
        let found = findings(&words("teh cat"), |w| w == "teh", no_suggestions);
        let ignored = Ignored::default();

        assert_eq!(
            next_finding(&found, &ignored, None).map(|f| f.at),
            Some(place(0))
        );
    }

    #[test]
    fn test_change_all_leaves_alone_what_was_already_passed_over() {
        // Ignore means "leave this one". Going back and rewriting it later
        // overrides a decision somebody made, and nothing tells them. Word
        // applies Change All from where you are forward for this reason.
        let found = findings(
            &words("teh cat teh dog teh"),
            |w| w == "teh",
            no_suggestions,
        );

        // Standing on the second, having passed the first with Ignore.
        assert_eq!(
            same_word_places(&found, "teh", place(8)),
            vec![place(16), place(8)],
            "it went back over one that was ignored"
        );
    }

    #[test]
    fn test_change_all_replaces_from_the_end_backwards() {
        // Otherwise replacing the first occurrence with a longer word moves
        // every later one, and the second replacement lands in the wrong
        // place.
        let found = findings(
            &words("teh cat teh dog teh"),
            |w| w == "teh",
            no_suggestions,
        );
        assert_eq!(
            same_word_places(&found, "teh", place(0)),
            vec![place(16), place(8), place(0)]
        );
    }

    #[test]
    fn test_change_all_matches_however_it_was_capitalised() {
        let found = findings(
            &words("Teh cat teh"),
            |w| w.to_lowercase() == "teh",
            no_suggestions,
        );
        assert_eq!(
            same_word_places(&found, "teh", place(0)),
            vec![place(8), place(0)]
        );
    }

    #[test]
    fn test_the_end_of_a_pass_says_which_end_it_was() {
        assert!(finished(0).contains("Nothing to correct"));
        assert!(finished(1).contains("One word"));
        assert!(finished(4).contains('4'));
    }
}

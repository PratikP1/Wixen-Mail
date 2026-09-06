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

    /// What is said when the caret lands on this word and nothing opens.
    ///
    /// A different sentence from [`Self::spoken`] because it is in a different
    /// situation, not because the wording was improved. That one opens a
    /// dialog whose next control is a list of suggestions somebody can arrow
    /// into, so naming the first one is enough and the rest are a key away.
    /// This one is the only thing that happens: there is no list, no dialog and
    /// nothing to arrow into, so a single suggestion leaves somebody with one
    /// guess and no way to hear the others.
    ///
    /// So it says more than one, and bounds how many, which is guardrail 5.
    /// See [`SUGGESTIONS_SAID`] for the number and the reason.
    pub fn spoken_without_a_dialog(&self) -> String {
        let mut said = format!("{}, {}", self.word, self.problem.spoken());
        if self.suggestions.is_empty() {
            said.push_str(". No suggestions");
            return said;
        }
        let a_few: Vec<&str> = self
            .suggestions
            .iter()
            .take(SUGGESTIONS_SAID)
            .map(String::as_str)
            .collect();
        said.push_str(&format!(". Try {}", one_or_another(&a_few)));
        if self.suggestions.len() > SUGGESTIONS_SAID {
            said.push_str(&format!(". {} suggestions in all", self.suggestions.len()));
        }
        said
    }
}

/// A list said the way a person says one: "a, b or c".
///
/// The last one joined with "or" rather than a comma, because a comma before
/// the last item is heard as another item and leaves somebody waiting for a
/// fourth suggestion that never arrives.
fn one_or_another(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [only] => (*only).to_string(),
        [rest @ .., last] => format!("{} or {last}", rest.join(", ")),
    }
}

/// How many suggestions are said when the caret lands on a word.
///
/// Three, and the number is a judgement rather than a measurement. One is what
/// the dialog says and is what this feature exists to improve on: a person who
/// disagrees with the first guess has heard nothing useful. Fourteen is an
/// announcement nobody can interrupt out of, and the words are gone by the time
/// the fourth arrives. Three is about as many as anybody holds from one spoken
/// sentence without asking for it again, and where there are more, the count
/// says so, so nobody is left thinking three is all there was.
pub const SUGGESTIONS_SAID: usize = 3;

/// How many to ask the speller for.
///
/// More than are said, so [`SUGGESTIONS_SAID`] is a bound with something behind
/// it rather than the whole list under another name: asking for three would
/// make the count clause unreachable and it would never be heard.
///
/// It is also what the count in the sentence counts. That count is the number
/// this program has, not a claim about how many the dictionary could produce,
/// because the speller truncates to what it was asked for. Nine is generous
/// enough that the difference almost never arises and small enough that nothing
/// is spent finding suggestions nobody will hear.
pub const SUGGESTIONS_TO_HAVE: usize = 9;

/// What one press of a walk key comes to.
#[derive(Debug, PartialEq, Eq)]
pub enum Step<'a> {
    /// Go to this word, and say what it is.
    Land(&'a Finding),
    /// Move nothing, and say this.
    ///
    /// Said rather than nothing at all. A key that does nothing and says
    /// nothing is indistinguishable from a key nobody bound, and somebody who
    /// cannot see the caret has no other way to tell the two apart.
    Stay(&'static str),
}

/// What is said when a walk key is pressed and the message has nothing wrong.
pub const NOTHING_MISSPELLED: &str = "No misspellings in this message.";

/// What is said at the end of the message.
///
/// Rather than going round to the top again. A walk that wraps silently means
/// somebody who has been pressing a key has no way to tell they are on their
/// second time through, and the words they hear are the same words either way.
pub const NOTHING_AFTER_HERE: &str = "No more misspellings after here.";

/// The next misspelling from where the caret is.
///
/// `from` is a place the page reported, and there is no arithmetic here for the
/// same reason [`next_finding`] has none: a walk that works out where things
/// have moved to gets it wrong by skipping a word and saying nothing about it.
/// This is a comparison against a position the page named.
///
/// Nothing here consults [`Ignored`]. That set belongs to one F7 pass and ends
/// with it, and this walk has no Ignore to press, so consulting it would mean
/// keeping an ignore list alive that nobody can see, add to, or clear. A word
/// somebody passed over in a dialog is still a word they can walk onto.
pub fn next_misspelling<'a>(found: &'a [Finding], from: Option<Position>) -> Step<'a> {
    if found.is_empty() {
        return Step::Stay(NOTHING_MISSPELLED);
    }
    found
        .iter()
        .find(|finding| from.is_none_or(|from| finding.at >= from))
        .map_or(Step::Stay(NOTHING_AFTER_HERE), Step::Land)
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
    fn test_the_first_word_is_never_a_repeat_even_when_nothing_marks_it_as_starting_a_sentence() {
        // Built by hand rather than through `words_in`, which always marks the
        // first word as coming after a break. That means every other test here
        // stops at the break before the index is ever worked out, and the
        // guard that keeps the index inside the list is never the thing doing
        // the work. A page that reported its words another way, or a caller
        // handing in a slice that starts mid-message, would reach it.
        let words = vec![
            Word {
                text: "the".to_string(),
                node: 0,
                start: 0,
                end: 3,
                after_break: false,
            },
            Word {
                text: "the".to_string(),
                node: 0,
                start: 4,
                end: 7,
                after_break: false,
            },
        ];

        let found = findings(&words, nothing_wrong, no_suggestions);

        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].problem, Problem::Repeated);
        assert_eq!(found[0].at, place(4));
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

    // ── Walking to a misspelling without a dialog ──────────────────────────

    /// A message whose every word the made-up speller rejects.
    ///
    /// Through `findings` rather than by hand, so the positions are the ones
    /// the real segmentation produces and a fixture cannot quietly disagree
    /// with the code it is testing.
    fn every_word_wrong(text: &str) -> Vec<Finding> {
        findings(&words(text), |_| true, |_| vec!["fixed".to_string()])
    }

    #[test]
    fn test_landing_on_a_word_offers_more_than_one_thing_to_try() {
        // The whole point of the sentence. The dialog says one suggestion and
        // follows it with a list somebody can arrow into; here there is no
        // list, so one suggestion is one guess and no way to hear the others.
        let finding = finding_at("wrold", 0, vec!["world".to_string(), "wold".to_string()]);

        let said = finding.spoken_without_a_dialog();

        assert!(said.contains("wrold"), "{said}");
        assert!(said.contains("not in the dictionary"), "{said}");
        assert!(said.contains("world"), "{said}");
        assert!(said.contains("wold"), "{said}");
    }

    #[test]
    fn test_a_word_with_more_suggestions_than_are_said_says_how_many_there_are() {
        // Bounded, which is guardrail 5. Fourteen read out in full is an
        // announcement nobody can interrupt out of. The count is what stops
        // the bound turning into a quiet lie: without it, three suggestions
        // and three of seven sound exactly the same.
        let many: Vec<String> = (1..=7).map(|n| format!("guess{n}")).collect();
        let finding = finding_at("wrold", 0, many);

        let said = finding.spoken_without_a_dialog();

        assert_eq!(
            said,
            "wrold, not in the dictionary. Try guess1, guess2 or guess3. \
             7 suggestions in all"
        );
    }

    #[test]
    fn test_a_word_with_no_suggestions_at_all_says_that_rather_than_trailing_off() {
        let finding = finding_at("Kowalczyk", 0, Vec::new());

        assert_eq!(
            finding.spoken_without_a_dialog(),
            "Kowalczyk, not in the dictionary. No suggestions"
        );
    }

    #[test]
    fn test_the_dialogs_sentence_is_not_the_one_said_when_there_is_no_dialog() {
        // Two situations, two sentences. Changing the shared one would make
        // the dialog read its suggestions out and then show them in the list
        // underneath, saying everything twice.
        let finding = finding_at("wrold", 0, vec!["world".to_string(), "wold".to_string()]);

        assert_eq!(
            finding.spoken(),
            "wrold, not in the dictionary. First suggestion, world"
        );
        assert_ne!(finding.spoken(), finding.spoken_without_a_dialog());
    }

    #[test]
    fn test_the_first_press_lands_on_the_first_misspelling() {
        let found = every_word_wrong("aaa bbb ccc");

        assert_eq!(
            next_misspelling(&found, Some(place(0))),
            Step::Land(&found[0])
        );
    }

    #[test]
    fn test_pressing_it_again_moves_on_rather_than_offering_the_same_word_twice() {
        // The caret is left at the end of the word it landed on, which is
        // where the page puts it when a word is selected, so the next press
        // starts past it. Offering the same word again would be a key that
        // looks like it did nothing.
        let found = every_word_wrong("aaa bbb ccc");
        let landed_on_the_first = place(found[0].end);

        assert_eq!(
            next_misspelling(&found, Some(landed_on_the_first)),
            Step::Land(&found[1])
        );
    }

    #[test]
    fn test_past_the_last_misspelling_it_says_so_and_wraps_nothing() {
        let found = every_word_wrong("aaa bbb");
        let past_them_all = place(100);

        assert_eq!(
            next_misspelling(&found, Some(past_them_all)),
            Step::Stay("No more misspellings after here.")
        );
    }

    #[test]
    fn test_a_message_with_nothing_wrong_says_so_rather_than_saying_nothing() {
        // The case where a key really does have nothing to do. Silence here is
        // indistinguishable from a key nobody bound, and somebody who cannot
        // see the caret has no other way to tell.
        let found = findings(
            &words("all of these are fine"),
            nothing_wrong,
            no_suggestions,
        );

        assert_eq!(
            next_misspelling(&found, None),
            Step::Stay("No misspellings in this message.")
        );
    }

    #[test]
    fn test_the_walk_offers_a_word_an_f7_pass_would_have_ignored() {
        // The decision, and both halves of it are asserted so the difference
        // is the test rather than a comment. An `Ignored` set belongs to one
        // F7 pass and ends with it; this walk has no Ignore to press, so
        // consulting one would mean an ignore list nobody can see or clear.
        let found = every_word_wrong("aaa bbb");
        let mut ignored = Ignored::default();
        ignored.add("bbb");

        assert_eq!(
            next_finding(&found, &ignored, Some(place(4))),
            None,
            "F7's walk should still skip a word it was told to ignore"
        );
        assert_eq!(
            next_misspelling(&found, Some(place(4))),
            Step::Land(&found[1])
        );
    }
}

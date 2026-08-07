//! One rule for punctuating a summary that is read aloud.
//!
//! A sync summary is a few counts and, when something needs explaining, a whole
//! sentence or two. Each module built its own by pushing the next piece on to
//! the end of the last, and every module got the same fault: a clause carrying
//! its own full stop, followed by a clause starting with a comma, leaves "send
//! them., 2 errors", and two sentences in a row leave "send them.. 2 contacts".
//!
//! On screen that is a typo. Spoken it is not: a screen reader pauses at every
//! full stop, so the person hears a stutter and then a fragment, and the
//! sentence that was worth interrupting them for is the one that arrives
//! broken.
//!
//! So the parts are collected rather than concatenated, and this decides the
//! punctuation once. A part says what it is, a count in the opening list or a
//! sentence of its own, and never how it is joined to its neighbours. A part
//! that arrives with a stop of its own has it taken off, because trusting every
//! caller to leave it off is what this replaces.

/// A summary built from its parts, punctuated in one place.
///
/// The opening is a label and a list of counts, so it is left without a full
/// stop when nothing follows it. Sentences after it are sentences: each one is
/// closed, and so is the opening in front of them.
#[derive(Debug)]
pub struct SummingUp {
    opening: String,
    counts: Vec<String>,
    sentences: Vec<String>,
}

impl SummingUp {
    /// Start with the line that always gets said.
    pub fn opening(opening: impl Into<String>) -> Self {
        Self {
            opening: opening.into(),
            counts: Vec::new(),
            sentences: Vec::new(),
        }
    }

    /// Add a count to the opening list, such as `"2 sent"`.
    ///
    /// Counts join the opening in the order they are added, whenever they are
    /// added, so a count worked out after a sentence does not end up behind it.
    pub fn count(&mut self, count: impl Into<String>) {
        self.counts.push(count.into());
    }

    /// Add a whole sentence after the counts, without a stop of its own.
    ///
    /// For what a count cannot carry: which setting to turn on, which calendar
    /// refused the change, what was lost.
    pub fn sentence(&mut self, sentence: impl Into<String>) {
        self.sentences.push(sentence.into());
    }

    /// The whole thing, in the words the status line and a screen reader share.
    pub fn spoken(&self) -> String {
        let sentences = worth_saying(&self.sentences);
        let mut said = without_a_stop(&self.opening).to_string();
        for count in worth_saying(&self.counts) {
            said.push_str(", ");
            said.push_str(count);
        }
        for sentence in &sentences {
            close(&mut said);
            said.push(' ');
            said.push_str(sentence);
        }
        if !sentences.is_empty() {
            close(&mut said);
        }
        said
    }
}

/// Close what has been said, so the next sentence starts after a stop.
///
/// A question or an exclamation is a stop already and keeps its own mark.
fn close(said: &mut String) {
    if !said.ends_with(['!', '?']) {
        said.push('.');
    }
}

/// A part with no space or full stop of its own, however it was written.
///
/// Both ends, because a part says what it is and never how it is joined to its
/// neighbours. The space in front is as much a joining decision as the stop
/// behind: a sentence written with one gave "deleted.  Term dates", and this
/// module exists to make that decision once.
fn without_a_stop(part: &str) -> &str {
    part.trim().trim_end_matches('.').trim_end()
}

/// The parts that have something in them, each without a stop of its own.
///
/// A part that says nothing is left out rather than punctuated. The sentences
/// are written in other modules, and one that came out empty would otherwise be
/// a stop with nothing in front of it: a pause where somebody is waiting to
/// hear what went wrong.
fn worth_saying(parts: &[String]) -> Vec<&str> {
    parts
        .iter()
        .map(|part| without_a_stop(part))
        .filter(|part| !part.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_an_opening_on_its_own_is_left_as_it_was_written() {
        // The opening is a label and a list of counts rather than a sentence,
        // and a sync where nothing needs explaining says only that.
        let said = SummingUp::opening("Contacts sync: 0 created, 0 updated, 0 deleted");

        assert_eq!(
            said.spoken(),
            "Contacts sync: 0 created, 0 updated, 0 deleted"
        );
    }

    #[test]
    fn test_counts_join_the_opening_list_with_a_pause_each() {
        let mut said = SummingUp::opening("Contacts sync: 1 created");
        said.count("2 sent");
        said.count("1 errors");

        assert_eq!(said.spoken(), "Contacts sync: 1 created, 2 sent, 1 errors");
    }

    #[test]
    fn test_a_sentence_starts_after_a_stop_and_ends_with_one() {
        let mut said = SummingUp::opening("Contacts sync: 1 created");
        said.sentence("2 changes are waiting here: turn on Allow Changes");

        assert_eq!(
            said.spoken(),
            "Contacts sync: 1 created. 2 changes are waiting here: turn on Allow Changes."
        );
    }

    #[test]
    fn test_two_sentences_in_a_row_are_separated_by_one_stop() {
        // The fault this exists to stop: two sentences pushed on to each other
        // gave "send them.. 2 contacts", which is heard as a stutter.
        let mut said = SummingUp::opening("Contacts sync: 1 created");
        said.sentence("Something was waiting");
        said.sentence("Something else went with it");

        assert_eq!(
            said.spoken(),
            "Contacts sync: 1 created. Something was waiting. Something else went with it."
        );
    }

    #[test]
    fn test_a_part_that_brought_its_own_stop_does_not_get_a_second() {
        // The other half of the fault, and the reason the rule lives here
        // rather than in a note asking every caller to leave the stop off. The
        // calendar's sentences are written elsewhere and arrive with a stop.
        let mut said = SummingUp::opening("Calendar sync: 1 created.");
        said.count("1 errors.");
        said.sentence("Term dates: 1 change made here cannot be saved.");

        assert_eq!(
            said.spoken(),
            "Calendar sync: 1 created, 1 errors. Term dates: 1 change made here cannot be saved."
        );
    }

    #[test]
    fn test_a_count_added_after_a_sentence_still_joins_the_opening_list() {
        // Where the second half of the fault came from: the errors count was
        // worked out last and pushed on after a sentence, so it arrived as ",
        // 2 errors" behind a full stop.
        let mut said = SummingUp::opening("Contacts sync: 1 created");
        said.sentence("2 changes are waiting here");
        said.count("2 errors");

        assert_eq!(
            said.spoken(),
            "Contacts sync: 1 created, 2 errors. 2 changes are waiting here."
        );
    }

    #[test]
    fn test_a_part_with_nothing_in_it_is_left_out_rather_than_spoken_as_a_stop() {
        // The sentences come from other modules, and one that came out empty
        // would otherwise be a full stop with nothing in front of it: a pause
        // where a person is waiting to hear what went wrong.
        let mut said = SummingUp::opening("Calendar sync: 1 created");
        said.count(String::new());
        said.sentence("  ");
        said.sentence("Term dates: 1 change cannot be saved");

        assert_eq!(
            said.spoken(),
            "Calendar sync: 1 created. Term dates: 1 change cannot be saved."
        );
    }

    #[test]
    fn test_a_part_written_with_a_space_in_front_of_it_is_not_spoken_with_two() {
        // The other half of the rule this module states and did not keep. A
        // part says what it is and never how it is joined to its neighbours,
        // so the space in front of it belongs here too. Written with one of
        // its own it gave "deleted.  Term dates", which is a typo on the
        // status line and an unknown quantity spoken: what a synthesiser does
        // with a double space is up to the synthesiser, and this line goes to
        // whichever one the person is using.
        let mut said = SummingUp::opening("  Calendar sync: 0 created, 0 updated, 0 deleted");
        said.count(" 1 sent");
        said.sentence(" Term dates: 1 change made here cannot be saved");

        assert_eq!(
            said.spoken(),
            "Calendar sync: 0 created, 0 updated, 0 deleted, 1 sent. \
             Term dates: 1 change made here cannot be saved."
        );
    }

    #[test]
    fn test_a_sentence_that_asks_something_keeps_its_question_mark() {
        // A question mark is a stop, so it is not given a second one, and the
        // sentence after it still starts after a stop.
        let mut said = SummingUp::opening("Mail sync: 1 arrived");
        said.sentence("Is this account still signed in?");
        said.sentence("Nothing was sent");

        assert_eq!(
            said.spoken(),
            "Mail sync: 1 arrived. Is this account still signed in? Nothing was sent."
        );
    }

    #[test]
    fn test_a_question_at_the_end_is_not_given_a_full_stop_as_well() {
        let mut said = SummingUp::opening("Mail sync: 1 arrived");
        said.sentence("Is this account still signed in?");

        assert_eq!(
            said.spoken(),
            "Mail sync: 1 arrived. Is this account still signed in?"
        );
    }
}

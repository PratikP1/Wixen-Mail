//! What a job that stops partway got done, and where it stopped.
//!
//! Some work here is a batch with no transaction under it. Deleting a folder
//! that has folders inside it is several `DELETE` commands, because RFC 9051
//! section 6.3.5 forbids one `DELETE` from removing the names inside a folder,
//! so the client sends one per folder, deepest first. Emptying a folder is the
//! same shape. Either can stop halfway, and there is nothing to roll back to.
//!
//! # Why all-or-nothing is not offered
//!
//! IMAP has no transaction to build it on. Putting messages back means
//! appending them again, and an appended message is a new message with a new
//! UID: every flag, every thread it was part of, and every reference to it from
//! anywhere else points at something that is gone. A rollback that loses more
//! than the failure did is not a rollback.
//!
//! So the job stops at the first refusal and says exactly how far it got. D-36
//! sets the shape of that sentence and this type exists to produce it:
//!
//! > Emptied Archive/2026 and Archive/2025. Stopped at Archive/2024: the server
//! > refused. 118 messages were not removed.
//!
//! Running it again finishes the job, because what was done is done and is no
//! longer in the list the next run walks.
//!
//! Nothing else in this codebase reports this way. Every other batch either
//! finishes or comes back as one error naming nothing, which is why this is its
//! own type with its own tests rather than a shape private to one command.

/// Where a job stopped, and what was said about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoppedAt {
    /// What it was working on when it stopped, as somebody reads it.
    pub name: String,
    /// Why, in the words of whatever refused. The server's own sentence rather
    /// than a house phrase, so what somebody hears is about what happened.
    pub because: String,
}

/// How far a batch got before it stopped, or that it did not stop.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HowFarItGot {
    /// What was finished, in the order it was finished.
    pub done: Vec<String>,
    /// Where it stopped, if it stopped. `None` means it finished.
    pub stopped_at: Option<StoppedAt>,
    /// How many of the things being counted were left where they were.
    ///
    /// Not the same as the number of items not reached: emptying counts
    /// messages while walking folders, and D-36's sentence says the messages.
    pub left_behind: usize,
}

impl HowFarItGot {
    /// Whether the whole job was done.
    pub const fn finished(&self) -> bool {
        self.stopped_at.is_none()
    }

    /// The sentence somebody hears and reads.
    ///
    /// `did` is what happened to the things in `done`, capitalised, as it opens
    /// the sentence: "Emptied", "Deleted". `leftover` names what `left_behind`
    /// counts, in the plural: "messages", "folders". Both are asked for rather
    /// than guessed, because the two jobs this serves count different things
    /// and a type that decided for them would be right for one of them.
    ///
    /// Nothing is said about what was left behind when nothing was, so a job
    /// that stopped on its very first item does not end with a zero.
    pub fn said(&self, did: &str, leftover: &str) -> String {
        let mut sentence = String::new();
        if !self.done.is_empty() {
            sentence.push_str(&format!("{did} {}.", in_a_list(&self.done)));
        }
        let Some(stopped) = &self.stopped_at else {
            return if sentence.is_empty() {
                format!("There was nothing to be {}.", did.to_lowercase())
            } else {
                sentence
            };
        };
        if !sentence.is_empty() {
            sentence.push(' ');
        }
        sentence.push_str(&format!(
            "Stopped at {}: {}.",
            stopped.name,
            stopped.because.trim_end_matches('.')
        ));
        if self.left_behind > 0 {
            // The noun as well as the verb. "1 folders was not deleted" is
            // exactly what a synthesiser reads out, word for word.
            let (leftover, being) = if self.left_behind == 1 {
                (leftover.strip_suffix('s').unwrap_or(leftover), "was")
            } else {
                (leftover, "were")
            };
            sentence.push_str(&format!(
                " {} {leftover} {being} not {}.",
                self.left_behind,
                past_tense(did),
            ));
        }
        sentence
    }

    /// What a walk that deleted folders did.
    ///
    /// The words live here rather than at the window, which is the rule
    /// `test_only_the_delete_owner_words_what_a_delete_did` already enforces
    /// for a message delete and which holds for the same reason: a sentence
    /// about a delete written in two places is two sentences that drift, and
    /// the one somebody hears then depends on which path they took.
    pub fn what_deleting_folders_did(&self) -> String {
        self.said("Deleted", "folders")
    }
}

/// Several names, read out the way a person says them.
///
/// "A, B and C" rather than a comma-separated list, because a synthesiser
/// reading the last comma gives no signal that the list has ended.
fn in_a_list(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [only] => only.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// The verb again, for the end of the sentence about what was left.
///
/// "Emptied" opens the sentence and "removed" ends it, because a message is not
/// emptied. The one irregular case is spelled out and everything else keeps the
/// word it came with, lowercased.
fn past_tense(did: &str) -> String {
    match did.to_lowercase().as_str() {
        "emptied" => "removed".to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stopped(name: &str, because: &str) -> Option<StoppedAt> {
        Some(StoppedAt {
            name: name.to_string(),
            because: because.to_string(),
        })
    }

    fn names(of: &[&str]) -> Vec<String> {
        of.iter().map(|name| (*name).to_string()).collect()
    }

    #[test]
    fn test_the_sentence_d_36_asks_for_is_the_sentence_this_produces() {
        // Quoted from the decision rather than paraphrased. If this type
        // cannot say this, it is not the type the decision asked for.
        let got = HowFarItGot {
            done: names(&["Archive/2026", "Archive/2025"]),
            stopped_at: stopped("Archive/2024", "the server refused"),
            left_behind: 118,
        };

        assert_eq!(
            got.said("Emptied", "messages"),
            "Emptied Archive/2026 and Archive/2025. Stopped at Archive/2024: the server refused. \
             118 messages were not removed."
        );
    }

    #[test]
    fn test_a_job_that_finished_says_what_it_did_and_stops_there() {
        let got = HowFarItGot {
            done: names(&["A/B/C", "A/B", "A"]),
            stopped_at: None,
            left_behind: 0,
        };

        assert_eq!(got.said("Deleted", "folders"), "Deleted A/B/C, A/B and A.");
        assert!(got.finished());
    }

    #[test]
    fn test_one_thing_done_is_not_read_out_as_a_list() {
        let got = HowFarItGot {
            done: names(&["Archive"]),
            stopped_at: None,
            left_behind: 0,
        };

        assert_eq!(got.said("Deleted", "folders"), "Deleted Archive.");
    }

    #[test]
    fn test_stopping_on_the_very_first_thing_says_so_without_claiming_anything_was_done() {
        let got = HowFarItGot {
            done: Vec::new(),
            stopped_at: stopped("Archive", "the server would not do it"),
            left_behind: 3,
        };

        let said = got.said("Deleted", "folders");
        assert!(
            said.starts_with("Stopped at Archive"),
            "it claimed something was done: {said}"
        );
        assert!(said.contains("3 folders were not deleted"), "{said}");
        assert!(!got.finished());
    }

    #[test]
    fn test_one_thing_left_behind_is_not_read_out_in_the_plural() {
        // "1 folders were not deleted" is the sort of thing a synthesiser
        // reads out exactly as written.
        let got = HowFarItGot {
            done: names(&["A/B"]),
            stopped_at: stopped("A", "the server would not do it"),
            left_behind: 1,
        };

        let said = got.said("Deleted", "folders");
        assert!(said.contains("1 folder was not deleted"), "{said}");
    }

    #[test]
    fn test_nothing_left_behind_is_not_read_out_as_a_zero() {
        let got = HowFarItGot {
            done: names(&["A/B"]),
            stopped_at: stopped("A", "the server would not do it"),
            left_behind: 0,
        };

        let said = got.said("Deleted", "folders");
        assert!(!said.contains('0'), "{said}");
        assert!(said.contains("Stopped at A"), "{said}");
    }

    #[test]
    fn test_a_reason_that_already_ends_in_a_full_stop_does_not_get_a_second() {
        // The server's own words arrive with whatever punctuation the server
        // used, and two full stops in a row is a pause a synthesiser takes.
        let got = HowFarItGot {
            done: Vec::new(),
            stopped_at: stopped("A", "the server would not do it."),
            left_behind: 0,
        };

        assert!(!got.said("Deleted", "folders").contains(".."));
    }

    #[test]
    fn test_a_job_with_nothing_to_do_says_that_rather_than_nothing() {
        let got = HowFarItGot::default();

        assert_eq!(
            got.said("Deleted", "folders"),
            "There was nothing to be deleted."
        );
    }

    #[test]
    fn test_emptying_says_removed_at_the_end_because_a_message_is_not_emptied() {
        let got = HowFarItGot {
            done: names(&["A"]),
            stopped_at: stopped("B", "the server refused"),
            left_behind: 4,
        };

        assert!(
            got.said("Emptied", "messages")
                .contains("4 messages were not removed"),
            "{}",
            got.said("Emptied", "messages")
        );
    }
}

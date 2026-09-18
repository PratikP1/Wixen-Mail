//! How much is said while mail and the other modules are fetched.
//!
//! The tester's words in #38: "When fetching mail and other items, the
//! announcements are too verbose. Only folders and items with new mail or
//! items should be announced," and, the same day, "Or make this
//! user-configurable. Let the user decide how much to announce while fetching
//! items." Until 2026-09-17 every line a check wrote to the status bar was
//! spoken, at Low under one topic, and a check of fifty folders spoke fifty
//! lines.
//!
//! A fetch produces lines of four kinds, and the kind is decided where the
//! line is made, not here: a step ("Checking Inbox..."), a result ("Inbox,
//! 3 new"), an error, and the answer to a key somebody pressed ("Settings
//! saved"). This module holds the person's choice of how much of that to hear
//! and answers one question for each kind: is a line of this kind spoken
//! under this choice. Errors and answers always are. The choice is offered on
//! the Feedback tab and kept beside the other routing choices on
//! `Accessibility`, where the arms that speak ask for it.

/// How much is said while things are fetched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HowMuchToSay {
    /// Which folders and modules received something, with counts, once per
    /// check. The default, and the tester's shape from #38: it is what an
    /// older settings file with no such key answers.
    #[default]
    WhatArrived,
    /// Every step as well: connecting, each folder checked, each chunk. What
    /// every check said until 2026-09-17.
    EveryStep,
    /// Nothing but what went wrong.
    ErrorsOnly,
}

/// The kinds of line a fetch produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A step on the way: connecting, checking a folder, a chunk landed.
    Progress,
    /// Something arrived, and where, with a count.
    Result,
    /// Something went wrong.
    Error,
    /// The answer to a key somebody pressed: saved, refreshed, filed.
    Answer,
}

/// The label beside the choice on the Feedback tab. The ampersand is the
/// keyboard accelerator, on "W" for while, which no other control on that
/// page claims.
pub const WHILE_FETCHING_LABEL: &str = "&While mail and the other modules are fetched, say:";

/// The sentence under the choice: what the choice does not change, because
/// a list of three answers cannot say on its own what stays the same under
/// every one of them.
pub const WHAT_THE_CHOICE_LEAVES_ALONE: &str =
    "Errors are always said. The sound for new mail follows its own row above.";

impl HowMuchToSay {
    /// The choices offered, in the order they are offered: the default
    /// first, then more, then less.
    pub const ALL: [HowMuchToSay; 3] = [
        HowMuchToSay::WhatArrived,
        HowMuchToSay::EveryStep,
        HowMuchToSay::ErrorsOnly,
    ];

    /// What the choice is called: what will be heard, in plain words. None
    /// names a mechanism, because the person choosing is choosing what to
    /// hear and not how the queue works.
    pub fn label(self) -> &'static str {
        match self {
            HowMuchToSay::WhatArrived => "Say what arrived",
            HowMuchToSay::EveryStep => "Say every step",
            HowMuchToSay::ErrorsOnly => "Errors only",
        }
    }

    /// How it is written in the settings file: words a person could read
    /// there, hyphenated as the other string-valued settings are.
    pub fn as_stored(self) -> String {
        match self {
            HowMuchToSay::WhatArrived => "what-arrived",
            HowMuchToSay::EveryStep => "every-step",
            HowMuchToSay::ErrorsOnly => "errors-only",
        }
        .to_string()
    }

    /// Read the stored setting.
    ///
    /// Anything unreadable is what arrived, because the other two answers
    /// each cost more: every step puts somebody back to the verbosity #38
    /// was filed about, and errors only silences arrivals. What arrived
    /// loses neither the counts nor the errors.
    pub fn from_stored(value: &str) -> Self {
        match value.trim() {
            "every-step" => HowMuchToSay::EveryStep,
            "errors-only" => HowMuchToSay::ErrorsOnly,
            _ => HowMuchToSay::WhatArrived,
        }
    }

    /// Whether a line of this kind is spoken under this choice.
    ///
    /// An error and the answer to a key are spoken whatever was chosen: the
    /// choice is about what a fetch says on the way, and neither of those
    /// is that. A result is spoken unless nothing but errors was asked for.
    /// A step is spoken only when every step was.
    pub fn is_spoken(self, kind: Kind) -> bool {
        match kind {
            Kind::Error | Kind::Answer => true,
            Kind::Result => self != HowMuchToSay::ErrorsOnly,
            Kind::Progress => self == HowMuchToSay::EveryStep,
        }
    }
}

/// Which entry of the offered list a stored choice selects.
///
/// A garbled stored value reads as the default, so the entry it selects is
/// the default's, and saving that back writes the default.
pub fn offered_index(stored: &str) -> usize {
    let wanted = HowMuchToSay::from_stored(stored);
    HowMuchToSay::ALL
        .iter()
        .position(|choice| *choice == wanted)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_choices_are_what_arrived_then_every_step_then_errors_only() {
        assert_eq!(
            HowMuchToSay::ALL,
            [
                HowMuchToSay::WhatArrived,
                HowMuchToSay::EveryStep,
                HowMuchToSay::ErrorsOnly,
            ],
            "the default first, then more, then less, which is the order a person \
             arrowing down a list meets them"
        );
    }

    #[test]
    fn test_each_choice_says_what_will_be_heard_in_plain_words() {
        let labels: Vec<&str> = HowMuchToSay::ALL.iter().map(|c| c.label()).collect();
        assert_eq!(
            labels,
            ["Say what arrived", "Say every step", "Errors only"],
            "three sentences a screen reader user picks one of; none names a mechanism"
        );
    }

    #[test]
    fn test_a_choice_is_stored_in_words_a_person_could_read_in_the_file() {
        assert_eq!(HowMuchToSay::WhatArrived.as_stored(), "what-arrived");
        assert_eq!(HowMuchToSay::EveryStep.as_stored(), "every-step");
        assert_eq!(HowMuchToSay::ErrorsOnly.as_stored(), "errors-only");
    }

    #[test]
    fn test_every_offered_choice_survives_being_stored_and_read_back() {
        for choice in HowMuchToSay::ALL {
            assert_eq!(HowMuchToSay::from_stored(&choice.as_stored()), choice);
        }
        assert_eq!(
            HowMuchToSay::from_stored("  every-step \n"),
            HowMuchToSay::EveryStep
        );
    }

    #[test]
    fn test_anything_unreadable_reads_as_what_arrived_because_the_other_two_cost_more() {
        // Every step puts somebody back to the verbosity #38 was filed about;
        // errors only silences arrivals. What arrived loses neither the
        // counts nor the errors.
        for garbled in ["", "   ", "everything", "0", "every step", "ERRORS-ONLY"] {
            assert_eq!(
                HowMuchToSay::from_stored(garbled),
                HowMuchToSay::WhatArrived,
                "{garbled:?} is not a choice and must read as the default"
            );
        }
    }

    #[test]
    fn test_the_default_says_what_arrived() {
        // The tester's shape from #38, and what an older settings file with
        // no such key answers.
        assert_eq!(HowMuchToSay::default(), HowMuchToSay::WhatArrived);
        assert_eq!(HowMuchToSay::default().as_stored(), "what-arrived");
    }

    #[test]
    fn test_a_stored_choice_selects_its_entry_and_a_garbled_one_selects_the_default() {
        assert_eq!(offered_index("what-arrived"), 0);
        assert_eq!(offered_index("every-step"), 1);
        assert_eq!(offered_index("errors-only"), 2);
        assert_eq!(offered_index("garbled"), 0);
    }

    // The table, one cell per test, so a wrong cell is named rather than
    // counted. Rows are the choices, columns the kinds.

    #[test]
    fn test_under_what_arrived_a_step_is_not_spoken() {
        assert!(!HowMuchToSay::WhatArrived.is_spoken(Kind::Progress));
    }

    #[test]
    fn test_under_what_arrived_a_result_is_spoken() {
        assert!(HowMuchToSay::WhatArrived.is_spoken(Kind::Result));
    }

    #[test]
    fn test_under_what_arrived_an_error_is_spoken() {
        assert!(HowMuchToSay::WhatArrived.is_spoken(Kind::Error));
    }

    #[test]
    fn test_under_what_arrived_an_answer_is_spoken() {
        assert!(HowMuchToSay::WhatArrived.is_spoken(Kind::Answer));
    }

    #[test]
    fn test_under_every_step_a_step_is_spoken() {
        assert!(HowMuchToSay::EveryStep.is_spoken(Kind::Progress));
    }

    #[test]
    fn test_under_every_step_a_result_is_spoken() {
        assert!(HowMuchToSay::EveryStep.is_spoken(Kind::Result));
    }

    #[test]
    fn test_under_every_step_an_error_is_spoken() {
        assert!(HowMuchToSay::EveryStep.is_spoken(Kind::Error));
    }

    #[test]
    fn test_under_every_step_an_answer_is_spoken() {
        assert!(HowMuchToSay::EveryStep.is_spoken(Kind::Answer));
    }

    #[test]
    fn test_under_errors_only_a_step_is_not_spoken() {
        assert!(!HowMuchToSay::ErrorsOnly.is_spoken(Kind::Progress));
    }

    #[test]
    fn test_under_errors_only_a_result_is_not_spoken() {
        assert!(!HowMuchToSay::ErrorsOnly.is_spoken(Kind::Result));
    }

    #[test]
    fn test_under_errors_only_an_error_is_spoken() {
        assert!(HowMuchToSay::ErrorsOnly.is_spoken(Kind::Error));
    }

    #[test]
    fn test_under_errors_only_an_answer_is_spoken() {
        // The answer to a key somebody pressed is not a fetch saying
        // something about itself; the quietest choice still answers a key.
        assert!(HowMuchToSay::ErrorsOnly.is_spoken(Kind::Answer));
    }
}

//! How much message text stays on this computer.
//!
//! The tester's words in #23: "Unless explicitly forbidden, message text
//! should be downloaded along with mail." The forbidding is the Message Text
//! box on the Permissions tab. What there was no way to say was how much of
//! the text, once downloaded, may stay: the body cache dropped the least
//! recently read text above half a gigabyte at the end of every folder sync,
//! by a constant whose own comment said a setting would plug into
//! `MessageCache::keeping_bodies_under` if anybody asked. This is that setting.
//!
//! The default is all of it, which is the tester's decision. A size is one
//! choice away on the same screen, beside the box that forbids fetching, so a
//! person who finds the one finds the other.
//!
//! One value feeds two things: the eviction at the end of a folder sync reads
//! it through [`TextKept::budget`], and the download of everything (10-05)
//! reads the same [`TextBudget`] to know when to stop fetching text. The two
//! agree because they are the same number.

use super::bringing_everything_down::TextBudget;

/// A gigabyte as the label means it: a thousand million bytes, so "Up to
/// 1 GB" and the number written to the settings file agree without anybody
/// having to know which kind of gigabyte was meant.
pub const GIGABYTE: u64 = 1_000_000_000;

/// How much message text stays on this computer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextKept {
    /// Every message's text, however much there is.
    All,
    /// Up to this many bytes; above it, the least recently read text goes.
    UpTo(u64),
}

/// The label beside the choice on the Permissions tab. The ampersand is the
/// keyboard accelerator, on "K" for keep, which no other control on that page
/// claims.
pub const KEEP_LABEL: &str = "&Keep the text of messages on this computer:";

/// The sentence under the choice: what leaves when a size is chosen, when,
/// and what stays, in that order, because a list of four sizes cannot say
/// on its own what passing one costs.
pub const WHAT_A_SIZE_DOES: &str = "With a size chosen, the text of the messages you read least \
     recently is removed when the size is passed and fetched again when you open them. The mail \
     itself stays.";

impl TextKept {
    /// The choices offered, in the order they are offered: the default
    /// first, then the sizes smallest to largest.
    pub const ALL: [TextKept; 4] = [
        TextKept::All,
        TextKept::UpTo(GIGABYTE),
        TextKept::UpTo(5 * GIGABYTE),
        TextKept::UpTo(20 * GIGABYTE),
    ];

    /// What the choice is called. The sizes offered are whole gigabytes, and
    /// the label says the whole number of them.
    pub fn label(self) -> String {
        match self {
            TextKept::All => "All of it".to_string(),
            TextKept::UpTo(bytes) => format!("Up to {} GB", bytes / GIGABYTE),
        }
    }

    /// How it is written in the settings file: "all", or the bytes.
    pub fn as_stored(self) -> String {
        match self {
            TextKept::All => "all".to_string(),
            TextKept::UpTo(bytes) => bytes.to_string(),
        }
    }

    /// Read the stored setting.
    ///
    /// Anything unreadable is `All`, and so is nought, because the other
    /// answer to a garbled value is a bound, and a small bound throws away
    /// the text of every message somebody downloaded (T-10-09). `All` loses
    /// nothing.
    pub fn from_stored(value: &str) -> Self {
        match value.trim() {
            "all" => TextKept::All,
            other => other
                .parse::<u64>()
                .ok()
                .filter(|bytes| *bytes > 0)
                .map(TextKept::UpTo)
                .unwrap_or_default(),
        }
    }

    /// The same answer under the name the eviction and the download use.
    ///
    /// One setting feeds both: the eviction at the end of a folder sync
    /// stops at this, and the runner's text pass (10-05) stops at this, so
    /// the text that was fetched is the text that is kept.
    pub fn budget(self) -> TextBudget {
        match self {
            TextKept::All => TextBudget::All,
            TextKept::UpTo(bytes) => TextBudget::UpTo(bytes),
        }
    }
}

impl Default for TextKept {
    /// All of it: the tester's decision in #23, and what an older settings
    /// file with no such key answers.
    fn default() -> Self {
        TextKept::All
    }
}

/// Which entry of the offered list a stored choice selects.
///
/// A stored size the list does not offer selects the default, which is
/// `All`: somebody who opens Settings with a hand-edited bound sees All, and
/// saving that back keeps their text rather than throwing any away. The
/// eviction itself honours the unoffered size until then.
pub fn offered_index(stored: &str) -> usize {
    let wanted = TextKept::from_stored(stored);
    TextKept::ALL
        .iter()
        .position(|choice| *choice == wanted)
        .or_else(|| {
            TextKept::ALL
                .iter()
                .position(|choice| *choice == TextKept::default())
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_choices_are_all_of_it_then_one_five_and_twenty_gigabytes() {
        assert_eq!(
            TextKept::ALL,
            [
                TextKept::All,
                TextKept::UpTo(GIGABYTE),
                TextKept::UpTo(5 * GIGABYTE),
                TextKept::UpTo(20 * GIGABYTE),
            ],
            "the default first, then the sizes smallest to largest, which is the order a \
             person arrowing down a list meets them"
        );
    }

    #[test]
    fn test_each_choice_says_what_it_keeps_in_words() {
        let labels: Vec<String> = TextKept::ALL.iter().map(|c| c.label()).collect();
        assert_eq!(
            labels,
            ["All of it", "Up to 1 GB", "Up to 5 GB", "Up to 20 GB"],
            "four words a screen reader user picks one of; a number of bytes is not a thing \
             anybody chooses"
        );
    }

    #[test]
    fn test_a_choice_is_stored_as_all_or_as_its_bytes() {
        assert_eq!(TextKept::All.as_stored(), "all");
        assert_eq!(TextKept::UpTo(GIGABYTE).as_stored(), "1000000000");
        assert_eq!(TextKept::UpTo(20 * GIGABYTE).as_stored(), "20000000000");
    }

    #[test]
    fn test_all_reads_back_as_all() {
        assert_eq!(TextKept::from_stored("all"), TextKept::All);
        assert_eq!(TextKept::from_stored("  all \n"), TextKept::All);
    }

    #[test]
    fn test_a_number_reads_back_as_that_many_bytes() {
        assert_eq!(
            TextKept::from_stored("5000000000"),
            TextKept::UpTo(5 * GIGABYTE)
        );
        assert_eq!(
            TextKept::from_stored("1500000000"),
            TextKept::UpTo(1_500_000_000),
            "a size the list does not offer is still honoured by the eviction; only the \
             screen falls back"
        );
    }

    #[test]
    fn test_a_garbled_value_reads_as_all_because_a_wrong_bound_evicts_and_all_loses_nothing() {
        // T-10-09: a garbled stored value read as a small bound would throw
        // away the text of every message somebody had downloaded. Anything
        // this cannot read is All, and nought is not a size anybody meant.
        for garbled in ["", "   ", "lots", "-5", "0", "5 GB", "1e9", "all of it"] {
            assert_eq!(
                TextKept::from_stored(garbled),
                TextKept::All,
                "{garbled:?} is not a size and must not evict anything"
            );
        }
    }

    #[test]
    fn test_every_offered_choice_survives_being_stored_and_read_back() {
        for choice in TextKept::ALL {
            assert_eq!(TextKept::from_stored(&choice.as_stored()), choice);
        }
    }

    #[test]
    fn test_the_default_keeps_all_of_it() {
        // The tester's decision in #23, and the answer an older settings file
        // with no such key gets.
        assert_eq!(TextKept::default(), TextKept::All);
        assert_eq!(TextKept::default().as_stored(), "all");
    }

    #[test]
    fn test_the_budget_is_the_setting_under_the_name_the_eviction_and_the_download_use() {
        assert_eq!(TextKept::All.budget(), TextBudget::All);
        assert_eq!(
            TextKept::UpTo(5 * GIGABYTE).budget(),
            TextBudget::UpTo(5 * GIGABYTE)
        );
    }

    #[test]
    fn test_a_stored_size_the_list_offers_is_selected_and_one_it_does_not_selects_all() {
        assert_eq!(offered_index("all"), 0);
        assert_eq!(offered_index("1000000000"), 1);
        assert_eq!(offered_index("5000000000"), 2);
        assert_eq!(offered_index("20000000000"), 3);
        // Falling back to All rather than to a size: somebody who opens
        // Settings with a hand-edited bound sees All, and saving that back
        // keeps their text rather than throwing any away.
        assert_eq!(offered_index("1500000000"), 0);
        assert_eq!(offered_index("garbled"), 0);
    }
}

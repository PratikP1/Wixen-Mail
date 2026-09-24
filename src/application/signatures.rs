//! Which signature a message starts with, and when a change of From account
//! changes it (#43).
//!
//! Signatures are one set. Each email account can be given one of them, and
//! one of them can be the default, used for every account that has not been
//! given one. The tester asked for exactly that on 2026-09-15: "Allow
//! signatures to be assigned by email account. Default should apply if no
//! signature is assigned to a particular account."
//!
//! Both rules here are pure, so a test reaches them without a window: the
//! store answers an account's signature through [`which_signature`], and the
//! composer decides through [`whether_to_swap`] whether a change of From
//! account may replace the signature already in the message.

/// The signature an account's messages start with: the one assigned to it,
/// else the default for everyone, else none.
pub fn which_signature<Id>(assigned: Option<Id>, default_for_all: Option<Id>) -> Option<Id> {
    let _ = (assigned, default_for_all);
    None
}

/// What a change of From account does to the message being written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Swap {
    /// The block the last account's signature went in as is still there,
    /// untouched: the message with that block replaced by the next one's.
    ReplaceBlock(String),
    /// The block was edited, removed or never there, or the next account
    /// signs the same way: the message stays as the person left it.
    LeaveAlone,
}

/// Whether the signature block in `body` may be replaced on a change of From
/// account, and the message it becomes.
///
/// `was` is the block exactly as the last account's signature went into the
/// message, and `becomes` the next account's the same way. The block is
/// replaced only where it still stands character for character, as a whole
/// line or lines: a signature somebody has typed into is theirs, and replacing
/// it would lose what they wrote. The first place it stands is the one taken,
/// since the composer puts the signature above anything it quotes.
pub fn whether_to_swap(body: &str, was: &str, becomes: &str) -> Swap {
    let _ = (body, was, becomes);
    Swap::LeaveAlone
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_an_assigned_signature_is_the_one_used() {
        assert_eq!(which_signature(Some("work"), Some("home")), Some("work"));
    }

    #[test]
    fn test_an_account_with_none_assigned_takes_the_default() {
        assert_eq!(which_signature(None, Some("home")), Some("home"));
    }

    #[test]
    fn test_an_account_with_none_assigned_and_no_default_has_none() {
        assert_eq!(which_signature::<&str>(None, None), None);
    }

    #[test]
    fn test_an_assignment_stands_with_no_default_set() {
        assert_eq!(which_signature(Some("work"), None), Some("work"));
    }

    #[test]
    fn test_an_untouched_block_is_replaced_by_the_next_accounts() {
        let body = "Dear Ada,<br><br>-- <br>Grace<br><br>--- Original Message ---";
        assert_eq!(
            whether_to_swap(body, "-- <br>Grace", "-- <br>Ada"),
            Swap::ReplaceBlock(
                "Dear Ada,<br><br>-- <br>Ada<br><br>--- Original Message ---".to_string()
            )
        );
    }

    #[test]
    fn test_a_block_somebody_typed_into_is_left_alone() {
        let body = "Dear Ada,<br><br>-- <br>Grace Hopper";
        assert_eq!(
            whether_to_swap(body, "-- <br>Grace", "-- <br>Ada"),
            Swap::LeaveAlone
        );
    }

    #[test]
    fn test_a_body_with_no_block_is_left_alone() {
        assert_eq!(
            whether_to_swap("Dear Ada,", "-- <br>Grace", "-- <br>Ada"),
            Swap::LeaveAlone
        );
    }

    #[test]
    fn test_a_message_opened_with_no_signature_is_left_alone() {
        assert_eq!(
            whether_to_swap("Dear Ada,", "", "-- <br>Ada"),
            Swap::LeaveAlone
        );
    }

    #[test]
    fn test_the_same_signature_on_both_accounts_changes_nothing() {
        assert_eq!(
            whether_to_swap("-- <br>Grace", "-- <br>Grace", "-- <br>Grace"),
            Swap::LeaveAlone
        );
    }

    #[test]
    fn test_a_next_account_with_no_signature_takes_the_block_away() {
        assert_eq!(
            whether_to_swap("Dear Ada,<br>-- <br>Grace", "-- <br>Grace", ""),
            Swap::ReplaceBlock("Dear Ada,<br>".to_string())
        );
    }

    #[test]
    fn test_a_line_added_under_the_block_stays_under_the_next_one() {
        assert_eq!(
            whether_to_swap(
                "-- <br>Grace<br>P.S. Thursday",
                "-- <br>Grace",
                "-- <br>Ada"
            ),
            Swap::ReplaceBlock("-- <br>Ada<br>P.S. Thursday".to_string())
        );
    }

    #[test]
    fn test_text_typed_onto_the_separator_line_is_left_alone() {
        assert_eq!(
            whether_to_swap("Thanks-- <br>Grace", "-- <br>Grace", "-- <br>Ada"),
            Swap::LeaveAlone
        );
    }

    #[test]
    fn test_only_the_first_block_is_replaced() {
        let body = "<div>--&nbsp;</div><p>Grace</p><p>quoted</p><div>--&nbsp;</div><p>Grace</p>";
        let was = "<div>--&nbsp;</div><p>Grace</p>";
        let becomes = "<div>--&nbsp;</div><p>Ada</p>";
        assert_eq!(
            whether_to_swap(body, was, becomes),
            Swap::ReplaceBlock(
                "<div>--&nbsp;</div><p>Ada</p><p>quoted</p><div>--&nbsp;</div><p>Grace</p>"
                    .to_string()
            )
        );
    }
}

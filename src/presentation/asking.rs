//! Which answer Enter gives on a question with two answers.
//!
//! A wxWidgets message box with Yes and No on it makes Yes the answer Enter
//! gives unless it is told otherwise, and that is the wrong way round for a
//! question asked before something that cannot be undone. Somebody who presses
//! Enter partway through hearing the question has deleted the thing before the
//! sentence finished.
//!
//! Anybody can be caught by that and it costs a screen reader user more, for
//! two reasons. Hearing a question takes longer than reading one, so there is
//! more of it left when the finger moves. And Enter is how a person working by
//! keyboard answers everything, so it is already on its way.
//!
//! Not every question wants No, though. The composer asks whether to send a
//! message the spell checker has doubts about, and there Yes on Enter is the
//! point: somebody who meant to send and heard the warning should not have to
//! go looking for a button, and the whole question is in the words, so it can
//! be answered from hearing it.
//!
//! So the answer Enter gives is a decision, and both answers live here so that
//! asking a new question means picking one of them. A house style check holds
//! that: nothing outside this file names the flag that puts two answers on a
//! box.

use wxdragon::dialogs::message_dialog::MessageDialogStyle;

/// The flag that stops Enter answering Yes.
///
/// `wxNO_DEFAULT`. wxWidgets has it and the Rust binding does not name it, so
/// it is built from the constant the binding does carry rather than from the
/// number, which would go stale silently if the number ever moved.
pub const ENTER_DOES_NOT_ANSWER_YES: MessageDialogStyle =
    MessageDialogStyle::from_bits_retain(wxdragon::ffi::WXD_NO_DEFAULT);

/// A question with two answers, where Enter answers No.
///
/// For anything that cannot be undone. The question still says what Yes does,
/// because the button labels are Yes and No and this builder cannot change
/// them, so "Delete Ada Lovelace?" has to be a question those two words answer.
pub fn yes_no_where_enter_answers_no() -> MessageDialogStyle {
    let two_answers = yes_no_where_enter_answers_yes();
    // `|` here could be `^` and nothing would change: `ENTER_DOES_NOT_ANSWER_YES`
    // is bit 7 (128) and `two_answers` never sets it, the same disjointness
    // `yes_no_where_enter_answers_yes` checks below for its own two flags. The
    // assert is what would notice a future wxdragon giving `WXD_NO_DEFAULT`
    // a bit one of `MessageDialogStyle::YesNo` or `IconQuestion` already uses,
    // which is the only way `|` and `^` could ever start disagreeing here.
    debug_assert_eq!(
        two_answers.bits() & ENTER_DOES_NOT_ANSWER_YES.bits(),
        0,
        "yes_no_where_enter_answers_yes now shares a bit with \
         ENTER_DOES_NOT_ANSWER_YES, so combining them is no longer safe to \
         read as an OR of two independent flags"
    );
    two_answers | ENTER_DOES_NOT_ANSWER_YES
}

/// A question with two answers, where Enter answers Yes.
///
/// For a question standing between somebody and the thing they came to do,
/// where the answer that gets on with it is the one they meant. Choosing this
/// is choosing that pressing Enter without hearing the end of the question does
/// the thing, so it belongs only where the thing can be undone or was asked
/// for.
pub fn yes_no_where_enter_answers_yes() -> MessageDialogStyle {
    // `|` and `^` compute the same value here, always: wxWidgets gives every
    // style flag its own bit, and `YesNo` (10, bits 1 and 3) and
    // `IconQuestion` (1024, bit 10) share none of them, which is the one case
    // where OR and XOR cannot disagree. A mutation test replacing this `|`
    // with `^` found exactly that: nothing failed, because nothing could.
    // The assert stands in for that proof at run time, so a future flag value
    // that starts overlapping fails loudly here instead of quietly changing
    // what this function returns.
    debug_assert_eq!(
        MessageDialogStyle::YesNo.bits() & MessageDialogStyle::IconQuestion.bits(),
        0,
        "YesNo and IconQuestion now share a bit, so combining them is no \
         longer safe to read as an OR of two independent flags"
    );
    MessageDialogStyle::YesNo | MessageDialogStyle::IconQuestion
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What wxWidgets calls it, so the two answers can be told apart by number
    /// rather than by the name this file gives them.
    const WX_NO_DEFAULT: i64 = 0x0000_0080;

    #[test]
    fn test_a_question_that_cannot_be_undone_does_not_let_enter_answer_yes() {
        // The whole point. Without the flag, wxWidgets puts the focus on Yes
        // and Enter deletes the thing.
        let style = yes_no_where_enter_answers_no();

        assert_eq!(
            style.bits() & WX_NO_DEFAULT,
            WX_NO_DEFAULT,
            "Enter still answers Yes on a question asked before a deletion"
        );
    }

    #[test]
    fn test_the_deliberate_one_still_lets_enter_answer_yes() {
        // The composer's send question. Getting this wrong in the other
        // direction is quieter and still wrong: somebody who meant to send
        // would have to find a button.
        let style = yes_no_where_enter_answers_yes();

        assert_eq!(
            style.bits() & WX_NO_DEFAULT,
            0,
            "Enter no longer sends a message somebody chose to send"
        );
    }

    #[test]
    fn test_both_still_ask_a_question_with_two_answers() {
        // The flag is added to the style rather than replacing it. Losing the
        // Yes and No answers would leave a box with one button on it, and the
        // deletion would go ahead on any answer.
        for style in [
            yes_no_where_enter_answers_no(),
            yes_no_where_enter_answers_yes(),
        ] {
            assert!(
                style.contains(MessageDialogStyle::YesNo),
                "a question with no answers to choose between"
            );
            assert!(
                style.contains(MessageDialogStyle::IconQuestion),
                "a question that does not look like one"
            );
        }
    }

    #[test]
    fn test_the_flag_is_the_number_wxwidgets_uses() {
        // Read from the binding rather than written here, so this is the one
        // place the two are compared. A wrong number would be a style flag
        // meaning something else, which is the kind of mistake that looks like
        // nothing at all until a dialog behaves oddly.
        assert_eq!(ENTER_DOES_NOT_ANSWER_YES.bits(), WX_NO_DEFAULT);
    }
}

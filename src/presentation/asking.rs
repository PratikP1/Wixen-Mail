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
///
/// Same reasoning as the OR inside [`yes_no_where_enter_answers_yes`]:
/// `ENTER_DOES_NOT_ANSWER_YES` is `wxNO_DEFAULT`, its own bit and disjoint from
/// every flag a Yes/No question already carries, so OR and XOR agree here too.
/// No test could ever tell the two operators apart, so the agreement is
/// asserted below instead of left for one to find.
pub fn yes_no_where_enter_answers_no() -> MessageDialogStyle {
    let yes = yes_no_where_enter_answers_yes();
    debug_assert_eq!(
        yes.bits() & ENTER_DOES_NOT_ANSWER_YES.bits(),
        0,
        "a style flag now overlaps wxNO_DEFAULT, so OR and XOR would disagree here"
    );
    yes | ENTER_DOES_NOT_ANSWER_YES
}

/// A question with two answers, where Enter answers Yes.
///
/// For a question standing between somebody and the thing they came to do,
/// where the answer that gets on with it is the one they meant. Choosing this
/// is choosing that pressing Enter without hearing the end of the question does
/// the thing, so it belongs only where the thing can be undone or was asked
/// for.
///
/// `YesNo` and `IconQuestion` sit in disjoint bits of the flags word: wxWidgets
/// reserves the button set and the icon their own separate ranges, so this OR
/// can never actually merge one answer's bit into the other's. That makes it
/// indistinguishable from XOR, and no test can observe a difference between two
/// operators that agree on every input they are ever given, so the agreement is
/// asserted below rather than left for a mutation test that could never catch
/// it either way.
pub fn yes_no_where_enter_answers_yes() -> MessageDialogStyle {
    debug_assert_eq!(
        MessageDialogStyle::YesNo.bits() & MessageDialogStyle::IconQuestion.bits(),
        0,
        "YesNo and IconQuestion now share a bit, so OR and XOR would disagree here"
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

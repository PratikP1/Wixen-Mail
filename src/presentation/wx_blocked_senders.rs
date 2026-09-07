//! The window that says who you have blocked, and lets you take a block off.
//!
//! A block somebody cannot find is a trap. Mail stops arriving, nothing says
//! why, and the rule doing it is one row among however many rules they have.
//! That sentence is [`crate::application::blocking`]'s own, and this is the
//! window it was written for: everything below it was written, tested and
//! never offered.
//!
//! Nothing here decides anything. The wording of a row, the sentence said on
//! opening and the rule a row stands for all live in the application layer,
//! where a test can read them back without a running window.

use crate::application::blocking::Blocked;

/// Where focus goes when the window opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhereFocusGoes {
    /// Into the list, on the first block, which is what somebody came for.
    TheList,
    /// Onto the button that closes the window.
    TheCloseButton,
}

/// Where focus should go when the window opens.
pub fn where_focus_goes(_rows: usize) -> WhereFocusGoes {
    WhereFocusGoes::TheList
}

/// The row a press acts on, out of what the list is showing.
pub fn the_row_chosen(_showing: &[Blocked], _selected: i64) -> Option<&Blocked> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::blocking;

    #[test]
    fn test_focus_opens_on_the_list_when_there_is_something_in_it() {
        // The second assertion is the one that carries this. A decision with
        // two answers is satisfied by whichever answer a stub happens to
        // return, so a test naming only one of them can be green from the
        // moment it is written and never able to go red. Asking that the two
        // differ is the half no constant answer can pass.
        assert_eq!(where_focus_goes(3), WhereFocusGoes::TheList);
        assert_ne!(where_focus_goes(3), where_focus_goes(0));
    }

    #[test]
    fn test_focus_does_not_open_on_an_empty_list() {
        // Focus put into a list with no rows is focus with nothing to say and
        // nowhere to go: the arrow keys do nothing, and somebody working by
        // ear cannot tell that from a window that failed to load. The sentence
        // saying nobody is blocked is elsewhere on the window, so focus goes
        // to the one control that answers a keypress.
        assert_eq!(where_focus_goes(0), WhereFocusGoes::TheCloseButton);
    }

    #[test]
    fn test_the_empty_sentence_is_the_one_the_application_layer_wrote() {
        // One phrasing of one fact. A second sentence written here would drift
        // from the one the tests in `blocking` hold to its wording.
        assert_eq!(
            blocking::what_the_list_holds(&[]),
            blocking::NOBODY_IS_BLOCKED
        );
    }

    /// Two blocks, listed the way the window lists them.
    fn showing() -> Vec<Blocked> {
        let ada = blocking::just_this_sender("ada@example.com").expect("an address");
        let bob = blocking::just_this_sender("bob@example.com").expect("an address");
        blocking::everyone_blocked(
            "acct",
            &[
                blocking::a_rule_that_blocks("acct", &ada, "Junk", "t"),
                blocking::a_rule_that_blocks("acct", &bob, "Junk", "t"),
            ],
        )
    }

    #[test]
    fn test_the_row_a_press_acts_on_is_the_one_chosen() {
        // Both rows, not one. A lookup that always answered with the first row
        // would satisfy a test that only ever asked for the first, and the
        // person would unblock somebody they did not choose.
        let rows = showing();

        assert_eq!(the_row_chosen(&rows, 0), Some(&rows[0]));
        assert_eq!(the_row_chosen(&rows, 1), Some(&rows[1]));
    }

    #[test]
    fn test_nothing_chosen_is_not_a_row() {
        // `wxListCtrl` answers -1 when nothing is selected. Read as an index
        // it is not merely out of range, it is a number a bounds check on the
        // top end alone would wave through.
        //
        // The second assertion is what stops this passing against a lookup
        // that answers nothing to everything, which is a shape that refuses
        // every press and looks like a dead button.
        let rows = showing();

        assert_eq!(the_row_chosen(&rows, -1), None);
        assert!(the_row_chosen(&rows, 0).is_some());
    }

    #[test]
    fn test_a_row_past_the_end_of_the_list_is_not_a_row() {
        // The list is re-read after every unblock, so a selection held from
        // before a refresh can point past the end of what is there now.
        let rows = showing();

        assert_eq!(the_row_chosen(&rows, rows.len() as i64), None);
        assert!(the_row_chosen(&rows, rows.len() as i64 - 1).is_some());
    }
}

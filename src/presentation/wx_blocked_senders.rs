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
}

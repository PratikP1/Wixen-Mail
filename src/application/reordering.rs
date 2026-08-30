//! Moving one thing up or down a list somebody arranged by hand.
//!
//! D-14 gave accounts Alt+Shift+Up and Alt+Shift+Down; D-31 gives pinned
//! folders the same gesture rather than a second one, because it is the same
//! gesture: one way of rearranging anything in this tree. One gesture that
//! announced itself two ways would undo most of that, so the wording lives here
//! and both commands read it.
//!
//! # Nothing here reaches a server
//!
//! The order of somebody's accounts and the order of their pinned folders are
//! both facts about their own list. No mail protocol has a notion of either, so
//! there is nothing to send and nowhere to send it, and a command that quietly
//! opened a connection to record a preference would be doing something nobody
//! asked for on a metered or watched network. The two commands that call this
//! each carry the check that measures it.
//!
//! # Why it says something when it does nothing
//!
//! Moving the first of anything up cannot move it. For somebody working by ear,
//! a keystroke that does nothing and says nothing is indistinguishable from a
//! keystroke that was not received, so they press it again. It answers instead,
//! and the answer says where the thing already is.

/// Which way something is being moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Up,
    Down,
}

/// What moving something came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Moved {
    /// Every item's id, in the order they should now be stored in.
    ///
    /// The whole list rather than the two that swapped. Writing only the pair
    /// would leave the rest with no stored position, and a list half ordered by
    /// choice and half by arrival reorders itself the next time anything is
    /// added.
    pub order: Vec<String>,
    /// What to say, whether or not anything moved.
    pub say: String,
    /// Whether anything actually changed, so a caller can skip the writing.
    pub moved: bool,
}

/// Move one item of a hand-arranged list up or down it.
///
/// `items` is the list as `(id, name)` in the order it sits in now, `which` is
/// the id of the one to move, and `when_it_is_not_in_the_list` is what to say
/// when the cursor is not on one of them. That last is the only thing the two
/// callers word differently: everything else a move says is the same sentence
/// about a different noun, and saying it in one place is what stops the two
/// drifting apart.
pub fn moved(
    items: &[(String, String)],
    which: &str,
    direction: Move,
    when_it_is_not_in_the_list: &str,
) -> Moved {
    let order: Vec<String> = items.iter().map(|(id, _)| id.clone()).collect();
    let howmany = items.len();

    let Some(at) = items.iter().position(|(id, _)| id == which) else {
        // Nothing is chosen, or the chosen row is not one of these. Said rather
        // than ignored, because the alternative is a key that sometimes works
        // and never explains itself.
        return Moved {
            order,
            say: when_it_is_not_in_the_list.to_string(),
            moved: false,
        };
    };
    let name = items[at].1.clone();

    let swap_with = match direction {
        Move::Up if at == 0 => {
            return Moved {
                order,
                say: format!("{name} is already first of {howmany}."),
                moved: false,
            };
        }
        Move::Down if at + 1 == howmany => {
            return Moved {
                order,
                say: format!("{name} is already last of {howmany}."),
                moved: false,
            };
        }
        Move::Up => at - 1,
        Move::Down => at + 1,
    };

    let mut order = order;
    order.swap(at, swap_with);
    Moved {
        // One past the index, because somebody counting a list of three says
        // first, second, third rather than nought, one, two.
        say: format!("{name}, {} of {howmany}.", swap_with + 1),
        order,
        moved: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOT_IN_THE_LIST: &str = "Choose one first.";

    fn three() -> Vec<(String, String)> {
        [("a", "Work"), ("b", "Home"), ("c", "Old")]
            .iter()
            .map(|(id, name)| (id.to_string(), name.to_string()))
            .collect()
    }

    #[test]
    fn test_one_gesture_words_a_move_the_same_way_whatever_it_moved() {
        // The property the two commands share, asserted on this function
        // rather than twice on theirs. Both call it, so a second wording
        // cannot be introduced without changing this.
        let accounts = moved(&three(), "a", Move::Down, NOT_IN_THE_LIST);
        let pins = moved(&three(), "a", Move::Down, "Choose a folder first.");
        assert_eq!(accounts.say, pins.say);
        assert_eq!(accounts.say, "Work, 2 of 3.");
    }

    #[test]
    fn test_the_sentence_that_differs_is_the_one_about_nothing_being_chosen() {
        assert_eq!(
            moved(&three(), "nobody", Move::Up, "Choose a folder first.").say,
            "Choose a folder first."
        );
    }

    #[test]
    fn test_nothing_is_lost_or_repeated_by_a_move() {
        // The property worth having whatever the wording is. A swap that wrote
        // the same id into both places would pass every count elsewhere.
        for (which, direction) in [
            ("a", Move::Down),
            ("b", Move::Up),
            ("b", Move::Down),
            ("c", Move::Up),
        ] {
            let after = moved(&three(), which, direction, NOT_IN_THE_LIST);
            let mut sorted = after.order.clone();
            sorted.sort();
            assert_eq!(sorted, vec!["a", "b", "c"], "{which} {direction:?}");
        }
    }
}

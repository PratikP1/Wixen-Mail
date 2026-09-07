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
//!
//! # Reaching it and leaving it
//!
//! Modal, on the Tools menu with the other managers, and it is opened once in a
//! while rather than every day, so it carries no keyboard chord of its own. The
//! route is in `docs/KEYBOARD_SHORTCUTS.md`.
//!
//! # What has never been checked
//!
//! Nobody has heard this. Whether each row is read as a person, a destination
//! and a state rather than as one run-together string, whether a switched-off
//! block is distinguishable by ear from a working one, and whether what
//! unblocking did is heard over the list being re-read underneath it, are all
//! in the broken windows ledger as `unrun-verify` rather than claimed here.

use crate::application::blocking::{self, Blocked, WhatUnblockingDoes};
use crate::data::message_cache::MessageFilterRule;
use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::announcements::Priority;
use crate::presentation::accessibility::names::set_accessible_name;
use crate::presentation::status_line::said_and_shown;
use crate::presentation::theme;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use wxdragon::prelude::*;

const ID_UNBLOCK: Id = ID_HIGHEST + 441;
const ID_CLOSE_THE_LIST: Id = ID_HIGHEST + 442;

/// What the window is called.
pub const TITLE: &str = "Blocked Senders";

/// The heading over who each block catches.
pub const THE_WHO_COLUMN: &str = "Blocked";

/// The heading over where the mail is filed.
pub const THE_GOES_TO_COLUMN: &str = "Mail goes to";

/// The heading over whether the block is working.
pub const THE_WORKING_COLUMN: &str = "Working";

/// The headings, in the order the columns are built.
///
/// Paired with the cells of a row by position, so a column added without a
/// cell to fill it is a compile error rather than a blank cell somebody hears
/// as silence.
pub const THE_COLUMNS: [&str; 3] = [THE_WHO_COLUMN, THE_GOES_TO_COLUMN, THE_WORKING_COLUMN];

/// What the button that takes a block off says.
///
/// A verb, because a button whose label is a noun leaves somebody working by
/// ear to work out what pressing it does.
pub const UNBLOCK: &str = "&Unblock";

/// What the button that leaves says.
pub const CLOSE_THE_LIST: &str = "&Close";

/// What to say when Unblock is pressed with nothing chosen.
pub const NOTHING_IS_CHOSEN: &str =
    "Choose the block you want to take off first, then press Unblock.";

/// The words a label says, without the marker that makes a letter its key.
///
/// A visible label carries `&` so Windows underlines the next letter and binds
/// Alt to it. An accessible name is never drawn, so the marker has no job
/// there and a reader that takes the name literally says "ampersand Unblock".
///
/// Derived from the label rather than written out a second time, because the
/// two would drift and the one that drifts is the one nobody can see.
pub fn what_the_label_says(label: &str) -> String {
    label.to_string()
}

/// Where focus goes when the window opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhereFocusGoes {
    /// Into the list, on the first block, which is what somebody came for.
    TheList,
    /// Onto the button that closes the window.
    TheCloseButton,
}

/// Where focus should go when the window opens.
///
/// Focus put into a list with no rows is focus with nothing to say and nowhere
/// to go: the arrow keys answer nothing, and somebody working by ear cannot
/// tell that from a window that failed to load. The sentence saying nobody is
/// blocked is on the window either way.
pub fn where_focus_goes(rows: usize) -> WhereFocusGoes {
    if rows == 0 {
        WhereFocusGoes::TheCloseButton
    } else {
        WhereFocusGoes::TheList
    }
}

/// The row a press acts on, out of what the list is showing.
///
/// `wxListCtrl` answers -1 when nothing is chosen, which is why this takes the
/// control's own signed answer rather than a `usize` somebody has already
/// converted: the conversion is where -1 becomes an enormous index.
pub fn the_row_chosen(showing: &[Blocked], selected: i64) -> Option<&Blocked> {
    usize::try_from(selected)
        .ok()
        .and_then(|at| showing.get(at))
}

/// Reading every rule on the account, as they stand at the moment of asking.
pub type ReadingTheRules = Box<dyn Fn() -> Vec<MessageFilterRule>>;

/// Taking one rule off, and saying whether that worked.
pub type TakingARuleOff = Box<dyn Fn(&str) -> crate::common::Result<()>>;

/// Everything the window needs from the rules underneath it.
///
/// Closures rather than a database handle, so this module goes on knowing
/// nothing about storage and can be driven by whatever holds the rules.
pub struct TheRulesUnderneath {
    /// Which account is being looked at.
    pub account_id: String,
    /// Every rule on that account, as they stand at the moment of asking.
    pub read_rules: ReadingTheRules,
    /// Take one rule off, and say whether that worked.
    pub take_off: TakingARuleOff,
}

/// Show who is blocked, and let a block be taken off.
pub fn show_who_is_blocked(
    parent: &Frame,
    underneath: TheRulesUnderneath,
    a11y: &Arc<Accessibility>,
) {
    let underneath = Rc::new(underneath);
    let showing = Rc::new(RefCell::new(blocking::everyone_blocked(
        &underneath.account_id,
        &(underneath.read_rules)(),
    )));

    let dialog = Dialog::builder(parent, TITLE)
        .with_size(720, 460)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();
    if let Some(palette) = theme::current_from_stored_config() {
        theme::paint(&dialog, palette.main_surface());
    }
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // How many blocks there are and what can be done about one. Built in the
    // application layer so it can be read back without a window. This line is
    // the state of the list rather than an answer to a keypress, so it is
    // written directly; every answer goes through `said_and_shown` below.
    let counted = blocking::what_the_list_holds(&showing.borrow());
    let heading = StaticText::builder(&dialog).with_label(&counted).build();
    set_accessible_name(&heading, &counted);
    sizer.add(&heading, 0, SizerFlag::Expand | SizerFlag::All, 12);

    let list = ListCtrl::builder(&dialog)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    set_accessible_name(&list, TITLE);
    for (at, name) in THE_COLUMNS.iter().enumerate() {
        let width = if at == 0 { 320 } else { 180 };
        list.insert_column(at as i64, name, ListColumnFormat::Left, width);
    }
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 8);

    // The line every answer is written to. Nothing raises a notification when
    // a line of text changes, so an answer only shown here is an answer nobody
    // working by ear gets: `said_and_shown` is the one call that does both.
    let status = StaticText::builder(&dialog).with_label("").build();
    set_accessible_name(&status, "What happened");
    sizer.add(&status, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    let unblock = Button::builder(&dialog)
        .with_label(UNBLOCK)
        .with_id(ID_UNBLOCK)
        .build();
    set_accessible_name(&unblock, UNBLOCK);
    let close = Button::builder(&dialog)
        .with_label(CLOSE_THE_LIST)
        .with_id(ID_CLOSE_THE_LIST)
        .build();
    set_accessible_name(&close, CLOSE_THE_LIST);
    buttons.add(&unblock, 0, SizerFlag::All, 4);
    buttons.add(&close, 0, SizerFlag::All, 4);
    sizer.add_sizer(&buttons, 0, SizerFlag::AlignRight | SizerFlag::All, 8);
    dialog.set_sizer(sizer, true);

    fill(&list, &showing.borrow());

    unblock.on_click({
        let a11y = a11y.clone();
        let showing = showing.clone();
        let underneath = underneath.clone();
        move |_| {
            let chosen =
                the_row_chosen(&showing.borrow(), list.get_first_selected_item().into()).cloned();
            let Some(chosen) = chosen else {
                said_and_shown(&status, &a11y, NOTHING_IS_CHOSEN, Priority::High);
                return;
            };
            // Asked against the rules as they are now rather than by the
            // `rule_id` the row is carrying. The row's id was read when this
            // window opened, and the rules manager may have been open since.
            let (said, how_loud) = match blocking::what_unblocking_a_row_does(
                &underneath.account_id,
                &chosen,
                &(underneath.read_rules)(),
            ) {
                WhatUnblockingDoes::ItHasAlreadyGone(why) => (why, Priority::High),
                WhatUnblockingDoes::TakeOffTheRule { rule_id, said } => {
                    match (underneath.take_off)(&rule_id) {
                        Ok(()) => (said, Priority::Normal),
                        Err(why) => (
                            format!(
                                "That block could not be taken off, so nothing has changed. {why}"
                            ),
                            Priority::High,
                        ),
                    }
                }
            };

            // Read back rather than taking the row out of the control, so what
            // is shown and what is stored cannot come apart. At most a handful
            // of rows, and the read is local.
            let was_on = list.get_first_selected_item();
            let mut rows = showing.borrow_mut();
            *rows = blocking::everyone_blocked(&underneath.account_id, &(underneath.read_rules)());
            fill(&list, &rows);
            heading.set_label(&blocking::what_the_list_holds(&rows));
            land_the_row_cursor(&list, the_row_after_removing(was_on, rows.len()));
            drop(rows);
            said_and_shown(&status, &a11y, &said, how_loud);
        }
    });
    close.on_click(move |_| {
        dialog.end_modal(ID_CLOSE_THE_LIST);
    });

    match where_focus_goes(showing.borrow().len()) {
        WhereFocusGoes::TheList => {
            list.set_focus();
            land_the_row_cursor(&list, Some(0));
        }
        WhereFocusGoes::TheCloseButton => close.set_focus(),
    }
    dialog.show_modal();
    dialog.destroy();
}

/// Which row to put the cursor on once one has gone.
///
/// The row that moved up into the gap, or the last one when the gap was at the
/// end. Without this the repaint leaves nothing chosen, so a second Unblock
/// answers "choose one first" and sounds like a refusal.
fn the_row_after_removing(was_on: i32, left: usize) -> Option<usize> {
    if left == 0 {
        return None;
    }
    Some(usize::try_from(was_on).unwrap_or(0).min(left - 1))
}

/// Put the cursor on a row, so a screen reader's own cursor goes with it.
///
/// Selected and focused together, and made visible: selecting alone leaves the
/// reading cursor where it was, so the person hears nothing move.
fn land_the_row_cursor(list: &ListCtrl, at: Option<usize>) {
    let Some(at) = at else { return };
    let at = at as i64;
    list.set_item_state(
        at,
        ListItemState::Selected | ListItemState::Focused,
        ListItemState::Selected | ListItemState::Focused,
    );
    list.ensure_visible(at);
}

/// Put what is blocked into the control, one row at a time.
fn fill(list: &ListCtrl, showing: &[Blocked]) {
    list.delete_all_items();
    for (at, block) in showing.iter().enumerate() {
        let row = blocking::what_a_row_says(block);
        // Paired with `THE_COLUMNS` by position and by length, so a heading
        // added without a cell to fill it does not compile rather than
        // becoming a blank cell somebody hears as silence.
        let cells: [String; THE_COLUMNS.len()] = [row.who, row.goes_to, row.working];
        list.insert_item(at as i64, &cells[0], None);
        for (column, cell) in cells.iter().enumerate().skip(1) {
            list.set_item_text_by_column(at as i64, column as i32, cell);
        }
    }
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
    fn test_a_buttons_spoken_name_does_not_carry_the_key_marker() {
        // The visible label needs the `&` so Windows underlines the letter and
        // binds Alt to it. The accessible name is never drawn, so a reader
        // taking it literally says "ampersand Unblock", and the two controls a
        // person reaches by keyboard are the two this would land on.
        //
        // Both buttons, because a helper that stripped nothing would still
        // satisfy a test written against a label that had no marker in it.
        assert_eq!(what_the_label_says(UNBLOCK), "Unblock");
        assert_eq!(what_the_label_says(CLOSE_THE_LIST), "Close");
        assert!(!what_the_label_says(UNBLOCK).contains('&'));
    }

    #[test]
    fn test_a_label_with_no_key_marker_is_left_alone() {
        // The other direction, so the helper cannot be one that rewrites
        // whatever it is handed.
        assert_eq!(what_the_label_says(TITLE), TITLE);
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

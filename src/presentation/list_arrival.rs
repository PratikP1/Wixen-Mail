//! Focus arriving on a list with no row under the cursor lands on a row.
//!
//! The tester on 2026-09-18 (#87), under NVDA: Tab from the folder tree to
//! the message list lands on the list itself, read as "list" with no row,
//! and Down then lands on the first row. Opening a folder loads the rows
//! and, by 10-02's rule, selects nothing while focus is in the tree, so a
//! screen reader's cursor is not taken away from the tree; and nothing
//! landed a row when focus later arrived.
//!
//! # Why the focus event, and not a key
//!
//! Tab is the toolkit's own navigation, F6 is this program's and a click is
//! the control's; the list's `SET_FOCUS` event is the one path all three
//! share, so the landing is bound there. It lands only when the control
//! holds no focused item, which is the case of arriving on a list that was
//! filled while focus was elsewhere; coming back to a list that holds a row
//! moves nothing, so a screen reader's cursor is never moved under the
//! person by a focus event alone.
//!
//! # Why the row is set inside the handler
//!
//! The event is skipped, so the control's own `WM_SETFOCUS` handling runs
//! after this closure. `comctl32` raises the focus event a screen reader
//! reads the row from as part of that handling, naming the focused item when
//! there is one and the list itself when there is none. Setting the row
//! before the control's handling runs is what makes the row, and not the
//! list, the thing that is read on arrival.
//! `tests/tab_from_the_tree_lands_on_the_newest_message.rs` moves focus from
//! a built tree to a built list through this binding and reads the focused
//! item back from the control, under a hook recording which child each
//! focus event named. Measured 2026-09-19: an arrival on a list holding no
//! row raised the list itself, then row 0 twice; coming back to a list
//! holding row 1 raised the list itself, then row 1. The first event of
//! each is the system's own for the window, raised by `SetFocus` before
//! `WM_SETFOCUS` is sent, and every list gets it; the row's is the one the
//! tester did not hear.

use wxdragon::prelude::*;

/// Bind the list's focus event: when focus arrives and no item is focused,
/// `choose` names the row to land on, which is selected, focused and brought
/// into view, or, when it names none, `on_empty` says so. When an item is
/// already focused nothing happens.
pub fn wire<C, E>(list: &ListCtrl, choose: C, on_empty: E)
where
    C: Fn() -> Option<usize> + 'static,
    E: Fn() + 'static,
{
    let owner = *list;
    list.bind_internal(EventType::SET_FOCUS, move |event| {
        // Skipped, so the control's own focus handling runs after this,
        // with the row already set.
        event.skip(true);
        if the_focused_row_of(&owner).is_some() {
            return;
        }
        match choose() {
            Some(row) => land_on(&owner, row),
            None => on_empty(),
        }
    });
}

/// The row the control holds as focused, or nothing.
fn the_focused_row_of(list: &ListCtrl) -> Option<usize> {
    usize::try_from(list.get_next_item(-1, ListNextItemFlag::All, ListItemState::Focused)).ok()
}

/// Select and focus `row` and bring it into view.
///
/// The control holds no focused row when this runs, so there is no state
/// to clear first; the setter in the window's file that lands after a
/// removal clears because the control may already hold the row there.
fn land_on(list: &ListCtrl, row: usize) {
    let both = ListItemState::Selected | ListItemState::Focused;
    list.set_item_state(row as i64, both, both);
    list.ensure_visible(row as i64);
}

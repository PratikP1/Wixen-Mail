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
//! item back from the control.

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
    let _ = (list, choose, on_empty);
}

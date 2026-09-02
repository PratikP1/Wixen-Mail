//! Which row of a tree control is selected, said as a position.
//!
//! A dialog that needs to know what a tree row means has two ways to find out.
//! It can hang the meaning off the row, which is what `set_custom_data` and
//! `append_item_with_data` look like they do; they actually put it in a
//! process-global map in `wxdragon` and store the map key on the row, and
//! nothing ever takes it out again, so every row of every opening stays there
//! for the life of the process. Or it can keep the meanings in a vector beside
//! the control and pair them by position, which is what
//! `WxUIState::tree_rows` does for the folder tree and what the two dialogs
//! using this module do.
//!
//! Pairing by position needs the position, and that is the part with no
//! obvious answer: `wxdragon`'s `TreeItemId` has no equality and does not
//! expose its pointer, so the selected row cannot be compared against a row
//! the caller is holding. `folder_tree::where_a_row_sits` answers it by
//! matching label chains, and that is not good enough for a dialog that
//! decides where mail is moved: two accounts with the same name produce the
//! same chain and the match takes the first, so the message goes somewhere
//! nobody named and everything reports success.
//!
//! `TreeCtrl::is_selected` asks the control instead, one row at a time, and
//! needs no comparison at all. Walking the rows in a fixed order and asking
//! each one gives an exact position.
//!
//! # What makes a position meaningful
//!
//! A position is only an answer while the tree and the vector beside it agree
//! on what the rows are. Both dialogs here build their tree once and never
//! touch it again while it is open: nothing is inserted, removed or re-sorted,
//! and each pushes its vector entry in the same statement that appends the
//! row. A tree that changed under a held vector would need something else.
//!
//! # Where this is tested
//!
//! Not here. Every function below needs a real `TreeCtrl` with real rows in
//! it. The budget is one live window per process, which is a number rather
//! than a prohibition: a test may build one and read it back, and each of
//! these has a process of its own under `tests/`.
//! `tests/tree_rows_leave_no_registry_entry.rs` builds both dialogs and counts
//! what they left in the registry, and
//! `tests/tree_dialogs_resolve_the_row_somebody_is_on.rs` chooses a row in
//! each and reads the answer back.

use wxdragon::prelude::*;

/// Every row of `tree`, in the order a depth-first walk meets them.
///
/// The root is not among them. It is not a row somebody chose, it is the thing
/// the rows hang off, and both callers here mean something else by it: the
/// whole conversation, or no destination at all.
pub fn rows_in_walk_order(tree: &TreeCtrl) -> Vec<TreeItemId> {
    let Some(root) = tree.get_root_item() else {
        return Vec::new();
    };

    let mut rows = Vec::new();
    let mut waiting: Vec<TreeItemId> = children_of(tree, &root).into_iter().rev().collect();
    while let Some(item) = waiting.pop() {
        for child in children_of(tree, &item).into_iter().rev() {
            waiting.push(child);
        }
        rows.push(item);
    }
    rows
}

/// Where the selected row sits in that walk.
///
/// `None` when the root is selected, when nothing is, or when there is no
/// root. All three mean the same thing to both callers: no row was chosen.
pub fn where_the_selection_sits(tree: &TreeCtrl) -> Option<usize> {
    rows_in_walk_order(tree)
        .iter()
        .position(|row| tree.is_selected(row))
}

/// One item's children, in the order they were appended.
///
/// The cookie dance is `wxTreeCtrl`'s own way of iterating children and there
/// is no other; `get_first_child` hands back a cookie that `get_next_child`
/// needs.
fn children_of(tree: &TreeCtrl, item: &TreeItemId) -> Vec<TreeItemId> {
    let mut found = Vec::new();
    if let Some((first, mut cookie)) = tree.get_first_child(item) {
        found.push(first);
        while let Some(next) = tree.get_next_child(item, &mut cookie) {
            found.push(next);
        }
    }
    found
}

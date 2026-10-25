//! A list cell reads back whole, and a tree frees what its rows held.
//!
//! Two defects of wxdragon 0.9.17, both fixed by the move to 0.9.23 and both
//! held here by a live control rather than by reading the toolkit's source.
//!
//! A list cell. `ListCtrl::get_item_text` asked wxWidgets how long the text
//! was, allocated exactly that many bytes, and the C++ side, which always
//! writes a NUL, wrote it over the last character: "Message text" came back
//! as "Message tex\0" (ledgers 28 and 159). 0.9.20 allocates one more byte
//! (upstream #205).
//!
//! A tree's rows. Data hung on a row lives in wxdragon's process-global
//! registry, `wxdragon::widgets::item_data`, keyed by a number stored on the
//! row. At 0.9.17 `delete_all_items` went straight to wxWidgets and freed
//! nothing, and `delete` freed the row it was handed and none of the rows
//! under it, so every such row stayed in the registry for the life of the
//! process (ledger 41). 0.9.23 frees each row's entry before the row goes
//! (upstream #214 and #215). Counted the way
//! `tests/tree_rows_leave_no_registry_entry.rs` counts it: one throwaway entry
//! stored before and one after fence off exactly the keys issued between
//! them, and each of those is asked whether it is still live.
//!
//! One `#[test]` function, which spends this target's one `wxdragon::main`
//! per process; a second in the same process asserts "initializing twice?"
//! and hangs. Every check gathers what was wrong into one list, so a failure
//! says which of the three it was.

use std::sync::{Arc, Mutex};
use wxdragon::prelude::*;
use wxdragon::widgets::item_data::{get_item_data, remove_item_data, store_item_data};

/// The words a cell is given and must read back.
const A_CELL_SAYS: &str = "Message text";

/// How many registry keys issued strictly between two fences are still live.
/// The fences are this file's own entries and are removed here, so the next
/// fenced range does not start behind a live key of ours.
fn live_between(opened: u64, closed: u64) -> usize {
    let live = ((opened + 1)..closed)
        .filter(|key| get_item_data(*key).is_some())
        .count();
    remove_item_data(opened);
    remove_item_data(closed);
    live
}

/// A branch holding two rows, each row and the branch carrying data, under
/// a root of its own. Returns the branch.
fn a_branch_with_rows(tree: &TreeCtrl, root: &TreeItemId) -> Option<TreeItemId> {
    let branch = tree.append_item_with_data(root, "person@example.com", 1_u32, None, None)?;
    tree.append_item_with_data(&branch, "Archive", 2_u32, None, None)?;
    tree.append_item_with_data(&branch, "Receipts", 3_u32, None, None)?;
    Some(branch)
}

/// What is wrong with the list cell, if anything.
fn what_the_cell_read(frame: &Frame) -> Option<String> {
    let list = ListCtrl::builder(frame)
        .with_style(ListCtrlStyle::Report)
        .build();
    list.insert_column(0, "Looks at", ListColumnFormat::Left, 200);
    list.insert_item(0, A_CELL_SAYS, None);
    let read_back = list.get_item_text(0, 0);
    list.destroy();
    (read_back != A_CELL_SAYS).then(|| {
        format!(
            "a list cell given {A_CELL_SAYS:?} read back {read_back:?}, so a screen reader \
             reading it through the list hears it cut short"
        )
    })
}

/// What `delete_all_items` left in the registry, if anything.
fn what_deleting_every_row_left(frame: &Frame) -> Option<String> {
    let tree = TreeCtrl::builder(frame).build();
    let opened = store_item_data(());
    let built = tree
        .add_root("Accounts", None, None)
        .and_then(|root| a_branch_with_rows(&tree, &root));
    tree.delete_all_items();
    let closed = store_item_data(());
    let live = live_between(opened, closed);
    tree.destroy();
    match (built, live) {
        (None, _) => Some("the tree for delete_all_items could not be built".to_string()),
        (Some(_), 0) => None,
        (Some(_), live) => Some(format!(
            "delete_all_items left {live} of three rows' data in the registry"
        )),
    }
}

/// What deleting a branch with `delete` left in the registry, if anything.
fn what_deleting_a_branch_left(frame: &Frame) -> Option<String> {
    let tree = TreeCtrl::builder(frame).build();
    let root = tree.add_root("Accounts", None, None);
    let opened = store_item_data(());
    let branch = root
        .as_ref()
        .and_then(|root| a_branch_with_rows(&tree, root));
    if let Some(branch) = &branch {
        tree.delete(branch);
    }
    let closed = store_item_data(());
    let live = live_between(opened, closed);
    tree.destroy();
    match (branch, live) {
        (None, _) => Some("the tree for delete could not be built".to_string()),
        (Some(_), 0) => None,
        (Some(_), live) => Some(format!(
            "deleting a branch left {live} of three rows' data in the registry"
        )),
    }
}

#[test]
fn test_a_list_cell_reads_whole_and_a_tree_frees_what_its_rows_held() {
    // The fenced count is first shown to see a planted entry, in sequence
    // rather than in a companion test, which would run on another thread and
    // issue keys inside the fences below.
    let opened = store_item_data(());
    let planted = store_item_data("a row's data nobody freed");
    let closed = store_item_data(());
    assert_eq!(
        live_between(opened, closed),
        1,
        "the fenced count cannot see an entry that is in the registry, so a zero \
         from it below would say nothing about either tree"
    );
    remove_item_data(planted);

    let wrong: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |_| {
            let frame = Frame::builder().build();
            let found: Vec<String> = [
                what_the_cell_read(&frame),
                what_deleting_every_row_left(&frame),
                what_deleting_a_branch_left(&frame),
            ]
            .into_iter()
            .flatten()
            .collect();
            *wrong.lock().unwrap() = found;
            wxdragon::call_after(Box::new(|| {
                if let Some(app) = wxdragon::get_app_instance() {
                    app.exit_main_loop();
                }
            }));
        })
    };
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");

    let wrong = wrong.lock().unwrap();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

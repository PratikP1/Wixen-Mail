//! A row drawn on a real control is found again by the identity beside it.
//!
//! The whole of the folder tree's memory rests on one assumption: that the rows
//! `folder_tree::rows` decides, drawn in order by `fill_the_tree`, can be paired
//! back to their identities afterwards. Nothing is hung off the control to do
//! it, and deliberately: `wxdragon`'s tree item data goes into a process-global
//! map that `delete_all_items` does not clear and `cleanup_all_custom_data`
//! never clears for a leaf, so a tree rebuilt on a timer would leak an entry per
//! folder per sync for the life of the process.
//!
//! So the pairing is done by the words above a row, and that is only sound if
//! two things hold on a real control rather than in a unit test: appending at a
//! depth really does put a row under the right parent, and walking up with
//! `get_item_parent` really does retrace it. Both are wxMSW's behaviour under
//! the binding, so both are asked here of a real `TreeCtrl`.
//!
//! The case that matters is the one plan 01-03 created: two folders whose leaf
//! is the same word, under different parents. Before this they were two rows
//! reading identically and one entry in the map that opens them.
//!
//! One `#[test]` function, for the reason `tests/tree_selection_raises.rs`
//! gives: wxWidgets supports one application per process and `cargo test` runs
//! each file under `tests/` as its own process.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use wixen_mail::application::folder_settings::UnreadOnAParent;
use wixen_mail::presentation::folder_tree::{
    AccountInTheTree, FolderInTheTree, WhichRow, rows as folder_rows,
};
use wixen_mail::presentation::wx_app::{fill_the_tree, land_the_folder_cursor, select_row};
use wxdragon::prelude::*;

fn folder(id: i64, path: &str, parent: Option<i64>) -> FolderInTheTree {
    FolderInTheTree {
        account: "acc".to_string(),
        id,
        path: path.to_string(),
        name: path.rsplit('/').next().unwrap_or(path).to_string(),
        unread: 0,
        parent,
    }
}

#[test]
fn test_a_row_drawn_on_a_real_control_is_found_again_by_its_identity() {
    let found: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let found = found.clone();
        wxdragon::main(move |app| {
            let frame = Frame::builder().build();
            let tree = TreeCtrl::builder(&frame).build();

            let rows = folder_rows(
                &[AccountInTheTree {
                    id: "acc".to_string(),
                    name: "Work".to_string(),
                }],
                &[
                    folder(1, "Archive", None),
                    folder(2, "Archive/2026", Some(1)),
                    folder(3, "Sent", None),
                    folder(4, "Sent/2026", Some(3)),
                ],
                &[],
                &[],
                &[],
                UnreadOnAParent::default(),
                &HashSet::new(),
            );

            let root = tree.add_root("Mail Folders", None, None).expect("a root");
            fill_the_tree(&tree, &root, &rows, &std::collections::HashSet::new());
            tree.expand(&root);

            let mut said = found.lock().unwrap();

            // Both folders called 2026 really are drawn, one under each parent.
            said.push(format!(
                "rows drawn: {}",
                rows.iter().filter(|row| row.label == "2026").count()
            ));

            // Put the cursor on the second one, by identity, then ask the
            // control which row it is on. The answer has to be that same one
            // and not the first, which reads identically.
            let second = WhichRow::Folder {
                account: "acc".to_string(),
                path: "Sent/2026".to_string(),
            };
            said.push(format!(
                "selected: {}",
                select_row(&tree, &rows, &second.stored())
            ));
            said.push(format!(
                "reads back as: {:?}",
                wixen_mail::presentation::wx_app::the_folder_row_the_cursor_was_on(&tree, &rows)
            ));

            // And a rebuild after a rename keeps the cursor on it. The row's
            // words change and its identity does not, which is the whole of
            // D-25.
            let renamed = folder_rows(
                &[AccountInTheTree {
                    id: "acc".to_string(),
                    name: "Work at home".to_string(),
                }],
                &[
                    folder(1, "Archive", None),
                    folder(2, "Archive/2026", Some(1)),
                    folder(3, "Sent", None),
                    folder(4, "Sent/2026", Some(3)),
                ],
                &[],
                &[],
                &[],
                UnreadOnAParent::default(),
                &HashSet::new(),
            );
            let was_on =
                wixen_mail::presentation::wx_app::the_folder_row_the_cursor_was_on(&tree, &rows);
            tree.delete_all_items();
            let root = tree.add_root("Mail Folders", None, None).expect("a root");
            fill_the_tree(&tree, &root, &renamed, &std::collections::HashSet::new());
            tree.expand(&root);
            land_the_folder_cursor(&tree, &root, &renamed, was_on.as_deref());
            said.push(format!(
                "after the rename: {:?}",
                wixen_mail::presentation::wx_app::the_folder_row_the_cursor_was_on(&tree, &renamed)
            ));

            drop(said);
            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");

    let said = found.lock().unwrap();
    let second = WhichRow::Folder {
        account: "acc".to_string(),
        path: "Sent/2026".to_string(),
    }
    .stored();

    assert_eq!(
        said[0], "rows drawn: 2",
        "both folders called 2026 are shown"
    );
    assert_eq!(said[1], "selected: true", "the cursor went onto the second");
    assert_eq!(
        said[2],
        format!("reads back as: Some({second:?})"),
        "the control's own row resolved to the folder under Sent, not to the \
         one under Archive that reads the same"
    );
    assert_eq!(
        said[3],
        format!("after the rename: Some({second:?})"),
        "renaming the account branch left the cursor where it was"
    );
}

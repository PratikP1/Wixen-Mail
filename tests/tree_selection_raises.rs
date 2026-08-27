//! Choosing a tree row in code raises the same event as choosing it by hand.
//!
//! The View menu's All Inboxes moves the cursor onto the first row of the mail
//! folder tree and lets that tree's own selection handler do the loading. That
//! is one path into a combined inbox rather than two that could disagree, and
//! the whole of it rests on one assumption: that `select_item` raises
//! `EVT_TREE_SEL_CHANGED` rather than only moving the cursor.
//!
//! If it does not, the menu command moves a cursor and loads nothing, which is
//! the exact shape of defect this project keeps writing down: a handler
//! written, an id allocated, a test guarding the id, and nothing ever raising
//! the event. Reading the C++ shim showed it calls `wxTreeCtrl::SelectItem`
//! and stopped short of answering the question, because what that generates is
//! decided by wxMSW underneath. So it is asked here of a real control.
//!
//! One `#[test]` function, for the reason `tests/theme_reach.rs` gives:
//! wxWidgets supports one application per process and `cargo test` runs each
//! file under `tests/` as its own process.

use std::sync::{Arc, Mutex};
use wxdragon::prelude::*;

#[test]
fn test_choosing_a_row_in_code_raises_the_selection_event() {
    let heard: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let heard = heard.clone();
        wxdragon::main(move |app| {
            let frame = Frame::builder().build();
            let tree = TreeCtrl::builder(&frame).build();

            let root = tree.add_root("Mail Folders", None, None).expect("a root");
            let all_inboxes = tree
                .append_item(&root, "All Inboxes", None, None)
                .expect("the combined inbox row");
            tree.append_item(&root, "Inbox", None, None);
            tree.expand(&root);

            tree.on_selection_changed({
                let heard = heard.clone();
                move |event| {
                    if let Some(item) = event.get_item()
                        && let Some(text) = tree.get_item_text(&item)
                    {
                        heard.lock().unwrap().push(text);
                    }
                }
            });

            // The call the menu command makes.
            tree.select_item(&all_inboxes);

            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");

    let heard = heard.lock().unwrap();
    assert!(
        heard.iter().any(|row| row == "All Inboxes"),
        "selecting a row in code raised no selection event, so the View menu's \
         All Inboxes moves the cursor and loads nothing. Heard: {heard:?}"
    );
}

//! Whether the two tree dialogs leave anything behind in wxdragon's registry.
//!
//! `set_custom_data` and `append_item_with_data` do not put the data on the
//! item. They put it in `wxdragon::widgets::item_data::ITEM_DATA_REGISTRY`, a
//! process-global `HashMap` behind a `LazyLock`, and store the map key on the
//! item. Nothing in wxWidgets knows about that map, so destroying the control
//! removes nothing from it.
//!
//! The record above `test_no_row_of_a_tree_hangs_its_identity_off_the_control`
//! in `guards/guards.toml` says the leak has no observable behaviour and that a
//! source read is therefore the only thing that can see it. That is not true,
//! and this file is the counter-example. The registry hands out its keys from a
//! monotonic counter, and `get_item_data` and `store_item_data` are both public.
//! So storing one throwaway entry before a dialog is built and another after it
//! fences off exactly the keys that build issued, and asking the registry for
//! each of them says how many are still live. That number is the leak, counted.
//!
//! One `#[test]` function, for the reason `tests/theme_reach.rs` gives at
//! length: wxWidgets supports one application per process, a second
//! `wxdragon::main` asserts "initializing twice?" and hangs, and `cargo test`
//! runs each file under `tests/` as its own process. Both dialogs are built
//! inside the one `wxdragon::main` here, fenced separately, so a failure says
//! which of the two leaked rather than only that something did.

use std::sync::{Arc, Mutex};
use wixen_mail::application::destinations::{Branch, Destination, Moving};
use wixen_mail::presentation::{wx_destination, wx_thread_view};
use wxdragon::prelude::*;
use wxdragon::widgets::item_data::{get_item_data, remove_item_data, store_item_data};

/// How many of the registry keys issued strictly between two fences are still
/// live, named by the dialog that issued them.
///
/// The fences themselves are removed here rather than counted: they are this
/// file's own entries and leaving them in would make the next fenced range
/// start behind a live key of ours.
fn what_was_left_between(dialog: &'static str, opened: u64, closed: u64) -> (&'static str, usize) {
    let entries = ((opened + 1)..closed)
        .filter(|key| get_item_data(*key).is_some())
        .count();
    remove_item_data(opened);
    remove_item_data(closed);
    (dialog, entries)
}

/// Two accounts, one of them with no places at all, which is the shape the
/// destination picker has to keep answering correctly however it resolves a
/// selection.
fn branches() -> Vec<Branch> {
    vec![
        Branch {
            account_id: "acct-1".to_string(),
            account_name: "person@example.com".to_string(),
            places: vec![
                Destination {
                    name: "Archive".to_string(),
                    id: "acct-1/archive".to_string(),
                    account_id: "acct-1".to_string(),
                    depth: 0,
                },
                Destination {
                    name: "Receipts".to_string(),
                    id: "acct-1/receipts".to_string(),
                    account_id: "acct-1".to_string(),
                    depth: 0,
                },
            ],
        },
        Branch {
            account_id: "acct-2".to_string(),
            account_name: "person@work.example".to_string(),
            places: Vec::new(),
        },
    ]
}

/// A conversation whose third message replies to the first, so the order the
/// nodes arrive in is not the order the built tree is walked in.
fn nodes() -> Vec<wx_thread_view::ThreadNode> {
    let message = |message_id: i64, sender: &str, parent: Option<usize>, depth: usize| {
        wx_thread_view::ThreadNode {
            message_id,
            uid: message_id as u32,
            sender: sender.to_string(),
            subject: "Quarterly report".to_string(),
            date: "2026-07-26".to_string(),
            read: true,
            depth,
            parent,
        }
    };
    vec![
        message(11, "Ada Lovelace", None, 0),
        message(22, "Grace Hopper", None, 0),
        message(33, "Katherine Johnson", Some(0), 1),
    ]
}

#[test]
fn test_neither_tree_dialog_leaves_an_entry_in_the_process_global_registry() {
    // Proving the measurement, as the first act of this one test function
    // rather than as a companion `#[test]` beside it. A companion would run on
    // another thread and issue registry keys inside the fences below, so its
    // own planted entry would be counted as a dialog's leak. Here it is
    // sequenced instead, and it answers the question a companion exists to
    // answer: a fenced count that always said zero would pass the assertion at
    // the bottom of this file without having looked at anything.
    let opened = store_item_data(());
    let planted = store_item_data("a row's identity, hung off a control");
    let closed = store_item_data(());
    assert_eq!(
        what_was_left_between("planted entry", opened, closed),
        ("planted entry", 1),
        "the fenced count cannot see an entry that is really in the registry, \
         so a zero from it below would say nothing about either dialog"
    );
    remove_item_data(planted);

    let left: Arc<Mutex<Vec<(&'static str, usize)>>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let left = left.clone();
        wxdragon::main(move |app| {
            let frame = Frame::builder().build();
            let mut found = Vec::new();

            let opened = store_item_data(());
            let (destination, _tree) = wx_destination::build_destination_dialog(
                &frame,
                Moving::Message,
                false,
                &branches(),
                None,
                None,
            );
            let closed = store_item_data(());
            found.push(what_was_left_between("destination picker", opened, closed));
            destination.destroy();

            let opened = store_item_data(());
            let (conversation, _tree, _chosen) =
                wx_thread_view::build_thread_dialog(&frame, "Quarterly report", &nodes(), None)
                    .expect("three nodes give the conversation tree a root");
            let closed = store_item_data(());
            found.push(what_was_left_between("conversation view", opened, closed));
            conversation.destroy();

            *left.lock().unwrap() = found;

            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");

    let left = left.lock().unwrap();
    assert_eq!(left.len(), 2, "both dialogs should have been measured");
    let leaking: Vec<String> = left
        .iter()
        .filter(|(_, entries)| *entries > 0)
        .map(|(dialog, entries)| format!("the {dialog} left {entries}"))
        .collect();
    assert!(
        leaking.is_empty(),
        "these hung a row's data off the control, and the entries are still in \
         wxdragon's process-global registry after the dialog was destroyed. \
         Nothing clears them: `cleanup_all_custom_data` walks the tree without \
         removing anything, and `delete_all_items` goes straight to wxWidgets, \
         which has never heard of the map. So this is one entry per row per \
         opening, for the life of the process: {leaking:?}"
    );
}

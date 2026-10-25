//! Whether a warning the window toolkit writes about itself reaches the log
//! file, and whether a line from any other library stays out of it.
//!
//! wxDragon reports through the `log` crate, not through `tracing`: a tree item
//! id the C++ side returned that is not valid, and from 0.9.23 a handler call
//! it skipped because the handler was already running and a panic it caught in
//! a callback from C++. The program's logger is a `tracing` subscriber, and
//! nothing reaches it from `log` unless a bridge is installed, so until
//! 2026-10-25 every one of those lines was dropped.
//!
//! The warning here is a real one, from wxDragon's own macro: asking a tree
//! with nothing in it for its selection. `wxd_TreeCtrl_GetSelection` wraps
//! whatever wxWidgets answers, valid or not, and wxDragon warns as it refuses
//! the invalid id. `log` is not a dependency of this package, so the warning
//! cannot be written here by hand, and it does not need to be.
//!
//! One test function, which spends this target's one `wxdragon::main` per
//! process; a second asks "initializing twice?" and hangs
//! (`tests/theme_reach.rs` says why at length). The global subscriber and the
//! `log` logger are each set once per process as well.

use std::path::Path;
use wixen_mail::common::logging::{LogLevel, LoggerConfig, init_logging};
use wxdragon::prelude::*;

/// Every log file in `folder`, read and joined.
fn everything_written_in(folder: &Path) -> String {
    let mut files: Vec<_> = std::fs::read_dir(folder)
        .expect("the log folder can be listed")
        .map(|entry| entry.expect("a log folder entry").path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|file| std::fs::read_to_string(file).expect("a log file can be read"))
        .collect()
}

/// Asks a tree with nothing in it for its selection, which is the call that
/// makes wxDragon warn, and ends the loop.
fn ask_an_empty_tree_for_its_selection() {
    let frame = Frame::builder().build();
    let tree = TreeCtrl::builder(&frame).build();
    let _nothing = tree.get_selection();
    frame.destroy();

    wxdragon::call_after(Box::new(|| {
        if let Some(app) = wxdragon::get_app_instance() {
            app.exit_main_loop();
        }
    }));
}

#[test]
fn test_a_warning_wxdragon_writes_reaches_the_log_file() {
    // RUST_LOG replaces the filter this test is about, so a run under it
    // would be measuring somebody's environment rather than the program.
    assert!(
        std::env::var_os("RUST_LOG").is_none(),
        "RUST_LOG is set, and it overrides the filter this test reads; unset it"
    );

    let folder = tempfile::tempdir().expect("a temporary folder");
    let guard = init_logging(LoggerConfig {
        level: LogLevel::Info,
        log_to_file: true,
        log_dir: folder.path().to_path_buf(),
        log_file_prefix: "wixen-mail".to_string(),
        console_logging: false,
    })
    .expect("logging starts");

    tracing::warn!(
        target: "some_library",
        "a warning from another library, which the filter must refuse"
    );
    tracing::info!(
        target: "wxdragon::widgets::treectrl",
        "an info line from the toolkit, below the warn it is admitted at"
    );

    let result = wxdragon::main(|_| ask_an_empty_tree_for_its_selection());
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");

    // The writer runs on a thread of its own, and dropping its guard is what
    // flushes it.
    drop(guard);
    let written = everything_written_in(folder.path());

    let mut wrong = Vec::new();
    match written
        .lines()
        .find(|line| line.contains(" WARN wxdragon") && line.contains("TreeItemId"))
    {
        Some(line) => println!("the toolkit's warning in the file: {line}"),
        None => wrong.push("no WARN line from a wxdragon target naming the tree item"),
    }
    if written.contains("some_library") {
        wrong.push("another library's warning was written");
    }
    if written.contains("an info line from the toolkit") {
        wrong.push("the toolkit's info line was written, below its warn");
    }
    assert!(wrong.is_empty(), "{wrong:#?}\nThe log held:\n{written}");
}

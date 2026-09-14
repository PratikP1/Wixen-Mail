//! No control is built with a label that is only whitespace.
//!
//! A `StaticText` built with `" "` is a line reserved for a sentence that has
//! not arrived yet: the manager windows put their status line there and fill
//! it when Delete or Save has something to say. The space was there to hold
//! the line's height open. It also became the control's name on both
//! accessibility channels, because Windows takes a static's name from its
//! text, and a name that is one space is worse than none: a screen reader
//! moving through the window meets a control that says nothing, and the
//! automated scan reports it as an error rather than as the reviewable
//! "no name, not focusable" it reports for an empty one.
//!
//! The scan of 2026-09-14 reported seven of these across five windows, five
//! statics and two sizing grips. The grips were the same three lines of code:
//! Windows names a sizing grip after the static before it, and the static
//! before it was the space. An empty label reserves the same height, since
//! wxWidgets measures an empty string as one line tall, and reaches the tree
//! with no name at all, which is what the status line is until it has
//! something to say.
//!
//! # Why this is its own target
//!
//! `tests/house_style.rs` is named by twenty-one guard records in their
//! `tests_last_seen` counts, so a test added to it puts twenty-one
//! re-measurements on the critical path. This file is named by one record,
//! its own, written when the rule was, so nothing else is owed.
//!
//! # What this reads and what it cannot see
//!
//! Every `.rs` file under `src/`, for `with_label(` or `set_label(` followed
//! by a string literal that is non-empty and only whitespace. A label built
//! from a variable is invisible to it, as is one produced by `format!`. Those
//! are accepted: the three the scan found were all literals, and a reading of
//! literals is what catches the shape somebody copies from the line above.

use std::fs;
use std::path::{Path, PathBuf};

/// This file's own target name, which is what `cargo test --test` is given.
const ME: &str = "no_label_is_only_a_space";

/// The two ways a label is put on a control in this tree.
const LABEL_CALLS: [&str; 2] = ["with_label(", "set_label("];

fn collect(dir: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, into);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            into.push(path);
        }
    }
}

/// Every source file the rule reads.
fn every_source_file() -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect(Path::new("src"), &mut found);
    found.sort();
    found
}

/// The string literal a label call is given, when it is given one directly.
///
/// `with_label("Save")` answers `Save`; `with_label(label)` answers nothing,
/// because there is no literal to read.
fn the_literal_after<'a>(call: &str, line: &'a str) -> Option<&'a str> {
    let after = &line[line.find(call)? + call.len()..];
    let inside = after.strip_prefix('"')?;
    let end = inside.find('"')?;
    Some(&inside[..end])
}

/// Every line of `text` that gives a label that is only whitespace, as
/// `line number: the line`.
fn labels_that_are_only_a_space(text: &str) -> Vec<String> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| {
            LABEL_CALLS.iter().any(|call| {
                the_literal_after(call, line)
                    .is_some_and(|label| !label.is_empty() && label.trim().is_empty())
            })
        })
        .map(|(index, line)| format!("{}: {}", index + 1, line.trim()))
        .collect()
}

#[test]
fn test_no_control_is_built_with_a_label_that_is_only_a_space() {
    let mut found = Vec::new();
    for path in every_source_file() {
        let text = fs::read_to_string(&path).expect("a source file to be readable");
        for hit in labels_that_are_only_a_space(&text) {
            found.push(format!("{}:{hit}", path.display()));
        }
    }
    assert!(
        found.is_empty(),
        "a label that is only whitespace is a name that says nothing on both \
         accessibility channels; use an empty label, which reserves the same \
         height and reaches the tree with no name:\n{}",
        found.join("\n")
    );
}

#[test]
fn test_the_reading_can_see_a_label_that_is_only_a_space() {
    // The companion that proves the check above is not passing over nothing.
    // A guard reading a tree for a shape passes the moment the shape stops
    // being recognised, and nothing else would say so.
    let planted = "let status = StaticText::builder(&dialog).with_label(\" \").build();\n\
                   let ok = Button::builder(&dialog).with_label(\"OK\").build();\n\
                   let blank = StaticText::builder(&dialog).with_label(\"\").build();\n\
                   let tab = StaticText::builder(&dialog).with_label(\"\\t\").build();\n\
                   status.set_label(\"   \");\n\
                   status.set_label(label);\n";

    let found = labels_that_are_only_a_space(planted);

    assert_eq!(
        found,
        vec![
            "1: let status = StaticText::builder(&dialog).with_label(\" \").build();",
            "5: status.set_label(\"   \");",
        ],
        "one space and three spaces are the shape; a word, an empty label, a \
         written-out tab and a variable are not"
    );
}

#[test]
fn test_the_reading_found_the_files_at_all() {
    // A walk that finds nothing passes the check above for free.
    let files = every_source_file();
    assert!(
        files.len() > 100,
        "src/ holds hundreds of source files and the walk found {}",
        files.len()
    );
}

/// The targets every scoped run ends with, read from the gate script.
fn the_whole_tree_targets(script: &str) -> Vec<String> {
    script
        .lines()
        .find_map(|line| line.strip_prefix("guards_that_read_the_whole_tree=("))
        .and_then(|rest| rest.split(')').next())
        .map(|inner| inner.split_whitespace().map(str::to_string).collect())
        .unwrap_or_default()
}

#[test]
fn test_this_target_runs_on_the_commits_that_could_break_it() {
    // A code commit answers `affected`, which scopes the run to the modules
    // that changed and reaches a target under `tests/` only when that file
    // itself changes. The list of targets that read the whole tree is written
    // out by hand in the gate script, and nothing else would say if this fell
    // out of it.
    let script = fs::read_to_string("scripts/check.sh").expect("the gate script to be readable");

    let whole_tree = the_whole_tree_targets(&script);
    assert!(
        whole_tree.iter().any(|target| target == ME),
        "a code commit runs {whole_tree:?} at the end of its scoped run and not {ME}, \
         so the label somebody copies from the line above is never read"
    );
}

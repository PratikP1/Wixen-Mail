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
//!
//! # The second reading: an empty static nothing fills
//!
//! An empty label is right for a line something fills later, and wrong for a
//! control built to be nothing. On 2026-09-15 two testers met an unnamed
//! checkbox, in the signature editor (#42) and on the contact editor's Basic
//! tab (#40), and both were built after a `StaticText` on `""` placed in a
//! two-column grid to hold the label column open. That static is a real
//! window with no name, sitting in the tree straight before the checkbox in
//! the order Tab moves in, and it is the window Windows picks when it names a
//! control that set no name from the nearest static text. Eleven were built
//! that way under `src/presentation/` that day, and the fix is a sizer spacer,
//! which is not a window at all.
//!
//! So this also reads every `let NAME = StaticText::builder(..)` whose chain
//! to `.build()` carries `.with_label("")`, across lines, and refuses it unless
//! `NAME` is used again before the end of its function for anything other
//! than being put in a sizer: a method called on it (`set_label`, `set_value`,
//! `get_handle`), a reference to it handed to a filler (`said_and_shown`,
//! `set_accessible_name`, a filler of the dialog's own), or the binding
//! handed on, bare, in a tuple or in a struct, to a caller that fills it. A
//! spacer is placed and never touched again, and a binding that appears only
//! in `add(&NAME, ..)` is one. The five the tester's two boxes were among
//! were that shape exactly.
//!
//! What it cannot see, said plainly. A static built without a `let` and added
//! inline. A label built from a variable. A binding reused for something else
//! later in the same function, which makes the first blind. And a spacer
//! handed on in a tuple to be stored and hidden with its neighbour, which is
//! how the account editor built its six until 2026-09-16: handed on reads the
//! same as filled from the site, and telling them apart is a reading of the
//! caller this does not make. For the fifteen checkboxes in the five editors
//! `tests/checkbox_labels.rs` reads the built tree instead, which sees a
//! spacer whatever shape built it. There is no allow list, because nothing in
//! the tree needs one: an empty list watched by a test that iterates over
//! nothing is the census-emptying failure `CLAUDE.md` describes, so the day
//! something needs one is the day it is written, with its reason and a test
//! that its site still exists.

use std::fs;
use std::path::{Path, PathBuf};

/// This file's own target name, which is what `cargo test --test` is given.
const ME: &str = "no_label_is_only_a_space";

/// The two ways a label is put on a control in this tree.
const LABEL_CALLS: [&str; 2] = ["with_label(", "set_label("];

/// Where a static text is built, which is where the second reading starts.
const A_STATIC_IS_BUILT: &str = "StaticText::builder(";

/// The end of a builder chain, which is where the second reading stops
/// looking for the empty label.
const THE_CHAIN_ENDS: &str = ".build()";

/// The label the second reading is about.
const AN_EMPTY_LABEL: &str = ".with_label(\"\")";

/// The two sizer calls a spacer is put in a grid with. A reference handed to
/// either is the spacer being placed, not filled.
const SIZER_ADDS: [&str; 2] = ["add(", "add_sizer("];

/// An empty static the second reading found, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
struct EmptyStatic {
    /// The line the builder is on, counted from one.
    line: usize,
    /// What the built control was bound to.
    binding: String,
    /// The byte the statement ends at, where the search for a later use
    /// starts.
    statement_ends: usize,
}

/// Every empty static `text` builds with a `let`, filled or not.
fn empty_statics_in(text: &str) -> Vec<EmptyStatic> {
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(offset) = text[from..].find(A_STATIC_IS_BUILT) {
        let at = from + offset;
        let chain_ends = text[at..]
            .find(THE_CHAIN_ENDS)
            .map_or(text.len(), |i| at + i + THE_CHAIN_ENDS.len());
        let line_starts = text[..at].rfind('\n').map_or(0, |i| i + 1);
        let binding = text[line_starts..at]
            .trim()
            .strip_prefix("let ")
            .and_then(|rest| rest.split('=').next())
            .map(|name| name.trim().to_string());
        if let Some(binding) = binding
            && text[at..chain_ends].contains(AN_EMPTY_LABEL)
        {
            found.push(EmptyStatic {
                line: text[..at].matches('\n').count() + 1,
                binding,
                statement_ends: chain_ends,
            });
        }
        from = chain_ends.max(at + 1);
    }
    found
}

/// Whether the byte at `at` is inside an identifier.
fn is_part_of_a_name(text: &str, at: usize) -> bool {
    text[at..]
        .chars()
        .next()
        .is_some_and(|c| c.is_alphanumeric() || c == '_')
}

/// Where the function holding the byte `from` ends: the first `}` at column
/// zero after it, or the end of the text.
fn end_of_this_function(text: &str, from: usize) -> usize {
    text[from..]
        .find("\n}\n")
        .map_or(text.len(), |i| from + i + 1)
}

/// Whether `binding` is used for anything other than being put in a sizer
/// between `statement_ends` and the end of its function.
///
/// Any later use but `add(&NAME, ..)` counts: a method called on it, a
/// reference handed to a filler, the binding handed on bare, in a tuple or
/// in a struct to a caller that fills it. A spacer is placed and never
/// touched again, and that is the whole of what tells it apart.
fn is_filled_later(text: &str, empty: &EmptyStatic) -> bool {
    let until = end_of_this_function(text, empty.statement_ends);
    let rest = &text[empty.statement_ends..until];
    let mut from = 0;
    while let Some(offset) = rest[from..].find(&empty.binding) {
        let at = from + offset;
        let after = at + empty.binding.len();
        from = after;
        let whole_word = (at == 0 || !is_part_of_a_name(rest, at - 1))
            && (after >= rest.len() || !is_part_of_a_name(rest, after));
        if !whole_word {
            continue;
        }
        let referenced = at > 0 && &rest[at - 1..at] == "&";
        if !referenced {
            return true;
        }
        let before = rest[..at - 1].trim_end();
        let placed = SIZER_ADDS.iter().any(|add| before.ends_with(add));
        if !placed {
            return true;
        }
    }
    false
}

/// Every empty static in `text` that nothing fills, as `line: binding`.
fn empty_statics_nothing_fills(text: &str) -> Vec<String> {
    empty_statics_in(text)
        .iter()
        .filter(|empty| !is_filled_later(text, empty))
        .map(|empty| format!("{}: {}", empty.line, empty.binding))
        .collect()
}

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

/// The file the spacer companions splice into: the one the tester's two
/// checkboxes were built in, read from disk so the reading is asked about
/// real lines and not a fixture that happens to match it.
const THE_FILE_THE_SPACERS_WERE_IN: &str = "src/presentation/wx_managers.rs";

/// The line the Favourite box is built after, which is where the spacer
/// stood until 2026-09-16.
const WHERE_THE_FAVOURITE_BOX_IS_BUILT: &str = "    let fav_label = \"&Favorite\";\n";

/// `text` with `planted` put in front of the Favourite box's first line,
/// and the line number `planted` starts on.
fn spliced_before_the_favourite_box(text: &str, planted: &str) -> (String, usize) {
    let at = text
        .find(WHERE_THE_FAVOURITE_BOX_IS_BUILT)
        .expect("the Favourite box is still built where the tester met it");
    let line = text[..at].matches('\n').count() + 1;
    (format!("{}{planted}{}", &text[..at], &text[at..]), line)
}

#[test]
fn test_no_empty_static_is_built_that_nothing_fills() {
    let mut seen = 0;
    let mut refused = Vec::new();
    for path in every_source_file() {
        let text = fs::read_to_string(&path).expect("a source file to be readable");
        seen += empty_statics_in(&text).len();
        for hit in empty_statics_nothing_fills(&text) {
            refused.push(format!("{}:{hit}", path.display()));
        }
    }
    println!(
        "empty statics seen: {seen}, filled later: {}, refused: {}",
        seen - refused.len(),
        refused.len()
    );
    assert!(
        refused.is_empty(),
        "an empty static text nothing fills is a nameless control in the tree, and the one \
         Windows names the control after it from; hold the cell open with a sizer spacer \
         (`leave_the_cell_empty`) instead:\n{}",
        refused.join("\n")
    );
}

#[test]
fn test_the_reading_can_see_a_spacer_nothing_fills() {
    // The companion, on the file's own lines: the spacer the tester's
    // Favourite box was built after, put back as it was under a name the
    // file never uses, since the reading is blind to a binding reused later
    // and a guard record plants the original under its own name. A reading
    // that could not see it would pass the check above for free.
    let text = fs::read_to_string(THE_FILE_THE_SPACERS_WERE_IN).expect("the managers file");
    let planted = "    let planted_spacer = StaticText::builder(&basic_panel).with_label(\"\").build();\n\
                   basic_fields.add(&planted_spacer, 0, SizerFlag::All, 4);\n";
    let (spliced, line) = spliced_before_the_favourite_box(&text, planted);

    let found = empty_statics_nothing_fills(&spliced);

    // Against what the file says unspliced rather than against nothing, so
    // this judges the splice: with a spacer planted in the file itself, as a
    // guard record does, the whole-tree check is what goes red, not this.
    let already = empty_statics_nothing_fills(&text).len();
    let planted = format!("{line}: planted_spacer");
    assert!(
        found.contains(&planted) && found.len() == already + 1,
        "the spacer before the Favourite box and nothing else the file did not already say: \
         wanted {planted:?} among {already} others, found {found:?}"
    );
}

#[test]
fn test_the_reading_passes_every_shape_something_fills_later() {
    // The other companion: each way this tree fills an empty static later,
    // spliced into the same file, and the reading silent on every one. A
    // reading that refused a status line would refuse the whole tree.
    let text = fs::read_to_string(THE_FILE_THE_SPACERS_WERE_IN).expect("the managers file");
    let planted = "    let status = StaticText::builder(&basic_panel).with_label(\"\").build();\n\
                   basic_fields.add(&status, 0, SizerFlag::All, 4);\n\
                   status.set_label(\"Saved\");\n\
                   let said = StaticText::builder(&basic_panel).with_label(\"\").build();\n\
                   basic_fields.add(\n        &said,\n        0,\n        SizerFlag::All,\n        4,\n    );\n\
                   said_and_shown(&said, a11y, \"Saved\");\n\
                   let named = StaticText::builder(&basic_panel).with_label(\"\").build();\n\
                   set_accessible_name(&named, \"Saved\");\n\
                   let region = StaticText::builder(&basic_panel)\n        .with_label(\"\")\n        .with_size(Size::new(1, 1))\n        .build();\n\
                   register(region.get_handle());\n\
                   let handed_on = {\n        let h = StaticText::builder(&basic_panel).with_label(\"\").build();\n        basic_fields.add(&h, 0, SizerFlag::All, 4);\n        h\n    };\n\
                   handed_on.set_label(\"Saved\");\n";
    let (spliced, _) = spliced_before_the_favourite_box(&text, planted);

    let found = empty_statics_nothing_fills(&spliced);

    // Counted against the unspliced file for the reason the companion above
    // gives: this judges the five planted, not the tree.
    assert_eq!(
        found.len(),
        empty_statics_nothing_fills(&text).len(),
        "a method called on it, a reference handed to something other than a sizer, and \
         a binding handed on bare are all a static something fills later; refused: {found:?}"
    );
    assert_eq!(
        empty_statics_in(&spliced).len(),
        empty_statics_in(&text).len() + 5,
        "the five planted statics were all seen, the one with its label on its own line \
         among them"
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

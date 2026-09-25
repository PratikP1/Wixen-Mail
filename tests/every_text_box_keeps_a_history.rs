//! Every box a person types into keeps a history of several steps, in every
//! window, and what keeps Windows' own single step is named here with the
//! reason (#47: "in every dialog with a text field ... with a multi-step
//! history where the native control gives one step").
//!
//! 13-05 gave the main window's three boxes the history. A dialog has no menu
//! bar, so there the box's own key-down, bound by
//! `text_history_keys::keep_a_history`, is what takes Ctrl+Z and Ctrl+Y. A box
//! built without that call keeps Windows' one step and nothing says so, which
//! is how a new dialog would quietly go back to one step. This reading is what
//! says so.
//!
//! **What is read.** Every `.rs` file under `src/presentation`. Each
//! `TextCtrl::builder(` and `ComboBox::builder(` that a release build compiles
//! (asked of `what_ships` over the file up to the site, so a box a test module
//! builds is not read as shipped) is found with the name its `let` binds. Its
//! statement is the `let` line and the lines indented under it; a statement
//! naming `ReadOnly` is set aside because nobody types into it, and one naming
//! `Password` because a password is never held in memory as steps. Every other
//! box must reach `keep_a_history(&name)` later in the block it is built in,
//! which is how a helper that builds a box for its callers is held: it calls
//! it once, on the box it returns.
//!
//! **What keeps Windows' single step.** The number fields, each a
//! `SpinCtrl::builder(`, and wxWidgets' own text-entry dialog, a
//! `TextEntryDialog::builder(`. Each is named in `KEEPS_WINDOWS_ONE_STEP` by
//! file with its reason; one built in a file the list does not name is a
//! complaint, and an entry naming a file that no longer builds one is a
//! complaint too, so the list cannot outlive what it excuses.
//!
//! **What it cannot see.** A box built through a path it does not follow: one
//! not bound by `let`, which it names as a complaint rather than passing over,
//! or one built in a loop and pushed into a list, where the name it binds is
//! the loop's and `keep_a_history` has to be called inside the loop to be seen.
//! A block is found by indentation, which rustfmt keeps and the gate enforces,
//! so a multi-line string at the margin inside a block ends the block early and
//! shows as a complaint, never as a pass. And it reads source: that the history
//! works on a real box is `tests/several_steps_come_back.rs`, which builds one.
//!
//! **Companions.** Each is named `test_the_reading_complains_...`, so
//! `cargo test --test every_text_box_keeps_a_history complains` runs them
//! alone, and each plants into the real tree: a box with no history is
//! refused, a read-only box is not, a history kept outside the box's block is
//! refused, a number field in a file the list does not name is refused, and an
//! entry naming a site that is gone is refused.

use std::fs;
use std::path::Path;
use wixen_mail::common::what_ships::what_ships;

/// The builders that make a box somebody types into, which keeps a history.
const BOXES: [&str; 2] = ["TextCtrl::builder(", "ComboBox::builder("];

/// A style that sets a box aside, and why.
const SET_ASIDE: [(&str, &str); 2] = [
    ("ReadOnly", "nobody types into it"),
    ("Password", "a password is never held in memory as steps"),
];

/// A kind of box that keeps Windows' own single step, the file that builds
/// it, and why it keeps no history of ours.
struct OneStep {
    builder: &'static str,
    file: &'static str,
    reason: &'static str,
}

const A_NUMBER_FIELD: &str = "a number field is an up-down control with an Edit beside it, and \
     the Edit is not handed back as a box; a typed number is short, and Windows' own step \
     undoes it";
const WXWIDGETS_OWN_DIALOG: &str = "wxWidgets' own text-entry dialog (Describe the picture, \
     Insert Link), whose box wxdragon does not hand back; a dialog of this program's own would \
     give it the history (ledgered)";

const KEEPS_WINDOWS_ONE_STEP: [OneStep; 5] = [
    OneStep {
        builder: "SpinCtrl::builder(",
        file: "wx_account_manager.rs",
        reason: A_NUMBER_FIELD,
    },
    OneStep {
        builder: "SpinCtrl::builder(",
        file: "wx_compose.rs",
        reason: A_NUMBER_FIELD,
    },
    OneStep {
        builder: "SpinCtrl::builder(",
        file: "wx_item_form.rs",
        reason: A_NUMBER_FIELD,
    },
    OneStep {
        builder: "SpinCtrl::builder(",
        file: "wx_settings.rs",
        reason: A_NUMBER_FIELD,
    },
    OneStep {
        builder: "TextEntryDialog::builder(",
        file: "wx_compose.rs",
        reason: WXWIDGETS_OWN_DIALOG,
    },
];

/// The builders `KEEPS_WINDOWS_ONE_STEP` may name.
const ONE_STEP_BUILDERS: [&str; 2] = ["SpinCtrl::builder(", "TextEntryDialog::builder("];

/// A file's name under `src/presentation` and its text.
type Source = (String, String);

fn the_presentation_sources() -> Vec<Source> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/presentation");
    let mut sources = Vec::new();
    gather(&root, &root, &mut sources);
    sources.sort();
    sources
}

fn gather(root: &Path, directory: &Path, sources: &mut Vec<Source>) {
    let entries =
        fs::read_dir(directory).unwrap_or_else(|e| panic!("{}: {e}", directory.display()));
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            gather(root, &path, sources);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let text =
                fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            let name = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            sources.push((name, text));
        }
    }
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn is_a_comment(line: &str) -> bool {
    line.trim_start().starts_with("//")
}

/// The lines holding `builder` as code a release build compiles, as indexes.
fn sites_of(lines: &[&str], builder: &str) -> Vec<usize> {
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| builds_with(line, builder))
        .map(|(at, _)| at)
        .filter(|at| ships(lines, *at))
        .collect()
}

/// Whether `line` calls `builder` outside a comment, and not as the tail of a
/// longer name (`BitmapComboBox::builder(` is not `ComboBox::builder(`).
fn builds_with(line: &str, builder: &str) -> bool {
    let code = line.split("//").next().unwrap_or_default();
    code.match_indices(builder).any(|(at, _)| {
        !code[..at]
            .chars()
            .next_back()
            .is_some_and(|before| before.is_alphanumeric() || before == '_')
    })
}

/// Whether the line at `at` is one a release build compiles: `what_ships` over
/// the file up to it keeps it as its last line, where a test-only item holding
/// it would be dropped to its end.
fn ships(lines: &[&str], at: usize) -> bool {
    what_ships(&lines[..=at].join("\n")).lines().last() == Some(lines[at])
}

/// The name `let` binds on `line`, if it binds one.
fn bound_name(line: &str) -> Option<&str> {
    let after = line.trim_start().strip_prefix("let ")?;
    let after = after.strip_prefix("mut ").unwrap_or(after);
    let name: &str = after
        .split(|letter: char| !(letter.is_alphanumeric() || letter == '_'))
        .next()?;
    (!name.is_empty()).then_some(name)
}

/// The statement starting at `at`: its line and the lines indented under it.
/// Returns where the statement ends, one past its last line.
fn statement_end(lines: &[&str], at: usize) -> usize {
    let own = indent(lines[at]);
    let mut end = at + 1;
    while end < lines.len() && (lines[end].trim().is_empty() || indent(lines[end]) > own) {
        end += 1;
    }
    end
}

/// The code lines after the statement and before the block it sits in closes.
fn rest_of_the_block<'a>(lines: &[&'a str], at: usize, from: usize) -> Vec<&'a str> {
    let own = indent(lines[at]);
    lines[from..]
        .iter()
        .take_while(|line| line.trim().is_empty() || indent(line) >= own)
        .filter(|line| !is_a_comment(line))
        .copied()
        .collect()
}

/// What the reading found wrong with the box built at `at`, if anything.
fn complaint_about_a_box(file: &str, lines: &[&str], at: usize) -> Option<String> {
    let place = format!("{file}:{}", at + 1);
    let Some(name) = bound_name(lines[at]) else {
        return Some(format!(
            "{place}: a box not bound by `let`, which this reading cannot follow: {}",
            lines[at].trim()
        ));
    };
    let end = statement_end(lines, at);
    let statement = lines[at..end].join("\n");
    if SET_ASIDE.iter().any(|(style, _)| statement.contains(style)) {
        return None;
    }
    let kept = format!("keep_a_history(&{name})");
    let reaches = rest_of_the_block(lines, at, end)
        .iter()
        .any(|line| line.contains(&kept));
    (!reaches).then(|| {
        format!(
            "{place}: `{name}` is built here and never reaches `{kept}` in its block, so it \
             keeps Windows' single step"
        )
    })
}

/// Everything wrong across `sources`, one sentence each.
fn complaints(sources: &[Source], one_step: &[OneStep]) -> Vec<String> {
    let mut found = Vec::new();
    let mut boxes = 0;
    for (file, text) in sources {
        let lines: Vec<&str> = text.lines().collect();
        for builder in BOXES {
            for at in sites_of(&lines, builder) {
                boxes += 1;
                found.extend(complaint_about_a_box(file, &lines, at));
            }
        }
        for builder in ONE_STEP_BUILDERS {
            let named = one_step
                .iter()
                .any(|kept| kept.builder == builder && kept.file == *file);
            if named {
                continue;
            }
            for at in sites_of(&lines, builder) {
                found.push(format!(
                    "{file}:{}: a `{builder}` in a file `KEEPS_WINDOWS_ONE_STEP` does not name, \
                     so nothing says it keeps Windows' single step or why",
                    at + 1
                ));
            }
        }
    }
    for kept in one_step {
        let still_there = sources.iter().any(|(file, text)| {
            *file == kept.file
                && !sites_of(&text.lines().collect::<Vec<_>>(), kept.builder).is_empty()
        });
        if !still_there {
            found.push(format!(
                "`KEEPS_WINDOWS_ONE_STEP` names a `{}` in {} ({}), and the file builds none, so \
                 the entry excuses nothing",
                kept.builder, kept.file, kept.reason
            ));
        }
    }
    if boxes == 0 {
        found.push("no box was found under src/presentation, so the reading read nothing".into());
    }
    found
}

// ── The reading ───────────────────────────────────────────────────────────

#[test]
fn test_every_box_a_person_types_into_keeps_a_history() {
    let found = complaints(&the_presentation_sources(), &KEEPS_WINDOWS_ONE_STEP);
    assert!(
        found.is_empty(),
        "{} box(es) keep Windows' single step without a reason:\n{}",
        found.len(),
        found.join("\n")
    );
}

// ── Companions ────────────────────────────────────────────────────────────

/// The real tree with `planted` added to the end of `wx_compose.rs`.
fn planted_in_the_composer(planted: &str) -> Vec<Source> {
    let mut sources = the_presentation_sources();
    let composer = sources
        .iter_mut()
        .find(|(file, _)| file == "wx_compose.rs")
        .expect("the composer is under src/presentation");
    composer.1.push_str(planted);
    sources
}

/// What the plant adds to the complaints about the real tree.
fn complaints_the_plant_adds(planted: &str, one_step: &[OneStep]) -> Vec<String> {
    let before = complaints(&the_presentation_sources(), &KEEPS_WINDOWS_ONE_STEP);
    complaints(&planted_in_the_composer(planted), one_step)
        .into_iter()
        .filter(|complaint| !before.contains(complaint))
        .collect()
}

#[test]
fn test_the_reading_complains_of_a_box_that_never_reaches_the_history() {
    let planted = "\nfn planted(dialog: &Dialog) {\n    let planted_box = TextCtrl::builder(dialog).build();\n    set_accessible_name(&planted_box, \"Planted\");\n}\n";
    let added = complaints_the_plant_adds(planted, &KEEPS_WINDOWS_ONE_STEP);
    assert_eq!(added.len(), 1, "{added:?}");
    assert!(added[0].contains("`planted_box`"), "{added:?}");
}

#[test]
fn test_the_reading_complains_of_an_editable_combo_box_with_no_history() {
    let planted = "\nfn planted(panel: &Panel) {\n    let planted_combo = ComboBox::builder(panel)\n        .with_string_choices(&[\"one\"])\n        .build();\n}\n";
    let added = complaints_the_plant_adds(planted, &KEEPS_WINDOWS_ONE_STEP);
    assert_eq!(added.len(), 1, "{added:?}");
    assert!(added[0].contains("`planted_combo`"), "{added:?}");
}

#[test]
fn test_the_reading_complains_of_nothing_about_a_read_only_box_or_a_password() {
    let planted = "\nfn planted(dialog: &Dialog) {\n    let shown = TextCtrl::builder(dialog)\n        .with_style(TextCtrlStyle::ReadOnly | TextCtrlStyle::MultiLine)\n        .build();\n    let secret = TextCtrl::builder(dialog)\n        .with_style(TextCtrlStyle::Password)\n        .build();\n}\n";
    let added = complaints_the_plant_adds(planted, &KEEPS_WINDOWS_ONE_STEP);
    assert!(added.is_empty(), "{added:?}");
}

#[test]
fn test_the_reading_complains_of_nothing_about_a_box_that_keeps_a_history() {
    let planted = "\nfn planted(dialog: &Dialog) -> TextCtrl {\n    let kept = TextCtrl::builder(dialog).build();\n    set_accessible_name(&kept, \"Kept\");\n    if true {\n        text_history_keys::keep_a_history(&kept);\n    }\n    kept\n}\n";
    let added = complaints_the_plant_adds(planted, &KEEPS_WINDOWS_ONE_STEP);
    assert!(added.is_empty(), "{added:?}");
}

#[test]
fn test_the_reading_complains_of_a_history_kept_outside_the_boxs_block() {
    let planted = "\nfn planted(dialog: &Dialog) {\n    {\n        let elsewhere = TextCtrl::builder(dialog).build();\n    }\n    keep_a_history(&elsewhere);\n    // keep_a_history(&elsewhere);\n}\n";
    let added = complaints_the_plant_adds(planted, &KEEPS_WINDOWS_ONE_STEP);
    assert_eq!(added.len(), 1, "{added:?}");
    assert!(added[0].contains("`elsewhere`"), "{added:?}");
}

#[test]
fn test_the_reading_complains_of_a_box_it_cannot_follow() {
    let planted =
        "\nfn planted(dialog: &Dialog) -> TextCtrl {\n    TextCtrl::builder(dialog).build()\n}\n";
    let added = complaints_the_plant_adds(planted, &KEEPS_WINDOWS_ONE_STEP);
    assert_eq!(added.len(), 1, "{added:?}");
    assert!(added[0].contains("not bound by `let`"), "{added:?}");
}

#[test]
fn test_the_reading_complains_of_nothing_about_a_box_a_test_builds() {
    let planted = "\n#[cfg(test)]\nmod planted_tests {\n    fn planted(dialog: &Dialog) {\n        let in_a_test = TextCtrl::builder(dialog).build();\n    }\n}\n";
    let added = complaints_the_plant_adds(planted, &KEEPS_WINDOWS_ONE_STEP);
    assert!(added.is_empty(), "{added:?}");
}

#[test]
fn test_the_reading_complains_of_a_number_field_in_a_file_the_list_does_not_name() {
    let mut sources = the_presentation_sources();
    let before = complaints(&sources, &KEEPS_WINDOWS_ONE_STEP);
    let feedback = sources
        .iter_mut()
        .find(|(file, _)| file == "wx_feedback.rs")
        .expect("the feedback window is under src/presentation");
    feedback.1.push_str(
        "\nfn planted(dialog: &Dialog) {\n    let count = SpinCtrl::builder(dialog).build();\n}\n",
    );
    let added: Vec<String> = complaints(&sources, &KEEPS_WINDOWS_ONE_STEP)
        .into_iter()
        .filter(|complaint| !before.contains(complaint))
        .collect();
    assert_eq!(added.len(), 1, "{added:?}");
    assert!(added[0].starts_with("wx_feedback.rs:"), "{added:?}");
}

#[test]
fn test_the_reading_complains_of_an_entry_naming_a_site_that_is_gone() {
    let mut widened: Vec<OneStep> = KEEPS_WINDOWS_ONE_STEP.into_iter().collect();
    widened.push(OneStep {
        builder: "SpinCtrl::builder(",
        file: "wx_feedback.rs",
        reason: A_NUMBER_FIELD,
    });
    let before = complaints(&the_presentation_sources(), &KEEPS_WINDOWS_ONE_STEP);
    let added: Vec<String> = complaints(&the_presentation_sources(), &widened)
        .into_iter()
        .filter(|complaint| !before.contains(complaint))
        .collect();
    assert_eq!(added.len(), 1, "{added:?}");
    assert!(added[0].contains("wx_feedback.rs"), "{added:?}");
}

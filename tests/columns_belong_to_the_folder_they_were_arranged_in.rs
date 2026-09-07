//! The window asks which folder somebody is in, rather than naming one.
//!
//! A column layout carries the kind of folder it was arranged in, and for as
//! long as it has, every place that read one named `Inbox`. So the Sent and
//! Drafts layout was written, tested and never once used, `Alt+R` in Sent put
//! back the inbox's columns, and sorting a column in Sent decided how the
//! inbox opened after a restart.
//!
//! The decisions themselves are pure functions now, driven in
//! `presentation::message_columns` and `presentation::wx_columns`, so what is
//! left here is the hop from the window to them. That hop lives inside a
//! closure inside a window builder, where reaching it needs a window, an
//! account with stored credentials and a mail server, so it is read out of the
//! source instead.
//!
//! What that cannot see, said plainly. It says the window asks; it says nothing
//! about what the answer is, whether the folder change is ever reached, or
//! whether anybody hears the list rearrange itself. The first two are covered
//! by the tests beside the functions being asked. The third is a question only
//! a screen reader settles.

use std::fs;

/// The half of a source file a release build compiles, so a shape that exists
/// only under `#[cfg(test)]` cannot answer for the shipped one.
use wixen_mail::common::what_ships::what_ships;

/// The lines following `marker`.
///
/// `body_of` in `tests/wired.rs` ends a function at a brace in column zero,
/// which is right for a function and no use here: the folder change this reads
/// sits inside a closure inside a builder, and no brace it is bounded by is
/// ever at the margin.
fn the_lines_after(source: &str, marker: &str, how_many: usize) -> String {
    let at = source.find(marker).unwrap_or_else(|| {
        panic!("`{marker}` is no longer in this file, so this guard is measuring nothing")
    });
    source[at..]
        .lines()
        .take(how_many)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Moving to a folder that wants different columns restores rather than
/// rebuilds.
///
/// Rebuilding is what shipped, and it is the half of this defect somebody meets
/// without restarting: arrange the inbox, look in Sent, come back, and the
/// arrangement is gone.
#[test]
fn test_a_change_of_folder_kind_restores_the_layout_rather_than_rebuilding_it() {
    let app =
        what_ships(&fs::read_to_string("src/presentation/wx_app.rs").expect("the main window"));
    let change = the_lines_after(
        &app,
        "let kind = message_columns::FolderKind::for_folder(stored_kind);",
        12,
    );

    assert!(
        change.contains(".keep(&layout)"),
        "the layout in effect is dropped on the way out of a folder, so there is \
         nothing to come back to"
    );
    assert!(
        change.contains("arriving_at(kind)"),
        "arriving at a folder does not ask what was last arranged there"
    );
    assert!(
        !change.contains("ColumnLayout::defaults_for(kind)"),
        "a change of folder kind rebuilds from the defaults, which is what threw \
         away a hand-arranged inbox on a trip to Sent and back"
    );
}

/// Restore Defaults is told which folder it is in by the layout, not by a
/// caller.
///
/// The parameter that used to say so is gone, which is the point: it had one
/// caller and that caller passed a literal.
#[test]
fn test_restore_defaults_asks_the_layout_which_folder_it_is_in() {
    let dialog = what_ships(
        &fs::read_to_string("src/presentation/wx_columns.rs").expect("the column chooser"),
    );

    assert!(
        dialog.contains("what_reset_restores(current)"),
        "the column chooser no longer asks the layout which folder it belongs to, \
         so Restore Defaults is back to putting the same columns everywhere"
    );
    assert!(
        !dialog.contains("kind: FolderKind"),
        "the column chooser takes a kind of folder from its caller again. It had \
         one caller, which passed a literal, and deleting the parameter is what \
         stops that rather than a comment asking nobody to do it"
    );
}

/// What one call really passes, from the bracket after `marker` to the one that
/// closes it.
///
/// Counted rather than cut off after so many characters, which is what the
/// first version of the guard below did. It read past the end of the call into
/// the fallback beside it, found the words `FolderKind::Inbox` there and
/// reported a defect in a line that is doing the right thing. A check that
/// cannot say where a call ends cannot say what it was passed.
fn the_arguments_of(source: &str, marker: &str) -> Vec<String> {
    let mut calls = Vec::new();
    for (at, _) in source.match_indices(marker) {
        let mut depth = 0usize;
        let opened = at + marker.len() - 1;
        for (offset, character) in source[opened..].char_indices() {
            match character {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        calls.push(source[opened + 1..opened + offset].to_string());
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    calls
}

/// No layout is read back as a folder somebody named rather than the one it
/// says.
///
/// The grep this replaces was the whole finding: four call sites, one literal
/// each, and a commit that corrected one of them and left three.
#[test]
fn test_nothing_names_a_kind_of_folder_when_reading_a_stored_layout() {
    for file in [
        "src/presentation/message_columns.rs",
        "src/presentation/wx_app.rs",
        "src/presentation/wx_settings.rs",
        "src/presentation/wx_columns.rs",
    ] {
        let source = what_ships(&fs::read_to_string(file).expect(file));
        for call in the_arguments_of(&source, "from_stored(") {
            assert!(
                !call.contains("FolderKind"),
                "{file} reads a stored layout as `{call}`, naming the folder it is \
                 for. That is how a layout arranged in Sent came back as the inbox's"
            );
        }
    }
}

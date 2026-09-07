//! A key is written where the surface that binds it is described.
//!
//! # Why this is not the check that already exists
//!
//! `tests/wired.rs` holds the pair that reads `docs/KEYBOARD_SHORTCUTS.md`
//! against the bindings, and both of its directions ask about a key name over
//! the whole document: is this documented key bound anywhere, and is this bound
//! key written anywhere. That is the right question for a menu accelerator,
//! which means one thing everywhere in the application.
//!
//! It is the wrong question for a key that means different things on different
//! surfaces, and this application has three of those. `F8` reaches the
//! attachments in the reader window, opens the Columns dialog on the View menu,
//! and reaches the toolbar in the composer. `Delete` throws a message away in
//! the message list and takes a file off a message in the composer. `F6` moves
//! between panes in the main window and closes the conversation window.
//!
//! So the whole-document check was green for all three while the composer's
//! `F8`, the composer's `Delete` and the conversation window's `F6` were written
//! nowhere, and a reader working entirely from the keyboard had no way to learn
//! any of them. Widening `documented_combinations` to see unmodified function
//! keys, which is the other half of this plan, does not close that: the name was
//! always in the document, for another surface.
//!
//! This asks the narrower question. For each surface that binds a key of its
//! own, the part of the document that describes that surface has to name it.
//!
//! # What this cannot see
//!
//! That the sentence beside the key is true. The document could name `F8` in the
//! composer section and say it does something else entirely, and this would
//! pass. It is a check that the key is written down where somebody would look
//! for it, not a check on what is written about it.
//!
//! Nor whether the key really arrives. `Ctrl+backslash` is bound in the
//! composer's page and was measured never reaching it, and no reader of source
//! text can tell that from a key that works.

use std::fs;
use wixen_mail::common::what_ships::what_ships;

const DOC: &str = "docs/KEYBOARD_SHORTCUTS.md";
const THE_COMPOSER_PAGE: &str = "src/presentation/editor_document.rs";
const THE_COMPOSER_WINDOW: &str = "src/presentation/wx_compose.rs";
const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

/// The part of the document under one heading, up to the next heading at the
/// same level or above it.
///
/// Anchored on the whole heading line, so `### Composition Window` cannot be
/// matched by a sentence that mentions the composition window. A heading that
/// is not in the document answers with nothing, and a check reading nothing
/// fails, which is the right direction: the section not existing is exactly the
/// case this is written to catch.
fn under(doc: &str, heading: &str) -> String {
    let level = heading.chars().take_while(|letter| *letter == '#').count();
    let mut reading = false;
    let mut kept = Vec::new();
    for line in doc.lines() {
        if line.trim_end() == heading {
            reading = true;
            continue;
        }
        if !reading {
            continue;
        }
        let here = line.chars().take_while(|letter| *letter == '#').count();
        if here > 0 && here <= level && line[here..].starts_with(' ') {
            break;
        }
        kept.push(line);
    }
    kept.join("\n")
}

/// Every key a page bound to this application asks for by name.
///
/// A web view keeps every key once it has focus, so a page that has to give one
/// back binds it in its own handler and posts the request out. Those bindings
/// reach no menu table, which is the only thing the reader in `tests/wired.rs`
/// can see, and that is how three keys hid at once.
///
/// The browser's own name for the key, which is a fixed string from a standard
/// list rather than a word this tree chose, so it is a key name and never a
/// role. Single characters are skipped: a letter on its own is not a shortcut
/// without the modifier the handler reads beside it, and this cannot see
/// modifiers.
///
/// Read over what a release build compiles, so an assertion in a test module
/// quoting the script back cannot answer for the script. `what_ships` is used
/// rather than a cut at the first `#[cfg(test)]`, which this tree has already
/// been bitten by twice.
fn keys_a_page_asks_for(source: &str) -> Vec<String> {
    let shipped = what_ships(source);
    let mut found = Vec::new();
    for (at, _) in shipped.match_indices(".key === '") {
        let rest = &shipped[at + ".key === '".len()..];
        let Some(ends) = rest.find('\'') else {
            continue;
        };
        let key = &rest[..ends];
        if key.chars().count() > 1 && !found.contains(&key.to_string()) {
            found.push(key.to_string());
        }
    }
    found
}

/// The key as the document spells it.
///
/// One entry, and it earns its place: every page in this application asks the
/// browser for `Escape`, which is the name the standard gives that key, and
/// every table in the document writes `Esc`, which is the name printed on it.
/// Both are right and they are not the same string.
fn as_the_document_writes_it(key: &str) -> &str {
    match key {
        "Escape" => "Esc",
        other => other,
    }
}

/// Whether a section names this key as a key, rather than mentioning the word.
///
/// Backticked, which is how every key in this document is written and what
/// `documented_combinations` in `tests/wired.rs` already relies on. Without it
/// the composer's `Delete` reads as documented by the spelling dialog's Delete
/// button, which is a word in a sentence, and the check passes against the
/// defect it was written for.
fn names_the_key(section: &str, key: &str) -> bool {
    section.contains(&format!("`{key}`"))
}

/// Every function key the composer's page binds is in the composer's section.
///
/// `F8` is the way into the composer's toolbar and was the key this found. The
/// document gave `Ctrl+\` for that, which `editor_document.rs`'s own comment
/// records as measured against the running composer and never arriving, while
/// two chords beside it arrive every time. Somebody working entirely from the
/// keyboard read the page, pressed the key three times and had every reason to
/// conclude the toolbar was unreachable.
#[test]
fn test_every_function_key_the_composers_page_binds_is_in_the_composers_section() {
    let page = fs::read_to_string(THE_COMPOSER_PAGE).expect("the composer's page");
    let doc = fs::read_to_string(DOC).expect("the shortcuts document");
    let section = under(&doc, "### Composition Window");
    assert!(
        !section.is_empty(),
        "docs/KEYBOARD_SHORTCUTS.md has no `### Composition Window` section"
    );

    let mut missing = Vec::new();
    for key in keys_a_page_asks_for(&page) {
        let a_function_key = key
            .strip_prefix('F')
            .is_some_and(|number| matches!(number.parse::<u8>(), Ok(1..=12)));
        if a_function_key && !names_the_key(&section, &key) {
            missing.push(key);
        }
    }

    assert!(
        missing.is_empty(),
        "the composer's page binds these keys and the Composition Window \
         section of docs/KEYBOARD_SHORTCUTS.md names none of them: {missing:?}"
    );
}

/// The key that takes a file off a message is in the composer's section.
///
/// Bound on the attachments list rather than in the page, because the list is an
/// ordinary wxWidgets control, so it is invisible to the reader above and to
/// every menu table. `127` is `Delete`, and it is written here rather than taken
/// from the source so that renaming the constant cannot quietly change what this
/// is about.
#[test]
fn test_the_key_that_takes_a_file_off_a_message_is_in_the_composers_section() {
    let window = what_ships(&fs::read_to_string(THE_COMPOSER_WINDOW).expect("the composer"));
    let doc = fs::read_to_string(DOC).expect("the shortcuts document");
    let section = under(&doc, "### Composition Window");

    assert!(
        window.contains("const KEY_DELETE: i32 = 127;"),
        "the composer no longer names 127 as Delete, so this check is about \
         nothing and wants measuring again by hand"
    );
    assert!(
        window.contains("attachment_list.on_key_down("),
        "nothing on the attachments list reads a key, so Delete takes no file \
         off a message"
    );
    assert!(
        window.contains("key.event.get_key_code() != Some(KEY_DELETE)"),
        "the attachments list's key handler no longer asks for Delete"
    );

    assert!(
        names_the_key(&section, "Delete"),
        "Delete takes the selected file off the message and the Composition \
         Window section of docs/KEYBOARD_SHORTCUTS.md does not name it"
    );
}

/// Every key that closes the conversation window is in the section about
/// getting out of it.
///
/// The window shares `wire_the_way_out`'s injected script with the message
/// preview, and its own handler closes on any leave message whichever key sent
/// it. So `Esc`, `F6` and `Shift+F6` all close it, and `F6` is the one that
/// matters: the same document tells the reader `F6` moves to the next pane, so
/// somebody who learned the key in the main window presses it here and the
/// window goes.
///
/// What this cannot see: the shift half. The script reads `e.shiftKey` for `F6`
/// and this reader answers key names, so `Shift+F6` is documented because it is
/// true rather than because anything here checks it.
#[test]
fn test_every_key_that_closes_the_conversation_window_is_in_its_section() {
    let app = what_ships(&fs::read_to_string(THE_MAIN_WINDOW).expect("the main window"));
    let doc = fs::read_to_string(DOC).expect("the shortcuts document");
    let section = under(&doc, "#### Getting out of the conversation window");
    assert!(
        !section.is_empty(),
        "docs/KEYBOARD_SHORTCUTS.md says nowhere how to get out of the \
         conversation window, and three keys do it"
    );

    assert!(
        app.contains("wire_the_way_out(&page, \"conversation window\")"),
        "the conversation window no longer takes the injected script, so the \
         keys this is about are not bound there"
    );
    assert!(
        app.contains("crate::presentation::panes::leaving_which_way(&json).is_some()"),
        "the conversation window no longer closes on a leave message"
    );

    let mut missing = Vec::new();
    for key in keys_a_page_asks_for(&app) {
        let written = as_the_document_writes_it(&key);
        if !names_the_key(&section, written) {
            missing.push(written.to_string());
        }
    }

    assert!(
        missing.is_empty(),
        "these keys close the conversation window and the section about \
         getting out of it names none of them: {missing:?}"
    );
}

//! Every shape a folder's mail goes out in, and the one question each export
//! asks before it writes anything.
//!
//! File, Export Mailbox writes a folder with the folders inside it into a zip,
//! File, Export Folder as a Mailbox File writes one folder alone into one
//! mailbox file (#53, point 4; 13-45), and File, Export Folder as Message Files
//! writes a folder and the folders inside it as one saved message per file
//! (#53, point 4; 13-46). All three take the row the cursor is on, and
//! a row in the folder tree is not always a folder: a saved search, a label or
//! a branch holds no mail of its own, and written out as one it is an empty
//! file or a claim about mail from somewhere that does not exist.
//!
//! So the question is asked once, in `a_folder_to_write_out`, and every export
//! handler asks it. A handler added later that answers the question itself, or
//! not at all, is what the census below refuses, and its companion plants one
//! to prove the reading can see it.
//!
//! What this cannot see. It reads source, so it says the menu item, the arm,
//! the handler and the question are written and joined, and not that a person
//! choosing the item hears what the handler says; the accessibility scan and
//! the tester's ear answer that.

use std::fs;

use wixen_mail::common::what_ships::what_ships;

/// The window every export handler is written in.
const THE_WINDOW: &str = "src/presentation/wx_app.rs";

/// The menu item's identifier.
const THE_ID: &str = "ID_EXPORT_A_FOLDER_AS_A_MAILBOX_FILE";

/// The item's label, with its letter and no shortcut.
const THE_LABEL: &str = "Export Folder as a Mailbox &File...";

/// The arm the item's identifier reaches.
const THE_ARM: &str = "_ if id == ID_EXPORT_A_FOLDER_AS_A_MAILBOX_FILE => {";

/// The handler, with its bracket so a comment naming it is not a call.
const THE_HANDLER: &str = "export_a_folder_as_a_mailbox_file(";

/// The one question every export asks.
const THE_QUESTION: &str = "a_folder_to_write_out(";

/// Where the question is written.
const WHERE_THE_QUESTION_IS: &str = "fn a_folder_to_write_out(";

/// What the question has to ask to be the question.
const IS_A_FOLDER: &str = "WhichRow::Folder";

/// The writer the handler hands the folder to.
const THE_WRITER: &str = "one_folder_as_a_mailbox_file(";

/// The message files item's identifier (13-46).
const THE_FILES_ID: &str = "ID_EXPORT_A_FOLDER_AS_MESSAGE_FILES";

/// The message files item's label, with its letter and no shortcut.
const THE_FILES_LABEL: &str = "E&xport Folder as Message Files...";

/// The arm the message files item's identifier reaches.
const THE_FILES_ARM: &str = "_ if id == ID_EXPORT_A_FOLDER_AS_MESSAGE_FILES => {";

/// The message files handler, with its bracket so a comment naming it is not
/// a call.
const THE_FILES_HANDLER: &str = "export_a_folder_as_message_files(";

/// What the message files handler asks for the folder to write into.
const A_FOLDER_PICKER: &str = "DirDialog::builder(";

/// The writer the message files handler hands the folder to.
const THE_FILES_WRITER: &str = "one_folder_as_message_files(";

/// How every export handler's name begins.
const AN_EXPORT_HANDLER: &str = "fn export_a";

/// The fewest export handlers the window has: Export Mailbox, Export Folder
/// as a Mailbox File and Export Folder as Message Files, as of 2026-10-03.
/// Fewer means the census read less than the window holds.
const AT_LEAST_THIS_MANY_EXPORTS: usize = 3;

/// What a release build compiles of the main window.
fn the_window() -> String {
    what_ships(&fs::read_to_string(THE_WINDOW).expect("the main window"))
}

/// The text of the item that opens at `start`, up to the first line after it
/// that is a closing brace at the margin.
fn the_item_at<'a>(source: &'a str, start: &str) -> Option<&'a str> {
    let rest = &source[source.find(start)?..];
    Some(&rest[..rest.find("\n}\n")?])
}

/// The File menu's builder, from its first line to the call that builds it.
fn the_file_menu(window: &str) -> &str {
    let rest = &window[window
        .find("let file = Menu::builder()")
        .expect("the File menu is built")..];
    &rest[..rest
        .find(".build();")
        .expect("the File menu's builder ends")]
}

/// The item's arm, up to the next arm, or nothing when it is not written.
fn the_arm(window: &str) -> &str {
    the_arm_opening(window, THE_ARM)
}

/// The arm that opens with `opening`, up to the next arm, or nothing when it
/// is not written.
fn the_arm_opening<'a>(window: &'a str, opening: &str) -> &'a str {
    let Some(at) = window.find(opening) else {
        return "";
    };
    let rest = &window[at + opening.len()..];
    &rest[..rest.find("_ if id ==").unwrap_or(rest.len())]
}

/// The string literal written first after `after` in `source`.
fn the_text_after<'a>(source: &'a str, after: &str) -> Option<&'a str> {
    let rest = &source[source.find(after)? + after.len()..];
    let opening = rest.find('"')? + 1;
    let closing = opening + rest[opening..].find('"')?;
    Some(&rest[opening..closing])
}

/// How many export handlers `source` holds, when every one asks the folder
/// question and the question asks whether the row is a folder; otherwise
/// what is wrong, naming every handler that skips it.
fn every_export_asks_the_folder_question(source: &str) -> Result<usize, String> {
    let question = the_item_at(source, WHERE_THE_QUESTION_IS)
        .ok_or_else(|| format!("{WHERE_THE_QUESTION_IS} is gone"))?;
    if !question.contains(IS_A_FOLDER) {
        return Err(format!(
            "{WHERE_THE_QUESTION_IS} no longer asks {IS_A_FOLDER}, so it answers for a saved \
             search as though it were a folder"
        ));
    }
    let handlers: Vec<&str> = source
        .lines()
        .filter(|line| line.starts_with(AN_EXPORT_HANDLER))
        .collect();
    let skipping: Vec<&str> = handlers
        .iter()
        .filter(|start| {
            the_item_at(source, start).is_none_or(|handler| !handler.contains(THE_QUESTION))
        })
        .copied()
        .collect();
    if !skipping.is_empty() {
        return Err(format!(
            "these exports do not ask {THE_QUESTION}, so a saved search or a label can be \
             written out as a folder: {skipping:?}"
        ));
    }
    Ok(handlers.len())
}

#[test]
fn test_export_folder_as_a_mailbox_file_is_on_the_file_menu_with_no_shortcut() {
    let window = the_window();
    let file = the_file_menu(&window);

    assert!(
        file.contains(&format!("{THE_ID},")),
        "the File menu has no item for {THE_ID}"
    );
    let label = the_text_after(file, &format!("{THE_ID},"));
    assert_eq!(
        label,
        Some(THE_LABEL),
        "the item's label is not {THE_LABEL:?}, or carries a shortcut after a tab"
    );
}

#[test]
fn test_export_folder_as_a_mailbox_file_reaches_its_writer() {
    let window = the_window();
    let arm = the_arm(&window);
    assert!(
        arm.contains(THE_HANDLER),
        "the item's arm does not call {THE_HANDLER}"
    );
    let handler = the_item_at(&window, &format!("fn {THE_HANDLER}"))
        .expect("the handler is written in the main window");
    assert!(
        handler.contains(THE_QUESTION),
        "the handler does not ask {THE_QUESTION}"
    );
    assert!(
        handler.contains(THE_WRITER),
        "the handler does not hand the folder to {THE_WRITER}"
    );
}

#[test]
fn test_export_folder_as_message_files_is_on_the_file_menu_with_no_shortcut() {
    let window = the_window();
    let file = the_file_menu(&window);

    assert!(
        file.contains(&format!("{THE_FILES_ID},")),
        "the File menu has no item for {THE_FILES_ID}"
    );
    let label = the_text_after(file, &format!("{THE_FILES_ID},"));
    assert_eq!(
        label,
        Some(THE_FILES_LABEL),
        "the item's label is not {THE_FILES_LABEL:?}, or carries a shortcut after a tab"
    );
}

#[test]
fn test_export_folder_as_message_files_reaches_its_writer() {
    // The folder question first, then a folder to write into from the
    // picker, then the writer, in that order: a writer reached before the
    // picker has answered writes wherever the program happens to be.
    let window = the_window();
    let arm = the_arm_opening(&window, THE_FILES_ARM);
    assert!(
        arm.contains(THE_FILES_HANDLER),
        "the item's arm does not call {THE_FILES_HANDLER}"
    );
    let handler = the_item_at(&window, &format!("fn {THE_FILES_HANDLER}"))
        .expect("the handler is written in the main window");
    let question = handler.find(THE_QUESTION);
    let picker = handler.find(A_FOLDER_PICKER);
    let writer = handler.find(THE_FILES_WRITER);
    assert!(
        question.is_some(),
        "the handler does not ask {THE_QUESTION}"
    );
    assert!(
        picker.is_some(),
        "the handler does not ask for a folder through {A_FOLDER_PICKER}"
    );
    assert!(
        writer.is_some(),
        "the handler does not hand the folder to {THE_FILES_WRITER}"
    );
    assert!(
        question < picker && picker < writer,
        "the handler asks its questions out of order: the folder question at {question:?}, \
         the picker at {picker:?}, the writer at {writer:?}"
    );
    let after_the_picker = &handler[picker.unwrap_or_default()..];
    assert!(
        after_the_picker.contains("picker.get_path()"),
        "the handler does not write where the picker said"
    );
}

#[test]
fn test_every_export_asks_whether_what_is_chosen_is_a_folder() {
    let counted = every_export_asks_the_folder_question(&the_window());

    let Ok(exports) = counted else {
        panic!("{}", counted.err().unwrap_or_default());
    };
    assert!(
        exports >= AT_LEAST_THIS_MANY_EXPORTS,
        "the census found {exports} export handlers and the window has at least \
         {AT_LEAST_THIS_MANY_EXPORTS}, so it read less than the window holds"
    );
}

#[test]
fn test_the_folder_question_reading_refuses_an_export_that_skips_it() {
    let planted = "fn a_folder_to_write_out(\n) {\n    WhichRow::Folder\n}\n\
                   fn export_a_mailbox(\n) {\n    a_folder_to_write_out(state);\n}\n\
                   fn export_a_thing(\n) {\n    write_whatever_is_chosen();\n}\n";

    let refused = every_export_asks_the_folder_question(planted);

    assert!(
        refused
            .as_ref()
            .is_err_and(|why| why.contains("fn export_a_thing(")),
        "an export that never asks the folder question was read as asking it: {refused:?}"
    );
}

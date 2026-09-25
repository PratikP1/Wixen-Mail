//! File, Print is where somebody looks for it, carries `Ctrl+P`, and prints
//! the message the way the reader shows it, through Windows' own dialog.
//!
//! The tester on 2026-09-15 (#45): "Add print functionality." The issue asks
//! for File, Print on `Ctrl+P`, "through the native Windows print dialog so the
//! printer, copies and pages are the ordinary dialog's", printing "the header
//! lines (From, To, Cc, Date, Subject) and the text".
//!
//! `tests/wired.rs` holds every item to a handler by id and every key to the
//! shortcuts page, and says nothing about which menu an item is on or what
//! its handler goes through. This reading holds those: Print on File, between
//! the PGP key item and Quit, with its key and its letter; its arm handing the
//! message to one function that composes it the way the reader window does,
//! with dates in full, asking Windows' dialog for the printer, and naming the
//! job by kind so no subject reaches a shared printer's queue.
//!
//! Read from the source rather than built, because a menu's order is the text
//! of one builder chain and an arm's calls are the text of one function. Each
//! reading has a companion that plants the wrong shape and requires a
//! complaint, so a reading that passes over anything is found here.
//!
//! What this cannot see: the dialog, which needs a person at it, and a page on
//! paper. `tests/printing_draws_what_the_layout_says.rs` reads the drawing back
//! and `tests/printing_spools_a_document.rs` spools a real job.

use std::fs;
use wixen_mail::common::what_ships::what_ships;

/// The builder chain for the menu bound to `name`, from `let name =
/// Menu::builder()` to its `.build()`.
fn menu_chain<'a>(ship: &'a str, name: &str) -> Option<&'a str> {
    let opens = ship.find(&format!("let {name} = Menu::builder()"))?;
    let chain = &ship[opens..];
    let closes = chain.find(".build()")?;
    Some(&chain[..closes])
}

/// The text of the `.append_item(` call whose first argument is `id`, from the
/// dot to the closing bracket.
fn the_call_appending<'a>(text: &'a str, id: &str) -> Option<&'a str> {
    let mut from = 0;
    while let Some(at) = text[from..].find(".append_item(") {
        let opens = from + at;
        let after = &text[opens + ".append_item(".len()..];
        if after.trim_start().starts_with(&format!("{id},")) {
            let mut depth = 0usize;
            for (offset, c) in text[opens..].char_indices() {
                match c {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            return Some(&text[opens..=opens + offset]);
                        }
                    }
                    _ => {}
                }
            }
            return None;
        }
        from = opens + 1;
    }
    None
}

/// One function's text, from its signature to the closing brace at column
/// nought, or a complaint when the signature is gone.
fn body_of(source: &str, signature: &str) -> Result<String, String> {
    let at = source.find(signature).ok_or(format!(
        "{signature} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let ends = rest.find("\n}\n").map_or(rest.len(), |end| end + 2);
    Ok(rest[..ends].to_string())
}

/// The label Print carries: the letter P and the key `Ctrl+P`, written as
/// the source writes a tab.
const THE_LABEL: &str = r#""&Print...\tCtrl+P""#;

/// The function the Print arm hands the message to.
const THE_HANDLER: &str = "fn print_the_message_under_the_cursor(";

/// What the handler has to call, and why each one matters.
const THE_CALLS: [(&str, &str); 5] = [
    (
        "a_message_as_the_reader_shows_it(",
        "no longer composes the message through the function the reader window \
         uses, so paper and the reader can come to show two different messages",
    ),
    (
        "on_paper(",
        "no longer asks for the dates in full, so a page says \"2 days ago\" and is \
         wrong the day after it is printed",
    ),
    (
        "ask_for_a_printer(",
        "no longer asks Windows' own print dialog, so the printer, the copies and \
         the pages are not chosen where they are in every other program",
    ),
    ("print_on(", "no longer sends the pages to the printer"),
    (
        "Kind::Message",
        "no longer names the job by kind, so a subject could reach a shared \
         printer's queue",
    ),
];

/// Where Print is and what it goes through, said as a complaint when either
/// is wrong.
fn where_print_is(app: &str) -> Result<(), String> {
    let ship = what_ships(app);
    let file = menu_chain(&ship, "file")
        .ok_or("the File menu is no longer built by that name, so this reads nothing")?;
    let print = the_call_appending(file, "ID_PRINT")
        .ok_or("the File menu appends no Print item, so File, Print is not there")?;
    if !print.contains(THE_LABEL) {
        return Err(format!(
            "Print on File no longer reads {THE_LABEL}, so its key or its letter is not \
             where somebody learns it: {}",
            print.split_whitespace().collect::<Vec<_>>().join(" ")
        ));
    }
    let at = |id: &str| file.find(&format!("{id},"));
    match (at("ID_IMPORT_PGP_KEY"), at("ID_PRINT"), at("ID_QUIT")) {
        (Some(key), Some(print), Some(quit)) if key < print && print < quit => {}
        _ => {
            return Err(
                "Print is no longer between Import PGP Private Key and Quit on File, in \
                 a group of its own"
                    .to_string(),
            );
        }
    }
    let arm = "_ if id == ID_PRINT =>";
    let the_arm = ship
        .find(arm)
        .map(|at| &ship[at + arm.len()..])
        .map(|rest| &rest[..rest.find("_ if id ==").unwrap_or(rest.len())])
        .ok_or("nothing answers ID_PRINT, so the item does nothing")?;
    if !the_arm.contains(&THE_HANDLER["fn ".len()..]) {
        return Err(format!(
            "the ID_PRINT arm no longer hands the message to {THE_HANDLER}"
        ));
    }
    let handler = body_of(&ship, THE_HANDLER)?;
    for (call, why) in THE_CALLS {
        if !handler.contains(call) {
            return Err(format!("File, Print {why}: {call} is not called"));
        }
    }
    Ok(())
}

fn the_main_window() -> String {
    fs::read_to_string("src/presentation/wx_app.rs").expect("the main window")
}

/// The main window with `was` replaced by `now` once inside the handler.
fn with_the_handler_changed(app: &str, was: &str, now: &str) -> String {
    let handler = body_of(app, THE_HANDLER).expect("the handler");
    let changed = handler.replacen(was, now, 1);
    assert_ne!(
        changed, handler,
        "the plant changed nothing, so the handler no longer reads {was}"
    );
    app.replacen(&handler, &changed, 1)
}

#[test]
fn test_print_is_on_file_with_its_key_and_prints_what_the_reader_shows_through_the_dialog() {
    if let Err(why) = where_print_is(&the_main_window()) {
        panic!("{why}");
    }
}

#[test]
fn test_the_reading_complains_when_print_loses_its_key() {
    let app = the_main_window();
    let planted = app.replacen(THE_LABEL, r#""&Print...""#, 1);
    assert_ne!(planted, app, "the label is not written as this expects");

    let complaint =
        where_print_is(&planted).expect_err("Print lost its key and the reading did not notice");
    assert!(complaint.contains("key or its letter"), "{complaint}");
}

#[test]
fn test_the_reading_complains_when_print_composes_its_own_document() {
    // The shape #51 had on four surfaces: a document built by hand beside the
    // one function that composes, so one surface says what the others do not.
    let planted = with_the_handler_changed(
        &the_main_window(),
        "a_message_as_the_reader_shows_it(",
        "reader_text::single_message(",
    );

    let complaint = where_print_is(&planted)
        .expect_err("the handler composed its own document and the reading did not notice");
    assert!(complaint.contains("the reader window uses"), "{complaint}");
}

#[test]
fn test_the_reading_complains_when_print_skips_the_dialog() {
    let planted = with_the_handler_changed(
        &the_main_window(),
        "ask_for_a_printer(",
        "ChosenPrinter::the_printer_named(",
    );

    let complaint = where_print_is(&planted)
        .expect_err("the handler skipped the dialog and the reading did not notice");
    assert!(
        complaint.contains("Windows' own print dialog"),
        "{complaint}"
    );
}

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
//! with dates in full, a conversation's row as the whole conversation, through
//! the one path that asks Windows' dialog for the printer, and naming the job
//! by kind so no subject reaches a shared printer's queue. And the same in the
//! reader window (13-04): Print on its File menu with the same key, going
//! through the same path, and every tab the main window opens handed its paper
//! composition, so the reader's tab prints its dates in full too.
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

/// The one path every surface prints by: Windows' dialog, then the printer
/// chosen there.
const THE_PATH: &str = "pub fn print_through_the_dialog(";

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
        "conversation_on_paper(",
        "no longer prints a conversation's row as the whole conversation, so the \
         row prints one message of several",
    ),
    (
        "print_through_the_dialog(",
        "no longer goes through the one path every surface prints by, so this \
         surface can come to print differently from the others",
    ),
    (
        "Kind::Message",
        "no longer names the job by kind, so a subject could reach a shared \
         printer's queue",
    ),
];

/// What the one path has to call, and why each one matters.
const THE_PATHS_CALLS: [(&str, &str); 2] = [
    (
        "ask_for_a_printer(",
        "no longer asks Windows' own print dialog, so the printer, the copies and \
         the pages are not chosen where they are in every other program",
    ),
    ("print_on(", "no longer sends the pages to the printer"),
];

/// Whether `text` names `call` as a whole name, not as the end of a longer
/// one: `on_paper(` inside `conversation_on_paper(` is not a call to
/// `on_paper`, and reading it as one let a handler that stopped asking for
/// the dates in full pass (found by this plan's guard re-measure, 2026-09-25).
fn calls(text: &str, call: &str) -> bool {
    text.match_indices(call).any(|(at, _)| {
        !text[..at]
            .chars()
            .next_back()
            .is_some_and(|before| before.is_alphanumeric() || before == '_')
    })
}

/// Whether the one path asks Windows' dialog and then prints.
fn the_path_asks_the_dialog(transport: &str) -> Result<(), String> {
    let path = body_of(transport, THE_PATH)?;
    for (call, why) in THE_PATHS_CALLS {
        if !calls(&path, call) {
            return Err(format!("Print {why}: {call} is not called"));
        }
    }
    Ok(())
}

/// Where Print is and what it goes through, said as a complaint when either
/// is wrong.
fn where_print_is(app: &str, transport: &str) -> Result<(), String> {
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
        if !calls(&handler, call) {
            return Err(format!("File, Print {why}: {call} is not called"));
        }
    }
    the_path_asks_the_dialog(transport)
}

/// The reader window's Print item.
const THE_READERS_ITEM: &str = "ID_READER_PRINT";

/// Where each tab the main window opens is handed what it prints, and the
/// paper composition it is handed.
const THE_HAND_OVERS: [(&str, &str); 2] = [
    ("fn open_in_the_text_reader(", "on_paper("),
    ("fn open_conversation(", "conversation_on_paper("),
];

/// Where the reader window's Print is and what it goes through, said as a
/// complaint when either is wrong.
fn where_the_reader_prints(reader: &str, app: &str) -> Result<(), String> {
    let ship = what_ships(reader);
    let file = menu_chain(&ship, "file")
        .ok_or("the reader's File menu is no longer built by that name, so this reads nothing")?;
    let print = the_call_appending(file, THE_READERS_ITEM).ok_or(
        "the reader's File menu appends no Print item, so a message open in the reader \
         cannot be printed",
    )?;
    if !print.contains(THE_LABEL) {
        return Err(format!(
            "Print on the reader's File no longer reads {THE_LABEL}, so its key or its \
             letter is not where somebody learns it: {}",
            print.split_whitespace().collect::<Vec<_>>().join(" ")
        ));
    }
    let at = |id: &str| file.find(&format!("{id},"));
    match (
        at("ID_SAVE_ATTACHMENT"),
        at(THE_READERS_ITEM),
        at("ID_CLOSE_TAB"),
    ) {
        (Some(save), Some(print), Some(close)) if save < print && print < close => {}
        _ => {
            return Err(
                "Print is no longer between Save Attachment and Close Tab on the reader's \
                 File menu"
                    .to_string(),
            );
        }
    }
    let arm = format!("if id == {THE_READERS_ITEM} {{");
    let handler = ship
        .find(&arm)
        .map(|at| &ship[at + arm.len()..])
        .map(|rest| &rest[..rest.find("if id ==").unwrap_or(rest.len())])
        .ok_or("nothing in the reader answers its Print item, so the item does nothing")?;
    // Without spaces, so the call reads the same however rustfmt wraps it.
    let squashed: String = handler.split_whitespace().collect();
    if !calls(&squashed, "print_through_the_dialog(&frame") {
        return Err(
            "the reader's Print no longer goes through the one path every surface prints by, \
             with the reader's own window owning the dialog"
                .to_string(),
        );
    }
    let app = what_ships(app);
    for (signature, paper) in THE_HAND_OVERS {
        let opens = body_of(&app, signature)?;
        if !calls(&opens, "open_with_paper(") || !calls(&opens, paper) {
            return Err(format!(
                "{signature} no longer hands the tab it opens what it prints ({paper}), so \
                 Print in that tab prints the screen's composition, dates and all"
            ));
        }
    }
    Ok(())
}

/// The function the Print arm hands a module's item to.
const THE_ITEM_HANDLER: &str = "fn print_the_item_under_the_cursor(";

/// Each module but Mail: the list the Print arm reads its row from, and the
/// thing its refusal names when no row is chosen.
const THE_MODULES: [(&str, &str, &str); 5] = [
    ("Contacts", "contact_list", "CONTACT"),
    ("Calendar", "cal_event_list", "EVENT"),
    ("Reminders", "reminder_list", "REMINDER"),
    ("Tasks", "task_list", "TASK"),
    ("Notes", "note_list", "NOTE"),
];

/// What the item handler has to call, and why each one matters.
const THE_ITEM_CALLS: [(&str, &str); 4] = [
    (
        "for_module(",
        "no longer names the job by the module's kind, so an event could be queued \
         as something else",
    ),
    (
        "from_item(",
        "no longer prints the fields the item's reading says, so paper and speech \
         can come to name different things",
    ),
    (
        "print_through_the_dialog(",
        "no longer goes through the one path every surface prints by",
    ),
    (
        "nothing_chosen(",
        "no longer says nothing is chosen when no row is, so Print does nothing",
    ),
];

/// Where Print in the other five modules goes, said as a complaint when it
/// goes wrong.
fn where_the_modules_print(app: &str) -> Result<(), String> {
    let ship = what_ships(app);
    let arm = "_ if id == ID_PRINT =>";
    let the_arm = ship
        .find(arm)
        .map(|at| &ship[at + arm.len()..])
        .map(|rest| &rest[..rest.find("_ if id ==").unwrap_or(rest.len())])
        .ok_or("nothing answers ID_PRINT, so the item does nothing")?;
    let squashed: String = the_arm.split_whitespace().collect();
    for (module, list, thing) in THE_MODULES {
        // rustfmt puts an arm too long for its line in braces, so the answer
        // is read with or without them.
        let answer = format!("Some((selected_row(&{list}),Thing::{thing}))");
        let arm = format!("PimModule::{module}=>");
        let answered = squashed.match_indices(&arm).any(|(at, _)| {
            let after = &squashed[at + arm.len()..];
            after
                .strip_prefix('{')
                .unwrap_or(after)
                .starts_with(&answer)
        });
        if !answered {
            return Err(format!(
                "the ID_PRINT arm no longer reads {module}'s row from {list} with a refusal \
                 naming a {thing}, so Print in {module} prints nothing or refuses in another \
                 module's words"
            ));
        }
    }
    if !calls(the_arm, &THE_ITEM_HANDLER["fn ".len()..]) {
        return Err(format!(
            "the ID_PRINT arm no longer hands a module's item to {THE_ITEM_HANDLER}"
        ));
    }
    let handler = body_of(&ship, THE_ITEM_HANDLER)?;
    for (call, why) in THE_ITEM_CALLS {
        if !calls(&handler, call) {
            return Err(format!("Print in a module {why}: {call} is not called"));
        }
    }
    if calls(&ship, "prints_messages_only(") {
        return Err(
            "the main window still says Print works on messages only, which it no longer is"
                .to_string(),
        );
    }
    Ok(())
}

fn the_main_window() -> String {
    fs::read_to_string("src/presentation/wx_app.rs").expect("the main window")
}

fn the_reader() -> String {
    fs::read_to_string("src/presentation/wx_reader.rs").expect("the reader window")
}

fn the_transport() -> String {
    fs::read_to_string("src/presentation/printing.rs").expect("the printing transport")
}

/// `source` with `was` replaced by `now` once inside the function `signature`.
fn with_the_body_changed(source: &str, signature: &str, was: &str, now: &str) -> String {
    let body = body_of(source, signature).expect("the function");
    let changed = body.replacen(was, now, 1);
    assert_ne!(
        changed, body,
        "the plant changed nothing, so {signature} no longer reads {was}"
    );
    source.replacen(&body, &changed, 1)
}

#[test]
fn test_print_is_on_file_with_its_key_and_prints_what_the_reader_shows_through_the_dialog() {
    if let Err(why) = where_print_is(&the_main_window(), &the_transport()) {
        panic!("{why}");
    }
}

#[test]
fn test_the_reading_complains_when_print_loses_its_key() {
    let app = the_main_window();
    let planted = app.replacen(THE_LABEL, r#""&Print...""#, 1);
    assert_ne!(planted, app, "the label is not written as this expects");

    let complaint = where_print_is(&planted, &the_transport())
        .expect_err("Print lost its key and the reading did not notice");
    assert!(complaint.contains("key or its letter"), "{complaint}");
}

#[test]
fn test_the_reading_complains_when_print_composes_its_own_document() {
    // The shape #51 had on four surfaces: a document built by hand beside the
    // one function that composes, so one surface says what the others do not.
    let planted = with_the_body_changed(
        &the_main_window(),
        THE_HANDLER,
        "a_message_as_the_reader_shows_it(",
        "reader_text::single_message(",
    );

    let complaint = where_print_is(&planted, &the_transport())
        .expect_err("the handler composed its own document and the reading did not notice");
    assert!(complaint.contains("the reader window uses"), "{complaint}");
}

#[test]
fn test_the_reading_complains_when_print_skips_the_dialog() {
    let planted = with_the_body_changed(
        &the_transport(),
        THE_PATH,
        "ask_for_a_printer(",
        "ChosenPrinter::the_printer_named(",
    );

    let complaint = where_print_is(&the_main_window(), &planted)
        .expect_err("the path skipped the dialog and the reading did not notice");
    assert!(
        complaint.contains("Windows' own print dialog"),
        "{complaint}"
    );
}

#[test]
fn test_the_reader_prints_its_tab_through_the_dialog_with_its_key() {
    if let Err(why) = where_the_reader_prints(&the_reader(), &the_main_window()) {
        panic!("{why}");
    }
}

#[test]
fn test_the_reading_complains_when_the_readers_print_loses_its_key() {
    let reader = the_reader();
    let planted = reader.replacen(THE_LABEL, r#""&Print...""#, 1);
    assert_ne!(
        planted, reader,
        "the reader's label is not written as this expects"
    );

    let complaint = where_the_reader_prints(&planted, &the_main_window())
        .expect_err("the reader's Print lost its key and the reading did not notice");
    assert!(complaint.contains("key or its letter"), "{complaint}");
}

#[test]
fn test_the_reading_complains_when_a_tab_is_opened_with_nothing_to_print() {
    // A tab opened the old way prints what the screen composed, so its dates
    // say "2 days ago" on paper.
    let planted = with_the_body_changed(
        &the_main_window(),
        "fn open_in_the_text_reader(",
        "open_with_paper(",
        "open(",
    );

    let complaint = where_the_reader_prints(&the_reader(), &planted)
        .expect_err("a tab was opened with nothing to print and the reading did not notice");
    assert!(complaint.contains("open_in_the_text_reader"), "{complaint}");
}

#[test]
fn test_print_in_every_module_prints_the_item_under_the_cursor() {
    // #45: "In the other modules, Print prints the open event, contact, task,
    // note or reminder the way it is shown."
    if let Err(why) = where_the_modules_print(&the_main_window()) {
        panic!("{why}");
    }
}

#[test]
fn test_the_reading_complains_when_a_module_is_answered_as_mail() {
    // The shape 13-03 left: one module sent to the messages-only answer, so
    // Print there says it works on messages.
    let app = the_main_window();
    let planted = app.replacen(
        "Some((selected_row(&cal_event_list), Thing::EVENT))",
        "None",
        1,
    );
    assert_ne!(
        planted, app,
        "Calendar's answer is not written as this expects"
    );

    let complaint = where_the_modules_print(&planted)
        .expect_err("Calendar was answered as Mail and the reading did not notice");
    assert!(complaint.contains("Calendar"), "{complaint}");
}

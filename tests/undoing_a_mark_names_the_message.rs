//! Edit, Undo in the message list takes back the last mark, star or label,
//! each message as it was, at the server the way the action went, and the
//! menu names what it will undo (#47's second level, 13-07).
//!
//! The tester on 2026-09-15 asked for "the last delete, move, copy, mark as
//! read or unread, star ... as 'Undo delete' or 'Undo move' with the item
//! named, for a bounded time or until the next action; and Redo of that."
//! What an undo changes and what it is called is `application::undoing`'s,
//! tested there without a window. This reads the window's half, over
//! `what_ships` of `src/presentation/wx_app.rs`:
//!
//! - the three commands that mark, Mark as Read, Star and a label, each
//!   remember the action once, after its writes, so a write that failed part
//!   way is never remembered as done;
//! - the mark the program makes by itself after the reading wait remembers
//!   nothing, because it is not something somebody did, and an undo that
//!   undid it would unread a message nobody marked;
//! - Undo and Redo in the message list reach the carrying out, which asks
//!   `application::undoing` what changes and puts each mark on through the
//!   same write and the same queue to the server the action used, so a
//!   refusal puts it back and says so as it always did;
//! - one sentence per undo, never one per message (guardrail 5);
//! - the Edit menu, with the message list focused, names the step.
//!
//! Each reading is a function over text, and a companion hands it the fault
//! planted in a snippet shaped as the window should be, so a reading that
//! stopped finding its anchor cannot pass by finding nothing.
//!
//! # Why a reading and not a built window
//!
//! The message list, the cache and the queue to the server are the main
//! window's, and `src/presentation/wx_app.rs` is named by more than a hundred
//! guard records, so a test added there is that many builds at the next
//! commit. This file is named by its own records, whose `suite` couples it to
//! the window, so it runs on the commits that could break it.
//!
//! # What this cannot see
//!
//! Whether the undo is heard, whether the menu's words read well by ear, and
//! what a real server does with a flag put back: the tester's ear and phase
//! 14's accounts. The window is not started.

use std::fs;

use wixen_mail::common::what_ships::what_ships;

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

fn the_main_window() -> String {
    let whole = fs::read_to_string(THE_MAIN_WINDOW)
        .unwrap_or_else(|why| panic!("{THE_MAIN_WINDOW}: {why}"))
        .replace("\r\n", "\n");
    what_ships(&whole)
}

/// One function's text, from its signature to the closing brace at column
/// nought, or a complaint when the signature is gone.
fn body_of<'a>(source: &'a str, signature: &str) -> Result<&'a str, String> {
    let at = source.find(signature).ok_or(format!(
        "{signature} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let ends = rest.find("\n}\n").map_or(rest.len(), |end| end + 2);
    Ok(&rest[..ends])
}

/// The body of one `_ if id == ...` arm of the command dispatch, up to the
/// next arm of the same shape.
fn the_id_arm<'a>(source: &'a str, heading: &str) -> Result<&'a str, String> {
    let start = source.find(heading).ok_or(format!(
        "{heading:?} is no longer here, so this reads nothing"
    ))? + heading.len();
    let rest = &source[start..];
    let end = rest.find("_ if id ==").unwrap_or(rest.len());
    Ok(&rest[..end])
}

/// The text of each `for` loop's body in `text`, brace to matching brace.
fn loop_bodies(text: &str) -> Vec<&str> {
    let mut bodies = Vec::new();
    let mut from = 0;
    while let Some(found) = text[from..].find("for ") {
        let at = from + found;
        from = at + 4;
        let starts_a_line = text[..at].ends_with(' ') && {
            let line_start = text[..at].rfind('\n').map_or(0, |n| n + 1);
            text[line_start..at].trim().is_empty()
        };
        if !starts_a_line {
            continue;
        }
        let Some(open) = text[at..].find('{').map(|n| at + n) else {
            break;
        };
        let mut depth = 0usize;
        for (offset, c) in text[open..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        bodies.push(&text[open..open + offset + 1]);
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    bodies
}

// ── The anchors, each a name or a literal ──────────────────────────────────

const THE_TOGGLE: &str = "fn toggle_read_state(";
const THE_STAR_ARM: &str = "_ if id == ID_TOGGLE_STAR =>";
const THE_LABELS: &str = "fn label_the_message(";
const THE_PROGRAMS_OWN_MARK: &str = "fn mark_what_was_read(";
const THE_EDIT_COMMAND: &str = "fn do_an_edit_command(";
const THE_CARRYING_OUT: &str = "fn take_back_or_do_again(";
const ONE_MARK_PUT_ON: &str = "fn put_a_mark_on(";
const THE_MENU_WORDS: &str = "fn name_the_step_on_the_edit_menu(";
const THE_MENU_HANDLERS: &str = "fn keep_the_edit_menu_honest(";
const REMEMBERS: &str = "remember_the_last_action(";
const A_WRITE_TO_THE_SERVER: &str = "spawn_server_change(";
const SAYS: [&str; 2] = ["say_what_the_undo_did(", ".announce("];

// ── The readings ───────────────────────────────────────────────────────────

/// The command remembers its action exactly once, after the last change it
/// sends to the server, so a command that stopped part way through a set
/// remembers nothing it did not finish.
fn remembers_after_its_writes(body: &str, what: &str) -> Result<(), String> {
    let times = body.matches(REMEMBERS).count();
    if times != 1 {
        return Err(format!(
            "{what} remembers its action {times} times, where once after its writes is right"
        ));
    }
    let remembered = body.find(REMEMBERS).unwrap_or_default();
    let last_write = body.rfind(A_WRITE_TO_THE_SERVER).ok_or(format!(
        "{what} sends nothing to the server, so this reads nothing"
    ))?;
    match remembered > last_write {
        true => Ok(()),
        false => Err(format!(
            "{what} remembers its action before its last write, so a write that fails \
             part way leaves an undo for changes that were never made"
        )),
    }
}

/// The program's own mark after the reading wait remembers nothing.
fn never_remembers(body: &str) -> Result<(), String> {
    match body.contains(REMEMBERS) || body.contains("last_action") {
        true => Err(
            "the mark made after the reading wait replaces the last action, so Undo would \
             unread a message nobody marked and lose what somebody did"
                .to_string(),
        ),
        false => Ok(()),
    }
}

/// Undo and Redo in the message list reach the carrying out, which asks what
/// changes and puts each mark on the way the action did.
fn goes_the_way_the_action_went(app: &str) -> Result<(), String> {
    let edit = body_of(app, THE_EDIT_COMMAND)?;
    let arm = edit.split_once("Doing::TheLastAction =>").ok_or(
        "the Edit command has no arm for the last action, so this reads nothing".to_string(),
    )?;
    if !arm.1.contains("take_back_or_do_again(") {
        return Err("Undo in a list never reaches the carrying out".to_string());
    }
    let carrying = body_of(app, THE_CARRYING_OUT)?;
    for asked in ["what_undo_does(", "what_redo_does(", "put_a_mark_on("] {
        if !carrying.contains(asked) {
            return Err(format!("the carrying out never calls {asked}"));
        }
    }
    let one = body_of(app, ONE_MARK_PUT_ON)?;
    for path in [A_WRITE_TO_THE_SERVER, "write_flags_or_put_the_row_back("] {
        if !one.contains(path) {
            return Err(format!(
                "a mark put back does not go through {path}, the path the action took, \
                 so a refusal would not put it back"
            ));
        }
    }
    Ok(())
}

/// No sentence inside a loop over the messages: one per undo.
fn says_one_sentence(body: &str) -> Result<(), String> {
    let loops = loop_bodies(body);
    if loops.is_empty() {
        return Err("the carrying out has no loop over the messages, so this reads nothing".into());
    }
    if let Some(inside) = loops
        .iter()
        .find(|looped| SAYS.iter().any(|say| looped.contains(say)))
    {
        return Err(format!(
            "a sentence is said inside the loop over the messages, one per message: {inside}"
        ));
    }
    match SAYS.iter().any(|say| body.contains(say)) {
        true => Ok(()),
        false => Err("the undo says nothing at all".to_string()),
    }
}

/// With the message list focused, the open menu names the step.
fn the_menu_names_the_step(app: &str) -> Result<(), String> {
    let words = body_of(app, THE_MENU_WORDS)?;
    if !words.contains("menu_label(") {
        return Err("the Edit menu's words never ask undoing::menu_label".to_string());
    }
    let handlers = body_of(app, THE_MENU_HANDLERS)?;
    if !handlers.contains("name_the_step_on_the_edit_menu(") {
        return Err("the menu's open handler never names the step".to_string());
    }
    match app.contains("keep_the_edit_menu_honest(\n                &frame,")
        && app.contains("Some((msg_list, state.clone()))")
    {
        true => Ok(()),
        false => Err("the main window never hands the Edit menu its message list".to_string()),
    }
}

// ── The tests ──────────────────────────────────────────────────────────────

#[test]
fn test_marking_read_remembers_the_action_after_its_writes() {
    let app = the_main_window();
    let body = body_of(&app, THE_TOGGLE).unwrap_or_else(|why| panic!("{why}"));
    remembers_after_its_writes(body, "Mark as Read").unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_starring_remembers_the_action_after_its_writes() {
    let app = the_main_window();
    let arm = the_id_arm(&app, THE_STAR_ARM).unwrap_or_else(|why| panic!("{why}"));
    remembers_after_its_writes(arm, "Star").unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_labelling_remembers_the_action_after_its_writes() {
    let app = the_main_window();
    let body = body_of(&app, THE_LABELS).unwrap_or_else(|why| panic!("{why}"));
    remembers_after_its_writes(body, "a label").unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_programs_own_mark_after_the_reading_wait_is_not_remembered() {
    let app = the_main_window();
    let body = body_of(&app, THE_PROGRAMS_OWN_MARK).unwrap_or_else(|why| panic!("{why}"));
    // The reading must find the function whole, or it reads nothing.
    assert!(body.contains(A_WRITE_TO_THE_SERVER), "{body}");
    never_remembers(body).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_undo_in_the_message_list_sends_each_change_the_way_the_action_went() {
    goes_the_way_the_action_went(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_undo_says_one_sentence_however_many_messages() {
    let app = the_main_window();
    let body = body_of(&app, THE_CARRYING_OUT).unwrap_or_else(|why| panic!("{why}"));
    says_one_sentence(body).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_edit_menu_names_the_last_action_when_the_list_has_focus() {
    the_menu_names_the_step(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_companion_a_path_that_remembers_before_it_writes_is_refused() {
    let planted = "fn toggle_read_state(app: AppHandles<'_>) {
    remember_the_last_action(state, action);
    for message in &chosen.messages {
        spawn_server_change(app, message.row_id, message.uid, subject, change);
    }
}
";
    assert!(remembers_after_its_writes(planted, "the planted toggle").is_err());
}

#[test]
fn test_companion_an_undo_that_speaks_per_message_is_refused() {
    let planted = "fn take_back_or_do_again(command: EditCommand) {
    for (message, mark) in &changes {
        put_a_mark_on(app, cache, message, mark);
        say_what_the_undo_did(frame, a11y, &said, Priority::Normal);
    }
}
";
    assert!(says_one_sentence(planted).is_err());
}

#[test]
fn test_companion_the_programs_own_mark_remembered_is_refused() {
    let planted = "fn mark_what_was_read(app: AppHandles<'_>) {
    spawn_server_change(app, row, uid, subject, change);
    remember_the_last_action(state, action);
}
";
    assert!(never_remembers(planted).is_err());
}

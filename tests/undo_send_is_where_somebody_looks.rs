//! Undo Send is where somebody looks for it, and the comments about the
//! composer's keys name keys that exist.
//!
//! The tester's words on 2026-09-15 were "undo send should be in the edit
//! menu" (#44). It was on Tools, between the address book commands and Flush
//! Outbox, and the Edit menu held Cut, Copy, Paste and Select All with no undo
//! of any kind. `Ctrl+Shift+Z` is Redo's key everywhere else on Windows, and
//! Undo Send is the one undo this program has, so somebody looking for it
//! opens Edit, and somebody working by ear opens Edit, hears four clipboard
//! commands and Search, and has no way to tell a command they walked past from
//! one that is not there.
//!
//! `tests/wired.rs` holds the item to its handler by id and says nothing about
//! which menu it is on, which is why the move broke nothing there and why this
//! reading exists: the id is what makes the key work, and the menu is what
//! makes the command findable, and only one of those was held.
//!
//! Read from the source rather than built, because a menu's order is a
//! property of the builder chain and the chain is one function's text. Each
//! reading is a function over that text with a companion that plants the old
//! shape and requires a complaint, so a reading that passes over anything is
//! found here rather than by the next tester.

use std::fs;
use wixen_mail::common::what_ships::what_ships;

/// The builder chain for the menu bound to `name`, from `let name = Menu::builder()`
/// to its `.build()`.
///
/// `None` when the file no longer builds a menu by that name, so the reading
/// complains about that rather than reading an empty string and passing.
fn menu_chain<'a>(ship: &'a str, name: &str) -> Option<&'a str> {
    let opens = ship.find(&format!("let {name} = Menu::builder()"))?;
    let chain = &ship[opens..];
    let closes = chain.find(".build()")?;
    Some(&chain[..closes])
}

/// The text of the `.append_item(` call whose first argument is `id`, from the
/// dot to the closing bracket, as it stands in `text`.
fn the_call_appending<'a>(text: &'a str, id: &str) -> Option<&'a str> {
    let mut from = 0;
    while let Some(at) = text[from..].find(".append_item(") {
        let opens = from + at;
        let after = &text[opens + ".append_item(".len()..];
        if after.trim_start().starts_with(id) {
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

/// The one place Undo Send is offered on the menu bar, said as a complaint
/// when it is not there.
///
/// First on Edit, with its key in the label, and not on Tools. Both halves,
/// because an item copied rather than moved is offered in two places and the
/// place it was moved to stops being the only answer.
fn where_undo_send_is(app: &str) -> Result<(), String> {
    let ship = what_ships(app);
    let edit = menu_chain(&ship, "edit")
        .ok_or("the Edit menu is no longer built by that name, so this reads nothing")?;
    let first = the_call_appending(edit, "ID_")
        .ok_or("the Edit menu appends no item at all, so this reads nothing")?;
    if !first.contains("ID_UNDO_SEND") {
        return Err(format!(
            "the first item on the Edit menu is not Undo Send, so somebody looking \
             for the one undo this program has opens Edit and does not find it: {}",
            first.split_whitespace().collect::<Vec<_>>().join(" ")
        ));
    }
    if !first.contains("Ctrl+Shift+Z") {
        return Err(
            "Undo Send is first on Edit but its label no longer carries Ctrl+Shift+Z, \
                    so the menu stops being where somebody learns the key"
                .to_string(),
        );
    }
    let tools = menu_chain(&ship, "tools")
        .ok_or("the Tools menu is no longer built by that name, so this reads nothing")?;
    if tools.contains("ID_UNDO_SEND") {
        return Err(
            "Undo Send is still on the Tools menu as well, so it is offered in two places \
             and Edit is not the only answer"
                .to_string(),
        );
    }
    Ok(())
}

/// The doc comment on the composer's schedule routine names a key that
/// exists, said as a complaint when it names one that does not.
///
/// The routine is the one way in for both the toolbar button and the key, and
/// its comment said the key was Alt+E for as long as the key was Alt+H (#56).
/// A comment is where the next person learns which key to test, so a wrong one
/// sends them to test a key that does nothing and conclude the routine is
/// unreachable.
fn which_key_the_schedule_comment_names(compose: &str) -> Result<(), String> {
    let signature = "fn ask_when_and_close(";
    let at = compose.find(signature).ok_or(
        "the schedule routine is no longer called ask_when_and_close, so this reads nothing",
    )?;
    let comment: Vec<&str> = compose[..at]
        .lines()
        .rev()
        .take_while(|line| line.trim_start().starts_with("///"))
        .collect();
    if comment.is_empty() {
        return Err(
            "the schedule routine has no doc comment, so nothing says how it is reached"
                .to_string(),
        );
    }
    let comment = comment.join("\n");
    if comment.contains("Alt+E") {
        return Err(
            "the schedule routine's comment names Alt+E, a key that does nothing in \
                    the composer; the key is Alt+H"
                .to_string(),
        );
    }
    if !comment.contains("Alt+H") {
        return Err(
            "the schedule routine's comment no longer names Alt+H, so the one place \
                    that says how the key reaches it says nothing"
                .to_string(),
        );
    }
    Ok(())
}

fn the_main_window() -> String {
    fs::read_to_string("src/presentation/wx_app.rs").expect("the main window")
}

fn the_composer() -> String {
    fs::read_to_string("src/presentation/wx_compose.rs").expect("the composer")
}

/// The main window's text with the Undo Send item taken off Edit and put back
/// on Tools, which is the shape the tester met.
fn with_undo_send_back_on_tools(app: &str) -> String {
    let call = the_call_appending(app, "ID_UNDO_SEND")
        .expect("an append_item naming ID_UNDO_SEND, or the plant has nothing to move");
    let call = call.to_string();
    let moved = app.replacen(&call, "", 1);
    let tools = "let tools = Menu::builder()";
    assert!(
        moved.contains(tools),
        "the Tools menu is no longer built by that name"
    );
    moved.replacen(tools, &format!("{tools}\n            {call}"), 1)
}

#[test]
fn test_undo_send_is_the_first_item_on_the_edit_menu_and_not_on_tools() {
    // The tester's sentence, held. The key is not moving and the handler is
    // not moving; what moves is where a person finds the command.
    let app = the_main_window();

    if let Err(why) = where_undo_send_is(&app) {
        panic!("{why}");
    }
}

#[test]
fn test_the_reading_complains_when_undo_send_is_put_back_on_tools() {
    // The companion. Without it, a reading that found no Edit menu and passed,
    // or read the wrong chain, would look exactly like one that holds the item
    // in place.
    let planted = with_undo_send_back_on_tools(&the_main_window());

    let complaint = where_undo_send_is(&planted)
        .expect_err("Undo Send was put back on Tools and the reading did not notice");
    assert!(
        complaint.contains("first item on the Edit menu"),
        "the reading complained about something other than the missing item: {complaint}"
    );
}

#[test]
fn test_the_reading_complains_when_undo_send_is_offered_on_both_menus() {
    // The other half: copied rather than moved. Edit is right and Tools still
    // has it, so the item is in two places and neither is the only answer.
    let app = the_main_window();
    let call = the_call_appending(&app, "ID_UNDO_SEND")
        .expect("an append_item naming ID_UNDO_SEND")
        .to_string();
    let tools = "let tools = Menu::builder()";
    let planted = app.replacen(tools, &format!("{tools}\n            {call}"), 1);

    let complaint = where_undo_send_is(&planted)
        .expect_err("Undo Send was copied onto Tools as well and the reading did not notice");
    assert!(
        complaint.contains("still on the Tools menu"),
        "the reading complained about something other than the copy: {complaint}"
    );
}

#[test]
fn test_the_schedule_routines_comment_names_the_key_that_reaches_it() {
    let compose = the_composer();

    if let Err(why) = which_key_the_schedule_comment_names(&compose) {
        panic!("{why}");
    }
}

#[test]
fn test_the_reading_complains_when_the_comment_names_a_key_that_does_nothing() {
    // The companion plants the stale key where the right one is. A reading
    // that took the wrong run of lines for the comment would pass over the
    // plant, and this is what says it did not.
    let compose = the_composer();
    let planted = compose.replacen("Alt+H out of the", "Alt+E out of the", 1);
    assert!(
        planted != compose,
        "the plant changed nothing, so the comment no longer reads as this expects"
    );

    let complaint = which_key_the_schedule_comment_names(&planted)
        .expect_err("the comment named Alt+E and the reading did not notice");
    assert!(complaint.contains("Alt+E"), "{complaint}");
}

//! Report as Junk is on the Action menu, acts on the selection, meets the
//! gate per account, and marks at the server before it moves (#54, GAP-06's
//! first `[D]` line, 13-22).
//!
//! Two kinds of check. The built menu bar, `cfg(windows)`: the item on
//! Action with its letter and its key, the letter claimed once on that menu,
//! and a description saying the command has never met a real mail server.
//! And source readings over `what_ships` of `src/presentation/wx_app.rs`:
//! the arm reaches the handler; the handler reads the selection the way Move
//! does and refuses above the bound; it asks `reporting_junk` per account
//! before any worker starts; the worker marks before it asks for the move;
//! the move is Move's own, handed the report's sentence; and that sentence
//! replaces Move's for the set. Each reading has a companion that plants the
//! fault into a snippet shaped as the window should be, so a reading that
//! stopped finding its anchor cannot pass by finding nothing.
//!
//! # Why this lives here rather than beside the code
//!
//! `src/presentation/wx_app.rs` is named by more than a hundred guard
//! records, so a test added there is that many builds at the next commit.
//! This target is named by its own records, whose `suite` couples it to the
//! window.
//!
//! # What this cannot see
//!
//! Whether any server keeps `$Junk` and trains on it, whether Gmail counts
//! the move into Spam, and what the sentence sounds like; the window is not
//! started and no server is met.

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
fn body_of(source: &str, signature: &str) -> Result<String, String> {
    let at = source.find(signature).ok_or(format!(
        "{signature} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let ends = rest.find("\n}\n").map_or(rest.len(), |end| end + 2);
    Ok(rest[..ends].to_string())
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

/// Where `first` is in `text`, and that `second` comes after it, or a
/// complaint naming which is missing or which came first.
fn comes_before(text: &str, first: &str, second: &str, fault: &str) -> Result<(), String> {
    let first_at = text
        .find(first)
        .ok_or(format!("{first:?} is not here, so this reads nothing"))?;
    let second_at = text
        .find(second)
        .ok_or(format!("{second:?} is not here, so this reads nothing"))?;
    match first_at < second_at {
        true => Ok(()),
        false => Err(fault.to_string()),
    }
}

// ── The anchors ────────────────────────────────────────────────────────────

const THE_ARM: &str = "_ if id == ID_REPORT_JUNK =>";
const THE_HANDLER: &str = "fn report_the_chosen_as_junk(";
const THE_WORKER: &str = "fn spawn_junk_marking(";
const THE_MOVE_AFTER: &str = "fn move_what_was_reported(";
const THE_UPDATE_ARM: &str = "UIUpdate::ReportedAsJunk(ready) =>";
const THE_SET_MOVE: &str = "fn move_or_copy_here_first(";

// ── The readings ───────────────────────────────────────────────────────────

/// The Action item's arm reaches the handler.
fn the_arm_reaches_the_report(app: &str) -> Result<(), String> {
    let arm = the_id_arm(app, THE_ARM)?;
    match arm.contains("report_the_chosen_as_junk(") {
        true => Ok(()),
        false => {
            Err("the Report as Junk arm does not call the report, so the key does nothing".into())
        }
    }
}

/// The handler reads the selection the way Move does, a conversation row
/// giving the messages in the folder being read, and refuses above the
/// bound; it never reads the cursor row alone.
fn the_report_reads_the_selection(handler: &str) -> Result<(), String> {
    if !handler.contains("chosen_messages(") {
        return Err("the report does not read the selection, so it acts on one row".into());
    }
    if !handler.contains("AConversationReaches::ThisFolderOnly") {
        return Err(
            "the report reads a conversation's messages outside the folder being read, which \
             Move never does"
                .into(),
        );
    }
    if !handler.contains("too_many(") {
        return Err("the report does not refuse above the bound a set command keeps".into());
    }
    match handler.contains("selected_message_index") {
        true => Err("the report reads the cursor row rather than the selection".into()),
        false => Ok(()),
    }
}

/// The handler asks `reporting_junk` what each account's report does, which
/// is where the gate is met, before any worker starts.
fn the_report_decides_before_a_worker_starts(handler: &str) -> Result<(), String> {
    comes_before(
        handler,
        "reporting_junk::what_a_report_does(",
        "spawn_junk_marking(",
        "the worker starts before each account's report was decided, so an account whose \
         changes are off could be written to",
    )
}

/// The worker marks at the server, through the account's held session,
/// before it asks for the move.
fn the_worker_marks_before_it_asks_the_move(worker: &str) -> Result<(), String> {
    if !worker.contains("mail_session::the_session_at(") {
        return Err("the worker marks through a session other than the account's held one".into());
    }
    comes_before(
        worker,
        "reporting_junk::mark_as_junk_at_the_server(",
        "UIUpdate::ReportedAsJunk(",
        "the move is asked for before the mark, so the message has left the folder its \
         number belongs to when the mark is sent",
    )
}

/// The move is Move's own, gated path, handed the report's sentence.
fn the_move_is_moves_own_with_the_reports_sentence(app: &str) -> Result<(), String> {
    let arm_at = app.find(THE_UPDATE_ARM).ok_or(format!(
        "{THE_UPDATE_ARM:?} is not here, so this reads nothing"
    ))?;
    let arm = &app[arm_at..app.len().min(arm_at + 400)];
    if !arm.contains("move_what_was_reported(") {
        return Err("the update the worker sends moves nothing".into());
    }
    let mover = body_of(app, THE_MOVE_AFTER)?;
    if !mover.contains("move_or_copy_here_first(") {
        return Err("a report moves by a path of its own rather than Move's gated one".into());
    }
    match mover.contains("said_for_the_set: Some(") {
        true => Ok(()),
        false => Err("the report's sentence is not handed to the move, so Move's is said".into()),
    }
}

/// Move's own sentence for a set is not built when a command handed one of
/// its own, and that one is spoken once the change was made here.
fn the_reports_sentence_replaces_moves(set_move: &str) -> Result<(), String> {
    if !set_move.contains("Some(_) => None,") {
        return Err("Move's sentence for the set is built beside the report's".into());
    }
    let at = set_move
        .find("if let Some(said) = said_for_the_set")
        .ok_or("the sentence a command handed the move is never said")?;
    let after = &set_move[at..set_move.len().min(at + 200)];
    match after.contains("send_status(tx, rt, &said)") {
        true => Ok(()),
        false => Err("the sentence a command handed the move is not spoken".into()),
    }
}

// ── The checks, each of which a companion hands a wrong state ─────────────

#[test]
fn test_the_report_as_junk_arm_reaches_the_report() {
    the_arm_reaches_the_report(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_arm_reading_sees_an_arm_that_does_nothing() {
    let planted = "_ if id == ID_REPORT_JUNK => {\n    let _ = id;\n}\n_ if id == ID_OTHER => {}";
    assert!(the_arm_reaches_the_report(planted).is_err());
}

#[test]
fn test_the_report_reads_the_selection_and_refuses_above_the_bound() {
    let handler = body_of(&the_main_window(), THE_HANDLER).unwrap_or_else(|why| panic!("{why}"));
    the_report_reads_the_selection(&handler).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_selection_reading_sees_a_report_off_the_cursor_row() {
    let planted = "fn report_the_chosen_as_junk() {\n    let chosen = chosen_messages(state, \
                   cache, list, AConversationReaches::ThisFolderOnly);\n    too_many(1);\n    \
                   let row = lock_state(state).selected_message_index;\n}\n";
    let why = the_report_reads_the_selection(planted).expect_err("the cursor row was not seen");
    assert!(why.contains("cursor row"), "{why}");
}

#[test]
fn test_the_report_decides_each_account_before_any_worker_starts() {
    let handler = body_of(&the_main_window(), THE_HANDLER).unwrap_or_else(|why| panic!("{why}"));
    the_report_decides_before_a_worker_starts(&handler).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_decision_reading_sees_a_worker_started_first() {
    let planted = "fn report_the_chosen_as_junk() {\n    spawn_junk_marking(app, reports);\n    \
                   reporting_junk::what_a_report_does(kind, junk, gate);\n}\n";
    assert!(the_report_decides_before_a_worker_starts(planted).is_err());
}

#[test]
fn test_the_worker_marks_at_the_server_before_it_asks_the_move() {
    let worker = body_of(&the_main_window(), THE_WORKER).unwrap_or_else(|why| panic!("{why}"));
    the_worker_marks_before_it_asks_the_move(&worker).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_order_reading_sees_the_move_asked_before_the_mark() {
    let planted = "fn spawn_junk_marking() {\n    let session = \
                   mail_session::the_session_at(&account);\n    \
                   tx.send(UIUpdate::ReportedAsJunk(ready));\n    \
                   reporting_junk::mark_as_junk_at_the_server(&session, folder, &uids);\n}\n";
    let why =
        the_worker_marks_before_it_asks_the_move(planted).expect_err("the order was not seen");
    assert!(why.contains("before the mark"), "{why}");
}

#[test]
fn test_the_report_moves_through_moves_own_path_with_its_sentence() {
    the_move_is_moves_own_with_the_reports_sentence(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_move_reading_sees_a_move_without_the_reports_sentence() {
    let planted = "UIUpdate::ReportedAsJunk(ready) => {\n    move_what_was_reported(app, ready);\n}\n\
                   fn move_what_was_reported() {\n    move_or_copy_here_first(app, list, cache, \
                   AMoveAsked { said_for_the_set: None });\n}\n";
    assert!(the_move_is_moves_own_with_the_reports_sentence(planted).is_err());
}

#[test]
fn test_the_reports_sentence_replaces_moves_for_the_set() {
    let set_move = body_of(&the_main_window(), THE_SET_MOVE).unwrap_or_else(|why| panic!("{why}"));
    the_reports_sentence_replaces_moves(&set_move).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_sentence_reading_sees_the_reports_sentence_dropped() {
    let planted = "fn move_or_copy_here_first() {\n    let one_sentence = match &said_for_the_set \
                   {\n        Some(_) => None,\n        None => a_set.then(|| what_was_done()),\n    \
                   };\n    if let Some(said) = None::<String> {\n        send_status(tx, rt, \
                   &said);\n    }\n}\n";
    assert!(the_reports_sentence_replaces_moves(planted).is_err());
}

// ── The built menu bar ─────────────────────────────────────────────────────

#[cfg(windows)]
mod the_built_menu {
    use std::sync::{Arc, Mutex, OnceLock};

    use wixen_mail::presentation::wx_app::WxMailApp;
    use wxdragon::prelude::*;

    /// What the Action menu offers: every item's label, in order, with its
    /// description, and each submenu's label.
    #[derive(Debug, Clone)]
    struct TheActionMenu {
        items: Vec<(String, String)>,
    }

    fn read_the_action_menu() -> Result<TheActionMenu, String> {
        let outcome: Arc<Mutex<Option<Result<TheActionMenu, String>>>> = Arc::new(Mutex::new(None));
        let result = {
            let outcome = outcome.clone();
            wxdragon::main(move |app| {
                let taken = (|| {
                    let frame = Frame::builder()
                        .with_title("Report as Junk is on the Action menu, the reading")
                        .build();
                    frame.set_menu_bar(WxMailApp::build_menu_bar());
                    let bar = frame.get_menu_bar().ok_or("no menu bar")?;
                    let action = usize::try_from(bar.find_menu("Action"))
                        .ok()
                        .and_then(|at| bar.get_menu(at))
                        .ok_or("no Action menu")?;
                    let items = action
                        .get_menu_items()
                        .iter()
                        .map(|item| (item.get_label(), action.get_help_string(item.get_item_id())))
                        .collect();
                    frame.destroy();
                    Ok(TheActionMenu { items })
                })();
                if let Ok(mut slot) = outcome.lock() {
                    *slot = Some(taken);
                }
                wxdragon::call_after(Box::new(move || {
                    app.exit_main_loop();
                }));
            })
        };
        if let Err(why) = result {
            return Err(format!("wxdragon::main returned {why:?}"));
        }
        let taken = outcome
            .lock()
            .map_err(|_| "the reading's lock was poisoned".to_string())?
            .take();
        taken.unwrap_or_else(|| Err("the window session ended without a reading".to_string()))
    }

    fn the_action_menu() -> &'static TheActionMenu {
        static READ: OnceLock<Result<TheActionMenu, String>> = OnceLock::new();
        match READ.get_or_init(|| {
            // SAFETY: set before the window session starts any thread.
            unsafe {
                std::env::set_var("WIXEN_NO_AUDIO", "1");
            }
            read_the_action_menu()
        }) {
            Ok(menu) => menu,
            Err(why) => panic!("the window session could not be read: {why}"),
        }
    }

    /// The item, its letter, its key, the letter claimed once, and its
    /// description saying it has never met a real mail server.
    fn offers_report_as_junk(menu: &TheActionMenu) -> Result<(), String> {
        let (_, description) = menu
            .items
            .iter()
            .find(|(label, _)| label == "Report as &Junk\tCtrl+Shift+J")
            .ok_or(format!(
                "no \"Report as &Junk\" item with Ctrl+Shift+J on Action: {:?}",
                menu.items
            ))?;
        let claiming_j = menu
            .items
            .iter()
            .filter(|(label, _)| label.to_lowercase().contains("&j"))
            .count();
        if claiming_j != 1 {
            return Err(format!(
                "{claiming_j} items claim J on Action, so the letter runs whichever comes first"
            ));
        }
        match description.contains("never been run against a real mail server") {
            true => Ok(()),
            false => Err(format!(
                "the item's description does not say it has never met a real mail server: \
                 {description}"
            )),
        }
    }

    #[test]
    fn test_the_action_menu_offers_report_as_junk_on_j_and_ctrl_shift_j() {
        offers_report_as_junk(the_action_menu()).unwrap_or_else(|why| panic!("{why}"));
    }

    #[test]
    fn test_the_menu_reading_sees_a_second_item_on_j() {
        let planted = TheActionMenu {
            items: vec![
                (
                    "Report as &Junk\tCtrl+Shift+J".to_string(),
                    "It has never been run against a real mail server.".to_string(),
                ),
                ("&Jump".to_string(), String::new()),
            ],
        };
        let why = offers_report_as_junk(&planted).expect_err("two items on J were not seen");
        assert!(why.contains("2 items"), "{why}");
    }
}

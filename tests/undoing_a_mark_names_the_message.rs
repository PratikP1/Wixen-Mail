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
//! Since 13-08 it reads a move, a delete and a copy as well:
//!
//! - the delete arm and the move and copy path each remember the action once,
//!   after the changes are made here, so a change this computer would not
//!   keep is never remembered as done;
//! - the undo of one asks the store what each message's row says now and
//!   `application::undoing` what that means, ends a change still waiting for
//!   its server through `undo_here`, and moves back or deletes through the
//!   path the action took, never answering a waiting change with a move back.
//!
//! Since 13-09 it reads the other five modules as well, over
//! `src/presentation/managers.rs` beside the window:
//!
//! - `pim_command` remembers an action on an item only once the command has
//!   been carried out, never before the delete's question is answered;
//! - Undo and Redo in a module's list reach the undo of an item, which asks
//!   the store and `application::undoing`, and takes an owed deletion back
//!   through `take_a_deletion_back` rather than making the item again;
//! - each of the four syncs of items counts itself under way, which is what a
//!   take-back asks before it drops a note the sync may be sending;
//! - the Edit menu is handed the five lists and asks which module a step was
//!   taken in.
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
const THE_MANAGERS: &str = "src/presentation/managers.rs";

fn the_main_window() -> String {
    what_ships_in(THE_MAIN_WINDOW)
}

fn the_managers() -> String {
    what_ships_in(THE_MANAGERS)
}

fn what_ships_in(path: &str) -> String {
    let whole = fs::read_to_string(path)
        .unwrap_or_else(|why| panic!("{path}: {why}"))
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
const THE_FLAGS_WRITTEN: &str = "fn write_the_flags(";
const THE_MENU_WORDS: &str = "fn name_the_step_on_the_edit_menu(";
const THE_MENU_HANDLERS: &str = "fn keep_the_edit_menu_honest(";
const REMEMBERS: &str = "remember_the_last_action(";
const A_WRITE_TO_THE_SERVER: &str = "spawn_server_change(";
const SAYS: [&str; 2] = ["say_what_the_undo_did(", ".announce("];
const THE_DELETE_ARM: &str = "_ if id == ID_DELETE || id == ID_DELETE_OUTRIGHT =>";
const THE_MOVE_PATH: &str = "fn move_or_copy_here_first(";
const MOVED_BACK: &str = "fn move_back_or_again(";
const MADE_HERE_FIRST: &str = "complete_here_then_tell_the_server(";
const ENDED_HERE: &str = "undo_here(";
const THE_ITEM_COMMAND: &str = "pub fn pim_command(";
const CARRIED_OUT: &str = "let outcome = match";
const REMEMBERS_AN_ITEM: &str = "remember_an_item_action(";
const THE_ITEM_UNDO: &str = "pub fn undo_or_redo_on_an_item(";
const TAKEN_BACK: &str = "take_a_deletion_back(";
const THE_SYNCS: [(&str, &str); 4] = [
    ("fn spawn_contacts_sync(", "ItemKind::Contact"),
    ("fn spawn_tasks_sync(", "ItemKind::Task"),
    ("fn spawn_notes_sync(", "ItemKind::Note"),
    ("pub(crate) fn spawn_calendar_sync(", "ItemKind::Event"),
];
const THE_FIVE_LISTS: [&str; 5] = [
    "(contact_list, PimModule::Contacts)",
    "(cal_event_list, PimModule::Calendar)",
    "(reminder_list, PimModule::Reminders)",
    "(task_list, PimModule::Tasks)",
    "(note_list, PimModule::Notes)",
];

// ── The readings ───────────────────────────────────────────────────────────

/// The command remembers its action exactly once, after the last change it
/// sends to the server, so a command that stopped part way through a set
/// remembers nothing it did not finish.
fn remembers_after_its_writes(body: &str, what: &str) -> Result<(), String> {
    remembers_after(body, A_WRITE_TO_THE_SERVER, what)
}

/// The same for a command whose last write is `write`.
fn remembers_after(body: &str, write: &str, what: &str) -> Result<(), String> {
    let times = body.matches(REMEMBERS).count();
    if times != 1 {
        return Err(format!(
            "{what} remembers its action {times} times, where once after its writes is right"
        ));
    }
    let remembered = body.find(REMEMBERS).unwrap_or_default();
    let last_write = body.rfind(write).ok_or(format!(
        "{what} never reaches {write}, so this reads nothing"
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
    let flags = body_of(app, THE_FLAGS_WRITTEN)?;
    for (body, path) in [
        (one, A_WRITE_TO_THE_SERVER),
        (one, "write_the_flags("),
        (flags, "write_flags_or_put_the_row_back("),
    ] {
        if !body.contains(path) {
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
    if !words.contains("UNDOING_AT_THE_SERVER_IS_EXPERIMENTAL") {
        return Err(
            "Undo names a change at the server without saying it is experimental where \
             the person choosing it reads"
                .to_string(),
        );
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

/// A change still waiting for its server is ended here through `undo_here`,
/// and never answered with a new ask, which would send the server a move
/// into the folder it still holds the message in.
fn ends_a_waiting_change_here(body: &str) -> Result<(), String> {
    let (_, from_the_arm) = body.split_once("OneChange::EndTheWaitingRow(").ok_or(
        "the carrying out has no arm for a change still waiting, so this reads nothing".to_string(),
    )?;
    let arm = from_the_arm
        .split("OneChange::")
        .next()
        .unwrap_or(from_the_arm);
    if !arm.contains(ENDED_HERE) {
        return Err(
            "a change still waiting for its server is not ended through undo_here".to_string(),
        );
    }
    match arm.contains(MADE_HERE_FIRST) {
        true => Err(
            "a change still waiting for its server is answered with a move back, so the \
             server is asked to move a message into the folder it holds it in"
                .to_string(),
        ),
        false => Ok(()),
    }
}

/// Undo and Redo over a move, a delete or a copy reach the carrying out,
/// which asks the store and the decision for each message and goes the ways
/// the action went, saying nothing inside its loop.
fn moves_back_the_way_the_action_went(app: &str) -> Result<(), String> {
    let carrying = body_of(app, THE_CARRYING_OUT)?;
    if !carrying.contains("move_back_or_again(") {
        return Err("Undo in a list never reaches the undo of a move".to_string());
    }
    let moved = body_of(app, MOVED_BACK)?;
    for asked in [
        "what_the_store_says(",
        "what_undo_does_to(",
        "what_redo_does_to(",
        ENDED_HERE,
        MADE_HERE_FIRST,
    ] {
        if !moved.contains(asked) {
            return Err(format!("the undo of a move never calls {asked}"));
        }
    }
    ends_a_waiting_change_here(moved)?;
    match loop_bodies(moved)
        .iter()
        .any(|looped| SAYS.iter().any(|say| looped.contains(say)))
    {
        true => Err("the undo of a move says something once per message".to_string()),
        false => Ok(()),
    }
}

/// An action on an item is remembered only once its command has been carried
/// out: never before the delete's question is answered, where a No would
/// leave an undo for a delete that never happened.
fn remembers_an_item_once_it_is_done(body: &str) -> Result<(), String> {
    let carried_out = body.find(CARRIED_OUT).ok_or(format!(
        "pim_command no longer carries its command out at {CARRIED_OUT:?}, so this reads nothing"
    ))?;
    let sites: Vec<usize> = body
        .match_indices(REMEMBERS_AN_ITEM)
        .map(|(at, _)| at)
        .collect();
    if sites.is_empty() {
        return Err(
            "pim_command remembers nothing, so Undo in a module's list has nothing to take back"
                .to_string(),
        );
    }
    match sites.iter().any(|at| *at < carried_out) {
        true => Err(
            "pim_command remembers an action before it is carried out, so a delete answered \
             No, or a write that failed, leaves an undo for something that never happened"
                .to_string(),
        ),
        false => Ok(()),
    }
}

/// An owed deletion is taken back through `take_a_deletion_back`, which
/// changes the row and the note together, and never answered by making the
/// item again, which would bring it back new while the deletion is still sent.
fn takes_an_owed_deletion_back(body: &str) -> Result<(), String> {
    let (_, from_the_arm) = body.split_once("UndoAnItem::TakeTheDeletionBack(").ok_or(
        "the undo of an item has no arm for an owed deletion, so this reads nothing".to_string(),
    )?;
    let arm = from_the_arm
        .split("UndoAnItem::")
        .next()
        .unwrap_or(from_the_arm);
    match arm.contains(TAKEN_BACK) {
        true => Ok(()),
        false => Err(
            "an owed deletion is not taken back through take_a_deletion_back, so the item \
             comes back while its deletion is still sent"
                .to_string(),
        ),
    }
}

/// Undo and Redo in a module's list reach the undo of an item, which asks the
/// store and the decision and goes the ways they answer.
fn undoes_an_item_the_way_the_store_says(app: &str, managers: &str) -> Result<(), String> {
    let edit = body_of(app, THE_EDIT_COMMAND)?;
    let (_, arm) = edit.split_once("Doing::TheLastAction =>").ok_or(
        "the Edit command has no arm for the last action, so this reads nothing".to_string(),
    )?;
    if !arm.contains("undo_or_redo_on_an_item(") {
        return Err("Undo in a module's list never reaches the undo of an item".to_string());
    }
    let carrying = body_of(managers, THE_ITEM_UNDO)?;
    for asked in [
        "what_the_store_says_of_an_item(",
        "what_undo_does_to_an_item(",
        "what_redo_does_to_an_item(",
        TAKEN_BACK,
        "make_it_again(",
    ] {
        if !carrying.contains(asked) {
            return Err(format!("the undo of an item never calls {asked}"));
        }
    }
    takes_an_owed_deletion_back(carrying)
}

/// Each sync of items counts itself under way for its account and kind.
fn each_sync_counts_itself(app: &str) -> Result<(), String> {
    for (sync, kind) in THE_SYNCS {
        let body = body_of(app, sync)?;
        if !(body.contains("ASyncUnderWay::begins(") && body.contains(kind)) {
            return Err(format!(
                "{sync} never counts itself under way for {kind}, so an undo can take back a \
                 deletion that sync is sending"
            ));
        }
    }
    Ok(())
}

/// The Edit menu is handed the five module lists, and its open handler asks
/// which module the step was taken in.
fn the_menu_names_an_items_step(app: &str) -> Result<(), String> {
    let at = app
        .find("keep_the_edit_menu_honest(\n                &frame,")
        .ok_or(
            "the main window never hands the Edit menu its lists, so this reads nothing"
                .to_string(),
        )?;
    let call = &app[at..];
    let call = &call[..call.find(");\n").unwrap_or(call.len())];
    if let Some(missing) = THE_FIVE_LISTS.iter().find(|list| !call.contains(*list)) {
        return Err(format!(
            "the Edit menu is not handed {missing}, so it cannot name that list's step"
        ));
    }
    let handlers = body_of(app, THE_MENU_HANDLERS)?;
    match handlers.contains(".module()") {
        true => Ok(()),
        false => Err(
            "the Edit menu never asks which module the step was taken in, so a list names a \
             step it cannot take back"
                .to_string(),
        ),
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
fn test_a_delete_remembers_the_action_after_it_is_made_here() {
    let app = the_main_window();
    let arm = the_id_arm(&app, THE_DELETE_ARM).unwrap_or_else(|why| panic!("{why}"));
    remembers_after(arm, MADE_HERE_FIRST, "Delete").unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_move_or_a_copy_remembers_the_action_after_it_is_made_here() {
    let app = the_main_window();
    let body = body_of(&app, THE_MOVE_PATH).unwrap_or_else(|why| panic!("{why}"));
    remembers_after(body, MADE_HERE_FIRST, "Move to and Copy to")
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_undo_of_a_move_asks_the_store_and_goes_the_way_the_action_went() {
    moves_back_the_way_the_action_went(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_companion_an_undo_that_moves_back_a_waiting_change_is_refused() {
    let planted = "fn move_back_or_again(app: AppHandles<'_>) {
    for message in went {
        match change {
            OneChange::EndTheWaitingRow(waiting) => asks.push(a_move_back(&waiting)),
            OneChange::Move { from, to } => asks.push(a_move(from, to)),
        }
    }
    complete_here_then_tell_the_server(app, list, cache, asks, None, refuse);
}
";
    assert!(ends_a_waiting_change_here(planted).is_err());
}

#[test]
fn test_companion_a_delete_that_remembers_before_it_is_made_is_refused() {
    let planted = "
        remember_the_last_action(&state, action);
        complete_here_then_tell_the_server(app, &msg_list, &cache, asks, None, server_first);
";
    assert!(remembers_after(planted, MADE_HERE_FIRST, "the planted delete").is_err());
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
fn test_an_action_on_an_item_is_remembered_after_it_is_carried_out() {
    let managers = the_managers();
    let body = body_of(&managers, THE_ITEM_COMMAND).unwrap_or_else(|why| panic!("{why}"));
    remembers_an_item_once_it_is_done(body).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_undo_in_a_modules_list_asks_the_store_and_takes_an_owed_deletion_back() {
    undoes_an_item_the_way_the_store_says(&the_main_window(), &the_managers())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_each_sync_of_items_counts_itself_under_way() {
    each_sync_counts_itself(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_edit_menu_names_the_last_action_on_an_item_when_its_list_has_focus() {
    the_menu_names_an_items_step(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_companion_an_item_remembered_before_it_is_carried_out_is_refused() {
    let planted = "pub fn pim_command(action: PimAction) {
    remember_an_item_action(state, done);
    let asked = MessageDialog::builder(frame, &confirm_delete(kind, &name), \"Delete\");
    let outcome = match one_day {
        None => cache.delete_task(&id),
    };
}
";
    assert!(remembers_an_item_once_it_is_done(planted).is_err());
}

#[test]
fn test_companion_an_undo_that_makes_every_deleted_item_again_is_refused() {
    let planted = "pub fn undo_or_redo_on_an_item(direction: Direction) {
    match answer {
        UndoAnItem::TakeTheDeletionBack(record) => cache.make_it_again(&record, &as_id),
        UndoAnItem::MakeItAgain(record) => cache.make_it_again(&record, &as_id),
    }
}
";
    assert!(takes_an_owed_deletion_back(planted).is_err());
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

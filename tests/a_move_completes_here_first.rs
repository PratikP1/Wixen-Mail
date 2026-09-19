//! A move, a delete or a copy within one account completes on this computer
//! first, and the server is brought into line afterwards (#86).
//!
//! The tester on 2026-09-18, under NVDA against Gmail: a move takes a
//! noticeable time before the row leaves and the sentence comes, because
//! the move was done at the server first and the list changed only when the
//! server had agreed; and Enter on the chosen folder in the Move dialog did
//! nothing. The decisions are in `application::moves_waiting`, held against
//! the loopback servers there; the table is `moves_waiting` in the cache.
//! What this file holds is the wiring in the window, which needs a frame and
//! a runtime to run and so is read as text.
//!
//! The source readings, over `what_ships` of `src/presentation/wx_app.rs`:
//! the move arm and the delete arm reaching the one function that makes the
//! change here, keeps it waiting and pushes once; that function taking the
//! row out and showing the line before any session is asked for, and
//! speaking a copy's outcome since its row stays; the arm that puts a refused
//! change back undoing it and speaking at High; every path that lists a
//! folder replaying the account's waiting moves before its first listing;
//! and the network coming back starting nothing that reaches a server, the
//! shape `nothing_sends_a_flag_change_unasked` has. Each with a companion
//! that plants the fault into a snippet shaped as the window should be, so a
//! reading that stopped finding its anchor cannot pass by finding nothing.
//!
//! # Why this lives here rather than beside the code
//!
//! `src/presentation/wx_app.rs` is named by 88 guard records on 2026-09-19,
//! so a test added there is 88 builds and 88 library runs at the next
//! commit. This file is named by its own records, whose `suite` couples it
//! to `wx_app.rs`, so it runs on the commits that could break it.
//!
//! # What this cannot see
//!
//! Whether the row is heard to leave at once under NVDA, whether a refusal
//! with the network off is heard putting the row back, and what a real
//! server does with a replayed move after a restart: the tester's, and #63's
//! move and delete proofs are re-taken after this. The window is not
//! started.

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

/// The body of one `UIUpdate::X => {` arm of the update handler, up to the
/// next arm of the same shape.
fn the_update_arm<'a>(source: &'a str, heading: &str) -> Result<&'a str, String> {
    let start = source.find(heading).ok_or(format!(
        "{heading:?} is no longer here, so this reads nothing"
    ))? + heading.len();
    let rest = &source[start..];
    let end = rest.find("\n        UIUpdate::").unwrap_or(rest.len());
    Ok(&rest[..end])
}

/// Whether `first` is in the text before `second`, or a complaint naming
/// which is missing.
fn comes_before(text: &str, first: &str, second: &str) -> Result<bool, String> {
    let a = text.find(first).ok_or(format!(
        "{first:?} is no longer here, so this reads nothing"
    ))?;
    let b = text.find(second).ok_or(format!(
        "{second:?} is no longer here, so this reads nothing"
    ))?;
    Ok(a < b)
}

// ── The anchors, each a name or a literal ──────────────────────────────────

const THE_DELETE_ARM: &str = "_ if id == ID_DELETE || id == ID_DELETE_OUTRIGHT =>";
const THE_MOVE: &str = "fn move_or_copy_message(";
/// The one function a move, a delete and a copy share for "made here, kept
/// waiting, pushed once".
const MADE_HERE: &str = "fn complete_here_then_tell_the_server(";
/// The move arm's half of the way there, and the delete arm's decision.
const THE_MOVE_MADE_HERE: &str = "fn move_or_copy_here_first(";
const THE_DELETE_DECIDED_HERE: &str = "fn where_a_delete_goes_here(";
/// The arm and not the send, which sits hundreds of lines above it: the
/// bare variant name found the line that sends the update, the finding
/// `nothing_sends_a_flag_change_unasked` recorded on 2026-09-05.
const THE_PUT_BACK_ARM: &str = "UIUpdate::MovePutBack { waiting, reason } => {";
const THE_CHECK: &str = "fn spawn_mail_sync(";
const THE_DOWNLOAD: &str = "fn start_the_download(";
const THE_REPLAY: &str = "replay_the_moves_that_were_waiting(";
const THE_NETWORK_BACK_ARM: &str = "UIUpdate::TheNetworkIsBack => {";
/// Everything in the window that ends with something reaching a server,
/// with the replay beside what `nothing_sends_a_flag_change_unasked` names.
const EVERYTHING_THAT_REACHES_A_SERVER: [&str; 3] =
    [THE_REPLAY, "spawn_mail_sync(", "flush_outbox("];

const CHANGES_IT_HERE: &str = "what_happens_here(";
const TAKES_THE_ROW_OUT: &str = "take_row_out_of_the_list(";
const SHOWS_THE_LINE: &str = "send_shown(";
const SPEAKS_THE_LINE: &str = "send_status(";
const ASKS_FOR_A_SESSION: &str = "the_session_at(";
const DECIDES_WHERE_A_DELETE_GOES: &str = "where_a_deleted_message_goes(";
const UNDOES_IT_HERE: &str = "undo_here(";
const SAYS_WHY_AT_HIGH: &str = "Priority::High";
const WORDS_THE_REFUSAL: &str = "put_back_because_the_server_refused(";
const LISTS_THE_FOLDERS: &str = "fetch_folders()";
/// The first arm after the rows that left, which ends the line's stretch.
const THE_FIRST_REFUSAL_ARM: &str = "Err(NotMadeHere::";

// ── The readings ───────────────────────────────────────────────────────────

/// A move and a copy within the account go through the one function that
/// makes the change here, keeps it waiting and pushes once.
fn the_move_arm_completes_here_first(app: &str) -> Result<(), String> {
    let asks = body_of(app, THE_MOVE)?;
    if !asks.contains(THE_MOVE_MADE_HERE.trim_start_matches("fn ")) {
        return Err(format!(
            "move_or_copy_message does not reach {THE_MOVE_MADE_HERE}, so a move waits for \
             the server before the row leaves"
        ));
    }
    let made = body_of(app, THE_MOVE_MADE_HERE)?;
    if !made.contains(MADE_HERE.trim_start_matches("fn ")) {
        return Err(format!(
            "move_or_copy_here_first does not reach {MADE_HERE}, so a move waits for the \
             server before the row leaves"
        ));
    }
    Ok(())
}

/// Delete and Delete Permanently decide where the message goes, before
/// anything changes, and go through the same function.
fn the_delete_arm_completes_here_first(app: &str) -> Result<(), String> {
    let arm = the_id_arm(app, THE_DELETE_ARM)?;
    for needed in [
        THE_DELETE_DECIDED_HERE.trim_start_matches("fn "),
        MADE_HERE.trim_start_matches("fn "),
        "say_the_one_word(&a11y, \"Delete\")",
    ] {
        if !arm.contains(needed) {
            return Err(format!(
                "the Delete arm does not reach {needed}, so a delete waits for the server \
                 before the row leaves, or asks it without deciding where the message goes"
            ));
        }
    }
    let decided = body_of(app, THE_DELETE_DECIDED_HERE)?;
    if !decided.contains(DECIDES_WHERE_A_DELETE_GOES) {
        return Err(format!(
            "where_a_delete_goes_here does not reach {DECIDES_WHERE_A_DELETE_GOES}, so the \
             delete decides for itself where the message goes"
        ));
    }
    Ok(())
}

/// The shared function changes the row here, takes it out of the list and
/// shows the line before any session is asked for; a copy's line is spoken,
/// since its row stays and nothing else says it happened.
fn made_here_before_the_server_is_asked(app: &str) -> Result<(), String> {
    let made = body_of(app, MADE_HERE)?;
    for (first, second) in [
        (CHANGES_IT_HERE, ASKS_FOR_A_SESSION),
        (TAKES_THE_ROW_OUT, ASKS_FOR_A_SESSION),
        (SHOWS_THE_LINE, ASKS_FOR_A_SESSION),
    ] {
        if !comes_before(&made, first, second)? {
            return Err(format!(
                "complete_here_then_tell_the_server asks for a session before it reaches \
                 {first}, so the row waits for the server after all"
            ));
        }
    }
    if !made.contains("is_a_copy()") || !made.contains(SPEAKS_THE_LINE) {
        return Err(
            "complete_here_then_tell_the_server does not speak a copy's line, so a copy \
             whose row stays is a dead key"
                .to_string(),
        );
    }
    // The line for a row that left: from the row leaving to the first
    // refusal arm, shown and never spoken, since the row the cursor lands
    // on is what is heard (#83).
    let a_row_that_left = between(&made, TAKES_THE_ROW_OUT, THE_FIRST_REFUSAL_ARM)?;
    if !a_row_that_left.contains(SHOWS_THE_LINE) || a_row_that_left.contains(SPEAKS_THE_LINE) {
        return Err(
            "complete_here_then_tell_the_server speaks the line for a row that left, or \
             does not show it, so a move is heard twice or the eye has nothing"
                .to_string(),
        );
    }
    Ok(())
}

/// The text after the first `from` up to the next `to`, or a complaint
/// naming which anchor is gone.
fn between<'a>(text: &'a str, from: &str, to: &str) -> Result<&'a str, String> {
    let start = text
        .find(from)
        .ok_or(format!("{from:?} is no longer here, so this reads nothing"))?
        + from.len();
    let rest = &text[start..];
    let end = rest
        .find(to)
        .ok_or(format!("{to:?} is no longer here, so this reads nothing"))?;
    Ok(&rest[..end])
}

/// A refusal arrives on its own update, whose arm undoes the change here
/// and speaks the reason at High.
fn a_refusal_is_undone_and_said(app: &str) -> Result<(), String> {
    let arm = the_update_arm(app, THE_PUT_BACK_ARM)?;
    for needed in [UNDOES_IT_HERE, SAYS_WHY_AT_HIGH, WORDS_THE_REFUSAL] {
        if !arm.contains(needed) {
            return Err(format!(
                "the MovePutBack arm does not reach {needed}, so a move the server refused \
                 stays made here, or comes back with nothing said"
            ));
        }
    }
    Ok(())
}

/// Every path that lists an account's folders replays the account's waiting
/// moves first: the check, which the watch and a folder opened go through,
/// and the download.
fn every_folder_read_replays_first(app: &str) -> Result<(), String> {
    for path in [THE_CHECK, THE_DOWNLOAD] {
        let body = body_of(app, path)?;
        if !comes_before(&body, THE_REPLAY, LISTS_THE_FOLDERS)? {
            return Err(format!(
                "{path} lists the folders before it replays the waiting moves, so a folder \
                 the server still has a moved message in brings it back"
            ));
        }
    }
    Ok(())
}

/// Every place the replay is called from, by line.
fn where_the_replay_is_called_from(source: &str) -> Vec<usize> {
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains(THE_REPLAY) && !line.trim_start().starts_with("//"))
        .map(|(at, _)| at + 1)
        .collect()
}

/// The network coming back starts nothing that ends at a server.
fn the_network_coming_back_replays_nothing(app: &str) -> Result<(), String> {
    let at = app
        .find(THE_NETWORK_BACK_ARM)
        .ok_or("the arm for the network coming back is gone".to_string())?;
    let body = &app[at..(at + 900).min(app.len())];
    let reached: Vec<&str> = EVERYTHING_THAT_REACHES_A_SERVER
        .iter()
        .copied()
        .filter(|call| body.contains(call))
        .collect();
    if !reached.is_empty() {
        return Err(format!(
            "the network coming back starts {reached:?}, and each of those ends at a \
             server. Nobody asked."
        ));
    }
    Ok(())
}

// ── The readings over the window ───────────────────────────────────────────

#[test]
fn test_a_move_within_the_account_completes_here_first() {
    the_move_arm_completes_here_first(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_delete_completes_here_first_after_deciding_where_it_goes() {
    the_delete_arm_completes_here_first(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_row_leaves_and_the_line_is_shown_before_any_session_is_asked_for() {
    made_here_before_the_server_is_asked(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_refusal_puts_the_change_back_here_and_is_said_at_high() {
    a_refusal_is_undone_and_said(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_every_path_that_lists_a_folder_replays_the_waiting_moves_first() {
    every_folder_read_replays_first(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_places_that_replay_a_waiting_move_are_the_ones_counted() {
    // Four, and each one is a thing somebody did.
    //
    //   1. The definition itself.
    //   2. A mail check, which had already signed in to that server: somebody
    //      pressed Check Mail, opened a folder, or the watch on the inbox
    //      woke.
    //   3. The download, which somebody started.
    //   4. The push after the key, on the session the person's own move or
    //      delete opened.
    //
    // A fifth call site is not a failure by itself. It is a question: what
    // asked for it. If the answer is "the network came back", or "a timer",
    // that is guardrail 7.
    let called_from = where_the_replay_is_called_from(&the_main_window());
    assert_eq!(
        called_from.len(),
        4,
        "the number of places that replay a waiting move has moved. Lines: \
         {called_from:?}. Read this test's comment before changing the number"
    );
}

#[test]
fn test_the_network_coming_back_replays_no_waiting_move() {
    the_network_coming_back_replays_nothing(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

// ── The companions: the window as it should be, with a fault planted ───────

/// A snippet shaped as the window should be, holding every anchor the
/// readings look for and nothing else.
fn a_window_as_it_should_be() -> String {
    let mut snippet = String::new();
    snippet.push_str(THE_MOVE);
    snippet.push_str(") {\n    move_or_copy_here_first(app, list, &cache, a11y, asked);\n}\n");
    snippet.push_str(THE_MOVE_MADE_HERE);
    snippet.push_str(") {\n    complete_here_then_tell_the_server(app, asks);\n}\n");
    snippet.push_str(THE_DELETE_ARM);
    snippet.push_str(
        " {\n    where_a_delete_goes_here(&state, &cache, row, asked);\n    \
         say_the_one_word(&a11y, \"Delete\");\n    \
         complete_here_then_tell_the_server(app, asks);\n}\n_ if id == ID_OTHER => {}\n",
    );
    snippet.push_str(THE_DELETE_DECIDED_HERE);
    snippet.push_str(") {\n    where_a_deleted_message_goes(folders, from, asked);\n}\n");
    snippet.push_str(MADE_HERE);
    snippet.push_str(
        ") {\n    match what_happens_here(cache, &asked, &subject) {\n        Ok(made) => {\n            \
         if asked.what.is_a_copy() {\n                send_status(tx, rt, &shown);\n            \
         } else {\n                take_row_out_of_the_list(state, list, row);\n                \
         send_shown(tx, rt, &shown);\n            }\n        }\n        \
         Err(NotMadeHere::RefusedInWords(words)) => send_refusal(tx, rt, &words),\n    }\n    \
         the_session_at(&account);\n}\n",
    );
    snippet.push_str("        ");
    snippet.push_str(THE_PUT_BACK_ARM);
    snippet.push_str(
        "\n            undo_here(cache, waiting);\n            \
         a11y.announce(&put_back_because_the_server_refused(what, subject, reason), \
         Priority::High);\n        }\n        UIUpdate::Other => {}\n",
    );
    for path in [THE_CHECK, THE_DOWNLOAD] {
        snippet.push_str(path);
        snippet.push_str(
            ") {\n    replay_the_moves_that_were_waiting(&cache, &controller, id, &handle, &say);\n    \
             controller.fetch_folders();\n}\n",
        );
    }
    snippet.push_str(THE_NETWORK_BACK_ARM);
    snippet.push_str("\n    offer_to_check();\n}\n");
    snippet
}

fn every_reading_over(app: &str) -> Result<(), String> {
    the_move_arm_completes_here_first(app)?;
    the_delete_arm_completes_here_first(app)?;
    made_here_before_the_server_is_asked(app)?;
    a_refusal_is_undone_and_said(app)?;
    every_folder_read_replays_first(app)?;
    the_network_coming_back_replays_nothing(app)
}

#[test]
fn test_the_readings_pass_a_window_shaped_as_it_should_be() {
    every_reading_over(&a_window_as_it_should_be()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_readings_complain_when_an_arm_waits_for_the_server_again() {
    let app = a_window_as_it_should_be();

    let move_waits = app.replacen(
        "fn move_or_copy_here_first() {\n    complete_here_then_tell_the_server(app, asks);",
        "fn move_or_copy_here_first() {\n    spawn_folder_move(app, moving);",
        1,
    );
    let why = the_move_arm_completes_here_first(&move_waits).expect_err("the move waits");
    assert!(why.contains("does not reach fn complete_here"), "{why}");

    let move_never_here = app.replacen(
        "fn move_or_copy_message() {\n    move_or_copy_here_first(app, list, &cache, a11y, asked);",
        "fn move_or_copy_message() {\n    spawn_folder_move(app, moving);",
        1,
    );
    let why = the_move_arm_completes_here_first(&move_never_here).expect_err("never here");
    assert!(
        why.contains("does not reach fn move_or_copy_here_first"),
        "{why}"
    );

    let delete_waits = app.replacen(
        "    complete_here_then_tell_the_server(app, asks);\n}\n_ if id == ID_OTHER",
        "    spawn_server_change(app, row);\n}\n_ if id == ID_OTHER",
        1,
    );
    let why = the_delete_arm_completes_here_first(&delete_waits).expect_err("the delete waits");
    assert!(why.contains("the Delete arm does not reach"), "{why}");

    let session_first = app
        .replacen("    the_session_at(&account);\n}\n", "}\n", 1)
        .replacen(
            "    match what_happens_here(cache, &asked, &subject) {",
            "    the_session_at(&account);\n    match what_happens_here(cache, &asked, &subject) {",
            1,
        );
    let why = made_here_before_the_server_is_asked(&session_first).expect_err("the session first");
    assert!(why.contains("asks for a session before"), "{why}");

    let copy_silent = app.replacen(
        "            if asked.what.is_a_copy() {\n                send_status(tx, rt, &shown);\n            } else {\n",
        "            {\n",
        1,
    );
    let why = made_here_before_the_server_is_asked(&copy_silent).expect_err("the copy silent");
    assert!(why.contains("does not speak a copy's line"), "{why}");

    let moved_row_spoken = app.replacen(
        "                take_row_out_of_the_list(state, list, row);\n                send_shown(tx, rt, &shown);",
        "                take_row_out_of_the_list(state, list, row);\n                send_status(tx, rt, &shown);\n                send_shown(tx, rt, &shown);",
        1,
    );
    let why = made_here_before_the_server_is_asked(&moved_row_spoken).expect_err("spoken again");
    assert!(why.contains("speaks the line for a row that left"), "{why}");
}

#[test]
fn test_the_readings_complain_when_a_refusal_is_not_undone_or_not_said() {
    let app = a_window_as_it_should_be();

    let not_undone = app.replacen("            undo_here(cache, waiting);\n", "", 1);
    let why = a_refusal_is_undone_and_said(&not_undone).expect_err("not undone");
    assert!(why.contains("does not reach undo_here("), "{why}");

    let said_quietly = app.replacen("Priority::High", "Priority::Normal", 1);
    let why = a_refusal_is_undone_and_said(&said_quietly).expect_err("said at Normal");
    assert!(why.contains("does not reach Priority::High"), "{why}");
}

#[test]
fn test_the_readings_complain_when_a_folder_is_listed_before_the_replay() {
    let app = a_window_as_it_should_be();
    let listed_first = app.replacen(
        "fn spawn_mail_sync() {\n    replay_the_moves_that_were_waiting(&cache, &controller, id, &handle, &say);\n    controller.fetch_folders();",
        "fn spawn_mail_sync() {\n    controller.fetch_folders();\n    replay_the_moves_that_were_waiting(&cache, &controller, id, &handle, &say);",
        1,
    );
    let why = every_folder_read_replays_first(&listed_first).expect_err("listed first");
    assert!(why.contains("lists the folders before it replays"), "{why}");

    let wired_to_the_network = app.replacen(
        "UIUpdate::TheNetworkIsBack => {\n    offer_to_check();",
        "UIUpdate::TheNetworkIsBack => {\n    spawn_mail_sync(app, accounts, None, wanted);",
        1,
    );
    let why = the_network_coming_back_replays_nothing(&wired_to_the_network)
        .expect_err("wired to the network");
    assert!(why.contains("Nobody asked"), "{why}");
}

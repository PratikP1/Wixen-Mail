//! A move or a copy to a folder on another account completes on this
//! computer first, and the two servers are brought into line afterwards
//! (#86, the second half, on Pratik's decision of 2026-09-19 overruling
//! 11-07.1's decision 29).
//!
//! 11-07.1 made a move within one account complete here first and left a
//! crossing waiting for both servers, since replaying one from a row needs
//! the message's bytes held. The bytes were already held, by the store
//! phase 4.1 wrote for a crossing interrupted by a restart, so 11-07.2
//! queues the crossing in `moves_waiting` with the bytes in
//! `moves_in_flight`, runs it in the background and at a check of either
//! account, and retires the question a restart used to ask. The decisions
//! are in `application::moves_waiting` and `application::mail_across_accounts`,
//! held against loopback servers there; what this file holds is the wiring
//! in the window, which needs a frame and a runtime to run and so is read
//! as text, and one comment in the sync that says why its forgetting
//! subtracts nothing for a crossing.
//!
//! The readings, over `what_ships` of `src/presentation/wx_app.rs`: the
//! move arm building a crossing's kind and reaching the one function every
//! move shares, with no session asked for before the change is made; the
//! ceiling path alone calling the whole crossing in front of the person,
//! with its line; the question at start gone, with no dialog built from
//! the store; the replay helper running the crossings after the moves
//! within the account, on the paths 11-07.1's target reads; the network
//! coming back reaching neither; and the put-back arm reading the
//! destination folder under the account it is in. Each with a companion
//! that plants the fault into a snippet shaped as the window should be.
//!
//! # Why this lives here rather than beside the code
//!
//! `src/presentation/wx_app.rs` is named by 91 guard records on 2026-09-19
//! and holds 199 tests, and this phase adds none there. This file is named
//! by its own records, whose `suite` couples it to `wx_app.rs`.
//!
//! # What this cannot see
//!
//! Whether a row is heard to leave at once for a folder of the other
//! account, whether the message is there after the next check, what a real
//! destination does with a message it already holds (ledger 187) and what
//! Gmail makes of an appended message: the tester's, and #63's crossing
//! proofs are re-taken after this. The window is not started.

use std::fs;

use wixen_mail::common::what_ships::what_ships;

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";
const THE_SYNC: &str = "src/application/mail_sync.rs";

fn the_shipped_half_of(path: &str) -> String {
    let whole = fs::read_to_string(path)
        .unwrap_or_else(|why| panic!("{path}: {why}"))
        .replace("\r\n", "\n");
    what_ships(&whole)
}

fn the_main_window() -> String {
    the_shipped_half_of(THE_MAIN_WINDOW)
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

/// How many times a call appears outside comments.
fn times_called(source: &str, call: &str) -> usize {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(|line| line.matches(call).count())
        .sum()
}

// ── The anchors, each a name or a literal ──────────────────────────────────

const THE_MOVE: &str = "fn move_or_copy_message(";
const THE_MOVE_MADE_HERE: &str = "fn move_or_copy_here_first(";
const MADE_HERE: &str = "fn complete_here_then_tell_the_server(";
const THE_WORKER: &str = "fn spawn_folder_move(";
const THE_REPLAY_HELPER: &str = "fn replay_the_moves_that_were_waiting(";
const THE_PUT_BACK_ARM: &str = "UIUpdate::MovePutBack { waiting, reason } => {";
const THE_NETWORK_BACK_ARM: &str = "UIUpdate::TheNetworkIsBack => {";

const A_MOVE_ACROSS: &str = "MoveAcross";
const A_COPY_ACROSS: &str = "CopyAcross";
const THE_SERVER_FIRST_WORKER: &str = "spawn_folder_move(";
const ASKS_FOR_A_SESSION: &str = "the_session_at(";
const THE_WHOLE_CROSSING_IN_FRONT: &str = "move_it_across(";
const THE_CEILING: &str = "LARGEST_MESSAGE_KEPT_WHILE_IT_MOVES_BYTES";
const THE_CEILING_LINE: &str = "larger than 25 MB";
const THE_QUESTION_AT_START: &str = "fn say_what_did_not_finish";
const THE_OFFER_TO_FINISH: &str = "fn spawn_finishing_the_move";
const THE_STORE_READ_FOR_A_DIALOG: &str = "moves_that_did_not_finish(";
const REPLAYS_THE_MOVES: &str = "replay_the_moves_waiting_for(";
const REPLAYS_THE_CROSSINGS: &str = "replay_the_crossings_waiting_for(";
const THE_DESTINATION_UNDER_ITS_ACCOUNT: &str = "the_account_it_is_going_to()";

// ── The readings ───────────────────────────────────────────────────────────

/// The move arm builds a crossing's kind beside a move's and hands both to
/// the one function that makes the change here, and nothing on the way
/// asks for a session or hands a crossing to the server-first worker.
fn the_arms_complete_a_crossing_here_first(app: &str) -> Result<(), String> {
    let asks = body_of(app, THE_MOVE)?;
    if asks.contains(THE_SERVER_FIRST_WORKER) {
        return Err(format!(
            "move_or_copy_message still reaches {THE_SERVER_FIRST_WORKER} itself, so a set \
             that crosses accounts waits for both servers before the row leaves"
        ));
    }
    let made = body_of(app, THE_MOVE_MADE_HERE)?;
    for kind in [A_MOVE_ACROSS, A_COPY_ACROSS] {
        if !made.contains(kind) {
            return Err(format!(
                "move_or_copy_here_first knows no {kind}, so a crossing is not made here first"
            ));
        }
    }
    if !made.contains(MADE_HERE.trim_start_matches("fn ")) {
        return Err(format!(
            "move_or_copy_here_first does not reach {MADE_HERE}, so the crossing takes a \
             path of its own that can drift from a move's"
        ));
    }
    if made.contains(ASKS_FOR_A_SESSION) {
        return Err(format!(
            "move_or_copy_here_first reaches {ASKS_FOR_A_SESSION}, so a session is asked for \
             before the row leaves"
        ));
    }
    Ok(())
}

/// The whole crossing in front of the person is reached by the worker
/// alone, and the move arm sends only a message the store cannot hold
/// there, with the line saying why.
fn only_the_ceiling_goes_server_first(app: &str) -> Result<(), String> {
    let worker = body_of(app, THE_WORKER)?;
    let in_the_worker = times_called(&worker, THE_WHOLE_CROSSING_IN_FRONT);
    let everywhere = times_called(app, THE_WHOLE_CROSSING_IN_FRONT);
    if in_the_worker == 0 {
        return Err(format!(
            "spawn_folder_move no longer reaches {THE_WHOLE_CROSSING_IN_FRONT}, so a message \
             over the ceiling has no path at all"
        ));
    }
    if everywhere != in_the_worker {
        return Err(format!(
            "{THE_WHOLE_CROSSING_IN_FRONT} is reached {everywhere} times in the window and \
             {in_the_worker} of them are in the worker, so a crossing runs in front of the \
             person somewhere else"
        ));
    }
    let made = body_of(app, THE_MOVE_MADE_HERE)?;
    for needed in [THE_CEILING, THE_CEILING_LINE, THE_SERVER_FIRST_WORKER] {
        if !made.contains(needed) {
            return Err(format!(
                "move_or_copy_here_first does not reach {needed}, so a message over the \
                 ceiling is queued with nothing to resume it from, or goes with nothing said"
            ));
        }
    }
    Ok(())
}

/// No question at start, and no dialog built from the store of held bytes.
fn the_question_at_start_is_retired(app: &str) -> Result<(), String> {
    for gone in [
        THE_QUESTION_AT_START,
        THE_OFFER_TO_FINISH,
        THE_STORE_READ_FOR_A_DIALOG,
    ] {
        if app.contains(gone) {
            return Err(format!(
                "{gone} is still in the window, so a restart asks about a crossing the \
                 person already asked for"
            ));
        }
    }
    Ok(())
}

/// The replay helper runs the crossings after the moves within the
/// account, so every path 11-07.1's target reads replays both before it
/// lists a folder.
fn the_helper_replays_crossings_after_moves(app: &str) -> Result<(), String> {
    let helper = body_of(app, THE_REPLAY_HELPER)?;
    if !comes_before(&helper, REPLAYS_THE_MOVES, REPLAYS_THE_CROSSINGS)? {
        return Err(format!(
            "replay_the_moves_that_were_waiting runs {REPLAYS_THE_CROSSINGS} before \
             {REPLAYS_THE_MOVES}, so a crossing of a row a move within the account is \
             still asking about goes first"
        ));
    }
    Ok(())
}

/// The network coming back reaches neither replay.
fn the_network_coming_back_replays_no_crossing(app: &str) -> Result<(), String> {
    let at = app
        .find(THE_NETWORK_BACK_ARM)
        .ok_or("the arm for the network coming back is gone".to_string())?;
    let body = &app[at..(at + 900).min(app.len())];
    for call in [
        REPLAYS_THE_CROSSINGS,
        THE_REPLAY_HELPER.trim_start_matches("fn "),
    ] {
        if body.contains(call) {
            return Err(format!(
                "the network coming back starts {call}, which ends at two servers. Nobody \
                 asked."
            ));
        }
    }
    Ok(())
}

/// The put-back arm reads the destination folder under the account it is
/// in, which for a crossing is the other account.
fn the_put_back_arm_reads_the_destination_under_its_account(app: &str) -> Result<(), String> {
    let arm = the_update_arm(app, THE_PUT_BACK_ARM)?;
    if !arm.contains(THE_DESTINATION_UNDER_ITS_ACCOUNT) {
        return Err(format!(
            "the MovePutBack arm does not reach {THE_DESTINATION_UNDER_ITS_ACCOUNT}, so a \
             crossing undone leaves the other account's folder unread on screen"
        ));
    }
    Ok(())
}

/// The sync's forgetting says why it subtracts nothing for a crossing, by
/// naming the replay that runs first. A comment, read from the whole file
/// rather than the shipped half, and held because the next reader of that
/// forgetting will look for the subtraction and needs to be told why there
/// is none.
fn the_sync_names_the_crossing_replay(sync: &str) -> Result<(), String> {
    let named = REPLAYS_THE_CROSSINGS.trim_end_matches('(');
    if !sync.contains(named) {
        return Err(format!(
            "mail_sync.rs no longer names {named}, so the next reader of its forgetting has \
             nothing saying why a crossing's row is left alone"
        ));
    }
    Ok(())
}

// ── The readings over the window ───────────────────────────────────────────

#[test]
fn test_a_move_or_a_copy_to_another_account_completes_here_first() {
    the_arms_complete_a_crossing_here_first(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_only_a_message_over_the_ceiling_goes_server_first_and_says_so() {
    only_the_ceiling_goes_server_first(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_restart_asks_no_question_about_a_crossing() {
    the_question_at_start_is_retired(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_every_path_that_replays_the_moves_replays_the_crossings_after_them() {
    the_helper_replays_crossings_after_moves(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_network_coming_back_replays_no_crossing() {
    the_network_coming_back_replays_no_crossing(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_crossing_put_back_rereads_the_other_accounts_folder() {
    the_put_back_arm_reads_the_destination_under_its_account(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_sync_says_why_its_forgetting_leaves_a_crossing_alone() {
    let whole = fs::read_to_string(THE_SYNC).unwrap_or_else(|why| panic!("{THE_SYNC}: {why}"));
    the_sync_names_the_crossing_replay(&whole).unwrap_or_else(|why| panic!("{why}"));
}

// ── The companions: the window as it should be, with a fault planted ───────

/// A snippet shaped as the window should be, holding every anchor the
/// readings look for and nothing else.
fn a_window_as_it_should_be() -> String {
    let mut snippet = String::new();
    snippet.push_str(THE_MOVE);
    snippet.push_str(") {\n    move_or_copy_here_first(app, list, &cache, a11y, asked);\n}\n");
    snippet.push_str(THE_MOVE_MADE_HERE);
    snippet.push_str(
        ") {\n    let can_be_held = size <= LARGEST_MESSAGE_KEPT_WHILE_IT_MOVES_BYTES;\n    \
         let what = WhatAWaitingMoveDoes::MoveAcross { into, to_account };\n    \
         let what = WhatAWaitingMoveDoes::CopyAcross { into, to_account };\n    \
         send_status(tx, rt, \"larger than 25 MB, so it goes now\");\n    \
         spawn_folder_move(app, too_large_to_hold, those, into, copying);\n    \
         complete_here_then_tell_the_server(app, list, cache, asks, one_sentence, fallback);\n}\n",
    );
    snippet.push_str(THE_WORKER);
    snippet.push_str(
        ") {\n    handle.block_on(move_it_across(controller, &from, uid, taking, &into));\n}\n",
    );
    snippet.push_str(THE_REPLAY_HELPER);
    snippet.push_str(
        ") {\n    replay_the_moves_waiting_for(controller, cache, account_id);\n    \
         replay_the_crossings_waiting_for(&TheAccountsSetUpHere(accounts), cache, account_id);\n}\n",
    );
    snippet.push_str("        ");
    snippet.push_str(THE_PUT_BACK_ARM);
    snippet.push_str(
        "\n            let going_to = waiting.the_account_it_is_going_to();\n            \
         undo_here(cache, waiting);\n        }\n        UIUpdate::Other => {}\n",
    );
    snippet.push_str(THE_NETWORK_BACK_ARM);
    snippet.push_str("\n    offer_to_check();\n}\n");
    snippet
}

fn every_reading_over(app: &str) -> Result<(), String> {
    the_arms_complete_a_crossing_here_first(app)?;
    only_the_ceiling_goes_server_first(app)?;
    the_question_at_start_is_retired(app)?;
    the_helper_replays_crossings_after_moves(app)?;
    the_network_coming_back_replays_no_crossing(app)?;
    the_put_back_arm_reads_the_destination_under_its_account(app)
}

#[test]
fn test_the_readings_pass_a_window_shaped_as_it_should_be() {
    every_reading_over(&a_window_as_it_should_be()).unwrap_or_else(|why| panic!("{why}"));
    the_sync_names_the_crossing_replay(
        "    // replayed through `replay_the_crossings_waiting_for` first\n",
    )
    .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_readings_complain_when_a_crossing_waits_for_the_servers_again() {
    let app = a_window_as_it_should_be();

    let routed_to_the_worker = app.replacen(
        "fn move_or_copy_message() {\n    move_or_copy_here_first(app, list, &cache, a11y, asked);",
        "fn move_or_copy_message() {\n    spawn_folder_move(app, moving, chosen, into, copying);",
        1,
    );
    let why = the_arms_complete_a_crossing_here_first(&routed_to_the_worker)
        .expect_err("the crossing goes to the worker");
    assert!(why.contains("still reaches spawn_folder_move("), "{why}");

    let no_crossing_kind = app.replacen(
        "    let what = WhatAWaitingMoveDoes::MoveAcross { into, to_account };\n",
        "",
        1,
    );
    let why =
        the_arms_complete_a_crossing_here_first(&no_crossing_kind).expect_err("no crossing kind");
    assert!(why.contains("knows no MoveAcross"), "{why}");

    let session_first = app.replacen(
        "    complete_here_then_tell_the_server(app, list, cache, asks, one_sentence, fallback);",
        "    the_session_at(&account);\n    \
         complete_here_then_tell_the_server(app, list, cache, asks, one_sentence, fallback);",
        1,
    );
    let why = the_arms_complete_a_crossing_here_first(&session_first)
        .expect_err("a session before the row leaves");
    assert!(why.contains("reaches the_session_at("), "{why}");
}

#[test]
fn test_the_readings_complain_when_the_ceiling_path_is_wrong() {
    let app = a_window_as_it_should_be();

    let crossing_in_front_elsewhere = app.replacen(
        "    send_status(tx, rt, \"larger than 25 MB, so it goes now\");\n",
        "    send_status(tx, rt, \"larger than 25 MB, so it goes now\");\n    \
         handle.block_on(move_it_across(controller, &from, uid, taking, &into));\n",
        1,
    );
    let why = only_the_ceiling_goes_server_first(&crossing_in_front_elsewhere)
        .expect_err("the whole crossing outside the worker");
    assert!(why.contains("somewhere else"), "{why}");

    let nothing_said = app.replacen(
        "    send_status(tx, rt, \"larger than 25 MB, so it goes now\");\n",
        "",
        1,
    );
    let why = only_the_ceiling_goes_server_first(&nothing_said).expect_err("nothing said");
    assert!(why.contains("does not reach larger than 25 MB"), "{why}");

    let no_ceiling = app.replacen(
        "    let can_be_held = size <= LARGEST_MESSAGE_KEPT_WHILE_IT_MOVES_BYTES;\n",
        "",
        1,
    );
    let why = only_the_ceiling_goes_server_first(&no_ceiling).expect_err("no ceiling");
    assert!(
        why.contains("does not reach LARGEST_MESSAGE_KEPT_WHILE_IT_MOVES_BYTES"),
        "{why}"
    );
}

#[test]
fn test_the_readings_complain_when_a_restart_asks_or_a_replay_is_misplaced() {
    let app = a_window_as_it_should_be();

    let asks_at_start = format!(
        "{app}{THE_QUESTION_AT_START}(app) {{\n    cache.moves_that_did_not_finish();\n}}\n"
    );
    let why = the_question_at_start_is_retired(&asks_at_start).expect_err("asks at start");
    assert!(why.contains("still in the window"), "{why}");

    let crossings_first = app.replacen(
        "    replay_the_moves_waiting_for(controller, cache, account_id);\n    \
         replay_the_crossings_waiting_for(&TheAccountsSetUpHere(accounts), cache, account_id);",
        "    replay_the_crossings_waiting_for(&TheAccountsSetUpHere(accounts), cache, account_id);\n    \
         replay_the_moves_waiting_for(controller, cache, account_id);",
        1,
    );
    let why = the_helper_replays_crossings_after_moves(&crossings_first)
        .expect_err("the crossings first");
    assert!(why.contains("goes first"), "{why}");

    let wired_to_the_network = app.replacen(
        "UIUpdate::TheNetworkIsBack => {\n    offer_to_check();",
        "UIUpdate::TheNetworkIsBack => {\n    \
         replay_the_crossings_waiting_for(&TheAccountsSetUpHere(accounts), cache, id);",
        1,
    );
    let why = the_network_coming_back_replays_no_crossing(&wired_to_the_network)
        .expect_err("wired to the network");
    assert!(why.contains("Nobody asked"), "{why}");

    let read_under_the_wrong_account = app.replacen(
        "            let going_to = waiting.the_account_it_is_going_to();\n",
        "            let going_to = &waiting.account_id;\n",
        1,
    );
    let why =
        the_put_back_arm_reads_the_destination_under_its_account(&read_under_the_wrong_account)
            .expect_err("the wrong account");
    assert!(
        why.contains("does not reach the_account_it_is_going_to()"),
        "{why}"
    );

    let why = the_sync_names_the_crossing_replay("    // nothing subtracted here\n")
        .expect_err("the sync says nothing");
    assert!(why.contains("no longer names"), "{why}");
}

//! One runner carries a settled set of actions out over the chosen
//! messages, through the set commands' quiet do-halves, in an order the
//! server sees (13-24.1, RESEARCH-4's AUT-6 tasks 2 and 3).
//!
//! #60 asks that a Quick Step reuse the rules engine rather than a second
//! one, #61 that a rule run over a folder go through the existing gated
//! write paths, and blocking a sender (GAP-06) moves the mail already here.
//! `run_these_actions_over` in the main window is that one path: it meets
//! each account's gate before anything changes, refuses a set above the
//! bound, then calls the do-halves 13-24 made in a fixed order, the flags,
//! then the labels, then the phrase said first, then the move or the
//! delete, and answers what it did without saying a word, so each caller
//! says one sentence of its own.
//!
//! Source readings over `what_ships` of `src/presentation/wx_app.rs`, with
//! comment lines left out, because a comment explaining why the runner does
//! not announce names the call it does not make. Each reading has a
//! companion that plants the fault into a snippet shaped as the window
//! should be, so a reading that stopped finding its anchor cannot pass by
//! finding nothing.
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
//! Whether the order holds at the server. The runner starts a worker per
//! flag and one push per account for a move, and which reaches the account's
//! session first is the scheduler's; nothing here or in the library's own
//! tests can drive both workers against one scripted server (ledger, 13-24.1).
//! What the readings hold is the order the changes are asked in and, for a
//! flag, that the folder it names is the one read when it was asked. A call
//! that says something from inside a do-half is the other target's
//! (`each_set_command_has_a_quiet_do_half`). The window is not started.

use std::fs;

use wixen_mail::common::what_ships::what_ships;

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

const THE_RUNNER: &str = "fn run_these_actions_over(";

fn the_main_window() -> String {
    let whole = fs::read_to_string(THE_MAIN_WINDOW)
        .unwrap_or_else(|why| panic!("{THE_MAIN_WINDOW}: {why}"))
        .replace("\r\n", "\n");
    what_ships(&whole)
}

/// One function's text, from its signature to the closing brace at column
/// nought, without its comment lines, or a complaint when the signature is
/// gone.
fn code_of(source: &str, signature: &str) -> Result<String, String> {
    let at = source.find(signature).ok_or(format!(
        "{signature} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let ends = rest.find("\n}\n").map_or(rest.len(), |end| end + 2);
    Ok(rest[..ends]
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n"))
}

/// Where `name` is called in `code`: each place `name(` stands after
/// something that cannot end an identifier, so `remove_these(` is not a
/// call of `move_these(`.
fn calls_of(code: &str, name: &str) -> Vec<usize> {
    let call = format!("{name}(");
    code.match_indices(&call)
        .map(|(at, _)| at)
        .filter(|&at| {
            code[..at]
                .chars()
                .next_back()
                .is_none_or(|before| !(before.is_alphanumeric() || before == '_'))
        })
        .collect()
}

// ── The anchors ────────────────────────────────────────────────────────────

/// The do-halves, in the groups the runner calls them in: each group's
/// calls all come before any call of the next group's.
const IN_ORDER: [(&str, &[&str]); 4] = [
    ("the flags", &["mark_these_read", "star_these"]),
    ("the labels", &["label_these"]),
    ("the phrase said first", &["set_says_first"]),
    ("the move or the delete", &["move_these", "delete_these"]),
];

/// Every do-half, whichever group it is in.
fn every_do_half() -> impl Iterator<Item = &'static str> {
    IN_ORDER.iter().flat_map(|(_, names)| names.iter().copied())
}

/// The calls that say something to somebody. The runner makes none of
/// them: it answers what it did, and a refusal is its answer too.
const SAYS: [&str; 6] = [
    "announce",
    "signal",
    "send_shown",
    "send_status",
    "send_refusal",
    "say_the_one_word",
];

// ── The readings ───────────────────────────────────────────────────────────

/// The first place any do-half is called, or a complaint naming one that
/// is not called at all.
fn the_first_change(code: &str) -> Result<usize, String> {
    let mut first = usize::MAX;
    for name in every_do_half() {
        let at = calls_of(code, name).into_iter().min().ok_or(format!(
            "{THE_RUNNER} never calls {name}(, so one kind of action a rule or a Quick Step \
             names is carried out some other way or not at all"
        ))?;
        first = first.min(at);
    }
    Ok(first)
}

/// `before` is called, and every call of it comes before the first change.
fn met_before_anything_changes(code: &str, before: &str, why: &str) -> Result<(), String> {
    let first_change = the_first_change(code)?;
    let calls = calls_of(code, before);
    if calls.is_empty() {
        return Err(format!("{THE_RUNNER} never calls {before}(, so {why}"));
    }
    if calls.iter().any(|&at| at > first_change) {
        return Err(format!(
            "{THE_RUNNER} calls {before}( after a do-half has changed something, so {why}"
        ));
    }
    Ok(())
}

fn the_gate_comes_first(app: &str) -> Result<(), String> {
    let code = code_of(app, THE_RUNNER)?;
    met_before_anything_changes(
        &code,
        "outward::permitted",
        "an account whose changes are off has some of its messages changed here before \
         the refusal",
    )
}

fn the_bound_comes_first(app: &str) -> Result<(), String> {
    let code = code_of(app, THE_RUNNER)?;
    met_before_anything_changes(
        &code,
        "too_many",
        "a set above the bound every set command refuses is part carried out first",
    )
}

/// Every group against every later group, not only its neighbour, so a
/// move asked for before the flags is named as that rather than as the
/// first neighbouring pair it happens to put out of order.
fn the_do_halves_are_called_in_order(app: &str) -> Result<(), String> {
    let code = code_of(app, THE_RUNNER)?;
    the_first_change(&code)?;
    let calls_in = |names: &[&str]| -> Vec<usize> {
        names
            .iter()
            .flat_map(|name| calls_of(&code, name))
            .collect()
    };
    for (at, (earlier, earlier_names)) in IN_ORDER.iter().enumerate() {
        for (later, later_names) in &IN_ORDER[at + 1..] {
            let last_of_earlier = calls_in(earlier_names).into_iter().max();
            let first_of_later = calls_in(later_names).into_iter().min();
            if let (Some(last), Some(first)) = (last_of_earlier, first_of_later)
                && last > first
            {
                return Err(format!(
                    "{THE_RUNNER} calls {later} before {earlier}, so the server is asked \
                     for them out of order, and a message moved before its flags are \
                     asked for has them sent to a folder it has left"
                ));
            }
        }
    }
    Ok(())
}

fn the_runner_says_nothing(app: &str) -> Result<(), String> {
    let code = code_of(app, THE_RUNNER)?;
    if let Some(said) = SAYS.iter().find(|says| !calls_of(&code, says).is_empty()) {
        return Err(format!(
            "{THE_RUNNER} calls {said}(, so a run says a sentence of its own beside its \
             caller's and a Quick Step over a set is heard twice"
        ));
    }
    Ok(())
}

// ── The readings over the window ───────────────────────────────────────────

#[test]
fn test_the_runner_meets_each_accounts_gate_before_anything_changes() {
    the_gate_comes_first(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_runner_refuses_a_set_above_the_bound_before_anything_changes() {
    the_bound_comes_first(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_runner_asks_for_the_flags_then_the_labels_then_the_phrase_then_the_move() {
    the_do_halves_are_called_in_order(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_runner_says_nothing_and_answers_what_it_did() {
    the_runner_says_nothing(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

// ── The companions: the window as it should be, with a fault planted ───────

/// A runner shaped as the window's should be: the bound, the gate, then
/// each do-half in order, with a comment that names a saying call it does
/// not make.
const A_RUNNER_AS_IT_SHOULD_BE: &str = "\
fn run_these_actions_over(
    app: AppHandles<'_>,
) -> Result<WhatWasDone, String> {
    if let Some(why) = too_many(chosen.messages.len()) {
        return Err(why);
    }
    crate::service::outward::permitted(may_change, \"change messages\")?;
    // Nothing here calls announce(: the caller says the sentence.
    mark_these_read(app, cache, list, &those, read);
    star_these(app, cache, list, &those, starred);
    label_these(app, cache, &those, &held, change);
    cache.set_says_first(message.row_id, Some(phrase));
    remember_the_set_leaving(state, &moving);
    move_these(app, list, cache, moving, into, false);
    delete_these(app, cache, list, &those, Deleting::ToTrash);
    Ok(done)
}
";

/// Every reading, for the companions that require all of them to pass.
fn every_reading(app: &str) -> Result<(), String> {
    the_gate_comes_first(app)?;
    the_bound_comes_first(app)?;
    the_do_halves_are_called_in_order(app)?;
    the_runner_says_nothing(app)
}

/// The snippet with the line holding `line` moved to just before the line
/// holding `before`.
fn moved(snippet: &str, line: &str, before: &str) -> String {
    let lines: Vec<&str> = snippet.lines().collect();
    let moving = lines
        .iter()
        .position(|held| held.contains(line))
        .unwrap_or_else(|| panic!("{line} is not in the snippet"));
    let mut rest: Vec<&str> = lines.clone();
    let taken = rest.remove(moving);
    let at = rest
        .iter()
        .position(|held| held.contains(before))
        .unwrap_or_else(|| panic!("{before} is not in the snippet"));
    rest.insert(at, taken);
    let planted = rest.join("\n") + "\n";
    assert_ne!(planted, snippet, "moving {line} changed nothing");
    planted
}

#[test]
fn test_the_readings_pass_a_runner_shaped_as_it_should_be() {
    every_reading(A_RUNNER_AS_IT_SHOULD_BE).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_order_reading_names_a_move_asked_for_before_the_flags() {
    let planted = moved(A_RUNNER_AS_IT_SHOULD_BE, "move_these(", "star_these(");
    let why = the_do_halves_are_called_in_order(&planted).expect_err("a move before a flag");
    assert!(
        why.contains("the move or the delete before the flags"),
        "{why}"
    );
}

#[test]
fn test_the_order_reading_names_a_label_put_on_after_the_move() {
    let planted = moved(A_RUNNER_AS_IT_SHOULD_BE, "label_these(", "delete_these(");
    let why = the_do_halves_are_called_in_order(&planted).expect_err("a label after the move");
    // Moved to just before the delete, the label comes after the phrase as
    // well, and the first pair out of order is the one named.
    assert!(
        why.contains("calls the phrase said first before the labels"),
        "{why}"
    );
}

#[test]
fn test_the_gate_reading_names_a_gate_met_after_a_do_half() {
    let planted = moved(
        A_RUNNER_AS_IT_SHOULD_BE,
        "outward::permitted(",
        "label_these(",
    );
    let why = the_gate_comes_first(&planted).expect_err("a gate after a change");
    assert!(why.contains("outward::permitted( after a do-half"), "{why}");
}

#[test]
fn test_the_bound_reading_names_a_bound_met_after_a_do_half() {
    let planted = moved(A_RUNNER_AS_IT_SHOULD_BE, "too_many(", "move_these(");
    let why = the_bound_comes_first(&planted).expect_err("a bound after a change");
    assert!(why.contains("too_many( after a do-half"), "{why}");
}

#[test]
fn test_the_saying_reading_names_every_saying_call_planted_in_the_runner() {
    for says in SAYS {
        let planted = A_RUNNER_AS_IT_SHOULD_BE.replacen(
            "    Ok(done)",
            &format!("    {says}(\"Done\");\n    Ok(done)"),
            1,
        );
        assert_ne!(planted, A_RUNNER_AS_IT_SHOULD_BE, "nothing was planted");
        let why = the_runner_says_nothing(&planted).expect_err("a runner that says something");
        assert!(why.contains(&format!("calls {says}(")), "{why}");
    }
}

#[test]
fn test_the_readings_name_a_do_half_the_runner_never_calls() {
    let planted = A_RUNNER_AS_IT_SHOULD_BE.replacen(
        "    label_these(app, cache, &those, &held, change);\n",
        "",
        1,
    );
    assert_ne!(planted, A_RUNNER_AS_IT_SHOULD_BE, "nothing was removed");
    let why = the_do_halves_are_called_in_order(&planted).expect_err("a label never put on");
    assert!(why.contains("never calls label_these("), "{why}");
}

// ── A flag change names the folder it was asked in (RESEARCH-4, F4) ────────
//
// The flag's worker used to read the message's folder when it ran. A move
// recorded here between the key and the worker, which the runner's own
// order makes routine and M then Ctrl+Shift+V pressed quickly made
// possible before it, left the worker reading the folder the message was
// going to, and sending the flag there with the number the message had in
// the folder it left: a flag on whatever message holds that number there.
// The folder is read when the change is asked for and handed to the worker.

const THE_FLAG_CHANGE: &str = "fn spawn_server_change(";

/// The parameter the folder arrives in, and the type it has: the folder
/// read by the caller when the change was asked for, or nothing when it is
/// in no folder this program knows.
const THE_FOLDER_ASKED_IN: &str = "asked_in: Option<String>";

/// Where `name` stands as a whole identifier in `code`.
fn mentions_of(code: &str, name: &str) -> Vec<usize> {
    let is_part_of_a_name = |c: char| c.is_alphanumeric() || c == '_';
    code.match_indices(name)
        .map(|(at, _)| at)
        .filter(|&at| {
            let before = code[..at].chars().next_back();
            let after = code[at + name.len()..].chars().next();
            !before.is_some_and(is_part_of_a_name) && !after.is_some_and(is_part_of_a_name)
        })
        .collect()
}

fn the_flag_change_names_the_folder_it_was_asked_in(app: &str) -> Result<(), String> {
    let code = code_of(app, THE_FLAG_CHANGE)?;
    let closes = code.find("\n)").ok_or(format!(
        "{THE_FLAG_CHANGE} has no parameter list closing at column nought, so this reads nothing"
    ))?;
    let (parameters, body) = code.split_at(closes);
    if mentions_of(parameters, "asked_in").is_empty() || !parameters.contains(THE_FOLDER_ASKED_IN) {
        return Err(format!(
            "{THE_FLAG_CHANGE} takes no {THE_FOLDER_ASKED_IN} from its caller, so its worker \
             finds the folder when it runs, after a move may have recorded the message in \
             another"
        ));
    }
    if !calls_of(body, "folder_path_for_message").is_empty() {
        return Err(format!(
            "{THE_FLAG_CHANGE} still calls folder_path_for_message( itself, so the flag goes \
             to the folder the message is in when the worker runs rather than when the change \
             was asked for"
        ));
    }
    if mentions_of(body, "asked_in").is_empty() {
        return Err(format!(
            "{THE_FLAG_CHANGE} is handed the folder asked in and never uses it, so the server \
             is told some other folder"
        ));
    }
    Ok(())
}

#[test]
fn test_a_flag_change_names_the_folder_it_was_asked_in() {
    the_flag_change_names_the_folder_it_was_asked_in(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

/// A flag change shaped as the window's should be: the folder handed in by
/// the caller and named to the server, with a comment naming the read it
/// no longer makes.
const A_FLAG_CHANGE_AS_IT_SHOULD_BE: &str = "\
fn spawn_server_change(
    app: AppHandles<'_>,
    message_row_id: i64,
    uid: u32,
    subject: String,
    asked_in: Option<String>,
    change: ServerChange,
) {
    rt.spawn_blocking(move || {
        // Not cache.folder_path_for_message(message_row_id): see above.
        let Some(folder_path) = asked_in else {
            return;
        };
        controller.set_flag(&folder_path, uid, flag, true);
    });
}
";

#[test]
fn test_the_flag_reading_passes_a_worker_shaped_as_it_should_be() {
    the_flag_change_names_the_folder_it_was_asked_in(A_FLAG_CHANGE_AS_IT_SHOULD_BE)
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_flag_reading_names_the_folder_read_when_the_worker_runs() {
    let planted = A_FLAG_CHANGE_AS_IT_SHOULD_BE.replacen(
        "        let Some(folder_path) = asked_in else {",
        "        let _ = asked_in;\n        let Some(folder_path) = cache.folder_path_for_message(message_row_id).ok().flatten() else {",
        1,
    );
    assert_ne!(
        planted, A_FLAG_CHANGE_AS_IT_SHOULD_BE,
        "nothing was planted"
    );
    let why = the_flag_change_names_the_folder_it_was_asked_in(&planted)
        .expect_err("a worker that reads the folder when it runs");
    assert!(
        why.contains("still calls folder_path_for_message("),
        "{why}"
    );
}

#[test]
fn test_the_flag_reading_names_a_worker_handed_no_folder() {
    let planted = A_FLAG_CHANGE_AS_IT_SHOULD_BE.replacen("    asked_in: Option<String>,\n", "", 1);
    assert_ne!(
        planted, A_FLAG_CHANGE_AS_IT_SHOULD_BE,
        "nothing was removed"
    );
    let why = the_flag_change_names_the_folder_it_was_asked_in(&planted)
        .expect_err("a worker handed no folder");
    assert!(why.contains("takes no asked_in"), "{why}");
}

#[test]
fn test_the_flag_reading_names_a_folder_handed_and_never_used() {
    let planted = A_FLAG_CHANGE_AS_IT_SHOULD_BE.replacen(
        "        let Some(folder_path) = asked_in else {",
        "        let Some(folder_path) = the_inbox() else {",
        1,
    );
    assert_ne!(
        planted, A_FLAG_CHANGE_AS_IT_SHOULD_BE,
        "nothing was planted"
    );
    let why = the_flag_change_names_the_folder_it_was_asked_in(&planted)
        .expect_err("a folder handed and never used");
    assert!(why.contains("never uses it"), "{why}");
}

// ── The runner has a caller (13-25) ────────────────────────────────────────
//
// 13-24.1 built the runner with no caller and an expectation that the build
// would fail the day one arrived. The block's move of the mail already here
// is the first (13-25); this holds that something outside the runner calls
// it, so it cannot drift back to code nothing reaches.

/// Somewhere other than the runner's own signature calls it.
fn the_runner_has_a_caller(app: &str) -> Result<(), String> {
    let callers = calls_of(app, "run_these_actions_over")
        .into_iter()
        .filter(|&at| !app[..at].ends_with("fn "))
        .count();
    match callers {
        0 => Err(format!(
            "nothing calls {THE_RUNNER} so a block, a Quick Step or a rule over a folder reaches \
             none of it"
        )),
        _ => Ok(()),
    }
}

#[test]
fn test_the_runner_has_a_caller_outside_itself() {
    the_runner_has_a_caller(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_caller_reading_names_a_runner_nobody_calls() {
    let why = the_runner_has_a_caller(A_RUNNER_AS_IT_SHOULD_BE).expect_err("a runner alone");
    assert!(why.contains("nothing calls"), "{why}");
}

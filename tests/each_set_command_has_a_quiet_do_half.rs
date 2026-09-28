//! Each command that changes a set of messages carries the change out in a
//! quiet function of its own, its do-half, and says its sentence itself
//! (13-24, RESEARCH-4's AUT-6 task 1).
//!
//! 13-24.1's runner carries several of a rule's or a Quick Step's actions
//! over one set of messages, and it can only do that through pieces that
//! change the messages and say nothing: a runner that called the commands
//! would say each command's sentence, several sentences for one key. So
//! Mark as Read, Star, a label, Move and Delete each keep their read half
//! (the selection, the refusals, the bound, which way to go) and their say
//! half (the sentence, the shown line, the signal), and hand the doing to a
//! do-half: a function that takes the chosen messages and which way to go,
//! carries the change out here and at the server through the paths it
//! always used, and answers what it did.
//!
//! Source readings over `what_ships` of `src/presentation/wx_app.rs`: each
//! do-half exists and takes the set and which way; its body holds none of
//! the calls that say something; and its command calls it and carries none
//! of the change out itself. Each reading has a companion that plants the
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
//! Whether anything heard moved, which the pull request's NVDA run reads.
//! A saying call inside a function a do-half calls: the line each message
//! gets as it leaves (#83) is part of the move and is said by the move's
//! own path, not by the command. And a refusal, which a do-half may still
//! say where the gate it meets refuses before anything changes. The window
//! is not started.

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
fn the_id_arm(source: &str, heading: &str) -> Result<String, String> {
    let start = source.find(heading).ok_or(format!(
        "{heading:?} is no longer here, so this reads nothing"
    ))? + heading.len();
    let rest = &source[start..];
    let end = rest.find("_ if id ==").unwrap_or(rest.len());
    Ok(rest[..end].to_string())
}

/// A command's text, whether it is a function or an arm of the dispatch.
fn the_command(source: &str, command: &str) -> Result<String, String> {
    match command.starts_with("_ if id ==") {
        true => the_id_arm(source, command),
        false => body_of(source, command),
    }
}

/// The parameters of one function: from its signature to the line that
/// closes them.
fn the_parameters<'a>(body: &'a str, signature: &str) -> Result<&'a str, String> {
    let end = body.find("\n)").ok_or(format!(
        "{signature} has no parameter list closing at column nought, so this reads nothing"
    ))?;
    Ok(&body[..end])
}

// ── The anchors ────────────────────────────────────────────────────────────

/// The calls that say something: a sentence spoken, a signal played, a
/// line shown or spoken, the one word at the key. A do-half makes none of
/// them, so a runner calling several of them says nothing until it says
/// its one sentence.
const SAYS: [&str; 5] = [
    "announce(",
    "signal(",
    "send_shown(",
    "send_status(",
    "say_the_one_word(",
];

/// One command over a set and the do-half it hands the doing to.
struct ADoHalf {
    /// What the person calls the command, for the complaint.
    command: &'static str,
    /// The do-half's signature.
    signature: &'static str,
    /// A word in its parameters naming the chosen messages.
    the_set: &'static str,
    /// A parameter saying which way to go, decided by the caller.
    which_way: &'static str,
    /// The commands that call it: a function's signature or an arm's heading.
    callers: &'static [&'static str],
    /// Calls only the doing makes, which no caller makes for itself.
    the_doing: &'static [&'static str],
}

const THE_DO_HALVES: [ADoHalf; 5] = [
    ADoHalf {
        command: "Mark as Read",
        signature: "fn mark_these_read(",
        the_set: "Chosen",
        which_way: "read: bool",
        callers: &["fn toggle_read_state("],
        the_doing: &["write_flags_or_put_the_row_back(", "spawn_server_change("],
    },
    ADoHalf {
        command: "Star",
        signature: "fn star_these(",
        the_set: "Chosen",
        which_way: "starred: bool",
        callers: &["_ if id == ID_TOGGLE_STAR =>"],
        the_doing: &["write_flags_or_put_the_row_back(", "spawn_server_change("],
    },
    ADoHalf {
        command: "a label",
        signature: "fn label_these(",
        the_set: "Chosen",
        which_way: "change: LabelChange",
        callers: &["fn label_the_message("],
        the_doing: &[
            "add_tag_to_message(",
            "remove_tag_from_message(",
            "spawn_server_change(",
        ],
    },
    // The chosen messages for a move are the messages moving, each with the
    // folder and the account it is in and its size, because Report as Junk
    // reads those at the key and moves only once the mark is settled, when
    // its rows may have left the screen. Report as Junk moves from the
    // update its worker sends, so its caller is the function that update
    // reaches.
    ADoHalf {
        command: "Move to, Copy to and Report as Junk",
        signature: "fn move_these(",
        the_set: "AMessageMoving",
        which_way: "into:",
        callers: &["fn move_or_copy_message(", "fn move_what_was_reported("],
        the_doing: &["complete_here_then_tell_the_server(", "spawn_folder_move("],
    },
    ADoHalf {
        command: "Delete",
        signature: "fn delete_these(",
        the_set: "Chosen",
        which_way: "asked: Deleting",
        callers: &["_ if id == ID_DELETE || id == ID_DELETE_OUTRIGHT =>"],
        the_doing: &[
            "complete_here_then_tell_the_server(",
            "where_a_delete_goes_here(",
            "cancel_if_queued(",
            "delete_if_local(",
        ],
    },
];

fn the_do_half(command: &str) -> &'static ADoHalf {
    THE_DO_HALVES
        .iter()
        .find(|half| half.command == command)
        .unwrap_or_else(|| panic!("no do-half is listed for {command}"))
}

// ── The readings ───────────────────────────────────────────────────────────

/// The do-half is there, takes the chosen set and which way to go, and its
/// body says nothing.
fn the_do_half_is_there_and_quiet(app: &str, half: &ADoHalf) -> Result<(), String> {
    let body = body_of(app, half.signature)?;
    let parameters = the_parameters(&body, half.signature)?;
    for needed in [half.the_set, half.which_way] {
        if !parameters.contains(needed) {
            return Err(format!(
                "{} takes no {needed}, so {} decides for itself what it does to which messages",
                half.signature, half.command
            ));
        }
    }
    if let Some(said) = SAYS.iter().find(|says| body.contains(*says)) {
        return Err(format!(
            "{} calls {said}, so a runner carrying {} out with other actions says a sentence \
             of its own for it",
            half.signature, half.command
        ));
    }
    Ok(())
}

/// Each command calls its do-half and carries none of the change out itself.
fn the_command_calls_its_do_half(app: &str, half: &ADoHalf) -> Result<(), String> {
    let call = half.signature.trim_start_matches("fn ");
    for caller in half.callers {
        let text = the_command(app, caller)?;
        if !text.contains(call) {
            return Err(format!(
                "{caller} does not call {call}, so {} and the runner carry the change out \
                 two ways that can drift apart",
                half.command
            ));
        }
        if let Some(inline) = half.the_doing.iter().find(|doing| text.contains(*doing)) {
            return Err(format!(
                "{caller} still calls {inline} itself, so part of {} is carried out beside \
                 the do-half rather than by it",
                half.command
            ));
        }
    }
    Ok(())
}

fn reads_whole(app: &str, half: &ADoHalf) -> Result<(), String> {
    the_do_half_is_there_and_quiet(app, half)?;
    the_command_calls_its_do_half(app, half)
}

// ── The readings over the window ───────────────────────────────────────────

#[test]
fn test_mark_as_read_carries_its_change_out_in_a_quiet_do_half() {
    reads_whole(&the_main_window(), the_do_half("Mark as Read"))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_star_carries_its_change_out_in_a_quiet_do_half() {
    reads_whole(&the_main_window(), the_do_half("Star")).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_label_carries_its_change_out_in_a_quiet_do_half() {
    reads_whole(&the_main_window(), the_do_half("a label")).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_move_copy_and_report_as_junk_carry_their_moves_out_in_a_quiet_do_half() {
    reads_whole(
        &the_main_window(),
        the_do_half("Move to, Copy to and Report as Junk"),
    )
    .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_delete_carries_its_change_out_in_a_quiet_do_half() {
    reads_whole(&the_main_window(), the_do_half("Delete")).unwrap_or_else(|why| panic!("{why}"));
}

// ── The companions: the window as it should be, with a fault planted ───────

/// A snippet shaped as the window should be: each do-half with its
/// parameters and one call of its doing, and each command calling it.
fn a_window_as_it_should_be() -> String {
    let mut snippet = String::new();
    for half in &THE_DO_HALVES {
        snippet.push_str(&format!(
            "{}\n    app: AppHandles<'_>,\n    set: &{},\n    {},\n) -> Answer {{\n    {}app);\n}}\n",
            half.signature, half.the_set, half.which_way, half.the_doing[0]
        ));
        for caller in half.callers {
            snippet.push_str(&format!(
                "{caller} {{\n    let answer = {}app, &chosen, way);\n}}\n",
                half.signature.trim_start_matches("fn ")
            ));
            // An arm is read to the next arm, so each is closed by one.
            if caller.starts_with("_ if id ==") {
                snippet.push_str("_ if id == ID_OTHER => {}\n");
            }
        }
    }
    snippet
}

/// The snippet with `planted` added as the first line of `signature`'s body.
fn planted_in(snippet: &str, signature: &str, planted: &str) -> String {
    let at = snippet
        .find(signature)
        .unwrap_or_else(|| panic!("{signature} is not in the snippet"));
    let opens = at
        + snippet[at..]
            .find("{\n")
            .unwrap_or_else(|| panic!("{signature} opens no body in the snippet"))
        + 2;
    format!("{}    {planted}\n{}", &snippet[..opens], &snippet[opens..])
}

#[test]
fn test_the_readings_pass_a_window_shaped_as_it_should_be() {
    let app = a_window_as_it_should_be();
    for half in &THE_DO_HALVES {
        reads_whole(&app, half).unwrap_or_else(|why| panic!("{why}"));
    }
}

#[test]
fn test_the_reading_names_every_saying_call_planted_in_every_do_half() {
    let app = a_window_as_it_should_be();
    for half in &THE_DO_HALVES {
        for says in SAYS {
            let planted = planted_in(&app, half.signature, &format!("{says}\"Done\");"));
            let why = the_do_half_is_there_and_quiet(&planted, half)
                .expect_err("a do-half that says something");
            assert!(why.contains(half.signature) && why.contains(says), "{why}");
        }
    }
}

#[test]
fn test_the_reading_names_a_command_that_carries_its_change_out_itself() {
    let app = a_window_as_it_should_be();
    for half in &THE_DO_HALVES {
        for caller in half.callers {
            for doing in half.the_doing {
                let planted = planted_in(&app, caller, &format!("{doing}app);"));
                let why = the_command_calls_its_do_half(&planted, half)
                    .expect_err("a command carrying its change out itself");
                assert!(why.contains("still calls") && why.contains(doing), "{why}");
            }
        }
    }
}

#[test]
fn test_the_reading_names_a_command_that_does_not_call_its_do_half() {
    let app = a_window_as_it_should_be();
    for half in &THE_DO_HALVES {
        let call = half.signature.trim_start_matches("fn ");
        let bypassed = app.replace(&format!("= {call}"), "= carry_it_out_here(");
        let why = the_command_calls_its_do_half(&bypassed, half)
            .expect_err("a command that does not call its do-half");
        assert!(why.contains("does not call"), "{why}");
    }
}

#[test]
fn test_the_reading_names_a_do_half_that_decides_for_itself() {
    let app = a_window_as_it_should_be();
    for half in &THE_DO_HALVES {
        for dropped in [half.the_set, half.which_way] {
            let at = app.find(half.signature).expect("the signature");
            let parameters = &app[at..at + app[at..].find("\n)").expect("the parameters")];
            let without = app.replacen(parameters, &parameters.replace(dropped, "Nothing"), 1);
            let why = the_do_half_is_there_and_quiet(&without, half)
                .expect_err("a do-half that decides for itself");
            assert!(why.contains(&format!("takes no {dropped}")), "{why}");
        }
    }
}

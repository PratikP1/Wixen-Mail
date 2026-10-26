//! Every window the main window's tick can open opens after the tick returns.
//!
//! The main window polls every 50 ms, and seven things that handler reaches
//! open a modal window: the Due now window, the question about folders a server
//! stopped listing, the two questions about an update, the question before a
//! counted rule runs, the question about a key in an attachment and the
//! question about mail a block caught. Opened from inside the handler, each
//! window's loop ran the next tick inside the one that opened it, which
//! wxDragon 0.9.17 allows and 0.9.23 skips with a warning, so after the move
//! the tick would stop while any of them was open. Each is handed to
//! `presentation::after_the_tick` instead, and
//! `the_main_tick_never_runs_inside_itself.rs` shows that queue with the real
//! Due now window.
//!
//! These read the source rather than run it. The tick is a closure inside the
//! function that starts the whole program, so nothing short of starting it, with
//! a profile, accounts and workers, reaches these seven places. Each reads the shipped half of `wx_app.rs` through
//! `what_ships`, and each has a companion that plants the old shape and is
//! refused, and the new shape and is accepted, so a reading that answers the
//! same thing whatever it reads cannot pass.

use std::fs;
use wixen_mail::common::what_ships::what_ships;

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

/// What hands a window on, as it is called.
const THE_QUEUE: &str = "after_the_tick";

const THE_HANDLER: &str = "fn handle_update(update: &UIUpdate, targets: UpdateTargets<'_>) {";
const THE_LOOK: &str = "fn raise_what_is_due(";
const THE_FOLDERS_QUESTION: &str = "fn ask_about_the_folders_that_have_gone(";

/// An arm of the handler starts a line indented eight spaces.
const AN_ARM: &str = "\n        UIUpdate::";

fn the_main_window() -> String {
    let whole = fs::read_to_string(THE_MAIN_WINDOW)
        .unwrap_or_else(|why| panic!("{THE_MAIN_WINDOW}: {why}"))
        .replace("\r\n", "\n");
    what_ships(&whole)
}

fn without_comments(code: &str) -> String {
    code.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// One function's text, from its signature to the closing brace at column
/// nought, without its comment lines.
fn body_of(source: &str, signature: &str) -> Result<String, String> {
    let at = source.find(signature).ok_or(format!(
        "{signature} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let ends = rest.find("\n}\n").map_or(rest.len(), |end| end + 2);
    Ok(without_comments(&rest[..ends]))
}

/// One arm of the update handler, up to the next arm.
fn arm_of(handler: &str, variant: &str) -> Result<String, String> {
    let marker = format!("{AN_ARM}{variant}");
    let at = handler.find(&marker).ok_or(format!(
        "the update handler has no arm for UIUpdate::{variant}, so this reads nothing"
    ))?;
    let rest = &handler[at + marker.len()..];
    let ends = rest.find(AN_ARM).unwrap_or(rest.len());
    Ok(rest[..ends].to_string())
}

/// Where `name` is called in `code`: each place `name(` stands after
/// something that cannot end an identifier.
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

/// Where the identifier `name` stands in `code`, whole.
fn uses_of(code: &str, name: &str) -> Vec<usize> {
    let part_of_a_name = |c: char| c.is_alphanumeric() || c == '_';
    code.match_indices(name)
        .map(|(at, _)| at)
        .filter(|&at| {
            !code[..at].chars().next_back().is_some_and(part_of_a_name)
                && !code[at + name.len()..]
                    .chars()
                    .next()
                    .is_some_and(part_of_a_name)
        })
        .collect()
}

/// Where the bracket opened at `open` closes, stepping over string and
/// character literals, or the end of `code`.
fn where_it_closes(code: &str, open: usize) -> usize {
    let bytes = code.as_bytes();
    let mut depth = 0_usize;
    let mut at = open;
    while at < bytes.len() {
        match bytes[at] {
            b'(' | b'{' | b'[' => depth += 1,
            b')' | b'}' | b']' => {
                depth -= 1;
                if depth == 0 {
                    return at;
                }
            }
            b'"' => at = the_end_of_a_string(bytes, at),
            b'\'' => at = the_end_of_a_character(bytes, at),
            _ => {}
        }
        at += 1;
    }
    bytes.len()
}

/// The closing quote of the string opened at `at`, raw strings included.
fn the_end_of_a_string(bytes: &[u8], at: usize) -> usize {
    let hashes = bytes[..at].iter().rev().take_while(|b| **b == b'#').count();
    let raw = at > hashes && bytes[at - hashes - 1] == b'r';
    let mut end = at + 1;
    while end < bytes.len() {
        match bytes[end] {
            b'\\' if !raw => end += 1,
            b'"' if bytes[end + 1..].iter().take(hashes).all(|b| *b == b'#') => {
                return end + hashes;
            }
            _ => {}
        }
        end += 1;
    }
    bytes.len()
}

/// The closing quote of a character literal opened at `at`, or `at` itself
/// when the quote begins a lifetime.
fn the_end_of_a_character(bytes: &[u8], at: usize) -> usize {
    match (bytes.get(at + 1), bytes.get(at + 2)) {
        (Some(b'\\'), _) => bytes[at + 2..]
            .iter()
            .position(|b| *b == b'\'')
            .map_or(bytes.len(), |end| at + 2 + end),
        (Some(_), Some(b'\'')) => at + 2,
        _ => at,
    }
}

/// The stretches of `code` handed to the queue: from each call of it to the
/// bracket that closes the call.
fn handed_on(code: &str) -> Vec<(usize, usize)> {
    calls_of(code, THE_QUEUE)
        .into_iter()
        .map(|at| {
            let open = at + THE_QUEUE.len();
            (at, where_it_closes(code, open))
        })
        .collect()
}

fn is_handed_on(stretches: &[(usize, usize)], at: usize) -> bool {
    stretches.iter().any(|(from, to)| *from < at && at < *to)
}

/// Every place in `code` that opens a window, `opens`, is inside a stretch
/// handed to the queue, and there is at least one.
fn every_opening_is_handed_on(code: &str, opens: &str, whose: &str) -> Result<(), String> {
    let stretches = handed_on(code);
    let openings: Vec<usize> = code.match_indices(opens).map(|(at, _)| at).collect();
    if openings.is_empty() {
        return Err(format!(
            "{whose} no longer holds {opens}, so this reads nothing"
        ));
    }
    let outside = openings
        .iter()
        .filter(|at| !is_handed_on(&stretches, **at))
        .count();
    match outside {
        0 => Ok(()),
        n => Err(format!(
            "{whose} reaches {opens} {n} of {} times outside a closure handed to \
             {THE_QUEUE}(, so the window opens inside the tick and the tick runs inside it",
            openings.len()
        )),
    }
}

/// The work handed on starts after `first`, the thing that must happen while
/// the tick is still running.
fn handed_on_after(code: &str, first: &str, whose: &str) -> Result<(), String> {
    let done = code
        .find(first)
        .ok_or(format!("{whose} no longer holds {first}"))?;
    match handed_on(code).first() {
        Some((from, _)) if done < *from => Ok(()),
        Some(_) => Err(format!(
            "{whose} hands the window on before {first}, which has to happen in the tick"
        )),
        None => Err(format!("{whose} hands nothing to {THE_QUEUE}(")),
    }
}

/// The one-at-a-time turn taken in `code` is moved into the work handed on,
/// so it is held until the window closes rather than given back when the
/// tick returns.
fn the_turn_goes_with_the_window(code: &str, gate: &str, whose: &str) -> Result<(), String> {
    let taken = code
        .find(&format!("{gate}.take()"))
        .ok_or(format!("{whose} takes no turn from {gate}"))?;
    let binding = code[..taken]
        .rfind("let ")
        .ok_or(format!("{whose} takes its turn without naming it"))?;
    let name: String = code[binding + "let ".len()..taken]
        .trim_start_matches("Some(")
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    let stretches = handed_on(code);
    let used_in_the_work = uses_of(code, &name)
        .into_iter()
        .any(|at| is_handed_on(&stretches, at));
    match used_in_the_work {
        true => Ok(()),
        false => Err(format!(
            "{whose} keeps its turn `{name}` in the tick, so the turn is given back when the \
             tick returns and another window can open over this one"
        )),
    }
}

fn the_due_now_window_waits(window: &str) -> Result<(), String> {
    let look = body_of(window, THE_LOOK)?;
    every_opening_is_handed_on(&look, "wx_reminder_alert::raise(", THE_LOOK)?;
    handed_on_after(&look, "already.borrow_mut().insert(", THE_LOOK)?;
    the_turn_goes_with_the_window(&look, "one_at_a_time", THE_LOOK)
}

fn the_folders_question_waits(window: &str) -> Result<(), String> {
    let asking = body_of(window, THE_FOLDERS_QUESTION)?;
    every_opening_is_handed_on(&asking, "asking::ask(", THE_FOLDERS_QUESTION)?;
    handed_on_after(&asking, ".raised(", THE_FOLDERS_QUESTION)?;
    the_turn_goes_with_the_window(&asking, "one_question_on_screen", THE_FOLDERS_QUESTION)
}

fn the_update_questions_wait(window: &str) -> Result<(), String> {
    let handler = body_of(window, THE_HANDLER)?;
    for variant in ["AnUpdateDidNotHappen", "AnUpdateIsReady"] {
        let arm = arm_of(&handler, variant)?;
        every_opening_is_handed_on(&arm, "asking::ask(", &format!("the arm for {variant}"))?;
    }
    Ok(())
}

/// The arms whose question lives in a function of its own, and that function.
const THE_ASKING_ARMS: [(&str, &str); 3] = [
    ("ARuleWasCounted", "ask_then_run_the_counted_rule("),
    ("KeyAttachmentOffered", "offer_the_key_attachment("),
    ("WhatABlockCaught", "answer_what_a_block_caught("),
];

fn the_asking_arms_wait(window: &str) -> Result<(), String> {
    let handler = body_of(window, THE_HANDLER)?;
    for (variant, asks) in THE_ASKING_ARMS {
        let arm = arm_of(&handler, variant)?;
        every_opening_is_handed_on(&arm, asks, &format!("the arm for {variant}"))?;
    }
    Ok(())
}

fn no_window_opens_in_the_handler(window: &str) -> Result<(), String> {
    let handler = body_of(window, THE_HANDLER)?;
    every_opening_is_handed_on(&handler, "asking::ask(", "the update handler")?;
    // A window opened any other way is held to the same rule wherever one
    // appears; since 13.4-12.3 every question in the handler is asked above.
    match handler.contains(".show_modal()") {
        true => every_opening_is_handed_on(&handler, ".show_modal()", "the update handler"),
        false => Ok(()),
    }
}

#[test]
fn test_the_due_now_window_opens_after_the_tick() {
    assert_eq!(the_due_now_window_waits(&the_main_window()), Ok(()));
}

#[test]
fn test_the_folders_question_opens_after_the_tick() {
    assert_eq!(the_folders_question_waits(&the_main_window()), Ok(()));
}

#[test]
fn test_the_two_update_questions_open_after_the_tick() {
    assert_eq!(the_update_questions_wait(&the_main_window()), Ok(()));
}

#[test]
fn test_the_counted_rule_the_key_and_the_block_questions_open_after_the_tick() {
    assert_eq!(the_asking_arms_wait(&the_main_window()), Ok(()));
}

#[test]
fn test_no_window_opens_inside_handle_update() {
    assert_eq!(no_window_opens_in_the_handler(&the_main_window()), Ok(()));
}

/// A look shaped the way `raise_what_is_due` is, with its window placed by
/// `window`.
fn a_look(turn: &str, window: &str) -> String {
    format!(
        "fn raise_what_is_due(frame: &Frame) {{\n\
         \x20   let Some({turn}) = one_at_a_time.take() else {{\n\
         \x20       return;\n\
         \x20   }};\n\
         \x20   for item in &due {{\n\
         \x20       already.borrow_mut().insert(item.identity.clone());\n\
         \x20   }}\n\
         {window}\n\
         }}\n"
    )
}

#[test]
fn test_the_due_now_reading_refuses_a_window_opened_in_the_tick() {
    let in_the_tick = "    let answers = wx_reminder_alert::raise(frame, due, \"(\");";
    let handed_on = "    after_the_tick(move || {\n\
                     \x20       let _turn = turn;\n\
                     \x20       let answers = wx_reminder_alert::raise(&frame, due, \")\");\n\
                     \x20   });";
    let turn_left_behind = "    after_the_tick(move || {\n\
                            \x20       let answers = wx_reminder_alert::raise(&frame, due);\n\
                            \x20   });";
    assert_eq!(the_due_now_window_waits(&a_look("turn", handed_on)), Ok(()));
    assert!(
        the_due_now_window_waits(&a_look("_turn", in_the_tick)).is_err(),
        "a window opened in the tick was passed"
    );
    assert!(
        the_due_now_window_waits(&a_look("turn", turn_left_behind)).is_err(),
        "a turn kept in the tick was passed"
    );
}

/// A folders question shaped the way the real one is.
fn a_folders_question(window: &str) -> String {
    format!(
        "fn ask_about_the_folders_that_have_gone(frame: &Frame) {{\n\
         \x20   let turn = one_question_on_screen.take();\n\
         \x20   let Some(question) = what_to_raise(&waiting.borrow(), typing, turn.is_none()) else {{\n\
         \x20       return;\n\
         \x20   }};\n\
         \x20   waiting.borrow_mut().raised(&question);\n\
         {window}\n\
         }}\n"
    )
}

#[test]
fn test_the_folders_reading_refuses_a_question_put_in_the_tick() {
    let in_the_tick = "    let asked = asking::ask(frame, Question::RemoveFoldersTheServerNoLongerLists, &q.title, &q.words);";
    let handed_on = "    let frame = *frame;\n\
                     \x20   after_the_tick(move || {\n\
                     \x20       let _turn = turn;\n\
                     \x20       let asked = asking::ask(&frame, Question::RemoveFoldersTheServerNoLongerLists, &q.title, &q.words);\n\
                     \x20   });";
    assert_eq!(
        the_folders_question_waits(&a_folders_question(handed_on)),
        Ok(())
    );
    assert!(
        the_folders_question_waits(&a_folders_question(in_the_tick)).is_err(),
        "a question put in the tick was passed"
    );
}

/// An update handler with the arms these readings read, each placed by the
/// caller.
fn a_handler(did_not_happen: &str, ready: &str, counted: &str, key: &str, block: &str) -> String {
    format!(
        "fn handle_update(update: &UIUpdate, targets: UpdateTargets<'_>) {{\n\
         \x20   match update {{\n\
         \x20       UIUpdate::AnUpdateDidNotHappen {{ said, page, asked }} => {{\n{did_not_happen}\n        }}\n\
         \x20       UIUpdate::AnUpdateIsReady {{ installer }} => {{\n{ready}\n        }}\n\
         \x20       UIUpdate::ARuleWasCounted(counted) => {{\n{counted}\n        }}\n\
         \x20       UIUpdate::KeyAttachmentOffered(offered) => {{\n{key}\n        }}\n\
         \x20       UIUpdate::WhatABlockCaught(caught) => {{\n{block}\n        }}\n\
         \x20       UIUpdate::CommandRefused(why) => {{}}\n\
         \x20   }}\n\
         }}\n"
    )
}

const A_QUESTION_IN_THE_TICK: &str =
    "            let asked = asking::ask(&frame, Question::InstallAnUpdate, title, words);";
const A_QUESTION_HANDED_ON: &str = "            after_the_tick(move || {\n\
                                    \x20               let asked = asking::ask(&frame, Question::InstallAnUpdate, title, words);\n\
                                    \x20           });";

fn asks_in_the_tick(name: &str) -> String {
    format!("            {name}frame, counted);")
}

fn asks_after_the_tick(name: &str) -> String {
    format!("            after_the_tick(move || {name}&frame, &counted));")
}

fn every_asking_arm_handed_on() -> [String; 3] {
    THE_ASKING_ARMS.map(|(_, asks)| asks_after_the_tick(asks))
}

#[test]
fn test_the_update_and_handler_readings_refuse_a_question_put_in_the_tick() {
    let [counted, key, block] = every_asking_arm_handed_on();
    let right = a_handler(
        A_QUESTION_HANDED_ON,
        A_QUESTION_HANDED_ON,
        &counted,
        &key,
        &block,
    );
    assert_eq!(the_update_questions_wait(&right), Ok(()));
    assert_eq!(no_window_opens_in_the_handler(&right), Ok(()));

    for wrong in [
        a_handler(
            A_QUESTION_IN_THE_TICK,
            A_QUESTION_HANDED_ON,
            &counted,
            &key,
            &block,
        ),
        a_handler(
            A_QUESTION_HANDED_ON,
            A_QUESTION_IN_THE_TICK,
            &counted,
            &key,
            &block,
        ),
    ] {
        assert!(
            the_update_questions_wait(&wrong).is_err(),
            "an update question put in the tick was passed:\n{wrong}"
        );
        assert!(
            no_window_opens_in_the_handler(&wrong).is_err(),
            "the handler's census passed a window opened in the tick:\n{wrong}"
        );
    }
}

#[test]
fn test_the_census_refuses_a_window_a_later_arm_opens_in_the_tick() {
    let [counted, key, block] = every_asking_arm_handed_on();
    let later = format!("{block}\n{A_QUESTION_IN_THE_TICK}");
    let handler = a_handler(
        A_QUESTION_HANDED_ON,
        A_QUESTION_HANDED_ON,
        &counted,
        &key,
        &later,
    );
    assert!(
        no_window_opens_in_the_handler(&handler).is_err(),
        "a window opened in the tick by another arm was passed"
    );
}

#[test]
fn test_the_asking_reading_refuses_each_question_put_in_the_tick() {
    let right = every_asking_arm_handed_on();
    let [counted, key, block] = right.clone();
    assert_eq!(
        the_asking_arms_wait(&a_handler(
            A_QUESTION_HANDED_ON,
            A_QUESTION_HANDED_ON,
            &counted,
            &key,
            &block
        )),
        Ok(())
    );
    for (which, (_, asks)) in THE_ASKING_ARMS.iter().enumerate() {
        let mut arms = right.clone();
        arms[which] = asks_in_the_tick(asks);
        let [counted, key, block] = arms;
        let handler = a_handler(
            A_QUESTION_HANDED_ON,
            A_QUESTION_HANDED_ON,
            &counted,
            &key,
            &block,
        );
        assert!(
            the_asking_arms_wait(&handler).is_err(),
            "{asks} called in the tick was passed"
        );
    }
}

#[test]
fn test_the_bracket_reading_steps_over_brackets_in_literals() {
    let code = "after_the_tick(move || { say(\")\"); say(')'); open('a'); }); outside";
    let stretches = handed_on(code);
    assert_eq!(stretches.len(), 1);
    let (_, closes) = stretches[0];
    assert_eq!(&code[closes..], "); outside");
}

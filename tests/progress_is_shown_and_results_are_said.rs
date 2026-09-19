//! A step is shown and spoken only when every step was asked for; what
//! arrived is said once, with counts; an error is said whatever was chosen;
//! and the answer to a key is said above the run of progress.
//!
//! #38, 10-04. Until 2026-09-17 every line a check wrote to the status bar
//! went out as `StatusUpdated`, spoken at Low under one topic, and a check of
//! fifty folders spoke fifty lines; "Settings saved" rode the same channel
//! and was replaced by the next sync line before it was heard. The fix sorts
//! the lines into kinds where they are made: a step goes out as
//! `UIUpdate::Progress`, whose arm asks the level before speaking; the
//! folders that received something go out once, after the loop, as
//! `UIUpdate::WhatArrived`, whose arm speaks at Normal unless nothing but
//! errors was asked for; and what stays on `StatusUpdated`, the answers to a
//! key, is spoken at Normal.
//!
//! A third kind since 2026-09-18 (#83, 11-06.1): `UIUpdate::Shown`, a line
//! the eye may want and the ear has already had, written to the status bar
//! and spoken by nothing. The success of a delete rides it, because the row
//! the cursor lands on is what is heard. Its reading is beside the other
//! two: the arm shows and never speaks, and `send_shown` is what sends it.
//!
//! Read from the source rather than run, because reaching the update
//! handler needs a window, a frame and a running event loop, and the
//! question is which channel each line was put on, which is a property of
//! the text. Each reading is a function over that text with a companion that
//! plants the opposite in the real text and requires a complaint naming it,
//! so a reading that passes over anything is found here rather than by the
//! next tester. What none of this can see: whether any sentence is heard,
//! and whether a step and a result are told apart by ear. That is the
//! listening page's.

use std::fs;
use wixen_mail::common::what_ships::what_ships;

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

/// The openings of a step, none of which may ride the answer channel.
const PROGRESS_OPENINGS: [&str; 6] = [
    "\"Checking ",
    "\"Connecting ",
    "\"Loading ",
    "\"Syncing ",
    "\"Fetching ",
    "sync requested",
];

/// The five lines a mail check says on the way, by their opening words or
/// the function that words them, each of which must go out as a step.
const THE_CHECKS_STEPS: [&str; 5] = [
    "\"Connecting to {}...\"",
    "how_many_on_the_server(",
    "\"Checking {}...\"",
    "what_the_folder_sync_did(",
    "\"Mail check finished.",
];

fn the_window_itself() -> String {
    fs::read_to_string(THE_MAIN_WINDOW)
        .unwrap_or_else(|e| panic!("{THE_MAIN_WINDOW}: {e}"))
        .replace("\r\n", "\n")
}

/// The one routine that handles every update, cut at the closing brace in
/// the first column, or a complaint when it is gone.
fn the_update_handler(source: &str) -> Result<String, String> {
    let after = source
        .split_once("fn handle_update(update: &UIUpdate, targets: UpdateTargets<'_>) {")
        .ok_or("the update handler is no longer called handle_update, so this reads nothing")?
        .1;
    let end = after.find("\n}\n").unwrap_or(after.len());
    Ok(after[..end].to_string())
}

/// One arm of that routine, from its label to the start of the next, or a
/// complaint when there is no such arm.
fn the_arm_for(handler: &str, variant: &str) -> Result<String, String> {
    let opens = format!("        UIUpdate::{variant}");
    let at = handler.find(&opens).ok_or(format!(
        "there is no arm for UIUpdate::{variant}, so this reads nothing"
    ))?;
    let rest = &handler[at + opens.len()..];
    let ends = rest.find("\n        UIUpdate::").unwrap_or(rest.len());
    Ok(rest[..ends].to_string())
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

/// The one `announce_topic` call in an arm, its arguments and nothing else,
/// so the prose beside it, which names the topics that were not chosen,
/// cannot be read as the call.
fn the_announcement_in(arm: &str) -> Result<&str, String> {
    let after = arm
        .split_once("a11y.announce_topic(")
        .ok_or("the arm announces nothing, so what it shows is written to nobody")?
        .1;
    let end = after.find(");").unwrap_or(after.len());
    Ok(&after[..end])
}

/// A step is spoken only when every step was asked for, said as a
/// complaint when the arm speaks without asking.
///
/// The arm must show the line, then ask the level, and only inside that ask
/// announce it, at Low and under a topic of its own rather than the answer
/// channel's.
fn a_step_is_spoken_only_when_every_step_was(app: &str) -> Result<(), String> {
    let arm = the_arm_for(&the_update_handler(app)?, "Progress(")?;
    if !arm.contains("set_status_text(") {
        return Err("the Progress arm no longer shows the step on the status bar".to_string());
    }
    let asks = arm.find("is_spoken(Kind::Progress)").ok_or(
        "the Progress arm does not ask whether a step is spoken, so every step is spoken \
         under every choice, which is the verbosity #38 was filed about",
    )?;
    let speaks = arm
        .find("a11y.announce_topic(")
        .ok_or("the Progress arm never speaks, so somebody who asked for every step hears none")?;
    if speaks < asks {
        return Err(
            "the Progress arm speaks before it asks the level, so the ask governs nothing"
                .to_string(),
        );
    }
    let call = the_announcement_in(&arm)?;
    if !call.contains("Priority::Low") {
        return Err(format!(
            "a step is announced above Low, so it queues ahead of what matters: {call}"
        ));
    }
    if call.contains("\"status\"") {
        return Err(
            "a step is announced on the answer channel's topic, where it replaces Settings \
             saved before anybody hears it"
                .to_string(),
        );
    }
    Ok(())
}

/// What arrived is said once at Normal unless nothing but errors was asked
/// for, and the new-mail event is signalled there.
fn what_arrived_is_said_at_normal_and_signals_new_mail(app: &str) -> Result<(), String> {
    let arm = the_arm_for(&the_update_handler(app)?, "WhatArrived")?;
    if !arm.contains("set_status_text(") {
        return Err("the WhatArrived arm no longer shows the result on the status bar".to_string());
    }
    let asks = arm.find("is_spoken(Kind::Result)").ok_or(
        "the WhatArrived arm does not ask whether a result is spoken, so Errors only still \
         hears every arrival",
    )?;
    let speaks = arm
        .find("a11y.announce_topic(")
        .ok_or("the WhatArrived arm never speaks, so what arrived is written to nobody")?;
    if speaks < asks {
        return Err(
            "the WhatArrived arm speaks before it asks the level, so the ask governs nothing"
                .to_string(),
        );
    }
    let call = the_announcement_in(&arm)?;
    if !call.contains("Priority::Normal") {
        return Err(format!(
            "what arrived is not announced at Normal, so a step behind it can bury it: {call}"
        ));
    }
    if call.contains("\"status\"") {
        return Err("what arrived is announced on the answer channel's topic".to_string());
    }
    if !arm.contains("a11y.signal(FeedbackEvent::NewMail") {
        return Err(
            "the WhatArrived arm does not signal NewMail, so the sound for new mail no \
             longer means mail arrived"
                .to_string(),
        );
    }
    Ok(())
}

/// The answer to a key on the status channel is said at Normal, so a sync
/// underneath cannot bury it.
fn an_answer_on_the_status_channel_is_said_at_normal(app: &str) -> Result<(), String> {
    let arm = the_arm_for(&the_update_handler(app)?, "StatusUpdated(")?;
    let call = the_announcement_in(&arm)?;
    if !call.contains("Priority::Normal") {
        return Err(format!(
            "what remains on StatusUpdated, Settings saved and the other answers to a key, \
             is announced below Normal, where a sync buries it: {call}"
        ));
    }
    Ok(())
}

/// A line sent as shown is written to the status bar and its record, and
/// spoken by nothing; and `send_shown` is the sender that puts a line there.
fn a_shown_line_is_written_and_never_spoken(app: &str) -> Result<(), String> {
    let arm = the_arm_for(&the_update_handler(app)?, "Shown(")?;
    if !arm.contains("set_status_text(") || !arm.contains("status_message") {
        return Err(
            "the Shown arm no longer writes the line to the status bar and its record, so a \
             line for the eye reaches nobody"
                .to_string(),
        );
    }
    if arm.contains("a11y.announce") || arm.contains("a11y.signal") {
        return Err(
            "the Shown arm announces or signals, so a line meant for the eye alone is spoken \
             over the row the cursor landed on"
                .to_string(),
        );
    }
    let sender = body_of(&what_ships(app), "fn send_shown(")?;
    if !sender.contains("UIUpdate::Shown(") {
        return Err(
            "send_shown does not send UIUpdate::Shown, so nothing rides the shown channel"
                .to_string(),
        );
    }
    Ok(())
}

/// The mail check's five lines on the way go out as steps, and what arrived
/// goes out once, after the loop.
fn the_checks_lines_are_steps_and_its_arrivals_one_result(app: &str) -> Result<(), String> {
    let body = body_of(&what_ships(app), "fn spawn_mail_sync(")?;
    for opening in THE_CHECKS_STEPS {
        let at = body.find(opening).ok_or(format!(
            "the mail check no longer says {opening}, so this reads nothing about it"
        ))?;
        let before = &body[..at];
        let sent_as = before
            .rfind("UIUpdate::")
            .map(|from| &before[from + "UIUpdate::".len()..])
            .ok_or(format!("{opening} is not sent as any update at all"))?;
        if !sent_as.starts_with("Progress(") {
            return Err(format!(
                "{opening} goes out as UIUpdate::{}, not as a step, so it is spoken under \
                 Say what arrived",
                sent_as.split(['(', ' ', '\n']).next().unwrap_or_default()
            ));
        }
    }
    let arrivals = body.matches("UIUpdate::WhatArrived").count();
    if arrivals != 1 {
        return Err(format!(
            "the mail check sends WhatArrived {arrivals} times, where the counts go out \
             once, after the loop"
        ));
    }
    let after_the_loop = body
        .find("\"Mail check finished.")
        .ok_or("the mail check no longer says it finished")?;
    let arrival = body
        .find("UIUpdate::WhatArrived")
        .ok_or("no WhatArrived to place")?;
    if arrival < after_the_loop {
        return Err(
            "WhatArrived is sent before the check has finished, so it cannot carry every \
             folder's count"
                .to_string(),
        );
    }
    Ok(())
}

/// The new-mail event is signalled once, when a check found mail, and not
/// when the watch wakes.
fn the_new_mail_sound_means_mail_arrived(app: &str) -> Result<(), String> {
    let handler = the_update_handler(app)?;
    let woke = the_arm_for(&handler, "MailboxChanged(")?;
    if woke.contains("FeedbackEvent::NewMail") {
        return Err(
            "the MailboxChanged arm signals NewMail, which fires when the watch wakes, \
             before the folder is read and whether or not anything arrives"
                .to_string(),
        );
    }
    let signals = what_ships(app)
        .matches("a11y.signal(FeedbackEvent::NewMail")
        .count();
    if signals != 1 {
        return Err(format!(
            "NewMail is signalled from {signals} places, where the one place is the arm \
             that a check reaches when it found mail"
        ));
    }
    Ok(())
}

/// Every call that puts a line on the answer channel, `send_status` or a
/// bare `StatusUpdated`, from the call to its closing bracket.
fn every_answer_channel_call(ship: &str) -> Vec<(usize, String)> {
    let mut calls = Vec::new();
    for (opens, _) in ship
        .match_indices("send_status(")
        .chain(ship.match_indices("UIUpdate::StatusUpdated("))
    {
        let rest = &ship[opens..];
        let mut depth = 0usize;
        let mut end = rest.len();
        for (offset, c) in rest.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = offset + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        let line = ship[..opens].matches('\n').count() + 1;
        calls.push((line, rest[..end].to_string()));
    }
    calls
}

/// No step rides the answer channel, said as a complaint naming each line
/// that does.
///
/// The arms above hold what each channel does with a line; this holds that
/// no line of the other kind was put on the wrong one, by its opening words,
/// which is as much as reading source gives.
fn no_step_rides_the_answer_channel(app: &str) -> Result<(), String> {
    let ship = what_ships(app);
    let calls = every_answer_channel_call(&ship);
    if calls.len() <= 40 {
        return Err(format!(
            "only {} answer-channel calls were read, so the reading is broken",
            calls.len()
        ));
    }
    let wrong: Vec<String> = calls
        .iter()
        .filter(|(_, call)| {
            PROGRESS_OPENINGS
                .iter()
                .any(|opening| call.contains(opening))
        })
        .map(|(line, call)| {
            format!(
                "{THE_MAIN_WINDOW}:{line}: {}",
                call.split_whitespace().collect::<Vec<_>>().join(" ")
            )
        })
        .collect();
    if !wrong.is_empty() {
        return Err(format!(
            "these steps ride the answer channel, so they are spoken under Say what arrived \
             and replace the answer to a key where it stands:\n  {}",
            wrong.join("\n  ")
        ));
    }
    Ok(())
}

// ── The readings, over the window as it ships ────────────────────────────────

#[test]
fn test_a_step_is_spoken_only_when_every_step_was_asked_for() {
    a_step_is_spoken_only_when_every_step_was(&the_window_itself())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_what_arrived_is_said_once_at_normal_and_signals_new_mail() {
    what_arrived_is_said_at_normal_and_signals_new_mail(&the_window_itself())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_settings_saved_and_the_other_answers_are_said_at_normal() {
    an_answer_on_the_status_channel_is_said_at_normal(&the_window_itself())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_line_sent_as_shown_is_written_and_never_spoken() {
    a_shown_line_is_written_and_never_spoken(&the_window_itself())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_mail_checks_lines_are_steps_and_what_arrived_goes_out_once_after_the_loop() {
    the_checks_lines_are_steps_and_its_arrivals_one_result(&the_window_itself())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_new_mail_sound_plays_when_a_check_found_mail_and_not_when_the_watch_woke() {
    the_new_mail_sound_means_mail_arrived(&the_window_itself())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_no_step_rides_the_answer_channel() {
    no_step_rides_the_answer_channel(&the_window_itself()).unwrap_or_else(|why| panic!("{why}"));
}

// ── The companions, each planting the opposite in the real text ──────────────

/// The real text with one substring replaced, or a panic saying the anchor
/// is gone, so a companion cannot pass by planting nothing.
fn with(app: &str, from: &str, to: &str) -> String {
    assert!(
        app.contains(from),
        "the companion's anchor is not in the window any more, so it plants nothing: {from}"
    );
    app.replacen(from, to, 1)
}

#[test]
fn test_the_reading_complains_when_a_step_is_spoken_without_asking() {
    let planted = with(
        &the_window_itself(),
        "if a11y.how_much_to_say().is_spoken(Kind::Progress) {",
        "{",
    );
    let why = a_step_is_spoken_only_when_every_step_was(&planted)
        .expect_err("a step spoken without asking the level was passed over");
    assert!(why.contains("does not ask"), "{why}");
}

#[test]
fn test_the_reading_complains_when_what_arrived_is_said_at_low() {
    let planted = with(
        &the_window_itself(),
        "a11y.announce_topic(what, Priority::Normal, \"arrived\")",
        "a11y.announce_topic(what, Priority::Low, \"arrived\")",
    );
    let why = what_arrived_is_said_at_normal_and_signals_new_mail(&planted)
        .expect_err("a result announced at Low was passed over");
    assert!(why.contains("not announced at Normal"), "{why}");
}

#[test]
fn test_the_reading_complains_when_an_answer_is_said_at_low() {
    let planted = with(
        &the_window_itself(),
        "a11y.announce_topic(status, Priority::Normal, \"status\")",
        "a11y.announce_topic(status, Priority::Low, \"status\")",
    );
    let why = an_answer_on_the_status_channel_is_said_at_normal(&planted)
        .expect_err("an answer announced at Low was passed over");
    assert!(why.contains("below Normal"), "{why}");
}

#[test]
fn test_the_reading_complains_when_a_shown_line_is_spoken() {
    let planted = with(
        &the_window_itself(),
        "        UIUpdate::Shown(shown) => {\n",
        "        UIUpdate::Shown(shown) => {\n            let _ = a11y.announce(shown, Priority::Normal);\n",
    );
    let why = a_shown_line_is_written_and_never_spoken(&planted)
        .expect_err("a shown line that is spoken was passed over");
    assert!(why.contains("announces or signals"), "{why}");
}

#[test]
fn test_the_reading_complains_when_a_folder_check_line_goes_out_as_an_answer() {
    let app = the_window_itself();
    let body = body_of(&app, "fn spawn_mail_sync(").expect("the mail check");
    let checking = body
        .find("\"Checking {}...\"")
        .expect("the per-folder line");
    let sent = body[..checking]
        .rfind("UIUpdate::Progress(")
        .expect("the per-folder line goes out as a step");
    let mut broken = body.clone();
    broken.replace_range(
        sent..sent + "UIUpdate::Progress(".len(),
        "UIUpdate::StatusUpdated(",
    );
    let planted = app.replacen(&body, &broken, 1);
    let why = the_checks_lines_are_steps_and_its_arrivals_one_result(&planted)
        .expect_err("a folder check line on the answer channel was passed over");
    assert!(
        why.contains("Checking") && why.contains("StatusUpdated"),
        "{why}"
    );
}

#[test]
fn test_the_reading_complains_when_the_new_mail_signal_is_back_where_the_watch_wakes() {
    let planted = with(
        &the_window_itself(),
        "        UIUpdate::MailboxChanged(folder) => {\n",
        "        UIUpdate::MailboxChanged(folder) => {\n            let _ = a11y.signal(FeedbackEvent::NewMail, \"\");\n",
    );
    let why = the_new_mail_sound_means_mail_arrived(&planted)
        .expect_err("the signal back in the MailboxChanged arm was passed over");
    assert!(why.contains("watch wakes"), "{why}");
}

#[test]
fn test_the_reading_complains_when_a_step_rides_the_answer_channel() {
    let planted = with(
        &the_window_itself(),
        "pub(crate) fn send_status(",
        "fn a_planted_step(tx: &Sender<UIUpdate>, rt: &Arc<Runtime>) {\n    \
         send_status(tx, rt, \"Checking the planted folder...\");\n}\n\
         pub(crate) fn send_status(",
    );
    let why = no_step_rides_the_answer_channel(&planted)
        .expect_err("a step on the answer channel was passed over");
    assert!(why.contains("Checking the planted folder"), "{why}");
}

#[test]
fn test_the_reading_refuses_a_window_with_too_few_answer_channel_calls_to_be_the_window() {
    let too_few = "fn one(tx: &Sender<UIUpdate>, rt: &Arc<Runtime>) {\n    \
                   send_status(tx, rt, \"Draft saved\");\n}\n";
    let why = no_step_rides_the_answer_channel(too_few)
        .expect_err("a text with one call was read as the window");
    assert!(why.contains("reading is broken"), "{why}");
}

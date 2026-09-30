//! A rule runs over a folder when somebody asks, from Action, This Folder and
//! from the Filter Manager, saying first what it would change (#61, GAP-12,
//! 13-44).
//!
//! 13-43 built the count and its words and 13-24.1 the runner. Until this plan
//! nothing asked either: a rule ran once, when mail arrived, and never again.
//!
//! **Source readings** over `what_ships` of `src/presentation/wx_app.rs`,
//! comment lines left out. The This Folder item opens its door, which chooses
//! a rule before it counts. The count runs on a worker and sends what it found
//! back as `UIUpdate::ARuleWasCounted`. The arm for that update meets the
//! account's gate before it asks, asks before it runs, runs through 13-24.1's
//! one runner once, and says one sentence after it. None of the three writes
//! mail of its own: every write is the runner's, through the gated paths the
//! set commands use. Each reading has a companion that plants the fault into
//! a snippet shaped as the window should be, so a reading that stopped finding
//! its anchor cannot pass by finding nothing.
//!
//! **The real menu bar**, built by `WxMailApp::build_menu_bar`, read item by
//! item: This Folder holds the item, and its help carries the sentence that
//! no rule run has met a real mail server.
//!
//! **What is not read.** Whether a screen reader reads the question when it
//! opens, and whether the chooser, the question and the result are heard as
//! one act: those are the tester's ear and are in the ledger. No run reaches a
//! mail server here, and the window as a whole is not started.
//!
//! One window session for the file, every reading sharing it through a
//! `OnceLock`. Runs under `WIXEN_NO_AUDIO` as CI does, and on a temporary data
//! directory in `WIXEN_MAIL_DATA`, never a person's profile. The frame is
//! never shown.

#![cfg(windows)]

use std::collections::BTreeMap;
use std::fs;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::running_a_rule_now::RUNNING_A_RULE_NOW_IS_EXPERIMENTAL;
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::presentation::wx_app::WxMailApp;
use wxdragon::prelude::*;

type Harvest = BTreeMap<&'static str, String>;

// ── The source ────────────────────────────────────────────────────────────

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

/// Where This Folder's item is answered.
const THE_ITEM_ARM: &str = "_ if id == ID_RUN_A_RULE_HERE =>";

/// The door on This Folder: the folder on screen, then a rule chosen.
const THE_DOOR: &str = "fn run_a_rule_on_this_folder(";

/// Where a chosen rule is counted over a folder, on a worker.
const THE_COUNT: &str = "fn count_what_a_rule_would_change(";

/// Where the count arrives: the arm, not the place that sends it, which is
/// spelled the same up to its bracket.
const THE_COUNTED_ARM: &str = "UIUpdate::ARuleWasCounted(counted) =>";

/// Where a counted rule is asked about and run.
const THE_QUESTION: &str = "fn ask_then_run_the_counted_rule(";

/// The one runner a rule run is carried out through (13-24.1).
const THE_RUNNER: &str = "run_these_actions_over";

/// Every path that writes mail, each the runner's to call and never a
/// door's, the count's or the question's.
const THE_WRITES: [&str; 8] = [
    "spawn_server_change",
    "move_or_copy_here_first",
    "set_says_first",
    "mark_these_read",
    "star_these",
    "label_these",
    "move_these",
    "delete_these",
];

fn the_main_window() -> String {
    let whole = fs::read_to_string(THE_MAIN_WINDOW)
        .unwrap_or_else(|why| panic!("{THE_MAIN_WINDOW}: {why}"))
        .replace("\r\n", "\n");
    what_ships(&whole)
}

/// The text of the match arm that starts at `anchor`, up to `next_arm`,
/// without its comment lines.
fn arm_of(source: &str, anchor: &str, next_arm: &str) -> Result<String, String> {
    let at = source.find(anchor).ok_or(format!(
        "{anchor} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at + anchor.len()..];
    let ends = rest.find(next_arm).unwrap_or(rest.len());
    Ok(without_comments(&rest[..ends]))
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

fn without_comments(code: &str) -> String {
    code.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
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

/// Whether `first` is first called in `code` before `second` is, or why not.
fn called_before(code: &str, first: &str, second: &str, whose: &str) -> Result<(), String> {
    match (
        calls_of(code, first).first(),
        calls_of(code, second).first(),
    ) {
        (Some(a), Some(b)) if a < b => Ok(()),
        (Some(_), Some(_)) => Err(format!("{whose} calls {first}( after {second}(")),
        (None, _) => Err(format!("{whose} never calls {first}(")),
        (_, None) => Err(format!("{whose} never calls {second}(")),
    }
}

/// `name` is called exactly once in `code`.
fn called_once(code: &str, name: &str, whose: &str) -> Result<(), String> {
    match calls_of(code, name).len() {
        1 => Ok(()),
        n => Err(format!(
            "{whose} calls {name}( {n} times; it should call it once"
        )),
    }
}

/// This Folder's item opens the door, once.
fn the_item_opens_the_door(app: &str) -> Result<(), String> {
    let arm = arm_of(app, THE_ITEM_ARM, "_ if id")?;
    called_once(&arm, "run_a_rule_on_this_folder", THE_ITEM_ARM)
}

/// The door has a rule chosen before anything is counted.
fn the_door_chooses_before_it_counts(app: &str) -> Result<(), String> {
    let door = body_of(app, THE_DOOR)?;
    called_before(
        &door,
        "choose_from_list",
        "count_what_a_rule_would_change",
        THE_DOOR,
    )?;
    called_once(&door, "count_what_a_rule_would_change", THE_DOOR)
}

/// The count is taken on a worker and sent back as its own update.
fn the_count_is_taken_on_a_worker(app: &str) -> Result<(), String> {
    let count = body_of(app, THE_COUNT)?;
    called_before(
        &count,
        "spawn_blocking",
        "what_a_rule_would_change",
        THE_COUNT,
    )?;
    let asked = calls_of(&count, "what_a_rule_would_change")
        .first()
        .copied()
        .unwrap_or_default();
    match count[asked..].contains("UIUpdate::ARuleWasCounted(") {
        true => Ok(()),
        false => Err(format!(
            "{THE_COUNT} sends no UIUpdate::ARuleWasCounted( after the count, so nothing \
             ever asks the question"
        )),
    }
}

/// The counted arm hands the count to the question, once.
fn the_counted_arm_asks(app: &str) -> Result<(), String> {
    let arm = arm_of(app, THE_COUNTED_ARM, "UIUpdate::")?;
    called_once(&arm, "ask_then_run_the_counted_rule", THE_COUNTED_ARM)
}

/// The account's gate is met before the question is built, so a refusal is
/// said in place of a question whose Yes would be refused.
fn the_gate_is_met_before_the_question(app: &str) -> Result<(), String> {
    let question = body_of(app, THE_QUESTION)?;
    called_before(
        &question,
        "permitted",
        "MessageDialog::builder",
        THE_QUESTION,
    )
}

/// The question is asked and its answer read before the runner is called,
/// and the runner is called once.
fn the_question_comes_before_the_run(app: &str) -> Result<(), String> {
    let question = body_of(app, THE_QUESTION)?;
    called_before(
        &question,
        "MessageDialog::builder",
        THE_RUNNER,
        THE_QUESTION,
    )?;
    called_before(&question, "which_of_the_two", THE_RUNNER, THE_QUESTION)?;
    called_once(&question, THE_RUNNER, THE_QUESTION)
}

/// Nothing on the way to the runner writes mail of its own, and only the
/// question calls the runner.
fn nothing_writes_round_the_runner(app: &str) -> Result<(), String> {
    for (signature, may_run) in [(THE_DOOR, false), (THE_COUNT, false), (THE_QUESTION, true)] {
        let body = body_of(app, signature)?;
        let written: Vec<&str> = THE_WRITES
            .into_iter()
            .filter(|write| !calls_of(&body, write).is_empty())
            .collect();
        if !written.is_empty() {
            return Err(format!(
                "{signature} calls {written:?} itself, which go round the runner and its gate"
            ));
        }
        if !may_run && !calls_of(&body, THE_RUNNER).is_empty() {
            return Err(format!(
                "{signature} calls {THE_RUNNER}( before anybody was asked"
            ));
        }
    }
    Ok(())
}

/// After the runner the question says one sentence with one Confirmed, and
/// nothing on the status line of its own.
fn the_run_says_one_sentence(app: &str) -> Result<(), String> {
    let question = body_of(app, THE_QUESTION)?;
    let at = calls_of(&question, THE_RUNNER)
        .first()
        .copied()
        .ok_or(format!("{THE_QUESTION} never calls {THE_RUNNER}("))?;
    let after = &question[at..];
    let announced = calls_of(after, "announce").len();
    let signalled = calls_of(after, "signal").len();
    let spoken_on_the_status_line = calls_of(after, "send_status").len();
    match (announced, signalled, spoken_on_the_status_line) {
        (1, 1, 0) => Ok(()),
        _ => Err(format!(
            "after the runner, {THE_QUESTION} announces {announced} times, signals {signalled} \
             times and speaks on the status line {spoken_on_the_status_line} times; a rule run \
             is one sentence and one Confirmed"
        )),
    }
}

// ── The window session ────────────────────────────────────────────────────

/// The item on This Folder, as its label is written.
const THE_ITEM: &str = "Run a Ru&le on This Folder...";

fn submenu_labelled(menu: &Menu, wanted: &str) -> Option<Menu> {
    menu.get_menu_items().iter().find_map(|item| {
        let sub = item.get_sub_menu()?;
        match item.get_label() == wanted {
            true => Some(sub),
            false => submenu_labelled(&sub, wanted),
        }
    })
}

/// This Folder's items, each as its label and its help, one a line.
fn this_folders_items(bar: &MenuBar) -> String {
    let Some(menu) = (0..bar.get_menu_count())
        .filter_map(|at| bar.get_menu(at))
        .find_map(|menu| submenu_labelled(&menu, "&This Folder"))
    else {
        return "no This Folder submenu".to_string();
    };
    menu.get_menu_items()
        .iter()
        .map(|item| {
            format!(
                "{}\t{}",
                item.get_label(),
                menu.get_help_string(item.get_item_id())
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn take_the_harvest() -> Result<Harvest, String> {
    let data = tempfile::tempdir().map_err(|e| format!("a data directory: {e}"))?;
    // SAFETY: set before the window session starts any thread, and nothing
    // has read either yet.
    unsafe {
        std::env::set_var("WIXEN_MAIL_DATA", data.path());
        std::env::set_var("WIXEN_NO_AUDIO", "1");
    }
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let frame = Frame::builder()
                .with_title("A rule runs over a folder when asked, the reading")
                .build();
            let mut harvest = Harvest::new();
            frame.set_menu_bar(WxMailApp::build_menu_bar());
            harvest.insert(
                "This Folder's items",
                frame
                    .get_menu_bar()
                    .map(|bar| this_folders_items(&bar))
                    .unwrap_or_else(|| "no menu bar".to_string()),
            );
            frame.destroy();
            if let Ok(mut slot) = outcome.lock() {
                *slot = Some(Ok(harvest));
            }
            wxdragon::call_after(Box::new(move || app.exit_main_loop()));
        })
    };
    if let Err(why) = result {
        return Err(format!("wxdragon::main returned {why:?}"));
    }
    let taken = outcome
        .lock()
        .map_err(|_| "the harvest's lock was poisoned".to_string())?
        .take();
    taken.unwrap_or_else(|| Err("the window session ended without a harvest".to_string()))
}

fn reading(name: &str) -> &'static str {
    static HARVEST: OnceLock<Result<Harvest, String>> = OnceLock::new();
    let harvest = match HARVEST.get_or_init(take_the_harvest) {
        Ok(harvest) => harvest,
        Err(why) => panic!("the window session could not be read: {why}"),
    };
    harvest
        .get(name)
        .map(String::as_str)
        .unwrap_or_else(|| panic!("nothing was read for {name:?}"))
}

// ── The checks ────────────────────────────────────────────────────────────

#[test]
fn test_this_folder_offers_the_item_saying_no_rule_run_has_met_a_real_server() {
    let items = reading("This Folder's items");
    let Some(line) = items.lines().find(|line| line.starts_with(THE_ITEM)) else {
        panic!("This Folder holds no {THE_ITEM}:\n{items}");
    };
    assert!(
        line.ends_with(RUNNING_A_RULE_NOW_IS_EXPERIMENTAL),
        "the item's help does not end with the experimental sentence: {line}"
    );
}

#[test]
fn test_the_item_opens_the_door_once() {
    the_item_opens_the_door(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_door_has_a_rule_chosen_before_it_counts() {
    the_door_chooses_before_it_counts(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_count_is_taken_on_a_worker_and_sent_back() {
    the_count_is_taken_on_a_worker(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_counted_arm_hands_the_count_to_the_question() {
    the_counted_arm_asks(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_gate_is_met_before_the_question_is_built() {
    the_gate_is_met_before_the_question(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_question_is_answered_before_the_one_runner_runs_once() {
    the_question_comes_before_the_run(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_nothing_on_the_way_to_the_runner_writes_mail_of_its_own() {
    nothing_writes_round_the_runner(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_rule_run_says_one_sentence_with_one_confirmed() {
    the_run_says_one_sentence(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

/// A window shaped as it should be, cut down to what the readings read.
const SHAPED: &str = r#"                        _ if id == ID_RUN_A_RULE_HERE => {
                            run_a_rule_on_this_folder(app, &message_cache, &frame);
                        }
                        _ if id == ID_CHECK_FOR_UPDATES => {}
fn run_a_rule_on_this_folder(
    app: AppHandles<'_>,
) {
    let Some(at) = wx_managers::choose_from_list(frame, title, &label, "&Count", &names, palette) else {
        return;
    };
    count_what_a_rule_would_change(app, &account, &folder, rule);
}
fn count_what_a_rule_would_change(
    app: AppHandles<'_>,
) {
    rt.spawn_blocking(move || {
        let would = what_a_rule_would_change(&rule, &here);
        say(UIUpdate::ARuleWasCounted(Box::new(counted)));
    });
}
        UIUpdate::ARuleWasCounted(counted) => {
            ask_then_run_the_counted_rule(app, message_cache, msg_list, frame, a11y, counted);
        }
        UIUpdate::SavedSearchRan { messages, said } => {}
fn ask_then_run_the_counted_rule(
    app: AppHandles<'_>,
) {
    if let Err(why) = crate::service::outward::permitted(allowed, "change these messages") {
        return send_refusal(tx, rt, &why.to_string());
    }
    let answered = MessageDialog::builder(frame, &question.text, "Run a Rule")
        .with_style(style)
        .build()
        .show_modal();
    if which_of_the_two(answered) != Answered::Yes {
        return;
    }
    let done = match run_these_actions_over(app, list, cache, &set, &outcome) {
        Ok(done) => done,
        Err(why) => return send_refusal(tx, rt, &why),
    };
    let said = what_the_rule_did(&rule_name, &acting_on_a_set::said(&set, &done));
    let _ = a11y.announce(&said, Priority::Normal);
    send_shown(tx, rt, &said);
    let _ = a11y.signal(FeedbackEvent::Confirmed, &rule_name);
}
"#;

fn planted(from: &str, to: &str) -> String {
    let planted = SHAPED.replacen(from, to, 1);
    assert_ne!(planted, SHAPED, "the companion lost its anchor: {from}");
    planted
}

#[test]
fn test_the_readings_pass_a_window_shaped_as_it_should_be() {
    for reading in [
        the_item_opens_the_door,
        the_door_chooses_before_it_counts,
        the_count_is_taken_on_a_worker,
        the_counted_arm_asks,
        the_gate_is_met_before_the_question,
        the_question_comes_before_the_run,
        nothing_writes_round_the_runner,
        the_run_says_one_sentence,
    ] {
        reading(SHAPED).unwrap_or_else(|why| panic!("{why}"));
    }
}

#[test]
fn test_companion_a_run_before_the_question_is_refused() {
    let unasked = planted(
        "    let answered = MessageDialog::builder(",
        "    let _ = run_these_actions_over(app, list, cache, &set, &outcome);\n    let answered = MessageDialog::builder(",
    );

    let said = the_question_comes_before_the_run(&unasked).expect_err("run before asking");

    assert!(said.contains("after run_these_actions_over("), "{said}");
}

#[test]
fn test_companion_a_gate_met_after_the_question_is_refused() {
    let late = planted(
        "    if let Err(why) = crate::service::outward::permitted(allowed, \"change these messages\") {\n        return send_refusal(tx, rt, &why.to_string());\n    }\n",
        "",
    )
    .replacen(
        "    if which_of_the_two(",
        "    if let Err(why) = crate::service::outward::permitted(allowed, \"change these messages\") {\n        return send_refusal(tx, rt, &why.to_string());\n    }\n    if which_of_the_two(",
        1,
    );

    let said = the_gate_is_met_before_the_question(&late).expect_err("gate after the question");

    assert!(
        said.contains("calls permitted( after MessageDialog::builder("),
        "{said}"
    );
}

#[test]
fn test_companion_a_door_that_skips_the_count_is_refused() {
    let uncounted = planted(
        "    count_what_a_rule_would_change(app, &account, &folder, rule);\n",
        "    let _ = run_these_actions_over(app, list, cache, &set, &outcome);\n",
    );

    let said = the_door_chooses_before_it_counts(&uncounted).expect_err("no count");
    assert!(
        said.contains("never calls count_what_a_rule_would_change("),
        "{said}"
    );
    let said = nothing_writes_round_the_runner(&uncounted).expect_err("runner in the door");
    assert!(said.contains("before anybody was asked"), "{said}");
}

#[test]
fn test_companion_a_count_on_the_window_thread_is_refused() {
    let on_the_window = planted("    rt.spawn_blocking(move || {\n", "    {\n");

    let said = the_count_is_taken_on_a_worker(&on_the_window).expect_err("no worker");

    assert!(said.contains("never calls spawn_blocking("), "{said}");
}

#[test]
fn test_companion_a_question_that_writes_round_the_runner_is_refused() {
    let round = planted(
        "    let said = what_the_rule_did(",
        "    move_these(app, list, cache, moving, into, false);\n    let said = what_the_rule_did(",
    );

    let said = nothing_writes_round_the_runner(&round).expect_err("a write of its own");

    assert!(said.contains("move_these"), "{said}");
}

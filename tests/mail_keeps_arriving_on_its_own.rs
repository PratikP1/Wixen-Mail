//! Mail keeps arriving on its own for as long as the program runs: a watch
//! that ends is started again after a growing wait, the network coming back
//! starts one at once, every enabled account has one, the accounts are
//! checked on their own interval where a watch cannot cover, a start checks
//! without a keystroke, and nothing says that new mail will not appear on
//! its own (#37, MAIL-04).
//!
//! # What the tester saw, and what the tree said
//!
//! "After running a few hours, automatic mail fetching is switched off. The
//! user has to manually fetch mail." Until 2026-09-18 the inbox watch was
//! one, on one account, started only when a check for mail finished, and
//! ended for good by anything but mail arriving, with a comment saying so
//! and a sentence telling the person to use Refresh. Two things the issue
//! did not say: nothing checked at startup, so nothing watched until the
//! first F9; and the account editor had offered a Check Interval since it
//! was written that nothing read.
//!
//! # Why this lives here rather than beside the code
//!
//! The watch, the timer and the arms are in `src/presentation/wx_app.rs`,
//! which needs a window, a frame and a running event loop to reach, so this
//! is a source read. It is an integration target rather than a test inside
//! that file for a cost reason: sixty-six guard records fingerprinted the
//! number of tests in `wx_app.rs` when this was written (by the TOML reader
//! on 2026-09-18), so one test added there is that many builds and that
//! many full library runs at the next commit. Records name this file
//! instead, with `suite` naming the target, so `check.sh --suites-for`
//! couples it to `wx_app.rs` and it runs on the commits that could break it.
//!
//! # What this cannot see
//!
//! It reads source. It says the window is written to ask
//! `application::checking_on_a_schedule` what to do when a watch ends, to
//! start the watches whose wait is over from the timer, to check the due
//! accounts on the interval the account row carries, and to check at
//! startup. It does not say a provider drops a watch, after how long, or
//! that the restart brings mail in over hours: that is the tester's account
//! to show, ledger 64 and 65. The decisions themselves are answered in
//! `application::checking_on_a_schedule` and `application::trying_again`,
//! where they run without a window.

use std::fs;

use wixen_mail::common::what_ships::what_ships;

/// The window, which is where the watch, the timer and the arms live.
const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

/// The function that watches one account's inbox.
const THE_WATCH: &str = "fn spawn_mail_watch(";

/// The function that decides what a watch that ended does next.
const WHAT_NEXT: &str = "fn what_the_watch_does_next(";

/// The function that checks the accounts the schedule says are due.
const THE_SCHEDULE: &str = "fn check_the_accounts_that_are_due(";

/// The function that starts the watches whose wait has run out.
const THE_WAIT_SERVER: &str = "fn start_the_watches_whose_wait_is_over(";

/// The function that acts on the network coming or going.
const THE_NETWORK: &str = "fn act_on_what_the_network_did(";

/// The function that asks for a watch on every enabled IMAP account.
const WATCH_THEM_ALL: &str = "fn watch_every_enabled_account(";

/// The function that checks every enabled account, which F9 and a start
/// both reach.
const CHECK_THEM_ALL: &str = "fn check_every_enabled_account(";

/// The function that checks for mail, whose end asks for the watch.
const THE_CHECK: &str = "fn spawn_mail_sync(";

/// The sentence the window said when a watch ended, until 2026-09-18.
const THE_SENTENCE_THAT_IS_GONE: &str = "will not appear on its own";

/// The function that said it, gone with it.
const THE_FUNCTION_THAT_IS_GONE: &str = "say_the_watch_is_off";

/// The shipping half of the main window.
fn the_window_itself() -> String {
    what_ships(&fs::read_to_string(THE_MAIN_WINDOW).expect("the main window's source"))
}

/// The body of the item that starts with `starts`, up to the next item at the
/// left margin, or nothing when the file holds no such item.
///
/// Anchored on a line at column nought, because every item this asks about is
/// one. A body read to the end of the file would pick up whatever came after
/// it and pass on its neighbour's words, which is the failure a source read
/// makes rather than one it finds.
fn the_item_starting_with(source: &str, starts: &str) -> Option<String> {
    let mut lines = source.lines().skip_while(|line| !line.starts_with(starts));
    let first = lines.next()?;
    let rest: Vec<&str> = lines
        .take_while(|line| {
            line.is_empty()
                || line.starts_with(' ')
                || line.starts_with('}')
                || line.starts_with(')')
        })
        .collect();
    Some(format!("{first}\n{}", rest.join("\n")))
}

/// The arm of the menu handler that answers for `id`, up to the next arm.
fn the_menu_arm_for(source: &str, id: &str) -> Option<String> {
    let opens = format!("_ if id == {id} => {{");
    let at = source.find(&opens)?;
    let rest = &source[at + opens.len()..];
    let ends = rest.find("_ if id ==").unwrap_or(rest.len());
    Some(rest[..ends].to_string())
}

/// The text between two markers, or nothing when either is missing.
fn between<'a>(source: &'a str, from: &str, to: &str) -> Option<&'a str> {
    let start = source.find(from)?;
    let end = source[start..].find(to)? + start;
    Some(&source[start..end])
}

// ── A watch that ends asks what to do next ───────────────────────────────────

/// What is wrong with a watch that ended, as sentences; nothing when the
/// `Stopped` arm hands the reason to the decision and the decision asks the
/// rule.
///
/// The arm is held to naming `WhyTheWatchEnded::Ended(`, because that is the
/// reason leaving the arm for the rule; the function the arm reaches is held
/// to `whether_to_watch_again(`. An arm that logs and breaks, which is the
/// tree as it stood until 2026-09-18, is the defect.
fn what_is_wrong_with_a_watch_that_ended(watch: Option<&str>, next: Option<&str>) -> Vec<String> {
    let mut wrong = Vec::new();
    let Some(watch) = watch else {
        wrong.push("the window no longer watches an inbox at all".into());
        return wrong;
    };
    let stopped = watch
        .find("ImapIdleEvent::Stopped")
        .map(|at| &watch[at..])
        .map(|rest| {
            let ends = rest.find("\n                }").unwrap_or(rest.len());
            &rest[..ends]
        });
    match stopped {
        None => {
            wrong.push("the watch no longer handles Stopped, so an ended watch is nobody's".into())
        }
        Some(arm) if !arm.contains("WhyTheWatchEnded::Ended(") => wrong.push(
            "the Stopped arm does not hand the reason to the decision, so a watch that ends is \
             logged and never started again"
                .into(),
        ),
        Some(_) => {}
    }
    match next {
        None => wrong.push("nothing decides what a watch that ended does next".into()),
        Some(next) if !next.contains("whether_to_watch_again(") => wrong.push(
            "the decision does not ask whether_to_watch_again, so it decides something of its \
             own that nothing tests"
                .into(),
        ),
        Some(next) if !next.contains(".next_wait()") || !next.contains("next_watch_at = Some(") => {
            wrong.push(
                "a watch tried again is not given the wait rule's wait, so a refusing server is \
                 asked again at once"
                    .into(),
            );
        }
        Some(_) => {}
    }
    wrong
}

#[test]
fn test_a_watch_that_ends_asks_whether_to_watch_again() {
    // The connection dropping, the server closing it, DONE failing on a dead
    // socket: every way a watch ends reaches one arm, and that arm hands the
    // reason to a decision tested in milliseconds rather than breaking.
    let window = the_window_itself();
    let wrong = what_is_wrong_with_a_watch_that_ended(
        the_item_starting_with(&window, THE_WATCH).as_deref(),
        the_item_starting_with(&window, WHAT_NEXT).as_deref(),
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_a_watch_that_only_broke() {
    // Proving the reading before believing it: the arm as it stood until
    // 2026-09-18, and a decision that never asks the rule.
    let the_old_arm = "fn spawn_mail_watch(app: AppHandles<'_>) {\n\
                       \x20                   crate::service::protocols::imap::ImapIdleEvent::Stopped { folder, reason } => {\n\
                       \x20                       tracing::info!(\"Stopped watching {}: {}\", folder, reason);\n\
                       \x20                       say_the_watch_is_off(&tx);\n\
                       \x20                       break;\n\
                       \x20               }\n";
    let a_decision = "fn what_the_watch_does_next() {\n    whether_to_watch_again(&why, failures);\n    \
                      let wait = entry.wait.next_wait();\n    entry.next_watch_at = Some(now + wait);\n}\n";
    let wrong = what_is_wrong_with_a_watch_that_ended(Some(the_old_arm), Some(a_decision));
    assert!(
        wrong.iter().any(|w| w.contains("does not hand the reason")),
        "the reading did not see the arm that only breaks: {wrong:?}"
    );

    let the_arm_that_asks = "fn spawn_mail_watch(app: AppHandles<'_>) {\n\
                             \x20                   ImapIdleEvent::Stopped { folder, reason } => {\n\
                             \x20                       what_the_watch_does_next(WhyTheWatchEnded::Ended(reason));\n\
                             \x20                       break;\n\
                             \x20               }\n";
    let a_decision_of_its_own = "fn what_the_watch_does_next() {\n    if reason.contains(\"lost\") {\n        \
                                 entry.next_watch_at = Some(now + Duration::from_secs(1));\n    }\n}\n";
    let wrong =
        what_is_wrong_with_a_watch_that_ended(Some(the_arm_that_asks), Some(a_decision_of_its_own));
    assert!(
        wrong
            .iter()
            .any(|w| w.contains("does not ask whether_to_watch_again")),
        "the reading did not see a decision of the window's own: {wrong:?}"
    );

    let wrong = what_is_wrong_with_a_watch_that_ended(Some(the_arm_that_asks), Some(a_decision));
    assert!(
        wrong.is_empty(),
        "the reading complained about a watch that asks: {wrong:?}"
    );
}

// ── A watch that never started asks too, before it returns ───────────────────

/// What is wrong with the start-failure branch, as sentences; nothing when
/// the `Err` from `watch_folder` reaches the decision as `NeverStarted`
/// before the branch returns.
///
/// This is the branch the tree had until 2026-09-18: it logged, said the
/// watch was off, and returned, and never reached `Stopped`, so a server
/// that could not be reached at startup was never tried again.
fn what_is_wrong_with_a_watch_that_never_started(watch: Option<&str>) -> Vec<String> {
    let mut wrong = Vec::new();
    let Some(watch) = watch else {
        wrong.push("the window no longer watches an inbox at all".into());
        return wrong;
    };
    let Some(after_the_call) = watch.find("watch_folder(").map(|at| &watch[at..]) else {
        wrong.push(
            "the watch no longer calls watch_folder, so this reads nothing about its start".into(),
        );
        return wrong;
    };
    let Some(branch) = after_the_call
        .find("Err(e) => {")
        .map(|at| &after_the_call[at..])
        .map(|rest| {
            let ends = rest.find("\n            }").unwrap_or(rest.len());
            &rest[..ends]
        })
    else {
        wrong.push(
            "the start of the watch has no failure branch, so a refused start is unhandled".into(),
        );
        return wrong;
    };
    // The statement, with its semicolon: the word alone is in the comment
    // that says the question comes before it.
    match (
        branch.find("WhyTheWatchEnded::NeverStarted("),
        branch.find("return;"),
    ) {
        (None, _) => wrong.push(
            "the start-failure branch returns without asking whether to watch again, so a \
             server that could not be reached is never tried again"
                .into(),
        ),
        (Some(asks), Some(returns)) if returns < asks => wrong.push(
            "the start-failure branch returns before it asks, so the question is never reached"
                .into(),
        ),
        _ => {}
    }
    wrong
}

#[test]
fn test_a_watch_that_never_started_asks_too_before_it_returns() {
    let window = the_window_itself();
    let wrong = what_is_wrong_with_a_watch_that_never_started(
        the_item_starting_with(&window, THE_WATCH).as_deref(),
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_a_start_failure_that_only_returned() {
    let the_old_branch = "fn spawn_mail_watch(app: AppHandles<'_>) {\n\
                          \x20       let watching = handle.block_on(crate::application::mail_sync::watch_folder(\n\
                          \x20       ));\n\
                          \x20       let (mut events, watch) = match watching {\n\
                          \x20           Ok(watching) => watching,\n\
                          \x20           Err(e) => {\n\
                          \x20               tracing::warn!(\"Could not watch the inbox: {}\", e);\n\
                          \x20               say_the_watch_is_off(&tx);\n\
                          \x20               return;\n\
                          \x20           }\n\
                          \x20       };\n";
    let wrong = what_is_wrong_with_a_watch_that_never_started(Some(the_old_branch));
    assert!(
        wrong.iter().any(|w| w.contains("returns without asking")),
        "the reading did not see the branch that only returns: {wrong:?}"
    );

    let the_branch_that_asks = "fn spawn_mail_watch(app: AppHandles<'_>) {\n\
                                \x20       let watching = handle.block_on(watch_folder(&account));\n\
                                \x20       let (mut events, watch) = match watching {\n\
                                \x20           Ok(watching) => watching,\n\
                                \x20           Err(e) => {\n\
                                \x20               what_the_watch_does_next(WhyTheWatchEnded::NeverStarted(e.to_string()));\n\
                                \x20               return;\n\
                                \x20           }\n\
                                \x20       };\n";
    let wrong = what_is_wrong_with_a_watch_that_never_started(Some(the_branch_that_asks));
    assert!(
        wrong.is_empty(),
        "the reading complained about a branch that asks: {wrong:?}"
    );
}

// ── Nothing says new mail will not appear on its own ─────────────────────────

/// What is wrong with what the window says about the watch, as sentences;
/// nothing when neither the sentence nor the function that said it is left.
fn what_is_wrong_with_what_is_said_about_the_watch(window: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    if window.contains(THE_SENTENCE_THAT_IS_GONE) {
        wrong.push(format!(
            "the window still says new mail {THE_SENTENCE_THAT_IS_GONE}, which stopped being \
             true when the watch started again"
        ));
    }
    if window.contains(THE_FUNCTION_THAT_IS_GONE) {
        wrong.push(format!(
            "{THE_FUNCTION_THAT_IS_GONE} is still in the window, so something can still tell a \
             person the watch is off for good"
        ));
    }
    wrong
}

#[test]
fn test_nothing_says_new_mail_will_not_appear_on_its_own() {
    // The sentence was true of a watch nothing restarted. Said of one that
    // is tried again after a wait, it sends somebody to Refresh for mail
    // that is about to arrive on its own.
    let wrong = what_is_wrong_with_what_is_said_about_the_watch(&the_window_itself());
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_the_sentence_back() {
    let with_the_sentence = "fn say_the_watch_is_off(tx: &Sender<UIUpdate>) {\n    \
                             \"New mail will not appear on its own. Use Refresh to check for it.\"\n}\n";
    let wrong = what_is_wrong_with_what_is_said_about_the_watch(with_the_sentence);
    assert_eq!(
        wrong.len(),
        2,
        "the reading did not see both halves: {wrong:?}"
    );

    let wrong = what_is_wrong_with_what_is_said_about_the_watch("fn spawn_mail_watch() {}\n");
    assert!(
        wrong.is_empty(),
        "the reading complained about a clean window: {wrong:?}"
    );
}

// ── The network coming back starts the watches at once ───────────────────────

/// What is wrong with what the network coming back does, as sentences;
/// nothing when the back branch asks for a watch on every enabled IMAP
/// account and a check of every due one, and the asking sends the request.
fn what_is_wrong_with_the_network_coming_back(
    network: Option<&str>,
    watch_them_all: Option<&str>,
) -> Vec<String> {
    let mut wrong = Vec::new();
    let Some(network) = network else {
        wrong.push("the window no longer acts on the network at all".into());
        return wrong;
    };
    let Some(back) = network.find("offer_the_way_back").map(|at| &network[at..]) else {
        wrong.push(
            "the network coming back is no longer a branch, so nothing restarts on it".into(),
        );
        return wrong;
    };
    if !back.contains("watch_every_enabled_account(") {
        wrong.push(
            "the network coming back does not ask for the watches, so an account whose watch \
             died with the network waits out the whole of its wait"
                .into(),
        );
    }
    if !back.contains("check_the_accounts_that_are_due(") {
        wrong.push(
            "the network coming back does not check the due accounts, so mail that arrived \
             while it was gone waits for the schedule"
                .into(),
        );
    }
    match watch_them_all {
        None => wrong.push("nothing asks for a watch on every enabled account".into()),
        Some(all) if !all.contains("UIUpdate::MailboxWatchRequested(") => wrong.push(
            "asking for the watches sends no MailboxWatchRequested, so nothing is started".into(),
        ),
        Some(_) => {}
    }
    wrong
}

#[test]
fn test_the_network_coming_back_starts_the_watches_at_once() {
    // Reads follow the network; sends follow offline mode. A watch that
    // died with the network is not made to wait out a wait that was about
    // the network, and the check reaches mail that arrived meanwhile.
    let window = the_window_itself();
    let wrong = what_is_wrong_with_the_network_coming_back(
        the_item_starting_with(&window, THE_NETWORK).as_deref(),
        the_item_starting_with(&window, WATCH_THEM_ALL).as_deref(),
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_a_network_that_came_back_to_nothing() {
    let the_old_branch = "fn act_on_what_the_network_did(news, tx, rt) {\n    \
                          let offer_the_way_back = news == WhatToDoAboutIt::OfferToGoBackOnline;\n    \
                          rt.spawn(async move {\n        if offer_the_way_back {\n            \
                          let _ = tx.send(UIUpdate::TheNetworkIsBack).await;\n        }\n    });\n}\n";
    let asks = "fn watch_every_enabled_account(app: AppHandles<'_>) {\n    \
                let _ = tx.try_send(UIUpdate::MailboxWatchRequested(id));\n}\n";
    let wrong = what_is_wrong_with_the_network_coming_back(Some(the_old_branch), Some(asks));
    assert!(
        wrong
            .iter()
            .any(|w| w.contains("does not ask for the watches"))
            && wrong
                .iter()
                .any(|w| w.contains("does not check the due accounts")),
        "the reading did not see the branch that restarts nothing: {wrong:?}"
    );

    let the_branch_that_restarts = "fn act_on_what_the_network_did(news, app) {\n    \
                                    let offer_the_way_back = news == WhatToDoAboutIt::OfferToGoBackOnline;\n    \
                                    if offer_the_way_back {\n        watch_every_enabled_account(app);\n        \
                                    check_the_accounts_that_are_due(app, true);\n    }\n}\n";
    let asks_nothing =
        "fn watch_every_enabled_account(app: AppHandles<'_>) {\n    let _ = app;\n}\n";
    let wrong = what_is_wrong_with_the_network_coming_back(
        Some(the_branch_that_restarts),
        Some(asks_nothing),
    );
    assert!(
        wrong
            .iter()
            .any(|w| w.contains("sends no MailboxWatchRequested")),
        "the reading did not see the asking that sends nothing: {wrong:?}"
    );

    let wrong =
        what_is_wrong_with_the_network_coming_back(Some(the_branch_that_restarts), Some(asks));
    assert!(
        wrong.is_empty(),
        "the reading complained about a branch that restarts: {wrong:?}"
    );
}

// ── The timer checks on the account's own interval ───────────────────────────

/// What is wrong with the schedule, as sentences; nothing when the timer's
/// body reaches the schedule and the wait server, the schedule asks
/// `which_are_due` over accounts built from the rows by `From`, and the
/// wait server starts a watch.
///
/// `AccountToCheck::from` is what holds the interval to the row: an
/// account built by hand with a number in it is the field stopping to act
/// again, silently, as it did from the day it was written. Matched without
/// its bracket, because clippy has a mapped conversion written as the path
/// and not as a closure around a call.
fn what_is_wrong_with_the_schedule(
    timer: Option<&str>,
    schedule: Option<&str>,
    wait_server: Option<&str>,
) -> Vec<String> {
    let mut wrong = Vec::new();
    match timer {
        None => wrong.push("the timer's body could not be found, so this reads nothing".into()),
        Some(timer) => {
            if !timer.contains("check_the_accounts_that_are_due(") {
                wrong.push(
                    "the timer never asks which accounts are due, so nothing is checked on a \
                     schedule and a server without IDLE is never checked at all"
                        .into(),
                );
            }
            if !timer.contains("start_the_watches_whose_wait_is_over(") {
                wrong.push(
                    "the timer never starts a watch whose wait is over, so a wait is a stop".into(),
                );
            }
        }
    }
    match schedule {
        None => wrong.push("nothing checks the accounts that are due".into()),
        Some(schedule) => {
            if !schedule.contains("which_are_due(") {
                wrong.push(
                    "the schedule does not ask which_are_due, so it decides something of its own \
                     that nothing tests"
                        .into(),
                );
            }
            if !schedule.contains("AccountToCheck::from") {
                wrong.push(
                    "the accounts handed to the schedule are not built from the rows, so the \
                     Check Interval on the account editor can stop acting again without a test \
                     going red"
                        .into(),
                );
            }
        }
    }
    match wait_server {
        None => wrong.push("nothing starts a watch whose wait is over".into()),
        Some(server) if !server.contains("spawn_mail_watch(") => {
            wrong.push("the wait server clears the wait and starts no watch".into());
        }
        Some(_) => {}
    }
    wrong
}

/// The tail of the timer's body: from the network look to the question about
/// gone folders, which is where every periodic thing on the window runs.
fn the_timers_tail(window: &str) -> Option<&str> {
    between(
        window,
        "asked_the_network_at.get().elapsed()",
        "ask_about_the_folders_that_have_gone(",
    )
}

#[test]
fn test_the_timer_checks_on_the_accounts_own_interval() {
    // The main timer, beside the network look and the download's wait,
    // because a timer event reaches every handler on the window and a
    // second timer would run this one as well.
    let window = the_window_itself();
    let wrong = what_is_wrong_with_the_schedule(
        the_timers_tail(&window),
        the_item_starting_with(&window, THE_SCHEDULE).as_deref(),
        the_item_starting_with(&window, THE_WAIT_SERVER).as_deref(),
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_a_timer_with_no_schedule() {
    let the_old_tail = "asked_the_network_at.get().elapsed() >= HOW_OFTEN_TO_ASK_ABOUT_THE_NETWORK {\n        \
                        start_the_download_if_its_wait_is_over(app);\n    }\n";
    let a_schedule = "fn check_the_accounts_that_are_due(app: AppHandles<'_>) {\n    \
                      let to_check: Vec<AccountToCheck> = s.accounts.iter().map(AccountToCheck::from).collect();\n    \
                      let due = which_are_due(&to_check, &s.last_checked, now);\n}\n";
    let a_wait_server = "fn start_the_watches_whose_wait_is_over(app: AppHandles<'_>) {\n    \
                         spawn_mail_watch(app, &id);\n}\n";
    let wrong =
        what_is_wrong_with_the_schedule(Some(the_old_tail), Some(a_schedule), Some(a_wait_server));
    assert!(
        wrong
            .iter()
            .any(|w| w.contains("never asks which accounts are due")),
        "the reading did not see the timer with no schedule: {wrong:?}"
    );

    let the_tail_that_asks = "asked_the_network_at.get().elapsed() >= HOW_OFTEN_TO_ASK_ABOUT_THE_NETWORK {\n        \
                              start_the_download_if_its_wait_is_over(app);\n        \
                              start_the_watches_whose_wait_is_over(app);\n    }\n    \
                              check_the_accounts_that_are_due(app, there_is_a_network);\n";
    let five_minutes_for_everybody = "fn check_the_accounts_that_are_due(app: AppHandles<'_>) {\n    \
                                      let to_check: Vec<AccountToCheck> = s.accounts.iter().map(|a| AccountToCheck {\n        \
                                      id: a.id.clone(), enabled: a.enabled, protocol: a.protocol(), check_interval_minutes: 5,\n    \
                                      }).collect();\n    let due = which_are_due(&to_check, &s.last_checked, now);\n}\n";
    let wrong = what_is_wrong_with_the_schedule(
        Some(the_tail_that_asks),
        Some(five_minutes_for_everybody),
        Some(a_wait_server),
    );
    assert!(
        wrong.iter().any(|w| w.contains("not built from the rows")),
        "the reading did not see the fixed five minutes: {wrong:?}"
    );

    let wrong = what_is_wrong_with_the_schedule(
        Some(the_tail_that_asks),
        Some(a_schedule),
        Some(a_wait_server),
    );
    assert!(
        wrong.is_empty(),
        "the reading complained about a timer that schedules: {wrong:?}"
    );
}

// ── A start checks without a keystroke ───────────────────────────────────────

/// What is wrong with the start, as sentences; nothing when the startup
/// section, after the module is filled, checks every enabled account and
/// asks for a watch on every enabled IMAP one.
fn what_is_wrong_with_the_start(startup: Option<&str>) -> Vec<String> {
    let mut wrong = Vec::new();
    let Some(startup) = startup else {
        wrong.push("the startup section could not be found, so this reads nothing".into());
        return wrong;
    };
    let Some(after_the_fill) = startup.find("load_module_data(").map(|at| &startup[at..]) else {
        wrong.push(
            "the start no longer fills the module, so this reads nothing about what follows".into(),
        );
        return wrong;
    };
    if !after_the_fill.contains("check_every_enabled_account(") {
        wrong.push(
            "the start checks nothing, so the first hours after opening the program are hours \
             with no watch and no check until somebody presses F9"
                .into(),
        );
    }
    if !after_the_fill.contains("watch_every_enabled_account(") {
        wrong.push(
            "the start asks for no watch, so an account whose first check fails is not watched \
             until a check works"
                .into(),
        );
    }
    wrong
}

/// The startup section: from the frame being shown to the event loop's
/// result being read, as `tests/the_numbers_the_targets_ask_for.rs` reads
/// it.
fn the_startup_section(window: &str) -> Option<&str> {
    between(
        window,
        "Main frame shown, entering event loop",
        "wxdragon::main blocks until the window is closed",
    )
}

#[test]
fn test_a_start_checks_without_a_keystroke() {
    // After the fill, so the list somebody opens on is the cached one and
    // the check's lines follow it rather than racing it.
    let wrong = what_is_wrong_with_the_start(the_startup_section(&the_window_itself()));
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_a_start_that_only_filled() {
    let the_old_start = "Main frame shown, entering event loop\n    \
                         load_module_data(module, &message_cache, account_id, &scan_tx, showing);\n    \
                         ask_about_the_alpha_once(&frame, &a11y);\n";
    let wrong = what_is_wrong_with_the_start(Some(the_old_start));
    assert!(
        wrong.iter().any(|w| w.contains("the start checks nothing")),
        "the reading did not see the start that only filled: {wrong:?}"
    );

    let the_start_that_checks = "Main frame shown, entering event loop\n    \
                                 load_module_data(module, &message_cache, account_id, &scan_tx, showing);\n    \
                                 watch_every_enabled_account(app);\n    check_every_enabled_account(app);\n";
    let wrong = what_is_wrong_with_the_start(Some(the_start_that_checks));
    assert!(
        wrong.is_empty(),
        "the reading complained about a start that checks: {wrong:?}"
    );
}

// ── The check walks every enabled account and writes down when ───────────────

/// What is wrong with the check, as sentences; nothing when F9 reaches the
/// check of every enabled account, that check says how many and walks the
/// enabled ones, the check itself walks the accounts it is handed, ends each
/// with its own watch request, and writes down when the account was checked.
fn what_is_wrong_with_the_check(
    f9: Option<&str>,
    check_them_all: Option<&str>,
    check: Option<&str>,
) -> Vec<String> {
    let mut wrong = Vec::new();
    match f9 {
        None => wrong.push("F9 no longer has an arm".into()),
        Some(arm) if !arm.contains("check_every_enabled_account(") => {
            wrong.push("F9 checks one account, so a person with two hears about one".into());
        }
        Some(_) => {}
    }
    match check_them_all {
        None => wrong.push("nothing checks every enabled account".into()),
        Some(all) => {
            if !all.contains("every_enabled_account(") {
                wrong.push("the check of every account does not read which are enabled".into());
            }
            if !all.contains("what_a_check_of_them_all_says(") {
                wrong.push("the check of every account does not say how many it checks".into());
            }
        }
    }
    match check {
        None => wrong.push("the window no longer checks for mail at all".into()),
        Some(check) => {
            // With or without `mut`: the check marks each account synced
            // on its own copy, and the walk is the point.
            if !check.contains(" account in accounts {") {
                wrong.push("the check does not walk the accounts it is handed".into());
            }
            if !check.contains("say(UIUpdate::MailboxWatchRequested(account.id.clone()))") {
                wrong.push(
                    "the check does not end with a watch request naming the account it checked, \
                     so the watch is on whichever account is active"
                        .into(),
                );
            }
            if !check.contains("mark_synced()") || !check.contains("update_account_last_sync(") {
                wrong.push(
                    "the check does not write down when the account was checked, so the row's \
                     last_sync stays empty and mark_synced is called by its own test alone"
                        .into(),
                );
            }
            // Found by running the release binary against the measurement
            // profile, whose account has no password saved: the first draft
            // of the walk asked for the download after every check, so a
            // check in which nothing went through started a download that
            // failed the same way and waited on its own, two retry rows
            // against one server. 10-05's tree never asked for the download
            // after a failed check, and neither does this. The branch, not
            // the flag's name: the flag is declared before the walk, so a
            // reading of the name alone would pass a check that declared it
            // and asked nothing.
            match (
                check.find("if nothing_went_through {"),
                check.find("say(UIUpdate::DownloadRequested)"),
            ) {
                (_, None) => {}
                (Some(asked), Some(download)) if asked < download => {}
                _ => wrong.push(
                    "the check asks for the download whether or not any account went through, \
                     so a server that refused the check is asked again by the download and \
                     waited on twice"
                        .into(),
                ),
            }
        }
    }
    wrong
}

#[test]
fn test_the_check_for_mail_walks_every_enabled_account_and_writes_down_when() {
    let window = the_window_itself();
    let wrong = what_is_wrong_with_the_check(
        the_menu_arm_for(&window, "ID_CHECK_MAIL").as_deref(),
        the_item_starting_with(&window, CHECK_THEM_ALL).as_deref(),
        the_item_starting_with(&window, THE_CHECK).as_deref(),
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_a_check_of_one_account() {
    let the_old_f9 = "\n    send_progress(&ui_tx, &runtime, \"Checking for new mail...\");\n    \
                      spawn_mail_sync(app, None, WhatThisSyncIsFor::WhateverHasChanged);\n";
    let them_all = "fn check_every_enabled_account(app: AppHandles<'_>) {\n    \
                    let accounts = every_enabled_account(app.state);\n    \
                    send_progress(tx, rt, &what_a_check_of_them_all_says(accounts.len()));\n}\n";
    let the_old_check = "fn spawn_mail_sync(app: AppHandles<'_>, only: Option<String>) {\n    \
                         let account = accounts.first().cloned();\n    say(UIUpdate::MailboxWatchRequested);\n}\n";
    let wrong = what_is_wrong_with_the_check(Some(the_old_f9), Some(them_all), Some(the_old_check));
    assert!(
        wrong.iter().any(|w| w.contains("F9 checks one account"))
            && wrong
                .iter()
                .any(|w| w.contains("does not walk the accounts"))
            && wrong.iter().any(|w| w.contains("does not write down when")),
        "the reading did not see the check of one account: {wrong:?}"
    );

    let the_f9_that_walks = "\n    check_every_enabled_account(app);\n";
    let the_check_that_downloads_regardless = "fn spawn_mail_sync(app: AppHandles<'_>, accounts: Vec<Account>) {\n    \
         for account in accounts {\n        account.mark_synced();\n        \
         cache.update_account_last_sync(&account.id);\n        \
         say(UIUpdate::MailboxWatchRequested(account.id.clone()));\n    }\n    \
         say(UIUpdate::DownloadRequested);\n}\n";
    let wrong = what_is_wrong_with_the_check(
        Some(the_f9_that_walks),
        Some(them_all),
        Some(the_check_that_downloads_regardless),
    );
    assert!(
        wrong
            .iter()
            .any(|w| w.contains("whether or not any account went through")),
        "the reading did not see the download asked for after a check that failed: {wrong:?}"
    );

    let the_check_that_walks = "fn spawn_mail_sync(app: AppHandles<'_>, accounts: Vec<Account>) {\n    \
                                let mut nothing_went_through = true;\n    \
                                for account in accounts {\n        account.mark_synced();\n        \
                                cache.update_account_last_sync(&account.id);\n        \
                                say(UIUpdate::MailboxWatchRequested(account.id.clone()));\n        \
                                nothing_went_through = false;\n    }\n    \
                                if nothing_went_through {\n        return;\n    }\n    \
                                say(UIUpdate::DownloadRequested);\n}\n";
    let wrong = what_is_wrong_with_the_check(
        Some(the_f9_that_walks),
        Some(them_all),
        Some(the_check_that_walks),
    );
    assert!(
        wrong.is_empty(),
        "the reading complained about a check that walks: {wrong:?}"
    );
}

// ── An account removed or switched off stops being watched (finding #33) ─────

/// The function that answers the Account Manager closing.
const THE_ACCOUNT_MANAGER: &str = "fn handle_account_mgr(";

/// What is wrong with the Account Manager's handler about the watches of the
/// accounts it removed or switched off; nothing when it asks which stop, takes
/// each out of the list and stops it before anything is said to a server.
fn what_is_wrong_with_the_watches_it_leaves(manager: Option<&str>) -> Vec<String> {
    let Some(manager) = manager else {
        return vec!["the Account Manager has no handler".into()];
    };
    let mut wrong = Vec::new();
    if !manager.contains("which_watches_stop(") {
        wrong.push(
            "the Account Manager does not ask which watches stop, so an account it removed or \
             switched off goes on being watched and checked until the program closes"
                .into(),
        );
    }
    if !manager.contains("mail_watches.remove(") {
        wrong.push(
            "the Account Manager leaves the watch in the list, so the timer can start it again"
                .into(),
        );
    }
    match (manager.find(".stop().await"), manager.find("signing_off(")) {
        (None, _) => wrong.push("the Account Manager stops no watch".into()),
        (Some(stops), Some(signs_off)) if stops > signs_off => wrong
            .push("the watches are stopped after the sign-off, which is a round trip each".into()),
        _ => {}
    }
    wrong
}

#[test]
fn test_the_account_manager_stops_the_watch_of_an_account_it_removed_or_switched_off() {
    let window = the_window_itself();
    let wrong = what_is_wrong_with_the_watches_it_leaves(
        the_item_starting_with(&window, THE_ACCOUNT_MANAGER).as_deref(),
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_the_account_manager_leave_a_watch_running() {
    let today = "fn handle_account_mgr(frame: &Frame) {\n    \
                 s.accounts = new;\n    drop(s);\n    \
                 for id in removed {\n        signing_off(rt, no_longer_signed_in_to(&id));\n    }\n}\n";
    let stopping = "fn handle_account_mgr(frame: &Frame) {\n    \
                    let stopping = which_watches_stop(&accounts, &new);\n    \
                    let handles = stopping.iter().filter_map(|id| s.mail_watches.remove(id));\n    \
                    drop(s);\n    \
                    for handle in handles {\n        rt.spawn(async move { let _ = handle.stop().await; });\n    }\n    \
                    for id in removed {\n        signing_off(rt, no_longer_signed_in_to(&id));\n    }\n}\n";

    assert_eq!(
        what_is_wrong_with_the_watches_it_leaves(Some(today)).len(),
        3
    );
    assert!(what_is_wrong_with_the_watches_it_leaves(Some(stopping)).is_empty());
}

/// What is wrong with whom a watch starts for: nothing when the watch finds
/// its account by id and by whether it is switched on.
fn what_is_wrong_with_whom_a_watch_is_for(watch: Option<&str>) -> Vec<String> {
    let Some(watch) = watch else {
        return vec!["the window no longer watches an inbox at all".into()];
    };
    // By statement rather than by line, because rustfmt breaks a long chain
    // over several lines and a line reader would then see none of it.
    match watch
        .split(';')
        .find(|statement| statement.contains("accounts") && statement.contains(".find("))
    {
        None => vec!["the watch no longer finds its account in the window's list".into()],
        Some(statement) if !statement.contains(".enabled") => vec![
            "the watch finds its account by id alone, so the end of a check already running, or \
             a wait running out, watches an account that is switched off again"
                .into(),
        ],
        Some(_) => Vec::new(),
    }
}

#[test]
fn test_a_watch_is_started_only_for_an_account_that_is_switched_on() {
    let window = the_window_itself();
    let wrong = what_is_wrong_with_whom_a_watch_is_for(
        the_item_starting_with(&window, THE_WATCH).as_deref(),
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_a_watch_for_an_account_switched_off() {
    let today = "fn spawn_mail_watch(app: AppHandles<'_>, account_id: &str) {\n    \
                 let account = s.accounts.iter().find(|a| a.id == account_id).cloned();\n}\n";
    let switched_on = "fn spawn_mail_watch(app: AppHandles<'_>, account_id: &str) {\n    \
                       let account = s.accounts.iter().find(|a| a.id == account_id && a.enabled).cloned();\n}\n";

    assert_eq!(what_is_wrong_with_whom_a_watch_is_for(Some(today)).len(), 1);
    assert!(what_is_wrong_with_whom_a_watch_is_for(Some(switched_on)).is_empty());
}

/// The arm that answers a watch waking, up to the arm after it.
fn the_wake(source: &str) -> Option<&str> {
    between(
        source,
        "UIUpdate::MailboxChanged(folder) => {",
        "UIUpdate::FolderMessagesArrived(",
    )
}

/// What is wrong with the arm that answers a watch waking: nothing when it
/// keeps only a switched-on account with that id, and checks nothing when none
/// is left.
fn what_is_wrong_with_a_wake(wake: Option<&str>) -> Vec<String> {
    let Some(wake) = wake else {
        return vec!["nothing answers a watch waking".into()];
    };
    let mut wrong = Vec::new();
    if !wake.contains(".enabled") {
        wrong.push(
            "a watch waking checks its account whether or not it is switched on, so its rules run \
             and its waiting changes are sent"
                .into(),
        );
    }
    match (wake.find(".is_empty()"), wake.find("spawn_mail_sync(")) {
        (_, None) => wrong.push("a watch waking no longer checks anything".into()),
        (Some(empty), Some(check)) if empty < check => {}
        _ => wrong.push(
            "a watch waking for an account that is gone hands the check nothing, and the check \
             says to add an account"
                .into(),
        ),
    }
    wrong
}

#[test]
fn test_a_watch_waking_for_an_account_that_is_gone_checks_nothing() {
    let window = the_window_itself();
    let wrong = what_is_wrong_with_a_wake(the_wake(&window));
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_a_wake_that_checks_a_gone_account() {
    let today = "UIUpdate::MailboxChanged(folder) => {\n    \
                 let woken = accounts.filter(|account| account.id == folder.account_id);\n    \
                 spawn_mail_sync(app, woken);\n}\nUIUpdate::FolderMessagesArrived(id) => {}\n";
    let switched_on = "UIUpdate::MailboxChanged(folder) => {\n    \
                       let woken = accounts.filter(|account| account.id == folder.account_id && account.enabled);\n    \
                       if !woken.is_empty() {\n        spawn_mail_sync(app, woken);\n    }\n}\n\
                       UIUpdate::FolderMessagesArrived(id) => {}\n";

    assert_eq!(what_is_wrong_with_a_wake(the_wake(today)).len(), 2);
    assert!(what_is_wrong_with_a_wake(the_wake(switched_on)).is_empty());
}

// ── One check of an account at a time (finding #15) ──────────────────────────

/// What is wrong with how the check takes its account's turn, as sentences;
/// nothing when the per-account loop's first statement takes the turn, binds
/// the guard to a name so it is held to the end of the pass, and does so
/// before the POP branch and before the session.
///
/// By statement rather than by line, because rustfmt breaks the call over
/// several lines. `let _ =` drops the guard at once, which is a turn taken
/// and given back before the check starts, so it is refused.
fn what_is_wrong_with_the_turn(check: Option<&str>) -> Vec<String> {
    let Some(check) = check else {
        return vec!["the window no longer checks for mail at all".into()];
    };
    let Some(the_loop) = check.find(" account in accounts {").map(|at| &check[at..]) else {
        return vec!["the check no longer walks the accounts it is handed".into()];
    };
    let Some(turn) = the_loop.find("one_check_at_a_time(") else {
        return vec![
            "the check takes no turn on its account, so F9 pressed during a check runs a second \
             one beside it, which announces the same new mail twice and runs the rules twice"
                .into(),
        ];
    };
    let mut wrong = Vec::new();
    // The code before the call, comments left out: a comment saying the turn
    // is "let go" would otherwise pass for a binding.
    let before: Vec<&str> = the_loop[..turn]
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect();
    let joined = before.join("\n");
    let taken = joined.rsplit(';').next().unwrap_or_default();
    if !taken.contains("let ") || taken.contains("let _ =") {
        wrong.push(
            "the turn is not held by a name, so it is given back before the check starts".into(),
        );
    }
    if the_loop
        .find("Protocol::Pop3")
        .is_some_and(|pop| pop < turn)
    {
        wrong.push("a POP account is checked before its turn is taken".into());
    }
    if the_loop
        .find("the_session_at(")
        .is_none_or(|session| session < turn)
    {
        wrong.push("the session is opened before the account's turn is taken".into());
    }
    wrong
}

#[test]
fn test_a_check_takes_its_accounts_turn_before_its_session() {
    let wrong = what_is_wrong_with_the_turn(
        the_item_starting_with(&the_window_itself(), THE_CHECK).as_deref(),
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_a_check_that_takes_no_turn() {
    let today = "fn spawn_mail_sync(app: AppHandles<'_>, accounts: Vec<Account>) {\n    \
                 for mut account in accounts {\n        \
                 if account.protocol() == crate::common::types::Protocol::Pop3 {\n            \
                 check_pop_mail(&account, &handle, &tx);\n            continue;\n        }\n        \
                 let controller = match handle.block_on(the_session_at(&account)) {};\n    }\n}\n";
    let dropped_at_once = "fn spawn_mail_sync(app: AppHandles<'_>, accounts: Vec<Account>) {\n    \
                           for mut account in accounts {\n        \
                           let _ = handle.block_on(one_check_at_a_time(&account.id));\n        \
                           let controller = match handle.block_on(the_session_at(&account)) {};\n    }\n}\n";
    let after_the_pop_branch = "fn spawn_mail_sync(app: AppHandles<'_>, accounts: Vec<Account>) {\n    \
                                for mut account in accounts {\n        \
                                if account.protocol() == crate::common::types::Protocol::Pop3 {\n            \
                                continue;\n        }\n        \
                                let _turn = handle.block_on(one_check_at_a_time(&account.id));\n        \
                                let controller = match handle.block_on(the_session_at(&account)) {};\n    }\n}\n";
    let held = "fn spawn_mail_sync(app: AppHandles<'_>, accounts: Vec<Account>) {\n    \
                for mut account in accounts {\n        \
                let _turn = handle.block_on(\n            one_check_at_a_time(&account.id),\n        );\n        \
                if account.protocol() == crate::common::types::Protocol::Pop3 {\n            \
                continue;\n        }\n        \
                let controller = match handle.block_on(the_session_at(&account)) {};\n    }\n}\n";

    assert!(
        what_is_wrong_with_the_turn(Some(today))
            .iter()
            .any(|w| w.contains("takes no turn")),
        "the reading did not see a check that takes no turn"
    );
    assert!(
        what_is_wrong_with_the_turn(Some(dropped_at_once))
            .iter()
            .any(|w| w.contains("not held by a name")),
        "the reading did not see a turn given back at once"
    );
    assert!(
        what_is_wrong_with_the_turn(Some(after_the_pop_branch))
            .iter()
            .any(|w| w.contains("POP account")),
        "the reading did not see a POP account checked before its turn"
    );
    let wrong = what_is_wrong_with_the_turn(Some(held));
    assert!(
        wrong.is_empty(),
        "the reading complained about a check that holds its turn: {wrong:?}"
    );
}

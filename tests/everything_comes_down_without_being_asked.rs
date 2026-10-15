//! Everything comes down without being asked: the download of every kept
//! folder, with its text, is started by every check for mail, does what the
//! model says, stops between chunks when asked, waits after a refusal, and
//! is what the retired commands used to do.
//!
//! Until 2026-09-17 this target was `a_whole_folder_moves_both_bounds.rs`
//! and read the whole-folder request, Download This Whole Folder, which
//! asked once for one folder and then carried on by itself. 10-05 retired
//! that command and Fetch Missing Message Text for #20 and #23, because the
//! download that runs after every check is what both of them did, for every
//! enabled IMAP account and with the text. The readings that held the
//! request now hold the download; the two that held the arm a chunk reaches
//! and the arm a step reaches are as 10-02 and 10-04 left them, because the
//! download sends the same two kinds of update.
//!
//! # Why this lives here rather than beside the code
//!
//! The runner is in `src/presentation/wx_app.rs`, which needs a window, a
//! frame and a running event loop to reach, so this is a source read. It is
//! an integration target rather than a test inside that file for a cost
//! reason: when this was rewritten, sixty-two guard records fingerprinted the
//! number of tests in `wx_app.rs` (by the TOML reader on 2026-09-17), so one
//! test added there is that many builds and that many full library runs at
//! the next commit. Records name this file instead, with `suite` naming the
//! target, so `check.sh --suites-for` couples it to `wx_app.rs` and it runs
//! on the commits that could break it rather than only on the commits that
//! change it.
//!
//! # What this cannot see
//!
//! It reads source. It says the runner is written to ask the model, to ask
//! whether to stop, to wait after a refusal, and to be started by a check. It
//! does not say the runner is reached, that a provider tolerates it, or that
//! anybody hears a line. The decisions are answered in
//! `application::bringing_everything_down` and `application::trying_again`,
//! where they run without a window; the chunk of text is answered in
//! `application::mail_sync` against the scripted mailbox. What a provider does
//! is ledger 11 and 72, and what somebody hears is the listening page.

use std::fs;

use wixen_mail::common::what_ships::what_ships;

/// The window, which is where the download is driven from.
const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

/// The function that runs the download.
const THE_RUNNER: &str = "fn start_the_download";

/// The function that checks for mail, whose end starts the download.
const THE_CHECK: &str = "fn spawn_mail_sync(";

/// The module the retired command's loop lived in, which is gone with it.
const THE_RETIRED_MODULE: &str = "src/application/asking_for_a_whole_folder.rs";

/// The names of the retired whole-folder command, none of which the window
/// may carry now.
const THE_WORDS_OF_THE_RETIRED_COMMAND: [&str; 3] = [
    "ID_GET_WHOLE_FOLDER",
    "spawn_whole_folder_fetch",
    "until_the_whole_folder_is_here",
];

/// The shipping half of the main window.
fn the_window_itself() -> String {
    what_ships(&fs::read_to_string(THE_MAIN_WINDOW).expect("the main window's source"))
}

/// The body of the item that starts with `starts`, up to the next item at the
/// left margin.
///
/// Anchored on a line at column nought, because every item this asks about is
/// one. A body read to the end of the file would pick up whatever came after it
/// and pass on its neighbour's words, which is the failure a source read makes
/// rather than one it finds.
fn the_item_starting_with(source: &str, starts: &str) -> String {
    let mut lines = source.lines().skip_while(|line| !line.starts_with(starts));
    let first = lines.next().unwrap_or_else(|| {
        panic!("{THE_MAIN_WINDOW} holds nothing starting with {starts}");
    });
    let rest: Vec<&str> = lines
        .take_while(|line| {
            line.is_empty()
                || line.starts_with(' ')
                || line.starts_with('}')
                || line.starts_with(')')
        })
        .collect();
    format!("{first}\n{}", rest.join("\n"))
}

/// One arm of the update handler, up to the start of the next one.
///
/// Bounded at the next arm rather than at the end of the match, because an arm
/// read to the end picks up every arm below it and passes on their words. Every
/// arm in that match starts at the same indentation, which is what this counts
/// on.
fn the_arm_for(source: &str, variant: &str) -> String {
    let opens = format!("        UIUpdate::{variant}");
    let at = source
        .find(&opens)
        .unwrap_or_else(|| panic!("{THE_MAIN_WINDOW} has no arm for UIUpdate::{variant}"));
    let rest = &source[at + opens.len()..];
    let ends = rest.find("\n        UIUpdate::").unwrap_or(rest.len());
    rest[..ends].to_string()
}

/// The arm of the menu handler that answers for `id`, up to the next arm.
///
/// Bounded at the next `_ if id ==` for the reason `the_arm_for` gives: an
/// arm read to the end holds every arm below it.
fn the_menu_arm_for(source: &str, id: &str) -> String {
    let opens = format!("_ if id == {id} => {{");
    let at = source
        .find(&opens)
        .unwrap_or_else(|| panic!("{THE_MAIN_WINDOW} has no menu arm for {id}"));
    let rest = &source[at + opens.len()..];
    let ends = rest.find("_ if id ==").unwrap_or(rest.len());
    rest[..ends].to_string()
}

// ── Every check ends by starting the download ────────────────────────────────

/// What is wrong with the end of a check, as sentences; nothing when it asks
/// for the download after it asks for the watch.
///
/// The send is matched as a call, `say(UIUpdate::...)`, and not as the
/// variant's name: a name can sit in a comment or a `let _ =` and send
/// nothing, and a reading that accepted a mention would stay green through
/// exactly the break that matters. The watch request carries the account
/// it is for since 2026-09-18 (#37), so the call is matched up to its
/// opening bracket.
fn what_is_wrong_with_the_end_of_a_check(check: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    let Some(watch) = check.find("say(UIUpdate::MailboxWatchRequested(") else {
        wrong.push(
            "the check no longer asks for the watch, so this reads nothing about its end".into(),
        );
        return wrong;
    };
    match check.find("say(UIUpdate::DownloadRequested)") {
        None => wrong.push(
            "the check never asks for the download, so everything past the first chunk of each \
             folder waits for a key"
                .into(),
        ),
        Some(download) if download < watch => wrong.push(
            "the check asks for the download before the watch, so a download that refuses to \
             start leaves the inbox unwatched"
                .into(),
        ),
        Some(_) => {}
    }
    wrong
}

#[test]
fn test_every_check_ends_by_starting_the_download() {
    // F9, the watch waking and, after 10-06, the schedule all end in this
    // function, so a download asked for here is a download that runs after
    // every check, whoever started it.
    let check = the_item_starting_with(&the_window_itself(), THE_CHECK);
    let wrong = what_is_wrong_with_the_end_of_a_check(&check);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_a_check_that_forgot_the_download() {
    // Proving the reading before believing it: the old end of the check, and
    // one that asks the other way round.
    let the_old_end = "        say(UIUpdate::ConnectionStatusChanged(\n\
                       \x20           ConnectionStatus::Disconnected,\n\
                       \x20       ));\n\
                       \x20       say(UIUpdate::MailboxWatchRequested(account.id.clone()));\n\
                       \x20   });\n";
    let wrong = what_is_wrong_with_the_end_of_a_check(the_old_end);
    assert!(
        wrong
            .iter()
            .any(|w| w.contains("never asks for the download")),
        "the reading did not see the download missing: {wrong:?}"
    );

    let the_wrong_way_round = "        say(UIUpdate::DownloadRequested);\n\
                               \x20       say(UIUpdate::MailboxWatchRequested(account.id.clone()));\n";
    let wrong = what_is_wrong_with_the_end_of_a_check(the_wrong_way_round);
    assert!(
        wrong.iter().any(|w| w.contains("before the watch")),
        "the reading did not see the order reversed: {wrong:?}"
    );
}

// ── The download does what the model says and nothing of its own ─────────────

#[test]
fn test_the_download_does_what_the_model_says_and_nothing_of_its_own() {
    // Which folder next, how big a chunk, headers before text, when there is
    // nothing left: every one of those is decided in
    // `application::bringing_everything_down`, where it is tested in
    // milliseconds. The runner asks, does the one thing answered, and asks
    // again; a decision made here instead would be a decision with no test.
    let runner = the_item_starting_with(&the_window_itself(), THE_RUNNER);

    for expected in [
        "what_to_do_next(",
        "INITIAL_FETCH_LIMIT",
        "fetch_over_a_mailbox(",
        "MoreOfWhatIsAlreadyThere",
    ] {
        assert!(
            runner.contains(expected),
            "the download does not reach {expected}, so it decides or fetches something of \
             its own that nothing tests: {runner}"
        );
    }
}

// ── The download asks whether to stop between chunks ─────────────────────────

/// What is wrong with how the download stops, as sentences; nothing when one
/// flag is read in one closure, asked before each chunk of headers and handed
/// to each chunk of text.
fn what_is_wrong_with_the_stop(runner: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    if !runner.contains("downloading.paused") {
        wrong.push(
            "the download never reads whether it is paused, so Pause Downloading holds nothing"
                .into(),
        );
    }
    if !runner.contains("if stop()") {
        wrong.push(
            "the download does not ask whether to stop before a chunk of headers, so a pause \
             waits for the text pass"
                .into(),
        );
    }
    if !runner.contains("&stop,") {
        wrong.push(
            "the chunk of text is not handed the stop, so a pause waits for fifty messages".into(),
        );
    }
    wrong
}

#[test]
fn test_the_download_asks_whether_to_stop_between_chunks() {
    // One flag, `downloading.paused`, read in one closure and asked in two
    // places: before each chunk of headers here, and before each message
    // inside the chunk of text, which `fetch_over_a_mailbox` does with the
    // stop it is handed. A chunk in flight finishes; the next does not start.
    let runner = the_item_starting_with(&the_window_itself(), THE_RUNNER);
    let wrong = what_is_wrong_with_the_stop(&runner);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_a_download_that_ignored_the_stop() {
    // Proving the reading before believing it: a runner that reads the flag
    // and asks nobody, and one that asks before headers and hands nothing to
    // the text.
    let reads_and_ignores = "fn start_the_download(app: AppHandles<'_>) {\n    \
                             let stop = || lock_state(&state).downloading.paused;\n    \
                             let _ = stop;\n}\n";
    let wrong = what_is_wrong_with_the_stop(reads_and_ignores);
    assert!(
        wrong
            .iter()
            .any(|w| w.contains("before a chunk of headers"))
            && wrong.iter().any(|w| w.contains("chunk of text")),
        "the reading did not see the stop ignored: {wrong:?}"
    );

    let asks_for_headers_only = "fn start_the_download(app: AppHandles<'_>) {\n    \
                                 let stop = || lock_state(&state).downloading.paused;\n    \
                                 if stop() { return; }\n    \
                                 fetch_over_a_mailbox(server, cache, &chunk, &never, &after);\n}\n";
    let wrong = what_is_wrong_with_the_stop(asks_for_headers_only);
    assert_eq!(
        wrong.len(),
        1,
        "the reading did not see exactly the text chunk left without a stop: {wrong:?}"
    );
}

// ── A failed chunk waits before the download is tried again ──────────────────

/// What is wrong with what a failure does, as sentences; nothing when the
/// wait rule is asked and the moment to try again is recorded for the timer.
fn what_is_wrong_with_the_wait(runner: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    if !runner.contains(".next_wait()") {
        wrong.push(
            "a failed chunk never asks the wait rule, so a server that refused is asked again \
             at once"
                .into(),
        );
    }
    if !runner.contains("next_attempt_at = Some(") {
        wrong.push(
            "the moment to try again is never recorded, so the timer has nothing to start".into(),
        );
    }
    if !runner.contains(".tell_it_worked()") {
        wrong.push(
            "a chunk that succeeds never tells the wait rule, so one refusal an hour grows the \
             wait to its cap"
                .into(),
        );
    }
    wrong
}

#[test]
fn test_a_failed_chunk_waits_before_the_download_is_tried_again() {
    // `trying_again::WaitBeforeTryingAgain` answers how long, thirty seconds
    // doubling to thirty minutes, and the runner records when rather than
    // sleeping: the main timer starts the next attempt when the moment has
    // passed, so no runtime thread is held asleep for half an hour.
    let runner = the_item_starting_with(&the_window_itself(), THE_RUNNER);
    let wrong = what_is_wrong_with_the_wait(&runner);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_a_failed_chunk_tried_again_at_once() {
    // Proving the reading before believing it: a runner that records a
    // moment without asking how long, and one that asks and never records.
    let no_wait_asked = "fn start_the_download(app: AppHandles<'_>) {\n    \
                         s.downloading.next_attempt_at = Some(std::time::Instant::now());\n    \
                         s.downloading.wait.tell_it_worked();\n}\n";
    let wrong = what_is_wrong_with_the_wait(no_wait_asked);
    assert!(
        wrong.iter().any(|w| w.contains("never asks the wait rule")),
        "the reading did not see the wait skipped: {wrong:?}"
    );

    let nothing_recorded = "fn start_the_download(app: AppHandles<'_>) {\n    \
                            let wait = s.downloading.wait.next_wait();\n    \
                            s.downloading.wait.tell_it_worked();\n    \
                            std::thread::sleep(wait);\n}\n";
    let wrong = what_is_wrong_with_the_wait(nothing_recorded);
    assert!(
        wrong.iter().any(|w| w.contains("never recorded")),
        "the reading did not see the moment go unrecorded: {wrong:?}"
    );
}

// ── Each message's text is asked for at most once a run ──────────────────────

/// The runner's arm for a chunk of text, up to the arm after it.
///
/// Bounded at `WhatToDoNext::EverythingIsHere`, the arm that follows it, for
/// the reason `the_arm_for` gives: an arm read to the end holds every arm
/// below it and passes on their words.
fn the_text_arm(runner: &str) -> &str {
    let Some(at) = runner.find("WhatToDoNext::TheNextChunkOfText {") else {
        return "";
    };
    let rest = &runner[at..];
    &rest[..rest
        .find("WhatToDoNext::EverythingIsHere")
        .unwrap_or(rest.len())]
}

/// What is wrong with the list the model is handed, as sentences; nothing
/// when what this run already asked for is left out of it first.
fn what_is_wrong_with_the_missing_list(runner: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    let leaves_out = runner.find("asked.not_yet_asked(&missing)");
    let asks = runner.find("what_to_do_next(");
    match (leaves_out, asks) {
        (Some(leaves_out), Some(asks)) if leaves_out < asks => {}
        _ => wrong.push(
            "the download does not leave out what this run already asked for before it asks \
             the model, so a message whose text always fails is asked for again with no wait"
                .into(),
        ),
    }
    if !runner.contains("messages: &not_yet_asked,") {
        wrong.push(
            "the model is handed the whole missing list rather than what this run has not \
             asked for yet"
                .into(),
        );
    }
    wrong
}

/// What is wrong with the arm for a chunk of text, as sentences; nothing when
/// it remembers the chunk and counts it as work only when it fetched
/// something.
fn what_is_wrong_with_the_text_arm(runner: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    let arm = the_text_arm(runner);
    if arm.is_empty() {
        wrong.push("the runner has no arm for a chunk of text, so this reads nothing".into());
        return wrong;
    }
    if !arm.contains("asked.remember(&messages)") {
        wrong.push(
            "a chunk of text is not remembered, so the next ask hands the same messages back"
                .into(),
        );
    }
    if arm.contains("did_anything = true;")
        || !arm.contains("a_chunk_of_text_did_work(done.fetched)")
    {
        wrong.push(
            "a chunk of text counts as work whatever it fetched, so a run that only asked again \
             for text that fails says the download is done at every check"
                .into(),
        );
    }
    wrong
}

/// What is wrong with how a refused chunk of text ends, as sentences;
/// nothing when it ends this account's run, still marked as a server that
/// stopped so the run waits and is tried again.
fn what_is_wrong_with_a_text_refusal(runner: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    let arm = the_text_arm(runner);
    let Some(at) = arm.find("Ending::TheServerStoppedAnswering") else {
        wrong.push("the arm for a chunk of text has no refusal, so this reads nothing".into());
        return wrong;
    };
    let refusal = &arm[at..];
    if refusal.contains("break 'accounts") || !refusal.contains("break;") {
        wrong.push(
            "a refused chunk of text ends the run for every account after this one, so one \
             account's trouble holds up the others"
                .into(),
        );
    }
    if !refusal.contains("ended = WhyTheRunEnded::AServerStopped;") {
        wrong.push(
            "a refused chunk of text is not marked as a server that stopped, so the run is not \
             tried again after the wait"
                .into(),
        );
    }
    wrong
}

/// The runner's text loop as it stood until 2026-10-15, planted, which
/// every reading in this section is proven against.
const THE_TEXT_LOOP_BEFORE_THE_FIX: &str = "\
                let missing = match cache.messages_with_no_text_here(&account.id) {
                    Ok(missing) => missing,
                    Err(e) => break,
                };
                let next = crate::application::bringing_everything_down::what_to_do_next(
                    &here,
                    crate::application::bringing_everything_down::TextStillMissing {
                        messages: &missing,
                        kept_bytes,
                    },
                );
                match next {
                    WhatToDoNext::TheNextChunkOfText { messages, .. } => {
                        did_anything = true;
                        let done = handle.block_on(fetch_over_a_mailbox(server, &cache, &messages, &stop, &|_| {}));
                        match done.ended {
                            Ending::WentThroughTheWholeList => worked(),
                            Ending::TheServerStoppedAnswering { after, because } => {
                                ended = WhyTheRunEnded::AServerStopped;
                                break 'accounts;
                            }
                        }
                    }
                    WhatToDoNext::EverythingIsHere { why } => break,
                }
";

#[test]
fn test_the_download_leaves_out_the_text_this_run_already_asked_for() {
    // Finding #11: nothing records a message whose text failed, so the list
    // of what has no text here hands it back after every chunk. The run
    // keeps what it asked for and the model is handed only the rest.
    let runner = the_item_starting_with(&the_window_itself(), THE_RUNNER);
    let wrong = what_is_wrong_with_the_missing_list(&runner);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_text_asked_for_again() {
    // Proving the reading before believing it.
    let wrong = what_is_wrong_with_the_missing_list(THE_TEXT_LOOP_BEFORE_THE_FIX);
    assert_eq!(
        wrong.len(),
        2,
        "the reading did not see the whole list handed to the model: {wrong:?}"
    );
}

#[test]
fn test_a_chunk_of_text_is_remembered_and_is_work_only_when_it_fetched_something() {
    let runner = the_item_starting_with(&the_window_itself(), THE_RUNNER);
    let wrong = what_is_wrong_with_the_text_arm(&runner);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_a_chunk_forgotten_or_counted_as_work_for_nothing() {
    // Proving the reading before believing it: the arm as it was, and one
    // that remembers the chunk and still counts every chunk as work.
    let wrong = what_is_wrong_with_the_text_arm(THE_TEXT_LOOP_BEFORE_THE_FIX);
    assert!(
        wrong.iter().any(|w| w.contains("not remembered"))
            && wrong.iter().any(|w| w.contains("whatever it fetched")),
        "the reading did not see the arm as it was: {wrong:?}"
    );

    let remembers_and_still_counts = THE_TEXT_LOOP_BEFORE_THE_FIX.replace(
        "did_anything = true;",
        "did_anything = true;\n asked.remember(&messages);",
    );
    let wrong = what_is_wrong_with_the_text_arm(&remembers_and_still_counts);
    assert_eq!(
        wrong.len(),
        1,
        "the reading did not see exactly the chunk counted as work for nothing: {wrong:?}"
    );
}

#[test]
fn test_a_text_refusal_ends_its_own_account_and_not_every_account() {
    // A server that stops answering one account's text ends that account's
    // run; the accounts after it still get theirs, and the run still waits
    // and is tried again because it is marked as a server that stopped.
    let runner = the_item_starting_with(&the_window_itself(), THE_RUNNER);
    let wrong = what_is_wrong_with_a_text_refusal(&runner);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_a_text_refusal_end_every_account() {
    // Proving the reading before believing it: the arm as it was, and one
    // that ends the account and forgets to wait.
    let wrong = what_is_wrong_with_a_text_refusal(THE_TEXT_LOOP_BEFORE_THE_FIX);
    assert_eq!(
        wrong.len(),
        1,
        "the reading did not see exactly every account ended: {wrong:?}"
    );
    assert!(
        wrong[0].contains("every account after this one"),
        "{wrong:?}"
    );

    let ends_without_waiting = THE_TEXT_LOOP_BEFORE_THE_FIX
        .replace("break 'accounts;", "break;")
        .replace("ended = WhyTheRunEnded::AServerStopped;", "");
    let wrong = what_is_wrong_with_a_text_refusal(&ends_without_waiting);
    assert!(
        wrong.iter().any(|w| w.contains("not tried again")),
        "the reading did not see the wait forgotten: {wrong:?}"
    );
}

// ── The whole-folder command is gone, because this is what it did ────────────

/// The words of the retired command the window still carries, if any.
fn the_retired_words_in(window: &str) -> Vec<&'static str> {
    THE_WORDS_OF_THE_RETIRED_COMMAND
        .into_iter()
        .filter(|word| window.contains(word))
        .collect()
}

#[test]
fn test_the_whole_folder_command_is_gone_because_the_download_is_what_it_did() {
    // Two paths that bring a folder down chunk by chunk would be two places
    // for the next bound to be forgotten in. The download prefers the folder
    // on screen, so the command that asked for one folder is this, and its
    // loop's module went with it.
    let still_named = the_retired_words_in(&the_window_itself());
    assert!(
        still_named.is_empty(),
        "the window still carries the retired whole-folder command: {still_named:?}"
    );
    assert!(
        !std::path::Path::new(THE_RETIRED_MODULE).exists(),
        "{THE_RETIRED_MODULE} is still in the tree, with a loop nothing runs"
    );
}

#[test]
fn test_the_reading_would_see_the_command_come_back() {
    // Proving the reading before believing it.
    let with_the_item = "            .append_item(\n                ID_GET_WHOLE_FOLDER,\n";
    assert_eq!(
        the_retired_words_in(with_the_item),
        vec!["ID_GET_WHOLE_FOLDER"],
        "the reading did not see the item come back"
    );
}

// ── Get Older Messages hands to the download, this folder first ───────────────

/// What is wrong with the Get Older Messages arm, as sentences; nothing when
/// it says the folder comes first and hands to the download rather than
/// running a sync of its own.
fn what_is_wrong_with_get_older_messages(arm: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    if arm.contains("spawn_mail_sync(") {
        wrong.push(
            "Get Older Messages runs a sync of its own, so there are two paths that bring older \
             mail down and the download is not one of them"
                .into(),
        );
    }
    let Some(hands_over) = arm.find("start_the_download(") else {
        wrong.push("Get Older Messages never hands to the download".into());
        return wrong;
    };
    match arm.find("send_status(") {
        Some(says) if says < hands_over => {}
        _ => wrong.push(
            "Get Older Messages says nothing before handing over, so pressing Shift+F9 is \
             silence for as long as a server takes"
                .into(),
        ),
    }
    wrong
}

#[test]
fn test_get_older_messages_hands_to_the_download_with_this_folder_first() {
    // Shift+F9 keeps its meaning, "carry on downloading, this folder first",
    // and loses its own sync: the download prefers the folder on screen, so
    // handing to it is the same thing with one path instead of two. It is an
    // answer to a key, said before the hand-over, because the download's own
    // lines are steps and a step is silent under Say what arrived.
    let arm = the_menu_arm_for(&the_window_itself(), "ID_GET_OLDER");
    let wrong = what_is_wrong_with_get_older_messages(&arm);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_get_older_messages_running_its_own_sync() {
    // Proving the reading before believing it: the arm as it was until
    // 2026-09-17, and one that hands over in silence.
    let the_old_arm = "\n    send_status(&ui_tx, &runtime, \"Getting older messages...\");\n    \
                       spawn_mail_sync(app, Some(path), MoreOfWhatIsAlreadyThere);\n";
    let wrong = what_is_wrong_with_get_older_messages(the_old_arm);
    assert!(
        wrong.iter().any(|w| w.contains("a sync of its own"))
            && wrong.iter().any(|w| w.contains("never hands")),
        "the reading did not see the old arm: {wrong:?}"
    );

    let hands_over_in_silence = "\n    start_the_download(app);\n";
    let wrong = what_is_wrong_with_get_older_messages(hands_over_in_silence);
    assert!(
        wrong.iter().any(|w| w.contains("says nothing")),
        "the reading did not see the silence: {wrong:?}"
    );
}

// ── Pause Downloading is on the Tools menu, ticked, with the warning ─────────

/// The source with every space taken out, so a call can be found whatever
/// way rustfmt has chosen to wrap it.
fn without_whitespace(source: &str) -> String {
    source.chars().filter(|c| !c.is_whitespace()).collect()
}

/// What is wrong with the Pause item, as sentences; nothing when it is a
/// check item on the Tools menu whose description is the experimental
/// sentence, its arm reaches the one function that flips the flag, and that
/// function ticks the item as well.
fn what_is_wrong_with_the_pause_item(window: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    let squashed = without_whitespace(window);
    match squashed.find("append_check_item(ID_PAUSE_DOWNLOADING,") {
        None if squashed.contains("append_item(ID_PAUSE_DOWNLOADING,") => wrong.push(
            "Pause Downloading is a plain item, so a screen reader cannot say whether the \
             download is paused"
                .into(),
        ),
        None => wrong.push("Pause Downloading is on no menu".into()),
        Some(at) => {
            let item = &squashed[at..squashed.len().min(at + 400)];
            if !item.contains("DOWNLOADING_EVERYTHING_IS_EXPERIMENTAL") {
                wrong.push(
                    "the Pause item's description is not the sentence saying the download has \
                     never met a real provider, so nobody deciding whether to pause reads it"
                        .into(),
                );
            }
        }
    }
    let Some(tools) = window.find("let tools = Menu::builder()") else {
        wrong.push("the Tools menu is not built where this reads".into());
        return wrong;
    };
    let tools_menu = &window[tools..];
    let tools_menu = &tools_menu[..tools_menu.find(".build();").unwrap_or(tools_menu.len())];
    if !tools_menu.contains("ID_PAUSE_DOWNLOADING") {
        wrong.push("Pause Downloading is not on the Tools menu".into());
    }
    match squashed.find("_ifid==ID_PAUSE_DOWNLOADING=>") {
        None => wrong.push("no arm answers for Pause Downloading".into()),
        Some(at) => {
            let arm = &squashed[at..];
            let ends = arm[1..].find("_ifid==").map_or(arm.len(), |next| next + 1);
            if !arm[..ends].contains("pause_or_carry_on_downloading(") {
                wrong.push("the Pause arm does not reach the function that pauses".into());
            }
        }
    }
    match window.find("fn pause_or_carry_on_downloading(") {
        None => wrong.push("nothing pauses".into()),
        Some(at) => {
            let body = &window[at..];
            let body = &body[..body.find("\n}\n").unwrap_or(body.len())];
            if !body.contains("sync_menu_check(frame, ID_PAUSE_DOWNLOADING") {
                wrong.push(
                    "pausing flips the flag and not the tick, so the menu says the opposite of \
                     the truth half the time"
                        .into(),
                );
            }
        }
    }
    wrong
}

#[test]
fn test_pause_downloading_is_on_the_tools_menu_ticked_and_carries_the_warning() {
    // The one way to hold the download. A check item, because a screen
    // reader says "checked" or "unchecked" from the item's own state and a
    // plain item would leave somebody guessing; on Tools, beside the other
    // things that act on the whole program; with the experimental sentence
    // as its description, which is what Windows shows in the status bar and
    // hands over as the item's accessible description, because somebody
    // deciding whether to pause is the person who needs to read it.
    let wrong = what_is_wrong_with_the_pause_item(&the_window_itself());
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_a_plain_item_or_an_untouched_tick() {
    // Proving the reading before believing it: a plain item, a check item
    // with a plain description, and a function that flips the flag alone.
    let a_plain_item = "        let tools = Menu::builder()\n            .append_item(\n                \
                        ID_PAUSE_DOWNLOADING,\n                \"&Pause Downloading\",\n                \
                        crate::application::allowed::DOWNLOADING_EVERYTHING_IS_EXPERIMENTAL,\n            )\n            \
                        .build();\n\
                        _ if id == ID_PAUSE_DOWNLOADING => {\n    pause_or_carry_on_downloading(app, &frame);\n}\n\
                        fn pause_or_carry_on_downloading(app: AppHandles<'_>, frame: &Frame) {\n    \
                        sync_menu_check(frame, ID_PAUSE_DOWNLOADING, paused);\n}\n";
    let wrong = what_is_wrong_with_the_pause_item(a_plain_item);
    assert_eq!(
        wrong,
        vec![
            "Pause Downloading is a plain item, so a screen reader cannot say whether the \
             download is paused"
                .to_string()
        ],
        "the reading did not see exactly the plain item"
    );

    let no_warning_and_no_tick = a_plain_item
        .replace("append_item(", "append_check_item(")
        .replace(
            "crate::application::allowed::DOWNLOADING_EVERYTHING_IS_EXPERIMENTAL",
            "\"Hold the download\"",
        )
        .replace(
            "    sync_menu_check(frame, ID_PAUSE_DOWNLOADING, paused);\n",
            "",
        );
    let wrong = what_is_wrong_with_the_pause_item(&no_warning_and_no_tick);
    assert!(
        wrong
            .iter()
            .any(|w| w.contains("never met a real provider"))
            && wrong.iter().any(|w| w.contains("not the tick")),
        "the reading did not see the warning and the tick missing: {wrong:?}"
    );
}

// ── A chunk that lands is shown, and no limit grows ──────────────────────────

/// The words a limit on the list would be written in, none of which the arm
/// may name now that there is no limit to grow.
const THE_WORDS_OF_A_LIMIT: [&str; 3] = ["message_list_limit", "FOLDER_LIST_PAGE_SIZE", "+="];

/// What is wrong with the arm a chunk's arrival reaches, as sentences;
/// nothing when it marks the folder for the timer to read again, reads
/// nothing itself and grows no limit.
///
/// Until 2026-10-15 this asked for `reread_folder_if_open(` in the arm. The
/// arm read the whole open folder once per chunk, which grows with the
/// square of the folder and spoke its count each time (finding #36), so the
/// arm marks and the main timer reads, at most every ten seconds.
fn what_is_wrong_with_the_arm(arm: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    if !arm.contains(".a_chunk_landed(") {
        wrong.push(format!(
            "the arm a chunk's arrival reaches does not mark the folder, so nothing reads it \
             again and what the chunk brought stays in the cache and off the list: {arm}"
        ));
    }
    if arm.contains("reread_folder_if_open(") {
        wrong.push(format!(
            "the arm a chunk's arrival reaches reads the folder again itself, once per chunk, \
             which grows with the square of the folder: {arm}"
        ));
    }
    for word in THE_WORDS_OF_A_LIMIT {
        if arm.contains(word) {
            wrong.push(format!(
                "the arm a chunk's arrival reaches names `{word}`: a limit on what the list shows \
                 is back, and the list holds everything the folder holds only while there is \
                 none: {arm}"
            ));
        }
    }
    wrong
}

#[test]
fn test_a_chunk_arriving_rereads_the_folder_and_grows_no_limit() {
    // The view side. The download runs on a worker thread and sends
    // `MoreOfTheFolderArrived` once per chunk of headers; the arm for that
    // update used to grow the list's limit by a page and then re-read the
    // folder. There is no limit since 10-02, and a limit written back into
    // it is the page returning under another name. Since 2026-10-15 the arm
    // marks the folder and the main timer reads it, at most every ten
    // seconds (finding #36), so the folder is still read again, and not
    // once per chunk.
    let source = the_window_itself();
    let runner = the_item_starting_with(&source, THE_RUNNER);

    assert!(
        runner.contains("MoreOfTheFolderArrived"),
        "the download tells nobody that a chunk landed, so nothing re-reads the folder it \
         lands in"
    );

    let wrong = what_is_wrong_with_the_arm(&the_arm_for(&source, "MoreOfTheFolderArrived"));
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_a_limit_grown_again() {
    // Proving the reading before believing it: the old arm, with the page
    // grown before the re-read, planted whole, which also reads the folder
    // once per chunk; and, since 2026-10-15, an arm that marks nothing.
    let the_old_arm = "(folder_id) => {\n\
                       \x20           {\n\
                       \x20               let mut s = lock_state(state);\n\
                       \x20               s.message_list_limit += FOLDER_LIST_PAGE_SIZE;\n\
                       \x20           }\n\
                       \x20           reread_folder_if_open(state, message_cache, *folder_id, tx);\n\
                       \x20       }\n";
    let wrong = what_is_wrong_with_the_arm(the_old_arm);

    assert_eq!(
        wrong.iter().filter(|w| w.contains("names `")).count(),
        THE_WORDS_OF_A_LIMIT.len(),
        "the reading did not see every word of the limit grown again: {wrong:?}"
    );
    assert!(
        wrong
            .iter()
            .any(|w| w.contains("reads the folder again itself")),
        "the reading did not see the folder read once per chunk: {wrong:?}"
    );

    let an_arm_that_forgot_to_mark = "(folder_id) => {\n    let _ = folder_id;\n}\n";
    let wrong = what_is_wrong_with_the_arm(an_arm_that_forgot_to_mark);
    assert_eq!(
        wrong.len(),
        1,
        "the reading did not see exactly the mark taken out: {wrong:?}"
    );
    assert!(wrong[0].contains("does not mark"), "{wrong:?}");
}

// ── The open folder is read again at most every ten seconds ──────────────────

/// What is wrong with the main timer's half of the paced re-read, as
/// sentences; nothing when the timer, between the held mail and the
/// reminders, asks which folder is due and reads it with a chunk's reason.
fn what_is_wrong_with_the_timers_reread(window: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    let (Some(held_mail), Some(reminders)) = (
        window.find("HOW_OFTEN_TO_LET_HELD_MAIL_GO {"),
        window.find("let somewhere_to_type = ["),
    ) else {
        wrong.push("the main timer is not where this reads, so this reads nothing".into());
        return wrong;
    };
    let timer = &window[held_mail..reminders.max(held_mail)];
    let Some(asks) = timer.find(".the_folder_due(") else {
        wrong.push(
            "the main timer never asks which folder is due, so a folder a chunk marked is never \
             read again and what the download brought stays off the list"
                .into(),
        );
        return wrong;
    };
    let block = &timer[asks..];
    let block = &block[..block.find("\n\n").unwrap_or(block.len())];
    if !block.contains("reread_folder_if_open(") {
        wrong.push(
            "the main timer asks which folder is due and never reads it, so what the download \
             brought stays off the list"
                .into(),
        );
    }
    if !block.contains("WhyTheRowsWereRead::AChunkOfTheDownloadLanded") {
        wrong.push(
            "the main timer reads the folder without saying a chunk of the download is why, so \
             the count is spoken every ten seconds"
                .into(),
        );
    }
    wrong
}

#[test]
fn test_the_timer_reads_the_marked_folder_again_with_a_chunks_reason() {
    // Finding #36. The arm marks; the main timer, which already paces held
    // mail on its own interval, asks the model which folder is due and
    // reads it, at most every ten seconds and once after the last chunk.
    let wrong = what_is_wrong_with_the_timers_reread(&the_window_itself());
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_the_timer_leave_the_folder_unread() {
    // Proving the reading before believing it: the timer as it was, and one
    // that reads the folder for an arrival.
    let the_timer_as_it_was = "if looked_for_held_mail_at.get().elapsed() >= \
                               HOW_OFTEN_TO_LET_HELD_MAIL_GO {\n    flush_outbox(app);\n}\n\n\
                               let somewhere_to_type = [\n";
    let wrong = what_is_wrong_with_the_timers_reread(the_timer_as_it_was);
    assert!(
        wrong.len() == 1 && wrong[0].contains("never asks which folder is due"),
        "the reading did not see the timer as it was: {wrong:?}"
    );

    let reads_for_an_arrival = "HOW_OFTEN_TO_LET_HELD_MAIL_GO {\n}\n\
                                if let Some(folder_id) = pacing.the_folder_due(now) {\n    \
                                reread_folder_if_open(&state, &message_cache, folder_id, \
                                WhyTheRowsWereRead::MailArrived, &ui_tx);\n}\n\n\
                                let somewhere_to_type = [\n";
    let wrong = what_is_wrong_with_the_timers_reread(reads_for_an_arrival);
    assert!(
        wrong.len() == 1 && wrong[0].contains("count is spoken every ten seconds"),
        "the reading did not see the folder read for an arrival: {wrong:?}"
    );
}

// ── A chunk's re-read shows the count and does not speak it ──────────────────

/// What is wrong with how one arm says the folder's count, as sentences;
/// nothing when it always shows the count and speaks it only when a chunk of
/// the download is not why the rows were read.
///
/// Read with the spaces taken out, so the check is about the code and not
/// about how rustfmt wrapped it.
fn what_is_wrong_with_how_the_count_is_said(arm: &str, which: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    let squashed = without_whitespace(arm);
    if !squashed.contains("frame.set_status_text(&msg,0);") {
        wrong.push(format!(
            "the {which} arm no longer shows the folder's count on the status bar"
        ));
    }
    let asks = squashed.find("if*why!=WhyTheRowsWereRead::AChunkOfTheDownloadLanded{");
    let speaks = squashed.find("announce_topic(&msg,Priority::Normal,\"messages\")");
    match (asks, speaks) {
        (_, None) => wrong.push(format!(
            "the {which} arm never speaks the folder's count, so opening a folder says nothing"
        )),
        (Some(asks), Some(speaks)) if asks < speaks => {}
        _ => wrong.push(format!(
            "the {which} arm speaks the folder's count whatever the reason, so a first download \
             reads it out every time the list is brought up to date"
        )),
    }
    wrong
}

/// What is wrong with the three arms a re-read reaches, as sentences.
fn what_is_wrong_with_what_a_reread_says(window: &str) -> Vec<String> {
    let mut wrong = what_is_wrong_with_how_the_count_is_said(
        &the_arm_for(window, "MessagesLoaded("),
        "MessagesLoaded",
    );
    let conversations = the_arm_for(window, "ConversationsLoaded(");
    wrong.extend(what_is_wrong_with_how_the_count_is_said(
        &conversations,
        "ConversationsLoaded",
    ));
    if !without_whitespace(&conversations).contains(
        "WhyTheRowsWereRead::MailArrived|WhyTheRowsWereRead::AChunkOfTheDownloadLanded=>{\
         repaint_the_rows_that_changed(",
    ) {
        wrong.push(
            "a chunk's conversations are not repainted the way an arrival's are, so the rows \
             on screen move or are announced again"
                .into(),
        );
    }
    if !the_arm_for(window, "FolderMessagesArrived(").contains("WhyTheRowsWereRead::MailArrived") {
        wrong.push(
            "the arm a check's arrival reaches does not read the folder for an arrival, so a \
             check that found mail may not say the count"
                .into(),
        );
    }
    wrong
}

#[test]
fn test_a_chunks_reread_shows_the_count_and_does_not_speak_it() {
    // Guardrail 5. A first download of a large folder lands a chunk every
    // few seconds, and the count spoken after each, in flat view and in
    // conversation view alike, is noise in a room where mail is private.
    // Opening a folder and a check that found mail still say it; every
    // reason shows it on the status bar.
    let wrong = what_is_wrong_with_what_a_reread_says(&the_window_itself());
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_reading_would_see_a_chunks_count_spoken() {
    // Proving the reading before believing it: the two arms' ends as they
    // were, each speaking the count whatever the reason.
    let the_arm_as_it_was = "messages) => {\n\
                             \x20   let msg = what_a_mailbox_holds(messages.len(), unread);\n\
                             \x20   frame.set_status_text(&msg, 0);\n\
                             \x20   let _ = a11y.announce_topic(&msg, Priority::Normal, \"messages\");\n\
                             }\n";
    let wrong = what_is_wrong_with_how_the_count_is_said(the_arm_as_it_was, "MessagesLoaded");
    assert!(
        wrong.len() == 1 && wrong[0].contains("whatever the reason"),
        "the reading did not see the count spoken for every reason: {wrong:?}"
    );

    let says_and_shows_nothing = the_arm_as_it_was
        .replace("    frame.set_status_text(&msg, 0);\n", "")
        .replace(
            "    let _ = a11y.announce_topic(&msg, Priority::Normal, \"messages\");\n",
            "",
        );
    let wrong = what_is_wrong_with_how_the_count_is_said(&says_and_shows_nothing, "MessagesLoaded");
    assert!(
        wrong.iter().any(|w| w.contains("no longer shows"))
            && wrong.iter().any(|w| w.contains("never speaks")),
        "the reading did not see the count neither shown nor spoken: {wrong:?}"
    );
}

#[test]
fn test_the_item_reading_stops_at_the_next_item() {
    // Proving the item reading before believing what it says. A cut that had
    // stopped finding the runner passes every assertion above by finding
    // nothing, and a cut that ran to the end of the file passes them by
    // finding the words somewhere else entirely.
    let two_items = "fn start_the_download(app: AppHandles<'_>) {\n    \
                     let limit = INITIAL_FETCH_LIMIT;\n}\n\
                     fn something_else() {\n    let _ = FOLDER_LIST_PAGE_SIZE;\n}\n";
    let cut = the_item_starting_with(two_items, THE_RUNNER);

    assert!(
        cut.contains("INITIAL_FETCH_LIMIT"),
        "the cut did not reach the runner at all: {cut}"
    );
    assert!(
        !cut.contains("FOLDER_LIST_PAGE_SIZE"),
        "the cut ran past the runner and read the next item's words: {cut}"
    );
}

// ── The download's lines are steps, and its result is said once per account ──

#[test]
fn test_the_downloads_lines_are_steps_and_its_result_is_said_once_per_account() {
    // Every line on the way, each chunk and each folder finished, goes out
    // as `UIUpdate::Progress`, shown always and spoken only under Say every
    // step (#38): a mailbox of fifty folders would otherwise say fifty
    // sentences at Normal over the hours a first download takes. What an
    // account came to goes out once, as `UIUpdate::WhatArrived`, which is
    // the one sentence a run produces under the default. Nothing rides the
    // answer channel, where it would be spoken under every choice.
    let window = the_window_itself();
    let runner = the_item_starting_with(&window, THE_RUNNER);

    assert!(
        runner.contains("UIUpdate::Progress("),
        "the download sends no line as a step: {runner}"
    );
    assert!(
        !runner.contains("UIUpdate::StatusUpdated("),
        "the download sends a line on the answer channel, where Say what arrived cannot \
         quieten it: {runner}"
    );
    assert_eq!(
        runner.matches("UIUpdate::WhatArrived").count(),
        1,
        "what an account came to is said other than once, at the end of its run: {runner}"
    );
    let arm = the_arm_for(&window, "Progress(");
    assert!(
        arm.contains("is_spoken(Kind::Progress)"),
        "the arm a step reaches does not ask the level, so every step is spoken: {arm}"
    );
    assert!(
        !arm.contains("\"status\""),
        "a step is announced where the next answer replaces it: {arm}"
    );
}

#[test]
fn test_the_arm_reading_stops_at_the_next_arm() {
    // Proving that reading too. An arm read to the end of the match holds every
    // arm below it, and one of those announces on `"status"`, so the assertion
    // above would fail against correct code and pass against nothing.
    let two_arms = "        UIUpdate::Progress(said) => {\n\
                    \x20           if level.is_spoken(Kind::Progress) { announce(said, \"progress\"); }\n\
                    \x20       }\n\
                    \x20       UIUpdate::StatusUpdated(status) => {\n\
                    \x20           announce(status, \"status\");\n\
                    \x20       }\n";
    let arm = the_arm_for(two_arms, "Progress(");

    assert!(
        arm.contains("is_spoken(Kind::Progress)"),
        "the arm was not read: {arm}"
    );
    assert!(
        !arm.contains("\"status\""),
        "the reading ran into the next arm and took its topic: {arm}"
    );
}

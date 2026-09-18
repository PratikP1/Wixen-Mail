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
/// exactly the break that matters.
fn what_is_wrong_with_the_end_of_a_check(check: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    let Some(watch) = check.find("say(UIUpdate::MailboxWatchRequested)") else {
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
                       \x20       say(UIUpdate::MailboxWatchRequested);\n\
                       \x20   });\n";
    let wrong = what_is_wrong_with_the_end_of_a_check(the_old_end);
    assert!(
        wrong
            .iter()
            .any(|w| w.contains("never asks for the download")),
        "the reading did not see the download missing: {wrong:?}"
    );

    let the_wrong_way_round = "        say(UIUpdate::DownloadRequested);\n\
                               \x20       say(UIUpdate::MailboxWatchRequested);\n";
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

// ── A chunk that lands is shown, and no limit grows ──────────────────────────

/// The words a limit on the list would be written in, none of which the arm
/// may name now that there is no limit to grow.
const THE_WORDS_OF_A_LIMIT: [&str; 3] = ["message_list_limit", "FOLDER_LIST_PAGE_SIZE", "+="];

/// What is wrong with the arm a chunk's arrival reaches, as sentences;
/// nothing when it re-reads the folder and grows no limit.
fn what_is_wrong_with_the_arm(arm: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    if !arm.contains("reread_folder_if_open(") {
        wrong.push(format!(
            "the arm a chunk's arrival reaches does not re-read the folder, so what the chunk \
             brought stays in the cache and off the list: {arm}"
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
    // folder. There is no limit since 10-02, so the arm re-reads and does
    // nothing else, and a limit written back into it is the page returning
    // under another name.
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
    // grown before the re-read, planted whole.
    let the_old_arm = "(folder_id) => {\n\
                       \x20           {\n\
                       \x20               let mut s = lock_state(state);\n\
                       \x20               s.message_list_limit += FOLDER_LIST_PAGE_SIZE;\n\
                       \x20           }\n\
                       \x20           reread_folder_if_open(state, message_cache, *folder_id, tx);\n\
                       \x20       }\n";
    let wrong = what_is_wrong_with_the_arm(the_old_arm);

    assert_eq!(
        wrong.len(),
        THE_WORDS_OF_A_LIMIT.len(),
        "the reading did not see every word of the limit grown again: {wrong:?}"
    );

    let an_arm_that_forgot_to_reread = "(folder_id) => {\n    let _ = folder_id;\n}\n";
    let wrong = what_is_wrong_with_the_arm(an_arm_that_forgot_to_reread);
    assert!(
        wrong.iter().any(|w| w.contains("does not re-read")),
        "the reading did not see the re-read taken out: {wrong:?}"
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

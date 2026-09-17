//! The whole-folder request moves the number that bounds what a folder
//! fetches, and a chunk arriving re-reads the folder without growing anything.
//!
//! There were two five hundreds and they were separate decisions.
//! `application::mail_sync::INITIAL_FETCH_LIMIT` bounds what comes down from
//! the server, and still does. Until 2026-09-17 a second one,
//! `presentation::wx_app::FOLDER_LIST_PAGE_SIZE`, bounded what was read out of
//! the cache into the list, and a request that moved one and not the other
//! appeared to do nothing, because the other still bound: either mail arrived
//! and was never shown, or the list asked for rows that were never fetched.
//! 10-02 took the second bound off for #24, because it hid every message
//! past the newest 500 and pushed the oldest shown off the end as mail
//! arrived. So the arm that once grew it is held here to the opposite: a chunk
//! arriving re-reads the folder if it is open and names no limit, because
//! there is none to grow. The fetch bound, the experimental sentence and the
//! loop are as they were, until 10-05 retires the command.
//!
//! # Why this lives here rather than beside the code
//!
//! The handler is in `src/presentation/wx_app.rs`, which needs a window, a
//! frame and a running event loop to reach, so this is a source read. It is an
//! integration target rather than a test inside that file for a cost reason
//! that is worth writing down: when this was written, thirty-four guard
//! records fingerprinted the number of tests in `wx_app.rs` (57 on 2026-09-17
//! by the TOML reader), so one test added there is that many builds and that
//! many full library runs at the next commit. One record fingerprints this
//! file.
//!
//! `guards/guards.toml` couples this target to `wx_app.rs`, so it runs on the
//! commits that could break it rather than only on the commits that change it.
//! Without that coupling a guard living under `tests/` runs on every commit
//! except the ones that matter, which is the trap `CLAUDE.md` records.
//!
//! # What this cannot see
//!
//! It reads source. It says the handler is written to move both bounds and to
//! announce where a sync line cannot replace it. It does not say the handler is
//! reached, that the loop terminates, or that anybody hears the announcement.
//! The loop is answered in `application::asking_for_a_whole_folder`, where it
//! can be run without a window. What somebody hears is answered by a screen
//! reader or not at all, and is in the ledger as `unrun-verify`.

use std::fs;

use wixen_mail::common::what_ships::what_ships;

/// The window, which is where the request is driven from.
const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

/// The function that drives the request.
const THE_HANDLER: &str = "fn spawn_whole_folder_fetch";

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

#[test]
fn test_the_whole_folder_request_moves_the_bound_on_what_is_fetched() {
    // The server side. Without it the request asks the cache to show rows that
    // were never brought down.
    let handler = the_item_starting_with(&the_window_itself(), THE_HANDLER);

    assert!(
        handler.contains("INITIAL_FETCH_LIMIT"),
        "the whole-folder request does not move the bound on what is fetched \
         from the server, so it asks the list to show mail nothing downloaded"
    );
}

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
    // The view side, the other way round from before 2026-09-17. The request
    // runs on a worker thread and sends `MoreOfTheFolderArrived` once per
    // chunk; the arm for that update used to grow the list's limit by a page
    // and then re-read the folder. There is no limit now, so the arm re-reads
    // and does nothing else, and a limit written back into it is the page
    // returning under another name.
    let source = the_window_itself();
    let handler = the_item_starting_with(&source, THE_HANDLER);

    assert!(
        handler.contains("MoreOfTheFolderArrived"),
        "the whole-folder request tells nobody that another chunk landed, so \
         nothing re-reads the folder it lands in"
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
fn test_the_command_says_it_is_experimental_where_somebody_choosing_it_reads() {
    // A warning that only exists in a changelog is a warning nobody gets. A
    // menu has nowhere to put the line of static text the missing message text
    // offer carries beside its button, so it goes on the label and in the
    // description: the description is what Windows shows in the status bar and
    // hands over as the item's accessible description, and the label is read
    // whatever anybody's settings say.
    let source = the_window_itself();
    // The menu item rather than the identifier's own declaration, which is the
    // first place the name appears and carries no words at all.
    let at = source
        .find("                ID_GET_WHOLE_FOLDER,")
        .expect("the menu item for the whole-folder command");
    let item = &source[at..at + 400.min(source.len() - at)];

    assert!(
        item.contains("(experimental)"),
        "the label does not say the command is experimental: {item}"
    );
    assert!(
        item.contains("DOWNLOADING_A_WHOLE_FOLDER_IS_EXPERIMENTAL"),
        "the description is not the warning that says what could go wrong, so \
         somebody choosing this is told it is experimental and not what of: {item}"
    );
}

#[test]
fn test_the_whole_folder_request_carries_on_by_itself() {
    // Through the loop that can be run without a window, rather than through a
    // loop written here where nothing can test it.
    let handler = the_item_starting_with(&the_window_itself(), THE_HANDLER);

    assert!(
        handler.contains("until_the_whole_folder_is_here"),
        "the whole-folder request does not use the loop that keeps asking, so \
         it is one more chunk rather than the whole folder"
    );
}

#[test]
fn test_the_reading_would_see_a_request_that_moved_only_one_bound() {
    // Proving the reading before believing what it says. A cut that had stopped
    // finding the handler passes every assertion above by finding nothing, and
    // a cut that ran to the end of the file passes them by finding the words
    // somewhere else entirely.
    let one_bound_only = "fn spawn_whole_folder_fetch(app: AppHandles<'_>) {\n    \
                          let limit = INITIAL_FETCH_LIMIT;\n}\n\
                          fn something_else() {\n    let _ = FOLDER_LIST_PAGE_SIZE;\n}\n";
    let cut = the_item_starting_with(one_bound_only, THE_HANDLER);

    assert!(
        cut.contains("INITIAL_FETCH_LIMIT"),
        "the cut did not reach the handler at all: {cut}"
    );
    assert!(
        !cut.contains("FOLDER_LIST_PAGE_SIZE"),
        "the cut ran past the handler and read the next item's words: {cut}"
    );
}

#[test]
fn test_the_progress_is_not_announced_where_the_next_sync_line_replaces_it() {
    // `"status"` carries every steady sync line and the queue keeps only the
    // newest of a topic, so a fetch running for minutes announced there would
    // silence all of them for as long as it ran. The topic is a constant beside
    // the loop, so this asks that the window uses the constant rather than
    // writing a topic of its own.
    let arm = the_arm_for(&the_window_itself(), "WholeFolderProgress");

    assert!(
        arm.contains("THE_PROGRESS_TOPIC"),
        "the whole-folder progress does not announce on the topic the loop \
         names, so the two can come apart: {arm}"
    );
    assert!(
        !arm.contains("\"status\""),
        "the whole-folder progress is announced where the next sync line \
         replaces it: {arm}"
    );
}

#[test]
fn test_the_arm_reading_stops_at_the_next_arm() {
    // Proving that reading too. An arm read to the end of the match holds every
    // arm below it, and one of those announces on `"status"`, so the assertion
    // above would fail against correct code and pass against nothing.
    let two_arms = "        UIUpdate::WholeFolderProgress(said) => {\n\
                    \x20           announce(said, THE_PROGRESS_TOPIC);\n\
                    \x20       }\n\
                    \x20       UIUpdate::StatusUpdated(status) => {\n\
                    \x20           announce(status, \"status\");\n\
                    \x20       }\n";
    let arm = the_arm_for(two_arms, "WholeFolderProgress");

    assert!(
        arm.contains("THE_PROGRESS_TOPIC"),
        "the arm was not read: {arm}"
    );
    assert!(
        !arm.contains("\"status\""),
        "the reading ran into the next arm and took its topic: {arm}"
    );
}

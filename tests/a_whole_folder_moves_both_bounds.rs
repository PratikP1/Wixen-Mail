//! The whole-folder request moves both of the numbers that bound a folder.
//!
//! There are two five hundreds and they are separate decisions.
//! `application::mail_sync::INITIAL_FETCH_LIMIT` bounds what comes down from
//! the server. `presentation::wx_app::FOLDER_LIST_PAGE_SIZE` bounds what is
//! read out of the cache into the list. A request that moves one and not the
//! other appears to do nothing, because the other still binds: either mail
//! arrives and is never shown, or the list asks for rows that were never
//! fetched. Get Older Messages already moves both, and the whole-folder request
//! has to as well.
//!
//! # Why this lives here rather than beside the code
//!
//! The handler is in `src/presentation/wx_app.rs`, which needs a window, a
//! frame and a running event loop to reach, so this is a source read. It is an
//! integration target rather than a test inside that file for a cost reason
//! that is worth writing down: thirty-four guard records fingerprint the number
//! of tests in `wx_app.rs`, so one test added there is thirty-four builds and
//! thirty-four full library runs at the next commit. None fingerprint this file.
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

#[test]
fn test_the_whole_folder_request_moves_the_bound_on_what_the_list_shows() {
    // The view side. Without it the mail arrives and stays hidden behind a
    // limit the list still reads through, which looks exactly like a request
    // that did nothing.
    let handler = the_item_starting_with(&the_window_itself(), THE_HANDLER);

    assert!(
        handler.contains("FOLDER_LIST_PAGE_SIZE"),
        "the whole-folder request does not move the bound on what the list \
         shows, so what it fetches stays hidden"
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

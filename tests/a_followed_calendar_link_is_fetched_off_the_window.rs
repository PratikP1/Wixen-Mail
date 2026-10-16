//! A followed calendar link is fetched off the window's thread (audit
//! finding #20).
//!
//! A `webcal:` link on any web page reaches this program through Windows,
//! and the handover's calendar arm fetched the feed with `rt.block_on` on the
//! window's own thread, under a comment saying it did the opposite. A host
//! that accepted the connection and never answered froze the window, and a
//! screen reader reported it as not responding, until the host gave up.
//!
//! The fetch now goes to the runtime and its answer comes back as an update,
//! read into the store on the window's thread, because the store cannot
//! leave that thread. What is said and shown afterwards is what was said and
//! shown before, through the one reader the opened calendar file uses.
//!
//! The source readings, over `what_ships` of `src/presentation/wx_app.rs`:
//! the calendar arm of `open_what_windows_handed_over` waits on nothing and
//! hands the fetch to the runtime; the update that carries the answer back
//! reaches a reader that says what the arm said before. A companion plants
//! the wait back into a snippet shaped as the arm is, so a reading that
//! stopped finding its anchor cannot pass by finding nothing.
//!
//! # Why this lives here rather than beside the code
//!
//! A test added to `src/presentation/wx_app.rs` is a build and a library run
//! for every guard record naming that file at the next commit. This file is
//! named by its own record, whose `suite` couples it to `wx_app.rs`.
//!
//! # What this cannot see
//!
//! Whether the window really keeps drawing while a real feed is fetched, and
//! whether the sentences are heard as before under NVDA: the tester's, on a
//! build holding this change. The window is not started.

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

/// The text from `heading` to the end of `text`, or a complaint when the
/// heading is gone.
fn from<'a>(text: &'a str, heading: &str) -> Result<&'a str, String> {
    let at = text.find(heading).ok_or(format!(
        "{heading:?} is no longer here, so this reads nothing"
    ))?;
    Ok(&text[at..])
}

/// The body of one `UIUpdate::X => ` arm of the update handler, up to the
/// next arm of the same shape.
fn the_update_arm<'a>(source: &'a str, heading: &str) -> Result<&'a str, String> {
    let start = source.find(heading).ok_or(format!(
        "{heading:?} is no longer here, so this reads nothing"
    ))? + heading.len();
    let rest = &source[start..];
    let end = rest.find("\n        UIUpdate::").unwrap_or(rest.len());
    Ok(&rest[..end])
}

// ── The anchors, each a name or a literal ──────────────────────────────────

const THE_HANDOVER: &str = "fn open_what_windows_handed_over(";
/// The last arm of the handover's match, so the text from it to the end of
/// the function is the arm.
const THE_CALENDAR_ARM: &str = "Opening::CalendarSubscription(address) =>";
/// The arm and not the send: the send names the variant too, and comes first.
const THE_ANSWER_ARM: &str = "UIUpdate::CalendarFetched { address, fetched } =>";
const THE_FETCHED_READ: &str = "fn read_the_fetched_calendar(";
/// The one reader an opened calendar file and a fetched one share, so both
/// say the same things.
const THE_SHARED_READ: &str = "fn read_into_the_calendar(";
const THE_FILE_ARM: &str = "Opening::CalendarFile(path) =>";

/// Every way the window's thread can stop and wait on the runtime.
const WAITS_ON_THE_RUNTIME: [&str; 2] = ["block_on(", "block_in_place("];

/// What is wrong with a calendar arm, in words, or nothing.
fn what_the_calendar_arm_gets_wrong(arm: &str) -> Vec<String> {
    let mut wrong: Vec<String> = WAITS_ON_THE_RUNTIME
        .iter()
        .filter(|wait| arm.contains(*wait))
        .map(|wait| format!("it waits on the runtime from the window's thread with {wait}"))
        .collect();
    if !arm.contains("rt.spawn(") {
        wrong.push("it does not hand the fetch to the runtime".to_string());
    }
    if !arm.contains("fetch_ics(") {
        wrong.push("it does not fetch the calendar at all".to_string());
    }
    if !arm.contains("UIUpdate::CalendarFetched {") {
        wrong.push("it hands no answer back to the window's thread".to_string());
    }
    wrong
}

#[test]
fn test_the_calendar_arm_hands_the_fetch_to_the_runtime_and_waits_on_nothing() {
    let window = the_main_window();
    let handover = body_of(&window, THE_HANDOVER).unwrap_or_else(|why| panic!("{why}"));
    let arm = from(&handover, THE_CALENDAR_ARM).unwrap_or_else(|why| panic!("{why}"));

    let wrong = what_the_calendar_arm_gets_wrong(arm);

    assert!(
        wrong.is_empty(),
        "a followed calendar link would stop the window while its host takes its time: \
         {wrong:?}\n{arm}"
    );
}

#[test]
fn test_the_fetched_calendar_is_read_on_the_window_and_said_as_before() {
    let window = the_main_window();

    let answer = the_update_arm(&window, THE_ANSWER_ARM).unwrap_or_else(|why| panic!("{why}"));
    assert!(
        answer.contains("opens_a_handover(Handover::CalendarFetched"),
        "the fetched calendar's update does not reach the handover's reader:\n{answer}"
    );

    let fetched = body_of(&window, THE_FETCHED_READ).unwrap_or_else(|why| panic!("{why}"));
    assert!(
        fetched.contains("read_into_the_calendar("),
        "a fetched calendar is not read by the reader an opened file uses:\n{fetched}"
    );
    assert!(
        fetched.contains("could not follow that calendar"),
        "a fetched calendar with nowhere to keep it is not said as before:\n{fetched}"
    );

    let shared = body_of(&window, THE_SHARED_READ).unwrap_or_else(|why| panic!("{why}"));
    for said in [
        "read_calendar_file(",
        "what_reading_the_calendar_did(",
        ".told(",
        ".went_wrong(",
        "switch_to(PimModule::Calendar)",
    ] {
        assert!(
            shared.contains(said),
            "the reader no longer does {said:?}, so a calendar is not said as before:\n{shared}"
        );
    }

    let handover = body_of(&window, THE_HANDOVER).unwrap_or_else(|why| panic!("{why}"));
    let file =
        &from(&handover, THE_FILE_ARM).unwrap_or_else(|why| panic!("{why}"))[THE_FILE_ARM.len()..];
    // Up to the next arm of the same match.
    let file = &file[..file.find("Opening::").unwrap_or(file.len())];
    assert!(
        file.contains("read_into_the_calendar("),
        "an opened calendar file and a fetched calendar are said by two readers:\n{file}"
    );
}

#[test]
fn test_the_reading_sees_a_calendar_arm_that_waits_on_the_window() {
    // Shaped as the arm is, with the wait put back. If this passed, the
    // reading above could pass by finding nothing.
    let waits = r#"Opening::CalendarSubscription(address) => {
            says.told("Fetching that calendar.");
            let fetched = rt.block_on(async {
                crate::service::ICalSubscriptionClient::new()
                    .fetch_ics(&address)
                    .await
            });
        }"#;
    let spawned_and_handed_back = r#"Opening::CalendarSubscription(address) => {
            says.told("Fetching that calendar.");
            let hands_back = tx.clone();
            rt.spawn(async move {
                let fetched = crate::service::ICalSubscriptionClient::new()
                    .fetch_ics(&address)
                    .await;
                let _ = hands_back.send(UIUpdate::CalendarFetched { address, fetched }).await;
            });
        }"#;

    let wrong = what_the_calendar_arm_gets_wrong(waits);

    assert!(
        wrong.iter().any(|said| said.contains("block_on(")),
        "the reading cannot see the wait: {wrong:?}"
    );
    assert!(
        wrong
            .iter()
            .any(|said| said.contains("hands no answer back")),
        "the reading cannot see an answer nobody hands back: {wrong:?}"
    );
    assert_eq!(
        what_the_calendar_arm_gets_wrong(spawned_and_handed_back),
        Vec::<String>::new(),
        "the reading refuses the shape it asks for"
    );
}

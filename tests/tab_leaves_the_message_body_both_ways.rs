//! Leaving the composer's message body forward lands on the control after it,
//! and leaving backward lands on the Subject line.
//!
//! The body is a browser's page, so Tab and Shift+Tab are caught inside the
//! page and posted out, and the composer moves the keyboard on with
//! `wx_compose::leave_the_message_body`. That function calls wxDragon's
//! `navigate`, whose argument reads backwards at 0.9.23: `navigate(true)`
//! passes `WXD_NAVIGATION_NEXT`, which is 0, and wxWidgets reads 0 as
//! backward. An unreleased change upstream (#248) puts it the right way
//! round, so the next wxDragon release may reverse where the keyboard goes
//! with nothing failing to compile. This test is what notices.
//!
//! What it reads: the real composer built by `build_compose_dialog`, shown,
//! its page set and its browser made, the keyboard put in the body, then the
//! function called each way and where Windows says the keyboard went.
//!
//! What it cannot reach: the page's own Tab handler. The handler that turns
//! the page's message into the call is wired only inside
//! `show_compose_dialog_full`, which is modal and waits for a person, so this
//! drives the function that handler calls rather than a key pressed in the
//! page.
//!
//! One `#[test]` function, because there is one `wxdragon::main` per process
//! and this file spends it on the composer. It presses no key, so it runs in
//! this process on the desktop it was started on, as
//! `tests/a_signature_follows_the_from_account.rs` runs its composer where
//! `CI` is set.

#![cfg(windows)]

use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wixen_mail::application::identities::FromEntry;
use wixen_mail::presentation::wx_compose::{self, ComposeDialogWidgets};
use wxdragon::event::WebViewEvents;
use wxdragon::prelude::*;

#[link(name = "user32")]
unsafe extern "system" {
    fn GetFocus() -> isize;
    fn IsChild(parent: isize, window: isize) -> i32;
}

const TICK_MS: i32 = 30;
/// Ticks between one step and the next, so what a step started, the browser
/// taking the keyboard among it, has finished.
const TICKS_TO_SETTLE: u32 = 10;
/// How long the browser has to report before the run says so and ends.
/// Generous, because the runner took over three seconds on 2026-09-15.
const GIVE_UP_AFTER_MS: i32 = 60_000;

const THE_BODY: &str = "the message body";
/// The control after the body in the window's tab order.
/// `build_compose_dialog` makes the body, then the attachment line, a static
/// text that takes no keyboard, then the attachment list, hidden until a file
/// is attached, then Save Draft.
const AFTER_THE_BODY: &str = "Save Draft";
/// The control before the body: the toolbar is made before the fields, so
/// the last field made before the body is the Subject line.
const BEFORE_THE_BODY: &str = "the Subject line";

/// Lines the run writes from inside the event loop, read after it ends.
type Lines = Arc<Mutex<Vec<String>>>;

#[test]
fn test_leaving_the_body_forward_reaches_save_draft_and_backward_reaches_the_subject() {
    let readings: Lines = Arc::default();
    let wrong: Lines = Arc::default();
    let result = wxdragon::main({
        let readings = readings.clone();
        let wrong = wrong.clone();
        move |_| run_the_composer(readings, wrong)
    });
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");

    let readings = joined(&readings);
    println!("{readings}");
    let wrong = joined(&wrong);
    assert!(
        wrong.is_empty(),
        "leaving the message body went the wrong way:\n{wrong}\n--- where it went ---\n{readings}"
    );
}

/// Build the composer, wait for its browser, and take the two readings.
fn run_the_composer(readings: Lines, wrong: Lines) {
    let frame = Frame::builder().build();
    let composer = wx_compose::build_compose_dialog(
        &frame,
        "leaving the message body",
        &[FromEntry {
            account_id: "an-account".to_string(),
            address: "person@example.com".to_string(),
            sender_name: String::new(),
            said: "person@example.com".to_string(),
        }],
        0,
        None,
    );
    let body = composer.body_editor;
    composer.dialog.show(true);

    let giving_up = Timer::new(&frame);
    giving_up.on_tick(move |_| {
        eprintln!("the composer's browser had not reported after {GIVE_UP_AFTER_MS} ms");
        std::process::exit(2);
    });
    giving_up.start(GIVE_UP_AFTER_MS, true);
    std::mem::forget(giving_up);

    let steps = Rc::new(Timer::new(&frame));
    let ticks = Cell::new(0u32);
    steps.on_tick(move |_| {
        let tick = ticks.get();
        ticks.set(tick + 1);
        if !tick.is_multiple_of(TICKS_TO_SETTLE) {
            return;
        }
        if one_step(tick / TICKS_TO_SETTLE, &composer, &readings, &wrong) == Step::Last {
            // The frame is left to the end of the loop: this handler is the
            // frame's timer's, and destroying the frame inside it ended the
            // process with an access violation (measured 2026-10-25).
            composer.dialog.destroy();
            end_the_loop();
        }
    });
    body.on_created({
        let steps = steps.clone();
        move |_| {
            if !steps.start(TICK_MS, false) {
                eprintln!("the timer that takes the readings would not start");
                std::process::exit(2);
            }
        }
    });
    body.set_page("<p>A message somebody is writing.</p>", "about:blank");
    // Dropping the last handle destroys the timer, and it must outlive
    // `on_init`. The browser's handler holds the other one.
    std::mem::forget(steps);
}

#[derive(PartialEq)]
enum Step {
    More,
    Last,
}

/// One step of the run, each a settle apart.
///
/// `leave_the_message_body`'s second argument is `back`: false leaves
/// forward, true leaves backward.
fn one_step(step: u32, composer: &ComposeDialogWidgets, readings: &Lines, wrong: &Lines) -> Step {
    match step {
        0 => composer.body_editor.set_focus(),
        1 => leave(composer, false, wrong),
        2 => {
            landed(composer, "forward", AFTER_THE_BODY, readings, wrong);
            composer.body_editor.set_focus();
        }
        3 => leave(composer, true, wrong),
        _ => {
            landed(composer, "backward", BEFORE_THE_BODY, readings, wrong);
            return Step::Last;
        }
    }
    Step::More
}

/// Leave the body, once the keyboard is known to be in it.
fn leave(composer: &ComposeDialogWidgets, back: bool, wrong: &Lines) {
    let here = where_the_keyboard_is(composer);
    if here != THE_BODY {
        note(
            wrong,
            format!("the keyboard was on {here}, not in {THE_BODY}, before leaving it"),
        );
    }
    wx_compose::leave_the_message_body(&composer.body_editor, back);
}

/// Where leaving one way put the keyboard, against where it should be.
fn landed(
    composer: &ComposeDialogWidgets,
    way: &str,
    expected: &str,
    readings: &Lines,
    wrong: &Lines,
) {
    let here = where_the_keyboard_is(composer);
    note(readings, format!("leaving {way}: {here}"));
    if here != expected {
        note(
            wrong,
            format!("leaving {way} landed on {here}, not {expected}"),
        );
    }
}

/// The control the keyboard is on, by the name a person would give it.
fn where_the_keyboard_is(composer: &ComposeDialogWidgets) -> String {
    // SAFETY: no arguments; the answer is a handle or nought.
    let focus = unsafe { GetFocus() };
    let body = composer.body_editor.get_handle() as isize;
    // SAFETY: two window handles, either of which may be nought.
    if focus != 0 && (focus == body || unsafe { IsChild(body, focus) } != 0) {
        return THE_BODY.to_string();
    }
    let named = [
        (AFTER_THE_BODY, composer.draft_btn.get_handle()),
        (BEFORE_THE_BODY, composer.subject_field.get_handle()),
        ("Discard", composer.discard_btn.get_handle()),
        ("Cancel", composer.cancel_btn.get_handle()),
        ("Send", composer.send_toolbar_btn.get_handle()),
        ("the attachment list", composer.attachment_list.get_handle()),
    ];
    named
        .iter()
        .find(|(_, handle)| *handle as isize == focus)
        .map_or_else(
            || format!("a window this test has no name for ({focus:#x})"),
            |(name, _)| (*name).to_string(),
        )
}

fn note(into: &Lines, line: String) {
    if let Ok(mut lines) = into.lock() {
        lines.push(line);
    }
}

fn joined(lines: &Lines) -> String {
    lines.lock().map(|l| l.join("\n")).unwrap_or_default()
}

/// Queue the end of the event loop, asking for the application on the main
/// thread rather than carrying it across.
fn end_the_loop() {
    wxdragon::call_after(Box::new(|| {
        if let Some(app) = wxdragon::get_app_instance() {
            app.exit_main_loop();
        }
    }));
}

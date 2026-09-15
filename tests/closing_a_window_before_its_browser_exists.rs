//! Closing a window that holds a browser before WebView2 has made the browser.
//!
//! wxWidgets 3.3.2, which wxdragon 0.9.17 builds, hands WebView2 a bare
//! pointer to the control and never takes it back (wxWidgets #26491, fixed
//! upstream for 3.3.4, which is not released). Destroy the control while its
//! creation is still pending and the completion lands on freed memory.
//! Windows reports an exception escaping a COM callback as exit code
//! 0xc000041d and nothing else: no panic, no message, no test name. That is
//! what killed `tests/theme_reach.rs` on GitHub's runners on 2026-09-15 (runs
//! 34956059032 and 34961574447), where making a browser took over three
//! seconds and the test tore its windows down after two. On the machine this
//! was written on it takes a quarter of a second, and a scratch test that
//! tore down at once died the same way four times in four.
//!
//! `presentation::browser_ready` is the answer: a window holding a browser is
//! hidden at once and destroyed when the browser reports. The live half below
//! does exactly what somebody who opened Compose by mistake does, closes the
//! window before the browser exists, and keeps the event loop alive until the
//! browser has reported so the completion is delivered while the process is
//! there to receive it.
//!
//! The live half runs in a child process, because the failure this guards
//! against is not a failing assertion but a process that dies. A test target
//! that dies reports nothing; cargo prints the exit code and moves on, and
//! `--no-fail-fast` hid exactly that behind a real failure on the same CI run.
//! The parent reads the exit status and says what it means. It also checks
//! the child ran a test at all, because a filter matching nothing exits zero.
//!
//! The budget is one live window per process, so this file spends its one
//! `#[test]` function on the live half and the reading at the bottom touches
//! no wxWidgets API at all, as `tests/theme_reach.rs` explains at length.

use std::process::{Command, ExitStatus};
use wixen_mail::presentation::browser_ready::BrowserReady;
use wxdragon::prelude::*;
use wxdragon::widgets::{WebView, WebViewBackend};

const THE_LIVE_HALF: &str = "the_live_half_closes_a_window_at_once";

/// What Windows reports when an exception escapes a window procedure or a
/// COM completion: STATUS_FATAL_USER_CALLBACK_EXCEPTION.
const AN_EXCEPTION_ESCAPED_A_CALLBACK: u32 = 0xc000041d;

/// How long the live half gives the browser to report before saying so and
/// leaving. Generous: the runner took over three seconds on 2026-09-15, and a
/// run that hangs without a word is the one nobody reads.
const AT_MOST_BEFORE_GIVING_UP_MS: i32 = 60_000;

#[test]
fn test_closing_a_window_before_its_browser_exists_does_not_kill_the_process() {
    let me = std::env::current_exe().expect("the test binary knows its own path");
    let output = Command::new(&me)
        .args([
            "--ignored",
            "--exact",
            "--nocapture",
            "--test-threads=1",
            THE_LIVE_HALF,
        ])
        .output()
        .expect("the live half could be started");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "the live half died: {}.\n--- its stderr ---\n{stderr}\n--- its stdout ---\n{stdout}",
        what_the_exit_means(output.status)
    );
    assert!(
        stdout.contains("test result: ok. 1 passed"),
        "the live half ran no test, so this proved nothing:\n{stdout}"
    );
}

fn what_the_exit_means(status: ExitStatus) -> String {
    match status.code() {
        Some(code) if code as u32 == AN_EXCEPTION_ESCAPED_A_CALLBACK => format!(
            "exit code 0x{AN_EXCEPTION_ESCAPED_A_CALLBACK:08x}, an exception escaping a callback, \
             which is what a browser torn down before WebView2 finished making it looks like"
        ),
        Some(code) => format!("exit code {code} (0x{:08x})", code as u32),
        None => "no exit code".to_string(),
    }
}

#[test]
#[ignore = "the live half of the test above, which runs it in a child process"]
fn the_live_half_closes_a_window_at_once() {
    let result = wxdragon::main(move |app| {
        let frame = Frame::builder().build();
        let dialog = Dialog::builder(&frame, "closed before its browser exists").build();
        let view = WebView::builder(&dialog)
            .with_backend(WebViewBackend::Edge)
            .build();
        let browser = BrowserReady::watch(&view);
        view.set_page("<p>closed at once</p>", "about:blank");

        // Closed at once, as somebody who opened it by mistake would.
        browser.destroy_when_ready(dialog);

        // The loop stays alive until the browser has reported, so the
        // completion arrives while there is a process to arrive in. Queued
        // rather than called from the report, the way theme_reach exits.
        browser.when_ready(move || {
            wxdragon::call_after(Box::new(move || app.exit_main_loop()));
        });

        // A browser that never reports would hang this forever. Say so and
        // leave without tearing down, because tearing down is the crash.
        let giving_up = Timer::new(&frame);
        giving_up.on_tick(move |_| {
            eprintln!(
                "the browser had not reported after {AT_MOST_BEFORE_GIVING_UP_MS} ms; leaving \
                 without tearing the window down, since that is the crash this test is about"
            );
            std::process::exit(2);
        });
        giving_up.start(AT_MOST_BEFORE_GIVING_UP_MS, true);
        // Dropping a Timer destroys it, and this one must outlive on_init.
        std::mem::forget(giving_up);
    });
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");
}

// ── What cannot be built standalone ─────────────────────────────────────
//
// `show_compose_dialog_full` and `show_send_preview` are modal: they do not
// return until a person closes them, so no test can drive them. What a test
// can do is what `tests/theme_reach.rs` does for the panels it cannot build:
// read the source and confirm the close path is written the way it must be.
// Weaker than the live half above, and says so.

fn function_body<'a>(source: &'a str, signature_start: &str) -> &'a str {
    let start = source
        .find(signature_start)
        .unwrap_or_else(|| panic!("{signature_start} is not in the source"));
    let rest = &source[start..];
    let open = rest.find('{').expect("a function has a body");
    let mut depth = 0usize;
    for (offset, ch) in rest[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &rest[..open + offset + 1];
                }
            }
            _ => {}
        }
    }
    panic!("{signature_start} never closes its body");
}

#[test]
fn test_the_two_dialogs_holding_a_browser_are_closed_through_the_browser_watch() {
    let source = std::fs::read_to_string("src/presentation/wx_compose.rs")
        .expect("src/presentation/wx_compose.rs should be readable");
    for signature in ["fn show_compose_dialog_full(", "fn show_send_preview("] {
        let body = function_body(&source, signature);
        assert!(
            body.contains(".destroy_when_ready("),
            "{signature} does not hand its dialog to BrowserReady::destroy_when_ready"
        );
        assert!(
            !body.contains(".destroy();"),
            "{signature} still destroys its dialog directly, which is the crash"
        );
    }
}

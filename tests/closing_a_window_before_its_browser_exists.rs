//! Closing a window that holds a browser before WebView2 has made the browser.
//!
//! wxWidgets 3.3.2, which wxdragon 0.9.17 built, handed WebView2 a bare
//! pointer to the control and never took it back (wxWidgets #26491). Destroy
//! the control while its creation was still pending and the completion landed
//! on freed memory. Windows reports an exception escaping a COM callback as
//! exit code 0xc000041d and nothing else: no panic, no message, no test name.
//! That is what killed `tests/theme_reach.rs` on GitHub's runners on
//! 2026-09-15 (runs 34956059032 and 34961574447), where making a browser took
//! over three seconds and the test tore its windows down after two. From then
//! until 2026-10-25 the program hid such a window at once and destroyed it
//! only when its browser reported.
//!
//! wxWidgets 3.3.4, which wxdragon 0.9.23 builds, fixed the crash, so Compose
//! and Preview Before Send are now destroyed the moment they close. The live
//! half below does exactly what somebody who opened Compose by mistake does:
//! it builds the real composer, closes it before its browser exists, and keeps
//! the event loop alive past the moment the browser would have arrived, so a
//! completion that still reached the gone control would end the process here.
//! The moment is read from a second browser made just after the composer's:
//! once it has reported, the composer's would have too, and one more second
//! is kept on top.
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
use std::rc::Rc;
use std::time::Instant;
use wixen_mail::application::identities::FromEntry;
use wixen_mail::presentation::wx_compose;
use wxdragon::event::WebViewEvents;
use wxdragon::prelude::*;
use wxdragon::widgets::{WebView, WebViewBackend};

const THE_LIVE_HALF: &str = "the_live_half_closes_a_window_at_once";

/// What Windows reports when an exception escapes a window procedure or a
/// COM completion: STATUS_FATAL_USER_CALLBACK_EXCEPTION.
const AN_EXCEPTION_ESCAPED_A_CALLBACK: u32 = 0xc000041d;

/// How long the loop stays alive after the second browser has reported.
const KEPT_ALIVE_AFTER_THE_MOMENT_MS: i32 = 1_000;

/// How long the live half gives the second browser to report before saying
/// so and leaving. Generous: the runner took over three seconds on
/// 2026-09-15, and a run that hangs without a word is the one nobody reads.
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
    let result = wxdragon::main(move |_| {
        let frame = Frame::builder().build();
        let composer = wx_compose::build_compose_dialog(
            &frame,
            "closed before its browser exists",
            &[FromEntry {
                account_id: "an-account".to_string(),
                address: "person@example.com".to_string(),
                sender_name: String::new(),
                said: "person@example.com".to_string(),
            }],
            0,
            None,
        );
        composer
            .body_editor
            .set_page("<p>closed at once</p>", "about:blank");

        // Closed at once, as somebody who opened it by mistake would.
        composer.dialog.destroy();
        let destroyed = Instant::now();

        // Made after the composer's browser, so it reports after it would
        // have. Then one more second, and the loop ends.
        let pacer = WebView::builder(&frame)
            .with_backend(WebViewBackend::Edge)
            .build();
        let kept_alive = Rc::new(Timer::new(&frame));
        kept_alive.on_tick(move |_| {
            println!(
                "the loop stayed alive {} ms after the composer was destroyed",
                destroyed.elapsed().as_millis()
            );
            end_the_loop();
        });
        // A creation that failed reports through on_error, and counts too.
        pacer.on_created({
            let kept_alive = kept_alive.clone();
            move |_| {
                kept_alive.start(KEPT_ALIVE_AFTER_THE_MOMENT_MS, true);
            }
        });
        pacer.on_error({
            let kept_alive = kept_alive.clone();
            move |_| {
                kept_alive.start(KEPT_ALIVE_AFTER_THE_MOMENT_MS, true);
            }
        });
        pacer.set_page("<p>the moment has passed</p>", "about:blank");
        // Dropping the last handle destroys the timer, and it must outlive
        // on_init. The browser's handlers hold the other handles.
        std::mem::forget(kept_alive);

        // A browser that never reports would hang this forever. Say so.
        let giving_up = Timer::new(&frame);
        giving_up.on_tick(move |_| {
            eprintln!("the second browser had not reported after {AT_MOST_BEFORE_GIVING_UP_MS} ms");
            std::process::exit(2);
        });
        giving_up.start(AT_MOST_BEFORE_GIVING_UP_MS, true);
        std::mem::forget(giving_up);
    });
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");
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

// ── What cannot be built standalone ─────────────────────────────────────
//
// `show_compose_dialog_full` and `show_send_preview` are modal: they do not
// return until a person closes them, so no test can drive them. What a test
// can do is what `tests/theme_reach.rs` does for the panels it cannot build:
// read the source and confirm each close destroys its dialog there and then,
// handing it to nothing that waits for the browser. The live half above is
// where closing a window that early is shown to be safe; this reading only
// holds the two closes to doing it. Weaker than a live run of the product's
// own close, and says so.

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
fn test_the_two_dialogs_holding_a_browser_are_destroyed_when_they_close() {
    let source = std::fs::read_to_string("src/presentation/wx_compose.rs")
        .expect("src/presentation/wx_compose.rs should be readable");
    let closes = [
        ("fn show_compose_dialog_full(", "dialog"),
        ("fn show_send_preview(", "dlg"),
    ];
    let mut wrong = Vec::new();
    for (signature, dialog) in closes {
        let body = function_body(&source, signature);
        if !body.contains(&format!("{dialog}.destroy();")) {
            wrong.push(format!(
                "{signature} never destroys `{dialog}` when it closes"
            ));
        }
        if body.contains("when_ready(") {
            wrong.push(format!(
                "{signature} still hands `{dialog}` to a watch that waits for its browser"
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

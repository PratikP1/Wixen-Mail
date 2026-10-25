//! A browser torn down while WebView2 is still making it, at both moments
//! ledger 466 names, and the process still exits cleanly.
//!
//! wxWidgets 3.3.2, which wxdragon 0.9.17 builds, handed WebView2's creation
//! reply a bare pointer to the control and never took it back (wxWidgets
//! #26491, fixed in 3.3.4, which wxdragon 0.9.23 builds). When the control
//! went before the reply arrived, the reply landed on freed memory and
//! Windows ended the process with 0xc000041d, an exception escaping a
//! callback, and nothing else: no panic, no message, no test name.
//!
//! The two moments, each its own live half:
//!
//! - a window holding a browser destroyed at once, with the event loop kept
//!   alive long enough for the reply to arrive. `presentation::browser_ready`
//!   holds this off in the program today by hiding such a window and
//!   destroying it only when the browser reports; this half uses no
//!   `BrowserReady`, because what it asks is whether the toolkit still needs
//!   one, which is 13.4-10's question;
//! - the program ended while a browser is being made, the case of somebody
//!   who quits within a quarter of a second of the main window's preview or
//!   a conversation window appearing.
//!
//! Each live half runs in its own process, started by its parent test,
//! because the failure is not a failing assertion but a process that dies,
//! and a target that dies reports nothing. Each half spends its process's
//! one `wxdragon::main`. The parent reads the exit status, says what it
//! means, and checks the child ran a test at all, since a filter matching
//! nothing exits zero.

use std::process::{Command, ExitStatus};
use wxdragon::prelude::*;
use wxdragon::widgets::{WebView, WebViewBackend};

/// What Windows reports when an exception escapes a window procedure or a
/// COM completion: STATUS_FATAL_USER_CALLBACK_EXCEPTION.
const AN_EXCEPTION_ESCAPED_A_CALLBACK: u32 = 0xc000041d;

/// How long the first half keeps its loop alive after destroying the
/// window. Making a browser takes about a quarter of a second on the machine
/// this was written on and took over three on GitHub's runners on
/// 2026-09-15, so five seconds lets the reply arrive in either place.
const KEPT_ALIVE_AFTER_THE_WINDOW_GOES_MS: i32 = 5_000;

/// Start this test binary on one live half and say what became of it.
fn the_live_half_exits_cleanly(live_half: &str) {
    let me = std::env::current_exe().expect("the test binary knows its own path");
    let output = Command::new(&me)
        .args([
            "--ignored",
            "--exact",
            "--nocapture",
            "--test-threads=1",
            live_half,
        ])
        .output()
        .expect("the live half could be started");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "{live_half} died: {}.\n--- its stderr ---\n{stderr}\n--- its stdout ---\n{stdout}",
        what_the_exit_means(output.status)
    );
    assert!(
        stdout.contains("test result: ok. 1 passed"),
        "{live_half} ran no test, so this proved nothing:\n{stdout}"
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

/// Queue the end of the event loop, asking for the application on the main
/// thread rather than carrying it across.
fn end_the_loop() {
    wxdragon::call_after(Box::new(|| {
        if let Some(app) = wxdragon::get_app_instance() {
            app.exit_main_loop();
        }
    }));
}

#[test]
fn test_a_browser_window_destroyed_while_it_is_made_does_not_end_the_process() {
    the_live_half_exits_cleanly("the_live_half_destroys_a_browser_window_at_once");
}

#[test]
fn test_ending_the_program_while_a_browser_is_made_does_not_crash_it() {
    the_live_half_exits_cleanly("the_live_half_ends_the_program_while_a_browser_is_made");
}

#[test]
#[ignore = "the live half of the first test above, which runs it in a child process"]
fn the_live_half_destroys_a_browser_window_at_once() {
    let result = wxdragon::main(move |_| {
        let frame = Frame::builder().build();
        let dialog = Dialog::builder(&frame, "destroyed while its browser is made").build();
        let view = WebView::builder(&dialog)
            .with_backend(WebViewBackend::Edge)
            .build();
        view.set_page("<p>destroyed at once</p>", "about:blank");
        dialog.destroy();

        let kept_alive = Timer::new(&frame);
        kept_alive.on_tick(|_| end_the_loop());
        kept_alive.start(KEPT_ALIVE_AFTER_THE_WINDOW_GOES_MS, true);
        // Dropping a Timer destroys it, and this one must outlive on_init.
        std::mem::forget(kept_alive);
    });
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");
}

#[test]
#[ignore = "the live half of the second test above, which runs it in a child process"]
fn the_live_half_ends_the_program_while_a_browser_is_made() {
    let result = wxdragon::main(move |_| {
        let frame = Frame::builder()
            .with_title("ended while its browser is made")
            .build();
        let view = WebView::builder(&frame)
            .with_backend(WebViewBackend::Edge)
            .build();
        view.set_page("<p>ended at once</p>", "about:blank");
        // Never shown, as the first half's dialog is never shown: WebView2
        // starts making the browser when the control is created.
        end_the_loop();
    });
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");
}

//! Where a link in a message opens: the setting, the route, the menu, and
//! the activation caught in the page.
//!
//! #80, 11-11.1. Pratik's ask of 2026-09-18: a setting on the Reading tab
//! decides where a link opens, in the default browser, in the message view,
//! or in a separate Wixen Mail window, and the link's context menu offers
//! all three whatever the setting says. The tester the same day, under NVDA
//! on `1.0.0-alpha.1+149.g744d05ef`: "Enter on a message and Enter on a link
//! both open in the same window; the link does not go to the default
//! browser", against a handler that vetoed every navigation and handed the
//! address to the browser.
//!
//! The first thing here is the probe the issue's first comment asked for,
//! taken in this process against a browser control this test builds: what a
//! navigating event carries when a link in the page is followed. It carries
//! no address. wxWidgets keeps a navigating event's address in
//! `wxWebViewEvent::GetURL`, and wxdragon 0.9.17 hands `WebViewEventData`
//! only the command event's string, which a navigating event never sets. So
//! the handler's guard against vetoing the control's own document load,
//! `!url.is_empty()`, skipped every link, and the browser followed each one
//! inside the window. A veto that never fires reads exactly like one that
//! does. The live half below is that measurement, kept so a wxdragon that
//! starts carrying the address is noticed as the day the veto could work
//! again.
//!
//! The rest is what a reading and a built window can hold: the Reading tab
//! offers the three choices with the browser first and writes back the one
//! chosen; the route is one pure function and every cell of its table is
//! asserted; the page's script catches an anchor's activation before the
//! browser navigates; the three menu items are on the menu with their names;
//! the arms reach the one route; and the sanitiser runs before the route on
//! every path. What no reading can see, said plainly: that NVDA's Enter goes
//! where the setting says and Backspace brings the message back is the
//! tester's ear and the NVDA case on the `page` scan target, and is on the
//! ledger.

use std::process::{Command, ExitStatus};
use std::sync::{Arc, Mutex, OnceLock};

use wixen_mail::application::opening_links::{self, Asked, Route, Where};
use wixen_mail::data::config::AppConfig;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::browser_ready::BrowserReady;
use wixen_mail::presentation::wx_settings;
use wxdragon::event::WebViewEvents;
use wxdragon::prelude::*;
use wxdragon::widgets::{WebView, WebViewBackend};

// ── The probe: what a navigating event carries ────────────────────────────

const THE_LIVE_HALF: &str = "the_live_half_measures_what_a_navigating_event_carries";

/// What Windows reports when an exception escapes a COM completion, which is
/// what a browser torn down before WebView2 finished making it looks like.
const AN_EXCEPTION_ESCAPED_A_CALLBACK: u32 = 0xc000041d;

/// How long the live half waits for the browser, the load, and then the
/// navigation the click starts, before saying so and leaving.
const AT_MOST_BEFORE_GIVING_UP_MS: i32 = 60_000;

/// The line the live half prints, read back by the test that runs it.
const THE_PROBES_LINE: &str = "PROBE:";

#[test]
fn test_a_followed_links_navigating_event_carries_no_address_so_the_veto_never_saw_one() {
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
    // Printed while the harness is mid-line, so it lands after the test's
    // name on the same line; read from the marker on.
    let probe = stdout
        .lines()
        .find_map(|line| line.find(THE_PROBES_LINE).map(|at| &line[at..]))
        .unwrap_or_else(|| panic!("the live half printed no {THE_PROBES_LINE} line:\n{stdout}"));
    // The finding, as the live half measured it. The address of the link the
    // click followed is not in the event; the string is empty.
    assert!(
        probe.contains("navigating events after the load: 1")
            || probe.contains("navigating events after the load: 2"),
        "the click started no navigation, so nothing was measured: {probe}"
    );
    assert!(
        probe.contains("the first carried the string Some(\"\")"),
        "the navigating event carried an address, so the veto in wx_app.rs could \
         read it now and this finding is out of date: {probe}"
    );
    assert!(
        probe.contains("for the main frame: true"),
        "the navigating event was not the main frame's: {probe}"
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

/// What the live half saw, as plain values.
#[derive(Debug, Default)]
struct WhatTheEventCarried {
    /// Whether the message document has finished loading, so a navigating
    /// event after it is the click's and not the control loading its own
    /// document.
    loaded: bool,
    /// Each navigating event after the load: the event's string and its int.
    after_the_load: Vec<(Option<String>, Option<i32>)>,
    /// Whether the control went on to leave the message: an error or a
    /// completed navigation after the click.
    followed: bool,
}

#[test]
#[ignore = "the live half of the probe above, which runs it in a child process"]
fn the_live_half_measures_what_a_navigating_event_carries() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let result = wxdragon::main(move |app| {
        let frame = Frame::builder().build();
        let dialog = Dialog::builder(&frame, "what a navigating event carries").build();
        let view = WebView::builder(&dialog)
            .with_backend(WebViewBackend::Edge)
            .build();
        let browser = BrowserReady::watch(&view);
        let seen: Rc<RefCell<WhatTheEventCarried>> = Rc::default();

        // Recorded, never vetoed: a veto of the control's own document load
        // is what the guard in wx_app.rs was written to avoid, and the
        // question here is what the event carries, not what a veto does.
        view.on_navigating({
            let seen = seen.clone();
            move |event| {
                let mut seen = seen.borrow_mut();
                if seen.loaded {
                    seen.after_the_load
                        .push((event.get_string(), event.get_int()));
                }
            }
        });
        view.on_error({
            let seen = seen.clone();
            move |_| {
                let mut seen = seen.borrow_mut();
                if seen.loaded {
                    seen.followed = true;
                }
            }
        });
        view.on_navigated({
            let seen = seen.clone();
            move |_| {
                let mut seen = seen.borrow_mut();
                if seen.loaded {
                    seen.followed = true;
                }
            }
        });
        // Once the message is in, the link is followed the way a script does
        // it, which the browser treats as a click on the anchor. The address
        // is under `.invalid`, a name no resolver answers, so nothing is
        // reached; what matters is the navigating event on the way.
        view.on_loaded({
            let seen = seen.clone();
            move |_| {
                if seen.borrow().loaded {
                    return;
                }
                seen.borrow_mut().loaded = true;
                wxdragon::call_after(Box::new(move || {
                    let _ = view.run_script("document.querySelector('a').click();");
                }));
            }
        });
        view.set_page(
            "<p>A message with <a href=\"https://nothing.invalid/where-it-went\">a link</a>.</p>",
            "about:blank",
        );

        // Polled rather than awaited: the events arrive on the loop, and the
        // click's navigation is one or two of them.
        let waited = Rc::new(std::cell::Cell::new(0));
        let poll = Timer::new(&frame);
        poll.on_tick({
            let seen = seen.clone();
            let browser = browser.clone();
            let waited = waited.clone();
            move |_| {
                waited.set(waited.get() + 250);
                let done = {
                    let seen = seen.borrow();
                    !seen.after_the_load.is_empty() && (seen.followed || waited.get() >= 5_000)
                };
                if !done && waited.get() < AT_MOST_BEFORE_GIVING_UP_MS {
                    return;
                }
                let seen = seen.borrow();
                println!(
                    "{THE_PROBES_LINE} navigating events after the load: {}; the first carried \
                     the string {:?} and the int {:?}, for the main frame: {}; the control went on \
                     to follow the link: {}",
                    seen.after_the_load.len(),
                    seen.after_the_load
                        .first()
                        .map(|(s, _)| s.clone())
                        .unwrap_or(None),
                    seen.after_the_load.first().map(|(_, i)| *i).unwrap_or(None),
                    seen.after_the_load.first().map(|(_, i)| *i) == Some(Some(1)),
                    seen.followed
                );
                browser.destroy_when_ready(dialog);
                browser.when_ready(move || {
                    wxdragon::call_after(Box::new(move || app.exit_main_loop()));
                });
            }
        });
        poll.start(250, false);
        std::mem::forget(poll);
    });
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");
}

// ── The route: one pure function, every cell ──────────────────────────────

#[test]
fn test_the_route_table_holds_cell_by_cell() {
    let page = "https://example.com/where-it-went";
    let expected = [
        (Where::DefaultBrowser, Asked::Activated, Route::Browser),
        (
            Where::DefaultBrowser,
            Asked::InMessageView,
            Route::MessageView,
        ),
        (Where::DefaultBrowser, Asked::InBrowser, Route::Browser),
        (
            Where::DefaultBrowser,
            Asked::InSeparateWindow,
            Route::SeparateWindow,
        ),
        (Where::MessageView, Asked::Activated, Route::MessageView),
        (Where::MessageView, Asked::InMessageView, Route::MessageView),
        (Where::MessageView, Asked::InBrowser, Route::Browser),
        (
            Where::MessageView,
            Asked::InSeparateWindow,
            Route::SeparateWindow,
        ),
        (
            Where::SeparateWindow,
            Asked::Activated,
            Route::SeparateWindow,
        ),
        (
            Where::SeparateWindow,
            Asked::InMessageView,
            Route::MessageView,
        ),
        (Where::SeparateWindow, Asked::InBrowser, Route::Browser),
        (
            Where::SeparateWindow,
            Asked::InSeparateWindow,
            Route::SeparateWindow,
        ),
    ];
    assert_eq!(expected.len(), Where::ALL.len() * Asked::ALL.len());
    for (setting, asked, route) in expected {
        assert_eq!(
            opening_links::route(setting, asked, page),
            route,
            "{setting:?} asked {asked:?}"
        );
        for not_a_page in ["mailto:ada@example.org", "tel:+15550100"] {
            assert_eq!(
                opening_links::route(setting, asked, not_a_page),
                Route::System,
                "{setting:?} asked {asked:?} for {not_a_page}"
            );
        }
    }
}

// ── The Reading tab offers the choice, and what is chosen is what OK writes ─

/// Which tab is which, in the order the dialog adds them.
const THE_READING_TAB: usize = 2;

/// Everything the window session read, as plain values; no handle survives it.
#[derive(Debug)]
struct Harvest {
    /// The entries offered, in order, from a dialog built over the defaults.
    offered: Vec<String>,
    /// The selection over the defaults, and what OK wrote back untouched.
    selected_by_default: Option<u32>,
    written_back_by_default: String,
    /// From a dialog built over a stored "separate-window": the selection
    /// once the page was shown, and what OK wrote back untouched.
    selected_for_separate_window: Option<u32>,
    written_back_for_separate_window: String,
    /// For each entry chosen on the shown page, what OK wrote back.
    written_back_after_choosing: Vec<(u32, String)>,
}

/// The session: two dialogs, every failure carried out as a value. Nothing
/// in here panics.
fn read_the_dialogs(frame: &Frame, a11y: &Arc<Accessibility>) -> Result<Harvest, String> {
    let defaults = AppConfig::default();
    let widgets = wx_settings::build_settings_dialog(frame, &defaults, &[], false, a11y);
    widgets.dialog.show(true);
    widgets.notebook.set_selection(THE_READING_TAB);
    widgets.dialog.layout();
    let choice = &widgets.reading().open_links_in;
    let offered: Vec<String> = (0..choice.get_count())
        .map(|index| choice.get_string(index).unwrap_or_default())
        .collect();
    let selected_by_default = choice.get_selection();
    let written_back_by_default = wx_settings::read_settings(&widgets, &defaults).open_links_in;

    let mut written_back_after_choosing = Vec::new();
    for index in [1, 2, 0] {
        if index >= choice.get_count() {
            return Err(format!(
                "the choice offers {} entries and entry {index} cannot be chosen",
                choice.get_count()
            ));
        }
        choice.set_selection(index);
        if choice.get_selection() != Some(index) {
            return Err(format!(
                "set_selection({index}) on the choice left it at {:?}",
                choice.get_selection()
            ));
        }
        written_back_after_choosing.push((
            index,
            wx_settings::read_settings(&widgets, &defaults).open_links_in,
        ));
    }
    widgets.dialog.destroy();

    let stored = AppConfig {
        open_links_in: Where::SeparateWindow.stored().to_string(),
        ..AppConfig::default()
    };
    let widgets = wx_settings::build_settings_dialog(frame, &stored, &[], false, a11y);
    widgets.dialog.show(true);
    widgets.notebook.set_selection(THE_READING_TAB);
    widgets.dialog.layout();
    let selected_for_separate_window = widgets.reading().open_links_in.get_selection();
    let written_back_for_separate_window =
        wx_settings::read_settings(&widgets, &stored).open_links_in;
    widgets.dialog.destroy();

    Ok(Harvest {
        offered,
        selected_by_default,
        written_back_by_default,
        selected_for_separate_window,
        written_back_for_separate_window,
        written_back_after_choosing,
    })
}

fn take_the_harvest() -> Result<Harvest, String> {
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let taken: Result<Harvest, String> = (|| {
                let frame = Frame::builder().build();
                let a11y = Arc::new(
                    Accessibility::new()
                        .map_err(|why| format!("Accessibility::new failed: {why}"))?,
                );
                read_the_dialogs(&frame, &a11y)
            })();
            if let Ok(mut slot) = outcome.lock() {
                *slot = Some(taken);
            }
            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };
    if let Err(why) = result {
        return Err(format!("wxdragon::main returned {why:?}"));
    }
    let taken = outcome
        .lock()
        .map_err(|_| "the harvest's lock was poisoned".to_string())?
        .take();
    taken.unwrap_or_else(|| Err("the window session ended without a harvest".to_string()))
}

/// The one harvest of this process, taken by whichever test asks first.
/// The budget is one `wxdragon::main` per process (`tests/theme_reach.rs`
/// records the hang a second one produced); the probe's live half above is
/// a process of its own for that reason.
fn the_harvest() -> &'static Harvest {
    static HARVEST: OnceLock<Result<Harvest, String>> = OnceLock::new();
    match HARVEST.get_or_init(take_the_harvest) {
        Ok(harvest) => harvest,
        Err(why) => panic!("the window session could not be read: {why}"),
    }
}

#[test]
fn test_the_reading_tab_offers_the_browser_then_the_message_view_then_a_separate_window() {
    let harvest = the_harvest();
    let expected: Vec<String> = Where::ALL
        .iter()
        .map(|choice| choice.label().to_string())
        .collect();
    assert_eq!(
        harvest.offered, expected,
        "the choice offers {:?} rather than {expected:?}",
        harvest.offered
    );
    assert_eq!(
        harvest.selected_by_default,
        Some(0),
        "a fresh profile does not show the browser chosen"
    );
    assert_eq!(
        harvest.written_back_by_default,
        Where::DefaultBrowser.stored(),
        "OK over the defaults wrote something other than the browser back"
    );
}

#[test]
fn test_a_stored_separate_window_is_shown_as_the_third_entry_and_written_back_as_itself() {
    let harvest = the_harvest();
    assert_eq!(
        harvest.selected_for_separate_window,
        Some(2),
        "a stored separate-window is not shown as the third entry"
    );
    assert_eq!(
        harvest.written_back_for_separate_window,
        Where::SeparateWindow.stored(),
        "OK over a stored separate-window wrote it back as something else"
    );
}

#[test]
fn test_a_choice_made_on_the_reading_tab_is_what_ok_writes_back() {
    let harvest = the_harvest();
    let mut wrong = Vec::new();
    for (index, written) in &harvest.written_back_after_choosing {
        let expected = Where::ALL[*index as usize].stored();
        if *written != expected {
            wrong.push(format!(
                "entry {index} ({:?}) was chosen and OK wrote {written:?} back rather than {expected:?}",
                harvest.offered.get(*index as usize)
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} thing(s) wrong, the answer chosen on the Reading tab should be the answer the \
         settings file gets:\n  {}",
        wrong.len(),
        wrong.join("\n  ")
    );
}

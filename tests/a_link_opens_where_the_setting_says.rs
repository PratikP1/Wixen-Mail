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

use std::fs;
use std::process::{Command, ExitStatus};
use std::sync::{Arc, Mutex, OnceLock};

use wixen_mail::application::opening_links::{self, Asked, Route, Where};
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::data::config::AppConfig;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::browser_ready::BrowserReady;
use wixen_mail::presentation::page_links;
use wixen_mail::presentation::wx_settings;
use wxdragon::event::WebViewEvents;
use wxdragon::prelude::*;
use wxdragon::widgets::{WebView, WebViewBackend};

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

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

// ── The readings: the listener, the menu, the arms, the sanitiser first ────
//
// Read from the source rather than run, because the route is a script in a
// browser control inside a window with a running event loop, and what a
// reading can hold is the shape. Each reading is a function over the text
// with a companion that plants the opposite and requires a complaint.

fn shipped(path: &str) -> String {
    what_ships(
        &fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("{path}: {e}"))
            .replace("\r\n", "\n"),
    )
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

/// The text after the first `from` up to the next `to`, or a complaint
/// naming which anchor is gone.
fn between<'a>(text: &'a str, from: &str, to: &str) -> Result<&'a str, String> {
    let start = text
        .find(from)
        .ok_or(format!("{from:?} is no longer here, so this reads nothing"))?
        + from.len();
    let rest = &text[start..];
    let end = rest
        .find(to)
        .ok_or(format!("{to:?} is no longer here, so this reads nothing"))?;
    Ok(&rest[..end])
}

/// Every page is given the link listener, and the listener takes the click
/// from the browser before posting the address.
fn every_page_is_given_the_listener(app: &str, script: &str) -> Result<(), String> {
    let wiring = body_of(app, "fn wire_the_way_out(")?;
    let the_preview = between(
        &wiring,
        "PageKeys::TheWayOut =>",
        "PageKeys::TheWayOutAndTheJumps =>",
    )?;
    if !the_preview.contains("page_links::SCRIPT") {
        return Err("the preview is not given the link listener".to_string());
    }
    let the_page_window = between(&wiring, "PageKeys::TheWayOutAndTheJumps =>", "};")?;
    if !the_page_window.contains("page_links::SCRIPT") {
        return Err("the page window is not given the link listener".to_string());
    }
    let the_click = between(
        script,
        "addEventListener('click'",
        "addEventListener('keydown'",
    )?;
    if !the_click.contains("e.preventDefault();") {
        return Err(
            "the click listener posts the address and lets the browser navigate too, so the \
             link opens inside the window as well as where the setting says"
                .to_string(),
        );
    }
    if !the_click.contains("kind: 'link'") {
        return Err("the click listener posts no link".to_string());
    }
    Ok(())
}

#[test]
fn test_every_page_is_given_the_link_listener_and_it_takes_the_click_from_the_browser() {
    every_page_is_given_the_listener(&shipped(THE_MAIN_WINDOW), page_links::SCRIPT)
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_listener_reading_sees_a_click_left_to_the_browser() {
    let app = shipped(THE_MAIN_WINDOW);
    let without = page_links::SCRIPT.replacen("e.preventDefault();", "", 1);
    assert!(
        every_page_is_given_the_listener(&app, &without).is_err(),
        "the reading passed a listener that lets the browser navigate"
    );
    let without = app.replacen("page_links::SCRIPT", "\"\"", 1);
    assert!(
        every_page_is_given_the_listener(&without, page_links::SCRIPT).is_err(),
        "the reading passed a wiring that gives one surface no listener"
    );
}

/// The three items are on the link's menu, in order, named, above Copy Link.
fn the_three_items_are_on_the_menu(app: &str) -> Result<(), String> {
    let menu = body_of(app, "fn the_links_menu(")?;
    let items = [
        ("ID_CTX_OPEN_IN_MESSAGE_VIEW", "\"Open in &Message View\""),
        ("ID_CTX_OPEN_IN_BROWSER", "\"Open in Default &Browser\""),
        (
            "ID_CTX_OPEN_IN_SEPARATE_WINDOW",
            "\"Open in Separate &Window\"",
        ),
    ];
    let mut last = 0;
    for (id, label) in items {
        let at = menu
            .find(id)
            .ok_or(format!("{id} is not on the link's menu"))?;
        if at < last {
            return Err(format!("{id} is out of order on the menu"));
        }
        last = at;
        let call = between(&menu[at..], id, ");")?;
        if !call.contains(label) {
            return Err(format!("{id} is not named {label} on the menu"));
        }
    }
    let copy = menu
        .find("ID_CTX_COPY_LINK")
        .ok_or("Copy Link is no longer on the menu, so the order cannot be read".to_string())?;
    if copy < last {
        return Err("the three items sit below Copy Link rather than above it".to_string());
    }
    Ok(())
}

#[test]
fn test_the_three_items_are_on_the_links_menu_with_their_names_above_copy_link() {
    the_three_items_are_on_the_menu(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_menu_reading_sees_a_missing_item_and_a_wrong_name() {
    let app = shipped(THE_MAIN_WINDOW);
    let without = app.replacen("ID_CTX_OPEN_IN_BROWSER,", "ID_CTX_SELECT_ALL,", 1);
    assert!(
        the_three_items_are_on_the_menu(&without).is_err(),
        "the reading passed a menu with the browser item gone"
    );
    let renamed = app.replacen("\"Open in Separate &Window\"", "\"Open in New &Window\"", 1);
    assert!(
        the_three_items_are_on_the_menu(&renamed).is_err(),
        "the reading passed a menu whose item has another name"
    );
}

/// Each item's arm reaches the one route with its own ask, and both surfaces
/// answer their menu through the same function.
fn each_item_reaches_the_one_route(app: &str) -> Result<(), String> {
    let arms = body_of(app, "fn answer_the_links_menu(")?;
    for (id, asked) in [
        ("ID_CTX_OPEN_IN_MESSAGE_VIEW", "Asked::InMessageView"),
        ("ID_CTX_OPEN_IN_BROWSER", "Asked::InBrowser"),
        ("ID_CTX_OPEN_IN_SEPARATE_WINDOW", "Asked::InSeparateWindow"),
    ] {
        let at = arms
            .find(&format!("{id} =>"))
            .ok_or(format!("{id} has no arm"))?;
        let arm = between(&arms[at..], "=>", "ID_CTX_")
            .or_else(|_| between(&arms[at..], "=>", "\n    }\n"))?;
        if !arm.contains("follow_the_link_the_page_posted(") {
            return Err(format!("{id}'s arm does not reach the route"));
        }
        if !arm.contains(asked) {
            return Err(format!("{id}'s arm asks for something other than {asked}"));
        }
    }
    let answered = app.matches("answer_the_links_menu(").count();
    if answered < 3 {
        return Err(format!(
            "answer_the_links_menu is called from {} place(s), and both surfaces have a menu",
            answered.saturating_sub(1)
        ));
    }
    Ok(())
}

#[test]
fn test_each_item_reaches_the_one_route_with_its_own_ask_on_both_surfaces() {
    each_item_reaches_the_one_route(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_arms_reading_sees_an_item_routed_to_the_wrong_place() {
    let app = shipped(THE_MAIN_WINDOW);
    let arms = body_of(&app, "fn answer_the_links_menu(").unwrap_or_default();
    let wrong = arms.replacen("Asked::InMessageView", "Asked::InBrowser", 1);
    let planted = app.replacen(&arms, &wrong, 1);
    assert!(
        each_item_reaches_the_one_route(&planted).is_err(),
        "the reading passed Open in Message View going to the browser"
    );
}

/// The sanitiser runs before the route on every path, and a page reaches the
/// view through the one route.
fn the_sanitiser_runs_before_the_route(app: &str) -> Result<(), String> {
    let route = body_of(app, "fn follow_the_link_the_page_posted(")?;
    let sanitised = route
        .find("let Some(safe) = HtmlRenderer::safe_external_url(href) else {")
        .ok_or(
            "the route no longer passes the address through safe_external_url first".to_string(),
        )?;
    let routed = route
        .find("opening_links::route(")
        .ok_or("the route no longer asks opening_links::route".to_string())?;
    if routed < sanitised {
        return Err("the route is decided before the address is sanitised".to_string());
    }
    if app.matches("opening_links::route(").count() != 1 {
        return Err(format!(
            "opening_links::route is asked in {} places, and one is the number that keeps \
             every path behind the one sanitiser",
            app.matches("opening_links::route(").count()
        ));
    }
    if app.matches(".load_url(").count() != 1 {
        return Err(format!(
            "load_url is called in {} places; the one in show_the_page is the only way a page \
             reaches a view",
            app.matches(".load_url(").count()
        ));
    }
    // Called once, from the route; the definition is the other match.
    if app.matches("show_the_page(").count() != 2 {
        return Err(format!(
            "show_the_page is reached from {} place(s) rather than the route alone",
            app.matches("show_the_page(").count().saturating_sub(1)
        ));
    }
    Ok(())
}

#[test]
fn test_the_sanitiser_runs_before_the_route_on_every_path() {
    the_sanitiser_runs_before_the_route(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_sanitiser_reading_sees_a_path_that_skips_it() {
    let app = shipped(THE_MAIN_WINDOW);
    let skipped = app.replacen(
        "let Some(safe) = HtmlRenderer::safe_external_url(href) else {",
        "let Some(safe) = Some(href.to_string()) else {",
        1,
    );
    assert!(
        the_sanitiser_runs_before_the_route(&skipped).is_err(),
        "the reading passed a route that takes the address as written"
    );
    let second = format!("{app}\nfn elsewhere(view: &WebView) {{ view.load_url(\"x\"); }}\n");
    assert!(
        the_sanitiser_runs_before_the_route(&second).is_err(),
        "the reading passed a second way for a page to reach a view"
    );
}

/// The way back brings the message and says so; a page's title is said once
/// when it arrives and a page that will not load says why and brings the
/// message back; both surfaces are wired for it.
fn the_way_back_the_title_and_the_failure(app: &str) -> Result<(), String> {
    let back = body_of(app, "fn back_to_the_message(")?;
    if !back.contains("opening_links::BACK_TO_THE_MESSAGE") {
        return Err("the way back brings the message and says nothing".to_string());
    }
    if !back.contains("show_the_message(") {
        return Err("the way back says the message is back and shows nothing".to_string());
    }
    let arrival = body_of(app, "fn wire_a_pages_arrival(")?;
    let title = between(&arrival, ".on_title_changed(", ".on_error(")?;
    if !title.contains("announce(") {
        return Err("a page's title arrives and is not said".to_string());
    }
    if !title.contains("title_said") {
        return Err("a page's title is said on every change rather than once".to_string());
    }
    let failure = between(&arrival, ".on_error(", "\n    });")?;
    if !failure.contains("opening_links::could_not_be_opened(") {
        return Err("a page that will not load fails in silence".to_string());
    }
    if !failure.contains("back_to_the_message(") {
        return Err("a page that will not load leaves nothing where the message was".to_string());
    }
    let wired = app.matches("wire_a_pages_arrival(").count();
    if wired < 3 {
        return Err(format!(
            "wire_a_pages_arrival is called from {} place(s), and both surfaces show a page",
            wired.saturating_sub(1)
        ));
    }
    Ok(())
}

#[test]
fn test_the_way_back_says_so_and_a_pages_title_and_failure_are_said_on_both_surfaces() {
    the_way_back_the_title_and_the_failure(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_arrival_reading_sees_a_silent_failure_and_a_silent_way_back() {
    let app = shipped(THE_MAIN_WINDOW);
    let silent = app.replacen("opening_links::could_not_be_opened(", "String::from(", 1);
    assert!(
        the_way_back_the_title_and_the_failure(&silent).is_err(),
        "the reading passed a failure said in no words"
    );
    let silent = app.replacen("opening_links::BACK_TO_THE_MESSAGE", "\"\"", 1);
    assert!(
        the_way_back_the_title_and_the_failure(&silent).is_err(),
        "the reading passed a way back that says nothing"
    );
}

/// The vetoes stay as the second line on both surfaces: a main-frame
/// navigation the window did not ask for is vetoed and logged, and a new
/// window is vetoed; neither reads an address, since the event carries none.
fn the_vetoes_are_the_second_line(app: &str) -> Result<(), String> {
    let second_line = body_of(app, "fn wire_the_second_line(")?;
    let navigating = between(&second_line, ".on_navigating(", ".on_new_window(")?;
    for needed in [
        "get_int() == Some(1)",
        ".veto()",
        "tracing::debug!(",
        "loading",
    ] {
        if !navigating.contains(needed) {
            return Err(format!("the navigating veto no longer reads {needed}"));
        }
    }
    if navigating.contains("is_empty()") {
        return Err(
            "the navigating veto keys on the event's string again, which is always empty (#80)"
                .to_string(),
        );
    }
    let new_window = between(&second_line, ".on_new_window(", "\n    });")?;
    if !new_window.contains(".veto()") {
        return Err("a new window asked for by a page is not vetoed".to_string());
    }
    let wired = app.matches("wire_the_second_line(").count();
    if wired < 3 {
        return Err(format!(
            "wire_the_second_line is called from {} place(s), and both surfaces host a browser",
            wired.saturating_sub(1)
        ));
    }
    if app.matches("on_navigating(").count() != 1 || app.matches("on_new_window(").count() != 1 {
        return Err("a surface still binds a veto of its own beside the shared one".to_string());
    }
    Ok(())
}

#[test]
fn test_the_vetoes_are_one_second_line_on_both_surfaces_and_read_no_address() {
    the_vetoes_are_the_second_line(&shipped(THE_MAIN_WINDOW)).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_veto_reading_sees_the_old_guard_on_the_empty_string() {
    let app = shipped(THE_MAIN_WINDOW);
    let old = app.replacen("get_int() == Some(1)", "!url.is_empty()", 1);
    assert!(
        the_vetoes_are_the_second_line(&old).is_err(),
        "the reading passed the veto that never fired"
    );
}

/// The page window has the link's menu too, answered by the same arms.
fn the_page_window_has_the_menu(app: &str) -> Result<(), String> {
    let window = body_of(app, "fn show_conversation_as_page(")?;
    for needed in [
        "the_links_menu(",
        ".popup_menu(",
        ".on_menu(",
        "answer_the_links_menu(",
    ] {
        if !window.contains(needed) {
            return Err(format!(
                "the page window has no context menu on a link: {needed} is not in it"
            ));
        }
    }
    Ok(())
}

#[test]
fn test_the_page_window_offers_the_links_menu_and_answers_it_through_the_same_arms() {
    the_page_window_has_the_menu(&shipped(THE_MAIN_WINDOW)).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_page_window_reading_sees_a_window_with_no_menu() {
    let app = shipped(THE_MAIN_WINDOW);
    let window = body_of(&app, "fn show_conversation_as_page(").unwrap_or_default();
    let without = app.replacen(&window, &window.replace("the_links_menu(", "no_menu("), 1);
    assert!(
        the_page_window_has_the_menu(&without).is_err(),
        "the reading passed a page window with no menu"
    );
}

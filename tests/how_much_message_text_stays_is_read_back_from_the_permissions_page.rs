//! How much message text stays on this computer is a choice on the
//! Permissions tab, and what a person chooses there is what OK writes back.
//!
//! #23, 10-03: the setting `message_text_kept` is offered under Message Text
//! beside the box that forbids fetching text, as four words, All of it first.
//! The two settings guards in `src/data/config.rs` hold that the field's
//! name appears in the settings screen's shipping half and in a file that
//! acts on it; neither can tell the line that builds the control from the
//! line that reads it back, so a `read_the_permissions_page` that stopped
//! writing the field would leave both green. This reading builds the real
//! dialog, chooses a size on the shown Permissions page, reads the settings
//! back through `read_settings` the way OK does, and holds the answer to the
//! choice. It also holds the other half of the lazy build: a Permissions page
//! nobody showed leaves the stored size alone, because the page's controls
//! do not exist to be read.
//!
//! The other end of the setting is read from the source: the eviction has
//! one caller, the end of a folder sync, and the folder sync has two callers
//! in the window, the check and the whole-folder request, so those two are
//! the workers that evict and each has to open its cache through
//! `keeping_bodies_under` with the setting. The guard that requires a reader
//! outside the settings files sees the field's name anywhere in the window's
//! shipping half and cannot tell a worker's cache line from the helper that
//! reads the file, so a worker that stopped handing its cache the setting
//! would leave it green; this reads each worker's body.
//!
//! The budget is one `wxdragon::main` per process (`tests/theme_reach.rs`
//! records the "initializing twice?" hang a second one produced), so the two
//! dialogs are built once inside a `OnceLock` by whichever test asks first,
//! every reading is harvested into plain values, and the tests assert over
//! the harvest. The initialiser stores a `Result` and panics on nothing
//! itself: a panic inside `OnceLock::get_or_init` leaves the cell empty and
//! the next test would spend the budget a second time.

#![cfg(windows)]

use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::keeping_message_text::{GIGABYTE, TextKept};
use wixen_mail::data::config::AppConfig;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::wx_settings;
use wxdragon::prelude::*;

/// Which tab is which, in the order the dialog adds them.
const THE_PERMISSIONS_TAB: usize = 3;

/// A size the list offers, stored, so the dialog has something other than
/// the default to show and to leave alone.
const FIVE_GIGABYTES: &str = "5000000000";

/// Everything the window session read, as plain values; no handle survives it.
#[derive(Debug)]
struct Harvest {
    /// What OK wrote back before the Permissions tab was ever shown, from a
    /// dialog built over a stored five gigabytes.
    written_back_before_the_page_was_shown: String,
    /// The choice's selection once the page was shown, over that stored value.
    selected_for_five_gigabytes: Option<u32>,
    /// The four entries offered, in order.
    offered: Vec<String>,
    /// For each entry chosen on the shown page, what OK wrote back.
    written_back_after_choosing: Vec<(u32, String)>,
    /// From a second dialog built over the defaults: the selection and what
    /// OK wrote back with nothing chosen.
    selected_by_default: Option<u32>,
    written_back_by_default: String,
}

/// The session: two dialogs, every failure carried out as a value. Nothing
/// in here panics.
fn read_the_dialogs(frame: &Frame, a11y: &Arc<Accessibility>) -> Result<Harvest, String> {
    let stored = AppConfig {
        message_text_kept: FIVE_GIGABYTES.to_string(),
        ..AppConfig::default()
    };
    let widgets = wx_settings::build_settings_dialog(frame, &stored, &[], false, a11y);
    widgets.dialog.show(true);

    let written_back_before_the_page_was_shown =
        wx_settings::read_settings(&widgets, &stored).message_text_kept;

    widgets.notebook.set_selection(THE_PERMISSIONS_TAB);
    widgets.dialog.layout();
    let choice = &widgets.permissions().message_text_kept;
    let selected_for_five_gigabytes = choice.get_selection();
    let offered: Vec<String> = (0..choice.get_count())
        .map(|index| choice.get_string(index).unwrap_or_default())
        .collect();

    let mut written_back_after_choosing = Vec::new();
    for index in [0, 1, 3, 2] {
        choice.set_selection(index);
        if choice.get_selection() != Some(index) {
            return Err(format!(
                "set_selection({index}) on the choice left it at {:?}",
                choice.get_selection()
            ));
        }
        written_back_after_choosing.push((
            index,
            wx_settings::read_settings(&widgets, &stored).message_text_kept,
        ));
    }
    widgets.dialog.destroy();

    let defaults = AppConfig::default();
    let widgets = wx_settings::build_settings_dialog(frame, &defaults, &[], false, a11y);
    widgets.dialog.show(true);
    widgets.notebook.set_selection(THE_PERMISSIONS_TAB);
    let selected_by_default = widgets.permissions().message_text_kept.get_selection();
    let written_back_by_default = wx_settings::read_settings(&widgets, &defaults).message_text_kept;
    widgets.dialog.destroy();

    Ok(Harvest {
        written_back_before_the_page_was_shown,
        selected_for_five_gigabytes,
        offered,
        written_back_after_choosing,
        selected_by_default,
        written_back_by_default,
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
fn the_harvest() -> &'static Harvest {
    static HARVEST: OnceLock<Result<Harvest, String>> = OnceLock::new();
    match HARVEST.get_or_init(take_the_harvest) {
        Ok(harvest) => harvest,
        Err(why) => panic!("the window session could not be read: {why}"),
    }
}

fn complain(what: &str, wrong: &[String]) {
    assert!(
        wrong.is_empty(),
        "{} thing(s) wrong, {what}:\n  {}",
        wrong.len(),
        wrong.join("\n  ")
    );
}

#[test]
fn test_a_size_chosen_on_the_permissions_page_is_what_ok_writes_back() {
    let harvest = the_harvest();
    let mut wrong = Vec::new();
    for (index, written) in &harvest.written_back_after_choosing {
        let expected = TextKept::ALL[*index as usize].as_stored();
        if *written != expected {
            wrong.push(format!(
                "entry {index} ({:?}) was chosen and OK wrote {written:?} back rather than {expected:?}",
                harvest.offered.get(*index as usize)
            ));
        }
    }
    complain(
        "the size chosen on the Permissions page should be the size the settings file gets",
        &wrong,
    );
}

#[test]
fn test_the_stored_size_is_selected_when_the_page_is_shown_and_left_alone_when_it_is_not() {
    let harvest = the_harvest();
    let mut wrong = Vec::new();
    if harvest.written_back_before_the_page_was_shown != FIVE_GIGABYTES {
        wrong.push(format!(
            "OK on a dialog whose Permissions tab was never shown wrote {:?} over the stored {FIVE_GIGABYTES}",
            harvest.written_back_before_the_page_was_shown
        ));
    }
    if harvest.selected_for_five_gigabytes != Some(2) {
        wrong.push(format!(
            "a stored {FIVE_GIGABYTES} selected entry {:?} rather than 2, {:?}",
            harvest.selected_for_five_gigabytes,
            TextKept::UpTo(5 * GIGABYTE).label()
        ));
    }
    complain(
        "a stored size should select its own entry once the page is shown, and stay as stored \
         while it is not",
        &wrong,
    );
}

#[test]
fn test_the_dialog_offers_all_of_it_first_and_the_default_writes_all_back() {
    let harvest = the_harvest();
    let mut wrong = Vec::new();
    let expected: Vec<String> = TextKept::ALL.iter().map(|choice| choice.label()).collect();
    if harvest.offered != expected {
        wrong.push(format!(
            "the choice offers {:?} rather than {expected:?}",
            harvest.offered
        ));
    }
    if harvest.selected_by_default != Some(0) {
        wrong.push(format!(
            "a dialog built over the defaults selected entry {:?} rather than 0, All of it",
            harvest.selected_by_default
        ));
    }
    if harvest.written_back_by_default != "all" {
        wrong.push(format!(
            "OK with nothing chosen wrote {:?} back rather than \"all\"",
            harvest.written_back_by_default
        ));
    }
    complain(
        "All of it should be offered first and be what an untouched dialog writes back",
        &wrong,
    );
}

/// The shipping half of one source file, read from the repository root.
fn what_ships_in(path: &str) -> String {
    let source = std::fs::read_to_string(path)
        .unwrap_or_else(|why| panic!("{path} could not be read: {why}"));
    wixen_mail::common::what_ships::what_ships(&source)
}

/// The body of one function: from its `fn name(` to the next line that is
/// exactly `}`, the shape `tests/the_settings_dialog_opens_in.rs` reads with.
fn body_of<'a>(source: &'a str, name: &str) -> &'a str {
    let opening = format!("fn {name}(");
    let from = source
        .find(&opening)
        .unwrap_or_else(|| panic!("{opening} is not in the source"));
    let rest = &source[from..];
    let to = rest
        .find("\n}\n")
        .unwrap_or_else(|| panic!("{opening} has no closing brace on a line of its own"));
    &rest[..to]
}

#[test]
fn test_the_two_workers_that_evict_are_handed_the_setting_and_nothing_else_evicts() {
    let sync = what_ships_in("src/application/mail_sync.rs");
    let window = what_ships_in("src/presentation/wx_app.rs");
    let mut wrong = Vec::new();

    // The eviction has one caller, the end of a folder sync, so the caches
    // that evict are the caches a folder sync runs on.
    let evictions = sync.matches("keep_bodies_within_budget()").count();
    if evictions != 1 {
        wrong.push(format!(
            "keep_bodies_within_budget() is called {evictions} times in mail_sync.rs, not once at \
             the end of sync_folder, so the workers that evict are no longer the two read below"
        ));
    }

    // The folder sync has two callers in the window, and each opens its
    // cache with the setting, read once before the cache is built.
    let workers = ["spawn_mail_sync", "spawn_whole_folder_fetch"];
    let callers = window.matches("mail_sync::sync_folder(").count();
    if callers != workers.len() {
        wrong.push(format!(
            "sync_folder is called {callers} times in the window's shipping half, and the two \
             workers read below are {}",
            workers.len()
        ));
    }
    for worker in workers {
        let body = body_of(&window, worker);
        for expected in [
            "mail_sync::sync_folder(",
            "how_much_message_text_stays()",
            ".keeping_bodies_under(",
            "text_kept.budget()",
        ] {
            if !body.contains(expected) {
                wrong.push(format!("{worker} does not contain {expected:?}"));
            }
        }
    }

    complain(
        "the two workers that evict should each open their cache with how much text stays, and \
         nothing else should evict",
        &wrong,
    );
}

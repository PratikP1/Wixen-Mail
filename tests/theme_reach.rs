//! Whether the theme's colours reach the windows this round wires up.
//!
//! Every other test in this crate proves structure or reads source text,
//! because building a real wxWidgets window has never been done inside a
//! test here before. `WxWidget::get_background_color` and
//! `get_foreground_color` round-trip through the real native handle, and a
//! window has one the moment `.build()` returns, well before it is shown. So
//! this file does what nothing else in the crate does: builds real controls
//! and reads back the real colour a real control is holding.
//!
//! wxWidgets supports exactly one application per process, and `cargo test`
//! runs the `#[test]` functions of one file in parallel threads by default.
//! The first version of this file had two functions that each called
//! `wxdragon::main`; wxWidgets asserted "initializing twice?" and the run
//! hung until it was killed. So building a live widget and reading it back
//! happens in exactly one `#[test]` function below, including the check that
//! the harness itself works, never split across more than one, and never
//! inside `src/**`'s own test modules.
//!
//! This is the first of three steps this round takes through this file: the
//! five PIM modules first, then the reader window, then the sites that stay
//! inside `wx_app.rs` and cannot be constructed standalone at all.

use std::sync::{Arc, Mutex};
use wixen_mail::presentation::theme::{self, Theme};
use wixen_mail::presentation::{
    wx_calendar_module, wx_contacts_module, wx_notes_module, wx_reminders_module, wx_tasks_module,
};
use wxdragon::prelude::*;

/// Proves the harness itself works, as the first site in the consolidated
/// test below rather than as a `#[test]` of its own (see the file comment
/// for why it cannot be the latter).
///
/// Not a red/green pair on its own: `theme::paint` and the palette it reads
/// from are already correct and already tested elsewhere as arithmetic. This
/// asks a narrower question, on the real target this round ships to: does a
/// colour handed to a real control come back out unchanged? If this is red,
/// nothing else in the same run can be trusted. Confirmed by running it the
/// direct way first, calling `exit_main_loop` before the loop it was meant to
/// stop had started: it hung past 60 seconds and had to be killed, which is
/// why the call below is queued through `call_after` instead.
fn check_harness_itself(frame: &Frame, into: &mut Vec<SiteResult>) {
    let light = Theme::Light
        .palette(false)
        .expect("Theme::Light always has a palette");
    let panel = Panel::builder(frame).build();
    theme::paint(&panel, light.main_surface());
    check("harness self-check", &panel, light.main_surface(), into);
}

/// One site's outcome: its name, whether it passed, and detail for a failure
/// message. A `Vec` of these rather than an assertion per site, so one
/// mismatch does not stop the rest of the sites in the same run from being
/// checked too, and a failure names exactly which site it was.
type SiteResult = (&'static str, bool, String);

/// Whether a live control's background and foreground match a surface,
/// recorded rather than asserted immediately. `impl WxWidget` rather than a
/// concrete type, because a `Panel`, a `TreeCtrl` and a `TextCtrl` all
/// implement it and none of this cares which one it is holding.
fn check(
    name: &'static str,
    widget: &impl WxWidget,
    want: theme::Surface,
    into: &mut Vec<SiteResult>,
) {
    let want_bg = Colour::rgb(want.background.r, want.background.g, want.background.b);
    let want_fg = Colour::rgb(want.text.r, want.text.g, want.text.b);
    let got_bg = widget.get_background_color();
    let got_fg = widget.get_foreground_color();
    let ok = got_bg.r == want_bg.r
        && got_bg.g == want_bg.g
        && got_bg.b == want_bg.b
        && got_fg.r == want_fg.r
        && got_fg.g == want_fg.g
        && got_fg.b == want_fg.b;
    into.push((
        name,
        ok,
        format!(
            "background: got {got_bg:?} want {want_bg:?}; foreground: got {got_fg:?} want {want_fg:?}"
        ),
    ));
}

fn check_calendar(parent: &Panel, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let sidebar = wx_calendar_module::build_calendar_sidebar(parent, Some(palette));
    check(
        "calendar sidebar panel",
        &sidebar.panel,
        palette.second_surface(),
        into,
    );
    check(
        "calendar sidebar tree",
        &sidebar.tree,
        palette.second_surface(),
        into,
    );

    let content = wx_calendar_module::build_calendar_panel(parent, Some(palette));
    check(
        "calendar content panel",
        &content.panel,
        palette.main_surface(),
        into,
    );
    check(
        "calendar content event list",
        &content.event_list,
        palette.main_surface(),
        into,
    );
}

fn check_contacts(parent: &Panel, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let sidebar = wx_contacts_module::build_contacts_sidebar(parent, Some(palette));
    check(
        "contacts sidebar panel",
        &sidebar.panel,
        palette.second_surface(),
        into,
    );
    check(
        "contacts sidebar tree",
        &sidebar.tree,
        palette.second_surface(),
        into,
    );

    let content = wx_contacts_module::build_contacts_panel(parent, Some(palette));
    check(
        "contacts content panel",
        &content.panel,
        palette.main_surface(),
        into,
    );
    check(
        "contacts content list",
        &content.contact_list,
        palette.main_surface(),
        into,
    );
    check(
        "contacts content detail",
        &content.detail,
        palette.main_surface(),
        into,
    );
}

fn check_reminders(parent: &Panel, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let sidebar = wx_reminders_module::build_reminders_sidebar(parent, Some(palette));
    check(
        "reminders sidebar panel",
        &sidebar.panel,
        palette.second_surface(),
        into,
    );
    check(
        "reminders sidebar tree",
        &sidebar.tree,
        palette.second_surface(),
        into,
    );

    let content = wx_reminders_module::build_reminders_panel(parent, Some(palette));
    check(
        "reminders content panel",
        &content.panel,
        palette.main_surface(),
        into,
    );
    check(
        "reminders content list",
        &content.reminder_list,
        palette.main_surface(),
        into,
    );
}

fn check_tasks(parent: &Panel, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let sidebar = wx_tasks_module::build_tasks_sidebar(parent, Some(palette));
    check(
        "tasks sidebar panel",
        &sidebar.panel,
        palette.second_surface(),
        into,
    );
    check(
        "tasks sidebar tree",
        &sidebar.tree,
        palette.second_surface(),
        into,
    );

    let content = wx_tasks_module::build_tasks_panel(parent, Some(palette));
    check(
        "tasks content panel",
        &content.panel,
        palette.main_surface(),
        into,
    );
    check(
        "tasks content list",
        &content.task_list,
        palette.main_surface(),
        into,
    );
}

fn check_notes(parent: &Panel, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let sidebar = wx_notes_module::build_notes_sidebar(parent, Some(palette));
    check(
        "notes sidebar panel",
        &sidebar.panel,
        palette.second_surface(),
        into,
    );
    check(
        "notes sidebar tree",
        &sidebar.tree,
        palette.second_surface(),
        into,
    );

    let content = wx_notes_module::build_notes_panel(parent, Some(palette));
    check(
        "notes content panel",
        &content.panel,
        palette.main_surface(),
        into,
    );
    check(
        "notes content list panel",
        &content.list_panel,
        palette.main_surface(),
        into,
    );
    check(
        "notes content note list",
        &content.note_list,
        palette.main_surface(),
        into,
    );
    check(
        "notes content editor panel",
        &content.editor_panel,
        palette.main_surface(),
        into,
    );
    check(
        "notes content title input",
        &content.title_input,
        palette.main_surface(),
        into,
    );
    check(
        "notes content body input",
        &content.body_input,
        palette.main_surface(),
        into,
    );
}

/// Every site this round wires that can be reached without a running
/// application, checked against the real colour a real control reports.
///
/// One test function, because wxWidgets supports exactly one application per
/// process and this is the file's only one that builds a live widget. A
/// failure names the exact site: the assertion below lists every one that did
/// not carry the colour it was given, rather than stopping at the first.
#[test]
fn test_every_site_this_round_reaches_carries_the_colour_a_live_control_reports() {
    let results: Arc<Mutex<Vec<SiteResult>>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let results = results.clone();
        wxdragon::main(move |app| {
            let palette = Theme::Light
                .palette(false)
                .expect("Theme::Light always has a palette");

            let frame = Frame::builder().build();
            let parent = Panel::builder(&frame).build();

            let mut sites = Vec::new();
            check_harness_itself(&frame, &mut sites);
            check_calendar(&parent, palette, &mut sites);
            check_contacts(&parent, palette, &mut sites);
            check_reminders(&parent, palette, &mut sites);
            check_tasks(&parent, palette, &mut sites);
            check_notes(&parent, palette, &mut sites);

            *results.lock().unwrap() = sites;

            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };

    assert!(result.is_ok(), "wxdragon::main returned {result:?}");
    let sites = results.lock().unwrap();
    assert!(
        !sites.is_empty(),
        "on_init never ran, so nothing was checked"
    );
    let failed: Vec<String> = sites
        .iter()
        .filter(|(_, ok, _)| !ok)
        .map(|(name, _, detail)| format!("{name} -- {detail}"))
        .collect();
    assert!(
        failed.is_empty(),
        "{} site(s) did not carry the colour they were given:\n{}",
        failed.len(),
        failed.join("\n")
    );
}

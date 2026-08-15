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
//! inside `src/**`'s own test modules. The two source-text checks at the
//! bottom of this file touch no wxWidgets API at all, so running them
//! alongside the live one in the same process is not that hazard.
//!
//! Two call sites this round wires are not reachable from here at all:
//! `wx_app.rs`'s own local panels (the module switcher, the mail sidebar and
//! content panels, the splitter) and `show_conversation_as_page`'s frame and
//! attachment list are built deep inside functions that are private, or that
//! need a running application's accounts, database and runtime to reach.
//! Making them independently constructible was not part of this round's
//! remit, so those sites are proven the way `tests/wired.rs` proves the
//! things it cannot construct either: by reading the source and confirming
//! the call is written where it should be, which is weaker than a live
//! read-back and says so.

use std::sync::{Arc, Mutex};
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::reader_text::{ReaderAttachment, ReaderDocument};
use wixen_mail::presentation::theme::{self, Theme};
use wixen_mail::presentation::{
    wx_calendar_module, wx_contacts_module, wx_notes_module, wx_reader, wx_reminders_module,
    wx_tasks_module,
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
        "contacts content search",
        &content.search_input,
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

/// A message with both a warning and an attachment, so the reader builds
/// both of its optional tab widgets and this can check them rather than
/// finding `None` and having nothing to read a colour from.
fn document_that_exercises_every_optional_widget() -> ReaderDocument {
    ReaderDocument {
        title: "Test message".to_string(),
        text: "Subject: Test message\n\nThis is a test.".to_string(),
        landmarks: Vec::new(),
        warning: Some("This message could not be verified.".to_string()),
        attachments: vec![ReaderAttachment {
            message_row_id: 1,
            uid: 1,
            index: 0,
            name: "report.pdf".to_string(),
            mime_type: "application/pdf".to_string(),
            size: 1024,
        }],
    }
}

fn check_reader(parent: &Frame, a11y: &Arc<Accessibility>, into: &mut Vec<SiteResult>) {
    let reader = wx_reader::ReaderWindow::new(parent, a11y);

    // The reader reads its own palette rather than taking one as a
    // parameter, the same as `show_conversation_as_page` and for the same
    // reason given on both (see their doc comments). So this test asks the
    // same question production code does, instead of assuming a fixed
    // answer: whatever `theme::current` reports right now, on whatever
    // machine is running this suite, is what a freshly built reader should
    // show. `theme.rs`'s own tests follow the same rule for the same reason:
    // a test that asserted a fixed light or dark answer would go red the
    // moment Pratik switched high contrast on to do an accessibility pass,
    // which is the one time this suite must not be lying to him.
    let palette = theme::current(
        &wixen_mail::data::config::ConfigManager::load_stored()
            .map(|mgr| mgr.app_config().theme.clone())
            .unwrap_or_default(),
    );
    match palette {
        Some(palette) => {
            check("reader frame", reader.frame(), palette.main_surface(), into);
            check(
                "reader notebook",
                reader.notebook(),
                palette.main_surface(),
                into,
            );

            let handles = reader.open(document_that_exercises_every_optional_widget());
            check(
                "reader tab panel",
                &handles.panel,
                palette.main_surface(),
                into,
            );
            check(
                "reader tab text",
                &handles.text,
                palette.main_surface(),
                into,
            );
            match &handles.warning {
                Some(bar) => check("reader tab warning", bar, palette.main_surface(), into),
                None => into.push((
                    "reader tab warning",
                    false,
                    "no warning bar was built for a document with a warning".to_string(),
                )),
            }
            match &handles.attachments {
                Some(list) => check("reader tab attachments", list, palette.main_surface(), into),
                None => into.push((
                    "reader tab attachments",
                    false,
                    "no attachment list was built for a document with an attachment".to_string(),
                )),
            }
        }
        None => {
            // High contrast is on wherever this suite is running. Nothing
            // should be painted, and there is no fixed colour to compare
            // against, so this records that the branch was reached rather
            // than asserting a colour nobody can predict from here.
            for site in [
                "reader frame",
                "reader notebook",
                "reader tab panel",
                "reader tab text",
                "reader tab warning",
                "reader tab attachments",
            ] {
                into.push((
                    site,
                    true,
                    "high contrast is on for this run: nothing is painted, which is correct"
                        .to_string(),
                ));
            }
        }
    }

    // Whether `repaint` moves a window that is already built, already open
    // and already showing one palette to a different one immediately, with
    // nothing torn down and rebuilt: the guarantee the Theme setting's own
    // live handler depends on. `Theme::Light` and `Theme::Dark` directly,
    // not `theme::current`, so this cannot read the same answer on both
    // sides and pass by accident on a machine that happens to be running
    // high contrast.
    let light = Theme::Light
        .palette(false)
        .expect("Theme::Light always has a palette");
    let dark = Theme::Dark
        .palette(false)
        .expect("Theme::Dark always has a palette");

    reader.repaint(Some(light));
    check(
        "reader frame repainted to light on the same window",
        reader.frame(),
        light.main_surface(),
        into,
    );

    reader.repaint(Some(dark));
    check(
        "reader frame repainted to dark on the same window, not rebuilt",
        reader.frame(),
        dark.main_surface(),
        into,
    );
    check(
        "reader notebook repainted to dark on the same window, not rebuilt",
        reader.notebook(),
        dark.main_surface(),
        into,
    );

    // A tab opened after `repaint` uses the palette it was given, not the
    // one the window was built with: the same guarantee, for the tabs
    // `repaint` cannot reach directly because it keeps hold of none of
    // them (see its own doc comment).
    let after = reader.open(document_that_exercises_every_optional_widget());
    check(
        "reader tab text opened after repaint uses the new palette",
        &after.text,
        dark.main_surface(),
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

            let a11y = Arc::new(Accessibility::new().expect("accessibility"));
            check_reader(&frame, &a11y, &mut sites);

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

// ── What cannot be built standalone ─────────────────────────────────────
//
// `wx_app.rs`'s own local panels and `show_conversation_as_page` are not
// reachable from outside a running application: the first are built deep
// inside the window's own private setup function, alongside accounts, a
// database and a runtime this test has none of, and the second is a private
// function only that setup function calls. Splitting either apart so a test
// could construct them standalone was not part of this round's work.
//
// So these two checks do what `tests/wired.rs` already does for the things
// it cannot construct either: read the source and confirm the call is
// written where it should be. That is weaker than everything above, and says
// so. It proves the line exists, not that a control ever showed the colour.

fn wx_app_source() -> String {
    std::fs::read_to_string("src/presentation/wx_app.rs")
        .expect("src/presentation/wx_app.rs should be readable")
}

/// The body of one named function, cut from the rest of the file so a check
/// cannot pass by matching a similar line that belongs to something else.
fn function_body<'a>(source: &'a str, signature_start: &str) -> &'a str {
    let start = source
        .find(signature_start)
        .unwrap_or_else(|| panic!("{signature_start} was not found in wx_app.rs"));
    let after = &source[start..];
    // The next line that starts a top level item at column zero, which is
    // where this function ends and whatever follows it begins.
    let end = after[1..]
        .find("\nfn ")
        .map(|at| at + 1)
        .unwrap_or(after.len());
    &after[..end]
}

#[test]
fn test_wx_app_local_panels_are_written_to_paint_themselves() {
    let source = wx_app_source();

    for needle in [
        "theme::paint(&btn_panel, palette.second_surface());",
        "theme::paint(&mail_sidebar, palette.second_surface());",
        "theme::paint(&right_panel, palette.main_surface());",
        "theme::paint(&mail_content, palette.main_surface());",
        "theme::paint(&inner, palette.main_surface());",
    ] {
        assert!(
            source.contains(needle),
            "wx_app.rs no longer contains `{needle}`; this only checks the \
             call is written, not that any control shows it"
        );
    }
}

#[test]
fn test_the_conversation_as_headings_window_is_written_to_paint_itself() {
    let source = wx_app_source();
    let body = function_body(&source, "fn show_conversation_as_page(");

    for needle in [
        "theme::paint(&frame, palette.main_surface());",
        "theme::paint(&list, palette.main_surface());",
    ] {
        assert!(
            body.contains(needle),
            "show_conversation_as_page no longer contains `{needle}`; this \
             only checks the call is written, not that any control shows it"
        );
    }

    // The WebView is the named exception: it owns its colour through its
    // document's own HTML and CSS, independent of this setting, the same as
    // the mail preview and the compose body editor. Recorded here so a
    // change that starts painting it is a deliberate one, not an oversight
    // this check would otherwise wave through silently.
    assert!(
        !body.contains("theme::paint(&page,"),
        "the conversation page WebView is now painted; if that is deliberate, \
         update this test and the comment above it rather than deleting either"
    );
}

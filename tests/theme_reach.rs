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
use wixen_mail::application::calendar::{WhatIsBeingDone, WhatTheCalendarAllows, WhereAChangeGoes};
use wixen_mail::application::destinations::{Branch, Destination, Moving};
use wixen_mail::application::due::Snooze;
use wixen_mail::application::new_item::ItemKind;
use wixen_mail::application::spell_session;
use wixen_mail::application::words::{TextNode, words_in};
use wixen_mail::data::config::AppConfig;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::message_columns::{ColumnLayout, FolderKind};
use wixen_mail::presentation::reader_text::{ReaderAttachment, ReaderDocument};
use wixen_mail::presentation::theme::{self, Theme};
use wixen_mail::presentation::{
    wx_account_manager, wx_add_calendar, wx_app, wx_calendar, wx_calendar_module, wx_columns,
    wx_compose, wx_contacts_module, wx_destination, wx_first_run, wx_folder_choice, wx_item_form,
    wx_managers, wx_notes_module, wx_reader, wx_reminder_alert, wx_reminders_module, wx_settings,
    wx_tasks_module, wx_thread_view, wx_which_days,
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

/// The Settings dialog: the first of the standalone, `show_modal`-in-one-go
/// dialogs this file proves without a human closing a live modal. `config`'s
/// `theme` field, not a palette argument, because `build_settings_dialog`
/// deliberately reads its palette from the config it already holds rather
/// than a second, independent disk read that could disagree with it (see the
/// function's own doc comment).
fn check_settings(
    parent: &Frame,
    palette: theme::Palette,
    a11y: &Arc<Accessibility>,
    into: &mut Vec<SiteResult>,
) {
    let config = AppConfig {
        theme: "light".to_string(),
        ..AppConfig::default()
    };
    let widgets = wx_settings::build_settings_dialog(parent, &config, a11y);

    check(
        "settings dialog",
        &widgets.dialog,
        palette.main_surface(),
        into,
    );
    check(
        "settings notebook",
        &widgets.notebook,
        palette.main_surface(),
        into,
    );
    check(
        "settings general tab panel",
        &widgets.general_panel,
        palette.main_surface(),
        into,
    );
    check(
        "settings compose tab panel",
        &widgets.compose_panel,
        palette.main_surface(),
        into,
    );
    check(
        "settings reading tab panel",
        &widgets.reading_panel,
        palette.main_surface(),
        into,
    );
    check(
        "settings language tab panel",
        &widgets.permissions_panel,
        palette.main_surface(),
        into,
    );
    check(
        "settings calendar and pim tab panel",
        &widgets.pim_panel,
        palette.main_surface(),
        into,
    );
    check(
        "settings feedback tab panel",
        &widgets.feedback_panel,
        palette.main_surface(),
        into,
    );
    check(
        "settings advanced tab panel",
        &widgets.advanced_panel,
        palette.main_surface(),
        into,
    );
    check(
        "settings font size field",
        &widgets.font_size,
        palette.main_surface(),
        into,
    );
    check(
        "settings default reminder field",
        &widgets.default_reminder,
        palette.main_surface(),
        into,
    );
    check(
        "settings download folder field",
        &widgets.download_folder,
        palette.main_surface(),
        into,
    );
}

/// The Account Manager's own list window, and the Add/Edit Account dialog it
/// opens: two dialogs, proved the same way as Settings above, one nested
/// inside the other the way it really is opened in production (the edit
/// dialog's parent is the manager dialog, not the main frame).
fn check_account_manager(
    parent: &Frame,
    a11y: &Arc<Accessibility>,
    palette: theme::Palette,
    into: &mut Vec<SiteResult>,
) {
    let manager =
        wx_account_manager::build_account_manager_dialog(parent, &[], None, None, Some(palette));
    check(
        "account manager dialog",
        &manager.dialog,
        palette.main_surface(),
        into,
    );
    check(
        "account manager list",
        &manager.list,
        palette.main_surface(),
        into,
    );

    let edit =
        wx_account_manager::build_account_edit_dialog(&manager.dialog, None, a11y, Some(palette));
    check(
        "account edit dialog",
        &edit.dialog,
        palette.main_surface(),
        into,
    );
    for (name, field) in [
        ("account edit name field", &edit.name_f),
        ("account edit sender name field", &edit.sender_name_f),
        ("account edit email field", &edit.email_f),
        ("account edit imap server field", &edit.imap_f),
        ("account edit imap port field", &edit.imap_port_f),
        ("account edit pop server field", &edit.pop_f),
        ("account edit pop port field", &edit.pop_port_f),
        ("account edit smtp server field", &edit.smtp_f),
        ("account edit smtp port field", &edit.smtp_port_f),
        ("account edit username field", &edit.user_f),
        ("account edit password field", &edit.pass_f),
        ("account edit check interval field", &edit.interval_f),
    ] {
        check(name, field, palette.main_surface(), into);
    }
}

/// The calendar's New/Edit Event dialog: the item form dialog, opened
/// nested under a throwaway `Dialog` rather than under `parent` directly,
/// the same as production, since the Calendar window's own New and Edit
/// Event buttons open this dialog from inside its own agenda dialog rather
/// than from the main frame.
fn check_event_editor(
    parent: &Frame,
    palette: theme::Palette,
    a11y: &Arc<Accessibility>,
    into: &mut Vec<SiteResult>,
) {
    let scratch_parent = Dialog::builder(parent, "scratch parent for the event editor").build();
    let widgets = wx_item_form::build_item_form_dialog(
        &scratch_parent,
        ItemKind::Event,
        &[],
        &[],
        wx_item_form::Chrome {
            palette: Some(palette),
            a11y,
            asking: None,
        },
        wixen_mail::presentation::date_display::DateSettings::default(),
        None,
    )
    .expect("an Event has fields to ask for");
    check(
        "event editor dialog",
        &widgets.dialog,
        palette.main_surface(),
        into,
    );
    for (field, control) in &widgets.text_fields {
        check(field.label, control, palette.main_surface(), into);
    }
}

/// The "which days do you mean" question a repeating event's edit or delete
/// can open. No `TextCtrl`, `ListCtrl` or `TreeCtrl` anywhere in this dialog
/// (`StaticText`, `RadioButton` and `Button` only), so the dialog itself is
/// the only site to check.
fn check_which_days(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let allows = WhatTheCalendarAllows::just(WhereAChangeGoes::KeptHere);
    let (dialog, _buttons) = wx_which_days::build_which_days_dialog(
        parent,
        "Test event",
        "Weekly",
        WhatIsBeingDone::Changing,
        &allows,
        Some(palette),
    );
    check("which days dialog", &dialog, palette.main_surface(), into);
}

/// A single misspelling, built through the real `findings` function rather
/// than a `Finding` typed by hand: the words come from `words_in`, the same
/// segmenter the real spell check runs the message body through, so the
/// fixture is a shape this codebase's own code actually produces.
fn one_misspelling() -> spell_session::Finding {
    let words = words_in(&[TextNode {
        text: "the wrold turns".to_string(),
        block: 0,
    }]);
    let found = spell_session::findings(
        &words,
        |word| word == "wrold",
        |_| vec!["world".to_string()],
    );
    found
        .into_iter()
        .next()
        .expect("\"wrold\" is misspelled by construction")
}

/// The Check Spelling dialog Compose opens for one misspelled or repeated
/// word. Parented to a throwaway `Dialog` rather than to `parent` directly,
/// the same as production: it always opens from inside the Compose window,
/// never from the main frame.
fn check_check_spelling(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let scratch_parent = Dialog::builder(parent, "scratch parent for check spelling").build();
    let finding = one_misspelling();
    let widgets = wx_compose::build_check_spelling_dialog(&scratch_parent, &finding, Some(palette));
    check(
        "check spelling dialog",
        &widgets.dialog,
        palette.main_surface(),
        into,
    );
    check(
        "check spelling replacement field",
        &widgets.replacement,
        palette.main_surface(),
        into,
    );
    check(
        "check spelling suggestions list",
        &widgets.suggestions,
        palette.main_surface(),
        into,
    );
}

/// The Insert Table dialog Compose's formatting menu opens. Parented to a
/// throwaway `Dialog`, the same as production: it always opens from inside
/// the Compose window, never from the main frame. No `TextCtrl`, `ListCtrl`
/// or `TreeCtrl` anywhere in this dialog (a `SpinCtrl` for rows and columns,
/// a `CheckBox` for the header row), so the dialog itself is the only site.
fn check_insert_table(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let scratch_parent = Dialog::builder(parent, "scratch parent for insert table").build();
    let (dialog, _rows, _columns, _header) =
        wx_compose::build_insert_table_dialog(&scratch_parent, Some(palette));
    check("insert table dialog", &dialog, palette.main_surface(), into);
}

/// The Preview Before Send dialog Compose opens before a message goes out.
/// Parented to a throwaway `Dialog`, the same as production: it always opens
/// from inside the Compose window, never from the main frame. The body
/// preview is a `WebView`, excluded from painting for the same reason the
/// compose body editor and the conversation-as-headings page are (see the
/// `appears_live` checks at the bottom of this file), so the dialog itself
/// is the only site.
fn check_send_preview(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let scratch_parent = Dialog::builder(parent, "scratch parent for send preview").build();
    let data = wx_compose::ComposeData {
        to: "person@example.com".to_string(),
        cc: String::new(),
        bcc: String::new(),
        subject: "Test subject".to_string(),
        body: "<p>Test body</p>".to_string(),
        body_plain: "Test body".to_string(),
        html_mode: true,
        account_index: None,
        attachments: Vec::new(),
        answering: None,
    };
    let (dialog, _send_btn, _back_btn) =
        wx_compose::build_send_preview_dialog(&scratch_parent, &data, &[], Some(palette));
    check("send preview dialog", &dialog, palette.main_surface(), into);
}

/// The Calendar list window. An empty event list is enough: painting the
/// dialog and the list does not depend on what the list holds, and this
/// keeps the fixture from asserting anything about `CalendarEventItem`
/// beyond what this round actually changed.
fn check_calendar_list(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let widgets = wx_calendar::build_calendar_dialog(parent, &[], Some(palette));
    check(
        "calendar list dialog",
        &widgets.dialog,
        palette.main_surface(),
        into,
    );
    check("calendar list", &widgets.list, palette.main_surface(), into);
}

/// The Confirm Delete dialog the Calendar list window opens before deleting
/// one event. Parented to a throwaway `Dialog`, mirroring the way the real
/// caller nests it inside the Calendar list window rather than the main
/// frame. No `TextCtrl`, `ListCtrl` or `TreeCtrl` anywhere in this dialog, so
/// the dialog itself is the only site.
fn check_confirm_delete(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let scratch_parent = Dialog::builder(parent, "scratch parent for confirm delete").build();
    let (dialog, _yes_btn, _no_btn) =
        wx_calendar::build_confirm_delete_dialog(&scratch_parent, "Test event", Some(palette));
    check(
        "confirm delete dialog",
        &dialog,
        palette.main_surface(),
        into,
    );
}

/// The Search dialog. The scope `Choice` is left to Windows, matching every
/// other `Choice` this round paints around, so only the dialog and the
/// search field are checked here.
fn check_search(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    // Built with a folder open, so the "In" box is there with every one of its
    // answers on it and this checks the dialog somebody using mail really meets.
    let (dialog, q_field, _scope) = wx_app::build_search_dialog(
        parent,
        Some(palette),
        &wx_app::what_the_in_box_offers(Some(1)),
    );
    check("search dialog", &dialog, palette.main_surface(), into);
    check("search query field", &q_field, palette.main_surface(), into);
}

/// The Ask For A Name dialog, reached both from "New" on a PIM item and
/// directly from a rename. A note is included so the taller of the two
/// window sizes is the one this test builds, the same way
/// `document_that_exercises_every_optional_widget` builds a reader document
/// that exercises both of its optional tabs.
fn check_ask_for_a_name(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let (dialog, title_field) = wx_app::build_ask_for_a_name_dialog(
        parent,
        wx_app::Asking {
            window: "New Event",
            label: "Event &title:",
            note: Some("Kept in the calendar you last used."),
            filled_in: "",
            button: "C&reate",
        },
        Some(palette),
    );
    check(
        "ask for a name dialog",
        &dialog,
        palette.main_surface(),
        into,
    );
    check(
        "ask for a name title field",
        &title_field,
        palette.main_surface(),
        into,
    );
}

/// The Add Calendar dialog. The two radio buttons are left to Windows,
/// matching every other `RadioButton` this round paints around, so the
/// dialog and its four `TextCtrl` fields are what is checked here.
fn check_add_calendar(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let widgets = wx_add_calendar::build_add_calendar_dialog(parent, Some(palette));
    check(
        "add calendar dialog",
        &widgets.dialog,
        palette.main_surface(),
        into,
    );
    for (name, field) in [
        ("add calendar address field", &widgets.address),
        ("add calendar name field", &widgets.name),
        ("add calendar user name field", &widgets.user_name),
        ("add calendar password field", &widgets.password),
    ] {
        check(name, field, palette.main_surface(), into);
    }
}

/// The Columns dialog. The `CheckListBox` draws its own check marks rather
/// than going through a control the established pattern paints, so it is
/// left to Windows here, the same as every `Choice`, `ComboBox`,
/// `RadioButton` and `CheckBox` elsewhere in this round; only the dialog
/// itself is checked.
fn check_columns(
    parent: &Frame,
    a11y: &Arc<Accessibility>,
    palette: theme::Palette,
    into: &mut Vec<SiteResult>,
) {
    let layout = ColumnLayout::defaults_for(FolderKind::Inbox);
    let (dialog, _working) =
        wx_columns::build_column_dialog(parent, &layout, FolderKind::Inbox, a11y, Some(palette));
    check("columns dialog", &dialog, palette.main_surface(), into);
}

/// The "where does this go" dialog, opened to move or copy a message or a
/// PIM item. The tree is left to Windows here, the same as every `Choice`,
/// `ComboBox`, `RadioButton` and `CheckBox` elsewhere in this round; only the
/// dialog itself is checked.
fn check_destination(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let branches = [Branch {
        account_id: "acct-1".to_string(),
        account_name: "person@example.com".to_string(),
        places: vec![Destination {
            name: "Archive".to_string(),
            id: "archive".to_string(),
            account_id: "acct-1".to_string(),
            depth: 0,
        }],
    }];
    let (dialog, _tree) = wx_destination::build_destination_dialog(
        parent,
        Moving::Message,
        false,
        &branches,
        None,
        Some(palette),
    );
    check("destination dialog", &dialog, palette.main_surface(), into);
}

/// The first-run "what may Wixen Mail change" dialog. No `TextCtrl`,
/// `ListCtrl` or `TreeCtrl` anywhere in this dialog (`StaticText` and
/// `RadioButton` only), so the dialog itself is the only site.
fn check_first_run(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let (dialog, _buttons) = wx_first_run::build_first_run_dialog(parent, Some(palette));
    check("first run dialog", &dialog, palette.main_surface(), into);
}

/// The folder chooser opened from "Choose folders to keep up to date". The
/// `CheckListBox` draws its own check marks rather than going through a
/// control the established pattern paints, so it is left to Windows here,
/// the same as every `Choice`, `ComboBox`, `RadioButton` and `CheckBox`
/// elsewhere in this round; only the dialog itself is checked.
fn check_folder_choice(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let folders = [wx_folder_choice::FolderRow {
        path: "INBOX".to_string(),
        name: "Inbox".to_string(),
        syncing: true,
        subscribed: true,
        holds_all_mail: false,
        total: 10,
    }];
    let (dialog, _list) = wx_folder_choice::build_folder_choice_dialog(
        parent,
        "person@example.com",
        &folders,
        Some(palette),
    );
    check(
        "folder choice dialog",
        &dialog,
        palette.main_surface(),
        into,
    );
}

/// The item form dialog, asking for an Event: the one kind whose fields cover
/// every `Entry::Line` and `Entry::Paragraph` field this round paints (title,
/// location, description), alongside the ones it leaves to Windows (date,
/// time, choice, spin, category, tick).
fn check_item_form(
    parent: &Frame,
    palette: theme::Palette,
    a11y: &Arc<Accessibility>,
    into: &mut Vec<SiteResult>,
) {
    let widgets = wx_item_form::build_item_form_dialog(
        parent,
        ItemKind::Event,
        &[],
        &[],
        wx_item_form::Chrome {
            palette: Some(palette),
            a11y,
            asking: None,
        },
        wixen_mail::presentation::date_display::DateSettings::default(),
        None,
    )
    .expect("an Event has fields to ask for");
    check(
        "item form dialog",
        &widgets.dialog,
        palette.main_surface(),
        into,
    );
    for (field, control) in &widgets.text_fields {
        check(field.label, control, palette.main_surface(), into);
    }
}

/// The reminder alert window. No `TextCtrl`, `ListCtrl` or `TreeCtrl`
/// anywhere in this dialog (`StaticText`, `Choice` and buttons only), so the
/// dialog itself is the only site.
fn check_reminder_alert(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let (dialog, _snooze_choice) = wx_reminder_alert::build_reminder_alert_dialog(
        parent,
        "Test reminder due now",
        Snooze::ALL[0],
        Some(palette),
    );
    check(
        "reminder alert dialog",
        &dialog,
        palette.main_surface(),
        into,
    );
}

/// The Conversation tree dialog, opened before choosing to view a thread as
/// headings, one message, or plain text. The tree is left to Windows here,
/// the same as every `Choice`, `ComboBox`, `RadioButton` and `CheckBox`
/// elsewhere in this round; only the dialog itself is checked.
fn check_thread_view(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let nodes = [wx_thread_view::ThreadNode {
        message_id: 1,
        uid: 1,
        sender: "Ada Lovelace".to_string(),
        subject: "Quarterly report".to_string(),
        date: "2026-07-26".to_string(),
        read: true,
        depth: 0,
        parent: None,
    }];
    let (dialog, _chosen) =
        wx_thread_view::build_thread_dialog(parent, "Quarterly report", &nodes, Some(palette))
            .expect("a tree root should build from one node");
    check("thread view dialog", &dialog, palette.main_surface(), into);
}

/// The About dialog. No `TextCtrl`, `ListCtrl` or `TreeCtrl` anywhere in
/// this dialog (four `StaticText` and a button), so the dialog itself is the
/// only site.
fn check_about(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let dialog = wx_app::build_about_dialog(parent, Some(palette));
    check("about dialog", &dialog, palette.main_surface(), into);
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
        // The bar in this fixture is a real warning, which is what makes it
        // exercise the warning styling this test is about.
        looks_unsafe: true,
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

/// Compose's own window: the single most-used window in the application.
/// Parented directly to `parent`, the same as production: Compose opens
/// from the main frame, not from inside another dialog, unlike the three
/// dialogs above that open from inside Compose itself. The account choice
/// is left to Windows, matching every other `Choice` this round paints
/// around, and the message body is a `WebView` excluded from painting for
/// the same reason `check_send_preview` above excludes one: it owns its
/// colour through its own document's HTML and CSS.
fn check_compose(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let widgets = wx_compose::build_compose_dialog(
        parent,
        "Compose New Message",
        &["person@example.com".to_string()],
        0,
        Some(palette),
    );
    check(
        "compose dialog",
        &widgets.dialog,
        palette.main_surface(),
        into,
    );
    for (name, field) in [
        ("compose to field", &widgets.to_field),
        ("compose cc field", &widgets.cc_field),
        ("compose bcc field", &widgets.bcc_field),
        ("compose subject field", &widgets.subject_field),
    ] {
        check(name, field, palette.main_surface(), into);
    }
    check(
        "compose attachment list",
        &widgets.attachment_list,
        palette.main_surface(),
        into,
    );
}

/// The shell shared by the Filter, Tag and Signature managers' own list
/// windows: one dialog and one list stand in for all three, since each of
/// them only adds its own columns and its own Add/Edit/Delete loop on top of
/// what `make_shell` builds. "Filter Manager" is the title and size the real
/// filter manager passes; its own columns are inserted here too, the same
/// way a real caller inserts them right after this call returns, so this
/// proves the colour survives `InsertColumn` rather than only the moment
/// before it.
fn check_managers_shell(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let (dialog, _sizer, list, _status) =
        wx_managers::make_shell(parent, "Filter Manager", "Filters", 650, 450, Some(palette));
    list.insert_column(0, "Name", ListColumnFormat::Left, 130);
    list.insert_column(1, "Condition", ListColumnFormat::Left, 220);
    list.insert_column(2, "Action", ListColumnFormat::Left, 150);
    list.insert_column(3, "Status", ListColumnFormat::Centre, 70);
    check(
        "managers shell dialog",
        &dialog,
        palette.main_surface(),
        into,
    );
    check("managers shell list", &list, palette.main_surface(), into);
}

/// The Contact Manager's own list window, built directly rather than through
/// `make_shell`. An empty contact list is enough: painting the dialog, the
/// search field and the list does not depend on what the list holds. Takes
/// `a11y` because Delete's own click is wired up inside this same builder
/// now, and needs somewhere real to send its announcement.
fn check_contact_manager(
    parent: &Frame,
    a11y: &Arc<Accessibility>,
    palette: theme::Palette,
    into: &mut Vec<SiteResult>,
) {
    let handles = wx_managers::build_contact_manager_dialog(parent, &[], a11y, Some(palette));
    check(
        "contact manager dialog",
        &handles.dialog,
        palette.main_surface(),
        into,
    );
    check(
        "contact manager search field",
        &handles.search,
        palette.main_surface(),
        into,
    );
    check(
        "contact manager list",
        &handles.list,
        palette.main_surface(),
        into,
    );
}

/// The "please wait" window shown while a calendar server is asked what it
/// has. No `TextCtrl`, `ListCtrl` or `TreeCtrl` anywhere in this dialog
/// (`StaticText` and a `Button` only), so the dialog itself is the only site.
fn check_wait_for_an_answer(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let (dialog, _stop_button) = wx_managers::build_wait_for_an_answer_dialog(
        parent,
        "Adding a calendar",
        "Looking for calendars...",
        "Stop looking",
        Some(palette),
    );
    check(
        "wait for an answer dialog",
        &dialog,
        palette.main_surface(),
        into,
    );
}

/// The "pick one" window used to choose a calendar to add and to reopen a
/// saved draft. The list offers one item per choice, the same as a `Choice`
/// or a radio group elsewhere in this round, so it is left to Windows here
/// too; only the dialog itself is checked.
fn check_choose_from_list(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let items = vec!["Calendar A".to_string(), "Calendar B".to_string()];
    let (dialog, _list) = wx_managers::build_choose_from_list_dialog(
        parent,
        "Choose a calendar",
        "2 calendars found:",
        "&Add",
        &items,
        Some(palette),
    );
    check(
        "choose from list dialog",
        &dialog,
        palette.main_surface(),
        into,
    );
}

/// The Add/Edit Contact dialog `show_contact_edit` opens, both directly from
/// "New Contact" and from inside the Contact Manager's own list window.
/// `None` prefill is enough: painting the dialog, its notebook, its four
/// tab panels and every field does not depend on what is already filled in.
/// The favourite `CheckBox` is left to Windows, the same as every checkbox
/// elsewhere in this round, so it is not checked here.
fn check_contact_edit(
    parent: &Frame,
    palette: theme::Palette,
    a11y: &Arc<Accessibility>,
    into: &mut Vec<SiteResult>,
) {
    let handles = wx_managers::build_contact_edit_dialog(parent, None, Some(palette), a11y);
    check(
        "contact edit dialog",
        &handles.dialog,
        palette.main_surface(),
        into,
    );
    check(
        "contact edit notebook",
        &handles.notebook,
        palette.main_surface(),
        into,
    );
    for (name, panel) in [
        ("contact edit basic info panel", &handles.basic_panel),
        ("contact edit email and phone panel", &handles.contact_panel),
        ("contact edit addresses panel", &handles.addr_panel),
        ("contact edit notes and custom panel", &handles.notes_panel),
    ] {
        check(name, panel, palette.main_surface(), into);
    }
    for (name, field) in [
        ("contact edit name field", &handles.name_f),
        ("contact edit given name field", &handles.given_f),
        ("contact edit family name field", &handles.family_f),
        ("contact edit nickname field", &handles.nick_f),
        ("contact edit company field", &handles.company_f),
        ("contact edit department field", &handles.dept_f),
        ("contact edit job title field", &handles.title_f),
        ("contact edit birthday field", &handles.bday_f),
        ("contact edit website field", &handles.web_f),
        ("contact edit relationship field", &handles.rel_f),
        ("contact edit avatar url field", &handles.avatar_f),
        ("contact edit notes field", &handles.notes_f),
    ] {
        check(name, field, palette.main_surface(), into);
    }
    for (name, list) in [
        ("contact edit email list", &handles.email_list),
        ("contact edit phone list", &handles.phone_list),
        ("contact edit address list", &handles.addr_list),
        ("contact edit custom field list", &handles.custom_list),
    ] {
        check(name, list, palette.main_surface(), into);
    }
}

/// The Contact editor's own Add Email Address sub-dialog. Parented to a
/// throwaway `Dialog`, the same as production: it always opens from inside
/// the Contact editor, never from the main frame. The type `Choice` is left
/// to Windows, matching every other `Choice` this round paints around, so
/// only the dialog and the address field are checked here.
fn check_email_sub_dialog(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let scratch_parent = Dialog::builder(parent, "scratch parent for add email").build();
    let (dialog, _type_choice, addr_f) =
        wx_managers::build_email_sub_dialog(&scratch_parent, Some(palette));
    check("add email dialog", &dialog, palette.main_surface(), into);
    check(
        "add email address field",
        &addr_f,
        palette.main_surface(),
        into,
    );
}

/// The Contact editor's own Add Phone Number sub-dialog. Parented to a
/// throwaway `Dialog`, the same as production. The type `Choice` is left to
/// Windows, matching every other `Choice` this round paints around, so only
/// the dialog and the number field are checked here.
fn check_phone_sub_dialog(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let scratch_parent = Dialog::builder(parent, "scratch parent for add phone").build();
    let (dialog, _type_choice, num_f) =
        wx_managers::build_phone_sub_dialog(&scratch_parent, Some(palette));
    check("add phone dialog", &dialog, palette.main_surface(), into);
    check(
        "add phone number field",
        &num_f,
        palette.main_surface(),
        into,
    );
}

/// The Contact editor's own Add Address sub-dialog. Parented to a throwaway
/// `Dialog`, the same as production. Both `Choice` controls (country and
/// type) are left to Windows, matching every other `Choice` this round
/// paints around, so the dialog and its four `TextCtrl` fields are what is
/// checked here.
fn check_address_sub_dialog(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let scratch_parent = Dialog::builder(parent, "scratch parent for add address").build();
    let widgets = wx_managers::build_address_sub_dialog(&scratch_parent, Some(palette));
    check(
        "add address dialog",
        &widgets.dialog,
        palette.main_surface(),
        into,
    );
    for (name, field) in [
        ("add address street field", &widgets.street_f),
        ("add address city field", &widgets.city_f),
        ("add address region field", &widgets.region_f),
        ("add address postal code field", &widgets.code_f),
    ] {
        check(name, field, palette.main_surface(), into);
    }
}

/// The Contact editor's own Add Custom Field sub-dialog. Parented to a
/// throwaway `Dialog`, the same as production.
fn check_custom_field_sub_dialog(
    parent: &Frame,
    palette: theme::Palette,
    into: &mut Vec<SiteResult>,
) {
    let scratch_parent = Dialog::builder(parent, "scratch parent for add custom field").build();
    let (dialog, label_f, value_f) =
        wx_managers::build_custom_field_sub_dialog(&scratch_parent, Some(palette));
    check(
        "add custom field dialog",
        &dialog,
        palette.main_surface(),
        into,
    );
    check(
        "add custom field label field",
        &label_f,
        palette.main_surface(),
        into,
    );
    check(
        "add custom field value field",
        &value_f,
        palette.main_surface(),
        into,
    );
}

/// The Add/Edit Filter Rule dialog the Filter Manager's own list window
/// opens. Parented to a throwaway `Dialog`, the same as production: it
/// always opens from inside the Filter Manager, never from the main frame.
/// The three `Choice` controls and the two `CheckBox` controls are left to
/// Windows, matching every other `Choice` and `CheckBox` this round paints
/// around, so only the dialog and its three `TextCtrl` fields are checked
/// here.
fn check_filter_edit(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let scratch_parent = Dialog::builder(parent, "scratch parent for filter edit").build();
    let widgets = wx_managers::build_filter_edit_dialog(&scratch_parent, None, Some(palette));
    check(
        "filter edit dialog",
        &widgets.dialog,
        palette.main_surface(),
        into,
    );
    for (name, field) in [
        ("filter edit name field", &widgets.name_f),
        ("filter edit pattern field", &widgets.pattern_f),
        ("filter edit action value field", &widgets.action_value_f),
    ] {
        check(name, field, palette.main_surface(), into);
    }
}

/// The Add/Edit Tag dialog the Tag Manager's own list window opens.
/// Parented to a throwaway `Dialog`, the same as production. The colour
/// `Choice` is left to Windows, matching every other `Choice` this round
/// paints around, so only the dialog and the name field are checked here.
fn check_tag_edit(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let scratch_parent = Dialog::builder(parent, "scratch parent for tag edit").build();
    let (dialog, name_f, _color_choice) =
        wx_managers::build_tag_edit_dialog(&scratch_parent, None, Some(palette));
    check("tag edit dialog", &dialog, palette.main_surface(), into);
    check("tag edit name field", &name_f, palette.main_surface(), into);
}

/// The Add/Edit Signature dialog the Signature Manager's own list window
/// opens. Parented to a throwaway `Dialog`, the same as production. The
/// default-signature `CheckBox` is left to Windows, the same as every
/// checkbox elsewhere in this round, so the dialog and its three `TextCtrl`
/// fields are what is checked here.
fn check_sig_edit(parent: &Frame, palette: theme::Palette, into: &mut Vec<SiteResult>) {
    let scratch_parent = Dialog::builder(parent, "scratch parent for signature edit").build();
    let widgets = wx_managers::build_sig_edit_dialog(&scratch_parent, None, Some(palette));
    check(
        "signature edit dialog",
        &widgets.dialog,
        palette.main_surface(),
        into,
    );
    for (name, field) in [
        ("signature edit name field", &widgets.name_f),
        ("signature edit signature field", &widgets.content_f),
    ] {
        check(name, field, palette.main_surface(), into);
    }
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
            check_settings(&frame, palette, &a11y, &mut sites);
            check_account_manager(&frame, &a11y, palette, &mut sites);
            check_event_editor(&frame, palette, &a11y, &mut sites);
            check_which_days(&frame, palette, &mut sites);
            check_check_spelling(&frame, palette, &mut sites);
            check_insert_table(&frame, palette, &mut sites);
            check_send_preview(&frame, palette, &mut sites);
            check_calendar_list(&frame, palette, &mut sites);
            check_confirm_delete(&frame, palette, &mut sites);
            check_search(&frame, palette, &mut sites);
            check_ask_for_a_name(&frame, palette, &mut sites);
            check_add_calendar(&frame, palette, &mut sites);
            check_columns(&frame, &a11y, palette, &mut sites);
            check_destination(&frame, palette, &mut sites);
            check_first_run(&frame, palette, &mut sites);
            check_folder_choice(&frame, palette, &mut sites);
            check_item_form(&frame, palette, &a11y, &mut sites);
            check_reminder_alert(&frame, palette, &mut sites);
            check_thread_view(&frame, palette, &mut sites);
            check_about(&frame, palette, &mut sites);
            check_compose(&frame, palette, &mut sites);
            check_managers_shell(&frame, palette, &mut sites);
            check_contact_manager(&frame, &a11y, palette, &mut sites);
            check_wait_for_an_answer(&frame, palette, &mut sites);
            check_choose_from_list(&frame, palette, &mut sites);
            check_contact_edit(&frame, palette, &a11y, &mut sites);
            check_email_sub_dialog(&frame, palette, &mut sites);
            check_phone_sub_dialog(&frame, palette, &mut sites);
            check_address_sub_dialog(&frame, palette, &mut sites);
            check_custom_field_sub_dialog(&frame, palette, &mut sites);
            check_filter_edit(&frame, palette, &mut sites);
            check_tag_edit(&frame, palette, &mut sites);
            check_sig_edit(&frame, palette, &mut sites);

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

/// Whether `needle` sits on a line of `haystack` that a `//` comment has not
/// swallowed.
///
/// `str::contains` cannot tell a live call from a commented-out one, because
/// a line commented out with `// theme::paint(...)` still holds the call's
/// exact text as a literal substring. Proven by hand before this existed:
/// commenting out a real call left `haystack.contains(needle)` reporting
/// true, so the two checks below traded that method for this one.
fn appears_live(haystack: &str, needle: &str) -> bool {
    haystack.lines().any(|line| {
        line.find(needle)
            .is_some_and(|at| !line[..at].contains("//"))
    })
}

#[cfg(test)]
mod appears_live_tests {
    use super::appears_live;

    #[test]
    fn test_a_call_on_its_own_line_is_live() {
        assert!(appears_live(
            "theme::paint(&x, y);\n",
            "theme::paint(&x, y);"
        ));
    }

    #[test]
    fn test_a_commented_out_call_is_not_live() {
        assert!(!appears_live(
            "// theme::paint(&x, y);\n",
            "theme::paint(&x, y);"
        ));
    }

    #[test]
    fn test_a_call_after_other_code_on_the_same_line_is_still_live() {
        assert!(appears_live(
            "if let Some(y) = z { theme::paint(&x, y); }\n",
            "theme::paint(&x, y);"
        ));
    }

    #[test]
    fn test_a_call_present_only_inside_a_doc_comment_is_not_live() {
        assert!(!appears_live(
            "/// See also `theme::paint(&x, y);` for the pattern this follows.\n",
            "theme::paint(&x, y);"
        ));
    }
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
            appears_live(&source, needle),
            "wx_app.rs no longer has a live `{needle}`; this only checks the \
             call is written and not commented out, not that any control \
             shows it"
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
            appears_live(body, needle),
            "show_conversation_as_page no longer has a live `{needle}`; this \
             only checks the call is written and not commented out, not \
             that any control shows it"
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

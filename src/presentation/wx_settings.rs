//! Settings / Preferences dialog
//!
//! A tabbed dialog accessible from Tools > Settings that exposes the most
//! commonly used email client configuration options.  Settings are read from
//! and persisted through `AppConfig` / `ConfigManager`.

use crate::application::autosave::AutosaveInterval;
use crate::application::folder_settings::{self, UnreadOnAParent};
use crate::application::reading_habits::{CopyLines, MarkRead, WorkingDay};
use crate::application::reading_style::Style as ReadingStyle;
use crate::application::receipts::Policy;
use crate::common::paths::AppPaths;
use crate::data::config::AppConfig;
use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::feedback::{Channel, FeedbackSettings};
use crate::presentation::accessibility::names::{
    name_from_label, set_accessible_name, set_accessible_name_and_description,
};
use crate::presentation::accessibility::sound_scheme::SoundScheme;
use crate::presentation::accessibility::sound_scheme_import;
use crate::presentation::theme;
use crate::service::spellcheck::available_languages;
use std::sync::Arc;
use wxdragon::prelude::*;

// ── Result type ──────────────────────────────────────────────────────────────

/// The outcome of the settings dialog.
pub enum SettingsResult {
    /// User pressed OK, carrying the possibly modified configuration.
    ///
    /// Boxed because the configuration dwarfs the other variant, and this
    /// grew past the point where every Cancelled paid for it.
    Updated(Box<AppConfig>),
    /// User cancelled: no changes.
    Cancelled,
}

// ── Widget references ────────────────────────────────────────────────────────

/// Holds references to all mutable settings widgets so we can read them back
/// when the user presses OK.
///
/// Public, and every field with it, so a test can build the dialog without
/// showing it and read back what a live control was really painted: the same
/// shape `CalendarPanelHandles` and `ReaderTabHandles` already use for the
/// eight surfaces round 25 could reach directly.
pub struct SettingsWidgets {
    /// The dialog itself, and the notebook that tabs it, so both can be
    /// painted and both can be read back by a test.
    pub dialog: Dialog,
    pub notebook: Notebook,
    // Which panel backs which tab, so a test can check each one was painted
    // without walking the notebook's own children to find it.
    pub general_panel: Panel,
    pub compose_panel: Panel,
    pub reading_panel: Panel,
    pub permissions_panel: Panel,
    pub pim_panel: Panel,
    pub feedback_panel: Panel,
    pub advanced_panel: Panel,
    // General
    theme: Choice,
    pub font_size: TextCtrl,
    pub font_family: Choice,
    // Compose
    preview_before_send: CheckBox,
    keep_sent_mail_on_this_computer: CheckBox,
    draft_autosave: SpinCtrl,
    add_signature_automatically: CheckBox,
    // Reading
    sort_order: Choice,
    read_receipts: Choice,
    read_messages_as: Choice,
    date_style: Choice,
    date_order: Choice,
    date_wording: Choice,
    clock_hours: Choice,
    mark_read_after: Choice,
    sort_then: Choice,
    copy_lines: Choice,
    start_in_all_inboxes: CheckBox,
    unread_on_a_parent: Choice,
    empty_reaches_subfolders: CheckBox,
    mark_read_reaches_subfolders: CheckBox,
    hold_back_remote_pictures: CheckBox,
    smooth_scrolling: CheckBox,
    keep_selected_message_in_view: CheckBox,
    keep_running_in_the_tray: CheckBox,
    check_default_programs_at_startup: CheckBox,
    // Language
    language: Choice,
    check_spelling_before_send: CheckBox,
    allow_mail: CheckBox,
    allow_pim: CheckBox,
    send_contact_changes_everywhere: CheckBox,
    check_spelling_as_you_type: CheckBox,
    // Calendar & PIM
    pub default_reminder: TextCtrl,
    day_starts: Choice,
    day_ends: Choice,
    // Advanced
    log_level: Choice,
    pub download_folder: TextCtrl,
    look_at_message_contents: CheckBox,
    check_links_with_google: CheckBox,
    // Feedback channels: each box carries the channel it switches, so a tick
    // cannot be read back against a different one.
    feedback: Vec<(Channel, CheckBox)>,
    // Which sound plays. Read back by re-running the same discovery that
    // populated it and indexing by selection, the same shape `sel` already
    // gives every other Choice, rather than carrying a second, parallel
    // list of ids that could drift out of position with the widget's own.
    sound_scheme: Choice,
}

/// Helper: unwrap get_selection() returning 0 if None.
fn sel(choice: &Choice) -> u32 {
    choice.get_selection().unwrap_or(0)
}

// ── Section helper ───────────────────────────────────────────────────────────

/// Create a labelled section sizer using StaticBoxSizerBuilder::new_with_label.
fn section(parent: &Panel, label: &str) -> StaticBoxSizer {
    StaticBoxSizerBuilder::new_with_label(Orientation::Vertical, parent, label).build()
}

// ── Public entry point ───────────────────────────────────────────────────────

/// Show the Settings dialog and return the (possibly updated) configuration.
pub fn show_settings_dialog(
    parent: &Frame,
    config: &AppConfig,
    a11y: &Arc<Accessibility>,
) -> SettingsResult {
    let widgets = build_settings_dialog(parent, config, a11y);
    if widgets.dialog.show_modal() != ID_OK {
        return SettingsResult::Cancelled;
    }
    // An answer this cannot keep is said rather than written over in
    // silence. A working day that runs past midnight is refused, which is a
    // real limit, and the screen used to put nine to five back with nothing
    // said at all: somebody choosing a night shift set it, heard nothing, and
    // found the built-in day again the next time they looked.
    let starts = sel(&widgets.day_starts) as u8;
    let ends = sel(&widgets.day_ends) as u8;
    if WorkingDay::could_not_be_used(starts, ends) {
        let _ = a11y.announce(
            "A working day that runs past midnight cannot be kept, so the \
             working day is unchanged. Everything else you changed is saved.",
            crate::presentation::accessibility::announcements::Priority::High,
        );
    }
    SettingsResult::Updated(Box::new(read_settings(&widgets, config)))
}

/// Build the Settings dialog without showing it.
///
/// Everything `show_settings_dialog` used to do up to its own
/// `.show_modal()` call, split out so a test can build the real dialog, read
/// back the real colour a real control is holding, and never call
/// `.show_modal()` at all. Round 25 used this shape for the eight surfaces it
/// could reach directly; this is the same shape reaching a standalone dialog
/// for the first time.
///
/// Takes `&AppConfig` rather than a separate palette argument: Settings is
/// the one dialog in this application that already holds, in `config`, the
/// exact settings it is about to display and let somebody change, so it asks
/// [`theme::current`] with that value directly rather than through
/// [`theme::current_from_stored_config`], which would mean a second,
/// independent disk read that could in principle disagree with the config
/// already in hand, including the very Theme dropdown this dialog is about
/// to build.
pub fn build_settings_dialog(
    parent: &Frame,
    config: &AppConfig,
    a11y: &Arc<Accessibility>,
) -> SettingsWidgets {
    let dlg = Dialog::builder(parent, "Settings")
        .with_size(560, 520)
        .build();

    let root_sizer = BoxSizer::builder(Orientation::Vertical).build();

    // Notebook (tabbed pane)
    let notebook = Notebook::builder(&dlg).build();

    // ── Tab 1: General
    let general_panel = Panel::builder(&notebook).build();
    let GeneralTabControls {
        theme,
        font_family,
        font_size,
        language,
        check_before_send: check_spelling_before_send,
        check_as_you_type: check_spelling_as_you_type,
        smooth_scrolling,
        keep_selected_message_in_view,
        keep_running_in_the_tray,
        choose_default_programs,
        check_default_programs_at_startup,
    } = build_general_tab(&general_panel, config);
    notebook.add_page(&general_panel, "General", true, None);

    // ── Tab 2: Compose
    let compose_panel = Panel::builder(&notebook).build();
    let (
        preview_before_send,
        keep_sent_mail_on_this_computer,
        draft_autosave,
        add_signature_automatically,
    ) = build_compose_tab(&compose_panel, config);
    notebook.add_page(&compose_panel, "Compose", false, None);

    // ── Tab 3: Reading
    let reading_panel = Panel::builder(&notebook).build();
    let ReadingTabControls {
        sort_order,
        read_receipts,
        read_messages_as,
        date_style,
        date_order,
        date_wording,
        clock_hours,
        mark_read_after,
        sort_then,
        copy_lines,
        start_in_all_inboxes,
        unread_on_a_parent,
        empty_reaches_subfolders,
        mark_read_reaches_subfolders,
        hold_back_remote_pictures,
    } = build_reading_tab(&reading_panel, config);
    notebook.add_page(&reading_panel, "Reading", false, None);

    // ── Tab 4: what this application may change
    let permissions_panel = Panel::builder(&notebook).build();
    let (allow_mail, allow_pim, send_contact_changes_everywhere) =
        build_permissions_tab(&permissions_panel, config);
    notebook.add_page(&permissions_panel, "Permissions", false, None);

    // ── Tab 5: Calendar & PIM
    let pim_panel = Panel::builder(&notebook).build();
    let (default_reminder, day_starts, day_ends) = build_calendar_pim_tab(&pim_panel, config);
    notebook.add_page(&pim_panel, "Calendar && PIM", false, None);

    // ── Tab 6: Feedback
    let feedback_panel = Panel::builder(&notebook).build();
    let (feedback, sound_scheme) = build_feedback_tab(&feedback_panel, config, a11y);
    notebook.add_page(&feedback_panel, "Feedback", false, None);

    // ── Tab 7: Advanced
    let advanced_panel = Panel::builder(&notebook).build();
    let (log_level, download_folder, look_at_message_contents, check_links_with_google) =
        build_advanced_tab(&advanced_panel, config);
    notebook.add_page(&advanced_panel, "Advanced", false, None);

    root_sizer.add(&notebook, 1, SizerFlag::Expand | SizerFlag::All, 8);

    // ── OK / Cancel buttons
    let btn_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    btn_sizer.add_spacer(0);
    let ok_btn = Button::builder(&dlg)
        .with_label("OK")
        .with_id(ID_OK)
        .build();
    let cancel_btn = Button::builder(&dlg)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    btn_sizer.add(&ok_btn, 0, SizerFlag::All, 4);
    btn_sizer.add(&cancel_btn, 0, SizerFlag::All, 4);
    root_sizer.add_sizer(&btn_sizer, 0, SizerFlag::AlignRight | SizerFlag::All, 8);

    dlg.set_sizer(root_sizer, true);

    // Windows is where this is chosen, and this is the only thing an
    // application is allowed to do about it: take somebody there. The failure
    // is reported rather than swallowed, because a button that opens nothing
    // and says nothing is indistinguishable from a button that does not work.
    choose_default_programs.on_click({
        let d = dlg;
        move |_ev| {
            if let Err(why) = crate::service::default_apps::open_windows_default_apps_page() {
                tracing::warn!("The Windows default programs page would not open: {why}");
                // A message box rather than an announcement: a screen reader
                // reads one out on its own, and this dialog carries no
                // accessibility handle to announce through.
                let said = MessageDialog::builder(
                    &d,
                    "The Windows settings page for default programs would not open. \
                     It is under Settings, Apps, Default apps.",
                    "Could not open Windows settings",
                )
                .with_style(MessageDialogStyle::OK | MessageDialogStyle::IconInformation)
                .build();
                said.show_modal();
            }
        }
    });

    ok_btn.on_click({
        let d = dlg;
        move |_ev| {
            d.end_modal(ID_OK);
        }
    });
    cancel_btn.on_click({
        let d = dlg;
        move |_ev| {
            d.end_modal(ID_CANCEL);
        }
    });

    // Painted last, after every tab has built and populated its own
    // controls, and computed from `config` rather than a fresh disk read
    // (see this function's own doc comment for why). `None` means high
    // contrast is on, or the system is set up in a way this application
    // should not paint over, so nothing is set here and Windows decides.
    if let Some(palette) = theme::current(&config.theme) {
        theme::paint(&dlg, palette.main_surface());
        theme::paint(&notebook, palette.main_surface());
        for panel in [
            &general_panel,
            &compose_panel,
            &reading_panel,
            &permissions_panel,
            &pim_panel,
            &feedback_panel,
            &advanced_panel,
        ] {
            theme::paint(panel, palette.main_surface());
        }
        for field in [&font_size, &default_reminder, &download_folder] {
            theme::paint(field, palette.main_surface());
        }
    }

    SettingsWidgets {
        dialog: dlg,
        notebook,
        general_panel,
        compose_panel,
        reading_panel,
        permissions_panel,
        pim_panel,
        feedback_panel,
        advanced_panel,
        theme,
        font_size,
        font_family,
        preview_before_send,
        keep_sent_mail_on_this_computer,
        draft_autosave,
        add_signature_automatically,
        sort_order,
        read_receipts,
        read_messages_as,
        date_style,
        date_order,
        date_wording,
        clock_hours,
        mark_read_after,
        sort_then,
        copy_lines,
        start_in_all_inboxes,
        unread_on_a_parent,
        empty_reaches_subfolders,
        mark_read_reaches_subfolders,
        hold_back_remote_pictures,
        smooth_scrolling,
        keep_selected_message_in_view,
        keep_running_in_the_tray,
        check_default_programs_at_startup,
        language,
        check_spelling_before_send,
        check_spelling_as_you_type,
        allow_mail,
        allow_pim,
        send_contact_changes_everywhere,
        default_reminder,
        day_starts,
        day_ends,
        log_level,
        download_folder,
        look_at_message_contents,
        check_links_with_google,
        feedback,
        sound_scheme,
    }
}

// ── Tab builders ─────────────────────────────────────────────────────────────

/// General settings: theme and font size.
/// How views move: whether they slide, and whether they follow the cursor.
///
/// Smooth scrolling says outright that Windows can overrule it, because it
/// can. [`crate::application::scrolling`] holds that rule and the reason: an
/// animation setting somebody has already made once should not have to be made
/// again in every program, and the way they would find out this one ignored
/// them is by being made unwell.
fn add_scrolling(panel: &Panel, config: &AppConfig, sizer: &BoxSizer) -> (CheckBox, CheckBox) {
    use crate::application::scrolling::{
        SystemMotion, system_motion, what_the_machine_has_overruled,
    };

    let scroll_sec = section(panel, "Scrolling");

    let smooth = CheckBox::builder(panel)
        .with_label("&Slide when a view scrolls, rather than jumping")
        .build();
    set_accessible_name_and_description(
        &smooth,
        "Slide when a view scrolls, rather than jumping",
        "Windows overrules this when it is set to reduce animation",
    );
    smooth.set_value(config.smooth_scrolling);
    scroll_sec.add(&smooth, 0, SizerFlag::All, 4);

    // Said only on a machine it is true of, rather than as a permanent caveat
    // that the people it applies to cannot pick out from the people it does
    // not. A ticked box doing nothing otherwise reads as a broken program.
    if system_motion() == SystemMotion::Reduced {
        let said = what_the_machine_has_overruled(true, SystemMotion::Reduced);
        let note = StaticText::builder(panel).with_label(&said).build();
        set_accessible_name(&note, &said);
        scroll_sec.add(&note, 0, SizerFlag::Expand | SizerFlag::All, 4);
    }

    let keep_in_view = CheckBox::builder(panel)
        .with_label("&Keep the chosen message in view when the list reloads")
        .build();
    set_accessible_name_and_description(
        &keep_in_view,
        "Keep the chosen message in view when the list reloads",
        "Turning this off leaves the view where it is when a sync finishes. \
         Your place in the list is not lost either way",
    );
    keep_in_view.set_value(config.keep_selected_message_in_view);
    scroll_sec.add(&keep_in_view, 0, SizerFlag::All, 4);

    sizer.add_sizer(&scroll_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);
    (smooth, keep_in_view)
}

/// Which program Windows opens email, calendar files and contact cards with.
///
/// Read rather than set. Since Windows 8 an application cannot make itself the
/// default, so there is no button here that does it: there is a list saying
/// what is set now, and a button that takes somebody to the Windows screen
/// where they choose. A button promising more would be one that cannot keep
/// its promise, which is guardrail 3.
///
/// Six rows, including the three Windows keeps no default for. Leaving those
/// out would leave somebody looking for tasks in this list wondering whether
/// they had missed it.
fn add_default_programs(panel: &Panel, config: &AppConfig, sizer: &BoxSizer) -> (Button, CheckBox) {
    use crate::presentation::default_app_words::{button_label, row, why_windows_asks};
    use crate::service::default_apps::{DefaultKind, is_default};

    let programs_sec = section(panel, "Default programs");

    let why = StaticText::builder(panel)
        .with_label(why_windows_asks())
        .build();
    set_accessible_name(&why, why_windows_asks());
    programs_sec.add(&why, 0, SizerFlag::Expand | SizerFlag::All, 4);

    // Each row is one whole sentence rather than a name and a state in two
    // columns, because a screen reader reads a row and the halves have to
    // arrive together to mean anything.
    for kind in DefaultKind::ALL {
        let said = row(kind, &is_default(kind));
        let line = StaticText::builder(panel).with_label(&said).build();
        set_accessible_name(&line, &said);
        programs_sec.add(&line, 0, SizerFlag::Left | SizerFlag::All, 2);
    }

    let choose = Button::builder(panel).with_label(button_label()).build();
    set_accessible_name_and_description(
        &choose,
        "Choose default programs in Windows",
        "Opens the Windows settings screen at Wixen Mail, where you choose \
         which file types and links it opens. Wixen Mail cannot change this \
         itself",
    );
    programs_sec.add(&choose, 0, SizerFlag::Left | SizerFlag::All, 4);

    let check_at_startup = CheckBox::builder(panel)
        .with_label("Check this &every time Wixen Mail starts")
        .build();
    set_accessible_name_and_description(
        &check_at_startup,
        "Check this every time Wixen Mail starts",
        "Says so at startup when another program holds one of these. Off by \
         default, because somebody who chose another program on purpose does \
         not need telling about it every time",
    );
    check_at_startup.set_value(config.check_default_programs_at_startup);
    programs_sec.add(&check_at_startup, 0, SizerFlag::Left | SizerFlag::All, 4);

    sizer.add_sizer(&programs_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);
    (choose, check_at_startup)
}

/// What closing the window does.
///
/// Off by default, and deliberately so. Closing a window and having the program
/// carry on is not what closing a window means to most people, and it is worse
/// than most for somebody who cannot see the screen: the window goes, the
/// reading stops, and nothing says the program is still there. So it is asked
/// for rather than assumed, it is announced the first time it happens, and Quit
/// always really quits.
fn add_closing(panel: &Panel, config: &AppConfig, sizer: &BoxSizer) -> CheckBox {
    let close_sec = section(panel, "Closing the window");

    let to_tray = CheckBox::builder(panel)
        .with_label("Keep Wixen Mail running in the &notification area")
        .build();
    set_accessible_name_and_description(
        &to_tray,
        "Keep Wixen Mail running in the notification area",
        "Closing the window hides it instead of ending the program. Quit still \
         ends it. The notification area holds a menu with New Message, Check \
         Mail and All Inboxes on it",
    );
    to_tray.set_value(config.keep_running_in_the_tray);
    close_sec.add(&to_tray, 0, SizerFlag::All, 4);

    sizer.add_sizer(&close_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);
    to_tray
}

/// What the General tab hands back.
///
/// A struct rather than a tuple because it carries seven controls now, and a
/// seven-place tuple is a list nobody can read at the call site: swapping two
/// of the check boxes would compile and save each other's setting.
struct GeneralTabControls {
    theme: Choice,
    font_family: Choice,
    font_size: TextCtrl,
    language: Choice,
    check_before_send: CheckBox,
    check_as_you_type: CheckBox,
    smooth_scrolling: CheckBox,
    keep_selected_message_in_view: CheckBox,
    keep_running_in_the_tray: CheckBox,
    choose_default_programs: Button,
    check_default_programs_at_startup: CheckBox,
}

/// Which language spelling is checked in, and how the checking behaves.
///
/// Under General now rather than on a tab of its own. The default comes from
/// the machine's own locale, which `data::config` reads when it writes a fresh
/// settings file, so the first thing somebody sees is the language they
/// already work in.
fn add_language_and_spelling(
    panel: &Panel,
    config: &AppConfig,
    sizer: &BoxSizer,
) -> (Choice, CheckBox, CheckBox) {
    // -- Language
    //
    // "Interface language" was the wrong label. Nothing here is translated, and
    // this setting has only ever decided which dictionary the spell checker
    // uses. The list was wrong too: it offered the same six languages whatever
    // the machine had, so picking one it could not check set a value that
    // changed nothing, and the only way to find that out was to write in that
    // language and have every word of it called a mistake.
    let lang_sec = section(panel, "Language and spelling");

    let lang_row = BoxSizer::builder(Orientation::Horizontal).build();
    let lang_label = StaticText::builder(panel)
        .with_label("Check spelling in:")
        .build();

    let languages = available_languages();
    let lang_names: Vec<String> = languages
        .iter()
        .map(|language| {
            if language.available {
                language.name.clone()
            } else {
                // Shown and marked rather than hidden. A language with no
                // dictionary is still worth offering, because installing one
                // is something somebody can go and do.
                format!("{} (no dictionary installed)", language.name)
            }
        })
        .collect();
    let lang_idx = languages
        .iter()
        .position(|language| language.tag == config.language)
        .unwrap_or(0) as u32;
    let lang_choice = Choice::builder(panel)
        .with_choices(lang_names)
        .with_selection(Some(lang_idx))
        .build();
    set_accessible_name_and_description(
        &lang_choice,
        "Check spelling in",
        "Starts as the language this computer is set to",
    );
    lang_row.add(
        &lang_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    lang_row.add(&lang_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    lang_sec.add_sizer(&lang_row, 0, SizerFlag::Expand, 0);

    let check_before_send = CheckBox::builder(panel)
        .with_label("&Check spelling before sending a message")
        .build();
    set_accessible_name(
        &check_before_send,
        "Check spelling before sending a message",
    );
    check_before_send.set_value(config.check_spelling_before_send);
    lang_sec.add(&check_before_send, 0, SizerFlag::All, 4);

    let check_as_you_type = CheckBox::builder(panel)
        .with_label("&Mark misspelled words as I write")
        .build();
    set_accessible_name(&check_as_you_type, "Mark misspelled words as I write");
    check_as_you_type.set_value(config.check_spelling_as_you_type);
    lang_sec.add(&check_as_you_type, 0, SizerFlag::All, 4);

    // What the marking is, said plainly, because it is not this application
    // doing the announcing and somebody comparing it with another program
    // should know why it sounds like their browser.
    let marking_note = StaticText::builder(panel)
        .with_label(
            "Marked words are announced by your screen reader as you move over them. \
             There is also a sound at the end of a word that is wrong, which is off \
             until earcons are switched on under Feedback.",
        )
        .build();
    lang_sec.add(&marking_note, 0, SizerFlag::All, 4);

    let speller = crate::service::spellcheck::for_language(&config.language);
    let checker_note = StaticText::builder(panel)
        .with_label(&format!(
            "Spelling is checked by {}.",
            speller.source().describe()
        ))
        .build();
    lang_sec.add(&checker_note, 0, SizerFlag::All, 4);

    sizer.add_sizer(&lang_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);
    (lang_choice, check_before_send, check_as_you_type)
}

fn build_general_tab(panel: &Panel, config: &AppConfig) -> GeneralTabControls {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Appearance
    let app_sec = section(panel, "Appearance");

    let theme_row = BoxSizer::builder(Orientation::Horizontal).build();
    let theme_label = StaticText::builder(panel).with_label("Theme:").build();
    let theme_choices: Vec<String> = ["Default", "Light", "Dark", "High Contrast"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let theme_idx: u32 = match config.theme.as_str() {
        "light" => 1,
        "dark" => 2,
        "high_contrast" => 3,
        _ => 0,
    };
    let theme_choice = Choice::builder(panel)
        .with_choices(theme_choices)
        .with_selection(Some(theme_idx))
        .build();
    set_accessible_name(&theme_choice, "Theme");
    theme_row.add(
        &theme_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    theme_row.add(&theme_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    app_sec.add_sizer(&theme_row, 0, SizerFlag::Expand, 0);

    // One sentence, said once, and it comes from the code it describes rather
    // than being retyped here where it can drift away from what the theme
    // actually does. A setting that changes less than it looks like it should
    // is a setting somebody reads as broken.
    let theme_note = StaticText::builder(panel)
        .with_label(crate::presentation::theme::REACH)
        .build();
    set_accessible_name(&theme_note, crate::presentation::theme::REACH);
    app_sec.add(&theme_note, 0, SizerFlag::Expand | SizerFlag::All, 4);

    // Only the fonts this computer has. A list written here would be the same
    // mistake the language list once made: Windows draws something else for a
    // typeface it does not have and says nothing, so choosing one that is not
    // installed would look like it worked and change nothing on the screen.
    let installed = crate::service::fonts::installed_families().unwrap_or_default();
    let type_row = BoxSizer::builder(Orientation::Horizontal).build();
    let type_label = StaticText::builder(panel).with_label("Font:").build();
    let type_choice = Choice::builder(panel)
        .with_choices(crate::application::font_choice::what_the_list_offers(
            &installed,
        ))
        .with_selection(Some(crate::application::font_choice::which_row_is_chosen(
            &config.font_family,
            &installed,
        ) as u32))
        .build();
    set_accessible_name_and_description(
        &type_choice,
        "Font",
        "The typeface your messages, contacts, calendar, tasks, notes and \
         reminders are listed in. Only fonts installed on this computer are \
         offered",
    );
    type_row.add(
        &type_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    type_row.add(&type_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    app_sec.add_sizer(&type_row, 0, SizerFlag::Expand, 0);

    // Said only when it is true of this machine. A font uninstalled after it
    // was chosen, or a settings file carried from another computer, otherwise
    // shows as a program that looks wrong for no stated reason.
    let wrong = crate::application::font_choice::what_is_wrong_with_the_choice(
        &config.font_family,
        &installed,
    );
    if !wrong.is_empty() {
        let note = StaticText::builder(panel).with_label(&wrong).build();
        set_accessible_name(&note, &wrong);
        app_sec.add(&note, 0, SizerFlag::Expand | SizerFlag::All, 4);
    }

    let font_row = BoxSizer::builder(Orientation::Horizontal).build();
    let font_label = StaticText::builder(panel).with_label("Font size:").build();
    let font_field = TextCtrl::builder(panel).build();
    set_accessible_name(&font_field, "Font size");
    font_field.set_value(&config.font_size.to_string());
    font_row.add(
        &font_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    font_row.add(&font_field, 0, SizerFlag::All, 4);
    app_sec.add_sizer(&font_row, 0, SizerFlag::Expand, 0);

    sizer.add_sizer(&app_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // New-mail notifications and checking for updates were both offered here
    // and read by nothing: there is no notification path and no update check
    // in this program. A control that takes an answer and ignores it is worse
    // than no control, so they are gone rather than sitting switched off.

    let (language, check_before_send, check_as_you_type) =
        add_language_and_spelling(panel, config, &sizer);
    let (smooth_scrolling, keep_selected_message_in_view) = add_scrolling(panel, config, &sizer);
    let keep_running_in_the_tray = add_closing(panel, config, &sizer);
    let (choose_default_programs, check_default_programs_at_startup) =
        add_default_programs(panel, config, &sizer);

    panel.set_sizer(sizer, true);
    GeneralTabControls {
        theme: theme_choice,
        font_family: type_choice,
        font_size: font_field,
        language,
        check_before_send,
        check_as_you_type,
        smooth_scrolling,
        keep_selected_message_in_view,
        keep_running_in_the_tray,
        choose_default_programs,
        check_default_programs_at_startup,
    }
}

/// Compose settings: preview before sending, what Sent keeps, drafts, signature.
fn build_compose_tab(
    panel: &Panel,
    config: &AppConfig,
) -> (CheckBox, CheckBox, SpinCtrl, CheckBox) {
    use crate::application::sent_copy::{KEEP_A_COPY_CONSEQUENCE, KEEP_A_COPY_LABEL};

    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Sending
    let send_sec = section(panel, "Sending");
    let preview_cb = CheckBox::builder(panel)
        .with_label("Show &preview before sending")
        .build();
    set_accessible_name(&preview_cb, "Show preview before sending");
    preview_cb.set_value(config.preview_before_send);
    send_sec.add(&preview_cb, 0, SizerFlag::All, 4);

    // The description carries the consequence: with this on, Sent lists every
    // message twice once the server's own copy comes down. That is what the
    // setting does rather than a fault, and it is the part nobody can see
    // coming from the label alone.
    let keep_a_copy_cb = CheckBox::builder(panel)
        .with_label(KEEP_A_COPY_LABEL)
        .build();
    set_accessible_name_and_description(
        &keep_a_copy_cb,
        &name_from_label(KEEP_A_COPY_LABEL),
        KEEP_A_COPY_CONSEQUENCE,
    );
    keep_a_copy_cb.set_value(config.keep_sent_mail_on_this_computer);
    send_sec.add(&keep_a_copy_cb, 0, SizerFlag::All, 4);

    // A choice of "HTML" or "Plain Text" used to sit here, fixed on HTML and
    // read back by nothing. There is no such setting: the composer is one
    // editor, every message goes out with both a plain part and a formatted
    // one, and nothing anywhere asks which was wanted. Choosing the format a
    // message is written in is a feature, not a switch, so the control is gone
    // rather than left saying a decision was taken.

    sizer.add_sizer(&send_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Drafts
    let draft_sec = section(panel, "Drafts");
    // A spin box rather than the checkbox that used to be here. That claimed
    // "every 60 seconds", was ticked, was never read back, and nothing saved
    // anything. Minutes are the right grain: the difference between ninety
    // seconds and two minutes is not a decision anybody can make usefully, and
    // a spin box steps with the arrow keys rather than needing a number typed.
    let autosave_row = BoxSizer::builder(Orientation::Horizontal).build();
    let autosave_label = StaticText::builder(panel)
        .with_label("Save &drafts automatically every (minutes, 0 for never):")
        .build();
    let autosave_spin = SpinCtrl::builder(panel)
        .with_range(0, AutosaveInterval::MAX_MINUTES as i32)
        .build();
    set_accessible_name(
        &autosave_spin,
        "Save drafts automatically every, minutes, 0 for never",
    );
    autosave_spin
        .set_value(AutosaveInterval::from_setting(config.draft_autosave_minutes).minutes() as i32);
    autosave_row.add(
        &autosave_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    autosave_row.add(&autosave_spin, 0, SizerFlag::All, 4);
    draft_sec.add_sizer(&autosave_row, 0, SizerFlag::Expand, 0);
    sizer.add_sizer(&draft_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Signatures
    //
    // The label says every message rather than a new one, because that is what
    // the composer does: a reply, a forward and a message written from a
    // contact all open with it too. It said "on new messages" while being
    // hard-set to yes and read back by nothing, so it was narrower than the
    // truth and made no difference either way.
    let sig_sec = section(panel, "Signatures");
    let sig_cb = CheckBox::builder(panel).with_label(SIGNATURE_LABEL).build();
    set_accessible_name_and_description(
        &sig_cb,
        &name_from_label(SIGNATURE_LABEL),
        SIGNATURE_WHEN_THIS_IS_OFF,
    );
    sig_cb.set_value(config.add_signature_automatically);
    sig_sec.add(&sig_cb, 0, SizerFlag::All, 4);

    // The same sentence on screen. A description reaches a screen reader that
    // reads through Microsoft Active Accessibility and nothing else, so what
    // the unticked state means would otherwise be there for one reader and
    // nobody else.
    let sig_note = StaticText::builder(panel)
        .with_label(SIGNATURE_WHEN_THIS_IS_OFF)
        .build();
    set_accessible_name(&sig_note, SIGNATURE_WHEN_THIS_IS_OFF);
    sig_sec.add(&sig_note, 0, SizerFlag::Expand | SizerFlag::All, 4);

    sizer.add_sizer(&sig_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    (preview_cb, keep_a_copy_cb, autosave_spin, sig_cb)
}

/// One sentence, said once, in the label and in the accessible name.
const SIGNATURE_LABEL: &str = "Start every message with my &signature";

/// What the unticked state means, which a checkbox alone cannot say.
const SIGNATURE_WHEN_THIS_IS_OFF: &str = "Off: a message starts empty. Your signature stays on the account and can \
     still be added by hand.";

/// The controls `build_reading_tab` lays out, one field per choice, named
/// for what it actually controls rather than by position.
struct ReadingTabControls {
    empty_reaches_subfolders: CheckBox,
    mark_read_reaches_subfolders: CheckBox,
    sort_order: Choice,
    start_in_all_inboxes: CheckBox,
    unread_on_a_parent: Choice,
    hold_back_remote_pictures: CheckBox,
    read_receipts: Choice,
    read_messages_as: Choice,
    date_style: Choice,
    date_order: Choice,
    date_wording: Choice,
    clock_hours: Choice,
    mark_read_after: Choice,
    sort_then: Choice,
    copy_lines: Choice,
}

/// Reading settings: how the list is sorted, how a message opens, dates.
fn build_reading_tab(panel: &Panel, config: &AppConfig) -> ReadingTabControls {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Message List
    let list_sec = section(panel, "Message List");

    let sort_row = BoxSizer::builder(Orientation::Horizontal).build();
    let sort_label = StaticText::builder(panel)
        .with_label("Default sort order:")
        .build();
    let sort_choices: Vec<String> = [
        "Date (Newest First)",
        "Date (Oldest First)",
        "Sender (A-Z)",
        "Sender (Z-A)",
        "Subject (A-Z)",
        "Subject (Z-A)",
        "Unread First",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let sort_idx: u32 = match config.default_sort_order.as_str() {
        "date_oldest" => 1,
        "sender_az" => 2,
        "sender_za" => 3,
        "subject_az" => 4,
        "subject_za" => 5,
        "unread_first" => 6,
        _ => 0,
    };
    let sort_choice = Choice::builder(panel)
        .with_choices(sort_choices)
        .with_selection(Some(sort_idx))
        .build();
    set_accessible_name(&sort_choice, "Default sort order");
    sort_row.add(
        &sort_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    sort_row.add(&sort_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    list_sec.add_sizer(&sort_row, 0, SizerFlag::Expand, 0);

    // A checkbox reading "Enable threaded view by default" used to sit here.
    // Threaded view is not implemented: its View menu item is built disabled
    // for that reason and says so, and this box was unticked, saved by nothing
    // and read by nothing. A default for something that cannot be switched on
    // is a setting for a feature that is not there.

    // This one does something, which is the difference. The folder tree opens
    // with no row chosen, so mail is listed only once somebody arrows onto a
    // folder; ticking this lands them in the combined inbox instead.
    let start_in_all_inboxes = CheckBox::builder(panel)
        .with_label("Start in All &Inboxes")
        .build();
    start_in_all_inboxes.set_value(config.start_in_all_inboxes);
    set_accessible_name_and_description(
        &start_in_all_inboxes,
        "Start in All Inboxes",
        "Open showing every account's inbox in one list, rather than with no folder chosen",
    );
    list_sec.add(
        &start_in_all_inboxes,
        0,
        SizerFlag::Left | SizerFlag::All,
        4,
    );
    sizer.add_sizer(&list_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Folders and message lists, D-42: one group for the settings about how
    // the folder tree and the message list behave, rather than five of them
    // scattered down this page. The group's name comes from one constant, so
    // the first sentence anywhere that sends somebody here reads the name they
    // will hear rather than a second copy of it.
    let folders_sec = section(panel, folder_settings::SETTINGS_SECTION);

    // A choice rather than a check box, because neither option is the absence
    // of the other: both say something, and a box labelled for one of them
    // would have to word the other as "not that".
    let unread_on_a_parent = labelled_choice(
        panel,
        &folders_sec,
        "&Unread on a folder or account that holds others:",
        "Unread on a folder or account that holds others",
        &UnreadOnAParent::ALL.map(|option| option.words()),
        UnreadOnAParent::ALL
            .iter()
            .position(|option| *option == UnreadOnAParent::from_stored(&config.unread_on_a_parent))
            .unwrap_or(0) as u32,
    );

    // D-34 and D-35, and they are two boxes rather than one on purpose: one of
    // them destroys mail and the other loses somebody their place, and neither
    // has an undo. A single box covering both would make anybody who wanted
    // the careful reach on one of them take it on the other too.
    //
    // Both carry their label through `with_label` and their name through
    // `set_accessible_name_and_description`. A box labelled only through
    // `set_accessible_name` has a name under NVDA and none under Narrator,
    // which `tests/checkbox_labels.rs` is the guard for.
    //
    // The descriptions say what the answer costs rather than repeating the
    // label: the name already says what the box does, and what a screen reader
    // user needs next is which way is the wider one.
    let empty_reaches_subfolders = CheckBox::builder(panel)
        .with_label("&Empty Folder also empties the folders inside it")
        .build();
    empty_reaches_subfolders.set_value(config.empty_reaches_subfolders);
    set_accessible_name_and_description(
        &empty_reaches_subfolders,
        "Empty Folder also empties the folders inside it",
        "On, so emptying a folder empties everything filed under it too. The confirmation always \
         says how many folders and how many messages, and whether they move or go for good",
    );
    folders_sec.add(
        &empty_reaches_subfolders,
        0,
        SizerFlag::Left | SizerFlag::All,
        4,
    );

    let mark_read_reaches_subfolders = CheckBox::builder(panel)
        .with_label("Mark Folder &Read also marks the folders inside it")
        .build();
    mark_read_reaches_subfolders.set_value(config.mark_read_reaches_subfolders);
    set_accessible_name_and_description(
        &mark_read_reaches_subfolders,
        "Mark Folder Read also marks the folders inside it",
        "On, so marking a folder read marks everything filed under it too. There is no undo, so \
         turn this off to keep your place in folders you have not finished",
    );
    folders_sec.add(
        &mark_read_reaches_subfolders,
        0,
        SizerFlag::Left | SizerFlag::All,
        4,
    );

    sizer.add_sizer(&folders_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Reading Behaviour
    let read_sec = section(panel, "Reading Behaviour");

    // Built from the list rather than from a second copy of the words, and
    // read back below. It was neither before: four fixed choices, a fixed
    // selection, and nothing that saved it, so the answer was always
    // "immediately" whatever it said here.
    let markread_choice = labelled_choice(
        panel,
        &read_sec,
        "&Mark as read after:",
        "Mark as read after",
        &MarkRead::ALL
            .iter()
            .map(|c| c.label())
            .collect::<Vec<_>>()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        crate::application::reading_habits::offered_index(&config.mark_read_after) as u32,
    );

    // How a message opens. First in this section, because it is the biggest
    // difference between two ways of reading the same mail.
    let style_row = BoxSizer::builder(Orientation::Horizontal).build();
    let style_label = StaticText::builder(panel)
        .with_label("Open messages:")
        .build();
    let style_choice = Choice::builder(panel)
        .with_choices(
            ReadingStyle::ALL
                .iter()
                .map(|style| style.spoken().to_string())
                .collect(),
        )
        .with_selection(Some(0))
        .build();
    set_accessible_name_and_description(
        &style_choice,
        "Open messages",
        "Formatted keeps the sender's headings, links and tables. Plain text \
         gives you a caret to move through the message with, and flattens them.",
    );
    let chosen_style = ReadingStyle::from_stored(&config.read_messages_as);
    style_choice.set_selection(
        ReadingStyle::ALL
            .iter()
            .position(|style| *style == chosen_style)
            .unwrap_or(0) as u32,
    );
    style_row.add(
        &style_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    style_row.add(&style_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    read_sec.add_sizer(&style_row, 0, SizerFlag::Expand, 0);

    // Read receipts. On the Reading tab because it is a thing that happens
    // when you open a message, which is where somebody would look for it.
    let receipt_row = BoxSizer::builder(Orientation::Horizontal).build();
    let receipt_label = StaticText::builder(panel)
        .with_label("Tell senders when you read their mail:")
        .build();
    let receipt_choice = Choice::builder(panel)
        .with_choices(
            Policy::ALL
                .iter()
                .map(|policy| policy.spoken().to_string())
                .collect(),
        )
        .with_selection(Some(0))
        .build();
    // The whole sentence is the accessible name, because each choice says what
    // it costs and a name of "Read receipts" would hide that from the person
    // most likely to care about being tracked.
    set_accessible_name_and_description(
        &receipt_choice,
        "Tell senders when you read their mail",
        "A read receipt tells the sender your address is live and roughly when \
         you were at your desk. Nothing is sent unless you choose it here.",
    );
    let chosen = Policy::from_stored(&config.read_receipts);
    receipt_choice.set_selection(
        Policy::ALL
            .iter()
            .position(|policy| *policy == chosen)
            .unwrap_or(0) as u32,
    );
    receipt_row.add(
        &receipt_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    receipt_row.add(&receipt_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    read_sec.add_sizer(&receipt_row, 0, SizerFlag::Expand, 0);

    // There was a checkbox here once that saved nothing and was read by
    // nothing, and then a sentence admitting the pictures were fetched and
    // there was no switch. This is the switch, and it does what it says.
    let hold_back_remote_pictures = CheckBox::builder(panel)
        .with_label("Do not &fetch pictures a message only points at")
        .build();
    hold_back_remote_pictures.set_value(config.hold_back_remote_pictures);
    set_accessible_name_and_description(
        &hold_back_remote_pictures,
        "Do not fetch pictures a message only points at",
        "On by default. Fetching one tells the sender you opened the message. \
         Pictures the message carries are always shown; they are already here \
         and showing them tells nobody anything",
    );
    read_sec.add(&hold_back_remote_pictures, 0, SizerFlag::All, 4);

    let images_note = StaticText::builder(panel)
        .with_label(REMOTE_IMAGES_ARE_FETCHED)
        .build();
    set_accessible_name(&images_note, REMOTE_IMAGES_ARE_FETCHED);
    read_sec.add(&images_note, 0, SizerFlag::Expand | SizerFlag::All, 4);

    sizer.add_sizer(&read_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Dates and times
    //
    // Read in every module, and until now settable in none of them: the values
    // were in the settings file with nothing that could change them. What a
    // date sounds like is not a detail to somebody who hears every one of them.
    let date_sec = section(panel, "Dates and Times");

    // Two of the settings below say "Follow this computer", which makes the
    // whole section look as though it follows the computer's language too. It
    // does not, and somebody hearing an English month in the middle of an
    // otherwise French date deserves to be told that rather than left thinking
    // their screen reader is at fault.
    let date_language_note = StaticText::builder(panel)
        .with_label(crate::presentation::date_display::ENGLISH_ONLY)
        .build();
    set_accessible_name(
        &date_language_note,
        crate::presentation::date_display::ENGLISH_ONLY,
    );
    date_sec.add(
        &date_language_note,
        0,
        SizerFlag::Expand | SizerFlag::All,
        4,
    );

    let date_style = labelled_choice(
        panel,
        &date_sec,
        "How much of a date to say:",
        "How much of a date to say",
        &["Relative within the last week", "Always the full date"],
        match config.date_style.as_str() {
            "absolute" => 1,
            _ => 0,
        },
    );
    let date_order = labelled_choice(
        panel,
        &date_sec,
        "Day and month order:",
        "Day and month order",
        &[
            "Follow this computer",
            "Month first, July 26",
            "Day first, 26 July",
        ],
        match config.date_order.as_str() {
            "month_first" => 1,
            "day_first" => 2,
            _ => 0,
        },
    );
    let date_wording = labelled_choice(
        panel,
        &date_sec,
        "Write the month as:",
        "Write the month as",
        &["A word, July 26, 2026", "A number, 07/26/2026"],
        match config.date_wording.as_str() {
            "numeric" => 1,
            _ => 0,
        },
    );
    let sort_then = labelled_choice(
        panel,
        &date_sec,
        "Then &by:",
        "Then by",
        &SECOND_LEVEL_LABELS,
        second_level_index(&config.message_columns),
    );
    let copy_lines = labelled_choice(
        panel,
        &date_sec,
        "Cc and Bcc &lines:",
        "Cc and Bcc lines",
        &["Always in the compose window", "Only when they are in use"],
        match CopyLines::from_setting(&config.copy_lines) {
            CopyLines::Shown => 0,
            CopyLines::Hidden => 1,
        },
    );
    let clock_hours = labelled_choice(
        panel,
        &date_sec,
        "Clock:",
        "Clock",
        &[
            "Follow this computer",
            "Twelve hour, 2:30 PM",
            "Twenty-four hour, 14:30",
        ],
        match config.clock_hours.as_str() {
            "12" => 1,
            "24" => 2,
            _ => 0,
        },
    );

    sizer.add_sizer(&date_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    ReadingTabControls {
        sort_order: sort_choice,
        read_receipts: receipt_choice,
        read_messages_as: style_choice,
        date_style,
        date_order,
        date_wording,
        clock_hours,
        mark_read_after: markread_choice,
        sort_then,
        copy_lines,
        start_in_all_inboxes,
        unread_on_a_parent,
        empty_reaches_subfolders,
        mark_read_reaches_subfolders,
        hold_back_remote_pictures,
    }
}

/// What happens to a picture a message points at, said rather than switched.
///
/// Narrower than the checkbox that used to stand here, and narrower than the
/// first version of this sentence, which said a picture was fetched whenever a
/// message was shown. The reading window is a text control and fetches
/// nothing. The two surfaces that show a message body in a browser are the
/// preview pane and the conversation window, and the address the sender wrote
/// is still in the message on both of them.
///
/// Read rather than measured: the code leaves the address in the message and
/// hands it to a browser, and nothing here refuses the request. No network
/// trace has been taken.
const REMOTE_IMAGES_ARE_FETCHED: &str = "A picture a message points at rather than carries is left in the message. \
     The preview pane and the conversation window show a message in a browser, \
     which fetches the picture and so tells the sender the message was opened. \
     The reading window shows text and fetches nothing. There is no setting \
     for this yet.";

/// What the second level of the sort can be, in the order it is offered.
///
/// A short list rather than every column. The question people actually ask of a
/// mailbox is "among the ones from today, which have I not read", and offering
/// fifteen columns here would bury it.
const SECOND_LEVEL_LABELS: [&str; 4] = [
    "Nothing else",
    "Unread first",
    "Sender, A to Z",
    "Subject, A to Z",
];

/// The second level a stored layout holds, as a position in that list.
fn second_level_index(stored: &str) -> u32 {
    use crate::presentation::message_columns::{ColumnLayout, FolderKind, MessageColumn};
    let layout = ColumnLayout::from_stored(stored, FolderKind::Inbox);
    match layout.sort.then.map(|then| then.column) {
        Some(MessageColumn::Unread) => 1,
        Some(MessageColumn::Correspondent) => 2,
        Some(MessageColumn::Subject) => 3,
        _ => 0,
    }
}

/// The second level chosen, put back into the stored layout.
///
/// The layout is where the sort lives, so this reads the one that is stored,
/// changes the one part this control decides, and writes it out again. Writing
/// a fresh layout instead would throw away the columns somebody arranged, which
/// for anybody navigating a list by ear is real work.
fn with_second_level(stored: &str, chosen: u32) -> String {
    use crate::presentation::message_columns::{
        By, ColumnLayout, FolderKind, MessageColumn, SortDirection,
    };
    let mut layout = ColumnLayout::from_stored(stored, FolderKind::Inbox);
    layout.sort.then = match chosen {
        1 => Some(By {
            column: MessageColumn::Unread,
            direction: SortDirection::Ascending,
        }),
        2 => Some(By {
            column: MessageColumn::Correspondent,
            direction: SortDirection::Ascending,
        }),
        3 => Some(By {
            column: MessageColumn::Subject,
            direction: SortDirection::Ascending,
        }),
        _ => None,
    };
    layout.to_stored()
}

/// A choice with a label beside it, added to a section.
///
/// Four of these in a row is four copies of the same nine lines, and the copy
/// that gets the accessible name wrong is the one nobody notices until somebody
/// meets a control that announces nothing.
fn labelled_choice(
    panel: &Panel,
    section: &StaticBoxSizer,
    label: &str,
    spoken: &str,
    choices: &[&str],
    selected: u32,
) -> Choice {
    let row = BoxSizer::builder(Orientation::Horizontal).build();
    let text = StaticText::builder(panel).with_label(label).build();
    let choice = Choice::builder(panel)
        .with_choices(choices.iter().map(|c| c.to_string()).collect())
        .with_selection(Some(selected))
        .build();
    set_accessible_name(&choice, spoken);
    row.add(&text, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    row.add(&choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    section.add_sizer(&row, 0, SizerFlag::Expand, 0);
    choice
}

/// Language & Spelling: which language to check, and whether to check on send.
/// One sentence, said once, in the label and in the accessible name. Those two
/// were hand-written copies elsewhere in this file and had already drifted.
const CONTACT_CHANGES_WHEN_THIS_IS_OFF: &str =
    "Off: a change goes only to the address book the contact came from.";

/// What Wixen Mail may change at a server, and how a contact edit travels.
///
/// This was the Language and Spelling tab, which held neither of those things
/// on its own: the language picker and the two spelling boxes are under General
/// now, and what was left is the two permissions and the contacts rule. Named
/// for what it holds, because a tab named for something it does not contain is
/// a tab nobody looks in for what it does.
fn build_permissions_tab(panel: &Panel, config: &AppConfig) -> (CheckBox, CheckBox, CheckBox) {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Spell Check
    //
    // Two checkboxes used to sit here, "enable spell checking in compose
    // editor" and "show suggestions as you type". Both were ticked, neither
    // was ever read back, and nothing checked anything, so they were three
    // claims in a row. What replaces them says what this machine actually has,
    // which is worth more than a switch for something that does not happen.
    // ── What may be changed at a server ──────────────────────────────────
    //
    // Two checkboxes rather than one, because the two cost different amounts
    // to get wrong: a sent message cannot be recalled and a deleted one may
    // have been the only copy, while a task in the wrong place can be moved
    // back. Both say they are experimental, because they are: none of it has
    // run against a real account.
    // The heading comes from the same constant the sync sentences name, so a
    // person told to turn this on reads the words they were told. The label
    // and the sentence were typed separately and said different things.
    let allowed_sec = section(panel, crate::application::allowed::SETTINGS_SECTION);

    let allow_pim = CheckBox::builder(panel)
        .with_label("Allow Wixen Mail to change my &tasks, contacts and calendar")
        .build();
    set_accessible_name(
        &allow_pim,
        "Allow Wixen Mail to change my tasks, contacts and calendar",
    );
    allow_pim.set_value(config.allowed_changes.personal_information);
    allowed_sec.add(&allow_pim, 0, SizerFlag::All, 4);

    let allow_mail = CheckBox::builder(panel)
        .with_label("Allow Wixen Mail to &send and delete mail")
        .build();
    set_accessible_name(&allow_mail, "Allow Wixen Mail to send and delete mail");
    allow_mail.set_value(config.allowed_changes.mail);
    allowed_sec.add(&allow_mail, 0, SizerFlag::All, 4);

    // One sentence, said once. The label and the accessible name were two
    // hand-written copies that had already drifted apart.
    let allowed_note = StaticText::builder(panel)
        .with_label(crate::application::allowed::EXPERIMENTAL_WARNING)
        .build();
    set_accessible_name(
        &allowed_note,
        crate::application::allowed::EXPERIMENTAL_WARNING,
    );
    allowed_sec.add(&allowed_note, 0, SizerFlag::Expand | SizerFlag::All, 4);

    // This line was missing. The section was built, the two checkboxes and the
    // experimental warning were put into it, and the section itself was never
    // put into the panel's layout, so the one place that says none of this has
    // run against a real account had nowhere to appear.
    sizer.add_sizer(&allowed_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // ── Contacts ─────────────────────────────────────────────────────────
    //
    // Directly under the warning above, so somebody reading down the panel
    // meets the sentence saying none of this has run against a real account
    // before they meet this. The label says what happens rather than naming
    // the machinery, and the sentence under it says what turning it off does,
    // because a checkbox alone cannot say what its unticked state means.
    let contacts_sec = section(panel, "Contacts");

    let send_contact_changes_everywhere = CheckBox::builder(panel)
        .with_label(
            // Alt+V rather than Alt+S, which the mail setting above already
            // claims on this page.
            "Send a change to a contact to e&very address book that has that contact",
        )
        .build();
    set_accessible_name(
        &send_contact_changes_everywhere,
        "Send a change to a contact to every address book that has that contact",
    );
    send_contact_changes_everywhere.set_value(config.send_contact_changes_everywhere);
    contacts_sec.add(&send_contact_changes_everywhere, 0, SizerFlag::All, 4);

    let contacts_note = StaticText::builder(panel)
        .with_label(CONTACT_CHANGES_WHEN_THIS_IS_OFF)
        .build();
    set_accessible_name(&contacts_note, CONTACT_CHANGES_WHEN_THIS_IS_OFF);
    contacts_sec.add(&contacts_note, 0, SizerFlag::Expand | SizerFlag::All, 4);

    sizer.add_sizer(&contacts_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    (allow_mail, allow_pim, send_contact_changes_everywhere)
}

/// Calendar & PIM settings: default view, weekends, first day, reminder time.
fn build_calendar_pim_tab(panel: &Panel, config: &AppConfig) -> (TextCtrl, Choice, Choice) {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Calendar View
    let view_sec = section(panel, "Calendar");

    // A default view, showing weekends, and the first day of the week were
    // all offered here and read by nothing. The view picker was the worst of
    // them: it offered Day, Week and Month, three views this program cannot
    // draw at all. There is one agenda view and no week to start.

    sizer.add_sizer(&view_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Reminders
    let rem_sec = section(panel, "Reminders");

    let rem_row = BoxSizer::builder(Orientation::Horizontal).build();
    let rem_label = StaticText::builder(panel)
        .with_label("Default &reminder (minutes):")
        .build();
    let rem_field = TextCtrl::builder(panel).build();
    set_accessible_name(&rem_field, "Default reminder in minutes");
    rem_field.set_value(&config.default_reminder_minutes.to_string());
    rem_row.add(
        &rem_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    rem_row.add(&rem_field, 0, SizerFlag::All, 4);
    rem_sec.add_sizer(&rem_row, 0, SizerFlag::Expand, 0);

    sizer.add_sizer(&rem_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- The working day
    //
    // The calendar read every hour the same way, so nine in the morning and
    // three in the morning sounded alike and an event outside the working day
    // said nothing about itself. A meeting at seven in the evening is a fact
    // somebody wants to notice.
    let day_sec = section(panel, "Working Day");
    let day = WorkingDay::from_setting(config.working_day_starts, config.working_day_ends);
    let day_starts = labelled_choice(
        panel,
        &day_sec,
        "Starts &at:",
        "The working day starts at",
        &HOURS,
        day.starts as u32,
    );
    let day_ends = labelled_choice(
        panel,
        &day_sec,
        "&Ends at:",
        "The working day ends at",
        &HOURS,
        // The list runs from midnight, and the end is the first hour outside
        // the day, so five in the afternoon is the entry at seventeen.
        (day.ends as u32).min(HOURS.len() as u32 - 1),
    );
    sizer.add_sizer(&day_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    (rem_field, day_starts, day_ends)
}

/// Every hour of the day, named rather than numbered.
///
/// "Midnight" and "noon" rather than "12 AM" and "12 PM", which are the two
/// nobody agrees about, and a twenty-four hour reading beside each so the list
/// reads the same to somebody who keeps either clock.
const HOURS: [&str; 25] = [
    "Midnight, 00",
    "1 AM, 01",
    "2 AM, 02",
    "3 AM, 03",
    "4 AM, 04",
    "5 AM, 05",
    "6 AM, 06",
    "7 AM, 07",
    "8 AM, 08",
    "9 AM, 09",
    "10 AM, 10",
    "11 AM, 11",
    "Noon, 12",
    "1 PM, 13",
    "2 PM, 14",
    "3 PM, 15",
    "4 PM, 16",
    "5 PM, 17",
    "6 PM, 18",
    "7 PM, 19",
    "8 PM, 20",
    "9 PM, 21",
    "10 PM, 22",
    "11 PM, 23",
    "Midnight, 24",
];

/// Advanced: log level, download folder, cache info, link checking.
fn build_advanced_tab(panel: &Panel, config: &AppConfig) -> (Choice, TextCtrl, CheckBox, CheckBox) {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Logging
    let log_sec = section(panel, "Logging");

    let log_row = BoxSizer::builder(Orientation::Horizontal).build();
    let log_label = StaticText::builder(panel).with_label("Log level:").build();
    let log_choices: Vec<String> = ["Error", "Warn", "Info", "Debug", "Trace"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let log_idx: u32 = match config.log_level.as_str() {
        "error" => 0,
        "warn" => 1,
        "debug" => 3,
        "trace" => 4,
        _ => 2,
    };
    let log_choice = Choice::builder(panel)
        .with_choices(log_choices)
        .with_selection(Some(log_idx))
        .build();
    set_accessible_name(&log_choice, "Log level");
    log_row.add(
        &log_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    log_row.add(&log_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    log_sec.add_sizer(&log_row, 0, SizerFlag::Expand, 0);

    sizer.add_sizer(&log_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Storage
    let store_sec = section(panel, "Storage");

    let dl_row = BoxSizer::builder(Orientation::Horizontal).build();
    let dl_label = StaticText::builder(panel)
        .with_label("Download folder:")
        .build();
    let dl_field = TextCtrl::builder(panel).build();
    set_accessible_name(&dl_field, "Download folder");
    dl_field.set_value(&config.download_folder.to_string_lossy());
    let dl_browse = Button::builder(panel).with_label("&Browse...").build();

    dl_browse.on_click({
        let dl_field_clone = dl_field;
        let panel_ref = *panel;
        move |_ev| {
            let dir_dlg = DirDialog::builder(&panel_ref, "Select download folder", "").build();
            if dir_dlg.show_modal() == ID_OK
                && let Some(path) = dir_dlg.get_path()
            {
                dl_field_clone.set_value(&path);
            }
        }
    });

    dl_row.add(
        &dl_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    dl_row.add(&dl_field, 1, SizerFlag::Expand | SizerFlag::All, 4);
    dl_row.add(&dl_browse, 0, SizerFlag::All, 4);
    store_sec.add_sizer(&dl_row, 0, SizerFlag::Expand, 0);

    let cache_hint = StaticText::builder(panel)
        .with_label("Message cache is stored in the system cache directory.\nClearing cache will require re-downloading messages.")
        .build();
    store_sec.add(&cache_hint, 0, SizerFlag::All, 4);

    sizer.add_sizer(&store_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Checking whether a message is what it says it is
    //
    // Two boxes, deliberately apart. The first reads the message on this
    // computer and sends nothing, so it is on unless somebody turns it off.
    // The second can put four bytes of a link on the wire, so it is off unless
    // somebody turns it on. Sharing one switch would mean agreeing to the
    // second to get the first.
    let links_sec = section(panel, "Checking whether a message is what it says it is");

    let body_box = CheckBox::builder(panel)
        .with_label("Read each message on this computer and mark suspicious ones")
        .build();
    set_accessible_name(
        &body_box,
        "Read each message on this computer and mark suspicious ones",
    );
    body_box.set_value(config.look_at_message_contents);
    links_sec.add(&body_box, 0, SizerFlag::All, 4);

    let body_hint = StaticText::builder(panel)
        .with_label(crate::application::body_safety::LOOKING_AT_THE_MESSAGE_ITSELF)
        .build();
    links_sec.add(&body_hint, 0, SizerFlag::All, 4);

    let links_box = CheckBox::builder(panel)
        .with_label("Check links against Google Safe Browsing")
        .build();
    set_accessible_name(&links_box, "Check links against Google Safe Browsing");
    links_box.set_value(config.check_links_with_google);
    links_sec.add(&links_box, 0, SizerFlag::All, 4);

    let links_hint = StaticText::builder(panel)
        .with_label(
            "Google's lists of known phishing and malware sites are downloaded to 
             this computer, and links are compared here. Your links are not sent to 
             Google. Only if a link matches one of the downloaded entries do four 
             bytes of it go to Google to confirm, and those four bytes stand for 
             millions of possible addresses. Nothing about the sender, the subject 
             or the message is ever sent.
             
             Needs a Google API key in oauth.toml. Without one this does nothing.",
        )
        .build();
    links_sec.add(&links_hint, 0, SizerFlag::All, 4);

    sizer.add_sizer(&links_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    (log_choice, dl_field, body_box, links_box)
}

/// Feedback channels: how the application tells you something happened.
///
/// One checkbox per channel rather than a grid of events, because the choice
/// people actually make is "words, not sounds" or "sounds, not words". The
/// per-event overrides exist in the model for anyone who wants them and are
/// not worth forty checkboxes here.
///
/// Nothing here can produce a sound-only application by accident: the routing
/// adds a written equivalent unless every text channel is off, and the wording
/// says so rather than leaving it to be discovered.
fn build_feedback_tab(
    panel: &Panel,
    config: &AppConfig,
    a11y: &Arc<Accessibility>,
) -> (Vec<(Channel, CheckBox)>, Choice) {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    let settings = FeedbackSettings::from_stored(&config.feedback_channels);

    let intro = StaticText::builder(panel)
        .with_label(
            "Choose how Wixen Mail tells you about new mail, sent messages, connection \
             changes, and errors. Sounds never replace words unless you switch every \
             other channel off yourself.",
        )
        .build();
    sizer.add(&intro, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let sec = section(panel, "Channels");
    // Each box carries the channel it switches. The wording comes off the
    // channel too, so there is no second list to fall out of step with this
    // one and no position to pair by.
    let mut boxes = Vec::new();
    for channel in Channel::ALL {
        let label = channel.setting_label();
        let cb = CheckBox::builder(panel).with_label(label).build();
        set_accessible_name(&cb, &name_from_label(label));
        cb.set_value(settings.is_channel_enabled(channel));
        sec.add(&cb, 0, SizerFlag::All, 4);
        boxes.push((channel, cb));
    }
    sizer.add_sizer(&sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let note = StaticText::builder(panel)
        .with_label(
            "Each event is given its own tone, and sounds are spaced out so a busy \
             mailbox does not run them together. Whether every pair really is easy \
             to tell apart by ear has not been tested yet, so please say if two of \
             them sound alike.",
        )
        .build();
    sizer.add(&note, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let scheme_sec = section(panel, "Sound");
    let scheme_row = BoxSizer::builder(Orientation::Horizontal).build();
    let scheme_label = StaticText::builder(panel)
        .with_label("Sound scheme:")
        .build();
    let schemes = discovered_schemes();
    let scheme_choices: Vec<String> = schemes.iter().map(|s| s.name.clone()).collect();
    let scheme_idx = schemes
        .iter()
        .position(|s| s.id == config.sound_scheme_id)
        .unwrap_or(0) as u32;
    let scheme_choice = Choice::builder(panel)
        .with_choices(scheme_choices)
        .with_selection(Some(scheme_idx))
        .build();
    set_accessible_name(&scheme_choice, "Sound scheme");
    scheme_row.add(
        &scheme_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::Right,
        8,
    );
    scheme_row.add(&scheme_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    scheme_sec.add_sizer(&scheme_row, 0, SizerFlag::Expand, 0);

    let scheme_btn_row = BoxSizer::builder(Orientation::Horizontal).build();
    let import_btn = Button::builder(panel)
        .with_label("&Import sound scheme...")
        .build();
    set_accessible_name(&import_btn, "Import sound scheme");
    scheme_btn_row.add(&import_btn, 0, SizerFlag::Right, 4);

    let delete_btn = Button::builder(panel)
        .with_label("&Delete sound scheme")
        .build();
    set_accessible_name(&delete_btn, "Delete sound scheme");
    // Disabled the moment there is nothing but the built-in default to act
    // on: a button that always opens a confirmation only to refuse whatever
    // was chosen teaches nothing except to stop trying it.
    delete_btn.enable(schemes.len() > 1);
    scheme_btn_row.add(&delete_btn, 0, SizerFlag::Left, 4);
    scheme_sec.add_sizer(&scheme_btn_row, 0, SizerFlag::All, 4);

    import_btn.on_click({
        let panel = *panel;
        let a11y = a11y.clone();
        move |_| import_sound_scheme(&panel, &scheme_choice, &delete_btn, &a11y)
    });
    delete_btn.on_click({
        let panel = *panel;
        let a11y = a11y.clone();
        move |_| delete_sound_scheme(&panel, &scheme_choice, &delete_btn, &a11y)
    });

    sizer.add_sizer(&scheme_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    (boxes, scheme_choice)
}

/// Rebuild the picker's items and the delete button's enabled state from
/// the real current disk state.
///
/// `select_id` is looked for in the freshly discovered list, not assumed
/// still to be there: the caller may be selecting what it just imported, or
/// falling back to Generated because what was selected is what just got
/// deleted. Either way this is the one place that decides which index that
/// becomes, so the picker and the button can never show one truth while the
/// disk holds another.
fn refresh_scheme_controls(scheme_choice: &Choice, delete_btn: &Button, select_id: &str) {
    let schemes = discovered_schemes();
    scheme_choice.clear();
    for candidate in &schemes {
        scheme_choice.append(&candidate.name);
    }
    let index = schemes.iter().position(|s| s.id == select_id).unwrap_or(0);
    scheme_choice.set_selection(index as u32);
    delete_btn.enable(schemes.len() > 1);
}

/// Picking a zip, importing it, and refreshing the picker to include it.
///
/// Announced through `a11y` rather than only shown, the same as everything
/// else this dialog reports: a picker whose new item appeared with no
/// sentence to go with it is invisible to anyone not looking at the screen
/// at the exact moment it changed.
fn import_sound_scheme(
    panel: &Panel,
    scheme_choice: &Choice,
    delete_btn: &Button,
    a11y: &Arc<Accessibility>,
) {
    let picker = FileDialog::builder(panel)
        .with_message("Import sound scheme")
        .with_wildcard("Sound scheme packs (*.zip)|*.zip")
        .with_style(FileDialogStyle::Open | FileDialogStyle::FileMustExist)
        .build();
    if picker.show_modal() != ID_OK {
        // Cancelling is a decision, not an outcome to report.
        return;
    }
    let Some(path) = picker.get_path() else {
        let _ = a11y.announce(
            "No file was chosen",
            crate::presentation::accessibility::announcements::Priority::High,
        );
        return;
    };
    let zip_path = std::path::Path::new(&path);
    let stem = zip_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("imported");
    let id = sound_scheme_import::slug_for(stem);

    let Ok(paths) = AppPaths::resolve() else {
        let _ = a11y.announce(
            "The sound-schemes folder could not be found",
            crate::presentation::accessibility::announcements::Priority::High,
        );
        return;
    };

    match sound_scheme_import::import_zip(zip_path, &id, &paths.sound_schemes_dir()) {
        Ok(scheme) => {
            refresh_scheme_controls(scheme_choice, delete_btn, &scheme.id);
            let _ = a11y.announce(
                &format!(
                    "Imported {}, covers {} of {} events",
                    scheme.name,
                    scheme.covers(),
                    crate::presentation::accessibility::feedback::Event::ALL.len()
                ),
                crate::presentation::accessibility::announcements::Priority::High,
            );
        }
        Err(err) => {
            let _ = a11y.announce(
                &format!("Could not import that sound scheme: {err}"),
                crate::presentation::accessibility::announcements::Priority::High,
            );
        }
    }
}

/// Confirming, deleting, and refreshing the picker to drop what is gone.
///
/// The selected scheme is read fresh from `discovered_schemes()` rather
/// than trusted from whatever was on screen when the dialog was built: an
/// import earlier in this same dialog session can have changed what index
/// `scheme_choice`'s selection actually points at.
fn delete_sound_scheme(
    panel: &Panel,
    scheme_choice: &Choice,
    delete_btn: &Button,
    a11y: &Arc<Accessibility>,
) {
    let schemes = discovered_schemes();
    let selected = schemes.get(sel(scheme_choice) as usize);
    let Some(scheme) = selected.filter(|s| s.id != "generated") else {
        // Reachable even with the button enabled: the picker's own
        // selection can sit on Generated while an imported scheme exists
        // alongside it. Said plainly rather than silently doing nothing,
        // which reads as a broken button.
        let _ = a11y.announce(
            "Generated tones is the built-in default and cannot be deleted. \
             Choose an imported scheme first.",
            crate::presentation::accessibility::announcements::Priority::High,
        );
        return;
    };

    let question = format!("Delete \"{}\"? This cannot be undone.", scheme.name);
    // Enter answers No, the same reasoning every other delete confirmation
    // in this application uses: a key one row away from every other key
    // somebody might have meant should not be the one that finishes an
    // irreversible action if pressed early.
    let asked = MessageDialog::builder(panel, &question, "Delete")
        .with_style(crate::presentation::asking::yes_no_where_enter_answers_no())
        .build()
        .show_modal();
    if asked != ID_YES {
        return;
    }

    let Ok(paths) = AppPaths::resolve() else {
        let _ = a11y.announce(
            "The sound-schemes folder could not be found",
            crate::presentation::accessibility::announcements::Priority::High,
        );
        return;
    };
    match SoundScheme::delete(&scheme.id, &paths.sound_schemes_dir()) {
        Ok(()) => {
            let name = scheme.name.clone();
            refresh_scheme_controls(scheme_choice, delete_btn, "generated");
            let _ = a11y.announce(
                &format!("Deleted {name}"),
                crate::presentation::accessibility::announcements::Priority::High,
            );
        }
        Err(err) => {
            let _ = a11y.announce(
                &format!("Could not delete that sound scheme: {err}"),
                crate::presentation::accessibility::announcements::Priority::High,
            );
        }
    }
}

/// Every sound scheme this installation can currently offer: the built-in
/// default first, then whatever real packs are sitting in the sound-schemes
/// folder. Falls back to just the default if the folder itself cannot be
/// resolved, the same graceful degradation `SoundScheme::discover` already
/// gives an unreadable or missing folder.
fn discovered_schemes() -> Vec<SoundScheme> {
    match AppPaths::resolve() {
        Ok(paths) => SoundScheme::discover(&paths.sound_schemes_dir()),
        Err(_) => vec![SoundScheme::generated()],
    }
}

// ── Read settings back from widget references ────────────────────────────────

/// Collect current widget values and produce an updated `AppConfig`.
fn read_settings(w: &SettingsWidgets, base: &AppConfig) -> AppConfig {
    let mut cfg = base.clone();

    // Feedback channels. The per-event overrides in the stored value are
    // preserved: this tab only decides which channels are on at all.
    let mut feedback = FeedbackSettings::from_stored(&base.feedback_channels);
    for (channel, cb) in &w.feedback {
        feedback.set_channel_enabled(*channel, cb.get_value());
    }
    cfg.feedback_channels = feedback.to_stored();

    // The scheme picker's own order is whatever discovery produced when the
    // dialog was built; reading it back the same way is what makes the
    // selection index mean the same scheme it meant a moment ago.
    let schemes = discovered_schemes();
    cfg.sound_scheme_id = schemes
        .get(sel(&w.sound_scheme) as usize)
        .map(|s| s.id.clone())
        .unwrap_or_default();

    // What may be changed at a server. Read back as two answers, because they
    // are two: sending cannot be undone, a task can be moved back.
    cfg.allowed_changes = crate::application::allowed::Allowed {
        mail: w.allow_mail.get_value(),
        personal_information: w.allow_pim.get_value(),
    };
    cfg.send_contact_changes_everywhere = w.send_contact_changes_everywhere.get_value();

    // General
    cfg.theme = match sel(&w.theme) {
        1 => "light",
        2 => "dark",
        3 => "high_contrast",
        _ => "default",
    }
    .to_string();
    cfg.font_size = w
        .font_size
        .get_value()
        .parse::<u32>()
        .unwrap_or(base.font_size)
        .clamp(8, 72);

    // Compose
    cfg.preview_before_send = w.preview_before_send.get_value();
    cfg.keep_sent_mail_on_this_computer = w.keep_sent_mail_on_this_computer.get_value();
    cfg.draft_autosave_minutes =
        AutosaveInterval::from_setting(w.draft_autosave.value().max(0) as u32).minutes();
    cfg.add_signature_automatically = w.add_signature_automatically.get_value();

    // Reading
    cfg.start_in_all_inboxes = w.start_in_all_inboxes.get_value();
    cfg.hold_back_remote_pictures = w.hold_back_remote_pictures.get_value();
    cfg.smooth_scrolling = w.smooth_scrolling.get_value();
    cfg.keep_running_in_the_tray = w.keep_running_in_the_tray.get_value();
    // By the words shown rather than the row number. A row number needs the
    // installed list a second time to mean anything, and if that list differed
    // at saving from the one somebody chose from, their choice would be stored
    // as a different font or quietly reset.
    cfg.font_family = w
        .font_family
        .get_string_selection()
        .map(|chosen| crate::application::font_choice::what_the_words_store(&chosen))
        .unwrap_or_else(|| cfg.font_family.clone());
    // By the words shown rather than the row number, for the same reason
    // `font_family` above gives. Words nothing recognises fall to the default
    // rather than to whichever branch is written first, which is what a
    // hand-edited settings file gets.
    cfg.unread_on_a_parent = w
        .unread_on_a_parent
        .get_string_selection()
        .map(|chosen| UnreadOnAParent::from_words(&chosen))
        .unwrap_or_else(|| UnreadOnAParent::from_stored(&cfg.unread_on_a_parent))
        .as_str()
        .to_string();
    // D-34 and D-35, written back separately because they are two answers.
    cfg.empty_reaches_subfolders = w.empty_reaches_subfolders.get_value();
    cfg.mark_read_reaches_subfolders = w.mark_read_reaches_subfolders.get_value();
    cfg.check_default_programs_at_startup = w.check_default_programs_at_startup.get_value();
    cfg.keep_selected_message_in_view = w.keep_selected_message_in_view.get_value();
    cfg.default_sort_order = match sel(&w.sort_order) {
        1 => "date_oldest",
        2 => "sender_az",
        3 => "sender_za",
        4 => "subject_az",
        5 => "subject_za",
        6 => "unread_first",
        _ => "date_newest",
    }
    .to_string();

    // Read receipts. Read by position out of `Policy::ALL`, which is the same
    // order the choices were built from, so the two cannot drift apart the way
    // a second list of words would.
    cfg.read_messages_as = ReadingStyle::ALL
        .get(sel(&w.read_messages_as) as usize)
        .copied()
        .unwrap_or_default()
        .as_str()
        .to_string();

    // Dates and times, read in every module.
    cfg.date_style = match sel(&w.date_style) {
        1 => "absolute",
        _ => "relative",
    }
    .to_string();
    cfg.date_order = match sel(&w.date_order) {
        1 => "month_first",
        2 => "day_first",
        _ => "auto",
    }
    .to_string();
    cfg.date_wording = match sel(&w.date_wording) {
        1 => "numeric",
        _ => "verbal",
    }
    .to_string();
    cfg.clock_hours = match sel(&w.clock_hours) {
        1 => "12",
        2 => "24",
        _ => "auto",
    }
    .to_string();

    cfg.mark_read_after = MarkRead::ALL
        .get(sel(&w.mark_read_after) as usize)
        .copied()
        .unwrap_or_default()
        .as_stored();
    cfg.copy_lines = match sel(&w.copy_lines) {
        1 => CopyLines::Hidden,
        _ => CopyLines::Shown,
    }
    .as_stored()
    .to_string();
    cfg.message_columns = with_second_level(&base.message_columns, sel(&w.sort_then));

    cfg.read_receipts = Policy::ALL
        .get(sel(&w.read_receipts) as usize)
        .copied()
        .unwrap_or_default()
        .as_str()
        .to_string();

    // Language. Rebuilt rather than remembered, because it is what the picker
    // was filled from and the two have to stay the same list.
    let languages = available_languages();
    let idx = sel(&w.language) as usize;
    if idx < languages.len() {
        cfg.language = languages[idx].tag.clone();
    }
    cfg.check_spelling_before_send = w.check_spelling_before_send.is_checked();
    cfg.check_spelling_as_you_type = w.check_spelling_as_you_type.is_checked();

    // Calendar & PIM
    // Kept through the same check the calendar reads it through, so a day
    // that ends before it starts never reaches the file.
    let day = WorkingDay::from_setting(sel(&w.day_starts) as u8, sel(&w.day_ends) as u8);
    cfg.working_day_starts = day.starts;
    cfg.working_day_ends = day.ends;

    cfg.default_reminder_minutes = w
        .default_reminder
        .get_value()
        .parse::<u32>()
        .unwrap_or(base.default_reminder_minutes)
        .min(1440);

    // Advanced
    cfg.log_level = match sel(&w.log_level) {
        0 => "error",
        1 => "warn",
        3 => "debug",
        4 => "trace",
        _ => "info",
    }
    .to_string();
    let path = w.download_folder.get_value();
    if !path.is_empty() {
        cfg.download_folder = std::path::PathBuf::from(path);
    }
    cfg.look_at_message_contents = w.look_at_message_contents.is_checked();
    cfg.check_links_with_google = w.check_links_with_google.is_checked();

    cfg
}

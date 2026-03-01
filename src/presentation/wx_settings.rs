//! Settings / Preferences dialog
//!
//! A tabbed dialog accessible from Tools > Settings that exposes the most
//! commonly used email client configuration options.  Settings are read from
//! and persisted through `AppConfig` / `ConfigManager`.

use crate::data::config::AppConfig;
use crate::service::spellcheck::supported_languages;
use wxdragon::prelude::*;

// ── Result type ──────────────────────────────────────────────────────────────

/// The outcome of the settings dialog.
pub enum SettingsResult {
    /// User pressed OK — contains the (possibly modified) AppConfig.
    Updated(AppConfig),
    /// User cancelled — no changes.
    Cancelled,
}

// ── Widget references ────────────────────────────────────────────────────────

/// Holds references to all mutable settings widgets so we can read them back
/// when the user presses OK.
struct SettingsWidgets {
    // General
    theme: Choice,
    font_size: TextCtrl,
    notifications: CheckBox,
    check_updates: CheckBox,
    // Compose
    preview_before_send: CheckBox,
    // Reading
    sort_order: Choice,
    // Language
    language: Choice,
    // Advanced
    log_level: Choice,
    download_folder: TextCtrl,
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
pub fn show_settings_dialog(parent: &Frame, config: &AppConfig) -> SettingsResult {
    let dlg = Dialog::builder(parent, "Settings")
        .with_size(560, 520)
        .build();

    let root_sizer = BoxSizer::builder(Orientation::Vertical).build();

    // Notebook (tabbed pane)
    let notebook = Notebook::builder(&dlg).build();

    // ── Tab 1: General
    let general_panel = Panel::builder(&notebook).build();
    let (theme, font_size, notifications, check_updates) =
        build_general_tab(&general_panel, config);
    notebook.add_page(&general_panel, "General", true, None);

    // ── Tab 2: Compose
    let compose_panel = Panel::builder(&notebook).build();
    let preview_before_send = build_compose_tab(&compose_panel, config);
    notebook.add_page(&compose_panel, "Compose", false, None);

    // ── Tab 3: Reading
    let reading_panel = Panel::builder(&notebook).build();
    let sort_order = build_reading_tab(&reading_panel, config);
    notebook.add_page(&reading_panel, "Reading", false, None);

    // ── Tab 4: Language & Spelling
    let lang_panel = Panel::builder(&notebook).build();
    let language = build_language_tab(&lang_panel, config);
    notebook.add_page(&lang_panel, "Language", false, None);

    // ── Tab 5: Advanced
    let advanced_panel = Panel::builder(&notebook).build();
    let (log_level, download_folder) = build_advanced_tab(&advanced_panel, config);
    notebook.add_page(&advanced_panel, "Advanced", false, None);

    root_sizer.add(&notebook, 1, SizerFlag::Expand | SizerFlag::All, 8);

    // ── OK / Cancel buttons
    let btn_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    btn_sizer.add_spacer(0);
    let ok_btn = Button::builder(&dlg).with_label("OK").with_id(ID_OK).build();
    let cancel_btn = Button::builder(&dlg).with_label("Cancel").with_id(ID_CANCEL).build();
    btn_sizer.add(&ok_btn, 0, SizerFlag::All, 4);
    btn_sizer.add(&cancel_btn, 0, SizerFlag::All, 4);
    root_sizer.add_sizer(&btn_sizer, 0, SizerFlag::AlignRight | SizerFlag::All, 8);

    dlg.set_sizer(root_sizer, true);

    ok_btn.on_click({ let d = dlg; move |_ev| { d.end_modal(ID_OK); } });
    cancel_btn.on_click({ let d = dlg; move |_ev| { d.end_modal(ID_CANCEL); } });

    let widgets = SettingsWidgets {
        theme, font_size, notifications, check_updates,
        preview_before_send, sort_order, language,
        log_level, download_folder,
    };

    if dlg.show_modal() == ID_OK {
        SettingsResult::Updated(read_settings(&widgets, config))
    } else {
        SettingsResult::Cancelled
    }
}

// ── Tab builders ─────────────────────────────────────────────────────────────

/// General settings: theme, font size, notifications, check-for-updates.
fn build_general_tab(
    panel: &Panel,
    config: &AppConfig,
) -> (Choice, TextCtrl, CheckBox, CheckBox) {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Appearance
    let app_sec = section(panel, "Appearance");

    let theme_row = BoxSizer::builder(Orientation::Horizontal).build();
    let theme_label = StaticText::builder(panel).with_label("Theme:").build();
    let theme_choices: Vec<String> = ["Default", "Light", "Dark", "High Contrast"]
        .iter().map(|s| s.to_string()).collect();
    let theme_idx: u32 = match config.theme.as_str() {
        "light" => 1, "dark" => 2, "high_contrast" => 3, _ => 0,
    };
    let theme_choice = Choice::builder(panel)
        .with_choices(theme_choices)
        .with_selection(Some(theme_idx))
        .build();
    theme_row.add(&theme_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    theme_row.add(&theme_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    app_sec.add_sizer(&theme_row, 0, SizerFlag::Expand, 0);

    let font_row = BoxSizer::builder(Orientation::Horizontal).build();
    let font_label = StaticText::builder(panel).with_label("Font size:").build();
    let font_field = TextCtrl::builder(panel).build();
    font_field.set_value(&config.font_size.to_string());
    font_row.add(&font_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    font_row.add(&font_field, 0, SizerFlag::All, 4);
    app_sec.add_sizer(&font_row, 0, SizerFlag::Expand, 0);

    sizer.add_sizer(&app_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Notifications
    let notif_sec = section(panel, "Notifications");
    let notif_cb = CheckBox::builder(panel).with_label("Enable &new-mail notifications").build();
    notif_cb.set_value(config.enable_notifications);
    notif_sec.add(&notif_cb, 0, SizerFlag::All, 4);
    sizer.add_sizer(&notif_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Updates
    let upd_sec = section(panel, "Updates");
    let update_cb = CheckBox::builder(panel).with_label("Check for &updates on startup").build();
    update_cb.set_value(config.check_updates);
    upd_sec.add(&update_cb, 0, SizerFlag::All, 4);
    sizer.add_sizer(&upd_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    (theme_choice, font_field, notif_cb, update_cb)
}

/// Compose settings: preview-before-send, default format, signatures.
fn build_compose_tab(panel: &Panel, config: &AppConfig) -> CheckBox {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Sending
    let send_sec = section(panel, "Sending");
    let preview_cb = CheckBox::builder(panel).with_label("Show &preview before sending").build();
    preview_cb.set_value(config.preview_before_send);
    send_sec.add(&preview_cb, 0, SizerFlag::All, 4);

    let format_row = BoxSizer::builder(Orientation::Horizontal).build();
    let format_label = StaticText::builder(panel).with_label("Default format:").build();
    let format_choices: Vec<String> = ["HTML", "Plain Text"]
        .iter().map(|s| s.to_string()).collect();
    let format_choice = Choice::builder(panel)
        .with_choices(format_choices)
        .with_selection(Some(0))
        .build();
    format_row.add(&format_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    format_row.add(&format_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    send_sec.add_sizer(&format_row, 0, SizerFlag::Expand, 0);

    sizer.add_sizer(&send_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Drafts
    let draft_sec = section(panel, "Drafts");
    let autosave_cb = CheckBox::builder(panel).with_label("&Auto-save drafts every 60 seconds").build();
    autosave_cb.set_value(true);
    draft_sec.add(&autosave_cb, 0, SizerFlag::All, 4);
    sizer.add_sizer(&draft_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Signatures
    let sig_sec = section(panel, "Signatures");
    let sig_cb = CheckBox::builder(panel).with_label("Automatically insert &signature on new messages").build();
    sig_cb.set_value(true);
    sig_sec.add(&sig_cb, 0, SizerFlag::All, 4);
    sizer.add_sizer(&sig_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    preview_cb
}

/// Reading settings: sort order, mark-as-read, threading.
fn build_reading_tab(panel: &Panel, config: &AppConfig) -> Choice {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Message List
    let list_sec = section(panel, "Message List");

    let sort_row = BoxSizer::builder(Orientation::Horizontal).build();
    let sort_label = StaticText::builder(panel).with_label("Default sort order:").build();
    let sort_choices: Vec<String> = [
        "Date (Newest First)", "Date (Oldest First)",
        "Sender (A-Z)", "Sender (Z-A)",
        "Subject (A-Z)", "Subject (Z-A)",
        "Unread First",
    ].iter().map(|s| s.to_string()).collect();
    let sort_idx: u32 = match config.default_sort_order.as_str() {
        "date_oldest" => 1, "sender_az" => 2, "sender_za" => 3,
        "subject_az" => 4, "subject_za" => 5, "unread_first" => 6,
        _ => 0,
    };
    let sort_choice = Choice::builder(panel)
        .with_choices(sort_choices)
        .with_selection(Some(sort_idx))
        .build();
    sort_row.add(&sort_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    sort_row.add(&sort_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    list_sec.add_sizer(&sort_row, 0, SizerFlag::Expand, 0);

    let thread_cb = CheckBox::builder(panel).with_label("Enable &threaded view by default").build();
    thread_cb.set_value(false);
    list_sec.add(&thread_cb, 0, SizerFlag::All, 4);
    sizer.add_sizer(&list_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Reading Behaviour
    let read_sec = section(panel, "Reading Behaviour");

    let markread_row = BoxSizer::builder(Orientation::Horizontal).build();
    let markread_label = StaticText::builder(panel).with_label("Mark as read after:").build();
    let markread_choices: Vec<String> = ["Immediately", "After 2 seconds", "After 5 seconds", "Manually"]
        .iter().map(|s| s.to_string()).collect();
    let markread_choice = Choice::builder(panel)
        .with_choices(markread_choices)
        .with_selection(Some(0))
        .build();
    markread_row.add(&markread_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    markread_row.add(&markread_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    read_sec.add_sizer(&markread_row, 0, SizerFlag::Expand, 0);

    let external_cb = CheckBox::builder(panel).with_label("Load remote &images in messages").build();
    external_cb.set_value(false);
    read_sec.add(&external_cb, 0, SizerFlag::All, 4);

    sizer.add_sizer(&read_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    sort_choice
}

/// Language & Spelling: language, spell-check toggle.
fn build_language_tab(panel: &Panel, config: &AppConfig) -> Choice {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Language
    let lang_sec = section(panel, "Language");

    let lang_row = BoxSizer::builder(Orientation::Horizontal).build();
    let lang_label = StaticText::builder(panel).with_label("Interface language:").build();

    let languages = supported_languages();
    let lang_names: Vec<String> = languages.iter()
        .map(|l| format!("{} ({})", l.name, l.native_name))
        .collect();
    let lang_idx = languages.iter()
        .position(|l| l.code == config.language)
        .unwrap_or(0) as u32;
    let lang_choice = Choice::builder(panel)
        .with_choices(lang_names)
        .with_selection(Some(lang_idx))
        .build();
    lang_row.add(&lang_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    lang_row.add(&lang_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    lang_sec.add_sizer(&lang_row, 0, SizerFlag::Expand, 0);

    sizer.add_sizer(&lang_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Spell Check
    let spell_sec = section(panel, "Spell Check");

    let enable_spell = CheckBox::builder(panel).with_label("&Enable spell checking in compose editor").build();
    enable_spell.set_value(true);
    spell_sec.add(&enable_spell, 0, SizerFlag::All, 4);

    let autocorrect_cb = CheckBox::builder(panel).with_label("Show su&ggestions as you type").build();
    autocorrect_cb.set_value(true);
    spell_sec.add(&autocorrect_cb, 0, SizerFlag::All, 4);

    let hunspell_note = StaticText::builder(panel)
        .with_label("Hunspell dictionaries are loaded automatically from system paths.\nInstall additional language packs for more spell-check languages.")
        .build();
    spell_sec.add(&hunspell_note, 0, SizerFlag::All, 4);

    sizer.add_sizer(&spell_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    lang_choice
}

/// Advanced: log level, download folder, cache info.
fn build_advanced_tab(panel: &Panel, config: &AppConfig) -> (Choice, TextCtrl) {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Logging
    let log_sec = section(panel, "Logging");

    let log_row = BoxSizer::builder(Orientation::Horizontal).build();
    let log_label = StaticText::builder(panel).with_label("Log level:").build();
    let log_choices: Vec<String> = ["Error", "Warn", "Info", "Debug", "Trace"]
        .iter().map(|s| s.to_string()).collect();
    let log_idx: u32 = match config.log_level.as_str() {
        "error" => 0, "warn" => 1, "debug" => 3, "trace" => 4, _ => 2,
    };
    let log_choice = Choice::builder(panel)
        .with_choices(log_choices)
        .with_selection(Some(log_idx))
        .build();
    log_row.add(&log_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    log_row.add(&log_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    log_sec.add_sizer(&log_row, 0, SizerFlag::Expand, 0);

    sizer.add_sizer(&log_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Storage
    let store_sec = section(panel, "Storage");

    let dl_row = BoxSizer::builder(Orientation::Horizontal).build();
    let dl_label = StaticText::builder(panel).with_label("Download folder:").build();
    let dl_field = TextCtrl::builder(panel).build();
    dl_field.set_value(&config.download_folder.to_string_lossy());
    let dl_browse = Button::builder(panel).with_label("&Browse...").build();

    dl_browse.on_click({
        let dl_field_clone = dl_field;
        let panel_ref = *panel;
        move |_ev| {
            let dir_dlg = DirDialog::builder(&panel_ref, "Select download folder", "").build();
            if dir_dlg.show_modal() == ID_OK {
                if let Some(path) = dir_dlg.get_path() {
                    dl_field_clone.set_value(&path);
                }
            }
        }
    });

    dl_row.add(&dl_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
    dl_row.add(&dl_field, 1, SizerFlag::Expand | SizerFlag::All, 4);
    dl_row.add(&dl_browse, 0, SizerFlag::All, 4);
    store_sec.add_sizer(&dl_row, 0, SizerFlag::Expand, 0);

    let cache_hint = StaticText::builder(panel)
        .with_label("Message cache is stored in the system cache directory.\nClearing cache will require re-downloading messages.")
        .build();
    store_sec.add(&cache_hint, 0, SizerFlag::All, 4);

    sizer.add_sizer(&store_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    (log_choice, dl_field)
}

// ── Read settings back from widget references ────────────────────────────────

/// Collect current widget values and produce an updated `AppConfig`.
fn read_settings(w: &SettingsWidgets, base: &AppConfig) -> AppConfig {
    let mut cfg = base.clone();

    // General
    cfg.theme = match sel(&w.theme) {
        1 => "light", 2 => "dark", 3 => "high_contrast", _ => "default",
    }.to_string();
    cfg.font_size = w.font_size.get_value()
        .parse::<u32>()
        .unwrap_or(base.font_size)
        .max(8).min(72);
    cfg.enable_notifications = w.notifications.get_value();
    cfg.check_updates = w.check_updates.get_value();

    // Compose
    cfg.preview_before_send = w.preview_before_send.get_value();

    // Reading
    cfg.default_sort_order = match sel(&w.sort_order) {
        1 => "date_oldest", 2 => "sender_az", 3 => "sender_za",
        4 => "subject_az", 5 => "subject_za", 6 => "unread_first",
        _ => "date_newest",
    }.to_string();

    // Language
    let languages = supported_languages();
    let idx = sel(&w.language) as usize;
    if idx < languages.len() {
        cfg.language = languages[idx].code.clone();
    }

    // Advanced
    cfg.log_level = match sel(&w.log_level) {
        0 => "error", 1 => "warn", 3 => "debug", 4 => "trace", _ => "info",
    }.to_string();
    let path = w.download_folder.get_value();
    if !path.is_empty() {
        cfg.download_folder = std::path::PathBuf::from(path);
    }

    cfg
}

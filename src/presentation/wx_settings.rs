//! Settings / Preferences dialog
//!
//! A tabbed dialog accessible from Tools > Settings that exposes the most
//! commonly used email client configuration options.  Settings are read from
//! and persisted through `AppConfig` / `ConfigManager`.

use crate::application::autosave::AutosaveInterval;
use crate::application::reading_habits::{CopyLines, MarkRead, WorkingDay};
use crate::application::reading_style::Style as ReadingStyle;
use crate::application::receipts::Policy;
use crate::data::config::AppConfig;
use crate::presentation::accessibility::feedback::{Channel, FeedbackSettings};
use crate::presentation::accessibility::names::{
    set_accessible_name, set_accessible_name_and_description,
};
use crate::service::spellcheck::available_languages;
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
struct SettingsWidgets {
    // General
    theme: Choice,
    font_size: TextCtrl,
    notifications: CheckBox,
    check_updates: CheckBox,
    // Compose
    preview_before_send: CheckBox,
    draft_autosave: SpinCtrl,
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
    // Language
    language: Choice,
    check_spelling_before_send: CheckBox,
    allow_mail: CheckBox,
    allow_pim: CheckBox,
    check_spelling_as_you_type: CheckBox,
    // Calendar & PIM
    cal_default_view: Choice,
    cal_show_weekends: CheckBox,
    cal_first_day: Choice,
    default_reminder: TextCtrl,
    day_starts: Choice,
    day_ends: Choice,
    // Advanced
    log_level: Choice,
    download_folder: TextCtrl,
    check_links_with_google: CheckBox,
    // Feedback channels, one checkbox per channel in Channel::ALL order.
    feedback: Vec<CheckBox>,
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
    let (preview_before_send, draft_autosave) = build_compose_tab(&compose_panel, config);
    notebook.add_page(&compose_panel, "Compose", false, None);

    // ── Tab 3: Reading
    let reading_panel = Panel::builder(&notebook).build();
    let (
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
    ) = build_reading_tab(&reading_panel, config);
    notebook.add_page(&reading_panel, "Reading", false, None);

    // ── Tab 4: Language & Spelling
    let lang_panel = Panel::builder(&notebook).build();
    let (language, check_spelling_before_send, check_spelling_as_you_type, allow_mail, allow_pim) =
        build_language_tab(&lang_panel, config);
    notebook.add_page(&lang_panel, "Language", false, None);

    // ── Tab 5: Calendar & PIM
    let pim_panel = Panel::builder(&notebook).build();
    let (
        cal_default_view,
        cal_show_weekends,
        cal_first_day,
        default_reminder,
        day_starts,
        day_ends,
    ) = build_calendar_pim_tab(&pim_panel, config);
    notebook.add_page(&pim_panel, "Calendar && PIM", false, None);

    // ── Tab 6: Feedback
    let feedback_panel = Panel::builder(&notebook).build();
    let feedback = build_feedback_tab(&feedback_panel, config);
    notebook.add_page(&feedback_panel, "Feedback", false, None);

    // ── Tab 7: Advanced
    let advanced_panel = Panel::builder(&notebook).build();
    let (log_level, download_folder, check_links_with_google) =
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

    let widgets = SettingsWidgets {
        theme,
        font_size,
        notifications,
        check_updates,
        preview_before_send,
        draft_autosave,
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
        language,
        check_spelling_before_send,
        check_spelling_as_you_type,
        allow_mail,
        allow_pim,
        cal_default_view,
        cal_show_weekends,
        cal_first_day,
        default_reminder,
        day_starts,
        day_ends,
        log_level,
        download_folder,
        check_links_with_google,
        feedback,
    };

    if dlg.show_modal() == ID_OK {
        SettingsResult::Updated(Box::new(read_settings(&widgets, config)))
    } else {
        SettingsResult::Cancelled
    }
}

// ── Tab builders ─────────────────────────────────────────────────────────────

/// General settings: theme, font size, notifications, check-for-updates.
fn build_general_tab(panel: &Panel, config: &AppConfig) -> (Choice, TextCtrl, CheckBox, CheckBox) {
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

    // -- Notifications
    let notif_sec = section(panel, "Notifications");
    let notif_cb = CheckBox::builder(panel)
        .with_label("Enable &new-mail notifications")
        .build();
    set_accessible_name(&notif_cb, "Enable new-mail notifications");
    notif_cb.set_value(config.enable_notifications);
    notif_sec.add(&notif_cb, 0, SizerFlag::All, 4);
    sizer.add_sizer(&notif_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Updates
    let upd_sec = section(panel, "Updates");
    let update_cb = CheckBox::builder(panel)
        .with_label("Check for &updates on startup")
        .build();
    set_accessible_name(&update_cb, "Check for updates on startup");
    update_cb.set_value(config.check_updates);
    upd_sec.add(&update_cb, 0, SizerFlag::All, 4);
    sizer.add_sizer(&upd_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    (theme_choice, font_field, notif_cb, update_cb)
}

/// Compose settings: preview-before-send, default format, signatures.
fn build_compose_tab(panel: &Panel, config: &AppConfig) -> (CheckBox, SpinCtrl) {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Sending
    let send_sec = section(panel, "Sending");
    let preview_cb = CheckBox::builder(panel)
        .with_label("Show &preview before sending")
        .build();
    set_accessible_name(&preview_cb, "Show preview before sending");
    preview_cb.set_value(config.preview_before_send);
    send_sec.add(&preview_cb, 0, SizerFlag::All, 4);

    let format_row = BoxSizer::builder(Orientation::Horizontal).build();
    let format_label = StaticText::builder(panel)
        .with_label("Default format:")
        .build();
    let format_choices: Vec<String> = ["HTML", "Plain Text"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let format_choice = Choice::builder(panel)
        .with_choices(format_choices)
        .with_selection(Some(0))
        .build();
    set_accessible_name(&format_choice, "Default format");
    format_row.add(
        &format_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    format_row.add(&format_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    send_sec.add_sizer(&format_row, 0, SizerFlag::Expand, 0);

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
    let sig_sec = section(panel, "Signatures");
    let sig_cb = CheckBox::builder(panel)
        .with_label("Automatically insert &signature on new messages")
        .build();
    set_accessible_name(&sig_cb, "Automatically insert signature on new messages");
    sig_cb.set_value(true);
    sig_sec.add(&sig_cb, 0, SizerFlag::All, 4);
    sizer.add_sizer(&sig_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    (preview_cb, autosave_spin)
}

/// Reading settings: sort order, mark-as-read, threading.
#[allow(clippy::type_complexity)]
fn build_reading_tab(
    panel: &Panel,
    config: &AppConfig,
) -> (
    Choice,
    Choice,
    Choice,
    Choice,
    Choice,
    Choice,
    Choice,
    Choice,
    Choice,
    Choice,
) {
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

    let thread_cb = CheckBox::builder(panel)
        .with_label("Enable &threaded view by default")
        .build();
    set_accessible_name(&thread_cb, "Enable threaded view by default");
    thread_cb.set_value(false);
    list_sec.add(&thread_cb, 0, SizerFlag::All, 4);
    sizer.add_sizer(&list_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

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
        MarkRead::ALL
            .iter()
            .position(|c| *c == MarkRead::from_setting(&config.mark_read_after))
            .unwrap_or(0) as u32,
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
        "Formatted keeps the sender's headings, links and tables. Plain text          gives you a caret to move through the message with, and flattens them.",
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
        "A read receipt tells the sender your address is live and roughly when          you were at your desk. Nothing is sent unless you choose it here.",
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

    let external_cb = CheckBox::builder(panel)
        .with_label("Load remote &images in messages")
        .build();
    set_accessible_name(&external_cb, "Load remote images in messages");
    external_cb.set_value(false);
    read_sec.add(&external_cb, 0, SizerFlag::All, 4);

    sizer.add_sizer(&read_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    // -- Dates and times
    //
    // Read in every module, and until now settable in none of them: the values
    // were in the settings file with nothing that could change them. What a
    // date sounds like is not a detail to somebody who hears every one of them.
    let date_sec = section(panel, "Dates and Times");

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
    (
        sort_choice,
        receipt_choice,
        style_choice,
        date_style,
        date_order,
        date_wording,
        clock_hours,
        markread_choice,
        sort_then,
        copy_lines,
    )
}

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
fn build_language_tab(
    panel: &Panel,
    config: &AppConfig,
) -> (Choice, CheckBox, CheckBox, CheckBox, CheckBox) {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Language
    //
    // "Interface language" was the wrong label. Nothing here is translated, and
    // this setting has only ever decided which dictionary the spell checker
    // uses. The list was wrong too: it offered the same six languages whatever
    // the machine had, so picking one it could not check set a value that
    // changed nothing, and the only way to find that out was to write in that
    // language and have every word of it called a mistake.
    let lang_sec = section(panel, "Language");

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
    set_accessible_name(&lang_choice, "Check spelling in");
    lang_row.add(
        &lang_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    lang_row.add(&lang_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    lang_sec.add_sizer(&lang_row, 0, SizerFlag::Expand, 0);

    sizer.add_sizer(&lang_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

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
    let allowed_sec = section(panel, "Allowed Changes");

    let allow_pim = CheckBox::builder(panel)
        .with_label("Let Wixen Mail change my &tasks, contacts and calendar")
        .build();
    set_accessible_name(
        &allow_pim,
        "Let Wixen Mail change my tasks, contacts and calendar",
    );
    allow_pim.set_value(config.allowed_changes.personal_information);
    allowed_sec.add(&allow_pim, 0, SizerFlag::All, 4);

    let allow_mail = CheckBox::builder(panel)
        .with_label("Let Wixen Mail &send and delete mail")
        .build();
    set_accessible_name(&allow_mail, "Let Wixen Mail send and delete mail");
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

    let spell_sec = section(panel, "Spell Check");

    let check_before_send = CheckBox::builder(panel)
        .with_label("&Check spelling before sending a message")
        .build();
    set_accessible_name(
        &check_before_send,
        "Check spelling before sending a message",
    );
    check_before_send.set_value(config.check_spelling_before_send);
    spell_sec.add(&check_before_send, 0, SizerFlag::All, 4);

    let check_as_you_type = CheckBox::builder(panel)
        .with_label("&Mark misspelled words as I write")
        .build();
    set_accessible_name(&check_as_you_type, "Mark misspelled words as I write");
    check_as_you_type.set_value(config.check_spelling_as_you_type);
    spell_sec.add(&check_as_you_type, 0, SizerFlag::All, 4);

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
    spell_sec.add(&marking_note, 0, SizerFlag::All, 4);

    let speller = crate::service::spellcheck::for_language(&config.language);
    let checker_note = StaticText::builder(panel)
        .with_label(&format!(
            "Spelling is checked by {}.",
            speller.source().describe()
        ))
        .build();
    spell_sec.add(&checker_note, 0, SizerFlag::All, 4);

    sizer.add_sizer(&spell_sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    (
        lang_choice,
        check_before_send,
        check_as_you_type,
        allow_mail,
        allow_pim,
    )
}

/// Calendar & PIM settings: default view, weekends, first day, reminder time.
fn build_calendar_pim_tab(
    panel: &Panel,
    config: &AppConfig,
) -> (Choice, CheckBox, Choice, TextCtrl, Choice, Choice) {
    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // -- Calendar View
    let view_sec = section(panel, "Calendar");

    let view_row = BoxSizer::builder(Orientation::Horizontal).build();
    let view_label = StaticText::builder(panel)
        .with_label("Default &view:")
        .build();
    let view_choices: Vec<String> = ["Agenda", "Day", "Week", "Month"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let view_idx: u32 = match config.calendar_default_view.as_str() {
        "day" => 1,
        "week" => 2,
        "month" => 3,
        _ => 0,
    };
    let view_choice = Choice::builder(panel)
        .with_choices(view_choices)
        .with_selection(Some(view_idx))
        .build();
    set_accessible_name(&view_choice, "Default view");
    view_row.add(
        &view_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    view_row.add(&view_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    view_sec.add_sizer(&view_row, 0, SizerFlag::Expand, 0);

    let weekends_cb = CheckBox::builder(panel)
        .with_label("Show &weekends")
        .build();
    set_accessible_name(&weekends_cb, "Show weekends");
    weekends_cb.set_value(config.calendar_show_weekends);
    view_sec.add(&weekends_cb, 0, SizerFlag::All, 4);

    let first_day_row = BoxSizer::builder(Orientation::Horizontal).build();
    let first_day_label = StaticText::builder(panel)
        .with_label("&First day of week:")
        .build();
    let day_choices: Vec<String> = ["Sunday", "Monday", "Saturday"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let day_idx: u32 = match config.calendar_first_day_of_week {
        1 => 1,
        6 => 2,
        _ => 0,
    };
    let first_day_choice = Choice::builder(panel)
        .with_choices(day_choices)
        .with_selection(Some(day_idx))
        .build();
    set_accessible_name(&first_day_choice, "First day of week");
    first_day_row.add(
        &first_day_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    first_day_row.add(&first_day_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    view_sec.add_sizer(&first_day_row, 0, SizerFlag::Expand, 0);

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
    (
        view_choice,
        weekends_cb,
        first_day_choice,
        rem_field,
        day_starts,
        day_ends,
    )
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
fn build_advanced_tab(panel: &Panel, config: &AppConfig) -> (Choice, TextCtrl, CheckBox) {
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

    // -- Link checking
    //
    // Off unless somebody turns it on, and the label says exactly what is sent
    // rather than asking anybody to take "privacy-preserving" on trust. The
    // whole reason this is offerable at all is that the lists come here and
    // the comparison happens here.
    let links_sec = section(panel, "Link checking");

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
    (log_choice, dl_field, links_box)
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
fn build_feedback_tab(panel: &Panel, config: &AppConfig) -> Vec<CheckBox> {
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
    let descriptions = [
        (
            Channel::Speech,
            "&Speak events through the screen reader",
            "Speak events through the screen reader",
        ),
        (
            Channel::Braille,
            "Send events to a &braille display",
            "Send events to a braille display",
        ),
        (
            Channel::Earcon,
            "Play a short &sound for each event",
            "Play a short sound for each event",
        ),
        (
            Channel::Visual,
            "Show events in the s&tatus bar",
            "Show events in the status bar",
        ),
    ];

    let mut boxes = Vec::new();
    for (channel, label, name) in descriptions {
        let cb = CheckBox::builder(panel).with_label(label).build();
        set_accessible_name(&cb, name);
        cb.set_value(settings.is_channel_enabled(channel));
        sec.add(&cb, 0, SizerFlag::All, 4);
        boxes.push(cb);
    }
    sizer.add_sizer(&sec, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let note = StaticText::builder(panel)
        .with_label(
            "Each event has its own tone, so they can be told apart. Sounds are spaced \
             out so a busy mailbox does not run them together.",
        )
        .build();
    sizer.add(&note, 0, SizerFlag::Expand | SizerFlag::All, 8);

    panel.set_sizer(sizer, true);
    boxes
}

// ── Read settings back from widget references ────────────────────────────────

/// Collect current widget values and produce an updated `AppConfig`.
fn read_settings(w: &SettingsWidgets, base: &AppConfig) -> AppConfig {
    let mut cfg = base.clone();

    // Feedback channels. The per-event overrides in the stored value are
    // preserved: this tab only decides which channels are on at all.
    let mut feedback = FeedbackSettings::from_stored(&base.feedback_channels);
    for (channel, cb) in Channel::ALL.into_iter().zip(w.feedback.iter()) {
        feedback.set_channel_enabled(channel, cb.get_value());
    }
    cfg.feedback_channels = feedback.to_stored();

    // What may be changed at a server. Read back as two answers, because they
    // are two: sending cannot be undone, a task can be moved back.
    cfg.allowed_changes = crate::application::allowed::Allowed {
        mail: w.allow_mail.get_value(),
        personal_information: w.allow_pim.get_value(),
    };

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
    cfg.enable_notifications = w.notifications.get_value();
    cfg.check_updates = w.check_updates.get_value();

    // Compose
    cfg.preview_before_send = w.preview_before_send.get_value();
    cfg.draft_autosave_minutes =
        AutosaveInterval::from_setting(w.draft_autosave.value().max(0) as u32).minutes();

    // Reading
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
    cfg.calendar_default_view = match sel(&w.cal_default_view) {
        1 => "day",
        2 => "week",
        3 => "month",
        _ => "agenda",
    }
    .to_string();
    cfg.calendar_show_weekends = w.cal_show_weekends.get_value();
    cfg.calendar_first_day_of_week = match sel(&w.cal_first_day) {
        1 => 1,
        2 => 6,
        _ => 0,
    };
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
    cfg.check_links_with_google = w.check_links_with_google.is_checked();

    cfg
}

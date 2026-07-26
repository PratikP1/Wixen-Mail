//! Calendar agenda-style UI dialog with event editor.
//!
//! Accessibility-first: keyboard-navigable list, accelerator keys on all buttons,
//! screen reader announces status changes.

use wxdragon::prelude::*;

use crate::presentation::ui_types::CalendarEventItem;

// ── Button IDs ──────────────────────────────────────────────────────────────

const ID_CAL_NEW: Id = ID_HIGHEST + 500;
const ID_CAL_EDIT: Id = ID_HIGHEST + 501;
const ID_CAL_DELETE: Id = ID_HIGHEST + 502;
const ID_CAL_SYNC: Id = ID_HIGHEST + 503;

// ── Calendar Action Result ──────────────────────────────────────────────────

/// Actions that the calendar dialog can produce.
#[derive(Debug, Clone)]
pub enum CalendarAction {
    /// No action; dialog was closed.
    None,
    /// User wants to sync calendar events.
    SyncRequested,
    /// User created a new event.
    CreateEvent(CalendarEventData),
    /// User edited an existing event (includes the event ID).
    UpdateEvent(String, CalendarEventData),
    /// User deleted an event (by event ID).
    DeleteEvent(String),
}

/// Data captured from the event editor dialog.
#[derive(Debug, Clone)]
pub struct CalendarEventData {
    pub summary: String,
    pub start_date: String,
    pub start_time: String,
    pub end_date: String,
    pub end_time: String,
    pub is_all_day: bool,
    pub location: String,
    pub description: String,
    pub reminder_minutes: i32,
}

impl CalendarEventData {
    /// Validate the event data, returning a list of error messages.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.summary.trim().is_empty() {
            errors.push("Summary is required".to_string());
        }
        if self.start_date.trim().is_empty() {
            errors.push("Start date is required".to_string());
        }
        if self.end_date.trim().is_empty() {
            errors.push("End date is required".to_string());
        }
        if !self.is_all_day {
            if self.start_time.trim().is_empty() {
                errors.push("Start time is required for timed events".to_string());
            }
            if self.end_time.trim().is_empty() {
                errors.push("End time is required for timed events".to_string());
            }
        }
        // Check end >= start (simple string comparison works for YYYY-MM-DD + HH:MM)
        if !self.start_date.is_empty() && !self.end_date.is_empty() {
            let start = format!("{} {}", self.start_date, self.start_time);
            let end = format!("{} {}", self.end_date, self.end_time);
            if end < start {
                errors.push("End date/time must be after start date/time".to_string());
            }
        }
        if self.reminder_minutes < 0 {
            errors.push("Reminder minutes cannot be negative".to_string());
        }
        errors
    }

    /// Check if the event data is valid.
    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }
}

// ── Calendar Dialog ─────────────────────────────────────────────────────────

/// Show the calendar agenda dialog.
///
/// Returns a list of `CalendarAction`s the user performed.
pub fn show_calendar_dialog(parent: &Frame, events: &[CalendarEventItem]) -> Vec<CalendarAction> {
    let dialog = Dialog::builder(parent, "Calendar")
        .with_size(800, 600)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();

    let main_sizer = BoxSizer::builder(Orientation::Vertical).build();

    // Status text
    let status = StaticText::builder(&dialog)
        .with_label(&format!("{} event(s)", events.len()))
        .build();
    main_sizer.add(&status, 0, SizerFlag::Expand | SizerFlag::All, 4);

    // Event list (Report mode)
    let list = ListCtrl::builder(&dialog)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::HRules)
        .build();
    list.insert_column(0, "Date/Time", ListColumnFormat::Left, 180);
    list.insert_column(1, "Summary", ListColumnFormat::Left, 280);
    list.insert_column(2, "Location", ListColumnFormat::Left, 160);
    list.insert_column(3, "Status", ListColumnFormat::Left, 80);

    populate_event_list(&list, events);

    main_sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // Buttons
    let new_btn = Button::builder(&dialog)
        .with_label("&New Event")
        .with_id(ID_CAL_NEW)
        .build();
    let edit_btn = Button::builder(&dialog)
        .with_label("&Edit Event")
        .with_id(ID_CAL_EDIT)
        .build();
    let del_btn = Button::builder(&dialog)
        .with_label("&Delete Event")
        .with_id(ID_CAL_DELETE)
        .build();
    let sync_btn = Button::builder(&dialog)
        .with_label("S&ync")
        .with_id(ID_CAL_SYNC)
        .build();
    let close_btn = Button::builder(&dialog)
        .with_label("&Close")
        .with_id(ID_OK)
        .build();

    let btn_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    btn_sizer.add(&new_btn, 0, SizerFlag::All, 4);
    btn_sizer.add(&edit_btn, 0, SizerFlag::All, 4);
    btn_sizer.add(&del_btn, 0, SizerFlag::All, 4);
    btn_sizer.add_spacer(16);
    btn_sizer.add(&sync_btn, 0, SizerFlag::All, 4);
    btn_sizer.add_spacer(16);
    btn_sizer.add(&close_btn, 0, SizerFlag::All, 4);

    main_sizer.add_sizer(&btn_sizer, 0, SizerFlag::AlignRight | SizerFlag::All, 4);

    dialog.set_sizer(*main_sizer, true);
    dialog.centre();

    // Button handlers
    new_btn.on_click({
        let d = dialog;
        move |_| {
            d.end_modal(ID_CAL_NEW);
        }
    });
    edit_btn.on_click({
        let d = dialog;
        move |_| {
            d.end_modal(ID_CAL_EDIT);
        }
    });
    del_btn.on_click({
        let d = dialog;
        move |_| {
            d.end_modal(ID_CAL_DELETE);
        }
    });
    sync_btn.on_click({
        let d = dialog;
        move |_| {
            d.end_modal(ID_CAL_SYNC);
        }
    });
    close_btn.on_click({
        let d = dialog;
        move |_| {
            d.end_modal(ID_OK);
        }
    });

    // Modal event loop
    let mut actions = Vec::<CalendarAction>::new();
    let events_data: Vec<CalendarEventItem> = events.to_vec();

    loop {
        match dialog.show_modal() {
            r if r == ID_OK || r == ID_CANCEL => break,
            r if r == ID_CAL_SYNC => {
                actions.push(CalendarAction::SyncRequested);
                status.set_label("Sync requested...");
            }
            r if r == ID_CAL_NEW => {
                if let Some(data) = show_event_editor(&dialog, None) {
                    actions.push(CalendarAction::CreateEvent(data));
                    status.set_label("Event created. Sync to push to provider.");
                }
            }
            r if r == ID_CAL_EDIT => {
                let sel = list.get_first_selected_item();
                if sel >= 0 {
                    let idx = sel as usize;
                    if let Some(item) = events_data.get(idx) {
                        let prefill = CalendarEventData {
                            summary: item.summary.clone(),
                            start_date: item.start.get(..10).unwrap_or("").to_string(),
                            start_time: item.start.get(11..16).unwrap_or("").to_string(),
                            end_date: item.end.get(..10).unwrap_or("").to_string(),
                            end_time: item.end.get(11..16).unwrap_or("").to_string(),
                            is_all_day: item.is_all_day,
                            location: item.location.clone(),
                            description: String::new(),
                            reminder_minutes: 15,
                        };
                        if let Some(data) = show_event_editor(&dialog, Some(&prefill)) {
                            actions.push(CalendarAction::UpdateEvent(item.id.clone(), data));
                            status.set_label("Event updated.");
                        }
                    }
                } else {
                    status.set_label("Select an event to edit.");
                }
            }
            r if r == ID_CAL_DELETE => {
                let sel = list.get_first_selected_item();
                if sel >= 0 {
                    let idx = sel as usize;
                    if let Some(item) = events_data.get(idx) {
                        // Use a simple confirmation dialog
                        let confirm = Dialog::builder(&dialog, "Confirm Delete")
                            .with_size(350, 150)
                            .build();
                        let cs = BoxSizer::builder(Orientation::Vertical).build();
                        let msg = StaticText::builder(&confirm)
                            .with_label(&format!("Delete '{}'?", item.summary))
                            .build();
                        cs.add(&msg, 0, SizerFlag::Expand | SizerFlag::All, 12);
                        let cbs = BoxSizer::builder(Orientation::Horizontal).build();
                        let yes_btn = Button::builder(&confirm)
                            .with_label("&Yes")
                            .with_id(ID_OK)
                            .build();
                        let no_btn = Button::builder(&confirm)
                            .with_label("&No")
                            .with_id(ID_CANCEL)
                            .build();
                        cbs.add(&yes_btn, 0, SizerFlag::All, 4);
                        cbs.add(&no_btn, 0, SizerFlag::All, 4);
                        cs.add_sizer(&cbs, 0, SizerFlag::AlignRight | SizerFlag::All, 8);
                        confirm.set_sizer(*cs, true);
                        confirm.centre();
                        yes_btn.on_click({
                            let d = confirm;
                            move |_| {
                                d.end_modal(ID_OK);
                            }
                        });
                        no_btn.on_click({
                            let d = confirm;
                            move |_| {
                                d.end_modal(ID_CANCEL);
                            }
                        });

                        if confirm.show_modal() == ID_OK {
                            actions.push(CalendarAction::DeleteEvent(item.id.clone()));
                            status.set_label("Event deleted.");
                        }
                        confirm.destroy();
                    }
                } else {
                    status.set_label("Select an event to delete.");
                }
            }
            _ => break,
        }
    }

    dialog.destroy();
    actions
}

fn populate_event_list(list: &ListCtrl, events: &[CalendarEventItem]) {
    list.delete_all_items();
    for (i, event) in events.iter().enumerate() {
        let time_display = if event.is_all_day {
            format!(
                "{} (All day)",
                event.start.get(..10).unwrap_or(&event.start)
            )
        } else {
            event.start.get(..16).unwrap_or(&event.start).to_string()
        };

        let idx = i as i64;
        list.insert_item(idx, &time_display, None);
        list.set_item_text_by_column(idx, 1, &event.summary);
        list.set_item_text_by_column(idx, 2, &event.location);
        list.set_item_text_by_column(idx, 3, &event.status);
    }
}

// ── Event Editor Dialog ─────────────────────────────────────────────────────

/// Show the event editor dialog with optional prefill data.
///
/// Returns `Some(data)` if the user clicked OK, `None` if cancelled.
fn show_event_editor(
    parent: &Dialog,
    prefill: Option<&CalendarEventData>,
) -> Option<CalendarEventData> {
    let title = if prefill.is_some() {
        "Edit Event"
    } else {
        "New Event"
    };
    let editor = Dialog::builder(parent, title)
        .with_size(500, 450)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();

    let sizer = FlexGridSizer::builder(0, 2)
        .with_vgap(4)
        .with_hgap(8)
        .build();
    sizer.add_growable_col(1, 1);

    // Summary
    let lbl_summary = StaticText::builder(&editor).with_label("&Summary:").build();
    let txt_summary = TextCtrl::builder(&editor).build();
    sizer.add(
        &lbl_summary,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    sizer.add(&txt_summary, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // Start Date
    let lbl_start_date = StaticText::builder(&editor)
        .with_label("Start &Date (YYYY-MM-DD):")
        .build();
    let txt_start_date = TextCtrl::builder(&editor).build();
    sizer.add(
        &lbl_start_date,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    sizer.add(&txt_start_date, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // Start Time
    let lbl_start_time = StaticText::builder(&editor)
        .with_label("Start &Time (HH:MM):")
        .build();
    let txt_start_time = TextCtrl::builder(&editor).build();
    sizer.add(
        &lbl_start_time,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    sizer.add(&txt_start_time, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // End Date
    let lbl_end_date = StaticText::builder(&editor)
        .with_label("&End Date (YYYY-MM-DD):")
        .build();
    let txt_end_date = TextCtrl::builder(&editor).build();
    sizer.add(
        &lbl_end_date,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    sizer.add(&txt_end_date, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // End Time
    let lbl_end_time = StaticText::builder(&editor)
        .with_label("End Ti&me (HH:MM):")
        .build();
    let txt_end_time = TextCtrl::builder(&editor).build();
    sizer.add(
        &lbl_end_time,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    sizer.add(&txt_end_time, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // All Day checkbox
    let lbl_allday = StaticText::builder(&editor).with_label("").build();
    let chk_allday = CheckBox::builder(&editor)
        .with_label("All &day event")
        .build();
    sizer.add(&lbl_allday, 0, SizerFlag::All, 4);
    sizer.add(&chk_allday, 0, SizerFlag::All, 4);

    // Location
    let lbl_location = StaticText::builder(&editor)
        .with_label("&Location:")
        .build();
    let txt_location = TextCtrl::builder(&editor).build();
    sizer.add(
        &lbl_location,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    sizer.add(&txt_location, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // Description
    let lbl_desc = StaticText::builder(&editor)
        .with_label("D&escription:")
        .build();
    let txt_desc = TextCtrl::builder(&editor)
        .with_style(TextCtrlStyle::MultiLine)
        .build();
    sizer.add(
        &lbl_desc,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    sizer.add(&txt_desc, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // Reminder
    let lbl_reminder = StaticText::builder(&editor)
        .with_label("&Reminder (minutes):")
        .build();
    let txt_reminder = TextCtrl::builder(&editor).build();
    sizer.add(
        &lbl_reminder,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    sizer.add(&txt_reminder, 1, SizerFlag::Expand | SizerFlag::All, 4);

    // Prefill
    if let Some(data) = prefill {
        txt_summary.set_value(&data.summary);
        txt_start_date.set_value(&data.start_date);
        txt_start_time.set_value(&data.start_time);
        txt_end_date.set_value(&data.end_date);
        txt_end_time.set_value(&data.end_time);
        chk_allday.set_value(data.is_all_day);
        txt_location.set_value(&data.location);
        txt_desc.set_value(&data.description);
        txt_reminder.set_value(&data.reminder_minutes.to_string());
    } else {
        // Defaults for new event
        let now = chrono::Local::now();
        txt_start_date.set_value(&now.format("%Y-%m-%d").to_string());
        txt_start_time.set_value(&now.format("%H:00").to_string());
        let end = now + chrono::Duration::hours(1);
        txt_end_date.set_value(&end.format("%Y-%m-%d").to_string());
        txt_end_time.set_value(&end.format("%H:00").to_string());
        txt_reminder.set_value("15");
    }

    // OK / Cancel buttons
    let btn_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    let ok = Button::builder(&editor)
        .with_label("OK")
        .with_id(ID_OK)
        .build();
    let cancel = Button::builder(&editor)
        .with_label("Cancel")
        .with_id(ID_CANCEL)
        .build();
    btn_sizer.add(&ok, 0, SizerFlag::All, 4);
    btn_sizer.add(&cancel, 0, SizerFlag::All, 4);

    let outer = BoxSizer::builder(Orientation::Vertical).build();
    outer.add_sizer(&sizer, 1, SizerFlag::Expand | SizerFlag::All, 8);
    outer.add_sizer(&btn_sizer, 0, SizerFlag::AlignRight | SizerFlag::All, 8);
    editor.set_sizer(*outer, true);
    editor.centre();

    ok.on_click({
        let d = editor;
        move |_| {
            d.end_modal(ID_OK);
        }
    });
    cancel.on_click({
        let d = editor;
        move |_| {
            d.end_modal(ID_CANCEL);
        }
    });

    if editor.show_modal() == ID_OK {
        let result = CalendarEventData {
            summary: txt_summary.get_value(),
            start_date: txt_start_date.get_value(),
            start_time: txt_start_time.get_value(),
            end_date: txt_end_date.get_value(),
            end_time: txt_end_time.get_value(),
            is_all_day: chk_allday.get_value(),
            location: txt_location.get_value(),
            description: txt_desc.get_value(),
            reminder_minutes: txt_reminder.get_value().parse().unwrap_or(15),
        };
        editor.destroy();
        Some(result)
    } else {
        editor.destroy();
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event_data(
        summary: &str,
        start_date: &str,
        start_time: &str,
        end_date: &str,
        end_time: &str,
        all_day: bool,
    ) -> CalendarEventData {
        CalendarEventData {
            summary: summary.to_string(),
            start_date: start_date.to_string(),
            start_time: start_time.to_string(),
            end_date: end_date.to_string(),
            end_time: end_time.to_string(),
            is_all_day: all_day,
            location: String::new(),
            description: String::new(),
            reminder_minutes: 15,
        }
    }

    #[test]
    fn test_valid_timed_event() {
        let data = make_event_data(
            "Meeting",
            "2026-03-05",
            "09:00",
            "2026-03-05",
            "10:00",
            false,
        );
        assert!(data.is_valid());
        assert!(data.validate().is_empty());
    }

    #[test]
    fn test_valid_all_day_event() {
        let data = make_event_data("Holiday", "2026-03-05", "", "2026-03-05", "", true);
        assert!(data.is_valid());
    }

    #[test]
    fn test_empty_summary_invalid() {
        let data = make_event_data("", "2026-03-05", "09:00", "2026-03-05", "10:00", false);
        assert!(!data.is_valid());
        assert!(data.validate().iter().any(|e| e.contains("Summary")));
    }

    #[test]
    fn test_empty_start_date_invalid() {
        let data = make_event_data("Meeting", "", "09:00", "2026-03-05", "10:00", false);
        assert!(!data.is_valid());
        assert!(data.validate().iter().any(|e| e.contains("Start date")));
    }

    #[test]
    fn test_timed_event_requires_times() {
        let data = make_event_data("Meeting", "2026-03-05", "", "2026-03-05", "", false);
        let errors = data.validate();
        assert!(errors.iter().any(|e| e.contains("Start time")));
        assert!(errors.iter().any(|e| e.contains("End time")));
    }

    #[test]
    fn test_all_day_event_ignores_times() {
        let data = make_event_data("Holiday", "2026-03-05", "", "2026-03-05", "", true);
        assert!(data.is_valid());
    }

    #[test]
    fn test_end_before_start_invalid() {
        let data = make_event_data(
            "Meeting",
            "2026-03-05",
            "10:00",
            "2026-03-05",
            "09:00",
            false,
        );
        assert!(!data.is_valid());
        assert!(data.validate().iter().any(|e| e.contains("after start")));
    }

    #[test]
    fn test_negative_reminder_invalid() {
        let mut data = make_event_data(
            "Meeting",
            "2026-03-05",
            "09:00",
            "2026-03-05",
            "10:00",
            false,
        );
        data.reminder_minutes = -5;
        assert!(!data.is_valid());
        assert!(data.validate().iter().any(|e| e.contains("negative")));
    }

    #[test]
    fn test_multi_day_event_valid() {
        let data = make_event_data(
            "Conference",
            "2026-03-05",
            "09:00",
            "2026-03-07",
            "17:00",
            false,
        );
        assert!(data.is_valid());
    }
}

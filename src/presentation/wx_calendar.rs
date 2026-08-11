//! Calendar agenda-style UI dialog with event editor.
//!
//! Accessibility-first: keyboard-navigable list, accelerator keys on all buttons,
//! screen reader announces status changes.

use wxdragon::prelude::*;

use crate::application::calendar::{EditMeans, WhereAChangeGoes};
use crate::presentation::accessibility::names::set_accessible_name;
use crate::presentation::ui_types::CalendarEventItem;
use crate::presentation::wx_which_days::which_days_are_meant;

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
    /// User edited an existing event: which day was open, which days they
    /// meant, and what the boxes held.
    ///
    /// A repeating event has one stored row behind every day it falls on, so
    /// neither of the first two is decoration. Without the answer, changing the
    /// fortieth Tuesday rewrites all fifty-two. Without the row, nothing on the
    /// far side knows which Tuesday was open, and the save reads the difference
    /// between the day shown and the day the series starts from as a date
    /// somebody typed.
    UpdateEvent(CalendarEventItem, EditMeans, CalendarEventData),
    /// User deleted an event: which day was open, and which days they meant.
    DeleteEvent(CalendarEventItem, EditMeans),
}

/// Data captured from the event editor dialog.
///
/// Compared as a whole to answer "did anybody change anything?", which is why
/// it carries `PartialEq`. The editor is filled from [`Self::as_shown`] and
/// hands back one of these, so the two are equal exactly when nothing was
/// typed.
#[derive(Debug, Clone, PartialEq)]
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
    /// What the editor is filled with for an event already stored.
    ///
    /// Written once rather than at the dialog and again wherever the answer is
    /// compared with what came back. Two copies of these nine assignments
    /// drift, and the moment they do an event nobody touched reads as edited
    /// and the whole record goes back to the provider.
    ///
    /// The row is the list's, not the stored event's, so a repeating event
    /// fills the boxes with the day somebody was standing on rather than the
    /// day the series starts from. That is what the editor shows and therefore
    /// what it writes back, and it is why a repeating event opened on its
    /// fortieth day reads as changed even when nothing was typed.
    pub fn as_shown(item: &CalendarEventItem) -> Self {
        Self {
            summary: item.summary.clone(),
            // Ten characters of date, a separator, then five of time. A stored
            // value with no time in it answers `None` here and leaves the box
            // empty, which is what an all-day event wants.
            start_date: item.start.get(..10).unwrap_or("").to_string(),
            start_time: item.start.get(11..16).unwrap_or("").to_string(),
            end_date: item.end.get(..10).unwrap_or("").to_string(),
            end_time: item.end.get(11..16).unwrap_or("").to_string(),
            is_all_day: item.is_all_day,
            location: item.location.clone(),
            // What the event actually says, rather than a blank and a quarter
            // of an hour. The editor writes back what it shows, so showing the
            // wrong thing here wiped the notes and reset the alert on every
            // edit.
            description: item.description.clone(),
            reminder_minutes: item.reminder_minutes.unwrap_or(0),
        }
    }

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
///
/// `where_changes_go` answers, for one row, which kind of calendar the event is
/// filed in. Handed in rather than worked out here, because answering it needs
/// the stored calendar and this window has none, and because there is one place
/// that question is answered for the whole program.
pub fn show_calendar_dialog(
    parent: &Frame,
    events: &[CalendarEventItem],
    where_changes_go: &dyn Fn(&CalendarEventItem) -> WhereAChangeGoes,
) -> Vec<CalendarAction> {
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
    set_accessible_name(&list, "Calendar events");
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
                        let prefill = CalendarEventData::as_shown(item);
                        // Asked before the editor opens, so somebody who meant
                        // one day is not made to fill a form first and then
                        // told it cannot be done.
                        if let Some(means) = which_days_are_meant(
                            &dialog,
                            &item.summary,
                            &item.repeats,
                            where_changes_go(item),
                        ) && let Some(data) = show_event_editor(&dialog, Some(&prefill))
                        {
                            actions.push(CalendarAction::UpdateEvent(item.clone(), means, data));
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

                        if confirm.show_modal() == ID_OK
                            && let Some(means) = which_days_are_meant(
                                &dialog,
                                &item.summary,
                                &item.repeats,
                                where_changes_go(item),
                            )
                        {
                            actions.push(CalendarAction::DeleteEvent(item.clone(), means));
                            // Said as what really happens. Taking one day off a
                            // series is not a deletion: the event stays and the
                            // other days keep their own values.
                            status.set_label(match means {
                                EditMeans::OneDay => "That one day is taken off.",
                                EditMeans::WholeSeries => "Event deleted.",
                            });
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
    set_accessible_name(&txt_summary, "Summary");
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
    set_accessible_name(&txt_start_date, "Start date");
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
    set_accessible_name(&txt_start_time, "Start time");
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
    set_accessible_name(&txt_end_date, "End date");
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
    set_accessible_name(&txt_end_time, "End time");
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
    set_accessible_name(&txt_location, "Location");
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
    set_accessible_name(&txt_desc, "Description");
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
    set_accessible_name(&txt_reminder, "Reminder in minutes");
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

    // ── The window that asks which days somebody means ──────────────────
    //
    // Read as text, the same way the first-run screen is read and for the same
    // reason: the fact is real and it matters, and a test cannot reach it by
    // running because reaching it needs a window on a screen. It is read from
    // here rather than from inside itself, because a test that names
    // `set_value` in an assertion would be counted as a second place the tick
    // is put on.

    /// The window that asks the question, read as text.
    fn the_window_that_asks() -> String {
        std::fs::read_to_string("src/presentation/wx_which_days.rs")
            .expect("the window that asks which days to be readable")
            .replace("\r\n", "\n")
    }

    #[test]
    fn test_the_window_ticks_the_answer_that_is_preselected() {
        let window = the_window_that_asks();

        assert!(
            window.contains("button.set_value(*means == EditMeans::PRESELECTED)"),
            "the window no longer ticks the answer that is offered first, so \
             nothing here knows which one it opens on"
        );
        assert_eq!(
            window.matches("set_value").count(),
            1,
            "the tick is put on in more than one place, so the answer focus \
             found is not the one left ticked"
        );
    }

    #[test]
    fn test_the_window_puts_focus_on_the_answer_it_ticks() {
        // The rule this window has to keep. It was three push buttons, which
        // cannot be ticked at all, so the question opened with focus on Cancel
        // and the answer that would be taken was nowhere in what a screen
        // reader read out.
        let window = the_window_that_asks();

        assert!(
            window.contains(
                "if let Some((_, ticked)) = buttons.iter().find(|(_, button)| \
                 button.get_value()) {\n        ticked.set_focus();\n    }"
            ),
            "the window no longer puts focus on the answer it ticks"
        );
        assert_eq!(
            window.matches("set_focus").count(),
            1,
            "something else in the window takes focus as well, so what is heard \
             is no longer the answer that is ticked"
        );
        let ticked = window
            .find("button.set_value")
            .expect("the window to tick an answer");
        let focused = window
            .find("ticked.set_focus")
            .expect("the window to focus");
        assert!(
            ticked < focused,
            "focus goes looking for a ticked answer before anything is ticked"
        );
    }

    #[test]
    fn test_the_window_gives_each_answer_a_description_and_puts_it_on_the_screen_too() {
        // A description is what a screen reader working through Microsoft
        // Active Accessibility reads when the button takes focus. The words on
        // screen are the only copy a sighted reader gets, and the only copy a
        // reader working through UI Automation gets. Dropping either takes the
        // sentence away from somebody.
        let window = the_window_that_asks();

        assert!(
            window.contains(
                "set_accessible_name_and_description(&button, &means.spoken(), &will_do)"
            ),
            "what an answer will do is no longer read out when that answer takes focus"
        );
        assert!(
            window.contains("StaticText::builder(&dialog).with_label(&will_do)"),
            "what an answer will do is no longer on the screen"
        );
        assert!(
            window.contains("StaticBoxSizerBuilder::new_with_label"),
            "the answers are no longer a group with a label, so the question \
             itself is loose text somebody has to go looking for"
        );
        assert!(
            window.contains("RadioButtonStyle::GroupStart"),
            "the answers are no longer one set, so the arrow keys leave them"
        );
    }

    #[test]
    fn test_the_window_does_not_let_enter_carry_the_question_out() {
        // Both answers act on somebody's calendar and one of them acts on every
        // day of it. Enter pressed partway through hearing the question must
        // change nothing.
        let window = the_window_that_asks();

        assert!(
            window.contains("cancel.set_default();"),
            "Enter no longer answers the question with the one answer that \
             touches nobody's calendar"
        );
        assert_eq!(
            window.matches("set_default").count(),
            1,
            "something else in the window is the default button as well"
        );
    }

    #[test]
    fn test_the_window_reads_its_answer_back_from_the_buttons() {
        // What is ticked and what happens have to be one answer rather than
        // two, and only Continue is an answer at all.
        let window = the_window_that_asks();

        assert!(
            window.contains("let chosen = (answered == ID_OK).then(|| {"),
            "the window answers with something other than what Continue took"
        );
        assert!(
            window.contains(".find(|(_, button)| button.get_value())"),
            "the answer no longer comes off the buttons"
        );
    }

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

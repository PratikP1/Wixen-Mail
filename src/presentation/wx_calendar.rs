//! Calendar agenda-style UI dialog with event editor.
//!
//! Keyboard-navigable list, accelerator keys on all buttons, and a line of
//! status text under the title.
//!
//! Every sentence this window gives back is shown on that status line and
//! said out loud, from one call. Each of them answers a key somebody has just
//! pressed, and a line of text under a title is not somewhere anybody
//! navigating by ear goes, so a sentence that was only shown was an answer
//! nobody got. Refusals are said above the ordinary run of status, because a
//! refusal is the one answer nobody can be left to miss.
//!
//! Nothing sets a fixed accessible name on that line. Windows gives a piece of
//! static text its own label as its name, so naming it would replace the
//! sentence with a fixed string and take the sentence away.

use wxdragon::prelude::*;

use crate::application::calendar::{
    EditMeans, WhatIsBeingDone, WhatTheCalendarAllows, WrittenDown, can_be_honoured,
    what_is_waiting,
};
use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::announcements::Priority;
use crate::presentation::accessibility::names::set_accessible_name;
use crate::presentation::status_line::said_and_shown;
use crate::presentation::theme;
use crate::presentation::ui_types::CalendarEventItem;
use crate::presentation::wx_which_days::which_days_are_meant;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

// ── Button IDs ──────────────────────────────────────────────────────────────

const ID_CAL_NEW: Id = ID_HIGHEST + 500;
const ID_CAL_EDIT: Id = ID_HIGHEST + 501;
const ID_CAL_DELETE: Id = ID_HIGHEST + 502;
const ID_CAL_SYNC: Id = ID_HIGHEST + 503;

// ── Calendar Action Result ──────────────────────────────────────────────────

/// Actions that the calendar dialog can produce.
#[derive(Debug, Clone)]
pub enum CalendarAction {
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

/// The Calendar list window's own controls, returned so a test can build it
/// without a human closing a live modal.
///
/// `dialog`, `list` and `status` are what `show_calendar_dialog`'s own loop
/// still needs after construction; New and Close are wired to `end_modal`
/// entirely inside [`build_calendar_dialog`] and are never referred to
/// again. `sync`, `edit` and `delete` are handed to
/// [`wire_calendar_actions`] instead, which wires each straight to the
/// function that does its work rather than to `end_modal`; see that
/// function's own doc comment for why.
pub struct CalendarDialogHandles {
    pub dialog: Dialog,
    pub list: ListCtrl,
    pub status: StaticText,
    sync: Button,
    edit: Button,
    delete: Button,
}

/// Build the Calendar list window without showing it.
///
/// Everything `show_calendar_dialog` used to do up to its own modal loop,
/// split out the same way [`crate::presentation::wx_settings::build_settings_dialog`]
/// splits Settings: a test can build the real dialog and read back the real
/// colour a live control holds, and never call `.show_modal()` at all.
pub fn build_calendar_dialog(
    parent: &Frame,
    events: &[CalendarEventItem],
    palette: Option<theme::Palette>,
) -> CalendarDialogHandles {
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

    // Button handlers. Edit, Delete and Sync are deliberately wired to
    // nothing here: each used to end this modal with its own ID, the same
    // as New and Close still do below, but [`wire_calendar_actions`] wires
    // them instead, once this dialog's handles exist, straight to the
    // function that does the work.
    new_btn.on_click({
        let d = dialog;
        move |_| {
            d.end_modal(ID_CAL_NEW);
        }
    });
    close_btn.on_click({
        let d = dialog;
        move |_| {
            d.end_modal(ID_OK);
        }
    });

    // Painted last, after the list's columns are inserted and its first
    // population has run: nothing in this codebase proves whether a native
    // list-view control keeps a manually set background colour across
    // `InsertColumn`, so the buttons and list are fully built and populated
    // before either of these calls, never before (the same caution the
    // Account Manager's own list takes). `None` means high contrast is on,
    // or the system is set up in a way this application should not paint
    // over, so nothing is set here and Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dialog, palette.main_surface());
        theme::paint(&list, palette.main_surface());
    }

    CalendarDialogHandles {
        dialog,
        list,
        status,
        sync: sync_btn,
        edit: edit_btn,
        delete: del_btn,
    }
}

/// What Sync, Edit and Delete can change, shared with the functions below so
/// each can be called on its own rather than only from inside
/// [`run_calendar_loop`].
///
/// `allows` sits beside `events_data` rather than being asked of
/// `where_changes_go` again from inside one of those functions:
/// [`wire_calendar_actions`] wires each straight to a button's own
/// `on_click`, which has to outlive `show_calendar_dialog`'s stack frame
/// since wxdragon's `on_click` requires `'static`, and a borrowed
/// `where_changes_go` does not. Answering the question once, here, and
/// carrying the answer alongside the events it is about reads no fresher
/// and no staler than `events_data` itself already does: both are a
/// snapshot taken once, when the dialog opens, never read live again.
///
/// `pub`, and every field with it, for the same reason
/// [`CalendarDialogHandles`] is: so a test can build one directly and call
/// [`request_sync`], [`edit_selected_event`] or [`delete_selected_event`]
/// against it, which is the only way to prove what one of them does without
/// a human clicking a real button inside a real modal dialog.
pub struct CalendarDialogState {
    pub actions: Vec<CalendarAction>,
    pub events_data: Vec<CalendarEventItem>,
    pub allows: Vec<WhatTheCalendarAllows>,
}

/// Wire Sync, Edit and Delete straight to the function that does their
/// work, rather than to `end_modal`.
///
/// Every button in this dialog used to end the modal with its own ID, the
/// same as New and Close still do inside [`build_calendar_dialog`]:
/// `on_click` called `end_modal`, which hides the dialog and returns
/// control to Rust, and only then did [`run_calendar_loop`]'s `match
/// dialog.show_modal()` see the ID and run the button's own work, including
/// its `said_and_shown`. A live NVDA run against the Account Manager's own
/// Sign In Again, the same shape as these three, found neither of its two
/// sentences is heard: NVDA jumps straight from the button's name to its
/// own generic "Wixen Mail, unavailable", because nothing is yielded to the
/// Windows message pump between `EndModal` hiding the dialog and
/// `show_modal` being called again to re-show it, and the announcement runs
/// inside that gap.
///
/// New is the one exception: it always shows its own nested event editor
/// before it can answer anything, so [`run_calendar_loop`] still answers
/// it, the same reason Account Manager's Add and Edit stay wired to
/// `end_modal` too. Delete looks similar at first glance: it always shows
/// Confirm Delete before it can act, but its "nothing is selected" answer
/// runs before that dialog is ever built, and [`edit_selected_event`]'s own
/// refusal for a day that does not repeat runs before `which_days_are_meant`
/// ever shows one either. Both are exactly the shape this bug needs:
/// `end_modal` hides the dialog, the answer runs with nothing shown,
/// `show_modal` re-shows it. So Edit and Delete are wired here regardless of
/// what either can also do once a nested dialog does open.
fn wire_calendar_actions(
    widgets: &CalendarDialogHandles,
    state: &Rc<RefCell<CalendarDialogState>>,
    a11y: &Arc<Accessibility>,
    palette: Option<theme::Palette>,
) {
    let dialog = widgets.dialog;
    let list = widgets.list;
    let status = widgets.status;

    widgets.sync.on_click({
        let state = Rc::clone(state);
        let a11y = Arc::clone(a11y);
        move |_| {
            request_sync(&mut state.borrow_mut(), &status, &a11y);
        }
    });
    widgets.edit.on_click({
        let state = Rc::clone(state);
        let a11y = Arc::clone(a11y);
        move |_| {
            edit_selected_event(
                &mut state.borrow_mut(),
                &dialog,
                &list,
                &status,
                &a11y,
                palette,
            );
        }
    });
    widgets.delete.on_click({
        let state = Rc::clone(state);
        let a11y = Arc::clone(a11y);
        move |_| {
            delete_selected_event(
                &mut state.borrow_mut(),
                &dialog,
                &list,
                &status,
                &a11y,
                palette,
            );
        }
    });
}

/// The row selected in the calendar list, and what its calendar allows,
/// read back from the state both are stored in.
///
/// `None` covers both "nothing is selected" and an index that does not
/// match what was stored, so a caller only has one thing to check either
/// way, and both existing callers already say the same sentence for the
/// first of those.
fn selected_event(
    state: &CalendarDialogState,
    list: &ListCtrl,
) -> Option<(CalendarEventItem, WhatTheCalendarAllows)> {
    let sel = list.get_first_selected_item();
    if sel < 0 {
        return None;
    }
    let idx = sel as usize;
    match (state.events_data.get(idx), state.allows.get(idx)) {
        (Some(item), Some(allows)) => Some((item.clone(), allows.clone())),
        _ => None,
    }
}

/// What the Sync button answers.
///
/// No row to select, so nothing here can go unanswered the way Edit and
/// Delete can when nothing is chosen first.
///
/// Extracted verbatim from what used to be `show_calendar_dialog`'s own
/// `ID_CAL_SYNC` arm: same state mutation, same `said_and_shown` call.
/// [`wire_calendar_actions`] wires this straight to the S&ync button's
/// `on_click`; see that function's own doc comment for why. `pub` so a test
/// can call it directly, which is the only way to prove what it does
/// without a human clicking a real button inside a real modal dialog.
pub fn request_sync(state: &mut CalendarDialogState, status: &StaticText, a11y: &Accessibility) {
    state.actions.push(CalendarAction::SyncRequested);
    said_and_shown(status, a11y, "Sync requested...", Priority::Normal);
}

/// What the Edit Event button answers.
///
/// Asks which days are meant, and whether the calendar will allow it,
/// before the editor opens: filling in a form and being told afterwards
/// that it cannot be done is the one thing the comment on
/// [`CalendarAction::UpdateEvent`] has always promised this does not do.
///
/// Extracted verbatim from what used to be `show_calendar_dialog`'s own
/// `ID_CAL_EDIT` arm, for the same reason and the same way as
/// [`request_sync`]; wired to the &Edit Event button's `on_click` in
/// [`wire_calendar_actions`].
pub fn edit_selected_event(
    state: &mut CalendarDialogState,
    dialog: &Dialog,
    list: &ListCtrl,
    status: &StaticText,
    a11y: &Accessibility,
    palette: Option<theme::Palette>,
) {
    let Some((item, allows)) = selected_event(state, list) else {
        said_and_shown(status, a11y, "Select an event to edit.", Priority::High);
        return;
    };
    let prefill = CalendarEventData::as_shown(&item);
    // Both asked before the editor opens, so somebody who meant one day is
    // not made to fill a form in first and then told it cannot be done. The
    // refusal is the same sentence the manager would give afterwards, from
    // the same place, because two spellings of that rule is how this window
    // comes to offer what the manager refuses.
    if let Some(means) = which_days_are_meant(
        dialog,
        &item.summary,
        &item.repeats,
        WhatIsBeingDone::Changing,
        &allows,
        palette,
    ) {
        if let Err(refused) = can_be_honoured(WhatIsBeingDone::Changing, means, &allows) {
            said_and_shown(status, a11y, &refused, Priority::High);
        } else if let Some(data) = show_event_editor(dialog, Some(&prefill), palette) {
            state
                .actions
                .push(CalendarAction::UpdateEvent(item, means, data));
            said_and_shown(
                status,
                a11y,
                what_is_waiting(match means {
                    EditMeans::OneDay => WrittenDown::OneDayChanged,
                    EditMeans::WholeSeries => WrittenDown::WholeSeriesChanged,
                }),
                Priority::Normal,
            );
        }
    }
}

/// What the Delete Event button answers.
///
/// Confirms, then asks which days are meant and whether the calendar will
/// allow it, before anything is queued: the same order as
/// [`edit_selected_event`], and for the same reason.
///
/// Extracted verbatim from what used to be `show_calendar_dialog`'s own
/// `ID_CAL_DELETE` arm, for the same reason and the same way as
/// [`request_sync`]; wired to the &Delete Event button's `on_click` in
/// [`wire_calendar_actions`].
pub fn delete_selected_event(
    state: &mut CalendarDialogState,
    dialog: &Dialog,
    list: &ListCtrl,
    status: &StaticText,
    a11y: &Accessibility,
    palette: Option<theme::Palette>,
) {
    let Some((item, allows)) = selected_event(state, list) else {
        said_and_shown(status, a11y, "Select an event to delete.", Priority::High);
        return;
    };
    let (confirm, _yes_btn, _no_btn) = build_confirm_delete_dialog(dialog, &item.summary, palette);

    // Asked here, where the person is standing, rather than after this
    // window closes. A calendar that will refuse the answer refuses it now,
    // instead of saying the day is off and then saying nothing was changed
    // a moment later.
    if confirm.show_modal() == ID_OK
        && let Some(means) = which_days_are_meant(
            dialog,
            &item.summary,
            &item.repeats,
            WhatIsBeingDone::Deleting,
            &allows,
            palette,
        )
    {
        if let Err(refused) = can_be_honoured(WhatIsBeingDone::Deleting, means, &allows) {
            said_and_shown(status, a11y, &refused, Priority::High);
        } else {
            state.actions.push(CalendarAction::DeleteEvent(item, means));
            // Said as what is waiting to happen. Nothing on this list has
            // happened yet, and taking one day off a series is not a
            // deletion either: the event stays and the other days keep
            // their own values.
            said_and_shown(
                status,
                a11y,
                what_is_waiting(match means {
                    EditMeans::OneDay => WrittenDown::OneDayTakenOff,
                    EditMeans::WholeSeries => WrittenDown::WholeSeriesDeleted,
                }),
                Priority::Normal,
            );
        }
    }
    confirm.destroy();
}

/// The New/Close modal loop `show_calendar_dialog` runs against the dialog
/// [`build_calendar_dialog`] built.
///
/// Sync, Edit and Delete used to have arms here too.
/// [`wire_calendar_actions`] answers their `on_click` directly now, so
/// `show_modal()` never returns one of their IDs and an arm for one here
/// could never be reached; see that function's own doc comment for why.
/// New stays because it has to leave this dialog to show its own nested
/// event editor; the terminal case stays as the loop's own way out.
fn run_calendar_loop(
    widgets: &CalendarDialogHandles,
    state: &Rc<RefCell<CalendarDialogState>>,
    a11y: &Arc<Accessibility>,
    palette: Option<theme::Palette>,
) {
    let dialog = &widgets.dialog;
    let status = &widgets.status;
    loop {
        match dialog.show_modal() {
            r if r == ID_OK || r == ID_CANCEL => break,
            r if r == ID_CAL_NEW => {
                if let Some(data) = show_event_editor(dialog, None, palette) {
                    state
                        .borrow_mut()
                        .actions
                        .push(CalendarAction::CreateEvent(data));
                    said_and_shown(
                        status,
                        a11y,
                        what_is_waiting(WrittenDown::Created),
                        Priority::Normal,
                    );
                }
            }
            _ => break,
        }
    }
}

/// Show the calendar agenda dialog.
///
/// Returns a list of `CalendarAction`s the user performed.
///
/// `where_changes_go` answers, for one row, what the calendar the event is
/// filed in allows: which kind of calendar it is, and whether the day could be
/// kept as an appointment of its own there. Handed in rather than worked out
/// here, because answering it needs the stored calendar and this window has
/// none, and because there is one place that question is answered for the whole
/// program.
pub fn show_calendar_dialog(
    parent: &Frame,
    events: &[CalendarEventItem],
    where_changes_go: &dyn Fn(&CalendarEventItem) -> WhatTheCalendarAllows,
    a11y: &Arc<Accessibility>,
) -> Vec<CalendarAction> {
    let palette = theme::current_from_stored_config();
    let widgets = build_calendar_dialog(parent, events, palette);

    let state = Rc::new(RefCell::new(CalendarDialogState {
        actions: Vec::new(),
        events_data: events.to_vec(),
        allows: events.iter().map(where_changes_go).collect(),
    }));

    wire_calendar_actions(&widgets, &state, a11y, palette);
    run_calendar_loop(&widgets, &state, a11y, palette);

    widgets.dialog.destroy();
    state.borrow().actions.clone()
}

/// Build the Confirm Delete dialog `show_calendar_dialog` opens before
/// deleting one event, without showing it.
///
/// Everything the delete branch used to build up to its own `.show_modal()`
/// call, split out the same way [`crate::presentation::wx_settings::build_settings_dialog`]
/// splits Settings: a test can build the real dialog and read back the real
/// colour a live control holds, and never call `.show_modal()` at all.
///
/// Returns the dialog alongside the two buttons the caller still needs after
/// a real `.show_modal()`, though both only ever end the dialog with the id
/// already wired here.
pub fn build_confirm_delete_dialog(
    parent: &Dialog,
    summary: &str,
    palette: Option<theme::Palette>,
) -> (Dialog, Button, Button) {
    let confirm = Dialog::builder(parent, "Confirm Delete")
        .with_size(350, 150)
        .build();
    let cs = BoxSizer::builder(Orientation::Vertical).build();
    let msg = StaticText::builder(&confirm)
        .with_label(&format!("Delete '{summary}'?"))
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

    // Painted last. No `TextCtrl`, `ListCtrl` or `TreeCtrl` anywhere in this
    // dialog, so the dialog itself is the only site, the same shape as
    // `build_which_days_dialog`. `None` means high contrast is on, or the
    // system is set up in a way this application should not paint over, so
    // nothing is set here and Windows decides.
    if let Some(palette) = palette {
        theme::paint(&confirm, palette.main_surface());
    }

    (confirm, yes_btn, no_btn)
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
/// The New/Edit Event dialog's fields, returned so a test can build it
/// without a human closing a live modal and so `show_event_editor` can read
/// every field back after a real `.show_modal()`.
pub struct EventEditorWidgets {
    pub dialog: Dialog,
    pub txt_summary: TextCtrl,
    pub txt_start_date: TextCtrl,
    pub txt_start_time: TextCtrl,
    pub txt_end_date: TextCtrl,
    pub txt_end_time: TextCtrl,
    pub chk_allday: CheckBox,
    pub txt_location: TextCtrl,
    pub txt_desc: TextCtrl,
    pub txt_reminder: TextCtrl,
}

fn show_event_editor(
    parent: &Dialog,
    prefill: Option<&CalendarEventData>,
    palette: Option<theme::Palette>,
) -> Option<CalendarEventData> {
    let w = build_event_editor_dialog(parent, prefill, palette);
    if w.dialog.show_modal() == ID_OK {
        let result = CalendarEventData {
            summary: w.txt_summary.get_value(),
            start_date: w.txt_start_date.get_value(),
            start_time: w.txt_start_time.get_value(),
            end_date: w.txt_end_date.get_value(),
            end_time: w.txt_end_time.get_value(),
            is_all_day: w.chk_allday.get_value(),
            location: w.txt_location.get_value(),
            description: w.txt_desc.get_value(),
            reminder_minutes: w.txt_reminder.get_value().parse().unwrap_or(15),
        };
        w.dialog.destroy();
        Some(result)
    } else {
        w.dialog.destroy();
        None
    }
}

/// Build the New/Edit Event dialog without showing it.
///
/// Everything `show_event_editor` used to do up to its own `.show_modal()`
/// call, split out the same way [`crate::presentation::wx_settings::build_settings_dialog`]
/// splits Settings: a test can build the real dialog and read back the real
/// colour a live control holds, and never call `.show_modal()` at all.
pub fn build_event_editor_dialog(
    parent: &Dialog,
    prefill: Option<&CalendarEventData>,
    palette: Option<theme::Palette>,
) -> EventEditorWidgets {
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

    // Painted last, once every field is built. `None` means high contrast is
    // on, or the system is set up in a way this application should not paint
    // over, so nothing is set here and Windows decides.
    if let Some(palette) = palette {
        theme::paint(&editor, palette.main_surface());
        for field in [
            &txt_summary,
            &txt_start_date,
            &txt_start_time,
            &txt_end_date,
            &txt_end_time,
            &txt_location,
            &txt_desc,
            &txt_reminder,
        ] {
            theme::paint(field, palette.main_surface());
        }
    }

    EventEditorWidgets {
        dialog: editor,
        txt_summary,
        txt_start_date,
        txt_start_time,
        txt_end_date,
        txt_end_time,
        chk_allday,
        txt_location,
        txt_desc,
        txt_reminder,
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

    // ── The calendar window's own sentences ─────────────────────────────
    //
    // Read as text for the same reason: reaching these arms needs a window, a
    // list with a row selected, and somebody clicking through two dialogs.
    // What can be read from here is the order things happen in, and the order
    // is the whole fault: the window announced what it had written down at the
    // moment it wrote it down, so on a Google calendar, an Outlook calendar or
    // one this program can only read, "That one day is taken off" was followed
    // a moment later by a refusal saying nothing had been changed.

    /// This window, read as text, without the tests underneath it.
    ///
    /// The tests are cut off because they quote the sentences they are looking
    /// for. Read whole, the file always holds every one of them and the check
    /// can never pass.
    fn the_calendar_window() -> String {
        let whole = std::fs::read_to_string("src/presentation/wx_calendar.rs")
            .expect("the calendar window to be readable")
            .replace("\r\n", "\n");
        whole
            .split_once("\n#[cfg(test)]")
            .map_or(whole.clone(), |(window, _)| window.to_string())
    }

    /// One function's own body, from its signature to the line that starts
    /// the next top-level item, so a check reading it cannot match a
    /// similar line that belongs to a different function.
    ///
    /// Edit and Delete's own logic used to live in the loop below as two of
    /// its match arms, which is what this helper's own predecessor,
    /// `the_arm_for`, once cut out of the window's source. Both now answer
    /// their own button's `on_click` directly instead of waiting for a pass
    /// through that loop (see [`super::wire_calendar_actions`]), so their
    /// logic lives in named functions now, and this is that same idea
    /// pointed at a function's signature rather than a match arm's label.
    /// The same idea as `tests/theme_reach.rs`'s own `function_body`,
    /// generalised to a name that can start with `fn ` or `pub fn `.
    fn the_function_for(source: &str, signature: &str) -> String {
        let start = source
            .find(signature)
            .unwrap_or_else(|| panic!("no function found for {signature}"));
        let after = &source[start..];
        let end = ["\nfn ", "\npub fn "]
            .iter()
            .filter_map(|next| after[1..].find(next))
            .min()
            .map(|at| at + 1)
            .unwrap_or(after.len());
        after[..end].to_string()
    }

    /// A function's own body, with the "nothing is selected" guard clause
    /// at its top cut away.
    ///
    /// Edit and Delete both open the same way: read the row and its
    /// allowance back, or say "Select an event to ..." and return with
    /// nothing asked yet. That early return is a separate rule from the one
    /// `what_the_arm_gets_wrong` checks below, and leaving it in would have
    /// the guard's own `said_and_shown` read as something that happens
    /// before `can_be_honoured` is ever reached, which is not what this is
    /// checking at all.
    fn after_the_guard(function: &str) -> &str {
        let marker = "let Some((item, allows)) = selected_event(state, list) else {";
        let from = function
            .find(marker)
            .unwrap_or_else(|| panic!("the guard clause this expects is not here: {function}"));
        let after = &function[from..];
        let end = after
            .find("};\n")
            .unwrap_or_else(|| panic!("the guard clause's own close was not found"));
        &after[end + 3..]
    }

    /// What one arm gets wrong about asking, and about acting on the answer.
    ///
    /// Two rules, and asking used to be the only one. Asking early is not the
    /// behaviour: taking the other branch when the answer is no is. An arm that
    /// asks, reads the refusal out and then queues the change anyway keeps
    /// every order this used to check, and it is the shape that has come up
    /// three times in this project now. So the refusal and the thing it refuses
    /// have to sit on opposite branches.
    fn what_the_arm_gets_wrong(arm: &str, before: &[&str]) -> Vec<String> {
        let Some(asked) = arm.find("can_be_honoured(") else {
            return vec![
                "the arm never asks whether the answer can be carried out, so the \
                 window offers what the calendar will refuse"
                    .to_string(),
            ];
        };
        let mut wrong = Vec::new();
        for later in before {
            let Some(acted) = arm.find(later) else {
                continue;
            };
            if acted < asked {
                wrong.push(format!("{later} happens before the answer is asked about"));
                continue;
            }
            let Some(refused) = arm.find("&refused") else {
                wrong.push(
                    "the arm asks and never reads the refusal out, so a calendar that \
                     will not take the change says nothing at all"
                        .to_string(),
                );
                continue;
            };
            // Measured from the refusal onwards, because the refusal is itself
            // one of these calls: the sentence is said with the same routine
            // that says what is waiting, so the first match is the refusal and
            // the one that matters is the next.
            let Some(acted) = arm[refused..].find(later).map(|at| refused + at) else {
                continue;
            };
            if !arm[refused..acted].contains("else") {
                wrong.push(format!(
                    "{later} runs whether the answer was refused or not, so the \
                     refusal is read out and the calendar is changed anyway"
                ));
            }
        }
        wrong
    }

    /// What this window says without saying it out loud.
    ///
    /// A line of text under the title is not somewhere anybody navigating by
    /// ear goes, and nothing raises a notification when it changes. Every
    /// sentence this window produces is an answer to a key somebody has just
    /// pressed, so a sentence that is only shown is an answer nobody gets.
    ///
    /// The one call that shows and says now lives in its own module, shared
    /// with the account manager and the manager windows, so its body is read
    /// from there rather than from this file.
    fn what_the_window_never_says(window: &str, the_one_call: &str) -> Vec<String> {
        let mut wrong = Vec::new();
        let Some((_, helper)) = the_one_call.split_once("pub(crate) fn said_and_shown(") else {
            return vec![
                "nothing this window can call both shows a sentence and says it, so every \
                 answer it gives is silent"
                    .to_string(),
            ];
        };
        let body = &helper[..helper.find("\n}").unwrap_or(helper.len())];
        if !body.contains("a11y.announce(") {
            wrong.push("the one place that shows a sentence never says it out loud".to_string());
        }
        let shown = window.matches("status.set_label(").count();
        if shown != 0 {
            wrong.push(format!(
                "{shown} places in this window put a sentence on the status line by \
                 themselves, rather than through the one call that says it as well"
            ));
        }
        if !window.contains("said_and_shown(") {
            wrong
                .push("this window never reaches the one call that says what it shows".to_string());
        }
        wrong
    }

    /// The one call that shows and says, with its own tests cut off.
    fn the_one_call_that_says() -> String {
        let whole = std::fs::read_to_string("src/presentation/status_line.rs")
            .expect("the shared status line to be readable")
            .replace("\r\n", "\n");
        match whole.split_once("\n#[cfg(test)]") {
            Some((code, _)) => code.to_string(),
            None => whole,
        }
    }

    #[test]
    fn test_every_answer_the_calendar_window_gives_is_said_out_loud() {
        // The regression this round is about. The window answered "just this
        // one day" on a Google, an Outlook or a read-only calendar by putting
        // the refusal on a line of text and queueing nothing, so a refusal that
        // used to be read out at once became silence.
        //
        // What this cannot see: whether the announcement reaches a screen
        // reader, and whether the sentence handed to it is a true one. It asks
        // that one routine both shows and says, and that nothing else in the
        // window puts a sentence on the line by itself.
        let source = the_calendar_window();
        let wrong = what_the_window_never_says(&source, &the_one_call_that_says());
        assert!(wrong.is_empty(), "{}", wrong.join("\n  "));
    }

    #[test]
    fn test_the_saying_check_can_tell_the_two_apart() {
        // Proving the measurement. A source read that finds nothing passes, and
        // from outside that is indistinguishable from one that finds
        // everything.
        let call = "pub(crate) fn said_and_shown(\n\
            \x20   line: &StaticText,\n\
            ) {\n\
            \x20   line.set_label(said);\n\
            \x20   let _ = a11y.announce(said, priority);\n\
            }\n";
        let sound = "                said_and_shown(&status, a11y, &refused, Priority::High);\n";
        assert!(
            what_the_window_never_says(sound, call).is_empty(),
            "a window that says everything was reported as silent"
        );

        let one_left_silent = format!("{sound}                status.set_label(\"x\");\n");
        let wrong = what_the_window_never_says(&one_left_silent, call);
        assert_eq!(wrong.len(), 1, "{wrong:?}");
        assert!(wrong[0].contains("status line"), "{wrong:?}");

        let never_says = call.replace("let _ = a11y.announce(said, priority);", "let _ = said;");
        assert!(
            what_the_window_never_says(sound, &never_says)[0].contains("never says it out loud"),
            "a call that only shows was not reported"
        );

        let no_call = "                status.set_label(\"x\");\n";
        assert!(
            what_the_window_never_says(sound, no_call)[0]
                .contains("every answer it gives is silent"),
            "a window with nothing to call was not reported"
        );

        let never_calls = "                status.set_label(\"x\");\n";
        let wrong = what_the_window_never_says(never_calls, call);
        assert!(
            wrong.iter().any(|said| said.contains("never reaches")),
            "a window that never calls it was not reported: {wrong:?}"
        );

        // And the shared call really was read.
        assert!(
            the_one_call_that_says().contains("pub(crate) fn said_and_shown("),
            "the shared call was not read, so this check is measuring nothing"
        );

        // And the tests underneath are really cut off, because they quote every
        // sentence the check looks for.
        let window = the_calendar_window();
        assert!(
            window.len() > 5000,
            "only {} characters of the window were read, so the reading is broken",
            window.len()
        );
        assert!(
            !window.contains("fn the_calendar_window("),
            "the tests were not cut off, so the check is reading its own words"
        );
    }

    #[test]
    fn test_the_calendar_window_asks_before_it_says_anything() {
        // What this cannot see. Reaching either function's own "something is
        // selected" branch takes a window on the screen, a list with a row
        // picked out and somebody clicking through the dialogs each can open,
        // so nothing here runs a line of the code it is about. It reads the
        // functions as text.
        //
        // That means either function could stop being reachable, the button's
        // own `on_click` could lose its call to it, the whole branch could go
        // dead, and this stays green. It cannot say whether the refusal is a
        // true one, whether the sentence is the right sentence for what was
        // refused, or whether an announcement raised from inside a modal
        // dialog is heard at all. Only a screen reader run answers the last of
        // those.
        //
        // What it does hold is the order: that the question is asked before
        // anything is queued, and that the refusal and the thing it refuses sit
        // on opposite branches. That is the shape this project has got wrong
        // three times.
        let source = the_calendar_window();

        let delete = the_function_for(&source, "pub fn delete_selected_event(");
        let edit = the_function_for(&source, "pub fn edit_selected_event(");
        assert!(
            delete.len() > 200 && edit.len() > 200,
            "the functions read as {} and {} characters, so the reading is broken",
            delete.len(),
            edit.len()
        );

        let mut wrong = what_the_arm_gets_wrong(
            after_the_guard(&delete),
            &["actions.push(", "said_and_shown("],
        );
        // The editor is a form somebody fills in. Asking after it opens means
        // filling the whole thing in and then being told it cannot be done,
        // which is what the comment on this function has always promised it
        // does not do.
        wrong.extend(what_the_arm_gets_wrong(
            after_the_guard(&edit),
            &["show_event_editor("],
        ));
        for said in [
            "That one day is taken off.",
            "Event deleted.",
            "Event updated.",
            "Event created.",
        ] {
            if source.contains(said) {
                wrong.push(format!(
                    "{said} is still said here, in the past tense, about something \
                     that has not happened yet"
                ));
            }
        }
        if !source.contains("what_is_waiting(") {
            wrong.push(
                "the window words what it is waiting to do itself, so its words and \
                 the ones said afterwards can drift apart"
                    .to_string(),
            );
        }
        assert!(wrong.is_empty(), "{}", wrong.join("\n  "));
    }

    #[test]
    fn test_the_calendar_window_check_can_tell_the_two_apart() {
        // Proving the measurement. A source read that finds nothing passes,
        // and from outside that is indistinguishable from one that finds
        // everything.
        let sound = "                let allows = where_changes_go(item);\n\
            \x20               if let Err(refused) = can_be_honoured(done, means, &allows) {\n\
            \x20                   said_and_shown(&status, a11y, &refused, Priority::High);\n\
            \x20               } else {\n\
            \x20                   actions.push(CalendarAction::DeleteEvent(item.clone(), means));\n\
            \x20               }\n";
        assert!(
            what_the_arm_gets_wrong(sound, &["actions.push(", "said_and_shown("]).is_empty(),
            "a sound arm was reported as broken"
        );

        let said_first = "                said_and_shown(&status, a11y, &off, Priority::High);\n\
            \x20               if let Err(refused) = can_be_honoured(done, means, &allows) {}\n";
        let wrong = what_the_arm_gets_wrong(said_first, &["actions.push(", "said_and_shown("]);
        assert_eq!(wrong.len(), 1, "{wrong:?}");
        assert!(wrong[0].contains("said_and_shown("), "{wrong:?}");

        let never_asks =
            "                actions.push(CalendarAction::DeleteEvent(item, means));\n";
        assert!(
            what_the_arm_gets_wrong(never_asks, &["actions.push("])[0].contains("never asks"),
            "an arm that never asks was not reported"
        );

        // Asking and then doing it anyway, which is the shape the order rule
        // above cannot see: everything still happens in the right order and the
        // refusal is read out, and the change is queued all the same.
        let asks_and_does_it_anyway =
            sound.replace("                } else {\n", "                }\n");
        let wrong = what_the_arm_gets_wrong(
            &asks_and_does_it_anyway,
            &["actions.push(", "said_and_shown("],
        );
        assert!(
            wrong.iter().any(|said| said.contains("refused or not")),
            "an arm that reads the refusal out and queues the change anyway was \
             reported as sound: {wrong:?}"
        );

        // And the tests underneath are really cut off, because they quote every
        // sentence the check looks for.
        let window = the_calendar_window();
        assert!(
            window.len() > 5000,
            "only {} characters of the window were read, so the reading is broken",
            window.len()
        );
        assert!(
            !window.contains("fn the_calendar_window("),
            "the tests were not cut off, so the check is reading its own words"
        );

        // And the cutter stops at the next function rather than running on.
        let two_functions = "pub fn edit_only() {\n    edit_only_body();\n}\n\npub fn delete_only() {\n    delete_only_body();\n}\n";
        let edit_only = the_function_for(two_functions, "pub fn edit_only(");
        assert!(edit_only.contains("edit_only_body"));
        assert!(
            !edit_only.contains("delete_only_body"),
            "the cutter ran on into the next function"
        );

        // And the guard-cutter finds the same guard in both real functions
        // and cuts it away rather than leaving it in or cutting too much.
        let window = the_calendar_window();
        for signature in [
            "pub fn edit_selected_event(",
            "pub fn delete_selected_event(",
        ] {
            let function = the_function_for(&window, signature);
            let after = after_the_guard(&function);
            assert!(
                !after.contains("Select an event to"),
                "{signature} still carries its own guard clause after cutting"
            );
            assert!(
                after.len() > 50,
                "{signature} was cut down to {} characters, so the cutter took too much",
                after.len()
            );
        }
    }

    // ── Sync, Edit and Delete answer their own click, not the loop's ────
    //
    // The structural half of the fix this round makes: Sync, Edit and
    // Delete used to end this dialog's own modal loop with their own ID,
    // the same as New and Close still do, and the loop's match arm ran
    // their work only once `show_modal()` returned. A live NVDA run against
    // the Account Manager's own Sign In Again, which used to be wired the
    // same way, found neither of its two sentences is heard: `end_modal`
    // hides the dialog, the work runs and announces entirely
    // synchronously, and the loop calling `show_modal()` again to re-show
    // it leaves nothing yielded to the Windows message pump in between.
    // `wire_calendar_actions` wires these three straight to the button's
    // own `on_click` instead, so this reads each button's own on_click
    // block and checks it calls its function directly rather than
    // `end_modal`.
    //
    // What this cannot see: whether NVDA hears the difference. Only a
    // screen reader run answers that; see the workflow report for this
    // round for what it found.

    /// The body of one `X.on_click({ ... });` call, from its opening brace
    /// to the matching close.
    ///
    /// Brace-counted rather than a fixed character window: a fixed window
    /// either misses a call sitting past its edge, or spills into the next
    /// button's own block and reports its calls as this one's. Every `{`
    /// and `}` inside one of this file's own `on_click` blocks belongs to
    /// Rust code, never a string or a comment holding a stray one, so
    /// counting is exact here.
    fn the_on_click_for<'a>(source: &'a str, marker: &str) -> &'a str {
        let start = source
            .find(marker)
            .unwrap_or_else(|| panic!("no on_click block found for {marker}"));
        let body_start = start + marker.len();
        let mut depth = 1i32;
        let mut end = body_start;
        for (i, c) in source[body_start..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = body_start + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        &source[body_start..end]
    }

    /// Whether `needle` sits on a line of `haystack` that a `//` comment
    /// has not swallowed.
    ///
    /// The same idea `tests/theme_reach.rs`'s own `appears_live` proves it
    /// needs, and for the same reason: `str::contains` cannot tell a live
    /// call from a commented-out one, because a line commented out with
    /// `// end_modal(...)` still holds the call's exact text as a literal
    /// substring.
    fn appears_live(haystack: &str, needle: &str) -> bool {
        haystack.lines().any(|line| {
            line.find(needle)
                .is_some_and(|at| !line[..at].contains("//"))
        })
    }

    #[test]
    fn test_sync_edit_and_delete_answer_their_own_click_directly() {
        let source = the_calendar_window();
        for (marker, function) in [
            ("widgets.sync.on_click({", "request_sync("),
            ("widgets.edit.on_click({", "edit_selected_event("),
            ("widgets.delete.on_click({", "delete_selected_event("),
        ] {
            let block = the_on_click_for(&source, marker);
            assert!(
                appears_live(block, function),
                "{marker} no longer calls {function}"
            );
            assert!(
                !appears_live(block, "end_modal("),
                "{marker} still calls end_modal, which re-triggers the \
                 end_modal-then-immediate-show_modal round trip NVDA cannot \
                 hear an announcement across"
            );
        }

        // New and Close are the two exceptions: New always opens a real
        // nested form before it can reach `said_and_shown`, and Close
        // legitimately ends the session, so both still end this dialog's
        // own modal loop.
        for marker in ["new_btn.on_click({", "close_btn.on_click({"] {
            let block = the_on_click_for(&source, marker);
            assert!(
                appears_live(block, "end_modal("),
                "{marker} no longer calls end_modal, so \
                 show_calendar_dialog's own loop can never see it"
            );
        }
    }

    #[test]
    fn test_the_on_click_reading_can_tell_the_two_apart() {
        // Proving the measurement, the same way every other check in this
        // file does: a read that finds nothing passes, and from outside
        // that is indistinguishable from one that finds everything.
        let wired = "widgets.sync.on_click({\n\
            \x20   let state = Rc::clone(state);\n\
            \x20   move |_| {\n\
            \x20       request_sync(&mut state.borrow_mut(), &status, &a11y);\n\
            \x20   }\n\
            });\n\
            new_btn.on_click({\n\
            \x20   let d = dialog;\n\
            \x20   move |_| {\n\
            \x20       d.end_modal(ID_CAL_NEW);\n\
            \x20   }\n\
            });\n";

        let sync_block = the_on_click_for(wired, "widgets.sync.on_click({");
        assert!(appears_live(sync_block, "request_sync("));
        assert!(!appears_live(sync_block, "end_modal("));
        // And the cutter really does stop at the matching close rather than
        // running on into the next button's own block: New's `end_modal`
        // call sits right after Sync's block in this fixture, and must not
        // turn up in what was read back for Sync.
        assert!(
            !sync_block.contains("end_modal"),
            "the cutter ran on into the next button's own block"
        );

        // Sabotage: comment the real call out, the way a fix half-applied
        // by hand might leave it, and confirm the check goes red rather
        // than reading its own words back.
        let sabotaged = wired.replace(
            "request_sync(&mut state.borrow_mut(), &status, &a11y);",
            "// request_sync(&mut state.borrow_mut(), &status, &a11y);",
        );
        let sabotaged_block = the_on_click_for(&sabotaged, "widgets.sync.on_click({");
        assert!(
            !appears_live(sabotaged_block, "request_sync("),
            "commenting the real call out still read as calling it live"
        );

        // And a comment naming end_modal without calling it does not trip
        // the negative check either: a doc comment explaining why a button
        // no longer calls it must not itself read as the call.
        let mentions_without_calling = "widgets.sync.on_click({\n\
            \x20   // does not call end_modal(ID_CAL_SYNC) any more\n\
            \x20   move |_| {\n\
            \x20       request_sync(&mut state.borrow_mut(), &status, &a11y);\n\
            \x20   }\n\
            });\n";
        let block = the_on_click_for(mentions_without_calling, "widgets.sync.on_click({");
        assert!(
            !appears_live(block, "end_modal("),
            "a comment naming end_modal was read as a live call to it"
        );

        // And the window really was read, because a read that finds
        // nothing looks the same from outside as one that finds
        // everything.
        let window = the_calendar_window();
        assert!(
            window.len() > 5000,
            "only {} characters of the window were read, so the reading is broken",
            window.len()
        );
    }

    /// The window that asks the question, read as text.
    fn the_window_that_asks() -> String {
        std::fs::read_to_string("src/presentation/wx_which_days.rs")
            .expect("the window that asks which days to be readable")
            .replace("\r\n", "\n")
    }

    #[test]
    fn test_the_window_ticks_the_answer_that_is_preselected() {
        // What this cannot see: whether the window opens, or whether the tick is
        // announced. It reads the window's text for the call that preselects.
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
        //
        // What this cannot see: whether a screen reader really reads that
        // answer out when the window opens. Only a run with one says that.
        // What is pinned here is that the code asks for the right thing.
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
        // What this cannot see: whether the description reaches anybody. It
        // reads the window's text for both calls. A description set on a control
        // that is never shown reads the same here as one that is.
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
        //
        // What this cannot see: what Enter really does when the window is on
        // screen. It asks that one button in the source is made the default and
        // that nothing else is. A framework that ignores that call, or a
        // control that swallows Enter first, keeps this green.
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
        // What this cannot see: whether the answer read back is acted on. It
        // reads the window's text for where the answer comes from.
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

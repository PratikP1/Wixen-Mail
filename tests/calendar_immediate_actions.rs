//! Whether each of the Calendar list window's three "answers its own click"
//! functions does its own work correctly: the state it mutates and the
//! sentence it hands to `said_and_shown`.
//!
//! These three - `request_sync`, `edit_selected_event`,
//! `delete_selected_event` - used to run only as a match arm inside
//! `show_calendar_dialog`'s own loop, reached after the button's own
//! `on_click` had already called `end_modal` and hidden the dialog. A live
//! NVDA run against the Account Manager's own Sign In Again, the same shape
//! as these three, found that round trip left nothing yielded to the
//! Windows message pump in between, so neither of its two sentences was
//! ever heard. `wx_calendar.rs`'s own `wire_calendar_actions` now wires
//! each straight to the button's `on_click` instead.
//!
//! Their own logic did not change in that fix, only how they are reached
//! did, so this file only has to pin the logic: given a selection (or
//! none) and a working list, does the right sentence land on `status` and
//! does `CalendarDialogState` end up changed the right way. Whether the
//! sentence really reaches NVDA from inside a real modal dialog is
//! answered by a live screen reader run, not by this file; no such run has
//! been made against this window yet.
//!
//! Two of the three cannot be driven past their own "something is
//! selected" branch from here, because there is no way to fire a real wx
//! click event and observe what a nested modal dialog does from inside a
//! test:
//!
//! - `delete_selected_event` always shows a Confirm Delete dialog before it
//!   can answer anything once a row is selected, so only its "nothing is
//!   selected" answer is reachable.
//! - `edit_selected_event`'s "selected, and the calendar allows it" answer
//!   always shows the event editor, so that path is unreachable too.
//!
//! What is reachable, and what this checks: both functions' "nothing is
//! selected" answer, and `edit_selected_event`'s "selected, but the
//! calendar refuses" answer, which a day that does not repeat reaches with
//! no dialog in the way at all (`which_days_are_meant` answers instantly
//! when nothing repeats, before it ever touches its own parent window).
//! `request_sync` never opens anything, so its one path is checked whole.
//!
//! One `#[test]` function, building one real dialog, for the reason
//! `tests/theme_reach.rs`'s own file comment gives: wxWidgets supports
//! exactly one application per process. `cargo test` runs every file under
//! `tests/` as its own process, so this file having its own single
//! `wxdragon::main` call does not collide with `theme_reach.rs`'s or
//! `account_manager_immediate_actions.rs`'s.

use std::sync::{Arc, Mutex};
use wixen_mail::application::calendar::{WhatTheCalendarAllows, WhereAChangeGoes};
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::ui_types::CalendarEventItem;
use wixen_mail::presentation::wx_calendar::{
    self, CalendarAction, CalendarDialogState, build_calendar_dialog,
};
use wxdragon::prelude::*;

/// One outcome this file checked: its name, and why it was wrong. Empty
/// `Vec` when every check agreed, the same shape `tests/theme_reach.rs` and
/// `tests/account_manager_immediate_actions.rs` use, and for the same
/// reason: one mismatch here should not stop the rest of the checks in the
/// same run from being made, and a failure names exactly which one it was.
type Wrong = Vec<(&'static str, String)>;

fn expect(name: &'static str, got: &str, want: &str, into: &mut Wrong) {
    if got != want {
        into.push((name, format!("got {got:?}, want {want:?}")));
    }
}

fn select_only_row(list: &ListCtrl) {
    list.set_item_state(0, ListItemState::Selected, ListItemState::Selected);
}

/// A day that does not repeat, so `which_days_are_meant` answers instantly
/// and neither `edit_selected_event` nor `delete_selected_event` ever shows
/// a "which days do you mean" dialog when this is the row selected.
fn a_non_repeating_event(id: &str, summary: &str) -> CalendarEventItem {
    CalendarEventItem {
        id: id.to_string(),
        summary: summary.to_string(),
        description: String::new(),
        start: "2026-09-01T09:00:00".to_string(),
        end: "2026-09-01T10:00:00".to_string(),
        location: String::new(),
        is_all_day: false,
        status: String::new(),
        provider: String::new(),
        calendar_id: None,
        calendar_name: None,
        calendar_color: None,
        reminder_minutes: None,
        repeats: String::new(),
        categories: String::new(),
        show_as: String::new(),
        recurrence_rule: None,
    }
}

/// What an ordinary calendar kept on this computer allows: nothing stops an
/// edit or a delete.
fn ordinarily_allowed() -> WhatTheCalendarAllows {
    WhatTheCalendarAllows::just(WhereAChangeGoes::KeptHere)
}

/// What `can_be_honoured` refuses before it ever asks which day was meant:
/// a day still filed at the address of the series it was cut out of. Set
/// directly on the struct, rather than through a calendar server's own
/// sync, because reaching this refusal only needs the answer
/// `edit_selected_event` is handed, not a real occurrence a real sync
/// produced.
fn refuses_a_shared_address() -> WhatTheCalendarAllows {
    WhatTheCalendarAllows {
        goes: WhereAChangeGoes::KeptHere,
        keeping_the_day_apart: None,
        shares_its_address_with_the_series_it_left: true,
        the_series_it_left_is_known_here: false,
    }
}

fn empty_state() -> CalendarDialogState {
    CalendarDialogState {
        actions: Vec::new(),
        events_data: Vec::new(),
        allows: Vec::new(),
    }
}

#[test]
fn test_the_three_immediate_actions_mutate_state_and_report_correctly() {
    let wrong: Arc<Mutex<Wrong>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |app| {
            let frame = Frame::builder().build();
            let a11y = Accessibility::new().expect("accessibility");
            let widgets = build_calendar_dialog(&frame, &[], None);
            let dialog = &widgets.dialog;
            let list = &widgets.list;
            let status = &widgets.status;
            let mut into = Vec::new();

            // ── Sync ────────────────────────────────────────────────────
            // No row to select, so this is the whole of what it answers.
            let mut state = empty_state();
            wx_calendar::request_sync(&mut state, status, &a11y);
            expect(
                "sync: what is said",
                &status.get_label(),
                "Sync requested...",
                &mut into,
            );
            if state.actions.len() != 1
                || !matches!(state.actions[0], CalendarAction::SyncRequested)
            {
                into.push((
                    "sync: state.actions",
                    format!("{:?}, want one SyncRequested", state.actions),
                ));
            }

            // ── Edit, nothing selected ─────────────────────────────────
            let mut state = CalendarDialogState {
                events_data: vec![a_non_repeating_event("evt-1", "Standup")],
                allows: vec![ordinarily_allowed()],
                ..empty_state()
            };
            wx_calendar::edit_selected_event(
                &mut state,
                dialog,
                list,
                status,
                &a11y,
                None,
                &|_, _| None,
            );
            expect(
                "edit, nothing selected: what is said",
                &status.get_label(),
                "Select an event to edit.",
                &mut into,
            );
            if !state.actions.is_empty() {
                into.push((
                    "edit, nothing selected: state.actions",
                    format!("{:?}, want none queued", state.actions),
                ));
            }

            // ── Delete, nothing selected ───────────────────────────────
            // The one path of Delete's this file can reach: a selected row
            // always shows Confirm Delete first, and there is no way to
            // drive that dialog's own click from inside a test.
            let mut state = CalendarDialogState {
                events_data: vec![a_non_repeating_event("evt-1", "Standup")],
                allows: vec![ordinarily_allowed()],
                ..empty_state()
            };
            wx_calendar::delete_selected_event(&mut state, dialog, list, status, &a11y, None);
            expect(
                "delete, nothing selected: what is said",
                &status.get_label(),
                "Select an event to delete.",
                &mut into,
            );
            if !state.actions.is_empty() {
                into.push((
                    "delete, nothing selected: state.actions",
                    format!("{:?}, want none queued", state.actions),
                ));
            }

            // ── Edit, selected, and the calendar refuses ───────────────
            // Reaches `can_be_honoured`'s own refusal with no dialog in the
            // way at all, since `which_days_are_meant` answers instantly
            // for a day that does not repeat, before it ever shows one.
            list.insert_item(0, "2026-09-01 09:00", None);
            select_only_row(list);
            let mut state = CalendarDialogState {
                events_data: vec![a_non_repeating_event("evt-1", "Standup")],
                allows: vec![refuses_a_shared_address()],
                ..empty_state()
            };
            wx_calendar::edit_selected_event(
                &mut state,
                dialog,
                list,
                status,
                &a11y,
                None,
                &|_, _| None,
            );
            let shown = status.get_label();
            if !shown.contains("Changing this day on its own is not something this can do yet") {
                into.push(("edit, refused: what is said", shown));
            }
            if !state.actions.is_empty() {
                into.push((
                    "edit, refused: state.actions",
                    format!("{:?}, want none queued despite the refusal", state.actions),
                ));
            }

            *wrong.lock().unwrap() = into;

            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };

    assert!(result.is_ok(), "wxdragon::main returned {result:?}");
    let wrong = wrong.lock().unwrap();
    assert!(
        wrong.is_empty(),
        "{}",
        wrong
            .iter()
            .map(|(name, why)| format!("{name}: {why}"))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

//! The window that opens when something comes due holds every kind, one row
//! each, and offers for each row only the answers that mean something.
//!
//! Until 06-09 this window held one reminder as a line of text. A task with a
//! due date and a calendar event with an alert now arrive in the same window,
//! and a screen reader user hears the list one row at a time, so each row has
//! to say its kind first, and the buttons have to say what they are for the
//! row that is selected. Those are properties of the running window, read
//! back from the real controls rather than from the source.
//!
//! One `#[test]` function calling `wxdragon::main` once, for the reason
//! `tests/theme_reach.rs` and `tests/every_event_has_a_control.rs` both
//! record: wxWidgets supports one application per process, `cargo test` runs
//! a file's `#[test]` functions in parallel threads, and a second one asserted
//! "initializing twice?" and hung until it was killed.
//!
//! **What this cannot reach, and it is the load-bearing limit.** wxdragon
//! 0.9.17 exposes no way to raise a widget event from outside, so nothing
//! here can press a button or move the selection with the keyboard. What it
//! does instead is drive the real list through the same function the
//! selection handler calls, and read back the real buttons' labels and
//! enabled states. So the assertions below prove the window's structure:
//! that the rows are there in the order given with the kind first, that Mark
//! Done and Details say why they are unavailable, and that six buttons carry
//! six distinct Alt keys. Whether a person tabbing past a disabled button
//! hears why, and whether the rows read well by ear, is a listening pass and
//! is in `.planning/WINDOWS.md`.

use std::sync::{Arc, Mutex};
use wixen_mail::application::due::{Due, Identity, Kind, Snooze};
use wixen_mail::presentation::date_display::DateSettings;
use wixen_mail::presentation::wx_reminder_alert::{
    self, DueWindow, Editors, alt_key_of, build_reminder_alert_dialog,
};
use wxdragon::prelude::*;

/// One property that did not hold: what was being asked, and what was found.
type Wrong = Vec<(String, String)>;

/// Only an event has an editor to open, which is the program's own answer:
/// nothing in it edits an existing task or reminder.
struct OnlyEventsOpen;

impl Editors for OnlyEventsOpen {
    fn has_one_for(&self, kind: Kind) -> bool {
        kind == Kind::Event
    }

    fn open(&mut self, _parent: &Dialog, _row: &Due) -> Option<Due> {
        None
    }
}

fn row(kind: Kind, id: &str, title: &str, when: &str) -> Due {
    Due {
        identity: Identity {
            kind,
            id: id.to_string(),
        },
        title: title.to_string(),
        when: when.to_string(),
        late: false,
    }
}

/// One row of each kind, in time order: the task's whole day is the start of
/// the day, the reminder is at nine, the event at twenty past.
fn one_of_each() -> Vec<Due> {
    vec![
        row(Kind::Task, "t1", "File the report", "2026-09-14"),
        row(Kind::Reminder, "r1", "Call the bank", "2026-09-14T09:00:00"),
        row(
            Kind::Event,
            "e1|2026-09-14T09:20:00",
            "Standup",
            "2026-09-14T09:20:00",
        ),
    ]
}

const TASK: u32 = 0;
const EVENT: u32 = 2;

#[test]
fn test_the_due_window_holds_one_row_of_each_kind_and_offers_what_each_can_take() {
    // `Accessibility::new` is not built here, so `WIXEN_NO_AUDIO` is not
    // needed: the window is built and never shown, and nothing sounds.
    let wrong: Arc<Mutex<Wrong>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |app| {
            let mut wrong = wrong.lock().unwrap();
            let frame = Frame::builder().build();
            let now = chrono::Local::now();
            let window = build_reminder_alert_dialog(
                &frame,
                one_of_each(),
                now,
                DateSettings::default(),
                Snooze::ALL[2],
                None,
                Box::new(OnlyEventsOpen),
            );

            the_rows_are_there_in_order_each_saying_its_kind_first(&window, now, &mut wrong);
            the_buttons_say_what_the_selected_row_can_take(&window, &mut wrong);
            six_buttons_carry_six_distinct_alt_keys(&window, &mut wrong);

            window.dialog.destroy();
            drop(wrong);
            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");

    let wrong = wrong.lock().unwrap();
    assert!(
        wrong.is_empty(),
        "the due window does not hold every kind the way 06-09 asks:\n{}",
        wrong
            .iter()
            .map(|(what, found)| format!("  {what}: {found}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// Three rows, in the order given, each row's text being that `Due`'s own
/// sentence, so the kind is the first word heard.
fn the_rows_are_there_in_order_each_saying_its_kind_first(
    window: &DueWindow,
    now: chrono::DateTime<chrono::Local>,
    wrong: &mut Wrong,
) {
    let rows = one_of_each();
    let listed = window.list.get_count() as usize;
    if listed != rows.len() {
        wrong.push((
            "the list".to_string(),
            format!("holds {listed} rows where {} were given", rows.len()),
        ));
        return;
    }
    for (at, expected) in rows.iter().enumerate() {
        let shown = window.list.get_string(at as u32).unwrap_or_default();
        let sentence = expected.spoken(now, DateSettings::default());
        if shown != sentence {
            wrong.push((
                format!("row {at}"),
                format!("reads {shown:?} where its sentence is {sentence:?}"),
            ));
        }
        let word = expected.identity.kind.word();
        if !shown.starts_with(word) {
            wrong.push((
                format!("row {at}"),
                format!("does not begin with {word:?}: {shown:?}"),
            ));
        }
    }
}

/// Mark Done is unavailable on the event row and its label says why; Details
/// is available there. On the task row it is the other way about.
fn the_buttons_say_what_the_selected_row_can_take(window: &DueWindow, wrong: &mut Wrong) {
    let button = |name: &str, id: Id, wrong: &mut Wrong| -> Option<Button> {
        let found = window.button(id);
        if found.is_none() {
            wrong.push((name.to_string(), "is not in the window".to_string()));
        }
        found
    };

    window.select(EVENT);
    if let Some(done) = button("Mark Done", DueWindow::BUTTONS[2], wrong) {
        if done.is_enabled() {
            wrong.push((
                "Mark Done on the event row".to_string(),
                "is enabled, and an event cannot be done".to_string(),
            ));
        }
        let label = done.get_label();
        if !label.contains("not for an event") {
            wrong.push((
                "Mark Done on the event row".to_string(),
                format!("is labelled {label:?}, which does not say why it is unavailable"),
            ));
        }
    }
    if let Some(details) = button("Details", DueWindow::BUTTONS[5], wrong)
        && !details.is_enabled()
    {
        wrong.push((
            "Details on the event row".to_string(),
            "is disabled, and an event has an editor to open".to_string(),
        ));
    }

    window.select(TASK);
    if let Some(done) = button("Mark Done", DueWindow::BUTTONS[2], wrong) {
        if !done.is_enabled() {
            wrong.push((
                "Mark Done on the task row".to_string(),
                "is disabled, and a task can be done".to_string(),
            ));
        }
        if done.get_label() != wx_reminder_alert::MARK_DONE {
            wrong.push((
                "Mark Done on the task row".to_string(),
                format!("is labelled {:?}", done.get_label()),
            ));
        }
    }
    if let Some(details) = button("Details", DueWindow::BUTTONS[5], wrong) {
        if details.is_enabled() {
            wrong.push((
                "Details on the task row".to_string(),
                "is enabled, and nothing edits an existing task".to_string(),
            ));
        }
        let label = details.get_label();
        if !label.contains("not for a task") {
            wrong.push((
                "Details on the task row".to_string(),
                format!("is labelled {label:?}, which does not say why it is unavailable"),
            ));
        }
    }
}

/// Six buttons, each with a label carrying an Alt key, all six keys distinct,
/// so every answer is one key press and none of them collides.
fn six_buttons_carry_six_distinct_alt_keys(window: &DueWindow, wrong: &mut Wrong) {
    let mut keys = std::collections::HashSet::new();
    for id in DueWindow::BUTTONS {
        let Some(button) = window.button(id) else {
            wrong.push((format!("button {id}"), "is not in the window".to_string()));
            continue;
        };
        let label = button.get_label();
        match alt_key_of(&label) {
            Some(key) => {
                if !keys.insert(key) {
                    wrong.push((
                        format!("button {label:?}"),
                        format!("shares Alt+{key} with another button"),
                    ));
                }
            }
            None => wrong.push((
                format!("button {label:?}"),
                "carries no Alt key in its label".to_string(),
            )),
        }
    }
    if keys.len() != DueWindow::BUTTONS.len() {
        wrong.push((
            "the buttons".to_string(),
            format!(
                "carry {} distinct Alt keys for {} buttons",
                keys.len(),
                DueWindow::BUTTONS.len()
            ),
        ));
    }
}

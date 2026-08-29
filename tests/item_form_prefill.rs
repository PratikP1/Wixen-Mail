//! Editing an existing item has to open on what it already holds, not a
//! blank form: a form with no way to say "this is what is there already" is
//! not an editor, it is a second way to make a new one.
//!
//! This is what lets the Calendar window's own New/Edit Event buttons open
//! this same dialog instead of a second, older one built by hand: `existing`
//! is `None` for New and `Some(&filled)` for Edit, and everything else about
//! the dialog is unchanged either way.
//!
//! One `#[test]` function, building a real dialog, for the reason
//! `tests/theme_reach.rs`'s own file comment gives: wxWidgets supports
//! exactly one application per process, and `cargo test` runs each file
//! under `tests/` as its own process.

use std::sync::{Arc, Mutex};
use wixen_mail::application::item_fields::{FieldName, Filled};
use wixen_mail::application::new_item::ItemKind;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::date_display::DateSettings;
use wixen_mail::presentation::wx_item_form::{Chrome, Container, Prefill, build_item_form_dialog};
use wxdragon::prelude::*;

/// One outcome this file checked: its name, and why it was wrong. Empty
/// `Vec` when every check agreed, the same shape used throughout this
/// project's own live-widget tests, and for the same reason: one mismatch
/// here should not stop the rest of the checks in the same run from being
/// made, and a failure names exactly which one it was.
type Wrong = Vec<(&'static str, String)>;

fn expect_eq(name: &'static str, got: &str, want: &str, into: &mut Wrong) {
    if got != want {
        into.push((name, format!("got {got:?}, want {want:?}")));
    }
}

fn expect(name: &'static str, got: i32, want: i32, into: &mut Wrong) {
    if got != want {
        into.push((name, format!("got {got}, want {want}")));
    }
}

#[test]
fn test_an_existing_item_fills_every_kind_of_field_it_is_opened_on() {
    let wrong: Arc<Mutex<Wrong>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |app| {
            let mut wrong = wrong.lock().unwrap();
            let frame = Frame::builder().build();
            let a11y = Arc::new(Accessibility::new().expect("accessibility"));

            // ── A blank form still says "New", the same as before this
            // ── existed. ─────────────────────────────────────────────────
            let fresh = build_item_form_dialog(
                &frame,
                ItemKind::Event,
                &[],
                &[],
                Chrome {
                    palette: None,
                    a11y: &a11y,
                    asking: None,
                },
                DateSettings::default(),
                None,
            )
            .expect("an Event has fields to ask for");
            expect_eq(
                "a blank form's heading",
                &fresh.dialog.get_label().unwrap_or_default(),
                "New Event",
                &mut wrong,
            );
            fresh.dialog.destroy();

            // ── One of everything: a line, a date, a time, a pick, a
            // ── container, a category, a whole number, a tick. ─────────────
            let mut existing = Filled::default();
            existing.put(FieldName::Title, "Standup");
            existing.put(FieldName::AllDay, "false");
            existing.put(FieldName::StartDate, "2026-03-12");
            existing.put(FieldName::StartTime, "09:00");
            existing.put(FieldName::EndDate, "2026-03-12");
            existing.put(FieldName::EndTime, "09:15");
            existing.put(FieldName::Location, "Room 2");
            existing.put(FieldName::Repeat, "Every week");
            existing.put(FieldName::Status, "Tentative");
            existing.put(FieldName::Category, "Birthday");
            existing.put(FieldName::AlertMinutes, "30");

            let containers = vec![
                Container {
                    id: "cal-1".to_string(),
                    name: "Work".to_string(),
                },
                Container {
                    id: "cal-2".to_string(),
                    name: "Home".to_string(),
                },
            ];

            let widgets = build_item_form_dialog(
                &frame,
                ItemKind::Event,
                &containers,
                &[],
                Chrome {
                    palette: None,
                    a11y: &a11y,
                    asking: None,
                },
                DateSettings::default(),
                Some(Prefill {
                    filled: &existing,
                    container: Some("cal-2"),
                }),
            )
            .expect("an Event has fields to ask for");

            expect_eq(
                "an existing item's heading",
                &widgets.dialog.get_label().unwrap_or_default(),
                "Edit Event",
                &mut wrong,
            );

            let (_, title) = widgets
                .text_fields
                .iter()
                .find(|(f, _)| f.name == FieldName::Title)
                .expect("a title field");
            expect_eq("title prefilled", &title.get_value(), "Standup", &mut wrong);

            let (_, location) = widgets
                .text_fields
                .iter()
                .find(|(f, _)| f.name == FieldName::Location)
                .expect("a location field");
            expect_eq(
                "location prefilled",
                &location.get_value(),
                "Room 2",
                &mut wrong,
            );

            let (_, starts) = widgets
                .date_fields
                .iter()
                .find(|(f, _)| f.name == FieldName::StartDate)
                .expect("a start date field");
            expect(
                "start month prefilled",
                starts
                    .month
                    .get_selection()
                    .map(|i| i as i32 + 1)
                    .unwrap_or(-1),
                3,
                &mut wrong,
            );
            expect("start day prefilled", starts.day.value(), 12, &mut wrong);
            expect(
                "start year prefilled",
                starts.year.value(),
                2026,
                &mut wrong,
            );

            let (_, start_time) = widgets
                .time_fields
                .iter()
                .find(|(f, _)| f.name == FieldName::StartTime)
                .expect("a start time field");
            expect(
                "start hour prefilled",
                start_time.hour.value(),
                9,
                &mut wrong,
            );
            expect(
                "start minute prefilled",
                start_time.minute.value(),
                0,
                &mut wrong,
            );

            let (_, repeat) = widgets
                .pick_fields
                .iter()
                .find(|(f, _)| f.name == FieldName::Repeat)
                .expect("a repeat field");
            expect_eq(
                "repeat prefilled",
                &repeat.get_string_selection().unwrap_or_default(),
                "Every week",
                &mut wrong,
            );

            let (_, status) = widgets
                .pick_fields
                .iter()
                .find(|(f, _)| f.name == FieldName::Status)
                .expect("a status field");
            expect_eq(
                "status prefilled",
                &status.get_string_selection().unwrap_or_default(),
                "Tentative",
                &mut wrong,
            );

            let container = widgets.container_field.expect("a container field");
            expect_eq(
                "container prefilled",
                &container.get_string_selection().unwrap_or_default(),
                "Home",
                &mut wrong,
            );

            let category = widgets.category_field.expect("a category field");
            expect_eq(
                "category prefilled",
                &category.get_value(),
                "Birthday",
                &mut wrong,
            );

            let (_, alert) = widgets
                .whole_fields
                .iter()
                .find(|(f, _)| f.name == FieldName::AlertMinutes)
                .expect("an alert minutes field");
            expect("alert minutes prefilled", alert.value(), 30, &mut wrong);

            let (_, all_day) = widgets
                .tick_fields
                .iter()
                .find(|(f, _)| f.name == FieldName::AllDay)
                .expect("an all day field");
            if all_day.get_value() {
                wrong.push(("all day prefilled", "ticked, wanted unticked".to_string()));
            }

            widgets.dialog.destroy();

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
        "{}",
        wrong
            .iter()
            .map(|(name, why)| format!("{name}: {why}"))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

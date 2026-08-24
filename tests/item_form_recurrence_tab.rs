//! Recurrence used to sit on the same page as everything else an event or a
//! reminder is asked for, four fields between the location and the alert.
//! Most events and most reminders never repeat, so everybody filling in the
//! form had to get past "Repeat", "Repeat for", "Last day" and "How many
//! times" whether or not they were ever going to touch any of them.
//!
//! Recurrence now lives on its own page, reached by moving off the first one
//! rather than by tabbing past it. A kind with nothing to say about
//! recurrence, a task or a note, keeps the single page it always had: an
//! empty second page is not a courtesy, it is a tab stop that goes nowhere.
//!
//! One `#[test]` function, building real dialogs, for the reason
//! `tests/theme_reach.rs`'s own file comment gives: wxWidgets supports
//! exactly one application per process, and `cargo test` runs each file
//! under `tests/` as its own process.

use std::sync::{Arc, Mutex};
use wixen_mail::application::item_fields::FieldName;
use wixen_mail::application::new_item::ItemKind;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::date_display::DateSettings;
use wixen_mail::presentation::wx_item_form::{Chrome, build_item_form_dialog};
use wxdragon::prelude::*;

/// One outcome this file checked: its name, and why it was wrong. Empty
/// `Vec` when every check agreed, the same shape used throughout this
/// project's own live-widget tests, and for the same reason: one mismatch
/// here should not stop the rest of the checks in the same run from being
/// made, and a failure names exactly which one it was.
type Wrong = Vec<(&'static str, String)>;

fn expect(name: &'static str, got: bool, want: bool, into: &mut Wrong) {
    if got != want {
        into.push((name, format!("got {got}, want {want}")));
    }
}

/// Whether a control's parent really is the panel a test expects it on,
/// compared by the underlying native handle: the one thing that cannot lie
/// about which page a control ended up on, where wxdragon derives no
/// equality for either `Window` or `Panel` itself.
fn on_page(control: &impl WxWidget, page: &Panel) -> bool {
    control
        .get_parent()
        .is_some_and(|parent| parent.handle_ptr() == page.handle_ptr())
}

#[test]
fn test_recurrence_is_a_second_page_only_for_a_kind_that_has_any() {
    let wrong: Arc<Mutex<Wrong>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |app| {
            let mut wrong = wrong.lock().unwrap();
            let frame = Frame::builder().build();
            let a11y = Arc::new(Accessibility::new().expect("accessibility"));

            // ── An event has recurrence fields, so it gets a second page,
            // ── not the one showing when the dialog opens. ─────────────────
            let event = build_item_form_dialog(
                &frame,
                ItemKind::Event,
                &[],
                &[],
                Chrome {
                    palette: None,
                    a11y: &a11y,
                },
                DateSettings::default(),
                None,
            )
            .expect("an Event has fields to ask for");
            let pages = event
                .recurrence
                .expect("an Event asks how often it repeats");
            expect(
                "event: two pages",
                pages.notebook.get_page_count() == 2,
                true,
                &mut wrong,
            );
            expect(
                "event: the one-time page opens selected",
                pages.notebook.selection() == 0,
                true,
                &mut wrong,
            );

            // Starts, a one-time field, is a child of the first page.
            let (_, starts) = event
                .date_fields
                .iter()
                .find(|(f, _)| f.name == FieldName::StartDate)
                .expect("an Event asks when it starts");
            expect(
                "event: Starts is on the one-time page",
                on_page(&starts.month, &pages.one_time_page),
                true,
                &mut wrong,
            );

            // The last day a series can end on, a recurrence field, is a
            // child of the second page, not the first.
            let (_, last_day) = event
                .date_fields
                .iter()
                .find(|(f, _)| f.name == FieldName::RepeatUntilDate)
                .expect("an Event can be told when a series stops");
            expect(
                "event: Last day is on the recurrence page",
                on_page(&last_day.month, &pages.recurrence_page),
                true,
                &mut wrong,
            );
            expect(
                "event: Last day is not on the one-time page",
                on_page(&last_day.month, &pages.one_time_page),
                false,
                &mut wrong,
            );

            // The page showing when the dialog opens is the one Starts is
            // on, and the other page is hidden until somebody moves to it:
            // the same rule `wx_settings`'s own tabs already keep.
            expect(
                "event: the one-time page is shown",
                pages.one_time_page.is_shown(),
                true,
                &mut wrong,
            );
            expect(
                "event: the recurrence page is not shown yet",
                pages.recurrence_page.is_shown(),
                false,
                &mut wrong,
            );

            event.dialog.destroy();

            // ── A task asks nothing about recurrence, so it keeps its one
            // ── page rather than being given an empty second one. ────────
            let task = build_item_form_dialog(
                &frame,
                ItemKind::Task,
                &[],
                &[],
                Chrome {
                    palette: None,
                    a11y: &a11y,
                },
                DateSettings::default(),
                None,
            )
            .expect("a Task has fields to ask for");
            expect(
                "task: no recurrence page",
                task.recurrence.is_some(),
                false,
                &mut wrong,
            );
            task.dialog.destroy();

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

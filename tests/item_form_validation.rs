//! Save used to be a bare button carrying `ID_OK`, closing on a click or on
//! Enter through wxWidgets' own default handling for a dialog's affirmative
//! button, with nothing asking whether the answer made sense first. What
//! `read_back` would have given back either way is what this file checks:
//! the same read [`wixen_mail::presentation::wx_item_form::ItemFormWidgets::filled`]
//! exposes and Save's own handler uses before deciding whether to accept an
//! answer.
//!
//! What actually happens when Save is clicked, whether the dialog closes or
//! stays open, is not something this file can drive: nothing in this
//! project's own tests simulates a real button click, since every other
//! `on_click` handler in the whole codebase is likewise proven correct by
//! reading real wxWidgets source for how the click and the dialog's own
//! `ID_OK` handling interact, not by clicking a real button. That reading is
//! recorded in `wx_item_form.rs`'s own module doc comment. What this file
//! proves instead is the part that is genuinely at risk of a real bug: that
//! reading a live dialog's fields back and asking what is wrong with them
//! for real controls gives the right answer, both when something is missing
//! and once everything is filled in correctly.
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

#[test]
fn test_a_live_dialogs_own_fields_are_read_back_the_way_save_reads_them() {
    let wrong: Arc<Mutex<Wrong>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |app| {
            let mut wrong = wrong.lock().unwrap();
            let frame = Frame::builder().build();
            let a11y = Arc::new(Accessibility::new().expect("accessibility"));

            // ── Blank: Title is the one thing genuinely missing. Starts and
            // ── Ends are `required` too, but a real date field is never
            // ── empty in the first place: it opens already showing today,
            // ── through the same real month `Choice` and day and year
            // ── `SpinCtrl` a person would turn to change it, so there is no
            // ── way to reach the "empty date" state `missing` checks for
            // ── through this dialog at all. ─────────────────────────────
            let widgets = build_item_form_dialog(
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

            let (blank, _) = widgets.filled(&[]);
            let problems = blank.problems(ItemKind::Event);
            expect(
                "blank: exactly one problem",
                problems.len() == 1,
                true,
                &mut wrong,
            );
            expect(
                "blank: it names Title",
                problems
                    .first()
                    .is_some_and(|p| p.field.name == FieldName::Title),
                true,
                &mut wrong,
            );

            // ── Type a title in, through the real control: that was the
            // ── only thing missing, and the two dates already shown are
            // ── still equal to each other, so nothing is wrong at all now.
            let (_, title) = widgets
                .text_fields
                .iter()
                .find(|(f, _)| f.name == FieldName::Title)
                .expect("a title field");
            title.set_value("Standup");
            let (filled_in, _) = widgets.filled(&[]);
            expect(
                "title filled in: no problems left",
                filled_in.problems(ItemKind::Event).is_empty(),
                true,
                &mut wrong,
            );

            widgets.dialog.destroy();

            // ── An Event whose Ends is moved before Starts is refused for
            // ── that alone, title and both dates present or not. ───────────
            let widgets = build_item_form_dialog(
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
            let (_, title) = widgets
                .text_fields
                .iter()
                .find(|(f, _)| f.name == FieldName::Title)
                .expect("a title field");
            title.set_value("Standup");
            let (_, ends) = widgets
                .date_fields
                .iter()
                .find(|(f, _)| f.name == FieldName::EndDate)
                .expect("an end date field");
            let (_, starts) = widgets
                .date_fields
                .iter()
                .find(|(f, _)| f.name == FieldName::StartDate)
                .expect("a start date field");
            // A year before whatever Starts already defaults to, so it is
            // before Starts regardless of which real day this test runs on.
            ends.year.set_value(starts.year.value() - 1);

            let (moved, _) = widgets.filled(&[]);
            let problems = moved.problems(ItemKind::Event);
            expect(
                "ends before it starts: exactly one problem",
                problems.len() == 1,
                true,
                &mut wrong,
            );
            expect(
                "ends before it starts: names Ends",
                problems
                    .first()
                    .is_some_and(|p| p.field.name == FieldName::EndDate),
                true,
                &mut wrong,
            );

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

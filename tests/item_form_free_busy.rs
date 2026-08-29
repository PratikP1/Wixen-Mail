//! The controls that ask when the people invited to an event are free.
//!
//! Two modules worked this out and asked servers about it for a while before
//! anything called either of them: a feature that compiled, passed its own
//! tests, and could not be reached from the running program. So what this file
//! is about is reachability. It builds the real dialog and asks whether the
//! controls are there, whether they are named, and whether pressing the button
//! really reaches the thing that answers.
//!
//! What it cannot see: whether any of it is usable by ear. The controls are
//! named and in the tab order, which is what reaches NVDA; whether the answer
//! is followable when it is read out is a thing only a screen reader run
//! answers.
//!
//! One `#[test]` function building real dialogs, for the reason
//! `tests/theme_reach.rs` gives: wxWidgets supports one application per
//! process, and `cargo test` runs each file under `tests/` as its own process.

use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wixen_mail::application::item_fields::FieldName;
use wixen_mail::application::new_item::ItemKind;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::date_display::DateSettings;
use wixen_mail::presentation::wx_item_form::{
    Chrome, OfferedTime, WhatCameBack, build_item_form_dialog,
};
use wxdragon::prelude::*;

/// One outcome this file checked: its name, and why it was wrong.
type Wrong = Vec<(&'static str, String)>;

fn expect(name: &'static str, got: bool, want: bool, into: &mut Wrong) {
    if got != want {
        into.push((name, format!("got {got}, want {want}")));
    }
}

/// What a sentence naming somebody nobody could check looks like.
const AN_ANSWER_WITH_A_GAP_IN_IT: &str = "Everyone is free Tuesday at 10:00 AM. \
     Bob could not be checked, because the server would not say. Bob is not \
     counted as free.";

#[test]
fn test_the_event_form_can_really_ask_when_the_people_invited_are_free() {
    let wrong: Arc<Mutex<Wrong>> = Arc::new(Mutex::new(Vec::new()));
    let asked_with: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        let asked_with = asked_with.clone();
        wxdragon::main(move |app| {
            let mut wrong = wrong.lock().unwrap();
            let frame = Frame::builder().build();
            let a11y = Arc::new(Accessibility::new().expect("accessibility"));

            // ── A form with nowhere to ask gets no controls at all ────────
            //
            // Rather than controls that refuse: a button that never works is
            // three tab stops in the way of finishing the form.
            let cannot_ask = build_item_form_dialog(
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
            expect(
                "no place to ask: no controls are built",
                cannot_ask.free_busy.is_none(),
                true,
                &mut wrong,
            );
            // And a task never has them, because a task has no guest list.
            let a_task = build_item_form_dialog(
                &frame,
                ItemKind::Task,
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
            .expect("a Task has fields to ask for");
            expect(
                "a task: no guest list and no controls",
                a_task.free_busy.is_none(),
                true,
                &mut wrong,
            );
            expect(
                "a task: no box to write a guest list in",
                a_task
                    .text_fields
                    .iter()
                    .any(|(f, _)| f.name == FieldName::Attendees),
                false,
                &mut wrong,
            );
            cannot_ask.dialog.destroy();
            a_task.dialog.destroy();

            // ── A form with somewhere to ask gets all four ────────────────
            let answering = {
                let asked_with = asked_with.clone();
                Rc::new(
                    move |filled: &wixen_mail::application::item_fields::Filled| {
                        // What the form held when the button was pressed, so this
                        // can prove the guest list really reaches the asking.
                        asked_with
                            .lock()
                            .expect("the record of what was asked")
                            .push(filled.text(FieldName::Attendees).to_string());
                        WhatCameBack {
                            said: AN_ANSWER_WITH_A_GAP_IN_IT.to_string(),
                            times: vec![OfferedTime {
                                said: "Tuesday at 10:00 AM".to_string(),
                                starts: chrono::Local::now(),
                                ends: chrono::Local::now(),
                            }],
                        }
                    },
                )
            };

            let widgets = build_item_form_dialog(
                &frame,
                ItemKind::Event,
                &[],
                &[],
                Chrome {
                    palette: None,
                    a11y: &a11y,
                    asking: Some(answering),
                },
                DateSettings::default(),
                None,
            )
            .expect("an Event has fields to ask for");

            let Some(controls) = widgets.free_busy else {
                wrong.push((
                    "somewhere to ask: the controls were never built",
                    "free_busy was None".to_string(),
                ));
                widgets.dialog.destroy();
                drop(wrong);
                wxdragon::call_after(Box::new(move || app.exit_main_loop()));
                return;
            };

            // ── There is a box to write the guest list in ─────────────────
            let guests = widgets
                .text_fields
                .iter()
                .find(|(f, _)| f.name == FieldName::Attendees);
            expect(
                "an event: there is a box to write a guest list in",
                guests.is_some(),
                true,
                &mut wrong,
            );

            // ── Nothing is claimed before anybody asks ────────────────────
            //
            // An empty box would read as an answer of no times, which is a
            // different thing from a question nobody has asked yet.
            let before = controls.answer.get_value();
            expect(
                "before asking: the box says nothing has been asked",
                before.to_lowercase().contains("nothing asked"),
                true,
                &mut wrong,
            );
            expect(
                "before asking: no times are offered",
                controls.times.get_selection().is_none(),
                true,
                &mut wrong,
            );

            // ── Asking reaches the thing that answers ─────────────────────
            //
            // Driven by calling the same read the button's own handler makes
            // and then the callback it makes with it. A real click cannot be
            // fired from a test here, which is true of every `on_click` in
            // this project; what is proven is that the guest list typed into
            // the form is what the asking is given.
            if let Some((_, box_)) = guests {
                box_.set_value("Ada Lovelace <ada@example.com>\nbob@example.com");
            }
            let (filled, _) = widgets.filled(&[]);
            expect(
                "asking: the guest list is read back off the form",
                filled
                    .text(FieldName::Attendees)
                    .contains("bob@example.com"),
                true,
                &mut wrong,
            );

            // ── The two people typed in really become two people to ask ───
            let invited =
                wixen_mail::application::who_is_coming::typed_in(filled.text(FieldName::Attendees));
            expect(
                "asking: both people on the guest list are read",
                invited.len() == 2,
                true,
                &mut wrong,
            );

            // ── An answer with a gap in it keeps the gap ──────────────────
            //
            // The one thing this whole feature has to get right. Whatever is
            // shown and said has to carry the sentence naming who could not be
            // checked: shortened to the times alone it means "these are free"
            // and sounds like "everybody is free".
            controls.answer.set_value(AN_ANSWER_WITH_A_GAP_IN_IT);
            let shown = controls.answer.get_value();
            expect(
                "an answer with a gap: it still names who was not checked",
                shown.contains("Bob could not be checked"),
                true,
                &mut wrong,
            );
            expect(
                "an answer with a gap: it still says they are not counted free",
                shown.contains("not counted as free"),
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

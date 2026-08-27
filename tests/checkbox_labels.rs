//! A check box has to carry its own label, not borrow one from the text
//! beside it.
//!
//! Windows has two accessibility channels and this project needs both, which
//! `CLAUDE.md` says at length and this file is the check for. `set_accessible_name`
//! writes MSAA, which is what NVDA reads for a native control. UI Automation,
//! which is what Narrator reads, is served by the system's own provider for a
//! native control, and that provider takes the name from the window's text.
//!
//! So a check box built with an empty label and named only through
//! `set_accessible_name` has a name on one channel and none on the other. It
//! reads correctly under NVDA and is an unnamed check box under Narrator, which
//! is the shape of bug that passes every test written from the source and every
//! listening pass done with one reader.
//!
//! The item form built both of its check boxes that way: the label went onto a
//! `StaticText` beside the control, the same as it does for a text field, and a
//! text field is the case where that is right. A check box is not, because a
//! check box has somewhere of its own to put it.
//!
//! One `#[test]` function building real dialogs, for the reason
//! `tests/theme_reach.rs` gives: wxWidgets supports one application per process
//! and `cargo test` runs each file under `tests/` as its own process.

use std::sync::{Arc, Mutex};
use wixen_mail::application::item_fields::Filled;
use wixen_mail::application::new_item::ItemKind;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::date_display::DateSettings;
use wixen_mail::presentation::wx_item_form::{Chrome, Prefill, build_item_form_dialog};
use wxdragon::prelude::*;

/// One check that failed: what it was, and what was wrong with it.
type Wrong = Vec<(String, String)>;

/// The label without its mnemonic marker, which is what a reader would say.
fn without_mnemonic(label: &str) -> String {
    label
        .replace("&&", "\u{0}")
        .replace('&', "")
        .replace('\u{0}', "&")
}

#[test]
fn test_every_check_box_in_a_form_carries_its_own_label() {
    let wrong: Arc<Mutex<Wrong>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |app| {
            let mut wrong = wrong.lock().unwrap();
            let frame = Frame::builder().build();
            let a11y = Arc::new(Accessibility::new().expect("accessibility"));

            // Every kind, so a check box added to any of them later is covered
            // without anybody remembering to come back here. The two that have
            // one today are an event's "All day" and a note's "Pin to the top".
            let mut ticks_seen = 0;
            for kind in [
                ItemKind::Event,
                ItemKind::Task,
                ItemKind::Reminder,
                ItemKind::Note,
                ItemKind::Contact,
            ] {
                let Some(widgets) = build_item_form_dialog(
                    &frame,
                    kind,
                    &[],
                    &[],
                    Chrome {
                        palette: None,
                        a11y: &a11y,
                    },
                    DateSettings::default(),
                    None,
                ) else {
                    continue;
                };

                for (field, tick) in &widgets.tick_fields {
                    ticks_seen += 1;
                    let carried = tick.get_label().unwrap_or_default();
                    let wanted = without_mnemonic(field.label);
                    if without_mnemonic(&carried) != wanted {
                        wrong.push((
                            format!("{kind:?} {}", field.label),
                            format!(
                                "the check box carries {carried:?}, so UI Automation has \
                                 no name for it and Narrator reads an unnamed check box. \
                                 Wanted {wanted:?} on the control itself"
                            ),
                        ));
                    }
                }

                widgets.dialog.destroy();
            }

            if ticks_seen == 0 {
                wrong.push((
                    "the check boxes themselves".to_string(),
                    "no form built one, so this guard measured nothing".to_string(),
                ));
            }

            // Nothing else in the form should have been disturbed: a filled
            // form still fills.
            let mut existing = Filled::default();
            existing.put(
                wixen_mail::application::item_fields::FieldName::Pinned,
                "true",
            );
            if let Some(widgets) = build_item_form_dialog(
                &frame,
                ItemKind::Note,
                &[],
                &[],
                Chrome {
                    palette: None,
                    a11y: &a11y,
                },
                DateSettings::default(),
                Some(Prefill {
                    filled: &existing,
                    container: None,
                }),
            ) {
                if let Some((_, pinned)) = widgets.tick_fields.first() {
                    if !pinned.get_value() {
                        wrong.push((
                            "a pinned note opens ticked".to_string(),
                            "it opened unticked, so moving the label broke the prefill".to_string(),
                        ));
                    }
                }
                widgets.dialog.destroy();
            }

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
        "check boxes without a label of their own:\n{}",
        wrong
            .iter()
            .map(|(what, why)| format!("  {what}: {why}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

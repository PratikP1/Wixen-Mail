//! A new event or reminder is headed New, whatever it opens with (ledger 604).
//!
//! Found by 12-08 on 2026-09-24 and not its subject: a new event or reminder
//! opened while Settings holds a default reminder was headed "Edit Event" or
//! "Edit Reminder". The New command hands the form Settings' alert as answers
//! filled in ahead, and the form headed any form with answers "Edit". The
//! default is 15 minutes, so that was every new event and reminder opened from
//! New unless somebody had set the default to none, and the accessibility
//! scan's rows 408 to 417 named the new-event window "Edit Event" for it.
//!
//! Each reading builds the real form the way the New command does, with the
//! alert filled in and the form told it is new, and reads the heading back.
//! The companion builds it the way Edit does and reads "Edit".
//!
//! One window session for the file, every reading sharing it through a
//! `OnceLock`, on the shape `tests/event_times_move_in_blocks.rs` uses.

#![cfg(windows)]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::item_fields::{FieldName, Filled};
use wixen_mail::application::new_item::ItemKind;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::date_display::DateSettings;
use wixen_mail::presentation::wx_item_form::{Chrome, Prefill, build_item_form_dialog};
use wxdragon::prelude::*;

type Harvest = BTreeMap<&'static str, String>;

/// Settings' default reminder, as the New command fills it in.
fn the_default_reminder() -> Filled {
    let mut filled = Filled::default();
    filled.put(FieldName::AlertMinutes, "15");
    filled
}

fn the_heading_of(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
    kind: ItemKind,
    prefill: Prefill,
) -> Result<String, String> {
    let form = build_item_form_dialog(
        frame,
        kind,
        &[],
        &[],
        Chrome {
            palette: None,
            a11y,
            asking: None,
        },
        DateSettings::default(),
        Some(prefill),
    )
    .ok_or_else(|| format!("{kind:?} has no fields to build"))?;
    let heading = form.dialog.get_label().unwrap_or_default();
    form.dialog.destroy();
    Ok(heading)
}

fn read_the_headings(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
    into: &mut Harvest,
) -> Result<(), String> {
    let alert = the_default_reminder();
    for (reading, kind) in [
        ("new event", ItemKind::Event),
        ("new reminder", ItemKind::Reminder),
    ] {
        let heading = the_heading_of(
            frame,
            a11y,
            kind,
            Prefill {
                filled: &alert,
                container: None,
                is_new: true,
            },
        )?;
        into.insert(reading, heading);
    }
    let mut made = the_default_reminder();
    made.put(FieldName::Title, "Standup");
    let heading = the_heading_of(
        frame,
        a11y,
        ItemKind::Event,
        Prefill {
            filled: &made,
            container: None,
            is_new: false,
        },
    )?;
    into.insert("event already made", heading);
    Ok(())
}

fn take_the_harvest() -> Result<Harvest, String> {
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let taken: Result<Harvest, String> = (|| {
                let frame = Frame::builder()
                    .with_title("A new item is headed New, the reading")
                    .build();
                let a11y = Arc::new(
                    Accessibility::new().map_err(|why| format!("accessibility: {why:?}"))?,
                );
                let mut harvest = Harvest::new();
                read_the_headings(&frame, &a11y, &mut harvest)?;
                frame.destroy();
                Ok(harvest)
            })();
            if let Ok(mut slot) = outcome.lock() {
                *slot = Some(taken);
            }
            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };
    if let Err(why) = result {
        return Err(format!("wxdragon::main returned {why:?}"));
    }
    let taken = outcome
        .lock()
        .map_err(|_| "the harvest's lock was poisoned".to_string())?
        .take();
    taken.unwrap_or_else(|| Err("the window session ended without a harvest".to_string()))
}

fn reading(name: &str) -> &'static str {
    static HARVEST: OnceLock<Result<Harvest, String>> = OnceLock::new();
    let harvest = match HARVEST.get_or_init(take_the_harvest) {
        Ok(harvest) => harvest,
        Err(why) => panic!("the window session could not be read: {why}"),
    };
    harvest
        .get(name)
        .map(String::as_str)
        .unwrap_or_else(|| panic!("nothing was read for {name:?}"))
}

#[test]
fn test_a_new_event_opened_with_the_default_reminder_is_headed_new_event() {
    assert_eq!(reading("new event"), "New Event");
}

#[test]
fn test_a_new_reminder_opened_with_the_default_reminder_is_headed_new_reminder() {
    assert_eq!(reading("new reminder"), "New Reminder");
}

#[test]
fn test_companion_an_event_already_made_is_still_headed_edit_event() {
    assert_eq!(reading("event already made"), "Edit Event");
}

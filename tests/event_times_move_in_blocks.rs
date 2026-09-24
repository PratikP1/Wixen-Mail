//! Event times move in blocks, read from built forms (#41).
//!
//! The tester on 2026-09-15: "each event should be blocked for 30 minutes by
//! default ... Pressing up and down arrow keys when picking time should move
//! in those blocks. However the user should be able to choose other times
//! minutely using left and right arrow keys." `application::time_blocks`
//! holds the arithmetic at its boundaries; this file holds the form to it.
//! Each reading builds a real item form with a fixed moment and block, sends
//! keys to the minute spinner's typing field the way the message loop hands
//! them to it, and reads the start and the end back off the controls.
//!
//! **Where the key arrives.** A spin control is two windows, the arrows and
//! the field a person types in, and the keyboard is in the field. The plan
//! assumed a key-down handler bound on the `SpinCtrl` would see the arrows,
//! and asked for that to be measured first. Measured 2026-09-24 by
//! `test_right_reaches_a_handler_on_the_spin_control_and_up_does_not`: a
//! counter bound on a bare spin control's key-down, Right and then Up sent to
//! its field. The handler saw Right and never saw Up, and the value went from
//! 0 to 1: the arrows take Up and Down in the field before wxWidgets forwards
//! the key, and step by one. So the form takes the arrow keys on the field's
//! own window, ahead of the arrows (`presentation::spin_field_keys`), and a
//! key it takes goes no further.
//!
//! **A task has no time.** The plan asked for the task form to open "at the
//! same times"; `item_fields::TASK` holds a due date and no time at all, so
//! there is nothing to open on a boundary or to move by a block. The reading
//! says so, and holds the due date to today rather than a boundary's date.
//!
//! One window session for the file, every reading sharing it through a
//! `OnceLock`, on the shape `tests/mark_as_read_says_which_way_it_will_go.rs`
//! uses. The Windows calls are declared by hand from the headers, as every
//! reading in `tests/` does.

#![cfg(windows)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::item_fields::FieldName;
use wixen_mail::application::new_item::ItemKind;
use wixen_mail::application::time_blocks::Block;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::date_display::{
    Clock, DateOrder, DateSettings, DateStyle, DateWording,
};
use wixen_mail::presentation::wx_item_form::{
    Chrome, DateFields, ItemFormWidgets, TimeFields, Timekeeping, build_item_form_dialog, hour_from,
};
use wxdragon::prelude::*;

const WM_SETTEXT: u32 = 0x000C;
const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
/// commctrl.h: `WM_USER + 106`, the field an up-down control is attached to.
const UDM_GETBUDDY: u32 = 0x0400 + 106;
const VK_LEFT: usize = 0x25;
const VK_UP: usize = 0x26;
const VK_RIGHT: usize = 0x27;
const VK_DOWN: usize = 0x28;
/// The `lParam` of an arrow key's key-down: a repeat count of one and the
/// extended-key bit, which the arrow keys beside the letters carry. Without
/// it wxWidgets reads the key as the numeric keypad's arrow (378 for Right,
/// measured 2026-09-24), which is a different key code.
const AN_ARROW_KEY_GOES_DOWN: isize = 0x0100_0001;
/// The same key's key-up: the previous-state and transition bits as well.
const AN_ARROW_KEY_COMES_UP: isize = 0xC100_0001_u32 as i32 as isize;
/// wxWidgets' own codes for the two arrows the bare reading counts.
const WXK_UP: i32 = 315;
const WXK_RIGHT: i32 = 316;

#[link(name = "user32")]
unsafe extern "system" {
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
}

/// The field a person types in, beside a spin control's arrows.
fn field_of(spin: &SpinCtrl) -> Result<isize, String> {
    let arrows = spin.get_handle() as isize;
    // SAFETY: the arrows are a live window built on this thread; the message
    // takes and returns no pointer.
    let field = unsafe { SendMessageW(arrows, UDM_GETBUDDY, 0, 0) };
    match field {
        0 => Err("a spin control's arrows answered no typing field".to_string()),
        field => Ok(field),
    }
}

/// A key pressed and released in the typing field, the way the message loop
/// hands it over. Arrow keys make no character, so there is no `WM_CHAR`.
fn press(spin: &SpinCtrl, key: usize) -> Result<(), String> {
    let field = field_of(spin)?;
    // SAFETY: a live window on this thread; no pointers.
    unsafe {
        SendMessageW(field, WM_KEYDOWN, key, AN_ARROW_KEY_GOES_DOWN);
        SendMessageW(field, WM_KEYUP, key, AN_ARROW_KEY_COMES_UP);
    }
    Ok(())
}

/// Text put into the typing field the way typing over the selection leaves
/// it, which raises the same change notification a keystroke does.
fn type_into(spin: &SpinCtrl, text: &str) -> Result<(), String> {
    let field = field_of(spin)?;
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: a live window on this thread; the string outlives the call.
    unsafe {
        SendMessageW(field, WM_SETTEXT, 0, wide.as_ptr() as isize);
    }
    Ok(())
}

/// A date and time as the form's controls hold it, `2026-09-24 15:00`.
fn shown(date: &DateFields, time: &TimeFields) -> String {
    let month = date.month.get_selection().map_or(0, |at| at + 1);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        date.year.value(),
        month,
        date.day.value(),
        hour_from(time),
        time.minute.value()
    )
}

fn date_of(widgets: &ItemFormWidgets, name: FieldName) -> Result<DateFields, String> {
    widgets
        .date_fields
        .iter()
        .find(|(field, _)| field.name == name)
        .map(|(_, fields)| *fields)
        .ok_or_else(|| format!("no {name:?} date on the form"))
}

fn time_of(widgets: &ItemFormWidgets, name: FieldName) -> Result<TimeFields, String> {
    widgets
        .time_fields
        .iter()
        .find(|(field, _)| field.name == name)
        .map(|(_, fields)| *fields)
        .ok_or_else(|| format!("no {name:?} time on the form"))
}

/// An event's start and end as the form shows them, `start | end`.
fn start_and_end(widgets: &ItemFormWidgets) -> Result<String, String> {
    Ok(format!(
        "{} | {}",
        shown(
            &date_of(widgets, FieldName::StartDate)?,
            &time_of(widgets, FieldName::StartTime)?
        ),
        shown(
            &date_of(widgets, FieldName::EndDate)?,
            &time_of(widgets, FieldName::EndTime)?
        ),
    ))
}

fn a_moment(day: u32, hour: u32, minute: u32, second: u32) -> chrono::NaiveDateTime {
    chrono::NaiveDate::from_ymd_opt(2026, 9, day)
        .and_then(|date| date.and_hms_opt(hour, minute, second))
        .expect("a real moment")
}

fn on_a_clock(clock: Clock) -> DateSettings {
    DateSettings {
        style: DateStyle::Absolute,
        order: DateOrder::MonthFirst,
        wording: DateWording::Verbal,
        clock,
    }
}

fn a_form(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
    kind: ItemKind,
    time: Timekeeping,
) -> Result<ItemFormWidgets, String> {
    build_item_form_dialog(
        frame,
        kind,
        &[],
        &[],
        Chrome {
            palette: None,
            a11y,
            asking: None,
        },
        time,
        None,
    )
    .ok_or_else(|| format!("{kind:?} has no fields to build"))
}

/// Everything read in the one window session, by reading name.
type Harvest = BTreeMap<&'static str, String>;

/// An event opened at 14:37:10 with half an hour, on a twenty-four hour
/// clock, and the tester's keys on its start.
fn read_the_event(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
    into: &mut Harvest,
) -> Result<(), String> {
    let form = a_form(
        frame,
        a11y,
        ItemKind::Event,
        Timekeeping {
            settings: on_a_clock(Clock::TwentyFourHour),
            now: a_moment(24, 14, 37, 10),
            block: Block::Thirty,
        },
    )?;
    let start = time_of(&form, FieldName::StartTime)?;
    let end = time_of(&form, FieldName::EndTime)?;

    into.insert("event opened", start_and_end(&form)?);
    press(&start.minute, VK_UP)?;
    into.insert("event after Up", start_and_end(&form)?);
    press(&start.minute, VK_DOWN)?;
    press(&start.minute, VK_DOWN)?;
    into.insert("event after Down twice", start_and_end(&form)?);
    press(&start.minute, VK_RIGHT)?;
    into.insert("event after Right", start_and_end(&form)?);
    press(&start.minute, VK_LEFT)?;
    press(&start.minute, VK_LEFT)?;
    into.insert("event after Left twice", start_and_end(&form)?);
    type_into(&end.hour, "16")?;
    type_into(&end.minute, "0")?;
    into.insert("event with the end typed", start_and_end(&form)?);
    press(&start.minute, VK_UP)?;
    into.insert("event after Up with the end typed", start_and_end(&form)?);
    type_into(&start.hour, "9")?;
    into.insert("event with the start typed", start_and_end(&form)?);
    form.dialog.destroy();
    Ok(())
}

/// An event opened at 23:50 with a quarter of an hour: the start is
/// midnight, on the next day.
fn read_the_late_event(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
    into: &mut Harvest,
) -> Result<(), String> {
    let form = a_form(
        frame,
        a11y,
        ItemKind::Event,
        Timekeeping {
            settings: on_a_clock(Clock::TwentyFourHour),
            now: a_moment(24, 23, 50, 0),
            block: Block::Fifteen,
        },
    )?;
    into.insert("late event opened", start_and_end(&form)?);
    let start = time_of(&form, FieldName::StartTime)?;
    press(&start.minute, VK_DOWN)?;
    press(&start.minute, VK_DOWN)?;
    into.insert("late event after Down twice", start_and_end(&form)?);
    form.dialog.destroy();
    Ok(())
}

/// An event on a twelve-hour clock, opened at 11:10 with half an hour, and
/// Up across noon.
fn read_the_twelve_hour_event(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
    into: &mut Harvest,
) -> Result<(), String> {
    let form = a_form(
        frame,
        a11y,
        ItemKind::Event,
        Timekeeping {
            settings: on_a_clock(Clock::TwelveHour),
            now: a_moment(24, 11, 10, 0),
            block: Block::Thirty,
        },
    )?;
    let start = time_of(&form, FieldName::StartTime)?;
    let face = |time: &TimeFields| {
        format!(
            "{}:{:02} {}",
            time.hour.value(),
            time.minute.value(),
            time.am_pm
                .and_then(|choice| choice.get_string_selection())
                .unwrap_or_default()
        )
    };
    into.insert("twelve-hour event opened", face(&start));
    press(&start.minute, VK_UP)?;
    into.insert("twelve-hour event after Up", face(&start));
    let end = time_of(&form, FieldName::EndTime)?;
    into.insert("twelve-hour event's end after Up", face(&end));
    form.dialog.destroy();
    Ok(())
}

/// A reminder opened at 14:37:10 with an hour: one time, and the same keys.
fn read_the_reminder(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
    into: &mut Harvest,
) -> Result<(), String> {
    let form = a_form(
        frame,
        a11y,
        ItemKind::Reminder,
        Timekeeping {
            settings: on_a_clock(Clock::TwentyFourHour),
            now: a_moment(24, 14, 37, 10),
            block: Block::Sixty,
        },
    )?;
    let date = date_of(&form, FieldName::DueDate)?;
    let time = time_of(&form, FieldName::DueTime)?;
    into.insert("reminder opened", shown(&date, &time));
    press(&time.minute, VK_UP)?;
    into.insert("reminder after Up", shown(&date, &time));
    press(&time.minute, VK_LEFT)?;
    into.insert("reminder after Left", shown(&date, &time));
    form.dialog.destroy();
    Ok(())
}

/// A task opened at 23:50: no time to move, and the due date is today.
fn read_the_task(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
    into: &mut Harvest,
) -> Result<(), String> {
    let form = a_form(
        frame,
        a11y,
        ItemKind::Task,
        Timekeeping {
            settings: on_a_clock(Clock::TwentyFourHour),
            now: a_moment(24, 23, 50, 0),
            block: Block::Fifteen,
        },
    )?;
    into.insert("task's times", form.time_fields.len().to_string());
    let due = date_of(&form, FieldName::DueDate)?;
    into.insert(
        "task's due date",
        format!(
            "{:04}-{:02}-{:02}",
            due.year.value(),
            due.month.get_selection().map_or(0, |at| at + 1),
            due.day.value()
        ),
    );
    form.dialog.destroy();
    Ok(())
}

/// Where the key arrives, and what an Up nobody consumes does: a bare spin
/// control with a counter on its key-down that leaves the key to the control.
fn read_a_bare_spin_control(frame: &Frame, into: &mut Harvest) -> Result<(), String> {
    let spin = SpinCtrl::builder(frame).build();
    spin.set_range(0, 59);
    spin.set_value(0);
    let seen = Rc::new(RefCell::new(Vec::new()));
    spin.bind_internal(EventType::KEY_DOWN, {
        let seen = seen.clone();
        move |event| {
            seen.borrow_mut()
                .push(event.get_key_code().unwrap_or_default());
            event.skip(true);
        }
    });
    press(&spin, VK_RIGHT)?;
    press(&spin, VK_UP)?;
    let seen = seen.borrow();
    into.insert(
        "bare spin control's handler saw Right",
        seen.iter()
            .filter(|code| **code == WXK_RIGHT)
            .count()
            .to_string(),
    );
    into.insert(
        "bare spin control's handler saw Up",
        seen.iter()
            .filter(|code| **code == WXK_UP)
            .count()
            .to_string(),
    );
    into.insert("bare spin control after Up", spin.value().to_string());
    spin.destroy();
    Ok(())
}

fn take_the_harvest() -> Result<Harvest, String> {
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let taken: Result<Harvest, String> = (|| {
                let frame = Frame::builder()
                    .with_title("Event times move in blocks, the reading")
                    .build();
                let a11y = Arc::new(
                    Accessibility::new().map_err(|why| format!("accessibility: {why:?}"))?,
                );
                let mut harvest = Harvest::new();
                read_a_bare_spin_control(&frame, &mut harvest)?;
                read_the_event(&frame, &a11y, &mut harvest)?;
                read_the_late_event(&frame, &a11y, &mut harvest)?;
                read_the_twelve_hour_event(&frame, &a11y, &mut harvest)?;
                read_the_reminder(&frame, &a11y, &mut harvest)?;
                read_the_task(&frame, &a11y, &mut harvest)?;
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

/// The one harvest of this process, taken by whichever test asks first.
fn the_harvest() -> &'static Harvest {
    static HARVEST: OnceLock<Result<Harvest, String>> = OnceLock::new();
    match HARVEST.get_or_init(take_the_harvest) {
        Ok(harvest) => harvest,
        Err(why) => panic!("the window session could not be read: {why}"),
    }
}

fn reading(name: &str) -> &'static str {
    the_harvest()
        .get(name)
        .map(String::as_str)
        .unwrap_or_else(|| panic!("nothing was read for {name:?}"))
}

// ── Where the key arrives, and the step it replaces ────────────────────────

#[test]
fn test_right_reaches_a_handler_on_the_spin_control_and_up_does_not() {
    // The measurement that put the form's handler on the field's own window.
    // If Up ever starts arriving here, a handler on the spin control would do,
    // and this says so.
    assert_eq!(reading("bare spin control's handler saw Right"), "1");
    assert_eq!(reading("bare spin control's handler saw Up"), "0");
}

#[test]
fn test_companion_an_up_nobody_consumes_is_the_native_step_of_one() {
    // What a handler that stepped by the block and left the key unconsumed
    // would add on top: the event readings below expect 15:30 after Up, and
    // this is the minute that would make it 15:31.
    assert_eq!(reading("bare spin control after Up"), "1");
}

// ── An event ───────────────────────────────────────────────────────────────

#[test]
fn test_a_new_event_at_14_37_opens_at_15_00_and_ends_at_15_30() {
    assert_eq!(
        reading("event opened"),
        "2026-09-24 15:00 | 2026-09-24 15:30"
    );
}

#[test]
fn test_up_on_the_start_moves_it_half_an_hour_and_the_end_with_it() {
    assert_eq!(
        reading("event after Up"),
        "2026-09-24 15:30 | 2026-09-24 16:00"
    );
}

#[test]
fn test_down_twice_on_the_start_moves_it_back_an_hour_and_the_end_with_it() {
    assert_eq!(
        reading("event after Down twice"),
        "2026-09-24 14:30 | 2026-09-24 15:00"
    );
}

#[test]
fn test_right_on_the_start_moves_it_one_minute_and_the_end_with_it() {
    assert_eq!(
        reading("event after Right"),
        "2026-09-24 14:31 | 2026-09-24 15:01"
    );
}

#[test]
fn test_left_on_the_start_moves_it_back_one_minute_at_a_time() {
    assert_eq!(
        reading("event after Left twice"),
        "2026-09-24 14:29 | 2026-09-24 14:59"
    );
}

#[test]
fn test_an_end_typed_over_is_where_the_person_put_it() {
    assert_eq!(
        reading("event with the end typed"),
        "2026-09-24 14:29 | 2026-09-24 16:00"
    );
}

#[test]
fn test_an_end_the_person_typed_stays_when_the_start_moves_again() {
    assert_eq!(
        reading("event after Up with the end typed"),
        "2026-09-24 14:30 | 2026-09-24 16:00"
    );
}

#[test]
fn test_a_start_typed_over_still_works_and_the_edited_end_stays() {
    assert_eq!(
        reading("event with the start typed"),
        "2026-09-24 09:30 | 2026-09-24 16:00"
    );
}

#[test]
fn test_a_new_event_late_in_the_evening_opens_at_midnight_on_the_next_day() {
    assert_eq!(
        reading("late event opened"),
        "2026-09-25 00:00 | 2026-09-25 00:15"
    );
}

#[test]
fn test_down_on_the_start_crosses_midnight_back_into_the_day_before_with_the_end() {
    assert_eq!(
        reading("late event after Down twice"),
        "2026-09-24 23:30 | 2026-09-24 23:45"
    );
}

#[test]
fn test_on_a_twelve_hour_clock_up_crosses_noon_and_says_afternoon() {
    assert_eq!(reading("twelve-hour event opened"), "11:30 AM");
    assert_eq!(reading("twelve-hour event after Up"), "12:00 PM");
    assert_eq!(reading("twelve-hour event's end after Up"), "12:30 PM");
}

// ── A reminder and a task ──────────────────────────────────────────────────

#[test]
fn test_a_new_reminder_opens_on_the_next_hour_and_moves_by_it() {
    assert_eq!(reading("reminder opened"), "2026-09-24 15:00");
    assert_eq!(reading("reminder after Up"), "2026-09-24 16:00");
    assert_eq!(reading("reminder after Left"), "2026-09-24 15:59");
}

#[test]
fn test_a_task_has_no_time_and_its_due_date_is_today() {
    assert_eq!(reading("task's times"), "0");
    assert_eq!(reading("task's due date"), "2026-09-24");
}

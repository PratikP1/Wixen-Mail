//! The event form scrolls to the field in focus, and never grows past the
//! screen (ledger 417, #57 point 4).
//!
//! The scan read Edit Event 720 pixels tall on the Accessibility runner's
//! 768-pixel screen, with Show as, Status, Category and Times offered six
//! pixels tall at the bottom edge. The form did not scroll, so on a small
//! screen or at twice the text size the last fields were cut off and a
//! keyboard user reached a field nobody could see (WCAG 1.4.10, 1.4.4,
//! 2.4.11).
//!
//! Each reading builds the real form, shows it, gives it less room than its
//! fields need, moves focus to a field at the bottom and reads, through
//! Windows, whether that field's rectangle lies inside the client area of
//! every window it sits in up to the dialog: the one geometric fact that says
//! it can be seen. Save and the line that says why Save refused are read the
//! same way, since a form whose Save has scrolled away cannot be kept.
//!
//! **Twice the text is read as the smallest size the form can be made.** The
//! plan asked for the form built with its font doubled. That cannot be done
//! from here: a dialog does not inherit its parent's font (measured
//! 2026-09-28, the parent set to 18 points and the form's title at 9),
//! wxdragon offers no way to reach a form's labels once it is built, and
//! Windows' own text size is a setting of the machine this runs on, which a
//! test does not change. Twice the text in a given room is the same form in
//! half of it, since every part of it doubles, so a form read at every size
//! from the smallest it can be made up to its own has been read at twice the
//! text wherever that lands. The first red of this file read half of 480
//! instead, and found the description taller than the page that leaves it,
//! which wxWidgets then does not scroll to at all; the form now keeps a page
//! at least as tall as its tallest field, so half of 480 is below its
//! smallest and the smallest is what is read.
//!
//! What this cannot read is width: the fields twice as wide as well.
//!
//! **The largest size is read as well as the size.** On a screen tall enough
//! to hold the form, the size it opens at is within the working area whether
//! or not anything caps it, so the cap is read as the largest size the form
//! can be made, which is set whatever the screen.
//!
//! Tab is walked with `Navigate`, which is what wxWidgets' own Tab handling
//! calls, and the focus is read from Windows after each step. The walks are
//! compared with the order read before the scrolling existed, written below
//! as data with its date.
//!
//! One window session for the file, every reading sharing it through a
//! `OnceLock`, on the shape `tests/event_times_move_in_blocks.rs` uses.

#![cfg(windows)]

use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::item_fields::Filled;
use wixen_mail::application::new_item::ItemKind;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::date_display::DateSettings;
use wixen_mail::presentation::wx_item_form::{
    Chrome, ItemFormWidgets, WhatCameBack, build_item_form_dialog,
};
use wxdragon::prelude::*;

/// The height the runner's screen leaves a form once its title bar and the
/// taskbar are paid for, less a margin: small enough that the event form's
/// fields cannot all show at once.
const A_SMALL_SCREENS_ROOM: i32 = 480;

/// Less room than any form can be made to fit in, so the form takes the
/// smallest it allows.
const NO_ROOM_AT_ALL: i32 = 1;

/// commctrl.h: `WM_USER + 106`, the field an up-down control is attached to.
const UDM_GETBUDDY: u32 = 0x0400 + 106;
/// winuser.h: `MONITOR_DEFAULTTONEAREST`.
const THE_NEAREST_MONITOR: u32 = 2;

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl Rect {
    fn holds(&self, other: &Rect) -> bool {
        other.left >= self.left
            && other.top >= self.top
            && other.right <= self.right
            && other.bottom <= self.bottom
    }
}

#[repr(C)]
#[derive(Default)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
#[derive(Default)]
struct MonitorInfo {
    size: u32,
    monitor: Rect,
    work: Rect,
    flags: u32,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn GetWindowRect(hwnd: isize, rect: *mut Rect) -> i32;
    fn GetClientRect(hwnd: isize, rect: *mut Rect) -> i32;
    fn ClientToScreen(hwnd: isize, point: *mut Point) -> i32;
    fn GetParent(hwnd: isize) -> isize;
    fn GetFocus() -> isize;
    fn GetClassNameW(hwnd: isize, name: *mut u16, most: i32) -> i32;
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
    fn MonitorFromWindow(hwnd: isize, flags: u32) -> isize;
    fn GetMonitorInfoW(monitor: isize, info: *mut MonitorInfo) -> i32;
}

type Harvest = BTreeMap<&'static str, String>;

/// A window's own rectangle, on the screen.
fn window_rect(hwnd: isize) -> Rect {
    let mut rect = Rect::default();
    // SAFETY: a live window on this thread; the rectangle outlives the call.
    unsafe { GetWindowRect(hwnd, &mut rect) };
    rect
}

/// The part of a window its children can be seen in, on the screen.
fn client_rect_on_screen(hwnd: isize) -> Rect {
    let mut rect = Rect::default();
    let mut corner = Point::default();
    // SAFETY: a live window on this thread; both outlive the calls.
    unsafe {
        GetClientRect(hwnd, &mut rect);
        ClientToScreen(hwnd, &mut corner);
    }
    Rect {
        left: corner.x,
        top: corner.y,
        right: corner.x + rect.right,
        bottom: corner.y + rect.bottom,
    }
}

fn class_of(hwnd: isize) -> String {
    let mut name = [0u16; 128];
    // SAFETY: the buffer is as long as the call is told.
    let len = unsafe { GetClassNameW(hwnd, name.as_mut_ptr(), name.len() as i32) };
    String::from_utf16_lossy(&name[..len.max(0) as usize])
}

/// Whether a control can be seen: its rectangle inside the client area of
/// every window it sits in, up to and including the dialog. "seen", or which
/// window cuts it off and where.
fn seen(control: isize, dialog: isize) -> String {
    let own = window_rect(control);
    let mut inside = control;
    loop {
        // SAFETY: a live window on this thread.
        inside = unsafe { GetParent(inside) };
        if inside == 0 {
            return format!("not seen: {own:?} is not inside the dialog at all");
        }
        let room = client_rect_on_screen(inside);
        if !room.holds(&own) {
            return format!(
                "not seen: {own:?} runs outside {} at {room:?}",
                class_of(inside)
            );
        }
        if inside == dialog {
            return "seen".to_string();
        }
    }
}

/// The working area of the screen a window is on: the screen less the taskbar.
fn working_area_of(hwnd: isize) -> Rect {
    let mut info = MonitorInfo {
        size: std::mem::size_of::<MonitorInfo>() as u32,
        ..MonitorInfo::default()
    };
    // SAFETY: the monitor is one Windows handed back; the structure carries
    // its own size, as the call requires.
    unsafe {
        let monitor = MonitorFromWindow(hwnd, THE_NEAREST_MONITOR);
        GetMonitorInfoW(monitor, &mut info);
    }
    info.work
}

/// The window a form opens from, never shown.
fn a_frame() -> Frame {
    Frame::builder()
        .with_title("The event form scrolls, the reading")
        .build()
}

fn no_answer() -> Rc<dyn Fn(&Filled) -> WhatCameBack> {
    Rc::new(|_: &Filled| WhatCameBack {
        said: String::new(),
        times: Vec::new(),
    })
}

fn a_form(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
    kind: ItemKind,
) -> Result<ItemFormWidgets, String> {
    let asking = (kind == ItemKind::Event).then(no_answer);
    build_item_form_dialog(
        frame,
        kind,
        &[],
        &[],
        Chrome {
            palette: None,
            a11y,
            asking,
        },
        DateSettings::default(),
        None,
    )
    .ok_or_else(|| format!("{kind:?} built no form"))
}

/// The event form's last field on the One-Time page, the description.
fn the_description(form: &ItemFormWidgets) -> Result<TextCtrl, String> {
    form.text_fields
        .last()
        .map(|(_, field)| *field)
        .ok_or_else(|| "the event form has no text field".to_string())
}

/// The event form given `height` pixels inside its frame: the height it
/// opened at, the height it took, and whether the fields at the bottom and
/// Save are seen.
fn read_the_event_in(
    height: i32,
    a11y: &Arc<Accessibility>,
    into: &mut Harvest,
    keys: [&'static str; 5],
) -> Result<(), String> {
    let [opened, took, description, times_offered, save_and_problem] = keys;
    let frame = a_frame();
    let form = a_form(&frame, a11y, ItemKind::Event)?;
    let dialog = form.dialog.get_handle() as isize;

    form.dialog.show(true);
    let client = form.dialog.get_client_size();
    into.insert(opened, client.height.to_string());
    form.dialog.set_client_size(Size::new(client.width, height));
    into.insert(took, form.dialog.get_client_size().height.to_string());

    let notes = the_description(&form)?;
    notes.set_focus();
    into.insert(description, seen(notes.get_handle() as isize, dialog));

    let offered = form
        .free_busy
        .as_ref()
        .map(|controls| controls.times)
        .ok_or("the event form was built with somewhere to ask and has no times")?;
    offered.set_focus();
    into.insert(times_offered, seen(offered.get_handle() as isize, dialog));

    let save = seen(form.save.get_handle() as isize, dialog);
    let problem = seen(form.problem_line.get_handle() as isize, dialog);
    into.insert(
        save_and_problem,
        format!("save {save}, problem line {problem}"),
    );

    form.dialog.destroy();
    frame.destroy();
    Ok(())
}

/// Whether a size fits inside the working area, or how it does not.
fn within(size: Size, work: &Rect) -> String {
    let (wide, tall) = (work.right - work.left, work.bottom - work.top);
    match size.width <= wide && size.height <= tall {
        true => "within".to_string(),
        false => format!(
            "{} by {} on a working area {wide} by {tall}",
            size.width, size.height
        ),
    }
}

/// The event form as built, against the working area of the screen its
/// parent is on: the size it opens at, and the largest it can be made.
///
/// The largest is what reads the cap on any screen. The size it opens at is
/// within the working area whenever the form's fields fit, which on a tall
/// screen they do with or without a cap; the largest size is the cap itself,
/// set whatever the fields need, so it is read here even where the screen has
/// room to spare. A form with no largest size answers -1 by -1, which is
/// within nothing.
fn read_the_size_built(a11y: &Arc<Accessibility>, into: &mut Harvest) -> Result<(), String> {
    let frame = a_frame();
    let form = a_form(&frame, a11y, ItemKind::Event)?;
    let work = working_area_of(frame.get_handle() as isize);
    into.insert(
        "event: the size it opens at",
        within(form.dialog.get_size(), &work),
    );
    let largest = form.dialog.get_max_size();
    into.insert(
        "event: the largest it can be made",
        match largest.width > 0 && largest.height > 0 {
            true => within(largest, &work),
            false => format!("no largest size: {} by {}", largest.width, largest.height),
        },
    );
    form.dialog.destroy();
    frame.destroy();
    Ok(())
}

/// A task, which fits: whether its last field shows as built, with nothing
/// scrolled and no focus moved.
fn read_the_task(a11y: &Arc<Accessibility>, into: &mut Harvest) -> Result<(), String> {
    let frame = a_frame();
    let form = a_form(&frame, a11y, ItemKind::Task)?;
    let dialog = form.dialog.get_handle() as isize;
    form.dialog.show(true);
    let last = form
        .text_fields
        .last()
        .map(|(_, field)| *field)
        .ok_or("the task form has no text field")?;
    into.insert(
        "task: its last field as built",
        seen(last.get_handle() as isize, dialog),
    );
    form.dialog.destroy();
    frame.destroy();
    Ok(())
}

/// One place Tab can stop, by the name this file gives it, the windows focus
/// can land in for it, and a way to move on from it.
struct Stop {
    name: String,
    handles: Vec<isize>,
    onward: Box<dyn Fn() -> bool>,
}

/// Tab from `widget`, the way wxWidgets' own Tab handling moves on.
///
/// `false` because wxdragon 0.9.17 passes `navigate(true)` to wxWidgets as
/// its backward flag: `WXD_NAVIGATION_NEXT` is 0, and 0 is
/// `wxNavigationKeyEvent::IsBackward`. Measured here on 2026-09-28, when the
/// first walk went from the title back to the tabs; `wx_compose.rs` found the
/// same and says so where it calls it.
fn tab_from(widget: &impl WxWidget) -> bool {
    widget.navigate(false)
}

fn stop(name: impl Into<String>, widget: impl WxWidget + 'static) -> Stop {
    let handle = widget.get_handle() as isize;
    Stop {
        name: name.into(),
        handles: vec![handle],
        onward: Box::new(move || tab_from(&widget)),
    }
}

/// A spin control, where focus lands in the field beside the arrows.
fn spin_stop(name: impl Into<String>, spin: SpinCtrl) -> Stop {
    let arrows = spin.get_handle() as isize;
    // SAFETY: a live window on this thread; no pointers.
    let field = unsafe { SendMessageW(arrows, UDM_GETBUDDY, 0, 0) };
    Stop {
        name: name.into(),
        handles: vec![arrows, field],
        onward: Box::new(move || tab_from(&spin)),
    }
}

/// Every place Tab can stop in a form, named.
fn stops_in(form: &ItemFormWidgets) -> Vec<Stop> {
    let mut stops = Vec::new();
    for (field, text) in &form.text_fields {
        stops.push(stop(format!("{:?}", field.name), *text));
    }
    for (field, date) in &form.date_fields {
        stops.push(stop(format!("{:?} month", field.name), date.month));
        stops.push(spin_stop(format!("{:?} day", field.name), date.day));
        stops.push(spin_stop(format!("{:?} year", field.name), date.year));
    }
    for (field, time) in &form.time_fields {
        stops.push(spin_stop(format!("{:?} hour", field.name), time.hour));
        stops.push(spin_stop(format!("{:?} minute", field.name), time.minute));
        if let Some(am_pm) = time.am_pm {
            stops.push(stop(format!("{:?} am or pm", field.name), am_pm));
        }
    }
    for (field, pick) in &form.pick_fields {
        stops.push(stop(format!("{:?}", field.name), *pick));
    }
    for (field, whole) in &form.whole_fields {
        stops.push(spin_stop(format!("{:?}", field.name), *whole));
    }
    for (field, tick) in &form.tick_fields {
        stops.push(stop(format!("{:?}", field.name), *tick));
    }
    if let Some(container) = form.container_field {
        stops.push(stop("Container", container));
    }
    if let Some(category) = form.category_field {
        stops.push(stop("Category", category));
    }
    if let Some(asking) = form.free_busy.as_ref() {
        stops.push(stop("find when everyone is free", asking.ask));
        stops.push(stop("the answer", asking.answer));
        stops.push(stop("times offered", asking.times));
        stops.push(stop("put it in", asking.put_it_in));
    }
    if let Some(pages) = form.recurrence {
        stops.push(stop("the tabs", pages.notebook));
    }
    stops.push(stop("Save", form.save));
    stops
}

/// Which stop has focus: the window Windows says, or, for a combo box, the
/// typing field inside it, which is where its focus lands.
fn the_stop_with_focus(stops: &[Stop]) -> Result<&Stop, String> {
    // SAFETY: no arguments; the answer is a handle or nought.
    let focus = unsafe { GetFocus() };
    // SAFETY: a live window on this thread, or nought, which answers nought.
    let around = unsafe { GetParent(focus) };
    let in_a_combo_box = class_of(focus) == "Edit" && class_of(around) == "ComboBox";
    stops
        .iter()
        .find(|stop| stop.handles.contains(&focus))
        .or_else(|| {
            stops
                .iter()
                .find(|stop| in_a_combo_box && stop.handles.contains(&around))
        })
        .ok_or_else(|| {
            format!(
                "something else, a {} in a {}",
                class_of(focus),
                class_of(around)
            )
        })
}

/// Tab from the first field to Save, each stop by name, or where it went
/// somewhere this file has no name for.
fn walk_from_the_first_field(form: &ItemFormWidgets) -> String {
    let stops = stops_in(form);
    let mut met = Vec::new();
    if let Some((_, first)) = form.text_fields.first() {
        first.set_focus();
    }
    for _ in 0..60 {
        match the_stop_with_focus(&stops) {
            Ok(here) => {
                met.push(here.name.clone());
                if here.name == "Save" || !(here.onward)() {
                    break;
                }
            }
            Err(somewhere) => {
                met.push(somewhere);
                break;
            }
        }
    }
    met.join(", ")
}

fn read_the_walks(a11y: &Arc<Accessibility>, into: &mut Harvest) -> Result<(), String> {
    let frame = a_frame();
    for (kind, key) in [
        (ItemKind::Event, "event: Tab from the first field to Save"),
        (ItemKind::Task, "task: Tab from the first field to Save"),
    ] {
        let form = a_form(&frame, a11y, kind)?;
        form.dialog.show(true);
        into.insert(key, walk_from_the_first_field(&form));
        form.dialog.destroy();
    }
    frame.destroy();
    Ok(())
}

fn take_the_harvest() -> Result<Harvest, String> {
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let taken: Result<Harvest, String> = (|| {
                let a11y = Arc::new(
                    Accessibility::new().map_err(|why| format!("accessibility: {why:?}"))?,
                );
                let mut harvest = Harvest::new();
                read_the_event_in(
                    A_SMALL_SCREENS_ROOM,
                    &a11y,
                    &mut harvest,
                    [
                        "a small screen: the height it opened at",
                        "a small screen: the height it took",
                        "a small screen: the description with focus",
                        "a small screen: times offered with focus",
                        "a small screen: save and the problem line",
                    ],
                )?;
                read_the_event_in(
                    NO_ROOM_AT_ALL,
                    &a11y,
                    &mut harvest,
                    [
                        "its smallest: the height it opened at",
                        "its smallest: the height it took",
                        "its smallest: the description with focus",
                        "its smallest: times offered with focus",
                        "its smallest: save and the problem line",
                    ],
                )?;
                read_the_size_built(&a11y, &mut harvest)?;
                read_the_task(&a11y, &mut harvest)?;
                read_the_walks(&a11y, &mut harvest)?;
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

// ── A small screen's room ──────────────────────────────────────────────────

#[test]
fn test_the_event_form_can_be_given_a_small_screens_room() {
    assert_eq!(
        reading("a small screen: the height it took"),
        A_SMALL_SCREENS_ROOM.to_string()
    );
}

#[test]
fn test_on_a_small_screen_the_description_is_seen_once_it_has_focus() {
    assert_eq!(
        reading("a small screen: the description with focus"),
        "seen"
    );
}

#[test]
fn test_on_a_small_screen_times_offered_is_seen_once_it_has_focus() {
    assert_eq!(reading("a small screen: times offered with focus"), "seen");
}

#[test]
fn test_on_a_small_screen_save_and_the_problem_line_stay_in_the_window() {
    assert_eq!(
        reading("a small screen: save and the problem line"),
        "save seen, problem line seen"
    );
}

// ── The smallest it can be made, which is where twice the text lands ───────

fn the_height(name: &str) -> i32 {
    reading(name)
        .parse()
        .unwrap_or_else(|why| panic!("{name:?} is not a height: {why}"))
}

#[test]
fn test_the_event_form_can_be_made_smaller_than_its_fields() {
    let opened = the_height("its smallest: the height it opened at");
    let took = the_height("its smallest: the height it took");
    assert!(
        took < opened,
        "asked for no room at all, the form stayed {took} of the {opened} its fields take"
    );
}

#[test]
fn test_at_its_smallest_the_description_is_seen_once_it_has_focus() {
    assert_eq!(reading("its smallest: the description with focus"), "seen");
}

#[test]
fn test_at_its_smallest_times_offered_is_seen_once_it_has_focus() {
    assert_eq!(reading("its smallest: times offered with focus"), "seen");
}

#[test]
fn test_at_its_smallest_save_and_the_problem_line_stay_in_the_window() {
    assert_eq!(
        reading("its smallest: save and the problem line"),
        "save seen, problem line seen"
    );
}

// ── Never larger than the screen ───────────────────────────────────────────

#[test]
fn test_the_event_form_opens_within_the_working_area() {
    assert_eq!(reading("event: the size it opens at"), "within");
}

#[test]
fn test_the_event_form_can_be_made_no_larger_than_the_working_area() {
    assert_eq!(reading("event: the largest it can be made"), "within");
}

#[test]
fn test_a_task_that_fits_shows_its_last_field_as_built() {
    assert_eq!(reading("task: its last field as built"), "seen");
}

// ── Tab meets what it met before ───────────────────────────────────────────

/// The event form's Tab order, read 2026-09-28 on `main` at `d18296cd`,
/// before the fields were put in a scrolled area. The calendar is left out
/// because the form was given no calendars to choose from, and the tabs are
/// not met because Tab stops at Save before it gets round to them.
const THE_EVENT_FORMS_TAB_ORDER: &str = "Title, AllDay, \
     StartDate month, StartDate day, StartDate year, \
     StartTime hour, StartTime minute, StartTime am or pm, \
     EndDate month, EndDate day, EndDate year, \
     EndTime hour, EndTime minute, EndTime am or pm, \
     Location, Attendees, AlertMinutes, ShowAs, Status, Category, Notes, \
     find when everyone is free, the answer, times offered, put it in, Save";

/// The task form's Tab order, read the same day at the same commit.
const THE_TASK_FORMS_TAB_ORDER: &str =
    "Title, DueDate month, DueDate day, DueDate year, Priority, Notes, Save";

#[test]
fn test_tab_walks_the_event_form_in_the_order_it_did_before_it_scrolled() {
    assert_eq!(
        reading("event: Tab from the first field to Save"),
        THE_EVENT_FORMS_TAB_ORDER
    );
}

#[test]
fn test_tab_walks_the_task_form_in_the_order_it_did_before_it_scrolled() {
    assert_eq!(
        reading("task: Tab from the first field to Save"),
        THE_TASK_FORMS_TAB_ORDER
    );
}

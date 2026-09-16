//! One arrow key on the Settings tab row raises one focus event.
//!
//! The tester's words on 2026-09-15 were "Arrowing left/right on a tab list in
//! the settings dialog frequently reads the focused tab twice" (#33). Nothing
//! in `wx_settings.rs` announces a page change, so the readings come from what
//! the control raises. `scripts/uia-events.ps1` captured that on 2026-09-16
//! against the release build at `96298371`, on the channel NVDA reads for a
//! native tab control: per key, `EVENT_OBJECT_SELECTION` once on the reached
//! tab, then `EVENT_OBJECT_FOCUS` twice on that same tab, one millisecond
//! apart, from the control's own arrow-key handler in comctl32. Moving the
//! selection through `TCM_SETCURSEL` instead raised the focus event once. So
//! the dialog takes Left, Right, Up and Down on the tab row itself and moves
//! the selection through wxWidgets' `SetSelection`, which goes that way, and
//! the control's own handler never runs.
//!
//! Asked of the built dialog rather than read from the source, and asked the
//! way the capture asked it: a `WM_KEYDOWN` is sent to the tab control's own
//! window and an in-context win-event hook counts what the control raises
//! while the message is handled. That is a Windows-only question, and this
//! whole file is `cfg(windows)` for it. wxdragon exposes no way to raise a
//! widget event from outside, which `tests/every_event_has_a_control.rs`
//! records as its load-bearing limit; a message sent to the window's own
//! handle goes past wxdragon to the same window procedure a real key reaches,
//! so this is one way round that limit for a native control.
//!
//! One `#[test]` function building real windows, for the reason
//! `tests/theme_reach.rs` gives: wxWidgets supports one application per
//! process. The companion that proves the counting can see two events shares
//! it, and the two readings of the pure function stand on their own.

#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use wixen_mail::data::config::AppConfig;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::wx_settings::{self, Along};
use wxdragon::prelude::*;

/// One check that failed: what it was, and what was wrong with it.
type Wrong = Vec<(String, String)>;

const EVENT_OBJECT_FOCUS: u32 = 0x8005;
const EVENT_OBJECT_SELECTION: u32 = 0x8006;
const OBJID_CLIENT: i32 = -4;
const WINEVENT_INCONTEXT: u32 = 0x0004;
const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const VK_LEFT: usize = 0x25;
const VK_RIGHT: usize = 0x27;

#[link(name = "user32")]
unsafe extern "system" {
    fn SetWinEventHook(
        min: u32,
        max: u32,
        module: *mut c_void,
        callback: extern "system" fn(*mut c_void, u32, isize, i32, i32, u32, u32),
        process: u32,
        thread: u32,
        flags: u32,
    ) -> *mut c_void;
    fn UnhookWinEvent(hook: *mut c_void) -> i32;
    fn NotifyWinEvent(event: u32, hwnd: isize, id_object: i32, id_child: i32);
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
    fn GetCurrentThreadId() -> u32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcessId() -> u32;
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
    fn GetLastError() -> u32;
}

/// One event the control raised: which kind, on which window, about which child.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Raised {
    event: u32,
    hwnd: isize,
    id_child: i32,
}

thread_local! {
    static RAISED: RefCell<Vec<Raised>> = const { RefCell::new(Vec::new()) };
}

extern "system" fn record(
    _hook: *mut c_void,
    event: u32,
    hwnd: isize,
    id_object: i32,
    id_child: i32,
    _thread: u32,
    _time: u32,
) {
    if id_object != OBJID_CLIENT {
        return;
    }
    RAISED.with(|raised| {
        raised.borrow_mut().push(Raised {
            event,
            hwnd,
            id_child,
        })
    });
}

/// The focus and selection events `hwnd` raised while `act` ran, in order.
///
/// An in-context hook on this thread alone: `NotifyWinEvent` calls it before
/// returning, so everything the control raised inside `act` is in the list by
/// the time `act` returns, and nothing another window raised is. In context,
/// the hook wants the module holding the callback, which for a test binary
/// is the executable itself.
fn events_raised_by(hwnd: isize, act: impl FnOnce()) -> Vec<Raised> {
    RAISED.with(|raised| raised.borrow_mut().clear());
    // SAFETY: the callback is a plain function that touches only this thread's
    // local, and the hook is removed before this function returns.
    let hook = unsafe {
        SetWinEventHook(
            EVENT_OBJECT_FOCUS,
            EVENT_OBJECT_SELECTION,
            GetModuleHandleW(std::ptr::null()),
            record,
            GetCurrentProcessId(),
            GetCurrentThreadId(),
            WINEVENT_INCONTEXT,
        )
    };
    // SAFETY: reading the calling thread's last error.
    let last_error = unsafe { GetLastError() };
    assert!(
        !hook.is_null(),
        "the win-event hook could not be set (error {last_error})"
    );
    act();
    // SAFETY: `hook` came from the call above and has not been unhooked.
    unsafe { UnhookWinEvent(hook) };
    RAISED.with(|raised| {
        raised
            .borrow()
            .iter()
            .copied()
            .filter(|one| one.hwnd == hwnd)
            .collect()
    })
}

/// A key pressed and released on the window, the way the message loop hands
/// a real key to it: the repeat count of one on the down, the transition bit
/// on the up.
fn press(hwnd: isize, key: usize) {
    // SAFETY: `hwnd` is a live window on this thread, built by the caller.
    unsafe {
        SendMessageW(hwnd, WM_KEYDOWN, key, 1);
        SendMessageW(hwnd, WM_KEYUP, key, 0xC000_0001_u32 as i32 as isize);
    }
}

/// What the control raised about the reached tab, said as counts, and a
/// complaint when it is not one selection and one focus on that tab.
fn one_reading_of(raised: &[Raised], reached: i32) -> Result<(), String> {
    let count = |event| {
        raised
            .iter()
            .filter(|one| one.event == event && one.id_child == reached)
            .count()
    };
    let elsewhere = raised.iter().filter(|one| one.id_child != reached).count();
    let focus = count(EVENT_OBJECT_FOCUS);
    let selection = count(EVENT_OBJECT_SELECTION);
    if focus == 1 && selection == 1 && elsewhere == 0 {
        return Ok(());
    }
    Err(format!(
        "tab {reached}: EVENT_OBJECT_SELECTION x{selection}, EVENT_OBJECT_FOCUS x{focus}, \
         {elsewhere} about another tab; wanted one of each and nothing else"
    ))
}

#[test]
fn test_one_arrow_on_the_settings_tab_row_raises_one_focus_event() {
    let wrong: Arc<Mutex<Wrong>> = Arc::new(Mutex::new(Vec::new()));
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |app| {
            let mut wrong = wrong.lock().unwrap();
            let frame = Frame::builder().build();
            let a11y = Arc::new(Accessibility::new().expect("accessibility"));
            let settings = wx_settings::build_settings_dialog(
                &frame,
                &AppConfig::default(),
                &[],
                false,
                &a11y,
            );
            let tab_row = settings.notebook.get_handle() as isize;
            assert!(tab_row != 0, "the notebook has no window handle");
            // The row is given focus the way it has it when somebody arrows
            // along it: the control raises its focus events for the focused
            // item, and the capture was of a row that had focus.
            settings.dialog.show(true);
            settings.notebook.set_focus();

            // The reading: Right from the first tab, then Left back, each a
            // real key on the control's own window. Tab items are numbered
            // from one on this channel.
            let raised = events_raised_by(tab_row, || press(tab_row, VK_RIGHT));
            if settings.notebook.selection() != 1 {
                wrong.push((
                    "Right from the first tab".to_string(),
                    format!(
                        "the selection is {} rather than 1",
                        settings.notebook.selection()
                    ),
                ));
            }
            if let Err(why) = one_reading_of(&raised, 2) {
                wrong.push(("what Right raised".to_string(), why));
            }
            let raised = events_raised_by(tab_row, || press(tab_row, VK_LEFT));
            if settings.notebook.selection() != 0 {
                wrong.push((
                    "Left back to the first tab".to_string(),
                    format!(
                        "the selection is {} rather than 0",
                        settings.notebook.selection()
                    ),
                ));
            }
            if let Err(why) = one_reading_of(&raised, 1) {
                wrong.push(("what Left raised".to_string(), why));
            }

            // Left at the first tab and Right at the last stay where they are,
            // as the native control does, and raise nothing about a tab that
            // was not reached.
            let raised = events_raised_by(tab_row, || press(tab_row, VK_LEFT));
            if settings.notebook.selection() != 0 || !raised.is_empty() {
                wrong.push((
                    "Left at the first tab".to_string(),
                    format!(
                        "the selection is {} and the control raised {raised:?}; wanted 0 and nothing",
                        settings.notebook.selection()
                    ),
                ));
            }

            // The companion: two focus events raised by hand on the same tab
            // are two to the reading, so a duplicate the control raised is
            // one it can see.
            let planted = events_raised_by(tab_row, || {
                // SAFETY: `tab_row` is the live control built above.
                unsafe {
                    NotifyWinEvent(EVENT_OBJECT_SELECTION, tab_row, OBJID_CLIENT, 3);
                    NotifyWinEvent(EVENT_OBJECT_FOCUS, tab_row, OBJID_CLIENT, 3);
                    NotifyWinEvent(EVENT_OBJECT_FOCUS, tab_row, OBJID_CLIENT, 3);
                }
            });
            match one_reading_of(&planted, 3) {
                Err(why) if why.contains("EVENT_OBJECT_FOCUS x2") => {}
                other => wrong.push((
                    "the companion for a focus event raised twice".to_string(),
                    format!("the reading answered {other:?} for two planted focus events"),
                )),
            }

            settings.dialog.destroy();
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
        "{} thing(s) wrong on the Settings tab row:\n{}",
        wrong.len(),
        wrong
            .iter()
            .map(|(what, why)| format!("  {what}: {why}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn test_the_tab_an_arrow_reaches_worked_examples() {
    let reaches = wx_settings::the_tab_an_arrow_reaches;
    assert_eq!(reaches(0, 7, Along::Forward), Some(1));
    assert_eq!(reaches(5, 7, Along::Forward), Some(6));
    assert_eq!(reaches(6, 7, Along::Forward), None, "the row does not wrap");
    assert_eq!(reaches(3, 7, Along::Back), Some(2));
    assert_eq!(reaches(0, 7, Along::Back), None, "nor at the front");
    assert_eq!(reaches(0, 0, Along::Forward), None, "a row with no tabs");
}

#[test]
fn test_an_arrow_is_read_from_its_key_code() {
    // wxWidgets' codes: WXK_LEFT 314, WXK_UP 315, WXK_RIGHT 316, WXK_DOWN 317.
    // Left and Up move back, Right and Down forward, as the native control
    // has it.
    assert_eq!(Along::from_key_code(314), Some(Along::Back));
    assert_eq!(Along::from_key_code(315), Some(Along::Back));
    assert_eq!(Along::from_key_code(316), Some(Along::Forward));
    assert_eq!(Along::from_key_code(317), Some(Along::Forward));
    assert_eq!(Along::from_key_code(313), None, "Home is not an arrow");
    assert_eq!(Along::from_key_code(9), None, "nor is Tab");
}

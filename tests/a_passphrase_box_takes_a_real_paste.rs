//! The passphrase box takes a real paste (GAP-03, #49, 13-17.1, 13-44.6.1).
//!
//! WCAG 3.3.8 asks that a secret can come from a password manager rather than
//! from memory, and the way a field gets that wrong is by refusing a paste. So
//! the real dialog is built with `wx_passphrase::build`, text is put on a
//! clipboard and `WM_PASTE` sent to its field, the message Ctrl+V and a
//! password manager both end in, and the field's text is read back. A
//! companion builds a field that refuses a paste and is caught by the same
//! reading.
//!
//! **A target of its own, because Windows refuses the clipboard to every
//! program while the session is locked.** Until 13-44.6.1 this paste ran in
//! `tests/a_locked_key_asks_for_its_passphrase.rs`, in front of four readings
//! of the dialog that never needed a clipboard, and the gate runs that target
//! on every commit touching `src/presentation/wx_app.rs`. One refused
//! clipboard failed six cases and held commits for hours (ledger 716). Here it
//! is coupled to `src/presentation/wx_passphrase.rs` alone, the dialog it
//! proves, and it still runs at the phase's full gate and on CI.
//!
//! **The clipboard is the logon's test clipboard, not one of this process's
//! own.** 13-17.1 said this process made "a clipboard of its own". It makes a
//! window station with no name, and Windows names such a station from the
//! logon, so every test process of one Windows logon that makes one gets the
//! same station and the same clipboard (measured 2026-10-02, 13-44.6.1). The
//! clipboard of whoever runs the tests, on their own station, is still never
//! read or written, and no window here reaches their screen.

#![cfg(windows)]

use std::ffi::c_void;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::presentation::wx_passphrase::build;
use wxdragon::prelude::*;

const WM_PASTE: u32 = 0x0302;
const CF_UNICODETEXT: u32 = 13;
const GMEM_MOVEABLE: u32 = 0x0002;
const WINSTA_ALL_ACCESS: u32 = 0x037F;
const GENERIC_ALL: u32 = 0x1000_0000;

/// What the paste puts on the clipboard: long, with spaces, the shape a
/// password manager's generated passphrase has.
const PASTED: &str = "correct horse battery staple, pasted";

const WHOSE: &str = "Ada Lovelace <ada@example.com>";

#[link(name = "user32")]
unsafe extern "system" {
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
    fn OpenClipboard(owner: isize) -> i32;
    fn EmptyClipboard() -> i32;
    fn SetClipboardData(format: u32, memory: *mut c_void) -> *mut c_void;
    fn CloseClipboard() -> i32;
    fn CreateWindowStationW(
        name: *const u16,
        flags: u32,
        access: u32,
        attributes: *const c_void,
    ) -> isize;
    fn SetProcessWindowStation(station: isize) -> i32;
    fn CreateDesktopW(
        name: *const u16,
        device: *const u16,
        mode: *const c_void,
        flags: u32,
        access: u32,
        attributes: *const c_void,
    ) -> isize;
    fn SetThreadDesktop(desktop: isize) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GlobalAlloc(flags: u32, bytes: usize) -> *mut c_void;
    fn GlobalLock(memory: *mut c_void) -> *mut c_void;
    fn GlobalUnlock(memory: *mut c_void) -> i32;
    fn GetLastError() -> u32;
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Move this thread onto a desktop in a window station that is not the one
/// of whoever runs the tests, so the clipboard it writes is not theirs and no
/// window it makes reaches their screen. Hands back the desktop, so another
/// thread can join it.
///
/// Must run before the first window of the process is made, since a thread
/// with a window cannot change desktop.
fn a_desktop_of_its_own() -> Result<isize, String> {
    // SAFETY: every argument is a valid null-terminated string or null.
    unsafe {
        let station =
            CreateWindowStationW(std::ptr::null(), 0, WINSTA_ALL_ACCESS, std::ptr::null());
        if station == 0 {
            return Err(format!("CreateWindowStationW failed: {}", GetLastError()));
        }
        if SetProcessWindowStation(station) == 0 {
            return Err(format!(
                "SetProcessWindowStation failed: {}",
                GetLastError()
            ));
        }
        let desktop = CreateDesktopW(
            wide("wixen-passphrase-paste-test").as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            0,
            GENERIC_ALL,
            std::ptr::null(),
        );
        if desktop == 0 {
            return Err(format!("CreateDesktopW failed: {}", GetLastError()));
        }
        if SetThreadDesktop(desktop) == 0 {
            return Err(format!("SetThreadDesktop failed: {}", GetLastError()));
        }
        Ok(desktop)
    }
}

/// Put `text` on this window station's clipboard, owned by `owner`.
fn put_on_the_clipboard(owner: isize, text: &str) -> Result<(), String> {
    let units = wide(text);
    // SAFETY: the memory is sized for the text and its terminator, written
    // while locked, and handed to the clipboard, which owns it from then on.
    unsafe {
        if OpenClipboard(owner) == 0 {
            return Err("OpenClipboard failed".to_string());
        }
        EmptyClipboard();
        let memory = GlobalAlloc(GMEM_MOVEABLE, units.len() * 2);
        let placed = !memory.is_null() && {
            let at = GlobalLock(memory) as *mut u16;
            if !at.is_null() {
                std::ptr::copy_nonoverlapping(units.as_ptr(), at, units.len());
                GlobalUnlock(memory);
            }
            !at.is_null() && !SetClipboardData(CF_UNICODETEXT, memory).is_null()
        };
        CloseClipboard();
        if placed {
            Ok(())
        } else {
            Err("the text could not be put on the clipboard".to_string())
        }
    }
}

/// What a paste from the clipboard puts in a field: `PASTED` put on the
/// clipboard, `WM_PASTE` sent, and the field's text read back.
fn what_a_paste_puts_in(field: &TextCtrl) -> Result<String, String> {
    let handle = field.get_handle() as isize;
    put_on_the_clipboard(handle, PASTED)?;
    // SAFETY: a live edit control on this thread; WM_PASTE takes no pointers.
    unsafe { SendMessageW(handle, WM_PASTE, 0, 0) };
    Ok(field.get_value())
}

/// Everything read out of the window session, as plain values.
#[derive(Debug)]
struct Harvest {
    pasted: String,
    ok: Option<String>,
    cancel: Option<String>,
    refused_a_paste: String,
}

fn take_the_harvest() -> Result<Harvest, String> {
    a_desktop_of_its_own()?;
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let taken: Result<Harvest, String> = (|| {
                let frame = Frame::builder().build();

                let asking = build(&frame, WHOSE, None);
                asking.dialog.show(true);
                let pasted = what_a_paste_puts_in(&asking.field)?;
                let ok = asking.answer(ID_OK);
                let cancel = asking.answer(ID_CANCEL);

                // The companion: a field that refuses a paste, read the same way.
                let refusing = TextCtrl::builder(&asking.dialog)
                    .with_style(TextCtrlStyle::Password | TextCtrlStyle::ReadOnly)
                    .build();
                let refused_a_paste = what_a_paste_puts_in(&refusing)?;
                asking.dialog.destroy();

                Ok(Harvest {
                    pasted,
                    ok,
                    cancel,
                    refused_a_paste,
                })
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

#[test]
fn test_a_pasted_passphrase_is_taken_and_ok_hands_it_back() {
    let harvest = the_harvest();

    assert_eq!(harvest.pasted, PASTED, "the field refused a paste");
    assert_eq!(harvest.ok.as_deref(), Some(PASTED));
    assert_eq!(harvest.cancel, None, "Cancel handed back what was typed");
}

#[test]
fn test_the_paste_reading_sees_a_field_that_refuses_one() {
    // The companion. A reading that always found the text would pass the
    // case above whatever the field did with a paste.
    assert_eq!(the_harvest().refused_a_paste, "");
}

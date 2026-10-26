//! A question's two buttons say what each does, where the keyboard lands, read
//! the way a screen reader reads them, and every way of answering one gives
//! the answer it should.
//!
//! Two processes. The window process, an ignored test of this binary, opens
//! Delete Saved Search through `asking::show` with no owner and prints what it
//! answered; it runs no wxDragon and no event loop of its own, only the one
//! the task dialog runs while it is up. The test itself is the reader, a
//! process of its own as a screen reader is: it finds the box by its title
//! among the windows of the process it started, reads each button on its own
//! window over MSAA (`AccessibleObjectFromWindow`, the client object's name)
//! and over UI Automation (`ElementFromHandle`, its name), reads which window
//! has the keyboard when the box opens, and then answers it with the box's
//! own messages rather than with keystrokes from the desktop, which need the
//! foreground: a posted Enter or Escape to the button the keyboard is on,
//! `WM_SYSCOMMAND` with `SC_CLOSE` for Alt+F4, and `TDM_CLICK_BUTTON` for the
//! first button. None of these ever holds the foreground.
//!
//! Both sit on a desktop made for this run, named with the reader's process
//! id, as every target here that sends a key does: a box on the interactive
//! desktop is put in front of whoever uses the machine and read by their
//! screen reader, and takes the keys they type. The reader makes the desktop
//! and moves its own thread onto it, since window messages pass only between
//! threads on one desktop, and hands its name to the window process, which
//! moves its thread there before the box opens and clears the modifier keys
//! the box will read Enter and Escape with.
//!
//! The whole of one reading is bounded at 60 seconds, and the window process
//! is stopped by its id if it outlives it. A key or command meant to close the
//! box is given five seconds; if the box is still up after them the reader
//! answers it with the second button and says the attempt left it open, so a
//! red case fails at once with a sentence rather than at the bound.
//!
//! The box's "Don't ask me again" (13.4-12.4) is not a window of its own but a
//! part of the dialog's `DirectUIHWND`. The reader gives it the keyboard with
//! the task dialog's own `TDM_CLICK_VERIFICATION`, unticked, and reads it in
//! the window Windows then says has the keyboard: on MSAA as the check button
//! in that window's tree, with its name and its state, unticked and focused,
//! and on UI Automation as the element there that has the keyboard, with its
//! name and control type. Where a case ticks it, the reader does so with the
//! same message before it answers, and the window process prints whether
//! `asking::show` saw it ticked beside the answer.

#![cfg(windows)]

use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, EnumWindows, GUITHREADINFO, GWL_STYLE, GetClassNameW, GetGUIThreadInfo,
    GetWindowLongPtrW, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, IsWindow,
    IsWindowVisible, PostMessageW, SC_CLOSE, WM_KEYDOWN, WM_KEYUP, WM_SYSCOMMAND,
};
use wixen_mail::application::questions_before_acting::Question;
use wixen_mail::presentation::asking;

const TITLE: &str = "Delete Saved Search";
const WORDS: &str = "Delete the saved search Weekly? The mail it listed stays where it is.";

/// The environment variable that tells the ignored test it is the window
/// process, and its name.
const BE_THE_WINDOW_PROCESS: &str = "WIXEN_A_QUESTION_WINDOW_PROCESS";
const THE_WINDOW_PROCESS: &str = "the_window_process_asks_one_question";
/// The line the window process prints with what the box answered.
const ANSWERED: &str = "ANSWERED\t";

/// `BS_DEFPUSHBUTTON`, the style of the button Enter presses (winuser.h).
const BS_DEFPUSHBUTTON: isize = 0x0000_0001;
const BS_TYPEMASK: isize = 0x0000_000F;
/// The task dialog's message that presses one of its buttons (commctrl.h).
const TDM_CLICK_BUTTON: u32 = 0x0400 + 102;
/// The task dialog's message that ticks or clears its check box: `wParam`
/// says which, `lParam` whether it takes the keyboard (commctrl.h).
const TDM_CLICK_VERIFICATION: u32 = 0x0400 + 113;
/// What MSAA answers for a check box's role, and the state bits for ticked
/// and for having the keyboard.
const ROLE_SYSTEM_CHECKBUTTON: i64 = 0x2c;
const STATE_SYSTEM_CHECKED: i64 = 0x10;
const STATE_SYSTEM_FOCUSED: i64 = 0x4;
/// What UI Automation answers for a check box's control type.
const UIA_CHECKBOX_CONTROL_TYPE: i32 = 50002;
const VK_RETURN: usize = 0x0D;
const VK_ESCAPE: usize = 0x1B;

// ── A desktop made for the run ─────────────────────────────────────────────

const GENERIC_ALL: u32 = 0x1000_0000;
/// winuser.h: Shift, Control and Alt, each with its left and right key.
const MODIFIERS: [usize; 9] = [0x10, 0x11, 0x12, 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5];

#[link(name = "user32")]
unsafe extern "system" {
    fn CreateDesktopW(
        name: *const u16,
        device: *const u16,
        mode: *const c_void,
        flags: u32,
        access: u32,
        attributes: *const c_void,
    ) -> isize;
    fn OpenDesktopW(name: *const u16, flags: u32, inherit: i32, access: u32) -> isize;
    fn SetThreadDesktop(desktop: isize) -> i32;
    fn GetKeyboardState(state: *mut u8) -> i32;
    fn SetKeyboardState(state: *const u8) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetLastError() -> u32;
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain([0]).collect()
}

/// The name of the desktop this run's boxes open on.
fn the_runs_desktop_name() -> String {
    format!("wixen-question-{}", std::process::id())
}

/// The desktop this run's boxes open on, made once and open for the life of
/// the process, since windows live on it.
fn the_runs_desktop() -> Result<isize, String> {
    static MADE: std::sync::OnceLock<Result<isize, String>> = std::sync::OnceLock::new();
    MADE.get_or_init(|| {
        // SAFETY: the name is null-terminated and every other pointer is
        // null, which CreateDesktopW takes as "none".
        let desktop = unsafe {
            CreateDesktopW(
                wide(&the_runs_desktop_name()).as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                GENERIC_ALL,
                std::ptr::null(),
            )
        };
        match desktop {
            // SAFETY: reads this thread's last error and nothing else.
            0 => Err(format!("CreateDesktopW failed: {}", unsafe {
                GetLastError()
            })),
            made => Ok(made),
        }
    })
    .clone()
}

/// Move the calling thread onto `desktop`, before it has any window.
fn move_this_thread_to(desktop: isize) -> Result<(), String> {
    // SAFETY: a desktop handle this process holds open.
    match unsafe { SetThreadDesktop(desktop) } {
        // SAFETY: reads this thread's last error and nothing else.
        0 => Err(format!("SetThreadDesktop failed: {}", unsafe {
            GetLastError()
        })),
        _ => Ok(()),
    }
}

/// Set this thread's keyboard state with exactly `down` of the modifier keys
/// held and every other key as it was, before the box that reads Enter and
/// Escape with it opens. A desktop of the run's own keeps out the keys a
/// person presses but not the ones they hold: measured 2026-10-02, every one
/// of eight failed runs of two key targets came while the person at the
/// machine held Shift.
fn only_these_modifiers_down(down: &[usize]) {
    let mut held = [0u8; 256];
    // SAFETY: the buffer is the 256 bytes the call writes.
    unsafe { GetKeyboardState(held.as_mut_ptr()) };
    for key in MODIFIERS {
        held[key] = 0;
    }
    for &key in down {
        held[key] = 0x80;
    }
    // SAFETY: the buffer is the 256 bytes the call reads, for this thread.
    unsafe { SetKeyboardState(held.as_ptr()) };
}

// ── MSAA, by hand, as `tests/a_spin_controls_field_is_named_where_the_scan_reads_it.rs` reads it

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const VT_I4: u16 = 3;
const VTBL_RELEASE: usize = 2;
const VTBL_GET_ACC_NAME: usize = 10;
const VTBL_GET_ACC_ROLE: usize = 13;
const VTBL_GET_ACC_STATE: usize = 14;
const VTBL_GET_ACC_CHILD_COUNT: usize = 8;
const VTBL_QUERY_INTERFACE: usize = 0;
const VT_DISPATCH: u16 = 9;

#[repr(C)]
#[derive(Clone, Copy)]
struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

/// {618736E0-3C3D-11CF-810C-00AA00389B71}
const IID_IACCESSIBLE: Guid = Guid {
    data1: 0x618736E0,
    data2: 0x3C3D,
    data3: 0x11CF,
    data4: [0x81, 0x0C, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71],
};

/// A VARIANT as the 64-bit ABI lays it out, holding `CHILDID_SELF`.
#[repr(C)]
#[derive(Clone, Copy)]
struct Variant {
    vt: u16,
    reserved: [u16; 3],
    val: i64,
    extra: u64,
}

const CHILD_SELF: Variant = Variant {
    vt: VT_I4,
    reserved: [0; 3],
    val: 0,
    extra: 0,
};

type GetBstrFn = unsafe extern "system" fn(*mut c_void, Variant, *mut *mut u16) -> i32;
type GetVariantFn = unsafe extern "system" fn(*mut c_void, Variant, *mut Variant) -> i32;
type GetCountFn = unsafe extern "system" fn(*mut c_void, *mut i32) -> i32;
type QueryInterfaceFn =
    unsafe extern "system" fn(*mut c_void, *const Guid, *mut *mut c_void) -> i32;
type ReleaseFn = unsafe extern "system" fn(*mut c_void) -> u32;

#[link(name = "oleacc")]
unsafe extern "system" {
    fn AccessibleObjectFromWindow(
        hwnd: isize,
        id_object: u32,
        riid: *const Guid,
        out: *mut *mut c_void,
    ) -> i32;
    fn AccessibleChildren(
        container: *mut c_void,
        start: i32,
        count: i32,
        children: *mut Variant,
        obtained: *mut i32,
    ) -> i32;
}

#[link(name = "oleaut32")]
unsafe extern "system" {
    fn SysStringLen(s: *mut u16) -> u32;
    fn SysFreeString(s: *mut u16);
}

unsafe fn vtable_entry(object: *mut c_void, index: usize) -> *const c_void {
    // SAFETY: a COM object is a pointer to its vtable.
    unsafe { *(*(object as *const *const *const c_void)).add(index) }
}

/// The MSAA name of `window`'s own client object.
fn msaa_name_of(window: HWND) -> String {
    let mut object: *mut c_void = std::ptr::null_mut();
    // SAFETY: the out pointer is a local; on success it holds an IAccessible.
    let hr = unsafe {
        AccessibleObjectFromWindow(
            window.0 as isize,
            OBJID_CLIENT,
            &IID_IACCESSIBLE,
            &mut object,
        )
    };
    if hr < 0 || object.is_null() {
        return format!("(AccessibleObjectFromWindow failed 0x{hr:x})");
    }
    // SAFETY: a live IAccessible; its get_accName and Release slots, the BSTR
    // freed once and the object released once.
    unsafe {
        let get_name: GetBstrFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_NAME));
        let mut name: *mut u16 = std::ptr::null_mut();
        let read = if get_name(object, CHILD_SELF, &mut name) >= 0 && !name.is_null() {
            let text = String::from_utf16_lossy(std::slice::from_raw_parts(
                name,
                SysStringLen(name) as usize,
            ));
            SysFreeString(name);
            text
        } else {
            String::new()
        };
        let release: ReleaseFn = std::mem::transmute(vtable_entry(object, VTBL_RELEASE));
        release(object);
        read
    }
}

const EMPTY: Variant = Variant {
    vt: 0,
    reserved: [0; 3],
    val: 0,
    extra: 0,
};

/// The name, role and state of `child` of the IAccessible `object`.
///
/// # Safety
/// `object` is a live IAccessible.
unsafe fn msaa_name_role_and_state(
    object: *mut c_void,
    child: Variant,
) -> Result<(String, i64, i64), String> {
    // SAFETY: the caller's object; its get_accName, get_accRole and
    // get_accState slots, and the BSTR freed once.
    unsafe {
        let get_name: GetBstrFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_NAME));
        let get_role: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_ROLE));
        let get_state: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_STATE));
        let mut name: *mut u16 = std::ptr::null_mut();
        let name = if get_name(object, child, &mut name) >= 0 && !name.is_null() {
            let text = String::from_utf16_lossy(std::slice::from_raw_parts(
                name,
                SysStringLen(name) as usize,
            ));
            SysFreeString(name);
            text
        } else {
            String::new()
        };
        let (mut role, mut state) = (EMPTY, EMPTY);
        let role_read = get_role(object, child, &mut role);
        let state_read = get_state(object, child, &mut state);
        if role_read < 0 || role.vt != VT_I4 || state_read < 0 || state.vt != VT_I4 {
            return Err(format!(
                "role 0x{role_read:x} vt {}, state 0x{state_read:x} vt {}",
                role.vt, state.vt
            ));
        }
        Ok((name, role.val & 0xFFFF_FFFF, state.val & 0xFFFF_FFFF))
    }
}

/// Every check button in the MSAA tree under `object`, as its name, role and
/// state, `depth` levels down at most.
///
/// # Safety
/// `object` is a live IAccessible.
unsafe fn msaa_check_buttons_under(
    object: *mut c_void,
    depth: usize,
    found: &mut Vec<(String, i64, i64)>,
) {
    // SAFETY: the caller's object; its get_accChildCount slot, the children
    // AccessibleChildren hands out, each object among them queried for
    // IAccessible and released once.
    unsafe {
        let count_children: GetCountFn =
            std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_CHILD_COUNT));
        let mut count = 0i32;
        if count_children(object, &mut count) < 0 || count <= 0 {
            return;
        }
        let mut children = vec![EMPTY; count as usize];
        let mut obtained = 0i32;
        if AccessibleChildren(object, 0, count, children.as_mut_ptr(), &mut obtained) < 0 {
            return;
        }
        for child in children.into_iter().take(obtained.max(0) as usize) {
            match child.vt {
                VT_I4 => {
                    let id = Variant {
                        val: child.val,
                        ..CHILD_SELF
                    };
                    if let Ok(heard) = msaa_name_role_and_state(object, id)
                        && heard.1 == ROLE_SYSTEM_CHECKBUTTON
                    {
                        found.push(heard);
                    }
                }
                VT_DISPATCH => {
                    let dispatch = child.val as *mut c_void;
                    let query: QueryInterfaceFn =
                        std::mem::transmute(vtable_entry(dispatch, VTBL_QUERY_INTERFACE));
                    let mut inner: *mut c_void = std::ptr::null_mut();
                    let queried = query(dispatch, &IID_IACCESSIBLE, &mut inner);
                    let release: ReleaseFn =
                        std::mem::transmute(vtable_entry(dispatch, VTBL_RELEASE));
                    release(dispatch);
                    if queried >= 0 && !inner.is_null() {
                        if let Ok(heard) = msaa_name_role_and_state(inner, CHILD_SELF)
                            && heard.1 == ROLE_SYSTEM_CHECKBUTTON
                        {
                            found.push(heard);
                        }
                        if depth > 0 {
                            msaa_check_buttons_under(inner, depth - 1, found);
                        }
                        let release: ReleaseFn =
                            std::mem::transmute(vtable_entry(inner, VTBL_RELEASE));
                        release(inner);
                    }
                }
                _ => {}
            }
        }
    }
}

/// The check buttons MSAA lists in `window`'s client object, each as its
/// name, role and state. The task dialog's MSAA object answers no focused
/// child from another process (measured 2026-10-25: `get_accFocus` gave an
/// empty VARIANT with the keyboard on the check box), so the check box is
/// found in the tree and its state says whether the keyboard is on it.
fn msaa_check_buttons_in(window: HWND) -> Result<Vec<(String, i64, i64)>, String> {
    let mut object: *mut c_void = std::ptr::null_mut();
    // SAFETY: the out pointer is a local; on success it holds an IAccessible.
    let hr = unsafe {
        AccessibleObjectFromWindow(
            window.0 as isize,
            OBJID_CLIENT,
            &IID_IACCESSIBLE,
            &mut object,
        )
    };
    if hr < 0 || object.is_null() {
        return Err(format!("AccessibleObjectFromWindow failed 0x{hr:x}"));
    }
    let mut found = Vec::new();
    // SAFETY: a live IAccessible, walked and then released once.
    unsafe {
        msaa_check_buttons_under(object, 8, &mut found);
        let release: ReleaseFn = std::mem::transmute(vtable_entry(object, VTBL_RELEASE));
        release(object);
    }
    Ok(found)
}

// ── UI Automation ──────────────────────────────────────────────────────────

fn automation() -> windows::core::Result<windows::Win32::UI::Accessibility::IUIAutomation> {
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
    };
    // SAFETY: joins this test's thread to the multithreaded apartment, which
    // a UI Automation client is meant to call from; a second join answers
    // S_FALSE and changes nothing.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        CoCreateInstance(
            &windows::Win32::UI::Accessibility::CUIAutomation,
            None,
            CLSCTX_INPROC_SERVER,
        )
    }
}

/// What UI Automation answers as the name of the element for `window`. Asked
/// up to three times: a box just shown has answered `E_FAIL` once here and its
/// name a moment later.
fn uia_name_of(window: HWND) -> String {
    let ask = || {
        // SAFETY: a live window handle in the window process.
        automation()
            .and_then(|automation| unsafe { automation.ElementFromHandle(window) })
            .and_then(|element| unsafe { element.CurrentName() })
    };
    let mut answered = ask();
    for _ in 0..2 {
        if answered.is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
        answered = ask();
    }
    answered.map_or_else(|why| format!("({why})"), |name| name.to_string())
}

/// The name and control type UI Automation gives for the element under
/// `window` that has the keyboard. Every element is asked rather than a
/// condition built, because a property condition takes a VARIANT, whose
/// feature the `windows` dependency does not carry.
fn uia_keyboard_focus_under(window: HWND) -> Result<(String, i32), String> {
    use windows::Win32::UI::Accessibility::TreeScope_Descendants;
    let read = || -> windows::core::Result<Option<(String, i32)>> {
        let automation = automation()?;
        // SAFETY: a live window handle; each element is read once.
        unsafe {
            let all = automation
                .ElementFromHandle(window)?
                .FindAll(TreeScope_Descendants, &automation.CreateTrueCondition()?)?;
            for at in 0..all.Length()? {
                let element = all.GetElement(at)?;
                if element.CurrentHasKeyboardFocus()?.as_bool() {
                    return Ok(Some((
                        element.CurrentName()?.to_string(),
                        element.CurrentControlType()?.0,
                    )));
                }
            }
            Ok(None)
        }
    };
    read()
        .map_err(|why| why.to_string())?
        .ok_or_else(|| "no element under the window has the keyboard".to_string())
}

/// Every name on the UI Automation tree under `dialog`.
fn uia_names_under(dialog: HWND) -> Vec<String> {
    use windows::Win32::UI::Accessibility::TreeScope_Descendants;
    let read = || -> windows::core::Result<Vec<String>> {
        let automation = automation()?;
        // SAFETY: a live window handle; each element is read once.
        unsafe {
            let all = automation
                .ElementFromHandle(dialog)?
                .FindAll(TreeScope_Descendants, &automation.CreateTrueCondition()?)?;
            (0..all.Length()?)
                .map(|at| Ok(all.GetElement(at)?.CurrentName()?.to_string()))
                .collect()
        }
    };
    read().unwrap_or_else(|why| vec![format!("({why})")])
}

// ── Windows ────────────────────────────────────────────────────────────────

fn class_of(window: HWND) -> String {
    let mut buffer = [0u16; 256];
    // SAFETY: the buffer is as long as the call is told.
    let len = unsafe { GetClassNameW(window, &mut buffer) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

fn title_of(window: HWND) -> String {
    let mut buffer = [0u16; 256];
    // SAFETY: as above.
    let len = unsafe { GetWindowTextW(window, &mut buffer) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

unsafe extern "system" fn collect(window: HWND, found: LPARAM) -> windows::core::BOOL {
    // SAFETY: `found` is the vector handed to the enumeration, alive for it.
    unsafe { (*(found.0 as *mut Vec<HWND>)).push(window) };
    true.into()
}

fn top_windows() -> Vec<HWND> {
    let mut found: Vec<HWND> = Vec::new();
    // SAFETY: the callback writes only to `found`, which outlives the call.
    let _ = unsafe { EnumWindows(Some(collect), LPARAM(&mut found as *mut _ as isize)) };
    found
}

fn descendants_of(parent: HWND) -> Vec<HWND> {
    let mut found: Vec<HWND> = Vec::new();
    // SAFETY: as above.
    let _ = unsafe {
        EnumChildWindows(
            Some(parent),
            Some(collect),
            LPARAM(&mut found as *mut _ as isize),
        )
    };
    found
}

fn process_of(window: HWND) -> u32 {
    let mut process = 0u32;
    // SAFETY: the out pointer is a local.
    unsafe { GetWindowThreadProcessId(window, Some(&mut process)) };
    process
}

/// The question's box among the shown windows of `process`.
fn the_box_of(process: u32) -> Option<HWND> {
    top_windows().into_iter().find(|&window| {
        process_of(window) == process
            // SAFETY: a window handle Windows just listed.
            && unsafe { IsWindowVisible(window) }.as_bool()
            && class_of(window) == "#32770"
            && title_of(window) == TITLE
    })
}

/// The box's buttons, left to right.
fn the_buttons_of(dialog: HWND) -> Vec<HWND> {
    let mut buttons: Vec<(i32, HWND)> = descendants_of(dialog)
        .into_iter()
        .filter(|&window| class_of(window) == "Button")
        .map(|window| {
            let mut placed = RECT::default();
            // SAFETY: a live window and a local rectangle.
            let _ = unsafe { GetWindowRect(window, &mut placed) };
            (placed.left, window)
        })
        .collect();
    buttons.sort_by_key(|(left, _)| *left);
    buttons.into_iter().map(|(_, window)| window).collect()
}

/// The window that has the keyboard on the thread that owns `dialog`.
fn the_keyboard_is_on(dialog: HWND) -> Option<HWND> {
    // SAFETY: a live window handle.
    let thread = unsafe { GetWindowThreadProcessId(dialog, None) };
    let mut info = GUITHREADINFO {
        cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: the structure is sized as the call requires.
    unsafe { GetGUIThreadInfo(thread, &mut info) }.ok()?;
    (!info.hwndFocus.is_invalid()).then_some(info.hwndFocus)
}

fn is_the_default_button(window: HWND) -> bool {
    // SAFETY: a live window handle; only its style is read.
    let style = unsafe { GetWindowLongPtrW(window, GWL_STYLE) };
    style & BS_TYPEMASK == BS_DEFPUSHBUTTON
}

fn post(window: HWND, message: u32, wparam: usize, lparam: isize) -> Result<(), String> {
    // SAFETY: a window of another process is handed a message by value.
    unsafe { PostMessageW(Some(window), message, WPARAM(wparam), LPARAM(lparam)) }
        .map_err(|why| format!("PostMessageW 0x{message:x} failed: {why}"))
}

/// A key pressed and let go on `window`, by its own messages.
fn press(window: HWND, key: usize) -> Result<(), String> {
    post(window, WM_KEYDOWN, key, 1)?;
    post(window, WM_KEYUP, key, 0xC000_0001u32 as isize)
}

// ── The window process ─────────────────────────────────────────────────────

#[test]
#[ignore = "started by the tests below, in a process of its own"]
fn the_window_process_asks_one_question() {
    let Ok(desktop) = std::env::var(BE_THE_WINDOW_PROCESS) else {
        return;
    };
    // SAFETY: the name is null-terminated; the handle stays open for the life
    // of the process, since the box lives on it.
    let opened = unsafe { OpenDesktopW(wide(&desktop).as_ptr(), 0, 0, GENERIC_ALL) };
    let moved = match opened {
        // SAFETY: reads this thread's last error and nothing else.
        0 => Err(format!("OpenDesktopW {desktop} failed: {}", unsafe {
            GetLastError()
        })),
        opened => move_this_thread_to(opened),
    };
    if let Err(why) = moved {
        println!("{ANSWERED}could not reach the run's desktop: {why}");
        return;
    }
    only_these_modifiers_down(&[]);
    let the_box = asking::the_box(Question::DeleteASavedSearch, TITLE, WORDS);
    let asking::Shown { answered, ticked } = asking::show(&the_box, None);
    println!("{ANSWERED}{answered:?}\t{ticked}");
}

// ── The reading ────────────────────────────────────────────────────────────

/// How the reader answers the box once it has read it.
#[derive(Debug, Clone, Copy)]
enum Answer {
    Enter,
    Escape,
    AltF4,
    TheFirstButton,
    TheSecondButton,
}

/// What the reader does with "Don't ask me again" before it answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Before {
    /// Nothing: the keyboard stays where the box put it.
    Nothing,
    /// Tick it, with the keyboard left where it was.
    Tick,
    /// Give it the keyboard, unticked, and read it there.
    ReadTheCheckBox,
}

/// The box's check box as the reader heard it once it had the keyboard. It
/// is a part of the dialog's `DirectUIHWND` rather than a window of its own
/// (measured 2026-10-25: the box's only `Button`-class windows are its two
/// buttons), so it is read the way a screen reader finds it there: MSAA's
/// focused child of that window, and the UI Automation element under it
/// that has the keyboard.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CheckBoxHeard {
    /// The class of the window Windows says has the keyboard.
    window: String,
    /// Every check button MSAA lists in that window, with its name, role and
    /// state.
    msaa: Result<Vec<(String, i64, i64)>, String>,
    /// Its UI Automation name and control type.
    uia: Result<(String, i32), String>,
}

/// What one reading found.
#[derive(Debug, Default)]
struct Reading {
    /// Each button left to right, as MSAA and UI Automation name it.
    buttons: Vec<(String, String)>,
    /// The check box, when the reading gave it the keyboard.
    check_box: Option<CheckBoxHeard>,
    /// The name of the button the keyboard was on when the box opened, and
    /// whether it carried the default button's style.
    keyboard: Option<(String, bool)>,
    /// Every name on the UI Automation tree under the box.
    tree: Vec<String>,
    /// Whether the answer left the box up, so the reader closed it.
    left_open: bool,
    /// What the window process printed it answered.
    answered: String,
    /// Whether the window process printed that the check box was ticked.
    ticked: String,
}

/// Start the window process, read its box, answer it with `answer`, and hand
/// back what was found; every failure is a sentence.
fn read_then_answer(answer: Answer, before: Before) -> Result<Reading, String> {
    let deadline = Instant::now() + Duration::from_secs(60);
    move_this_thread_to(the_runs_desktop()?)?;
    let me = std::env::current_exe().map_err(|why| format!("current_exe: {why}"))?;
    let mut child = std::process::Command::new(me)
        .args([
            "--ignored",
            "--exact",
            "--nocapture",
            "--test-threads=1",
            THE_WINDOW_PROCESS,
        ])
        .env(BE_THE_WINDOW_PROCESS, the_runs_desktop_name())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|why| format!("the window process could not start: {why}"))?;
    let printed = Arc::new(Mutex::new(String::new()));
    if let Some(out) = child.stdout.take() {
        let printed = printed.clone();
        std::thread::spawn(move || {
            use std::io::BufRead;
            for line in std::io::BufReader::new(out).lines().map_while(Result::ok) {
                if let Ok(mut printed) = printed.lock() {
                    printed.push_str(&line);
                    printed.push('\n');
                }
            }
        });
    }
    let reading = read_and_answer(child.id(), (before, answer), deadline, &printed);
    if reading.is_err() {
        // Nothing will answer the box now, so the process is stopped by its id
        // rather than left waiting for the bound.
        let _ = child.kill();
    }
    let finished = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if Instant::now() > deadline => {
                let _ = child.kill();
                break Err("the window process was still running after 60 seconds".to_string());
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(why) => break Err(format!("waiting for the window process: {why}")),
        }
    };
    let mut reading = reading?;
    finished?;
    let read_until = Instant::now() + Duration::from_secs(5);
    let answered = loop {
        let so_far = printed.lock().map(|p| p.clone()).unwrap_or_default();
        if let Some(line) = so_far.lines().find_map(|line| line.split_once(ANSWERED)) {
            break Ok(line.1.trim().to_string());
        }
        if Instant::now() > read_until {
            break Err(format!("the window process printed no answer:\n{so_far}"));
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    let answered = answered?;
    let (answered, ticked) = answered.split_once('\t').unwrap_or((&answered, ""));
    reading.answered = answered.to_string();
    reading.ticked = ticked.to_string();
    Ok(reading)
}

/// Find the box of `process`, read it and answer it with `answer`. `printed`
/// is what the window process has printed so far, so one that ended before
/// its box opened is reported with its own words rather than at the bound.
fn read_and_answer(
    process: u32,
    (before, answer): (Before, Answer),
    deadline: Instant,
    printed: &Mutex<String>,
) -> Result<Reading, String> {
    let dialog = loop {
        if let Some(dialog) = the_box_of(process) {
            break dialog;
        }
        let so_far = printed.lock().map(|p| p.clone()).unwrap_or_default();
        if so_far.contains(ANSWERED) || Instant::now() > deadline {
            return Err(format!(
                "no box titled {TITLE} opened in the window process, which printed:\n{so_far}"
            ));
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let buttons = the_buttons_of(dialog);
    if buttons.len() != 2 {
        return Err(format!(
            "the box holds {} windows of the class Button, not two",
            buttons.len()
        ));
    }
    // The keyboard reaches a button a moment after the box is shown.
    let settle = Instant::now() + Duration::from_secs(5);
    let keyboard = loop {
        match the_keyboard_is_on(dialog) {
            Some(on) if buttons.contains(&on) => break Some(on),
            _ if Instant::now() > settle => break None,
            _ => std::thread::sleep(Duration::from_millis(50)),
        }
    };
    let mut reading = Reading {
        buttons: buttons
            .iter()
            .map(|&button| (msaa_name_of(button), uia_name_of(button)))
            .collect(),
        keyboard: keyboard.map(|on| (msaa_name_of(on), is_the_default_button(on))),
        tree: uia_names_under(dialog),
        ..Reading::default()
    };

    match before {
        Before::Nothing => {}
        // Ticked, and the keyboard left where it was.
        Before::Tick => post(dialog, TDM_CLICK_VERIFICATION, 1, 0)?,
        Before::ReadTheCheckBox => {
            // Left unticked, and given the keyboard, the way Tab reaches it.
            post(dialog, TDM_CLICK_VERIFICATION, 0, 1)?;
            let settle = Instant::now() + Duration::from_secs(5);
            let on = loop {
                match the_keyboard_is_on(dialog) {
                    Some(on) if !buttons.contains(&on) => break Some(on),
                    _ if Instant::now() > settle => break None,
                    _ => std::thread::sleep(Duration::from_millis(50)),
                }
            };
            reading.check_box = on.map(|on| CheckBoxHeard {
                window: class_of(on),
                msaa: msaa_check_buttons_in(on),
                uia: uia_keyboard_focus_under(on),
            });
        }
    }
    match answer {
        Answer::Enter => press(keyboard.ok_or("the keyboard is on no button")?, VK_RETURN)?,
        Answer::Escape => press(keyboard.ok_or("the keyboard is on no button")?, VK_ESCAPE)?,
        Answer::AltF4 => post(dialog, WM_SYSCOMMAND, SC_CLOSE as usize, 0)?,
        Answer::TheFirstButton => post(dialog, TDM_CLICK_BUTTON, asking::IDYES as usize, 0)?,
        Answer::TheSecondButton => post(dialog, TDM_CLICK_BUTTON, asking::IDNO as usize, 0)?,
    }
    let gone_by = Instant::now() + Duration::from_secs(5);
    // SAFETY: IsWindow only asks whether the handle still names a window.
    while unsafe { IsWindow(Some(dialog)) }.as_bool() && Instant::now() < gone_by {
        std::thread::sleep(Duration::from_millis(50));
    }
    // SAFETY: as above.
    if unsafe { IsWindow(Some(dialog)) }.as_bool() {
        reading.left_open = true;
        post(dialog, TDM_CLICK_BUTTON, asking::IDNO as usize, 0)?;
    }
    Ok(reading)
}

/// One box up at a time across this file's tests, so no two windows of the
/// readings contend for which of them is active.
static ONE_BOX_AT_A_TIME: Mutex<()> = Mutex::new(());

fn reading_after(answer: Answer) -> Reading {
    reading_after_doing(Before::Nothing, answer)
}

fn reading_after_doing(before: Before, answer: Answer) -> Reading {
    let _turn = ONE_BOX_AT_A_TIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    read_then_answer(answer, before)
        .unwrap_or_else(|why| panic!("{before:?}, then answered by {answer:?}: {why}"))
}

// ── The tests ──────────────────────────────────────────────────────────────

#[test]
fn test_a_question_names_both_answers_on_both_channels_with_the_keyboard_on_keep() {
    let reading = reading_after(Answer::Enter);
    // Printed whatever the verdict, so a run with --nocapture carries what was
    // heard.
    println!("{reading:#?}");
    let named = |name: &str| (name.to_string(), name.to_string());
    assert_eq!(
        reading.buttons,
        vec![named("Delete"), named("Keep")],
        "the buttons left to right, as MSAA and UI Automation name each: {reading:#?}"
    );
    assert_eq!(
        reading.keyboard,
        Some(("Keep".to_string(), true)),
        "the button the keyboard was on when the box opened, and whether it is the default: \
         {reading:#?}"
    );
    assert!(
        reading.tree.iter().any(|name| name.contains(WORDS)),
        "the words are nowhere on the box's UI Automation tree: {reading:#?}"
    );
}

#[test]
fn test_enter_gives_the_answer_the_keyboard_is_on() {
    let reading = reading_after(Answer::Enter);
    assert!(!reading.left_open, "Enter left the box open: {reading:#?}");
    assert_eq!(reading.answered, "No", "{reading:#?}");
}

#[test]
fn test_the_first_button_gives_the_first_answer() {
    let reading = reading_after(Answer::TheFirstButton);
    assert!(
        !reading.left_open,
        "the first button left the box open: {reading:#?}"
    );
    assert_eq!(reading.answered, "Yes", "{reading:#?}");
}

#[test]
fn test_escape_closes_a_question_with_neither_answer() {
    let reading = reading_after(Answer::Escape);
    assert!(!reading.left_open, "Escape left the box open: {reading:#?}");
    assert_eq!(reading.answered, "Neither", "{reading:#?}");
}

#[test]
fn test_alt_f4_closes_a_question_with_neither_answer() {
    let reading = reading_after(Answer::AltF4);
    assert!(!reading.left_open, "Alt+F4 left the box open: {reading:#?}");
    assert_eq!(reading.answered, "Neither", "{reading:#?}");
}

#[test]
fn test_a_question_carries_dont_ask_me_again_unticked_on_both_channels() {
    let reading = reading_after_doing(Before::ReadTheCheckBox, Answer::Escape);
    // Printed whatever the verdict, so a run with --nocapture carries what was
    // heard.
    println!("{:#?}", reading.check_box);
    let Some(check_box) = &reading.check_box else {
        panic!("given the keyboard, the check box was never where it landed: {reading:#?}");
    };
    let state = match &check_box.msaa {
        Ok(found) => found.first().map_or(0, |(_, _, state)| *state),
        Err(_) => 0,
    };
    assert_eq!(
        check_box,
        &CheckBoxHeard {
            window: check_box.window.clone(),
            msaa: Ok(vec![(
                "Don't ask me again".to_string(),
                ROLE_SYSTEM_CHECKBUTTON,
                state
            )]),
            uia: Ok(("Don't ask me again".to_string(), UIA_CHECKBOX_CONTROL_TYPE)),
        },
        "its name and role on MSAA, the box's one check button, and its name and control \
         type on UI Automation where the keyboard lands"
    );
    assert_eq!(
        state & STATE_SYSTEM_CHECKED,
        0,
        "ticked when the box opened"
    );
    assert_ne!(
        state & STATE_SYSTEM_FOCUSED,
        0,
        "MSAA does not say the keyboard is on it"
    );
    assert_eq!(reading.ticked, "false", "{reading:#?}");
}

#[test]
fn test_ticked_and_answered_with_the_first_button_is_yes_and_ticked() {
    let reading = reading_after_doing(Before::Tick, Answer::TheFirstButton);
    assert!(!reading.left_open, "{reading:#?}");
    assert_eq!(
        (reading.answered.as_str(), reading.ticked.as_str()),
        ("Yes", "true"),
        "{reading:#?}"
    );
}

#[test]
fn test_ticked_and_answered_with_the_second_button_is_no_and_ticked() {
    let reading = reading_after_doing(Before::Tick, Answer::TheSecondButton);
    assert!(!reading.left_open, "{reading:#?}");
    assert_eq!(
        (reading.answered.as_str(), reading.ticked.as_str()),
        ("No", "true"),
        "{reading:#?}"
    );
}

#[test]
fn test_ticked_and_closed_with_escape_is_neither_and_ticked() {
    let reading = reading_after_doing(Before::Tick, Answer::Escape);
    assert!(!reading.left_open, "{reading:#?}");
    assert_eq!(
        (reading.answered.as_str(), reading.ticked.as_str()),
        ("Neither", "true"),
        "{reading:#?}"
    );
}

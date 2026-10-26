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
//! The box's "Don't ask me again" (13.4-12.4) is read the same way, at its own
//! window: its name on both channels, its MSAA role and state and its UI
//! Automation control type, unticked when the box opens. Where a case ticks
//! it, the reader does so with the task dialog's own `TDM_CLICK_VERIFICATION`
//! before it answers, and the window process prints whether `asking::show`
//! saw it ticked beside the answer.

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
/// `BS_PUSHBUTTON`, the other push-button style (winuser.h).
const BS_PUSHBUTTON: isize = 0x0000_0000;
/// What MSAA answers for a check box's role, and the state bit for ticked.
const ROLE_SYSTEM_CHECKBUTTON: i64 = 0x2c;
const STATE_SYSTEM_CHECKED: i64 = 0x10;
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
type ReleaseFn = unsafe extern "system" fn(*mut c_void) -> u32;

#[link(name = "oleacc")]
unsafe extern "system" {
    fn AccessibleObjectFromWindow(
        hwnd: isize,
        id_object: u32,
        riid: *const Guid,
        out: *mut *mut c_void,
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

/// The MSAA role and state of `window`'s own client object.
fn msaa_role_and_state_of(window: HWND) -> Result<(i64, i64), String> {
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
    let empty = Variant {
        vt: 0,
        reserved: [0; 3],
        val: 0,
        extra: 0,
    };
    // SAFETY: a live IAccessible; its get_accRole, get_accState and Release
    // slots, the object released once.
    unsafe {
        let get_role: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_ROLE));
        let get_state: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_STATE));
        let (mut role, mut state) = (empty, empty);
        let role_read = get_role(object, CHILD_SELF, &mut role);
        let state_read = get_state(object, CHILD_SELF, &mut state);
        let release: ReleaseFn = std::mem::transmute(vtable_entry(object, VTBL_RELEASE));
        release(object);
        if role_read < 0 || role.vt != VT_I4 || state_read < 0 || state.vt != VT_I4 {
            return Err(format!(
                "role 0x{role_read:x} vt {}, state 0x{state_read:x} vt {}",
                role.vt, state.vt
            ));
        }
        Ok((role.val & 0xFFFF_FFFF, state.val & 0xFFFF_FFFF))
    }
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

/// What UI Automation answers as the control type of the element for
/// `window`.
fn uia_control_type_of(window: HWND) -> Result<i32, String> {
    // SAFETY: a live window handle in the window process.
    automation()
        .and_then(|automation| unsafe { automation.ElementFromHandle(window) })
        .and_then(|element| unsafe { element.CurrentControlType() })
        .map(|control_type| control_type.0)
        .map_err(|why| why.to_string())
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

/// The kind of a `Button`-class window, the low nibble of its style.
fn button_kind_of(window: HWND) -> isize {
    // SAFETY: a live window handle; only its style is read.
    let style = unsafe { GetWindowLongPtrW(window, GWL_STYLE) };
    style & BS_TYPEMASK
}

fn is_a_push_button(window: HWND) -> bool {
    class_of(window) == "Button"
        && matches!(button_kind_of(window), BS_PUSHBUTTON | BS_DEFPUSHBUTTON)
}

/// The box's check boxes: its `Button`-class windows that are not push
/// buttons.
fn the_check_boxes_of(dialog: HWND) -> Vec<HWND> {
    descendants_of(dialog)
        .into_iter()
        .filter(|&window| class_of(window) == "Button" && !is_a_push_button(window))
        .collect()
}

/// The box's buttons, left to right.
fn the_buttons_of(dialog: HWND) -> Vec<HWND> {
    let mut buttons: Vec<(i32, HWND)> = descendants_of(dialog)
        .into_iter()
        .filter(|&window| is_a_push_button(window))
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
    button_kind_of(window) == BS_DEFPUSHBUTTON
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

/// Whether the reader ticks "Don't ask me again" before it answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tick {
    First,
    Not,
}

/// The box's check box as the reader found it when the box opened.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CheckBoxHeard {
    /// Its name on MSAA and on UI Automation.
    names: (String, String),
    /// Its MSAA role and state.
    role_and_state: Result<(i64, i64), String>,
    /// Its UI Automation control type.
    control_type: Result<i32, String>,
}

/// What one reading found.
#[derive(Debug, Default)]
struct Reading {
    /// Each button left to right, as MSAA and UI Automation name it.
    buttons: Vec<(String, String)>,
    /// Each check box the box holds.
    check_boxes: Vec<CheckBoxHeard>,
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
fn read_then_answer(answer: Answer, tick: Tick) -> Result<Reading, String> {
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
    let reading = read_and_answer(child.id(), (answer, tick), deadline, &printed);
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
    (answer, tick): (Answer, Tick),
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
        check_boxes: the_check_boxes_of(dialog)
            .into_iter()
            .map(|check_box| CheckBoxHeard {
                names: (msaa_name_of(check_box), uia_name_of(check_box)),
                role_and_state: msaa_role_and_state_of(check_box),
                control_type: uia_control_type_of(check_box),
            })
            .collect(),
        keyboard: keyboard.map(|on| (msaa_name_of(on), is_the_default_button(on))),
        tree: uia_names_under(dialog),
        ..Reading::default()
    };

    if tick == Tick::First {
        // Ticked, and the keyboard left where it was.
        post(dialog, TDM_CLICK_VERIFICATION, 1, 0)?;
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
    reading_after_ticking(answer, Tick::Not)
}

fn reading_after_ticking(answer: Answer, tick: Tick) -> Reading {
    let _turn = ONE_BOX_AT_A_TIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    read_then_answer(answer, tick)
        .unwrap_or_else(|why| panic!("answered by {answer:?}, {tick:?}: {why}"))
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
    let reading = reading_after(Answer::Escape);
    println!("{:#?}", reading.check_boxes);
    let heard = |state| CheckBoxHeard {
        names: (
            "Don't ask me again".to_string(),
            "Don't ask me again".to_string(),
        ),
        role_and_state: Ok((ROLE_SYSTEM_CHECKBUTTON, state)),
        control_type: Ok(UIA_CHECKBOX_CONTROL_TYPE),
    };
    let [check_box] = reading.check_boxes.as_slice() else {
        panic!("the box holds no one check box: {reading:#?}");
    };
    let state = check_box
        .role_and_state
        .clone()
        .map_or(0, |(_, state)| state);
    assert_eq!(
        check_box,
        &heard(state),
        "its names on MSAA and UI Automation, its role and its control type"
    );
    assert_eq!(
        state & STATE_SYSTEM_CHECKED,
        0,
        "ticked when the box opened"
    );
    assert_eq!(reading.ticked, "false", "{reading:#?}");
}

#[test]
fn test_ticked_and_answered_with_the_first_button_is_yes_and_ticked() {
    let reading = reading_after_ticking(Answer::TheFirstButton, Tick::First);
    assert!(!reading.left_open, "{reading:#?}");
    assert_eq!(
        (reading.answered.as_str(), reading.ticked.as_str()),
        ("Yes", "true"),
        "{reading:#?}"
    );
}

#[test]
fn test_ticked_and_answered_with_the_second_button_is_no_and_ticked() {
    let reading = reading_after_ticking(Answer::TheSecondButton, Tick::First);
    assert!(!reading.left_open, "{reading:#?}");
    assert_eq!(
        (reading.answered.as_str(), reading.ticked.as_str()),
        ("No", "true"),
        "{reading:#?}"
    );
}

#[test]
fn test_ticked_and_closed_with_escape_is_neither_and_ticked() {
    let reading = reading_after_ticking(Answer::Escape, Tick::First);
    assert!(!reading.left_open, "{reading:#?}");
    assert_eq!(
        (reading.answered.as_str(), reading.ticked.as_str()),
        ("Neither", "true"),
        "{reading:#?}"
    );
}

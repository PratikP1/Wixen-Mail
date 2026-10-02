//! A key locked with a passphrase asks for it in a field that takes a paste,
//! named where focus lands, and only a reader window asks (GAP-03, #49,
//! 13-17.1).
//!
//! One window session builds the real dialog with `wx_passphrase::build` and
//! reads it the way a screen reader reads native controls: the children in the
//! order Windows holds them, which is the order Tab walks, and each over MSAA
//! through `AccessibleObjectFromWindow`, at its own handle. The field focus
//! lands on is found by asking Windows which window has focus, not by
//! assuming.
//!
//! **Paste is proven with a real paste, on a clipboard of its own.** WCAG
//! 3.3.8 asks that a secret can come from a password manager rather than from
//! memory, and the way a field gets that wrong is by refusing a paste. So text
//! is put on the clipboard and `WM_PASTE` sent to the field, the message
//! Ctrl+V and a password manager both end in. The clipboard belongs to a
//! window station, so this process makes a window station and a desktop of
//! its own before anything is built, and every window here lives there. The
//! clipboard of whoever runs the tests is never read or written, and none of
//! these windows appears on their screen. A companion builds a field that
//! refuses a paste and is caught by the same reading.
//!
//! **The key is never real.** An integration test is built without the
//! library's test backing, so unlocking a real key here would reach the
//! credential store of whoever runs it. What unlocking does is measured in the
//! library against GnuPG's fixtures: `service::pgp::keys` and
//! `application::pgp_keys`. What the main window does with the dialog is read
//! in `src/presentation/wx_app.rs` as text, because its open path needs the
//! main window's cache and runtime, which this file does not start.

#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::presentation::wx_passphrase::{self, THAT_DID_NOT_OPEN_IT, build};
use wxdragon::prelude::*;

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const GWL_STYLE: i32 = -16;
const WS_VISIBLE: isize = 0x1000_0000;
const ES_PASSWORD: isize = 0x0020;
const VT_I4: u16 = 3;
const CHILDID_SELF: i64 = 0;
const WM_PASTE: u32 = 0x0302;
const CF_UNICODETEXT: u32 = 13;
const GMEM_MOVEABLE: u32 = 0x0002;
const WINSTA_ALL_ACCESS: u32 = 0x037F;
const GENERIC_ALL: u32 = 0x1000_0000;

/// MSAA roles (oleacc.h).
const ROLE_SYSTEM_PUSHBUTTON: i64 = 0x2b;
const ROLE_SYSTEM_TEXT: i64 = 0x2a;

/// What the paste puts on the clipboard: long, with spaces, the shape a
/// password manager's generated passphrase has.
const PASTED: &str = "correct horse battery staple, pasted";

const WHOSE: &str = "Ada Lovelace <ada@example.com>";

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

/// A VARIANT as the 64-bit ABI lays it out: 24 bytes, the type at offset 0
/// and the payload at offset 8.
#[repr(C)]
#[derive(Clone, Copy)]
struct Variant {
    vt: u16,
    reserved1: u16,
    reserved2: u16,
    reserved3: u16,
    val: i64,
    extra: u64,
}

impl Variant {
    fn child(id: i64) -> Self {
        Variant {
            vt: VT_I4,
            reserved1: 0,
            reserved2: 0,
            reserved3: 0,
            val: id,
            extra: 0,
        }
    }

    fn empty() -> Self {
        Self::child(0).with_type(0)
    }

    fn with_type(self, vt: u16) -> Self {
        Variant { vt, ..self }
    }
}

type Hresult = i32;
type ReleaseFn = unsafe extern "system" fn(*mut c_void) -> u32;
type GetVariantFn = unsafe extern "system" fn(*mut c_void, Variant, *mut Variant) -> Hresult;
type GetBstrFn = unsafe extern "system" fn(*mut c_void, Variant, *mut *mut u16) -> Hresult;

// IAccessible's vtable: IUnknown (3), IDispatch (4), then get_accParent (7),
// get_accChildCount (8), get_accChild (9), get_accName (10), get_accValue
// (11), get_accDescription (12), get_accRole (13).
const VTBL_RELEASE: usize = 2;
const VTBL_GET_ACC_NAME: usize = 10;
const VTBL_GET_ACC_ROLE: usize = 13;

#[link(name = "user32")]
unsafe extern "system" {
    fn EnumChildWindows(
        parent: isize,
        callback: extern "system" fn(isize, isize) -> i32,
        lparam: isize,
    ) -> i32;
    fn GetClassNameW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
    fn GetWindowTextW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
    fn GetWindowLongPtrW(hwnd: isize, index: i32) -> isize;
    fn GetFocus() -> isize;
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

#[link(name = "oleacc")]
unsafe extern "system" {
    fn AccessibleObjectFromWindow(
        hwnd: isize,
        id_object: u32,
        riid: *const Guid,
        out: *mut *mut c_void,
    ) -> Hresult;
}

#[link(name = "oleaut32")]
unsafe extern "system" {
    fn SysStringLen(s: *mut u16) -> u32;
    fn SysFreeString(s: *mut u16);
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Move this thread onto a desktop in a window station of this process's
/// own, so the clipboard it writes is not the clipboard of whoever runs the
/// tests and no window it makes reaches their screen.
///
/// Must run before the first window of the process is made, since a thread
/// with a window cannot change desktop.
fn a_desktop_of_its_own() -> Result<(), String> {
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
            wide("wixen-passphrase-test").as_ptr(),
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
    }
    Ok(())
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

thread_local! {
    static FOUND: RefCell<Vec<isize>> = const { RefCell::new(Vec::new()) };
}

extern "system" fn collect(hwnd: isize, _lparam: isize) -> i32 {
    FOUND.with(|found| found.borrow_mut().push(hwnd));
    1
}

fn descendants_of(parent: isize) -> Vec<isize> {
    FOUND.with(|found| found.borrow_mut().clear());
    // SAFETY: the callback only pushes to this thread's local.
    unsafe { EnumChildWindows(parent, collect, 0) };
    FOUND.with(|found| found.borrow().clone())
}

fn class_name(hwnd: isize) -> String {
    let mut buffer = [0u16; 256];
    // SAFETY: the buffer is as long as the count says.
    let len = unsafe { GetClassNameW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

fn window_text(hwnd: isize) -> String {
    let mut buffer = [0u16; 4096];
    // SAFETY: the buffer is as long as the count says.
    let len = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

fn style_of(hwnd: isize) -> isize {
    // SAFETY: a live window handle; the style word is only read.
    unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) }
}

unsafe fn vtable_entry(object: *mut c_void, index: usize) -> *const c_void {
    // SAFETY: a COM object is a pointer to its vtable.
    unsafe {
        let vtable = *(object as *const *const *const c_void);
        *vtable.add(index)
    }
}

unsafe fn take_bstr(s: *mut u16) -> String {
    if s.is_null() {
        return String::new();
    }
    // SAFETY: a BSTR carries its length; it is freed once, here.
    unsafe {
        let len = SysStringLen(s) as usize;
        let text = String::from_utf16_lossy(std::slice::from_raw_parts(s, len));
        SysFreeString(s);
        text
    }
}

/// A window's own name and role over MSAA, which is what NVDA reads for a
/// native control.
fn msaa_of(hwnd: isize) -> Result<(String, i64), String> {
    let mut object: *mut c_void = std::ptr::null_mut();
    // SAFETY: a live window handle; the object is released before returning.
    let hr =
        unsafe { AccessibleObjectFromWindow(hwnd, OBJID_CLIENT, &IID_IACCESSIBLE, &mut object) };
    if hr < 0 || object.is_null() {
        return Err(format!("AccessibleObjectFromWindow failed 0x{hr:x}"));
    }
    // SAFETY: `object` is a live IAccessible; the slots are IAccessible's.
    unsafe {
        let get_name: GetBstrFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_NAME));
        let get_role: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_ROLE));
        let mut name: *mut u16 = std::ptr::null_mut();
        let name = match get_name(object, Variant::child(CHILDID_SELF), &mut name) >= 0 {
            true => take_bstr(name),
            false => String::new(),
        };
        let mut role = Variant::empty();
        let role = match get_role(object, Variant::child(CHILDID_SELF), &mut role) >= 0
            && role.vt == VT_I4
        {
            true => role.val & 0xFFFF_FFFF,
            false => -1,
        };
        let release: ReleaseFn = std::mem::transmute(vtable_entry(object, VTBL_RELEASE));
        release(object);
        Ok((name, role))
    }
}

/// One child of the dialog, as Windows holds it.
#[derive(Debug, Clone, PartialEq)]
struct Control {
    class: String,
    text: String,
    visible: bool,
    password: bool,
    name: String,
    role: i64,
}

fn control_at(hwnd: isize) -> Result<Control, String> {
    let (name, role) = msaa_of(hwnd)?;
    let style = style_of(hwnd);
    Ok(Control {
        class: class_name(hwnd),
        text: window_text(hwnd),
        visible: style & WS_VISIBLE != 0,
        password: style & ES_PASSWORD != 0,
        name,
        role,
    })
}

fn read_the_controls(dialog: &Dialog) -> Result<Vec<Control>, String> {
    descendants_of(dialog.get_handle() as isize)
        .into_iter()
        .map(control_at)
        .collect()
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
    controls: Vec<Control>,
    focused: Option<Control>,
    pasted: String,
    ok: Option<String>,
    cancel: Option<String>,
    asked_again: Vec<Control>,
    refused_a_paste: String,
}

fn take_the_harvest() -> Result<Harvest, String> {
    if std::mem::size_of::<Variant>() != 24 {
        return Err("VARIANT is not 24 bytes here, so the reader's layout is wrong".to_string());
    }
    a_desktop_of_its_own()?;
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let taken: Result<Harvest, String> = (|| {
                let frame = Frame::builder().build();

                let asking = build(&frame, WHOSE, None);
                let controls = read_the_controls(&asking.dialog)?;
                asking.dialog.show(true);
                // SAFETY: asks which window of this thread has focus.
                let focused = match unsafe { GetFocus() } {
                    0 => None,
                    hwnd => Some(control_at(hwnd)?),
                };
                let pasted = what_a_paste_puts_in(&asking.field)?;
                let ok = asking.answer(ID_OK);
                let cancel = asking.answer(ID_CANCEL);
                asking.dialog.destroy();

                let again = build(&frame, WHOSE, Some(THAT_DID_NOT_OPEN_IT));
                let asked_again = read_the_controls(&again.dialog)?;

                // The companion: a field that refuses a paste, read the same way.
                let refusing = TextCtrl::builder(&again.dialog)
                    .with_style(TextCtrlStyle::Password | TextCtrlStyle::ReadOnly)
                    .build();
                let refused_a_paste = what_a_paste_puts_in(&refusing)?;
                again.dialog.destroy();

                Ok(Harvest {
                    controls,
                    focused,
                    pasted,
                    ok,
                    cancel,
                    asked_again,
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

/// The controls a keyboard reaches, in the order it reaches them, as name
/// and role.
fn names_and_roles(controls: &[Control]) -> Vec<(String, i64)> {
    controls
        .iter()
        .filter(|control| control.visible && control.class != "Static")
        .map(|control| (control.name.clone(), control.role))
        .collect()
}

/// What the dialog says in lines of text, in the order Windows holds them,
/// with the line breaks wrapping put in read as the spaces they replaced.
fn what_it_says(controls: &[Control]) -> Vec<String> {
    controls
        .iter()
        .filter(|control| control.class == "Static")
        .map(|control| control.text.replace("\r\n", " ").replace('\n', " "))
        .collect()
}

// ── The dialog ─────────────────────────────────────────────────────────────

#[test]
fn test_focus_lands_on_the_field_and_it_is_named_passphrase_there() {
    let focused = the_harvest().focused.as_ref();

    let found = focused.map(|control| {
        (
            control.class.as_str(),
            control.name.as_str(),
            control.role,
            control.password,
        )
    });
    assert_eq!(
        found,
        Some(("Edit", "Passphrase", ROLE_SYSTEM_TEXT, true)),
        "{focused:?}"
    );
}

#[test]
fn test_the_field_and_the_buttons_are_named_in_tab_order() {
    assert_eq!(
        names_and_roles(&the_harvest().controls),
        vec![
            ("Passphrase".to_string(), ROLE_SYSTEM_TEXT),
            ("OK".to_string(), ROLE_SYSTEM_PUSHBUTTON),
            ("Cancel".to_string(), ROLE_SYSTEM_PUSHBUTTON),
        ]
    );
}

#[test]
fn test_it_says_whose_key_it_is_and_that_the_passphrase_is_never_saved() {
    let said = what_it_says(&the_harvest().controls);

    assert_eq!(
        said,
        vec![
            wx_passphrase::what_it_asks(WHOSE),
            "&Passphrase".to_string()
        ]
    );
    assert!(
        said[0].contains(WHOSE) && said[0].contains("never saved"),
        "{said:?}"
    );
}

#[test]
fn test_asked_again_after_a_wrong_passphrase_it_says_so_first() {
    let said = what_it_says(&the_harvest().asked_again);

    assert_eq!(said.first().map(String::as_str), Some(THAT_DID_NOT_OPEN_IT));
    assert_eq!(
        THAT_DID_NOT_OPEN_IT,
        "That passphrase did not open the key. Try again."
    );
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

// ── Only a reader window asks, read as text ────────────────────────────────
//
// The open path lives in the main window's own functions, with its cache and
// runtime, which this file does not start. So the path is read in
// `src/presentation/wx_app.rs`, and a companion plants the fault the reading
// exists for and is refused.

const WX_APP: &str = "src/presentation/wx_app.rs";
const ASKS: &str = "ask_for_the_passphrases_they_need(";

fn the_main_window() -> String {
    std::fs::read_to_string(WX_APP)
        .expect("the main window's source")
        .replace("\r\n", "\n")
}

/// One top-level function's text, from its signature to the brace that
/// closes it at the left margin, which is where rustfmt puts it.
fn body_of<'a>(source: &'a str, signature: &str) -> Option<&'a str> {
    let start = source.find(signature)?;
    let rest = &source[start..];
    rest.find("\n}\n").map(|end| &rest[..end])
}

/// Lines of text that are code rather than comments.
fn code_of(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<&str>>()
        .join("\n")
}

fn what_is_wrong_with_where_it_asks(source: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    let asks_in =
        |signature: &str| body_of(source, signature).map(|body| code_of(body).contains(ASKS));
    for (reader, signature) in [
        ("a single message", "fn open_single_message("),
        ("a conversation", "fn open_conversation_again("),
    ] {
        match asks_in(signature) {
            Some(true) => {}
            Some(false) => wrong.push(format!("opening {reader} in a reader window never asks")),
            None => wrong.push(format!("{signature} is not in the main window")),
        }
    }
    for (never, signature) in [
        ("the preview", "fn the_preview_of("),
        ("Space's second press", "fn read_the_whole_message("),
    ] {
        match asks_in(signature) {
            Some(false) => {}
            Some(true) => wrong.push(format!("{never} asks for a passphrase")),
            None => wrong.push(format!("{signature} is not in the main window")),
        }
    }
    wrong
}

#[test]
fn test_only_a_reader_window_asks_and_the_preview_never_does() {
    let found = what_is_wrong_with_where_it_asks(&the_main_window());

    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn test_the_reading_refuses_a_preview_that_asks() {
    let source = the_main_window();
    let Some(body) = body_of(&source, "fn the_preview_of(") else {
        panic!("fn the_preview_of( is not in {WX_APP}, so there is nowhere to plant");
    };
    let Some(opens) = body.find("{\n") else {
        panic!("the preview's body has no opening brace to plant after");
    };
    let mut planted_body = body.to_string();
    planted_body.insert_str(
        opens + 2,
        "    ask_for_the_passphrases_they_need(frame, cache, &[message.message_id]);\n",
    );
    let planted = source.replacen(body, &planted_body, 1);

    let found = what_is_wrong_with_where_it_asks(&planted);

    assert!(
        found.iter().any(|it| it.contains("the preview asks")),
        "the reading accepted a preview that asks: {found:?}"
    );
}

// ── Only the real paste opens the clipboard ────────────────────────────────
//
// Windows refuses the clipboard to every program while the session is locked,
// so a target that opens it fails on a locked machine whatever its code does
// (ledger 716). The one reading that needs a real clipboard lives in
// `tests/a_passphrase_box_takes_a_real_paste.rs`, and this census keeps every
// other target off it. It lives here because the record on the main window
// runs this target on every commit touching `src/presentation/wx_app.rs`.
//
// What it cannot see: a new file under `tests/` alone reaches it only at the
// phase's full gate and on CI, since the gate couples no file under `tests/`
// to another target; a clipboard reached through a public function further
// down; and a Ctrl+V posted as keys. It reads code only, with comments and
// literals blanked, so a sentence about the clipboard is never a hit.

const THE_REAL_PASTE: &str = "tests/a_passphrase_box_takes_a_real_paste.rs";

/// What a test reaching the clipboard writes, in code.
const CLIPBOARD_CALLS: [&str; 12] = [
    "OpenClipboard",
    "SetClipboardData",
    "GetClipboardData",
    "EmptyClipboard",
    "WM_PASTE",
    "WM_COPY",
    "WM_CUT",
    "Clipboard::",
    "TheDesktop::this_one",
    ".paste()",
    ".copy()",
    ".cut()",
];

fn is_identifier(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn blank(c: char) -> char {
    if c == '\n' { '\n' } else { ' ' }
}

/// Where Rust source is, for blanking what is not code.
#[derive(Clone, Copy)]
enum Within {
    Code,
    LineComment,
    BlockComment(u32),
    Literal,
    RawLiteral(usize),
}

/// How many `#` follow a raw string's `r` at `at`, when what is there opens
/// a raw string rather than naming an identifier.
fn opens_a_raw_literal(chars: &[char], at: usize) -> Option<usize> {
    let before = |back: usize| at.checked_sub(back).map(|i| chars[i]);
    let starts_a_token = match before(1) {
        Some('b') => before(2).is_none_or(|c| !is_identifier(c)),
        Some(c) => !is_identifier(c),
        None => true,
    };
    if chars.get(at) != Some(&'r') || !starts_a_token {
        return None;
    }
    let hashes = chars[at + 1..].iter().take_while(|&&c| c == '#').count();
    (chars.get(at + 1 + hashes) == Some(&'"')).then_some(hashes)
}

/// Whether the `"` at `at` and the `hashes` after it close a raw string.
fn closes_a_raw_literal(chars: &[char], at: usize, hashes: usize) -> bool {
    let after = &chars[at + 1..];
    chars[at] == '"' && after.len() >= hashes && after[..hashes].iter().all(|&c| c == '#')
}

/// How long the character literal at `at` is, or `None` for a lifetime.
fn a_character_literal_at(chars: &[char], at: usize) -> Option<usize> {
    match chars.get(at + 1) {
        Some('\\') => chars
            .get(at + 3..)?
            .iter()
            .position(|&c| c == '\'')
            .map(|closes| closes + 4),
        Some(_) if chars.get(at + 2) == Some(&'\'') => Some(3),
        _ => None,
    }
}

/// A Rust file's text with every comment and literal blanked to spaces and
/// its line breaks kept, so what is left is code and each line keeps its
/// number.
fn code_only(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut code = String::with_capacity(text.len());
    let mut within = Within::Code;
    let mut at = 0;
    while at < chars.len() {
        let (c, next) = (chars[at], chars.get(at + 1).copied());
        let (taken, now) = match within {
            Within::Code => match (c, next) {
                ('/', Some('/')) => (2, Within::LineComment),
                ('/', Some('*')) => (2, Within::BlockComment(1)),
                ('"', _) => (1, Within::Literal),
                ('r', _) => match opens_a_raw_literal(&chars, at) {
                    Some(hashes) => (hashes + 2, Within::RawLiteral(hashes)),
                    None => (0, Within::Code),
                },
                ('\'', _) => match a_character_literal_at(&chars, at) {
                    Some(length) => (length, Within::Code),
                    None => (0, Within::Code),
                },
                _ => (0, Within::Code),
            },
            Within::LineComment if c == '\n' => (1, Within::Code),
            Within::LineComment => (1, Within::LineComment),
            Within::BlockComment(depth) => match (c, next) {
                ('*', Some('/')) if depth == 1 => (2, Within::Code),
                ('*', Some('/')) => (2, Within::BlockComment(depth - 1)),
                ('/', Some('*')) => (2, Within::BlockComment(depth + 1)),
                _ => (1, within),
            },
            Within::Literal => match c {
                '\\' => (2, Within::Literal),
                '"' => (1, Within::Code),
                _ => (1, Within::Literal),
            },
            Within::RawLiteral(hashes) if closes_a_raw_literal(&chars, at, hashes) => {
                (hashes + 1, Within::Code)
            }
            Within::RawLiteral(_) => (1, within),
        };
        match taken {
            0 => {
                code.push(c);
                at += 1;
            }
            _ => {
                let end = (at + taken).min(chars.len());
                code.extend(chars[at..end].iter().map(|&c| blank(c)));
                at = end;
            }
        }
        within = now;
    }
    code
}

/// Whether `token` stands on its own at `at` in `line`, rather than inside a
/// longer name such as `WM_COPYDATA`.
fn stands_alone(line: &str, at: usize, token: &str) -> bool {
    let before = line[..at].chars().next_back();
    let after = line[at + token.len()..].chars().next();
    let edge_ok = |edge: Option<char>, side: Option<char>| match (edge, side) {
        (Some(e), Some(s)) if is_identifier(e) => !is_identifier(s),
        _ => true,
    };
    edge_ok(token.chars().next(), before) && edge_ok(token.chars().next_back(), after)
}

/// Every clipboard call in one file's code, as `path:line token`.
fn clipboard_calls_in(path: &str, text: &str) -> Vec<String> {
    let code = code_only(text);
    let mut hits = Vec::new();
    for (index, line) in code.lines().enumerate() {
        for token in CLIPBOARD_CALLS {
            let found = line
                .match_indices(token)
                .any(|(at, _)| stands_alone(line, at, token));
            if found {
                hits.push(format!("{path}:{} {token}", index + 1));
            }
        }
    }
    hits
}

/// Every `.rs` file directly under `tests/`, as path and text.
fn every_test_target() -> Vec<(String, String)> {
    let entries = std::fs::read_dir("tests").expect("the tests directory");
    let mut files: Vec<(String, String)> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_file() && path.extension().is_some_and(|it| it == "rs"))
        .map(|path| {
            let name = path.file_name().map(|it| it.to_string_lossy().into_owned());
            let shown = format!("tests/{}", name.unwrap_or_default());
            let text = std::fs::read_to_string(&path).expect("a test file read as text");
            (shown, text)
        })
        .collect();
    files.sort();
    files
}

#[test]
fn test_no_target_but_the_real_paste_opens_the_clipboard() {
    let files = every_test_target();
    assert!(
        files.len() > 100,
        "read {} files under tests, so the reading found nothing to judge",
        files.len()
    );

    let found: Vec<String> = files
        .iter()
        .filter(|(path, _)| path != THE_REAL_PASTE)
        .flat_map(|(path, text)| clipboard_calls_in(path, text))
        .collect();

    assert!(
        found.is_empty(),
        "these targets reach the clipboard, which Windows refuses while the session is locked; \
         only {THE_REAL_PASTE} may: {found:#?}"
    );
}

#[test]
fn test_the_real_paste_target_is_where_the_clipboard_is_opened() {
    let text = std::fs::read_to_string(THE_REAL_PASTE)
        .unwrap_or_else(|why| panic!("{THE_REAL_PASTE} could not be read: {why}"));

    let found = clipboard_calls_in(THE_REAL_PASTE, &text);

    for token in ["OpenClipboard", "WM_PASTE"] {
        assert!(
            found.iter().any(|hit| hit.ends_with(&format!(" {token}"))),
            "{THE_REAL_PASTE} has no {token} in code, so the census allows a file that does not use it: {found:#?}"
        );
    }
}

#[test]
fn test_the_clipboard_census_sees_a_call_planted_in_another_target() {
    let path = "tests/a_window_reading_that_pastes.rs";
    let in_code = "fn reads() {\n    let opened = unsafe { OpenClipboard(0) };\n}\n";
    let in_a_comment = "fn reads() {\n    // OpenClipboard(0) is never called here\n}\n";
    let in_a_literal = "fn reads() {\n    let said = \"OpenClipboard(0)\";\n    let raw = r#\"WM_PASTE \"\"#;\n}\n";

    assert_eq!(
        clipboard_calls_in(path, in_code),
        vec![format!("{path}:2 OpenClipboard")]
    );
    assert_eq!(clipboard_calls_in(path, in_a_comment), Vec::<String>::new());
    assert_eq!(clipboard_calls_in(path, in_a_literal), Vec::<String>::new());
}

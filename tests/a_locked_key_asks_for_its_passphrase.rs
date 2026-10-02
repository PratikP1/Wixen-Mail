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
//! **The paste is proven in a target of its own, and nothing here opens the
//! clipboard.** WCAG 3.3.8 asks that the field take a paste, and that is read
//! with a real paste in `tests/a_passphrase_box_takes_a_real_paste.rs`. It
//! lived here until 13-44.6.1. Windows refuses the clipboard to every program
//! while the session is locked, the gate runs this target on every commit
//! touching `src/presentation/wx_app.rs`, and one refused clipboard failed the
//! four readings below that never needed it (ledger 716). This process still
//! makes a window station and a desktop of its own before anything is built,
//! so none of these windows appears on the screen of whoever runs the tests.
//! A census at the end keeps every target but the real paste off the
//! clipboard.
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
const WINSTA_ALL_ACCESS: u32 = 0x037F;
const GENERIC_ALL: u32 = 0x1000_0000;

/// MSAA roles (oleacc.h).
const ROLE_SYSTEM_PUSHBUTTON: i64 = 0x2b;
const ROLE_SYSTEM_TEXT: i64 = 0x2a;

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

/// Move this thread onto a desktop in a window station that is not the one
/// of whoever runs the tests, so no window it makes reaches their screen.
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

/// Everything read out of the window session, as plain values.
#[derive(Debug)]
struct Harvest {
    controls: Vec<Control>,
    focused: Option<Control>,
    asked_again: Vec<Control>,
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
                asking.dialog.destroy();

                let again = build(&frame, WHOSE, Some(THAT_DID_NOT_OPEN_IT));
                let asked_again = read_the_controls(&again.dialog)?;
                again.dialog.destroy();

                Ok(Harvest {
                    controls,
                    focused,
                    asked_again,
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

// ── A key is sent only on a desktop made for its run ───────────────────────
//
// A window on the interactive desktop takes the keys a person types and is
// put in front of them and read by their screen reader, and two runs at once
// on one desktop activate each other's windows. Key-posting cases stopped the
// merges of 13-44 and 13-44.5 that way, and ledger 580 is mark_as_read
// failing while the tester used the machine. So every target that sends a key
// or a click builds its windows on a desktop it makes for its run, named with
// its process id, and this census refuses one that does not. It shares the
// clipboard census's reading, so a key named in a comment or a string is never
// a hit.
//
// What it cannot see: it reads presence, not order, so a desktop made after
// the first window is caught only at run time, where `SetThreadDesktop`
// refuses a thread that already holds a window; and a new file under `tests/`
// alone reaches it only at the phase's full gate and on CI.

/// What a test sending a key or a click to a window writes, in code.
const KEY_SENDS: [&str; 7] = [
    "WM_KEYDOWN",
    "WM_SYSKEYDOWN",
    "WM_CHAR",
    "WM_SYSCHAR",
    "BM_CLICK",
    "SendInput",
    "keybd_event",
];

/// Every key a file sends in code on a desktop it did not make for its run,
/// as `path: sends TOKEN on a desktop it did not make for its run`.
fn keys_sent_off_a_desktop_of_its_run(path: &str, text: &str) -> Vec<String> {
    let code = code_only(text);
    let names = |token: &str| {
        code.lines().any(|line| {
            line.match_indices(token)
                .any(|(at, _)| stands_alone(line, at, token))
        })
    };
    if names("CreateDesktopW") && names("process::id()") {
        return Vec::new();
    }
    KEY_SENDS
        .into_iter()
        .filter(|token| names(token))
        .map(|token| format!("{path}: sends {token} on a desktop it did not make for its run"))
        .collect()
}

/// What a test sending a key writes, in code, where the modifiers the key is
/// read with decide what it does. A click carries no modifiers.
const KEYS_READ_WITH_MODIFIERS: [&str; 6] = [
    "WM_KEYDOWN",
    "WM_SYSKEYDOWN",
    "WM_CHAR",
    "WM_SYSCHAR",
    "SendInput",
    "keybd_event",
];

/// Every key a file sends in code without first setting which modifiers
/// are down, as `path: sends TOKEN with whatever modifiers are held`.
///
/// A desktop of its own keeps a person's key presses away and not the keys
/// they hold: measured 2026-10-02 with the windows already on desktops made
/// for their runs, every one of eight failed runs of
/// `mark_as_read_says_which_way_it_will_go` and `several_steps_come_back`
/// came while the person at the machine held Shift, and none of the 345
/// watched runs without a modifier held failed.
fn keys_sent_with_whatever_modifiers_are_held(path: &str, text: &str) -> Vec<String> {
    let code = code_only(text);
    let names = |token: &str| {
        code.lines().any(|line| {
            line.match_indices(token)
                .any(|(at, _)| stands_alone(line, at, token))
        })
    };
    if names("only_these_modifiers_down") {
        return Vec::new();
    }
    KEYS_READ_WITH_MODIFIERS
        .into_iter()
        .filter(|token| names(token))
        .map(|token| format!("{path}: sends {token} with whatever modifiers are held"))
        .collect()
}

#[test]
fn test_no_target_sends_a_key_with_whatever_modifiers_are_held() {
    let files = every_test_target();
    assert!(
        files.len() > 100,
        "read {} files under tests, so the reading found nothing to judge",
        files.len()
    );

    let found: Vec<String> = files
        .iter()
        .flat_map(|(path, text)| keys_sent_with_whatever_modifiers_are_held(path, text))
        .collect();

    assert!(
        found.is_empty(),
        "these targets send keys read with whatever Shift, Control or Alt the person at the \
         machine holds; set the modifiers each key means with only_these_modifiers_down: \
         {found:#?}"
    );
}

#[test]
fn test_the_modifier_census_sees_a_sender_that_sets_no_modifiers() {
    let path = "tests/a_reading_that_types.rs";
    let leaves_them =
        "fn types(hwnd: isize) {\n    unsafe { SendMessageW(hwnd, WM_KEYDOWN, 0x0D, 1) };\n}\n";
    let sets_them = format!(
        "fn types(hwnd: isize) {{\n    only_these_modifiers_down(&[]);\n{}}}\n",
        "    unsafe { SendMessageW(hwnd, WM_KEYDOWN, 0x0D, 1) };\n"
    );
    let clicks =
        "fn clicks(hwnd: isize) {\n    unsafe { SendMessageW(hwnd, BM_CLICK, 0, 0) };\n}\n";

    assert_eq!(
        keys_sent_with_whatever_modifiers_are_held(path, leaves_them),
        vec![format!(
            "{path}: sends WM_KEYDOWN with whatever modifiers are held"
        )]
    );
    assert_eq!(
        keys_sent_with_whatever_modifiers_are_held(path, &sets_them),
        Vec::<String>::new()
    );
    assert_eq!(
        keys_sent_with_whatever_modifiers_are_held(path, clicks),
        Vec::<String>::new()
    );
}

#[test]
fn test_no_target_sends_a_key_outside_a_desktop_made_for_its_run() {
    let files = every_test_target();
    assert!(
        files.len() > 100,
        "read {} files under tests, so the reading found nothing to judge",
        files.len()
    );

    let found: Vec<String> = files
        .iter()
        .flat_map(|(path, text)| keys_sent_off_a_desktop_of_its_run(path, text))
        .collect();

    assert!(
        found.is_empty(),
        "these targets send keys where a person's typing, a locked session or a second run \
         reaches them; build their windows on a desktop made for the run: {found:#?}"
    );
}

#[test]
fn test_the_key_census_sees_a_sender_planted_without_a_desktop() {
    let path = "tests/a_reading_that_types.rs";
    let in_code =
        "fn types(hwnd: isize) {\n    unsafe { PostMessageW(hwnd, WM_KEYDOWN, 0x0D, 1) };\n}\n";
    let in_a_comment = "fn types() {\n    // WM_KEYDOWN is never posted here\n}\n";
    let in_a_literal =
        "fn types() {\n    let said = \"WM_KEYDOWN\";\n    let raw = r#\"BM_CLICK \"\"#;\n}\n";

    assert_eq!(
        keys_sent_off_a_desktop_of_its_run(path, in_code),
        vec![format!(
            "{path}: sends WM_KEYDOWN on a desktop it did not make for its run"
        )]
    );
    assert_eq!(
        keys_sent_off_a_desktop_of_its_run(path, in_a_comment),
        Vec::<String>::new()
    );
    assert_eq!(
        keys_sent_off_a_desktop_of_its_run(path, in_a_literal),
        Vec::<String>::new()
    );
}

#[test]
fn test_the_key_census_sees_a_desktop_not_named_for_its_run() {
    let path = "tests/a_reading_that_types.rs";
    let sender = "    unsafe { SendMessageW(hwnd, WM_KEYDOWN, 0x0D, 1) };\n";
    let fixed = format!(
        "fn reads(hwnd: isize) {{\n    let name = wide(\"wixen-a-reading-test\");\n    \
         let desktop = unsafe {{ CreateDesktopW(name.as_ptr(), null(), null(), 0, ALL, null()) }};\n\
         {sender}}}\n"
    );
    let for_its_run = fixed.replace(
        "wide(\"wixen-a-reading-test\")",
        "wide(&format!(\"wixen-a-reading-{}\", std::process::id()))",
    );

    assert_eq!(
        keys_sent_off_a_desktop_of_its_run(path, &fixed),
        vec![format!(
            "{path}: sends WM_KEYDOWN on a desktop it did not make for its run"
        )]
    );
    assert_eq!(
        keys_sent_off_a_desktop_of_its_run(path, &for_its_run),
        Vec::<String>::new()
    );
}

// ── A key sent into a browser is sent from a child, in the one turn ────────
//
// A browser's process starts on its process's desktop, not its thread's, so a
// target that types into a page cannot move its window thread alone: it runs
// its window tests in a child of the same executable started on a desktop
// made for the run. And WebView2 runs one browser per user data folder, named
// for the executable, so two runs of one target at once share a browser even
// on two desktops (ledger 761); such a target takes the one turn the clipboard
// target takes, around its child. Measured 2026-10-02 for 13-44.6.3: the
// marker, invitation and meeting readings with their window thread moved alone
// never saw a page come up, and each passed 20 of 20 started whole on a
// desktop of its own.
//
// What it cannot see: a browser is read by the names below, so a new way to
// build one escapes the rule until it is added to the list. The rule case
// holds each program function in the list to the program, so a renamed one
// fails rather than leaving the list reading nothing.

/// What a test building a browser writes, in code: the control, and the two
/// functions of the program that build a window holding one.
const BROWSER_BUILDERS: [&str; 3] = [
    "WebView",
    "build_compose_dialog",
    "show_conversation_as_page",
];

/// Where the program defines each function in `BROWSER_BUILDERS`.
const BUILDERS_THE_PROGRAM_HAS: [(&str, &str); 2] = [
    (
        "src/presentation/wx_compose.rs",
        "pub fn build_compose_dialog(",
    ),
    (
        "src/presentation/wx_app.rs",
        "pub fn show_conversation_as_page(",
    ),
];

/// What a file names, in code, when it runs its windows in a child and takes
/// the one turn.
const IN_A_CHILD_AND_THE_ONE_TURN: [&str; 2] = ["CreateProcessW", "THE_TURN"];

/// A file that sends a key into a window holding a browser without running
/// it in a child and taking the one turn, as `path: ... without WHAT`.
fn keys_sent_into_a_browser_on_the_screen(path: &str, text: &str) -> Vec<String> {
    let code = code_only(text);
    let names = |token: &str| {
        code.lines().any(|line| {
            line.match_indices(token)
                .any(|(at, _)| stands_alone(line, at, token))
        })
    };
    let sends_a_key = KEY_SENDS.into_iter().any(&names);
    let builds_a_browser = BROWSER_BUILDERS.into_iter().any(&names);
    let missing: Vec<&str> = IN_A_CHILD_AND_THE_ONE_TURN
        .into_iter()
        .filter(|token| !names(token))
        .collect();
    if !sends_a_key || !builds_a_browser || missing.is_empty() {
        return Vec::new();
    }
    vec![format!(
        "{path}: sends a key into a window holding a browser without {}",
        missing.join(" and ")
    )]
}

#[test]
fn test_a_key_sender_with_a_browser_runs_in_a_child_and_takes_the_one_turn() {
    for (file, signature) in BUILDERS_THE_PROGRAM_HAS {
        let source = std::fs::read_to_string(file)
            .unwrap_or_else(|why| panic!("{file} could not be read: {why}"));
        assert!(
            source.contains(signature),
            "{file} no longer has {signature}, so the browser rule reads a name nothing builds"
        );
    }
    let files = every_test_target();
    assert!(
        files.len() > 100,
        "read {} files under tests, so the reading found nothing to judge",
        files.len()
    );

    let found: Vec<String> = files
        .iter()
        .flat_map(|(path, text)| keys_sent_into_a_browser_on_the_screen(path, text))
        .collect();

    assert!(
        found.is_empty(),
        "these targets type into a page where a person, a lock or a second run reaches it; \
         run their window tests in a child on a desktop made for the run and take the one \
         turn: {found:#?}"
    );
}

#[test]
fn test_the_browser_rule_sees_a_planted_page_typed_into_on_the_screen() {
    let path = "tests/a_page_typed_into.rs";
    let types = "    unsafe { PostMessageW(hwnd, WM_CHAR, 0x61, 1) };\n";
    let neither = format!("fn types(page: &WebView, hwnd: isize) {{\n{types}}}\n");
    let a_child_alone = format!(
        "fn types(page: &WebView, hwnd: isize) {{\n    let started = unsafe {{ CreateProcessW() }};\n{types}}}\n"
    );
    let both = format!(
        "fn types(page: &WebView, hwnd: isize) {{\n    let turn = take(THE_TURN);\n    \
         let started = unsafe {{ CreateProcessW() }};\n{types}}}\n"
    );
    let no_browser = format!("fn types(hwnd: isize) {{\n{types}}}\n");

    assert_eq!(
        keys_sent_into_a_browser_on_the_screen(path, &neither),
        vec![format!(
            "{path}: sends a key into a window holding a browser without CreateProcessW and THE_TURN"
        )]
    );
    assert_eq!(
        keys_sent_into_a_browser_on_the_screen(path, &a_child_alone),
        vec![format!(
            "{path}: sends a key into a window holding a browser without THE_TURN"
        )]
    );
    assert_eq!(
        keys_sent_into_a_browser_on_the_screen(path, &both),
        Vec::<String>::new()
    );
    assert_eq!(
        keys_sent_into_a_browser_on_the_screen(path, &no_browser),
        Vec::<String>::new()
    );
}

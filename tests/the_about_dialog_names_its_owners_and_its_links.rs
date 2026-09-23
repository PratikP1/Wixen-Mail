//! The About dialog names its owners, its licence and its two pages, and a
//! screen reader hears each page as a link named by its address (#78, 12-04).
//!
//! One window session builds the real dialog with `build_about_dialog` and
//! reads it the way NVDA reads a native control: the children in the order
//! Windows holds them, which is the order Tab walks, each one's class and
//! text, and each link over MSAA through `AccessibleObjectFromWindow`. Nothing
//! calls `show_modal`, nothing is clicked, and no window is shown, so no page
//! is opened and the foreground does not move.
//!
//! **Why a link and not a button, measured 2026-09-23 before the dialog was
//! written.** A `HyperlinkCtrl` is a native `SysLink` here. Bare, its window
//! answers role client (0xa) with the address as its name, and its one item,
//! child 1, answers role link (0x1e), the address as its name and the whole
//! URL as its value, state 0x500000 (focusable, linked). A `Button` labelled
//! with the address answers push button (0x2b) with the address as its name,
//! bare or named. With `set_accessible_name` attached, the `SysLink` answered
//! no children at all and only the client object: the naming object this
//! program uses everywhere else erases the link. So the links carry the
//! control's own text as their name, which is the address on both channels,
//! and this reading holds them to the link item being there.
//!
//! The copyright is one sentence with three readers besides the dialog:
//! `application::about::COPYRIGHT`, line 3 of `LICENSE`, and the copyright
//! `build.rs` stamps on the executable, which Windows shows in the file's
//! properties. Two companions hand the pure readings a stale licence and a
//! dialog missing a page, so each is known to complain.
//!
//! The declarations are the ones `tests/a_kept_folder_reads_as_a_checked_check_box.rs`
//! uses, for the same reason: a feature on the `windows` crate is compiled
//! into the shipping binary.

#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::about;
use wixen_mail::presentation::theme;
use wixen_mail::presentation::wx_app::build_about_dialog;
use wxdragon::prelude::*;

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const VT_I4: u16 = 3;

/// The MSAA role a link answers (oleacc.h).
const ROLE_SYSTEM_LINK: i64 = 0x1e;

/// The message a control sends its parent for the colours to draw its text
/// in (winuser.h).
const WM_CTLCOLORSTATIC: u32 = 0x0138;

/// The two pages, as a person reads them and as the browser is handed them.
/// Literals, so a constant edited in `application::about` fails here too.
const PAGES: [(&str, &str); 2] = [
    ("wixen.app", "https://wixen.app"),
    ("wixen.app/support", "https://wixen.app/support"),
];

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
        Variant {
            vt: 0,
            reserved1: 0,
            reserved2: 0,
            reserved3: 0,
            val: 0,
            extra: 0,
        }
    }
}

type Hresult = i32;
type ReleaseFn = unsafe extern "system" fn(*mut c_void) -> u32;
type GetCountFn = unsafe extern "system" fn(*mut c_void, *mut i32) -> Hresult;
type GetVariantFn = unsafe extern "system" fn(*mut c_void, Variant, *mut Variant) -> Hresult;
type GetBstrFn = unsafe extern "system" fn(*mut c_void, Variant, *mut *mut u16) -> Hresult;

// IAccessible's vtable: IUnknown (3), IDispatch (4), then get_accParent (7),
// get_accChildCount (8), get_accChild (9), get_accName (10), get_accValue
// (11), get_accDescription (12), get_accRole (13).
const VTBL_RELEASE: usize = 2;
const VTBL_GET_ACC_CHILD_COUNT: usize = 8;
const VTBL_GET_ACC_NAME: usize = 10;
const VTBL_GET_ACC_VALUE: usize = 11;
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
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateCompatibleDC(hdc: isize) -> isize;
    fn DeleteDC(hdc: isize) -> i32;
    fn GetTextColor(hdc: isize) -> u32;
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

thread_local! {
    static FOUND: RefCell<Vec<isize>> = const { RefCell::new(Vec::new()) };
}

extern "system" fn collect(hwnd: isize, _lparam: isize) -> i32 {
    FOUND.with(|found| found.borrow_mut().push(hwnd));
    1
}

/// Every descendant window of `parent`, in the order Windows holds them,
/// which for a dialog's direct children is the order Tab walks.
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

/// A window's text, with Windows' line breaks read as the `\n` the dialog
/// was handed.
fn window_text(hwnd: isize) -> String {
    let mut buffer = [0u16; 1024];
    // SAFETY: the buffer is as long as the count says.
    let len = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize]).replace("\r\n", "\n")
}

unsafe fn vtable_entry(object: *mut c_void, index: usize) -> *const c_void {
    // SAFETY: a COM object is a pointer to its vtable.
    unsafe {
        let vtable = *(object as *const *const *const c_void);
        *vtable.add(index)
    }
}

/// A BSTR the object handed out, copied and freed.
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

/// What a link control answers over MSAA: how many items its object says it
/// holds, and the first item's role, name and value.
#[derive(Debug, Clone, PartialEq)]
struct LinkItem {
    items: i32,
    role: i64,
    name: String,
    value: String,
}

fn link_item_of(hwnd: isize) -> Result<LinkItem, String> {
    let mut object: *mut c_void = std::ptr::null_mut();
    // SAFETY: a live window handle; the object is released before returning.
    let hr =
        unsafe { AccessibleObjectFromWindow(hwnd, OBJID_CLIENT, &IID_IACCESSIBLE, &mut object) };
    if hr < 0 || object.is_null() {
        return Err(format!("AccessibleObjectFromWindow failed 0x{hr:x}"));
    }
    // SAFETY: `object` is a live IAccessible; the slots are IAccessible's.
    unsafe {
        let get_count: GetCountFn =
            std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_CHILD_COUNT));
        let get_name: GetBstrFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_NAME));
        let get_value: GetBstrFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_VALUE));
        let get_role: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_ROLE));
        let mut items = 0;
        let hr_count = get_count(object, &mut items);
        let mut role = Variant::empty();
        let hr_role = get_role(object, Variant::child(1), &mut role);
        let mut name: *mut u16 = std::ptr::null_mut();
        let hr_name = get_name(object, Variant::child(1), &mut name);
        let mut value: *mut u16 = std::ptr::null_mut();
        let hr_value = get_value(object, Variant::child(1), &mut value);
        let release: ReleaseFn = std::mem::transmute(vtable_entry(object, VTBL_RELEASE));
        release(object);
        if hr_count < 0 {
            return Err(format!("get_accChildCount hr=0x{hr_count:x}"));
        }
        Ok(LinkItem {
            items,
            role: if hr_role >= 0 && role.vt == VT_I4 {
                role.val & 0xFFFF_FFFF
            } else {
                0
            },
            name: if hr_name >= 0 {
                take_bstr(name)
            } else {
                String::new()
            },
            value: if hr_value >= 0 {
                take_bstr(value)
            } else {
                String::new()
            },
        })
    }
}

/// The colour a control's parent tells it to draw its text in, asked the way
/// the control asks: `WM_CTLCOLORSTATIC` with a device context, then the text
/// colour that context was given, as a COLORREF (0x00BBGGRR).
fn text_colour_for(parent: isize, control: isize) -> u32 {
    // SAFETY: a memory DC made and deleted here; the message is answered by
    // the dialog this file built.
    unsafe {
        let dc = CreateCompatibleDC(0);
        SendMessageW(parent, WM_CTLCOLORSTATIC, dc as usize, control);
        let colour = GetTextColor(dc);
        DeleteDC(dc);
        colour
    }
}

fn colorref(rgb: theme::Rgb) -> u32 {
    u32::from(rgb.r) | (u32::from(rgb.g) << 8) | (u32::from(rgb.b) << 16)
}

/// One child of the dialog, as Windows holds it.
#[derive(Debug, Clone, PartialEq)]
struct Child {
    class: String,
    text: String,
    link: Option<LinkItem>,
    text_colour: u32,
}

#[derive(Debug)]
struct Harvest {
    /// The dialog built on the dark palette.
    dark: Vec<Child>,
    /// The same dialog built on the light palette.
    light: Vec<Child>,
    dark_accent: u32,
    light_accent: u32,
}

fn read_the_dialog(frame: &Frame, palette: theme::Palette) -> Result<Vec<Child>, String> {
    let dialog = build_about_dialog(frame, Some(palette));
    let parent = dialog.get_handle() as isize;
    let mut children = Vec::new();
    for hwnd in descendants_of(parent) {
        let class = class_name(hwnd);
        let link = if class == "SysLink" {
            Some(link_item_of(hwnd)?)
        } else {
            None
        };
        children.push(Child {
            text: window_text(hwnd),
            text_colour: text_colour_for(parent, hwnd),
            class,
            link,
        });
    }
    dialog.destroy();
    Ok(children)
}

fn take_the_harvest() -> Result<Harvest, String> {
    if std::mem::size_of::<Variant>() != 24 {
        return Err("VARIANT is not 24 bytes here, so the reader's layout is wrong".to_string());
    }
    let (Some(dark), Some(light)) = (theme::current("dark"), theme::current("light")) else {
        return Err(
            "no palette: Windows High Contrast is on, so the dialog would be built unpainted"
                .to_string(),
        );
    };
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let taken: Result<Harvest, String> = (|| {
                let frame = Frame::builder().build();
                Ok(Harvest {
                    dark: read_the_dialog(&frame, dark)?,
                    light: read_the_dialog(&frame, light)?,
                    dark_accent: colorref(dark.accent),
                    light_accent: colorref(light.accent),
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

fn complain(what: &str, wrong: &[String]) {
    assert!(
        wrong.is_empty(),
        "{} thing(s) wrong, {what}:\n  {}",
        wrong.len(),
        wrong.join("\n  ")
    );
}

/// What the dialog's children should be, in order: the four lines, the two
/// pages, then OK.
fn what_is_wrong_with_the_order(children: &[Child]) -> Vec<String> {
    let found: Vec<(&str, &str)> = children
        .iter()
        .map(|child| (child.class.as_str(), child.text.as_str()))
        .collect();
    let lines = about::lines();
    let mut wanted: Vec<(&str, &str)> =
        lines.iter().map(|line| ("Static", line.as_str())).collect();
    wanted.push(("SysLink", PAGES[0].0));
    wanted.push(("SysLink", PAGES[1].0));
    wanted.push(("Button", "OK"));
    let mut wrong = Vec::new();
    if found.len() != wanted.len() {
        wrong.push(format!(
            "{} children where {} are wanted: {found:?}",
            found.len(),
            wanted.len()
        ));
    }
    for (at, want) in wanted.iter().enumerate() {
        match found.get(at) {
            // A link's window text is the markup Windows draws it from, so a
            // link is held to its class here and to its name over MSAA.
            Some((class, _)) if want.0 == "SysLink" && *class == "SysLink" => {}
            Some(got) if got == want => {}
            got => wrong.push(format!("child {at} is {got:?} where {want:?} is wanted")),
        }
    }
    wrong
}

/// What is wrong with each link as MSAA answers it.
fn what_is_wrong_with_the_links(children: &[Child]) -> Vec<String> {
    let links: Vec<&LinkItem> = children.iter().filter_map(|c| c.link.as_ref()).collect();
    let mut wrong = Vec::new();
    if links.len() != PAGES.len() {
        wrong.push(format!("{} links where 2 are wanted", links.len()));
    }
    for ((label, address), link) in PAGES.iter().zip(links) {
        if link.items != 1 || link.role != ROLE_SYSTEM_LINK {
            wrong.push(format!(
                "{label:?} answers {} item(s) and role 0x{:x} where one link item (0x1e) is wanted",
                link.items, link.role
            ));
        }
        if link.name != *label {
            wrong.push(format!("{label:?} is named {:?}", link.name));
        }
        if link.value != *address {
            wrong.push(format!("{label:?} goes to {:?}", link.value));
        }
    }
    wrong
}

/// What is wrong with a licence file's copyright line against the dialog's.
fn what_is_wrong_with_the_licence(licence: &str) -> Vec<String> {
    let wanted = about::COPYRIGHT.replacen("Copyright ", "Copyright (c) ", 1);
    match licence.lines().nth(2) {
        Some(line) if line == wanted => Vec::new(),
        other => vec![format!(
            "LICENSE line 3 is {other:?} where {wanted:?} is wanted"
        )],
    }
}

// ── The dialog ─────────────────────────────────────────────────────────────

#[test]
fn test_the_dialog_reads_its_lines_then_the_two_pages_then_ok() {
    complain(
        "in the About dialog's children",
        &what_is_wrong_with_the_order(&the_harvest().dark),
    );
}

#[test]
fn test_each_page_is_a_link_named_by_its_address_over_msaa() {
    complain(
        "in the About dialog's links over MSAA",
        &what_is_wrong_with_the_links(&the_harvest().dark),
    );
}

#[test]
fn test_each_page_is_drawn_in_the_palettes_accent_on_both_palettes() {
    // The native link draws in the system's link blue unless told otherwise,
    // and that blue on the dark palette's surface is under the 4.5:1 text
    // needs. The accent is held to 4.5:1 on both surfaces by the theme's own
    // contrast tests. What this cannot see is the pixels: it asks the colour
    // the dialog hands the link to draw with, the way the link asks.
    let harvest = the_harvest();
    let mut wrong = Vec::new();
    for (palette, children, accent) in [
        ("dark", &harvest.dark, harvest.dark_accent),
        ("light", &harvest.light, harvest.light_accent),
    ] {
        let links: Vec<&Child> = children.iter().filter(|c| c.class == "SysLink").collect();
        if links.len() != PAGES.len() {
            wrong.push(format!(
                "{palette}: {} links where 2 are wanted",
                links.len()
            ));
        }
        for link in links {
            if link.text_colour != accent {
                wrong.push(format!(
                    "{palette}: {:?} is told to draw in 0x{:06x} where the accent is 0x{accent:06x}",
                    link.link.as_ref().map(|l| l.name.as_str()),
                    link.text_colour
                ));
            }
        }
    }
    complain("in the colours the links draw in", &wrong);
}

#[test]
fn test_there_is_no_send_feedback_button_until_its_dialog_arrives() {
    // Pratik's decision on #64 of 2026-09-18: Send Feedback arrives in the
    // same commit as the dialog it opens, so nothing dead sits on About until
    // then. 12-05 rewrites this reading in place to hold it present.
    let children = &the_harvest().dark;
    assert!(
        children.len() > 1,
        "the reading found {} children, so it cannot say what is absent",
        children.len()
    );
    assert!(
        !children.iter().any(|c| c.text.contains("Send Feedback")),
        "the About dialog offers Send Feedback before its dialog exists: {children:?}"
    );
}

// ── The copyright's other readers ──────────────────────────────────────────

#[test]
fn test_the_licence_file_says_the_copyright_the_dialog_says() {
    let licence = std::fs::read_to_string("LICENSE").expect("LICENSE is at the root");
    complain("in LICENSE", &what_is_wrong_with_the_licence(&licence));
}

#[test]
fn test_the_build_script_stamps_the_same_copyright_on_the_executable() {
    // What Windows shows in the file's properties, Details tab.
    let build = std::fs::read_to_string("build.rs").expect("build.rs is at the root");
    let wanted = format!(
        "\"{}. MIT licensed.\"",
        about::COPYRIGHT.replacen("Copyright ", "Copyright (c) ", 1)
    );
    assert!(
        build.contains(&wanted),
        "build.rs does not stamp {wanted} as the executable's copyright"
    );
}

// ── The readings refuse what they exist to refuse ──────────────────────────

fn a_correct_dialog() -> Vec<Child> {
    let child = |class: &str, text: &str, link: Option<(&str, &str)>| Child {
        class: class.to_string(),
        text: text.to_string(),
        link: link.map(|(label, address)| LinkItem {
            items: 1,
            role: ROLE_SYSTEM_LINK,
            name: label.to_string(),
            value: address.to_string(),
        }),
        text_colour: 0,
    };
    let mut children: Vec<Child> = about::lines()
        .iter()
        .map(|line| child("Static", line, None))
        .collect();
    for page in PAGES {
        children.push(child("SysLink", "<a>markup</a>", Some(page)));
    }
    children.push(child("Button", "OK", None));
    children
}

#[test]
fn test_the_readings_refuse_a_dialog_missing_a_page() {
    let correct = a_correct_dialog();
    assert!(
        what_is_wrong_with_the_order(&correct).is_empty()
            && what_is_wrong_with_the_links(&correct).is_empty(),
        "the readings must accept a correct dialog before they can be held to refusing a wrong one"
    );
    let mut missing = correct;
    missing.remove(5);

    assert!(
        !what_is_wrong_with_the_order(&missing).is_empty(),
        "the order reading accepted a dialog with one page"
    );
    assert!(
        !what_is_wrong_with_the_links(&missing).is_empty(),
        "the link reading accepted a dialog with one page"
    );
}

#[test]
fn test_the_licence_reading_refuses_a_stale_copyright_line() {
    let stale = "MIT License\n\nCopyright (c) 2026 Pratik Patel\n\nPermission is hereby granted";

    let wrong = what_is_wrong_with_the_licence(stale);

    assert!(
        wrong.iter().any(|it| it.contains("2026 Pratik Patel")),
        "the licence reading accepted a stale line: {wrong:?}"
    );
}

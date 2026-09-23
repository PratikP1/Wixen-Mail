//! Every spin control names the field a person types in, on both channels, in
//! every window that holds one.
//!
//! A Windows spin control is two windows. The arrows are an `msctls_updown32`,
//! which is the handle wxWidgets hands back and the one `set_accessible_name`
//! attaches its object to. The number sits in the other one, an `Edit` the
//! arrows call their buddy, and that is where the keyboard lands: Tab reaches
//! the field, never the arrows. Phase 6's accessibility scan found the field of
//! every spinner in Edit Event and Send Later nameless on MSAA, which NVDA
//! reads for an edit, and either nameless or named from the nearest static
//! text on UI Automation (ledger 408 to 425, twelve of them about spinners).
//! The tester asked for spin controls where numbers are typed (#73, #35), so a
//! fix that did not reach the field would have added to the class.
//!
//! What is read, and how. Every `msctls_updown32` under each window this file
//! builds is found by class, its buddy asked for with `UDM_GETBUDDY`, and both
//! read over `AccessibleObjectFromWindow(hwnd, OBJID_CLIENT)`: the arrows'
//! name, description and child count, the field's role, name, description and
//! value. The field is read over UI Automation as well, through
//! `IUIAutomation::ElementFromHandle`, because Narrator reads that channel and
//! a name on one channel only is a name somebody does not hear.
//!
//! The whole object is read, not only the name. 12-04 measured a naming object
//! that erased a native link's only child while the name check stayed green,
//! so the field's role and value and the arrows' child count are held to what
//! a bare spin control answers.
//!
//! The windows are built hidden and never shown, so nothing here moves the
//! foreground. One `wxdragon::main` per process serves every test through a
//! `OnceLock` holding a `Result`, the shape
//! `tests/every_settings_checkbox_reads_as_a_checkbox_after_its_page_is_built.rs`
//! gives the reason for. The MSAA reader is that file's sixty lines of user32
//! and oleacc declarations; UI Automation comes from the `windows` crate,
//! whose `Win32_UI_Accessibility` feature the program itself needs for the
//! annotation service that names the field.

#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::new_item::ItemKind;
use wixen_mail::data::config::AppConfig;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::accessibility::names::set_accessible_name;
use wixen_mail::presentation::date_display::{
    Clock, DateOrder, DateSettings, DateStyle, DateWording,
};
use wixen_mail::presentation::wx_item_form::{Chrome, build_item_form_dialog};
use wixen_mail::presentation::{wx_account_manager, wx_compose, wx_send_later, wx_settings};
use wxdragon::prelude::*;

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const VT_I4: u16 = 3;
const CHILDID_SELF: i64 = 0;

/// The up-down control's messages asking for its buddy and its range
/// (commctrl.h): `WM_USER` is 0x400.
const UDM_GETBUDDY: u32 = 0x400 + 106;
const UDM_GETRANGE32: u32 = 0x400 + 112;

/// Typing over a field, and choosing in a list with the arrow keys
/// (winuser.h).
const WM_SETTEXT: u32 = 0x000C;
const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const CB_GETCURSEL: u32 = 0x0147;
const VK_UP: usize = 0x26;
const VK_DOWN: usize = 0x28;

/// What MSAA answers for an edit field (oleacc.h).
const ROLE_SYSTEM_TEXT: i64 = 0x2a;

/// How many spin controls each window holds, counted on the tree as it is.
/// A window that loses one, or a reading that finds none, is a red here
/// rather than a pass over nothing.
const EXPECTED: [(&str, usize); 5] = [
    ("the account editor", 2),
    ("Settings", 5),
    ("the event form", 12),
    ("the table asker", 2),
    ("Send Later", 4),
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
    fn child_self() -> Self {
        Variant {
            vt: VT_I4,
            reserved1: 0,
            reserved2: 0,
            reserved3: 0,
            val: CHILDID_SELF,
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
const VTBL_GET_ACC_DESCRIPTION: usize = 12;
const VTBL_GET_ACC_ROLE: usize = 13;

#[link(name = "user32")]
unsafe extern "system" {
    fn EnumChildWindows(
        parent: isize,
        callback: extern "system" fn(isize, isize) -> i32,
        lparam: isize,
    ) -> i32;
    fn GetClassNameW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
    fn GetParent(hwnd: isize) -> isize;
    fn IsWindowEnabled(hwnd: isize) -> i32;
    fn GetWindowTextW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
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

/// Every descendant window of `parent`, in the order Windows enumerates them.
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

fn send(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize {
    // SAFETY: every handle sent to here belongs to a window this file built
    // and still holds, and every pointer passed outlives the call.
    unsafe { SendMessageW(hwnd, message, wparam, lparam) }
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

/// One MSAA reading of a window's client object. A name, value or
/// description the object declines to give is an empty string, because an
/// edit with no name and an edit that failed to answer are heard the same.
#[derive(Debug, Clone, PartialEq)]
struct Msaa {
    role: i64,
    children: i32,
    name: String,
    description: String,
    value: String,
}

fn msaa_of(hwnd: isize) -> Result<Msaa, String> {
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
        let get_description: GetBstrFn =
            std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_DESCRIPTION));
        let get_role: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_ROLE));
        let release: ReleaseFn = std::mem::transmute(vtable_entry(object, VTBL_RELEASE));

        let mut children = 0i32;
        let hr_count = get_count(object, &mut children);
        let mut role = Variant::empty();
        let hr_role = get_role(object, Variant::child_self(), &mut role);
        let text = |getter: GetBstrFn| {
            let mut s: *mut u16 = std::ptr::null_mut();
            match getter(object, Variant::child_self(), &mut s) >= 0 {
                true => take_bstr(s),
                false => String::new(),
            }
        };
        let name = text(get_name);
        let value = text(get_value);
        let description = text(get_description);
        release(object);
        if hr_count < 0 {
            return Err(format!("get_accChildCount hr=0x{hr_count:x}"));
        }
        if hr_role < 0 || role.vt != VT_I4 {
            return Err(format!("get_accRole hr=0x{hr_role:x} vt={}", role.vt));
        }
        Ok(Msaa {
            role: role.val & 0xFFFF_FFFF,
            children,
            name,
            description,
            value,
        })
    }
}

/// What UI Automation answers as the name of the element for `hwnd`.
fn uia_name_of(hwnd: isize) -> Result<String, String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
    use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation};
    // SAFETY: COM is initialised on this thread by wxWidgets before the main
    // loop starts; the handle is a live window this file built.
    unsafe {
        let automation: IUIAutomation =
            CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
                .map_err(|why| format!("CUIAutomation: {why}"))?;
        let element = automation
            .ElementFromHandle(HWND(hwnd as *mut c_void))
            .map_err(|why| format!("ElementFromHandle: {why}"))?;
        let name = element
            .CurrentName()
            .map_err(|why| format!("CurrentName: {why}"))?;
        Ok(name.to_string())
    }
}

/// One spin control as both channels see it.
#[derive(Debug, Clone)]
struct Spinner {
    window: &'static str,
    arrows: Msaa,
    field_class: String,
    field: Msaa,
    field_on_uia: Result<String, String>,
    range: (i32, i32),
}

impl Spinner {
    fn called(&self) -> String {
        format!("{}: \"{}\"", self.window, self.arrows.name)
    }
}

fn read_the_spinner(window: &'static str, arrows: isize) -> Result<Spinner, String> {
    let buddy = send(arrows, UDM_GETBUDDY, 0, 0);
    if buddy == 0 {
        // SAFETY: a live window handle.
        let parent = class_name(unsafe { GetParent(arrows) });
        return Err(format!(
            "{window}: a spin control with no buddy, its parent a {parent}"
        ));
    }
    let (mut least, mut most) = (0i32, 0i32);
    send(
        arrows,
        UDM_GETRANGE32,
        &mut least as *mut i32 as usize,
        &mut most as *mut i32 as isize,
    );
    Ok(Spinner {
        window,
        arrows: msaa_of(arrows).map_err(|why| format!("{window}, the arrows: {why}"))?,
        field_class: class_name(buddy),
        field: msaa_of(buddy).map_err(|why| format!("{window}, the field: {why}"))?,
        field_on_uia: uia_name_of(buddy),
        range: (least, most),
    })
}

/// The arrows of the spin control under `root` whose arrows answer `name`
/// over MSAA.
fn the_spinner_named(root: isize, name: &str) -> Result<isize, String> {
    descendants_of(root)
        .into_iter()
        .filter(|hwnd| class_name(*hwnd) == "msctls_updown32")
        .find(|hwnd| msaa_of(*hwnd).is_ok_and(|read| read.name == name))
        .ok_or_else(|| format!("no spin control named \"{name}\""))
}

/// The list under `root` that answers `name` over MSAA.
fn the_list_named(root: isize, name: &str) -> Result<isize, String> {
    descendants_of(root)
        .into_iter()
        .filter(|hwnd| class_name(*hwnd) == "ComboBox")
        .find(|hwnd| msaa_of(*hwnd).is_ok_and(|read| read.name == name))
        .ok_or_else(|| format!("no list named \"{name}\""))
}

/// The field a person types in beside `arrows`.
fn the_field_of(arrows: isize) -> isize {
    send(arrows, UDM_GETBUDDY, 0, 0)
}

/// Type `text` over the number in a spin control's field.
fn type_into(arrows: isize, text: &str) {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    send(the_field_of(arrows), WM_SETTEXT, 0, wide.as_ptr() as isize);
}

/// Choose entry `index` in `list` the way a person does, with Up and Down on
/// the closed list, so the list itself sends whatever it tells its parent and
/// wxWidgets hears the choice as it would from a keyboard. Sending the
/// selection message and a hand-made `CBN_SELCHANGE` moved the selection and
/// reached no handler, measured here on 2026-09-23.
fn choose(list: isize, index: usize) {
    let now = send(list, CB_GETCURSEL, 0, 0);
    let key = match (index as isize).cmp(&now) {
        std::cmp::Ordering::Greater => VK_DOWN,
        std::cmp::Ordering::Less => VK_UP,
        std::cmp::Ordering::Equal => return,
    };
    for _ in 0..(index as isize - now).unsigned_abs() {
        send(list, WM_KEYDOWN, key, 0);
        send(list, WM_KEYUP, key, 0);
    }
}

fn is_enabled(hwnd: isize) -> bool {
    // SAFETY: a live window handle.
    unsafe { IsWindowEnabled(hwnd) != 0 }
}

fn text_of(hwnd: isize) -> String {
    let mut buffer = [0u16; 64];
    // SAFETY: the buffer is as long as the count says.
    let len = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

/// Every spin control under `root`, found by the arrows' class.
///
/// A tab row too narrow for its tabs scrolls them with an up-down of its own,
/// the same class, a child of the tab control and with no buddy. It is not a
/// spin control and nobody types in it, so it is left out by its parent
/// rather than by its missing buddy: a spin control that lost its field is a
/// fault this reading reports. wxWidgets registers the tab control under a
/// class of its own, `_wx_SysTabCtl32`, measured here on 2026-09-23.
fn the_spinners_under(window: &'static str, root: isize) -> Result<Vec<Spinner>, String> {
    descendants_of(root)
        .into_iter()
        .filter(|hwnd| class_name(*hwnd) == "msctls_updown32")
        // SAFETY: a live window handle.
        .filter(|hwnd| class_name(unsafe { GetParent(*hwnd) }) != "_wx_SysTabCtl32")
        .map(|arrows| read_the_spinner(window, arrows))
        .collect()
}

fn date_settings() -> DateSettings {
    DateSettings {
        style: DateStyle::Absolute,
        order: DateOrder::MonthFirst,
        wording: DateWording::Verbal,
        clock: Clock::TwentyFourHour,
    }
}

/// A spin control built on its own, twice: once named nowhere, and once named
/// the way every spin control was named before 12-06, through the arrows
/// alone. The first is what the whole object should look like; the second is
/// the defect, so the reading is shown to see it.
struct Bare {
    unnamed: Spinner,
    arrows_only: Spinner,
}

fn read_the_bare_ones(frame: &Frame) -> Result<Bare, String> {
    let unnamed = SpinCtrl::builder(frame).with_range(0, 9).build();
    let arrows_only = SpinCtrl::builder(frame).with_range(0, 9).build();
    set_accessible_name(&arrows_only, "Probe");
    let bare = Bare {
        unnamed: read_the_spinner("bare", unnamed.get_handle() as isize)?,
        arrows_only: read_the_spinner("named on the arrows", arrows_only.get_handle() as isize)?,
    };
    unnamed.destroy();
    arrows_only.destroy();
    Ok(bare)
}

/// The four numbers the tester asked about (#73, #35), by the name their
/// arrows answer, and the range each is to hold as its own.
const THE_FOUR_NUMBERS: [(&str, &str, (i32, i32)); 4] = [
    ("the account editor", "Check Interval (min),", (1, 60)),
    ("Settings", FONT_SIZE, (8, 72)),
    ("Settings", DEFAULT_REMINDER, (0, 1440)),
    ("Settings", MARK_READ_SECONDS, (1, 600)),
];
const FONT_SIZE: &str = "Font size";
const DEFAULT_REMINDER: &str = "Default reminder in minutes";
const MARK_READ_SECONDS: &str = "Mark as read after, in seconds";
const MARK_READ_AFTER: &str = "Mark as read after";

/// What Settings writes back and offers, worked the way a person works it: a
/// number typed over a field, an entry chosen in a list.
struct SettingsAnswers {
    /// Font size 20, Default reminder 45, and a wait of 45 seconds, typed
    /// and chosen, and what OK wrote back for each.
    saved_numbers: Result<(u32, u32, String), String>,
    /// Immediately chosen, then Never chosen, and what OK wrote back.
    saved_words: Result<(String, String), String>,
    /// Whether the seconds can be reached, at each step, in order.
    seconds_offered: Result<Vec<(&'static str, bool)>, String>,
    /// What the seconds field shows in a dialog built over a stored 5.
    seconds_shown_for_a_stored_five: Result<String, String>,
}

fn a_settings_dialog(
    frame: &Frame,
    config: &AppConfig,
    a11y: &Arc<Accessibility>,
) -> wx_settings::SettingsWidgets {
    let settings = wx_settings::build_settings_dialog(frame, config, &[], false, a11y);
    // The pages after General are built the first time their tab is reached,
    // which is what `set_selection` does here without the dialog being shown.
    for tab in 1..settings.notebook.get_page_count() {
        settings.notebook.set_selection(tab);
    }
    settings
}

fn read_what_settings_saves(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
) -> Result<(u32, u32, String), String> {
    let base = AppConfig::default();
    let settings = a_settings_dialog(frame, &base, a11y);
    let root = settings.dialog.get_handle() as isize;
    let read = (|| {
        type_into(the_spinner_named(root, FONT_SIZE)?, "20");
        type_into(the_spinner_named(root, DEFAULT_REMINDER)?, "45");
        choose(the_list_named(root, MARK_READ_AFTER)?, 1);
        type_into(the_spinner_named(root, MARK_READ_SECONDS)?, "45");
        let saved = wx_settings::read_settings(&settings, &base);
        Ok((
            saved.font_size,
            saved.default_reminder_minutes,
            saved.mark_read_after,
        ))
    })();
    settings.dialog.destroy();
    read
}

fn read_what_settings_saves_as_words(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
) -> Result<(String, String), String> {
    let base = AppConfig::default();
    let settings = a_settings_dialog(frame, &base, a11y);
    let root = settings.dialog.get_handle() as isize;
    let read = (|| {
        let list = the_list_named(root, MARK_READ_AFTER)?;
        choose(list, 0);
        let immediately = wx_settings::read_settings(&settings, &base).mark_read_after;
        choose(list, 2);
        let never = wx_settings::read_settings(&settings, &base).mark_read_after;
        Ok((immediately, never))
    })();
    settings.dialog.destroy();
    read
}

fn read_when_the_seconds_are_offered(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
) -> Result<Vec<(&'static str, bool)>, String> {
    let base = AppConfig {
        mark_read_after: "never".to_string(),
        ..AppConfig::default()
    };
    let settings = a_settings_dialog(frame, &base, a11y);
    let root = settings.dialog.get_handle() as isize;
    let read = (|| {
        let seconds = the_field_of(the_spinner_named(root, MARK_READ_SECONDS)?);
        let list = the_list_named(root, MARK_READ_AFTER)?;
        let built = is_enabled(seconds);
        choose(list, 1);
        let a_wait = is_enabled(seconds);
        choose(list, 0);
        let immediately = is_enabled(seconds);
        Ok(vec![
            ("built over Never", built),
            ("After a number of seconds chosen", a_wait),
            ("Immediately chosen", immediately),
        ])
    })();
    settings.dialog.destroy();
    read
}

fn read_the_seconds_for_a_stored_five(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
) -> Result<String, String> {
    let base = AppConfig {
        mark_read_after: "5".to_string(),
        ..AppConfig::default()
    };
    let settings = a_settings_dialog(frame, &base, a11y);
    let root = settings.dialog.get_handle() as isize;
    let read = the_spinner_named(root, MARK_READ_SECONDS).map(|arrows| {
        let field = the_field_of(arrows);
        format!("{}, enabled {}", text_of(field), is_enabled(field))
    });
    settings.dialog.destroy();
    read
}

struct Harvest {
    spinners: Vec<Spinner>,
    bare: Bare,
    settings: SettingsAnswers,
}

/// Every window, built hidden, read, and destroyed.
fn read_every_window(frame: &Frame, a11y: &Arc<Accessibility>) -> Result<Harvest, String> {
    let mut spinners = Vec::new();

    let editor = wx_account_manager::build_account_edit_dialog(frame, None, a11y, None);
    spinners.extend(the_spinners_under(
        "the account editor",
        editor.dialog.get_handle() as isize,
    )?);
    editor.dialog.destroy();

    let settings = a_settings_dialog(frame, &AppConfig::default(), a11y);
    spinners.extend(the_spinners_under(
        "Settings",
        settings.dialog.get_handle() as isize,
    )?);
    settings.dialog.destroy();

    let form = build_item_form_dialog(
        frame,
        ItemKind::Event,
        &[],
        &[],
        Chrome {
            palette: None,
            a11y,
            asking: None,
        },
        date_settings(),
        None,
    )
    .ok_or("the event form was not built")?;
    spinners.extend(the_spinners_under(
        "the event form",
        form.dialog.get_handle() as isize,
    )?);
    form.dialog.destroy();

    let composer = Dialog::builder(frame, "Composer stand-in").build();
    let (asker, _rows, _columns, _header) = wx_compose::build_insert_table_dialog(&composer, None);
    spinners.extend(the_spinners_under(
        "the table asker",
        asker.get_handle() as isize,
    )?);
    asker.destroy();
    composer.destroy();

    let chosen = Rc::new(std::cell::Cell::new(None));
    let send_later = wx_send_later::build_the_asking_dialog(
        frame,
        chrono::Local::now(),
        date_settings(),
        a11y,
        &chosen,
        None,
    );
    spinners.extend(the_spinners_under(
        "Send Later",
        send_later.get_handle() as isize,
    )?);
    send_later.destroy();

    let settings = SettingsAnswers {
        saved_numbers: read_what_settings_saves(frame, a11y),
        saved_words: read_what_settings_saves_as_words(frame, a11y),
        seconds_offered: read_when_the_seconds_are_offered(frame, a11y),
        seconds_shown_for_a_stored_five: read_the_seconds_for_a_stored_five(frame, a11y),
    };

    Ok(Harvest {
        spinners,
        bare: read_the_bare_ones(frame)?,
        settings,
    })
}

/// Everything, inside one `wxdragon::main`, with every failure carried out
/// as a value: nothing in here panics.
fn take_the_harvest() -> Result<Harvest, String> {
    if std::mem::size_of::<Variant>() != 24 {
        return Err("VARIANT is not 24 bytes here, so the reader's layout is wrong".to_string());
    }
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let taken: Result<Harvest, String> = (|| {
                let frame = Frame::builder().build();
                let a11y = Arc::new(
                    Accessibility::new()
                        .map_err(|why| format!("Accessibility::new failed: {why}"))?,
                );
                read_every_window(&frame, &a11y)
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

fn spinners_in(window: &str) -> Vec<&'static Spinner> {
    the_harvest()
        .spinners
        .iter()
        .filter(|spinner| spinner.window == window)
        .collect()
}

fn complain(what: &str, wrong: &[String]) {
    assert!(
        wrong.is_empty(),
        "{} thing(s) wrong, {what}:\n  {}",
        wrong.len(),
        wrong.join("\n  ")
    );
}

/// The field of every spin control in `window` carries the arrows' name over
/// MSAA, and the arrows carry one.
fn assert_every_field_in(window: &str) {
    let spinners = spinners_in(window);
    assert!(
        !spinners.is_empty(),
        "no spin control was found in {window}, so nothing was read"
    );
    let wrong: Vec<String> = spinners
        .iter()
        .filter(|spinner| {
            spinner.arrows.name.is_empty() || spinner.field.name != spinner.arrows.name
        })
        .map(|spinner| {
            format!(
                "{}: the field a person types in answers {:?} over MSAA",
                spinner.called(),
                spinner.field.name
            )
        })
        .collect();
    complain(
        &format!("the typing fields of the spin controls in {window}"),
        &wrong,
    );
}

#[test]
fn test_every_spin_control_in_the_account_editor_names_its_typing_field() {
    assert_every_field_in("the account editor");
}

#[test]
fn test_every_spin_control_in_settings_names_its_typing_field() {
    assert_every_field_in("Settings");
}

#[test]
fn test_every_spin_control_in_the_event_form_names_its_typing_field() {
    assert_every_field_in("the event form");
}

#[test]
fn test_every_spin_control_in_the_table_asker_names_its_typing_field() {
    assert_every_field_in("the table asker");
}

#[test]
fn test_every_spin_control_in_send_later_names_its_typing_field() {
    assert_every_field_in("Send Later");
}

#[test]
fn test_every_typing_field_carries_the_same_name_on_ui_automation() {
    let wrong: Vec<String> = the_harvest()
        .spinners
        .iter()
        .filter(|spinner| spinner.field_on_uia.as_deref() != Ok(spinner.arrows.name.as_str()))
        .map(|spinner| {
            format!(
                "{}: UI Automation answers {:?}",
                spinner.called(),
                spinner.field_on_uia
            )
        })
        .collect();
    complain("the typing fields over UI Automation", &wrong);
}

#[test]
fn test_a_description_on_the_arrows_is_carried_by_the_typing_field_too() {
    // Focus lands on the field, so a sentence only the arrows carry is one
    // nobody tabbing through the form ever hears.
    let described: Vec<&Spinner> = the_harvest()
        .spinners
        .iter()
        .filter(|spinner| !spinner.arrows.description.is_empty())
        .collect();
    assert!(
        !described.is_empty(),
        "no spin control carries a description, so this reading asked nothing"
    );
    let wrong: Vec<String> = described
        .iter()
        .filter(|spinner| spinner.field.description != spinner.arrows.description)
        .map(|spinner| {
            format!(
                "{}: the field's description is {:?}",
                spinner.called(),
                spinner.field.description
            )
        })
        .collect();
    complain("the descriptions of the typing fields", &wrong);
}

#[test]
fn test_each_window_holds_the_spin_controls_it_is_expected_to() {
    let wrong: Vec<String> = EXPECTED
        .iter()
        .filter_map(|(window, expected)| {
            let found = spinners_in(window).len();
            (found != *expected).then(|| format!("{window}: {found} found, {expected} expected"))
        })
        .collect();
    complain("the count of spin controls per window", &wrong);
}

#[test]
fn test_naming_leaves_the_field_and_the_arrows_what_a_bare_spin_control_has() {
    // A naming object can erase what it did not mean to touch (12-04, a link
    // that lost its only child), so the field's role and the arrows' child
    // count are held to a spin control nobody named, and the field's value to
    // the number the arrows hold.
    let bare = &the_harvest().bare.unnamed;
    assert_eq!(bare.field_class, "Edit", "the buddy of a bare spin control");
    assert_eq!(
        bare.field.role, ROLE_SYSTEM_TEXT,
        "a bare spin control's field"
    );
    let wrong: Vec<String> = the_harvest()
        .spinners
        .iter()
        .filter(|spinner| {
            spinner.field_class != bare.field_class
                || spinner.field.role != bare.field.role
                || spinner.field.children != bare.field.children
                || spinner.arrows.role != bare.arrows.role
                || spinner.arrows.children != bare.arrows.children
                || spinner.field.value.trim().parse::<i32>().is_err()
        })
        .map(|spinner| {
            format!(
                "{}: field {} {:?}, arrows {:?}; a bare one has field {:?}, arrows {:?}",
                spinner.called(),
                spinner.field_class,
                spinner.field,
                spinner.arrows,
                bare.field,
                bare.arrows
            )
        })
        .collect();
    complain("the objects a named spin control answers with", &wrong);
}

#[test]
fn test_the_reading_sees_a_field_named_only_on_the_arrows_as_nameless() {
    // The defect the ledger describes, built on purpose: the arrows carry
    // "Probe" and the field carries nothing. A reading that passed this would
    // pass the defect too.
    let arrows_only = &the_harvest().bare.arrows_only;
    assert_eq!(arrows_only.arrows.name, "Probe");
    assert_eq!(
        arrows_only.field.name, "",
        "the field of a spin control named on its arrows alone"
    );
}

#[test]
fn test_the_four_numbers_are_spin_controls_holding_their_own_ranges() {
    // The check interval (#73) and the three numbers in Settings (#35): the
    // bounds each save used to apply after the fact are the control's own,
    // so Up and Down stop at them and a screen reader can say them.
    let wrong: Vec<String> = THE_FOUR_NUMBERS
        .iter()
        .filter_map(|(window, name, range)| {
            let found = spinners_in(window)
                .into_iter()
                .find(|spinner| spinner.arrows.name == *name);
            match found {
                None => Some(format!("{window}: no spin control named \"{name}\"")),
                Some(spinner) if spinner.range != *range => Some(format!(
                    "{}: holds {:?}, not {range:?}",
                    spinner.called(),
                    spinner.range
                )),
                Some(_) => None,
            }
        })
        .collect();
    complain("the four numbers the tester asked about", &wrong);
}

#[test]
fn test_settings_saves_the_numbers_its_spin_controls_hold() {
    assert_eq!(
        the_harvest().settings.saved_numbers,
        Ok((20, 45, "45".to_string())),
        "Font size, Default reminder and a wait of 45 seconds, typed and saved"
    );
}

#[test]
fn test_mark_read_after_saves_immediately_and_never_as_the_words_it_always_stored() {
    // The stored value keeps its shape, so a settings file written before
    // the choice had three entries reads the same after.
    assert_eq!(
        the_harvest().settings.saved_words,
        Ok(("immediately".to_string(), "never".to_string()))
    );
}

#[test]
fn test_the_seconds_are_offered_only_while_a_wait_is_chosen() {
    assert_eq!(
        the_harvest().settings.seconds_offered,
        Ok(vec![
            ("built over Never", false),
            ("After a number of seconds chosen", true),
            ("Immediately chosen", false),
        ])
    );
    assert_eq!(
        the_harvest().settings.seconds_shown_for_a_stored_five,
        Ok("5, enabled true".to_string())
    );
}

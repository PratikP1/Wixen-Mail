//! Other addresses an account sends from, managed from the Account Manager on
//! `Alt+O` (#59, GAP-10, 13-33).
//!
//! One window session builds the real Account Manager, the real manager and
//! the real address window with their own builders, and reads them the way a
//! screen reader reads native controls: the children in the order Windows
//! holds them, which is the order Tab walks, each over MSAA through
//! `AccessibleObjectFromWindow` at its own handle, and the field focus lands
//! on found by asking Windows which window has focus. The manager is read
//! while it runs, opened through the Account Manager's own handler, so its
//! buttons are the ones the manager loop really adds; a pending callback reads
//! it from inside its modal loop and presses Close. Each letter reading has a
//! companion that plants a clash and is caught by it.
//!
//! **Nothing here reaches the settings or the mail of whoever runs it, or
//! their screen.** The profile and the store are directories of their own,
//! made before anything is built, and the windows live on a desktop of this
//! process's own.

#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::identities::{Identity, NOT_SAVED_YET, NOT_TRIED_WITH_A_PROVIDER};
use wixen_mail::application::reordering::Move;
use wixen_mail::application::status_sentences::{Thing, nothing_chosen};
use wixen_mail::data::account::Account;
use wixen_mail::data::message_cache::MessageCache;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::wx_account_manager::{
    build_account_manager_dialog, other_addresses_for,
};
use wixen_mail::presentation::wx_identities::{
    build_address_window, build_identity_manager, populate_identities, wire_the_address_window,
};
use wixen_mail::presentation::wx_managers::{ManagerState, move_the_chosen_row};
use wxdragon::prelude::*;

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const GWL_STYLE: i32 = -16;
const WS_VISIBLE: isize = 0x1000_0000;
const VT_I4: u16 = 3;
const CHILDID_SELF: i64 = 0;
const WINSTA_ALL_ACCESS: u32 = 0x037F;
const GENERIC_ALL: u32 = 0x1000_0000;
const WM_SETTEXT: u32 = 0x000C;
const WM_COMMAND: u32 = 0x0111;
const WM_CLOSE: u32 = 0x0010;
const BN_CLICKED: usize = 0;
const ID_OK: i32 = 5100;

/// MSAA roles (oleacc.h).
const ROLE_SYSTEM_PUSHBUTTON: i64 = 0x2b;
const ROLE_SYSTEM_TEXT: i64 = 0x2a;
const ROLE_SYSTEM_LIST: i64 = 0x21;

/// commctrl.h.
const LVM_FIRST: u32 = 0x1000;
const LVM_SETITEMSTATE: u32 = LVM_FIRST + 43;
const LVM_GETITEMTEXTW: u32 = LVM_FIRST + 115;
const LVM_GETCOLUMNW: u32 = LVM_FIRST + 95;
const LVIS_FOCUSED: u32 = 0x1;
const LVIS_SELECTED: u32 = 0x2;
const LVCF_TEXT: u32 = 0x4;

/// The Account Manager's button, as its label is written.
const THE_BUTTON: &str = "&Other Addresses to Send From...";

const WORK: &str = "acc-work";
const UNSAVED: &str = "acc-unsaved";

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
            ..Self::child(0)
        }
    }
}

/// commctrl.h's `LVITEMW`.
#[repr(C)]
struct ListViewItem {
    mask: u32,
    item: i32,
    sub_item: i32,
    state: u32,
    state_mask: u32,
    text: *mut u16,
    text_max: i32,
    image: i32,
    param: isize,
    indent: i32,
    group_id: i32,
    columns: u32,
    column_list: *mut u32,
    column_formats: *mut i32,
    group: i32,
}

impl ListViewItem {
    fn empty() -> Self {
        ListViewItem {
            mask: 0,
            item: 0,
            sub_item: 0,
            state: 0,
            state_mask: 0,
            text: std::ptr::null_mut(),
            text_max: 0,
            image: 0,
            param: 0,
            indent: 0,
            group_id: 0,
            columns: 0,
            column_list: std::ptr::null_mut(),
            column_formats: std::ptr::null_mut(),
            group: 0,
        }
    }
}

/// commctrl.h's `LVCOLUMNW`.
#[repr(C)]
struct ListViewColumn {
    mask: u32,
    fmt: i32,
    cx: i32,
    text: *mut u16,
    text_max: i32,
    sub_item: i32,
    image: i32,
    order: i32,
    cx_min: i32,
    cx_default: i32,
    cx_ideal: i32,
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
    fn EnumThreadWindows(
        thread: u32,
        callback: extern "system" fn(isize, isize) -> i32,
        lparam: isize,
    ) -> i32;
    fn GetClassNameW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
    fn GetWindowTextW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
    fn GetWindowLongPtrW(hwnd: isize, index: i32) -> isize;
    fn GetDlgCtrlID(hwnd: isize) -> i32;
    fn GetFocus() -> isize;
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
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
    fn GetCurrentThreadId() -> u32;
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
/// own, so no window it makes reaches the screen of whoever runs the tests.
/// Must run before the first window of the process is made.
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
            wide("wixen-other-addresses-test").as_ptr(),
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

/// The top-level window of this thread whose title is `titled`, if one is up.
fn the_window_titled(titled: &str) -> Option<isize> {
    FOUND.with(|found| found.borrow_mut().clear());
    // SAFETY: the callback only pushes to this thread's local.
    unsafe { EnumThreadWindows(GetCurrentThreadId(), collect, 0) };
    FOUND
        .with(|found| found.borrow().clone())
        .into_iter()
        .find(|hwnd| win32_text(*hwnd, GetWindowTextW) == titled)
}

fn win32_text(hwnd: isize, read: unsafe extern "system" fn(isize, *mut u16, i32) -> i32) -> String {
    let mut buffer = [0u16; 4096];
    // SAFETY: the buffer is as long as the count says.
    let len = unsafe { read(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
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
        let mut said: *mut u16 = std::ptr::null_mut();
        let name = match get_name(object, Variant::child(CHILDID_SELF), &mut said) >= 0 {
            true => take_bstr(said),
            false => String::new(),
        };
        let get_role: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_ROLE));
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

/// One child of a window, as Windows holds it.
#[derive(Debug, Clone, PartialEq)]
struct Control {
    hwnd: isize,
    class: String,
    text: String,
    visible: bool,
    name: String,
    role: i64,
}

fn control_at(hwnd: isize) -> Result<Control, String> {
    let (name, role) = msaa_of(hwnd)?;
    // SAFETY: a live window handle; the style word is only read.
    let style = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) };
    Ok(Control {
        hwnd,
        class: win32_text(hwnd, GetClassNameW),
        text: win32_text(hwnd, GetWindowTextW),
        visible: style & WS_VISIBLE != 0,
        name,
        role,
    })
}

fn the_controls_of(window: isize) -> Result<Vec<Control>, String> {
    descendants_of(window).into_iter().map(control_at).collect()
}

fn the_focused_control() -> Result<Option<Control>, String> {
    // SAFETY: asks which window of this thread has focus.
    match unsafe { GetFocus() } {
        0 => Ok(None),
        hwnd => control_at(hwnd).map(Some),
    }
}

/// Press a button the way a click does: its command, sent to its window.
fn press(window: isize, button: isize) {
    // SAFETY: two live windows of this thread; the command is what the
    // button sends on a click, so the handler that runs is the click's.
    unsafe {
        let id = GetDlgCtrlID(button) as u16 as usize;
        SendMessageW(window, WM_COMMAND, id | (BN_CLICKED << 16), button);
    }
}

/// Whether a window carries the visible style. `IsWindowVisible` also asks
/// the window it belongs to, and the Account Manager here is built and never
/// shown, so it answers no for a dialog that is up over it.
fn is_up(window: isize) -> bool {
    // SAFETY: a window handle of this thread; the style word is only read.
    unsafe { GetWindowLongPtrW(window, GWL_STYLE) & WS_VISIBLE != 0 }
}

/// Close a window that is still up once a reading is done with it, so a
/// window that did not answer the way it should leaves the reading red
/// rather than the process waiting on it for ever.
fn close_if_still_up(window: isize) {
    if is_up(window) {
        // SAFETY: a live window of this thread; WM_CLOSE takes no pointers.
        unsafe { SendMessageW(window, WM_CLOSE, 0, 0) };
    }
}

/// The button of `window` whose label is `labelled`.
fn the_button(controls: &[Control], labelled: &str) -> Option<isize> {
    controls
        .iter()
        .find(|control| control.class == "Button" && control.text == labelled)
        .map(|control| control.hwnd)
}

fn type_into(field: isize, text: &str) {
    let text = wide(text);
    // SAFETY: a live edit box of this thread; the text outlives the call.
    unsafe { SendMessageW(field, WM_SETTEXT, 0, text.as_ptr() as isize) };
}

/// Put the list's cursor on a row, selected and focused, the state Windows
/// keeps when a person arrows to it.
fn choose_the_row(list: isize, row: usize) {
    let mut item = ListViewItem::empty();
    item.state = LVIS_SELECTED | LVIS_FOCUSED;
    item.state_mask = LVIS_SELECTED | LVIS_FOCUSED;
    // SAFETY: a live list of this thread; the item outlives the call.
    unsafe {
        SendMessageW(
            list,
            LVM_SETITEMSTATE,
            row,
            &mut item as *mut ListViewItem as isize,
        )
    };
}

/// One cell of a live list, read from the list itself.
fn cell(list: isize, row: usize, column: i32) -> String {
    let mut buffer = [0u16; 512];
    let mut item = ListViewItem::empty();
    item.item = row as i32;
    item.sub_item = column;
    item.text = buffer.as_mut_ptr();
    item.text_max = buffer.len() as i32;
    // SAFETY: a live list on this thread; the item and its buffer outlive the
    // call, and the length handed over is the buffer's.
    let length = unsafe {
        SendMessageW(
            list,
            LVM_GETITEMTEXTW,
            row,
            &mut item as *mut ListViewItem as isize,
        )
    };
    String::from_utf16_lossy(&buffer[..length.clamp(0, buffer.len() as isize) as usize])
}

/// A column's heading, read from the list itself.
fn heading(list: isize, column: usize) -> String {
    let mut buffer = [0u16; 256];
    let mut asked = ListViewColumn {
        mask: LVCF_TEXT,
        fmt: 0,
        cx: 0,
        text: buffer.as_mut_ptr(),
        text_max: buffer.len() as i32,
        sub_item: 0,
        image: 0,
        order: 0,
        cx_min: 0,
        cx_default: 0,
        cx_ideal: 0,
    };
    // SAFETY: a live list on this thread; the column and its buffer outlive
    // the call.
    unsafe {
        SendMessageW(
            list,
            LVM_GETCOLUMNW,
            column,
            &mut asked as *mut ListViewColumn as isize,
        )
    };
    let end = buffer.iter().position(|c| *c == 0).unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..end])
}

/// A list's rows as "address | name", one per row.
fn the_rows(list: isize, count: usize) -> Vec<String> {
    (0..count)
        .map(|row| format!("{} | {}", cell(list, row, 0), cell(list, row, 1)))
        .collect()
}

fn work() -> Account {
    let mut account = Account::new("Work".to_string(), "me@example.com".to_string());
    account.id = WORK.to_string();
    account.username = "me@example.com".to_string();
    account.imap_server = "imap.example.com".to_string();
    account.smtp_server = "smtp.example.com".to_string();
    account
}

fn unsaved() -> Account {
    let mut account = Account::new("Unsaved".to_string(), "new@example.com".to_string());
    account.id = UNSAVED.to_string();
    account
}

fn three_addresses() -> Vec<Identity> {
    vec![
        Identity::typed("i1", "help@example.com", "Help Desk"),
        Identity::typed("i2", "sales@example.com", "Sales"),
        Identity::typed("i3", "old@example.com", ""),
    ]
}

/// What the running manager was read as, from inside its modal loop.
#[derive(Debug, Default, Clone)]
struct WhileItRan {
    found: bool,
    controls: Vec<Control>,
    said_after_the_move: String,
}

/// What pressing OK on an address that cannot be kept left behind.
#[derive(Debug, Default, Clone)]
struct Refused {
    still_shown: bool,
    shows: Vec<String>,
    focused: Option<Control>,
}

/// Everything read out of the window session, as plain values.
#[derive(Debug)]
struct Harvest {
    account_manager: Vec<Control>,
    account_manager_with_a_second_o: Vec<Control>,
    built_title: String,
    built_headings: Vec<String>,
    built_rows: Vec<String>,
    rows_after_a_move: Vec<String>,
    said_after_a_move: String,
    running: WhileItRan,
    stored_after_close: Vec<String>,
    said_after_close: String,
    window: Vec<Control>,
    window_with_a_second_a: Vec<Control>,
    focused_on_open: Option<Control>,
    refused: Refused,
    kept_closes_with_ok: bool,
    said_with_nothing_chosen: String,
    said_for_an_unsaved_account: String,
    a_manager_opened_for_the_unsaved_account: bool,
}

/// The shown text of every static line, with wrapping's breaks as spaces.
fn what_it_shows(controls: &[Control]) -> Vec<String> {
    controls
        .iter()
        .filter(|control| control.visible && control.class == "Static")
        .map(|control| control.text.replace("\r\n", " ").replace('\n', " "))
        .collect()
}

/// Read the manager from inside its modal loop, move the first row down,
/// and close it, once it is up.
fn read_the_manager_while_it_runs(titled: &'static str, into: Arc<Mutex<WhileItRan>>) {
    wxdragon::call_after(Box::new(move || {
        let Some(window) = the_window_titled(titled) else {
            return;
        };
        let mut read = WhileItRan {
            found: true,
            controls: the_controls_of(window).unwrap_or_default(),
            ..WhileItRan::default()
        };
        if let Some(list) = read.controls.iter().find(|c| c.class == "SysListView32") {
            choose_the_row(list.hwnd, 0);
        }
        if let Some(down) = the_button(&read.controls, "Move Do&wn") {
            press(window, down);
        }
        read.said_after_the_move = what_it_shows(&the_controls_of(window).unwrap_or_default())
            .into_iter()
            .find(|line| line.contains(" of "))
            .unwrap_or_default();
        if let Some(close) = the_button(&read.controls, "&Close") {
            press(window, close);
        }
        close_if_still_up(window);
        if let Ok(mut slot) = into.lock() {
            *slot = read;
        }
    }));
}

/// Close a window with this title if one opens, and record that it did.
fn close_it_if_it_opens(titled: &'static str, opened: Arc<Mutex<bool>>) {
    wxdragon::call_after(Box::new(move || {
        let Some(window) = the_window_titled(titled) else {
            return;
        };
        if let Ok(mut slot) = opened.lock() {
            *slot = true;
        }
        let controls = the_controls_of(window).unwrap_or_default();
        if let Some(close) = the_button(&controls, "&Close") {
            press(window, close);
        }
        close_if_still_up(window);
    }));
}

/// Type an address that is not one into the running address window, press
/// OK and read what is left; then type one that is, and press OK again.
fn refuse_then_keep(titled: &'static str, into: Arc<Mutex<Refused>>) {
    wxdragon::call_after(Box::new(move || {
        let Some(window) = the_window_titled(titled) else {
            return;
        };
        let controls = the_controls_of(window).unwrap_or_default();
        let fields: Vec<isize> = controls
            .iter()
            .filter(|c| c.class == "Edit")
            .map(|c| c.hwnd)
            .collect();
        if let (Some(ok), [address, name, ..]) = (the_button(&controls, "OK"), &fields[..]) {
            let (address, name) = (*address, *name);
            type_into(address, "not an address");
            type_into(name, "Help Desk");
            press(window, ok);
            let still_shown = is_up(window);
            let refused = Refused {
                still_shown,
                shows: what_it_shows(&the_controls_of(window).unwrap_or_default()),
                focused: the_focused_control().ok().flatten(),
            };
            if let Ok(mut slot) = into.lock() {
                *slot = refused;
            }
            if still_shown {
                type_into(address, "new@example.com");
                press(window, ok);
            }
        }
        close_if_still_up(window);
    }));
}

fn take_the_harvest() -> Result<Harvest, String> {
    if std::mem::size_of::<Variant>() != 24 {
        return Err("VARIANT is not 24 bytes here, so the reader's layout is wrong".to_string());
    }
    let profile = tempfile::tempdir().map_err(|why| format!("a profile of its own: {why}"))?;
    let store_at = tempfile::tempdir().map_err(|why| format!("a store of its own: {why}"))?;
    // SAFETY: set once, before any thread of this process reads either.
    unsafe {
        std::env::set_var("WIXEN_MAIL_DATA", profile.path());
        std::env::set_var("WIXEN_NO_AUDIO", "1");
    }
    a_desktop_of_its_own()?;
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        let store_at = store_at.path().to_path_buf();
        wxdragon::main(move |app| {
            let taken: Result<Harvest, String> = (|| {
                let frame = Frame::builder().build();
                let a11y = Arc::new(Accessibility::new().map_err(|why| format!("{why}"))?);
                let store = MessageCache::new(store_at, None).map_err(|why| format!("{why}"))?;
                let account = work();
                store
                    .save_account(&account)
                    .map_err(|why| format!("the account to be saved: {why}"))?;
                store
                    .keep_the_identities(WORK, &three_addresses())
                    .map_err(|why| format!("the addresses to be kept: {why}"))?;

                // The Account Manager, and a second O planted on it.
                let manager = build_account_manager_dialog(
                    &frame,
                    std::slice::from_ref(&account),
                    None,
                    None,
                    None,
                );
                let account_manager = the_controls_of(manager.dialog.get_handle() as isize)?;
                Button::builder(&manager.dialog).with_label("&Open").build();
                let account_manager_with_a_second_o =
                    the_controls_of(manager.dialog.get_handle() as isize)?;

                // The manager built and not run, and a move made in it.
                let built = build_identity_manager(&frame, &account, &three_addresses(), None);
                let list = built.list.get_handle() as isize;
                let built_title = built.dialog.get_label().unwrap_or_default();
                let built_headings = vec![heading(list, 0), heading(list, 1)];
                let built_rows = the_rows(list, 3);
                let state = Rc::new(RefCell::new(ManagerState {
                    working: three_addresses(),
                    changed: false,
                }));
                choose_the_row(list, 1);
                move_the_chosen_row(
                    &state,
                    &built.list,
                    &built.status,
                    &a11y,
                    populate_identities,
                    |row: &Identity| row.address.clone(),
                    Move::Down,
                );
                let rows_after_a_move = the_rows(list, 3);
                let said_after_a_move = built.status.get_label();
                built.dialog.destroy();

                // The manager run from the Account Manager's own handler.
                let running = Arc::new(Mutex::new(WhileItRan::default()));
                read_the_manager_while_it_runs(
                    "Other Addresses to Send From: Work",
                    running.clone(),
                );
                other_addresses_for(
                    Some(&account),
                    Some(&store),
                    &manager.dialog,
                    &manager.status,
                    &a11y,
                    None,
                );
                let running = running
                    .lock()
                    .map_err(|_| "the manager reading's lock was poisoned".to_string())?
                    .clone();
                let stored_after_close = store
                    .identities_for(WORK)
                    .map_err(|why| format!("{why}"))?
                    .into_iter()
                    .map(|identity| identity.address)
                    .collect();
                let said_after_close = manager.status.get_label();

                // The address window built, and a second A planted in it.
                let bare = build_address_window(&manager.dialog, None, None);
                let window = the_controls_of(bare.dialog.get_handle() as isize)?;
                StaticText::builder(&bare.dialog)
                    .with_label("&Also:")
                    .build();
                let window_with_a_second_a = the_controls_of(bare.dialog.get_handle() as isize)?;
                bare.dialog.destroy();

                // The address window run, refused once and then kept.
                let asking = build_address_window(&manager.dialog, None, None);
                wire_the_address_window(&asking, &account, &three_addresses(), &a11y);
                asking.dialog.show(true);
                let focused_on_open = the_focused_control()?;
                asking.dialog.show(false);
                let refused = Arc::new(Mutex::new(Refused::default()));
                refuse_then_keep("Add Other Address", refused.clone());
                let kept_closes_with_ok = asking.dialog.show_modal() == ID_OK;
                asking.dialog.destroy();
                let refused = refused
                    .lock()
                    .map_err(|_| "the refusal's lock was poisoned".to_string())?
                    .clone();

                // No account chosen.
                other_addresses_for(
                    None,
                    Some(&store),
                    &manager.dialog,
                    &manager.status,
                    &a11y,
                    None,
                );
                let said_with_nothing_chosen = manager.status.get_label();

                // An account the Account Manager has not saved yet. Last, so
                // the callback armed for it, which finds nothing when the
                // refusal holds, meets no later window.
                let opened = Arc::new(Mutex::new(false));
                close_it_if_it_opens("Other Addresses to Send From: Unsaved", opened.clone());
                other_addresses_for(
                    Some(&unsaved()),
                    Some(&store),
                    &manager.dialog,
                    &manager.status,
                    &a11y,
                    None,
                );
                let said_for_an_unsaved_account = manager.status.get_label();
                let a_manager_opened_for_the_unsaved_account = *opened
                    .lock()
                    .map_err(|_| "the opened flag's lock was poisoned".to_string())?;
                manager.dialog.destroy();

                Ok(Harvest {
                    account_manager,
                    account_manager_with_a_second_o,
                    built_title,
                    built_headings,
                    built_rows,
                    rows_after_a_move,
                    said_after_a_move,
                    running,
                    stored_after_close,
                    said_after_close,
                    window,
                    window_with_a_second_a,
                    focused_on_open,
                    refused,
                    kept_closes_with_ok,
                    said_with_nothing_chosen,
                    said_for_an_unsaved_account,
                    a_manager_opened_for_the_unsaved_account,
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
    drop(profile);
    drop(store_at);
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

/// The letter a label claims, when it claims one; `&&` is a literal
/// ampersand, the rule Windows follows.
fn alt_key_of(label: &str) -> Option<char> {
    let mut chars = label.chars();
    while let Some(c) = chars.next() {
        if c == '&' {
            match chars.next() {
                Some('&') => continue,
                Some(letter) if letter.is_alphanumeric() => {
                    return Some(letter.to_ascii_uppercase());
                }
                _ => {}
            }
        }
    }
    None
}

/// Every letter the showing labels and buttons claim, with the labels that
/// claim it.
fn letters(controls: &[Control]) -> std::collections::BTreeMap<char, Vec<String>> {
    let mut claimed: std::collections::BTreeMap<char, Vec<String>> = Default::default();
    for control in controls.iter().filter(|control| {
        control.visible && (control.class == "Static" || control.class == "Button")
    }) {
        if let Some(letter) = alt_key_of(&control.text) {
            claimed
                .entry(letter)
                .or_default()
                .push(control.text.clone());
        }
    }
    claimed
}

/// Every letter more than one showing control claims.
fn claimed_twice(controls: &[Control]) -> Vec<String> {
    letters(controls)
        .into_iter()
        .filter(|(_, labels)| labels.len() > 1)
        .map(|(letter, labels)| format!("Alt+{letter} on {}", labels.join(" and ")))
        .collect()
}

/// The controls a keyboard reaches, in the order it reaches them, as name
/// and role.
///
/// Not the size grip, which a window that can be resized carries as a child,
/// and not the list's column headings, a child of the list itself: neither
/// takes focus, and the scan reads both on every manager.
fn names_and_roles(controls: &[Control]) -> Vec<(String, i64)> {
    controls
        .iter()
        .filter(|control| {
            control.visible && !["Static", "ScrollBar", "SysHeader32"].contains(&&*control.class)
        })
        .map(|control| (control.name.clone(), control.role))
        .collect()
}

// ── The Account Manager ────────────────────────────────────────────────────

#[test]
fn test_the_account_manager_offers_other_addresses_on_o_and_nothing_else_takes_o() {
    let manager = &the_harvest().account_manager;

    assert_eq!(
        letters(manager).get(&'O'),
        Some(&vec![THE_BUTTON.to_string()]),
        "{:?}",
        letters(manager)
    );
    assert_eq!(claimed_twice(manager), Vec::<String>::new());
}

#[test]
fn test_the_account_managers_letter_reading_sees_a_second_o_when_one_is_planted() {
    // The companion. A reading that never found a clash would pass the case
    // above whatever the window did.
    assert_eq!(
        claimed_twice(&the_harvest().account_manager_with_a_second_o),
        vec![format!("Alt+O on {THE_BUTTON} and &Open")]
    );
}

#[test]
fn test_with_no_account_chosen_the_button_says_what_edit_says() {
    assert_eq!(
        the_harvest().said_with_nothing_chosen,
        nothing_chosen(Thing::ACCOUNT)
    );
}

#[test]
fn test_an_account_not_saved_yet_is_refused_with_a_sentence_and_no_manager_opens() {
    let harvest = the_harvest();

    assert_eq!(harvest.said_for_an_unsaved_account, NOT_SAVED_YET);
    assert!(
        !harvest.a_manager_opened_for_the_unsaved_account,
        "a manager opened for an account nothing holds yet"
    );
}

// ── The manager ────────────────────────────────────────────────────────────

#[test]
fn test_the_manager_is_titled_for_its_account_and_lists_the_addresses_in_their_order() {
    let harvest = the_harvest();

    assert_eq!(harvest.built_title, "Other Addresses to Send From: Work");
    assert_eq!(harvest.built_headings, ["Address", "Name people see"]);
    assert_eq!(
        harvest.built_rows,
        [
            "help@example.com | Help Desk",
            "sales@example.com | Sales",
            "old@example.com | ",
        ]
    );
}

#[test]
fn test_moving_an_address_down_moves_its_row_and_says_where_it_went() {
    let harvest = the_harvest();

    assert_eq!(
        harvest.rows_after_a_move,
        [
            "help@example.com | Help Desk",
            "old@example.com | ",
            "sales@example.com | Sales",
        ]
    );
    assert_eq!(harvest.said_after_a_move, "sales@example.com, 3 of 3.");
}

#[test]
fn test_the_running_manager_gives_each_of_its_letters_to_one_button() {
    let running = &the_harvest().running;
    assert!(
        running.found,
        "the manager did not open from the Account Manager"
    );

    let found: Vec<(char, Vec<String>)> = letters(&running.controls).into_iter().collect();
    assert_eq!(
        found,
        vec![
            ('A', vec!["&Add...".to_string()]),
            ('C', vec!["&Close".to_string()]),
            ('D', vec!["&Delete".to_string()]),
            ('E', vec!["&Edit...".to_string()]),
            ('U', vec!["Move &Up".to_string()]),
            ('W', vec!["Move Do&wn".to_string()]),
        ]
    );
}

#[test]
fn test_the_running_manager_names_its_list_and_its_buttons_on_msaa_in_tab_order() {
    assert_eq!(
        names_and_roles(&the_harvest().running.controls),
        vec![
            ("Other addresses".to_string(), ROLE_SYSTEM_LIST),
            ("Add...".to_string(), ROLE_SYSTEM_PUSHBUTTON),
            ("Edit...".to_string(), ROLE_SYSTEM_PUSHBUTTON),
            ("Delete".to_string(), ROLE_SYSTEM_PUSHBUTTON),
            ("Move Up".to_string(), ROLE_SYSTEM_PUSHBUTTON),
            ("Move Down".to_string(), ROLE_SYSTEM_PUSHBUTTON),
            ("Close".to_string(), ROLE_SYSTEM_PUSHBUTTON),
        ]
    );
}

#[test]
fn test_the_running_manager_says_where_it_is_read_that_no_provider_has_been_asked() {
    assert!(
        what_it_shows(&the_harvest().running.controls)
            .iter()
            .any(|line| line == NOT_TRIED_WITH_A_PROVIDER),
        "{:?}",
        what_it_shows(&the_harvest().running.controls)
    );
}

#[test]
fn test_the_order_is_written_when_the_manager_closes() {
    let harvest = the_harvest();

    assert_eq!(
        harvest.running.said_after_the_move,
        "help@example.com, 2 of 3."
    );
    assert_eq!(
        harvest.stored_after_close,
        ["sales@example.com", "help@example.com", "old@example.com"]
    );
    assert_eq!(
        harvest.said_after_close,
        "The other addresses Work sends from are kept."
    );
}

// ── The address window ─────────────────────────────────────────────────────

#[test]
fn test_every_label_in_the_address_window_has_a_letter_of_its_own() {
    let found: Vec<(char, Vec<String>)> = letters(&the_harvest().window).into_iter().collect();

    assert_eq!(
        found,
        vec![
            ('A', vec!["&Address:".to_string()]),
            ('N', vec!["The &name people see:".to_string()]),
        ]
    );
}

#[test]
fn test_the_address_windows_letter_reading_sees_a_second_a_when_one_is_planted() {
    assert_eq!(
        claimed_twice(&the_harvest().window_with_a_second_a),
        vec!["Alt+A on &Address: and &Also:".to_string()]
    );
}

#[test]
fn test_focus_lands_on_the_address_and_it_is_named_there() {
    let focused = the_harvest().focused_on_open.as_ref();

    assert_eq!(
        focused.map(|control| (control.class.as_str(), control.name.as_str(), control.role)),
        Some(("Edit", "Address,", ROLE_SYSTEM_TEXT)),
        "{focused:?}"
    );
}

/// Each field's name ends in the comma `name_from_label` puts where its
/// label's colon was, which is the pause a screen reader makes before the
/// role; every labelled field in the program is named that way.
#[test]
fn test_the_address_window_names_its_fields_and_buttons_on_msaa_in_tab_order() {
    assert_eq!(
        names_and_roles(&the_harvest().window),
        vec![
            ("Address,".to_string(), ROLE_SYSTEM_TEXT),
            ("The name people see,".to_string(), ROLE_SYSTEM_TEXT),
            ("OK".to_string(), ROLE_SYSTEM_PUSHBUTTON),
            ("Cancel".to_string(), ROLE_SYSTEM_PUSHBUTTON),
        ]
    );
}

#[test]
fn test_an_address_that_cannot_be_kept_keeps_the_window_open_says_why_and_goes_to_the_address() {
    let refused = &the_harvest().refused;

    assert!(
        refused.still_shown,
        "the window closed on an address it cannot keep"
    );
    assert!(
        refused
            .shows
            .iter()
            .any(|line| line
                == "That is not an email address. One is written like name@example.com."),
        "{:?}",
        refused.shows
    );
    assert_eq!(
        refused
            .focused
            .as_ref()
            .map(|control| (control.class.as_str(), control.name.as_str())),
        Some(("Edit", "Address,"))
    );
}

#[test]
fn test_an_address_that_can_be_kept_closes_the_window_with_ok() {
    assert!(the_harvest().kept_closes_with_ok);
}

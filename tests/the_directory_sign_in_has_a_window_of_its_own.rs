//! The directory sign-in has a window of its own, Look People Up at Work,
//! reached from the Account Manager on `Alt+L` (#55, GAP-07, 13-27).
//!
//! One window session builds the real Account Manager and the real window
//! with `wx_account_manager`'s own builders and reads them the way a screen
//! reader reads native controls: the children in the order Windows holds
//! them, which is the order Tab walks, each over MSAA through
//! `AccessibleObjectFromWindow` at its own handle, and the field focus lands
//! on found by asking Windows which window has focus. The letters are read
//! from the showing labels and buttons, which is what the keyboard reaches at
//! once, and each letter reading has a companion that plants a clash and is
//! caught by it.
//!
//! **Nothing here reaches the settings of whoever runs it, or their screen.**
//! The profile is pointed at a directory of its own before anything is built,
//! and the windows live on a desktop of this process's own. The only saves
//! pressed are ones the window refuses, which write nothing; what a save
//! writes is decided in `application::directory_sign_in` and measured there.
//! Where the password goes once decided is read in the source, with a
//! companion that plants the fault.

#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::directory_sign_in::{A_PASSWORD_IS_SAVED, NOT_TRIED_YET};
use wixen_mail::data::account::Account;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::wx_account_manager::{
    build_account_manager_dialog, build_directory_sign_in_dialog, wire_the_directory_sign_in,
};
use wixen_mail::service::directory::{self, Directory};
use wxdragon::prelude::*;

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const GWL_STYLE: i32 = -16;
const WS_VISIBLE: isize = 0x1000_0000;
const ES_PASSWORD: isize = 0x0020;
const VT_I4: u16 = 3;
const CHILDID_SELF: i64 = 0;
const BM_CLICK: u32 = 0x00F5;
const WINSTA_ALL_ACCESS: u32 = 0x037F;
const GENERIC_ALL: u32 = 0x1000_0000;

/// MSAA roles (oleacc.h).
const ROLE_SYSTEM_PUSHBUTTON: i64 = 0x2b;
const ROLE_SYSTEM_TEXT: i64 = 0x2a;

const ADDRESS: &str = "ldaps://directory.example.com";
/// The same directory reached without encryption, where no password goes.
const PLAIN_ADDRESS: &str = "ldap://directory.example.com";
const LOOK_IN: &str = "ou=people,dc=example,dc=com";
const NAME: &str = "cn=reader,dc=example,dc=com";

/// The Account Manager's button, as its label is written.
const THE_BUTTON: &str = "&Look People Up at Work...";

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

type Hresult = i32;
type ReleaseFn = unsafe extern "system" fn(*mut c_void) -> u32;
type GetVariantFn = unsafe extern "system" fn(*mut c_void, Variant, *mut Variant) -> Hresult;
type GetBstrFn = unsafe extern "system" fn(*mut c_void, Variant, *mut *mut u16) -> Hresult;

// IAccessible's vtable: IUnknown (3), IDispatch (4), then get_accParent (7),
// get_accChildCount (8), get_accChild (9), get_accName (10), get_accValue
// (11), get_accDescription (12), get_accRole (13).
const VTBL_RELEASE: usize = 2;
const VTBL_GET_ACC_NAME: usize = 10;
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
    fn GetWindowTextW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
    fn GetWindowLongPtrW(hwnd: isize, index: i32) -> isize;
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
            wide("wixen-directory-window-test").as_ptr(),
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

/// A window's own name, description and role over MSAA, which is what NVDA
/// reads for a native control.
fn msaa_of(hwnd: isize) -> Result<(String, String, i64), String> {
    let mut object: *mut c_void = std::ptr::null_mut();
    // SAFETY: a live window handle; the object is released before returning.
    let hr =
        unsafe { AccessibleObjectFromWindow(hwnd, OBJID_CLIENT, &IID_IACCESSIBLE, &mut object) };
    if hr < 0 || object.is_null() {
        return Err(format!("AccessibleObjectFromWindow failed 0x{hr:x}"));
    }
    // SAFETY: `object` is a live IAccessible; the slots are IAccessible's.
    unsafe {
        let text_at = |slot: usize| {
            let get: GetBstrFn = std::mem::transmute(vtable_entry(object, slot));
            let mut said: *mut u16 = std::ptr::null_mut();
            match get(object, Variant::child(CHILDID_SELF), &mut said) >= 0 {
                true => take_bstr(said),
                false => String::new(),
            }
        };
        let name = text_at(VTBL_GET_ACC_NAME);
        let description = text_at(VTBL_GET_ACC_DESCRIPTION);
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
        Ok((name, description, role))
    }
}

/// One child of a window, as Windows holds it.
#[derive(Debug, Clone, PartialEq)]
struct Control {
    class: String,
    text: String,
    visible: bool,
    password: bool,
    name: String,
    description: String,
    role: i64,
}

fn control_at(hwnd: isize) -> Result<Control, String> {
    let (name, description, role) = msaa_of(hwnd)?;
    // SAFETY: a live window handle; the style word is only read.
    let style = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) };
    Ok(Control {
        class: win32_text(hwnd, GetClassNameW),
        text: win32_text(hwnd, GetWindowTextW),
        visible: style & WS_VISIBLE != 0,
        password: style & ES_PASSWORD != 0,
        name,
        description,
        role,
    })
}

fn read_the_controls(window: &Dialog) -> Result<Vec<Control>, String> {
    descendants_of(window.get_handle() as isize)
        .into_iter()
        .map(control_at)
        .collect()
}

fn the_focused_control() -> Result<Option<Control>, String> {
    // SAFETY: asks which window of this thread has focus.
    match unsafe { GetFocus() } {
        0 => Ok(None),
        hwnd => control_at(hwnd).map(Some),
    }
}

fn an_account() -> Account {
    Account::new("Work".to_string(), "me@example.com".to_string())
}

fn the_directory() -> Directory {
    Directory {
        url: ADDRESS.to_string(),
        search_under: LOOK_IN.to_string(),
        sign_in_as: Some(NAME.to_string()),
    }
}

/// What a refused save left behind.
#[derive(Debug)]
struct Refused {
    still_shown: bool,
    status: String,
    focused: Option<Control>,
}

/// Everything read out of the window session, as plain values.
#[derive(Debug)]
struct Harvest {
    manager: Vec<Control>,
    manager_with_a_second_l: Vec<Control>,
    saved: Vec<Control>,
    focused_on_open: Option<Control>,
    boxes_on_open: [String; 4],
    nothing_saved: Vec<Control>,
    window_with_a_second_d: Vec<Control>,
    refused: Refused,
    refused_unencrypted: Refused,
}

/// Press OK on a window built with nothing saved, this address, the place and
/// a sign-in name, and the password box empty, and read what the refusal
/// left. A name with no password and nothing saved is always refused, so no
/// save pressed here writes the settings or the credential store.
fn refused_with(
    parent: &Dialog,
    account: &Account,
    a11y: &Arc<Accessibility>,
    address: &str,
) -> Result<Refused, String> {
    let refusing = build_directory_sign_in_dialog(parent, &account.name, None, false, None);
    wire_the_directory_sign_in(&refusing, &account.id, false, a11y);
    refusing.address.set_value(address);
    refusing.look_in.set_value(LOOK_IN);
    refusing.sign_in_as.set_value(NAME);
    refusing.dialog.show(true);
    // SAFETY: a live button on this thread; BM_CLICK takes no pointers.
    unsafe { SendMessageW(refusing.ok.get_handle() as isize, BM_CLICK, 0, 0) };
    let refused = Refused {
        still_shown: refusing.dialog.is_shown(),
        status: refusing.status.get_label(),
        focused: the_focused_control()?,
    };
    refusing.dialog.destroy();
    Ok(refused)
}

fn take_the_harvest() -> Result<Harvest, String> {
    if std::mem::size_of::<Variant>() != 24 {
        return Err("VARIANT is not 24 bytes here, so the reader's layout is wrong".to_string());
    }
    let profile = tempfile::tempdir().map_err(|why| format!("a profile of its own: {why}"))?;
    // SAFETY: set once, before any thread of this process reads it.
    unsafe { std::env::set_var("WIXEN_MAIL_DATA", profile.path()) };
    a_desktop_of_its_own()?;
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let taken: Result<Harvest, String> = (|| {
                let frame = Frame::builder().build();
                let a11y = Arc::new(Accessibility::new().map_err(|why| format!("{why}"))?);
                let account = an_account();

                let manager = build_account_manager_dialog(
                    &frame,
                    std::slice::from_ref(&account),
                    None,
                    None,
                    None,
                );
                let manager_controls = read_the_controls(&manager.dialog)?;
                // The companion: a second button on L, read the same way.
                Button::builder(&manager.dialog)
                    .with_label("&Leave")
                    .build();
                let manager_with_a_second_l = read_the_controls(&manager.dialog)?;

                let saved = build_directory_sign_in_dialog(
                    &manager.dialog,
                    &account.name,
                    Some(&the_directory()),
                    true,
                    None,
                );
                let saved_controls = read_the_controls(&saved.dialog)?;
                saved.dialog.show(true);
                let focused_on_open = the_focused_control()?;
                let boxes_on_open = [
                    saved.address.get_value(),
                    saved.look_in.get_value(),
                    saved.sign_in_as.get_value(),
                    saved.password.get_value(),
                ];
                saved.dialog.destroy();

                let nothing = build_directory_sign_in_dialog(
                    &manager.dialog,
                    &account.name,
                    None,
                    false,
                    None,
                );
                let nothing_saved = read_the_controls(&nothing.dialog)?;
                // The companion: a second label on D, read the same way.
                StaticText::builder(&nothing.dialog)
                    .with_label("&Down here:")
                    .build();
                let window_with_a_second_d = read_the_controls(&nothing.dialog)?;
                nothing.dialog.destroy();

                // A save the window refuses: a name, an empty password box and
                // nothing saved. Refused before anything is written.
                let refused = refused_with(&manager.dialog, &account, &a11y, ADDRESS)?;
                // And the same over an unencrypted address, refused for the
                // address. The password box stays empty even so: a window that
                // kept a typed password would hand it to the credential store
                // of whoever runs this, and an empty box is refused either way.
                let refused_unencrypted =
                    refused_with(&manager.dialog, &account, &a11y, PLAIN_ADDRESS)?;

                Ok(Harvest {
                    manager: manager_controls,
                    manager_with_a_second_l,
                    saved: saved_controls,
                    focused_on_open,
                    boxes_on_open,
                    nothing_saved,
                    window_with_a_second_d,
                    refused,
                    refused_unencrypted,
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
fn names_and_roles(controls: &[Control]) -> Vec<(String, i64)> {
    controls
        .iter()
        .filter(|control| control.visible && control.class != "Static")
        .map(|control| (control.name.clone(), control.role))
        .collect()
}

/// What the window shows in lines of text, with the line breaks wrapping
/// put in read as the spaces they replaced.
fn what_it_shows(controls: &[Control]) -> Vec<String> {
    controls
        .iter()
        .filter(|control| control.visible && control.class == "Static")
        .map(|control| control.text.replace("\r\n", " ").replace('\n', " "))
        .collect()
}

fn the_password_box(controls: &[Control]) -> Option<&Control> {
    controls
        .iter()
        .find(|control| control.class == "Edit" && control.password)
}

// ── The Account Manager ────────────────────────────────────────────────────

#[test]
fn test_the_account_manager_offers_look_people_up_at_work_on_l_and_nothing_else_takes_l() {
    let manager = &the_harvest().manager;

    assert_eq!(
        letters(manager).get(&'L'),
        Some(&vec![THE_BUTTON.to_string()]),
        "{:?}",
        letters(manager)
    );
    assert_eq!(claimed_twice(manager), Vec::<String>::new());
}

#[test]
fn test_the_managers_letter_reading_sees_a_second_l_when_one_is_planted() {
    // The companion. A reading that never found a clash would pass the case
    // above whatever the window did.
    assert_eq!(
        claimed_twice(&the_harvest().manager_with_a_second_l),
        vec![format!("Alt+L on {THE_BUTTON} and &Leave")]
    );
}

// ── The window ─────────────────────────────────────────────────────────────

#[test]
fn test_every_label_in_the_window_has_a_letter_of_its_own() {
    let found: Vec<(char, Vec<String>)> = letters(&the_harvest().saved).into_iter().collect();

    assert_eq!(
        found,
        vec![
            ('D', vec!["&Directory address:".to_string()]),
            ('N', vec!["Sign-in &name:".to_string()]),
            ('P', vec!["&Password:".to_string()]),
            ('W', vec!["&Where in it to look:".to_string()]),
        ]
    );
}

#[test]
fn test_the_windows_letter_reading_sees_a_second_d_when_one_is_planted() {
    assert_eq!(
        claimed_twice(&the_harvest().window_with_a_second_d),
        vec!["Alt+D on &Directory address: and &Down here:".to_string()]
    );
}

#[test]
fn test_focus_lands_on_the_directory_address_and_it_is_named_there() {
    let focused = the_harvest().focused_on_open.as_ref();

    assert_eq!(
        focused.map(|control| (control.class.as_str(), control.name.as_str(), control.role)),
        Some(("Edit", "Directory address,", ROLE_SYSTEM_TEXT)),
        "{focused:?}"
    );
}

/// Each field's name ends in the comma `name_from_label` puts where its
/// label's colon was, which is the pause a screen reader makes before the
/// role; every labelled field in the program is named that way.
#[test]
fn test_every_field_and_button_is_named_on_msaa_at_its_own_handle_in_tab_order() {
    assert_eq!(
        names_and_roles(&the_harvest().saved),
        vec![
            ("Directory address,".to_string(), ROLE_SYSTEM_TEXT),
            ("Where in it to look,".to_string(), ROLE_SYSTEM_TEXT),
            ("Sign-in name,".to_string(), ROLE_SYSTEM_TEXT),
            ("Password,".to_string(), ROLE_SYSTEM_TEXT),
            ("OK".to_string(), ROLE_SYSTEM_PUSHBUTTON),
            ("Cancel".to_string(), ROLE_SYSTEM_PUSHBUTTON),
        ]
    );
}

#[test]
fn test_the_boxes_open_on_the_directory_the_account_names_and_the_password_box_empty() {
    assert_eq!(
        the_harvest().boxes_on_open,
        [
            ADDRESS.to_string(),
            LOOK_IN.to_string(),
            NAME.to_string(),
            String::new()
        ]
    );
}

#[test]
fn test_with_a_password_saved_the_box_and_the_window_both_say_so() {
    let saved = &the_harvest().saved;

    assert_eq!(
        the_password_box(saved).map(|password| password.description.as_str()),
        Some(A_PASSWORD_IS_SAVED),
        "the password box does not hide what is typed, or does not say one is saved"
    );
    assert!(
        what_it_shows(saved)
            .iter()
            .any(|line| line == A_PASSWORD_IS_SAVED),
        "{:?}",
        what_it_shows(saved)
    );
}

#[test]
fn test_with_nothing_saved_neither_the_box_nor_the_window_says_one_is() {
    let nothing = &the_harvest().nothing_saved;

    let described = the_password_box(nothing).map(|password| password.description.clone());
    assert!(
        described
            .as_deref()
            .is_some_and(|said| !said.is_empty() && said != A_PASSWORD_IS_SAVED),
        "{described:?}"
    );
    assert!(
        !what_it_shows(nothing)
            .iter()
            .any(|line| line == A_PASSWORD_IS_SAVED),
        "{:?}",
        what_it_shows(nothing)
    );
}

#[test]
fn test_the_window_says_first_that_it_has_not_been_tried_against_a_real_directory() {
    assert_eq!(
        what_it_shows(&the_harvest().saved)
            .first()
            .map(String::as_str),
        Some(NOT_TRIED_YET)
    );
}

#[test]
fn test_a_refused_save_keeps_the_window_open_says_why_and_puts_focus_on_the_password() {
    let refused = &the_harvest().refused;

    assert!(refused.still_shown, "the window closed on a refused save");
    // The line wraps, so its breaks are read as the spaces they replaced.
    assert_eq!(
        refused.status.replace("\r\n", " ").replace('\n', " "),
        directory::no_password_is_saved_for("directory.example.com", NAME)
    );
    assert_eq!(
        refused
            .focused
            .as_ref()
            .map(|control| (control.name.as_str(), control.password)),
        Some(("Password,", true)),
        "{:?}",
        refused.focused
    );
}

#[test]
fn test_a_sign_in_over_an_unencrypted_address_is_refused_at_ok_with_focus_on_the_address() {
    let refused = &the_harvest().refused_unencrypted;

    assert!(refused.still_shown, "the window closed on a refused save");
    assert_eq!(
        refused.status.replace("\r\n", " ").replace('\n', " "),
        directory::no_password_is_sent_unencrypted_to("directory.example.com")
    );
    // The address is what has to change, so focus goes there.
    assert_eq!(
        refused
            .focused
            .as_ref()
            .map(|control| (control.name.as_str(), control.password)),
        Some(("Directory address,", false)),
        "{:?}",
        refused.focused
    );
}

// ── Where the password goes, read as text ──────────────────────────────────
//
// A save that is not refused writes the settings and the credential store of
// whoever runs it, so it is not pressed here. The settings cannot hold a
// password at all, since the directory they store has no field for one; the
// reading asks that stays so, and that the save hands a typed password to the
// credential store and nowhere else.

const THE_MANAGER: &str = "src/presentation/wx_account_manager.rs";
const THE_DIRECTORY: &str = "src/service/directory.rs";
const THE_SAVE: &str = "fn keep_what_the_directory_window_holds(";

fn source_of(path: &str) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|why| panic!("{path}: {why}"))
        .replace("\r\n", "\n")
}

/// One top-level item's text, from its opening line to the brace that closes
/// it at the left margin, which is where rustfmt puts it.
fn body_of<'a>(source: &'a str, opening: &str) -> Option<&'a str> {
    let start = source.find(opening)?;
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

fn what_is_wrong_with_where_the_password_goes(manager: &str, service: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    match body_of(manager, THE_SAVE).map(code_of) {
        None => wrong.push(format!("{THE_SAVE} is not in {THE_MANAGER}")),
        Some(save) => {
            for call in [
                "directory::keep_the_password(",
                "directory::forget_the_password(",
            ] {
                if !save.contains(call) {
                    wrong.push(format!("the save never calls {call}"));
                }
            }
        }
    }
    match body_of(service, "pub struct Directory {").map(code_of) {
        None => wrong.push(format!(
            "the Directory the settings store is not in {THE_DIRECTORY}"
        )),
        Some(fields) if fields.to_lowercase().contains("password") => {
            wrong.push("the Directory the settings store has a field for a password".to_string())
        }
        Some(_) => {}
    }
    wrong
}

#[test]
fn test_a_typed_password_goes_to_the_credential_store_and_the_settings_have_no_room_for_one() {
    let found = what_is_wrong_with_where_the_password_goes(
        &source_of(THE_MANAGER),
        &source_of(THE_DIRECTORY),
    );

    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn test_the_reading_sees_a_save_that_skips_the_store_and_a_directory_that_holds_a_password() {
    let manager = source_of(THE_MANAGER).replacen(
        "directory::keep_the_password(",
        "directory::not_the_store(",
        1,
    );
    let service = source_of(THE_DIRECTORY).replacen(
        "pub struct Directory {",
        "pub struct Directory {\n    pub password: String,",
        1,
    );

    let found = what_is_wrong_with_where_the_password_goes(&manager, &service);

    assert_eq!(
        found,
        vec![
            "the save never calls directory::keep_the_password(".to_string(),
            "the Directory the settings store has a field for a password".to_string(),
        ]
    );
}

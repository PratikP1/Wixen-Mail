//! The PGP key manager lists every key as a person first, names every control
//! at its own handle over MSAA, and asks before it removes anything (GAP-03,
//! #49, 13-17).
//!
//! One window session builds the real window with `wx_pgp_keys::build` over
//! two keys and reads it the way NVDA reads native controls: the children in
//! the order Windows holds them, which is the order Tab walks, and each one
//! over MSAA through `AccessibleObjectFromWindow`, its name, role and state.
//! The list's rows are read as MSAA children of the list's own object, which
//! is what a screen reader says as the cursor lands on each.
//!
//! **The keys are made up and kept in memory, on purpose.** An integration
//! test is built against the library without its test backing, so a real key
//! imported here would go into the Windows credential store of whoever ran
//! the tests, under the name their own Wixen Mail reads, and listing the keys
//! here would read theirs. So the window is given its keys through the same
//! seam `show` gives it the real ones, and what the real ones do is measured
//! in `application::pgp_keys` and in `wx_pgp_keys`'s own tests against
//! GnuPG's fixtures: that Export and Copy hand over a public half, that a
//! locked key is refused, that removal leaves nothing behind. The question
//! and the clipboard are handed in for the same reason: a test that answered
//! a real message box would need somebody to press it, and one that wrote to
//! the real clipboard would overwrite whatever the person running it had
//! copied.
//!
//! What this cannot see: that a click on a button reaches the method it calls.
//! Each button's handler is one call to the method pressed here.

#![cfg(windows)]

use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::c_void;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::pgp_keys::{Imported, WHAT_KEYS_CAN_DO_HERE};
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::date_display::{
    Clock, DateOrder, DateSettings, DateStyle, DateWording,
};
use wixen_mail::presentation::wx_pgp_keys::{
    TheDesktop, TheKeysUnderneath, build, build_the_paste_dialog,
};
use wixen_mail::service::pgp::KeyListing;
use wxdragon::prelude::*;

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const GWL_STYLE: i32 = -16;
const WS_VISIBLE: isize = 0x1000_0000;
const VT_I4: u16 = 3;
const CHILDID_SELF: i64 = 0;

/// MSAA roles (oleacc.h).
const ROLE_SYSTEM_PUSHBUTTON: i64 = 0x2b;
const ROLE_SYSTEM_TEXT: i64 = 0x2a;
const ROLE_SYSTEM_LIST: i64 = 0x21;
const ROLE_SYSTEM_LISTITEM: i64 = 0x22;
/// The resize grip a resizable dialog carries in its corner, which no key
/// reaches.
const ROLE_SYSTEM_GRIP: i64 = 0x4;

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
// (11), get_accDescription (12), get_accRole (13), get_accState (14).
const VTBL_RELEASE: usize = 2;
const VTBL_GET_ACC_CHILD_COUNT: usize = 8;
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
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
}

/// commctrl.h: `LVM_FIRST + 115`.
const LVM_GETITEMTEXTW: u32 = 0x1000 + 115;

/// commctrl.h's `LVITEMW`, of which `LVM_GETITEMTEXTW` reads the sub-item,
/// the buffer and its length.
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

/// One cell of the live list, read from the list itself.
///
/// Not through `ListCtrl::get_item_text`, which loses the last character of
/// every cell and puts a NUL in its place (`wxdragon-0.9.17`,
/// `src/widgets/list_ctrl.rs:429`): read that way, "27/09/2026" came back as
/// "27/09/202" and a NUL on 2026-09-29. A date's last digit is the year, so
/// the cell is read whole, the way `tests/a_signature_follows_the_from_account.rs`
/// reads one.
fn cell(list: &ListCtrl, row: i64, column: i32) -> String {
    let mut buffer = [0u16; 512];
    let mut item = ListViewItem {
        mask: 0,
        item: row as i32,
        sub_item: column,
        state: 0,
        state_mask: 0,
        text: buffer.as_mut_ptr(),
        text_max: buffer.len() as i32,
        image: 0,
        param: 0,
        indent: 0,
        group_id: 0,
        columns: 0,
        column_list: std::ptr::null_mut(),
        column_formats: std::ptr::null_mut(),
        group: 0,
    };
    // SAFETY: a live list on this thread; the item and its buffer outlive the
    // call, and the length handed over is the buffer's.
    let length = unsafe {
        SendMessageW(
            list.get_handle() as isize,
            LVM_GETITEMTEXTW,
            row as usize,
            &mut item as *mut ListViewItem as isize,
        )
    };
    String::from_utf16_lossy(&buffer[..length.clamp(0, buffer.len() as isize) as usize])
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
    String::from_utf16_lossy(&buffer[..len.max(0) as usize]).replace("\r\n", "\n")
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

/// What a window's own object answers over MSAA, and the names of its
/// children, which for a list are its rows.
#[derive(Debug, Clone, PartialEq)]
struct Msaa {
    name: String,
    role: i64,
    children: Vec<String>,
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
        let get_role: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_ROLE));
        let name_of = |child: i64| {
            let mut name: *mut u16 = std::ptr::null_mut();
            match get_name(object, Variant::child(child), &mut name) >= 0 {
                true => take_bstr(name),
                false => String::new(),
            }
        };
        let role_of = |child: i64| {
            let mut role = Variant::empty();
            match get_role(object, Variant::child(child), &mut role) >= 0 && role.vt == VT_I4 {
                true => role.val & 0xFFFF_FFFF,
                false => -1,
            }
        };
        let name = name_of(CHILDID_SELF);
        let role = role_of(CHILDID_SELF);
        // A list's rows are its children with the list item role, numbered
        // from one. The count also holds the header, a window of its own
        // with no name, which is why the role is asked. Only a list is asked,
        // since every other control's children are its own parts rather than
        // anything a person reads.
        let mut children = Vec::new();
        if role == ROLE_SYSTEM_LIST {
            let mut count = 0i32;
            if get_count(object, &mut count) >= 0 {
                children = (1..=i64::from(count))
                    .filter(|child| role_of(*child) == ROLE_SYSTEM_LISTITEM)
                    .map(name_of)
                    .collect();
            }
        }
        let release: ReleaseFn = std::mem::transmute(vtable_entry(object, VTBL_RELEASE));
        release(object);
        Ok(Msaa {
            name,
            role,
            children,
        })
    }
}

/// One child of a dialog, as Windows holds it.
#[derive(Debug, Clone, PartialEq)]
struct Control {
    class: String,
    text: String,
    visible: bool,
    msaa: Msaa,
}

fn read_the_controls(dialog: &Dialog) -> Result<Vec<Control>, String> {
    let mut controls = Vec::new();
    for hwnd in descendants_of(dialog.get_handle() as isize) {
        controls.push(Control {
            class: class_name(hwnd),
            text: window_text(hwnd),
            // The control's own style bit, not `IsWindowVisible`, which also
            // asks every parent and so answers false for every control of a
            // dialog that is never shown.
            // SAFETY: a live window handle; the style word is only read.
            visible: unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) } & WS_VISIBLE != 0,
            msaa: msaa_of(hwnd)?,
        });
    }
    Ok(controls)
}

// ── Two keys, made up ──────────────────────────────────────────────────────

const ALICE: &str = "Alice Example <alice@example.com>";
const ALICES_FINGERPRINT: &str = "6EFD87D1527731DE679B8E1BA97E7BB74101FB3E";
const BOB: &str = "Bob Example <bob@example.com>";
const BOBS_FINGERPRINT: &str = "0B1C2D3E4F5A6B7C8D9E0F1A2B3C4D5E6F7A8B9C";

fn a_key(who: &str, fingerprint: &str, private: bool) -> KeyListing {
    KeyListing {
        user_ids: vec![who.to_string()],
        key_id: fingerprint[24..].to_string(),
        fingerprint: fingerprint.to_string(),
        created: chrono::DateTime::parse_from_rfc3339("2026-09-27T10:00:00Z")
            .expect("a date")
            .to_utc(),
        expires: None,
        private,
        locked: false,
        can_encrypt: true,
        can_sign: private,
    }
}

/// What the made-up export hands over for a key: standing in for its public
/// half, and naming the key, so the clipboard can be read for which key it
/// was.
fn the_public_half_of(fingerprint: &str) -> String {
    format!(
        "-----BEGIN PGP PUBLIC KEY BLOCK-----\n{fingerprint}\n-----END PGP PUBLIC KEY BLOCK-----\n"
    )
}

/// Keys held in memory, behind the seam the real ones come through.
fn keys_in_memory(keys: Rc<RefCell<Vec<KeyListing>>>) -> TheKeysUnderneath {
    let listing = keys.clone();
    let importing = keys.clone();
    let removing = keys.clone();
    let exporting = keys;
    TheKeysUnderneath {
        every_key: Box::new(move || Ok(listing.borrow().clone())),
        import: Box::new(move |_text| {
            let bob = a_key(BOB, BOBS_FINGERPRINT, false);
            importing.borrow_mut().push(bob.clone());
            vec![Imported {
                listing: Some(bob),
                said: format!("The public key for {BOB} was kept."),
            }]
        }),
        remove: Box::new(move |fingerprint| {
            let mut keys = removing.borrow_mut();
            let before = keys.len();
            keys.retain(|key| key.fingerprint != fingerprint);
            Ok(keys.len() < before)
        }),
        export_public: Box::new(move |fingerprint| {
            Ok(exporting
                .borrow()
                .iter()
                .find(|key| key.fingerprint == fingerprint)
                .map(|key| the_public_half_of(&key.fingerprint)))
        }),
    }
}

/// The date choices the window is built with: the day first, in numbers.
const DAY_FIRST_IN_NUMBERS: DateSettings = DateSettings {
    style: DateStyle::Absolute,
    order: DateOrder::DayFirst,
    wording: DateWording::Numeric,
    clock: Clock::TwentyFourHour,
};

/// Everything read out of the window session, as plain values.
#[derive(Debug)]
struct Harvest {
    on_open: Vec<Control>,
    first_rows_created: String,
    first_rows_expires: String,
    status_after_copy: String,
    clipboard: Vec<String>,
    questions: Vec<String>,
    rows_after_no: Vec<String>,
    status_after_no: String,
    rows_after_yes: Vec<String>,
    status_after_yes: String,
    status_after_nothing_chosen: String,
    rows_after_import: Vec<String>,
    status_after_import: String,
    paste_controls: Vec<Control>,
}

/// The names of the list's rows as MSAA gives them.
fn rows_of(dialog: &Dialog) -> Result<Vec<String>, String> {
    Ok(read_the_controls(dialog)?
        .into_iter()
        .find(|control| control.msaa.role == ROLE_SYSTEM_LIST && control.class != "SysHeader32")
        .map(|list| list.msaa.children)
        .unwrap_or_default())
}

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
                    Accessibility::new().map_err(|why| format!("Accessibility::new: {why}"))?,
                );
                let keys = Rc::new(RefCell::new(vec![
                    a_key(ALICE, ALICES_FINGERPRINT, true),
                    a_key(BOB, BOBS_FINGERPRINT, false),
                ]));
                let questions = Rc::new(RefCell::new(Vec::new()));
                let answers = Rc::new(RefCell::new(VecDeque::from([false, true, true])));
                let clipboard = Rc::new(RefCell::new(Vec::new()));
                let desktop = TheDesktop {
                    ask: Box::new({
                        let questions = questions.clone();
                        move |_, question| {
                            questions.borrow_mut().push(question.to_string());
                            answers.borrow_mut().pop_front().unwrap_or(false)
                        }
                    }),
                    clipboard: Box::new({
                        let clipboard = clipboard.clone();
                        move |text| {
                            clipboard.borrow_mut().push(text.to_string());
                            true
                        }
                    }),
                };

                let manager = build(
                    &frame,
                    keys_in_memory(keys),
                    desktop,
                    DAY_FIRST_IN_NUMBERS,
                    &a11y,
                );
                let on_open = read_the_controls(&manager.dialog)?;
                let first_rows_created = cell(&manager.list, 0, 4);
                let first_rows_expires = cell(&manager.list, 0, 5);

                manager.choose(0);
                manager.copy_the_chosen_key();
                let status_after_copy = manager.status.get_label();

                manager.remove_the_chosen_key();
                let rows_after_no = rows_of(&manager.dialog)?;
                let status_after_no = manager.status.get_label();

                manager.remove_the_chosen_key();
                let rows_after_yes = rows_of(&manager.dialog)?;
                let status_after_yes = manager.status.get_label();

                manager.remove_the_chosen_key();
                manager.copy_the_chosen_key();
                let status_after_nothing_chosen = manager.status.get_label();

                manager.import_text("a key somebody pasted");
                let rows_after_import = rows_of(&manager.dialog)?;
                let status_after_import = manager.status.get_label();
                manager.dialog.destroy();

                let paste = build_the_paste_dialog(&frame);
                let paste_controls = read_the_controls(&paste.dialog)?;
                paste.dialog.destroy();

                let clipboard = clipboard.borrow().clone();
                let questions = questions.borrow().clone();
                Ok(Harvest {
                    on_open,
                    first_rows_created,
                    first_rows_expires,
                    status_after_copy,
                    clipboard,
                    questions,
                    rows_after_no,
                    status_after_no,
                    rows_after_yes,
                    status_after_yes,
                    status_after_nothing_chosen,
                    rows_after_import,
                    status_after_import,
                    paste_controls,
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

/// The controls a keyboard reaches, in the order it reaches them.
fn focusable(controls: &[Control]) -> Vec<&Control> {
    controls
        .iter()
        .filter(|control| control.visible && control.msaa.role != ROLE_SYSTEM_GRIP)
        .filter(|control| control.class != "Static" && control.class != "SysHeader32")
        .collect()
}

fn names_and_roles(controls: &[Control]) -> Vec<(String, i64)> {
    focusable(controls)
        .iter()
        .map(|control| (control.msaa.name.clone(), control.msaa.role))
        .collect()
}

fn wanted(pairs: &[(&str, i64)]) -> Vec<(String, i64)> {
    pairs
        .iter()
        .map(|(name, role)| (name.to_string(), *role))
        .collect()
}

// ── The window ─────────────────────────────────────────────────────────────

#[test]
fn test_every_control_is_named_at_its_own_handle_in_tab_order() {
    assert_eq!(
        names_and_roles(&the_harvest().on_open),
        wanted(&[
            ("What these keys can do here", ROLE_SYSTEM_TEXT),
            ("Keys", ROLE_SYSTEM_LIST),
            ("Import from File...", ROLE_SYSTEM_PUSHBUTTON),
            ("Paste a Key...", ROLE_SYSTEM_PUSHBUTTON),
            ("Export Public Key...", ROLE_SYSTEM_PUSHBUTTON),
            ("Copy Public Key", ROLE_SYSTEM_PUSHBUTTON),
            ("Remove...", ROLE_SYSTEM_PUSHBUTTON),
            ("Close", ROLE_SYSTEM_PUSHBUTTON),
        ])
    );
}

#[test]
fn test_the_rows_dates_follow_the_date_setting_the_window_was_given() {
    // The window was built with the day first, in numbers, so the first key's
    // Created cell is its day on this computer's clock written that way, and a
    // key with no end says Never.
    let made = a_key(ALICE, ALICES_FINGERPRINT, true)
        .created
        .with_timezone(&chrono::Local)
        .format("%d/%m/%Y")
        .to_string();

    assert_eq!(the_harvest().first_rows_created, made);
    assert_eq!(the_harvest().first_rows_expires, "Never");
}

#[test]
fn test_the_limits_are_the_first_thing_in_the_window_and_say_this_builds_sentence() {
    let first = focusable(&the_harvest().on_open)
        .first()
        .map(|control| control.text.clone());

    assert!(!WHAT_KEYS_CAN_DO_HERE.is_empty());
    assert_eq!(first.as_deref(), Some(WHAT_KEYS_CAN_DO_HERE));
}

#[test]
fn test_each_row_is_read_as_the_person_the_key_is_for() {
    let list = focusable(&the_harvest().on_open)
        .into_iter()
        .find(|control| control.msaa.role == ROLE_SYSTEM_LIST)
        .map(|list| list.msaa.children.clone());

    assert_eq!(list, Some(vec![ALICE.to_string(), BOB.to_string()]));
}

#[test]
fn test_the_line_every_answer_goes_to_is_named() {
    let named: Vec<&str> = the_harvest()
        .on_open
        .iter()
        .filter(|control| control.class == "Static")
        .map(|control| control.msaa.name.as_str())
        .collect();

    assert!(named.contains(&"What happened"), "{named:?}");
}

#[test]
fn test_copy_public_key_hands_over_the_chosen_keys_public_half_and_says_so() {
    let harvest = the_harvest();

    assert_eq!(
        harvest.clipboard,
        vec![the_public_half_of(ALICES_FINGERPRINT)]
    );
    assert_eq!(
        harvest.status_after_copy,
        format!("The public key for {ALICE} is on the clipboard.")
    );
}

#[test]
fn test_remove_asks_naming_the_key_and_no_keeps_it() {
    let harvest = the_harvest();

    assert!(
        harvest
            .questions
            .first()
            .is_some_and(|asked| asked.starts_with(&format!("Remove the private key for {ALICE}"))),
        "{:?}",
        harvest.questions
    );
    assert_eq!(
        harvest.rows_after_no,
        vec![ALICE.to_string(), BOB.to_string()]
    );
    assert_eq!(harvest.status_after_no, "Nothing was removed.");
}

#[test]
fn test_remove_answered_yes_takes_the_key_off_the_list_and_says_so() {
    let harvest = the_harvest();

    assert_eq!(harvest.rows_after_yes, vec![BOB.to_string()]);
    assert_eq!(
        harvest.status_after_yes,
        format!("The private key for {ALICE} was removed.")
    );
    assert_eq!(harvest.questions.len(), 3, "{:?}", harvest.questions);
}

#[test]
fn test_a_button_pressed_with_no_key_to_act_on_says_to_choose_one() {
    assert_eq!(
        the_harvest().status_after_nothing_chosen,
        "Choose a key first."
    );
}

#[test]
fn test_an_import_is_said_and_the_list_is_read_again() {
    let harvest = the_harvest();

    assert_eq!(harvest.rows_after_import, vec![BOB.to_string()]);
    assert_eq!(
        harvest.status_after_import,
        format!("The public key for {BOB} was kept.")
    );
}

// ── A key sent as an attachment, read as text ──────────────────────────────
//
// The question and the import happen inside the main window's own update
// handling, with its accounts, database and runtime, which this file does not
// start. So the path is read in `src/presentation/wx_app.rs` as text, and a
// companion plants the fault the reading exists for and is refused.

const WX_APP: &str = "src/presentation/wx_app.rs";

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

fn what_is_wrong_with_the_key_attachment_path(source: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    let offered = code_of(source)
        .matches("UIUpdate::KeyAttachmentOffered(")
        .count();
    if offered < 2 {
        wrong.push(format!(
            "UIUpdate::KeyAttachmentOffered appears {offered} times in code, where a worker \
             sending it and an arm answering it are wanted"
        ));
    }
    let Some(body) = body_of(source, "fn offer_the_key_attachment(").map(code_of) else {
        wrong.push("fn offer_the_key_attachment( is not in the main window".to_string());
        return wrong;
    };
    for wanted in [
        "the_attachment_question(",
        "yes_no_where_enter_answers_no()",
    ] {
        if !body.contains(wanted) {
            wrong.push(format!("the offer does not call {wanted}"));
        }
    }
    match (body.find("ID_YES"), body.find("pgp_keys::import(")) {
        (Some(asked), Some(imported)) if asked < imported => {}
        (_, None) => wrong.push("the offer never imports the key".to_string()),
        _ => wrong.push("the offer imports the key before it asks".to_string()),
    }
    wrong
}

#[test]
fn test_a_key_attachment_is_imported_only_after_a_yes() {
    let found = what_is_wrong_with_the_key_attachment_path(&the_main_window());

    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn test_the_attachment_reading_refuses_an_import_before_the_question() {
    let source = the_main_window();
    let Some(body) = body_of(&source, "fn offer_the_key_attachment(") else {
        panic!("fn offer_the_key_attachment( is not in {WX_APP}, so there is nowhere to plant");
    };
    let Some(opens) = body.find("{\n") else {
        panic!("the offer's body has no opening brace to plant after");
    };
    let mut planted_body = body.to_string();
    planted_body.insert_str(
        opens + 2,
        "    let planted = crate::application::pgp_keys::import(cache, offered.text());\n",
    );
    let planted = source.replacen(body, &planted_body, 1);

    let found = what_is_wrong_with_the_key_attachment_path(&planted);

    assert!(
        found.iter().any(|it| it.contains("before it asks")),
        "the reading accepted an import before the question: {found:?}"
    );
}

#[test]
fn test_the_paste_dialog_names_its_box_and_its_buttons() {
    assert_eq!(
        names_and_roles(&the_harvest().paste_controls),
        wanted(&[
            ("Key text", ROLE_SYSTEM_TEXT),
            ("OK", ROLE_SYSTEM_PUSHBUTTON),
            ("Cancel", ROLE_SYSTEM_PUSHBUTTON),
        ])
    );
}

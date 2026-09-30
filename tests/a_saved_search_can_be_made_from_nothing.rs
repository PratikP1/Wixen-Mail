//! A saved search can be made from nothing (#58 point 3, GAP-09, 13-39).
//!
//! Save This Search with nothing searched, and New Saved Search on the Saved
//! Searches submenu, open New Saved Search, which asks a name and where to
//! look, and then the conditions window, which asks every or any as well as
//! the conditions. Edit Conditions opens the same conditions window on the
//! search's own answer, so an "any" search can become "every".
//!
//! One window session builds both windows with their own builders and reads
//! them the way a screen reader reads native controls: the children in the
//! order Windows holds them, which is the order Tab walks, each over MSAA
//! through `AccessibleObjectFromWindow` at its own handle. The letters are
//! read from the showing labels and buttons, and each letter reading has a
//! companion that plants a clash and is caught by it.
//!
//! The doors are read as text over `what_ships` of the main window's source,
//! each with a companion that plants the fault and is caught. What a changed
//! answer to every or any gives back is asked of the rule the window's loop
//! ends on, since the loop itself is modal.
//!
//! **Nothing here reaches the settings of whoever runs it, or their screen.**
//! The profile is pointed at a directory of its own before anything is built,
//! the windows live on a desktop of this process's own, and nothing is
//! written: neither window is run, only built and read.

#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::saved_searches::{
    EditedConditions, Join, Question, where_a_search_can_look,
};
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::presentation::wx_managers::{
    build_conditions_window, what_the_conditions_window_gives_back,
};
use wixen_mail::presentation::wx_new_saved_search::build_new_saved_search_dialog;
use wxdragon::prelude::*;

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const GWL_STYLE: i32 = -16;
const WS_VISIBLE: isize = 0x1000_0000;
const VT_I4: u16 = 3;
const CHILDID_SELF: i64 = 0;
const WINSTA_ALL_ACCESS: u32 = 0x037F;
const GENERIC_ALL: u32 = 0x1000_0000;

/// MSAA roles (oleacc.h).
const ROLE_SYSTEM_LIST: i64 = 0x21;
const ROLE_SYSTEM_TEXT: i64 = 0x2a;
const ROLE_SYSTEM_PUSHBUTTON: i64 = 0x2b;
const ROLE_SYSTEM_COMBOBOX: i64 = 0x2e;

/// The account the places are offered for, and its folders in the tree's
/// order.
const ACCOUNT: &str = "Work";
const FOLDERS: [&str; 2] = ["INBOX", "INBOX/Accounts"];

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
    fn GetWindowLongPtrW(hwnd: isize, index: i32) -> isize;
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
            wide("wixen-new-saved-search-test").as_ptr(),
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

/// A window's own name, value and role over MSAA, which is what NVDA reads
/// for a native control.
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
        let value = text_at(VTBL_GET_ACC_VALUE);
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
        Ok((name, value, role))
    }
}

/// One child of a window, as Windows holds it.
#[derive(Debug, Clone, PartialEq)]
struct Control {
    class: String,
    text: String,
    visible: bool,
    name: String,
    value: String,
    role: i64,
}

fn control_at(hwnd: isize) -> Result<Control, String> {
    let (name, value, role) = msaa_of(hwnd)?;
    // SAFETY: a live window handle; the style word is only read.
    let style = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) };
    Ok(Control {
        class: win32_text(hwnd, GetClassNameW),
        text: win32_text(hwnd, GetWindowTextW),
        visible: style & WS_VISIBLE != 0,
        name,
        value,
        role,
    })
}

fn read_the_controls(window: &Dialog) -> Result<Vec<Control>, String> {
    descendants_of(window.get_handle() as isize)
        .into_iter()
        .map(control_at)
        .collect()
}

fn asking(field: &str, pattern: &str) -> Question {
    Question {
        field: field.to_string(),
        match_type: "contains".to_string(),
        pattern: pattern.to_string(),
        case_sensitive: false,
    }
}

fn two_questions() -> Vec<Question> {
    vec![asking("from", "billing"), asking("subject", "overdue")]
}

/// Everything read out of the window session, as plain values.
#[derive(Debug)]
struct Harvest {
    new_window: Vec<Control>,
    new_window_with_a_second_n: Vec<Control>,
    places_offered: Vec<String>,
    place_chosen: Option<u32>,
    conditions: Vec<Control>,
    conditions_with_a_second_m: Vec<Control>,
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
                let folders: Vec<String> = FOLDERS.iter().map(|path| path.to_string()).collect();

                let new = build_new_saved_search_dialog(
                    &frame,
                    &where_a_search_can_look(ACCOUNT, &folders),
                    None,
                );
                let new_window = read_the_controls(&new.dialog)?;
                let places_offered = (0..new.look_in.get_count())
                    .filter_map(|place| new.look_in.get_string(place))
                    .collect();
                let place_chosen = new.look_in.get_selection();
                // The companion: a second label on N, read the same way.
                StaticText::builder(&new.dialog)
                    .with_label("&Nothing here:")
                    .build();
                let new_window_with_a_second_n = read_the_controls(&new.dialog)?;
                new.dialog.destroy();

                let window =
                    build_conditions_window(&frame, "Bills", &two_questions(), Join::Any, None);
                let conditions = read_the_controls(&window.dialog)?;
                // The companion: a second label on M, read the same way.
                StaticText::builder(&window.dialog)
                    .with_label("&Mine:")
                    .build();
                let conditions_with_a_second_m = read_the_controls(&window.dialog)?;
                window.dialog.destroy();

                Ok(Harvest {
                    new_window,
                    new_window_with_a_second_n,
                    places_offered,
                    place_chosen,
                    conditions,
                    conditions_with_a_second_m,
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
fn letters(controls: &[Control]) -> Vec<(char, Vec<String>)> {
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
    claimed.into_iter().collect()
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
/// and role. The list's header is a window of its own inside the list, and
/// Tab never lands on it.
fn names_and_roles(controls: &[Control]) -> Vec<(String, i64)> {
    controls
        .iter()
        .filter(|control| {
            control.visible && control.class != "Static" && control.class != "SysHeader32"
        })
        .map(|control| (control.name.clone(), control.role))
        .collect()
}

fn the_choice_in(controls: &[Control]) -> Option<&Control> {
    controls.iter().find(|control| control.class == "ComboBox")
}

// ── New Saved Search ───────────────────────────────────────────────────────

/// Each field's name ends in the comma `name_from_label` puts where its
/// label's colon was, which is the pause a screen reader makes before the
/// role; every labelled field in the program is named that way.
#[test]
fn test_the_new_window_names_its_name_box_and_its_look_in_choice_on_msaa_in_tab_order() {
    assert_eq!(
        names_and_roles(&the_harvest().new_window),
        vec![
            ("Name for this search,".to_string(), ROLE_SYSTEM_TEXT),
            ("Look in,".to_string(), ROLE_SYSTEM_COMBOBOX),
            ("OK".to_string(), ROLE_SYSTEM_PUSHBUTTON),
            ("Cancel".to_string(), ROLE_SYSTEM_PUSHBUTTON),
        ]
    );
}

#[test]
fn test_the_new_window_offers_everywhere_in_the_account_first_then_its_folders() {
    let harvest = the_harvest();

    assert_eq!(
        harvest.places_offered,
        ["Everywhere in Work", "INBOX", "INBOX/Accounts"]
    );
    assert_eq!(harvest.place_chosen, Some(0));
    assert_eq!(
        the_choice_in(&harvest.new_window).map(|choice| choice.value.as_str()),
        Some("Everywhere in Work"),
        "the choice does not say its answer over MSAA"
    );
}

#[test]
fn test_the_new_windows_letters_are_n_for_the_name_and_l_for_where_to_look() {
    assert_eq!(
        letters(&the_harvest().new_window),
        vec![
            ('L', vec!["&Look in:".to_string()]),
            ('N', vec!["&Name for this search:".to_string()]),
        ]
    );
}

#[test]
fn test_the_new_windows_letter_reading_sees_a_second_n_when_one_is_planted() {
    // The companion. A reading that never found a clash would pass the case
    // above whatever the window did.
    assert_eq!(
        claimed_twice(&the_harvest().new_window_with_a_second_n),
        vec!["Alt+N on &Name for this search: and &Nothing here:".to_string()]
    );
}

#[test]
fn test_the_new_window_says_what_comes_next() {
    let shown: Vec<&str> = the_harvest()
        .new_window
        .iter()
        .filter(|control| control.visible && control.class == "Static")
        .map(|control| control.text.as_str())
        .collect();

    assert!(
        shown.contains(&"Next, add the conditions a message has to meet."),
        "{shown:?}"
    );
}

// ── The conditions window ──────────────────────────────────────────────────

#[test]
fn test_the_conditions_window_names_its_every_or_any_choice_after_the_list() {
    // After the list in the order Tab walks, which is also where it sits on
    // screen, so focus still opens on the list and comes back to it after
    // each Add and Edit.
    assert_eq!(
        names_and_roles(&the_harvest().conditions),
        vec![
            ("Conditions".to_string(), ROLE_SYSTEM_LIST),
            (
                "Find messages that match,".to_string(),
                ROLE_SYSTEM_COMBOBOX
            ),
        ]
    );
}

#[test]
fn test_the_conditions_window_opens_on_the_searchs_own_answer() {
    assert_eq!(
        the_choice_in(&the_harvest().conditions).map(|choice| choice.value.as_str()),
        Some("any condition")
    );
}

#[test]
fn test_the_conditions_windows_choice_takes_m_and_nothing_else_does() {
    // The loop adds Add, Edit, Delete and Close on a, e, d and c; M is the
    // one letter the window claims before them.
    assert_eq!(
        letters(&the_harvest().conditions),
        vec![('M', vec!["Find messages that &match:".to_string()])]
    );
}

#[test]
fn test_the_conditions_windows_letter_reading_sees_a_second_m_when_one_is_planted() {
    assert_eq!(
        claimed_twice(&the_harvest().conditions_with_a_second_m),
        vec!["Alt+M on Find messages that &match: and &Mine:".to_string()]
    );
}

// ── What the conditions window gives back ──────────────────────────────────

#[test]
fn test_a_join_changed_on_its_own_is_given_back_as_a_change() {
    // The whole reason the join is in the window: an "any" search becomes
    // "every" with no condition touched, and that is written.
    assert_eq!(
        what_the_conditions_window_gives_back(false, Join::Any, Join::All, two_questions()),
        Some(EditedConditions {
            join: Join::All,
            questions: two_questions(),
        })
    );
}

#[test]
fn test_changed_conditions_are_given_back_with_the_join_the_window_holds() {
    assert_eq!(
        what_the_conditions_window_gives_back(true, Join::Any, Join::Any, two_questions()),
        Some(EditedConditions {
            join: Join::Any,
            questions: two_questions(),
        })
    );
}

#[test]
fn test_a_window_closed_with_nothing_changed_gives_back_nothing() {
    // Nothing given back is nothing written, so opening a search's
    // conditions to read them moves no date.
    assert_eq!(
        what_the_conditions_window_gives_back(false, Join::All, Join::All, two_questions()),
        None
    );
}

// ── The doors, read as text ────────────────────────────────────────────────

const THE_WINDOW: &str = "src/presentation/wx_app.rs";
const THE_FLOW: &str = "fn make_a_saved_search_from_nothing(";
const THE_CALL: &str = "make_a_saved_search_from_nothing(";

fn the_shipping_window() -> String {
    let whole = std::fs::read_to_string(THE_WINDOW)
        .unwrap_or_else(|why| panic!("{THE_WINDOW}: {why}"))
        .replace("\r\n", "\n");
    what_ships(&whole)
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

/// One arm of the command dispatch, from its guard to the next arm's.
fn arm_of<'a>(source: &'a str, guard: &str) -> Option<&'a str> {
    let start = source.find(guard)?;
    let rest = &source[start + guard.len()..];
    let end = rest.find("_ if id == ").unwrap_or(rest.len());
    Some(&rest[..end])
}

/// What is wrong with Save This Search when nothing was searched.
fn what_is_wrong_with_save_this_search(source: &str) -> Vec<String> {
    let Some(save) = body_of(source, "fn save_this_search(").map(code_of) else {
        return vec!["save_this_search is not in the main window".to_string()];
    };
    let mut wrong = Vec::new();
    if !save.contains(THE_CALL) {
        wrong.push("Save This Search never opens New Saved Search".to_string());
    }
    if save.contains("Search your mail first") {
        wrong.push("Save This Search still refuses when nothing was searched".to_string());
    }
    wrong
}

/// What is wrong with New Saved Search on the Saved Searches submenu.
fn what_is_wrong_with_the_menu_door(source: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    match body_of(source, "pub fn rebuild_the_saved_search_menu(").map(code_of) {
        Some(menu)
            if menu.contains("ID_NEW_SAVED_SEARCH")
                && menu.contains("\"&New Saved Search...\"") => {}
        _ => wrong.push("the Saved Searches submenu has no New Saved Search".to_string()),
    }
    match arm_of(source, "_ if id == ID_NEW_SAVED_SEARCH =>") {
        Some(arm) if arm.contains(THE_CALL) => {}
        _ => wrong.push("New Saved Search on the menu reaches nothing".to_string()),
    }
    wrong
}

/// What is wrong with the order the flow refuses and writes in.
fn what_is_wrong_with_the_flow(source: &str) -> Vec<String> {
    let Some(flow) = body_of(source, THE_FLOW).map(code_of) else {
        return vec![format!("{THE_FLOW} is not in the main window")];
    };
    match (
        flow.find("a_search_from_nothing("),
        flow.find("create_saved_search("),
    ) {
        (Some(refusal), Some(write)) if refusal < write => Vec::new(),
        (_, None) => vec!["the flow never writes the search".to_string()],
        _ => vec!["the flow writes before a search that asks nothing is refused".to_string()],
    }
}

/// What is wrong with Edit Conditions' answer to every or any.
fn what_is_wrong_with_edit_conditions(source: &str) -> Vec<String> {
    let opened = body_of(source, "fn edit_the_chosen_searchs_conditions(")
        .map(code_of)
        .and_then(|edit| {
            let start = edit.find("show_rule_manager_dialog(")?;
            let rest = &edit[start..];
            rest.find(");").map(|end| rest[..end].to_string())
        });
    match opened {
        Some(call) if call.contains("search.join") => Vec::new(),
        Some(_) => vec!["Edit Conditions opens the window on some other answer".to_string()],
        None => vec!["Edit Conditions never opens the conditions window".to_string()],
    }
}

/// The source with one item's body changed, so a planted fault lands where
/// the reading looks rather than at the first match in the file.
fn with_a_fault_in(source: &str, opening: &str, from: &str, to: &str) -> String {
    let body = body_of(source, opening).unwrap_or_else(|| panic!("{opening} is not there"));
    source.replacen(body, &body.replacen(from, to, 1), 1)
}

#[test]
fn test_save_this_search_with_nothing_searched_opens_new_saved_search() {
    let found = what_is_wrong_with_save_this_search(&the_shipping_window());

    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn test_the_save_reading_sees_the_old_refusal_put_back() {
    let planted = with_a_fault_in(
        &the_shipping_window(),
        "fn save_this_search(",
        THE_CALL,
        "refuse_a_command(tx, \"Search your mail first, and then this will keep that search.\"); (",
    );

    assert_eq!(
        what_is_wrong_with_save_this_search(&planted),
        [
            "Save This Search never opens New Saved Search",
            "Save This Search still refuses when nothing was searched",
        ]
    );
}

#[test]
fn test_new_saved_search_is_on_the_menu_and_its_arm_opens_the_flow() {
    let found = what_is_wrong_with_the_menu_door(&the_shipping_window());

    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn test_the_menu_reading_sees_an_item_and_an_arm_that_reach_nothing() {
    let source = the_shipping_window();
    let without_the_item = with_a_fault_in(
        &source,
        "pub fn rebuild_the_saved_search_menu(",
        "\"&New Saved Search...\"",
        "\"&Something Else...\"",
    );
    let arm = arm_of(&source, "_ if id == ID_NEW_SAVED_SEARCH =>").unwrap_or_default();
    let planted = without_the_item.replacen(arm, &arm.replacen(THE_CALL, "nothing_at_all(", 1), 1);

    assert_eq!(
        what_is_wrong_with_the_menu_door(&planted),
        [
            "the Saved Searches submenu has no New Saved Search",
            "New Saved Search on the menu reaches nothing",
        ]
    );
}

#[test]
fn test_a_search_from_nothing_is_refused_before_it_is_written() {
    let found = what_is_wrong_with_the_flow(&the_shipping_window());

    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn test_the_flow_reading_sees_a_write_before_the_refusal() {
    let planted = with_a_fault_in(
        &the_shipping_window(),
        THE_FLOW,
        "{\n",
        "{\n    let _ = cache.create_saved_search(&account_id, &nothing);\n",
    );

    assert_eq!(
        what_is_wrong_with_the_flow(&planted),
        ["the flow writes before a search that asks nothing is refused"]
    );
}

#[test]
fn test_edit_conditions_opens_the_window_on_the_searchs_own_answer() {
    let found = what_is_wrong_with_edit_conditions(&the_shipping_window());

    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn test_the_edit_reading_sees_the_window_opened_on_another_answer() {
    let planted = with_a_fault_in(
        &the_shipping_window(),
        "fn edit_the_chosen_searchs_conditions(",
        "search.join,",
        "Join::All,",
    );

    assert_eq!(
        what_is_wrong_with_edit_conditions(&planted),
        ["Edit Conditions opens the window on some other answer"]
    );
}

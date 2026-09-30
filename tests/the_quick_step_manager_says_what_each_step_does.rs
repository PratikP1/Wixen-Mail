//! The Quick Step Manager says what each step does (#60, GAP-11, 13-41).
//!
//! #60 asked for "a manager beside the Filter Manager". Pratik's settlement of
//! 2026-09-24 puts it inside the Action menu, under Quick Steps, as Manage
//! Quick Steps. 13-40 built the rules and the store; this is the window
//! somebody works by keyboard to name a step, say what it does and put the
//! steps in the order their keys follow.
//!
//! **What is read.** A store built in the test with one account holding four
//! steps, one of them written with an action word no build knows. The
//! manager is built over them and its rows read off the live list, cell by
//! cell: the name, the key the step's place gives it and what it does. A move
//! is made in it the way Move Down and Alt+Shift+Down make one, the rows it
//! hands back are saved the way the manager's close saves them, and the
//! store's order is read back.
//!
//! The step editor is built over a step and read the way a screen reader
//! reads native controls: its children in the order Windows holds them, which
//! is the order Tab walks, each over MSAA at its own handle, and its letters
//! off the showing labels. Its controls are set and read back into a step,
//! and the decision that keeps or refuses a step is asked. The real menu
//! bar's Action menu is read for Quick Steps, and the main window's source
//! for the arm Manage Quick Steps runs.
//!
//! **Companions.** Each check is handed a wrong state, a Key column that
//! gives a fourth step a key, a store whose order was never written, a
//! question with no name, a second N, Mark unread read as read, Action
//! without Quick Steps and an arm that reaches nothing, and refuses it, so a
//! check that passes is one that could have failed.
//!
//! **Nothing here reaches the settings of whoever runs it, or their screen.**
//! The profile is pointed at a directory of its own before anything is built,
//! the windows live on a desktop of this process's own, and the store is a
//! temporary directory. One window session for the file, every reading
//! sharing it through a `OnceLock`.

#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::filters::Outcome;
use wixen_mail::application::quick_steps::{
    QUICK_STEPS_ARE_EXPERIMENTAL, QuickStep, StoredStep, key_for, name_for,
    what_stops_a_step_being_saved,
};
use wixen_mail::application::reordering::Move;
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::data::message_cache::MessageCache;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::managers::save_what_the_quick_step_manager_returned;
use wixen_mail::presentation::wx_app::WxMailApp;
use wixen_mail::presentation::wx_managers::{
    ManagerState, QuickStepEditor, QuickStepEntry, StepRefused, build_quick_step_edit_dialog,
    build_quick_step_manager, move_the_chosen_row, populate_quick_steps, what_the_editor_holds,
    what_the_editor_keeps,
};
use wxdragon::prelude::*;

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const GWL_STYLE: i32 = -16;
const WS_VISIBLE: isize = 0x1000_0000;
const VT_I4: u16 = 3;
const CHILDID_SELF: i64 = 0;
const WINSTA_ALL_ACCESS: u32 = 0x037F;
const GENERIC_ALL: u32 = 0x1000_0000;

/// commctrl.h: `LVM_FIRST + 115` and `LVM_FIRST + 95`.
const LVM_GETITEMTEXTW: u32 = 0x1000 + 115;
const LVM_GETCOLUMNW: u32 = 0x1000 + 95;
const LVCF_TEXT: u32 = 0x4;

/// MSAA roles (oleacc.h).
const ROLE_SYSTEM_TEXT: i64 = 0x2a;
const ROLE_SYSTEM_PUSHBUTTON: i64 = 0x2b;
const ROLE_SYSTEM_CHECKBUTTON: i64 = 0x2c;
const ROLE_SYSTEM_COMBOBOX: i64 = 0x2e;

/// The account the steps are kept under.
const ACCOUNT: &str = "acct-quick";

/// The step whose action word no build knows, and the word.
const FROM_THE_FUTURE: &str = "From the future";
const A_WORD_NO_BUILD_KNOWS: &str = "sing_a_song";

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

/// commctrl.h's `LVCOLUMNW`, of which `LVM_GETCOLUMNW` reads the heading.
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

#[link(name = "user32")]
unsafe extern "system" {
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
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
            wide("wixen-quick-step-manager-test").as_ptr(),
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

/// A window's children in the order Windows holds them, which is the order
/// Tab walks, each read over MSAA at its own handle.
fn read_the_controls(window: &Dialog) -> Result<Vec<Control>, String> {
    descendants_of(window.get_handle() as isize)
        .into_iter()
        .map(control_at)
        .collect()
}

/// One cell of a live list, read from the list itself, whole: not through
/// `ListCtrl::get_item_text`, which loses a cell's last character
/// (`tests/manager_dialog_labels.rs` measured it).
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

/// A column's heading, read from the list itself.
fn heading(list: &ListCtrl, column: usize) -> String {
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
            list.get_handle() as isize,
            LVM_GETCOLUMNW,
            column,
            &mut asked as *mut ListViewColumn as isize,
        )
    };
    let end = buffer.iter().position(|c| *c == 0).unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..end])
}

/// One row of the manager's list as the three cells it shows.
#[derive(Debug, Clone, PartialEq)]
struct Row {
    name: String,
    key: String,
    does: String,
}

fn the_rows(list: &ListCtrl) -> Vec<Row> {
    (0..list.get_item_count() as i64)
        .map(|row| Row {
            name: cell(list, row, 0),
            key: cell(list, row, 1),
            does: cell(list, row, 2),
        })
        .collect()
}

// ── The store ─────────────────────────────────────────────────────────────

fn a_step(id: &str, name: &str, does: Outcome) -> QuickStep {
    QuickStep {
        id: id.to_string(),
        name: name.to_string(),
        does,
    }
}

/// The four steps in the order they are made, which is the order they keep.
fn four_steps() -> Vec<QuickStep> {
    vec![
        a_step(
            "step-archive",
            "Archive and read",
            Outcome {
                read: Some(true),
                move_to: Some("INBOX/Archive".to_string()),
                ..Outcome::default()
            },
        ),
        a_step(
            "step-later",
            "Flag for later",
            Outcome {
                starred: Some(true),
                tags: vec!["Later".to_string()],
                ..Outcome::default()
            },
        ),
        a_step(
            "step-future",
            FROM_THE_FUTURE,
            Outcome {
                read: Some(true),
                ..Outcome::default()
            },
        ),
        a_step(
            "step-bin",
            "Bin it",
            Outcome {
                delete: true,
                ..Outcome::default()
            },
        ),
    ]
}

/// The store, with the third step's action rewritten in a word no build
/// knows, the way a newer version would leave it, then opened again.
fn a_store(at: &Path) -> Result<MessageCache, String> {
    {
        let cache = MessageCache::new(at.to_path_buf(), None).map_err(|e| format!("{e}"))?;
        for step in four_steps() {
            cache
                .create_quick_step(ACCOUNT, &step)
                .map_err(|e| format!("the step {}: {e}", step.name))?;
        }
    }
    let written_later = rusqlite::Connection::open(at.join("message_cache.db"))
        .map_err(|e| format!("the store's file: {e}"))?;
    written_later
        .execute(
            "UPDATE quick_step_actions SET action_type = ?1 WHERE step_id = 'step-future'",
            [A_WORD_NO_BUILD_KNOWS],
        )
        .map_err(|e| format!("the newer version's word: {e}"))?;
    drop(written_later);
    MessageCache::new(at.to_path_buf(), None).map_err(|e| format!("{e}"))
}

/// What the store says of the newer version's step.
fn the_newer_step(cache: &MessageCache) -> String {
    let steps = cache
        .get_quick_steps_for_account(ACCOUNT)
        .unwrap_or_default();
    match steps.iter().find(|step| step.id() == "step-future") {
        Some(StoredStep::WrittenByANewerVersion { .. }) => "written by a newer version",
        Some(StoredStep::Readable(_)) => "readable by this build",
        None => "gone",
    }
    .to_string()
}

fn the_stored_order(cache: &MessageCache) -> Vec<String> {
    cache
        .get_quick_steps_for_account(ACCOUNT)
        .map(|steps| steps.iter().map(|step| step.name().to_string()).collect())
        .unwrap_or_else(|e| vec![format!("the steps could not be read: {e}")])
}

// ── The session ───────────────────────────────────────────────────────────

/// Everything read out of the window session, as plain values.
#[derive(Debug)]
struct Harvest {
    title: String,
    list_name: String,
    headings: Vec<String>,
    rows: Vec<Row>,
    rows_after_a_move: Vec<Row>,
    what_the_move_said: String,
    order_before_the_close: Vec<String>,
    order_after_the_close: Vec<String>,
    what_the_save_failed_on: Vec<String>,
    the_newer_step_after_the_close: String,
    editor: EditorReading,
    menu: MenuReading,
}

/// The Action menu of the real menu bar the window is given.
#[derive(Debug, Default, Clone)]
struct MenuReading {
    /// Every item on Action, by its label.
    action_items: Vec<String>,
    /// What the Quick Steps item on Action says where it is chosen.
    quick_steps_help: String,
    /// The Quick Steps submenu's items as label and help, `None` when
    /// Action has no such submenu.
    quick_steps_items: Option<Vec<(String, String)>>,
}

fn read_the_menu(frame: &Frame) -> MenuReading {
    frame.set_menu_bar(WxMailApp::build_menu_bar());
    let Some(action) = frame.get_menu_bar().and_then(|bar| {
        usize::try_from(bar.find_menu("Action"))
            .ok()
            .and_then(|at| bar.get_menu(at))
    }) else {
        return MenuReading::default();
    };
    let items = action.get_menu_items();
    let quick_steps = items.iter().find(|item| item.get_label() == "&Quick Steps");
    MenuReading {
        action_items: items.iter().map(|item| item.get_label()).collect(),
        quick_steps_help: quick_steps
            .map(|item| action.get_help_string(item.get_item_id()))
            .unwrap_or_default(),
        quick_steps_items: quick_steps.and_then(|item| item.get_sub_menu()).map(|sub| {
            sub.get_menu_items()
                .iter()
                .map(|item| (item.get_label(), sub.get_help_string(item.get_item_id())))
                .collect()
        }),
    }
}

/// What the step editor's controls hold, as plain values.
#[derive(Debug, Default, PartialEq)]
struct Filled {
    name: String,
    read: Option<u32>,
    flag: Option<u32>,
    label: Option<u32>,
    move_to: Option<u32>,
    delete: bool,
    phrase: String,
}

/// Everything read out of the step editor.
#[derive(Debug, Default)]
struct EditorReading {
    new_title: String,
    existing_title: String,
    controls: Vec<Control>,
    with_a_second_n: Vec<Control>,
    offered: Vec<Vec<String>>,
    filled: Filled,
    gone_offered: Vec<Vec<String>>,
    gone_filled: Filled,
    gone_holds: Outcome,
    together: (String, Outcome),
    delete_alone: Outcome,
    a_phrase: Outcome,
}

/// The account's labels and folders the editor offers, in their order.
fn the_labels() -> Vec<String> {
    vec!["Work".to_string(), "Later".to_string()]
}

fn the_folders() -> Vec<String> {
    vec!["INBOX".to_string(), "INBOX/Receipts".to_string()]
}

/// A step that marks read, moves to a folder the account has, and says a
/// phrase first.
fn a_receipts_step() -> QuickStepEntry {
    QuickStepEntry {
        id: "step-receipts".to_string(),
        name: "Receipts".to_string(),
        does: Some(Outcome {
            read: Some(true),
            move_to: Some("INBOX/Receipts".to_string()),
            say_first: Some("Receipt".to_string()),
            ..Outcome::default()
        }),
    }
}

/// A step whose label and folder the account no longer has.
fn a_step_from_before() -> QuickStepEntry {
    QuickStepEntry {
        id: "step-before".to_string(),
        name: "Old ways".to_string(),
        does: Some(Outcome {
            tags: vec!["Old".to_string()],
            move_to: Some("Gone".to_string()),
            ..Outcome::default()
        }),
    }
}

fn entries(choice: &Choice) -> Vec<String> {
    (0..choice.get_count())
        .filter_map(|at| choice.get_string(at))
        .collect()
}

fn filled(editor: &QuickStepEditor) -> Filled {
    Filled {
        name: editor.name.get_value(),
        read: editor.read.get_selection(),
        flag: editor.flag.get_selection(),
        label: editor.label.get_selection(),
        move_to: editor.move_to.get_selection(),
        delete: editor.delete.get_value(),
        phrase: editor.phrase.get_value(),
    }
}

/// Choose an entry the way somebody would, where there is one to choose.
fn choose(choice: &Choice, at: u32) {
    if at < choice.get_count() {
        choice.set_selection(at);
    }
}

fn read_the_editor(frame: &Frame) -> Result<EditorReading, String> {
    let (labels, folders) = (the_labels(), the_folders());

    let existing =
        build_quick_step_edit_dialog(frame, Some(&a_receipts_step()), &labels, &folders, None);
    let existing_title = existing.dialog.get_label().unwrap_or_default();
    let controls = read_the_controls(&existing.dialog)?;
    let offered = [
        &existing.read,
        &existing.flag,
        &existing.label,
        &existing.move_to,
    ]
    .into_iter()
    .map(entries)
    .collect();
    let filled_in = filled(&existing);
    // The companion: a second label on N, read the same way.
    StaticText::builder(&existing.dialog)
        .with_label("&Nothing here:")
        .build();
    let with_a_second_n = read_the_controls(&existing.dialog)?;
    existing.dialog.destroy();

    let gone =
        build_quick_step_edit_dialog(frame, Some(&a_step_from_before()), &labels, &folders, None);
    let gone_offered = [&gone.label, &gone.move_to]
        .into_iter()
        .map(entries)
        .collect();
    let gone_filled = filled(&gone);
    let (_, gone_holds) = what_the_editor_holds(&gone);
    gone.dialog.destroy();

    let together = build_quick_step_edit_dialog(frame, None, &labels, &folders, None);
    let new_title = together.dialog.get_label().unwrap_or_default();
    together.name.set_value("Sort it");
    choose(&together.read, 2);
    choose(&together.flag, 1);
    choose(&together.label, 1);
    choose(&together.move_to, 1);
    let held_together = what_the_editor_holds(&together);
    together.dialog.destroy();

    let deleting = build_quick_step_edit_dialog(frame, None, &labels, &folders, None);
    deleting.delete.set_value(true);
    let (_, delete_alone) = what_the_editor_holds(&deleting);
    deleting.dialog.destroy();

    let saying = build_quick_step_edit_dialog(frame, None, &labels, &folders, None);
    saying.phrase.set_value("Urgent");
    let (_, a_phrase) = what_the_editor_holds(&saying);
    saying.dialog.destroy();

    Ok(EditorReading {
        new_title,
        existing_title,
        controls,
        with_a_second_n,
        offered,
        filled: filled_in,
        gone_offered,
        gone_filled,
        gone_holds,
        together: held_together,
        delete_alone,
        a_phrase,
    })
}

fn read_the_manager(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
    cache: &MessageCache,
) -> Result<Harvest, String> {
    let stored = cache
        .get_quick_steps_for_account(ACCOUNT)
        .map_err(|e| format!("the steps: {e}"))?;
    let rows: Vec<QuickStepEntry> = stored.iter().map(QuickStepEntry::from).collect();
    let widgets = build_quick_step_manager(frame, &rows, None);
    let title = widgets.dialog.get_label().unwrap_or_default();
    let (list_name, _, _) = msaa_of(widgets.list.get_handle() as isize)?;
    let headings = (0..widgets.list.get_column_count().max(0) as usize)
        .map(|column| heading(&widgets.list, column))
        .collect();
    let first_rows = the_rows(&widgets.list);

    // The cursor on the first row, Archive and read, and Move Down, as
    // Alt+Shift+Down and the Move Down button both do.
    let state = Rc::new(RefCell::new(ManagerState {
        working: rows,
        changed: false,
    }));
    widgets.list.set_item_state(
        0,
        ListItemState::Selected | ListItemState::Focused,
        ListItemState::Selected | ListItemState::Focused,
    );
    move_the_chosen_row(
        &state,
        &widgets.list,
        &widgets.status,
        a11y,
        populate_quick_steps,
        |row: &QuickStepEntry| row.name.clone(),
        Move::Down,
    );
    let rows_after_a_move = the_rows(&widgets.list);
    let what_the_move_said = widgets.status.get_label();

    let order_before_the_close = the_stored_order(cache);
    let handed_back = state.borrow().working.clone();
    let what_the_save_failed_on =
        save_what_the_quick_step_manager_returned(cache, ACCOUNT, &stored, handed_back);
    let order_after_the_close = the_stored_order(cache);
    let the_newer_step_after_the_close = the_newer_step(cache);
    widgets.dialog.destroy();

    Ok(Harvest {
        title,
        list_name,
        headings,
        rows: first_rows,
        rows_after_a_move,
        what_the_move_said,
        order_before_the_close,
        order_after_the_close,
        what_the_save_failed_on,
        the_newer_step_after_the_close,
        editor: EditorReading::default(),
        menu: MenuReading::default(),
    })
}

fn take_the_harvest() -> Result<Harvest, String> {
    if std::mem::size_of::<Variant>() != 24 {
        return Err("VARIANT is not 24 bytes here, so the reader's layout is wrong".to_string());
    }
    let profile = tempfile::tempdir().map_err(|why| format!("a profile of its own: {why}"))?;
    let store_at = tempfile::tempdir().map_err(|why| format!("a store directory: {why}"))?;
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
                let a11y = Arc::new(
                    Accessibility::new().map_err(|why| format!("accessibility: {why:?}"))?,
                );
                let cache = a_store(&store_at)?;
                let mut harvest = read_the_manager(&frame, &a11y, &cache)?;
                harvest.editor = read_the_editor(&frame)?;
                harvest.menu = read_the_menu(&frame);
                frame.destroy();
                Ok(harvest)
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
    drop(store_at);
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

// ── The manager ───────────────────────────────────────────────────────────

/// Every row whose Key cell is not the key its place gives it: the first
/// three carry Ctrl+Shift+7 to Ctrl+Shift+9 and the rest carry nothing.
fn what_is_wrong_with_the_keys(rows: &[Row]) -> Vec<String> {
    rows.iter()
        .enumerate()
        .filter_map(|(at, row)| {
            let wanted = key_for(at + 1).unwrap_or_default();
            (row.key != wanted).then(|| {
                format!(
                    "{} at place {} shows {:?} where its key is {wanted:?}",
                    row.name,
                    at + 1,
                    row.key
                )
            })
        })
        .collect()
}

/// Where the store's order differs from the order the manager handed back.
fn what_is_wrong_with_the_stored_order(stored: &[String], handed_back: &[Row]) -> Vec<String> {
    let wanted: Vec<&str> = handed_back.iter().map(|row| row.name.as_str()).collect();
    match stored.iter().map(String::as_str).eq(wanted.iter().copied()) {
        true => Vec::new(),
        false => vec![format!(
            "the store holds {stored:?} where the manager handed back {wanted:?}"
        )],
    }
}

fn row(name: &str, key: &str, does: &str) -> Row {
    Row {
        name: name.to_string(),
        key: key.to_string(),
        does: does.to_string(),
    }
}

#[test]
fn test_the_manager_is_titled_and_its_list_named_for_quick_steps() {
    let harvest = the_harvest();

    assert_eq!(harvest.title, "Quick Step Manager");
    assert_eq!(harvest.list_name, "Quick Steps");
    assert_eq!(harvest.headings, ["Name", "Key", "What it does"]);
}

#[test]
fn test_the_manager_lists_the_steps_in_their_order_with_their_keys_and_what_each_does() {
    assert_eq!(
        the_harvest().rows,
        [
            row(
                "Archive and read",
                "Ctrl+Shift+7",
                "Mark read, move to INBOX/Archive"
            ),
            row("Flag for later", "Ctrl+Shift+8", "Flag, label Later"),
            row(
                FROM_THE_FUTURE,
                "Ctrl+Shift+9",
                "Written by a newer version of Wixen Mail; it can be moved or removed here"
            ),
            row("Bin it", "", "Delete"),
        ]
    );
}

#[test]
fn test_the_key_column_gives_the_first_three_steps_their_keys_and_the_fourth_none() {
    let found = what_is_wrong_with_the_keys(&the_harvest().rows);

    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn test_the_key_reading_sees_a_fourth_step_given_a_key() {
    // The companion. A reading that never found a wrong key would pass the
    // case above whatever the column said.
    let mut planted = the_harvest().rows.clone();
    if let Some(fourth) = planted.get_mut(3) {
        fourth.key = "Ctrl+Shift+10".to_string();
    }

    assert_eq!(
        what_is_wrong_with_the_keys(&planted),
        ["Bin it at place 4 shows \"Ctrl+Shift+10\" where its key is \"\""]
    );
}

#[test]
fn test_a_move_in_the_manager_moves_the_row_and_its_key_and_says_where_it_went() {
    let harvest = the_harvest();

    assert_eq!(
        harvest.rows_after_a_move,
        [
            row("Flag for later", "Ctrl+Shift+7", "Flag, label Later"),
            row(
                "Archive and read",
                "Ctrl+Shift+8",
                "Mark read, move to INBOX/Archive"
            ),
            row(
                FROM_THE_FUTURE,
                "Ctrl+Shift+9",
                "Written by a newer version of Wixen Mail; it can be moved or removed here"
            ),
            row("Bin it", "", "Delete"),
        ]
    );
    assert_eq!(harvest.what_the_move_said, "Archive and read, 2 of 4.");
}

#[test]
fn test_the_order_the_manager_hands_back_is_the_order_stored_when_it_closes() {
    let harvest = the_harvest();
    let found = what_is_wrong_with_the_stored_order(
        &harvest.order_after_the_close,
        &harvest.rows_after_a_move,
    );

    assert!(
        harvest.what_the_save_failed_on.is_empty(),
        "{:#?}",
        harvest.what_the_save_failed_on
    );
    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn test_the_order_reading_sees_a_store_whose_order_was_never_written() {
    // The companion: the store as it stood before the close, which is what a
    // save that skipped the order would leave.
    let harvest = the_harvest();

    assert_eq!(
        what_is_wrong_with_the_stored_order(
            &harvest.order_before_the_close,
            &harvest.rows_after_a_move
        )
        .len(),
        1
    );
}

#[test]
fn test_a_step_a_newer_version_wrote_keeps_its_action_through_the_close() {
    // Saved with the rest, and still the newer version's step: nothing this
    // build wrote over the action it cannot read.
    assert_eq!(
        the_harvest().the_newer_step_after_the_close,
        "written by a newer version"
    );
}

// ── The step editor ────────────────────────────────────────────────────────

/// The questions in the order Tab reaches them, each with the name a screen
/// reader hears over MSAA and the role. A labelled field's name ends in the
/// comma `name_from_label` puts where its label's colon was.
fn the_questions() -> Vec<(String, i64)> {
    [
        ("Name,", ROLE_SYSTEM_TEXT),
        ("Mark as read or unread,", ROLE_SYSTEM_COMBOBOX),
        ("Flag,", ROLE_SYSTEM_COMBOBOX),
        ("Label,", ROLE_SYSTEM_COMBOBOX),
        ("Move to,", ROLE_SYSTEM_COMBOBOX),
        ("Delete it", ROLE_SYSTEM_CHECKBUTTON),
        ("Phrase to say first,", ROLE_SYSTEM_TEXT),
        ("OK", ROLE_SYSTEM_PUSHBUTTON),
        ("Cancel", ROLE_SYSTEM_PUSHBUTTON),
    ]
    .into_iter()
    .map(|(name, role)| (name.to_string(), role))
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

/// Every question read under some other name or role than it should be, or
/// the count when the window asks more or fewer.
fn what_is_wrong_with_the_names(controls: &[Control]) -> Vec<String> {
    let (read, wanted) = (names_and_roles(controls), the_questions());
    if read.len() != wanted.len() {
        return vec![format!(
            "{} controls where {} were wanted: {read:?}",
            read.len(),
            wanted.len()
        )];
    }
    read.iter()
        .zip(&wanted)
        .filter(|(got, want)| got != want)
        .map(|((name, role), (want_name, want_role))| {
            format!("{want_name} ({want_role}) is read as {name:?} ({role})")
        })
        .collect()
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

/// Every letter the showing labels, the check box and the buttons claim,
/// with the labels that claim it.
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

/// Which of a step's answers the editor read differently from the answer
/// set, by the question's name.
fn what_the_reading_got_wrong(got: &Outcome, wanted: &Outcome) -> Vec<&'static str> {
    [
        ("read", got.read != wanted.read),
        ("flag", got.starred != wanted.starred),
        ("label", got.tags != wanted.tags),
        ("move", got.move_to != wanted.move_to),
        ("delete", got.delete != wanted.delete),
        ("phrase", got.say_first != wanted.say_first),
    ]
    .into_iter()
    .filter(|(_, differs)| *differs)
    .map(|(question, _)| question)
    .collect()
}

/// Mark unread, flag it, label Work and move to INBOX, set together.
fn marked_flagged_labelled_and_moved() -> Outcome {
    Outcome {
        read: Some(false),
        starred: Some(true),
        tags: vec!["Work".to_string()],
        move_to: Some("INBOX".to_string()),
        ..Outcome::default()
    }
}

#[test]
fn test_the_editor_is_titled_for_a_new_step_and_for_one_being_changed() {
    let editor = &the_harvest().editor;

    assert_eq!(editor.new_title, "New Quick Step");
    assert_eq!(editor.existing_title, "Edit Quick Step");
}

#[test]
fn test_the_editor_names_each_question_on_msaa_in_tab_order() {
    let found = what_is_wrong_with_the_names(&the_harvest().editor.controls);

    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn test_the_name_reading_sees_a_question_left_without_its_name() {
    // The companion: the Flag choice read with no name, which is what a
    // choice nothing named gives a screen reader that finds no label.
    let mut planted = the_harvest().editor.controls.clone();
    if let Some(flag) = planted.iter_mut().find(|control| control.name == "Flag,") {
        flag.name = String::new();
    }

    assert_eq!(
        what_is_wrong_with_the_names(&planted),
        [format!(
            "Flag, ({ROLE_SYSTEM_COMBOBOX}) is read as \"\" ({ROLE_SYSTEM_COMBOBOX})"
        )]
    );
}

#[test]
fn test_the_editor_offers_each_questions_answers_in_order() {
    assert_eq!(
        the_harvest().editor.offered,
        [
            vec!["Leave as it is", "Mark read", "Mark unread"],
            vec!["Leave as it is", "Flag it", "Take the flag off"],
            vec!["No label", "Work", "Later"],
            vec!["Leave it where it is", "INBOX", "INBOX/Receipts"],
        ]
    );
}

#[test]
fn test_the_editor_opens_on_what_the_step_does() {
    assert_eq!(
        the_harvest().editor.filled,
        Filled {
            name: "Receipts".to_string(),
            read: Some(1),
            flag: Some(0),
            label: Some(0),
            move_to: Some(2),
            delete: false,
            phrase: "Receipt".to_string(),
        }
    );
}

#[test]
fn test_a_label_and_a_folder_the_account_no_longer_has_are_shown_chosen_and_kept() {
    // Shown for what they are, so saving the step with them again is a
    // choice somebody sees rather than one made for them.
    let editor = &the_harvest().editor;

    assert_eq!(
        editor.gone_offered,
        [
            vec![
                "No label",
                "Work",
                "Later",
                "Old (not in this account any more)"
            ],
            vec![
                "Leave it where it is",
                "INBOX",
                "INBOX/Receipts",
                "Gone (not in this account any more)"
            ],
        ]
    );
    assert_eq!(
        (editor.gone_filled.label, editor.gone_filled.move_to),
        (Some(3), Some(3))
    );
    assert_eq!(
        (
            editor.gone_holds.tags.clone(),
            editor.gone_holds.move_to.clone()
        ),
        (vec!["Old".to_string()], Some("Gone".to_string()))
    );
}

#[test]
fn test_the_editor_reads_marking_flagging_labelling_and_moving_together() {
    let (name, does) = &the_harvest().editor.together;

    assert_eq!(name, "Sort it");
    assert_eq!(
        what_the_reading_got_wrong(does, &marked_flagged_labelled_and_moved()),
        Vec::<&str>::new()
    );
}

#[test]
fn test_the_reading_sees_mark_unread_read_as_mark_read() {
    // The companion: what a choice mapped the wrong way would hand back.
    let (_, does) = &the_harvest().editor.together;
    let planted = Outcome {
        read: Some(true),
        ..does.clone()
    };

    assert_eq!(
        what_the_reading_got_wrong(&planted, &marked_flagged_labelled_and_moved()),
        ["read"]
    );
}

#[test]
fn test_the_editor_reads_delete_alone() {
    assert_eq!(
        the_harvest().editor.delete_alone,
        Outcome {
            delete: true,
            ..Outcome::default()
        }
    );
}

#[test]
fn test_the_editor_reads_a_phrase_to_say_first() {
    assert_eq!(
        the_harvest().editor.a_phrase,
        Outcome {
            say_first: Some("Urgent".to_string()),
            ..Outcome::default()
        }
    );
}

#[test]
fn test_the_editors_letters_are_seven_and_none_is_claimed_twice() {
    // Allocated once for the window: N, R, F, L, M, D and H, the rule
    // editor's own letter for the phrase. OK and Cancel carry none.
    assert_eq!(
        letters(&the_harvest().editor.controls),
        vec![
            ('D', vec!["&Delete it".to_string()]),
            ('F', vec!["&Flag:".to_string()]),
            ('H', vec!["P&hrase to say first:".to_string()]),
            ('L', vec!["&Label:".to_string()]),
            ('M', vec!["&Move to:".to_string()]),
            ('N', vec!["&Name:".to_string()]),
            ('R', vec!["Mark as &read or unread:".to_string()]),
        ]
    );
}

#[test]
fn test_the_editors_letter_reading_sees_a_second_n_when_one_is_planted() {
    assert_eq!(
        claimed_twice(&the_harvest().editor.with_a_second_n),
        vec!["Alt+N on &Name: and &Nothing here:".to_string()]
    );
}

#[test]
fn test_a_name_another_step_has_is_refused_before_the_step_is_kept() {
    let others = ["Archive and read".to_string()];
    let does = marked_flagged_labelled_and_moved();

    assert_eq!(
        what_the_editor_keeps("archive and read", does, &others),
        Err(StepRefused::TheName(
            name_for("archive and read", &others)
                .why_not()
                .unwrap_or_default()
        ))
    );
}

#[test]
fn test_a_step_that_deletes_and_moves_is_refused_before_it_is_kept() {
    let does = Outcome {
        delete: true,
        move_to: Some("INBOX".to_string()),
        ..Outcome::default()
    };

    assert_eq!(
        what_the_editor_keeps("Tidy up", does.clone(), &[]),
        Err(StepRefused::WhatItDoes(
            what_stops_a_step_being_saved(&does).unwrap_or_default()
        ))
    );
}

// ── Action, Quick Steps, Manage Quick Steps ───────────────────────────────

/// What the item on Action says where Quick Steps is chosen.
const WHAT_QUICK_STEPS_ARE: &str =
    "Commands you make that do several things to the selected messages at once";

/// What is wrong with Action's Quick Steps submenu, in the order a keyboard
/// meets it: where it sits, what it says, and what it holds.
fn what_is_wrong_with_the_menu(menu: &MenuReading) -> Vec<String> {
    let mut wrong = Vec::new();
    let after_saved_searches = menu
        .action_items
        .iter()
        .position(|label| label == "Saved Searc&hes")
        .and_then(|at| menu.action_items.get(at + 1));
    if after_saved_searches.map(String::as_str) != Some("&Quick Steps") {
        wrong.push("Action has no Quick Steps submenu after Saved Searches".to_string());
    }
    if menu.quick_steps_help != WHAT_QUICK_STEPS_ARE {
        wrong.push(format!(
            "Quick Steps says {:?} where it is chosen",
            menu.quick_steps_help
        ));
    }
    let wanted = vec![(
        "&Manage Quick Steps...".to_string(),
        QUICK_STEPS_ARE_EXPERIMENTAL.to_string(),
    )];
    if menu.quick_steps_items.as_ref() != Some(&wanted) {
        wrong.push(format!(
            "the submenu holds {:?} rather than Manage Quick Steps with the experimental \
             sentence",
            menu.quick_steps_items
        ));
    }
    wrong
}

#[test]
fn test_action_holds_quick_steps_after_saved_searches_with_manage_quick_steps_in_it() {
    let found = what_is_wrong_with_the_menu(&the_harvest().menu);

    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn test_the_menu_reading_sees_quick_steps_left_off_action() {
    // The companion: Action as it stood before this plan.
    let mut planted = the_harvest().menu.clone();
    planted.action_items.retain(|label| label != "&Quick Steps");
    planted.quick_steps_help = String::new();
    planted.quick_steps_items = None;

    assert_eq!(what_is_wrong_with_the_menu(&planted).len(), 3);
}

const THE_WINDOW: &str = "src/presentation/wx_app.rs";

fn the_shipping_window() -> String {
    let whole = std::fs::read_to_string(THE_WINDOW)
        .unwrap_or_else(|why| panic!("{THE_WINDOW}: {why}"))
        .replace("\r\n", "\n");
    what_ships(&whole)
}

/// One arm of the command dispatch, from its guard to the next arm's.
fn arm_of<'a>(source: &'a str, guard: &str) -> Option<&'a str> {
    let start = source.find(guard)?;
    let rest = &source[start + guard.len()..];
    let end = rest.find("_ if id == ").unwrap_or(rest.len());
    Some(&rest[..end])
}

const THE_ARM: &str = "_ if id == ID_QUICK_STEP_MGR =>";

/// What is wrong with what Manage Quick Steps runs.
fn what_is_wrong_with_the_arm(source: &str) -> Vec<String> {
    let Some(arm) = arm_of(source, THE_ARM) else {
        return vec!["nothing answers Manage Quick Steps".to_string()];
    };
    let mut wrong = Vec::new();
    if !arm.contains("managers::manage_quick_steps(") {
        wrong.push("Manage Quick Steps never opens the manager".to_string());
    }
    if !arm.contains("read_the_tree_back(") {
        wrong.push("the tree is not read back after the manager closes".to_string());
    }
    wrong
}

#[test]
fn test_manage_quick_steps_opens_the_manager_and_reads_the_tree_back() {
    let found = what_is_wrong_with_the_arm(&the_shipping_window());

    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn test_the_arm_reading_sees_an_arm_that_reaches_nothing() {
    let source = the_shipping_window();
    let arm = arm_of(&source, THE_ARM).unwrap_or_default();
    let planted = match arm.is_empty() {
        true => source.clone(),
        false => source.replacen(
            arm,
            &arm.replacen("managers::manage_quick_steps(", "nothing_at_all(", 1),
            1,
        ),
    };

    assert_eq!(
        what_is_wrong_with_the_arm(&planted),
        ["Manage Quick Steps never opens the manager"]
    );
}

#[test]
fn test_a_step_the_editor_can_keep_is_kept_under_its_tidied_name() {
    let does = marked_flagged_labelled_and_moved();

    assert_eq!(
        what_the_editor_keeps("  Sort it  ", does.clone(), &["Receipts".to_string()]),
        Ok(("Sort it".to_string(), does))
    );
}

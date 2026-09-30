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
//! **Companions.** Each check is handed a wrong state, a Key column that
//! gives a fourth step a key and a store whose order was never written, and
//! refuses it, so a check that passes is one that could have failed.
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
use wixen_mail::application::quick_steps::{QuickStep, StoredStep, key_for};
use wixen_mail::application::reordering::Move;
use wixen_mail::data::message_cache::MessageCache;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::managers::save_what_the_quick_step_manager_returned;
use wixen_mail::presentation::wx_managers::{
    ManagerState, QuickStepEntry, build_quick_step_manager, move_the_chosen_row,
    populate_quick_steps,
};
use wxdragon::prelude::*;

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const VT_I4: u16 = 3;
const CHILDID_SELF: i64 = 0;
const WINSTA_ALL_ACCESS: u32 = 0x037F;
const GENERIC_ALL: u32 = 0x1000_0000;

/// commctrl.h: `LVM_FIRST + 115` and `LVM_FIRST + 95`.
const LVM_GETITEMTEXTW: u32 = 0x1000 + 115;
const LVM_GETCOLUMNW: u32 = 0x1000 + 95;
const LVCF_TEXT: u32 = 0x4;

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
}

type Hresult = i32;
type ReleaseFn = unsafe extern "system" fn(*mut c_void) -> u32;
type GetBstrFn = unsafe extern "system" fn(*mut c_void, Variant, *mut *mut u16) -> Hresult;

// IAccessible's vtable: IUnknown (3), IDispatch (4), then get_accParent (7),
// get_accChildCount (8), get_accChild (9), get_accName (10).
const VTBL_RELEASE: usize = 2;
const VTBL_GET_ACC_NAME: usize = 10;

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

/// A window's own name over MSAA, which is what NVDA reads for a native
/// control.
fn msaa_name_of(hwnd: isize) -> Result<String, String> {
    let mut object: *mut c_void = std::ptr::null_mut();
    // SAFETY: a live window handle; the object is released before returning.
    let hr =
        unsafe { AccessibleObjectFromWindow(hwnd, OBJID_CLIENT, &IID_IACCESSIBLE, &mut object) };
    if hr < 0 || object.is_null() {
        return Err(format!("AccessibleObjectFromWindow failed 0x{hr:x}"));
    }
    // SAFETY: `object` is a live IAccessible; the slots are IAccessible's.
    unsafe {
        let get: GetBstrFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_NAME));
        let mut said: *mut u16 = std::ptr::null_mut();
        let name = match get(object, Variant::child(CHILDID_SELF), &mut said) >= 0 {
            true => take_bstr(said),
            false => String::new(),
        };
        let release: ReleaseFn = std::mem::transmute(vtable_entry(object, VTBL_RELEASE));
        release(object);
        Ok(name)
    }
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
    let list_name = msaa_name_of(widgets.list.get_handle() as isize)?;
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
                let harvest = read_the_manager(&frame, &a11y, &cache)?;
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

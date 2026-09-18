//! Mark as Read says which way it will go, and M reaches it on a real list.
//!
//! The tester on 2026-09-15 (#27): "The mark read command in the action menu
//! and the corresponding context menu should reflect the current status of
//! the message ... Use 'm' bound to message lists to toggle the state and
//! announcement, 'read'/'unread'." Two of the three things this needs cannot
//! be read from the source, and this file reads them from built controls.
//!
//! **Reading A, the letter on a real list.** A `SysListView32` in report view
//! takes a typed letter as the start of a search and jumps to the next row
//! beginning with it, on the `WM_CHAR` that follows the key-down. A handler
//! that runs but leaves the event skipped is one the search still gets, and
//! under wxdragon a closure that says nothing is exactly that
//! (`wxdragon-sys-0.9.17/cpp/src/event.cpp:379` resets `Skip(true)` before
//! every closure). So the reading builds a list with rows "Alpha", "Mango"
//! and "Zulu", selects the first, wires M through
//! `presentation::list_keys::wire_letter`, sends `WM_KEYDOWN` for M and the
//! `WM_CHAR` for m to the list's own window the way the message loop would,
//! and reads where the selection is: still on "Alpha" when the letter was
//! consumed, on "Mango" when the search got it. The companion is a second
//! list wired with another letter, where M has to reach the search and move
//! the cursor, which is what shows the search is what the letter would
//! otherwise reach.
//!
//! **Reading B, the toolbar's label where a screen reader reads it.** The
//! main toolbar is `Flat | Text`, and wxdragon 0.9.17 can set a tool's short
//! help and not its label, so `presentation::toolbar_text::relabel` goes
//! through `TB_SETBUTTONINFOW` on the toolbar's own window. The reading
//! builds a toolbar with one tool "Mark Read", relabels it "Mark as Unread",
//! and reads the text back two ways: through `TB_GETBUTTONTEXTW`, the
//! control's own answer, and through `AccessibleObjectFromWindow(hwnd,
//! OBJID_CLIENT)` and `get_accName` on child 1, which is the object NVDA
//! reads a toolbar button's name from. The companion reads both before the
//! relabel and requires the old label, so a reader that answered the new
//! label whatever was set would be seen.
//!
//! One window session for the whole file, on the shape
//! `tests/a_kept_folder_reads_as_a_checked_check_box.rs` set and for the
//! reason `tests/theme_reach.rs` gives, every reading sharing it through a
//! `OnceLock`. The readings over the main window's source, which
//! hold the three surfaces to the one rule and the key to the toggle, are
//! functions over `what_ships` with companions, and need no window.
//!
//! The Windows calls are declared by hand from the headers, as every reading
//! in `tests/` does, so the `windows` crate is not compiled in for a test.

#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::presentation::list_keys::wire_letter;
use wixen_mail::presentation::toolbar_text::{label_of, relabel};
use wxdragon::prelude::*;

const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const WM_CHAR: u32 = 0x0102;
/// winuser.h: the virtual key for the letter M.
const VK_M: usize = 0x4D;
const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const VT_I4: u16 = 3;

/// The rows the lists hold, in order. M has to find "Mango" and no earlier
/// row, so the search's answer and the wired handler's are different rows.
const ROWS: [&str; 3] = ["Alpha", "Mango", "Zulu"];
const THE_TOOL: Id = 4_027;
const THE_OLD_LABEL: &str = "Mark Read";
const THE_NEW_LABEL: &str = "Mark as Unread";

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

#[link(name = "user32")]
unsafe extern "system" {
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
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

unsafe fn vtable_entry(object: *mut c_void, index: usize) -> *const c_void {
    // SAFETY: a COM object is a pointer to its vtable.
    unsafe {
        let vtable = *(object as *const *const *const c_void);
        *vtable.add(index)
    }
}

unsafe fn release(object: *mut c_void) {
    // SAFETY: `object` is a live COM object; Release is vtable slot 2.
    unsafe {
        let f: ReleaseFn = std::mem::transmute(vtable_entry(object, VTBL_RELEASE));
        f(object);
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

/// The MSAA name of one child of a window's client object: what NVDA says
/// for a toolbar button.
fn msaa_name_of(hwnd: isize, child_id: i64) -> Result<String, String> {
    let mut object: *mut c_void = std::ptr::null_mut();
    // SAFETY: a live window handle; the object is released before returning.
    let hr =
        unsafe { AccessibleObjectFromWindow(hwnd, OBJID_CLIENT, &IID_IACCESSIBLE, &mut object) };
    if hr < 0 || object.is_null() {
        return Err(format!("AccessibleObjectFromWindow failed 0x{hr:x}"));
    }
    // SAFETY: `object` is a live IAccessible; the slot is get_accName's.
    unsafe {
        let get_name: GetBstrFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_NAME));
        let mut name: *mut u16 = std::ptr::null_mut();
        let hr_name = get_name(object, Variant::child(child_id), &mut name);
        release(object);
        if hr_name < 0 {
            return Err(format!("get_accName hr=0x{hr_name:x}"));
        }
        Ok(take_bstr(name))
    }
}

/// M pressed and released on the window the way the message loop hands a
/// real key to it: the key-down with a repeat count of one, the char the
/// loop's `TranslateMessage` would make of it, and the key-up with the
/// transition bit. The char is what the list's search reads, and it is sent
/// whether or not the key-down was consumed, because that is the order the
/// loop sends them in; a consumed key-down is what makes the window eat it.
fn press_m(hwnd: isize) {
    // SAFETY: `hwnd` is a live window on this thread, built by the caller.
    unsafe {
        SendMessageW(hwnd, WM_KEYDOWN, VK_M, 1);
        SendMessageW(hwnd, WM_CHAR, usize::from(b'm'), 1);
        SendMessageW(hwnd, WM_KEYUP, VK_M, 0xC000_0001_u32 as i32 as isize);
    }
}

/// One list of the three rows, the first selected and focused, with `letter`
/// wired on it counting the rows the handler was called with.
fn a_list_wired_with(frame: &Frame, letter: char) -> (ListCtrl, Rc<RefCell<Vec<i64>>>) {
    let list = ListCtrl::builder(frame)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel)
        .build();
    list.insert_column(0, "Subject", ListColumnFormat::Left, 160);
    for (at, row) in ROWS.iter().enumerate() {
        list.insert_item(at as i64, row, None);
    }
    let called = Rc::new(RefCell::new(Vec::new()));
    wire_letter(&list, letter, {
        let called = called.clone();
        move |row| called.borrow_mut().push(row)
    });
    list.set_item_state(
        0,
        ListItemState::Selected | ListItemState::Focused,
        ListItemState::Selected | ListItemState::Focused,
    );
    (list, called)
}

/// What the readings found in the one window session.
#[derive(Debug, Clone)]
struct Harvest {
    /// Reading A: the rows the wired handler was called with, and the
    /// selected row afterwards, on the list wired with M.
    wired_calls: Vec<i64>,
    wired_selection_after: i32,
    /// Its companion: the same on a list wired with another letter.
    other_calls: Vec<i64>,
    other_selection_after: i32,
    /// Reading B: the tool's text before and after the relabel, through the
    /// control's own message and through the accessible object.
    text_before: Option<String>,
    name_before: String,
    text_after: Option<String>,
    name_after: String,
}

fn read_the_letter_on_two_lists(frame: &Frame) -> Result<[(Vec<i64>, i32); 2], String> {
    let (wired, wired_calls) = a_list_wired_with(frame, 'M');
    let (other, other_calls) = a_list_wired_with(frame, 'Q');
    let mut readings = Vec::new();
    for (list, calls) in [(wired, wired_calls), (other, other_calls)] {
        if list.get_first_selected_item() != 0 {
            return Err(format!(
                "the first row was not selected before the key: {}",
                list.get_first_selected_item()
            ));
        }
        let hwnd = list.get_handle() as isize;
        if hwnd == 0 {
            return Err("the list has no window handle".to_string());
        }
        list.set_focus();
        press_m(hwnd);
        readings.push((calls.borrow().clone(), list.get_first_selected_item()));
    }
    readings
        .try_into()
        .map_err(|_| "two lists were read and two readings were not taken".to_string())
}

fn read_the_relabelled_tool(
    frame: &Frame,
) -> Result<(Option<String>, String, Option<String>, String), String> {
    let toolbar = frame
        .create_tool_bar(Some(ToolBarStyle::Flat | ToolBarStyle::Text), ID_ANY as Id)
        .ok_or_else(|| "the frame made no toolbar".to_string())?;
    let bitmap = Bitmap::new(16, 16).unwrap_or_else(Bitmap::null_bitmap);
    toolbar.add_tool(THE_TOOL, THE_OLD_LABEL, &bitmap, "Mark as read");
    toolbar.realize();
    let hwnd = toolbar.get_handle() as isize;
    if hwnd == 0 {
        return Err("the toolbar has no window handle".to_string());
    }
    let text_before = label_of(&toolbar, THE_TOOL);
    let name_before = msaa_name_of(hwnd, 1)?;
    relabel(&toolbar, THE_TOOL, THE_NEW_LABEL);
    let text_after = label_of(&toolbar, THE_TOOL);
    let name_after = msaa_name_of(hwnd, 1)?;
    Ok((text_before, name_before, text_after, name_after))
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
                let frame = Frame::builder()
                    .with_title("Mark as Read, the reading")
                    .with_size(Size::new(320, 240))
                    .build();
                // Shown, because the list's search and the accessible object
                // are read from a control somebody could see; the frame is
                // destroyed before the session ends.
                frame.show(true);
                let [
                    (wired_calls, wired_selection_after),
                    (other_calls, other_selection_after),
                ] = read_the_letter_on_two_lists(&frame)?;
                let (text_before, name_before, text_after, name_after) =
                    read_the_relabelled_tool(&frame)?;
                frame.destroy();
                Ok(Harvest {
                    wired_calls,
                    wired_selection_after,
                    other_calls,
                    other_selection_after,
                    text_before,
                    name_before,
                    text_after,
                    name_after,
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

// ── Reading A ──────────────────────────────────────────────────────────────

#[test]
fn test_reading_a_the_letter_reaches_its_handler_on_a_real_list_and_the_search_does_not_get_it() {
    let harvest = the_harvest();
    assert_eq!(
        harvest.wired_calls,
        vec![0],
        "the handler wired on M was called with these rows; wanted the selected row, once"
    );
    assert_eq!(
        harvest.wired_selection_after,
        0,
        "after M on the list wired with M the selection is on {:?}; the search got the letter",
        ROWS.get(harvest.wired_selection_after.max(0) as usize)
    );
}

#[test]
fn test_reading_a_companion_a_letter_nobody_wired_reaches_the_search_and_moves_the_cursor() {
    // What the letter would otherwise reach, shown: on a list wired with
    // another letter, M is the control's, and the control jumps to the row
    // that begins with it. A reading that could not see the search move the
    // cursor could not tell a consumed letter from one nobody typed.
    let harvest = the_harvest();
    assert!(
        harvest.other_calls.is_empty(),
        "the handler wired on Q was called with {:?} for M",
        harvest.other_calls
    );
    assert_eq!(
        harvest.other_selection_after,
        1,
        "after M on the list wired with Q the selection is on {:?}; wanted Mango",
        ROWS.get(harvest.other_selection_after.max(0) as usize)
    );
}

// ── Reading B ──────────────────────────────────────────────────────────────

#[test]
fn test_reading_b_a_relabelled_tool_says_the_new_label_where_the_control_is_asked() {
    let harvest = the_harvest();
    assert_eq!(
        harvest.text_after.as_deref(),
        Some(THE_NEW_LABEL),
        "TB_GETBUTTONTEXTW after the relabel"
    );
}

#[test]
fn test_reading_b_a_relabelled_tool_says_the_new_label_where_a_screen_reader_reads_it() {
    let harvest = the_harvest();
    assert_eq!(
        harvest.name_after, THE_NEW_LABEL,
        "get_accName on the button after the relabel"
    );
}

#[test]
fn test_reading_b_companion_both_readers_saw_the_old_label_before_the_relabel() {
    // Both readers answered the label the toolkit gave the button, so an
    // answer of the new label afterwards is the relabel and not a reader that
    // answers whatever it is asked for.
    let harvest = the_harvest();
    assert_eq!(harvest.text_before.as_deref(), Some(THE_OLD_LABEL));
    assert_eq!(harvest.name_before, THE_OLD_LABEL);
}

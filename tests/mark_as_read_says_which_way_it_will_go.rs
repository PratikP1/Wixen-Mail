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

// ── The main window's source: the three surfaces, the key, the toggle ──────
//
// Read from the source rather than run, because the sites are inside the
// main window's builder and its update handler, which need a live window, a
// running event loop and a mail store to reach. Each reading is a function
// over the shipped text returning a complaint, and each has a companion that
// hands it a snippet shaped like the site with the fault planted.

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

fn the_main_window() -> String {
    let whole = std::fs::read_to_string(THE_MAIN_WINDOW)
        .unwrap_or_else(|why| panic!("{THE_MAIN_WINDOW}: {why}"))
        .replace("\r\n", "\n");
    wixen_mail::common::what_ships::what_ships(&whole)
}

/// One function's text, from its signature to the closing brace at column
/// nought, or a complaint when the signature is gone.
fn body_of(source: &str, signature: &str) -> Result<String, String> {
    let at = source.find(signature).ok_or(format!(
        "{signature} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let ends = rest.find("\n}\n").map_or(rest.len(), |end| end + 2);
    Ok(rest[..ends].to_string())
}

/// The text after the first `from` up to the next `to`, or a complaint
/// naming which anchor is gone.
fn between<'a>(text: &'a str, from: &str, to: &str) -> Result<&'a str, String> {
    let start = text
        .find(from)
        .ok_or(format!("{from:?} is no longer here, so this reads nothing"))?
        + from.len();
    let rest = &text[start..];
    let end = rest
        .find(to)
        .ok_or(format!("{to:?} is no longer here, so this reads nothing"))?;
    Ok(&rest[..end])
}

/// The body of one `_ if id == ...` arm of the command dispatch, up to the
/// next arm of the same shape.
fn the_id_arm<'a>(source: &'a str, heading: &str) -> Result<&'a str, String> {
    let start = source.find(heading).ok_or(format!(
        "{heading:?} is no longer here, so this reads nothing"
    ))? + heading.len();
    let rest = &source[start..];
    let end = rest.find("_ if id ==").unwrap_or(rest.len());
    Ok(&rest[..end])
}

const THE_ARM: &str = "_ if id == ID_MARK_READ =>";
const THE_TOGGLE: &str = "fn toggle_read_state(";
const THE_REFRESH: &str = "fn refresh_mark_read_wording(";
/// Where the key is wired on the message list, and the first line after the
/// read-aloud and key wirings.
const THE_KEY_WIRING: (&str, &str) = ("wire_letter(&msg_list, 'M'", "let preview_visible");
/// Where the selection handler starts and where the next handler on the
/// list starts, which is where it ends.
const THE_SELECTION_HANDLER: (&str, &str) =
    ("msg_list.on_item_selected({", "msg_list.on_column_click({");
/// The update arm that lands a read flag on the row, and the next arm.
const THE_TOGGLED_ARM: (&str, &str) = (
    "UIUpdate::MessageReadToggled(cache_id, new_read) => {",
    "\n        UIUpdate::",
);
/// Where the message list's context menu is wired, and the next control's.
const THE_CONTEXT_WIRING: (&str, &str) = (
    "wire_context_menu(&msg_list,",
    "wire_context_menu(&folder_tree",
);
/// Where the main toolbar's tool is added, and the end of that call.
const THE_TOOL_ADDED: (&str, &str) = ("toolbar.add_tool(\n                    ID_MARK_READ,", ");");
const THE_ITEM: &str = ".append_item(ID_MARK_READ, ";

/// The Action menu's item and the key run one toggle, and the key says one
/// word: the arm calls it, the key wiring calls it under the key's name, and
/// the toggle asks the rule for the word, signals Confirmed and sends the
/// flag to the server as the arm always did.
fn the_arm_and_the_key_share_one_toggle(app: &str) -> Result<(), String> {
    let arm = the_id_arm(app, THE_ARM)?;
    if !arm.contains("toggle_read_state(") {
        return Err(
            "the ID_MARK_READ arm does not call toggle_read_state, so the menu item and the key \
             run two copies of the toggle"
                .to_string(),
        );
    }
    let wiring = between(app, THE_KEY_WIRING.0, THE_KEY_WIRING.1)?;
    if !wiring.contains("toggle_read_state(") || !wiring.contains("How::TheKey") {
        return Err(
            "the key wired on the message list does not run toggle_read_state as the key, so M \
             does something other than the command"
                .to_string(),
        );
    }
    let toggle = body_of(app, THE_TOGGLE)?;
    for needed in [
        "what_the_key_says(",
        "a11y.signal(FeedbackEvent::Confirmed",
        "FlagChange::Read(",
    ] {
        if !toggle.contains(needed) {
            return Err(format!(
                "toggle_read_state does not reach {needed}, so the toggle under the key is not \
                 the toggle the command was"
            ));
        }
    }
    Ok(())
}

/// The words are refreshed wherever the state can change under them: on
/// selection, after the toggle, and when a read flag lands on the row from
/// the server or is put back by a refusal. The refresh sets all three
/// surfaces from the one rule.
fn the_wording_follows_the_state_everywhere_it_can_change(app: &str) -> Result<(), String> {
    let handler = between(app, THE_SELECTION_HANDLER.0, THE_SELECTION_HANDLER.1)?;
    if !handler.contains("refresh_mark_read_wording(") {
        return Err(
            "the selection handler never refreshes the wording, so arrowing onto a read message \
             leaves the command saying Mark as Read"
                .to_string(),
        );
    }
    let toggle = body_of(app, THE_TOGGLE)?;
    if !toggle.contains("refresh_mark_read_wording(") {
        return Err(
            "toggle_read_state never refreshes the wording, so the item says the old way after \
             the key or the command"
                .to_string(),
        );
    }
    let toggled = between(app, THE_TOGGLED_ARM.0, THE_TOGGLED_ARM.1)?;
    if !toggled.contains("refresh_mark_read_wording(") {
        return Err(
            "the MessageReadToggled arm never refreshes the wording, so a flag that lands from \
             the server or is put back by a refusal leaves the command saying the wrong way"
                .to_string(),
        );
    }
    let refresh = body_of(app, THE_REFRESH)?;
    for needed in [
        "what_the_command_says(",
        "find_item(ID_MARK_READ)",
        ".set_label(",
        "toolbar_text::relabel(",
        "set_tool_short_help(",
    ] {
        if !refresh.contains(needed) {
            return Err(format!(
                "refresh_mark_read_wording does not reach {needed}, so one of the three surfaces \
                 is not set from the rule"
            ));
        }
    }
    Ok(())
}

/// The message list's context menu is answered from the state at the moment
/// the key is pressed, and no longer from the focus alone.
fn the_context_menu_asks_the_state(app: &str) -> Result<(), String> {
    let wiring = between(app, THE_CONTEXT_WIRING.0, THE_CONTEXT_WIRING.1)?;
    if !wiring.contains("entries_for_messages(") {
        return Err(
            "the message list's context menu is not built from entries_for_messages, so it says \
             Mark as read whatever the message's state"
                .to_string(),
        );
    }
    if wiring.contains("entries_for(Focus::Messages)") {
        return Err(
            "the message list's context menu still answers the focus form beside the state's"
                .to_string(),
        );
    }
    Ok(())
}

/// The tool and the item are built with the unread wording's words, which
/// is what they say until the first selection refreshes them.
fn the_tool_and_the_item_start_with_the_rules_words(app: &str) -> Result<(), String> {
    let tool = between(app, THE_TOOL_ADDED.0, THE_TOOL_ADDED.1)?;
    if !tool.contains("what_the_command_says(true)") {
        return Err(
            "the toolbar's tool is added with its own words rather than the rule's".to_string(),
        );
    }
    let item = between(app, THE_ITEM, ")")?;
    let literal = item
        .split('"')
        .nth(1)
        .ok_or("the Action menu's item carries no quoted label".to_string())?;
    let rule = wixen_mail::application::marking_read::what_the_command_says(true).menu;
    if literal != rule {
        return Err(format!(
            "the Action menu's item is built with {literal:?} and the rule says {rule:?}; the \
             literal stays so the menu-letter guard in tests/wired.rs can read it, and it has \
             to be the rule's word"
        ));
    }
    Ok(())
}

#[test]
fn test_the_menu_item_and_the_key_run_one_toggle_and_the_key_says_one_word() {
    the_arm_and_the_key_share_one_toggle(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_wording_is_refreshed_on_selection_after_the_toggle_and_when_a_flag_lands() {
    the_wording_follows_the_state_everywhere_it_can_change(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_context_menu_on_the_message_list_asks_the_state() {
    the_context_menu_asks_the_state(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_tool_and_the_item_are_built_with_the_rules_words() {
    the_tool_and_the_item_start_with_the_rules_words(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

// ── The companions ─────────────────────────────────────────────────────────
//
// Each hands its reading a snippet shaped like the sites, with the fault
// planted, rather than editing the real file's text: the readings' anchors
// exist only once the sites are built.

/// The sites as they should be, in a snippet.
fn a_window_as_it_should_be() -> String {
    format!(
        "{}\n    refresh_mark_read_wording(&frame, toolbar_handle, &state);\n{}\n\
         {} Some(entries_for_messages(any_unread)){}\n\
         {}\n    what_the_command_says(true).spoken,\n{}\n\
         {}\"Mark as R&ead\", \"Mark the selected message as read\")\n\
         {}\n    toggle_read_state(app, &a11y, How::TheKey, &frame, toolbar_handle);\n{}\n\
         {} {{\n    toggle_read_state(app, &a11y, How::TheCommand, &frame, toolbar_handle);\n\
         }}\n                        _ if id == ID_SEARCH => {{\n\
         {}\n    refresh_mark_read_wording(frame, toolbar, state);{}\n\
         {}) {{\n    let word = what_the_key_says(new_read);\n    a11y.signal(FeedbackEvent::Confirmed, word);\n\
         ServerChange::Flag(FlagChange::Read(new_read));\n    refresh_mark_read_wording(frame, toolbar, state);\n}}\n\
         {}) {{\n    let wording = what_the_command_says(any_unread);\n\
         bar.find_item(ID_MARK_READ).set_label(wording.menu);\n\
         toolbar_text::relabel(&toolbar, ID_MARK_READ, wording.spoken);\n\
         toolbar.set_tool_short_help(ID_MARK_READ, wording.help);\n}}\n",
        THE_SELECTION_HANDLER.0,
        THE_SELECTION_HANDLER.1,
        THE_CONTEXT_WIRING.0,
        THE_CONTEXT_WIRING.1,
        THE_TOOL_ADDED.0,
        THE_TOOL_ADDED.1,
        THE_ITEM,
        THE_KEY_WIRING.0,
        THE_KEY_WIRING.1,
        THE_ARM,
        THE_TOGGLED_ARM.0,
        THE_TOGGLED_ARM.1,
        THE_TOGGLE,
        THE_REFRESH,
    )
}

#[test]
fn test_the_readings_pass_a_window_shaped_as_it_should_be() {
    let app = a_window_as_it_should_be();
    the_arm_and_the_key_share_one_toggle(&app).unwrap_or_else(|why| panic!("{why}"));
    the_wording_follows_the_state_everywhere_it_can_change(&app)
        .unwrap_or_else(|why| panic!("{why}"));
    the_context_menu_asks_the_state(&app).unwrap_or_else(|why| panic!("{why}"));
    the_tool_and_the_item_start_with_the_rules_words(&app).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_reading_complains_when_the_key_runs_its_own_toggle() {
    let app = a_window_as_it_should_be().replacen(
        "toggle_read_state(app, &a11y, How::TheKey, &frame, toolbar_handle);",
        "lock_state(&state).messages[row].read ^= true;",
        1,
    );
    let why = the_arm_and_the_key_share_one_toggle(&app)
        .expect_err("a key running its own toggle was passed over");
    assert!(
        why.contains("does not run toggle_read_state as the key"),
        "{why}"
    );
}

#[test]
fn test_the_reading_complains_when_a_surface_is_left_behind() {
    let app = a_window_as_it_should_be().replacen(
        "    refresh_mark_read_wording(&frame, toolbar_handle, &state);\n",
        "",
        1,
    );
    let why = the_wording_follows_the_state_everywhere_it_can_change(&app)
        .expect_err("a selection handler refreshing nothing was passed over");
    assert!(why.contains("selection handler never refreshes"), "{why}");

    let app = a_window_as_it_should_be().replacen(
        "toolbar_text::relabel(&toolbar, ID_MARK_READ, wording.spoken);\n",
        "",
        1,
    );
    let why = the_wording_follows_the_state_everywhere_it_can_change(&app)
        .expect_err("a refresh leaving the toolbar behind was passed over");
    assert!(why.contains("toolbar_text::relabel("), "{why}");
}

#[test]
fn test_the_reading_complains_when_the_context_menu_answers_the_focus_alone() {
    let app = a_window_as_it_should_be().replacen(
        "Some(entries_for_messages(any_unread))",
        "Some(entries_for(Focus::Messages))",
        1,
    );
    let why = the_context_menu_asks_the_state(&app)
        .expect_err("a context menu answering the focus alone was passed over");
    assert!(why.contains("not built from entries_for_messages"), "{why}");
}

#[test]
fn test_the_reading_complains_when_the_item_and_the_rule_differ() {
    let app = a_window_as_it_should_be().replacen("\"Mark as R&ead\"", "\"&Mark Read\"", 1);
    let why = the_tool_and_the_item_start_with_the_rules_words(&app)
        .expect_err("an item built with its own words was passed over");
    assert!(why.contains("the rule says"), "{why}");
}

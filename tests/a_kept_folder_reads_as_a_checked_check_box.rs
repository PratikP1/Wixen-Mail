//! A kept folder in Folders to Keep Up to Date reads as checked on the channel
//! a screen reader reads, and the dialog is a tree nested the way the folder
//! tree is.
//!
//! The tester's words on 2026-09-17 under NVDA against `1.0.0-alpha.1` (#70):
//! the list is flat where the folder tree is nested, a kept folder is heard as
//! "check box, read-only, not checked", and the title is an identifier. The
//! module's own header said "Still to be confirmed with a screen reader"; that
//! confirmation happened, and it failed.
//!
//! Three readings, taken in one window session and asserted over separately.
//!
//! **Reading A** is the dialog as it was built until 2026-09-18: a
//! `CheckListBox` whose rows answered through an accessible object this
//! program wrote (`names::CheckedRows`). It is read over
//! `AccessibleObjectFromWindow(hwnd, OBJID_CLIENT)`, which is what NVDA asks a
//! `ListBox`-class window for, and each row's role, state and name are kept.
//! The test holds the object to what the code claimed, and whether it is green
//! or red is the finding: green means the object was right and NVDA read
//! something else for the row; red means the snapshot was wrong.
//!
//! **Reading B** is a scratch `SysTreeView32` with `TVS_CHECKBOXES` added
//! through `SetWindowLongPtrW` before any item, one item ticked through
//! `TVM_SETITEMW`, and the state read back the way NVDA reads it: NVDA's
//! `sysTreeView32.py` (read 2026-09-18) sends `TVM_GETITEMSTATE` with
//! `TVIS_STATEIMAGEMASK` and says checked on state image 2, not checked on 1,
//! half on 3; the level comes from `accValue`. No object this program writes is
//! in that path, which is the point of asking. Then Space is sent to the tree
//! and the state read again, so whether the control toggles on its own is
//! measured and not assumed.
//!
//! **Reading C** is the dialog as it is now, built over rows that carry their
//! parents: the control's class, every item in the order the tree walks them,
//! each item's parent, its state image over `TVM_GETITEMSTATE`, its MSAA state,
//! whether it is expanded, and where the cursor is. A pure predicate holds that
//! harvest to what the rows asked for, and two companions hand the predicate a
//! wrong state and a wrong parent so it is known to complain.
//!
//! The reader is the same sixty lines of user32 and oleacc declarations that
//! `tests/every_settings_checkbox_reads_as_a_checkbox_after_its_page_is_built.rs`
//! uses, for the same reason: a feature on the `windows` crate is compiled into
//! the shipping binary. One `wxdragon::main` per process serves every test here
//! through a `OnceLock` holding a `Result`, so the budget is spent once and the
//! initialiser panics on nothing. Every window read here was built here, and
//! nothing calls `show_modal`.

#![cfg(windows)]

use std::ffi::c_void;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::presentation::theme;
use wixen_mail::presentation::wx_folder_choice::{FolderRow, build_folder_choice_dialog};
use wxdragon::prelude::*;

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const GWL_STYLE: i32 = -16;
const VT_I4: u16 = 3;

/// MSAA roles and the state bits this file reads (oleacc.h).
const ROLE_SYSTEM_CHECKBUTTON: i64 = 0x2c;
const STATE_SYSTEM_CHECKED: i64 = 0x10;
const STATE_SYSTEM_READONLY: i64 = 0x40;

/// The tree view's style bit, messages, flags and masks (commctrl.h).
/// `TV_FIRST` is 0x1100 and each message is an offset from it.
const TVS_CHECKBOXES: isize = 0x0100;
const TVM_GETNEXTITEM: u32 = 0x1100 + 10;
const TVM_SELECTITEM: u32 = 0x1100 + 11;
const TVM_GETITEMSTATE: u32 = 0x1100 + 39;
const TVM_MAPHTREEITEMTOACCID: u32 = 0x1100 + 43;
const TVM_GETITEMW: u32 = 0x1100 + 62;
const TVM_SETITEMW: u32 = 0x1100 + 63;
const TVGN_ROOT: usize = 0;
const TVGN_NEXT: usize = 1;
const TVGN_PARENT: usize = 3;
const TVGN_CHILD: usize = 4;
const TVGN_CARET: usize = 9;
const TVIF_TEXT: u32 = 0x0001;
const TVIF_STATE: u32 = 0x0008;
const TVIF_HANDLE: u32 = 0x0010;
const TVIS_EXPANDED: u32 = 0x0020;
const TVIS_STATEIMAGEMASK: u32 = 0xF000;

/// The two messages a key press is, and the key (winuser.h).
const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const VK_SPACE: usize = 0x20;

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

/// `TVITEMW` as commctrl.h lays it out on 64-bit Windows: 56 bytes, the
/// handle at offset 8 and `lParam` at 48. The size is checked before any
/// message carries one, because a wrong layout here is undefined behaviour in
/// the control and not an error anybody is told about.
#[repr(C)]
struct TreeItemW {
    mask: u32,
    h_item: isize,
    state: u32,
    state_mask: u32,
    psz_text: *mut u16,
    cch_text_max: i32,
    i_image: i32,
    i_selected_image: i32,
    c_children: i32,
    l_param: isize,
}

type Hresult = i32;
type ReleaseFn = unsafe extern "system" fn(*mut c_void) -> u32;
type GetVariantFn = unsafe extern "system" fn(*mut c_void, Variant, *mut Variant) -> Hresult;
type GetBstrFn = unsafe extern "system" fn(*mut c_void, Variant, *mut *mut u16) -> Hresult;

// IAccessible's vtable: IUnknown (3), IDispatch (4), then get_accParent (7),
// get_accChildCount (8), get_accChild (9), get_accName (10), get_accValue
// (11), get_accDescription (12), get_accRole (13), get_accState (14).
const VTBL_RELEASE: usize = 2;
const VTBL_GET_ACC_NAME: usize = 10;
const VTBL_GET_ACC_VALUE: usize = 11;
const VTBL_GET_ACC_ROLE: usize = 13;
const VTBL_GET_ACC_STATE: usize = 14;

#[link(name = "user32")]
unsafe extern "system" {
    fn GetClassNameW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
    fn GetWindowLongPtrW(hwnd: isize, index: i32) -> isize;
    fn SetWindowLongPtrW(hwnd: isize, index: i32, value: isize) -> isize;
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
    fn GetFocus() -> isize;
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

fn class_name(hwnd: isize) -> String {
    let mut buffer = [0u16; 256];
    // SAFETY: the buffer is as long as the count says.
    let len = unsafe { GetClassNameW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

fn send(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize {
    // SAFETY: every handle sent to here belongs to a window this file built
    // and still holds.
    unsafe { SendMessageW(hwnd, message, wparam, lparam) }
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

/// One MSAA reading of a child of a window's client object: role, state, name
/// and value, or why they could not be read.
#[derive(Debug, Clone)]
struct Msaa {
    role: i64,
    state: i64,
    name: String,
    value: String,
}

fn msaa_of(hwnd: isize, child_id: i64) -> Result<Msaa, String> {
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
        let get_value: GetBstrFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_VALUE));
        let get_role: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_ROLE));
        let get_state: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_STATE));
        let mut role = Variant::empty();
        let hr_role = get_role(object, Variant::child(child_id), &mut role);
        let mut state = Variant::empty();
        let hr_state = get_state(object, Variant::child(child_id), &mut state);
        let mut name: *mut u16 = std::ptr::null_mut();
        let hr_name = get_name(object, Variant::child(child_id), &mut name);
        let mut value: *mut u16 = std::ptr::null_mut();
        let hr_value = get_value(object, Variant::child(child_id), &mut value);
        release(object);
        if hr_role < 0 || role.vt != VT_I4 {
            return Err(format!("get_accRole hr=0x{hr_role:x} vt={}", role.vt));
        }
        if hr_state < 0 || state.vt != VT_I4 {
            return Err(format!("get_accState hr=0x{hr_state:x} vt={}", state.vt));
        }
        // A name or a value the object declines to give is an empty string
        // here and not an error: a tree item has a value (its level) and a
        // list row has none, and both are readings worth keeping.
        let name = if hr_name >= 0 {
            take_bstr(name)
        } else {
            String::new()
        };
        let value = if hr_value >= 0 {
            take_bstr(value)
        } else {
            String::new()
        };
        Ok(Msaa {
            role: role.val & 0xFFFF_FFFF,
            state: state.val & 0xFFFF_FFFF,
            name,
            value,
        })
    }
}

/// The MSAA child id of a tree item, which on 64-bit Windows is not the
/// handle: NVDA maps the other way with `TVM_MAPACCIDTOHTREEITEM`.
fn acc_id_of(tree: isize, item: isize) -> i64 {
    send(tree, TVM_MAPHTREEITEMTOACCID, item as usize, 0) as i64
}

/// The state image index the control holds for an item: 0 for none, 1 for
/// unchecked, 2 for checked, 3 for half. This is the message NVDA sends.
fn state_image_of(tree: isize, item: isize) -> u32 {
    let state = send(
        tree,
        TVM_GETITEMSTATE,
        item as usize,
        TVIS_STATEIMAGEMASK as isize,
    ) as u32;
    (state & TVIS_STATEIMAGEMASK) >> 12
}

fn is_expanded(tree: isize, item: isize) -> bool {
    send(
        tree,
        TVM_GETITEMSTATE,
        item as usize,
        TVIS_EXPANDED as isize,
    ) as u32
        & TVIS_EXPANDED
        != 0
}

/// Tick or untick one item through the control's own item state.
fn set_state_image(tree: isize, item: isize, index: u32) {
    let mut tvitem = TreeItemW {
        mask: TVIF_STATE | TVIF_HANDLE,
        h_item: item,
        state: index << 12,
        state_mask: TVIS_STATEIMAGEMASK,
        psz_text: std::ptr::null_mut(),
        cch_text_max: 0,
        i_image: 0,
        i_selected_image: 0,
        c_children: 0,
        l_param: 0,
    };
    send(
        tree,
        TVM_SETITEMW,
        0,
        &mut tvitem as *mut TreeItemW as isize,
    );
}

fn item_text(tree: isize, item: isize) -> String {
    let mut buffer = [0u16; 512];
    let mut tvitem = TreeItemW {
        mask: TVIF_TEXT | TVIF_HANDLE,
        h_item: item,
        state: 0,
        state_mask: 0,
        psz_text: buffer.as_mut_ptr(),
        cch_text_max: buffer.len() as i32,
        i_image: 0,
        i_selected_image: 0,
        c_children: 0,
        l_param: 0,
    };
    send(
        tree,
        TVM_GETITEMW,
        0,
        &mut tvitem as *mut TreeItemW as isize,
    );
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..len])
}

/// Every item in the tree, in the order the tree walks them: each item, then
/// its children, then its next sibling. With a hidden root, `TVGN_ROOT` is the
/// first top-level item.
fn items_in_walk_order(tree: isize) -> Vec<isize> {
    fn walk(tree: isize, item: isize, into: &mut Vec<isize>) {
        into.push(item);
        let mut child = send(tree, TVM_GETNEXTITEM, TVGN_CHILD, item);
        while child != 0 {
            walk(tree, child, into);
            child = send(tree, TVM_GETNEXTITEM, TVGN_NEXT, child);
        }
    }
    let mut items = Vec::new();
    let mut item = send(tree, TVM_GETNEXTITEM, TVGN_ROOT, 0);
    while item != 0 {
        walk(tree, item, &mut items);
        item = send(tree, TVM_GETNEXTITEM, TVGN_NEXT, item);
    }
    items
}

fn parent_text(tree: isize, item: isize) -> Option<String> {
    let parent = send(tree, TVM_GETNEXTITEM, TVGN_PARENT, item);
    (parent != 0).then(|| item_text(tree, parent))
}

/// The level oleacc answers as a tree item's value, as a number.
fn level_of(value: &str) -> Option<u32> {
    value.trim().parse().ok()
}

// ── Reading A: the dialog as it stood ──────────────────────────────────────

/// One row of the old dialog's list, read over MSAA as child `child`.
#[derive(Debug, Clone)]
struct ListRow {
    child: i64,
    msaa: Msaa,
}

impl ListRow {
    fn describe(&self) -> String {
        format!(
            "row {}: role 0x{:x}, state 0x{:x}, name {:?}",
            self.child, self.msaa.role, self.msaa.state, self.msaa.name
        )
    }
}

#[derive(Debug)]
struct ReadingA {
    control_class: String,
    rows: Vec<ListRow>,
}

fn row(
    path: &str,
    name: &str,
    parent: Option<&str>,
    syncing: bool,
    holds_all_mail: bool,
) -> FolderRow {
    FolderRow {
        path: path.to_string(),
        name: name.to_string(),
        parent: parent.map(str::to_string),
        syncing,
        subscribed: true,
        holds_all_mail,
        total: 12,
    }
}

/// The three rows the tester's report is about: a kept one, a dropped one and
/// the one that holds every message.
fn the_rows_of_the_day() -> Vec<FolderRow> {
    vec![
        row("INBOX", "Inbox", None, true, false),
        row("Archive", "Archive", None, false, false),
        row("[Gmail]/All Mail", "All Mail", None, false, true),
    ]
}

fn read_the_dialog_as_it_stood(frame: &Frame) -> Result<ReadingA, String> {
    let (dialog, control) = build_folder_choice_dialog(
        frame,
        "work@example.com",
        &the_rows_of_the_day(),
        theme::current(""),
    );
    let hwnd = control.get_handle() as isize;
    let control_class = class_name(hwnd);
    let mut rows = Vec::new();
    for child in 1..=3 {
        let msaa = msaa_of(hwnd, child).map_err(|why| format!("reading A, row {child}: {why}"))?;
        rows.push(ListRow { child, msaa });
    }
    dialog.destroy();
    Ok(ReadingA {
        control_class,
        rows,
    })
}

// ── Reading B: a native tree with check boxes ──────────────────────────────

/// One item of the scratch tree, read three ways.
#[derive(Debug, Clone)]
struct ProbeItem {
    label: &'static str,
    state_image: u32,
    msaa: Msaa,
}

impl ProbeItem {
    fn describe(&self) -> String {
        format!(
            "{:?}: state image {}, MSAA role 0x{:x}, state 0x{:x}, value {:?}",
            self.label, self.state_image, self.msaa.role, self.msaa.state, self.msaa.value
        )
    }
}

#[derive(Debug)]
struct ReadingB {
    tree_class: String,
    style_carries_checkboxes: bool,
    items: Vec<ProbeItem>,
    /// The first item's state image before Space and after `WM_KEYDOWN`
    /// `VK_SPACE` was sent to the tree with it selected.
    first_item_before_space: u32,
    first_item_after_space: u32,
}

const PROBE_LABELS: [&str; 3] = ["INBOX", "[Gmail]", "QC Docs"];

fn read_a_native_tree_with_check_boxes(frame: &Frame) -> Result<ReadingB, String> {
    let tree = TreeCtrl::builder(frame)
        .with_style(
            TreeCtrlStyle::HideRoot | TreeCtrlStyle::HasButtons | TreeCtrlStyle::LinesAtRoot,
        )
        .build();
    let hwnd = tree.get_handle() as isize;
    // SAFETY: a live window this file built; the style is read, one bit
    // added, and written back before any item exists, which is the order the
    // control documents for this style.
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        SetWindowLongPtrW(hwnd, GWL_STYLE, style | TVS_CHECKBOXES);
    }
    let style_carries_checkboxes =
        // SAFETY: a live window handle.
        unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) } & TVS_CHECKBOXES != 0;
    let root = tree
        .add_root("root", None, None)
        .ok_or("reading B: the tree gave no root")?;
    let inbox = tree
        .append_item(&root, PROBE_LABELS[0], None, None)
        .ok_or("reading B: no item for INBOX")?;
    let gmail = tree
        .append_item(&root, PROBE_LABELS[1], None, None)
        .ok_or("reading B: no item for [Gmail]")?;
    tree.append_item(&gmail, PROBE_LABELS[2], None, None)
        .ok_or("reading B: no item for QC Docs")?;
    tree.expand_all();
    let items = items_in_walk_order(hwnd);
    if items.len() != 3 {
        return Err(format!(
            "reading B: the native walk found {} items where 3 were appended",
            items.len()
        ));
    }
    set_state_image(hwnd, items[1], 2);
    let mut read = Vec::new();
    for (label, item) in PROBE_LABELS.iter().zip(&items) {
        let msaa = msaa_of(hwnd, acc_id_of(hwnd, *item))
            .map_err(|why| format!("reading B, {label:?}: {why}"))?;
        read.push(ProbeItem {
            label,
            state_image: state_image_of(hwnd, *item),
            msaa,
        });
    }
    tree.select_item(&inbox);
    send(hwnd, TVM_SELECTITEM, TVGN_CARET, items[0]);
    let first_item_before_space = state_image_of(hwnd, items[0]);
    send(hwnd, WM_KEYDOWN, VK_SPACE, 0);
    send(hwnd, WM_KEYUP, VK_SPACE, 0);
    let first_item_after_space = state_image_of(hwnd, items[0]);
    let tree_class = class_name(hwnd);
    tree.destroy();
    Ok(ReadingB {
        tree_class,
        style_carries_checkboxes,
        items: read,
        first_item_before_space,
        first_item_after_space,
    })
}

// ── Reading C: the dialog as it is now ─────────────────────────────────────

/// One item of the dialog's tree, as the control and MSAA describe it.
#[derive(Debug, Clone, PartialEq)]
struct TreeItem {
    label: String,
    parent_label: Option<String>,
    state_image: u32,
    msaa_state: i64,
    expanded: bool,
    has_children: bool,
}

/// What the built dialog's control answered.
#[derive(Debug, Clone, PartialEq)]
struct ReadingC {
    control_class: String,
    items: Vec<TreeItem>,
    /// The label of the item the cursor is on, if any.
    cursor_label: Option<String>,
    focus_is_on_the_control: bool,
}

/// What a row asked for: its name, its parent's name and whether it is kept.
#[derive(Debug, Clone)]
struct Expected {
    name: &'static str,
    parent: Option<&'static str>,
    ticked: bool,
}

/// The rows reading C is built over, nested three deep with one kept.
fn the_nested_rows() -> Vec<FolderRow> {
    vec![
        row("INBOX", "Inbox", None, true, false),
        row("[Gmail]", "[Gmail]", None, false, false),
        row("[Gmail]/QC Docs", "QC Docs", Some("[Gmail]"), false, false),
        row(
            "[Gmail]/QC Docs/QILC",
            "QILC",
            Some("[Gmail]/QC Docs"),
            false,
            false,
        ),
    ]
}

fn what_the_nested_rows_ask_for() -> Vec<Expected> {
    vec![
        Expected {
            name: "Inbox",
            parent: None,
            ticked: true,
        },
        Expected {
            name: "[Gmail]",
            parent: None,
            ticked: false,
        },
        Expected {
            name: "QC Docs",
            parent: Some("[Gmail]"),
            ticked: false,
        },
        Expected {
            name: "QILC",
            parent: Some("QC Docs"),
            ticked: false,
        },
    ]
}

/// A label starts with the folder's name; the rest is the row's facts.
fn labelled(label: &str, name: &str) -> bool {
    label == name || label.starts_with(&format!("{name},"))
}

/// Everything wrong with what the control answered, against what the rows
/// asked for. Pure over the harvest, so the companions can feed it a wrong
/// tree and hold it to complaining.
fn what_is_wrong_with(reading: &ReadingC, expected: &[Expected]) -> Vec<String> {
    let mut wrong = Vec::new();
    if reading.control_class != "SysTreeView32" {
        wrong.push(format!(
            "the control is a {:?}, not a SysTreeView32",
            reading.control_class
        ));
        return wrong;
    }
    if reading.items.len() != expected.len() {
        wrong.push(format!(
            "{} items in the tree where {} rows were given",
            reading.items.len(),
            expected.len()
        ));
    }
    for want in expected {
        let Some(item) = reading
            .items
            .iter()
            .find(|it| labelled(&it.label, want.name))
        else {
            wrong.push(format!("no item is labelled {:?}", want.name));
            continue;
        };
        match (&item.parent_label, want.parent) {
            (None, None) => {}
            (Some(parent), Some(wanted)) if labelled(parent, wanted) => {}
            (parent, wanted) => wrong.push(format!(
                "{:?} sits under {:?} where its parent is {:?}",
                want.name, parent, wanted
            )),
        }
        let wanted_image = if want.ticked { 2 } else { 1 };
        if item.state_image != wanted_image {
            wrong.push(format!(
                "{:?} has state image {} where {} means {}",
                want.name,
                item.state_image,
                wanted_image,
                if want.ticked {
                    "checked"
                } else {
                    "not checked"
                }
            ));
        }
        let checked = item.msaa_state & STATE_SYSTEM_CHECKED != 0;
        if checked != want.ticked {
            wrong.push(format!(
                "{:?} answers MSAA state 0x{:x}, {} STATE_SYSTEM_CHECKED, where the row is {}",
                want.name,
                item.msaa_state,
                if checked { "with" } else { "without" },
                if want.ticked { "kept" } else { "not kept" }
            ));
        }
        if item.msaa_state & STATE_SYSTEM_READONLY != 0 {
            wrong.push(format!(
                "{:?} answers STATE_SYSTEM_READONLY, which is what the tester heard",
                want.name
            ));
        }
        if item.has_children && !item.expanded {
            wrong.push(format!("{:?} holds folders and is not expanded", want.name));
        }
    }
    match &reading.cursor_label {
        Some(label) if labelled(label, expected[0].name) => {}
        other => wrong.push(format!(
            "the cursor is on {other:?}, not on the first row {:?}",
            expected[0].name
        )),
    }
    if !reading.focus_is_on_the_control {
        wrong.push("focus is not on the tree".to_string());
    }
    wrong
}

fn read_the_dialog_as_it_is(frame: &Frame) -> Result<ReadingC, String> {
    let (dialog, control) = build_folder_choice_dialog(
        frame,
        "Pratik at work",
        &the_nested_rows(),
        theme::current(""),
    );
    dialog.show(true);
    let hwnd = control.get_handle() as isize;
    let control_class = class_name(hwnd);
    let mut items = Vec::new();
    if control_class == "SysTreeView32" {
        for item in items_in_walk_order(hwnd) {
            let msaa = msaa_of(hwnd, acc_id_of(hwnd, item))
                .map_err(|why| format!("reading C, item {item:#x}: {why}"))?;
            items.push(TreeItem {
                label: item_text(hwnd, item),
                parent_label: parent_text(hwnd, item),
                state_image: state_image_of(hwnd, item),
                msaa_state: msaa.state,
                expanded: is_expanded(hwnd, item),
                has_children: send(hwnd, TVM_GETNEXTITEM, TVGN_CHILD, item) != 0,
            });
        }
    }
    let caret = send(hwnd, TVM_GETNEXTITEM, TVGN_CARET, 0);
    let cursor_label =
        (control_class == "SysTreeView32" && caret != 0).then(|| item_text(hwnd, caret));
    // SAFETY: reads which window holds focus; touches nothing.
    let focus_is_on_the_control = unsafe { GetFocus() } == hwnd;
    dialog.destroy();
    Ok(ReadingC {
        control_class,
        items,
        cursor_label,
        focus_is_on_the_control,
    })
}

// ── The one window session ─────────────────────────────────────────────────

#[derive(Debug)]
struct Harvest {
    reading_a: ReadingA,
    reading_b: ReadingB,
    reading_c: ReadingC,
}

fn take_the_harvest() -> Result<Harvest, String> {
    if std::mem::size_of::<Variant>() != 24 {
        return Err("VARIANT is not 24 bytes here, so the reader's layout is wrong".to_string());
    }
    if std::mem::size_of::<TreeItemW>() != 56 {
        return Err("TVITEMW is not 56 bytes here, so the reader's layout is wrong".to_string());
    }
    if theme::current("").is_none() {
        return Err(
            "theme::current(\"\") answers no palette, so Windows High Contrast is on and the \
             dialog would be built unpainted, which is not the dialog the tester used"
                .to_string(),
        );
    }
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let taken: Result<Harvest, String> = (|| {
                let frame = Frame::builder().build();
                let reading_a = read_the_dialog_as_it_stood(&frame)?;
                let reading_b = read_a_native_tree_with_check_boxes(&frame)?;
                let reading_c = read_the_dialog_as_it_is(&frame)?;
                Ok(Harvest {
                    reading_a,
                    reading_b,
                    reading_c,
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

// ── Reading A ──────────────────────────────────────────────────────────────

#[test]
fn test_reading_a_the_rows_of_the_old_list_answered_what_the_code_claimed() {
    // What `names::CheckedRows` claimed: every row a check button, the kept
    // row checked and the others not, read-only on none. Green means the
    // object was right and NVDA read something else for the row; red means
    // the snapshot was wrong. The harvest is quoted either way.
    let reading = &the_harvest().reading_a;
    let mut wrong = Vec::new();
    let quoted: Vec<String> = reading.rows.iter().map(ListRow::describe).collect();
    for row in &reading.rows {
        if row.msaa.role != ROLE_SYSTEM_CHECKBUTTON {
            wrong.push(format!("not a check button: {}", row.describe()));
        }
        let checked = row.msaa.state & STATE_SYSTEM_CHECKED != 0;
        if checked != (row.child == 1) {
            wrong.push(format!("the checked bit is wrong: {}", row.describe()));
        }
        if row.msaa.state & STATE_SYSTEM_READONLY != 0 {
            wrong.push(format!("read-only: {}", row.describe()));
        }
    }
    complain(
        &format!(
            "the {} the old dialog held should have answered what its object claimed; it answered\n  {}",
            reading.control_class,
            quoted.join("\n  ")
        ),
        &wrong,
    );
}

// ── Reading B ──────────────────────────────────────────────────────────────

#[test]
fn test_reading_b_a_native_tree_with_check_boxes_carries_its_state_where_nvda_reads() {
    let reading = &the_harvest().reading_b;
    let mut wrong = Vec::new();
    let quoted: Vec<String> = reading.items.iter().map(ProbeItem::describe).collect();
    if reading.tree_class != "SysTreeView32" {
        wrong.push(format!("the tree is a {:?}", reading.tree_class));
    }
    if !reading.style_carries_checkboxes {
        wrong.push("TVS_CHECKBOXES did not take".to_string());
    }
    for (at, item) in reading.items.iter().enumerate() {
        let ticked = at == 1;
        let wanted_image = if ticked { 2 } else { 1 };
        if item.state_image != wanted_image {
            wrong.push(format!(
                "TVM_GETITEMSTATE answers state image {} where {} was set: {}",
                item.state_image,
                wanted_image,
                item.describe()
            ));
        }
        // Windows' own tree proxy answers check button, not outline item, for
        // an item under TVS_CHECKBOXES: measured 2026-09-18, role 0x2c on all
        // three, which is the role the tester's report names. The first draft
        // of this reading expected 0x24 and was corrected by the harvest.
        if item.msaa.role != ROLE_SYSTEM_CHECKBUTTON {
            wrong.push(format!("not a check button over MSAA: {}", item.describe()));
        }
        if (item.msaa.state & STATE_SYSTEM_CHECKED != 0) != ticked {
            wrong.push(format!(
                "MSAA's checked bit does not follow the state image: {}",
                item.describe()
            ));
        }
        if item.msaa.state & STATE_SYSTEM_READONLY != 0 {
            wrong.push(format!("read-only: {}", item.describe()));
        }
    }
    let levels: Vec<Option<u32>> = reading
        .items
        .iter()
        .map(|item| level_of(&item.msaa.value))
        .collect();
    match (levels[1], levels[2]) {
        (Some(parent), Some(child)) if child == parent + 1 => {}
        _ => wrong.push(format!(
            "accValue does not carry the level: [Gmail] answers {:?} and QC Docs under it {:?}",
            reading.items[1].msaa.value, reading.items[2].msaa.value
        )),
    }
    complain(
        &format!(
            "a SysTreeView32 with TVS_CHECKBOXES should carry the ticked item's state where NVDA \
             reads it; it answered\n  {}",
            quoted.join("\n  ")
        ),
        &wrong,
    );
}

#[test]
fn test_reading_b_space_on_the_tree_and_what_it_did_to_the_state_image() {
    // The finding this pins is whether the control toggles the state image on
    // Space by itself under wx, or whether the dialog has to. Either is a fact
    // about the tree the dialog is built on; the number below is the one the
    // first run answered, and `wx_folder_choice.rs` says what it does about it.
    let reading = &the_harvest().reading_b;
    assert_eq!(
        reading.first_item_before_space, 1,
        "the first item should start unchecked (state image 1)"
    );
    assert_eq!(
        reading.first_item_after_space, 2,
        "after Space the state image is {}, so the control did not toggle it on its own",
        reading.first_item_after_space
    );
}

// ── Reading C ──────────────────────────────────────────────────────────────

#[test]
fn test_reading_c_the_dialog_is_a_tree_whose_kept_folder_is_checked_where_nvda_reads() {
    let reading = &the_harvest().reading_c;
    let wrong = what_is_wrong_with(reading, &what_the_nested_rows_ask_for());
    complain(
        &format!(
            "Folders to Keep Up to Date should be a SysTreeView32 nested by each row's parent, \
             the kept row checked on TVM_GETITEMSTATE and over MSAA, every branch open and the \
             cursor on the first row; it answered {reading:#?}"
        ),
        &wrong,
    );
}

fn a_correct_reading() -> ReadingC {
    let item = |label: &str, parent: Option<&str>, state_image: u32, has_children: bool| TreeItem {
        label: format!("{label}, 12 messages"),
        parent_label: parent.map(|it| format!("{it}, 12 messages")),
        state_image,
        msaa_state: if state_image == 2 {
            STATE_SYSTEM_CHECKED
        } else {
            0
        },
        expanded: has_children,
        has_children,
    };
    ReadingC {
        control_class: "SysTreeView32".to_string(),
        items: vec![
            item("Inbox", None, 2, false),
            item("[Gmail]", None, 1, true),
            item("QC Docs", Some("[Gmail]"), 1, true),
            item("QILC", Some("QC Docs"), 1, false),
        ],
        cursor_label: Some("Inbox, 12 messages".to_string()),
        focus_is_on_the_control: true,
    }
}

#[test]
fn test_the_reading_complains_when_the_kept_folder_is_not_checked() {
    let expected = what_the_nested_rows_ask_for();
    assert!(
        what_is_wrong_with(&a_correct_reading(), &expected).is_empty(),
        "the reading must accept a correct tree before it can be held to refusing a wrong one"
    );
    let mut unchecked = a_correct_reading();
    unchecked.items[0].state_image = 1;
    unchecked.items[0].msaa_state = 0;

    let wrong = what_is_wrong_with(&unchecked, &expected);

    assert!(
        wrong.iter().any(|it| it.contains("state image 1")),
        "the reading did not complain about the state: {wrong:?}"
    );
    assert!(
        wrong
            .iter()
            .any(|it| it.contains("without STATE_SYSTEM_CHECKED")),
        "the reading did not complain about the MSAA state: {wrong:?}"
    );
}

#[test]
fn test_the_reading_complains_when_a_folder_sits_under_the_wrong_parent() {
    let expected = what_the_nested_rows_ask_for();
    let mut flattened = a_correct_reading();
    flattened.items[3].parent_label = Some("[Gmail], 12 messages".to_string());

    let wrong = what_is_wrong_with(&flattened, &expected);

    assert!(
        wrong.iter().any(|it| it.contains("\"QILC\" sits under")),
        "the reading did not complain about the parent: {wrong:?}"
    );
}

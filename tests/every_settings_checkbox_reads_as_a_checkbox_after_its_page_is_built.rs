//! Every checkbox on every Settings page answers as a check box, and toggles,
//! after its page is built the way the tab row builds it.
//!
//! The tester's words on 2026-09-17 against `1.0.0-alpha.1` (`7d57cd49`) were
//! "every checkbox on every tab except General is read as a button. Pressing
//! one does not appear to change its state" (#67). Since 09-09 the six pages
//! after General are built the first time their tab is shown, and the dialog
//! painted their empty panels with the theme before that. wxWidgets hands a
//! parent's foreground colour to every child created after it
//! (`src/common/wincmn.cpp:1524-1552`) and makes a checkbox given a colour
//! owner-drawn (`src/msw/control.cpp:422-444`), so its `BS_CHECKBOX` style
//! became `BS_OWNERDRAW`, which Windows' standard accessible object reports as
//! a push button with no checked state. General was spared because its boxes
//! existed before the paint.
//!
//! What this measures, and on which channel. NVDA reads a `Button`-class
//! window through `IAccessible`, because that class is on its list of window
//! classes it never asks UI Automation about, so the reading here is the MSAA
//! object `AccessibleObjectFromWindow(hwnd, OBJID_CLIENT)` answers: its role
//! and its state, before and after a click. The style bits are read beside
//! them, because the style is the mechanism and the role is what is heard.
//!
//! The reader is sixty lines of user32 and oleacc declarations and three
//! vtable slots rather than a feature on the `windows` crate: that dependency
//! sits under the library's own `[target.'cfg(windows)'.dependencies]`, so a
//! feature added for a test is compiled into the shipping binary, and
//! `Win32_UI_Accessibility` brings the whole UI Automation surface with it.
//! The shape is the probe's that diagnosed #67, measured on both trees.
//!
//! One window session serves three tests. The budget is one `wxdragon::main`
//! per process (`tests/theme_reach.rs` records the "initializing twice?" hang
//! a second one produced), so the windows are built once inside a `OnceLock`
//! by whichever test asks first, every reading is harvested into plain
//! values, and the three tests assert over the harvest. The initialiser
//! stores a `Result` and panics on nothing itself: a panic inside
//! `OnceLock::get_or_init` leaves the cell empty and the next test would
//! spend the budget a second time.
//!
//! `BM_CLICK` hit-tests a rectangle, so a control that has not been laid out
//! ignores it; General's boxes had no rectangle before a layout when the probe
//! ran. The dialog is laid out before any click, and where `BM_CLICK` still
//! moves nothing the notification a clicked button sends its parent,
//! `WM_COMMAND` with `BN_CLICKED`, is sent by hand, which wxWidgets handles in
//! `wxCheckBox::MSWCommand`. A row that moves on neither is wrong, not skipped.

#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::data::config::AppConfig;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::accessibility::names::set_accessible_name;
use wixen_mail::presentation::theme;
use wixen_mail::presentation::wx_settings;
use wxdragon::prelude::*;

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const BM_CLICK: u32 = 0x00F5;
const WM_COMMAND: u32 = 0x0111;
const BN_CLICKED: usize = 0;
const GWL_STYLE: i32 = -16;
const VT_I4: u16 = 3;
const CHILDID_SELF: i64 = 0;

/// The low nibble of a `Button`-class window's style says what kind it is.
const BS_TYPEMASK: u32 = 0xF;
const BS_PUSHBUTTON: u32 = 0x0;
const BS_CHECKBOX: u32 = 0x2;
const BS_3STATE: u32 = 0x5;
const BS_GROUPBOX: u32 = 0x7;
const BS_OWNERDRAW: u32 = 0xB;

/// What MSAA answers for role, and the one state bit this file reads.
const ROLE_SYSTEM_PUSHBUTTON: i64 = 0x2b;
const ROLE_SYSTEM_CHECKBUTTON: i64 = 0x2c;
const STATE_SYSTEM_CHECKED: i64 = 0x10;

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
/// and the payload at offset 8. The payload union holds a BRECORD of two
/// pointers, which is why the whole thing is 24 rather than 16.
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
type GetVariantFn = unsafe extern "system" fn(*mut c_void, Variant, *mut Variant) -> Hresult;

// IAccessible's vtable: IUnknown (3), IDispatch (4), then get_accParent (7),
// get_accChildCount (8), get_accChild (9), get_accName (10), get_accValue
// (11), get_accDescription (12), get_accRole (13), get_accState (14).
const VTBL_RELEASE: usize = 2;
const VTBL_GET_ACC_ROLE: usize = 13;
const VTBL_GET_ACC_STATE: usize = 14;

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
    fn GetParent(hwnd: isize) -> isize;
    fn GetDlgCtrlID(hwnd: isize) -> i32;
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

fn window_text(hwnd: isize) -> String {
    let mut buffer = [0u16; 512];
    // SAFETY: the buffer is as long as the count says.
    let len = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

fn style_of(hwnd: isize) -> u32 {
    // SAFETY: a live window handle.
    (unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) } as u64 & 0xFFFF_FFFF) as u32
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

/// One MSAA reading of a window's client object: its role and its state, or
/// why neither could be read.
fn msaa_of(hwnd: isize) -> Result<(i64, i64), String> {
    let mut object: *mut c_void = std::ptr::null_mut();
    // SAFETY: a live window handle; the object is released before returning.
    let hr =
        unsafe { AccessibleObjectFromWindow(hwnd, OBJID_CLIENT, &IID_IACCESSIBLE, &mut object) };
    if hr < 0 || object.is_null() {
        return Err(format!("AccessibleObjectFromWindow failed 0x{hr:x}"));
    }
    // SAFETY: `object` is a live IAccessible; the slots are IAccessible's.
    unsafe {
        let get_role: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_ROLE));
        let get_state: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_STATE));
        let mut role = Variant::empty();
        let hr_role = get_role(object, Variant::child_self(), &mut role);
        let mut state = Variant::empty();
        let hr_state = get_state(object, Variant::child_self(), &mut state);
        release(object);
        if hr_role < 0 || role.vt != VT_I4 {
            return Err(format!("get_accRole hr=0x{hr_role:x} vt={}", role.vt));
        }
        if hr_state < 0 || state.vt != VT_I4 {
            return Err(format!("get_accState hr=0x{hr_state:x} vt={}", state.vt));
        }
        Ok((role.val & 0xFFFF_FFFF, state.val & 0xFFFF_FFFF))
    }
}

/// The notification a button control sends its parent when it is clicked,
/// sent by hand, so the reading does not depend on `BM_CLICK`'s hit test.
fn send_bn_clicked(hwnd: isize) {
    // SAFETY: live window handles.
    unsafe {
        let parent = GetParent(hwnd);
        let id = GetDlgCtrlID(hwnd) as usize & 0xFFFF;
        SendMessageW(parent, WM_COMMAND, (BN_CLICKED << 16) | id, hwnd);
    }
}

fn checked(state: i64) -> bool {
    state & STATE_SYSTEM_CHECKED != 0
}

fn kind_name(style: u32) -> &'static str {
    match style & BS_TYPEMASK {
        BS_PUSHBUTTON => "BS_PUSHBUTTON",
        BS_CHECKBOX => "BS_CHECKBOX",
        BS_3STATE => "BS_3STATE",
        BS_GROUPBOX => "BS_GROUPBOX",
        BS_OWNERDRAW => "BS_OWNERDRAW",
        _ => "another kind",
    }
}

fn role_name(role: i64) -> &'static str {
    match role {
        ROLE_SYSTEM_PUSHBUTTON => "ROLE_SYSTEM_PUSHBUTTON",
        ROLE_SYSTEM_CHECKBUTTON => "ROLE_SYSTEM_CHECKBUTTON",
        _ => "another role",
    }
}

/// One `Button`-class window under a later page that is neither a group box
/// nor a push button: what it is, what it answers, and what a click did.
#[derive(Debug, Clone)]
struct Row {
    theme: &'static str,
    page: &'static str,
    label: String,
    style: u32,
    role: i64,
    state_before: i64,
    state_after_bm_click: i64,
    /// `None` where `BM_CLICK` moved the checked bit and nothing more was
    /// sent; `Some` is the state after `BN_CLICKED` sent by hand.
    state_after_bn_clicked: Option<i64>,
}

impl Row {
    fn describe(&self) -> String {
        format!(
            "[{}] {}: \"{}\" style 0x{:08x} ({}), role 0x{:x} ({}), state 0x{:x} then 0x{:x} after BM_CLICK{}",
            self.theme,
            self.page,
            self.label,
            self.style,
            kind_name(self.style),
            self.role,
            role_name(self.role),
            self.state_before,
            self.state_after_bm_click,
            match self.state_after_bn_clicked {
                Some(state) => format!(" then 0x{state:x} after BN_CLICKED"),
                None => String::new(),
            }
        )
    }

    fn toggled(&self) -> bool {
        match self.state_after_bn_clicked {
            None => checked(self.state_before) != checked(self.state_after_bm_click),
            Some(after) => checked(self.state_after_bm_click) != checked(after),
        }
    }
}

/// A push button read through the same reader, so the companion can hold
/// the reader to seeing one as not a check box.
#[derive(Debug, Clone)]
struct PushButton {
    style: u32,
    role: i64,
}

/// The fault in isolation: a checkbox created under a fresh panel that was
/// painted first and then named, which is the one cell of the probe's four
/// that answered push button.
#[derive(Debug, Clone)]
struct Mechanism {
    style: u32,
    role: i64,
    state_before: i64,
    state_after_bn_clicked: i64,
}

/// Everything the window session read, as plain values; no handle survives it.
#[derive(Debug)]
struct Harvest {
    rows: Vec<Row>,
    ok_button: PushButton,
    mechanism: Mechanism,
}

/// The two themes the dialog is built in: the default the tester's profile
/// would have carried on a fresh install, and dark, which the tester runs.
const THEMES: [&str; 2] = ["", "dark"];

/// The six later tabs, in the notebook's order, and whether each holds a
/// checkbox. Calendar & PIM has none, and the reading holds that too.
const LATER_PAGES: [(usize, &str, bool); 6] = [
    (1, "Compose", true),
    (2, "Reading", true),
    (3, "Permissions", true),
    (4, "Calendar & PIM", false),
    (5, "Feedback", true),
    (6, "Advanced", true),
];

/// The `Button`-class descendants of `panel` that are neither group boxes
/// nor push buttons, which on a Settings page means the check boxes, however
/// they are currently styled.
fn checkbox_like_windows_under(panel: &Panel) -> Vec<isize> {
    descendants_of(panel.get_handle() as isize)
        .into_iter()
        .filter(|hwnd| class_name(*hwnd) == "Button")
        .filter(|hwnd| {
            let kind = style_of(*hwnd) & BS_TYPEMASK;
            kind != BS_GROUPBOX && kind != BS_PUSHBUTTON
        })
        .collect()
}

/// One page reached the way the arrow keys reach it, laid out, and every
/// check box on it read and clicked.
fn read_the_page(
    theme_name: &'static str,
    widgets: &wx_settings::SettingsWidgets,
    tab: usize,
    page: &'static str,
    panel: &Panel,
) -> Result<Vec<Row>, String> {
    widgets.notebook.set_selection(tab);
    widgets.dialog.layout();
    let mut rows = Vec::new();
    for hwnd in checkbox_like_windows_under(panel) {
        let label = window_text(hwnd);
        let style = style_of(hwnd);
        let (role, state_before) =
            msaa_of(hwnd).map_err(|why| format!("[{theme_name}] {page}: \"{label}\": {why}"))?;
        // SAFETY: a live window handle.
        unsafe { SendMessageW(hwnd, BM_CLICK, 0, 0) };
        let (_, state_after_bm_click) = msaa_of(hwnd)
            .map_err(|why| format!("[{theme_name}] {page}: \"{label}\" after BM_CLICK: {why}"))?;
        let state_after_bn_clicked = if checked(state_before) == checked(state_after_bm_click) {
            send_bn_clicked(hwnd);
            let (_, after) = msaa_of(hwnd).map_err(|why| {
                format!("[{theme_name}] {page}: \"{label}\" after BN_CLICKED: {why}")
            })?;
            Some(after)
        } else {
            None
        };
        rows.push(Row {
            theme: theme_name,
            page,
            label,
            style,
            role,
            state_before,
            state_after_bm_click,
            state_after_bn_clicked,
        });
    }
    Ok(rows)
}

/// The OK button, a direct child of the dialog with that text.
fn read_the_ok_button(dialog: &Dialog) -> Result<PushButton, String> {
    let dialog_hwnd = dialog.get_handle() as isize;
    for hwnd in descendants_of(dialog_hwnd) {
        // SAFETY: a live window handle.
        let parent = unsafe { GetParent(hwnd) };
        if parent == dialog_hwnd && class_name(hwnd) == "Button" && window_text(hwnd) == "OK" {
            let (role, _) = msaa_of(hwnd).map_err(|why| format!("the OK button: {why}"))?;
            return Ok(PushButton {
                style: style_of(hwnd),
                role,
            });
        }
    }
    Err("no OK button was found among the dialog's children".to_string())
}

/// One dialog built with `theme_name`, shown, every later page reached and
/// read, then destroyed.
fn read_one_dialog(
    theme_name: &'static str,
    frame: &Frame,
    a11y: &Arc<Accessibility>,
) -> Result<(Vec<Row>, PushButton), String> {
    let config = AppConfig {
        theme: theme_name.to_string(),
        ..AppConfig::default()
    };
    if theme::current(&config.theme).is_none() {
        return Err(format!(
            "theme::current({theme_name:?}) answers no palette, so Windows High Contrast is on, \
             nothing is painted, and a run that painted nothing cannot see the fault"
        ));
    }
    let widgets = wx_settings::build_settings_dialog(frame, &config, &[], false, a11y);
    widgets.dialog.show(true);
    let ok_button = read_the_ok_button(&widgets.dialog)?;
    let pages: [(usize, &'static str, &Panel); 6] = [
        (1, "Compose", &widgets.compose_panel),
        (2, "Reading", &widgets.reading_panel),
        (3, "Permissions", &widgets.permissions_panel),
        (4, "Calendar & PIM", &widgets.pim_panel),
        (5, "Feedback", &widgets.feedback_panel),
        (6, "Advanced", &widgets.advanced_panel),
    ];
    let mut rows = Vec::new();
    for (tab, page, panel) in pages {
        rows.extend(read_the_page(theme_name, &widgets, tab, page, panel)?);
    }
    widgets.dialog.destroy();
    Ok((rows, ok_button))
}

/// The fault in isolation: a fresh panel painted, then a check box built
/// under it and named the way every Settings check box is named.
fn read_the_mechanism(frame: &Frame) -> Result<Mechanism, String> {
    let Some(palette) = theme::current("") else {
        return Err("theme::current(\"\") answers no palette for the mechanism cell".to_string());
    };
    let panel = Panel::builder(frame).build();
    theme::paint(&panel, palette.main_surface());
    let checkbox = CheckBox::builder(&panel).with_label("Probe").build();
    set_accessible_name(&checkbox, "Probe");
    let hwnd = checkbox.get_handle() as isize;
    let style = style_of(hwnd);
    let (role, state_before) = msaa_of(hwnd).map_err(|why| format!("the mechanism cell: {why}"))?;
    send_bn_clicked(hwnd);
    let (_, state_after_bn_clicked) =
        msaa_of(hwnd).map_err(|why| format!("the mechanism cell after BN_CLICKED: {why}"))?;
    panel.destroy();
    Ok(Mechanism {
        style,
        role,
        state_before,
        state_after_bn_clicked,
    })
}

/// Everything, inside one `wxdragon::main`, with every failure carried out
/// as a value: nothing in here panics, for the reason the file comment gives.
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
                let mut rows = Vec::new();
                let mut ok_button = None;
                for theme_name in THEMES {
                    let (theme_rows, ok) = read_one_dialog(theme_name, &frame, &a11y)?;
                    rows.extend(theme_rows);
                    ok_button.get_or_insert(ok);
                }
                let mechanism = read_the_mechanism(&frame)?;
                Ok(Harvest {
                    rows,
                    ok_button: ok_button.ok_or_else(|| "no dialog was read".to_string())?,
                    mechanism,
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

#[test]
fn test_every_checkbox_on_a_later_page_answers_check_button_after_set_selection() {
    let harvest = the_harvest();
    let mut wrong = Vec::new();
    for theme_name in THEMES {
        for (_, page, has_checkboxes) in LATER_PAGES {
            let on_this_page = harvest
                .rows
                .iter()
                .filter(|row| row.theme == theme_name && row.page == page)
                .count();
            if has_checkboxes && on_this_page == 0 {
                wrong.push(format!(
                    "[{theme_name}] {page}: no check box was found, so the page was not read"
                ));
            }
            if !has_checkboxes && on_this_page != 0 {
                wrong.push(format!(
                    "[{theme_name}] {page}: {on_this_page} check box(es) on a page that has none"
                ));
            }
        }
    }
    for row in &harvest.rows {
        let kind = row.style & BS_TYPEMASK;
        if kind != BS_CHECKBOX && kind != BS_3STATE {
            wrong.push(format!("not a check box by style: {}", row.describe()));
        } else if row.role != ROLE_SYSTEM_CHECKBUTTON {
            wrong.push(format!("not a check button by role: {}", row.describe()));
        }
    }
    complain(
        "every check box on a later Settings page should be BS_CHECKBOX or BS_3STATE and answer \
         ROLE_SYSTEM_CHECKBUTTON over MSAA",
        &wrong,
    );
}

#[test]
fn test_bm_click_toggles_the_checked_state_on_every_later_page() {
    let harvest = the_harvest();
    let wrong: Vec<String> = harvest
        .rows
        .iter()
        .filter(|row| !row.toggled())
        .map(|row| format!("the checked state did not move: {}", row.describe()))
        .collect();
    complain(
        "a click on every check box on a later Settings page should move its STATE_SYSTEM_CHECKED \
         bit",
        &wrong,
    );
}

#[test]
fn test_the_reading_tells_a_push_button_from_a_check_box() {
    let harvest = the_harvest();
    let mut wrong = Vec::new();
    let ok = &harvest.ok_button;
    if ok.style & BS_TYPEMASK != BS_PUSHBUTTON {
        wrong.push(format!(
            "the OK button's style is 0x{:08x} ({}), not BS_PUSHBUTTON",
            ok.style,
            kind_name(ok.style)
        ));
    }
    if ok.role != ROLE_SYSTEM_PUSHBUTTON {
        wrong.push(format!(
            "the OK button answers role 0x{:x} ({}), not ROLE_SYSTEM_PUSHBUTTON",
            ok.role,
            role_name(ok.role)
        ));
    }
    let cell = &harvest.mechanism;
    if cell.style & BS_TYPEMASK != BS_OWNERDRAW {
        wrong.push(format!(
            "a check box built under a painted, then named, panel is 0x{:08x} ({}), not \
             BS_OWNERDRAW, so the fault this reading exists to see is not reproduced here",
            cell.style,
            kind_name(cell.style)
        ));
    }
    if cell.role != ROLE_SYSTEM_PUSHBUTTON {
        wrong.push(format!(
            "that check box answers role 0x{:x} ({}), not the ROLE_SYSTEM_PUSHBUTTON the fault \
             produces",
            cell.role,
            role_name(cell.role)
        ));
    }
    if checked(cell.state_before) != checked(cell.state_after_bn_clicked) {
        wrong.push(format!(
            "that check box's state moved from 0x{:x} to 0x{:x} on BN_CLICKED, where the fault \
             leaves it still",
            cell.state_before, cell.state_after_bn_clicked
        ));
    }
    complain(
        "the reader should see the OK button as a push button and see the #67 fault in a check \
         box created under a painted, then named, panel",
        &wrong,
    );
}

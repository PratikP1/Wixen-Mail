//! A spin control's typing field keeps its name in the program's own
//! condition, read the way the Accessibility scan reads it.
//!
//! **What the program does that a test process did not.** The main window
//! builds its message preview, a WebView2 browser, before any dialog names a
//! spin control. Measured by 12-06.1 on 2026-09-24: once a WebView2 has been
//! built in a process whose annotation service has not yet written anything,
//! every later `IAccPropServices::SetHwndPropStr` in that process answers
//! `S_OK` and stores nothing, on any window, from any thread. The bisect
//! across processes (Accessibility run 35970583171) found it: every process
//! whose first annotation write came before the preview kept the name, and
//! every one whose first write came after it kept none. One write before the
//! browser, even onto a window destroyed straight after, and every later write
//! is kept. Why the browser does this is not known.
//!
//! `tests/every_spin_control_names_the_field_a_person_types_in.rs` built no
//! browser, so it read every field named while the running program's scan
//! found them nameless (ledger 593). This reading builds one first, the way
//! the program does.
//!
//! How it reads, and why three processes. A browser and `wxdragon::main` can
//! each be had once per process, and the two conditions this file compares,
//! the store readied before the browser and not, cannot share one: a write
//! after the browser never recovers. So each condition is a child process of
//! this test binary that builds the windows and names them, and that child
//! starts a grandchild that reads them, because a screen reader reads from a
//! process of its own. Nothing in the writing process reads a field's
//! accessible object before the reader has. The reader takes MSAA twice, by
//! the field's own object and by the walk `scripts/msaa-names.ps1` takes,
//! `AccessibleChildren` from the dialog's client object down to the editable
//! text whose window is the field, and UI Automation by `ElementFromHandle`
//! on the field, which is the handle Tab lands on.
//!
//! The windows are built hidden and never shown, so nothing here moves the
//! foreground. The Send Later and event form builders are the ones the
//! program's scan targets call, with no palette, as a fresh profile has.

#![cfg(windows)]

use std::collections::HashMap;
use std::ffi::c_void;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::new_item::ItemKind;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::accessibility::names::{
    ready_the_annotation_store, set_accessible_name,
};
use wixen_mail::presentation::date_display::{
    Clock, DateOrder, DateSettings, DateStyle, DateWording,
};
use wixen_mail::presentation::wx_item_form::{Chrome, build_item_form_dialog};
use wixen_mail::presentation::wx_send_later;
use wxdragon::prelude::*;
use wxdragon::widgets::{WebView, WebViewBackend};

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const VT_I4: u16 = 3;
const VT_DISPATCH: u16 = 9;
const CHILDID_SELF: i64 = 0;
/// What MSAA answers for an edit field (oleacc.h).
const ROLE_SYSTEM_TEXT: i64 = 0x2a;
/// The up-down control's message asking for its buddy (commctrl.h).
const UDM_GETBUDDY: u32 = 0x400 + 106;
/// `PeekMessageW` takes the message off the queue (winuser.h).
const PM_REMOVE: u32 = 1;

/// How many spin controls each window holds, so a window that loses one, or
/// a reading that finds none, is a red rather than a pass over nothing.
const SEND_LATER: (&str, usize) = ("Send Later", 4);
const THE_EVENT_FORM: (&str, usize) = ("the event form", 12);
const BARE: (&str, usize) = ("a spin control named on its arrows alone", 1);
const BARE_WORDS: &str = "Named on the arrows alone";

/// The child's two conditions: the store readied before the browser, as the
/// program's startup does it, and not, which is the running program before
/// 12-06.1's fix.
const READIED: &str = "readied";
const NOT_READIED: &str = "not-readied";

/// The environment variables handing a child its condition, and a grandchild
/// its fields.
const THE_CONDITION: &str = "WIXEN_SCAN_READING_CONDITION";
const THE_FIELDS_TO_READ: &str = "WIXEN_SCAN_READING_FIELDS";
/// The two ignored tests only a parent starts.
const THE_WINDOW_PROCESS: &str = "the_window_process_builds_names_and_has_its_fields_read";
const THE_READER_PROCESS: &str = "the_reader_process_reads_each_field_it_is_handed";
/// Line markers, separated by tabs.
const A_FIELD_HEARD: &str = "HEARD\t";
const A_FIELD_REPORTED: &str = "FIELD\t";
const A_FAILURE: &str = "FAILED\t";

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
    fn child_self() -> Self {
        Variant {
            vt: VT_I4,
            val: CHILDID_SELF,
            ..Variant::empty()
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
type QueryInterfaceFn =
    unsafe extern "system" fn(*mut c_void, *const Guid, *mut *mut c_void) -> Hresult;
type GetCountFn = unsafe extern "system" fn(*mut c_void, *mut i32) -> Hresult;
type GetVariantFn = unsafe extern "system" fn(*mut c_void, Variant, *mut Variant) -> Hresult;
type GetBstrFn = unsafe extern "system" fn(*mut c_void, Variant, *mut *mut u16) -> Hresult;

// IAccessible's vtable: IUnknown (3), IDispatch (4), then get_accParent (7),
// get_accChildCount (8), get_accChild (9), get_accName (10), get_accValue
// (11), get_accDescription (12), get_accRole (13).
const VTBL_QUERY_INTERFACE: usize = 0;
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
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
    fn GetParent(hwnd: isize) -> isize;
    fn PeekMessageW(message: *mut [u8; 48], hwnd: isize, first: u32, last: u32, remove: u32)
    -> i32;
    fn TranslateMessage(message: *const [u8; 48]) -> i32;
    fn DispatchMessageW(message: *const [u8; 48]) -> isize;
}

#[link(name = "oleacc")]
unsafe extern "system" {
    fn AccessibleObjectFromWindow(
        hwnd: isize,
        id_object: u32,
        riid: *const Guid,
        out: *mut *mut c_void,
    ) -> Hresult;
    fn AccessibleChildren(
        container: *mut c_void,
        start: i32,
        count: i32,
        children: *mut Variant,
        obtained: *mut i32,
    ) -> Hresult;
    fn WindowFromAccessibleObject(object: *mut c_void, hwnd: *mut isize) -> Hresult;
}

#[link(name = "oleaut32")]
unsafe extern "system" {
    fn SysStringLen(s: *mut u16) -> u32;
    fn SysFreeString(s: *mut u16);
}

// ── Finding the spin controls, in the writing process ──────────────────────

thread_local! {
    static FOUND: std::cell::RefCell<Vec<isize>> = const { std::cell::RefCell::new(Vec::new()) };
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

/// One spin control to read: which window, the dialog it sits in, the arrows
/// and the field beside them. Handles only; nothing here reads a name.
#[derive(Debug, Clone, Copy)]
struct ToRead {
    dialog: isize,
    arrows: isize,
    field: isize,
}

/// Every spin control under `dialog`, found by the arrows' class. A tab row's
/// own scrolling up-down, a child of `_wx_SysTabCtl32`, is not one.
fn the_spinners_under(dialog: isize) -> Vec<ToRead> {
    descendants_of(dialog)
        .into_iter()
        .filter(|hwnd| class_name(*hwnd) == "msctls_updown32")
        // SAFETY: a live window handle.
        .filter(|hwnd| class_name(unsafe { GetParent(*hwnd) }) != "_wx_SysTabCtl32")
        .map(|arrows| ToRead {
            dialog,
            arrows,
            // SAFETY: a live window this process built; no pointer passed.
            field: unsafe { SendMessageW(arrows, UDM_GETBUDDY, 0, 0) },
        })
        .collect()
}

// ── MSAA and UI Automation, in the reading process ─────────────────────────

unsafe fn vtable_entry(object: *mut c_void, index: usize) -> *const c_void {
    // SAFETY: a COM object is a pointer to its vtable.
    unsafe {
        let vtable = *(object as *const *const *const c_void);
        *vtable.add(index)
    }
}

unsafe fn release(object: *mut c_void) {
    // SAFETY: a live COM object; its Release slot.
    unsafe {
        let release: ReleaseFn = std::mem::transmute(vtable_entry(object, VTBL_RELEASE));
        release(object);
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

/// An accessible object's own name, empty when it gives none.
unsafe fn name_of(object: *mut c_void) -> String {
    // SAFETY: a live IAccessible; the slot is get_accName.
    unsafe {
        let get_name: GetBstrFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_NAME));
        let mut s: *mut u16 = std::ptr::null_mut();
        match get_name(object, Variant::child_self(), &mut s) >= 0 {
            true => take_bstr(s),
            false => String::new(),
        }
    }
}

/// An accessible object's own role, or 0 when it gives none.
unsafe fn role_of(object: *mut c_void) -> i64 {
    // SAFETY: a live IAccessible; the slot is get_accRole.
    unsafe {
        let get_role: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_ROLE));
        let mut role = Variant::empty();
        match get_role(object, Variant::child_self(), &mut role) >= 0 && role.vt == VT_I4 {
            true => role.val & 0xFFFF_FFFF,
            false => 0,
        }
    }
}

/// The client object of `hwnd`, as a reader asks for it.
fn client_object_of(hwnd: isize) -> Result<*mut c_void, String> {
    let mut object: *mut c_void = std::ptr::null_mut();
    // SAFETY: the out pointer is a local; on success it holds an IAccessible.
    let hr =
        unsafe { AccessibleObjectFromWindow(hwnd, OBJID_CLIENT, &IID_IACCESSIBLE, &mut object) };
    if hr < 0 || object.is_null() {
        return Err(format!("(AccessibleObjectFromWindow failed 0x{hr:x})"));
    }
    Ok(object)
}

/// The MSAA name of `hwnd`'s own client object.
fn msaa_name_of(hwnd: isize) -> String {
    match client_object_of(hwnd) {
        Ok(object) => {
            // SAFETY: a live IAccessible, released once here.
            unsafe {
                let name = name_of(object);
                release(object);
                name
            }
        }
        Err(why) => why,
    }
}

/// Every editable text the scan's walk reaches under `dialog`, by the window
/// it belongs to, with its MSAA name: from the dialog's client object down
/// through `AccessibleChildren`, as `scripts/msaa-names.ps1` walks it. One
/// walk per dialog, because a cross-process walk of a large form is slow.
fn texts_by_the_walk(dialog: isize) -> Result<HashMap<isize, String>, String> {
    let root = client_object_of(dialog)?;
    let mut texts = HashMap::new();
    let mut budget = 20_000;
    // SAFETY: a live IAccessible, released once here.
    unsafe {
        walk(root, 0, &mut budget, &mut texts);
        release(root);
    }
    Ok(texts)
}

/// Depth first from `object`, every editable text's window and name into
/// `texts`. Takes no ownership of `object`.
unsafe fn walk(
    object: *mut c_void,
    depth: u32,
    budget: &mut u32,
    texts: &mut HashMap<isize, String>,
) {
    if depth > 40 || *budget == 0 {
        return;
    }
    *budget -= 1;
    // SAFETY: a live IAccessible for every call below; each child is
    // released once, after its own walk.
    unsafe {
        let mut window = 0isize;
        if role_of(object) == ROLE_SYSTEM_TEXT
            && WindowFromAccessibleObject(object, &mut window) >= 0
        {
            texts.entry(window).or_insert_with(|| name_of(object));
        }
        let get_count: GetCountFn =
            std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_CHILD_COUNT));
        let mut count = 0i32;
        if get_count(object, &mut count) < 0 || count <= 0 {
            return;
        }
        let mut children = vec![Variant::empty(); count as usize];
        let mut obtained = 0i32;
        if AccessibleChildren(object, 0, count, children.as_mut_ptr(), &mut obtained) < 0 {
            return;
        }
        for child in children.iter().take(obtained.max(0) as usize) {
            if child.vt != VT_DISPATCH || child.val == 0 {
                continue;
            }
            let dispatch = child.val as *mut c_void;
            let query: QueryInterfaceFn =
                std::mem::transmute(vtable_entry(dispatch, VTBL_QUERY_INTERFACE));
            let mut accessible: *mut c_void = std::ptr::null_mut();
            if query(dispatch, &IID_IACCESSIBLE, &mut accessible) >= 0 && !accessible.is_null() {
                walk(accessible, depth + 1, budget, texts);
                release(accessible);
            }
            release(dispatch);
        }
    }
}

/// What UI Automation answers as the name of the element for `hwnd`.
fn uia_name_of(hwnd: isize) -> String {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
    use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation};
    // SAFETY: COM is joined on this thread by the reader; the handle is a live
    // window in the process that started this one.
    let read = unsafe {
        CoCreateInstance::<_, IUIAutomation>(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
            .and_then(|automation| automation.ElementFromHandle(HWND(hwnd as *mut c_void)))
            .and_then(|element| element.CurrentName())
    };
    read.map_or_else(|why| format!("({why})"), |name| name.to_string())
}

/// The reader's half: read each field it is handed, in its own process, and
/// print what each channel answered, the arrows' name with them. Started only
/// by the window process; run on its own it is handed nothing.
#[test]
#[ignore = "started by the window process, in a process of its own"]
fn the_reader_process_reads_each_field_it_is_handed() {
    let Ok(list) = std::env::var(THE_FIELDS_TO_READ) else {
        return;
    };
    // SAFETY: this thread is the reader's own and nothing else uses COM on it.
    unsafe {
        let _ = windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
        );
    }
    let mut walked: HashMap<isize, Result<HashMap<isize, String>, String>> = HashMap::new();
    for entry in list.split(';').filter(|entry| !entry.is_empty()) {
        let handles: Vec<isize> = entry
            .split(':')
            .filter_map(|handle| handle.parse().ok())
            .collect();
        let [dialog, arrows, field] = handles[..] else {
            continue;
        };
        let by_the_walk = match walked
            .entry(dialog)
            .or_insert_with(|| texts_by_the_walk(dialog))
        {
            Ok(texts) => texts
                .get(&field)
                .cloned()
                .unwrap_or_else(|| "(the walk did not reach the field)".to_string()),
            Err(why) => why.clone(),
        };
        println!(
            "{A_FIELD_HEARD}{field}\t{}\t{}\t{by_the_walk}\t{}",
            msaa_name_of(arrows),
            msaa_name_of(field),
            uia_name_of(field),
        );
    }
}

// ── The window process ─────────────────────────────────────────────────────

/// One field as the reader heard it.
#[derive(Debug, Clone)]
struct Heard {
    window: String,
    arrows: String,
    own: String,
    walk: String,
    uia: String,
}

fn date_settings() -> DateSettings {
    DateSettings {
        style: DateStyle::Absolute,
        order: DateOrder::MonthFirst,
        wording: DateWording::Verbal,
        clock: Clock::TwentyFourHour,
    }
}

/// Dispatch every message waiting for this thread's windows.
fn answer_waiting_messages() {
    let mut message = [0u8; 48];
    // SAFETY: the buffer is at least as large as a MSG on 64-bit Windows.
    unsafe {
        while PeekMessageW(&mut message, 0, 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

/// Start `test` of this binary as a child with `env` set, answering this
/// thread's messages while it runs, and hand back what it printed.
fn run_a_child(
    test: &str,
    env: (&str, String),
    at_most: std::time::Duration,
) -> Result<String, String> {
    let me = std::env::current_exe().map_err(|why| format!("current_exe: {why}"))?;
    let mut child = std::process::Command::new(me)
        .args([
            "--ignored",
            "--exact",
            "--nocapture",
            "--test-threads=1",
            test,
        ])
        .env(env.0, env.1)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|why| format!("{test} could not start: {why}"))?;
    // The child's reads are messages to this thread's windows, so they are
    // answered here while it runs; a parent that only waited would never let
    // it finish. The output is drained on a thread of its own so a full pipe
    // cannot stall the child.
    let mut out = child.stdout.take();
    let drained = std::thread::spawn(move || {
        let mut printed = String::new();
        if let Some(out) = out.as_mut() {
            use std::io::Read;
            let _ = out.read_to_string(&mut printed);
        }
        printed
    });
    let deadline = std::time::Instant::now() + at_most;
    let finished = loop {
        answer_waiting_messages();
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if std::time::Instant::now() > deadline => {
                let _ = child.kill();
                break Err(format!("{test} was still running after {at_most:?}"));
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(5)),
            Err(why) => break Err(format!("waiting for {test}: {why}")),
        }
    }?;
    let printed = drained
        .join()
        .map_err(|_| format!("reading {test}'s output panicked"))?;
    if !finished.success() || !printed.contains("test result: ok. 1 passed") {
        return Err(format!("{test} ran no reading ({finished}):\n{printed}"));
    }
    Ok(printed)
}

/// Build the program's condition, name the fields the way the program does,
/// and have another process read them. `readied` is whether the store is
/// readied before the browser, as the program's startup does.
fn build_name_and_read(readied: bool) -> Result<Vec<String>, String> {
    if readied {
        ready_the_annotation_store();
    }
    let frame = Frame::builder().build();
    if !WebView::is_backend_available(WebViewBackend::Edge) {
        return Err(
            "WebView2 is not available here, so the program's condition cannot be built"
                .to_string(),
        );
    }
    // The main window's message preview, built before any dialog as it is in
    // the program. Kept for the life of the reading.
    let preview = WebView::builder(&frame)
        .with_backend(WebViewBackend::Edge)
        .build();
    let a11y =
        Arc::new(Accessibility::new().map_err(|why| format!("Accessibility::new failed: {why}"))?);

    let chosen = Rc::new(std::cell::Cell::new(None));
    let send_later = wx_send_later::build_the_asking_dialog(
        &frame,
        chrono::Local::now(),
        date_settings(),
        &a11y,
        &chosen,
        None,
    );
    let form = build_item_form_dialog(
        &frame,
        ItemKind::Event,
        &[],
        &[],
        Chrome {
            palette: None,
            a11y: &a11y,
            asking: None,
        },
        date_settings(),
        None,
    )
    .ok_or("the event form was not built")?;
    let bare_dialog = Dialog::builder(&frame, "Bare").build();
    let bare = SpinCtrl::builder(&bare_dialog).with_range(0, 9).build();
    set_accessible_name(&bare, BARE_WORDS);

    let windows = [
        (SEND_LATER.0, send_later.get_handle() as isize),
        (THE_EVENT_FORM.0, form.dialog.get_handle() as isize),
        (BARE.0, bare_dialog.get_handle() as isize),
    ];
    let mut which_window = std::collections::HashMap::new();
    let mut list = String::new();
    for (window, dialog) in windows {
        for spinner in the_spinners_under(dialog) {
            which_window.insert(spinner.field, window);
            list.push_str(&format!(
                "{}:{}:{};",
                spinner.dialog, spinner.arrows, spinner.field
            ));
        }
    }
    let printed = run_a_child(
        THE_READER_PROCESS,
        (THE_FIELDS_TO_READ, list),
        std::time::Duration::from_secs(60),
    );

    send_later.destroy();
    form.dialog.destroy();
    bare_dialog.destroy();
    preview.destroy();

    Ok(printed?
        .lines()
        .filter_map(|line| {
            line.find(A_FIELD_HEARD)
                .map(|at| &line[at + A_FIELD_HEARD.len()..])
        })
        .filter_map(|heard| {
            let (field, rest) = heard.split_once('\t')?;
            let window = which_window.get(&field.parse::<isize>().ok()?)?;
            Some(format!("{A_FIELD_REPORTED}{window}\t{rest}"))
        })
        .collect())
}

/// The window process: one `wxdragon::main`, one condition, every failure a
/// line rather than a panic. Started only by a test below.
#[test]
#[ignore = "started by the tests below, in a process of its own"]
fn the_window_process_builds_names_and_has_its_fields_read() {
    let Ok(condition) = std::env::var(THE_CONDITION) else {
        return;
    };
    let outcome: Arc<Mutex<Option<Reported>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let taken = build_name_and_read(condition == READIED);
            if let Ok(mut slot) = outcome.lock() {
                *slot = Some(taken);
            }
            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };
    let taken = match result {
        Err(why) => Err(format!("wxdragon::main returned {why:?}")),
        Ok(()) => outcome
            .lock()
            .map_err(|_| "the outcome's lock was poisoned".to_string())
            .and_then(|mut slot| {
                slot.take().unwrap_or_else(|| {
                    Err("the window session ended without a reading".to_string())
                })
            }),
    };
    match taken {
        Ok(lines) => lines.iter().for_each(|line| println!("{line}")),
        Err(why) => println!("{A_FAILURE}{}", why.replace('\n', " | ")),
    }
}

// ── The tests ──────────────────────────────────────────────────────────────

/// What one window process reported: its lines, or why it could not.
type Reported = Result<Vec<String>, String>;
/// Every field one condition's reader heard, or why it could not be taken.
type Reading = Result<Vec<Heard>, String>;

/// Every field the reader heard under `condition`, taken once per process.
fn heard_under(condition: &'static str) -> &'static Reading {
    static WITH_THE_STORE_READIED: OnceLock<Reading> = OnceLock::new();
    static WITHOUT: OnceLock<Reading> = OnceLock::new();
    let reading = if condition == READIED {
        &WITH_THE_STORE_READIED
    } else {
        &WITHOUT
    };
    reading.get_or_init(|| take_a_reading(condition))
}

fn take_a_reading(condition: &'static str) -> Reading {
    let printed = run_a_child(
        THE_WINDOW_PROCESS,
        (THE_CONDITION, condition.to_string()),
        std::time::Duration::from_secs(180),
    )?;
    if let Some(failed) = printed.lines().find_map(|line| line.split_once(A_FAILURE)) {
        return Err(failed.1.to_string());
    }
    Ok(printed
        .lines()
        .filter_map(|line| {
            line.find(A_FIELD_REPORTED)
                .map(|at| &line[at + A_FIELD_REPORTED.len()..])
        })
        .filter_map(|reported| {
            let mut parts = reported.split('\t');
            Some(Heard {
                window: parts.next()?.to_string(),
                arrows: parts.next()?.to_string(),
                own: parts.next()?.to_string(),
                walk: parts.next()?.to_string(),
                uia: parts.next()?.to_string(),
            })
        })
        .collect())
}

/// The fields of `window` heard under `condition`, held to the window's count.
fn the_fields_of((window, expected): (&str, usize), condition: &'static str) -> Vec<Heard> {
    let heard = match heard_under(condition) {
        Ok(heard) => heard,
        Err(why) => panic!("the reading under {condition} could not be taken: {why}"),
    };
    let fields: Vec<Heard> = heard
        .iter()
        .filter(|field| field.window == window)
        .cloned()
        .collect();
    assert_eq!(
        fields.len(),
        expected,
        "{window} under {condition}: expected {expected} spin controls, heard {fields:#?}"
    );
    fields
}

/// Every field whose three readings are not the arrows' words.
fn not_named(fields: &[Heard]) -> Vec<&Heard> {
    fields
        .iter()
        .filter(|field| {
            field.arrows.is_empty()
                || field.own != field.arrows
                || field.walk != field.arrows
                || field.uia != field.arrows
        })
        .collect()
}

#[test]
fn test_send_later_names_every_typing_field_where_the_scan_reads_it() {
    let fields = the_fields_of(SEND_LATER, READIED);
    let missed = not_named(&fields);
    assert!(
        missed.is_empty(),
        "Send Later, a browser built first as the program does: these fields do not \
         answer their arrows' words on MSAA, by the scan's walk and on UI Automation: \
         {missed:#?}"
    );
}

#[test]
fn test_the_event_form_names_every_typing_field_where_the_scan_reads_it() {
    let fields = the_fields_of(THE_EVENT_FORM, READIED);
    let missed = not_named(&fields);
    assert!(
        missed.is_empty(),
        "the event form, a browser built first as the program does: these fields do not \
         answer their arrows' words on MSAA, by the scan's walk and on UI Automation: \
         {missed:#?}"
    );
}

/// The companion that shows the reading sees the fault ledger 408 describes:
/// a spin control named on its arrows alone answers the words there and
/// nothing on the field a person types in.
#[test]
fn test_a_spin_control_named_on_its_arrows_alone_reads_no_name_on_its_field() {
    let fields = the_fields_of(BARE, READIED);
    let bare = &fields[0];
    assert_eq!(bare.arrows, BARE_WORDS, "the arrows: {bare:#?}");
    assert!(
        bare.own.is_empty() && bare.walk.is_empty() && bare.uia != BARE_WORDS,
        "a field nobody named was heard with a name: {bare:#?}"
    );
}

/// The program's condition before 12-06.1, reproduced: with the store not
/// readied, a browser built first leaves every field in Send Later nameless
/// on both channels, as the scan of the running program found. This is the
/// measurement the fix rests on, and it stays true after the fix, because
/// nothing here readies the store.
#[test]
fn test_a_browser_built_before_the_store_is_readied_leaves_every_field_nameless() {
    let fields = the_fields_of(SEND_LATER, NOT_READIED);
    let named: Vec<&Heard> = fields
        .iter()
        .filter(|field| {
            field.arrows.is_empty()
                || !field.own.is_empty()
                || !field.walk.is_empty()
                || field.uia == field.arrows
        })
        .collect();
    assert!(
        named.is_empty(),
        "with a browser built before any annotation, these fields were heard named, \
         so this reading no longer shows the program's condition: {named:#?}"
    );
}

//! Every check box on Settings, Questions is named with its words where the
//! keyboard lands, read from a process of its own on both channels
//! (13.4-12.4).
//!
//! The page holds one box for each question a person can stop, "Ask before
//! ..." and ticked while the question is asked. The words expected are taken
//! from `asking::in_settings` over `Question::ALL`, with the access letter's
//! ampersand gone, rather than written out here, so a question taken off the
//! page there is one fewer box here with no edit.
//!
//! The shape is `tests/an_editable_combo_box_is_named_where_typing_lands.rs`'s:
//! the window process builds the program's condition, the annotation store
//! readied and then a WebView2 preview, builds Settings hidden, reaches the
//! eighth tab the way the tab row does, gives each box the keyboard and takes
//! the window Windows says has it, and a reader process it starts reads each
//! handle by `AccessibleObjectFromWindow`, which is MSAA, NVDA's channel for a
//! `Button`-class window, and by `ElementFromHandle`, which is UI Automation,
//! Narrator's. Nothing is shown, so nothing here moves the foreground or
//! takes a key.
//!
//! Each box's role, state and whether a click moves it are read by
//! `tests/every_settings_checkbox_reads_as_a_checkbox_after_its_page_is_built.rs`,
//! whose harvest reaches this page too.

#![cfg(windows)]

use std::ffi::c_void;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::questions_before_acting::Question;
use wixen_mail::data::config::AppConfig;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::accessibility::names::ready_the_annotation_store;
use wixen_mail::presentation::asking;
use wixen_mail::presentation::wx_settings;
use wxdragon::prelude::*;
use wxdragon::widgets::{WebView, WebViewBackend};

/// Where Questions sits in the tab row: after General and the six after it.
const THE_QUESTIONS_TAB: usize = 7;

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const VT_I4: u16 = 3;
const CHILDID_SELF: i64 = 0;
const GWL_STYLE: i32 = -16;
/// The low nibble of a `Button`-class window's style, and the kinds that are
/// not check boxes (winuser.h).
const BS_TYPEMASK: isize = 0xF;
const BS_PUSHBUTTON: isize = 0x0;
const BS_DEFPUSHBUTTON: isize = 0x1;
const BS_GROUPBOX: isize = 0x7;
/// `PeekMessageW` takes the message off the queue (winuser.h).
const PM_REMOVE: u32 = 1;

const THE_WINDOW_PROCESS: &str = "the_window_process_builds_the_questions_page_and_has_it_read";
const THE_READER_PROCESS: &str = "the_reader_process_reads_each_box_it_is_handed";
const RUN_THE_WINDOWS: &str = "WIXEN_QUESTIONS_PAGE_WINDOWS";
const THE_HANDLES_TO_READ: &str = "WIXEN_QUESTIONS_PAGE_HANDLES";
/// Line markers, separated by tabs.
const A_HANDLE_HEARD: &str = "HEARD\t";
const A_FAILURE: &str = "FAILED\t";
const THE_READER_PASSED: &str = "test result: ok. 1 passed";
const THE_READING_PRINTED: &str = "PRINTED\tthe whole reading";

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

type Hresult = i32;
type ReleaseFn = unsafe extern "system" fn(*mut c_void) -> u32;
type GetBstrFn = unsafe extern "system" fn(*mut c_void, Variant, *mut *mut u16) -> Hresult;

// IAccessible's vtable: IUnknown (3), IDispatch (4), then get_accParent (7),
// get_accChildCount (8), get_accChild (9), get_accName (10).
const VTBL_RELEASE: usize = 2;
const VTBL_GET_ACC_NAME: usize = 10;

#[link(name = "user32")]
unsafe extern "system" {
    fn EnumChildWindows(
        parent: isize,
        callback: extern "system" fn(isize, isize) -> i32,
        lparam: isize,
    ) -> i32;
    fn GetClassNameW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
    fn GetWindowLongPtrW(hwnd: isize, index: i32) -> isize;
    fn GetFocus() -> isize;
    fn SetFocus(hwnd: isize) -> isize;
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

// ── MSAA and UI Automation, in the reading process ─────────────────────────

unsafe fn vtable_entry(object: *mut c_void, index: usize) -> *const c_void {
    // SAFETY: a COM object is a pointer to its vtable.
    unsafe {
        let vtable = *(object as *const *const *const c_void);
        *vtable.add(index)
    }
}

/// The MSAA name of `hwnd`'s own client object, as a reader asks for it.
fn msaa_name_of(hwnd: isize) -> String {
    let mut object: *mut c_void = std::ptr::null_mut();
    // SAFETY: the out pointer is a local; on success it holds an IAccessible,
    // released once below.
    let hr =
        unsafe { AccessibleObjectFromWindow(hwnd, OBJID_CLIENT, &IID_IACCESSIBLE, &mut object) };
    if hr < 0 || object.is_null() {
        return format!("(AccessibleObjectFromWindow failed 0x{hr:x})");
    }
    let child_self = Variant {
        vt: VT_I4,
        reserved1: 0,
        reserved2: 0,
        reserved3: 0,
        val: CHILDID_SELF,
        extra: 0,
    };
    // SAFETY: a live IAccessible; the slots are get_accName and Release, and
    // the BSTR it hands out carries its length and is freed once here.
    unsafe {
        let get_name: GetBstrFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_NAME));
        let mut s: *mut u16 = std::ptr::null_mut();
        let name = match get_name(object, child_self, &mut s) >= 0 && !s.is_null() {
            true => {
                let text = String::from_utf16_lossy(std::slice::from_raw_parts(
                    s,
                    SysStringLen(s) as usize,
                ));
                SysFreeString(s);
                text
            }
            false => String::new(),
        };
        let release: ReleaseFn = std::mem::transmute(vtable_entry(object, VTBL_RELEASE));
        release(object);
        name
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

/// The reader's half: read each handle it is handed, in its own process, and
/// print what each channel answered. Started only by the window process; run
/// on its own it is handed nothing.
#[test]
#[ignore = "started by the window process, in a process of its own"]
fn the_reader_process_reads_each_box_it_is_handed() {
    let Ok(list) = std::env::var(THE_HANDLES_TO_READ) else {
        return;
    };
    // SAFETY: this thread is the reader's own and nothing else uses COM on it.
    unsafe {
        let _ = windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
        );
    }
    for handle in list
        .split(';')
        .filter_map(|handle| handle.parse::<isize>().ok())
    {
        println!(
            "{A_HANDLE_HEARD}{}\t{}",
            msaa_name_of(handle),
            uia_name_of(handle),
        );
    }
}

// ── The window process ─────────────────────────────────────────────────────

thread_local! {
    static FOUND: std::cell::RefCell<Vec<isize>> = const { std::cell::RefCell::new(Vec::new()) };
}

extern "system" fn collect(hwnd: isize, _lparam: isize) -> i32 {
    FOUND.with(|found| found.borrow_mut().push(hwnd));
    1
}

/// The check boxes under `page`, in the order Windows lists them, which is
/// the order they were made.
fn the_check_boxes_under(page: isize) -> Vec<isize> {
    FOUND.with(|found| found.borrow_mut().clear());
    // SAFETY: the callback only pushes to this thread's local.
    unsafe { EnumChildWindows(page, collect, 0) };
    FOUND
        .with(|found| found.borrow().clone())
        .into_iter()
        .filter(|hwnd| class_name(*hwnd) == "Button")
        .filter(|hwnd| {
            // SAFETY: a live window handle; only its style is read.
            let kind = unsafe { GetWindowLongPtrW(*hwnd, GWL_STYLE) } & BS_TYPEMASK;
            !matches!(kind, BS_PUSHBUTTON | BS_DEFPUSHBUTTON | BS_GROUPBOX)
        })
        .collect()
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
/// thread's messages while it runs, since the child's reads are messages to
/// this thread's windows, and hand back what it printed once it has printed
/// `done`, the line that says it finished.
fn run_a_child(
    test: &str,
    env: (&str, String),
    at_most: std::time::Duration,
    done: &str,
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
    let mut out = child.stdout.take();
    let so_far = Arc::new(Mutex::new(String::new()));
    let printed_so_far = || {
        so_far
            .lock()
            .map(|printed| printed.clone())
            .unwrap_or_default()
    };
    {
        let so_far = so_far.clone();
        std::thread::spawn(move || {
            if let Some(out) = out.as_mut() {
                use std::io::BufRead;
                for line in std::io::BufReader::new(out).lines().map_while(Result::ok) {
                    if let Ok(mut printed) = so_far.lock() {
                        printed.push_str(&line);
                        printed.push('\n');
                    }
                }
            }
        });
    }
    let deadline = std::time::Instant::now() + at_most;
    let finished = loop {
        answer_waiting_messages();
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if std::time::Instant::now() > deadline => {
                let _ = child.kill();
                break Err(format!(
                    "{test} was still running after {at_most:?}, having printed:\n{}",
                    printed_so_far()
                ));
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(5)),
            Err(why) => break Err(format!("waiting for {test}: {why}")),
        }
    }?;
    let read_until = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !printed_so_far().contains(done) && std::time::Instant::now() < read_until {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let printed = printed_so_far();
    if !finished.success() || !printed.contains(done) {
        return Err(format!("{test} ran no reading ({finished}):\n{printed}"));
    }
    Ok(printed)
}

/// Build the program's condition and Settings, reach the Questions tab, give
/// each box the keyboard, and have another process read where it landed.
fn build_and_read() -> Result<Vec<String>, String> {
    ready_the_annotation_store();
    let frame = Frame::builder().build();
    if !WebView::is_backend_available(WebViewBackend::Edge) {
        return Err(
            "WebView2 is not available here, so the program's condition cannot be built"
                .to_string(),
        );
    }
    let preview = WebView::builder(&frame)
        .with_backend(WebViewBackend::Edge)
        .build();
    let a11y =
        Arc::new(Accessibility::new().map_err(|why| format!("Accessibility::new failed: {why}"))?);
    let widgets =
        wx_settings::build_settings_dialog(&frame, &AppConfig::default(), &[], false, &a11y);
    // Counted first, because wxWidgets asserts on a page asked for past the
    // end.
    let page = (widgets.notebook.get_page_count() > THE_QUESTIONS_TAB)
        .then(|| widgets.notebook.get_page(THE_QUESTIONS_TAB))
        .flatten()
        .ok_or("Settings has no Questions page: its tab row has no eighth tab")?;
    // The way the tab row reaches a page, which is what builds it.
    widgets.notebook.set_selection(THE_QUESTIONS_TAB);

    let mut landed_on = Vec::new();
    for check_box in the_check_boxes_under(page.get_handle() as isize) {
        // SAFETY: plain reads and a focus change on this thread's windows.
        let landed = unsafe {
            SetFocus(check_box);
            GetFocus()
        };
        if landed != check_box {
            return Err(format!(
                "a box given the keyboard, Windows says {landed} ({}) has it, not the box {check_box}",
                class_name(landed)
            ));
        }
        landed_on.push(landed.to_string());
    }
    let printed = run_a_child(
        THE_READER_PROCESS,
        (THE_HANDLES_TO_READ, landed_on.join(";")),
        std::time::Duration::from_secs(60),
        THE_READER_PASSED,
    );

    widgets.dialog.destroy();
    preview.destroy();

    Ok(printed?
        .lines()
        .filter_map(|line| line.find(A_HANDLE_HEARD).map(|at| line[at..].to_string()))
        .collect())
}

/// The window process: one `wxdragon::main`, every failure a line rather than
/// a panic. Started only by a test below. It ends inside `on_init`, the
/// reading printed, as the combo box target's does (ledger 966).
#[test]
#[ignore = "started by the tests below, in a process of its own"]
fn the_window_process_builds_the_questions_page_and_has_it_read() {
    if std::env::var(RUN_THE_WINDOWS).is_err() {
        return;
    }
    let result = wxdragon::main(move |_| {
        match build_and_read() {
            Ok(lines) => lines.iter().for_each(|line| println!("{line}")),
            Err(why) => println!("{A_FAILURE}{}", why.replace('\n', " | ")),
        }
        println!("{THE_READING_PRINTED}");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        std::process::exit(0);
    });
    println!("{A_FAILURE}wxdragon::main returned {result:?} without running the reading");
    println!("{THE_READING_PRINTED}");
}

// ── The test ───────────────────────────────────────────────────────────────

/// Each box as the reader heard it, in the page's order: its MSAA name and
/// its UI Automation name.
fn the_reading() -> &'static Result<Vec<(String, String)>, String> {
    static READING: OnceLock<Result<Vec<(String, String)>, String>> = OnceLock::new();
    READING.get_or_init(|| {
        let printed = run_a_child(
            THE_WINDOW_PROCESS,
            (RUN_THE_WINDOWS, "yes".to_string()),
            std::time::Duration::from_secs(180),
            THE_READING_PRINTED,
        )?;
        if let Some((_, why)) = printed.lines().find_map(|line| line.split_once(A_FAILURE)) {
            return Err(why.to_string());
        }
        Ok(printed
            .lines()
            .filter_map(|line| line.split_once(A_HANDLE_HEARD))
            .filter_map(|(_, heard)| heard.split_once('\t'))
            .map(|(msaa, uia)| (msaa.to_string(), uia.to_string()))
            .collect())
    })
}

#[test]
fn test_every_box_on_the_questions_page_is_named_where_the_keyboard_lands_on_both_channels() {
    let expected: Vec<(String, String)> = Question::ALL
        .into_iter()
        .filter_map(asking::in_settings)
        .map(|label| {
            let words = label.replace('&', "");
            (words.clone(), words)
        })
        .collect();
    assert!(
        !expected.is_empty(),
        "asking::in_settings offers no question on the page, so this reads nothing"
    );
    let heard = match the_reading() {
        Ok(heard) => heard,
        Err(why) => panic!("the reading could not be taken: {why}"),
    };
    println!("{heard:#?}");
    assert_eq!(
        heard, &expected,
        "each box on Settings, Questions, as MSAA and UI Automation name it where the keyboard \
         lands, in the page's order"
    );
}

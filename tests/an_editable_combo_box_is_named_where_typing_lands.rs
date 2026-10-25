//! An editable combo box is named at the window the keyboard lands in, read
//! from a process of its own on both channels (13.4-12).
//!
//! An editable combo box is two windows, like a spin control: the box, whose
//! handle wxWidgets hands back and where `set_accessible_name` attaches its
//! object, and an `Edit` inside it, where the keyboard goes. The spin
//! controls were named on the outer handle alone until 2026-09-14 and the
//! field a person typed in was nameless (ledger 408). Three boxes have this
//! shape: the item form's Category, named with `set_accessible_name` on the
//! box, and the contact editor's Prefix and Suffix, named by the label built
//! before each and deliberately not by an object of ours (pull request #99).
//!
//! The handle read is the one Windows says has the keyboard after the box is
//! given it, so nothing here assumes which child that is. The reading is
//! taken the way a screen reader takes it: the window process builds the
//! dialogs and names them, and a reader process it starts reads each handle
//! by `AccessibleObjectFromWindow`, which is MSAA, NVDA's channel for these
//! controls, and by `ElementFromHandle`, which is UI Automation, Narrator's.
//!
//! The window process builds the program's condition first, the annotation
//! store readied and then a WebView2 preview, because a name written through
//! the annotation service after a browser is kept only in that order
//! (`tests/a_spin_controls_field_is_named_where_the_scan_reads_it.rs`). The
//! dialogs are built hidden and never shown, so nothing here moves the
//! foreground.
//!
//! Two companions read boxes with no label beside them. One named on its own
//! handle alone is heard with that name where typing lands, which is why the
//! Category box's typing field is named although nothing writes to it: unlike
//! a spin control's buddy, the `Edit` inside a combo box answers with the
//! box's name. One named by nothing is heard with no name, so the reading can
//! see a nameless typing field when there is one.
//!
//! The Prefix and Suffix boxes are heard with their labels as Windows reads a
//! label it names a control by: the mnemonic gone and the colon kept,
//! "Prefix:". `tests/the_contact_editor_fills_the_name_and_its_parts_from_each_other.rs`
//! reads the same two fields in its own process and compares them with the
//! colon trimmed; this one compares the words as heard.

#![cfg(windows)]

use std::ffi::c_void;
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
use wixen_mail::presentation::wx_managers;
use wxdragon::prelude::*;
use wxdragon::widgets::{WebView, WebViewBackend};

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const VT_I4: u16 = 3;
const CHILDID_SELF: i64 = 0;
/// `PeekMessageW` takes the message off the queue (winuser.h).
const PM_REMOVE: u32 = 1;

/// The three boxes, by the name this file gives each, and the words a screen
/// reader should hear on its typing field: the Category field's name as
/// `name_it` gives it, `name_from_label("Cate&gory")`, and the Prefix and
/// Suffix labels as Windows reads a label it names a control by, the
/// mnemonic gone and the colon kept.
const CATEGORY: (&str, &str) = ("the item form's Category box", "Category");
const PREFIX: (&str, &str) = ("the contact editor's Prefix box", "Prefix:");
const SUFFIX: (&str, &str) = ("the contact editor's Suffix box", "Suffix:");
/// A box with no label beside it, named on its own handle and nowhere else.
const NAMED_ON_THE_BOX: (&str, &str) = (
    "a combo box named on its own handle alone",
    "Named on the box alone",
);
/// A box with no label beside it and no name, whose typing field has nothing
/// to be named by.
const UNNAMED: (&str, &str) = ("a combo box nobody named", "");

const THE_WINDOW_PROCESS: &str = "the_window_process_builds_the_boxes_and_has_them_read";
const THE_READER_PROCESS: &str = "the_reader_process_reads_each_handle_it_is_handed";
const RUN_THE_WINDOWS: &str = "WIXEN_COMBO_READING_WINDOWS";
const THE_HANDLES_TO_READ: &str = "WIXEN_COMBO_READING_HANDLES";
/// Line markers, separated by tabs.
const A_HANDLE_HEARD: &str = "HEARD\t";
const A_BOX_REPORTED: &str = "BOX\t";
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
    fn GetClassNameW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
    fn GetFocus() -> isize;
    fn IsChild(parent: isize, hwnd: isize) -> i32;
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
/// print its class and what each channel answered. Started only by the window
/// process; run on its own it is handed nothing.
#[test]
#[ignore = "started by the window process, in a process of its own"]
fn the_reader_process_reads_each_handle_it_is_handed() {
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
            "{A_HANDLE_HEARD}{handle}\t{}\t{}\t{}",
            class_name(handle),
            msaa_name_of(handle),
            uia_name_of(handle),
        );
    }
}

// ── The window process ─────────────────────────────────────────────────────

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

/// Give `combo` the keyboard and answer the window Windows says has it, which
/// must be the box or a window inside it.
fn where_the_keyboard_lands(name: &str, combo: &ComboBox) -> Result<isize, String> {
    combo.set_focus();
    let outer = combo.get_handle() as isize;
    // SAFETY: plain reads of window state on this thread.
    let landed = unsafe { GetFocus() };
    // SAFETY: as above.
    match landed == outer || unsafe { IsChild(outer, landed) } != 0 {
        true => Ok(landed),
        false => Err(format!(
            "{name}: given the keyboard, Windows says {landed} ({}) has it, not the box {outer} or a window inside it",
            class_name(landed)
        )),
    }
}

fn date_settings() -> DateSettings {
    DateSettings {
        style: DateStyle::Absolute,
        order: DateOrder::MonthFirst,
        wording: DateWording::Verbal,
        clock: Clock::TwentyFourHour,
    }
}

/// Build the program's condition, the three boxes and the bare one, find
/// where the keyboard lands in each, and have another process read them.
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
    let category = form
        .category_field
        .ok_or("the event form has no Category box")?;
    let contact = wx_managers::build_contact_edit_dialog(&frame, None, None, &a11y);
    let bare_dialog = Dialog::builder(&frame, "Bare").build();
    let named_on_the_box = ComboBox::builder(&bare_dialog).build();
    set_accessible_name(&named_on_the_box, NAMED_ON_THE_BOX.1);
    let unnamed = ComboBox::builder(&bare_dialog).build();

    let boxes = [
        (CATEGORY.0, category),
        (PREFIX.0, contact.prefix_f),
        (SUFFIX.0, contact.suffix_f),
        (NAMED_ON_THE_BOX.0, named_on_the_box),
        (UNNAMED.0, unnamed),
    ];
    let mut which_box = std::collections::HashMap::new();
    let mut list = String::new();
    for (name, combo) in &boxes {
        let landed = where_the_keyboard_lands(name, combo)?;
        which_box.insert(landed, *name);
        list.push_str(&format!("{landed};"));
    }
    let printed = run_a_child(
        THE_READER_PROCESS,
        (THE_HANDLES_TO_READ, list),
        std::time::Duration::from_secs(60),
        THE_READER_PASSED,
    );

    form.dialog.destroy();
    contact.dialog.destroy();
    bare_dialog.destroy();
    preview.destroy();

    Ok(printed?
        .lines()
        .filter_map(|line| {
            line.find(A_HANDLE_HEARD)
                .map(|at| &line[at + A_HANDLE_HEARD.len()..])
        })
        .filter_map(|heard| {
            let (handle, rest) = heard.split_once('\t')?;
            let name = which_box.get(&handle.parse::<isize>().ok()?)?;
            Some(format!("{A_BOX_REPORTED}{name}\t{rest}"))
        })
        .collect())
}

/// The window process: one `wxdragon::main`, every failure a line rather than
/// a panic. Started only by a test below. It ends inside `on_init`, the
/// reading printed, because a window process at wxDragon 0.9.23 on GitHub's
/// runner has come back from `on_init` and never reached idle (ledger 966).
#[test]
#[ignore = "started by the tests below, in a process of its own"]
fn the_window_process_builds_the_boxes_and_has_them_read() {
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

// ── The tests ──────────────────────────────────────────────────────────────

/// One box's typing field as the reader heard it.
#[derive(Debug, Clone)]
struct Heard {
    which: String,
    class: String,
    msaa: String,
    uia: String,
}

type Reading = Result<Vec<Heard>, String>;

fn the_reading() -> &'static Reading {
    static READING: OnceLock<Reading> = OnceLock::new();
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
            .filter_map(|line| {
                line.find(A_BOX_REPORTED)
                    .map(|at| &line[at + A_BOX_REPORTED.len()..])
            })
            .filter_map(|reported| {
                let mut parts = reported.split('\t');
                Some(Heard {
                    which: parts.next()?.to_string(),
                    class: parts.next()?.to_string(),
                    msaa: parts.next()?.to_string(),
                    uia: parts.next()?.to_string(),
                })
            })
            .collect())
    })
}

fn heard((which, _): (&str, &str)) -> &'static Heard {
    let reading = match the_reading() {
        Ok(reading) => reading,
        Err(why) => panic!("the reading could not be taken: {why}"),
    };
    reading
        .iter()
        .find(|heard| heard.which == which)
        .unwrap_or_else(|| panic!("{which} was not read: {reading:#?}"))
}

/// Where the keyboard lands is the box's typing field, and both channels
/// answer it with `words`.
fn named_where_typing_lands(heard: &Heard, words: &str) -> Result<(), String> {
    match heard.class == "Edit" && heard.msaa == words && heard.uia == words {
        true => Ok(()),
        false => Err(format!(
            "{}: the keyboard lands in a {:?}, which reads {:?} on MSAA and {:?} on UI \
             Automation, not {words:?} on both",
            heard.which, heard.class, heard.msaa, heard.uia
        )),
    }
}

#[test]
fn test_every_editable_combo_box_is_named_where_typing_lands_on_both_channels() {
    let wrong: Vec<String> = [CATEGORY, PREFIX, SUFFIX]
        .into_iter()
        .filter_map(|(which, words)| named_where_typing_lands(heard((which, words)), words).err())
        .collect();

    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Why the Category box is heard where typing lands while a spin control
/// named on its arrows was not: Windows answers for an editable combo box's
/// `Edit` with the box's own name, on both channels. Read with no label
/// beside the box, so the name can have come from nowhere else. Measured
/// first on 2026-10-25, at wxDragon 0.9.23.
#[test]
fn test_a_combo_box_named_on_its_own_handle_is_heard_where_typing_lands() {
    let named = heard(NAMED_ON_THE_BOX);

    assert_eq!(
        named_where_typing_lands(named, NAMED_ON_THE_BOX.1),
        Ok(()),
        "{named:#?}"
    );
}

/// The companion that shows the reading can see a nameless typing field: a
/// box with no label beside it and no name of its own answers nothing on
/// either channel where the keyboard lands.
#[test]
fn test_a_combo_box_nobody_named_reads_no_name_where_typing_lands() {
    let unnamed = heard(UNNAMED);

    assert_eq!(
        named_where_typing_lands(unnamed, UNNAMED.1),
        Ok(()),
        "{unnamed:#?}"
    );
}

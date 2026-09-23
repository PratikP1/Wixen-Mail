//! Send Feedback shows what it sends before it goes, a security concern goes
//! to an address of its own with its log left out, and a report takes the
//! one path every message takes (#64, #71, #78, ALPHA-02, 12-05).
//!
//! One window session builds the real dialog with `build_feedback_dialog`
//! three times and reads it the way NVDA reads native controls: the children
//! in the order Windows holds them, which is the order Tab walks, and each
//! focusable one over MSAA through `AccessibleObjectFromWindow`, its name,
//! role and state. Nothing calls `show_modal`, nothing is clicked, no window
//! is shown, nothing is sent and no page is opened.
//!
//! What this proves and what it does not. The fields are filled with
//! `set_value`, which raises the text event the dialog's refresh is bound to,
//! so the payload reading also proves that link. The category is changed by
//! calling `choose`, the method the choice's handler calls; that a native
//! selection reaches the handler is not proved here. How the window sounds
//! is on the ledger for the tester.
//!
//! Two readings are of `src/presentation/wx_app.rs` as text, because both
//! facts sit inside the main window's own start-up, with its accounts,
//! database and runtime, which this file does not start: that one function
//! builds the queued row and both the composer's send and the report's send
//! call it, and that the Help menu offers the item on Ctrl+Shift+F. Each has
//! a companion that plants the fault and is refused.
//!
//! The declarations are the ones `tests/the_about_dialog_names_its_owners_and_its_links.rs`
//! uses, for the same reason: a feature on the `windows` crate is compiled
//! into the shipping binary.

#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::feedback_report::{
    Category, Fact, Facts, ISSUE_PAGE, Include, LogFile, PRIVATE_REPORTING_PAGE, Report,
    SECURITY_ADDRESS, SUPPORT_ADDRESS, compose,
};
use wixen_mail::data::account::Account;
use wixen_mail::presentation::theme;
use wixen_mail::presentation::wx_feedback::{
    Opening, SendDoor, Sender, build_feedback_dialog, from_line, payload_text, what_the_doors_do,
};
use wxdragon::prelude::*;

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const GWL_STYLE: i32 = -16;
const WS_VISIBLE: isize = 0x1000_0000;
const VT_I4: u16 = 3;
const CHILDID_SELF: i64 = 0;

/// MSAA roles and states (oleacc.h).
const ROLE_SYSTEM_PUSHBUTTON: i64 = 0x2b;
const ROLE_SYSTEM_CHECKBUTTON: i64 = 0x2c;
const ROLE_SYSTEM_TEXT: i64 = 0x2a;
const ROLE_SYSTEM_COMBOBOX: i64 = 0x2e;
/// The resize grip a resizable dialog carries in its corner, which no key
/// reaches and which the first run of this reading found at the end.
const ROLE_SYSTEM_GRIP: i64 = 0x4;
const STATE_SYSTEM_UNAVAILABLE: i64 = 0x1;
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

type Hresult = i32;
type ReleaseFn = unsafe extern "system" fn(*mut c_void) -> u32;
type GetVariantFn = unsafe extern "system" fn(*mut c_void, Variant, *mut Variant) -> Hresult;
type GetBstrFn = unsafe extern "system" fn(*mut c_void, Variant, *mut *mut u16) -> Hresult;

// IAccessible's vtable: IUnknown (3), IDispatch (4), then get_accParent (7),
// get_accChildCount (8), get_accChild (9), get_accName (10), get_accValue
// (11), get_accDescription (12), get_accRole (13), get_accState (14).
const VTBL_RELEASE: usize = 2;
const VTBL_GET_ACC_NAME: usize = 10;
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

fn class_name(hwnd: isize) -> String {
    let mut buffer = [0u16; 256];
    // SAFETY: the buffer is as long as the count says.
    let len = unsafe { GetClassNameW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

fn window_text(hwnd: isize) -> String {
    let mut buffer = [0u16; 4096];
    // SAFETY: the buffer is as long as the count says.
    let len = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize]).replace("\r\n", "\n")
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

/// What a window's own object answers over MSAA.
#[derive(Debug, Clone, PartialEq)]
struct Msaa {
    name: String,
    role: i64,
    state: i64,
}

fn msaa_of(hwnd: isize) -> Result<Msaa, String> {
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
        let get_role: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_ROLE));
        let get_state: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_STATE));
        let mut name: *mut u16 = std::ptr::null_mut();
        let hr_name = get_name(object, Variant::child(CHILDID_SELF), &mut name);
        let mut role = Variant::empty();
        let hr_role = get_role(object, Variant::child(CHILDID_SELF), &mut role);
        let mut state = Variant::empty();
        let hr_state = get_state(object, Variant::child(CHILDID_SELF), &mut state);
        let release: ReleaseFn = std::mem::transmute(vtable_entry(object, VTBL_RELEASE));
        release(object);
        let read = |hr: Hresult, v: Variant| match hr >= 0 && v.vt == VT_I4 {
            true => v.val & 0xFFFF_FFFF,
            false => -1,
        };
        Ok(Msaa {
            name: if hr_name >= 0 {
                take_bstr(name)
            } else {
                String::new()
            },
            role: read(hr_role, role),
            state: read(hr_state, state),
        })
    }
}

/// One child of the dialog, as Windows holds it.
#[derive(Debug, Clone, PartialEq)]
struct Control {
    class: String,
    text: String,
    visible: bool,
    msaa: Msaa,
}

fn read_the_controls(dialog: &Dialog) -> Result<Vec<Control>, String> {
    let mut controls = Vec::new();
    for hwnd in descendants_of(dialog.get_handle() as isize) {
        controls.push(Control {
            class: class_name(hwnd),
            text: window_text(hwnd),
            // SAFETY: a live window handle.
            // The control's own style bit, not `IsWindowVisible`, which also
            // asks every parent and so answers false for every control of a
            // dialog that is never shown.
            // SAFETY: a live window handle; the style word is only read.
            visible: unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) } & WS_VISIBLE != 0,
            msaa: msaa_of(hwnd)?,
        });
    }
    Ok(controls)
}

/// The facts the dialog opens on, made up so nothing reads this machine.
fn facts() -> Facts {
    Facts {
        version: "1.0.0-alpha.1+9.gabcdef12".to_string(),
        windows_build: "Windows 11, build 26200".to_string(),
        display_language: "en-GB".to_string(),
        screen_reader: Some(("NVDA".to_string(), "2026.3.0.1".to_string())),
        providers: vec!["Gmail".to_string()],
    }
}

fn log() -> LogFile {
    LogFile {
        file_name: "wixen-mail.2026-09-23.log".to_string(),
        date: "2026-09-23".to_string(),
        text: "INFO sync started\nWARN could not reach dana@example.org\nINFO done\n".to_string(),
    }
}

fn sender(allowed: bool) -> Sender {
    Sender {
        account: Account {
            id: "feedback-reading".to_string(),
            ..Account::new("Mail".to_string(), "dana@example.org".to_string())
        },
        is_default: true,
        allowed,
    }
}

const STAMP: &str = "20260923-184512";

/// The two answers the reading types into a report of a problem.
const TYPED: [&str; 2] = [
    "The message list went quiet after I pressed Delete",
    "The next message to be read",
];

#[derive(Debug)]
struct Harvest {
    on_open: Vec<Control>,
    payload_as_filled: String,
    payload_expected: String,
    security_controls: Vec<Control>,
    security_payload: String,
    security_github_page: &'static str,
    no_account_controls: Vec<Control>,
    no_account_why: String,
}

fn opening(sender: Option<Sender>) -> Opening {
    Opening {
        facts: facts(),
        log: Some(log()),
        sender,
        stamp: STAMP.to_string(),
    }
}

fn take_the_harvest() -> Result<Harvest, String> {
    if std::mem::size_of::<Variant>() != 24 {
        return Err("VARIANT is not 24 bytes here, so the reader's layout is wrong".to_string());
    }
    let palette = theme::current("dark");
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let taken: Result<Harvest, String> = (|| {
                let frame = Frame::builder().build();

                let problem = build_feedback_dialog(&frame, opening(Some(sender(true))), palette);
                let on_open = read_the_controls(&problem.dialog)?;
                for ((_, field), typed) in problem.questions.iter().zip(TYPED) {
                    field.set_value(typed);
                }
                let payload_as_filled = problem.payload.get_value().replace("\r\n", "\n");
                let expected_report = Report {
                    category: Category::Problem,
                    answers: TYPED.iter().map(|typed| typed.to_string()).collect(),
                    include: Include::for_category(Category::Problem),
                    reply_to: "dana@example.org".to_string(),
                    log: Some(log()),
                    stamp: STAMP.to_string(),
                };
                let payload_expected = payload_text(
                    &compose(&expected_report, &facts()),
                    &from_line(Some(&sender(true))),
                );

                problem.choose(Category::Security);
                let security_controls = read_the_controls(&problem.dialog)?;
                let security_payload = problem.payload.get_value().replace("\r\n", "\n");
                let security_github_page = problem.doors().github_page;
                problem.dialog.destroy();

                let nobody = build_feedback_dialog(&frame, opening(None), palette);
                let no_account_controls = read_the_controls(&nobody.dialog)?;
                let no_account_why = nobody.why_not.get_label();
                nobody.dialog.destroy();

                Ok(Harvest {
                    on_open,
                    payload_as_filled,
                    payload_expected,
                    security_controls,
                    security_payload,
                    security_github_page,
                    no_account_controls,
                    no_account_why,
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

/// The controls a keyboard reaches, in the order it reaches them.
fn focusable(controls: &[Control]) -> Vec<&Control> {
    controls
        .iter()
        .filter(|control| control.visible && control.msaa.role != ROLE_SYSTEM_GRIP)
        .filter(|control| control.class != "Static")
        .collect()
}

/// What a report of a problem's controls should be, in Tab order: the
/// category, the two questions, the five boxes, the reply field, the payload,
/// then the four buttons.
fn wanted_on_open() -> Vec<(String, i64)> {
    let mut wanted = vec![("What is this about".to_string(), ROLE_SYSTEM_COMBOBOX)];
    for question in [
        "What were you doing, and what did you hear or see?",
        "What did you expect instead?",
    ] {
        wanted.push((question.to_string(), ROLE_SYSTEM_TEXT));
    }
    for fact in Fact::ALL {
        wanted.push((fact.sends().to_string(), ROLE_SYSTEM_CHECKBUTTON));
    }
    wanted.push(("How to reach you".to_string(), ROLE_SYSTEM_TEXT));
    wanted.push(("What will be sent".to_string(), ROLE_SYSTEM_TEXT));
    for button in [
        "Send",
        "Copy to clipboard",
        "Open the GitHub issue page",
        "Cancel",
    ] {
        wanted.push((button.to_string(), ROLE_SYSTEM_PUSHBUTTON));
    }
    wanted
}

fn what_is_wrong_with_the_order(controls: &[Control]) -> Vec<String> {
    let found: Vec<(String, i64)> = focusable(controls)
        .iter()
        .map(|control| (control.msaa.name.clone(), control.msaa.role))
        .collect();
    let wanted = wanted_on_open();
    let mut wrong = Vec::new();
    if found.len() != wanted.len() {
        wrong.push(format!(
            "{} controls where {} are wanted: {found:?}",
            found.len(),
            wanted.len()
        ));
    }
    for (at, want) in wanted.iter().enumerate() {
        match found.get(at) {
            Some(got) if got == want => {}
            got => wrong.push(format!("control {at} is {got:?} where {want:?} is wanted")),
        }
    }
    wrong
}

/// The box for one fact, by its name over MSAA.
fn box_for(controls: &[Control], fact: Fact) -> Option<&Control> {
    controls
        .iter()
        .find(|control| control.msaa.name == fact.sends())
}

fn what_is_wrong_with_the_default_ticks(controls: &[Control], category: Category) -> Vec<String> {
    let mut wrong = Vec::new();
    let wanted = Include::for_category(category);
    for fact in Fact::ALL {
        match box_for(controls, fact) {
            None => wrong.push(format!("no box named {:?}", fact.sends())),
            Some(found) => {
                let ticked = found.msaa.state & STATE_SYSTEM_CHECKED != 0;
                if ticked != wanted.includes(fact) {
                    wrong.push(format!(
                        "{fact:?} answers ticked={ticked} (state 0x{:x}) where {} is wanted",
                        found.msaa.state,
                        wanted.includes(fact)
                    ));
                }
            }
        }
    }
    wrong
}

// ── The window ─────────────────────────────────────────────────────────────

#[test]
fn test_the_controls_are_in_tab_order_with_their_names_and_roles_over_msaa() {
    complain(
        "in the dialog's controls over MSAA",
        &what_is_wrong_with_the_order(&the_harvest().on_open),
    );
}

#[test]
fn test_the_version_and_the_log_excerpt_are_ticked_when_it_opens() {
    let on_open = &the_harvest().on_open;
    let excerpt = box_for(on_open, Fact::LogExcerpt).map(|found| found.msaa.state);
    assert!(
        excerpt.is_some_and(|state| state & STATE_SYSTEM_CHECKED != 0),
        "the log excerpt's box answers {excerpt:?}"
    );
    complain(
        "in the ticks on open",
        &what_is_wrong_with_the_default_ticks(on_open, Category::Problem),
    );
}

#[test]
fn test_the_payload_box_holds_the_message_as_the_fields_are_filled() {
    let harvest = the_harvest();
    let shown = &harvest.payload_as_filled;

    for wanted in [
        "To: support@wixen.app",
        "From: dana@example.org, your default account",
        TYPED[0],
        TYPED[1],
        "da***@example.org",
        "20260923-184512-log-excerpt.txt",
    ] {
        assert!(shown.contains(wanted), "{wanted:?} is not shown:\n{shown}");
    }
    assert_eq!(shown, &harvest.payload_expected);
}

#[test]
fn test_a_security_concern_unticks_the_excerpt_and_goes_to_its_own_address() {
    let harvest = the_harvest();

    complain(
        "in the ticks after choosing a security concern",
        &what_is_wrong_with_the_default_ticks(&harvest.security_controls, Category::Security),
    );
    assert!(
        harvest
            .security_payload
            .starts_with(&format!("To: {SECURITY_ADDRESS}\n")),
        "{}",
        harvest.security_payload
    );
    assert!(
        harvest
            .security_payload
            .contains("Subject: [Wixen Mail] Report a security concern\n"),
        "{}",
        harvest.security_payload
    );
    assert!(
        !harvest.security_payload.contains("log-excerpt"),
        "{}",
        harvest.security_payload
    );
    let github = focusable(&harvest.security_controls)
        .into_iter()
        .map(|control| control.msaa.name.clone())
        .find(|name| name.contains("GitHub"));
    assert_eq!(
        github.as_deref(),
        Some("Open GitHub's private reporting page")
    );
    assert_eq!(harvest.security_github_page, PRIVATE_REPORTING_PAGE);
}

#[test]
fn test_with_no_account_send_is_unavailable_and_the_line_under_it_says_why() {
    let harvest = the_harvest();
    let send = focusable(&harvest.no_account_controls)
        .into_iter()
        .find(|control| control.msaa.name == "Send")
        .map(|control| control.msaa.state);

    assert!(
        send.is_some_and(|state| state & STATE_SYSTEM_UNAVAILABLE != 0),
        "Send answers {send:?}"
    );
    assert!(
        harvest
            .no_account_why
            .starts_with("No account is set up to send from."),
        "{:?}",
        harvest.no_account_why
    );
}

#[test]
fn test_the_doors_for_each_case_the_plan_names() {
    let dana = sender(true).account;
    for (category, who, allowed, open, page) in [
        (Category::Problem, None, true, false, ISSUE_PAGE),
        (Category::Problem, Some(&dana), false, false, ISSUE_PAGE),
        (Category::Problem, Some(&dana), true, true, ISSUE_PAGE),
        (
            Category::Security,
            Some(&dana),
            true,
            true,
            PRIVATE_REPORTING_PAGE,
        ),
        (
            Category::Security,
            None,
            true,
            false,
            PRIVATE_REPORTING_PAGE,
        ),
    ] {
        let doors = what_the_doors_do(category, who, allowed);
        assert_eq!(
            doors.send == SendDoor::Open,
            open,
            "{category:?} with an account {} and allowed {allowed}: {doors:?}",
            who.is_some()
        );
        assert_eq!(doors.github_page, page, "{category:?}");
    }
    assert_ne!(SUPPORT_ADDRESS, SECURITY_ADDRESS);
}

// ── The main window, read as text ──────────────────────────────────────────

const WX_APP: &str = "src/presentation/wx_app.rs";

/// One top-level function's text, from its signature to the brace that
/// closes it at the left margin, which is where rustfmt puts it.
fn body_of<'a>(source: &'a str, signature: &str) -> Option<&'a str> {
    let start = source.find(signature)?;
    let rest = &source[start..];
    rest.find("\n}\n").map(|end| &rest[..end])
}

/// How many times a queued row is built, outside comments.
fn rows_built(text: &str) -> usize {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .filter(|line| line.contains("QueuedOutboxMessage {"))
        .count()
}

fn what_is_wrong_with_the_one_path(source: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    let built = rows_built(source);
    if built != 1 {
        wrong.push(format!(
            "a queued row is built {built} times where once is wanted"
        ));
    }
    match body_of(source, "fn put_in_the_outbox(") {
        Some(body) if rows_built(body) == 1 => {}
        _ => wrong.push("put_in_the_outbox does not build the queued row".to_string()),
    }
    for caller in ["fn queue_for_sending(", "fn send_the_report("] {
        match body_of(source, caller) {
            Some(body) if body.contains("put_in_the_outbox(") => {}
            Some(_) => wrong.push(format!("{caller} does not call put_in_the_outbox")),
            None => wrong.push(format!("{caller} is not in {WX_APP}")),
        }
    }
    wrong
}

fn what_is_wrong_with_the_help_item(source: &str) -> Vec<String> {
    let mut wrong = Vec::new();
    let menu = source.find("let help = Menu::builder()").and_then(|start| {
        let rest = &source[start..];
        rest.find("MenuBar::builder()").map(|end| &rest[..end])
    });
    let Some(menu) = menu else {
        return vec!["the Help menu's building could not be found".to_string()];
    };
    let item = r#"ID_SEND_FEEDBACK,
            "Send &Feedback...\tCtrl+Shift+F","#;
    match (menu.find(item), menu.find("ID_CHECK_FOR_UPDATES")) {
        (Some(at), Some(updates)) if at < updates => {}
        (Some(_), _) => wrong.push("Send Feedback is not before Check for Updates".to_string()),
        (None, _) => wrong.push(format!("the Help menu does not append {item}")),
    }
    if !source.contains("_ if id == ID_SEND_FEEDBACK =>") {
        wrong.push("no menu arm answers ID_SEND_FEEDBACK".to_string());
    }
    wrong
}

fn the_main_window() -> String {
    std::fs::read_to_string(WX_APP)
        .expect("the main window's source")
        .replace("\r\n", "\n")
}

#[test]
fn test_one_function_builds_the_queued_row_and_both_sends_call_it() {
    complain(
        "in the one sending path",
        &what_is_wrong_with_the_one_path(&the_main_window()),
    );
}

#[test]
fn test_help_offers_send_feedback_on_ctrl_shift_f() {
    complain(
        "in the Help menu",
        &what_is_wrong_with_the_help_item(&the_main_window()),
    );
}

// ── The readings refuse what they exist to refuse ──────────────────────────

#[test]
fn test_the_one_path_reading_refuses_a_second_queued_row() {
    // Judged by the delta, so a recorded break that plants a second row for
    // real leaves this green: one more row than the tree holds, and the
    // reading says how many.
    let source = the_main_window();
    let Some(body) = body_of(&source, "fn send_the_report(") else {
        panic!("fn send_the_report( is not in {WX_APP}, so there is nowhere to plant");
    };
    let Some(opens) = body.find("{\n") else {
        panic!("send_the_report's body has no opening brace to plant after");
    };
    let mut planted_body = body.to_string();
    planted_body.insert_str(
        opens + 2,
        "    let planted = crate::data::message_cache::QueuedOutboxMessage { planted };\n",
    );
    let planted = source.replacen(body, &planted_body, 1);
    let rows = rows_built(&planted);
    assert_eq!(rows, rows_built(&source) + 1, "the plant added no row");

    let found = what_is_wrong_with_the_one_path(&planted);

    assert!(
        found
            .iter()
            .any(|it| it.contains(&format!("built {rows} times"))),
        "the reading accepted a second queued row: {found:?}"
    );
}

#[test]
fn test_the_tick_reading_refuses_an_excerpt_left_unticked() {
    let mut controls = the_harvest().on_open.clone();
    let already = what_is_wrong_with_the_default_ticks(&controls, Category::Problem).len();
    let Some(excerpt) = controls
        .iter_mut()
        .find(|control| control.msaa.name == Fact::LogExcerpt.sends())
    else {
        panic!("no excerpt box to untick");
    };
    excerpt.msaa.state &= !STATE_SYSTEM_CHECKED;

    let found = what_is_wrong_with_the_default_ticks(&controls, Category::Problem);

    assert!(
        found.len() > already && found.iter().any(|it| it.contains("LogExcerpt")),
        "the reading accepted an unticked excerpt: {found:?}"
    );
}

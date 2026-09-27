//! Sign and Encrypt in the composer: two check boxes a person reaches by
//! keyboard from anywhere in the window, and a Send that either protects the
//! message or says exactly why it cannot, with nothing sent (GAP-05, #52
//! points 4 and 5, 13-21).
//!
//! **What is read.** The real composer, built by `wx_compose::build_compose_dialog`
//! in this test's own window session: the two boxes over MSAA at their own
//! handles, their name, role, state and description, and where they stand in
//! the order Tab walks. Then the page the composer loads, with Alt+G and Alt+Y
//! pressed inside the message the way the page hears them, and the boxes read
//! again. Then what Send decides, through `wx_compose::may_it_go`, the one
//! function the composer's Send asks, handed a check built from
//! `application::protecting::at_send` and what is held.
//!
//! **What is read from the source, and why.** The wiring between those pieces
//! lives in functions no test in their own layer can reach without a person
//! pressing keys in a modal window: the composer's modal loop, its page
//! handler, the main window's queue and draft, and the send loop. Each is read
//! as text with a companion that hands the reading a body missing the call and
//! requires a complaint.
//!
//! **What is not read here, said plainly.** A message queued with a real key
//! and sent: an integration test is built against the library without its test
//! backing, so a real key here would be read from, or written to, the
//! credential store and the certificate store of whoever runs the tests. Those
//! halves are measured in `application::protecting`'s own tests with the
//! keyholder's certificate in memory and a credential store of the test's own.
//! What a screen reader says when the boxes are ticked is the ledger's.
//!
//! One window session for the file, shared through a `OnceLock`, on the shape
//! `tests/a_signature_follows_the_from_account.rs` uses. Runs under
//! `WIXEN_NO_AUDIO` as CI does, on a temporary data directory in
//! `WIXEN_MAIL_DATA`, never a person's profile.

#![cfg(windows)]

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::ffi::c_void;
use std::fs;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::allowed::SIGNING_AND_ENCRYPTING_IS_EXPERIMENTAL;
use wixen_mail::application::pgp_keys::LockedKey;
use wixen_mail::application::protecting::{
    CannotProtect, Choice, KeptFor, WhatIsHeld, YourPgpKey, at_send,
};
use wixen_mail::common::types::MessageBody;
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::editor_document::{self, EditorMessage};
use wixen_mail::presentation::wx_compose::{self, ComposeData, ProtectionCheck, may_it_go};
use wxdragon::event::WebViewEvents;
use wxdragon::prelude::*;
use wxdragon::widgets::WebView;

type Harvest = BTreeMap<&'static str, String>;

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const VT_I4: u16 = 3;
const CHILDID_SELF: i64 = 0;
const GW_HWNDNEXT: u32 = 2;

/// MSAA's role for a check box (oleacc.h).
const ROLE_SYSTEM_CHECKBUTTON: i64 = 0x2c;
/// MSAA's state bits for a greyed control and a ticked one (oleacc.h).
const STATE_SYSTEM_UNAVAILABLE: i64 = 0x1;
const STATE_SYSTEM_CHECKED: i64 = 0x10;

const TICK_MS: i32 = 30;
/// Ticks before the run says it waited too long: a minute.
const GIVE_UP_AFTER_TICKS: u32 = 2000;
/// Ticks of nothing after a key, so what it started has finished.
const TICKS_TO_SETTLE: u32 = 6;

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
        Variant::child(0).with_type(0)
    }

    fn with_type(self, vt: u16) -> Self {
        Variant { vt, ..self }
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
const VTBL_GET_ACC_DESCRIPTION: usize = 12;
const VTBL_GET_ACC_ROLE: usize = 13;
const VTBL_GET_ACC_STATE: usize = 14;

#[link(name = "user32")]
unsafe extern "system" {
    fn GetWindow(hwnd: isize, command: u32) -> isize;
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
    description: String,
    role: i64,
    state: i64,
}

impl Msaa {
    /// One line a reading can be compared against.
    fn said(&self) -> String {
        format!(
            "{} | check box: {} | greyed: {} | ticked: {} | {}",
            self.name,
            self.role == ROLE_SYSTEM_CHECKBUTTON,
            self.state & STATE_SYSTEM_UNAVAILABLE != 0,
            self.state & STATE_SYSTEM_CHECKED != 0,
            self.description
        )
    }
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
        let get_description: GetBstrFn =
            std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_DESCRIPTION));
        let get_role: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_ROLE));
        let get_state: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_STATE));
        let text = |get: GetBstrFn| {
            let mut read: *mut u16 = std::ptr::null_mut();
            match get(object, Variant::child(CHILDID_SELF), &mut read) >= 0 {
                true => take_bstr(read),
                false => String::new(),
            }
        };
        let number = |get: GetVariantFn| {
            let mut read = Variant::empty();
            match get(object, Variant::child(CHILDID_SELF), &mut read) >= 0 && read.vt == VT_I4 {
                true => read.val & 0xFFFF_FFFF,
                false => -1,
            }
        };
        let read = Msaa {
            name: text(get_name),
            description: text(get_description),
            role: number(get_role),
            state: number(get_state),
        };
        let release: ReleaseFn = std::mem::transmute(vtable_entry(object, VTBL_RELEASE));
        release(object);
        Ok(read)
    }
}

/// The window Windows puts after this one among its siblings, which is the
/// order controls were made in and the order Tab walks.
fn the_one_after(hwnd: isize) -> isize {
    // SAFETY: a live window handle; nothing is changed.
    unsafe { GetWindow(hwnd, GW_HWNDNEXT) }
}

/// A reading, or what stopped it being taken.
fn or_why(result: Result<String, String>) -> String {
    result.unwrap_or_else(|why| format!("(not read: {why})"))
}

// ── The window session ──────────────────────────────────────────────────────

/// One thing the page's run does.
#[derive(Clone, Copy)]
enum Act {
    /// Alt and a letter pressed inside the message, the way the page hears
    /// it, then ticks of nothing.
    Press(char),
    /// Both boxes read over MSAA into the harvest under this name.
    Read(&'static str),
}

const THE_ACTS: [Act; 4] = [
    Act::Press('g'),
    Act::Read("after Alt+G"),
    Act::Press('y'),
    Act::Read("after Alt+Y"),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    WaitingForBrowser,
    WaitingForPage,
    Settling(u32),
    Acting,
    Done,
}

struct Run {
    next: usize,
    phase: Phase,
    ticks: u32,
    busy: bool,
    harvest: Harvest,
    failure: Option<String>,
}

fn unquoted(answer: Option<String>) -> String {
    answer
        .map(|text| editor_document::plain_from_editor(&text))
        .unwrap_or_else(|| "(run_script answered None)".to_string())
}

fn the_page_is_up(body_editor: &WebView) -> bool {
    unquoted(body_editor.run_script(
        "(typeof window.wixenRules === 'object' && !window.__protection_run) ? 'up' : 'not yet'",
    )) == "up"
}

/// Alt and a letter, dispatched on the message the way a key arrives there.
fn press_inside_the_message(body_editor: &WebView, letter: char) {
    let _ = body_editor.run_script(&format!(
        "(function () {{ var b = document.getElementById('wixen-body'); \
         b.dispatchEvent(new KeyboardEvent('keydown', {{ key: '{letter}', altKey: true, bubbles: true, cancelable: true }})); \
         return 'pressed'; }})()"
    ));
}

fn both_boxes(sign: &CheckBox, encrypt: &CheckBox) -> Result<String, String> {
    Ok(format!(
        "Sign {} ({}), Encrypt {} ({})",
        sign.get_value(),
        msaa_of(sign.get_handle() as isize)?.said(),
        encrypt.get_value(),
        msaa_of(encrypt.get_handle() as isize)?.said(),
    ))
}

fn one_act(
    run: &Rc<RefCell<Run>>,
    body_editor: &WebView,
    sign: &CheckBox,
    encrypt: &CheckBox,
) -> Phase {
    let index = run.borrow().next;
    let Some(act) = THE_ACTS.get(index).copied() else {
        return Phase::Done;
    };
    run.borrow_mut().next = index + 1;
    match act {
        Act::Press(letter) => {
            press_inside_the_message(body_editor, letter);
            Phase::Settling(TICKS_TO_SETTLE)
        }
        Act::Read(name) => {
            let read = or_why(both_boxes(sign, encrypt));
            run.borrow_mut().harvest.insert(name, read);
            Phase::Acting
        }
    }
}

/// The boxes as the composer is built, read before anything is pressed.
fn read_as_built(widgets: &wx_compose::ComposeDialogWidgets, into: &mut Harvest) {
    for (name, check) in [
        ("sign, as built", &widgets.sign_box),
        ("encrypt, as built", &widgets.encrypt_box),
    ] {
        into.insert(
            name,
            or_why(msaa_of(check.get_handle() as isize).map(|read| read.said())),
        );
    }
    let after_schedule = the_one_after(widgets.schedule_btn.get_handle() as isize);
    let after_that = the_one_after(after_schedule);
    into.insert(
        "the two after Schedule",
        or_why((|| {
            Ok(format!(
                "{}, {}",
                msaa_of(after_schedule)?.name,
                msaa_of(after_that)?.name
            ))
        })()),
    );
}

fn take_the_harvest() -> Result<Harvest, String> {
    let data = tempfile::tempdir().map_err(|e| format!("a data directory: {e}"))?;
    // SAFETY: set before the window session starts any thread, and nothing
    // has read either yet.
    unsafe {
        std::env::set_var("WIXEN_MAIL_DATA", data.path());
        std::env::set_var("WIXEN_NO_AUDIO", "1");
    }
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let finish = {
                let outcome = outcome.clone();
                move |taken: Result<Harvest, String>| {
                    if let Ok(mut slot) = outcome.lock() {
                        *slot = Some(taken);
                    }
                    wxdragon::call_after(Box::new(move || {
                        app.exit_main_loop();
                    }));
                }
            };
            let frame = Frame::builder()
                .with_title("Sign and Encrypt in the composer, the reading")
                .build();
            let a11y = match Accessibility::new() {
                Ok(a11y) => Arc::new(a11y),
                Err(why) => return finish(Err(format!("accessibility: {why:?}"))),
            };
            let widgets = wx_compose::build_compose_dialog(
                &frame,
                "Compose New Message",
                &["ada@example.com".to_string()],
                0,
                None,
            );
            let mut harvest = Harvest::new();
            read_as_built(&widgets, &mut harvest);

            let dialog = widgets.dialog;
            let body_editor = widgets.body_editor;
            let browser = widgets.browser.clone();
            let sign = widgets.sign_box;
            let encrypt = widgets.encrypt_box;
            // The route the composer's own handler takes: the page posts the
            // letter it heard, and what it reached is handed to the function
            // the composer hands it to.
            body_editor.on_script_message_received({
                let a11y = a11y.clone();
                move |event| {
                    let reached = event
                        .get_string()
                        .and_then(|raw| editor_document::parse_message(&raw));
                    if let Some(EditorMessage::Reached(reached)) = reached {
                        wx_compose::tick_from_the_page(reached, &sign, &encrypt, &a11y);
                    }
                }
            });
            body_editor.set_page(
                &editor_document::editor_document(&MessageBody::Plain(String::new()), "en", false),
                "",
            );
            dialog.show(true);

            let run = Rc::new(RefCell::new(Run {
                next: 0,
                phase: Phase::WaitingForBrowser,
                ticks: 0,
                busy: false,
                harvest,
                failure: None,
            }));
            let ticker = Rc::new(Timer::new(&dialog));
            ticker.on_tick({
                let run = run.clone();
                let ticker = ticker.clone();
                move |_| {
                    let phase = {
                        let mut run = run.borrow_mut();
                        if run.busy {
                            return;
                        }
                        run.busy = true;
                        run.ticks += 1;
                        if run.ticks > GIVE_UP_AFTER_TICKS && run.phase != Phase::Done {
                            run.failure = Some("the page's run waited too long".to_string());
                            run.phase = Phase::Done;
                            run.next = THE_ACTS.len();
                        }
                        run.phase
                    };
                    let next = match phase {
                        Phase::WaitingForBrowser if browser.is_ready() => Phase::WaitingForPage,
                        Phase::WaitingForPage if the_page_is_up(&body_editor) => {
                            let _ = body_editor.run_script("window.__protection_run = 1; 'marked'");
                            Phase::Settling(TICKS_TO_SETTLE)
                        }
                        Phase::Settling(left) if left > 1 => Phase::Settling(left - 1),
                        Phase::Settling(_) => Phase::Acting,
                        Phase::Acting => one_act(&run, &body_editor, &sign, &encrypt),
                        waiting => waiting,
                    };
                    let finished = next == Phase::Done && phase != Phase::Done;
                    {
                        let mut run = run.borrow_mut();
                        run.phase = next;
                        run.busy = false;
                    }
                    if finished {
                        ticker.stop();
                        let mut run = run.borrow_mut();
                        let taken = match run.failure.take() {
                            Some(why) => Err(why),
                            None => Ok(std::mem::take(&mut run.harvest)),
                        };
                        browser.destroy_when_ready(dialog);
                        finish(taken);
                    }
                }
            });
            ticker.start(TICK_MS, false);
            // The tick closure holds the other handle; the timer must outlive
            // on_init.
            std::mem::forget(ticker);
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

fn reading(name: &str) -> &'static str {
    static HARVEST: OnceLock<Result<Harvest, String>> = OnceLock::new();
    let harvest = match HARVEST.get_or_init(take_the_harvest) {
        Ok(harvest) => harvest,
        Err(why) => panic!("the window session could not be read: {why}"),
    };
    harvest
        .get(name)
        .map(String::as_str)
        .unwrap_or_else(|| panic!("nothing was read for {name:?}"))
}

// ── The boxes, over MSAA at their own handles ─────────────────────────────

fn a_box_reads(said: &str, name: &str, what_it_does: &str) -> Result<(), String> {
    let wanted =
        format!("{name} | check box: true | greyed: false | ticked: false | {what_it_does}");
    match said.starts_with(&wanted) && said.ends_with(SIGNING_AND_ENCRYPTING_IS_EXPERIMENTAL) {
        true => Ok(()),
        false => Err(format!("the box reads {said:?}, not {wanted:?}...")),
    }
}

#[test]
fn test_the_sign_box_is_named_a_check_box_and_never_greyed() {
    a_box_reads(
        reading("sign, as built"),
        "Sign (experimental)",
        "Signs this message",
    )
    .unwrap();
}

#[test]
fn test_the_encrypt_box_is_named_a_check_box_and_never_greyed() {
    a_box_reads(
        reading("encrypt, as built"),
        "Encrypt (experimental)",
        "Encrypts this message",
    )
    .unwrap();
}

#[test]
fn test_companion_a_greyed_box_is_refused() {
    let greyed = format!(
        "Sign (experimental) | check box: true | greyed: true | ticked: false | Signs this message. {SIGNING_AND_ENCRYPTING_IS_EXPERIMENTAL}"
    );
    assert!(a_box_reads(&greyed, "Sign (experimental)", "Signs this message").is_err());
}

#[test]
fn test_the_two_boxes_come_right_after_schedule_in_the_order_tab_walks() {
    assert_eq!(
        reading("the two after Schedule"),
        "Sign (experimental), Encrypt (experimental)"
    );
}

// ── Alt+G and Alt+Y from inside the message ───────────────────────────────

#[test]
fn test_alt_g_inside_the_message_ticks_sign_and_leaves_encrypt() {
    let after = reading("after Alt+G");
    assert!(
        after.starts_with("Sign true (Sign (experimental) | check box: true | greyed: false | ticked: true")
            && after.contains("Encrypt false (Encrypt (experimental) | check box: true | greyed: false | ticked: false"),
        "{after}"
    );
}

#[test]
fn test_alt_y_inside_the_message_ticks_encrypt() {
    let after = reading("after Alt+Y");
    assert!(
        after.contains(
            "Encrypt true (Encrypt (experimental) | check box: true | greyed: false | ticked: true"
        ),
        "{after}"
    );
}

// ── What Send decides ──────────────────────────────────────────────────────

const ADA: &str = "ada@example.com";
const GRACE: &str = "grace@example.com";
const ADAS_FINGERPRINT: &str = "ADA0000000000000000000000000000000000000";

fn a_message_to_grace(protection: Choice) -> ComposeData {
    ComposeData {
        to: format!("Grace Hopper <{GRACE}>"),
        cc: String::new(),
        bcc: String::new(),
        subject: "Private".to_string(),
        body: "<p>Only for you.</p>".to_string(),
        body_plain: "Only for you.".to_string(),
        html_mode: true,
        account_index: Some(0),
        attachments: Vec::new(),
        answering: None,
        send_at: None,
        protection,
    }
}

fn adas_key(waiting: Option<LockedKey>) -> YourPgpKey {
    YourPgpKey {
        fingerprint: ADAS_FINGERPRINT.to_string(),
        signs: true,
        can_be_encrypted_to: true,
        waiting_for_its_passphrase: waiting,
    }
}

/// The check the main window hands the composer, over what is held here
/// rather than what the stores hold.
fn a_check(held: WhatIsHeld) -> ProtectionCheck {
    Box::new(move |data: &ComposeData| {
        at_send(data.protection, ADA, &data.to, &data.cc, &data.bcc, &held)
    })
}

/// What Send decided, what it asked for and what it said.
fn at_send_press(
    data: &ComposeData,
    check: &ProtectionCheck,
    passphrase_given: bool,
) -> (bool, Vec<String>, Vec<String>) {
    let mut asked = Vec::new();
    let mut said = Vec::new();
    let goes = may_it_go(
        data,
        check,
        &mut |key: &LockedKey| {
            asked.push(key.whose.clone());
            passphrase_given
        },
        &mut |words: &str| said.push(words.to_string()),
    );
    (goes, asked, said)
}

#[test]
fn test_send_with_encrypt_ticked_and_nothing_kept_for_a_recipient_does_not_go_and_names_them() {
    let check = a_check(WhatIsHeld {
        own_key: Some(adas_key(None)),
        ..WhatIsHeld::default()
    });

    let (goes, asked, said) = at_send_press(&a_message_to_grace(Choice::Encrypted), &check, true);

    assert!(!goes, "a message that cannot be encrypted went");
    assert!(asked.is_empty(), "{asked:?}");
    assert!(
        said.len() == 1 && said[0].contains(GRACE) && said[0].ends_with("Nothing was sent."),
        "{said:?}"
    );
}

#[test]
fn test_send_with_a_key_kept_for_everybody_goes_and_says_nothing() {
    let check = a_check(WhatIsHeld {
        own_key: Some(adas_key(None)),
        theirs: vec![KeptFor {
            address: GRACE.to_string(),
            certificates: Vec::new(),
            public_keys: vec!["Grace's armour".to_string()],
        }],
        ..WhatIsHeld::default()
    });

    let (goes, asked, said) = at_send_press(
        &a_message_to_grace(Choice::SignedAndEncrypted),
        &check,
        true,
    );

    assert!(
        goes && asked.is_empty() && said.is_empty(),
        "{asked:?} {said:?}"
    );
}

/// A check whose key is locked until `open` says the passphrase was typed.
fn a_check_with_a_locked_key(open: Rc<Cell<bool>>) -> ProtectionCheck {
    let locked = LockedKey {
        whose: "Ada Lovelace <ada@example.com>".to_string(),
        fingerprint: ADAS_FINGERPRINT.to_string(),
    };
    Box::new(move |data: &ComposeData| {
        let held = WhatIsHeld {
            own_key: Some(adas_key((!open.get()).then(|| locked.clone()))),
            ..WhatIsHeld::default()
        };
        at_send(data.protection, ADA, &data.to, &data.cc, &data.bcc, &held)
    })
}

#[test]
fn test_a_locked_key_is_asked_for_once_and_the_message_goes_when_it_opens() {
    let open = Rc::new(Cell::new(false));
    let check = a_check_with_a_locked_key(open.clone());
    let mut asked = Vec::new();

    let goes = may_it_go(
        &a_message_to_grace(Choice::Signed),
        &check,
        &mut |key: &LockedKey| {
            asked.push(key.whose.clone());
            open.set(true);
            true
        },
        &mut |said: &str| panic!("nothing should be said: {said}"),
    );

    assert!(goes);
    assert_eq!(asked, vec!["Ada Lovelace <ada@example.com>".to_string()]);
}

#[test]
fn test_a_locked_key_whose_passphrase_is_not_given_does_not_go() {
    let check = a_check_with_a_locked_key(Rc::new(Cell::new(false)));

    let (goes, asked, said) = at_send_press(&a_message_to_grace(Choice::Signed), &check, false);

    assert!(!goes, "a message whose key stayed locked went");
    assert_eq!(asked.len(), 1, "{asked:?}");
    assert!(
        said.len() == 1 && said[0].contains("locked") && said[0].ends_with("Nothing was sent."),
        "{said:?}"
    );
}

#[test]
fn test_a_key_still_locked_after_its_passphrase_is_not_asked_for_twice() {
    // A passphrase that answers yes and a key that stays locked: asking again
    // would be a dialog that never stops coming back.
    let check = a_check_with_a_locked_key(Rc::new(Cell::new(false)));

    let (goes, asked, said) = at_send_press(&a_message_to_grace(Choice::Signed), &check, true);

    assert!(!goes);
    assert_eq!(asked.len(), 1, "{asked:?}");
    assert_eq!(said.len(), 1, "{said:?}");
}

// ── The wiring, read from the source ──────────────────────────────────────

const THE_COMPOSER: &str = "src/presentation/wx_compose.rs";
const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

fn shipped(path: &str) -> String {
    what_ships(
        &fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("{path}: {e}"))
            .replace("\r\n", "\n"),
    )
}

/// A function's text, from its signature to the brace that closes it at the
/// left margin.
fn body_of(source: &str, signature: &str) -> Result<String, String> {
    let at = source
        .find(signature)
        .ok_or_else(|| format!("{signature} is no longer in the file"))?;
    let rest = &source[at..];
    let ends = rest.find("\n}\n").map_or(rest.len(), |end| end + 2);
    Ok(rest[..ends].to_string())
}

/// The text between two marks, or why there is none.
fn between(source: &str, from: &str, to: &str) -> Result<String, String> {
    let starts = source
        .find(from)
        .ok_or_else(|| format!("{from:?} is no longer in the file"))?;
    let rest = &source[starts..];
    let ends = rest
        .find(to)
        .ok_or_else(|| format!("{to:?} no longer follows {from:?}"))?;
    Ok(rest[..ends].to_string())
}

/// Whether a piece of the source says what it has to, however it is wrapped:
/// both are read with every space and line break taken out, since a call
/// rustfmt breaks over three lines is still the call.
fn it_says(text: Result<String, String>, what: &str) -> Result<(), String> {
    let text = text?;
    let unwrapped = |written: &str| written.split_whitespace().collect::<String>();
    match unwrapped(&text).contains(&unwrapped(what)) {
        true => Ok(()),
        false => Err(format!("it does not say {what:?}:\n{text}")),
    }
}

#[test]
fn test_companion_a_body_without_the_call_is_refused() {
    let planted = "fn put_in_the_outbox(\n    let queued = QueuedOutboxMessage {\n        references: None,\n    };\n}\n";
    assert!(
        it_says(
            body_of(planted, "fn put_in_the_outbox("),
            "protection: data.protection"
        )
        .is_err()
    );
    assert!(it_says(body_of(planted, "fn gone("), "anything").is_err());
}

#[test]
fn test_the_composers_send_asks_about_the_boxes_before_the_message_goes() {
    it_says(
        between(
            &shipped(THE_COMPOSER),
            "if let Some(why) = why_it_cannot_be_sent(&data)",
            "break 'compose ComposeResult::Send",
        ),
        "may_it_go(",
    )
    .unwrap();
}

#[test]
fn test_the_page_hands_alt_g_and_alt_y_to_the_boxes() {
    it_says(
        Ok(shipped(THE_COMPOSER)),
        "tick_from_the_page(reached, &sign_box, &encrypt_box, &a11y)",
    )
    .unwrap();
}

#[test]
fn test_the_message_read_from_the_window_carries_the_boxes() {
    it_says(
        Ok(shipped(THE_COMPOSER)),
        "Choice::from_boxes(sign_box.get_value(), encrypt_box.get_value()",
    )
    .unwrap();
}

#[test]
fn test_a_draft_reopens_with_its_boxes_as_they_were_left() {
    it_says(
        Ok(shipped(THE_COMPOSER)),
        "show_the_choice(&sign_box, &encrypt_box, data.protection)",
    )
    .unwrap();
}

#[test]
fn test_the_queued_row_and_the_saved_draft_carry_the_choice() {
    let app = shipped(THE_MAIN_WINDOW);
    it_says(
        body_of(&app, "fn put_in_the_outbox("),
        "protection: data.protection",
    )
    .unwrap();
    it_says(
        body_of(&app, "fn save_as_draft("),
        "protection: data.protection",
    )
    .unwrap();
    it_says(
        body_of(&app, "fn a_message_taken_back("),
        "protection: message.protection",
    )
    .unwrap();
}

#[test]
fn test_the_composer_is_handed_the_check_and_the_send_loop_gathers_what_is_held() {
    let app = shipped(THE_MAIN_WINDOW);
    it_says(
        between(&app, "wx_compose::show_compose_dialog_full(", ") {"),
        "checking_protection",
    )
    .unwrap();
    it_says(body_of(&app, "fn flush_outbox("), "protected_as_asked(").unwrap();
}

#[test]
fn test_the_announcement_after_a_protected_send_says_how_it_went() {
    it_says(
        between(
            &shipped(THE_MAIN_WINDOW),
            "UIUpdate::OutboxSendResult {",
            "UIUpdate::TheNetworkChanged",
        ),
        "sent_from_the_outbox(",
    )
    .unwrap();
}

#[test]
fn test_companion_a_protection_nobody_asked_about_is_not_passed() {
    // The shape this plan replaced: a request that always sent plain.
    let planted = "fn put_in_the_outbox(\n    protection: Choice::Plain,\n}\n";
    assert!(
        it_says(
            body_of(planted, "fn put_in_the_outbox("),
            "protection: data.protection"
        )
        .is_err()
    );
    let _ = CannotProtect::ABlindCopyWouldShow;
}

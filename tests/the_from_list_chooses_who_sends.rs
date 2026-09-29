//! Compose's From list offers every address an account sends from, is named
//! for what it chooses, and the entry chosen is who the message is from for
//! everything that asks (#59, GAP-10, 13-35).
//!
//! Until 13-35 the list was a position among the accounts. People lookup, the
//! signature and the preview followed that position, and the send and the
//! draft went out from whichever account was open in the main window, so a
//! message could show one account's address, sign with its signature and
//! leave from another.
//!
//! **What is read in a window.** The real composer, built by
//! `wx_compose::build_compose_dialog` from a From list made by the rule the
//! program makes it by, `identities::the_from_list`: its entries in order, and
//! its name over MSAA at the handle Windows gives focus to, which is what NVDA
//! reads for a native list. Then the real Preview Before Send, built for a
//! message from an other address and for one from an account's own, and its
//! From line read off the window.
//!
//! **What is read from the source, and why.** The composer's modal loop and
//! the main window's `open_compose` cannot be reached from a test without a
//! person pressing keys in a modal window, so the wiring between them is read
//! as text, as `tests/signing_and_encrypting_from_the_composer.rs` reads it.
//! Each reading has a companion that hands it the shape it replaced and
//! requires a complaint.
//!
//! One window session for the file, shared through a `OnceLock`. Runs under
//! `WIXEN_NO_AUDIO` as CI does, on a temporary data directory in
//! `WIXEN_MAIL_DATA`, never a person's profile.

#![cfg(windows)]

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::ffi::c_void;
use std::fs;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::identities::{self, FromEntry, Identity};
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::data::Account;
use wixen_mail::presentation::browser_ready::BrowserReady;
use wixen_mail::presentation::wx_compose::{self, ComposeData};
use wxdragon::prelude::*;

type Harvest = BTreeMap<&'static str, String>;

const WORK: &str = "acct-work";
const HOME: &str = "acct-home";

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const VT_I4: u16 = 3;
const CHILDID_SELF: i64 = 0;
/// MSAA's role for a combo box (oleacc.h), which a wxChoice is on Windows.
const ROLE_SYSTEM_COMBOBOX: i64 = 0x2e;

const TICK_MS: i32 = 30;
/// Ticks before the run says it waited too long: a minute.
const GIVE_UP_AFTER_TICKS: u32 = 2000;

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
            ..Self::child(0)
        }
    }
}

type Hresult = i32;
type ReleaseFn = unsafe extern "system" fn(*mut c_void) -> u32;
type GetVariantFn = unsafe extern "system" fn(*mut c_void, Variant, *mut Variant) -> Hresult;
type GetBstrFn = unsafe extern "system" fn(*mut c_void, Variant, *mut *mut u16) -> Hresult;

// IAccessible's vtable: IUnknown (3), IDispatch (4), then get_accParent (7),
// get_accChildCount (8), get_accChild (9), get_accName (10), get_accValue
// (11), get_accDescription (12), get_accRole (13).
const VTBL_RELEASE: usize = 2;
const VTBL_GET_ACC_NAME: usize = 10;
const VTBL_GET_ACC_ROLE: usize = 13;

#[link(name = "user32")]
unsafe extern "system" {
    fn EnumChildWindows(
        parent: isize,
        callback: extern "system" fn(isize, isize) -> i32,
        lparam: isize,
    ) -> i32;
    fn GetWindowTextW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
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

/// A window's own name and role over MSAA, which is what NVDA reads for a
/// native control.
fn msaa_of(hwnd: isize) -> Result<(String, i64), String> {
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
        let mut said: *mut u16 = std::ptr::null_mut();
        let name = match get_name(object, Variant::child(CHILDID_SELF), &mut said) >= 0 {
            true => take_bstr(said),
            false => String::new(),
        };
        let get_role: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_ROLE));
        let mut role = Variant::empty();
        let role = match get_role(object, Variant::child(CHILDID_SELF), &mut role) >= 0
            && role.vt == VT_I4
        {
            true => role.val & 0xFFFF_FFFF,
            false => -1,
        };
        let release: ReleaseFn = std::mem::transmute(vtable_entry(object, VTBL_RELEASE));
        release(object);
        Ok((name, role))
    }
}

thread_local! {
    static FOUND: RefCell<Vec<isize>> = const { RefCell::new(Vec::new()) };
}

extern "system" fn collect(hwnd: isize, _lparam: isize) -> i32 {
    FOUND.with(|found| found.borrow_mut().push(hwnd));
    1
}

fn window_text(hwnd: isize) -> String {
    let mut buffer = [0u16; 1024];
    // SAFETY: the buffer is as long as the count says.
    let len = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

/// The text of every child of a window, in the order Windows holds them.
fn the_texts_of(window: isize) -> Vec<String> {
    FOUND.with(|found| found.borrow_mut().clear());
    // SAFETY: the callback only pushes to this thread's local.
    unsafe { EnumChildWindows(window, collect, 0) };
    FOUND
        .with(|found| found.borrow().clone())
        .into_iter()
        .map(window_text)
        .collect()
}

/// A reading, or what stopped it being taken.
fn or_why(result: Result<String, String>) -> String {
    result.unwrap_or_else(|why| format!("(not read: {why})"))
}

// ── What the window is built from ─────────────────────────────────────────

fn an_account(id: &str, name: &str, sender_name: &str) -> Account {
    Account {
        id: id.to_string(),
        name: name.to_string(),
        email: format!("{}@example.com", name.to_lowercase()),
        sender_name: sender_name.to_string(),
        ..Account::default()
    }
}

/// Work, then Home, then the other address Home sends from, by the rule the
/// program builds the list by.
fn the_from_list() -> Vec<FromEntry> {
    let others = HashMap::from([(
        HOME.to_string(),
        vec![Identity::typed("i-help", "help@example.com", "Help Desk")],
    )]);
    identities::the_from_list(
        &[
            an_account(WORK, "Work", "Ada Lovelace"),
            an_account(HOME, "Home", ""),
        ],
        &others,
    )
}

fn a_message_from(entry: &FromEntry) -> ComposeData {
    ComposeData {
        to: "grace@example.com".to_string(),
        cc: String::new(),
        bcc: String::new(),
        subject: "Tomorrow".to_string(),
        body: "<p>See you then.</p>".to_string(),
        body_plain: "See you then.".to_string(),
        html_mode: true,
        from: Some(entry.clone()),
        attachments: Vec::new(),
        answering: None,
        send_at: None,
        protection: Default::default(),
    }
}

// ── The window session ────────────────────────────────────────────────────

/// Every entry the list holds, in its order.
fn the_entries(choice: &Choice) -> String {
    (0..choice.get_count())
        .map(|at| choice.get_string(at).unwrap_or_default())
        .collect::<Vec<_>>()
        .join(" | ")
}

/// The list given focus, and the name and role of the window focus landed
/// on, and whether that window is the list.
fn where_focus_lands(choice: &Choice) -> Result<String, String> {
    choice.set_focus();
    // SAFETY: asks which window of this thread has focus.
    let focused = unsafe { GetFocus() };
    if focused == 0 {
        return Err("no window has focus".to_string());
    }
    let (name, role) = msaa_of(focused)?;
    Ok(format!(
        "{name} | combo box: {} | the list itself: {}",
        role == ROLE_SYSTEM_COMBOBOX,
        focused == choice.get_handle() as isize
    ))
}

/// What the preview's From line says, read off the window: the text after
/// the label "From:".
fn the_previews_from_line(texts: &[String]) -> Result<String, String> {
    texts
        .iter()
        .position(|text| text == "From:")
        .and_then(|at| texts.get(at + 1))
        .cloned()
        .ok_or_else(|| format!("no From line among {texts:?}"))
}

struct Built {
    dialog: Dialog,
    browser: BrowserReady,
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
                .with_title("The From list chooses who sends, the reading")
                .build();
            let from_list = the_from_list();
            let widgets = wx_compose::build_compose_dialog(
                &frame,
                "Compose New Message",
                &from_list,
                0,
                None,
            );
            widgets.dialog.show(true);

            let mut harvest = Harvest::new();
            harvest.insert(
                "the From list's entries",
                the_entries(&widgets.account_choice),
            );
            harvest.insert(
                "the From list, where focus lands",
                or_why(where_focus_lands(&widgets.account_choice)),
            );

            let mut built = vec![Built {
                dialog: widgets.dialog,
                browser: widgets.browser.clone(),
            }];
            for (name, entry) in [
                ("the preview's From line, an other address", &from_list[2]),
                ("the preview's From line, an account's own", &from_list[0]),
            ] {
                let preview = wx_compose::build_send_preview_dialog(
                    &widgets.dialog,
                    &a_message_from(entry),
                    None,
                );
                harvest.insert(
                    name,
                    or_why(the_previews_from_line(&the_texts_of(
                        preview.dialog.get_handle() as isize,
                    ))),
                );
                built.push(Built {
                    dialog: preview.dialog,
                    browser: preview.browser,
                });
            }

            // Nothing is torn down before its browser exists: a dialog
            // destroyed first takes the process with it.
            let ticks = Rc::new(RefCell::new(0u32));
            let ticker = Rc::new(Timer::new(&frame));
            ticker.on_tick({
                let ticker = ticker.clone();
                let harvest = RefCell::new(Some(harvest));
                move |_| {
                    *ticks.borrow_mut() += 1;
                    let gave_up = *ticks.borrow() > GIVE_UP_AFTER_TICKS;
                    if !gave_up && !built.iter().all(|each| each.browser.is_ready()) {
                        return;
                    }
                    ticker.stop();
                    for each in &built {
                        each.browser.destroy_when_ready(each.dialog);
                    }
                    let Some(harvest) = harvest.borrow_mut().take() else {
                        return;
                    };
                    finish(match gave_up {
                        true => Err("the browsers were never made".to_string()),
                        false => Ok(harvest),
                    });
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

// ── The list, read in the window ──────────────────────────────────────────

/// What the list is named where focus lands, which has to be "From": it
/// chooses an address now, not an account.
fn named_from_where_focus_lands(read: &str) -> Result<(), String> {
    match read == "From | combo box: true | the list itself: true" {
        true => Ok(()),
        false => Err(format!("where focus lands the list reads {read:?}")),
    }
}

#[test]
fn test_the_from_list_offers_each_account_then_the_other_addresses_it_sends_from() {
    assert_eq!(
        reading("the From list's entries"),
        "work@example.com | home@example.com | help@example.com, another address on Home"
    );
}

#[test]
fn test_the_from_list_is_named_from_on_msaa_where_focus_lands() {
    named_from_where_focus_lands(reading("the From list, where focus lands")).unwrap();
}

#[test]
fn test_companion_a_list_still_named_from_account_is_refused() {
    // The name it had while it chose an account.
    assert!(
        named_from_where_focus_lands("From account | combo box: true | the list itself: true")
            .is_err()
    );
    assert!(
        named_from_where_focus_lands("From | combo box: true | the list itself: false").is_err()
    );
}

// ── The preview's From line, read in the window ───────────────────────────

#[test]
fn test_the_preview_says_the_other_address_and_its_name_on_its_from_line() {
    assert_eq!(
        reading("the preview's From line, an other address"),
        "Help Desk <help@example.com>"
    );
}

#[test]
fn test_the_preview_says_an_accounts_own_address_and_name_on_its_from_line() {
    assert_eq!(
        reading("the preview's From line, an account's own"),
        "Ada Lovelace <work@example.com>"
    );
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

/// The request people lookup makes, as the composer writes it.
fn the_lookup(composer: &str) -> Result<String, String> {
    between(composer, "(finding.start)(looking::LookFor {", "});")
}

#[test]
fn test_the_list_is_built_from_the_accounts_and_the_other_addresses_they_keep() {
    let open_compose = body_of(&shipped(THE_MAIN_WINDOW), "fn open_compose(");
    it_says(open_compose.clone(), "identities::the_from_list(").unwrap();
    it_says(open_compose, "the_other_addresses(").unwrap();
}

#[test]
fn test_each_entry_signs_with_its_accounts_signature() {
    it_says(
        body_of(&shipped(THE_MAIN_WINDOW), "fn open_compose("),
        "the_signature_a_message_starts_with(cache.as_deref(), &entry.account_id, sign_it)",
    )
    .unwrap();
}

#[test]
fn test_people_are_looked_up_in_the_chosen_entrys_account() {
    it_says(
        the_lookup(&shipped(THE_COMPOSER)),
        "from_account_id: the_entry_chosen(&account_choice, &from_list).map(|entry| entry.account_id)",
    )
    .unwrap();
}

#[test]
fn test_the_message_read_from_the_window_carries_the_chosen_entry() {
    it_says(
        Ok(shipped(THE_COMPOSER)),
        "from: the_entry_chosen(&account_choice, &from_list),",
    )
    .unwrap();
}

#[test]
fn test_companion_a_lookup_by_the_lists_position_is_refused() {
    // The shape this plan replaced: a position the worker turned back into an
    // account by a list of ids kept beside the names.
    let planted = "(finding.start)(looking::LookFor {\n    search,\n    name,\n    from_account: account_choice.get_selection(),\n});";
    assert!(
        it_says(
            the_lookup(planted),
            "from_account_id: the_entry_chosen(&account_choice, &from_list).map(|entry| entry.account_id)",
        )
        .is_err()
    );
    assert!(it_says(the_lookup("nothing here"), "anything").is_err());
}

// ── The send and the draft, read from the source ──────────────────────────

const THE_MANAGERS: &str = "src/presentation/managers.rs";

/// Whether a function takes who the message is from out of the entry the
/// composer chose, and never out of the account open in the main window
/// first. The open account is only what `who_sends` falls back to when
/// nothing was chosen.
fn the_entry_decides(body: Result<String, String>) -> Result<(), String> {
    let body = body?;
    it_says(
        Ok(body.clone()),
        "identities::who_sends(data.from.as_ref(),",
    )?;
    match body.contains("active_account_id.clone().ok_or_else") {
        true => Err(format!("it reads the open account first:\n{body}")),
        false => Ok(()),
    }
}

#[test]
fn test_the_send_goes_out_as_the_entry_chosen() {
    the_entry_decides(body_of(&shipped(THE_MAIN_WINDOW), "fn queue_for_sending(")).unwrap();
}

#[test]
fn test_a_draft_is_saved_as_the_entry_chosen() {
    let app = shipped(THE_MAIN_WINDOW);
    the_entry_decides(body_of(&app, "fn save_as_draft(")).unwrap();
    it_says(
        body_of(&app, "fn save_as_draft("),
        "from_address: goes_as.from_address,",
    )
    .unwrap();
    it_says(
        body_of(&app, "fn save_as_draft("),
        "from_name: goes_as.from_name,",
    )
    .unwrap();
}

#[test]
fn test_the_outbox_row_keeps_the_address_and_the_name_of_the_entry() {
    let put = body_of(&shipped(THE_MAIN_WINDOW), "fn put_in_the_outbox(");
    it_says(put.clone(), "from_address: goes_as.from_address,").unwrap();
    it_says(put, "from_name: goes_as.from_name,").unwrap();
}

#[test]
fn test_companion_a_send_from_the_open_account_is_refused() {
    // The shape this plan replaced: the account open in the main window,
    // whatever the From list said.
    let planted = "fn queue_for_sending(\n    let account_id = lock_state(state).active_account_id.clone().ok_or_else(|| {\n        \"Choose an account first\".to_string()\n    })?;\n    put_in_the_outbox(cache, account_id, data)\n}\n";
    assert!(the_entry_decides(body_of(planted, "fn queue_for_sending(")).is_err());
    let both = "fn queue_for_sending(\n    let goes_as = identities::who_sends(data.from.as_ref(), &s.accounts, None);\n    let account_id = lock_state(state).active_account_id.clone().ok_or_else(|| x)?;\n}\n";
    assert!(the_entry_decides(body_of(both, "fn queue_for_sending(")).is_err());
    assert!(the_entry_decides(body_of(planted, "fn gone(")).is_err());
}

#[test]
fn test_the_check_at_send_reads_the_address_chosen() {
    // Sign and Encrypt are checked against the keys held for the address the
    // message goes out from (ledger 660).
    it_says(
        body_of(&shipped(THE_MAIN_WINDOW), "fn the_protection_check("),
        "data.from.as_ref()",
    )
    .unwrap();
}

#[test]
fn test_a_reopened_draft_opens_on_the_address_it_was_written_from() {
    let app = shipped(THE_MAIN_WINDOW);
    it_says(
        body_of(&app, "fn open_compose("),
        "identities::where_the_list_opens(",
    )
    .unwrap();
    it_says(
        body_of(&shipped(THE_MANAGERS), "pub fn open_draft("),
        "from_address: draft.from_address.clone(),",
    )
    .unwrap();
    // A message taken back from the Outbox keeps the address it was queued
    // from, as a draft and when it reopens.
    it_says(
        body_of(&app, "fn the_draft_it_became("),
        "from_address: message.from_address.clone(),",
    )
    .unwrap();
    it_says(
        body_of(&app, "fn a_message_taken_back("),
        "message.from_address",
    )
    .unwrap();
}

// ── Every account's Outbox, read from the source ──────────────────────────
//
// Once the From list chooses the account, a message can wait in the Outbox of
// an account nobody has open. Until 13-35 the send loop, the clock that lets
// held mail go and Undo Send each read the open account's queue alone, so
// such a message would have waited until somebody opened its account, and
// Undo Send would have said nothing was waiting while it went.

/// Whether the send loop reads what may go in every account's Outbox.
fn every_outbox_is_sent(flush: Result<String, String>) -> Result<(), String> {
    let flush = flush?;
    it_says(Ok(flush.clone()), "s.accounts.clone()")?;
    it_says(
        Ok(flush),
        "cache.outbox_messages_that_may_go_now(&account.id, now)",
    )
}

#[test]
fn test_the_send_loop_sends_what_waits_in_every_accounts_outbox() {
    every_outbox_is_sent(body_of(&shipped(THE_MAIN_WINDOW), "fn flush_outbox(")).unwrap();
}

#[test]
fn test_companion_a_send_loop_that_reads_the_open_account_alone_is_refused() {
    let planted = "fn flush_outbox(app: AppHandles<'_>) {\n    let id = s.active_account_id.clone();\n    let queued = match cache.outbox_messages_that_may_go_now(aid, now) {\n}\n";
    assert!(every_outbox_is_sent(body_of(planted, "fn flush_outbox(")).is_err());
}

#[test]
fn test_the_clock_lets_held_mail_go_in_every_account() {
    let app = shipped(THE_MAIN_WINDOW);
    let asked = between(&app, "let held_mail_is_due = {", "if held_mail_is_due {");
    it_says(asked.clone(), "s.accounts").unwrap();
    it_says(
        asked,
        "anything_reached_its_moment(&account.id, since, up_to)",
    )
    .unwrap();
}

#[test]
fn test_undo_send_reads_every_accounts_queue() {
    it_says(
        body_of(&shipped(THE_MAIN_WINDOW), "fn undo_send("),
        "cache.every_queue_with_their_times()",
    )
    .unwrap();
}

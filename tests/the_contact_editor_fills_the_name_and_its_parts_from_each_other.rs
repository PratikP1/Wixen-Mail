//! The contact editor as the tester asked for it in #40, read from the built
//! dialogs rather than from the functions behind them.
//!
//! Four of his five points: a prefix, a middle name and a suffix beside the
//! whole name; the whole name and its parts each filling the other, as a
//! first guess that never writes over what somebody typed; the birthday as a
//! date, with a way to say the year is not known; and an address and a phone
//! number checked, the number against its own country's numbering plan. The
//! fifth, the Favourite box nobody could hear, was fixed in 165fd811 and is
//! read again here over MSAA so the record carries a measurement.
//!
//! Text is put into a field with `set_value`, which sends the same text event
//! a keystroke does, so the fills run through the handlers a person meets.
//! OK is not pressed: its handler shows a message box, which would wait for
//! somebody to close it, so the decision it acts on is read the way
//! `tests/item_form_validation.rs` reads Save's, through the function the
//! handler calls.
//!
//! The windows are built hidden and never shown, so nothing here moves the
//! foreground. One `wxdragon::main` per process serves every test through a
//! `OnceLock` holding a `Result`, the shape
//! `tests/every_spin_control_names_the_field_a_person_types_in.rs` uses, and
//! the MSAA reader is that file's.

#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use wixen_mail::application::phone_numbers::{self, Region};
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::wx_managers::{
    self, ContactEditDialogHandles, ContactEntry, PhoneOk,
};
use wixen_mail::service::this_machine;
use wxdragon::prelude::*;

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const VT_I4: u16 = 3;
const CHILDID_SELF: i64 = 0;

/// What MSAA answers for a check box (oleacc.h).
const ROLE_SYSTEM_CHECKBUTTON: i64 = 0x2c;

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
    fn GetClassNameW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
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

/// The MSAA role and name of a window's client object.
#[derive(Debug, Clone, PartialEq)]
struct Msaa {
    role: i64,
    name: String,
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
        let release: ReleaseFn = std::mem::transmute(vtable_entry(object, VTBL_RELEASE));
        let mut role = Variant::empty();
        let hr_role = get_role(object, Variant::child_self(), &mut role);
        let mut s: *mut u16 = std::ptr::null_mut();
        let name = match get_name(object, Variant::child_self(), &mut s) >= 0 {
            true => take_bstr(s),
            false => String::new(),
        };
        release(object);
        if hr_role < 0 || role.vt != VT_I4 {
            return Err(format!("get_accRole hr=0x{hr_role:x} vt={}", role.vt));
        }
        Ok(Msaa {
            role: role.val & 0xFFFF_FFFF,
            name,
        })
    }
}

/// What UI Automation answers as the name of the element for `hwnd`.
fn uia_name_of(hwnd: isize) -> Result<String, String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
    use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation};
    // SAFETY: COM is initialised on this thread by wxWidgets before the main
    // loop starts; the handle is a live window this file built.
    unsafe {
        let automation: IUIAutomation =
            CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
                .map_err(|why| format!("CUIAutomation: {why}"))?;
        let element = automation
            .ElementFromHandle(HWND(hwnd as *mut c_void))
            .map_err(|why| format!("ElementFromHandle: {why}"))?;
        let name = element
            .CurrentName()
            .map_err(|why| format!("CurrentName: {why}"))?;
        Ok(name.to_string())
    }
}

/// A control as both channels read it, at the handle the keyboard reaches.
#[derive(Debug, Clone)]
struct Heard {
    control: &'static str,
    msaa: Result<Msaa, String>,
    uia: Result<String, String>,
}

fn heard(control: &'static str, hwnd: isize) -> Heard {
    Heard {
        control,
        msaa: msaa_of(hwnd),
        uia: uia_name_of(hwnd),
    }
}

/// The edit a combo box types into, which is where focus lands in it.
fn the_edit_inside(combo: isize) -> isize {
    descendants_of(combo)
        .into_iter()
        .find(|child| class_name(*child) == "Edit")
        .unwrap_or(combo)
}

/// The five name fields as the dialog held them.
#[derive(Debug, Clone, PartialEq)]
struct Parts {
    name: String,
    prefix: String,
    given: String,
    middle: String,
    family: String,
    suffix: String,
}

fn parts_of(editor: &ContactEditDialogHandles) -> Parts {
    Parts {
        name: editor.name_f.get_value(),
        prefix: editor.prefix_f.get_value(),
        given: editor.given_f.get_value(),
        middle: editor.middle_f.get_value(),
        family: editor.family_f.get_value(),
        suffix: editor.suffix_f.get_value(),
    }
}

/// What the Add Phone Number dialog's Country list offers.
#[derive(Debug, Clone)]
struct Countries {
    shown: Vec<String>,
    opened_on: Option<Region>,
}

/// Everything read, inside one `wxdragon::main`.
#[derive(Debug, Clone)]
struct Harvest {
    first_load: Duration,
    a_whole_name: Parts,
    a_titled_name: Parts,
    parts_first: Parts,
    family_first: Parts,
    a_stored_name_changed: Parts,
    a_birthday_with_no_year: String,
    a_stored_birthday_with_no_year: (bool, bool, String),
    no_birthday: (bool, String),
    address_answers: Vec<(String, Result<String, String>)>,
    countries: Countries,
    british_number: PhoneOk,
    italian_number_with_britain_chosen: PhoneOk,
    no_digit: PhoneOk,
    doubted_then_kept: (PhoneOk, PhoneOk),
    changed_between_two_oks: (PhoneOk, PhoneOk),
    heard: Vec<Heard>,
}

fn a_new_editor(frame: &Frame, a11y: &Arc<Accessibility>) -> ContactEditDialogHandles {
    wx_managers::build_contact_edit_dialog(frame, None, None, a11y)
}

fn a_stored_contact() -> ContactEntry {
    ContactEntry {
        id: "stored".to_string(),
        name: "Grace van der Berg".to_string(),
        given_name: "Grace".to_string(),
        family_name: "van der Berg".to_string(),
        name_prefix: String::new(),
        middle_name: String::new(),
        name_suffix: String::new(),
        nickname: String::new(),
        company: String::new(),
        department: String::new(),
        job_title: String::new(),
        emails: Vec::new(),
        phones: Vec::new(),
        addresses: Vec::new(),
        birthday: "--03-14".to_string(),
        website: String::new(),
        relationship: String::new(),
        notes: String::new(),
        custom_fields: Vec::new(),
        avatar_url: String::new(),
        favorite: false,
    }
}

fn choose_country(asker: &wx_managers::PhoneAsker, code: &str) {
    let wanted = Region::from_code(code);
    if let Some(at) = asker.countries.iter().position(|region| *region == wanted) {
        asker.country_choice.set_selection(at as u32);
    }
}

fn a_number_read(parent: &Dialog, typed: &str, country: &str) -> PhoneOk {
    let phone = wx_managers::build_phone_sub_dialog(parent, None);
    phone.asker.number_f.set_value(typed);
    choose_country(&phone.asker, country);
    let decided = phone.asker.decide_on_ok();
    phone.dialog.destroy();
    decided
}

fn read_everything(frame: &Frame, a11y: &Arc<Accessibility>) -> Result<Harvest, String> {
    // Timed first, before anything else asks for the numbering data.
    let started = Instant::now();
    let regions = phone_numbers::every_region();
    let first_load = started.elapsed();
    if regions.is_empty() {
        return Err("the numbering data listed no region".to_string());
    }

    let editor = a_new_editor(frame, a11y);
    editor.name_f.set_value("Grace Brewster Murray Hopper");
    let a_whole_name = parts_of(&editor);
    editor.dialog.destroy();

    let editor = a_new_editor(frame, a11y);
    editor.name_f.set_value("Dr. Jane Q. Public, PhD");
    let a_titled_name = parts_of(&editor);
    editor.dialog.destroy();

    let editor = a_new_editor(frame, a11y);
    editor.given_f.set_value("Ada");
    editor.family_f.set_value("Lovelace");
    let parts_first = parts_of(&editor);
    editor.dialog.destroy();

    let editor = a_new_editor(frame, a11y);
    editor.family_f.set_value("van der Berg");
    editor.name_f.set_value("Anna Maria Berg");
    let family_first = parts_of(&editor);
    editor.dialog.destroy();

    let stored = a_stored_contact();
    let editor = wx_managers::build_contact_edit_dialog(frame, Some(&stored), None, a11y);
    editor.name_f.set_value("Grace Anne van der Berg");
    let a_stored_name_changed = parts_of(&editor);
    let a_stored_birthday_with_no_year = (
        editor.birthday.known.get_value(),
        editor.birthday.no_year.get_value(),
        editor.what_it_holds("stored".to_string()).birthday,
    );
    editor.dialog.destroy();

    let editor = a_new_editor(frame, a11y);
    let no_birthday = (
        editor.birthday.known.get_value(),
        editor.what_it_holds("new".to_string()).birthday,
    );
    editor.birthday.known.set_value(true);
    editor.birthday.date.month.set_selection(2);
    editor.birthday.date.day.set_value(14);
    editor.birthday.no_year.set_value(true);
    let a_birthday_with_no_year = editor.what_it_holds("new".to_string()).birthday;

    // Every new control, read at the handle the keyboard reaches, before
    // this editor goes.
    let mut heard_here = vec![
        heard(
            "the Prefix box's edit",
            the_edit_inside(editor.prefix_f.get_handle() as isize),
        ),
        heard(
            "the Middle name field",
            editor.middle_f.get_handle() as isize,
        ),
        heard(
            "the Suffix box's edit",
            the_edit_inside(editor.suffix_f.get_handle() as isize),
        ),
        heard(
            "the Birthday check box",
            editor.birthday.known.get_handle() as isize,
        ),
        heard(
            "the birthday's month",
            editor.birthday.date.month.get_handle() as isize,
        ),
        heard(
            "the No year check box",
            editor.birthday.no_year.get_handle() as isize,
        ),
        heard(
            "the Favourite check box",
            editor.fav_check.get_handle() as isize,
        ),
    ];
    editor.dialog.destroy();

    let parent = Dialog::builder(frame, "Contact editor stand-in").build();
    let email = wx_managers::build_email_sub_dialog(&parent, None);
    let address_answers = [
        "grace.example.com",
        "grace@example.com",
        " grace@example.com ",
        "   ",
    ]
    .into_iter()
    .map(|typed| {
        email.2.set_value(typed);
        let value = email.2.get_value();
        (typed.to_string(), wx_managers::an_address_to_add(&value))
    })
    .collect();
    email.0.destroy();

    let phone = wx_managers::build_phone_sub_dialog(&parent, None);
    let countries = Countries {
        shown: (0..phone.asker.country_choice.get_count())
            .map(|at| {
                phone
                    .asker
                    .country_choice
                    .get_string(at)
                    .unwrap_or_default()
            })
            .collect(),
        opened_on: phone.asker.chosen_country(),
    };
    heard_here.push(heard(
        "the Country list",
        phone.asker.country_choice.get_handle() as isize,
    ));
    phone.dialog.destroy();

    let british_number = a_number_read(&parent, "0121 234 5678", "GB");
    let italian_number_with_britain_chosen = a_number_read(&parent, "+39 06 3551 1397", "GB");
    let no_digit = a_number_read(&parent, "call reception", "GB");

    let phone = wx_managers::build_phone_sub_dialog(&parent, None);
    phone.asker.number_f.set_value("0121 234");
    choose_country(&phone.asker, "GB");
    let doubted_then_kept = (phone.asker.decide_on_ok(), phone.asker.decide_on_ok());
    phone.dialog.destroy();

    let phone = wx_managers::build_phone_sub_dialog(&parent, None);
    phone.asker.number_f.set_value("0121 234");
    choose_country(&phone.asker, "GB");
    let first = phone.asker.decide_on_ok();
    phone.asker.number_f.set_value("0121 23");
    let changed_between_two_oks = (first, phone.asker.decide_on_ok());
    phone.dialog.destroy();
    parent.destroy();

    Ok(Harvest {
        first_load,
        a_whole_name,
        a_titled_name,
        parts_first,
        family_first,
        a_stored_name_changed,
        a_birthday_with_no_year,
        a_stored_birthday_with_no_year,
        no_birthday,
        address_answers,
        countries,
        british_number,
        italian_number_with_britain_chosen,
        no_digit,
        doubted_then_kept,
        changed_between_two_oks,
        heard: heard_here,
    })
}

fn take_the_harvest() -> Result<Harvest, String> {
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
                read_everything(&frame, &a11y)
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

fn the_harvest() -> &'static Harvest {
    static HARVEST: OnceLock<Result<Harvest, String>> = OnceLock::new();
    match HARVEST.get_or_init(take_the_harvest) {
        Ok(harvest) => harvest,
        Err(why) => panic!("the window session could not be read: {why}"),
    }
}

// ── The checks, apart from what they read, so each can be shown a failure ──

/// A field somebody typed in holds what they typed.
fn the_typed_family_name_stands(parts: &Parts) -> Result<(), String> {
    match parts.family == "van der Berg" {
        true => Ok(()),
        false => Err(format!("the family name typed first became {parts:?}")),
    }
}

/// An address that is not the shape of one is refused, in a sentence naming
/// it; a real one is added without the spaces around it.
fn the_address_check_holds(answers: &[(String, Result<String, String>)]) -> Result<(), String> {
    for (typed, answer) in answers {
        let address = typed.trim();
        match (address.contains('@'), answer) {
            (true, Ok(added)) if added == address => {}
            (false, Err(said)) if address.is_empty() || said.contains(address) => {}
            _ => return Err(format!("{typed:?} was answered {answer:?}")),
        }
    }
    Ok(())
}

/// A doubted number is said once and kept exactly as typed on a second OK
/// with nothing changed.
fn a_doubt_keeps_the_number_on_the_second_ok(
    oks: &(PhoneOk, PhoneOk),
    typed: &str,
) -> Result<(), String> {
    match oks {
        (PhoneOk::Doubt(_), PhoneOk::Add(item)) if item.number == typed => Ok(()),
        other => Err(format!("two OKs on a doubted {typed:?} answered {other:?}")),
    }
}

fn is_named(heard: &Heard, name: &str) -> Result<(), String> {
    let on_msaa = heard.msaa.as_ref().map(|msaa| msaa.name.as_str());
    let on_uia = heard.uia.as_deref();
    // A field's name carries the comma `name_from_label` puts where its
    // label's colon was, so a screen reader pauses before the value.
    let spoken = |said: Result<&str, &String>| {
        said.is_ok_and(|said| said.trim_end_matches([':', ',']) == name)
    };
    match spoken(on_msaa) && spoken(on_uia) {
        true => Ok(()),
        false => Err(format!(
            "{} reads {on_msaa:?} on MSAA and {on_uia:?} on UI Automation, not {name:?}",
            heard.control
        )),
    }
}

fn the_heard(control: &str) -> &'static Heard {
    the_harvest()
        .heard
        .iter()
        .find(|heard| heard.control == control)
        .unwrap_or_else(|| panic!("{control} was not read"))
}

// ── The fills ─────────────────────────────────────────────────────────────

#[test]
fn test_a_whole_name_typed_fills_the_parts_with_both_middle_names() {
    let parts = &the_harvest().a_whole_name;

    assert_eq!(
        (
            parts.given.as_str(),
            parts.middle.as_str(),
            parts.family.as_str()
        ),
        ("Grace", "Brewster Murray", "Hopper"),
        "{parts:?}"
    );
}

#[test]
fn test_a_title_and_a_degree_typed_in_the_name_fill_the_prefix_and_the_suffix() {
    let parts = &the_harvest().a_titled_name;

    assert_eq!(parts.prefix, "Dr.", "{parts:?}");
    assert_eq!(parts.given, "Jane", "{parts:?}");
    assert_eq!(parts.middle, "Q.", "{parts:?}");
    assert_eq!(parts.family, "Public", "{parts:?}");
    assert_eq!(parts.suffix, "PhD", "{parts:?}");
}

#[test]
fn test_parts_filled_with_no_name_compose_the_name() {
    let parts = &the_harvest().parts_first;

    assert_eq!(parts.name, "Ada Lovelace", "{parts:?}");
}

#[test]
fn test_a_family_name_typed_first_is_not_overwritten_by_the_name() {
    let parts = &the_harvest().family_first;

    the_typed_family_name_stands(parts).unwrap_or_else(|why| panic!("{why}"));
    assert_eq!(parts.given, "Anna", "{parts:?}");
    assert_eq!(parts.middle, "Maria", "{parts:?}");
}

#[test]
fn test_a_planted_fill_over_a_typed_family_name_is_seen() {
    let planted = Parts {
        family: "Berg".to_string(),
        ..the_harvest().family_first.clone()
    };

    assert!(the_typed_family_name_stands(&planted).is_err());
}

#[test]
fn test_a_stored_contacts_parts_are_not_guessed_over_when_its_name_changes() {
    let parts = &the_harvest().a_stored_name_changed;

    assert_eq!(parts.given, "Grace", "{parts:?}");
    assert_eq!(parts.family, "van der Berg", "{parts:?}");
    // The one part the contact was stored without is filled from the guess,
    // since nobody's words were in it.
    assert_eq!(parts.middle, "Anne", "{parts:?}");
}

// ── The birthday ──────────────────────────────────────────────────────────

#[test]
fn test_a_birthday_with_no_year_is_saved_as_the_marker_the_sync_writes() {
    assert_eq!(the_harvest().a_birthday_with_no_year, "--03-14");
}

#[test]
fn test_a_stored_birthday_with_no_year_opens_with_no_year_ticked_and_saves_unchanged() {
    let (known, no_year, saved) = &the_harvest().a_stored_birthday_with_no_year;

    assert!(*known && *no_year, "Birthday {known}, No year {no_year}");
    assert_eq!(saved, "--03-14");
}

#[test]
fn test_a_contact_with_no_birthday_saves_none_and_not_today() {
    let (known, saved) = &the_harvest().no_birthday;

    assert!(!known);
    assert_eq!(saved, "");
}

// ── The address ───────────────────────────────────────────────────────────

#[test]
fn test_an_address_that_is_not_the_shape_of_one_is_refused_in_a_sentence_naming_it() {
    // Pull request #99's review on 2026-09-24: an address with spaces around
    // it passed the check and was stored with them.
    the_address_check_holds(&the_harvest().address_answers).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_planted_save_of_a_bad_address_is_seen() {
    let planted = vec![(
        "grace.example.com".to_string(),
        Ok("grace.example.com".to_string()),
    )];

    assert!(the_address_check_holds(&planted).is_err());
}

// ── The number ────────────────────────────────────────────────────────────

#[test]
fn test_the_country_list_offers_no_country_first_then_every_region_by_name_and_code() {
    let countries = &the_harvest().countries;
    let regions = phone_numbers::every_region();
    let britain = format!(
        "{} (+44)",
        this_machine::region_name("GB").unwrap_or_else(|| "GB".to_string())
    );

    assert_eq!(
        countries.shown.first().map(String::as_str),
        Some("No country")
    );
    assert_eq!(countries.shown.len(), regions.len() + 1);
    assert!(
        countries.shown.contains(&britain),
        "{britain} is not offered"
    );
}

#[test]
fn test_the_country_list_opens_on_where_this_person_says_they_are() {
    let home = this_machine::home_region().and_then(|code| Region::from_code(&code));

    assert_eq!(the_harvest().countries.opened_on, home);
}

#[test]
fn test_a_british_number_is_added_in_its_international_form_with_its_country() {
    match &the_harvest().british_number {
        PhoneOk::Add(item) => {
            assert_eq!(item.number, "+44 121 234 5678");
            assert_eq!(item.country.as_deref(), Some("GB"));
        }
        other => panic!("answered {other:?}"),
    }
}

#[test]
fn test_a_number_typed_with_its_code_is_read_as_its_own_countrys() {
    match &the_harvest().italian_number_with_britain_chosen {
        PhoneOk::Add(item) => {
            assert_eq!(item.number, "+39 06 3551 1397");
            assert_eq!(item.country.as_deref(), Some("IT"));
        }
        other => panic!("answered {other:?}"),
    }
}

#[test]
fn test_text_with_no_digit_is_refused() {
    assert_eq!(
        the_harvest().no_digit,
        PhoneOk::Refuse(phone_numbers::no_digit_sentence("call reception"))
    );
}

#[test]
fn test_a_doubted_number_is_said_once_and_kept_exactly_as_typed_on_the_second_ok() {
    let oks = &the_harvest().doubted_then_kept;

    a_doubt_keeps_the_number_on_the_second_ok(oks, "0121 234")
        .unwrap_or_else(|why| panic!("{why}"));
    match oks {
        (PhoneOk::Doubt(said), PhoneOk::Add(item)) => {
            assert!(
                said.contains("0121 234") && said.contains("too short"),
                "{said}"
            );
            assert_eq!(item.country.as_deref(), Some("GB"));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn test_a_planted_refusal_on_the_second_ok_is_seen() {
    let planted = (
        PhoneOk::Doubt("0121 234 is too short.".to_string()),
        PhoneOk::Doubt("0121 234 is too short.".to_string()),
    );

    assert!(a_doubt_keeps_the_number_on_the_second_ok(&planted, "0121 234").is_err());
}

#[test]
fn test_changing_the_number_between_two_oks_asks_again() {
    match &the_harvest().changed_between_two_oks {
        (PhoneOk::Doubt(_), PhoneOk::Doubt(said)) => assert!(said.contains("0121 23"), "{said}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn test_the_numbering_data_loads_in_under_two_seconds() {
    let took = the_harvest().first_load;
    println!(
        "the numbering data's first load took {} ms",
        took.as_millis()
    );

    assert!(took < Duration::from_secs(2), "{took:?}");
}

// ── Heard ─────────────────────────────────────────────────────────────────

#[test]
fn test_every_new_control_is_named_where_the_keyboard_lands_on_both_channels() {
    let wrong: Vec<String> = [
        ("the Prefix box's edit", "Prefix"),
        ("the Middle name field", "Middle name"),
        ("the Suffix box's edit", "Suffix"),
        ("the Birthday check box", "Birthday"),
        ("the birthday's month", "Birthday Month"),
        ("the No year check box", "No year"),
        ("the Country list", "Country"),
    ]
    .into_iter()
    .filter_map(|(control, name)| is_named(the_heard(control), name).err())
    .collect();

    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn test_the_favourite_box_reads_as_a_check_box_named_favorite_over_msaa() {
    let favourite = the_heard("the Favourite check box");

    match &favourite.msaa {
        Ok(msaa) => {
            assert_eq!(msaa.role, ROLE_SYSTEM_CHECKBUTTON, "{msaa:?}");
            assert_eq!(msaa.name, "Favorite", "{msaa:?}");
        }
        Err(why) => panic!("{why}"),
    }
}

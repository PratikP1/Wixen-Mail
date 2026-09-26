//! Accept, Tentative and Decline as buttons where an invitation is said, in the
//! text reader's tab (GAP-04, #50 points 4 and 7, 13-11).
//!
//! One window session builds the real reader window, opens a tab for an
//! invitation that can be answered, one for a cancellation, and reads each tab
//! the way NVDA reads native controls: the children in the order Windows holds
//! them, which is the order Tab walks, and each one over MSAA through
//! `AccessibleObjectFromWindow` at its own handle, its name, description,
//! role and state. The documents are composed the way every surface composes
//! one, through `the_answer_buttons`, `what_the_invitation_says` and the
//! reader's own fold, from the same calendar documents the library's cases use.
//!
//! **Where Alt+C arrives, read from wx's source before anything was bound.**
//! `wxWindowMSW::MSWProcessMessage` (`target/debug/wxWidgets/src/msw/window.cpp:2548-2749`)
//! hands every key message for a window with `wxTAB_TRAVERSAL` and
//! `WS_EX_CONTROLPARENT` to `::IsDialogMessage`, and a `wxPanel` has both. For
//! `WM_SYSCHAR` the dialog manager looks for the letter among the panel's own
//! children whatever control has the keyboard, so Alt+C in the message's text
//! presses the button labelled `A&ccept` with nothing bound on the text. A
//! binding on the text as well would press it twice, because `IsDialogMessage`
//! translates the key before the text control sees it. So the reader binds
//! nothing, and this file posts `WM_SYSKEYDOWN` to the text control through
//! the window's own message loop and reads how many presses arrived.
//!
//! What this proves and what it does not. The buttons are built, named,
//! described, in Tab order, pressed once by the letter and handed the row of
//! the message the tab shows; a cancellation gets none and says why. How they
//! sound is the tester's ear and is on the ledger.

#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::allowed::Allowed;
use wixen_mail::application::answering::the_answer_buttons;
use wixen_mail::application::invitations::{Answer, what_the_invitation_says};
use wixen_mail::application::reading_a_message::WhatIsSaidAboutIt;
use wixen_mail::common::types::MessageBody;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::date_display::{
    Clock, DateOrder, DateSettings, DateStyle, DateWording,
};
use wixen_mail::presentation::read_aloud::Reading;
use wixen_mail::presentation::reader_text::{self, ConversationPart, ReaderDocument};
use wixen_mail::presentation::ui_types::MessageItem;
use wixen_mail::presentation::wx_app;
use wixen_mail::presentation::wx_reader::ReaderWindow;
use wxdragon::prelude::*;

const OBJID_CLIENT: u32 = 0xFFFF_FFFC;
const VT_I4: u16 = 3;
const CHILDID_SELF: i64 = 0;

/// MSAA roles and states (oleacc.h).
const ROLE_SYSTEM_PUSHBUTTON: i64 = 0x2b;
const STATE_SYSTEM_UNAVAILABLE: i64 = 0x1;

const WM_SYSKEYDOWN: u32 = 0x0104;
const WM_SYSKEYUP: u32 = 0x0105;
const VK_C: usize = 0x43;
/// The context code, bit 29, which is what makes a key an Alt chord.
const ALT_HELD: isize = 0x2000_0001;
/// The same with the previous-state and transition bits a release carries.
const ALT_RELEASED: isize = 0xE000_0001_u32 as i32 as isize;

const TICK_MS: i32 = 30;
/// Ticks for the loop to translate the key, hand it to the dialog manager and
/// run the button's handler.
const TICKS_FOR_THE_KEY: u32 = 10;

/// The row the invitation's tab is opened for, which a press has to carry.
const THE_INVITATIONS_ROW: i64 = 42;

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
const VTBL_GET_ACC_DESCRIPTION: usize = 12;
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
    fn PostMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> i32;
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
    fn SetFocus(hwnd: isize) -> isize;
    fn GetFocus() -> isize;
    fn GetDlgCtrlID(hwnd: isize) -> i32;
    fn EnumThreadWindows(
        thread: u32,
        callback: extern "system" fn(isize, isize) -> i32,
        lparam: isize,
    ) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentThreadId() -> u32;
}

/// What a button sends its parent when it is clicked, sent to the parent
/// directly: `BM_CLICK` is refused by a window that is not in front, and a
/// window a test builds is not.
const WM_COMMAND: u32 = 0x0111;
const BN_CLICKED: usize = 0;

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

/// Every top-level window the calling thread owns.
fn this_threads_windows() -> Vec<isize> {
    FOUND.with(|found| found.borrow_mut().clear());
    // SAFETY: the callback only pushes to this thread's local.
    unsafe { EnumThreadWindows(GetCurrentThreadId(), collect, 0) };
    FOUND.with(|found| found.borrow().clone())
}

/// Press a button the way a click does: its command, sent to its parent.
fn press(parent: isize, button: isize) {
    // SAFETY: two live windows of this thread; the command is what the
    // button sends on a click, so the handler that runs is the click's.
    unsafe {
        let id = GetDlgCtrlID(button) as u16 as usize;
        SendMessageW(parent, WM_COMMAND, id | (BN_CLICKED << 16), button);
    }
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
    description: String,
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
        let get_description: GetBstrFn =
            std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_DESCRIPTION));
        let get_role: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_ROLE));
        let get_state: GetVariantFn = std::mem::transmute(vtable_entry(object, VTBL_GET_ACC_STATE));
        let text = |getter: GetBstrFn| {
            let mut s: *mut u16 = std::ptr::null_mut();
            match getter(object, Variant::child(CHILDID_SELF), &mut s) >= 0 {
                true => take_bstr(s),
                false => String::new(),
            }
        };
        let name = text(get_name);
        let description = text(get_description);
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
            name,
            description,
            role: read(hr_role, role),
            state: read(hr_state, state),
        })
    }
}

/// One control of a tab, as Windows holds it.
#[derive(Debug, Clone, PartialEq)]
struct Control {
    hwnd: isize,
    class: String,
    text: String,
    msaa: Msaa,
}

fn read_the_controls(parent: isize) -> Result<Vec<Control>, String> {
    descendants_of(parent)
        .into_iter()
        .map(|hwnd| {
            Ok(Control {
                hwnd,
                class: class_name(hwnd),
                text: window_text(hwnd),
                msaa: msaa_of(hwnd)?,
            })
        })
        .collect()
}

// ── The documents ─────────────────────────────────────────────────────────

/// An invitation to Sam from Ada, version 2, the calendar holding nothing.
const AN_INVITATION: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nMETHOD:REQUEST\r\n\
    BEGIN:VEVENT\r\nUID:m-1@example.com\r\nSEQUENCE:2\r\nSUMMARY:Quarterly review\r\n\
    LOCATION:Room 3\r\nDTSTART:20260305T090000\r\nDTEND:20260305T100000\r\n\
    ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n\
    ATTENDEE;CN=Sam;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:sam@example.com\r\n\
    END:VEVENT\r\nEND:VCALENDAR\r\n";

fn a_cancellation() -> String {
    AN_INVITATION.replace("METHOD:REQUEST", "METHOD:CANCEL")
}

fn written_out_in_full() -> DateSettings {
    DateSettings {
        style: DateStyle::Absolute,
        order: DateOrder::DayFirst,
        wording: DateWording::Numeric,
        clock: Clock::TwentyFourHour,
    }
}

/// A message carrying `document`, composed the way the text reader composes
/// one: what is said about it decided by the application and folded by the
/// reader.
fn a_message_carrying(document: &str, row: i64) -> ReaderDocument {
    let message = MessageItem {
        message_id: row,
        subject: "Invitation: Quarterly review".to_string(),
        from: "Ada Lovelace <ada@example.com>".to_string(),
        ..Default::default()
    };
    reader_text::single_message(
        &message,
        &MessageBody::Plain("Are you free?".to_string()),
        Reading {
            dates: written_out_in_full(),
            now: chrono::Local::now(),
        },
    )
    .with_what_is_said(&said_about(document, row))
}

/// What is said about a message carrying `document`, asked the way every
/// surface asks it, with Sam answering and sending switched on.
fn said_about(document: &str, row: i64) -> WhatIsSaidAboutIt {
    WhatIsSaidAboutIt {
        invitation: what_the_invitation_says(document, None, None, written_out_in_full()),
        answering: the_answer_buttons(
            document,
            "sam@example.com",
            Allowed::EVERYTHING,
            row,
            None,
            written_out_in_full(),
        ),
        ..WhatIsSaidAboutIt::nothing()
    }
}

/// One message of a conversation the formatted window opens on.
fn a_part_carrying(document: &str, row: i64, depth: usize) -> ConversationPart {
    ConversationPart {
        message: MessageItem {
            message_id: row,
            subject: "Invitation: Quarterly review".to_string(),
            from: "Ada Lovelace <ada@example.com>".to_string(),
            ..Default::default()
        },
        body: MessageBody::Plain("Are you free?".to_string()),
        said: said_about(document, row),
        depth,
    }
}

/// The row the formatted window's one message is, which a press has to carry.
const THE_FORMATTED_ROW: i64 = 44;

/// The formatted window `show_conversation_as_page` builds for `parts`, found
/// by the title it gives itself, and its controls in the order Windows holds
/// them.
fn the_formatted_window(
    parent: &Frame,
    reader: &Rc<ReaderWindow>,
    a11y: &Arc<Accessibility>,
    subject: &str,
    parts: &[ConversationPart],
) -> Result<(isize, Vec<Control>), String> {
    wx_app::show_conversation_as_page(parent, reader, a11y, subject, parts, None);
    let title = format!("{subject} - headings - Wixen Mail");
    let window = this_threads_windows()
        .into_iter()
        .find(|hwnd| window_text(*hwnd) == title)
        .ok_or(format!("no window titled {title:?} was opened"))?;
    Ok((window, read_the_controls(window)?))
}

// ── The harvest ───────────────────────────────────────────────────────────

#[derive(Debug, Default)]
struct Harvest {
    /// The invitation's tab, its controls in the order Windows holds them.
    invitation_tab: Vec<Control>,
    /// The same tab with one button disabled, for the companion.
    with_a_greyed_button: Vec<Control>,
    /// What the answer handler was handed after Alt+C.
    pressed_by_alt_c: Vec<(i64, Answer)>,
    /// Which control had the keyboard after Alt+C.
    focus_after_alt_c: isize,
    /// The message's text, where the keyboard was when Alt+C was pressed.
    the_message_text: isize,
    /// The cancellation's tab.
    cancellation_tab: Vec<Control>,
    /// The formatted window opened on the invitation alone.
    formatted_window: Vec<Control>,
    /// What the answer handler was handed when its Decline was pressed.
    pressed_in_the_formatted_window: Vec<(i64, Answer)>,
    /// The formatted window opened on a conversation of two messages that
    /// each carry the invitation.
    two_messages: Vec<Control>,
}

/// What the session read, held while it waits for the formatted windows'
/// browsers to exist.
struct Gathered {
    harvest: Result<Harvest, String>,
    page_windows: Vec<isize>,
}

/// The classes WebView2 gives the windows it puts under a host control.
const CHROMIUMS_CLASSES: [&str; 2] = ["Chrome_WidgetWin_1", "Chrome_RenderWidgetHostHWND"];

/// Whether the browser under a formatted window's host control exists yet.
fn the_browser_is_there(page_window: isize) -> bool {
    descendants_of(page_window)
        .into_iter()
        .any(|hwnd| CHROMIUMS_CLASSES.contains(&class_name(hwnd).as_str()))
}

/// Ticks before the session stops waiting for a browser: half a minute,
/// which is generous. GitHub's runners have taken over three seconds to make
/// one.
const GIVE_UP_AFTER_TICKS: u32 = 1000;

fn take_the_harvest() -> Result<Harvest, String> {
    if std::mem::size_of::<Variant>() != 24 {
        return Err("VARIANT is not 24 bytes here, so the reader's layout is wrong".to_string());
    }
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let settle = move |taken: Result<Harvest, String>| {
                if let Ok(mut slot) = outcome.lock() {
                    *slot = Some(taken);
                }
                wxdragon::call_after(Box::new(move || app.exit_main_loop()));
            };
            let a11y = match Accessibility::new() {
                Ok(a11y) => Arc::new(a11y),
                Err(why) => {
                    settle(Err(format!("no accessibility layer: {why}")));
                    return;
                }
            };
            let parent = Frame::builder().build();
            let reader = Rc::new(ReaderWindow::new(&parent, &a11y));
            reader.wire_menu();
            let pressed: Rc<RefCell<Vec<(i64, Answer)>>> = Rc::default();
            reader.on_answer({
                let pressed = pressed.clone();
                move |row, answer| pressed.borrow_mut().push((row, answer))
            });

            let tab = reader.open(a_message_carrying(AN_INVITATION, THE_INVITATIONS_ROW));
            let panel = tab.panel.get_handle() as isize;
            let invitation_tab = match read_the_controls(panel) {
                Ok(read) => read,
                Err(why) => {
                    settle(Err(why));
                    return;
                }
            };
            let text = tab.text.get_handle() as isize;
            // SAFETY: a live window of this thread; the key goes through the
            // window's own loop, which is what a real key does.
            unsafe {
                SetFocus(text);
                PostMessageW(text, WM_SYSKEYDOWN, VK_C, ALT_HELD);
                PostMessageW(text, WM_SYSKEYUP, VK_C, ALT_RELEASED);
            }

            // The key's time to arrive, then everything is read, then the
            // session waits for each formatted window's browser to exist
            // before it ends: a browser still being made when the loop stops
            // takes the process down with it, which the hook's run met here as
            // 0xc000041d (tests/closing_a_window_before_its_browser_exists.rs).
            let ticks = Rc::new(std::cell::Cell::new(0u32));
            let gathered: Rc<RefCell<Option<Gathered>>> = Rc::default();
            let ticker = Rc::new(Timer::new(&parent));
            ticker.on_tick({
                let ticker = ticker.clone();
                let reader = reader.clone();
                move |_| {
                    ticks.set(ticks.get() + 1);
                    if ticks.get() < TICKS_FOR_THE_KEY {
                        return;
                    }
                    let waiting = match gathered.borrow().as_ref() {
                        Some(already) => already
                            .page_windows
                            .iter()
                            .any(|window| !the_browser_is_there(*window)),
                        None => false,
                    };
                    if gathered.borrow().is_some() {
                        if waiting && ticks.get() < GIVE_UP_AFTER_TICKS {
                            return;
                        }
                        ticker.stop();
                        if let Some(done) = gathered.borrow_mut().take() {
                            settle(done.harvest);
                        }
                        return;
                    }
                    // SAFETY: asks this thread's own queue.
                    let focus_after_alt_c = unsafe { GetFocus() };
                    let pressed_by_alt_c = pressed.borrow().clone();

                    let greyed = tab.answer_buttons.first().copied();
                    if let Some(button) = greyed {
                        button.enable(false);
                    }
                    let with_a_greyed_button = read_the_controls(panel);
                    if let Some(button) = greyed {
                        button.enable(true);
                    }

                    let cancelled = reader.open(a_message_carrying(&a_cancellation(), 43));
                    let cancellation_tab = read_the_controls(cancelled.panel.get_handle() as isize);

                    // The formatted window, which is how a message opens by
                    // default: the invitation alone, its Decline pressed the
                    // way a click presses it, and a conversation of two.
                    let formatted = the_formatted_window(
                        &parent,
                        &reader,
                        &a11y,
                        "Invitation to answer",
                        &[a_part_carrying(AN_INVITATION, THE_FORMATTED_ROW, 0)],
                    );
                    let before = pressed.borrow().len();
                    if let Ok((window, controls)) = &formatted
                        && let Some(decline) = the_buttons(controls).get(2)
                    {
                        press(*window, decline.hwnd);
                    }
                    let pressed_in_the_formatted_window = pressed.borrow()[before..].to_vec();
                    let two = the_formatted_window(
                        &parent,
                        &reader,
                        &a11y,
                        "Two messages about a meeting",
                        &[
                            a_part_carrying(AN_INVITATION, 46, 0),
                            a_part_carrying(AN_INVITATION, 47, 1),
                        ],
                    );

                    let page_windows: Vec<isize> = [&formatted, &two]
                        .into_iter()
                        .filter_map(|opened| opened.as_ref().ok().map(|(window, _)| *window))
                        .collect();
                    let harvest = (|| {
                        Ok(Harvest {
                            invitation_tab: invitation_tab.clone(),
                            with_a_greyed_button: with_a_greyed_button?,
                            pressed_by_alt_c,
                            focus_after_alt_c,
                            the_message_text: text,
                            cancellation_tab: cancellation_tab?,
                            formatted_window: formatted?.1,
                            pressed_in_the_formatted_window,
                            two_messages: two?.1,
                        })
                    })();
                    *gathered.borrow_mut() = Some(Gathered {
                        harvest,
                        page_windows,
                    });
                }
            });
            ticker.start(TICK_MS, false);
            std::mem::forget(ticker);
            std::mem::forget(reader);
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

// ── The readings ──────────────────────────────────────────────────────────

fn the_buttons(controls: &[Control]) -> Vec<&Control> {
    controls
        .iter()
        .filter(|control| control.class == "Button")
        .collect()
}

/// What is wrong with a tab's buttons: three, in the order Accept, Tentative,
/// Decline, each a push button named for its answer, described by the sentence
/// saying what pressing it will do, and none greyed.
fn what_is_wrong_with_the_buttons(controls: &[Control]) -> Vec<String> {
    let buttons = the_buttons(controls);
    let mut wrong = Vec::new();
    let names: Vec<&str> = buttons
        .iter()
        .map(|button| button.msaa.name.as_str())
        .collect();
    if names != ["Accept", "Tentative", "Decline"] {
        wrong.push(format!(
            "the buttons are {names:?}, not Accept, Tentative, Decline"
        ));
    }
    for (button, (label, opens)) in buttons.iter().zip([
        ("A&ccept", "Accept Quarterly review"),
        ("&Tentative", "Say you might come to Quarterly review"),
        ("&Decline", "Decline Quarterly review"),
    ]) {
        if button.text != label {
            wrong.push(format!(
                "{:?} is labelled {:?}, not {label:?}",
                button.msaa.name, button.text
            ));
        }
        if button.msaa.role != ROLE_SYSTEM_PUSHBUTTON {
            wrong.push(format!(
                "{:?} is not read as a push button",
                button.msaa.name
            ));
        }
        if !button.msaa.description.starts_with(opens) || !button.msaa.description.contains("Ada") {
            wrong.push(format!(
                "{:?} is described as {:?}, which does not say what pressing it will do",
                button.msaa.name, button.msaa.description
            ));
        }
        if button.msaa.state & STATE_SYSTEM_UNAVAILABLE != 0 {
            wrong.push(format!(
                "{:?} is greyed, and Tab passes a greyed button by with its reason",
                button.msaa.name
            ));
        }
    }
    wrong
}

/// Where the buttons are: after the warning bar and before the message.
fn what_is_wrong_with_the_order(controls: &[Control]) -> Vec<String> {
    let at = |wanted: &dyn Fn(&Control) -> bool| controls.iter().position(wanted);
    let bar = at(&|control| control.msaa.name == "Security warning");
    let first_button = at(&|control| control.class == "Button");
    let message = at(&|control| control.class.starts_with("RICHEDIT"));
    match (bar, first_button, message) {
        (Some(bar), Some(button), Some(message)) if bar < button && button < message => Vec::new(),
        found => vec![format!(
            "the bar, the first button and the message are at {found:?} among {:?}",
            controls
                .iter()
                .map(|control| (&control.class, &control.msaa.name))
                .collect::<Vec<_>>()
        )],
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
fn test_an_answerable_invitation_has_three_named_described_buttons() {
    complain(
        "on the invitation's buttons",
        &what_is_wrong_with_the_buttons(&the_harvest().invitation_tab),
    );
}

#[test]
fn test_the_buttons_come_after_the_bar_and_before_the_message() {
    complain(
        "on where the buttons are",
        &what_is_wrong_with_the_order(&the_harvest().invitation_tab),
    );
}

#[test]
fn test_alt_c_in_the_message_presses_accept_once_for_the_message_the_tab_shows() {
    let harvest = the_harvest();
    assert_eq!(
        harvest.pressed_by_alt_c,
        vec![(THE_INVITATIONS_ROW, Answer::Accepted)],
        "Alt+C in the message's text did not press Accept exactly once for row {THE_INVITATIONS_ROW}"
    );
    // Where the keyboard is afterwards is the dialog manager's to decide. Read
    // here, in a window a test builds and which is not in front, it stays in
    // the message; whether a window in front moves it to the button is on the
    // ledger for the tester. What must never happen is the keyboard going
    // somewhere else in the tab, or nowhere.
    let accept = the_buttons(&harvest.invitation_tab)
        .first()
        .map(|button| button.hwnd)
        .expect("the invitation's tab has an Accept button");
    let landed_on = match harvest.focus_after_alt_c {
        at if at == accept => "the Accept button",
        at if at == harvest.the_message_text => "the message, where it was",
        _ => "",
    };
    println!("after Alt+C the keyboard is on: {landed_on}");
    assert!(
        !landed_on.is_empty(),
        "after Alt+C the keyboard is on neither the button pressed nor the message: {} ({})",
        harvest.focus_after_alt_c,
        class_name(harvest.focus_after_alt_c)
    );
}

#[test]
fn test_a_cancellation_has_no_buttons_and_its_bar_says_why() {
    let harvest = the_harvest();
    assert!(
        the_buttons(&harvest.cancellation_tab).is_empty(),
        "a cancellation was offered buttons: {:?}",
        harvest.cancellation_tab
    );
    let bar = harvest
        .cancellation_tab
        .iter()
        .find(|control| control.msaa.name == "Security warning")
        .expect("a cancellation's tab has a bar saying what it is");
    assert!(
        bar.text.starts_with("Meeting cancelled: Quarterly review."),
        "{:?}",
        bar.text
    );
}

#[test]
fn test_the_reading_refuses_a_greyed_button() {
    let wrong = what_is_wrong_with_the_buttons(&the_harvest().with_a_greyed_button);
    assert!(
        wrong.iter().any(|why| why.contains("is greyed")),
        "a greyed button was passed over: {wrong:?}"
    );
}

// ── The formatted window ──────────────────────────────────────────────────

/// Where the buttons are in the formatted window: after the browser, which
/// has to stay the first control that takes the keyboard, so the buttons are
/// reached by Tab after the message and by their keys from inside it.
fn what_is_wrong_with_the_formatted_order(controls: &[Control]) -> Vec<String> {
    let the_browser = controls
        .iter()
        .position(|control| !["Edit", "Button", "ListBox"].contains(&control.class.as_str()));
    let first_button = controls
        .iter()
        .position(|control| control.class == "Button");
    match (the_browser, first_button) {
        (Some(browser), Some(button)) if browser < button => Vec::new(),
        found => vec![format!(
            "the browser and the first button are at {found:?} among {:?}",
            controls
                .iter()
                .map(|control| (&control.class, &control.msaa.name))
                .collect::<Vec<_>>()
        )],
    }
}

#[test]
fn test_the_formatted_window_has_the_three_buttons_after_the_page() {
    let harvest = the_harvest();
    complain(
        "on the formatted window's buttons",
        &what_is_wrong_with_the_buttons(&harvest.formatted_window),
    );
    complain(
        "on where the formatted window's buttons are",
        &what_is_wrong_with_the_formatted_order(&harvest.formatted_window),
    );
}

#[test]
fn test_a_press_in_the_formatted_window_answers_the_message_it_shows() {
    assert_eq!(
        the_harvest().pressed_in_the_formatted_window,
        vec![(THE_FORMATTED_ROW, Answer::Declined)],
        "Decline in the formatted window did not answer its own message once"
    );
}

#[test]
fn test_a_conversation_of_two_messages_has_no_buttons() {
    // One row of buttons over two messages would be heard as answering both.
    let harvest = the_harvest();
    assert!(
        the_buttons(&harvest.two_messages).is_empty(),
        "a conversation of two was offered buttons: {:?}",
        harvest.two_messages
    );
}

// ── The page's keys, read from the source ─────────────────────────────────
//
// The route is a script injected into a browser control, and a key typed into
// the browser comes from another process, so what a reading can hold is the
// shape: the page's jump for an answer reaches the same handler the buttons
// do, with the window's own message, and says so when there is none.

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

fn shipped(path: &str) -> String {
    wixen_mail::common::what_ships::what_ships(
        &std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("{path}: {e}"))
            .replace("\r\n", "\n"),
    )
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

fn the_page_answers_through_the_buttons_handler(app: &str) -> Result<(), String> {
    let body = body_of(app, "pub fn show_conversation_as_page(")?;
    const THE_ARM: &str = "Some(page_jumps::Jump::Answer(answer)) =>";
    let at = body
        .find(THE_ARM)
        .ok_or("the page window has no arm for the page's answer keys")?;
    let arm = &body[at..];
    let arm = &arm[..arm.find("\n                None =>").unwrap_or(arm.len())];
    if !arm.contains("reader.answer_now(") {
        return Err(
            "the page's answer keys do not reach the handler the buttons press, so Alt+C in \
             the message answers nothing"
                .to_string(),
        );
    }
    if !arm.contains("\"There is no invitation here to answer.\"") {
        return Err(
            "the page's answer keys say nothing on a message with no invitation to answer"
                .to_string(),
        );
    }
    Ok(())
}

#[test]
fn test_the_pages_answer_keys_press_the_buttons_handler_with_the_windows_message() {
    the_page_answers_through_the_buttons_handler(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_reading_complains_when_the_answer_keys_reach_nothing() {
    // Planted inside the arm itself, since the window's buttons reach the
    // same handler and a plant there would leave the arm as it was.
    let app = shipped(THE_MAIN_WINDOW);
    let body = body_of(&app, "pub fn show_conversation_as_page(").expect("the page window");
    let at = body
        .find("Some(page_jumps::Jump::Answer(answer)) =>")
        .expect("the companion's anchor, the page's answer arm, is not in the page window");
    let (before, the_arm_onwards) = body.split_at(at);
    let planted_body = format!(
        "{before}{}",
        the_arm_onwards.replacen("reader.answer_now(", "drop((", 1)
    );
    assert_ne!(planted_body, body, "the companion planted nothing");
    let planted = app.replacen(&body, &planted_body, 1);
    let why = the_page_answers_through_the_buttons_handler(&planted)
        .expect_err("answer keys reaching nothing were passed over");
    assert!(why.contains("answers nothing"), "{why}");
}

// ── What answering says, said once ────────────────────────────────────────
//
// Ledger 155 asked whether accepting was heard twice. It was: every outcome
// went to the status line, which speaks what it shows, and was announced
// beside it as well. Read from the source because the path sends mail from an
// account and files a meeting, neither of which this file starts.

fn answering_says_each_outcome_once(app: &str) -> Result<(), String> {
    let body = body_of(app, "fn answer_the_invitation(")?;
    if body.contains(".announce(") {
        return Err(
            "answer_the_invitation announces an outcome beside the status line, which speaks \
             it too, so it is heard twice (ledger 155)"
                .to_string(),
        );
    }
    for (channel, what) in [
        ("send_status(", "what answering did"),
        ("send_refusal(", "why it could not answer"),
    ] {
        if !body.contains(channel) {
            return Err(format!(
                "answer_the_invitation no longer says {what} through {channel}"
            ));
        }
    }
    Ok(())
}

#[test]
fn test_answering_says_each_outcome_once() {
    answering_says_each_outcome_once(&shipped(THE_MAIN_WINDOW))
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_reading_complains_when_an_outcome_is_also_announced() {
    let app = shipped(THE_MAIN_WINDOW);
    let body = body_of(&app, "fn answer_the_invitation(").expect("the answer path");
    let planted_body = body.replacen(
        "send_status(",
        "let _ = a11y.announce(\"Accepted\", Priority::Normal);\n    send_status(",
        1,
    );
    assert_ne!(planted_body, body, "the companion planted nothing");
    let why = answering_says_each_outcome_once(&app.replacen(&body, &planted_body, 1))
        .expect_err("an outcome said twice was passed over");
    assert!(why.contains("heard twice"), "{why}");
}

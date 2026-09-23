//! When the formatted message window is activated, does the keyboard land in
//! the message, and does it land there again when the window comes back?
//!
//! The question comes from NVDA run 35520201976 on `main` at `0ad66e48`,
//! where `nvda-tests/tests/a-link-opens-where-the-setting-says.test.js`
//! failed. That run's record settles the first half of the case: after Enter
//! on the sender's link the browser opened Example Domain and the page window
//! kept its title and its message, so the route 11-11.1 built held. It does
//! not settle the second half. The case put the page window back with
//! `WScript.Shell.AppActivate`, kept that call's answer (`true`) as its only
//! evidence, pressed `K` for the second link and heard one empty phrase. Two
//! causes fit and the record cannot tell them apart: the window never came
//! back to the front, so `K` went to the browser; or it came back and the
//! document did not take the keyboard, so `K` was not a next-link key.
//!
//! This file settles the second of those. The first belongs to the case
//! itself, which now waits for Windows to say the window is in front and
//! writes down what has focus before it presses anything.
//!
//! **The chain being read.** wxWidgets answers `WM_ACTIVATE` in
//! `wxTopLevelWindowMSW::OnActivate`
//! (`target/debug/wxWidgets/src/msw/toplevel.cpp:1325-1361`): activating,
//! it calls `DoRestoreLastFocus` unless a descendant already has the
//! keyboard, and `DoRestoreLastFocus` calls `wxSetFocusToChild` on the child
//! it saved when the window was deactivated, or on the window itself, which
//! picks the first child that takes focus. For this window that child is the
//! WebView, and `wxWebViewEdge::OnSetFocus`
//! (`src/msw/webview_edge.cpp:1171-1175`) calls `MoveFocus(PROGRAMMATIC)` on
//! the controller, which is what puts the keyboard inside the document rather
//! than on the host window. Three links, and nothing in this tree had ever
//! read the end of them.
//!
//! **Why the message and not a real activation.** A window is activated by
//! the system when the foreground moves to it, and no process here can move
//! the foreground: the desktop this runs on has no foreground window at all,
//! so `GetForegroundWindow` answers 0 and `SetForegroundWindow` does nothing,
//! for this test and for any other process started the same way. Taking the
//! foreground on a machine somebody is using would be worse than not
//! measuring. On 2026-09-23 the first of those two reasons no longer held:
//! `GetForegroundWindow` answered 395246 when 12-03.1 was planned and 264618
//! when it was carried out, with the tester using the machine and NVDA
//! running. So the foreground is not moved here because doing so would take
//! the tester's screen, not because it cannot be done. The reading does what
//! `tests/a_settings_page_reached_from_inside_a_page_gives_focus_to_its_first_control.rs`
//! does for a key it cannot send: it sets up the exact precondition and calls
//! the same entry point the system calls. `WM_ACTIVATE` sent to the window
//! runs `wxWindowMSW::MSWWindowProc`'s own arm
//! (`src/msw/window.cpp:3120-3128`) and everything after it is wx's code and
//! this window's. What is not read here is whether Windows delivers that
//! message, which it does for every window it activates, and which the run on
//! the runner exercises for real. NVDA runs 35839692317 and 35839954840 of
//! 2026-09-23 did exercise it, and both ended with the keyboard on the page
//! window's frame, which is what the frame-saved steps below are for.
//!
//! **The frame-saved steps, added 2026-09-23 by 12-03.1.** wx saves the focused
//! child when a window is deactivated, and `IsDescendant` is true for the
//! frame itself, so a frame holding the keyboard at that moment is saved as
//! its own last focused child and given the keyboard back on every activation
//! after it (`msw/toplevel.cpp:1301-1324`, `common/containr.cpp:611-661`). That
//! is the one path in wx's code that ends with the keyboard on the frame. The
//! steps take it: the keyboard put on the frame and the window deactivated in
//! the same step, so nothing runs between the two, then the keyboard cleared
//! and the window activated. On the page window `presentation::page_focus`
//! then gives the keyboard to the page; on the companion, which has no such
//! code, it stays on the frame, which is the end state the runner recorded.
//! Whether the runner took this path is the NVDA case's record to say, not
//! this file's. The page window is built with a way back that counts its
//! calls, so a timer sharing the frame with the window's own way back shows up
//! here as a call nobody asked for.
//!
//! **Read, never inferred.** `GetFocus()` answers about the calling thread's
//! own queue, and every window here is built on the run's thread, so it
//! answers about these windows. The foreground is read beside it at every
//! step and kept, never asserted on: an activation call answers that it
//! asked, not that Windows agreed, and that confusion is the whole of the run
//! this file exists for. Every step's reading is printed on a failure, so a
//! red here says which step went wrong.
//!
//! **The companion.** The same steps over a window whose one control is a
//! `TextCtrl`, asserting the class is `Edit`. Without it a reading that
//! always found Chromium, or never found anything at all, would read the same
//! as one that measured the chain.
//!
//! **One window session.** The budget is one `wxdragon::main` per process
//! (`tests/theme_reach.rs` records the hang a second one makes), so both
//! readings are steps of one run, every value is harvested into plain data,
//! and the two tests assert over the harvest. The initialiser stores a
//! `Result` and panics on nothing itself, so a failure leaves a message
//! rather than an empty cell the next test would spend the budget filling.
//!
//! Runs under `WIXEN_NO_AUDIO` as CI does, and on a temporary data directory
//! in `WIXEN_MAIL_DATA`, never a person's profile: this window reads the
//! stored palette.

#![cfg(windows)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::reading_a_message::WhatIsSaidAboutIt;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::reader_text::ConversationPart;
use wixen_mail::presentation::scan_fixtures;
use wixen_mail::presentation::wx_app;
use wixen_mail::presentation::wx_reader;
use wxdragon::prelude::*;

/// The title `show_conversation_as_page` builds from the fixture's subject,
/// which is the title the NVDA case looks for too.
const THE_PAGE_WINDOW: &str = "Scan target - headings - Wixen Mail";

/// The companion's window, whose one control is not a browser.
const THE_NATIVE_WINDOW: &str = "A window with a text control in it";

/// The classes WebView2 gives the windows it puts under a host control. The
/// outer one hosts, the inner one takes the keys; which of the two holds the
/// keyboard is the browser's business and either answers this question, so
/// the reading names both and the summary says which it saw.
const CHROMIUMS_CLASSES: [&str; 2] = ["Chrome_WidgetWin_1", "Chrome_RenderWidgetHostHWND"];

/// What Windows calls a `wxTextCtrl`.
const A_TEXT_CONTROLS_CLASS: &str = "Edit";

/// What every step of the run prints on its way out, passing or failing, so a
/// green run leaves the classes it saw on the record rather than only the
/// word "ok". A measurement nobody can read afterwards is not a measurement.
const THE_READINGS_LINE: &str = "what the page window did with the keyboard:";

const WM_ACTIVATE: u32 = 0x0006;
const WA_INACTIVE: usize = 0;
const WA_ACTIVE: usize = 1;

const TICK_MS: i32 = 30;
/// Ticks of nothing after an act, so the window has finished answering it
/// before anything is read.
const TICKS_TO_SETTLE: u32 = 8;
/// Ticks before the run gives up waiting for the browser: half a minute,
/// which is generous. GitHub's runners have taken over three seconds to make
/// one (`tests/closing_a_window_before_its_browser_exists.rs`).
const GIVE_UP_AFTER_TICKS: u32 = 1000;

#[link(name = "user32")]
unsafe extern "system" {
    fn GetFocus() -> isize;
    fn SetFocus(hwnd: isize) -> isize;
    fn GetForegroundWindow() -> isize;
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
    fn IsChild(parent: isize, child: isize) -> i32;
    fn GetClassNameW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
    fn GetWindowTextW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
    fn EnumThreadWindows(
        thread: u32,
        callback: extern "system" fn(isize, isize) -> i32,
        lparam: isize,
    ) -> i32;
    fn EnumChildWindows(
        parent: isize,
        callback: extern "system" fn(isize, isize) -> i32,
        lparam: isize,
    ) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentThreadId() -> u32;
}

thread_local! {
    static FOUND: RefCell<Vec<isize>> = const { RefCell::new(Vec::new()) };
}

extern "system" fn collect(hwnd: isize, _lparam: isize) -> i32 {
    FOUND.with(|found| found.borrow_mut().push(hwnd));
    1
}

/// Every top-level window the calling thread owns, in the order Windows
/// enumerates them.
fn this_threads_windows() -> Vec<isize> {
    FOUND.with(|found| found.borrow_mut().clear());
    // SAFETY: the callback only pushes to this thread's local.
    unsafe { EnumThreadWindows(GetCurrentThreadId(), collect, 0) };
    FOUND.with(|found| found.borrow().clone())
}

/// Everything under `parent`, however deep.
fn descendants_of(parent: isize) -> Vec<isize> {
    FOUND.with(|found| found.borrow_mut().clear());
    // SAFETY: the callback only pushes to this thread's local.
    unsafe { EnumChildWindows(parent, collect, 0) };
    FOUND.with(|found| found.borrow().clone())
}

fn text_of(hwnd: isize, read: unsafe extern "system" fn(isize, *mut u16, i32) -> i32) -> String {
    if hwnd == 0 {
        return String::new();
    }
    let mut buffer = [0u16; 256];
    // SAFETY: the buffer is as long as the count says.
    let len = unsafe { read(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

fn class_name(hwnd: isize) -> String {
    text_of(hwnd, GetClassNameW)
}

fn window_title(hwnd: isize) -> String {
    text_of(hwnd, GetWindowTextW)
}

/// The one top-level window of this thread whose title is `title`, or 0.
fn the_window_titled(title: &str) -> isize {
    this_threads_windows()
        .into_iter()
        .find(|hwnd| window_title(*hwnd) == title)
        .unwrap_or(0)
}

/// Whether the browser under a host control exists yet, which is what the
/// run waits for before it activates anything: a window with no browser in it
/// cannot answer a question about the browser taking the keyboard.
fn the_browser_is_there(page_window: isize) -> bool {
    page_window != 0
        && descendants_of(page_window)
            .into_iter()
            .any(|hwnd| CHROMIUMS_CLASSES.contains(&class_name(hwnd).as_str()))
}

/// Every top-level window the run's thread owns with its title and class, for
/// a failure message. A run that cannot find the window it is about must say
/// what it did find, or the next reader is left guessing between a window
/// that never opened and one whose title moved.
fn the_windows_this_thread_has() -> String {
    let windows: Vec<String> = this_threads_windows()
        .into_iter()
        .map(|hwnd| format!("0x{hwnd:x} {:?} ({})", window_title(hwnd), class_name(hwnd)))
        .collect();
    if windows.is_empty() {
        return "the run's thread owned no top-level window at all".to_string();
    }
    format!("the run's thread owned: {}", windows.join("; "))
}

/// Whether `hwnd` is `ancestor` itself or anything under it.
fn inside(ancestor: isize, hwnd: isize) -> bool {
    if ancestor == 0 || hwnd == 0 {
        return false;
    }
    // SAFETY: plain reads of live handles.
    ancestor == hwnd || unsafe { IsChild(ancestor, hwnd) } != 0
}

/// The title of the window in front when it is one of the run's own, and
/// nothing more than that it is somebody else's otherwise.
///
/// Until 2026-09-23 every title was read, which was harmless while nothing
/// here was ever in front. That day the tester was using the machine, and the
/// run printed the title of the window he was reading in, into test output
/// that a failing commit gate prints and a refused red commit copies to
/// `red-run.log`. Another process's title is somebody's business, never this
/// reading's.
fn the_title_if_it_is_ours(in_front: isize) -> String {
    if this_threads_windows().contains(&in_front) {
        window_title(in_front)
    } else {
        "a window of another thread".to_string()
    }
}

/// What has the keyboard on the run's thread, and what is in front, read
/// rather than taken from the answer of the call that asked for either.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Reading {
    /// The window with the keyboard, 0 for none.
    focus: isize,
    /// Its class, which is what says whether the keyboard reached the
    /// document or stopped at the window hosting it.
    focus_class: String,
    /// Whether it is inside the window this step was about.
    focus_is_in_the_window: bool,
    /// What Windows says is in front, kept and never asserted on: no process
    /// started the way this one is can move the foreground here.
    in_front: isize,
    in_front_title: String,
}

impl Reading {
    fn take(the_window_this_step_is_about: isize) -> Self {
        // SAFETY: plain reads.
        let (focus, in_front) = unsafe { (GetFocus(), GetForegroundWindow()) };
        Self {
            focus,
            focus_class: class_name(focus),
            focus_is_in_the_window: inside(the_window_this_step_is_about, focus),
            in_front,
            in_front_title: the_title_if_it_is_ours(in_front),
        }
    }

    fn says(&self, what: &str) -> String {
        let keyboard = if self.focus == 0 {
            "nothing on the run's thread had the keyboard".to_string()
        } else {
            format!(
                "0x{:x} ({}), inside the window: {}",
                self.focus, self.focus_class, self.focus_is_in_the_window
            )
        };
        let front = if self.in_front == 0 {
            "no window was in front".to_string()
        } else {
            format!("0x{:x} ({:?})", self.in_front, self.in_front_title)
        };
        format!("{what}: the keyboard, {keyboard}; in front, {front}")
    }
}

/// Everything the run saw, as plain values the tests assert over.
#[derive(Debug, Default, Clone)]
struct WhatTheReadingSaw {
    /// After the page window was activated the first time, which is the state
    /// the NVDA case is in when it presses its first `K`.
    the_page_window_activated: Reading,
    /// After it was deactivated, the way the browser taking the front
    /// deactivates it.
    the_page_window_deactivated: Reading,
    /// After it was activated again: the step the run of 2026-09-20 could not
    /// say anything about.
    the_page_window_back: Reading,
    /// The companion, the same three steps over a text control.
    the_native_window_activated: Reading,
    the_native_window_deactivated: Reading,
    the_native_window_back: Reading,
    /// The frame-saved steps on the page window: its frame's handle, what
    /// held the keyboard just before the deactivation, after it was cleared,
    /// and after the window came back.
    the_page_frame: isize,
    the_page_frame_held_it: Reading,
    the_page_frame_cleared: Reading,
    the_page_window_back_from_its_frame: Reading,
    /// How many times the page window's way back had run when it was back.
    /// Nothing here closes the window, so anything but 0 is a timer whose tick
    /// reached the way back's handler.
    way_back_calls: u32,
    /// The same steps on the companion.
    the_native_frame: isize,
    the_native_frame_held_it: Reading,
    the_native_frame_cleared: Reading,
    the_native_window_back_from_its_frame: Reading,
    /// Every top-level window the run's own thread owned when it ended, read
    /// there and carried here, because the tests assert on another thread and
    /// `EnumThreadWindows` would answer about that one.
    the_windows_the_run_had: String,
}

impl WhatTheReadingSaw {
    fn says(&self) -> String {
        [
            self.the_windows_the_run_had.clone(),
            self.the_page_window_activated
                .says("the page window activated"),
            self.the_page_window_deactivated
                .says("the page window deactivated"),
            self.the_page_window_back.says("the page window back"),
            self.the_native_window_activated
                .says("the text control's window activated"),
            self.the_native_window_deactivated
                .says("the text control's window deactivated"),
            self.the_native_window_back
                .says("the text control's window back"),
            format!("the page window's frame is 0x{:x}", self.the_page_frame),
            self.the_page_frame_held_it
                .says("the page window's frame given the keyboard, before the deactivation"),
            self.the_page_frame_cleared
                .says("the page window deactivated with its frame saved, keyboard cleared"),
            self.the_page_window_back_from_its_frame
                .says("the page window back from its frame"),
            format!(
                "the page window's way back ran {} times",
                self.way_back_calls
            ),
            format!(
                "the text control's window's frame is 0x{:x}",
                self.the_native_frame
            ),
            self.the_native_frame_held_it
                .says("the text control's frame given the keyboard, before the deactivation"),
            self.the_native_frame_cleared.says(
                "the text control's window deactivated with its frame saved, keyboard cleared",
            ),
            self.the_native_window_back_from_its_frame
                .says("the text control's window back from its frame"),
        ]
        .join("\n")
    }
}

/// The windows and the harvest, carried through the steps.
struct TheRun {
    parent: Frame,
    page_window: Cell<isize>,
    native_window: RefCell<Option<Frame>>,
    saw: RefCell<WhatTheReadingSaw>,
    /// Counted by the way back the page window was built with.
    way_back_calls: Rc<Cell<u32>>,
}

impl TheRun {
    fn native_window_handle(&self) -> isize {
        self.native_window
            .borrow()
            .as_ref()
            .map(|frame| frame.get_handle() as isize)
            .unwrap_or(0)
    }
}

/// What a step asks of the next tick.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Next {
    /// Not ready: ask this step again.
    AskAgain,
    /// Done: let the window settle for this many ticks, then take the next
    /// step.
    Settle(u32),
}

/// The message Windows sends a window when the foreground arrives at it or
/// leaves it, sent here for the reason the file header gives.
fn activation(hwnd: isize, state: usize) {
    // SAFETY: a live handle from a window this thread built.
    unsafe { SendMessageW(hwnd, WM_ACTIVATE, state, 0) };
}

/// The rest of what really happens when another process takes the front: the
/// thread loses the keyboard, so `GetFocus` answers 0 for it.
///
/// Both halves are needed and in this order. `wxTopLevelWindowMSW::OnActivate`
/// saves the focused child while answering the message, so the message comes
/// first; and on the way back it restores nothing at all while a descendant
/// still holds the keyboard, so without this the coming-back step would find
/// the keyboard where it left it and prove nothing. That is how this reading
/// first passed, and it is the shape of check this project's own file warns
/// about.
fn and_the_keyboard_goes_with_it() {
    // SAFETY: clearing this thread's own focus, which is what Windows does
    // to a thread whose window stops being active.
    unsafe { SetFocus(0) };
}

/// The first half of the frame-saved steps, in one step so nothing runs
/// between its parts: the keyboard put on `frame` itself, what holds it read,
/// and the window deactivated, so wx saves the frame as its last focused
/// child before anything deferred to a later turn of the loop can move it.
fn the_frame_takes_the_keyboard_and_is_deactivated(frame: isize) -> Reading {
    // SAFETY: a live handle from a window this thread built.
    unsafe { SetFocus(frame) };
    let held = Reading::take(frame);
    activation(frame, WA_INACTIVE);
    held
}

/// The second half: the keyboard taken away as a deactivating thread loses
/// it, what holds it read, and the window activated again.
fn the_keyboard_is_cleared_and_the_window_activated(frame: isize) -> Reading {
    and_the_keyboard_goes_with_it();
    let cleared = Reading::take(frame);
    activation(frame, WA_ACTIVE);
    cleared
}

fn one_step(at: usize, run: &TheRun) -> Next {
    match at {
        // The page window is built before the timer starts. This waits for
        // WebView2 to have made the browser, since a window with no browser
        // in it cannot answer where the keyboard goes.
        0 => {
            let page_window = the_window_titled(THE_PAGE_WINDOW);
            run.page_window.set(page_window);
            if the_browser_is_there(page_window) {
                Next::Settle(TICKS_TO_SETTLE)
            } else {
                Next::AskAgain
            }
        }
        1 => {
            activation(run.page_window.get(), WA_ACTIVE);
            Next::Settle(TICKS_TO_SETTLE)
        }
        2 => {
            run.saw.borrow_mut().the_page_window_activated = Reading::take(run.page_window.get());
            Next::Settle(0)
        }
        3 => {
            activation(run.page_window.get(), WA_INACTIVE);
            and_the_keyboard_goes_with_it();
            Next::Settle(TICKS_TO_SETTLE)
        }
        4 => {
            run.saw.borrow_mut().the_page_window_deactivated = Reading::take(run.page_window.get());
            Next::Settle(0)
        }
        5 => {
            activation(run.page_window.get(), WA_ACTIVE);
            Next::Settle(TICKS_TO_SETTLE)
        }
        6 => {
            run.saw.borrow_mut().the_page_window_back = Reading::take(run.page_window.get());
            Next::Settle(0)
        }
        // The companion: the same three steps over a control that is not a
        // browser, on a window built the same way.
        7 => {
            let native = Frame::builder()
                .with_parent(&run.parent)
                .with_title(THE_NATIVE_WINDOW)
                .with_size(Size::new(500, 400))
                .build();
            let _box_to_type_in = TextCtrl::builder(&native).build();
            native.show(true);
            run.native_window.replace(Some(native));
            Next::Settle(TICKS_TO_SETTLE)
        }
        8 => {
            activation(run.native_window_handle(), WA_ACTIVE);
            Next::Settle(TICKS_TO_SETTLE)
        }
        9 => {
            run.saw.borrow_mut().the_native_window_activated =
                Reading::take(run.native_window_handle());
            Next::Settle(0)
        }
        10 => {
            activation(run.native_window_handle(), WA_INACTIVE);
            and_the_keyboard_goes_with_it();
            Next::Settle(TICKS_TO_SETTLE)
        }
        11 => {
            run.saw.borrow_mut().the_native_window_deactivated =
                Reading::take(run.native_window_handle());
            Next::Settle(0)
        }
        12 => {
            activation(run.native_window_handle(), WA_ACTIVE);
            Next::Settle(TICKS_TO_SETTLE)
        }
        13 => {
            run.saw.borrow_mut().the_native_window_back = Reading::take(run.native_window_handle());
            Next::Settle(0)
        }
        // The frame-saved steps on the page window, which the file header
        // describes. Activated first, so the keyboard is in the browser and
        // the window is the active one when its frame is given the keyboard.
        14 => {
            activation(run.page_window.get(), WA_ACTIVE);
            Next::Settle(TICKS_TO_SETTLE)
        }
        15 => {
            let frame = run.page_window.get();
            let held = the_frame_takes_the_keyboard_and_is_deactivated(frame);
            let mut saw = run.saw.borrow_mut();
            saw.the_page_frame = frame;
            saw.the_page_frame_held_it = held;
            Next::Settle(TICKS_TO_SETTLE)
        }
        16 => {
            let cleared = the_keyboard_is_cleared_and_the_window_activated(run.page_window.get());
            run.saw.borrow_mut().the_page_frame_cleared = cleared;
            Next::Settle(TICKS_TO_SETTLE)
        }
        17 => {
            let mut saw = run.saw.borrow_mut();
            saw.the_page_window_back_from_its_frame = Reading::take(run.page_window.get());
            saw.way_back_calls = run.way_back_calls.get();
            Next::Settle(0)
        }
        // The same steps on the companion, which has no code of its own for
        // coming back.
        18 => {
            activation(run.native_window_handle(), WA_ACTIVE);
            Next::Settle(TICKS_TO_SETTLE)
        }
        19 => {
            let frame = run.native_window_handle();
            let held = the_frame_takes_the_keyboard_and_is_deactivated(frame);
            let mut saw = run.saw.borrow_mut();
            saw.the_native_frame = frame;
            saw.the_native_frame_held_it = held;
            Next::Settle(TICKS_TO_SETTLE)
        }
        20 => {
            let cleared =
                the_keyboard_is_cleared_and_the_window_activated(run.native_window_handle());
            run.saw.borrow_mut().the_native_frame_cleared = cleared;
            Next::Settle(TICKS_TO_SETTLE)
        }
        21 => {
            run.saw.borrow_mut().the_native_window_back_from_its_frame =
                Reading::take(run.native_window_handle());
            Next::Settle(0)
        }
        // Hidden rather than destroyed: a frame holding a browser torn down
        // while WebView2 is still working takes the process with it, which is
        // what `presentation::browser_ready` exists for and what this run has
        // no handle to ask.
        _ => {
            if let Some(native) = run.native_window.borrow().as_ref() {
                native.show(false);
            }
            Next::Settle(0)
        }
    }
}

/// The last step above, after which the run is over.
const THE_LAST_STEP: usize = 22;

fn the_reading() -> &'static Result<WhatTheReadingSaw, String> {
    static READING: OnceLock<Result<WhatTheReadingSaw, String>> = OnceLock::new();
    READING.get_or_init(run_the_reading)
}

fn run_the_reading() -> Result<WhatTheReadingSaw, String> {
    let data_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(why) => return Err(format!("no temporary data directory: {why}")),
    };
    // SAFETY: set before any thread is started and before anything reads it.
    unsafe {
        std::env::set_var("WIXEN_MAIL_DATA", data_dir.path());
        std::env::set_var("WIXEN_NO_AUDIO", "1");
    }

    let outcome: Arc<Mutex<Result<WhatTheReadingSaw, String>>> = Arc::new(Mutex::new(Err(
        "the run ended before it read anything".to_string(),
    )));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let a11y = match Accessibility::new() {
                Ok(a11y) => Arc::new(a11y),
                Err(why) => {
                    *outcome.lock().unwrap() = Err(format!("no accessibility layer: {why}"));
                    app.exit_main_loop();
                    return;
                }
            };
            let parent = Frame::builder().build();
            let reader = Rc::new(wx_reader::ReaderWindow::new(&parent, &a11y));
            // The fixture's own bodies, one written as a page with the
            // sender's link and one as text with an address on a line of its
            // own. The scan target folds each through the armour and safety
            // composition first; neither has anything to say about this
            // fixture, which is ordinary and unsigned, and neither can move
            // the keyboard, so the parts are built from the bodies here.
            let parts: Vec<ConversationPart> = scan_fixtures::page_conversation()
                .into_iter()
                .enumerate()
                .map(|(depth, (message, body))| ConversationPart {
                    message,
                    body,
                    said: WhatIsSaidAboutIt::nothing(),
                    depth,
                })
                .collect();
            // A way back that counts, where `None` stood until 2026-09-23:
            // the window runs it from a timer on its frame, so a second timer
            // on that frame would run it too, and the count is what says so.
            let way_back_calls = Rc::new(Cell::new(0u32));
            let way_back: Rc<dyn Fn()> = Rc::new({
                let way_back_calls = way_back_calls.clone();
                move || way_back_calls.set(way_back_calls.get() + 1)
            });
            wx_app::show_conversation_as_page(
                &parent,
                &reader,
                &a11y,
                "Scan target",
                &parts,
                Some(way_back),
            );

            let run = Rc::new(TheRun {
                parent,
                page_window: Cell::new(0),
                native_window: RefCell::new(None),
                saw: RefCell::new(WhatTheReadingSaw::default()),
                way_back_calls,
            });
            let at = Rc::new(Cell::new(0usize));
            let settling = Rc::new(Cell::new(0u32));
            let ticks = Rc::new(Cell::new(0u32));

            let ticker = Rc::new(Timer::new(&run.parent));
            ticker.on_tick({
                let run = run.clone();
                let outcome = outcome.clone();
                let ticker = ticker.clone();
                move |_| {
                    ticks.set(ticks.get() + 1);
                    if settling.get() > 0 {
                        settling.set(settling.get() - 1);
                        return;
                    }
                    let gave_up = ticks.get() > GIVE_UP_AFTER_TICKS;
                    if !gave_up && at.get() <= THE_LAST_STEP {
                        match one_step(at.get(), &run) {
                            Next::AskAgain => return,
                            Next::Settle(pause) => {
                                settling.set(pause);
                                at.set(at.get() + 1);
                                return;
                            }
                        }
                    }
                    run.saw.borrow_mut().the_windows_the_run_had = the_windows_this_thread_has();
                    println!("{THE_READINGS_LINE}\n{}", run.saw.borrow().says());
                    *outcome.lock().unwrap() = if gave_up {
                        Err(format!(
                            "the run waited too long at step {} and gave up. What it had:\n{}",
                            at.get(),
                            run.saw.borrow().says()
                        ))
                    } else {
                        Ok(run.saw.borrow().clone())
                    };
                    // Stopped before its owner goes: a timer firing on a
                    // destroyed window is an access violation.
                    ticker.stop();
                    wxdragon::call_after(Box::new(move || app.exit_main_loop()));
                }
            });
            ticker.start(TICK_MS, false);
            // Dropping the last handle destroys the timer, and it must
            // outlive this closure. The tick closure holds the other handle.
            std::mem::forget(ticker);
            std::mem::forget(run);
            std::mem::forget(reader);
        })
    };
    if let Err(why) = result {
        return Err(format!("wxdragon::main returned {why:?}"));
    }
    let outcome = outcome.lock().unwrap();
    outcome.clone()
}

fn harvest() -> WhatTheReadingSaw {
    match the_reading() {
        Ok(saw) => saw.clone(),
        Err(why) => panic!("the reading could not be taken: {why}"),
    }
}

#[test]
fn test_the_document_keeps_focus_when_the_page_window_comes_back() {
    let saw = harvest();
    let all = saw.says();

    assert!(
        saw.the_page_window_activated.focus_is_in_the_window
            && CHROMIUMS_CLASSES.contains(&saw.the_page_window_activated.focus_class.as_str()),
        "the page window's document did not take the keyboard when the window was activated, so \
         what happens when it comes back cannot be asked, and a next-link key would do nothing \
         the first time it was pressed.\n{all}"
    );
    assert_eq!(
        saw.the_page_window_deactivated.focus, 0,
        "the keyboard was still in the window after it was deactivated, so nothing was given \
         back when it came back and this reading asserts nothing.\n{all}"
    );
    assert!(
        saw.the_page_window_back.focus_is_in_the_window,
        "the page window came back and the keyboard landed outside it.\n{all}"
    );
    assert!(
        CHROMIUMS_CLASSES.contains(&saw.the_page_window_back.focus_class.as_str()),
        "the page window came back and the keyboard landed on {:?}, which is not one of the \
         browser's windows ({CHROMIUMS_CLASSES:?}), so browse mode has no document and a \
         next-link key is not a next-link key.\n{all}",
        saw.the_page_window_back.focus_class
    );
}

#[test]
fn test_the_reading_sees_a_control_that_is_not_the_browser() {
    let saw = harvest();
    let all = saw.says();

    assert_eq!(
        saw.the_native_window_activated.focus_class, A_TEXT_CONTROLS_CLASS,
        "a window whose one control is a text control did not put the keyboard in it when it was \
         activated, so this reading cannot tell a browser from anything else.\n{all}"
    );
    assert_eq!(
        saw.the_native_window_deactivated.focus, 0,
        "the keyboard was still in the text control's window after it was deactivated, so \
         nothing was given back when it came back.\n{all}"
    );
    assert_eq!(
        saw.the_native_window_back.focus_class, A_TEXT_CONTROLS_CLASS,
        "the text control's window came back and the keyboard landed elsewhere.\n{all}"
    );
}

#[test]
fn test_the_page_takes_the_keyboard_back_when_the_window_restored_it_to_itself() {
    let saw = harvest();
    let all = saw.says();

    assert_ne!(
        saw.the_page_frame, 0,
        "the page window was never found.\n{all}"
    );
    assert_eq!(
        saw.the_page_frame_held_it.focus, saw.the_page_frame,
        "the page window's frame did not hold the keyboard when the window was deactivated, so wx \
         did not save the frame and the step reproduces nothing.\n{all}"
    );
    assert_eq!(
        saw.the_page_frame_cleared.focus, 0,
        "the keyboard was still held after it was cleared, so the window's own restore was never \
         asked for.\n{all}"
    );
    assert!(
        saw.the_page_window_back_from_its_frame
            .focus_is_in_the_window
            && CHROMIUMS_CLASSES
                .contains(&saw.the_page_window_back_from_its_frame.focus_class.as_str()),
        "the page window came back with its frame saved and the keyboard landed on 0x{:x} ({:?}), \
         not in the browser ({CHROMIUMS_CLASSES:?}), so browse mode has no document and K and H do \
         nothing. This is the state NVDA runs 35839692317 and 35839954840 recorded.\n{all}",
        saw.the_page_window_back_from_its_frame.focus,
        saw.the_page_window_back_from_its_frame.focus_class
    );
    assert_eq!(
        saw.way_back_calls, 0,
        "the page window's way back ran although nothing closed the window: a timer's tick reached \
         the handler that runs it, which would open the conversation again over a window still \
         showing.\n{all}"
    );
}

#[test]
fn test_a_window_with_no_such_handler_is_left_with_the_keyboard_on_its_frame() {
    let saw = harvest();
    let all = saw.says();

    assert_ne!(
        saw.the_native_frame, 0,
        "the companion was never built.\n{all}"
    );
    assert_eq!(
        saw.the_native_frame_held_it.focus, saw.the_native_frame,
        "the companion's frame did not hold the keyboard when it was deactivated.\n{all}"
    );
    assert_eq!(
        saw.the_native_frame_cleared.focus, 0,
        "the keyboard was still held on the companion after it was cleared.\n{all}"
    );
    assert_eq!(
        saw.the_native_window_back_from_its_frame.focus, saw.the_native_frame,
        "the companion came back with its frame saved and the keyboard did not land on the frame, \
         so wx's own restore does not end where the runner's record did and the page window's \
         reading above is not reproducing that state.\n{all}"
    );
}

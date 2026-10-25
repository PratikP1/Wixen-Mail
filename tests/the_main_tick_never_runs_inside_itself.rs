//! The main window's tick runs beside a window it opened, never inside itself.
//!
//! The main window polls every 50 ms, and several things that handler does open
//! a modal window, the Due now window first among them. Opened from inside the
//! handler, the window's loop ran the next tick inside the one that opened it.
//! wxDragon 0.9.17 allows that; from 0.9.23 a handler called inside itself is
//! skipped with a warning, so the tick would stop for as long as the window
//! stayed open. `presentation::after_the_tick` is what the main window hands
//! those windows to, and this runs it with the real Due now window.
//!
//! The main window's own tick cannot be built in a test, so a frame here
//! carries a 50 ms timer standing in for it, which counts how deeply it is
//! running and drains a channel the way the real one drains the workers'
//! updates. The readings in `the_main_window_opens_its_windows_after_its_tick.rs`
//! hold the real tick to the same queue.
//!
//! A test cannot press a key in the window, so the tick closes it by posting
//! `WM_CLOSE` to the handle found by its title among this thread's windows,
//! which dismisses what is left on it. A give-up timer, owned by another window
//! because a timer event reaches every handler bound on its owner, closes it
//! after ten seconds whatever happened and says that it had to, so a tick that
//! never runs again cannot hang the target.
//!
//! One `#[test]` function calling `wxdragon::main` once, for the reason
//! `tests/theme_reach.rs` gives: wxWidgets supports one application per
//! process. **Nothing here reaches the settings of whoever runs it, or their
//! screen.** The profile is a directory of its own, made before anything is
//! built, the sound goes nowhere, and the windows live on a desktop of this
//! process's own.

#![cfg(windows)]

use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, Mutex};
use wixen_mail::application::due::{Due, Identity, Kind, Snooze};
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::after_the_tick::after_the_tick;
use wixen_mail::presentation::date_display::DateSettings;
use wixen_mail::presentation::wx_reminder_alert::{self, NoEditors};
use wxdragon::prelude::*;

const TICK_MS: i32 = 50;
const GIVE_UP_MS: i32 = 10_000;
const WM_CLOSE: u32 = 0x0010;
const GWL_STYLE: i32 = -16;
const WS_VISIBLE: isize = 0x1000_0000;
const WINSTA_ALL_ACCESS: u32 = 0x037F;
const GENERIC_ALL: u32 = 0x1000_0000;

#[link(name = "user32")]
unsafe extern "system" {
    fn EnumThreadWindows(
        thread: u32,
        callback: extern "system" fn(isize, isize) -> i32,
        lparam: isize,
    ) -> i32;
    fn GetWindowTextW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
    fn GetWindowLongPtrW(hwnd: isize, index: i32) -> isize;
    fn PostMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> i32;
    fn CreateWindowStationW(
        name: *const u16,
        flags: u32,
        access: u32,
        attributes: *const c_void,
    ) -> isize;
    fn SetProcessWindowStation(station: isize) -> i32;
    fn CreateDesktopW(
        name: *const u16,
        device: *const u16,
        mode: *const c_void,
        flags: u32,
        access: u32,
        attributes: *const c_void,
    ) -> isize;
    fn SetThreadDesktop(desktop: isize) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetLastError() -> u32;
    fn GetCurrentThreadId() -> u32;
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Move this thread onto a desktop in a window station of this process's
/// own, so no window it makes reaches the screen of whoever runs the tests.
/// Must run before the first window of the process is made.
fn a_desktop_of_its_own() -> Result<(), String> {
    // SAFETY: every argument is a valid null-terminated string or null.
    unsafe {
        let station =
            CreateWindowStationW(std::ptr::null(), 0, WINSTA_ALL_ACCESS, std::ptr::null());
        if station == 0 {
            return Err(format!("CreateWindowStationW failed: {}", GetLastError()));
        }
        if SetProcessWindowStation(station) == 0 {
            return Err(format!(
                "SetProcessWindowStation failed: {}",
                GetLastError()
            ));
        }
        let desktop = CreateDesktopW(
            wide("wixen-the-main-tick-test").as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            0,
            GENERIC_ALL,
            std::ptr::null(),
        );
        if desktop == 0 {
            return Err(format!("CreateDesktopW failed: {}", GetLastError()));
        }
        if SetThreadDesktop(desktop) == 0 {
            return Err(format!("SetThreadDesktop failed: {}", GetLastError()));
        }
    }
    Ok(())
}

thread_local! {
    static FOUND: RefCell<Vec<isize>> = const { RefCell::new(Vec::new()) };
}

extern "system" fn collect(hwnd: isize, _lparam: isize) -> i32 {
    FOUND.with(|found| found.borrow_mut().push(hwnd));
    1
}

fn title_of(hwnd: isize) -> String {
    let mut buffer = [0u16; 512];
    // SAFETY: the buffer is as long as the count says.
    let len = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

/// The Due now window, if one of this thread's top-level windows is it and is
/// on screen. Shown as well as titled, because a closed dialog's handle can
/// outlive it until the toolkit's next idle. Read from the window's style
/// rather than `IsWindowVisible`, which answered no here, on this desktop of
/// the process's own, for the open window whose style said it was shown.
fn the_due_now_window() -> Option<isize> {
    FOUND.with(|found| found.borrow_mut().clear());
    // SAFETY: the callback only pushes to this thread's local.
    unsafe { EnumThreadWindows(GetCurrentThreadId(), collect, 0) };
    FOUND
        .with(|found| found.borrow().clone())
        .into_iter()
        .find(|hwnd| title_of(*hwnd) == wx_reminder_alert::TITLE && is_shown(*hwnd))
}

fn is_shown(hwnd: isize) -> bool {
    // SAFETY: reading a window's style is safe whatever the handle holds.
    unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) & WS_VISIBLE != 0 }
}

fn close(hwnd: isize) {
    // SAFETY: posting a message to a window handle is safe whatever it holds.
    unsafe { PostMessageW(hwnd, WM_CLOSE, 0, 0) };
}

/// Ends the event loop from a closure that captures nothing, so nothing
/// owned by this thread is sent to another.
fn end_the_loop() {
    wxdragon::call_after(Box::new(|| {
        if let Some(app) = wxdragon::get_app_instance() {
            app.exit_main_loop();
        }
    }));
}

/// What the run hands back: what went wrong, or why it could not run.
type Outcome = Arc<Mutex<Option<Result<Vec<String>, String>>>>;

/// What the run saw.
#[derive(Debug, Default)]
struct Seen {
    /// Where what was wrong is written when the run ends.
    outcome: Outcome,
    /// The channel the stand-in worker sends on, made once the window is up.
    from_the_worker: RefCell<Option<Receiver<&'static str>>>,
    /// How deeply the tick is running right now, and the deepest it ran.
    depth: Cell<u32>,
    deepest: Cell<u32>,
    /// Whether the Due now window's work has started and not yet returned.
    window_open: Cell<bool>,
    window_opened: Cell<bool>,
    window_closed: Cell<bool>,
    /// Whether the thread standing in for a worker has been started.
    sender_started: Cell<bool>,
    /// The tick that took the message, and whether the window was open then.
    message_taken_at_tick: Cell<Option<u32>>,
    message_taken_while_open: Cell<Option<bool>>,
    /// Whether the work queued while the window was open has run, and
    /// whether the window was still open when it did.
    queued_ran: Cell<bool>,
    queued_ran_while_open: Cell<Option<bool>>,
    ticks: Cell<u32>,
    gave_up: Cell<bool>,
    ended: Cell<bool>,
}

impl Seen {
    /// Ends the loop once the window has closed and the work queued behind
    /// it has run, whichever came last.
    fn end_when_both_are_done(&self) {
        if self.window_closed.get() && self.queued_ran.get() {
            self.end();
        }
    }

    /// Writes down what was wrong and ends the loop, once.
    fn end(&self) {
        if self.ended.replace(true) {
            return;
        }
        if let Ok(mut slot) = self.outcome.lock() {
            *slot = Some(Ok(self.what_was_wrong()));
        }
        end_the_loop();
    }

    /// Every property the run should have shown that it did not.
    fn what_was_wrong(&self) -> Vec<String> {
        let mut wrong = Vec::new();
        if self.deepest.get() != 1 {
            wrong.push(format!(
                "the tick ran {} deep, so it ran inside itself while the window it opened was up",
                self.deepest.get()
            ));
        }
        if !self.window_opened.get() || !self.window_closed.get() {
            wrong.push(format!(
                "the Due now window opened: {}, and closed: {}",
                self.window_opened.get(),
                self.window_closed.get()
            ));
        }
        match self.message_taken_while_open.get() {
            Some(true) => {}
            Some(false) => wrong.push(
                "the worker's message was taken only after the Due now window had closed".into(),
            ),
            None => wrong.push("the worker's message was never taken".into()),
        }
        match self.queued_ran_while_open.get() {
            Some(false) => {}
            Some(true) => wrong.push(
                "work the tick queued while the Due now window was open ran inside that window"
                    .into(),
            ),
            None => wrong.push("work the tick queued while the window was open never ran".into()),
        }
        if self.gave_up.get() {
            wrong.push(format!(
                "the give-up timer had to close the window after {GIVE_UP_MS} ms"
            ));
        }
        wrong
    }
}

/// One row, a reminder due now.
fn one_reminder() -> Vec<Due> {
    vec![Due {
        identity: Identity {
            kind: Kind::Reminder,
            id: "r1".to_string(),
        },
        title: "Call the bank".to_string(),
        when: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        late: false,
    }]
}

/// The work the tick hands on at its first run: the real Due now window.
fn open_the_due_now_window(seen: &Rc<Seen>, frame: Frame, a11y: &Arc<Accessibility>) {
    let seen = Rc::clone(seen);
    let a11y = Arc::clone(a11y);
    after_the_tick(move || {
        seen.window_open.set(true);
        seen.window_opened.set(true);
        let _answers = wx_reminder_alert::raise(
            &frame,
            one_reminder(),
            chrono::Local::now(),
            DateSettings::default(),
            &a11y,
            Snooze::ALL[2],
            Box::new(NoEditors),
        );
        seen.window_open.set(false);
        seen.window_closed.set(true);
        seen.end_when_both_are_done();
    });
}

/// One run of the stand-in tick.
fn tick(seen: &Rc<Seen>, frame: Frame, a11y: &Arc<Accessibility>) {
    seen.depth.set(seen.depth.get() + 1);
    seen.deepest.set(seen.deepest.get().max(seen.depth.get()));
    seen.ticks.set(seen.ticks.get() + 1);

    if seen.ticks.get() == 1 {
        open_the_due_now_window(seen, frame, a11y);
    }

    // The worker starts once the window is up, so its message can only
    // arrive while the window is open.
    let window = the_due_now_window();
    if window.is_some() && !seen.sender_started.replace(true) {
        let (tx, rx) = channel::<&'static str>();
        *seen.from_the_worker.borrow_mut() = Some(rx);
        std::thread::spawn(move || {
            let _ = tx.send("a message arrived");
        });
    }

    let arrived = seen
        .from_the_worker
        .borrow()
        .as_ref()
        .and_then(|rx| rx.try_recv().ok());
    if arrived.is_some() {
        seen.message_taken_at_tick.set(Some(seen.ticks.get()));
        seen.message_taken_while_open
            .set(Some(seen.window_open.get()));
        let queued = Rc::clone(seen);
        after_the_tick(move || {
            queued.queued_ran.set(true);
            queued
                .queued_ran_while_open
                .set(Some(queued.window_open.get()));
            queued.end_when_both_are_done();
        });
        if let Some(window) = window {
            close(window);
        }
    }

    seen.depth.set(seen.depth.get() - 1);
}

#[test]
fn test_the_tick_runs_beside_the_due_now_window_and_never_inside_itself() {
    let profile = tempfile::tempdir().expect("a profile of its own");
    // SAFETY: set once, before any thread of this process reads either.
    unsafe {
        std::env::set_var("WIXEN_MAIL_DATA", profile.path());
        std::env::set_var("WIXEN_NO_AUDIO", "1");
    }
    a_desktop_of_its_own().expect("a desktop of its own");

    let outcome = Outcome::default();
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |_| {
            let a11y = match Accessibility::new() {
                Ok(a11y) => Arc::new(a11y),
                Err(why) => {
                    if let Ok(mut slot) = outcome.lock() {
                        *slot = Some(Err(format!("accessibility: {why}")));
                    }
                    return end_the_loop();
                }
            };
            let frame = Frame::builder()
                .with_title("The main tick, standing in")
                .build();
            let keeper = Frame::builder()
                .with_title("The give-up timer's owner")
                .build();
            let seen = Rc::new(Seen {
                outcome: outcome.clone(),
                ..Seen::default()
            });

            let ticker = Timer::new(&frame);
            ticker.on_tick({
                let seen = Rc::clone(&seen);
                let a11y = Arc::clone(&a11y);
                move |_| tick(&seen, frame, &a11y)
            });

            let giving_up = Timer::new(&keeper);
            giving_up.on_tick({
                let seen = Rc::clone(&seen);
                move |_| {
                    if seen.ended.get() {
                        return;
                    }
                    seen.gave_up.set(true);
                    if let Some(window) = the_due_now_window() {
                        close(window);
                    }
                    seen.end();
                }
            });

            if !ticker.start(TICK_MS, false) || !giving_up.start(GIVE_UP_MS, true) {
                if let Ok(mut slot) = outcome.lock() {
                    *slot = Some(Err("a timer refused to start".into()));
                }
                return end_the_loop();
            }
            std::mem::forget(ticker);
            std::mem::forget(giving_up);
        })
    };
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");

    let outcome = outcome.lock().expect("the outcome").take();
    match outcome {
        Some(Ok(wrong)) => assert!(
            wrong.is_empty(),
            "the tick did not run beside the Due now window:\n  {}",
            wrong.join("\n  ")
        ),
        Some(Err(why)) => panic!("the run could not be made: {why}"),
        None => panic!("the run said nothing"),
    }
}

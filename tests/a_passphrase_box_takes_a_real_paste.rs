//! The passphrase box takes a real paste (GAP-03, #49, 13-17.1, 13-44.6.1).
//!
//! WCAG 3.3.8 asks that a secret can come from a password manager rather than
//! from memory, and the way a field gets that wrong is by refusing a paste. So
//! the real dialog is built with `wx_passphrase::build`, text is put on a
//! clipboard and `WM_PASTE` sent to its field, the message Ctrl+V and a
//! password manager both end in, and the field's text is read back. A
//! companion builds a field that refuses a paste and is caught by the same
//! reading.
//!
//! **A target of its own, because Windows refuses the clipboard to every
//! program while the session is locked.** Until 13-44.6.1 this paste ran in
//! `tests/a_locked_key_asks_for_its_passphrase.rs`, in front of four readings
//! of the dialog that never needed a clipboard, and the gate runs that target
//! on every commit touching `src/presentation/wx_app.rs`. One refused
//! clipboard failed six cases and held commits for hours (ledger 716). Here it
//! is coupled to `src/presentation/wx_passphrase.rs` alone, the dialog it
//! proves, and it still runs at the phase's full gate and on CI.
//!
//! **A refused clipboard fails, saying why, and never passes.** The clipboard
//! is tried for two seconds, which outlasts a brief hold by another program,
//! and then the case fails with one sentence naming the cause as Windows
//! gives it: the session locked, asked of `WTSQuerySessionInformationW` and
//! never read from the process list; another window holding it; or neither.
//! A refusal is never read as a field that refused a paste, which has a
//! sentence of its own.
//!
//! **The clipboard is the logon's test clipboard, not one of this process's
//! own.** 13-17.1 said this process made "a clipboard of its own". It makes a
//! window station with no name, and Windows names such a station from the
//! logon, so every test process of one Windows logon that makes one gets the
//! same station and the same clipboard (measured 2026-10-02, 13-44.6.1). So
//! two runs take turns at it. The clipboard of whoever runs the tests, on
//! their own station, is still never read or written, and no window here
//! reaches their screen.

#![cfg(windows)]

use std::ffi::c_void;
use std::fmt;
use std::sync::mpsc;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use wixen_mail::presentation::wx_passphrase::build;
use wxdragon::prelude::*;

const WM_PASTE: u32 = 0x0302;
const CF_UNICODETEXT: u32 = 13;
const GMEM_MOVEABLE: u32 = 0x0002;
const WINSTA_ALL_ACCESS: u32 = 0x037F;
const GENERIC_ALL: u32 = 0x1000_0000;
const ERROR_ACCESS_DENIED: u32 = 5;
const HWND_MESSAGE: isize = -3;

/// How many times the clipboard is tried: two seconds in all, which outlasts
/// a brief hold by another program and only delays the failure while the
/// session is locked.
const TRIES: u32 = 50;

/// The turn every run of this target takes at the logon's test clipboard.
const THE_TURN: &str = "Local\\wixen-mail-tests-one-turn-at-the-clipboard";

/// What the paste puts on the clipboard: long, with spaces, the shape a
/// password manager's generated passphrase has.
const PASTED: &str = "correct horse battery staple, pasted";

const WHOSE: &str = "Ada Lovelace <ada@example.com>";

const THE_FIELD_REFUSED_A_PASTE: &str = "the text was on the clipboard and WM_PASTE was sent, \
     and the field stayed empty: the field refused a paste";

#[link(name = "user32")]
unsafe extern "system" {
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
    fn OpenClipboard(owner: isize) -> i32;
    fn EmptyClipboard() -> i32;
    fn SetClipboardData(format: u32, memory: *mut c_void) -> *mut c_void;
    fn CloseClipboard() -> i32;
    fn CreateWindowExW(
        extended_style: u32,
        class: *const u16,
        title: *const u16,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: isize,
        menu: isize,
        instance: isize,
        parameter: *const c_void,
    ) -> isize;
    fn DestroyWindow(hwnd: isize) -> i32;
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
    fn GlobalAlloc(flags: u32, bytes: usize) -> *mut c_void;
    fn GlobalLock(memory: *mut c_void) -> *mut c_void;
    fn GlobalUnlock(memory: *mut c_void) -> i32;
    fn GetLastError() -> u32;
    fn CreateMutexW(attributes: *const c_void, initial_owner: i32, name: *const u16) -> isize;
    fn ReleaseMutex(handle: isize) -> i32;
    fn CloseHandle(handle: isize) -> i32;
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

// ── Why the clipboard was refused ──────────────────────────────────────────

/// Whether Windows says this session is locked, asked of Windows and never
/// read from the process list.
#[derive(Debug, Clone, PartialEq)]
enum SessionLock {
    Locked,
    Unlocked,
    NotSaid(String),
}

/// The window Windows names as holding the clipboard, and its process when
/// Windows gives one.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Holder {
    window: isize,
    process: Option<u32>,
}

/// The clipboard refused on every try, with what Windows said about why.
#[derive(Debug, Clone, PartialEq)]
struct Refusal {
    error: u32,
    tries: u32,
    waited: Duration,
    holder: Option<Holder>,
    session: SessionLock,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = (self.error, self.tries, self.waited);
        let _ = self.holder.map(|holder| (holder.window, holder.process));
        if let SessionLock::NotSaid(answer) = &self.session {
            let _ = answer;
        }
        write!(f, "")
    }
}

/// Why nothing was pasted: the clipboard was refused, or it opened and would
/// not take the text.
#[derive(Debug, Clone, PartialEq)]
enum NoPaste {
    Refused(Refusal),
    NotTaken(u32),
}

impl fmt::Display for NoPaste {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NoPaste::Refused(refusal) => refusal.fmt(f),
            NoPaste::NotTaken(error) => write!(
                f,
                "the clipboard opened and would not take the text, error {error}, so nothing \
                 was pasted"
            ),
        }
    }
}

/// Whether this session is locked, as Windows answers it.
fn this_sessions_lock() -> SessionLock {
    SessionLock::NotSaid("not asked yet".to_string())
}

/// Open the clipboard for `owner`, trying `tries` times, and on the last
/// refusal say what Windows says about why.
fn open_the_clipboard(owner: isize, tries: u32) -> Result<(), Refusal> {
    let started = Instant::now();
    // SAFETY: opens the clipboard for a live window or for none.
    if unsafe { OpenClipboard(owner) } != 0 {
        return Ok(());
    }
    // SAFETY: reads this thread's last error.
    let error = unsafe { GetLastError() };
    Err(Refusal {
        error,
        tries: tries.min(1),
        waited: started.elapsed(),
        holder: None,
        session: this_sessions_lock(),
    })
}

// ── The station, the turn and the paste ────────────────────────────────────

/// Move this thread onto a desktop in a window station that is not the one
/// of whoever runs the tests, so the clipboard it writes is not theirs and no
/// window it makes reaches their screen. Hands back the desktop, so another
/// thread can join it.
///
/// Must run before the first window of the process is made, since a thread
/// with a window cannot change desktop.
fn a_desktop_of_its_own() -> Result<isize, String> {
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
            wide("wixen-passphrase-paste-test").as_ptr(),
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
        Ok(desktop)
    }
}

/// One run's turn at the logon's test clipboard, which every test process of
/// this Windows logon shares, held until it is dropped on the thread that
/// took it.
struct OneTurnAtTheClipboard(isize);

impl OneTurnAtTheClipboard {
    fn take() -> Result<Self, String> {
        let name = wide(THE_TURN);
        // SAFETY: a null-terminated name and no security attributes.
        let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
        if handle == 0 {
            // SAFETY: reads this thread's last error.
            let error = unsafe { GetLastError() };
            return Err(format!("the turn's mutex could not be made: error {error}"));
        }
        Ok(OneTurnAtTheClipboard(handle))
    }
}

impl Drop for OneTurnAtTheClipboard {
    fn drop(&mut self) {
        // SAFETY: the handle this turn took, released on the thread that took it.
        unsafe {
            ReleaseMutex(self.0);
            CloseHandle(self.0);
        }
    }
}

/// Put `text` on this window station's clipboard, owned by `owner`, trying
/// the clipboard `tries` times.
fn put_on_the_clipboard(owner: isize, text: &str, tries: u32) -> Result<(), NoPaste> {
    open_the_clipboard(owner, tries).map_err(NoPaste::Refused)?;
    let units = wide(text);
    // SAFETY: the memory is sized for the text and its terminator, written
    // while locked, and handed to the clipboard, which owns it from then on.
    unsafe {
        EmptyClipboard();
        let memory = GlobalAlloc(GMEM_MOVEABLE, units.len() * 2);
        let placed = !memory.is_null() && {
            let at = GlobalLock(memory) as *mut u16;
            if !at.is_null() {
                std::ptr::copy_nonoverlapping(units.as_ptr(), at, units.len());
                GlobalUnlock(memory);
            }
            !at.is_null() && !SetClipboardData(CF_UNICODETEXT, memory).is_null()
        };
        let error = GetLastError();
        CloseClipboard();
        if placed {
            Ok(())
        } else {
            Err(NoPaste::NotTaken(error))
        }
    }
}

/// What a paste from the clipboard puts in a field: `PASTED` put on the
/// clipboard, `WM_PASTE` sent, and the field's text read back.
fn what_a_paste_puts_in(field: &TextCtrl) -> Result<String, NoPaste> {
    let handle = field.get_handle() as isize;
    put_on_the_clipboard(handle, PASTED, TRIES)?;
    // SAFETY: a live edit control on this thread; WM_PASTE takes no pointers.
    unsafe { SendMessageW(handle, WM_PASTE, 0, 0) };
    Ok(field.get_value())
}

// ── Another thread holding the clipboard ───────────────────────────────────

/// How long the second thread holds the clipboard.
#[derive(Clone, Copy)]
enum Hold {
    Briefly,
    Throughout,
}

/// What putting text on the clipboard came to while another thread held it,
/// and the window that held it.
#[derive(Debug)]
struct WhileHeld {
    holder: isize,
    reading: Result<(), NoPaste>,
}

/// A second thread on `desktop` holding the clipboard through a message-only
/// window of its own, for as long as `hold` says.
fn hold_the_clipboard(
    desktop: isize,
    hold: Hold,
    held: mpsc::Sender<Result<isize, String>>,
    done: mpsc::Receiver<()>,
) {
    // SAFETY: this thread has no window yet, so it may change desktop.
    if unsafe { SetThreadDesktop(desktop) } == 0 {
        // SAFETY: reads this thread's last error.
        let error = unsafe { GetLastError() };
        let _ = held.send(Err(format!("SetThreadDesktop failed: {error}")));
        return;
    }
    // SAFETY: a message-only window of a system class, made on this thread.
    let window = unsafe {
        CreateWindowExW(
            0,
            wide("STATIC").as_ptr(),
            std::ptr::null(),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            0,
            0,
            std::ptr::null(),
        )
    };
    if window == 0 {
        // SAFETY: reads this thread's last error.
        let error = unsafe { GetLastError() };
        let _ = held.send(Err(format!("CreateWindowExW failed: {error}")));
        return;
    }
    match open_the_clipboard(window, TRIES) {
        Err(refusal) => {
            let _ = held.send(Err(format!("the holding thread was refused: {refusal}")));
        }
        Ok(()) => {
            let _ = held.send(Ok(window));
            match hold {
                Hold::Briefly => std::thread::sleep(Duration::from_millis(200)),
                Hold::Throughout => {
                    let _ = done.recv();
                }
            }
            // SAFETY: this thread opened it.
            unsafe { CloseClipboard() };
        }
    }
    // SAFETY: this thread's own window.
    unsafe { DestroyWindow(window) };
}

/// Hold the clipboard from a second thread on `desktop`, and once that thread
/// says it holds it, put text on it for `owner` with `tries`.
fn read_while_another_thread_holds(
    desktop: isize,
    owner: isize,
    tries: u32,
    hold: Hold,
) -> Result<WhileHeld, String> {
    let (held_tx, held_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let holder = std::thread::spawn(move || hold_the_clipboard(desktop, hold, held_tx, done_rx));
    let outcome = match held_rx.recv() {
        Ok(Ok(window)) => Ok(WhileHeld {
            holder: window,
            reading: put_on_the_clipboard(owner, PASTED, tries),
        }),
        Ok(Err(why)) => Err(why),
        Err(_) => Err("the holding thread ended without saying whether it held it".to_string()),
    };
    let _ = done_tx.send(());
    let _ = holder.join();
    outcome
}

// ── The window session ─────────────────────────────────────────────────────

/// Everything read out of the window session, as plain values.
#[derive(Debug)]
struct Harvest {
    pasted: Result<String, NoPaste>,
    ok: Option<String>,
    cancel: Option<String>,
    refused_a_paste: Result<String, NoPaste>,
    held_throughout: Result<WhileHeld, String>,
    held_briefly: Result<WhileHeld, String>,
}

fn take_the_harvest() -> Result<Harvest, String> {
    let _turn = OneTurnAtTheClipboard::take()?;
    let desktop = a_desktop_of_its_own()?;
    let outcome: Arc<Mutex<Option<Harvest>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let frame = Frame::builder().build();

            let asking = build(&frame, WHOSE, None);
            asking.dialog.show(true);
            let pasted = what_a_paste_puts_in(&asking.field);
            let ok = asking.answer(ID_OK);
            let cancel = asking.answer(ID_CANCEL);

            // The companion: a field that refuses a paste, read the same way.
            let refusing = TextCtrl::builder(&asking.dialog)
                .with_style(TextCtrlStyle::Password | TextCtrlStyle::ReadOnly)
                .build();
            let refused_a_paste = what_a_paste_puts_in(&refusing);

            let owner = asking.dialog.get_handle() as isize;
            let held_throughout =
                read_while_another_thread_holds(desktop, owner, 3, Hold::Throughout);
            let held_briefly =
                read_while_another_thread_holds(desktop, owner, TRIES, Hold::Briefly);
            asking.dialog.destroy();

            if let Ok(mut slot) = outcome.lock() {
                *slot = Some(Harvest {
                    pasted,
                    ok,
                    cancel,
                    refused_a_paste,
                    held_throughout,
                    held_briefly,
                });
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
    taken.ok_or_else(|| "the window session ended without a harvest".to_string())
}

/// The one harvest of this process, taken by whichever test asks first.
fn the_harvest() -> &'static Harvest {
    static HARVEST: OnceLock<Result<Harvest, String>> = OnceLock::new();
    match HARVEST.get_or_init(take_the_harvest) {
        Ok(harvest) => harvest,
        Err(why) => panic!("the window session could not be read: {why}"),
    }
}

// ── The paste ──────────────────────────────────────────────────────────────

#[test]
fn test_a_pasted_passphrase_is_taken_and_ok_hands_it_back() {
    let harvest = the_harvest();

    match &harvest.pasted {
        Err(why) => panic!("{why}"),
        Ok(text) => assert_eq!(text, PASTED, "{THE_FIELD_REFUSED_A_PASTE}"),
    }
    assert_eq!(harvest.ok.as_deref(), Some(PASTED));
    assert_eq!(harvest.cancel, None, "Cancel handed back what was typed");
}

#[test]
fn test_the_paste_reading_sees_a_field_that_refuses_one() {
    // The companion. A reading that always found the text would pass the
    // case above whatever the field did with a paste, and a refused
    // clipboard would leave this field empty too, so it is told apart.
    match &the_harvest().refused_a_paste {
        Err(why) => panic!("{why}"),
        Ok(text) => assert_eq!(text, ""),
    }
}

// ── A refused clipboard says why ───────────────────────────────────────────

fn a_refusal(holder: Option<Holder>, session: SessionLock) -> Refusal {
    Refusal {
        error: ERROR_ACCESS_DENIED,
        tries: TRIES,
        waited: Duration::from_millis(2_000),
        holder,
        session,
    }
}

const A_HOLDER: Holder = Holder {
    window: 0x1234,
    process: Some(4242),
};

#[test]
fn test_a_refusal_while_the_session_is_locked_says_the_session_is_locked() {
    let said = a_refusal(Some(A_HOLDER), SessionLock::Locked).to_string();

    assert!(
        said.starts_with("the clipboard is refused while the session is locked"),
        "{said}"
    );
    for part in [
        "error 5",
        "50 tries",
        "2000 ms",
        "No test can paste while the session is locked",
        "Unlock it and run this target again",
        "says nothing about whether the field takes a paste",
    ] {
        assert!(said.contains(part), "{part:?} is not in: {said}");
    }
}

#[test]
fn test_a_refusal_while_a_window_holds_the_clipboard_names_that_window() {
    let said = a_refusal(Some(A_HOLDER), SessionLock::Unlocked).to_string();

    for part in [
        "window 0x1234",
        "process 4242",
        "Another run of a test, or another program on this logon's test window station",
        "error 5",
    ] {
        assert!(said.contains(part), "{part:?} is not in: {said}");
    }
}

#[test]
fn test_a_refusal_with_the_session_unlocked_and_no_window_says_the_lock_is_not_the_cause() {
    let said = a_refusal(None, SessionLock::Unlocked).to_string();

    for part in [
        "the lock is not the cause",
        "a holder that opened the clipboard without a window cannot be named",
        "error 5",
    ] {
        assert!(said.contains(part), "{part:?} is not in: {said}");
    }
}

#[test]
fn test_a_refusal_windows_gave_no_lock_state_for_says_so() {
    let answer = "WTSQuerySessionInformationW failed with error 87";
    let said = a_refusal(None, SessionLock::NotSaid(answer.to_string())).to_string();

    for part in [
        "Windows did not say whether the session is locked",
        answer,
        "error 5",
    ] {
        assert!(said.contains(part), "{part:?} is not in: {said}");
    }
}

#[test]
fn test_every_refusal_and_a_refused_paste_are_different_sentences() {
    let sentences = [
        a_refusal(None, SessionLock::Locked).to_string(),
        a_refusal(Some(A_HOLDER), SessionLock::Unlocked).to_string(),
        a_refusal(None, SessionLock::Unlocked).to_string(),
        a_refusal(None, SessionLock::NotSaid("nothing".to_string())).to_string(),
        THE_FIELD_REFUSED_A_PASTE.to_string(),
    ];

    assert!(sentences.iter().all(|it| !it.is_empty()), "{sentences:#?}");
    let different: std::collections::HashSet<&String> = sentences.iter().collect();
    assert_eq!(different.len(), sentences.len(), "{sentences:#?}");
}

#[test]
fn test_the_lock_state_is_asked_of_windows_and_answered() {
    let lock = this_sessions_lock();

    assert!(
        matches!(lock, SessionLock::Locked | SessionLock::Unlocked),
        "{lock:?}"
    );
}

// ── Held clipboards and the turn ───────────────────────────────────────────

#[test]
fn test_a_clipboard_held_throughout_is_refused_naming_the_window_that_held_it() {
    let held = match &the_harvest().held_throughout {
        Ok(held) => held,
        Err(why) => panic!("{why}"),
    };

    match &held.reading {
        Err(NoPaste::Refused(refusal)) => {
            println!("{refusal}");
            assert_eq!(refusal.error, ERROR_ACCESS_DENIED, "{refusal}");
            assert_eq!(
                refusal.holder.map(|it| it.window),
                Some(held.holder),
                "{refusal}"
            );
            assert_eq!(refusal.session, SessionLock::Unlocked, "{refusal}");
        }
        other => panic!("a clipboard held throughout was not refused: {other:?}"),
    }
}

#[test]
fn test_a_clipboard_held_briefly_is_waited_for() {
    let held = match &the_harvest().held_briefly {
        Ok(held) => held,
        Err(why) => panic!("{why}"),
    };

    if let Err(why) = &held.reading {
        panic!("a brief hold was not waited out: {why}");
    }
}

#[test]
fn test_a_second_run_waits_its_turn_at_the_clipboard() {
    let events = Arc::new(Mutex::new(Vec::<&'static str>::new()));
    let note = |events: &Arc<Mutex<Vec<&'static str>>>, what: &'static str| {
        if let Ok(mut events) = events.lock() {
            events.push(what);
        }
    };
    let (took_tx, took_rx) = mpsc::channel::<()>();
    let (asked_tx, asked_rx) = mpsc::channel::<()>();

    let first = {
        let events = events.clone();
        std::thread::spawn(move || {
            let turn = OneTurnAtTheClipboard::take();
            let said = if turn.is_ok() {
                "first took"
            } else {
                "first refused"
            };
            note(&events, said);
            let _ = took_tx.send(());
            let _ = asked_rx.recv();
            // Long enough for the second to be waiting; the order below is
            // what is checked, not this time.
            std::thread::sleep(Duration::from_millis(300));
            note(&events, "first let go");
            drop(turn);
        })
    };
    let second = {
        let events = events.clone();
        std::thread::spawn(move || {
            let _ = took_rx.recv();
            note(&events, "second asked");
            let _ = asked_tx.send(());
            let turn = OneTurnAtTheClipboard::take();
            let said = if turn.is_ok() {
                "second took"
            } else {
                "second refused"
            };
            note(&events, said);
        })
    };
    let _ = first.join();
    let _ = second.join();

    let happened = events.lock().map(|it| it.clone()).unwrap_or_default();
    assert_eq!(
        happened,
        vec!["first took", "second asked", "first let go", "second took"]
    );
}

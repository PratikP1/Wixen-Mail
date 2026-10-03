//! A Markdown block marker typed with its space at the start of any line of
//! the message body becomes the structure it stands for (#79, LIST-18).
//!
//! The tester on 2026-09-18, under NVDA, on `1.0.0-alpha.1` at `744d05ef`:
//! typed Markdown made no heading. The probe of the same day, on this
//! machine's WebView2 runtime, made an `<h2>` on the first line of an empty
//! message at every build of the round and found the rule refusing a marker
//! on any line that is not the first: the guard in `blockMarkdown` asked
//! whether the text node had a previous sibling, and every line after the
//! first of a reply, forward or mailto body that arrived as plain text is a
//! text node after a `<br>`, as is a line after Shift+Enter, as is a line
//! after Enter on the empty first line. Refused before the marker was read,
//! with no post, no announcement and no log line, since 2026-07-29.
//!
//! Nothing in `tests/` drove the real editor page with keystrokes before
//! this file: `src/presentation/editor_page_harness.rs` runs the page's
//! recognisers in `boa`, which is where every rule bug so far had been, and
//! its own header says `execCommand` is outside what it models. The guard
//! that failed here is not a recogniser. It is a question about the document
//! the engine built, so only the engine can answer it.
//!
//! **How this drives the page.** It builds the real compose dialog through
//! `wx_compose::build_compose_dialog`, sets the shipped page the way
//! `set_body` does, and delivers each character as a `WM_CHAR` posted to the
//! window the browser gave the keyboard focus, one per timer tick, so the
//! page's `input` event fires once per character the way it does under
//! somebody's fingers. Enter, the arrows and End go as `WM_KEYDOWN` and
//! `WM_KEYUP`. Shift+Enter is the one key a posted message cannot carry,
//! because the engine reads the Shift state from the keyboard and not from
//! the message, so that step runs `insertLineBreak`, the command the key
//! runs, from script. Not `SendInput`: that lands on the foreground window,
//! which is not this dialog when a test runs it, and taking the foreground
//! would take it from whoever is using the machine. So the keys arrive as
//! posted characters and not through a screen reader's hook, and what the
//! tester hears is still his ear's; the ledger holds the by-ear steps.
//!
//! **What it asserts.** The document's `innerHTML`, read back through
//! `editor_document::read_body_script()`, and the messages the page posted,
//! read through `editor_document::parse_message` the way the composer reads
//! them. Never an announcement: the composer's arms have their own tests.
//!
//! **One function.** The budget is one `wxdragon::main` per process
//! (`tests/theme_reach.rs` explains it, and this file spends its one), so
//! every shape is a step inside one run, each asserted on its own, and the
//! failures are gathered rather than stopping at the first, so a red run
//! says which steps were red. The first thing read is how many `input`
//! events the page saw, so a run where no key arrived is told apart from a
//! run where the rule refused; and each character is waited for before the
//! next goes, for the reason `wait_for_the_key` gives.
//!
//! **Where it runs.** In a child of this executable started on a desktop
//! made for the run, where nothing a person types arrives, inside the one
//! turn the targets holding a browser share; the parent passes the test only
//! on the child's own `ok` line for it (13-44.6.3, ledger 761). The section
//! "The child on a desktop of its own" says why and what was measured.
//!
//! Runs under `WIXEN_NO_AUDIO` as CI does, and on a temporary data
//! directory in `WIXEN_MAIL_DATA`, never a person's profile.

#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::{OsStr, c_void};
use std::fmt;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::AsRawHandle;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::common::types::MessageBody;
use wixen_mail::presentation::editor_document::{
    self, EditorMessage, Format, WhyAMarkerWasRefused,
};
use wixen_mail::presentation::wx_compose;
use wxdragon::event::WebViewEvents;
use wxdragon::prelude::*;
use wxdragon::widgets::WebView;

const BODY_ID: &str = "wixen-body";
const TICK_MS: i32 = 30;
/// Ticks before the run says it waited too long and stops: a minute.
const GIVE_UP_AFTER_TICKS: u32 = 2000;
/// Ticks of nothing before a reading, so a posted message has arrived.
const TICKS_TO_SETTLE: u32 = 4;

const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const WM_CHAR: u32 = 0x0102;
const VK_RETURN: u16 = 0x0D;
const VK_END: u16 = 0x23;
const VK_DOWN: u16 = 0x28;

/// A reply to a message that arrived as plain text, the shape
/// `wx_compose::format_reply_body` makes: the separator under two empty
/// lines, the original under it, and every line break a bare `<br>` once
/// `escaped_plain_text` has turned it, with no block round any of them.
fn a_reply_to_plain_text() -> MessageBody {
    MessageBody::Plain("\n\n--- Original Message ---\nTheir words\non two lines".to_string())
}

/// A message holding a link, so that typing at the end of the line lands in
/// a text node after an inline element: the engine keeps typed text out of
/// an anchor, which is the one shape that reliably puts a marker mid-line in
/// a node of its own.
fn a_message_ending_in_a_link() -> MessageBody {
    MessageBody::Html("<p>See <a href=\"https://example.com/\">the page</a></p>".to_string())
}

#[link(name = "user32")]
unsafe extern "system" {
    fn GetFocus() -> isize;
    fn PostMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> i32;
    fn EnumChildWindows(
        parent: isize,
        callback: extern "system" fn(isize, isize) -> i32,
        lparam: isize,
    ) -> i32;
    fn GetClassNameW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
}

/// winuser.h: Shift, Control and Alt, each with its left and right key.
const MODIFIERS: [usize; 9] = [0x10, 0x11, 0x12, 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5];

/// Set this thread's keyboard state with exactly `down` of the modifier keys
/// held and every other key as it was.
///
/// The browser reads a key's modifiers from the keyboard state, and a desktop
/// of the run's own keeps out the keys a person presses but not the ones they
/// hold: measured 2026-10-02 by 13-44.6.2, every one of eight failed runs of
/// two key targets came while the person at the machine held Shift. The keys
/// here are posted and read when the page takes them, so the state is left
/// as set rather than put back.
fn only_these_modifiers_down(down: &[usize]) {
    let mut held = [0u8; 256];
    // SAFETY: the buffer is the 256 bytes the call writes.
    unsafe { GetKeyboardState(held.as_mut_ptr()) };
    for key in MODIFIERS {
        held[key] = 0;
    }
    for &key in down {
        held[key] = 0x80;
    }
    // SAFETY: the buffer is the 256 bytes the call reads, for this thread.
    unsafe { SetKeyboardState(held.as_ptr()) };
}

// ── The child on a desktop of its own ─────────────────────────────────────
//
// A browser's process starts on its process's desktop, not on the desktop of
// the thread that asks for it, so this reading cannot move its window thread
// the way the plain-control readings do: measured 2026-10-02, a copy with its
// thread on a desktop of its own hung until stopped after about nine minutes,
// the page never coming up. So the window test runs in a child of this same
// executable started on a desktop made for the run, where nothing a person
// types arrives, and the parent passes the test only on the child's own `ok`
// line for it. Started whole that way it passed 20 of 20. On the interactive
// desktop it passed 11 of 11 with nobody at the machine and 3 of 9 with
// somebody using it, the failing runs' keys going to the browser's own window
// because the page held no focus once the window was not kept in front.

/// Set in the child's environment to the desktop it runs on; a window test
/// that finds it set runs its body.
const ON_A_DESKTOP_OF_THEIR_OWN: &str = "WIXEN_TESTS_ON_A_DESKTOP_OF_THEIR_OWN";

/// How long the child may run before it is stopped: five times the longest
/// run measured, 55 s with somebody at the machine.
const THE_CHILDS_BOUND_MS: u32 = 5 * 60 * 1000;

/// The turn every run whose window holds a browser takes around its child,
/// and that the real paste target takes at the logon's test clipboard: one
/// kernel object for both, since neither may overlap a second run.
const THE_TURN: &str = "Local\\wixen-mail-tests-one-turn-at-what-runs-share";

/// How long a parent waits for the turn: the bound a run holding it is
/// stopped at, and a minute more.
const LONGEST_WAIT_FOR_THE_TURN_MS: u32 = THE_CHILDS_BOUND_MS + 60_000;

const WAIT_TIMEOUT: u32 = 0x0000_0102;
const WAIT_FAILED: u32 = 0xFFFF_FFFF;
const GENERIC_ALL: u32 = 0x1000_0000;
const UOI_NAME: i32 = 2;
const HANDLE_FLAG_INHERIT: u32 = 0x0000_0001;
const STARTF_USESTDHANDLES: u32 = 0x0000_0100;
const CREATE_UNICODE_ENVIRONMENT: u32 = 0x0000_0400;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const WTS_CURRENT_SESSION: u32 = 0xFFFF_FFFF;
const WTS_SESSION_INFO_EX: u32 = 25;
const WTS_SESSIONSTATE_LOCK: u32 = 0;
const WTS_SESSIONSTATE_UNLOCK: u32 = 1;

/// processthreadsapi.h: `STARTUPINFOW`.
#[repr(C)]
struct StartupInfo {
    size: u32,
    reserved: *mut u16,
    desktop: *mut u16,
    title: *mut u16,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    columns: u32,
    rows: u32,
    fill_attribute: u32,
    flags: u32,
    show_window: u16,
    reserved2_size: u16,
    reserved2: *mut u8,
    std_input: isize,
    std_output: isize,
    std_error: isize,
}

/// processthreadsapi.h: `PROCESS_INFORMATION`.
#[repr(C)]
#[derive(Default)]
struct ProcessInformation {
    process: isize,
    thread: isize,
    process_id: u32,
    thread_id: u32,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn CreateDesktopW(
        name: *const u16,
        device: *const u16,
        mode: *const c_void,
        flags: u32,
        access: u32,
        attributes: *const c_void,
    ) -> isize;
    fn CloseDesktop(desktop: isize) -> i32;
    fn GetProcessWindowStation() -> isize;
    fn GetUserObjectInformationW(
        object: isize,
        index: i32,
        info: *mut u16,
        length: u32,
        needed: *mut u32,
    ) -> i32;
    fn GetKeyboardState(state: *mut u8) -> i32;
    fn SetKeyboardState(state: *const u8) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateMutexW(attributes: *const c_void, initial_owner: i32, name: *const u16) -> isize;
    fn WaitForSingleObject(handle: isize, milliseconds: u32) -> u32;
    fn ReleaseMutex(handle: isize) -> i32;
    fn CloseHandle(handle: isize) -> i32;
    fn GetLastError() -> u32;
    fn SetHandleInformation(handle: isize, mask: u32, flags: u32) -> i32;
    fn CreateProcessW(
        application: *const u16,
        command_line: *mut u16,
        process_attributes: *const c_void,
        thread_attributes: *const c_void,
        inherit_handles: i32,
        creation_flags: u32,
        environment: *const c_void,
        current_directory: *const u16,
        startup: *const StartupInfo,
        information: *mut ProcessInformation,
    ) -> i32;
    fn GetExitCodeProcess(process: isize, code: *mut u32) -> i32;
    fn TerminateProcess(process: isize, code: u32) -> i32;
}

#[link(name = "wtsapi32")]
unsafe extern "system" {
    fn WTSQuerySessionInformationW(
        server: isize,
        session: u32,
        class: u32,
        answer: *mut *mut c_void,
        bytes: *mut u32,
    ) -> i32;
    fn WTSFreeMemory(memory: *mut c_void);
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// What a failed call says, with Windows' error for it.
fn failed(call: &str) -> String {
    // SAFETY: reads this thread's last error and nothing else.
    format!("{call} failed with error {}", unsafe { GetLastError() })
}

/// Whether Windows says this session is locked, asked of Windows and never
/// read from the process list.
#[derive(Debug, Clone, PartialEq)]
enum SessionLock {
    Locked,
    Unlocked,
    NotSaid(String),
}

/// Whether this session is locked, as Windows answers it.
fn this_sessions_lock() -> SessionLock {
    let mut answer: *mut c_void = std::ptr::null_mut();
    let mut bytes = 0u32;
    // SAFETY: Windows writes a buffer it owns into `answer` and its length
    // into `bytes`; the buffer is copied and then freed once.
    unsafe {
        let asked = WTSQuerySessionInformationW(
            0,
            WTS_CURRENT_SESSION,
            WTS_SESSION_INFO_EX,
            &mut answer,
            &mut bytes,
        );
        if asked == 0 || answer.is_null() {
            return SessionLock::NotSaid(format!(
                "WTSQuerySessionInformationW failed with error {}",
                GetLastError()
            ));
        }
        let copied = std::slice::from_raw_parts(answer as *const u8, bytes as usize).to_vec();
        WTSFreeMemory(answer);
        the_lock_in(&copied)
    }
}

/// The lock state in a `WTSINFOEXW` answer: its level in the first four
/// bytes, and the session's flags at byte 16.
fn the_lock_in(answer: &[u8]) -> SessionLock {
    let word = |at: usize| {
        answer
            .get(at..at + 4)
            .map(|four| u32::from_le_bytes([four[0], four[1], four[2], four[3]]))
    };
    match (word(0), word(16)) {
        (Some(1), Some(WTS_SESSIONSTATE_LOCK)) => SessionLock::Locked,
        (Some(1), Some(WTS_SESSIONSTATE_UNLOCK)) => SessionLock::Unlocked,
        (level, flags) => SessionLock::NotSaid(format!(
            "{} bytes, level {level:?}, session flags {flags:?}",
            answer.len()
        )),
    }
}

/// The one turn, held by the parent from before its child starts until the
/// child has ended, and released on the thread that took it.
///
/// WebView2 runs one browser process per user data folder, and the folder the
/// editor is given is named for the executable, so two runs of this target at
/// once share one browser and, through it, one keyboard focus, even on two
/// desktops. Measured on 2026-10-01 with two runs started two seconds apart,
/// five rounds: each page took in the other's letters ("bolHd", "ite#m"), step
/// 8c came out as "<strong>bold</strong><div>-a item</div>", a timing probe
/// typing the same keys came out as `<li><strong>item</strong></li>`, the
/// shape ledger 754 reported, and a run that started while the other held the
/// browser never opened a page. Twenty runs one after another were all green.
/// It is taken in the parent and never in the body the child runs, since the
/// child would otherwise wait on a turn its own parent holds.
struct TheOneTurn(isize);

impl TheOneTurn {
    fn take() -> Result<Self, String> {
        let name = wide(THE_TURN);
        // SAFETY: a null-terminated name and no security attributes.
        let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
        if handle == 0 {
            return Err(failed("CreateMutexW for the one turn"));
        }
        // SAFETY: a live mutex handle. An abandoned one, left by a run that
        // ended while holding it, is taken like a released one.
        let waited = unsafe { WaitForSingleObject(handle, LONGEST_WAIT_FOR_THE_TURN_MS) };
        if waited == WAIT_TIMEOUT || waited == WAIT_FAILED {
            // SAFETY: the handle made above, closed once.
            unsafe { CloseHandle(handle) };
            return Err(format!(
                "another run held the one turn for {} s",
                LONGEST_WAIT_FOR_THE_TURN_MS / 1000
            ));
        }
        Ok(TheOneTurn(handle))
    }
}

impl Drop for TheOneTurn {
    fn drop(&mut self) {
        // SAFETY: the handle this turn took, released on the thread that took it.
        unsafe {
            ReleaseMutex(self.0);
            CloseHandle(self.0);
        }
    }
}

/// A desktop made for this run on the window station the process is on,
/// closed when dropped.
struct ADesktopOfItsOwn {
    handle: isize,
    /// As `CreateProcessW` takes it: `station\desktop`.
    full_name: String,
}

impl ADesktopOfItsOwn {
    fn make(short: &str) -> Result<Self, String> {
        let station = the_window_stations_name()?;
        let name = format!("wixen-{short}-{}", std::process::id());
        // SAFETY: the name is null-terminated and every other pointer is
        // null, which CreateDesktopW takes as "none".
        let handle = unsafe {
            CreateDesktopW(
                wide(&name).as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                GENERIC_ALL,
                std::ptr::null(),
            )
        };
        if handle == 0 {
            return Err(failed("CreateDesktopW"));
        }
        Ok(ADesktopOfItsOwn {
            handle,
            full_name: format!("{station}\\{name}"),
        })
    }
}

impl Drop for ADesktopOfItsOwn {
    fn drop(&mut self) {
        // SAFETY: the handle made in `make`, closed once.
        unsafe { CloseDesktop(self.handle) };
    }
}

/// The name of the window station this process is on, read rather than
/// written, since a runner may not be on `WinSta0`.
fn the_window_stations_name() -> Result<String, String> {
    let mut buffer = [0u16; 256];
    let mut needed = 0u32;
    // SAFETY: the buffer's length is passed in bytes, and the station handle
    // is the process's own, which Windows owns and is never closed here.
    let read = unsafe {
        GetUserObjectInformationW(
            GetProcessWindowStation(),
            UOI_NAME,
            buffer.as_mut_ptr(),
            (buffer.len() * 2) as u32,
            &mut needed,
        )
    };
    if read == 0 {
        return Err(failed("GetUserObjectInformationW for the window station"));
    }
    let end = buffer
        .iter()
        .position(|&unit| unit == 0)
        .unwrap_or(buffer.len());
    Ok(String::from_utf16_lossy(&buffer[..end]))
}

/// This process's environment with `ON_A_DESKTOP_OF_THEIR_OWN` set to
/// `desktop`, as the block `CreateProcessW` takes.
fn the_childs_environment(desktop: &str) -> Vec<u16> {
    let mut block = Vec::new();
    let ours = OsStr::new(ON_A_DESKTOP_OF_THEIR_OWN);
    let theirs = std::env::vars_os().filter(|(key, _)| key.as_os_str() != ours);
    for (key, value) in theirs.chain(std::iter::once((ours.into(), desktop.into()))) {
        block.extend(key.encode_wide());
        block.push(u16::from(b'='));
        block.extend(value.encode_wide());
        block.push(0);
    }
    block.push(0);
    block
}

/// Run this executable whole, with no filter, on a desktop made for the run,
/// and say how it ended.
fn the_child_run(short: &str) -> ChildRun {
    let started = std::time::Instant::now();
    let desktop = ADesktopOfItsOwn::make(short);
    let full_name = desktop
        .as_ref()
        .map_or_else(|_| format!("wixen-{short}"), |it| it.full_name.clone());
    let (end, output) = desktop
        .and_then(|desktop| run_in_the_one_turn(&desktop))
        .unwrap_or_else(|why| (ChildEnd::NeverStarted(why), String::new()));
    say(&format!(
        "the child ran on {full_name} for {} s and {end}",
        started.elapsed().as_secs()
    ));
    ChildRun {
        desktop: full_name,
        end,
        output,
        lock: this_sessions_lock(),
    }
}

/// Take the one turn, run the child on `desktop` with its output written to
/// a file, let the turn go once it has ended, and hand back how it ended and
/// what it wrote.
fn run_in_the_one_turn(desktop: &ADesktopOfItsOwn) -> Result<(ChildEnd, String), String> {
    let folder = tempfile::tempdir().map_err(|why| format!("the child's output folder: {why}"))?;
    let output = folder.path().join("child.txt");
    let turn = TheOneTurn::take()?;
    let end = run_and_wait(desktop, &output);
    drop(turn);
    let said = std::fs::read(&output)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default();
    Ok((end?, said))
}

/// Start the child on `desktop` with its output written to `output`, wait at
/// most `THE_CHILDS_BOUND_MS`, and say how it ended.
fn run_and_wait(desktop: &ADesktopOfItsOwn, output: &Path) -> Result<ChildEnd, String> {
    let file = std::fs::File::create(output).map_err(|why| format!("the child's output: {why}"))?;
    let handle = file.as_raw_handle() as isize;
    // SAFETY: a live file handle this function owns; only its flag changes.
    if unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) } == 0 {
        return Err(failed("SetHandleInformation"));
    }
    let me = std::env::current_exe().map_err(|why| format!("this test's own path: {why}"))?;
    let mut command_line = wide(&format!("\"{}\"", me.display()));
    let mut desktop_name = wide(&desktop.full_name);
    let environment = the_childs_environment(&desktop.full_name);
    let startup = StartupInfo {
        size: std::mem::size_of::<StartupInfo>() as u32,
        reserved: std::ptr::null_mut(),
        desktop: desktop_name.as_mut_ptr(),
        title: std::ptr::null_mut(),
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        columns: 0,
        rows: 0,
        fill_attribute: 0,
        flags: STARTF_USESTDHANDLES,
        show_window: 0,
        reserved2_size: 0,
        reserved2: std::ptr::null_mut(),
        std_input: 0,
        std_output: handle,
        std_error: handle,
    };
    let mut child = ProcessInformation::default();
    // SAFETY: every string is null-terminated and outlives the call, the
    // environment block ends in two nulls, and Windows writes `child`.
    let made = unsafe {
        CreateProcessW(
            std::ptr::null(),
            command_line.as_mut_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            1,
            CREATE_NO_WINDOW | CREATE_UNICODE_ENVIRONMENT,
            environment.as_ptr() as *const c_void,
            std::ptr::null(),
            &startup,
            &mut child,
        )
    };
    if made == 0 {
        return Err(failed("CreateProcessW"));
    }
    drop(file);
    let ended = the_end_of(&child);
    // SAFETY: the two handles CreateProcessW handed back, closed once.
    unsafe {
        CloseHandle(child.thread);
        CloseHandle(child.process);
    }
    ended
}

/// Wait for the child at most `THE_CHILDS_BOUND_MS`, stopping it there.
fn the_end_of(child: &ProcessInformation) -> Result<ChildEnd, String> {
    // SAFETY: a live process handle.
    match unsafe { WaitForSingleObject(child.process, THE_CHILDS_BOUND_MS) } {
        WAIT_TIMEOUT => {
            // SAFETY: a live process handle; the wait lets it finish going.
            unsafe {
                TerminateProcess(child.process, 1);
                WaitForSingleObject(child.process, 10_000);
            }
            Ok(ChildEnd::StoppedAtTheBound)
        }
        WAIT_FAILED => Err(failed("WaitForSingleObject on the child")),
        _ => {
            let mut code = 0u32;
            // SAFETY: a live process handle that has ended.
            if unsafe { GetExitCodeProcess(child.process, &mut code) } == 0 {
                return Err(failed("GetExitCodeProcess"));
            }
            Ok(ChildEnd::Exited(code))
        }
    }
}

/// Whether this window test was run in a child on a desktop made for the
/// run, which is so in the parent; there it fails, with one sentence saying
/// why, unless the child's line for it says ok. In the child it answers
/// false and the test runs its body, and on a runner it says so and answers
/// false, and the test runs its body here.
fn ran_in_a_child_on_a_desktop_of_its_own(test: &str) -> bool {
    if std::env::var_os(ON_A_DESKTOP_OF_THEIR_OWN).is_some() {
        return false;
    }
    if where_the_body_runs(std::env::var_os("CI").as_deref())
        == WhereTheBodyRuns::OnTheRunnersOwnDesktop
    {
        say_past_the_capture(ON_THE_RUNNERS_OWN_DESKTOP);
        return false;
    }
    static THE_CHILD: OnceLock<ChildRun> = OnceLock::new();
    let run = THE_CHILD.get_or_init(|| the_child_run("marker"));
    if let Err(why) = what_the_child_said(test, run) {
        panic!("{why}");
    }
    true
}

/// Where a window test's body runs.
#[derive(Debug, Clone, Copy, PartialEq)]
enum WhereTheBodyRuns {
    /// In a child of this executable on a desktop made for the run.
    InAChild,
    /// In this process, on the desktop it was started on.
    OnTheRunnersOwnDesktop,
}

/// Where a window test's body runs, given the `CI` variable, which GitHub
/// sets on every runner.
///
/// On the runner every process is elevated, and the browser an elevated
/// child asks for is started by `explorer.exe`, unelevated and on the
/// shell's desktop, so its window cannot be made inside the child's: every
/// run there ended "Invalid window handle" with the page never coming (ledger
/// 785; CI runs 37151597179 and 37154501219 named the starter, and asking the
/// browser not to de-elevate changed nothing). Nobody types on a runner, so
/// there the body runs here, as it did before 13-44.6.3, and says so.
fn where_the_body_runs(ci: Option<&OsStr>) -> WhereTheBodyRuns {
    match ci {
        Some(_) => WhereTheBodyRuns::OnTheRunnersOwnDesktop,
        None => WhereTheBodyRuns::InAChild,
    }
}

/// What a run whose body runs on the runner's own desktop says, once.
const ON_THE_RUNNERS_OWN_DESKTOP: &str = "the window test ran on the runner's own desktop, not \
     in a child on a desktop of its own, because CI is set: nobody types on a runner, and there \
     an elevated child's browser opens on the shell's desktop and cannot open in the child's \
     (ledger 785)";

/// Write `line` to standard error itself, past libtest's capture, so a
/// runner's log carries it whatever the verdict.
fn say_past_the_capture(line: &str) {
    use std::io::Write;
    let mut stderr = std::io::stderr().lock();
    // A line that cannot be written is a sentence lost, not a failure.
    let _ = writeln!(stderr, "{line}");
}

/// The one turn, for a body that runs in this process on the runner's own
/// desktop; in the child the parent holds it already.
fn the_turn_for_a_body_run_here() -> Result<Option<TheOneTurn>, String> {
    if std::env::var_os(ON_A_DESKTOP_OF_THEIR_OWN).is_some() {
        return Ok(None);
    }
    TheOneTurn::take().map(Some)
}

/// How the child run ended.
#[derive(Debug, Clone, PartialEq)]
enum ChildEnd {
    /// It ended by itself with this exit code.
    Exited(u32),
    /// It ran past its five minutes and was stopped.
    StoppedAtTheBound,
    /// It never started, for this reason.
    NeverStarted(String),
}

impl fmt::Display for ChildEnd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChildEnd::Exited(code) => write!(f, "exited with exit code {code:#x}"),
            ChildEnd::StoppedAtTheBound => {
                write!(f, "did not finish in five minutes and was stopped")
            }
            ChildEnd::NeverStarted(why) => write!(f, "never started: {why}"),
        }
    }
}

impl fmt::Display for SessionLock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionLock::Locked => write!(f, "Windows says the session is locked."),
            SessionLock::Unlocked => write!(f, "Windows says the session is unlocked."),
            SessionLock::NotSaid(answer) => write!(
                f,
                "Windows did not say whether the session is locked: asking answered {answer}."
            ),
        }
    }
}

/// What the child run left for the parent to judge each window test by.
#[derive(Debug, Clone, PartialEq)]
struct ChildRun {
    /// The desktop it ran on, as `station\desktop`.
    desktop: String,
    end: ChildEnd,
    /// Its standard output and error, as libtest wrote them.
    output: String,
    /// What Windows said about the session lock once the child had ended.
    lock: SessionLock,
}

/// How many lines of the child's output a sentence quotes when the test's
/// own failure block is not there to quote.
const LINES_QUOTED_FROM_THE_END: usize = 20;

/// Whether the child's run says `test` passed, and if not, one sentence
/// saying what happened instead, ending with what Windows says about the
/// lock.
fn what_the_child_said(test: &str, run: &ChildRun) -> Result<(), String> {
    let line = |verdict: &str| format!("test {test} ... {verdict}");
    let said = |line: &str| run.output.lines().any(|it| it.trim_end() == line);
    let where_ = format!("the child run on {}", run.desktop);
    let lock = &run.lock;
    if said(&line("ok")) {
        return Ok(());
    }
    if said(&line("FAILED")) {
        let block = the_failure_block(test, &run.output).unwrap_or_else(|| {
            format!(
                "(no failure block for it; its output ends: {})",
                the_end_of_the_output(&run.output)
            )
        });
        return Err(format!("{test} failed in {where_}:\n{block}\n{lock}"));
    }
    match &run.end {
        ChildEnd::StoppedAtTheBound => Err(format!(
            "{where_} {}, so {test} has no result; its output ends:\n{}\n{lock}",
            run.end,
            the_end_of_the_output(&run.output)
        )),
        end => Err(format!(
            "{where_} {end} and never reported {test}; its output ends:\n{}\n{lock}",
            the_end_of_the_output(&run.output)
        )),
    }
}

/// The block libtest writes for a failed test: from its `---- NAME stdout
/// ----` line to the next such line or the list of failures after them.
fn the_failure_block(test: &str, output: &str) -> Option<String> {
    let header = format!("---- {test} stdout ----");
    let mut lines = output.lines().skip_while(|line| line.trim_end() != header);
    let first = lines.next()?;
    let rest =
        lines.take_while(|line| !line.starts_with("---- ") && line.trim_end() != "failures:");
    let block: Vec<&str> = std::iter::once(first).chain(rest).collect();
    Some(block.join("\n").trim_end().to_string())
}

/// The last lines of the child's output, or a sentence saying it wrote none.
fn the_end_of_the_output(output: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    if lines.is_empty() {
        return "(it wrote nothing)".to_string();
    }
    let from = lines.len().saturating_sub(LINES_QUOTED_FROM_THE_END);
    lines[from..].join("\n")
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

/// The window a key is posted to: the one this thread has the keyboard
/// focus on, which the browser sets when the page takes focus, or failing
/// that the browser's own top window under the control.
fn where_keys_go(body_editor: &WebView) -> isize {
    // SAFETY: a plain read.
    let focus = unsafe { GetFocus() };
    if focus != 0 {
        return focus;
    }
    descendants_of(body_editor.get_handle() as isize)
        .into_iter()
        .find(|hwnd| class_name(*hwnd) == "Chrome_WidgetWin_1")
        .unwrap_or(0)
}

fn describe(hwnd: isize) -> String {
    if hwnd == 0 {
        return "no window".to_string();
    }
    format!("0x{hwnd:x} ({})", class_name(hwnd))
}

fn post_char(hwnd: isize, ch: char) {
    only_these_modifiers_down(&[]);
    // SAFETY: a live window handle; the message carries a character.
    unsafe {
        PostMessageW(hwnd, WM_CHAR, ch as usize, 1);
    }
}

fn post_key(hwnd: isize, vk: u16) {
    only_these_modifiers_down(&[]);
    // SAFETY: a live window handle; the messages carry a virtual key.
    unsafe {
        PostMessageW(hwnd, WM_KEYDOWN, vk as usize, 1);
        PostMessageW(hwnd, WM_KEYUP, vk as usize, 0xC000_0001_u32 as i32 as isize);
    }
}

/// The wx result of a string expression is JSON, so quoted.
fn unquoted(answer: Option<String>) -> String {
    match answer {
        Some(text) => text.trim().trim_matches('"').to_string(),
        None => "(run_script answered None)".to_string(),
    }
}

fn script_with_body(rest: &str) -> String {
    format!("(function () {{ var b = document.getElementById({BODY_ID:?}); {rest} }})()")
}

/// What the page held and what it posted, read for one step's assertions.
struct Seen {
    /// The body's `innerHTML`.
    html: String,
    /// What the page posted since the last clearing, as the composer reads
    /// it, in order.
    posts: Vec<EditorMessage>,
    /// The raw text of every post since the last clearing, for the message
    /// when an assertion fails.
    raw: Vec<String>,
    /// How many `input` events the page saw since it was opened.
    inputs: usize,
}

impl Seen {
    fn formatted(&self, format: Format) -> bool {
        self.posts.contains(&EditorMessage::Formatted(format))
    }

    fn any_format_post(&self) -> bool {
        self.posts
            .iter()
            .any(|post| matches!(post, EditorMessage::Formatted(_)))
    }

    fn styled_with(&self, delimiter: &str) -> bool {
        self.posts.iter().any(
            |post| matches!(post, EditorMessage::Styled(style) if style.delimiter == delimiter),
        )
    }

    fn refused(&self, why: WhyAMarkerWasRefused) -> bool {
        self.posts.contains(&EditorMessage::BlockMarkerRefused(why))
    }

    /// What one element holds, for an element that occurs once.
    fn inside(&self, tag: &str) -> Option<&str> {
        let open = format!("<{tag}>");
        let close = format!("</{tag}>");
        let start = self.html.find(&open)? + open.len();
        let end = self.html[start..].find(&close)? + start;
        Some(&self.html[start..end])
    }
}

/// One thing the run does on one tick.
enum Act {
    /// The name of the step every failure until the next `Step` is filed
    /// under.
    Step(&'static str),
    /// Load a body into the editor and wait for its page to be up and
    /// instrumented.
    Open(MessageBody),
    /// One character, as the person types it.
    Char(char),
    /// One key that is not a character.
    Key(u16),
    /// A script run in the page, for the one key a posted message cannot
    /// carry.
    Script(&'static str),
    /// Forget what was posted so far, so the next reading is one step's.
    ClearPosts,
    /// Ticks of nothing, so the last key's post has arrived before a
    /// reading.
    Settle,
    /// Read the body and the posts and assert on them.
    Check(fn(&Seen) -> Result<(), String>),
}

fn typed(text: &str) -> Vec<Act> {
    text.chars().map(Act::Char).collect()
}

/// A reading, after the page has settled.
fn checked(assertion: fn(&Seen) -> Result<(), String>) -> [Act; 2] {
    [Act::Settle, Act::Check(assertion)]
}

/// Where the run is between ticks.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    WaitingForBrowser,
    /// The page was set; waiting for its script to be up, then instrumenting
    /// it and putting the focus in the body.
    WaitingForPage,
    /// Ticks of nothing before the next act, so posts have arrived.
    Settling(u32),
    /// A character was posted; waiting for the page's `input` event for it
    /// before the next key goes, and posting it again if it never comes.
    WaitingForInput {
        ch: char,
        /// How many `input` events the page will have seen once this one
        /// lands.
        expected: usize,
        /// The tick it was posted on, so a character the page never saw is
        /// told from one it has not seen yet.
        posted_at: u32,
        /// How many times it has been posted.
        tries: u32,
    },
    Acting,
    Done,
}

/// Ticks to wait for a character's `input` event before posting it again:
/// a second, which is many times longer than the engine takes when it is
/// not dropping the key.
const TICKS_BEFORE_A_KEY_IS_POSTED_AGAIN: u32 = 33;
/// How many times one character is posted before the step gives up on it.
const TRIES_PER_KEY: u32 = 4;

struct Run {
    acts: Vec<Act>,
    next: usize,
    phase: Phase,
    ticks: u32,
    posted: Vec<String>,
    failures: Vec<String>,
    step: &'static str,
    /// Where each key of the current step went, as the window's class, so a
    /// step that lost a key says which window it was posted to.
    delivered: Vec<String>,
    /// A tick is running. `run_script` waits through `wxYield`, which
    /// delivers the next `WM_TIMER` into the middle of this one; a nested
    /// tick returns at once rather than running the same act twice.
    busy: bool,
}

fn the_steps() -> Vec<Act> {
    let mut acts = Vec::new();

    // 1. The first line of an empty message: the calibration case, green
    // at every build. A run where this fails with no input events seen is
    // a run where no key arrived, and says so.
    acts.push(Act::Step("1: ## on the first line of an empty message"));
    acts.push(Act::Open(MessageBody::Plain(String::new())));
    acts.extend(typed("## Heading"));
    acts.extend(checked(|seen| {
        if seen.inputs == 0 {
            return Err(
                "the page saw no input event at all, so no key reached it: a delivery \
                 failure, not a rule failure"
                    .to_string(),
            );
        }
        expect(
            seen.inside("h2") == Some("Heading"),
            seen,
            "an <h2> holding Heading",
        )?;
        expect(
            seen.formatted(Format::Heading2),
            seen,
            "a format post for Heading level 2",
        )
    }));

    // 2. A line after a line break in a reply that arrived as plain text:
    // the case the tester was most likely in. Down from the first line
    // lands between the two <br>s above the separator.
    acts.push(Act::Step(
        "2: ## on a line after a <br> in a reply to plain text",
    ));
    acts.push(Act::Open(a_reply_to_plain_text()));
    acts.push(Act::Key(VK_DOWN));
    acts.extend(typed("## Heading"));
    acts.extend(checked(|seen| {
        expect(
            seen.inside("h2") == Some("Heading"),
            seen,
            "an <h2> holding Heading alone",
        )?;
        expect(
            seen.html.contains("--- Original Message ---")
                && seen.html.contains("Their words<br>on two lines"),
            seen,
            "the quoted original still there and still text",
        )?;
        expect(
            seen.formatted(Format::Heading2),
            seen,
            "a format post for Heading level 2",
        )
    }));

    // 3. Enter on the empty first line of the same reply, then the marker.
    acts.push(Act::Step(
        "3: ## after Enter on the empty first line of a reply",
    ));
    acts.push(Act::Open(a_reply_to_plain_text()));
    acts.push(Act::Key(VK_RETURN));
    acts.extend(typed("## x"));
    acts.extend(checked(|seen| {
        expect(seen.inside("h2") == Some("x"), seen, "an <h2> holding x")?;
        expect(
            seen.formatted(Format::Heading2),
            seen,
            "a format post for Heading level 2",
        )
    }));

    // 4. A word, Shift+Enter, then the marker, in a new message.
    acts.push(Act::Step("4: ## after Shift+Enter in a new message"));
    acts.push(Act::Open(MessageBody::Plain(String::new())));
    acts.extend(typed("Hello"));
    acts.push(Act::Script(
        "document.execCommand('insertLineBreak'); 'broke the line'",
    ));
    acts.extend(typed("## x"));
    acts.extend(checked(|seen| {
        expect(seen.inside("h2") == Some("x"), seen, "an <h2> holding x")?;
        expect(
            seen.html.contains("Hello") && !seen.html.contains("<h2>Hello"),
            seen,
            "Hello still there and outside the heading",
        )?;
        expect(
            seen.formatted(Format::Heading2),
            seen,
            "a format post for Heading level 2",
        )
    }));

    // 5. No space after the marker: not a marker, and never was. Pins the
    // space as the trigger so a later change cannot widen the rule.
    acts.push(Act::Step("5: ## with no space after it"));
    acts.push(Act::Open(MessageBody::Plain(String::new())));
    acts.extend(typed("##Heading "));
    acts.extend(checked(|seen| {
        expect(
            seen.html.contains("##Heading") && !seen.html.contains("<h2"),
            seen,
            "the text left as it was typed",
        )?;
        expect(!seen.any_format_post(), seen, "no format post")?;
        expect(
            !seen.refused(WhyAMarkerWasRefused::NotAtTheStartOfItsLine),
            seen,
            "no refusal either, since nothing was a marker",
        )
    }));

    // 6. After step 2's heading: bold and a bullet on fresh lines, so the
    // inline rule and the list rule are read on the real engine too.
    acts.push(Act::Step(
        "6: **bold** and - item on fresh lines after a heading",
    ));
    acts.push(Act::Open(a_reply_to_plain_text()));
    acts.push(Act::Key(VK_DOWN));
    acts.extend(typed("## Heading"));
    acts.push(Act::Key(VK_RETURN));
    acts.push(Act::ClearPosts);
    acts.extend(typed("**bold** "));
    acts.push(Act::Key(VK_RETURN));
    acts.extend(typed("- item"));
    acts.extend(checked(|seen| {
        expect(
            seen.html.contains("<strong>bold"),
            seen,
            "a <strong> holding bold",
        )?;
        expect(
            seen.html.contains("<ul><li>") && seen.html.contains("item</"),
            seen,
            "a bulleted list holding item",
        )?;
        expect(seen.styled_with("**"), seen, "a style post for Bold")?;
        expect(
            seen.formatted(Format::BulletList),
            seen,
            "a format post for Bulleted list",
        )
    }));

    // 7. What a refusal posts, and what an ordinary space does not. A
    // marker after words on the same line is not a marker, and posts
    // nothing: every ordinary space would otherwise post. A marker in a
    // node of its own that is not at the start of its line is a marker the
    // rule refused, and says so on the wire so the log can say which path.
    acts.push(Act::Step(
        "7a: ## after words on the same line posts nothing",
    ));
    acts.push(Act::Open(a_reply_to_plain_text()));
    acts.push(Act::Key(VK_DOWN));
    acts.extend(typed("Their ## "));
    acts.extend(checked(|seen| {
        expect(
            seen.html.contains("Their ## ") || seen.html.contains("Their ##&nbsp;"),
            seen,
            "the words left as typed",
        )?;
        expect(
            seen.raw.iter().all(|raw| raw.contains("\"word\"")),
            seen,
            "no post but the words",
        )
    }));
    acts.push(Act::Step(
        "7b: ## in a node of its own after a link posts the refusal",
    ));
    acts.push(Act::Open(a_message_ending_in_a_link()));
    acts.push(Act::Key(VK_END));
    acts.extend(typed("## "));
    acts.extend(checked(|seen| {
        expect(
            !seen.html.contains("<h2"),
            seen,
            "no heading made of a marker mid-line",
        )?;
        expect(!seen.any_format_post(), seen, "no format post")?;
        expect(
            seen.refused(WhyAMarkerWasRefused::NotAtTheStartOfItsLine),
            seen,
            "a refused post saying the marker was not at the start of its line",
        )
    }));

    // 8. Text after a closing inline delimiter is plain. The probe of
    // 2026-09-18 found the caret continuing the <strong> the applier had
    // made, so the space and every line after it stayed bold; the applier
    // has to release the style as well as move the caret, and a code span
    // has no command to release it with, so it is read as well.
    acts.push(Act::Step("8a: the word after **bold** is not bold"));
    acts.push(Act::Open(MessageBody::Plain(String::new())));
    acts.extend(typed("**bold** next"));
    acts.extend(checked(|seen| {
        expect(
            seen.inside("strong") == Some("bold"),
            seen,
            "a <strong> holding bold alone",
        )?;
        expect(
            seen.html.contains("</strong>") && seen.html.ends_with("next"),
            seen,
            "next outside the <strong>, at the end",
        )?;
        expect(seen.styled_with("**"), seen, "a style post for Bold")
    }));
    acts.push(Act::Step("8b: the word after `code` is not code"));
    acts.push(Act::Open(MessageBody::Plain(String::new())));
    acts.extend(typed("`code` after"));
    acts.extend(checked(|seen| {
        expect(
            seen.inside("code") == Some("code"),
            seen,
            "a <code> holding code alone",
        )?;
        expect(
            seen.html.contains("</code>") && seen.html.ends_with("after"),
            seen,
            "after outside the <code>, at the end",
        )?;
        expect(seen.styled_with("`"), seen, "a style post for Code")
    }));

    // 9. A list marker as the first thing in an empty message. July's note
    // on `block()` says `insertUnorderedList` did nothing on an empty root
    // from the menu; the typed rule is measured here.
    acts.push(Act::Step(
        "9: - item as the first thing in an empty message",
    ));
    acts.push(Act::Open(MessageBody::Plain(String::new())));
    acts.extend(typed("- item"));
    acts.extend(checked(|seen| {
        expect(
            seen.inside("li") == Some("item"),
            seen,
            "a list item holding item",
        )?;
        expect(
            seen.formatted(Format::BulletList),
            seen,
            "a format post for Bulleted list",
        )
    }));

    // 8c. Enter right after the closing delimiter: the engine carries the
    // style onto the new line, which is the "every line after it" half of
    // the side finding, so the first thing typed there has to be plain too.
    acts.push(Act::Step(
        "8c: the line after **bold** and Enter is not bold",
    ));
    acts.push(Act::Open(MessageBody::Plain(String::new())));
    acts.extend(typed("**bold**"));
    acts.push(Act::Key(VK_RETURN));
    acts.extend(typed("- item"));
    acts.extend(checked(|seen| {
        expect(
            seen.inside("strong") == Some("bold"),
            seen,
            "a <strong> holding bold alone",
        )?;
        expect(
            seen.inside("li") == Some("item"),
            seen,
            "a list item holding item with no style round it",
        )
    }));

    acts
}

fn expect(held: bool, seen: &Seen, what: &str) -> Result<(), String> {
    if held {
        Ok(())
    } else {
        Err(format!(
            "expected {what}; the body holds {:?} and the page posted {:?}",
            seen.html, seen.raw
        ))
    }
}

fn say(line: &str) {
    println!("{line}");
}

#[test]
fn test_a_marker_typed_at_the_start_of_any_line_makes_its_structure() {
    if ran_in_a_child_on_a_desktop_of_its_own(
        "test_a_marker_typed_at_the_start_of_any_line_makes_its_structure",
    ) {
        return;
    }
    let _turn = the_turn_for_a_body_run_here().unwrap_or_else(|why| panic!("{why}"));
    let data_dir = tempfile::tempdir().expect("a temporary data directory");
    // SAFETY: set before any thread is started and before anything reads it.
    unsafe {
        std::env::set_var("WIXEN_MAIL_DATA", data_dir.path());
        std::env::set_var("WIXEN_NO_AUDIO", "1");
    }

    let outcome: Arc<Mutex<Result<(), Vec<String>>>> = Arc::new(Mutex::new(Ok(())));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let frame = Frame::builder().build();
            let widgets = wx_compose::build_compose_dialog(
                &frame,
                "Compose New Message",
                &[wixen_mail::application::identities::FromEntry {
                    account_id: "an-account".to_string(),
                    address: "person@example.com".to_string(),
                    sender_name: String::new(),
                    said: "person@example.com".to_string(),
                }],
                0,
                None,
            );
            let dialog = widgets.dialog;
            let body_editor = widgets.body_editor;
            let browser = widgets.browser.clone();
            let run = Rc::new(RefCell::new(Run {
                acts: the_steps(),
                next: 0,
                phase: Phase::WaitingForBrowser,
                ticks: 0,
                posted: Vec::new(),
                failures: Vec::new(),
                step: "before the first step",
                delivered: Vec::new(),
                busy: false,
            }));

            body_editor.on_script_message_received({
                let run = run.clone();
                move |event| {
                    if let Some(raw) = event.get_string() {
                        run.borrow_mut().posted.push(raw);
                    }
                }
            });

            dialog.show(true);

            let ticker = Rc::new(Timer::new(&dialog));
            ticker.on_tick({
                let run = run.clone();
                let outcome = outcome.clone();
                let browser = browser.clone();
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
                            let step = run.step;
                            run.failures
                                .push(format!("{step}: the run waited too long and gave up"));
                            run.phase = Phase::Done;
                            run.next = run.acts.len();
                        }
                        run.phase
                    };
                    let next = match phase {
                        Phase::WaitingForBrowser => {
                            if browser.is_ready() {
                                Phase::Acting
                            } else {
                                phase
                            }
                        }
                        Phase::WaitingForPage => {
                            if the_page_is_up(&body_editor) {
                                instrument(&body_editor);
                                body_editor.set_focus();
                                Phase::Settling(TICKS_TO_SETTLE)
                            } else {
                                phase
                            }
                        }
                        Phase::Settling(left) => {
                            if left > 1 {
                                Phase::Settling(left - 1)
                            } else {
                                Phase::Acting
                            }
                        }
                        Phase::WaitingForInput {
                            ch,
                            expected,
                            posted_at,
                            tries,
                        } => wait_for_the_key(&run, &body_editor, ch, expected, posted_at, tries),
                        Phase::Acting => one_act(&run, &body_editor),
                        Phase::Done => Phase::Done,
                    };
                    let finished = next == Phase::Done && phase != Phase::Done;
                    {
                        let mut run = run.borrow_mut();
                        run.phase = next;
                        run.busy = false;
                    }
                    if finished {
                        let failures = std::mem::take(&mut run.borrow_mut().failures);
                        *outcome.lock().unwrap() = if failures.is_empty() {
                            Ok(())
                        } else {
                            Err(failures)
                        };
                        // Stopped before its owner goes: a timer firing on a
                        // destroyed dialog is an access violation.
                        ticker.stop();
                        browser.destroy_when_ready(dialog);
                        wxdragon::call_after(Box::new(move || {
                            app.exit_main_loop();
                        }));
                    }
                }
            });
            ticker.start(TICK_MS, false);
            // Dropping the last handle destroys the timer, and it must
            // outlive on_init. The tick closure holds the other handle.
            std::mem::forget(ticker);
        })
    };
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");
    let outcome = outcome.lock().unwrap();
    if let Err(failures) = &*outcome {
        panic!(
            "{} step(s) did not hold on the real page:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}

/// Whether the shipped page's script is up and it is a page this run has
/// not instrumented yet, which is how a fresh page is told from the one
/// before it after `set_page`.
fn the_page_is_up(body_editor: &WebView) -> bool {
    unquoted(body_editor.run_script(
        "(typeof window.wixenRules === 'object' && !window.__inputs_seen) ? 'up' : 'not yet'",
    )) == "up"
}

/// Count the page's `input` events, so a run where no key arrived is told
/// apart from one where a rule refused.
fn instrument(body_editor: &WebView) {
    let _ = body_editor.run_script(&script_with_body(
        "window.__inputs_seen = 0; \
         b.addEventListener('input', function () { window.__inputs_seen++; }); \
         return 'instrumented';",
    ));
}

/// How many `input` events the page has seen since it was instrumented.
fn inputs_seen(body_editor: &WebView) -> usize {
    unquoted(body_editor.run_script("String(window.__inputs_seen || 0)"))
        .parse()
        .unwrap_or(0)
}

/// Post one character and wait for the page to see it.
fn post_a_char(
    run: &Rc<RefCell<Run>>,
    body_editor: &WebView,
    ch: char,
    expected: usize,
    tries: u32,
) -> Phase {
    let target = where_keys_go(body_editor);
    let posted_at = {
        let mut run = run.borrow_mut();
        let went = if tries == 1 {
            format!("{ch:?} to {}", describe(target))
        } else {
            let again = format!("{ch:?} again, try {tries}, to {}", describe(target));
            say(&format!("POSTED AGAIN {again}"));
            again
        };
        run.delivered.push(went);
        run.ticks
    };
    post_char(target, ch);
    Phase::WaitingForInput {
        ch,
        expected,
        posted_at,
        tries,
    }
}

/// Whether the character posted has reached the page, and what to do when
/// it has not.
///
/// Posted one per tick with nothing waited for, the engine dropped a
/// character now and then in the tenth of a second after it had rebuilt the
/// line: measured on 2026-09-20, three runs in ten lost two or three
/// characters right after an Enter or right after a marker had made its
/// block, with every key posted to the same window. A person's keys go
/// through the input method's own queue and a posted message does not. So
/// each character is waited for before the next goes, which was twenty runs
/// green with no character posted twice; and one the page has not seen
/// after a second is posted again, with a line saying so, so a run that
/// needed it is not read as a clean one.
fn wait_for_the_key(
    run: &Rc<RefCell<Run>>,
    body_editor: &WebView,
    ch: char,
    expected: usize,
    posted_at: u32,
    tries: u32,
) -> Phase {
    if inputs_seen(body_editor) >= expected {
        return Phase::Acting;
    }
    let now = run.borrow().ticks;
    if now - posted_at < TICKS_BEFORE_A_KEY_IS_POSTED_AGAIN {
        return Phase::WaitingForInput {
            ch,
            expected,
            posted_at,
            tries,
        };
    }
    if tries >= TRIES_PER_KEY {
        let step = run.borrow().step;
        run.borrow_mut().failures.push(format!(
            "{step}: the page never saw {ch:?} after {tries} postings, so the step's keys \
             did not all arrive"
        ));
        return Phase::Acting;
    }
    post_a_char(run, body_editor, ch, expected, tries + 1)
}

fn read(run: &Rc<RefCell<Run>>, body_editor: &WebView) -> Seen {
    let html = unquoted(body_editor.run_script(&editor_document::read_body_script()));
    let inputs = inputs_seen(body_editor);
    let raw = run.borrow().posted.clone();
    let posts = raw
        .iter()
        .filter_map(|raw| editor_document::parse_message(raw))
        .collect();
    Seen {
        html,
        posts,
        raw,
        inputs,
    }
}

/// One act, and the phase after it.
fn one_act(run: &Rc<RefCell<Run>>, body_editor: &WebView) -> Phase {
    let index = run.borrow().next;
    if index >= run.borrow().acts.len() {
        return Phase::Done;
    }
    run.borrow_mut().next = index + 1;
    // The act is borrowed for the match and released before anything that
    // borrows the run again.
    let act = std::mem::replace(&mut run.borrow_mut().acts[index], Act::ClearPosts);
    match act {
        Act::Step(name) => {
            say(&format!("STEP {name}"));
            let mut run = run.borrow_mut();
            run.step = name;
            run.delivered.clear();
            Phase::Acting
        }
        Act::Open(body) => {
            run.borrow_mut().posted.clear();
            body_editor.set_page(&editor_document::editor_document(&body, "en", true), "");
            Phase::WaitingForPage
        }
        Act::Char(ch) => {
            let expected = inputs_seen(body_editor) + 1;
            post_a_char(run, body_editor, ch, expected, 1)
        }
        Act::Key(vk) => {
            let target = where_keys_go(body_editor);
            run.borrow_mut()
                .delivered
                .push(format!("key 0x{vk:02x} to {}", describe(target)));
            post_key(target, vk);
            Phase::Settling(TICKS_TO_SETTLE)
        }
        Act::Script(script) => {
            let _ = body_editor.run_script(script);
            Phase::Settling(TICKS_TO_SETTLE)
        }
        Act::ClearPosts => {
            run.borrow_mut().posted.clear();
            Phase::Acting
        }
        Act::Settle => Phase::Settling(TICKS_TO_SETTLE),
        Act::Check(assertion) => {
            let seen = read(run, body_editor);
            let step = run.borrow().step;
            match assertion(&seen) {
                Ok(()) => say(&format!("HELD {step}: {:?}", seen.html)),
                Err(why) => {
                    let delivered = run.borrow().delivered.join(", ");
                    say(&format!("RED {step}: {why}; the keys went: {delivered}"));
                    run.borrow_mut()
                        .failures
                        .push(format!("{step}: {why}; the keys went: {delivered}"));
                }
            }
            Phase::Acting
        }
    }
}

// ── What the child said, as one sentence ──────────────────────────────────
//
// The parent passes a window test only on the child's own `ok` line for it,
// so a verdict is never invented on the way across the process boundary, and
// every other ending says which it was and what Windows says about the lock.

const A_TEST: &str = "test_a_marker_typed_at_the_start_of_any_line_makes_its_structure";

#[test]
fn test_the_window_test_runs_on_the_runners_own_desktop_only_where_ci_is_set() {
    assert_eq!(
        where_the_body_runs(Some(OsStr::new("true"))),
        WhereTheBodyRuns::OnTheRunnersOwnDesktop,
        "with CI set"
    );
    assert_eq!(
        where_the_body_runs(None),
        WhereTheBodyRuns::InAChild,
        "without CI"
    );
}

fn a_child_run(end: ChildEnd, output: &str, lock: SessionLock) -> ChildRun {
    ChildRun {
        desktop: "WinSta0\\wixen-marker-1".to_string(),
        end,
        output: output.to_string(),
        lock,
    }
}

#[test]
fn test_a_child_whose_line_says_ok_passes_the_test() {
    let output = format!("running 5 tests\ntest {A_TEST} ... ok\n\ntest result: ok.\n");
    let run = a_child_run(ChildEnd::Exited(0), &output, SessionLock::Unlocked);

    assert_eq!(what_the_child_said(A_TEST, &run), Ok(()));
}

#[test]
fn test_a_failed_child_quotes_its_failure_and_the_lock() {
    let output = format!(
        "running 5 tests\ntest {A_TEST} ... FAILED\n\nfailures:\n\n---- {A_TEST} stdout ----\n\
         STEP 1: ## on the first line of an empty message\n\
         1 step(s) did not hold on the real page\n\n\nfailures:\n    {A_TEST}\n\n\
         test result: FAILED. 4 passed; 1 failed\n"
    );
    let run = a_child_run(ChildEnd::Exited(101), &output, SessionLock::Locked);

    let said = what_the_child_said(A_TEST, &run).expect_err("a failed child fails the test");

    assert!(said.contains("STEP 1: ## on the first line"), "{said}");
    assert!(
        said.contains("1 step(s) did not hold on the real page"),
        "{said}"
    );
    assert!(!said.contains("test result: FAILED"), "{said}");
    assert!(
        said.ends_with("Windows says the session is locked."),
        "{said}"
    );
}

#[test]
fn test_a_child_stopped_at_its_bound_says_so() {
    let output = "running 5 tests\nSTEP 1: ## on the first line of an empty message\n";
    let run = a_child_run(ChildEnd::StoppedAtTheBound, output, SessionLock::Unlocked);

    let said = what_the_child_said(A_TEST, &run).expect_err("a stopped child fails the test");

    assert!(
        said.contains("did not finish in five minutes and was stopped"),
        "{said}"
    );
    assert!(said.contains("STEP 1: ## on the first line"), "{said}");
    assert!(
        said.ends_with("Windows says the session is unlocked."),
        "{said}"
    );
}

#[test]
fn test_a_child_that_never_reported_the_test_says_so() {
    let output =
        format!("running 5 tests\ntest {A_TEST}_and_more ... ok\nthe child stopped here\n");
    let answer = "WTSQuerySessionInformationW failed with error 87";
    let run = a_child_run(
        ChildEnd::Exited(0xC000_0005),
        &output,
        SessionLock::NotSaid(answer.to_string()),
    );

    let said = what_the_child_said(A_TEST, &run).expect_err("an unreported test fails");

    assert!(said.contains(&format!("never reported {A_TEST}")), "{said}");
    assert!(said.contains("exit code 0xc0000005"), "{said}");
    assert!(said.contains("the child stopped here"), "{said}");
    assert!(
        said.ends_with(&format!(
            "Windows did not say whether the session is locked: asking answered {answer}."
        )),
        "{said}"
    );
}

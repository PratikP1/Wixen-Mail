//! A signature follows the From account, and is chosen in two places that
//! show one answer (#43).
//!
//! The tester on 2026-09-15: "Allow signatures to be assigned by email
//! account. Default should apply if no signature is assigned to a particular
//! account." Pratik on 2026-09-23: the choice is made on the account's own
//! dialog and in the Signature Manager, one stored setting reachable from
//! both, so what either sets is what the other shows the next time it opens.
//!
//! **What is read.** A store built in the test with two accounts and three
//! signatures, one assigned to the first account and one the default. The
//! store's answer for each account; the Signature Manager's rows, built
//! through `wx_managers` and read off the live list; the account dialog's
//! choice, built and read back; each surface after the other has written;
//! the default set and cleared. Then the real composer, built from a From
//! list with Work, Home and an other address Home sends from: its page loaded
//! the way the composer loads it, the From entry changed through the control
//! with the key a person presses, and the page's markup read back.
//!
//! **Companions.** Each reading is a check that can be handed a wrong state.
//! The companions hand it one, a swap over an edited signature, a manager
//! showing one account's signatures, and an account dialog that did not read
//! what the manager wrote, and are refused, so a reading that passes is one
//! that could have failed.
//!
//! One window session for the file, every reading sharing it through a
//! `OnceLock`, on the shape `tests/event_times_move_in_blocks.rs` uses; the
//! composer's page is waited for on a timer, on the shape
//! `tests/a_marker_counts_at_the_start_of_any_line.rs` uses, since a page is
//! only there once the browser has made it. Runs under `WIXEN_NO_AUDIO` as
//! CI does, and on a temporary data directory in `WIXEN_MAIL_DATA`, never a
//! person's profile.

#![cfg(windows)]

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::ffi::{OsStr, c_void};
use std::fmt;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::AsRawHandle;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::identities::{self, FromEntry, Identity};
use wixen_mail::common::types::MessageBody;
use wixen_mail::data::Account;
use wixen_mail::data::message_cache::{MessageCache, Signature};
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::editor_document;
use wixen_mail::presentation::managers::{
    save_what_the_signature_manager_returned, the_signature_managers_rows,
};
use wixen_mail::presentation::wx_account_manager::{
    SignatureChoices, build_account_edit_dialog, keep_the_signature_choice,
};
use wixen_mail::presentation::wx_compose::{self, SignatureFor};
use wixen_mail::presentation::wx_managers::{
    ManagedRow, SignatureAccount, SignatureEntry, build_sig_edit_dialog, build_signature_manager,
    offers_for, the_signature_as_edited,
};
use wxdragon::prelude::*;
use wxdragon::widgets::WebView;

type Harvest = BTreeMap<&'static str, String>;

const WORK: &str = "acct-work";
const HOME: &str = "acct-home";
/// An account nobody assigned anything to, for the default's readings.
const NOBODY: &str = "acct-nobody";

const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const VK_END: usize = 0x23;
const VK_UP: usize = 0x26;
const VK_DOWN: usize = 0x28;

const TICK_MS: i32 = 30;
/// Ticks before the run says it waited too long: a minute.
const GIVE_UP_AFTER_TICKS: u32 = 2000;
/// Ticks of nothing after a key, so what it started has finished.
const TICKS_TO_SETTLE: u32 = 6;

#[link(name = "user32")]
unsafe extern "system" {
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
}

/// winuser.h: Shift, Control and Alt, each with its left and right key.
const MODIFIERS: [usize; 9] = [0x10, 0x11, 0x12, 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5];

/// Set this thread's keyboard state with exactly `down` of the modifier keys
/// held and every other key as it was.
///
/// A control reads a key's modifiers from the keyboard state, and a desktop of
/// the run's own keeps out the keys a person presses but not the ones they
/// hold: measured 2026-10-02 by 13-44.6.2, every one of eight failed runs of
/// two key targets came while the person at the machine held Shift.
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
// The composer's body is a browser's page, and a browser's process starts on
// its process's desktop, not on the desktop of the thread that asks for it, so
// this reading cannot move its window thread alone: the marker reading, built
// the same way, hung with its thread moved, the page never coming up
// (measured 2026-10-02). So the window tests run in a child of this same
// executable started on a desktop made for the run, where nothing a person
// types arrives, inside the one turn, and the parent passes each only on the
// child's own `ok` line for it. The helper is a copy of the one in
// `tests/a_marker_counts_at_the_start_of_any_line.rs`, where its sentence
// cases live.

/// Set in the child's environment to the desktop it runs on; a window test
/// that finds it set runs its body.
const ON_A_DESKTOP_OF_THEIR_OWN: &str = "WIXEN_TESTS_ON_A_DESKTOP_OF_THEIR_OWN";

/// How long the child may run before it is stopped.
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
/// WebView2 runs one browser process per user data folder, and the folder is
/// named for the executable, so two runs of this target at once would share
/// one browser, even on two desktops (ledger 761). It is taken in the parent
/// and never in the body the child runs, since the child would otherwise wait
/// on a turn its own parent holds.
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
    println!(
        "the child ran on {full_name} for {} s and {end}",
        started.elapsed().as_secs()
    );
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
/// false and the test runs its body.
fn ran_in_a_child_on_a_desktop_of_its_own(test: &str) -> bool {
    if std::env::var_os(ON_A_DESKTOP_OF_THEIR_OWN).is_some() {
        return false;
    }
    if where_the_body_runs(std::env::var_os("CI").as_deref())
        == WhereTheBodyRuns::OnTheRunnersOwnDesktop
    {
        static SAID: std::sync::Once = std::sync::Once::new();
        SAID.call_once(|| say_past_the_capture(ON_THE_RUNNERS_OWN_DESKTOP));
        return false;
    }
    static THE_CHILD: OnceLock<ChildRun> = OnceLock::new();
    let run = THE_CHILD.get_or_init(|| the_child_run("signature"));
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
/// 785; CI runs 37151597179 and 37154501219 named the starter, in the marker
/// reading's child). Nobody types on a runner, so there the bodies run here,
/// as they did before 13-44.6.3, and say so.
fn where_the_body_runs(ci: Option<&OsStr>) -> WhereTheBodyRuns {
    match ci {
        Some(_) => WhereTheBodyRuns::OnTheRunnersOwnDesktop,
        None => WhereTheBodyRuns::InAChild,
    }
}

/// What a run whose bodies run on the runner's own desktop says, once.
const ON_THE_RUNNERS_OWN_DESKTOP: &str = "the window tests ran on the runner's own desktop, not \
     in a child on a desktop of their own, because CI is set: nobody types on a runner, and \
     there an elevated child's browser opens on the shell's desktop and cannot open in the \
     child's (ledger 785)";

/// Write `line` to standard error itself, past libtest's capture, so a
/// runner's log carries it whatever the verdict.
fn say_past_the_capture(line: &str) {
    use std::io::Write;
    let mut stderr = std::io::stderr().lock();
    // A line that cannot be written is a sentence lost, not a failure.
    let _ = writeln!(stderr, "{line}");
}

/// The one turn, for a session that runs in this process on the runner's
/// own desktop; in the child the parent holds it already.
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

/// commctrl.h: `LVM_FIRST + 115`.
const LVM_GETITEMTEXTW: u32 = 0x1000 + 115;

/// commctrl.h's `LVITEMW`, of which `LVM_GETITEMTEXTW` reads the sub-item,
/// the buffer and its length.
#[repr(C)]
struct ListViewItem {
    mask: u32,
    item: i32,
    sub_item: i32,
    state: u32,
    state_mask: u32,
    text: *mut u16,
    text_max: i32,
    image: i32,
    param: isize,
    indent: i32,
    group_id: i32,
    columns: u32,
    column_list: *mut u32,
    column_formats: *mut i32,
    group: i32,
}

/// One cell of a live list, read from the list itself.
///
/// Not through `ListCtrl::get_item_text`, which loses the last character of
/// every cell and puts a NUL in its place (`wxdragon-0.9.17`,
/// `src/widgets/list_ctrl.rs:429`, measured in
/// `tests/manager_dialog_labels.rs`). This reading is about whole words, "Work"
/// against "Work, Home", so the cell is read whole.
fn cell(list: &ListCtrl, row: i64, column: i32) -> String {
    let mut buffer = [0u16; 512];
    let mut item = ListViewItem {
        mask: 0,
        item: row as i32,
        sub_item: column,
        state: 0,
        state_mask: 0,
        text: buffer.as_mut_ptr(),
        text_max: buffer.len() as i32,
        image: 0,
        param: 0,
        indent: 0,
        group_id: 0,
        columns: 0,
        column_list: std::ptr::null_mut(),
        column_formats: std::ptr::null_mut(),
        group: 0,
    };
    // SAFETY: a live list on this thread; the item and its buffer outlive the
    // call, and the length handed over is the buffer's.
    let length = unsafe {
        SendMessageW(
            list.get_handle() as isize,
            LVM_GETITEMTEXTW,
            row as usize,
            &mut item as *mut ListViewItem as isize,
        )
    };
    String::from_utf16_lossy(&buffer[..length.clamp(0, buffer.len() as isize) as usize])
}

// ── The store ─────────────────────────────────────────────────────────────

fn the_accounts() -> Vec<SignatureAccount> {
    vec![
        SignatureAccount {
            id: WORK.to_string(),
            name: "Work".to_string(),
        },
        SignatureAccount {
            id: HOME.to_string(),
            name: "Home".to_string(),
        },
    ]
}

fn an_account(id: &str, name: &str) -> Account {
    Account {
        id: id.to_string(),
        name: name.to_string(),
        email: format!("{}@example.com", name.to_lowercase()),
        ..Account::default()
    }
}

fn a_signature(id: &str, account_id: &str, name: &str, text: &str) -> Signature {
    Signature {
        id: id.to_string(),
        account_id: account_id.to_string(),
        name: name.to_string(),
        content_plain: text.to_string(),
        content_html: None,
        is_default: false,
        created_at: "2026-09-24T00:00:00Z".to_string(),
    }
}

/// Two accounts and three signatures: Work's own, the default, and Brief,
/// which nobody uses yet. Two written while Work was active and one while
/// Home was, so a manager that listed one account's would show a part.
fn a_store(at: &std::path::Path) -> Result<MessageCache, String> {
    let cache = MessageCache::new(at.to_path_buf(), None).map_err(|e| format!("{e}"))?;
    for signature in [
        a_signature("sig-work", WORK, "Work signature", "Regards, Work"),
        a_signature("sig-home", HOME, "Home signature", "Cheers, Home"),
        a_signature("sig-brief", WORK, "Brief", "B."),
    ] {
        cache
            .create_signature(&signature)
            .map_err(|e| format!("{e}"))?;
    }
    cache
        .assign(WORK, Some("sig-work"))
        .map_err(|e| format!("{e}"))?;
    cache
        .set_the_default(Some("sig-home"))
        .map_err(|e| format!("{e}"))?;
    Ok(cache)
}

fn signing(cache: &MessageCache, account: &str) -> Result<String, String> {
    Ok(cache
        .signature_for_account(account)
        .map_err(|e| format!("{e}"))?
        .map_or_else(|| "none".to_string(), |signature| signature.name))
}

// ── The surfaces ──────────────────────────────────────────────────────────

fn the_rows(cache: &MessageCache) -> Result<Vec<SignatureEntry>, String> {
    the_signature_managers_rows(cache, &the_accounts()).map_err(|e| format!("{e}"))
}

/// The Signature Manager's list as it stands, one line per row: name,
/// default, used by.
fn the_manager_as_shown(frame: &Frame, cache: &MessageCache) -> Result<String, String> {
    let manager = build_signature_manager(frame, &the_rows(cache)?, None);
    let rows: Vec<String> = (0..manager.list.get_item_count() as i64)
        .map(|row| {
            (0..3)
                .map(|column| cell(&manager.list, row, column))
                .collect::<Vec<_>>()
                .join(" | ")
        })
        .collect();
    manager.dialog.destroy();
    Ok(rows.join("\n"))
}

/// The account dialog's signature choice for one account: what it opens on,
/// and its first entry.
fn the_account_dialog(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
    cache: &MessageCache,
    account: &Account,
) -> Result<(String, String), String> {
    let choices = SignatureChoices::read(cache, Some(&account.id)).map_err(|e| format!("{e}"))?;
    let dialog = build_account_edit_dialog(frame, Some(account), a11y, None, &choices);
    let opens_on = dialog
        .signature_choice
        .get_string_selection()
        .unwrap_or_default();
    let first = dialog.signature_choice.get_string(0).unwrap_or_default();
    dialog.dialog.destroy();
    Ok((opens_on, first))
}

/// A signature's editor opened on `name`, changed by `change`, and OK
/// pressed: read back, settled among the rows as the manager settles it, and
/// saved through the manager's save.
fn edit_in_the_manager(
    frame: &Frame,
    cache: &MessageCache,
    name: &str,
    change: impl Fn(&wixen_mail::presentation::wx_managers::SigEditWidgets),
) -> Result<(), String> {
    let accounts = the_accounts();
    let opened = the_rows(cache)?;
    let at = opened
        .iter()
        .position(|row| row.name == name)
        .ok_or_else(|| format!("no row named {name}"))?;
    let offers = offers_for(&accounts, Some(&opened[at]), &opened);
    let editor = build_sig_edit_dialog(frame, Some(&opened[at]), &offers, None);
    change(&editor);
    let edited = the_signature_as_edited(&editor, Some(&opened[at]));
    editor.dialog.destroy();
    let mut returned = opened.clone();
    returned[at] = edited;
    SignatureEntry::settle(&mut returned, at);
    let failures =
        save_what_the_signature_manager_returned(cache, WORK, &opened, returned, &accounts);
    match failures.is_empty() {
        true => Ok(()),
        false => Err(failures.join("; ")),
    }
}

/// Whether a signature's editor opens with an account's box ticked.
fn the_editor_ticks(
    frame: &Frame,
    cache: &MessageCache,
    name: &str,
    account: &str,
) -> Result<String, String> {
    let accounts = the_accounts();
    let rows = the_rows(cache)?;
    let row = rows
        .iter()
        .find(|row| row.name == name)
        .ok_or_else(|| format!("no row named {name}"))?;
    let editor = build_sig_edit_dialog(
        frame,
        Some(row),
        &offers_for(&accounts, Some(row), &rows),
        None,
    );
    let ticked = editor
        .account_boxes
        .iter()
        .find(|(offered, _)| offered.id == account)
        .map_or_else(
            || "no box".to_string(),
            |(_, check)| check.get_value().to_string(),
        );
    editor.dialog.destroy();
    Ok(ticked)
}

/// A reading, or what stopped it being taken. A step that cannot be taken
/// fails the readings that depend on it and not the rest of the session.
fn or_why(result: Result<String, String>) -> String {
    result.unwrap_or_else(|why| format!("(not read: {why})"))
}

/// Work's own dialog set to Brief and saved.
fn choose_brief_on_works_dialog(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
    cache: &MessageCache,
    work: &Account,
) -> Result<(), String> {
    let choices = SignatureChoices::read(cache, Some(WORK)).map_err(|e| format!("{e}"))?;
    let dialog = build_account_edit_dialog(frame, Some(work), a11y, None, &choices);
    let brief = (0..dialog.signature_choice.get_count())
        .find(|at| dialog.signature_choice.get_string(*at).as_deref() == Some("Brief"));
    let kept = match brief {
        Some(brief) => {
            dialog.signature_choice.set_selection(brief);
            keep_the_signature_choice(cache, WORK, &dialog, &choices).map_err(|e| format!("{e}"))
        }
        None => Err("Work's dialog does not offer Brief".to_string()),
    };
    dialog.dialog.destroy();
    kept
}

fn read_the_surfaces(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
    cache: &MessageCache,
    into: &mut Harvest,
) {
    let work = an_account(WORK, "Work");
    let home = an_account(HOME, "Home");
    let dialog = |account: &Account| the_account_dialog(frame, a11y, cache, account);

    into.insert("the store: Work signs with", or_why(signing(cache, WORK)));
    into.insert("the store: Home signs with", or_why(signing(cache, HOME)));
    into.insert(
        "the manager, opened",
        or_why(the_manager_as_shown(frame, cache)),
    );
    into.insert(
        "Work's dialog, opened on",
        or_why(dialog(&work).map(|(opens_on, _)| opens_on)),
    );
    into.insert(
        "Work's dialog, first entry",
        or_why(dialog(&work).map(|(_, first)| first)),
    );

    // The manager writes, the account dialog reads: Brief given to Home in
    // Brief's editor.
    let given = edit_in_the_manager(frame, cache, "Brief", |editor| {
        for (account, check) in &editor.account_boxes {
            if account.id == HOME {
                check.set_value(true);
            }
        }
    });
    into.insert(
        "Home's dialog, after Brief's editor gave it Home",
        or_why(given.and_then(|()| dialog(&home).map(|(opens_on, _)| opens_on))),
    );

    // The account dialog writes, the manager reads: Work set to Brief on
    // Work's own dialog and saved.
    let chosen = choose_brief_on_works_dialog(frame, a11y, cache, &work);
    into.insert(
        "the manager, after Work's dialog chose Brief",
        or_why(
            chosen
                .clone()
                .and_then(|()| the_manager_as_shown(frame, cache)),
        ),
    );
    into.insert(
        "Brief's editor, Work's box, after Work's dialog chose Brief",
        or_why(chosen.and_then(|()| the_editor_ticks(frame, cache, "Brief", WORK))),
    );

    // The default, set and cleared in Brief's editor.
    let set = edit_in_the_manager(frame, cache, "Brief", |editor| {
        editor.def_check.set_value(true)
    });
    into.insert(
        "Work's dialog, first entry, with Brief the default",
        or_why(
            set.clone()
                .and_then(|()| dialog(&work).map(|(_, first)| first)),
        ),
    );
    into.insert(
        "the store: nobody's, with Brief the default",
        or_why(set.and_then(|()| signing(cache, NOBODY))),
    );
    let cleared = edit_in_the_manager(frame, cache, "Brief", |editor| {
        editor.def_check.set_value(false)
    });
    into.insert(
        "Work's dialog, first entry, with no default",
        or_why(
            cleared
                .clone()
                .and_then(|()| dialog(&work).map(|(_, first)| first)),
        ),
    );
    into.insert(
        "the store: nobody's, with no default",
        or_why(cleared.and_then(|()| signing(cache, NOBODY))),
    );
}

// ── The composer ──────────────────────────────────────────────────────────

/// The From list the composer is built with, by the rule the program builds
/// it by: Work, then Home, then the other address Home sends from.
fn the_from_list() -> Vec<FromEntry> {
    let others = HashMap::from([(
        HOME.to_string(),
        vec![Identity::typed("i-help", "help@example.com", "Help Desk")],
    )]);
    identities::the_from_list(
        &[an_account(WORK, "Work"), an_account(HOME, "Home")],
        &others,
    )
}

/// The signature an entry's account signs with, which is what an other
/// address signs with too (phase 13 decision 34).
fn the_signature_of(account_id: &str) -> SignatureFor {
    match account_id {
        WORK => SignatureFor {
            name: "Work signature".to_string(),
            text: "Regards, Work".to_string(),
        },
        _ => SignatureFor {
            name: "Home signature".to_string(),
            text: "Cheers, Home".to_string(),
        },
    }
}

/// A reply's body the way the composer quotes one written as a page.
fn a_reply() -> MessageBody {
    MessageBody::Html("<p><br></p><p>--- Original Message ---</p><p>Their words</p>".to_string())
}

/// One thing the composer's run does.
enum Act {
    /// Load a message into the page the way the composer does, signed with
    /// this text, and wait for the page.
    Open(MessageBody, &'static str),
    /// A script run in the page, standing for somebody's typing.
    Type(&'static str),
    /// The body's markup, read into the harvest under this name.
    Read(&'static str),
    /// A key pressed on the From account, then ticks of nothing.
    Press(usize),
}

fn the_composers_acts() -> Vec<Act> {
    vec![
        // A new message from Work, and From changed to Home.
        Act::Open(MessageBody::Plain(String::new()), "Regards, Work"),
        Act::Read("a new message, opened"),
        Act::Press(VK_DOWN),
        Act::Read("a new message, after From changed to Home"),
        // A new message from Home whose signature somebody typed into, and
        // From changed back to Work.
        Act::Open(MessageBody::Plain(String::new()), "Cheers, Home"),
        Act::Type("b.innerHTML = b.innerHTML.replace('Cheers, Home', 'Cheers, Home team');"),
        Act::Read("an edited message, before From changed"),
        Act::Press(VK_UP),
        Act::Read("an edited message, after From changed to Work"),
        // A reply from Work, and From changed to Home.
        Act::Open(a_reply(), "Regards, Work"),
        Act::Press(VK_DOWN),
        Act::Read("a reply, after From changed to Home"),
        // Back to Work, then a new message from Work, and From taken straight
        // to the last entry, the other address Home sends from.
        Act::Press(VK_UP),
        Act::Open(MessageBody::Plain(String::new()), "Regards, Work"),
        Act::Press(VK_END),
        Act::Read("a new message, after From changed to Home's other address"),
    ]
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    WaitingForBrowser,
    WaitingForPage,
    Settling(u32),
    Acting,
    Done,
}

struct Run {
    acts: Vec<Act>,
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
        "(typeof window.wixenRules === 'object' && !window.__signature_run) ? 'up' : 'not yet'",
    )) == "up"
}

fn press(choice: &Choice, key: usize) {
    only_these_modifiers_down(&[]);
    let hwnd = choice.get_handle() as isize;
    // SAFETY: a live window on this thread; the messages carry a key.
    unsafe {
        SendMessageW(hwnd, WM_KEYDOWN, key, 0x0100_0001);
        SendMessageW(hwnd, WM_KEYUP, key, 0xC100_0001_u32 as i32 as isize);
    }
}

fn one_act(run: &Rc<RefCell<Run>>, body_editor: &WebView, choice: &Choice) -> Phase {
    let index = run.borrow().next;
    if index >= run.borrow().acts.len() {
        return Phase::Done;
    }
    run.borrow_mut().next = index + 1;
    let act = std::mem::replace(&mut run.borrow_mut().acts[index], Act::Read("spent"));
    match act {
        Act::Open(body, signature) => {
            let signed = wx_compose::with_signature(&body, signature);
            body_editor.set_page(&editor_document::editor_document(&signed, "en", false), "");
            Phase::WaitingForPage
        }
        Act::Type(script) => {
            let _ = body_editor.run_script(&format!(
                "(function () {{ var b = document.getElementById('wixen-body'); {script} return 'typed'; }})()"
            ));
            Phase::Settling(TICKS_TO_SETTLE)
        }
        Act::Read(name) => {
            let html = unquoted(body_editor.run_script(&editor_document::read_body_script()));
            run.borrow_mut().harvest.insert(name, html);
            Phase::Acting
        }
        Act::Press(key) => {
            press(choice, key);
            Phase::Settling(TICKS_TO_SETTLE)
        }
    }
}

// ── The session ───────────────────────────────────────────────────────────

fn take_the_harvest() -> Result<Harvest, String> {
    let _turn = the_turn_for_a_body_run_here()?;
    let data = tempfile::tempdir().map_err(|e| format!("a data directory: {e}"))?;
    // SAFETY: set before the window session starts any thread, and nothing
    // has read either yet.
    unsafe {
        std::env::set_var("WIXEN_MAIL_DATA", data.path());
        std::env::set_var("WIXEN_NO_AUDIO", "1");
    }
    let store_at = tempfile::tempdir().map_err(|e| format!("a store directory: {e}"))?;
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        let store_at = store_at.path().to_path_buf();
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
                .with_title("A signature follows the From account, the reading")
                .build();
            let a11y = match Accessibility::new() {
                Ok(a11y) => Arc::new(a11y),
                Err(why) => return finish(Err(format!("accessibility: {why:?}"))),
            };
            let mut harvest = Harvest::new();
            match a_store(&store_at) {
                Ok(cache) => read_the_surfaces(&frame, &a11y, &cache, &mut harvest),
                Err(why) => return finish(Err(why)),
            }

            let from_list = the_from_list();
            let widgets = wx_compose::build_compose_dialog(
                &frame,
                "Compose New Message",
                &from_list,
                0,
                None,
            );
            let dialog = widgets.dialog;
            let body_editor = widgets.body_editor;
            let choice = widgets.account_choice;
            let browser = widgets.browser.clone();
            wx_compose::follow_the_from_account(
                choice,
                body_editor,
                from_list
                    .iter()
                    .map(|entry| the_signature_of(&entry.account_id))
                    .collect(),
                0,
                a11y.clone(),
            );
            dialog.show(true);

            let run = Rc::new(RefCell::new(Run {
                acts: the_composers_acts(),
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
                            run.failure = Some("the composer's run waited too long".to_string());
                            run.phase = Phase::Done;
                            run.next = run.acts.len();
                        }
                        run.phase
                    };
                    let next = match phase {
                        Phase::WaitingForBrowser if browser.is_ready() => Phase::Acting,
                        Phase::WaitingForPage if the_page_is_up(&body_editor) => {
                            let _ = body_editor.run_script("window.__signature_run = 1; 'marked'");
                            Phase::Settling(TICKS_TO_SETTLE)
                        }
                        Phase::Settling(left) if left > 1 => Phase::Settling(left - 1),
                        Phase::Settling(_) => Phase::Acting,
                        Phase::Acting => one_act(&run, &body_editor, &choice),
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

// ── The checks, each of which a companion hands a wrong state ─────────────

fn every_signature_is_listed(manager: &str) -> Result<(), String> {
    let names: Vec<&str> = manager
        .lines()
        .map(|row| row.split(" | ").next().unwrap_or_default())
        .collect();
    match names == ["Brief", "Home signature", "Work signature"] {
        true => Ok(()),
        false => Err(format!("the manager listed {names:?}")),
    }
}

fn the_row(manager: &str, name: &str) -> String {
    manager
        .lines()
        .find(|row| row.starts_with(&format!("{name} | ")))
        .unwrap_or_default()
        .to_string()
}

fn the_dialog_opens_on(reading: &str, wanted: &str) -> Result<(), String> {
    match reading == wanted {
        true => Ok(()),
        false => Err(format!(
            "the account dialog opened on {reading:?}, not {wanted:?}"
        )),
    }
}

fn the_block_was_left_alone(before: &str, after: &str) -> Result<(), String> {
    match before == after && after.contains("Cheers, Home team") {
        true => Ok(()),
        false => Err(format!(
            "the edited message changed:\nbefore {before}\nafter  {after}"
        )),
    }
}

// ── Where the window tests run ────────────────────────────────────────────

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

// ── The store ─────────────────────────────────────────────────────────────

#[test]
fn test_an_account_with_a_signature_assigned_signs_with_it() {
    if ran_in_a_child_on_a_desktop_of_its_own(
        "test_an_account_with_a_signature_assigned_signs_with_it",
    ) {
        return;
    }
    assert_eq!(reading("the store: Work signs with"), "Work signature");
}

#[test]
fn test_an_account_with_none_assigned_signs_with_the_default() {
    if ran_in_a_child_on_a_desktop_of_its_own(
        "test_an_account_with_none_assigned_signs_with_the_default",
    ) {
        return;
    }
    assert_eq!(reading("the store: Home signs with"), "Home signature");
}

// ── The Signature Manager ─────────────────────────────────────────────────

#[test]
fn test_the_manager_lists_every_signature_whichever_account_wrote_it() {
    if ran_in_a_child_on_a_desktop_of_its_own(
        "test_the_manager_lists_every_signature_whichever_account_wrote_it",
    ) {
        return;
    }
    every_signature_is_listed(reading("the manager, opened")).unwrap();
}

#[test]
fn test_the_manager_says_who_uses_each_signature() {
    if ran_in_a_child_on_a_desktop_of_its_own("test_the_manager_says_who_uses_each_signature") {
        return;
    }
    let manager = reading("the manager, opened");
    assert_eq!(
        the_row(manager, "Work signature"),
        "Work signature |  | Work"
    );
    assert_eq!(
        the_row(manager, "Home signature"),
        "Home signature | ★ | everyone else"
    );
    assert_eq!(the_row(manager, "Brief"), "Brief |  | ");
}

#[test]
fn test_companion_a_manager_showing_one_accounts_signatures_is_refused() {
    let one_account = "Brief |  | \nWork signature |  | Work";
    assert!(every_signature_is_listed(one_account).is_err());
}

// ── The account's own dialog ──────────────────────────────────────────────

#[test]
fn test_the_account_dialog_opens_on_the_signature_the_account_uses() {
    if ran_in_a_child_on_a_desktop_of_its_own(
        "test_the_account_dialog_opens_on_the_signature_the_account_uses",
    ) {
        return;
    }
    the_dialog_opens_on(reading("Work's dialog, opened on"), "Work signature").unwrap();
}

#[test]
fn test_the_account_dialogs_first_entry_names_the_default() {
    if ran_in_a_child_on_a_desktop_of_its_own(
        "test_the_account_dialogs_first_entry_names_the_default",
    ) {
        return;
    }
    assert_eq!(
        reading("Work's dialog, first entry"),
        "Use the default: Home signature"
    );
}

// ── Each surface shows what the other chose ──────────────────────────────

#[test]
fn test_an_account_ticked_in_a_signatures_editor_is_what_its_dialog_shows() {
    if ran_in_a_child_on_a_desktop_of_its_own(
        "test_an_account_ticked_in_a_signatures_editor_is_what_its_dialog_shows",
    ) {
        return;
    }
    the_dialog_opens_on(
        reading("Home's dialog, after Brief's editor gave it Home"),
        "Brief",
    )
    .unwrap();
}

#[test]
fn test_companion_an_assignment_the_account_dialog_does_not_read_is_refused() {
    // A manager that kept its own copy of the assignment: the dialog would
    // open on the default, as though nothing had been chosen.
    assert!(the_dialog_opens_on("Use the default: Home signature", "Brief").is_err());
}

#[test]
fn test_a_signature_chosen_on_the_account_dialog_is_what_the_manager_shows() {
    if ran_in_a_child_on_a_desktop_of_its_own(
        "test_a_signature_chosen_on_the_account_dialog_is_what_the_manager_shows",
    ) {
        return;
    }
    assert_eq!(
        the_row(
            reading("the manager, after Work's dialog chose Brief"),
            "Brief"
        ),
        "Brief |  | Work, Home"
    );
    assert_eq!(
        reading("Brief's editor, Work's box, after Work's dialog chose Brief"),
        "true"
    );
}

// ── The default, set and cleared in the manager ──────────────────────────

#[test]
fn test_the_default_set_in_the_manager_is_named_on_the_account_dialog() {
    if ran_in_a_child_on_a_desktop_of_its_own(
        "test_the_default_set_in_the_manager_is_named_on_the_account_dialog",
    ) {
        return;
    }
    assert_eq!(
        reading("Work's dialog, first entry, with Brief the default"),
        "Use the default: Brief"
    );
    assert_eq!(
        reading("the store: nobody's, with Brief the default"),
        "Brief"
    );
}

#[test]
fn test_the_default_cleared_in_the_manager_leaves_an_unassigned_account_with_none() {
    if ran_in_a_child_on_a_desktop_of_its_own(
        "test_the_default_cleared_in_the_manager_leaves_an_unassigned_account_with_none",
    ) {
        return;
    }
    assert_eq!(
        reading("Work's dialog, first entry, with no default"),
        "Use the default (none is set)"
    );
    assert_eq!(reading("the store: nobody's, with no default"), "none");
}

// ── The composer ──────────────────────────────────────────────────────────

#[test]
fn test_a_new_messages_signature_follows_the_from_account() {
    if ran_in_a_child_on_a_desktop_of_its_own(
        "test_a_new_messages_signature_follows_the_from_account",
    ) {
        return;
    }
    let opened = reading("a new message, opened");
    assert!(opened.contains("Regards, Work"), "opened as {opened}");
    let after = reading("a new message, after From changed to Home");
    assert!(
        after.contains("Cheers, Home") && !after.contains("Regards, Work"),
        "after the change: {after}"
    );
}

#[test]
fn test_a_signature_somebody_typed_into_stays_when_the_from_account_changes() {
    if ran_in_a_child_on_a_desktop_of_its_own(
        "test_a_signature_somebody_typed_into_stays_when_the_from_account_changes",
    ) {
        return;
    }
    the_block_was_left_alone(
        reading("an edited message, before From changed"),
        reading("an edited message, after From changed to Work"),
    )
    .unwrap();
}

#[test]
fn test_companion_a_swap_over_an_edited_signature_is_refused() {
    let before = "<br><br>-- <br>Cheers, Home team";
    let swapped = "<br><br>-- <br>Regards, Work";
    assert!(the_block_was_left_alone(before, swapped).is_err());
}

#[test]
fn test_a_replys_signature_follows_the_from_account_above_the_quote() {
    if ran_in_a_child_on_a_desktop_of_its_own(
        "test_a_replys_signature_follows_the_from_account_above_the_quote",
    ) {
        return;
    }
    let after = reading("a reply, after From changed to Home");
    let signature = after.find("Cheers, Home");
    let quote = after.find("--- Original Message ---");
    assert!(
        matches!((signature, quote), (Some(s), Some(q)) if s < q)
            && !after.contains("Regards, Work")
            && after.contains("Their words"),
        "the reply after the change: {after}"
    );
}

#[test]
fn test_an_other_address_signs_with_its_accounts_signature() {
    if ran_in_a_child_on_a_desktop_of_its_own(
        "test_an_other_address_signs_with_its_accounts_signature",
    ) {
        return;
    }
    // From Work straight to the address Home also sends from: the message
    // takes Home's signature, as Home's own entry would.
    let after = reading("a new message, after From changed to Home's other address");
    assert!(
        after.contains("Cheers, Home") && !after.contains("Regards, Work"),
        "after the change: {after}"
    );
}

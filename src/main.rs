#![windows_subsystem = "windows"]

use wixen_mail::application::running::Claim;
use wixen_mail::common::logging::{LoggerConfig, init_logging};
use wixen_mail::common::paths::{AppPaths, LegacyLocations, MigrationReport};
use wixen_mail::common::version;
use wixen_mail::presentation::WxMailApp;
use wixen_mail::presentation::command_line::{self, Command};
use wixen_mail::presentation::scan_target;

fn main() {
    // Install a panic hook FIRST so crashes are always captured to a file,
    // even when running as a GUI app with no console.
    install_panic_hook();

    let asked = command_line::parse(std::env::args().skip(1));
    // Filled in only if Windows would not say whether a copy is running, so
    // the reason can wait for the log rather than being lost.
    let mut unmarked: Option<String> = None;

    let run = match asked {
        Command::EraseAllData => {
            // Deliberately before logging is set up. The log file would be
            // opened inside the folder being removed, and an open file is
            // exactly what stops Windows removing it.
            std::process::exit(erase_all_data());
        }
        Command::Help => return say(command_line::HELP),
        Command::Version => return say(&format!("Wixen Mail {}\n", version::current())),
        Command::Refused(why) => {
            // Nothing opens. A window that appeared having quietly ignored a
            // misspelled --read-only is the accident this exists to prevent,
            // and the exit code lets a script tell it from a clean start.
            complain(&format!("{why}\n"));
            std::process::exit(2);
        }
        Command::Run(run) => run,
    };

    // Held for as long as this program is up, so the uninstaller and
    // --erase-all-data can both see that deleting the data folder right now
    // would be deleting it out from under an open window. Bound to a name
    // rather than to `_`, which would drop it here and mark nothing.
    //
    // A second copy is not stopped. Whether Wixen Mail should be one window or
    // many is a separate question, and the marker only has to say that at
    // least one is up. Both cases hold it, so closing the first while the
    // second is still open does not clear the mark.
    let _running = match wixen_mail::application::running::claim() {
        Claim::Granted(marker) | Claim::AnotherIsRunning(marker) => Some(marker),
        Claim::Unknown(why) => {
            // Deferred: logging is not up yet, and this is worth a line in the
            // log rather than nothing at all.
            unmarked = Some(why);
            None
        }
    };

    // Before anything opens a file, including the log the next line writes.
    let migration = prepare_data_folder();

    let _log_guard = init_logging(LoggerConfig::default()).ok();
    // The build identifier is part of this on purpose: a bug report arrives
    // with a log, and several builds can share a version now.
    tracing::info!("Starting Wixen Mail v{}", version::current());
    if let Some(why) = unmarked {
        // Not silently absorbed: without the mark, an uninstall started now
        // would not know this window is open.
        tracing::warn!("Could not mark Wixen Mail as running: {why}");
    }
    // Before anything can be built that might write. Recorded once here, and
    // read wherever a client is made, so the command line narrows every one of
    // them without being threaded through the window layer.
    wixen_mail::application::allowed::narrow_this_run_to(run.allowed);
    if !run.allowed.anything() {
        tracing::info!("Started read only: nothing will be changed at any server");
    }
    report_migration(migration.as_ref());

    // Before the application is built, because an unknown name here has to
    // stop rather than start normally: a scan that walks the main window and
    // reports a clean pass for a dialog it never opened is worse than no scan.
    let scan_target = match run.scan_target.as_deref().map(scan_target::named) {
        None => None,
        Some(Ok(target)) => Some(target),
        Some(Err(e)) => {
            // The log and the crash file, not a dialog. This flag is only ever
            // passed by the accessibility workflow, which runs with nobody
            // watching, and a modal error box there does not report a failure,
            // it hangs the job until its timeout and reports that instead.
            tracing::error!("{}", e);
            log_crash(&e.to_string());
            std::process::exit(2);
        }
    };

    match WxMailApp::new() {
        Ok(app) => {
            if let Err(e) = app.run(scan_target) {
                tracing::error!("UI error: {}", e);
                log_crash(&format!("UI run error: {}", e));
                show_error_dialog(&format!("Wixen Mail failed to run:\n{}", e));
                std::process::exit(1);
            }
        }
        Err(e) => {
            tracing::error!("Failed to initialize Wixen Mail: {}", e);
            log_crash(&format!("Init error: {}", e));
            show_error_dialog(&format!("Wixen Mail failed to start:\n{}", e));
            std::process::exit(1);
        }
    }
}

/// Exit code for a run that would not erase because a copy is open.
///
/// Its own number rather than the general failure 1, so a script or a wizard
/// can tell "close the program and try again" from "something went wrong".
const REFUSED_A_COPY_IS_OPEN: i32 = 3;

/// Remove the credential store entries and the data folder.
///
/// Refuses while another copy of Wixen Mail is open. Deleting the folder under
/// a running program takes the files it does not have open, leaves the ones it
/// does, and lets the copy still running write its settings back over the gap,
/// so what is left is neither an installation nor a clean machine. The
/// uninstaller stops before this through `AppMutex`; this is the backstop, and
/// the whole of the protection for somebody typing the flag by hand.
///
/// Otherwise nothing is shown on screen. This runs from the uninstaller, where
/// a dialog would be a window nobody asked for in the middle of somebody
/// else's progress bar. Anything that could not be removed is written to a
/// file in the temporary directory, because the folder that would normally
/// hold the log is the one being deleted.
fn erase_all_data() -> i32 {
    let mut left_behind = Vec::new();

    // Held for the whole erase, so a second one cannot start half way through
    // the first. Named rather than bound to `_`, which would give it up here.
    let _marker = match wixen_mail::application::running::claim() {
        Claim::Granted(marker) => marker,
        Claim::AnotherIsRunning(_) => {
            let refusal = "Wixen Mail is still open. Nothing has been erased. \
                           Close it and run this again.";
            complain(&format!("{refusal}\n"));
            report_what_was_left(&[refusal.to_string()]);
            return REFUSED_A_COPY_IS_OPEN;
        }
        Claim::Unknown(why) => {
            // Carrying on rather than refusing. Somebody unable to answer the
            // question is still entitled to uninstall, and the note is what
            // keeps the gap visible instead of swallowing it.
            left_behind.push(format!(
                "Could not check whether Wixen Mail was still open, so this ran anyway: {why}"
            ));
            return finish_erasing(left_behind);
        }
    };

    finish_erasing(left_behind)
}

/// Erase, now that it has been established that nothing else is running.
fn finish_erasing(mut left_behind: Vec<String>) -> i32 {
    // Resolved once and used for both halves: the credentials named by what is
    // in the database, and then the folder the database is in.
    let resolved = AppPaths::resolve();

    let outcome = wixen_mail::application::forget::run(resolved.as_ref().ok());
    left_behind.extend(
        outcome
            .refused
            .iter()
            .map(|refusal| format!("Credential left in the store, {refusal}")),
    );

    match resolved {
        Ok(paths) => {
            if paths.root().exists()
                && let Err(e) = std::fs::remove_dir_all(paths.root())
            {
                left_behind.push(format!("Could not remove {}: {e}", paths.root().display()));
            }
        }
        Err(e) => left_behind.push(format!(
            "Could not find the data folder, so none of it was removed: {e}"
        )),
    }

    // Written either way. A note only on failure makes silence ambiguous
    // between "it worked" and "it never ran", and an uninstall that left the
    // whole data folder behind wrote nothing at all, so there was no way to
    // tell which had happened.
    report_what_was_left(&left_behind);

    if left_behind.is_empty() {
        return 0;
    }
    // Something was left. The uninstaller does not read this today, which is
    // recorded against the uninstall test rather than guessed at here, but a
    // command that could not do what it was asked should not report success to
    // whatever does read it.
    1
}

/// Leave a note somewhere that survives the uninstall.
///
/// Silence would mean somebody believes their mail and their stored passwords
/// are gone when they are not.
fn report_what_was_left(problems: &[String]) {
    let path = std::env::temp_dir().join("wixen-mail-uninstall.log");
    let body = wixen_mail::application::forget::note(env!("CARGO_PKG_VERSION"), problems);
    let _ = std::fs::write(path, body);
}

/// Make the data folders, and collect files earlier versions left elsewhere.
///
/// Earlier versions kept the security key in the roaming profile, oauth.toml in
/// a dotfolder in the home directory, and the message database beside the
/// settings. This runs once per start and does nothing on an install that has
/// already been collected.
fn prepare_data_folder() -> Option<MigrationReport> {
    let paths = match AppPaths::resolve() {
        Ok(paths) => paths,
        Err(e) => {
            log_crash(&format!("Could not work out where to keep data: {e}"));
            return None;
        }
    };
    if let Err(e) = paths.create() {
        log_crash(&format!("Could not create the data folder: {e}"));
        return None;
    }
    Some(paths.migrate_legacy(&LegacyLocations::detect()))
}

/// Say what the migration did, once logging can record it.
fn report_migration(report: Option<&MigrationReport>) {
    let Some(report) = report.filter(|report| !report.is_empty()) else {
        return;
    };
    for path in &report.moved {
        tracing::info!("Collected into the data folder: {}", path.display());
    }
    for (path, reason) in &report.failed {
        // Not silently absorbed: the file is still readable where it is, and
        // somebody has to be able to find out why it did not move.
        tracing::warn!("Left in place, could not move {}: {reason}", path.display());
    }
}

/// Install a panic hook that writes crash info to a log file.
/// With `#![windows_subsystem = "windows"]` there is no console, so panics
/// would otherwise vanish silently.
fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic payload".to_string()
        };

        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let message = format!(
            "PANIC at {}\n  {}\n  Wixen Mail v{}\n  Time: {:?}",
            location,
            payload,
            env!("CARGO_PKG_VERSION"),
            std::time::SystemTime::now(),
        );

        // Try logging via tracing (may not be initialized yet)
        tracing::error!("{}", message);

        // Always write to crash file
        log_crash(&message);
    }));
}

/// Write a crash/error message to a persistent log file.
fn log_crash(message: &str) {
    let crash_dir = wixen_mail::common::logging::default_log_dir();
    let _ = std::fs::create_dir_all(&crash_dir);
    let crash_file = crash_dir.join("crash.log");
    let timestamped = format!(
        "[{}] {}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        message
    );
    // Append to crash log
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&crash_file)
        .and_then(|mut f| std::io::Write::write_all(&mut f, timestamped.as_bytes()));
}

/// Whether what is being said is an answer or a complaint.
///
/// It decides two things: which stream the words go to, and, when there is no
/// stream and a dialog is all that is left, what the dialog calls itself. A
/// box headed "Wixen Mail Error" containing the help text is a lie about the
/// help text.
#[derive(Clone, Copy)]
enum Tone {
    Answer,
    Complaint,
}

/// Show a message using the Windows MessageBox API (works without wxWidgets).
fn show_dialog(message: &str, tone: Tone) {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use std::ptr;

        const MB_ICONERROR: u32 = 0x10;
        const MB_ICONINFORMATION: u32 = 0x40;

        #[link(name = "user32")]
        unsafe extern "system" {
            fn MessageBoxW(hwnd: *mut (), text: *const u16, caption: *const u16, flags: u32)
            -> i32;
        }

        fn to_wide(s: &str) -> Vec<u16> {
            OsStr::new(s)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect()
        }

        let (heading, icon) = match tone {
            Tone::Answer => ("Wixen Mail", MB_ICONINFORMATION),
            Tone::Complaint => ("Wixen Mail Error", MB_ICONERROR),
        };

        let text = to_wide(message);
        let caption = to_wide(heading);
        unsafe {
            MessageBoxW(ptr::null_mut(), text.as_ptr(), caption.as_ptr(), icon);
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = tone;
        eprintln!("{}", message);
    }
}

/// Something went wrong: say so and keep it off the output stream.
fn show_error_dialog(message: &str) {
    show_dialog(message, Tone::Complaint);
}

/// Answer somebody who started this from a terminal.
fn say(words: &str) {
    put(words, Tone::Answer);
}

/// Tell somebody who started this from a terminal that it will not run.
///
/// Separate from [`say`] because it goes to the error stream. Redirecting the
/// output of a run that refused to start should leave the reason on the screen
/// rather than hiding it in the file, and anything reading a successful run's
/// output should not have to sift complaints out of it.
fn complain(words: &str) {
    put(words, Tone::Complaint);
}

/// Get words in front of a person, wherever there is to put them.
///
/// `windows_subsystem = "windows"` means the program starts with no console,
/// so `println!` can go nowhere at all and it has to look. In order:
///
/// 1. **The stream it was given.** Redirected to a file or a pipe, or
///    inherited from the shell that launched it. This is the one that does the
///    work, for both `wixen-mail --help` typed at a prompt and `--version`
///    read by a script, because a shell hands its console down as the
///    program's own handles. It has to be tried first: a build server has no
///    console but does have a pipe, and reaching past it for the dialog in
///    step 3 would hang the job rather than fail it. Covered by
///    `tests/command_line_output.rs`.
/// 2. **The console it was launched from.** For a launcher that starts us with
///    empty handles but leaves a console above us to attach to. Attaching does
///    not fill the empty handles, so the console is opened by name instead.
///    This is a fallback and has not been seen to fire; it is here because the
///    alternative when it is needed is silence.
/// 3. **A dialog.** Nothing else is left, which means a double click or a
///    shortcut, and appearing to do nothing is the worst answer.
fn put(words: &str, tone: Tone) {
    if wrote_to_stream(words, tone) {
        return;
    }

    #[cfg(windows)]
    {
        // ATTACH_PARENT_PROCESS. Fails when there is no console to attach to,
        // which is what a double click looks like.
        let attached = unsafe { windows::Win32::System::Console::AttachConsole(u32::MAX).is_ok() };
        if attached && wrote_to_console(words) {
            return;
        }
    }

    show_dialog(words, tone);
}

/// Write to stdout or stderr, saying whether the words landed.
///
/// They do not land when the handle is empty, which is the state a program
/// with no console and no redirection starts in.
fn wrote_to_stream(words: &str, tone: Tone) -> bool {
    use std::io::Write;

    let written = match tone {
        Tone::Answer => {
            let mut out = std::io::stdout().lock();
            out.write_all(words.as_bytes()).and_then(|()| out.flush())
        }
        Tone::Complaint => {
            let mut out = std::io::stderr().lock();
            out.write_all(words.as_bytes()).and_then(|()| out.flush())
        }
    };
    written.is_ok()
}

/// Write straight to the attached console, saying whether the words landed.
///
/// `CONOUT$` is the console's screen buffer under the name Windows gives it.
/// Attaching to a console does not fill in the program's own empty handles, so
/// this is the only way to reach it. Both streams share the one buffer, which
/// is why the tone does not choose between two of them here.
#[cfg(windows)]
fn wrote_to_console(words: &str) -> bool {
    use std::io::Write;

    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("CONOUT$")
        .and_then(|mut console| {
            console
                .write_all(words.as_bytes())
                .and_then(|()| console.flush())
        })
        .is_ok()
}

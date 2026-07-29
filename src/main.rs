#![windows_subsystem = "windows"]

use wixen_mail::common::logging::{LoggerConfig, init_logging};
use wixen_mail::common::paths::{AppPaths, LegacyLocations, MigrationReport};
use wixen_mail::presentation::WxMailApp;
use wixen_mail::presentation::command_line::{self, Command};
use wixen_mail::presentation::scan_target;

fn main() {
    // Install a panic hook FIRST so crashes are always captured to a file,
    // even when running as a GUI app with no console.
    install_panic_hook();

    let asked = command_line::parse(std::env::args().skip(1));

    let run = match asked {
        Command::EraseAllData => {
            // Deliberately before logging is set up. The log file would be
            // opened inside the folder being removed, and an open file is
            // exactly what stops Windows removing it.
            erase_all_data();
            return;
        }
        Command::Help => return say(command_line::HELP),
        Command::Version => return say(&format!("Wixen Mail {}\n", env!("CARGO_PKG_VERSION"))),
        Command::Refused(why) => {
            // Nothing opens. A window that appeared having quietly ignored a
            // misspelled --read-only is the accident this exists to prevent,
            // and the exit code lets a script tell it from a clean start.
            say(&format!("{why}\n"));
            std::process::exit(2);
        }
        Command::Run(run) => run,
    };

    // Before anything opens a file, including the log the next line writes.
    let migration = prepare_data_folder();

    let _log_guard = init_logging(LoggerConfig::default()).ok();
    tracing::info!("Starting Wixen Mail v{}", env!("CARGO_PKG_VERSION"));
    if !run.allowed.anything() {
        tracing::info!("Started read only: nothing will be changed at any server");
    }
    report_migration(migration.as_ref());

    // Before the application is built, because an unknown name here has to
    // stop rather than start normally: a scan that walks the main window and
    // reports a clean pass for a dialog it never opened is worse than no scan.
    let scan_target = match run
        .scan_target
        .as_deref()
        .map(|name| scan_target::from_args(["--scan-target", name]))
    {
        None => None,
        Some(Ok(target)) => target,
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

/// Remove the credential store entries and the data folder.
///
/// Nothing is shown on screen. This runs from the uninstaller, where a dialog
/// would be a window nobody asked for in the middle of somebody else's progress
/// bar. Anything that could not be removed is written to a file in the
/// temporary directory, because the folder that would normally hold the log is
/// the one being deleted.
fn erase_all_data() {
    let mut left_behind = Vec::new();

    let outcome = wixen_mail::application::forget::run();
    left_behind.extend(
        outcome
            .refused
            .iter()
            .map(|refusal| format!("Credential left in the store, {refusal}")),
    );

    match AppPaths::resolve() {
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

    if !left_behind.is_empty() {
        report_what_was_left(&left_behind);
    }
}

/// Leave a note somewhere that survives the uninstall.
///
/// Silence would mean somebody believes their mail and their stored passwords
/// are gone when they are not.
fn report_what_was_left(problems: &[String]) {
    let path = std::env::temp_dir().join("wixen-mail-uninstall.log");
    let body = format!(
        "Wixen Mail v{} could not remove everything:\n{}\n",
        env!("CARGO_PKG_VERSION"),
        problems.join("\n")
    );
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

/// Show an error dialog using the Windows MessageBox API (works without wxWidgets).
fn show_error_dialog(message: &str) {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use std::ptr;

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

        let text = to_wide(message);
        let caption = to_wide("Wixen Mail Error");
        unsafe {
            MessageBoxW(ptr::null_mut(), text.as_ptr(), caption.as_ptr(), 0x10);
            // MB_ICONERROR
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("{}", message);
    }
}

/// Put something in front of somebody who started this from a terminal.
///
/// `windows_subsystem = "windows"` means there is no console, so `println!`
/// goes nowhere: a person typing `wixen-mail --help` gets their prompt back
/// and nothing else. Attaching to the console that launched us fixes that,
/// and does nothing when there was not one, which is the ordinary case of
/// double-clicking the icon.
///
/// Falls back to a dialog when there is no console at all, so `--help` from a
/// shortcut still says something rather than appearing to do nothing.
fn say(words: &str) {
    #[cfg(windows)]
    let attached = unsafe {
        // ATTACH_PARENT_PROCESS. Fails harmlessly when there is no console to
        // attach to, which is what happens on a double click.
        windows::Win32::System::Console::AttachConsole(u32::MAX).is_ok()
    };
    #[cfg(not(windows))]
    let attached = true;

    if attached {
        use std::io::Write;
        let mut out = std::io::stdout();
        let _ = out.write_all(words.as_bytes());
        let _ = out.flush();
        return;
    }
    show_error_dialog(words);
}

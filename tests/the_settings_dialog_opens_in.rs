//! How long the Settings dialog takes to build, and which part of the build
//! takes the time.
//!
//! #34, the tester on 2026-09-15: "Loading settings by pressing ctrl+, is
//! noticeably slow." Nothing in the tree had timed it. This target does,
//! under the definitions below, and the tests that run on every commit hold
//! the shape of the lines the measurement prints and the line the application
//! writes, so the figures on `docs/development/measurements.md` were taken
//! under a shape a test holds.
//!
//! # The definitions
//!
//! **Built** means from the top of the `ID_SETTINGS` arm in the window, which
//! is where `Ctrl+,` lands, to the moment before `show_modal`: the
//! configuration read, the calendar-server query, and every page's controls.
//! It is not the tester's "until the dialog is spoken and takes keys". The
//! show, the focus landing and the screen reader's first announcement all
//! come after that moment, and nothing here can see them. The line the
//! application writes, `settings built in N ms`, spans exactly this.
//!
//! **A candidate** is one of the three things the build asks the machine for
//! rather than builds: `spellcheck::available_languages()`, which creates the
//! Windows spell-checker factory and asks it what it supports;
//! `fonts::installed_families()`, which walks every installed typeface; and
//! `SoundScheme::discover`, which reads the sound-schemes folder. Each is
//! timed five times on its own, in this process, before any window exists.
//! The first run is reported on its own, because it is what a process pays
//! the first time it asks, and the median of the five is the cost of asking
//! again.
//!
//! **The dialog build** is `build_settings_dialog` with a default `AppConfig`,
//! no accounts and no calendar server, in this process under
//! `wxdragon::main`, five times, each dialog destroyed before the next is
//! built, first and median. It includes the three candidates; the difference
//! between it and their sum is what building the pages of controls costs.
//! The frame it is built on is never shown, and that understates what the
//! running program pays: found on 2026-09-16 by showing the frame in a
//! scratch run, the same build cost three times as much, within three
//! percent of the release binary's own line, because a screen reader's
//! in-process hooks attach to a process with a window on screen and watch
//! every control's creation. The line below is the figure that counts; this
//! one is kept because it is the figure a runner with no screen reader can
//! take.
//!
//! **The line** is what the release binary writes. It is measured by starting
//! the binary against a throwaway profile, opening Settings through the same
//! menu command `Ctrl+,` sends, reading the line from the log, closing the
//! dialog, and doing that five times in one process: the first open on its
//! own, the median of the five beside it. No key is sent to any window this
//! test did not start.
//!
//! # Running the measurement
//!
//! ```text
//! cargo build --release
//! cargo test --release --test the_settings_dialog_opens_in -- --ignored --nocapture
//! ```
//!
//! On a machine doing nothing else. The test refuses a debug build, because a
//! debug figure is a figure about a binary nobody ships.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use wixen_mail::application::reading_habits::WHAT_MARK_READ_COUNTS_FROM;
use wixen_mail::common::paths::AppPaths;
use wixen_mail::common::started;
use wixen_mail::data::config::AppConfig;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::accessibility::sound_scheme::SoundScheme;
use wixen_mail::presentation::wx_settings;
use wixen_mail::service::fonts;
use wixen_mail::service::spellcheck::available_languages;
use wxdragon::prelude::*;

/// How many times each figure is taken.
const FIVE: usize = 5;

/// The command every row carries, backticked because the page's reading
/// refuses a command cell with no backticked token.
const THE_COMMAND: &str =
    "`cargo test --release --test the_settings_dialog_opens_in -- --ignored --nocapture`";

// ── The lines ───────────────────────────────────────────────────────────────

/// Word one figure: what was timed, its first run, the median of the runs,
/// and every run in the order they were taken.
///
/// Plain digits, no separators, because the summary and the page quote this
/// and a thousands separator is locale.
fn a_figure_line(what: &str, runs: &[Duration]) -> String {
    let first = runs.first().map_or(0, |run| run.as_millis());
    let median = median_of(runs).as_millis();
    let each = runs
        .iter()
        .map(|run| run.as_millis().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{what}: first {first} ms, median {median} ms, the {}: {each} ms",
        runs.len()
    )
}

/// The tabs after General, in the order the dialog adds them; each is built
/// the first time it is shown, and the harness times that first showing.
const THE_TABS_AFTER_GENERAL: [&str; 6] = [
    "Compose",
    "Reading",
    "Permissions",
    "Calendar & PIM",
    "Feedback",
    "Advanced",
];

/// Word the cost of showing each later tab for the first time, in the
/// order they were shown.
fn the_first_visits_line(visits: &[(String, Duration)]) -> String {
    let each = visits
        .iter()
        .map(|(name, took)| format!("{name} {} ms", took.as_millis()))
        .collect::<Vec<_>>()
        .join(", ");
    format!("first visit of each later tab: {each}")
}

/// Read the first-visits line back: each tab and its milliseconds.
fn parse_the_first_visits_line(line: &str) -> Option<Vec<(String, u64)>> {
    let (_, rest) = line.split_once("first visit of each later tab: ")?;
    rest.split(", ")
        .map(|one| {
            let (name, ms) = one.strip_suffix(" ms")?.rsplit_once(' ')?;
            Some((name.to_string(), ms.parse().ok()?))
        })
        .collect()
}

/// Read a figure line back: what was timed, its first run and its median.
fn parse_a_figure_line(line: &str) -> Option<(String, u64, u64)> {
    let (what, rest) = line.split_once(": first ")?;
    let (first, rest) = rest.split_once(" ms, median ")?;
    let (median, _) = rest.split_once(" ms, the ")?;
    Some((what.to_string(), first.parse().ok()?, median.parse().ok()?))
}

/// The line the application writes, read back as its milliseconds.
fn parse_the_settings_built_line(log: &str) -> Vec<u64> {
    log.lines()
        .filter_map(|line| {
            let (_, rest) = line.split_once("settings built in ")?;
            let (ms, _) = rest.split_once(" ms")?;
            ms.parse().ok()
        })
        .collect()
}

/// The middle run when they are sorted, or the upper middle of an even
/// count. Nothing for no runs.
fn median_of(runs: &[Duration]) -> Duration {
    let mut sorted = runs.to_vec();
    sorted.sort();
    sorted.get(sorted.len() / 2).copied().unwrap_or_default()
}

/// Time one call, `FIVE` times.
fn timed<T>(mut call: impl FnMut() -> T) -> Vec<Duration> {
    (0..FIVE)
        .map(|_| {
            let began = Instant::now();
            // The answer lives to the end of the block, so what it costs to
            // let go of is outside the timing.
            let _answer = call();
            began.elapsed()
        })
        .collect()
}

/// Word a row the page will accept: a pipe inside a cell is written `\|` so
/// the table stays a table.
fn the_row(what: &str, value: &str, date: &str, commit: &str, conditions: &str) -> String {
    let cell = |text: &str| text.replace('|', "\\|");
    format!(
        "| {} | {} | {} | {} | {} | {} |",
        cell(what),
        cell(value),
        cell(THE_COMMAND),
        cell(date),
        cell(commit),
        cell(conditions)
    )
}

/// The date, the commit and the version, for the rows.
fn today_commit_and_version() -> (String, String, String) {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let commit = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
        .ok()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    (date, commit, env!("CARGO_PKG_VERSION").to_string())
}

/// Refuse to measure a debug build.
fn refuse_a_debug_build() -> Result<(), String> {
    if cfg!(debug_assertions) {
        return Err(String::from(
            "this is a debug build and a debug figure is a figure about a binary nobody ships; \
             run with --release: cargo test --release --test the_settings_dialog_opens_in -- --ignored --nocapture",
        ));
    }
    Ok(())
}

// ── The throwaway profile ───────────────────────────────────────────────────

/// A profile with settings and nothing else: the alpha notice already shown,
/// so a start does not stop at that dialog, and no account, so nothing
/// reaches the credential store or a server. The sound-schemes folder is
/// created and empty, which is what a profile nobody imported a scheme into
/// holds.
fn a_throwaway_profile(into: &Path) -> Result<AppPaths, String> {
    let paths = AppPaths::under(into);
    paths.create().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(paths.sound_schemes_dir()).map_err(|e| e.to_string())?;
    let settings = AppConfig {
        told_about_the_alpha: true,
        ..AppConfig::default()
    };
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(paths.config_dir().join("app_config.json"), json).map_err(|e| e.to_string())?;
    Ok(paths)
}

/// The newest log file under the profile, read whole.
fn the_newest_log(paths: &AppPaths) -> Option<String> {
    let mut newest: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in std::fs::read_dir(paths.logs_dir()).ok()?.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "log")
            && let Ok(modified) = entry.metadata().and_then(|m| m.modified())
            && newest.as_ref().is_none_or(|(when, _)| modified > *when)
        {
            newest = Some((modified, path));
        }
    }
    std::fs::read_to_string(newest?.1).ok()
}

// ── The release binary, and the one command sent to it ──────────────────────

/// A started application, and the whole tree under it stopped when this is
/// dropped, on the failure path too.
struct Started {
    child: Child,
}

impl Started {
    /// Start the release binary against a profile, changing nothing at any
    /// server.
    fn against(profile: &Path) -> Result<Self, String> {
        let child = Command::new(env!("CARGO_BIN_EXE_wixen-mail"))
            .arg("--read-only")
            .env("WIXEN_MAIL_DATA", profile)
            .env_remove("RUST_LOG")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("the application did not start: {e}"))?;
        Ok(Self { child })
    }

    fn id(&self) -> u32 {
        self.child.id()
    }

    fn has_exited(&mut self) -> Option<String> {
        match self.child.try_wait() {
            Ok(Some(status)) => Some(format!("the application exited with {status}")),
            Ok(None) => None,
            Err(e) => Some(format!("could not ask whether the application is up: {e}")),
        }
    }
}

impl Drop for Started {
    fn drop(&mut self) {
        let _ = Command::new("taskkill")
            .args(["/PID", &self.child.id().to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = self.child.wait();
    }
}

/// How long to wait for a window, or for the line, before a run is reported
/// as having produced nothing.
const THE_LONGEST_WAIT: Duration = Duration::from_secs(60);

/// The main window's title and the dialog's, as the window layer sets them.
const THE_MAIN_WINDOW: &str = "Wixen Mail";
const THE_DIALOG: &str = "Settings";

/// The menu item `Ctrl+,` is the accelerator of, as the menu bar labels it.
const THE_MENU_ITEM: &str = "&Settings";

#[cfg(windows)]
mod windows_of {
    //! Finding the started process's own windows and its menu, and posting
    //! it the one command the accelerator would. By process id, never by
    //! title alone: the tester's own copy of this program may be running
    //! in a locked session on this machine, and nothing here may reach it.

    type Hwnd = isize;
    type Hmenu = isize;

    const WM_CLOSE: u32 = 0x0010;
    const WM_COMMAND: u32 = 0x0111;
    const MF_BYPOSITION: u32 = 0x0400;
    const GW_CHILD: u32 = 5;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn EnumWindows(each: extern "system" fn(Hwnd, isize) -> i32, carried: isize) -> i32;
        fn GetWindowThreadProcessId(window: Hwnd, process: *mut u32) -> u32;
        fn GetWindowTextW(window: Hwnd, text: *mut u16, most: i32) -> i32;
        fn IsWindowVisible(window: Hwnd) -> i32;
        fn GetMenu(window: Hwnd) -> Hmenu;
        fn GetMenuItemCount(menu: Hmenu) -> i32;
        fn GetSubMenu(menu: Hmenu, at: i32) -> Hmenu;
        fn GetMenuItemID(menu: Hmenu, at: i32) -> u32;
        fn GetMenuStringW(menu: Hmenu, item: u32, text: *mut u16, most: i32, by: u32) -> i32;
        fn PostMessageW(window: Hwnd, message: u32, w: usize, l: isize) -> i32;
        fn GetWindow(window: Hwnd, which: u32) -> Hwnd;
    }

    /// Whether the window has any child window at all, which is what a page
    /// with controls on it has and an empty panel has not.
    pub fn has_a_child(window: Hwnd) -> bool {
        // Safe: a query on a handle the toolkit gave us.
        unsafe { GetWindow(window, GW_CHILD) != 0 }
    }

    /// The window text of each direct child of the window, in sibling
    /// order: a label's words, a static text's sentence, a button's caption.
    pub fn child_texts(window: Hwnd) -> Vec<String> {
        const GW_HWNDNEXT: u32 = 2;
        let mut texts = Vec::new();
        // Safe: plain window queries on handles the toolkit gave us; every
        // buffer is passed with its length.
        unsafe {
            let mut child = GetWindow(window, GW_CHILD);
            while child != 0 {
                let mut text = [0u16; 512];
                let length = GetWindowTextW(child, text.as_mut_ptr(), text.len() as i32);
                texts.push(String::from_utf16_lossy(&text[..length.max(0) as usize]));
                child = GetWindow(child, GW_HWNDNEXT);
            }
        }
        texts
    }

    /// Every visible top-level window of one process, with its title.
    fn visible_windows_of(process: u32) -> Vec<(Hwnd, String)> {
        struct Walk {
            process: u32,
            found: Vec<(Hwnd, String)>,
        }
        extern "system" fn take_one(window: Hwnd, carried: isize) -> i32 {
            // Safe: `carried` is the address of the `Walk` below, which
            // outlives the enumeration it is driving.
            let walk = unsafe { &mut *(carried as *mut Walk) };
            let mut owner = 0u32;
            // Safe: plain window queries on a handle Windows just gave us.
            unsafe {
                GetWindowThreadProcessId(window, &raw mut owner);
                if owner == walk.process && IsWindowVisible(window) != 0 {
                    let mut title = [0u16; 256];
                    let length = GetWindowTextW(window, title.as_mut_ptr(), title.len() as i32);
                    let title = String::from_utf16_lossy(&title[..length.max(0) as usize]);
                    walk.found.push((window, title));
                }
            }
            1
        }
        let mut walk = Walk {
            process,
            found: Vec::new(),
        };
        // Safe: the callback and the address it is handed are both above.
        unsafe {
            EnumWindows(take_one, (&raw mut walk) as isize);
        }
        walk.found
    }

    /// The process's visible window whose title is exactly `title`.
    pub fn window_titled(process: u32, title: &str) -> Option<Hwnd> {
        visible_windows_of(process)
            .into_iter()
            .find(|(_, found)| found == title)
            .map(|(window, _)| window)
    }

    /// The id of the menu item whose label begins `label`, anywhere in the
    /// window's menu bar.
    pub fn menu_item_beginning(window: Hwnd, label: &str) -> Option<u32> {
        // Safe: a handle Windows gave us, queried and not changed.
        let bar = unsafe { GetMenu(window) };
        if bar == 0 {
            return None;
        }
        item_in(bar, label)
    }

    fn item_in(menu: Hmenu, label: &str) -> Option<u32> {
        // Safe: menu handles Windows gave us, queried and not changed; every
        // buffer is passed with its length.
        unsafe {
            for at in 0..GetMenuItemCount(menu).max(0) {
                let sub = GetSubMenu(menu, at);
                if sub != 0 {
                    if let Some(found) = item_in(sub, label) {
                        return Some(found);
                    }
                    continue;
                }
                let mut text = [0u16; 128];
                let length = GetMenuStringW(
                    menu,
                    at as u32,
                    text.as_mut_ptr(),
                    text.len() as i32,
                    MF_BYPOSITION,
                );
                let text = String::from_utf16_lossy(&text[..length.max(0) as usize]);
                if text.starts_with(label) {
                    return Some(GetMenuItemID(menu, at));
                }
            }
        }
        None
    }

    /// Post the window the menu command, which is what the accelerator does.
    pub fn post_menu_command(window: Hwnd, id: u32) -> bool {
        // Safe: a post to a handle Windows gave us; the result says whether
        // it was queued.
        unsafe { PostMessageW(window, WM_COMMAND, id as usize, 0) != 0 }
    }

    /// Ask the window to close, which a dialog answers as Cancel.
    pub fn post_close(window: Hwnd) -> bool {
        // Safe: as above.
        unsafe { PostMessageW(window, WM_CLOSE, 0, 0) != 0 }
    }
}

/// Wait for something, polling, or say what never happened.
fn wait_for<T>(what: &str, mut ask: impl FnMut() -> Option<T>) -> Result<T, String> {
    let deadline = Instant::now() + THE_LONGEST_WAIT;
    while Instant::now() < deadline {
        if let Some(found) = ask() {
            return Ok(found);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    Err(format!(
        "{what} did not happen within {} s",
        THE_LONGEST_WAIT.as_secs()
    ))
}

/// Open Settings in the started application `FIVE` times through its own
/// menu command, closing the dialog each time, and hand back the line's
/// figures in the order they were written.
#[cfg(windows)]
fn the_lines_the_release_binary_writes(
    paths: &AppPaths,
    profile: &Path,
) -> Result<Vec<u64>, String> {
    let mut started = Started::against(profile)?;
    let process = started.id();
    let main_window = wait_for("the main window", || {
        windows_of::window_titled(process, THE_MAIN_WINDOW)
    })?;
    let settings_command = windows_of::menu_item_beginning(main_window, THE_MENU_ITEM)
        .ok_or_else(|| format!("no menu item beginning {THE_MENU_ITEM:?} on the main window"))?;
    // The window is up before its startup work is done; give the log its
    // first lines and the module its fill before the first open, so the
    // first figure is an open on an idle program rather than one racing
    // startup.
    std::thread::sleep(Duration::from_secs(3));

    let mut figures = Vec::new();
    for open in 0..FIVE {
        if let Some(gone) = started.has_exited() {
            return Err(format!("before open {}: {gone}", open + 1));
        }
        if !windows_of::post_menu_command(main_window, settings_command) {
            return Err("the Settings command could not be posted".to_string());
        }
        let so_far = figures.len();
        let now_written = wait_for("the settings built line", || {
            let written = parse_the_settings_built_line(&the_newest_log(paths)?);
            (written.len() > so_far).then_some(written)
        })?;
        figures = now_written;
        let dialog = wait_for("the Settings dialog", || {
            windows_of::window_titled(process, THE_DIALOG)
        })?;
        if !windows_of::post_close(dialog) {
            return Err("the Settings dialog could not be asked to close".to_string());
        }
        wait_for("the Settings dialog closing", || {
            windows_of::window_titled(process, THE_DIALOG)
                .is_none()
                .then_some(())
        })?;
    }
    drop(started);
    Ok(figures)
}

#[cfg(not(windows))]
fn the_lines_the_release_binary_writes(_: &AppPaths, _: &Path) -> Result<Vec<u64>, String> {
    Err(
        "the release binary is driven through its Windows menu, and this is not Windows"
            .to_string(),
    )
}

// ── The measurement, behind #[ignore] ───────────────────────────────────────

#[test]
#[ignore = "times the real dialog and starts the release binary; run by hand on a quiet machine"]
fn test_how_long_the_settings_dialog_takes_to_build_and_which_part_takes_it() {
    refuse_a_debug_build().expect("a release build");
    let home = tempfile::tempdir().expect("a temporary folder");
    let paths = a_throwaway_profile(home.path()).expect("the profile writes");
    // The dialog's sound-scheme picker reads the profile this process
    // resolves, and so does the release binary below through its own
    // environment. Set here, once, before anything reads it; this is the
    // only test in the binary that runs under `--ignored`, and no test that
    // runs on every commit reads the environment.
    // Safe: nothing else in this process is reading or writing the
    // environment at this point.
    unsafe { std::env::set_var("WIXEN_MAIL_DATA", home.path()) };

    let (date, commit, version) = today_commit_and_version();
    println!();
    println!("== the Settings dialog: {version} at {commit} on {date}");

    // The three candidates, each on its own, no window.
    let languages = timed(available_languages);
    let families = timed(|| fonts::installed_families().unwrap_or_default());
    let schemes_dir = paths.sound_schemes_dir();
    let schemes = timed(|| SoundScheme::discover(&schemes_dir));
    let how_many_languages = available_languages().len();
    let how_many_families = fonts::installed_families().map_or(0, |found| found.len());
    println!("{}", a_figure_line("available_languages", &languages));
    println!("{}", a_figure_line("installed_families", &families));
    println!("{}", a_figure_line("SoundScheme::discover", &schemes));
    println!(
        "the machine offers {how_many_languages} spelling languages and {how_many_families} typeface families; the schemes folder is empty"
    );

    // The dialog, built five times in the one window this process may have,
    // then one more time to show each later tab once, which is where the
    // pages after General are built since 09-09: the cost of reaching each
    // tab for the first time is the cost that moved out of the open.
    let builds: Arc<std::sync::Mutex<Vec<Duration>>> = Arc::default();
    let first_visits: Arc<std::sync::Mutex<Vec<(String, Duration)>>> = Arc::default();
    let result = {
        let builds = builds.clone();
        let first_visits = first_visits.clone();
        wxdragon::main(move |app| {
            let frame = Frame::builder().build();
            let a11y = Arc::new(Accessibility::new().expect("accessibility"));
            let config = AppConfig::default();
            let took = timed(|| {
                let widgets =
                    wx_settings::build_settings_dialog(&frame, &config, &[], false, &a11y);
                widgets.dialog.destroy();
            });
            *builds.lock().expect("the builds") = took;

            let widgets = wx_settings::build_settings_dialog(&frame, &config, &[], false, &a11y);
            let visits = THE_TABS_AFTER_GENERAL
                .iter()
                .enumerate()
                .map(|(before, name)| {
                    let began = Instant::now();
                    widgets.notebook.set_selection(before + 1);
                    (name.to_string(), began.elapsed())
                })
                .collect();
            *first_visits.lock().expect("the visits") = visits;
            widgets.dialog.destroy();
            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");
    let builds = builds.lock().expect("the builds").clone();
    println!("{}", a_figure_line("build_settings_dialog", &builds));
    let first_visits = first_visits.lock().expect("the visits").clone();
    println!("{}", the_first_visits_line(&first_visits));

    // The line the release binary writes, five opens in one process.
    let lines = the_lines_the_release_binary_writes(&paths, home.path())
        .expect("the release binary opened Settings and said how long it took");
    let opens: Vec<Duration> = lines.iter().map(|ms| Duration::from_millis(*ms)).collect();
    println!(
        "{}",
        a_figure_line("settings built in, the release binary", &opens)
    );

    let conditions = format!(
        "{version} at {commit}; fill in the machine, the build, what else was running and the boundary"
    );
    for (what, runs) in [
        ("Settings: available_languages() on its own", &languages),
        (
            "Settings: fonts::installed_families() on its own",
            &families,
        ),
        (
            "Settings: SoundScheme::discover on an empty folder",
            &schemes,
        ),
        ("Settings: build_settings_dialog in a test process", &builds),
        ("Settings built, the release binary's own line", &opens),
    ] {
        println!(
            "{}",
            the_row(
                what,
                &format!(
                    "first {} ms, median {} ms",
                    runs.first().map_or(0, |run| run.as_millis()),
                    median_of(runs).as_millis()
                ),
                &date,
                &commit,
                &conditions
            )
        );
    }
}

// ── The tests that run on every commit ──────────────────────────────────────

#[test]
fn test_the_settings_built_line_the_library_writes_parses_back() {
    let log = format!(
        "2026-09-16T10:00:00Z  INFO wixen_mail::presentation::wx_settings: {}\n",
        started::settings_built_line(Duration::from_millis(321))
    );

    assert_eq!(
        parse_the_settings_built_line(&log),
        vec![321],
        "the harness reads back the line the library writes"
    );
}

#[test]
fn test_a_figure_line_carries_its_unit_and_parses_back() {
    let runs: Vec<Duration> = [50, 10, 40, 20, 30]
        .into_iter()
        .map(Duration::from_millis)
        .collect();

    let line = a_figure_line("available_languages", &runs);

    assert_eq!(
        line, "available_languages: first 50 ms, median 30 ms, the 5: 50, 10, 40, 20, 30 ms",
        "every figure carries its unit, because a number without one is a number somebody guesses at"
    );
    assert_eq!(
        parse_a_figure_line(&line),
        Some(("available_languages".to_string(), 50, 30)),
        "and the summary can read it back"
    );
}

#[test]
fn test_the_first_visits_line_carries_its_units_and_parses_back() {
    let visits = vec![
        ("Compose".to_string(), Duration::from_millis(20)),
        ("Calendar & PIM".to_string(), Duration::from_millis(75)),
    ];

    let line = the_first_visits_line(&visits);

    assert_eq!(
        line, "first visit of each later tab: Compose 20 ms, Calendar & PIM 75 ms",
        "every tab's figure carries its unit, and a tab name may hold spaces"
    );
    assert_eq!(
        parse_the_first_visits_line(&line),
        Some(vec![
            ("Compose".to_string(), 20),
            ("Calendar & PIM".to_string(), 75)
        ]),
        "and the summary can read each tab back"
    );
}

// ── What the before rows justified, held on every commit ────────────────────
//
// The rows of 2026-09-16 said the three lists cost a millisecond each and the
// pages of controls cost the rest, so the change is to the build and not to
// the lists: the dialog is frozen while it is built, the spelling sentence is
// worded without building a checker, and only the first page is built before
// the dialog is shown. The tests below hold each of those. The one that shows
// the Reading page also reads, since 2026-09-18, that the sentence under Mark
// as read after is on the built page (#25), because that is the one test here
// that builds the page in a window session.

/// Which page of the notebook is which, by the order the dialog adds them.
const THE_READING_PAGE: usize = 2;

#[cfg(windows)]
#[test]
fn test_pages_after_the_first_are_built_when_their_tab_is_first_shown_and_read_from_the_settings_when_never_shown()
 {
    use std::sync::Mutex;

    let wrong: Arc<Mutex<Vec<String>>> = Arc::default();
    let result = {
        let wrong = wrong.clone();
        wxdragon::main(move |app| {
            let mut wrong = wrong.lock().expect("the findings");
            let frame = Frame::builder().build();
            let a11y = Arc::new(Accessibility::new().expect("accessibility"));
            // A stored answer on a page nobody visits, so what OK writes
            // back for that page is visible.
            let config = AppConfig {
                default_sort_order: "sender_az".to_string(),
                ..AppConfig::default()
            };
            let widgets = wx_settings::build_settings_dialog(&frame, &config, &[], false, &a11y);

            if widgets.notebook.get_page_count() != 7 {
                wrong.push(format!(
                    "the tab row holds {} tabs and there are seven pages, so a setting is off the screen",
                    widgets.notebook.get_page_count()
                ));
            }
            let reading = widgets.reading_panel.get_handle() as isize;
            if windows_of::has_a_child(reading) {
                wrong.push(
                    "the Reading page has controls on it before its tab was ever shown, so it was built up front"
                        .to_string(),
                );
            }
            let never_shown = wx_settings::read_settings(&widgets, &config);
            if never_shown.default_sort_order != "sender_az" {
                wrong.push(format!(
                    "OK on a dialog whose Reading tab was never shown wrote {:?} over the stored sender_az",
                    never_shown.default_sort_order
                ));
            }

            widgets.notebook.set_selection(THE_READING_PAGE);
            if !windows_of::has_a_child(reading) {
                wrong.push(
                    "the Reading tab was shown and its page still has no controls on it"
                        .to_string(),
                );
            }
            // The sentence under Mark as read after is on the built page
            // (#25, 11-05): the wait is counted from reading, never from
            // moving onto a row, and the choice cannot say that on its own.
            if !windows_of::child_texts(reading)
                .iter()
                .any(|text| text == WHAT_MARK_READ_COUNTS_FROM)
            {
                wrong.push(format!(
                    "the built Reading page holds no static text saying {WHAT_MARK_READ_COUNTS_FROM:?} under Mark as read after"
                ));
            }
            widgets.reading().sort_order.set_selection(0);
            let after_a_change = wx_settings::read_settings(&widgets, &config);
            if after_a_change.default_sort_order != "date_newest" {
                wrong.push(format!(
                    "the sort order was changed on the shown Reading page and OK wrote {:?}",
                    after_a_change.default_sort_order
                ));
            }

            widgets.dialog.destroy();
            drop(wrong);
            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };
    assert!(result.is_ok(), "wxdragon::main returned {result:?}");

    let wrong = wrong.lock().expect("the findings");
    assert!(
        wrong.is_empty(),
        "the pages after the first are not built when their tab is first shown:\n  {}",
        wrong.join("\n  ")
    );
}

/// The window layer's source, for the two readings below.
fn the_settings_source() -> String {
    std::fs::read_to_string("src/presentation/wx_settings.rs")
        .expect("src/presentation/wx_settings.rs is read from the repository root")
}

/// The body of one function in a source file: from its `fn name(` to the
/// next line that is exactly `}`.
fn body_of<'a>(source: &'a str, name: &str) -> &'a str {
    let opening = format!("fn {name}(");
    let from = source
        .find(&opening)
        .unwrap_or_else(|| panic!("{opening} is not in the source"));
    let rest = &source[from..];
    let to = rest
        .find("\n}\n")
        .unwrap_or_else(|| panic!("{opening} has no closing brace on a line of its own"));
    &rest[..to]
}

#[test]
fn test_the_dialog_and_each_later_page_are_built_frozen() {
    // `wxChoice::DoInsertItems` resizes the control after every item it is
    // handed unless the control is frozen, and a child added under a frozen
    // window is frozen with it. Measured on 2026-09-16: the typeface list
    // alone cost about a second of the build with the window shown, and a
    // tenth of that frozen. So the dialog is frozen before the first page is
    // built and thawed after the last, and a page built later on its own
    // panel is frozen the same way.
    let source = the_settings_source();
    let build = body_of(&source, "build_settings_dialog");

    let frozen_at = build.find("dlg.freeze();");
    let first_page_at = build.find("notebook.add_page(");
    let thawed_at = build.find("dlg.thaw();");
    let laid_out_at = build.find("dlg.set_sizer(");
    assert!(
        matches!((frozen_at, first_page_at), (Some(frozen), Some(page)) if frozen < page),
        "build_settings_dialog does not freeze the dialog before its first page is built"
    );
    assert!(
        matches!((laid_out_at, thawed_at), (Some(laid_out), Some(thawed)) if laid_out < thawed),
        "build_settings_dialog does not thaw the dialog after it is laid out"
    );

    let later = body_of(&source, "build_the_page_for");
    assert!(
        later.find("panel.freeze();").is_some_and(|frozen| {
            later
                .find("panel.thaw();")
                .is_some_and(|thawed| frozen < thawed)
        }),
        "a page built when its tab is first shown is not built frozen and thawed after"
    );
}

#[test]
fn test_the_spelling_sentence_is_worded_without_building_a_checker() {
    // Letting go of a Windows spell checker costs about a fifth of a second
    // (measured 2026-09-16), and the sentence "Spelling is checked by ..."
    // only needs to know which checker would be built.
    let source = the_settings_source();
    let language = body_of(&source, "add_language_and_spelling");

    assert!(
        language.contains("spellcheck::source_for_language("),
        "the spelling sentence is not worded from source_for_language"
    );
    assert!(
        !language.contains("spellcheck::for_language("),
        "add_language_and_spelling still builds a checker to say which one it is"
    );
}

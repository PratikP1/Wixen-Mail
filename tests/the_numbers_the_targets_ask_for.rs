//! The instrument for PERF-01, PERF-02 and PERF-04, and the definitions the
//! numbers are taken under.
//!
//! Three targets have sat in the requirements with no measurement behind
//! them: memory under 150 MB with 1,000 cached messages, cold start under
//! 2 seconds to a usable list, and idle memory under 100 MB. Nothing in the
//! tree built a cache of a known size, timed a start, or read a working set.
//! This target does all three, and the tests that run on every commit hold
//! the profile writers, the parsers and the row shape, so the measurement
//! tests behind `#[ignore]` print rows the page will accept.
//!
//! # The definitions
//!
//! Each of the requirements uses a word it does not define. These are the
//! meanings the numbers on `docs/development/measurements.md` were taken
//! under, written here once and quoted there, so the next person re-taking a
//! number takes the same one.
//!
//! **1,000 cached messages** means 1,000 rows in `messages` for one IMAP
//! account's `INBOX`, written through the same call the sync uses, with a
//! plain-text body of about 2 KB each through the body store, no attachment
//! content and no signed originals. Because that is what the list reads at
//! startup and what the target was written about; the attachment and
//! signed-original budgets are constants with their own rows and their own
//! limits, and mixing them in would measure the budgets rather than the
//! messages.
//!
//! **Cold start** means a fresh process of the release binary against the
//! 1,000-message profile, from the first instruction of `main` to the usable
//! line. The first start after the binary is built is reported on its own as
//! the file-cache-cold figure; the next five starts are the series and their
//! median is the number. The requirement says "cold" and does not say cold
//! for what; both readings are given so nobody has to guess which was meant.
//!
//! **Usable** means the first `MessagesLoaded` after startup whose rows
//! reached the list control's count and were at least one, which is the line
//! `wixen_mail::common::started` writes once per process.
//!
//! **Idle** means no input after the usable line, the window left where the
//! start put it, and memory read at 60 s and at 120 s; the 120 s reading is
//! the number, the 60 s reading sits beside it, and the difference is
//! reported as idle growth.
//!
//! **Memory with 1,000 cached messages** is the peak working set of the
//! application process between start and the 60 s reading, plus the WebView2
//! tree's working set at 60 s, on the 1,000-message profile, because "with
//! 1,000 cached messages" is about what loading them costs and the peak is
//! when it cost most. The empty profile's 120 s reading is the floor and is
//! reported beside both.
//!
//! # What the harness does
//!
//! It writes a profile into a `tempfile` directory, starts the release
//! binary against it with `WIXEN_MAIL_DATA` and `--read-only`, polls the
//! newest log file for the usable line, reads the working set of the process
//! and of every process under it through PowerShell, stops the whole tree,
//! and prints rows in the page's shape. Memory is read through the operating
//! system rather than a crate, because the OS already answers this for a
//! started process and a dependency for one number is a dependency too many.
//!
//! The profile's account points at `127.0.0.1` on a closed port, so the
//! startup connection is refused at once rather than timing out. Its
//! settings say the alpha notice has been shown, or every start would stop at
//! that dialog, and that the window opens on All Inboxes, because a fresh
//! profile otherwise opens with no folder chosen and no list ever loads.
//!
//! # Running the measurement
//!
//! ```text
//! cargo build --release
//! cargo test --release --test the_numbers_the_targets_ask_for -- --ignored --nocapture
//! ```
//!
//! On a machine doing nothing else. The tests refuse a debug build, because
//! a debug figure is a figure about a binary nobody ships.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use wixen_mail::common::paths::AppPaths;
use wixen_mail::common::started;
use wixen_mail::common::types::FolderType;
use wixen_mail::data::account::Account;
use wixen_mail::data::config::AppConfig;
use wixen_mail::data::message_cache::{CachedFolder, IncomingMessage, MessageCache};
use wixen_mail::service::safety::Verdict;

// ── The profiles ────────────────────────────────────────────────────────────

/// The account every measurement profile holds.
const THE_MEASUREMENT_ACCOUNT: &str = "perf-measurement";

/// The one folder the profile holds.
const THE_FOLDER: &str = "INBOX";

/// How many rows the measurement profile holds.
const A_THOUSAND: usize = 1_000;

/// How many rows the profile the gate builds on every commit holds.
const A_FEW: usize = 10;

/// A port nothing listens on, so the startup connection is refused at once.
const A_CLOSED_PORT: &str = "1";

/// About how many bytes each plain-text body holds.
const ABOUT_TWO_KILOBYTES: usize = 2_048;

/// See `a_profile_with`: the credential store is reached one thread at a time.
static ONE_ACCOUNT_WRITE_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Write a profile holding `how_many` cached messages, as the definition
/// above says: settings that open on the list, one refusable IMAP account,
/// one `INBOX`, the rows, and a plain-text body for each.
fn a_profile_with(into: &Path, how_many: usize) -> Result<(), String> {
    let paths = AppPaths::under(into);
    paths.create().map_err(|e| e.to_string())?;
    write_the_settings(&paths)?;

    let cache = MessageCache::new(paths.cache_dir(), None).map_err(|e| e.to_string())?;
    {
        // One at a time. Saving an account reaches the Windows credential
        // store even with an empty password, because an empty password is a
        // request to forget, and keyring 4.1.5's `Entry::new` races its own
        // lazy initialisation when several threads reach it together
        // (ledger 374: "No default store has been set", about one run in
        // three). The seam that would keep an integration test out of the
        // real store is `cfg(test)` and this target cannot see it.
        let _one_at_a_time = ONE_ACCOUNT_WRITE_AT_A_TIME
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        cache
            .save_account(&the_refusable_account())
            .map_err(|e| e.to_string())?;
    }
    let folder_id = cache
        .save_folder(&CachedFolder {
            id: 0,
            account_id: THE_MEASUREMENT_ACCOUNT.to_string(),
            name: THE_FOLDER.to_string(),
            path: THE_FOLDER.to_string(),
            folder_type: FolderType::Inbox.as_str().to_string(),
            unread_count: 0,
            total_count: 0,
        })
        .map_err(|e| e.to_string())?;

    let arriving: Vec<IncomingMessage> = (0..how_many).map(|n| a_message(folder_id, n)).collect();
    let row_ids = cache
        .upsert_messages(&arriving)
        .map_err(|e| e.to_string())?;
    for (n, row_id) in row_ids.into_iter().enumerate() {
        cache
            .save_message_body(row_id, Some(&a_body(n)), None)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Write a profile holding the settings and nothing else.
fn an_empty_profile(into: &Path) -> Result<(), String> {
    let paths = AppPaths::under(into);
    paths.create().map_err(|e| e.to_string())?;
    write_the_settings(&paths)
}

/// The settings file the application reads, saying the alpha notice has been
/// shown and the window opens on All Inboxes.
///
/// Written as the file rather than through `ConfigManager`, which resolves
/// its folder from the environment and would write into whoever's profile
/// the environment names.
fn write_the_settings(paths: &AppPaths) -> Result<(), String> {
    let settings = AppConfig {
        told_about_the_alpha: true,
        start_in_all_inboxes: true,
        ..AppConfig::default()
    };
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(paths.config_dir().join("app_config.json"), json).map_err(|e| e.to_string())
}

/// One IMAP account whose server refuses at once and whose password is empty,
/// so nothing is written to the credential store and nothing waits on a
/// timeout.
fn the_refusable_account() -> Account {
    Account {
        id: THE_MEASUREMENT_ACCOUNT.to_string(),
        name: "The measurement account".to_string(),
        email: "measurement@127.0.0.1".to_string(),
        imap_server: "127.0.0.1".to_string(),
        imap_port: A_CLOSED_PORT.to_string(),
        imap_use_tls: false,
        smtp_server: "127.0.0.1".to_string(),
        smtp_port: A_CLOSED_PORT.to_string(),
        smtp_use_tls: false,
        username: "measurement".to_string(),
        password: String::new(),
        ..Account::default()
    }
}

/// The `n`th message: dated in descending order, a quarter unread, one in
/// seven with an attachment flag, and the envelope fields the sync fills.
fn a_message(folder_id: i64, n: usize) -> IncomingMessage {
    let uid = u32::try_from(n + 1).unwrap_or(u32::MAX);
    let minutes_ago = n as i64;
    let date = chrono::DateTime::parse_from_rfc3339("2026-09-14T12:00:00+00:00")
        .expect("a fixed date")
        - chrono::Duration::minutes(minutes_ago);
    IncomingMessage {
        folder_id,
        uid,
        message_id: format!("<measurement-{uid}@127.0.0.1>"),
        subject: format!("Measurement message {uid}"),
        from_addr: format!("Sender {} <sender{}@example.com>", n % 50, n % 50),
        to_addr: "measurement@127.0.0.1".to_string(),
        cc: None,
        reply_to: None,
        date: date.to_rfc3339(),
        internal_date: Some(date.to_rfc3339()),
        size_bytes: Some(ABOUT_TWO_KILOBYTES as i64 + 600),
        refs_header: None,
        read: !n.is_multiple_of(4),
        starred: false,
        answered: false,
        draft: false,
        deleted: false,
        has_attachments: n.is_multiple_of(7),
        safety: Verdict::ordinary(),
        gmail_message_id: None,
        labels: None,
        receipt_to: None,
        list_unsubscribe: None,
        pop_uidl: None,
    }
}

/// A plain-text body of about 2 KB, different for each message so the store
/// cannot share them.
fn a_body(n: usize) -> String {
    let sentence = format!("This is measurement message {n} and it says nothing of interest. ");
    let mut body = String::with_capacity(ABOUT_TWO_KILOBYTES + sentence.len());
    while body.len() < ABOUT_TWO_KILOBYTES {
        body.push_str(&sentence);
    }
    body
}

// ── The parsers ─────────────────────────────────────────────────────────────

/// What the usable line starts with, after the log's own prefix.
const THE_USABLE_LINE_BEGINS: &str = "the message list is usable: ";

/// The rows and the milliseconds out of the usable line, wherever it sits
/// in a log.
fn parse_usable_line(log: &str) -> Option<(usize, u64)> {
    let line = log.lines().find_map(|line| {
        line.split_once(THE_USABLE_LINE_BEGINS)
            .map(|(_, rest)| rest)
    })?;
    let (rows, rest) = line.split_once(" rows, ")?;
    let (millis, _) = rest.split_once(" ms after start")?;
    Some((rows.parse().ok()?, millis.parse().ok()?))
}

/// One process's memory, as `Get-Process` reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Memory {
    working_set: u64,
    peak_working_set: u64,
    private_bytes: u64,
}

impl Memory {
    /// Bytes as the page writes them: whole megabytes, `1 MB = 1,048,576`.
    fn megabytes(bytes: u64) -> u64 {
        bytes / (1024 * 1024)
    }
}

/// Read one process's memory out of
/// `Get-Process -Id N | Select-Object WorkingSet64,PeakWorkingSet64,PrivateMemorySize64 | Format-List`.
fn parse_memory(powershell_output: &str) -> Result<Memory, String> {
    let field = |name: &str| -> Result<u64, String> {
        powershell_output
            .lines()
            .find_map(|line| {
                let (key, value) = line.split_once(':')?;
                (key.trim() == name).then(|| value.trim().parse::<u64>().ok())?
            })
            .ok_or_else(|| format!("no {name} in {powershell_output:?}"))
    };
    Ok(Memory {
        working_set: field("WorkingSet64")?,
        peak_working_set: field("PeakWorkingSet64")?,
        private_bytes: field("PrivateMemorySize64")?,
    })
}

/// The memory of a tree of processes, summed.
fn sum_tree(rows: &[Memory]) -> Memory {
    rows.iter().fold(Memory::default(), |sum, row| Memory {
        working_set: sum.working_set + row.working_set,
        peak_working_set: sum.peak_working_set + row.peak_working_set,
        private_bytes: sum.private_bytes + row.private_bytes,
    })
}

/// The processes under `root`, to any depth, out of `(id, parent, name)` rows.
fn descendants_of(root: u32, processes: &[(u32, u32, String)]) -> Vec<u32> {
    let mut found = Vec::new();
    let mut frontier = vec![root];
    while let Some(parent) = frontier.pop() {
        for (id, parent_of, _) in processes {
            if *parent_of == parent && *id != root && !found.contains(id) {
                found.push(*id);
                frontier.push(*id);
            }
        }
    }
    found
}

// ── The row ─────────────────────────────────────────────────────────────────

/// One cell per column of `docs/development/measurements.md`, in its order.
struct Row<'a> {
    what: &'a str,
    value: &'a str,
    command: &'a str,
    date: &'a str,
    commit: &'a str,
    conditions: &'a str,
}

/// Word a row the page will accept: a pipe inside a cell is written `\|` so
/// the table stays a table.
fn the_row(row: &Row<'_>) -> String {
    let cell = |text: &str| text.replace('|', "\\|");
    format!(
        "| {} | {} | {} | {} | {} | {} |",
        cell(row.what),
        cell(row.value),
        cell(row.command),
        cell(row.date),
        cell(row.commit),
        cell(row.conditions)
    )
}

/// Refuse to measure a debug build.
fn refuse_a_debug_build() -> Result<(), String> {
    if cfg!(debug_assertions) {
        return Err(String::from(
            "this is a debug build and a debug figure is a figure about a binary nobody ships; \
             run with --release: cargo test --release --test the_numbers_the_targets_ask_for -- --ignored --nocapture",
        ));
    }
    Ok(())
}

// ── The harness ─────────────────────────────────────────────────────────────

/// How long to wait for the usable line before a run is reported as having
/// produced nothing.
const THE_LONGEST_WAIT_FOR_USABLE: Duration = Duration::from_secs(60);

/// The three moments memory is read at, after the usable line and from the
/// start.
const AFTER_USABLE: Duration = Duration::from_secs(5);
const THE_FIRST_IDLE_READING: Duration = Duration::from_secs(60);
const THE_SECOND_IDLE_READING: Duration = Duration::from_secs(120);

/// A started application, and the whole tree under it stopped when this is
/// dropped, on the failure path too.
struct Started {
    child: Child,
    began: Instant,
}

impl Started {
    /// Start the release binary against a profile, changing nothing anywhere.
    fn against(profile: &Path) -> Result<Self, String> {
        let child = Command::new(env!("CARGO_BIN_EXE_wixen-mail"))
            .arg("--read-only")
            .env("WIXEN_MAIL_DATA", profile)
            // The application's own filter, not whatever the shell had.
            .env_remove("RUST_LOG")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("the application did not start: {e}"))?;
        Ok(Self {
            child,
            began: Instant::now(),
        })
    }

    fn id(&self) -> u32 {
        self.child.id()
    }

    /// Whether the process has already gone, which is a run that produced
    /// nothing.
    fn has_exited(&mut self) -> Option<String> {
        match self.child.try_wait() {
            Ok(Some(status)) => Some(format!("the application exited with {status}")),
            Ok(None) => None,
            Err(e) => Some(format!("could not ask whether the application is up: {e}")),
        }
    }

    /// Sleep until `since_start` has passed since the process was started.
    fn wait_until(&self, since_start: Duration) {
        if let Some(left) = since_start.checked_sub(self.began.elapsed()) {
            std::thread::sleep(left);
        }
    }
}

impl Drop for Started {
    fn drop(&mut self) {
        // The whole tree, because WebView2's processes are not this one's
        // children in any sense `Child::kill` knows about.
        let _ = Command::new("taskkill")
            .args(["/PID", &self.child.id().to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        let _ = self.child.wait();
    }
}

/// Run a PowerShell command and hand back what it printed.
fn powershell(command: &str) -> Result<String, String> {
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", command])
        .output()
        .map_err(|e| format!("powershell did not run: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "powershell refused `{command}`: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// One process's memory, read now.
fn memory_of(id: u32) -> Result<Memory, String> {
    parse_memory(&powershell(&format!(
        "Get-Process -Id {id} | Select-Object WorkingSet64,PeakWorkingSet64,PrivateMemorySize64 | Format-List"
    ))?)
}

/// Every process on the machine as `(id, parent, name)`.
fn every_process() -> Result<Vec<(u32, u32, String)>, String> {
    let listed = powershell(
        "Get-CimInstance Win32_Process | ForEach-Object { '{0} {1} {2}' -f $_.ProcessId, $_.ParentProcessId, $_.Name }",
    )?;
    Ok(listed
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let id = parts.next()?.parse().ok()?;
            let parent = parts.next()?.parse().ok()?;
            let name = parts.collect::<Vec<_>>().join(" ");
            Some((id, parent, name))
        })
        .collect())
}

/// The application and its tree, read at one moment.
#[derive(Debug, Clone)]
struct AReading {
    application: Memory,
    /// Every process under the application, summed; WebView2's, in practice.
    tree: Memory,
    /// Each process in the tree by name, with its working set, so a reading
    /// says what the tree is made of and not only what it weighs.
    members: Vec<(String, Memory)>,
}

fn read_the_application_and_its_tree(id: u32) -> Result<AReading, String> {
    let application = memory_of(id)?;
    let processes = every_process()?;
    let under = descendants_of(id, &processes);
    let mut members = Vec::with_capacity(under.len());
    for child in &under {
        let name = processes
            .iter()
            .find(|(each, _, _)| each == child)
            .map_or("unnamed", |(_, _, name)| name.as_str())
            .to_string();
        // A process that went between the listing and the reading is a
        // process that weighs nothing now, which is the true answer.
        if let Ok(memory) = memory_of(*child) {
            members.push((name, memory));
        }
    }
    let rows: Vec<Memory> = members.iter().map(|(_, memory)| *memory).collect();
    Ok(AReading {
        application,
        tree: sum_tree(&rows),
        members,
    })
}

/// The newest log file under the profile, read whole.
fn the_newest_log(profile: &Path) -> Option<String> {
    let logs = AppPaths::under(profile).logs_dir();
    let mut newest: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in std::fs::read_dir(logs).ok()?.flatten() {
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

/// Wait for the usable line, or say what happened instead.
fn wait_for_usable(started: &mut Started, profile: &Path) -> Result<(usize, u64), String> {
    let deadline = Instant::now() + THE_LONGEST_WAIT_FOR_USABLE;
    while Instant::now() < deadline {
        if let Some(gone) = started.has_exited() {
            return Err(format!("this run produced nothing: {gone}"));
        }
        if let Some(found) = the_newest_log(profile)
            .as_deref()
            .and_then(parse_usable_line)
        {
            return Ok(found);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Err(format!(
        "this run produced nothing: no usable line within {} s; the log held:\n{}",
        THE_LONGEST_WAIT_FOR_USABLE.as_secs(),
        the_newest_log(profile).unwrap_or_default()
    ))
}

/// What one run of the harness measured.
struct ARun {
    rows_in_the_list: usize,
    usable_after_ms: u64,
    after_usable: AReading,
    at_sixty: AReading,
    at_one_twenty: AReading,
    /// Every WARN and ERROR line the application wrote, so the summary can
    /// say what it did with the refused connection rather than guess.
    complaints: Vec<String>,
}

/// The WARN and ERROR lines of a log, without their timestamps.
fn complaints_in(log: &str) -> Vec<String> {
    log.lines()
        .filter(|line| line.contains(" WARN ") || line.contains(" ERROR "))
        .map(|line| {
            line.split_once(" ")
                .map_or(line, |(_, rest)| rest)
                .trim()
                .to_string()
        })
        .collect()
}

/// Read memory at the three moments after the usable line, then take the
/// log's complaints, refusing a process that went before the last reading.
fn the_three_readings(
    started: &mut Started,
    profile: &Path,
) -> Result<(AReading, AReading, AReading, Vec<String>), String> {
    started.wait_until(started.began.elapsed() + AFTER_USABLE);
    if let Some(gone) = started.has_exited() {
        return Err(format!("this run produced nothing: {gone}"));
    }
    let after_usable = read_the_application_and_its_tree(started.id())?;
    started.wait_until(THE_FIRST_IDLE_READING);
    let at_sixty = read_the_application_and_its_tree(started.id())?;
    started.wait_until(THE_SECOND_IDLE_READING);
    let at_one_twenty = read_the_application_and_its_tree(started.id())?;
    if let Some(gone) = started.has_exited() {
        return Err(format!(
            "this run produced nothing: {gone} before the last reading"
        ));
    }
    let log = the_newest_log(profile).unwrap_or_default();
    // The whole log, when asked for, so a reading can say what the
    // application did at startup and not only that it did not complain.
    if std::env::var_os(SHOW_THE_LOG).is_some() {
        println!("the log, whole:");
        for line in log.lines() {
            println!("  {line}");
        }
    }
    Ok((after_usable, at_sixty, at_one_twenty, complaints_in(&log)))
}

/// Set this to have a run print the application's whole log.
const SHOW_THE_LOG: &str = "WIXEN_MEASUREMENT_SHOW_LOG";

/// Start the binary against a profile, wait for the usable line, read memory
/// at the three moments, and stop it.
fn one_run(profile: &Path) -> Result<ARun, String> {
    let mut started = Started::against(profile)?;
    let (rows_in_the_list, usable_after_ms) = wait_for_usable(&mut started, profile)?;
    let (after_usable, at_sixty, at_one_twenty, complaints) =
        the_three_readings(&mut started, profile)?;
    Ok(ARun {
        rows_in_the_list,
        usable_after_ms,
        after_usable,
        at_sixty,
        at_one_twenty,
        complaints,
    })
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

/// The processor, its logical core count and the memory, as Windows reports
/// them.
fn the_machine() -> String {
    powershell(
        "$p = Get-CimInstance Win32_Processor | Select-Object -First 1; \
         $c = Get-CimInstance Win32_ComputerSystem; \
         '{0}, {1} logical processors, {2} GB' -f $p.Name.Trim(), $p.NumberOfLogicalProcessors, [math]::Round($c.TotalPhysicalMemory / 1GB)",
    )
    .map(|line| line.trim().to_string())
    .unwrap_or_else(|why| format!("machine not read: {why}"))
}

/// Say what one reading held, in megabytes.
fn describe(reading: &AReading) -> String {
    format!(
        "application working set {} MB, peak {} MB, private {} MB; tree of {} processes working set {} MB; together {} MB",
        Memory::megabytes(reading.application.working_set),
        Memory::megabytes(reading.application.peak_working_set),
        Memory::megabytes(reading.application.private_bytes),
        reading.members.len(),
        Memory::megabytes(reading.tree.working_set),
        Memory::megabytes(reading.application.working_set + reading.tree.working_set),
    )
}

/// Each process in the tree, by name and working set.
fn describe_the_tree(reading: &AReading) -> String {
    reading
        .members
        .iter()
        .map(|(name, memory)| format!("{name} {} MB", Memory::megabytes(memory.working_set)))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Print one run's readings and the rows the page takes.
fn print_the_run(profile_name: &str, command: &str, run: &ARun) {
    let (date, commit, version) = today_commit_and_version();
    let machine = the_machine();
    println!();
    println!("== {profile_name}: {version} at {commit} on {date}, {machine}");
    println!(
        "usable: {} rows in the list, {} ms after start",
        run.rows_in_the_list, run.usable_after_ms
    );
    println!("at usable + 5 s: {}", describe(&run.after_usable));
    println!("at 60 s: {}", describe(&run.at_sixty));
    println!("the tree at 60 s: {}", describe_the_tree(&run.at_sixty));
    println!("at 120 s: {}", describe(&run.at_one_twenty));
    if run.complaints.is_empty() {
        println!("the log holds no WARN or ERROR line");
    }
    for complaint in &run.complaints {
        println!("the log complained: {complaint}");
    }

    let with_the_messages = format!(
        "{} MB",
        Memory::megabytes(
            run.at_sixty.application.peak_working_set + run.at_sixty.tree.working_set
        )
    );
    let idle = format!(
        "{} MB",
        Memory::megabytes(
            run.at_one_twenty.application.working_set + run.at_one_twenty.tree.working_set
        )
    );
    let cold = format!("{} ms", run.usable_after_ms);
    for (what, value) in [
        (format!("Cold start to a usable list, {profile_name}"), cold),
        (
            format!("Memory with the list loaded, {profile_name}"),
            with_the_messages,
        ),
        (format!("Idle memory at 120 s, {profile_name}"), idle),
    ] {
        println!(
            "{}",
            the_row(&Row {
                what: &what,
                value: &value,
                command,
                date: &date,
                commit: &commit,
                conditions: &format!(
                    "{version} on {machine}; fill in the run, the thread setting and the machine state"
                ),
            })
        );
    }
}

/// The command the rows carry, backticked because the page's reading refuses
/// a row whose command cell holds no backticked token.
const THE_COMMAND: &str =
    "`cargo test --release --test the_numbers_the_targets_ask_for -- --ignored --nocapture`";

// ── The measurements, behind #[ignore] ──────────────────────────────────────

#[test]
#[ignore = "starts the release binary and waits two minutes; run by hand on a quiet machine"]
fn test_cold_start_and_memory_with_a_thousand_cached_messages() {
    refuse_a_debug_build().expect("a release build");
    let home = tempfile::tempdir().expect("a temporary folder");
    a_profile_with(home.path(), A_THOUSAND).expect("the profile writes");

    let run = one_run(home.path()).expect("a run that produced a number");
    print_the_run("1,000 cached messages", THE_COMMAND, &run);
}

#[test]
#[ignore = "starts the release binary and waits two minutes; run by hand on a quiet machine"]
fn test_the_empty_profile_floor() {
    refuse_a_debug_build().expect("a release build");
    let home = tempfile::tempdir().expect("a temporary folder");
    an_empty_profile(home.path()).expect("the profile writes");

    // No account and no mail, so nothing loads and there is no usable line
    // to wait for: the floor is the application on nothing, read at the same
    // moments from the start.
    let mut started = Started::against(home.path()).expect("the application starts");
    let (after_usable, at_sixty, at_one_twenty, complaints) =
        the_three_readings(&mut started, home.path()).expect("a run that produced a number");
    drop(started);

    let run = ARun {
        rows_in_the_list: 0,
        usable_after_ms: 0,
        after_usable,
        at_sixty,
        at_one_twenty,
        complaints,
    };
    print_the_run("empty profile", THE_COMMAND, &run);
}

// ── The tests that run on every commit ──────────────────────────────────────

/// The window fills the module it opens on, at startup.
///
/// Found by running the harness rather than by reading: the first release
/// run against the thousand-message profile waited 60 s for a usable line
/// that never came, and the log showed why. Every fill of a module comes
/// from a switch, and a switch to the module already on screen is refused,
/// so the mail module the window opens on was filled by nothing. The folder
/// tree came up empty and cached mail was not listed until a sync finished
/// or somebody switched modules away and back, on every profile.
///
/// This reads the startup section of `run`, from the frame being shown to
/// the event loop returning, and requires a fill of the active module in
/// it. What it cannot see: whether the fill reaches the list. The
/// measurement tests behind `#[ignore]` see that, because without it there
/// is no usable line to read.
#[test]
fn test_the_module_the_window_opens_on_is_filled_at_startup() {
    let app = std::fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
    let from = app
        .find("Main frame shown, entering event loop")
        .expect("the line logged when the frame is shown");
    let to = app
        .find("wxdragon::main blocks until the window is closed")
        .expect("the comment before the event loop's result is read");
    assert!(
        from < to,
        "the startup section runs from the frame being shown"
    );
    let startup = &app[from..to];

    assert!(
        startup.contains("load_module_data(")
            && startup.contains("active_module")
            && startup.contains("account_id"),
        "nothing fills the module the window opens on at startup, so the \
         folder tree comes up empty and cached mail is not listed until a \
         sync finishes or somebody switches modules away and back"
    );
}

#[test]
fn test_a_ten_row_profile_reads_back_ten_messages_and_ten_bodies() {
    let home = tempfile::tempdir().expect("a temporary folder");
    a_profile_with(home.path(), A_FEW).expect("the profile writes");

    let paths = AppPaths::under(home.path());
    let cache = MessageCache::new(paths.cache_dir(), None).expect("the cache opens");
    let folders = cache
        .get_folders_for_account(THE_MEASUREMENT_ACCOUNT)
        .expect("the folders read");
    let inbox = folders
        .iter()
        .find(|folder| folder.name == THE_FOLDER)
        .unwrap_or_else(|| panic!("no {THE_FOLDER}; the folders were {folders:?}"));

    let rows = cache
        .get_message_list_sorted(inbox.id, THE_MEASUREMENT_ACCOUNT, None, None)
        .expect("the folder lists");
    assert_eq!(rows.len(), A_FEW, "the rows in {THE_FOLDER}");

    let bodies = rows
        .iter()
        .filter(|row| {
            cache
                .get_message_body(row.id)
                .ok()
                .flatten()
                .and_then(|body| body.body_plain)
                .is_some()
        })
        .count();
    assert_eq!(bodies, A_FEW, "every row has a plain body");
}

#[test]
fn test_the_profile_has_the_shape_the_definition_gives() {
    let home = tempfile::tempdir().expect("a temporary folder");
    a_profile_with(home.path(), A_FEW).expect("the profile writes");

    let paths = AppPaths::under(home.path());
    let cache = MessageCache::new(paths.cache_dir(), None).expect("the cache opens");
    let accounts = cache.load_accounts().expect("the accounts read");
    assert_eq!(accounts.len(), 1, "one account");
    assert_eq!(
        accounts[0].imap_server, "127.0.0.1",
        "a server that refuses at once"
    );
    assert!(accounts[0].password.is_empty(), "no credentials");

    let rows = cache
        .unified_inbox(None, None)
        .expect("the unified inbox reads, which is what All Inboxes opens on");
    assert_eq!(rows.len(), A_FEW, "every row is in an inbox");
    let dates: Vec<&str> = rows.iter().map(|row| row.date.as_str()).collect();
    let mut descending = dates.clone();
    descending.sort_unstable_by(|a, b| b.cmp(a));
    assert_eq!(dates, descending, "dated in descending order");
    assert!(
        rows.iter().filter(|row| !row.read).count() > 0
            && rows.iter().filter(|row| row.read).count() > 0,
        "some read and some unread"
    );

    let a_body = cache
        .get_message_body(rows[0].id)
        .expect("the body reads")
        .and_then(|body| body.body_plain)
        .expect("a plain body");
    assert!(
        (1_800..=2_300).contains(&a_body.len()),
        "a body of about 2 KB, got {} bytes",
        a_body.len()
    );
}

#[test]
fn test_the_settings_open_on_the_list_and_skip_the_alpha_question() {
    let home = tempfile::tempdir().expect("a temporary folder");
    a_profile_with(home.path(), A_FEW).expect("the profile writes");

    let paths = AppPaths::under(home.path());
    let settings = std::fs::read_to_string(paths.config_dir().join("app_config.json"))
        .expect("the settings file the application reads");
    let settings: serde_json::Value = serde_json::from_str(&settings).expect("json");
    assert_eq!(
        settings["told_about_the_alpha"],
        serde_json::Value::Bool(true),
        "or every start stops at the first-run dialog"
    );
    assert_eq!(
        settings["start_in_all_inboxes"],
        serde_json::Value::Bool(true),
        "or the window opens with no folder chosen and no list ever loads"
    );
}

#[test]
fn test_an_empty_profile_has_the_settings_and_no_mail() {
    let home = tempfile::tempdir().expect("a temporary folder");
    an_empty_profile(home.path()).expect("the profile writes");

    let paths = AppPaths::under(home.path());
    assert!(
        paths.config_dir().join("app_config.json").is_file(),
        "the settings are there"
    );
    assert!(
        !paths.cache_dir().join("message_cache.db").exists(),
        "and no database is: the floor is the application on nothing"
    );
}

#[test]
fn test_the_usable_line_the_library_writes_parses_back() {
    let line = started::usable_line(500, Duration::from_millis(1234));
    let log = format!(
        "2026-09-14T21:00:00.000000Z  INFO wixen_mail::presentation::wx_app: Message list now holds 500 rows\n\
         2026-09-14T21:00:00.000100Z  INFO wixen_mail::presentation::wx_app: {line}\n"
    );

    assert_eq!(parse_usable_line(&log), Some((500, 1234)));
}

#[test]
fn test_a_log_without_the_usable_line_parses_to_nothing() {
    let log = "2026-09-14T21:00:00.000000Z  INFO wixen_mail: Starting Wixen Mail v0.124.0\n\
               2026-09-14T21:00:00.000000Z  INFO wixen_mail::presentation::wx_app: Message list now holds 500 rows\n";

    assert_eq!(parse_usable_line(log), None);
}

#[test]
fn test_the_complaints_in_a_log_are_its_warn_and_error_lines_without_their_timestamps() {
    let log = "2026-09-14T21:00:00.000000Z  INFO wixen_mail: Starting Wixen Mail v0.124.0\n\
               2026-09-14T21:00:01.000000Z  WARN wixen_mail::application::mail_sync: The server refused the connection\n\
               2026-09-14T21:00:02.000000Z ERROR wixen_mail::presentation::wx_app: Failed to read folder 1: gone\n";

    assert_eq!(
        complaints_in(log),
        vec![
            "WARN wixen_mail::application::mail_sync: The server refused the connection",
            "ERROR wixen_mail::presentation::wx_app: Failed to read folder 1: gone",
        ]
    );
    assert!(complaints_in("").is_empty(), "no log, no complaints");
}

#[test]
fn test_memory_is_read_from_the_shape_powershell_prints() {
    let output = "\r\n\r\nWorkingSet64        : 123456789\r\nPeakWorkingSet64    : 234567890\r\nPrivateMemorySize64 : 34567890\r\n\r\n\r\n";

    assert_eq!(
        parse_memory(output),
        Ok(Memory {
            working_set: 123_456_789,
            peak_working_set: 234_567_890,
            private_bytes: 34_567_890,
        })
    );
}

#[test]
fn test_memory_missing_a_field_is_refused_rather_than_read_as_nought() {
    let output = "WorkingSet64        : 123456789\r\nPrivateMemorySize64 : 34567890\r\n";

    let refused = parse_memory(output).expect_err("a field is missing");
    assert!(
        refused.contains("PeakWorkingSet64"),
        "the refusal names the field, got {refused}"
    );
}

#[test]
fn test_a_tree_is_summed_reading_by_reading() {
    let one = Memory {
        working_set: 10,
        peak_working_set: 20,
        private_bytes: 30,
    };
    let two = Memory {
        working_set: 1,
        peak_working_set: 2,
        private_bytes: 3,
    };

    assert_eq!(
        sum_tree(&[one, two]),
        Memory {
            working_set: 11,
            peak_working_set: 22,
            private_bytes: 33,
        }
    );
    assert_eq!(sum_tree(&[]), Memory::default(), "no tree, nothing");
}

#[test]
fn test_the_descendants_of_a_process_are_found_to_any_depth() {
    let processes = vec![
        (1, 0, "wininit.exe".to_string()),
        (100, 1, "wixen-mail.exe".to_string()),
        (200, 100, "msedgewebview2.exe".to_string()),
        (201, 200, "msedgewebview2.exe".to_string()),
        (202, 200, "msedgewebview2.exe".to_string()),
        (300, 1, "explorer.exe".to_string()),
    ];

    let mut found = descendants_of(100, &processes);
    found.sort_unstable();
    assert_eq!(
        found,
        vec![200, 201, 202],
        "the browser process and both under it"
    );
    assert!(
        descendants_of(300, &processes).is_empty(),
        "a process with nothing under it"
    );
}

#[test]
fn test_the_row_has_the_pages_columns_in_the_pages_order() {
    let page = std::fs::read_to_string("docs/development/measurements.md").expect("the page");
    let header = page
        .lines()
        .find(|line| line.starts_with("| What |"))
        .expect("the table's header");
    let columns: Vec<&str> = header.trim_matches('|').split('|').map(str::trim).collect();

    let row = the_row(&Row {
        what: "what",
        value: "value",
        command: "`a | b`",
        date: "2026-09-14",
        commit: "abcdef01",
        conditions: "conditions",
    });
    // Split the way the page's reader splits: an escaped pipe is not a
    // cell boundary.
    let cells: Vec<String> = row
        .replace("\\|", "\u{1}")
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().replace('\u{1}', "\\|"))
        .collect();

    assert_eq!(
        columns,
        ["What", "Value", "Command", "Date", "Commit", "Conditions"],
        "the page's header moved; move the row with it"
    );
    assert_eq!(
        cells,
        [
            "what",
            "value",
            "`a \\| b`",
            "2026-09-14",
            "abcdef01",
            "conditions"
        ],
        "the pipe inside the command is escaped so the table stays a table"
    );
}

#[test]
fn test_a_debug_build_is_refused_with_the_flag_that_fixes_it() {
    let answer = refuse_a_debug_build();

    if cfg!(debug_assertions) {
        let refusal = answer.expect_err("a debug build is refused");
        assert!(
            refusal.contains("--release"),
            "the refusal names the flag, got {refusal}"
        );
    } else {
        assert_eq!(answer, Ok(()), "a release build is measured");
    }
}

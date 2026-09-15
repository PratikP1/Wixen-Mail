//! The message list over 200,000 rows: sort, filter and scroll, each timed
//! with no window, and the rows the page takes.
//!
//! PERF-03 asks for the list to be exercised against 200,000 synthetic rows
//! with a recorded number for the sort, the filter and the scroll. The
//! generator has shipped on the Help menu since phase 3 and no number from it
//! existed anywhere in the tree; phase 3 measured once with a temporary test
//! and removed it. This one stays, behind `#[ignore]`, with a format-holding
//! half at 2,000 rows that runs on every commit so the rows it prints are
//! rows `docs/development/measurements.md` will accept.
//!
//! # What is timed, and what is not
//!
//! **The listing** is `MessageCache::get_messages_for_folder`, the read the
//! list takes when a folder opens and the one that precedes any scroll, over
//! a cache holding 200,000 rows in one folder of one account. Cold is the
//! first read on a connection opened after the rows were written; warm is the
//! reads after it. The file itself was warm in the operating system's cache
//! either way, because this process had just written it.
//!
//! **The filter** is `MessageCache::search_messages`, the query the search
//! box runs, at the limit the search box passes. The sample mailbox never
//! enters SQLite in the running program: the Help menu pushes it straight
//! into memory through `MessagesLoaded`. So this harness writes the same rows
//! into a `tempfile` cache through `upsert_messages`, the call the sync uses,
//! with the subjects and senders `sample_mailbox` generates, so a word in one
//! subject in five hits one row in five.
//!
//! **The sort** is `mail_sort::sort_messages` over `sample_mailbox(200_000)`
//! in memory, each of the seven orders on a fresh clone. In the running
//! program `apply_sort` clones the rows, sorts them off the interface thread
//! and sends them back through `MessagesLoaded`; the cost of the list control
//! taking that result is not timed here, because there is no window.
//!
//! **The scroll** is `virtual_rows::text_for` over one page of 40 rows and
//! every visible column of an inbox, and over all 200,000 rows for one
//! column. A scroll in the running program is that plus wxWidgets' own
//! painting, which is not timed here either, for the same reason.
//!
//! Every timing is taken three times and the median is the number; the three
//! takes are written into the row's conditions. The rows are synthetic and no
//! provider mailbox was used, and every row says so.
//!
//! # Running it
//!
//! ```text
//! cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture
//! ```
//!
//! On a machine doing nothing else: check `tasklist` for `cargo.exe` and
//! `rustc.exe` first. A debug build is refused, because a debug figure is a
//! figure about a binary nobody ships.

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use wixen_mail::common::what_ships::what_ships;
use wixen_mail::data::message_cache::MessageCache;
use wixen_mail::presentation::sample_mailbox::SAMPLE_MAILBOX_SIZE;

/// The account the one folder belongs to. Only the folder row names it; no
/// account is saved, because saving one reaches the credential store and
/// nothing here needs an account beyond the folder's `account_id`.
const THE_ACCOUNT: &str = "scale-measurement";
/// How many rows the format-holding half runs over on every commit.
const A_FEW: usize = 2_000;

// ── The cache ───────────────────────────────────────────────────────────────

/// Write `count` rows into one folder of one account in a cache at `into`,
/// in batches through `upsert_messages`, and answer the folder's row id.
fn a_cache_of(count: usize, into: &Path) -> Result<i64, String> {
    let _ = (count, into);
    Err(String::from("not written yet"))
}

// ── Timing ──────────────────────────────────────────────────────────────────

/// The middle take.
fn median(takes: &[Duration]) -> Duration {
    let mut sorted = takes.to_vec();
    sorted.sort();
    sorted[sorted.len() / 2]
}

fn milliseconds(d: Duration) -> String {
    format!("{:.2} ms", d.as_secs_f64() * 1000.0)
}

fn the_takes(takes: &[Duration]) -> String {
    takes
        .iter()
        .map(|take| milliseconds(*take))
        .collect::<Vec<_>>()
        .join(", ")
}

/// One timing as the harness reports it, before it is worded as a row.
struct Measured {
    what: String,
    takes: Vec<Duration>,
    /// What the rows returned, the counts, cold or warm: the rest of the
    /// conditions cell.
    detail: String,
}

/// Every timing over a cache of `count` rows at `into` and the same rows in
/// memory, in the order the page lists them.
fn measure(count: usize, into: &Path) -> Result<Vec<Measured>, String> {
    let _ = (count, into);
    Ok(Vec::new())
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

/// The command the rows carry, backticked because the page's reading refuses
/// a row whose command cell holds no backticked token.
const THE_COMMAND: &str =
    "`cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture`";

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
    Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "$p = Get-CimInstance Win32_Processor | Select-Object -First 1; \
             $c = Get-CimInstance Win32_ComputerSystem; \
             '{0}, {1} logical processors, {2} GB' -f $p.Name.Trim(), $p.NumberOfLogicalProcessors, [math]::Round($c.TotalPhysicalMemory / 1GB)",
        ])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .unwrap_or_else(|| "machine not read".to_string())
}

/// The rows the page takes, one per timing, each saying what was timed and
/// what was not.
fn the_rows(count: usize, measured: &[Measured], build: &str, machine: &str) -> Vec<String> {
    let (date, commit, version) = today_commit_and_version();
    measured
        .iter()
        .map(|m| {
            let conditions = format!(
                "{version} at {commit}, {build} build, {machine}, `WIXEN_TEST_THREADS` unset and one test running. \
                 {} synthetic rows from `sample_mailbox` and no provider mailbox; the cache rows in a `tempfile` directory. \
                 The three takes: {}. {}",
                count,
                the_takes(&m.takes),
                m.detail
            );
            the_row(&Row {
                what: &m.what,
                value: &milliseconds(median(&m.takes)),
                command: THE_COMMAND,
                date: &date,
                commit: &commit,
                conditions: &conditions,
            })
        })
        .collect()
}

/// Refuse to measure a debug build.
fn refuse_a_debug_build() -> Result<(), String> {
    if cfg!(debug_assertions) {
        return Err(String::from(
            "this is a debug build and a debug figure is a figure about a binary nobody ships; \
             run with --release: cargo test --release --test the_list_at_two_hundred_thousand_rows -- --ignored --nocapture",
        ));
    }
    Ok(())
}

// ── The measurement, behind #[ignore] ───────────────────────────────────────

#[test]
#[ignore = "writes 200,000 rows and times them; run by hand on a quiet machine with --release"]
fn test_the_list_at_two_hundred_thousand_rows() {
    refuse_a_debug_build().expect("a release build");
    let into = tempfile::tempdir().expect("a folder to leave nothing in");
    let measured = measure(SAMPLE_MAILBOX_SIZE, into.path()).expect("the measurement");
    for row in the_rows(SAMPLE_MAILBOX_SIZE, &measured, "release", &the_machine()) {
        println!("{row}");
    }
}

// ── What runs on every commit ───────────────────────────────────────────────

/// What kind of thing each row times; a row naming none of these is a row
/// the page cannot be read from.
const WHAT_A_ROW_TIMES: [&str; 5] = ["Listing", "Filter", "Sort", "Page paint", "Full pass"];

#[test]
fn test_two_thousand_rows_written_read_back_two_thousand() {
    let into = tempfile::tempdir().expect("a folder to leave nothing in");

    let folder_id = a_cache_of(A_FEW, into.path()).expect("the rows written");
    let cache = MessageCache::new(into.path().to_path_buf(), None).expect("the cache reopened");
    let listed = cache
        .get_messages_for_folder(folder_id, THE_ACCOUNT)
        .expect("the listing");

    assert_eq!(listed.len(), A_FEW);
}

#[test]
fn test_every_row_the_measurement_prints_has_the_pages_shape_and_names_what_it_timed() {
    let into = tempfile::tempdir().expect("a folder to leave nothing in");
    let measured = measure(A_FEW, into.path()).expect("the measurement at a few rows");
    let rows = the_rows(A_FEW, &measured, "debug", "a machine");

    assert!(
        rows.len() >= 11,
        "{} rows printed and the page wants at least eleven: the listing cold and warm, the \
         filters, the seven sort orders, the page paint and the full pass",
        rows.len()
    );
    for row in &rows {
        let cells: Vec<&str> = row.split(" | ").collect();
        assert_eq!(cells.len(), 6, "not the page's six columns: {row}");
        assert!(
            WHAT_A_ROW_TIMES.iter().any(|kind| cells[0].contains(kind)),
            "the row does not say what it timed: {row}"
        );
        assert!(
            cells[5].contains("no provider mailbox"),
            "the row does not say the rows are synthetic: {row}"
        );
    }
    let what: Vec<&str> = measured.iter().map(|m| m.what.as_str()).collect();
    for order in [
        "Newest",
        "Oldest",
        "Sender (A-Z)",
        "Sender (Z-A)",
        "Subject (A-Z)",
        "Subject (Z-A)",
        "Unread",
    ] {
        assert!(
            what.iter().any(|w| w.contains(order)),
            "no sort row for {order}: {what:?}"
        );
    }
    for kind in WHAT_A_ROW_TIMES {
        assert!(
            what.iter().any(|w| w.contains(kind)),
            "no {kind} row: {what:?}"
        );
    }
}

#[test]
fn test_the_median_is_the_middle_take_and_the_row_carries_all_three() {
    let takes = [
        Duration::from_millis(30),
        Duration::from_millis(10),
        Duration::from_millis(20),
    ];

    assert_eq!(median(&takes), Duration::from_millis(20));
    assert_eq!(the_takes(&takes), "30.00 ms, 10.00 ms, 20.00 ms");
}

#[test]
fn test_a_debug_build_is_refused_with_the_flag_that_fixes_it() {
    let answer = refuse_a_debug_build();
    if cfg!(debug_assertions) {
        let refusal = answer.expect_err("a debug build is refused");
        assert!(refusal.contains("--release"), "{refusal}");
    } else {
        assert_eq!(answer, Ok(()), "a release build is measured");
    }
}

/// The Help menu still loads the sample through the moved generator.
///
/// A reading rather than a run, because the handler needs a window. The
/// generator moved out of `wx_app.rs` in 08-04 and this holds the handler to
/// still calling it with the constant and sending the rows to the list.
#[test]
fn test_the_help_menu_loads_the_sample_through_the_generator() {
    let window = what_ships(
        &std::fs::read_to_string("src/presentation/wx_app.rs").expect("the main window"),
    );
    let arm = "id == ID_LOAD_SCALE_SAMPLE =>";
    let at = window
        .find(arm)
        .expect("the Help menu's handler for the sample");
    let handler = &window[at..];
    let handler = &handler[..handler[arm.len()..]
        .find("_ if id ==")
        .map_or(handler.len(), |next| next + arm.len())];

    assert!(
        handler.contains("sample_mailbox(SAMPLE_MAILBOX_SIZE)"),
        "the handler no longer builds the sample through the generator: {handler}"
    );
    assert!(
        handler.contains("UIUpdate::MessagesLoaded("),
        "the handler no longer sends the sample to the list: {handler}"
    );
}

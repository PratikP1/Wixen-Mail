//! The proof reader reads a profile the same way every time and prints none
//! of the mail in it (14-06).
//!
//! Phase 14's sittings are each read before and after from the tester's own
//! profile (answer 9 of 2026-10-05): with Python, read-only, printing counts,
//! the build and masked lines, never a body, and no subject unless it begins
//! "Wixen proof". `scripts/read-a-proof.py` is that rule written once, and
//! this target holds it to it over a fixture profile built here: the real
//! schema through `MessageCache::new`, a private subject and address it must
//! not print, two daily logs either side of UTC midnight, and a crash file
//! written through the program's own crash writer.
//!
//! Missing Python fails rather than skips, as `tests/house_style.rs` does for
//! its two scripts: a check that quietly does nothing is the defect this tree
//! keeps finding.
//!
//! What this cannot see. The fixture holds the shapes the program writes
//! today, read from `src`; a subject inside a spoken shape nobody has met yet
//! is printed as its length, which is the reader's rule and is tested, but a
//! log line of a new shape carrying a subject outside the spoken section is
//! not printed at all unless it matches a template, and the companion below
//! only holds the templates the reader already has.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use wixen_mail::application::feedback_report::redact;
use wixen_mail::common::logging;
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::data::message_cache::moves_waiting::{AWaitingMove, WhatAWaitingMoveDoes};
use wixen_mail::data::message_cache::{CachedFolder, CalendarContainer, MessageCache};
use wixen_mail::presentation::page_window::FLAG;

const THE_READER: &str = "scripts/read-a-proof.py";

const ACCOUNT: &str = "account-1";

/// What the reader must never print: the private message's subject and its
/// sender, the account's own address and its stored secret.
const PRIVATE_SUBJECT: &str = "Lunch with Ada";
const PRIVATE_ADDRESS: &str = "ada@example.com";
const THE_ACCOUNTS_ADDRESS: &str = "owner@example.org";
const THE_STORED_SECRET: &str = "fixture-secret-never-read";

/// The proof's subject, which the reader may print.
const THE_PROOF: &str = "Wixen proof copy 1";

/// The build the window's start line names, and the one outside the window.
const THE_BUILD: &str = "1.0.0-alpha.1+1071.g01ef4589";
const THE_BUILD_BEFORE: &str = "1.0.0-alpha.1+1070.g0000aaaa";

/// The window the log cases read, either side of UTC midnight.
const SINCE: &str = "2026-10-05T23:50Z";
const UNTIL: &str = "2026-10-06T00:10Z";

/// The day before midnight, the 2026-09-18 shape and the one after 14-05.
const THE_DAY_BEFORE: &str = "\
2026-10-05T23:40:00.000001Z  INFO wixen_mail: Starting Wixen Mail v1.0.0-alpha.1+1070.g0000aaaa
2026-10-05T23:40:30.000001Z  INFO wixen_mail::application::moves_waiting: A waiting delete of message 99 in INBOX was replayed: Deleted
2026-10-05T23:50:01.000001Z  INFO wixen_mail: Starting Wixen Mail v1.0.0-alpha.1+1071.g01ef4589
2026-10-05T23:50:01.000002Z  INFO wixen_mail::common::logging: Logging initialized at level: Info
2026-10-05T23:51:00.000000Z  INFO wixen_mail::application::moves_waiting: A waiting copy was replayed: copied to Wixen proof
2026-10-05T23:52:00.000000Z  INFO wixen_mail::presentation::accessibility: Speaking: Copied to Wixen proof: Lunch with Ada topic=Some(\"thread-1\")
2026-10-05T23:53:00.000000Z  INFO wixen_mail::presentation::accessibility: Speaking: Ada wrote back about Lunch with Ada topic=None
2026-10-05T23:54:00.000000Z  WARN wixen_mail::application::mail_sync: Could not fetch the text of message 12: Network error: closed by ada@example.com
";

const THE_DAY_AFTER: &str = "\
2026-10-06T00:01:00.000000Z  INFO wixen_mail::application::moves_waiting: A waiting copy of message 2 in INBOX was replayed: Copied to Wixen proof; the server holds it in Wixen proof as 4
2026-10-06T00:02:00.000000Z  INFO wixen_mail::presentation::accessibility: Speaking: Copied to Wixen proof: Wixen proof copy 1 topic=None
2026-10-06T00:03:00.000000Z  INFO wixen_mail::service::protocols::imap: Signed in to imap.example.com with a password
2026-10-06T00:04:00.000000Z  INFO wixen_mail::service::google_api: Asked Google: GET www.googleapis.com/calendar/v3/calendars/primary/events, answered 200
2026-10-06T00:05:00.000000Z  INFO wixen_mail::presentation::wx_app: calendar sync finished, account at google, answered, created 1, updated 0, deleted 0, sent 0, errors 0
2026-10-06T00:06:00.000000Z  INFO wixen_mail::application::moves_waiting: A move of message 3 to another account waits: ada@example.com is not signed in
2026-10-06T00:20:00.000000Z  INFO wixen_mail::application::moves_waiting: A waiting move of message 77 in INBOX was replayed: Moved to Archive
";

/// The three refusals the page window's test wrote into the tester's crash
/// file on every library run until 14-06.
const THE_TEST_REFUSALS: [&str; 3] = ["mailto:somebody@example.com", "not-a-page", ""];

/// A panic in the 2026-09-18 shape, which named the crate's version only.
const THE_OLD_SHAPE: &str = "PANIC at src\\presentation\\wx_app.rs:30:9
  called `Option::unwrap()` on a `None` value
  Wixen Mail v1.0.0-alpha.1
  Time: SystemTime { intervals: 134031849450000000 }";

/// A profile in a temporary folder, and the rows the cases ask about.
struct AProfile {
    home: tempfile::TempDir,
    private_row: i64,
}

impl AProfile {
    fn root(&self) -> &Path {
        self.home.path()
    }

    fn database(&self) -> PathBuf {
        self.root().join("cache").join("message_cache.db")
    }
}

fn a_folder(name: &str, folder_type: &str) -> CachedFolder {
    CachedFolder {
        id: 0,
        account_id: ACCOUNT.to_string(),
        name: name.to_string(),
        path: name.to_string(),
        folder_type: folder_type.to_string(),
        unread_count: 0,
        total_count: 0,
    }
}

fn a_calendar(id: &str, provider: &str) -> CalendarContainer {
    CalendarContainer {
        id: id.to_string(),
        account_id: ACCOUNT.to_string(),
        name: id.to_string(),
        color: String::new(),
        source_provider: Some(provider.to_string()),
        caldav_url: None,
        subscription_url: None,
        is_default: false,
        is_visible: true,
        is_read_only: false,
        display_order: 0,
        etag: None,
        ctag: None,
        sync_token: None,
        refresh_interval_minutes: None,
        created_at: String::new(),
        updated_at: String::new(),
    }
}

/// One message row by plain SQL, since the store's writer takes some thirty
/// fields of which these five are the ones the reader reads.
fn a_message(
    sql: &rusqlite::Connection,
    folder_id: i64,
    uid: u32,
    subject: &str,
    filed_here: bool,
) -> i64 {
    sql.execute(
        "INSERT INTO messages (uid, folder_id, message_id, subject, from_addr, to_addr, date, \
         body_plain, filed_here) VALUES (?1, ?2, ?3, ?4, ?5, ?6, '2026-10-05T23:00:00Z', ?7, ?8)",
        rusqlite::params![
            i64::from(uid),
            folder_id,
            format!("<{uid}.{folder_id}@example.com>"),
            subject,
            PRIVATE_ADDRESS,
            THE_ACCOUNTS_ADDRESS,
            format!("The body of {subject}"),
            filed_here,
        ],
    )
    .expect("a message row");
    sql.last_insert_rowid()
}

fn a_profile() -> AProfile {
    let home = tempfile::tempdir().expect("a temporary folder");
    let root = home.path();

    let cache = MessageCache::new(root.join("cache"), None).expect("the real schema");
    let sql = rusqlite::Connection::open(root.join("cache").join("message_cache.db"))
        .expect("the same database");
    sql.execute(
        "INSERT INTO accounts (id, name, email, imap_server, imap_port, imap_use_tls, \
         smtp_server, smtp_port, smtp_use_tls, username, password, enabled, \
         check_interval_minutes, provider, color, created_at, updated_at) VALUES (?1, 'Mine', \
         ?2, 'imap.example.com', '993', 1, 'smtp.example.com', '465', 1, ?2, ?3, 1, 5, \
         'Gmail', '', '', '')",
        rusqlite::params![ACCOUNT, THE_ACCOUNTS_ADDRESS, THE_STORED_SECRET],
    )
    .expect("an account");
    let inbox = cache
        .save_folder(&a_folder("INBOX", "Inbox"))
        .expect("an Inbox");
    let proof = cache
        .save_folder(&a_folder("Wixen proof", "Custom"))
        .expect("the proof folder");

    a_message(&sql, inbox, 5, THE_PROOF, false);
    let copy = a_message(&sql, proof, u32::MAX, THE_PROOF, true);
    let private_row = a_message(&sql, inbox, 7, PRIVATE_SUBJECT, false);

    cache
        .keep_a_move_waiting(&AWaitingMove {
            message_row_id: copy,
            account_id: ACCOUNT.to_string(),
            from_folder_path: "INBOX".to_string(),
            uid: 5,
            what: WhatAWaitingMoveDoes::Copy {
                into_folder_path: "Wixen proof".to_string(),
            },
            asked_at: "2026-10-05T23:55:00Z".to_string(),
        })
        .expect("a waiting copy");
    cache
        .save_calendar(&a_calendar("on-this-computer", "local"))
        .expect("a calendar here");
    cache
        .save_calendar(&a_calendar("google:account-1:primary", "gmail"))
        .expect("a Google calendar");
    drop(sql);
    drop(cache);

    let config = root.join("config");
    std::fs::create_dir_all(&config).expect("a settings folder");
    std::fs::write(
        config.join("app_config.json"),
        "{\"log_level\": \"info\", \"allowed_changes\": {\"mail\": true}}",
    )
    .expect("the settings");

    let logs = root.join("logs");
    std::fs::create_dir_all(&logs).expect("a log folder");
    std::fs::write(logs.join("wixen-mail.2026-10-05.log"), THE_DAY_BEFORE).expect("a day");
    std::fs::write(logs.join("wixen-mail.2026-10-06.log"), THE_DAY_AFTER).expect("the next");

    for address in THE_TEST_REFUSALS {
        logging::append_to_the_crash_file(
            &logs,
            &format!("{FLAG} was given {address:?}, which is not a page. Nothing was opened."),
        )
        .expect("a refusal");
    }
    logging::append_to_the_crash_file(
        &logs,
        &logging::crash_entry(
            THE_BUILD,
            "src\\presentation\\wx_app.rs:12:5",
            "byte index 3 is not a char boundary; it is inside 'e' (bytes 2..4) of `Lunch with Ada`",
        ),
    )
    .expect("a crash naming its build");
    logging::append_to_the_crash_file(&logs, THE_OLD_SHAPE).expect("a crash in the old shape");

    AProfile { home, private_row }
}

fn python() -> Command {
    let mut python = Command::new("python");
    python.env("PYTHONIOENCODING", "utf-8");
    python
}

fn could_not_start<T>(e: std::io::Error) -> T {
    panic!(
        "python could not be run, so the proof reader went untested: {e}.\nInstall python and \
         put it on the path; every sitting's reading needs it too."
    )
}

/// What the reader prints, run with `args`, which must succeed.
fn the_reader(args: &[&str]) -> String {
    let ran = python()
        .arg(THE_READER)
        .args(args)
        .output()
        .unwrap_or_else(could_not_start);
    assert!(
        ran.status.success(),
        "the reader failed: {}{}",
        String::from_utf8_lossy(&ran.stdout),
        String::from_utf8_lossy(&ran.stderr)
    );
    String::from_utf8_lossy(&ran.stdout).replace("\r\n", "\n")
}

/// The reader over the fixture, for the log window either side of midnight.
fn read_the_window(profile: &AProfile, more: &[&str]) -> String {
    let root = profile.root().to_string_lossy().into_owned();
    let mut args = vec!["--profile", &root, "--since", SINCE, "--until", UNTIL];
    args.extend_from_slice(more);
    the_reader(&args)
}

/// Everything the reader prints for any question this target asks, so the
/// privacy cases read all of it.
fn everything_it_prints(profile: &AProfile) -> String {
    let row = profile.private_row.to_string();
    let an_hour_ago = (chrono::Utc::now() - chrono::Duration::hours(1))
        .format("%Y-%m-%dT%H:%MZ")
        .to_string();
    let root = profile.root().to_string_lossy().into_owned();
    [
        read_the_window(
            profile,
            &[
                "--folder",
                "Wixen proof",
                "--folder",
                "INBOX",
                "--message",
                &row,
            ],
        ),
        the_reader(&["--profile", &root, "--since", &an_hour_ago]),
    ]
    .concat()
}

#[test]
fn test_it_prints_the_build_the_level_and_the_settings() {
    let profile = a_profile();
    let read = read_the_window(&profile, &[]);

    assert!(read.contains(&format!("build {THE_BUILD}")), "{read}");
    assert!(read.contains("level Info"), "{read}");
    assert!(read.contains("log_level info"), "{read}");
}

#[test]
fn test_it_prints_the_waiting_queue_and_the_proof_messages() {
    let profile = a_profile();
    let read = read_the_window(&profile, &[]);

    assert!(
        read.contains("copy, from INBOX number 5 into Wixen proof, asked 2026-10-05T23:55:00Z"),
        "{read}"
    );
    assert!(read.contains("moves in flight: 0"), "{read}");
    assert!(read.contains("messages waiting to send: 0"), "{read}");
    assert!(
        read.contains("in Wixen proof, number 4294967295, reserved here, filed here 1, deleted 0: Wixen proof copy 1"),
        "{read}"
    );
    assert!(
        read.contains(
            "in INBOX, number 5, not reserved, filed here 0, deleted 0: Wixen proof copy 1"
        ),
        "{read}"
    );
}

#[test]
fn test_it_counts_calendars_by_provider_events_contacts_tasks_and_sync_state() {
    let profile = a_profile();
    let read = read_the_window(&profile, &[]);

    for counted in [
        "calendars from gmail: 1",
        "calendars from local: 1",
        "events: 0",
        "contacts: 0",
        "tasks: 0",
        "sync state rows: 0",
    ] {
        assert!(read.contains(counted), "{counted} is not in {read}");
    }
}

#[test]
fn test_it_prints_the_lines_by_template_from_both_days_in_both_shapes() {
    let profile = a_profile();
    let read = read_the_window(&profile, &[]);

    for line in [
        // A replay before 14-05, in the day before midnight.
        "A waiting copy was replayed: copied to Wixen proof",
        // And after it, in the day after.
        "A waiting copy of message 2 in INBOX was replayed: Copied to Wixen proof; the server \
         holds it in Wixen proof as 4",
        "Signed in to imap.example.com with a password",
        "Asked Google: GET www.googleapis.com/calendar/v3/calendars/primary/events, answered 200",
        "calendar sync finished, account at google, answered, created 1",
        "A move of message 3 to another account waits: ad***@example.com is not signed in",
        "Could not fetch the text of message N",
    ] {
        assert!(read.contains(line), "{line:?} is not in {read}");
    }
}

#[test]
fn test_it_prints_nothing_from_outside_the_window() {
    let profile = a_profile();
    let read = read_the_window(&profile, &[]);

    for outside in [THE_BUILD_BEFORE, "message 99", "message 77"] {
        assert!(!read.contains(outside), "{outside} was printed: {read}");
    }
}

#[test]
fn test_it_answers_for_a_folder_and_for_a_message() {
    let profile = a_profile();
    let row = profile.private_row.to_string();
    let read = read_the_window(&profile, &["--folder", "Wixen proof", "--message", &row]);

    assert!(
        read.contains("Wixen proof: 1 messages, 1 filed here, 1 holding a reserved number"),
        "{read}"
    );
    assert!(
        read.contains(&format!(
            "row {row}: in INBOX, number 7, not reserved, filed here 0, deleted 0, subject withheld"
        )),
        "{read}"
    );
}

#[test]
fn test_it_prints_no_private_subject_address_body_or_secret() {
    let profile = a_profile();
    let read = everything_it_prints(&profile);

    for private in [
        PRIVATE_SUBJECT,
        "Lunch",
        PRIVATE_ADDRESS,
        THE_ACCOUNTS_ADDRESS,
        THE_STORED_SECRET,
        "The body of",
        "Ada wrote back",
    ] {
        assert!(!read.contains(private), "{private} was printed: {read}");
    }
    // The spoken line of a known shape is printed with its subject withheld,
    // and a proof's subject is printed as it was said.
    assert!(
        read.contains("Speaking: Copied to Wixen proof: [subject withheld]"),
        "{read}"
    );
    assert!(
        read.contains("Speaking: Copied to Wixen proof: Wixen proof copy 1"),
        "{read}"
    );
    assert!(
        read.contains("not what NVDA said"),
        "the spoken section does not say what it is: {read}"
    );
}

#[test]
fn test_it_masks_the_addresses_the_feedback_report_masks() {
    let lines: Vec<String> = [
        "Sending email from ada@example.com to [bob.smith@example.org]",
        "Moved to Archive in someone@work.example: Wixen proof copy 1",
        "a@b.c and x@y and ab@cd.ef, and not @here or here@",
        "Subject: Lunch with ada@example.com",
        "subject=Lunch, from ada@example.com",
    ]
    .map(str::to_string)
    .to_vec();

    let mut reader = python()
        .args([THE_READER, "--mask-stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(could_not_start);
    reader
        .stdin
        .take()
        .expect("the reader's input")
        .write_all(lines.join("\n").as_bytes())
        .expect("the lines written to the reader");
    let ran = reader.wait_with_output().expect("the reader finishes");
    assert!(ran.status.success(), "the reader failed to mask");

    let masked: Vec<String> = String::from_utf8_lossy(&ran.stdout)
        .lines()
        .map(str::to_string)
        .collect();
    assert_eq!(masked, redact(&lines));
}

#[test]
fn test_it_prints_each_real_crash_and_drops_the_tests_refusals() {
    let profile = a_profile();
    let an_hour_ago = (chrono::Utc::now() - chrono::Duration::hours(1))
        .format("%Y-%m-%dT%H:%MZ")
        .to_string();
    let root = profile.root().to_string_lossy().into_owned();
    let read = the_reader(&["--profile", &root, "--since", &an_hour_ago]);

    assert!(read.contains(&format!("Wixen Mail v{THE_BUILD}")), "{read}");
    assert!(read.contains("src\\presentation\\wx_app.rs:12:5"), "{read}");
    assert!(read.contains("Wixen Mail v1.0.0-alpha.1\n"), "{read}");
    assert!(read.contains("src\\presentation\\wx_app.rs:30:9"), "{read}");
    assert!(
        read.contains("dropped 3 lines the library's tests wrote"),
        "{read}"
    );
    assert!(
        !read.contains("not-a-page"),
        "a test's refusal was printed: {read}"
    );
}

#[test]
fn test_it_leaves_the_database_as_it_found_it() {
    let profile = a_profile();
    let before = std::fs::read(profile.database()).expect("the database");

    read_the_window(&profile, &["--folder", "Wixen proof", "--message", "1"]);

    let after = std::fs::read(profile.database()).expect("the database");
    assert!(before == after, "the reader changed the database file");
}

#[test]
fn test_it_reads_no_secret_store_and_opens_the_database_read_only() {
    let reader = std::fs::read_to_string(THE_READER).expect("the reader's source");

    assert!(
        reader.contains("mode=ro"),
        "the database is not opened read-only"
    );
    for never in ["immutable", "password", "oauth.toml", "keyring"] {
        assert!(
            !reader.contains(never),
            "the reader names {never}, which it must never read"
        );
    }
}

#[test]
fn test_every_text_the_reader_matches_is_still_written_in_src() {
    // A reworded line drops out of the reader's output in silence, so each
    // fixed text the reader matches is held to the shipped source.
    let mut source = String::new();
    gather_the_shipped_source(Path::new("src"), &mut source);

    let openings = the_reader(&["--openings"]);
    let openings: Vec<&str> = openings.lines().filter(|line| !line.is_empty()).collect();
    assert!(
        openings.len() >= 10,
        "the reader names only {} fixed texts, so this reads almost nothing",
        openings.len()
    );
    let gone: Vec<&str> = openings
        .into_iter()
        .filter(|opening| !source.contains(opening))
        .collect();
    assert!(
        gone.is_empty(),
        "the reader matches texts no longer written in src: {gone:?}"
    );
}

fn gather_the_shipped_source(dir: &Path, into: &mut String) {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|entry| entry.expect("a folder entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            gather_the_shipped_source(&path, into);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let text = std::fs::read_to_string(&path).expect("a source file");
            into.push_str(&what_ships(&text.replace("\r\n", "\n")));
        }
    }
}

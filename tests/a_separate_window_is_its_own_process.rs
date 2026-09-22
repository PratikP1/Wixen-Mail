//! The separate Wixen Mail window a link opens in, as a process of its own.
//!
//! #80's third place, 12-02. The Reading tab has offered three since
//! 11-11.1 and the third opened the default browser with a line saying so,
//! because every `WebView` in one process shares one WebView2 profile under
//! wxdragon 0.9.17 and nothing in either crate can clear it or move it. A
//! window in the same process would therefore have sat in the message
//! preview's cookie jar, which is what the issue's third item is about. A
//! process of its own, under an application name of its own, is the
//! isolation this toolkit can reach.
//!
//! The built executable is run here with addresses it must refuse: that is
//! the whole of what a test can drive, because the case that works is an
//! event loop and a window somebody has to close. Each refused run is
//! measured on its exit code, on what it said, and on how long it took.
//!
//! A start that did open a window is killed and reported, rather than
//! waited for: the first draft waited for the process to finish, which turns
//! the one failure this target exists to catch into a hang. A guard record
//! breaks the scheme check on purpose, so that failure is a thing this file
//! is really driven into.
//!
//! The rest is read from the source, and each reading says what it cannot
//! see. That a page really appears in that window, with its title spoken and
//! Escape closing it, is the tester's ear and is on the ledger.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use wixen_mail::common::what_ships::what_ships;

/// How long a start that must refuse is given before its timing is called a
/// failure.
///
/// Generous on purpose: the refusal happens before anything is opened, so a
/// run anywhere near this got further than it should have.
const LONG_ENOUGH_TO_REFUSE: Duration = Duration::from_secs(5);

/// What one run of the built executable answered.
struct WhatItAnswered {
    /// Whether it stopped by itself, rather than being killed for taking
    /// too long. A start that opened a window stops by nothing.
    stopped_by_itself: bool,
    code: Option<i32>,
    complaint: String,
    took: Duration,
}

/// Run the built executable with these arguments, against a throwaway data
/// folder, and kill it if it is still there after [`LONG_ENOUGH_TO_REFUSE`].
///
/// `WIXEN_MAIL_DATA` is set because a refused start writes its reason to the
/// crash file, and that file belongs to whoever is running this rather than
/// to the person whose machine it is.
fn start_and_wait(arguments: &[&str]) -> WhatItAnswered {
    let folder = tempfile::TempDir::new().expect("a throwaway data folder");
    let began = Instant::now();
    let mut child = Command::new(env!("CARGO_BIN_EXE_wixen-mail"))
        .args(arguments)
        .env("WIXEN_MAIL_DATA", folder.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the built executable should run");

    let mut stopped_by_itself = false;
    while began.elapsed() < LONG_ENOUGH_TO_REFUSE {
        if child
            .try_wait()
            .expect("the child should be askable")
            .is_some()
        {
            stopped_by_itself = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let took = began.elapsed();
    if !stopped_by_itself {
        let _ = child.kill();
    }
    let answered = child
        .wait_with_output()
        .expect("the child should be waitable");

    WhatItAnswered {
        stopped_by_itself,
        code: answered.status.code(),
        complaint: String::from_utf8_lossy(&answered.stderr).to_string(),
        took,
    }
}

/// What a release build of one of this project's files compiles, so a
/// reading cannot be satisfied by a test's own mention of what it looks for.
fn what_ships_in(path: &str) -> String {
    what_ships(
        &std::fs::read_to_string(Path::new(path))
            .unwrap_or_else(|e| panic!("{path} should be readable: {e}")),
    )
}

#[test]
fn test_an_address_that_is_not_a_page_opens_no_window() {
    // The boundary a process has that a function does not: `--show-page`
    // can be typed, or passed by something that is not this program, and
    // the address it carries has not been through this program's sanitiser
    // on the way in (T-12-04). Each of these is refused before anything is
    // opened, so the run finishes rather than waiting for somebody to close
    // a window.
    //
    // `not-a-page` is first on purpose, and the order matters. A break that
    // lets everything through loads whatever it was given, and this list is
    // otherwise five addresses Windows hands to five other programs. The
    // first assertion stops the loop, so the one that opens nothing is the
    // one a broken tree tries.
    for refused in [
        "not-a-page",
        "mailto:somebody@example.com",
        "javascript:alert(1)",
        "file:///C:/Windows/win.ini",
        "tel:+15550100",
    ] {
        let answered = start_and_wait(&["--show-page", refused]);

        assert!(
            answered.stopped_by_itself,
            "{refused} was still running after {:?}, so it opened something",
            answered.took
        );
        assert_eq!(answered.code, Some(2), "{refused} did not stop the start");
    }
}

#[test]
fn test_the_page_flag_with_nothing_after_it_is_refused_and_says_what_it_wanted() {
    // The same rule every other flag with a value follows, measured on the
    // executable rather than on the parser, because this is the one flag a
    // person meets without having typed it, and the complaint is the only
    // thing it can read.
    let answered = start_and_wait(&["--show-page"]);

    assert!(answered.stopped_by_itself);
    assert_eq!(answered.code, Some(2));
    assert!(
        answered.complaint.contains("needs an address"),
        "it complained {:?}",
        answered.complaint
    );
}

#[test]
fn test_a_page_process_is_answered_before_this_start_claims_or_prepares_anything() {
    // T-12-05. A page process opens no database, prepares no data folder,
    // opens no log file and holds no single-copy marker, and the only thing
    // that makes that true is where it is answered. Read as an order of
    // names in `main`, because nothing here can start a process and then ask
    // it what it opened.
    //
    // What this cannot see: whether those calls do what their names say.
    // That they are the claim and the folder is held by their own tests.
    let ships = what_ships_in("src/main.rs");
    let at = |name: &str| {
        ships
            .find(name)
            .unwrap_or_else(|| panic!("src/main.rs does not name {name}"))
    };

    let page = at("page_window::show");
    for afterwards in [
        "running::claim",
        "prepare_data_folder(",
        "init_logging(",
        "how_to_start(",
    ] {
        assert!(
            page < at(afterwards),
            "a page process is answered after {afterwards}, so it does what a page \
             process must not"
        );
    }
}

#[test]
fn test_the_page_process_is_the_only_one_that_renames_itself() {
    // T-12-06. The application name is what decides where WebView2 puts a
    // profile, so the two processes are only apart for as long as exactly
    // one of them changes it. The main process keeps the executable's name,
    // which is where its profile has always been; the page process takes a
    // name a level down. A second caller anywhere would move a profile
    // somebody's cookies are already in, and the order of the calls inside
    // the page process is held by that module's own reading.
    //
    // What this cannot see: whether the name it sets is the one the paths
    // module builds the folder from. `page_window` reads that from
    // `paths::page_profile_app_name`, and the paths module holds the two to
    // each other.
    let mut renames = Vec::new();
    for file in every_rust_file_under("src") {
        let ships = what_ships_in(&file);
        if ships.contains("set_app_name") {
            renames.push(file);
        }
    }

    assert_eq!(
        renames,
        vec!["src/presentation/page_window.rs".to_string()],
        "the application name is set somewhere other than the page process"
    );
}

/// The text of one function, as a release build sees it: from its signature
/// to the first line that closes at the left margin.
fn the_body_of(ships: &str, signature: &str) -> String {
    let from = ships
        .find(signature)
        .unwrap_or_else(|| panic!("the tree should define {signature}"));
    let rest = &ships[from..];
    let to = rest.find("\n}").map(|at| at + 2).unwrap_or(rest.len());
    rest[..to].to_string()
}

/// Every `.rs` file under a directory, in a stable order.
fn every_rust_file_under(directory: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut to_walk = vec![Path::new(directory).to_path_buf()];
    while let Some(here) = to_walk.pop() {
        let entries = std::fs::read_dir(&here)
            .unwrap_or_else(|e| panic!("{} should be readable: {e}", here.display()));
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                to_walk.push(path);
            } else if path.extension().is_some_and(|kind| kind == "rs") {
                found.push(path.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    found.sort();
    found
}

#[test]
fn test_the_route_starts_this_program_again_rather_than_opening_the_browser() {
    // 11-11.1 left the third route calling `open::that` and telling a line
    // that said separate windows arrive with the next build, because the
    // window did not exist. It starts a page process now, and the browser is
    // what it falls back to when that fails rather than what it does.
    //
    // What this cannot see: that the process really starts. Three of this
    // target's cases run the built executable and are what say that.
    let ships = what_ships_in("src/presentation/wx_app.rs");
    let starting = the_body_of(&ships, "fn a_window_of_its_own(");

    assert!(
        starting.contains("current_exe"),
        "the route starts something other than this program"
    );
    assert!(
        starting.contains("page_window::FLAG"),
        "the route names the flag by hand rather than reading the one spelling of it"
    );
    assert!(
        starting.contains("spawn()"),
        "the route waits for the window it started, which would stop the mail"
    );
    assert!(
        ships.contains("a_window_of_its_own(&safe)"),
        "nothing reaches the spawn with the sanitised address"
    );
}

#[test]
fn test_nothing_in_the_program_still_says_a_separate_window_is_coming() {
    // Ledger 566: the sentence under Open links on the Reading tab said a
    // separate Wixen Mail window arrives with the next build, and the status
    // line when that choice was taken said it again. Both were true when
    // they were written and stopped being true when the plan that builds the
    // window was put off, so a build cut in between would have carried a
    // promise the guide already said was wrong.
    //
    // Read with the newlines taken out, because both sentences are wrapped
    // and a line-at-a-time search finds neither.
    for file in every_rust_file_under("src") {
        let joined = what_ships_in(&file).replace('\n', " ");
        for promise in ["next build", "SEPARATE_WINDOWS_ARRIVE_LATER"] {
            assert!(
                !joined.contains(promise),
                "{file} still says {promise:?}, and the window is here"
            );
        }
    }
}

#[test]
fn test_the_erase_removes_the_page_profile_as_well_as_the_root() {
    // The page profile is the one folder this program writes that
    // `WIXEN_MAIL_DATA` does not move: WebView2's data path comes from
    // wxWidgets, which reads the local data folder from the Windows
    // known-folder API. So with the variable set it is a second place, and
    // an erase that removed only the root would leave a browser profile
    // behind on a machine somebody had just wiped.
    //
    // What this cannot see: that the removal succeeds. `erase_all_data`
    // reports what it could not remove, and that path is its own.
    let ships = what_ships_in("src/main.rs");

    assert!(
        ships.contains("page_profile_dir("),
        "the erase never asks where the page profile is, so it cannot remove it"
    );
}

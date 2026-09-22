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
//! What that cannot see, said plainly. A start that did open a window would
//! not fail here, it would hang: the run waits for the process to finish, so
//! the timing below catches a refusal that got slow and not a refusal that
//! never came. A hang in this target is that, and reads as one.
//!
//! The rest is read from the source, and each reading says what it cannot
//! see. That a page really appears in that window, with its title spoken and
//! Escape closing it, is the tester's ear and is on the ledger.

use std::path::Path;
use std::process::Command;
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
    code: Option<i32>,
    complaint: String,
    took: Duration,
}

/// Run the built executable with these arguments, against a throwaway data
/// folder.
///
/// `WIXEN_MAIL_DATA` is set because a refused start writes its reason to the
/// crash file, and that file belongs to whoever is running this rather than
/// to the person whose machine it is.
fn start_and_wait(arguments: &[&str]) -> WhatItAnswered {
    let folder = tempfile::TempDir::new().expect("a throwaway data folder");
    let began = Instant::now();
    let finished = Command::new(env!("CARGO_BIN_EXE_wixen-mail"))
        .args(arguments)
        .env("WIXEN_MAIL_DATA", folder.path())
        .output()
        .expect("the built executable should run");
    WhatItAnswered {
        code: finished.status.code(),
        complaint: String::from_utf8_lossy(&finished.stderr).to_string(),
        took: began.elapsed(),
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
    for refused in [
        "not-a-page",
        "mailto:somebody@example.com",
        "javascript:alert(1)",
        "file:///C:/Windows/win.ini",
        "tel:+15550100",
    ] {
        let answered = start_and_wait(&["--show-page", refused]);

        assert_eq!(answered.code, Some(2), "{refused} did not stop the start");
        assert!(
            answered.took < LONG_ENOUGH_TO_REFUSE,
            "{refused} took {:?}, which is long enough to have opened something",
            answered.took
        );
    }
}

#[test]
fn test_the_page_flag_with_nothing_after_it_is_refused_and_says_what_it_wanted() {
    // The same rule every other flag with a value follows, measured on the
    // executable rather than on the parser, because this is the one flag a
    // person meets without having typed it, and the complaint is the only
    // thing it can read.
    let answered = start_and_wait(&["--show-page"]);

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
fn test_the_page_process_names_its_profile_before_it_builds_a_browser() {
    // T-12-06. The application name decides where WebView2 puts the
    // profile, and it is read when the environment is made, which is when
    // the first browser control is built. Named afterwards it would name a
    // folder nothing uses and the page would share the message preview's
    // profile, which is the thing #80's third item is about.
    let ships = what_ships_in("src/presentation/page_window.rs");
    let names = ships
        .find("set_app_name")
        .expect("the page process names its profile");
    let builds = ships
        .find("WebView::builder")
        .expect("the page process builds a browser");

    assert!(
        names < builds,
        "the profile is named after the browser is built, so the browser did not get it"
    );
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

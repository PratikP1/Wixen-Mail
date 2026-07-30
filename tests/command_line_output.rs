//! What the program prints when it is run from a script rather than clicked.
//!
//! These run the real executable, because the thing being checked is not what
//! a function returns, it is which stream the words come out of and what the
//! exit code is. Nothing here opens a window: `--help`, `--version` and a
//! refusal all answer and stop.
//!
//! # Why it matters which stream
//!
//! `wixen-mail --allow evrything > log.txt` should leave the complaint on the
//! screen, not silently in the file. Anything reading the output of a
//! successful run should not have to filter error text out of it. That is the
//! ordinary contract, and this application is meant to grow a command line
//! somebody can rely on.
//!
//! # Why it matters that something is written at all
//!
//! `windows_subsystem = "windows"` means the program starts with no console,
//! so it has to go looking for somewhere to put words. When it finds nowhere
//! it falls back to a message box, which is right for a double click and very
//! wrong for a build server: a modal dialog nobody can see does not fail, it
//! hangs until the job times out. Whenever output has been redirected, as it
//! is here, writing to the redirect has to win over the dialog.

use std::process::{Command, Output};

/// Run the built executable with the given arguments, capturing both streams.
///
/// Both are piped, which is the situation the tests are about: there is
/// somewhere to write, so nothing should ever reach for a dialog.
fn run(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_wixen-mail"))
        .args(arguments)
        .output()
        .expect("the executable should run")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

#[test]
fn test_version_goes_to_the_output_stream() {
    let result = run(&["--version"]);

    assert_eq!(result.status.code(), Some(0));
    assert!(
        text(&result.stdout).contains(env!("CARGO_PKG_VERSION")),
        "stdout was {:?}",
        text(&result.stdout)
    );
    assert_eq!(text(&result.stderr), "", "nothing went wrong, so stderr");
}

#[test]
fn test_help_goes_to_the_output_stream() {
    let result = run(&["--help"]);

    assert_eq!(result.status.code(), Some(0));
    let said = text(&result.stdout);
    assert!(said.contains("--read-only"), "stdout was {said:?}");
    assert!(
        said.contains("experimental"),
        "the warning that writing is untested is part of --help"
    );
    assert_eq!(text(&result.stderr), "");
}

#[test]
fn test_a_refusal_goes_to_the_error_stream() {
    // The bug this was written for: it went to stdout, so redirecting the
    // output of a run that refused to start put the reason in the file and
    // left the screen blank.
    let result = run(&["--frobnicate"]);

    assert_eq!(result.status.code(), Some(2), "a refusal is not a success");
    assert!(
        text(&result.stderr).contains("--frobnicate"),
        "stderr was {:?}",
        text(&result.stderr)
    );
    assert_eq!(
        text(&result.stdout),
        "",
        "a run that refused to start has no output"
    );
}

#[test]
fn test_a_bad_value_for_allow_is_refused_the_same_way() {
    let result = run(&["--allow", "evrything"]);

    assert_eq!(result.status.code(), Some(2));
    assert!(
        text(&result.stderr).contains("evrything"),
        "the complaint should quote what was typed, stderr was {:?}",
        text(&result.stderr)
    );
    assert_eq!(text(&result.stdout), "");
}

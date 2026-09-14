//! Opening one window on purpose, so the accessibility scan can see it.
//!
//! The automated scan launches the application, waits for the main window, and
//! walks the UI Automation tree of the whole process. That covers every window
//! the process owns, which sounds like everything and is not: at that moment
//! the process owns one window. Every dialog in the application, which is where
//! most of the controls live, has never been scanned by anything.
//!
//! `--scan-window=<name>` opens one and leaves it open. The scan runs once per
//! name, so the settings dialog, the accounts dialog, the composer and the rest
//! each get looked at.
//!
//! # Why an unknown name has to fail
//!
//! A name nobody recognises must not quietly open nothing. The scan would then
//! run against the main window, find what it always finds, and report a clean
//! pass for a dialog it never opened. That is the failure mode this project has
//! already shipped once, in a scan that reported success while its own scan
//! step had errored, and it is worse than no scan at all because it looks
//! maintained.

use crate::common::{Error, Result};

/// The flag that asks for a window to be opened for scanning.
///
/// One spelling, in one place, read by the command line parser, quoted in
/// `--help`, and used by `.github/workflows/accessibility.yml`. There were two
/// for a while, `--scan-window` here and `--scan-target` in the parser, and
/// the result was the failure this module was written to prevent: the parser
/// accepted `--scan-target settings`, handed the name to a reader that was
/// looking for the other spelling, got back "no window asked for", and started
/// normally. Every dialog scan since had been a second scan of the main window
/// reported as a pass. A test below pins the flag to the workflow.
pub const FLAG: &str = "--scan-target";

/// The exit code when the window the scan asked for is not open.
///
/// Every dialog target is modal, so the call that opens it does not return
/// until the window closes, and the workflow kills the process while the
/// window is still up. The call returning at all therefore means the window
/// is not on screen: it refused to open, or it opened on nothing and closed
/// itself. Either way the process owns one window, the main one, and a scan
/// of it now would be reported as a pass for a dialog nobody looked at. So
/// the process leaves with this code instead, and the workflow reads it as
/// "not scanned" rather than as a crash, which is a different failure with a
/// different fix.
pub const WINDOW_NOT_OPEN: i32 = 3;

/// A window the scan can be pointed at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanTarget {
    Settings,
    Accounts,
    Compose,
    Reader,
    Search,
    Filters,
    /// The window listing the events on a calendar, opened on whatever the
    /// account being looked at already holds rather than a fixture built for
    /// the scan: a fresh profile has none, so Edit and Delete are scanned
    /// with nothing selected, the same shape their own "nothing selected"
    /// answer is tested against.
    Calendar,
    /// The screen asking what Wixen Mail may change, which everybody meets
    /// once and which no scan could reach before: it opens by itself on a
    /// fresh profile and never again after the answer is stored.
    FirstRun,
    /// The screen that adds a calendar by its server or feed address. It is
    /// the first window in the application that asks for a password to send
    /// somewhere other than a mail server, so what it says about its own
    /// controls matters more than most.
    AddCalendar,
    /// The list of who is blocked. A report list read row by row, with a
    /// destructive button beside it, which is the control shape where
    /// structure being present and the experience being good come apart most
    /// often. A fresh profile has nothing blocked, so the scan meets the
    /// window in its empty state, which is the state that has to sound
    /// deliberate rather than broken.
    BlockedSenders,
}

impl ScanTarget {
    /// Every target, so the workflow and the tests iterate the same list
    /// rather than each keeping their own copy of it.
    pub const ALL: [ScanTarget; 10] = [
        ScanTarget::Settings,
        ScanTarget::Accounts,
        ScanTarget::Compose,
        ScanTarget::Reader,
        ScanTarget::Search,
        ScanTarget::Filters,
        ScanTarget::Calendar,
        ScanTarget::FirstRun,
        ScanTarget::AddCalendar,
        ScanTarget::BlockedSenders,
    ];

    /// The name used on the command line.
    pub const fn as_name(self) -> &'static str {
        match self {
            Self::Settings => "settings",
            Self::Accounts => "accounts",
            Self::Compose => "compose",
            Self::Reader => "reader",
            Self::Search => "search",
            Self::Filters => "filters",
            Self::Calendar => "calendar",
            Self::FirstRun => "first-run",
            Self::AddCalendar => "add-calendar",
            Self::BlockedSenders => "blocked-senders",
        }
    }

    /// Match a name from the command line.
    fn matching(name: &str) -> Option<Self> {
        let wanted = name.trim().to_ascii_lowercase();
        Self::ALL.into_iter().find(|t| t.as_name() == wanted)
    }
}

/// Which window a name asks for.
///
/// An unrecognised name is an error rather than "no window", because silently
/// starting normally lets the scan report a clean pass for a window it never
/// opened.
pub fn named(name: &str) -> Result<ScanTarget> {
    ScanTarget::matching(name).ok_or_else(|| {
        Error::Other(format!(
            "{FLAG}={name} is not a window this knows about. Known names: {}",
            known_names()
        ))
    })
}

/// The names, for an error message that says what would have worked.
fn known_names() -> String {
    ScanTarget::ALL
        .iter()
        .map(|target| target.as_name())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_target_can_be_named_on_the_command_line() {
        for target in ScanTarget::ALL {
            assert_eq!(
                named(target.as_name()).expect("a known name"),
                target,
                "{target:?} could not be asked for"
            );
        }
    }

    #[test]
    fn test_a_name_nobody_recognises_is_an_error_rather_than_a_shrug() {
        // The whole point. Starting normally on a typo would have the scan
        // walk the main window, find what it always finds, and report a clean
        // pass for a dialog it never opened.
        let error = named("setings").expect_err("a typo");

        assert!(error.to_string().contains("setings"), "{error}");
        // And it says what would have worked.
        assert!(error.to_string().contains("settings"), "{error}");
    }

    #[test]
    fn test_a_name_is_matched_whatever_case_it_is_written_in() {
        assert_eq!(named("Settings").expect("mixed case"), ScanTarget::Settings);
    }

    #[test]
    fn test_the_command_line_and_the_workflow_use_the_same_flag() {
        // The workflow file is the artefact, and it cannot be run from here.
        // What this cannot see is whether the workflow passes, only whether the
        // two places spell one flag the same way.
        // The bug this was written for was silent and total: the parser knew
        // --scan-target, this module was reading --scan-window, so every
        // dialog scan quietly became a second scan of the main window and the
        // workflow reported a pass. Nothing failed, which is why it survived.
        let workflow = std::fs::read_to_string(".github/workflows/accessibility.yml")
            .expect("the accessibility workflow");

        assert!(
            workflow.contains(FLAG),
            "the workflow does not pass {FLAG}, so it is asking for a window by some other name"
        );
        assert!(
            !workflow.contains("--scan-window"),
            "the workflow still uses the old spelling, which the parser refuses"
        );

        let help = crate::presentation::command_line::HELP;
        assert!(help.contains(FLAG), "--help does not document {FLAG}");
    }

    #[test]
    fn test_the_workflow_asks_for_every_target() {
        // The workflow file is the artefact, and it cannot be run from here.
        // What this cannot see is whether a scan of any target really happens.
        // Adding a window to the list and forgetting the workflow means it is
        // never scanned, and nothing says so.
        let workflow = std::fs::read_to_string(".github/workflows/accessibility.yml")
            .expect("the accessibility workflow");

        for target in ScanTarget::ALL {
            assert!(
                workflow.contains(&format!("'{}'", target.as_name())),
                "{} is not in the workflow's target list, so it is never scanned",
                target.as_name()
            );
        }
    }

    #[test]
    fn test_every_window_a_fresh_profile_can_reach_has_a_name() {
        // The list Pratik answered on 2026-09-14: every window a fresh
        // profile can open, not the eleven that happened to be there. Each
        // name here is a window somebody meets, counted from the tree rather
        // than from the plan, and a name missing from this list is a window
        // the scan has never looked at.
        for name in [
            "columns",
            "which-copy",
            "destination",
            "folder-choice",
            "new-event",
            "contacts",
            "tags",
            "signatures",
            "reminder",
            "conversation",
            "which-days",
            "send-later",
            "add-address-book",
            "about",
            "calendar-module",
            "contacts-module",
            "reminders-module",
            "tasks-module",
            "notes-module",
        ] {
            named(name)
                .unwrap_or_else(|e| panic!("{name} is not a window the scan can ask for: {e}"));
        }
    }

    #[test]
    fn test_the_workflow_pins_the_scanner_to_a_release_and_its_checksum() {
        // The rule set the scan runs has to be one a document can name. With
        // `releases/latest` it was whatever Microsoft shipped most recently,
        // and the rule table read twice five days apart did not agree, with
        // nobody able to say how much was upstream moving. A tag can be moved
        // by whoever owns the repository; a checksum cannot, so the coverage
        // list 06-07 writes is a claim about one binary rather than a name.
        let workflow = std::fs::read_to_string(".github/workflows/accessibility.yml")
            .expect("the accessibility workflow");

        assert!(
            !workflow.contains("releases/latest"),
            "the workflow still fetches whatever release is newest, so the rule set can \
             change with no commit here"
        );
        assert!(
            workflow.contains("/releases/download/v"),
            "the workflow does not download a named release"
        );
        let has_a_sha256 = workflow
            .lines()
            .any(|line| line.contains("Get-FileHash") && line.contains("SHA256"));
        assert!(has_a_sha256, "the workflow never hashes what it downloaded");
        let names_the_expected_hash = workflow.lines().any(|line| {
            line.split_whitespace().any(|word| {
                let word = word.trim_matches(|c| c == '\'' || c == '"');
                word.len() == 64 && word.chars().all(|c| c.is_ascii_hexdigit())
            })
        });
        assert!(
            names_the_expected_hash,
            "the workflow hashes the download but never says what the hash has to be"
        );
    }

    #[test]
    fn test_the_workflow_asks_whether_the_application_is_still_running_before_it_scans() {
        // A dialog that fails to open leaves the main window up and the
        // process alive, so a scan of it is a second scan of the main window
        // reported as a pass for a dialog nobody looked at. The program now
        // exits when the window it was asked for is not open, and the
        // workflow has to notice that before it scans, or the exit is a
        // sentence in a log nobody reads.
        let workflow = std::fs::read_to_string(".github/workflows/accessibility.yml")
            .expect("the accessibility workflow");

        let scans_at = workflow
            .find("AxeWindowsCLI.exe `")
            .expect("the workflow runs the scanner");
        let asks_at = workflow
            .find("$app.HasExited")
            .expect("the workflow never asks whether the application is still running");
        assert!(
            asks_at < scans_at,
            "the workflow asks whether the application is still running only after the \
             scan, so a window that never opened is scanned as the main window"
        );
        assert!(
            workflow.contains(&format!("-eq {WINDOW_NOT_OPEN}")),
            "the workflow does not tell the exit code that means the window was not \
             open from a crash"
        );
    }

    #[test]
    fn test_every_target_has_its_own_name() {
        // Two targets sharing a name would mean one of them could never be
        // scanned, and nothing would say which.
        let mut names: Vec<&str> = ScanTarget::ALL.iter().map(|t| t.as_name()).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();

        assert_eq!(names.len(), count, "two targets share a name");
    }
}

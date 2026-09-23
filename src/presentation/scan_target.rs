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

/// What the call that opened a target's window reports once it returns.
///
/// A modal window holds the call until it closes, so the call returning means
/// the window has gone; a frame or a module panel is shown and left, so the
/// call returns with the window still up. The two need telling apart at the
/// one place that decides whether returning is a failure, and a `bool` there
/// would read as "did it work", which is the opposite of what a modal
/// returning means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnReturn {
    /// A modal window was up and is no longer, or never opened at all.
    WindowClosed,
    /// A window or panel that does not block was shown and is still there.
    WindowStillUp,
}

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
    // The nineteen below arrived together on 2026-09-14, on Pratik's answer
    // that the scan should look at every window a fresh profile can reach
    // rather than the eleven that happened to be there. Each opens the way a
    // person opens it where a fresh profile allows that, and on made-up data
    // from `scan_fixtures` where the window refuses to open on nothing.
    /// The column chooser, on the inbox's default layout.
    Columns,
    /// The window asking which copy of a contact to keep, over two copies
    /// that disagree.
    WhichCopy,
    /// The window asking where a message goes, over two accounts that both
    /// have an Archive, which is the window's own reason for being a tree.
    Destination,
    /// The checked list of folders an account keeps up to date, with one
    /// row that holds a copy of every message.
    FolderChoice,
    /// The item form, opened as a new event: the one shape with date and time
    /// fields, a repeat notebook and the guest list. Reached the way File,
    /// New, Event reaches it, filed on this computer since a fresh profile
    /// has no account.
    NewEvent,
    /// The contact manager, the same shape as the filter manager and scanned
    /// for the same reason.
    Contacts,
    /// The tag manager.
    Tags,
    /// The signature manager.
    Signatures,
    /// The window that opens when something comes due, with one row of each
    /// kind, a task, a reminder and an event, all late, which is the wording
    /// said first. Named for the reminder because it was one reminder as a
    /// line of text until 06-09 and the name is fingerprinted.
    Reminder,
    /// The conversation tree, on a conversation with a reply in it so the
    /// tree has a second level to announce.
    Conversation,
    /// The window asking whether a change is meant for one day or all of
    /// them, on an event that repeats. The one skipped NVDA test names this
    /// dialog and says the skip exists because there was no target for it.
    WhichDays,
    /// The window asking when a message should go, which is the composer's
    /// and is opened here on the main window instead, since the composer is
    /// its own target and one scan is one window.
    SendLater,
    /// The window that adds an address book by its address, the second in
    /// the application that asks for a password to send somewhere other
    /// than a mail server.
    AddAddressBook,
    /// The About window.
    About,
    /// The main window on its own, showing mail. `main` is not that: with no
    /// target given the first-run question opens over the frame on a fresh
    /// profile, so `main` has always been the frame with a modal on top of
    /// it, and the bare window had never been scanned. This asks for a
    /// target, which is what skips that question.
    MailModule,
    /// The main window with the calendar module showing. A fresh profile
    /// opens on mail, so the other five module panels had never been scanned
    /// at all.
    CalendarModule,
    /// The main window with the contacts module showing.
    ContactsModule,
    /// The main window with the reminders module showing.
    RemindersModule,
    /// The main window with the tasks module showing.
    TasksModule,
    /// The main window with the notes module showing.
    NotesModule,
    // The five below arrived on 2026-09-16 (#42, #40 point 5). Each is an
    // editor that opens only from inside a manager, behind Add or Edit, so
    // the manager targets above never reached it: the scan walked the
    // manager and reported a pass for an editor nobody looked at. Two
    // testers met an unnamed checkbox in two of them on the first day of
    // testing. Each opens directly on a fixture from `scan_fixtures`, as
    // `WhichDays` and `SendLater` do, and the manager it belongs to is not
    // built at all.
    /// The contact editor, "Edit Contact", opened on a contact with every
    /// field filled and a row on each of its four lists. `Contacts` reached
    /// the Contact Manager and stopped at Add.
    ContactEditor,
    /// The condition editor, "Edit Condition", the second dialog of the
    /// saved-search editor, opened on a stored condition. No target reached
    /// it: it opens from inside the rule manager, which opens from inside a
    /// saved search.
    ConditionEditor,
    /// The filter editor, "Edit Filter Rule", opened on a stored rule.
    /// `Filters` reached the Filter Manager and stopped at Add.
    FilterEditor,
    /// The signature editor, "Edit Signature", opened on the default
    /// signature, which is where the tester met the unnamed checkbox.
    /// `Signatures` reached the Signature Manager and stopped at Add.
    SignatureEditor,
    /// The account editor, "Edit Account", opened on the scan-only account
    /// the `Accounts` target uses, and turned to its second page, where
    /// every checkbox it has is. `Accounts` reached the Account Manager and
    /// stopped at Edit.
    AccountEditor,
    /// The formatted message window, which a message opens into under the
    /// default reading style and which `Reader` never reached: that target
    /// opens the plain-text reader. Opened on a made-up conversation of two
    /// messages, one written as a page with a link in it and one written as
    /// text with an address on a line of its own that the renderer makes a
    /// link, so the NVDA case for #80 has a sender's link and a made one to
    /// press Enter on (11-11.1, 2026-09-20).
    Page,
    /// The separate Wixen Mail window a link opens in, which in the shipped
    /// program is a process of its own with a browser profile of its own
    /// (#80, 12-02). Built here inside this process instead, on a document
    /// of its own: the runner has no network, and what the scan walks is
    /// this window's names and roles rather than a live page's. Starting the
    /// real page process would also put the scan on the wrong process, since
    /// it walks the tree of the one it launched.
    PageWindow,
}

impl ScanTarget {
    /// Every target, so the workflow and the tests iterate the same list
    /// rather than each keeping their own copy of it.
    pub const ALL: [ScanTarget; 37] = [
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
        ScanTarget::Columns,
        ScanTarget::WhichCopy,
        ScanTarget::Destination,
        ScanTarget::FolderChoice,
        ScanTarget::NewEvent,
        ScanTarget::Contacts,
        ScanTarget::Tags,
        ScanTarget::Signatures,
        ScanTarget::Reminder,
        ScanTarget::Conversation,
        ScanTarget::WhichDays,
        ScanTarget::SendLater,
        ScanTarget::AddAddressBook,
        ScanTarget::About,
        ScanTarget::MailModule,
        ScanTarget::CalendarModule,
        ScanTarget::ContactsModule,
        ScanTarget::RemindersModule,
        ScanTarget::TasksModule,
        ScanTarget::NotesModule,
        ScanTarget::ContactEditor,
        ScanTarget::ConditionEditor,
        ScanTarget::FilterEditor,
        ScanTarget::SignatureEditor,
        ScanTarget::AccountEditor,
        ScanTarget::Page,
        ScanTarget::PageWindow,
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
            Self::Columns => "columns",
            Self::WhichCopy => "which-copy",
            Self::Destination => "destination",
            Self::FolderChoice => "folder-choice",
            Self::NewEvent => "new-event",
            Self::Contacts => "contacts",
            Self::Tags => "tags",
            Self::Signatures => "signatures",
            Self::Reminder => "reminder",
            Self::Conversation => "conversation",
            Self::WhichDays => "which-days",
            Self::SendLater => "send-later",
            Self::AddAddressBook => "add-address-book",
            Self::About => "about",
            Self::MailModule => "mail-module",
            Self::CalendarModule => "calendar-module",
            Self::ContactsModule => "contacts-module",
            Self::RemindersModule => "reminders-module",
            Self::TasksModule => "tasks-module",
            Self::NotesModule => "notes-module",
            Self::ContactEditor => "contact-editor",
            Self::ConditionEditor => "condition-editor",
            Self::FilterEditor => "filter-editor",
            Self::SignatureEditor => "signature-editor",
            Self::AccountEditor => "account-editor",
            Self::Page => "page",
            Self::PageWindow => "page-window",
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
        //
        // The five editors arrived on 2026-09-16 (#42, #40): each opens only
        // from inside a manager, behind Add or Edit, so the manager targets
        // never reached them, and two testers met an unnamed checkbox in two
        // of them on the first day of testing.
        //
        // The page window arrived on 2026-09-20 (#80): the default way a
        // message opens, never scanned, because `reader` opens the other
        // surface. The separate window followed it on 2026-09-22, when the
        // third place a link can open in stopped being a status line.
        //
        // Send Feedback arrived on 2026-09-23 (#64), with the dialog it names.
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
            "feedback",
            "mail-module",
            "calendar-module",
            "contacts-module",
            "reminders-module",
            "tasks-module",
            "notes-module",
            "contact-editor",
            "condition-editor",
            "filter-editor",
            "signature-editor",
            "account-editor",
            "page",
            "page-window",
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
        // What the workflow does, not what its comments say about what it
        // used to do.
        let commands: Vec<&str> = workflow
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .collect();

        assert!(
            !commands.iter().any(|line| line.contains("releases/latest")),
            "the workflow still fetches whatever release is newest, so the rule set can \
             change with no commit here"
        );
        assert!(
            commands.iter().any(|line| line.contains("$tag = 'v")),
            "the workflow does not name a release tag"
        );
        let has_a_sha256 = commands
            .iter()
            .any(|line| line.contains("Get-FileHash") && line.contains("SHA256"));
        assert!(has_a_sha256, "the workflow never hashes what it downloaded");
        let names_the_expected_hash = commands.iter().any(|line| {
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

    /// The three sentences Axe.Windows prints under a scan, and the count
    /// each one carries. The singular is real: the CLI writes `1 error was
    /// found`, not `1 errors were found`.
    const WHAT_AXE_PRINTS: [(&str, Option<u32>); 3] = [
        ("1 error was found", Some(1)),
        ("3 errors were found", Some(3)),
        ("No errors were found", None),
    ];

    /// The pattern the workflow counts findings with, read from the line that
    /// applies it, so the test is about the pattern the run really uses.
    fn the_counting_pattern(workflow: &str) -> String {
        let line = workflow
            .lines()
            .find(|line| line.contains("Select-String -Pattern '") && line.contains("were found"))
            .expect("the workflow counts findings with a Select-String pattern");
        let after =
            &line[line.find("-Pattern '").expect("the pattern opens") + "-Pattern '".len()..];
        after
            .split('\'')
            .next()
            .expect("the pattern closes")
            .to_string()
    }

    /// Which of Axe's three sentences `pattern` either misses or counts
    /// wrongly. Empty when it reads all three the way the CLI writes them.
    fn what_the_pattern_gets_wrong(pattern: &str) -> Vec<String> {
        let pattern = regex::Regex::new(pattern).expect("the workflow's pattern is a regex");
        WHAT_AXE_PRINTS
            .iter()
            .filter_map(|(printed, count)| {
                let read = pattern
                    .captures(printed)
                    .map(|found| found.get(1).map(|n| n.as_str().parse::<u32>().ok()));
                match (read, count) {
                    (None, _) => Some(format!("{printed:?} is not matched at all")),
                    (Some(read), count) if read.flatten() != *count => {
                        Some(format!("{printed:?} is read as {read:?}, not {count:?}"))
                    }
                    _ => None,
                }
            })
            .collect()
    }

    #[test]
    fn test_the_workflow_counts_one_error_as_one_and_not_as_none() {
        // Axe prints the singular as `1 error was found`. The pattern that
        // counted findings matched `errors? were found`, which reads the
        // plural and the clean case and nothing else, so on 2026-09-14 three
        // windows that each printed one error were recorded as clean and the
        // run's total was 26 where the log held 29. A check that reports a
        // finding as no finding is guardrail 4 in the check itself.
        let workflow = std::fs::read_to_string(".github/workflows/accessibility.yml")
            .expect("the accessibility workflow");

        let wrong = what_the_pattern_gets_wrong(&the_counting_pattern(&workflow));

        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    }

    #[test]
    fn test_the_reading_can_see_a_pattern_that_misses_the_singular() {
        // The pattern that was in the workflow until 2026-09-14, so the check
        // above is known to be able to fail.
        let wrong = what_the_pattern_gets_wrong(r"(\d+) errors? were found|No errors were found");

        assert_eq!(wrong, vec!["\"1 error was found\" is not matched at all"]);
    }
}

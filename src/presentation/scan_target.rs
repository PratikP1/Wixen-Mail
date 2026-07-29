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
pub const FLAG: &str = "--scan-window";

/// A window the scan can be pointed at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanTarget {
    Settings,
    Accounts,
    Compose,
    Reader,
    Search,
    Filters,
}

impl ScanTarget {
    /// Every target, so the workflow and the tests iterate the same list
    /// rather than each keeping their own copy of it.
    pub const ALL: [ScanTarget; 6] = [
        ScanTarget::Settings,
        ScanTarget::Accounts,
        ScanTarget::Compose,
        ScanTarget::Reader,
        ScanTarget::Search,
        ScanTarget::Filters,
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
        }
    }

    /// Match a name from the command line.
    fn from_name(name: &str) -> Option<Self> {
        let wanted = name.trim().to_ascii_lowercase();
        Self::ALL.into_iter().find(|t| t.as_name() == wanted)
    }
}

/// Which window, if any, the arguments ask to be opened for scanning.
///
/// `Ok(None)` is the ordinary case: no flag, so the application starts
/// normally. An unrecognised name is an error rather than `None`, because
/// silently starting normally would let the scan report a clean pass for a
/// window it never opened.
pub fn from_args<I, S>(args: I) -> Result<Option<ScanTarget>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    for arg in args {
        let arg = arg.as_ref();
        let Some(value) = arg.strip_prefix(FLAG) else {
            continue;
        };
        // `--scan-window=settings`, and nothing else. A bare `--scan-window`
        // with the name as the next argument would be a second way to write
        // the same thing, and two ways to write it is two things to get wrong.
        let Some(name) = value.strip_prefix('=') else {
            return Err(Error::Other(format!(
                "{FLAG} needs a name, as {FLAG}=settings. Known names: {}",
                known_names()
            )));
        };
        return match ScanTarget::from_name(name) {
            Some(target) => Ok(Some(target)),
            None => Err(Error::Other(format!(
                "{FLAG}={name} is not a window this knows about. Known names: {}",
                known_names()
            ))),
        };
    }
    Ok(None)
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
    fn test_no_flag_means_start_normally() {
        assert_eq!(from_args::<[&str; 0], &str>([]).expect("no args"), None);
        assert_eq!(
            from_args(["wixen-mail.exe", "--something-else"]).expect("other args"),
            None
        );
    }

    #[test]
    fn test_every_target_can_be_named_on_the_command_line() {
        for target in ScanTarget::ALL {
            let arg = format!("{FLAG}={}", target.as_name());

            assert_eq!(
                from_args([arg.as_str()]).expect("a known name"),
                Some(target),
                "{target:?} could not be asked for"
            );
        }
    }

    #[test]
    fn test_a_name_nobody_recognises_is_an_error_rather_than_a_shrug() {
        // The whole point. Starting normally on a typo would have the scan
        // walk the main window, find what it always finds, and report a clean
        // pass for a dialog it never opened.
        let error = from_args([format!("{FLAG}=setings")]).expect_err("a typo");

        assert!(error.to_string().contains("setings"), "{error}");
        // And it says what would have worked.
        assert!(error.to_string().contains("settings"), "{error}");
    }

    #[test]
    fn test_the_flag_without_a_name_says_how_to_write_it() {
        let error = from_args([FLAG]).expect_err("no name");

        assert!(error.to_string().contains("=settings"), "{error}");
    }

    #[test]
    fn test_a_name_is_matched_whatever_case_it_is_written_in() {
        assert_eq!(
            from_args([format!("{FLAG}=Settings")]).expect("mixed case"),
            Some(ScanTarget::Settings)
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

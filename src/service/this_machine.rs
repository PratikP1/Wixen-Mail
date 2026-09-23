//! What this machine says about itself, for a feedback report (#64).
//!
//! Three questions asked of Windows: which build it is, which language it
//! shows, and which screen reader is running. Each has a pure half that
//! decides the answer from plain values, which the cases below drive, and a
//! Win32 half that fetches those values, declared by hand on the tree's
//! `extern "system"` pattern so no crate and no feature is added. On other
//! platforms the Win32 half answers that it does not know.
//!
//! Nothing here reads an address, a name or a message. The accounts are
//! described by the kind of mail they are, never by what they are called.

use crate::data::account::Account;

/// Which Windows this is, such as "Windows 11, build 26200".
pub fn windows_build() -> String {
    String::new()
}

/// The language Windows shows, such as "en-GB".
pub fn display_language() -> String {
    String::new()
}

/// The screen reader running and its version, when one is.
pub fn screen_reader() -> Option<(String, String)> {
    None
}

/// Which screen reader a list of running process names shows, if any.
pub fn which_reader(process_names: &[String]) -> Option<&'static str> {
    let _ = process_names;
    None
}

/// A Windows version as a person names it.
pub fn describe_windows(major: u32, minor: u32, build: u32) -> String {
    let _ = (major, minor, build);
    String::new()
}

/// A file version from the two words a version resource carries it in.
pub fn file_version_text(most_significant: u32, least_significant: u32) -> String {
    let _ = (most_significant, least_significant);
    String::new()
}

/// The kinds of account set up, without their addresses.
pub fn providers(accounts: &[Account]) -> Vec<String> {
    let _ = accounts;
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|name| name.to_string()).collect()
    }

    #[test]
    fn test_each_reader_is_found_by_its_process_whatever_case_it_is_written_in() {
        for (running, reader) in [
            ("nvda.exe", "NVDA"),
            ("NVDA.EXE", "NVDA"),
            ("jfw.exe", "JAWS"),
            ("Narrator.exe", "Narrator"),
            ("narrator.EXE", "Narrator"),
        ] {
            assert_eq!(
                which_reader(&names(&["explorer.exe", running, "svchost.exe"])),
                Some(reader),
                "{running}"
            );
        }
    }

    #[test]
    fn test_no_reader_is_named_when_none_runs_and_nvda_is_named_before_narrator() {
        assert_eq!(
            which_reader(&names(&["explorer.exe", "nvda_helper.exe"])),
            None
        );
        assert_eq!(
            which_reader(&names(&["Narrator.exe", "nvda.exe"])),
            Some("NVDA")
        );
    }

    #[test]
    fn test_windows_is_named_the_way_a_person_names_it() {
        assert_eq!(describe_windows(10, 0, 26200), "Windows 11, build 26200");
        assert_eq!(describe_windows(10, 0, 22000), "Windows 11, build 22000");
        assert_eq!(describe_windows(10, 0, 19045), "Windows 10, build 19045");
        assert_eq!(describe_windows(6, 3, 9600), "Windows 6.3, build 9600");
    }

    #[test]
    fn test_a_file_version_is_read_from_its_two_words() {
        assert_eq!(file_version_text(0x07E9_0003, 0x0000_0001), "2025.3.0.1");
    }

    #[test]
    fn test_accounts_are_named_by_their_kind_and_never_by_their_address() {
        let mut gmail = Account::new("Work".to_string(), "dana@gmail.com".to_string());
        gmail.provider = Some("Gmail".to_string());
        let mut pop = Account::new("Home".to_string(), "dana@example.net".to_string());
        pop.protocol = crate::common::types::Protocol::Pop3.as_str().to_string();
        let second_gmail = Account {
            id: "b".to_string(),
            ..gmail.clone()
        };

        let named = providers(&[gmail, pop, second_gmail]);

        assert_eq!(named, names(&["Gmail", "POP3"]));
        assert!(
            !named.iter().any(|n| n.contains('@') || n.contains("Work")),
            "{named:?}"
        );
    }

    #[test]
    fn test_this_machine_answers_with_a_windows_it_names() {
        // The one case that asks the real machine, so the Win32 half is
        // known to be reached and to answer something a person can read.
        let answer = windows_build();

        assert!(answer.starts_with("Windows"), "{answer:?}");
    }
}

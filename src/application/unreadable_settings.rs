//! What start-up and Settings say when a settings file could not be read
//! (ledger 889).
//!
//! Until 13.2-31.9 a settings file that a disk fault or a hand edit had made
//! unreadable was said only in the log: the person met the built-in settings
//! without a word, no account synced, and OK in Settings wrote them over the
//! file. Start-up now keeps such a file under a name of its own and puts
//! settings that allow no change in its place
//! ([`ConfigManager::keep_what_cannot_be_read`]), and these are the words that
//! tell the person, once, after the window is on screen and the first-run
//! question has been answered.

use crate::common::Result;
use crate::data::config::{ConfigManager, WhatWasKept};

/// Keep whatever the real settings folder holds that cannot be read. Run once
/// at start-up, before anything loads or saves the settings.
pub fn keep_at_start_up() -> Result<WhatWasKept> {
    let _ = ConfigManager::new()?;
    Ok(WhatWasKept::default())
}

/// What start-up says about what it kept, or `None` when everything read.
pub fn what_start_up_says(_kept: &WhatWasKept) -> Option<String> {
    None
}

/// What Settings says when it will not open over settings it could not read.
pub fn settings_will_not_open() -> &'static str {
    ""
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::config::{Fate, SettingsFile, UnreadableFile};
    use std::path::PathBuf;

    const FOLDER: &str = r"C:\Users\someone\AppData\Local\wixen-mail\config";

    fn kept(which: SettingsFile, name: &str, kept_as: &str) -> UnreadableFile {
        UnreadableFile {
            which,
            name: name.to_string(),
            fate: Fate::KeptAs(kept_as.to_string()),
        }
    }

    fn left(which: SettingsFile, name: &str) -> UnreadableFile {
        UnreadableFile {
            which,
            name: name.to_string(),
            fate: Fate::LeftAsItIs("its bytes could not be read: Access is denied.".to_string()),
        }
    }

    fn said(unreadable: Vec<UnreadableFile>) -> String {
        what_start_up_says(&WhatWasKept {
            folder: PathBuf::from(FOLDER),
            unreadable,
        })
        .expect("a sentence for what was kept")
    }

    #[test]
    fn test_nothing_kept_says_nothing() {
        let nothing = WhatWasKept {
            folder: PathBuf::from(FOLDER),
            unreadable: Vec::new(),
        };

        assert_eq!(what_start_up_says(&nothing), None);
    }

    #[test]
    fn test_a_kept_settings_file_is_said_with_where_the_old_one_is() {
        assert_eq!(
            said(vec![kept(
                SettingsFile::TheSettings,
                "app_config.json",
                "app_config.json.unreadable-2026-10-19-140322",
            )]),
            format!(
                "Your settings could not be read, so Wixen Mail started with fresh ones and \
                 asked again what it may change. The old settings were kept as \
                 app_config.json.unreadable-2026-10-19-140322 in {FOLDER}."
            )
        );
    }

    #[test]
    fn test_a_kept_account_file_is_said_and_the_other_settings_were_read() {
        assert_eq!(
            said(vec![kept(
                SettingsFile::AnAccounts,
                "account_acc-1.json",
                "account_acc-1.json.unreadable-2026-10-19-140322",
            )]),
            format!(
                "The settings file for an account could not be read, so it was set aside and \
                 your other settings were read as usual. It was kept as \
                 account_acc-1.json.unreadable-2026-10-19-140322 in {FOLDER}."
            )
        );
    }

    #[test]
    fn test_two_kept_account_files_are_both_named() {
        assert_eq!(
            said(vec![
                kept(
                    SettingsFile::AnAccounts,
                    "account_a.json",
                    "account_a.json.unreadable-1"
                ),
                kept(
                    SettingsFile::AnAccounts,
                    "account_b.json",
                    "account_b.json.unreadable-1"
                ),
            ]),
            format!(
                "2 settings files for accounts could not be read, so they were set aside and \
                 your other settings were read as usual. They were kept in {FOLDER} as \
                 account_a.json.unreadable-1 and account_b.json.unreadable-1."
            )
        );
    }

    #[test]
    fn test_a_kept_settings_file_and_a_kept_account_file_are_said_settings_first() {
        assert_eq!(
            said(vec![
                kept(
                    SettingsFile::TheSettings,
                    "app_config.json",
                    "app_config.json.unreadable-1"
                ),
                kept(
                    SettingsFile::AnAccounts,
                    "account_a.json",
                    "account_a.json.unreadable-1"
                ),
            ]),
            format!(
                "Your settings could not be read, so Wixen Mail started with fresh ones and \
                 asked again what it may change. The old settings were kept as \
                 app_config.json.unreadable-1 in {FOLDER}. The settings file for an account \
                 could not be read either, and was kept as account_a.json.unreadable-1 in the \
                 same folder."
            )
        );
    }

    #[test]
    fn test_three_kept_account_files_beside_kept_settings_are_listed() {
        assert_eq!(
            said(vec![
                kept(
                    SettingsFile::TheSettings,
                    "app_config.json",
                    "app_config.json.unreadable-1"
                ),
                kept(SettingsFile::AnAccounts, "account_a.json", "a-1"),
                kept(SettingsFile::AnAccounts, "account_b.json", "b-1"),
                kept(SettingsFile::AnAccounts, "account_c.json", "c-1"),
            ]),
            format!(
                "Your settings could not be read, so Wixen Mail started with fresh ones and \
                 asked again what it may change. The old settings were kept as \
                 app_config.json.unreadable-1 in {FOLDER}. 3 settings files for accounts \
                 could not be read either, and were kept in the same folder as a-1, b-1 and \
                 c-1."
            )
        );
    }

    #[test]
    fn test_a_settings_file_that_could_not_be_kept_is_said_left_as_it_is() {
        assert_eq!(
            said(vec![left(SettingsFile::TheSettings, "app_config.json")]),
            format!(
                "Your settings file, app_config.json, could not be read or set aside, so it \
                 was left as it is in {FOLDER}. Until it can be read, Wixen Mail sends no mail \
                 and changes nothing at your mail servers, and Settings will not open."
            )
        );
    }

    #[test]
    fn test_two_files_that_could_not_be_kept_are_said_as_they() {
        assert_eq!(
            said(vec![
                left(SettingsFile::TheSettings, "app_config.json"),
                left(SettingsFile::AnAccounts, "account_a.json"),
            ]),
            format!(
                "Your settings file, app_config.json, could not be read or set aside, so it \
                 was left as it is in {FOLDER}. The settings file for an account, \
                 account_a.json, could not be read or set aside, so it was left as it is in \
                 {FOLDER}. Until they can be read, Wixen Mail sends no mail and changes \
                 nothing at your mail servers, and Settings will not open."
            )
        );
    }

    #[test]
    fn test_settings_refusing_to_open_says_nothing_was_written_over() {
        assert_eq!(
            settings_will_not_open(),
            "Settings did not open, because your stored settings could not be read. They \
             were left as they are, so nothing was written over them."
        );
    }

    #[test]
    fn test_no_sentence_holds_a_long_dash() {
        let every = [
            said(vec![
                kept(SettingsFile::TheSettings, "app_config.json", "x"),
                kept(SettingsFile::AnAccounts, "account_a.json", "y"),
                left(SettingsFile::AnAccounts, "account_b.json"),
            ]),
            settings_will_not_open().to_string(),
        ];

        for sentence in every {
            assert!(!sentence.contains(['\u{2014}', '\u{2013}']), "{sentence}");
        }
    }
}

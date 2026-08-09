//! Configuration management
//!
//! Handles application settings, account configurations, and persistence.

use crate::common::paths::AppPaths;
use crate::common::{Error, Result, types::Id};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Application-wide configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application version
    pub version: String,
    /// Default folder for downloads
    pub download_folder: PathBuf,
    /// Check for updates on startup
    pub check_updates: bool,
    /// Whether to check links against Google Safe Browsing.
    ///
    /// Off unless somebody turns it on, and inert without an API key. What it
    /// sends is in the setting's own text and in docs/privacy.md: the lists
    /// come down to this machine and links are compared here, so nothing goes
    /// to Google for ordinary mail.
    #[serde(default)]
    pub check_links_with_google: bool,
    /// Whether to read each message here and mark it when it looks wrong.
    ///
    /// On unless somebody turns it off, and the one setting in this file that
    /// starts on while still being about safety. It sends nothing to anybody:
    /// the reading happens on this computer, over text already in hand, and the
    /// most it can do is put a word in the safety column. It is also what mail
    /// arriving over IMAP has had since the body fetch was written, so starting
    /// it off would take a marking away from people who never asked for that.
    ///
    /// Deliberately not the same switch as `check_links_with_google` above.
    /// That one can put four bytes of a link on the wire, so turning this on to
    /// get a check that touches nothing would mean agreeing to something else
    /// entirely.
    #[serde(default = "default_true")]
    pub look_at_message_contents: bool,
    /// Theme name
    pub theme: String,
    /// Font size
    pub font_size: u32,
    /// Enable notifications
    pub enable_notifications: bool,
    /// Log level
    pub log_level: String,
    /// Show preview dialog before sending emails
    #[serde(default = "default_true")]
    pub preview_before_send: bool,
    /// Whether to keep a copy of sent mail here even when the server saved one.
    ///
    /// Off unless somebody asks for it, and off in every settings file written
    /// before it existed. On, the Sent folder lists each message twice once the
    /// server's own copy comes down on the next check for mail: one row filed
    /// here and one from the server. That is what it does rather than a fault,
    /// and both the setting's label and its description say so.
    ///
    /// It has no effect on an account whose mail is collected over POP, which
    /// has no server folder and whose copy is always the one on this computer,
    /// and none on a copy the server refuses, which is kept here regardless.
    #[serde(default)]
    pub keep_sent_mail_on_this_computer: bool,
    /// Whether a message being written opens with the account's signature.
    ///
    /// On unless somebody turns it off, which is what every message has done
    /// since signatures reached the composer, so a settings file written before
    /// this existed keeps the behaviour it had. Off, the composer opens with an
    /// empty body; the signature stays on the account and can still be put in
    /// by hand.
    ///
    /// The compose tab offered this answer for as long as it has existed, hard
    /// set to yes, saved by nothing. A screen reader read out a setting that
    /// was not there.
    #[serde(default = "default_true")]
    pub add_signature_automatically: bool,
    /// The language messages are spell-checked in.
    ///
    /// A BCP 47 tag such as `en-GB` where Windows is doing the checking, and a
    /// bare code such as `en` where the built-in list is. Nothing else reads
    /// this: the interface itself is not translated, and calling it the
    /// interface language, as the settings dialog once did, was a claim.
    #[serde(default = "default_language")]
    pub language: String,
    /// Whether to check the spelling of a message before sending it.
    ///
    /// On by default. Somebody who does not want it can say so, and the
    /// question is then never asked, which is different from asking and being
    /// dismissed every time.
    #[serde(default = "default_true")]
    pub check_spelling_before_send: bool,
    /// What Wixen Mail may change at a server, for every account.
    ///
    /// Defaults to `default_allowed` below, which names the answer instead of
    /// saying it again. A config file written before this existed gets the same
    /// thing, which is the point: an upgrade should take permissions away
    /// rather than carry on sending from code that has never been proved.
    #[serde(default = "default_allowed")]
    pub allowed_changes: crate::application::allowed::Allowed,
    /// What one account may change, when it differs from the setting above.
    ///
    /// Kept here, keyed by account id, rather than as a field on `Account`.
    /// `Account` is built in eleven places, so a field on it is eleven chances
    /// to write the permissive answer by accident; a map that answers with the
    /// application-wide setting for an id it has never seen cannot be got
    /// wrong that way.
    ///
    /// Nothing writes it. It is read by `allowed_for` below and honoured all
    /// the way out to the provider clients, and the settings screen writes the
    /// application-wide answer only, so every account gets that one. The gap is
    /// written down here rather than left to be rediscovered: the testing page
    /// and the first-run screen both used to offer this as a control somebody
    /// could reach, and neither the screen nor the sentence a sync says names
    /// an account any more. Wiring it means a control per account on the
    /// settings screen, and the answer it writes can only ever narrow the
    /// application-wide one.
    #[serde(default)]
    pub allowed_per_account: HashMap<String, crate::application::allowed::Allowed>,
    /// Whether a change to a contact goes to every address book that has that
    /// contact, or only to the one it came from.
    ///
    /// Not a permission and it cannot open the write gate: whether anything is
    /// written at all is decided by `allowed_changes` above, and this only
    /// decides how many of the already-allowed address books an edit reaches.
    /// It is on unless somebody turns it off, because a contact in two address
    /// books is one person, and quietly correcting a phone number in one of
    /// them leaves the other wrong with nothing to say so.
    #[serde(default = "default_true")]
    pub send_contact_changes_everywhere: bool,
    /// The folder each account's last move or copy went to, by account id.
    ///
    /// Filing mail is repetitive: several messages go into the same folder one
    /// after another. Holding the last one lets the destination window open on
    /// it, so filing the next is the shortcut and Enter rather than another
    /// walk through the folder tree.
    ///
    /// Per account, because two accounts have different folders and offering
    /// one account's to the other is offering somewhere the message cannot go.
    #[serde(default)]
    pub last_filed_into: HashMap<String, String>,
    /// Whether to tell a sender when you have read their message.
    ///
    /// Stored as a word rather than as the enum, so a settings file written by
    /// a later version, or edited by hand, reads back as the private answer
    /// instead of failing to parse. `crate::application::receipts::Policy`
    /// decides what each word means, and anything it does not know is "never".
    #[serde(default)]
    pub read_receipts: String,
    /// Which surface a message opens into, formatted or plain text.
    ///
    /// Stored as a word for the same reason as the others: a settings file
    /// written by a later version reads back as the default rather than
    /// failing to parse, and the default here keeps what the sender wrote.
    #[serde(default)]
    pub read_messages_as: String,
    /// Whether the person has been shown what this alpha can and cannot do.
    ///
    /// False on a fresh installation and on an upgrade from before this
    /// existed, so somebody who has been using it already still gets told once
    /// that writing has been switched on and has never been tried for real.
    #[serde(default)]
    pub told_about_the_alpha: bool,
    /// Whether misspellings are marked in the editor as you write.
    ///
    /// One setting, two things, deliberately. It turns on the engine's own
    /// marking, which is what a screen reader reads as the caret crosses a
    /// word, and the sound at the end of a word that is wrong. They are the
    /// same question asked twice, and asking it twice is how people end up
    /// with one on and the other off and no idea why.
    #[serde(default = "default_true")]
    pub check_spelling_as_you_type: bool,
    /// Default sort order for message list
    #[serde(default = "default_sort_order")]
    pub default_sort_order: String,
    /// How dates are shown in lists: "absolute" or "relative".
    ///
    /// Relative says "2 days ago" within the last week, which is three
    /// syllables where a full date is a dozen, and answers the question most
    /// people are actually asking of a date in a mail list.
    #[serde(default = "default_date_style")]
    pub date_style: String,
    /// Day and month order: "auto", "month_first", or "day_first".
    #[serde(default = "default_date_order")]
    pub date_order: String,
    /// How long a message is looked at before it counts as read.
    ///
    /// "immediately", "never", or a number of seconds. The settings window has
    /// offered this since it was written and nothing ever read it back, so the
    /// answer was always immediately, whatever it said on screen.
    #[serde(default = "default_mark_read_after")]
    pub mark_read_after: String,
    /// Whether the Cc and Bcc lines are in the compose window from the start:
    /// "shown" or "hidden".
    #[serde(default = "default_copy_lines")]
    pub copy_lines: String,
    /// The hour the working day starts, 0 to 23.
    ///
    /// Read by the calendar, which says so when an event falls outside it.
    #[serde(default = "default_day_starts")]
    pub working_day_starts: u8,
    /// The hour the working day ends, 1 to 24. Exclusive: 17 means five.
    #[serde(default = "default_day_ends")]
    pub working_day_ends: u8,
    /// Whether the month is a word or a number: "verbal" or "numeric".
    ///
    /// Spelled out is easier to hear and longer to read, and which of those
    /// matters depends on the person and on whether they are listening or
    /// looking at it.
    #[serde(default = "default_date_wording")]
    pub date_wording: String,
    /// Whether the clock runs to twelve or twenty-four: "auto", "12", or "24".
    ///
    /// "auto" follows the machine, so nothing has to be set here to get the
    /// clock the rest of the computer already keeps.
    #[serde(default = "default_clock_hours")]
    pub clock_hours: String,
    /// Whether message text is read aloud.
    ///
    /// Kept across restarts because someone who works in a shared room needs
    /// mail to stay quiet without switching it off again every session.
    #[serde(default)]
    pub mute_message_reading: bool,
    /// Which message list columns are shown, in what order, and the sort.
    ///
    /// Stored as the compact form `ColumnLayout::to_stored` writes. Empty means
    /// nothing has been chosen yet and the per-folder defaults apply. Anyone
    /// who arranges a list they navigate by ear has spent real effort on it,
    /// and losing that on restart is not a small annoyance.
    #[serde(default)]
    pub message_columns: String,
    /// Which channels each event reaches: speech, braille, earcon, visual.
    ///
    /// Stored as the compact form `FeedbackSettings::to_stored` writes. Empty
    /// means the defaults: words on, sounds off. This is the setting that lets
    /// a deaf-blind user drop speech and keep braille, or someone in an open
    /// office swap a sentence of speech for a short tone.
    #[serde(default)]
    pub feedback_channels: String,
    /// Which account new items are created in.
    ///
    /// Set to the first account configured, without asking, because for most
    /// people it is the only one. Empty means nothing has been chosen yet, and
    /// the first account added takes it.
    #[serde(default)]
    pub default_account_id: String,
    /// How often a draft is saved while composing, in minutes.
    ///
    /// Nought means never. See `application::autosave` for why the range stops
    /// at ten.
    #[serde(default = "default_autosave_minutes")]
    pub draft_autosave_minutes: u32,
    /// Calendar default view: "agenda", "day", "week", "month"
    #[serde(default = "default_calendar_view")]
    pub calendar_default_view: String,
    /// Show weekends in calendar views
    #[serde(default = "default_true")]
    pub calendar_show_weekends: bool,
    /// First day of week: 0 = Sunday, 1 = Monday
    #[serde(default)]
    pub calendar_first_day_of_week: u8,
    /// Default reminder lead-time in minutes (e.g. 15 = remind 15 min before)
    #[serde(default = "default_reminder_minutes")]
    pub default_reminder_minutes: u32,
}

/// What a new or upgraded installation may change.
///
/// `Allowed::FOR_TESTING` holds the answer and the reason for it. This names it
/// rather than saying it a second time, and that is the whole point: a sentence
/// repeating a fact the code already holds goes stale on its own, and nothing
/// here could have stopped it. The check in `tests/house_style.rs` reads prose
/// that names the setting the way the settings screen does, and no line in this
/// file does, so a sentence here claiming the opposite of the truth was
/// measured passing every test in the project.
///
/// Ten of those sentences have been written wrong in this repository. A
/// sentence a check reads may state the answer, because it will be held to it.
/// A sentence no check reads should name the answer instead.
fn default_allowed() -> crate::application::allowed::Allowed {
    crate::application::allowed::Allowed::FOR_TESTING
}

fn default_true() -> bool {
    true
}
/// The language messages are checked in, when nothing has been chosen.
///
/// This machine's own, when something here can check it. It was "en" for
/// everybody, so anybody writing in anything else had every word of it called a
/// mistake until they found the setting, and finding a setting by hearing every
/// word marked wrong is not finding it.
///
/// English when the machine's language is one nothing can check, because
/// English checked is better than nothing checked.
fn default_language() -> String {
    crate::service::spellcheck::language_of_this_machine().unwrap_or_else(|| "en".to_string())
}
fn default_sort_order() -> String {
    "date_newest".to_string()
}
fn default_date_style() -> String {
    "relative".to_string()
}

fn default_mark_read_after() -> String {
    crate::application::reading_habits::MarkRead::default().as_stored()
}

fn default_copy_lines() -> String {
    crate::application::reading_habits::CopyLines::default()
        .as_stored()
        .to_string()
}

fn default_day_starts() -> u8 {
    crate::application::reading_habits::WorkingDay::default().starts
}

fn default_day_ends() -> u8 {
    crate::application::reading_habits::WorkingDay::default().ends
}

fn default_date_wording() -> String {
    "verbal".to_string()
}

fn default_clock_hours() -> String {
    "auto".to_string()
}

fn default_date_order() -> String {
    "auto".to_string()
}

fn default_calendar_view() -> String {
    "agenda".to_string()
}
fn default_reminder_minutes() -> u32 {
    15
}

fn default_autosave_minutes() -> u32 {
    crate::application::autosave::AutosaveInterval::default().minutes()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            download_folder: dirs::download_dir().unwrap_or_else(|| PathBuf::from(".")),
            check_updates: true,
            check_links_with_google: false,
            look_at_message_contents: default_true(),
            theme: "default".to_string(),
            font_size: 12,
            date_style: default_date_style(),
            date_order: default_date_order(),
            date_wording: default_date_wording(),
            mark_read_after: default_mark_read_after(),
            copy_lines: default_copy_lines(),
            working_day_starts: default_day_starts(),
            working_day_ends: default_day_ends(),
            clock_hours: default_clock_hours(),
            mute_message_reading: false,
            default_account_id: String::new(),
            draft_autosave_minutes: default_autosave_minutes(),
            message_columns: String::new(),
            feedback_channels: String::new(),
            enable_notifications: true,
            log_level: "info".to_string(),
            preview_before_send: true,
            // The safe answer is the one that changes nothing: the server's own
            // copy is what Sent has always listed.
            keep_sent_mail_on_this_computer: false,
            add_signature_automatically: default_true(),
            language: default_language(),
            check_spelling_before_send: true,
            allowed_changes: default_allowed(),
            allowed_per_account: HashMap::new(),
            send_contact_changes_everywhere: default_true(),
            last_filed_into: HashMap::new(),
            read_receipts: crate::application::receipts::Policy::Never
                .as_str()
                .to_string(),
            read_messages_as: crate::application::reading_style::Style::Formatted
                .as_str()
                .to_string(),
            told_about_the_alpha: false,
            check_spelling_as_you_type: default_true(),
            default_sort_order: default_sort_order(),
            calendar_default_view: default_calendar_view(),
            calendar_show_weekends: default_true(),
            calendar_first_day_of_week: 0,
            default_reminder_minutes: default_reminder_minutes(),
        }
    }
}

impl AppConfig {
    /// What this account may change, before the command line narrows it.
    ///
    /// The account's own answer when it has one, and the application-wide
    /// setting otherwise, with the application-wide setting applied either
    /// way. So a per-account entry can only ever be narrower, never wider:
    /// somebody who turns everything off globally has turned it off, whatever
    /// any account says.
    pub fn allowed_for(&self, account_id: &str) -> crate::application::allowed::Allowed {
        self.allowed_per_account
            .get(account_id)
            .copied()
            .unwrap_or(self.allowed_changes)
            .and(self.allowed_changes)
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        if self.font_size < 8 || self.font_size > 72 {
            return Err(Error::Config(
                "Font size must be between 8 and 72".to_string(),
            ));
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(Error::Config(format!(
                "Invalid log level: {}. Must be one of: error, warn, info, debug, trace",
                self.log_level
            )));
        }

        Ok(())
    }
}

/// Account-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    /// Account ID
    pub id: Id,
    /// Account name
    pub name: String,
    /// Check interval in minutes
    pub check_interval_minutes: u32,
    /// Signature text
    pub signature: Option<String>,
    /// Default folder
    pub default_folder: String,
    /// Auto-download attachments
    pub auto_download_attachments: bool,
}

impl AccountConfig {
    /// Create a new account configuration
    pub fn new(id: Id, name: String) -> Self {
        Self {
            id,
            name,
            check_interval_minutes: 15,
            signature: None,
            default_folder: "INBOX".to_string(),
            auto_download_attachments: false,
        }
    }

    /// Validate account configuration
    pub fn validate(&self) -> Result<()> {
        if self.check_interval_minutes < 1 || self.check_interval_minutes > 1440 {
            return Err(Error::Config(
                "Check interval must be between 1 and 1440 minutes".to_string(),
            ));
        }

        Ok(())
    }
}

/// Configuration manager with file persistence
pub struct ConfigManager {
    /// Application configuration
    app_config: AppConfig,
    /// Account configurations
    account_configs: HashMap<Id, AccountConfig>,
    /// Configuration directory
    config_dir: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Result<Self> {
        Self::in_dir(AppPaths::resolve()?.config_dir())
    }

    /// Open the settings folder and read what is in it.
    ///
    /// Most callers want this rather than [`new`](Self::new): they need the
    /// stored values, and both steps fail the same way from their point of
    /// view.
    pub fn load_stored() -> Result<Self> {
        let mut manager = Self::new()?;
        manager.load()?;
        Ok(manager)
    }

    /// Keep configuration in a directory of the caller's choosing.
    ///
    /// Tests use this so they never write into the profile of whoever is
    /// running them.
    fn in_dir(config_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&config_dir).map_err(|e| {
            Error::Config(format!("Could not create {}: {e}", config_dir.display()))
        })?;

        Ok(Self {
            app_config: AppConfig::default(),
            account_configs: HashMap::new(),
            config_dir,
        })
    }

    /// Get app config file path
    fn app_config_path(&self) -> PathBuf {
        self.config_dir.join("app_config.json")
    }

    /// Get account config file path
    fn account_config_path(&self, account_id: &str) -> PathBuf {
        self.config_dir.join(format!("account_{}.json", account_id))
    }

    /// Load configuration from file
    pub fn load(&mut self) -> Result<()> {
        // Load app config
        let app_config_path = self.app_config_path();
        if app_config_path.exists() {
            let content = fs::read_to_string(&app_config_path)
                .map_err(|e| Error::Config(format!("Failed to read app config: {}", e)))?;
            self.app_config = serde_json::from_str(&content)
                .map_err(|e| Error::Config(format!("Failed to parse app config: {}", e)))?;
            self.app_config.validate()?;
        } else {
            // Create default config file
            self.save_app_config()?;
        }

        // Load account configs
        for entry in fs::read_dir(&self.config_dir)
            .map_err(|e| Error::Config(format!("Failed to read config directory: {}", e)))?
        {
            let entry = entry
                .map_err(|e| Error::Config(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy();
                if filename_str.starts_with("account_") && filename_str.ends_with(".json") {
                    let content = fs::read_to_string(&path).map_err(|e| {
                        Error::Config(format!("Failed to read account config: {}", e))
                    })?;
                    let account_config: AccountConfig =
                        serde_json::from_str(&content).map_err(|e| {
                            Error::Config(format!("Failed to parse account config: {}", e))
                        })?;
                    account_config.validate()?;
                    self.account_configs
                        .insert(account_config.id.clone(), account_config);
                }
            }
        }

        Ok(())
    }

    /// Save app configuration to file
    fn save_app_config(&self) -> Result<()> {
        self.app_config.validate()?;
        let content = serde_json::to_string_pretty(&self.app_config)
            .map_err(|e| Error::Config(format!("Failed to serialize app config: {}", e)))?;
        fs::write(self.app_config_path(), content)
            .map_err(|e| Error::Config(format!("Failed to write app config: {}", e)))?;
        Ok(())
    }

    /// Save all configurations to files
    pub fn save(&self) -> Result<()> {
        // Save app config
        self.save_app_config()?;

        // Save account configs
        for account_config in self.account_configs.values() {
            account_config.validate()?;
            let content = serde_json::to_string_pretty(account_config)
                .map_err(|e| Error::Config(format!("Failed to serialize account config: {}", e)))?;
            let path = self.account_config_path(&account_config.id);
            fs::write(path, content)
                .map_err(|e| Error::Config(format!("Failed to write account config: {}", e)))?;
        }

        Ok(())
    }

    /// Get application configuration
    pub fn app_config(&self) -> &AppConfig {
        &self.app_config
    }

    /// Get mutable application configuration
    pub fn app_config_mut(&mut self) -> &mut AppConfig {
        &mut self.app_config
    }

    /// Get account configuration
    pub fn get_account_config(&self, account_id: &str) -> Option<&AccountConfig> {
        self.account_configs.get(account_id)
    }

    /// Add or update account configuration
    pub fn set_account_config(&mut self, account_config: AccountConfig) -> Result<()> {
        account_config.validate()?;
        self.account_configs
            .insert(account_config.id.clone(), account_config);
        Ok(())
    }

    /// Remove account configuration
    pub fn remove_account_config(&mut self, account_id: &str) -> Result<()> {
        self.account_configs.remove(account_id);
        let path = self.account_config_path(account_id);
        if path.exists() {
            fs::remove_file(path).map_err(|e| {
                Error::Config(format!("Failed to remove account config file: {}", e))
            })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_chosen_in_one_session_are_there_in_the_next() {
        // Nothing covered the round trip through the folder, so reading the
        // settings back could have done nothing at all and every test still
        // passed. What that looks like is every setting somebody chose
        // reverting on restart, silently, including what this build is allowed
        // to change at a server.
        let dir = tempfile::TempDir::new().expect("a temporary folder");
        let mut first =
            ConfigManager::in_dir(dir.path().join("config")).expect("a settings folder");
        first.app_config_mut().date_wording = "numeric".to_string();
        first.app_config_mut().working_day_starts = 6;
        first.app_config_mut().allowed_changes = crate::application::allowed::Allowed::NOTHING;
        first.save().expect("the settings to be written");

        let mut later =
            ConfigManager::in_dir(dir.path().join("config")).expect("the same settings folder");
        later.load().expect("the settings to be read back");

        assert_eq!(later.app_config().date_wording, "numeric");
        assert_eq!(later.app_config().working_day_starts, 6);
        assert_eq!(
            later.app_config().allowed_changes,
            crate::application::allowed::Allowed::NOTHING,
            "an upgrade would have handed back permissions somebody took away"
        );
    }

    #[test]
    fn test_an_account_s_settings_come_back_from_its_own_file() {
        let dir = tempfile::TempDir::new().expect("a temporary folder");
        let mut first =
            ConfigManager::in_dir(dir.path().join("config")).expect("a settings folder");
        let mut account = AccountConfig::new("acc-1".to_string(), "Work".to_string());
        account.check_interval_minutes = 30;
        account.default_folder = "Archive".to_string();
        account.signature = Some("Sent from Wixen Mail".to_string());
        first
            .set_account_config(account)
            .expect("the account to be accepted");
        first.save().expect("the settings to be written");

        let mut later =
            ConfigManager::in_dir(dir.path().join("config")).expect("the same settings folder");
        later.load().expect("the settings to be read back");

        let read_back = later
            .get_account_config("acc-1")
            .expect("the account to have been read back");
        assert_eq!(read_back.name, "Work");
        assert_eq!(read_back.check_interval_minutes, 30);
        assert_eq!(read_back.default_folder, "Archive");
        assert_eq!(read_back.signature.as_deref(), Some("Sent from Wixen Mail"));
    }

    #[test]
    fn test_only_the_account_files_are_read_as_accounts() {
        // The second time this shape has come up: `is_settings_file` in
        // `common` had the same two halves never checked apart. With `or` in
        // place of `and`, every .json file in the folder is opened as an
        // account, and one that does not parse stops the settings loading at
        // all, which is somebody's whole configuration gone on startup.
        let dir = tempfile::TempDir::new().expect("a temporary folder");
        let folder = dir.path().join("config");
        let mut manager = ConfigManager::in_dir(folder.clone()).expect("a settings folder");
        manager
            .set_account_config(a_valid_account_config())
            .expect("the account to be accepted");
        manager.save().expect("the settings to be written");

        // Neither of these is an account file, and each is one half of the
        // test the loader makes.
        fs::write(folder.join("notes.json"), "{\"not\": \"an account\"}")
            .expect("a stray json file");
        fs::write(folder.join("account_notes.txt"), "not json at all")
            .expect("a stray account-looking file");

        let mut later = ConfigManager::in_dir(folder).expect("the same settings folder");
        later
            .load()
            .expect("a stray file in the folder stopped the settings loading");

        assert!(later.get_account_config("acc-1").is_some());
        assert!(later.get_account_config("notes").is_none());
        assert!(later.get_account_config("account_notes").is_none());
    }

    #[test]
    fn test_removing_an_account_takes_its_file_with_it() {
        // A file left behind is an account that comes back on the next start,
        // after somebody has deleted it.
        let dir = tempfile::TempDir::new().expect("a temporary folder");
        let folder = dir.path().join("config");
        let mut manager = ConfigManager::in_dir(folder.clone()).expect("a settings folder");
        manager
            .set_account_config(a_valid_account_config())
            .expect("the account to be accepted");
        manager.save().expect("the settings to be written");
        assert!(folder.join("account_acc-1.json").exists());

        manager
            .remove_account_config("acc-1")
            .expect("the account to be removed");

        assert!(
            !folder.join("account_acc-1.json").exists(),
            "the account's file is still in the settings folder"
        );
        let mut later = ConfigManager::in_dir(folder).expect("the same settings folder");
        later.load().expect("the settings to be read back");
        assert!(
            later.get_account_config("acc-1").is_none(),
            "the deleted account came back on the next start"
        );
    }

    fn a_valid_account_config() -> AccountConfig {
        AccountConfig::new("acc-1".to_string(), "Work".to_string())
    }

    #[test]
    fn test_config_manager() {
        let dir = tempfile::TempDir::new().unwrap();
        let manager = ConfigManager::in_dir(dir.path().join("config"));
        assert!(manager.is_ok());
    }

    #[test]
    fn test_settings_are_written_to_the_folder_they_were_given() {
        let dir = tempfile::TempDir::new().unwrap();
        let config_dir = dir.path().join("config");
        let manager = ConfigManager::in_dir(config_dir.clone()).unwrap();

        manager.save().unwrap();

        assert!(config_dir.join("app_config.json").is_file());
    }

    #[test]
    fn test_the_settings_folder_is_created_if_it_is_not_there() {
        // A first run has an empty application data folder, and the manager is
        // built before anything asks it to save.
        let dir = tempfile::TempDir::new().unwrap();
        let config_dir = dir.path().join("missing").join("config");

        ConfigManager::in_dir(config_dir.clone()).unwrap();

        assert!(config_dir.is_dir());
    }

    #[test]
    fn test_app_config_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.theme, "default");
        assert_eq!(config.font_size, 12);
        assert!(config.enable_notifications);
    }

    #[test]
    fn test_app_config_validation() {
        let mut config = AppConfig::default();
        assert!(config.validate().is_ok());

        config.font_size = 150;
        assert!(config.validate().is_err());

        config.font_size = 12;
        config.log_level = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_account_config() {
        let config = AccountConfig::new("acc-1".to_string(), "Test Account".to_string());
        assert_eq!(config.id, "acc-1");
        assert_eq!(config.name, "Test Account");
        assert_eq!(config.check_interval_minutes, 15);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_account_config_validation() {
        let mut config = AccountConfig::new("acc-1".to_string(), "Test".to_string());
        assert!(config.validate().is_ok());

        config.check_interval_minutes = 0;
        assert!(config.validate().is_err());

        config.check_interval_minutes = 2000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_manager_account_config() {
        let mut manager = ConfigManager::new().unwrap();
        let account_config = AccountConfig::new("acc-1".to_string(), "Test".to_string());

        manager.set_account_config(account_config.clone()).unwrap();
        let retrieved = manager.get_account_config("acc-1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test");
    }

    #[test]
    fn test_app_config_defaults_complete() {
        let config = AppConfig::default();
        // Not "en". The language follows this machine, so asserting English
        // only passes on an English machine, and it passed here while a fresh
        // installation was quietly hardcoding "en" for everybody.
        assert!(
            config.language.len() >= 2
                && config
                    .language
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-'),
            "{} is not a language the spell checker could be asked for",
            config.language
        );
        assert_eq!(config.default_sort_order, "date_newest");
        assert_eq!(config.calendar_default_view, "agenda");
        assert!(config.calendar_show_weekends);
        assert_eq!(config.calendar_first_day_of_week, 0);
        assert_eq!(config.default_reminder_minutes, 15);
        assert!(config.preview_before_send);
        assert_eq!(config.log_level, "info");
        assert!(config.check_updates);
    }

    #[test]
    fn test_app_config_validates_valid_log_levels() {
        let valid = ["error", "warn", "info", "debug", "trace"];
        for level in &valid {
            let config = AppConfig {
                log_level: level.to_string(),
                ..Default::default()
            };
            assert!(config.validate().is_ok(), "Expected {} to be valid", level);
        }
    }

    #[test]
    fn test_app_config_font_size_boundaries() {
        let cases = [(8, true), (72, true), (7, false), (73, false)];

        for (font_size, expect_valid) in cases {
            let config = AppConfig {
                font_size,
                ..Default::default()
            };
            assert_eq!(
                config.validate().is_ok(),
                expect_valid,
                "Expected font size {} validity to be {}",
                font_size,
                expect_valid
            );
        }
    }

    #[test]
    fn test_account_config_check_interval_boundaries() {
        let mut config = AccountConfig::new("a".to_string(), "A".to_string());

        config.check_interval_minutes = 1;
        assert!(config.validate().is_ok());

        config.check_interval_minutes = 1440;
        assert!(config.validate().is_ok());

        config.check_interval_minutes = 0;
        assert!(config.validate().is_err());

        config.check_interval_minutes = 1441;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_app_config_serialization_round_trip() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.theme, config.theme);
        assert_eq!(deserialized.font_size, config.font_size);
        assert_eq!(deserialized.language, config.language);
        assert_eq!(
            deserialized.calendar_default_view,
            config.calendar_default_view
        );
    }
}

#[cfg(test)]
mod permission_tests {
    use super::*;
    use crate::application::allowed::Allowed;

    fn config() -> AppConfig {
        AppConfig::default()
    }

    #[test]
    fn test_a_new_installation_can_change_tasks_and_not_mail() {
        // What an alpha tester gets without touching anything.
        assert_eq!(config().allowed_for("any-account"), Allowed::FOR_TESTING);
    }

    #[test]
    fn test_an_account_nobody_has_set_follows_the_application_wide_setting() {
        let mut settings = config();
        settings.allowed_changes = Allowed::EVERYTHING;

        assert_eq!(settings.allowed_for("never-seen"), Allowed::EVERYTHING);
    }

    #[test]
    fn test_an_account_can_be_narrower_than_the_setting() {
        // The case the whole thing is for: everything allowed generally, and
        // one real account marked read-only while it is being tested against.
        let mut settings = config();
        settings.allowed_changes = Allowed::EVERYTHING;
        settings
            .allowed_per_account
            .insert("my-real-mail".to_string(), Allowed::NOTHING);

        assert_eq!(settings.allowed_for("my-real-mail"), Allowed::NOTHING);
        assert_eq!(settings.allowed_for("a-throwaway"), Allowed::EVERYTHING);
    }

    #[test]
    fn test_an_account_cannot_be_wider_than_the_setting() {
        // Turning everything off has to mean off. An account entry left over
        // from before must not quietly put it back.
        let mut settings = config();
        settings.allowed_changes = Allowed::NOTHING;
        settings
            .allowed_per_account
            .insert("eager".to_string(), Allowed::EVERYTHING);

        assert_eq!(settings.allowed_for("eager"), Allowed::NOTHING);
    }

    #[test]
    fn test_a_settings_file_written_before_these_existed_reads_the_way_it_should() {
        // Every settings file on disk predates most of these, so what they
        // fall back to is what an upgrade gets. Several of them decide how a
        // date or a time is spoken, and a wrong one is not an error anybody
        // sees: it is every date read the wrong way round, with nothing on
        // screen to say why.
        //
        // Built by taking the fields back out of a current config rather than
        // hand-writing the older shape, so it does not break the next time an
        // unrelated field is added.
        let mut older = serde_json::to_value(AppConfig::default()).expect("a config to serialise");
        let fields = older.as_object_mut().expect("an object");
        for gone in [
            "check_spelling_as_you_type",
            "default_sort_order",
            "date_style",
            "date_order",
            "mark_read_after",
            "copy_lines",
            "working_day_starts",
            "working_day_ends",
            "date_wording",
            "clock_hours",
            "draft_autosave_minutes",
            "calendar_default_view",
            "calendar_show_weekends",
            "default_reminder_minutes",
        ] {
            assert!(
                fields.remove(gone).is_some(),
                "{gone} is not written to the settings file any more, so this test covers nothing"
            );
        }

        let parsed: AppConfig =
            serde_json::from_value(older).expect("an older settings file still opens");

        assert_eq!(parsed.date_style, "relative");
        assert_eq!(parsed.date_order, "auto");
        assert_eq!(parsed.date_wording, "verbal");
        assert_eq!(parsed.clock_hours, "auto");
        assert_eq!(parsed.default_sort_order, "date_newest");
        assert_eq!(parsed.calendar_default_view, "agenda");
        assert_eq!(parsed.default_reminder_minutes, 15);
        assert!(
            parsed.check_spelling_as_you_type,
            "spelling would stop being checked for everybody upgrading"
        );
        assert!(
            parsed.calendar_show_weekends,
            "weekends would disappear from the calendar"
        );

        // These belong to the module that owns the setting. What matters here
        // is that the field falls back to it rather than to nothing.
        assert_eq!(
            parsed.mark_read_after,
            crate::application::reading_habits::MarkRead::default().as_stored()
        );
        assert_eq!(
            parsed.copy_lines,
            crate::application::reading_habits::CopyLines::default().as_stored()
        );
        assert_eq!(
            parsed.working_day_starts,
            crate::application::reading_habits::WorkingDay::default().starts
        );
        assert_eq!(
            parsed.working_day_ends,
            crate::application::reading_habits::WorkingDay::default().ends
        );
        assert_eq!(
            parsed.draft_autosave_minutes,
            crate::application::autosave::AutosaveInterval::default().minutes()
        );
        assert!(
            parsed.working_day_starts < parsed.working_day_ends,
            "the working day ends before it starts"
        );
    }

    #[test]
    fn test_a_fresh_installation_checks_spelling_in_this_machine_s_language() {
        // A settings file comes into being two ways: this default for a fresh
        // installation, and serde's per-field defaults for a file written
        // before a field existed. They were written out twice and drifted.
        //
        // The language is where it showed. A fresh installation got "en"
        // whatever the machine was set to, so anybody writing in another
        // language had every word of it called a mistake until they found the
        // setting, and finding a setting by hearing every word marked wrong is
        // not finding it. An upgraded file already followed the machine.
        assert_eq!(
            AppConfig::default().language,
            crate::service::spellcheck::language_of_this_machine()
                .unwrap_or_else(|| "en".to_string()),
            "a fresh installation does not check spelling in this machine's language"
        );
    }

    #[test]
    fn test_a_config_written_before_this_existed_reads_as_the_safe_answer() {
        // An upgrade takes permissions away rather than granting them, and a
        // file with no such field at all still parses.
        // Built by taking the two new fields back out of a current one, rather
        // than hand-writing the older shape. A literal would have to list
        // every unrelated field and would break the next time one is added,
        // which is not what this test is about.
        let mut older = serde_json::to_value(AppConfig::default()).expect("a config to serialise");
        let fields = older.as_object_mut().expect("an object");
        fields.remove("allowed_changes");
        fields.remove("allowed_per_account");

        let parsed: AppConfig = serde_json::from_value(older).expect("an older config still opens");

        assert_eq!(parsed.allowed_changes, Allowed::FOR_TESTING);
        assert!(parsed.allowed_per_account.is_empty());
    }

    #[test]
    fn test_a_setting_nobody_has_touched_sends_a_change_to_every_address_book_that_has_it() {
        assert!(AppConfig::default().send_contact_changes_everywhere);
    }

    #[test]
    fn test_a_settings_file_written_before_this_setting_existed_still_sends_to_all_of_them() {
        let mut older = serde_json::to_value(AppConfig::default()).expect("a config to serialise");
        let fields = older.as_object_mut().expect("an object");
        assert!(
            fields.remove("send_contact_changes_everywhere").is_some(),
            "the setting is not written to the settings file, so this test covers nothing"
        );

        let parsed: AppConfig = serde_json::from_value(older).expect("an older config still opens");

        assert!(parsed.send_contact_changes_everywhere);
    }

    #[test]
    fn test_a_setting_nobody_has_touched_looks_at_the_message_itself() {
        // The only new setting here that starts on. It sends nothing anywhere,
        // it only adds a warning, and it is what mail arriving over IMAP has
        // had since the body fetch was written: starting it off would quietly
        // take that away from people who never asked.
        assert!(AppConfig::default().look_at_message_contents);
    }

    #[test]
    fn test_a_settings_file_written_before_this_setting_existed_still_looks_at_the_message() {
        let mut older = serde_json::to_value(AppConfig::default()).expect("a config to serialise");
        let fields = older.as_object_mut().expect("an object");
        assert!(
            fields.remove("look_at_message_contents").is_some(),
            "the setting is not written to the settings file, so this test covers nothing"
        );

        let parsed: AppConfig = serde_json::from_value(older).expect("an older config still opens");

        assert!(parsed.look_at_message_contents);
    }

    #[test]
    fn test_a_settings_file_written_before_this_existed_keeps_no_extra_copy() {
        // Off on a fresh install and off in a settings file that predates it.
        // Turning it on for somebody who never asked would double every row in
        // their Sent folder, which reads as the folder having gone wrong.
        assert!(!AppConfig::default().keep_sent_mail_on_this_computer);

        let mut older = serde_json::to_value(AppConfig::default()).expect("a config to serialise");
        let fields = older.as_object_mut().expect("an object");
        assert!(
            fields.remove("keep_sent_mail_on_this_computer").is_some(),
            "the setting is not written to the settings file, so this test covers nothing"
        );

        let parsed: AppConfig = serde_json::from_value(older).expect("an older config still opens");

        assert!(!parsed.keep_sent_mail_on_this_computer);
    }

    #[test]
    fn test_keeping_a_copy_here_survives_a_restart() {
        // A setting that reads back as the default after a restart is a setting
        // nobody can turn on, and the only sign is sent mail quietly not being
        // where they put it.
        let dir = tempfile::tempdir().expect("a temporary folder");
        {
            let mut manager =
                ConfigManager::in_dir(dir.path().to_path_buf()).expect("a config folder");
            manager.app_config_mut().keep_sent_mail_on_this_computer = true;
            manager.save().expect("the settings to save");
        }

        let mut reopened =
            ConfigManager::in_dir(dir.path().to_path_buf()).expect("the settings to open again");
        reopened.load().expect("the settings to read back");

        assert!(reopened.app_config().keep_sent_mail_on_this_computer);
    }

    #[test]
    fn test_asking_google_about_links_is_still_off_unless_somebody_asks_for_it() {
        // The other half of the pair, kept apart deliberately: one reads the
        // message here and sends nothing, the other can put four bytes of a
        // link on the wire. They cannot share a switch.
        assert!(!AppConfig::default().check_links_with_google);
    }
}

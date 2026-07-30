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
    /// Defaults to tasks, contacts and calendar but not mail. A config file
    /// written before this existed gets that too, which is the point: an
    /// upgrade should take permissions away rather than carry on sending from
    /// code that has never been proved.
    #[serde(default = "default_allowed")]
    pub allowed_changes: crate::application::allowed::Allowed,
    /// What one account may change, when it differs from the setting above.
    ///
    /// Kept here, keyed by account id, rather than as a field on `Account`.
    /// `Account` is built in eleven places, so a field on it is eleven chances
    /// to write the permissive answer by accident; a map that answers with the
    /// application-wide setting for an id it has never seen cannot be got
    /// wrong that way.
    #[serde(default)]
    pub allowed_per_account: HashMap<String, crate::application::allowed::Allowed>,
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
/// Tasks, contacts and the calendar, but not mail. Somebody can point this at
/// their real account and use it all day, and the worst that happens is a task
/// in the wrong place. Sending is the deliberate step afterwards.
fn default_allowed() -> crate::application::allowed::Allowed {
    crate::application::allowed::Allowed::FOR_TESTING
}

fn default_true() -> bool {
    true
}
fn default_language() -> String {
    "en".to_string()
}
fn default_sort_order() -> String {
    "date_newest".to_string()
}
fn default_date_style() -> String {
    "relative".to_string()
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
            theme: "default".to_string(),
            font_size: 12,
            date_style: default_date_style(),
            date_order: default_date_order(),
            mute_message_reading: false,
            default_account_id: String::new(),
            draft_autosave_minutes: default_autosave_minutes(),
            message_columns: String::new(),
            feedback_channels: String::new(),
            enable_notifications: true,
            log_level: "info".to_string(),
            preview_before_send: true,
            language: "en".to_string(),
            check_spelling_before_send: true,
            allowed_changes: default_allowed(),
            allowed_per_account: HashMap::new(),
            last_filed_into: HashMap::new(),
            told_about_the_alpha: false,
            check_spelling_as_you_type: true,
            default_sort_order: "date_newest".to_string(),
            calendar_default_view: "agenda".to_string(),
            calendar_show_weekends: true,
            calendar_first_day_of_week: 0,
            default_reminder_minutes: 15,
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

/// Legacy configuration (for backwards compatibility)
#[derive(Debug, Clone)]
pub struct Config {
    settings: HashMap<String, String>,
}

impl Config {
    /// Create a new configuration
    pub fn new() -> Self {
        Self {
            settings: HashMap::new(),
        }
    }

    /// Get a setting value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    /// Set a setting value
    pub fn set(&mut self, key: String, value: String) {
        self.settings.insert(key, value);
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration manager with file persistence
pub struct ConfigManager {
    /// Legacy config
    config: Config,
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
            config: Config::new(),
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

    /// Get legacy configuration (deprecated)
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get mutable legacy configuration (deprecated)
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let mut config = Config::new();
        config.set("test".to_string(), "value".to_string());
        assert_eq!(config.get("test"), Some(&"value".to_string()));
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
        assert_eq!(config.language, "en");
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
}

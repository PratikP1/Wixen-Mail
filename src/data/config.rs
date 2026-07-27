//! Configuration management
//!
//! Handles application settings, account configurations, and persistence.

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
    /// Spell-check language code (e.g. "en", "es", "fr", "de")
    #[serde(default = "default_language")]
    pub language: String,
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

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            download_folder: dirs::download_dir().unwrap_or_else(|| PathBuf::from(".")),
            check_updates: true,
            theme: "default".to_string(),
            font_size: 12,
            date_style: default_date_style(),
            date_order: default_date_order(),
            mute_message_reading: false,
            message_columns: String::new(),
            feedback_channels: String::new(),
            enable_notifications: true,
            log_level: "info".to_string(),
            preview_before_send: true,
            language: "en".to_string(),
            default_sort_order: "date_newest".to_string(),
            calendar_default_view: "agenda".to_string(),
            calendar_show_weekends: true,
            calendar_first_day_of_week: 0,
            default_reminder_minutes: 15,
        }
    }
}

impl AppConfig {
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
        let config_dir = Self::get_config_dir()?;
        Ok(Self {
            config: Config::new(),
            app_config: AppConfig::default(),
            account_configs: HashMap::new(),
            config_dir,
        })
    }

    /// Get the configuration directory
    fn get_config_dir() -> Result<PathBuf> {
        let base_dir = if cfg!(windows) {
            dirs::data_local_dir().or_else(dirs::config_dir)
        } else {
            dirs::config_dir()
        };

        let config_dir = base_dir
            .ok_or_else(|| Error::Config("Could not determine config directory".to_string()))?
            .join("wixen-mail");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| Error::Config(format!("Failed to create config directory: {}", e)))?;
        }

        Ok(config_dir)
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

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            config: Config::new(),
            app_config: AppConfig::default(),
            account_configs: HashMap::new(),
            config_dir: PathBuf::from("."),
        })
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
        let manager = ConfigManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_config_manager_uses_app_directory() {
        let config_dir = ConfigManager::get_config_dir().unwrap();
        assert!(config_dir.ends_with("wixen-mail"));

        if cfg!(windows)
            && let Some(base) = dirs::data_local_dir().or_else(dirs::config_dir)
        {
            assert!(config_dir.starts_with(base));
        }
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

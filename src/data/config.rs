//! Configuration management
//!
//! Handles application settings and preferences.

use crate::common::Result;
use std::collections::HashMap;

/// Application configuration
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

/// Configuration manager
#[derive(Default)]
pub struct ConfigManager {
    config: Config,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: Config::new(),
        })
    }

    /// Load configuration from file
    pub fn load(&mut self) -> Result<()> {
        // TODO: Load configuration from file
        Ok(())
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        // TODO: Save configuration to file
        Ok(())
    }

    /// Get configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get mutable configuration
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
        let manager = ConfigManager::new();
        assert!(manager.is_ok());
    }
}

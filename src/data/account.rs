//! Account management for multiple email accounts
//!
//! This module provides data structures and logic for managing multiple email accounts.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Email account configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Unique account identifier
    pub id: String,

    /// User-friendly account name
    pub name: String,

    /// Email address
    pub email: String,

    /// IMAP server configuration
    pub imap_server: String,
    pub imap_port: String,
    pub imap_use_tls: bool,

    /// SMTP server configuration
    pub smtp_server: String,
    pub smtp_port: String,
    pub smtp_use_tls: bool,

    /// Authentication
    pub username: String,
    #[serde(skip_serializing)] // Don't serialize password to logs
    pub password: String,

    /// Account settings
    pub enabled: bool,
    pub check_interval_minutes: u32,

    /// Provider name (if using a preset)
    pub provider: Option<String>,

    /// Last sync timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sync: Option<SystemTime>,

    /// Account color for visual distinction (hex code)
    #[serde(default = "default_account_color")]
    pub color: String,
}

fn default_account_color() -> String {
    "#4A90E2".to_string() // Default blue
}

impl Account {
    /// Create a new account with default settings
    pub fn new(name: String, email: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            email,
            imap_server: String::new(),
            imap_port: "993".to_string(),
            imap_use_tls: true,
            smtp_server: String::new(),
            smtp_port: "465".to_string(),
            smtp_use_tls: true,
            username: String::new(),
            password: String::new(),
            enabled: true,
            check_interval_minutes: 5,
            provider: None,
            last_sync: None,
            color: default_account_color(),
        }
    }

    /// Create account from provider preset
    pub fn from_provider(
        name: String,
        email: String,
        provider: &crate::data::email_providers::EmailProvider,
    ) -> Self {
        let mut account = Self::new(name, email.clone());
        account.imap_server = provider.imap_server.clone();
        account.imap_port = provider.imap_port.to_string();
        account.imap_use_tls = provider.imap_tls;
        account.smtp_server = provider.smtp_server.clone();
        account.smtp_port = provider.smtp_port.to_string();
        account.smtp_use_tls = provider.smtp_tls;
        account.username = email;
        account.provider = Some(provider.name.clone());
        account
    }

    /// Validate account configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Account name is required".to_string());
        }

        if self.email.trim().is_empty() {
            return Err("Email address is required".to_string());
        }

        if !self.email.contains('@') {
            return Err("Invalid email address format".to_string());
        }

        if self.imap_server.trim().is_empty() {
            return Err("IMAP server is required".to_string());
        }

        if self.smtp_server.trim().is_empty() {
            return Err("SMTP server is required".to_string());
        }

        if self.username.trim().is_empty() {
            return Err("Username is required".to_string());
        }

        if self.password.is_empty() {
            return Err("Password is required".to_string());
        }

        Ok(())
    }

    /// Get a display name for the account
    pub fn display_name(&self) -> String {
        format!("{} <{}>", self.name, self.email)
    }

    /// Update last sync timestamp
    pub fn mark_synced(&mut self) {
        self.last_sync = Some(SystemTime::now());
    }

    /// Migrate from old AccountConfig to new Account
    pub fn from_account_config(config: &crate::presentation::ui_integrated::AccountConfig) -> Self {
        let email = config.email.clone();

        // Detect provider from email
        let provider = if let Some(provider_name) = &config.selected_provider {
            Some(provider_name.clone())
        } else {
            crate::data::email_providers::detect_provider_from_email(&email).map(|p| p.name.clone())
        };

        Account {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Primary Account".to_string(), // User can rename later
            email,
            imap_server: config.imap_server.clone(),
            imap_port: config.imap_port.clone(),
            imap_use_tls: config.imap_use_tls,
            smtp_server: config.smtp_server.clone(),
            smtp_port: config.smtp_port.clone(),
            smtp_use_tls: config.smtp_use_tls,
            username: config.username.clone(),
            password: config.password.clone(),
            enabled: true,
            check_interval_minutes: 5,
            provider,
            last_sync: None,
            color: "#4A90E2".to_string(), // Default blue
        }
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new("New Account".to_string(), "user@example.com".to_string())
    }
}

/// Account manager for CRUD operations
pub struct AccountManager {
    accounts: Vec<Account>,
    active_account_id: Option<String>,
}

impl AccountManager {
    /// Create a new account manager
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
            active_account_id: None,
        }
    }

    /// Load accounts from a vector
    pub fn load(&mut self, accounts: Vec<Account>, active_id: Option<String>) {
        self.accounts = accounts;
        self.active_account_id = active_id;
    }

    /// Get all accounts
    pub fn get_accounts(&self) -> &[Account] {
        &self.accounts
    }

    /// Get active account
    pub fn get_active_account(&self) -> Option<&Account> {
        self.active_account_id
            .as_ref()
            .and_then(|id| self.accounts.iter().find(|a| &a.id == id))
    }

    /// Get active account ID
    pub fn get_active_account_id(&self) -> Option<&String> {
        self.active_account_id.as_ref()
    }

    /// Get account by ID
    pub fn get_account(&self, id: &str) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == id)
    }

    /// Get mutable account by ID
    pub fn get_account_mut(&mut self, id: &str) -> Option<&mut Account> {
        self.accounts.iter_mut().find(|a| a.id == id)
    }

    /// Add a new account
    pub fn add_account(&mut self, account: Account) -> Result<String, String> {
        account.validate()?;

        // Check for duplicate email
        if self.accounts.iter().any(|a| a.email == account.email) {
            return Err(format!(
                "Account with email {} already exists",
                account.email
            ));
        }

        let id = account.id.clone();
        self.accounts.push(account);

        // Set as active if this is the first account
        if self.accounts.len() == 1 {
            self.active_account_id = Some(id.clone());
        }

        Ok(id)
    }

    /// Update an existing account
    pub fn update_account(&mut self, account: Account) -> Result<(), String> {
        account.validate()?;

        let index = self
            .accounts
            .iter()
            .position(|a| a.id == account.id)
            .ok_or_else(|| "Account not found".to_string())?;

        self.accounts[index] = account;
        Ok(())
    }

    /// Delete an account
    pub fn delete_account(&mut self, id: &str) -> Result<(), String> {
        let index = self
            .accounts
            .iter()
            .position(|a| a.id == id)
            .ok_or_else(|| "Account not found".to_string())?;

        self.accounts.remove(index);

        // Clear active account if it was deleted
        if self.active_account_id.as_ref() == Some(&id.to_string()) {
            self.active_account_id = self.accounts.first().map(|a| a.id.clone());
        }

        Ok(())
    }

    /// Set active account
    pub fn set_active_account(&mut self, id: &str) -> Result<(), String> {
        if !self.accounts.iter().any(|a| a.id == id) {
            return Err("Account not found".to_string());
        }

        self.active_account_id = Some(id.to_string());
        Ok(())
    }

    /// Get enabled accounts
    pub fn get_enabled_accounts(&self) -> Vec<&Account> {
        self.accounts.iter().filter(|a| a.enabled).collect()
    }

    /// Enable or disable an account
    pub fn set_account_enabled(&mut self, id: &str, enabled: bool) -> Result<(), String> {
        let account = self
            .get_account_mut(id)
            .ok_or_else(|| "Account not found".to_string())?;

        account.enabled = enabled;
        Ok(())
    }
}

impl Default for AccountManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let account = Account::new("Test".to_string(), "test@example.com".to_string());
        assert_eq!(account.name, "Test");
        assert_eq!(account.email, "test@example.com");
        assert!(account.enabled);
        assert_eq!(account.check_interval_minutes, 5);
    }

    #[test]
    fn test_account_validation() {
        let mut account = Account::new("".to_string(), "test@example.com".to_string());
        assert!(account.validate().is_err()); // Missing name

        account.name = "Test".to_string();
        account.email = "invalid".to_string();
        assert!(account.validate().is_err()); // Invalid email

        account.email = "test@example.com".to_string();
        // Set required fields for validation
        account.imap_server = "imap.example.com".to_string();
        account.smtp_server = "smtp.example.com".to_string();
        account.username = "test".to_string();
        assert!(account.validate().is_err()); // Missing password

        account.password = "password".to_string();
        assert!(account.validate().is_ok()); // All fields valid
    }

    #[test]
    fn test_account_display_name() {
        let account = Account::new("Test Account".to_string(), "test@example.com".to_string());
        assert_eq!(account.display_name(), "Test Account <test@example.com>");
    }

    fn create_valid_account(name: &str, email: &str) -> Account {
        let mut account = Account::new(name.to_string(), email.to_string());
        account.imap_server = "imap.example.com".to_string();
        account.smtp_server = "smtp.example.com".to_string();
        account.username = email.to_string();
        account.password = "password".to_string();
        account
    }

    #[test]
    fn test_account_manager_add() {
        let mut manager = AccountManager::new();
        let account = create_valid_account("Test", "test@example.com");

        let id = manager.add_account(account).unwrap();
        assert_eq!(manager.get_accounts().len(), 1);
        assert_eq!(manager.get_active_account_id(), Some(&id));
    }

    #[test]
    fn test_account_manager_duplicate_email() {
        let mut manager = AccountManager::new();
        let account1 = create_valid_account("Test1", "test@example.com");
        let account2 = create_valid_account("Test2", "test@example.com");

        manager.add_account(account1).unwrap();
        assert!(manager.add_account(account2).is_err());
    }

    #[test]
    fn test_account_manager_delete() {
        let mut manager = AccountManager::new();
        let account = create_valid_account("Test", "test@example.com");

        let id = manager.add_account(account).unwrap();
        manager.delete_account(&id).unwrap();
        assert_eq!(manager.get_accounts().len(), 0);
        assert!(manager.get_active_account_id().is_none());
    }

    #[test]
    fn test_account_manager_switch_active() {
        let mut manager = AccountManager::new();
        let account1 = create_valid_account("Test1", "test1@example.com");
        let account2 = create_valid_account("Test2", "test2@example.com");

        let id1 = manager.add_account(account1).unwrap();
        let id2 = manager.add_account(account2).unwrap();

        assert_eq!(manager.get_active_account_id(), Some(&id1));
        manager.set_active_account(&id2).unwrap();
        assert_eq!(manager.get_active_account_id(), Some(&id2));
    }

    #[test]
    fn test_migrate_from_account_config() {
        use crate::presentation::ui_integrated::AccountConfig;

        let config = AccountConfig {
            email: "user@gmail.com".to_string(),
            selected_provider: Some("Gmail".to_string()),
            imap_server: "imap.gmail.com".to_string(),
            imap_port: "993".to_string(),
            imap_use_tls: true,
            smtp_server: "smtp.gmail.com".to_string(),
            smtp_port: "465".to_string(),
            smtp_use_tls: true,
            username: "user@gmail.com".to_string(),
            password: "password123".to_string(),
        };

        let account = Account::from_account_config(&config);

        assert_eq!(account.email, "user@gmail.com");
        assert_eq!(account.name, "Primary Account");
        assert_eq!(account.imap_server, "imap.gmail.com");
        assert_eq!(account.imap_port, "993");
        assert!(account.enabled);
        assert!(!account.id.is_empty()); // UUID generated
        assert_eq!(account.provider, Some("Gmail".to_string()));
    }
}

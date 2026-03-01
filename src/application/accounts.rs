//! Account management
//!
//! Manages email accounts, credentials, and authentication.

use crate::common::{
    types::{AccountSettings, Credentials, Id, Protocol, ServerConfig},
    Result,
};

/// Account information with full server configuration
#[derive(Debug, Clone)]
pub struct Account {
    pub id: Id,
    pub name: String,
    pub email_address: String,
    pub protocol: Protocol,
    pub incoming_server: ServerConfig,
    pub outgoing_server: ServerConfig,
    pub credentials: Credentials,
    pub settings: AccountSettings,
}

impl Account {
    /// Create a new account with basic information
    pub fn new(
        name: String,
        email_address: String,
        protocol: Protocol,
        incoming_server: ServerConfig,
        outgoing_server: ServerConfig,
        credentials: Credentials,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            email_address,
            protocol,
            incoming_server,
            outgoing_server,
            credentials,
            settings: AccountSettings::default(),
        }
    }

    /// Create a simple account for testing
    pub fn new_simple(name: String, email_address: String, protocol: Protocol) -> Self {
        let incoming = ServerConfig::new("imap.example.com".to_string(), 993, true);
        let outgoing = ServerConfig::new("smtp.example.com".to_string(), 587, true);
        let credentials = Credentials::new(email_address.clone(), "password".to_string());

        Self::new(
            name,
            email_address,
            protocol,
            incoming,
            outgoing,
            credentials,
        )
    }
}

/// Manages email accounts
#[derive(Default)]
pub struct AccountManager {
    accounts: Vec<Account>,
}

impl AccountManager {
    /// Create a new account manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            accounts: Vec::new(),
        })
    }

    /// Add a new account
    pub fn add_account(&mut self, account: Account) -> Result<()> {
        self.accounts.push(account);
        Ok(())
    }

    /// Get all accounts
    pub fn get_accounts(&self) -> &[Account] {
        &self.accounts
    }

    /// Get account by ID
    pub fn get_account(&self, id: &str) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let account = Account::new_simple(
            "Test Account".to_string(),
            "test@example.com".to_string(),
            Protocol::Imap,
        );
        assert_eq!(account.name, "Test Account");
        assert_eq!(account.email_address, "test@example.com");
        assert_eq!(account.incoming_server.host, "imap.example.com");
        assert_eq!(account.outgoing_server.host, "smtp.example.com");
    }

    #[test]
    fn test_account_with_full_config() {
        let incoming = ServerConfig::new("imap.gmail.com".to_string(), 993, true);
        let outgoing = ServerConfig::new("smtp.gmail.com".to_string(), 465, true);
        let creds = Credentials::new("user@gmail.com".to_string(), "pass".to_string());

        let account = Account::new(
            "Gmail Account".to_string(),
            "user@gmail.com".to_string(),
            Protocol::Imap,
            incoming,
            outgoing,
            creds,
        );

        assert_eq!(account.name, "Gmail Account");
        assert_eq!(account.incoming_server.port, 993);
        assert_eq!(account.outgoing_server.port, 465);
    }

    #[test]
    fn test_account_manager() {
        let mut manager = AccountManager::new().unwrap();
        let account = Account::new_simple(
            "Test".to_string(),
            "test@example.com".to_string(),
            Protocol::Imap,
        );
        let id = account.id.clone();
        manager.add_account(account).unwrap();

        assert_eq!(manager.get_accounts().len(), 1);
        assert!(manager.get_account(&id).is_some());
    }
}

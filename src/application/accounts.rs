//! Account management
//!
//! Manages email accounts, credentials, and authentication.

use crate::common::{Result, types::{Id, Protocol}};

/// Account information
#[derive(Debug, Clone)]
pub struct Account {
    pub id: Id,
    pub name: String,
    pub email_address: String,
    pub protocol: Protocol,
}

impl Account {
    /// Create a new account
    pub fn new(name: String, email_address: String, protocol: Protocol) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            email_address,
            protocol,
        }
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
        let account = Account::new(
            "Test Account".to_string(),
            "test@example.com".to_string(),
            Protocol::Imap,
        );
        assert_eq!(account.name, "Test Account");
        assert_eq!(account.email_address, "test@example.com");
    }

    #[test]
    fn test_account_manager() {
        let mut manager = AccountManager::new().unwrap();
        let account = Account::new(
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

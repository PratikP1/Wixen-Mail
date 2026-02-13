//! Contact management
//!
//! Manages contacts and address book.

use crate::common::{types::EmailAddress, Result};

/// Contact information
#[derive(Debug, Clone)]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub email: EmailAddress,
    pub notes: Option<String>,
}

impl Contact {
    /// Create a new contact
    pub fn new(name: String, email: EmailAddress) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            email,
            notes: None,
        }
    }
}

/// Contact manager
#[derive(Default)]
pub struct ContactManager {
    contacts: Vec<Contact>,
}

impl ContactManager {
    /// Create a new contact manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            contacts: Vec::new(),
        })
    }

    /// Add a contact
    pub fn add_contact(&mut self, contact: Contact) -> Result<()> {
        self.contacts.push(contact);
        Ok(())
    }

    /// Get all contacts
    pub fn get_contacts(&self) -> &[Contact] {
        &self.contacts
    }

    /// Search contacts by name or email
    pub fn search(&self, query: &str) -> Vec<&Contact> {
        let query = query.to_lowercase();
        self.contacts
            .iter()
            .filter(|c| {
                c.name.to_lowercase().contains(&query)
                    || c.email.address.to_lowercase().contains(&query)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_creation() {
        let email = EmailAddress::new("test@example.com".to_string(), None);
        let contact = Contact::new("Test User".to_string(), email);
        assert_eq!(contact.name, "Test User");
    }

    #[test]
    fn test_contact_search() {
        let mut manager = ContactManager::new().unwrap();
        let email = EmailAddress::new("test@example.com".to_string(), None);
        let contact = Contact::new("Test User".to_string(), email);
        manager.add_contact(contact).unwrap();

        let results = manager.search("test");
        assert_eq!(results.len(), 1);
    }
}

//! Contact management
//!
//! Manages contacts, address book, and contact groups (distribution lists).

use crate::common::{types::EmailAddress, Result};
use crate::data::message_cache::ContactEntry;

/// Contact information
#[derive(Debug, Clone)]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub email: EmailAddress,
    pub notes: Option<String>,
    /// Group IDs this contact belongs to
    pub group_ids: Vec<String>,
}

impl Contact {
    /// Create a new contact
    pub fn new(name: String, email: EmailAddress) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            email,
            notes: None,
            group_ids: Vec::new(),
        }
    }
}

impl From<ContactEntry> for Contact {
    fn from(ce: ContactEntry) -> Self {
        Self {
            id: ce.id,
            name: ce.name,
            email: EmailAddress::new(ce.email, None),
            notes: if ce.notes.as_deref() == Some("") { None } else { ce.notes },
            group_ids: Vec::new(),
        }
    }
}

/// Contact group (distribution list)
#[derive(Debug, Clone)]
pub struct ContactGroupInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub member_count: usize,
}

/// Contact manager
#[derive(Default)]
pub struct ContactManager {
    contacts: Vec<Contact>,
    groups: Vec<ContactGroupInfo>,
}

impl ContactManager {
    /// Create a new contact manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            contacts: Vec::new(),
            groups: Vec::new(),
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

    /// Create a new contact group
    pub fn create_group(&mut self, name: String, description: Option<String>) -> ContactGroupInfo {
        let group = ContactGroupInfo {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            member_count: 0,
        };
        self.groups.push(group.clone());
        group
    }

    /// Get all contact groups
    pub fn get_groups(&self) -> &[ContactGroupInfo] {
        &self.groups
    }

    /// Add a contact to a group
    pub fn add_to_group(&mut self, contact_id: &str, group_id: &str) -> Result<()> {
        if let Some(contact) = self.contacts.iter_mut().find(|c| c.id == contact_id) {
            if !contact.group_ids.contains(&group_id.to_string()) {
                contact.group_ids.push(group_id.to_string());
            }
        }
        if let Some(group) = self.groups.iter_mut().find(|g| g.id == group_id) {
            group.member_count = self
                .contacts
                .iter()
                .filter(|c| c.group_ids.contains(&group_id.to_string()))
                .count();
        }
        Ok(())
    }

    /// Remove a contact from a group
    pub fn remove_from_group(&mut self, contact_id: &str, group_id: &str) -> Result<()> {
        if let Some(contact) = self.contacts.iter_mut().find(|c| c.id == contact_id) {
            contact.group_ids.retain(|id| id != group_id);
        }
        if let Some(group) = self.groups.iter_mut().find(|g| g.id == group_id) {
            group.member_count = self
                .contacts
                .iter()
                .filter(|c| c.group_ids.contains(&group_id.to_string()))
                .count();
        }
        Ok(())
    }

    /// Get all contacts in a specific group
    pub fn contacts_in_group(&self, group_id: &str) -> Vec<&Contact> {
        self.contacts
            .iter()
            .filter(|c| c.group_ids.contains(&group_id.to_string()))
            .collect()
    }

    /// Resolve a group to a comma-separated email list (for compose dialog)
    pub fn resolve_group_emails(&self, group_id: &str) -> String {
        self.contacts_in_group(group_id)
            .iter()
            .map(|c| c.email.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Delete a group (does not delete contacts)
    pub fn delete_group(&mut self, group_id: &str) {
        // Remove group from all contacts
        for contact in &mut self.contacts {
            contact.group_ids.retain(|id| id != group_id);
        }
        self.groups.retain(|g| g.id != group_id);
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
        assert!(contact.group_ids.is_empty());
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

    #[test]
    fn test_contact_group_lifecycle() {
        let mut manager = ContactManager::new().unwrap();

        // Create group
        let group = manager.create_group("Team A".to_string(), Some("The A team".to_string()));
        assert_eq!(group.name, "Team A");
        assert_eq!(group.member_count, 0);

        // Add contacts
        let email1 = EmailAddress::new("alice@example.com".to_string(), Some("Alice".to_string()));
        let c1 = Contact::new("Alice".to_string(), email1);
        let c1_id = c1.id.clone();
        manager.add_contact(c1).unwrap();

        let email2 = EmailAddress::new("bob@example.com".to_string(), Some("Bob".to_string()));
        let c2 = Contact::new("Bob".to_string(), email2);
        let c2_id = c2.id.clone();
        manager.add_contact(c2).unwrap();

        // Add to group
        manager.add_to_group(&c1_id, &group.id).unwrap();
        manager.add_to_group(&c2_id, &group.id).unwrap();

        // Check membership
        let members = manager.contacts_in_group(&group.id);
        assert_eq!(members.len(), 2);

        // Resolve emails
        let emails = manager.resolve_group_emails(&group.id);
        assert!(emails.contains("alice@example.com"));
        assert!(emails.contains("bob@example.com"));

        // Remove from group
        manager.remove_from_group(&c1_id, &group.id).unwrap();
        let members = manager.contacts_in_group(&group.id);
        assert_eq!(members.len(), 1);

        // Delete group
        manager.delete_group(&group.id);
        assert!(manager.get_groups().is_empty());
        // Contacts still exist
        assert_eq!(manager.get_contacts().len(), 2);
    }
}

//! Message composition
//!
//! Handles creation and editing of email messages.

use crate::common::{Result, types::EmailAddress};

/// Draft message
#[derive(Debug, Clone)]
pub struct Draft {
    pub id: String,
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub bcc: Vec<EmailAddress>,
    pub subject: String,
    pub body: String,
}

impl Draft {
    /// Create a new draft
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            subject: String::new(),
            body: String::new(),
        }
    }
}

impl Default for Draft {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages message composition
#[derive(Default)]
pub struct CompositionManager {
    drafts: Vec<Draft>,
}

impl CompositionManager {
    /// Create a new composition manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            drafts: Vec::new(),
        })
    }

    /// Create a new draft
    pub fn create_draft(&mut self) -> &mut Draft {
        let draft = Draft::new();
        self.drafts.push(draft);
        self.drafts.last_mut().unwrap()
    }

    /// Get all drafts
    pub fn get_drafts(&self) -> &[Draft] {
        &self.drafts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draft_creation() {
        let draft = Draft::new();
        assert!(draft.to.is_empty());
        assert!(draft.subject.is_empty());
    }

    #[test]
    fn test_composition_manager() {
        let mut manager = CompositionManager::new().unwrap();
        let draft = manager.create_draft();
        draft.subject = "Test".to_string();
        
        assert_eq!(manager.get_drafts().len(), 1);
        assert_eq!(manager.get_drafts()[0].subject, "Test");
    }
}

//! Filter engine
//!
//! Rule-based message filtering and organization.

use crate::common::Result;
use crate::data::message_cache::{CachedMessage, MessageFilterRule};

/// Filter action types
#[derive(Debug, Clone)]
pub enum FilterAction {
    MoveToFolder(String),
    AddTag(String),
    MarkAsRead,
    Delete,
}

/// Message filter rule
#[derive(Debug, Clone)]
pub struct FilterRule {
    pub id: String,
    pub name: String,
    /// Message field to evaluate ("subject", "from", or "to")
    pub field: String,
    /// Case-insensitive "contains" match text
    pub pattern: String,
    pub action: FilterAction,
    pub enabled: bool,
}

/// Filter engine for automatic message processing
#[derive(Default)]
pub struct FilterEngine {
    rules: Vec<FilterRule>,
}

impl FilterEngine {
    /// Create a new filter engine
    pub fn new() -> Result<Self> {
        Ok(Self { rules: Vec::new() })
    }

    /// Add a filter rule
    pub fn add_rule(&mut self, rule: FilterRule) -> Result<()> {
        self.rules.push(rule);
        Ok(())
    }

    /// Get all rules
    pub fn get_rules(&self) -> &[FilterRule] {
        &self.rules
    }
    
    /// Evaluate all enabled rules against a message and return matched actions
    pub fn evaluate_message(&self, message: &CachedMessage) -> Vec<FilterAction> {
        self.rules
            .iter()
            .filter(|rule| rule.enabled && Self::matches(rule, message))
            .map(|rule| rule.action.clone())
            .collect()
    }
    
    /// Convert persisted rules into runtime rules for execution
    pub fn load_from_persisted(&mut self, rules: &[MessageFilterRule]) {
        self.rules = rules
            .iter()
            .filter_map(Self::from_persisted_rule)
            .collect();
    }
    
    fn matches(rule: &FilterRule, message: &CachedMessage) -> bool {
        let target = match rule.field.as_str() {
            "subject" => &message.subject,
            "from" => &message.from_addr,
            "to" => &message.to_addr,
            _ => return false,
        };
        target.to_lowercase().contains(&rule.pattern.to_lowercase())
    }
    
    fn from_persisted_rule(rule: &MessageFilterRule) -> Option<FilterRule> {
        let action = match rule.action_type.as_str() {
            "move_to_folder" => FilterAction::MoveToFolder(Self::validated_action_value(rule.action_value.as_ref())?),
            "add_tag" => FilterAction::AddTag(Self::validated_action_value(rule.action_value.as_ref())?),
            "mark_as_read" => FilterAction::MarkAsRead,
            "delete" => FilterAction::Delete,
            _ => return None,
        };
        
        Some(FilterRule {
            id: rule.id.clone(),
            name: rule.name.clone(),
            field: rule.field.clone(),
            pattern: rule.pattern.clone(),
            action,
            enabled: rule.enabled,
        })
    }
    
    fn validated_action_value(value: Option<&String>) -> Option<String> {
        let value = value?.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_engine_creation() {
        let engine = FilterEngine::new();
        assert!(engine.is_ok());
    }
    
    #[test]
    fn test_filter_engine_evaluates_message() {
        let mut engine = FilterEngine::new().unwrap();
        engine.add_rule(FilterRule {
            id: "r1".to_string(),
            name: "Mark newsletter as read".to_string(),
            field: "subject".to_string(),
            pattern: "newsletter".to_string(),
            action: FilterAction::MarkAsRead,
            enabled: true,
        }).unwrap();
        
        let message = CachedMessage {
            id: 1,
            uid: 1,
            folder_id: 1,
            message_id: "msg-1".to_string(),
            subject: "Weekly Newsletter".to_string(),
            from_addr: "sender@example.com".to_string(),
            to_addr: "user@example.com".to_string(),
            cc: None,
            date: "2026-01-01".to_string(),
            body_plain: None,
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        };
        
        let actions = engine.evaluate_message(&message);
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], FilterAction::MarkAsRead));
    }
}

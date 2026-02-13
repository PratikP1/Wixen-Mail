//! Filter engine
//!
//! Rule-based message filtering and organization.

use crate::common::Result;
use crate::data::message_cache::{CachedMessage, MessageFilterRule};
use regex::Regex;

/// Filter action types
#[derive(Debug, Clone)]
pub enum FilterAction {
    MoveToFolder(String),
    AddTag(String),
    MarkAsRead,
    MarkAsUnread,
    Star,
    Unstar,
    Delete,
}

/// Message filter rule
#[derive(Debug, Clone)]
pub struct FilterRule {
    pub id: String,
    pub name: String,
    /// Message field to evaluate ("subject", "from", or "to")
    pub field: String,
    /// Match type ("contains", "equals", "starts_with", "regex", "is_true", etc.)
    pub match_type: String,
    /// Case-insensitive "contains" match text
    pub pattern: String,
    /// Whether match should be case-sensitive for string comparisons
    pub case_sensitive: bool,
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
        fn bool_to_str(value: bool) -> &'static str {
            if value { "true" } else { "false" }
        }
        
        let target_text = match rule.field.as_str() {
            "subject" => Some(message.subject.as_str()),
            "from" => Some(message.from_addr.as_str()),
            "to" => Some(message.to_addr.as_str()),
            "cc" => message.cc.as_deref(),
            "date" => Some(message.date.as_str()),
            "message_id" => Some(message.message_id.as_str()),
            "body_plain" => message.body_plain.as_deref(),
            "body_html" => message.body_html.as_deref(),
            "read" => Some(bool_to_str(message.read)),
            "starred" => Some(bool_to_str(message.starred)),
            "deleted" => Some(bool_to_str(message.deleted)),
            _ => None,
        };
        let Some(target_text) = target_text else {
            return false;
        };
        
        let lhs = if rule.case_sensitive {
            target_text.to_string()
        } else {
            target_text.to_lowercase()
        };
        let rhs = if rule.case_sensitive {
            rule.pattern.clone()
        } else {
            rule.pattern.to_lowercase()
        };
        
        match rule.match_type.as_str() {
            "contains" => lhs.contains(&rhs),
            "not_contains" => !lhs.contains(&rhs),
            "equals" => lhs == rhs,
            "not_equals" => lhs != rhs,
            "starts_with" => lhs.starts_with(&rhs),
            "ends_with" => lhs.ends_with(&rhs),
            "is_empty" => lhs.trim().is_empty(),
            "is_not_empty" => !lhs.trim().is_empty(),
            "is_true" => lhs == "true",
            "is_false" => lhs == "false",
            "regex" => {
                match Regex::new(&rule.pattern) {
                    Ok(regex) => regex.is_match(target_text),
                    Err(e) => {
                        tracing::warn!("Invalid regex pattern '{}' in rule '{}': {}", rule.pattern, rule.name, e);
                        false
                    }
                }
            }
            _ => false,
        }
    }
    
    fn from_persisted_rule(rule: &MessageFilterRule) -> Option<FilterRule> {
        let action = match rule.action_type.as_str() {
            "move_to_folder" => FilterAction::MoveToFolder(Self::validated_action_value(rule.action_value.as_ref())?),
            "add_tag" => FilterAction::AddTag(Self::validated_action_value(rule.action_value.as_ref())?),
            "mark_as_read" => FilterAction::MarkAsRead,
            "mark_as_unread" => FilterAction::MarkAsUnread,
            "star" => FilterAction::Star,
            "unstar" => FilterAction::Unstar,
            "delete" => FilterAction::Delete,
            _ => return None,
        };
        
        Some(FilterRule {
            id: rule.id.clone(),
            name: rule.name.clone(),
            field: rule.field.clone(),
            match_type: rule.match_type.clone(),
            pattern: rule.pattern.clone(),
            case_sensitive: rule.case_sensitive,
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
            match_type: "contains".to_string(),
            pattern: "newsletter".to_string(),
            case_sensitive: false,
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
    
    #[test]
    fn test_filter_match_types() {
        let mut engine = FilterEngine::new().unwrap();
        engine.add_rule(FilterRule {
            id: "r2".to_string(),
            name: "Starts with Re".to_string(),
            field: "subject".to_string(),
            match_type: "starts_with".to_string(),
            pattern: "Re:".to_string(),
            case_sensitive: true,
            action: FilterAction::Star,
            enabled: true,
        }).unwrap();
        
        let message = CachedMessage {
            id: 1,
            uid: 1,
            folder_id: 1,
            message_id: "msg-1".to_string(),
            subject: "Re: Project Update".to_string(),
            from_addr: "sender@example.com".to_string(),
            to_addr: "user@example.com".to_string(),
            cc: None,
            date: "2026-01-01".to_string(),
            body_plain: Some("Update".to_string()),
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        };
        
        let actions = engine.evaluate_message(&message);
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], FilterAction::Star));
    }
}

//! Message filter rule persistence operations

use super::{MessageCache, MessageFilterRule};
use crate::common::{Error, Result};
use rusqlite::params;

impl MessageCache {
    /// Create a new message filter rule
    pub fn create_filter_rule(&self, rule: &MessageFilterRule) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO message_filter_rules
             (id, account_id, name, field, match_type, pattern, case_sensitive, action_type, action_value, enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                &rule.id, &rule.account_id, &rule.name, &rule.field,
                &rule.match_type, &rule.pattern, &rule.case_sensitive,
                &rule.action_type, &rule.action_value, &rule.enabled,
                &rule.created_at, &now,
            ],
        ).map_err(|e| Error::Other(format!("Failed to create filter rule: {}", e)))?;
        Ok(())
    }

    /// Get all message filter rules for an account
    pub fn get_filter_rules_for_account(&self, account_id: &str) -> Result<Vec<MessageFilterRule>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, name, field, match_type, pattern, case_sensitive, action_type, action_value, enabled, created_at
             FROM message_filter_rules
             WHERE account_id = ?1
             ORDER BY name"
        ).map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let rules = stmt
            .query_map(params![account_id], |row| {
                Ok(MessageFilterRule {
                    id: row.get(0)?, account_id: row.get(1)?, name: row.get(2)?,
                    field: row.get(3)?, match_type: row.get(4)?, pattern: row.get(5)?,
                    case_sensitive: row.get(6)?, action_type: row.get(7)?,
                    action_value: row.get(8)?, enabled: row.get(9)?, created_at: row.get(10)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query filter rules: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect filter rules: {}", e)))?;
        Ok(rules)
    }

    /// Update an existing message filter rule
    pub fn update_filter_rule(&self, rule: &MessageFilterRule) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE message_filter_rules
             SET name = ?1, field = ?2, match_type = ?3, pattern = ?4, case_sensitive = ?5, action_type = ?6, action_value = ?7, enabled = ?8, updated_at = ?9
             WHERE id = ?10",
            params![
                &rule.name, &rule.field, &rule.match_type, &rule.pattern,
                &rule.case_sensitive, &rule.action_type, &rule.action_value,
                &rule.enabled, &now, &rule.id,
            ],
        ).map_err(|e| Error::Other(format!("Failed to update filter rule: {}", e)))?;
        Ok(())
    }

    /// Delete a message filter rule by ID
    pub fn delete_filter_rule(&self, rule_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM message_filter_rules WHERE id = ?1", params![rule_id])
            .map_err(|e| Error::Other(format!("Failed to delete filter rule: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_filter_rule_operations() {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_filter_rules_{}", nanos));
        let cache = MessageCache::new(temp_dir, None).unwrap();

        let mut rule = MessageFilterRule {
            id: "rule-1".to_string(), account_id: "test@example.com".to_string(),
            name: "Newsletter Cleanup".to_string(), field: "subject".to_string(),
            match_type: "contains".to_string(), pattern: "newsletter".to_string(),
            case_sensitive: false, action_type: "move_to_folder".to_string(),
            action_value: Some("Archive".to_string()), enabled: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        cache.create_filter_rule(&rule).unwrap();
        let rules = cache.get_filter_rules_for_account("test@example.com").unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "Newsletter Cleanup");

        rule.enabled = false;
        rule.match_type = "starts_with".to_string();
        rule.pattern = "promo".to_string();
        cache.update_filter_rule(&rule).unwrap();

        let updated = cache.get_filter_rules_for_account("test@example.com").unwrap();
        assert_eq!(updated[0].pattern, "promo");
        assert!(!updated[0].enabled);

        cache.delete_filter_rule("rule-1").unwrap();
        let empty = cache.get_filter_rules_for_account("test@example.com").unwrap();
        assert!(empty.is_empty());
    }
}

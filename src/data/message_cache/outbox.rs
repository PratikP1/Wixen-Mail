//! Offline outbox queue persistence operations

use super::{MessageCache, QueuedOutboxMessage};
use crate::common::{Error, Result};
use rusqlite::params;

impl MessageCache {
    /// Queue message for later sending when offline
    pub fn queue_outbox_message(&self, item: &QueuedOutboxMessage) -> Result<()> {
        self.conn.execute(
            "INSERT INTO outbox_queue (id, account_id, to_addr, subject, body, attempt_count, last_error, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &item.id, &item.account_id, &item.to_addr, &item.subject,
                &item.body, &item.attempt_count, &item.last_error, &item.created_at,
            ],
        ).map_err(|e| Error::Other(format!("Failed to queue outbox message: {}", e)))?;
        Ok(())
    }

    /// Load queued outbox messages for an account
    pub fn load_outbox_messages(&self, account_id: &str) -> Result<Vec<QueuedOutboxMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, to_addr, subject, body, attempt_count, last_error, created_at
             FROM outbox_queue
             WHERE account_id = ?1
             ORDER BY created_at ASC"
        ).map_err(|e| Error::Other(format!("Failed to prepare outbox query: {}", e)))?;

        let rows = stmt
            .query_map(params![account_id], |row| {
                Ok(QueuedOutboxMessage {
                    id: row.get(0)?, account_id: row.get(1)?, to_addr: row.get(2)?,
                    subject: row.get(3)?, body: row.get(4)?, attempt_count: row.get(5)?,
                    last_error: row.get(6)?, created_at: row.get(7)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query outbox messages: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect outbox messages: {}", e)))?;
        Ok(rows)
    }

    /// Delete queued outbox message
    pub fn delete_outbox_message(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM outbox_queue WHERE id = ?1", params![id])
            .map_err(|e| Error::Other(format!("Failed to delete outbox message: {}", e)))?;
        Ok(())
    }

    /// Update outbox attempt count/error after failed send
    pub fn update_outbox_failure(&self, id: &str, last_error: &str) -> Result<()> {
        self.conn
            .execute(
                "UPDATE outbox_queue
             SET attempt_count = attempt_count + 1, last_error = ?2
             WHERE id = ?1",
                params![id, last_error],
            )
            .map_err(|e| Error::Other(format!("Failed to update outbox failure: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_offline_outbox_queue_operations() {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_outbox_{}", nanos));
        let cache = MessageCache::new(temp_dir, None).unwrap();

        let item = QueuedOutboxMessage {
            id: "outbox-1".to_string(), account_id: "acc-1".to_string(),
            to_addr: "user@example.com".to_string(), subject: "Queued".to_string(),
            body: "Queued body".to_string(), attempt_count: 0, last_error: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        cache.queue_outbox_message(&item).unwrap();

        let loaded = cache.load_outbox_messages("acc-1").unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].subject, "Queued");

        cache.update_outbox_failure("outbox-1", "network down").unwrap();
        let loaded2 = cache.load_outbox_messages("acc-1").unwrap();
        assert_eq!(loaded2[0].attempt_count, 1);
        assert_eq!(loaded2[0].last_error.as_deref(), Some("network down"));

        cache.delete_outbox_message("outbox-1").unwrap();
        let empty = cache.load_outbox_messages("acc-1").unwrap();
        assert!(empty.is_empty());
    }
}

//! Offline outbox queue persistence operations

use super::{MessageCache, QueuedOutboxMessage};
use crate::common::{Error, Result};
use rusqlite::params;

/// What a queued message's row says it is doing.
///
/// The state goes in the subject because that is the column somebody reads
/// and the only one wide enough to carry a sentence. A message that has
/// failed four times looks exactly like one that has not been tried, and
/// the difference is the whole reason to open this folder.
///
/// Free of the error's own text on the first failure, and carrying it after
/// that: one failure is usually the network and says nothing useful, while
/// a repeated one is a reason somebody has to act on.
fn waiting_label(subject: &str, attempts: i64, last_error: Option<&str>) -> String {
    let subject = if subject.trim().is_empty() {
        "No subject"
    } else {
        subject
    };
    match (attempts, last_error) {
        (0, _) => format!("{subject}, waiting to send"),
        (1, _) => format!("{subject}, tried once"),
        (tried, Some(why)) => format!("{subject}, tried {tried} times: {why}"),
        (tried, None) => format!("{subject}, tried {tried} times"),
    }
}

impl MessageCache {
    /// Queue message for later sending when offline
    pub fn queue_outbox_message(&self, item: &QueuedOutboxMessage) -> Result<()> {
        self.conn.execute(
            "INSERT INTO outbox_queue (id, account_id, to_addr, cc_addr, bcc_addr, subject, body, body_html, attachments, attempt_count, last_error, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                &item.id, &item.account_id, &item.to_addr, &item.cc_addr, &item.bcc_addr,
                &item.subject, &item.body, &item.body_html, &item.attachments,
                &item.attempt_count, &item.last_error, &item.created_at,
            ],
        ).map_err(|e| Error::Other(format!("Failed to queue outbox message: {}", e)))?;
        Ok(())
    }

    /// Load queued outbox messages for an account
    pub fn load_outbox_messages(&self, account_id: &str) -> Result<Vec<QueuedOutboxMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, to_addr, cc_addr, bcc_addr, subject, body, body_html, attachments, attempt_count, last_error, created_at
             FROM outbox_queue
             WHERE account_id = ?1
             ORDER BY created_at ASC"
        ).map_err(|e| Error::Other(format!("Failed to prepare outbox query: {}", e)))?;

        let rows = stmt
            .query_map(params![account_id], |row| {
                Ok(QueuedOutboxMessage {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    to_addr: row.get(2)?,
                    cc_addr: row.get(3)?,
                    bcc_addr: row.get(4)?,
                    subject: row.get(5)?,
                    body: row.get(6)?,
                    body_html: row.get(7)?,
                    attachments: row.get(8)?,
                    attempt_count: row.get(9)?,
                    last_error: row.get(10)?,
                    created_at: row.get(11)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query outbox messages: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect outbox messages: {}", e)))?;
        Ok(rows)
    }

    /// The queue, as rows the message list can show.
    ///
    /// Read from the queue itself rather than copied into the messages table.
    /// A copy is a second place for the same fact, and the two drift: mail
    /// would sit in the Outbox folder after it had gone, or leave the folder
    /// while still queued, and there would be nothing to say which was right.
    ///
    /// `rowid` is the identifier, which SQLite gives every row here. It is only
    /// meaningful inside this folder, and the one command that acts on it,
    /// cancelling, looks it up the same way.
    ///
    /// The attempt count and the last error go in the subject line, because
    /// that is the column somebody reads and "tried 4 times" is the thing they
    /// need to know about a message that has not gone.
    pub fn outbox_rows(&self, account_id: &str) -> Result<Vec<super::MessageListRow>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT rowid, to_addr, subject, body, created_at, attempt_count, last_error
                 FROM outbox_queue
                 WHERE account_id = ?1
                 ORDER BY created_at ASC",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare outbox query: {}", e)))?;

        let rows = stmt
            .query_map(params![account_id], |row| {
                let subject: String = row.get(2)?;
                let attempts: i64 = row.get(5)?;
                let last_error: Option<String> = row.get(6)?;
                let snippet: String = row.get(3)?;
                Ok(super::MessageListRow {
                    id: row.get(0)?,
                    uid: 0,
                    message_id: String::new(),
                    refs_header: None,
                    subject: waiting_label(&subject, attempts, last_error.as_deref()),
                    // Where it is going, which is what identifies a message
                    // nobody has received yet. The From column would be your
                    // own address on every row.
                    from_addr: row.get(1)?,
                    to_addr: row.get(1)?,
                    cc: None,
                    reply_to: None,
                    date: row.get(4)?,
                    snippet: Some(snippet.chars().take(200).collect()),
                    size_bytes: None,
                    // Nothing here has been anywhere, so none of the flags a
                    // server would set mean anything. Read, so a queue does not
                    // report itself as unread mail to deal with.
                    read: true,
                    starred: false,
                    answered: false,
                    draft: false,
                    has_attachments: false,
                    safety: crate::service::safety::Safety::Ordinary,
                    safety_reasons: Vec::new(),
                    receipt_to: None,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query the outbox: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect the outbox: {}", e)))?;
        Ok(rows)
    }

    /// Take a message out of the queue by the identifier the list shows.
    ///
    /// Cancelling a send, which is the one thing worth being able to do to a
    /// message that has not gone yet and could not be done at all before: the
    /// queue was a number on the status bar.
    pub fn cancel_queued(&self, rowid: i64) -> Result<bool> {
        let removed = self
            .conn
            .execute("DELETE FROM outbox_queue WHERE rowid = ?1", params![rowid])
            .map_err(|e| Error::Other(format!("Failed to cancel the message: {}", e)))?;
        Ok(removed > 0)
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
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = env::temp_dir().join(format!("wixen_mail_test_outbox_{}", nanos));
        let cache = MessageCache::new(temp_dir, None).unwrap();

        let item = QueuedOutboxMessage {
            id: "outbox-1".to_string(),
            account_id: "acc-1".to_string(),
            to_addr: "user@example.com".to_string(),
            cc_addr: String::new(),
            bcc_addr: String::new(),
            subject: "Queued".to_string(),
            body: "Queued body".to_string(),
            attempt_count: 0,
            last_error: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            body_html: None,
            attachments: String::new(),
        };
        cache.queue_outbox_message(&item).unwrap();

        let loaded = cache.load_outbox_messages("acc-1").unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].subject, "Queued");

        cache
            .update_outbox_failure("outbox-1", "network down")
            .unwrap();
        let loaded2 = cache.load_outbox_messages("acc-1").unwrap();
        assert_eq!(loaded2[0].attempt_count, 1);
        assert_eq!(loaded2[0].last_error.as_deref(), Some("network down"));

        cache.delete_outbox_message("outbox-1").unwrap();
        let empty = cache.load_outbox_messages("acc-1").unwrap();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_the_queue_keeps_cc_and_bcc() {
        // The queue is where they were lost. The composer collects them, the
        // preview shows them, Reply All fills Cc in and announces "2
        // recipients", the draft keeps them, and then the queue had nowhere to
        // put them and only the To addresses were sent. Nothing said so, and
        // there is no copy in Sent to notice it in afterwards.
        let temp_dir = env::temp_dir().join(format!(
            "wixen_mail_test_outbox_cc_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let cache = MessageCache::new(temp_dir, None).unwrap();

        cache
            .queue_outbox_message(&QueuedOutboxMessage {
                id: "outbox-cc".to_string(),
                account_id: "acc-1".to_string(),
                to_addr: "alice@example.com".to_string(),
                cc_addr: "bob@example.com".to_string(),
                bcc_addr: "carol@example.com".to_string(),
                subject: "Reply to all".to_string(),
                body: "Body".to_string(),
                attempt_count: 0,
                last_error: None,
                created_at: chrono::Utc::now().to_rfc3339(),
                body_html: None,
                attachments: String::new(),
            })
            .unwrap();

        let loaded = cache.load_outbox_messages("acc-1").unwrap();
        assert_eq!(loaded[0].cc_addr, "bob@example.com");
        assert_eq!(loaded[0].bcc_addr, "carol@example.com");
    }
}

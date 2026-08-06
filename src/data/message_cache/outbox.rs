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
            "INSERT INTO outbox_queue (id, account_id, to_addr, cc_addr, bcc_addr, subject, body, body_html, attachments, attempt_count, last_error, created_at, in_reply_to, references_header)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                &item.id, &item.account_id, &item.to_addr, &item.cc_addr, &item.bcc_addr,
                &item.subject, &item.body, &item.body_html, &item.attachments,
                &item.attempt_count, &item.last_error, &item.created_at,
                &item.in_reply_to, &item.references,
            ],
        ).map_err(|e| Error::Other(format!("Failed to queue outbox message: {}", e)))?;
        Ok(())
    }

    /// Load queued outbox messages for an account
    pub fn load_outbox_messages(&self, account_id: &str) -> Result<Vec<QueuedOutboxMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, account_id, to_addr, cc_addr, bcc_addr, subject, body, body_html, attachments, attempt_count, last_error, created_at, in_reply_to, references_header
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
                    in_reply_to: row.get(12)?,
                    references: row.get(13)?,
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
                    account_id: account_id.to_string(),
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
    use crate::common::temp_home::TempHome;

    #[test]
    fn test_offline_outbox_queue_operations() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();

        let item = QueuedOutboxMessage {
            id: "outbox-1".to_string(),
            account_id: "acc-1".to_string(),
            to_addr: "user@example.com".to_string(),
            cc_addr: String::new(),
            bcc_addr: String::new(),
            subject: "Queued".to_string(),
            body: "Queued body".to_string(),
            in_reply_to: None,
            references: None,
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

    fn a_cache(what_for: &str) -> TempHome<MessageCache> {
        TempHome::named(what_for, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
        })
    }

    fn queued(id: &str, account_id: &str, subject: &str, created_at: &str) -> QueuedOutboxMessage {
        QueuedOutboxMessage {
            id: id.to_string(),
            account_id: account_id.to_string(),
            to_addr: "alice@example.com".to_string(),
            cc_addr: String::new(),
            bcc_addr: String::new(),
            subject: subject.to_string(),
            body: "Body".to_string(),
            in_reply_to: None,
            references: None,
            attempt_count: 0,
            last_error: None,
            created_at: created_at.to_string(),
            body_html: None,
            attachments: String::new(),
        }
    }

    #[test]
    fn test_a_waiting_message_says_what_it_is_doing() {
        // This is the line read out for every message in the Outbox, and all
        // four ways of writing it were untested. A message that has failed
        // four times sounding exactly like one that has not been tried is the
        // whole difficulty this label exists to remove.
        for (attempts, last_error, expected) in [
            (0, None, "Report, waiting to send"),
            (0, Some("network down"), "Report, waiting to send"),
            (1, None, "Report, tried once"),
            // One failure is usually the network and says nothing worth
            // hearing, so the reason is held back until it repeats.
            (1, Some("network down"), "Report, tried once"),
            (
                4,
                Some("mailbox full"),
                "Report, tried 4 times: mailbox full",
            ),
            (4, None, "Report, tried 4 times"),
        ] {
            assert_eq!(
                waiting_label("Report", attempts, last_error),
                expected,
                "{attempts} attempts with {last_error:?} was said wrongly"
            );
        }
    }

    #[test]
    fn test_a_waiting_message_with_no_subject_is_still_something_to_hear() {
        // An empty subject leaves a row that opens with a comma, which is a
        // row somebody cannot tell from the one above it.
        assert_eq!(waiting_label("", 0, None), "No subject, waiting to send");
        // A subject of nothing but spaces counts as no subject, or the row is
        // read out as a pause followed by a comma.
        assert_eq!(waiting_label("   ", 2, None), "No subject, tried 2 times");
    }

    #[test]
    fn test_the_outbox_shows_this_account_s_waiting_mail_oldest_first() {
        // Showing another account's queue here would be somebody's mail listed
        // under the wrong account, and cancelling from that list would then
        // reach across to it.
        let cache = a_cache("rows");
        cache
            .queue_outbox_message(&queued("b", "acc-1", "Second", "2026-08-01T10:00:00Z"))
            .expect("a message to queue");
        cache
            .queue_outbox_message(&queued("a", "acc-1", "First", "2026-08-01T09:00:00Z"))
            .expect("a message to queue");
        cache
            .queue_outbox_message(&queued("z", "acc-2", "Elsewhere", "2026-08-01T09:30:00Z"))
            .expect("a message to queue");

        let rows = cache.outbox_rows("acc-1").expect("the outbox to be read");

        assert_eq!(
            rows.iter().map(|r| r.subject.as_str()).collect::<Vec<_>>(),
            ["First, waiting to send", "Second, waiting to send"],
            "the outbox listed the wrong messages, or in the wrong order"
        );
        assert!(
            rows.iter().all(|r| r.account_id == "acc-1"),
            "a row was labelled with the wrong account"
        );
        assert!(
            rows.iter().all(|r| r.read),
            "waiting mail counted itself as unread mail to deal with"
        );
        assert_eq!(
            rows[0].to_addr, "alice@example.com",
            "the row does not say where the message is going"
        );

        // The same filter, on the other way in.
        assert_eq!(
            cache
                .load_outbox_messages("acc-1")
                .expect("the queue to load")
                .len(),
            2
        );
    }

    #[test]
    fn test_cancelling_a_waiting_message_removes_it_and_says_whether_it_did() {
        // Cancelling is the one thing worth doing to a message that has not
        // gone. Reporting success without removing it leaves somebody
        // believing they stopped a message that is still on its way.
        let cache = a_cache("cancel");
        cache
            .queue_outbox_message(&queued("a", "acc-1", "Going", "2026-08-01T09:00:00Z"))
            .expect("a message to queue");
        let row_id = cache.outbox_rows("acc-1").expect("the outbox")[0].id;

        assert!(
            cache.cancel_queued(row_id).expect("the cancel to run"),
            "cancelling a message that is there reported that it was not"
        );
        assert!(
            cache.outbox_rows("acc-1").expect("the outbox").is_empty(),
            "the message was reported cancelled and is still queued"
        );
        assert!(
            !cache.cancel_queued(row_id).expect("the cancel to run"),
            "cancelling nothing reported that it cancelled something"
        );
    }

    #[test]
    fn test_a_failure_is_counted_against_the_message_it_happened_to() {
        let cache = a_cache("failure");
        cache
            .queue_outbox_message(&queued("a", "acc-1", "Mine", "2026-08-01T09:00:00Z"))
            .expect("a message to queue");
        cache
            .queue_outbox_message(&queued("b", "acc-1", "Theirs", "2026-08-01T10:00:00Z"))
            .expect("a message to queue");

        cache
            .update_outbox_failure("a", "mailbox full")
            .expect("the failure to be recorded");
        cache
            .update_outbox_failure("a", "mailbox full")
            .expect("the failure to be recorded");

        let queue = cache.load_outbox_messages("acc-1").expect("the queue");
        let counted = |id: &str| {
            queue
                .iter()
                .find(|m| m.id == id)
                .expect("the message")
                .attempt_count
        };

        assert_eq!(counted("a"), 2, "the failures were not counted up");
        assert_eq!(counted("b"), 0, "a failure was counted against both");
        assert_eq!(
            cache.outbox_rows("acc-1").expect("the outbox")[0].subject,
            "Mine, tried 2 times: mailbox full",
            "the outbox does not say why it keeps failing"
        );
    }

    #[test]
    fn test_the_queue_keeps_cc_and_bcc() {
        // The queue is where they were lost. The composer collects them, the
        // preview shows them, Reply All fills Cc in and announces "2
        // recipients", the draft keeps them, and then the queue had nowhere to
        // put them and only the To addresses were sent. Nothing said so, and
        // there is no copy in Sent to notice it in afterwards.
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();

        cache
            .queue_outbox_message(&QueuedOutboxMessage {
                id: "outbox-cc".to_string(),
                account_id: "acc-1".to_string(),
                to_addr: "alice@example.com".to_string(),
                cc_addr: "bob@example.com".to_string(),
                bcc_addr: "carol@example.com".to_string(),
                subject: "Reply to all".to_string(),
                body: "Body".to_string(),
                in_reply_to: None,
                references: None,
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

    #[test]
    fn test_the_queue_keeps_the_conversation_a_reply_belongs_to() {
        // The same shape of loss as Cc: everything above the queue can know
        // which message is being answered, and the queue had nowhere to put it,
        // so every reply arrived as the start of a new conversation.
        let cache = a_cache("outbox_thread");

        cache
            .queue_outbox_message(&QueuedOutboxMessage {
                id: "outbox-reply".to_string(),
                account_id: "acc-1".to_string(),
                to_addr: "alice@example.com".to_string(),
                cc_addr: String::new(),
                bcc_addr: String::new(),
                subject: "Re: Notes".to_string(),
                body: "Body".to_string(),
                in_reply_to: Some("<c@x>".to_string()),
                references: Some("<a@x> <b@x> <c@x>".to_string()),
                attempt_count: 0,
                last_error: None,
                created_at: chrono::Utc::now().to_rfc3339(),
                body_html: None,
                attachments: String::new(),
            })
            .unwrap();

        let loaded = cache.load_outbox_messages("acc-1").unwrap();
        assert_eq!(loaded[0].in_reply_to.as_deref(), Some("<c@x>"));
        assert_eq!(loaded[0].references.as_deref(), Some("<a@x> <b@x> <c@x>"));
    }

    #[test]
    fn test_a_message_queued_before_there_was_a_conversation_still_sends() {
        // The columns arrive on a database that already exists, so a message
        // queued by an older build reads as no reply, which is what it was.
        let folder = tempfile::tempdir().expect("a temporary folder");
        {
            let cache = MessageCache::new(folder.path().to_path_buf(), None).unwrap();
            cache
                .queue_outbox_message(&queued("old-1", "acc-1", "Sent long ago", "2026-01-01"))
                .unwrap();
            for column in ["in_reply_to", "references_header"] {
                cache
                    .conn
                    .execute(
                        &format!("ALTER TABLE outbox_queue DROP COLUMN {column}"),
                        [],
                    )
                    .expect("the column to come off, making this an older database");
            }
        }

        let reopened = MessageCache::new(folder.path().to_path_buf(), None)
            .expect("the older database to open again");
        let loaded = reopened.load_outbox_messages("acc-1").unwrap();
        assert_eq!(loaded.len(), 1, "the queued message did not survive");
        assert_eq!(loaded[0].subject, "Sent long ago");
        assert!(loaded[0].in_reply_to.is_none());
        assert!(loaded[0].references.is_none());
    }
}

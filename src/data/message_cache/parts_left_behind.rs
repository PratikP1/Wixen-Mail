//! Whether a message's parts were left behind on the server.
//!
//! The download of everything fetched each whole message for its text and,
//! until 13-10's build, kept the text alone. The reader fetches nothing for a
//! message whose text is here, so nothing ever stored such a message's parts:
//! it lists no attachments, and a meeting it carries is never said (ledger
//! 633). What the store still knows about it is the bit the header sync wrote
//! from the server's own description of the message, and that no part row
//! names it.
//!
//! These three answers are what the reader asks before fetching such a
//! message once more, whole, when it is selected: whether its parts were left
//! behind, which account's server holds it, and, when its own parse finds no
//! attachment after all, a way to stop asking.

use super::MessageCache;
use crate::common::{Error, Result};
use rusqlite::OptionalExtension;

impl MessageCache {
    /// Whether this message says it carries attachments, none is recorded,
    /// and a server holds it to be asked again.
    ///
    /// False for a message collected over POP or filed here: this computer
    /// has the only copy, so there is nothing to fetch (ledger 92). False for
    /// a row that does not exist.
    pub fn parts_were_left_behind(&self, message_id: i64) -> Result<bool> {
        let only_copy_is_here = super::messages::ONLY_COPY_IS_HERE;
        self.conn
            .query_row(
                &format!(
                    "SELECT EXISTS (
                         SELECT 1 FROM messages m
                         WHERE m.id = ?1
                           AND m.has_attachments = 1
                           -- no part row names it: the download left them behind
                           AND NOT EXISTS (SELECT 1 FROM attachments a WHERE a.message_id = m.id)
                           -- a server holds it: not collected over POP, not filed here
                           AND NOT {only_copy_is_here}
                     )"
                ),
                [message_id],
                |row| row.get(0),
            )
            .map_err(|e| Error::Other(format!("Failed to ask whether parts were left behind: {e}")))
    }

    /// Record that this message carries no attachment, so it is not asked
    /// about again.
    ///
    /// For a message whose own parse found none where the server's
    /// description of it counted one. The two disagree for a part the server
    /// calls an attachment and the parser reads as the message's text, and
    /// without this "fetched once" would be "fetched on every selection".
    pub fn record_that_it_carries_no_attachments(&self, message_id: i64) -> Result<()> {
        self.conn
            .execute(
                "UPDATE messages SET has_attachments = 0 WHERE id = ?1",
                [message_id],
            )
            .map_err(|e| Error::Other(format!("Failed to record that nothing is attached: {e}")))?;
        Ok(())
    }

    /// The account whose folder this message is filed in.
    ///
    /// Asked of the row itself, because All Inboxes and a conversation show
    /// messages from several accounts at once and the account last opened is
    /// the wrong server for some of them.
    pub fn the_account_a_message_is_in(&self, message_id: i64) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT f.account_id FROM messages m JOIN folders f ON f.id = m.folder_id
                 WHERE m.id = ?1",
                [message_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("Failed to read the account a message is in: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::attachment_content::AttachmentWithContent;
    use crate::data::message_cache::{CachedAttachment, CachedFolder, CachedMessage, MessageCache};
    use crate::service::mime::WhatTheSenderSaid;

    fn a_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_parts_left_behind_", |dir| {
            let cache = MessageCache::new(dir.to_path_buf(), None).expect("cache");
            cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: "acc-1".to_string(),
                    name: "INBOX".to_string(),
                    path: "INBOX".to_string(),
                    folder_type: "Inbox".to_string(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("a folder");
            cache
        })
    }

    /// A message the header sync said carries attachments, with none
    /// recorded: how the download of everything left one before 13-10.
    fn left_behind(cache: &MessageCache, uid: u32) -> i64 {
        let row = cache
            .save_message(&CachedMessage {
                id: 0,
                uid,
                folder_id: 1,
                message_id: format!("<{uid}@example.com>"),
                subject: "The agenda".to_string(),
                from_addr: "alice@example.com".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                date: "2026-09-06".to_string(),
                body_plain: Some("The agenda is attached.".to_string()),
                body_html: None,
                read: false,
                starred: false,
                deleted: false,
                safety: crate::service::safety::Safety::Ordinary,
            })
            .expect("the row");
        cache
            .conn
            .execute(
                "UPDATE messages SET has_attachments = 1 WHERE id = ?1",
                [row],
            )
            .expect("the bit the header sync writes");
        row
    }

    fn a_part_recorded(cache: &MessageCache, row: i64) {
        cache
            .replace_attachments_with_content(
                row,
                &[AttachmentWithContent::described_only(CachedAttachment {
                    id: 0,
                    message_id: row,
                    filename: "agenda.pdf".to_string(),
                    mime_type: "application/pdf".to_string(),
                    size: 18,
                    content_id: None,
                    description: WhatTheSenderSaid::Nothing,
                })],
            )
            .expect("the part recorded");
    }

    #[test]
    fn test_a_message_saying_it_carries_attachments_with_none_recorded_was_left_behind() {
        let cache = a_cache();
        let left = left_behind(&cache, 1);
        let kept = left_behind(&cache, 2);
        a_part_recorded(&cache, kept);

        assert!(cache.parts_were_left_behind(left).expect("asked"));
        assert!(
            !cache.parts_were_left_behind(kept).expect("asked"),
            "a message whose parts are recorded is fetched again"
        );
    }

    #[test]
    fn test_a_message_filed_here_is_never_left_behind() {
        // This computer has the only copy of a message filed here, so there
        // is no server to fetch its parts from.
        let cache = a_cache();
        let on_the_server = left_behind(&cache, 1);
        let filed_here = left_behind(&cache, 2);
        cache
            .conn
            .execute(
                "UPDATE messages SET filed_here = 1 WHERE id = ?1",
                [filed_here],
            )
            .expect("filed here");

        assert!(cache.parts_were_left_behind(on_the_server).expect("asked"));
        assert!(
            !cache.parts_were_left_behind(filed_here).expect("asked"),
            "a message only this computer holds would be fetched from a server"
        );
    }

    #[test]
    fn test_a_missing_row_was_not_left_behind() {
        let cache = a_cache();
        let there = left_behind(&cache, 1);

        assert!(cache.parts_were_left_behind(there).expect("asked"));
        assert!(!cache.parts_were_left_behind(there + 1).expect("asked"));
    }

    #[test]
    fn test_a_message_recorded_as_carrying_no_attachments_is_no_longer_left_behind() {
        let cache = a_cache();
        let corrected = left_behind(&cache, 1);
        let beside = left_behind(&cache, 2);

        cache
            .record_that_it_carries_no_attachments(corrected)
            .expect("recorded");

        assert!(!cache.parts_were_left_behind(corrected).expect("asked"));
        assert!(
            cache.parts_were_left_behind(beside).expect("asked"),
            "the correction reached a message it was not about"
        );
    }

    #[test]
    fn test_the_account_a_message_is_in_is_its_folders() {
        let cache = a_cache();
        let other = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acc-2".to_string(),
                name: "INBOX".to_string(),
                path: "INBOX".to_string(),
                folder_type: "Inbox".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a second account's inbox");
        let first = left_behind(&cache, 1);
        let second = left_behind(&cache, 2);
        cache
            .conn
            .execute(
                "UPDATE messages SET folder_id = ?2 WHERE id = ?1",
                [second, other],
            )
            .expect("moved to the second account");

        assert_eq!(
            cache.the_account_a_message_is_in(first).expect("asked"),
            Some("acc-1".to_string())
        );
        assert_eq!(
            cache.the_account_a_message_is_in(second).expect("asked"),
            Some("acc-2".to_string())
        );
        assert_eq!(
            cache
                .the_account_a_message_is_in(second + 1)
                .expect("asked"),
            None
        );
    }
}

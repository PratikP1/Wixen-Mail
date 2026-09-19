//! The store's half of giving mail already stored the server's word (#88).
//!
//! A message stored before 2026-09-19 on a Gmail account has no
//! `server_thread_id`, and a check never fetches a stored message again, so
//! without a pass the tester's 17,753 messages would stay threaded by their
//! headers and the fix would show only on new mail. The pass itself, which
//! asks the server, is [`crate::application::server_thread_ids`]; this is
//! what it asks the store: which rows still want a word, per kept folder,
//! and the write that gives a row one.
//!
//! # Why the write goes through the merge
//!
//! Giving a row the server's word is not a column update. The row's chain
//! may have joined it to siblings that are still header-named, and those
//! settle under the server's word through the same
//! [`crate::application::thread_identity::rejoin`] an arrival goes through,
//! so a conversation is named one way whether its word arrived with the
//! message or a day later. A sibling the server filed elsewhere keeps its
//! own word, because a name the server gave is never rewritten.
//!
//! The write is one row at a time on purpose. Rewriting a whole
//! header-named conversation onto the first word found would move a sibling
//! Gmail files elsewhere; each row gets its own word when its turn comes,
//! and the end state is the same whatever order the server answered in.

use super::MessageCache;
use crate::common::{Error, Result};
use rusqlite::params;

/// The stored rows of one folder that have no word from the server yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowsWithoutTheServersWord {
    /// The folder as the server names it, which is what a fetch is asked in.
    pub folder_path: String,
    /// Each row and the number the server knows it by, by number.
    pub rows: Vec<(i64, u32)>,
}

impl MessageCache {
    /// Whether a named piece of once-only work has been recorded as done.
    pub fn has_this_been_done(&self, name: &str) -> Result<bool> {
        self.conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM work_done_once WHERE name = ?1)",
                params![name],
                |row| row.get(0),
            )
            .map_err(|e| Error::Other(format!("Failed to ask whether {name} was done: {e}")))
    }

    /// Record a named piece of once-only work as done, now.
    ///
    /// Written only after the work finished, by the caller, so work that
    /// failed halfway is tried again and a row done twice is done the same
    /// way.
    pub fn record_this_as_done(&self, name: &str) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO work_done_once (name, done_at) VALUES (?1, ?2)",
                params![name, chrono::Utc::now().to_rfc3339()],
            )
            .map_err(|e| Error::Other(format!("Failed to record that {name} was done: {e}")))?;
        Ok(())
    }

    /// The account's stored rows that have no word from the server, by the
    /// folder the server holds them in.
    ///
    /// Rows this program filed here are left out: the number they carry was
    /// reserved here and names nothing at the server. One folder per entry,
    /// in path order, the rows by number, so the pass asks each folder once
    /// and a test can say what it asked.
    pub fn rows_without_the_servers_word(
        &self,
        account_id: &str,
    ) -> Result<Vec<RowsWithoutTheServersWord>> {
        let mut statement = self
            .conn
            .prepare_cached(
                "SELECT f.path, m.id, m.uid
                 FROM messages m
                 INNER JOIN folders f ON m.folder_id = f.id
                 WHERE f.account_id = ?1
                   AND m.server_thread_id IS NULL
                   AND m.filed_here = 0
                 ORDER BY f.path, m.uid",
            )
            .map_err(|e| Error::Other(format!("Failed to look for rows without a word: {e}")))?;
        let rows = statement
            .query_map(params![account_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, u32>(2)?,
                ))
            })
            .map_err(|e| Error::Other(format!("Failed to read rows without a word: {e}")))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to read a row without a word: {e}")))?;

        let mut by_folder: Vec<RowsWithoutTheServersWord> = Vec::new();
        for (path, id, uid) in rows {
            match by_folder.last_mut() {
                Some(folder) if folder.folder_path == path => folder.rows.push((id, uid)),
                _ => by_folder.push(RowsWithoutTheServersWord {
                    folder_path: path,
                    rows: vec![(id, uid)],
                }),
            }
        }
        Ok(by_folder)
    }

    /// Give a stored row the server's word for its conversation.
    ///
    /// `word` is the name as
    /// [`crate::application::thread_identity::the_servers_name`] spells it.
    /// The row is filed under it, and whatever its chain connects is
    /// rejoined under it through the one merge, as an arrival is. Returns how
    /// many other messages moved.
    pub fn name_the_conversation_after_the_server(&self, row: i64, word: &str) -> Result<usize> {
        let (folder_id, message_id, refs_header): (i64, String, Option<String>) = self
            .conn
            .query_row(
                "SELECT folder_id, message_id, refs_header FROM messages WHERE id = ?1",
                params![row],
                |found| Ok((found.get(0)?, found.get(1)?, found.get(2)?)),
            )
            .map_err(|e| Error::Other(format!("Failed to read the message to name: {e}")))?;
        // The row's own name first, and the merge after, so the merge finds
        // the row already under the word and moves the rest of its old
        // conversation to it rather than the other way round.
        self.conn
            .execute(
                "UPDATE messages SET server_thread_id = ?1, thread_id = ?1 WHERE id = ?2",
                params![word, row],
            )
            .map_err(|e| Error::Other(format!("Failed to name the conversation: {e}")))?;
        self.merge_what_this_row_connects(row, folder_id, &message_id, refs_header.as_deref(), word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::thread_identity::the_servers_name;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{CachedFolder, IncomingMessage};

    const THE_ACCOUNT: &str = "acct-gmail";

    fn a_cache() -> TempHome<MessageCache> {
        TempHome::new(|dir| MessageCache::new(dir.to_path_buf(), None).expect("a cache"))
    }

    fn a_folder(cache: &MessageCache, account: &str, path: &str) -> i64 {
        cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: account.to_string(),
                name: path.to_string(),
                path: path.to_string(),
                folder_type: "Custom".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder")
    }

    fn arriving(folder_id: i64, uid: u32, message_id: &str, chain: &str) -> IncomingMessage {
        IncomingMessage {
            folder_id,
            uid,
            message_id: message_id.to_string(),
            subject: "Re: the figures".to_string(),
            from_addr: "ada@example.com".to_string(),
            to_addr: "me@example.com".to_string(),
            cc: None,
            reply_to: None,
            date: format!("2026-09-{uid:02}T09:00:00Z"),
            internal_date: None,
            size_bytes: Some(100),
            refs_header: Some(chain.to_string()).filter(|chain| !chain.is_empty()),
            read: false,
            starred: false,
            answered: false,
            draft: false,
            deleted: false,
            has_attachments: false,
            safety: crate::service::safety::Verdict::ordinary(),
            gmail_message_id: None,
            server_thread_id: None,
            labels: None,
            receipt_to: None,
            list_unsubscribe: None,
            pop_uidl: None,
        }
    }

    fn the_word_of(cache: &MessageCache, row: i64) -> (Option<String>, Option<String>) {
        cache
            .conn
            .query_row(
                "SELECT server_thread_id, thread_id FROM messages WHERE id = ?1",
                params![row],
                |found| Ok((found.get(0)?, found.get(1)?)),
            )
            .expect("the row read back")
    }

    #[test]
    fn test_the_rows_without_a_word_are_listed_by_folder_and_number() {
        let cache = a_cache();
        let inbox = a_folder(&cache, THE_ACCOUNT, "INBOX");
        let archive = a_folder(&cache, THE_ACCOUNT, "Archive");
        let elsewhere = a_folder(&cache, "another account", "INBOX");

        let in_archive = cache
            .upsert_message(&arriving(archive, 7, "x@x", ""))
            .expect("stored");
        let later = cache
            .upsert_message(&arriving(inbox, 3, "b@x", ""))
            .expect("stored");
        let earlier = cache
            .upsert_message(&arriving(inbox, 2, "a@x", ""))
            .expect("stored");
        // One already named, one filed here, one of another account: none
        // of them wanted.
        let named = cache
            .upsert_message(&IncomingMessage {
                server_thread_id: Some(the_servers_name(5)),
                ..arriving(inbox, 4, "c@x", "")
            })
            .expect("stored");
        cache
            .file_message_here(&arriving(inbox, 900, "d@x", ""))
            .expect("filed here");
        cache
            .upsert_message(&arriving(elsewhere, 1, "e@x", ""))
            .expect("stored");

        let listed = cache
            .rows_without_the_servers_word(THE_ACCOUNT)
            .expect("listed");

        assert_eq!(
            listed,
            vec![
                RowsWithoutTheServersWord {
                    folder_path: "Archive".to_string(),
                    rows: vec![(in_archive, 7)],
                },
                RowsWithoutTheServersWord {
                    folder_path: "INBOX".to_string(),
                    rows: vec![(earlier, 2), (later, 3)],
                },
            ]
        );
        assert!(
            !listed
                .iter()
                .any(|folder| folder.rows.iter().any(|(row, _)| *row == named)),
            "a row the server already named was listed"
        );
    }

    #[test]
    fn test_naming_a_row_writes_the_word_and_files_the_row_under_it() {
        let cache = a_cache();
        let inbox = a_folder(&cache, THE_ACCOUNT, "INBOX");
        let row = cache
            .upsert_message(&arriving(inbox, 1, "a@x", ""))
            .expect("stored");
        assert_eq!(the_word_of(&cache, row), (None, Some("a@x".to_string())));

        cache
            .name_the_conversation_after_the_server(row, &the_servers_name(9))
            .expect("named");

        assert_eq!(
            the_word_of(&cache, row),
            (Some("gm:9".to_string()), Some("gm:9".to_string()))
        );
        assert!(
            cache
                .rows_without_the_servers_word(THE_ACCOUNT)
                .expect("listed")
                .is_empty(),
            "a named row still wants a word"
        );
    }

    #[test]
    fn test_naming_a_reply_brings_its_header_named_conversation_under_the_word() {
        // A and B stored by their headers under a@x; B's word arrives. Both
        // settle under it, because a chain that joined them still does and
        // the server's word wins the merge.
        let cache = a_cache();
        let inbox = a_folder(&cache, THE_ACCOUNT, "INBOX");
        let a = cache
            .upsert_message(&arriving(inbox, 1, "a@x", ""))
            .expect("stored");
        let b = cache
            .upsert_message(&arriving(inbox, 2, "b@x", "a@x"))
            .expect("stored");

        let moved = cache
            .name_the_conversation_after_the_server(b, &the_servers_name(9))
            .expect("named");

        assert_eq!(moved, 1, "A did not move under B's word");
        assert_eq!(the_word_of(&cache, a).1.as_deref(), Some("gm:9"));
        assert_eq!(the_word_of(&cache, b).1.as_deref(), Some("gm:9"));
        assert_eq!(
            the_word_of(&cache, a).0,
            None,
            "A was given a word nobody sent for it"
        );
    }

    #[test]
    fn test_a_sibling_the_server_filed_elsewhere_keeps_its_own_word_whatever_order_the_words_came()
    {
        // Case (d)'s stored-rows variant, with case (e)'s twist: A and B
        // under a@x by their headers; B's word comes first and takes A with
        // it; then A's own word comes and A moves out again, and B stays
        // where the server put it. The other order ends the same way.
        for b_first in [true, false] {
            let cache = a_cache();
            let inbox = a_folder(&cache, THE_ACCOUNT, "INBOX");
            let a = cache
                .upsert_message(&arriving(inbox, 1, "a@x", ""))
                .expect("stored");
            let b = cache
                .upsert_message(&arriving(inbox, 2, "b@x", "a@x"))
                .expect("stored");

            let mut in_order = vec![(a, 8u64), (b, 9u64)];
            if b_first {
                in_order.reverse();
            }
            for (row, word) in in_order {
                cache
                    .name_the_conversation_after_the_server(row, &the_servers_name(word))
                    .expect("named");
            }

            assert_eq!(
                the_word_of(&cache, a),
                (Some("gm:8".to_string()), Some("gm:8".to_string())),
                "b first: {b_first}"
            );
            assert_eq!(
                the_word_of(&cache, b),
                (Some("gm:9".to_string()), Some("gm:9".to_string())),
                "b first: {b_first}"
            );
        }
    }

    #[test]
    fn test_once_only_work_is_not_done_until_it_is_recorded_and_then_stays_done() {
        let cache = a_cache();
        assert!(!cache.has_this_been_done("the pass").expect("asked"));
        cache.record_this_as_done("the pass").expect("recorded");
        assert!(cache.has_this_been_done("the pass").expect("asked"));
        cache
            .record_this_as_done("the pass")
            .expect("recorded twice, harmlessly");
        assert!(!cache.has_this_been_done("another pass").expect("asked"));
    }
}

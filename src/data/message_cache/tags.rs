//! Tag and message-tag junction persistence operations

#[cfg(test)]
use super::CachedMessage;
use super::{MessageCache, Tag};
use crate::common::{Error, Result};
use rusqlite::{OptionalExtension, params};

impl MessageCache {
    /// Create a new tag, after every label its account already has.
    pub fn create_tag(&self, tag: &Tag) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO tags (id, account_id, name, color, created_at, keyword, position)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6,
                     (SELECT COALESCE(MAX(position), 0) + 1 FROM tags WHERE account_id = ?2))",
                params![
                    &tag.id,
                    &tag.account_id,
                    &tag.name,
                    &tag.color,
                    &tag.created_at,
                    &tag.keyword
                ],
            )
            .map_err(|e| Error::Other(format!("Failed to create tag: {}", e)))?;
        Ok(())
    }

    /// Make a message's labels match the keywords the server reports.
    ///
    /// The server is right. A label put on elsewhere arrives, one taken off
    /// elsewhere goes, and a keyword from a client this knows nothing about is
    /// left alone rather than turned into a label nobody made.
    ///
    /// Only labels that have a keyword take part. One that stays on this
    /// computer is not touched by what the server does or does not say about
    /// it, because the server was never told about it in the first place.
    pub fn match_labels_to_keywords(
        &self,
        message_id: i64,
        account_id: &str,
        keywords: &[String],
    ) -> Result<usize> {
        let ours = self.get_tags_for_account(account_id)?;
        let on_it: Vec<String> = self
            .get_tags_for_message(message_id)?
            .into_iter()
            .map(|tag| tag.id)
            .collect();

        let mut changed = 0;
        for tag in ours.iter() {
            let Some(keyword) = tag.keyword.as_deref() else {
                continue;
            };
            let server_has_it = keywords.iter().any(|held| held == keyword);
            let we_have_it = on_it.iter().any(|held| held == &tag.id);
            if server_has_it == we_have_it {
                continue;
            }
            if server_has_it {
                self.add_tag_to_message(message_id, &tag.id)?;
            } else {
                self.remove_tag_from_message(message_id, &tag.id)?;
            }
            changed += 1;
        }
        Ok(changed)
    }

    /// An account's labels, in the order the person keeps them.
    ///
    /// The one order: the Label menu shows it, the number keys apply by it,
    /// and the sidebar and the Label Manager list by it (#48). By name until
    /// 2026-09-24, which is why the pass that numbers rows from earlier
    /// builds numbers them in name order.
    pub fn get_tags_for_account(&self, account_id: &str) -> Result<Vec<Tag>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id, account_id, name, color, created_at, keyword
             FROM tags WHERE account_id = ?1 ORDER BY position, name",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let tags = stmt
            .query_map(params![account_id], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    name: row.get(2)?,
                    color: row.get(3)?,
                    created_at: row.get(4)?,
                    keyword: row.get(5)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query tags: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect tags: {}", e)))?;
        Ok(tags)
    }

    /// Write an account's labels in the order given, the first at one.
    ///
    /// The whole order rather than the two that swapped, for the reason
    /// `reordering::Moved::order` gives. One transaction, so a write that
    /// fails part way leaves the order it found.
    pub fn put_labels_in_order(&self, account_id: &str, ids: &[String]) -> Result<()> {
        let failed = |e: rusqlite::Error| Error::Other(format!("Failed to order labels: {}", e));
        let writing = self.conn.unchecked_transaction().map_err(failed)?;
        for (at, id) in ids.iter().enumerate() {
            writing
                .execute(
                    "UPDATE tags SET position = ?1 WHERE id = ?2 AND account_id = ?3",
                    params![at as i64 + 1, id, account_id],
                )
                .map_err(failed)?;
        }
        writing.commit().map_err(failed)
    }

    /// Give every label with no place in its account's order one, after the
    /// labels that have one, in name order.
    ///
    /// Run on every open rather than once under a marker: a label written by
    /// a build before 2026-09-24 has no place, and neither does one written by
    /// such a build to a database this one has opened, and either way the
    /// answer is the same, so there is nothing a marker would decide. Name
    /// order because that is the order those builds listed an account's
    /// labels in and the order their keys applied them, so Ctrl+2 goes on
    /// applying what it applied. One transaction.
    pub fn number_the_unnumbered_labels(&self) -> Result<()> {
        let failed = |e: rusqlite::Error| Error::Other(format!("Failed to number labels: {}", e));
        let numbering = self.conn.unchecked_transaction().map_err(failed)?;
        let unnumbered: Vec<(String, String)> = numbering
            .prepare("SELECT id, account_id FROM tags WHERE position IS NULL ORDER BY account_id, name, id")
            .map_err(failed)?
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(failed)?
            .collect::<std::result::Result<_, _>>()
            .map_err(failed)?;
        for (id, account_id) in unnumbered {
            numbering
                .execute(
                    "UPDATE tags SET position =
                         (SELECT COALESCE(MAX(position), 0) + 1 FROM tags WHERE account_id = ?1)
                     WHERE id = ?2",
                    params![account_id, id],
                )
                .map_err(failed)?;
        }
        numbering.commit().map_err(failed)
    }

    /// Get a specific tag by ID
    pub fn get_tag(&self, tag_id: &str) -> Result<Option<Tag>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT id, account_id, name, color, created_at, keyword FROM tags WHERE id = ?1",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let tag = stmt
            .query_row(params![tag_id], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    name: row.get(2)?,
                    color: row.get(3)?,
                    created_at: row.get(4)?,
                    keyword: row.get(5)?,
                })
            })
            .optional()
            .map_err(|e| Error::Other(format!("Failed to get tag: {}", e)))?;
        Ok(tag)
    }

    /// Update a tag, and say how many rows that touched.
    ///
    /// The count matters. Updating a row that is not there is not an error in
    /// SQL, so a caller that writes by trying an update and creating only when
    /// that fails would never create anything, and every label made in the
    /// manager was silently dropped that way.
    pub fn update_tag(&self, tag: &Tag) -> Result<usize> {
        let touched = self
            .conn
            .execute(
                // The keyword is left alone. It is what this label was already
                // sent to the server under, and rewriting it when somebody
                // renames the label would orphan every message already
                // carrying it: the old keyword would stay on the server with
                // nothing here recognising it.
                "UPDATE tags SET name = ?1, color = ?2 WHERE id = ?3",
                params![&tag.name, &tag.color, &tag.id],
            )
            .map_err(|e| Error::Other(format!("Failed to update tag: {}", e)))?;
        Ok(touched)
    }

    /// Delete a tag
    pub fn delete_tag(&self, tag_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM tags WHERE id = ?1", params![tag_id])
            .map_err(|e| Error::Other(format!("Failed to delete tag: {}", e)))?;
        Ok(())
    }

    /// Add a tag to a message
    pub fn add_tag_to_message(&self, message_id: i64, tag_id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT OR IGNORE INTO message_tags (message_id, tag_id, created_at)
             VALUES (?1, ?2, ?3)",
                params![message_id, tag_id, now],
            )
            .map_err(|e| Error::Other(format!("Failed to add tag to message: {}", e)))?;
        Ok(())
    }

    /// Remove a tag from a message
    pub fn remove_tag_from_message(&self, message_id: i64, tag_id: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM message_tags WHERE message_id = ?1 AND tag_id = ?2",
                params![message_id, tag_id],
            )
            .map_err(|e| Error::Other(format!("Failed to remove tag from message: {}", e)))?;
        Ok(())
    }

    /// Get all tags for a message
    pub fn get_tags_for_message(&self, message_id: i64) -> Result<Vec<Tag>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT t.id, t.account_id, t.name, t.color, t.created_at, t.keyword
             FROM tags t
             INNER JOIN message_tags mt ON t.id = mt.tag_id
             WHERE mt.message_id = ?1
             ORDER BY t.name",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let tags = stmt
            .query_map(params![message_id], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    name: row.get(2)?,
                    color: row.get(3)?,
                    created_at: row.get(4)?,
                    keyword: row.get(5)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query message tags: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect message tags: {}", e)))?;
        Ok(tags)
    }

    /// The labels of every message in a folder, by message id.
    ///
    /// One query joined through `messages.folder_id`, with one bound
    /// parameter however many rows the folder holds. What `attach_labels`
    /// asks since 2026-09-17. Before that it asked `get_tags_for_messages`
    /// over every id the list had read, which sent one parameter per row
    /// through an `IN (?1, ?2, ...)`, and the bundled SQLite refuses more
    /// than 32,766 of them, so a folder past that size showed no labels at
    /// all; the row that refused, at 200,000, is on
    /// `docs/development/measurements.md`. That read had no caller left and
    /// went with the change. A message with no label is absent from the map,
    /// so a caller reads `by_message.get(&id)` and treats `None` as an empty
    /// list.
    pub fn tags_by_message_in_folder(
        &self,
        folder_id: i64,
    ) -> Result<std::collections::HashMap<i64, Vec<Tag>>> {
        self.tags_by_message_where("m.folder_id = ?1", params![folder_id])
    }

    /// The labels of every message in an account, by message id.
    ///
    /// The same shape as [`Self::tags_by_message_in_folder`], for the label view,
    /// which lists one account's mail across its folders.
    pub fn tags_by_message_in_account(
        &self,
        account_id: &str,
    ) -> Result<std::collections::HashMap<i64, Vec<Tag>>> {
        self.tags_by_message_where("f.account_id = ?1", params![account_id])
    }

    /// The labels of every message in every inbox, by message id.
    ///
    /// The same shape again, for All Inboxes, which spans every account and
    /// so cannot ask [`Self::tags_by_message_in_account`]; the `WHERE` is the one
    /// `unified_inbox` lists by.
    pub fn tags_by_message_in_every_inbox(
        &self,
    ) -> Result<std::collections::HashMap<i64, Vec<Tag>>> {
        self.tags_by_message_where("f.folder_type = 'Inbox'", params![])
    }

    /// Every label on every message the clause selects, grouped by message.
    ///
    /// `narrowed_to` must be one of the fixed strings the three readers above
    /// pass; nothing a user typed reaches it, which is what makes writing it
    /// into the query safe. Grouped here in Rust rather than in SQL, because
    /// a message id repeats as a key across several rows and SQLite has no
    /// map type to build one into directly.
    fn tags_by_message_where(
        &self,
        narrowed_to: &str,
        bound: impl rusqlite::Params,
    ) -> Result<std::collections::HashMap<i64, Vec<Tag>>> {
        let mut stmt = self
            .conn
            .prepare_cached(&format!(
                "SELECT mt.message_id, t.id, t.account_id, t.name, t.color, t.created_at, t.keyword
                 FROM message_tags mt
                 INNER JOIN tags t ON t.id = mt.tag_id
                 INNER JOIN messages m ON m.id = mt.message_id
                 INNER JOIN folders f ON f.id = m.folder_id
                 WHERE {narrowed_to} AND m.deleted = 0
                 ORDER BY mt.message_id, t.name"
            ))
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let rows = stmt
            .query_map(bound, |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    Tag {
                        id: row.get(1)?,
                        account_id: row.get(2)?,
                        name: row.get(3)?,
                        color: row.get(4)?,
                        created_at: row.get(5)?,
                        keyword: row.get(6)?,
                    },
                ))
            })
            .map_err(|e| Error::Other(format!("Failed to query message tags: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect message tags: {}", e)))?;

        let mut by_message: std::collections::HashMap<i64, Vec<Tag>> =
            std::collections::HashMap::new();
        for (message_id, tag) in rows {
            by_message.entry(message_id).or_default().push(tag);
        }
        Ok(by_message)
    }

    /// The messages carrying a label, as the list draws them.
    ///
    /// The same row a folder listing produces, and deliberately so: the mail
    /// list has one shape, and a second one would mean a label view missing
    /// the snippet, the size and the attachment marker that every other view
    /// of the same messages shows.
    ///
    /// `order_by` is the sort that was chosen, in the query rather than
    /// applied to the rows afterwards, for the reason
    /// [`MessageCache::get_message_list_sorted`] gives; it must come from
    /// `Sort::order_by_clause`, fixed strings chosen by matching on an enum,
    /// and nothing a person typed reaches it. Since 2026-09-17 (#69): until
    /// then a label view read a fixed newest-first order while a folder read
    /// the chosen one. `None` is newest first by the sent date, what a
    /// person who never chose a sort reads.
    ///
    /// `None` for the limit is every message carrying the label, which is
    /// what the window asks for since 2026-09-17; it asked for the newest 500
    /// until then, the same page the folder list read through (#24).
    ///
    /// Replaced `get_messages_by_tag`, which answered with a different shape,
    /// read body text out of the columns it stopped being written to, and had
    /// no caller at all outside its own test: nothing in the application
    /// could ever ask to see the mail carrying a label.
    pub fn messages_with_label(
        &self,
        account_id: &str,
        tag_id: &str,
        order_by: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<super::MessageListRow>> {
        let order = order_by.unwrap_or(super::messages::NEWEST_MESSAGE_FIRST);
        let query = label_listing_query(order, limit);
        let mut stmt = self
            .conn
            .prepare_cached(&query)
            .map_err(|e| Error::Other(format!("Failed to prepare the label listing: {}", e)))?;

        stmt.query_map(params![tag_id, account_id], super::messages::listing_row)
            .map_err(|e| Error::Other(format!("Failed to list messages with a label: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to read a labelled message: {}", e)))
    }
}

/// The query a label view runs, in one place.
///
/// Built here rather than inline for the reason `listing_query` is: a test
/// asks SQLite whether this exact query reads message text or a table a
/// listing may not, in every order it can be asked for, and a copy held in
/// the test is the copy that goes stale. The same columns in the same order
/// as a folder listing, because `listing_row` reads them.
///
/// `order` must come from `Sort::order_by_clause` or be
/// `NEWEST_MESSAGE_FIRST`; nothing a person typed reaches it. The uid
/// follows it as the tie-break, so rows sharing a timestamp do not shuffle
/// between refreshes.
pub(super) fn label_listing_query(order: &str, limit: Option<usize>) -> String {
    format!(
        "SELECT m.id, m.uid, f.account_id, m.message_id, m.refs_header, m.subject, m.from_addr,
                m.to_addr, m.cc, m.reply_to, m.date, m.snippet, m.size_bytes,
                m.read, m.starred, m.answered, m.draft,
                (m.has_attachments = 1
                 OR EXISTS(SELECT 1 FROM attachments a WHERE a.message_id = m.id)),
                m.safety, m.safety_reasons, m.receipt_to, m.list_unsubscribe, m.thread_id,
                m.says_first
         FROM messages m
         INNER JOIN message_tags mt ON m.id = mt.message_id
         INNER JOIN folders f ON m.folder_id = f.id
         WHERE mt.tag_id = ?1 AND f.account_id = ?2 AND m.deleted = 0
         ORDER BY {order}, m.uid DESC{}",
        super::messages::limit_clause(limit)
    )
}

#[cfg(test)]
mod keyword_tests {
    use super::super::{CachedFolder, IncomingMessage, MessageCache, Tag};
    use crate::common::temp_home::TempHome;

    fn cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_kw_", |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache")
        })
    }

    fn a_message(cache: &MessageCache) -> i64 {
        let folder_id = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acct".into(),
                name: "INBOX".into(),
                path: "INBOX".into(),
                folder_type: "Inbox".into(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder");
        cache
            .upsert_message(&IncomingMessage {
                folder_id,
                uid: 1,
                message_id: "m1@example.com".into(),
                subject: "Quarterly report".into(),
                from_addr: "ada@example.com".into(),
                to_addr: "me@example.com".into(),
                cc: None,
                reply_to: None,
                date: "2026-08-01T09:00:00+00:00".into(),
                internal_date: None,
                size_bytes: None,
                refs_header: None,
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
            })
            .expect("a message")
    }

    fn a_label(cache: &MessageCache, name: &str, keyword: Option<&str>) -> String {
        let id = format!("acct:{name}");
        cache
            .create_tag(&Tag {
                id: id.clone(),
                account_id: "acct".into(),
                name: name.into(),
                color: "#FF0000".into(),
                created_at: "2026-08-01T00:00:00Z".into(),
                keyword: keyword.map(str::to_string),
            })
            .expect("a label");
        id
    }

    #[test]
    fn test_updating_a_label_that_is_not_there_says_so() {
        // The manager writes a new label by trying an update and creating only
        // if that failed. An update of a row that does not exist is not a
        // failure in SQL, so it reported success, the create never ran, and
        // every label made in the manager was silently dropped.
        let cache = cache();

        let touched = cache
            .update_tag(&Tag {
                id: "acct:nothing".into(),
                account_id: "acct".into(),
                name: "Nothing".into(),
                color: "#000000".into(),
                created_at: "2026-08-01T00:00:00Z".into(),
                keyword: None,
            })
            .expect("the update runs");

        assert_eq!(
            touched, 0,
            "an update of a missing row claimed to change one"
        );
    }

    #[test]
    fn test_a_label_set_on_another_device_arrives() {
        // The whole reason keywords are sent at all. Without this a label put
        // on from a phone never showed up here.
        let cache = cache();
        let message = a_message(&cache);
        a_label(&cache, "Work", Some("$label2"));

        let changed = cache
            .match_labels_to_keywords(message, "acct", &["$label2".to_string()])
            .expect("reconciled");

        assert_eq!(changed, 1);
        let names: Vec<String> = cache
            .get_tags_for_message(message)
            .expect("read back")
            .into_iter()
            .map(|tag| tag.name)
            .collect();
        assert_eq!(names, vec!["Work"]);
    }

    #[test]
    fn test_a_label_taken_off_elsewhere_comes_off_here() {
        let cache = cache();
        let message = a_message(&cache);
        let work = a_label(&cache, "Work", Some("$label2"));
        cache.add_tag_to_message(message, &work).expect("labelled");

        let changed = cache
            .match_labels_to_keywords(message, "acct", &[])
            .expect("reconciled");

        assert_eq!(changed, 1);
        assert!(
            cache
                .get_tags_for_message(message)
                .expect("read")
                .is_empty()
        );
    }

    #[test]
    fn test_a_keyword_from_another_client_is_not_turned_into_a_label() {
        // A mailbox carries keywords from everything that has touched it, and
        // inventing labels from them would fill somebody's list with names
        // they never chose.
        let cache = cache();
        let message = a_message(&cache);
        a_label(&cache, "Work", Some("$label2"));

        let changed = cache
            .match_labels_to_keywords(message, "acct", &["$MailFlagBit0".to_string()])
            .expect("reconciled");

        assert_eq!(changed, 0);
        assert!(
            cache
                .get_tags_for_message(message)
                .expect("read")
                .is_empty()
        );
    }

    #[test]
    fn test_a_label_that_stays_on_this_computer_is_left_alone() {
        // It was never sent, so the server saying nothing about it is not the
        // server saying it was removed.
        let cache = cache();
        let message = a_message(&cache);
        let local = a_label(&cache, "!!!", None);
        cache.add_tag_to_message(message, &local).expect("labelled");

        let changed = cache
            .match_labels_to_keywords(message, "acct", &[])
            .expect("reconciled");

        assert_eq!(changed, 0, "a label with no keyword was taken off");
        assert_eq!(cache.get_tags_for_message(message).expect("read").len(), 1);
    }

    #[test]
    fn test_reconciling_twice_changes_nothing_the_second_time() {
        let cache = cache();
        let message = a_message(&cache);
        a_label(&cache, "Work", Some("$label2"));
        let keywords = vec!["$label2".to_string()];

        cache
            .match_labels_to_keywords(message, "acct", &keywords)
            .expect("first");
        let again = cache
            .match_labels_to_keywords(message, "acct", &keywords)
            .expect("second");

        assert_eq!(again, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::message_cache::CachedFolder;

    #[test]
    fn test_tag_operations() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();

        let tag = Tag {
            id: "tag-work".to_string(),
            account_id: "test@example.com".to_string(),
            name: "Work".to_string(),
            color: "#FF0000".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            keyword: None,
        };

        cache.create_tag(&tag).unwrap();

        let loaded_tag = cache.get_tag("tag-work").unwrap();
        assert!(loaded_tag.is_some());
        assert_eq!(loaded_tag.unwrap().name, "Work");

        let tags = cache.get_tags_for_account("test@example.com").unwrap();
        assert_eq!(tags.len(), 1);

        let mut updated_tag = tag.clone();
        updated_tag.name = "Work Projects".to_string();
        updated_tag.color = "#00FF00".to_string();
        cache.update_tag(&updated_tag).unwrap();

        let loaded = cache.get_tag("tag-work").unwrap().unwrap();
        assert_eq!(loaded.name, "Work Projects");
        assert_eq!(loaded.color, "#00FF00");

        cache.delete_tag("tag-work").unwrap();
        let deleted = cache.get_tag("tag-work").unwrap();
        assert!(deleted.is_none());
    }

    #[test]
    fn test_message_tagging() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();

        let folder = CachedFolder {
            id: 0,
            account_id: "test@example.com".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "inbox".to_string(),
            unread_count: 0,
            total_count: 0,
        };
        let folder_id = cache.save_folder(&folder).unwrap();

        let message = CachedMessage {
            id: 0,
            uid: 1,
            folder_id,
            message_id: "msg-1@example.com".to_string(),
            subject: "Test Message".to_string(),
            from_addr: "sender@example.com".to_string(),
            to_addr: "recipient@example.com".to_string(),
            cc: None,
            date: chrono::Utc::now().to_rfc3339(),
            body_plain: Some("Test body".to_string()),
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
            safety: crate::service::safety::Safety::Ordinary,
        };
        let message_id = cache.save_message(&message).unwrap();

        let tag1 = Tag {
            id: "tag-important".to_string(),
            account_id: "test@example.com".to_string(),
            name: "Important".to_string(),
            color: "#FF0000".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            keyword: None,
        };
        let tag2 = Tag {
            id: "tag-personal".to_string(),
            account_id: "test@example.com".to_string(),
            name: "Personal".to_string(),
            color: "#00FF00".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            keyword: None,
        };
        cache.create_tag(&tag1).unwrap();
        cache.create_tag(&tag2).unwrap();

        cache
            .add_tag_to_message(message_id, "tag-important")
            .unwrap();
        cache
            .add_tag_to_message(message_id, "tag-personal")
            .unwrap();

        let message_tags = cache.get_tags_for_message(message_id).unwrap();
        assert_eq!(message_tags.len(), 2);

        let messages = cache
            .messages_with_label("test@example.com", "tag-important", None, None)
            .unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].subject, "Test Message");

        // Scoped to the account, the way every other listing here is. A label
        // belongs to an account, so a label id from one must never draw in
        // another's mail even if the ids were ever to collide.
        assert!(
            cache
                .messages_with_label("someone@else.example", "tag-important", None, Some(50))
                .unwrap()
                .is_empty(),
            "a label listing reached another account's mail"
        );

        cache
            .remove_tag_from_message(message_id, "tag-personal")
            .unwrap();
        let remaining_tags = cache.get_tags_for_message(message_id).unwrap();
        assert_eq!(remaining_tags.len(), 1);
        assert_eq!(remaining_tags[0].name, "Important");
    }

    #[test]
    fn test_the_labels_of_a_folder_keep_each_messages_own_apart() {
        // `attach_labels` used to call `get_tags_for_message` once per row,
        // then one query over every id, which SQLite refuses above 32,766 of
        // them. This is the read by folder it asks now, and the map it
        // builds has to keep each message's own tags separate from its
        // neighbours' rather than merging or swapping them.
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();
        let folder = CachedFolder {
            id: 0,
            account_id: "test@example.com".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "inbox".to_string(),
            unread_count: 0,
            total_count: 0,
        };
        let folder_id = cache.save_folder(&folder).unwrap();

        let save = |uid: u32, subject: &str| {
            cache
                .save_message(&CachedMessage {
                    id: 0,
                    uid,
                    folder_id,
                    message_id: format!("m{uid}@example.com"),
                    subject: subject.to_string(),
                    from_addr: "sender@example.com".to_string(),
                    to_addr: "recipient@example.com".to_string(),
                    cc: None,
                    date: chrono::Utc::now().to_rfc3339(),
                    body_plain: None,
                    body_html: None,
                    read: false,
                    starred: false,
                    deleted: false,
                    safety: crate::service::safety::Safety::Ordinary,
                })
                .unwrap()
        };
        let work_msg = save(1, "Budget");
        let personal_msg = save(2, "Dinner");
        let untagged_msg = save(3, "Newsletter");

        cache
            .create_tag(&Tag {
                id: "tag-work".to_string(),
                account_id: "test@example.com".to_string(),
                name: "Work".to_string(),
                color: "#FF0000".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                keyword: None,
            })
            .unwrap();
        cache
            .create_tag(&Tag {
                id: "tag-personal".to_string(),
                account_id: "test@example.com".to_string(),
                name: "Personal".to_string(),
                color: "#00FF00".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                keyword: None,
            })
            .unwrap();
        cache.add_tag_to_message(work_msg, "tag-work").unwrap();
        cache.add_tag_to_message(personal_msg, "tag-work").unwrap();
        cache
            .add_tag_to_message(personal_msg, "tag-personal")
            .unwrap();
        // untagged_msg is left with nothing on purpose.

        let by_message = cache.tags_by_message_in_folder(folder_id).unwrap();

        let names = |id: i64| -> Vec<String> {
            by_message
                .get(&id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|tag| tag.name)
                .collect()
        };
        assert_eq!(names(work_msg), vec!["Work".to_string()]);
        assert_eq!(
            names(personal_msg),
            vec!["Personal".to_string(), "Work".to_string()],
            "sorted by name, the same order get_tags_for_message already gives"
        );
        assert!(
            !by_message.contains_key(&untagged_msg),
            "an untagged message should not appear in the map at all"
        );
    }

    fn a_label_named(cache: &MessageCache, account_id: &str, name: &str) {
        cache
            .create_tag(&Tag {
                id: format!("{account_id}:{name}"),
                account_id: account_id.to_string(),
                name: name.to_string(),
                color: "#FF0000".to_string(),
                created_at: "2026-08-01T00:00:00Z".to_string(),
                keyword: None,
            })
            .expect("a label");
    }

    fn names_in_order(cache: &MessageCache, account_id: &str) -> Vec<String> {
        cache
            .get_tags_for_account(account_id)
            .expect("the labels to read")
            .into_iter()
            .map(|tag| tag.name)
            .collect()
    }

    #[test]
    fn test_labels_come_back_in_the_order_they_were_made() {
        // The order an account starts with is Thunderbird's, which is not
        // alphabetical, and a label made later goes after the rest (#48).
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();
        for name in [
            "Important",
            "Work",
            "Personal",
            "To Do",
            "Later",
            "Invoices",
        ] {
            a_label_named(&cache, "acct", name);
        }

        assert_eq!(
            names_in_order(&cache, "acct"),
            [
                "Important",
                "Work",
                "Personal",
                "To Do",
                "Later",
                "Invoices"
            ]
        );
    }

    #[test]
    fn test_the_order_written_is_the_order_read_back() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();
        for name in ["Important", "Work", "Later"] {
            a_label_named(&cache, "acct", name);
        }

        cache
            .put_labels_in_order(
                "acct",
                &[
                    "acct:Later".to_string(),
                    "acct:Important".to_string(),
                    "acct:Work".to_string(),
                ],
            )
            .expect("the order to be written");

        assert_eq!(
            names_in_order(&cache, "acct"),
            ["Later", "Important", "Work"]
        );
    }

    #[test]
    fn test_labels_from_before_are_numbered_in_the_order_they_were_listed() {
        // A build before this one listed an account's labels by name and its
        // keys applied them in that order, so that is the order they keep:
        // Ctrl+2 goes on applying what it applied. Per account, each from one.
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();
        for (account_id, name) in [
            ("acct", "Work"),
            ("acct", "Important"),
            ("acct", "Later"),
            ("other", "Zeta"),
        ] {
            cache
                .conn
                .execute(
                    "INSERT INTO tags (id, account_id, name, color, created_at)
                     VALUES (?1, ?2, ?3, '#FF0000', '2026-08-01T00:00:00Z')",
                    params![format!("{account_id}:{name}"), account_id, name],
                )
                .expect("a row written before labels had a place");
        }

        cache.number_the_unnumbered_labels().expect("the pass");

        let placed: Vec<(String, Option<i64>)> = cache
            .conn
            .prepare("SELECT name, position FROM tags ORDER BY account_id, name")
            .expect("a query")
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("the rows")
            .collect::<std::result::Result<_, _>>()
            .expect("every row");
        assert_eq!(
            placed,
            [
                ("Important".to_string(), Some(1)),
                ("Later".to_string(), Some(2)),
                ("Work".to_string(), Some(3)),
                ("Zeta".to_string(), Some(1)),
            ]
        );
    }

    #[test]
    fn test_a_folder_nobody_has_labelled_answers_an_empty_map() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();
        let folder_id = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "test@example.com".to_string(),
                name: "INBOX".to_string(),
                path: "INBOX".to_string(),
                folder_type: "inbox".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .unwrap();

        let by_message = cache.tags_by_message_in_folder(folder_id).unwrap();

        assert!(by_message.is_empty());
        assert!(cache.tags_by_message_in_every_inbox().unwrap().is_empty());
        assert!(
            cache
                .tags_by_message_in_account("test@example.com")
                .unwrap()
                .is_empty()
        );
    }
}

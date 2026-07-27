//! Folder persistence operations

use super::{CachedFolder, MessageCache};
use crate::common::{Error, Result};
use rusqlite::{OptionalExtension, params};

impl MessageCache {
    /// Save a folder to cache
    pub fn save_folder(&self, folder: &CachedFolder) -> Result<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO folders (account_id, name, path, folder_type, unread_count, total_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                folder.account_id,
                folder.name,
                folder.path,
                folder.folder_type,
                folder.unread_count,
                folder.total_count,
            ],
        ).map_err(|e| Error::Other(format!("Failed to save folder: {}", e)))?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Record how many messages a folder holds and how many are unread.
    ///
    /// Written by a sync from what the server says, not counted from the rows
    /// stored here: only part of a large folder is cached, so counting locally
    /// would tell somebody their inbox holds five hundred messages when it
    /// holds forty thousand.
    pub fn set_folder_counts(&self, folder_id: i64, unread: usize, total: usize) -> Result<()> {
        self.conn
            .execute(
                "UPDATE folders SET unread_count = ?1, total_count = ?2 WHERE id = ?3",
                params![unread as i64, total as i64, folder_id],
            )
            .map_err(|e| Error::Other(format!("Failed to record folder counts: {}", e)))?;
        Ok(())
    }

    /// Get folder by account and path
    pub fn get_folder(&self, account_id: &str, path: &str) -> Result<Option<CachedFolder>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, account_id, name, path, folder_type, unread_count, total_count
             FROM folders WHERE account_id = ?1 AND path = ?2",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let folder = stmt
            .query_row(params![account_id, path], |row| {
                Ok(CachedFolder {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    name: row.get(2)?,
                    path: row.get(3)?,
                    folder_type: row.get(4)?,
                    unread_count: row.get(5)?,
                    total_count: row.get(6)?,
                })
            })
            .optional()
            .map_err(|e| Error::Other(format!("Failed to get folder: {}", e)))?;

        Ok(folder)
    }

    /// Get all folders for an account
    pub fn get_folders_for_account(&self, account_id: &str) -> Result<Vec<CachedFolder>> {
        let mut stmt = self
            .conn
            .prepare(
                // Ordered by what the folder is for, then by name. Alphabetical
                // order alone puts Archive and Drafts above the inbox, so
                // somebody arrowing down the tree passes them both every time
                // to reach their mail. The cases match `FolderType::tree_order`.
                "SELECT id, account_id, name, path, folder_type, unread_count, total_count
             FROM folders WHERE account_id = ?1
             ORDER BY CASE lower(folder_type)
                          WHEN 'inbox' THEN 0
                          WHEN 'drafts' THEN 1
                          WHEN 'sent' THEN 2
                          WHEN 'archive' THEN 3
                          WHEN 'spam' THEN 4
                          WHEN 'junk' THEN 4
                          WHEN 'trash' THEN 5
                          ELSE 6
                      END,
                      lower(name)",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let folders = stmt
            .query_map(params![account_id], |row| {
                Ok(CachedFolder {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    name: row.get(2)?,
                    path: row.get(3)?,
                    folder_type: row.get(4)?,
                    unread_count: row.get(5)?,
                    total_count: row.get(6)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to query folders: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to collect folders: {}", e)))?;

        Ok(folders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_folder_operations() {
        let temp_dir = env::temp_dir().join("wixen_mail_test_folders");
        let cache = MessageCache::new(temp_dir, None).unwrap();

        let folder = CachedFolder {
            id: 0,
            account_id: "test@example.com".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "Inbox".to_string(),
            unread_count: 5,
            total_count: 10,
        };

        let id = cache.save_folder(&folder).unwrap();
        assert!(id > 0);

        let retrieved = cache.get_folder("test@example.com", "INBOX").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "INBOX");
    }

    #[test]
    fn test_the_tree_lists_the_inbox_first_and_ordinary_folders_last() {
        // Alphabetical order alone means arrowing past Archive and Drafts to
        // reach the inbox, every time.
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let cache = MessageCache::new(
            env::temp_dir().join(format!("wixen_folder_order_{nanos}")),
            None,
        )
        .unwrap();

        for (name, folder_type) in [
            ("Zebra project", "Custom"),
            ("Archive", "Archive"),
            ("INBOX", "Inbox"),
            ("Deleted Items", "Trash"),
            ("Drafts", "Drafts"),
            ("Apple project", "Custom"),
        ] {
            cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: "acc".to_string(),
                    name: name.to_string(),
                    path: name.to_string(),
                    folder_type: folder_type.to_string(),
                    unread_count: 0,
                    total_count: 0,
                })
                .unwrap();
        }

        let order: Vec<String> = cache
            .get_folders_for_account("acc")
            .unwrap()
            .into_iter()
            .map(|f| f.name)
            .collect();
        assert_eq!(
            order,
            vec![
                "INBOX",
                "Drafts",
                "Archive",
                "Deleted Items",
                "Apple project",
                "Zebra project",
            ]
        );
    }
}

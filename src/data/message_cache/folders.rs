//! Folder persistence operations

use super::{CachedFolder, MessageCache};
use crate::common::{Error, Result};
use rusqlite::{params, OptionalExtension};

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
                "SELECT id, account_id, name, path, folder_type, unread_count, total_count
             FROM folders WHERE account_id = ?1 ORDER BY name",
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
}

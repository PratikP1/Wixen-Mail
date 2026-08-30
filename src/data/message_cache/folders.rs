//! Folder persistence operations

use super::{CachedFolder, MessageCache};
use crate::common::{Error, Result};
use rusqlite::{OptionalExtension, params};

impl MessageCache {
    /// Record a folder the server listed, and return its id.
    ///
    /// Updates in place rather than replacing. Replacing looks equivalent and
    /// is not: the row is deleted and a new one inserted, so the folder gets a
    /// new id, everything the row remembered goes with it, and the delete
    /// cascades into every message cached in that folder. Since the folder list
    /// is saved on every sync, that meant UIDVALIDITY, the modification
    /// sequence and the sync choice were all thrown away every time, and each
    /// folder was downloaded again from scratch as though it had never been
    /// seen.
    ///
    /// The name and the role do follow the server, because a folder renamed
    /// elsewhere should not keep announcing its old name. The counts do not:
    /// they are written by the sync from what the server says, and blanking
    /// them here would empty the tree for as long as the sync takes.
    ///
    /// `parent_id` is left out of that list for the same reason and a sharper
    /// one. It is worked out in a second pass, after every folder in the
    /// account has an id, because a child can be listed before its parent. An
    /// upsert that ran first would therefore always blank it, and every sync
    /// would flatten the tree until the second pass caught up.
    pub fn save_folder(&self, folder: &CachedFolder) -> Result<i64> {
        self.conn
            .query_row(
                "INSERT INTO folders (account_id, name, path, folder_type, unread_count, total_count)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(account_id, path) DO UPDATE SET
                     name = excluded.name,
                     folder_type = excluded.folder_type
                 RETURNING id",
                params![
                    folder.account_id,
                    folder.name,
                    folder.path,
                    folder.folder_type,
                    folder.unread_count,
                    folder.total_count,
                ],
                |row| row.get(0),
            )
            .map_err(|e| Error::Other(format!("Failed to save folder: {}", e)))
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

    /// Which folders somebody has chosen to sync, and which they have not.
    ///
    /// Only the folders they have actually answered for. A folder missing from
    /// the map has never been asked about and gets the default, which is not
    /// the same as one they turned off.
    pub fn folder_choices(
        &self,
        account_id: &str,
    ) -> Result<std::collections::HashMap<String, bool>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT path, sync_enabled FROM folders
                 WHERE account_id = ?1 AND sync_enabled IS NOT NULL",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let rows = stmt
            .query_map(params![account_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? != 0))
            })
            .map_err(|e| Error::Other(format!("Failed to read folder choices: {}", e)))?
            .collect::<std::result::Result<std::collections::HashMap<_, _>, _>>()
            .map_err(|e| Error::Other(format!("Failed to read folder choices: {}", e)))?;

        Ok(rows)
    }

    /// Record that somebody chose whether a folder syncs.
    pub fn set_folder_choice(&self, account_id: &str, path: &str, sync: bool) -> Result<()> {
        self.conn
            .execute(
                "UPDATE folders SET sync_enabled = ?1 WHERE account_id = ?2 AND path = ?3",
                params![i64::from(sync), account_id, path],
            )
            .map_err(|e| Error::Other(format!("Failed to record the folder choice: {}", e)))?;
        Ok(())
    }

    /// Record what the server said about a folder, beyond its name and role.
    ///
    /// Kept so the window that asks somebody which folders to sync shows the
    /// same default the sync itself would use. Written by the sync, because
    /// these are the server's answers and nothing local can work them out.
    pub fn set_folder_server_facts(
        &self,
        folder_id: i64,
        holds_all_mail: bool,
        subscribed: bool,
    ) -> Result<()> {
        self.conn
            .execute(
                "UPDATE folders SET holds_all_mail = ?1, subscribed = ?2 WHERE id = ?3",
                params![i64::from(holds_all_mail), i64::from(subscribed), folder_id],
            )
            .map_err(|e| Error::Other(format!("Failed to record the folder facts: {}", e)))?;
        Ok(())
    }

    /// Remember that a row of the folder tree is collapsed, or that it is not.
    ///
    /// `identity` is `folder_tree::WhichRow::stored`, never a label. See the
    /// `tree_state` table for why this is not a setting.
    ///
    /// Setting a row open removes it rather than storing a nought, so the
    /// table holds only what somebody actually closed. A tree left entirely
    /// open costs no rows, and a folder that has gone leaves nothing behind.
    pub fn set_row_collapsed(&self, identity: &str, collapsed: bool) -> Result<()> {
        if collapsed {
            self.conn
                .execute(
                    "INSERT INTO tree_state (identity, collapsed) VALUES (?1, ?2)
                     ON CONFLICT(identity) DO UPDATE SET collapsed = excluded.collapsed",
                    params![identity, i64::from(collapsed)],
                )
                .map_err(|e| Error::Other(format!("Failed to record the collapsed row: {}", e)))?;
        } else {
            self.conn
                .execute(
                    "DELETE FROM tree_state WHERE identity = ?1",
                    params![identity],
                )
                .map_err(|e| Error::Other(format!("Failed to record the opened row: {}", e)))?;
        }
        Ok(())
    }

    /// Every row of the folder tree somebody has collapsed.
    ///
    /// One query returning a set, because the rebuild asks this once and then
    /// asks the set about each row. Asked per row it would be one query per
    /// folder per sync, on a timer.
    pub fn collapsed_rows(&self) -> Result<std::collections::HashSet<String>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT identity FROM tree_state WHERE collapsed != 0")
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| Error::Other(format!("Failed to read what is collapsed: {}", e)))?
            .collect::<std::result::Result<std::collections::HashSet<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to read what is collapsed: {}", e)))?;

        Ok(rows)
    }

    /// Pin a folder to the top of the tree, FOLDER-03.
    ///
    /// A new pin goes to the bottom of its own account's part of the group,
    /// D-31, so pinning three folders puts them in the order they were pinned
    /// and nothing already there moves under somebody.
    ///
    /// Pinning a folder that is already pinned changes nothing and is not a
    /// failure. The caller has a sentence to say about it, and a second row for
    /// one folder is the defect this whole phase exists to remove.
    ///
    /// A folder this computer does not have cannot be pinned: the row points at
    /// a real folder through a foreign key, so a pin can never outlive one.
    /// Nothing here reaches a server. See `application::favourites`.
    pub fn pin_row(&self, account_id: &str, path: &str) -> Result<()> {
        // The position is worked out inside the statement rather than read
        // first and written second. Read and then written, two pins arriving
        // together from the interface and a sync would compute the same number
        // and the second would take the first one's place. `COALESCE` over an
        // empty group gives nought, which is where the first pin belongs.
        //
        // Doing nothing on conflict is what makes pinning twice leave one pin
        // without moving it to the bottom again: somebody who pins what is
        // already pinned has said nothing new, and the caller answers them.
        self.conn
            .execute(
                "INSERT INTO favourites (account_id, path, position)
                 VALUES (
                     ?1, ?2,
                     (SELECT COALESCE(MAX(position) + 1, 0) FROM favourites WHERE account_id = ?1)
                 )
                 ON CONFLICT(account_id, path) DO NOTHING",
                params![account_id, path],
            )
            .map_err(|e| Error::Other(format!("Failed to pin the folder: {}", e)))?;
        Ok(())
    }

    /// Take a folder off the top of the tree.
    ///
    /// The folder itself is untouched: a pin was a copy, and the row in the
    /// account's own branch was never the pin. Unpinning something that is not
    /// pinned is not a failure, for the reason `forget_folder` gives.
    pub fn unpin_row(&self, account_id: &str, path: &str) -> Result<()> {
        self.conn
            .execute(
                "DELETE FROM favourites WHERE account_id = ?1 AND path = ?2",
                params![account_id, path],
            )
            .map_err(|e| Error::Other(format!("Failed to unpin the folder: {}", e)))?;
        Ok(())
    }

    /// Every pinned folder, by account and then by where it sits.
    ///
    /// One query returning the lot, because the tree asks this once per rebuild
    /// and then arranges what it got. Asked per folder it would be one query
    /// per row per sync, on a timer, which is what `collapsed_rows` says just
    /// above and for the same reason.
    pub fn pinned_rows(&self) -> Result<Vec<crate::application::favourites::Pin>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT account_id, path, position FROM favourites
                 ORDER BY account_id, position",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(crate::application::favourites::Pin {
                    account: row.get::<_, String>(0)?,
                    path: row.get::<_, String>(1)?,
                    position: row.get::<_, i64>(2)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to read what is pinned: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to read what is pinned: {}", e)))?;

        Ok(rows)
    }

    /// Put a pin at a given place in its own account's part of the group.
    ///
    /// The whole of that account's order is written by the caller, one call per
    /// pin, for the reason `account_order::Moved` gives about writing only the
    /// pair that swapped: a group half ordered by choice and half by arrival
    /// rearranges itself the next time anything is added.
    pub fn set_pin_position(&self, account_id: &str, path: &str, position: i64) -> Result<()> {
        self.conn
            .execute(
                "UPDATE favourites SET position = ?3 WHERE account_id = ?1 AND path = ?2",
                params![account_id, path, position],
            )
            .map_err(|e| Error::Other(format!("Failed to record where the pin sits: {}", e)))?;
        Ok(())
    }

    /// What the server said about each folder, by path.
    pub fn folder_server_facts(
        &self,
        account_id: &str,
    ) -> Result<std::collections::HashMap<String, (bool, bool)>> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT path, holds_all_mail, subscribed FROM folders WHERE account_id = ?1",
            )
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let rows = stmt
            .query_map(params![account_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    (row.get::<_, i64>(1)? != 0, row.get::<_, i64>(2)? != 0),
                ))
            })
            .map_err(|e| Error::Other(format!("Failed to read the folder facts: {}", e)))?
            .collect::<std::result::Result<std::collections::HashMap<_, _>, _>>()
            .map_err(|e| Error::Other(format!("Failed to read the folder facts: {}", e)))?;

        Ok(rows)
    }

    /// Record which folder this one sits under, or that it sits under none.
    ///
    /// Written once per sync, in a second pass after every folder in the
    /// account has an id. `None` is a real answer and not a missing one: a
    /// folder the server has moved to the top level has to stop claiming the
    /// parent it used to have, and the only way back otherwise is deleting the
    /// row.
    pub fn set_folder_parent(&self, folder_id: i64, parent_id: Option<i64>) -> Result<()> {
        self.conn
            .execute(
                "UPDATE folders SET parent_id = ?1 WHERE id = ?2",
                params![parent_id, folder_id],
            )
            .map_err(|e| Error::Other(format!("Failed to record the folder's parent: {}", e)))?;
        Ok(())
    }

    /// Move a folder's row to where the server has just put it.
    ///
    /// The row is moved rather than a second one written, because everything
    /// else points at the id: the messages in the folder, and the folders
    /// underneath it. Writing a new row and leaving the old one shows the
    /// folder twice in a tree that is read from here, with all of its mail
    /// under the name it no longer has.
    ///
    /// A path another folder in the account already holds comes back as a
    /// failure, from the unique index the folders table has always carried. The
    /// caller says so: the server has done the rename either way, and what is
    /// wrong is only the copy kept here, which the next check for mail settles.
    pub fn set_folder_path(&self, folder_id: i64, path: &str, name: &str) -> Result<()> {
        self.conn
            .execute(
                "UPDATE folders SET path = ?1, name = ?2 WHERE id = ?3",
                params![path, name, folder_id],
            )
            .map_err(|e| Error::Other(format!("Failed to record the folder's new name: {}", e)))?;
        Ok(())
    }

    /// Take a folder off this computer, along with the mail it held.
    ///
    /// Called after the server has taken it, so this is the copy catching up
    /// rather than a delete of its own. The mail goes with it, which is not
    /// incidental: a folder row removed while its messages stayed would leave
    /// the words of somebody's mail on the disk under a folder nothing lists,
    /// reachable by search and by nothing else. `foreign_keys` is on and the
    /// messages table cascades from the folder, so the one statement is enough,
    /// and the test beside this asserts the cascade really runs rather than
    /// trusting the schema to have been read correctly.
    ///
    /// A folder that is already gone is not a failure. A delete that stopped
    /// partway is run again against the folders left, and a row that went in
    /// between is one less thing to do.
    pub fn forget_folder(&self, folder_id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM folders WHERE id = ?1", params![folder_id])
            .map_err(|e| Error::Other(format!("Failed to remove the folder: {}", e)))?;
        Ok(())
    }

    /// Which folder each of an account's folders sits under, by path.
    ///
    /// One query for the whole account, because the tree is built from all of
    /// it at once and asking per folder would be one round trip per row on
    /// every rebuild. A path present with `None` is a folder at the top level;
    /// a path absent altogether is a folder this account does not have.
    pub fn folder_parents(
        &self,
        account_id: &str,
    ) -> Result<std::collections::HashMap<String, Option<i64>>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT path, parent_id FROM folders WHERE account_id = ?1")
            .map_err(|e| Error::Other(format!("Failed to prepare statement: {}", e)))?;

        let rows = stmt
            .query_map(params![account_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<i64>>(1)?))
            })
            .map_err(|e| Error::Other(format!("Failed to read the folder parents: {}", e)))?
            .collect::<std::result::Result<std::collections::HashMap<_, _>, _>>()
            .map_err(|e| Error::Other(format!("Failed to read the folder parents: {}", e)))?;

        Ok(rows)
    }

    /// The mailbox's modification sequence as at the last sync, if it has one.
    pub fn folder_modseq(&self, folder_id: i64) -> Result<Option<u64>> {
        self.conn
            .query_row(
                "SELECT highest_modseq FROM folders WHERE id = ?1",
                params![folder_id],
                |row| row.get::<_, Option<i64>>(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("Failed to read the folder state: {}", e)))
            .map(|found| found.flatten().map(|value| value as u64))
    }

    /// Record the mailbox's modification sequence after a sync.
    pub fn set_folder_modseq(&self, folder_id: i64, modseq: u64) -> Result<()> {
        self.conn
            .execute(
                "UPDATE folders SET highest_modseq = ?1 WHERE id = ?2",
                params![modseq as i64, folder_id],
            )
            .map_err(|e| Error::Other(format!("Failed to record the folder state: {}", e)))?;
        Ok(())
    }

    /// What kind of folder this is, by its identifier.
    ///
    /// Asked before a folder is listed, because the Outbox is not read from the
    /// messages table and nothing else can tell from the id alone.
    /// Which account a folder belongs to.
    ///
    /// The sync works in folder ids and the tag tables work in account ids, and
    /// this is the one row that knows both.
    pub fn folder_account(&self, folder_id: i64) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT account_id FROM folders WHERE id = ?1",
                params![folder_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("Failed to read the folder: {}", e)))
    }

    pub fn folder_kind(&self, folder_id: i64) -> Result<Option<crate::common::types::FolderType>> {
        self.conn
            .query_row(
                "SELECT folder_type FROM folders WHERE id = ?1",
                params![folder_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("Failed to read the folder: {}", e)))
            .map(|found| found.map(|kind| crate::common::types::FolderType::from_stored(&kind)))
    }

    /// Get folder by account and path
    /// Which account a stored folder belongs to.
    ///
    /// Asked where only the folder's own id is in hand, which is how the
    /// sync reaches the account's other folders to file mail into one.
    pub fn account_of_folder(&self, folder_id: i64) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT account_id FROM folders WHERE id = ?1",
                rusqlite::params![folder_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| Error::Other(format!("Failed to read the folder's account: {e}")))
    }

    pub fn get_folder(&self, account_id: &str, path: &str) -> Result<Option<CachedFolder>> {
        let mut stmt = self
            .conn
            .prepare_cached(
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
            .prepare_cached(
                // Read in a settled order and sorted below, rather than sorted
                // here. The database used to hold a second answer to where a
                // folder sits, and it disagreed: it had no place for mail
                // waiting to go, so the Outbox fell in among somebody's own
                // folders, and it read the kind of a folder without trimming it
                // where every other reader trims. Both are gone with the
                // expression. Alphabetical order alone is wrong either way,
                // because it puts Archive and Drafts above the inbox and makes
                // somebody arrow past both to reach their mail.
                "SELECT id, account_id, name, path, folder_type, unread_count, total_count
             FROM folders WHERE account_id = ?1
             ORDER BY id",
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

        let mut folders = folders;
        folders.sort_by_key(|folder| {
            crate::common::types::tree_position(
                crate::common::types::FolderType::from_stored(&folder.folder_type),
                &folder.name,
            )
        });
        Ok(folders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;

    #[test]
    fn test_folder_operations() {
        let temp_dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(temp_dir.path().to_path_buf(), None).unwrap();

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

    fn fresh(name: &str) -> TempHome<MessageCache> {
        TempHome::named(name, |dir| {
            MessageCache::new(dir.to_path_buf(), None).unwrap()
        })
    }

    fn inbox() -> CachedFolder {
        CachedFolder {
            id: 0,
            account_id: "acc".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "Inbox".to_string(),
            unread_count: 0,
            total_count: 0,
        }
    }

    #[test]
    fn test_a_row_that_was_collapsed_reads_back_as_collapsed() {
        let home = fresh("tree_state_one_row");
        let cache = &*home;
        cache.set_row_collapsed("account\u{1f}acc", true).unwrap();
        assert!(cache.collapsed_rows().unwrap().contains("account\u{1f}acc"));
    }

    #[test]
    fn test_opening_a_row_again_takes_it_out_of_what_is_collapsed() {
        let home = fresh("tree_state_reopened");
        let cache = &*home;
        cache.set_row_collapsed("account\u{1f}acc", true).unwrap();
        cache.set_row_collapsed("account\u{1f}other", true).unwrap();
        cache.set_row_collapsed("account\u{1f}acc", false).unwrap();
        let collapsed = cache.collapsed_rows().unwrap();
        assert!(
            !collapsed.contains("account\u{1f}acc"),
            "a row somebody opened again is not a row somebody closed"
        );
        assert!(
            collapsed.contains("account\u{1f}other"),
            "and opening one row does not open every other"
        );
    }

    #[test]
    fn test_collapsing_the_same_row_twice_is_not_an_error() {
        let home = fresh("tree_state_twice");
        let cache = &*home;
        cache.set_row_collapsed("account\u{1f}acc", true).unwrap();
        cache
            .set_row_collapsed("account\u{1f}acc", true)
            .expect("closing a row that is already closed is not a failure");
        assert_eq!(cache.collapsed_rows().unwrap().len(), 1);
    }

    #[test]
    fn test_only_the_rows_somebody_closed_come_back() {
        let home = fresh("tree_state_only_closed");
        let cache = &*home;
        cache.set_row_collapsed("account\u{1f}one", true).unwrap();
        cache.set_row_collapsed("account\u{1f}two", false).unwrap();
        let collapsed = cache.collapsed_rows().unwrap();
        assert_eq!(
            collapsed,
            std::collections::HashSet::from(["account\u{1f}one".to_string()])
        );
    }

    #[test]
    fn test_what_the_tree_remembers_survives_closing_and_reopening_the_cache() {
        let folder = tempfile::tempdir().expect("a temporary folder");
        {
            let cache =
                MessageCache::new(folder.path().to_path_buf(), None).expect("a cache to open");
            cache
                .set_row_collapsed("folder\u{1f}3\u{1f}accArchive", true)
                .unwrap();
        }
        // The same file, opened again, which is what a restart is.
        let again = MessageCache::new(folder.path().to_path_buf(), None).unwrap();
        assert!(
            again
                .collapsed_rows()
                .unwrap()
                .contains("folder\u{1f}3\u{1f}accArchive"),
            "a branch somebody collapsed is still collapsed after a restart"
        );
    }

    #[test]
    fn test_renaming_a_folder_does_not_lose_what_the_tree_remembered_about_its_account() {
        let home = fresh("tree_state_rename");
        let cache = &*home;
        // The account branch is keyed on the account id, and renaming an
        // account changes what it is called and not its id. This is the case
        // D-25 is about: a key built from the words in the row would have been
        // thrown away by the rename.
        cache.set_row_collapsed("account\u{1f}acc", true).unwrap();
        assert!(cache.collapsed_rows().unwrap().contains("account\u{1f}acc"));
    }

    /// A folder row for a pin to point at, since a pin cannot outlive one.
    fn a_folder(cache: &MessageCache, account: &str, path: &str) -> i64 {
        let leaf = path.rsplit('/').next().unwrap_or(path).to_string();
        cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: account.to_string(),
                name: leaf,
                path: path.to_string(),
                folder_type: "Custom".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder to pin")
    }

    /// What is pinned, as `(account, path)` in the order it came back.
    fn pinned(cache: &MessageCache) -> Vec<(String, String)> {
        cache
            .pinned_rows()
            .expect("what is pinned")
            .into_iter()
            .map(|pin| (pin.account, pin.path))
            .collect()
    }

    #[test]
    fn test_a_folder_that_was_pinned_reads_back_as_pinned() {
        let home = fresh("favourites_one");
        let cache = &*home;
        a_folder(cache, "acc", "Receipts");
        cache.pin_row("acc", "Receipts").unwrap();
        assert_eq!(pinned(cache), vec![("acc".to_string(), "Receipts".into())]);
    }

    #[test]
    fn test_unpinning_takes_that_folder_out_and_leaves_the_others() {
        let home = fresh("favourites_unpin");
        let cache = &*home;
        a_folder(cache, "acc", "Receipts");
        a_folder(cache, "acc", "Invoices");
        cache.pin_row("acc", "Receipts").unwrap();
        cache.pin_row("acc", "Invoices").unwrap();
        cache.unpin_row("acc", "Receipts").unwrap();
        // Both halves. Without the second, a body that emptied the table
        // wholesale would pass the first just as well.
        assert_eq!(pinned(cache), vec![("acc".to_string(), "Invoices".into())]);
    }

    #[test]
    fn test_unpinning_leaves_the_folder_itself_where_it_was() {
        // D-30. A pin was a copy, so taking it off must not take the folder
        // with it, and the cascade that removes a pin when a folder goes must
        // not be readable in the other direction.
        let home = fresh("favourites_unpin_keeps_folder");
        let cache = &*home;
        a_folder(cache, "acc", "Receipts");
        cache.pin_row("acc", "Receipts").unwrap();
        // That it was pinned at all, before asking whether it stopped being
        // pinned. An empty answer is what a read that does nothing gives, so
        // without this line the assertion below is satisfied by a pin that was
        // never stored and an unpin that never ran, and this test would say the
        // folder survived an operation nothing performed.
        assert_eq!(pinned(cache).len(), 1, "it was pinned to begin with");
        cache.unpin_row("acc", "Receipts").unwrap();
        // The folder surviving is what D-30 is about, but on its own it is
        // satisfied by an unpin that does nothing, so the pin having gone is
        // asserted too.
        assert!(
            pinned(cache).is_empty(),
            "the pin came off, so something happened"
        );
        assert!(
            cache.get_folder("acc", "Receipts").unwrap().is_some(),
            "and unpinning a folder never removes the folder"
        );
    }

    #[test]
    fn test_what_was_pinned_is_still_pinned_after_a_restart() {
        let folder = tempfile::tempdir().expect("a temporary folder");
        {
            let cache =
                MessageCache::new(folder.path().to_path_buf(), None).expect("a cache to open");
            a_folder(&cache, "acc", "Receipts");
            cache.pin_row("acc", "Receipts").unwrap();
        }
        let again =
            MessageCache::new(folder.path().to_path_buf(), None).expect("the cache to open again");
        assert_eq!(pinned(&again), vec![("acc".to_string(), "Receipts".into())]);
    }

    #[test]
    fn test_a_new_pin_goes_to_the_bottom_of_its_accounts_group() {
        // D-31. Three rather than two, so an implementation that put every new
        // pin at the top would give the exact reverse and be caught, and one
        // that swapped the last two would be caught as well.
        let home = fresh("favourites_bottom");
        let cache = &*home;
        for path in ["One", "Two", "Three"] {
            a_folder(cache, "acc", path);
            cache.pin_row("acc", path).unwrap();
        }
        let order: Vec<String> = cache
            .pinned_rows()
            .unwrap()
            .into_iter()
            .map(|pin| pin.path)
            .collect();
        assert_eq!(order, vec!["One", "Two", "Three"]);
    }

    #[test]
    fn test_each_account_counts_its_own_pins_from_the_top() {
        // The ordinary case as well as the odd one. A group numbered across
        // every account would give the second account's first pin a position of
        // one, and every rule below about moving a pin within its own account
        // would then be working on numbers that mean something else.
        let home = fresh("favourites_per_account");
        let cache = &*home;
        a_folder(cache, "one", "Receipts");
        a_folder(cache, "two", "Receipts");
        cache.pin_row("one", "Receipts").unwrap();
        cache.pin_row("two", "Receipts").unwrap();
        let firsts: Vec<i64> = cache
            .pinned_rows()
            .unwrap()
            .into_iter()
            .map(|pin| pin.position)
            .collect();
        assert_eq!(
            firsts,
            vec![0, 0],
            "each account's first pin is its own first"
        );
    }

    #[test]
    fn test_a_pin_put_in_a_new_place_stays_there_after_a_restart() {
        let folder = tempfile::tempdir().expect("a temporary folder");
        {
            let cache =
                MessageCache::new(folder.path().to_path_buf(), None).expect("a cache to open");
            for path in ["One", "Two"] {
                a_folder(&cache, "acc", path);
                cache.pin_row("acc", path).unwrap();
            }
            cache.set_pin_position("acc", "Two", 0).unwrap();
            cache.set_pin_position("acc", "One", 1).unwrap();
        }
        let again =
            MessageCache::new(folder.path().to_path_buf(), None).expect("the cache to open again");
        let order: Vec<String> = again
            .pinned_rows()
            .unwrap()
            .into_iter()
            .map(|pin| pin.path)
            .collect();
        assert_eq!(order, vec!["Two", "One"]);
    }

    #[test]
    fn test_pinning_the_same_folder_twice_leaves_one_pin() {
        let home = fresh("favourites_twice");
        let cache = &*home;
        a_folder(cache, "acc", "Receipts");
        cache.pin_row("acc", "Receipts").unwrap();
        cache
            .pin_row("acc", "Receipts")
            .expect("pinning what is already pinned is not a failure");
        assert_eq!(pinned(cache).len(), 1, "one folder, one row in the group");
    }

    #[test]
    fn test_a_renamed_folder_keeps_its_pin_under_its_new_name() {
        // D-32's first half, and the one that decided the shape of the table. A
        // rename rewrites a folder's path, `set_folder_path` says so in as many
        // words, so a pin holding its own copy of that path would be orphaned
        // by exactly the thing D-32 promises it survives. The pin follows
        // because the path has one writer and the row points at it.
        let home = fresh("favourites_rename");
        let cache = &*home;
        let id = a_folder(cache, "acc", "Reciepts");
        cache.pin_row("acc", "Reciepts").unwrap();
        cache.set_folder_path(id, "Receipts", "Receipts").unwrap();
        assert_eq!(
            pinned(cache),
            vec![("acc".to_string(), "Receipts".into())],
            "a folder somebody renamed is still pinned, under the name it has now"
        );
    }

    #[test]
    fn test_a_folder_that_is_really_gone_takes_its_pin_with_it() {
        // D-32's second half. Not lazily and not on the next rebuild: in the
        // same statement that removes the folder, so there is no window in
        // which a pin names a folder nothing has.
        let home = fresh("favourites_deleted");
        let cache = &*home;
        let id = a_folder(cache, "acc", "Receipts");
        a_folder(cache, "acc", "Invoices");
        cache.pin_row("acc", "Receipts").unwrap();
        cache.pin_row("acc", "Invoices").unwrap();
        cache.forget_folder(id).unwrap();
        assert_eq!(
            pinned(cache),
            vec![("acc".to_string(), "Invoices".into())],
            "the deleted folder's pin went with it and the other one stayed"
        );
    }

    #[test]
    fn test_a_folder_made_again_at_the_same_path_is_not_pinned_already() {
        // T-01-33. A pin that outlived its folder would pre-pin whatever turned
        // up at that path next, which for a mailbox name a server reuses is
        // somebody else's folder in somebody's favourites.
        let home = fresh("favourites_recreated");
        let cache = &*home;
        let id = a_folder(cache, "acc", "Receipts");
        cache.pin_row("acc", "Receipts").unwrap();
        cache.forget_folder(id).unwrap();
        a_folder(cache, "acc", "Receipts");
        assert!(
            pinned(cache).is_empty(),
            "a folder made again at an old path starts unpinned"
        );
    }

    #[test]
    fn test_a_folder_this_computer_does_not_have_cannot_be_pinned() {
        // The pin points at a real folder, so this is refused rather than
        // stored and later found to name nothing.
        let home = fresh("favourites_no_such_folder");
        let cache = &*home;
        assert!(
            cache.pin_row("acc", "Nowhere").is_err(),
            "there is no such folder to pin"
        );
        assert!(pinned(cache).is_empty());
    }

    #[test]
    fn test_a_database_written_before_anything_was_ever_pinned_still_opens() {
        let folder = tempfile::tempdir().expect("a temporary folder");
        {
            let cache =
                MessageCache::new(folder.path().to_path_buf(), None).expect("a cache to open");
            a_folder(&cache, "acc", "Receipts");
            // An older database is one with no such table at all.
            cache
                .conn
                .execute("DROP TABLE favourites", [])
                .expect("the table to come off, making this an older database");
        }
        let again = MessageCache::new(folder.path().to_path_buf(), None)
            .expect("an older database still opens");
        assert!(
            again.pinned_rows().unwrap().is_empty(),
            "and nothing is pinned yet, rather than the open failing"
        );
        // The table really came back, rather than the read merely surviving its
        // absence. Without this the test passes against a database that still
        // has no such table.
        again.pin_row("acc", "Receipts").unwrap();
        assert_eq!(pinned(&again).len(), 1);
    }

    #[test]
    fn test_a_database_written_before_the_tree_remembered_anything_still_opens() {
        let folder = tempfile::tempdir().expect("a temporary folder");
        {
            let cache =
                MessageCache::new(folder.path().to_path_buf(), None).expect("a cache to open");
            cache.save_folder(&inbox()).unwrap();
            // An older database is one with no such table at all.
            cache
                .conn
                .execute("DROP TABLE tree_state", [])
                .expect("the table to come off, making this an older database");
        }
        let again = MessageCache::new(folder.path().to_path_buf(), None)
            .expect("an older database still opens");
        assert!(
            again.collapsed_rows().unwrap().is_empty(),
            "and nothing is remembered yet, rather than the open failing"
        );
        assert!(
            again.get_folder("acc", "INBOX").unwrap().is_some(),
            "and the folders that were there are still there"
        );
        // The table really came back, rather than the read merely surviving
        // its absence. Without this the test passes against a database that
        // still has no such table.
        again.set_row_collapsed("account\u{1f}acc", true).unwrap();
        assert!(again.collapsed_rows().unwrap().contains("account\u{1f}acc"));
    }

    #[test]
    fn test_a_folder_says_which_account_it_belongs_to_and_what_kind_it_is() {
        // The sync works in folder ids and everything else works in account
        // ids, and this row is the only thing holding both. The wrong answer
        // sends a flag, a deletion or a send to another account's server, and
        // no answer at all stops the action with nothing to explain it.
        let cache = fresh("folder_identity");
        let mut mine = inbox();
        mine.account_id = "acc-mine".to_string();
        mine.path = "INBOX".to_string();
        let mine_id = cache.save_folder(&mine).unwrap();

        let mut theirs = inbox();
        theirs.account_id = "acc-theirs".to_string();
        theirs.path = "Sent".to_string();
        theirs.folder_type = "Sent".to_string();
        let theirs_id = cache.save_folder(&theirs).unwrap();

        assert_eq!(
            cache.folder_account(mine_id).unwrap(),
            Some("acc-mine".to_string())
        );
        assert_eq!(
            cache.folder_account(theirs_id).unwrap(),
            Some("acc-theirs".to_string())
        );
        assert_eq!(
            cache.folder_account(999_999).unwrap(),
            None,
            "a folder that is not there was given an account"
        );

        assert_eq!(
            cache.folder_kind(mine_id).unwrap(),
            Some(crate::common::types::FolderType::Inbox)
        );
        assert_eq!(
            cache.folder_kind(theirs_id).unwrap(),
            Some(crate::common::types::FolderType::Sent)
        );
        assert_eq!(cache.folder_kind(999_999).unwrap(), None);
    }

    #[test]
    fn test_a_renamed_folder_keeps_its_id_its_messages_and_its_place() {
        // The tree is read from what is stored, so a folder renamed on the
        // server and not here is a sentence saying it was renamed beside a row
        // that still says the old name. Updating the row rather than writing a
        // second one is what keeps the messages, which point at the id, and the
        // folders under it, which point at the id as well.
        let cache = fresh("renamed_folder");
        let mut folder = inbox();
        folder.path = "Archive".to_string();
        folder.name = "Archive".to_string();
        folder.folder_type = "Custom".to_string();
        let id = cache.save_folder(&folder).unwrap();

        let mut under = inbox();
        under.path = "Archive/2026".to_string();
        under.name = "2026".to_string();
        under.folder_type = "Custom".to_string();
        let under_id = cache.save_folder(&under).unwrap();
        cache.set_folder_parent(under_id, Some(id)).unwrap();

        cache.set_folder_path(id, "Old", "Old").unwrap();

        let moved = cache
            .get_folder("acc", "Old")
            .unwrap()
            .expect("the folder at its new path");
        assert_eq!(moved.id, id, "the row was replaced rather than moved");
        assert_eq!(moved.name, "Old");
        assert!(
            cache.get_folder("acc", "Archive").unwrap().is_none(),
            "the old path is still there, so the tree shows the folder twice"
        );
        assert_eq!(
            cache.folder_parents("acc").unwrap().get("Archive/2026"),
            Some(&Some(id)),
            "the folder inside it lost the folder above it"
        );
    }

    #[test]
    fn test_a_folder_taken_off_the_server_takes_its_mail_off_this_computer_too() {
        // A folder row removed while its messages stayed would leave the words
        // of somebody's mail on the disk under a folder nothing lists, which is
        // the opposite of what deleting a folder is for. `foreign_keys` is on
        // and the messages table cascades from the folder, so this asserts the
        // cascade really runs rather than assuming the schema is enough.
        let cache = fresh("forget_folder");
        let mut folder = inbox();
        folder.path = "Archive".to_string();
        let id = cache.save_folder(&folder).unwrap();
        cache
            .save_message(&crate::data::message_cache::CachedMessage {
                id: 0,
                uid: 7,
                folder_id: id,
                message_id: "<one@example.com>".to_string(),
                subject: "Quarterly report".to_string(),
                from_addr: "ada@example.com".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                date: "2026-08-24".to_string(),
                body_plain: Some("the words of somebody's mail".to_string()),
                body_html: None,
                read: false,
                starred: false,
                deleted: false,
            })
            .expect("a message");

        cache.forget_folder(id).unwrap();

        assert!(cache.get_folder("acc", "Archive").unwrap().is_none());
        assert!(
            cache.get_messages_for_folder(id, "acc").unwrap().is_empty(),
            "the mail in a deleted folder is still on this computer"
        );
    }

    #[test]
    fn test_forgetting_a_folder_that_is_already_gone_is_not_a_failure() {
        // A delete run a second time after one that stopped partway walks the
        // folders left, and a row somebody removed in between is not a fault:
        // the job of this call is that the row is not there afterwards.
        let cache = fresh("forget_folder_twice");
        let id = cache.save_folder(&inbox()).unwrap();

        cache.forget_folder(id).unwrap();

        assert!(cache.forget_folder(id).is_ok());
    }

    #[test]
    fn test_moving_a_folder_onto_a_path_another_folder_already_has_is_refused() {
        // Two rows with one path is the state the unique index exists to stop.
        // What matters is that it comes back as a failure the caller can say
        // something about, rather than as a silent no-op leaving the tree
        // disagreeing with the server.
        let cache = fresh("renamed_folder_collision");
        let mut first = inbox();
        first.path = "Archive".to_string();
        let id = cache.save_folder(&first).unwrap();
        let mut second = inbox();
        second.path = "Old".to_string();
        cache.save_folder(&second).unwrap();

        assert!(
            cache.set_folder_path(id, "Old", "Old").is_err(),
            "two folders were allowed to share one path"
        );
    }

    #[test]
    fn test_what_the_server_says_a_folder_holds_is_what_is_stored() {
        // Counted from the server, not from the rows here, because only part
        // of a large folder is cached. Not storing it leaves every folder
        // reading as empty in the tree, which is where somebody looks to find
        // out whether anything new has arrived.
        let cache = fresh("folder_counts");
        let first = cache.save_folder(&inbox()).unwrap();
        let mut second = inbox();
        second.path = "Archive".to_string();
        let second_id = cache.save_folder(&second).unwrap();

        cache.set_folder_counts(first, 3, 40_000).unwrap();

        let read_back = |id: i64| {
            cache
                .get_folders_for_account(&inbox().account_id)
                .unwrap()
                .into_iter()
                .find(|f| f.id == id)
                .expect("the folder")
        };
        let counted = read_back(first);
        assert_eq!(counted.unread_count, 3, "the unread count was not stored");
        assert_eq!(
            counted.total_count, 40_000,
            "the total was not stored, or was counted from what is cached"
        );
        assert_eq!(
            read_back(second_id).unread_count,
            0,
            "one folder's counts were written to another"
        );
    }

    #[test]
    fn test_saving_the_folder_list_again_keeps_what_the_folder_knows() {
        // Every sync saves the folder list. Replacing the row instead of
        // updating it threw away UIDVALIDITY, the modification sequence and
        // the sync choice, gave the folder a new id, and cascaded the delete
        // into every message cached in it. The folder then looked empty and
        // was downloaded again from scratch, every time.
        let cache = fresh("folder_state");
        let first_id = cache.save_folder(&inbox()).unwrap();
        cache.set_folder_uid_validity(first_id, 42).unwrap();
        cache.set_folder_modseq(first_id, 900).unwrap();
        cache.set_folder_choice("acc", "INBOX", false).unwrap();

        cache.save_folder(&inbox()).unwrap();

        let same = cache.get_folder("acc", "INBOX").unwrap().expect("the row");
        assert_eq!(same.id, first_id, "the folder kept its id");
        assert_eq!(cache.folder_uid_validity(first_id).unwrap(), Some(42));
        assert_eq!(cache.folder_modseq(first_id).unwrap(), Some(900));
        assert_eq!(
            cache.folder_choices("acc").unwrap().get("INBOX"),
            Some(&false)
        );
    }

    #[test]
    fn test_a_folder_that_was_renamed_on_the_server_takes_the_new_name() {
        // The row is updated rather than replaced, so the name still has to
        // follow the server. A folder renamed elsewhere would otherwise keep
        // announcing its old name for ever.
        let cache = fresh("folder_rename");
        cache.save_folder(&inbox()).unwrap();

        let mut renamed = inbox();
        renamed.name = "Inbox (work)".to_string();
        renamed.folder_type = "Custom".to_string();
        cache.save_folder(&renamed).unwrap();

        let row = cache.get_folder("acc", "INBOX").unwrap().expect("the row");
        assert_eq!(row.name, "Inbox (work)");
        assert_eq!(row.folder_type, "Custom");
    }

    #[test]
    fn test_what_the_server_said_about_a_folder_is_kept() {
        // Worked out from the folder's name instead, this only held for an
        // English Gmail account. All Mail is called something else in every
        // other language, and the row would then be ticked by default and
        // download the whole account a second time.
        let cache = fresh("folder_facts");
        let id = cache.save_folder(&inbox()).unwrap();

        cache.set_folder_server_facts(id, true, false).unwrap();

        assert_eq!(
            cache.folder_server_facts("acc").unwrap().get("INBOX"),
            Some(&(true, false))
        );
    }

    #[test]
    fn test_a_folder_nothing_was_recorded_about_reads_as_an_ordinary_one() {
        // What an existing database looks like the first time this runs. A
        // default of unsubscribed would read as "nobody wants any of these
        // folders", and the sync would stop downloading everything.
        let cache = fresh("folder_facts_default");
        cache.save_folder(&inbox()).unwrap();

        assert_eq!(
            cache.folder_server_facts("acc").unwrap().get("INBOX"),
            Some(&(false, true))
        );
    }

    #[test]
    fn test_a_folder_nobody_has_answered_for_is_absent_rather_than_false() {
        // "Never asked" and "asked and said no" are opposite instructions and
        // would look identical as a false.
        let cache = fresh("folder_unanswered");
        cache.save_folder(&inbox()).unwrap();

        assert!(cache.folder_choices("acc").unwrap().is_empty());
    }

    #[test]
    fn test_the_tree_lists_the_inbox_first_and_ordinary_folders_last() {
        // Alphabetical order alone means arrowing past Archive and Drafts to
        // reach the inbox, every time.
        let folder = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(folder.path().to_path_buf(), None).unwrap();

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

    /// Save these folders for one account and read the tree back by name.
    fn tree_of(cache: &MessageCache, folders: &[(&str, &str)]) -> Vec<String> {
        for (name, folder_type) in folders {
            cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: "acc".to_string(),
                    name: (*name).to_string(),
                    path: (*name).to_string(),
                    folder_type: (*folder_type).to_string(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("a folder");
        }
        cache
            .get_folders_for_account("acc")
            .expect("the tree")
            .into_iter()
            .map(|folder| folder.name)
            .collect()
    }

    fn a_cache() -> (tempfile::TempDir, MessageCache) {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
        (dir, cache)
    }

    #[test]
    fn test_mail_waiting_to_go_sits_between_the_drafts_and_what_has_gone() {
        // Mail that has not gone anywhere yet is the one folder somebody has to
        // act on, and a reader arrowing down the tree used to meet it last,
        // after everything they had already dealt with.
        //
        // Stored back to front, so the answer below is one nothing could give
        // by handing the rows back the way they went in.
        let (_dir, cache) = a_cache();

        let order = tree_of(
            &cache,
            &[
                ("Zebra project", "Custom"),
                ("Apple project", "Custom"),
                ("Deleted Items", "Trash"),
                ("Junk", "Spam"),
                ("Archive", "Archive"),
                ("Sent", "Sent"),
                ("Outbox", "Outbox"),
                ("Drafts", "Drafts"),
                ("INBOX", "Inbox"),
            ],
        );

        assert_eq!(
            order,
            vec![
                "INBOX",
                "Drafts",
                "Outbox",
                "Sent",
                "Archive",
                "Junk",
                "Deleted Items",
                "Apple project",
                "Zebra project",
            ]
        );
    }

    #[test]
    fn test_the_tree_reads_folders_in_the_order_the_folder_type_gives() {
        // The two answers are one answer. Names run against the order on
        // purpose, so a tree that fell back to sorting by name would come back
        // exactly reversed. This is also what would notice a ninth kind of
        // folder added later with no place of its own in a sort.
        //
        // Stored in an order that is neither the one expected back nor the one
        // the names alone would give, so neither handing the rows back
        // untouched nor sorting them by name can pass this.
        let (_dir, cache) = a_cache();

        let order = tree_of(
            &cache,
            &[
                ("d", "Archive"),
                ("a", "Custom"),
                ("h", "Inbox"),
                ("f", "Outbox"),
                ("b", "Trash"),
                ("g", "Drafts"),
                ("c", "Spam"),
                ("e", "Sent"),
            ],
        );

        let read_back = cache.get_folders_for_account("acc").expect("the tree");
        let places: Vec<u8> = read_back
            .iter()
            .map(|folder| {
                crate::common::types::FolderType::from_stored(&folder.folder_type).tree_order()
            })
            .collect();
        assert!(
            places.windows(2).all(|pair| pair[0] <= pair[1]),
            "the tree came back out of the order the folder types give: {places:?}"
        );

        let mut by_hand = read_back.clone();
        by_hand.sort_by_key(|folder| {
            crate::common::types::tree_position(
                crate::common::types::FolderType::from_stored(&folder.folder_type),
                &folder.name,
            )
        });
        let by_hand: Vec<String> = by_hand.into_iter().map(|folder| folder.name).collect();
        assert_eq!(order, by_hand);
    }

    #[test]
    fn test_a_folder_type_stored_with_spaces_round_it_sorts_as_what_it_is() {
        // Every other reader of this column trims before it decides what the
        // folder is, so a row written with spaces round it read as Sent
        // everywhere in the tree and sorted among somebody's own folders.
        //
        // The padded one is stored second and expected first, so this fails
        // two ways: nothing sorting at all leaves them as they went in, and a
        // reader that keeps the spaces takes the folder for one of somebody's
        // own and puts it behind a name beginning with A.
        let (_dir, cache) = a_cache();

        let order = tree_of(&cache, &[("Aaa", "Custom"), ("Sent", " Sent ")]);

        assert_eq!(order, vec!["Sent", "Aaa"]);
    }

    // ── A folder that knows its parent ──────────────────────────────────────

    #[test]
    fn test_a_folder_saved_with_no_parent_reads_back_without_one() {
        // A folder at the top level has no parent, and that is a real answer
        // rather than a missing one. It is why the column is nullable.
        let cache = fresh("folder_parent_none");

        cache.save_folder(&inbox()).unwrap();

        let parents = cache.folder_parents("acc").unwrap();
        assert_eq!(
            parents.get("INBOX"),
            Some(&None),
            "a folder with no parent has to be in the map, saying so"
        );
    }

    #[test]
    fn test_a_folder_can_be_told_which_folder_it_sits_under() {
        // The whole point of storing the nesting rather than working it out:
        // the tree reads a parent and never splits a path.
        let cache = fresh("folder_parent_set");
        let mut archive = inbox();
        archive.path = "Archive".to_string();
        archive.name = "Archive".to_string();
        archive.folder_type = "Custom".to_string();
        let archive_id = cache.save_folder(&archive).unwrap();

        let mut year = archive.clone();
        year.path = "Archive/2026".to_string();
        year.name = "2026".to_string();
        let year_id = cache.save_folder(&year).unwrap();

        cache.set_folder_parent(year_id, Some(archive_id)).unwrap();

        let parents = cache.folder_parents("acc").unwrap();
        assert_eq!(parents.get("Archive/2026"), Some(&Some(archive_id)));
        assert_eq!(
            parents.get("Archive"),
            Some(&None),
            "the parent itself is still at the top level"
        );
    }

    #[test]
    fn test_a_folder_moved_to_the_top_level_stops_claiming_its_old_parent() {
        // A server can move a folder out from under another one. Left as it
        // was, the tree would go on showing it in a branch it is no longer in,
        // and the only way back would be deleting the row.
        let cache = fresh("folder_parent_cleared");
        let mut archive = inbox();
        archive.path = "Archive".to_string();
        let archive_id = cache.save_folder(&archive).unwrap();

        let mut year = inbox();
        year.path = "Archive/2026".to_string();
        let year_id = cache.save_folder(&year).unwrap();

        cache.set_folder_parent(year_id, Some(archive_id)).unwrap();
        cache.set_folder_parent(year_id, None).unwrap();

        assert_eq!(
            cache.folder_parents("acc").unwrap().get("Archive/2026"),
            Some(&None),
            "clearing a parent has to clear it, not leave the old one"
        );
    }

    #[test]
    fn test_the_parents_of_a_whole_account_are_read_in_one_go() {
        // The tree builder asks once for the account rather than once per
        // folder. Every folder of the account is in the answer, and no other
        // account's is.
        let cache = fresh("folder_parents_map");
        let mut archive = inbox();
        archive.path = "Archive".to_string();
        let archive_id = cache.save_folder(&archive).unwrap();
        for path in ["Archive/2026", "Archive/2025", "Sent"] {
            let mut folder = inbox();
            folder.path = path.to_string();
            let id = cache.save_folder(&folder).unwrap();
            if path.starts_with("Archive/") {
                cache.set_folder_parent(id, Some(archive_id)).unwrap();
            }
        }

        let mut theirs = inbox();
        theirs.account_id = "other".to_string();
        theirs.path = "Archive/2026".to_string();
        cache.save_folder(&theirs).unwrap();

        let parents = cache.folder_parents("acc").unwrap();

        assert_eq!(parents.len(), 4, "every folder of the account, and no more");
        assert_eq!(parents.get("Archive/2026"), Some(&Some(archive_id)));
        assert_eq!(parents.get("Archive/2025"), Some(&Some(archive_id)));
        assert_eq!(parents.get("Sent"), Some(&None));
        assert_eq!(
            cache.folder_parents("other").unwrap().get("Archive/2026"),
            Some(&None),
            "the other account's folder answers for itself"
        );
    }

    #[test]
    fn test_saving_a_folder_again_does_not_lose_the_parent_it_was_given() {
        // The folder list is saved on every sync, and the parent is worked out
        // in a second pass after every folder has an id. An upsert that blanked
        // it would flatten the tree for as long as a sync takes, which is the
        // same reason the counts are left out of that list.
        let cache = fresh("folder_parent_survives_upsert");
        let mut archive = inbox();
        archive.path = "Archive".to_string();
        let archive_id = cache.save_folder(&archive).unwrap();

        let mut year = inbox();
        year.path = "Archive/2026".to_string();
        let year_id = cache.save_folder(&year).unwrap();
        cache.set_folder_parent(year_id, Some(archive_id)).unwrap();

        // The same path again, which is what the next sync does.
        year.name = "2026".to_string();
        let again = cache.save_folder(&year).unwrap();

        assert_eq!(again, year_id, "the same row, not a new one");
        assert_eq!(
            cache.folder_parents("acc").unwrap().get("Archive/2026"),
            Some(&Some(archive_id)),
            "saving the folder again threw its parent away"
        );
    }

    #[test]
    fn test_a_database_written_before_folders_had_parents_still_opens() {
        // Every installation from before this change has one of these. The
        // column arrives on open and every folder already in it starts at the
        // top level, which is what the tree showed before there was nesting.
        let folder = tempfile::tempdir().expect("a temporary folder");
        {
            let cache =
                MessageCache::new(folder.path().to_path_buf(), None).expect("a cache to open");
            cache.save_folder(&inbox()).expect("the folder to save");
            cache
                .conn
                .execute("ALTER TABLE folders DROP COLUMN parent_id", [])
                .expect("the column to come off, making this an older database");
        }

        let reopened = MessageCache::new(folder.path().to_path_buf(), None)
            .expect("the older database to open again");

        assert_eq!(
            reopened.folder_parents("acc").unwrap().get("INBOX"),
            Some(&None),
            "a folder written before there were parents has to read as having none"
        );
    }
}

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
            .prepare(
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

    /// What the server said about each folder, by path.
    pub fn folder_server_facts(
        &self,
        account_id: &str,
    ) -> Result<std::collections::HashMap<String, (bool, bool)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT path, holds_all_mail, subscribed FROM folders WHERE account_id = ?1")
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
}

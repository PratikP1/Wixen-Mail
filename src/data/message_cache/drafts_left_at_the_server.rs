//! Sent drafts whose copy at the server has not been seen to leave Drafts.
//!
//! Send takes a sent draft's filed copy out of the server's Drafts folder
//! (13.2-05). When the server cannot be reached or refuses, the copy stays,
//! and every other device shows it as a draft nobody has sent, which somebody
//! can send a second time. So the removal is kept here and tried again at the
//! account's next check for mail (ledger 845, Pratik's yes of 2026-10-15).
//!
//! # Kept before it is tried
//!
//! A removal written down only after it failed is lost when the program closes
//! while the removal is under way, which is the offline case at its likeliest.
//! So the window keeps it before the removal starts, and the removal forgets
//! it once the copy is gone or there was none. A removal kept while a check
//! also tries it costs nothing: both take the one turn every filing and
//! removal at a server takes, and the second finds nothing and forgets.
//!
//! The subject is kept because a sentence said at a later check has no other
//! way to name the message. This database is not encrypted and gets copied and
//! backed up, so removing an account forgets its removals with its drafts.

use super::MessageCache;
use crate::common::{Error, Result};
use rusqlite::params;

/// A removal of a sent draft's copy from the server's Drafts folder that has
/// not been seen to finish.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftLeftAtTheServer {
    /// The account whose server holds the copy.
    pub account_id: String,
    /// The draft's own id here, to tell whether it has become a draft here
    /// again before anything is removed.
    pub draft_id: String,
    /// The identifier the copy was filed under.
    pub message_id: String,
    /// The server's Drafts folder.
    pub folder_path: String,
    /// What the message is called, for the one sentence a failed second try
    /// says.
    pub subject: String,
    /// When it was kept.
    pub kept_at: String,
}

impl MessageCache {
    /// Keep a removal that has not been seen to finish. Keeping the same
    /// draft's removal again leaves one.
    pub fn keep_a_draft_left_at_the_server(&self, left: &DraftLeftAtTheServer) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO drafts_left_at_the_server
                 (account_id, draft_id, message_id, folder_path, subject, kept_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    left.account_id,
                    left.draft_id,
                    left.message_id,
                    left.folder_path,
                    left.subject,
                    left.kept_at,
                ],
            )
            .map_err(|e| Error::Other(format!("A sent draft's removal could not be kept: {e}")))?;
        Ok(())
    }

    /// Every removal kept for one account, oldest first.
    pub fn drafts_left_at_the_server_for(
        &self,
        account_id: &str,
    ) -> Result<Vec<DraftLeftAtTheServer>> {
        let could_not_read =
            |e: rusqlite::Error| Error::Other(format!("The kept removals could not be read: {e}"));
        let mut statement = self
            .conn
            .prepare(
                "SELECT account_id, draft_id, message_id, folder_path, subject, kept_at
                 FROM drafts_left_at_the_server WHERE account_id = ?1
                 ORDER BY kept_at, draft_id",
            )
            .map_err(could_not_read)?;
        let rows = statement
            .query_map(params![account_id], |row| {
                Ok(DraftLeftAtTheServer {
                    account_id: row.get(0)?,
                    draft_id: row.get(1)?,
                    message_id: row.get(2)?,
                    folder_path: row.get(3)?,
                    subject: row.get(4)?,
                    kept_at: row.get(5)?,
                })
            })
            .map_err(could_not_read)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(could_not_read)
    }

    /// Forget one account's removal of one draft, answering whether there was
    /// one to forget.
    pub fn forget_the_draft_left_at_the_server(
        &self,
        account_id: &str,
        draft_id: &str,
    ) -> Result<bool> {
        let forgotten = self
            .conn
            .execute(
                "DELETE FROM drafts_left_at_the_server WHERE account_id = ?1 AND draft_id = ?2",
                params![account_id, draft_id],
            )
            .map_err(|e| {
                Error::Other(format!(
                    "A sent draft's removal could not be forgotten: {e}"
                ))
            })?;
        Ok(forgotten > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;

    fn a_cache(what_for: &str) -> TempHome<MessageCache> {
        TempHome::named(what_for, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
        })
    }

    fn left(account_id: &str, draft_id: &str, subject: &str) -> DraftLeftAtTheServer {
        DraftLeftAtTheServer {
            account_id: account_id.to_string(),
            draft_id: draft_id.to_string(),
            message_id: format!("<draft-{draft_id}@wixen-mail.invalid>"),
            folder_path: "Drafts".to_string(),
            subject: subject.to_string(),
            kept_at: "2026-10-19T09:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_a_removal_kept_for_one_account_is_read_back_for_it_alone() {
        let cache = a_cache("left_for_one_account");
        let lunch = left("acc-1", "d-1", "Lunch on Friday");
        let theirs = left("acc-2", "d-2", "Someone else's");
        cache
            .keep_a_draft_left_at_the_server(&lunch)
            .expect("the removal to be kept");
        cache
            .keep_a_draft_left_at_the_server(&theirs)
            .expect("the other account's removal to be kept");

        let read = cache
            .drafts_left_at_the_server_for("acc-1")
            .expect("the removals to be read");

        assert_eq!(read, vec![lunch]);
    }

    #[test]
    fn test_keeping_the_same_draft_twice_keeps_one() {
        let cache = a_cache("left_twice");
        cache
            .keep_a_draft_left_at_the_server(&left("acc-1", "d-1", "First words"))
            .expect("the removal to be kept");
        let again = left("acc-1", "d-1", "Lunch on Friday");
        cache
            .keep_a_draft_left_at_the_server(&again)
            .expect("the removal to be kept again");

        let read = cache
            .drafts_left_at_the_server_for("acc-1")
            .expect("the removals to be read");

        assert_eq!(read, vec![again], "one removal, the one kept last");
    }

    #[test]
    fn test_a_forgotten_removal_is_gone_and_says_so() {
        let cache = a_cache("left_forgotten");
        cache
            .keep_a_draft_left_at_the_server(&left("acc-1", "d-1", "Lunch on Friday"))
            .expect("the removal to be kept");
        let kept_too = left("acc-1", "d-2", "Minutes");
        cache
            .keep_a_draft_left_at_the_server(&kept_too)
            .expect("the second removal to be kept");

        assert!(
            cache
                .forget_the_draft_left_at_the_server("acc-1", "d-1")
                .expect("the removal to be forgotten"),
            "a kept removal was forgotten and the answer said there was none"
        );
        assert_eq!(
            cache
                .drafts_left_at_the_server_for("acc-1")
                .expect("the removals to be read"),
            vec![kept_too],
            "the forgotten removal is still kept, or the other went with it"
        );
        assert!(
            !cache
                .forget_the_draft_left_at_the_server("acc-1", "d-1")
                .expect("the removal to be looked for again"),
            "a removal already forgotten was reported as forgotten again"
        );
    }

    #[test]
    fn test_removing_an_accounts_drafts_forgets_its_removals() {
        // The subject is kept here, in a file that is not encrypted and gets
        // copied, so it goes when the account's drafts go (premise 6).
        let cache = a_cache("left_cleared");
        cache
            .keep_a_draft_left_at_the_server(&left("acc-1", "d-1", "Lunch on Friday"))
            .expect("the removal to be kept");
        let theirs = left("acc-2", "d-2", "Someone else's");
        cache
            .keep_a_draft_left_at_the_server(&theirs)
            .expect("the other account's removal to be kept");

        cache
            .clear_drafts("acc-1")
            .expect("the drafts to be cleared");

        assert_eq!(
            cache
                .drafts_left_at_the_server_for("acc-2")
                .expect("the removals to be read"),
            vec![theirs],
            "another account's removal went with this account's drafts"
        );
        assert_eq!(
            cache
                .drafts_left_at_the_server_for("acc-1")
                .expect("the removals to be read"),
            Vec::new(),
            "the removed account's removal, subject and all, is still kept"
        );
    }
}

//! Emptying an account's Trash after the days somebody chose (13-44.6).
//!
//! Pratik asked on 2026-09-29 for a setting that empties the Trash of an
//! account whose provider does not, after 15 or 30 days. This module decides
//! what is due and what is said, and carries an emptying out at a check for
//! mail on the session the check already holds.
//!
//! # Through the menu's own Delete, one message at a time
//!
//! A message deleted inside the Trash is taken off the mail server, which is
//! what the menu's Delete does there (`WhatAWaitingMoveDoes::DeleteOutright`).
//! An emptying does exactly that for each message due, through
//! [`crate::application::moves_waiting::delete_here_then_at_the_server`], the
//! step a rule's Delete shares: made here first and kept in the store the
//! menu's Delete waits in, sent on the check's session, put back where the
//! server says no, and left waiting for the next check's replay where the
//! server cannot be reached (D4).
//!
//! # Guardrail 7
//!
//! **Nothing here may observe the network coming back and send.** An
//! emptying deletes at somebody's provider with nobody at the key, so it
//! happens only at a check for mail, which is this program already in front
//! of that server. `tests/the_trash_is_emptied_on_purpose.rs` holds the one
//! call to the check.
//!
//! # What has never been checked
//!
//! No real mail server has been emptied by this. The cases below drive a
//! double that keeps folders and writes down what it was asked; phase 14's
//! ledger line carries a real account.

use chrono::{DateTime, Duration, Utc};

use crate::application::destinations::{DeletedGoesTo, Deleting, where_a_deleted_message_goes};
use crate::application::moves_waiting::{
    APushUnderWay, DeletedHereThenAtTheServer, ReplaysAMove, delete_here_then_at_the_server,
};
use crate::application::what_rules_tell_the_server::ForTheChange;
use crate::common::Result;
use crate::common::types::FolderType;
use crate::data::message_cache::MessageCache;
use crate::data::message_cache::in_the_trash::InTheTrash;
use crate::data::message_cache::moves_waiting::{AWaitingMove, WhatAWaitingMoveDoes};
use crate::service::caldav::how_many;

/// When an account's Trash is emptied, as the account editor offers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WhenTheTrashIsEmptied {
    /// Never, the answer for every account until somebody chooses, because
    /// emptying the Trash cannot be undone (D1).
    #[default]
    Never,
    /// What went into the Trash here more than 15 days ago.
    After15Days,
    /// What went into the Trash here more than 30 days ago.
    After30Days,
}

impl WhenTheTrashIsEmptied {
    /// Every answer, in the order the choice offers them.
    pub const ALL: [Self; 3] = [Self::Never, Self::After15Days, Self::After30Days];

    /// Read back from the word stored, where a word this build does not know
    /// reads as Never, the safe end (D11).
    pub fn from_stored(stored: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|when| when.as_stored() == stored)
            .unwrap_or_default()
    }

    /// The word kept in the settings.
    pub fn as_stored(self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::After15Days => "after_15_days",
            Self::After30Days => "after_30_days",
        }
    }

    /// The words the choice shows.
    pub fn said(self) -> &'static str {
        match self {
            Self::Never => "Never",
            Self::After15Days => "After 15 days",
            Self::After30Days => "After 30 days",
        }
    }

    /// How many days a message stays in the Trash before it is due, or
    /// `None` for an answer that empties nothing on a schedule.
    pub fn days(self) -> Option<i64> {
        match self {
            Self::Never => None,
            Self::After15Days => Some(15),
            Self::After30Days => Some(30),
        }
    }
}

/// The account an emptying is for.
#[derive(Debug, Clone, Copy)]
pub struct TheAccount<'a> {
    pub id: &'a str,
    /// What the sentence calls it.
    pub name: &'a str,
}

/// What of the Trash is due: what went in here more than `days` days before
/// `now`, oldest first.
pub fn what_is_due(in_the_trash: &[InTheTrash], days: i64, now: DateTime<Utc>) -> Vec<InTheTrash> {
    in_the_trash
        .iter()
        .filter(|message| now - message.since > Duration::days(days))
        .cloned()
        .collect()
}

/// The one sentence an emptying says for an account, or `None` when it took
/// nothing (D8). The folder's own name, so the sentence names what the tree
/// shows.
pub fn what_the_emptying_said(
    folder_name: &str,
    account_name: &str,
    emptied: usize,
    days: i64,
) -> Option<String> {
    (emptied > 0).then(|| {
        format!(
            "Emptied {} from {folder_name} in {account_name} that had been there more than \
             {days} days.",
            how_many(emptied, "message")
        )
    })
}

/// Empty this account's Trash of what is due, on the check's own session,
/// and answer the sentence to say, if any.
///
/// The Trash is the one the menu's Delete moves to, found the same way
/// (D9), and each message due is deleted inside it as the menu's Delete
/// there deletes it: off the server, through the step a rule's Delete
/// shares. Only messages stored here are taken (D15), oldest first.
pub(crate) async fn empty_at_a_check<S: ReplaysAMove>(
    server: &S,
    cache: &MessageCache,
    account: TheAccount<'_>,
    when: WhenTheTrashIsEmptied,
    now: DateTime<Utc>,
) -> Result<Option<String>> {
    let Some(days) = when.days() else {
        return Ok(None);
    };
    let folders = cache.get_folders_for_account(account.id)?;
    let DeletedGoesTo::TheTrash(trash_path) = where_a_deleted_message_goes(
        folders.iter().map(|folder| {
            (
                folder.path.as_str(),
                FolderType::from_stored(&folder.folder_type),
            )
        }),
        "",
        Deleting::ToTrash,
    ) else {
        return Ok(None);
    };
    let Some(trash) = folders.iter().find(|folder| folder.path == trash_path) else {
        return Ok(None);
    };
    let due = what_is_due(&cache.what_has_been_in_the_trash(trash.id)?, days, now);
    let _under_way = APushUnderWay::begins(account.id);
    let mut emptied = 0;
    for message in &due {
        let asked = AWaitingMove {
            message_row_id: message.row,
            account_id: account.id.to_string(),
            from_folder_path: trash.path.clone(),
            uid: message.uid,
            what: WhatAWaitingMoveDoes::DeleteOutright,
            asked_at: now.to_rfc3339(),
        };
        // No subject: the line it makes is the menu's, for the eye at the
        // key, and an emptying says one sentence for the account (D8).
        let deleted = delete_here_then_at_the_server(server, cache, &asked, "").await;
        if let DeletedHereThenAtTheServer::Became(ForTheChange::Done) = deleted {
            emptied += 1;
        }
    }
    Ok(what_the_emptying_said(
        &trash.name,
        account.name,
        emptied,
        days,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::message_cache::moves_waiting::MarksFirst;
    use crate::data::message_cache::{CachedFolder, CachedMessage};
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    const THE_ACCOUNT: &str = "acct";

    /// A mail server that keeps each folder's messages by number and
    /// identifier, takes a message out when asked to delete it, and writes
    /// down every command it was asked, in order. On the shape of the double
    /// in `what_rules_tell_the_server`, of its own, since these cases are
    /// about what an emptying leaves at the server.
    struct AServerThatHoldsTheTrash {
        folders: RefCell<BTreeMap<String, Vec<(u32, String)>>>,
        log: RefCell<Vec<String>>,
    }

    impl AServerThatHoldsTheTrash {
        /// A server holding these numbers in its Trash.
        fn holding_in_the_trash(uids: &[u32]) -> Self {
            let held = uids.iter().map(|uid| (*uid, the_id_of(*uid))).collect();
            Self {
                folders: RefCell::new(BTreeMap::from([("Trash".to_string(), held)])),
                log: RefCell::new(Vec::new()),
            }
        }

        fn the_log(&self) -> Vec<String> {
            self.log.borrow().clone()
        }

        fn the_trash(&self) -> Vec<u32> {
            self.folders
                .borrow()
                .get("Trash")
                .map(|held| held.iter().map(|(uid, _)| *uid).collect())
                .unwrap_or_default()
        }
    }

    impl ReplaysAMove for AServerThatHoldsTheTrash {
        async fn move_it(&self, from: &str, uid: u32, into: &str) -> Result<()> {
            self.log
                .borrow_mut()
                .push(format!("MOVE {uid} in {from} into {into}"));
            Ok(())
        }

        async fn delete_it(&self, folder: &str, uid: u32, trash: Option<&str>) -> Result<()> {
            self.log.borrow_mut().push(match trash {
                Some(trash) => format!("DELETE {uid} in {folder} into {trash}"),
                None => format!("DELETE {uid} in {folder} off the server"),
            });
            if let Some(held) = self.folders.borrow_mut().get_mut(folder) {
                held.retain(|(held_uid, _)| *held_uid != uid);
            }
            Ok(())
        }

        async fn copy_it(&self, from: &str, uid: u32, into: &str) -> Result<()> {
            self.log
                .borrow_mut()
                .push(format!("COPY {uid} in {from} into {into}"));
            Ok(())
        }

        async fn where_it_is(&self, folder: &str, message_id: &str) -> Result<Vec<u32>> {
            Ok(self
                .folders
                .borrow()
                .get(folder)
                .map(|held| {
                    held.iter()
                        .filter(|(_, id)| id == message_id)
                        .map(|(uid, _)| *uid)
                        .collect()
                })
                .unwrap_or_default())
        }

        async fn mark_it(&self, _folder: &str, _uid: u32, _marks: MarksFirst) -> Result<()> {
            Ok(())
        }
    }

    fn the_id_of(uid: u32) -> String {
        format!("{uid}.trash@example.com")
    }

    /// A cache holding the account's Inbox and Trash, answering the Trash.
    fn an_account() -> (tempfile::TempDir, MessageCache, i64) {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
        a_folder(&cache, "Inbox", "INBOX", FolderType::Inbox);
        let trash = a_folder(&cache, "Trash", "Trash", FolderType::Trash);
        (dir, cache, trash)
    }

    fn a_folder(cache: &MessageCache, name: &str, path: &str, kind: FolderType) -> i64 {
        cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: THE_ACCOUNT.to_string(),
                name: name.to_string(),
                path: path.to_string(),
                folder_type: kind.as_str().to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("the folder")
    }

    /// A message stored in the Trash that went in there `days` days ago.
    fn in_the_trash_for(cache: &MessageCache, trash: i64, uid: u32, days: i64) -> i64 {
        let row = cache
            .save_message(&CachedMessage {
                id: 0,
                uid,
                folder_id: trash,
                message_id: the_id_of(uid),
                subject: "Lunch".to_string(),
                from_addr: "ada@example.com".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                date: "2026-09-19".to_string(),
                body_plain: None,
                body_html: None,
                read: false,
                starred: false,
                deleted: false,
                safety: crate::service::safety::Safety::Ordinary,
            })
            .expect("a message");
        cache
            .it_went_into_the_trash_at(row, Utc::now() - Duration::days(days))
            .expect("the stamp set");
        row
    }

    fn work() -> TheAccount<'static> {
        TheAccount {
            id: THE_ACCOUNT,
            name: "Work",
        }
    }

    fn a_check(
        server: &AServerThatHoldsTheTrash,
        cache: &MessageCache,
        when: WhenTheTrashIsEmptied,
    ) -> Option<String> {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
            .block_on(empty_at_a_check(server, cache, work(), when, Utc::now()))
            .expect("the emptying to finish")
    }

    fn is_deleted_here(cache: &MessageCache, row: i64) -> bool {
        cache
            .get_message(row)
            .expect("the row read")
            .expect("the row is there")
            .deleted
    }

    fn a_row_since(days: i64, now: DateTime<Utc>) -> InTheTrash {
        InTheTrash {
            row: days,
            uid: days as u32,
            since: now - Duration::days(days),
        }
    }

    #[test]
    fn test_an_account_set_to_thirty_days_has_what_went_into_the_trash_more_than_thirty_days_ago_taken_off_the_server()
     {
        let (_dir, cache, trash) = an_account();
        let server = AServerThatHoldsTheTrash::holding_in_the_trash(&[1, 2, 3]);
        let oldest = in_the_trash_for(&cache, trash, 1, 45);
        let older = in_the_trash_for(&cache, trash, 2, 31);
        let recent = in_the_trash_for(&cache, trash, 3, 10);

        let said = a_check(&server, &cache, WhenTheTrashIsEmptied::After30Days);

        assert_eq!(
            server.the_log(),
            [
                "DELETE 1 in Trash off the server",
                "DELETE 2 in Trash off the server"
            ]
        );
        assert_eq!(server.the_trash(), [3]);
        for row in [oldest, older] {
            assert!(
                is_deleted_here(&cache, row),
                "row {row} is not marked deleted here"
            );
            assert_eq!(
                cache.the_move_waiting_for(row).expect("the store read"),
                None,
                "row {row} is still waiting"
            );
        }
        assert!(!is_deleted_here(&cache, recent));
        assert_eq!(
            said.as_deref(),
            Some("Emptied 2 messages from Trash in Work that had been there more than 30 days.")
        );
    }

    #[test]
    fn test_a_message_in_the_trash_exactly_the_days_chosen_is_not_yet_due() {
        let now = Utc::now();
        let exactly = InTheTrash {
            row: 1,
            uid: 1,
            since: now - Duration::days(30),
        };
        let a_moment_more = InTheTrash {
            row: 2,
            uid: 2,
            since: now - Duration::days(30) - Duration::seconds(1),
        };
        assert_eq!(
            what_is_due(&[exactly, a_moment_more.clone()], 30, now),
            [a_moment_more]
        );
        assert_eq!(
            what_is_due(&[a_row_since(40, now), a_row_since(29, now)], 30, now),
            [a_row_since(40, now)]
        );
    }

    #[test]
    fn test_never_empties_nothing_and_dials_nothing() {
        let (_dir, cache, trash) = an_account();
        let server = AServerThatHoldsTheTrash::holding_in_the_trash(&[1]);
        let row = in_the_trash_for(&cache, trash, 1, 400);

        assert_eq!(a_check(&server, &cache, WhenTheTrashIsEmptied::Never), None);

        assert!(server.the_log().is_empty(), "{:?}", server.the_log());
        assert!(!is_deleted_here(&cache, row));
    }

    #[test]
    fn test_the_words_stored_are_read_back_and_a_word_this_build_does_not_know_is_never() {
        for when in WhenTheTrashIsEmptied::ALL {
            assert_eq!(
                WhenTheTrashIsEmptied::from_stored(when.as_stored()),
                when,
                "{when:?} stored as {:?}",
                when.as_stored()
            );
            assert!(!when.said().is_empty(), "{when:?} shows no words");
        }
        assert_eq!(
            WhenTheTrashIsEmptied::from_stored("weekly"),
            WhenTheTrashIsEmptied::Never
        );
        assert_eq!(
            WhenTheTrashIsEmptied::ALL.map(WhenTheTrashIsEmptied::days),
            [None, Some(15), Some(30)]
        );
        assert_eq!(
            WhenTheTrashIsEmptied::ALL.map(WhenTheTrashIsEmptied::said),
            ["Never", "After 15 days", "After 30 days"]
        );
    }

    #[test]
    fn test_each_account_keeps_its_own_answer_and_never_keeps_no_row() {
        let mut settings = crate::data::config::AppConfig::default();
        settings.set_trash_emptying_for("work", WhenTheTrashIsEmptied::After30Days);
        settings.set_trash_emptying_for("home", WhenTheTrashIsEmptied::After15Days);
        assert_eq!(
            settings.trash_emptying_for("work"),
            WhenTheTrashIsEmptied::After30Days
        );
        assert_eq!(
            settings.trash_emptying_for("home"),
            WhenTheTrashIsEmptied::After15Days
        );
        assert_eq!(
            settings.trash_emptying_for("nobody chose"),
            WhenTheTrashIsEmptied::Never
        );

        settings.set_trash_emptying_for("work", WhenTheTrashIsEmptied::Never);
        assert_eq!(
            settings.trash_emptying_for("work"),
            WhenTheTrashIsEmptied::Never
        );
        assert!(
            !settings.trash_emptying.contains_key("work"),
            "Never kept a row: {:?}",
            settings.trash_emptying
        );
    }

    #[test]
    fn test_one_message_is_said_in_the_singular() {
        assert_eq!(
            what_the_emptying_said("Deleted Items", "Home", 1, 15).as_deref(),
            Some(
                "Emptied 1 message from Deleted Items in Home that had been there more than 15 days."
            )
        );
        assert_eq!(what_the_emptying_said("Trash", "Home", 0, 15), None);
    }
}

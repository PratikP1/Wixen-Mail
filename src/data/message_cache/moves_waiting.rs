//! Moves and deletes made on this computer and not yet at the server, kept
//! across restarts.
//!
//! A message moved to Archive on a train has to still be in Archive when the
//! train arrives, and has to reach the server then. A move remembered only
//! in memory is one the program loses on its way out, which is the same as
//! never having kept it; worse, the next check would list the source folder,
//! find the message still there, and bring it back.
//!
//! The decisions and the words live in [`crate::application::moves_waiting`],
//! which has no database in it, and its module header carries the constraint
//! that matters: nothing here may observe the network coming back and send.
//!
//! # One row per message, and it keeps where the server still has it
//!
//! A message can be moved again before the server has heard of the first
//! move: Inbox to Archive, then Archive to Work, both with no network. The
//! second move is asked for from Archive, under the number this computer gave
//! the row when it moved it there, and the server knows neither. So the row
//! waiting for a message is one row, keyed on the message, and a second move
//! replaces what it is asking for and keeps where it is asking from: the
//! folder and the number the server still holds the message under. A replay
//! is then one command the server can carry out, from the folder it has the
//! message in to the folder the person last chose.

use super::MessageCache;
use crate::common::Result;

/// What a waiting move asks the server to do.
///
/// Written out as words rather than stored as a number, because these go in a
/// database column somebody may read and a discriminant is neither stable
/// across a reordering nor legible in a browser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatAWaitingMoveDoes {
    /// Move it into this folder of the same account.
    Move { into_folder_path: String },
    /// Move it into the account's trash, which is what the ordinary delete
    /// means.
    DeleteToTrash { trash_path: String },
    /// Take it off the server, which is what Delete Permanently means, and
    /// what the ordinary delete means inside the trash.
    DeleteOutright,
}

impl WhatAWaitingMoveDoes {
    /// The word the row carries, and the folder beside it when there is one.
    fn as_stored(&self) -> (&'static str, Option<&str>) {
        match self {
            Self::Move { into_folder_path } => ("move", Some(into_folder_path)),
            Self::DeleteToTrash { trash_path } => ("delete_to_trash", Some(trash_path)),
            Self::DeleteOutright => ("delete_outright", None),
        }
    }

    /// Where the message is meant to end up, for the kinds that have one.
    pub fn destination(&self) -> Option<&str> {
        self.as_stored().1
    }
}

/// One move or delete waiting to go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AWaitingMove {
    /// The message's row on this computer, which is what the window knows it
    /// by and what the local state is put back on.
    pub message_row_id: i64,
    pub account_id: String,
    /// The folder the server still has the message in, which is what a
    /// replay names.
    pub from_folder_path: String,
    /// The number the server still has it under.
    pub uid: u32,
    pub what: WhatAWaitingMoveDoes,
    pub asked_at: String,
}

impl MessageCache {
    /// Keep a move or delete that has been made here and not yet at the
    /// server.
    ///
    /// A message already waiting keeps the folder and the number the server
    /// still has it under, and takes the new ask: the module header says why.
    pub fn keep_a_move_waiting(&self, waiting: &AWaitingMove) -> Result<()> {
        let _ = waiting;
        Ok(())
    }

    /// Every move waiting for one account, in the order they were asked.
    ///
    /// A row whose kind this version does not recognise is left out rather
    /// than refused, so a database written by a later version still hands
    /// back the moves this one understands instead of failing whole.
    pub fn moves_waiting_for(&self, account_id: &str) -> Result<Vec<AWaitingMove>> {
        let _ = account_id;
        Ok(Vec::new())
    }

    /// Let one waiting move go, because it went or because it was put back.
    pub fn stop_waiting_for_a_move(&self, message_row_id: i64) -> Result<()> {
        let _ = message_row_id;
        Ok(())
    }

    /// The row is where the server holds it: in this folder, under this
    /// number, not deleted and not a row this program filed.
    ///
    /// The one write behind two facts. A move the server carried out leaves
    /// the row in the folder it was moved to under the number the server gave
    /// it there, so the next read of that folder finds it held and neither
    /// fetches it again nor forgets it. A move the server refused puts the row
    /// back in the folder and under the number it never left.
    ///
    /// When the sync has already brought that message down, so another row
    /// sits at that folder and number, this row is the copy and goes: the
    /// table keys a message on folder and number, and two rows for one message
    /// is the duplicate the marker on a moved row exists to prevent.
    pub fn the_server_holds_it_at(
        &self,
        message_row_id: i64,
        folder_id: i64,
        uid: u32,
    ) -> Result<()> {
        let _ = (message_row_id, folder_id, uid);
        Ok(())
    }

    /// Take the row off this computer so the next read of its folder brings
    /// the message down as the server holds it.
    ///
    /// For a move the server carried out whose landing place it could not
    /// say: the row here carries a number the server never gave, and left in
    /// place it would sit beside the real message once the folder is read.
    pub fn let_the_next_read_bring_it(&self, message_row_id: i64) -> Result<()> {
        let _ = message_row_id;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{CachedFolder, CachedMessage};
    use rusqlite::params;
    use std::path::Path;

    /// A cache in this folder, holding one account's Inbox, Archive and Trash.
    fn a_cache_at(dir: &Path) -> MessageCache {
        let cache = MessageCache::new(dir.to_path_buf(), None).expect("a cache");
        for (name, kind) in [
            ("INBOX", "Inbox"),
            ("Archive", "Archive"),
            ("Trash", "Trash"),
        ] {
            cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: "an account".to_string(),
                    name: name.to_string(),
                    path: name.to_string(),
                    folder_type: kind.to_string(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("a folder");
        }
        cache
    }

    fn a_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_moves_waiting_", a_cache_at)
    }

    fn the_folder(cache: &MessageCache, path: &str) -> i64 {
        cache
            .get_folder("an account", path)
            .expect("the folder")
            .expect("the folder is there")
            .id
    }

    /// One message in the Inbox, answering with the row it was given.
    fn a_message_in_the_inbox(cache: &MessageCache, uid: u32) -> i64 {
        cache
            .save_message(&CachedMessage {
                id: 0,
                uid,
                folder_id: the_folder(cache, "INBOX"),
                message_id: format!("lunch.{uid}@example.com"),
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
            .expect("a message")
    }

    fn a_move_of(row: i64, uid: u32, from: &str, into: &str) -> AWaitingMove {
        AWaitingMove {
            message_row_id: row,
            account_id: "an account".to_string(),
            from_folder_path: from.to_string(),
            uid,
            what: WhatAWaitingMoveDoes::Move {
                into_folder_path: into.to_string(),
            },
            asked_at: "2026-09-19T04:00:00Z".to_string(),
        }
    }

    fn where_the_row_is(cache: &MessageCache, row: i64) -> (i64, u32, bool, bool) {
        let message = cache
            .get_message(row)
            .expect("the message")
            .expect("the row is still there");
        (
            message.folder_id,
            message.uid,
            message.deleted,
            cache.was_filed_here(row).expect("the marker"),
        )
    }

    #[test]
    fn test_a_waiting_move_survives_being_written_down_and_read_back() {
        // The one test that proves the table does the thing it exists for: a
        // second connection, opened the way a restart opens one, reads what
        // the first wrote.
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        home.keep_a_move_waiting(&a_move_of(row, 42, "INBOX", "Archive"))
            .expect("a move kept");
        let reopened = MessageCache::new(home.path().to_path_buf(), None).expect("the cache again");
        assert_eq!(
            reopened
                .moves_waiting_for("an account")
                .expect("the waiting moves"),
            vec![a_move_of(row, 42, "INBOX", "Archive")],
            "a move kept only in memory is one the program loses on its way out"
        );
    }

    #[test]
    fn test_a_second_move_of_a_waiting_message_keeps_where_the_server_still_has_it() {
        // Inbox to Archive, then Archive to Work, both with no network. The
        // server has the message in the Inbox under 42 and has never heard of
        // Archive's number for it, so what waits is one command it can carry
        // out: from the Inbox under 42, to Work.
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        home.keep_a_move_waiting(&a_move_of(row, 42, "INBOX", "Archive"))
            .expect("a move kept");
        home.keep_a_move_waiting(&a_move_of(row, 4_294_967_000, "Archive", "Work"))
            .expect("a move kept");

        let waiting = home
            .moves_waiting_for("an account")
            .expect("the waiting moves");
        assert_eq!(waiting.len(), 1, "two rows for one message: {waiting:?}");
        assert_eq!(waiting[0].from_folder_path, "INBOX", "{waiting:?}");
        assert_eq!(waiting[0].uid, 42, "{waiting:?}");
        assert_eq!(
            waiting[0].what,
            WhatAWaitingMoveDoes::Move {
                into_folder_path: "Work".to_string()
            },
            "the earlier ask survived the later one: {waiting:?}"
        );
    }

    #[test]
    fn test_moves_wait_in_the_order_they_were_asked() {
        let home = a_cache();
        let first = a_message_in_the_inbox(&home, 7);
        let second = a_message_in_the_inbox(&home, 3);
        home.keep_a_move_waiting(&AWaitingMove {
            asked_at: "2026-09-19T04:00:01Z".to_string(),
            ..a_move_of(first, 7, "INBOX", "Archive")
        })
        .expect("a move kept");
        home.keep_a_move_waiting(&AWaitingMove {
            asked_at: "2026-09-19T04:00:02Z".to_string(),
            ..a_move_of(second, 3, "INBOX", "Archive")
        })
        .expect("a move kept");

        let rows: Vec<i64> = home
            .moves_waiting_for("an account")
            .expect("the waiting moves")
            .iter()
            .map(|waiting| waiting.message_row_id)
            .collect();
        assert_eq!(
            rows,
            vec![first, second],
            "asked in one order, replayed in another"
        );
    }

    #[test]
    fn test_a_move_that_stopped_waiting_is_not_offered_again() {
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        home.keep_a_move_waiting(&a_move_of(row, 42, "INBOX", "Archive"))
            .expect("a move kept");
        home.stop_waiting_for_a_move(row).expect("letting go");
        assert!(
            home.moves_waiting_for("an account")
                .expect("the waiting moves")
                .is_empty()
        );
    }

    #[test]
    fn test_a_kind_this_version_does_not_know_is_left_out_rather_than_refused() {
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        home.keep_a_move_waiting(&a_move_of(row, 42, "INBOX", "Archive"))
            .expect("a move kept");
        home.conn
            .execute(
                "UPDATE moves_waiting SET kind = 'something later' WHERE message_row_id = ?1",
                params![row],
            )
            .expect("a kind from the future");
        assert!(
            home.moves_waiting_for("an account")
                .expect("the waiting moves")
                .is_empty(),
            "a kind this version cannot replay was handed back as though it could"
        );
    }

    #[test]
    fn test_the_row_is_put_where_the_server_holds_it_with_the_marker_off() {
        // Moved here into Archive, the row carries a number this computer
        // reserved and the marker that keeps the sync's hands off it. Once
        // the server has it in Archive under 77, the row is that message:
        // under 77, unmarked, so the next read of Archive neither fetches it
        // again nor forgets it.
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let archive = the_folder(&home, "Archive");
        home.move_message(row, archive).expect("moved here");
        let (_, reserved, _, marked) = where_the_row_is(&home, row);
        assert!(
            marked && reserved != 42,
            "the move here did not reserve a number"
        );

        home.the_server_holds_it_at(row, archive, 77)
            .expect("the row settled");

        assert_eq!(
            where_the_row_is(&home, row),
            (archive, 77, false, false),
            "(folder, uid, deleted, filed here)"
        );
        assert_eq!(
            home.stored_uids(archive)
                .expect("the uids the sync compares"),
            vec![77],
            "the sync would fetch the message again beside this row"
        );
    }

    #[test]
    fn test_the_same_write_puts_a_refused_move_back_where_it_never_left() {
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let inbox = the_folder(&home, "INBOX");
        home.move_message(row, the_folder(&home, "Archive"))
            .expect("moved here");
        home.delete_message(row).expect("and deleted here");

        home.the_server_holds_it_at(row, inbox, 42)
            .expect("the row put back");

        assert_eq!(
            where_the_row_is(&home, row),
            (inbox, 42, false, false),
            "(folder, uid, deleted, filed here)"
        );
    }

    #[test]
    fn test_a_row_the_read_already_brought_down_wins_over_the_copy_moved_here() {
        // The read of Archive got there first and fetched the message under
        // 77. The row moved here is now the second copy of one message, and
        // the table keys a message on folder and number, so it goes.
        let home = a_cache();
        let moved_here = a_message_in_the_inbox(&home, 42);
        let archive = the_folder(&home, "Archive");
        home.move_message(moved_here, archive).expect("moved here");
        let brought_down = home
            .save_message(&CachedMessage {
                id: 0,
                uid: 77,
                folder_id: archive,
                message_id: "lunch.42@example.com".to_string(),
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
            .expect("the fetched copy");

        home.the_server_holds_it_at(moved_here, archive, 77)
            .expect("the row settled");

        assert!(
            home.get_message(moved_here).expect("the read").is_none(),
            "two rows for one message in one folder"
        );
        assert_eq!(where_the_row_is(&home, brought_down).1, 77);
    }

    #[test]
    fn test_a_row_left_for_the_next_read_is_gone_from_this_computer() {
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        home.move_message(row, the_folder(&home, "Archive"))
            .expect("moved here");
        home.let_the_next_read_bring_it(row)
            .expect("the row dropped");
        assert!(home.get_message(row).expect("the read").is_none());
    }
}

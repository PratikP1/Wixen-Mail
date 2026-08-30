//! Merging each account's local folders into the ones they now share.
//!
//! D-18 made Sent, Outbox, Drafts, Junk and Trash one each rather than one per
//! account. A database written before that has one set per account, holding
//! mail this program is the only copy of: there is no server to fetch a local
//! Sent or a local Drafts back from. So this is the same risk
//! `application::importing_messages` opens by warning about, and it takes the
//! same care.
//!
//! # The five properties copied from `migrate_inline_bodies`
//!
//! That function is the analog and this follows it in all five:
//!
//! 1. The whole candidate set is read into a `Vec` before anything is written.
//!    Holding a statement open over a table being written to is how a migration
//!    stops halfway with no error.
//! 2. Each message lands in the shared folder before the old row lets go of it.
//!    That is D-19's "nothing is removed until every message has landed", one
//!    message at a time.
//! 3. The count is the contract. The log line is not part of it, so the report
//!    is returned and tested rather than asserted on through tracing.
//! 4. Nothing is dropped. The old folder rows stay, emptied, and no column is
//!    dropped or renamed.
//! 5. It is idempotent by its `WHERE` clause: a second open finds no messages
//!    left in a per-account copy of a shared folder and moves nothing.
//!
//! # Why the numbers are rewritten, and how
//!
//! `messages` carries `UNIQUE(folder_id, uid)`. IMAP UIDs are per mailbox and
//! POP ones per download, so two accounts' local Trash both holding uid 42 is
//! expected rather than hypothetical, and carrying the old number across would
//! either fail the move or lose the second message. D-40 settles it: a fresh
//! number, unique within the shared folder, with the original written down
//! beside it.
//!
//! The number is not invented here. [`MessageCache::move_message_recording_its_origin`]
//! hands it out through the same path an ordinary move uses, which is the one
//! place that decides which end of a folder's numbering a filed row counts
//! from. It also carries the marker saying this program wrote the row, and that
//! marker is what stops the next sync deleting it. A second mover written here
//! that forgot either would lose mail in the migration meant to protect it.

use super::MessageCache;
use crate::application::local_folders::{SHARED_BY_EVERY_ACCOUNT, THIS_COMPUTER};
use crate::common::{Error, Result};

/// What the merge found, and what it did.
///
/// `found` is measured before anything moves and `moved` after, so the two
/// disagreeing is the signal that something stopped partway. Both are said
/// aloud and written to the log, because the person did not ask for this and
/// their mail moved.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MergeReport {
    /// How many messages were about to be moved, counted before the first one
    /// was.
    pub found: usize,
    /// How many actually landed in a shared folder.
    pub moved: usize,
    /// The accounts they came from, in the order they were met, without
    /// repeats.
    pub from_accounts: Vec<String>,
    /// How many came out of the move holding a different number from the one
    /// they went in with.
    pub renumbered: usize,
}

impl MergeReport {
    /// Whether anything moved, which is what decides whether to say so.
    pub fn anything_happened(&self) -> bool {
        self.moved > 0 || self.found > 0
    }

    /// The sentence somebody hears and the log records.
    ///
    /// Counts and account names only. Never a subject, an address or a body:
    /// this is spoken aloud, and mail gets read in rooms.
    pub fn said(&self) -> String {
        if self.moved == 0 {
            return "Your folders on this computer are already shared between your accounts."
                .to_string();
        }
        let messages = if self.moved == 1 {
            "1 message".to_string()
        } else {
            format!("{} messages", self.moved)
        };
        let whose = match self.from_accounts.as_slice() {
            [] => String::new(),
            [only] => format!(" from {only}"),
            [rest @ .., last] => format!(" from {} and {last}", rest.join(", ")),
        };
        format!(
            "Your accounts now share one Sent, Outbox, Drafts, Junk and Trash on this computer. \
             {messages}{whose} moved into them."
        )
    }
}

impl MessageCache {
    /// What the merge did when this cache was opened, if it ran.
    ///
    /// Read once, on the way in, by whoever opened the cache. It happens
    /// unasked and it moves somebody's mail, so they are owed a sentence about
    /// it, and this is the only moment that sentence exists.
    pub fn merge_of_local_folders(&self) -> Option<&MergeReport> {
        self.merge_of_local_folders.as_ref()
    }

    /// Move every account's local Sent, Outbox, Drafts, Junk and Trash into the
    /// five shared ones.
    ///
    /// Run once on open from `MessageCache::new`, and not fatal there: the mail
    /// is still readable where it is and the next open tries again.
    pub fn merge_local_folders(&self) -> Result<MergeReport> {
        // Property 1: the whole candidate set first, into a Vec, before
        // anything is written. A statement held open over `messages` while
        // `messages` is being written is how a migration stops halfway with
        // nothing to say for itself.
        //
        // Property 5: this is also what makes a second open a no-op. It asks
        // for messages still sitting in a per-account copy of a shared folder,
        // and after a complete run there are none.
        let shared_paths: Vec<String> = SHARED_BY_EVERY_ACCOUNT
            .iter()
            .map(|folder| folder.path())
            .collect();
        let placeholders = vec!["?"; shared_paths.len()].join(", ");

        let mut stmt = self
            .conn
            .prepare(&format!(
                "SELECT m.id, f.id, f.account_id, f.path
                 FROM messages m JOIN folders f ON m.folder_id = f.id
                 WHERE f.account_id <> ?1 AND f.path IN ({placeholders})
                 ORDER BY f.account_id, m.id",
            ))
            .map_err(|e| {
                Error::Other(format!("Failed to look for mail to bring together: {}", e))
            })?;

        let mut arguments: Vec<&dyn rusqlite::ToSql> = vec![&THIS_COMPUTER];
        for path in &shared_paths {
            arguments.push(path);
        }

        let pending: Vec<(i64, i64, String, String)> = stmt
            .query_map(arguments.as_slice(), |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .map_err(|e| Error::Other(format!("Failed to read the mail to bring together: {}", e)))?
            .collect::<std::result::Result<_, _>>()
            .map_err(|e| Error::Other(format!("Failed to read a row of it: {}", e)))?;
        drop(stmt);

        let mut report = MergeReport {
            found: pending.len(),
            ..MergeReport::default()
        };
        if pending.is_empty() {
            return Ok(report);
        }

        // The rows the mail is coming out of, each once, so they can be put
        // away after every one of their messages has landed.
        let mut emptied: Vec<(i64, String)> = Vec::new();

        for (message_id, out_of, came_from, path) in pending {
            if !emptied.iter().any(|(id, _)| *id == out_of) {
                emptied.push((out_of, path.clone()));
            }
            // Property 2: the message lands before the old row lets go of it.
            // One statement does both, which is stronger than doing them in
            // order: there is no moment at which the message is in neither
            // folder, even if the process stops between two messages.
            //
            // Property 4 as well: the account's own folder row is left alone.
            // What leaves it is the mail.
            //
            // A message that cannot move does not stop the ones after it. One
            // unmovable message in one account's Trash would otherwise leave
            // every other account's mail where it was, unmerged, on every open
            // for ever. It stays where it is, which is safe, and `found`
            // against `moved` is what says some did not make it.
            let landed = self.shared_folder_for(&path).and_then(|into| {
                self.move_message_recording_its_origin(message_id, into, &came_from)
            });
            let renumbering = match landed {
                Ok(renumbering) => renumbering,
                Err(e) => {
                    tracing::warn!(
                        "A message could not be brought across and was left where it is: {e}"
                    );
                    continue;
                }
            };

            report.moved += 1;
            if renumbering.changed() {
                report.renumbered += 1;
            }
            if !report.from_accounts.contains(&came_from) {
                report.from_accounts.push(came_from);
            }
        }

        // Now, and only now, the per-account rows those messages came out of.
        // Every one of them has landed, which is what D-19's "nothing is
        // removed until every message has landed" licenses removing them after,
        // and what the invariant below requires: an emptied row left behind is
        // still a per-account Sent, and the tree would show one under every
        // account beside the shared one.
        //
        // `folders` cascades to `messages`, so deleting a row that still holds
        // mail destroys that mail. A message that could not be brought across
        // is exactly that case, and it is not hypothetical: the loop above
        // carries on past one. The emptiness test is therefore part of the
        // statement rather than a check before it, so there is no window
        // between asking and deleting for the answer to change.
        for (folder_id, _) in emptied {
            self.conn
                .execute(
                    "DELETE FROM folders
                     WHERE id = ?1
                       AND NOT EXISTS (SELECT 1 FROM messages WHERE folder_id = ?1)",
                    rusqlite::params![folder_id],
                )
                .map_err(|e| Error::Other(format!("Failed to put away the old folder: {}", e)))?;
        }

        // Property 3: this is a log line and not the contract. What a caller
        // acts on is the report above.
        tracing::info!(
            "Brought {} of {} messages into the folders shared by every account, {} renumbered",
            report.moved,
            report.found,
            report.renumbered
        );
        Ok(report)
    }

    /// The row for a shared folder at this path, made if it is not there yet.
    ///
    /// Saving is an upsert keyed on the account and the path, so this returns
    /// the same row every time after the first.
    fn shared_folder_for(&self, path: &str) -> Result<i64> {
        if let Some(existing) = self.get_folder(THIS_COMPUTER, path)? {
            return Ok(existing.id);
        }
        let kind = SHARED_BY_EVERY_ACCOUNT
            .iter()
            .find(|folder| folder.path() == path)
            .ok_or_else(|| Error::Other(format!("{path} is not a folder every account shares")))?;

        let id = self.save_folder(&super::CachedFolder {
            id: 0,
            account_id: THIS_COMPUTER.to_string(),
            name: kind.name.to_string(),
            path: path.to_string(),
            folder_type: kind.kind.as_str().to_string(),
            unread_count: 0,
            total_count: 0,
        })?;
        // Never opened over the network, whatever else decides what syncs.
        self.set_folder_server_facts(id, false, true)?;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::local_folders::LOCAL_PREFIX;
    use crate::data::message_cache::{CachedFolder, IncomingMessage};

    fn a_cache() -> (tempfile::TempDir, MessageCache) {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
        (dir, cache)
    }

    /// A folder row, under whichever account is named.
    fn a_folder(cache: &MessageCache, account: &str, name: &str) -> i64 {
        cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: account.to_string(),
                name: name.to_string(),
                path: format!("{LOCAL_PREFIX}/{name}"),
                folder_type: name.to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder")
    }

    /// A message in a folder, numbered as asked.
    fn a_message(cache: &MessageCache, folder_id: i64, uid: u32, subject: &str) -> i64 {
        cache
            .upsert_message(&IncomingMessage {
                folder_id,
                uid,
                message_id: format!("<{subject}@example.com>"),
                subject: subject.to_string(),
                from_addr: "them@example.com".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                reply_to: None,
                date: "2026-08-30T09:00:00Z".to_string(),
                internal_date: Some("2026-08-30T09:00:00Z".to_string()),
                size_bytes: Some(10),
                refs_header: None,
                read: false,
                starred: false,
                answered: false,
                draft: false,
                deleted: false,
                has_attachments: false,
                safety: crate::service::safety::Verdict::ordinary(),
                gmail_message_id: None,
                labels: None,
                receipt_to: None,
                pop_uidl: None,
            })
            .expect("a message")
    }

    /// Where a message is now, and what it is numbered.
    fn whereabouts(
        cache: &MessageCache,
        message_id: i64,
    ) -> (i64, u32, Option<u32>, Option<String>) {
        cache
            .conn
            .query_row(
                "SELECT folder_id, uid, original_uid, original_account_id
                 FROM messages WHERE id = ?1",
                rusqlite::params![message_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)? as u32,
                        row.get::<_, Option<i64>>(2)?.map(|uid| uid as u32),
                        row.get::<_, Option<String>>(3)?,
                    ))
                },
            )
            .expect("the message is still there")
    }

    fn how_many_messages(cache: &MessageCache) -> i64 {
        cache
            .conn
            .query_row("SELECT COUNT(*) FROM messages", [], |row| row.get(0))
            .expect("a count")
    }

    #[test]
    fn test_two_accounts_trash_becomes_one_and_the_report_names_both() {
        // The ordinary case, and the one a fixture of nothing but collisions
        // cannot tell a working merge from a do-nothing one on. Six plain
        // messages, no clash between any of them, all six in one Trash
        // afterwards.
        let (_dir, cache) = a_cache();
        let one = a_folder(&cache, "one", "Trash");
        let two = a_folder(&cache, "two", "Trash");
        for uid in 1..=3 {
            a_message(&cache, one, uid, &format!("one-{uid}"));
        }
        for uid in 10..=12 {
            a_message(&cache, two, uid, &format!("two-{uid}"));
        }

        let report = cache.merge_local_folders().expect("the merge runs");

        assert_eq!(report.found, 6, "it did not see all six before moving");
        assert_eq!(report.moved, 6);
        assert_eq!(
            report.from_accounts,
            vec!["one".to_string(), "two".to_string()]
        );

        let shared = cache
            .get_folder(THIS_COMPUTER, &format!("{LOCAL_PREFIX}/Trash"))
            .expect("the lookup")
            .expect("the shared Trash was made");
        let landed: i64 = cache
            .conn
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE folder_id = ?1",
                rusqlite::params![shared.id],
                |row| row.get(0),
            )
            .expect("a count");
        assert_eq!(landed, 6, "not every message landed in the shared Trash");
        assert_eq!(how_many_messages(&cache), 6, "a message went missing");
    }

    #[test]
    fn test_two_messages_numbered_the_same_both_survive_and_one_records_its_old_number() {
        // The case the merge exists to handle. UNIQUE(folder_id, uid) means
        // carrying both numbers across is impossible, and two accounts' Trash
        // both holding uid 42 is expected rather than hypothetical: IMAP
        // numbers are per mailbox and POP ones per download.
        let (_dir, cache) = a_cache();
        let one = a_folder(&cache, "one", "Trash");
        let two = a_folder(&cache, "two", "Trash");
        let first = a_message(&cache, one, 42, "the first");
        let second = a_message(&cache, two, 42, "the second");

        let report = cache.merge_local_folders().expect("the merge runs");

        assert_eq!(report.moved, 2, "a message with a clashing number was lost");
        assert_eq!(how_many_messages(&cache), 2);

        let (first_folder, first_uid, first_was, first_from) = whereabouts(&cache, first);
        let (second_folder, second_uid, second_was, second_from) = whereabouts(&cache, second);

        assert_eq!(
            first_folder, second_folder,
            "the two did not end up in one folder"
        );
        assert_ne!(
            first_uid, second_uid,
            "two messages in one folder were left holding the same number"
        );
        assert_eq!(first_was, Some(42), "the first forgot the number it had");
        assert_eq!(second_was, Some(42), "the second forgot the number it had");
        assert_eq!(first_from.as_deref(), Some("one"));
        assert_eq!(second_from.as_deref(), Some("two"));
    }

    #[test]
    fn test_every_moved_message_records_where_it_came_from_not_only_the_clashing_ones() {
        // What makes the merge reversible. Recording the account only for the
        // messages that had to be renumbered would leave most of them with
        // nothing saying whose they were, and the merge rewrites the only copy
        // of that mail.
        let (_dir, cache) = a_cache();
        let sent = a_folder(&cache, "one", "Sent");
        let alone = a_message(&cache, sent, 1, "nothing clashes with this");

        cache.merge_local_folders().expect("the merge runs");

        let (_, _, was, from) = whereabouts(&cache, alone);
        assert_eq!(from.as_deref(), Some("one"), "it forgot whose it was");
        assert_eq!(was, Some(1), "it forgot the number it had");
    }

    #[test]
    fn test_a_second_open_finds_nothing_to_move() {
        // Idempotent by the WHERE clause. Opening the database again must not
        // move anything a second time, renumber anything a second time, or
        // overwrite the record of where a message first came from.
        let (_dir, cache) = a_cache();
        let trash = a_folder(&cache, "one", "Trash");
        let message = a_message(&cache, trash, 7, "moved once");

        let first = cache.merge_local_folders().expect("the first merge");
        let after_first = whereabouts(&cache, message);
        let again = cache.merge_local_folders().expect("the second merge");

        assert_eq!(first.moved, 1);
        assert_eq!(again.found, 0, "it found work to do a second time");
        assert_eq!(again.moved, 0);
        assert_eq!(
            whereabouts(&cache, message),
            after_first,
            "a second open moved the message again"
        );
    }

    #[test]
    fn test_an_empty_database_and_one_with_no_local_mail_both_merge_to_nothing() {
        let (_dir, empty) = a_cache();
        assert_eq!(
            empty.merge_local_folders().expect("the merge runs"),
            MergeReport::default()
        );

        let (_dir2, quiet) = a_cache();
        a_folder(&quiet, "one", "Trash");
        let report = quiet.merge_local_folders().expect("the merge runs");
        assert_eq!(report.found, 0);
        assert_eq!(report.moved, 0);
    }

    #[test]
    fn test_the_old_folder_row_is_put_away_once_every_message_has_landed() {
        // D-19 removes nothing until every message has landed, and then the
        // per-account row has nothing left to be. Leaving it would put an empty
        // Trash under every account beside the shared one, which is the shape
        // this whole plan removes.
        let (_dir, cache) = a_cache();
        let trash = a_folder(&cache, "one", "Trash");
        a_message(&cache, trash, 1, "off it goes");

        cache.merge_local_folders().expect("the merge runs");

        assert!(
            cache
                .get_folder("one", &format!("{LOCAL_PREFIX}/Trash"))
                .expect("the lookup")
                .is_none(),
            "the account kept a Trash of its own beside the shared one"
        );
        assert_eq!(
            how_many_messages(&cache),
            1,
            "the message went with the row"
        );
    }

    #[test]
    fn test_a_folder_that_still_holds_mail_is_never_put_away() {
        // `folders` cascades to `messages`, so deleting a row that still holds
        // mail destroys that mail. This is the same partial run as the test
        // above, asked of the folder rather than of the messages: the row it
        // could not empty has to still be there.
        let (_dir, cache) = a_cache();
        let trash = a_folder(&cache, "one", "Trash");
        for uid in 1..=3 {
            a_message(&cache, trash, uid, &format!("keep-{uid}"));
        }
        let shared = a_folder(&cache, THIS_COMPUTER, "Trash");
        cache
            .conn
            .execute_batch(&format!(
                "CREATE TRIGGER nothing_lands BEFORE UPDATE ON messages
                 WHEN NEW.folder_id = {shared} AND OLD.subject = 'keep-3'
                 BEGIN SELECT RAISE(ABORT, 'it could not land'); END"
            ))
            .expect("the trigger");

        let _ = cache.merge_local_folders();

        assert!(
            cache
                .get_folder("one", &format!("{LOCAL_PREFIX}/Trash"))
                .expect("the lookup")
                .is_some(),
            "a folder still holding mail was put away, and cascading took the mail with it"
        );
        assert_eq!(how_many_messages(&cache), 3, "a message was lost");
    }

    #[test]
    fn test_afterwards_no_account_keeps_a_copy_of_a_folder_that_is_shared() {
        // The invariant, and what keeps the promotion decided rather than
        // drifting back. It goes red the day something reintroduces a
        // per-account local Trash, whether by making one or by leaving one
        // behind after the merge.
        //
        // Asked of the folder rows rather than only of the messages, because a
        // per-account row with no mail in it is still a second Trash in the
        // tree, and that is the repetition D-18 removes.
        let (_dir, cache) = a_cache();
        for account in ["one", "two"] {
            for name in ["Inbox", "Sent", "Outbox", "Drafts", "Junk", "Trash"] {
                let id = a_folder(&cache, account, name);
                a_message(&cache, id, 1, &format!("{account}-{name}"));
            }
        }

        cache.merge_local_folders().expect("the merge runs");

        let stragglers: Vec<String> = cache
            .conn
            .prepare(
                "SELECT account_id || ' ' || path FROM folders
                 WHERE account_id <> ?1 AND path LIKE ?2",
            )
            .expect("the query")
            .query_map(
                rusqlite::params![THIS_COMPUTER, format!("{LOCAL_PREFIX}%")],
                |row| row.get::<_, String>(0),
            )
            .expect("the rows")
            .collect::<std::result::Result<Vec<_>, _>>()
            .expect("a row")
            .into_iter()
            .filter(|row| !row.ends_with("/Inbox"))
            .collect();

        assert!(
            stragglers.is_empty(),
            "an account kept its own copy of a folder that is shared: {stragglers:?}"
        );
        // And the Inbox is untouched, which is the other half of D-18 and what
        // stops this invariant being satisfied by merging everything.
        assert!(
            cache
                .get_folder("one", &format!("{LOCAL_PREFIX}/Inbox"))
                .expect("the lookup")
                .is_some(),
            "an account's own inbox was merged away"
        );
        assert_eq!(how_many_messages(&cache), 12, "a message was lost");
    }

    #[test]
    fn test_an_accounts_inbox_is_not_touched_because_it_is_not_shared() {
        // Only the five move. An Inbox on this computer is the one folder that
        // stays per account, and moving it would put two accounts' incoming
        // mail in one list nobody asked for.
        let (_dir, cache) = a_cache();
        let inbox = a_folder(&cache, "one", "Inbox");
        let message = a_message(&cache, inbox, 3, "still mine");

        let report = cache.merge_local_folders().expect("the merge runs");

        assert_eq!(report.moved, 0);
        let (folder, uid, was, from) = whereabouts(&cache, message);
        assert_eq!(folder, inbox, "an account's inbox was merged away");
        assert_eq!(uid, 3);
        assert_eq!(was, None);
        assert_eq!(from, None);
    }

    #[test]
    fn test_the_count_is_the_returned_report_rather_than_a_log_line() {
        // Stated as a test because it is the contract. Nothing here captures
        // tracing output, on the established convention that a log line is not
        // part of a function's contract, so the numbers a caller acts on have
        // to come back from the call.
        let (_dir, cache) = a_cache();
        let drafts = a_folder(&cache, "one", "Drafts");
        a_message(&cache, drafts, 1, "a draft");
        a_message(&cache, drafts, 2, "another");

        let report = cache.merge_local_folders().expect("the merge runs");

        assert_eq!(report.found, 2);
        assert_eq!(report.moved, 2);
        assert!(report.anything_happened());
        assert!(
            report.said().contains('2'),
            "the sentence does not carry the count: {:?}",
            report.said()
        );
        assert!(
            report.said().contains("one"),
            "the sentence does not say where the mail came from: {:?}",
            report.said()
        );
    }

    #[test]
    fn test_a_merge_that_stops_partway_has_landed_what_it_moved_and_lost_nothing() {
        // Nothing is removed until it has landed, one message at a time, so a
        // merge that cannot finish leaves every message either where it came
        // from or where it was going and never in neither.
        //
        // The third message is stopped by a trigger rather than by arranging a
        // number that clashes, because the numbering has to stay exactly as the
        // application does it for the first two to prove anything.
        //
        // Asserting that the first two landed, rather than only that nothing
        // was lost, is what makes this test discriminate: a merge that did
        // nothing at all would also lose nothing.
        let (_dir, cache) = a_cache();
        let trash = a_folder(&cache, "one", "Trash");
        for uid in 1..=3 {
            a_message(&cache, trash, uid, &format!("keep-{uid}"));
        }
        let shared = a_folder(&cache, THIS_COMPUTER, "Trash");
        cache
            .conn
            .execute_batch(&format!(
                "CREATE TRIGGER the_third_cannot_land BEFORE UPDATE ON messages
                 WHEN NEW.folder_id = {shared} AND OLD.subject = 'keep-3'
                 BEGIN SELECT RAISE(ABORT, 'it could not land'); END"
            ))
            .expect("the trigger");
        let before = how_many_messages(&cache);

        let report = cache.merge_local_folders().expect("the merge runs");

        assert_eq!(report.found, 3, "it did not see all three before moving");
        assert_eq!(
            report.moved, 2,
            "one message could not move, so two should have"
        );
        assert_eq!(
            how_many_messages(&cache),
            before,
            "a message was lost when the move could not complete"
        );

        let landed: i64 = cache
            .conn
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE folder_id = ?1",
                rusqlite::params![shared],
                |row| row.get(0),
            )
            .expect("a count");
        let left: i64 = cache
            .conn
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE folder_id = ?1",
                rusqlite::params![trash],
                |row| row.get(0),
            )
            .expect("a count");

        assert_eq!(
            landed, 2,
            "the two that could land did not, so this proves nothing about a partial run"
        );
        assert_eq!(left, 1, "a message left its old folder without arriving");
        assert_eq!(landed + left, before, "a message is in neither folder");
    }
}

//! Deleting a message that lives on this computer rather than on a server.
//!
//! A POP account has no server-side folders, no flags and no delete worth the
//! name: POP3 removes mail from the mailbox for good, and whether that happens
//! is decided by the account's own removal policy and by nothing else. So
//! deleting a downloaded message is entirely a local matter, and the route that
//! asks a server about it can only ever fail. What it used to say when it did
//! was that the account had no usable IMAP port, which is not the problem, and
//! that the change had been undone, which had not happened either.
//!
//! What decides whether this path applies is the folder the message is in, not
//! the account's protocol. That keeps an IMAP account's behaviour identical,
//! and it is the right question anyway: an IMAP account's Outbox is on this
//! computer too.
//!
//! # Nothing here reaches a server
//!
//! Not the POP server and not any other. Everything below is a row moving
//! between two folders in the local database, which is why the sentences it
//! hands back say "on this computer" and never "deleted": mail stays in the POP
//! mailbox until the account's removal policy takes it, and telling somebody it
//! had gone from a place it is still sitting in is the kind of wrong that is
//! only found out from another device.

use crate::application::destinations::Deleting;
use crate::application::local_folders::{self, LocalDelete};
use crate::common::Result;
use crate::common::types::FolderType;
use crate::data::account::Account;
use crate::data::message_cache::{CachedFolder, MessageCache};

/// What happened, and what to say about it.
pub struct Outcome {
    /// Whether the row should leave the list somebody is looking at.
    ///
    /// False on a refusal, and only on a refusal. A message that moved and a
    /// message that was taken off the machine have both left the folder it was
    /// selected in.
    pub message_left_the_folder: bool,
    /// The sentence for the status line, in the words for what actually
    /// happened.
    pub said: String,
}

/// Delete a message that lives here, if it does.
///
/// `None` means the message is on a server and this is not the path for it, so
/// the caller carries on to the one that asks the server. Every other answer,
/// including a refusal, has been dealt with here.
pub fn perform(
    cache: &MessageCache,
    account: &Account,
    message_row_id: i64,
    asked: Deleting,
) -> Result<Option<Outcome>> {
    let Some(folder_path) = cache.folder_path_for_message(message_row_id)? else {
        return Ok(None);
    };
    let Some(what) = local_folders::deleting(
        &folder_path,
        account.protocol(),
        asked,
        account.allow_deleting_here,
    ) else {
        return Ok(None);
    };

    match what {
        LocalDelete::Refuse(why) => Ok(Some(Outcome {
            message_left_the_folder: false,
            said: why.to_string(),
        })),
        LocalDelete::MoveTo(trash) => {
            let into = folder_here(cache, account, &trash)?;
            cache.move_message(message_row_id, into)?;
            Ok(Some(Outcome {
                message_left_the_folder: true,
                said: "Moved to the Trash folder on this computer".to_string(),
            }))
        }
        LocalDelete::RemoveFromThisComputer => {
            cache.delete_message(message_row_id)?;
            Ok(Some(Outcome {
                message_left_the_folder: true,
                said: "Deleted from this computer".to_string(),
            }))
        }
    }
}

/// The identifier of one of this account's local folders, making it if it is
/// not there yet.
///
/// An account that has never completed a check for mail has no folder rows, and
/// somebody deleting their first message should not be told to go and fix
/// something they cannot see. Saving is an upsert keyed on the account and the
/// path, so calling this when the row already exists returns the same one.
fn folder_here(cache: &MessageCache, account: &Account, path: &str) -> Result<i64> {
    if let Some(existing) = cache
        .get_folders_for_account(&account.id)?
        .into_iter()
        .find(|folder| folder.path == path)
    {
        return Ok(existing.id);
    }

    let name = local_folders::for_account(account.protocol())
        .iter()
        .find(|folder| folder.path() == path);
    let id = cache.save_folder(&CachedFolder {
        id: 0,
        account_id: account.id.clone(),
        name: name.map_or("Trash", |folder| folder.name).to_string(),
        path: path.to_string(),
        folder_type: name
            .map_or(FolderType::Trash, |folder| folder.kind)
            .as_str()
            .to_string(),
        unread_count: 0,
        total_count: 0,
    })?;
    // Never opened over the network, whatever else decides what is synced.
    cache.set_folder_server_facts(id, false, true)?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use crate::application::destinations::Deleting;
    use crate::application::local_delete::perform;
    use crate::application::local_folders::{self, LOCAL_PREFIX};
    use crate::common::temp_home::TempHome;
    use crate::common::types::Protocol;
    use crate::data::account::Account;
    use crate::data::message_cache::{CachedFolder, IncomingMessage, MessageCache};

    /// A cache in a directory of its own, so tests do not share a database.
    fn a_cache(what_for: &str) -> TempHome<MessageCache> {
        TempHome::named(what_for, |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
        })
    }

    fn a_pop_account() -> Account {
        let mut account = Account::new("Home".to_string(), "me@example.com".to_string());
        account.id = "acct".to_string();
        account.protocol = Protocol::Pop3.as_str().to_string();
        account
    }

    /// The folders a POP account really gets, built the way the application
    /// builds them, so these tests exercise the rows it actually writes.
    fn its_folders(cache: &MessageCache, account: &Account) {
        for folder in local_folders::for_account(account.protocol()) {
            let id = cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: account.id.clone(),
                    name: folder.name.to_string(),
                    path: folder.path(),
                    folder_type: folder.kind.as_str().to_string(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("a folder");
            cache
                .set_folder_server_facts(id, false, true)
                .expect("facts about the folder");
        }
    }

    fn a_folder(cache: &MessageCache, account_id: &str, path: &str, kind: &str) -> i64 {
        cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: account_id.to_string(),
                name: path.to_string(),
                path: path.to_string(),
                folder_type: kind.to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder")
    }

    fn a_message(cache: &MessageCache, folder_id: i64, subject: &str) -> i64 {
        cache
            .upsert_message(&IncomingMessage {
                folder_id,
                uid: cache.next_local_uid(folder_id).expect("a number"),
                message_id: format!("<{subject}@example.com>"),
                subject: subject.to_string(),
                from_addr: "Ada <ada@example.com>".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                reply_to: None,
                date: "2026-07-26T10:00:00+00:00".to_string(),
                internal_date: None,
                size_bytes: Some(64),
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
                pop_uidl: Some(subject.to_string()),
            })
            .expect("a stored message")
    }

    fn path_of(cache: &MessageCache, row: i64) -> String {
        cache
            .folder_path_for_message(row)
            .expect("the lookup")
            .expect("a folder")
    }

    #[test]
    fn test_deleting_a_downloaded_message_moves_it_to_the_trash_on_this_computer() {
        let cache = a_cache("moves_to_trash");
        let account = a_pop_account();
        its_folders(&cache, &account);
        let inbox = a_folder(
            &cache,
            &account.id,
            &format!("{LOCAL_PREFIX}/Inbox"),
            "Inbox",
        );
        let row = a_message(&cache, inbox, "Notes on the engine");

        let outcome = perform(&cache, &account, row, Deleting::ToTrash)
            .expect("the delete runs")
            .expect("a message on this computer is this path's business");

        assert!(outcome.message_left_the_folder);
        assert_eq!(path_of(&cache, row), format!("{LOCAL_PREFIX}/Trash"));
    }

    #[test]
    fn test_deleting_from_the_trash_removes_it_and_says_so() {
        let cache = a_cache("removes_from_trash");
        let account = a_pop_account();
        its_folders(&cache, &account);
        let trash = a_folder(
            &cache,
            &account.id,
            &format!("{LOCAL_PREFIX}/Trash"),
            "Trash",
        );
        let row = a_message(&cache, trash, "Already binned");

        let outcome = perform(&cache, &account, row, Deleting::ToTrash)
            .expect("the delete runs")
            .expect("a message on this computer is this path's business");

        assert!(outcome.message_left_the_folder);
        assert!(
            cache
                .get_message_list(trash, &account.id)
                .expect("the list")
                .is_empty(),
            "it is still in the trash"
        );
        assert!(
            outcome.said.contains("this computer"),
            "the sentence does not say where it went: {}",
            outcome.said
        );
    }

    #[test]
    fn test_the_trash_folder_is_made_if_it_is_not_there_yet() {
        // An account that has never successfully checked for mail has no
        // folders. Delete has to work on the very first message all the same,
        // rather than refusing for a reason nobody can act on.
        let cache = a_cache("makes_the_trash");
        let account = a_pop_account();
        let inbox = a_folder(
            &cache,
            &account.id,
            &format!("{LOCAL_PREFIX}/Inbox"),
            "Inbox",
        );
        let row = a_message(&cache, inbox, "The first one");

        perform(&cache, &account, row, Deleting::ToTrash)
            .expect("the delete runs")
            .expect("a message on this computer is this path's business");

        assert_eq!(path_of(&cache, row), format!("{LOCAL_PREFIX}/Trash"));
    }

    #[test]
    fn test_the_trash_made_on_the_first_delete_is_called_trash_and_stays_off_the_network() {
        // The path alone is not enough. The row also carries the name the
        // folder tree reads out and the type everything else switches on, and
        // taking the wrong entry from the list of local folders gives a second
        // folder announced as Inbox that holds the deleted mail. The facts say
        // it is never opened at a server, which is what stops a sync trying.
        let cache = a_cache("names_the_trash");
        let account = a_pop_account();
        let inbox = a_folder(
            &cache,
            &account.id,
            &format!("{LOCAL_PREFIX}/Inbox"),
            "Inbox",
        );
        let row = a_message(&cache, inbox, "The first one");

        perform(&cache, &account, row, Deleting::ToTrash)
            .expect("the delete runs")
            .expect("a message on this computer is this path's business");

        let path = format!("{LOCAL_PREFIX}/Trash");
        let made = cache
            .get_folders_for_account(&account.id)
            .expect("the folder list")
            .into_iter()
            .find(|folder| folder.path == path)
            .expect("the trash was made");
        assert_eq!(
            made.name, "Trash",
            "the folder tree would announce it wrong"
        );
        assert_eq!(
            made.folder_type,
            crate::common::types::FolderType::Trash.as_str()
        );
        assert_eq!(
            cache
                .folder_server_facts(&account.id)
                .expect("the facts")
                .get(&path),
            Some(&(false, true)),
            "a folder on this computer was left looking like one to open at the server"
        );
    }

    #[test]
    fn test_an_account_with_deleting_switched_off_is_told_the_real_reason() {
        // What was said before named an IMAP port, which a POP account does not
        // have and never needed, and then claimed the change had been undone
        // when nothing had been done in the first place.
        let cache = a_cache("switched_off");
        let mut account = a_pop_account();
        account.allow_deleting_here = false;
        its_folders(&cache, &account);
        let inbox = a_folder(
            &cache,
            &account.id,
            &format!("{LOCAL_PREFIX}/Inbox"),
            "Inbox",
        );
        let row = a_message(&cache, inbox, "Still wanted");

        let outcome = perform(&cache, &account, row, Deleting::ToTrash)
            .expect("the delete runs")
            .expect("a message on this computer is this path's business");

        assert!(!outcome.message_left_the_folder);
        assert_eq!(
            path_of(&cache, row),
            format!("{LOCAL_PREFIX}/Inbox"),
            "the message moved anyway"
        );
        let lowered = outcome.said.to_lowercase();
        for machinery in ["imap", "port", "undone"] {
            assert!(
                !lowered.contains(machinery),
                "the refusal blames {machinery}: {}",
                outcome.said
            );
        }
    }

    #[test]
    fn test_shift_delete_takes_it_off_this_computer_without_a_trip_to_the_trash() {
        let cache = a_cache("outright");
        let account = a_pop_account();
        its_folders(&cache, &account);
        let inbox = a_folder(
            &cache,
            &account.id,
            &format!("{LOCAL_PREFIX}/Inbox"),
            "Inbox",
        );
        let row = a_message(&cache, inbox, "Gone for good");

        let outcome = perform(&cache, &account, row, Deleting::Outright)
            .expect("the delete runs")
            .expect("a message on this computer is this path's business");

        assert!(outcome.message_left_the_folder);
        assert_eq!(
            path_of(&cache, row),
            format!("{LOCAL_PREFIX}/Inbox"),
            "it was moved to the trash after being asked for outright"
        );
        assert!(
            cache
                .get_message_list(inbox, &account.id)
                .expect("the list")
                .is_empty()
        );
    }

    #[test]
    fn test_a_message_on_a_server_is_left_for_the_server_path() {
        // `None` is what keeps every IMAP account behaving exactly as it does
        // today: the route that asks the server runs unchanged.
        let cache = a_cache("server_message");
        let mut account = a_pop_account();
        account.protocol = Protocol::Imap.as_str().to_string();
        let inbox = a_folder(&cache, &account.id, "INBOX", "Inbox");
        let row = a_message(&cache, inbox, "On the server");

        assert!(
            perform(&cache, &account, row, Deleting::ToTrash)
                .expect("the lookup runs")
                .is_none()
        );
    }

    #[test]
    fn test_a_message_in_no_folder_we_know_about_is_left_alone() {
        let cache = a_cache("no_folder");
        let account = a_pop_account();

        assert!(
            perform(&cache, &account, 404, Deleting::ToTrash)
                .expect("the lookup runs")
                .is_none()
        );
    }

    #[test]
    fn test_nothing_said_here_claims_a_server_did_anything() {
        // POP3 has no delete this path uses. Mail stays on the server until the
        // account's own removal policy takes it, so a sentence mentioning a
        // server would be telling somebody their mail had gone from a place it
        // is still sitting in.
        let cache = a_cache("says_nothing_about_a_server");
        let account = a_pop_account();
        its_folders(&cache, &account);
        let inbox = a_folder(
            &cache,
            &account.id,
            &format!("{LOCAL_PREFIX}/Inbox"),
            "Inbox",
        );
        let mut refusing = a_pop_account();
        refusing.allow_deleting_here = false;

        let every_sentence = [
            perform(
                &cache,
                &account,
                a_message(&cache, inbox, "One"),
                Deleting::ToTrash,
            ),
            perform(
                &cache,
                &account,
                a_message(&cache, inbox, "Two"),
                Deleting::Outright,
            ),
            perform(
                &cache,
                &refusing,
                a_message(&cache, inbox, "Three"),
                Deleting::ToTrash,
            ),
        ];

        for outcome in every_sentence {
            let said = outcome
                .expect("the delete runs")
                .expect("a message on this computer is this path's business")
                .said;
            assert!(
                !said.to_lowercase().contains("server"),
                "a sentence about the server: {said}"
            );
            assert!(!said.trim().is_empty(), "nothing was said at all");
        }
    }
}

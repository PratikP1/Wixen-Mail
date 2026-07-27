//! Bringing a mailbox down from the server into the cache.
//!
//! The decisions here are about scale and about honesty. A folder may hold two
//! hundred thousand messages, so a sync cannot be "fetch everything" and cannot
//! be one call that either works or does not. And whatever it does has to leave
//! the cache agreeing with the server, because a listing that shows a message
//! the server deleted is worse than a listing that is a little behind: the
//! reader arrows onto a row, presses Enter, and gets an error instead of mail.
//!
//! What gets fetched is decided by the pure functions at the top of this file,
//! which is where the tests are. The driver underneath moves bytes.

use crate::application::mail_controller::MailController;
use crate::common::{Result, types::FolderType};
use crate::data::message_cache::{CachedFolder, IncomingMessage, MessageCache};
use crate::service::protocols::imap::{ImapFolder, ImapMessage};

/// How many messages a first look at a folder brings down.
///
/// Enough that somebody opening their inbox sees a full screen and can keep
/// arrowing for a long time; small enough that the first sync of an old
/// mailbox finishes rather than appearing to hang. The rest arrive when the
/// list is asked to go further back.
pub const INITIAL_FETCH_LIMIT: usize = 500;

/// What one folder's sync did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FolderSync {
    /// The folder's name, as it would be announced.
    pub folder: String,
    pub fetched: usize,
    pub forgotten: usize,
    pub total_on_server: usize,
    /// Whether the server had renumbered the mailbox since the last sync.
    pub renumbered: bool,
}

/// Which messages to fetch headers for, newest first, bounded.
///
/// Newest first because that is where a reader starts, and bounded because the
/// alternative on a large mailbox is a sync that never visibly finishes.
/// Returned in ascending order, which is how they go into a sequence set.
pub fn uids_to_fetch(on_server: &[u32], stored: &[u32], limit: usize) -> Vec<u32> {
    if limit == 0 {
        return Vec::new();
    }
    let held: std::collections::HashSet<u32> = stored.iter().copied().collect();
    let mut missing: Vec<u32> = on_server
        .iter()
        .copied()
        .filter(|uid| !held.contains(uid))
        .collect();

    // Descending to take the newest, then back to ascending for the caller.
    missing.sort_unstable_by(|a, b| b.cmp(a));
    missing.truncate(limit);
    missing.sort_unstable();
    missing
}

/// Which stored messages the server no longer has.
///
/// Deleted from another client, or expunged. Leaving them listed means a reader
/// arrows onto a row, presses Enter, and gets an error rather than a message.
///
/// The server list must be the whole mailbox, not the page just fetched.
/// Comparing against a page would delete everything outside it.
pub fn uids_to_forget(on_server: &[u32], stored: &[u32]) -> Vec<u32> {
    let present: std::collections::HashSet<u32> = on_server.iter().copied().collect();
    let mut gone: Vec<u32> = stored
        .iter()
        .copied()
        .filter(|uid| !present.contains(uid))
        .collect();
    gone.sort_unstable();
    gone
}

/// Turn a fetched message into the row the cache stores.
pub fn to_incoming(message: &ImapMessage, folder_id: i64) -> IncomingMessage {
    IncomingMessage {
        folder_id,
        uid: message.uid,
        message_id: message.message_id.clone().unwrap_or_default(),
        subject: message.subject.clone(),
        from_addr: join_addresses(&message.from),
        to_addr: join_addresses(&message.to),
        cc: Some(join_addresses(&message.cc)).filter(|cc| !cc.is_empty()),
        // The sender's date when it is usable, the server's when it is not, so
        // the column is never blank and the folder never sorts a message to an
        // end its reader will not look at.
        date: message
            .date
            .clone()
            .or_else(|| message.internal_date.clone())
            .unwrap_or_default(),
        internal_date: message.internal_date.clone(),
        size_bytes: Some(i64::from(message.size)),
        refs_header: reference_chain(message),
        read: message.seen(),
        starred: message.flagged(),
        deleted: message.deleted(),
        has_attachments: message.has_attachments,
    }
}

/// The whole ancestry a message names, as one stored string.
///
/// `In-Reply-To` is appended when it is not already in `References`, because
/// some senders write only one of the two and threading needs whichever
/// arrived.
fn reference_chain(message: &ImapMessage) -> Option<String> {
    let mut chain: Vec<&str> = message.references.iter().map(String::as_str).collect();
    if let Some(parent) = message.in_reply_to.as_deref()
        && !chain.contains(&parent)
    {
        chain.push(parent);
    }
    if chain.is_empty() {
        return None;
    }
    Some(chain.join(" "))
}

/// Addresses as one line, the way the list column shows them.
fn join_addresses(addresses: &[crate::common::types::EmailAddress]) -> String {
    addresses
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Store the folder list, and return the rows with the ids the cache gave them.
///
/// The path is stored as the server spells it, because that is what goes back
/// in a SELECT. The name is stored as the decoded path rather than the last
/// segment: the tree is one flat level, so two folders called "2026" under
/// different parents would be two rows reading the same and nothing to tell
/// them apart. "Archive/2026" says which one it is.
pub fn store_folders(
    cache: &MessageCache,
    account_id: &str,
    folders: &[ImapFolder],
) -> Result<Vec<(ImapFolder, i64)>> {
    let mut stored = Vec::with_capacity(folders.len());
    for folder in folders {
        cache.save_folder(&CachedFolder {
            id: 0,
            account_id: account_id.to_string(),
            name: folder.display_path.clone(),
            path: folder.path.clone(),
            folder_type: folder.folder_type.as_str().to_string(),
            unread_count: 0,
            total_count: 0,
        })?;
        // `save_folder` replaces on conflict, so its returned rowid is not the
        // one to keep. Reading it back gives the id the messages have to point
        // at.
        if let Some(row) = cache.get_folder(account_id, &folder.path)? {
            stored.push((folder.clone(), row.id));
        }
    }
    Ok(stored)
}

/// Sync one folder into the cache.
pub async fn sync_folder(
    controller: &MailController,
    cache: &MessageCache,
    folder: &ImapFolder,
    folder_id: i64,
    limit: usize,
) -> Result<FolderSync> {
    if !folder.selectable {
        // A container such as Gmail's `[Gmail]`. Selecting it fails.
        return Ok(FolderSync {
            folder: folder.name.clone(),
            ..Default::default()
        });
    }

    let status = controller.select_folder(&folder.path).await?;
    let renumbered = matches!(
        (status.uid_validity, cache.folder_uid_validity(folder_id)?),
        (Some(now), Some(before)) if now != before
    );
    if renumbered {
        // Every UID we hold now names a different message, or none.
        tracing::info!(
            "{} was renumbered by the server; re-reading it",
            folder.name
        );
        cache.forget_folder_messages(folder_id)?;
    }
    if let Some(validity) = status.uid_validity {
        cache.set_folder_uid_validity(folder_id, validity)?;
    }

    let on_server = controller.list_uids(&folder.path).await?;
    let stored = cache.stored_uids(folder_id)?;

    let forgotten = uids_to_forget(&on_server, &stored);
    for uid in &forgotten {
        cache.forget_message(folder_id, *uid)?;
    }

    let wanted = uids_to_fetch(&on_server, &stored, limit);
    let fetched = controller.fetch_headers(&folder.path, &wanted).await?;
    for message in &fetched {
        cache.upsert_message(&to_incoming(message, folder_id))?;
    }

    Ok(FolderSync {
        folder: folder.name.clone(),
        fetched: fetched.len(),
        forgotten: forgotten.len(),
        total_on_server: on_server.len(),
        renumbered,
    })
}

/// The folders worth syncing, and the order to do them in.
///
/// The inbox first, because it is what somebody pressing Check Mail is waiting
/// for; then the folders mail is filed into. Containers that hold no messages
/// are skipped.
pub fn folders_to_sync(folders: &[ImapFolder]) -> Vec<&ImapFolder> {
    let mut worth: Vec<&ImapFolder> = folders
        .iter()
        .filter(|folder| folder.selectable && folder.folder_type != FolderType::Spam)
        .collect();
    worth.sort_by_key(|folder| folder.folder_type.tree_order());
    worth
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::types::EmailAddress;

    fn message(uid: u32) -> ImapMessage {
        ImapMessage {
            uid,
            subject: "Notes on the engine".to_string(),
            from: vec![EmailAddress::new(
                "ada@example.com".to_string(),
                Some("Ada Lovelace".to_string()),
            )],
            to: vec![EmailAddress::new("me@example.com".to_string(), None)],
            date: Some("2026-07-20T10:00:00+00:00".to_string()),
            internal_date: Some("2026-07-20T10:00:05+00:00".to_string()),
            size: 2048,
            flags: vec!["\\Seen".to_string()],
            message_id: Some("note-1@example.com".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_a_first_look_at_a_small_folder_takes_all_of_it() {
        assert_eq!(uids_to_fetch(&[1, 2, 3], &[], 500), vec![1, 2, 3]);
    }

    #[test]
    fn test_a_first_look_at_a_large_folder_takes_the_newest() {
        // Where a reader starts. Taking the oldest instead would show them
        // mail from years ago and nothing from today.
        let on_server: Vec<u32> = (1..=10_000).collect();
        let wanted = uids_to_fetch(&on_server, &[], 3);
        assert_eq!(wanted, vec![9998, 9999, 10_000]);
    }

    #[test]
    fn test_messages_already_held_are_not_fetched_again() {
        assert_eq!(uids_to_fetch(&[1, 2, 3, 4], &[1, 3], 500), vec![2, 4]);
    }

    #[test]
    fn test_a_folder_that_has_not_changed_costs_no_fetch() {
        assert!(uids_to_fetch(&[1, 2, 3], &[1, 2, 3], 500).is_empty());
    }

    #[test]
    fn test_asking_for_nothing_fetches_nothing() {
        assert!(uids_to_fetch(&[1, 2, 3], &[], 0).is_empty());
    }

    #[test]
    fn test_an_unordered_server_list_still_yields_the_newest() {
        let wanted = uids_to_fetch(&[7, 2, 99, 41, 5], &[], 2);
        assert_eq!(wanted, vec![41, 99]);
    }

    #[test]
    fn test_a_message_deleted_elsewhere_is_forgotten() {
        // Otherwise the reader arrows onto a row, presses Enter, and gets an
        // error instead of a message.
        assert_eq!(uids_to_forget(&[1, 3], &[1, 2, 3]), vec![2]);
    }

    #[test]
    fn test_nothing_is_forgotten_when_the_server_still_has_it_all() {
        assert!(uids_to_forget(&[1, 2, 3], &[1, 2, 3]).is_empty());
    }

    #[test]
    fn test_an_empty_mailbox_forgets_everything_that_was_in_it() {
        assert_eq!(uids_to_forget(&[], &[1, 2]), vec![1, 2]);
    }

    #[test]
    fn test_a_message_becomes_the_row_the_list_shows() {
        let stored = to_incoming(&message(42), 7);
        assert_eq!(stored.folder_id, 7);
        assert_eq!(stored.uid, 42);
        assert_eq!(stored.subject, "Notes on the engine");
        assert_eq!(stored.from_addr, "Ada Lovelace <ada@example.com>");
        assert_eq!(stored.to_addr, "me@example.com");
        assert_eq!(stored.size_bytes, Some(2048));
        assert!(stored.read);
        assert!(!stored.starred);
    }

    #[test]
    fn test_a_message_with_no_date_header_falls_back_to_when_it_arrived() {
        // An empty date sorts the message to one end of the folder, where its
        // reader will not look for it.
        let mut without = message(1);
        without.date = None;
        let stored = to_incoming(&without, 1);
        assert_eq!(stored.date, "2026-07-20T10:00:05+00:00");
    }

    #[test]
    fn test_the_arrival_time_is_kept_as_well_as_the_senders_date() {
        // The Date header is written by the sender and is sometimes wrong.
        let stored = to_incoming(&message(1), 1);
        assert_eq!(stored.date, "2026-07-20T10:00:00+00:00");
        assert_eq!(
            stored.internal_date.as_deref(),
            Some("2026-07-20T10:00:05+00:00")
        );
    }

    #[test]
    fn test_no_recipients_in_copy_is_stored_as_nothing_rather_than_an_empty_line() {
        let stored = to_incoming(&message(1), 1);
        assert_eq!(stored.cc, None);
    }

    #[test]
    fn test_the_reference_chain_keeps_both_headers() {
        // Some senders write References, some write only In-Reply-To, and
        // threading needs whichever arrived.
        let mut reply = message(2);
        reply.references = vec!["first@example.com".to_string()];
        reply.in_reply_to = Some("second@example.com".to_string());
        let stored = to_incoming(&reply, 1);
        assert_eq!(
            stored.refs_header.as_deref(),
            Some("first@example.com second@example.com")
        );
    }

    #[test]
    fn test_a_parent_already_in_the_chain_is_not_repeated() {
        let mut reply = message(2);
        reply.references = vec!["first@example.com".to_string()];
        reply.in_reply_to = Some("first@example.com".to_string());
        let stored = to_incoming(&reply, 1);
        assert_eq!(stored.refs_header.as_deref(), Some("first@example.com"));
    }

    #[test]
    fn test_a_message_starting_a_conversation_has_no_chain() {
        assert_eq!(to_incoming(&message(1), 1).refs_header, None);
    }

    #[test]
    fn test_a_message_with_no_id_is_still_stored() {
        // It cannot be threaded, but it is real mail and hiding it would be
        // worse than showing it alone.
        let mut anonymous = message(1);
        anonymous.message_id = None;
        assert_eq!(to_incoming(&anonymous, 1).message_id, "");
    }

    fn folder(name: &str, folder_type: FolderType, selectable: bool) -> ImapFolder {
        ImapFolder {
            name: name.to_string(),
            display_path: name.to_string(),
            path: name.to_string(),
            delimiter: Some("/".to_string()),
            folder_type,
            selectable,
        }
    }

    #[test]
    fn test_the_inbox_is_synced_first() {
        // It is what somebody pressing Check Mail is waiting for.
        let folders = [
            folder("Archive", FolderType::Archive, true),
            folder("INBOX", FolderType::Inbox, true),
            folder("Sent", FolderType::Sent, true),
        ];
        let order: Vec<&str> = folders_to_sync(&folders)
            .iter()
            .map(|f| f.name.as_str())
            .collect();
        assert_eq!(order.first(), Some(&"INBOX"));
    }

    #[test]
    fn test_a_container_that_holds_no_mail_is_not_synced() {
        // Selecting Gmail's `[Gmail]` fails, and the failure would be reported
        // as a sync error for a folder that never had messages.
        let folders = [
            folder("[Gmail]", FolderType::Custom, false),
            folder("INBOX", FolderType::Inbox, true),
        ];
        let names: Vec<&str> = folders_to_sync(&folders)
            .iter()
            .map(|f| f.name.as_str())
            .collect();
        assert_eq!(names, vec!["INBOX"]);
    }

    #[test]
    fn test_junk_is_left_alone() {
        // Downloading a spam folder costs the whole of it and gives the reader
        // a mailbox of mail they did not ask for. They can open it when they
        // want it.
        let folders = [
            folder("INBOX", FolderType::Inbox, true),
            folder("Junk", FolderType::Spam, true),
        ];
        let names: Vec<&str> = folders_to_sync(&folders)
            .iter()
            .map(|f| f.name.as_str())
            .collect();
        assert_eq!(names, vec!["INBOX"]);
    }
}

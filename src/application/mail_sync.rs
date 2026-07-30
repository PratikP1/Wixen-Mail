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
use crate::service::protocols::imap::{
    ImapClient, ImapConfig, ImapFolder, ImapIdleEvent, ImapIdleHandle, ImapMessage,
};

/// Whether there is older mail still to fetch.
///
/// The pure half of "get older messages": the command is worth offering only
/// when it would do something, and when it would not, saying so is better than
/// a key that appears to be broken.
pub fn more_to_fetch(held: usize, total_on_server: usize) -> bool {
    held < total_on_server
}

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
    /// How many of them are unread, as the server counts.
    pub unread: usize,
    /// How many are now downloaded, out of `total_on_server`.
    ///
    /// The number worth saying on a large mailbox. "500 messages" after a sync
    /// of a forty thousand message inbox reads as a complete answer and is
    /// not one; "1,000 of 40,000" says there is more and that asking again
    /// will get it.
    pub held: usize,
    /// How many messages already held had their flags brought up to date.
    ///
    /// Read as well as counted, because it is the number that says whether
    /// state set on another device is arriving. Zero on every sync of a mailbox
    /// somebody also reads on a phone means it is not.
    pub flags_updated: usize,
    /// Whether the server had renumbered the mailbox since the last sync.
    pub renumbered: bool,
}

/// Which messages to fetch headers for, newest first, bounded.
///
/// Newest first because that is where a reader starts, and bounded because the
/// alternative on a large mailbox is a sync that never visibly finishes.
/// Returned in ascending order, which is how they go into a sequence set.
fn uids_to_fetch(on_server: &[u32], stored: &[u32], limit: usize) -> Vec<u32> {
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
fn uids_to_forget(on_server: &[u32], stored: &[u32]) -> Vec<u32> {
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
fn to_incoming(message: &ImapMessage, folder_id: i64, in_junk_folder: bool) -> IncomingMessage {
    IncomingMessage {
        folder_id,
        uid: message.uid,
        message_id: message.message_id.clone().unwrap_or_default(),
        subject: message.subject.clone(),
        from_addr: join_addresses(&message.from),
        to_addr: join_addresses(&message.to),
        cc: Some(join_addresses(&message.cc)).filter(|cc| !cc.is_empty()),
        reply_to: Some(join_addresses(&message.reply_to)).filter(|to| !to.is_empty()),
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
        answered: message.answered(),
        draft: message.draft(),
        deleted: message.deleted(),
        has_attachments: message.has_attachments,
        // Two sources, worst wins. The headers carry what a filter decided;
        // the folder carries what Gmail decided, which is all Gmail tells an
        // IMAP client.
        safety: message
            .safety
            .clone()
            .and(crate::service::safety::from_folder(in_junk_folder)),
        gmail_message_id: message.gmail_message_id,
        // Space separated, which is how IMAP writes a flag list and what the
        // labels already are on the wire. A label with a space in it was
        // quoted there and is not quoted here, so this is for showing and for
        // telling two rows apart, not for handing back to the server.
        labels: Some(message.labels.join(" ")).filter(|labels| !labels.is_empty()),
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
        let id = cache.save_folder(&CachedFolder {
            id: 0,
            account_id: account_id.to_string(),
            name: folder.display_path.clone(),
            path: folder.path.clone(),
            folder_type: folder.folder_type.as_str().to_string(),
            unread_count: 0,
            total_count: 0,
        })?;
        // The two facts only the server can answer, kept so the window that
        // asks which folders to sync shows the same default the sync uses.
        cache.set_folder_server_facts(id, folder.holds_all_mail, folder.subscribed)?;
        stored.push((folder.clone(), id));
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

    // Counts first, in one STATUS, before the mailbox is opened. STATUS is
    // discouraged on the mailbox that is currently selected, and asking here
    // also means the tree can be given numbers for folders that are never
    // opened at all.
    let counts = controller.folder_counts(&folder.path).await?;

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
        cache.upsert_message(&to_incoming(
            message,
            folder_id,
            folder.folder_type == crate::common::types::FolderType::Spam,
        ))?;
    }

    // Messages already held, whose flags may have changed elsewhere. The
    // header fetch above only asks about messages this cache does not have, so
    // without this a message read on a phone stays unread here for as long as
    // the account exists.
    let already_held: Vec<u32> = stored
        .iter()
        .copied()
        .filter(|uid| !wanted.contains(uid))
        .collect();
    let since = if renumbered {
        None
    } else {
        cache.folder_modseq(folder_id)?
    };
    let changed = if already_held.is_empty() && since.is_none() {
        Vec::new()
    } else {
        controller
            .fetch_flags(&folder.path, &already_held, since)
            .await?
    };
    for (uid, flags) in &changed {
        cache.set_message_flags(folder_id, *uid, flags)?;
    }
    if let Some(modseq) = status.highest_modseq {
        cache.set_folder_modseq(folder_id, modseq)?;
    }

    // Counts for the folder tree. The server's, not the cache's: only part of
    // a large folder is stored, so counting rows here would tell somebody
    // their inbox holds five hundred messages when it holds forty thousand.
    // Both numbers from the same STATUS, so the tree never reads "3 unread of
    // 2", which is what a pair taken from two commands a moment apart can say.
    let unread = counts.unread as usize;
    cache.set_folder_counts(folder_id, unread, counts.total as usize)?;

    Ok(FolderSync {
        folder: folder.name.clone(),
        fetched: fetched.len(),
        forgotten: forgotten.len(),
        total_on_server: on_server.len(),
        unread,
        flags_updated: changed.len(),
        // Counted after the write, so it includes what this round brought
        // down. Asking the cache rather than adding up, because a message
        // already held and re-fetched is not a new one.
        held: cache
            .stored_uids(folder_id)
            .map(|held| held.len())
            .unwrap_or(0),
        renumbered,
    })
}

/// Open a connection whose only job is to watch one folder for changes.
///
/// A second connection, not the one the rest of the client uses. IDLE takes a
/// connection over: no other command can run on it until the watch ends, so
/// watching the inbox on the working connection would mean nothing else could
/// happen while it watched.
pub async fn watch_folder(
    server: &str,
    port: u16,
    username: &str,
    auth: &crate::service::protocols::MailAuth,
    use_tls: bool,
    folder: &str,
) -> Result<(
    tokio::sync::mpsc::UnboundedReceiver<ImapIdleEvent>,
    ImapIdleHandle,
)> {
    let client = ImapClient::new(ImapConfig {
        server: server.to_string(),
        port,
        use_tls,
        username: username.to_string(),
    })?;
    let mut session = client.connect(auth).await?;
    session.select_folder(folder).await?;
    Ok(session.watch(folder.to_string()))
}

/// What somebody has chosen about each folder, by the server's own path.
///
/// A folder with no entry has never been asked about, and gets the default.
/// The distinction matters: "not chosen yet" and "chosen not to" look the same
/// as a `false` and mean opposite things when a new folder appears.
pub type FolderChoices = std::collections::HashMap<String, bool>;

/// What decides whether a folder is worth syncing.
///
/// A small type rather than four loose booleans in a signature, so a call site
/// cannot quietly swap "subscribed" for "selectable" and still compile. It is
/// also what the cache stores, so the window that asks somebody about a folder
/// and the sync that acts on the answer are working from the same facts rather
/// than each guessing from a name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FolderFacts {
    pub kind: FolderType,
    /// Whether it can hold messages at all.
    pub selectable: bool,
    /// Whether it holds a copy of every message in the account.
    pub holds_all_mail: bool,
    /// Whether the account is subscribed to it on the server.
    pub subscribed: bool,
}

impl From<&ImapFolder> for FolderFacts {
    fn from(folder: &ImapFolder) -> Self {
        Self {
            kind: folder.folder_type,
            selectable: folder.selectable,
            holds_all_mail: folder.holds_all_mail,
            subscribed: folder.subscribed,
        }
    }
}

/// Whether a folder should sync when nobody has said either way.
///
/// Four kinds are left out. Containers that hold no messages, because selecting
/// one fails. Junk, because downloading a spam folder costs the whole of it and
/// gives somebody a mailbox of mail they did not ask for. The mailbox that
/// holds a copy of everything, which is Gmail's All Mail, because every message
/// in it is also somewhere else, so syncing both downloads the account twice
/// and lists every message twice. And anything the account is not subscribed
/// to, which is how somebody has already said they do not want a folder.
///
/// Subscription only decides anything where the server keeps subscriptions at
/// all. A server that keeps none reports none, and reading that as "nothing is
/// wanted" would sync no folders and look like an account with no mail in it.
pub fn sync_by_default(facts: FolderFacts, server_keeps_subscriptions: bool) -> bool {
    facts.selectable
        && facts.kind != FolderType::Spam
        && !facts.holds_all_mail
        && (facts.subscribed || !server_keeps_subscriptions)
}

/// What the cache knows about each folder, beyond its name and role.
///
/// Keyed by the server's own path: whether it holds a copy of every message,
/// and whether the account is subscribed to it. Written by a sync from what the
/// server said, because nothing local can work either out.
pub type StoredFacts = std::collections::HashMap<String, (bool, bool)>;

/// Whether a folder the cache holds should be kept up to date.
///
/// The same rule as [`folders_to_sync`], from the same facts, for the two
/// places that have cache rows rather than a live folder list: the tree that
/// shows the folders, and the window that asks about them. Written once so the
/// three can never disagree about which folders exist, which would show a
/// folder in the tree that the sync then never fills.
pub fn cached_folder_syncs(
    folder: &CachedFolder,
    chosen: &FolderChoices,
    facts: &StoredFacts,
    keeps_subscriptions: bool,
) -> bool {
    if let Some(wanted) = chosen.get(&folder.path) {
        return *wanted;
    }
    let (holds_all_mail, subscribed) = facts.get(&folder.path).copied().unwrap_or((false, true));
    sync_by_default(
        FolderFacts {
            kind: FolderType::from_stored(&folder.folder_type),
            // Anything in the cache was listed as somewhere messages could be
            // stored against, so it is selectable.
            selectable: true,
            holds_all_mail,
            subscribed,
        },
        keeps_subscriptions,
    )
}

/// Whether the server keeps a subscription list at all, from stored facts.
pub fn keeps_subscriptions_stored(facts: &StoredFacts) -> bool {
    facts.values().any(|(_, subscribed)| *subscribed)
}

/// Whether the server keeps a subscription list at all.
///
/// Read from the folders it listed rather than from a capability, because there
/// is no capability for it: a server either answers LSUB with something or it
/// does not.
pub fn keeps_subscriptions(folders: &[ImapFolder]) -> bool {
    folders.iter().any(|folder| folder.subscribed)
}

/// The folders worth syncing, and the order to do them in.
///
/// The inbox first, because it is what somebody pressing Check Mail is waiting
/// for; then the folders mail is filed into.
///
/// A folder somebody has chosen for is obeyed, whatever the default would have
/// said, except that an unselectable container is never synced: selecting one
/// fails, so a tick beside it would be a promise nothing can keep.
pub fn folders_to_sync<'a>(
    folders: &'a [ImapFolder],
    chosen: &FolderChoices,
) -> Vec<&'a ImapFolder> {
    let keeps = keeps_subscriptions(folders);

    let mut worth: Vec<&ImapFolder> = folders
        .iter()
        .filter(|folder| folder.selectable)
        .filter(|folder| match chosen.get(&folder.path) {
            Some(wanted) => *wanted,
            None => sync_by_default(FolderFacts::from(*folder), keeps),
        })
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
    fn test_there_is_more_to_fetch_until_everything_is_held() {
        assert!(more_to_fetch(500, 40_000));
        assert!(more_to_fetch(39_999, 40_000));
        assert!(!more_to_fetch(40_000, 40_000));
    }

    #[test]
    fn test_an_empty_folder_has_nothing_older_to_ask_for() {
        // Offering the command here would be a key that does nothing, which
        // reads as a key that is broken.
        assert!(!more_to_fetch(0, 0));
    }

    #[test]
    fn test_holding_more_than_the_server_lists_is_not_more_to_fetch() {
        // Reachable between an expunge on the server and the next sync.
        assert!(!more_to_fetch(600, 500));
    }

    #[test]
    fn test_an_empty_mailbox_forgets_everything_that_was_in_it() {
        assert_eq!(uids_to_forget(&[], &[1, 2]), vec![1, 2]);
    }

    #[test]
    fn test_a_message_becomes_the_row_the_list_shows() {
        let stored = to_incoming(&message(42), 7, false);
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
        let stored = to_incoming(&without, 1, false);
        assert_eq!(stored.date, "2026-07-20T10:00:05+00:00");
    }

    #[test]
    fn test_the_arrival_time_is_kept_as_well_as_the_senders_date() {
        // The Date header is written by the sender and is sometimes wrong.
        let stored = to_incoming(&message(1), 1, false);
        assert_eq!(stored.date, "2026-07-20T10:00:00+00:00");
        assert_eq!(
            stored.internal_date.as_deref(),
            Some("2026-07-20T10:00:05+00:00")
        );
    }

    #[test]
    fn test_no_recipients_in_copy_is_stored_as_nothing_rather_than_an_empty_line() {
        let stored = to_incoming(&message(1), 1, false);
        assert_eq!(stored.cc, None);
    }

    #[test]
    fn test_the_reference_chain_keeps_both_headers() {
        // Some senders write References, some write only In-Reply-To, and
        // threading needs whichever arrived.
        let mut reply = message(2);
        reply.references = vec!["first@example.com".to_string()];
        reply.in_reply_to = Some("second@example.com".to_string());
        let stored = to_incoming(&reply, 1, false);
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
        let stored = to_incoming(&reply, 1, false);
        assert_eq!(stored.refs_header.as_deref(), Some("first@example.com"));
    }

    #[test]
    fn test_a_message_starting_a_conversation_has_no_chain() {
        assert_eq!(to_incoming(&message(1), 1, false).refs_header, None);
    }

    #[test]
    fn test_a_message_with_no_id_is_still_stored() {
        // It cannot be threaded, but it is real mail and hiding it would be
        // worse than showing it alone.
        let mut anonymous = message(1);
        anonymous.message_id = None;
        assert_eq!(to_incoming(&anonymous, 1, false).message_id, "");
    }

    fn folder(name: &str, folder_type: FolderType, selectable: bool) -> ImapFolder {
        ImapFolder {
            name: name.to_string(),
            display_path: name.to_string(),
            path: name.to_string(),
            folder_type,
            selectable,
            holds_all_mail: false,
            subscribed: true,
        }
    }

    #[test]
    fn test_the_folder_holding_every_message_is_left_alone() {
        // Gmail's All Mail. Every message in the inbox is also in here under a
        // different UID, so syncing both downloads the account twice and shows
        // every message twice: once as Inbox, once as All Mail.
        let mut all_mail = folder("[Gmail]/All Mail", FolderType::Archive, true);
        all_mail.holds_all_mail = true;
        let folders = [folder("INBOX", FolderType::Inbox, true), all_mail];

        let names: Vec<&str> = folders_to_sync(&folders, &FolderChoices::new())
            .iter()
            .map(|f| f.name.as_str())
            .collect();

        assert_eq!(names, vec!["INBOX"]);
    }

    #[test]
    fn test_an_ordinary_archive_is_still_synced() {
        // Only the mailbox that holds a copy of everything is skipped. A real
        // archive holds mail that is nowhere else, and skipping it would lose
        // it from the listing entirely.
        let folders = [
            folder("INBOX", FolderType::Inbox, true),
            folder("Archive", FolderType::Archive, true),
        ];

        let names: Vec<&str> = folders_to_sync(&folders, &FolderChoices::new())
            .iter()
            .map(|f| f.name.as_str())
            .collect();

        assert_eq!(names, vec!["INBOX", "Archive"]);
    }

    #[test]
    fn test_a_folder_nobody_is_subscribed_to_is_left_alone() {
        // Shared servers list mailboxes by the hundred and people subscribe to
        // a handful. Syncing all of them is somebody else's mail, downloaded
        // slowly, in a folder tree they cannot navigate.
        let mut unsubscribed = folder("Old backups 2009", FolderType::Custom, true);
        unsubscribed.subscribed = false;
        let folders = [folder("INBOX", FolderType::Inbox, true), unsubscribed];

        let names: Vec<&str> = folders_to_sync(&folders, &FolderChoices::new())
            .iter()
            .map(|f| f.name.as_str())
            .collect();

        assert_eq!(names, vec!["INBOX"]);
    }

    #[test]
    fn test_the_inbox_is_synced_even_when_nothing_is_subscribed() {
        // A server that keeps no subscription list reports none, and then
        // subscription cannot be what decides anything. Reading it as "nothing
        // is subscribed" would sync no folders at all and look like an account
        // with no mail in it.
        let folders = [
            folder_unsubscribed("INBOX", FolderType::Inbox),
            folder_unsubscribed("Work", FolderType::Custom),
        ];

        let names: Vec<&str> = folders_to_sync(&folders, &FolderChoices::new())
            .iter()
            .map(|f| f.name.as_str())
            .collect();

        assert_eq!(names, vec!["INBOX", "Work"]);
    }

    fn cached(path: &str, folder_type: FolderType) -> CachedFolder {
        CachedFolder {
            id: 0,
            account_id: "acc".to_string(),
            name: path.to_string(),
            path: path.to_string(),
            folder_type: folder_type.as_str().to_string(),
            unread_count: 0,
            total_count: 0,
        }
    }

    #[test]
    fn test_the_tree_and_the_sync_agree_about_a_stored_folder() {
        // Three places read this: the sync, the tree that lists the folders,
        // and the window that asks about them. A folder shown in the tree that
        // the sync never fills is a folder somebody opens and finds empty.
        let facts = StoredFacts::from([
            ("INBOX".to_string(), (false, true)),
            ("[Gmail]/All Mail".to_string(), (true, true)),
            ("Old backups".to_string(), (false, false)),
        ]);
        let nothing_chosen = FolderChoices::new();

        assert!(cached_folder_syncs(
            &cached("INBOX", FolderType::Inbox),
            &nothing_chosen,
            &facts,
            true
        ));
        assert!(
            !cached_folder_syncs(
                &cached("[Gmail]/All Mail", FolderType::Archive),
                &nothing_chosen,
                &facts,
                true
            ),
            "the folder holding everything"
        );
        assert!(
            !cached_folder_syncs(
                &cached("Old backups", FolderType::Custom),
                &nothing_chosen,
                &facts,
                true
            ),
            "not subscribed"
        );
    }

    #[test]
    fn test_a_stored_folder_somebody_answered_for_obeys_them() {
        let facts = StoredFacts::from([("[Gmail]/All Mail".to_string(), (true, true))]);
        let asked_for = FolderChoices::from([("[Gmail]/All Mail".to_string(), true)]);

        assert!(cached_folder_syncs(
            &cached("[Gmail]/All Mail", FolderType::Archive),
            &asked_for,
            &facts,
            true
        ));
    }

    #[test]
    fn test_a_folder_the_cache_knows_nothing_about_is_treated_as_ordinary() {
        // What every folder looks like in a database written before the facts
        // were stored. Reading it as unsubscribed would empty the folder tree
        // the moment this shipped.
        let empty = StoredFacts::new();

        assert!(cached_folder_syncs(
            &cached("Work", FolderType::Custom),
            &FolderChoices::new(),
            &empty,
            keeps_subscriptions_stored(&empty)
        ));
    }

    #[test]
    fn test_a_folder_somebody_asked_for_is_synced_whatever_the_default_says() {
        // Somebody who wants All Mail, or their junk folder, gets it. The
        // default is a starting point, not a rule they cannot change.
        let mut all_mail = folder("[Gmail]/All Mail", FolderType::Archive, true);
        all_mail.holds_all_mail = true;
        let folders = [
            folder("INBOX", FolderType::Inbox, true),
            all_mail,
            folder("Junk", FolderType::Spam, true),
        ];
        let chosen = FolderChoices::from([
            ("[Gmail]/All Mail".to_string(), true),
            ("Junk".to_string(), true),
        ]);

        let names: Vec<&str> = folders_to_sync(&folders, &chosen)
            .iter()
            .map(|f| f.name.as_str())
            .collect();

        assert!(names.contains(&"[Gmail]/All Mail"), "{names:?}");
        assert!(names.contains(&"Junk"), "{names:?}");
    }

    #[test]
    fn test_a_folder_somebody_turned_off_stays_off() {
        let folders = [
            folder("INBOX", FolderType::Inbox, true),
            folder("Work", FolderType::Custom, true),
        ];
        let chosen = FolderChoices::from([("Work".to_string(), false)]);

        let names: Vec<&str> = folders_to_sync(&folders, &chosen)
            .iter()
            .map(|f| f.name.as_str())
            .collect();

        assert_eq!(names, vec!["INBOX"]);
    }

    #[test]
    fn test_a_container_stays_unsynced_even_if_it_was_ticked() {
        // Selecting one fails, so a tick beside it is a promise nothing can
        // keep, and the failure would be reported as a sync error for a folder
        // that never had messages in it.
        let folders = [folder("[Gmail]", FolderType::Custom, false)];
        let chosen = FolderChoices::from([("[Gmail]".to_string(), true)]);

        assert!(folders_to_sync(&folders, &chosen).is_empty());
    }

    fn folder_unsubscribed(name: &str, folder_type: FolderType) -> ImapFolder {
        let mut folder = folder(name, folder_type, true);
        folder.subscribed = false;
        folder
    }

    #[test]
    fn test_the_inbox_is_synced_first() {
        // It is what somebody pressing Check Mail is waiting for.
        let folders = [
            folder("Archive", FolderType::Archive, true),
            folder("INBOX", FolderType::Inbox, true),
            folder("Sent", FolderType::Sent, true),
        ];
        let order: Vec<&str> = folders_to_sync(&folders, &FolderChoices::new())
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
        let names: Vec<&str> = folders_to_sync(&folders, &FolderChoices::new())
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
        let names: Vec<&str> = folders_to_sync(&folders, &FolderChoices::new())
            .iter()
            .map(|f| f.name.as_str())
            .collect();
        assert_eq!(names, vec!["INBOX"]);
    }
}

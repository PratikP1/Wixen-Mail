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
use crate::data::message_cache::{CachedFolder, CachedMessage, IncomingMessage, MessageCache};
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
    ///
    /// Counted from the rows that really changed, not from the size of the
    /// server's reply. The reply covers messages this cache does not hold and
    /// messages whose headers this same sync brought down, and on a server
    /// that cannot say what changed it covers everything held whether it
    /// changed or not.
    ///
    /// This is read, starred, answered, draft and deleted. A label put on or
    /// taken off elsewhere travels in the same reply and is stored somewhere
    /// else, so it does not appear in this number.
    pub flags_updated: usize,
    /// Whether the server had renumbered the mailbox since the last sync.
    pub renumbered: bool,
    /// What the rules did to the mail that just arrived.
    pub filtered: Filtered,
}

/// The rules to run on arriving mail, and what may be done as a result.
pub struct Filtering<'a> {
    pub rules: &'a crate::application::filters::FilterEngine,
    /// What this account is allowed to change. Moving and deleting reach the
    /// server; marking read and flagging do not, and go out later through the
    /// flag sync, which has its own gate.
    pub allowed: crate::application::allowed::Allowed,
}

/// What the rules did, and what they were not allowed to do.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filtered {
    /// How many arriving messages a rule touched.
    pub changed: usize,
    /// How many were left alone because moving or deleting is not allowed.
    ///
    /// Counted rather than passed over quietly. A rule that files invoices
    /// into a folder and does not, on a build where writing to the server is
    /// off, is a rule somebody believes is working.
    pub held_back: usize,
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

/// Which held messages still need their flags read back.
///
/// Everything the cache holds, less the ones whose headers this sync just
/// fetched, whose flags came with them, and less the ones it just forgot,
/// which are not on the server to be asked about.
fn still_to_check(stored: &[u32], fetched: &[u32], forgotten: &[u32]) -> Vec<u32> {
    let skip: std::collections::HashSet<u32> =
        fetched.iter().chain(forgotten.iter()).copied().collect();
    stored
        .iter()
        .copied()
        .filter(|uid| !skip.contains(uid))
        .collect()
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
        receipt_to: message.receipt_to.clone(),
        // IMAP has UIDs of its own; this is the POP identifier and there is none.
        pop_uidl: None,
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
///
/// A mailbox named the same as a folder that lives on this computer is left
/// out. Both kinds share a table and are told apart by the path, so storing one
/// would hand the server the row holding mail it has never seen: a sent copy, a
/// draft, or the whole of a mailbox read over POP. The next check for mail
/// would ask the server which of those messages it still has, be told none of
/// them, and delete every one of them and its stored body. Nothing legitimate
/// is lost by refusing, because the prefix uses a character a mailbox name does
/// not carry.
pub fn store_folders(
    cache: &MessageCache,
    account_id: &str,
    folders: &[ImapFolder],
) -> Result<Vec<(ImapFolder, i64)>> {
    let mut stored = Vec::with_capacity(folders.len());
    for folder in folders {
        if crate::application::local_folders::is_local(&folder.path) {
            tracing::warn!(
                "The server listed a mailbox named like a folder kept on this computer, so it was left out of the folder list"
            );
            continue;
        }
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
/// What a sync needs from a mail server.
///
/// Named for what it is rather than for the protocol behind it. The sync does
/// not care whether the answers come over IMAP, and saying so in the type is
/// what lets the whole of `sync_folder` be tested: everything below this line
/// is decisions about what to fetch, forget and reconcile, and none of it had
/// ever been run in a test because running it meant having a server.
///
/// Five methods, which is the whole of what a sync asks. Anything larger would
/// be a description of the IMAP client rather than of what this needs.
pub(crate) trait Mailbox {
    /// How many messages a folder holds and how many are unread.
    ///
    /// Asked before the folder is opened, because the answer is wanted for
    /// folders that are never opened at all.
    async fn folder_counts(
        &self,
        folder: &str,
    ) -> Result<crate::service::protocols::imap::FolderCounts>;

    /// Open a folder, and say what state the server reports for it.
    async fn select_folder(
        &self,
        folder: &str,
    ) -> Result<crate::service::protocols::imap::MailboxStatus>;

    /// Every message the folder holds, by uid.
    ///
    /// The whole folder rather than a page: it is what says which stored
    /// messages the server no longer has, and a page would report every
    /// message outside it as deleted.
    async fn list_uids(&self, folder: &str) -> Result<Vec<u32>>;

    /// The headers of the named messages.
    async fn fetch_headers(&self, folder: &str, uids: &[u32]) -> Result<Vec<ImapMessage>>;

    /// The flags of messages already held, for the ones that have changed.
    ///
    /// `changed_since` is the modification sequence a CONDSTORE server uses to
    /// answer in one round trip. `None` asks about every uid given.
    async fn fetch_flags(
        &self,
        folder: &str,
        held: &[u32],
        changed_since: Option<u64>,
    ) -> Result<Vec<(u32, Vec<String>)>>;
}

impl Mailbox for MailController {
    async fn folder_counts(
        &self,
        folder: &str,
    ) -> Result<crate::service::protocols::imap::FolderCounts> {
        MailController::folder_counts(self, folder).await
    }

    async fn select_folder(
        &self,
        folder: &str,
    ) -> Result<crate::service::protocols::imap::MailboxStatus> {
        MailController::select_folder(self, folder).await
    }

    async fn list_uids(&self, folder: &str) -> Result<Vec<u32>> {
        MailController::list_uids(self, folder).await
    }

    async fn fetch_headers(&self, folder: &str, uids: &[u32]) -> Result<Vec<ImapMessage>> {
        MailController::fetch_headers(self, folder, uids).await
    }

    async fn fetch_flags(
        &self,
        folder: &str,
        held: &[u32],
        changed_since: Option<u64>,
    ) -> Result<Vec<(u32, Vec<String>)>> {
        MailController::fetch_flags(self, folder, held, changed_since).await
    }
}

/// Run the rules over messages that have just arrived.
///
/// The rules could be written, named, ordered and stored, and nothing had ever
/// evaluated one: the engine, the editor and the table were all there and no
/// arriving message was ever passed to them.
///
/// One message at a time and one failure at a time. A rule that cannot be
/// carried out on one message is not a reason to stop filtering the rest, and
/// the alternative, stopping the whole sync, would turn a bad rule into a
/// mailbox that no longer updates.
fn apply_rules(cache: &MessageCache, filtering: &Filtering<'_>, arrived: &[i64]) -> Filtered {
    let mut done = Filtered::default();
    for id in arrived {
        let Ok(Some(message)) = cache.get_message(*id) else {
            continue;
        };
        let outcome =
            crate::application::filters::settle(&filtering.rules.evaluate_message(&message));
        if outcome.is_nothing() {
            continue;
        }
        if outcome.touches_the_server() && !filtering.allowed.mail {
            // Not done quietly. A rule that files invoices into a folder and
            // does not is a rule somebody believes is working.
            tracing::info!(
                "A rule would move or delete a message and changing mail is not allowed; \
                 leaving it where it is"
            );
            done.held_back += 1;
            continue;
        }
        match carry_out(cache, &message, &outcome) {
            Ok(()) => done.changed += 1,
            Err(e) => tracing::warn!("A rule could not be carried out: {}", e),
        }
    }
    done
}

/// Do to one message what its rules settled on.
fn carry_out(
    cache: &MessageCache,
    message: &CachedMessage,
    outcome: &crate::application::filters::Outcome,
) -> Result<()> {
    let id = message.id;
    if outcome.delete {
        // Locally. Taking it off the server is the move-to-trash path, which
        // is somebody's own deliberate action rather than a rule's.
        cache.delete_message(id)?;
        return Ok(());
    }
    if outcome.read.is_some() || outcome.starred.is_some() {
        cache.update_message_flags(
            id,
            outcome.read.unwrap_or(message.read),
            outcome.starred.unwrap_or(message.starred),
        )?;
    }
    for tag in &outcome.tags {
        cache.add_tag_to_message(id, tag)?;
    }
    if let Some(folder) = &outcome.move_to {
        // Named rather than done. Moving needs the folder's id and a write to
        // the server, and doing half of it, in the cache only, would show
        // somebody a message in a folder it is not in until the next sync put
        // it back.
        tracing::info!(
            "A rule would move a message to {}, which is not built yet; it is left where it is",
            folder
        );
    }
    Ok(())
}

pub(crate) async fn sync_folder<M: Mailbox>(
    controller: &M,
    cache: &MessageCache,
    folder: &ImapFolder,
    folder_id: i64,
    limit: usize,
    filtering: Option<&Filtering<'_>>,
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
    let mut arrived = Vec::new();
    for message in &fetched {
        let id = cache.upsert_message(&to_incoming(
            message,
            folder_id,
            folder.folder_type == crate::common::types::FolderType::Spam,
        ))?;
        arrived.push(id);
    }

    // Rules, on what has just arrived and nothing else. Running them over
    // messages already held would apply them again on every sync, and a rule
    // somebody has since changed their mind about would keep firing on mail
    // they had already sorted by hand.
    let filtered = match filtering {
        Some(rules) => apply_rules(cache, rules, &arrived),
        None => Filtered::default(),
    };

    // Messages already held, whose flags may have changed elsewhere. The
    // header fetch above only asks about messages this cache does not have, so
    // without this a message read on a phone stays unread here for as long as
    // the account exists.
    //
    // The ones just fetched are left out, since their flags arrived with them,
    // and so are the ones just forgotten, which are not on the server to ask
    // about. Through sets rather than by scanning two lists: a folder of forty
    // thousand against a page of five hundred is twenty million comparisons
    // done for nothing.
    let already_held: Vec<u32> = still_to_check(&stored, &wanted, &forgotten);
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
    // Counted from what the cache actually rewrote rather than from the length
    // of the reply. A server that can answer "what changed since" answers for
    // the whole mailbox, so the reply names messages this cache does not hold
    // and messages whose headers arrived a moment ago in this same sync, and a
    // server that cannot answers about every message it was asked about
    // whether anything changed or not.
    let mut brought_up_to_date = 0usize;
    for (uid, flags) in &changed {
        brought_up_to_date += cache.set_message_flags(folder_id, *uid, flags)?;
        // The keywords among those flags are labels, put on elsewhere or taken
        // off elsewhere. Without this a label set on a phone never arrived and
        // one removed there stayed here for as long as the account existed,
        // which is the same gap the flag sync itself was written to close.
        if let Ok(Some(account)) = cache.folder_account(folder_id)
            && let Ok(Some(row)) = cache.message_row_for_uid(folder_id, *uid)
            && let Err(e) = cache.match_labels_to_keywords(row, &account, flags)
        {
            // Said rather than swallowed: labels quietly not arriving looks
            // exactly like nobody having set any.
            tracing::warn!("The labels on message {row} could not be brought up to date: {e}");
        }
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
        flags_updated: brought_up_to_date,
        // Counted after the write, so it includes what this round brought
        // down. Asking the cache rather than adding up, because a message
        // already held and re-fetched is not a new one.
        held: cache
            .stored_uids(folder_id)
            .map(|held| held.len())
            .unwrap_or(0),
        renumbered,
        filtered,
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
    // A folder on this computer is always there to be opened. Every rule below
    // is about what to download from a server, and this folder has no server
    // behind it: the junk rule in particular was keeping a POP account's own
    // Junk folder out of its tree, where a filter could file into it and
    // nobody could reach it.
    if crate::application::local_folders::is_local(&folder.path) {
        return true;
    }
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

    /// A rule written by somebody, on a message that has just arrived.
    ///
    /// The whole point of the task this comes from: the engine, the rule
    /// editor and the table all existed and no arriving message had ever been
    /// passed to them. Every step here, because the step that was missing was
    /// the one joining two halves that each worked.
    #[test]
    fn test_a_rule_reaches_a_message_that_has_just_arrived() {
        use crate::application::filters::FilterEngine;
        use crate::data::message_cache::MessageFilterRule;

        let dir = std::env::temp_dir().join(format!("wixen_filter_{}", uuid::Uuid::new_v4()));
        let cache = MessageCache::new(dir, None).expect("a cache");
        let folder_id = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acct".into(),
                name: "Inbox".into(),
                path: "INBOX".into(),
                folder_type: "Inbox".into(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder");
        let id = cache
            .upsert_message(&IncomingMessage {
                folder_id,
                uid: 1,
                message_id: "inv-1@example.com".into(),
                subject: "Invoice #4021".into(),
                from_addr: "billing@example.com".into(),
                to_addr: "me@example.com".into(),
                cc: None,
                reply_to: None,
                date: "2026-07-31T09:00:00+00:00".into(),
                internal_date: None,
                size_bytes: None,
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
            .expect("a message");

        let mut engine = FilterEngine::default();
        engine.load_from_persisted(&[MessageFilterRule {
            id: "r1".into(),
            account_id: "acct".into(),
            name: "Invoices".into(),
            field: "subject".into(),
            match_type: "contains".into(),
            pattern: "Invoice".into(),
            case_sensitive: false,
            action_type: "mark_as_read".into(),
            action_value: None,
            enabled: true,
            created_at: "2026-07-31T00:00:00Z".into(),
        }]);

        let done = apply_rules(
            &cache,
            &Filtering {
                rules: &engine,
                allowed: crate::application::allowed::Allowed::EVERYTHING,
            },
            &[id],
        );

        assert_eq!(done.changed, 1, "the rule did not reach the message");
        assert!(
            cache
                .get_message(id)
                .expect("read back")
                .expect("there")
                .read,
            "the message is still unread"
        );
    }

    #[test]
    fn test_a_rule_that_only_marks_read_still_runs_on_an_account_that_may_not_be_changed() {
        // Only the rules that reach the server are held back. Marking read is
        // written here and goes out later through the flag sync, which has its
        // own gate, so holding it back too would mean nothing is ever filed on
        // an account in the shipping default.
        use crate::application::filters::FilterEngine;
        use crate::data::message_cache::MessageFilterRule;

        let dir = std::env::temp_dir().join(format!("wixen_filter_{}", uuid::Uuid::new_v4()));
        let cache = MessageCache::new(dir, None).expect("a cache");
        let folder_id = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acct".into(),
                name: "Inbox".into(),
                path: "INBOX".into(),
                folder_type: "Inbox".into(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder");
        let id = cache
            .upsert_message(&IncomingMessage {
                folder_id,
                uid: 3,
                message_id: "inv-3@example.com".into(),
                subject: "Invoice #4023".into(),
                from_addr: "billing@example.com".into(),
                to_addr: "me@example.com".into(),
                cc: None,
                reply_to: None,
                date: "2026-07-31T09:00:00+00:00".into(),
                internal_date: None,
                size_bytes: None,
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
            .expect("a message");

        let mut engine = FilterEngine::default();
        engine.load_from_persisted(&[MessageFilterRule {
            id: "r3".into(),
            account_id: "acct".into(),
            name: "Invoices".into(),
            field: "subject".into(),
            match_type: "contains".into(),
            pattern: "Invoice".into(),
            case_sensitive: false,
            action_type: "mark_as_read".into(),
            action_value: None,
            enabled: true,
            created_at: "2026-07-31T00:00:00Z".into(),
        }]);

        let done = apply_rules(
            &cache,
            &Filtering {
                rules: &engine,
                allowed: crate::application::allowed::Allowed::NOTHING,
            },
            &[id],
        );

        assert_eq!(done.changed, 1, "the rule was held back for nothing");
        assert_eq!(done.held_back, 0);
        assert!(
            cache
                .get_message(id)
                .expect("read back")
                .expect("there")
                .read,
            "the message is still unread"
        );
    }

    #[test]
    fn test_a_rule_that_moves_is_held_back_when_mail_may_not_be_changed() {
        // And counted, so it can be said. A rule that files invoices into a
        // folder and does not is a rule somebody believes is working.
        use crate::application::filters::FilterEngine;
        use crate::data::message_cache::MessageFilterRule;

        let dir = std::env::temp_dir().join(format!("wixen_filter_{}", uuid::Uuid::new_v4()));
        let cache = MessageCache::new(dir, None).expect("a cache");
        let folder_id = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acct".into(),
                name: "Inbox".into(),
                path: "INBOX".into(),
                folder_type: "Inbox".into(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder");
        let id = cache
            .upsert_message(&IncomingMessage {
                folder_id,
                uid: 2,
                message_id: "inv-2@example.com".into(),
                subject: "Invoice #4022".into(),
                from_addr: "billing@example.com".into(),
                to_addr: "me@example.com".into(),
                cc: None,
                reply_to: None,
                date: "2026-07-31T09:00:00+00:00".into(),
                internal_date: None,
                size_bytes: None,
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
            .expect("a message");

        let mut engine = FilterEngine::default();
        engine.load_from_persisted(&[MessageFilterRule {
            id: "r2".into(),
            account_id: "acct".into(),
            name: "File invoices".into(),
            field: "subject".into(),
            match_type: "contains".into(),
            pattern: "Invoice".into(),
            case_sensitive: false,
            action_type: "delete".into(),
            action_value: None,
            enabled: true,
            created_at: "2026-07-31T00:00:00Z".into(),
        }]);

        let done = apply_rules(
            &cache,
            &Filtering {
                rules: &engine,
                allowed: crate::application::allowed::Allowed::NOTHING,
            },
            &[id],
        );

        assert_eq!(done.held_back, 1);
        assert_eq!(done.changed, 0);
        assert!(
            cache.get_message(id).expect("read back").is_some(),
            "the message was deleted anyway"
        );
    }

    /// A mail server that answers from a script rather than a socket.
    ///
    /// The whole of `sync_folder` was untestable before the transport had a
    /// name: what to fetch, what to forget, whose flags to ask about and what
    /// to do with what comes back are all decisions, and none of them had ever
    /// been run in a test because running them meant having a server.
    struct Scripted {
        on_server: Vec<u32>,
        headers: Vec<ImapMessage>,
        flags: Vec<(u32, Vec<String>)>,
        counts: crate::service::protocols::imap::FolderCounts,
        uid_validity: Option<u32>,
        /// The mailbox's modification sequence, which is what a server that
        /// can answer "what changed since" reports. `None` is a server that
        /// cannot, and the two take different paths through the flag fetch.
        highest_modseq: Option<u64>,
        /// Which uids the header fetch was asked for, so a test can check that
        /// a sync asked for the right ones rather than only that it ended up
        /// with the right rows.
        asked_for: std::cell::RefCell<Vec<u32>>,
    }

    impl Default for Scripted {
        fn default() -> Self {
            Self {
                on_server: Vec::new(),
                headers: Vec::new(),
                flags: Vec::new(),
                counts: crate::service::protocols::imap::FolderCounts {
                    total: 0,
                    unread: 0,
                },
                uid_validity: Some(1),
                highest_modseq: None,
                asked_for: std::cell::RefCell::new(Vec::new()),
            }
        }
    }

    impl Mailbox for Scripted {
        async fn folder_counts(
            &self,
            _folder: &str,
        ) -> Result<crate::service::protocols::imap::FolderCounts> {
            Ok(self.counts)
        }

        async fn select_folder(
            &self,
            _folder: &str,
        ) -> Result<crate::service::protocols::imap::MailboxStatus> {
            Ok(crate::service::protocols::imap::MailboxStatus {
                uid_validity: self.uid_validity,
                highest_modseq: self.highest_modseq,
            })
        }

        async fn list_uids(&self, _folder: &str) -> Result<Vec<u32>> {
            Ok(self.on_server.clone())
        }

        async fn fetch_headers(&self, _folder: &str, uids: &[u32]) -> Result<Vec<ImapMessage>> {
            self.asked_for.borrow_mut().extend_from_slice(uids);
            Ok(self
                .headers
                .iter()
                .filter(|m| uids.contains(&m.uid))
                .cloned()
                .collect())
        }

        async fn fetch_flags(
            &self,
            _folder: &str,
            held: &[u32],
            since: Option<u64>,
        ) -> Result<Vec<(u32, Vec<String>)>> {
            // Asked "what changed since", a server answers for the whole
            // mailbox and ignores the list of messages this cache holds. So
            // the answer names messages that were never downloaded here, and
            // messages whose headers this same sync has just brought down.
            if since.is_some() {
                return Ok(self.flags.clone());
            }
            Ok(self
                .flags
                .iter()
                .filter(|(uid, _)| held.contains(uid))
                .cloned()
                .collect())
        }
    }

    fn a_cache() -> (MessageCache, i64, ImapFolder) {
        let dir = std::env::temp_dir().join(format!("wixen_sync_{}", uuid::Uuid::new_v4()));
        let cache = MessageCache::new(dir, None).expect("a cache");
        let folder_id = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acct".into(),
                name: "Inbox".into(),
                path: "INBOX".into(),
                folder_type: "Inbox".into(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder");
        let folder = ImapFolder {
            name: "Inbox".into(),
            display_path: "INBOX".into(),
            path: "INBOX".into(),
            folder_type: FolderType::Inbox,
            selectable: true,
            holds_all_mail: false,
            subscribed: true,
        };
        (cache, folder_id, folder)
    }

    fn run<M: Mailbox>(
        server: &M,
        cache: &MessageCache,
        id: i64,
        folder: &ImapFolder,
    ) -> FolderSync {
        run_limited(server, cache, id, folder, INITIAL_FETCH_LIMIT)
    }

    /// The same sync with a smaller first look, so a test can put a message on
    /// the server that this round will not download.
    fn run_limited<M: Mailbox>(
        server: &M,
        cache: &MessageCache,
        id: i64,
        folder: &ImapFolder,
        limit: usize,
    ) -> FolderSync {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
            .block_on(sync_folder(server, cache, folder, id, limit, None))
            .expect("the sync runs")
    }

    #[test]
    fn test_the_folder_list_is_stored_with_the_facts_only_the_server_knows() {
        // Storing the list is not just writing names. It hands back the id the
        // cache gave each row, which is what every later sync of that folder is
        // keyed on, and it keeps the two answers only the server has: whether a
        // folder holds every message, and whether anybody subscribed to it.
        // Losing either turns the window that asks which folders to sync into a
        // window offering a different default from the one the sync uses.
        let dir = std::env::temp_dir().join(format!("wixen_folders_{}", uuid::Uuid::new_v4()));
        let cache = MessageCache::new(dir, None).expect("a cache");
        let everything = ImapFolder {
            holds_all_mail: true,
            subscribed: false,
            ..folder("All Mail", FolderType::Archive, true)
        };
        let folders = vec![folder("INBOX", FolderType::Inbox, true), everything];

        let stored = store_folders(&cache, "acct", &folders).expect("the list is stored");

        assert_eq!(stored.len(), 2);
        assert!(
            stored.iter().all(|(_, id)| *id != 0),
            "a folder came back without the id the cache gave it"
        );

        let facts = cache.folder_server_facts("acct").expect("the facts");
        assert_eq!(facts.get("INBOX").copied(), Some((false, true)));
        assert_eq!(facts.get("All Mail").copied(), Some((true, false)));

        let rows = cache.get_folders_for_account("acct").expect("the rows");
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn test_mail_that_lives_only_here_survives_a_server_claiming_its_folder() {
        // The folders on this computer share a table with the server's and are
        // told apart by the one thing they have: a path under a reserved
        // prefix. Nothing else marks them, so if the server hands back a
        // mailbox with that same path, the row is taken over: the sync gets the
        // identifier of the folder on this computer, asks the server which
        // messages it holds, is told none of them, and deletes every one of
        // them and its stored body. For a sent copy, a draft, or a whole POP
        // mailbox, that is the only copy there was.
        let dir = std::env::temp_dir().join(format!("wixen_reserved_{}", uuid::Uuid::new_v4()));
        let cache = MessageCache::new(dir, None).expect("a cache");
        let here =
            crate::application::local_folders::local_sent(crate::common::types::Protocol::Pop3)
                .expect("a folder on this computer");
        let mine = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acct".into(),
                name: "Sent".into(),
                path: here.clone(),
                folder_type: "Sent".into(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("the folder on this computer");
        let message = cache
            .upsert_message(&IncomingMessage {
                folder_id: mine,
                uid: 9001,
                message_id: "sent-1@example.com".into(),
                subject: "What went out".into(),
                from_addr: "me@example.com".into(),
                to_addr: "them@example.com".into(),
                cc: None,
                reply_to: None,
                date: "2026-08-01T09:00:00+00:00".into(),
                internal_date: None,
                size_bytes: None,
                refs_header: None,
                read: true,
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
            .expect("the only copy");
        cache
            .save_message_body(message, Some("what went out"), None)
            .expect("its body");

        // A server that lists a mailbox named the same as the reserved path,
        // which nothing stops it doing, and then reports it as empty.
        let listed = vec![folder(&here, FolderType::Sent, true)];
        let stored = store_folders(&cache, "acct", &listed).expect("the list is stored");

        // The same steps a check for mail runs, in the same order.
        for wanted in folders_to_sync(&listed, &FolderChoices::new()) {
            let Some((_, id)) = stored.iter().find(|(f, _)| f.path == wanted.path) else {
                continue;
            };
            run(&Scripted::default(), &cache, *id, wanted);
        }

        assert!(
            cache
                .stored_uids(mine)
                .expect("the folder is readable")
                .contains(&9001),
            "the message was deleted because the server did not list it"
        );
        assert!(
            cache
                .get_message_body(message)
                .expect("the body is readable")
                .is_some(),
            "the stored body went with it"
        );
        assert_eq!(
            cache
                .get_folder("acct", &here)
                .expect("the folder is readable")
                .expect("the folder is still there")
                .name,
            "Sent",
            "the server renamed a folder that is not its own"
        );
    }

    #[test]
    fn test_a_mailbox_with_no_connection_reports_the_failure_rather_than_an_empty_folder() {
        // Every one of these five, because the sync believes what it is told.
        // An empty uid list is read as "the server no longer has any of these"
        // and deletes the whole cached folder; a made up flag list is read as
        // "none of these are read" and marks a mailbox unread; a made up header
        // writes a message with no subject into somebody's inbox. A failure
        // reported as a failure does none of it.
        let controller = MailController::new();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime");

        let answered: Vec<&str> = runtime.block_on(async {
            // Through the trait rather than the inherent methods of the same
            // name, since it is the adapter between the two being tested. All
            // five asked before anything is said, so one run names every one
            // that made an answer up rather than stopping at the first.
            let asked = [
                (
                    "how many the folder holds",
                    Mailbox::folder_counts(&controller, "INBOX").await.is_err(),
                ),
                (
                    "what state the folder is in",
                    Mailbox::select_folder(&controller, "INBOX").await.is_err(),
                ),
                (
                    "which messages the folder holds",
                    Mailbox::list_uids(&controller, "INBOX").await.is_err(),
                ),
                (
                    "the headers of a message",
                    Mailbox::fetch_headers(&controller, "INBOX", &[1])
                        .await
                        .is_err(),
                ),
                (
                    "the flags of a message",
                    Mailbox::fetch_flags(&controller, "INBOX", &[1], None)
                        .await
                        .is_err(),
                ),
            ];
            asked
                .into_iter()
                .filter(|(_, failed)| !failed)
                .map(|(what, _)| what)
                .collect()
        });

        assert!(
            answered.is_empty(),
            "answered without a connection: {answered:?}"
        );
    }

    #[test]
    fn test_mail_synced_out_of_the_junk_folder_is_marked_as_spam() {
        // What Gmail decided, which is all Gmail tells an IMAP client. Read
        // aloud it is the difference between a warning and none.
        let (cache, id, mut folder) = a_cache();
        folder.folder_type = FolderType::Spam;

        let done = run(
            &Scripted {
                on_server: vec![1],
                headers: vec![message(1)],
                ..Default::default()
            },
            &cache,
            id,
            &folder,
        );

        assert_eq!(done.fetched, 1);
        assert_eq!(
            verdict_for(&cache, id, 1),
            crate::service::safety::Safety::Spam
        );
    }

    #[test]
    fn test_mail_synced_out_of_an_ordinary_folder_is_not_marked_as_spam() {
        // The other half. Announcing every message in the inbox as spam is the
        // same defect wearing the opposite sign, and it is the one that makes
        // the warning worth nothing.
        let (cache, id, folder) = a_cache();

        run(
            &Scripted {
                on_server: vec![1],
                headers: vec![message(1)],
                ..Default::default()
            },
            &cache,
            id,
            &folder,
        );

        assert_eq!(
            verdict_for(&cache, id, 1),
            crate::service::safety::Safety::Ordinary
        );
    }

    /// How a stored message was judged, read back out of the cache.
    fn verdict_for(
        cache: &MessageCache,
        folder_id: i64,
        uid: u32,
    ) -> crate::service::safety::Safety {
        let row = cache
            .message_row_for_uid(folder_id, uid)
            .expect("read back")
            .expect("the row");
        cache.message_safety(row).expect("a verdict").level
    }

    #[test]
    fn test_a_sync_brings_down_what_the_server_has() {
        let (cache, id, folder) = a_cache();
        let server = Scripted {
            on_server: vec![1, 2],
            headers: vec![message(1), message(2)],
            counts: crate::service::protocols::imap::FolderCounts {
                total: 2,
                unread: 1,
            },
            ..Default::default()
        };

        let done = run(&server, &cache, id, &folder);

        assert_eq!(done.fetched, 2);
        assert_eq!(done.held, 2);
        assert_eq!(done.total_on_server, 2);
        assert_eq!(done.unread, 1);
        assert_eq!(*server.asked_for.borrow(), vec![1, 2]);
    }

    #[test]
    fn test_a_second_sync_asks_only_about_what_it_does_not_have() {
        // Asking again for everything is how a large mailbox becomes a sync
        // that never visibly finishes.
        let (cache, id, folder) = a_cache();
        run(
            &Scripted {
                on_server: vec![1],
                headers: vec![message(1)],
                ..Default::default()
            },
            &cache,
            id,
            &folder,
        );

        let second = Scripted {
            on_server: vec![1, 2],
            headers: vec![message(1), message(2)],
            ..Default::default()
        };
        let done = run(&second, &cache, id, &folder);

        assert_eq!(done.fetched, 1, "it fetched something it already had");
        assert_eq!(*second.asked_for.borrow(), vec![2]);
    }

    #[test]
    fn test_a_message_the_server_no_longer_has_leaves_the_list() {
        // A row that is gone from the server but still listed is worse than a
        // list a little behind: somebody arrows onto it, presses Enter, and
        // gets an error instead of mail.
        let (cache, id, folder) = a_cache();
        run(
            &Scripted {
                on_server: vec![1, 2],
                headers: vec![message(1), message(2)],
                ..Default::default()
            },
            &cache,
            id,
            &folder,
        );

        let done = run(
            &Scripted {
                on_server: vec![2],
                headers: vec![message(2)],
                ..Default::default()
            },
            &cache,
            id,
            &folder,
        );

        assert_eq!(done.forgotten, 1);
        assert_eq!(cache.stored_uids(id).expect("held").len(), 1);
    }

    #[test]
    fn test_a_flag_set_on_another_device_arrives_on_the_next_sync() {
        let (cache, id, folder) = a_cache();
        run(
            &Scripted {
                on_server: vec![1],
                headers: vec![ImapMessage {
                    flags: Vec::new(),
                    ..message(1)
                }],
                ..Default::default()
            },
            &cache,
            id,
            &folder,
        );

        let done = run(
            &Scripted {
                on_server: vec![1],
                headers: vec![message(1)],
                flags: vec![(
                    1,
                    vec![crate::service::protocols::imap::flag::SEEN.to_string()],
                )],
                ..Default::default()
            },
            &cache,
            id,
            &folder,
        );

        assert_eq!(done.flags_updated, 1);
    }

    #[test]
    fn test_only_messages_this_cache_holds_count_as_changed_elsewhere() {
        // The number is read out after every sync as the count of messages
        // whose state was set somewhere else. A server that can answer "what
        // changed since" answers for the whole mailbox, so the reply names
        // messages that were never downloaded here and messages this same sync
        // has only just brought down. Counting those says that mail changed on
        // a phone when none of it did.
        let (cache, id, folder) = a_cache();
        let seen = vec![crate::service::protocols::imap::flag::SEEN.to_string()];
        run(
            &Scripted {
                on_server: vec![1],
                headers: vec![ImapMessage {
                    flags: Vec::new(),
                    ..message(1)
                }],
                highest_modseq: Some(10),
                ..Default::default()
            },
            &cache,
            id,
            &folder,
        );

        let done = run_limited(
            &Scripted {
                on_server: vec![1, 2, 77],
                // Only the newest missing one is downloaded this round, so 2
                // stays a message the server has and this cache does not.
                headers: vec![message(77)],
                flags: vec![(1, seen.clone()), (2, seen.clone()), (77, seen.clone())],
                highest_modseq: Some(11),
                ..Default::default()
            },
            &cache,
            id,
            &folder,
            1,
        );

        assert_eq!(
            done.flags_updated, 1,
            "only the message that was held and really changed counts"
        );
    }

    #[test]
    fn test_a_renumbered_mailbox_is_read_again_from_scratch() {
        // Every uid held names a different message now, or none. Keeping them
        // would show somebody the wrong mail under the right subject.
        let (cache, id, folder) = a_cache();
        run(
            &Scripted {
                on_server: vec![1],
                headers: vec![message(1)],
                uid_validity: Some(1),
                ..Default::default()
            },
            &cache,
            id,
            &folder,
        );

        // The same uid, meaning a different message. This is the case the
        // forget exists for: uid 1 is still on the server, so nothing else
        // would notice it had changed, and without re-reading the folder
        // somebody would see the old subject on the new mail for ever.
        let after = Scripted {
            on_server: vec![1],
            headers: vec![ImapMessage {
                subject: "A different message entirely".into(),
                ..message(1)
            }],
            uid_validity: Some(2),
            ..Default::default()
        };
        let done = run(&after, &cache, id, &folder);

        assert!(done.renumbered);
        assert_eq!(
            *after.asked_for.borrow(),
            vec![1],
            "the folder was not read again after being renumbered"
        );
        assert_eq!(done.fetched, 1);
    }

    #[test]
    fn test_a_folder_that_cannot_be_opened_is_passed_over_rather_than_failing() {
        // Gmail's `[Gmail]` is a container. Selecting it fails, and one such
        // folder must not end the sync of the account it sits in.
        let (cache, id, mut folder) = a_cache();
        folder.selectable = false;

        let done = run(&Scripted::default(), &cache, id, &folder);

        assert_eq!(done.fetched, 0);
        assert_eq!(done.folder, "Inbox");
    }

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
            flags: vec![crate::service::protocols::imap::flag::SEEN.to_string()],
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

    #[test]
    fn test_a_message_with_no_cc_has_no_cc_rather_than_an_empty_one() {
        // Mutation testing found this: nothing noticed when the emptiness test
        // was inverted, so the column could have been filled with "" for every
        // message with no Cc and read as a recipient nobody can see.
        let plain = message(1);

        let row = to_incoming(&plain, 1, false);

        assert_eq!(row.cc, None, "no Cc");
        assert_eq!(row.reply_to, None, "no Reply-To");
        assert_eq!(row.labels, None, "no labels off Gmail");
    }

    #[test]
    fn test_the_cc_and_reply_to_that_are_there_are_kept() {
        // The other half. A test for the empty case alone passes against a
        // function that drops the field always.
        let mut copied = message(1);
        copied.cc = vec![EmailAddress::new("bob@example.com".to_string(), None)];
        copied.reply_to = vec![EmailAddress::new("list@example.com".to_string(), None)];
        copied.labels = vec!["Work".to_string(), "Urgent".to_string()];

        let row = to_incoming(&copied, 1, false);

        assert_eq!(row.cc.as_deref(), Some("bob@example.com"));
        assert_eq!(row.reply_to.as_deref(), Some("list@example.com"));
        assert_eq!(row.labels.as_deref(), Some("Work Urgent"));
    }

    #[test]
    fn test_a_server_that_keeps_no_subscriptions_is_told_apart_from_one_that_does() {
        // The answer decides whether an unsubscribed folder syncs. Reading a
        // server with no subscription list as "nothing is wanted" would sync
        // no folders at all and look like an account with no mail in it.
        let keeps = StoredFacts::from([
            ("INBOX".to_string(), (false, true)),
            ("Old".to_string(), (false, false)),
        ]);
        let keeps_none = StoredFacts::from([
            ("INBOX".to_string(), (false, false)),
            ("Old".to_string(), (false, false)),
        ]);

        assert!(keeps_subscriptions_stored(&keeps));
        assert!(!keeps_subscriptions_stored(&keeps_none));
        assert!(!keeps_subscriptions_stored(&StoredFacts::new()));
    }

    #[test]
    fn test_nothing_subscribed_anywhere_still_syncs_every_folder() {
        // What the answer above is for. Both folders read as unsubscribed, so
        // subscription cannot be what somebody meant by it.
        let keeps_none = StoredFacts::from([("Work".to_string(), (false, false))]);

        assert!(cached_folder_syncs(
            &cached("Work", FolderType::Custom),
            &FolderChoices::new(),
            &keeps_none,
            keeps_subscriptions_stored(&keeps_none)
        ));
    }

    #[test]
    fn test_a_folder_on_this_computer_is_always_in_the_tree() {
        // The junk rule is about not downloading a server's spam folder, which
        // costs the whole of it. A folder on this computer has nothing to
        // download, and a POP account's Junk folder has existed in the database
        // since local folders shipped and has never once been reachable: a
        // filter could file into it and nobody could open it.
        let junk = cached(
            &crate::application::local_folders::LocalFolder {
                kind: FolderType::Spam,
                name: "Junk",
            }
            .path(),
            FolderType::Spam,
        );

        assert!(cached_folder_syncs(
            &junk,
            &FolderChoices::new(),
            &StoredFacts::new(),
            false
        ));
    }

    #[test]
    fn test_a_spam_folder_on_a_server_is_still_left_alone() {
        // The other half, and the reason the rule exists: downloading a
        // server's junk folder costs the whole of it and hands somebody a
        // mailbox of mail they never asked for.
        assert!(!cached_folder_syncs(
            &cached("[Gmail]/Spam", FolderType::Spam),
            &FolderChoices::new(),
            &StoredFacts::new(),
            false
        ));
    }

    #[test]
    fn test_a_message_just_fetched_is_not_asked_about_again() {
        // Its flags arrived with its headers a moment ago.
        assert_eq!(still_to_check(&[1, 2, 3], &[2, 3], &[]), vec![1]);
    }

    #[test]
    fn test_a_message_just_forgotten_is_not_asked_about() {
        // It is not on the server, so asking is a longer command for nothing.
        assert_eq!(still_to_check(&[1, 2, 3], &[], &[3]), vec![1, 2]);
    }

    #[test]
    fn test_everything_held_and_untouched_is_asked_about() {
        // The whole point: these are the messages whose read state can have
        // changed on somebody's phone since the last sync.
        assert_eq!(still_to_check(&[1, 2, 3], &[], &[]), vec![1, 2, 3]);
    }

    #[test]
    fn test_a_first_sync_has_nothing_to_ask_about() {
        assert!(still_to_check(&[], &[1, 2], &[]).is_empty());
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

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
use crate::application::summing_up::SummingUp;
use crate::common::{Error, Result, types::FolderType};
use crate::data::message_cache::{
    CachedFolder, CachedMessage, IncomingMessage, MessageCache, WhatTheServerSaid,
};
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
    /// How many messages were thrown away because of that renumbering.
    ///
    /// Two values rather than one, because they answer different questions. A
    /// folder can be renumbered with nothing held for it, on the sync after
    /// somebody added the account and before anything was downloaded, and that
    /// is a renumbering that cost nobody anything. Saying "0 messages" is the
    /// honest answer there and it needs both fields to be sayable at all.
    ///
    /// From what [`crate::data::message_cache::MessageCache::forget_folder_messages`]
    /// answers, which is the number of rows it really deleted, rather than
    /// from the size of anything counted afterwards. What is counted afterwards
    /// is the folder as it stands after the same sync has begun refilling it.
    pub discarded_after_renumbering: usize,
    /// What the rules did to the mail that just arrived.
    pub filtered: Filtered,
}

/// What one folder's sync did, in the words the status line uses.
///
/// Named here rather than built where it is shown, for the reason the contacts
/// and calendar summaries were: a sentence assembled at the call site cannot be
/// argued about in a test, and this one was assembled inside a closure on a
/// background thread where nothing could reach it.
///
/// Every part of it is a count, so they are all items in one list. It says how
/// many are held rather than how many arrived, because "500 messages" after a
/// sync of a forty thousand message inbox reads as a complete answer and is
/// not one.
pub fn what_the_folder_sync_did(result: &FolderSync) -> String {
    let mut said = SummingUp::opening(format!(
        "{}: {} of {} downloaded",
        result.folder,
        result.held,
        crate::service::caldav::how_many(result.total_on_server, "message")
    ));
    if more_to_fetch(result.held, result.total_on_server) {
        said.count("Shift+F9 for older");
    }
    if result.flags_updated > 0 {
        // What changed on another device. Worth saying because rows quietly
        // turning read is otherwise unexplained, and because a mailbox somebody
        // also reads on a phone that never reports any is one where this is
        // broken.
        said.count(format!("{} changed elsewhere", result.flags_updated));
    }
    if result.forgotten > 0 {
        said.count(format!("{} removed elsewhere", result.forgotten));
    }
    if result.renumbered {
        // Messages that went away and a mailbox the server renumbered are both
        // things the reader will notice as rows disappearing. Saying so turns
        // that from unexplained into expected.
        said.count("read again after the server renumbered it");
    }
    say_what_the_rules_did(&result.filtered, &mut said);
    said.spoken()
}

/// What to say when the server renumbered a folder, or nothing when it did not.
///
/// Its own sentence rather than a clause in the folder's summary, because it
/// is a different kind of fact. The summary says what a sync did; this says
/// that mail was deleted from this computer, which is the one thing in a sync
/// somebody would want to be told about afterwards. It goes out on its own
/// announcement topic for the same reason, and the arm in `wx_app` that sends
/// it says so beside the call.
///
/// No protocol word in it. The server calls this UIDVALIDITY and nobody
/// outside this codebase has any reason to know that; what a person needs is
/// which folder, that the numbering changed, and how much it cost them.
///
/// Worded so the count is a noun phrase at the end and no verb has to agree
/// with it. "1 message have been discarded" is what the obvious ordering
/// produces, and a second wording for the singular is the thing
/// [`crate::service::caldav::how_many`] exists to avoid.
pub fn what_the_renumbering_discarded(result: &FolderSync) -> Option<String> {
    if !result.renumbered {
        return None;
    }
    Some(format!(
        "The mail server gave {} new numbers, so what this computer held for it no longer \
         matches. It has been discarded and is being read again: {}.",
        result.folder,
        crate::service::caldav::how_many(result.discarded_after_renumbering, "message")
    ))
}

/// Add what the rules did to a summary, in one place.
///
/// Here rather than in each summary that needs it, because both kinds of
/// account run the same rules and a second wording would let one of them go
/// quiet without anything saying so. A POP check files mail on this computer
/// and an IMAP sync files it at the server, and what happened to the reader's
/// rules is the same sentence either way.
pub(crate) fn say_what_the_rules_did(filtered: &Filtered, said: &mut SummingUp) {
    if filtered.changed > 0 {
        said.count(format!("{} sorted by your rules", filtered.changed));
    }
    if filtered.held_back > 0 {
        // Said, not passed over. A rule that files invoices into a folder and
        // does not is a rule somebody believes is working, and the reason is a
        // setting they can change.
        said.count(format!(
            "{} left alone because changing mail is not allowed",
            filtered.held_back
        ));
    }
    if filtered.could_not_be_filed.is_empty() {
        return;
    }
    // Said, not passed over. These sentences were built, handed back and
    // read by nothing: not shown, not counted, not even logged. A rule
    // that files invoices and does not is a rule somebody believes is
    // working, and this is the only thing that says otherwise.
    // "Not filed as asked" rather than "could not be filed", because two
    // of the five ways this goes wrong did put the message in the folder
    // and left a copy behind as well. A clause the sentence after it has
    // to correct is a clause that was not worth saying.
    said.count(format!(
        "{} not filed as asked",
        crate::service::caldav::how_many(filtered.could_not_be_filed.len(), "message")
    ));
    let reasons = the_different_reasons(&filtered.could_not_be_filed);
    for reason in reasons.iter().take(REASONS_SAID_ALOUD) {
        said.sentence(*reason);
    }
    // Counted rather than dropped. Every one of them is in the log, and
    // somebody told two of four knows there are two more to look for.
    match reasons.len().saturating_sub(REASONS_SAID_ALOUD) {
        0 => {}
        1 => said.sentence("1 other reason is in the log"),
        more => said.sentence(format!("{more} other reasons are in the log")),
    }
}

/// How many different reasons a folder sync reads out.
///
/// A status line is heard from the first word to the last, so every extra
/// sentence is one more somebody listens through before the next thing that
/// happened. Two is enough to tell one thing going wrong from several
/// different things going wrong, and the rest are in the log.
const REASONS_SAID_ALOUD: usize = 2;

/// The reasons filing failed, each said once, in the order they happened.
///
/// One rule naming a folder somebody has since renamed fails on every message
/// it matches, so a hundred messages produce a hundred copies of one sentence.
/// Said a hundred times that is the flood this project's rule about bounded
/// feedback exists to stop, and the repeats say nothing the first copy did
/// not: the count in front of them already says how many messages there were.
///
/// A list rather than a set, because the order somebody hears them in should
/// be the order they happened in, and because the list is short by the time it
/// gets here.
fn the_different_reasons(sentences: &[String]) -> Vec<&str> {
    let mut different: Vec<&str> = Vec::new();
    for sentence in sentences {
        if !different.contains(&sentence.as_str()) {
            different.push(sentence);
        }
    }
    different
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
    /// How many arriving messages had everything their rules asked for done.
    ///
    /// Messages, counted once each, and only when the whole of what the rules
    /// asked for really happened. Both halves of the work used to add to this:
    /// [`apply_rules`], which writes flags and tags here, and
    /// [`carry_out_the_moves`], which reaches the server. So one message that a
    /// rule marked read and another rule filed was counted twice, and a message
    /// whose move the server then refused was counted anyway.
    ///
    /// A message with a move still to do is left for the mover to count, which
    /// is the only place that knows whether it happened.
    pub changed: usize,
    /// How many were left alone because moving or deleting is not allowed.
    ///
    /// Counted rather than passed over quietly. A rule that files invoices
    /// into a folder and does not, on a build where writing to the server is
    /// off, is a rule somebody believes is working.
    pub held_back: usize,
    /// Mail a rule meant to file that could not be filed, one sentence each.
    ///
    /// Said as well as logged. A rule that files invoices and does not is a
    /// rule somebody believes is working, and this project's own rule is that
    /// a warning nobody gets is not a warning. Sentences rather than a count,
    /// because which folder and why are the useful part.
    ///
    /// One per message, and the summary folds the repeats: the same rule fails
    /// the same way on every message it matches, and a hundred copies of one
    /// sentence is the flood the rule about bounded feedback exists to stop.
    pub could_not_be_filed: Vec<String>,
    /// Messages a rule says belong in another folder.
    ///
    /// Named here and carried out by the sync, which is the half with a
    /// server. `apply_rules` runs with a cache and no connection, and a move
    /// done in the cache alone would show a message in a folder it is not in
    /// until the next sync put it back.
    pub to_move: Vec<Moving>,
}

/// One message a rule says belongs somewhere else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Moving {
    /// The row here, so the cache can be brought into line afterwards.
    pub message_row: i64,
    /// What the server calls it in the folder it is in now.
    pub uid: u32,
    /// The folder it is going to, as the rule names it.
    pub into: String,
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

/// What the server answered when it was asked which messages a folder holds,
/// and how much of the folder that answer covers.
///
/// The two are one value because they are never safely apart. Plan 03-07 exists
/// to stop `sync_folder` listing every uid on every sync, and the listing it
/// will narrow is the same listing the forget path compares against. A page
/// compared against what this computer holds says that every message outside the
/// page has gone from the server, and the first sync after such a change would
/// then delete somebody's whole cached mailbox.
///
/// That constraint was a sentence in the doc comment below, where nothing could
/// hold anybody to it. Here it is a type: the forget path is handed this rather
/// than a bare list, so a caller with only part of a folder cannot reach the
/// deletion without writing the false claim down on the line where a reader can
/// see it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerListing {
    /// Every uid the folder holds, because the server was asked about all of it
    /// and answered.
    TheWholeMailbox(Vec<u32>),
    /// Some of them. A page, a window, or whatever a narrower question returned.
    PartOfIt(Vec<u32>),
}

impl ServerListing {
    /// The uids themselves, for the readers that do not care what they cover.
    ///
    /// Fetching headers and asking after flags are both safe against a page:
    /// asking about fewer messages than exist is a smaller sync, not a deletion.
    /// Only the forget path is held to the whole folder, so only the forget path
    /// is given the type instead of this.
    pub fn uids(&self) -> &[u32] {
        match self {
            Self::TheWholeMailbox(uids) | Self::PartOfIt(uids) => uids,
        }
    }
}

/// What this computer already knows about a folder it has synced before.
///
/// Three facts, read together because the decision below needs all three and
/// any one of them missing is a folder that cannot be resumed. Gathered into a
/// value rather than passed as three arguments so a caller cannot put the two
/// numbers the wrong way round.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct WhatThisComputerHolds {
    /// The server's numbering as at the last sync. `None` on a folder never
    /// synced.
    pub uid_validity: Option<u32>,
    /// The mailbox's modification sequence as at the last sync. `None` on a
    /// server without CONDSTORE, and on a folder never synced.
    pub modseq: Option<u64>,
    /// The highest uid held, which is where a resumed sync starts asking.
    /// `None` on an empty folder, which is nothing to resume from.
    pub highest_uid: Option<u32>,
}

/// What the server said about the folder when this sync opened it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct WhatTheServerReports {
    pub uid_validity: Option<u32>,
    /// The mailbox's highest modification sequence, from the SELECT.
    pub highest_modseq: Option<u64>,
}

/// What a sync has to ask the server before it can bring a folder up to date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhatToAskFor {
    /// Every uid the folder holds. The first sync of a folder, every sync of
    /// one that cannot be resumed, and every sync where the deletion
    /// comparison is due.
    EveryUid,
    /// Only the uids above the one given, which is the highest already held.
    TheUidsAbove(u32),
}

/// Whether this sync can resume the folder or has to read it out in full.
///
/// SCALE-01, and the whole of it. The state this reads has been stored and
/// read back since before this function existed; what was missing was anything
/// that acted on it, so every sync asked a forty thousand message mailbox to
/// list forty thousand numbers to find the three that arrived.
///
/// Four things have to hold before a folder can be resumed, and each one is a
/// different way of not knowing enough:
///
/// - a stored UIDVALIDITY, and the server reporting the same one. Without both,
///   the uids held either mean nothing or cannot be shown to mean anything, and
///   a resume from a number under one numbering into another is a resume from
///   the wrong place.
/// - a stored HIGHESTMODSEQ, which is what the flag read resumes from. A folder
///   with none was last synced against a server that could not answer "what
///   changed since", so there is nothing to resume.
/// - the server reporting a HIGHESTMODSEQ now. Read from the mailbox rather
///   than from the connection's capability list, because it is this number the
///   resume actually uses and a server can advertise CONDSTORE and report none
///   for a particular mailbox. Gmail reports neither: it has never advertised
///   CONDSTORE, which is asserted at `imap/abilities.rs:112`, so a Gmail
///   account reads out in full whatever else changes here.
/// - a highest uid held, which is where the narrow question starts. A folder
///   holding nothing has nowhere to resume from and is cheap to list anyway.
///
/// Its own function rather than a condition inside [`sync_folder`], following
/// [`uids_to_fetch`] and [`listing_contradicts_the_count`] above: a decision
/// with four inputs and two answers is testable without a server only if it
/// can be called without one.
pub(crate) const fn what_to_ask_for(
    held: WhatThisComputerHolds,
    reported: WhatTheServerReports,
) -> WhatToAskFor {
    let (Some(stored), Some(now)) = (held.uid_validity, reported.uid_validity) else {
        return WhatToAskFor::EveryUid;
    };
    if stored != now {
        return WhatToAskFor::EveryUid;
    }
    match held.highest_uid {
        Some(highest) => WhatToAskFor::TheUidsAbove(highest),
        None => WhatToAskFor::EveryUid,
    }
}

/// Which stored messages the server no longer has.
///
/// Deleted from another client, or expunged. Leaving them listed means a reader
/// arrows onto a row, presses Enter, and gets an error rather than a message.
///
/// The server list must be the whole mailbox, not the page just fetched.
/// Comparing against a page would delete everything outside it. That was the
/// whole of the rule and it was written here in prose. [`ServerListing`] is what
/// enforces it now, and this paragraph is the reason rather than the rule.
fn uids_to_forget(on_server: &ServerListing, stored: &[u32]) -> Vec<u32> {
    // A listing of part of a folder says nothing about the messages outside
    // it. Every uid held and not named is one this listing never asked about,
    // not one the server has lost, and the empty page is the case that would
    // take a whole mailbox: nothing named, everything held, so everything
    // held reads as gone.
    let ServerListing::TheWholeMailbox(present) = on_server else {
        return Vec::new();
    };
    let present: std::collections::HashSet<u32> = present.iter().copied().collect();
    let mut gone: Vec<u32> = stored
        .iter()
        .copied()
        .filter(|uid| !present.contains(uid))
        .collect();
    gone.sort_unstable();
    gone
}

/// Whether the two answers about one mailbox disagree with each other.
///
/// Two commands ask about the same mailbox in every sync, and only one of them
/// used to be believed. Counting it reads the server's answer properly and
/// fails when the server refuses; listing what is in it goes over a library
/// whose stream ends at the server's answer without reading it, so a refusal
/// arrives as an empty list. The read side is fixed, and this is the belt: a
/// server that says a mailbox holds messages and then lists none of them is
/// contradicting itself, and nothing here is sure enough of that to delete
/// somebody's mail on the strength of it.
///
/// The cost is one round of tidying. A mailbox genuinely emptied by another
/// client between the two commands reads as a disagreement, so the rows it left
/// behind are cleaned up on the next sync instead of this one. That is a
/// delayed tidy against wiping a mailbox that was never empty.
///
/// Takes the listing rather than its length, because only a listing that claims
/// the whole mailbox can contradict a count of it. A resumed sync asks about the
/// uids above the highest one held and is answered with none on every folder
/// nothing has arrived in, which is most folders on most syncs: read as a
/// contradiction that would refuse the ordinary sync of a quiet mailbox.
fn listing_contradicts_the_count(listed: &ServerListing, counted: u32) -> bool {
    matches!(listed, ServerListing::TheWholeMailbox(uids) if uids.is_empty()) && counted > 0
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
    crate::application::threading::as_stored(&message.references, message.in_reply_to.as_deref())
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
/// in a SELECT. The name is stored as the leaf, which is what somebody reads
/// off one row of the tree.
///
/// It used to be the whole decoded path, and the reason was sound while it
/// held: the tree was one flat level, so two folders called "2026" under
/// different parents were two rows reading the same with nothing to tell them
/// apart, and "Archive/2026" said which was which. What tells them apart now
/// is where they sit. The second pass below records that, so the hierarchy is
/// carried by the tree rather than spelled into a label, which is also the
/// rule for a screen reader: level and position are the tree control's to
/// announce and never the label's to repeat.
///
/// # The parent is worked out in a second pass, and has to be
///
/// A server may list a child before its parent, so the parent's id does not
/// exist yet when the child is stored. The pass below runs once every folder
/// in the account has one, over the rows this call just made and nothing else,
/// so a parent is only ever found inside the same account. The alternative, a
/// lookup by path across the table, would hang one account's mail under
/// another account's branch wherever two accounts spell a folder the same,
/// which is most of the time.
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
            name: folder.name.clone(),
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

    // Every folder in the account has an id now, so the paths can become a
    // hierarchy. This is the one place a path is split; everything downstream
    // reads a parent. A folder with no parent is written as having none rather
    // than left alone, because a server can move a folder to the top level and
    // a row that kept its old parent would go on showing it in a branch it has
    // left.
    for (folder, id) in &stored {
        let parent = the_folder_above(folder).and_then(|above| {
            stored
                .iter()
                .find(|(other, _)| other.path == above)
                .map(|(_, above_id)| *above_id)
        });
        cache.set_folder_parent(*id, parent)?;
    }
    Ok(stored)
}

/// The stored folders a fresh LIST no longer holds.
///
/// D-27's first half. A folder the server has stopped listing is not removed
/// without asking, so this only says which ones they are; marking and asking
/// are somebody else's job and deleting is nobody's until the answer arrives.
///
/// A comparison over two lists and nothing else. It takes no cache handle on
/// purpose: every rule below is a rule about two lists, and each one is worth a
/// test that needs no database and no server.
///
/// # An empty answer is a failed sync, never a mass deletion
///
/// On the wire, a server that lists nothing and a server whose mailboxes have
/// all been deleted send the same thing. Only one of those is survivable, so an
/// empty `listed` reports nothing at all. The other reading offers to delete
/// somebody's whole mailbox because their connection dropped between the
/// greeting and the LIST.
///
/// # Folders the server was never told about are never reported
///
/// Two kinds, filtered for the same reason [`store_folders`] refuses to store
/// them: a folder kept on this computer, and a folder owned by the reserved
/// this-computer account. The server has never seen either, so its silence
/// about them says nothing. Without this a server that answered with one
/// mailbox would appear to have deleted the user's Drafts and Outbox.
pub fn folders_the_server_no_longer_lists(stored: &[CachedFolder], listed: &[String]) -> Vec<i64> {
    // Before the comparison, not inside it. An answer holding nothing is a
    // sync that failed, and the difference between saying so here and reading
    // it as a deletion is the difference between somebody seeing a warning and
    // somebody being asked whether to delete their whole mailbox.
    if listed.is_empty() {
        return Vec::new();
    }

    stored
        .iter()
        .filter(|folder| !crate::application::local_folders::is_local(&folder.path))
        .filter(|folder| !crate::application::local_folders::is_this_computer(&folder.account_id))
        .filter(|folder| !listed.contains(&folder.path))
        .map(|folder| folder.id)
        .collect()
}

/// What to record about a folder after a sync, given what was recorded before.
///
/// The whole of it is one rule a sync must not break: **an answer is not a
/// sync's to overwrite.** Somebody who has said to keep a folder the server no
/// longer lists has decided, and a sync putting that row back to undecided
/// would put the same question to them at the next launch, and the one after
/// that, for as long as the server went on not listing it. That is the dialog
/// storm arriving once a session instead of once a minute, which is slower and
/// no better.
///
/// A folder in the answer goes back to plainly listed whatever was recorded
/// before, including an answer. The decision was about a folder that had gone;
/// the folder is back, so there is nothing left for the decision to be about,
/// and leaving it would mean a folder that vanished and returned could never be
/// asked about again.
pub fn what_the_server_now_says(
    before: WhatTheServerSaid,
    still_listed: bool,
) -> WhatTheServerSaid {
    match (still_listed, before) {
        (true, _) => WhatTheServerSaid::ItListedIt,
        (false, WhatTheServerSaid::ItStoppedListingItAndSomebodySaidKeepIt) => {
            WhatTheServerSaid::ItStoppedListingItAndSomebodySaidKeepIt
        }
        (false, _) => WhatTheServerSaid::ItStoppedListingIt,
    }
}

/// The path of the folder this one sits under, if its own path names one.
///
/// The wire path, not the readable one: the wire path is the identifier, it is
/// what `UNIQUE(account_id, path)` keys on, and it is what the lookup above
/// compares against. Deriving it from the readable form would fail for exactly
/// the folder whose name could not be decoded, which is the reason `ImapFolder`
/// keeps the two apart in the first place.
///
/// `None` for a folder at the top level, for one the server gave no separator
/// for, and for one whose separator is empty. All three mean the same thing
/// here: nothing to split, so nothing is split. A separator of several
/// characters is matched whole, because nothing in the protocol promises one.
fn the_folder_above(folder: &ImapFolder) -> Option<&str> {
    let separator = folder.delimiter.as_deref().filter(|sep| !sep.is_empty())?;
    let (above, _leaf) = folder.path.rsplit_once(separator)?;
    (!above.is_empty()).then_some(above)
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
/// Six methods, which is the whole of what a sync and a backfill ask between
/// them. Anything larger would be a description of the IMAP client rather than
/// of what this needs.
///
/// Crate-private, and it stays that way: this is the seam the tests need, not
/// something a caller should have to know about. That is why the backfill has
/// two entry points, [`fetch_the_missing_message_text`] taking the real
/// controller and [`fetch_over_a_mailbox`] taking anything shaped like one.
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

    /// The messages numbered above `after`, by uid.
    ///
    /// The narrow question, and the whole of what a resumed sync needs to ask:
    /// a uid is handed out once and never reused while UIDVALIDITY holds, so
    /// everything that arrived since the last sync is numbered above the
    /// highest one this computer already has.
    ///
    /// Its answer says nothing about the messages below `after`, which is why
    /// it is carried as [`ServerListing::PartOfIt`] and cannot reach the forget
    /// path. On a folder of forty thousand messages this is a handful of
    /// numbers where [`Mailbox::list_uids`] is forty thousand.
    async fn list_uids_above(&self, folder: &str, after: u32) -> Result<Vec<u32>>;

    /// The headers of the named messages.
    async fn fetch_headers(&self, folder: &str, uids: &[u32]) -> Result<Vec<ImapMessage>>;

    /// Move one message to another folder at the server.
    ///
    /// On the trait because a rule that files mail has to reach the server:
    /// doing it in the cache alone shows a message in a folder it is not in
    /// until the next sync puts it back.
    ///
    /// Answers with what really happened, because a server without MOVE gets
    /// a copy and a flag instead and the message is then in both folders.
    /// That is a fact about somebody's mail and is theirs to hear.
    async fn move_message(
        &self,
        from: &str,
        uid: u32,
        into: &str,
    ) -> Result<crate::service::protocols::imap::Moved>;

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

    /// One whole message exactly as it arrived.
    ///
    /// On the trait because [`fetch_the_missing_message_text`] asks for one
    /// per message, and that per-message call is the property that makes the
    /// read gate real: it is the first line of `ImapSession::fetch_body`, so
    /// reading turned off while a backfill runs stops it at the next message
    /// rather than at the next run.
    async fn fetch_message_body(&self, folder: &str, uid: u32) -> Result<Vec<u8>>;
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

    async fn list_uids_above(&self, folder: &str, after: u32) -> Result<Vec<u32>> {
        MailController::list_uids_above(self, folder, after).await
    }

    async fn move_message(
        &self,
        from: &str,
        uid: u32,
        into: &str,
    ) -> Result<crate::service::protocols::imap::Moved> {
        MailController::move_message(self, from, uid, into).await
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

    async fn fetch_message_body(&self, folder: &str, uid: u32) -> Result<Vec<u8>> {
        MailController::fetch_message_body(self, folder, uid).await
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
pub(crate) fn apply_rules(
    cache: &MessageCache,
    filtering: &Filtering<'_>,
    arrived: &[i64],
) -> Filtered {
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
        // Named before anything else is done to the message, so a move is
        // still asked for even when the same rules also marked it read.
        if let Some(into) = outcome.move_to.clone() {
            done.to_move.push(Moving {
                message_row: message.id,
                uid: message.uid,
                into,
            });
        }
        match carry_out(cache, &message, &outcome) {
            Ok(Carried::Everything) => done.changed += 1,
            // Counted by `carry_out_the_moves` when the move has really
            // happened, and not here as well. Counting it in both places made
            // one message two in "sorted by your rules" whenever a rule marked
            // it read and another filed it, and counting it here alone
            // reported a move the server went on to refuse as done.
            Ok(Carried::ExceptTheMove) => {}
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
) -> Result<Carried> {
    let id = message.id;
    if outcome.delete {
        // Locally. Taking it off the server is the move-to-trash path, which
        // is somebody's own deliberate action rather than a rule's.
        cache.delete_message(id)?;
        return Ok(Carried::Everything);
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
    Ok(match outcome.move_to.is_some() {
        true => Carried::ExceptTheMove,
        false => Carried::Everything,
    })
}

/// Whether everything a message's rules asked for has now been done.
///
/// A move is the one action this half cannot finish, because it needs a
/// server. It used to be reported as an action this program cannot do at all,
/// which stopped being true when moves were built: the count of rules nobody
/// had built counted rules that were carried out a moment later, and the
/// message was counted a second time when the move landed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Carried {
    /// Everything the rules asked for is done.
    Everything,
    /// Everything but the move, which reaches the server in
    /// [`carry_out_the_moves`] and is counted there, once, when it has really
    /// happened.
    ExceptTheMove,
}

/// Do the moves the rules asked for, and say how many really happened.
///
/// The server first, then the cache. The other order shows a message in a
/// folder it is not in until the next sync puts it back, which is why this
/// was left unbuilt rather than half-built.
///
/// A folder the account does not have is passed over with a word in the log
/// rather than failing the whole sync: a rule naming a folder somebody has
/// since renamed should not stop their mail arriving.
async fn carry_out_the_moves<M: Mailbox>(
    controller: &M,
    cache: &MessageCache,
    from: &ImapFolder,
    from_id: i64,
    moves: &[Moving],
) -> (usize, Vec<String>) {
    let mut could_not = Vec::new();
    if moves.is_empty() {
        return (0, could_not);
    }
    // Everything below turns the folder name a rule uses into a folder on the
    // server, which takes the account's folder list. Without it nothing can be
    // filed, and giving up quietly made that sync read exactly like a sync
    // with no rules in it. One sentence per message, as everywhere else here,
    // so the count is right and the summary folds the repeats into one.
    let every_one_of_them_dropped =
        || vec![THE_FOLDER_LIST_COULD_NOT_BE_READ.to_string(); moves.len()];
    let Ok(Some(account_id)) = cache.account_of_folder(from_id) else {
        return (0, every_one_of_them_dropped());
    };
    let Ok(folders) = cache.get_folders_for_account(&account_id) else {
        return (0, every_one_of_them_dropped());
    };
    let mut done = 0;
    for moving in moves {
        let Some(into) = the_folder_a_rule_names(&folders, &moving.into) else {
            could_not.push(no_folder_of_that_name(&moving.into));
            continue;
        };
        // Already where the rule wants it, so there is nothing to do and
        // nothing to report. Rules run on whatever has just arrived in
        // whichever folder is being checked, so a rule that files a sender
        // into a folder goes on matching that sender's mail once it is in
        // that folder. Asking a server to move a message into the folder it
        // is in either fails, once per message on every check for mail, or
        // takes a copy, and nothing anywhere removes duplicates.
        if into.id == from_id {
            continue;
        }
        // What really happened, not just whether it failed. A server without
        // MOVE copies instead, which leaves the message in both folders, and
        // that is somebody's mail being in two places.
        match controller
            .move_message(&from.path, moving.uid, &into.path)
            .await
        {
            Ok(crate::service::protocols::imap::Moved::Moved) => {}
            // Copied, not moved. Not counted as sorted, and the row here is
            // left where it is: the message really is still in this folder,
            // so moving the row would hide a second copy that exists. It also
            // keeps this uid in what the cache holds, which is what stops the
            // next sync fetching the original again as though it were new mail
            // and filing a third copy of it.
            Ok(crate::service::protocols::imap::Moved::CopiedAndFlagged(_)) => {
                could_not.push(copied_and_the_original_marked(&from.name, &into.name));
                continue;
            }
            Ok(crate::service::protocols::imap::Moved::CopiedAndNotFlagged(said)) => {
                could_not.push(copied_and_the_original_left(&from.name, &into.name, &said));
                continue;
            }
            Err(why) => {
                could_not.push(it_is_still_where_it_was(&into.name, &from.name, &why));
                continue;
            }
        }
        // The server has it in the new folder; bring this computer into line.
        // A failure here leaves the two disagreeing until the next read of
        // either folder, which is worth saying and is not a failed sync.
        match cache.move_message(moving.message_row, into.id) {
            Ok(()) => {}
            Err(why) => {
                could_not.push(format!(
                    "A message was filed into {} at the server but not here yet: {why}",
                    into.name
                ));
                continue;
            }
        }
        done += 1;
    }
    (done, could_not)
}

/// What to say when the account's folder list could not be read at all.
///
/// Nothing a rule files elsewhere can be filed without it, so this is every
/// move that sync asked for, dropped before it reached the server.
pub(crate) const THE_FOLDER_LIST_COULD_NOT_BE_READ: &str = "The folder list for this account could not be read, so nothing a rule files into another \
     folder was filed, and every one of those messages is where it was";

/// The folder a rule means, out of the ones the account has.
///
/// By name or by path, because a rule can be written either way: the Rules
/// Manager offers folders by the name somebody reads, and blocking a sender
/// writes the path of the Junk folder it found. One place answers it for both
/// kinds of account, so a rule that files mail on one cannot quietly file
/// nothing on the other.
pub(crate) fn the_folder_a_rule_names<'a>(
    folders: &'a [CachedFolder],
    named: &str,
) -> Option<&'a CachedFolder> {
    folders
        .iter()
        .find(|folder| folder.name.eq_ignore_ascii_case(named) || folder.path == named)
}

/// What to say when a rule names a folder the account does not have.
///
/// Renaming a folder leaves every rule that files into it naming something
/// that is no longer there, and the rule goes on looking enabled in the Rules
/// Manager for as long as nobody opens it.
pub(crate) fn no_folder_of_that_name(named: &str) -> String {
    format!(
        "A rule files mail into {named}, which this account does not have, so it was left where it is"
    )
}

/// What to say when the filing itself would not happen.
///
/// Both folders by name and the reason after them, so the sentence says where
/// the message actually is rather than only that something went wrong.
pub(crate) fn it_is_still_where_it_was(
    into: &str,
    from: &str,
    why: &impl std::fmt::Display,
) -> String {
    format!("A message could not be filed into {into}, so it is still in {from}: {why}")
}

/// The sentence both copying answers open with, so they cannot drift apart.
///
/// Both folders by name. [`crate::service::protocols::imap::Moved::spoken`]
/// says "this folder", which is right beside a message somebody is looking at
/// and names nothing in a sync summary: the summary is about a folder the
/// reader is not necessarily in, and several folders are summed up in a row.
///
/// One fact to a sentence, because this is read aloud. Written as one sentence
/// it ran "so it is in both folders, because it would not mark the one left
/// behind", which is a listener holding a clause open while the reason for it
/// arrives.
fn copied_rather_than_moved(from: &str, into: &str) -> String {
    format!(
        "A rule filed a message into {into} and the server copied it rather than moving it, \
         so the message is now in both {into} and {from}."
    )
}

/// What to say when the server copied a message and marked the original.
fn copied_and_the_original_marked(from: &str, into: &str) -> String {
    format!(
        "{} The one in {from} is marked for removal and goes when that folder is tidied up.",
        copied_rather_than_moved(from, into)
    )
}

/// What to say when the server copied a message and left the original alone.
///
/// Kept apart from the marked case because they are different facts about
/// somebody's mail. A copy marked for removal goes on its own; one nothing
/// marked stays until somebody deletes it, and filing it again makes a third.
fn copied_and_the_original_left(from: &str, into: &str, why: &str) -> String {
    format!(
        "{} The server would not mark the copy left behind: {why}. \
         Delete the one in {from} yourself. Filing it again would make a third copy.",
        copied_rather_than_moved(from, into)
    )
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
    // Read before it is written over below, because the resume decision needs
    // the numbering this computer synced under, not the one it is about to
    // store.
    let stored_uid_validity = cache.folder_uid_validity(folder_id)?;
    let renumbered = matches!(
        (status.uid_validity, stored_uid_validity),
        (Some(now), Some(before)) if now != before
    );
    // Said as well as logged. The log line stays because a log is where
    // somebody looks after the fact; the count leaves here because nobody
    // watching the program run ever sees a log line.
    let discarded_after_renumbering = if renumbered {
        // Every UID we hold now names a different message, or none.
        tracing::info!(
            "{} was renumbered by the server; re-reading it",
            folder.name
        );
        cache.forget_folder_messages(folder_id)?
    } else {
        0
    };
    if let Some(validity) = status.uid_validity {
        cache.set_folder_uid_validity(folder_id, validity)?;
    }

    // Read before the question is asked rather than after, because the highest
    // uid held is one of the four facts that decide which question it is.
    let stored = cache.stored_uids(folder_id)?;

    // The narrowing plan 03-01's type was built to make safe. Each arm makes
    // its own claim about how much of the folder the answer covers, on the one
    // line that knows whether the claim is true, and the forget path below
    // accepts only the first of them.
    let asking = what_to_ask_for(
        WhatThisComputerHolds {
            uid_validity: stored_uid_validity,
            modseq: cache.folder_modseq(folder_id)?,
            highest_uid: stored.iter().copied().max(),
        },
        WhatTheServerReports {
            uid_validity: status.uid_validity,
            highest_modseq: status.highest_modseq,
        },
    );
    let on_server = match asking {
        // Asks the server about the whole folder and gets the whole folder
        // back, so the claim holds.
        WhatToAskFor::EveryUid => {
            ServerListing::TheWholeMailbox(controller.list_uids(&folder.path).await?)
        }
        // Asks about the uids above the highest one held and nothing else. The
        // answer says nothing about the messages below it, so it is part of the
        // folder however few or many come back, and `uids_to_forget` hands back
        // nothing for it rather than every uid it did not name.
        WhatToAskFor::TheUidsAbove(highest) => {
            ServerListing::PartOfIt(controller.list_uids_above(&folder.path, highest).await?)
        }
    };

    if listing_contradicts_the_count(&on_server, counts.total) {
        let counted = crate::service::caldav::how_many(counts.total as usize, "message");
        let name = &folder.name;
        return Err(Error::Protocol(format!(
            "The mail server says {name} holds {counted}, then listed none of them. Nothing has been removed from this computer."
        )));
    }

    let forgotten = uids_to_forget(&on_server, &stored);
    cache.forget_messages(folder_id, &forgotten)?;

    let wanted = uids_to_fetch(on_server.uids(), &stored, limit);
    let fetched = controller.fetch_headers(&folder.path, &wanted).await?;
    // One transaction for the batch rather than one per message. The sync has
    // its own connection on a worker thread, so writing them one at a time
    // took the database's write lock once per message and left the interface's
    // connection waiting behind each one.
    let spam = folder.folder_type == crate::common::types::FolderType::Spam;
    let arriving: Vec<_> = fetched
        .iter()
        .map(|message| to_incoming(message, folder_id, spam))
        .collect();
    let arrived = cache.upsert_messages(&arriving)?;

    // Rules, on what has just arrived and nothing else. Running them over
    // messages already held would apply them again on every sync, and a rule
    // somebody has since changed their mind about would keep firing on mail
    // they had already sorted by hand.
    let mut filtered = match filtering {
        Some(rules) => apply_rules(cache, rules, &arrived),
        None => Filtered::default(),
    };
    // What the rules said belongs elsewhere, done here because this is the
    // half with a server. Each one reaches the server first and the cache
    // second, so a move the server refuses leaves the message where it is
    // rather than showing it somewhere it is not.
    let (filed, could_not) =
        carry_out_the_moves(controller, cache, folder, folder_id, &filtered.to_move).await;
    filtered.changed += filed;
    for reason in &could_not {
        // Logged as well as said. The summary reads out the first few and says
        // how many others there are, and this is where those others are. A
        // status line is also gone as soon as the next one replaces it.
        //
        // Folder names and what the server answered, never a subject: a
        // subject line is close enough to the message to be held to the same
        // rule as its body.
        tracing::warn!("A rule could not file a message: {reason}");
    }
    filtered.could_not_be_filed = could_not;

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

    // Message text is kept so a message opened once can be read again without
    // the server, and until this call the keeping had no end: every body ever
    // fetched stayed for as long as the account did. Applied here because this
    // is a worker thread with its own connection, and eviction writes.
    //
    // Reported rather than propagated. A cache that could not be trimmed is
    // worth knowing about and is not a reason to fail a sync that otherwise
    // worked: the mail is downloaded, and refusing it here would lose that
    // over a housekeeping step.
    match cache.keep_bodies_within_budget() {
        Ok(0) => {}
        Ok(freed) => tracing::debug!("Freed {freed} bytes of cached message text"),
        Err(e) => tracing::warn!("Could not bring the message text cache within its budget: {e}"),
    }

    Ok(FolderSync {
        folder: folder.name.clone(),
        fetched: fetched.len(),
        forgotten: forgotten.len(),
        // The server's own count rather than the length of the listing, and
        // that is the resume's doing. On the whole-mailbox path the two agree,
        // and the check above refuses the sync where they do not. On a resumed
        // one the listing is however many messages arrived since the last sync,
        // so reading it as the folder's total would say a forty thousand
        // message inbox holds three, take "Get older messages" off the menu,
        // and say "3 of 3" to somebody looking at a full mailbox.
        total_on_server: counts.total as usize,
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
        discarded_after_renumbering,
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

/// What a backfill of missing message text came to.
///
/// Four outcomes rather than a count and a flag. "Reading is off", "there was
/// nothing to fetch" and "it ran and fetched none" are three different pieces
/// of news, and a person acts differently on each: turn a setting on, nothing
/// to do, or something is wrong with the connection. Collapsing any two of
/// them into a zero would report the most reassuring of the three.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Backfill {
    /// Reading was off before anything started, so nothing left the machine.
    ///
    /// Carries the sentence rather than an error code, because what somebody
    /// needs is the name of the setting and where to find it.
    NotAllowed(String),
    /// Nothing in this account is missing text that a server could supply.
    ///
    /// Which is also the answer for a POP account, where every message came
    /// down whole and there is no server left holding another copy.
    NothingToFetch,
    /// It ran.
    Ran(Backfilled),
}

/// How a backfill that really ran got on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backfilled {
    /// Messages whose text arrived and is stored.
    pub fetched: usize,
    /// Messages that were asked for and did not arrive.
    ///
    /// Reported rather than swallowed. A run that gives only its successes
    /// reads as complete, which is the "short answer that looks finished"
    /// the whole disclosure exists to prevent.
    pub could_not: usize,
    /// Whether it reached the end of the list, and if not, why not.
    pub ended: Ending,
}

/// Why a backfill stopped.
///
/// Separate from the two counts because "it stopped early" is not a number.
/// A boolean beside the counts could hold it, and then nothing would carry
/// the sentence saying which setting closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ending {
    /// Every message in the list was attempted.
    WentThroughTheWholeList,
    /// Reading was turned off while it ran, so it stopped at that message.
    ///
    /// Told apart from an ordinary failure by
    /// [`crate::service::outward::was_refused_by_the_gate`], which is the one
    /// answer to that question. A refusal is not a message that would not
    /// arrive: carrying on would ask the server for the rest and be refused
    /// once per message, and report hundreds of failures for one setting.
    ReadingWasTurnedOff(String),
}

/// What the read gate is told this fetch is doing, for its refusal sentence.
///
/// A refusal names the act, so somebody reads why rather than only that
/// something was refused. Spelled out here rather than at the call site so the
/// same words reach the up-front refusal and the mid-run one.
pub const FETCHING_THE_MISSING_TEXT: &str = "fetch the message text that is not stored here";

/// At most this many progress lines between the count and the report.
///
/// Guardrail 5: feedback must be bounded and must not flood. A backfill of
/// three hundred messages that announces each one is not progress reporting,
/// it is three hundred announcements over a status topic that also carries
/// everything else, and the way somebody stops it is to close the program.
///
/// Ten rather than one per message, and never more however much mail there is.
const AT_MOST_THIS_MANY_PROGRESS_LINES: usize = 10;

/// Whether the run says where it has got to, having done `done` of `total`.
///
/// Pure and separate so the bound above is a property with a test rather than
/// a constant somebody trusts. Nothing at all for a short run: the count line
/// and the report are two sentences either side of a few seconds, and a
/// progress line between them would be noise.
pub(crate) fn says_where_it_is(done: usize, total: usize) -> bool {
    // Rounded up, so the step is never nought and the run never says anything
    // after every single message. A step of one means the whole run is shorter
    // than the number of lines it would be allowed, and then it says nothing
    // at all rather than one line per message.
    let step = total.div_ceil(AT_MOST_THIS_MANY_PROGRESS_LINES);
    step > 1 && done < total && done.is_multiple_of(step)
}

/// The line said before the first message is asked for.
///
/// Both numbers written out whole rather than assembled from a stem and an
/// "s", the same as every other counted sentence in this program: three words
/// have to agree, and a sentence built from fragments reads like one.
pub fn about_to_fetch(count: usize) -> String {
    match count {
        1 => "Fetching the text of 1 message that is not stored on this computer.".to_string(),
        many => {
            format!("Fetching the text of {many} messages that are not stored on this computer.")
        }
    }
}

/// The line said part-way through a long run.
///
/// Said at most nine times whatever the size of the run. See
/// [`says_where_it_is`], which decides when, and guardrail 5, which is why.
pub fn how_far_the_fetch_has_got(done: usize, total: usize) -> String {
    format!("Fetched {done} of {total} so far.")
}

/// What a run with nothing to do says.
///
/// Its own sentence rather than a report of nought fetched. "There is nothing
/// to do" and "it tried and got none" are different pieces of news, and the
/// second reads as a fault in the fetch.
pub const NOTHING_TO_FETCH: &str = "Every message in this account already has its text on this computer, so there is \
     nothing to fetch.";

/// How many messages arrived, as a clause.
fn arrived(count: usize) -> String {
    match count {
        0 => "No message text arrived".to_string(),
        1 => "The text of 1 message arrived".to_string(),
        many => format!("The text of {many} messages arrived"),
    }
}

/// How many did not, as a clause.
///
/// Said even when it is none, because a run that gives only its successes
/// reads as complete, and a person cannot tell a run that fetched twelve of
/// twelve from one that fetched twelve of two hundred.
fn did_not_arrive(count: usize) -> String {
    match count {
        0 => "nothing failed".to_string(),
        1 => "1 message could not be fetched".to_string(),
        many => format!("{many} messages could not be fetched"),
    }
}

/// What a finished backfill is told to somebody, in one sentence.
pub fn what_the_fetch_did(outcome: &Backfill) -> String {
    match outcome {
        Backfill::NotAllowed(why) => why.clone(),
        Backfill::NothingToFetch => NOTHING_TO_FETCH.to_string(),
        Backfill::Ran(done) => {
            let stopped = match &done.ended {
                Ending::WentThroughTheWholeList => String::new(),
                Ending::ReadingWasTurnedOff(why) => format!(" {why}"),
            };
            format!(
                "{}, and {}.{stopped}",
                arrived(done.fetched),
                did_not_arrive(done.could_not)
            )
        }
    }
}

/// Fetch the text of every message in one account that has none stored here.
///
/// The other half of what a saved search says it covers: the disclosure names
/// a number, and this is what somebody does about it.
///
/// Not generic, deliberately. [`Mailbox`] is the seam that lets the routine be
/// tested without a server, and a caller has a real controller and should not
/// have to know the seam is there. The work is in [`fetch_over_a_mailbox`]
/// just below.
pub async fn fetch_the_missing_message_text(
    controller: &MailController,
    cache: &MessageCache,
    account_id: &str,
    allowed: crate::application::allowed::Allowed,
    say: &dyn Fn(&str),
) -> Result<Backfill> {
    fetch_over_a_mailbox(controller, cache, account_id, allowed, say).await
}

/// The backfill, against anything that answers like a mailbox.
///
/// Where the work is, and where every test of it runs.
pub(crate) async fn fetch_over_a_mailbox<M: Mailbox>(
    server: &M,
    cache: &MessageCache,
    account_id: &str,
    allowed: crate::application::allowed::Allowed,
    say: &dyn Fn(&str),
) -> Result<Backfill> {
    // Before the list is read and before anything is asked of a server. The
    // gate itself is `fetch_body`'s first line and would refuse every message
    // on its own, but it would do so after a connection had been opened, and
    // once per message. Somebody who has turned reading off should be told at
    // once, in a sentence naming the setting, which is D-2-06.
    if !allowed.reading {
        let refusal = crate::service::outward::read_refusal(FETCHING_THE_MISSING_TEXT);
        say(&refusal);
        return Ok(Backfill::NotAllowed(refusal));
    }

    let wanted = cache.messages_with_no_text_here(account_id)?;
    if wanted.is_empty() {
        say(NOTHING_TO_FETCH);
        return Ok(Backfill::NothingToFetch);
    }

    // Before the first request, because the number is the whole point: the
    // coverage sentence said how much text is missing a moment ago, and a
    // fetch that then runs silently for an unknown length of time is the one
    // somebody stops half way through.
    let total = wanted.len();
    say(&about_to_fetch(total));

    let mut done = Backfilled {
        fetched: 0,
        could_not: 0,
        ended: Ending::WentThroughTheWholeList,
    };
    for (attempted, message) in wanted.iter().enumerate() {
        match fetch_and_store_one(server, cache, message).await {
            Ok(()) => done.fetched += 1,
            // Told apart from an ordinary failure by the one function that
            // answers that question. Somebody turning reading off while this
            // runs is not hundreds of messages that would not arrive: going on
            // would ask the server for every one of them, be refused each
            // time, and report a folder full of failures for one setting.
            Err(e) if crate::service::outward::was_refused_by_the_gate(&e) => {
                done.ended = Ending::ReadingWasTurnedOff(crate::service::outward::read_refusal(
                    FETCHING_THE_MISSING_TEXT,
                ));
                break;
            }
            Err(e) => {
                // The uid and the reason, and nothing that came back. This is
                // the one routine in the program holding hundreds of message
                // bodies in a row, so a log line built from the payload would
                // put somebody's mail on the disk in plain text outside the
                // cache. Every error in this path is worded by us for the same
                // reason.
                tracing::warn!("Could not fetch the text of message {}: {e}", message.uid);
                done.could_not += 1;
            }
        }
        if says_where_it_is(attempted + 1, total) {
            say(&how_far_the_fetch_has_got(attempted + 1, total));
        }
    }

    let outcome = Backfill::Ran(done);
    say(&what_the_fetch_did(&outcome));
    Ok(outcome)
}

/// Fetch one message, read it, and store its text.
///
/// The same three steps the single message download in the window already
/// does, in the same order and through the same functions, rather than a
/// second parse and a second store. The signed original is kept for the reason
/// that path gives: a signature is arithmetic over exactly the bytes that
/// arrived, and nothing the parse produces can be turned back into them, so a
/// message reopened from the cache could never be checked again. Ordinary mail
/// writes nothing there and the call decides that for itself.
///
/// A failure to keep those bytes costs a verdict rather than a message, so it
/// is logged and the message still counts as fetched. Anything earlier is a
/// real failure and is handed back to be counted.
async fn fetch_and_store_one<M: Mailbox>(
    server: &M,
    cache: &MessageCache,
    message: &crate::data::message_cache::bodies::MessageToFetch,
) -> Result<()> {
    let raw = server
        .fetch_message_body(&message.folder_path, message.uid)
        .await?;
    let parsed = crate::service::mime::parse(&raw)?;
    cache.save_message_body(
        message.message_id,
        parsed.body_plain.as_deref(),
        parsed.body_html.as_deref(),
    )?;
    if let Err(e) = cache.keep_signed_original(message.message_id, &raw) {
        tracing::warn!("Could not keep the form a signed message arrived in: {e}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
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

        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
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

    /// A message as it arrives, for a rule to be run over.
    fn an_arriving_message(folder_id: i64, uid: u32, subject: &str) -> IncomingMessage {
        IncomingMessage {
            folder_id,
            uid,
            message_id: format!("{uid}@example.com"),
            subject: subject.to_string(),
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
        }
    }

    #[test]
    fn test_a_rule_that_moves_says_which_message_and_where() {
        // Moving needs the folder's id and a write to the server, neither of
        // which `apply_rules` has: it runs with a cache and no connection.
        // So it names what has to move and the sync, which does have a
        // server, carries it out. Doing it in the cache alone would show a
        // message in a folder it is not in until the next sync put it back.
        use crate::application::filters::FilterEngine;
        use crate::data::message_cache::MessageFilterRule;

        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
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
            .upsert_message(&an_arriving_message(folder_id, 41, "Invoice 2026-08"))
            .expect("a message");

        let mut engine = FilterEngine::default();
        engine.load_from_persisted(&[MessageFilterRule {
            id: "r1".into(),
            account_id: "acct".into(),
            name: "File the invoices".into(),
            field: "subject".into(),
            match_type: "contains".into(),
            pattern: "Invoice".into(),
            case_sensitive: false,
            action_type: "move_to_folder".into(),
            action_value: Some("Invoices".into()),
            enabled: true,
            created_at: "2026-08-24T00:00:00Z".into(),
        }]);

        let done = apply_rules(
            &cache,
            &Filtering {
                rules: &engine,
                allowed: crate::application::allowed::Allowed::EVERYTHING,
            },
            &[id],
        );

        assert_eq!(
            done.to_move,
            vec![Moving {
                message_row: id,
                uid: 41,
                into: "Invoices".to_string(),
            }],
            "the rule did not say which message goes where"
        );
        assert_eq!(
            done.changed, 0,
            "nothing has moved yet, so nothing is counted as sorted"
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

        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
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

        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
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

    /// What [`Scripted`] writes down when it is asked to list a whole folder.
    ///
    /// A constant rather than a string in each test, because the assertion
    /// these exist for is that the line is *absent*, and a typo in a search for
    /// something that should not be there is a test that passes for the wrong
    /// reason and can never fail.
    const LISTED_EVERY_UID: &str = "listed every uid";

    /// And when it is asked the narrow question. The uid it was asked about is
    /// on the end, so a test can say which resume it saw.
    const LISTED_THE_UIDS_ABOVE: &str = "listed the uids above ";

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
        /// Every move this server was asked to make, so a test can check that
        /// a rule that files mail really reached it.
        moved: std::cell::RefCell<Vec<(String, u32, String)>>,
        /// What this server answers a move with.
        ///
        /// A server without MOVE copies the message and leaves the original,
        /// which is the answer that puts somebody's mail in two folders. There
        /// was no way to write a test about it: this double always answered
        /// that the move had happened.
        answers_a_move_with: crate::service::protocols::imap::Moved,
        /// Which uids the header fetch was asked for, so a test can check that
        /// a sync asked for the right ones rather than only that it ended up
        /// with the right rows.
        asked_for: std::cell::RefCell<Vec<u32>>,
        /// A server that will not say which messages a mailbox holds.
        refuse_list_uids: bool,
        /// How this server answers a request for a whole message, by uid.
        ///
        /// A uid with no entry is answered with an ordinary message, so a test
        /// about one awkward message names only that one.
        bodies: std::collections::HashMap<u32, AnswersABodyWith>,
        /// Every fetch, and every line the run said, in the order they
        /// happened.
        ///
        /// One log rather than two, because "the count is said before the
        /// first fetch" is a claim about the order of two different kinds of
        /// event, and two lists cannot be interleaved after the fact.
        happened: std::rc::Rc<std::cell::RefCell<Vec<String>>>,
    }

    /// How a scripted server answers a request for one message's whole text.
    #[derive(Clone)]
    enum AnswersABodyWith {
        /// A failure about this one message. The run counts it and goes on.
        AFailure,
        /// A refusal by the read gate, which is what somebody turning reading
        /// off part-way through a run looks like from in here.
        TheGateIsClosed,
    }

    impl Default for Scripted {
        fn default() -> Self {
            Self {
                on_server: Vec::new(),
                headers: Vec::new(),
                flags: Vec::new(),
                moved: std::cell::RefCell::new(Vec::new()),
                counts: crate::service::protocols::imap::FolderCounts {
                    total: 0,
                    unread: 0,
                },
                uid_validity: Some(1),
                highest_modseq: None,
                asked_for: std::cell::RefCell::new(Vec::new()),
                refuse_list_uids: false,
                answers_a_move_with: crate::service::protocols::imap::Moved::Moved,
                bodies: std::collections::HashMap::new(),
                happened: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
            }
        }
    }

    impl Mailbox for Scripted {
        async fn move_message(
            &self,
            from: &str,
            uid: u32,
            into: &str,
        ) -> Result<crate::service::protocols::imap::Moved> {
            self.moved
                .borrow_mut()
                .push((from.to_string(), uid, into.to_string()));
            Ok(self.answers_a_move_with.clone())
        }

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
            // Recorded before the refusal, because a server that was asked and
            // said no was still asked, and this log is about what the sync
            // asked for.
            self.happened
                .borrow_mut()
                .push(LISTED_EVERY_UID.to_string());
            if self.refuse_list_uids {
                return Err(crate::common::Error::Protocol(
                    "The mail server refused while searching the folder.".to_string(),
                ));
            }
            Ok(self.on_server.clone())
        }

        async fn list_uids_above(&self, _folder: &str, after: u32) -> Result<Vec<u32>> {
            self.happened
                .borrow_mut()
                .push(format!("{LISTED_THE_UIDS_ABOVE}{after}"));
            // The open-ended range a real server answers. `n:*` on a mailbox
            // whose highest uid is below `n` comes back with that highest one
            // rather than empty, because the specification reads a reversed
            // range the other way round, and this double says so rather than
            // being tidier than the thing it stands for.
            let above: Vec<u32> = self
                .on_server
                .iter()
                .copied()
                .filter(|uid| *uid >= after)
                .collect();
            Ok(match (above.is_empty(), self.on_server.iter().max()) {
                (true, Some(highest)) => vec![*highest],
                _ => above,
            })
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

        async fn fetch_message_body(&self, _folder: &str, uid: u32) -> Result<Vec<u8>> {
            self.happened.borrow_mut().push(format!("fetched {uid}"));
            match self.bodies.get(&uid) {
                Some(AnswersABodyWith::AFailure) => Err(crate::common::Error::Protocol(
                    "The mail server would not hand that message over.".to_string(),
                )),
                Some(AnswersABodyWith::TheGateIsClosed) => {
                    crate::service::outward::permitted_to_read(false, FETCHING_THE_MISSING_TEXT)
                        .map(|()| Vec::new())
                }
                None => Ok(a_whole_message(uid)),
            }
        }
    }

    /// One message as it comes off a server, with a word only its text holds.
    ///
    /// "Bezoar" is nowhere in the subject or the sender, and it is past the two
    /// hundred characters a snippet keeps, so finding it afterwards can only
    /// have come from the text being stored and indexed.
    fn a_whole_message(uid: u32) -> Vec<u8> {
        format!(
            "Subject: Quarterly report {uid}\r\n\
             From: sender@example.com\r\n\
             To: me@example.com\r\n\
             \r\n\
             {}\r\n\
             The word this test looks for is bezoar.\r\n",
            "Padding so the word below falls past the snippet. ".repeat(6)
        )
        .into_bytes()
    }

    /// A cache holding `how_many` messages in one account, none with any text.
    fn an_account_with_no_message_text(cache: &MessageCache, folder_id: i64, how_many: u32) {
        for uid in 1..=how_many {
            cache
                .upsert_message(&IncomingMessage {
                    folder_id,
                    uid,
                    message_id: format!("<{uid}@example.com>"),
                    refs_header: None,
                    subject: format!("Quarterly report {uid}"),
                    from_addr: "sender@example.com".into(),
                    to_addr: "me@example.com".into(),
                    cc: None,
                    reply_to: None,
                    date: format!("2026-07-{uid:02}"),
                    internal_date: None,
                    size_bytes: None,
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
        }
    }

    /// Run a backfill, handing back what it answered and everything it said.
    fn backfill(server: &Scripted, cache: &MessageCache, reading: bool) -> (Backfill, Vec<String>) {
        let happened = std::rc::Rc::clone(&server.happened);
        let say = {
            let happened = std::rc::Rc::clone(&happened);
            move |line: &str| happened.borrow_mut().push(format!("said {line}"))
        };
        let allowed = crate::application::allowed::Allowed {
            mail: false,
            personal_information: false,
            reading,
        };
        let outcome = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
            .block_on(fetch_over_a_mailbox(server, cache, "acct", allowed, &say))
            .expect("the backfill answers");
        let log = happened.borrow().clone();
        (outcome, log)
    }

    #[test]
    fn test_with_reading_off_nothing_is_asked_of_a_server() {
        // D-2-06: a refusal is a sentence naming the setting, not an error
        // code and not a silent nought. And nothing may reach the server: the
        // point of the up-front check is that somebody is told before a
        // connection is opened rather than after.
        let (cache, folder_id, _) = a_cache();
        an_account_with_no_message_text(&cache, folder_id, 3);
        let server = Scripted::default();

        let (outcome, log) = backfill(&server, &cache, false);

        let Backfill::NotAllowed(sentence) = &outcome else {
            panic!("reading was off and the backfill did not refuse: {outcome:?}");
        };
        assert!(
            sentence.contains(crate::application::allowed::READING_SECTION),
            "the refusal does not name the setting to turn on: {sentence}"
        );
        assert!(
            !log.iter().any(|line| line.starts_with("fetched")),
            "a refused backfill still asked the server for something: {log:?}"
        );
    }

    #[test]
    fn test_each_missing_message_is_fetched_stored_and_then_findable() {
        // The end to end claim of this plan, and the one that says the fetch
        // is worth having: the coverage numbers move afterwards, and so does
        // what a search can reach.
        let (cache, folder_id, _) = a_cache();
        an_account_with_no_message_text(&cache, folder_id, 3);
        let server = Scripted::default();

        let (outcome, _) = backfill(&server, &cache, true);

        assert_eq!(
            outcome,
            Backfill::Ran(Backfilled {
                fetched: 3,
                could_not: 0,
                ended: Ending::WentThroughTheWholeList,
            }),
            "the run did not fetch all three"
        );
        assert!(
            cache
                .messages_with_no_text_here("acct")
                .expect("the list")
                .is_empty(),
            "messages are still missing their text after a run that fetched them"
        );
        // Two searches, because they read different things and only one of
        // them proves the store went through `save_message_body`. The scan
        // reads `message_bodies`, which a direct insert would also satisfy.
        // The word search reads the index, which only `index_message_for_search`
        // fills, and `save_message_body` is the one thing that calls it.
        let scanned = cache
            .messages_a_saved_search_reads(
                "acct",
                None,
                crate::data::message_cache::saved_searches::TheMessageText::Read,
            )
            .expect("the scan");
        assert_eq!(
            scanned
                .iter()
                .filter(|message| message
                    .body_plain
                    .as_deref()
                    .is_some_and(|text| text.contains("bezoar")))
                .count(),
            3,
            "a saved search cannot reach the text that was just fetched"
        );
        let found = cache
            .search_messages(
                "acct",
                "bezoar",
                crate::data::message_cache::WhereToSearch::EveryFolder,
                10,
            )
            .expect("the search");
        assert_eq!(
            found.len(),
            3,
            "the fetched text was stored without being indexed"
        );
    }

    #[test]
    fn test_reading_turned_off_part_way_stops_the_run_at_the_next_message() {
        // The gate is `fetch_body`'s first line and the loop calls it once per
        // message, so a setting changed mid-run is honoured at the next one.
        // A refusal is not a message that would not arrive: carrying on would
        // ask for the rest and be refused once each, and report a folder full
        // of failures for one setting.
        let (cache, folder_id, _) = a_cache();
        an_account_with_no_message_text(&cache, folder_id, 4);
        let mut server = Scripted::default();
        // Newest first, so the second message asked for is uid 3.
        server.bodies.insert(3, AnswersABodyWith::TheGateIsClosed);

        let (outcome, log) = backfill(&server, &cache, true);

        let Backfill::Ran(done) = &outcome else {
            panic!("the run did not start: {outcome:?}");
        };
        assert_eq!(
            done.fetched, 1,
            "it did not fetch the message before the refusal"
        );
        let Ending::ReadingWasTurnedOff(sentence) = &done.ended else {
            panic!(
                "a refusal part way through was not reported as one: {:?}",
                done.ended
            );
        };
        assert!(
            sentence.contains(crate::application::allowed::READING_SECTION),
            "the refusal does not name the setting to turn on: {sentence}"
        );
        assert_eq!(
            log.iter()
                .filter(|line| line.starts_with("fetched"))
                .count(),
            2,
            "it went on asking after being refused: {log:?}"
        );
    }

    #[test]
    fn test_one_message_that_will_not_fetch_does_not_end_the_run() {
        let (cache, folder_id, _) = a_cache();
        an_account_with_no_message_text(&cache, folder_id, 3);
        let mut server = Scripted::default();
        server.bodies.insert(2, AnswersABodyWith::AFailure);

        let (outcome, _) = backfill(&server, &cache, true);

        assert_eq!(
            outcome,
            Backfill::Ran(Backfilled {
                fetched: 2,
                could_not: 1,
                ended: Ending::WentThroughTheWholeList,
            }),
            "one message that would not arrive ended the run, or was not counted"
        );
    }

    #[test]
    fn test_the_count_is_said_before_the_first_message_is_asked_for() {
        // Not decoration. Somebody was told a number a moment ago and is about
        // to wait for an unknown length of time; a fetch that starts silently
        // is the one they kill half way through.
        let (cache, folder_id, _) = a_cache();
        an_account_with_no_message_text(&cache, folder_id, 3);
        let server = Scripted::default();

        let (_, log) = backfill(&server, &cache, true);

        let first = log.first().expect("the run said nothing at all");
        assert!(
            first.starts_with("said") && first.contains('3'),
            "the run did not say how many it would attempt before asking: {log:?}"
        );
    }

    #[test]
    fn test_a_run_with_nothing_to_fetch_says_so_rather_than_reporting_success() {
        // An account whose text is all here, and a POP account, answer the
        // same way. "Nothing to do" and "it ran and fetched none" are
        // different pieces of news and the second reads as a fault.
        let (cache, folder_id, _) = a_cache();
        let server = Scripted::default();

        let (outcome, log) = backfill(&server, &cache, true);

        assert_eq!(outcome, Backfill::NothingToFetch);
        assert!(
            !log.iter().any(|line| line.starts_with("fetched")),
            "a run with nothing to fetch still asked the server: {log:?}"
        );
        let _ = folder_id;
    }

    #[test]
    fn test_progress_lines_are_bounded_however_much_mail_there_is() {
        // Guardrail 5: feedback must be bounded and must not flood. Asserted
        // as a property over the whole range rather than by reading the
        // constant, because the constant is a step size and the bound is what
        // somebody actually hears.
        for total in [1usize, 2, 5, 11, 47, 300, 10_000, 200_000] {
            let lines = (1..=total)
                .filter(|done| says_where_it_is(*done, total))
                .count();
            assert!(
                lines < AT_MOST_THIS_MANY_PROGRESS_LINES,
                "{total} messages would say where it had got to {lines} times"
            );
        }
    }

    #[test]
    fn test_a_short_run_says_nothing_between_the_count_and_the_report() {
        // Two sentences either side of a few seconds. A progress line in
        // between is noise, and it is the same topic as both of them.
        for total in [1usize, 2, 5] {
            assert!(
                (1..=total).all(|done| !says_where_it_is(done, total)),
                "a run of {total} said where it had got to"
            );
        }
    }

    #[test]
    fn test_the_report_carries_both_numbers_even_when_none_failed() {
        // A run that gives only its successes reads as complete.
        let some_failed = what_the_fetch_did(&Backfill::Ran(Backfilled {
            fetched: 12,
            could_not: 3,
            ended: Ending::WentThroughTheWholeList,
        }));
        assert!(
            some_failed.contains("12") && some_failed.contains('3'),
            "the report dropped one of its two numbers: {some_failed}"
        );

        let none_failed = what_the_fetch_did(&Backfill::Ran(Backfilled {
            fetched: 12,
            could_not: 0,
            ended: Ending::WentThroughTheWholeList,
        }));
        assert!(
            none_failed.contains("12"),
            "the report dropped how many arrived: {none_failed}"
        );
        assert!(
            !none_failed.is_empty() && none_failed != some_failed,
            "a run where nothing failed reads the same as one where three did"
        );

        let nothing = what_the_fetch_did(&Backfill::NothingToFetch);
        assert!(
            !nothing.is_empty() && !nothing.contains('0'),
            "nothing to fetch was reported as a count of nought: {nothing}"
        );
    }

    fn a_cache() -> (TempHome<MessageCache>, i64, ImapFolder) {
        let cache = TempHome::named("wixen_sync_", |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache")
        });
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
            delimiter: None,
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
        attempt(server, cache, id, folder, limit).expect("the sync runs")
    }

    /// The same sync, handing back whatever it answered rather than unwrapping.
    ///
    /// What a test about a sync that should fail needs, and there was no way to
    /// write one before: every helper here unwrapped, so a sync that refused
    /// could only be asserted by a panic.
    fn attempt<M: Mailbox>(
        server: &M,
        cache: &MessageCache,
        id: i64,
        folder: &ImapFolder,
        limit: usize,
    ) -> Result<FolderSync> {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
            .block_on(sync_folder(server, cache, folder, id, limit, None))
    }

    #[test]
    fn test_a_sync_brings_the_body_cache_back_under_its_budget() {
        // `evict_bodies_over` was written, tested, documented in the module
        // header as what the cache does, and called by nothing outside its own
        // tests. So the body cache grew without any limit at all: every message
        // ever opened kept its text for as long as the account existed, in a
        // file the documentation correctly says is not encrypted.
        //
        // The budget is applied here, at the end of a folder sync, because that
        // is a worker thread with its own connection. Eviction deletes rows,
        // and a write on the interface thread is the thing this whole layer
        // is careful about.
        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None)
            .expect("a cache")
            .keeping_bodies_under(60);
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
            delimiter: None,
        };

        // Six messages an ordinary sync brought down, each with a body well
        // over the budget between them. Ordinary matters: mail collected over
        // POP and a copy of a sent message are the only copy of their text and
        // are never evicted, so a test built from those would pass by never
        // having a candidate.
        for uid in 1..=6u32 {
            let row = cache
                .upsert_message(&IncomingMessage {
                    folder_id,
                    uid,
                    message_id: format!("held-{uid}@example.com"),
                    subject: format!("Message {uid}"),
                    from_addr: "someone@example.com".into(),
                    to_addr: "me@example.com".into(),
                    cc: None,
                    reply_to: None,
                    date: format!("2026-07-{uid:02}T09:00:00+00:00"),
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
            cache
                .save_message_body(row, Some(&"body text ".repeat(20)), None)
                .expect("a body");
        }

        // Sixty bytes, not the four hundred this first said. Message text is
        // packed before it is stored, so six repetitive bodies came to 126
        // bytes on the disk rather than 1200 and the test sat under its own
        // budget, proving nothing. The guard below is what caught that, and it
        // stays for the next time something changes what a body costs.
        let before = cache.cached_body_bytes().expect("a total");
        assert!(
            before > 60,
            "the test has to start over budget or it proves nothing: {before}"
        );

        // Every one of the six is still on the server, so this sync fetches
        // nothing and forgets nothing. That is the point: the first version of
        // this test left one uid on the server, the sync deleted the other
        // five as gone, their bodies went with them, and the test passed
        // against code that never evicted anything. Eviction has to be the
        // only thing that can bring the total down.
        let server = Scripted {
            on_server: vec![1, 2, 3, 4, 5, 6],
            counts: crate::service::protocols::imap::FolderCounts {
                total: 6,
                unread: 0,
            },
            ..Scripted::default()
        };
        let did = run(&server, &cache, folder_id, &folder);
        assert_eq!(
            did.forgotten, 0,
            "the sync removed messages, so this measures deletion and not eviction"
        );

        let after = cache.cached_body_bytes().expect("a total");
        assert!(
            after <= 400,
            "a sync finished with the body cache still over its budget: \
             {before} bytes before, {after} after, budget 400"
        );
    }

    #[test]
    fn test_a_rule_that_files_mail_reaches_the_server_and_this_computer() {
        // Naming a folder in a rule did nothing at all for as long as rules
        // have existed, and the message was still counted as sorted. It has
        // to reach the server: filing it here alone shows the message in a
        // folder it is not in until the next sync puts it back.
        use crate::application::filters::FilterEngine;
        use crate::data::message_cache::MessageFilterRule;

        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
        let inbox = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acct".into(),
                name: "Inbox".into(),
                path: "INBOX".into(),
                folder_type: "Inbox".into(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("an inbox");
        let invoices = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acct".into(),
                name: "Invoices".into(),
                path: "INBOX/Invoices".into(),
                folder_type: "Custom".into(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder to file into");

        let mut engine = FilterEngine::default();
        engine.load_from_persisted(&[MessageFilterRule {
            id: "r1".into(),
            account_id: "acct".into(),
            name: "File the invoices".into(),
            field: "subject".into(),
            match_type: "contains".into(),
            pattern: "Invoice".into(),
            case_sensitive: false,
            action_type: "move_to_folder".into(),
            action_value: Some("Invoices".into()),
            enabled: true,
            created_at: "2026-08-24T00:00:00Z".into(),
        }]);
        let filtering = Filtering {
            rules: &engine,
            allowed: crate::application::allowed::Allowed::EVERYTHING,
        };

        let server = Scripted {
            on_server: vec![7],
            headers: vec![ImapMessage {
                subject: "Invoice #4021".to_string(),
                ..message(7)
            }],
            ..Default::default()
        };
        let folder = folder("INBOX", FolderType::Inbox, true);

        let done = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
            .block_on(sync_folder(
                &server,
                &cache,
                &folder,
                inbox,
                50,
                Some(&filtering),
            ))
            .expect("the sync to finish");

        assert_eq!(
            server.moved.borrow().as_slice(),
            [("INBOX".to_string(), 7, "INBOX/Invoices".to_string())],
            "the move never reached the server"
        );
        assert_eq!(
            done.filtered.changed, 1,
            "a message that really moved is not counted as sorted"
        );
        assert_eq!(
            cache
                .get_message_list_sorted(invoices, "acct", None, Some(50))
                .expect("the folder reads")
                .len(),
            1,
            "the message is not in the folder here"
        );
    }

    /// An account with an inbox, a folder to file into, and the rules given.
    ///
    /// Each rule is an action and its value, in the order they are written,
    /// which is the order the engine settles them in. Three tests need exactly
    /// this and differ only in the rules and in what the server answers, which
    /// is the thing each of them is about.
    fn an_account_with_rules(
        rules: &[(&str, Option<&str>)],
    ) -> (
        tempfile::TempDir,
        MessageCache,
        i64,
        i64,
        ImapFolder,
        crate::application::filters::FilterEngine,
    ) {
        use crate::data::message_cache::MessageFilterRule;

        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
        let inbox = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acct".into(),
                name: "Inbox".into(),
                path: "INBOX".into(),
                folder_type: "Inbox".into(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("an inbox");
        let invoices = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acct".into(),
                name: "Invoices".into(),
                path: "INBOX/Invoices".into(),
                folder_type: "Custom".into(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder to file into");

        let written: Vec<MessageFilterRule> = rules
            .iter()
            .enumerate()
            .map(|(nth, (action, value))| MessageFilterRule {
                id: format!("r{nth}"),
                account_id: "acct".into(),
                name: format!("Rule {nth}"),
                field: "subject".into(),
                match_type: "contains".into(),
                pattern: "Invoice".into(),
                case_sensitive: false,
                action_type: (*action).to_string(),
                action_value: value.map(str::to_string),
                enabled: true,
                created_at: "2026-08-24T00:00:00Z".into(),
            })
            .collect();
        let mut engine = crate::application::filters::FilterEngine::default();
        engine.load_from_persisted(&written);

        let folder = ImapFolder {
            name: "Inbox".into(),
            display_path: "INBOX".into(),
            path: "INBOX".into(),
            folder_type: FolderType::Inbox,
            selectable: true,
            holds_all_mail: false,
            subscribed: true,
            delimiter: None,
        };
        (dir, cache, inbox, invoices, folder, engine)
    }

    /// One invoice on the server, waiting for the rules to be run over it.
    fn a_server_holding_one_invoice() -> Scripted {
        Scripted {
            on_server: vec![7],
            headers: vec![ImapMessage {
                subject: "Invoice #4021".to_string(),
                ..message(7)
            }],
            ..Default::default()
        }
    }

    fn sync_with_rules(
        server: &Scripted,
        cache: &MessageCache,
        folder: &ImapFolder,
        folder_id: i64,
        engine: &crate::application::filters::FilterEngine,
    ) -> FolderSync {
        let filtering = Filtering {
            rules: engine,
            allowed: crate::application::allowed::Allowed::EVERYTHING,
        };
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
            .block_on(sync_folder(
                server,
                cache,
                folder,
                folder_id,
                50,
                Some(&filtering),
            ))
            .expect("the sync to finish")
    }

    #[test]
    fn test_a_message_the_server_put_in_two_folders_is_not_counted_as_sorted() {
        // A server without MOVE copies the message and marks the original for
        // removal. One that will not even mark it leaves the message in both
        // folders, and that answer used to fall through to the same two lines
        // as a real move: the row was filed here and the message counted into
        // "sorted by your rules", while the sentence explaining it was thrown
        // away. Somebody told 12 were sorted who then finds 12 duplicates has
        // been actively misled.
        let (_dir, cache, inbox, invoices, folder, engine) =
            an_account_with_rules(&[("move_to_folder", Some("Invoices"))]);
        let server = Scripted {
            answers_a_move_with: crate::service::protocols::imap::Moved::CopiedAndNotFlagged(
                "it does not allow that flag".to_string(),
            ),
            ..a_server_holding_one_invoice()
        };

        let done = sync_with_rules(&server, &cache, &folder, inbox, &engine);

        assert_eq!(
            done.filtered.changed, 0,
            "a message the server left in two folders was counted as sorted"
        );
        let said = what_the_folder_sync_did(&done);
        assert!(
            said.contains("Inbox") && said.contains("Invoices"),
            "the sentence did not name both folders the message is in: {said}"
        );
        assert!(
            said.contains("in both Invoices and Inbox"),
            "the sentence did not say the message is in two named places: {said}"
        );
        // The row is left where it is, because the message really is still
        // there. Filing it here would hide a second copy that exists, and it
        // would take this uid out of what the cache holds, so the next sync
        // would fetch the original again as new mail and file a third copy.
        assert_eq!(
            cache
                .get_message_list_sorted(inbox, "acct", None, Some(50))
                .expect("the inbox reads")
                .len(),
            1,
            "the copy still in the inbox at the server is not listed here"
        );
        assert_eq!(
            cache
                .get_message_list_sorted(invoices, "acct", None, Some(50))
                .expect("the folder reads")
                .len(),
            0,
            "the message was filed here as though the move had happened"
        );
    }

    #[test]
    fn test_no_sentence_about_filing_carries_a_subject_line() {
        // A subject is close enough to the message to be held to the same rule
        // as its body, and every one of these sentences goes to the log.
        let (_dir, cache, inbox, _invoices, folder, engine) =
            an_account_with_rules(&[("move_to_folder", Some("A folder nobody has"))]);

        let done = sync_with_rules(
            &a_server_holding_one_invoice(),
            &cache,
            &folder,
            inbox,
            &engine,
        );

        assert_eq!(done.filtered.could_not_be_filed.len(), 1);
        for reason in &done.filtered.could_not_be_filed {
            assert!(
                !reason.contains("4021") && !reason.contains("Invoice #"),
                "a sentence that goes to the log named the message: {reason}"
            );
        }
    }

    #[test]
    fn test_a_message_a_rule_marks_read_and_files_is_counted_once() {
        // Two halves count: `apply_rules` writes the flags on this computer,
        // and `carry_out_the_moves` reaches the server, and each of them added
        // one to the same total. So one message matching a rule that marks it
        // read and a rule that files it was reported as two messages sorted,
        // and there was no test at this seam to notice.
        let (_dir, cache, inbox, _invoices, folder, engine) =
            an_account_with_rules(&[("mark_as_read", None), ("move_to_folder", Some("Invoices"))]);

        let done = sync_with_rules(
            &a_server_holding_one_invoice(),
            &cache,
            &folder,
            inbox,
            &engine,
        );

        assert_eq!(
            done.filtered.changed, 1,
            "one message was counted once for each half of what its rules asked for"
        );
    }

    #[test]
    fn test_moves_that_never_got_as_far_as_the_server_are_not_passed_over() {
        // Two ways this gives up before it starts: the folder's account
        // cannot be read, and that account's folder list cannot be. Both
        // handed back nothing at all, so every move the rules asked for was
        // dropped and the sync read exactly like a sync with no rules in it.
        let (_dir, cache, _inbox, _invoices, folder, _engine) = an_account_with_rules(&[]);
        let a_folder_that_is_not_there = 9_999;

        let (filed, could_not) = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
            .block_on(carry_out_the_moves(
                &Scripted::default(),
                &cache,
                &folder,
                a_folder_that_is_not_there,
                &[Moving {
                    message_row: 1,
                    uid: 7,
                    into: "Invoices".to_string(),
                }],
            ));

        assert_eq!(filed, 0, "a move was counted where none was attempted");
        assert_eq!(
            could_not.len(),
            1,
            "a move that never got as far as the server was passed over in silence"
        );
    }

    #[test]
    fn test_a_rule_never_files_a_message_into_the_folder_it_is_already_in() {
        // Rules run on whatever has just arrived in whichever folder is being
        // checked, so a rule that files a sender into Invoices goes on
        // matching that sender once their mail is in Invoices. Asking a server
        // to move a message into the folder it is already in either fails,
        // once per message on every check for mail, or takes a copy, and
        // nothing anywhere removes duplicates.
        //
        // Blocking writes exactly this rule, and blocking now switches the
        // junk folder on, so this is no longer a folder nobody downloads: mail
        // filed into Junk comes back down on the next check as newly arrived
        // there, and the block matches it again.
        let (_dir, cache, _inbox, invoices, _folder, engine) =
            an_account_with_rules(&[("move_to_folder", Some("Invoices"))]);
        let its_own_folder = ImapFolder {
            name: "Invoices".into(),
            display_path: "INBOX/Invoices".into(),
            path: "INBOX/Invoices".into(),
            folder_type: FolderType::Custom,
            selectable: true,
            holds_all_mail: false,
            subscribed: true,
            delimiter: None,
        };

        let server = a_server_holding_one_invoice();
        let done = sync_with_rules(&server, &cache, &its_own_folder, invoices, &engine);

        assert_eq!(
            done.filtered.to_move.len(),
            1,
            "the rule did not match the message, so this test proves nothing"
        );
        assert!(
            server.moved.borrow().is_empty(),
            "the server was asked to move a message into the folder it is in: {:?}",
            server.moved.borrow()
        );
        assert_eq!(
            done.filtered.changed, 0,
            "a message already in the right folder was counted as sorted again"
        );
        assert!(
            done.filtered.could_not_be_filed.is_empty(),
            "a message already where the rule wants it was reported as a problem: {:?}",
            done.filtered.could_not_be_filed
        );
    }

    #[test]
    fn test_the_folder_list_is_stored_with_the_facts_only_the_server_knows() {
        // Storing the list is not just writing names. It hands back the id the
        // cache gave each row, which is what every later sync of that folder is
        // keyed on, and it keeps the two answers only the server has: whether a
        // folder holds every message, and whether anybody subscribed to it.
        // Losing either turns the window that asks which folders to sync into a
        // window offering a different default from the one the sync uses.
        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
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

    /// A mailbox the way a server separating with the given character lists
    /// it: the leaf as the name, the whole path as the path.
    fn nested(path: &str, separator: Option<&str>) -> ImapFolder {
        let leaf = match separator.filter(|sep| !sep.is_empty()) {
            Some(sep) => path.rsplit(sep).next().unwrap_or(path),
            None => path,
        };
        ImapFolder {
            name: leaf.to_string(),
            display_path: path.to_string(),
            path: path.to_string(),
            folder_type: FolderType::Custom,
            selectable: true,
            holds_all_mail: false,
            subscribed: true,
            delimiter: separator.map(str::to_string),
        }
    }

    fn a_fresh_cache() -> (tempfile::TempDir, MessageCache) {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
        (dir, cache)
    }

    fn parent_of(cache: &MessageCache, account: &str, path: &str) -> Option<Option<i64>> {
        cache
            .folder_parents(account)
            .expect("the parents")
            .get(path)
            .copied()
    }

    fn id_of(stored: &[(ImapFolder, i64)], path: &str) -> i64 {
        stored
            .iter()
            .find(|(f, _)| f.path == path)
            .map(|(_, id)| *id)
            .unwrap_or_else(|| panic!("no row was stored for {path}"))
    }

    #[test]
    fn test_a_folder_is_linked_to_the_folder_whose_name_its_path_carries() {
        // The whole of what this pass is for. The path is split once here,
        // where the separator the server gave for that mailbox is in hand, and
        // the tree afterwards reads a parent instead of splitting anything.
        let (_dir, cache) = a_fresh_cache();
        let listed = vec![
            nested("Archive", Some("/")),
            nested("Archive/2026", Some("/")),
        ];

        let stored = store_folders(&cache, "acct", &listed).expect("the list is stored");

        assert_eq!(
            parent_of(&cache, "acct", "Archive/2026"),
            Some(Some(id_of(&stored, "Archive")))
        );
        assert_eq!(parent_of(&cache, "acct", "Archive"), Some(None));
    }

    #[test]
    fn test_the_stored_name_is_the_leaf_and_not_the_whole_path() {
        // The tree nests, so a row says what the folder is called and its
        // place says where it sits. Storing "Archive/2026" as the name would
        // spell the hierarchy into the label, which the tree control announces
        // and a label must never carry.
        let (_dir, cache) = a_fresh_cache();
        let listed = vec![
            nested("Archive", Some("/")),
            nested("Archive/2026", Some("/")),
        ];

        store_folders(&cache, "acct", &listed).expect("the list is stored");

        let rows = cache.get_folders_for_account("acct").expect("the rows");
        let name_of = |path: &str| {
            rows.iter()
                .find(|r| r.path == path)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| panic!("the row for {path}"))
        };
        assert_eq!(name_of("Archive/2026"), "2026");
        assert_eq!(name_of("Archive"), "Archive");
    }

    #[test]
    fn test_a_folder_whose_parent_the_server_did_not_list_stays_where_it_is() {
        // A server can list a child without its parent, and a subscription
        // list is one ordinary way that happens. Inventing the missing row
        // would put a folder in the tree that nothing can open; dropping the
        // child would hide mail. It stays, at the top level.
        let (_dir, cache) = a_fresh_cache();
        let listed = vec![nested("Archive/2026", Some("/"))];

        let stored = store_folders(&cache, "acct", &listed).expect("the list is stored");

        assert_eq!(stored.len(), 1, "the child was dropped");
        assert_eq!(parent_of(&cache, "acct", "Archive/2026"), Some(None));
        assert_eq!(
            cache.folder_parents("acct").expect("the parents").len(),
            1,
            "a row was invented for the parent the server did not list"
        );
    }

    /// A stored folder, as much of one as the comparison reads.
    ///
    /// The account and the path are what decide whether a folder is even the
    /// server's to have an opinion about, and the id is what comes back.
    fn stored_row(id: i64, account_id: &str, path: &str) -> CachedFolder {
        CachedFolder {
            id,
            account_id: account_id.to_string(),
            name: path.rsplit('/').next().unwrap_or(path).to_string(),
            path: path.to_string(),
            folder_type: "Custom".to_string(),
            unread_count: 0,
            total_count: 0,
        }
    }

    #[test]
    fn test_a_stored_folder_the_list_no_longer_holds_is_reported_and_one_it_holds_is_not() {
        // The ordinary case, and the one that says the comparison discriminates
        // at all: two folders, one in the answer and one not.
        let stored = vec![
            stored_row(1, "acct", "INBOX"),
            stored_row(2, "acct", "Archive"),
        ];
        let listed = vec!["INBOX".to_string()];

        assert_eq!(
            folders_the_server_no_longer_lists(&stored, &listed),
            vec![2],
            "the folder missing from the answer was not reported, or the one in it was"
        );
    }

    #[test]
    fn test_a_folder_that_came_back_stops_being_reported() {
        // A folder can be missing from one answer and present in the next: a
        // server moving mailboxes, a partial LSUB, somebody re-subscribing. The
        // comparison is against this answer and carries nothing over from the
        // last one.
        let stored = vec![stored_row(2, "acct", "Archive")];

        assert_eq!(
            folders_the_server_no_longer_lists(&stored, &["INBOX".to_string()]),
            vec![2],
            "the folder absent from the answer was not reported"
        );
        assert!(
            folders_the_server_no_longer_lists(
                &stored,
                &["INBOX".to_string(), "Archive".to_string()]
            )
            .is_empty(),
            "the folder came back and was still reported gone"
        );
    }

    #[test]
    fn test_a_folder_kept_on_this_computer_is_never_reported() {
        // The test that matters most here. The server has never seen a folder
        // kept on this computer, so its silence about one says nothing, and
        // reading that silence as a deletion would offer to destroy somebody's
        // Drafts because their mail server did not mention them.
        let local = crate::application::local_folders::SHARED_BY_EVERY_ACCOUNT[2].path();
        let stored = vec![
            stored_row(1, "acct", &local),
            stored_row(2, "acct", "Archive"),
        ];
        let listed = vec!["INBOX".to_string()];

        let gone = folders_the_server_no_longer_lists(&stored, &listed);
        assert!(
            gone.contains(&2),
            "the server's own folder was not reported, so this fixture proves nothing"
        );
        assert!(
            !gone.contains(&1),
            "a folder kept on this computer was reported as deleted by the server"
        );
    }

    #[test]
    fn test_a_folder_owned_by_the_reserved_account_is_never_reported() {
        // The other half of the same rule, and a separate one: the shared five
        // are filtered by their path, and this is filtered by their owner. A
        // row under the reserved id whose path is an ordinary one proves the
        // owner is read rather than only the path.
        let stored = vec![
            stored_row(1, crate::application::local_folders::THIS_COMPUTER, "Sent"),
            stored_row(2, "acct", "Sent"),
        ];
        let listed = vec!["INBOX".to_string()];

        let gone = folders_the_server_no_longer_lists(&stored, &listed);
        assert!(
            gone.contains(&2),
            "the account's own folder was not reported, so this fixture proves nothing"
        );
        assert!(
            !gone.contains(&1),
            "a folder owned by the reserved this-computer account was reported gone"
        );
    }

    #[test]
    fn test_an_answer_holding_nothing_reports_nothing() {
        // An empty LIST and a mailbox whose every folder has been deleted are
        // the same answer on the wire. Reported, this would ask somebody
        // whether to delete their whole mailbox because a connection dropped.
        // The pairing is what stops this passing against a function that
        // reports nothing at all: the same stored rows, against an answer that
        // really is missing them, are both reported.
        let stored = vec![
            stored_row(1, "acct", "INBOX"),
            stored_row(2, "acct", "Archive"),
        ];

        assert!(
            folders_the_server_no_longer_lists(&stored, &[]).is_empty(),
            "an empty answer was read as every folder having been deleted"
        );
        assert_eq!(
            folders_the_server_no_longer_lists(&stored, &["Junk".to_string()]).len(),
            2,
            "an answer that really is missing both did not report them"
        );
    }

    #[test]
    fn test_a_sync_does_not_overwrite_an_answer_somebody_has_already_given() {
        // The rule that keeps the question from coming back at every launch
        // about a folder somebody has already said to keep. The undecided row
        // beside it is what says the folder is still marked at all, so a
        // function that answered "keep it" to everything would fail that half.
        use WhatTheServerSaid::*;

        assert_eq!(
            what_the_server_now_says(ItStoppedListingItAndSomebodySaidKeepIt, false),
            ItStoppedListingItAndSomebodySaidKeepIt,
            "a sync put an answered folder back to being a question"
        );
        assert_eq!(
            what_the_server_now_says(ItStoppedListingIt, false),
            ItStoppedListingIt,
            "a folder still missing stopped being marked"
        );
        assert_eq!(
            what_the_server_now_says(ItListedIt, false),
            ItStoppedListingIt,
            "a folder that has just gone missing was not marked"
        );
    }

    #[test]
    fn test_a_folder_the_server_lists_again_is_plainly_listed_whatever_was_said_about_it() {
        // A decision about a folder that had gone has nothing left to be about
        // once the folder is back. Left in place it would also mean a folder
        // that vanished, was kept, and vanished again could never be asked
        // about a second time.
        use WhatTheServerSaid::*;

        for before in [
            ItListedIt,
            ItStoppedListingIt,
            ItStoppedListingItAndSomebodySaidKeepIt,
        ] {
            assert_eq!(
                what_the_server_now_says(before, true),
                ItListedIt,
                "a folder the server lists again did not read as listed, from {before:?}"
            );
        }
    }

    #[test]
    fn test_a_mailbox_that_is_only_a_name_in_the_hierarchy_can_still_be_a_parent() {
        // A non-selectable mailbox holds no mail and exists so the name does.
        // Those rows are exactly the branches this tree hangs children from,
        // so refusing them as parents would flatten every hierarchy that has
        // one.
        let (_dir, cache) = a_fresh_cache();
        let mut container = nested("Archive", Some("/"));
        container.selectable = false;
        let listed = vec![container, nested("Archive/2026", Some("/"))];

        let stored = store_folders(&cache, "acct", &listed).expect("the list is stored");

        assert_eq!(
            parent_of(&cache, "acct", "Archive/2026"),
            Some(Some(id_of(&stored, "Archive")))
        );
    }

    #[test]
    fn test_two_accounts_holding_the_same_path_each_link_inside_their_own() {
        // A parent looked up across the whole table would hang one account's
        // mail under another account's branch, which shows somebody the wrong
        // person's folder full of the wrong person's messages.
        let (_dir, cache) = a_fresh_cache();
        let listed = vec![
            nested("Archive", Some("/")),
            nested("Archive/2026", Some("/")),
        ];

        let mine = store_folders(&cache, "mine", &listed).expect("mine stored");
        let theirs = store_folders(&cache, "theirs", &listed).expect("theirs stored");

        assert_ne!(
            id_of(&mine, "Archive"),
            id_of(&theirs, "Archive"),
            "one row served both accounts"
        );
        assert_eq!(
            parent_of(&cache, "mine", "Archive/2026"),
            Some(Some(id_of(&mine, "Archive")))
        );
        assert_eq!(
            parent_of(&cache, "theirs", "Archive/2026"),
            Some(Some(id_of(&theirs, "Archive")))
        );
    }

    #[test]
    fn test_storing_the_same_folder_list_again_changes_nothing() {
        // The folder list is saved on every check for mail, so this pass runs
        // over and over on an account nothing has happened to. It has to
        // settle rather than drift.
        let (_dir, cache) = a_fresh_cache();
        let listed = vec![
            nested("Archive", Some("/")),
            nested("Archive/2026", Some("/")),
        ];

        store_folders(&cache, "acct", &listed).expect("the first time");
        let first = cache.folder_parents("acct").expect("the parents");
        store_folders(&cache, "acct", &listed).expect("the second time");
        let again = cache.folder_parents("acct").expect("the parents");

        assert_eq!(first, again);
        assert_eq!(
            cache
                .get_folders_for_account("acct")
                .expect("the rows")
                .len(),
            2,
            "storing the same list twice made new rows"
        );
    }

    #[test]
    fn test_a_separator_that_separates_nothing_splits_nothing() {
        // Both answers come off the wire, so both are a stranger's. A mailbox
        // the server gave no separator for and one it gave an empty separator
        // for are the same case, and neither names a parent however many
        // slashes are in the name.
        let (_dir, cache) = a_fresh_cache();
        let listed = vec![nested("Work/2026", None), nested("Notes", Some(""))];

        store_folders(&cache, "acct", &listed).expect("the list is stored");

        assert_eq!(parent_of(&cache, "acct", "Work/2026"), Some(None));
        assert_eq!(parent_of(&cache, "acct", "Notes"), Some(None));
        let rows = cache.get_folders_for_account("acct").expect("the rows");
        assert_eq!(
            rows.iter()
                .find(|r| r.path == "Work/2026")
                .map(|r| r.name.as_str()),
            Some("Work/2026"),
            "a name was split on a separator the server did not give"
        );
    }

    #[test]
    fn test_a_separator_of_several_characters_is_used_whole() {
        // Nothing in the protocol promises one character, and a reader that
        // took the last character of it would split on a fragment.
        let (_dir, cache) = a_fresh_cache();
        let listed = vec![
            nested("Archive", Some("::")),
            nested("Archive::2026", Some("::")),
        ];

        let stored = store_folders(&cache, "acct", &listed).expect("the list is stored");

        assert_eq!(
            parent_of(&cache, "acct", "Archive::2026"),
            Some(Some(id_of(&stored, "Archive")))
        );
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
        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
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

    /// Fill a folder with `held` messages the server also lists, so a later
    /// sync of the same folder has something to discard.
    ///
    /// Numbered from one, and the same numbers are put back on the server
    /// afterwards under a new numbering: that is the case the discard exists
    /// for, since a uid the server still lists is one nothing else would
    /// notice had changed hands.
    fn a_folder_already_holding(
        cache: &MessageCache,
        id: i64,
        folder: &ImapFolder,
        held: u32,
        numbering: u32,
    ) {
        let uids: Vec<u32> = (1..=held).collect();
        run(
            &Scripted {
                on_server: uids.clone(),
                headers: uids.iter().copied().map(message).collect(),
                uid_validity: Some(numbering),
                ..Default::default()
            },
            cache,
            id,
            folder,
        );
    }

    /// A folder synced once against a server that can answer "what changed
    /// since", which is what leaves a stored modification sequence behind.
    ///
    /// [`a_folder_already_holding`] is not that folder: its server offers no
    /// HIGHESTMODSEQ, so nothing is stored and a resume has nothing to start
    /// from. The two fixtures differ by exactly the fact this decision turns
    /// on, which is why there are two of them.
    fn a_folder_synced_against_a_server_that_answers_what_changed(
        cache: &MessageCache,
        id: i64,
        folder: &ImapFolder,
        held: u32,
        numbering: u32,
    ) {
        let uids: Vec<u32> = (1..=held).collect();
        run(
            &Scripted {
                on_server: uids.clone(),
                headers: uids.iter().copied().map(message).collect(),
                counts: crate::service::protocols::imap::FolderCounts {
                    total: held,
                    unread: 0,
                },
                uid_validity: Some(numbering),
                highest_modseq: Some(100),
                ..Default::default()
            },
            cache,
            id,
            folder,
        );
    }

    /// Everything a scripted server was asked, in order.
    fn what_it_was_asked(server: &Scripted) -> Vec<String> {
        server.happened.borrow().clone()
    }

    #[test]
    fn test_a_folder_with_everything_stored_resumes_from_the_highest_uid_held() {
        // SCALE-01. Four facts line up, so the sync asks what arrived instead
        // of asking a forty thousand message mailbox for forty thousand
        // numbers to find the three that did.
        assert_eq!(
            what_to_ask_for(
                WhatThisComputerHolds {
                    uid_validity: Some(7),
                    modseq: Some(4200),
                    highest_uid: Some(39_998),
                },
                WhatTheServerReports {
                    uid_validity: Some(7),
                    highest_modseq: Some(4300),
                },
            ),
            WhatToAskFor::TheUidsAbove(39_998)
        );
    }

    #[test]
    fn test_a_folder_this_computer_has_never_synced_is_read_out_in_full() {
        // No stored numbering, because nothing has ever been stored. The uids
        // held mean nothing to resume from and there are none anyway.
        assert_eq!(
            what_to_ask_for(
                WhatThisComputerHolds::default(),
                WhatTheServerReports {
                    uid_validity: Some(7),
                    highest_modseq: Some(4300),
                },
            ),
            WhatToAskFor::EveryUid
        );
    }

    #[test]
    fn test_a_numbering_that_changed_is_read_out_in_full() {
        // The uids held name different messages now, or none. Resuming from a
        // number under the old numbering starts at the wrong place, and
        // everything below it stays wrong until somebody notices.
        assert_eq!(
            what_to_ask_for(
                WhatThisComputerHolds {
                    uid_validity: Some(7),
                    modseq: Some(4200),
                    highest_uid: Some(500),
                },
                WhatTheServerReports {
                    uid_validity: Some(8),
                    highest_modseq: Some(4300),
                },
            ),
            WhatToAskFor::EveryUid
        );
    }

    #[test]
    fn test_a_server_reporting_no_modification_sequence_is_read_out_in_full() {
        // Every sync against Gmail, which has never advertised CONDSTORE. The
        // flag read has nothing to resume from, so neither has the listing, and
        // saying so here is what keeps a Gmail account correct rather than fast
        // and wrong.
        assert_eq!(
            what_to_ask_for(
                WhatThisComputerHolds {
                    uid_validity: Some(7),
                    modseq: Some(4200),
                    highest_uid: Some(500),
                },
                WhatTheServerReports {
                    uid_validity: Some(7),
                    highest_modseq: None,
                },
            ),
            WhatToAskFor::EveryUid
        );
    }

    #[test]
    fn test_a_folder_with_nothing_in_it_is_read_out_in_full() {
        // Nowhere to resume from: the narrow question starts at the highest uid
        // held and there is none. Listing it is free, since it is empty.
        assert_eq!(
            what_to_ask_for(
                WhatThisComputerHolds {
                    uid_validity: Some(7),
                    modseq: Some(4200),
                    highest_uid: None,
                },
                WhatTheServerReports {
                    uid_validity: Some(7),
                    highest_modseq: Some(4300),
                },
            ),
            WhatToAskFor::EveryUid
        );
    }

    #[test]
    fn test_a_folder_that_was_synced_before_is_not_listed_again() {
        // The requirement itself, asserted against a server that wrote down
        // what it was asked rather than against a number this program keeps.
        let (cache, id, folder) = a_cache();
        a_folder_synced_against_a_server_that_answers_what_changed(&cache, id, &folder, 3, 1);

        let server = Scripted {
            on_server: vec![1, 2, 3],
            counts: crate::service::protocols::imap::FolderCounts {
                total: 3,
                unread: 0,
            },
            uid_validity: Some(1),
            highest_modseq: Some(200),
            ..Default::default()
        };
        run(&server, &cache, id, &folder);

        let asked = what_it_was_asked(&server);
        assert!(
            !asked.iter().any(|line| line == LISTED_EVERY_UID),
            "a folder that had been synced before was listed in full again: {asked:?}"
        );
        assert!(
            asked
                .iter()
                .any(|line| line == &format!("{LISTED_THE_UIDS_ABOVE}3")),
            "the sync did not ask what arrived above the highest uid held: {asked:?}"
        );
    }

    #[test]
    fn test_a_resumed_folder_still_gains_the_messages_that_arrived_since() {
        // The half that makes the saving worth having. A resume that asks a
        // cheaper question and brings nothing down is a folder that stops
        // updating, which is worse than the listing it replaced.
        let (cache, id, folder) = a_cache();
        a_folder_synced_against_a_server_that_answers_what_changed(&cache, id, &folder, 3, 1);

        let done = run(
            &Scripted {
                on_server: vec![1, 2, 3, 4, 5],
                headers: vec![message(4), message(5)],
                counts: crate::service::protocols::imap::FolderCounts {
                    total: 5,
                    unread: 0,
                },
                uid_validity: Some(1),
                highest_modseq: Some(200),
                ..Default::default()
            },
            &cache,
            id,
            &folder,
        );

        assert_eq!(done.fetched, 2, "the two that arrived did not come down");
        assert_eq!(
            cache.stored_uids(id).expect("what is held"),
            vec![1, 2, 3, 4, 5]
        );
        assert_eq!(
            done.total_on_server, 5,
            "the folder's total was read off the narrow listing rather than off the count"
        );
    }

    #[test]
    fn test_a_folder_never_synced_before_is_listed_in_full() {
        // The first sync after an account is added, for every folder in it.
        let (cache, id, folder) = a_cache();

        let server = Scripted {
            on_server: vec![1, 2],
            headers: vec![message(1), message(2)],
            counts: crate::service::protocols::imap::FolderCounts {
                total: 2,
                unread: 0,
            },
            uid_validity: Some(1),
            highest_modseq: Some(200),
            ..Default::default()
        };
        run(&server, &cache, id, &folder);

        assert!(
            what_it_was_asked(&server)
                .iter()
                .any(|line| line == LISTED_EVERY_UID),
            "a folder nothing had ever synced was resumed rather than read"
        );
    }

    #[test]
    fn test_a_folder_the_server_renumbered_is_listed_in_full() {
        // The uids held mean nothing now, and plan 03-01's discard has just
        // emptied the folder, so there is a whole mailbox to read back.
        let (cache, id, folder) = a_cache();
        a_folder_synced_against_a_server_that_answers_what_changed(&cache, id, &folder, 3, 1);

        let server = Scripted {
            on_server: vec![1, 2, 3],
            headers: vec![message(1), message(2), message(3)],
            counts: crate::service::protocols::imap::FolderCounts {
                total: 3,
                unread: 0,
            },
            uid_validity: Some(2),
            highest_modseq: Some(200),
            ..Default::default()
        };
        let done = run(&server, &cache, id, &folder);

        assert!(done.renumbered, "the fixture did not renumber the folder");
        assert!(
            what_it_was_asked(&server)
                .iter()
                .any(|line| line == LISTED_EVERY_UID),
            "a renumbered folder was resumed from numbers that no longer mean anything"
        );
    }

    #[test]
    fn test_a_server_offering_no_modification_sequence_lists_the_folder_in_full() {
        // Gmail, and every server without CONDSTORE. The folder was synced
        // before and there is still nothing to resume from.
        let (cache, id, folder) = a_cache();
        a_folder_synced_against_a_server_that_answers_what_changed(&cache, id, &folder, 3, 1);

        let server = Scripted {
            on_server: vec![1, 2, 3],
            counts: crate::service::protocols::imap::FolderCounts {
                total: 3,
                unread: 0,
            },
            uid_validity: Some(1),
            highest_modseq: None,
            ..Default::default()
        };
        run(&server, &cache, id, &folder);

        assert!(
            what_it_was_asked(&server)
                .iter()
                .any(|line| line == LISTED_EVERY_UID),
            "a server that reported no modification sequence was resumed from anyway"
        );
    }

    #[test]
    fn test_nothing_is_forgotten_when_a_folder_resumes() {
        // The hazard the research document names, asserted rather than argued.
        // A resume asks about part of a folder, and part of a folder compared
        // against what this computer holds says every message outside it has
        // gone. `ServerListing::PartOfIt` is what stops that, and this is the
        // assertion that would notice if it stopped.
        let (cache, id, folder) = a_cache();
        a_folder_synced_against_a_server_that_answers_what_changed(&cache, id, &folder, 4, 1);
        let before = cache.stored_uids(id).expect("what is held").len();

        let done = run(
            &Scripted {
                on_server: vec![1, 2, 3, 4, 5],
                headers: vec![message(5)],
                counts: crate::service::protocols::imap::FolderCounts {
                    total: 5,
                    unread: 0,
                },
                uid_validity: Some(1),
                highest_modseq: Some(200),
                ..Default::default()
            },
            &cache,
            id,
            &folder,
        );

        assert_eq!(done.forgotten, 0, "a resume forgot something");
        assert_eq!(
            cache.stored_uids(id).expect("what is held").len(),
            before + 1,
            "a resume that added one message did not leave the other four alone"
        );
    }

    #[test]
    fn test_a_renumbered_folder_says_what_it_discarded() {
        // Criterion 1. Before this the discard reached `tracing::info!` and a
        // clause in the folder's summary line, which is announced at Low under
        // the topic every steady sync line shares, so the next "Checking..."
        // replaced it where it stood. Neither said how much went.
        let (cache, id, folder) = a_cache();
        a_folder_already_holding(&cache, id, &folder, 2, 1);

        let done = run(
            &Scripted {
                on_server: vec![1, 2],
                headers: vec![message(1), message(2)],
                uid_validity: Some(2),
                ..Default::default()
            },
            &cache,
            id,
            &folder,
        );

        assert!(done.renumbered);
        assert_eq!(
            done.discarded_after_renumbering, 2,
            "the count the discard itself answered was thrown away"
        );
        assert_eq!(
            what_the_renumbering_discarded(&done).as_deref(),
            Some(
                "The mail server gave Inbox new numbers, so what this computer held for it \
                 no longer matches. It has been discarded and is being read again: 2 messages."
            )
        );
    }

    #[test]
    fn test_a_server_that_gives_no_numbering_at_all_discards_nothing() {
        // Absent is not changed. A server that answers no UIDVALIDITY has said
        // nothing about whether it renumbered, and reading silence as a change
        // empties every folder on every sync against such a server. This is
        // the arm with the most to lose and it had no test.
        let (cache, id, folder) = a_cache();
        a_folder_already_holding(&cache, id, &folder, 2, 1);

        let done = run(
            &Scripted {
                on_server: vec![1, 2],
                // Nothing offered, so anything still held has to have survived
                // rather than been fetched back.
                headers: Vec::new(),
                uid_validity: None,
                ..Default::default()
            },
            &cache,
            id,
            &folder,
        );

        assert!(!done.renumbered, "silence was read as a renumbering");
        assert_eq!(done.discarded_after_renumbering, 0);
        assert_eq!(what_the_renumbering_discarded(&done), None);
        assert_eq!(
            cache.stored_uids(id).expect("what is held"),
            vec![1, 2],
            "the mail this computer held was thrown away on a server that said nothing"
        );
    }

    #[test]
    fn test_a_folder_read_for_the_first_time_discards_nothing() {
        // Every folder, on the sync that follows adding an account. There is
        // no stored numbering to differ from, so there is nothing to discard
        // and nothing to say about it.
        let (cache, id, folder) = a_cache();

        let done = run(
            &Scripted {
                on_server: vec![1],
                headers: vec![message(1)],
                uid_validity: Some(9),
                ..Default::default()
            },
            &cache,
            id,
            &folder,
        );

        assert!(!done.renumbered, "a first sync was read as a renumbering");
        assert_eq!(done.discarded_after_renumbering, 0);
        assert_eq!(what_the_renumbering_discarded(&done), None);
        assert_eq!(done.fetched, 1);
    }

    #[test]
    fn test_a_folder_the_server_numbered_the_same_way_discards_nothing() {
        // The ordinary sync, which is every sync but the two above. Kept
        // beside them because the four arms are one decision and a reader
        // checking it wants all four in one place.
        let (cache, id, folder) = a_cache();
        a_folder_already_holding(&cache, id, &folder, 2, 7);

        let done = run(
            &Scripted {
                on_server: vec![1, 2],
                headers: Vec::new(),
                uid_validity: Some(7),
                ..Default::default()
            },
            &cache,
            id,
            &folder,
        );

        assert!(!done.renumbered);
        assert_eq!(done.discarded_after_renumbering, 0);
        assert_eq!(what_the_renumbering_discarded(&done), None);
        assert_eq!(
            cache.stored_uids(id).expect("what is held"),
            vec![1, 2],
            "an unchanged numbering threw mail away"
        );
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

    #[test]
    fn test_a_sync_over_a_message_with_an_attachment_stores_no_attachment_and_no_file() {
        // SCALE-04's third deliverable: opening a folder must not fetch
        // anything an attachment is made of. That is bandwidth and disk
        // somebody did not agree to spend, on a folder they may only be
        // glancing at, and the files can be tens of megabytes each.
        //
        // What a sync can know about an attachment turns out to be one bit.
        // `ImapMessage` carries `has_attachments` and no list, because a header
        // fetch does not read a message's structure, so a sync cannot write an
        // attachment's name or its size either. The only thing that writes
        // either is `replace_attachments_with_content`, whose one production
        // caller is the reader, on a message somebody has opened.
        let (cache, id, folder) = a_cache();
        let server = Scripted {
            on_server: vec![1],
            headers: vec![ImapMessage {
                has_attachments: true,
                ..message(1)
            }],
            counts: crate::service::protocols::imap::FolderCounts {
                total: 1,
                unread: 0,
            },
            ..Default::default()
        };

        let done = run(&server, &cache, id, &folder);
        assert_eq!(done.fetched, 1, "the sync did not bring the message down");

        let listed = cache
            .get_message_list(id, "acct")
            .expect("the folder lists");
        let row = listed.first().expect("the message the sync brought down");

        let described = cache
            .get_attachments_for_message(row.id)
            .expect("the attachments read back");
        assert!(
            described.is_empty(),
            "a sync wrote {} attachment rows for a message nobody has opened: {described:#?}",
            described.len()
        );

        let files = cache
            .cached_attachment_bytes()
            .expect("the stored files are measured");
        assert_eq!(
            files, 0,
            "a sync put {files} bytes of attachment file on this computer for a \
             message nobody has opened, which is a download nobody asked for"
        );

        // And the one bit it does know is kept, so the row can say there is an
        // attachment without anything having been fetched.
        assert!(
            row.has_attachments,
            "the sync threw away the one thing it does know about the \
             attachment, so the row cannot say there is one"
        );
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

    /// A mail server that will not take a copy of a sent message.
    ///
    /// Which is what puts a locally written row into a folder a sync
    /// reconciles, and is the whole reason the marker exists.
    struct WillNotFileACopy;

    impl crate::application::sent_copy::FilesACopy for WillNotFileACopy {
        async fn keep_a_copy(&self, _folder: &str, _raw: &[u8]) -> std::result::Result<(), String> {
            Err("the mailbox is over quota".to_string())
        }
    }

    /// A Sent folder holding a copy of one message that was kept here, plus
    /// however many the server has in it.
    fn a_sent_folder_with_a_copy_kept_here() -> (TempHome<MessageCache>, i64, ImapFolder, i64) {
        let cache = TempHome::named("wixen_sent_sync_", |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache")
        });
        let folder_id = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acct".into(),
                name: "Sent".into(),
                path: "Sent".into(),
                folder_type: "Sent".into(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder");
        let mut account =
            crate::data::account::Account::new("Work".into(), "me@example.com".into());
        account.id = "acct".to_string();
        let raw = concat!(
            "From: me@example.com\r\n",
            "To: you@example.com\r\n",
            "Subject: What I sent\r\n",
            "Date: Tue, 4 Aug 2026 10:00:00 +0000\r\n",
            "Message-ID: <mine-1@example.com>\r\n",
            "\r\n",
            "The only copy of this.\r\n",
        )
        .as_bytes();
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
            .block_on(async {
                let goes_to = crate::application::sent_copy::destination(&cache, &account);
                let said = crate::application::sent_copy::offer_to_the_server(
                    &WillNotFileACopy,
                    &goes_to,
                    raw,
                )
                .await;
                crate::application::sent_copy::file_the_copy(
                    &cache, &account, &goes_to, &said, false, raw,
                )
            });
        let kept = cache
            .get_message_list(folder_id, "acct")
            .expect("list the folder")
            .into_iter()
            .find(|row| row.subject == "What I sent")
            .expect("the copy the server refused");
        let folder = ImapFolder {
            name: "Sent".into(),
            display_path: "Sent".into(),
            path: "Sent".into(),
            folder_type: FolderType::Sent,
            selectable: true,
            holds_all_mail: false,
            subscribed: true,
            delimiter: None,
        };
        (cache, folder_id, folder, kept.id)
    }

    #[test]
    fn test_a_copy_kept_here_survives_a_sync_that_finds_it_is_not_on_the_server() {
        // #112 through the whole sync rather than at the statement. The server
        // has never heard of this message, so the forget step reads it as one
        // the server no longer has and deletes the row. The body goes with it,
        // through the foreign key, and it was the only copy.
        let (cache, folder_id, folder, copy) = a_sent_folder_with_a_copy_kept_here();
        let server = Scripted {
            on_server: vec![1, 2],
            headers: vec![message(1), message(2)],
            ..Default::default()
        };

        let done = run(&server, &cache, folder_id, &folder);

        assert!(
            cache.get_message(copy).expect("read back").is_some(),
            "the sync deleted the only copy of a sent message"
        );
        assert!(
            cache.get_message_body(copy).expect("read back").is_some(),
            "the sync deleted the only copy of the message text"
        );
        assert_eq!(done.fetched, 2, "the server's own messages did not arrive");
    }

    #[test]
    fn test_a_sync_does_not_report_forgetting_a_copy_it_kept() {
        // A count that lies is the same defect as a status line that lies. The
        // copy is on neither side of the comparison a sync makes, so naming it
        // means reporting a deletion that the guard refused, and asking the
        // server for the flags of a message it has never had.
        let (cache, folder_id, folder, _) = a_sent_folder_with_a_copy_kept_here();
        let server = Scripted {
            on_server: vec![1],
            headers: vec![message(1)],
            ..Default::default()
        };

        let done = run(&server, &cache, folder_id, &folder);

        assert_eq!(
            done.forgotten, 0,
            "a deletion that never happened was counted"
        );
        assert!(
            !server.asked_for.borrow().contains(&u32::MAX),
            "the server was asked about a message it has never had"
        );
    }

    #[test]
    fn test_a_copy_kept_here_survives_the_server_renumbering_the_mailbox() {
        // A new UIDVALIDITY empties the folder, because every number held now
        // names a different message. It says nothing about the copies kept
        // here, which no server ever numbered.
        let (cache, folder_id, folder, copy) = a_sent_folder_with_a_copy_kept_here();
        run(
            &Scripted {
                on_server: vec![1],
                headers: vec![message(1)],
                uid_validity: Some(1),
                ..Default::default()
            },
            &cache,
            folder_id,
            &folder,
        );

        let after = Scripted {
            on_server: vec![1],
            headers: vec![message(1)],
            uid_validity: Some(2),
            ..Default::default()
        };
        let done = run(&after, &cache, folder_id, &folder);

        assert!(
            cache.get_message(copy).expect("read back").is_some(),
            "renumbering took the copy kept here with it"
        );
        assert!(cache.get_message_body(copy).expect("read back").is_some());
        assert_eq!(
            done.fetched, 1,
            "the server's own messages were not re-read"
        );
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
        assert_eq!(
            uids_to_forget(&ServerListing::TheWholeMailbox(vec![1, 3]), &[1, 2, 3]),
            vec![2]
        );
    }

    #[test]
    fn test_nothing_is_forgotten_when_the_server_still_has_it_all() {
        assert!(
            uids_to_forget(&ServerListing::TheWholeMailbox(vec![1, 2, 3]), &[1, 2, 3]).is_empty()
        );
    }

    #[test]
    fn test_a_listing_that_covers_part_of_a_folder_forgets_nothing() {
        // The whole point of the type. The same two arguments that make the
        // test above answer "forget uid 2" answer nothing here, because a
        // listing of part of a folder says nothing at all about the messages
        // outside it. Every uid held and not named is a message this listing
        // never asked about, not a message the server has lost.
        assert!(
            uids_to_forget(&ServerListing::PartOfIt(vec![1, 3]), &[1, 2, 3]).is_empty(),
            "a page of a folder was allowed to say which messages the server \
             no longer has"
        );
    }

    #[test]
    fn test_a_page_that_named_nothing_forgets_nothing_rather_than_everything() {
        // The case that destroys a mailbox. Ask a narrowed listing about a
        // folder whose recent messages are all held already and it can answer
        // with nothing, which compared against what is held reads as "the
        // server has none of these" and takes the lot. This is the arm plan
        // 03-07 would otherwise reach on its first sync.
        assert!(
            uids_to_forget(&ServerListing::PartOfIt(Vec::new()), &[1, 2, 3, 4]).is_empty(),
            "an empty page of a folder was read as an emptied folder"
        );
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
        assert_eq!(
            uids_to_forget(&ServerListing::TheWholeMailbox(Vec::new()), &[1, 2]),
            vec![1, 2]
        );
    }

    /// The call that forgets messages one uid at a time.
    ///
    /// Not `forget_folder_messages`, which discards a whole folder on a
    /// renumbering and answers to a different rule: that one is right to take
    /// everything, because every uid held really has stopped meaning what it
    /// meant. This is the one that decides from a comparison, and a comparison
    /// is only as good as what it was given.
    const FORGETTING_BY_UID: &str = ".forget_messages(";

    /// Every line of `source` that forgets messages by uid, with its number.
    ///
    /// The number is the line in the shipping half rather than in the file,
    /// which for this file is the same number: its one `#[cfg(test)]` sits
    /// below everything the census is about. Said because a file with a test
    /// module part way up would number from there on differently, and a line
    /// number that is nearly right is worse than none.
    fn forgetting_lines_in(source: &str) -> Vec<String> {
        crate::common::what_ships::what_ships(source)
            .lines()
            .enumerate()
            .filter(|(_, line)| line.contains(FORGETTING_BY_UID))
            .map(|(at, line)| format!("line {}: {}", at + 1, line.trim()))
            .collect()
    }

    /// Every place in the shipping half of `src/application` that forgets
    /// messages by uid.
    fn where_messages_are_forgotten_by_uid() -> Vec<String> {
        let mut found = Vec::new();
        let mut looking = vec![std::path::PathBuf::from("src/application")];
        while let Some(here) = looking.pop() {
            let Ok(entries) = std::fs::read_dir(&here) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    looking.push(path);
                    continue;
                }
                if path.extension().is_none_or(|kind| kind != "rs") {
                    continue;
                }
                let Ok(text) = std::fs::read_to_string(&path) else {
                    continue;
                };
                found.extend(
                    forgetting_lines_in(&text)
                        .into_iter()
                        .map(|at| format!("{}, {at}", path.display())),
                );
            }
        }
        found
    }

    #[test]
    fn test_one_place_in_the_application_forgets_messages_by_uid() {
        // A second one is a second chance to hand this a listing that does not
        // cover the folder, and the type only helps where somebody has to say
        // out loud what their listing covers. This is the census that makes
        // adding one a failing test rather than a thing found afterwards.
        let found = where_messages_are_forgotten_by_uid();
        assert_eq!(
            found.len(),
            1,
            "{} places forget messages by uid:\n  {}\n\
             Each one deletes somebody's cached mail on the strength of a \
             comparison, so each one needs its own argument about what the \
             listing it compares against covers.",
            found.len(),
            found.join("\n  ")
        );
    }

    #[test]
    fn test_the_census_would_see_a_second_place_that_forgets() {
        // Proving the reading before believing what it counts. A census that
        // has stopped reading passes exactly as a tree with one call does, and
        // nothing tells the two apart.
        assert_eq!(
            forgetting_lines_in(
                "    cache.forget_messages(folder_id, &gone)?;\n\
                 something else\n\
                     other.forget_messages(id, &also)?;\n"
            )
            .len(),
            2
        );
        // And it counts the right call. The whole-folder discard is a
        // different decision with a different argument behind it, and a census
        // that lumped the two together would report two here for ever and stop
        // meaning anything.
        assert!(forgetting_lines_in("    cache.forget_folder_messages(id)?;\n").is_empty());
        // And a mention inside a test module is not a call. Without this the
        // census counts its own words and can never answer one.
        assert!(
            forgetting_lines_in(
                "#[cfg(test)]\nmod tests {\n    fn a() { cache.forget_messages(1, &[]); }\n}\n"
            )
            .is_empty()
        );
        // And the walk really reaches files. Without this, a walk that found
        // nothing anywhere would be indistinguishable from a clean tree in
        // every assertion above.
        assert!(
            !where_messages_are_forgotten_by_uid().is_empty(),
            "the walk over src/application reached no call at all, so it is \
             reading nothing rather than finding nothing"
        );
    }

    #[test]
    fn test_two_answers_about_one_mailbox_that_disagree_are_not_believed() {
        // The server counted messages and then listed none of them.
        let nothing = ServerListing::TheWholeMailbox(Vec::new());
        assert!(listing_contradicts_the_count(&nothing, 1));
        assert!(listing_contradicts_the_count(&nothing, 40_000));
    }

    #[test]
    fn test_two_answers_about_one_mailbox_that_agree_are_believed() {
        // A mailbox that really is empty, and any mailbox that listed
        // something. Both are ordinary and neither may be turned into an error.
        assert!(!listing_contradicts_the_count(
            &ServerListing::TheWholeMailbox(Vec::new()),
            0
        ));
        assert!(!listing_contradicts_the_count(
            &ServerListing::TheWholeMailbox(vec![9]),
            1
        ));
        assert!(!listing_contradicts_the_count(
            &ServerListing::TheWholeMailbox(vec![9, 10]),
            0
        ));
    }

    #[test]
    fn test_a_narrow_listing_that_names_nothing_does_not_contradict_the_count() {
        // A resumed sync of a quiet mailbox, which is most syncs of most
        // folders. It asked what arrived above the highest uid held and the
        // answer is nothing, beside a count of forty thousand. Read as a
        // contradiction, that refuses the ordinary sync of every folder nothing
        // has arrived in, which is how the resume would have broken this check
        // rather than the check catching the resume.
        assert!(!listing_contradicts_the_count(
            &ServerListing::PartOfIt(Vec::new()),
            40_000
        ));
    }

    /// A folder holding two messages, bodies and all.
    fn a_folder_holding_two() -> (TempHome<MessageCache>, i64, ImapFolder, Vec<i64>) {
        let (cache, id, folder) = a_cache();
        let rows: Vec<i64> = [1_u32, 2]
            .iter()
            .map(|uid| {
                let row = cache
                    .upsert_message(&to_incoming(&message(*uid), id, false))
                    .expect("a stored message");
                cache
                    .save_message_body(row, Some("the whole message"), None)
                    .expect("a stored body");
                row
            })
            .collect();
        (cache, id, folder, rows)
    }

    #[test]
    fn test_a_server_that_counts_a_full_mailbox_and_then_lists_none_of_it_removes_nothing() {
        // The shape of the whole defect, one layer up from the protocol. A
        // server that refuses to list a mailbox answers with nothing, and
        // nothing used to mean "this mailbox is empty", so every message
        // downloaded from it was deleted here to match.
        let (cache, id, folder, rows) = a_folder_holding_two();
        let server = Scripted {
            on_server: vec![],
            counts: crate::service::protocols::imap::FolderCounts {
                total: 2,
                unread: 0,
            },
            ..Default::default()
        };

        let refused = attempt(&server, &cache, id, &folder, INITIAL_FETCH_LIMIT)
            .expect_err("two answers that disagree were believed");

        let said = refused.to_string();
        assert!(said.contains("Inbox"), "{said}");
        assert!(said.contains("2 messages"), "{said}");
        assert!(said.contains("Nothing has been removed"), "{said}");
        assert_eq!(cache.stored_uids(id).expect("held"), vec![1, 2]);
        for row in rows {
            assert!(
                cache.get_message_body(row).expect("read back").is_some(),
                "a stored message body was deleted anyway"
            );
        }
    }

    #[test]
    fn test_a_mailbox_that_really_emptied_still_has_its_rows_removed() {
        // The other direction, and what stops the check above turning into
        // "never delete anything". A mailbox somebody really emptied still has
        // to stop being listed here, or every row is a message that opens on an
        // error.
        let (cache, id, folder, _) = a_folder_holding_two();
        let server = Scripted {
            on_server: vec![],
            ..Default::default()
        };

        let done = run(&server, &cache, id, &folder);

        assert_eq!(done.forgotten, 2);
        assert!(cache.stored_uids(id).expect("held").is_empty());
    }

    #[test]
    fn test_a_folder_the_server_would_not_list_the_messages_of_keeps_the_mail_already_here() {
        // A refusal that arrives as a refusal, which is what the protocol layer
        // now does. The sync must carry it up rather than treating a folder it
        // knows nothing about as a folder with nothing in it.
        let (cache, id, folder, rows) = a_folder_holding_two();
        let server = Scripted {
            refuse_list_uids: true,
            counts: crate::service::protocols::imap::FolderCounts {
                total: 2,
                unread: 0,
            },
            ..Default::default()
        };

        attempt(&server, &cache, id, &folder, INITIAL_FETCH_LIMIT)
            .expect_err("a refusal was reported as a successful sync");

        assert_eq!(cache.stored_uids(id).expect("held"), vec![1, 2]);
        for row in rows {
            assert!(
                cache.get_message_body(row).expect("read back").is_some(),
                "a stored message body was deleted anyway"
            );
        }
    }

    #[test]
    fn test_a_quiet_folder_sync_says_only_how_much_of_it_is_here() {
        // Every other clause is worth hearing only when it happened. A line
        // that counts the nothings teaches somebody to stop listening to the
        // one that matters.
        let said = what_the_folder_sync_did(&FolderSync {
            folder: "Inbox".to_string(),
            held: 500,
            total_on_server: 500,
            ..FolderSync::default()
        });

        assert_eq!(said, "Inbox: 500 of 500 messages downloaded");
    }

    #[test]
    fn test_a_folder_holding_one_message_does_not_say_one_messages() {
        // The same shape the contacts and calendar summaries had. A folder
        // with one message in it is an ordinary folder, so this is heard.
        let said = what_the_folder_sync_did(&FolderSync {
            folder: "Archive".to_string(),
            held: 1,
            total_on_server: 1,
            ..FolderSync::default()
        });

        assert_eq!(said, "Archive: 1 of 1 message downloaded");
    }

    #[test]
    fn test_every_clause_at_once_is_still_one_list() {
        // The mail sync's parts are all counts, so this one had nothing to
        // collide: the fault the contacts, calendar and task summaries had
        // needs a clause carrying its own full stop. It is built as a list
        // here so that the first clause somebody writes as a sentence cannot
        // start it off.
        let said = what_the_folder_sync_did(&FolderSync {
            folder: "Inbox".to_string(),
            held: 500,
            total_on_server: 40_000,
            forgotten: 2,
            flags_updated: 3,
            renumbered: true,
            filtered: Filtered {
                changed: 4,
                held_back: 5,
                to_move: Vec::new(),
                could_not_be_filed: Vec::new(),
            },
            ..FolderSync::default()
        });

        assert!(!said.contains(".."), "a stop spoken twice: {said}");
        assert!(!said.contains("., "), "a fragment after a stop: {said}");
        assert!(!said.contains("  "), "a space spoken twice: {said}");
        assert_eq!(
            said,
            "Inbox: 500 of 40000 messages downloaded, Shift+F9 for older, \
             3 changed elsewhere, 2 removed elsewhere, read again after the \
             server renumbered it, 4 sorted by your rules, 5 left alone \
             because changing mail is not allowed"
        );
    }

    #[test]
    fn test_a_sync_says_when_a_rule_could_not_file_mail() {
        // Five sentences were built for the five ways filing can fail, handed
        // back, and read by nothing at all: not shown, not counted, not even
        // logged. A rule that files invoices and does not is a rule somebody
        // believes is working, and this is the only thing that would tell them
        // otherwise.
        let said = what_the_folder_sync_did(&FolderSync {
            folder: "Inbox".to_string(),
            held: 3,
            total_on_server: 3,
            filtered: Filtered {
                could_not_be_filed: vec![
                    "A rule files mail into Invoices, which this account does not have, so it \
                     was left where it is"
                        .to_string(),
                ],
                ..Filtered::default()
            },
            ..FolderSync::default()
        });

        assert!(
            said.contains("1 message not filed as asked"),
            "the sync did not say that filing failed: {said}"
        );
        assert!(
            said.contains("Invoices"),
            "the sync did not say which folder or why: {said}"
        );
    }

    #[test]
    fn test_a_hundred_messages_that_failed_the_same_way_are_one_sentence() {
        // This project's own rule is that feedback must be bounded and must
        // not flood under a syncing mailbox. A rule naming a folder somebody
        // has since renamed fails on every message it matches, so the sentence
        // explaining it arrives once per message. Repeated it is a status line
        // nobody reaches the end of, and it says nothing the first copy did
        // not: the count in front of it already says how many messages.
        let same = "A rule files mail into Invoices, which this account does not have, so it was \
                    left where it is";
        let said = what_the_folder_sync_did(&FolderSync {
            folder: "Inbox".to_string(),
            held: 100,
            total_on_server: 100,
            filtered: Filtered {
                could_not_be_filed: vec![same.to_string(); 100],
                ..Filtered::default()
            },
            ..FolderSync::default()
        });

        assert!(
            said.contains("100 messages not filed as asked"),
            "the count of messages was lost: {said}"
        );
        assert_eq!(
            said.matches("which this account does not have").count(),
            1,
            "one reason was said more than once: {said}"
        );
    }

    #[test]
    fn test_a_sync_that_failed_several_different_ways_says_a_few_and_counts_the_rest() {
        // Different reasons cannot be folded into each other, so the only
        // bound left is a limit. Two, and the rest in the log: a status line
        // is read from the first word to the last, and every extra sentence
        // is one more a person listens through before the next thing that
        // happened. Two is enough to tell one thing going wrong from several
        // different things going wrong.
        let said = what_the_folder_sync_did(&FolderSync {
            folder: "Inbox".to_string(),
            held: 4,
            total_on_server: 4,
            filtered: Filtered {
                could_not_be_filed: vec![
                    "The first thing that went wrong".to_string(),
                    "The second thing that went wrong".to_string(),
                    "The third thing that went wrong".to_string(),
                    "The fourth thing that went wrong".to_string(),
                ],
                ..Filtered::default()
            },
            ..FolderSync::default()
        });

        assert!(said.contains("The first thing"), "{said}");
        assert!(said.contains("The second thing"), "{said}");
        assert!(
            !said.contains("The third thing"),
            "every reason was read out, so a bad sync floods the status line: {said}"
        );
        assert!(
            said.contains("2 other reasons are in the log"),
            "the reasons that were not read out were not accounted for: {said}"
        );
    }

    #[test]
    fn test_one_reason_left_over_is_not_read_out_as_reasons_are() {
        // The same shape as "1 messages": a count and its verb have to agree,
        // and this line is read aloud.
        let said = what_the_folder_sync_did(&FolderSync {
            folder: "Inbox".to_string(),
            held: 3,
            total_on_server: 3,
            filtered: Filtered {
                could_not_be_filed: vec![
                    "The first thing that went wrong".to_string(),
                    "The second thing that went wrong".to_string(),
                    "The third thing that went wrong".to_string(),
                ],
                ..Filtered::default()
            },
            ..FolderSync::default()
        });

        assert!(
            said.contains("1 other reason is in the log"),
            "a count and its verb do not agree: {said}"
        );
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
            delimiter: None,
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

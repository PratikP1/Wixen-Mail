//! What a rule did to arriving mail, and what the mail server is told of it.
//!
//! # The defect this is about
//!
//! A rule that marks arriving mail read, flags it or labels it wrote that here
//! and nowhere else. The next check read the message's flags back from the
//! server, which had never heard of the change, and undid it: on a server that
//! cannot say only what changed, every rule's mark lasted until the next check
//! (ledger 678). Pratik decided on 2026-09-29 that a rule's changes go to the
//! server through the gate and the queue the menu commands use.
//!
//! # Where the sending happens
//!
//! In the check that brought the message, on the session it already holds,
//! after the rules have run and before the rule's move and before the flags are
//! read back. Before the move, so a mark names the folder and the number the
//! server still has the message under; before the flag read, so the read finds
//! the server already holding what the rule did.
//!
//! This module decides; [`crate::application::mail_sync`] sends. Nothing here
//! names a connection.

use std::collections::BTreeMap;

use crate::application::filters::Outcome;
use crate::application::flag_changes_waiting::{
    self, WhatAFailedPushCallsFor, WhichFlag, WhyThePushFailed,
};
use crate::application::moves_waiting::Replayed;
use crate::common::Result;
use crate::data::message_cache::waiting_flag_changes::AWaitingFlagChange;
use crate::data::message_cache::{CachedMessage, MessageCache, Tag};
use crate::service::caldav::how_many;
use crate::service::protocols::imap::flag::{FLAGGED, SEEN};

/// One change a rule made here that the mail server is to be told about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AChange {
    /// Read, or unread.
    Read(bool),
    /// Flagged, or not.
    Flagged(bool),
    /// A label put on, by the keyword it travels as.
    Label {
        keyword: String,
        tag_id: String,
        name: String,
    },
}

impl AChange {
    /// Which flag of the waiting queue this is, for the two it holds.
    pub fn which_flag(&self) -> Option<WhichFlag> {
        match self {
            AChange::Read(_) => Some(WhichFlag::Read),
            AChange::Flagged(_) => Some(WhichFlag::Starred),
            AChange::Label { .. } => None,
        }
    }

    /// The flag the server is asked to set or clear, and which.
    pub fn as_sent(&self) -> (&str, bool) {
        match self {
            AChange::Read(read) => (SEEN, *read),
            AChange::Flagged(flagged) => (FLAGGED, *flagged),
            AChange::Label { keyword, .. } => (keyword, true),
        }
    }
}

/// What the server is told about one arriving message, from what it arrived
/// as, what its rules settled on and the labels they put on here.
///
/// Only what changed: a mark the message arrived with is not sent again, and
/// a label with no keyword has nothing it could travel as. A delete carries
/// nothing else, as [`crate::application::filters::settle`] decides.
pub fn what_the_server_is_told(
    arrived: &CachedMessage,
    outcome: &Outcome,
    labels_put_on: &[Tag],
) -> Vec<AChange> {
    if outcome.delete {
        return Vec::new();
    }
    let read = outcome
        .read
        .filter(|read| *read != arrived.read)
        .map(AChange::Read);
    let flagged = outcome
        .starred
        .filter(|flagged| *flagged != arrived.starred)
        .map(AChange::Flagged);
    let labels = labels_put_on.iter().filter_map(|label| {
        Some(AChange::Label {
            keyword: label.keyword.clone()?,
            tag_id: label.id.clone(),
            name: label.name.clone(),
        })
    });
    read.into_iter().chain(flagged).chain(labels).collect()
}

/// One message's changes waiting to be told to the server by the check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Telling {
    /// The row here, which a change put back is put back on.
    pub message_row: i64,
    /// What the server calls it in the folder it arrived in.
    pub uid: u32,
    pub changes: Vec<AChange>,
    /// Whether the rules counted it as sorted already, so a change put back
    /// can take it out of that count again.
    pub counted: bool,
}

/// How the server answered one change, or that it was never asked.
#[derive(Debug, Clone, Copy)]
pub enum Answered<'a> {
    /// The server took it.
    Went,
    /// Nothing was dialled, because changing mail is not allowed.
    NotDialled,
    /// The send failed, for the reason given.
    Failed(&'a crate::common::Error),
}

/// What a change that did not go waits for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Until {
    ChangingMailIsAllowed,
    TheServerCanBeReached,
}

impl Until {
    /// The reason in the words a clause ends with, which an emptying of the
    /// Trash says too (13-44.6).
    pub fn in_words(self) -> &'static str {
        match self {
            Until::ChangingMailIsAllowed => "changing mail is allowed",
            Until::TheServerCanBeReached => "the mail server can be reached",
        }
    }
}

/// Why a change was put back here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Because {
    ChangingMailIsNotAllowed,
    TheServerCouldNotBeReached,
    TheServerSaidNo,
}

impl Because {
    /// The reason in the words a clause ends with, which an emptying of the
    /// Trash says too (13-44.6).
    pub fn in_words(self) -> &'static str {
        match self {
            Because::ChangingMailIsNotAllowed => "changing mail is not allowed",
            Because::TheServerCouldNotBeReached => "the mail server could not be reached",
            Because::TheServerSaidNo => "the mail server said no",
        }
    }

    /// What would let a change that failed this way go, when anything would.
    fn waits_until(self) -> Option<Until> {
        match self {
            Because::ChangingMailIsNotAllowed => Some(Until::ChangingMailIsAllowed),
            Because::TheServerCouldNotBeReached => Some(Until::TheServerCanBeReached),
            Because::TheServerSaidNo => None,
        }
    }
}

impl From<WhyThePushFailed> for Because {
    fn from(why: WhyThePushFailed) -> Self {
        match why {
            WhyThePushFailed::TheServerWasNeverAsked => Because::TheServerCouldNotBeReached,
            WhyThePushFailed::ThisComputerRefusedIt => Because::ChangingMailIsNotAllowed,
            WhyThePushFailed::TheServerSaidNo => Because::TheServerSaidNo,
        }
    }
}

/// What one change calls for once the server has answered, or was not asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ForTheChange {
    /// It is at the server. Nothing to do and nothing to say.
    Done,
    /// Kept here in the queue a Mark as Read made with changes off waits in.
    KeptWaiting(Until),
    /// Taken off again here.
    PutBack(Because),
}

/// What a change calls for, from how the server answered.
///
/// The three ways a send fails are told apart by
/// [`flag_changes_waiting::why_the_push_failed`] and what each calls for by
/// [`flag_changes_waiting::what_to_do_about_it`], as for a menu's mark; a gate
/// that dialled nothing is this computer refusing it. A mark or a flag that is
/// worth keeping waits in the queue the menu's marks wait in. A label has
/// nowhere to wait, since that queue holds read and flag only, so it is put
/// back for the same reason (D4).
pub fn what_the_change_calls_for(change: &AChange, answer: Answered<'_>) -> ForTheChange {
    let why = match answer {
        Answered::Went => return ForTheChange::Done,
        Answered::NotDialled => WhyThePushFailed::ThisComputerRefusedIt,
        Answered::Failed(error) => flag_changes_waiting::why_the_push_failed(error),
    };
    let because = Because::from(why);
    match (
        flag_changes_waiting::what_to_do_about_it(why),
        change.which_flag(),
        because.waits_until(),
    ) {
        (WhatAFailedPushCallsFor::KeepItAndWait, Some(_), Some(until)) => {
            ForTheChange::KeptWaiting(until)
        }
        _ => ForTheChange::PutBack(because),
    }
}

/// What a delete made here and sent on a check's session calls for, from what
/// the menu's replay answered and how the failure was read when there was one.
///
/// A rule's Delete asks it, and so does an emptying of the Trash (13-44.6),
/// through [`crate::application::moves_waiting::delete_here_then_at_the_server`].
/// The menu's replay already decides what its answer means, in
/// [`crate::application::moves_waiting::what_a_replay_answered`]; this only
/// says it in the words the folder's line uses. A server never reached leaves
/// the delete waiting in the menu's store, for the next check's replay, and a
/// refusal is put back, by this computer's gate or by the server, since a
/// waiting delete would stop the account's check (D11).
pub fn what_a_waiting_delete_calls_for(
    replayed: &Replayed,
    failed: Option<WhyThePushFailed>,
) -> ForTheChange {
    match (replayed, failed) {
        (Replayed::Done | Replayed::AlreadyDone | Replayed::DoneWithSomethingToSay(_), _) => {
            ForTheChange::Done
        }
        (Replayed::NotReached, _) => ForTheChange::KeptWaiting(Until::TheServerCanBeReached),
        (Replayed::Refused(_), Some(WhyThePushFailed::ThisComputerRefusedIt)) => {
            ForTheChange::PutBack(Because::ChangingMailIsNotAllowed)
        }
        (Replayed::Refused(_), _) => ForTheChange::PutBack(Because::TheServerSaidNo),
    }
}

/// Do here what one change calls for: keep it in the queue, or put it back
/// as the message arrived.
pub fn do_here_what_it_calls_for(
    cache: &MessageCache,
    waiting_in: &AWaitingPlace<'_>,
    message: &Telling,
    change: &AChange,
    became: ForTheChange,
) -> Result<()> {
    match became {
        ForTheChange::Done => Ok(()),
        ForTheChange::KeptWaiting(_) => match change.which_flag() {
            Some(which_flag) => cache.keep_a_flag_change_waiting(&AWaitingFlagChange {
                message_row_id: message.message_row,
                account_id: waiting_in.account_id.to_string(),
                folder_path: waiting_in.folder_path.to_string(),
                uid: message.uid,
                which_flag,
                changed_to: change.as_sent().1,
                changed_at: chrono::Utc::now().to_rfc3339(),
            }),
            None => Ok(()),
        },
        ForTheChange::PutBack(_) => put_back_here(cache, message.message_row, change),
    }
}

/// The account and folder a kept change names, which is where the server
/// still has the message.
#[derive(Debug, Clone, Copy)]
pub struct AWaitingPlace<'a> {
    pub account_id: &'a str,
    pub folder_path: &'a str,
}

/// Take one change off the row again, back to how the message arrived.
fn put_back_here(cache: &MessageCache, row: i64, change: &AChange) -> Result<()> {
    if let AChange::Label { tag_id, .. } = change {
        return cache.remove_tag_from_message(row, tag_id);
    }
    let Some(now) = cache.get_message(row)? else {
        return Ok(());
    };
    match change {
        AChange::Read(read) => cache.update_message_flags(row, !read, now.starred),
        AChange::Flagged(flagged) => cache.update_message_flags(row, now.read, !flagged),
        AChange::Label { .. } => Ok(()),
    }
}

/// What became of the changes the check told the server about.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Told {
    /// How many changes came to each outcome that was not done.
    came_to: BTreeMap<ForTheChange, usize>,
    /// The rows a change to which waits, which the check does not file in
    /// that check (D6).
    pub waiting: Vec<i64>,
    /// The rows the rules counted as sorted whose change was put back.
    pub put_back_after_counting: Vec<i64>,
}

impl Told {
    /// Write down what one change of one message came to.
    pub fn record(&mut self, message: &Telling, became: ForTheChange) {
        let rows = match became {
            ForTheChange::Done => return,
            ForTheChange::KeptWaiting(_) => Some(&mut self.waiting),
            ForTheChange::PutBack(_) if message.counted => Some(&mut self.put_back_after_counting),
            ForTheChange::PutBack(_) => None,
        };
        if let Some(rows) = rows
            && !rows.contains(&message.message_row)
        {
            rows.push(message.message_row);
        }
        self.count(became);
    }

    /// Count one change's outcome for the folder's line, a rule's Delete
    /// among them.
    pub fn count(&mut self, became: ForTheChange) {
        if became != ForTheChange::Done {
            *self.came_to.entry(became).or_default() += 1;
        }
    }

    /// The clauses the folder's line says, one per kind of outcome that was
    /// not done, each with its count.
    ///
    /// The count first and the reason last, and no verb that has to agree
    /// with the count, so "1 change" and "3 changes" need no second wording.
    /// None opens as the held-back clause does, "left alone", which is about
    /// a message a rule did not touch at all.
    pub fn clauses(&self) -> Vec<String> {
        self.came_to
            .iter()
            .filter_map(|(became, count)| {
                let changes = how_many(*count, "change");
                match became {
                    ForTheChange::Done => None,
                    ForTheChange::KeptWaiting(until) => Some(format!(
                        "{changes} from your rules kept here until {}",
                        until.in_words()
                    )),
                    ForTheChange::PutBack(because) => Some(format!(
                        "{changes} from your rules put back because {}",
                        because.in_words()
                    )),
                }
            })
            .collect()
    }
}

/// What the folder's line says of a message a rule would have filed and that
/// stayed where it arrived, because a change to it waits.
///
/// Folder names only, never a subject: these sentences go to the log too.
pub fn left_where_it_arrived(from: &str, into: &str) -> String {
    format!(
        "A rule's change to a message has not reached the mail server yet, so the message \
         stays in {from} rather than being filed into {into}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::filters::FilterEngine;
    use crate::application::mail_sync::{
        Filtering, FolderSync, Mailbox, WhatThisSyncIsFor, sync_folder,
    };
    use crate::application::moves_waiting::ReplaysAMove;
    use crate::common::Result;
    use crate::common::types::FolderType;
    use crate::data::message_cache::moves_waiting::{MarksFirst, WhatAWaitingMoveDoes};
    use crate::data::message_cache::{CachedFolder, MessageCache, MessageFilterRule};
    use crate::service::protocols::imap::abilities::Abilities;
    use crate::service::protocols::imap::{
        FolderCounts, ImapFolder, ImapMessage, MailboxStatus, Moved,
    };
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    const THE_ACCOUNT: &str = "acct";

    /// One message as a server holds it in one folder.
    #[derive(Debug, Clone)]
    struct HeldThere {
        uid: u32,
        message_id: String,
        subject: String,
        flags: Vec<String>,
    }

    /// A mail server that keeps each message's flags and changes them when
    /// asked, and writes down every change it was asked for, in order.
    ///
    /// Of its own rather than `mail_sync`'s scripted server, because these
    /// cases are about what a check leaves at the server and what the next
    /// check reads back, and that server answers from a script that nothing
    /// a check sends can change.
    struct AServerThatKeepsFlags {
        folders: RefCell<BTreeMap<String, Vec<HeldThere>>>,
        /// `Some` for a server that can answer "what changed since".
        highest_modseq: Option<u64>,
        /// Every STORE and MOVE, in the order asked.
        log: RefCell<Vec<String>>,
        /// How every STORE is answered, when it is not taken.
        refuses_a_store_with: Option<Refusal>,
        /// How every DELETE is answered, when it is not taken; a cell, so a
        /// case can let the next one through.
        refuses_a_delete_with: std::cell::Cell<Option<Refusal>>,
    }

    /// The ways a change does not go.
    #[derive(Debug, Clone, Copy)]
    enum Refusal {
        /// The server answered, and the answer was no.
        SaidNo,
        /// The connection went before anything came back.
        Dropped,
        /// This computer's own gate refused it before anything was sent.
        TheGate,
    }

    impl Refusal {
        fn as_error(self) -> crate::common::Error {
            match self {
                Refusal::SaidNo => crate::common::Error::Protocol("NO not today".into()),
                Refusal::Dropped => crate::common::Error::Network("the connection went".into()),
                Refusal::TheGate => crate::common::Error::Security("Allow Changes is off".into()),
            }
        }
    }

    impl AServerThatKeepsFlags {
        fn refusing_a_store(self, refusal: Refusal) -> Self {
            Self {
                refuses_a_store_with: Some(refusal),
                ..self
            }
        }

        fn refusing_a_delete(self, refusal: Refusal) -> Self {
            self.refuses_a_delete_with.set(Some(refusal));
            self
        }

        /// The same server, taking every delete from now on.
        fn answering_again(&self) {
            self.refuses_a_delete_with.set(None);
        }

        /// Take a message out of one folder and into another under the next
        /// number there, or out of the server when there is no other.
        fn take_it(&self, from: &str, uid: u32, into: Option<&str>) {
            let mut folders = self.folders.borrow_mut();
            let leaving = folders.get_mut(from).and_then(|held| {
                let at = held.iter().position(|message| message.uid == uid)?;
                Some(held.remove(at))
            });
            if let (Some(mut message), Some(into)) = (leaving, into) {
                let arriving = folders.entry(into.to_string()).or_default();
                message.uid = arriving.iter().map(|held| held.uid).max().unwrap_or(0) + 1;
                arriving.push(message);
            }
        }

        fn holding(folder: &str, messages: &[(u32, &str, &[&str])]) -> Self {
            let held = messages
                .iter()
                .map(|(uid, subject, flags)| HeldThere {
                    uid: *uid,
                    message_id: format!("{uid}.{folder}@example.com"),
                    subject: (*subject).to_string(),
                    flags: flags.iter().map(|flag| (*flag).to_string()).collect(),
                })
                .collect();
            Self {
                folders: RefCell::new(BTreeMap::from([(folder.to_string(), held)])),
                highest_modseq: None,
                log: RefCell::new(Vec::new()),
                refuses_a_store_with: None,
                refuses_a_delete_with: std::cell::Cell::new(None),
            }
        }

        fn answering_what_changed_since(self) -> Self {
            Self {
                highest_modseq: Some(5),
                ..self
            }
        }

        /// A message arriving after the first check.
        fn and_then_receiving(&self, folder: &str, uid: u32, subject: &str) {
            self.folders
                .borrow_mut()
                .entry(folder.to_string())
                .or_default()
                .push(HeldThere {
                    uid,
                    message_id: format!("{uid}.{folder}@example.com"),
                    subject: subject.to_string(),
                    flags: Vec::new(),
                });
        }

        fn the_log(&self) -> Vec<String> {
            self.log.borrow().clone()
        }

        fn in_folder(&self, folder: &str) -> Vec<HeldThere> {
            self.folders
                .borrow()
                .get(folder)
                .cloned()
                .unwrap_or_default()
        }
    }

    impl Mailbox for AServerThatKeepsFlags {
        async fn folder_counts(&self, folder: &str) -> Result<FolderCounts> {
            let held = self.in_folder(folder);
            let unread = held
                .iter()
                .filter(|message| !message.flags.iter().any(|flag| flag == SEEN))
                .count();
            Ok(FolderCounts {
                total: held.len() as u32,
                unread: unread as u32,
            })
        }

        async fn select_folder(&self, _folder: &str) -> Result<MailboxStatus> {
            Ok(MailboxStatus {
                uid_validity: Some(1),
                highest_modseq: self.highest_modseq,
                keeps_the_junk_mark: false,
            })
        }

        async fn what_this_server_can_do(&self) -> Abilities {
            Abilities::default()
        }

        async fn list_uids(&self, folder: &str) -> Result<Vec<u32>> {
            Ok(self.in_folder(folder).iter().map(|held| held.uid).collect())
        }

        async fn list_uids_above(&self, folder: &str, after: u32) -> Result<Vec<u32>> {
            Ok(self
                .list_uids(folder)
                .await?
                .into_iter()
                .filter(|uid| *uid >= after)
                .collect())
        }

        async fn fetch_headers(&self, folder: &str, uids: &[u32]) -> Result<Vec<ImapMessage>> {
            Ok(self
                .in_folder(folder)
                .into_iter()
                .filter(|held| uids.contains(&held.uid))
                .map(|held| ImapMessage {
                    uid: held.uid,
                    subject: held.subject,
                    message_id: Some(held.message_id),
                    flags: held.flags,
                    ..Default::default()
                })
                .collect())
        }

        async fn thread_ids_of(&self, _folder: &str, _uids: &[u32]) -> Result<Vec<(u32, u64)>> {
            Ok(Vec::new())
        }

        async fn move_message(&self, from: &str, uid: u32, into: &str) -> Result<Moved> {
            self.log
                .borrow_mut()
                .push(format!("MOVE {uid} in {from} into {into}"));
            self.take_it(from, uid, Some(into));
            Ok(Moved::Moved)
        }

        async fn fetch_flags(
            &self,
            folder: &str,
            held: &[u32],
            changed_since: Option<u64>,
        ) -> Result<Vec<(u32, Vec<String>)>> {
            // Asked "what changed since", a server answers for the whole
            // mailbox, messages this same check just brought down included,
            // as `mail_sync`'s scripted server does.
            Ok(self
                .in_folder(folder)
                .into_iter()
                .filter(|message| changed_since.is_some() || held.contains(&message.uid))
                .map(|message| (message.uid, message.flags))
                .collect())
        }

        async fn fetch_message_body(&self, _folder: &str, _uid: u32) -> Result<Vec<u8>> {
            Ok(Vec::new())
        }

        async fn set_flag(&self, folder: &str, uid: u32, flag: &str, on: bool) -> Result<()> {
            let sign = if on { '+' } else { '-' };
            self.log
                .borrow_mut()
                .push(format!("STORE {sign}{flag} on {uid} in {folder}"));
            if let Some(refusal) = self.refuses_a_store_with {
                return Err(refusal.as_error());
            }
            let mut folders = self.folders.borrow_mut();
            if let Some(message) = folders
                .get_mut(folder)
                .and_then(|held| held.iter_mut().find(|message| message.uid == uid))
            {
                message.flags.retain(|held| held != flag);
                if on {
                    message.flags.push(flag.to_string());
                }
            }
            Ok(())
        }
    }

    /// What the menu's replay asks, which a rule's Delete is sent through.
    impl ReplaysAMove for AServerThatKeepsFlags {
        async fn move_it(&self, from: &str, uid: u32, into: &str) -> Result<()> {
            Mailbox::move_message(self, from, uid, into)
                .await
                .map(|_| ())
        }

        async fn delete_it(&self, folder: &str, uid: u32, trash: Option<&str>) -> Result<()> {
            self.log.borrow_mut().push(match trash {
                Some(trash) => format!("DELETE {uid} in {folder} into {trash}"),
                None => format!("DELETE {uid} in {folder}"),
            });
            if let Some(refusal) = self.refuses_a_delete_with.get() {
                return Err(refusal.as_error());
            }
            self.take_it(folder, uid, trash);
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
                .in_folder(folder)
                .into_iter()
                .filter(|held| held.message_id == message_id)
                .map(|held| held.uid)
                .collect())
        }

        async fn mark_it(&self, folder: &str, uid: u32, marks: MarksFirst) -> Result<()> {
            if let Some(read) = marks.read {
                Mailbox::set_flag(self, folder, uid, SEEN, read).await?;
            }
            if let Some(starred) = marks.starred {
                Mailbox::set_flag(self, folder, uid, FLAGGED, starred).await?;
            }
            for keyword in &marks.keywords {
                Mailbox::set_flag(self, folder, uid, keyword, true).await?;
            }
            Ok(())
        }
    }

    /// A cache holding the account's Inbox, a folder to file into and its
    /// Trash, with the label Money, sent as a keyword, and the label Local,
    /// which has none.
    fn an_account() -> (tempfile::TempDir, MessageCache, i64) {
        let (dir, cache, inbox) = an_account_without_a_trash();
        a_folder(&cache, "Trash", "Trash", FolderType::Trash);
        (dir, cache, inbox)
    }

    /// The same account, with no folder it keeps deleted mail in.
    fn an_account_without_a_trash() -> (tempfile::TempDir, MessageCache, i64) {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
        let inbox = a_folder(&cache, "Inbox", "INBOX", FolderType::Inbox);
        a_folder(&cache, "Invoices", "Invoices", FolderType::Custom);
        for (id, name, keyword) in [
            ("tag-money", "Money", Some("Money")),
            ("tag-local", "Local", None),
        ] {
            cache
                .create_tag(&Tag {
                    id: id.to_string(),
                    account_id: THE_ACCOUNT.to_string(),
                    name: name.to_string(),
                    color: "#009900".to_string(),
                    created_at: "2026-10-01T00:00:00Z".to_string(),
                    keyword: keyword.map(str::to_string),
                })
                .expect("the label made");
        }
        (dir, cache, inbox)
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

    /// The folder as the server lists it.
    fn the_inbox() -> ImapFolder {
        a_folder_listed("Inbox", "INBOX", FolderType::Inbox)
    }

    fn a_folder_listed(name: &str, path: &str, kind: FolderType) -> ImapFolder {
        ImapFolder {
            name: name.into(),
            display_path: path.into(),
            path: path.into(),
            folder_type: kind,
            selectable: true,
            holds_all_mail: false,
            subscribed: true,
            delimiter: None,
        }
    }

    /// Rules that each match a subject holding "Invoice", one per action and
    /// its value, in the order given.
    fn rules(actions: &[(&str, Option<&str>)]) -> FilterEngine {
        let written: Vec<MessageFilterRule> = actions
            .iter()
            .enumerate()
            .map(|(nth, (action, value))| MessageFilterRule {
                id: format!("r{nth}"),
                account_id: THE_ACCOUNT.into(),
                name: format!("Rule {nth}"),
                field: "subject".into(),
                match_type: "contains".into(),
                pattern: "Invoice".into(),
                case_sensitive: false,
                action_type: (*action).to_string(),
                action_value: value.map(str::to_string),
                enabled: true,
                plays_a_sound: false,
                created_at: "2026-10-01T00:00:00Z".into(),
            })
            .collect();
        let mut engine = FilterEngine::default();
        engine.load_from_persisted(&written);
        engine
    }

    /// One check of the Inbox, with the rules and the account's answer to
    /// whether mail may be changed.
    fn a_check(
        server: &AServerThatKeepsFlags,
        cache: &MessageCache,
        inbox: i64,
        engine: &FilterEngine,
        allowed: crate::application::allowed::Allowed,
    ) -> FolderSync {
        a_check_of(server, cache, (&the_inbox(), inbox), engine, allowed)
    }

    /// One check of any folder, as the server lists it and as its row here.
    fn a_check_of(
        server: &AServerThatKeepsFlags,
        cache: &MessageCache,
        (folder, folder_id): (&ImapFolder, i64),
        engine: &FilterEngine,
        allowed: crate::application::allowed::Allowed,
    ) -> FolderSync {
        let filtering = Filtering {
            rules: engine,
            allowed,
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
                WhatThisSyncIsFor::WhateverHasChanged,
            ))
            .expect("the check to finish")
    }

    fn allowed() -> crate::application::allowed::Allowed {
        crate::application::allowed::Allowed::EVERYTHING
    }

    /// The row here for a number in a folder, read back.
    fn the_row(cache: &MessageCache, folder: i64, uid: u32) -> CachedMessage {
        let row = cache
            .message_row_for_uid(folder, uid)
            .expect("the row looked up")
            .expect("the row is here");
        cache
            .get_message(row)
            .expect("the row read")
            .expect("the row is there")
    }

    fn the_labels_on(cache: &MessageCache, row: i64) -> Vec<String> {
        cache
            .get_tags_for_message(row)
            .expect("the labels read")
            .into_iter()
            .map(|tag| tag.name)
            .collect()
    }

    #[test]
    fn test_a_rule_that_marks_read_and_files_tells_the_server_the_mark_before_the_move() {
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])]);
        let engine = rules(&[("mark_as_read", None), ("move_to_folder", Some("Invoices"))]);

        a_check(&server, &cache, inbox, &engine, allowed());

        assert_eq!(
            server.the_log(),
            [
                format!("STORE +{SEEN} on 7 in INBOX"),
                "MOVE 7 in INBOX into Invoices".to_string(),
            ],
            "the mark did not reach the server before the move, from where the message was"
        );
    }

    #[test]
    fn test_a_rule_that_flags_and_labels_tells_the_server_the_flag_and_the_labels_keyword() {
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])]);
        let engine = rules(&[("star", None), ("add_tag", Some("Money"))]);

        a_check(&server, &cache, inbox, &engine, allowed());

        assert_eq!(
            server.the_log(),
            [
                format!("STORE +{FLAGGED} on 7 in INBOX"),
                "STORE +Money on 7 in INBOX".to_string(),
            ]
        );
        let row = the_row(&cache, inbox, 7);
        assert!(row.starred, "the flag is not on the row here");
        assert_eq!(the_labels_on(&cache, row.id), ["Money"]);
    }

    #[test]
    fn test_a_mark_the_message_arrived_with_is_not_sent_again() {
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[SEEN])]);
        let engine = rules(&[("mark_as_read", None)]);

        a_check(&server, &cache, inbox, &engine, allowed());

        assert!(
            server.the_log().is_empty(),
            "a mark the server already held was sent again: {:?}",
            server.the_log()
        );
        assert!(the_row(&cache, inbox, 7).read);
    }

    #[test]
    fn test_a_label_with_no_keyword_is_put_on_here_and_sent_nowhere() {
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])]);
        let engine = rules(&[("add_tag", Some("Local"))]);

        let done = a_check(&server, &cache, inbox, &engine, allowed());

        assert!(server.the_log().is_empty(), "{:?}", server.the_log());
        let row = the_row(&cache, inbox, 7);
        assert_eq!(the_labels_on(&cache, row.id), ["Local"]);
        assert_eq!(done.filtered.changed, 1);
    }

    #[test]
    fn test_the_next_check_leaves_a_rules_mark_the_server_now_holds() {
        // A server that cannot say only what changed answers the next check
        // with every message's flags, and until 2026-10-01 those flags were
        // the ones the message arrived with, so the rule's mark was undone.
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])]);
        let engine = rules(&[("mark_as_read", None)]);
        a_check(&server, &cache, inbox, &engine, allowed());

        let next = a_check(&server, &cache, inbox, &engine, allowed());

        assert!(
            the_row(&cache, inbox, 7).read,
            "the next check undid the rule's mark"
        );
        assert_eq!(next.flags_updated, 0, "the next check changed a row");
    }

    #[test]
    fn test_a_server_that_answers_what_changed_since_keeps_a_rules_mark_in_the_same_check() {
        // Asked what changed since, the server answers for the whole mailbox,
        // the message this check just brought down included, so a mark the
        // server never heard of was undone by the check that made it.
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Lunch", &[])])
            .answering_what_changed_since();
        let engine = rules(&[("mark_as_read", None)]);
        a_check(&server, &cache, inbox, &engine, allowed());
        server.and_then_receiving("INBOX", 8, "Invoice 4021");

        a_check(&server, &cache, inbox, &engine, allowed());

        assert!(
            the_row(&cache, inbox, 8).read,
            "the check that made the rule's mark undid it"
        );
    }

    #[test]
    fn test_what_the_server_is_told_is_only_what_the_rules_changed() {
        let arrived = |read: bool, starred: bool| CachedMessage {
            id: 1,
            uid: 7,
            folder_id: 1,
            message_id: "7.INBOX@example.com".to_string(),
            subject: "Invoice 4021".to_string(),
            from_addr: String::new(),
            to_addr: String::new(),
            cc: None,
            date: String::new(),
            body_plain: None,
            body_html: None,
            read,
            starred,
            deleted: false,
            safety: crate::service::safety::Safety::default(),
        };
        let label = |id: &str, name: &str, keyword: Option<&str>| Tag {
            id: id.to_string(),
            account_id: THE_ACCOUNT.to_string(),
            name: name.to_string(),
            color: String::new(),
            created_at: String::new(),
            keyword: keyword.map(str::to_string),
        };
        let marking = |read: Option<bool>, starred: Option<bool>| Outcome {
            read,
            starred,
            ..Outcome::default()
        };
        let rows: Vec<(CachedMessage, Outcome, Vec<Tag>, Vec<AChange>)> = vec![
            (
                arrived(false, false),
                marking(Some(true), None),
                vec![],
                vec![AChange::Read(true)],
            ),
            (
                arrived(true, false),
                marking(Some(true), None),
                vec![],
                vec![],
            ),
            (
                arrived(true, false),
                marking(Some(false), None),
                vec![],
                vec![AChange::Read(false)],
            ),
            (
                arrived(false, false),
                marking(None, Some(true)),
                vec![],
                vec![AChange::Flagged(true)],
            ),
            (
                arrived(false, true),
                marking(None, Some(true)),
                vec![],
                vec![],
            ),
            (
                arrived(false, true),
                marking(None, Some(false)),
                vec![],
                vec![AChange::Flagged(false)],
            ),
            (
                arrived(false, false),
                Outcome {
                    tags: vec!["Money".into(), "Local".into()],
                    ..Outcome::default()
                },
                vec![
                    label("tag-money", "Money", Some("Money")),
                    label("tag-local", "Local", None),
                ],
                vec![AChange::Label {
                    keyword: "Money".into(),
                    tag_id: "tag-money".into(),
                    name: "Money".into(),
                }],
            ),
            (
                arrived(false, false),
                Outcome {
                    read: Some(true),
                    delete: true,
                    ..Outcome::default()
                },
                vec![],
                vec![],
            ),
            (arrived(false, false), Outcome::default(), vec![], vec![]),
        ];
        for (message, outcome, labels, told) in rows {
            assert_eq!(
                what_the_server_is_told(&message, &outcome, &labels),
                told,
                "arrived read {} starred {}, outcome {outcome:?}",
                message.read,
                message.starred
            );
        }
    }

    // ── What cannot go now waits here or comes back, and is said once ───────

    fn not_allowed() -> crate::application::allowed::Allowed {
        crate::application::allowed::Allowed::NOTHING
    }

    /// What the check's line says, as the status bar shows it.
    fn said(done: &FolderSync) -> String {
        crate::application::mail_sync::what_the_folder_sync_did(done)
    }

    /// The changes waiting to go for the account, as (row, which, to).
    fn waiting(cache: &MessageCache) -> Vec<(i64, String, u32, bool)> {
        cache
            .flag_changes_waiting_for(THE_ACCOUNT)
            .expect("the waiting changes read")
            .into_iter()
            .map(|change| {
                (
                    change.message_row_id,
                    change.folder_path,
                    change.uid,
                    change.changed_to,
                )
            })
            .collect()
    }

    #[test]
    fn test_with_changing_mail_off_nothing_is_dialled_and_a_rules_mark_waits_here() {
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])]);
        let engine = rules(&[("mark_as_read", None)]);

        let done = a_check(&server, &cache, inbox, &engine, not_allowed());

        assert!(server.the_log().is_empty(), "{:?}", server.the_log());
        let row = the_row(&cache, inbox, 7);
        assert!(row.read, "the rule's mark is not on the row here");
        assert_eq!(waiting(&cache), [(row.id, "INBOX".to_string(), 7, true)]);
        let line = said(&done);
        assert!(
            line.contains("1 change from your rules kept here until changing mail is allowed"),
            "{line}"
        );
    }

    #[test]
    fn test_with_changing_mail_off_a_rules_label_comes_off_again() {
        // Nothing holds a label waiting, so a label the server cannot be told
        // about now is taken off here and said, rather than left for the next
        // check to take off unsaid.
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])]);
        let engine = rules(&[("add_tag", Some("Money"))]);

        let done = a_check(&server, &cache, inbox, &engine, not_allowed());

        assert!(server.the_log().is_empty(), "{:?}", server.the_log());
        let row = the_row(&cache, inbox, 7);
        assert!(the_labels_on(&cache, row.id).is_empty());
        let line = said(&done);
        assert!(
            line.contains("1 change from your rules put back because changing mail is not allowed"),
            "{line}"
        );
    }

    #[test]
    fn test_a_mark_the_server_refuses_is_put_back_here() {
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])])
            .refusing_a_store(Refusal::SaidNo);
        let engine = rules(&[("mark_as_read", None)]);

        let done = a_check(&server, &cache, inbox, &engine, allowed());

        assert!(
            !the_row(&cache, inbox, 7).read,
            "a refused mark stayed here"
        );
        assert!(
            waiting(&cache).is_empty(),
            "a refused mark was kept waiting"
        );
        let line = said(&done);
        assert!(
            line.contains("1 change from your rules put back because the mail server said no"),
            "{line}"
        );
    }

    #[test]
    fn test_a_mark_the_server_could_not_be_asked_about_waits_and_its_message_is_not_moved() {
        // A move made while its mark waits would leave the waiting mark naming
        // a number the folder no longer holds, 688's shape (D6).
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])])
            .refusing_a_store(Refusal::Dropped);
        let engine = rules(&[("mark_as_read", None), ("move_to_folder", Some("Invoices"))]);

        let done = a_check(&server, &cache, inbox, &engine, allowed());

        assert_eq!(
            server.the_log(),
            [format!("STORE +{SEEN} on 7 in INBOX")],
            "the message was moved while its mark waits"
        );
        let row = the_row(&cache, inbox, 7);
        assert!(row.read);
        assert_eq!(waiting(&cache), [(row.id, "INBOX".to_string(), 7, true)]);
        assert_eq!(
            done.filtered.could_not_be_filed,
            [left_where_it_arrived("Inbox", "Invoices")]
        );
        let line = said(&done);
        assert!(
            line.contains(
                "1 change from your rules kept here until the mail server can be reached"
            ),
            "{line}"
        );
    }

    #[test]
    fn test_a_label_the_server_could_not_be_asked_about_comes_off_again() {
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])])
            .refusing_a_store(Refusal::Dropped);
        let engine = rules(&[("add_tag", Some("Money"))]);

        let done = a_check(&server, &cache, inbox, &engine, allowed());

        let row = the_row(&cache, inbox, 7);
        assert!(the_labels_on(&cache, row.id).is_empty());
        assert!(
            waiting(&cache).is_empty(),
            "a label was kept in the flag queue"
        );
        let line = said(&done);
        assert!(
            line.contains(
                "1 change from your rules put back because the mail server could not be reached"
            ),
            "{line}"
        );
    }

    #[test]
    fn test_the_check_keeps_a_waiting_mark_over_what_the_server_says() {
        // The server has not heard of a mark still waiting to go, so a check
        // that wrote the server's flags as they came undid it (D5), and with
        // it every Mark as Read made while changes were off.
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])]);
        let engine = rules(&[("mark_as_read", None)]);
        a_check(&server, &cache, inbox, &engine, not_allowed());

        let next = a_check(&server, &cache, inbox, &engine, not_allowed());

        assert!(
            the_row(&cache, inbox, 7).read,
            "the next check undid a mark still waiting to go"
        );
        assert_eq!(next.flags_updated, 0);
    }

    #[test]
    fn test_a_server_that_answers_what_changed_since_keeps_a_waiting_mark_in_the_same_check() {
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Lunch", &[])])
            .answering_what_changed_since();
        let engine = rules(&[("mark_as_read", None)]);
        a_check(&server, &cache, inbox, &engine, not_allowed());
        server.and_then_receiving("INBOX", 8, "Invoice 4021");

        a_check(&server, &cache, inbox, &engine, not_allowed());

        assert!(
            the_row(&cache, inbox, 8).read,
            "the check that made the waiting mark undid it"
        );
    }

    #[test]
    fn test_a_message_whose_change_was_put_back_is_not_counted_as_sorted() {
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])])
            .refusing_a_store(Refusal::SaidNo);
        let engine = rules(&[("add_tag", Some("Money"))]);

        let done = a_check(&server, &cache, inbox, &engine, allowed());

        assert_eq!(
            done.filtered.changed, 0,
            "a message whose label was taken off again is counted as sorted"
        );
        assert!(
            !said(&done).contains("sorted by your rules"),
            "{}",
            said(&done)
        );
    }

    #[test]
    fn test_what_each_change_calls_for() {
        let said_no = crate::common::Error::Protocol("NO".into());
        let dropped = crate::common::Error::Network("gone".into());
        let the_gate = crate::common::Error::Security("off".into());
        let label = AChange::Label {
            keyword: "Money".into(),
            tag_id: "tag-money".into(),
            name: "Money".into(),
        };
        let reached = ForTheChange::KeptWaiting(Until::TheServerCanBeReached);
        let allowed = ForTheChange::KeptWaiting(Until::ChangingMailIsAllowed);
        let rows: Vec<(AChange, Answered<'_>, ForTheChange)> = vec![
            (AChange::Read(true), Answered::Went, ForTheChange::Done),
            (AChange::Read(true), Answered::NotDialled, allowed),
            (AChange::Read(false), Answered::Failed(&dropped), reached),
            (AChange::Read(true), Answered::Failed(&the_gate), allowed),
            (
                AChange::Read(true),
                Answered::Failed(&said_no),
                ForTheChange::PutBack(Because::TheServerSaidNo),
            ),
            (AChange::Flagged(true), Answered::Went, ForTheChange::Done),
            (AChange::Flagged(false), Answered::NotDialled, allowed),
            (AChange::Flagged(true), Answered::Failed(&dropped), reached),
            (AChange::Flagged(true), Answered::Failed(&the_gate), allowed),
            (
                AChange::Flagged(true),
                Answered::Failed(&said_no),
                ForTheChange::PutBack(Because::TheServerSaidNo),
            ),
            (label.clone(), Answered::Went, ForTheChange::Done),
            (
                label.clone(),
                Answered::NotDialled,
                ForTheChange::PutBack(Because::ChangingMailIsNotAllowed),
            ),
            (
                label.clone(),
                Answered::Failed(&dropped),
                ForTheChange::PutBack(Because::TheServerCouldNotBeReached),
            ),
            (
                label.clone(),
                Answered::Failed(&the_gate),
                ForTheChange::PutBack(Because::ChangingMailIsNotAllowed),
            ),
            (
                label,
                Answered::Failed(&said_no),
                ForTheChange::PutBack(Because::TheServerSaidNo),
            ),
        ];
        for (change, answer, calls_for) in rows {
            assert_eq!(
                what_the_change_calls_for(&change, answer),
                calls_for,
                "{change:?} answered {answer:?}"
            );
        }
    }

    fn a_message_told(counted: bool) -> Telling {
        Telling {
            message_row: 1,
            uid: 7,
            changes: vec![AChange::Read(true)],
            counted,
        }
    }

    #[test]
    fn test_the_clauses_say_each_count_and_reason_once_and_the_singular_right() {
        let mut told = Told::default();
        let message = a_message_told(true);
        for became in [
            ForTheChange::KeptWaiting(Until::ChangingMailIsAllowed),
            ForTheChange::KeptWaiting(Until::ChangingMailIsAllowed),
            ForTheChange::KeptWaiting(Until::ChangingMailIsAllowed),
            ForTheChange::KeptWaiting(Until::TheServerCanBeReached),
            ForTheChange::PutBack(Because::ChangingMailIsNotAllowed),
            ForTheChange::PutBack(Because::TheServerCouldNotBeReached),
            ForTheChange::PutBack(Because::TheServerSaidNo),
            ForTheChange::PutBack(Because::TheServerSaidNo),
            ForTheChange::Done,
        ] {
            told.record(&message, became);
        }

        let clauses = told.clauses();

        assert_eq!(
            clauses,
            [
                "3 changes from your rules kept here until changing mail is allowed",
                "1 change from your rules kept here until the mail server can be reached",
                "1 change from your rules put back because changing mail is not allowed",
                "1 change from your rules put back because the mail server could not be reached",
                "2 changes from your rules put back because the mail server said no",
            ]
        );
        for clause in &clauses {
            assert!(
                !clause.contains("left alone"),
                "a clause opens as the held-back one does: {clause}"
            );
        }
    }

    #[test]
    fn test_a_check_that_told_the_server_everything_says_nothing_more() {
        let mut told = Told::default();
        for _ in 0..3 {
            told.record(&a_message_told(true), ForTheChange::Done);
        }

        assert!(told.clauses().is_empty(), "{:?}", told.clauses());
        assert_eq!(told, Told::default(), "a change that went was written down");
    }

    // ── A rule's Delete, through the menu's delete path ─────────────────────

    /// The row here of the folder at `path`.
    fn the_folder_here(cache: &MessageCache, path: &str) -> i64 {
        cache
            .get_folder(THE_ACCOUNT, path)
            .expect("the folder read")
            .expect("the folder is here")
            .id
    }

    /// Every delete waiting in the menu's store for the account.
    fn deletes_waiting(cache: &MessageCache) -> Vec<(String, u32, WhatAWaitingMoveDoes)> {
        cache
            .moves_waiting_for(THE_ACCOUNT)
            .expect("the waiting moves read")
            .into_iter()
            .map(|waiting| (waiting.from_folder_path, waiting.uid, waiting.what))
            .collect()
    }

    fn a_rule_that_deletes() -> FilterEngine {
        rules(&[("delete", None)])
    }

    #[test]
    fn test_a_rule_that_deletes_on_arrival_sends_the_message_to_the_trash_at_the_server() {
        // Until 2026-10-01 a rule's Delete marked the message deleted on this
        // computer and told the server nothing, so on a server that cannot
        // say only what changed the next check brought it back (D10).
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])]);

        let done = a_check(&server, &cache, inbox, &a_rule_that_deletes(), allowed());

        assert_eq!(server.the_log(), ["DELETE 7 in INBOX into Trash"]);
        assert!(
            server.in_folder("INBOX").is_empty(),
            "the server's Inbox still holds it"
        );
        let at_the_server = server.in_folder("Trash");
        assert_eq!(
            at_the_server.len(),
            1,
            "the server's Trash does not hold it"
        );
        let row = the_row(
            &cache,
            the_folder_here(&cache, "Trash"),
            at_the_server[0].uid,
        );
        assert!(
            !row.deleted,
            "the row is marked deleted rather than in the Trash"
        );
        assert!(deletes_waiting(&cache).is_empty(), "the delete still waits");
        assert_eq!(
            done.filtered.changed, 1,
            "the deleted message is not counted as sorted"
        );
    }

    #[test]
    fn test_the_next_check_does_not_bring_back_a_message_a_rule_deleted() {
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])]);
        a_check(&server, &cache, inbox, &a_rule_that_deletes(), allowed());

        a_check(&server, &cache, inbox, &a_rule_that_deletes(), allowed());

        assert_eq!(
            cache.message_row_for_uid(inbox, 7).expect("the lookup"),
            None,
            "the next check brought the deleted message back to the Inbox"
        );
        assert_eq!(
            cache
                .stored_uids(the_folder_here(&cache, "Trash"))
                .expect("the Trash read")
                .len(),
            1,
            "the message is no longer in the Trash here"
        );
    }

    #[test]
    fn test_a_rules_delete_the_server_refuses_is_put_back_where_it_arrived() {
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])])
            .refusing_a_delete(Refusal::SaidNo);

        let done = a_check(&server, &cache, inbox, &a_rule_that_deletes(), allowed());

        let row = the_row(&cache, inbox, 7);
        assert!(!row.deleted, "a refused delete left the row marked deleted");
        assert!(
            deletes_waiting(&cache).is_empty(),
            "a refused delete still waits"
        );
        let line = said(&done);
        assert!(
            line.contains("1 change from your rules put back because the mail server said no"),
            "{line}"
        );
    }

    #[test]
    fn test_a_rules_delete_the_server_could_not_be_asked_about_waits_in_the_menus_queue() {
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])])
            .refusing_a_delete(Refusal::Dropped);

        let done = a_check(&server, &cache, inbox, &a_rule_that_deletes(), allowed());

        assert_eq!(
            deletes_waiting(&cache),
            [(
                "INBOX".to_string(),
                7,
                WhatAWaitingMoveDoes::DeleteToTrash {
                    trash_path: "Trash".to_string()
                }
            )]
        );
        assert_eq!(
            cache.message_row_for_uid(inbox, 7).expect("the lookup"),
            None,
            "the row stayed in the Inbox here while its delete waits"
        );
        let line = said(&done);
        assert!(
            line.contains(
                "1 change from your rules kept here until the mail server can be reached"
            ),
            "{line}"
        );

        // The next check's own path: the replay before any folder is read.
        server.answering_again();
        let replayed = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
            .block_on(
                crate::application::moves_waiting::replay_the_moves_waiting_for(
                    &server,
                    &cache,
                    THE_ACCOUNT,
                ),
            )
            .expect("the replay");
        assert_eq!(
            replayed
                .into_iter()
                .map(|(_, answer)| answer)
                .collect::<Vec<_>>(),
            [Replayed::Done]
        );
        assert!(deletes_waiting(&cache).is_empty(), "the delete still waits");
        assert_eq!(server.in_folder("Trash").len(), 1);
    }

    #[test]
    fn test_a_rules_delete_this_computers_gate_refuses_is_put_back_and_said_as_not_allowed() {
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])])
            .refusing_a_delete(Refusal::TheGate);

        let done = a_check(&server, &cache, inbox, &a_rule_that_deletes(), allowed());

        let row = the_row(&cache, inbox, 7);
        assert!(!row.deleted);
        assert!(deletes_waiting(&cache).is_empty());
        let line = said(&done);
        assert!(
            line.contains("1 change from your rules put back because changing mail is not allowed"),
            "{line}"
        );
    }

    #[test]
    fn test_a_rules_delete_on_an_account_with_no_trash_deletes_nothing_and_says_why() {
        let (_dir, cache, inbox) = an_account_without_a_trash();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])]);

        let done = a_check(&server, &cache, inbox, &a_rule_that_deletes(), allowed());

        assert!(server.the_log().is_empty(), "{:?}", server.the_log());
        assert!(!the_row(&cache, inbox, 7).deleted);
        assert_eq!(
            said(&done)
                .matches(crate::application::destinations::NO_TRASH_FOLDER_FOUND)
                .count(),
            1,
            "{}",
            said(&done)
        );
    }

    #[test]
    fn test_a_rules_delete_of_a_message_already_in_the_trash_leaves_it_there() {
        // A rule never takes a message off the server: in the Trash, where the
        // menu's Delete would delete outright, it is left alone (D12).
        let (_dir, cache, _inbox) = an_account();
        let trash = the_folder_here(&cache, "Trash");
        let server = AServerThatKeepsFlags::holding("Trash", &[(7, "Invoice 4021", &[])]);

        a_check_of(
            &server,
            &cache,
            (&a_folder_listed("Trash", "Trash", FolderType::Trash), trash),
            &a_rule_that_deletes(),
            allowed(),
        );

        assert!(server.the_log().is_empty(), "{:?}", server.the_log());
        assert!(!the_row(&cache, trash, 7).deleted);
    }

    #[test]
    fn test_with_changing_mail_off_a_rules_delete_is_held_back_and_nothing_is_dialled() {
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])]);

        let done = a_check(
            &server,
            &cache,
            inbox,
            &a_rule_that_deletes(),
            not_allowed(),
        );

        assert!(server.the_log().is_empty(), "{:?}", server.the_log());
        assert!(!the_row(&cache, inbox, 7).deleted);
        assert_eq!(done.filtered.held_back, 1);
    }

    #[test]
    fn test_what_a_rules_delete_calls_for() {
        let rows = [
            (Replayed::Done, None, ForTheChange::Done),
            (
                Replayed::AlreadyDone,
                Some(WhyThePushFailed::TheServerSaidNo),
                ForTheChange::Done,
            ),
            (
                Replayed::DoneWithSomethingToSay("it is in both places".into()),
                None,
                ForTheChange::Done,
            ),
            (
                Replayed::NotReached,
                Some(WhyThePushFailed::TheServerWasNeverAsked),
                ForTheChange::KeptWaiting(Until::TheServerCanBeReached),
            ),
            (
                Replayed::Refused("Allow Changes is off".into()),
                Some(WhyThePushFailed::ThisComputerRefusedIt),
                ForTheChange::PutBack(Because::ChangingMailIsNotAllowed),
            ),
            (
                Replayed::Refused("NO".into()),
                Some(WhyThePushFailed::TheServerSaidNo),
                ForTheChange::PutBack(Because::TheServerSaidNo),
            ),
            (
                Replayed::Refused("NO".into()),
                None,
                ForTheChange::PutBack(Because::TheServerSaidNo),
            ),
        ];
        for (replayed, failed, calls_for) in rows {
            assert_eq!(
                what_a_waiting_delete_calls_for(&replayed, failed),
                calls_for,
                "{replayed:?} read as {failed:?}"
            );
        }
    }
}

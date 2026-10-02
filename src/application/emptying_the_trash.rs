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

use std::collections::BTreeMap;

use chrono::{DateTime, Duration, NaiveDate, Utc};

use crate::application::allowed::SETTINGS_SECTION;
use crate::application::destinations::{DeletedGoesTo, Deleting, where_a_deleted_message_goes};
use crate::application::moves_waiting::{
    APushUnderWay, DeletedHereThenAtTheServer, ReplaysAMove, delete_here_then_at_the_server,
};
use crate::application::what_rules_tell_the_server::{Because, ForTheChange, Until};
use crate::application::who_runs_the_mail::WhoRunsTheMail;
use crate::common::Result;
use crate::common::types::{FolderType, Protocol};
use crate::data::message_cache::MessageCache;
use crate::data::message_cache::in_the_trash::InTheTrash;
use crate::data::message_cache::moves_waiting::{AWaitingMove, WhatAWaitingMoveDoes};
use crate::service::caldav::how_many;
use crate::service::outward;

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

/// Who empties an account's Trash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhoEmptiesTheTrash {
    /// Nobody but this program, if somebody chooses a schedule.
    ThisProgram,
    /// Google, 30 days after a message goes into it.
    Gmail,
    /// Microsoft, as Outlook.com or an organisation sets it.
    Microsoft,
}

/// Who empties this account's Trash, from its protocol and the one check's
/// answer (D12).
///
/// POP first, emptied here whatever its provider, as `AccountKind::of` puts
/// POP first; then what 13-44.5's check says. Reads no address, server or
/// recorded name itself, so this can never disagree with the rest of the
/// program about who runs an account's mail.
pub fn who_empties_the_trash(
    protocol: Protocol,
    who_runs_it: WhoRunsTheMail,
) -> WhoEmptiesTheTrash {
    match (protocol, who_runs_it) {
        (Protocol::Pop3, _) | (Protocol::Imap, WhoRunsTheMail::SomebodyElse) => {
            WhoEmptiesTheTrash::ThisProgram
        }
        (Protocol::Imap, WhoRunsTheMail::Gmail) => WhoEmptiesTheTrash::Gmail,
        (Protocol::Imap, WhoRunsTheMail::Microsoft) => WhoEmptiesTheTrash::Microsoft,
    }
}

/// The line the account editor shows in place of the choice, for an
/// account whose provider empties its Trash itself (D2).
///
/// Google's from its help page 7401, read on 2026-09-29: "After 30 days:
/// The message is permanently deleted." Microsoft's from its page "Recover
/// and restore deleted items in Outlook", read on 2026-10-02: "Email is
/// automatically deleted from your Deleted Items folder after 30 days", in
/// its section on Outlook.com and Outlook on the web; an organisation's own
/// policy decides for a work or school account, as the same page says.
pub fn the_provider_empties_it(who: WhoEmptiesTheTrash) -> Option<&'static str> {
    match who {
        WhoEmptiesTheTrash::ThisProgram => None,
        WhoEmptiesTheTrash::Gmail => Some(
            "Gmail empties this account's Trash itself, 30 days after a message goes into it, \
             so Wixen Mail leaves it alone.",
        ),
        WhoEmptiesTheTrash::Microsoft => Some(
            "Microsoft empties this account's Deleted Items itself: Outlook.com after 30 days, \
             and a work or school account as its organisation has set it, so Wixen Mail leaves \
             it alone.",
        ),
    }
}

/// The most one emptying takes, the oldest first: the most a check keeps of
/// one folder (D14). What is due beyond it waits for the next day.
pub const AT_MOST_IN_A_DAY: usize = crate::application::mail_sync::INITIAL_FETCH_LIMIT;

/// What is said of an account none of whose folders is its Trash, in the
/// account editor and once a day at its check (D9).
pub const NO_TRASH_TO_EMPTY: &str = "This account does not say which of its folders it keeps \
     deleted mail in, so Wixen Mail cannot empty its Trash.";

/// The account an emptying is for.
#[derive(Debug, Clone, Copy)]
pub struct TheAccount<'a> {
    pub id: &'a str,
    /// What the sentence calls it.
    pub name: &'a str,
    /// Who empties its Trash, which decides whether this program does.
    pub who_empties: WhoEmptiesTheTrash,
}

/// What one emptying came to, counted for its sentence.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WhatTheEmptyingCameTo {
    /// Taken off the server.
    pub emptied: usize,
    /// Due and not tried, left for the next day's first check.
    pub left_for_another_day: usize,
    /// Made here and kept in the queue, because the server could not be
    /// reached; the next check's replay sends it.
    pub kept_waiting: usize,
    /// Put back in the Trash here, by why.
    pub put_back: BTreeMap<Because, usize>,
}

/// Whether an emptying goes on to the next message due.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Next {
    GoOn,
    /// The server could not be reached, so the rest are left untouched for
    /// the next day rather than queued for the next check to replay before
    /// it lists anything (D14).
    StopForToday,
}

impl WhatTheEmptyingCameTo {
    /// Count what one delete came to, and say whether to go on.
    fn count(&mut self, deleted: DeletedHereThenAtTheServer) -> Next {
        match deleted {
            DeletedHereThenAtTheServer::Became(ForTheChange::Done) => self.emptied += 1,
            DeletedHereThenAtTheServer::Became(ForTheChange::KeptWaiting(_)) => {
                self.kept_waiting += 1;
                return Next::StopForToday;
            }
            DeletedHereThenAtTheServer::Became(ForTheChange::PutBack(because)) => {
                *self.put_back.entry(because).or_default() += 1;
            }
            // Nothing changed here and nothing was sent, so it is due again
            // at the next day's first check.
            DeletedHereThenAtTheServer::NotMadeHere(not_made) => {
                tracing::warn!(
                    "A message due to be emptied from the Trash was left there: {not_made}"
                );
                self.left_for_another_day += 1;
            }
            // The server took it; only this computer's record of that did
            // not settle, and the next read of the Trash forgets the row.
            DeletedHereThenAtTheServer::NotSettledHere(why) => {
                tracing::warn!("A message emptied from the Trash was not settled here: {why}");
                self.emptied += 1;
            }
        }
        Next::GoOn
    }
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

/// The one sentence an emptying says for an account, or `None` when nothing
/// was due (D8). The folder's own name, so the sentence names what the tree
/// shows.
///
/// After the count, one clause for each other thing that happened, each with
/// its own count and the reason last, in the words a rule's changes use for
/// the same reasons (13-44.3), and no verb that has to agree with the count.
pub fn what_the_emptying_said(
    folder_name: &str,
    account_name: &str,
    days: i64,
    came_to: &WhatTheEmptyingCameTo,
) -> Option<String> {
    if *came_to == WhatTheEmptyingCameTo::default() {
        return None;
    }
    let emptied = format!(
        "Emptied {} from {folder_name} in {account_name} that had been there more than {days} days",
        how_many(came_to.emptied, "message")
    );
    let kept_waiting = (came_to.kept_waiting > 0).then(|| {
        format!(
            "{} kept waiting until {}",
            how_many(came_to.kept_waiting, "message"),
            Until::TheServerCanBeReached.in_words()
        )
    });
    let put_back = came_to.put_back.iter().map(|(because, count)| {
        format!(
            "{} put back because {}",
            how_many(*count, "message"),
            because.in_words()
        )
    });
    let left = (came_to.left_for_another_day > 0).then(|| {
        format!(
            "{} left for the next day",
            how_many(came_to.left_for_another_day, "more message")
        )
    });
    let clauses: Vec<String> = std::iter::once(emptied)
        .chain(kept_waiting)
        .chain(put_back)
        .chain(left)
        .collect();
    Some(format!("{}.", clauses.join("; ")))
}

/// What the check says when Allow Changes keeps this account's Trash as it
/// is, once a day. Worded here rather than taken from `outward::refusal`,
/// whose words are about sending and deleting at a key.
fn nothing_emptied_with_changes_off(folder_name: &str, account_name: &str) -> String {
    format!(
        "Nothing was emptied from {folder_name} in {account_name}, because {SETTINGS_SECTION} \
         does not let this account change mail."
    )
}

/// Empty this account's Trash of what is due, on the check's own session,
/// and answer the sentence to say, if any.
///
/// The Trash is the one the menu's Delete moves to, found the same way
/// (D9), and each message due is deleted inside it as the menu's Delete
/// there deletes it: off the server, through the step a rule's Delete
/// shares. Only messages stored here are taken (D15), oldest first, at most
/// [`AT_MOST_IN_A_DAY`].
///
/// Once a day per account, at its first check of `today` on this
/// computer's clock (D7). The day is used by an emptying, by nothing being
/// due, by finding no Trash and by meeting Allow Changes closed, so each is
/// said at most once a day; it is not used by an account this program does
/// not empty, nor by one that has never listed its folders, which says
/// nothing (D9).
pub(crate) async fn empty_at_a_check<S: ReplaysAMove>(
    server: &S,
    cache: &MessageCache,
    account: TheAccount<'_>,
    when: WhenTheTrashIsEmptied,
    allowed_mail: bool,
    now: DateTime<Utc>,
    today: NaiveDate,
) -> Result<Option<String>> {
    let Some(days) = when
        .days()
        .filter(|_| account.who_empties == WhoEmptiesTheTrash::ThisProgram)
    else {
        return Ok(None);
    };
    if cache.the_trash_was_last_emptied_on(account.id)? == Some(today) {
        return Ok(None);
    }
    let folders = cache.get_folders_for_account(account.id)?;
    let trash = match where_a_deleted_message_goes(
        folders.iter().map(|folder| {
            (
                folder.path.as_str(),
                FolderType::from_stored(&folder.folder_type),
            )
        }),
        "",
        Deleting::ToTrash,
    ) {
        DeletedGoesTo::NoFoldersKnownYet => return Ok(None),
        DeletedGoesTo::TheTrash(trash_path) => {
            folders.iter().find(|folder| folder.path == trash_path)
        }
        DeletedGoesTo::NoTrashFolderFound | DeletedGoesTo::OffTheServer => None,
    };
    cache.the_trash_was_emptied_on(account.id, today)?;
    let Some(trash) = trash else {
        return Ok(Some(format!(
            "Nothing was emptied in {}. {NO_TRASH_TO_EMPTY}",
            account.name
        )));
    };
    if outward::permitted(allowed_mail, "empty the Trash").is_err() {
        return Ok(Some(nothing_emptied_with_changes_off(
            &trash.name,
            account.name,
        )));
    }
    let due = what_is_due(&cache.what_has_been_in_the_trash(trash.id)?, days, now);
    let _under_way = APushUnderWay::begins(account.id);
    let mut came_to = WhatTheEmptyingCameTo::default();
    let mut tried = 0;
    for message in due.iter().take(AT_MOST_IN_A_DAY) {
        tried += 1;
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
        if came_to.count(deleted) == Next::StopForToday {
            break;
        }
    }
    came_to.left_for_another_day += due.len() - tried;
    Ok(what_the_emptying_said(
        &trash.name,
        account.name,
        days,
        &came_to,
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
        /// How every delete is answered, when it is not taken.
        refuses_a_delete_with: std::cell::Cell<Option<Refusal>>,
    }

    /// The ways a delete does not go.
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

    impl AServerThatHoldsTheTrash {
        /// A server holding these numbers in its Trash.
        fn holding_in_the_trash(uids: &[u32]) -> Self {
            let held = uids.iter().map(|uid| (*uid, the_id_of(*uid))).collect();
            Self {
                folders: RefCell::new(BTreeMap::from([("Trash".to_string(), held)])),
                log: RefCell::new(Vec::new()),
                refuses_a_delete_with: std::cell::Cell::new(None),
            }
        }

        fn refusing_a_delete(self, refusal: Refusal) -> Self {
            self.refuses_a_delete_with.set(Some(refusal));
            self
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
            if let Some(refusal) = self.refuses_a_delete_with.get() {
                return Err(refusal.as_error());
            }
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
            who_empties: WhoEmptiesTheTrash::ThisProgram,
        }
    }

    /// The day a case's first check runs on.
    fn the_first_day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 2).expect("a day")
    }

    /// One check of the account, with changes allowed, now, on the first day.
    fn a_check(
        server: &AServerThatHoldsTheTrash,
        cache: &MessageCache,
        when: WhenTheTrashIsEmptied,
    ) -> Option<String> {
        a_check_of(server, cache, work(), when, true, 0)
    }

    /// One check of an account, as many days after the first as `later`.
    fn a_check_of(
        server: &AServerThatHoldsTheTrash,
        cache: &MessageCache,
        account: TheAccount<'_>,
        when: WhenTheTrashIsEmptied,
        allowed_mail: bool,
        later: i64,
    ) -> Option<String> {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
            .block_on(empty_at_a_check(
                server,
                cache,
                account,
                when,
                allowed_mail,
                Utc::now() + Duration::days(later),
                the_first_day() + Duration::days(later),
            ))
            .expect("the emptying to finish")
    }

    fn is_waiting(cache: &MessageCache, row: i64) -> bool {
        cache
            .the_move_waiting_for(row)
            .expect("the store read")
            .is_some()
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

    /// An emptying that took this many and nothing else happened.
    fn only_emptied(emptied: usize) -> WhatTheEmptyingCameTo {
        WhatTheEmptyingCameTo {
            emptied,
            ..WhatTheEmptyingCameTo::default()
        }
    }

    #[test]
    fn test_one_message_is_said_in_the_singular() {
        assert_eq!(
            what_the_emptying_said("Deleted Items", "Home", 15, &only_emptied(1)).as_deref(),
            Some(
                "Emptied 1 message from Deleted Items in Home that had been there more than 15 days."
            )
        );
        assert_eq!(
            what_the_emptying_said("Trash", "Home", 15, &only_emptied(0)),
            None
        );
    }

    /// The day this account's Trash was last emptied, as the store says.
    fn last_emptied(cache: &MessageCache) -> Option<NaiveDate> {
        cache
            .the_trash_was_last_emptied_on(THE_ACCOUNT)
            .expect("the day read")
    }

    /// A cache holding the account's Inbox and no folder it keeps deleted
    /// mail in.
    fn an_account_without_a_trash() -> (tempfile::TempDir, MessageCache) {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
        a_folder(&cache, "Inbox", "INBOX", FolderType::Inbox);
        (dir, cache)
    }

    /// A message stored in the Trash that went in there this long ago.
    fn in_the_trash_ago(cache: &MessageCache, trash: i64, uid: u32, ago: Duration) -> i64 {
        let row = in_the_trash_for(cache, trash, uid, 0);
        cache
            .it_went_into_the_trash_at(row, Utc::now() - ago)
            .expect("the stamp set");
        row
    }

    const THIRTY: WhenTheTrashIsEmptied = WhenTheTrashIsEmptied::After30Days;

    #[test]
    fn test_a_second_check_the_same_day_empties_nothing_more() {
        let (_dir, cache, trash) = an_account();
        let server = AServerThatHoldsTheTrash::holding_in_the_trash(&[1, 2]);
        in_the_trash_for(&cache, trash, 1, 45);
        assert!(a_check(&server, &cache, THIRTY).is_some());

        let second = in_the_trash_for(&cache, trash, 2, 45);
        assert_eq!(a_check(&server, &cache, THIRTY), None);

        assert_eq!(server.the_log(), ["DELETE 1 in Trash off the server"]);
        assert!(!is_deleted_here(&cache, second));
        assert_eq!(last_emptied(&cache), Some(the_first_day()));
    }

    #[test]
    fn test_the_next_day_empties_what_has_come_due_since() {
        let (_dir, cache, trash) = an_account();
        let server = AServerThatHoldsTheTrash::holding_in_the_trash(&[1, 2]);
        in_the_trash_for(&cache, trash, 1, 45);
        let a_day_short = in_the_trash_ago(&cache, trash, 2, Duration::hours(29 * 24 + 12));
        a_check(&server, &cache, THIRTY);
        assert!(!is_deleted_here(&cache, a_day_short), "emptied a day early");

        let said = a_check_of(&server, &cache, work(), THIRTY, true, 1);

        assert_eq!(
            server.the_log(),
            [
                "DELETE 1 in Trash off the server",
                "DELETE 2 in Trash off the server"
            ]
        );
        assert_eq!(
            said.as_deref(),
            Some("Emptied 1 message from Trash in Work that had been there more than 30 days.")
        );
        assert_eq!(
            last_emptied(&cache),
            Some(the_first_day() + Duration::days(1))
        );
    }

    #[test]
    fn test_with_changing_mail_off_nothing_is_dialled_and_it_is_said_once_a_day() {
        let (_dir, cache, trash) = an_account();
        let server = AServerThatHoldsTheTrash::holding_in_the_trash(&[1]);
        let row = in_the_trash_for(&cache, trash, 1, 45);

        let said = a_check_of(&server, &cache, work(), THIRTY, false, 0);
        assert_eq!(
            said.as_deref(),
            Some(
                "Nothing was emptied from Trash in Work, because Allow Changes does not let this \
                 account change mail."
            )
        );
        assert_eq!(a_check_of(&server, &cache, work(), THIRTY, false, 0), None);
        assert!(server.the_log().is_empty(), "{:?}", server.the_log());
        assert!(!is_deleted_here(&cache, row));

        // Allowed the next day, it empties at that day's first check.
        assert!(a_check_of(&server, &cache, work(), THIRTY, true, 1).is_some());
        assert_eq!(server.the_log(), ["DELETE 1 in Trash off the server"]);
    }

    #[test]
    fn test_an_account_with_no_trash_is_left_alone_and_says_so_once_a_day() {
        let (_dir, cache) = an_account_without_a_trash();
        let server = AServerThatHoldsTheTrash::holding_in_the_trash(&[]);

        let said = a_check(&server, &cache, THIRTY);
        assert_eq!(
            said,
            Some(format!("Nothing was emptied in Work. {NO_TRASH_TO_EMPTY}"))
        );
        assert_eq!(a_check(&server, &cache, THIRTY), None);
        assert!(server.the_log().is_empty(), "{:?}", server.the_log());
    }

    #[test]
    fn test_an_account_that_has_never_listed_its_folders_says_nothing() {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
        let server = AServerThatHoldsTheTrash::holding_in_the_trash(&[1]);

        assert_eq!(a_check(&server, &cache, THIRTY), None);
        assert_eq!(
            last_emptied(&cache),
            None,
            "a day was used with nothing known"
        );

        // Its folders listed later the same day, the day is still there to use.
        a_folder(&cache, "Inbox", "INBOX", FolderType::Inbox);
        let trash = a_folder(&cache, "Trash", "Trash", FolderType::Trash);
        in_the_trash_for(&cache, trash, 1, 45);
        assert_eq!(
            a_check(&server, &cache, THIRTY).as_deref(),
            Some("Emptied 1 message from Trash in Work that had been there more than 30 days.")
        );
        assert_eq!(last_emptied(&cache), Some(the_first_day()));
    }

    #[test]
    fn test_no_more_than_five_hundred_go_in_a_day_and_the_rest_are_said_to_be_left() {
        let (_dir, cache, trash) = an_account();
        let uids: Vec<u32> = (1..=501).collect();
        let server = AServerThatHoldsTheTrash::holding_in_the_trash(&uids);
        // The first is the oldest, so the last is the one left.
        let rows: Vec<i64> = uids
            .iter()
            .map(|uid| in_the_trash_for(&cache, trash, *uid, 31 + 501 - i64::from(*uid)))
            .collect();

        let said = a_check(&server, &cache, THIRTY);

        let log = server.the_log();
        assert_eq!(log.len(), 500);
        assert_eq!(
            log.first().map(String::as_str),
            Some("DELETE 1 in Trash off the server")
        );
        assert_eq!(
            log.last().map(String::as_str),
            Some("DELETE 500 in Trash off the server")
        );
        assert!(!is_deleted_here(&cache, rows[500]));
        assert_eq!(
            said.as_deref(),
            Some(
                "Emptied 500 messages from Trash in Work that had been there more than 30 days; \
                 1 more message left for the next day."
            )
        );
    }

    #[test]
    fn test_a_server_that_cannot_be_reached_leaves_one_waiting_and_the_rest_untouched() {
        let (_dir, cache, trash) = an_account();
        let server = AServerThatHoldsTheTrash::holding_in_the_trash(&[1, 2, 3])
            .refusing_a_delete(Refusal::Dropped);
        let first = in_the_trash_for(&cache, trash, 1, 50);
        let rest = [
            in_the_trash_for(&cache, trash, 2, 40),
            in_the_trash_for(&cache, trash, 3, 35),
        ];

        let said = a_check(&server, &cache, THIRTY);

        assert_eq!(server.the_log(), ["DELETE 1 in Trash off the server"]);
        assert!(is_deleted_here(&cache, first) && is_waiting(&cache, first));
        for row in rest {
            assert!(!is_deleted_here(&cache, row) && !is_waiting(&cache, row));
        }
        assert_eq!(
            said.as_deref(),
            Some(
                "Emptied 0 messages from Trash in Work that had been there more than 30 days; \
                 1 message kept waiting until the mail server can be reached; 2 more messages \
                 left for the next day."
            )
        );
    }

    #[test]
    fn test_a_delete_the_server_refuses_is_put_back_in_the_trash() {
        let (_dir, cache, trash) = an_account();
        let server = AServerThatHoldsTheTrash::holding_in_the_trash(&[1, 2])
            .refusing_a_delete(Refusal::SaidNo);
        let rows = [
            in_the_trash_for(&cache, trash, 1, 50),
            in_the_trash_for(&cache, trash, 2, 40),
        ];

        let said = a_check(&server, &cache, THIRTY);

        assert_eq!(
            server.the_log(),
            [
                "DELETE 1 in Trash off the server",
                "DELETE 2 in Trash off the server"
            ]
        );
        for row in rows {
            assert!(!is_deleted_here(&cache, row) && !is_waiting(&cache, row));
            assert!(
                cache.in_the_trash_since(row).expect("the stamp").is_some(),
                "row {row} is no longer in the Trash here"
            );
        }
        assert_eq!(
            said.as_deref(),
            Some(
                "Emptied 0 messages from Trash in Work that had been there more than 30 days; \
                 2 messages put back because the mail server said no."
            )
        );
    }

    #[test]
    fn test_this_computers_gate_refusing_at_the_session_puts_it_back_and_says_so() {
        let (_dir, cache, trash) = an_account();
        let server = AServerThatHoldsTheTrash::holding_in_the_trash(&[1])
            .refusing_a_delete(Refusal::TheGate);
        let row = in_the_trash_for(&cache, trash, 1, 50);

        let said = a_check(&server, &cache, THIRTY);

        assert!(!is_deleted_here(&cache, row) && !is_waiting(&cache, row));
        assert_eq!(
            said.as_deref(),
            Some(
                "Emptied 0 messages from Trash in Work that had been there more than 30 days; \
                 1 message put back because changing mail is not allowed."
            )
        );
    }

    #[test]
    fn test_a_message_whose_delete_is_still_waiting_is_left_for_another_day() {
        let (_dir, cache, trash) = an_account();
        let server = AServerThatHoldsTheTrash::holding_in_the_trash(&[1, 2]);
        in_the_trash_for(&cache, trash, 1, 50);
        let waiting = in_the_trash_for(&cache, trash, 2, 40);
        cache
            .keep_a_move_waiting(&AWaitingMove {
                message_row_id: waiting,
                account_id: THE_ACCOUNT.to_string(),
                from_folder_path: "Trash".to_string(),
                uid: 2,
                what: WhatAWaitingMoveDoes::DeleteOutright,
                asked_at: Utc::now().to_rfc3339(),
            })
            .expect("a delete kept waiting");

        let said = a_check(&server, &cache, THIRTY);

        assert_eq!(server.the_log(), ["DELETE 1 in Trash off the server"]);
        assert!(
            is_waiting(&cache, waiting),
            "the waiting delete was settled here"
        );
        assert_eq!(
            said.as_deref(),
            Some("Emptied 1 message from Trash in Work that had been there more than 30 days.")
        );
    }

    #[test]
    fn test_an_account_whose_provider_empties_its_trash_is_left_alone() {
        for who in [WhoEmptiesTheTrash::Gmail, WhoEmptiesTheTrash::Microsoft] {
            let (_dir, cache, trash) = an_account();
            let server = AServerThatHoldsTheTrash::holding_in_the_trash(&[1]);
            let row = in_the_trash_for(&cache, trash, 1, 45);
            let account = TheAccount {
                who_empties: who,
                ..work()
            };

            assert_eq!(a_check_of(&server, &cache, account, THIRTY, true, 0), None);

            assert!(
                server.the_log().is_empty(),
                "{who:?}: {:?}",
                server.the_log()
            );
            assert!(!is_deleted_here(&cache, row), "{who:?}");
            assert_eq!(last_emptied(&cache), None, "{who:?} used a day");
        }
    }

    #[test]
    fn test_each_answer_of_the_one_check_says_who_empties_the_trash() {
        for (runs_it, empties_it, line) in [
            (
                WhoRunsTheMail::Gmail,
                WhoEmptiesTheTrash::Gmail,
                Some(
                    "Gmail empties this account's Trash itself, 30 days after a message goes \
                     into it, so Wixen Mail leaves it alone.",
                ),
            ),
            (
                WhoRunsTheMail::Microsoft,
                WhoEmptiesTheTrash::Microsoft,
                Some(
                    "Microsoft empties this account's Deleted Items itself: Outlook.com after 30 \
                     days, and a work or school account as its organisation has set it, so Wixen \
                     Mail leaves it alone.",
                ),
            ),
            (
                WhoRunsTheMail::SomebodyElse,
                WhoEmptiesTheTrash::ThisProgram,
                None,
            ),
        ] {
            let who = who_empties_the_trash(Protocol::Imap, runs_it);
            assert_eq!(who, empties_it, "{runs_it:?}");
            assert_eq!(the_provider_empties_it(who), line, "{who:?}");
        }
    }

    #[test]
    fn test_pop_is_emptied_here_whatever_its_provider() {
        for runs_it in [
            WhoRunsTheMail::Gmail,
            WhoRunsTheMail::Microsoft,
            WhoRunsTheMail::SomebodyElse,
        ] {
            assert_eq!(
                who_empties_the_trash(Protocol::Pop3, runs_it),
                WhoEmptiesTheTrash::ThisProgram,
                "{runs_it:?}"
            );
        }
    }

    #[test]
    fn test_the_sentence_says_each_count_and_reason_once_and_the_singular_right() {
        let everything = WhatTheEmptyingCameTo {
            emptied: 1,
            left_for_another_day: 1,
            kept_waiting: 1,
            put_back: BTreeMap::from([
                (Because::TheServerSaidNo, 1),
                (Because::ChangingMailIsNotAllowed, 2),
            ]),
        };
        assert_eq!(
            what_the_emptying_said("Trash", "Work", 30, &everything).as_deref(),
            Some(
                "Emptied 1 message from Trash in Work that had been there more than 30 days; \
                 1 message kept waiting until the mail server can be reached; 2 messages put \
                 back because changing mail is not allowed; 1 message put back because the mail \
                 server said no; 1 more message left for the next day."
            )
        );
        let many = WhatTheEmptyingCameTo {
            emptied: 3,
            left_for_another_day: 12,
            ..WhatTheEmptyingCameTo::default()
        };
        assert_eq!(
            what_the_emptying_said("Deleted Items", "Home", 15, &many).as_deref(),
            Some(
                "Emptied 3 messages from Deleted Items in Home that had been there more than 15 \
                 days; 12 more messages left for the next day."
            )
        );
        assert_eq!(
            what_the_emptying_said("Trash", "Work", 30, &WhatTheEmptyingCameTo::default()),
            None
        );
    }
}

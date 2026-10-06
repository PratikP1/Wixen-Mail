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
use crate::application::local_delete;
use crate::application::local_folders::{self, LocalDelete};
use crate::application::moves_waiting::{
    APushUnderWay, DeletedHereThenAtTheServer, ReplaysAMove, delete_here_then_at_the_server,
};
use crate::application::what_rules_tell_the_server::{Because, ForTheChange, Until};
use crate::application::who_runs_the_mail::WhoRunsTheMail;
use crate::common::Result;
use crate::common::types::{FolderType, Protocol};
use crate::data::account::Account;
use crate::data::message_cache::in_the_trash::InTheTrash;
use crate::data::message_cache::moves_waiting::{AWaitingMove, WhatAWaitingMoveDoes};
use crate::data::message_cache::{CachedFolder, MessageCache};
use crate::service::caldav::how_many;
use crate::service::outward;

/// When an account's Trash is emptied, as the account editor offers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WhenTheTrashIsEmptied {
    /// Never, the answer for every account until somebody chooses, because
    /// emptying the Trash cannot be undone (D1).
    #[default]
    Never,
    /// Everything in the Trash, as Wixen Mail really closes, within a few
    /// seconds, and never at a check (13-44.7, D19 and D23).
    WhenWixenMailCloses,
    /// What went into the Trash here more than 15 days ago.
    After15Days,
    /// What went into the Trash here more than 30 days ago.
    After30Days,
}

impl WhenTheTrashIsEmptied {
    /// Every answer, in the order the choice offers them.
    pub const ALL: [Self; 4] = [
        Self::Never,
        Self::WhenWixenMailCloses,
        Self::After15Days,
        Self::After30Days,
    ];

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
            Self::WhenWixenMailCloses => "when_wixen_mail_closes",
            Self::After15Days => "after_15_days",
            Self::After30Days => "after_30_days",
        }
    }

    /// The words the choice shows.
    pub fn said(self) -> &'static str {
        match self {
            Self::Never => "Never",
            Self::WhenWixenMailCloses => "When Wixen Mail closes",
            Self::After15Days => "After 15 days",
            Self::After30Days => "After 30 days",
        }
    }

    /// How many days a message stays in the Trash before it is due, or
    /// `None` for an answer that empties nothing on a schedule.
    pub fn days(self) -> Option<i64> {
        match self {
            Self::Never | Self::WhenWixenMailCloses => None,
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
    the_emptying_in_words(
        &format!("from {folder_name} in {account_name}"),
        days,
        came_to,
    )
}

/// The one sentence a POP account's emptying says, naming the account and
/// saying the Trash is the one on this computer (13-44.7, D5).
fn what_emptying_here_said(
    account_name: &str,
    days: i64,
    came_to: &WhatTheEmptyingCameTo,
) -> Option<String> {
    the_emptying_in_words(
        &format!("of {account_name}'s from the Trash on this computer"),
        days,
        came_to,
    )
}

/// The sentence, with where the messages were emptied from in words.
fn the_emptying_in_words(
    emptied_from: &str,
    days: i64,
    came_to: &WhatTheEmptyingCameTo,
) -> Option<String> {
    if *came_to == WhatTheEmptyingCameTo::default() {
        return None;
    }
    let emptied = format!(
        "Emptied {} {emptied_from} that had been there more than {days} days",
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

/// Where an account keeps deleted mail on this computer, as the menu's
/// Delete finds it (D9).
enum TheTrashHere {
    /// The account has never listed its folders.
    NotKnownYet,
    /// None of its folders is the one deleted mail goes to.
    NotRecognised,
    Found(CachedFolder),
}

/// Find this account's Trash the way the menu's Delete finds it.
fn the_trash_of(cache: &MessageCache, account_id: &str) -> Result<TheTrashHere> {
    let folders = cache.get_folders_for_account(account_id)?;
    let goes_to = where_a_deleted_message_goes(
        folders.iter().map(|folder| {
            (
                folder.path.as_str(),
                FolderType::from_stored(&folder.folder_type),
            )
        }),
        "",
        Deleting::ToTrash,
    );
    Ok(match goes_to {
        DeletedGoesTo::NoFoldersKnownYet => TheTrashHere::NotKnownYet,
        DeletedGoesTo::TheTrash(trash_path) => folders
            .iter()
            .find(|folder| folder.path == trash_path)
            .cloned()
            .map_or(TheTrashHere::NotRecognised, TheTrashHere::Found),
        DeletedGoesTo::NoTrashFolderFound | DeletedGoesTo::OffTheServer => {
            TheTrashHere::NotRecognised
        }
    })
}

/// What a check says of an account set to When Wixen Mail closes whose
/// Trash the close will not empty, or `None` when the close will (D26).
/// Worded as what will not happen, because nothing is said at close and
/// silence would read as working.
fn why_closing_will_not_empty_it(
    trash: Option<&CachedFolder>,
    account_name: &str,
    allowed_mail: bool,
) -> Option<String> {
    match trash {
        None => Some(format!(
            "Nothing will be emptied in {account_name} when Wixen Mail closes. {NO_TRASH_TO_EMPTY}"
        )),
        Some(trash) if outward::permitted(allowed_mail, "empty the Trash").is_err() => {
            Some(format!(
                "Nothing will be emptied from {} in {account_name} when Wixen Mail closes, \
                 because {SETTINGS_SECTION} does not let this account change mail.",
                trash.name
            ))
        }
        Some(_) => None,
    }
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
///
/// An account set to When Wixen Mail closes is never emptied here (13-44.7,
/// D23). Nothing is said at close, so its check says, once a day, what will
/// stop the close emptying it: no Trash recognised, or Allow Changes closed
/// (D26). With neither, it says nothing and uses no day.
pub(crate) async fn empty_at_a_check<S: ReplaysAMove>(
    server: &S,
    cache: &MessageCache,
    account: TheAccount<'_>,
    when: WhenTheTrashIsEmptied,
    allowed_mail: bool,
    now: DateTime<Utc>,
    today: NaiveDate,
) -> Result<Option<String>> {
    if when == WhenTheTrashIsEmptied::Never
        || account.who_empties != WhoEmptiesTheTrash::ThisProgram
    {
        return Ok(None);
    }
    if cache.the_trash_was_last_emptied_on(account.id)? == Some(today) {
        return Ok(None);
    }
    let trash = match the_trash_of(cache, account.id)? {
        TheTrashHere::NotKnownYet => return Ok(None),
        TheTrashHere::NotRecognised => None,
        TheTrashHere::Found(trash) => Some(trash),
    };
    let Some(days) = when.days() else {
        let why = why_closing_will_not_empty_it(trash.as_ref(), account.name, allowed_mail);
        if why.is_some() {
            cache.the_trash_was_emptied_on(account.id, today)?;
        }
        return Ok(why);
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

// ── A POP account's Trash, on this computer (13-44.7) ──────────────────────

/// Empty a POP account's Trash on this computer of its own messages that are
/// due, at the start of its check, before the POP server is dialled, and
/// answer the sentence to say, if any (D5, D22).
///
/// The Trash is the one every account shares, where Delete moves a POP
/// message, and only what this account put there is taken (D18). Each
/// message goes through [`local_delete::perform`], the delete Empty Folder
/// uses there: marked deleted here, keeping its identifier and its download
/// time, so the next check does not download it again and the account's own
/// removal setting still counts from it. The POP server is never asked. A
/// refusal, "Let me delete mail on this computer" off, ends it with one
/// sentence. Once a day and at most [`AT_MOST_IN_A_DAY`], as an IMAP
/// account's check is.
///
/// Set to When Wixen Mail closes, a check empties nothing and says, once a
/// day, only what will stop the close emptying it (D23, D26).
pub fn empty_the_trash_here_at_a_pop_check(
    cache: &MessageCache,
    account: &Account,
    when: WhenTheTrashIsEmptied,
    now: DateTime<Utc>,
    today: NaiveDate,
) -> Result<Option<String>> {
    if when == WhenTheTrashIsEmptied::Never {
        return Ok(None);
    }
    if cache.the_trash_was_last_emptied_on(&account.id)? == Some(today) {
        return Ok(None);
    }
    let Some(trash) = the_trash_here_of(cache, account)? else {
        return Ok(None);
    };
    let Some(days) = when.days() else {
        let why = why_closing_will_not_empty_the_trash_here(&trash, account);
        if why.is_some() {
            cache.the_trash_was_emptied_on(&account.id, today)?;
        }
        return Ok(why);
    };
    cache.the_trash_was_emptied_on(&account.id, today)?;
    let due = what_is_due(
        &cache.what_this_account_put_in_the_shared_trash(trash.id, &account.id)?,
        days,
        now,
    );
    let mut came_to = WhatTheEmptyingCameTo::default();
    let mut tried = 0;
    for message in due.iter().take(AT_MOST_IN_A_DAY) {
        tried += 1;
        match take_it_off_this_computer(cache, account, message.row)? {
            TakenHere::Emptied => came_to.emptied += 1,
            TakenHere::Refused(why) => {
                return Ok(Some(format!(
                    "Nothing was emptied from the Trash on this computer for {}. {why}",
                    account.name
                )));
            }
            TakenHere::NotOnThisComputer => came_to.left_for_another_day += 1,
        }
    }
    came_to.left_for_another_day += due.len() - tried;
    Ok(what_emptying_here_said(&account.name, days, &came_to))
}

/// The Trash on this computer an account's deleted mail goes to, the one
/// every account shares, or `None` when it keeps none here or it has not
/// been made yet.
fn the_trash_here_of(cache: &MessageCache, account: &Account) -> Result<Option<CachedFolder>> {
    let Some(path) = local_folders::local_trash(account.protocol()) else {
        return Ok(None);
    };
    cache.get_folder(local_folders::stored_under(&path, &account.id), &path)
}

/// What one delete on this computer came to.
enum TakenHere {
    Emptied,
    /// Refused, with the words the refusal gave.
    Refused(String),
    /// Not a folder on this computer after all, so nothing was done.
    NotOnThisComputer,
}

/// Take one message out of a Trash on this computer through the delete
/// Empty Folder uses, deciding nothing it decides.
fn take_it_off_this_computer(
    cache: &MessageCache,
    account: &Account,
    message_row_id: i64,
) -> Result<TakenHere> {
    Ok(
        match local_delete::perform(cache, account, message_row_id, Deleting::ToTrash)? {
            Some(outcome) if outcome.message_left_the_folder => TakenHere::Emptied,
            Some(refused) => TakenHere::Refused(refused.said),
            None => TakenHere::NotOnThisComputer,
        },
    )
}

/// What a check says of a POP account set to When Wixen Mail closes whose
/// Trash on this computer the close will not empty, in the words the delete
/// itself refuses with (D26).
fn why_closing_will_not_empty_the_trash_here(
    trash: &CachedFolder,
    account: &Account,
) -> Option<String> {
    match local_folders::deleting(
        &trash.path,
        account.protocol(),
        Deleting::ToTrash,
        account.allow_deleting_here,
    ) {
        Some(LocalDelete::Refuse(why)) => Some(format!(
            "Nothing will be emptied from the Trash on this computer for {} when Wixen Mail \
             closes. {why}",
            account.name
        )),
        _ => None,
    }
}

// ── When Wixen Mail closes (13-44.7) ───────────────────────────────────────

/// What opens the session an account's Trash is emptied on as Wixen Mail
/// closes.
///
/// A seam of its own rather than `moves_waiting::OpensASession`, whose
/// session has to answer for a crossing between two accounts as well; an
/// emptying asks only what [`ReplaysAMove`] asks. The program's answer is
/// [`crate::application::mail_session::TheAccountsSetUpHere`], signing in
/// through `mail_session::the_session_at`. The session is whatever holds
/// one, the shared session the program keeps per account.
pub(crate) trait OpensTheSessionToEmpty {
    type Session: std::ops::Deref<Target: ReplaysAMove + Sized>;
    async fn session_for(&self, account_id: &str) -> Result<Self::Session>;
}

impl OpensTheSessionToEmpty for crate::application::mail_session::TheAccountsSetUpHere<'_> {
    type Session = std::sync::Arc<crate::application::mail_controller::MailController>;

    async fn session_for(&self, account_id: &str) -> Result<Self::Session> {
        let account = self
            .0
            .iter()
            .find(|account| account.id == account_id)
            .ok_or_else(|| {
                crate::common::Error::Other(format!(
                    "The account {account_id} is no longer set up here"
                ))
            })?;
        crate::application::mail_session::the_session_at(account).await
    }
}

/// An account as the close sees it: what was chosen for its Trash and what
/// lets it be emptied.
#[derive(Debug, Clone, Copy)]
pub struct AnAccountToEmpty<'a> {
    /// The account as it is set up here, whole, because a POP account's
    /// Trash is emptied through the delete Empty Folder uses, which reads
    /// its protocol and whether it may delete mail on this computer.
    pub account: &'a Account,
    /// What the account editor stored for its Trash.
    pub answer: WhenTheTrashIsEmptied,
    /// Who empties its Trash, from the one check.
    pub who_empties: WhoEmptiesTheTrash,
    /// Whether Allow Changes lets this account change mail.
    pub allowed_mail: bool,
}

/// What closing did with one account's Trash, for the log (D21).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatTheCloseDid {
    pub account_name: String,
    /// Taken off the server, or off this computer for POP.
    pub emptied: usize,
    /// Made here and kept in the queue, for the next start's first check.
    pub waiting: usize,
    /// Still in the Trash, untouched, for the next close.
    pub left: usize,
}

/// The log's line for what closing did with one account's Trash: the
/// account and three counts, never a subject (D21).
pub fn what_the_close_did_for_the_log(did: &WhatTheCloseDid) -> String {
    format!(
        "On the way out, the Trash in {}: {} emptied, {} waiting for the next check, {} left for \
         the next close.",
        did.account_name, did.emptied, did.waiting, did.left
    )
}

impl WhatTheCloseDid {
    /// Counted from what the deletes came to, out of what the Trash held:
    /// whatever was neither emptied nor kept waiting is still there.
    fn of(account_name: &str, in_the_trash: usize, came_to: &WhatTheEmptyingCameTo) -> Self {
        Self {
            account_name: account_name.to_string(),
            emptied: came_to.emptied,
            waiting: came_to.kept_waiting,
            left: in_the_trash.saturating_sub(came_to.emptied + came_to.kept_waiting),
        }
    }
}

impl AnAccountToEmpty<'_> {
    /// Whether closing empties this account's Trash: set to When Wixen Mail
    /// closes, a Trash this program empties, and, for a Trash at a server,
    /// Allow Changes letting it change mail (D23, D26). A POP account's
    /// Trash is on this computer and its own delete answers for it. A
    /// refusal is not said here; the account's check says it.
    fn is_emptied_on_the_way_out(&self) -> bool {
        self.answer == WhenTheTrashIsEmptied::WhenWixenMailCloses
            && self.who_empties == WhoEmptiesTheTrash::ThisProgram
            && (self.is_emptied_here()
                || outward::permitted(self.allowed_mail, "empty the Trash").is_ok())
    }

    /// Whether its Trash is on this computer, which is a POP account's.
    fn is_emptied_here(&self) -> bool {
        self.account.protocol() == Protocol::Pop3
    }
}

/// What one account's emptying at close came to, for the log, with a
/// failure written there rather than ending the close.
fn the_close_did(
    account: &Account,
    emptying: Result<Option<WhatTheCloseDid>>,
) -> Option<WhatTheCloseDid> {
    emptying
        .inspect_err(|why| {
            tracing::warn!(
                "The Trash of {} could not be emptied on the way out: {why}",
                account.name
            );
        })
        .ok()
        .flatten()
}

/// Empty, as Wixen Mail closes, the Trash of every account set to When Wixen
/// Mail closes, returning by `deadline` whatever the servers do (D19, D20).
///
/// One deadline for every account together, read before each message and
/// bounding every wait, so a server that has gone quiet costs the limit and
/// no more. Each message is made here and sent before the next is touched:
/// at the limit the one in flight is left waiting in the store the next
/// start's first check replays before it lists anything, and the rest stay
/// in the Trash, untouched, for the next close. One answer per account
/// whose Trash held something, for the log; nothing is said (D21).
///
/// POP accounts first, since their Trash is on this computer and needs no
/// network, so a quiet server cannot spend the limit before them (D22).
pub(crate) async fn empty_on_the_way_out<O: OpensTheSessionToEmpty>(
    opener: &O,
    cache: &MessageCache,
    accounts: &[AnAccountToEmpty<'_>],
    deadline: tokio::time::Instant,
) -> Vec<WhatTheCloseDid> {
    let (here, at_a_server): (Vec<&AnAccountToEmpty<'_>>, Vec<&AnAccountToEmpty<'_>>) = accounts
        .iter()
        .filter(|to_empty| to_empty.is_emptied_on_the_way_out())
        .partition(|to_empty| to_empty.is_emptied_here());
    let mut did: Vec<WhatTheCloseDid> = here
        .into_iter()
        .filter_map(|to_empty| {
            the_close_did(
                to_empty.account,
                empty_one_trash_here_on_the_way_out(cache, to_empty.account, deadline),
            )
        })
        .collect();
    for to_empty in at_a_server {
        let emptying =
            empty_one_imap_trash_on_the_way_out(opener, cache, to_empty.account, deadline).await;
        did.extend(the_close_did(to_empty.account, emptying));
    }
    did
}

/// One POP account's Trash on this computer, of its own messages, emptied
/// through the delete Empty Folder uses, the deadline read before each.
fn empty_one_trash_here_on_the_way_out(
    cache: &MessageCache,
    account: &Account,
    deadline: tokio::time::Instant,
) -> Result<Option<WhatTheCloseDid>> {
    let Some(trash) = the_trash_here_of(cache, account)? else {
        return Ok(None);
    };
    let in_the_trash = cache.what_this_account_put_in_the_shared_trash(trash.id, &account.id)?;
    if in_the_trash.is_empty() {
        return Ok(None);
    }
    let mut came_to = WhatTheEmptyingCameTo::default();
    for message in &in_the_trash {
        if tokio::time::Instant::now() >= deadline {
            break;
        }
        match take_it_off_this_computer(cache, account, message.row)? {
            TakenHere::Emptied => came_to.emptied += 1,
            // Said at the account's check, once a day (D26).
            TakenHere::Refused(_) => break,
            TakenHere::NotOnThisComputer => {}
        }
    }
    Ok(Some(WhatTheCloseDid::of(
        &account.name,
        in_the_trash.len(),
        &came_to,
    )))
}

/// One IMAP account's Trash, emptied at the server within the deadline,
/// each message through the step the menu's Delete in the Trash takes. An
/// account with nothing there opens no session.
async fn empty_one_imap_trash_on_the_way_out<O: OpensTheSessionToEmpty>(
    opener: &O,
    cache: &MessageCache,
    account: &Account,
    deadline: tokio::time::Instant,
) -> Result<Option<WhatTheCloseDid>> {
    let TheTrashHere::Found(trash) = the_trash_of(cache, &account.id)? else {
        return Ok(None);
    };
    let in_the_trash = cache.what_has_been_in_the_trash(trash.id)?;
    if in_the_trash.is_empty() {
        return Ok(None);
    }
    let mut came_to = WhatTheEmptyingCameTo::default();
    match tokio::time::timeout_at(deadline, opener.session_for(&account.id)).await {
        Ok(Ok(session)) => {
            let _under_way = APushUnderWay::begins(&account.id);
            for message in &in_the_trash {
                if tokio::time::Instant::now() >= deadline {
                    break;
                }
                let asked = AWaitingMove {
                    message_row_id: message.row,
                    account_id: account.id.clone(),
                    from_folder_path: trash.path.clone(),
                    uid: message.uid,
                    what: WhatAWaitingMoveDoes::DeleteOutright,
                    asked_at: Utc::now().to_rfc3339(),
                };
                let sent = tokio::time::timeout_at(
                    deadline,
                    delete_here_then_at_the_server(&*session, cache, &asked, ""),
                );
                match sent.await {
                    Ok(deleted) => {
                        if came_to.count(deleted) == Next::StopForToday {
                            break;
                        }
                    }
                    // Made here and kept in the store, and the server not
                    // heard from by the limit (D20).
                    Err(_at_the_limit) => {
                        came_to.kept_waiting += 1;
                        break;
                    }
                }
            }
        }
        Ok(Err(why)) => tracing::warn!(
            "{} could not be signed in to on the way out: {why}",
            account.name
        ),
        Err(_at_the_limit) => tracing::warn!(
            "{} was not signed in to before the limit on the way out",
            account.name
        ),
    }
    Ok(Some(WhatTheCloseDid::of(
        &account.name,
        in_the_trash.len(),
        &came_to,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::message_cache::moves_waiting::MarksFirst;
    use crate::data::message_cache::{CachedFolder, CachedMessage};
    use local_folders::DELETING_IS_SWITCHED_OFF;
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
        /// The number whose delete is never answered, as a server that has
        /// taken the command and gone quiet.
        never_answers_the_delete_of: std::cell::Cell<Option<u32>>,
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
                never_answers_the_delete_of: std::cell::Cell::new(None),
            }
        }

        fn refusing_a_delete(self, refusal: Refusal) -> Self {
            self.refuses_a_delete_with.set(Some(refusal));
            self
        }

        fn never_answering_the_delete_of(self, uid: u32) -> Self {
            self.never_answers_the_delete_of.set(Some(uid));
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
        async fn move_it(&self, from: &str, uid: u32, into: &str) -> Result<String> {
            self.log
                .borrow_mut()
                .push(format!("MOVE {uid} in {from} into {into}"));
            Ok(format!("Moved to {into}"))
        }

        async fn delete_it(&self, folder: &str, uid: u32, trash: Option<&str>) -> Result<String> {
            self.log.borrow_mut().push(match trash {
                Some(trash) => format!("DELETE {uid} in {folder} into {trash}"),
                None => format!("DELETE {uid} in {folder} off the server"),
            });
            if self.never_answers_the_delete_of.get() == Some(uid) {
                std::future::pending::<()>().await;
            }
            if let Some(refusal) = self.refuses_a_delete_with.get() {
                return Err(refusal.as_error());
            }
            if let Some(held) = self.folders.borrow_mut().get_mut(folder) {
                held.retain(|(held_uid, _)| *held_uid != uid);
            }
            Ok("Deleted".to_string())
        }

        async fn copy_it(&self, from: &str, uid: u32, into: &str) -> Result<String> {
            self.log
                .borrow_mut()
                .push(format!("COPY {uid} in {from} into {into}"));
            Ok(format!("Copied to {into}"))
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
            WhenTheTrashIsEmptied::ALL
                .map(WhenTheTrashIsEmptied::days)
                .as_slice(),
            [None, None, Some(15), Some(30)]
        );
        assert_eq!(
            WhenTheTrashIsEmptied::ALL
                .map(WhenTheTrashIsEmptied::said)
                .as_slice(),
            [
                "Never",
                "When Wixen Mail closes",
                "After 15 days",
                "After 30 days"
            ]
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

    // ── When Wixen Mail closes (13-44.7) ───────────────────────────────────

    const ON_CLOSE: WhenTheTrashIsEmptied = WhenTheTrashIsEmptied::WhenWixenMailCloses;

    /// The sessions a close opens: one double per account, and the accounts
    /// it was asked for, in order.
    struct Sessions {
        servers: BTreeMap<&'static str, std::rc::Rc<AServerThatHoldsTheTrash>>,
        opened: RefCell<Vec<String>>,
    }

    impl Sessions {
        fn of(servers: impl IntoIterator<Item = (&'static str, AServerThatHoldsTheTrash)>) -> Self {
            Self {
                servers: servers
                    .into_iter()
                    .map(|(id, server)| (id, std::rc::Rc::new(server)))
                    .collect(),
                opened: RefCell::new(Vec::new()),
            }
        }

        fn the_server_of(&self, account_id: &str) -> &AServerThatHoldsTheTrash {
            self.servers
                .get(account_id)
                .expect("a server for the account")
        }

        fn opened(&self) -> Vec<String> {
            self.opened.borrow().clone()
        }
    }

    impl OpensTheSessionToEmpty for Sessions {
        type Session = std::rc::Rc<AServerThatHoldsTheTrash>;

        async fn session_for(&self, account_id: &str) -> Result<Self::Session> {
            self.opened.borrow_mut().push(account_id.to_string());
            self.servers
                .get(account_id)
                .cloned()
                .ok_or_else(|| crate::common::Error::Network(format!("no server for {account_id}")))
        }
    }

    /// A second account in the same store, with its own Inbox and Trash,
    /// answering its Trash.
    fn another_account(cache: &MessageCache, id: &str) -> i64 {
        for (name, kind) in [("Inbox", FolderType::Inbox), ("Trash", FolderType::Trash)] {
            cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: id.to_string(),
                    name: name.to_string(),
                    path: name.to_string(),
                    folder_type: kind.as_str().to_string(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("the folder");
        }
        cache
            .get_folder(id, "Trash")
            .expect("the folder read")
            .expect("the Trash is there")
            .id
    }

    /// An account set up here, under this id, read with this protocol.
    fn an_account_set_up(id: &str, name: &str, protocol: Protocol) -> Account {
        Account {
            id: id.to_string(),
            protocol: protocol.as_str().to_string(),
            ..Account::new(name.to_string(), "me@example.com".to_string())
        }
    }

    static WORK: std::sync::LazyLock<Account> =
        std::sync::LazyLock::new(|| an_account_set_up(THE_ACCOUNT, "Work", Protocol::Imap));
    static HOME: std::sync::LazyLock<Account> =
        std::sync::LazyLock::new(|| an_account_set_up("home", "Home", Protocol::Imap));

    /// Work, an IMAP account set to empty its Trash as Wixen Mail closes.
    fn work_on_close() -> AnAccountToEmpty<'static> {
        AnAccountToEmpty {
            account: &WORK,
            answer: ON_CLOSE,
            who_empties: WhoEmptiesTheTrash::ThisProgram,
            allowed_mail: true,
        }
    }

    fn home(answer: WhenTheTrashIsEmptied) -> AnAccountToEmpty<'static> {
        AnAccountToEmpty {
            account: &HOME,
            answer,
            ..work_on_close()
        }
    }

    /// How long past its limit a close may take before a case calls it
    /// hung: the limit is read before each message and bounds each wait,
    /// so what is left is this computer's own scheduling.
    const A_MARGIN: std::time::Duration = std::time::Duration::from_secs(1);

    /// One close with this long to empty, answering what it did and how
    /// long it took. A close that runs past its limit and a margin fails
    /// here rather than holding the run.
    fn closing(
        sessions: &Sessions,
        cache: &MessageCache,
        accounts: &[AnAccountToEmpty<'_>],
        within: std::time::Duration,
    ) -> (Vec<WhatTheCloseDid>, std::time::Duration) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("a runtime");
        let started = std::time::Instant::now();
        let did = runtime
            .block_on(async {
                tokio::time::timeout(
                    within + A_MARGIN,
                    empty_on_the_way_out(
                        sessions,
                        cache,
                        accounts,
                        tokio::time::Instant::now() + within,
                    ),
                )
                .await
            })
            .expect("the close ran past its limit and a margin");
        (did, started.elapsed())
    }

    const FIVE_SECONDS: std::time::Duration = std::time::Duration::from_secs(5);

    fn did(account_name: &str, emptied: usize, waiting: usize, left: usize) -> WhatTheCloseDid {
        WhatTheCloseDid {
            account_name: account_name.to_string(),
            emptied,
            waiting,
            left,
        }
    }

    #[test]
    fn test_closing_empties_everything_in_the_trash_of_an_account_set_to_empty_on_close() {
        let (_dir, cache, trash) = an_account();
        let sessions = Sessions::of([(
            THE_ACCOUNT,
            AServerThatHoldsTheTrash::holding_in_the_trash(&[1, 2, 3]),
        )]);
        // However long each has been there: a close empties the whole Trash.
        let rows = [
            in_the_trash_for(&cache, trash, 1, 45),
            in_the_trash_for(&cache, trash, 2, 3),
            in_the_trash_for(&cache, trash, 3, 0),
        ];

        let (did_it, _) = closing(&sessions, &cache, &[work_on_close()], FIVE_SECONDS);

        assert_eq!(
            sessions.the_server_of(THE_ACCOUNT).the_log(),
            [
                "DELETE 1 in Trash off the server",
                "DELETE 2 in Trash off the server",
                "DELETE 3 in Trash off the server"
            ]
        );
        assert!(sessions.the_server_of(THE_ACCOUNT).the_trash().is_empty());
        for row in rows {
            assert!(is_deleted_here(&cache, row) && !is_waiting(&cache, row));
        }
        assert_eq!(did_it, [did("Work", 3, 0, 0)]);
    }

    #[test]
    fn test_closing_stops_at_the_deadline_leaving_one_waiting_and_the_rest_untouched() {
        let (_dir, cache, trash) = an_account();
        let sessions = Sessions::of([(
            THE_ACCOUNT,
            AServerThatHoldsTheTrash::holding_in_the_trash(&[1, 2, 3])
                .never_answering_the_delete_of(2),
        )]);
        let first = in_the_trash_for(&cache, trash, 1, 50);
        let in_flight = in_the_trash_for(&cache, trash, 2, 40);
        let untouched = in_the_trash_for(&cache, trash, 3, 30);
        let within = std::time::Duration::from_millis(300);

        let (did_it, took) = closing(&sessions, &cache, &[work_on_close()], within);

        assert!(took < within + A_MARGIN, "the close took {took:?}");
        assert_eq!(
            sessions.the_server_of(THE_ACCOUNT).the_log(),
            [
                "DELETE 1 in Trash off the server",
                "DELETE 2 in Trash off the server"
            ]
        );
        assert!(is_deleted_here(&cache, first) && !is_waiting(&cache, first));
        // Made here and kept in the store the next start's check replays
        // before it lists any folder.
        assert!(is_deleted_here(&cache, in_flight) && is_waiting(&cache, in_flight));
        assert!(!is_deleted_here(&cache, untouched) && !is_waiting(&cache, untouched));
        assert!(
            cache
                .in_the_trash_since(untouched)
                .expect("the stamp")
                .is_some()
        );
        assert_eq!(did_it, [did("Work", 1, 1, 1)]);
    }

    #[test]
    fn test_closing_opens_no_session_for_an_account_with_nothing_in_the_trash() {
        let (_dir, cache, trash) = an_account();
        another_account(&cache, "home");
        let sessions = Sessions::of([
            (
                THE_ACCOUNT,
                AServerThatHoldsTheTrash::holding_in_the_trash(&[1]),
            ),
            ("home", AServerThatHoldsTheTrash::holding_in_the_trash(&[])),
        ]);
        in_the_trash_for(&cache, trash, 1, 2);

        let (did_it, _) = closing(
            &sessions,
            &cache,
            &[home(ON_CLOSE), work_on_close()],
            FIVE_SECONDS,
        );

        assert_eq!(sessions.opened(), [THE_ACCOUNT]);
        assert_eq!(did_it, [did("Work", 1, 0, 0)]);
    }

    #[test]
    fn test_closing_leaves_an_account_set_to_thirty_days_alone() {
        let (_dir, cache, trash) = an_account();
        let homes_trash = another_account(&cache, "home");
        let sessions = Sessions::of([
            (
                THE_ACCOUNT,
                AServerThatHoldsTheTrash::holding_in_the_trash(&[1]),
            ),
            ("home", AServerThatHoldsTheTrash::holding_in_the_trash(&[7])),
        ]);
        in_the_trash_for(&cache, trash, 1, 2);
        let homes = in_the_trash_for(&cache, homes_trash, 7, 45);

        let (did_it, _) = closing(
            &sessions,
            &cache,
            &[home(THIRTY), work_on_close()],
            FIVE_SECONDS,
        );

        assert_eq!(sessions.opened(), [THE_ACCOUNT]);
        assert!(sessions.the_server_of("home").the_log().is_empty());
        assert!(!is_deleted_here(&cache, homes));
        assert_eq!(did_it, [did("Work", 1, 0, 0)]);
    }

    #[test]
    fn test_a_check_leaves_an_account_set_to_empty_on_close_alone() {
        let (_dir, cache, trash) = an_account();
        let server = AServerThatHoldsTheTrash::holding_in_the_trash(&[1]);
        let row = in_the_trash_for(&cache, trash, 1, 45);

        assert_eq!(a_check(&server, &cache, ON_CLOSE), None);

        assert!(server.the_log().is_empty(), "{:?}", server.the_log());
        assert!(!is_deleted_here(&cache, row));
        assert_eq!(last_emptied(&cache), None, "a day was used");
    }

    #[test]
    fn test_with_changing_mail_off_closing_dials_nothing_and_a_check_says_it_once_a_day() {
        let (_dir, cache, trash) = an_account();
        let sessions = Sessions::of([(
            THE_ACCOUNT,
            AServerThatHoldsTheTrash::holding_in_the_trash(&[1]),
        )]);
        let row = in_the_trash_for(&cache, trash, 1, 45);
        let changes_off = AnAccountToEmpty {
            allowed_mail: false,
            ..work_on_close()
        };

        let (did_it, _) = closing(&sessions, &cache, &[changes_off], FIVE_SECONDS);

        assert!(sessions.opened().is_empty(), "{:?}", sessions.opened());
        assert!(did_it.is_empty(), "{did_it:?}");
        assert!(!is_deleted_here(&cache, row) && !is_waiting(&cache, row));

        // Nothing is said at close, so the check says it, once a day.
        let server = sessions.the_server_of(THE_ACCOUNT);
        assert_eq!(
            a_check_of(server, &cache, work(), ON_CLOSE, false, 0).as_deref(),
            Some(
                "Nothing will be emptied from Trash in Work when Wixen Mail closes, because \
                 Allow Changes does not let this account change mail."
            )
        );
        assert_eq!(a_check_of(server, &cache, work(), ON_CLOSE, false, 0), None);
        assert!(server.the_log().is_empty(), "{:?}", server.the_log());

        // And an account none of whose folders is its Trash, the same way.
        let (_dir, without) = an_account_without_a_trash();
        assert_eq!(
            a_check_of(server, &without, work(), ON_CLOSE, true, 0),
            Some(format!(
                "Nothing will be emptied in Work when Wixen Mail closes. {NO_TRASH_TO_EMPTY}"
            ))
        );
        assert_eq!(
            a_check_of(server, &without, work(), ON_CLOSE, true, 0),
            None
        );
    }

    #[test]
    fn test_what_the_close_did_names_the_account_and_three_counts_and_no_subject() {
        assert_eq!(
            what_the_close_did_for_the_log(&did("Work", 2, 1, 3)),
            "On the way out, the Trash in Work: 2 emptied, 1 waiting for the next check, 3 left \
             for the next close."
        );
        assert_eq!(
            what_the_close_did_for_the_log(&did("Home", 0, 0, 12)),
            "On the way out, the Trash in Home: 0 emptied, 0 waiting for the next check, 12 left \
             for the next close."
        );
    }

    // ── A POP account's Trash, on this computer (13-44.7) ──────────────────

    static OLD_ISP: std::sync::LazyLock<Account> =
        std::sync::LazyLock::new(|| an_account_set_up("pop", "Old ISP", Protocol::Pop3));
    static THE_CLUB: std::sync::LazyLock<Account> =
        std::sync::LazyLock::new(|| an_account_set_up("club", "Club", Protocol::Pop3));

    fn the_inbox_path() -> String {
        format!("{}/Inbox", crate::application::local_folders::LOCAL_PREFIX)
    }

    /// A store holding each POP account's Inbox on this computer and the
    /// Trash every account shares, stored where the program stores them,
    /// answering the shared Trash.
    fn a_pop_store() -> (tempfile::TempDir, MessageCache, i64) {
        use crate::application::local_folders;
        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
        for account in [&*OLD_ISP, &*THE_CLUB] {
            cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: account.id.clone(),
                    name: "Inbox".to_string(),
                    path: the_inbox_path(),
                    folder_type: FolderType::Inbox.as_str().to_string(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("the inbox");
        }
        let path = local_folders::local_trash(Protocol::Pop3).expect("a Trash here");
        let trash = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: local_folders::stored_under(&path, &OLD_ISP.id).to_string(),
                name: "Trash".to_string(),
                path,
                folder_type: FolderType::Trash.as_str().to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("the shared Trash");
        (dir, cache, trash)
    }

    fn the_uidl_of(account: &Account, uid: u32) -> String {
        format!("{}-{uid}", account.id)
    }

    /// A message this POP account downloaded, deleted here the way Delete
    /// deletes it, into the shared Trash, `days` ago.
    fn deleted_here_by(cache: &MessageCache, account: &Account, uid: u32, days: i64) -> i64 {
        let inbox = cache
            .get_folder(&account.id, &the_inbox_path())
            .expect("the folder read")
            .expect("the inbox is there")
            .id;
        let row = cache
            .upsert_message(&crate::data::message_cache::IncomingMessage {
                folder_id: inbox,
                uid,
                message_id: format!("<{uid}.{}@example.com>", account.id),
                subject: "Lunch".to_string(),
                from_addr: "ada@example.com".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                reply_to: None,
                date: "2026-09-19T09:00:00Z".to_string(),
                internal_date: None,
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
                server_thread_id: None,
                labels: None,
                receipt_to: None,
                list_unsubscribe: None,
                pop_uidl: Some(the_uidl_of(account, uid)),
            })
            .expect("a downloaded message");
        crate::application::local_delete::perform(cache, account, row, Deleting::ToTrash)
            .expect("the delete")
            .expect("a folder on this computer");
        cache
            .it_went_into_the_trash_at(row, Utc::now() - Duration::days(days))
            .expect("the stamp set");
        row
    }

    /// One POP check of an account, as many days after the first as `later`.
    fn a_pop_check(
        cache: &MessageCache,
        account: &Account,
        when: WhenTheTrashIsEmptied,
        later: i64,
    ) -> Option<String> {
        empty_the_trash_here_at_a_pop_check(
            cache,
            account,
            when,
            Utc::now() + Duration::days(later),
            the_first_day() + Duration::days(later),
        )
        .expect("the emptying to finish")
    }

    fn on_close(account: &Account) -> AnAccountToEmpty<'_> {
        AnAccountToEmpty {
            account,
            ..work_on_close()
        }
    }

    #[test]
    fn test_a_pop_account_set_to_thirty_days_marks_its_own_old_messages_deleted_here_and_keeps_their_identifiers()
     {
        let (_dir, cache, _trash) = a_pop_store();
        let old = deleted_here_by(&cache, &OLD_ISP, 1, 45);
        let recent = deleted_here_by(&cache, &OLD_ISP, 2, 10);

        let said = a_pop_check(&cache, &OLD_ISP, THIRTY, 0);

        assert!(is_deleted_here(&cache, old));
        assert!(!is_deleted_here(&cache, recent));
        // The identifier is what stops the next check downloading it again.
        assert!(
            cache
                .pop_uidls_for_account(&OLD_ISP.id)
                .expect("the identifiers read")
                .contains(&the_uidl_of(&OLD_ISP, 1)),
            "the emptied message's identifier is no longer this account's"
        );
        assert_eq!(
            said.as_deref(),
            Some(
                "Emptied 1 message of Old ISP's from the Trash on this computer that had been \
                 there more than 30 days."
            )
        );
    }

    #[test]
    fn test_a_pop_accounts_emptying_leaves_none_of_what_it_emptied_said() {
        // The tracer of 13-44.8 (D27): emptied by the setting, the message
        // keeps only what stops the next check downloading it again.
        use crate::data::message_cache::CachedAttachment;
        use crate::data::message_cache::attachment_content::AttachmentWithContent;
        let (_dir, cache, _trash) = a_pop_store();
        let due = deleted_here_by(&cache, &OLD_ISP, 1, 45);
        cache
            .save_message_body(due, Some("The minutes of the meeting."), None)
            .expect("its text");
        cache
            .replace_attachments_with_content(
                due,
                &[AttachmentWithContent {
                    described: CachedAttachment {
                        id: 0,
                        message_id: due,
                        filename: "minutes.txt".to_string(),
                        mime_type: "text/plain".to_string(),
                        size: 7,
                        content_id: None,
                        description: Default::default(),
                    },
                    content: Some(b"minutes".to_vec()),
                }],
            )
            .expect("its file");

        a_pop_check(&cache, &OLD_ISP, THIRTY, 0);

        assert_eq!(
            cache.what_a_row_still_holds(due).expect("the row read"),
            Vec::<String>::new(),
            "the emptied message still holds what it said"
        );
        assert_eq!(cache.get_message_body(due).expect("the lookup"), None);
        assert!(
            cache
                .attachments_with_content(due)
                .expect("the attachments read")
                .is_empty()
        );
        assert!(
            cache
                .pop_uidls_for_account(&OLD_ISP.id)
                .expect("the identifiers read")
                .contains(&the_uidl_of(&OLD_ISP, 1)),
            "the emptied message's identifier is no longer this account's"
        );
    }

    #[test]
    fn test_a_pop_accounts_emptying_leaves_another_accounts_messages_in_the_shared_trash() {
        let (_dir, cache, _trash) = a_pop_store();
        let mine = deleted_here_by(&cache, &OLD_ISP, 1, 45);
        let theirs = deleted_here_by(&cache, &THE_CLUB, 1, 45);

        a_pop_check(&cache, &OLD_ISP, THIRTY, 0);

        assert!(is_deleted_here(&cache, mine));
        assert!(
            !is_deleted_here(&cache, theirs),
            "another account's message in the shared Trash was emptied"
        );
    }

    #[test]
    fn test_a_pop_account_with_deleting_here_off_empties_nothing_and_says_so_once_a_day() {
        let (_dir, cache, _trash) = a_pop_store();
        let row = deleted_here_by(&cache, &OLD_ISP, 1, 45);
        let deleting_off = Account {
            allow_deleting_here: false,
            ..OLD_ISP.clone()
        };

        assert_eq!(
            a_pop_check(&cache, &deleting_off, THIRTY, 0),
            Some(format!(
                "Nothing was emptied from the Trash on this computer for Old ISP. \
                 {DELETING_IS_SWITCHED_OFF}"
            ))
        );
        assert_eq!(a_pop_check(&cache, &deleting_off, THIRTY, 0), None);
        assert!(!is_deleted_here(&cache, row));

        // Set to empty as Wixen Mail closes, its check says what the close
        // will not do, once a day (D26).
        assert_eq!(
            a_pop_check(&cache, &deleting_off, ON_CLOSE, 1),
            Some(format!(
                "Nothing will be emptied from the Trash on this computer for Old ISP when \
                 Wixen Mail closes. {DELETING_IS_SWITCHED_OFF}"
            ))
        );
        assert_eq!(a_pop_check(&cache, &deleting_off, ON_CLOSE, 1), None);
        assert!(!is_deleted_here(&cache, row));
    }

    #[test]
    fn test_a_pop_account_is_emptied_with_no_server_to_ask() {
        let (_dir, cache, _trash) = a_pop_store();
        let rows = [
            deleted_here_by(&cache, &OLD_ISP, 1, 45),
            deleted_here_by(&cache, &OLD_ISP, 2, 0),
        ];
        let no_server_at_all = Sessions::of([]);

        let (did_it, _) = closing(
            &no_server_at_all,
            &cache,
            &[on_close(&OLD_ISP)],
            FIVE_SECONDS,
        );

        assert!(no_server_at_all.opened().is_empty());
        for row in rows {
            assert!(is_deleted_here(&cache, row) && !is_waiting(&cache, row));
        }
        assert_eq!(did_it, [did("Old ISP", 2, 0, 0)]);
    }

    #[test]
    fn test_closing_empties_a_pop_account_first_and_within_the_deadline() {
        let (_dir, cache, _trash) = a_pop_store();
        a_folder(&cache, "Inbox", "INBOX", FolderType::Inbox);
        let works_trash = a_folder(&cache, "Trash", "Trash", FolderType::Trash);
        let in_flight = in_the_trash_for(&cache, works_trash, 1, 10);
        let pop_rows = [
            deleted_here_by(&cache, &OLD_ISP, 1, 45),
            deleted_here_by(&cache, &OLD_ISP, 2, 3),
        ];
        let sessions = Sessions::of([(
            THE_ACCOUNT,
            AServerThatHoldsTheTrash::holding_in_the_trash(&[1]).never_answering_the_delete_of(1),
        )]);
        let within = std::time::Duration::from_millis(300);

        // Work is listed first; POP goes first anyway, since it needs no
        // network and the limit is shared.
        let (did_it, took) = closing(
            &sessions,
            &cache,
            &[work_on_close(), on_close(&OLD_ISP)],
            within,
        );

        assert!(took < within + A_MARGIN, "the close took {took:?}");
        for row in pop_rows {
            assert!(is_deleted_here(&cache, row));
        }
        assert!(is_deleted_here(&cache, in_flight) && is_waiting(&cache, in_flight));
        assert_eq!(did_it, [did("Old ISP", 2, 0, 0), did("Work", 0, 1, 0)]);
    }
}

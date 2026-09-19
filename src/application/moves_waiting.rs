//! A move or a delete made here first, and the server brought into line after.
//!
//! # The defect this is about
//!
//! Moving a message, or deleting one, asked the server first and changed the
//! list only when the server had agreed: the row went "once the server has
//! agreed and not before", so that a "deleted" another device contradicts was
//! never announced. The tester (#86, 2026-09-18) felt the round trip on every
//! move: a noticeable wait before the row left and the sentence came, and the
//! same wait on every delete.
//!
//! This module keeps what that design guarded and moves the wait. The change
//! is made here at once and recorded as made here and not yet at the server;
//! the server is told in the background on the session the person's own
//! action opened, and again at the next check of that account before any
//! folder of it is read; a refusal undoes the change here and is said with the
//! reason; a restart replays what was waiting. The shape is the one
//! [`crate::application::flag_changes_waiting`] and the outbox already have,
//! and [`flag_changes_waiting::why_the_push_failed`] is still the one place
//! that tells a server that refused from a server that was never reached.
//!
//! # Why the replay runs before a folder is listed
//!
//! A message moved here keeps the folder and number the server still has it
//! under in the waiting row, and the row itself sits in the destination under
//! a number this computer reserved, marked as filed here so the sync neither
//! fetches it again nor forgets it. But the source folder, listed before the
//! server has heard of the move, still names the message, and a listing brings
//! back what it names and this computer does not hold. So a check replays the
//! account's waiting moves first, and a server it could not reach ends that
//! account's check rather than listing over a change the server has not heard
//! of.
//!
//! # Guardrail 7, which is the constraint a later reader will be most tempted
//! to break
//!
//! **Nothing in this module may observe the network coming back and send.** A
//! move reaching the server is a write at somebody else's service, so it
//! happens on purpose: when the person's own action puts this program in front
//! of that server, or when a check for mail does. [`Replayed`] has no member
//! meaning "send now" for that reason, the shape plan 03-08 settled for the
//! outbox and the flag queue repeated: a decision that cannot express the
//! dangerous act cannot be wired to it by accident.
//!
//! # A crossing to another account waits here too
//!
//! Since 11-07.2 (Pratik's decision of 2026-09-19) a move or a copy to a
//! folder on another account completes here first as well. Its row is the
//! same kind of row with the other account named, its bytes are held in
//! [`crate::data::message_cache::moves_in_flight`] from the moment the
//! source hands them over, and [`replay_the_crossings_waiting_for`] runs it
//! at a check of either account, in the three steps
//! [`crate::application::mail_across_accounts`] is cut into: fetch and keep,
//! append and ask, remove at the source, resumed from the held bytes when a
//! restart came between them. The order is the safeguard that module
//! states, and nothing here may put a message back here as if it were
//! nowhere when it may be in two places: an answer nobody could settle is
//! [`Replayed::NotReached`], and the destination is asked again next time.
//!
//! # What has never been checked
//!
//! No real server has replayed a move after a restart. The cases below drive
//! a loopback server that can carry a move out, refuse it, or hang up on it,
//! which is closer than a mocked error and is not a real mail server; what a
//! real server does with a message another client changed meanwhile, or
//! with an appended message it already holds, is the tester's account to
//! settle.

use std::sync::Arc;

use crate::application::flag_changes_waiting::{WhyThePushFailed, why_the_push_failed};
use crate::application::mail_across_accounts::{
    Appended, MovedAcross, TheAccountItIsGoingTo, TheAccountItIsIn, TheAccountItIsLeaving,
    TheMoveAsThisProgramRecordsIt, append_and_ask, fetch_and_keep, remove_at_the_source,
    resume_from_the_held_bytes, resume_the_append, why_it_cannot_be_finished_from_here,
};
use crate::application::server_delete::after_a_move_across_accounts;
use crate::common::{Error, Result};
use crate::data::message_cache::MessageCache;
use crate::data::message_cache::moves_waiting::{AWaitingMove, WhatAWaitingMoveDoes};

/// What a replay of a waiting move answered.
///
/// Five members and none of them means "send now": what sends a waiting move
/// is the person's own action or a check that already has a session, and
/// both call the replay directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Replayed {
    /// The server carried it out. Stop waiting.
    Done,
    /// The servers carried it out and one of them left something worth a
    /// sentence: a crossing whose source would not let the message go, so
    /// it is in both places, or marked it and could not remove it. Stop
    /// waiting, and say so.
    DoneWithSomethingToSay(String),
    /// The server refused it, and the message is already where the move
    /// wanted it: the server did it before the restart, or another client
    /// did. Read as done, not as refused, so nothing is put back that is
    /// already right.
    AlreadyDone,
    /// The server answered no, and the message is still where it was. Put
    /// the change back here and say why; sending it again meets the same
    /// answer.
    Refused(String),
    /// The server was never reached, so it has said nothing about this
    /// change. Leave it waiting.
    NotReached,
}

/// Where the message turned out to be when the server refused the replay.
///
/// Asked only then, because a refusal for a message the server no longer
/// holds where the move named it is the server having moved on, not having
/// said no.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhereItIsNow {
    /// Where the waiting move wanted it: in the destination, or off the
    /// server for a delete outright.
    WhereTheMoveWantedIt,
    /// Not there, so the refusal stands.
    NotThere,
}

/// What a replay's answer means, decided in one place.
///
/// `answer` is the server's answer to the command; `now` is where the
/// message is when the server refused, which the caller asks the server only
/// then.
///
/// A refusal by this computer's own gate is read as the server's refusal
/// rather than kept, and that is a decision the flag queue took the other
/// way: a flag change the gate refused waits for the setting. A waiting move
/// cannot, because a check ends at a move it could not replay, so a move
/// kept under a closed gate would stop the account being read until the
/// setting moved. The gate is met at the key before anything changes, so a
/// refusal met here means the setting moved in between; the move comes back
/// and says so.
pub fn what_a_replay_answered(answer: &Result<()>, now: WhereItIsNow) -> Replayed {
    let Err(why) = answer else {
        return Replayed::Done;
    };
    match why_the_push_failed(why) {
        WhyThePushFailed::TheServerWasNeverAsked => Replayed::NotReached,
        WhyThePushFailed::ThisComputerRefusedIt => Replayed::Refused(why.to_string()),
        WhyThePushFailed::TheServerSaidNo => match now {
            WhereItIsNow::WhereTheMoveWantedIt => Replayed::AlreadyDone,
            WhereItIsNow::NotThere => Replayed::Refused(why.to_string()),
        },
    }
}

/// What the status bar shows when the change has been made here.
///
/// For a move or a delete, shown and not spoken: the one word at the key and
/// the row the cursor lands on are what is heard (#83), and the fuller line
/// is for the eye. For a copy the row stays, so nothing else says it
/// happened, and this line is spoken as the answer to the key, in place of
/// the one word: 11-06.1's rule that a row which stayed has its outcome
/// spoken, kept, with the outcome known at once.
///
/// Worded by [`crate::application::server_delete`], the one owner of what a
/// delete, a move or a copy says, over the ending the server will give when
/// it agrees; a second spelling here is the drift the window's own
/// one-owner test exists to stop, and that test reads the window alone.
pub fn shown_when_made_here(waiting: &AWaitingMove, subject: &str) -> String {
    use crate::application::server_delete::{Copied, after_a_copy, after_a_delete, after_a_move};
    use crate::service::protocols::imap::{Deletion, Moved};
    match &waiting.what {
        WhatAWaitingMoveDoes::Move { into_folder_path } => {
            after_a_move(&Moved::Moved, into_folder_path, subject).said
        }
        WhatAWaitingMoveDoes::DeleteToTrash { .. } => {
            after_a_delete(&Deletion::MovedToTrash, subject).said
        }
        WhatAWaitingMoveDoes::DeleteOutright => after_a_delete(&Deletion::Removed, subject).said,
        WhatAWaitingMoveDoes::Copy { into_folder_path } => {
            after_a_copy(Copied::WithinTheAccount, into_folder_path, subject).said
        }
        WhatAWaitingMoveDoes::MoveAcross {
            into_folder_path,
            to_account,
        } => {
            after_a_move_across_accounts(
                &MovedAcross::ItArrivedAndTheSourceLetItGo,
                into_folder_path,
                &to_account.name,
                &waiting.from_folder_path,
                subject,
            )
            .said
        }
        WhatAWaitingMoveDoes::CopyAcross {
            into_folder_path,
            to_account,
        } => {
            after_a_copy(
                Copied::IntoTheAccount(&to_account.name),
                into_folder_path,
                subject,
            )
            .said
        }
    }
}

/// What a copy of a row that is itself a copy not yet at the server is
/// refused with.
///
/// A second copy, or a move or a delete, of a row the server does not hold
/// yet has nothing at the server to act on until the first copy has landed.
/// Refused at the key, in words, rather than queued behind a copy whose
/// outcome would decide what the second ask means.
pub const THAT_COPY_HAS_NOT_REACHED_THE_SERVER: &str =
    "That copy has not reached the server yet. Try again after the next check for mail.";

/// What a second ask about a row whose crossing is still waiting is refused
/// with.
///
/// The row sits in the other account's folder here and at neither server
/// yet as the row says: a move of it from there would name a folder and a
/// number the other account's server has never heard of, and a delete would
/// remove it from a server that does not hold it. Refused in words until
/// the crossing has landed, for the reason the copy above is.
pub const THAT_MOVE_TO_ANOTHER_ACCOUNT_HAS_NOT_FINISHED: &str = "That message is still on its \
     way to the other account. Try again after the next check for mail.";

/// What a crossing is refused with when one of its two accounts is no
/// longer set up on this computer.
pub const ONE_OF_THE_TWO_ACCOUNTS_IS_GONE: &str = "one of the two accounts it was between is no \
     longer set up on this computer, so it cannot be finished from here";

/// What is said, at High, when the server refused and the change is undone.
///
/// Deliberately unlike the shown line: that one says where the message went,
/// this one says it is back and why. They share no opening clause and no verb
/// (guardrail 5).
pub fn put_back_because_the_server_refused(
    what: &WhatAWaitingMoveDoes,
    subject: &str,
    reason: &str,
) -> String {
    match what {
        WhatAWaitingMoveDoes::Move { into_folder_path } => format!(
            "Could not move {subject} to {into_folder_path}: {reason}. It is back where it was."
        ),
        WhatAWaitingMoveDoes::DeleteToTrash { .. } | WhatAWaitingMoveDoes::DeleteOutright => {
            format!("Could not delete {subject}: {reason}. It is back where it was.")
        }
        // Nothing to put back: the original never moved, and the copy made
        // here is gone.
        WhatAWaitingMoveDoes::Copy { into_folder_path } => {
            format!("Could not copy {subject} to {into_folder_path}: {reason}. Nothing was copied.")
        }
        WhatAWaitingMoveDoes::MoveAcross {
            into_folder_path,
            to_account,
        } => format!(
            "Could not move {subject} to {into_folder_path} in {}: {reason}. It is back where \
             it was.",
            to_account.name
        ),
        WhatAWaitingMoveDoes::CopyAcross {
            into_folder_path,
            to_account,
        } => format!(
            "Could not copy {subject} to {into_folder_path} in {}: {reason}. Nothing was copied.",
            to_account.name
        ),
    }
}

/// What was made here: the row the change is recorded against, and the line
/// for the status bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MadeHere {
    /// The waiting row as it was kept. For a move or a delete it is the
    /// message's own row; for a copy it is the copy's row, which did not
    /// exist before.
    pub kept: AWaitingMove,
    pub shown: String,
}

/// Why a change was not made here.
///
/// Two answers rather than one error, because the window does two different
/// things with them: a refusal in words is said and that is the end of it,
/// and a change that could not be recorded here is asked of the server first
/// instead, the path every move and delete took until 2026-09-19, since a
/// change made here with no record of it is one the next read of the folder
/// undoes.
#[derive(Debug)]
pub enum NotMadeHere {
    /// Nothing is done and this is said.
    RefusedInWords(String),
    /// The store would not take the change or its record; the server is
    /// asked first, as before.
    CouldNotBeRecorded(Error),
}

impl std::fmt::Display for NotMadeHere {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RefusedInWords(words) => f.write_str(words),
            Self::CouldNotBeRecorded(why) => write!(f, "{why}"),
        }
    }
}

impl From<Error> for NotMadeHere {
    fn from(why: Error) -> Self {
        Self::CouldNotBeRecorded(why)
    }
}

/// Make the change here: the row into the folder the move names, in this
/// account or the other, or marked deleted, or copied into the folder named,
/// and the waiting row written.
///
/// The row leaving the list is the window's to do, since only it holds the
/// control. A row that is itself a copy not yet at the server is refused
/// with [`THAT_COPY_HAS_NOT_REACHED_THE_SERVER`], and a row whose crossing
/// is still waiting with [`THAT_MOVE_TO_ANOTHER_ACCOUNT_HAS_NOT_FINISHED`]:
/// the row is in the other account's folder here and the server that holds
/// the message is still the first one, so a second ask from where the row
/// now sits would name a folder and a number the wrong server never gave.
pub fn what_happens_here(
    cache: &MessageCache,
    asked: &AWaitingMove,
    subject: &str,
) -> std::result::Result<MadeHere, NotMadeHere> {
    let already_waiting = cache.the_move_waiting_for(asked.message_row_id)?;
    if let Some(waiting) = already_waiting.as_ref() {
        if waiting.what.crosses_to().is_some() {
            return Err(NotMadeHere::RefusedInWords(
                THAT_MOVE_TO_ANOTHER_ACCOUNT_HAS_NOT_FINISHED.to_string(),
            ));
        }
        if waiting.what.is_a_copy() {
            return Err(NotMadeHere::RefusedInWords(
                THAT_COPY_HAS_NOT_REACHED_THE_SERVER.to_string(),
            ));
        }
    }
    let going_to = asked.the_account_it_is_going_to();
    let kept = match (asked.what.is_a_copy(), asked.what.destination()) {
        // A copy of the row, under a number reserved from the top of the
        // destination's range and marked as filed here, waiting under its own
        // row; the server copies from where it still has the original, which
        // is the original's waiting row's answer when it has one. The same
        // for a copy into another account's folder: the folder is one this
        // computer holds, whichever account it belongs to.
        (true, Some(destination)) => {
            let into = the_folder_here(cache, going_to, destination)?;
            let copy = cache.copy_message_here(asked.message_row_id, into.id)?;
            let (from_folder_path, uid) = match already_waiting {
                Some(waiting) => (waiting.from_folder_path, waiting.uid),
                None => (asked.from_folder_path.clone(), asked.uid),
            };
            AWaitingMove {
                message_row_id: copy,
                from_folder_path,
                uid,
                ..asked.clone()
            }
        }
        // Into the folder as this program moves a row of its own: under a
        // number reserved from the top of the folder's range and marked as
        // filed here, so the next read of that folder neither fetches the
        // message again nor forgets a row the server never listed. A
        // crossing's folder is the other account's, held here like any.
        (_, Some(destination)) => {
            let into = the_folder_here(cache, going_to, destination)?;
            cache.move_message(asked.message_row_id, into.id)?;
            asked.clone()
        }
        // Marked deleted under its own number, which is what the next read
        // of the folder forgets once the server no longer lists it, as a
        // delete's row always was.
        (_, None) => {
            cache.delete_message(asked.message_row_id)?;
            asked.clone()
        }
    };
    cache.keep_a_move_waiting(&kept)?;
    Ok(MadeHere {
        shown: shown_when_made_here(&kept, subject),
        kept,
    })
}

/// The folder on this computer a path names, or a sentence saying it is not
/// here.
fn the_folder_here(
    cache: &MessageCache,
    account_id: &str,
    path: &str,
) -> Result<crate::data::message_cache::CachedFolder> {
    cache.get_folder(account_id, path)?.ok_or_else(|| {
        Error::InPlainWords(format!(
            "There is no folder called {path} on this computer, so nothing was done."
        ))
    })
}

/// Undo the change here, because the server refused it: the row back in the
/// folder and under the number it never left, or the copy made here gone,
/// the waiting row gone, and any bytes held for a crossing let go.
///
/// A source folder no longer on this computer, which is an account taken
/// off, leaves the row nowhere true to put it: it goes, and the next read of
/// wherever the server has the message brings it down.
pub fn undo_here(cache: &MessageCache, waiting: &AWaitingMove) -> Result<()> {
    cache.the_move_is_over(waiting.message_row_id)?;
    if waiting.what.is_a_copy() {
        cache.let_the_next_read_bring_it(waiting.message_row_id)?;
        return cache.stop_waiting_for_a_move(waiting.message_row_id);
    }
    match cache.get_folder(&waiting.account_id, &waiting.from_folder_path)? {
        Some(from) => cache.the_server_holds_it_at(waiting.message_row_id, from.id, waiting.uid)?,
        None => cache.let_the_next_read_bring_it(waiting.message_row_id)?,
    }
    cache.stop_waiting_for_a_move(waiting.message_row_id)
}

/// What a replay asks of a mail server.
///
/// Named for what it does rather than for the protocol, so the replay can be
/// held against a loopback server from this module's own tests. Crate-private,
/// as [`crate::application::mail_sync::Mailbox`] is: the seam the tests need,
/// not something a caller should know about.
pub(crate) trait ReplaysAMove {
    /// Move the message; any answer that is not an error is the server
    /// having done something with it.
    async fn move_it(&self, from: &str, uid: u32, into: &str) -> Result<()>;
    /// Delete the message into `trash`, or off the server when there is
    /// none.
    async fn delete_it(&self, folder: &str, uid: u32, trash: Option<&str>) -> Result<()>;
    /// Copy the message, leaving the original where it is.
    async fn copy_it(&self, from: &str, uid: u32, into: &str) -> Result<()>;
    /// The numbers the folder holds a message with this identifier under.
    async fn where_it_is(&self, folder: &str, message_id: &str) -> Result<Vec<u32>>;
}

impl ReplaysAMove for crate::application::mail_controller::MailController {
    async fn move_it(&self, from: &str, uid: u32, into: &str) -> Result<()> {
        let moved = self.move_message(from, uid, into).await?;
        tracing::info!("A waiting move was replayed: {}", moved.spoken(into));
        Ok(())
    }

    async fn copy_it(&self, from: &str, uid: u32, into: &str) -> Result<()> {
        self.copy_message(from, uid, into).await?;
        tracing::info!("A waiting copy was replayed: copied to {into}");
        Ok(())
    }

    async fn delete_it(&self, folder: &str, uid: u32, trash: Option<&str>) -> Result<()> {
        let deletion = self.delete_message(folder, uid, trash).await?;
        tracing::info!("A waiting delete was replayed: {}", deletion.spoken());
        Ok(())
    }

    async fn where_it_is(&self, folder: &str, message_id: &str) -> Result<Vec<u32>> {
        self.uids_with_message_id(folder, message_id).await
    }
}

/// Replay every move waiting for the account, in the order asked, and settle
/// each row by what the server answered.
///
/// A move the server carried out, or had already, leaves its row where the
/// server holds the message now; a refusal is handed back with its reason and
/// the undo is the caller's, since the window says it; a server that could not
/// be reached ends the replay, and the moves after that one wait with it.
pub(crate) async fn replay_the_moves_waiting_for<S: ReplaysAMove>(
    server: &S,
    cache: &MessageCache,
    account_id: &str,
) -> Result<Vec<(AWaitingMove, Replayed)>> {
    let mut replayed = Vec::new();
    for waiting in cache.moves_waiting_for(account_id)? {
        let from = waiting.from_folder_path.as_str();
        let answer = match &waiting.what {
            WhatAWaitingMoveDoes::Move { into_folder_path } => {
                server.move_it(from, waiting.uid, into_folder_path).await
            }
            WhatAWaitingMoveDoes::DeleteToTrash { trash_path } => {
                server.delete_it(from, waiting.uid, Some(trash_path)).await
            }
            WhatAWaitingMoveDoes::DeleteOutright => server.delete_it(from, waiting.uid, None).await,
            WhatAWaitingMoveDoes::Copy { into_folder_path } => {
                server.copy_it(from, waiting.uid, into_folder_path).await
            }
            // Two servers' work, which the read above leaves out and
            // [`replay_the_crossings_waiting_for`] does; a row here would be
            // a command at the wrong server.
            WhatAWaitingMoveDoes::MoveAcross { .. } | WhatAWaitingMoveDoes::CopyAcross { .. } => {
                continue;
            }
        };
        // Where the message is, asked only of a server that answered no: a
        // server that hung up cannot be asked, and asking would turn the
        // question into a second failure. A question that fails is read the
        // way the answer would have been.
        let what_it_means = match &answer {
            Err(why) if why_the_push_failed(why) == WhyThePushFailed::TheServerSaidNo => {
                match where_it_is_now(server, cache, &waiting).await {
                    Ok(now) => what_a_replay_answered(&answer, now),
                    Err(asking) => what_a_replay_answered(&Err(asking), WhereItIsNow::NotThere),
                }
            }
            _ => what_a_replay_answered(&answer, WhereItIsNow::NotThere),
        };
        match &what_it_means {
            Replayed::Done | Replayed::DoneWithSomethingToSay(_) | Replayed::AlreadyDone => {
                settle_the_row(server, cache, &waiting).await?;
                cache.stop_waiting_for_a_move(waiting.message_row_id)?;
            }
            // The undo is the window's, so the sentence is said beside it,
            // and the row stops waiting there too: a refusal met with the
            // program gone before the undo ran is met again at the next
            // check, which undoes it then.
            Replayed::Refused(_) => {}
            Replayed::NotReached => {
                replayed.push((waiting, what_it_means));
                break;
            }
        }
        replayed.push((waiting, what_it_means));
    }
    Ok(replayed)
}

/// Where the message is, asked of the server once it has refused a replay.
///
/// For a move or a delete to the trash, whether the destination holds the
/// message's identifier; for a delete outright, whether the folder it was in
/// no longer does.
async fn where_it_is_now<S: ReplaysAMove>(
    server: &S,
    cache: &MessageCache,
    waiting: &AWaitingMove,
) -> Result<WhereItIsNow> {
    let Some(message) = cache.get_message(waiting.message_row_id)? else {
        return Ok(WhereItIsNow::NotThere);
    };
    let found = match waiting.what.destination() {
        Some(destination) => !server
            .where_it_is(destination, &message.message_id)
            .await?
            .is_empty(),
        None => server
            .where_it_is(&waiting.from_folder_path, &message.message_id)
            .await?
            .is_empty(),
    };
    Ok(if found {
        WhereItIsNow::WhereTheMoveWantedIt
    } else {
        WhereItIsNow::NotThere
    })
}

/// Leave the row where the server holds the message now that the move is
/// done.
///
/// A move or a delete to the trash: the destination is asked which number it
/// holds the identifier under, and the row becomes that message there,
/// unmarked; when it cannot say, the row goes and the next read of the
/// destination brings the message down. A delete outright changes nothing
/// here: the row stays marked deleted under its number, which is what the
/// next read of its folder forgets, as a delete's row always was.
async fn settle_the_row<S: ReplaysAMove>(
    server: &S,
    cache: &MessageCache,
    waiting: &AWaitingMove,
) -> Result<()> {
    let Some(destination) = waiting.what.destination() else {
        return Ok(());
    };
    let Some(message) = cache.get_message(waiting.message_row_id)? else {
        return Ok(());
    };
    let Some(folder) = cache.get_folder(&waiting.account_id, destination)? else {
        return cache.let_the_next_read_bring_it(waiting.message_row_id);
    };
    match server
        .where_it_is(destination, &message.message_id)
        .await?
        .as_slice()
    {
        [uid] => cache.the_server_holds_it_at(waiting.message_row_id, folder.id, *uid),
        _ => cache.let_the_next_read_bring_it(waiting.message_row_id),
    }
}

// ── A crossing: two accounts, three steps, resumed from held bytes ─────────

/// The session an account is signed in with, or why there is none.
///
/// Three answers, because two of them lead to different things: an account
/// no longer set up here can never be reached, so the crossing is undone and
/// said, while a session that could not be opened now may open at the next
/// check, so the crossing waits.
pub(crate) enum ASessionFor<S> {
    Open(Arc<S>),
    NotSetUpHere,
    CouldNotBeOpened(Error),
}

/// What a crossing's replay asks for: the session of each of its two
/// accounts.
///
/// A seam, so the crossing can be held against two loopback servers from this
/// module's tests; in the program it is the accounts set up here and the
/// sessions they are signed in with.
pub(crate) trait OpensASession {
    type Session: TheAccountItIsIn + TheAccountItIsGoingTo + TheAccountItIsLeaving;
    async fn session_for(&self, account_id: &str) -> ASessionFor<Self::Session>;
}

/// What a crossing's ending means for the waiting row, decided in one place.
///
/// The two clean arrivals are done; the two where the message is at the
/// destination and still at the source, marked or not, are done with a
/// sentence, since two copies is a fact somebody has to hear; a refusal and
/// an append the destination was asked about and does not hold are refusals
/// with the reason, since nothing was removed and the row can be put back;
/// and an append nobody could settle is not reached, never a refusal, since
/// a message that may be in two places must not be put back here as if it
/// were nowhere.
pub fn what_a_crossing_answered(across: &MovedAcross, of: &ACrossingSaidAs<'_>) -> Replayed {
    match across {
        MovedAcross::ItArrivedAndTheSourceLetItGo => Replayed::Done,
        MovedAcross::ItArrivedAndIsStillHereMarked(_)
        | MovedAcross::ItArrivedAndTheSourceWouldNotLetGo(_) => Replayed::DoneWithSomethingToSay(
            after_a_move_across_accounts(across, of.into, of.to_account, of.still_in, of.subject)
                .said,
        ),
        MovedAcross::TheDestinationRefusedIt(why) => Replayed::Refused(why.clone()),
        MovedAcross::ItNeverArrivedSoNothingWasRemoved(why) => Replayed::Refused(format!(
            "{} stopped answering and does not have the message; {why}",
            of.to_account
        )),
        MovedAcross::ItIsNotKnownWhereItIs(_) => Replayed::NotReached,
    }
}

/// What a sentence about a crossing names.
#[derive(Debug, Clone, Copy)]
pub struct ACrossingSaidAs<'a> {
    pub into: &'a str,
    /// What the account it is going to is called.
    pub to_account: &'a str,
    /// The folder the server still has it in.
    pub still_in: &'a str,
    pub subject: &'a str,
}

/// Replay every crossing waiting that this account is one end of, in the
/// order asked, on the sessions of both accounts, and settle each row by
/// what the servers answered.
///
/// Each crossing is resumed from its held bytes when the store has them,
/// which is a restart between the fetch and the ending, and run from the
/// fetch when it does not. A crossing nothing can finish, because one of
/// its accounts is no longer set up here or its held row cannot be asked
/// about, is a refusal with that reason and is undone by the window's arm.
/// A session that could not be opened ends the replay, and the crossings
/// after that one wait with it. Nothing here sends because the network came
/// back: the module header says why.
pub(crate) async fn replay_the_crossings_waiting_for<O: OpensASession>(
    accounts: &O,
    cache: &MessageCache,
    account_id: &str,
) -> Result<Vec<(AWaitingMove, Replayed)>> {
    let mut replayed = Vec::new();
    for waiting in cache.crossings_waiting_touching(account_id)? {
        let Some(message) = cache.get_message(waiting.message_row_id)? else {
            cache.the_move_is_over(waiting.message_row_id)?;
            cache.stop_waiting_for_a_move(waiting.message_row_id)?;
            continue;
        };
        let (answer, was_there_before) =
            one_crossing(accounts, cache, &waiting, &message.subject).await;
        match &answer {
            Replayed::Done | Replayed::DoneWithSomethingToSay(_) | Replayed::AlreadyDone => {
                settle_the_crossed_row(accounts, cache, &waiting, was_there_before.as_deref())
                    .await?;
                cache.the_move_is_over(waiting.message_row_id)?;
                cache.stop_waiting_for_a_move(waiting.message_row_id)?;
            }
            // The undo is the window's, so the sentence is said beside it
            // and the bytes go there.
            Replayed::Refused(_) => {}
            Replayed::NotReached => {
                replayed.push((waiting, answer));
                break;
            }
        }
        replayed.push((waiting, answer));
    }
    Ok(replayed)
}

/// One crossing, from wherever it stopped to an answer, and what the
/// destination folder held before the message went, for the settle.
async fn one_crossing<O: OpensASession>(
    accounts: &O,
    cache: &MessageCache,
    waiting: &AWaitingMove,
    subject: &str,
) -> (Replayed, Option<Vec<u32>>) {
    let Some(other) = waiting.what.crosses_to() else {
        return (
            Replayed::Refused(ONE_OF_THE_TWO_ACCOUNTS_IS_GONE.to_string()),
            None,
        );
    };
    let (source, destination) = match (
        accounts.session_for(&waiting.account_id).await,
        accounts.session_for(&other.id).await,
    ) {
        (ASessionFor::Open(source), ASessionFor::Open(destination)) => (source, destination),
        (ASessionFor::NotSetUpHere, _) | (_, ASessionFor::NotSetUpHere) => {
            return (
                Replayed::Refused(ONE_OF_THE_TWO_ACCOUNTS_IS_GONE.to_string()),
                None,
            );
        }
        (ASessionFor::CouldNotBeOpened(why), _) | (_, ASessionFor::CouldNotBeOpened(why)) => {
            tracing::info!(
                "A move of message {} to another account waits: {why}",
                waiting.message_row_id
            );
            return (Replayed::NotReached, None);
        }
    };
    let Some(into) = waiting.what.destination() else {
        return (
            Replayed::Refused(ONE_OF_THE_TWO_ACCOUNTS_IS_GONE.to_string()),
            None,
        );
    };
    let said_as = ACrossingSaidAs {
        into,
        to_account: &other.name,
        still_in: &waiting.from_folder_path,
        subject,
    };
    let copying = waiting.what.is_a_copy();
    let held = match cache.the_move_left_unfinished_for(waiting.message_row_id) {
        Ok(held) => held,
        Err(why) => return (Replayed::Refused(why.to_string()), None),
    };
    let Some(kept) = held else {
        // No restart came between the fetch and an ending, so the crossing
        // runs from its first step. The source saying nothing usable, or
        // never being reached, or this computer's gate refusing, is read
        // the way the within-account replay reads a push that failed.
        let fetched = match fetch_and_keep(
            source.as_ref(),
            destination.as_ref(),
            &waiting.from_folder_path,
            waiting.uid,
            into,
            TheMoveAsThisProgramRecordsIt {
                cache: Some(cache),
                row: waiting.message_row_id,
                from_account_id: &waiting.account_id,
                to_account_id: &other.id,
            },
        )
        .await
        {
            Ok(fetched) => fetched,
            Err(why) => {
                return (
                    what_a_replay_answered(&Err(why), WhereItIsNow::NotThere),
                    None,
                );
            }
        };
        let ending = match append_and_ask(destination.as_ref(), &fetched.message).await {
            Appended::ItLanded if copying => MovedAcross::ItArrivedAndTheSourceLetItGo,
            Appended::ItLanded => remove_at_the_source(source.as_ref(), &fetched.message).await,
            Appended::ItDidNot(ending) => ending,
        };
        return (
            what_a_crossing_answered(&ending, &said_as),
            fetched.message.was_there_before,
        );
    };
    if let Some(why) = why_it_cannot_be_finished_from_here(&kept, &other.name) {
        return (Replayed::Refused(why), kept.was_there_before);
    }
    let ending = if copying {
        match resume_the_append(destination.as_ref(), &kept).await {
            Appended::ItLanded => MovedAcross::ItArrivedAndTheSourceLetItGo,
            Appended::ItDidNot(ending) => ending,
        }
    } else {
        resume_from_the_held_bytes(&kept, destination.as_ref(), source.as_ref()).await
    };
    (
        what_a_crossing_answered(&ending, &said_as),
        kept.was_there_before,
    )
}

/// Leave the row where the other account holds the message now that the
/// crossing is done: under the number the destination folder holds the
/// identifier under and did not before, unmarked, or gone for the next read
/// of that folder to bring down when it cannot be told.
async fn settle_the_crossed_row<O: OpensASession>(
    accounts: &O,
    cache: &MessageCache,
    waiting: &AWaitingMove,
    was_there_before: Option<&[u32]>,
) -> Result<()> {
    let (Some(other), Some(into)) = (waiting.what.crosses_to(), waiting.what.destination()) else {
        return Ok(());
    };
    let Some(message) = cache.get_message(waiting.message_row_id)? else {
        return Ok(());
    };
    let Some(folder) = cache.get_folder(&other.id, into)? else {
        return cache.let_the_next_read_bring_it(waiting.message_row_id);
    };
    let ASessionFor::Open(destination) = accounts.session_for(&other.id).await else {
        return cache.let_the_next_read_bring_it(waiting.message_row_id);
    };
    let now = destination
        .which_messages_carry(into, &message.message_id)
        .await
        .unwrap_or_default();
    let arrived: Vec<u32> = now
        .into_iter()
        .filter(|uid| !was_there_before.is_some_and(|before| before.contains(uid)))
        .collect();
    match arrived.as_slice() {
        [uid] => cache.the_server_holds_it_at(waiting.message_row_id, folder.id, *uid),
        _ => cache.let_the_next_read_bring_it(waiting.message_row_id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Error;
    use crate::common::answering::{Conversation, Turn, conversing};
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{CachedFolder, CachedMessage};
    use crate::service::protocols::imap::ImapSession;
    use crate::service::protocols::imap::against_a_server_that_answers::{
        a_server_that_can, a_server_that_refuses, signed_in_to,
    };
    use std::path::Path;

    /// One account's Inbox, Archive and Trash, and another account's Work
    /// folder for a crossing to go to.
    fn a_cache_at(dir: &Path) -> MessageCache {
        let cache = MessageCache::new(dir.to_path_buf(), None).expect("a cache");
        for (account, name, kind) in [
            ("an account", "INBOX", "Inbox"),
            ("an account", "Archive", "Archive"),
            ("an account", "Trash", "Trash"),
            ("another account", "Work", "Custom"),
        ] {
            cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: account.to_string(),
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
        TempHome::named("wixen_moves_waiting_replay_", a_cache_at)
    }

    fn the_folder(cache: &MessageCache, path: &str) -> i64 {
        cache
            .get_folder("an account", path)
            .expect("the folder")
            .expect("the folder is there")
            .id
    }

    /// The other account's Work folder, where a crossing goes.
    fn the_other_accounts_work(cache: &MessageCache) -> i64 {
        cache
            .get_folder("another account", "Work")
            .expect("the folder")
            .expect("the folder is there")
            .id
    }

    /// The other account as a crossing names it.
    fn home() -> crate::data::message_cache::moves_waiting::TheOtherAccount {
        crate::data::message_cache::moves_waiting::TheOtherAccount {
            id: "another account".to_string(),
            name: "Home".to_string(),
        }
    }

    fn into_the_other_accounts_work() -> WhatAWaitingMoveDoes {
        WhatAWaitingMoveDoes::MoveAcross {
            into_folder_path: "Work".to_string(),
            to_account: home(),
        }
    }

    fn a_copy_into_the_other_accounts_work() -> WhatAWaitingMoveDoes {
        WhatAWaitingMoveDoes::CopyAcross {
            into_folder_path: "Work".to_string(),
            to_account: home(),
        }
    }

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

    fn a_move_of(row: i64, uid: u32, what: WhatAWaitingMoveDoes) -> AWaitingMove {
        AWaitingMove {
            message_row_id: row,
            account_id: "an account".to_string(),
            from_folder_path: "INBOX".to_string(),
            uid,
            what,
            asked_at: "2026-09-19T04:00:00Z".to_string(),
        }
    }

    fn into_the_archive() -> WhatAWaitingMoveDoes {
        WhatAWaitingMoveDoes::Move {
            into_folder_path: "Archive".to_string(),
        }
    }

    /// (folder, uid, deleted, filed here) for the row, or nothing when the
    /// row is gone.
    fn where_the_row_is(cache: &MessageCache, row: i64) -> Option<(i64, u32, bool, bool)> {
        let message = cache.get_message(row).expect("the message")?;
        Some((
            message.folder_id,
            message.uid,
            message.deleted,
            cache.was_filed_here(row).expect("the marker"),
        ))
    }

    fn still_waiting(cache: &MessageCache) -> Vec<i64> {
        cache
            .moves_waiting_for("an account")
            .expect("the waiting moves")
            .iter()
            .map(|waiting| waiting.message_row_id)
            .collect()
    }

    /// A session of its own, below the controller.
    ///
    /// Below it for the reason `flag_changes_waiting`'s tests record: the
    /// controller opens the write gate from a setting on the machine running
    /// the tests, so through it both servers below answered a gate refusal
    /// before either was asked anything. What is exercised is the socket, the
    /// library's own error and this project's mapping of it.
    struct ASessionOfItsOwn(tokio::sync::Mutex<ImapSession>);

    impl ASessionOfItsOwn {
        async fn at(server: &Conversation) -> Self {
            Self(tokio::sync::Mutex::new(signed_in_to(server).await))
        }
    }

    impl ReplaysAMove for ASessionOfItsOwn {
        async fn move_it(&self, from: &str, uid: u32, into: &str) -> Result<()> {
            let mut session = self.0.lock().await;
            session.select_folder(from).await?;
            session.move_message(uid, into).await.map(|_| ())
        }

        async fn delete_it(&self, folder: &str, uid: u32, trash: Option<&str>) -> Result<()> {
            let mut session = self.0.lock().await;
            session.select_folder(folder).await?;
            session.delete_message(uid, trash).await.map(|_| ())
        }

        async fn copy_it(&self, from: &str, uid: u32, into: &str) -> Result<()> {
            let mut session = self.0.lock().await;
            session.select_folder(from).await?;
            session.copy_message(uid, into).await
        }

        async fn where_it_is(&self, folder: &str, message_id: &str) -> Result<Vec<u32>> {
            let mut session = self.0.lock().await;
            session.select_folder(folder).await?;
            session.uids_with_message_id(message_id).await
        }
    }

    fn a_copy_into_the_archive() -> WhatAWaitingMoveDoes {
        WhatAWaitingMoveDoes::Copy {
            into_folder_path: "Archive".to_string(),
        }
    }

    /// The copy made here of the Inbox message, waiting under its own row.
    fn a_copy_made_here(home: &MessageCache, row: i64) -> AWaitingMove {
        what_happens_here(
            home,
            &a_move_of(row, 42, a_copy_into_the_archive()),
            "Lunch",
        )
        .expect("copied here")
        .kept
    }

    #[test]
    fn test_a_copy_made_here_leaves_the_original_and_waits_under_the_copys_row() {
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let inbox = the_folder(&home, "INBOX");
        let archive = the_folder(&home, "Archive");

        let made = what_happens_here(
            &home,
            &a_move_of(row, 42, a_copy_into_the_archive()),
            "Lunch",
        )
        .expect("copied here");

        assert_eq!(made.shown, "Copied to Archive: Lunch");
        assert_ne!(
            made.kept.message_row_id, row,
            "the copy waits under the original's row"
        );
        assert_eq!(
            (made.kept.from_folder_path.as_str(), made.kept.uid),
            ("INBOX", 42),
            "the server copies from where it has the original"
        );
        assert_eq!(
            where_the_row_is(&home, row),
            Some((inbox, 42, false, false))
        );
        let (folder, _, _, marked) =
            where_the_row_is(&home, made.kept.message_row_id).expect("the copy");
        assert!(folder == archive && marked);
        assert_eq!(still_waiting(&home), vec![made.kept.message_row_id]);
    }

    #[test]
    fn test_a_copy_of_a_row_whose_move_is_waiting_copies_from_where_the_server_has_it() {
        // Moved here from the Inbox to Archive with no network, then copied
        // from Archive to Trash. The server has the message in the Inbox
        // under 42 and nowhere else, so that is what the copy names.
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        what_happens_here(&home, &a_move_of(row, 42, into_the_archive()), "Lunch")
            .expect("moved here");
        let reserved = where_the_row_is(&home, row).expect("the row").1;

        let made = what_happens_here(
            &home,
            &AWaitingMove {
                from_folder_path: "Archive".to_string(),
                uid: reserved,
                ..a_move_of(
                    row,
                    reserved,
                    WhatAWaitingMoveDoes::Copy {
                        into_folder_path: "Trash".to_string(),
                    },
                )
            },
            "Lunch",
        )
        .expect("copied here");

        assert_eq!(
            (made.kept.from_folder_path.as_str(), made.kept.uid),
            ("INBOX", 42)
        );
    }

    #[test]
    fn test_a_copy_not_yet_at_the_server_refuses_a_second_ask_in_words() {
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let copy = a_copy_made_here(&home, row);

        let refused = what_happens_here(
            &home,
            &a_move_of(copy.message_row_id, copy.uid, into_the_archive()),
            "Lunch",
        )
        .expect_err("a move of a copy the server does not hold yet");

        let NotMadeHere::RefusedInWords(words) = refused else {
            panic!("the refusal was handed to the server-first path: {refused}");
        };
        assert_eq!(words, THAT_COPY_HAS_NOT_REACHED_THE_SERVER);
        assert_eq!(
            still_waiting(&home),
            vec![copy.message_row_id],
            "the ask was kept"
        );
    }

    #[test]
    fn test_undoing_a_copy_drops_the_copy_and_leaves_the_original() {
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let inbox = the_folder(&home, "INBOX");
        let copy = a_copy_made_here(&home, row);

        undo_here(&home, &copy).expect("undone");

        assert!(
            where_the_row_is(&home, copy.message_row_id).is_none(),
            "the copy stayed"
        );
        assert_eq!(
            where_the_row_is(&home, row),
            Some((inbox, 42, false, false))
        );
        assert!(still_waiting(&home).is_empty());
    }

    #[tokio::test]
    async fn test_a_server_that_can_copy_answers_done_and_the_copy_settles_under_its_number() {
        let server = a_server_that_can("MOVE UIDPLUS").await;
        let session = ASessionOfItsOwn::at(&server).await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let archive = the_folder(&home, "Archive");
        let copy = a_copy_made_here(&home, row);

        let replayed = replay_the_moves_waiting_for(&session, &home, "an account")
            .await
            .expect("the replay");

        assert_eq!(
            replayed
                .iter()
                .map(|(_, answer)| answer.clone())
                .collect::<Vec<_>>(),
            vec![Replayed::Done]
        );
        assert!(still_waiting(&home).is_empty());
        assert_eq!(
            where_the_row_is(&home, copy.message_row_id),
            Some((archive, 4, false, false))
        );
        let transcript = server.transcript().await;
        assert!(
            transcript
                .iter()
                .any(|line| line.to_uppercase().contains("UID COPY 42")),
            "the server was not asked to copy the message: {transcript:?}"
        );
        assert!(
            !transcript
                .iter()
                .any(|line| line.to_uppercase().contains("UID MOVE")),
            "a copy reached the server as a move: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_a_server_that_refuses_the_copy_answers_refused_and_the_undo_drops_the_copy() {
        let server = a_server_that_refuses_and_holds_nothing("UID COPY").await;
        let session = ASessionOfItsOwn::at(&server).await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let copy = a_copy_made_here(&home, row);

        let replayed = replay_the_moves_waiting_for(&session, &home, "an account")
            .await
            .expect("the replay");

        let Some((_, Replayed::Refused(reason))) = replayed.first() else {
            panic!("a refusal was read as something else: {replayed:?}");
        };
        assert!(reason.contains("would not do it"), "{reason}");
        undo_here(&home, &copy).expect("undone");
        assert!(where_the_row_is(&home, copy.message_row_id).is_none());
        assert!(still_waiting(&home).is_empty());
    }

    #[tokio::test]
    async fn test_a_server_that_hung_up_on_the_copy_leaves_it_waiting() {
        let server = a_server_that_hangs_up_on_the_change().await;
        let session = ASessionOfItsOwn::at(&server).await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let copy = a_copy_made_here(&home, row);

        let replayed = replay_the_moves_waiting_for(&session, &home, "an account")
            .await
            .expect("the replay");

        assert_eq!(
            replayed
                .iter()
                .map(|(_, answer)| answer.clone())
                .collect::<Vec<_>>(),
            vec![Replayed::NotReached]
        );
        assert_eq!(still_waiting(&home), vec![copy.message_row_id]);
        assert!(
            where_the_row_is(&home, copy.message_row_id).is_some(),
            "the copy was dropped"
        );
    }

    #[tokio::test]
    async fn test_a_refused_copy_the_destination_already_holds_is_already_done() {
        let server = a_server_that_refuses("MOVE UIDPLUS", "UID COPY").await;
        let session = ASessionOfItsOwn::at(&server).await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let archive = the_folder(&home, "Archive");
        let copy = a_copy_made_here(&home, row);

        let replayed = replay_the_moves_waiting_for(&session, &home, "an account")
            .await
            .expect("the replay");

        assert_eq!(
            replayed
                .iter()
                .map(|(_, answer)| answer.clone())
                .collect::<Vec<_>>(),
            vec![Replayed::AlreadyDone]
        );
        assert!(still_waiting(&home).is_empty());
        assert_eq!(
            where_the_row_is(&home, copy.message_row_id),
            Some((archive, 4, false, false))
        );
    }

    /// A server that refuses the one command and answers a search with
    /// nothing found, so a refusal cannot be read as the move having landed.
    async fn a_server_that_refuses_and_holds_nothing(refusing: &'static str) -> Conversation {
        conversing("* OK loopback ready\r\n", move |line| {
            let tag = line.split_whitespace().next().unwrap_or("*").to_string();
            let said = line.to_uppercase();
            if said.contains(refusing) {
                return Turn::Say(format!("{tag} NO the server would not do it\r\n"));
            }
            if said.contains("SEARCH") {
                return Turn::Say(format!("* SEARCH\r\n{tag} OK done\r\n"));
            }
            match said.split_whitespace().nth(1).unwrap_or_default() {
                "CAPABILITY" => Turn::Say(format!(
                    "* CAPABILITY IMAP4rev1 MOVE UIDPLUS\r\n{tag} OK done\r\n"
                )),
                "LOGIN" | "AUTHENTICATE" => Turn::Say(format!("{tag} OK signed in\r\n")),
                "SELECT" | "EXAMINE" => Turn::Say(format!(
                    "* 0 EXISTS\r\n* OK [UIDVALIDITY 1] valid\r\n{tag} OK [READ-WRITE] open\r\n"
                )),
                _ => Turn::Say(format!("{tag} OK done\r\n")),
            }
        })
        .await
    }

    /// A server that takes the sign-in and the SELECT and hangs up on the
    /// command that would change the message.
    async fn a_server_that_hangs_up_on_the_change() -> Conversation {
        conversing("* OK loopback ready\r\n", |line| {
            let tag = line.split_whitespace().next().unwrap_or("*").to_string();
            let said = line.to_uppercase();
            match said.split_whitespace().nth(1).unwrap_or_default() {
                "CAPABILITY" => {
                    Turn::Say(format!("* CAPABILITY IMAP4rev1 MOVE\r\n{tag} OK done\r\n"))
                }
                "LOGIN" | "AUTHENTICATE" => Turn::Say(format!("{tag} OK signed in\r\n")),
                "SELECT" | "EXAMINE" => Turn::Say(format!(
                    "* 0 EXISTS\r\n* OK [UIDVALIDITY 1] valid\r\n{tag} OK [READ-WRITE] open\r\n"
                )),
                _ => Turn::HangUp,
            }
        })
        .await
    }

    // ── A crossing: two accounts, replayed at a check of either ─────────────

    use crate::application::mail_across_accounts::for_tests::{
        ASourceServer, AnAccountAt, THE_IDENTIFIER_AS_IT_IS_HELD, THE_MESSAGE, THE_UID,
        a_source_server, the_account_it_is_going_to, the_account_it_is_leaving,
    };

    /// A crossing of the Inbox message, which the source server holds under
    /// [`THE_UID`], into the other account's Work folder.
    fn a_crossing_of(row: i64) -> AWaitingMove {
        a_move_of(row, THE_UID, into_the_other_accounts_work())
    }

    /// The crossing made here: the row in the other account's Work folder,
    /// marked, and waiting.
    fn a_crossing_made_here(home: &MessageCache, row: i64) -> AWaitingMove {
        what_happens_here(home, &a_crossing_of(row), "Lunch")
            .expect("made here")
            .kept
    }

    /// What a scripted destination does with the append it is sent.
    #[derive(Clone, Copy)]
    enum TheAppend {
        IsNeverAnswered,
    }

    /// A destination that answers however the test says, without a server:
    /// a real connection that stops answering is a closed connection, and a
    /// closed connection cannot then be asked what the folder holds, which is
    /// the whole subject of the hang-up cases.
    struct AScriptedDestination {
        the_append: TheAppend,
        /// What each successive search answers, oldest first: once before
        /// the append and once after.
        searches: tokio::sync::Mutex<std::collections::VecDeque<Result<Vec<u32>>>>,
        asked: tokio::sync::Mutex<Vec<String>>,
    }

    impl AScriptedDestination {
        fn that_hangs_up_and_then(searches: Vec<Result<Vec<u32>>>) -> Self {
            Self {
                the_append: TheAppend::IsNeverAnswered,
                searches: tokio::sync::Mutex::new(searches.into()),
                asked: tokio::sync::Mutex::new(Vec::new()),
            }
        }
    }

    /// One end of a crossing as the seam hands it out: a loopback server, or
    /// the scripted destination above.
    enum AnEnd {
        AServer(AnAccountAt),
        Scripted(AScriptedDestination),
    }

    fn not_a_source() -> Error {
        Error::Other("a scripted destination holds no message to fetch".to_string())
    }

    impl TheAccountItIsIn for AnEnd {
        async fn the_headers_of(
            &self,
            folder: &str,
            uids: &[u32],
        ) -> Result<Vec<crate::service::protocols::imap::ImapMessage>> {
            match self {
                Self::AServer(at) => at.the_headers_of(folder, uids).await,
                Self::Scripted(_) => Err(not_a_source()),
            }
        }

        async fn the_bytes_of(&self, folder: &str, uid: u32) -> Result<Vec<u8>> {
            match self {
                Self::AServer(at) => at.the_bytes_of(folder, uid).await,
                Self::Scripted(_) => Err(not_a_source()),
            }
        }
    }

    impl TheAccountItIsGoingTo for AnEnd {
        async fn take_this_message(
            &self,
            into: &str,
            flags: Option<&str>,
            arrived: Option<&str>,
            raw: &[u8],
        ) -> Result<()> {
            match self {
                Self::AServer(at) => at.take_this_message(into, flags, arrived, raw).await,
                Self::Scripted(scripted) => {
                    scripted.asked.lock().await.push(format!("APPEND {into}"));
                    match scripted.the_append {
                        TheAppend::IsNeverAnswered => Err(Error::Network(
                            "the connection to the mail server failed".to_string(),
                        )),
                    }
                }
            }
        }

        async fn which_messages_carry(&self, folder: &str, message_id: &str) -> Result<Vec<u32>> {
            match self {
                Self::AServer(at) => at.which_messages_carry(folder, message_id).await,
                Self::Scripted(scripted) => {
                    scripted
                        .asked
                        .lock()
                        .await
                        .push(format!("SEARCH {folder} {message_id}"));
                    scripted
                        .searches
                        .lock()
                        .await
                        .pop_front()
                        .unwrap_or_else(|| Ok(Vec::new()))
                }
            }
        }
    }

    impl TheAccountItIsLeaving for AnEnd {
        async fn take_it_off_the_server(
            &self,
            folder: &str,
            uid: u32,
        ) -> Result<crate::service::protocols::imap::LetGo> {
            match self {
                Self::AServer(at) => at.take_it_off_the_server(folder, uid).await,
                Self::Scripted(_) => Err(not_a_source()),
            }
        }
    }

    /// The accounts a crossing's replay can open: each by name with its end,
    /// and the ones whose server cannot be signed in to now.
    struct TheseAccounts {
        open: Vec<(&'static str, Arc<AnEnd>)>,
        unreachable: Vec<&'static str>,
    }

    impl OpensASession for TheseAccounts {
        type Session = AnEnd;

        async fn session_for(&self, account_id: &str) -> ASessionFor<AnEnd> {
            if self.unreachable.contains(&account_id) {
                return ASessionFor::CouldNotBeOpened(Error::Network(
                    "the server could not be reached".to_string(),
                ));
            }
            match self.open.iter().find(|(id, _)| *id == account_id) {
                Some((_, end)) => ASessionFor::Open(end.clone()),
                None => ASessionFor::NotSetUpHere,
            }
        }
    }

    /// The source account's server holding the message, and the destination
    /// account at the given server, both open.
    async fn two_accounts_at(source: &Conversation, destination: &Conversation) -> TheseAccounts {
        TheseAccounts {
            open: vec![
                (
                    "an account",
                    Arc::new(AnEnd::AServer(the_account_it_is_leaving(source).await)),
                ),
                (
                    "another account",
                    Arc::new(AnEnd::AServer(
                        the_account_it_is_going_to(destination).await,
                    )),
                ),
            ],
            unreachable: Vec::new(),
        }
    }

    /// The source at a real server and the destination scripted.
    async fn a_source_and_a_scripted_destination(
        source: &Conversation,
        destination: AScriptedDestination,
    ) -> TheseAccounts {
        TheseAccounts {
            open: vec![
                (
                    "an account",
                    Arc::new(AnEnd::AServer(the_account_it_is_leaving(source).await)),
                ),
                ("another account", Arc::new(AnEnd::Scripted(destination))),
            ],
            unreachable: Vec::new(),
        }
    }

    /// A destination server that holds nothing carrying the identifier until
    /// an append has arrived, and number 9 after it: what an arrival looks
    /// like from the destination's side.
    async fn a_destination_that_takes_it() -> Conversation {
        let appended = Arc::new(std::sync::atomic::AtomicBool::new(false));
        conversing("* OK loopback ready\r\n", move |line| {
            let tag = line.split_whitespace().next().unwrap_or("*").to_string();
            let said = line.to_uppercase();
            let verb = said.split_whitespace().nth(1).unwrap_or_default();
            match verb {
                "CAPABILITY" => Turn::Say(format!(
                    "* CAPABILITY IMAP4rev1 UIDPLUS\r\n{tag} OK done\r\n"
                )),
                "LOGIN" | "AUTHENTICATE" => Turn::Say(format!("{tag} OK signed in\r\n")),
                "SELECT" | "EXAMINE" => Turn::Say(format!(
                    "* 0 EXISTS\r\n* OK [UIDVALIDITY 1] valid\r\n{tag} OK [READ-WRITE] open\r\n"
                )),
                "APPEND" => {
                    appended.store(true, std::sync::atomic::Ordering::SeqCst);
                    Turn::TakingALiteral {
                        done: format!("{tag} OK saved\r\n"),
                    }
                }
                _ if said.contains("SEARCH") => {
                    if appended.load(std::sync::atomic::Ordering::SeqCst) {
                        Turn::Say(format!("* SEARCH 9\r\n{tag} OK done\r\n"))
                    } else {
                        Turn::Say(format!("* SEARCH\r\n{tag} OK done\r\n"))
                    }
                }
                "LOGOUT" => Turn::Say(format!("* BYE signing off\r\n{tag} OK done\r\n")),
                _ => Turn::Say(format!("{tag} BAD unscripted\r\n")),
            }
        })
        .await
    }

    /// Every answer of a replay, in order.
    fn answers(replayed: &[(AWaitingMove, Replayed)]) -> Vec<Replayed> {
        replayed.iter().map(|(_, answer)| answer.clone()).collect()
    }

    /// Whether the source was told to give the message up.
    async fn the_source_was_told_to_let_go(source: &Conversation) -> bool {
        source.was_told("UID EXPUNGE 4").await || source.was_told("UID STORE 4 +FLAGS").await
    }

    #[test]
    fn test_a_crossing_made_here_moves_the_row_into_the_other_accounts_folder_and_waits() {
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, THE_UID);
        let work = the_other_accounts_work(&home);

        let made = what_happens_here(&home, &a_crossing_of(row), "Lunch").expect("made here");

        assert_eq!(made.shown, "Moved to Work in Home: Lunch");
        let (folder, uid, deleted, marked) = where_the_row_is(&home, row).expect("the row");
        assert!(
            folder == work && !deleted && marked && uid != THE_UID,
            "(folder, uid, deleted, filed here) = {:?}",
            (folder, uid, deleted, marked)
        );
        let waiting = home
            .crossings_waiting_touching("another account")
            .expect("the waiting crossings");
        assert_eq!(waiting, vec![made.kept]);
        assert_eq!(
            (waiting[0].from_folder_path.as_str(), waiting[0].uid),
            ("INBOX", THE_UID),
            "the crossing forgot where the server has the message"
        );
    }

    #[test]
    fn test_a_copy_across_accounts_made_here_is_a_marked_copy_in_the_other_accounts_folder() {
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, THE_UID);
        let inbox = the_folder(&home, "INBOX");
        let work = the_other_accounts_work(&home);

        let made = what_happens_here(
            &home,
            &a_move_of(row, THE_UID, a_copy_into_the_other_accounts_work()),
            "Lunch",
        )
        .expect("copied here");

        assert_eq!(made.shown, "Copied to Work in Home: Lunch");
        assert_ne!(made.kept.message_row_id, row);
        assert_eq!(
            where_the_row_is(&home, row),
            Some((inbox, THE_UID, false, false)),
            "the original moved"
        );
        let (folder, _, _, marked) =
            where_the_row_is(&home, made.kept.message_row_id).expect("the copy");
        assert!(folder == work && marked);
    }

    #[test]
    fn test_a_second_ask_about_a_row_whose_crossing_is_waiting_is_refused_in_words() {
        // The row is in Work here and at neither server yet as the row says;
        // a move from Work would name a folder and a number the other
        // account's server has never heard of.
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, THE_UID);
        a_crossing_made_here(&home, row);
        let reserved = where_the_row_is(&home, row).expect("the row").1;

        let refused = what_happens_here(
            &home,
            &AWaitingMove {
                account_id: "another account".to_string(),
                from_folder_path: "Work".to_string(),
                uid: reserved,
                ..a_move_of(row, reserved, WhatAWaitingMoveDoes::DeleteOutright)
            },
            "Lunch",
        )
        .expect_err("a delete of a row still on its way");

        let NotMadeHere::RefusedInWords(words) = refused else {
            panic!("the refusal was handed to the server-first path: {refused}");
        };
        assert_eq!(words, THAT_MOVE_TO_ANOTHER_ACCOUNT_HAS_NOT_FINISHED);
        assert_eq!(
            home.crossings_waiting_touching("an account")
                .expect("the waiting crossings")
                .len(),
            1,
            "the crossing was replaced"
        );
    }

    #[tokio::test]
    async fn test_a_crossing_that_lands_answers_done_and_the_row_settles_in_the_other_account() {
        // Fetched from the source, appended at the destination, the
        // identifier read back under 9, removed at the source, the bytes
        // let go: the whole crossing from a waiting row with no bytes held.
        let source = a_source_server(ASourceServer::default()).await;
        let destination = a_destination_that_takes_it().await;
        let accounts = two_accounts_at(&source, &destination).await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, THE_UID);
        let work = the_other_accounts_work(&home);
        a_crossing_made_here(&home, row);

        let replayed = replay_the_crossings_waiting_for(&accounts, &home, "an account")
            .await
            .expect("the replay");

        assert_eq!(answers(&replayed), vec![Replayed::Done]);
        assert!(still_waiting_crossings(&home).is_empty());
        assert_eq!(
            where_the_row_is(&home, row),
            Some((work, 9, false, false)),
            "(folder, uid, deleted, filed here): the row is not the message the other \
             account holds under 9"
        );
        assert!(
            home.the_move_left_unfinished_for(row)
                .expect("read")
                .is_none(),
            "the bytes were kept after the crossing landed"
        );
        assert!(
            source.was_told("UID EXPUNGE 4").await,
            "the message was not removed at the source"
        );
        assert!(
            destination
                .transcript()
                .await
                .iter()
                .any(|line| line.to_uppercase().contains("APPEND")),
            "the message never reached the destination"
        );
        assert!(
            source
                .transcript()
                .await
                .iter()
                .any(|line| line.to_uppercase().contains("BODY.PEEK[]")),
            "nothing was fetched from the source, so what was appended came from nowhere"
        );
    }

    #[tokio::test]
    async fn test_a_crossing_is_replayed_at_a_check_of_the_destination_account() {
        // The other account's check is this program in front of the
        // destination; the source is opened from there.
        let source = a_source_server(ASourceServer::default()).await;
        let destination = a_destination_that_takes_it().await;
        let accounts = two_accounts_at(&source, &destination).await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, THE_UID);
        a_crossing_made_here(&home, row);

        let replayed = replay_the_crossings_waiting_for(&accounts, &home, "another account")
            .await
            .expect("the replay");

        assert_eq!(answers(&replayed), vec![Replayed::Done]);
        assert!(the_source_was_told_to_let_go(&source).await);
    }

    #[tokio::test]
    async fn test_a_destination_that_refuses_the_append_answers_refused_and_the_undo_puts_the_row_back()
     {
        let source = a_source_server(ASourceServer::default()).await;
        let destination = a_server_that_refuses("UIDPLUS", "APPEND").await;
        let accounts = two_accounts_at(&source, &destination).await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, THE_UID);
        let inbox = the_folder(&home, "INBOX");
        let waiting = a_crossing_made_here(&home, row);

        let replayed = replay_the_crossings_waiting_for(&accounts, &home, "an account")
            .await
            .expect("the replay");

        let Some((_, Replayed::Refused(reason))) = replayed.first() else {
            panic!("a refusal was read as something else: {replayed:?}");
        };
        assert!(reason.contains("could not append"), "{reason}");
        assert!(
            !the_source_was_told_to_let_go(&source).await,
            "the source was told to give up a message the destination refused"
        );
        undo_here(&home, &waiting).expect("undone");
        assert_eq!(
            where_the_row_is(&home, row),
            Some((inbox, THE_UID, false, false))
        );
        assert!(still_waiting_crossings(&home).is_empty());
        assert!(
            home.the_move_left_unfinished_for(row)
                .expect("read")
                .is_none(),
            "the bytes were kept after the crossing was undone"
        );
    }

    #[tokio::test]
    async fn test_a_destination_that_hung_up_and_then_holds_it_answers_done_and_removes_at_the_source()
     {
        let source = a_source_server(ASourceServer::default()).await;
        let accounts = a_source_and_a_scripted_destination(
            &source,
            AScriptedDestination::that_hangs_up_and_then(vec![Ok(vec![]), Ok(vec![9])]),
        )
        .await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, THE_UID);
        a_crossing_made_here(&home, row);

        let replayed = replay_the_crossings_waiting_for(&accounts, &home, "an account")
            .await
            .expect("the replay");

        assert_eq!(answers(&replayed), vec![Replayed::Done]);
        assert!(
            source.was_told("UID EXPUNGE 4").await,
            "the destination was found to hold the message and the source still has it"
        );
    }

    #[tokio::test]
    async fn test_a_destination_that_hung_up_and_does_not_hold_it_answers_refused() {
        let source = a_source_server(ASourceServer::default()).await;
        let accounts = a_source_and_a_scripted_destination(
            &source,
            AScriptedDestination::that_hangs_up_and_then(vec![Ok(vec![]), Ok(vec![])]),
        )
        .await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, THE_UID);
        a_crossing_made_here(&home, row);

        let replayed = replay_the_crossings_waiting_for(&accounts, &home, "an account")
            .await
            .expect("the replay");

        let Some((_, Replayed::Refused(reason))) = replayed.first() else {
            panic!(
                "an append the destination does not hold was read as something else: {replayed:?}"
            );
        };
        assert!(reason.contains("Home stopped answering"), "{reason}");
        assert!(!the_source_was_told_to_let_go(&source).await);
    }

    #[tokio::test]
    async fn test_a_destination_that_hung_up_and_cannot_be_asked_answers_not_reached_and_the_bytes_stay()
     {
        // The message may be in one place or in two. Not a refusal: a
        // refusal is undone, and undoing this would put the row back here
        // as if the message were nowhere else.
        let source = a_source_server(ASourceServer::default()).await;
        let accounts = a_source_and_a_scripted_destination(
            &source,
            AScriptedDestination::that_hangs_up_and_then(vec![
                Ok(vec![]),
                Err(Error::Protocol("no search here".to_string())),
            ]),
        )
        .await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, THE_UID);
        let work = the_other_accounts_work(&home);
        a_crossing_made_here(&home, row);

        let replayed = replay_the_crossings_waiting_for(&accounts, &home, "an account")
            .await
            .expect("the replay");

        assert_eq!(answers(&replayed), vec![Replayed::NotReached]);
        assert_eq!(still_waiting_crossings(&home), vec![row]);
        let held = home
            .the_move_left_unfinished_for(row)
            .expect("read")
            .expect("the bytes are held for the next check");
        assert_eq!(held.raw, THE_MESSAGE.as_bytes());
        assert_eq!(held.was_there_before, Some(Vec::new()));
        let (folder, _, _, marked) = where_the_row_is(&home, row).expect("the row");
        assert!(
            folder == work && marked,
            "the row was put back with nobody having refused"
        );
        assert!(!the_source_was_told_to_let_go(&source).await);
    }

    #[tokio::test]
    async fn test_a_crossing_resumed_from_held_bytes_fetches_nothing_from_the_source() {
        // A restart between the fetch and the ending: the bytes are here,
        // the destination is asked first, sent the message since it does not
        // hold it, and the source is asked to let go. Nothing is fetched.
        let source = a_source_server(ASourceServer::default()).await;
        let destination = a_destination_that_takes_it().await;
        let accounts = two_accounts_at(&source, &destination).await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, THE_UID);
        let work = the_other_accounts_work(&home);
        a_crossing_made_here(&home, row);
        home.keep_the_message_while_it_moves(
            &crate::data::message_cache::moves_in_flight::AMoveStarting {
                message_row_id: row,
                to_account_id: "another account",
                to_folder: "Work",
                flags: Some("(\\Seen)"),
                arrived: None,
                was_there_before: Some(&[]),
                raw: THE_MESSAGE.as_bytes(),
            },
        )
        .expect("the bytes held from the earlier run");

        let replayed = replay_the_crossings_waiting_for(&accounts, &home, "an account")
            .await
            .expect("the replay");

        assert_eq!(answers(&replayed), vec![Replayed::Done]);
        assert!(
            !source
                .transcript()
                .await
                .iter()
                .any(|line| line.to_uppercase().contains("BODY.PEEK[]")),
            "the message was fetched from the source again, so the held bytes bought nothing"
        );
        assert!(source.was_told("UID EXPUNGE 4").await);
        assert_eq!(where_the_row_is(&home, row), Some((work, 9, false, false)));
        assert!(
            home.the_move_left_unfinished_for(row)
                .expect("read")
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_a_copy_across_accounts_lands_with_nothing_removed_at_the_source() {
        let source = a_source_server(ASourceServer::default()).await;
        let destination = a_destination_that_takes_it().await;
        let accounts = two_accounts_at(&source, &destination).await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, THE_UID);
        let inbox = the_folder(&home, "INBOX");
        let work = the_other_accounts_work(&home);
        let copy = what_happens_here(
            &home,
            &a_move_of(row, THE_UID, a_copy_into_the_other_accounts_work()),
            "Lunch",
        )
        .expect("copied here")
        .kept;

        let replayed = replay_the_crossings_waiting_for(&accounts, &home, "an account")
            .await
            .expect("the replay");

        assert_eq!(answers(&replayed), vec![Replayed::Done]);
        let said = source.transcript().await.join("\n").to_uppercase();
        assert!(
            !said.contains("STORE") && !said.contains("EXPUNGE"),
            "{said}"
        );
        assert_eq!(
            where_the_row_is(&home, row),
            Some((inbox, THE_UID, false, false))
        );
        assert_eq!(
            where_the_row_is(&home, copy.message_row_id),
            Some((work, 9, false, false))
        );
        assert!(still_waiting_crossings(&home).is_empty());
    }

    #[tokio::test]
    async fn test_a_message_too_large_to_hold_is_fetched_and_the_store_says_so() {
        // The store's ceiling, through the first step: the message still
        // comes back for the append, and the answer says nothing could be
        // resumed from here. The window keeps such a message out of the
        // queue on its size at the key; this is the step's own word.
        use crate::application::mail_across_accounts::{
            TheMoveAsThisProgramRecordsIt, fetch_and_keep,
        };
        use crate::data::message_cache::moves_in_flight::Held;
        let source = a_source_server(ASourceServer::default()).await;
        let destination = a_destination_that_takes_it().await;
        let home = TempHome::named("wixen_moves_waiting_ceiling_", |dir| {
            a_cache_at(dir).keeping_no_move_larger_than(1)
        });
        let row = a_message_in_the_inbox(&home, THE_UID);
        let source_end = the_account_it_is_leaving(&source).await;
        let destination_end = the_account_it_is_going_to(&destination).await;

        let fetched = fetch_and_keep(
            &source_end,
            &destination_end,
            "INBOX",
            THE_UID,
            "Work",
            TheMoveAsThisProgramRecordsIt {
                cache: Some(&home),
                row,
                from_account_id: "an account",
                to_account_id: "another account",
            },
        )
        .await
        .expect("the message fetched");

        assert_eq!(fetched.held, Some(Held::TooLargeToHold));
        assert_eq!(fetched.message.raw, THE_MESSAGE.as_bytes());
        assert!(
            home.the_move_left_unfinished_for(row)
                .expect("read")
                .is_none(),
            "a message over the ceiling was kept anyway"
        );
    }

    #[tokio::test]
    async fn test_two_crossings_are_replayed_in_the_order_they_were_asked() {
        // Two source accounts, each holding its message, both going to the
        // other account's Work; replayed at the destination's check, in the
        // order asked.
        let first_source = a_source_server(ASourceServer::default()).await;
        let second_source = a_source_server(ASourceServer::default()).await;
        let destination = a_destination_that_takes_it().await;
        let mut accounts = two_accounts_at(&first_source, &destination).await;
        accounts.open.push((
            "a third account",
            Arc::new(AnEnd::AServer(
                the_account_it_is_leaving(&second_source).await,
            )),
        ));
        let home = a_cache();
        home.save_folder(&CachedFolder {
            id: 0,
            account_id: "a third account".to_string(),
            name: "INBOX".to_string(),
            path: "INBOX".to_string(),
            folder_type: "Inbox".to_string(),
            unread_count: 0,
            total_count: 0,
        })
        .expect("the third account's inbox");
        let first = a_message_in_the_inbox(&home, THE_UID);
        let second = home
            .save_message(&CachedMessage {
                id: 0,
                uid: THE_UID,
                folder_id: home
                    .get_folder("a third account", "INBOX")
                    .expect("the folder")
                    .expect("it is there")
                    .id,
                message_id: THE_IDENTIFIER_AS_IT_IS_HELD.to_string(),
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
            .expect("the second message");
        what_happens_here(
            &home,
            &AWaitingMove {
                asked_at: "2026-09-19T06:00:01Z".to_string(),
                ..a_crossing_of(first)
            },
            "Lunch",
        )
        .expect("made here");
        what_happens_here(
            &home,
            &AWaitingMove {
                account_id: "a third account".to_string(),
                asked_at: "2026-09-19T06:00:02Z".to_string(),
                ..a_crossing_of(second)
            },
            "Lunch",
        )
        .expect("made here");

        let replayed = replay_the_crossings_waiting_for(&accounts, &home, "another account")
            .await
            .expect("the replay");

        assert_eq!(
            replayed
                .iter()
                .map(|(waiting, _)| waiting.message_row_id)
                .collect::<Vec<_>>(),
            vec![first, second]
        );
        assert_eq!(answers(&replayed), vec![Replayed::Done, Replayed::Done]);
        assert!(the_source_was_told_to_let_go(&first_source).await);
        assert!(the_source_was_told_to_let_go(&second_source).await);
    }

    #[tokio::test]
    async fn test_a_destination_that_already_held_the_identifier_reads_only_a_new_number_as_the_arrival()
     {
        // The folder already holds number 4 carrying the identifier, and the
        // append's answer never comes; afterwards it still holds 4 and
        // nothing new. That is not an arrival, and nothing is removed at the
        // source (T-11-91). A destination that says yes to its own append
        // is a different matter and is done on that word.
        let source = a_source_server(ASourceServer::default()).await;
        let accounts = a_source_and_a_scripted_destination(
            &source,
            AScriptedDestination::that_hangs_up_and_then(vec![Ok(vec![4]), Ok(vec![4])]),
        )
        .await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, THE_UID);
        a_crossing_made_here(&home, row);

        let replayed = replay_the_crossings_waiting_for(&accounts, &home, "an account")
            .await
            .expect("the replay");

        assert!(
            matches!(replayed.first(), Some((_, Replayed::Refused(_)))),
            "a message that was in the folder before the send was counted as the one \
             that was sent: {replayed:?}"
        );
        assert!(!the_source_was_told_to_let_go(&source).await);

        let source = a_source_server(ASourceServer::default()).await;
        let destination = a_server_that_can("UIDPLUS").await;
        let accounts = two_accounts_at(&source, &destination).await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, THE_UID);
        a_crossing_made_here(&home, row);

        let replayed = replay_the_crossings_waiting_for(&accounts, &home, "an account")
            .await
            .expect("the replay");

        assert_eq!(answers(&replayed), vec![Replayed::Done]);
        assert!(
            where_the_row_is(&home, row).is_none(),
            "the destination held 4 before and after, so which number is the message \
             cannot be told; the row goes for the next read to bring down"
        );
    }

    #[tokio::test]
    async fn test_a_crossing_whose_other_account_is_gone_is_refused_and_the_undo_puts_the_row_back()
    {
        let source = a_source_server(ASourceServer::default()).await;
        let accounts = TheseAccounts {
            open: vec![(
                "an account",
                Arc::new(AnEnd::AServer(the_account_it_is_leaving(&source).await)),
            )],
            unreachable: Vec::new(),
        };
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, THE_UID);
        let inbox = the_folder(&home, "INBOX");
        let waiting = a_crossing_made_here(&home, row);

        let replayed = replay_the_crossings_waiting_for(&accounts, &home, "an account")
            .await
            .expect("the replay");

        assert_eq!(
            answers(&replayed),
            vec![Replayed::Refused(
                ONE_OF_THE_TWO_ACCOUNTS_IS_GONE.to_string()
            )]
        );
        assert!(
            !source
                .transcript()
                .await
                .iter()
                .any(|line| line.to_uppercase().contains("FETCH")),
            "the source was asked for a message with nowhere to send it"
        );
        undo_here(&home, &waiting).expect("undone");
        assert_eq!(
            where_the_row_is(&home, row),
            Some((inbox, THE_UID, false, false))
        );
    }

    #[tokio::test]
    async fn test_a_crossing_whose_other_account_cannot_be_signed_in_to_waits() {
        let source = a_source_server(ASourceServer::default()).await;
        let accounts = TheseAccounts {
            open: vec![(
                "an account",
                Arc::new(AnEnd::AServer(the_account_it_is_leaving(&source).await)),
            )],
            unreachable: vec!["another account"],
        };
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, THE_UID);
        a_crossing_made_here(&home, row);

        let replayed = replay_the_crossings_waiting_for(&accounts, &home, "an account")
            .await
            .expect("the replay");

        assert_eq!(answers(&replayed), vec![Replayed::NotReached]);
        assert_eq!(still_waiting_crossings(&home), vec![row]);
    }

    #[tokio::test]
    async fn test_a_held_crossing_nothing_can_settle_is_refused_with_where_to_look() {
        // Held bytes for a message with no identifier: the destination
        // cannot be asked, so nothing is sent to either server, the row
        // comes back, and the sentence says where to look.
        let source = a_source_server(ASourceServer::default()).await;
        let destination = a_destination_that_takes_it().await;
        let accounts = two_accounts_at(&source, &destination).await;
        let home = a_cache();
        let row = home
            .save_message(&CachedMessage {
                id: 0,
                uid: THE_UID,
                folder_id: the_folder(&home, "INBOX"),
                message_id: String::new(),
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
            .expect("a message with no identifier");
        a_crossing_made_here(&home, row);
        home.keep_the_message_while_it_moves(
            &crate::data::message_cache::moves_in_flight::AMoveStarting {
                message_row_id: row,
                to_account_id: "another account",
                to_folder: "Work",
                flags: None,
                arrived: None,
                was_there_before: Some(&[]),
                raw: THE_MESSAGE.as_bytes(),
            },
        )
        .expect("the bytes held from the earlier run");

        let replayed = replay_the_crossings_waiting_for(&accounts, &home, "an account")
            .await
            .expect("the replay");

        let Some((_, Replayed::Refused(reason))) = replayed.first() else {
            panic!("a crossing nothing can settle was read as something else: {replayed:?}");
        };
        assert!(reason.contains("Look in Work in Home"), "{reason}");
        assert!(
            !destination
                .transcript()
                .await
                .iter()
                .any(|line| line.to_uppercase().contains("APPEND")),
            "something was sent to the destination about a crossing nothing can settle"
        );
        assert!(!the_source_was_told_to_let_go(&source).await);
    }

    #[test]
    fn test_an_append_nobody_could_settle_is_not_reached_and_never_a_refusal() {
        // A message possibly in two places must not be put back here as if
        // it were nowhere: a refusal is undone, and this must not be.
        let said_as = ACrossingSaidAs {
            into: "Work",
            to_account: "Home",
            still_in: "INBOX",
            subject: "Lunch",
        };
        assert_eq!(
            what_a_crossing_answered(
                &MovedAcross::ItIsNotKnownWhereItIs("no search there".to_string()),
                &said_as
            ),
            Replayed::NotReached
        );
        assert_eq!(
            what_a_crossing_answered(&MovedAcross::ItArrivedAndTheSourceLetItGo, &said_as),
            Replayed::Done
        );
        assert!(matches!(
            what_a_crossing_answered(
                &MovedAcross::TheDestinationRefusedIt("over quota".to_string()),
                &said_as
            ),
            Replayed::Refused(_)
        ));
        let Replayed::DoneWithSomethingToSay(said) = what_a_crossing_answered(
            &MovedAcross::ItArrivedAndTheSourceWouldNotLetGo("read-only".to_string()),
            &said_as,
        ) else {
            panic!("a message left in both places was not said");
        };
        assert!(
            said.contains("Work in Home") && said.contains("INBOX"),
            "{said}"
        );
    }

    /// The rows of every crossing still waiting that either account is one
    /// end of.
    fn still_waiting_crossings(cache: &MessageCache) -> Vec<i64> {
        cache
            .crossings_waiting_touching("an account")
            .expect("the waiting crossings")
            .iter()
            .map(|waiting| waiting.message_row_id)
            .collect()
    }

    #[test]
    fn test_a_server_that_did_it_is_done() {
        assert_eq!(
            what_a_replay_answered(&Ok(()), WhereItIsNow::NotThere),
            Replayed::Done
        );
    }

    #[test]
    fn test_a_refusal_for_a_message_already_where_it_was_going_is_already_done() {
        // The server did it before the restart, or another client did. A
        // refusal read as a refusal would put the row back in a folder the
        // server no longer has the message in.
        assert_eq!(
            what_a_replay_answered(
                &Err(Error::Protocol("NO no such message".to_string())),
                WhereItIsNow::WhereTheMoveWantedIt
            ),
            Replayed::AlreadyDone
        );
    }

    #[test]
    fn test_a_refusal_for_a_message_still_where_it_was_is_a_refusal_with_its_words() {
        // The words as the flag queue carries them, the error's own display,
        // so the two refusals a person can hear are worded the same way.
        let Replayed::Refused(reason) = what_a_replay_answered(
            &Err(Error::Protocol("NO over quota".to_string())),
            WhereItIsNow::NotThere,
        ) else {
            panic!("a refusal for a message still where it was was read as something else");
        };
        assert!(reason.contains("NO over quota"), "{reason}");
    }

    #[test]
    fn test_a_server_never_reached_leaves_the_move_waiting() {
        assert_eq!(
            what_a_replay_answered(
                &Err(Error::Network("the connection went".to_string())),
                WhereItIsNow::NotThere
            ),
            Replayed::NotReached
        );
        assert_eq!(
            what_a_replay_answered(
                &Err(Error::Authentication("the token has expired".to_string())),
                WhereItIsNow::NotThere
            ),
            Replayed::NotReached
        );
    }

    #[test]
    fn test_this_computers_own_gate_undoes_the_move_rather_than_holding_the_check() {
        // The flag queue keeps a change the gate refused, because the setting
        // is what clears it. A waiting move cannot be kept that way: the
        // check ends at a move it cannot replay, so a move kept under a
        // closed gate would stop the account being read until the setting
        // moved. The gate refuses at the key before anything changes; a
        // refusal met here means the setting moved in between, and the move
        // comes back and says so.
        let Replayed::Refused(reason) = what_a_replay_answered(
            &Err(Error::Security("Allow Changes is off".to_string())),
            WhereItIsNow::NotThere,
        ) else {
            panic!("the gate's refusal was read as something else");
        };
        assert!(reason.contains("Allow Changes is off"), "{reason}");
    }

    #[test]
    fn test_nothing_here_can_say_send_now() {
        // Guardrail 7, held by the shape of the type rather than by a comment.
        // The fifth, added 2026-09-19 for a crossing that landed with the
        // message left in both places, says something; it sends nothing.
        let every_answer = [
            Replayed::Done,
            Replayed::DoneWithSomethingToSay(String::new()),
            Replayed::AlreadyDone,
            Replayed::Refused(String::new()),
            Replayed::NotReached,
        ];
        assert_eq!(
            every_answer.len(),
            5,
            "a sixth answer was added to what a replay can mean. If it means \
             sending one, that is guardrail 7 and it needs an argument rather \
             than an arm"
        );
    }

    #[test]
    fn test_the_shown_line_and_the_spoken_refusal_are_plainly_different() {
        let shown = shown_when_made_here(&a_move_of(1, 42, into_the_archive()), "Lunch");
        let put_back =
            put_back_because_the_server_refused(&into_the_archive(), "Lunch", "over quota");
        assert_eq!(shown, "Moved to Archive: Lunch");
        assert_eq!(
            put_back,
            "Could not move Lunch to Archive: over quota. It is back where it was."
        );
        assert_eq!(
            shown_when_made_here(
                &a_move_of(
                    1,
                    42,
                    WhatAWaitingMoveDoes::DeleteToTrash {
                        trash_path: "Trash".to_string()
                    }
                ),
                "Lunch"
            ),
            "Moved to Trash: Lunch"
        );
        assert_eq!(
            shown_when_made_here(
                &a_move_of(1, 42, WhatAWaitingMoveDoes::DeleteOutright),
                "Lunch"
            ),
            "Deleted: Lunch"
        );
        assert_eq!(
            shown_when_made_here(&a_move_of(1, 42, into_the_other_accounts_work()), "Lunch"),
            "Moved to Work in Home: Lunch"
        );
        assert_eq!(
            put_back_because_the_server_refused(
                &into_the_other_accounts_work(),
                "Lunch",
                "over quota"
            ),
            "Could not move Lunch to Work in Home: over quota. It is back where it was."
        );
        assert_eq!(
            put_back_because_the_server_refused(
                &WhatAWaitingMoveDoes::DeleteOutright,
                "Lunch",
                "over quota"
            ),
            "Could not delete Lunch: over quota. It is back where it was."
        );
    }

    #[test]
    fn test_made_here_moves_the_row_and_keeps_the_move_waiting() {
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let archive = the_folder(&home, "Archive");

        let made = what_happens_here(&home, &a_move_of(row, 42, into_the_archive()), "Lunch")
            .expect("made here");

        let (folder, _, deleted, marked) = where_the_row_is(&home, row).expect("the row");
        assert_eq!(folder, archive, "the row is not in Archive");
        assert!(!deleted && marked, "the row is not marked as filed here");
        assert_eq!(still_waiting(&home), vec![row]);
        assert_eq!(made.shown, "Moved to Archive: Lunch");
        assert_eq!(made.kept.message_row_id, row);
    }

    #[test]
    fn test_made_here_a_delete_outright_marks_the_row_deleted_and_waits() {
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let inbox = the_folder(&home, "INBOX");

        what_happens_here(
            &home,
            &a_move_of(row, 42, WhatAWaitingMoveDoes::DeleteOutright),
            "Lunch",
        )
        .expect("made here");

        assert_eq!(
            where_the_row_is(&home, row),
            Some((inbox, 42, true, false)),
            "(folder, uid, deleted, filed here): the row should stay in the Inbox \
             under its number, deleted, for the next read to forget"
        );
        assert_eq!(still_waiting(&home), vec![row]);
    }

    #[test]
    fn test_undone_here_puts_the_row_back_and_stops_the_wait() {
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let inbox = the_folder(&home, "INBOX");
        let waiting = a_move_of(row, 42, into_the_archive());
        what_happens_here(&home, &waiting, "Lunch").expect("made here");

        undo_here(&home, &waiting).expect("undone");

        assert_eq!(
            where_the_row_is(&home, row),
            Some((inbox, 42, false, false))
        );
        assert!(still_waiting(&home).is_empty());
    }

    #[tokio::test]
    async fn test_a_server_that_can_move_answers_done_and_the_row_stops_waiting() {
        let server = a_server_that_can("MOVE UIDPLUS").await;
        let session = ASessionOfItsOwn::at(&server).await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let archive = the_folder(&home, "Archive");
        what_happens_here(&home, &a_move_of(row, 42, into_the_archive()), "Lunch")
            .expect("made here");

        let replayed = replay_the_moves_waiting_for(&session, &home, "an account")
            .await
            .expect("the replay");

        assert_eq!(
            replayed
                .iter()
                .map(|(_, answer)| answer.clone())
                .collect::<Vec<_>>(),
            vec![Replayed::Done]
        );
        assert!(
            still_waiting(&home).is_empty(),
            "the move went and still waits"
        );
        // The scripted server finds one message, number 4, for any search:
        // the row is that message now, unmarked, so the next read of Archive
        // neither fetches it again nor forgets it.
        assert_eq!(
            where_the_row_is(&home, row),
            Some((archive, 4, false, false))
        );
        let transcript = server.transcript().await;
        assert!(
            transcript
                .iter()
                .any(|line| line.to_uppercase().contains("UID MOVE 42")),
            "the server was not asked to move the message: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_a_server_that_refuses_answers_refused_with_its_words_and_the_row_waits_for_the_undo()
     {
        let server = a_server_that_refuses_and_holds_nothing("UID MOVE").await;
        let session = ASessionOfItsOwn::at(&server).await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let inbox = the_folder(&home, "INBOX");
        let waiting = a_move_of(row, 42, into_the_archive());
        what_happens_here(&home, &waiting, "Lunch").expect("made here");

        let replayed = replay_the_moves_waiting_for(&session, &home, "an account")
            .await
            .expect("the replay");

        let Some((_, Replayed::Refused(reason))) = replayed.first() else {
            panic!("a refusal was read as something else: {replayed:?}");
        };
        assert!(reason.contains("would not do it"), "{reason}");
        // The undo is the window's, so it can say the sentence beside it.
        undo_here(&home, &waiting).expect("undone");
        assert_eq!(
            where_the_row_is(&home, row),
            Some((inbox, 42, false, false))
        );
        assert!(still_waiting(&home).is_empty());
    }

    #[tokio::test]
    async fn test_a_server_that_hung_up_answers_not_reached_and_the_row_goes_on_waiting() {
        let server = a_server_that_hangs_up_on_the_change().await;
        let session = ASessionOfItsOwn::at(&server).await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let archive = the_folder(&home, "Archive");
        what_happens_here(&home, &a_move_of(row, 42, into_the_archive()), "Lunch")
            .expect("made here");

        let replayed = replay_the_moves_waiting_for(&session, &home, "an account")
            .await
            .expect("the replay");

        assert_eq!(
            replayed
                .iter()
                .map(|(_, answer)| answer.clone())
                .collect::<Vec<_>>(),
            vec![Replayed::NotReached]
        );
        assert_eq!(
            still_waiting(&home),
            vec![row],
            "a move nobody was asked about was let go"
        );
        let (folder, _, _, marked) = where_the_row_is(&home, row).expect("the row");
        assert!(
            folder == archive && marked,
            "the row was moved back with nobody having refused it"
        );
    }

    #[tokio::test]
    async fn test_a_refusal_for_a_message_the_destination_already_holds_is_already_done() {
        // The scripted server refuses the move and finds number 4 for any
        // search: the message is in Archive already, so the server did it
        // before the restart. Read as done, and the row settles under 4.
        let server = a_server_that_refuses("MOVE UIDPLUS", "UID MOVE").await;
        let session = ASessionOfItsOwn::at(&server).await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let archive = the_folder(&home, "Archive");
        what_happens_here(&home, &a_move_of(row, 42, into_the_archive()), "Lunch")
            .expect("made here");

        let replayed = replay_the_moves_waiting_for(&session, &home, "an account")
            .await
            .expect("the replay");

        assert_eq!(
            replayed
                .iter()
                .map(|(_, answer)| answer.clone())
                .collect::<Vec<_>>(),
            vec![Replayed::AlreadyDone]
        );
        assert!(still_waiting(&home).is_empty());
        assert_eq!(
            where_the_row_is(&home, row),
            Some((archive, 4, false, false))
        );
    }

    #[tokio::test]
    async fn test_a_delete_to_the_trash_replays_as_a_move_into_it() {
        let server = a_server_that_can("MOVE UIDPLUS").await;
        let session = ASessionOfItsOwn::at(&server).await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let trash = the_folder(&home, "Trash");
        what_happens_here(
            &home,
            &a_move_of(
                row,
                42,
                WhatAWaitingMoveDoes::DeleteToTrash {
                    trash_path: "Trash".to_string(),
                },
            ),
            "Lunch",
        )
        .expect("made here");

        let replayed = replay_the_moves_waiting_for(&session, &home, "an account")
            .await
            .expect("the replay");

        assert_eq!(
            replayed
                .iter()
                .map(|(_, answer)| answer.clone())
                .collect::<Vec<_>>(),
            vec![Replayed::Done]
        );
        assert_eq!(where_the_row_is(&home, row), Some((trash, 4, false, false)));
        let transcript = server.transcript().await;
        assert!(
            transcript
                .iter()
                .any(|line| line.to_uppercase().contains("UID MOVE 42") && line.contains("Trash")),
            "the delete did not reach the server as a move into the trash: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_a_delete_outright_replays_as_a_removal_and_the_row_stays_for_the_read_to_forget()
    {
        let server = a_server_that_can("UIDPLUS").await;
        let session = ASessionOfItsOwn::at(&server).await;
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let inbox = the_folder(&home, "INBOX");
        what_happens_here(
            &home,
            &a_move_of(row, 42, WhatAWaitingMoveDoes::DeleteOutright),
            "Lunch",
        )
        .expect("made here");

        let replayed = replay_the_moves_waiting_for(&session, &home, "an account")
            .await
            .expect("the replay");

        assert_eq!(
            replayed
                .iter()
                .map(|(_, answer)| answer.clone())
                .collect::<Vec<_>>(),
            vec![Replayed::Done]
        );
        assert!(still_waiting(&home).is_empty());
        assert_eq!(
            where_the_row_is(&home, row),
            Some((inbox, 42, true, false)),
            "the row is what the next read of the Inbox forgets, as a delete's row always was"
        );
        let transcript = server.transcript().await;
        assert!(
            transcript
                .iter()
                .any(|line| line.to_uppercase().contains("UID EXPUNGE 42")),
            "the delete did not reach the server: {transcript:?}"
        );
    }

    #[tokio::test]
    async fn test_two_waiting_moves_are_replayed_in_the_order_they_were_asked() {
        let server = a_server_that_can("MOVE UIDPLUS").await;
        let session = ASessionOfItsOwn::at(&server).await;
        let home = a_cache();
        let first = a_message_in_the_inbox(&home, 7);
        let second = a_message_in_the_inbox(&home, 3);
        what_happens_here(
            &home,
            &AWaitingMove {
                asked_at: "2026-09-19T04:00:01Z".to_string(),
                ..a_move_of(first, 7, into_the_archive())
            },
            "Lunch",
        )
        .expect("made here");
        what_happens_here(
            &home,
            &AWaitingMove {
                asked_at: "2026-09-19T04:00:02Z".to_string(),
                ..a_move_of(second, 3, into_the_archive())
            },
            "Lunch",
        )
        .expect("made here");

        let replayed = replay_the_moves_waiting_for(&session, &home, "an account")
            .await
            .expect("the replay");

        assert_eq!(
            replayed
                .iter()
                .map(|(waiting, _)| waiting.message_row_id)
                .collect::<Vec<_>>(),
            vec![first, second]
        );
        let moves: Vec<String> = server
            .transcript()
            .await
            .into_iter()
            .filter(|line| line.to_uppercase().contains("UID MOVE"))
            .collect();
        assert_eq!(moves.len(), 2, "{moves:?}");
        assert!(
            moves[0].contains("UID MOVE 7") && moves[1].contains("UID MOVE 3"),
            "{moves:?}"
        );
    }

    /// A server as a folder read sees it: what each folder lists, and a
    /// header for anything asked for.
    struct AServerThatLists {
        inbox: Vec<u32>,
        archive: Vec<u32>,
    }

    impl crate::application::mail_sync::Mailbox for AServerThatLists {
        async fn folder_counts(
            &self,
            folder: &str,
        ) -> Result<crate::service::protocols::imap::FolderCounts> {
            let total = if folder == "Archive" {
                self.archive.len()
            } else {
                self.inbox.len()
            } as u32;
            Ok(crate::service::protocols::imap::FolderCounts { total, unread: 0 })
        }

        async fn select_folder(
            &self,
            _folder: &str,
        ) -> Result<crate::service::protocols::imap::MailboxStatus> {
            Ok(crate::service::protocols::imap::MailboxStatus {
                uid_validity: Some(1),
                highest_modseq: None,
            })
        }

        async fn what_this_server_can_do(
            &self,
        ) -> crate::service::protocols::imap::abilities::Abilities {
            crate::service::protocols::imap::abilities::Abilities::default()
        }

        async fn list_uids(&self, folder: &str) -> Result<Vec<u32>> {
            Ok(if folder == "Archive" {
                self.archive.clone()
            } else {
                self.inbox.clone()
            })
        }

        async fn list_uids_above(&self, folder: &str, after: u32) -> Result<Vec<u32>> {
            Ok(self
                .list_uids(folder)
                .await?
                .into_iter()
                .filter(|uid| *uid >= after)
                .collect())
        }

        async fn fetch_headers(
            &self,
            _folder: &str,
            uids: &[u32],
        ) -> Result<Vec<crate::service::protocols::imap::ImapMessage>> {
            Ok(uids
                .iter()
                .map(|uid| crate::service::protocols::imap::ImapMessage {
                    uid: *uid,
                    subject: "Lunch".to_string(),
                    message_id: Some(format!("lunch.{uid}@example.com")),
                    ..Default::default()
                })
                .collect())
        }

        async fn move_message(
            &self,
            _from: &str,
            _uid: u32,
            _into: &str,
        ) -> Result<crate::service::protocols::imap::Moved> {
            Ok(crate::service::protocols::imap::Moved::Moved)
        }

        async fn fetch_flags(
            &self,
            _folder: &str,
            _held: &[u32],
            _changed_since: Option<u64>,
        ) -> Result<Vec<(u32, Vec<String>)>> {
            Ok(Vec::new())
        }

        async fn fetch_message_body(&self, _folder: &str, _uid: u32) -> Result<Vec<u8>> {
            Ok(Vec::new())
        }
    }

    fn a_folder_read(path: &str) -> crate::service::protocols::imap::ImapFolder {
        crate::service::protocols::imap::ImapFolder {
            name: path.to_string(),
            display_path: path.to_string(),
            path: path.to_string(),
            folder_type: if path == "Archive" {
                crate::common::types::FolderType::Archive
            } else {
                crate::common::types::FolderType::Inbox
            },
            selectable: true,
            holds_all_mail: false,
            subscribed: true,
            delimiter: None,
        }
    }

    #[tokio::test]
    async fn test_a_read_of_the_destination_neither_forgets_nor_doubles_what_a_waiting_move_holds()
    {
        // The plan that wrote this module expected the destination's listing
        // to forget the moved row, because it took the row to keep the
        // source's number. It does not: the cache's own move gives the row a
        // number reserved from the top of the range and marks it as filed
        // here, and the forgetting reads that marker, so the read of Archive
        // leaves the row alone with no subtraction anywhere. This holds that
        // the property is the marker's and not luck, on the real sync over a
        // server that lists Archive as empty and the Inbox as still holding
        // the message.
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let archive = the_folder(&home, "Archive");
        what_happens_here(&home, &a_move_of(row, 42, into_the_archive()), "Lunch")
            .expect("made here");
        let server = AServerThatLists {
            inbox: vec![42],
            archive: Vec::new(),
        };

        crate::application::mail_sync::sync_folder(
            &server,
            &home,
            &a_folder_read("Archive"),
            archive,
            crate::application::mail_sync::INITIAL_FETCH_LIMIT,
            None,
            crate::application::mail_sync::WhatThisSyncIsFor::WhateverHasChanged,
        )
        .await
        .expect("the read of Archive");

        let (folder, _, deleted, marked) =
            where_the_row_is(&home, row).expect("the row was forgotten");
        assert!(
            folder == archive && !deleted && marked,
            "the row moved or lost its marker"
        );
        assert_eq!(still_waiting(&home), vec![row]);
    }

    #[tokio::test]
    async fn test_a_read_of_the_source_before_the_replay_would_bring_the_message_back() {
        // The other half, and the reason the replay runs before any folder
        // is listed: the Inbox still lists 42 at the server, this computer
        // no longer holds 42 in the Inbox, and a read of the Inbox brings it
        // down as new mail. Held as a measurement rather than a rule, so the
        // ordering the window keeps has a test saying what it prevents.
        let home = a_cache();
        let row = a_message_in_the_inbox(&home, 42);
        let inbox = the_folder(&home, "INBOX");
        what_happens_here(&home, &a_move_of(row, 42, into_the_archive()), "Lunch")
            .expect("made here");
        let server = AServerThatLists {
            inbox: vec![42],
            archive: Vec::new(),
        };

        crate::application::mail_sync::sync_folder(
            &server,
            &home,
            &a_folder_read("INBOX"),
            inbox,
            crate::application::mail_sync::INITIAL_FETCH_LIMIT,
            None,
            crate::application::mail_sync::WhatThisSyncIsFor::WhateverHasChanged,
        )
        .await
        .expect("the read of the Inbox");

        assert!(
            home.message_row_for_uid(inbox, 42)
                .expect("the read")
                .is_some(),
            "the measurement this test records has changed: a read of the source \
             no longer brings back a message the server still lists there"
        );
    }
}

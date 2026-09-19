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
//! # What has never been checked
//!
//! No real server has replayed a move after a restart. The cases below drive
//! a loopback server that can carry a move out, refuse it, or hang up on it,
//! which is closer than a mocked error and is not a real mail server; what a
//! real server does with a message another client changed meanwhile is the
//! tester's account to settle.

use crate::application::flag_changes_waiting::{WhyThePushFailed, why_the_push_failed};
use crate::common::{Error, Result};
use crate::data::message_cache::MessageCache;
use crate::data::message_cache::moves_waiting::{AWaitingMove, WhatAWaitingMoveDoes};

/// What a replay of a waiting move answered.
///
/// Four members and none of them means "send now": what sends a waiting move
/// is the person's own action or a check that already has a session, and
/// both call the replay directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Replayed {
    /// The server carried it out. Stop waiting.
    Done,
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
pub fn shown_when_made_here(what: &WhatAWaitingMoveDoes, subject: &str) -> String {
    match what {
        WhatAWaitingMoveDoes::Move { into_folder_path } => {
            format!("Moved to {into_folder_path}: {subject}")
        }
        WhatAWaitingMoveDoes::DeleteToTrash { .. } => format!("Moved to Trash: {subject}"),
        WhatAWaitingMoveDoes::DeleteOutright => format!("Deleted: {subject}"),
        WhatAWaitingMoveDoes::Copy { into_folder_path } => {
            format!("Copied to {into_folder_path}: {subject}")
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

/// Make the change here: the row into the folder the move names, or marked
/// deleted, or copied into the folder named, and the waiting row written.
///
/// The row leaving the list is the window's to do, since only it holds the
/// control. A row that is itself a copy not yet at the server is refused
/// with [`THAT_COPY_HAS_NOT_REACHED_THE_SERVER`].
pub fn what_happens_here(
    cache: &MessageCache,
    asked: &AWaitingMove,
    subject: &str,
) -> Result<MadeHere> {
    let already_waiting = cache.the_move_waiting_for(asked.message_row_id)?;
    if already_waiting
        .as_ref()
        .is_some_and(|waiting| waiting.what.is_a_copy())
    {
        return Err(Error::InPlainWords(
            THAT_COPY_HAS_NOT_REACHED_THE_SERVER.to_string(),
        ));
    }
    let kept = match (&asked.what, asked.what.destination()) {
        // A copy of the row, under a number reserved from the top of the
        // destination's range and marked as filed here, waiting under its own
        // row; the server copies from where it still has the original, which
        // is the original's waiting row's answer when it has one.
        (WhatAWaitingMoveDoes::Copy { .. }, Some(destination)) => {
            let into = the_folder_here(cache, &asked.account_id, destination)?;
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
        // message again nor forgets a row the server never listed.
        (_, Some(destination)) => {
            let into = the_folder_here(cache, &asked.account_id, destination)?;
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
        shown: shown_when_made_here(&kept.what, subject),
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
/// and the waiting row gone.
pub fn undo_here(cache: &MessageCache, waiting: &AWaitingMove) -> Result<()> {
    if waiting.what.is_a_copy() {
        cache.let_the_next_read_bring_it(waiting.message_row_id)?;
        return cache.stop_waiting_for_a_move(waiting.message_row_id);
    }
    let from = cache
        .get_folder(&waiting.account_id, &waiting.from_folder_path)?
        .ok_or_else(|| {
            Error::InPlainWords(format!(
                "The folder the message came from, {}, is no longer on this computer.",
                waiting.from_folder_path
            ))
        })?;
    cache.the_server_holds_it_at(waiting.message_row_id, from.id, waiting.uid)?;
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
            Replayed::Done | Replayed::AlreadyDone => {
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

    fn a_cache_at(dir: &Path) -> MessageCache {
        let cache = MessageCache::new(dir.to_path_buf(), None).expect("a cache");
        for (name, kind) in [
            ("INBOX", "Inbox"),
            ("Archive", "Archive"),
            ("Trash", "Trash"),
        ] {
            cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: "an account".to_string(),
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

        assert_eq!(refused.to_string(), THAT_COPY_HAS_NOT_REACHED_THE_SERVER);
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
        let every_answer = [
            Replayed::Done,
            Replayed::AlreadyDone,
            Replayed::Refused(String::new()),
            Replayed::NotReached,
        ];
        assert_eq!(
            every_answer.len(),
            4,
            "a fifth answer was added to what a replay can mean. If it means \
             sending one, that is guardrail 7 and it needs an argument rather \
             than an arm"
        );
    }

    #[test]
    fn test_the_shown_line_and_the_spoken_refusal_are_plainly_different() {
        let shown = shown_when_made_here(&into_the_archive(), "Lunch");
        let put_back =
            put_back_because_the_server_refused(&into_the_archive(), "Lunch", "over quota");
        assert_eq!(shown, "Moved to Archive: Lunch");
        assert_eq!(
            put_back,
            "Could not move Lunch to Archive: over quota. It is back where it was."
        );
        assert_eq!(
            shown_when_made_here(
                &WhatAWaitingMoveDoes::DeleteToTrash {
                    trash_path: "Trash".to_string()
                },
                "Lunch"
            ),
            "Moved to Trash: Lunch"
        );
        assert_eq!(
            shown_when_made_here(&WhatAWaitingMoveDoes::DeleteOutright, "Lunch"),
            "Deleted: Lunch"
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

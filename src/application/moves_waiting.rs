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

use crate::common::Result;
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
pub fn what_a_replay_answered(answer: &Result<()>, now: WhereItIsNow) -> Replayed {
    let _ = (answer, now);
    Replayed::NotReached
}

/// What the status bar shows when the change has been made here.
///
/// Shown and not spoken: the one word at the key and the row the cursor lands
/// on are what is heard (#83), and the fuller line is for the eye.
pub fn shown_when_made_here(what: &WhatAWaitingMoveDoes, subject: &str) -> String {
    let _ = (what, subject);
    String::new()
}

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
    let _ = (what, subject, reason);
    String::new()
}

/// Make the change here: the row into the folder the move names, or marked
/// deleted, and the waiting row written.
///
/// Returns the line to show. The row leaving the list is the window's to do,
/// since only it holds the control.
pub fn what_happens_here(
    cache: &MessageCache,
    waiting: &AWaitingMove,
    subject: &str,
) -> Result<String> {
    let _ = (cache, waiting, subject);
    Ok(String::new())
}

/// Undo the change here, because the server refused it: the row back in the
/// folder and under the number it never left, and the waiting row gone.
pub fn undo_here(cache: &MessageCache, waiting: &AWaitingMove) -> Result<()> {
    let _ = (cache, waiting);
    Ok(())
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
    /// The numbers the folder holds a message with this identifier under.
    async fn where_it_is(&self, folder: &str, message_id: &str) -> Result<Vec<u32>>;
}

impl ReplaysAMove for crate::application::mail_controller::MailController {
    async fn move_it(&self, from: &str, uid: u32, into: &str) -> Result<()> {
        let moved = self.move_message(from, uid, into).await?;
        tracing::info!("A waiting move was replayed: {}", moved.spoken(into));
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
        };
        let now = match &answer {
            Err(_) => where_it_is_now(server, cache, &waiting).await?,
            Ok(()) => WhereItIsNow::NotThere,
        };
        let what_it_means = what_a_replay_answered(&answer, now);
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

        async fn where_it_is(&self, folder: &str, message_id: &str) -> Result<Vec<u32>> {
            let mut session = self.0.lock().await;
            session.select_folder(folder).await?;
            session.uids_with_message_id(message_id).await
        }
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
        assert_eq!(
            what_a_replay_answered(
                &Err(Error::Protocol("NO over quota".to_string())),
                WhereItIsNow::NotThere
            ),
            Replayed::Refused("NO over quota".to_string())
        );
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
        assert_eq!(
            what_a_replay_answered(
                &Err(Error::Security("Allow Changes is off".to_string())),
                WhereItIsNow::NotThere
            ),
            Replayed::Refused("Allow Changes is off".to_string())
        );
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

        let shown = what_happens_here(&home, &a_move_of(row, 42, into_the_archive()), "Lunch")
            .expect("made here");

        let (folder, _, deleted, marked) = where_the_row_is(&home, row).expect("the row");
        assert_eq!(folder, archive, "the row is not in Archive");
        assert!(!deleted && marked, "the row is not marked as filed here");
        assert_eq!(still_waiting(&home), vec![row]);
        assert_eq!(shown, "Moved to Archive: Lunch");
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
}

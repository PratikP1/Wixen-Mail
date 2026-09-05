//! Carrying out a delete at somebody's mail server, or deciding not to.
//!
//! A message on an account whose trash this program does not recognise has
//! exactly one copy, and it is on that server. Deleting it there is the one
//! action in this program that cannot be undone by sending another command.
//!
//! # Why the order is the whole of it
//!
//! Where a deleted message goes is decided before anything is dialled, and two
//! of the four answers are refusals. Those two return here, in front of the
//! step that speaks to a server, so an account this program cannot delete
//! safely from is never even connected to.
//!
//! That order used to live as two early returns in the middle of a window
//! handler, guarded only by a check that read the handler as text. The text
//! could go on saying the refusal while the arm carried on and destroyed the
//! message, and nothing running would have known. Here the refusals and the
//! sending are the same function, so a test can hold a server that records what
//! it was told and assert that it was told nothing.

use crate::application::destinations::{
    DeletedGoesTo, Deleting, NO_FOLDERS_KNOWN_YET, NO_TRASH_FOLDER_FOUND,
    where_a_deleted_message_goes,
};
use crate::common::types::FolderType;
use crate::service::protocols::imap::Deletion;

/// Somewhere a message can be deleted that is not this computer.
///
/// One step, which is the whole of what a delete asks of a mail server. Named
/// for what it does rather than for a protocol, so the decision above it can be
/// run in a test: whether anything is sent at all is a decision, and it could
/// not be reached before without a window, an account with stored credentials
/// and a mail server to answer.
pub(crate) trait DeletesAMessage {
    /// Move it to `trash`, or take it off the server when there is none. The
    /// string in the error is what the person is shown, so it says what the
    /// server said.
    async fn delete_it(
        &self,
        folder: &str,
        uid: u32,
        trash: Option<&str>,
    ) -> std::result::Result<Deletion, String>;
}

/// What became of a delete.
///
/// Three answers rather than success and failure, because a delete that was
/// never sent is not a failure. Nothing went wrong, the message is exactly
/// where it was, and the sentence says what to do instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Deleted {
    /// Nothing was sent and nothing was dialled. The sentence says why, and
    /// what to do instead.
    NothingWasSent(&'static str),
    /// The server was asked, and this is what it did with it.
    TheServerDidThis(Deletion),
    /// The server was asked and would not.
    TheServerWouldNot(String),
}

/// Delete a message, once it is clear that deleting it means something.
///
/// The two refusals return before `server` is touched at all. That is the
/// whole point of this function: it is the only place where "we do not know
/// where this account's deleted mail goes" and "send the delete" are near
/// enough to each other to be tested together.
pub(crate) async fn delete_a_message<'a, S: DeletesAMessage>(
    server: &S,
    folders: impl IntoIterator<Item = (&'a str, FolderType)>,
    deleting_from: &str,
    uid: u32,
    asked: Deleting,
) -> Deleted {
    let trash = match where_a_deleted_message_goes(folders, deleting_from, asked) {
        DeletedGoesTo::TheTrash(path) => Some(path),
        DeletedGoesTo::OffTheServer => None,
        DeletedGoesTo::NoTrashFolderFound => {
            return Deleted::NothingWasSent(NO_TRASH_FOLDER_FOUND);
        }
        DeletedGoesTo::NoFoldersKnownYet => {
            return Deleted::NothingWasSent(NO_FOLDERS_KNOWN_YET);
        }
    };

    match server.delete_it(deleting_from, uid, trash).await {
        Ok(deletion) => Deleted::TheServerDidThis(deletion),
        Err(refused) => Deleted::TheServerWouldNot(refused),
    }
}

/// The account's own mail server, signed in to only when there is something to
/// send.
///
/// The sign-in is inside the step rather than around it, so an account whose
/// trash this program does not recognise is never dialled: nothing decides to
/// connect until something has decided there is a command to carry. The
/// account's own id goes with the sign-in, so this runs under the same
/// permission the rest of that account's writing does.
pub(crate) struct TheAccountsServer<'a> {
    pub account: &'a crate::data::account::Account,
}

impl DeletesAMessage for TheAccountsServer<'_> {
    async fn delete_it(
        &self,
        folder: &str,
        uid: u32,
        trash: Option<&str>,
    ) -> std::result::Result<Deletion, String> {
        let session = crate::application::mail_session::a_session_at(self.account)
            .await
            .map_err(|why| why.to_string())?;
        let outcome = session
            .delete_message(folder, uid, trash)
            .await
            .map_err(|e| e.to_string());
        let _ = session.disconnect_imap().await;
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A mail server that carries out whatever it is told and writes it down.
    ///
    /// The writing down is the measurement. "Nothing was sent" is the whole of
    /// what these tests are about, and a server that only answered could not
    /// tell that apart from one that was never asked.
    #[derive(Default)]
    struct WhatTheServerWasTold {
        told: std::sync::Mutex<Vec<(String, u32, Option<String>)>>,
    }

    impl WhatTheServerWasTold {
        fn heard(&self) -> Vec<(String, u32, Option<String>)> {
            self.told.lock().expect("what the server was told").clone()
        }
    }

    impl DeletesAMessage for WhatTheServerWasTold {
        async fn delete_it(
            &self,
            folder: &str,
            uid: u32,
            trash: Option<&str>,
        ) -> std::result::Result<Deletion, String> {
            self.told.lock().expect("what the server was told").push((
                folder.to_string(),
                uid,
                trash.map(str::to_string),
            ));
            Ok(Deletion::MovedToTrash)
        }
    }

    #[tokio::test]
    async fn test_a_delete_on_an_account_with_no_recognised_trash_asks_no_server_anything() {
        // The message loss this whole module is about. A server naming its
        // trash in another language is enough to get here, and taking the
        // message off the server would be the only copy of it gone.
        let server = WhatTheServerWasTold::default();

        let outcome = delete_a_message(
            &server,
            [
                ("INBOX", FolderType::Inbox),
                ("Archive", FolderType::Archive),
            ],
            "INBOX",
            7,
            Deleting::ToTrash,
        )
        .await;

        assert_eq!(outcome, Deleted::NothingWasSent(NO_TRASH_FOLDER_FOUND));
        assert!(
            server.heard().is_empty(),
            "a delete reached the server for an account with no trash we know: {:?}",
            server.heard()
        );
    }

    #[tokio::test]
    async fn test_a_delete_on_an_account_whose_folders_are_not_known_yet_asks_no_server_anything() {
        // Not knowing yet is not the same as having no trash, and the sentence
        // for it is a different one, because what to do next is different.
        let server = WhatTheServerWasTold::default();

        let outcome = delete_a_message(&server, [], "INBOX", 7, Deleting::ToTrash).await;

        assert_eq!(outcome, Deleted::NothingWasSent(NO_FOLDERS_KNOWN_YET));
        assert!(
            server.heard().is_empty(),
            "a delete reached the server for an account with no folders read yet: {:?}",
            server.heard()
        );
    }

    #[tokio::test]
    async fn test_a_delete_with_a_trash_folder_is_sent_naming_that_folder() {
        // The ordinary case, and the proof that the two above are not passing
        // because this function never sends anything at all.
        let server = WhatTheServerWasTold::default();

        let outcome = delete_a_message(
            &server,
            [("INBOX", FolderType::Inbox), ("Trash", FolderType::Trash)],
            "INBOX",
            7,
            Deleting::ToTrash,
        )
        .await;

        assert_eq!(outcome, Deleted::TheServerDidThis(Deletion::MovedToTrash));
        assert_eq!(
            server.heard(),
            vec![("INBOX".to_string(), 7, Some("Trash".to_string()))]
        );
    }

    #[tokio::test]
    async fn test_deleting_outright_is_sent_with_nowhere_to_move_it_to() {
        // Delete Permanently has to keep working on an account that has never
        // checked for mail, so asking for it outright is answered before the
        // folder list is looked at.
        let server = WhatTheServerWasTold::default();

        let outcome = delete_a_message(&server, [], "INBOX", 7, Deleting::Outright).await;

        assert_eq!(outcome, Deleted::TheServerDidThis(Deletion::MovedToTrash));
        assert_eq!(server.heard(), vec![("INBOX".to_string(), 7, None)]);
    }

    #[tokio::test]
    async fn test_a_server_that_refuses_a_delete_is_not_reported_as_having_done_one() {
        // A refusal and a deletion are opposite facts about where somebody's
        // mail now is.
        struct AServerThatWont;
        impl DeletesAMessage for AServerThatWont {
            async fn delete_it(
                &self,
                _folder: &str,
                _uid: u32,
                _trash: Option<&str>,
            ) -> std::result::Result<Deletion, String> {
                Err("the mailbox is read only".to_string())
            }
        }

        let outcome = delete_a_message(
            &AServerThatWont,
            [("INBOX", FolderType::Inbox), ("Trash", FolderType::Trash)],
            "INBOX",
            7,
            Deleting::ToTrash,
        )
        .await;

        assert_eq!(
            outcome,
            Deleted::TheServerWouldNot("the mailbox is read only".to_string())
        );
    }
}

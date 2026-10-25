//! Every question Wixen Mail asks before it does something, by name.
//!
//! One variant for each question with two answers, so the words on its two
//! buttons, the answer Enter gives and, from 13.4-12.4, whether somebody may
//! stop it being asked are each decided once, for that question, rather than
//! at every place that asks it. The words themselves live in
//! `presentation::asking`, beside the box that shows them.
//!
//! A command that asks two different questions depending on what it will do is
//! two variants: Delete and Shift+Delete on a conversation, and emptying a
//! folder into a Trash or for good. One of each pair cannot be undone, and
//! somebody who stops being asked the other must still be asked that one.

/// One question asked before acting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Question {
    /// Delete a contact, event, task, note or reminder, from a module's list,
    /// from Redo of a delete, or from the Calendar window.
    DeleteAnItem,
    /// Undo of a Copy, which takes the copy away.
    TakeAwayACopy,
    /// Delete a list, calendar, notebook or contact group, with what it holds.
    DeleteAContainer,
    /// Delete on a conversation, which moves its messages to the Trash.
    DeleteAConversation,
    /// Shift+Delete on a conversation, which deletes without the Trash.
    DeleteAConversationPermanently,
    /// Delete a saved search; the mail it listed stays.
    DeleteASavedSearch,
    /// Move a folder and everything in it.
    MoveAFolder,
    /// Delete a folder on the server with its mail.
    DeleteAFolder,
    /// Empty a folder whose messages move to a Trash.
    EmptyAFolder,
    /// Empty a folder whose messages are taken off for good.
    EmptyAFolderForGood,
    /// Run a rule over a folder, a rule that does not delete.
    RunARule,
    /// Run a rule over a folder, a rule that deletes.
    RunARuleThatDeletes,
    /// Remove the folders a server no longer lists, with their cached mail.
    RemoveFoldersTheServerNoLongerLists,
    /// Import a key from an attachment.
    ImportAKey,
    /// Set the view of one folder on others.
    ApplyAViewToOtherFolders,
    /// Open the releases page when an update could not be fetched.
    OpenTheReleasesPage,
    /// Close Wixen Mail and start a downloaded installer.
    InstallAnUpdate,
    /// Move the mail already here that a new block catches to Junk.
    MoveTheMailABlockCatches,
    /// Send a message the spelling check has doubts about.
    SendDespiteSpelling,
    /// Send a small picture as decorative, with no description.
    MarkAPictureDecorative,
    /// Remove a PGP key.
    RemoveAKey,
    /// Delete an imported sound scheme.
    DeleteASoundScheme,
}

impl Question {
    /// Every question, once each.
    pub const ALL: [Question; 22] = [
        Question::DeleteAnItem,
        Question::TakeAwayACopy,
        Question::DeleteAContainer,
        Question::DeleteAConversation,
        Question::DeleteAConversationPermanently,
        Question::DeleteASavedSearch,
        Question::MoveAFolder,
        Question::DeleteAFolder,
        Question::EmptyAFolder,
        Question::EmptyAFolderForGood,
        Question::RunARule,
        Question::RunARuleThatDeletes,
        Question::RemoveFoldersTheServerNoLongerLists,
        Question::ImportAKey,
        Question::ApplyAViewToOtherFolders,
        Question::OpenTheReleasesPage,
        Question::InstallAnUpdate,
        Question::MoveTheMailABlockCatches,
        Question::SendDespiteSpelling,
        Question::MarkAPictureDecorative,
        Question::RemoveAKey,
        Question::DeleteASoundScheme,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn test_every_question_is_listed_once() {
        // A match with no wildcard arm names every variant, so adding one
        // without listing it here fails to compile rather than going unasked.
        let listed = |question: Question| match question {
            Question::DeleteAnItem
            | Question::TakeAwayACopy
            | Question::DeleteAContainer
            | Question::DeleteAConversation
            | Question::DeleteAConversationPermanently
            | Question::DeleteASavedSearch
            | Question::MoveAFolder
            | Question::DeleteAFolder
            | Question::EmptyAFolder
            | Question::EmptyAFolderForGood
            | Question::RunARule
            | Question::RunARuleThatDeletes
            | Question::RemoveFoldersTheServerNoLongerLists
            | Question::ImportAKey
            | Question::ApplyAViewToOtherFolders
            | Question::OpenTheReleasesPage
            | Question::InstallAnUpdate
            | Question::MoveTheMailABlockCatches
            | Question::SendDespiteSpelling
            | Question::MarkAPictureDecorative
            | Question::RemoveAKey
            | Question::DeleteASoundScheme => question,
        };
        let once: BTreeSet<Question> = Question::ALL.iter().copied().map(listed).collect();
        assert_eq!(
            once.len(),
            22,
            "a question is listed twice in ALL, so another is missing: {:?}",
            Question::ALL
        );
    }
}

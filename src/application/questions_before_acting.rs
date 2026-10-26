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

use std::collections::BTreeSet;

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

    /// The name the question is kept under in the settings file.
    ///
    /// Written out rather than taken from the variant's spelling, so renaming
    /// a variant cannot turn a stop somebody made into a name nothing reads.
    pub fn stored_name(self) -> &'static str {
        match self {
            Question::DeleteAnItem => "delete_an_item",
            Question::TakeAwayACopy => "take_away_a_copy",
            Question::DeleteAContainer => "delete_a_container",
            Question::DeleteAConversation => "delete_a_conversation",
            Question::DeleteAConversationPermanently => "delete_a_conversation_permanently",
            Question::DeleteASavedSearch => "delete_a_saved_search",
            Question::MoveAFolder => "move_a_folder",
            Question::DeleteAFolder => "delete_a_folder",
            Question::EmptyAFolder => "empty_a_folder",
            Question::EmptyAFolderForGood => "empty_a_folder_for_good",
            Question::RunARule => "run_a_rule",
            Question::RunARuleThatDeletes => "run_a_rule_that_deletes",
            Question::RemoveFoldersTheServerNoLongerLists => {
                "remove_folders_the_server_no_longer_lists"
            }
            Question::ImportAKey => "import_a_key",
            Question::ApplyAViewToOtherFolders => "apply_a_view_to_other_folders",
            Question::OpenTheReleasesPage => "open_the_releases_page",
            Question::InstallAnUpdate => "install_an_update",
            Question::MoveTheMailABlockCatches => "move_the_mail_a_block_catches",
            Question::SendDespiteSpelling => "send_despite_spelling",
            Question::MarkAPictureDecorative => "mark_a_picture_decorative",
            Question::RemoveAKey => "remove_a_key",
            Question::DeleteASoundScheme => "delete_a_sound_scheme",
        }
    }

    /// The question kept under `name`, or `None` for a name this build does
    /// not know, written by a newer one.
    pub fn from_stored_name(name: &str) -> Option<Question> {
        Question::ALL
            .into_iter()
            .find(|question| question.stored_name() == name)
    }
}

/// The questions somebody has stopped being asked, by ticking "Don't ask me
/// again" and choosing the answer the question remembers.
///
/// Kept in the settings file as a list of stored names, so the file reads the
/// same to a person opening it. A name this build does not know is passed over
/// with a log line rather than refused, because a settings file that fails to
/// read takes every other setting with it.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(from = "Vec<String>", into = "Vec<String>")]
pub struct QuestionsNotAsked(BTreeSet<Question>);

impl QuestionsNotAsked {
    /// Whether `question` has been stopped.
    pub fn holds(&self, question: Question) -> bool {
        self.0.contains(&question)
    }

    /// Stop asking `question`.
    pub fn stop(&mut self, question: Question) {
        self.0.insert(question);
    }

    /// Ask `question` again.
    pub fn ask_again(&mut self, question: Question) {
        self.0.remove(&question);
    }

    /// What Settings saves: the set stored now, with only the questions the
    /// person changed on the Questions page changed.
    ///
    /// `as_built` is the set the page was built from and `as_shown` the set
    /// its boxes say at OK. A question stopped from inside Settings after the
    /// page was built is in `stored_now` and in neither of the others, so it
    /// stays stopped; a box the person did change wins over what is stored.
    pub fn with_changes(stored_now: &Self, as_built: &Self, as_shown: &Self) -> Self {
        let mut saved = stored_now.clone();
        as_shown
            .0
            .difference(&as_built.0)
            .for_each(|stopped| saved.stop(*stopped));
        as_built
            .0
            .difference(&as_shown.0)
            .for_each(|asked_again| saved.ask_again(*asked_again));
        saved
    }
}

impl From<Vec<String>> for QuestionsNotAsked {
    fn from(names: Vec<String>) -> Self {
        QuestionsNotAsked(
            names
                .iter()
                .filter_map(|name| {
                    let known = Question::from_stored_name(name);
                    if known.is_none() {
                        tracing::warn!(
                            "A question stopped in the settings file, \"{name}\", is not one this \
                             build asks, so it is passed over"
                        );
                    }
                    known
                })
                .collect(),
        )
    }
}

impl From<QuestionsNotAsked> for Vec<String> {
    fn from(not_asked: QuestionsNotAsked) -> Self {
        not_asked
            .0
            .into_iter()
            .map(|question| question.stored_name().to_string())
            .collect()
    }
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

    fn the_set(questions: &[Question]) -> QuestionsNotAsked {
        let mut set = QuestionsNotAsked::default();
        questions.iter().for_each(|question| set.stop(*question));
        set
    }

    #[test]
    fn test_every_question_has_a_stored_name_of_its_own() {
        let names: BTreeSet<&str> = Question::ALL.iter().map(|q| q.stored_name()).collect();
        assert_eq!(
            names.len(),
            Question::ALL.len(),
            "two questions share a name"
        );
        for question in Question::ALL {
            assert_eq!(
                Question::from_stored_name(question.stored_name()),
                Some(question)
            );
        }
    }

    #[test]
    fn test_the_stopped_questions_are_stored_as_names_and_read_back_the_same() {
        let stopped = the_set(&[Question::DeleteASavedSearch, Question::RunARule]);
        let stored = serde_json::to_string(&stopped).expect("written");
        assert_eq!(stored, r#"["delete_a_saved_search","run_a_rule"]"#);
        let read: QuestionsNotAsked = serde_json::from_str(&stored).expect("read");
        assert_eq!(read, stopped);
    }

    #[test]
    fn test_a_stored_name_this_build_does_not_know_is_passed_over() {
        // A file written by a newer build, with a question this one never
        // asks, still reads, and every other stop in it holds.
        let read: QuestionsNotAsked =
            serde_json::from_str(r#"["a_question_from_a_newer_build","move_a_folder"]"#)
                .expect("a name nobody knows does not fail the read");
        assert_eq!(read, the_set(&[Question::MoveAFolder]));
    }

    #[derive(serde::Deserialize)]
    struct AFileWrittenBeforeTheField {
        #[serde(default)]
        not_asked: QuestionsNotAsked,
    }

    #[test]
    fn test_an_absent_field_reads_as_every_question_asked() {
        let read: AFileWrittenBeforeTheField = serde_json::from_str("{}").expect("read");
        assert_eq!(read.not_asked, QuestionsNotAsked::default());
        assert!(Question::ALL.iter().all(|q| !read.not_asked.holds(*q)));
    }

    #[test]
    fn test_settings_saves_only_what_the_person_changed_on_the_page() {
        // Delete Sound Scheme stopped from inside Settings after the page was
        // built: in the stored set, in neither of the page's, and kept.
        let stored_now = the_set(&[Question::DeleteASoundScheme, Question::MoveAFolder]);
        let as_built = the_set(&[Question::MoveAFolder]);
        let untouched = as_built.clone();
        assert_eq!(
            QuestionsNotAsked::with_changes(&stored_now, &as_built, &untouched),
            stored_now,
            "a page nobody touched changes nothing"
        );

        // The person unticks Run a Rule, which stops it, and ticks Move a
        // Folder, which asks it again.
        let as_shown = the_set(&[Question::RunARule]);
        assert_eq!(
            QuestionsNotAsked::with_changes(&stored_now, &as_built, &as_shown),
            the_set(&[Question::DeleteASoundScheme, Question::RunARule]),
        );
    }
}

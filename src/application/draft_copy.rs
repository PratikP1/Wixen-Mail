//! Replacing the copy of a draft that is kept at the server.
//!
//! A draft is saved on this computer first, so the work itself is never at
//! risk. What this is about is the copy on the server, which is the one that
//! reaches somebody's other devices, and which they believe is the latest.
//!
//! # Why the order is the whole of it
//!
//! It used to remove the old copy and then file the new one. When the filing
//! then failed there was no copy on the server at all, and both failures went
//! to a line in a log inside a background task with nowhere to speak. Somebody
//! looking at their phone found the draft they had been writing all morning
//! gone from it, and nothing had said a word.
//!
//! Swapping the two lines is not the fix. Both copies of a draft carry the same
//! identifier, so a sweep run afterwards takes the copy just filed along with
//! the old one, which is worse and looks correct. So the copies that are there
//! are asked for first, the new one is filed second, and the removal names
//! exactly the ones the first step found. At no point in that sequence is there
//! no copy on the server.

/// Somewhere a draft can be kept that is not this computer.
///
/// Three steps, which is the whole of what replacing a filed draft asks of a
/// mail server. Named for what they do rather than for a protocol, so the
/// decisions around them can be run in a test: the order, what to do when each
/// step is refused, and what to say afterwards are all decisions, and none of
/// them could be reached before without a server.
pub(crate) trait KeepsTheDraft {
    /// The copies of this draft the folder already holds.
    async fn copies_already_there(
        &self,
        folder: &str,
        message_id: &str,
    ) -> std::result::Result<Vec<u32>, String>;
    /// Put this one in the folder. The string in the error is what the person
    /// is shown, so it says what the server said.
    async fn file_this_one(&self, folder: &str, raw: &[u8]) -> std::result::Result<(), String>;
    /// Take these ones out again, and say how many really went.
    async fn remove_these(&self, folder: &str, uids: &[u32]) -> std::result::Result<usize, String>;
}

/// What became of the copy of a draft at the server.
///
/// Four answers rather than success and failure, because they are four
/// different things for the person to hear and only one of them is ordinary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filed {
    /// The new copy is there and the older one is gone.
    Replaced,
    /// The new copy is there and an older one may still be beside it: either
    /// the server would not take the old one away, or it would not say which
    /// copies were there to begin with.
    FiledAndAnOlderCopyMayStillBeThere(String),
    /// The server would not take the new copy. The older one is untouched.
    NotFiledAndTheOlderOneIsStillThere(String),
    /// The server would not take it and there was no earlier copy, so the
    /// draft is on this computer and nowhere else.
    NotFiledAndThereIsNoCopyThere(String),
}

impl Filed {
    /// One sentence saying where the draft is, for the status line.
    ///
    /// Never empty. This is read aloud, and a status line that goes blank is
    /// indistinguishable from one that never spoke.
    pub fn what_happened(&self) -> String {
        match self {
            Self::Replaced => "The draft was saved.".to_string(),
            Self::FiledAndAnOlderCopyMayStillBeThere(reason) => format!(
                "The draft was saved, but an older copy of it may still be on the server, so \
                 your other devices could show two. The server said: {}",
                ended(reason)
            ),
            Self::NotFiledAndTheOlderOneIsStillThere(reason) => format!(
                "The draft is saved on this computer. The server would not take it, so the copy \
                 your other devices have is the older one. The server said: {}",
                ended(reason)
            ),
            Self::NotFiledAndThereIsNoCopyThere(reason) => format!(
                "The draft is saved on this computer and nowhere else. The server would not take \
                 a copy, so your other devices will not have it. The server said: {}",
                ended(reason)
            ),
        }
    }

    /// Whether this is worth putting in front of the person.
    ///
    /// A draft is saved again every minute while somebody writes, so the
    /// ordinary ending has to stay quiet or it is a sentence a minute burying
    /// the three that matter.
    pub fn needs_saying(&self) -> bool {
        !matches!(self, Self::Replaced)
    }
}

/// A reason with a full stop after it.
///
/// Read aloud, a sentence running into the next one without a stop is heard as
/// one long sentence, so the stop is put here rather than hoped for.
fn ended(reason: &str) -> String {
    let reason = reason.trim_end();
    if reason.ends_with(['.', '!', '?']) {
        return reason.to_string();
    }
    format!("{reason}.")
}

/// Put the newer copy of a draft at the server in place of the older one.
///
/// Ask, file, then remove what the asking found. A failure at the first step
/// does not stop the filing: it only means the old copy cannot be named
/// afterwards, which leaves two copies rather than none.
pub(crate) async fn replace_the_filed_copy<K: KeepsTheDraft>(
    server: &K,
    folder: &str,
    message_id: &str,
    raw: &[u8],
) -> Filed {
    let asked = server.copies_already_there(folder, message_id).await;
    let older = asked.clone().unwrap_or_default();

    if let Err(refused) = server.file_this_one(folder, raw).await {
        // Only when the server was asked and said there was nothing there is
        // "on this computer and nowhere else" a fact. A server that answered
        // neither question is one nothing is known about, and the guess that
        // reads best there is the one that does not frighten anybody.
        return if asked.is_ok() && older.is_empty() {
            Filed::NotFiledAndThereIsNoCopyThere(refused)
        } else {
            Filed::NotFiledAndTheOlderOneIsStillThere(refused)
        };
    }
    // A search that failed leaves an older copy unnamed rather than absent, so
    // the newer one is filed and the doubt is said out loud.
    if let Err(refused) = asked {
        return Filed::FiledAndAnOlderCopyMayStillBeThere(refused);
    }
    if older.is_empty() {
        return Filed::Replaced;
    }
    match server.remove_these(folder, &older).await {
        Ok(removed) if removed == older.len() => Filed::Replaced,
        Ok(_) => Filed::FiledAndAnOlderCopyMayStillBeThere(
            "this server cannot remove one message at a time".to_string(),
        ),
        Err(refused) => Filed::FiledAndAnOlderCopyMayStillBeThere(refused),
    }
}

/// The account's Drafts folder at its own mail server.
///
/// One session for all three steps. Filing a draft again happens on every
/// automatic save, once a minute for as long as somebody writes, and a
/// connection of its own per step would be three sign-ins a minute.
///
/// Built on a session from [`crate::application::mail_session::a_session_at`],
/// which passes the account's own id, so this runs under the same permission
/// the rest of the account's writing does: an account that may not change
/// anything at its server gets a session that refuses each of these rather than
/// one that carries them out.
pub(crate) struct DraftAtTheServer<'a> {
    pub session: &'a crate::application::mail_controller::MailController,
}

impl KeepsTheDraft for DraftAtTheServer<'_> {
    async fn copies_already_there(
        &self,
        folder: &str,
        message_id: &str,
    ) -> std::result::Result<Vec<u32>, String> {
        self.session
            .uids_with_message_id(folder, message_id)
            .await
            .map_err(|e| e.to_string())
    }

    async fn file_this_one(&self, folder: &str, raw: &[u8]) -> std::result::Result<(), String> {
        let as_a_read_draft = format!(
            "({} {})",
            crate::service::protocols::imap::flag::DRAFT,
            crate::service::protocols::imap::flag::SEEN
        );
        self.session
            .append_message(folder, Some(&as_a_read_draft), raw)
            .await
            .map_err(|e| e.to_string())
    }

    async fn remove_these(&self, folder: &str, uids: &[u32]) -> std::result::Result<usize, String> {
        self.session
            .remove_these(folder, uids)
            .await
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// What was asked of the server, in order.
    #[derive(Debug, PartialEq, Eq)]
    enum Asked {
        WhichCopies,
        File,
        Remove(Vec<u32>),
    }

    /// A server that answers each step however the test set it up to.
    struct Scripted {
        already_there: std::result::Result<Vec<u32>, String>,
        filing: std::result::Result<(), String>,
        removal: std::result::Result<usize, String>,
        asked: RefCell<Vec<Asked>>,
    }

    impl Scripted {
        fn holding(uids: Vec<u32>) -> Self {
            let count = uids.len();
            Self {
                already_there: Ok(uids),
                filing: Ok(()),
                removal: Ok(count),
                asked: RefCell::new(Vec::new()),
            }
        }
    }

    impl KeepsTheDraft for Scripted {
        async fn copies_already_there(
            &self,
            _folder: &str,
            _message_id: &str,
        ) -> std::result::Result<Vec<u32>, String> {
            self.asked.borrow_mut().push(Asked::WhichCopies);
            self.already_there.clone()
        }

        async fn file_this_one(
            &self,
            _folder: &str,
            _raw: &[u8],
        ) -> std::result::Result<(), String> {
            self.asked.borrow_mut().push(Asked::File);
            self.filing.clone()
        }

        async fn remove_these(
            &self,
            _folder: &str,
            uids: &[u32],
        ) -> std::result::Result<usize, String> {
            self.asked.borrow_mut().push(Asked::Remove(uids.to_vec()));
            self.removal.clone()
        }
    }

    async fn saving(server: &Scripted) -> Filed {
        replace_the_filed_copy(server, "Drafts", "<d@x>", b"raw").await
    }

    #[tokio::test]
    async fn test_the_new_copy_is_filed_before_the_old_one_is_removed() {
        // The whole of the fix. Removing first meant that a filing the server
        // then refused left no copy on the server at all.
        let server = Scripted::holding(vec![4]);

        let filed = saving(&server).await;

        assert_eq!(filed, Filed::Replaced);
        assert_eq!(
            *server.asked.borrow(),
            vec![Asked::WhichCopies, Asked::File, Asked::Remove(vec![4])]
        );
    }

    #[tokio::test]
    async fn test_the_removal_only_names_the_copies_that_were_there_before_the_new_one() {
        // Both copies carry the same identifier, so a sweep by identifier run
        // after the filing takes the copy just filed with it. That is a worse
        // bug than the one being fixed and it looks correct in review.
        let server = Scripted::holding(vec![4, 9]);

        saving(&server).await;

        let asked = server.asked.borrow();
        let Some(Asked::Remove(named)) = asked.last() else {
            panic!("nothing was removed: {asked:?}");
        };
        assert_eq!(named, &vec![4, 9]);
    }

    #[tokio::test]
    async fn test_a_draft_the_server_will_not_take_leaves_the_older_copy_alone() {
        let mut server = Scripted::holding(vec![4]);
        server.filing = Err("the mailbox is over quota".to_string());

        let filed = saving(&server).await;

        assert!(
            matches!(filed, Filed::NotFiledAndTheOlderOneIsStillThere(_)),
            "{filed:?}"
        );
        assert!(
            !server
                .asked
                .borrow()
                .iter()
                .any(|step| matches!(step, Asked::Remove(_))),
            "the older copy was removed after the new one was refused, which \
             leaves nothing on the server: {:?}",
            server.asked.borrow()
        );
        let said = filed.what_happened();
        assert!(said.contains("older"), "{said}");
    }

    #[tokio::test]
    async fn test_the_first_save_the_server_refuses_says_the_draft_is_only_on_this_computer() {
        let mut server = Scripted::holding(vec![]);
        server.filing = Err("the mailbox is over quota".to_string());

        let filed = saving(&server).await;

        assert!(
            matches!(filed, Filed::NotFiledAndThereIsNoCopyThere(_)),
            "{filed:?}"
        );
        let said = filed.what_happened();
        assert!(said.contains("this computer"), "{said}");
        assert!(
            !said.to_lowercase().contains("lost"),
            "the draft is saved here, so nothing was lost and saying so would \
             frighten somebody for no reason: {said}"
        );
    }

    #[tokio::test]
    async fn test_a_server_that_answered_nothing_at_all_is_not_read_as_one_holding_nothing() {
        // Both steps failed, which is what an unreachable server looks like.
        // Nothing was learnt about what is on it, so saying the draft is on
        // this computer and nowhere else is a guess, and the guess it makes is
        // the frightening one.
        let mut server = Scripted::holding(vec![]);
        server.already_there = Err("the server could not be reached".to_string());
        server.filing = Err("the server could not be reached".to_string());

        let filed = saving(&server).await;

        assert!(
            matches!(filed, Filed::NotFiledAndTheOlderOneIsStillThere(_)),
            "nothing was learnt about the server and the answer says there is \
             no copy on it: {filed:?}"
        );
    }

    #[tokio::test]
    async fn test_an_old_copy_that_could_not_be_removed_is_said_rather_than_only_logged() {
        let mut server = Scripted::holding(vec![4]);
        server.removal = Err("the server would not do it".to_string());

        let filed = saving(&server).await;

        assert!(
            matches!(filed, Filed::FiledAndAnOlderCopyMayStillBeThere(_)),
            "{filed:?}"
        );
        assert!(filed.needs_saying());
        let said = filed.what_happened();
        assert!(said.contains("two"), "{said}");
    }

    #[tokio::test]
    async fn test_a_copy_the_server_could_only_mark_is_said_too() {
        // A server without UIDPLUS flags the old copy and leaves it, and says
        // so by counting none removed. Two drafts are on the server and only
        // one of them is marked, so nothing on any device says which is newer.
        let mut server = Scripted::holding(vec![4]);
        server.removal = Ok(0);

        let filed = saving(&server).await;

        assert!(
            matches!(filed, Filed::FiledAndAnOlderCopyMayStillBeThere(_)),
            "{filed:?}"
        );
        assert!(filed.needs_saying());
    }

    #[tokio::test]
    async fn test_a_search_that_fails_still_files_the_new_copy() {
        // Not knowing what is there is no reason to leave the newer work off
        // the server. The cost is a second copy, and that is what is said.
        let mut server = Scripted::holding(vec![]);
        server.already_there = Err("the search was refused".to_string());

        let filed = saving(&server).await;

        assert!(
            server
                .asked
                .borrow()
                .iter()
                .any(|step| matches!(step, Asked::File)),
            "the newer draft was never offered: {:?}",
            server.asked.borrow()
        );
        assert!(
            matches!(filed, Filed::FiledAndAnOlderCopyMayStillBeThere(_)),
            "{filed:?}"
        );
        assert!(filed.needs_saying());
    }

    #[tokio::test]
    async fn test_an_ordinary_save_says_nothing() {
        // Automatic saving runs once a minute for as long as somebody writes.
        assert!(!Filed::Replaced.needs_saying());
    }

    #[test]
    fn test_a_reason_that_already_ends_in_a_full_stop_does_not_get_a_second_one() {
        // Every reason in these sentences is a mail server's own words, and
        // plenty of servers punctuate them. Read aloud, a doubled stop is a
        // stumble in the middle of the one sentence that says where somebody's
        // draft is, and the trailing space some servers leave is the same
        // thing again with a gap in it.
        //
        // Nothing here can be reached by changing the code in small ways and
        // seeing what the suite notices: the early return is a call to
        // `ends_with` on a list of three characters, and the trim next to it is
        // a call with no arguments. Neither is a line a mutation run produces a
        // mutant for, so both were untested and would have stayed that way.
        for said_by_the_server in ["the mailbox is over quota.", "the mailbox is over quota. "] {
            let filed = Filed::NotFiledAndTheOlderOneIsStillThere(said_by_the_server.to_string());

            let said = filed.what_happened();

            assert!(
                said.ends_with("the mailbox is over quota."),
                "the server's own words were punctuated again: {said}"
            );
        }
    }

    #[test]
    fn test_every_outcome_says_something_and_no_two_say_the_same_thing() {
        let all = [
            Filed::Replaced,
            Filed::FiledAndAnOlderCopyMayStillBeThere("over quota".to_string()),
            Filed::NotFiledAndTheOlderOneIsStillThere("over quota".to_string()),
            Filed::NotFiledAndThereIsNoCopyThere("over quota".to_string()),
        ];

        let mut said: Vec<String> = all.iter().map(Filed::what_happened).collect();
        for sentence in &said {
            assert!(!sentence.trim().is_empty());
            assert!(sentence.ends_with('.'), "{sentence}");
        }
        let before = said.len();
        said.sort();
        said.dedup();
        assert_eq!(before, said.len(), "{said:?}");
    }
}

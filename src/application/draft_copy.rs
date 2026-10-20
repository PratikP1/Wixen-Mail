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
    /// The `Message-ID` the message under this number really carries, or
    /// nothing when it carries none. What the search above cannot say: it
    /// matches any identifier that contains the one asked for.
    async fn identifier_at(
        &self,
        folder: &str,
        uid: u32,
    ) -> std::result::Result<Option<String>, String>;
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
        Ok(_) => Filed::FiledAndAnOlderCopyMayStillBeThere(CANNOT_REMOVE_ONE.to_string()),
        Err(refused) => Filed::FiledAndAnOlderCopyMayStillBeThere(refused),
    }
}

/// What a server that counts fewer removed than it was given has told us.
///
/// A server without UIDPLUS marks a message for removal and leaves it, and the
/// only sign is the count.
const CANNOT_REMOVE_ONE: &str = "this server cannot remove one message at a time";

/// What became of a sent draft's copy in the server's Drafts folder.
///
/// Three answers, and only the last is worth a sentence: a copy left behind is
/// one somebody's phone shows as an unsent draft, and sends a second time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TakenAway {
    /// Every copy the server held under the draft's identifier is gone.
    Gone,
    /// The server held no copy: none was filed, or every filing was refused.
    NothingWasFiled,
    /// A copy may still be there, because the server would not say which
    /// copies it held or would not take them away.
    ACopyMayStillBeThere(String),
}

impl TakenAway {
    /// One sentence saying where the sent draft's copy is. Never empty.
    pub fn what_happened(&self) -> String {
        match self {
            Self::Gone => "The sent message's draft was taken out of Drafts.".to_string(),
            Self::NothingWasFiled => {
                "The sent message had no draft in Drafts on the server.".to_string()
            }
            Self::ACopyMayStillBeThere(reason) => format!(
                "The message you sent may still be in Drafts on the server, so your other \
                 devices could show it as a draft nobody has sent. Wixen Mail tries to take it \
                 out again at the next check for mail. The server said: {}",
                ended(reason)
            ),
        }
    }

    /// Whether this is worth putting in front of the person.
    ///
    /// Taking the copy away is what Send is expected to do, so the ordinary
    /// endings stay quiet and only a copy left behind is said.
    pub fn needs_saying(&self) -> bool {
        matches!(self, Self::ACopyMayStillBeThere(_))
    }
}

/// What a kept removal's second try, at the account's next check for mail,
/// leaves to do (ledger 845).
///
/// Both answers forget the removal. A server that refuses for good is said
/// once, at the check that tried again, and not at every check after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AfterTheSecondTry {
    /// The copy went, or none was there: nothing to say.
    Forget,
    /// The copy may still be there: say so once, with the server's words.
    ForgetAndSay(String),
}

/// What the answer of a second try means.
pub fn after_the_second_try(taken: &TakenAway) -> AfterTheSecondTry {
    match taken {
        TakenAway::Gone | TakenAway::NothingWasFiled => AfterTheSecondTry::Forget,
        TakenAway::ACopyMayStillBeThere(reason) => AfterTheSecondTry::ForgetAndSay(reason.clone()),
    }
}

/// The one sentence a check says for the sent messages whose copy is still
/// in Drafts after a second try, or nothing when there are none.
///
/// `subjects` are those messages' subjects and `reason` the server's words
/// for the first of them.
pub fn still_in_drafts_after_a_second_try(subjects: &[String], reason: &str) -> Option<String> {
    let reason = ended(reason);
    match subjects {
        [] => None,
        [subject] => {
            let which = match subject.trim() {
                "" => "A message you sent with no subject".to_string(),
                named => format!("The message \"{named}\" you sent"),
            };
            Some(format!(
                "{which} is still in Drafts on the server, so your other devices could show it \
                 as a draft nobody has sent. Taking it out failed again, so delete it from \
                 Drafts yourself. The server said: {reason}"
            ))
        }
        several => Some(format!(
            "{} messages you sent are still in Drafts on the server, so your other devices \
             could show them as drafts nobody has sent. Taking them out failed again, so \
             delete them from Drafts yourself. The server said: {reason}",
            several.len()
        )),
    }
}

/// Take a sent draft's copies out of the server's Drafts folder.
///
/// Ask which copies the folder holds under the draft's identifier, then
/// remove exactly those, the same rule the replacement above keeps: the
/// removal names what the server said it holds and can reach nothing else.
/// Run under [`one_filing_at_a_time`], so no filing is in flight while it asks.
pub(crate) async fn take_away_the_filed_copy<K: KeepsTheDraft>(
    server: &K,
    folder: &str,
    message_id: &str,
) -> TakenAway {
    let found = match server.copies_already_there(folder, message_id).await {
        Ok(found) => found,
        Err(refused) => return TakenAway::ACopyMayStillBeThere(refused),
    };
    let there = match the_ones_carrying_it_exactly(server, folder, message_id, found).await {
        Ok(there) => there,
        Err(refused) => return TakenAway::ACopyMayStillBeThere(refused),
    };
    if there.is_empty() {
        return TakenAway::NothingWasFiled;
    }
    match server.remove_these(folder, &there).await {
        Ok(removed) if removed == there.len() => TakenAway::Gone,
        Ok(_) => TakenAway::ACopyMayStillBeThere(CANNOT_REMOVE_ONE.to_string()),
        Err(refused) => TakenAway::ACopyMayStillBeThere(refused),
    }
}

/// The numbers the search found whose message carries the draft's identifier
/// exactly (ledger 854).
///
/// The search behind [`KeepsTheDraft::copies_already_there`], `HEADER
/// MESSAGE-ID`, matches any message whose header contains the text asked for
/// (RFC 3501, section 6.4.4), so a message in Drafts whose identifier merely
/// holds the draft's would be removed with it. Each number is read first and
/// kept only when the renumbering decision's own comparison,
/// [`crate::application::renumbered::the_number_confirmed`], says it is the
/// same identifier. A number whose message cannot be read is an answer nobody
/// has, so it stops the removal rather than being guessed either way.
async fn the_ones_carrying_it_exactly<K: KeepsTheDraft>(
    server: &K,
    folder: &str,
    message_id: &str,
    found: Vec<u32>,
) -> std::result::Result<Vec<u32>, String> {
    let mut exactly = Vec::with_capacity(found.len());
    for uid in found {
        let held = server.identifier_at(folder, uid).await?;
        exactly.extend(crate::application::renumbered::the_number_confirmed(
            message_id,
            uid,
            held.as_deref(),
        ));
    }
    Ok(exactly)
}

/// The lock behind [`one_filing_at_a_time`]. One for the whole process.
static THE_TURN: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// The one turn every filing and every removal at a server takes.
///
/// An automatic save files its copy on a thread of its own and can still be
/// talking to the server when Send is pressed. A removal that searched while
/// that filing was in flight would miss the copy, which would then land after
/// it and stay in Drafts as an unsent draft. So each filing and each removal
/// holds this for its whole length, from before its session opens until after
/// it ends, and a removal searches only once every filing that began before it
/// has finished. Nothing files a draft after Send for that draft, because the
/// composer has closed.
pub(crate) fn one_filing_at_a_time() -> &'static tokio::sync::Mutex<()> {
    &THE_TURN
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
            // No date. A draft is being written now, so now is when it arrived,
            // and the one call that has a date to carry is a message copied
            // across from another account.
            .append_message(folder, Some(&as_a_read_draft), None, raw)
            .await
            .map_err(|e| e.to_string())
    }

    async fn remove_these(&self, folder: &str, uids: &[u32]) -> std::result::Result<usize, String> {
        self.session
            .remove_these(folder, uids)
            .await
            .map_err(|e| e.to_string())
    }

    async fn identifier_at(
        &self,
        folder: &str,
        uid: u32,
    ) -> std::result::Result<Option<String>, String> {
        self.session
            .message_id_at(folder, uid)
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
        /// What reading a number answers, where it is not the draft's own
        /// identifier, [`THE_DRAFTS_IDENTIFIER`].
        carrying: std::collections::HashMap<u32, std::result::Result<Option<String>, String>>,
        asked: RefCell<Vec<Asked>>,
        /// The numbers whose message was read, in order. Kept apart from
        /// `asked` so the sequences the cases above compare are untouched.
        read: RefCell<Vec<u32>>,
    }

    /// The identifier every removal and filing here is asked about.
    const THE_DRAFTS_IDENTIFIER: &str = "<d@x>";

    impl Scripted {
        fn holding(uids: Vec<u32>) -> Self {
            let count = uids.len();
            Self {
                already_there: Ok(uids),
                filing: Ok(()),
                removal: Ok(count),
                carrying: std::collections::HashMap::new(),
                asked: RefCell::new(Vec::new()),
                read: RefCell::new(Vec::new()),
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

        async fn identifier_at(
            &self,
            _folder: &str,
            uid: u32,
        ) -> std::result::Result<Option<String>, String> {
            self.read.borrow_mut().push(uid);
            self.carrying
                .get(&uid)
                .cloned()
                .unwrap_or_else(|| Ok(Some(THE_DRAFTS_IDENTIFIER.to_string())))
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
    async fn test_saving_leaves_a_message_whose_identifier_only_contains_the_drafts() {
        // Ledger 902. The save's search is Send's search, so it answers a
        // message whose identifier merely holds the draft's as well, and the
        // automatic save, once a minute, took it out of Drafts.
        let mut server = Scripted::holding(vec![4, 9]);
        server
            .carrying
            .insert(9, Ok(Some(ONE_THAT_ONLY_CONTAINS_IT.to_string())));
        server.removal = Ok(1);

        let filed = saving(&server).await;

        assert_eq!(
            *server.asked.borrow(),
            vec![Asked::WhichCopies, Asked::File, Asked::Remove(vec![4])],
            "the removal named a message that is not an older copy of the draft"
        );
        assert_eq!(*server.read.borrow(), vec![4, 9]);
        assert_eq!(filed, Filed::Replaced);
    }

    #[tokio::test]
    async fn test_a_save_whose_search_found_only_another_message_removes_nothing() {
        let mut server = Scripted::holding(vec![9]);
        server
            .carrying
            .insert(9, Ok(Some(ONE_THAT_ONLY_CONTAINS_IT.to_string())));

        let filed = saving(&server).await;

        assert_eq!(
            *server.asked.borrow(),
            vec![Asked::WhichCopies, Asked::File],
            "something was removed although no message carried the draft's identifier"
        );
        assert_eq!(filed, Filed::Replaced);
    }

    #[tokio::test]
    async fn test_a_first_save_refused_after_finding_only_another_message_says_the_draft_is_only_here()
     {
        // The search found a number, but not a copy of this draft, so there is
        // no older copy for the person's other devices to show.
        let mut server = Scripted::holding(vec![9]);
        server
            .carrying
            .insert(9, Ok(Some(ONE_THAT_ONLY_CONTAINS_IT.to_string())));
        server.filing = Err("the mailbox is over quota".to_string());

        let filed = saving(&server).await;

        assert_eq!(
            filed,
            Filed::NotFiledAndThereIsNoCopyThere("the mailbox is over quota".to_string())
        );
    }

    #[tokio::test]
    async fn test_a_save_whose_older_copy_cannot_be_read_files_the_new_one_and_leaves_it() {
        // Not knowing what a message carries is not knowing it is an older
        // copy, so it stays, the newer copy is still filed, and the doubt is
        // said with the server's words.
        let mut server = Scripted::holding(vec![4]);
        server
            .carrying
            .insert(4, Err("the fetch was refused".to_string()));

        let filed = saving(&server).await;

        assert_eq!(
            *server.asked.borrow(),
            vec![Asked::WhichCopies, Asked::File]
        );
        assert_eq!(
            filed,
            Filed::FiledAndAnOlderCopyMayStillBeThere("the fetch was refused".to_string())
        );
    }

    #[tokio::test]
    async fn test_a_first_save_reads_nothing() {
        // The reading costs one round trip per number the search found, so a
        // first save, which finds none, costs nothing more than it did.
        let server = Scripted::holding(vec![]);

        let filed = saving(&server).await;

        assert_eq!(filed, Filed::Replaced);
        assert_eq!(*server.read.borrow(), Vec::<u32>::new());
        assert_eq!(
            *server.asked.borrow(),
            vec![Asked::WhichCopies, Asked::File]
        );
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

    async fn sending(server: &Scripted) -> TakenAway {
        take_away_the_filed_copy(server, "Drafts", "<d@x>").await
    }

    /// An identifier that holds the draft's, so the server's search answers
    /// its number too, and is another message's.
    const ONE_THAT_ONLY_CONTAINS_IT: &str = "<re.<d@x>>";

    #[tokio::test]
    async fn test_a_message_whose_identifier_only_contains_the_drafts_stays_in_drafts() {
        // Ledger 854. The search matches any Message-ID that contains the
        // text asked for (RFC 3501, section 6.4.4), so a message in Drafts
        // whose identifier holds the draft's was removed with the sent copy.
        let mut server = Scripted::holding(vec![4, 9]);
        server
            .carrying
            .insert(9, Ok(Some(ONE_THAT_ONLY_CONTAINS_IT.to_string())));
        server.removal = Ok(1);

        let taken = sending(&server).await;

        assert_eq!(
            *server.asked.borrow(),
            vec![Asked::WhichCopies, Asked::Remove(vec![4])],
            "the removal named a message that is not the sent draft's copy"
        );
        assert_eq!(taken, TakenAway::Gone);
    }

    #[tokio::test]
    async fn test_a_search_that_found_only_another_message_removes_nothing() {
        let mut server = Scripted::holding(vec![9]);
        server
            .carrying
            .insert(9, Ok(Some(ONE_THAT_ONLY_CONTAINS_IT.to_string())));

        let taken = sending(&server).await;

        assert_eq!(
            *server.asked.borrow(),
            vec![Asked::WhichCopies],
            "something was removed although no message carried the draft's identifier"
        );
        assert_eq!(taken, TakenAway::NothingWasFiled);
    }

    #[tokio::test]
    async fn test_a_message_the_server_will_not_let_be_read_is_not_removed() {
        // Not knowing what a message carries is not knowing it is the copy,
        // so it stays, and the person hears that a copy may still be there.
        let mut server = Scripted::holding(vec![4]);
        server
            .carrying
            .insert(4, Err("the fetch was refused".to_string()));

        let taken = sending(&server).await;

        assert_eq!(*server.asked.borrow(), vec![Asked::WhichCopies]);
        assert_eq!(
            taken,
            TakenAway::ACopyMayStillBeThere("the fetch was refused".to_string())
        );
    }

    #[tokio::test]
    async fn test_sending_takes_away_exactly_the_copies_the_server_named() {
        // Finding #35: Send took the draft out of the list on this computer
        // and left its copy in the server's Drafts folder, where every other
        // device showed it as a draft nobody had sent.
        let server = Scripted::holding(vec![4, 9]);

        let taken = sending(&server).await;

        assert_eq!(taken, TakenAway::Gone);
        assert_eq!(
            *server.asked.borrow(),
            vec![Asked::WhichCopies, Asked::Remove(vec![4, 9])],
            "the removal named something other than the copies the server held"
        );
        assert!(!taken.needs_saying(), "{}", taken.what_happened());
    }

    #[tokio::test]
    async fn test_a_draft_never_filed_is_taken_away_without_asking_to_remove_anything() {
        let server = Scripted::holding(vec![]);

        let taken = sending(&server).await;

        assert_eq!(taken, TakenAway::NothingWasFiled);
        assert_eq!(*server.asked.borrow(), vec![Asked::WhichCopies]);
        assert!(!taken.needs_saying());
    }

    #[tokio::test]
    async fn test_a_copy_the_server_would_not_remove_is_said() {
        // Three ways a copy can be left behind, and each is said with the
        // server's own words, because the person's phone will show it.
        let mut refused_search = Scripted::holding(vec![4]);
        refused_search.already_there = Err("the search was refused".to_string());
        let mut refused_removal = Scripted::holding(vec![4]);
        refused_removal.removal = Err("the server would not do it".to_string());
        let mut only_marked = Scripted::holding(vec![4, 9]);
        only_marked.removal = Ok(1);

        for (server, its_words) in [
            (&refused_search, "the search was refused."),
            (&refused_removal, "the server would not do it."),
            (&only_marked, "one message at a time."),
        ] {
            let taken = sending(server).await;

            assert!(
                matches!(taken, TakenAway::ACopyMayStillBeThere(_)),
                "{taken:?}"
            );
            assert!(taken.needs_saying(), "{taken:?}");
            let said = taken.what_happened();
            assert!(said.ends_with(its_words), "{said}");
            assert!(said.contains("Drafts"), "{said}");
        }
        assert_eq!(
            *refused_search.asked.borrow(),
            vec![Asked::WhichCopies],
            "something was removed although the server would not say what it held"
        );
    }

    #[tokio::test]
    async fn test_a_removal_waits_for_a_filing_that_began_before_it() {
        // An automatic save's filing can still be in flight when Send is
        // pressed. A removal that searched then would miss the copy, which
        // would land afterwards and stay in Drafts.
        let server = Scripted::holding(vec![4]);
        let filing = one_filing_at_a_time().lock().await;
        server.asked.borrow_mut().push(Asked::File);

        let mut removal = std::pin::pin!(async {
            let _turn = one_filing_at_a_time().lock().await;
            sending(&server).await
        });
        let while_filing = std::future::poll_fn(|cx| {
            std::task::Poll::Ready(std::future::Future::poll(removal.as_mut(), cx))
        })
        .await;
        assert!(
            while_filing.is_pending(),
            "the removal ran while a filing held the turn"
        );
        assert_eq!(*server.asked.borrow(), vec![Asked::File]);

        drop(filing);
        let taken = removal.await;

        assert_eq!(taken, TakenAway::Gone);
        assert_eq!(
            *server.asked.borrow(),
            vec![Asked::File, Asked::WhichCopies, Asked::Remove(vec![4])]
        );
    }

    #[test]
    fn test_send_says_a_copy_left_behind_will_be_tried_again_at_the_next_check() {
        // Ledger 845: the removal is kept and tried again, so the sentence
        // Send says tells the person when.
        let said = TakenAway::ACopyMayStillBeThere("the server could not be reached".to_string())
            .what_happened();

        assert!(said.contains("next check for mail"), "{said}");
        assert!(
            said.ends_with("the server could not be reached."),
            "the server's words are no longer the end of the sentence: {said}"
        );
    }

    #[test]
    fn test_a_second_try_that_took_the_copy_away_or_found_none_says_nothing() {
        for taken in [TakenAway::Gone, TakenAway::NothingWasFiled] {
            assert_eq!(
                after_the_second_try(&taken),
                AfterTheSecondTry::Forget,
                "{taken:?}"
            );
        }
    }

    #[test]
    fn test_a_second_try_that_failed_again_is_forgotten_and_said_with_the_servers_words() {
        let taken = TakenAway::ACopyMayStillBeThere("the server would not do it".to_string());

        assert_eq!(
            after_the_second_try(&taken),
            AfterTheSecondTry::ForgetAndSay("the server would not do it".to_string())
        );
    }

    /// What a sentence about a sent message must never say, because it was
    /// sent.
    fn says_it_was_not_sent(said: &str) -> bool {
        let said = said.to_lowercase();
        ["not sent", "not been sent", "wasn't sent", "unsent"]
            .iter()
            .any(|words| said.contains(words))
    }

    #[test]
    fn test_one_message_still_in_drafts_after_a_second_try_is_named_by_its_subject() {
        let said = still_in_drafts_after_a_second_try(
            &["Lunch on Friday".to_string()],
            "the server would not do it",
        )
        .expect("a sentence for a copy still in Drafts");

        assert!(said.contains("\"Lunch on Friday\""), "{said}");
        assert!(said.contains("delete it from Drafts"), "{said}");
        assert!(
            said.ends_with("The server said: the server would not do it."),
            "{said}"
        );
        assert!(!says_it_was_not_sent(&said), "{said}");

        let without_a_subject =
            still_in_drafts_after_a_second_try(&[String::new()], "the server would not do it")
                .expect("a sentence for a copy with no subject");
        assert!(
            without_a_subject.contains("with no subject"),
            "{without_a_subject}"
        );
        assert!(!without_a_subject.contains("\"\""), "{without_a_subject}");
    }

    #[test]
    fn test_several_messages_still_in_drafts_after_a_second_try_are_said_with_their_number() {
        let subjects = ["Lunch on Friday", "Minutes", "Rota"].map(String::from);

        let said = still_in_drafts_after_a_second_try(&subjects, "over quota")
            .expect("a sentence for the copies still in Drafts");

        assert!(said.starts_with("3 messages you sent"), "{said}");
        assert!(said.contains("delete them from Drafts"), "{said}");
        assert!(said.ends_with("The server said: over quota."), "{said}");
        assert!(
            !subjects
                .iter()
                .any(|subject| said.contains(subject.as_str())),
            "one sentence for several names none of them: {said}"
        );
        assert!(!says_it_was_not_sent(&said), "{said}");
    }

    #[test]
    fn test_nothing_is_said_when_no_copy_failed_again() {
        assert_eq!(still_in_drafts_after_a_second_try(&[], "over quota"), None);
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

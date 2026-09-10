//! Where an account's notes go, asked in one place.
//!
//! A note belongs to an account, and where that account's notes are kept is a
//! question two screens and a menu each used to answer for themselves. The
//! note folder menu held a constant saying no sync exists, the settings screen
//! said nothing at all, and `new_item::supports` wrote the answer out a third
//! time as a `false` with a comment beside it. Three copies of one fact is how
//! a menu comes to offer a sync a screen says is impossible.
//!
//! This is that one place. Everything that wants to know asks here.
//!
//! # Why it is not in `new_item`
//!
//! [`crate::application::new_item`] is about where a new thing goes when
//! somebody presses a key. This is about where a thing already made is sent,
//! and how it gets back. They meet at one point, which is that a note made in
//! an account whose notes go somewhere is filed under that account, and
//! `supports` asks here for that answer rather than keeping a second one. Two
//! subjects in one file is how the next person comes to believe there is one
//! question.
//!
//! # Nothing here talks to a server
//!
//! No backend is implemented. This answers which one an account uses, and
//! today the answer is the same for every account: their notes stay on this
//! computer. `05.1-03` puts a CalDAV journal behind
//! [`NotesBackend::CalDavJournal`], and phase 5.2 adds OneNote. Neither of
//! those changes the shape of the question, which is the point of asking it
//! here first.

use crate::data::account::Account;

/// Where an account's notes go.
///
/// A which rather than a yes or no, because "does this account sync notes"
/// cannot say which of two backends is answering once there are two, and a
/// menu that offers "Sync notes now" has to know what it is syncing to.
///
/// [`NotesBackend::Other`] follows
/// [`crate::data::message_cache::AddressBook`], whose own doc comment gives
/// the reason: a word this code does not recognise is still somebody else's
/// answer, and forgetting it rewrites their row on the next save without
/// anybody having asked for that. A notes backend named by a build that came
/// later is still a notes backend, and a build that quietly replaced its name
/// with its own would send that account's notes to the wrong place.
///
/// [`NotesBackend::ThisComputer`] is a real answer and not an absence. The
/// settings screen has a sentence to say and the menu has a decision to make,
/// and both need something to be told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotesBackend {
    /// Nowhere else. The notes are kept on this computer and go no further.
    ThisComputer,
    /// A CalDAV server's journal entries, which is what `05.1-03` builds.
    ///
    /// Named here before it exists, because decision 2 of 2026-09-06 settled
    /// which backends this program will speak and naming one is not shipping
    /// a client for it. Nothing answers this yet.
    CalDavJournal,
    /// A word this build does not recognise.
    Other(String),
}

impl NotesBackend {
    /// Whether these notes go anywhere other than this computer.
    ///
    /// The one derived answer, so a menu, a screen and where a new note is
    /// filed cannot come to disagree about one account.
    ///
    /// A word this build does not recognise answers no, and that is a
    /// decision rather than a default. A build that meets the name of a
    /// backend it has never heard of has no client for it, so offering to
    /// sync would be offering something that cannot happen, which is the rule
    /// [`crate::application::context_menu`] is already written to. The note
    /// itself is still read, still edited and still kept: that is what
    /// [`NotesBackend::Other`] is for. Surviving being read is not the same
    /// as being acted on.
    pub fn goes_somewhere_else(&self) -> bool {
        match self {
            NotesBackend::ThisComputer => false,
            NotesBackend::CalDavJournal => true,
            NotesBackend::Other(_) => false,
        }
    }
}

/// Where this account's notes go.
///
/// `None` is every part of the program that has no account in hand, and it is
/// a real question rather than a missing argument: notes made before anybody
/// has signed in anywhere are kept here, so the answer is the same one an
/// account with no provider gets.
///
/// Every arm answers the same today and each says why in its own words,
/// because the reasons are different and only one of them is going to change.
pub fn for_account(account: Option<&Account>) -> NotesBackend {
    let provider = account.and_then(crate::application::mail_auth::provider_of);
    match provider.as_deref() {
        // Google Keep's API is Workspace only, so a consumer Gmail account
        // cannot use it at all. This arm is not waiting for anybody: it is
        // the answer after all three backends ship.
        Some("gmail") => NotesBackend::ThisComputer,
        // OneNote could carry them and does not yet. A page is an HTML
        // document inside a section inside a notebook rather than a title and
        // a body, so the mapping is a decision somebody has to make. Phase 5.2
        // is where it is made, and this is the arm that changes.
        Some("outlook") => NotesBackend::ThisComputer,
        // Every plain IMAP or POP account, every account whose provider this
        // build does not recognise, and no account at all. A mail server is a
        // mail server, and a note "in" one would live in a database on this
        // computer while claiming to belong somewhere else.
        _ => NotesBackend::ThisComputer,
    }
}

/// Where the default account's notes go.
///
/// The default account is the one a new note is filed under, so it is the one
/// the notes sidebar is showing and the one the settings screen is talking
/// about. Written here rather than at each caller so the menu and the screen
/// cannot come to mean different accounts by "the account whose notes these
/// are".
pub fn for_default_account(default_id: Option<&str>, accounts: &[Account]) -> NotesBackend {
    for_account(default_id.and_then(|id| accounts.iter().find(|account| account.id == id)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(id: &str, email: &str) -> Account {
        let mut account = Account::new("Work".to_string(), email.to_string());
        account.id = id.to_string();
        account
    }

    #[test]
    fn test_a_gmail_accounts_notes_stay_on_this_computer() {
        // Google Keep's API is Workspace only. This is not a gap waiting to be
        // filled: it is still the answer after all three backends ship.
        assert_eq!(
            for_account(Some(&account("a1", "me@gmail.com"))),
            NotesBackend::ThisComputer
        );
    }

    #[test]
    fn test_an_outlook_accounts_notes_stay_on_this_computer() {
        // OneNote is phase 5.2. Until then this account has no notes backend,
        // which is a different sentence from "not yet" and is the true one
        // today.
        assert_eq!(
            for_account(Some(&account("a1", "me@outlook.com"))),
            NotesBackend::ThisComputer
        );
    }

    #[test]
    fn test_an_account_whose_provider_this_build_does_not_recognise_keeps_its_notes_here() {
        // Answered rather than refused and rather than guessed. A provider
        // this build has never heard of has no notes backend this build can
        // talk to, and saying so is an answer.
        assert_eq!(
            for_account(Some(&account("a1", "me@myhost.example"))),
            NotesBackend::ThisComputer
        );
    }

    #[test]
    fn test_an_account_with_no_provider_at_all_keeps_its_notes_here() {
        // Every plain IMAP and POP account, and somebody who has not signed in
        // anywhere yet. Both reach this through `None`.
        assert_eq!(for_account(None), NotesBackend::ThisComputer);
    }

    #[test]
    fn test_a_backend_word_this_build_does_not_recognise_is_not_somewhere_it_can_sync_to() {
        // The decision `Other` forces and the plan did not make. A build that
        // meets the name of a backend it has never heard of has no client for
        // it, so it cannot sync to it, and the honest answer is no. The note
        // still survives being read, which is the whole reason the variant
        // exists.
        assert!(!NotesBackend::Other("something-later".to_string()).goes_somewhere_else());
        assert!(!NotesBackend::ThisComputer.goes_somewhere_else());
        assert!(NotesBackend::CalDavJournal.goes_somewhere_else());
    }

    #[test]
    fn test_the_default_account_is_the_one_the_answer_is_about() {
        // Not the first account that has a backend, and not whichever mailbox
        // is being looked at. A note is filed under the default account, so
        // the sentence and the menu are about that one.
        let accounts = vec![
            account("a1", "me@gmail.com"),
            account("a2", "me@myhost.example"),
        ];

        assert_eq!(
            for_default_account(Some("a2"), &accounts),
            for_account(Some(&accounts[1]))
        );
    }

    #[test]
    fn test_a_default_account_that_is_not_there_is_the_same_as_none() {
        // The default account can name one that has been deleted, and a note
        // can be made before any account exists. Neither is a reason to
        // refuse an answer.
        assert_eq!(
            for_default_account(Some("gone"), &[]),
            NotesBackend::ThisComputer
        );
        assert_eq!(for_default_account(None, &[]), NotesBackend::ThisComputer);
    }
}

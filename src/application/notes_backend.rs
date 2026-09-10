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
//!
//! [`NotesService`] is the same statement in a second form: the operations a
//! backend performs, with nothing behind them. What a backend has to answer,
//! and what it may not decide for itself, is written out in
//! `docs/development/the-notes-seam.md`.

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

/// The default account, when there is one and it is still there.
///
/// The default account is the one a new note is filed under, so it is the one
/// the notes sidebar is showing and the one the settings screen is talking
/// about. Resolved here rather than at each caller so the menu and the screen
/// cannot come to mean different accounts by "the account whose notes these
/// are".
///
/// `None` covers three things that need the same answer and are not worth
/// telling apart: nobody has set a default, the default names an account that
/// has been deleted, and there are no accounts at all.
pub fn default_account<'a>(
    default_id: Option<&str>,
    accounts: &'a [Account],
) -> Option<&'a Account> {
    default_id
        .filter(|id| !id.is_empty())
        .and_then(|id| accounts.iter().find(|account| account.id == id))
}

/// Where the default account's notes go.
pub fn for_default_account(default_id: Option<&str>, accounts: &[Account]) -> NotesBackend {
    for_account(default_account(default_id, accounts))
}

/// What a screen says about where an account's notes go.
///
/// The words live here rather than in the settings screen for the reason
/// `allowed::MESSAGE_TEXT_LABEL` and `calendar_source::NOT_TRIED_FOR_REAL`
/// give: a sentence typed into a screen gets a second hand-written copy beside
/// it for the accessibility name, and the two drift. This is one string and
/// there is nothing to drift from.
///
/// It names the account, because it is about one account and the screen can
/// only reach the default one. A sentence that said "your notes" would read as
/// a statement about all of them, which it is not and will be less so once a
/// backend ships.
///
/// It says what is true rather than what is true yet. "This account has no
/// notes backend" goes on being right after all three backends ship, because a
/// consumer Gmail account still has none: Google Keep's API is Workspace only.
/// "Notes do not sync yet" would have to be rewritten and, worse, would tell
/// somebody to wait for something that is not coming.
pub fn where_they_go(account: Option<&Account>) -> String {
    match account {
        Some(account) => where_they_go_for(&for_account(Some(account)), &account.display_name()),
        // Before any account is set up, and after the default one is deleted.
        // A blank where the sentence should be is worse than the answer, which
        // is that the notes are here.
        None => "Notes are kept on this computer. There is no default account, so \
                 nothing sends them anywhere."
            .to_string(),
    }
}

/// The sentence, given the answer and what to call the account.
///
/// Apart from [`where_they_go`] because no account can answer anything but
/// [`NotesBackend::ThisComputer`] today, so the other two arms are unreachable
/// from an account and a test written through one could only ever drive the
/// arm that already ships. That is the "test double that cannot fail" shape,
/// and splitting the words from the lookup is what avoids it here.
pub fn where_they_go_for(backend: &NotesBackend, account_named: &str) -> String {
    match backend {
        NotesBackend::ThisComputer => format!(
            "Notes in {account_named} are kept on this computer. This account has no \
             notes backend, so nothing sends them anywhere."
        ),
        NotesBackend::CalDavJournal => format!(
            "Notes in {account_named} are kept on this computer and sent to that \
             account's calendar server."
        ),
        // Named rather than described, because somebody working out why their
        // notes are not moving needs the word to say when they ask. It is
        // shown in quotation marks so it reads as a name this program is
        // repeating rather than as a word it chose.
        NotesBackend::Other(word) => format!(
            "Notes in {account_named} name a notes backend this version does not \
             recognise, \"{word}\", so they are kept on this computer and nothing \
             sends them anywhere."
        ),
    }
}

/// The same, for the default account, which is the only one a screen can reach.
pub fn where_the_default_accounts_notes_go(
    default_id: Option<&str>,
    accounts: &[Account],
) -> String {
    where_they_go(default_account(default_id, accounts))
}

/// What one backend calls a note, and how it says whether that copy has moved.
///
/// Both halves belong to the pairing of a note and a backend rather than to the
/// note, which is the shape
/// [`crate::data::message_cache::ProviderIdentity`] already takes for a
/// contact, and its own comment says what went wrong before it did: a flag kept
/// on the contact meant a push refused at one address book either lost the
/// change at the other or resent it to both for ever.
///
/// `version` is an `Option` because not every backend gives one, and a backend
/// that does give one does not promise it is opaque or that it changes only
/// when the content does. A OneNote page has no ETag at all: what it has is a
/// `lastModifiedDateTime` the service writes. Reading a version marker as an
/// opaque token that means "this is the copy I saw" is therefore a CalDAV
/// assumption, and `docs/development/the-notes-seam.md` says so in writing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ANoteThere {
    /// What this backend calls the note. Opaque to everything here.
    pub named: String,
    /// The version marker this backend last gave, when it gives one.
    pub version: Option<String>,
}

/// What a backend said when it was asked to do something.
///
/// Separate variants rather than one error string, and the reason is not
/// tidiness. A crate's error text is written for whoever is reading a stack
/// trace, and what reaches somebody here is read aloud. The three that a
/// person can do something about have to be told apart from the ones they
/// cannot, because the sentence differs and so does what it asks of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatTheBackendSaid {
    /// It did what was asked, and this is what it calls the note now.
    ///
    /// A write may change the answer. A OneNote page's body cannot be
    /// replaced, only appended to, so a backend may have to remove a page and
    /// make another one to leave it saying what was asked; that changes its
    /// identity, and the caller has to be told rather than left holding the
    /// old one.
    Done(ANoteThere),
    /// This backend does not hold that note.
    ///
    /// Not an error. Somebody deleted it at the other end, and what the caller
    /// does about it is written down rather than guessed at.
    ItIsNotThere,
    /// The backend's copy moved since this computer last looked.
    ///
    /// Nothing was written. Which copy is kept is not the backend's decision,
    /// and this variant exists so that it cannot quietly make one:
    /// [`crate::application::conflict_choice`] is where that is asked, and
    /// this is how a backend hands the question over.
    ItMovedFirst {
        /// What the backend says the marker is now, so the hold that is about
        /// to be written can record it and the next sync does not ask the same
        /// question again.
        version_now: Option<String>,
    },
    /// Nobody is signed in to this account.
    ///
    /// Its own variant rather than one more failure, because it is the one
    /// thing on this list only the person can fix, and a count says "1 problem"
    /// on every sync forever while telling them nothing.
    NotSignedIn,
    /// Refused because of what this program is allowed to change.
    ///
    /// Apart from [`Self::NotSignedIn`] because what fixes it is different: one
    /// is a sign-in and one is a setting on the settings screen, and the
    /// sentence has to name the right one.
    NotAllowedToChangeAnything,
    /// It could not be reached at all, and this is what was said.
    ///
    /// The string is for the log. Nothing reads it out: text a crate wrote for
    /// a developer is not text to speak to somebody whose notes did not sync.
    CouldNotBeReached(String),
}

/// What a notes sync asks of a service.
///
/// Named for what it is rather than for any provider's HTTP, which is the
/// argument [`crate::application::tasks_sync`]'s own `TaskService` makes about
/// itself: saying it in the type is what lets the deciding be tested, because
/// every decision a sync makes can then be driven without an account.
///
/// **Nothing implements this.** No backend exists, nothing here opens a
/// connection, and there is no fake either: a second implementation written
/// the same day as the interface, by the same person, against the same
/// assumptions, agrees with the interface and proves nothing. `05.1-04` writes
/// one shaped from what OneNote really does, which is a different thing, and
/// `05.1-03` writes the first real one.
///
/// The methods are written as functions returning a future rather than as
/// `async fn`, and that is not a style choice. Two lints pull opposite ways
/// here and the bind is worth recording rather than rediscovering.
/// `pub(crate)`, which is where `TaskService` sits, is `dead_code` for a trait
/// with no implementor, and this has none on purpose. `pub` is reachable from
/// the crate root so `dead_code` does not fire, but `async fn` in a public
/// trait warns, because a caller cannot then name a `Send` bound on the
/// future. Both are build failures under `-D warnings` and silencing either
/// with an `allow` is what `CLAUDE.md` forbids, so what is left is the
/// desugared form the lint itself suggests. It says the `Send` bound out loud,
/// which a sync spawned on the runtime needs anyway, so the shape the lints
/// forced is also the more honest one.
///
/// A container is an opaque string the backend hands out and this code never
/// takes apart. A CalDAV journal lives in one collection at one address; a
/// OneNote page lives four levels down, in a section in a section group in a
/// notebook. "Which container" is therefore not one identifier everywhere, and
/// anything here that split a container on a separator would be reading a
/// CalDAV address.
pub trait NotesService {
    /// Every note this backend holds in one place, and what it calls each.
    fn notes_it_holds(
        &self,
        container: &str,
    ) -> impl std::future::Future<Output = crate::common::Result<Vec<ANoteThere>>> + Send;

    /// Leave the backend's copy of one note saying this.
    ///
    /// Not "replace the body", which is an operation some backends do not have.
    /// What a backend is asked for is the end state, and how it reaches it is
    /// its own business as long as it reports what it ended up calling the
    /// note.
    ///
    /// `known_as` is `None` for a note the backend has never held.
    fn leave_a_note_saying(
        &self,
        container: &str,
        known_as: Option<&ANoteThere>,
        title: &str,
        body: &str,
    ) -> impl std::future::Future<Output = crate::common::Result<WhatTheBackendSaid>> + Send;

    /// Take one note away.
    ///
    /// `Done` carries what the backend called the note it removed, because the
    /// record of the deletion outlives the row and has to name it. See
    /// [`crate::application::deletions`] for why that record exists at all.
    fn take_a_note_away(
        &self,
        container: &str,
        known_as: &ANoteThere,
    ) -> impl std::future::Future<Output = crate::common::Result<WhatTheBackendSaid>> + Send;
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
    fn test_the_sentence_names_the_account_it_is_about() {
        // The settings screen can only reach the default account, so a
        // sentence saying "your notes" would read as a statement about all of
        // them. It is not one now and will be less so once a backend ships.
        let said = where_they_go(Some(&account("a1", "me@gmail.com")));

        assert!(said.contains("Work"), "{said}");
    }

    #[test]
    fn test_an_account_with_no_notes_backend_is_told_so_rather_than_told_not_yet() {
        // Not a placeholder a backend removes. A consumer Gmail account has no
        // notes backend after all three ship, because Google Keep's API is
        // Workspace only, so this sentence has real work left in it. "Not yet"
        // would have to be rewritten and would tell somebody to wait for
        // something that is not coming.
        let said = where_they_go(Some(&account("a1", "me@gmail.com")));

        assert!(said.contains("no notes backend"), "{said}");
        assert!(said.contains("on this computer"), "{said}");
        assert!(!said.contains("yet"), "{said}");
    }

    #[test]
    fn test_somebody_with_no_default_account_is_still_told_where_their_notes_are() {
        // Before any account is set up, and after the default one is deleted.
        // Both reach a screen, and a blank where the sentence should be is
        // worse than the answer, which is that the notes are here.
        let said = where_the_default_accounts_notes_go(None, &[]);

        assert!(said.contains("on this computer"), "{said}");
        assert!(!said.is_empty());
    }

    #[test]
    fn test_the_sentence_says_where_the_notes_really_go() {
        // The half that stops this being a fixed sentence dressed as an
        // answer. An account whose notes reach a server has to be told that,
        // and a backend word this build does not recognise has to be told
        // plainly rather than described as a backend that works.
        let reaches_a_server = where_they_go_for(&NotesBackend::CalDavJournal, "Work");
        assert!(
            reaches_a_server.contains("calendar server"),
            "{reaches_a_server}"
        );
        assert!(
            !reaches_a_server.contains("no notes backend"),
            "{reaches_a_server}"
        );

        let unknown =
            where_they_go_for(&NotesBackend::Other("something-later".to_string()), "Work");
        assert!(unknown.contains("on this computer"), "{unknown}");
        assert!(unknown.contains("does not recognise"), "{unknown}");
    }

    #[test]
    fn test_the_sentence_uses_no_phrase_a_document_guard_forbids() {
        // `A_CONTROL_NO_SCREEN_WRITES` in tests/house_style.rs forbids telling
        // somebody to set something per account, because no screen writes a
        // per-account answer. A notes backend chosen by account is exactly the
        // subject that invites the phrase.
        //
        // That guard reads every source file as well as every document, so it
        // does cover this sentence. This asks the same question of the string
        // the sentence really produces rather than of the text that produces
        // it, which is the difference between a phrase written here and a
        // phrase assembled from pieces at run time.
        //
        // Both phrases are split the way that guard's own constant splits
        // them, and for the same reason: written whole, they are the very
        // thing the guard is looking for, so this test would trip it and
        // report itself. That was found by writing them whole and watching the
        // commit be refused.
        let forbidden = [
            concat!("set it ", "per account"),
            concat!("for each account ", "separately"),
        ];
        for backend in [
            NotesBackend::ThisComputer,
            NotesBackend::CalDavJournal,
            NotesBackend::Other("something-later".to_string()),
        ] {
            let said = where_they_go_for(&backend, "Work");
            for phrase in forbidden {
                assert!(!said.contains(phrase), "{said}");
            }
        }
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

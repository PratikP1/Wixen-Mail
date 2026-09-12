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
//! # This is where a backend is chosen, and the only place
//!
//! One backend exists. An account with a calendar on a calendar server sends
//! its notes to that server's journal entries, under the same sign-in, and
//! every other account keeps its notes here. Phase 5.2 adds OneNote, and it is
//! a second arm in two matches rather than a change anywhere else: the sync
//! itself, [`crate::application::notes_sync`], names no backend at all and a
//! grep of that file proves it.
//!
//! [`NotesService`] says what a backend has to do, and
//! `docs/development/the-notes-seam.md` says what it must not decide for
//! itself. [`crate::service::caldav_journal`] is the first implementation.
//!
//! # Nothing here has ever met a real server
//!
//! No account, no calendar server and no journal collection has been used with
//! this program. Every part of this has been driven against a stand-in written
//! in this repository, which proves the program agrees with itself and says
//! nothing about anybody's server. The settings screen says so where somebody
//! reads it before they rely on it, and `.planning/WINDOWS.md` names each
//! unknown separately.

use crate::data::account::Account;
use crate::data::message_cache::MessageCache;

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
    /// Pages in the sections of a Microsoft account's notebooks.
    ///
    /// One section is one container and so one note folder, which is the
    /// arrangement decided 2026-09-11 and written into
    /// `docs/development/the-notes-seam.md`.
    /// [`crate::service::onenote_notes`] is the implementation.
    OneNote,
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
            NotesBackend::OneNote => true,
            NotesBackend::Other(_) => false,
        }
    }
}

/// Whether this account has a calendar held on a calendar server.
///
/// The one lookup that turns a stored row into the fact
/// [`for_account`] is given, so that "does this account have a calendar
/// server" is answered in the same place as "where do its notes go" rather
/// than at each screen that asks.
///
/// A calendar somebody added by its address carries the word
/// [`crate::application::calendar_source::ON_A_SERVER`] and the address it was
/// found at. Both are asked for: the word alone would count a row half written
/// by a discovery that failed, and an address alone would count a published
/// feed, which is read and never written to.
pub fn has_a_calendar_server(cache: &MessageCache, account_id: &str) -> bool {
    cache
        .get_calendars_for_account(account_id)
        .unwrap_or_default()
        .iter()
        .any(|calendar| {
            calendar.source_provider.as_deref()
                == Some(crate::application::calendar_source::ON_A_SERVER)
                && calendar.caldav_url.is_some()
        })
}

/// Where this account's notes go.
///
/// `None` is every part of the program that has no account in hand, and it is
/// a real question rather than a missing argument: notes made before anybody
/// has signed in anywhere are kept here, so the answer is the same one an
/// account with no provider gets.
///
/// `a_calendar_server` is [`has_a_calendar_server`], asked by the caller
/// because this takes an [`Account`] and the answer is in the store. It is a
/// second argument rather than a store handle so that the words a screen says
/// can still be driven without one.
///
/// # Why a calendar server decides it, and why it is asked first
///
/// A CalDAV server holds journal entries in the same place it holds calendars,
/// under the same sign-in. So an account that already has a calendar on a
/// server already has everywhere a note needs to go and everything needed to
/// get there, and nobody has to type a second address or a second password.
/// That is what `05.1-03`'s threat register anticipates when it asks whether
/// this backend reuses the sign-in of a calendar somebody already added: it
/// does, and one server signed in to once is one credential owner rather than
/// two.
///
/// It is asked before the provider, because it is a fact about this account
/// and the provider arms are facts about a mail service. Somebody with a
/// Fastmail calendar added to their Gmail account has a place for their
/// journal entries whatever Google Keep does or does not offer.
pub fn for_account(account: Option<&Account>, a_calendar_server: bool) -> NotesBackend {
    if a_calendar_server {
        return NotesBackend::CalDavJournal;
    }
    let provider = account.and_then(crate::application::mail_auth::provider_of);
    match provider.as_deref() {
        // Google Keep's API is Workspace only, so a consumer Gmail account
        // cannot use it at all. This arm is not waiting for anybody: it is
        // the answer after all three backends ship.
        Some("gmail") => NotesBackend::ThisComputer,
        // OneNote carries them, as of 5.2. A page is an HTML document inside a
        // section inside a notebook rather than a title and a body, and the
        // mapping that used to be called a decision nobody had made is
        // `service::onenote_page`, with a fidelity table behind it saying what
        // a note loses on the way. One section is one note folder.
        Some("outlook") => NotesBackend::OneNote,
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
pub fn for_default_account(
    default_id: Option<&str>,
    accounts: &[Account],
    a_calendar_server: bool,
) -> NotesBackend {
    for_account(default_account(default_id, accounts), a_calendar_server)
}

/// Where the default account's notes go, asked of the store.
///
/// The form every part of the running program uses, because every part of it
/// holds a store. Kept beside [`for_default_account`] rather than replacing it
/// so that the words a screen says can still be driven with no store at all,
/// which is what lets the three sentences below be tested without one.
pub fn for_the_default_account_in(
    cache: &MessageCache,
    default_id: Option<&str>,
    accounts: &[Account],
) -> NotesBackend {
    let account = default_account(default_id, accounts);
    let server = account.is_some_and(|account| has_a_calendar_server(cache, &account.id));
    for_account(account, server)
}

/// What one run of an account's notes sync did, or why it did nothing.
///
/// Three answers rather than a result and an error, because two of the three
/// are not failures and reporting them as ones is how "1 problem" appears on a
/// status line every time somebody presses a key that was never going to do
/// anything.
#[derive(Debug)]
pub enum WhatTheNotesSyncDid {
    /// This account's notes stay on this computer, so nothing was asked of
    /// anybody.
    TheyStayHere,
    /// This account's notes go to a server nobody has signed in to.
    ///
    /// The one thing on this list only the person can fix, so it is told apart
    /// from the others and reaches them in words.
    NobodyIsSignedIn,
    /// It ran, and this is what it did.
    ItRan(crate::application::notes_sync::NoteSyncResult),
}

/// Every calendar server this account's notes go beside.
///
/// **All of them, which is what closed ledger 245.** This used to be
/// `the_calendar_server_of`, which took the first the store answered with and
/// nothing asked the person which. That was a limit nobody chose. One backend
/// container is one note folder, decided 2026-09-11, so two calendar servers
/// are two folders and there is no first to pick.
///
/// In the order the store answers, which is the order the sidebar shows them
/// in, so the folders come out in an order somebody recognises.
fn the_calendar_servers_of(cache: &MessageCache, account_id: &str) -> Vec<ACalendarServersJournal> {
    cache
        .get_calendars_for_account(account_id)
        .unwrap_or_default()
        .into_iter()
        .filter(|calendar| {
            calendar.source_provider.as_deref()
                == Some(crate::application::calendar_source::ON_A_SERVER)
        })
        // The address and the word are both asked for, the same pair
        // [`has_a_calendar_server`] asks, so the two answers cannot disagree
        // about one account. Resolved into the struct here rather than left as
        // an `Option` for every caller to unwrap or branch on twice.
        .filter_map(|calendar| {
            Some(ACalendarServersJournal {
                container: calendar.caldav_url.clone()?,
                calendar_id: calendar.id,
                called: calendar.name,
            })
        })
        .collect()
}

/// One calendar server's journal entries, as the three facts a sync needs.
///
/// The address the notes are at, the calendar row the sign-in is stored under,
/// and what to call the folder. Three fields rather than the whole calendar
/// row, so that nothing downstream can reach for a calendar's colour or its
/// read-only flag and start deciding something about notes from it.
struct ACalendarServersJournal {
    /// The opaque container, which is a CalDAV collection address here and is
    /// never read as one outside `service::caldav_journal`.
    container: String,
    /// Which calendar row the sign-in for this server is stored under.
    calendar_id: String,
    /// What the calendar is called, which is what its note folder is called.
    called: String,
}

/// The note folders this account's backends are, made if they are not there yet.
///
/// One folder per container, named by what the backend calls the place. For a
/// calendar server that is the calendar's own name; a backend with levels above
/// a note flattens them into the name itself, which is requirement 2 of the
/// seam's container section and the backend's business rather than this
/// function's.
///
/// A folder somebody made here is not in this list and is never touched by it.
/// It has no container, so there is nowhere to sync it to, and
/// [`crate::data::message_cache::MessageCache::a_note_folder_for`] will not
/// adopt one however its name reads.
///
/// The places a sync runs for, which is why it is one function rather than a
/// lookup at each site: the list of folders and the list of syncs must be the
/// same list, and two readings of "which containers does this account have"
/// disagree the day either changes.
pub fn note_folders_for_the_backends_of(
    cache: &MessageCache,
    account_id: &str,
) -> crate::common::Result<Vec<crate::data::message_cache::NoteFolderEntry>> {
    // **A OneNote account's folders are not in this list, and that is a limit
    // rather than an oversight.** A calendar server's containers are rows this
    // computer already holds, so they can be answered without asking anybody. A
    // notebook's sections are Microsoft's answer and asking for them is a
    // request, which this cannot make: it is called while a screen is being
    // filled and it is not async. So a Microsoft account's note folders appear
    // at its first sync rather than before it, made by `sync_the_notes_of`
    // through the same `a_note_folder_for` this uses. What somebody sees is a
    // Notes list with nothing in it until they sync once.
    //
    // One kind of backend answers here today. A second one whose containers are
    // stored rather than fetched adds them to this list and changes nothing
    // else, which is what the module header promises about a second arm.
    the_calendar_servers_of(cache, account_id)
        .iter()
        .map(|journal| the_note_folder_for(cache, account_id, journal))
        .collect()
}

/// The note folder one calendar server's journal entries are.
///
/// One function rather than the same two arguments written out at each site, so
/// that the folder a sync runs against and the folder a screen shows are the
/// same row by construction.
fn the_note_folder_for(
    cache: &MessageCache,
    account_id: &str,
    journal: &ACalendarServersJournal,
) -> crate::common::Result<crate::data::message_cache::NoteFolderEntry> {
    cache.a_note_folder_for(account_id, &journal.container, &journal.called)
}

/// Send this account's notes wherever they go, and take back what has arrived.
///
/// The one place a backend is chosen, which is what this module is for. The
/// sync itself is [`crate::application::notes_sync`] and it names no backend:
/// everything about which client is built and where it is pointed is decided
/// here, in the match below, so a second backend is a second arm and changes
/// nothing else.
pub async fn sync_the_notes_of(
    cache: &MessageCache,
    account: &Account,
) -> crate::common::Result<WhatTheNotesSyncDid> {
    match for_account(Some(account), has_a_calendar_server(cache, &account.id)) {
        // Asked nothing of anybody, which is the point: an account whose notes
        // stay here is never offered this command in the first place, and
        // reaching it by another route still costs nobody a request.
        NotesBackend::ThisComputer | NotesBackend::Other(_) => {
            Ok(WhatTheNotesSyncDid::TheyStayHere)
        }
        NotesBackend::OneNote => {
            let Some(notebooks) =
                crate::service::onenote_notes::ANotebookOnAMicrosoftAccount::for_account(
                    &account.id,
                )
                .await
            else {
                return Ok(WhatTheNotesSyncDid::NobodyIsSignedIn);
            };
            // The sections are asked for here rather than kept anywhere,
            // because the containers of a hosted backend are the service's
            // answer and not a row on this computer. A section somebody made in
            // OneNote this morning is a folder here this afternoon without
            // anybody adding one.
            //
            // A read cut short is said rather than swallowed. A notebook nested
            // deeper than the walk follows comes back incomplete, and reporting
            // a clean sync over it is how somebody comes to trust a list that
            // is missing half of itself.
            let found = notebooks.the_sections().await?;
            let mut all = crate::application::notes_sync::NoteSyncResult::default();
            if !found.complete {
                all.errors.push(
                    "Some of this account's notebooks go deeper than Wixen Mail will \
                     follow, so not every section is a note folder here."
                        .to_string(),
                );
            }
            for section in &found.items {
                // Made before the sync is asked for, the same way and for the
                // same reason the calendar arm does it: a sync is about one
                // folder, and the folder a note is filed into and the folder a
                // screen shows have to be one row by construction.
                let folder = cache.a_note_folder_for(
                    &account.id,
                    &section.id,
                    &crate::service::onenote_notes::ANotebookOnAMicrosoftAccount::the_folder_name_of(
                        section,
                    ),
                )?;
                all.absorb(
                    crate::application::notes_sync::sync_notes(
                        cache,
                        &notebooks,
                        &account.id,
                        &folder.container.clone().unwrap_or_default(),
                    )
                    .await?,
                );
            }
            Ok(WhatTheNotesSyncDid::ItRan(all))
        }
        NotesBackend::CalDavJournal => {
            let journals = the_calendar_servers_of(cache, &account.id);
            if journals.is_empty() {
                // The answer above came from the same lookup, so this is only
                // reached if the row went between the two.
                return Ok(WhatTheNotesSyncDid::TheyStayHere);
            }
            let mut all = crate::application::notes_sync::NoteSyncResult::default();
            let mut somebody_is_signed_in = false;
            for journal in &journals {
                // Made before the sync is asked for, because a sync is about
                // one folder and this says which. The same call the list of
                // folders is built from, so the folder a note is filed into and
                // the folder a screen shows are one row by construction.
                the_note_folder_for(cache, &account.id, journal)?;
                let Some(service) =
                    crate::service::caldav_journal::AJournalOnACalendarServer::for_account(
                        &account.id,
                        &journal.calendar_id,
                    )
                else {
                    // One server nobody is signed in to does not stop the
                    // others. Recorded on the result, so the summary carries it
                    // whichever container it came from.
                    all.needs_sign_in = true;
                    continue;
                };
                somebody_is_signed_in = true;
                all.absorb(
                    crate::application::notes_sync::sync_notes(
                        cache,
                        &service,
                        &account.id,
                        &journal.container,
                    )
                    .await?,
                );
            }
            if !somebody_is_signed_in {
                // Nobody is signed in to any of them, which is the one thing on
                // this list only the person can fix. Said in its own words
                // rather than as a result whose every count is zero, which is
                // what the enum's own comment asks for.
                return Ok(WhatTheNotesSyncDid::NobodyIsSignedIn);
            }
            Ok(WhatTheNotesSyncDid::ItRan(all))
        }
    }
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
pub fn where_they_go(account: Option<&Account>, a_calendar_server: bool) -> String {
    match account {
        Some(account) => where_they_go_for(
            &for_account(Some(account), a_calendar_server),
            &account.display_name(),
        ),
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
        // It says it is experimental where somebody meets it before they rely
        // on it, which is `CLAUDE.md`'s rule about a warning that only exists
        // in a changelog. `calendar_source::NOT_TRIED_FOR_REAL` says the same
        // thing on the window that adds a calendar server, and this is a second
        // sentence rather than that one because it is about a second thing:
        // somebody who added a calendar months ago and has just found the notes
        // sync never saw that window today, and what has never been tried here
        // is the note half rather than the calendar half.
        NotesBackend::CalDavJournal => format!(
            "Notes in {account_named} are kept on this computer and sent to that \
             account's calendar server. This is experimental. No build has ever \
             sent a note to a real server, so expect problems, and turning on \
             Allow Changes is what lets a note go at all."
        ),
        // Its own sentence rather than the calendar server's with a word
        // swapped, because two things about it are different and both matter to
        // whoever reads it. It names OneNote, since somebody looking for their
        // notes has to know which application to open. And it says what a page
        // cannot hold, because unlike a calendar server OneNote really does
        // change a note on the way: `service::onenote_page`'s fidelity table
        // measures it construct by construct, and a person who finds out by
        // losing a code block has been told too late.
        NotesBackend::OneNote => format!(
            "Notes in {account_named} are kept on this computer and sent to OneNote, \
             one section for each note folder. This is experimental. No build has ever \
             sent a note to a real Microsoft account, so expect problems, and turning \
             on Allow Changes is what lets a note go at all. A page cannot hold bold, \
             italic, struck-out text, a quotation, code or a line across the page, so \
             a note carrying any of those comes back without it and the sync says so."
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
    a_calendar_server: bool,
) -> String {
    where_they_go(default_account(default_id, accounts), a_calendar_server)
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

/// One note as a backend holds it now: what it is called there, and what it
/// says.
///
/// Apart from [`ANoteThere`], which is identity and nothing else, because the
/// two are asked for at different times and at different prices. A sync asks
/// what a container holds on every run and has to be able to do that cheaply;
/// it asks what one note says only for the ones whose marker moved.
///
/// # This was missing, and nothing could arrive without it
///
/// `05.1-02` shipped [`NotesService`] with three operations and no way to read
/// a note's words: `notes_it_holds` answers identities, and the other two
/// write. A sync built on that could discover that a backend held a note it
/// had never seen and had nothing to write down. `05.1-03`, the first
/// implementation, is where that was found, which is what a first
/// implementation is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ANoteAsItStands {
    /// What the backend calls it, and the marker its copy carries now.
    pub known_as: ANoteThere,
    /// What the note is called.
    pub title: String,
    /// What it says, in whatever the backend gave, unchanged.
    pub body: String,
}

/// What a backend kept, where it could not keep what it was handed.
///
/// Only a backend knows this, which is why it is said here rather than worked
/// out by whatever is driving one. A sync comparing what came back later
/// against what it sent cannot tell a backend's own normalising from a change
/// somebody made at the other end, and guessing either way is how somebody's
/// note is quietly rewritten or a real change is quietly dropped.
///
/// Two formats in this program already lose bytes and neither is exotic. A
/// calendar journal document has one escape for a line break and no way to
/// write a carriage return inside a value, so a body typed on a Windows machine
/// comes back with plain line feeds. A OneNote page is an HTML document, and
/// HTML collapses a run of whitespace to one space and cannot hold one at the
/// end of a line, so the indentation that makes a nested list a nested list
/// does not survive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatTheBackendKept {
    /// What the note is called at the backend now.
    pub title: String,
    /// What the backend's copy says now.
    pub body: String,
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
    Done {
        /// What the backend calls the note now.
        known_as: ANoteThere,
        /// What it really kept, where that is not what it was handed.
        ///
        /// `None` from a backend that kept the bytes, which is the ordinary
        /// answer and the one [`WhatTheBackendSaid::done`] gives.
        ///
        /// Its own field rather than a second variant, because "it worked" and
        /// "it worked and the copy there is not quite the copy here" are one
        /// outcome with one thing to write down afterwards, and splitting them
        /// would give every caller two arms to keep in step.
        what_it_could_keep: Option<WhatTheBackendKept>,
    },
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

impl WhatTheBackendSaid {
    /// It did what was asked and kept exactly what it was given.
    ///
    /// The ordinary answer, and a removal's only one: a note taken away has no
    /// copy left to have kept anything differently.
    pub fn done(known_as: ANoteThere) -> Self {
        Self::Done {
            known_as,
            what_it_could_keep: None,
        }
    }
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

    /// What one note the backend holds says now.
    ///
    /// `None` where the backend no longer holds it, which is not an error:
    /// somebody deleted it at the other end between the listing and this.
    ///
    /// Asked for one note rather than folded into [`Self::notes_it_holds`] so
    /// that a container of five hundred notes is not read whole on every sync
    /// to find the two that moved. A backend that really answers both in one
    /// request, which a CalDAV `REPORT` does, is free to remember what it
    /// already read; a backend that cannot, which OneNote cannot because a
    /// page's content is a second call, is not asked to pretend it can.
    fn what_a_note_says(
        &self,
        container: &str,
        known_as: &ANoteThere,
    ) -> impl std::future::Future<Output = crate::common::Result<Option<ANoteAsItStands>>> + Send;

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

    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::CalendarContainer;

    fn account(id: &str, email: &str) -> Account {
        let mut account = Account::new("Work".to_string(), email.to_string());
        account.id = id.to_string();
        account
    }

    fn a_store() -> TempHome<MessageCache> {
        TempHome::named("wixen_notes_backend_", |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a store to write into")
        })
    }

    /// A calendar of this account's, from wherever the caller says.
    fn a_calendar(id: &str, came_from: &str, at: Option<&str>) -> CalendarContainer {
        CalendarContainer {
            id: id.to_string(),
            account_id: "a1".to_string(),
            name: "Work".to_string(),
            color: String::new(),
            source_provider: Some(came_from.to_string()),
            caldav_url: at.map(str::to_string),
            subscription_url: None,
            is_default: false,
            is_visible: true,
            is_read_only: false,
            display_order: 0,
            etag: None,
            ctag: None,
            sync_token: None,
            refresh_interval_minutes: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn test_an_account_with_a_calendar_server_has_somewhere_to_put_its_notes() {
        // The arm that makes this seam answer anything but "here", and the
        // whole reason `05.1-03` can reach production at all. A server holding
        // somebody's calendar holds their journal entries in the same place
        // under the same sign-in, so nobody has to type a second address.
        let cache = a_store();
        cache
            .save_calendar(&a_calendar(
                "cal-1",
                crate::application::calendar_source::ON_A_SERVER,
                Some("https://example.test/dav/cal"),
            ))
            .expect("a calendar on a server");

        assert!(has_a_calendar_server(&cache, "a1"));
        assert_eq!(
            for_account(Some(&account("a1", "me@gmail.com")), true),
            NotesBackend::CalDavJournal,
            "an account with a calendar server was told its notes stay here"
        );
    }

    #[test]
    fn test_a_feed_somebody_subscribed_to_is_not_somewhere_to_put_notes() {
        // A published feed is read and never written to, so it is not a place
        // a note can go. Asked as its own fixture because the row looks almost
        // the same: it is a calendar, on this account, that came from a URL.
        let cache = a_store();
        cache
            .save_calendar(&a_calendar(
                "cal-feed",
                crate::application::calendar_source::FROM_A_FEED,
                Some("https://example.test/holidays.ics"),
            ))
            .expect("a feed");

        assert!(!has_a_calendar_server(&cache, "a1"));
    }

    #[test]
    fn test_a_calendar_row_with_no_address_on_it_is_not_a_server() {
        // Half a row, which is what a discovery that failed part way leaves.
        // The word alone would count it, and a note offered to an address that
        // is not there is a sync that fails on every run.
        let cache = a_store();
        cache
            .save_calendar(&a_calendar(
                "cal-half",
                crate::application::calendar_source::ON_A_SERVER,
                None,
            ))
            .expect("half a calendar");

        assert!(!has_a_calendar_server(&cache, "a1"));
    }

    #[test]
    fn test_an_account_with_no_calendars_at_all_has_no_calendar_server() {
        let cache = a_store();
        assert!(!has_a_calendar_server(&cache, "a1"));
    }

    fn run<F: std::future::Future>(work: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("a runtime")
            .block_on(work)
    }

    #[test]
    fn test_an_account_whose_notes_stay_here_is_never_asked_for_a_backend() {
        // Nothing is built, nothing is pointed anywhere and nobody is asked.
        // The assertion that matters is the answer rather than the absence: a
        // sync that did nothing because nothing exists yet would pass a test
        // written the other way round before any of this was built.
        let cache = a_store();
        let said =
            run(sync_the_notes_of(&cache, &account("a1", "me@gmail.com"))).expect("an answer");

        assert!(
            matches!(said, WhatTheNotesSyncDid::TheyStayHere),
            "{said:?}"
        );
    }

    #[test]
    fn test_an_account_with_a_server_nobody_has_signed_in_to_is_told_so() {
        // Its own answer rather than an error, because it is the one thing on
        // the list only the person can fix and a count says "1 problem" on
        // every sync forever while telling them nothing.
        let cache = a_store();
        cache
            .save_calendar(&a_calendar(
                "cal-nobody-signed-in-to",
                crate::application::calendar_source::ON_A_SERVER,
                Some("https://example.test/dav/cal"),
            ))
            .expect("a calendar on a server");

        let said =
            run(sync_the_notes_of(&cache, &account("a1", "me@gmail.com"))).expect("an answer");

        assert!(
            matches!(said, WhatTheNotesSyncDid::NobodyIsSignedIn),
            "{said:?}"
        );
    }

    /// Asking for a sync really runs one.
    ///
    /// Read as source, which is what `data::config`'s own screen checks do and
    /// for the same reason: the window needs a display and a running
    /// application, and nothing about it can be driven from a test. What this
    /// catches is the stub shape this project has met before, where a handler
    /// reports success and calls nothing, and a test that only asks whether the
    /// menu offers the command passes against it every time.
    ///
    /// What it cannot see is said here rather than left to be assumed. It reads
    /// two names out of one file. It does not press a key, does not open a
    /// window, and says nothing about whether the status line that comes back
    /// is heard.
    #[test]
    fn test_asking_for_a_notes_sync_reaches_the_sync_rather_than_reporting_one() {
        let window = std::fs::read_to_string("src/presentation/wx_app.rs")
            .expect("the main window's source");
        assert!(
            !window.is_empty(),
            "the main window could not be read, so this proves nothing"
        );

        // The command reaches the spawn.
        assert!(
            window.contains("PimModule::Notes => {") && window.contains("spawn_notes_sync(app);"),
            "choosing Sync now on a note folder does not reach a notes sync, so \
             the command is a stop somebody lands on that does nothing"
        );
        // And the spawn reaches the work, rather than reporting that it did.
        assert!(
            window.contains("sync_the_notes_of(&cache, &account)"),
            "the notes sync handler never asks for a sync, so it reports one \
             that never happened"
        );
    }

    #[test]
    fn test_an_account_whose_notes_reach_a_server_is_told_that_has_never_been_tried() {
        // A warning that only exists in a changelog is a warning nobody gets.
        // This one is on the settings screen, beside the sentence saying where
        // the notes go, which is where somebody looks before they trust it.
        let said = where_they_go_for(&NotesBackend::CalDavJournal, "Work");

        assert!(said.contains("experimental"), "{said}");
        assert!(said.contains("real server"), "{said}");
        // And what it costs them to find out, said rather than implied.
        assert!(said.contains("Allow Changes"), "{said}");
    }

    #[test]
    fn test_a_gmail_accounts_notes_stay_on_this_computer() {
        // Google Keep's API is Workspace only. This is not a gap waiting to be
        // filled: it is still the answer after all three backends ship.
        assert_eq!(
            for_account(Some(&account("a1", "me@gmail.com")), false),
            NotesBackend::ThisComputer
        );
    }

    #[test]
    fn test_an_outlook_accounts_notes_go_to_onenote() {
        // The arm that changed in 5.2, and the whole of what makes a note made
        // in a Microsoft account reach anybody. Its neighbour above stays
        // false, and the two together are the answer per provider rather than
        // one answer for both.
        assert_eq!(
            for_account(Some(&account("a1", "me@outlook.com")), false),
            NotesBackend::OneNote
        );
    }

    #[test]
    fn test_an_account_whose_notes_reach_onenote_is_told_that_has_never_been_tried() {
        // The same promise the calendar server's sentence makes, for the same
        // reason: a warning that only exists in a changelog is a warning
        // nobody gets. It names OneNote, because somebody looking for their
        // notes needs to know where to look.
        let said = where_they_go_for(&NotesBackend::OneNote, "Work");

        assert!(said.contains("OneNote"), "{said}");
        assert!(said.contains("experimental"), "{said}");
        assert!(said.contains("Allow Changes"), "{said}");
        // Not "yet". A sentence that says a backend is coming has to be
        // rewritten when it arrives, and this one arrived.
        assert!(!said.contains("yet"), "{said}");
    }

    #[test]
    fn test_an_outlook_account_nobody_has_signed_in_to_is_told_so_rather_than_told_notes_stay_here()
    {
        // Nothing is stored in this machine's credential store for this
        // account, which is the state every account is in during a test and
        // the state a real one is in before anybody signs in. It is the one
        // thing on the list only the person can fix, so it is said in its own
        // words rather than reported as an account whose notes stay here,
        // which would be a lie about where they go.
        let cache = a_store();

        let said =
            run(sync_the_notes_of(&cache, &account("a1", "me@outlook.com"))).expect("an answer");

        assert!(
            matches!(said, WhatTheNotesSyncDid::NobodyIsSignedIn),
            "{said:?}"
        );
    }

    #[test]
    fn test_an_account_whose_provider_this_build_does_not_recognise_keeps_its_notes_here() {
        // Answered rather than refused and rather than guessed. A provider
        // this build has never heard of has no notes backend this build can
        // talk to, and saying so is an answer.
        assert_eq!(
            for_account(Some(&account("a1", "me@myhost.example")), false),
            NotesBackend::ThisComputer
        );
    }

    #[test]
    fn test_an_account_with_no_provider_at_all_keeps_its_notes_here() {
        // Every plain IMAP and POP account, and somebody who has not signed in
        // anywhere yet. Both reach this through `None`.
        assert_eq!(for_account(None, false), NotesBackend::ThisComputer);
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
        assert!(NotesBackend::OneNote.goes_somewhere_else());
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
            for_default_account(Some("a2"), &accounts, false),
            for_account(Some(&accounts[1]), false)
        );
    }

    #[test]
    fn test_the_sentence_names_the_account_it_is_about() {
        // The settings screen can only reach the default account, so a
        // sentence saying "your notes" would read as a statement about all of
        // them. It is not one now and will be less so once a backend ships.
        let said = where_they_go(Some(&account("a1", "me@gmail.com")), false);

        assert!(said.contains("Work"), "{said}");
    }

    #[test]
    fn test_an_account_with_no_notes_backend_is_told_so_rather_than_told_not_yet() {
        // Not a placeholder a backend removes. A consumer Gmail account has no
        // notes backend after all three ship, because Google Keep's API is
        // Workspace only, so this sentence has real work left in it. "Not yet"
        // would have to be rewritten and would tell somebody to wait for
        // something that is not coming.
        let said = where_they_go(Some(&account("a1", "me@gmail.com")), false);

        assert!(said.contains("no notes backend"), "{said}");
        assert!(said.contains("on this computer"), "{said}");
        assert!(!said.contains("yet"), "{said}");
    }

    #[test]
    fn test_somebody_with_no_default_account_is_still_told_where_their_notes_are() {
        // Before any account is set up, and after the default one is deleted.
        // Both reach a screen, and a blank where the sentence should be is
        // worse than the answer, which is that the notes are here.
        let said = where_the_default_accounts_notes_go(None, &[], false);

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
            for_default_account(Some("gone"), &[], false),
            NotesBackend::ThisComputer
        );
        assert_eq!(
            for_default_account(None, &[], false),
            NotesBackend::ThisComputer
        );
    }

    /// A calendar of this account's, under a name of its own.
    fn a_calendar_called(id: &str, name: &str, at: &str) -> CalendarContainer {
        CalendarContainer {
            name: name.to_string(),
            ..a_calendar(
                id,
                crate::application::calendar_source::ON_A_SERVER,
                Some(at),
            )
        }
    }

    #[test]
    fn test_every_calendar_server_on_an_account_is_a_note_folder_of_its_own() {
        // Ledger 245. An account with two calendar servers used to send its
        // notes to whichever the store answered with first, and nothing asked
        // the person which. With a folder for each there is no first to pick.
        let cache = a_store();
        for (id, name, at) in [
            ("cal-1", "Work", "https://example.test/dav/work"),
            ("cal-2", "Home", "https://example.test/dav/home"),
        ] {
            cache
                .save_calendar(&a_calendar_called(id, name, at))
                .expect("a calendar on a server");
        }

        let folders = note_folders_for_the_backends_of(&cache, "a1").expect("the folders");

        assert_eq!(
            folders
                .iter()
                .map(|folder| (folder.name.as_str(), folder.container.as_deref()))
                .collect::<Vec<_>>(),
            [
                ("Home", Some("https://example.test/dav/home")),
                ("Work", Some("https://example.test/dav/work")),
            ],
            "an account with two calendar servers did not get a folder for each"
        );
        // And in the order the sidebar shows the calendars in, which is what
        // the doc comment promises and is the reason the pairs above read
        // Home before Work rather than in the order they were stored.
        // Compared against the store's own answer rather than written out, or
        // this asserts an alphabetical coincidence of the fixture.
        assert_eq!(
            folders
                .iter()
                .map(|folder| folder.name.as_str())
                .collect::<Vec<_>>(),
            cache
                .get_calendars_for_account("a1")
                .expect("the calendars")
                .iter()
                .map(|calendar| calendar.name.as_str())
                .collect::<Vec<_>>(),
            "the folders are not in the order their calendars are"
        );
    }

    #[test]
    fn test_a_calendar_that_is_on_no_server_is_not_a_note_folder() {
        // A calendar made here and one somebody subscribed to are not places
        // notes can be written. A folder for either is a folder whose every
        // sync fails, or worse, one that looks like somewhere to file a note.
        let cache = a_store();
        cache
            .save_calendar(&a_calendar("cal-here", "local", None))
            .expect("a calendar made here");
        cache
            .save_calendar(&a_calendar("cal-feed", "subscription", None))
            .expect("a calendar somebody subscribed to");

        assert!(
            note_folders_for_the_backends_of(&cache, "a1")
                .expect("the folders")
                .is_empty(),
            "a calendar that holds nobody's journal entries became a note folder"
        );
    }

    #[test]
    fn test_a_folder_somebody_made_here_is_not_a_place_this_account_syncs() {
        // The other half of requirement 4: a folder made here has no container,
        // so it is never one of the places a sync is run for, and it is left
        // exactly as it was.
        let cache = a_store();
        let made_here = cache
            .ensure_default_note_folder("a1")
            .expect("a folder made here");
        cache
            .save_calendar(&a_calendar_called(
                "cal-1",
                "Work",
                "https://example.test/dav/work",
            ))
            .expect("a calendar on a server");

        let folders = note_folders_for_the_backends_of(&cache, "a1").expect("the folders");

        assert_eq!(
            folders.iter().map(|f| f.id.as_str()).collect::<Vec<_>>(),
            [folders[0].id.as_str()],
            "more places to sync than there are calendar servers"
        );
        assert_ne!(
            folders[0].id, made_here.id,
            "a folder somebody made here was turned into one a backend syncs"
        );
        assert_eq!(
            cache
                .get_note_folder(&made_here.id)
                .expect("the folder is read")
                .expect("the folder is still there")
                .container,
            None,
            "a folder made here was given a container"
        );
    }
}

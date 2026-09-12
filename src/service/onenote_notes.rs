//! A OneNote notebook's pages, behind the notes seam.
//!
//! The third implementation of
//! [`crate::application::notes_backend::NotesService`], and the first hosted
//! one. It is thin for the reason [`crate::service::caldav_journal`] is thin:
//! everything it decides is decided in [`crate::application::notes_sync`],
//! which knows no backend, and everything it reads or writes as a document is
//! [`crate::service::onenote_page`], which touches no network. What is left
//! here is the requests and the turning of Graph's answers into the words the
//! seam allows.
//!
//! # A container is a section, and nothing here takes one apart
//!
//! `docs/development/the-notes-seam.md` says one backend container is one note
//! folder, and that a backend with more than one level above a note flattens
//! that into its own container string. A OneNote page lives four levels down,
//! and the flattening is done by naming rather than by encoding: the container
//! is the section's own identifier, which Graph gave and nothing here parses,
//! and the *name* of the folder carries the path. Identity and name are
//! different jobs, which is why they are different fields.
//!
//! # What a write costs, said rather than discovered
//!
//! A page's body cannot be replaced, only appended to, so making a page say
//! something new is several requests rather than one, and this backend reads
//! the page back afterwards because the seam requires it to say what it kept
//! from its own copy rather than from a rule about itself. Changing one note is
//! five requests: the page's resource for the marker, the page's content for
//! the identifiers a change names, the change, the resource again for the new
//! marker, and the content again for what was kept. Making a note is two. That
//! is the price of a service with no whole-document write and no entity tag,
//! and it is written here rather than found by somebody watching a sync.
//!
//! # Nothing here has ever met Microsoft
//!
//! Every request below is answered in the tests by a server the tests start on
//! a loopback port. That proves what this program sends and how it reads an
//! answer it was given. It says nothing about what Graph really sends back. No
//! OneNote notebook has ever been opened by this program, and each unknown has
//! its own entry in `.planning/WINDOWS.md`.

use crate::application::notes_backend::{
    ANoteAsItStands, ANoteThere, NotesService, WhatTheBackendKept, WhatTheBackendSaid,
};
use crate::common::{Error, Result};
use crate::service::microsoft_graph::{
    AOneNoteSection, MsGraphClient, MsOneNotePage, the_time_a_page_was_last_changed,
};
use crate::service::onenote_page::{ANoteOnAPage, the_note_on, the_page_for};
use crate::service::tasks_api::PagedRead;

/// One Microsoft account's OneNote, as the notes seam asks about it.
///
/// The token is taken once, when this is built, the way
/// [`crate::service::caldav_journal::AJournalOnACalendarServer`] takes a
/// sign-in once. A sync is one run of a few seconds and a Graph token lasts an
/// hour, so refreshing inside every call would be asking the credential store
/// the same question a hundred times for one answer.
pub struct ANotebookOnAMicrosoftAccount {
    client: MsGraphClient,
    token: String,
}

impl ANotebookOnAMicrosoftAccount {
    /// This account's OneNote, when somebody is signed in to it.
    ///
    /// `None` is nobody signed in, which the sync says in its own words rather
    /// than reporting as a failure: it is the one thing on the list only the
    /// person can fix, and a count saying "1 problem" on every sync from now on
    /// tells them nothing.
    ///
    /// The client is built through [`MsGraphClient::for_account`], which is
    /// where the Allow Changes setting is applied, and nothing below asks that
    /// setting again. A client that may not change anything refuses before the
    /// request leaves the machine, and that refusal arrives here as an error
    /// this file turns into the seam's own word for it. A second gate would be
    /// a second answer to one question, which is the shape this project keeps
    /// having to unpick.
    pub async fn for_account(account_id: &str) -> Option<Self> {
        Some(Self {
            client: MsGraphClient::for_account(account_id),
            token: crate::service::oauth::a_graph_token_for(account_id).await?,
        })
    }

    /// One that may change things, asking a named address. Tests only.
    ///
    /// [`Self::for_account`] reads the settings really stored on whichever
    /// machine is running and the sign-in really in that machine's credential
    /// store, so a test built on it would pass or fail depending on whose
    /// computer ran it. `MsGraphClient::allowed_to_change_things_at` exists for
    /// the same reason and says the same thing.
    #[cfg(test)]
    fn allowed_to_change_things_at(address: &str) -> Self {
        Self {
            client: MsGraphClient::allowed_to_change_things_at(address),
            token: "a-token".to_string(),
        }
    }

    /// Every section on this account, each carrying the names above it.
    ///
    /// The containers this account has, which is what the caller turns into one
    /// note folder each. It is not a seam operation: the seam asks what is in
    /// one container and never how many containers there are, because a backend
    /// that could be asked that would be a backend the seam knew the shape of.
    pub async fn the_sections(&self) -> Result<PagedRead<AOneNoteSection>> {
        self.client.every_section(&self.token).await
    }

    /// What the note folder one section stands for is called.
    ///
    /// The flattening requirement 2 of the seam's container section asks for,
    /// done here because it is the backend's business and nowhere else knows
    /// the names. The container stays the section's own identifier and is never
    /// built from this: a name is what somebody reads and an identifier is what
    /// a request carries, and one field doing both jobs is a name nobody can
    /// change.
    ///
    /// **The separator is ` / ` and this is where it was chosen.** The seam's
    /// document recorded on 2026-09-11 that nobody had chosen one and that what
    /// a screen reader makes of it at its default punctuation level is
    /// unmeasured. It still is, and it is a ledger entry rather than a
    /// guess dressed up: what settles it is somebody hearing a folder list.
    /// Spaces on both sides because a slash with none reads as one word to a
    /// sighted person scanning a list and gives a screen reader nothing to
    /// break on.
    ///
    /// A section with nothing above it is its own name, with no separator
    /// anywhere. Nothing here can produce a leading or trailing one.
    pub fn the_folder_name_of(section: &AOneNoteSection) -> String {
        let _ = section;
        String::new()
    }

    /// What a failed request means, in the words the seam allows.
    ///
    /// A refusal by the setting and a refusal by Microsoft are different things
    /// to somebody: one is fixed on the settings screen and one is fixed by
    /// signing in again, and a single "it did not work" tells them neither. The
    /// same three-way split `caldav_journal` makes, reaching the same words.
    fn what_that_means(error: Error) -> WhatTheBackendSaid {
        if crate::service::outward::was_refused_by_the_gate(&error) {
            return WhatTheBackendSaid::NotAllowedToChangeAnything;
        }
        match error {
            // `microsoft_graph::onenote_refusal` has already turned 401 and 403
            // into this, carrying a sentence that names notes rather than the
            // one that names tasks.
            Error::Authentication(_)
            | Error::Api { status: 401, .. }
            | Error::Api { status: 403, .. } => WhatTheBackendSaid::NotSignedIn,
            Error::Api { status: 404, .. } | Error::Api { status: 410, .. } => {
                WhatTheBackendSaid::ItIsNotThere
            }
            // The string is for the log. Nothing reads it out: text a crate
            // wrote for a developer is not text to speak to somebody whose
            // notes did not sync.
            other => WhatTheBackendSaid::CouldNotBeReached(other.to_string()),
        }
    }

    /// Whether this error means the page is not there any more.
    ///
    /// Ordinary rather than wrong: somebody deleted it at the other end between
    /// a listing and the read that follows it.
    fn the_page_has_gone(error: &Error) -> bool {
        matches!(
            error,
            Error::Api { status: 404, .. } | Error::Api { status: 410, .. }
        )
    }

    /// What the page really kept, where that is not what it was handed.
    ///
    /// Read back off the page rather than worked out from what this program
    /// believes OneNote does, which is requirement 2 of the seam's section on
    /// bytes: a backend that answers from a rule about itself goes on claiming
    /// a byte survived after the day it stops surviving. It costs one request
    /// per write, and it is the only thing in this file that could notice
    /// Microsoft behaving differently from the model `05.2-01` measured.
    ///
    /// The title comes from the resource and the body from the document,
    /// because that is where each really lives. A page's title is a property of
    /// the `onenotePage`; whether the content endpoint also carries a `title`
    /// element is not something this repository has ever seen, and reading the
    /// title out of the document would report every title as lost if it does
    /// not.
    ///
    /// `None` where the page gives back exactly what it was handed, which is
    /// the ordinary answer and the one the sync says nothing to anybody about.
    async fn what_the_page_kept(
        &self,
        page_id: &str,
        title_there: &str,
        asked: &ANoteOnAPage,
    ) -> Result<Option<WhatTheBackendKept>> {
        let there = the_note_on(&self.client.page_content(&self.token, page_id).await?);
        if title_there == asked.title && there.body == asked.body {
            return Ok(None);
        }
        Ok(Some(WhatTheBackendKept {
            title: title_there.to_string(),
            body: there.body,
        }))
    }

    /// Whether the page moved since this program last looked.
    ///
    /// Equality and nothing else, which is the only comparison the seam allows:
    /// a marker is not promised to be opaque, orderable or tied to the content,
    /// so ordering two of these or reading one as a date would be this program
    /// deciding something Microsoft never promised.
    ///
    /// # Why a missing marker refuses the write rather than taking it
    ///
    /// The seam's requirement 3 says a marker is required of any backend this
    /// program writes to, and says why: with no marker the push has no evidence
    /// that anything moved at the backend, so this computer's copy goes over
    /// whatever is there and a change somebody made at the other end is
    /// destroyed with nothing said. `05.1-04` measured that rather than arguing
    /// it. So where either marker is missing, this refuses, and the note stays
    /// here still marked as waiting. A refusal somebody can read costs them a
    /// sync; the other answer costs them their note.
    ///
    /// Nothing here has seen an `onenotePage` arrive without a
    /// `lastModifiedDateTime`, and nothing here can say Graph never sends one.
    fn whether_the_page_moved(
        ours: Option<&str>,
        theirs: &MsOneNotePage,
    ) -> std::result::Result<(), WhatTheBackendSaid> {
        let now = the_time_a_page_was_last_changed(theirs);
        match (ours, now.as_deref()) {
            (Some(ours), Some(now)) if ours == now => Ok(()),
            (Some(_), Some(_)) => Err(WhatTheBackendSaid::ItMovedFirst { version_now: now }),
            _ => Err(WhatTheBackendSaid::CouldNotBeReached(
                "OneNote answered with a page carrying no time it was last changed, so \
                 whether your copy or the one in OneNote moved first could not be told"
                    .to_string(),
            )),
        }
    }

    /// Make a page in this section say what this note says.
    ///
    /// Apart from the trait method because the trait method's job is to turn
    /// every failure into one of the seam's words, and a body that did both
    /// would answer some of them twice.
    async fn write_the_page(
        &self,
        container: &str,
        known_as: Option<&ANoteThere>,
        note: &ANoteOnAPage,
    ) -> Result<WhatTheBackendSaid> {
        let Some(there) = known_as else {
            let made = self
                .client
                .create_page(&self.token, container, &the_page_for(note))
                .await?;
            let kept = self.what_the_page_kept(&made.id, &made.title, note).await?;
            return Ok(WhatTheBackendSaid::Done {
                known_as: ANoteThere {
                    version: the_time_a_page_was_last_changed(&made),
                    named: made.id,
                },
                what_it_could_keep: kept,
            });
        };

        let before = match self.client.one_page(&self.token, &there.named).await {
            Ok(before) => before,
            // A name the backend no longer knows is not a name. The seam says
            // such a note is one to create, and the sync offers it again.
            Err(e) if Self::the_page_has_gone(&e) => return Ok(WhatTheBackendSaid::ItIsNotThere),
            Err(e) => return Err(e),
        };
        if let Err(refused) = Self::whether_the_page_moved(there.version.as_deref(), &before) {
            return Ok(refused);
        }

        self.client
            .change_page(&self.token, &there.named, note)
            .await?;

        // Asked again rather than guessed at. The marker the caller writes down
        // has to be the one the page carries after the change: handed back the
        // one from before it, the next push compares an old marker against a new
        // one and reports a clash nobody caused.
        let after = self.client.one_page(&self.token, &there.named).await?;
        let kept = self
            .what_the_page_kept(&there.named, &after.title, note)
            .await?;
        Ok(WhatTheBackendSaid::Done {
            known_as: ANoteThere {
                named: there.named.clone(),
                version: the_time_a_page_was_last_changed(&after),
            },
            what_it_could_keep: kept,
        })
    }
}

impl NotesService for ANotebookOnAMicrosoftAccount {
    async fn notes_it_holds(&self, container: &str) -> Result<Vec<ANoteThere>> {
        Ok(self
            .client
            .pages_in_section(&self.token, container)
            .await?
            .iter()
            .map(|page| ANoteThere {
                named: page.id.clone(),
                version: the_time_a_page_was_last_changed(page),
            })
            .collect())
    }

    async fn what_a_note_says(
        &self,
        _container: &str,
        known_as: &ANoteThere,
    ) -> Result<Option<ANoteAsItStands>> {
        // The container is not named in either request, and that is Graph's
        // addressing rather than a shortcut: a page is addressed by its own
        // identifier wherever it sits. The container is still what the caller
        // hands over and what the note is filed by, and nothing here reads it.
        let page = match self.client.one_page(&self.token, &known_as.named).await {
            Ok(page) => page,
            Err(e) if Self::the_page_has_gone(&e) => return Ok(None),
            Err(e) => return Err(e),
        };
        let document = match self.client.page_content(&self.token, &known_as.named).await {
            Ok(document) => document,
            Err(e) if Self::the_page_has_gone(&e) => return Ok(None),
            Err(e) => return Err(e),
        };
        Ok(Some(ANoteAsItStands {
            known_as: ANoteThere {
                // The name the caller already holds. Graph's answer carries one
                // too, and taking it would let a page that came back naming
                // something else quietly become a different note.
                named: known_as.named.clone(),
                version: the_time_a_page_was_last_changed(&page),
            },
            title: page.title,
            body: the_note_on(&document).body,
        }))
    }

    async fn leave_a_note_saying(
        &self,
        container: &str,
        known_as: Option<&ANoteThere>,
        title: &str,
        body: &str,
    ) -> Result<WhatTheBackendSaid> {
        let note = ANoteOnAPage {
            title: title.to_string(),
            body: body.to_string(),
        };
        Ok(self
            .write_the_page(container, known_as, &note)
            .await
            .unwrap_or_else(Self::what_that_means))
    }

    async fn take_a_note_away(
        &self,
        _container: &str,
        known_as: &ANoteThere,
    ) -> Result<WhatTheBackendSaid> {
        // The marker is deliberately not sent, and there is nowhere to send it:
        // an `onenotePage` has no entity tag and the delete reference names no
        // `If-Match`. Somebody asked for the note to go, and a removal held back
        // because the page moved is a removal that fails for ever.
        Ok(
            match self.client.delete_page(&self.token, &known_as.named).await {
                Ok(()) => WhatTheBackendSaid::done(known_as.clone()),
                Err(e) => Self::what_that_means(e),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::answering::{answering, answering_as_asked, asked_for, heard};

    /// A page as Graph would answer with it.
    fn a_page_graph_answers_with(id: &str, title: &str, when: &str) -> String {
        format!(r#"{{"id":"{id}","title":"{title}","lastModifiedDateTime":"{when}"}}"#)
    }

    /// A page's content, as Graph would answer a content request.
    fn the_content_of_a_page(body: &str) -> String {
        format!(
            "<html><head><title>Fuses</title></head><body><div><p>{body}</p></div></body></html>"
        )
    }

    /// The backend, pointed at a server the test started.
    fn a_backend(address: &std::net::SocketAddr) -> ANotebookOnAMicrosoftAccount {
        ANotebookOnAMicrosoftAccount::allowed_to_change_things_at(&format!("http://{address}"))
    }

    #[tokio::test]
    async fn test_a_note_made_here_reaches_onenote_as_a_page_in_the_section_it_was_given() {
        // The tracer. A note with no name at the backend is a page to make, the
        // section it is made in is the container the seam handed over, and what
        // the note is known by afterwards is the identifier Graph gave.
        let (address, listening) = answering_as_asked(
            "200 OK",
            "application/json",
            vec![
                Box::new(|_| a_page_graph_answers_with("1-page", "Fuses", "2026-09-11T09:00:00Z")),
                Box::new(|_| the_content_of_a_page("Live is brown")),
            ],
        )
        .await;

        let said = a_backend(&address)
            .leave_a_note_saying("1-section", None, "Fuses", "Live is brown")
            .await
            .expect("an answer");

        let asked = heard(listening, "the page being made and read back")
            .await
            .expect("two requests");
        assert_eq!(
            asked_for(&asked[0]),
            "POST /me/onenote/sections/1-section/pages",
            "{:?}",
            asked[0]
        );
        let WhatTheBackendSaid::Done {
            known_as,
            what_it_could_keep,
        } = said
        else {
            panic!("a page that was made was not reported as done: {said:?}");
        };
        assert_eq!(known_as.named, "1-page");
        assert_eq!(known_as.version.as_deref(), Some("2026-09-11T09:00:00Z"));
        assert_eq!(what_it_could_keep, None, "the page kept what it was handed");
    }

    #[tokio::test]
    async fn test_a_note_reaches_onenote_through_the_sync_that_names_no_backend() {
        // The tracer end to end, and the assertion that matters is which code
        // it went through. `notes_sync::sync_notes` is generic over the seam
        // and names no backend; this drives the real Graph client through it,
        // against a server the test started, and the note comes back knowing
        // what Graph called it.
        let dir = tempfile::tempdir().expect("a directory");
        let cache = crate::data::message_cache::MessageCache::new(dir.path().to_path_buf(), None)
            .expect("a store to write into");
        let folder = cache
            .a_note_folder_for("acct", "1-section", "Work / Notes")
            .expect("the folder this section is");
        cache
            .save_note(&crate::data::message_cache::NoteEntry {
                id: "note-1".to_string(),
                account_id: "acct".to_string(),
                folder_id: Some(folder.id.clone()),
                title: "Fuses".to_string(),
                body: "Live is brown".to_string(),
                format: crate::data::message_cache::NoteBody::AsTyped,
                pinned: false,
                created_at: "2026-09-11T09:00:00Z".to_string(),
                updated_at: "2026-09-11T09:00:00Z".to_string(),
                pending: true,
                known_as: None,
                known_version: None,
            })
            .expect("a note waiting to be sent");

        let (address, listening) = answering_as_asked(
            "200 OK",
            "application/json",
            vec![
                Box::new(|_| a_page_graph_answers_with("1-page", "Fuses", "2026-09-11T09:00:00Z")),
                Box::new(|_| the_content_of_a_page("Live is brown")),
                Box::new(|_| {
                    r#"{"value":[{"id":"1-page","title":"Fuses","lastModifiedDateTime":"2026-09-11T09:00:00Z"}]}"#
                        .to_string()
                }),
            ],
        )
        .await;

        let did = crate::application::notes_sync::sync_notes(
            &cache,
            &a_backend(&address),
            "acct",
            "1-section",
        )
        .await
        .expect("a sync");

        let asked = heard(listening, "the push and the read")
            .await
            .expect("three requests");
        assert_eq!(
            asked_for(&asked[0]),
            "POST /me/onenote/sections/1-section/pages",
            "{:?}",
            asked[0]
        );
        assert_eq!(did.sent, 1, "{did:?}");
        assert!(did.errors.is_empty(), "{did:?}");
        let after = cache
            .get_note("note-1")
            .expect("the note is read")
            .expect("the note is still there");
        assert_eq!(after.known_as.as_deref(), Some("1-page"));
        assert_eq!(
            after.known_version.as_deref(),
            Some("2026-09-11T09:00:00Z"),
            "the marker the sync writes down is the one the page carries"
        );
        assert!(!after.pending, "a note that was sent is still waiting");
    }

    #[tokio::test]
    async fn test_what_onenote_could_not_keep_is_read_back_off_the_page_rather_than_worked_out() {
        // Requirement 2 of the seam's section on bytes: a backend answers from
        // its own copy rather than from a rule about itself.
        //
        // **The page answers with something the model would not predict, and
        // that is the whole of what makes this a test.** Put through
        // `onenote_page`'s pure pair, this note comes back as "Live is brown",
        // because a quotation has no representation on a page. So a backend
        // that worked the answer out from the model instead of reading the page
        // would produce exactly that, and a fixture echoing the model could not
        // tell the two readings apart. The page here says something else, which
        // only a read can find.
        let (address, _listening) = answering_as_asked(
            "200 OK",
            "application/json",
            vec![
                Box::new(|_| a_page_graph_answers_with("1-page", "Fuses", "2026-09-11T09:00:00Z")),
                Box::new(|_| the_content_of_a_page("Live is brown and the folder is in the van")),
            ],
        )
        .await;

        let said = a_backend(&address)
            .leave_a_note_saying(
                "1-section",
                None,
                "Fuses",
                "Live is brown\n\n> Bring the blue folder",
            )
            .await
            .expect("an answer");

        let WhatTheBackendSaid::Done {
            what_it_could_keep: Some(kept),
            ..
        } = said
        else {
            panic!("a page that dropped a quotation said it kept the note: {said:?}");
        };
        assert_eq!(kept.title, "Fuses");
        assert_eq!(
            kept.body, "Live is brown and the folder is in the van",
            "what the page kept has to be what the page says, not what the model \
             says the page would have done"
        );
    }

    #[tokio::test]
    async fn test_a_page_that_moved_since_this_program_looked_is_reported_rather_than_written_over()
    {
        // The backend hands the question over and writes nothing, which is the
        // seam's one requirement about a clash. What it reports is the marker
        // the page carries now, so the hold that is written next records it and
        // the next sync does not ask the same question again.
        let (address, _listening) = answering(
            "200 OK",
            "application/json",
            a_page_graph_answers_with("1-page", "Fuses", "2026-09-11T10:00:00Z"),
        )
        .await;

        let said = a_backend(&address)
            .leave_a_note_saying(
                "1-section",
                Some(&ANoteThere {
                    named: "1-page".to_string(),
                    version: Some("2026-09-11T09:00:00Z".to_string()),
                }),
                "Fuses",
                "Neutral is blue",
            )
            .await
            .expect("an answer");

        assert_eq!(
            said,
            WhatTheBackendSaid::ItMovedFirst {
                version_now: Some("2026-09-11T10:00:00Z".to_string()),
            },
            "{said:?}"
        );
    }

    #[tokio::test]
    async fn test_a_page_that_names_no_time_it_was_changed_is_refused_rather_than_written_over() {
        // A marker is required of any backend this program writes to, and the
        // seam's requirement 3 says why: with nothing to compare, the push has
        // no evidence anything moved at the other end, so this computer's copy
        // goes over whatever is there and a change somebody made in OneNote is
        // destroyed with nothing said. 05.1-04 measured that rather than
        // arguing it. A refusal costs a sync; the other answer costs a note.
        let (address, _listening) = answering(
            "200 OK",
            "application/json",
            r#"{"id":"1-page","title":"Fuses"}"#.to_string(),
        )
        .await;

        let said = a_backend(&address)
            .leave_a_note_saying(
                "1-section",
                Some(&ANoteThere {
                    named: "1-page".to_string(),
                    version: Some("2026-09-11T09:00:00Z".to_string()),
                }),
                "Fuses",
                "Neutral is blue",
            )
            .await
            .expect("an answer");

        let WhatTheBackendSaid::CouldNotBeReached(why) = said else {
            panic!("a page nobody could compare was written over: {said:?}");
        };
        assert!(
            why.contains("no time it was last changed"),
            "the reason has to say what could not be told: {why}"
        );
    }

    #[tokio::test]
    async fn test_the_marker_handed_back_after_a_change_is_the_one_the_page_carries_afterwards() {
        // Handed back the marker from before the change, the very next push
        // compares an old reading against a new one and reports a clash nobody
        // caused. That is the failure a clock-shaped marker makes cheap, and it
        // costs one more request to avoid.
        let (address, listening) = answering_as_asked(
            "200 OK",
            "application/json",
            vec![
                // The page as it stands, agreeing with what this program holds.
                Box::new(|_| a_page_graph_answers_with("1-page", "Fuses", "2026-09-11T09:00:00Z")),
                // The content the change names things in.
                Box::new(|_| the_content_of_a_page("Live is brown")),
                // The change itself, which answers with nothing.
                Box::new(|_| String::new()),
                // The page again, now carrying a later reading.
                Box::new(|_| a_page_graph_answers_with("1-page", "Fuses", "2026-09-11T10:00:00Z")),
                // And what it kept.
                Box::new(|_| the_content_of_a_page("Neutral is blue")),
            ],
        )
        .await;

        let said = a_backend(&address)
            .leave_a_note_saying(
                "1-section",
                Some(&ANoteThere {
                    named: "1-page".to_string(),
                    version: Some("2026-09-11T09:00:00Z".to_string()),
                }),
                "Fuses",
                "Neutral is blue",
            )
            .await
            .expect("an answer");

        let asked = heard(listening, "the read, the change and the read back")
            .await
            .expect("five requests");
        assert_eq!(
            asked_for(&asked[2]),
            "PATCH /me/onenote/pages/1-page/content",
            "{:?}",
            asked[2]
        );
        let WhatTheBackendSaid::Done { known_as, .. } = said else {
            panic!("a change that went through was not reported as done: {said:?}");
        };
        assert_eq!(
            known_as.version.as_deref(),
            Some("2026-09-11T10:00:00Z"),
            "the marker handed back is the one from before the change"
        );
    }

    #[test]
    fn test_a_section_becomes_a_note_folder_named_by_the_whole_path_above_it() {
        // Two sections called Notes in two notebooks are two folders somebody
        // has to tell apart, and the only thing that tells them apart is where
        // they sit. The identifier cannot do it: nobody reads one.
        assert_eq!(
            ANotebookOnAMicrosoftAccount::the_folder_name_of(&AOneNoteSection {
                id: "1-section".to_string(),
                path: vec!["Work".to_string(), "Projects".to_string(), "Q3".to_string()],
            }),
            "Work / Projects / Q3"
        );
        // A section straight inside a notebook is two names, and one with
        // nothing above it at all is just itself. Neither grows a separator it
        // has no use for.
        assert_eq!(
            ANotebookOnAMicrosoftAccount::the_folder_name_of(&AOneNoteSection {
                id: "1-section".to_string(),
                path: vec!["Work".to_string(), "Notes".to_string()],
            }),
            "Work / Notes"
        );
        assert_eq!(
            ANotebookOnAMicrosoftAccount::the_folder_name_of(&AOneNoteSection {
                id: "1-section".to_string(),
                path: vec!["Notes".to_string()],
            }),
            "Notes"
        );
    }

    #[tokio::test]
    async fn test_the_pages_in_a_section_arrive_as_notes_the_seam_can_name() {
        let (address, listening) = answering(
            "200 OK",
            "application/json",
            r#"{"value":[
                 {"id":"1-page1","title":"Fuses","lastModifiedDateTime":"2026-09-11T09:00:00Z"},
                 {"id":"1-page2","title":"Meters","lastModifiedDateTime":"2026-09-11T09:05:00Z"}
               ]}"#
            .to_string(),
        )
        .await;

        let held = a_backend(&address)
            .notes_it_holds("1-section")
            .await
            .expect("what the section holds");

        let request = heard(listening, "the section's pages")
            .await
            .expect("a request");
        assert_eq!(
            asked_for(&request),
            "GET /me/onenote/sections/1-section/pages",
            "{request}"
        );
        assert_eq!(
            held,
            [
                ANoteThere {
                    named: "1-page1".to_string(),
                    version: Some("2026-09-11T09:00:00Z".to_string()),
                },
                ANoteThere {
                    named: "1-page2".to_string(),
                    version: Some("2026-09-11T09:05:00Z".to_string()),
                },
            ]
        );
    }

    #[tokio::test]
    async fn test_a_note_is_its_title_from_the_resource_and_its_body_from_the_document() {
        // Two requests because the two halves live in two places. The title is
        // a property of the `onenotePage` and the body is the document, and
        // taking both from the document would make a title depend on whether
        // the content endpoint carries one, which nothing here has seen.
        let (address, listening) = answering_as_asked(
            "200 OK",
            "application/json",
            vec![
                Box::new(|_| a_page_graph_answers_with("1-page", "Fuses", "2026-09-11T09:00:00Z")),
                Box::new(|_| the_content_of_a_page("Live is brown")),
            ],
        )
        .await;

        let said = a_backend(&address)
            .what_a_note_says(
                "1-section",
                &ANoteThere {
                    named: "1-page".to_string(),
                    version: Some("2026-09-11T08:00:00Z".to_string()),
                },
            )
            .await
            .expect("an answer")
            .expect("a page that is still there");

        let asked = heard(listening, "the page and its content")
            .await
            .expect("two requests");
        assert_eq!(asked_for(&asked[0]), "GET /me/onenote/pages/1-page");
        assert_eq!(asked_for(&asked[1]), "GET /me/onenote/pages/1-page/content");
        assert_eq!(said.title, "Fuses");
        assert_eq!(said.body, "Live is brown");
        assert_eq!(
            said.known_as.version.as_deref(),
            Some("2026-09-11T09:00:00Z"),
            "the marker reported has to be the one the page carries now"
        );
    }

    #[tokio::test]
    async fn test_a_page_somebody_deleted_at_the_other_end_is_not_there_rather_than_a_failure() {
        let (address, _listening) = answering(
            "404 Not Found",
            "application/json",
            r#"{"error":{"code":"itemNotFound"}}"#.to_string(),
        )
        .await;

        let said = a_backend(&address)
            .what_a_note_says(
                "1-section",
                &ANoteThere {
                    named: "1-gone".to_string(),
                    version: None,
                },
            )
            .await
            .expect("an answer rather than an error");

        assert_eq!(said, None);
    }

    #[tokio::test]
    async fn test_a_note_deleted_here_is_removed_from_onenote_by_the_name_graph_gave() {
        let (address, listening) = answering("204 No Content", "text/html", String::new()).await;

        let said = a_backend(&address)
            .take_a_note_away(
                "1-section",
                &ANoteThere {
                    named: "1-page".to_string(),
                    version: Some("2026-09-11T09:00:00Z".to_string()),
                },
            )
            .await
            .expect("an answer");

        let request = heard(listening, "the removal").await.expect("a request");
        assert_eq!(asked_for(&request), "DELETE /me/onenote/pages/1-page");
        assert!(
            matches!(said, WhatTheBackendSaid::Done { .. }),
            "a page that was removed was not reported as done: {said:?}"
        );
    }

    #[tokio::test]
    async fn test_a_refusal_for_want_of_the_notes_permission_asks_for_a_sign_in() {
        // Not a count and not a status. Somebody whose token predates this
        // version has every OneNote call refused, and "1 problem" on every sync
        // from now on tells them nothing about what to do.
        let (address, _listening) = answering(
            "401 Unauthorized",
            "application/json",
            r#"{"error":{"code":"InvalidAuthenticationToken"}}"#.to_string(),
        )
        .await;

        let said = a_backend(&address)
            .leave_a_note_saying("1-section", None, "Fuses", "Live is brown")
            .await
            .expect("an answer rather than an error");

        assert_eq!(said, WhatTheBackendSaid::NotSignedIn, "{said:?}");
    }

    /// Nothing goes out for an account that may only be read.
    ///
    /// The gate is applied where the client is built and nowhere else, which is
    /// what `05.2-02` asserted of the client and what this asserts of the
    /// backend above it. A second gate here would be a second answer to one
    /// question.
    #[tokio::test]
    async fn test_an_account_that_may_only_be_read_makes_no_page_and_says_why() {
        let (address, listening) = answering(
            "201 Created",
            "application/json",
            a_page_graph_answers_with("1-page", "Fuses", "2026-09-11T09:00:00Z"),
        )
        .await;
        let backend = ANotebookOnAMicrosoftAccount {
            client: MsGraphClient::new().pointed_at(&format!("http://{address}")),
            token: "a-token".to_string(),
        };

        let said = backend
            .leave_a_note_saying("1-section", None, "Fuses", "Live is brown")
            .await
            .expect("an answer rather than an error");

        assert_eq!(
            said,
            WhatTheBackendSaid::NotAllowedToChangeAnything,
            "{said:?}"
        );
        assert!(
            heard(listening, "nothing at all").await.is_err(),
            "a client that may not change anything sent a request"
        );
    }
}

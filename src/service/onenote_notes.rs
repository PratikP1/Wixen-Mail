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
    ANoteAsItStands, ANoteThere, NotesService, WhatTheBackendSaid,
};
use crate::common::Result;
use crate::service::microsoft_graph::{AOneNoteSection, MsGraphClient};
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
}

impl NotesService for ANotebookOnAMicrosoftAccount {
    async fn notes_it_holds(&self, _container: &str) -> Result<Vec<ANoteThere>> {
        Ok(Vec::new())
    }

    async fn what_a_note_says(
        &self,
        _container: &str,
        _known_as: &ANoteThere,
    ) -> Result<Option<ANoteAsItStands>> {
        Ok(None)
    }

    async fn leave_a_note_saying(
        &self,
        _container: &str,
        _known_as: Option<&ANoteThere>,
        _title: &str,
        _body: &str,
    ) -> Result<WhatTheBackendSaid> {
        Ok(WhatTheBackendSaid::CouldNotBeReached(
            "no page was written".to_string(),
        ))
    }

    async fn take_a_note_away(
        &self,
        _container: &str,
        _known_as: &ANoteThere,
    ) -> Result<WhatTheBackendSaid> {
        Ok(WhatTheBackendSaid::CouldNotBeReached(
            "no page was removed".to_string(),
        ))
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
        // Requirement 2 of the seam's section on bytes. The answer comes from
        // the page's own copy, so a service that stopped reshaping a note, or
        // started reshaping a different part of one, would be reported as it
        // really is rather than as this program believes it to be.
        let (address, _listening) = answering_as_asked(
            "200 OK",
            "application/json",
            vec![
                Box::new(|_| a_page_graph_answers_with("1-page", "Fuses", "2026-09-11T09:00:00Z")),
                Box::new(|_| the_content_of_a_page("Live is brown")),
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
            kept.body, "Live is brown",
            "what the page kept has to be what the page says, not what was sent"
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

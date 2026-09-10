//! A calendar server's journal entries, behind the notes seam.
//!
//! The first implementation of
//! [`crate::application::notes_backend::NotesService`]. It is thin on purpose:
//! everything it decides is decided in
//! [`crate::application::notes_sync`], which knows no backend, and everything
//! it reads or writes as a document is [`crate::service::note_document`],
//! which touches no network. What is left here is the four requests and the
//! turning of a server's answer into one of the words the seam allows.
//!
//! # It reuses the sign-in of a calendar somebody already added
//!
//! A calendar server keeps journal entries in the same place it keeps
//! calendars, under the same sign-in. So this asks
//! [`crate::service::caldav::sign_in`] for the calendar the account already
//! has, and nobody types a second address or a second password. That is one
//! credential owner rather than two, which is what `CLAUDE.md` asks for: the
//! code that erases secrets on uninstall names the same entries as the code
//! that wrote them, and there is still only one naming.
//!
//! # Nothing here has ever run against a real server
//!
//! Every request below is written from the standard and from what this
//! repository already does. No account, no calendar server and no journal
//! collection has been used with this program, and each unknown has its own
//! entry in `.planning/WINDOWS.md` rather than one entry covering all of them.

use crate::application::notes_backend::{
    ANoteAsItStands, ANoteThere, NotesService, WhatTheBackendSaid,
};
use crate::common::{Error, Result};
use crate::service::caldav::{CalDavClient, sign_in};
use crate::service::note_document;

/// One account's journal entries at the calendar server it already uses.
pub struct AJournalOnACalendarServer {
    client: CalDavClient,
    user_name: String,
    password: String,
}

impl AJournalOnACalendarServer {
    /// A journal backend for this account, using this calendar's sign-in.
    ///
    /// `None` when there is no whole sign-in stored for that calendar, which is
    /// the state an account is in before anybody has typed one and after
    /// somebody has removed it. Half a sign-in is not a sign-in, and the
    /// credential store's own reader already answers it that way.
    ///
    /// The client is built through [`CalDavClient::for_account`], which is
    /// where the Allowed Changes setting is applied, the same way the four
    /// services that already exist here apply it. Nothing below asks the
    /// setting again: a client that may not change anything refuses the
    /// request before it reaches the network, and that refusal arrives here as
    /// one this code turns into words.
    pub fn for_account(account_id: &str, calendar_id: &str) -> Option<Self> {
        let (user_name, password) = sign_in::load(calendar_id)?;
        Some(Self {
            client: CalDavClient::for_account(account_id),
            user_name,
            password,
        })
    }

    /// One that may change things, for tests only.
    ///
    /// [`Self::for_account`] reads the settings really stored on whichever
    /// machine is running and the sign-in really in that machine's credential
    /// store, so a test built on it would pass or fail depending on whose
    /// computer ran it. [`CalDavClient::allowed_to_change_things`] exists for
    /// the same reason and says the same thing.
    #[cfg(test)]
    fn allowed_to_change_things() -> Self {
        Self {
            client: CalDavClient::allowed_to_change_things(),
            user_name: "sam".to_string(),
            password: "secret".to_string(),
        }
    }

    /// What a failed request means, in the words the seam allows.
    ///
    /// A refusal by the setting and a refusal by the server are different
    /// things to somebody: one is fixed on the settings screen and one is fixed
    /// by signing in again, and a single "it did not work" tells them neither.
    fn what_that_means(error: Error) -> WhatTheBackendSaid {
        if crate::service::outward::was_refused_by_the_gate(&error) {
            return WhatTheBackendSaid::NotAllowedToChangeAnything;
        }
        match error {
            Error::Api { status: 401, .. } | Error::Api { status: 403, .. } => {
                WhatTheBackendSaid::NotSignedIn
            }
            Error::Api { status: 404, .. } | Error::Api { status: 410, .. } => {
                WhatTheBackendSaid::ItIsNotThere
            }
            // The string is for the log. Nothing reads it out.
            other => WhatTheBackendSaid::CouldNotBeReached(other.to_string()),
        }
    }

    /// The address a note this computer made is written to.
    ///
    /// Inside the collection and never beside it, with the identifier escaped:
    /// whatever made the note chose it, and one holding a space breaks the
    /// request line while one holding a hash truncates the address at the
    /// fragment, so the write lands somewhere nobody named.
    /// [`CalDavClient::create_event`] does the same and its comment says the
    /// same, because it is the same mistake.
    fn where_a_new_note_goes(container: &str, uid: &str) -> String {
        format!(
            "{}/{}.ics",
            container.trim_end_matches('/'),
            crate::service::outward::in_a_path(uid)
        )
    }
}

impl NotesService for AJournalOnACalendarServer {
    async fn notes_it_holds(&self, container: &str) -> Result<Vec<ANoteThere>> {
        Ok(self
            .client
            .journal_entries_in(container, &self.user_name, &self.password)
            .await?
            .into_iter()
            .map(|(at, version)| ANoteThere { named: at, version })
            .collect())
    }

    async fn what_a_note_says(
        &self,
        _container: &str,
        known_as: &ANoteThere,
    ) -> Result<Option<ANoteAsItStands>> {
        let held = match self
            .client
            .document_at(
                &known_as.named,
                &self.user_name,
                &self.password,
                "Reading a note",
            )
            .await
        {
            Ok(held) => held,
            // Gone between the listing and the reading, which is ordinary
            // rather than wrong: somebody deleted it at the other end while
            // this sync was running.
            Err(Error::Api { status: 404, .. }) | Err(Error::Api { status: 410, .. }) => {
                return Ok(None);
            }
            Err(e) => return Err(e),
        };
        // A document that is not a note is not a note. Refused with a reason
        // rather than turned into a half-filled one, which is the whole of what
        // `NotANote` is for, and the reason goes to the log rather than to
        // somebody's ears.
        let note = match note_document::the_note_in(&held.document) {
            Ok(note) => note,
            Err(why) => {
                return Err(Error::Other(format!(
                    "A document at {} could not be read as a note: {why:?}",
                    known_as.named
                )));
            }
        };
        Ok(Some(ANoteAsItStands {
            known_as: ANoteThere {
                named: known_as.named.clone(),
                version: held.tag,
            },
            title: note.title,
            body: note.body,
        }))
    }

    async fn leave_a_note_saying(
        &self,
        container: &str,
        known_as: Option<&ANoteThere>,
        title: &str,
        body: &str,
    ) -> Result<WhatTheBackendSaid> {
        let Some(there) = known_as else {
            // A note this backend has never held. Its identifier is minted
            // here, which is the one place this program is allowed to make one:
            // a name the backend has never given is not a name to look up.
            let uid = uuid::Uuid::new_v4().to_string();
            let at = Self::where_a_new_note_goes(container, &uid);
            let document = note_document::a_document_saying(&uid, title, body);
            return Ok(
                match self
                    .client
                    .write_a_document(
                        &at,
                        &self.user_name,
                        &self.password,
                        &document,
                        None,
                        "add a note to the calendar server",
                    )
                    .await
                {
                    Ok(version) => WhatTheBackendSaid::Done(ANoteThere { named: at, version }),
                    Err(e) => Self::what_that_means(e),
                },
            );
        };

        // Read before writing, and not only to build the new document. The
        // version that comes back is the version this moment, and comparing it
        // against the one in hand is how the disagreement is found before
        // anything is written over. Sending the write and reading a 412 would
        // find it too, and would find it after this program had already decided
        // what the document should say.
        let held = match self
            .client
            .document_at(
                &there.named,
                &self.user_name,
                &self.password,
                "Reading a note",
            )
            .await
        {
            Ok(held) => held,
            Err(e) => return Ok(Self::what_that_means(e)),
        };
        if held.tag.is_some() && held.tag != there.version {
            // Nothing is written. Which copy is kept is not this backend's
            // decision, and `conflict_choice` is where it is asked.
            return Ok(WhatTheBackendSaid::ItMovedFirst {
                version_now: held.tag,
            });
        }

        // Only the two properties this program owns are changed. Everything
        // else the server had, including whatever it has that this program has
        // never modelled, is left exactly where it was.
        let document =
            note_document::the_document_with_the_note_changed(&held.document, title, body);
        Ok(
            match self
                .client
                .write_a_document(
                    &there.named,
                    &self.user_name,
                    &self.password,
                    &document,
                    there.version.as_deref(),
                    "change a note on the calendar server",
                )
                .await
            {
                Ok(version) => WhatTheBackendSaid::Done(ANoteThere {
                    named: there.named.clone(),
                    version,
                }),
                Err(e) => Self::what_that_means(e),
            },
        )
    }

    async fn take_a_note_away(
        &self,
        _container: &str,
        known_as: &ANoteThere,
    ) -> Result<WhatTheBackendSaid> {
        Ok(
            match self
                .client
                .remove_a_document(
                    &known_as.named,
                    &self.user_name,
                    &self.password,
                    "remove a note from the calendar server",
                )
                .await
            {
                Ok(()) => WhatTheBackendSaid::Done(known_as.clone()),
                Err(e) => Self::what_that_means(e),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::answering::{answering, asked_for, heard};

    /// The collection this account's journal entries live in.
    fn a_collection_at(address: std::net::SocketAddr) -> String {
        format!("http://{address}/dav/sam/notes/")
    }

    /// What a server answers a listing with: itself, then one entry.
    fn a_listing_holding_one_note() -> String {
        "<?xml version=\"1.0\"?>\n\
         <d:multistatus xmlns:d=\"DAV:\">\n\
           <d:response>\n\
             <d:href>/dav/sam/notes/</d:href>\n\
             <d:propstat><d:prop>\n\
               <d:resourcetype><d:collection/></d:resourcetype>\n\
             </d:prop></d:propstat>\n\
           </d:response>\n\
           <d:response>\n\
             <d:href>/dav/sam/notes/n-1.ics</d:href>\n\
             <d:propstat><d:prop>\n\
               <d:getetag>\"v1\"</d:getetag>\n\
             </d:prop></d:propstat>\n\
           </d:response>\n\
         </d:multistatus>"
            .to_string()
    }

    #[tokio::test]
    async fn test_a_new_note_is_written_inside_the_collection_rather_than_beside_it() {
        // The address was built by sticking the identifier onto the end of the
        // collection with no separator once, for an event, and the write landed
        // on a sibling of the calendar rather than in it. The same mistake is
        // available here and this is the fixture that would see it.
        let (address, listening) = answering("201 Created", "text/calendar", String::new()).await;
        let collection = a_collection_at(address);

        let said = AJournalOnACalendarServer::allowed_to_change_things()
            .leave_a_note_saying(&collection, None, "Wiring colours", "Brown is live")
            .await
            .expect("a write through a client allowed to make one");

        let request = heard(listening, "a write").await.expect("the request");
        let asked = asked_for(&request);
        assert!(asked.starts_with("PUT /dav/sam/notes/"), "{asked}");
        assert!(asked.ends_with(".ics"), "{asked}");
        assert!(
            request.contains("BEGIN:VJOURNAL"),
            "what was sent is not a journal entry:\n{request}"
        );
        assert!(request.contains("SUMMARY:Wiring colours"), "{request}");
        // Nothing may be there already. A create that replaced whatever it
        // found would write over a stranger's note on an identifier collision,
        // silently.
        assert!(request.contains("if-none-match: *"), "{request}");
        assert!(
            matches!(said, WhatTheBackendSaid::Done(_)),
            "{said:?} for a write the server took"
        );
    }

    #[tokio::test]
    async fn test_a_note_taken_away_is_asked_for_by_its_own_address() {
        let (address, listening) = answering("204 No Content", "text/plain", String::new()).await;
        let at = format!("http://{address}/dav/sam/notes/n-1.ics");

        let said = AJournalOnACalendarServer::allowed_to_change_things()
            .take_a_note_away(
                &a_collection_at(address),
                &ANoteThere {
                    named: at,
                    version: Some("\"v1\"".to_string()),
                },
            )
            .await
            .expect("a removal through a client allowed to make one");

        let request = heard(listening, "a removal").await.expect("the request");
        assert_eq!(asked_for(&request), "DELETE /dav/sam/notes/n-1.ics");
        // No version named, deliberately: somebody asked for the note to go,
        // and a version that had moved on would make the removal fail for ever.
        assert!(
            !request.to_ascii_lowercase().contains("if-match:"),
            "{request}"
        );
        assert!(matches!(said, WhatTheBackendSaid::Done(_)), "{said:?}");
    }

    #[tokio::test]
    async fn test_the_collection_itself_is_not_read_back_as_one_of_the_notes_in_it() {
        // A server answers `Depth: 1` with the collection first and then what
        // is in it. Reading that first block as a note asks for a whole
        // calendar as though it were a document, on every sync.
        let (address, _listening) = answering(
            "207 Multi-Status",
            "application/xml",
            a_listing_holding_one_note(),
        )
        .await;

        let held = AJournalOnACalendarServer::allowed_to_change_things()
            .notes_it_holds(&a_collection_at(address))
            .await
            .expect("the listing");

        assert_eq!(held.len(), 1, "{held:?}");
        assert!(
            held[0].named.ends_with("/dav/sam/notes/n-1.ics"),
            "{held:?}"
        );
        assert_eq!(held[0].version.as_deref(), Some("\"v1\""));
    }

    #[tokio::test]
    async fn test_a_note_the_server_no_longer_holds_is_not_a_failure() {
        // Somebody deleted it at the other end between the listing and the
        // reading. Ordinary rather than wrong, and reporting it as a failure
        // would put a problem on the status line on every sync until the
        // listing caught up.
        let (address, _listening) = answering("404 Not Found", "text/plain", String::new()).await;

        let said = AJournalOnACalendarServer::allowed_to_change_things()
            .what_a_note_says(
                &a_collection_at(address),
                &ANoteThere {
                    named: format!("http://{address}/dav/sam/notes/n-1.ics"),
                    version: None,
                },
            )
            .await
            .expect("a read that found nothing");

        assert_eq!(said, None);
    }

    #[tokio::test]
    async fn test_a_document_that_is_not_a_note_is_refused_rather_than_half_read() {
        let (address, _listening) = answering(
            "200 OK",
            "text/calendar",
            "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:e-1\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
                .to_string(),
        )
        .await;

        let said = AJournalOnACalendarServer::allowed_to_change_things()
            .what_a_note_says(
                &a_collection_at(address),
                &ANoteThere {
                    named: format!("http://{address}/dav/sam/notes/n-1.ics"),
                    version: None,
                },
            )
            .await;

        assert!(
            said.is_err(),
            "a calendar event was read back as a half-filled note"
        );
    }
}

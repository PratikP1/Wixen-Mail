//! Notes out to a backend and back, through a seam that names no backend.
//!
//! Everything here is written against [`crate::application::notes_backend`]'s
//! trait and knows nothing about what is behind it. That is not tidiness: it
//! is the whole claim `05.1-03` makes and the thing `05.1-04` is written to
//! test, by putting a second implementation behind the same seam and finding
//! out what had really been shaped around the first.
//!
//! ```text
//! grep -rn "caldav\|CalDav\|VJOURNAL\|OneNote" src/application/notes_sync.rs
//! ```
//!
//! That returning nothing is an acceptance criterion of the plan that wrote
//! this file, and it is quoted here so the next person can re-run it rather
//! than trust it.
//!
//! # What a sync does, in order
//!
//! The push goes first and the read second, which is the order every other
//! sync here runs in and the order [`crate::application::deletions`] is
//! written about: a deletion this computer owes has to leave before a read can
//! be told the thing is still there.
//!
//! # What is not decided here
//!
//! Which copy wins. [`crate::application::contacts_sync::whose_copy_wins`]
//! answers that by comparing version markers, and a second answer written here
//! would disagree with it the first time either changed. When both copies have
//! moved this holds them through [`crate::application::conflict_choice`] and
//! stops, which is what the seam's contract requires of every backend and of
//! everything driving one.
//!
//! # One container per account, which is a limit and is said rather than hidden
//!
//! A sync is given one container. Note folders on this computer are not
//! mirrored at the backend and nothing here pretends they are: a note arriving
//! from the backend is filed in the account's first note folder. Folders that
//! mean something at both ends is a decision somebody has to make about what a
//! folder *is* at each backend, and this plan does not make it.

use crate::application::notes_backend::NotesService;
use crate::application::summing_up::SummingUp;
use crate::common::Result;
use crate::data::message_cache::MessageCache;

/// What one notes sync did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct NoteSyncResult {
    /// Notes written here because they were new or had changed there.
    ///
    /// Not the number seen. A sync that rewrites every note every time can
    /// only report the size of the container, which tells nobody whether
    /// anything happened.
    pub stored: usize,
    /// Notes the backend had not touched since the last sync.
    pub unchanged: usize,
    /// Changes made here that reached the backend.
    pub sent: usize,
    /// Changes still waiting because this program is not allowed to change
    /// anything on this account.
    ///
    /// Counted rather than reported as a failure. Nothing went wrong: the
    /// change is waiting on a setting, and one error per waiting note on every
    /// sync from now on is how a warning somebody needs stops being read.
    pub waiting_on_the_setting: usize,
    /// Nobody is signed in to the backend.
    ///
    /// Said rather than counted, for the reason
    /// [`crate::application::tasks_sync`]'s own field gives: an account that
    /// keeps being refused is "1 problem" on every sync forever, with nothing
    /// saying what to do about it.
    pub needs_sign_in: bool,
    /// Said rather than swallowed. A container that could not be read is a
    /// gap, and reporting a clean sync over it is how somebody comes to trust
    /// a list that is missing half of itself.
    pub errors: Vec<String>,
}

impl NoteSyncResult {
    /// What the status line says afterwards.
    pub fn summary(&self) -> String {
        let mut said = SummingUp::opening(format!("{} stored", how_many(self.stored, "note")));
        if self.unchanged > 0 {
            said.count(format!("{} unchanged", self.unchanged));
        }
        if self.sent > 0 {
            said.count(format!("{} of yours sent", self.sent));
        }
        if !self.errors.is_empty() {
            // The count, not the text. The messages are in the log, and a
            // status line that grows with the number of failures pushes
            // everything else off it.
            said.count(how_many(self.errors.len(), "problem"));
        }
        if self.waiting_on_the_setting > 0 {
            // The task, calendar and contacts syncs all say this, so it is
            // said in one place. Its own comment records that two copies of it
            // drifted and only one was corrected.
            said.sentence(crate::application::allowed::changes_waiting_here(
                self.waiting_on_the_setting,
            ));
        }
        said.spoken()
    }
}

/// A count and the thing it counts, in one place.
///
/// The same routine the calendar and task syncs use, so "1 note" and
/// "2 notes" are decided once rather than by each clause writing its own `s`.
fn how_many(count: usize, thing: &str) -> String {
    crate::service::caldav::how_many(count, thing)
}

/// Send what is waiting, then take what has arrived.
///
/// `container` is opaque and is handed straight back to the backend. Nothing
/// here parses it, splits it or builds one, which is a requirement of the seam
/// rather than a habit: a CalDAV journal collection is one address and a
/// OneNote page lives four levels down, so anything that took a container
/// apart would be reading one backend's addressing.
pub async fn sync_notes<S: NotesService>(
    cache: &MessageCache,
    service: &S,
    account_id: &str,
    container: &str,
) -> Result<NoteSyncResult> {
    // Not written yet. This is the RED half of `05.1-03` task 1, and the
    // commit that carries it names the tests it leaves failing.
    let _ = (cache, service, account_id, container);
    Ok(NoteSyncResult::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::notes_backend::{ANoteAsItStands, ANoteThere, WhatTheBackendSaid};
    use crate::common::temp_home::TempHome;
    use crate::common::{Error, Result as OurResult};
    use crate::data::message_cache::{NoteBody, NoteEntry, NoteFolderEntry};
    use std::sync::Mutex;

    const ACCOUNT: &str = "acct-1";
    const CONTAINER: &str = "https://example.test/dav/journals/";

    fn a_store() -> TempHome<MessageCache> {
        TempHome::named("wixen_notes_sync_", |dir| {
            let cache = MessageCache::new(dir.to_path_buf(), None).expect("a store");
            cache
                .save_note_folder(&NoteFolderEntry {
                    id: "folder-1".to_string(),
                    account_id: ACCOUNT.to_string(),
                    name: "General".to_string(),
                    display_order: 0,
                    created_at: "2026-01-01".to_string(),
                })
                .expect("a folder for notes");
            cache
        })
    }

    /// A note on this computer, waiting or not.
    fn a_note(id: &str, title: &str, body: &str) -> NoteEntry {
        NoteEntry {
            id: id.to_string(),
            account_id: ACCOUNT.to_string(),
            folder_id: Some("folder-1".to_string()),
            title: title.to_string(),
            body: body.to_string(),
            format: NoteBody::AsTyped,
            pinned: false,
            created_at: "2026-01-01".to_string(),
            updated_at: "2026-01-01".to_string(),
            pending: false,
            known_as: None,
            known_version: None,
        }
    }

    /// What this service makes of a change sent to it.
    ///
    /// The four answers the push has to tell apart, and refusing is the
    /// default so that a test which never meant to send anything fails loudly
    /// when a write is reached by accident.
    #[derive(Default, Clone, Copy, PartialEq, Eq)]
    enum Writes {
        /// Refused, for an ordinary reason nobody can act on.
        #[default]
        NothingIsSent,
        /// Taken.
        Accepted,
        /// Refused because of what this program is allowed to change, which is
        /// a setting rather than a fault.
        RefusedByTheSetting,
        /// Nobody is signed in.
        NotSignedIn,
    }

    /// A notes backend that answers from a script rather than a socket.
    ///
    /// It can refuse, it can answer an empty container, and it can say its own
    /// copy has moved on. A fake that only ever succeeds tests the happy path
    /// and nothing else, which is the shape this project's own observation log
    /// records as the first thing a fake gets wrong.
    #[derive(Default)]
    struct Scripted {
        /// What the backend holds, in the order it lists them.
        holds: Vec<ANoteAsItStands>,
        /// Names this service refuses to read, so a test can be a backend that
        /// lost a note between the listing and the reading.
        will_not_say: Vec<String>,
        /// Whether listing the container fails outright.
        cannot_be_listed: bool,
        /// What this service makes of a change sent to it.
        writes: Writes,
        /// Every note this service was asked to write, in the order asked.
        ///
        /// The only sound way to assert that a push did not happen. Reading it
        /// off `NoteSyncResult` cannot do it: a write refused by the setting is
        /// counted rather than pushed into `errors`, so an empty `errors` is
        /// true whether the call was made or not, and an assertion whose
        /// emptiness has two causes is not an assertion about either.
        ///
        /// Recorded before the answer rather than after, because the question
        /// is whether the call reached the backend at all and a refusal is
        /// still a call that reached it.
        written: Mutex<Vec<String>>,
        /// Every note this service was asked to remove, in the order asked.
        taken_away: Mutex<Vec<String>>,
    }

    impl Scripted {
        /// The marker the backend hands out for a copy it has just written.
        const A_NEW_MARKER: &'static str = "v2";
    }

    impl NotesService for Scripted {
        async fn notes_it_holds(&self, _container: &str) -> OurResult<Vec<ANoteThere>> {
            if self.cannot_be_listed {
                return Err(Error::Network(
                    "the container could not be read".to_string(),
                ));
            }
            Ok(self
                .holds
                .iter()
                .map(|note| note.known_as.clone())
                .collect())
        }

        async fn what_a_note_says(
            &self,
            _container: &str,
            known_as: &ANoteThere,
        ) -> OurResult<Option<ANoteAsItStands>> {
            if self.will_not_say.contains(&known_as.named) {
                return Ok(None);
            }
            Ok(self
                .holds
                .iter()
                .find(|note| note.known_as.named == known_as.named)
                .cloned())
        }

        async fn leave_a_note_saying(
            &self,
            _container: &str,
            known_as: Option<&ANoteThere>,
            title: &str,
            _body: &str,
        ) -> OurResult<WhatTheBackendSaid> {
            self.written
                .lock()
                .expect("the record of what was written")
                .push(title.to_string());
            Ok(match self.writes {
                Writes::NothingIsSent => {
                    WhatTheBackendSaid::CouldNotBeReached("the backend said no".to_string())
                }
                Writes::Accepted => WhatTheBackendSaid::Done(ANoteThere {
                    named: known_as
                        .map(|there| there.named.clone())
                        .unwrap_or_else(|| format!("made-for-{title}")),
                    version: Some(Self::A_NEW_MARKER.to_string()),
                }),
                Writes::RefusedByTheSetting => WhatTheBackendSaid::NotAllowedToChangeAnything,
                Writes::NotSignedIn => WhatTheBackendSaid::NotSignedIn,
            })
        }

        async fn take_a_note_away(
            &self,
            _container: &str,
            known_as: &ANoteThere,
        ) -> OurResult<WhatTheBackendSaid> {
            self.taken_away
                .lock()
                .expect("the record of what was removed")
                .push(known_as.named.clone());
            Ok(match self.writes {
                Writes::NothingIsSent => {
                    WhatTheBackendSaid::CouldNotBeReached("the backend said no".to_string())
                }
                Writes::Accepted => WhatTheBackendSaid::Done(known_as.clone()),
                Writes::RefusedByTheSetting => WhatTheBackendSaid::NotAllowedToChangeAnything,
                Writes::NotSignedIn => WhatTheBackendSaid::NotSignedIn,
            })
        }
    }

    fn run<F: std::future::Future>(work: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("a runtime")
            .block_on(work)
    }

    #[test]
    fn test_a_note_waiting_here_is_offered_to_the_backend_and_then_is_not_offered_again() {
        // Asserted on what the backend was asked to do and on the row left
        // behind, not on a counter the code under test increments. A count is
        // green against a push that counts without pushing.
        let cache = a_store();
        let mut note = a_note("n1", "Wiring colours", "Brown is live");
        note.pending = true;
        cache.save_note(&note).expect("a note waiting to be sent");
        let service = Scripted {
            writes: Writes::Accepted,
            ..Scripted::default()
        };

        let first = run(sync_notes(&cache, &service, ACCOUNT, CONTAINER)).expect("a sync");

        assert_eq!(
            *service.written.lock().expect("what was written"),
            ["Wiring colours"],
            "the note waiting here was not offered to the backend"
        );
        assert_eq!(first.sent, 1, "{first:?}");
        let after = cache.get_note("n1").expect("the note").expect("the note");
        assert!(
            !after.pending,
            "the note is still waiting after it was taken"
        );
        assert_eq!(after.known_as.as_deref(), Some("made-for-Wiring colours"));
        assert_eq!(after.known_version.as_deref(), Some("v2"));

        let second = run(sync_notes(&cache, &service, ACCOUNT, CONTAINER)).expect("a second sync");
        assert_eq!(
            service.written.lock().expect("what was written").len(),
            1,
            "the note was offered again after the backend had taken it"
        );
        assert_eq!(second.sent, 0, "{second:?}");
    }

    #[test]
    fn test_a_note_the_setting_held_stays_here_and_is_counted_rather_than_reported_as_a_failure() {
        let cache = a_store();
        let mut note = a_note("n1", "Wiring colours", "Brown is live");
        note.pending = true;
        cache.save_note(&note).expect("a note waiting to be sent");
        let service = Scripted {
            writes: Writes::RefusedByTheSetting,
            ..Scripted::default()
        };

        let result = run(sync_notes(&cache, &service, ACCOUNT, CONTAINER)).expect("a sync");

        assert_eq!(result.waiting_on_the_setting, 1, "{result:?}");
        assert_eq!(result.sent, 0, "{result:?}");
        assert!(
            result.errors.is_empty(),
            "a setting holding a change was reported as a failure: {result:?}"
        );
        let after = cache.get_note("n1").expect("the note").expect("the note");
        assert!(
            after.pending,
            "the note stopped waiting although nothing was sent, so turning the \
             setting on would never send it"
        );
    }

    #[test]
    fn test_the_summary_names_the_setting_only_when_something_was_really_held() {
        // A summary that always names the setting satisfies the test above and
        // says the wrong thing on every clean sync, so both cases are driven.
        let held = NoteSyncResult {
            waiting_on_the_setting: 1,
            ..NoteSyncResult::default()
        };
        assert_eq!(
            held.summary(),
            format!(
                "0 notes stored. {}.",
                crate::application::allowed::changes_waiting_here(1)
            ),
            "the sentence the other three syncs say was not the one said here"
        );

        let clean = NoteSyncResult {
            stored: 2,
            ..NoteSyncResult::default()
        };
        assert!(
            !clean.summary().contains("waiting here"),
            "a sync that held nothing named the setting anyway: {}",
            clean.summary()
        );
    }

    #[test]
    fn test_a_note_only_the_backend_changed_arrives_and_replaces_what_is_here() {
        let cache = a_store();
        let mut note = a_note("n1", "Old title", "Old words");
        note.known_as = Some("there-1".to_string());
        note.known_version = Some("v1".to_string());
        cache.save_note(&note).expect("a note already synced");
        let service = Scripted {
            holds: vec![ANoteAsItStands {
                known_as: ANoteThere {
                    named: "there-1".to_string(),
                    version: Some("v9".to_string()),
                },
                title: "New title".to_string(),
                body: "New words".to_string(),
            }],
            ..Scripted::default()
        };

        let result = run(sync_notes(&cache, &service, ACCOUNT, CONTAINER)).expect("a sync");

        assert_eq!(result.stored, 1, "{result:?}");
        let after = cache.get_note("n1").expect("the note").expect("the note");
        assert_eq!(after.title, "New title");
        assert_eq!(after.body, "New words");
        assert_eq!(after.known_version.as_deref(), Some("v9"));
    }

    #[test]
    fn test_a_note_the_backend_has_not_touched_is_left_alone() {
        let cache = a_store();
        let mut note = a_note("n1", "Wiring colours", "Brown is live");
        note.known_as = Some("there-1".to_string());
        note.known_version = Some("v1".to_string());
        cache.save_note(&note).expect("a note already synced");
        let service = Scripted {
            holds: vec![ANoteAsItStands {
                known_as: ANoteThere {
                    named: "there-1".to_string(),
                    version: Some("v1".to_string()),
                },
                title: "Wiring colours".to_string(),
                body: "Brown is live".to_string(),
            }],
            ..Scripted::default()
        };

        let result = run(sync_notes(&cache, &service, ACCOUNT, CONTAINER)).expect("a sync");

        assert_eq!(result.unchanged, 1, "{result:?}");
        assert_eq!(result.stored, 0, "{result:?}");
    }

    #[test]
    fn test_a_note_the_backend_has_never_heard_of_arrives_as_a_new_note() {
        let cache = a_store();
        let service = Scripted {
            holds: vec![ANoteAsItStands {
                known_as: ANoteThere {
                    named: "there-1".to_string(),
                    version: Some("v1".to_string()),
                },
                title: "Made elsewhere".to_string(),
                body: "Typed on another machine".to_string(),
            }],
            ..Scripted::default()
        };

        let result = run(sync_notes(&cache, &service, ACCOUNT, CONTAINER)).expect("a sync");

        assert_eq!(result.stored, 1, "{result:?}");
        let here = cache
            .get_all_notes_for_account(ACCOUNT)
            .expect("what is here");
        assert_eq!(here.len(), 1, "{here:?}");
        assert_eq!(here[0].title, "Made elsewhere");
        assert_eq!(here[0].known_as.as_deref(), Some("there-1"));
        assert!(
            !here[0].pending,
            "a note that arrived from the backend is marked as waiting to be \
             sent back to it, so the next sync writes it out again"
        );
    }

    #[test]
    fn test_a_note_sent_in_this_sync_is_not_overwritten_by_the_read_that_follows_it() {
        // The push runs before the read, so the backend's answer to the read
        // is the copy the push just wrote. Taking it back down is harmless
        // when the backend echoes what it was given and is a lost edit the
        // moment it does not, which is why the marker rather than the words
        // decides.
        let cache = a_store();
        let mut note = a_note("n1", "Wiring colours", "Brown is live");
        note.pending = true;
        note.known_as = Some("there-1".to_string());
        note.known_version = Some("v1".to_string());
        cache.save_note(&note).expect("a note waiting to be sent");
        let service = Scripted {
            writes: Writes::Accepted,
            holds: vec![ANoteAsItStands {
                known_as: ANoteThere {
                    named: "there-1".to_string(),
                    version: Some(Scripted::A_NEW_MARKER.to_string()),
                },
                title: "Wiring colours".to_string(),
                body: "Brown is live".to_string(),
            }],
            ..Scripted::default()
        };

        let result = run(sync_notes(&cache, &service, ACCOUNT, CONTAINER)).expect("a sync");

        assert_eq!(result.sent, 1, "{result:?}");
        assert_eq!(
            result.stored, 0,
            "the read wrote back down the copy the push had just sent: {result:?}"
        );
        assert_eq!(result.unchanged, 1, "{result:?}");
    }

    #[test]
    fn test_a_container_that_cannot_be_read_is_said_rather_than_reported_as_empty() {
        let cache = a_store();
        let service = Scripted {
            cannot_be_listed: true,
            ..Scripted::default()
        };

        let result = run(sync_notes(&cache, &service, ACCOUNT, CONTAINER)).expect("a sync");

        assert_eq!(
            result.errors.len(),
            1,
            "a container that could not be read was reported as a clean sync: {result:?}"
        );
    }

    #[test]
    fn test_a_note_the_backend_lost_between_the_listing_and_the_reading_is_not_a_failure() {
        // Somebody deleted it at the other end while this sync was running.
        // Ordinary rather than wrong, and a note nobody here has ever seen is
        // simply not written down.
        let cache = a_store();
        let service = Scripted {
            holds: vec![ANoteAsItStands {
                known_as: ANoteThere {
                    named: "there-1".to_string(),
                    version: Some("v1".to_string()),
                },
                title: "Going".to_string(),
                body: "Gone".to_string(),
            }],
            will_not_say: vec!["there-1".to_string()],
            ..Scripted::default()
        };

        let result = run(sync_notes(&cache, &service, ACCOUNT, CONTAINER)).expect("a sync");

        assert!(result.errors.is_empty(), "{result:?}");
        assert_eq!(result.stored, 0, "{result:?}");
    }

    #[test]
    fn test_nobody_signed_in_is_said_in_words_rather_than_counted_as_a_problem() {
        let cache = a_store();
        let mut note = a_note("n1", "Wiring colours", "Brown is live");
        note.pending = true;
        cache.save_note(&note).expect("a note waiting to be sent");
        let service = Scripted {
            writes: Writes::NotSignedIn,
            ..Scripted::default()
        };

        let result = run(sync_notes(&cache, &service, ACCOUNT, CONTAINER)).expect("a sync");

        assert!(result.needs_sign_in, "{result:?}");
        assert!(result.errors.is_empty(), "{result:?}");
    }
}

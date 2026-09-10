//! Notes out to a backend and back, through a seam that names no backend.
//!
//! Everything here is written against [`crate::application::notes_backend`]'s
//! trait and knows nothing about what is behind it. That is not tidiness: it
//! is the whole claim `05.1-03` makes and the thing `05.1-04` is written to
//! test, by putting a second implementation behind the same seam and finding
//! out what had really been shaped around the first.
//!
//! `05.1-03` set that as an acceptance criterion and checks it by grepping this
//! file for the name of every backend this program will speak. The command is
//! deliberately not quoted here: written out, it would put those names in this
//! file and the check would answer with its own quotation for ever. It is in
//! `05.1-03-SUMMARY.md`, where it can be re-run rather than trusted.
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

use crate::application::notes_backend::{ANoteThere, NotesService, WhatTheBackendSaid};
use crate::application::summing_up::{SummingUp, how_many};
use crate::common::Result;
use crate::data::message_cache::{MessageCache, NoteEntry};

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
    /// Changes made here that reached the backend, deletions among them.
    pub sent: usize,
    /// Notes that moved here and at the backend since the last sync, and are
    /// held for somebody to choose between.
    ///
    /// Nothing is written and nothing is sent for one of these. Which copy is
    /// kept is [`crate::application::conflict_choice`]'s question and it is
    /// asked of the person rather than answered here.
    pub held: usize,
    /// Notes the backend could not keep exactly as they were typed.
    ///
    /// Not a failure and not a problem. The note went, and what came back is
    /// what the backend can hold: a format that cannot carry a run of spaces or
    /// a trailing one hands back something equivalent and not identical, and no
    /// client speaking it can do better.
    ///
    /// Counted and said, because the alternative is the two copies quietly
    /// differing from the moment of the first push, with nothing said until
    /// something at the other end moves a marker and the read writes the
    /// backend's version of somebody's note over theirs.
    pub not_kept_exactly: usize,
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
        if self.held > 0 {
            // The sentence the contacts and calendar syncs already say about a
            // disagreement, with the hole filled in for a note. One set of
            // words with a hole in it cannot drift from itself, which is what
            // `TheOtherCopy`'s own doc comment is about.
            said.sentence(
                crate::application::conflict_choice::how_many_are_waiting_to_be_chosen(
                    self.held,
                    crate::application::conflict_choice::TheOtherCopy::ANotesBackend,
                ),
            );
        }
        if self.waiting_on_the_setting > 0 {
            // The task, calendar and contacts syncs all say this, so it is
            // said in one place. Its own comment records that two copies of it
            // drifted and only one was corrected.
            said.sentence(crate::application::allowed::changes_waiting_here(
                self.waiting_on_the_setting,
            ));
        }
        if self.not_kept_exactly > 0 {
            // Written here rather than beside the other three sentences,
            // because nothing else says it. `allowed::changes_waiting_here` and
            // `conflict_choice`'s question are shared by four syncs and live
            // where all four can reach them; this is about a notes backend and
            // has one caller.
            said.sentence(format!(
                "{} could not be kept exactly by your notes backend",
                how_many(self.not_kept_exactly, "note")
            ));
        }
        said.spoken()
    }
}

/// Send what is waiting, then take what has arrived.
///
/// `container` is opaque and is handed straight back. Nothing here parses it,
/// splits it or builds one, which is a requirement of the seam rather than a
/// habit: one backend's container is one address and another's is four levels
/// of hierarchy flattened into a string, so anything that took a container
/// apart would be reading one backend's addressing.
///
/// The push goes first. A change made here that has not left is the thing most
/// easily lost, and running the read first would answer the disagreement with
/// the backend's copy before anybody had offered ours.
pub async fn sync_notes<S: NotesService>(
    cache: &MessageCache,
    service: &S,
    account_id: &str,
    container: &str,
) -> Result<NoteSyncResult> {
    let mut result = NoteSyncResult::default();
    // Before the push reads what it is owed and before the read asks what was
    // deleted, so that both work from the same answer.
    if let Err(e) = crate::application::deletions::let_go_of_what_was_remembered_long_enough(
        cache,
        chrono::Utc::now(),
    ) {
        result.errors.push(format!(
            "The deletions remembered here could not be swept: {e}"
        ));
    }
    send_the_deletions(cache, service, account_id, container, &mut result).await;
    push_what_is_waiting(cache, service, account_id, container, &mut result).await;
    take_what_has_arrived(cache, service, account_id, container, &mut result).await;
    Ok(result)
}

/// Ask the backend to remove every note somebody deleted here.
///
/// A record with no backend name on it is a note that never left this
/// computer. There is nothing to ask for and nothing a read could name it by,
/// so it is a memory from the moment it is written rather than work the push
/// owes.
async fn send_the_deletions<S: NotesService>(
    cache: &MessageCache,
    service: &S,
    account_id: &str,
    container: &str,
    result: &mut NoteSyncResult,
) {
    let owed = match cache.deleted_notes(account_id) {
        Ok(owed) => owed,
        Err(e) => {
            result.errors.push(format!(
                "The deletions waiting to be sent could not be read: {e}"
            ));
            return;
        }
    };
    for gone in owed.into_iter().filter(|gone| gone.so_far.still_owed()) {
        let Some(named) = gone.known_as.clone() else {
            continue;
        };
        let known_as = ANoteThere {
            named,
            // Deliberately not passed on. Somebody asked for the note to go,
            // and a version that had moved on would make the removal fail for
            // ever. The calendar sync's own removal settles the same question
            // the same way, and its comment says so where it is written.
            version: None,
        };
        match service.take_a_note_away(container, &known_as).await {
            Ok(WhatTheBackendSaid::Done { .. }) | Ok(WhatTheBackendSaid::ItIsNotThere) => {
                // Gone at the other end either way, and the record stays. It
                // stops being work the push has and becomes the only thing
                // standing between the note and a read that is still naming
                // it.
                match cache.a_backend_took_the_deletion_of_a_note(
                    &gone.id,
                    &crate::application::deletions::written(chrono::Utc::now()),
                ) {
                    Ok(()) => result.sent += 1,
                    Err(e) => result.errors.push(format!("Note {}: {e}", gone.id)),
                }
            }
            // The record is not marked taken, so turning the setting on still
            // sends it.
            Ok(WhatTheBackendSaid::NotAllowedToChangeAnything) => {
                result.waiting_on_the_setting += 1;
            }
            Ok(WhatTheBackendSaid::NotSignedIn) => result.needs_sign_in = true,
            Ok(WhatTheBackendSaid::ItMovedFirst { .. }) => result.errors.push(format!(
                "Note {}: the backend's copy moved first, so it was not removed",
                gone.id
            )),
            Ok(WhatTheBackendSaid::CouldNotBeReached(said)) => {
                result.errors.push(format!("Note {}: {said}", gone.id));
            }
            Err(e) if crate::service::outward::was_refused_by_the_gate(&e) => {
                result.waiting_on_the_setting += 1;
            }
            Err(e) => result.errors.push(format!("Note {}: {e}", gone.id)),
        }
    }
}

/// Offer every note changed here that nobody has been told about.
async fn push_what_is_waiting<S: NotesService>(
    cache: &MessageCache,
    service: &S,
    account_id: &str,
    container: &str,
    result: &mut NoteSyncResult,
) {
    let waiting = match cache.notes_waiting_to_be_sent(account_id) {
        Ok(waiting) => waiting,
        Err(e) => {
            result.errors.push(format!(
                "The notes waiting to be sent could not be read: {e}"
            ));
            return;
        }
    };

    for note in waiting {
        // A note waiting on somebody's choice is not offered again. A later
        // sync must not resolve what the person has not, and offering it would
        // get the same answer and write a second hold over the first every
        // time.
        if cache.is_held_for_a_choice(&note.id).unwrap_or(false) {
            continue;
        }
        // Built here rather than kept as a field, because the two columns are
        // one fact about one pairing and a backend is handed the pairing.
        let known_as = note.known_as.as_ref().map(|named| ANoteThere {
            named: named.clone(),
            version: note.known_version.clone(),
        });
        let said = service
            .leave_a_note_saying(container, known_as.as_ref(), &note.title, &note.body)
            .await;
        match said {
            Ok(WhatTheBackendSaid::Done {
                known_as: there,
                what_it_could_keep,
            }) => {
                match cache.a_backend_took_the_note(
                    &note.id,
                    &there.named,
                    there.version.as_deref(),
                ) {
                    // The note's id, not its title. This goes to the log, and a
                    // title is the person's own words in the same way a message
                    // body is. The id finds the row.
                    Err(e) => result.errors.push(format!("Note {}: {e}", note.id)),
                    Ok(()) => {
                        result.sent += 1;
                        keep_what_the_backend_could(cache, &note, what_it_could_keep, result);
                    }
                }
            }
            // Held by what this program is allowed to change, before anything
            // left the machine. Counted rather than pushed into `errors`,
            // because nothing went wrong: as an error it is "1 problem" on
            // every sync until the setting changes. Nothing else happens here
            // on purpose, and that is the load-bearing half: the flag stays
            // set, so turning the setting on still sends it.
            Ok(WhatTheBackendSaid::NotAllowedToChangeAnything) => {
                result.waiting_on_the_setting += 1;
            }
            // Said in words rather than counted. An account that keeps being
            // refused is "1 problem" every sync forever, and the count is all
            // the status line shows.
            Ok(WhatTheBackendSaid::NotSignedIn) => result.needs_sign_in = true,
            // Both copies moved. Which one is kept is not decided here and was
            // not decided by the backend either: it wrote nothing and handed
            // the question over, which is what the seam's contract requires of
            // it. Both copies are held and somebody is asked.
            Ok(WhatTheBackendSaid::ItMovedFirst { version_now }) => {
                hold_both_copies_of(cache, service, container, &note, version_now, result).await;
            }
            // The backend does not hold the note this computer thinks it does,
            // so what this computer holds is a name the backend has never
            // given. The seam's contract says such a note is one to create
            // rather than one to look up, and offering it afresh is the only
            // ending that does not leave the change waiting here for ever
            // while the same sentence arrives on every sync from now on.
            //
            // Not a resurrection of something deleted. A note deleted here
            // leaves a record and never reaches this loop at all; what reaches
            // it is a note somebody still has, holding a change they made,
            // whose copy at the other end has gone.
            Ok(WhatTheBackendSaid::ItIsNotThere) => {
                offer_it_as_a_new_note(cache, service, container, &note, result).await;
            }
            // The string is for the log. Nothing reads it out: text a crate
            // wrote for a developer is not text to speak to somebody whose
            // notes did not sync.
            Ok(WhatTheBackendSaid::CouldNotBeReached(said)) => {
                result.errors.push(format!("Note {}: {said}", note.id));
            }
            // The same refusal as `NotAllowedToChangeAnything`, arriving as an
            // error because the gate stops a request before the client is even
            // asked. Both are the setting, and both are counted.
            Err(e) if crate::service::outward::was_refused_by_the_gate(&e) => {
                result.waiting_on_the_setting += 1;
            }
            Err(e) => result.errors.push(format!("Note {}: {e}", note.id)),
        }
    }
}

/// Write down what the backend kept, where it could not keep what it was given.
///
/// Nothing to do in the ordinary case, which is a backend that kept the bytes.
///
/// # Why the copy here becomes the copy there
///
/// The two have to agree, and the only moment when this program can tell a
/// backend's own normalising from a change somebody made at the other end is
/// this one, because the backend has just said which it is. Left alone, the two
/// copies differ from the moment of the first push with nothing said, and the
/// first thing that moves a marker at the backend, which the seam's contract
/// says may happen for reasons the content did not cause, brings the backend's
/// version of somebody's note down over theirs with no question asked.
///
/// So the loss happens once, at the moment somebody asked for a sync, and is
/// counted so the sync can say it. That is the difference between a limit of a
/// format and a note that changed for no reason anybody can see.
fn keep_what_the_backend_could(
    cache: &MessageCache,
    note: &NoteEntry,
    what_it_could_keep: Option<crate::application::notes_backend::WhatTheBackendKept>,
    result: &mut NoteSyncResult,
) {
    let Some(kept) = what_it_could_keep else {
        return;
    };
    let changed = NoteEntry {
        title: kept.title,
        body: kept.body,
        // It is now exactly what the backend holds, so there is nothing left to
        // send. The identity and the marker were written a moment ago by
        // `a_backend_took_the_note` and are read back rather than guessed at,
        // because writing the ones in hand would put back the ones from before
        // the push.
        pending: false,
        updated_at: chrono::Utc::now().to_rfc3339(),
        ..match cache.get_note(&note.id) {
            Ok(Some(now)) => now,
            Ok(None) => return,
            Err(e) => {
                result.errors.push(format!("Note {}: {e}", note.id));
                return;
            }
        }
    };
    match cache.save_note(&changed) {
        Ok(()) => result.not_kept_exactly += 1,
        Err(e) => result.errors.push(format!("Note {}: {e}", note.id)),
    }
}

/// Offer a note the backend turns out not to hold as one it has never held.
///
/// Offered once, not in a loop. A backend that refuses the create as well has
/// said something else, and that answer is reported the way any other is.
///
/// The identifier is dropped rather than kept, because a name the backend has
/// never given is not a name: keeping it would send the next change to the same
/// place and get the same answer for ever.
async fn offer_it_as_a_new_note<S: NotesService>(
    cache: &MessageCache,
    service: &S,
    container: &str,
    note: &NoteEntry,
    result: &mut NoteSyncResult,
) {
    match service
        .leave_a_note_saying(container, None, &note.title, &note.body)
        .await
    {
        Ok(WhatTheBackendSaid::Done {
            known_as: there,
            what_it_could_keep,
        }) => {
            match cache.a_backend_took_the_note(&note.id, &there.named, there.version.as_deref()) {
                Ok(()) => {
                    result.sent += 1;
                    keep_what_the_backend_could(cache, note, what_it_could_keep, result);
                }
                Err(e) => result.errors.push(format!("Note {}: {e}", note.id)),
            }
        }
        Ok(WhatTheBackendSaid::NotAllowedToChangeAnything) => {
            result.waiting_on_the_setting += 1;
        }
        Ok(WhatTheBackendSaid::NotSignedIn) => result.needs_sign_in = true,
        // A backend that says it does not hold a note it was asked to make is
        // saying something this code cannot act on, and the same is true of a
        // clash reported for a note it has never held.
        Ok(WhatTheBackendSaid::ItIsNotThere) | Ok(WhatTheBackendSaid::ItMovedFirst { .. }) => {
            result.errors.push(format!(
                "Note {}: the backend would not make a note it says it does not hold",
                note.id
            ));
        }
        Ok(WhatTheBackendSaid::CouldNotBeReached(said)) => {
            result.errors.push(format!("Note {}: {said}", note.id));
        }
        Err(e) if crate::service::outward::was_refused_by_the_gate(&e) => {
            result.waiting_on_the_setting += 1;
        }
        Err(e) => result.errors.push(format!("Note {}: {e}", note.id)),
    }
}

/// Keep both copies of a note that moved in two places, and ask nobody.
///
/// The backend's words have to be fetched, because a backend that reports the
/// disagreement reports a marker and not a document, and somebody choosing
/// needs to hear what each copy actually says. A difference on its own reads as
/// an instruction to reconstruct the two copies in your head.
///
/// A fetch that fails leaves the note waiting and says so. Holding with one
/// side empty would be a question nobody can answer.
async fn hold_both_copies_of<S: NotesService>(
    cache: &MessageCache,
    service: &S,
    container: &str,
    note: &NoteEntry,
    version_now: Option<String>,
    result: &mut NoteSyncResult,
) {
    let Some(named) = note.known_as.clone() else {
        // The backend cannot have an older copy of a note it has never held.
        result.errors.push(format!(
            "Note {}: the backend said its copy moved first for a note it has \
             never held",
            note.id
        ));
        return;
    };
    let theirs = ANoteThere {
        named,
        version: version_now.clone(),
    };
    let said = match service.what_a_note_says(container, &theirs).await {
        Ok(Some(said)) => said,
        Ok(None) => {
            result.errors.push(format!(
                "Note {}: the backend said its copy moved first and then had no \
                 copy to show",
                note.id
            ));
            return;
        }
        Err(e) => {
            result.errors.push(format!("Note {}: {e}", note.id));
            return;
        }
    };

    hold_these_two_copies(cache, note, container, &said, version_now, result);
}

/// Write down both copies of one note and what each says.
///
/// Apart from [`hold_both_copies_of`] because the two callers arrive holding
/// different things. The push is told a marker and has to fetch the words; the
/// read has already fetched them, and asking again would be a second request
/// for an answer in hand.
///
/// `version_now` is what the backend said the marker was when it reported the
/// disagreement, used only where what was fetched carries none. Without a
/// marker written down, the next sync finds the same disagreement and asks the
/// same question again, and a choice that has to be made every sync is not a
/// choice.
fn hold_these_two_copies(
    cache: &MessageCache,
    note: &NoteEntry,
    container: &str,
    said: &crate::application::notes_backend::ANoteAsItStands,
    version_now: Option<String>,
    result: &mut NoteSyncResult,
) {
    let held = crate::data::message_cache::held_conflicts::AHeldConflict {
        id: note.id.clone(),
        account_id: note.account_id.clone(),
        // The container, which is opaque and is stored rather than read. It is
        // "which one is on the other side" for a note in the way an address
        // book's name is for a contact.
        at: container.to_string(),
        copies: crate::application::conflict_choice::BothCopies {
            what_it_is_called: note.title.clone(),
            other_copy: crate::application::conflict_choice::TheOtherCopy::ANotesBackend,
            // Named the way somebody would say them rather than by column, and
            // in the order the note editor asks for them, because these are
            // read aloud.
            here: vec![
                crate::application::conflict_choice::AField::new("Title", note.title.clone()),
                crate::application::conflict_choice::AField::new("Body", note.body.clone()),
            ],
            theirs: vec![
                crate::application::conflict_choice::AField::new("Title", said.title.clone()),
                crate::application::conflict_choice::AField::new("Body", said.body.clone()),
            ],
        },
        // What the other copy carries now, so settling writes it down. Without
        // it the marker here is still the one from before the backend moved,
        // and the very next sync finds the same disagreement and asks the same
        // question again: a choice that has to be made every sync is not a
        // choice.
        their_version: said.known_as.version.clone().or(version_now),
        held_at: chrono::Utc::now().to_rfc3339(),
    };
    match cache.hold_a_conflict(&held) {
        Ok(()) => result.held += 1,
        Err(e) => result.errors.push(format!("Note {}: {e}", note.id)),
    }
}

/// Take down whatever the backend holds that this computer does not have.
async fn take_what_has_arrived<S: NotesService>(
    cache: &MessageCache,
    service: &S,
    account_id: &str,
    container: &str,
    result: &mut NoteSyncResult,
) {
    let there = match service.notes_it_holds(container).await {
        Ok(there) => there,
        Err(e) => {
            // Said rather than reported as an empty container. A read that
            // could not happen and a container with nothing in it are the same
            // silence, and only one of them means somebody's notes are missing.
            result
                .errors
                .push(format!("The notes at the backend could not be read: {e}"));
            return;
        }
    };
    let here = match cache.get_all_notes_for_account(account_id) {
        Ok(here) => here,
        Err(e) => {
            result
                .errors
                .push(format!("The notes on this computer could not be read: {e}"));
            return;
        }
    };

    // Asked before anything is written down, which is the whole of the rule
    // `application::deletions` states. Built from every record the account
    // holds, still owed or already taken, because "did this computer delete
    // it" does not depend on whether the backend has been told yet.
    let deleted_here: crate::application::deletions::DeletedHere =
        match cache.deleted_notes(account_id) {
            Ok(gone) => gone
                .into_iter()
                .filter_map(|record| record.known_as)
                .collect(),
            Err(e) => {
                result.errors.push(format!(
                    "What was deleted here could not be read, so nothing was taken down: {e}"
                ));
                return;
            }
        };

    for one in there {
        // A note this computer deleted is not written back down, however long
        // the backend's own list goes on naming it. Without this the note comes
        // back on the screen under the backend's own identifier, with nothing
        // left to say it was ever deleted, which is what the deletion record
        // exists to prevent.
        if deleted_here.holds(&one.named) {
            continue;
        }
        let ours = here
            .iter()
            .find(|note| note.known_as.as_deref() == Some(one.named.as_str()));
        // A note waiting on somebody's choice is not written over by a later
        // sync. Losing the hold is losing the choice and one of the two copies
        // with it.
        if ours.is_some_and(|note| cache.is_held_for_a_choice(&note.id).unwrap_or(false)) {
            continue;
        }
        if the_marker_stayed_still(ours.and_then(|note| note.known_version.as_deref()), &one) {
            result.unchanged += 1;
            continue;
        }
        let said = match service.what_a_note_says(container, &one).await {
            Ok(Some(said)) => said,
            // Gone between the listing and the reading, which is ordinary
            // rather than wrong: somebody deleted it at the other end while
            // this was running.
            Ok(None) => continue,
            Err(e) => {
                result
                    .errors
                    .push(format!("A note at the backend could not be read: {e}"));
                continue;
            }
        };
        // What arrived is what is here already. The marker moved and the words
        // did not, which the seam's contract says a backend is allowed to do
        // and which it calls the cost of a fetch nobody needed. It is more than
        // that if the row is written again: the note's changed time becomes the
        // time of the sync, so somebody's list of what they last worked on
        // becomes a list of what they last synced, and `stored` counts a note
        // that was not.
        //
        // The marker is still written down, or the next sync fetches the same
        // note again and the one after that as well.
        if let Some(here) = ours
            && here.title == said.title
            && here.body == said.body
        {
            match cache.a_backend_took_the_note(
                &here.id,
                &said.known_as.named,
                said.known_as.version.as_deref(),
            ) {
                Ok(()) => result.unchanged += 1,
                Err(e) => result
                    .errors
                    .push(format!("A note from the backend could not be kept: {e}")),
            }
            continue;
        }
        // A change nobody has sent is not written over by a read.
        //
        // The push is where a clash is normally found, and the push does not
        // always run. The setting can refuse it, nobody may be signed in, the
        // request can fail. In each of those the waiting flag stays set on
        // purpose, so that fixing the cause still sends the change, and a read
        // that wrote the backend's copy over it in the same sync made that
        // promise false: the change was gone and the flag was cleared, with a
        // count saying one change was waiting and nothing saying it no longer
        // was.
        //
        // `05.1-03` put the whole conflict story on the push side, which is
        // where a backend with a marker reports one. Nothing asked what happens
        // when the push never reaches the backend at all.
        if let Some(here) = ours
            && here.pending
            && (here.title != said.title || here.body != said.body)
        {
            hold_these_two_copies(cache, here, container, &said, None, result);
            continue;
        }
        match write_it_down(cache, account_id, ours, &said) {
            Ok(()) => result.stored += 1,
            Err(e) => result
                .errors
                .push(format!("A note from the backend could not be kept: {e}")),
        }
    }
}

/// Whether the backend's copy is the one this computer already has.
///
/// A missing marker is not evidence that a copy stayed still, it is no
/// evidence at all, so a backend that gives none has every copy treated as
/// having moved. `contacts_sync::the_marker_moved` says the same thing for a
/// contact and for the same reason, and the seam's contract writes it down as
/// a requirement rather than leaving each caller to decide.
///
/// Compared for equality and nothing else. A backend does not promise its
/// marker is opaque, orderable or tied to the content, so ordering two of them
/// or reading one as a date would be one backend's reading of the word.
fn the_marker_stayed_still(ours: Option<&str>, theirs: &ANoteThere) -> bool {
    match (ours, theirs.version.as_deref()) {
        (Some(ours), Some(theirs)) => ours == theirs,
        _ => false,
    }
}

/// Keep what the backend said, over the note it belongs to or as a new one.
fn write_it_down(
    cache: &MessageCache,
    account_id: &str,
    ours: Option<&NoteEntry>,
    said: &crate::application::notes_backend::ANoteAsItStands,
) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let note = match ours {
        // Everything else about the row stays: which folder somebody filed it
        // in, whether they pinned it, when they made it. None of those is the
        // backend's to say, and writing a whole row from what arrived is how a
        // sync comes to unpin somebody's note.
        Some(ours) => NoteEntry {
            title: said.title.clone(),
            body: said.body.clone(),
            known_as: Some(said.known_as.named.clone()),
            known_version: said.known_as.version.clone(),
            // It is now exactly what the backend holds, so there is nothing
            // left to send. Left waiting, the next push would write the
            // backend's own words back at it on every sync forever.
            pending: false,
            updated_at: now,
            ..ours.clone()
        },
        None => NoteEntry {
            id: format!("note-{}", uuid::Uuid::new_v4()),
            account_id: account_id.to_string(),
            // The account's first note folder. Folders on this computer are
            // not mirrored at the backend and this does not pretend they are:
            // what a folder means at each backend is a decision somebody has
            // to make, and the module header says so rather than leaving a
            // reader to work out why every arrival lands in one place.
            folder_id: Some(cache.ensure_default_note_folder(account_id)?.id),
            title: said.title.clone(),
            body: said.body.clone(),
            format: crate::data::message_cache::NoteBody::AsTyped,
            pinned: false,
            created_at: now.clone(),
            updated_at: now,
            pending: false,
            known_as: Some(said.known_as.named.clone()),
            known_version: said.known_as.version.clone(),
        },
    };
    cache.save_note(&note)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::notes_backend::{ANoteAsItStands, ANoteThere, WhatTheBackendSaid};
    use crate::common::temp_home::TempHome;
    use crate::common::{Error, Result as OurResult};
    use crate::data::message_cache::{NoteBody, NoteFolderEntry};
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
        /// The backend's copy moved since this computer last looked, so it
        /// wrote nothing and handed the question back.
        ItMovedFirst,
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
        /// The marker it says its own copy carries, when it says that copy
        /// moved first.
        const THE_MARKER_IT_HAS_NOW: &'static str = "moved-on";
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
                Writes::Accepted => WhatTheBackendSaid::done(ANoteThere {
                    named: known_as
                        .map(|there| there.named.clone())
                        .unwrap_or_else(|| format!("made-for-{title}")),
                    version: Some(Self::A_NEW_MARKER.to_string()),
                }),
                Writes::RefusedByTheSetting => WhatTheBackendSaid::NotAllowedToChangeAnything,
                Writes::NotSignedIn => WhatTheBackendSaid::NotSignedIn,
                Writes::ItMovedFirst => WhatTheBackendSaid::ItMovedFirst {
                    version_now: Some(Self::THE_MARKER_IT_HAS_NOW.to_string()),
                },
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
                Writes::Accepted => WhatTheBackendSaid::done(known_as.clone()),
                Writes::RefusedByTheSetting => WhatTheBackendSaid::NotAllowedToChangeAnything,
                Writes::NotSignedIn => WhatTheBackendSaid::NotSignedIn,
                Writes::ItMovedFirst => WhatTheBackendSaid::ItMovedFirst {
                    version_now: Some(Self::THE_MARKER_IT_HAS_NOW.to_string()),
                },
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
    fn test_a_note_deleted_here_is_offered_to_the_backend_for_deletion() {
        let cache = a_store();
        let mut note = a_note("n1", "Going", "Gone");
        note.known_as = Some("there-1".to_string());
        note.known_version = Some("v1".to_string());
        cache.save_note(&note).expect("a note the backend holds");
        cache.delete_note("n1").expect("the deletion");
        let service = Scripted {
            writes: Writes::Accepted,
            ..Scripted::default()
        };

        let result = run(sync_notes(&cache, &service, ACCOUNT, CONTAINER)).expect("a sync");

        assert_eq!(
            *service.taken_away.lock().expect("what was removed"),
            ["there-1"],
            "the backend was never asked to remove the note"
        );
        assert_eq!(result.sent, 1, "{result:?}");
        let record = &cache.deleted_notes(ACCOUNT).expect("the deletions")[0];
        assert!(
            !record.so_far.still_owed(),
            "the deletion is still owed after the backend took it, so it is \
             sent again on every sync"
        );
    }

    #[test]
    fn test_a_deletion_the_backend_has_taken_is_still_remembered() {
        // The half that is easy to get wrong and impossible to see. Dropping
        // the record the moment the backend takes it leaves the read with
        // nothing to consult while the backend's own list is still naming the
        // note, and that is what puts a deleted note back on the screen in the
        // very sync that deleted it.
        let cache = a_store();
        let mut note = a_note("n1", "Going", "Gone");
        note.known_as = Some("there-1".to_string());
        cache.save_note(&note).expect("a note the backend holds");
        cache.delete_note("n1").expect("the deletion");
        let service = Scripted {
            writes: Writes::Accepted,
            ..Scripted::default()
        };

        run(sync_notes(&cache, &service, ACCOUNT, CONTAINER)).expect("a sync");

        assert_eq!(
            cache.deleted_notes(ACCOUNT).expect("the deletions").len(),
            1,
            "the record went the moment the backend took it, so nothing is \
             left to stop a read writing the note back down"
        );
    }

    #[test]
    fn test_a_note_deleted_here_is_not_written_back_down_while_the_backend_still_names_it() {
        // The case the rule is about, driven rather than assumed. The backend's
        // own list has not caught up, so it still names the note, and the read
        // has to skip it. A fake whose list simply does not name it would pass
        // against a read that consults nothing.
        let cache = a_store();
        let mut note = a_note("n1", "Going", "Gone");
        note.known_as = Some("there-1".to_string());
        cache.save_note(&note).expect("a note the backend holds");
        cache.delete_note("n1").expect("the deletion");
        let service = Scripted {
            writes: Writes::Accepted,
            holds: vec![ANoteAsItStands {
                known_as: ANoteThere {
                    named: "there-1".to_string(),
                    version: Some("v1".to_string()),
                },
                title: "Going".to_string(),
                body: "Gone".to_string(),
            }],
            ..Scripted::default()
        };

        let result = run(sync_notes(&cache, &service, ACCOUNT, CONTAINER)).expect("a sync");

        assert_eq!(result.stored, 0, "{result:?}");
        assert!(
            cache
                .get_all_notes_for_account(ACCOUNT)
                .expect("what is here")
                .is_empty(),
            "a note somebody deleted came back in the sync that deleted it"
        );
    }

    #[test]
    fn test_a_note_that_moved_in_both_places_is_held_rather_than_written_over() {
        let cache = a_store();
        let mut note = a_note("n1", "What is here", "The words typed here");
        note.pending = true;
        note.known_as = Some("there-1".to_string());
        note.known_version = Some("v1".to_string());
        cache.save_note(&note).expect("a note changed here");
        let service = Scripted {
            writes: Writes::ItMovedFirst,
            holds: vec![ANoteAsItStands {
                known_as: ANoteThere {
                    named: "there-1".to_string(),
                    version: Some(Scripted::THE_MARKER_IT_HAS_NOW.to_string()),
                },
                title: "What they have".to_string(),
                body: "The words typed there".to_string(),
            }],
            ..Scripted::default()
        };

        let result = run(sync_notes(&cache, &service, ACCOUNT, CONTAINER)).expect("a sync");

        assert_eq!(result.held, 1, "{result:?}");
        assert_eq!(result.stored, 0, "the backend's copy was written over here");
        assert_eq!(result.sent, 0, "{result:?}");
        assert!(
            cache.is_held_for_a_choice("n1").expect("the hold"),
            "both copies moved and nothing is holding the question"
        );

        let held = cache
            .the_conflict_held_for("n1")
            .expect("the store to answer")
            .expect("the hold");
        assert_eq!(
            held.copies.fields_that_differ(),
            ["Title", "Body"],
            "somebody choosing is not told what differs"
        );
        assert_eq!(
            held.their_version.as_deref(),
            Some(Scripted::THE_MARKER_IT_HAS_NOW),
            "the marker the other copy carries now was not written down, so the \
             same question is asked on every sync from here on"
        );

        let here = cache.get_note("n1").expect("the note").expect("the note");
        assert_eq!(
            here.body, "The words typed here",
            "the copy here was written over while the question was being asked"
        );
    }

    #[test]
    fn test_a_note_held_for_a_choice_is_not_offered_again_on_the_next_sync() {
        // A later sync must not resolve what the person has not. Without this
        // the push offers the note again, gets the same answer, and writes a
        // second hold over the first every time.
        let cache = a_store();
        let mut note = a_note("n1", "What is here", "The words typed here");
        note.pending = true;
        note.known_as = Some("there-1".to_string());
        note.known_version = Some("v1".to_string());
        cache.save_note(&note).expect("a note changed here");
        let service = Scripted {
            writes: Writes::ItMovedFirst,
            holds: vec![ANoteAsItStands {
                known_as: ANoteThere {
                    named: "there-1".to_string(),
                    version: Some(Scripted::THE_MARKER_IT_HAS_NOW.to_string()),
                },
                title: "What they have".to_string(),
                body: "The words typed there".to_string(),
            }],
            ..Scripted::default()
        };

        run(sync_notes(&cache, &service, ACCOUNT, CONTAINER)).expect("the first sync");
        let asked_once = service.written.lock().expect("what was written").len();
        // The note is still waiting after the first sync, which is what makes
        // the count below mean anything. Without this the test passes against a
        // read that wrote the backend's copy over the unsent change: nothing is
        // waiting any more, so of course nothing is offered again.
        let after_one = cache.get_note("n1").expect("the note").expect("the note");
        assert!(after_one.pending, "the change here was given up");
        assert_eq!(after_one.body, "The words typed here");

        let second = run(sync_notes(&cache, &service, ACCOUNT, CONTAINER)).expect("a second sync");

        assert_eq!(
            service.written.lock().expect("what was written").len(),
            asked_once,
            "the held note was offered again while somebody had still not chosen"
        );
        assert_eq!(second.stored, 0, "{second:?}");
        assert_eq!(second.held, 0, "the same question was asked a second time");
    }

    #[test]
    fn test_a_note_changed_only_here_is_sent_and_is_not_held() {
        // The first of the two cases a hold-everything implementation gets
        // wrong. Asked because "a conflict is held" is satisfied by holding
        // every note there is.
        let cache = a_store();
        let mut note = a_note("n1", "Wiring colours", "Brown is live");
        note.pending = true;
        note.known_as = Some("there-1".to_string());
        note.known_version = Some("v1".to_string());
        cache.save_note(&note).expect("a note changed here");
        let service = Scripted {
            writes: Writes::Accepted,
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

        assert_eq!(result.sent, 1, "{result:?}");
        assert_eq!(result.held, 0, "a note only this computer changed was held");
    }

    #[test]
    fn test_a_note_changed_only_at_the_backend_is_taken_and_is_not_held() {
        // The second. Nothing is waiting here, so there is nothing to lose and
        // no question to ask.
        let cache = a_store();
        let mut note = a_note("n1", "Old title", "Old words");
        note.known_as = Some("there-1".to_string());
        note.known_version = Some("v1".to_string());
        cache.save_note(&note).expect("a note nobody changed here");
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
        assert_eq!(
            result.held, 0,
            "a note only the backend changed was held, so somebody is asked a \
             question that has one answer"
        );
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

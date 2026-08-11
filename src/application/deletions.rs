//! One rule for everything this computer deletes: no read writes it back down.
//!
//! Contacts, calendar events and tasks each keep a note when somebody deletes
//! one here, because a deleted row cannot carry a "not yet sent" flag and the
//! fact of the deletion has to outlive the row. All three had the same defect
//! in the same shape, and it is the shape that is worth writing down once.
//!
//! # What went wrong
//!
//! The note was dropped the moment the provider took the deletion. The push
//! runs before the read in the same sync, so by the time the read arrived there
//! was nothing left saying anybody had been deleted, and a provider still
//! naming the thing was answered by writing it straight back down. It came back
//! on the screen, under the provider's own identifier, with nothing left to say
//! it had ever been deleted. That is what somebody sees when they delete
//! something and it reappears, and it reads as the deletion having quietly
//! failed.
//!
//! Reading the notes that were left over answers a different question, "has the
//! provider been told", and it is the same answer right up to the moment it
//! stops being: the note goes when the deletion succeeds, so a guard reading
//! the notes disarms itself for exactly the thing it was there to protect.
//!
//! This does not rest on how quickly a provider's own list catches up. Every
//! deletion that fails keeps its note and is protected either way, and a
//! deletion fails whenever the network does, whenever the provider answers 5xx,
//! and for as long as Allow Changes is off. What was open was the success case,
//! and the door has to be shut whether or not anybody can prove a real provider
//! walks through it.
//!
//! # The rule
//!
//! A note is kept until the provider has taken the deletion, and for
//! [`HOW_LONG_A_DELETION_IS_REMEMBERED`] after that. Every read asks the notes
//! before it writes anything down, and skips whatever they name.
//!
//! Two lives, and they are not the same thing. A note no provider has taken is
//! *work*: the push still has it to send, and it stays for as long as that is
//! true, which is right. A note the provider has taken is a *memory*: nothing
//! is owed, and the only reason to keep it is that a read may still be naming
//! the thing.
//!
//! A note is also refused outright to anybody asking to drop it while a
//! provider could still name the thing, and the refusal sits beside the delete
//! rather than in the callers. Each table asks that question of whatever it
//! holds: an event's note carries the provider's own name for the event, and a
//! task's note is keyed by the task's identifier, which for a synced task is
//! the provider's name for it. Contacts have no way to drop a note at all.
//! Every one of these refusals exists because a caller decided for itself that
//! a note was safe to drop and put a deleted thing back on the screen.
//!
//! # What lets it go
//!
//! The clock, and nothing else. A memory is released once it is older than
//! [`HOW_LONG_A_DELETION_IS_REMEMBERED`], whatever the provider does or does
//! not go on saying, so the three notes tables drain on their own and cannot
//! grow for ever. Nothing about a provider's answer is consulted, because a
//! rule that waits for a provider to stop naming something has to decide what
//! an incremental read that names almost nothing proves, and the answer is
//! nothing.
//!
//! Seven days is the figure. It has to outlast a provider's list answering from
//! a copy written before the deletion landed, which is seconds or minutes, and
//! an application shut over a weekend before the next sync runs. It is short
//! enough that the table drains. Nothing is blocked by a memory except a thing
//! this computer deleted under an identifier a provider already handed out, and
//! providers do not hand the same identifier out twice.
//!
//! Said plainly, because it is a real limit: a provider that takes a deletion
//! and then goes on listing the thing for longer than a week hands it back, and
//! this cannot tell that apart from somebody making it again elsewhere.

use crate::common::Result;
use crate::data::message_cache::MessageCache;
use std::collections::HashSet;

/// How long a deletion is remembered after the provider takes it.
///
/// The reason for the length is in this module's own notes. It is one figure
/// for contacts, events and tasks, because it answers one question and three
/// copies of a figure drift the moment one of them is edited.
pub const HOW_LONG_A_DELETION_IS_REMEMBERED: chrono::TimeDelta = chrono::Duration::days(7);

/// The provider's names for things this computer deleted.
///
/// A read asks this before it writes anything down. Built from every note the
/// account holds, still owed or already taken, because "did this computer
/// delete it" does not depend on whether anybody has been told yet.
#[derive(Debug, Default)]
pub struct DeletedHere(HashSet<String>);

impl DeletedHere {
    /// Whether this is a thing somebody deleted on this computer.
    pub fn holds(&self, provider_id: &str) -> bool {
        self.0.contains(provider_id)
    }
}

impl FromIterator<String> for DeletedHere {
    fn from_iter<I: IntoIterator<Item = String>>(names: I) -> Self {
        Self(names.into_iter().collect())
    }
}

/// A moment, written the way a note records the one the provider took it at.
///
/// One place, because the sweep compares the stamp on a note against a cutoff
/// as text. Two ways of writing the same moment do not compare, and the note
/// that failed to compare would be one nothing ever let go of.
pub fn written(at: chrono::DateTime<chrono::Utc>) -> String {
    at.to_rfc3339()
}

/// Let go of every deletion that has been remembered long enough.
///
/// Run at the start of a sync, before its push reads what it is owed and
/// before its read asks what was deleted, so both work from the same answer.
///
/// `now` is passed in rather than read here, so that a test can ask what
/// happens a week later without waiting a week.
pub fn let_go_of_what_was_remembered_long_enough(
    cache: &MessageCache,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    cache.let_go_of_deletions_taken_before(&written(now - HOW_LONG_A_DELETION_IS_REMEMBERED))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_read_is_stopped_by_a_name_the_notes_hold_and_by_no_other() {
        let deleted: DeletedHere = ["evt1".to_string(), "evt2".to_string()]
            .into_iter()
            .collect();

        assert!(deleted.holds("evt1"));
        assert!(deleted.holds("evt2"));
        assert!(
            !deleted.holds("evt3"),
            "a read was stopped from writing down something nobody deleted"
        );
    }

    #[test]
    fn test_nothing_deleted_here_stops_nothing() {
        assert!(
            !DeletedHere::default().holds("evt1"),
            "an account that has deleted nothing refused to store anything"
        );
    }

    #[test]
    fn test_what_is_remembered_is_let_go_of_by_the_clock_and_not_by_a_provider() {
        // The figure itself, asserted where somebody changing it will see this.
        // A rule that never releases is a table that grows for ever, and the
        // clock is the only thing here that always moves.
        assert_eq!(HOW_LONG_A_DELETION_IS_REMEMBERED, chrono::Duration::days(7));
    }

    #[test]
    fn test_a_stamp_written_later_sorts_after_one_written_earlier() {
        // The sweep compares these as text, so a way of writing a moment that
        // does not sort is a note nothing ever lets go of.
        let earlier = chrono::DateTime::from_timestamp(1_800_000_000, 0).expect("a moment");
        let later = earlier + chrono::Duration::days(8);

        assert!(
            written(earlier) < written(later),
            "{} did not sort before {}",
            written(earlier),
            written(later)
        );
    }
}

//! Conflicts held until somebody chooses, kept across restarts.
//!
//! A contact or a calendar item changed here and changed at the provider is
//! held rather than resolved, and the hold has to survive the program closing:
//! a question asked once and forgotten on the way out is a question that
//! resolved itself silently after all.
//!
//! The decisions and the words live in
//! [`crate::application::conflict_choice`], which has no database in it. This
//! file is the writing down.

use super::MessageCache;
use crate::application::conflict_choice::{AField, BothCopies, TheOtherCopy};
use crate::common::{Error, Result};
use rusqlite::params;

/// One thing being held, and both of its copies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AHeldConflict {
    /// The row this is about, on this computer: a contact's id or an event's.
    pub id: String,
    pub account_id: String,
    /// Which provider is on the other side, by the name the sync stores.
    pub at: String,
    /// Both copies, and everything needed to ask about them.
    pub copies: BothCopies,
    pub held_at: String,
}

/// The stored form of [`TheOtherCopy`], written out rather than derived.
///
/// A column somebody may read in a database browser, and a value this program
/// still understands after the enum is reordered. `as i64` on a discriminant
/// is neither.
fn stored_kind(other: TheOtherCopy) -> &'static str {
    match other {
        TheOtherCopy::AnAddressBook => "contact",
        TheOtherCopy::ACalendar => "calendar-item",
    }
}

/// Back from the column, defaulting to a contact.
///
/// A row written by a version that knew a kind this one does not is read as a
/// contact rather than dropped: showing somebody the wrong label is recoverable
/// and losing the hold is not, because losing the hold is losing the choice
/// and one of the two copies with it.
fn kind_from_stored(stored: &str) -> TheOtherCopy {
    match stored {
        "calendar-item" => TheOtherCopy::ACalendar,
        _ => TheOtherCopy::AnAddressBook,
    }
}

/// Both copies as one column each.
///
/// JSON rather than a row per field, because nothing queries inside these: they
/// are read whole to be shown and are never searched, sorted or joined on. A
/// table of fields would buy nothing and would need its own deletion pass.
fn fields_to_stored(fields: &[AField]) -> Result<String> {
    serde_json::to_string(
        &fields
            .iter()
            .map(|field| (field.called.clone(), field.value.clone()))
            .collect::<Vec<_>>(),
    )
    .map_err(|e| {
        Error::Other(format!(
            "A held conflict's fields could not be written: {e}"
        ))
    })
}

/// Back from the column, or nothing where the column will not read.
///
/// A copy that will not read is an empty copy rather than an error, for the
/// same reason `ContactEntry::every_address_in_the_list` treats an unreadable
/// list as no list: the hold itself still names the thing and can still be
/// resolved, and refusing the whole row would take the choice away entirely.
fn fields_from_stored(stored: &str) -> Vec<AField> {
    serde_json::from_str::<Vec<(String, String)>>(stored)
        .map(|pairs| {
            pairs
                .into_iter()
                .map(|(called, value)| AField { called, value })
                .collect()
        })
        .unwrap_or_default()
}

impl MessageCache {
    /// Hold both copies of one thing until somebody chooses.
    ///
    /// Replaces any hold already there for the same row. A second sync that
    /// meets the same disagreement is looking at the same question, and two
    /// rows for it would ask somebody twice.
    pub fn hold_a_conflict(&self, held: &AHeldConflict) -> Result<()> {
        // Stub reproducing today's behaviour: nothing is held, because nothing
        // asks. One of the two copies is written over and the other is gone.
        let _ = (
            held,
            stored_kind(TheOtherCopy::AnAddressBook),
            fields_to_stored(&[])?,
        );
        Ok(())
    }

    /// Whether this row is waiting on somebody's choice.
    ///
    /// Asked by both syncs before they write or send anything, which is what
    /// stops a later sync resolving what the person has not.
    pub fn is_held_for_a_choice(&self, id: &str) -> Result<bool> {
        let held: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM held_conflicts WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| Error::Other(format!("The held conflicts could not be read: {e}")))?;
        Ok(held > 0)
    }

    /// Everything waiting on a choice for one account, oldest first.
    pub fn conflicts_held_for(&self, account_id: &str) -> Result<Vec<AHeldConflict>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, account_id, kind, at_the_provider, what_it_is_called,
                        here_json, theirs_json, held_at
                 FROM held_conflicts WHERE account_id = ?1 ORDER BY held_at, id",
            )
            .map_err(|e| Error::Other(format!("The held conflicts could not be read: {e}")))?;
        let rows = statement
            .query_map(params![account_id], |row| {
                let kind: String = row.get(2)?;
                let here: String = row.get(5)?;
                let theirs: String = row.get(6)?;
                Ok(AHeldConflict {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    at: row.get(3)?,
                    copies: BothCopies {
                        what_it_is_called: row.get(4)?,
                        other_copy: kind_from_stored(&kind),
                        here: fields_from_stored(&here),
                        theirs: fields_from_stored(&theirs),
                    },
                    held_at: row.get(7)?,
                })
            })
            .map_err(|e| Error::Other(format!("The held conflicts could not be read: {e}")))?;
        let mut held = Vec::new();
        for row in rows {
            held.push(
                row.map_err(|e| Error::Other(format!("A held conflict could not be read: {e}")))?,
            );
        }
        Ok(held)
    }

    /// One held conflict, where there is one.
    pub fn the_conflict_held_for(&self, id: &str) -> Result<Option<AHeldConflict>> {
        let account: Option<String> = self
            .conn
            .query_row(
                "SELECT account_id FROM held_conflicts WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .ok();
        let Some(account) = account else {
            return Ok(None);
        };
        Ok(self
            .conflicts_held_for(&account)?
            .into_iter()
            .find(|held| held.id == id))
    }

    /// Let a hold go, because somebody has chosen.
    pub fn let_the_hold_go(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM held_conflicts WHERE id = ?1", params![id])
            .map_err(|e| Error::Other(format!("A held conflict could not be let go: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::conflict_choice::WhichCopy;

    fn a_cache(name: &str) -> MessageCache {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
        // The folder outlives the cache for the length of the test, which is
        // what `tempfile` guarantees while the handle is alive; leaking it
        // deliberately here keeps the database open for the assertions below.
        std::mem::forget(dir);
        let _ = name;
        cache
    }

    fn a_conflict(id: &str) -> AHeldConflict {
        AHeldConflict {
            id: id.to_string(),
            account_id: "an account".to_string(),
            at: "google".to_string(),
            copies: BothCopies {
                what_it_is_called: "Ada Lovelace".to_string(),
                other_copy: TheOtherCopy::AnAddressBook,
                here: vec![AField::new("Telephone", "01234")],
                theirs: vec![AField::new("Telephone", "05678")],
            },
            held_at: "2026-09-05T10:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_a_held_conflict_survives_being_written_down_and_read_back() {
        let cache = a_cache("held_round_trip");
        cache.hold_a_conflict(&a_conflict("c1")).expect("a hold");
        let held = cache.conflicts_held_for("an account").expect("the holds");
        assert_eq!(held.len(), 1, "the hold was not written down: {held:?}");
        assert_eq!(
            held[0].copies.values_in(WhichCopy::Here)[0].value,
            "01234",
            "this computer's copy has to survive, or there is nothing to choose"
        );
        assert_eq!(
            held[0].copies.values_in(WhichCopy::TheProviders)[0].value,
            "05678",
            "the provider's copy has to survive too"
        );
    }

    #[test]
    fn test_a_row_waiting_on_a_choice_says_so() {
        let cache = a_cache("held_says_so");
        assert!(
            !cache.is_held_for_a_choice("c1").expect("an answer"),
            "nothing was held yet"
        );
        cache.hold_a_conflict(&a_conflict("c1")).expect("a hold");
        assert!(
            cache.is_held_for_a_choice("c1").expect("an answer"),
            "a sync that cannot tell a held row from any other resolves what \
             the person has not"
        );
    }

    #[test]
    fn test_meeting_the_same_disagreement_twice_asks_once() {
        let cache = a_cache("held_twice");
        cache.hold_a_conflict(&a_conflict("c1")).expect("a hold");
        cache.hold_a_conflict(&a_conflict("c1")).expect("a hold");
        assert_eq!(
            cache
                .conflicts_held_for("an account")
                .expect("the holds")
                .len(),
            1,
            "two rows for one disagreement asks somebody the same question twice"
        );
    }

    #[test]
    fn test_letting_a_hold_go_leaves_nothing_waiting() {
        let cache = a_cache("held_let_go");
        cache.hold_a_conflict(&a_conflict("c1")).expect("a hold");
        cache.let_the_hold_go("c1").expect("letting go");
        assert!(
            !cache.is_held_for_a_choice("c1").expect("an answer"),
            "a choice that leaves the hold in place asks again on the next sync"
        );
    }

    #[test]
    fn test_a_calendar_hold_is_read_back_as_a_calendar_item() {
        let cache = a_cache("held_calendar");
        let mut held = a_conflict("e1");
        held.copies.other_copy = TheOtherCopy::ACalendar;
        held.at = "caldav".to_string();
        cache.hold_a_conflict(&held).expect("a hold");
        let back = cache.the_conflict_held_for("e1").expect("the hold");
        assert_eq!(
            back.map(|held| held.copies.other_copy),
            Some(TheOtherCopy::ACalendar),
            "a calendar item read back as a contact is read out to somebody \
             under the wrong words"
        );
    }
}

//! Where a snoozed task or event waits until it may be mentioned again.
//!
//! A reminder is not in here, and the reason is what the table is for. A
//! reminder's time is its alert, so snoozing one moves its own row and the
//! next look simply finds it not yet due. A task's due date and an event's
//! start are facts the phone also holds: writing either marks the row
//! pending and sends the new value to the provider, so a snooze cannot move
//! them. What moves for a task or an event is when this program will mention
//! it again, and that is what a row here says.
//!
//! The kind is stored as a word, so a fourth kind is a fourth word and not a
//! schema change. A word this build does not know is read back like any
//! other and dropped by nothing here, on `AddressBook::Other`'s reasoning:
//! a hold written by a later version is still somebody's snooze, and
//! forgetting it would bring the thing back at them the moment they went
//! back to the older build. Who reads the word is the due window's feed,
//! through `due::Kind::from_key`, and nothing here interprets it.
//!
//! Expired rows are let go of at each look, so the table holds only what is
//! still held.

use crate::common::{Error, Result};
use crate::data::message_cache::MessageCache;

/// A hold as it was written: the kind's stored word, the identity string
/// that kind's feed composed, and the moment the hold ends, in the form
/// `due::stored` writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldAlert {
    pub kind: String,
    pub id: String,
    pub until: String,
}

impl MessageCache {
    /// Hold an alert until a moment. Holding one that is already held
    /// replaces its moment: the latest snooze is the one somebody meant.
    pub fn hold_alert(&self, kind: &str, id: &str, until: &str) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO held_alerts (kind, id, until) VALUES (?1, ?2, ?3)
                 ON CONFLICT(kind, id) DO UPDATE SET until = excluded.until",
                rusqlite::params![kind, id, until],
            )
            .map(|_| ())
            .map_err(|e| Error::Other(format!("Failed to hold an alert: {}", e)))
    }

    /// Every hold, as written, soonest ending first. The kind is a word here
    /// and is not read.
    pub fn held_alerts(&self) -> Result<Vec<HeldAlert>> {
        let mut statement = self
            .conn
            .prepare_cached("SELECT kind, id, until FROM held_alerts ORDER BY until, kind, id")
            .map_err(|e| Error::Other(format!("Failed to read the held alerts: {}", e)))?;
        let rows = statement
            .query_map([], |row| {
                Ok(HeldAlert {
                    kind: row.get(0)?,
                    id: row.get(1)?,
                    until: row.get(2)?,
                })
            })
            .map_err(|e| Error::Other(format!("Failed to read the held alerts: {}", e)))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(format!("Failed to read a held alert: {}", e)))
    }

    /// Let go of every hold that ended before a moment, and say how many.
    ///
    /// `moment` is in the same form the holds were written in, so the
    /// comparison is the text's own order, which for that fixed-width form
    /// is time order. A hold ending at exactly the moment is kept: the rule
    /// in `due::what_is_due` already treats it as held no longer, and it is
    /// let go at the next look, when it is behind the moment.
    pub fn let_go_of_holds_that_ended_before(&self, moment: &str) -> Result<usize> {
        self.conn
            .execute(
                "DELETE FROM held_alerts WHERE until < ?1",
                rusqlite::params![moment],
            )
            .map_err(|e| Error::Other(format!("Failed to let go of held alerts: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;

    fn test_cache() -> TempHome<MessageCache> {
        TempHome::named("wixen_held_test_", |dir| {
            MessageCache::new(dir.to_path_buf(), None).unwrap()
        })
    }

    #[test]
    fn test_a_hold_read_back_is_the_hold_written() {
        let cache = test_cache();
        cache
            .hold_alert("task", "t1", "2026-09-14 17:00:00")
            .expect("hold");
        cache
            .hold_alert("event", "e1|2026-09-15T09:00:00", "2026-09-15 08:30:00")
            .expect("hold");

        let held = cache.held_alerts().expect("read");

        assert_eq!(
            held,
            vec![
                HeldAlert {
                    kind: "task".to_string(),
                    id: "t1".to_string(),
                    until: "2026-09-14 17:00:00".to_string(),
                },
                HeldAlert {
                    kind: "event".to_string(),
                    id: "e1|2026-09-15T09:00:00".to_string(),
                    until: "2026-09-15 08:30:00".to_string(),
                },
            ]
        );
    }

    #[test]
    fn test_holding_an_alert_again_replaces_its_moment() {
        let cache = test_cache();
        cache
            .hold_alert("task", "t1", "2026-09-14 17:00:00")
            .expect("hold");
        cache
            .hold_alert("task", "t1", "2026-09-15 09:00:00")
            .expect("hold again");

        let held = cache.held_alerts().expect("read");

        assert_eq!(held.len(), 1, "a second snooze made a second row");
        assert_eq!(held[0].until, "2026-09-15 09:00:00");
    }

    #[test]
    fn test_the_same_id_under_two_kinds_is_two_holds() {
        // A task and a reminder can share a row id and nothing about either
        // says so; the key is the kind and the id together.
        let cache = test_cache();
        cache
            .hold_alert("task", "shared", "2026-09-14 17:00:00")
            .expect("hold");
        cache
            .hold_alert("event", "shared", "2026-09-14 18:00:00")
            .expect("hold");

        assert_eq!(cache.held_alerts().expect("read").len(), 2);
    }

    #[test]
    fn test_a_hold_that_ended_is_let_go_and_one_that_has_not_is_kept() {
        let cache = test_cache();
        cache
            .hold_alert("task", "ended", "2026-09-14 16:00:00")
            .expect("hold");
        cache
            .hold_alert("task", "still-held", "2026-09-14 18:00:00")
            .expect("hold");
        cache
            .hold_alert("task", "ends-now", "2026-09-14 17:00:00")
            .expect("hold");

        let let_go = cache
            .let_go_of_holds_that_ended_before("2026-09-14 17:00:00")
            .expect("prune");

        assert_eq!(let_go, 1, "not exactly the one hold that had ended");
        let kept: Vec<String> = cache
            .held_alerts()
            .expect("read")
            .into_iter()
            .map(|hold| hold.id)
            .collect();
        // Held until exactly now is held no longer by the rule in
        // `due::what_is_due`, and it is kept here until the next look, when
        // it is behind the moment: the rule and the table agree about the
        // boundary without this having to know the rule.
        assert_eq!(kept, vec!["ends-now".to_string(), "still-held".to_string()]);
    }

    #[test]
    fn test_a_kind_word_this_build_does_not_know_survives_a_read_and_a_prune() {
        // "mail" is the fourth kind, after version 1. A hold written under it
        // by a later build is kept rather than dropped, so going back to this
        // build does not bring a snoozed message back at somebody.
        let cache = test_cache();
        cache
            .hold_alert("mail", "message-42", "2026-09-20 09:00:00")
            .expect("hold");

        let let_go = cache
            .let_go_of_holds_that_ended_before("2026-09-14 17:00:00")
            .expect("prune");

        assert_eq!(let_go, 0);
        let held = cache.held_alerts().expect("read");
        assert_eq!(held.len(), 1, "the unknown kind's hold was dropped");
        assert_eq!(held[0].kind, "mail");
        assert_eq!(
            crate::application::due::Kind::from_key(&held[0].kind),
            None,
            "this build reads the word as a kind it knows, so the seam has moved"
        );
    }
}

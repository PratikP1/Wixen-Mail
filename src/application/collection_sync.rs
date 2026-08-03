//! Working out what changed when a manager dialog hands back a whole list.
//!
//! The tag, signature, filter, contact and calendar dialogs all take the
//! current set, let the user add, edit and remove rows, and return the set they
//! ended up with. Saving that means three things: delete what is gone, write
//! what is new or changed, and leave the rest alone.
//!
//! Doing it by hand in five places is five chances to forget the delete, which
//! is the mistake that leaves a record the user removed still sitting in the
//! database and reappearing next time the dialog opens. So it is one function
//! over an identity, tested once.

use std::collections::HashSet;

/// What to do to the store to make it match what the dialog returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Changes<T> {
    /// Identities present before and gone now.
    pub removed: Vec<String>,
    /// Rows to write, whether new or edited.
    ///
    /// Everything the dialog returned, not only what changed. Working out
    /// which rows are untouched needs the records to be comparable, and an
    /// idempotent write of a row that did not change costs nothing next to
    /// getting it wrong.
    pub written: Vec<T>,
}

impl<T> Changes<T> {
    /// Whether anything at all needs saving.
    pub fn is_empty(&self) -> bool {
        self.removed.is_empty() && self.written.is_empty()
    }
}

/// Compare what was loaded with what the dialog returned.
///
/// `identity` pulls the stable key out of a record. A record whose key is empty
/// is treated as new: dialogs create rows before they have been saved, and an
/// empty key must never match an existing row and delete it by accident.
pub fn changes_between<T, U, F, G>(
    before: &[T],
    after: Vec<U>,
    id_of: F,
    id_of_new: G,
) -> Changes<U>
where
    F: Fn(&T) -> String,
    G: Fn(&U) -> String,
{
    let kept: HashSet<String> = after
        .iter()
        .map(&id_of_new)
        .filter(|id| !id.trim().is_empty())
        .collect();

    let removed = before
        .iter()
        .map(&id_of)
        .filter(|id| !id.trim().is_empty() && !kept.contains(id))
        .collect();

    Changes {
        removed,
        written: after,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Row {
        id: String,
        name: String,
    }

    fn row(id: &str, name: &str) -> Row {
        Row {
            id: id.to_string(),
            name: name.to_string(),
        }
    }

    fn diff(before: &[Row], after: Vec<Row>) -> Changes<Row> {
        changes_between(before, after, |r| r.id.clone(), |r| r.id.clone())
    }

    #[test]
    fn test_a_row_the_user_removed_is_deleted() {
        // The mistake this function exists to prevent: a record the user
        // deleted staying in the database and coming back next time.
        let before = vec![row("a", "Work"), row("b", "Personal")];
        let changes = diff(&before, vec![row("a", "Work")]);
        assert_eq!(changes.removed, vec!["b".to_string()]);
    }

    #[test]
    fn test_an_edited_row_is_written_and_not_deleted() {
        let before = vec![row("a", "Work")];
        let changes = diff(&before, vec![row("a", "Office")]);
        assert!(changes.removed.is_empty());
        assert_eq!(changes.written, vec![row("a", "Office")]);
    }

    #[test]
    fn test_a_new_row_with_no_id_yet_does_not_delete_anything() {
        // A dialog creates a row before it has been saved, so its key is
        // empty. An empty key matching an existing row would delete it.
        let before = vec![row("a", "Work")];
        let changes = diff(&before, vec![row("a", "Work"), row("", "Brand new")]);
        assert!(
            changes.removed.is_empty(),
            "an unsaved row deleted something"
        );
        assert_eq!(changes.written.len(), 2);
    }

    #[test]
    fn test_clearing_the_list_removes_everything_that_was_there() {
        let before = vec![row("a", "Work"), row("b", "Personal")];
        let changes = diff(&before, Vec::new());
        assert_eq!(changes.removed.len(), 2);
        assert!(changes.written.is_empty());
    }

    #[test]
    fn test_an_unchanged_list_removes_nothing() {
        let before = vec![row("a", "Work"), row("b", "Personal")];
        let changes = diff(&before, before.clone());
        assert!(changes.removed.is_empty());
        assert_eq!(changes.written.len(), 2);
    }

    #[test]
    fn test_a_stored_row_with_a_blank_id_is_never_deleted() {
        // It cannot be addressed, so a delete would either do nothing or hit
        // the wrong row. Leaving it is the safer of two bad options, and it is
        // a state the application should not be able to reach anyway.
        let before = vec![row("", "Corrupt"), row("a", "Work")];
        let changes = diff(&before, vec![row("a", "Work")]);
        assert!(changes.removed.is_empty());
    }

    #[test]
    fn test_nothing_to_do_is_reported_as_nothing_to_do() {
        let changes: Changes<Row> = diff(&[], Vec::new());
        assert!(changes.is_empty());
    }

    #[test]
    fn test_a_list_with_a_row_to_delete_has_something_to_save() {
        // The half that is easy to lose. Nothing left to write looks like
        // nothing to do, and the delete is skipped, which is the record the
        // user removed still sitting in the database.
        let changes = diff(&[row("a", "Work")], Vec::new());

        assert!(
            !changes.is_empty(),
            "a deletion was reported as nothing to do"
        );
    }

    #[test]
    fn test_a_list_with_a_row_to_write_has_something_to_save() {
        let changes = diff(&[], vec![row("a", "Work")]);

        assert!(
            !changes.is_empty(),
            "a new row was reported as nothing to do"
        );
    }

    #[test]
    fn test_identity_can_differ_between_the_two_sides() {
        // The dialogs hand back their own row type, not the stored one, so the
        // two keys are read by different functions.
        struct Stored {
            key: String,
        }
        let before = vec![
            Stored {
                key: "a".to_string(),
            },
            Stored {
                key: "b".to_string(),
            },
        ];
        let changes = changes_between(
            &before,
            vec![row("a", "Work")],
            |s| s.key.clone(),
            |r| r.id.clone(),
        );
        assert_eq!(changes.removed, vec!["b".to_string()]);
    }
}

//! What the message list should do once the server has answered.
//!
//! The twin of [`crate::application::local_delete`], which answers the same
//! question for a message that lives on this computer.
//!
//! Deleting and moving a message at a server can end several ways, and two of
//! them leave the message exactly where it was. The handlers used to read every
//! answer that was not a failure as "it left the folder", take the row out of
//! the list and say so. On a server that copied the message and then refused to
//! remove the original, the list said one thing and the server said another,
//! and the only way anybody found out was from another device.
//!
//! # Why marking the row deleted is the right mirror
//!
//! A row marked deleted here is what a message flagged for removal at the
//! server syncs back as, so the two agree by construction. Every outcome where
//! the server either no longer has the message in this folder or has it flagged
//! is marked here. The two where the original is untouched and unflagged leave
//! the row exactly as it is, because taking it out would be the list saying
//! something the server does not.

use crate::service::protocols::imap::{Deletion, Moved};

/// What the list should do with the row now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThenWhat {
    /// Nothing on the server changed for this row, so it stays as it is.
    LeaveTheRow,
    /// The server has it flagged for removal, or no longer has it in this
    /// folder. Marking it deleted here is what agrees with that.
    MarkItDeletedHere,
}

/// The row and the sentence, decided together so they cannot disagree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatToDoNext {
    pub then: ThenWhat,
    pub said: String,
}

/// What to do and what to say after the server answered a delete.
pub fn after_a_delete(deletion: &Deletion, subject: &str) -> WhatToDoNext {
    let then = match deletion {
        // The original is still in the folder and carries no mark, so the row
        // stays. A list that dropped it would be claiming something the server
        // has not done.
        Deletion::CopiedToTrashAndNotFlagged(_) => ThenWhat::LeaveTheRow,
        Deletion::MovedToTrash
        | Deletion::Removed
        | Deletion::CopiedToTrashAndFlagged(_)
        | Deletion::MarkedOnly(_) => ThenWhat::MarkItDeletedHere,
    };
    WhatToDoNext {
        then,
        said: format!("{}: {subject}", deletion.spoken()),
    }
}

/// What to do and what to say after the server answered a move.
pub fn after_a_move(moved: &Moved, into: &str, subject: &str) -> WhatToDoNext {
    let then = match moved {
        Moved::CopiedAndNotFlagged(_) => ThenWhat::LeaveTheRow,
        Moved::Moved | Moved::CopiedAndFlagged(_) => ThenWhat::MarkItDeletedHere,
    };
    WhatToDoNext {
        then,
        said: format!("{}: {subject}", moved.spoken(into)),
    }
}

/// What to say when the server would not take the change at all.
///
/// Nothing reached the server, so nothing was deleted and nothing was put back.
/// Saying it was undone tells somebody a message came back that never went
/// anywhere, and there was a second sentence a few lines away in the delete
/// handler saying it differently again.
pub fn nothing_changed(reason: &str) -> String {
    format!("Nothing was deleted: {reason}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::protocols::imap::StillHere;

    fn refused() -> StillHere {
        StillHere::TheServerRefusedIt("over quota".to_string())
    }

    fn every_deletion() -> Vec<Deletion> {
        vec![
            Deletion::MovedToTrash,
            Deletion::Removed,
            Deletion::CopiedToTrashAndFlagged(StillHere::TheServerCannotRemoveOneMessage),
            Deletion::CopiedToTrashAndFlagged(refused()),
            Deletion::CopiedToTrashAndNotFlagged("over quota".to_string()),
            Deletion::MarkedOnly(StillHere::TheServerCannotRemoveOneMessage),
            Deletion::MarkedOnly(refused()),
        ]
    }

    fn every_move() -> Vec<Moved> {
        vec![
            Moved::Moved,
            Moved::CopiedAndFlagged(StillHere::TheServerCannotRemoveOneMessage),
            Moved::CopiedAndFlagged(refused()),
            Moved::CopiedAndNotFlagged("over quota".to_string()),
        ]
    }

    #[test]
    fn test_a_message_the_server_still_holds_unflagged_keeps_its_row() {
        // The copy landed and the original was not even marked. Taking the row
        // out would be the list saying the message left a folder it is still
        // sitting in, unflagged, which no later sync would correct.
        assert_eq!(
            after_a_delete(
                &Deletion::CopiedToTrashAndNotFlagged("over quota".to_string()),
                "Invoice",
            )
            .then,
            ThenWhat::LeaveTheRow
        );
        assert_eq!(
            after_a_move(
                &Moved::CopiedAndNotFlagged("over quota".to_string()),
                "Archive",
                "Invoice",
            )
            .then,
            ThenWhat::LeaveTheRow
        );
    }

    #[test]
    fn test_a_message_the_server_has_flagged_or_moved_leaves_the_list() {
        for deletion in every_deletion() {
            if matches!(deletion, Deletion::CopiedToTrashAndNotFlagged(_)) {
                continue;
            }
            assert_eq!(
                after_a_delete(&deletion, "Invoice").then,
                ThenWhat::MarkItDeletedHere,
                "{deletion:?}"
            );
        }
        for moved in every_move() {
            if matches!(moved, Moved::CopiedAndNotFlagged(_)) {
                continue;
            }
            assert_eq!(
                after_a_move(&moved, "Archive", "Invoice").then,
                ThenWhat::MarkItDeletedHere,
                "{moved:?}"
            );
        }
    }

    #[test]
    fn test_nothing_that_only_half_happened_is_announced_as_deleted() {
        let gone = after_a_delete(&Deletion::Removed, "Invoice").said;
        for deletion in every_deletion() {
            if deletion == Deletion::Removed {
                continue;
            }
            let said = after_a_delete(&deletion, "Invoice").said;
            assert!(!said.starts_with("Deleted"), "{deletion:?}: {said}");
            assert_ne!(said, gone, "{deletion:?}");
        }
    }

    #[test]
    fn test_a_copy_already_made_is_named_so_a_second_try_is_not_a_surprise() {
        // Nothing anywhere removes duplicates, so somebody who presses the key
        // again without being told ends up with three copies of the message.
        let deleted = after_a_delete(
            &Deletion::CopiedToTrashAndNotFlagged("over quota".to_string()),
            "Invoice",
        )
        .said;
        assert!(deleted.contains("Trash"), "{deleted}");
        assert!(deleted.to_lowercase().contains("still"), "{deleted}");

        let moved = after_a_move(
            &Moved::CopiedAndNotFlagged("over quota".to_string()),
            "Archive",
            "Invoice",
        )
        .said;
        assert!(moved.contains("Archive"), "{moved}");
        assert!(moved.to_lowercase().contains("still"), "{moved}");
    }

    #[test]
    fn test_every_outcome_says_something_and_no_two_say_the_same_thing() {
        let mut said: Vec<String> = every_deletion()
            .iter()
            .map(|d| after_a_delete(d, "Invoice").said)
            .collect();
        said.extend(
            every_move()
                .iter()
                .map(|m| after_a_move(m, "Archive", "Invoice").said),
        );

        for sentence in &said {
            assert!(!sentence.trim().is_empty());
            assert!(sentence.contains("Invoice"), "{sentence}");
        }
        let mut distinct = said.clone();
        distinct.sort();
        distinct.dedup();
        assert_eq!(distinct.len(), said.len(), "{said:?}");
    }

    #[test]
    fn test_the_sentence_for_a_change_that_never_happened_says_nothing_was_deleted() {
        let said = nothing_changed("the server said no");

        assert!(
            !said.to_lowercase().contains("undone"),
            "a delete that never happened was reported as undone: {said}"
        );
        assert!(said.contains("the server said no"), "{said}");
        assert!(
            said.to_lowercase().contains("nothing was deleted"),
            "{said}"
        );
    }
}

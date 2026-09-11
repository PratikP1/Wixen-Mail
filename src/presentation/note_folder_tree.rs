//! How the note folders are laid out for the tree that shows them.
//!
//! One backend container is one note folder, decided 2026-09-11 and written
//! into `docs/development/the-notes-seam.md`. So an account's note folders are
//! now two kinds of thing in one list: the ones a backend gave, which sync, and
//! the ones somebody made here, which stay on this computer and are never sent
//! anywhere. Two kinds of folder in one undifferentiated list is what this
//! avoids.
//!
//! # The words are mail's, and they are borrowed rather than invented
//!
//! Phase 1 put the mail folders that are on no server under a branch called
//! [`crate::application::local_folders::ON_THIS_COMPUTER`], and the New Item
//! destination announces the same place in the same words. A note folder
//! somebody made here belongs under that branch and under those words.
//! `local_folders`' own comment gives the reason there is one spelling: two
//! spellings of one place is how they come to disagree after somebody edits one
//! of them.
//!
//! # Why the decision is here and not in the window
//!
//! The tree itself is built in `wx_app`, which needs a running window to reach.
//! What goes in which branch, and what each row says, is a decision about a list
//! of folders, so it is answered here where it can be driven from a test. The
//! window walks the answer.
//!
//! # What this cannot say
//!
//! Nobody has heard it. Whether a screen reader user meets the branch, opens it
//! and understands that the folders under it go nowhere is a question for an
//! ear, and `.planning/WINDOWS.md` carries it as an unknown rather than as a
//! claim.

use super::ui_types::NoteFolderItem;
use crate::service::caldav::how_many;

/// The note folders, sorted into the two branches a screen shows.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct TheNotesTree {
    /// The folders a backend gave, at the top level in the order they arrived.
    ///
    /// At the top level rather than under a branch of their own, because they
    /// are the ordinary case: an account with a backend has these and usually
    /// nothing else, and a branch holding everything is a keystroke somebody
    /// pays on every visit for no information.
    pub from_a_backend: Vec<NoteFolderItem>,
    /// The folders somebody made here, which go under the borrowed branch.
    pub on_this_computer: Vec<NoteFolderItem>,
}

/// Sort an account's note folders into the branches they are shown in.
///
/// The order within each branch is the order given, which is the store's:
/// `display_order` then name, the same order the sidebar shows a calendar in.
pub fn the_notes_tree(folders: Vec<NoteFolderItem>) -> TheNotesTree {
    let (on_this_computer, from_a_backend) = folders
        .into_iter()
        .partition(|folder| folder.on_this_computer);
    TheNotesTree {
        from_a_backend,
        on_this_computer,
    }
}

/// What one folder's row says.
///
/// The name and how many notes are in it, which is what the row has always
/// said. Nothing here says where the folder came from: the branch it is under
/// says that, and a row that said it too would be the same fact twice for
/// somebody moving through by keyboard.
pub fn a_row_for(folder: &NoteFolderItem) -> String {
    format!("{} ({})", folder.name, folder.note_count)
}

/// What the branch holding the folders somebody made here says.
///
/// Mail's words and then what is under it, which is the shape
/// `folder_tree::group_text` gives a group heading: a group is not a folder, so
/// it holds nothing of its own and says what is beneath it while closed.
/// Somebody who never opens the branch still hears whether there is anything in
/// it.
pub fn what_the_local_branch_says(folders: &[NoteFolderItem]) -> String {
    let notes: usize = folders.iter().map(|folder| folder.note_count).sum();
    format!(
        "{}, {} in all",
        crate::application::local_folders::ON_THIS_COMPUTER,
        how_many(notes, "note")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_folder(id: &str, name: &str, notes: usize, on_this_computer: bool) -> NoteFolderItem {
        NoteFolderItem {
            id: id.to_string(),
            name: name.to_string(),
            note_count: notes,
            on_this_computer,
        }
    }

    #[test]
    fn test_a_folder_a_backend_gave_and_one_somebody_made_here_are_told_apart() {
        let tree = the_notes_tree(vec![
            a_folder("f1", "Work", 3, false),
            a_folder("f2", "Ideas", 1, true),
            a_folder("f3", "Home", 2, false),
        ]);

        assert_eq!(
            tree.from_a_backend
                .iter()
                .map(|folder| folder.name.as_str())
                .collect::<Vec<_>>(),
            ["Work", "Home"],
            "a folder a backend gave was put under the branch for ones made here"
        );
        assert_eq!(
            tree.on_this_computer
                .iter()
                .map(|folder| folder.name.as_str())
                .collect::<Vec<_>>(),
            ["Ideas"],
            "a folder somebody made here is shown as one a backend gave, so it \
             reads as somewhere notes are sent from"
        );
    }

    #[test]
    fn test_the_order_within_each_branch_is_the_order_it_was_given() {
        // The store's order, which is the order the sidebar shows a calendar
        // in. Sorting again here would put the folders in an order nothing else
        // in the program uses.
        let tree = the_notes_tree(vec![
            a_folder("f1", "Work", 0, false),
            a_folder("f2", "Admin", 0, false),
        ]);

        assert_eq!(
            tree.from_a_backend
                .iter()
                .map(|folder| folder.name.as_str())
                .collect::<Vec<_>>(),
            ["Work", "Admin"]
        );
    }

    #[test]
    fn test_a_row_says_the_folders_name_and_what_is_in_it() {
        assert_eq!(a_row_for(&a_folder("f1", "Work", 4, false)), "Work (4)");
        assert_eq!(a_row_for(&a_folder("f2", "Ideas", 0, true)), "Ideas (0)");
    }

    #[test]
    fn test_the_branch_for_folders_made_here_says_mails_own_words() {
        // Borrowed rather than written again. A second spelling of this place
        // is the drift `local_folders::ON_THIS_COMPUTER` exists to stop, and
        // comparing against the constant is what would notice one.
        let said = what_the_local_branch_says(&[
            a_folder("f1", "Ideas", 2, true),
            a_folder("f2", "Later", 1, true),
        ]);

        assert!(
            said.starts_with(crate::application::local_folders::ON_THIS_COMPUTER),
            "the branch does not open with the words mail already uses: {said}"
        );
        assert_eq!(said, "On this computer, 3 notes in all");
    }

    #[test]
    fn test_the_branch_says_what_is_under_it_even_when_it_is_one_or_none() {
        // Somebody who never opens the branch still hears whether there is
        // anything in it, and "1 notes" is the kind of thing an ear catches
        // straight away.
        assert_eq!(
            what_the_local_branch_says(&[a_folder("f1", "Ideas", 1, true)]),
            "On this computer, 1 note in all"
        );
        assert_eq!(
            what_the_local_branch_says(&[a_folder("f1", "Ideas", 0, true)]),
            "On this computer, 0 notes in all"
        );
    }
}

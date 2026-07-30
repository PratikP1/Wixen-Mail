//! Where a thing can be moved or copied to.
//!
//! One tree, built from the accounts that are set up and what each of them
//! holds: its mail folders, its calendars, its task lists, its note folders.
//! The same tree answers "which folder do I file this message in" and "which
//! list does this task belong on", so somebody learns it once.
//!
//! # Why a tree and not a list
//!
//! Two accounts can both have an Archive, and a flat list of folder names
//! makes those two rows that read identically. Under the account they belong
//! to, they are distinguishable by where they sit, which is how a screen
//! reader user tells them apart: the tree says the account when you move into
//! it and the folder when you move within it.
//!
//! # Why the shape is here rather than in the dialog
//!
//! So it can be tested. Which destinations are offered for a given thing is
//! the part that goes wrong: offering a task list as somewhere to put a
//! message, or offering the folder the message is already in.

use crate::application::new_item::ContainerKind;

/// What is being moved or copied, which decides what it can go into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Moving {
    /// A message, which goes into a mail folder.
    Message,
    /// Something that lives in one of our own containers.
    Item(ContainerKind),
}

/// One place in the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Destination {
    /// What it is called where it lives. Not the full path: the tree says the
    /// rest, and a row reading "Work / Archive / 2026" is a mouthful when
    /// every row above it already said "Work".
    pub name: String,
    /// What to hand back to the code doing the move.
    pub id: String,
    /// The account it belongs to.
    pub account_id: String,
    /// How deep, so the tree can be built without a second pass. Nought is a
    /// child of the account.
    pub depth: usize,
}

/// One account, and what it can hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    pub account_id: String,
    /// The address, which is what tells two accounts apart when read aloud.
    pub account_name: String,
    pub places: Vec<Destination>,
}

/// The tree, with the places you cannot use taken out.
///
/// `already_in` is where the thing is now. It is removed, because offering
/// somebody the place a thing already is is offering them a command that
/// silently does nothing, and they will not know which of the two it was.
pub fn offer(branches: Vec<Branch>, already_in: Option<&str>) -> Vec<Branch> {
    branches
        .into_iter()
        .map(|mut branch| {
            branch
                .places
                .retain(|place| Some(place.id.as_str()) != already_in);
            branch
        })
        // An account with nowhere left to put it is not shown. An empty
        // branch is a row somebody opens, finds nothing in, and closes.
        .filter(|branch| !branch.places.is_empty())
        .collect()
}

/// Whether there is anywhere at all to put it.
///
/// Asked before the dialog opens, so "there is nowhere to move this to" is
/// said in a sentence rather than shown as an empty window.
pub fn anywhere(branches: &[Branch]) -> bool {
    branches.iter().any(|branch| !branch.places.is_empty())
}

/// What to say when there is nowhere to put it.
pub fn nothing_to_offer(moving: Moving) -> &'static str {
    match moving {
        Moving::Message => "There is no other folder to put this in",
        Moving::Item(ContainerKind::Calendar) => "There is no other calendar to put this in",
        Moving::Item(ContainerKind::TaskList) => "There is no other list to put this in",
        Moving::Item(ContainerKind::NoteFolder) => "There is no other folder to put this in",
        Moving::Item(ContainerKind::ContactGroup) => "There is no other group to put this in",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn place(id: &str, name: &str) -> Destination {
        Destination {
            name: name.to_string(),
            id: id.to_string(),
            account_id: "one".to_string(),
            depth: 0,
        }
    }

    fn one_account(places: Vec<Destination>) -> Vec<Branch> {
        vec![Branch {
            account_id: "one".to_string(),
            account_name: "me@example.com".to_string(),
            places,
        }]
    }

    #[test]
    fn test_the_place_it_is_already_in_is_not_offered() {
        // Otherwise it is a command that silently does nothing, and nobody
        // can tell that from one that failed.
        let tree = offer(
            one_account(vec![place("inbox", "Inbox"), place("archive", "Archive")]),
            Some("inbox"),
        );

        let names: Vec<&str> = tree[0].places.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["Archive"]);
    }

    #[test]
    fn test_an_account_with_nowhere_left_is_not_shown() {
        // An empty branch is a row somebody opens, finds nothing in, and
        // closes, having learnt nothing.
        let tree = offer(one_account(vec![place("inbox", "Inbox")]), Some("inbox"));

        assert!(tree.is_empty());
    }

    #[test]
    fn test_other_accounts_keep_their_places() {
        // Two accounts can both have an Archive, and removing the one you are
        // in must not remove the other account's.
        let mut two = one_account(vec![place("a-inbox", "Inbox")]);
        two.push(Branch {
            account_id: "two".to_string(),
            account_name: "work@example.com".to_string(),
            places: vec![Destination {
                name: "Inbox".to_string(),
                id: "b-inbox".to_string(),
                account_id: "two".to_string(),
                depth: 0,
            }],
        });

        let tree = offer(two, Some("a-inbox"));

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].account_name, "work@example.com");
    }

    #[test]
    fn test_nowhere_to_go_is_known_before_a_window_opens() {
        let nothing = offer(one_account(vec![place("inbox", "Inbox")]), Some("inbox"));

        assert!(!anywhere(&nothing));
        assert!(anywhere(&one_account(vec![place("inbox", "Inbox")])));
    }

    #[test]
    fn test_every_kind_has_something_to_say_when_there_is_nowhere() {
        // "Nothing happened" is the failure this avoids.
        assert!(!nothing_to_offer(Moving::Message).is_empty());
        for kind in ContainerKind::ALL {
            let said = nothing_to_offer(Moving::Item(kind));
            assert!(!said.is_empty(), "{kind:?}");
            assert!(said.contains("no other"), "{kind:?}: {said}");
        }
    }

    #[test]
    fn test_nothing_is_removed_when_it_is_not_in_anything_yet() {
        let tree = offer(
            one_account(vec![place("inbox", "Inbox"), place("archive", "Archive")]),
            None,
        );

        assert_eq!(tree[0].places.len(), 2);
    }
}

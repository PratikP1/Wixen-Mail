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
use crate::common::types::FolderType;

/// Which delete somebody asked for.
///
/// Two words rather than a `bool`, because both calls read the same at the call
/// site and only one of them can be taken back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deleting {
    /// The ordinary one, on `Delete`. Recoverable from the trash.
    ToTrash,
    /// `Shift+Delete`. Gone from the server, with no copy anywhere.
    Outright,
}

/// What the ordinary delete means for this message on this account.
///
/// Four answers rather than a folder or nothing, because "there is nowhere to
/// move it to" and "I do not know where this account's trash is" used to be the
/// same answer, and that answer removed the message from the server for good.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeletedGoesTo<'a> {
    /// Move it here. Recoverable from there, on every device.
    TheTrash(&'a str),
    /// Take it off the server. Asked for outright, or it is already in the
    /// trash and deleting it a second time means it.
    OffTheServer,
    /// This account has folders and none of them is its trash, so nothing is
    /// deleted and [`NO_TRASH_FOLDER_FOUND`] says what to do instead.
    NoTrashFolderFound,
    /// This account has no folders yet, so nothing is known about its trash.
    NoFoldersKnownYet,
}

/// What to say when no folder on the account reads as its trash.
///
/// Nothing was deleted, and both ways forward are named. A server that calls
/// its trash Papierkorb, Corbeille or something a provider invented is enough
/// to get here, and none of that is the person's fault or their problem to
/// diagnose.
pub const NO_TRASH_FOLDER_FOUND: &str = "Nothing was deleted. This account does not say which of its folders it keeps deleted mail \
     in, so there is nowhere to move this to. It is still where it was. Move it to the folder \
     this account keeps deleted mail in, or use Delete Permanently to take it off the server \
     for good.";

/// What to say when the account has never been asked what folders it has.
///
/// A new account that has never checked for mail knows nothing about its
/// folders, and reading that as "it has no trash" is the worst possible reading
/// of not knowing yet.
pub const NO_FOLDERS_KNOWN_YET: &str = "Nothing was deleted. This account has not learned what folders it has yet. Check for mail \
     once, and Delete will move messages to the Trash from then on.";

/// Where a deleted message should go, if anywhere.
///
/// The ordinary delete moves to the trash. It is what every other client does,
/// it is recoverable, and it is the only behaviour that means the same thing on
/// every server: flagging a message and expunging it in place removes a label
/// on Gmail, and which of three things that turns into depends on a setting
/// only reachable in Gmail's own web interface.
///
/// The order of the decisions matters. Asking for it outright is answered
/// before the folder list is looked at, so Delete Permanently keeps working on
/// an account that has never synced. Then not knowing the folders, then knowing
/// them and finding no trash among them, are two different refusals, because
/// what to do next is different for each.
pub fn where_a_deleted_message_goes<'a>(
    folders: impl IntoIterator<Item = (&'a str, FolderType)>,
    deleting_from: &str,
    asked: Deleting,
) -> DeletedGoesTo<'a> {
    if asked == Deleting::Outright {
        return DeletedGoesTo::OffTheServer;
    }
    let mut folders = folders.into_iter().peekable();
    if folders.peek().is_none() {
        return DeletedGoesTo::NoFoldersKnownYet;
    }
    let Some(trash) = folders
        .find(|(_, kind)| *kind == FolderType::Trash)
        .map(|(path, _)| path)
    else {
        return DeletedGoesTo::NoTrashFolderFound;
    };
    if trash == deleting_from {
        return DeletedGoesTo::OffTheServer;
    }
    DeletedGoesTo::TheTrash(trash)
}

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

/// Which destination the window should open on.
///
/// Filing mail is repetitive: twenty messages go into the same folder one after
/// another, and walking the tree to it twenty times is nineteen more journeys
/// than anybody wants. So the window opens on wherever the last one went, and
/// filing the next is the shortcut and Enter.
///
/// The last one is only used if it is still on offer. It will not be when the
/// message is already in it, which is the ordinary case of moving something
/// back out of the folder it was just put in, and a window that opened on a row
/// which is not there would open on nothing.
pub fn open_on<'a>(branches: &'a [Branch], last_used: Option<&str>) -> Option<&'a Destination> {
    if let Some(last) = last_used
        && let Some(again) = branches
            .iter()
            .flat_map(|branch| branch.places.iter())
            .find(|place| place.id == last)
    {
        return Some(again);
    }
    branches
        .iter()
        .flat_map(|branch| branch.places.iter())
        .next()
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
    fn test_the_window_opens_on_where_the_last_one_went() {
        // Filing is repetitive. Twenty messages into the same folder should
        // not be twenty walks through the tree.
        let tree = one_account(vec![
            place("inbox", "Inbox"),
            place("archive", "Archive"),
            place("work", "Work"),
        ]);

        let opens = open_on(&tree, Some("work")).expect("a destination");

        assert_eq!(opens.id, "work");
    }

    #[test]
    fn test_a_last_destination_that_is_not_offered_is_not_used() {
        // Which is the ordinary case of moving something back out of the
        // folder it was just filed into: that folder is where the message is,
        // so it is not on offer, and opening on it would open on nothing.
        let tree = one_account(vec![place("inbox", "Inbox"), place("archive", "Archive")]);

        let opens = open_on(&tree, Some("work")).expect("a destination");

        assert_eq!(opens.id, "inbox", "falls back to the first");
    }

    #[test]
    fn test_the_first_time_opens_on_the_first_place() {
        let tree = one_account(vec![place("inbox", "Inbox"), place("archive", "Archive")]);

        assert_eq!(open_on(&tree, None).expect("a destination").id, "inbox");
    }

    #[test]
    fn test_nothing_to_open_on_when_there_is_nowhere() {
        assert!(open_on(&[], Some("work")).is_none());
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

    fn mailboxes() -> [(&'static str, FolderType); 3] {
        [
            ("INBOX", FolderType::Inbox),
            ("[Gmail]/Trash", FolderType::Trash),
            ("Work", FolderType::Custom),
        ]
    }

    #[test]
    fn test_a_deleted_message_goes_to_the_trash() {
        assert_eq!(
            where_a_deleted_message_goes(mailboxes(), "INBOX", Deleting::ToTrash),
            DeletedGoesTo::TheTrash("[Gmail]/Trash")
        );
    }

    #[test]
    fn test_asking_to_delete_it_outright_does_not_go_to_the_trash() {
        // Shift+Delete. Somebody who means it should not have to go to the
        // trash and delete it a second time, which is what the ordinary
        // delete alone leaves them doing.
        assert_eq!(
            where_a_deleted_message_goes(mailboxes(), "INBOX", Deleting::Outright),
            DeletedGoesTo::OffTheServer
        );
    }

    #[test]
    fn test_deleting_from_the_trash_still_takes_it_off_the_server() {
        // Somebody emptying the trash means it. Moving a message from the
        // trash to the trash is a command that does nothing, and they cannot
        // tell that from one that failed.
        assert_eq!(
            where_a_deleted_message_goes(mailboxes(), "[Gmail]/Trash", Deleting::ToTrash),
            DeletedGoesTo::OffTheServer
        );
    }

    #[test]
    fn test_an_account_with_no_folder_that_is_the_trash_does_not_delete_the_message() {
        // This replaces a test that asserted the opposite. Deleting in place
        // on an account whose trash is not recognised destroys the only copy
        // of the message, and the program announced it as deleted, which it
        // was. A server naming its trash in another language, or a provider
        // naming it something of its own, was enough.
        let no_trash = [("INBOX", FolderType::Inbox), ("Work", FolderType::Custom)];

        assert_eq!(
            where_a_deleted_message_goes(no_trash, "INBOX", Deleting::ToTrash),
            DeletedGoesTo::NoTrashFolderFound
        );
    }

    #[test]
    fn test_an_account_that_has_not_learned_its_folders_yet_says_so_rather_than_deleting() {
        // A brand new account that has never checked for mail knows nothing
        // about its folders. Reading that as "there is no trash" and removing
        // the message is the worst possible reading of not knowing.
        assert_eq!(
            where_a_deleted_message_goes([], "INBOX", Deleting::ToTrash),
            DeletedGoesTo::NoFoldersKnownYet
        );
    }

    #[test]
    fn test_delete_permanently_still_works_on_an_account_whose_folders_are_unknown() {
        // Asking for it outright is answered before the folder list is
        // consulted at all, so the one command that says "I mean it" keeps
        // working when nothing is known about where anything is.
        assert_eq!(
            where_a_deleted_message_goes([], "INBOX", Deleting::Outright),
            DeletedGoesTo::OffTheServer
        );
    }

    #[test]
    fn test_each_refusal_says_what_to_do_next_and_names_no_machinery() {
        // Read aloud. A refusal that only says no leaves somebody with a key
        // that did nothing and no idea what to press instead.
        for said in [NO_TRASH_FOLDER_FOUND, NO_FOLDERS_KNOWN_YET] {
            assert!(!said.is_empty());
            for jargon in ["IMAP", "UID", "expunge", "folder type", "cache"] {
                assert!(
                    !said.to_lowercase().contains(&jargon.to_lowercase()),
                    "{jargon} is machinery, not something anybody hears: {said}"
                );
            }
        }
        assert_ne!(NO_TRASH_FOLDER_FOUND, NO_FOLDERS_KNOWN_YET);
        assert!(
            NO_TRASH_FOLDER_FOUND.contains("Delete Permanently"),
            "the other way forward is a menu item, and it has to be named: {NO_TRASH_FOLDER_FOUND}"
        );
        assert!(
            NO_FOLDERS_KNOWN_YET.contains("Check for mail"),
            "{NO_FOLDERS_KNOWN_YET}"
        );
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

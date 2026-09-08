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
    /// A mail folder, which goes inside another mail folder or comes out to
    /// the top level.
    ///
    /// A kind of its own rather than a flag beside `Message`, because this
    /// enum's job is to name what is being moved and that is what decides where
    /// it can go. A folder can contain a folder, which is the one rule a
    /// message move never needed: [`where_a_folder_can_go`] leaves out the
    /// folder itself and everything inside it, and nothing about a message
    /// needs that.
    Folder,
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
    ///
    /// Read by [`crate::presentation::wx_destination::build_destination_dialog`]
    /// since 04.1-01, and by nothing before it. This sentence and the one on
    /// `test_how_deep_each_row_is_says_where_it_sits` both said the field built
    /// the tree, and for as long as they said it every place was appended as a
    /// direct child of its account whatever its depth. A passing test on a
    /// field nothing read is what made them look checked.
    ///
    /// **A place attaches to the row above it**, so the places in a branch have
    /// to arrive in the order a walk down the tree meets them: a folder after
    /// the folder it is in, never before. Both producers here answer that way,
    /// and [`where_a_folder_can_go`] sorts for it rather than trusting the
    /// order the folders were stored in.
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

/// One folder, named the only way a folder can be named across accounts.
///
/// A path is unique inside one account and not across them: two accounts can
/// both have an `Archive`, which is the first sentence of this module's own
/// doc. So everything here that asks "which folder is this" asks for the pair,
/// and the field names say which half is which because the two are both
/// strings and swapping them compiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FolderInAnAccount<'a> {
    /// The account's identifier, not its name. A name is what somebody hears
    /// and two accounts can share one.
    pub account: &'a str,
    /// The path as the server spells it.
    pub path: &'a str,
}

/// The tree, with the places you cannot use taken out.
///
/// `already_in` is where the thing is now. It is removed, because offering
/// somebody the place a thing already is is offering them a command that
/// silently does nothing, and they will not know which of the two it was.
///
/// Removed from that account's branch only. Another account that happens to
/// hold a folder at the same path keeps it, because it is a different folder
/// on a different server and the message is not in it.
pub fn offer(branches: Vec<Branch>, already_in: Option<FolderInAnAccount<'_>>) -> Vec<Branch> {
    branches
        .into_iter()
        .map(|mut branch| {
            // The account read off the place rather than off the branch it is
            // in. Both say the same thing wherever a branch is built, and the
            // one the place carries is the one that travels back with the
            // answer, so it is the one an identity should be asked of.
            branch.places.retain(|place| {
                already_in
                    != Some(FolderInAnAccount {
                        account: place.account_id.as_str(),
                        path: place.id.as_str(),
                    })
            });
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
///
/// The account as well as the path, for the reason [`offer`] takes both: the
/// window opens on the folder somebody last filed into, in the account they
/// filed it into, rather than on whichever branch happens to hold a folder at
/// that path first.
pub fn open_on<'a>(
    branches: &'a [Branch],
    last_used: Option<FolderInAnAccount<'_>>,
) -> Option<&'a Destination> {
    if let Some(last) = last_used
        && let Some(again) = branches
            .iter()
            .flat_map(|branch| branch.places.iter())
            .find(|place| place.account_id == last.account && place.id == last.path)
    {
        return Some(again);
    }
    branches
        .iter()
        .flat_map(|branch| branch.places.iter())
        .next()
}

/// Every account's mail folders, as branches the picker can be given.
///
/// The picker's tree and the sidebar's tree are one hierarchy, and this is
/// where that is made true: it asks
/// [`crate::presentation::folder_tree::rows`] for the sidebar's own answer and
/// translates it. Nothing here decides which folders exist, which account owns
/// one, how deep it sits or what an account is called, because all four
/// already have an answer and a second one is a divergence waiting to happen.
/// A folder the sidebar shows and the picker does not, or the two disagreeing
/// about which account a folder belongs to, would be invisible until somebody
/// moved mail into the wrong place.
///
/// Application reaching into presentation is the arrangement this tree already
/// has in thirty-one places. The alternative is a second builder, which is the
/// thing being removed.
///
/// # What is translated rather than taken
///
/// Three things, and they are the whole of the difference between the two
/// views. The sidebar draws an account as a row, so its folders start one deep
/// and the picker's start at nought, where the account is the root. The
/// sidebar's row reads out with its unread count on the end; the picker's is
/// the bare name, because a destination is somewhere to put mail rather than
/// somewhere to read it. And the sidebar's identity is the account and the
/// path, which is what a [`Destination`] carries in two fields.
///
/// # What is left out, and why each
///
/// The rows that are not folders: Favourites, All Inboxes, Labels and the
/// saved searches. None of them is somewhere a message can be put. Nothing is
/// passed in for the first and the last two, so they do not arise, and All
/// Inboxes is dropped by naming the two kinds of row this reads.
///
/// The folders kept on this computer, which the sidebar draws under their own
/// heading rather than under an account. They belong to no account here, so
/// they match no branch and are not offered, which is what the picker has
/// always done.
///
/// # A folder the server has stopped listing
///
/// **Offered.** Decided here rather than left to be inferred later from what
/// the code does. `gone` is a fact about the server's last answer (D-27) and
/// not a verdict on the folder: it still holds its mail, the sidebar still
/// draws it, and somebody can still open it and read what is in it. A picker
/// that left it out would make the list saying where mail can go disagree with
/// the list saying where mail is, for a reason nobody could hear. If the
/// folder really has gone, the server refuses the command and says so, which
/// is a failure somebody can act on; a folder silently missing from the tree
/// is not.
pub fn where_mail_can_go(
    accounts: &[crate::presentation::folder_tree::AccountInTheTree],
    folders: &[crate::presentation::folder_tree::FolderInTheTree],
) -> Vec<Branch> {
    use crate::application::folder_settings::UnreadOnAParent;
    use crate::presentation::folder_tree::{WhichRow, rows};

    let mut branches: Vec<Branch> = Vec::with_capacity(accounts.len());
    // Nothing pinned, no labels, no saved searches, and nothing collapsed. The
    // first three are other kinds of row and the fourth only changes how a row
    // is worded, and the wording is the one thing here that is not used.
    let drawn = rows(
        accounts,
        folders,
        &[],
        &[],
        &[],
        UnreadOnAParent::default(),
        &std::collections::HashSet::new(),
    );
    for row in drawn {
        match row.identity {
            WhichRow::Account(id) => {
                // What the sidebar decided to call it, which is the label with
                // an address after it only where a second account reads the
                // same. Named here rather than looked up again, because a
                // second rule for when an address is read out is two accounts
                // called Work reading as one row in one of the two windows.
                let called = row.name;
                branches.push(Branch {
                    account_id: id,
                    account_name: called,
                    places: Vec::new(),
                });
            }
            WhichRow::Folder { account, path } => {
                let Some(branch) = branches
                    .iter_mut()
                    .find(|branch| branch.account_id == account)
                else {
                    continue;
                };
                branch.places.push(Destination {
                    name: row.name,
                    id: path,
                    account_id: account,
                    depth: row.depth.saturating_sub(1),
                });
            }
            _ => {}
        }
    }
    branches
}

/// Whose folders a move or copy is about.
///
/// The account the chosen message is in, not the account that happens to be
/// open. All Inboxes reads every account's inbox as one list, so those differ
/// routinely, and the two halves of a move used to answer this question
/// separately: the folders offered came from the account on screen and the
/// command went to the account the row belongs to. So the path was chosen on
/// one server and sent to another, where a mailbox of that name is a different
/// mailbox and a message that happens to share a UID is a different message.
///
/// Asked once here and passed to both halves, rather than each half deciding.
///
/// A row that records no account of its own falls back to the one on screen,
/// which is the ordinary case outside All Inboxes, where every row belongs to
/// the account being looked at. Whether the answer then names an account this
/// program knows is the caller's question, and the caller refuses rather than
/// reaching for another: the shape this replaces fell back to whichever account
/// came first in the list, so a command aimed at nothing in particular still
/// reached a real server.
pub fn whose_folders_a_move_is_about<'a>(
    the_message_is_in: Option<&'a str>,
    the_account_that_is_open: Option<&'a str>,
) -> Option<&'a str> {
    the_message_is_in
        .filter(|account| !account.is_empty())
        .or(the_account_that_is_open)
}

/// What to say when there is nowhere to put it.
pub fn nothing_to_offer(moving: Moving) -> &'static str {
    match moving {
        Moving::Message => "There is no other folder to put this in",
        Moving::Item(ContainerKind::Calendar) => "There is no other calendar to put this in",
        Moving::Item(ContainerKind::TaskList) => "There is no other list to put this in",
        Moving::Item(ContainerKind::NoteFolder) => "There is no other folder to put this in",
        Moving::Item(ContainerKind::ContactGroup) => "There is no other group to put this in",
        Moving::Folder => "There is nowhere else to put this folder",
    }
}

/// The places one folder can be moved to, within its own account.
///
/// Three things are left out, and each is a command that would otherwise be
/// offered and then do something nobody wants.
///
/// The folder itself, and everything inside it. A folder can contain a folder,
/// which is why this rule exists here and nowhere in a message move: a server
/// asked to put a folder inside its own subtree either refuses or does something
/// with no way back. The subtree comes from
/// [`crate::application::folders_underneath::deepest_first`], which is bounded,
/// so a cycle in stored parents cannot leave this walking while it holds the
/// window.
///
/// The folder it is already in, which is [`offer`]'s own rule: offering somebody
/// the place a thing already is offers them a command that silently does
/// nothing, and they cannot tell which of the two happened.
///
/// The top level is offered as a place in its own right, named by the empty
/// path, so a folder that went into another one can come back out. Without it
/// this is a one-way door. It is left out for a folder already at the top level,
/// by the same rule as above.
pub fn where_a_folder_can_go(
    folders: &[crate::application::folders_underneath::Placed],
    moving: i64,
    account_id: &str,
) -> Vec<Destination> {
    use crate::application::folders_underneath::deepest_first;

    let Some(folder) = folders.iter().find(|folder| folder.id == moving) else {
        return Vec::new();
    };
    let inside_it: Vec<i64> = deepest_first(folders, moving)
        .into_iter()
        .map(|under| under.id)
        .collect();

    // The top level first, so somebody arrowing down meets the way out before
    // the folders, and reads as a place rather than as an account.
    let mut places: Vec<Destination> = Vec::with_capacity(folders.len());
    if folder.parent.is_some() {
        places.push(Destination {
            name: "Not inside any folder".to_string(),
            id: String::new(),
            account_id: account_id.to_string(),
            depth: 0,
        });
    }

    // Parents before their children, which is what [`Destination::depth`]
    // requires and what the stored order does not give. Folders come back
    // `ORDER BY id`, which is the order they were first heard of, so `Archive`,
    // `Work`, `Archive/2026` is an ordinary stored order once somebody makes a
    // folder inside another one after making a folder beside it. Drawn in that
    // order, `Archive/2026` would hang under `Work`.
    //
    // Sorted on the chain of folders each one sits inside, which puts every
    // folder straight after the one it is in and leaves brothers and sisters in
    // the order they arrived. The sort is stable, so nothing else moves.
    let mut offered: Vec<(Vec<i64>, &crate::application::folders_underneath::Placed)> = folders
        .iter()
        .filter(|other| !inside_it.contains(&other.id))
        .filter(|other| Some(other.id) != folder.parent)
        .map(|other| (the_way_down_to(folders, other.id), other))
        .collect();
    offered.sort_by(|(one, _), (other, _)| one.cmp(other));

    places.extend(offered.into_iter().map(|(chain, other)| Destination {
        name: other.name.clone(),
        id: other.path.clone(),
        account_id: account_id.to_string(),
        depth: chain.len().saturating_sub(1),
    }));
    places
}

/// The folders this one sits inside, outermost first, ending with itself.
///
/// One walk answering two questions, because they are one question: how far in
/// a folder sits is how long this is, and the order a tree is drawn in is this
/// read as a sort key. Two walks would be two chances to disagree about what is
/// under what.
///
/// Bounded for the reason every walk over stored parents is: the column comes
/// from a database an earlier version wrote, and a walk that does not return
/// does not return while holding the window open. Past the bound the chain is
/// cut, which puts a folder in a cycle at the deepest a row can be drawn rather
/// than leaving the window still.
fn the_way_down_to(
    folders: &[crate::application::folders_underneath::Placed],
    of: i64,
) -> Vec<i64> {
    use crate::application::folders_underneath::AS_DEEP_AS_A_TREE_GOES;

    let mut chain = vec![of];
    let mut at = folders.iter().find(|folder| folder.id == of);
    while let Some(folder) = at {
        let Some(parent) = folder.parent else { break };
        chain.push(parent);
        if chain.len() >= AS_DEEP_AS_A_TREE_GOES {
            break;
        }
        at = folders.iter().find(|above| above.id == parent);
    }
    chain.reverse();
    chain
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

    /// A folder in the account these fixtures call `one`.
    fn in_one(path: &str) -> Option<FolderInAnAccount<'_>> {
        Some(FolderInAnAccount {
            account: "one",
            path,
        })
    }

    /// A folder in the second account, which is the account `two_accounts`
    /// adds.
    fn in_two(path: &str) -> Option<FolderInAnAccount<'_>> {
        Some(FolderInAnAccount {
            account: "two",
            path,
        })
    }

    /// Two accounts, each holding a folder at the path `Archive`.
    ///
    /// The shared path is the whole point of this fixture and the reason it
    /// replaced one that gave its two accounts `a-inbox` and `b-inbox`. A real
    /// IMAP path is not prefixed with an account, so a fixture whose two
    /// accounts cannot collide passes against a comparison that ignores the
    /// account entirely, which is exactly the defect these tests are about.
    ///
    /// The first account has an `INBOX` as well, so removing its `Archive`
    /// leaves it with somewhere to put things and the branch is not dropped
    /// for being empty. Without that, "the other account kept its Archive"
    /// and "both accounts lost theirs" are both one branch and the assertion
    /// cannot tell them apart.
    fn two_accounts() -> Vec<Branch> {
        vec![
            Branch {
                account_id: "one".to_string(),
                account_name: "me@example.com".to_string(),
                places: vec![
                    Destination {
                        name: "Inbox".to_string(),
                        id: "INBOX".to_string(),
                        account_id: "one".to_string(),
                        depth: 0,
                    },
                    Destination {
                        name: "Archive".to_string(),
                        id: "Archive".to_string(),
                        account_id: "one".to_string(),
                        depth: 0,
                    },
                ],
            },
            Branch {
                account_id: "two".to_string(),
                account_name: "work@example.com".to_string(),
                places: vec![Destination {
                    name: "Archive".to_string(),
                    id: "Archive".to_string(),
                    account_id: "two".to_string(),
                    depth: 0,
                }],
            },
        ]
    }

    #[test]
    fn test_the_place_it_is_already_in_is_not_offered() {
        // Otherwise it is a command that silently does nothing, and nobody
        // can tell that from one that failed.
        let tree = offer(
            one_account(vec![place("inbox", "Inbox"), place("archive", "Archive")]),
            in_one("inbox"),
        );

        let names: Vec<&str> = tree[0].places.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["Archive"]);
    }

    #[test]
    fn test_an_account_with_nowhere_left_is_not_shown() {
        // An empty branch is a row somebody opens, finds nothing in, and
        // closes, having learnt nothing.
        let tree = offer(one_account(vec![place("inbox", "Inbox")]), in_one("inbox"));

        assert!(tree.is_empty());
    }

    #[test]
    fn test_other_accounts_keep_their_places() {
        // Two accounts can both have an Archive, and removing the one you are
        // in must not remove the other account's.
        //
        // This test said that before and could not see it: its two accounts
        // held `a-inbox` and `b-inbox`, which cannot collide, so it passed
        // against an `offer` that ignored the account altogether.
        let tree = offer(two_accounts(), in_one("Archive"));

        assert_eq!(
            tree.len(),
            2,
            "the other account's Archive is a different folder on a different \
             server, so its branch is still somewhere the message can go: {tree:?}"
        );
        let theirs: Vec<&str> = tree[1].places.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(theirs, ["Archive"], "{tree:?}");
        let mine: Vec<&str> = tree[0].places.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(mine, ["INBOX"], "{tree:?}");
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

        let opens = open_on(&tree, in_one("work")).expect("a destination");

        assert_eq!(opens.id, "work");
    }

    #[test]
    fn test_the_window_opens_on_the_account_it_was_filed_into() {
        // Both accounts hold an `Archive`. Opening on the path alone opens on
        // whichever branch comes first, which is a window that says it is
        // about to file into the account somebody is looking at when they
        // named a different one.
        let tree = two_accounts();

        let opens = open_on(&tree, in_two("Archive")).expect("a destination");

        assert_eq!(opens.account_id, "two", "{opens:?}");
        assert_eq!(opens.id, "Archive");
    }

    #[test]
    fn test_a_last_destination_in_another_account_is_not_used() {
        // The remembered folder is in an account this window is not showing,
        // so it is not on offer here at all, whatever the first branch happens
        // to hold at that path. Opening on it would open on a row about
        // somewhere else.
        let tree = vec![Branch {
            account_id: "one".to_string(),
            account_name: "me@example.com".to_string(),
            places: vec![
                Destination {
                    name: "Inbox".to_string(),
                    id: "INBOX".to_string(),
                    account_id: "one".to_string(),
                    depth: 0,
                },
                Destination {
                    name: "Archive".to_string(),
                    id: "Archive".to_string(),
                    account_id: "one".to_string(),
                    depth: 0,
                },
            ],
        }];

        let opens = open_on(&tree, in_two("Archive")).expect("a destination");

        assert_eq!(opens.id, "INBOX", "falls back to the first: {opens:?}");
    }

    #[test]
    fn test_a_last_destination_that_is_not_offered_is_not_used() {
        // Which is the ordinary case of moving something back out of the
        // folder it was just filed into: that folder is where the message is,
        // so it is not on offer, and opening on it would open on nothing.
        let tree = one_account(vec![place("inbox", "Inbox"), place("archive", "Archive")]);

        let opens = open_on(&tree, in_one("work")).expect("a destination");

        assert_eq!(opens.id, "inbox", "falls back to the first");
    }

    #[test]
    fn test_the_first_time_opens_on_the_first_place() {
        let tree = one_account(vec![place("inbox", "Inbox"), place("archive", "Archive")]);

        assert_eq!(open_on(&tree, None).expect("a destination").id, "inbox");
    }

    #[test]
    fn test_nothing_to_open_on_when_there_is_nowhere() {
        assert!(open_on(&[], in_one("work")).is_none());
    }

    #[test]
    fn test_nowhere_to_go_is_known_before_a_window_opens() {
        let nothing = offer(one_account(vec![place("inbox", "Inbox")]), in_one("inbox"));

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
            None::<FolderInAnAccount<'_>>,
        );

        assert_eq!(tree[0].places.len(), 2);
    }

    // ── Where a folder can go ───────────────────────────────────────────────

    mod moving_a_folder {
        use super::*;
        use crate::application::folders_underneath::Placed;

        fn placed(id: i64, path: &str, parent: Option<i64>) -> Placed {
            Placed {
                id,
                name: path.rsplit('/').next().unwrap_or(path).to_string(),
                path: path.to_string(),
                parent,
            }
        }

        /// `Archive`, with `Archive/2026` in it and `Archive/2026/June` in that,
        /// beside `Work` and `Old`.
        fn an_account() -> Vec<Placed> {
            vec![
                placed(1, "Archive", None),
                placed(2, "Archive/2026", Some(1)),
                placed(3, "Archive/2026/June", Some(2)),
                placed(4, "Work", None),
                placed(5, "Old", None),
            ]
        }

        fn offered(folders: &[Placed], moving: i64) -> Vec<String> {
            where_a_folder_can_go(folders, moving, "acc")
                .into_iter()
                .map(|place| place.id)
                .collect()
        }

        #[test]
        fn test_a_folder_is_not_offered_a_place_inside_itself() {
            // The rule a message move never needed, because a message cannot
            // contain a folder. A folder can, and a server asked to move one
            // into itself either refuses or does something nobody can undo.
            let places = offered(&an_account(), 1);
            assert!(!places.contains(&"Archive".to_string()), "{places:?}");
        }

        #[test]
        fn test_a_folder_is_not_offered_a_place_inside_its_own_subtree() {
            // `Archive` under `Archive/2026` is the same fault one level down,
            // and it is the one somebody actually reaches by arrowing.
            let places = offered(&an_account(), 1);
            assert!(!places.contains(&"Archive/2026".to_string()), "{places:?}");
            assert!(
                !places.contains(&"Archive/2026/June".to_string()),
                "{places:?}"
            );
        }

        #[test]
        fn test_the_folder_it_is_already_in_is_not_offered() {
            // `offer`'s own rule: offering somebody the place a thing already
            // is offers them a command that silently does nothing.
            let places = offered(&an_account(), 2);
            assert!(!places.contains(&"Archive".to_string()), "{places:?}");
        }

        #[test]
        fn test_everything_else_in_the_account_is_offered() {
            let places = offered(&an_account(), 1);
            assert!(places.contains(&"Work".to_string()), "{places:?}");
            assert!(places.contains(&"Old".to_string()), "{places:?}");
        }

        #[test]
        fn test_a_folder_inside_one_is_offered_the_top_level_to_come_out_to() {
            // Without this a folder goes in and never comes out, which is the
            // kind of one-way door that makes a feature worse than not having
            // it. The top level is named by the empty path, which is what a
            // folder's path is when nothing is in front of it.
            let places = offered(&an_account(), 2);
            assert!(places.contains(&String::new()), "{places:?}");
        }

        #[test]
        fn test_a_folder_already_at_the_top_level_is_not_offered_it_again() {
            let places = offered(&an_account(), 1);
            assert!(!places.contains(&String::new()), "{places:?}");
        }

        #[test]
        fn test_how_deep_each_row_is_says_where_it_sits() {
            // `Destination.depth` is what builds the tree in the window
            // without a second pass, and nought is a child of the account.
            //
            // That sentence was false for as long as it stood here: nothing in
            // the program read the field, and the window drew every place as a
            // direct child of its account. This test passed throughout, which
            // is what made the claim look checked. What the window does with
            // the number is checked in
            // `tests/tree_dialogs_resolve_the_row_somebody_is_on.rs`, against a
            // live control, because only a live control can say which of a
            // tree and a list was built.
            let places = where_a_folder_can_go(&an_account(), 4, "acc");
            let archive = places
                .iter()
                .find(|place| place.id == "Archive")
                .expect("Archive is offered");
            let inside = places
                .iter()
                .find(|place| place.id == "Archive/2026")
                .expect("the folder inside it is offered");
            assert_eq!(archive.depth, 0);
            assert_eq!(inside.depth, 1);
        }

        #[test]
        fn test_a_row_reads_out_as_a_name_and_not_as_a_path() {
            let places = where_a_folder_can_go(&an_account(), 4, "acc");
            let inside = places
                .iter()
                .find(|place| place.id == "Archive/2026")
                .expect("the folder inside it is offered");
            assert_eq!(inside.name, "2026");
        }

        #[test]
        fn test_a_stored_cycle_offers_something_rather_than_hanging() {
            // Parents come from a database an earlier version wrote, so a
            // cycle is not hypothetical, and a walk that does not return does
            // not return while holding the window.
            let tree = vec![
                placed(1, "A", Some(2)),
                placed(2, "B", Some(1)),
                placed(3, "C", None),
            ];

            let places = where_a_folder_can_go(&tree, 1, "acc");

            assert!(places.iter().any(|place| place.id == "C"), "{places:?}");
        }

        #[test]
        fn test_a_folder_that_is_not_there_is_offered_nothing() {
            assert!(where_a_folder_can_go(&an_account(), 99, "acc").is_empty());
        }

        #[test]
        fn test_there_is_a_sentence_for_a_folder_with_nowhere_to_go() {
            let said = nothing_to_offer(Moving::Folder);
            assert!(said.contains("folder"), "{said}");
            assert_ne!(said, nothing_to_offer(Moving::Message));
        }
    }
}

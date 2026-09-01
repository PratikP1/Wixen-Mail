//! What the menu key offers, for whatever has focus.
//!
//! The Applications key, or `Shift+F10`, on a message, a task, a folder or a
//! calendar. For somebody who cannot see a toolbar it is the main way to find
//! out what can be done with the thing they are on, without leaving it to go
//! hunting through the menu bar and then finding their way back.
//!
//! # Only commands that work
//!
//! Everything here maps to something the application already does. Nothing is
//! listed because it ought to exist: a menu entry that does nothing is worse
//! than one that is absent, because it is a stop somebody lands on, hears, and
//! learns nothing from, and it costs a moment every time it is passed. That
//! rule is already written down for [`crate::application::pim_command`] and
//! this follows it.
//!
//! Marking a whole folder read and emptying a folder are the obvious
//! absences. Neither is implemented, so neither is offered. Rename was in that
//! list and is now written for a contact group, which is the only container
//! that has one; the other three still have no rename and are still not
//! offered it, and
//! [`crate::application::new_item::renaming_works`] is where that is decided.
//!
//! # Why this is data
//!
//! The same reason as [`crate::application::item_fields`]. Ten hand-written
//! menus drift, and the drift is invisible: one loses a mnemonic, one offers
//! something the panel cannot do, one gets its entries in a different order
//! from its neighbour. Described here and built by one function, the only
//! thing that differs between them is the list, and the list can be tested.

use crate::application::new_item::{ContainerKind, ItemKind};

/// What has focus when the menu key is pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    /// The list of messages.
    Messages,
    /// The tree of mail folders.
    MailFolders,
    /// A saved search's own row in that tree.
    ///
    /// A separate place rather than a second list for the same one, because
    /// the commands really are different: a saved search is not a folder, it
    /// has no older messages to fetch and nothing to keep up to date, and it
    /// has three commands of its own that no folder has. The menu bar already
    /// models it this way, with a Saved Search menu beside This Folder.
    ///
    /// Which of the two the tree reports is decided when the menu key is
    /// pressed, from the row the cursor is on.
    SavedSearch,
    /// The list in one of the personal information modules.
    Items(ItemKind),
    /// The sidebar tree of one of them: calendars, task lists, folders.
    Containers(ContainerKind),
}

impl Focus {
    /// Every place a menu can be raised, so tests cover the whole set.
    pub const ALL: [Focus; 13] = [
        Focus::Messages,
        Focus::MailFolders,
        Focus::SavedSearch,
        Focus::Items(ItemKind::Contact),
        Focus::Items(ItemKind::Event),
        Focus::Items(ItemKind::Reminder),
        Focus::Items(ItemKind::Task),
        Focus::Items(ItemKind::Note),
        Focus::Containers(ContainerKind::Calendar),
        Focus::Containers(ContainerKind::TaskList),
        Focus::Containers(ContainerKind::NoteFolder),
        Focus::Containers(ContainerKind::ContactGroup),
        Focus::Items(ItemKind::Mail),
    ];
}

/// What an entry does.
///
/// Named for the effect rather than for the control that used to be the only
/// way to reach it, so the same action can sit on a menu bar, a button and a
/// context menu without three names for one thing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Reply,
    ReplyAll,
    Forward,
    MarkRead,
    ToggleStar,
    DeleteMessage,
    /// Remove it from the server without putting it in the trash.
    DeleteMessageOutright,
    RefreshFolder,
    GetOlder,
    /// Make another one of whatever this list holds.
    NewItem,
    /// Remove the chosen row. Always confirmed.
    DeleteItem,
    /// Done or not done, for tasks and reminders.
    ToggleComplete,
    /// Pinned or not, for notes.
    TogglePin,
    /// Make another container of this kind.
    NewContainer,
    /// Remove this container and say what goes with it.
    DeleteContainer,
    /// Give this container a different name.
    ///
    /// Offered only where a rename is written, which today is a contact group.
    /// [`crate::application::new_item::renaming_works`] is the one answer, and
    /// a test holds this list to it.
    RenameContainer,
    /// Open a message addressed to everybody in this group.
    ///
    /// The reason anybody keeps a group. Without it a group is a name in a
    /// sidebar and nothing more.
    WriteToGroup,
    /// Put the chosen contact in a group.
    AddToGroup,
    /// Take the chosen contact out of a group, leaving the contact alone.
    ///
    /// Named apart from [`Action::DeleteItem`] because they are one keystroke
    /// apart and one of them is not reversible.
    RemoveFromGroup,
    /// Fetch this module from the provider now.
    SyncNow,
    /// Make a task from this message, keeping its subject and its text.
    CopyToTask,
    /// Make a calendar event from it.
    CopyToEvent,
    /// Make a note from it.
    CopyToNote,
    /// Put this message in another folder, and take it out of this one.
    MoveToFolder,
    /// Put this event, task or note in another calendar, list or folder.
    ///
    /// Separate from [`Action::MoveToFolder`] because what is offered differs:
    /// a message goes to a mail folder, and an item goes to a container of the
    /// one kind that holds it. Offering a calendar as a home for a note is not
    /// a mistake worth making reachable.
    ///
    /// Not offered for a contact. A contact is in as many groups as somebody
    /// puts it in, so it has no one home to move it out of, and "move" would be
    /// the wrong word for what it would do.
    MoveItem,
    /// Put a copy of this message in another folder, keeping this one.
    CopyToFolder,
    /// Choose which of this account's folders are kept up to date.
    ChooseFolders,
    /// Open the conditions of the chosen saved search.
    ///
    /// D-2-01's second door: one stored search, and a fuller editor beside the
    /// search box. The only one of a saved search's four commands with no
    /// other route from the tree, which is why it goes first on the menu.
    EditSearchConditions,
    /// Give the chosen saved search a different name.
    ///
    /// Apart from [`Action::RenameContainer`], which renames a contact group.
    /// One id per thing renamed, because the handler behind each reads a
    /// different part of the window.
    RenameSavedSearch,
    /// Take the chosen saved search away.
    ///
    /// Apart from [`Action::DeleteItem`] and [`Action::DeleteContainer`] for
    /// the same reason, and worth keeping apart for a second one: this is the
    /// only delete on this list that cannot reach any mail. The question goes
    /// and the messages it listed stay where they really live.
    DeleteSavedSearch,
}

/// One line on a context menu.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    /// What it says, carrying its own mnemonic.
    pub label: &'static str,
    pub action: Action,
}

const fn entry(label: &'static str, action: Action) -> Entry {
    Entry { label, action }
}

/// What to offer for whatever has focus.
pub fn entries_for(focus: Focus) -> &'static [Entry] {
    match focus {
        Focus::Messages => MESSAGES,
        Focus::MailFolders => MAIL_FOLDERS,
        Focus::SavedSearch => SAVED_SEARCHES,
        Focus::Items(ItemKind::Contact) => CONTACTS,
        Focus::Items(ItemKind::Event) => EVENTS,
        Focus::Items(ItemKind::Reminder) => REMINDERS,
        Focus::Items(ItemKind::Task) => TASKS,
        Focus::Items(ItemKind::Note) => NOTES,
        // Mail is a message list, which is the first arm. There is no second
        // kind of mail list.
        Focus::Items(ItemKind::Mail) => MESSAGES,
        Focus::Containers(ContainerKind::Calendar) => CALENDARS,
        Focus::Containers(ContainerKind::TaskList) => TASK_LISTS,
        Focus::Containers(ContainerKind::NoteFolder) => NOTE_FOLDERS,
        Focus::Containers(ContainerKind::ContactGroup) => CONTACT_GROUPS,
    }
}

static MESSAGES: &[Entry] = &[
    entry("&Reply", Action::Reply),
    entry("Reply &all", Action::ReplyAll),
    entry("&Forward", Action::Forward),
    entry("&Mark as read", Action::MarkRead),
    entry("&Star or unstar", Action::ToggleStar),
    entry("&Delete", Action::DeleteMessage),
    entry("Delete &permanently", Action::DeleteMessageOutright),
    entry("Mo&ve to folder", Action::MoveToFolder),
    entry("Cop&y to folder", Action::CopyToFolder),
    // The thing you have to do arrived as an email, and retyping its subject
    // into a task list is the clerical work software exists to remove.
    entry("Copy to a &task", Action::CopyToTask),
    entry("Copy to the &calendar", Action::CopyToEvent),
    entry("Copy to a &note", Action::CopyToNote),
];

static MAIL_FOLDERS: &[Entry] = &[
    entry("&Refresh this folder", Action::RefreshFolder),
    entry("Get &older messages", Action::GetOlder),
    entry("&Folders to keep up to date", Action::ChooseFolders),
    entry("Edit &conditions...", Action::EditSearchConditions),
];

/// A saved search's own row.
///
/// Editing its conditions first, because it is the only one of these with no
/// other way to reach it from the tree: Enter runs the search, Delete removes
/// it, and Rename sits on the menu bar. A context menu is how somebody finds
/// out what can be done with the thing they are on, so the entry that has no
/// other route is the one that most needs to be found.
///
/// Refreshing means running the search again. Results here are worked out when
/// somebody asks rather than kept up to date behind them, so this is the way
/// to ask whether the answer has changed since mail arrived.
///
/// Getting older messages and choosing folders to keep up to date are the two
/// entries a folder has and this does not. Neither means anything on a saved
/// search, and this list exists so that neither is offered.
static SAVED_SEARCHES: &[Entry] = &[
    entry("Edit &conditions...", Action::EditSearchConditions),
    entry("&Run this search again", Action::RefreshFolder),
    entry("Get &older messages", Action::GetOlder),
    entry("Re&name...", Action::RenameSavedSearch),
    entry("&Delete this search", Action::DeleteSavedSearch),
];

static CONTACTS: &[Entry] = &[
    entry("&New contact", Action::NewItem),
    // Worded as putting somebody in and taking them out, rather than as adding
    // and removing, so the one next to Delete does not read like one.
    entry("Put in a &group", Action::AddToGroup),
    entry("Take &out of a group", Action::RemoveFromGroup),
    entry("&Delete", Action::DeleteItem),
];

static EVENTS: &[Entry] = &[
    entry("&New event", Action::NewItem),
    entry("Mo&ve to another calendar", Action::MoveItem),
    entry("&Delete", Action::DeleteItem),
];

static REMINDERS: &[Entry] = &[
    entry("&New reminder", Action::NewItem),
    entry("Mar&k done or not done", Action::ToggleComplete),
    entry("&Delete", Action::DeleteItem),
];

static TASKS: &[Entry] = &[
    entry("&New task", Action::NewItem),
    entry("Mo&ve to another list", Action::MoveItem),
    entry("Mar&k done or not done", Action::ToggleComplete),
    entry("&Delete", Action::DeleteItem),
];

static NOTES: &[Entry] = &[
    entry("&New note", Action::NewItem),
    entry("Mo&ve to another folder", Action::MoveItem),
    entry("&Pin or unpin", Action::TogglePin),
    entry("&Delete", Action::DeleteItem),
];

static CALENDARS: &[Entry] = &[
    entry("&New calendar", Action::NewContainer),
    entry("&Delete this calendar", Action::DeleteContainer),
    entry("&Sync calendar now", Action::SyncNow),
];

static TASK_LISTS: &[Entry] = &[
    entry("&New task list", Action::NewContainer),
    entry("&Delete this list", Action::DeleteContainer),
    entry("&Sync tasks now", Action::SyncNow),
];

// No sync: notes are kept on this computer and go nowhere, so offering to
// sync them would be offering something that cannot happen.
static NOTE_FOLDERS: &[Entry] = &[
    entry("&New folder", Action::NewContainer),
    entry("&Delete this folder", Action::DeleteContainer),
];

static CONTACT_GROUPS: &[Entry] = &[
    // First, because it is what a group is for.
    entry("&Write to this group", Action::WriteToGroup),
    entry("&New group", Action::NewContainer),
    entry("&Rename this group", Action::RenameContainer),
    entry("&Delete this group", Action::DeleteContainer),
    entry("&Sync contacts now", Action::SyncNow),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_is_offered_exactly_where_it_means_something() {
        // The menu and the command have to agree. Offering it on a reminder,
        // which is filed nowhere, would be a stop that teaches nothing; not
        // offering it on a task would leave the only way to correct a
        // misfiled one as deleting it and typing it again.
        use crate::application::pim_command::PimCommand;

        for kind in ItemKind::ALL {
            let offered = entries_for(Focus::Items(kind))
                .iter()
                .any(|e| e.action == Action::MoveItem);
            assert_eq!(
                offered,
                PimCommand::Move.applies_to(kind),
                "the menu and the command disagree about {kind:?}"
            );
        }
    }

    #[test]
    fn test_rename_is_offered_exactly_where_it_works() {
        // A name typed in a hurry is the ordinary case, and until now the only
        // way to correct one was to delete the group and make it again, which
        // also emptied it. The other three containers have no rename written,
        // so offering one there would be a line that does nothing.
        for kind in ContainerKind::ALL {
            let offered = entries_for(Focus::Containers(kind))
                .iter()
                .any(|e| e.action == Action::RenameContainer);
            assert_eq!(
                offered,
                crate::application::new_item::renaming_works(kind),
                "the menu and what is written disagree about {kind:?}"
            );
        }
    }

    #[test]
    fn test_a_contact_can_be_put_in_a_group_from_its_own_menu() {
        // The two storage calls behind these have existed since groups did and
        // neither had a caller, so a group could be made and could never gain
        // or lose a member.
        let offered: Vec<Action> = entries_for(Focus::Items(ItemKind::Contact))
            .iter()
            .map(|e| e.action)
            .collect();

        assert!(offered.contains(&Action::AddToGroup), "{offered:?}");
        assert!(offered.contains(&Action::RemoveFromGroup), "{offered:?}");
    }

    #[test]
    fn test_a_group_can_be_written_to_from_its_own_menu() {
        // Without this a group is decoration: it can be made, named and filled
        // and never used for the one thing anybody keeps a group for.
        let offered: Vec<Action> = entries_for(Focus::Containers(ContainerKind::ContactGroup))
            .iter()
            .map(|e| e.action)
            .collect();

        assert!(offered.contains(&Action::WriteToGroup), "{offered:?}");
    }

    /// Every action offered where `focus` is.
    fn offered_for(focus: Focus) -> Vec<Action> {
        entries_for(focus).iter().map(|e| e.action).collect()
    }

    #[test]
    fn test_a_saved_search_row_offers_only_what_works_on_a_saved_search() {
        // A saved search is not a folder. It has no older messages to fetch,
        // because it has no server behind it, and nothing of its own to keep
        // up to date. Both of those were on this row's menu while the tree
        // reported one focus for every row in it, which is the fault this
        // list exists to fix: a menu entry that does nothing is a stop
        // somebody lands on, hears, and learns nothing from.
        let offered = offered_for(Focus::SavedSearch);

        assert!(
            offered.contains(&Action::EditSearchConditions),
            "{offered:?}"
        );
        assert!(offered.contains(&Action::RefreshFolder), "{offered:?}");
        assert!(offered.contains(&Action::RenameSavedSearch), "{offered:?}");
        assert!(offered.contains(&Action::DeleteSavedSearch), "{offered:?}");
        assert!(!offered.contains(&Action::GetOlder), "{offered:?}");
        assert!(!offered.contains(&Action::ChooseFolders), "{offered:?}");
    }

    #[test]
    fn test_a_searchs_conditions_are_offered_on_its_own_row_and_nowhere_else() {
        // Every other place a menu can be raised, including the folder tree's
        // ordinary rows. An entry offering to edit the conditions of a folder
        // is an entry that cannot work, and one on a message list is worse
        // still, because there is no saved search anywhere near it.
        for focus in Focus::ALL {
            let offered = offered_for(focus).contains(&Action::EditSearchConditions);
            assert_eq!(
                offered,
                focus == Focus::SavedSearch,
                "{focus:?} offers to edit a saved search's conditions: {}",
                offered
            );
        }
    }

    #[test]
    fn test_renaming_and_deleting_a_search_are_not_the_container_commands() {
        // One command id per thing renamed or removed. Reusing the container
        // ones here would send a saved search's Rename into the handler that
        // renames a contact group, which reads a different part of the window
        // and would rename whatever it found there.
        let offered = offered_for(Focus::SavedSearch);

        assert!(!offered.contains(&Action::RenameContainer), "{offered:?}");
        assert!(!offered.contains(&Action::DeleteContainer), "{offered:?}");
        assert!(!offered.contains(&Action::DeleteItem), "{offered:?}");
    }

    #[test]
    fn test_everything_that_can_hold_focus_offers_something() {
        for focus in Focus::ALL {
            assert!(
                !entries_for(focus).is_empty(),
                "{focus:?} has no context menu, so the menu key does nothing there"
            );
        }
    }

    #[test]
    fn test_no_menu_offers_the_same_thing_twice() {
        for focus in Focus::ALL {
            let mut actions: Vec<Action> = entries_for(focus).iter().map(|e| e.action).collect();
            let count = actions.len();
            actions.sort_by_key(|a| format!("{a:?}"));
            actions.dedup();
            assert_eq!(actions.len(), count, "{focus:?} lists something twice");
        }
    }

    #[test]
    fn test_no_two_entries_in_one_menu_share_a_mnemonic() {
        // Two entries on Alt+D means one of them cannot be reached by keyboard
        // once the menu is open, which is the only way it is opened by the
        // people this is for.
        for focus in Focus::ALL {
            let mut letters: Vec<char> = Vec::new();
            for item in entries_for(focus) {
                let mnemonic = item
                    .label
                    .split('&')
                    .nth(1)
                    .and_then(|rest| rest.chars().next())
                    .unwrap_or_else(|| panic!("{focus:?}: {:?} has no mnemonic", item.label))
                    .to_ascii_lowercase();
                assert!(
                    !letters.contains(&mnemonic),
                    "{focus:?}: two entries both use Alt+{mnemonic}"
                );
                letters.push(mnemonic);
            }
        }
    }

    #[test]
    fn test_done_and_pin_are_offered_only_where_they_mean_something() {
        // Marking a contact done, or pinning a calendar, would be a stop in
        // the menu that teaches nothing.
        use crate::application::pim_command::PimCommand;

        for kind in ItemKind::ALL {
            let offered: Vec<Action> = entries_for(Focus::Items(kind))
                .iter()
                .map(|e| e.action)
                .collect();

            assert_eq!(
                offered.contains(&Action::ToggleComplete),
                PimCommand::ToggleComplete.applies_to(kind),
                "{kind:?} disagrees with pim_command about being marked done"
            );
            assert_eq!(
                offered.contains(&Action::TogglePin),
                PimCommand::TogglePin.applies_to(kind),
                "{kind:?} disagrees with pim_command about pinning"
            );
        }
    }

    #[test]
    fn test_notes_are_not_offered_a_sync_they_cannot_do() {
        // Notes stay on this computer. Offering to sync them would be
        // offering something that cannot happen.
        let offered: Vec<Action> = entries_for(Focus::Containers(ContainerKind::NoteFolder))
            .iter()
            .map(|e| e.action)
            .collect();

        assert!(!offered.contains(&Action::SyncNow));
    }

    #[test]
    fn test_every_container_menu_can_make_and_remove_one() {
        for kind in ContainerKind::ALL {
            let offered: Vec<Action> = entries_for(Focus::Containers(kind))
                .iter()
                .map(|e| e.action)
                .collect();

            assert!(offered.contains(&Action::NewContainer), "{kind:?}");
            assert!(offered.contains(&Action::DeleteContainer), "{kind:?}");
        }
    }

    #[test]
    fn test_a_message_can_be_copied_into_the_other_modules() {
        // The subject becomes the title and the message becomes the body, so
        // what arrived as mail becomes something you can act on without
        // retyping it.
        let offered: Vec<Action> = entries_for(Focus::Messages)
            .iter()
            .map(|e| e.action)
            .collect();

        for wanted in [Action::CopyToTask, Action::CopyToEvent, Action::CopyToNote] {
            assert!(offered.contains(&wanted), "a message cannot be {wanted:?}");
        }
    }

    #[test]
    fn test_a_message_menu_has_the_things_you_do_to_a_message() {
        let offered: Vec<Action> = entries_for(Focus::Messages)
            .iter()
            .map(|e| e.action)
            .collect();

        for wanted in [
            Action::Reply,
            Action::ReplyAll,
            Action::Forward,
            Action::DeleteMessage,
        ] {
            assert!(
                offered.contains(&wanted),
                "a message menu has no {wanted:?}"
            );
        }
    }
}

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
//! Rename, move to another list, mark a whole folder read and empty a folder
//! are the obvious absences. None of them is implemented, so none of them is
//! offered.
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
    /// The list in one of the personal information modules.
    Items(ItemKind),
    /// The sidebar tree of one of them: calendars, task lists, folders.
    Containers(ContainerKind),
}

impl Focus {
    /// Every place a menu can be raised, so tests cover the whole set.
    pub const ALL: [Focus; 12] = [
        Focus::Messages,
        Focus::MailFolders,
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
    /// Put a copy of this message in another folder, keeping this one.
    CopyToFolder,
    /// Choose which of this account's folders are kept up to date.
    ChooseFolders,
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
];

static CONTACTS: &[Entry] = &[
    entry("&New contact", Action::NewItem),
    entry("&Delete", Action::DeleteItem),
];

static EVENTS: &[Entry] = &[
    entry("&New event", Action::NewItem),
    entry("&Delete", Action::DeleteItem),
];

static REMINDERS: &[Entry] = &[
    entry("&New reminder", Action::NewItem),
    entry("Mar&k done or not done", Action::ToggleComplete),
    entry("&Delete", Action::DeleteItem),
];

static TASKS: &[Entry] = &[
    entry("&New task", Action::NewItem),
    entry("Mar&k done or not done", Action::ToggleComplete),
    entry("&Delete", Action::DeleteItem),
];

static NOTES: &[Entry] = &[
    entry("&New note", Action::NewItem),
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
    entry("&New group", Action::NewContainer),
    entry("&Delete this group", Action::DeleteContainer),
    entry("&Sync contacts now", Action::SyncNow),
];

#[cfg(test)]
mod tests {
    use super::*;

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

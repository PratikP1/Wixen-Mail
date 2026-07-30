//! Putting the context menu on screen, wherever focus is.
//!
//! The words and the list are in [`crate::application::context_menu`]. This is
//! the part that turns them into a menu, gives each line the command id the
//! application already handles, and pops it up.
//!
//! # It has to be reachable without a mouse
//!
//! `EVT_CONTEXT_MENU` is the event to listen for, not a right click. Windows
//! raises it for the Applications key and for `Shift+F10` as well as for the
//! mouse, so listening for the right one is what makes this reachable by the
//! people it is for. A handler on the mouse event alone would have looked
//! finished and been useless.
//!
//! # Every line has to do something
//!
//! Each action maps to a command id the frame already handles, so the menu
//! cannot offer anything the application does not do.
//! `tests/wired.rs` fails the build if an id is handled and nothing raises it;
//! [`command_for`] is the other direction, and the test below walks every
//! action to make sure none of them is left without one.

use crate::application::context_menu::{Action, Focus, entries_for};
use wxdragon::prelude::*;

/// Which command id an action raises.
///
/// These are the ids the menu bar already uses, so a thing done from a context
/// menu and the same thing done from the menu bar run the same code. Adding a
/// second id for "reply, but from the context menu" would be two things to
/// keep working.
pub const fn command_for(action: Action) -> Id {
    use crate::presentation::wx_app::{
        ID_CHOOSE_FOLDERS, ID_CONTEXT_COPY_TO_EVENT, ID_CONTEXT_COPY_TO_NOTE,
        ID_CONTEXT_COPY_TO_TASK, ID_CONTEXT_DELETE_CONTAINER, ID_CONTEXT_DELETE_ITEM,
        ID_CONTEXT_NEW_CONTAINER, ID_CONTEXT_NEW_ITEM, ID_CONTEXT_SYNC_NOW,
        ID_CONTEXT_TOGGLE_COMPLETE, ID_CONTEXT_TOGGLE_PIN, ID_COPY_TO_FOLDER, ID_DELETE,
        ID_FORWARD, ID_GET_OLDER, ID_MARK_READ, ID_MOVE_TO_FOLDER, ID_REFRESH_FOLDER, ID_REPLY,
        ID_REPLY_ALL, ID_TOGGLE_STAR,
    };
    match action {
        // These are the menu bar's own ids, so the same thing done two ways
        // runs one piece of code.
        Action::Reply => ID_REPLY,
        Action::ReplyAll => ID_REPLY_ALL,
        Action::Forward => ID_FORWARD,
        Action::MarkRead => ID_MARK_READ,
        Action::ToggleStar => ID_TOGGLE_STAR,
        Action::DeleteMessage => ID_DELETE,
        Action::RefreshFolder => ID_REFRESH_FOLDER,
        Action::GetOlder => ID_GET_OLDER,
        Action::MoveToFolder => ID_MOVE_TO_FOLDER,
        Action::CopyToFolder => ID_COPY_TO_FOLDER,
        Action::ChooseFolders => ID_CHOOSE_FOLDERS,
        // These have no menu bar entry, because what they act on is whatever
        // panel is open. Each handler reads the open module and acts on that.
        Action::NewItem => ID_CONTEXT_NEW_ITEM,
        Action::DeleteItem => ID_CONTEXT_DELETE_ITEM,
        Action::ToggleComplete => ID_CONTEXT_TOGGLE_COMPLETE,
        Action::TogglePin => ID_CONTEXT_TOGGLE_PIN,
        Action::NewContainer => ID_CONTEXT_NEW_CONTAINER,
        Action::DeleteContainer => ID_CONTEXT_DELETE_CONTAINER,
        Action::SyncNow => ID_CONTEXT_SYNC_NOW,
        Action::CopyToTask => ID_CONTEXT_COPY_TO_TASK,
        Action::CopyToEvent => ID_CONTEXT_COPY_TO_EVENT,
        Action::CopyToNote => ID_CONTEXT_COPY_TO_NOTE,
    }
}

/// Show the menu for whatever has focus, at wherever the event asked for.
///
/// The chosen line raises an ordinary menu event, which bubbles to the frame
/// and lands in the same handler the menu bar uses.
pub fn show(on: &dyn WxWidget, focus: Focus) {
    let entries = entries_for(focus);
    if entries.is_empty() {
        return;
    }

    let mut menu = Menu::builder().build();
    for item in entries {
        menu.append(command_for(item.action), item.label, "", ItemKind::Normal);
    }

    // No position given, so wxWidgets puts it where the keyboard would expect
    // it: at the focused item rather than at the last place the mouse was.
    on.popup_menu(&mut menu, None);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::context_menu::Focus;

    #[test]
    fn test_every_action_has_a_command_behind_it() {
        // Walking the menus rather than the enum, because what matters is
        // that nothing a person can choose is left without a command. An
        // action nobody offers is dead weight; one offered with no command
        // would be a line that does nothing.
        for focus in Focus::ALL {
            for item in entries_for(focus) {
                let id = command_for(item.action);
                assert!(
                    id > 0,
                    "{focus:?}: {:?} has no command id, so choosing it does nothing",
                    item.label
                );
            }
        }
    }

    #[test]
    fn test_no_two_actions_share_a_command() {
        // Two actions on one id means choosing either does whatever the
        // handler was written for, which is one of them.
        let mut seen: Vec<(Id, Action)> = Vec::new();
        for focus in Focus::ALL {
            for item in entries_for(focus) {
                let id = command_for(item.action);
                if let Some((_, other)) = seen
                    .iter()
                    .find(|(seen_id, a)| *seen_id == id && *a != item.action)
                {
                    panic!("{:?} and {other:?} both raise {id}", item.action);
                }
                seen.push((id, item.action));
            }
        }
    }
}

//! Picking where something goes.
//!
//! One window for every move and copy: a message into another folder, a task
//! onto another list, an event into another calendar. A tree of accounts with
//! their places under them, because two accounts can both have an Archive and a
//! flat list of names makes those two rows that read identically.
//!
//! The shape of the tree, and which places are offered at all, is decided in
//! [`crate::application::destinations`] where it can be tested. This builds the
//! window and reads back what was chosen.
//!
//! # Keyboard
//!
//! A tree control, so arrows move, Left and Right collapse and expand, and
//! typing jumps to a name. Enter chooses, Escape leaves. Nothing here needs a
//! mouse and nothing needs a drag, which is what most mail clients make you do
//! for exactly this.

use crate::application::destinations::{Branch, Moving, nothing_to_offer};
use crate::presentation::accessibility::names::{
    set_accessible_name, set_accessible_name_and_description,
};
use crate::presentation::theme;
use wxdragon::prelude::*;

/// What the window is called, and what its tree is described as.
///
/// The words differ between moving and copying because they are different acts
/// and somebody hearing the title should know which one they are in the middle
/// of. Kept out of the builder so both can be tested.
pub fn heading(moving: Moving, copying: bool) -> String {
    let act = if copying { "Copy" } else { "Move" };
    match moving {
        Moving::Message => format!("{act} the message to"),
        // What is being moved, not what it is going into. "Move the calendar
        // to" is a different sentence from "Move the event to", and only one
        // of them is what is happening.
        Moving::Item(kind) => format!("{act} the {} to", kind.holds().label().to_lowercase()),
    }
}

/// Ask where something should go.
///
/// `None` when somebody cancelled or there was nowhere to offer. The caller
/// checks for nowhere first, with [`nothing_to_offer`], so that case is a
/// sentence rather than an empty window.
///
/// `last_used` is where the previous one went. The window opens on it if it is
/// still on offer, so filing several messages into the same folder is the
/// shortcut and Enter rather than a walk through the tree each time.
pub fn ask(
    parent: &Frame,
    moving: Moving,
    copying: bool,
    branches: &[Branch],
    last_used: Option<&str>,
) -> Option<String> {
    if branches.is_empty() {
        return None;
    }
    let (dialog, tree) = build_destination_dialog(
        parent,
        moving,
        copying,
        branches,
        last_used,
        theme::current_from_stored_config(),
    );

    let answer = dialog.show_modal();
    // An account row carries no destination, so choosing one gives nothing
    // rather than the first folder under it. Somebody who meant a folder and
    // landed on the account gets no move, which they can see, instead of a
    // move somewhere they did not name.
    let chosen = if answer == ID_OK {
        tree.get_selection()
            .and_then(|selected| tree.get_custom_data(&selected))
            .and_then(|data| data.downcast_ref::<String>().cloned())
    } else {
        None
    };
    tree.cleanup_custom_data();
    dialog.destroy();
    chosen
}

/// Build the "where does this go" dialog without showing it.
///
/// Everything `ask` used to do up to its own `.show_modal()` call, split out
/// the same way [`crate::presentation::wx_settings::build_settings_dialog`]
/// splits Settings: a test can build the real dialog and read back the real
/// colour a live control holds, and never call `.show_modal()` at all.
///
/// `branches` must not be empty; `ask` checks that before calling here, since
/// there is nothing to build a tree out of otherwise.
///
/// Returns the tree alongside the dialog, the same way the caller needs it
/// after a real `.show_modal()`: to read which destination was selected.
pub fn build_destination_dialog(
    parent: &Frame,
    moving: Moving,
    copying: bool,
    branches: &[Branch],
    last_used: Option<&str>,
    palette: Option<theme::Palette>,
) -> (Dialog, TreeCtrl) {
    let open_on = crate::application::destinations::open_on(branches, last_used)
        .map(|place| place.id.clone());

    let title = heading(moving, copying);
    let dialog = Dialog::builder(parent, &title)
        .with_size(520, 420)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();

    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let tree = TreeCtrl::builder(&dialog)
        .with_style(TreeCtrlStyle::HasButtons | TreeCtrlStyle::LinesAtRoot)
        .build();
    set_accessible_name_and_description(
        &tree,
        &title,
        "Arrow keys move, Right expands an account, Enter chooses.",
    );

    // Every tree needs a root. The accounts hang off it, and the style hides
    // it, so nobody has to arrow past a row that means nothing.
    let root = tree.add_root("Accounts", None, None);
    // Where focus starts: somewhere that is an answer, rather than on an
    // account name that is not one.
    let mut start_on: Option<TreeItemId> = None;

    if let Some(root) = root.as_ref() {
        for branch in branches {
            let Some(account) = tree.append_item(root, &branch.account_name, None, None) else {
                continue;
            };
            for place in &branch.places {
                // The identifier travels with the row. Matching rows by their
                // position in a list built alongside would be one list and one
                // tree that have to stay in step, and the failure is silent:
                // the wrong folder, with everything reporting success.
                let row =
                    tree.append_item_with_data(&account, &place.name, place.id.clone(), None, None);
                if open_on.as_deref() == Some(place.id.as_str()) {
                    start_on = row;
                }
            }
            tree.expand(&account);
        }
    }
    sizer.add(&tree, 1, SizerFlag::All | SizerFlag::Expand, 8);

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    let choose = Button::builder(&dialog)
        .with_label(if copying { "&Copy" } else { "&Move" })
        .with_id(ID_OK)
        .build();
    set_accessible_name(&choose, if copying { "Copy" } else { "Move" });
    let cancel = Button::builder(&dialog)
        .with_label("&Cancel")
        .with_id(ID_CANCEL)
        .build();
    set_accessible_name(&cancel, "Cancel");
    buttons.add(&choose, 0, SizerFlag::All, 6);
    buttons.add(&cancel, 0, SizerFlag::All, 6);
    sizer.add_sizer(&buttons, 0, SizerFlag::AlignRight | SizerFlag::All, 8);

    dialog.set_sizer(sizer, true);

    // Focus lands on a real destination rather than on the tree's notion of
    // nothing selected, so the first thing announced is somewhere the message
    // could go, and on the last folder used so that filing the next one is a
    // single Enter.
    if let Some(start_on) = start_on.as_ref() {
        tree.select_item(start_on);
        tree.ensure_visible(start_on);
    }
    tree.set_focus();

    // Painted last. The tree is left to Windows here, the same as every
    // `Choice`, `ComboBox`, `RadioButton` and `CheckBox` elsewhere in this
    // round. `None` means high contrast is on, or the system is set up in a
    // way this application should not paint over, so nothing is set here and
    // Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dialog, palette.main_surface());
    }

    (dialog, tree)
}

/// What to say when there is nowhere to put it.
///
/// Re-exported here so a caller that only knows about the window does not have
/// to reach past it for the sentence that replaces the window.
pub fn nowhere(moving: Moving) -> &'static str {
    nothing_to_offer(moving)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::new_item::ContainerKind;

    #[test]
    fn test_the_title_says_which_act_this_is() {
        // Moving and copying are different, and somebody hearing the title
        // should know which one they are in the middle of.
        assert!(heading(Moving::Message, false).starts_with("Move"));
        assert!(heading(Moving::Message, true).starts_with("Copy"));
    }

    #[test]
    fn test_the_title_says_what_is_being_moved() {
        assert!(heading(Moving::Message, false).contains("message"));

        for kind in ContainerKind::ALL {
            let said = heading(Moving::Item(kind), false);
            assert!(said.starts_with("Move the"), "{kind:?}: {said}");
            assert!(said.ends_with(" to"), "{kind:?}: {said}");
        }
    }

    #[test]
    fn test_every_kind_names_itself_rather_than_saying_item() {
        // "Move the item to" tells somebody nothing they did not already know
        // and reads the same for all four.
        let said: Vec<String> = ContainerKind::ALL
            .iter()
            .map(|kind| heading(Moving::Item(*kind), false))
            .collect();
        let unique: std::collections::HashSet<&String> = said.iter().collect();

        assert_eq!(unique.len(), said.len(), "{said:?}");
    }
}

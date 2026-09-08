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

use crate::application::destinations::{
    Branch, Destination, Filing, FolderInAnAccount, Moving, nothing_to_offer,
};
use crate::presentation::accessibility::names::{
    set_accessible_name, set_accessible_name_and_description,
};
use crate::presentation::theme;
use crate::presentation::tree_walk;
use wxdragon::prelude::*;

/// What the window is called, and what its tree is described as.
///
/// The words differ between moving and copying because they are different acts
/// and somebody hearing the title should know which one they are in the middle
/// of. Kept out of the builder so both can be tested.
pub fn heading(moving: Moving, filing: Filing) -> String {
    let act = filing.act();
    match moving {
        Moving::Message => format!("{act} the message to"),
        // What is being moved, not what it is going into. "Move the calendar
        // to" is a different sentence from "Move the event to", and only one
        // of them is what is happening.
        Moving::Item(kind) => format!("{act} the {} to", kind.holds().label().to_lowercase()),
        // "Move the folder to", and never "Copy": a folder is not copied, and
        // the caller never asks for one. Said the same way regardless, because
        // a heading that changed with a flag nobody sets is a branch nothing
        // reads that still looks like a decision.
        Moving::Folder => format!("{act} the folder to"),
    }
}

/// What choosing the row at `position` in the tree's walk means.
///
/// The whole destination and not its identifier. A folder's path is unique
/// inside one account and not across them, so a window drawing two accounts
/// that answered with a path could draw the right rows and still not say which
/// one was chosen. The account travels with the choice because it is a field of
/// the thing chosen, rather than being inferred by the caller from which branch
/// it believes it passed in.
///
/// `None` three ways, and all three mean the same thing: nothing was chosen.
/// The root has no position because it is not a row. An account heading has a
/// position and no destination, because it is somewhere to look rather than
/// somewhere to put something, and somebody who meant a folder and landed on
/// the account gets no move, which they can see, instead of a move somewhere
/// they did not name. A position past the end is a row this cannot place, and
/// the safe answer to that is nothing rather than a guess.
pub fn what_a_selection_means(
    destinations: &[Option<Destination>],
    position: Option<usize>,
) -> Option<Destination> {
    position
        .and_then(|position| destinations.get(position))
        .cloned()
        .flatten()
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
///
/// `the_account_it_is_in` is where to open when nothing was remembered. With
/// several accounts on offer, the first branch is whichever account the sidebar
/// happens to draw first, which is nobody's answer to "where should this go".
pub fn ask(
    parent: &Frame,
    moving: Moving,
    filing: Filing,
    branches: &[Branch],
    last_used: Option<FolderInAnAccount<'_>>,
    the_account_it_is_in: Option<&str>,
) -> Option<Destination> {
    if branches.is_empty() {
        return None;
    }
    let (dialog, tree, destinations) = build_destination_dialog(
        parent,
        moving,
        filing,
        branches,
        last_used,
        the_account_it_is_in,
        theme::current_from_stored_config(),
    );

    let answer = dialog.show_modal();
    let chosen = if answer == ID_OK {
        what_a_selection_means(&destinations, tree_walk::where_the_selection_sits(&tree))
    } else {
        None
    };
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
///
/// And the destinations beside it, one for every row the tree holds, in the
/// order a depth-first walk meets them. That vector is how a row is turned
/// back into a folder, so `ask` needs it as much as it needs the control.
pub fn build_destination_dialog(
    parent: &Frame,
    moving: Moving,
    filing: Filing,
    branches: &[Branch],
    last_used: Option<FolderInAnAccount<'_>>,
    the_account_it_is_in: Option<&str>,
    palette: Option<theme::Palette>,
) -> (Dialog, TreeCtrl, Vec<Option<Destination>>) {
    // Held whole rather than as its path, for the reason the answer is whole:
    // two accounts can both hold an `Archive`, and a row matched on the path
    // alone would put the cursor on whichever branch was drawn first.
    let open_on =
        crate::application::destinations::open_on(branches, last_used, the_account_it_is_in);
    // Which one branch is drawn open. Decided in `destinations`, with no
    // control in the room, and read here.
    let open_branch = crate::application::destinations::the_branch_to_open(branches, open_on);

    let title = heading(moving, filing);
    let dialog = Dialog::builder(parent, &title)
        .with_size(520, 420)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();

    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let tree = TreeCtrl::builder(&dialog)
        .with_style(TreeCtrlStyle::HasButtons | TreeCtrlStyle::LinesAtRoot)
        .build();
    // Every key named here is one a `TreeCtrl` answers to on its own: the
    // arrows, Left and Right on a row with children, and the activation the
    // dialog's default button takes. Left is named now that it earns its place:
    // accounts other than the one this opens on start closed, so somebody who
    // opened the wrong one needs the way back out as much as the way in.
    //
    // Escape is left out, and that is a judgement rather than an oversight. It
    // closes every dialog in this program, so it is the one key somebody
    // already knows, and this sentence is read out in full every time focus
    // reaches the tree.
    set_accessible_name_and_description(
        &tree,
        &title,
        "Arrow keys move, Right opens an account and Left closes it, Enter chooses.",
    );

    // Every tree needs a root. The accounts hang off it, and the style hides
    // it, so nobody has to arrow past a row that means nothing.
    let root = tree.add_root("Accounts", None, None);
    // Where focus starts: somewhere that is an answer, rather than on an
    // account name that is not one.
    //
    // Held as a position in the walk rather than as the row itself, because a
    // `TreeItemId` owns the pointer it wraps and cannot be copied, and a row
    // that is a place is also a row other places hang under. The position is
    // the same thing the answer is read back by, so focus starts on the row the
    // vector says holds the remembered destination.
    let mut start_on: Option<usize> = None;
    // What each row means, in the order the rows are appended, which for this
    // shape is also the order the built tree is walked: an account goes under
    // the root and its places go under it before the next account is reached.
    //
    // This used to be the other way round, with the identifier travelling on
    // the row itself, and the comment here argued for it: a list beside the
    // tree would be two things that have to stay in step, and the failure is
    // silent, which in this dialog means somebody's mail in a folder they did
    // not name. Three things answer that argument now. The entry is pushed in
    // the same step that appends the row and only when the append succeeded,
    // so the two cannot come apart. This tree is built once and never touched
    // again while the dialog is open: nothing is inserted, removed or
    // re-sorted. And the position comes from asking the control which row is
    // selected rather than from matching labels, so two accounts with the same
    // name stay two rows.
    //
    // What the old way cost is the reason it had to go. `append_item_with_data`
    // does not put the identifier on the row; it puts it in a process-global
    // map in `wxdragon` that nothing ever takes anything out of, so every
    // opening of this dialog left one entry per folder behind for the life of
    // the program. `tests/tree_rows_leave_no_registry_entry.rs` counts them.
    let mut destinations: Vec<Option<Destination>> = Vec::new();

    if let Some(root) = root.as_ref() {
        for branch in branches {
            let Some(account) = tree.append_item(root, &branch.account_name, None, None) else {
                // Its places go with it, as they always have: they have nothing
                // to hang under. Nothing is pushed either, so the vector still
                // holds one entry per row the tree really has.
                continue;
            };
            destinations.push(None);
            // What each depth's places hang under: the account for the places
            // at nought, and for anything deeper the last row drawn one above
            // it. `Destination::depth` says the places arrive in the order a
            // walk down the tree meets them, so the row a place belongs under
            // is always one this has already drawn.
            //
            // A place deeper than one below the place before it hangs under the
            // deepest row there is, rather than being dropped or drawn as a
            // sibling of the account. That is a folder whose own parent was not
            // offered, which the folder move really produces: it leaves out the
            // folder being moved and everything inside it, so a folder further
            // down can be offered when the one it sits in is not.
            let mut under: Vec<TreeItemId> = vec![account];
            for place in &branch.places {
                let depth = place.depth.min(under.len() - 1);
                let Some(row) = tree.append_item(&under[depth], &place.name, None, None) else {
                    continue;
                };
                destinations.push(Some(place.clone()));
                if open_on == Some(place) {
                    start_on = Some(destinations.len() - 1);
                }
                if depth > 0 {
                    // Opened, or the rows inside it are drawn and unreachable:
                    // a closed branch is rows a screen reader never meets and
                    // arrow keys never enter.
                    tree.expand(&under[depth]);
                }
                under.truncate(depth + 1);
                under.push(row);
            }
            // One account's folders showing, not every account's. A screen
            // reader user meets rows in order and cannot skim, so with eight
            // accounts open the last account's folders sit behind every folder
            // of the seven above. Closed, they are eight rows, each announced
            // as collapsed and with a count of what is inside, and Right opens
            // the one somebody wants.
            if open_branch == Some(branch.account_id.as_str()) {
                tree.expand(&under[0]);
            }
        }
    }
    sizer.add(&tree, 1, SizerFlag::All | SizerFlag::Expand, 8);

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    let choose = Button::builder(&dialog)
        // The mnemonic on the same letter the word starts with, which is what
        // it was when this said "&Copy" or "&Move" in two places. One source
        // for the label and the name, so the button cannot say one word and be
        // announced as another.
        .with_label(&format!("&{}", filing.act()))
        .with_id(ID_OK)
        .build();
    set_accessible_name(&choose, filing.act());
    let cancel = Button::builder(&dialog)
        // No mnemonic, which is what the other twenty-two Cancel buttons in
        // this program do and what Windows does. It had one, and it was the
        // same letter as Copy: pressing Alt+C on this dialog moved between the
        // two rather than choosing either, so the key that was meant to file a
        // message might have been the key that abandoned the job. Escape
        // closes this, as it closes every dialog here.
        .with_label("Cancel")
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
    if let Some(at) = start_on
        && let Some(row) = tree_walk::rows_in_walk_order(&tree).get(at)
    {
        tree.select_item(row);
        tree.ensure_visible(row);
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

    (dialog, tree, destinations)
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
        assert!(heading(Moving::Message, Filing::Moving).starts_with("Move"));
        assert!(heading(Moving::Message, Filing::Copying).starts_with("Copy"));
    }

    #[test]
    fn test_the_title_says_what_is_being_moved() {
        assert!(heading(Moving::Message, Filing::Moving).contains("message"));

        for kind in ContainerKind::ALL {
            let said = heading(Moving::Item(kind), Filing::Moving);
            assert!(said.starts_with("Move the"), "{kind:?}: {said}");
            assert!(said.ends_with(" to"), "{kind:?}: {said}");
        }
    }

    /// Two accounts with folders under each, as the rows come out of the
    /// build: the account heading first, then its places, then the next
    /// account.
    ///
    /// Both accounts have folders on purpose. A fixture whose second account
    /// were empty would let a resolution that reached forward for the first
    /// folder it could find answer `None` for that account by luck, and the
    /// account rows are the whole point of this vector.
    ///
    /// Both accounts have an `Archive`, spelled the way a server spells one
    /// rather than prefixed with the account. That is what makes the answer's
    /// account load-bearing: with the paths made unique by hand, a resolution
    /// that dropped the account would look right in every assertion below.
    fn two_accounts_with_folders() -> Vec<Option<Destination>> {
        let place = |name: &str, account: &str| {
            Some(Destination {
                name: name.to_string(),
                id: name.to_string(),
                account_id: account.to_string(),
                depth: 0,
            })
        };
        vec![
            None,
            place("Archive", "acct-1"),
            place("Receipts", "acct-1"),
            None,
            place("Archive", "acct-2"),
        ]
    }

    #[test]
    fn test_choosing_a_folder_gives_that_folder_in_that_account() {
        let rows = two_accounts_with_folders();

        for (position, path, account) in [
            (1, "Archive", "acct-1"),
            (2, "Receipts", "acct-1"),
            (4, "Archive", "acct-2"),
        ] {
            let chosen = what_a_selection_means(&rows, Some(position)).expect("a destination");
            assert_eq!(chosen.id, path, "row {position}");
            assert_eq!(
                chosen.account_id, account,
                "row {position} is the account's own Archive and not the other's"
            );
        }
    }

    #[test]
    fn test_choosing_an_account_gives_nothing_rather_than_the_first_folder_under_it() {
        // The row somebody landed on is not a place mail can go. Answering
        // with the folder under it would file the message somewhere they never
        // named, and everything would report success.
        let rows = two_accounts_with_folders();

        assert_eq!(what_a_selection_means(&rows, Some(0)), None);
        assert_eq!(what_a_selection_means(&rows, Some(3)), None);
    }

    #[test]
    fn test_a_row_this_cannot_place_gives_nothing() {
        // The root has no position because it is not a row, and a position
        // past the end is a row the vector does not know. Both answer nothing,
        // which is the safe answer rather than a guess.
        let rows = two_accounts_with_folders();

        assert_eq!(what_a_selection_means(&rows, None), None);
        assert_eq!(what_a_selection_means(&rows, Some(rows.len())), None);
        assert_eq!(what_a_selection_means(&[], Some(0)), None);
    }

    #[test]
    fn test_every_kind_names_itself_rather_than_saying_item() {
        // "Move the item to" tells somebody nothing they did not already know
        // and reads the same for all four.
        let said: Vec<String> = ContainerKind::ALL
            .iter()
            .map(|kind| heading(Moving::Item(*kind), Filing::Moving))
            .collect();
        let unique: std::collections::HashSet<&String> = said.iter().collect();

        assert_eq!(unique.len(), said.len(), "{said:?}");
    }
}

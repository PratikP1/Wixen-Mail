//! Choosing which folders an account keeps up to date.
//!
//! A mail server lists every mailbox the account can see. On a shared server or
//! a university system that is routinely dozens or hundreds, and somebody uses
//! four of them. On Gmail it includes All Mail, which holds a copy of every
//! message in the account, so keeping it up to date means downloading the whole
//! account a second time and reading every message twice in the list.
//!
//! So this asks. A tree, one item per folder, nested the way the folder tree
//! in the main window nests them, with a check box beside each and the item
//! saying how many messages the folder holds and whether it is one somebody
//! is subscribed to. What is ticked when the window opens is what the sync
//! would do anyway, so somebody who does not care can close it and lose
//! nothing.
//!
//! # Why a tree, and why it was a list until 2026-09-18
//!
//! This module's header used to say the folder tree in the main window is
//! one flat level, so a tree here would be a tree with no branches. That was
//! true when it was written and stopped being true when plan 01-04 stored
//! each folder's parent and the main window began nesting by it; the header
//! was not read again, and the tester met a flat list of fifty Gmail labels
//! where the window beside it showed a hierarchy (#70, first point). The
//! nesting here reads the same stored parent, through [`FolderRow::parent`],
//! so the two cannot disagree again.
//!
//! # Making the tick reach a screen reader
//!
//! On Windows wxWidgets draws a `CheckListBox`'s check boxes itself, so the
//! platform sees a plain list. The list this used to be answered for its
//! rows through an accessible object this program wrote, and a reading of
//! that object over the channel NVDA uses found it saying read-only, busy and
//! alerting where it meant checked: the toolkit's state constants and the
//! toolkit's own enumeration disagree, which the paragraph in
//! `accessibility/names.rs` where the object stood sets out. Its header said
//! "Still to be confirmed with a screen reader"; the tester confirmed it on
//! 2026-09-17 and heard "check box, read-only, not checked".
//!
//! A native tree with `TVS_CHECKBOXES` keeps the check state as the item's
//! own state image, which NVDA reads from the control with
//! `TVM_GETITEMSTATE` and which Windows' own accessible object for the tree
//! answers as a check button with the checked state. No object of this
//! program's is in that path. The style, the ticks and the reading go through
//! [`native_tree_checks`], and `tests/a_kept_folder_reads_as_a_checked_check_box.rs`
//! measured all of it on 2026-09-18 before this was written, Space included:
//! the control toggles the item under the cursor by itself, so nothing here
//! handles the key.
//!
//! What is written back at Save is read from the control's own state, item
//! by item, at that moment; nothing here remembers a tick, so nothing here
//! can remember it wrong.
//!
//! The item's own text carries everything except the tick: the folder's name,
//! how much is in it, and the warning about the one folder that doubles what
//! gets downloaded. Those are read whatever happens to the check box.
//!
//! **Windows only**, as the tree's check boxes are. Elsewhere [`ask`] answers
//! `None` without opening anything, because a dialog whose ticks reach nobody
//! is worse than no dialog; a port needs its own bridge, as `CLAUDE.md` says
//! of `wxAccessible`.

use crate::presentation::accessibility::names::set_accessible_name;
use crate::presentation::native_tree_checks;
use crate::presentation::theme;
use wxdragon::prelude::*;

/// One folder, and whether it is currently kept up to date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderRow {
    /// The server's own spelling, which is the identifier.
    pub path: String,
    /// What to show and announce.
    pub name: String,
    /// The path of the folder this one sits inside, or `None` at the top.
    ///
    /// Read from the stored `parent_id` the main window's folder tree nests
    /// by, never from the path, so the two cannot nest differently. The
    /// separator is never split here; that was plan 01-04's rule.
    pub parent: Option<String>,
    /// Whether it syncs now.
    pub syncing: bool,
    /// Whether the account is subscribed to it on the server.
    pub subscribed: bool,
    /// Whether it holds a copy of every message in the account.
    pub holds_all_mail: bool,
    /// How many messages the server says are in it.
    pub total: usize,
}

/// What the row says when it is read aloud.
///
/// The name first, because that is what somebody is looking for and a screen
/// reader reads from the start. Then the facts that decide the answer: how big
/// it is, and the warning about the one folder where ticking it means
/// downloading everything twice.
///
/// Separate from the dialog so it can be tested cheaply. What a row says is
/// the part that has to be right, and a row's words need no dialog. The dialog
/// itself is opened in a test: wxWidgets allows one live window per process,
/// and `tests/theme_reach.rs` spends its binary's one on this dialog among
/// others.
pub fn row_label(folder: &FolderRow) -> String {
    let mut label = folder.name.clone();

    if folder.holds_all_mail {
        label.push_str(", holds a copy of every message, so this doubles what is downloaded");
    } else if folder.total == 1 {
        label.push_str(", 1 message");
    } else {
        label.push_str(&format!(", {} messages", folder.total));
    }

    if !folder.subscribed {
        label.push_str(", not subscribed");
    }
    label
}

/// Where each row sits in the tree: the row's index, its depth, and the index
/// of the row it sits under, in the order the tree walks them.
///
/// Each top-level row in the stored order, then everything under it before
/// the next, so a child always follows its parent and siblings keep the
/// order the server listed them in. That order is also the order the native
/// tree walks its items, which is what lets a handle be matched to a row
/// without wxdragon ever handing one out.
///
/// A row whose parent is not among the rows sits at the top. A folder must
/// never vanish from this dialog because the folder above it was not listed,
/// since a folder nobody can see is a folder nobody can untick. Rows whose
/// parents form a loop, which a stored `parent_id` cannot produce but a
/// dialog must not hang on, sit at the top for the same reason.
pub fn nesting(rows: &[FolderRow]) -> Vec<(usize, usize, Option<usize>)> {
    let parent_of: Vec<Option<usize>> = rows
        .iter()
        .map(|row| {
            row.parent
                .as_deref()
                .and_then(|parent| rows.iter().position(|it| it.path == parent))
        })
        .collect();
    let children_of = |index: usize| {
        let parent_of = &parent_of;
        (0..rows.len())
            .rev()
            .filter(move |&child| parent_of[child] == Some(index))
    };

    let mut placed = Vec::with_capacity(rows.len());
    let mut seen = vec![false; rows.len()];
    let mut to_place: Vec<(usize, usize, Option<usize>)> = (0..rows.len())
        .rev()
        .filter(|&index| parent_of[index].is_none())
        .map(|index| (index, 0, None))
        .collect();
    while let Some((index, depth, parent)) = to_place.pop() {
        if seen[index] {
            continue;
        }
        seen[index] = true;
        placed.push((index, depth, parent));
        to_place.extend(children_of(index).map(|child| (child, depth + 1, Some(index))));
    }
    placed.extend(
        (0..rows.len())
            .filter(|&index| !seen[index])
            .map(|index| (index, 0, None)),
    );
    placed
}

/// Each row's native item, from the items the control walked in `nesting`'s
/// order: `handles[p]` belongs to row `placed[p].0`, so the answer is indexed
/// by row. `None` when the walk found a different number of items from the
/// rows placed, because a handle matched to the wrong row would read one
/// folder's tick as another's and write a choice nobody made; a caller then
/// leaves everything as it was.
pub fn handles_by_row<H: Copy>(
    placed: &[(usize, usize, Option<usize>)],
    handles: &[H],
) -> Option<Vec<H>> {
    if placed.len() != handles.len() {
        return None;
    }
    let mut by_row: Vec<Option<H>> = vec![None; placed.len()];
    for ((row, _, _), handle) in placed.iter().zip(handles) {
        by_row[*row] = Some(*handle);
    }
    by_row.into_iter().collect()
}

/// Whether an account is Gmail, by either fact the account row carries: the
/// provider it was made through, or the server it points at.
///
/// Both, because an account added by hand to `imap.gmail.com` has no
/// provider, and one made through the provider list has the provider whatever
/// its server spelling. The host is compared ignoring case because a host
/// name is one.
pub fn is_a_gmail_account(provider: Option<&str>, imap_server: &str) -> bool {
    provider == Some("Gmail") || imap_server.eq_ignore_ascii_case("imap.gmail.com")
}

/// The one sentence the dialog adds when Gmail did not list All Mail.
///
/// Only for Gmail, and only when no listed folder holds every message: a
/// Gmail account whose All Mail is in the tree needs no explanation, and
/// another provider has no All Mail to miss. Gmail hides a label from IMAP
/// when its Show in IMAP box is off, and a hidden label is absent from the
/// list this program reads folders from, so the dialog cannot show a row the
/// server never sent and says so instead.
pub fn the_all_mail_sentence(is_gmail: bool, any_holds_all_mail: bool) -> Option<String> {
    (is_gmail && !any_holds_all_mail).then(|| {
        "Gmail did not list All Mail, so it cannot be kept up to date from here. Gmail's own \
         settings, Labels, Show in IMAP, decide which labels it lists."
            .to_string()
    })
}

/// Ask which folders to keep up to date.
///
/// `account_name` is what the account manager calls the account, which is
/// what the title says; the tester heard an identifier there (#70, third
/// point). `is_gmail` decides whether a missing All Mail is worth a sentence.
///
/// `None` when somebody cancelled, which must leave everything as it was, and
/// on a platform where a tick would reach no screen reader, where nothing is
/// opened. Otherwise the folders whose answer changed, each with its new
/// state, so the caller writes only what was actually decided rather than
/// restating every folder as a choice somebody made.
pub fn ask(
    parent: &Frame,
    account_name: &str,
    is_gmail: bool,
    folders: &[FolderRow],
) -> Option<Vec<(String, bool)>> {
    if folders.is_empty() || !native_tree_checks::CHECK_STATES_REACH_A_SCREEN_READER {
        return None;
    }
    let (dialog, tree) = build_folder_choice_dialog(
        parent,
        account_name,
        is_gmail,
        folders,
        theme::current_from_stored_config(),
    );

    let answer = dialog.show_modal();
    let changed = (answer == ID_OK)
        .then(|| what_the_tree_shows(&tree, folders))
        .flatten();
    dialog.destroy();
    changed
}

/// The answers as the control shows them at this moment, read item by item
/// from its own state, or `None` when the control's items cannot be matched
/// to the rows, in which case nothing is written.
fn what_the_tree_shows(tree: &TreeCtrl, folders: &[FolderRow]) -> Option<Vec<(String, bool)>> {
    let by_row = handles_by_row(
        &nesting(folders),
        &native_tree_checks::items_in_walk_order(tree),
    );
    if by_row.is_none() {
        tracing::warn!(
            "The folder chooser's tree does not hold one item per folder, so no choice was written"
        );
    }
    let by_row = by_row?;
    Some(changes(folders, |row| {
        native_tree_checks::is_checked(tree, by_row[row])
    }))
}

/// Build the folder chooser without showing it.
///
/// Everything `ask` used to do up to its own `.show_modal()` call, split out
/// the same way [`crate::presentation::wx_settings::build_settings_dialog`]
/// splits Settings: a test can build the real dialog and read back what the
/// live control answers, and never call `.show_modal()` at all.
///
/// `folders` must not be empty; `ask` checks that before calling here, since
/// there is nothing to build a tree out of otherwise.
///
/// Returns the tree alongside the dialog, the same way the caller needs it
/// after a real `.show_modal()`: to read which items are ticked.
pub fn build_folder_choice_dialog(
    parent: &Frame,
    account_name: &str,
    is_gmail: bool,
    folders: &[FolderRow],
    palette: Option<theme::Palette>,
) -> (Dialog, TreeCtrl) {
    let dialog = Dialog::builder(
        parent,
        &format!("Folders to keep up to date: {account_name}"),
    )
    .with_size(560, 460)
    .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
    .build();

    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let explain = StaticText::builder(&dialog)
        .with_label(
            "Space ticks the folder under the cursor. Right arrow opens a folder that holds \
             others. Folders you untick stay on the server and stop being downloaded.",
        )
        .build();
    sizer.add(&explain, 0, SizerFlag::All | SizerFlag::Expand, 8);

    let tree = TreeCtrl::builder(&dialog)
        .with_style(
            TreeCtrlStyle::HideRoot | TreeCtrlStyle::HasButtons | TreeCtrlStyle::LinesAtRoot,
        )
        .build();
    // Before the first item, which is the order the style needs; see the
    // module it comes from.
    native_tree_checks::add_check_boxes(&tree);
    set_accessible_name(&tree, "Folders to keep up to date");
    let placed = nesting(folders);
    let mut items: Vec<Option<TreeItemId>> = vec![None; folders.len()];
    if let Some(root) = tree.add_root("Folders", None, None) {
        for (row, _depth, parent) in &placed {
            let under = parent.and_then(|at| items[at].as_ref()).unwrap_or(&root);
            items[*row] = tree.append_item(under, &row_label(&folders[*row]), None, None);
        }
    }
    // Every branch open, so a nested folder is reachable by arrowing down
    // without knowing it is there, which somebody working by ear does not.
    for item in items.iter().flatten() {
        tree.expand(item);
    }
    // The ticks, through the control's own state, matched to the rows by the
    // walk the module describes. A walk that does not match the rows ticks
    // nothing rather than ticking the wrong folder.
    if let Some(by_row) = handles_by_row(&placed, &native_tree_checks::items_in_walk_order(&tree)) {
        for (row, folder) in folders.iter().enumerate() {
            native_tree_checks::set_checked(&tree, by_row[row], folder.syncing);
        }
    }
    sizer.add(&tree, 1, SizerFlag::All | SizerFlag::Expand, 8);

    if let Some(sentence) =
        the_all_mail_sentence(is_gmail, folders.iter().any(|folder| folder.holds_all_mail))
    {
        let said = StaticText::builder(&dialog).with_label(&sentence).build();
        set_accessible_name(&said, &sentence);
        sizer.add(&said, 0, SizerFlag::All | SizerFlag::Expand, 8);
    }

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    let save = Button::builder(&dialog)
        .with_label("&Save")
        .with_id(ID_OK)
        .build();
    set_accessible_name(&save, "Save");
    let cancel = Button::builder(&dialog)
        .with_label("&Cancel")
        .with_id(ID_CANCEL)
        .build();
    set_accessible_name(&cancel, "Cancel");
    buttons.add(&save, 0, SizerFlag::All, 6);
    buttons.add(&cancel, 0, SizerFlag::All, 6);
    sizer.add_sizer(&buttons, 0, SizerFlag::AlignRight | SizerFlag::All, 8);

    dialog.set_sizer(sizer, true);
    // Focus starts in the tree rather than on Save, so the first thing heard is
    // a folder and its state rather than a button, and the cursor is put on the
    // first item, because a tree where nothing is selected has no folder to
    // announce.
    if let Some(first) = placed.first().and_then(|(row, _, _)| items[*row].as_ref()) {
        tree.select_item(first);
        tree.set_focused_item(first);
    }
    tree.set_focus();

    // Painted last. The tree is left to Windows here, the same as the tree in
    // the "where does this go" dialog and every `Choice`, `ComboBox`,
    // `RadioButton` and `CheckBox` elsewhere in this round; only the dialog is
    // painted. `None` means high contrast is on, or the system is set up in a
    // way this application should not paint over, so nothing is set here and
    // Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dialog, palette.main_surface());
    }

    (dialog, tree)
}

/// Which answers differ from what they were.
///
/// Split out from the dialog because it is the part that can be wrong in a way
/// nobody sees: writing every folder back would record a deliberate choice
/// against folders nobody touched, and those then stop following the default
/// for ever.
fn changes(folders: &[FolderRow], ticked: impl Fn(usize) -> bool) -> Vec<(String, bool)> {
    folders
        .iter()
        .enumerate()
        .filter(|(index, folder)| ticked(*index) != folder.syncing)
        .map(|(index, folder)| (folder.path.clone(), ticked(index)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(name: &str) -> FolderRow {
        FolderRow {
            path: name.to_string(),
            name: name.to_string(),
            parent: None,
            syncing: true,
            subscribed: true,
            holds_all_mail: false,
            total: 42,
        }
    }

    fn folder_under(path: &str, parent: &str) -> FolderRow {
        let mut row = folder(path);
        row.parent = Some(parent.to_string());
        row
    }

    #[test]
    fn test_a_child_comes_after_its_parent_one_deeper_and_names_it() {
        // [Gmail]/QC Docs is stored before INBOX here, and the tree still puts
        // it under [Gmail], one level down, pointing back at its parent's row.
        let rows = [
            folder_under("[Gmail]/QC Docs", "[Gmail]"),
            folder("INBOX"),
            folder("[Gmail]"),
        ];

        let nested = nesting(&rows);

        assert_eq!(nested, vec![(1, 0, None), (2, 0, None), (0, 1, Some(2))]);
    }

    #[test]
    fn test_a_grandchild_is_two_deep_and_follows_its_parent_before_the_next_top_row() {
        let rows = [
            folder("[Gmail]"),
            folder("Archive"),
            folder_under("[Gmail]/QC Docs", "[Gmail]"),
            folder_under("[Gmail]/QC Docs/QILC", "[Gmail]/QC Docs"),
        ];

        let nested = nesting(&rows);

        assert_eq!(
            nested,
            vec![(0, 0, None), (2, 1, Some(0)), (3, 2, Some(2)), (1, 0, None)]
        );
    }

    #[test]
    fn test_siblings_keep_the_order_the_server_listed_them_in() {
        let rows = [folder("Work"), folder("INBOX"), folder("Archive")];

        let nested = nesting(&rows);

        assert_eq!(nested, vec![(0, 0, None), (1, 0, None), (2, 0, None)]);
    }

    #[test]
    fn test_a_folder_whose_parent_was_not_listed_sits_at_the_top_rather_than_vanishing() {
        // The server can list a child and hide its parent; Gmail does exactly
        // that for a label whose parent is not shown in IMAP.
        let rows = [folder("INBOX"), folder_under("Hidden/Seen", "Hidden")];

        let nested = nesting(&rows);

        assert_eq!(nested, vec![(0, 0, None), (1, 0, None)]);
    }

    #[test]
    fn test_folders_whose_parents_form_a_loop_still_appear_rather_than_hanging_the_dialog() {
        // A stored parent_id cannot loop, and a dialog must not hang if one
        // ever does; the two rows sit at the top and nothing is lost.
        let rows = [
            folder("INBOX"),
            folder_under("A", "B"),
            folder_under("B", "A"),
        ];

        let nested = nesting(&rows);

        assert_eq!(nested, vec![(0, 0, None), (1, 0, None), (2, 0, None)]);
    }

    #[test]
    fn test_each_row_gets_the_handle_the_walk_found_at_its_place() {
        // Rows stored as [child, top A, top B]; placed as A, then its child,
        // then B; the walk answers three handles in that order.
        let placed = [(1, 0, None), (0, 1, Some(1)), (2, 0, None)];

        let by_row = handles_by_row(&placed, &['a', 'c', 'b']);

        assert_eq!(by_row, Some(vec!['c', 'a', 'b']));
    }

    #[test]
    fn test_a_walk_of_the_wrong_length_maps_nothing_rather_than_one_row_wrong() {
        let placed = [(0, 0, None), (1, 0, None)];

        assert_eq!(handles_by_row(&placed, &['a']), None);
        assert_eq!(handles_by_row(&placed, &['a', 'b', 'c']), None);
    }

    #[test]
    fn test_an_account_made_through_the_provider_list_is_gmail_by_its_provider() {
        assert!(is_a_gmail_account(Some("Gmail"), "imap.gmail.com"));
        assert!(is_a_gmail_account(Some("Gmail"), "imap.example.org"));
    }

    #[test]
    fn test_an_account_pointed_at_gmails_server_is_gmail_whatever_the_case_of_the_host() {
        assert!(is_a_gmail_account(None, "imap.gmail.com"));
        assert!(is_a_gmail_account(None, "IMAP.Gmail.com"));
        assert!(is_a_gmail_account(Some("Custom"), "imap.gmail.com"));
    }

    #[test]
    fn test_an_account_on_another_server_with_another_provider_is_not_gmail() {
        assert!(!is_a_gmail_account(None, "imap.example.org"));
        assert!(!is_a_gmail_account(
            Some("Outlook"),
            "outlook.office365.com"
        ));
    }

    #[test]
    fn test_gmail_with_no_folder_holding_every_message_gets_the_sentence() {
        let said =
            the_all_mail_sentence(true, false).expect("a sentence for Gmail with no All Mail");

        assert!(said.starts_with("Gmail did not list All Mail"), "{said}");
        assert!(said.contains("Show in IMAP"), "{said}");
    }

    #[test]
    fn test_gmail_with_all_mail_listed_gets_no_sentence() {
        assert_eq!(the_all_mail_sentence(true, true), None);
    }

    #[test]
    fn test_another_provider_gets_no_sentence_whether_or_not_a_folder_holds_everything() {
        assert_eq!(the_all_mail_sentence(false, false), None);
        assert_eq!(the_all_mail_sentence(false, true), None);
    }

    #[test]
    fn test_a_row_says_its_name_first() {
        // A screen reader reads from the start, and the name is what somebody
        // is looking for.
        let said = row_label(&folder("Work"));

        assert!(said.starts_with("Work"), "{said}");
    }

    #[test]
    fn test_a_row_says_how_much_is_in_the_folder() {
        // The number is what the answer turns on.
        assert!(row_label(&folder("Work")).contains("42 messages"));
    }

    #[test]
    fn test_one_message_is_not_one_messages() {
        let mut lonely = folder("Receipts");
        lonely.total = 1;

        assert!(
            row_label(&lonely).contains("1 message,") || row_label(&lonely).ends_with("1 message")
        );
        assert!(!row_label(&lonely).contains("1 messages"));
    }

    #[test]
    fn test_the_folder_holding_everything_says_so_instead_of_a_count() {
        // Its count is the whole account, which read on its own sounds like a
        // large folder rather than a second copy of every other folder.
        let mut all_mail = folder("[Gmail]/All Mail");
        all_mail.holds_all_mail = true;

        let said = row_label(&all_mail);

        assert!(said.contains("every message"), "{said}");
        assert!(said.contains("doubles"), "{said}");
    }

    #[test]
    fn test_a_folder_nobody_subscribed_to_says_so() {
        let mut old = folder("Old backups 2009");
        old.subscribed = false;

        assert!(row_label(&old).contains("not subscribed"));
    }

    #[test]
    fn test_only_the_answers_that_changed_come_back() {
        // Writing every folder back would record a deliberate choice against
        // folders nobody touched, and those would then stop following the
        // default for ever, including when the default is what fixes them.
        let mut off = folder("Archive");
        off.syncing = false;
        let folders = [folder("INBOX"), off, folder("Work")];

        // Everything ticked. Only the middle one was not before.
        let changed = changes(&folders, |_| true);

        assert_eq!(changed, vec![("Archive".to_string(), true)]);
    }

    #[test]
    fn test_turning_one_off_is_reported() {
        let folders = [folder("INBOX"), folder("Work")];

        let changed = changes(&folders, |index| index == 0);

        assert_eq!(changed, vec![("Work".to_string(), false)]);
    }

    #[test]
    fn test_changing_nothing_writes_nothing() {
        let folders = [folder("INBOX"), folder("Work")];

        assert!(changes(&folders, |_| true).is_empty());
    }
}

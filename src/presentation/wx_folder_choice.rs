//! Choosing which folders an account keeps up to date.
//!
//! A mail server lists every mailbox the account can see. On a shared server or
//! a university system that is routinely dozens or hundreds, and somebody uses
//! four of them. On Gmail it includes All Mail, which holds a copy of every
//! message in the account, so keeping it up to date means downloading the whole
//! account a second time and reading every message twice in the list.
//!
//! So this asks. A checked list, one row per folder, with the row saying how
//! many messages the folder holds and whether it is one somebody is subscribed
//! to. What is ticked when the window opens is what the sync would do anyway,
//! so somebody who does not care can close it and lose nothing.
//!
//! # Why a checked list and not a tree
//!
//! The folder tree in the main window is one flat level, so a tree here would
//! be a tree with no branches. A checked list is a single control where Space
//! ticks the row under the cursor and arrows move, which is the plainest
//! arrangement there is for exactly this question.
//!
//! # Making the tick reach a screen reader
//!
//! On Windows wxWidgets draws these check boxes itself rather than using a
//! control that has them, so the platform sees a plain list and the ticked
//! state reaches nobody. On a window whose entire purpose is ticking things,
//! that is the window.
//!
//! So each row reports itself, through
//! [`set_accessible_checked_rows`](crate::presentation::accessibility::names::set_accessible_checked_rows),
//! as a check box with a checked state, and the row the cursor is on reports
//! that it is the current one. It is the same fix NVDA makes in its own
//! settings, where the problem is identical and the answer is written in
//! Python: answer for the rows and not only for the control.
//!
//! The snapshot is refreshed on every tick and every move of the cursor, by the
//! list itself rather than by anything here. A state read once when the window
//! opened would announce confidently and be wrong from the first press of
//! Space, which is worse than announcing nothing.
//!
//! **Still to be confirmed with a screen reader.** The structure is there and
//! the reasoning is sound; whether NVDA says "ticked" is a thing only NVDA can
//! answer. Sixteen controls in this application were once named by a call that
//! compiled, passed the tests and never reached a screen reader.
//!
//! The row's own text carries everything except the tick: the folder's name,
//! how much is in it, and the warning about the one folder that doubles what
//! gets downloaded. Those are read whatever happens to the check box.

use crate::presentation::accessibility::names::{set_accessible_checked_rows, set_accessible_name};
use crate::presentation::theme;
use wxdragon::prelude::*;

/// One folder, and whether it is currently kept up to date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderRow {
    /// The server's own spelling, which is the identifier.
    pub path: String,
    /// What to show and announce.
    pub name: String,
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

/// Ask which folders to keep up to date.
///
/// `None` when somebody cancelled, which must leave everything as it was.
/// Otherwise the folders whose answer changed, each with its new state, so the
/// caller writes only what was actually decided rather than restating every
/// folder as a choice somebody made.
pub fn ask(parent: &Frame, account: &str, folders: &[FolderRow]) -> Option<Vec<(String, bool)>> {
    if folders.is_empty() {
        return None;
    }
    let (dialog, list) = build_folder_choice_dialog(
        parent,
        account,
        folders,
        theme::current_from_stored_config(),
    );

    let answer = dialog.show_modal();
    let changed = if answer == ID_OK {
        Some(changes(folders, |index| list.is_checked(index as u32)))
    } else {
        None
    };
    dialog.destroy();
    changed
}

/// Build the folder chooser without showing it.
///
/// Everything `ask` used to do up to its own `.show_modal()` call, split out
/// the same way [`crate::presentation::wx_settings::build_settings_dialog`]
/// splits Settings: a test can build the real dialog and read back the real
/// colour a live control holds, and never call `.show_modal()` at all.
///
/// `folders` must not be empty; `ask` checks that before calling here, since
/// there is nothing to build a list out of otherwise.
///
/// Returns the list alongside the dialog, the same way the caller needs it
/// after a real `.show_modal()`: to read which rows are ticked.
pub fn build_folder_choice_dialog(
    parent: &Frame,
    account: &str,
    folders: &[FolderRow],
    palette: Option<theme::Palette>,
) -> (Dialog, CheckListBox) {
    let dialog = Dialog::builder(parent, &format!("Folders to keep up to date: {account}"))
        .with_size(560, 460)
        .with_style(DialogStyle::DefaultDialogStyle | DialogStyle::ResizeBorder)
        .build();

    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    let explain = StaticText::builder(&dialog)
        .with_label(
            "Space ticks the folder under the cursor. Folders you untick stay on the \
             server and stop being downloaded.",
        )
        .build();
    sizer.add(&explain, 0, SizerFlag::All | SizerFlag::Expand, 8);

    let list = CheckListBox::builder(&dialog).build();
    for folder in folders {
        list.append(&row_label(folder));
    }
    for (index, folder) in folders.iter().enumerate() {
        // Set after every row exists. Checking as they are appended works on
        // Windows and is not guaranteed to, and a tick that silently lands on
        // the wrong row is a folder somebody did not ask for.
        list.check(index as u32, folder.syncing);
    }
    // Each row reports itself as a check box with its ticked state and says
    // whether it is the one the cursor is on, because Windows draws these check
    // boxes rather than using a control that has them, so without this neither
    // fact reaches anybody. It selects the first row, reads the list, and keeps
    // reading it as rows are ticked and the cursor moves, so nothing here has
    // to remember to.
    set_accessible_checked_rows(
        list,
        "Folders to keep up to date",
        "Tick a folder to download its messages. Untick one to leave it on the server.",
    );
    sizer.add(&list, 1, SizerFlag::All | SizerFlag::Expand, 8);

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
    // Focus starts in the list rather than on Save, so the first thing heard is
    // a folder and its state rather than a button. The first row is already the
    // current one, chosen when the rows were given their accessible state,
    // because a list where nothing is selected has no folder to announce.
    list.set_focus();

    // Painted last. The `CheckListBox` draws its own check marks rather than
    // going through a control the established pattern paints, so it is left
    // to Windows here, the same as every `Choice`, `ComboBox`, `RadioButton`
    // and `CheckBox` elsewhere in this round. `None` means high contrast is
    // on, or the system is set up in a way this application should not paint
    // over, so nothing is set here and Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dialog, palette.main_surface());
    }

    (dialog, list)
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
            syncing: true,
            subscribed: true,
            holds_all_mail: false,
            total: 42,
        }
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

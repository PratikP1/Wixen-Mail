//! Accessible names for controls that carry no visible label.
//!
//! A list, tree, or text field with no label next to it reaches the UI
//! Automation tree with a null Name. A screen reader then announces it as
//! "list" or "tree" with nothing to say which one, which is the difference
//! between a usable window and a guessing game.
//!
//! `wxWindow::SetName` does **not** do this. That name is an internal
//! wxWidgets identifier used for resource lookup and never reaches assistive
//! technology. Only a `wxAccessible` attached to the window does, which is
//! what this module provides.
//!
//! **This is Windows only.** `wxAccessible` is implemented against Microsoft
//! Active Accessibility and has no GTK or macOS counterpart in wxWidgets, so on
//! those platforms these calls are accepted and have no effect. A port would
//! need `setAccessibilityLabel:` on the NSView for macOS and an ATK name for
//! GTK, written as separate bridges rather than as a change here.
//!
//! # Which reader hears what, measured
//!
//! What is set here is an MSAA name, and for a control Windows draws itself,
//! that is not what UI Automation reports. Windows supplies its own UIA
//! provider for a native edit, button or combo box, and it answers before the
//! bridge that would otherwise expose this object. Read through the two APIs
//! side by side against the running composer, the same control answers twice:
//!
//! ```text
//! Button   UIA='B'                MSAA='Bold, Ctrl+B'
//! Button   UIA='Attach File...'   MSAA='Attach a file, Alt+A'
//! Edit     UIA='To:'              MSAA='To'
//! ```
//!
//! So these names are not lost, and a reader that uses IAccessible for ordinary
//! Win32 controls, which is what NVDA does in a window like this one, hears
//! exactly what is written here, descriptions included. A reader that uses UI
//! Automation, which is what Narrator does, hears the control's own window
//! text: the visible label for a button, the static label beside a field.
//!
//! Three things follow, and they are the reason this is written down.
//!
//! Set a name here **and** make the visible label say the same thing, wherever
//! the two can agree. Where they cannot, the name here is the fuller one and
//! the label is the shorter one, never a different fact.
//!
//! A description set here reaches MSAA only. Anything somebody must hear to
//! work a control cannot live only in a description; put it in the label, or
//! announce it.
//!
//! And the automated scan reads the UI Automation tree, so it is measuring the
//! labels rather than any of this. A clean scan says the labels are present. It
//! says nothing at all about these.

use wxdragon::accessible::{AccStatus, Accessible, AccessibleImpl};
use wxdragon::ffi;
use wxdragon::prelude::WxWidget;

/// Supplies one fixed name, and optionally a description, for a control,
/// leaving every other accessibility property to the platform's default
/// handling.
struct FixedName {
    name: String,
    description: Option<String>,
}

impl AccessibleImpl for FixedName {
    /// Name the control itself, and nothing inside it.
    ///
    /// Child zero is the control. Anything else is a row, a tree node, or a
    /// notebook tab asking for its own name, and answering those with the
    /// control's name gave every one of them the same label. That is why the
    /// settings tabs stayed silent even after child enumeration was restored:
    /// enumeration worked, and then every tab reported itself as "Settings
    /// categories".
    fn get_name(&self, child_id: i32) -> (AccStatus, Option<String>) {
        if child_id != 0 {
            return (ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED, None);
        }
        (ffi::wxd_AccStatus_WXD_ACC_OK, Some(self.name.clone()))
    }

    /// Defer to the control's own child enumeration.
    ///
    /// This override is not optional. The trait's default answers `OK` with a
    /// count of zero, which is a positive claim that the control has no
    /// children rather than a request to fall back. Attaching one of these to a
    /// notebook silenced its tabs, and to a list or tree it would have hidden
    /// every item, because the accessible object was answering "nothing in
    /// here" on the control's behalf.
    ///
    /// Every other method left at its default returns `NOT_IMPLEMENTED`, which
    /// is what makes wxWidgets use its own implementation. Only the name and
    /// the description are meant to be replaced here.
    fn get_child_count(&self) -> (AccStatus, i32) {
        (ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED, 0)
    }

    /// The sentence a screen reader reads after the name, on focus.
    ///
    /// This is the property that carries "what does choosing this cost me".
    /// Putting that in the name instead makes it part of every announcement,
    /// including the one when arrowing past, which is how a four-sentence
    /// radio button label happens. A description is spoken once, when the
    /// control takes focus.
    ///
    /// Same rule as the name about child zero: a row or a tab asking for its
    /// own description must not be handed the control's.
    fn get_description(&self, child_id: i32) -> (AccStatus, Option<String>) {
        match (&self.description, child_id) {
            (Some(description), 0) => (ffi::wxd_AccStatus_WXD_ACC_OK, Some(description.clone())),
            _ => (ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED, None),
        }
    }
}

/// Reports each row of a checked list as a check box that is ticked or not.
///
/// On Windows, wxWidgets draws the check boxes in a `wxCheckListBox` itself
/// rather than using a control that has them, so the platform sees a plain list
/// and the tick reaches nobody. A screen reader reads the row's text and says
/// nothing about whether it is on, which on a window whose entire purpose is
/// ticking things is the whole window.
///
/// The fix is the one NVDA uses in its own settings, where the same problem
/// exists and is solved in Python: attach an accessible object that answers for
/// each row rather than only for the control, give the row the check box role,
/// and put the checked state in its state flags. That is what MSAA has for
/// exactly this, and what a screen reader is already listening for.
///
/// Child ids are one-based, which is MSAA's convention: nought is the control
/// itself and every row after that counts from one. Getting that wrong reports
/// each row's state one row out, which reads plausibly and is wrong everywhere.
struct CheckedRows {
    name: String,
    description: String,
    /// Whether each row is ticked, in the order the rows were added.
    ///
    /// A snapshot rather than a live look at the control, because an accessible
    /// object is called from the screen reader's thread and must not reach into
    /// a widget. It is replaced whenever a tick changes, which is the only time
    /// it can go stale.
    ticked: std::sync::Arc<std::sync::Mutex<Vec<bool>>>,
}

impl AccessibleImpl for CheckedRows {
    /// Defer to the list's own row enumeration.
    ///
    /// The same override, for the same reason, as the one on the fixed name
    /// above: the trait's default answers `OK` with a count of nought, which
    /// wxWidgets takes as the real answer rather than a request to fall back.
    /// Without this, an object written to report each row's tick reported that
    /// there were no rows, so every folder vanished from the accessibility tree
    /// and nothing was left to carry a tick.
    fn get_child_count(&self) -> (AccStatus, i32) {
        (ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED, 0)
    }

    fn get_name(&self, child_id: i32) -> (AccStatus, Option<String>) {
        if child_id == 0 {
            return (ffi::wxd_AccStatus_WXD_ACC_OK, Some(self.name.clone()));
        }
        // The row's own text is the platform's to give: it is what was put in
        // the list, and repeating it here would be one more copy to keep in
        // step for no gain.
        (ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED, None)
    }

    fn get_description(&self, child_id: i32) -> (AccStatus, Option<String>) {
        match child_id {
            0 => (
                ffi::wxd_AccStatus_WXD_ACC_OK,
                Some(self.description.clone()),
            ),
            _ => (ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED, None),
        }
    }

    fn get_role(&self, child_id: i32) -> (AccStatus, wxdragon::accessible::AccRole) {
        if child_id == 0 {
            return (
                ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED,
                ffi::wxd_AccRole_WXD_ROLE_SYSTEM_LIST,
            );
        }
        // A row is a check box, which is what makes a screen reader look for a
        // checked state and say "ticked" or "not ticked" rather than reading
        // the text and stopping.
        (
            ffi::wxd_AccStatus_WXD_ACC_OK,
            ffi::wxd_AccRole_WXD_ROLE_SYSTEM_CHECKBUTTON,
        )
    }

    fn get_state(&self, child_id: i32) -> (AccStatus, i64) {
        if child_id == 0 {
            return (ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED, 0);
        }
        let Ok(ticked) = self.ticked.lock() else {
            // A poisoned lock is not a reason to claim a row is unticked, which
            // would be a wrong answer rather than no answer.
            return (ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED, 0);
        };
        // One-based, as MSAA counts children.
        let Some(on) = ticked.get((child_id - 1) as usize) else {
            return (ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED, 0);
        };
        let state = if *on {
            wxdragon::accessible::acc_state::CHECKED
        } else {
            0
        };
        (
            ffi::wxd_AccStatus_WXD_ACC_OK,
            state
                | wxdragon::accessible::acc_state::FOCUSABLE
                | wxdragon::accessible::acc_state::SELECTABLE,
        )
    }
}

/// What a checked list's rows report, so the ticks can be kept up to date.
///
/// Handed back from [`set_accessible_checked_rows`] so the caller can replace
/// the snapshot when somebody ticks a row. Without that the state is whatever
/// it was when the window opened, which is worse than none: it would announce
/// confidently and be wrong the moment anybody pressed Space.
pub type TickState = std::sync::Arc<std::sync::Mutex<Vec<bool>>>;

/// Give a checked list a name and rows that report whether they are ticked.
///
/// The returned handle holds the ticked state. Update it whenever a row is
/// toggled, in the same order the rows were added.
pub fn set_accessible_checked_rows(
    window: &dyn WxWidget,
    name: &str,
    description: &str,
    ticked: Vec<bool>,
) -> TickState {
    let (rows, state) = checked_rows(name, description, ticked);
    window.set_accessible(Accessible::new(window, rows));
    state
}

/// Work out what the rows will report, and the handle that changes it.
///
/// Kept apart from attaching them to a window so the contract the whole design
/// rests on can be stated on its own: the handle handed back is the same
/// snapshot the rows read, so writing a new set of ticks through it changes
/// what a row answers. Attaching is a call into wxWidgets that cannot be read
/// back, so it can only be checked with a screen reader.
fn checked_rows(name: &str, description: &str, ticked: Vec<bool>) -> (CheckedRows, TickState) {
    let state: TickState = std::sync::Arc::new(std::sync::Mutex::new(ticked));
    let rows = CheckedRows {
        name: name.to_string(),
        description: description.to_string(),
        ticked: state.clone(),
    };
    (rows, state)
}

/// Give `window` an accessible name that screen readers will announce.
///
/// Use the same wording as the control's visible heading or tree root, so what
/// is spoken matches what is on screen.
pub fn set_accessible_name(window: &dyn WxWidget, name: &str) {
    window.set_accessible(Accessible::new(
        window,
        FixedName {
            name: name.to_string(),
            description: None,
        },
    ));
}

/// Give `window` a name and the sentence that explains it.
///
/// For a control whose label is not enough on its own: a choice with a
/// consequence, a field with a rule about what it accepts. Screen readers read
/// the description after the name when the control takes focus, so what is on
/// screen next to the control is heard by somebody who cannot see it.
///
/// Without this, an explanation sitting beside a control is a label floating in
/// the window that a screen reader user reaches only by leaving the control and
/// reading around, if they think to. The first-run screen shipped that way:
/// three radio buttons that read correctly and three explanations of what each
/// one costs that were never spoken.
///
/// One call, not two. Attaching an accessible object replaces the last one, so
/// setting a name and then a description would leave only the description.
pub fn set_accessible_name_and_description(window: &dyn WxWidget, name: &str, description: &str) {
    window.set_accessible(Accessible::new(
        window,
        FixedName {
            name: name.to_string(),
            description: Some(description.to_string()),
        },
    ));
}

/// Turn a visible label into the name a screen reader should announce.
///
/// Drops the mnemonic ampersand and any trailing colon. Both are visual
/// conventions: spoken, "and Subject colon" is worse than "Subject", and some
/// screen readers read the ampersand aloud.
pub fn name_from_label(label: &str) -> String {
    label
        .replace('&', "")
        .trim()
        .trim_end_matches(':')
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{AccessibleImpl, CheckedRows, FixedName, name_from_label};
    use wxdragon::accessible::acc_state::{CHECKED, FOCUSABLE, SELECTABLE};
    use wxdragon::ffi;

    /// A checked list carrying the wording the folder chooser really uses.
    ///
    /// Every test below pins what this code *answers* when the platform asks
    /// it. Whether a screen reader then speaks the answer is a separate
    /// question that only a screen reader pass settles.
    fn checked_list(ticked: Vec<bool>) -> CheckedRows {
        CheckedRows {
            name: "Folders to keep up to date".to_string(),
            description: "Tick a folder to download its messages.".to_string(),
            ticked: std::sync::Arc::new(std::sync::Mutex::new(ticked)),
        }
    }

    #[test]
    fn test_a_checked_list_does_not_claim_it_has_no_rows() {
        // The trait default answers OK with a count of zero, which is a
        // positive claim that the list is empty rather than a request to fall
        // back, and it is written through whatever the control really holds.
        // Without this override the folder chooser reported no folders at all,
        // which is worse than the missing tick this class was written to fix.
        let list = checked_list(vec![true, false]);

        let (status, count) = list.get_child_count();

        assert_eq!(
            status,
            ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED,
            "reporting a row's tick must not hide the rows"
        );
        assert_eq!(count, 0);
    }

    #[test]
    fn test_a_checked_list_answers_with_its_own_name_when_asked_for_child_zero() {
        let list = checked_list(vec![true]);

        let (status, name) = list.get_name(0);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_OK);
        assert_eq!(name.as_deref(), Some("Folders to keep up to date"));
    }

    #[test]
    fn test_a_row_is_not_handed_the_lists_name() {
        // The same trap the settings tabs fell into, where every tab reported
        // itself as "Settings categories". Here it would give every folder the
        // name of the list holding them.
        let list = checked_list(vec![true, false]);

        let (status, name) = list.get_name(1);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED);
        assert!(name.is_none());
    }

    #[test]
    fn test_a_checked_list_answers_with_its_description_when_asked_for_child_zero() {
        let list = checked_list(vec![true]);

        let (status, description) = list.get_description(0);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_OK);
        assert_eq!(
            description.as_deref(),
            Some("Tick a folder to download its messages.")
        );
    }

    #[test]
    fn test_a_row_is_not_handed_the_lists_description() {
        // Handing it down would read the list's instructions again on every
        // arrow key, which is the flooding an announcement is supposed to
        // avoid.
        let list = checked_list(vec![true, false]);

        let (status, description) = list.get_description(2);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED);
        assert!(description.is_none());
    }

    #[test]
    fn test_a_row_asks_to_be_a_check_box() {
        // The whole point of the class. Windows draws these ticks itself, so
        // without the check box role a screen reader reads the folder's name
        // and stops, with nothing to say whether it is on.
        let list = checked_list(vec![true]);

        let (status, role) = list.get_role(1);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_OK);
        assert_eq!(role, ffi::wxd_AccRole_WXD_ROLE_SYSTEM_CHECKBUTTON);
    }

    #[test]
    fn test_the_list_itself_does_not_claim_to_be_a_check_box() {
        // Only the status is worth stating: wxWidgets reads the role back only
        // when the answer is OK, so the value beside NOT_IMPLEMENTED reaches
        // nobody and asserting on it would pin a fact nothing reads.
        let list = checked_list(vec![true]);

        let (status, _role) = list.get_role(0);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED);
    }

    #[test]
    fn test_a_ticked_row_answers_with_the_checked_flag() {
        let list = checked_list(vec![true]);

        let (status, state) = list.get_state(1);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_OK);
        assert!(state & CHECKED != 0, "a ticked row must report it");
    }

    #[test]
    fn test_an_unticked_row_answers_without_the_checked_flag() {
        let list = checked_list(vec![false]);

        let (status, state) = list.get_state(1);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_OK);
        assert!(
            state & CHECKED == 0,
            "an unticked row must not report a tick"
        );
    }

    #[test]
    fn test_the_list_itself_leaves_its_state_to_the_platform() {
        // Child nought is the list, not a row. Answering for it with a row's
        // flags would report the whole control as ticked.
        let list = checked_list(vec![true]);

        let (status, state) = list.get_state(0);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED);
        assert_eq!(state, 0);
    }

    #[test]
    fn test_the_first_row_is_child_one_and_reads_the_first_tick() {
        // Child ids count from one, as Microsoft Active Accessibility does.
        // Counting them from nought reports every row's state one row out,
        // which reads plausibly and is wrong everywhere.
        let list = checked_list(vec![true, false, false]);

        let (_status, state) = list.get_state(1);

        assert!(
            state & CHECKED != 0,
            "the first row must read the first tick"
        );
    }

    #[test]
    fn test_a_row_past_the_end_of_the_snapshot_answers_nothing_rather_than_unticked() {
        // Answering OK with no flags would be a claim that the row is not
        // ticked, which is a wrong answer rather than no answer.
        let list = checked_list(vec![true, true]);

        let (status, state) = list.get_state(3);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED);
        assert_eq!(state, 0);
    }

    #[test]
    fn test_a_poisoned_snapshot_answers_nothing_rather_than_unticked() {
        let ticked = std::sync::Arc::new(std::sync::Mutex::new(vec![true]));
        let poison = ticked.clone();
        let _ = std::thread::spawn(move || {
            let _held = poison.lock().expect("a fresh mutex is not poisoned");
            panic!("poison the snapshot");
        })
        .join();
        let list = CheckedRows {
            name: "Folders to keep up to date".to_string(),
            description: "Tick a folder to download its messages.".to_string(),
            ticked,
        };

        let (status, state) = list.get_state(1);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED);
        assert_eq!(state, 0);
    }

    #[test]
    fn test_every_row_answers_as_focusable_and_selectable() {
        // An OK answer replaces the platform's own flags rather than adding to
        // them, so anything left out here is a fact about the row that nothing
        // else supplies.
        let list = checked_list(vec![true]);

        let (_status, state) = list.get_state(1);

        assert!(state & CHECKED != 0, "the tick is missing");
        assert!(state & FOCUSABLE != 0, "the row cannot be focused");
        assert!(state & SELECTABLE != 0, "the row cannot be selected");
    }

    #[test]
    fn test_the_returned_handle_is_the_snapshot_the_rows_read() {
        // What the folder chooser relies on when somebody presses Space: it
        // writes the new ticks through this handle and expects the rows to
        // answer with them. A handle wired to a different snapshot would leave
        // every row reporting whatever was true when the window opened.
        let (rows, handle) = super::checked_rows(
            "Folders to keep up to date",
            "Tick a folder to download its messages.",
            vec![true],
        );

        *handle.lock().expect("a fresh mutex is not poisoned") = vec![false];

        let (_status, state) = rows.get_state(1);
        assert!(
            state & CHECKED == 0,
            "unticking a row must change what it answers"
        );
    }

    #[test]
    fn test_the_name_is_the_only_thing_replaced() {
        let named = FixedName {
            name: "Messages".to_string(),
            description: None,
        };
        let (status, name) = named.get_name(0);
        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_OK);
        assert_eq!(name.as_deref(), Some("Messages"));
    }

    #[test]
    fn test_children_name_themselves() {
        // A list row, a tree node and a notebook tab each ask for their own
        // name through the parent. Answering with the control's name labels
        // every one of them identically.
        let named = FixedName {
            name: "Settings categories".to_string(),
            description: None,
        };
        let (status, name) = named.get_name(1);
        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED);
        assert!(name.is_none());
    }

    #[test]
    fn test_a_description_is_offered_for_the_control_itself() {
        // The bug this was written for: the first-run screen's three choices
        // each had a sentence beside them saying what it costs, and a screen
        // reader never read any of them. The label was named, the explanation
        // was a separate piece of text nobody was pointed at, and somebody
        // choosing what Wixen Mail may change heard only "read my mail,
        // change nothing" with no idea what the other two did.
        let described = FixedName {
            name: "Read my mail, change nothing".to_string(),
            description: Some("Nothing you do here reaches your provider.".to_string()),
        };

        let (status, description) = described.get_description(0);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_OK);
        assert_eq!(
            description.as_deref(),
            Some("Nothing you do here reaches your provider.")
        );
    }

    #[test]
    fn test_a_control_with_no_description_leaves_the_question_to_the_platform() {
        // NOT_IMPLEMENTED rather than an empty string. Answering with nothing
        // is a claim that there is no description, which stops wxWidgets
        // supplying whatever the control would have said for itself.
        let named = FixedName {
            name: "Messages".to_string(),
            description: None,
        };

        let (status, description) = named.get_description(0);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED);
        assert!(description.is_none());
    }

    #[test]
    fn test_children_describe_themselves() {
        // Same trap as the name. Handing every row of a list the control's
        // description would have each one read it out.
        let described = FixedName {
            name: "Messages".to_string(),
            description: Some("Your inbox".to_string()),
        };

        let (status, description) = described.get_description(1);

        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED);
        assert!(description.is_none());
    }

    #[test]
    fn test_child_enumeration_is_left_to_the_control() {
        // The trait default answers OK with zero children, which is a claim
        // that the control is empty rather than a request to fall back. With
        // that default in place a named notebook lost its tabs and a named
        // list would have lost every row.
        let named = FixedName {
            name: "Messages".to_string(),
            description: None,
        };
        let (status, count) = named.get_child_count();
        assert_eq!(
            status,
            ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED,
            "naming a control must not claim it has no children"
        );
        assert_eq!(count, 0);
    }

    #[test]
    fn test_strips_the_mnemonic_and_colon() {
        assert_eq!(name_from_label("&Subject:"), "Subject");
        assert_eq!(
            name_from_label("Start &Date (YYYY-MM-DD):"),
            "Start Date (YYYY-MM-DD)"
        );
    }

    #[test]
    fn test_leaves_a_plain_label_alone() {
        assert_eq!(name_from_label("Accounts"), "Accounts");
    }

    #[test]
    fn test_handles_a_label_that_is_only_decoration() {
        assert_eq!(name_from_label(":"), "");
        assert_eq!(name_from_label("   "), "");
    }
}

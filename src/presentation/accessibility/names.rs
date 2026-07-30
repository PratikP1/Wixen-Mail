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
    let state: TickState = std::sync::Arc::new(std::sync::Mutex::new(ticked));
    window.set_accessible(Accessible::new(
        window,
        CheckedRows {
            name: name.to_string(),
            description: description.to_string(),
            ticked: state.clone(),
        },
    ));
    state
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
    use super::{AccessibleImpl, FixedName, name_from_label};
    use wxdragon::ffi;

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

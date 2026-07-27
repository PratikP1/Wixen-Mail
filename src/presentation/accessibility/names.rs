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

/// Supplies one fixed name for a control, leaving every other accessibility
/// property to the platform's default handling.
struct FixedName {
    name: String,
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
    /// is what makes wxWidgets use its own implementation. Only the name is
    /// meant to be replaced here.
    fn get_child_count(&self) -> (AccStatus, i32) {
        (ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED, 0)
    }
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
        };
        let (status, name) = named.get_name(1);
        assert_eq!(status, ffi::wxd_AccStatus_WXD_ACC_NOT_IMPLEMENTED);
        assert!(name.is_none());
    }

    #[test]
    fn test_child_enumeration_is_left_to_the_control() {
        // The trait default answers OK with zero children, which is a claim
        // that the control is empty rather than a request to fall back. With
        // that default in place a named notebook lost its tabs and a named
        // list would have lost every row.
        let named = FixedName {
            name: "Messages".to_string(),
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

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
    fn get_name(&self, _child_id: i32) -> (AccStatus, Option<String>) {
        (ffi::wxd_AccStatus_WXD_ACC_OK, Some(self.name.clone()))
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

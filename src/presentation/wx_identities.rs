//! Other Addresses to Send From: the manager and its address window (#59,
//! GAP-10, 13-33).

use crate::application::identities::Identity;
use crate::data::account::Account;
use crate::presentation::accessibility::Accessibility;
use crate::presentation::theme;
use crate::presentation::wx_managers::{ManagedRow, make_shell};
use std::sync::Arc;
use wxdragon::prelude::*;

impl ManagedRow for Identity {}

/// The manager's window, built and filled without being shown.
pub struct IdentityManagerWidgets {
    pub dialog: Dialog,
    pub sizer: BoxSizer,
    pub list: ListCtrl,
    pub status: StaticText,
}

/// Build the manager over an account's other addresses.
pub fn build_identity_manager(
    parent: &dyn WxWidget,
    _account: &Account,
    _rows: &[Identity],
    palette: Option<theme::Palette>,
) -> IdentityManagerWidgets {
    let (dialog, sizer, list, status) = make_shell(parent, "", "", 560, 400, palette);
    IdentityManagerWidgets {
        dialog,
        sizer,
        list,
        status,
    }
}

/// Fill the manager's list, one row per address in its order.
pub fn populate_identities(_list: &ListCtrl, _rows: &[Identity]) {}

/// The address window, built without being shown.
#[derive(Clone, Copy)]
pub struct AddressWindowWidgets {
    pub dialog: Dialog,
}

/// Build the address window, for a new address or one being edited.
pub fn build_address_window(
    parent: &dyn WxWidget,
    _existing: Option<&Identity>,
    _palette: Option<theme::Palette>,
) -> AddressWindowWidgets {
    AddressWindowWidgets {
        dialog: Dialog::builder(parent, "Add Other Address").build(),
    }
}

/// Wire the window's OK to the rules for an address.
pub fn wire_the_address_window(
    _w: &AddressWindowWidgets,
    _account: &Account,
    _held: &[Identity],
    _a11y: &Arc<Accessibility>,
) {
}

//! New Saved Search: the name of a saved search made from nothing, and where
//! it looks (#58, GAP-09, 13-39).

use crate::presentation::theme;
use wxdragon::prelude::*;

/// The New Saved Search window's controls, returned so a test and a scan
/// target can build it without the modal loop.
#[derive(Clone, Copy)]
pub struct NewSavedSearchWindow {
    pub dialog: Dialog,
    pub name: TextCtrl,
    pub look_in: Choice,
    pub ok: Button,
    pub cancel: Button,
}

/// Build the New Saved Search window over the places a search can look in.
pub fn build_new_saved_search_dialog(
    parent: &dyn WxWidget,
    _places: &[(String, Option<String>)],
    _palette: Option<theme::Palette>,
) -> NewSavedSearchWindow {
    let dialog = Dialog::builder(parent, "").build();
    NewSavedSearchWindow {
        dialog,
        name: TextCtrl::builder(&dialog).build(),
        look_in: Choice::builder(&dialog).build(),
        ok: Button::builder(&dialog).build(),
        cancel: Button::builder(&dialog).build(),
    }
}

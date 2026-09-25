//! Undo and Redo on a text box, as the Edit menu asks for them.
//!
//! Every box the Edit menu reaches keeps a history of several steps
//! (`presentation::text_history_keys`), so Undo and Redo, and the menu's
//! greying of them, ask that history. What each command means where the
//! cursor is, and what it says when it cannot act, is
//! `application::editing`'s; this only reaches the box.
//!
//! Until 13-05 this sent the box `EM_CANUNDO` and `EM_UNDO`, its own one
//! step, and Redo was offered only straight after an Undo, because on a box of
//! one step Redo is Undo again. With a history of its own in every box the
//! menu reaches, that path had no caller left and went.
//!
//! # What one step means for Cut
//!
//! Cut used to take the chosen words out with `replace`, which a box records
//! as a removal and then an empty write, and remembers only the second: the
//! box's own undo after it put nothing back. `remove_the_selection` writes
//! nothing over the chosen words instead, one change. Both measured on a real
//! box in `tests/undo_reaches_the_text.rs`.

use crate::presentation::text_history_keys::{can_redo_in, can_undo_in, redo_in, undo_in};
use wxdragon::prelude::*;

/// What Undo did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Undone {
    Done,
    NothingToUndo,
}

/// What Redo did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Redone {
    Done,
    NothingToRedo,
}

/// Which of the two the Edit menu offers while it is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Offer {
    pub undo: bool,
    pub redo: bool,
}

/// Take the box's last step back.
pub fn undo(box_: &TextCtrl) -> Undone {
    match undo_in(box_) {
        true => Undone::Done,
        false => Undone::NothingToUndo,
    }
}

/// Put back the box's last step undone.
pub fn redo(box_: &TextCtrl) -> Redone {
    match redo_in(box_) {
        true => Redone::Done,
        false => Redone::NothingToRedo,
    }
}

/// Take the chosen words out of the box as one change.
pub fn remove_the_selection(box_: &TextCtrl) {
    box_.write_text("");
}

/// What the Edit menu offers for the box that has focus, if one has. A box
/// that can only be read offers neither, as `application::editing` refuses
/// both there.
pub fn what_the_edit_menu_offers(focused: Option<&TextCtrl>) -> Offer {
    match focused {
        Some(box_) if box_.is_editable() => Offer {
            undo: can_undo_in(box_),
            redo: can_redo_in(box_),
        },
        _ => Offer::default(),
    }
}

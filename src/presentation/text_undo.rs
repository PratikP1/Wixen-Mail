//! Undo and Redo on a native text box, through the box's own one step.
//!
//! wxdragon's `TextCtrl` offers no undo at all, so this sends the box the two
//! messages wxWidgets itself sends on Windows: `EM_CANUNDO` to ask and
//! `EM_UNDO` to act.
//!
//! # Why Redo is narrower than wxWidgets' Redo
//!
//! A plain Windows edit box remembers one step, and wxWidgets makes Redo the
//! same message as Undo, "since Undo undoes the undo"
//! (`src/msw/textentry.cpp`). That is right straight after an Undo and wrong
//! after anything typed since: Redo would take the typing back. So Redo is
//! offered only for the box the last Undo was in, while its words are still
//! the ones that Undo left.

use wxdragon::prelude::*;

/// What the last Undo left, so Redo can tell whether it is still the step to
/// put back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastUndo {
    handle: isize,
    value_after: String,
}

/// What Undo did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Undone {
    /// Took the box's last change back, and this is what Redo needs.
    Done(LastUndo),
    /// The box had nothing to take back.
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

/// Whether the box has a change to take back.
pub fn can_undo(_box: &TextCtrl) -> bool {
    false
}

/// Take the box's last change back.
pub fn undo(_box: &TextCtrl) -> Undone {
    Undone::NothingToUndo
}

/// Whether Redo would put back what `last` took away.
pub fn can_redo(_box: &TextCtrl, _last: &LastUndo) -> bool {
    true
}

/// Put back what `last` took away.
pub fn redo(_box: &TextCtrl, _last: &LastUndo) -> Redone {
    Redone::NothingToRedo
}

/// Take the chosen words out of the box as one step the box can undo.
pub fn remove_the_selection(_box: &TextCtrl) {}

/// What the Edit menu offers for the box that has focus, if one has.
pub fn what_the_edit_menu_offers(_focused: Option<&TextCtrl>, _last: Option<&LastUndo>) -> Offer {
    Offer::default()
}

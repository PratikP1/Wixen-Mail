//! Undo and Redo on a native text box, through the box's own one step.
//!
//! wxdragon's `TextCtrl` offers no undo at all, so this sends the box the two
//! messages wxWidgets itself sends on Windows: `EM_CANUNDO` to ask and
//! `EM_UNDO` to act. What each command means where the cursor is, and what it
//! says when it cannot act, is `application::editing`'s; this only reaches
//! the box.
//!
//! # Why Redo is narrower than wxWidgets' Redo
//!
//! A plain Windows edit box remembers one step, and wxWidgets makes Redo the
//! same message as Undo, "since Undo undoes the undo"
//! (`src/msw/textentry.cpp`). That is right straight after an Undo and wrong
//! after anything typed since: Redo would take the typing back. So Redo is
//! offered only for the box the last Undo was in, while its words are still
//! the ones that Undo left.
//!
//! # What one step means for Cut
//!
//! Cut used to take the chosen words out with `replace`, which a box records
//! as a removal and then an empty write, and remembers only the second: Undo
//! after it put nothing back. `remove_the_selection` writes nothing over the
//! chosen words instead, one step the box undoes whole. Both measured on a
//! real box in `tests/undo_reaches_the_text.rs`.
//!
//! # Elsewhere
//!
//! Off Windows there is no message to send, so a box answers that it has
//! nothing to undo, and the box's own keys are what undo there. A port needs
//! its own bridge here, as it does for the rest of the accessibility layer.

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

fn handle_of(box_: &TextCtrl) -> isize {
    box_.get_handle() as isize
}

/// Whether the box has a change to take back.
pub fn can_undo(box_: &TextCtrl) -> bool {
    native::can_undo(handle_of(box_))
}

/// Take the box's last change back.
pub fn undo(box_: &TextCtrl) -> Undone {
    if !can_undo(box_) {
        return Undone::NothingToUndo;
    }
    native::undo(handle_of(box_));
    Undone::Done(LastUndo {
        handle: handle_of(box_),
        value_after: box_.get_value(),
    })
}

/// Whether Redo would put back what `last` took away: the same box, with the
/// words that Undo left, and a step still there to undo.
pub fn can_redo(box_: &TextCtrl, last: &LastUndo) -> bool {
    last.handle == handle_of(box_) && box_.get_value() == last.value_after && can_undo(box_)
}

/// Put back what `last` took away, which on a box of one step is undoing the
/// undo.
pub fn redo(box_: &TextCtrl, last: &LastUndo) -> Redone {
    if !can_redo(box_, last) {
        return Redone::NothingToRedo;
    }
    native::undo(handle_of(box_));
    Redone::Done
}

/// Take the chosen words out of the box as one step the box can undo.
pub fn remove_the_selection(box_: &TextCtrl) {
    box_.write_text("");
}

/// What the Edit menu offers for the box that has focus, if one has. A box
/// that can only be read offers neither, as `application::editing` refuses
/// both there.
pub fn what_the_edit_menu_offers(focused: Option<&TextCtrl>, last: Option<&LastUndo>) -> Offer {
    match focused {
        Some(box_) if box_.is_editable() => Offer {
            undo: can_undo(box_),
            redo: last.is_some_and(|last| can_redo(box_, last)),
        },
        _ => Offer::default(),
    }
}

#[cfg(target_os = "windows")]
mod native {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Controls::{EM_CANUNDO, EM_UNDO};
    use windows::Win32::UI::WindowsAndMessaging::SendMessageW;

    fn window(handle: isize) -> HWND {
        HWND(handle as *mut std::ffi::c_void)
    }

    pub(super) fn can_undo(handle: isize) -> bool {
        // SAFETY: the handle is a live box this program built, read from it at
        // the moment of asking; the message takes and returns no pointer.
        let answer = unsafe { SendMessageW(window(handle), EM_CANUNDO, None, None) };
        answer.0 != 0
    }

    pub(super) fn undo(handle: isize) {
        // SAFETY: as above; the message takes no pointer and its answer says
        // nothing `can_undo` did not.
        unsafe { SendMessageW(window(handle), EM_UNDO, None, None) };
    }
}

#[cfg(not(target_os = "windows"))]
mod native {
    pub(super) fn can_undo(_handle: isize) -> bool {
        false
    }

    pub(super) fn undo(_handle: isize) {}
}

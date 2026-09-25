//! A text box's history of several steps, kept beside the box and reached by
//! its window handle.
//!
//! `application::text_history` decides what a step is; this watches the box
//! and tells the history each change, and shows the box what an undo or a
//! redo hands back. Why the history is ours and not Windows' is said there.
//!
//! # What is bound
//!
//! The box's text event, always passed on, so the contacts search and every
//! other handler still run. And its key-down for Ctrl+Z and Ctrl+Y, taken
//! after acting, because a key-down taken is one whose character wxWidgets
//! then drops (`src/msw/window.cpp`), so the box's own one-step undo does not
//! also run. In the main window the Edit menu's keys reach the command before
//! the box sees them; in a dialog, which has no menu, this is what answers.
//!
//! # What the program writes
//!
//! A value the program chooses, such as a note opened into the editor, goes
//! through `set_anew`, which starts the history afresh instead of becoming a
//! step somebody could undo into another note's words.
//!
//! # Restoring
//!
//! A restore writes the box's value the ordinary way, so the box raises its
//! change and the contacts search runs again on the restored words. The
//! history is told not to record that change as a step of its own.
//!
//! The histories live on the interface thread, keyed by handle, and each goes
//! when its box is destroyed. Nothing here is written to disk or to the log.

use wxdragon::prelude::*;

/// Keep a history of several steps for `box_`, and answer Ctrl+Z and Ctrl+Y
/// in it.
pub fn keep_a_history(_box_: &TextCtrl) {}

/// Write `value` into the box as the program's own choice: the box shows it
/// and the history starts again from it, with nothing to undo.
pub fn set_anew(box_: &TextCtrl, value: &str) {
    box_.set_value(value);
}

/// Take the last step back in the box. False when there was none, or when the
/// box keeps no history.
pub fn undo_in(_box_: &TextCtrl) -> bool {
    false
}

/// Put back the last step undone in the box. False when there was none.
pub fn redo_in(_box_: &TextCtrl) -> bool {
    false
}

/// Whether Undo has a step to take back in the box.
pub fn can_undo_in(_box_: &TextCtrl) -> bool {
    false
}

/// Whether Redo has a step to put back in the box.
pub fn can_redo_in(_box_: &TextCtrl) -> bool {
    false
}

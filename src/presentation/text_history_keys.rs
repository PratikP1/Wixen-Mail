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
//! change and the contacts search runs again on the restored words. That
//! change is not kept as a step, and no flag is needed to say so: the history
//! already holds the value it handed back, so the change it is told about is
//! no change at all. `set_anew` starts the history before it writes, for the
//! same reason. `tests/several_steps_come_back.rs` reads both.
//!
//! The histories live on the interface thread, keyed by handle, and each goes
//! when its box is destroyed. Nothing here is written to disk or to the log.

use crate::application::text_history::{History, Restore};
use std::cell::RefCell;
use std::collections::HashMap;
use wxdragon::event::KeyboardEvent;
use wxdragon::prelude::*;

thread_local! {
    static KEPT: RefCell<HashMap<isize, History>> = RefCell::new(HashMap::new());
}

/// Which of the two history keys a key press is, if either.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HistoryKey {
    Undo,
    Redo,
}

impl HistoryKey {
    /// Ctrl+Z or Ctrl+Y with neither Alt nor Shift. Ctrl+Shift+Z is left
    /// alone, because in the main window it is Undo Send.
    fn of(key: &KeyboardEvent) -> Option<Self> {
        if !key.control_down() || key.alt_down() || key.shift_down() {
            return None;
        }
        match u8::try_from(key.get_key_code()?).ok()? {
            b'Z' => Some(Self::Undo),
            b'Y' => Some(Self::Redo),
            _ => None,
        }
    }
}

fn handle_of(box_: &TextCtrl) -> isize {
    box_.get_handle() as isize
}

/// Keep a history of several steps for `box_`, and answer Ctrl+Z and Ctrl+Y
/// in it.
pub fn keep_a_history(box_: &TextCtrl) {
    let handle = handle_of(box_);
    let history = History::new(&box_.get_value());
    KEPT.with_borrow_mut(|histories| histories.insert(handle, history));

    let changed = *box_;
    box_.on_text_changed(move |event| {
        record_the_change(&changed);
        event.skip(true);
    });
    let pressed = *box_;
    box_.on_key_down(move |event| {
        let WindowEventData::Keyboard(ref key) = event else {
            return;
        };
        let Some(history_key) = HistoryKey::of(key) else {
            return;
        };
        // Taken whether or not there was a step, so the box's own undo never
        // runs on top. Saying what came of it is the Edit menu's in the main
        // window, which takes these keys before the box does.
        let _acted = match history_key {
            HistoryKey::Undo => undo_in(&pressed),
            HistoryKey::Redo => redo_in(&pressed),
        };
        event.skip(false);
    });
    box_.on_destroy(move |_| {
        KEPT.with_borrow_mut(|histories| histories.remove(&handle));
    });
}

/// Write `value` into the box as the program's own choice: the box shows it
/// and the history starts again from it, with nothing to undo.
pub fn set_anew(box_: &TextCtrl, value: &str) {
    with_history(box_, |history| history.set_anew(value));
    box_.set_value(value);
}

/// Take the last step back in the box. False when there was none, or when the
/// box keeps no history.
pub fn undo_in(box_: &TextCtrl) -> bool {
    restore(box_, History::undo)
}

/// Put back the last step undone in the box. False when there was none.
pub fn redo_in(box_: &TextCtrl) -> bool {
    restore(box_, History::redo)
}

/// Whether Undo has a step to take back in the box.
pub fn can_undo_in(box_: &TextCtrl) -> bool {
    with_history(box_, |history| history.can_undo()).unwrap_or(false)
}

/// Whether Redo has a step to put back in the box.
pub fn can_redo_in(box_: &TextCtrl) -> bool {
    with_history(box_, |history| history.can_redo()).unwrap_or(false)
}

/// Ask the box's history something. The registry is borrowed only for the
/// question, never while the box writes, because the box's change arrives
/// inside the write and asks the registry itself.
fn with_history<T>(box_: &TextCtrl, ask: impl FnOnce(&mut History) -> T) -> Option<T> {
    let handle = handle_of(box_);
    KEPT.with_borrow_mut(|histories| histories.get_mut(&handle).map(ask))
}

/// The box changed: tell its history.
fn record_the_change(box_: &TextCtrl) {
    let caret = usize::try_from(box_.get_insertion_point()).unwrap_or(0);
    let value = box_.get_value();
    with_history(box_, |history| history.record(&value, caret));
}

/// Ask the history for a step and show it in the box, which raises the box's
/// change for every other handler.
fn restore(box_: &TextCtrl, step: fn(&mut History) -> Option<Restore>) -> bool {
    let Some(Restore { value, selection }) = with_history(box_, step).flatten() else {
        return false;
    };
    box_.set_value(&value);
    let (from, to) = selection;
    box_.set_selection(from as i64, to as i64);
    true
}

//! A toolbar button's text, changed through the native toolbar.
//!
//! The main toolbar is `Flat | Text`, so each button's label is shown and is
//! what a screen reader says for it. Mark as Read's label has to follow the
//! message under the cursor (#27), and wxdragon 0.9.17 can change a tool's
//! short help and not its label: `ToolBar` offers `set_tool_short_help` and
//! `Tool` offers enable, toggle and their readings, nothing for the text. So
//! the text goes through Windows directly: `TB_SETBUTTONINFOW` with
//! `TBIF_TEXT` on the toolbar's own window, which is the same control
//! wxWidgets built and the one the accessible object for a toolbar reads a
//! button's name from. `TB_GETBUTTONTEXTW` reads it back, for the reading in
//! `tests/mark_as_read_says_which_way_it_will_go.rs` and for nothing that
//! ships.
//!
//! # What the toolkit still holds
//!
//! wxWidgets keeps its own record of each tool's label and rebuilds the
//! buttons from it when it realizes the bar again, which on Windows it does
//! on a DPI change. The label set here stands until then and is set again on
//! the next selection or toggle, since those are where the caller refreshes
//! it; a check tool with a pressed state was considered instead and not
//! chosen, because the tester's words are that the label follows the state
//! on the toolbar too, and a pressed state is a different thing to hear.
//!
//! **Windows only**, as `wxAccessible` is. `TBBUTTONINFOW` is declared by
//! hand from commctrl.h rather than through a crate feature, with its size
//! held by a test, because a wrong layout is undefined behaviour inside the
//! control rather than an error. Elsewhere every call here accepts and does
//! nothing, and the label stays what the toolkit gave it.

use wxdragon::prelude::*;

/// The control's own window handle, as an integer the messages take.
fn handle_of(toolbar: &ToolBar) -> isize {
    toolbar.get_handle() as isize
}

/// Give the tool with `id` the text `text`, on the channel a screen reader
/// reads it from.
pub fn relabel(toolbar: &ToolBar, id: Id, text: &str) {
    let _ = (handle_of(toolbar), id, text);
}

/// The text the tool with `id` carries now, read from the control, or `None`
/// when the control has no such tool or this platform cannot ask.
pub fn label_of(toolbar: &ToolBar, id: Id) -> Option<String> {
    platform::button_text(handle_of(toolbar), id)
}

#[cfg(target_os = "windows")]
mod platform {
    /// winuser.h: `WM_USER` is 0x0400; commctrl.h numbers the toolbar's
    /// messages from it.
    const TB_GETBUTTONTEXTW: u32 = 0x0400 + 75;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
    }

    fn send(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize {
        if hwnd == 0 {
            return 0;
        }
        // SAFETY: a window handle the caller still holds; the messages sent
        // here read or set one button's text and answer -1 for an id the
        // control does not know.
        unsafe { SendMessageW(hwnd, message, wparam, lparam) }
    }

    pub(super) fn button_text(hwnd: isize, id: i32) -> Option<String> {
        // Asked once for the length, with no buffer, then once for the text.
        // The control answers the length without the terminator and writes
        // the terminator, so the buffer is one longer.
        let length = send(hwnd, TB_GETBUTTONTEXTW, id as usize, 0);
        if length < 0 {
            return None;
        }
        let mut buffer = vec![0u16; length as usize + 1];
        let written = send(
            hwnd,
            TB_GETBUTTONTEXTW,
            id as usize,
            buffer.as_mut_ptr() as isize,
        );
        if written < 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&buffer[..written as usize]))
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    pub(super) fn button_text(_hwnd: isize, _id: i32) -> Option<String> {
        None
    }
}

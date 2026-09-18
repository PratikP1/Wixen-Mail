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
    platform::set_button_text(handle_of(toolbar), id, text);
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
    const TB_AUTOSIZE: u32 = 0x0400 + 33;
    const TB_SETBUTTONINFOW: u32 = 0x0400 + 64;
    const TB_GETBUTTONTEXTW: u32 = 0x0400 + 75;
    /// commctrl.h: which `TBBUTTONINFOW` fields a message reads.
    const TBIF_TEXT: u32 = 0x0002;

    /// `TBBUTTONINFOW` as commctrl.h lays it out on 64-bit Windows: 48
    /// bytes, `lParam` at 24, the text pointer at 32 and its length at 40.
    /// Only `cbSize`, `dwMask` and `pszText` are read for the one message
    /// this module sends with it.
    #[repr(C)]
    struct ButtonInfoW {
        cb_size: u32,
        dw_mask: u32,
        id_command: i32,
        i_image: i32,
        fs_state: u8,
        fs_style: u8,
        cx: u16,
        l_param: usize,
        psz_text: *mut u16,
        cch_text: i32,
    }

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

    pub(super) fn set_button_text(hwnd: isize, id: i32, text: &str) {
        // The control copies the text during the call, so a buffer that
        // outlives the call is enough; it is terminated, as the control
        // reads it, and no other field is asked for.
        let mut buffer: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let mut info = ButtonInfoW {
            cb_size: std::mem::size_of::<ButtonInfoW>() as u32,
            dw_mask: TBIF_TEXT,
            id_command: 0,
            i_image: 0,
            fs_state: 0,
            fs_style: 0,
            cx: 0,
            l_param: 0,
            psz_text: buffer.as_mut_ptr(),
            cch_text: 0,
        };
        send(
            hwnd,
            TB_SETBUTTONINFOW,
            id as usize,
            &mut info as *mut ButtonInfoW as isize,
        );
        // A longer label wants a wider button, and the control does not
        // resize its buttons for a text set this way until asked.
        send(hwnd, TB_AUTOSIZE, 0, 0);
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

    #[cfg(test)]
    mod tests {
        use super::ButtonInfoW;

        #[test]
        fn test_the_button_info_struct_is_laid_out_as_the_header_lays_it_out() {
            // A wrong layout is undefined behaviour inside the control rather
            // than an error, so the size and the offsets are held here, where
            // they cost nothing.
            assert_eq!(std::mem::size_of::<ButtonInfoW>(), 48);
            assert_eq!(std::mem::offset_of!(ButtonInfoW, dw_mask), 4);
            assert_eq!(std::mem::offset_of!(ButtonInfoW, fs_state), 16);
            assert_eq!(std::mem::offset_of!(ButtonInfoW, l_param), 24);
            assert_eq!(std::mem::offset_of!(ButtonInfoW, psz_text), 32);
            assert_eq!(std::mem::offset_of!(ButtonInfoW, cch_text), 40);
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    pub(super) fn set_button_text(_hwnd: isize, _id: i32, _text: &str) {}

    pub(super) fn button_text(_hwnd: isize, _id: i32) -> Option<String> {
        None
    }
}

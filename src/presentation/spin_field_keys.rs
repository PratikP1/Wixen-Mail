//! The arrow keys in a spin control's typing field, taken before the
//! control's own arrows see them.
//!
//! A spin control is two windows: the arrows, and the field a person types
//! in, where the keyboard is. The arrows take Up and Down in that field and
//! step by one before wxWidgets forwards the key to anything bound on the
//! spin control, so a handler bound there sees Right and never sees Up.
//! Measured 2026-09-24 by `tests/event_times_move_in_blocks.rs`,
//! `test_right_reaches_a_handler_on_the_spin_control_and_up_does_not`. So
//! [`take_the_arrows`] subclasses the field itself, on top of the arrows' own
//! subclass, and a key its handler takes goes no further.
//!
//! Only a bare arrow is offered. With Shift, Ctrl or Alt held the key is the
//! field's own: Shift with Left selects, and Ctrl with Left moves by a word.

/// An arrow key pressed with nothing held.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arrow {
    Up,
    Down,
    Left,
    Right,
}

/// winuser.h: the four arrow keys' virtual key codes.
const VK_LEFT: usize = 0x25;
const VK_UP: usize = 0x26;
const VK_RIGHT: usize = 0x27;
const VK_DOWN: usize = 0x28;

/// The arrow a key-down names, or `None` for any other key and for an arrow
/// pressed with a modifier held.
pub fn the_arrow(virtual_key: usize, is_a_modifier_held: bool) -> Option<Arrow> {
    if is_a_modifier_held {
        return None;
    }
    match virtual_key {
        VK_UP => Some(Arrow::Up),
        VK_DOWN => Some(Arrow::Down),
        VK_LEFT => Some(Arrow::Left),
        VK_RIGHT => Some(Arrow::Right),
        _ => None,
    }
}

/// What a handler answers about an arrow: whether it took the key.
pub type OnArrow = dyn Fn(Arrow) -> bool;

/// Hand the bare arrow keys pressed in `spin`'s typing field to `on_arrow`
/// before the control's own arrows see them. A key `on_arrow` takes, by
/// answering `true`, goes no further, so the native step of one does not
/// also run; one it declines reaches the control as before.
///
/// Held until the field is destroyed, and let go then.
pub fn take_the_arrows(spin: &wxdragon::prelude::SpinCtrl, on_arrow: Box<OnArrow>) {
    the_field::take(spin, on_arrow);
}

#[cfg(target_os = "windows")]
mod the_field {
    use super::{OnArrow, the_arrow};
    use wxdragon::prelude::{SpinCtrl, WxWidget};

    const WM_NCDESTROY: u32 = 0x0082;
    const WM_KEYDOWN: u32 = 0x0100;
    /// commctrl.h: `WM_USER + 106`, the field an up-down control is attached to.
    const UDM_GETBUDDY: u32 = 0x0400 + 106;
    const VK_SHIFT: i32 = 0x10;
    const VK_CONTROL: i32 = 0x11;
    const VK_MENU: i32 = 0x12;
    /// The one subclass this module installs on a field.
    const THIS_SUBCLASS: usize = 0x5B1D;

    type SubclassProc = unsafe extern "system" fn(isize, u32, usize, isize, usize, usize) -> isize;

    // Declared by hand, as `toolbar_text` and `native_tree_checks` do, rather
    // than turning on the `windows` crate's whole Shell feature for three
    // functions. Its compile cost was not measured; the choice is only that.
    #[link(name = "comctl32")]
    unsafe extern "system" {
        fn SetWindowSubclass(hwnd: isize, proc: SubclassProc, id: usize, data: usize) -> i32;
        fn RemoveWindowSubclass(hwnd: isize, proc: SubclassProc, id: usize) -> i32;
        fn DefSubclassProc(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
        fn GetKeyState(key: i32) -> i16;
    }

    fn is_held(key: i32) -> bool {
        // SAFETY: reads this thread's keyboard state; no pointers.
        unsafe { GetKeyState(key) < 0 }
    }

    pub(super) fn take(spin: &SpinCtrl, on_arrow: Box<OnArrow>) {
        let arrows = spin.get_handle() as isize;
        // SAFETY: the arrows are a live window this program built; the
        // message takes and returns no pointer.
        let field = unsafe { SendMessageW(arrows, UDM_GETBUDDY, 0, 0) };
        if field == 0 {
            tracing::warn!("spin-field-keys: no field beside arrows={arrows}");
            return;
        }
        let handler = Box::into_raw(Box::new(on_arrow)) as usize;
        // SAFETY: a live window on this thread; the handler is freed by the
        // subclass itself at WM_NCDESTROY, or here if it was never installed.
        let installed = unsafe { SetWindowSubclass(field, arrows_first, THIS_SUBCLASS, handler) };
        if installed == 0 {
            tracing::warn!("spin-field-keys: the field={field} could not be subclassed");
            // SAFETY: made by `Box::into_raw` above and handed to nothing.
            drop(unsafe { Box::from_raw(handler as *mut Box<OnArrow>) });
        }
    }

    unsafe extern "system" fn arrows_first(
        hwnd: isize,
        message: u32,
        wparam: usize,
        lparam: isize,
        id: usize,
        handler: usize,
    ) -> isize {
        match message {
            WM_KEYDOWN => {
                let is_a_modifier_held = [VK_SHIFT, VK_CONTROL, VK_MENU].into_iter().any(is_held);
                // SAFETY: `handler` is the box `take` made, alive until
                // WM_NCDESTROY below.
                let on_arrow = unsafe { &*(handler as *const Box<OnArrow>) };
                if the_arrow(wparam, is_a_modifier_held).is_some_and(on_arrow) {
                    return 0;
                }
            }
            WM_NCDESTROY => {
                // SAFETY: the window is going; the box is dropped once, here.
                unsafe {
                    RemoveWindowSubclass(hwnd, arrows_first, id);
                    drop(Box::from_raw(handler as *mut Box<OnArrow>));
                }
            }
            _ => {}
        }
        // SAFETY: passes the message on down the chain it arrived through.
        unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
    }
}

/// Elsewhere the spin control's own key-down is the only place a key can be
/// taken, and nothing here has measured whether the arrows reach it there.
#[cfg(not(target_os = "windows"))]
mod the_field {
    use super::OnArrow;
    use wxdragon::prelude::SpinCtrl;

    pub(super) fn take(_spin: &SpinCtrl, _on_arrow: Box<OnArrow>) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_each_arrow_key_names_its_arrow() {
        assert_eq!(the_arrow(0x26, false), Some(Arrow::Up));
        assert_eq!(the_arrow(0x28, false), Some(Arrow::Down));
        assert_eq!(the_arrow(0x25, false), Some(Arrow::Left));
        assert_eq!(the_arrow(0x27, false), Some(Arrow::Right));
    }

    #[test]
    fn test_an_arrow_with_a_modifier_held_is_the_fields_own() {
        for key in [0x25, 0x26, 0x27, 0x28] {
            assert_eq!(the_arrow(key, true), None, "{key:#x}");
        }
    }

    #[test]
    fn test_companion_every_other_key_is_the_fields_own() {
        // Digits typed over the value, Home, End and Page Up among them.
        for key in [0x30, 0x39, 0x24, 0x23, 0x21, 0x22, 0x0D] {
            assert_eq!(the_arrow(key, false), None, "{key:#x}");
        }
    }
}

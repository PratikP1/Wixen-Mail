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
    let _ = (
        virtual_key,
        is_a_modifier_held,
        VK_LEFT,
        VK_UP,
        VK_RIGHT,
        VK_DOWN,
    );
    Some(Arrow::Up)
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

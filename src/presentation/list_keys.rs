//! A plain letter bound on a list, and kept from the list's own search.
//!
//! A `SysListView32` in report view takes any letter typed on it as the start
//! of a search: it jumps to the next row whose first column begins with that
//! letter, on the `WM_CHAR` that follows the key-down. So a letter given a
//! meaning on the message list, M for Mark as Read (#27), has to be consumed
//! at the key-down, or the search moves the cursor off the message the
//! command just acted on and the screen reader reads the wrong row.
//!
//! # Why `skip(false)` is the whole mechanism
//!
//! wxWidgets does not hand the `WM_CHAR` to the control when the `KEY_DOWN`
//! event was handled and not skipped: `wxWindowMSW::MSWWindowProc` records
//! `m_lastKeydownProcessed` from the key-down and eats the char that follows
//! (`src/msw/window.cpp`, the `WM_KEYDOWN` and `WM_CHAR` cases). Under
//! wxdragon "not skipped" is not the same as "did not call skip": the
//! dispatcher resets `Skip(true)` before every closure it calls
//! (`wxdragon-sys-0.9.17/cpp/src/event.cpp:379`) and treats the event as
//! consumed only when the closure cleared it. A closure that says nothing
//! therefore leaves the event skipped, the char reaches the control, and the
//! search gets the letter. The handler here calls `event.skip(false)` for a
//! bare press of its letter, which is how `wx_item_form.rs` already refuses a
//! key, and leaves every other key as the dispatcher set it, so Space and the
//! arrows stay the control's. Two `KEY_DOWN` closures on one list, Space's
//! and this, are fine: the dispatcher runs them in binding order and stops
//! at the first that cleared skip.
//!
//! `tests/mark_as_read_says_which_way_it_will_go.rs` sends the key-down and
//! the char to a built list the way the message loop would and reads where
//! the selection is afterwards, on a list wired with the letter and on one
//! wired with another.

use wxdragon::prelude::*;

/// Whether a key-down is a bare press of `letter`: that letter, with none of
/// Ctrl, Alt or Shift held.
///
/// wxWidgets numbers a letter key by its upper-case ASCII code whatever the
/// case typed, so `pressed` is compared with the letter's upper case. A
/// modifier held means another command, or a capital typed into the search,
/// and neither is this.
pub fn is_a_bare_press_of(
    letter: char,
    pressed: Option<i32>,
    control: bool,
    alt: bool,
    shift: bool,
) -> bool {
    let code = i32::from(letter.to_ascii_uppercase() as u8);
    pressed == Some(code) && !control && !alt && !shift
}

/// Bind `letter` on `list`: a bare press with a selected row calls
/// `on_pressed` with that row and is consumed, so the control's search never
/// gets it; every other key is left as the dispatcher set it.
pub fn wire_letter<F>(list: &ListCtrl, letter: char, on_pressed: F)
where
    F: Fn(i64) + 'static,
{
    let owner = *list;
    list.bind_internal(EventType::KEY_DOWN, move |event| {
        event.skip(true);
        let bare = is_a_bare_press_of(
            letter,
            event.get_key_code(),
            event.control_down(),
            event.alt_down(),
            event.shift_down(),
        );
        if !bare {
            return;
        }
        let selected = owner.get_first_selected_item();
        if selected < 0 {
            return;
        }
        on_pressed(i64::from(selected));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_letter_alone_is_a_bare_press_whatever_case_it_was_given_in() {
        assert!(is_a_bare_press_of('M', Some(77), false, false, false));
        assert!(is_a_bare_press_of('m', Some(77), false, false, false));
    }

    #[test]
    fn test_another_key_is_not() {
        assert!(!is_a_bare_press_of('M', Some(78), false, false, false));
        assert!(!is_a_bare_press_of('M', Some(32), false, false, false));
        assert!(!is_a_bare_press_of('M', None, false, false, false));
    }

    #[test]
    fn test_a_modifier_held_is_another_command_or_a_capital_for_the_search() {
        assert!(!is_a_bare_press_of('M', Some(77), true, false, false));
        assert!(!is_a_bare_press_of('M', Some(77), false, true, false));
        assert!(!is_a_bare_press_of('M', Some(77), false, false, true));
    }
}

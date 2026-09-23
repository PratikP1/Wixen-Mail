//! Where the keyboard goes when the formatted message window comes back.
//!
//! The window that shows a conversation as headings holds a browser, and a
//! screen reader's browse mode only has a document while the keyboard is
//! inside that browser. Switch away and back and the keyboard has to land in
//! the page again, or K and H do nothing.
//!
//! wxWidgets does that by itself nearly every time. When a window is
//! deactivated it remembers the focused child, and when it is activated again
//! it gives the keyboard back to that child
//! (`wxTopLevelWindowMSW::OnActivate`, `target/debug/wxWidgets/src/msw/toplevel.cpp:1326-1361`).
//! 12-01 measured that on a built window on 2026-09-22 and added nothing.
//! The NVDA workflow then reversed it on 2026-09-23: runs 35839692317 and
//! 35839954840 both came back to this window with a real activation and found
//! the keyboard on the window's own frame. One path in wx's own code ends
//! there, read in its source rather than inferred: `IsDescendant` is true for
//! the frame itself (`common/wincmn.cpp:1273-1287`), so a frame holding the
//! keyboard when it is deactivated is saved as its own last focused child,
//! and restored to itself on every activation after that
//! (`common/containr.cpp:611-661`). Whether the runner took that path is what
//! the NVDA case's record says at its next run; a test here can only take the
//! path by messages it sends.
//!
//! So this module answers the activation wx answers: when the window is
//! active and the keyboard is on its frame or on nothing, the page takes it.
//! A control beside the page that somebody moved to keeps it, nothing moves
//! while the window is not the active one, and one activation moves the
//! keyboard once and not again. An activation carrying the minimised flag gets no
//! activate event from wx at all (`msw/window.cpp:4380-4388`), so this cannot
//! answer that one; it is left to the case's record by name.
//!
//! The decision is [`the_page_takes_the_keyboard`], pure and tested arm by
//! arm. The binding reads what Windows says has the keyboard and asks it.

/// Where the keyboard is, as far as the formatted message window cares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhereTheKeyboardIs {
    /// In the page's own window or anything under it, the browser's windows
    /// included.
    InThePage,
    /// On the window's frame itself, where browse mode has no document.
    OnTheFrame,
    /// Nothing on the window's thread holds it.
    Nowhere,
    /// Any other control, or another window's: a list beside the page, a
    /// warning bar, a dialog the window opened.
    OnSomethingElse,
}

/// Where the keyboard is, from the handle holding it (0 for none), the
/// frame's handle, and whether the holder is the page or under it.
pub fn where_the_keyboard_is(
    focus: isize,
    frame: isize,
    focus_is_in_the_page: bool,
) -> WhereTheKeyboardIs {
    let _ = (focus, frame, focus_is_in_the_page);
    WhereTheKeyboardIs::OnSomethingElse
}

/// Whether the page takes the keyboard now: only from the frame or from
/// nothing, only while the window is the active one, and only once per
/// activation.
pub fn the_page_takes_the_keyboard(
    where_it_is: WhereTheKeyboardIs,
    the_window_is_active: bool,
    already_given_this_activation: bool,
) -> bool {
    let _ = (
        where_it_is,
        the_window_is_active,
        already_given_this_activation,
    );
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    const THE_FRAME: isize = 0x100;
    const A_BROWSER_WINDOW: isize = 0x200;
    const A_LIST_BESIDE_THE_PAGE: isize = 0x300;

    #[test]
    fn test_the_page_takes_the_keyboard_from_the_frame_while_the_window_is_active() {
        assert!(the_page_takes_the_keyboard(
            WhereTheKeyboardIs::OnTheFrame,
            true,
            false
        ));
    }

    #[test]
    fn test_the_page_takes_the_keyboard_when_nothing_holds_it_while_the_window_is_active() {
        assert!(the_page_takes_the_keyboard(
            WhereTheKeyboardIs::Nowhere,
            true,
            false
        ));
    }

    #[test]
    fn test_the_keyboard_already_in_the_page_stays_there() {
        assert!(!the_page_takes_the_keyboard(
            WhereTheKeyboardIs::InThePage,
            true,
            false
        ));
    }

    #[test]
    fn test_a_control_beside_the_page_keeps_the_keyboard() {
        assert!(!the_page_takes_the_keyboard(
            WhereTheKeyboardIs::OnSomethingElse,
            true,
            false
        ));
    }

    #[test]
    fn test_nothing_moves_while_the_window_is_not_the_active_one() {
        for where_it_is in [WhereTheKeyboardIs::OnTheFrame, WhereTheKeyboardIs::Nowhere] {
            assert!(
                !the_page_takes_the_keyboard(where_it_is, false, false),
                "moved from {where_it_is:?} while the window was not active"
            );
        }
    }

    #[test]
    fn test_the_keyboard_is_moved_at_most_once_per_activation() {
        for where_it_is in [WhereTheKeyboardIs::OnTheFrame, WhereTheKeyboardIs::Nowhere] {
            assert!(
                !the_page_takes_the_keyboard(where_it_is, true, true),
                "moved from {where_it_is:?} a second time in one activation"
            );
        }
    }

    #[test]
    fn test_where_the_keyboard_is_is_read_from_the_handle_holding_it() {
        assert_eq!(
            where_the_keyboard_is(THE_FRAME, THE_FRAME, false),
            WhereTheKeyboardIs::OnTheFrame
        );
        assert_eq!(
            where_the_keyboard_is(0, THE_FRAME, false),
            WhereTheKeyboardIs::Nowhere
        );
        assert_eq!(
            where_the_keyboard_is(A_BROWSER_WINDOW, THE_FRAME, true),
            WhereTheKeyboardIs::InThePage
        );
        assert_eq!(
            where_the_keyboard_is(A_LIST_BESIDE_THE_PAGE, THE_FRAME, false),
            WhereTheKeyboardIs::OnSomethingElse
        );
    }
}

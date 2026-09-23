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
//! arm. The binding, [`keep_the_keyboard_in_the_page`], reads what Windows
//! says has the keyboard and asks it.

use wxdragon::prelude::*;
use wxdragon::widgets::WebView;

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

impl WhereTheKeyboardIs {
    /// How the log line says where the keyboard was found.
    #[cfg(target_os = "windows")]
    fn in_words(self) -> &'static str {
        match self {
            Self::InThePage => "in the page",
            Self::OnTheFrame => "on the frame",
            Self::Nowhere => "nowhere",
            Self::OnSomethingElse => "on another control",
        }
    }
}

/// Where the keyboard is, from the handle holding it (0 for none), the
/// frame's handle, and whether the holder is the page or under it.
pub fn where_the_keyboard_is(
    focus: isize,
    frame: isize,
    focus_is_in_the_page: bool,
) -> WhereTheKeyboardIs {
    if focus == 0 {
        WhereTheKeyboardIs::Nowhere
    } else if focus_is_in_the_page {
        WhereTheKeyboardIs::InThePage
    } else if focus == frame {
        WhereTheKeyboardIs::OnTheFrame
    } else {
        WhereTheKeyboardIs::OnSomethingElse
    }
}

/// Whether the page takes the keyboard now: only from the frame or from
/// nothing, only while the window is the active one, and only once per
/// activation.
pub fn the_page_takes_the_keyboard(
    where_it_is: WhereTheKeyboardIs,
    the_window_is_active: bool,
    already_given_this_activation: bool,
) -> bool {
    let nobody_chose_where_it_is = matches!(
        where_it_is,
        WhereTheKeyboardIs::OnTheFrame | WhereTheKeyboardIs::Nowhere
    );
    nobody_chose_where_it_is && the_window_is_active && !already_given_this_activation
}

/// A one millisecond one-shot timer: the tree's way to run something on a
/// later turn of a window's event loop rather than inside the event that
/// asked for it, as the formatted window's own way back does.
#[cfg(target_os = "windows")]
const ON_A_LATER_TURN_MS: i32 = 1;

/// What the window knows between its events. Cells, because nothing here
/// leaves the interface thread.
#[cfg(target_os = "windows")]
#[derive(Default)]
struct ThisActivation {
    the_window_is_active: std::cell::Cell<bool>,
    already_given: std::cell::Cell<bool>,
}

/// Give the keyboard to `page` whenever `frame` comes back with it on the
/// frame or on nothing.
///
/// Binds the frame's activation and the frame's own focus event, skipping
/// both so wx's own handlers still run: an activation because it is when the
/// window comes back, and the focus event because wx's restore puts the
/// keyboard on the frame inside that activation, after this handler has run.
/// Either one starts a one-shot check on a later turn of the loop, and a
/// start before the check has ticked restarts it (`common/timerimpl.cpp:60-66`),
/// so an activation and the focus event it causes run one check between them.
/// The check reads the state as it is when it runs, not as it was when it
/// was started, so a deactivation in between leaves the keyboard alone.
///
/// **The timer belongs to the page, not the frame.** A timer event reaches
/// every handler bound on its owner, because wxdragon's `on_tick` binds
/// `TIMER` on the owner with any id (`wxdragon-sys` `event.cpp:617-658`), and
/// the frame already owns the window's way back, whose tick takes the way
/// back out of its cell and runs it. That cell is filled from the moment the
/// window is built. A second timer on the frame would therefore open the
/// conversation again on its first tick, over a window still showing. The
/// page owns no timer of its own, and a timer event is not a command event,
/// so it reaches the page and nothing else. The timer lives as long as the
/// two handlers on the frame that hold it; the check itself holds none.
///
/// When it moves the keyboard it logs one line naming `surface` and where it
/// found the keyboard, never the window's title, which carries a subject.
/// Off Windows it binds nothing: `GetFocus` is what it reads.
#[cfg(target_os = "windows")]
pub fn keep_the_keyboard_in_the_page(frame: &Frame, page: &WebView, surface: &'static str) {
    use wxdragon::event::window_events::WindowEvents;

    let activation = std::rc::Rc::new(ThisActivation::default());
    let check = std::rc::Rc::new(Timer::new(page));
    check.on_tick({
        let (frame, page, activation) = (*frame, *page, activation.clone());
        move |_| give_the_page_the_keyboard_if_it_is_owed(&frame, &page, &activation, surface)
    });
    frame.on_activate({
        let check = check.clone();
        move |event| {
            event.skip(true);
            let is_active = says_the_window_is_active(event);
            activation.the_window_is_active.set(is_active);
            if is_active {
                activation.already_given.set(false);
            }
            check.start(ON_A_LATER_TURN_MS, true);
        }
    });
    frame.on_set_focus(move |event| {
        event.skip(true);
        check.start(ON_A_LATER_TURN_MS, true);
    });
}

/// Whether a frame's activate event says the window became the active one.
///
/// Read by the event's own accessor whatever `WindowEventData` classed it
/// as. Measured 2026-09-23 on wxdragon 0.9.17: a frame's activate event
/// arrives as `General`, not `Activate`, on every activation and
/// deactivation this module's reading sends, so matching on `Activate` alone
/// never saw the window become active and the page never took the keyboard.
/// `wxd_ActivateEvent_IsActive` reads the flag through a checked cast that
/// answers false for any event that is not an activation, so reading a
/// `General` event this way cannot misread something else as active. That is
/// wxdragon's classification, noted here rather than worked around quietly.
#[cfg(target_os = "windows")]
fn says_the_window_is_active(event: WindowEventData) -> bool {
    use wxdragon::event::window_events::ActivateEventData;

    match event {
        WindowEventData::Activate(activated) => activated.is_active(),
        WindowEventData::General(event) => ActivateEventData::new(event).is_active(),
        _ => false,
    }
}

/// Binds nothing: the check reads `GetFocus`, which is Windows'.
#[cfg(not(target_os = "windows"))]
pub fn keep_the_keyboard_in_the_page(_frame: &Frame, _page: &WebView, _surface: &'static str) {}

#[cfg(target_os = "windows")]
fn give_the_page_the_keyboard_if_it_is_owed(
    frame: &Frame,
    page: &WebView,
    activation: &ThisActivation,
    surface: &str,
) {
    let where_it_is = where_the_keyboard_is_now(frame, page);
    if !the_page_takes_the_keyboard(
        where_it_is,
        activation.the_window_is_active.get(),
        activation.already_given.get(),
    ) {
        return;
    }
    page.set_focus();
    activation.already_given.set(true);
    tracing::info!(
        "{surface}: the keyboard was {} when the window came back, so it was given to the page",
        where_it_is.in_words()
    );
}

/// What Windows says holds the keyboard on this thread, placed against the
/// frame and the page.
#[cfg(target_os = "windows")]
fn where_the_keyboard_is_now(frame: &Frame, page: &WebView) -> WhereTheKeyboardIs {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetFocus() -> isize;
        fn IsChild(parent: isize, child: isize) -> i32;
    }

    let page_handle = page.get_handle() as isize;
    // SAFETY: plain reads of this thread's focus and of live handles.
    let focus = unsafe { GetFocus() };
    // SAFETY: as above; `IsChild` answers 0 for a handle that is not a child.
    let under_the_page = focus != 0 && unsafe { IsChild(page_handle, focus) } != 0;
    let focus_is_in_the_page = focus == page_handle || under_the_page;
    where_the_keyboard_is(focus, frame.get_handle() as isize, focus_is_in_the_page)
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

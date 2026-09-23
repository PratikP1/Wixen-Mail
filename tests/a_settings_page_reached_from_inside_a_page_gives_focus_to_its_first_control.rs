//! A Settings page reached from inside another page hands the keyboard to
//! its first control; a page reached from the tab row leaves it on the row.
//!
//! #68, found from the source on 2026-09-17 while tracing #67 and not heard:
//! with focus on a control on the General tab, Ctrl+Tab reached the next
//! page while it was still empty and left native focus on the page panel,
//! so a screen reader said "pane" and nothing else until Tab was pressed.
//! `wxNotebook::SetSelection` calls `UpdateSelection` before it sends the
//! page-changed event (`src/msw/notebook.cpp:342-361`), and `UpdateSelection`
//! gives the reached page focus when the notebook is shown and does not hold
//! focus itself (`:364-391`). Since 09-09 the page is built on that event,
//! after the focus has landed, and the empty panel is where it landed.
//!
//! Ctrl+Tab cannot be sent to a window. wxWidgets turns it into a page change
//! in `MSWProcessMessage`, which the event loop calls on a message it pulled
//! from the queue and which reads the Control key from the physical keyboard
//! (`src/msw/window.cpp`, `wxIsCtrlDown`), not from the message. A
//! `WM_KEYDOWN` sent to a window procedure goes to `HandleKeyDown` and
//! changes no page. Everything after that translation is wx's own code ending
//! in `SetSelection`, the same call the arrow handler makes, so the reading
//! puts focus on a General control and calls `set_selection`, which is the
//! fault's exact precondition: the notebook shown and not holding focus.
//! Injecting a real Control key with `SendInput` would change the machine's
//! keyboard state under a test and is not done. `SetSelection`, `SetFocus`
//! and `WM_SETFOCUS` are synchronous on the calling thread, so `GetFocus()`
//! answers as soon as `set_selection` returns and no loop is pumped.
//!
//! The first control of a page is read the way Windows reads a tab order:
//! the first child of the panel in sibling order that carries `WS_TABSTOP`
//! and is enabled. That is the pick `wxSetFocusToChild` makes too, so a later
//! plan that puts a new control first on a page and forgets to move the
//! hand-off reddens this reading rather than the tester.
//!
//! One window session serves three tests. The budget is one `wxdragon::main`
//! per process (`tests/theme_reach.rs` records the "initializing twice?" hang
//! a second one produced), so the windows are built once inside a `OnceLock`
//! by whichever test asks first, every reading is harvested into plain
//! values, and the three tests assert over the harvest. The initialiser
//! stores a `Result` and panics on nothing itself: a panic inside
//! `OnceLock::get_or_init` leaves the cell empty and the next test would
//! spend the budget a second time.

#![cfg(windows)]

use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::data::config::AppConfig;
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::wx_settings;
use wxdragon::prelude::*;

/// Which tab is which, in the order the dialog adds them.
const THE_GENERAL_TAB: usize = 0;
const THE_COMPOSE_TAB: usize = 1;
const THE_READING_TAB: usize = 2;

const GW_HWNDNEXT: u32 = 2;
const GW_CHILD: u32 = 5;
const GWL_STYLE: i32 = -16;
const WS_TABSTOP: u32 = 0x0001_0000;
const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const VK_RIGHT: usize = 0x27;
/// The up-down message asking for the field beside the arrows (commctrl.h).
const UDM_GETBUDDY: u32 = 0x400 + 106;

#[link(name = "user32")]
unsafe extern "system" {
    fn GetFocus() -> isize;
    fn SetFocus(hwnd: isize) -> isize;
    fn IsChild(parent: isize, hwnd: isize) -> i32;
    fn IsWindowEnabled(hwnd: isize) -> i32;
    fn GetWindow(hwnd: isize, which: u32) -> isize;
    fn GetWindowLongPtrW(hwnd: isize, index: i32) -> isize;
    fn GetClassNameW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
    fn GetWindowTextW(hwnd: isize, buffer: *mut u16, count: i32) -> i32;
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
}

fn class_name(hwnd: isize) -> String {
    let mut buffer = [0u16; 256];
    // SAFETY: the buffer is as long as the count says.
    let len = unsafe { GetClassNameW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

fn window_text(hwnd: isize) -> String {
    let mut buffer = [0u16; 512];
    // SAFETY: the buffer is as long as the count says.
    let len = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

fn style_of(hwnd: isize) -> u32 {
    // SAFETY: a live window handle.
    (unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) } as u64 & 0xFFFF_FFFF) as u32
}

fn focused_window() -> isize {
    // SAFETY: reads the calling thread's focus window.
    unsafe { GetFocus() }
}

/// One window named enough to say in a failure message.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Named {
    hwnd: isize,
    class: String,
    text: String,
    style: u32,
}

impl Named {
    fn of(hwnd: isize) -> Self {
        Named {
            hwnd,
            class: class_name(hwnd),
            text: window_text(hwnd),
            style: style_of(hwnd),
        }
    }

    fn describe(&self) -> String {
        format!(
            "0x{:x} class {} text {:?} style 0x{:08x}",
            self.hwnd, self.class, self.text, self.style
        )
    }
}

/// The first child of `panel` in sibling order that Windows would tab to:
/// carrying `WS_TABSTOP` and enabled.
fn first_tab_stop_under(panel: isize) -> Option<Named> {
    // SAFETY: live window handles; the walk reads and moves nothing.
    let mut child = unsafe { GetWindow(panel, GW_CHILD) };
    while child != 0 {
        // SAFETY: a live window handle.
        let enabled = unsafe { IsWindowEnabled(child) } != 0;
        if style_of(child) & WS_TABSTOP != 0 && enabled {
            return Some(Named::of(child));
        }
        // SAFETY: a live window handle.
        child = unsafe { GetWindow(child, GW_HWNDNEXT) };
    }
    None
}

/// A key pressed and released on the window the way the message loop hands
/// a real main-keyboard arrow to it, the shape
/// `tests/the_settings_tab_row_says_each_tab_once.rs` uses.
fn press(hwnd: isize, key: usize) {
    const KF_EXTENDED: isize = 0x0100 << 16;
    // SAFETY: `hwnd` is a live window on this thread, built by the caller.
    unsafe {
        SendMessageW(hwnd, WM_KEYDOWN, key, 1 | KF_EXTENDED);
        SendMessageW(
            hwnd,
            WM_KEYUP,
            key,
            0xC000_0001_u32 as i32 as isize | KF_EXTENDED,
        );
    }
}

/// Where the keyboard was left after one page change, and what the reached
/// page's first control was.
#[derive(Debug, Clone)]
struct Landing {
    what: &'static str,
    panel: Named,
    focus: Named,
    focus_is_a_descendant_of_the_panel: bool,
    first_tab_stop: Option<Named>,
}

impl Landing {
    fn take(what: &'static str, panel: &Panel) -> Self {
        let panel_hwnd = panel.get_handle() as isize;
        let focus = focused_window();
        Landing {
            what,
            panel: Named::of(panel_hwnd),
            focus: Named::of(focus),
            // SAFETY: live window handles.
            focus_is_a_descendant_of_the_panel: unsafe { IsChild(panel_hwnd, focus) } != 0,
            first_tab_stop: first_tab_stop_under(panel_hwnd),
        }
    }

    /// Whether the keyboard rests on the reached page's first control: not
    /// on the panel, inside it, on something Windows would tab to, and on
    /// the first such thing.
    fn rests_on_the_first_control(&self) -> Result<(), String> {
        let first = match &self.first_tab_stop {
            Some(first) => first,
            None => {
                return Err(format!(
                    "{}: the page panel {} has no enabled tab-stop child at all",
                    self.what,
                    self.panel.describe()
                ));
            }
        };
        if self.focus.hwnd == self.panel.hwnd {
            return Err(format!(
                "{}: focus is on the page panel itself, {}, and not on its first control, {}",
                self.what,
                self.panel.describe(),
                first.describe()
            ));
        }
        if !self.focus_is_a_descendant_of_the_panel {
            return Err(format!(
                "{}: focus is on {}, which is not inside the page panel {}; the first control is {}",
                self.what,
                self.focus.describe(),
                self.panel.describe(),
                first.describe()
            ));
        }
        if self.focus.style & WS_TABSTOP == 0 {
            return Err(format!(
                "{}: focus is on {}, which carries no WS_TABSTOP; the first control is {}",
                self.what,
                self.focus.describe(),
                first.describe()
            ));
        }
        if self.focus.hwnd != first.hwnd {
            return Err(format!(
                "{}: focus is on {}, not on the first tab-stop child {}",
                self.what,
                self.focus.describe(),
                first.describe()
            ));
        }
        Ok(())
    }
}

/// Where the keyboard was left after an arrow on the tab row.
#[derive(Debug, Clone)]
struct RowLanding {
    focus: Named,
    notebook: isize,
    selection: i32,
}

/// Everything the window session read, as plain values; no handle survives it.
#[derive(Debug)]
struct Harvest {
    from_a_general_control_into_compose: Landing,
    from_a_compose_control_into_reading: Landing,
    an_arrow_on_the_row: RowLanding,
    focus_planted_on_a_built_panel: Landing,
}

/// Put native focus on `hwnd` by hand and say so if Windows refused.
fn focus_by_hand(hwnd: isize, what: &str) -> Result<(), String> {
    // SAFETY: a live window on this thread.
    unsafe { SetFocus(hwnd) };
    let now = focused_window();
    if now == hwnd {
        Ok(())
    } else {
        Err(format!(
            "SetFocus on {what} (0x{hwnd:x}) left focus on {}",
            Named::of(now).describe()
        ))
    }
}

/// The session: one dialog, three readings and the companion, with every
/// failure carried out as a value. Nothing in here panics.
fn read_the_dialog(frame: &Frame, a11y: &Arc<Accessibility>) -> Result<Harvest, String> {
    let widgets =
        wx_settings::build_settings_dialog(frame, &AppConfig::default(), &[], false, a11y);
    widgets.dialog.show(true);

    // Reading A, first step: from a General control into Compose, which is
    // built by the page-changed handler as the selection moves.
    widgets.font_size.set_focus();
    // A spin control since 12-06 (#35): focus lands in the field a person
    // types in, the arrows' buddy, and not on the arrows `get_handle` answers.
    // SAFETY: a live window this file built; the message takes no pointer.
    let font_size =
        unsafe { SendMessageW(widgets.font_size.get_handle() as isize, UDM_GETBUDDY, 0, 0) };
    if focused_window() != font_size {
        return Err(format!(
            "set_focus on the font size field (0x{font_size:x}) left focus on {}",
            Named::of(focused_window()).describe()
        ));
    }
    widgets.notebook.set_selection(THE_COMPOSE_TAB);
    let from_a_general_control_into_compose = Landing::take(
        "from a General control into Compose",
        &widgets.compose_panel,
    );

    // Second step: from a Compose control into Reading, so a page reached
    // from a page that was itself built lazily is read too. The Compose
    // control is the first tab stop the reading just found, focused by hand
    // through user32 because the page's controls are not public.
    let first_on_compose = from_a_general_control_into_compose
        .first_tab_stop
        .as_ref()
        .ok_or_else(|| "Compose has no enabled tab-stop child to start from".to_string())?;
    focus_by_hand(first_on_compose.hwnd, "the first Compose control")?;
    widgets.notebook.set_selection(THE_READING_TAB);
    let from_a_compose_control_into_reading = Landing::take(
        "from a Compose control into Reading",
        &widgets.reading_panel,
    );

    // Reading B: the row holds focus and one Right arrow reaches Compose.
    widgets.notebook.set_focus();
    widgets.notebook.set_selection(THE_GENERAL_TAB);
    let tab_row = widgets.notebook.get_handle() as isize;
    if focused_window() != tab_row {
        return Err(format!(
            "set_focus on the tab row (0x{tab_row:x}) left focus on {}",
            Named::of(focused_window()).describe()
        ));
    }
    press(tab_row, VK_RIGHT);
    let an_arrow_on_the_row = RowLanding {
        focus: Named::of(focused_window()),
        notebook: tab_row,
        selection: widgets.notebook.selection(),
    };

    // Reading C, the companion: focus put on a built page's panel by hand,
    // read through the same fields.
    widgets.notebook.set_selection(THE_COMPOSE_TAB);
    focus_by_hand(
        widgets.compose_panel.get_handle() as isize,
        "the Compose page panel",
    )?;
    let focus_planted_on_a_built_panel = Landing::take(
        "focus planted on the built Compose panel",
        &widgets.compose_panel,
    );

    widgets.dialog.destroy();
    Ok(Harvest {
        from_a_general_control_into_compose,
        from_a_compose_control_into_reading,
        an_arrow_on_the_row,
        focus_planted_on_a_built_panel,
    })
}

fn take_the_harvest() -> Result<Harvest, String> {
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let taken: Result<Harvest, String> = (|| {
                let frame = Frame::builder().build();
                let a11y = Arc::new(
                    Accessibility::new()
                        .map_err(|why| format!("Accessibility::new failed: {why}"))?,
                );
                read_the_dialog(&frame, &a11y)
            })();
            if let Ok(mut slot) = outcome.lock() {
                *slot = Some(taken);
            }
            wxdragon::call_after(Box::new(move || {
                app.exit_main_loop();
            }));
        })
    };
    if let Err(why) = result {
        return Err(format!("wxdragon::main returned {why:?}"));
    }
    let taken = outcome
        .lock()
        .map_err(|_| "the harvest's lock was poisoned".to_string())?
        .take();
    taken.unwrap_or_else(|| Err("the window session ended without a harvest".to_string()))
}

/// The one harvest of this process, taken by whichever test asks first.
fn the_harvest() -> &'static Harvest {
    static HARVEST: OnceLock<Result<Harvest, String>> = OnceLock::new();
    match HARVEST.get_or_init(take_the_harvest) {
        Ok(harvest) => harvest,
        Err(why) => panic!("the window session could not be read: {why}"),
    }
}

fn complain(what: &str, wrong: &[String]) {
    assert!(
        wrong.is_empty(),
        "{} thing(s) wrong, {what}:\n  {}",
        wrong.len(),
        wrong.join("\n  ")
    );
}

#[test]
fn test_a_page_reached_from_inside_a_page_puts_focus_on_its_first_control() {
    let harvest = the_harvest();
    let wrong: Vec<String> = [
        &harvest.from_a_general_control_into_compose,
        &harvest.from_a_compose_control_into_reading,
    ]
    .into_iter()
    .filter_map(|landing| landing.rests_on_the_first_control().err())
    .collect();
    complain(
        "a page reached from inside another page should leave focus on its first tab-stop \
         control",
        &wrong,
    );
}

#[test]
fn test_a_page_reached_from_the_tab_row_leaves_focus_on_the_row() {
    let harvest = the_harvest();
    let row = &harvest.an_arrow_on_the_row;
    let mut wrong = Vec::new();
    if row.focus.hwnd != row.notebook {
        wrong.push(format!(
            "after Right on the tab row focus is on {}, not on the row 0x{:x}",
            row.focus.describe(),
            row.notebook
        ));
    }
    if row.selection != THE_COMPOSE_TAB as i32 {
        wrong.push(format!(
            "after Right from General the selection is {}, not {}",
            row.selection, THE_COMPOSE_TAB
        ));
    }
    complain(
        "an arrow on the tab row should move the selection and leave focus on the row, which \
         is #33's invariant",
        &wrong,
    );
}

#[test]
fn test_the_reading_tells_the_page_panel_from_a_control_on_it() {
    let harvest = the_harvest();
    let planted = &harvest.focus_planted_on_a_built_panel;
    let mut wrong = Vec::new();
    match planted.rests_on_the_first_control() {
        Err(why) if why.contains("focus is on the page panel itself") => {}
        other => wrong.push(format!(
            "the reading answered {other:?} for focus planted on the panel by hand, where it \
             should complain that focus is on the page panel itself"
        )),
    }
    if planted.panel.style & WS_TABSTOP != 0 {
        wrong.push(format!(
            "the page panel {} carries WS_TABSTOP, so \"on a tab stop\" could be satisfied by the \
             panel",
            planted.panel.describe()
        ));
    }
    if planted.focus.hwnd != planted.panel.hwnd {
        wrong.push(format!(
            "focus planted on the panel by hand did not stay there: it is on {}",
            planted.focus.describe()
        ));
    }
    complain(
        "the reading should see focus on a page panel as wrong, so the first test cannot pass \
         by reading nothing",
        &wrong,
    );
}

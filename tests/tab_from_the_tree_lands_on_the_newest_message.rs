//! Tab from the folder tree lands on a row of the message list, not on the
//! list itself.
//!
//! The tester on 2026-09-18 (#87), under NVDA: Tab from the folder tree to
//! the message list lands on the list with no row under the cursor, read as
//! "list", and Down then lands on the first row. Opening a folder loads the
//! rows and, by 10-02's rule, selects nothing while focus is in the tree;
//! nothing landed a row when focus later arrived. The wiring is
//! `presentation::list_arrival::wire` on the list's focus event, the one
//! path Tab, F6 and a click share, and the rule is
//! `presentation::landing_after_a_removal::where_to_land_on_arrival`.
//!
//! **The built window.** A frame holding a `TreeCtrl` with one item and a
//! `ListCtrl` in virtual report mode, the shape the message list has, three
//! rows drawn from a callback, wired through `list_arrival::wire` with a
//! `choose` over a cell standing for the window's remembered row and a
//! counter of how often it was asked. Four steps, each one focus moved from
//! the tree to the list by `set_focus`, which is the list's own `SET_FOCUS`
//! path and the path F6 takes in the window; the focused and selected item
//! read back from the control after each:
//!
//! 1. Nothing remembered, no item focused: the cursor lands on row 0, the
//!    newest message under the default sort.
//! 2. The cursor cleared and the cell holding row 2: it lands on row 2.
//! 3. The cursor already on row 1: nothing moves and `choose` is not asked,
//!    which is coming back to the list.
//! 4. A second list with no rows, wired the same way: no item is focused
//!    and `on_empty` was called once.
//!
//! Each arrival is taken under an in-context win-event hook recording the
//! focus events the list raised and which child each named, because a
//! screen reader reads the landed row from that event: on the first arrival
//! at least one event names the landed row, and the whole sequence is kept
//! so the summary can say what the control raised and in what order.
//!
//! One `#[test]`, on `tests/theme_reach.rs`'s budget of one window session
//! per process; the steps are inside it.
//!
//! The Windows calls are declared by hand from the headers, as every reading
//! in `tests/` does, so the `windows` crate is not compiled in for a test.

#![cfg(windows)]

use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wixen_mail::presentation::list_arrival;
use wxdragon::prelude::*;

const EVENT_OBJECT_FOCUS: u32 = 0x8005;
const OBJID_CLIENT: i32 = -4;
const WINEVENT_INCONTEXT: u32 = 0x0004;

#[link(name = "user32")]
unsafe extern "system" {
    fn SetWinEventHook(
        min: u32,
        max: u32,
        module: *mut c_void,
        callback: extern "system" fn(*mut c_void, u32, isize, i32, i32, u32, u32),
        process: u32,
        thread: u32,
        flags: u32,
    ) -> *mut c_void;
    fn UnhookWinEvent(hook: *mut c_void) -> i32;
    fn GetCurrentThreadId() -> u32;
    fn GetFocus() -> isize;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcessId() -> u32;
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
}

thread_local! {
    static FOCUS_RAISED: RefCell<Vec<(isize, i32)>> = const { RefCell::new(Vec::new()) };
}

extern "system" fn record(
    _hook: *mut c_void,
    event: u32,
    hwnd: isize,
    id_object: i32,
    id_child: i32,
    _thread: u32,
    _time: u32,
) {
    if event != EVENT_OBJECT_FOCUS || id_object != OBJID_CLIENT {
        return;
    }
    FOCUS_RAISED.with(|raised| raised.borrow_mut().push((hwnd, id_child)));
}

/// The children the focus events `hwnd` raised while `act` ran named, in
/// order: 0 for the list itself, `row + 1` for a row.
///
/// An in-context hook on this thread alone: `NotifyWinEvent` calls it before
/// returning, so everything the control raised inside `act` is recorded by
/// the time `act` returns, and nothing another window raised is.
fn focus_events_raised_by(hwnd: isize, act: impl FnOnce()) -> Result<Vec<i32>, String> {
    FOCUS_RAISED.with(|raised| raised.borrow_mut().clear());
    // SAFETY: the callback is a plain function that touches only this
    // thread's local, and the hook is removed before this function returns.
    let hook = unsafe {
        SetWinEventHook(
            EVENT_OBJECT_FOCUS,
            EVENT_OBJECT_FOCUS,
            GetModuleHandleW(std::ptr::null()),
            record,
            GetCurrentProcessId(),
            GetCurrentThreadId(),
            WINEVENT_INCONTEXT,
        )
    };
    if hook.is_null() {
        return Err("the win-event hook could not be set".to_string());
    }
    act();
    // SAFETY: `hook` came from the call above and has not been unhooked.
    unsafe { UnhookWinEvent(hook) };
    Ok(FOCUS_RAISED.with(|raised| {
        raised
            .borrow()
            .iter()
            .filter(|(on, _)| *on == hwnd)
            .map(|(_, child)| *child)
            .collect()
    }))
}

/// Where the cursor is, read from the control: the focused row and the
/// selected row, or -1 for none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cursor {
    focused: i32,
    selected: i32,
}

const NOWHERE: Cursor = Cursor {
    focused: -1,
    selected: -1,
};

fn on(row: i32) -> Cursor {
    Cursor {
        focused: row,
        selected: row,
    }
}

fn the_cursor_of(list: &ListCtrl) -> Cursor {
    Cursor {
        focused: list.get_next_item(-1, ListNextItemFlag::All, ListItemState::Focused),
        selected: list.get_first_selected_item(),
    }
}

/// What the built window answered at each step.
#[derive(Debug, Clone)]
struct Harvest {
    /// Whether the tree held focus before each arrival, which it has to for
    /// `set_focus` on the list to be an arrival at all.
    the_tree_held_focus_before: [bool; 4],
    /// Step 1: nothing remembered, no item focused.
    before_the_first_arrival: Cursor,
    after_the_first_arrival: Cursor,
    asked_by_the_first_arrival: usize,
    focus_events_on_the_first_arrival: Vec<i32>,
    /// Step 2: the cursor cleared and row 2 remembered.
    after_the_second_arrival: Cursor,
    asked_by_the_second_arrival: usize,
    /// Step 3: the cursor already on row 1.
    before_coming_back: Cursor,
    after_coming_back: Cursor,
    asked_by_coming_back: usize,
    focus_events_on_coming_back: Vec<i32>,
    /// Step 4: a list with no rows.
    on_the_empty_list: Cursor,
    said_it_was_empty: usize,
}

fn a_virtual_list_of(frame: &Frame, rows: usize) -> ListCtrl {
    let list = ListCtrl::builder(frame)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::Virtual)
        .build();
    list.insert_column(0, "Subject", ListColumnFormat::Left, 160);
    list.set_virtual_text_callback(move |row, _column| format!("Message {row}"));
    list.set_item_count(rows as i64);
    list
}

fn holds_focus(window_handle: isize) -> bool {
    // SAFETY: a plain read of this thread's focus window.
    unsafe { GetFocus() == window_handle }
}

fn take_the_harvest() -> Result<Harvest, String> {
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let taken: Result<Harvest, String> = (|| {
                let frame = Frame::builder()
                    .with_title("Tab from the tree, the reading")
                    .with_size(Size::new(480, 240))
                    .build();
                let tree = TreeCtrl::builder(&frame).build();
                let root = tree
                    .add_root("Mail Folders", None, None)
                    .ok_or("the tree has no root")?;
                tree.append_item(&root, "Inbox", None, None);
                let list = a_virtual_list_of(&frame, 3);
                let remembered: Rc<Cell<Option<usize>>> = Rc::new(Cell::new(None));
                let asked: Rc<Cell<usize>> = Rc::new(Cell::new(0));
                list_arrival::wire(
                    &list,
                    {
                        let remembered = remembered.clone();
                        let asked = asked.clone();
                        move || {
                            asked.set(asked.get() + 1);
                            wixen_mail::presentation::landing_after_a_removal::where_to_land_on_arrival(
                                remembered.get(),
                                3,
                            )
                        }
                    },
                    || {},
                );
                // Shown, because the focused item and the events are read
                // from a control somebody could see; the frame is destroyed
                // before the session ends.
                frame.show(true);
                let tree_handle = tree.get_handle() as isize;
                let list_handle = list.get_handle() as isize;
                if tree_handle == 0 || list_handle == 0 {
                    return Err("a control has no window handle".to_string());
                }
                let mut the_tree_held_focus_before = [false; 4];

                // Step 1: from the tree, with nothing remembered and no item
                // focused, the way a folder just opened leaves the list.
                tree.set_focus();
                the_tree_held_focus_before[0] = holds_focus(tree_handle);
                let before_the_first_arrival = the_cursor_of(&list);
                let focus_events_on_the_first_arrival =
                    focus_events_raised_by(list_handle, || list.set_focus())?;
                let after_the_first_arrival = the_cursor_of(&list);
                let asked_by_the_first_arrival = asked.get();

                // Step 2: the cursor cleared, the way a fresh list has none,
                // and row 2 remembered.
                let both = ListItemState::Selected | ListItemState::Focused;
                for row in 0..3 {
                    list.set_item_state(row, ListItemState::None, both);
                }
                remembered.set(Some(2));
                tree.set_focus();
                the_tree_held_focus_before[1] = holds_focus(tree_handle);
                list.set_focus();
                let after_the_second_arrival = the_cursor_of(&list);
                let asked_by_the_second_arrival = asked.get();

                // Step 3: the cursor on row 1, as arrowing there leaves it,
                // then away to the tree and back.
                list.set_item_state(1, both, both);
                remembered.set(Some(0));
                let before_coming_back = the_cursor_of(&list);
                tree.set_focus();
                the_tree_held_focus_before[2] = holds_focus(tree_handle);
                let focus_events_on_coming_back =
                    focus_events_raised_by(list_handle, || list.set_focus())?;
                let after_coming_back = the_cursor_of(&list);
                let asked_by_coming_back = asked.get();

                // Step 4: a list with no rows, wired the same way.
                let empty = a_virtual_list_of(&frame, 0);
                let said: Rc<Cell<usize>> = Rc::new(Cell::new(0));
                list_arrival::wire(
                    &empty,
                    || {
                        wixen_mail::presentation::landing_after_a_removal::where_to_land_on_arrival(
                            None, 0,
                        )
                    },
                    {
                        let said = said.clone();
                        move || said.set(said.get() + 1)
                    },
                );
                tree.set_focus();
                the_tree_held_focus_before[3] = holds_focus(tree_handle);
                empty.set_focus();
                let on_the_empty_list = the_cursor_of(&empty);
                let said_it_was_empty = said.get();

                frame.destroy();
                Ok(Harvest {
                    the_tree_held_focus_before,
                    before_the_first_arrival,
                    after_the_first_arrival,
                    asked_by_the_first_arrival,
                    focus_events_on_the_first_arrival,
                    after_the_second_arrival,
                    asked_by_the_second_arrival,
                    before_coming_back,
                    after_coming_back,
                    asked_by_coming_back,
                    focus_events_on_coming_back,
                    on_the_empty_list,
                    said_it_was_empty,
                })
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

#[test]
fn test_focus_arriving_from_the_tree_lands_on_the_remembered_row_or_the_first_and_an_empty_list_says_so()
 {
    let harvest = match take_the_harvest() {
        Ok(harvest) => harvest,
        Err(why) => panic!("the window session could not be read: {why}"),
    };
    eprintln!("{harvest:#?}");
    assert_eq!(
        harvest.the_tree_held_focus_before, [true; 4],
        "the tree has to hold focus before each arrival, or set_focus on the list moves \
         nothing and the reading is of nothing"
    );

    // Step 1: a list filled while focus was in the tree holds no row, and
    // the arrival lands on the first.
    assert_eq!(
        harvest.before_the_first_arrival, NOWHERE,
        "before the first arrival the list held a row already, so nothing below is about \
         the case the tester met"
    );
    assert_eq!(
        harvest.after_the_first_arrival,
        on(0),
        "focus arriving with nothing remembered: the control's focused and selected item"
    );
    assert_eq!(
        harvest.asked_by_the_first_arrival, 1,
        "choose asked once on the arrival"
    );
    assert!(
        harvest.focus_events_on_the_first_arrival.contains(&1),
        "no focus event on the arrival named row 0 (child 1); the events named {:?}, so a \
         screen reader reads the list and not the row",
        harvest.focus_events_on_the_first_arrival
    );

    // Step 2: the remembered row, still there.
    assert_eq!(
        harvest.after_the_second_arrival,
        on(2),
        "focus arriving with row 2 remembered: the control's focused and selected item"
    );
    assert_eq!(harvest.asked_by_the_second_arrival, 2);

    // Step 3: coming back to a list that holds a row moves nothing and asks
    // nothing, whatever is remembered.
    assert_eq!(harvest.before_coming_back, on(1));
    assert_eq!(
        harvest.after_coming_back,
        on(1),
        "coming back to the list with the cursor on row 1 moved it"
    );
    assert_eq!(
        harvest.asked_by_coming_back, 2,
        "coming back to a list that holds a row asked choose, so the landing fires on every \
         focus event and not only on arrival"
    );
    assert!(
        !harvest.focus_events_on_coming_back.contains(&1),
        "coming back raised a focus event naming row 0 (child 1): {:?}",
        harvest.focus_events_on_coming_back
    );

    // Step 4: a list with no rows takes focus without a row and says so once.
    assert_eq!(harvest.on_the_empty_list, NOWHERE);
    assert_eq!(
        harvest.said_it_was_empty, 1,
        "an empty list said it was empty {} times on one arrival",
        harvest.said_it_was_empty
    );
}

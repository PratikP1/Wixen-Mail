//! After a delete the cursor is on the next message, or the previous one
//! when the last was deleted, on the list itself; a re-read keeps it on the
//! same message by identity; and a delete says one word.
//!
//! The tester on 2026-09-18 (#76), under NVDA: deleting a message puts the
//! cursor at the top of the list. The rule that lands it on the next message
//! was already in the window's own record of the selection; nothing set the
//! control's focused and selected item to it, and the control is what a
//! screen reader follows. And since 10-06 our own expunge wakes the watch,
//! whose re-read replaces the rows without re-selecting, by 10-02's design,
//! so the surviving row could be a different message.
//!
//! **The built list.** A `ListCtrl` in virtual report mode, the shape the
//! message list has, five rows drawn from a callback over a list of ids,
//! row 2 focused and selected. Row 2 is taken out the way the window takes a
//! row out, the count and the repaint first and then the same helper the
//! window uses, `presentation::wx_app::land_the_cursor_after`, and the
//! control's focused item is read back with `get_next_item` and its
//! selection with `get_first_selected_item`: the next message, which now
//! sits on row 2. Then the cursor is put on the last row and that row is
//! taken out, and the cursor lands on the new last. What the control did on
//! its own is read between the count and the landing, and kept as the
//! record of the day: it holds the row when the index stays in range and
//! holds nothing when it does not, which is the top of the list the tester
//! met. Both landings are taken under an in-context win-event hook counting
//! `EVENT_OBJECT_FOCUS` on the list, because a screen reader reads the
//! landed row from that event and a landing on a row the control already
//! held would otherwise raise none. Then the ids are replaced the way a
//! re-read replaces them, with the cursor's message moved to another row,
//! through `keep_the_cursor_on_its_message`, and the focused row follows
//! it. Then a re-read that leaves the cursor's message where it was, under
//! the same hook: nothing is raised, which is 10-02's no-move held; the
//! companion is the move before it, which raised the event, so a count of
//! nought is a count that can see one.
//!
//! **What a delete says** (#83). The tester the same day: a delete said
//! "Deleting <subject>..." on the key and "Deleted: <subject>" after the
//! server's round trip, two spoken sentences, and the wait for the second
//! was a delay in the hand's rhythm. Pratik's decision: say "Delete" and
//! nothing more, and something only when the delete did not go through.
//! The readings below hold the Delete arm to the one word at the key with
//! the fuller line written for the eye, the outcome to being shown when the
//! row left and spoken when it stayed, and the shown channel to speaking
//! nothing.
//!
//! One window session for the whole file, on the shape
//! `tests/a_kept_folder_reads_as_a_checked_check_box.rs` set and for the
//! reason `tests/theme_reach.rs` gives, every reading sharing it through a
//! `OnceLock`. The readings over the main window's source, which hold the
//! two arms to calling the helpers and the delete to its words, are
//! functions over `what_ships` with companions, and need no window.
//!
//! The Windows calls are declared by hand from the headers, as every reading
//! in `tests/` does, so the `windows` crate is not compiled in for a test.

#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::presentation::wx_app::{keep_the_cursor_on_its_message, land_the_cursor_after};
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
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcessId() -> u32;
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
}

thread_local! {
    static FOCUS_RAISED_ON: RefCell<Vec<isize>> = const { RefCell::new(Vec::new()) };
}

extern "system" fn record(
    _hook: *mut c_void,
    event: u32,
    hwnd: isize,
    id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    if event != EVENT_OBJECT_FOCUS || id_object != OBJID_CLIENT {
        return;
    }
    FOCUS_RAISED_ON.with(|raised| raised.borrow_mut().push(hwnd));
}

/// How many focus events `hwnd` raised while `act` ran.
///
/// An in-context hook on this thread alone: `NotifyWinEvent` calls it before
/// returning, so everything the control raised inside `act` is counted by
/// the time `act` returns, and nothing another window raised is.
fn focus_events_raised_by(hwnd: isize, act: impl FnOnce()) -> Result<usize, String> {
    FOCUS_RAISED_ON.with(|raised| raised.borrow_mut().clear());
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
    Ok(FOCUS_RAISED_ON.with(|raised| raised.borrow().iter().filter(|on| **on == hwnd).count()))
}

/// The ids the five rows start with. The cursor is put on row 2, which is
/// message 30.
const THE_ROWS: [i64; 5] = [10, 20, 30, 40, 50];
const THE_CURSOR_ROW: usize = 2;

/// Where the cursor is, read from the control: the focused row and the
/// selected row, or -1 for none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cursor {
    focused: i32,
    selected: i32,
}

fn the_cursor_of(list: &ListCtrl) -> Cursor {
    Cursor {
        focused: list.get_next_item(-1, ListNextItemFlag::All, ListItemState::Focused),
        selected: list.get_first_selected_item(),
    }
}

/// What the built list answered at each step.
#[derive(Debug, Clone)]
struct Harvest {
    /// The cursor before anything was removed, which has to be row 2 for the
    /// rest to mean anything.
    before: Cursor,
    /// Step 1: row 2 of five removed. What the control held on its own once
    /// its count changed, then after the landing, and the helper's answer.
    left_by_the_control_after_a_middle_row: Cursor,
    after_a_middle_row_left: Cursor,
    landed_after_a_middle_row: Option<usize>,
    /// The focus events the landing raised, which is what a screen reader
    /// reads the landed row from.
    focus_events_on_landing_after_a_middle_row: usize,
    /// Step 2: the cursor put on the last row of four, then that row removed.
    before_the_last_row_left: Cursor,
    left_by_the_control_after_the_last_row: Cursor,
    after_the_last_row_left: Cursor,
    landed_after_the_last_row: Option<usize>,
    focus_events_on_landing_after_the_last_row: usize,
    /// Step 3: the rows replaced with the cursor's message on another row,
    /// the focus events the move raised, and the helper's answer.
    after_a_re_read_that_moved_it: Cursor,
    focus_events_when_it_moved: usize,
    followed_to: Option<usize>,
    /// Step 4: the rows replaced with the cursor's message where it was.
    after_a_re_read_that_left_it: Cursor,
    focus_events_when_it_stayed: usize,
    stayed: Option<usize>,
}

fn a_virtual_list_over(frame: &Frame, ids: &Rc<RefCell<Vec<i64>>>) -> ListCtrl {
    let list = ListCtrl::builder(frame)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel | ListCtrlStyle::Virtual)
        .build();
    list.insert_column(0, "Subject", ListColumnFormat::Left, 160);
    let rows = ids.clone();
    list.set_virtual_text_callback(move |row, _column| {
        rows.borrow()
            .get(row as usize)
            .map(|id| format!("Message {id}"))
            .unwrap_or_default()
    });
    list.set_item_count(ids.borrow().len() as i64);
    list
}

fn take_the_harvest() -> Result<Harvest, String> {
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let taken: Result<Harvest, String> = (|| {
                let frame = Frame::builder()
                    .with_title("Deleting a message, the reading")
                    .with_size(Size::new(320, 240))
                    .build();
                let ids = Rc::new(RefCell::new(THE_ROWS.to_vec()));
                let list = a_virtual_list_over(&frame, &ids);
                // Shown, because the focused item and the events are read
                // from a control somebody could see; the frame is destroyed
                // before the session ends.
                frame.show(true);
                list.set_focus();
                let hwnd = list.get_handle() as isize;
                if hwnd == 0 {
                    return Err("the list has no window handle".to_string());
                }
                list.set_item_state(
                    THE_CURSOR_ROW as i64,
                    ListItemState::Selected | ListItemState::Focused,
                    ListItemState::Selected | ListItemState::Focused,
                );
                let before = the_cursor_of(&list);

                // Step 1: the middle row goes, the way the window takes a row
                // out: the rows first, the count and the repaint second, the
                // landing third. What the control did on its own is read
                // between the second and the third.
                let removed = THE_CURSOR_ROW;
                ids.borrow_mut().remove(removed);
                let len_after = ids.borrow().len();
                list.set_item_count(len_after as i64);
                list.refresh(true, None);
                let left_by_the_control_after_a_middle_row = the_cursor_of(&list);
                let mut landed_after_a_middle_row = None;
                let focus_events_on_landing_after_a_middle_row =
                    focus_events_raised_by(hwnd, || {
                        landed_after_a_middle_row =
                            land_the_cursor_after(&list, &[removed], len_after);
                    })?;
                let after_a_middle_row_left = the_cursor_of(&list);

                // Step 2: the cursor is put on the last row, the way arrowing
                // there puts it, and the last row goes.
                let removed = ids.borrow().len() - 1;
                list.set_item_state(
                    removed as i64,
                    ListItemState::Selected | ListItemState::Focused,
                    ListItemState::Selected | ListItemState::Focused,
                );
                let before_the_last_row_left = the_cursor_of(&list);
                ids.borrow_mut().remove(removed);
                let len_after = ids.borrow().len();
                list.set_item_count(len_after as i64);
                list.refresh(true, None);
                let left_by_the_control_after_the_last_row = the_cursor_of(&list);
                let mut landed_after_the_last_row = None;
                let focus_events_on_landing_after_the_last_row =
                    focus_events_raised_by(hwnd, || {
                        landed_after_the_last_row =
                            land_the_cursor_after(&list, &[removed], len_after);
                    })?;
                let after_the_last_row_left = the_cursor_of(&list);

                // Step 3: a re-read, with the cursor's message moved from its
                // row to the last of five, as two messages arriving above it
                // would move it.
                let old_index = after_the_last_row_left.focused.max(0) as usize;
                let cursor = ids.borrow().get(old_index).copied();
                let re_read: Vec<i64> = {
                    let mut rows: Vec<i64> = ids
                        .borrow()
                        .iter()
                        .copied()
                        .filter(|id| Some(*id) != cursor)
                        .collect();
                    rows.insert(0, 60);
                    rows.insert(0, 70);
                    rows.extend(cursor);
                    rows
                };
                *ids.borrow_mut() = re_read.clone();
                list.set_item_count(re_read.len() as i64);
                let mut followed_to = None;
                let focus_events_when_it_moved = focus_events_raised_by(hwnd, || {
                    followed_to =
                        keep_the_cursor_on_its_message(&list, Some(old_index), cursor, &re_read);
                })?;
                let after_a_re_read_that_moved_it = the_cursor_of(&list);

                // Step 4: a re-read that changed nothing under the cursor.
                let old_index = after_a_re_read_that_moved_it.focused.max(0) as usize;
                list.set_item_count(re_read.len() as i64);
                let mut stayed = None;
                let focus_events_when_it_stayed = focus_events_raised_by(hwnd, || {
                    stayed =
                        keep_the_cursor_on_its_message(&list, Some(old_index), cursor, &re_read);
                })?;
                let after_a_re_read_that_left_it = the_cursor_of(&list);

                frame.destroy();
                Ok(Harvest {
                    before,
                    left_by_the_control_after_a_middle_row,
                    after_a_middle_row_left,
                    landed_after_a_middle_row,
                    focus_events_on_landing_after_a_middle_row,
                    before_the_last_row_left,
                    left_by_the_control_after_the_last_row,
                    after_the_last_row_left,
                    landed_after_the_last_row,
                    focus_events_on_landing_after_the_last_row,
                    after_a_re_read_that_moved_it,
                    focus_events_when_it_moved,
                    followed_to,
                    after_a_re_read_that_left_it,
                    focus_events_when_it_stayed,
                    stayed,
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

/// The one harvest of this process, taken by whichever test asks first.
fn the_harvest() -> &'static Harvest {
    static HARVEST: OnceLock<Result<Harvest, String>> = OnceLock::new();
    match HARVEST.get_or_init(take_the_harvest) {
        Ok(harvest) => harvest,
        Err(why) => panic!("the window session could not be read: {why}"),
    }
}

// ── The built list ─────────────────────────────────────────────────────────

#[test]
fn test_the_cursor_was_on_row_2_before_anything_left() {
    // What the rest is measured against: a list whose cursor was nowhere
    // before the delete would make every later reading a reading of nothing.
    let harvest = the_harvest();
    assert_eq!(
        harvest.before,
        Cursor {
            focused: 2,
            selected: 2
        }
    );
}

#[test]
fn test_deleting_a_middle_row_lands_the_cursor_on_the_row_that_followed_it() {
    let harvest = the_harvest();
    assert_eq!(
        harvest.landed_after_a_middle_row,
        Some(2),
        "the helper's answer"
    );
    assert_eq!(
        harvest.after_a_middle_row_left,
        Cursor {
            focused: 2,
            selected: 2
        },
        "after row 2 of five left, the control's focused and selected item; the next \
         message sits on row 2 now"
    );
    // The control held row 2 on its own once its count shrank (the record
    // below), so a landing that only set the state it already held would
    // raise nothing, and a screen reader reads the landed row from the focus
    // event and from nothing else.
    assert!(
        harvest.focus_events_on_landing_after_a_middle_row >= 1,
        "landing after a middle row raised {} focus events, so nothing reads the row the \
         cursor landed on",
        harvest.focus_events_on_landing_after_a_middle_row
    );
}

#[test]
fn test_what_the_control_did_on_its_own_once_its_count_shrank() {
    // The record of the day, 2026-09-18, on a virtual report list: a shrink
    // that leaves the focused index in range keeps the focused and selected
    // item where it was, so after a middle row the control already holds the
    // next message; a shrink that puts the focused index out of range, the
    // last row deleted with the cursor on it, leaves the control holding no
    // focused and no selected item at all, which is what reads as the top of
    // the list. The landing is needed for the second and the focus event for
    // both. A change here is comctl32 behaving differently and is worth
    // reading again before anything above is trusted.
    let harvest = the_harvest();
    assert_eq!(
        harvest.left_by_the_control_after_a_middle_row,
        Cursor {
            focused: 2,
            selected: 2
        },
        "what the control held on its own after a middle row left"
    );
    assert_eq!(
        harvest.left_by_the_control_after_the_last_row,
        Cursor {
            focused: -1,
            selected: -1
        },
        "what the control held on its own after the last row left with the cursor on it"
    );
}

#[test]
fn test_deleting_the_last_row_lands_the_cursor_on_the_row_before_it() {
    let harvest = the_harvest();
    assert_eq!(
        harvest.before_the_last_row_left,
        Cursor {
            focused: 3,
            selected: 3
        },
        "the cursor was to be on the last row of four before it left"
    );
    assert_eq!(
        harvest.landed_after_the_last_row,
        Some(2),
        "the helper's answer"
    );
    assert_eq!(
        harvest.after_the_last_row_left,
        Cursor {
            focused: 2,
            selected: 2
        },
        "after the last row of four left, the control's focused and selected item; the \
         previous message is the new last row"
    );
    assert!(
        harvest.focus_events_on_landing_after_the_last_row >= 1,
        "landing after the last row raised {} focus events, so nothing reads the row the \
         cursor landed on",
        harvest.focus_events_on_landing_after_the_last_row
    );
}

#[test]
fn test_a_re_read_that_moved_the_cursors_message_moves_the_cursor_to_it() {
    let harvest = the_harvest();
    assert_eq!(harvest.followed_to, Some(4), "the helper's answer");
    assert_eq!(
        harvest.after_a_re_read_that_moved_it,
        Cursor {
            focused: 4,
            selected: 4
        },
        "after a re-read put the cursor's message on the last row of five, the control's \
         focused and selected item"
    );
}

#[test]
fn test_a_re_read_that_left_the_cursors_message_where_it_was_raises_no_focus_event() {
    // 10-02's no-move, held on the channel a screen reader hears: nothing
    // is raised, so nothing is read.
    let harvest = the_harvest();
    assert_eq!(harvest.stayed, None, "the helper's answer");
    assert_eq!(
        harvest.focus_events_when_it_stayed, 0,
        "a re-read that changed nothing under the cursor raised focus events"
    );
    assert_eq!(
        harvest.after_a_re_read_that_left_it, harvest.after_a_re_read_that_moved_it,
        "the cursor after a re-read that changed nothing is not where it was"
    );
}

#[test]
fn test_companion_the_move_before_it_raised_a_focus_event_the_hook_could_see() {
    // A count of nought means something only when the same hook counted the
    // move that came before it: the control raises the event when its
    // focused item changes, and the reading saw it.
    let harvest = the_harvest();
    assert!(
        harvest.focus_events_when_it_moved >= 1,
        "the move to row 4 raised {} focus events, so a count of nought on the no-move \
         is a hook that sees nothing",
        harvest.focus_events_when_it_moved
    );
}

// ── The main window's source: the two arms reach the helpers ───────────────
//
// Read from the source rather than run, because the arms are inside the
// window's update handler, which needs a live window, a running event loop
// and a mail store to reach. Each reading is a function over the shipped
// text returning a complaint, and each has a companion that hands it a
// snippet shaped like the site with the fault planted.

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

fn the_main_window() -> String {
    let whole = std::fs::read_to_string(THE_MAIN_WINDOW)
        .unwrap_or_else(|why| panic!("{THE_MAIN_WINDOW}: {why}"))
        .replace("\r\n", "\n");
    wixen_mail::common::what_ships::what_ships(&whole)
}

/// One function's text, from its signature to the closing brace at column
/// nought, or a complaint when the signature is gone.
fn body_of(source: &str, signature: &str) -> Result<String, String> {
    let at = source.find(signature).ok_or(format!(
        "{signature} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let ends = rest.find("\n}\n").map_or(rest.len(), |end| end + 2);
    Ok(rest[..ends].to_string())
}

/// One arm of the update handler, from its label to the next arm's.
fn the_arm_for(source: &str, opens: &str) -> Result<String, String> {
    let at = source.find(opens).ok_or(format!(
        "{opens:?} is no longer here, so this reads nothing"
    ))?;
    let rest = &source[at + opens.len()..];
    let ends = rest.find("\n        UIUpdate::").unwrap_or(rest.len());
    Ok(rest[..ends].to_string())
}

const THE_REMOVAL: &str = "fn take_row_out_of_the_list(";
const THE_LANDING: &str = "pub fn land_the_cursor_after(";
const THE_FOLLOWING: &str = "pub fn keep_the_cursor_on_its_message(";
const THE_CURSOR_SETTER: &str = "fn put_the_cursor_on(";
const THE_LOAD_ARM: &str = "        UIUpdate::MessagesLoaded(messages) => {";

/// A removal lands the cursor on the control: the removal path calls the
/// landing helper, the helper asks the rule and sets the control's focused
/// and selected item, and the setter really sets both and brings the row
/// into view.
fn a_removal_lands_the_cursor_on_the_control(app: &str) -> Result<(), String> {
    let removal = body_of(app, THE_REMOVAL)?;
    if !removal.contains("land_the_cursor_after(") {
        return Err(
            "take_row_out_of_the_list never lands the cursor, so after a delete the state \
             knows the next message and the control reads as the top of the list"
                .to_string(),
        );
    }
    let landing = body_of(app, THE_LANDING)?;
    if !landing.contains("where_to_land(") {
        return Err(
            "land_the_cursor_after does not ask where_to_land, so the row it lands on is \
             its own rule and not the one the cases hold"
                .to_string(),
        );
    }
    if !landing.contains("put_the_cursor_on(") {
        return Err(
            "land_the_cursor_after answers a row and sets nothing on the control, so the \
             state moves and the screen reader's cursor does not"
                .to_string(),
        );
    }
    let setter = body_of(app, THE_CURSOR_SETTER)?;
    for needed in [
        "set_item_state(",
        "ListItemState::Focused",
        "ListItemState::Selected",
        "ensure_visible(",
    ] {
        if !setter.contains(needed) {
            return Err(format!(
                "put_the_cursor_on does not reach {needed}, so the cursor is not put on the \
                 row the way a screen reader follows"
            ));
        }
    }
    Ok(())
}

/// A re-read keeps the cursor on its message by identity, and moves it only
/// when the row changed: the load arm calls the following helper, and the
/// helper asks both rules before touching the control.
fn a_re_read_keeps_the_cursor_on_its_message(app: &str) -> Result<(), String> {
    let arm = the_arm_for(app, THE_LOAD_ARM)?;
    if !arm.contains("keep_the_cursor_on_its_message(") {
        return Err(
            "the MessagesLoaded arm never asks where the cursor's message is now, so a \
             re-read after a delete leaves the cursor on a different message"
                .to_string(),
        );
    }
    if arm.contains("put_the_cursor_on(") {
        return Err(
            "the MessagesLoaded arm sets the cursor itself, beside the helper that decides \
             whether to, so a load can move focus without asking"
                .to_string(),
        );
    }
    let following = body_of(app, THE_FOLLOWING)?;
    if !following.contains("where_the_same_message_is(") {
        return Err(
            "keep_the_cursor_on_its_message does not find the message by its identity, so \
             it follows a row and not a message"
                .to_string(),
        );
    }
    if !following.contains("whether_to_move(") {
        return Err(
            "keep_the_cursor_on_its_message does not ask whether_to_move, so every load \
             re-selects and moves focus, which 10-02 refused"
                .to_string(),
        );
    }
    if !following.contains("put_the_cursor_on(") {
        return Err(
            "keep_the_cursor_on_its_message decides a row and sets nothing on the control"
                .to_string(),
        );
    }
    Ok(())
}

#[test]
fn test_a_removal_lands_the_cursor_on_the_control_through_the_rule() {
    a_removal_lands_the_cursor_on_the_control(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_re_read_keeps_the_cursor_on_its_message_and_moves_it_only_when_its_row_changed() {
    a_re_read_keeps_the_cursor_on_its_message(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

// ── The companions ─────────────────────────────────────────────────────────

/// The sites as they should be, in a snippet.
fn a_window_as_it_should_be() -> String {
    format!(
        "{THE_REMOVAL}state, msg_list, cache_id) {{\n    \
         tell_the_list_how_many(state, msg_list);\n    \
         let landed = land_the_cursor_after(msg_list, &[idx], len_after);\n\
         }}\n\
         {THE_LANDING}list, removed, len_after) -> Option<usize> {{\n    \
         let row = landing_after_a_removal::where_to_land(removed, len_after)?;\n    \
         put_the_cursor_on(list, row);\n    Some(row)\n}}\n\
         {THE_FOLLOWING}list, old_index, cursor, ids_after) -> Option<usize> {{\n    \
         let now = landing_after_a_removal::where_the_same_message_is(cursor, ids_after);\n    \
         let row = landing_after_a_removal::whether_to_move(old_index, now)?;\n    \
         put_the_cursor_on(list, row);\n    Some(row)\n}}\n\
         {THE_CURSOR_SETTER}list, row) {{\n    \
         list.set_item_state(row, ListItemState::Selected | ListItemState::Focused, \
         ListItemState::Selected | ListItemState::Focused);\n    \
         list.ensure_visible(row);\n}}\n\
         fn handle_update() {{\n\
         {THE_LOAD_ARM}\n            \
         let followed = keep_the_cursor_on_its_message(msg_list, old_index, cursor, &ids);\n        \
         UIUpdate::ConversationsLoaded(conversations, why) => {{\n        }}\n}}\n"
    )
}

#[test]
fn test_the_readings_pass_a_window_shaped_as_it_should_be() {
    let app = a_window_as_it_should_be();
    a_removal_lands_the_cursor_on_the_control(&app).unwrap_or_else(|why| panic!("{why}"));
    a_re_read_keeps_the_cursor_on_its_message(&app).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_reading_complains_when_a_removal_moves_the_state_and_not_the_control() {
    let app = a_window_as_it_should_be().replacen(
        "    let landed = land_the_cursor_after(msg_list, &[idx], len_after);\n",
        "    lock_state(state).selected_message_index = Some(idx.min(len_after - 1));\n",
        1,
    );
    let why = a_removal_lands_the_cursor_on_the_control(&app)
        .expect_err("a removal that moved the state alone was passed over");
    assert!(why.contains("never lands the cursor"), "{why}");

    let app = a_window_as_it_should_be().replacen(
        "    put_the_cursor_on(list, row);\n    Some(row)\n}\n",
        "    Some(row)\n}\n",
        1,
    );
    let why = a_removal_lands_the_cursor_on_the_control(&app)
        .expect_err("a landing that set nothing on the control was passed over");
    assert!(why.contains("sets nothing on the control"), "{why}");
}

#[test]
fn test_the_reading_complains_when_a_load_re_selects_without_asking() {
    let app = a_window_as_it_should_be().replacen(
        "    let row = landing_after_a_removal::whether_to_move(old_index, now)?;\n",
        "    let row = now?;\n",
        1,
    );
    let why = a_re_read_keeps_the_cursor_on_its_message(&app)
        .expect_err("a load re-selecting on every load was passed over");
    assert!(why.contains("does not ask whether_to_move"), "{why}");

    let app = a_window_as_it_should_be().replacen(
        "let followed = keep_the_cursor_on_its_message(msg_list, old_index, cursor, &ids);",
        "put_the_cursor_on(msg_list, chosen);",
        1,
    );
    let why = a_re_read_keeps_the_cursor_on_its_message(&app)
        .expect_err("a load arm setting the cursor itself was passed over");
    assert!(
        why.contains("never asks where the cursor's message is"),
        "{why}"
    );
}

// ── The main window's source: a delete says one word, and the outcome is shown ──
//
// #83, the tester on 2026-09-18: a delete said "Deleting <subject>..." on
// the key and "Deleted: <subject>" after the server's round trip, two
// spoken sentences with the subject in each, and the wait for the second
// was a delay in the hand's rhythm. Pratik's decision: say "Delete" and
// nothing more; say something only when the delete did not go through.
// The row the cursor lands on is the confirmation, and the fuller line is
// still written to the status bar for the eye through `UIUpdate::Shown`,
// which is shown and never spoken.

/// The Delete arm of the command dispatch, up to the next arm.
const THE_DELETE_ARM: &str = "_ if id == ID_DELETE || id == ID_DELETE_OUTRIGHT => {";
/// The server's agreed answer to a delete, up to the return that ends it.
const THE_AGREED_CASE: (&str, &str) = ("Deleted::TheServerDidThis(deletion) => {", "return;");
/// The one function that says or shows what the list does next.
const THE_OUTCOME: &str = "fn show_or_say_what_happened_next(";
/// The one word said at the key.
const THE_ONE_WORD: &str = "fn say_the_one_word(";
/// The arm that shows a line and speaks nothing.
const THE_SHOWN_ARM: &str = "        UIUpdate::Shown(shown) => {";

/// The body of one `_ if id == ...` arm of the command dispatch, up to the
/// next arm of the same shape.
fn the_id_arm<'a>(source: &'a str, heading: &str) -> Result<&'a str, String> {
    let start = source.find(heading).ok_or(format!(
        "{heading:?} is no longer here, so this reads nothing"
    ))? + heading.len();
    let rest = &source[start..];
    let end = rest.find("_ if id ==").unwrap_or(rest.len());
    Ok(&rest[..end])
}

/// The text after the first `from` up to the next `to`, or a complaint
/// naming which anchor is gone.
fn between<'a>(text: &'a str, from: &str, to: &str) -> Result<&'a str, String> {
    let start = text
        .find(from)
        .ok_or(format!("{from:?} is no longer here, so this reads nothing"))?
        + from.len();
    let rest = &text[start..];
    let end = rest
        .find(to)
        .ok_or(format!("{to:?} is no longer here, so this reads nothing"))?;
    Ok(&rest[..end])
}

/// A delete says the one word at the key, at Normal, and puts the fuller
/// line on the status bar for the eye: the arm calls the one-word helper
/// with "Delete" and no subject, sends the "Deleting" line as shown, and
/// sends no "Deleting" line on the spoken channel.
fn a_delete_says_the_one_word_at_the_key(app: &str) -> Result<(), String> {
    let arm = the_id_arm(app, THE_DELETE_ARM)?;
    if !arm.contains("say_the_one_word(&a11y, \"Delete\")") {
        return Err(
            "the Delete arm does not say the one word Delete at the key, so what is said on \
             the key is a sentence with the subject in it, or nothing"
                .to_string(),
        );
    }
    if arm.contains("send_status(") && arm.contains("\"Deleting ") {
        return Err(
            "the Delete arm still sends a Deleting line on the spoken channel, so a delete \
             is two spoken sentences again"
                .to_string(),
        );
    }
    if !arm.contains("send_shown(") {
        return Err(
            "the Delete arm no longer writes the Deleting line for the eye, so the status \
             bar says nothing while the server is asked"
                .to_string(),
        );
    }
    let word = body_of(app, THE_ONE_WORD)?;
    if !word.contains("a11y.announce(") || !word.contains("Priority::Normal") {
        return Err(
            "say_the_one_word does not announce at Normal, so the word is either not said \
             or said above the answers to other keys"
                .to_string(),
        );
    }
    Ok(())
}

/// The outcome of a delete or a move is shown when the row left and spoken
/// when it stayed: the server's agreed case hands the answer to the one
/// outcome function, and that function sends the row out with the line as
/// shown, or the line as spoken when the row stays.
fn the_outcome_is_shown_when_the_row_left_and_spoken_when_it_stayed(
    app: &str,
) -> Result<(), String> {
    let agreed = between(app, THE_AGREED_CASE.0, THE_AGREED_CASE.1)?;
    if !agreed.contains("show_or_say_what_happened_next(") {
        return Err(
            "the server's agreed answer to a delete does not go through \
             show_or_say_what_happened_next, so what is said afterwards is decided in the arm"
                .to_string(),
        );
    }
    if agreed.contains("UIUpdate::StatusUpdated(") {
        return Err(
            "the server's agreed answer to a delete still sends the outcome as StatusUpdated, \
             which is spoken, so a delete that went through is announced after the row left"
                .to_string(),
        );
    }
    let outcome = body_of(app, THE_OUTCOME)?;
    let left = between(
        &outcome,
        "ThenWhat::MarkItDeletedHere =>",
        "ThenWhat::LeaveTheRow =>",
    )?;
    if left.contains("UIUpdate::StatusUpdated(") {
        return Err(
            "when the row leaves, the outcome function sends the line as StatusUpdated, so \
             the success is spoken after all"
                .to_string(),
        );
    }
    if !left.contains("UIUpdate::MessageDeletedFromCache(") || !left.contains("UIUpdate::Shown(") {
        return Err(
            "when the row leaves, the outcome function does not take the row out and show the \
             line, so either the row stays or the line for the eye is lost"
                .to_string(),
        );
    }
    let stayed = &outcome[outcome
        .find("ThenWhat::LeaveTheRow =>")
        .ok_or("the outcome function has no arm for a row that stays")?..];
    if !stayed.contains("UIUpdate::StatusUpdated(") {
        return Err(
            "when the row stays, the outcome function does not speak the line, and nothing \
             else tells somebody the message is still where it was"
                .to_string(),
        );
    }
    Ok(())
}

/// The shown arm writes the status bar and the record of it and speaks
/// nothing, and is registered as quiet on purpose with its reason.
fn a_shown_line_is_written_and_never_spoken(app: &str, whole: &str) -> Result<(), String> {
    let arm = between(app, THE_SHOWN_ARM, "\n        UIUpdate::")?;
    if !arm.contains("set_status_text(") || !arm.contains("status_message") {
        return Err(
            "the Shown arm does not write the status bar and the record of it, so the line \
             for the eye is written to nobody"
                .to_string(),
        );
    }
    if arm.contains("a11y.announce") || arm.contains("a11y.signal") {
        return Err(
            "the Shown arm announces or signals, so a line sent as shown is spoken after all"
                .to_string(),
        );
    }
    let quiet = body_of(whole, "fn quiet_on_purpose(")?;
    if !quiet.contains("\"Shown\"") {
        return Err(
            "the Shown arm is not named in quiet_on_purpose, so the check that every arm \
             which shows something says it has no reason for this one"
                .to_string(),
        );
    }
    Ok(())
}

#[test]
fn test_a_delete_says_the_one_word_delete_at_the_key_and_writes_the_fuller_line() {
    a_delete_says_the_one_word_at_the_key(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_outcome_of_a_delete_is_shown_when_the_row_left_and_spoken_when_it_stayed() {
    the_outcome_is_shown_when_the_row_left_and_spoken_when_it_stayed(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_shown_line_is_written_for_the_eye_and_never_spoken() {
    let whole = std::fs::read_to_string(THE_MAIN_WINDOW)
        .unwrap_or_else(|why| panic!("{THE_MAIN_WINDOW}: {why}"))
        .replace("\r\n", "\n");
    a_shown_line_is_written_and_never_spoken(&the_main_window(), &whole)
        .unwrap_or_else(|why| panic!("{why}"));
}

/// The sites as they should be, in a snippet, for the three readings above.
fn a_window_that_says_one_word() -> String {
    format!(
        "{THE_DELETE_ARM}\n    say_the_one_word(&a11y, \"Delete\");\n    \
         send_shown(&ui_tx, &runtime, &format!(\"Deleting {{subject}}...\"));\n\
         _ if id == ID_MARK_READ => {{\n\
         {THE_ONE_WORD}a11y: &Accessibility, word: &str) {{\n    \
         let _ = a11y.announce(word, Priority::Normal);\n}}\n\
         {} \n    show_or_say_what_happened_next(&say, message_row_id, next);\n    {}\n\
         {THE_OUTCOME}say: &impl Fn(UIUpdate), row_id: i64, next: WhatToDoNext) {{\n    \
         match next.then {{\n        ThenWhat::MarkItDeletedHere => {{\n            \
         say(UIUpdate::MessageDeletedFromCache(row_id));\n            \
         say(UIUpdate::Shown(next.said));\n        }}\n        \
         ThenWhat::LeaveTheRow => say(UIUpdate::StatusUpdated(next.said)),\n    }}\n}}\n\
         fn handle_update() {{\n{THE_SHOWN_ARM}\n            \
         lock_state(state).status_message = shown.clone();\n            \
         frame.set_status_text(shown, 0);\n        }}\n        UIUpdate::Progress(said) => {{\n}}\n",
        THE_AGREED_CASE.0, THE_AGREED_CASE.1,
    )
}

/// The test module's register, as the whole file would carry it.
fn a_register_naming_shown() -> String {
    "fn quiet_on_purpose() -> &'static [(&'static str, &'static str)] {\n    &[(\"Shown\", \
     \"a line the eye may want and the ear has already had\")]\n}\n"
        .to_string()
}

#[test]
fn test_the_one_word_readings_pass_a_window_shaped_as_it_should_be() {
    let app = a_window_that_says_one_word();
    a_delete_says_the_one_word_at_the_key(&app).unwrap_or_else(|why| panic!("{why}"));
    the_outcome_is_shown_when_the_row_left_and_spoken_when_it_stayed(&app)
        .unwrap_or_else(|why| panic!("{why}"));
    a_shown_line_is_written_and_never_spoken(&app, &a_register_naming_shown())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_reading_complains_when_the_subject_is_back_in_the_spoken_word() {
    let app = a_window_that_says_one_word().replacen(
        "say_the_one_word(&a11y, \"Delete\");",
        "send_status(&ui_tx, &runtime, &format!(\"Deleting {subject}...\"));",
        1,
    );
    let why = a_delete_says_the_one_word_at_the_key(&app)
        .expect_err("a delete saying the subject at the key was passed over");
    assert!(why.contains("does not say the one word"), "{why}");
}

#[test]
fn test_the_reading_complains_when_the_success_is_spoken_again() {
    let app = a_window_that_says_one_word().replacen(
        "say(UIUpdate::Shown(next.said));",
        "say(UIUpdate::StatusUpdated(next.said));",
        1,
    );
    let why = the_outcome_is_shown_when_the_row_left_and_spoken_when_it_stayed(&app)
        .expect_err("a success sent as StatusUpdated was passed over");
    assert!(why.contains("spoken after all"), "{why}");

    let app = a_window_that_says_one_word().replacen(
        "ThenWhat::LeaveTheRow => say(UIUpdate::StatusUpdated(next.said)),",
        "ThenWhat::LeaveTheRow => say(UIUpdate::Shown(next.said)),",
        1,
    );
    let why = the_outcome_is_shown_when_the_row_left_and_spoken_when_it_stayed(&app)
        .expect_err("a refusal-shaped outcome shown and not spoken was passed over");
    assert!(why.contains("does not speak the line"), "{why}");
}

#[test]
fn test_the_reading_complains_when_the_shown_arm_speaks_or_is_unregistered() {
    let app = a_window_that_says_one_word().replacen(
        "            frame.set_status_text(shown, 0);\n",
        "            frame.set_status_text(shown, 0);\n            let _ = a11y.announce(shown, Priority::Normal);\n",
        1,
    );
    let why = a_shown_line_is_written_and_never_spoken(&app, &a_register_naming_shown())
        .expect_err("a Shown arm that announces was passed over");
    assert!(why.contains("spoken after all"), "{why}");

    let register = a_register_naming_shown().replacen("\"Shown\"", "\"OutboxQueueCount\"", 1);
    let why = a_shown_line_is_written_and_never_spoken(&a_window_that_says_one_word(), &register)
        .expect_err("a Shown arm nobody registered was passed over");
    assert!(why.contains("not named in quiet_on_purpose"), "{why}");
}

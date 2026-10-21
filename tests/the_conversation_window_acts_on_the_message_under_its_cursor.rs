//! The conversation window acts on the message under its cursor (13.2-03).
//!
//! Pratik, 2026-10-14: "When the thread is opened, user is unable to use
//! context menu or other commands that can act on a message within the
//! thread. Actions can only be taken on an entire conversation upon
//! returning to the mailbox." The window was a picker that answered only
//! which reading to open; nothing in it answered the menu key, Delete or M.
//!
//! Three halves, each read where it can be read:
//!
//! - **The window**, built for real and shown modally on a desktop made for
//!   this run, with the key or the menu line posted to the tree the way the
//!   message loop delivers it, and a Cancel posted after it so a case nothing
//!   answers ends as Cancelled and fails rather than waiting.
//! - **The page**: every key the tree answers is written under
//!   "### Conversations" in `docs/KEYBOARD_SHORTCUTS.md`, read from the
//!   tree's own list of its keys.
//! - **The loop** in `wx_app.rs` that runs the command and brings the
//!   conversation back, which needs the whole main window and so is read
//!   from its source, with a planted fault beside each reading.
//!
//! One window session, behind a `OnceLock`: `cargo test` gives this file a
//! process of its own, and a second `wxdragon::main` in one process hangs.

use std::fs;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::context_menu::Action;
use wixen_mail::application::conversations::{AConversationReaches, WhoseConversation};
use wixen_mail::application::{quick_steps, tagging};
use wixen_mail::common::types::FolderType;
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::data::message_cache::{CachedFolder, IncomingMessage, MessageCache};
use wixen_mail::presentation::tree_walk;
use wixen_mail::presentation::ui_types::MessageItem;
use wixen_mail::presentation::view_state::Showing;
use wixen_mail::presentation::wx_app::{WxUIState, apply_threading, label_id_at, quick_step_id_at};
use wixen_mail::presentation::wx_context_menu::command_for;
use wixen_mail::presentation::wx_thread_view::{self, ThreadChoice, ThreadNode};
use wixen_mail::service::safety::Verdict;
use wxdragon::prelude::*;

/// The main window's commands, by the ids the message menu raises for them,
/// so the menu line posted below is the one the menu key's popup delivers.
const ID_DELETE: i32 = command_for(Action::DeleteMessage);
const ID_DELETE_OUTRIGHT: i32 = command_for(Action::DeleteMessageOutright);
const ID_MARK_READ: i32 = command_for(Action::MarkRead);
const ID_REPLY: i32 = command_for(Action::Reply);
const ID_REPLY_ALL: i32 = command_for(Action::ReplyAll);
const ID_FORWARD: i32 = command_for(Action::Forward);
const ID_TOGGLE_STAR: i32 = command_for(Action::ToggleStar);
const ID_MOVE_TO_FOLDER: i32 = command_for(Action::MoveToFolder);

// ── The window, driven by posted messages ──────────────────────────────────

const WM_KEYDOWN: u32 = 0x0100;
/// A key pressed with Alt held arrives as a system key.
const WM_SYSKEYDOWN: u32 = 0x0104;
const WM_COMMAND: u32 = 0x0111;
const VK_RETURN: usize = 0x0D;
const VK_DELETE: usize = 0x2E;
const VK_M: usize = 0x4D;
const VK_R: usize = 0x52;
const VK_V: usize = 0x56;
const VK_1: usize = 0x31;
const VK_7: usize = 0x37;
const VK_SHIFT: usize = 0x10;
const VK_LSHIFT: usize = 0xA0;
const VK_CONTROL: usize = 0x11;
const VK_LCONTROL: usize = 0xA2;
const VK_MENU: usize = 0x12;
const VK_LMENU: usize = 0xA4;
const PM_REMOVE: u32 = 0x0001;

/// The context-code bit of a system key-down's `lParam`, set when Alt is
/// held.
const ALT_IS_DOWN: isize = 1 << 29;
const GENERIC_ALL: u32 = 0x1000_0000;

/// A key-down's `lParam`: pressed once.
const KEY_DOWN_ONCE: isize = 1;

/// The extended-key bit of a key-down's `lParam`, which the dedicated Delete
/// key sets and the number pad's Del does not; wxWidgets names the first
/// `WXK_DELETE` and the second `WXK_NUMPAD_DELETE`. Measured 2026-10-14: a
/// posted `VK_DELETE` without it arrived as 385.
fn extended(key: usize) -> isize {
    if key == VK_DELETE { 1 << 24 } else { 0 }
}

/// winuser.h: Shift, Control and Alt, each with its left and right key.
const MODIFIERS: [usize; 9] = [0x10, 0x11, 0x12, 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5];

/// One `MSG`, as `PeekMessageW` fills it, only so leftovers can be taken out.
#[repr(C)]
struct Msg {
    hwnd: isize,
    message: u32,
    wparam: usize,
    lparam: isize,
    time: u32,
    x: i32,
    y: i32,
    private: u32,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn PostMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> i32;
    fn PeekMessageW(msg: *mut Msg, hwnd: isize, first: u32, last: u32, remove: u32) -> i32;
    fn CreateDesktopW(
        name: *const u16,
        device: *const u16,
        mode: *const u8,
        flags: u32,
        access: u32,
        attributes: *const u8,
    ) -> isize;
    fn SetThreadDesktop(desktop: isize) -> i32;
    fn GetKeyboardState(state: *mut u8) -> i32;
    fn SetKeyboardState(state: *const u8) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetLastError() -> u32;
}

/// Move the calling thread onto a desktop made for this run, before its
/// first window, so nothing a person types or holds reaches the window and
/// no second run shares it. The pattern `tests/a_move_completes_here_first.rs`
/// measured on 2026-10-02.
fn a_desktop_of_its_own() -> Result<(), String> {
    let name: Vec<u16> = format!("wixen-conversation-keys-{}", std::process::id())
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    // SAFETY: the name is null-terminated and every other pointer is null,
    // which CreateDesktopW takes as "none".
    unsafe {
        let desktop = CreateDesktopW(
            name.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            0,
            GENERIC_ALL,
            std::ptr::null(),
        );
        if desktop == 0 {
            return Err(format!("CreateDesktopW failed: {}", GetLastError()));
        }
        if SetThreadDesktop(desktop) == 0 {
            return Err(format!("SetThreadDesktop failed: {}", GetLastError()));
        }
    }
    Ok(())
}

/// This thread's keyboard state with exactly `down` of the modifiers held,
/// which is where wxWidgets reads a key's modifiers from.
fn only_these_modifiers_down(down: &[usize]) {
    let mut held = [0u8; 256];
    // SAFETY: the buffer is the 256 bytes the call writes.
    unsafe { GetKeyboardState(held.as_mut_ptr()) };
    for key in MODIFIERS {
        held[key] = 0;
    }
    for &key in down {
        held[key] = 0x80;
    }
    // SAFETY: the buffer is the 256 bytes the call reads, for this thread.
    unsafe { SetKeyboardState(held.as_ptr()) };
}

/// Messages 11, 22 and 33, where 33 replies to 11, so the tree walks them
/// 11, 33, 22: the order `tests/tree_dialogs_resolve_the_row_somebody_is_on.rs`
/// builds.
///
/// 22 is read and from Mary Somerville, so its row is the one row whose
/// label starts with M: a letter the tree's own type-ahead received would
/// move the cursor there, which is what the M case can see.
fn nodes() -> Vec<ThreadNode> {
    let message = |message_id: i64, parent: Option<usize>| ThreadNode {
        message_id,
        uid: message_id as u32,
        sender: "Ada Lovelace".to_string(),
        subject: "Quarterly report".to_string(),
        date: "2026-07-26".to_string(),
        read: false,
        depth: parent.map_or(0, |_| 1),
        parent,
    };
    let mary = ThreadNode {
        sender: "Mary Somerville".to_string(),
        read: true,
        ..message(22, None)
    };
    vec![message(11, None), mary, message(33, Some(0))]
}

/// Where the cursor starts, before anything is posted.
#[derive(Clone, Copy)]
enum StartOn {
    /// The message the list row stood for, 33.
    RowMessage,
    /// The row for 22, chosen after the dialog is built.
    SecondMessage,
    /// The root, the whole conversation.
    Root,
}

/// Which modifiers a posted key is held with.
#[derive(Clone, Copy)]
enum Held {
    Nothing,
    Shift,
    Ctrl,
    CtrlShift,
    AltShift,
}

impl Held {
    /// The virtual keys held down, each with its left key.
    fn keys(self) -> &'static [usize] {
        match self {
            Held::Nothing => &[],
            Held::Shift => &[VK_SHIFT, VK_LSHIFT],
            Held::Ctrl => &[VK_CONTROL, VK_LCONTROL],
            Held::CtrlShift => &[VK_CONTROL, VK_LCONTROL, VK_SHIFT, VK_LSHIFT],
            Held::AltShift => &[VK_MENU, VK_LMENU, VK_SHIFT, VK_LSHIFT],
        }
    }

    /// The message a key held this way arrives as, and its `lParam` bits.
    fn arrives_as(self) -> (u32, isize) {
        match self {
            Held::AltShift => (WM_SYSKEYDOWN, ALT_IS_DOWN),
            _ => (WM_KEYDOWN, 0),
        }
    }
}

/// What is posted, and to which window.
#[derive(Clone, Copy)]
enum Posted {
    Nothing,
    /// A key-down to the tree, with these modifiers held.
    AKey {
        key: usize,
        held: Held,
    },
    /// A menu line to the tree, where the menu key's popup delivers it.
    AMenuLine(i32),
}

/// What one case found: the choice before anything was pressed, what the
/// dialog answered, and which message the tree's cursor was on afterwards.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Found {
    before: ThreadChoice,
    answered: ThreadChoice,
    cursor_on: Option<i64>,
}

fn one_case(frame: &Frame, start: StartOn, posted: Posted) -> Result<Found, String> {
    let nodes = nodes();
    let (dialog, tree, chosen) =
        wx_thread_view::build_thread_dialog(frame, "Quarterly report", &nodes, Some(33), None)
            .ok_or("three nodes give the conversation tree a root")?;
    let rows = tree_walk::rows_in_walk_order(&tree);
    let walked: Vec<i64> = vec![11, 33, 22];
    match start {
        StartOn::RowMessage => {}
        StartOn::SecondMessage => tree.select_item(&rows[2]),
        StartOn::Root => {
            let root = tree.get_root_item().ok_or("the tree has no root")?;
            tree.select_item(&root);
        }
    }
    let before = chosen.borrow().clone();
    let tree_window = tree.get_handle() as isize;
    let dialog_window = dialog.get_handle() as isize;
    only_these_modifiers_down(&[]);
    // Posted from inside the modal loop, once the dialog is on screen and
    // the tree can take the focus: a key posted before it is shown reaches
    // a dialog with nothing focused, and the dialog manager keeps Enter.
    // The Cancel goes after it, so a case nothing answers ends as Cancelled
    // and fails rather than waiting.
    wxdragon::call_after(Box::new(move || {
        tree.set_focus();
        // SAFETY: two live windows this case built, each posted a message
        // the message loop delivers as it would a key or a menu line.
        unsafe {
            match posted {
                Posted::Nothing => {}
                Posted::AKey { key, held } => {
                    only_these_modifiers_down(held.keys());
                    let (message, bits) = held.arrives_as();
                    PostMessageW(
                        tree_window,
                        message,
                        key,
                        KEY_DOWN_ONCE | extended(key) | bits,
                    );
                }
                Posted::AMenuLine(command) => {
                    PostMessageW(tree_window, WM_COMMAND, command as u16 as usize, 0);
                }
            }
            PostMessageW(dialog_window, WM_COMMAND, ID_CANCEL as u16 as usize, 0);
        }
    }));
    let code = dialog.show_modal();
    only_these_modifiers_down(&[]);
    // Whatever the loop left behind is taken out, so it cannot reach the next
    // case's dialog.
    let mut left = Msg {
        hwnd: 0,
        message: 0,
        wparam: 0,
        lparam: 0,
        time: 0,
        x: 0,
        y: 0,
        private: 0,
    };
    // SAFETY: the buffer is one MSG, and the calls only remove messages
    // addressed to the two windows this case built.
    unsafe {
        while PeekMessageW(&mut left, dialog_window, 0, 0, PM_REMOVE) != 0 {}
        while PeekMessageW(&mut left, tree_window, 0, 0, PM_REMOVE) != 0 {}
    }
    let answered = wx_thread_view::what_the_dialog_answered(code, &chosen.borrow());
    let cursor_on =
        tree_walk::where_the_selection_sits(&tree).and_then(|at| walked.get(at).copied());
    dialog.destroy();
    Ok(Found {
        before,
        answered,
        cursor_on,
    })
}

/// Every case's finding, by name, read once in one window session.
struct Harvest {
    the_choice_on_arrival: Found,
    enter_on_the_root: Found,
    delete_on_the_root: Found,
    delete_on_a_message: Found,
    shift_delete_on_a_message: Found,
    m_on_a_message: Found,
    a_menu_line_on_a_message: Found,
    a_menu_line_on_another_message: Found,
    ctrl_r_on_a_message: Found,
    ctrl_shift_v_on_a_message: Found,
    ctrl_1_on_a_message: Found,
    ctrl_shift_7_on_a_message: Found,
    alt_shift_r_on_a_message: Found,
}

fn take_the_harvest() -> Result<Harvest, String> {
    a_desktop_of_its_own()?;
    // SAFETY: set before the window session starts any thread, and nothing
    // has read it yet.
    unsafe {
        std::env::set_var("WIXEN_NO_AUDIO", "1");
    }
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let taken: Result<Harvest, String> = (|| {
                // Never shown: the dialogs are shown modally on this run's
                // own desktop, where nobody sees them.
                let frame = Frame::builder().build();
                let key = |key: usize| Posted::AKey {
                    key,
                    held: Held::Nothing,
                };
                let held = |key: usize, held: Held| Posted::AKey { key, held };
                let on_a_message = |posted: Posted| one_case(&frame, StartOn::RowMessage, posted);
                let harvest = Harvest {
                    the_choice_on_arrival: one_case(&frame, StartOn::RowMessage, Posted::Nothing)?,
                    enter_on_the_root: one_case(&frame, StartOn::Root, key(VK_RETURN))?,
                    delete_on_the_root: one_case(&frame, StartOn::Root, key(VK_DELETE))?,
                    delete_on_a_message: on_a_message(key(VK_DELETE))?,
                    shift_delete_on_a_message: on_a_message(held(VK_DELETE, Held::Shift))?,
                    m_on_a_message: on_a_message(key(VK_M))?,
                    ctrl_r_on_a_message: on_a_message(held(VK_R, Held::Ctrl))?,
                    ctrl_shift_v_on_a_message: on_a_message(held(VK_V, Held::CtrlShift))?,
                    ctrl_1_on_a_message: on_a_message(held(VK_1, Held::Ctrl))?,
                    ctrl_shift_7_on_a_message: on_a_message(held(VK_7, Held::CtrlShift))?,
                    alt_shift_r_on_a_message: on_a_message(held(VK_R, Held::AltShift))?,
                    a_menu_line_on_a_message: one_case(
                        &frame,
                        StartOn::RowMessage,
                        Posted::AMenuLine(ID_MARK_READ),
                    )?,
                    a_menu_line_on_another_message: one_case(
                        &frame,
                        StartOn::SecondMessage,
                        Posted::AMenuLine(ID_MARK_READ),
                    )?,
                };
                frame.destroy();
                Ok(harvest)
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

fn the_harvest() -> &'static Harvest {
    static TAKEN: OnceLock<Result<Harvest, String>> = OnceLock::new();
    match TAKEN.get_or_init(take_the_harvest) {
        Ok(taken) => taken,
        Err(why) => panic!("the conversation window could not be read: {why}"),
    }
}

fn acts(command: i32, message: i64) -> ThreadChoice {
    ThreadChoice::Act { command, message }
}

#[test]
fn test_the_window_opens_on_the_message_the_row_stood_for() {
    let found = &the_harvest().the_choice_on_arrival;
    assert_eq!(found.before, ThreadChoice::Message(33));
    assert_eq!(
        found.answered,
        ThreadChoice::Cancelled,
        "the posted Cancel is what ends a case nothing else answers"
    );
}

/// Also the case that found a second ending overriding the first: the
/// Cancel posted behind Enter reached the dialog after Enter had ended it,
/// and the answer read Cancelled. A person pressing Escape just after Enter
/// met the same. The dialog now ends once and keeps its first answer.
#[test]
fn test_enter_on_the_root_still_opens_the_whole_conversation() {
    let found = &the_harvest().enter_on_the_root;
    assert_eq!(found.before, ThreadChoice::AsHeadings);
    assert_eq!(found.answered, ThreadChoice::AsHeadings);
}

#[test]
fn test_delete_on_the_root_does_nothing() {
    assert_eq!(
        the_harvest().delete_on_the_root.answered,
        ThreadChoice::Cancelled,
        "the root is the whole conversation, and the message keys are a message's"
    );
}

#[test]
fn test_delete_on_a_message_acts_on_that_message() {
    assert_eq!(
        the_harvest().delete_on_a_message.answered,
        acts(ID_DELETE, 33)
    );
}

#[test]
fn test_shift_delete_on_a_message_deletes_it_permanently() {
    assert_eq!(
        the_harvest().shift_delete_on_a_message.answered,
        acts(ID_DELETE_OUTRIGHT, 33)
    );
}

#[test]
fn test_m_on_a_message_marks_it_and_the_type_ahead_never_sees_the_letter() {
    let found = &the_harvest().m_on_a_message;
    assert_eq!(found.answered, acts(ID_MARK_READ, 33));
    assert_eq!(
        found.cursor_on,
        Some(33),
        "a letter the tree's own search received would move the cursor off the message"
    );
}

#[test]
fn test_a_line_from_the_menu_acts_on_the_message_under_the_cursor() {
    assert_eq!(
        the_harvest().a_menu_line_on_a_message.answered,
        acts(ID_MARK_READ, 33)
    );
}

#[test]
fn test_a_line_from_the_menu_acts_on_whichever_message_the_cursor_is_on() {
    let found = &the_harvest().a_menu_line_on_another_message;
    assert_eq!(found.before, ThreadChoice::Message(22));
    assert_eq!(
        found.answered,
        acts(ID_MARK_READ, 22),
        "a window answering a constant message would name 33 here"
    );
}

// ── The rest of the message keys, 13.2-03.1 ────────────────────────────────
//
// Pratik's "or other commands": each key the message list has for a message
// command, answered on the message under the cursor. A key nothing answers
// ends the case as Cancelled.

/// The command the tree's list says a key, written as the page writes it,
/// runs; a complaint when the list holds no such key.
fn the_trees_command_for(written: &str) -> i32 {
    wx_thread_view::THE_TREES_KEYS
        .iter()
        .find(|key| key.written == written)
        .map(|key| key.command)
        .unwrap_or_else(|| panic!("the tree's list holds no key written {written:?}"))
}

#[test]
fn test_ctrl_r_on_a_message_replies_to_that_message() {
    assert_eq!(
        the_harvest().ctrl_r_on_a_message.answered,
        acts(ID_REPLY, 33)
    );
}

#[test]
fn test_ctrl_shift_v_on_a_message_moves_that_message() {
    assert_eq!(
        the_harvest().ctrl_shift_v_on_a_message.answered,
        acts(ID_MOVE_TO_FOLDER, 33)
    );
}

#[test]
fn test_ctrl_1_on_a_message_puts_the_first_label_on_that_message() {
    let first_label = label_id_at(1).expect("the first label has an id");
    assert_eq!(
        the_harvest().ctrl_1_on_a_message.answered,
        acts(first_label, 33)
    );
}

#[test]
fn test_ctrl_shift_7_on_a_message_runs_the_first_quick_step_on_that_message() {
    let first_step = quick_step_id_at(1).expect("the first Quick Step has an id");
    assert_eq!(
        the_harvest().ctrl_shift_7_on_a_message.answered,
        acts(first_step, 33)
    );
}

/// Alt held makes the key a system key, which the tree reads as it reads
/// Shift. Which command it runs is the comparison's below, which holds the
/// list's line for it to the main window's Reply to Sender Only.
#[test]
fn test_alt_shift_r_on_a_message_replies_to_its_sender_alone() {
    assert_eq!(
        the_harvest().alt_shift_r_on_a_message.answered,
        acts(the_trees_command_for("Alt+Shift+R"), 33)
    );
}

// ── The keys are the main window's ─────────────────────────────────────────
//
// Each key the tree answers is the key the main window gives the same
// command. The menu labels stay literals, because `tests/wired.rs` finds a
// bound key in their source text, so the tree's list is compared with them
// rather than read from them: its lines in the tree's source are paired by
// the command's name with the main window's label for that command.

const THE_THREAD_VIEW: &str = "src/presentation/wx_thread_view.rs";
/// The tree's list of message keys, one `("written", ID_NAME),` per line.
const THE_LIST_OF_KEYS: &str = "const THE_MESSAGE_KEYS:";
/// The keys the message list binds itself rather than through the menu,
/// with the line that binds each.
const THE_LISTS_OWN_LETTERS: [(&str, &str); 1] = [("M", "wire_letter(&msg_list, 'M'")];

/// A key as written, read as which modifiers are held and which key: the
/// page's `Delete` and the menu's `Del` are one key. `None` for a spelling
/// this does not know.
fn read_as_pressed(written: &str) -> Option<(bool, bool, bool, String)> {
    let mut parts = written.split('+');
    let key = parts.next_back()?;
    let (mut control, mut alt, mut shift) = (false, false, false);
    for modifier in parts {
        match modifier {
            "Ctrl" => control = true,
            "Alt" => alt = true,
            "Shift" => shift = true,
            _ => return None,
        }
    }
    let key = match key {
        "Del" | "Delete" => "Delete".to_string(),
        one if one.chars().count() == 1 => one.to_ascii_uppercase(),
        _ => return None,
    };
    Some((control, alt, shift, key))
}

/// wxWidgets' code for a key as `read_as_pressed` names it.
fn the_code_of(key: &str) -> Option<i32> {
    match key {
        "Delete" => Some(127),
        one => one.bytes().next().map(i32::from),
    }
}

/// The tree's list of message keys as its source writes them, as
/// `(written, command name)`, or a complaint when the list is not there.
fn the_trees_lines(view: &str) -> Result<Vec<(String, String)>, String> {
    let listed = between(view, THE_LIST_OF_KEYS, "];").map_err(|_| {
        format!("{THE_THREAD_VIEW} has no {THE_LIST_OF_KEYS}, so this compares nothing")
    })?;
    let lines: Vec<(String, String)> = listed
        .lines()
        .filter_map(|line| {
            let line = line.trim().strip_prefix("(\"")?;
            let (written, command) = line.split_once("\", ")?;
            Some((
                written.to_string(),
                command.trim_end_matches("),").to_string(),
            ))
        })
        .collect();
    if lines.is_empty() {
        return Err(format!("{THE_LIST_OF_KEYS} lists nothing"));
    }
    Ok(lines)
}

/// The key the main window's menu writes after the tab in the label of
/// `command`, by its name, when it writes one.
fn the_menus_key_for(app: &str, command: &str) -> Option<String> {
    let named = format!("{command},");
    app.match_indices(&named).find_map(|(at, _)| {
        let whole_name = !app[..at]
            .chars()
            .next_back()
            .is_some_and(|before| before.is_alphanumeric() || before == '_');
        let label = app[at + named.len()..]
            .trim_start()
            .strip_prefix('"')
            .filter(|_| whole_name)?;
        let label = &label[..label.find('"')?];
        label.split_once("\\t").map(|(_, key)| key.to_string())
    })
}

/// Every line of the tree's list whose key is not the one the main window
/// gives its command, each in a sentence.
fn the_keys_the_menu_disagrees_with(lines: &[(String, String)], app: &str) -> Vec<String> {
    lines
        .iter()
        .filter_map(|(written, command)| {
            let ours = read_as_pressed(written);
            match the_menus_key_for(app, command) {
                Some(menu) if ours.is_some() && read_as_pressed(&menu) == ours => None,
                Some(menu) => Some(format!(
                    "{written} runs {command}, which the main window gives {menu}"
                )),
                None => {
                    let bound_by_the_list = THE_LISTS_OWN_LETTERS
                        .iter()
                        .any(|(letter, binding)| letter == written && app.contains(binding));
                    (!bound_by_the_list).then(|| {
                        format!(
                            "{written} runs {command}, which has no key on the main window's \
                             menu and is not a letter the list binds itself"
                        )
                    })
                }
            }
        })
        .collect()
}

#[test]
fn test_every_key_the_tree_answers_is_the_key_the_main_window_gives_its_command() {
    let view = fs::read_to_string(THE_THREAD_VIEW)
        .unwrap_or_else(|why| panic!("{THE_THREAD_VIEW}: {why}"))
        .replace("\r\n", "\n");
    let lines = the_trees_lines(&what_ships(&view)).unwrap_or_else(|why| panic!("{why}"));
    let differ = the_keys_the_menu_disagrees_with(&lines, &the_main_window());
    assert!(differ.is_empty(), "{differ:#?}");

    // Each line is in the list the handler reads, and the key it names
    // finds it there.
    let numbered = (1..=tagging::REACHABLE_BY_KEY)
        .filter_map(|place| Some((tagging::key_for(place)?, label_id_at(place)?)))
        .chain(
            (1..=quick_steps::REACHABLE_BY_KEY)
                .filter_map(|place| Some((quick_steps::key_for(place)?, quick_step_id_at(place)?))),
        );
    let lines = lines.iter().map(|(written, _)| (written.clone(), None));
    for (written, command) in lines.chain(numbered.map(|(written, id)| (written, Some(id)))) {
        let (control, alt, shift, key) = read_as_pressed(&written)
            .unwrap_or_else(|| panic!("{written:?} is not a key this can read"));
        let found = wx_thread_view::the_key_pressed(the_code_of(&key), control, alt, shift)
            .unwrap_or_else(|| panic!("{written} finds nothing in the tree's list"));
        assert_eq!(found.written, written, "{written} finds another key's line");
        if let Some(command) = command {
            assert_eq!(
                found.command, command,
                "{written} runs another command than the main window's"
            );
        }
    }
}

#[test]
fn test_the_comparison_refuses_a_key_the_main_window_does_not_give_its_command() {
    let app = "append_item(ID_REPLY, \"&Reply\\tCtrl+R\", \"\")\n\
               append_item(ID_REPLY_ALL, \"Reply &All\\tCtrl+Shift+R\", \"\")\n\
               append_item(\n    ID_DELETE,\n    \"&Delete\\tDel\",\n)\n\
               append_item(ID_MARK_READ, \"Mark as R&ead\", \"\")\n\
               wire_letter(&msg_list, 'M', {})\n";
    let line = |written: &str, command: &str| (written.to_string(), command.to_string());
    let right = [
        line("Ctrl+R", "ID_REPLY"),
        line("Ctrl+Shift+R", "ID_REPLY_ALL"),
        line("Delete", "ID_DELETE"),
        line("M", "ID_MARK_READ"),
    ];
    assert!(the_keys_the_menu_disagrees_with(&right, app).is_empty());
    for (planted, said) in [
        (
            line("Ctrl+K", "ID_REPLY"),
            "which the main window gives Ctrl+R",
        ),
        (line("Ctrl+R", "ID_REPLY_ALL"), "gives Ctrl+Shift+R"),
        (line("Shift+Del", "ID_DELETE"), "gives Del"),
        (
            line("N", "ID_MARK_READ"),
            "not a letter the list binds itself",
        ),
    ] {
        let differ = the_keys_the_menu_disagrees_with(std::slice::from_ref(&planted), app);
        assert!(
            differ.len() == 1 && differ[0].contains(said),
            "{planted:?}: {differ:?}"
        );
    }
}

/// Reply, Reply All, Reply to Sender Only and Forward open a window of their
/// own, so the conversation does not come back over it; every other key the
/// tree answers acts on the message and brings the conversation back.
#[test]
fn test_only_the_keys_that_open_a_window_leave_the_conversation_closed() {
    let opens_a_window: Vec<i32> = ["Ctrl+R", "Ctrl+Shift+R", "Alt+Shift+R", "Ctrl+L"]
        .into_iter()
        .map(the_trees_command_for)
        .collect();
    for command in &opens_a_window {
        assert!(
            !wx_thread_view::brings_the_conversation_back(*command),
            "{command} would bring the conversation back over the window it opens"
        );
    }
    for key in wx_thread_view::THE_TREES_KEYS.iter() {
        if !opens_a_window.contains(&key.command) {
            assert!(
                wx_thread_view::brings_the_conversation_back(key.command),
                "{} leaves the conversation closed",
                key.written
            );
        }
    }
    for command in [ID_REPLY, ID_REPLY_ALL, ID_FORWARD] {
        assert!(!wx_thread_view::brings_the_conversation_back(command));
    }
    for command in [ID_TOGGLE_STAR, ID_MOVE_TO_FOLDER] {
        assert!(wx_thread_view::brings_the_conversation_back(command));
    }
    assert_eq!(
        wx_thread_view::THE_TREES_KEYS.len(),
        3 + 8 + tagging::REACHABLE_BY_KEY + quick_steps::REACHABLE_BY_KEY,
        "13.2-03's three, the eight with their own menu keys, the labels and the Quick Steps"
    );
}

// ── The page ───────────────────────────────────────────────────────────────

const THE_PAGE: &str = "docs/KEYBOARD_SHORTCUTS.md";
const THE_SECTION: &str = "### Conversations";
/// The menu key, as the page writes it everywhere else.
const THE_MENU_KEYS: [&str; 2] = ["Applications", "Shift+F10"];

/// The part of the page under one heading, up to the next heading at the
/// same level or above it.
fn under(page: &str, heading: &str) -> String {
    let level = heading.chars().take_while(|letter| *letter == '#').count();
    let mut reading = false;
    let mut kept = Vec::new();
    for line in page.lines() {
        if line.trim_end() == heading {
            reading = true;
            continue;
        }
        if !reading {
            continue;
        }
        let here = line.chars().take_while(|letter| *letter == '#').count();
        if here > 0 && here <= level && line[here..].starts_with(' ') {
            break;
        }
        kept.push(line);
    }
    kept.join("\n")
}

fn the_keys_the_section_leaves_out(section: &str) -> Vec<String> {
    wx_thread_view::THE_TREES_KEYS
        .iter()
        .map(|key| key.written.as_str())
        .chain(THE_MENU_KEYS)
        .filter(|written| !section.contains(&format!("`{written}`")))
        .map(str::to_string)
        .collect()
}

#[test]
fn test_every_key_the_tree_answers_is_written_where_the_page_describes_the_window() {
    let page = fs::read_to_string(THE_PAGE).expect("the keyboard page");
    let section = under(&page, THE_SECTION);
    assert!(
        !section.is_empty(),
        "{THE_PAGE} has no {THE_SECTION:?} section"
    );
    let missing = the_keys_the_section_leaves_out(&section);
    assert!(
        missing.is_empty(),
        "{THE_SECTION} does not name {missing:?}, so somebody reading the page cannot learn \
         what the conversation window answers"
    );
}

#[test]
fn test_the_page_reading_sees_a_key_left_out() {
    let all: String = wx_thread_view::THE_TREES_KEYS
        .iter()
        .map(|key| format!("`{}` ", key.written))
        .chain(THE_MENU_KEYS.iter().map(|key| format!("`{key}` ")))
        .collect();
    assert!(the_keys_the_section_leaves_out(&all).is_empty());
    let without_m = all.replacen("`M` ", "", 1);
    assert_eq!(the_keys_the_section_leaves_out(&without_m), vec!["M"]);
}

// ── The loop ───────────────────────────────────────────────────────────────

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";
const THE_LOOP: &str = "fn open_conversation_again(";
const THE_ACT_ARM: &str = "wx_thread_view::ThreadChoice::Act {";
const THE_NEXT_ARM: &str = "wx_thread_view::ThreadChoice::";
const THE_GUARD: &str = "ActingOn::begin(";
const THE_GUARD_SETS: (&str, &str) = ("impl ActingOn", "acting_on = Some(");
const THE_GUARD_CLEARS: (&str, &str) = ("impl Drop for ActingOn", "acting_on = None");
const THE_CELL: &str = "runs_a_command.borrow()";
const THE_REBUILD: &str = "conversation_nodes(";
const THE_RULE: &str = "brings_the_conversation_back(";
const THE_RETURN: &str = "open_conversation_again(";

fn the_main_window() -> String {
    let whole = fs::read_to_string(THE_MAIN_WINDOW)
        .unwrap_or_else(|why| panic!("{THE_MAIN_WINDOW}: {why}"))
        .replace("\r\n", "\n");
    what_ships(&whole)
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

/// The text after `from` up to the next `to`, or a complaint naming which
/// anchor is gone.
fn between<'a>(text: &'a str, from: &str, to: &str) -> Result<&'a str, String> {
    let start = text
        .find(from)
        .ok_or(format!("{from:?} is no longer here, so this reads nothing"))?
        + from.len();
    let rest = &text[start..];
    let end = rest
        .find(to)
        .ok_or(format!("{to:?} is no longer here, so this reads nothing"))?;
    Ok(&rest[..end + to.len()])
}

/// The Act arm sets the message the command is for through a guard, runs
/// the command through the cell `dispatch` is put in, and only when the
/// command brings the conversation back rebuilds its nodes and opens it
/// again; the guard clears the message whatever the command did.
fn the_loop_runs_the_command_and_comes_back(app: &str) -> Result<(), String> {
    let again = body_of(app, THE_LOOP)?;
    let arm_at = again.find(THE_ACT_ARM).ok_or(format!(
        "{THE_LOOP} has no arm for {THE_ACT_ARM}, so a command chosen in the conversation is \
         never run"
    ))?;
    let arm = &again[arm_at + THE_ACT_ARM.len()..];
    let arm = &arm[..arm.find(THE_NEXT_ARM).unwrap_or(arm.len())];
    for (needed, why) in [
        (
            THE_GUARD,
            "so the command does not know which message it is for, or a later command from \
             the list still acts on it",
        ),
        (THE_CELL, "so the command is never run"),
        (
            THE_REBUILD,
            "so the conversation comes back listing what was true before the command",
        ),
        (
            THE_RULE,
            "so the conversation comes back over the window a reply opened",
        ),
        (THE_RETURN, "so the conversation does not come back at all"),
    ] {
        if !arm.contains(needed) {
            return Err(format!("the Act arm does not reach {needed}, {why}"));
        }
    }
    let rule = arm.find(THE_RULE).unwrap_or(0);
    let back = arm.rfind(THE_RETURN).unwrap_or(0);
    if back < rule {
        return Err(format!(
            "the Act arm opens the conversation again before it asks {THE_RULE}"
        ));
    }
    let sets = between(app, THE_GUARD_SETS.0, THE_GUARD_SETS.1)
        .map_err(|_| "the guard does not set acting_on".to_string())?;
    if sets.contains("\n}\n") {
        return Err(
            "the guard does not set acting_on before its impl ends, so a command from the \
             conversation does not know which message it is for"
                .to_string(),
        );
    }
    let clears = between(app, THE_GUARD_CLEARS.0, THE_GUARD_CLEARS.1).map_err(|_| {
        "nothing drops the guard by clearing acting_on, so a later command from the list acts \
         on the message the conversation named"
            .to_string()
    })?;
    if clears.contains("\n}\n") {
        return Err(
            "the guard's Drop ends before it clears acting_on, so a later command from the list \
             acts on the message the conversation named"
                .to_string(),
        );
    }
    Ok(())
}

#[test]
fn test_the_loop_runs_the_command_on_the_named_message_and_comes_back() {
    the_loop_runs_the_command_and_comes_back(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

/// The loop as it should be, holding every anchor the reading looks for.
fn a_loop_as_it_should_be() -> String {
    "impl ActingOn {\n    fn begin(state, message) -> Self {\n        lock_state(state).acting_on = Some(message);\n    }\n}\n\
     impl Drop for ActingOn {\n    fn drop(&mut self) {\n        lock_state(&self.state).acting_on = None;\n    }\n}\n\
     fn open_conversation_again(\n) {\n    match choice {\n        \
     wx_thread_view::ThreadChoice::Act { command, message } => {\n            \
     let acting = ActingOn::begin(&state, message);\n            \
     let run = runs_a_command.borrow().clone();\n            \
     drop(acting);\n            \
     if !wx_thread_view::brings_the_conversation_back(command) {\n                return;\n            }\n            \
     let now = conversation_nodes(&state, &whose.account_id, &whose.thread_id);\n            \
     open_conversation_again(frame, now);\n        }\n        \
     wx_thread_view::ThreadChoice::Cancelled => {}\n    }\n}\n"
        .to_string()
}

#[test]
fn test_the_loop_reading_passes_a_loop_shaped_as_it_should_be() {
    the_loop_runs_the_command_and_comes_back(&a_loop_as_it_should_be())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_loop_reading_complains_at_each_part_left_out() {
    let app = a_loop_as_it_should_be();
    for (planted, needle, said) in [
        (
            "            let acting = ActingOn::begin(&state, message);\n",
            "",
            "does not reach ActingOn::begin(",
        ),
        (
            "            let run = runs_a_command.borrow().clone();\n",
            "",
            "does not reach runs_a_command.borrow()",
        ),
        (
            "            let now = conversation_nodes(&state, &whose.account_id, &whose.thread_id);\n",
            "            let now = nodes.clone();\n",
            "does not reach conversation_nodes(",
        ),
        (
            "            if !wx_thread_view::brings_the_conversation_back(command) {\n                return;\n            }\n",
            "",
            "does not reach brings_the_conversation_back(",
        ),
        (
            "        lock_state(&self.state).acting_on = None;\n",
            "",
            "acts on the message the conversation named",
        ),
        (
            "        lock_state(state).acting_on = Some(message);\n",
            "",
            "does not set acting_on",
        ),
    ] {
        let broken = app.replacen(planted, needle, 1);
        assert_ne!(
            broken, app,
            "the plant {planted:?} found nothing to replace"
        );
        let why = the_loop_runs_the_command_and_comes_back(&broken)
            .expect_err("a part of the loop left out was passed over");
        assert!(why.contains(said), "{planted:?}: {why}");
    }
}

// ── A message whose text is not here is refused, 13.3-05 ──────────────────
//
// A reply or a forward from an opened conversation read the named message's
// text from the store and, when there was none, quoted nothing: a forward went
// out with a header and an empty body, and nothing was said. Both commands now
// ask `application::reply` what quoting finds, and when the text is not here
// they say so, start its download, and open no compose window.

const THE_QUOTE: &str = "fn the_body_a_command_quotes(";
const THE_DECISION: &str = "what_quoting_finds(";
const THE_REFUSAL: &str = "fn the_text_is_not_here_yet(";
const THE_REFUSAL_CALLED: &str = "the_text_is_not_here_yet(";
const THE_FETCH: &str = "spawn_body_fetch(";
const THE_SENTENCE: &str = "FeedbackEvent::ActionRefused";
const THE_REPLY: &str = "fn start_reply(";
const THE_FORWARD_ARM: &str = "_ if id == ID_FORWARD =>";
const THE_NEXT_COMMAND_ARM: &str = "_ if id ==";
const THE_COMPOSE: &str = "open_compose(";

/// The refusal is reached in `text` before any compose window opens.
fn refuses_before_composing(text: &str, which: &str) -> Result<(), String> {
    let refused = text.find(THE_REFUSAL_CALLED).ok_or(format!(
        "{which} never reaches {THE_REFUSAL_CALLED}, so a message whose text is not here \
         opens with nothing quoted"
    ))?;
    match text.find(THE_COMPOSE) {
        Some(composed) if composed < refused => Err(format!(
            "{which} opens a compose window before it reaches {THE_REFUSAL_CALLED}"
        )),
        _ => Ok(()),
    }
}

/// The quote asks the decision; the refusal fetches and says why; Reply and
/// Forward each refuse before they compose.
fn the_quote_is_refused_and_fetched(app: &str) -> Result<(), String> {
    if !body_of(app, THE_QUOTE)?.contains(THE_DECISION) {
        return Err(format!(
            "{THE_QUOTE} does not ask {THE_DECISION}, so a message whose text is not here is \
             quoted as nothing"
        ));
    }
    let refusal = body_of(app, THE_REFUSAL)?;
    if !refusal.contains(THE_FETCH) {
        return Err(format!(
            "{THE_REFUSAL} does not reach {THE_FETCH}, so the text never starts downloading"
        ));
    }
    if !refusal.contains(THE_SENTENCE) {
        return Err(format!(
            "{THE_REFUSAL} does not signal {THE_SENTENCE}, so the refusal says nothing"
        ));
    }
    refuses_before_composing(&body_of(app, THE_REPLY)?, "start_reply")?;
    let arm_at = app.find(THE_FORWARD_ARM).ok_or(format!(
        "{THE_FORWARD_ARM} is no longer in this file, so this reads nothing"
    ))?;
    let arm = &app[arm_at + THE_FORWARD_ARM.len()..];
    let arm = &arm[..arm.find(THE_NEXT_COMMAND_ARM).unwrap_or(arm.len())];
    refuses_before_composing(arm, "the Forward arm")
}

#[test]
fn test_reply_and_forward_say_the_text_is_not_here_and_fetch_it() {
    the_quote_is_refused_and_fetched(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

/// The four places as they should be, holding every anchor the reading
/// looks for.
fn a_quote_as_it_should_be() -> String {
    "fn the_body_a_command_quotes(state) -> TheQuote {\n    \
     match reply::what_quoting_finds(stored.as_ref(), reachability_of(state)) {}\n}\n\
     fn the_text_is_not_here_yet(app, a11y, not_here: NotHere) {\n    \
     spawn_body_fetch(app, not_here.message_row_id, not_here.uid);\n    \
     let _ = a11y.signal(FeedbackEvent::ActionRefused, sentence);\n}\n\
     fn start_reply(app) {\n    \
     let TheQuote::Here(quote) = the_body_a_command_quotes(state, cache, &message) else {\n        \
     return the_text_is_not_here_yet(app, a11y, not_here);\n    };\n    \
     open_compose(app, frame, cache, a11y, compose);\n}\n\
     fn dispatch() {\n    match id {\n        \
     _ if id == ID_FORWARD => {\n            \
     let TheQuote::Here(body) = quote else {\n                \
     the_text_is_not_here_yet(app, &a11y, not_here);\n                return;\n            };\n            \
     open_compose(app, &frame, &message_cache, &a11y, forward);\n        }\n        \
     _ if id == ID_DELETE => {}\n    }\n}\n"
        .to_string()
}

#[test]
fn test_the_quote_reading_complains_at_each_part_left_out() {
    let app = a_quote_as_it_should_be();
    the_quote_is_refused_and_fetched(&app).unwrap_or_else(|why| panic!("{why}"));
    for (planted, needle, said) in [
        (
            "reply::what_quoting_finds(stored.as_ref(), reachability_of(state))",
            "Some(stored)",
            "does not ask what_quoting_finds(",
        ),
        (
            "    spawn_body_fetch(app, not_here.message_row_id, not_here.uid);\n",
            "",
            "does not reach spawn_body_fetch(",
        ),
        (
            "    let _ = a11y.signal(FeedbackEvent::ActionRefused, sentence);\n",
            "",
            "the refusal says nothing",
        ),
        (
            "        return the_text_is_not_here_yet(app, a11y, not_here);\n",
            "        return;\n",
            "start_reply never reaches the_text_is_not_here_yet(",
        ),
        (
            "                the_text_is_not_here_yet(app, &a11y, not_here);\n",
            "",
            "the Forward arm never reaches the_text_is_not_here_yet(",
        ),
        (
            "            let TheQuote::Here(body) = quote else {\n",
            "            open_compose(app, &frame, &message_cache, &a11y, forward);\n            \
             let TheQuote::Here(body) = quote else {\n",
            "the Forward arm opens a compose window before",
        ),
    ] {
        let broken = app.replacen(planted, needle, 1);
        assert_ne!(
            broken, app,
            "the plant {planted:?} found nothing to replace"
        );
        let why = the_quote_is_refused_and_fetched(&broken)
            .expect_err("a part of the refusal left out was passed over");
        assert!(why.contains(said), "{planted:?}: {why}");
    }
}

// ── A message read from the conversation is marked read, 13.2-03.1 ────────
//
// The reading timer marked only the message the list's cursor row stands
// for, so a message read from an opened conversation, which is a member of
// that row and not its message, was never marked, and neither was a row's
// own message filed in another folder, which the loaded rows do not hold.
// The state is built the way the window builds it, through the store,
// `MessageItem::from_row` and `apply_threading`, because a conversation's
// membership lives in the join between the two.

const THE_ACCOUNT: &str = "acct-one";

fn a_folder(cache: &MessageCache, name: &str) -> i64 {
    cache
        .save_folder(&CachedFolder {
            id: 0,
            account_id: THE_ACCOUNT.to_string(),
            name: name.to_string(),
            path: name.to_string(),
            folder_type: FolderType::Inbox.as_str().to_string(),
            unread_count: 0,
            total_count: 0,
        })
        .expect("the folder")
}

/// Message `uid` of `conversation` in `folder`, arriving on day `day` of
/// September, read or not; its row id comes back.
fn a_message(
    cache: &MessageCache,
    folder: i64,
    conversation: &str,
    uid: u32,
    day: u32,
    read: bool,
) -> i64 {
    let received = format!("2026-09-{day:02}T09:00:00Z");
    cache
        .upsert_message(&IncomingMessage {
            folder_id: folder,
            uid,
            message_id: format!("<{conversation}-{uid}@example.com>"),
            subject: "Quarterly report".to_string(),
            from_addr: format!("Sender {uid} <s{uid}@example.com>"),
            to_addr: "me@example.com".to_string(),
            cc: None,
            reply_to: None,
            date: received.clone(),
            internal_date: Some(received),
            size_bytes: Some(1_000),
            refs_header: Some(format!("<{conversation}>")),
            read,
            starred: false,
            answered: false,
            draft: false,
            deleted: false,
            has_attachments: false,
            safety: Verdict::ordinary(),
            gmail_message_id: None,
            server_thread_id: None,
            labels: None,
            receipt_to: None,
            list_unsubscribe: None,
            pop_uidl: None,
        })
        .expect("the row written")
}

/// The window's state after `folder` opens, showing `showing`, the cursor
/// on the row that stands for or is `cursor_on`, and the reading begun on
/// `began`.
fn the_window_after_opening(
    cache: &MessageCache,
    folder: i64,
    showing: Showing,
    cursor_on: i64,
    began: i64,
) -> WxUIState {
    let rows = cache
        .get_message_list_sorted(folder, THE_ACCOUNT, None, None)
        .expect("the folder read");
    let mut messages: Vec<MessageItem> = rows.iter().map(MessageItem::from_row).collect();
    apply_threading(&rows, &mut messages);
    let conversations = cache
        .conversations_in(
            folder,
            THE_ACCOUNT,
            AConversationReaches::TheWholeAccount,
            None,
        )
        .expect("the conversations listed");
    let row = match showing {
        Showing::Conversations => conversations
            .iter()
            .position(|row| row.stands_for.id == cursor_on),
        _ => messages.iter().position(|row| row.message_id == cursor_on),
    };
    assert!(row.is_some(), "no row is or stands for {cursor_on}");
    WxUIState {
        messages,
        conversations,
        showing,
        selected_message_index: row,
        reading_began: Some((began, std::time::Instant::now())),
        ..WxUIState::default()
    }
}

#[test]
fn test_a_member_read_from_the_conversation_is_what_reading_may_mark() {
    // A, B and C of one conversation in INBOX, nothing read, so the row
    // stands for A; B was read from the opened conversation.
    let dir = tempfile::tempdir().expect("a temporary folder");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache, "INBOX");
    let a = a_message(&cache, inbox, "origin", 1, 1, false);
    let b = a_message(&cache, inbox, "origin", 2, 2, false);
    a_message(&cache, inbox, "origin", 3, 3, false);
    let mut state = the_window_after_opening(&cache, inbox, Showing::Conversations, a, b);
    assert_eq!(state.what_reading_may_mark(), Some(b));

    // Already read, there is nothing to mark.
    for message in state.messages.iter_mut().filter(|m| m.message_id == b) {
        message.read = true;
    }
    assert_eq!(state.what_reading_may_mark(), None);
}

#[test]
fn test_the_rows_own_message_filed_in_another_folder_is_what_reading_may_mark() {
    // The originator in INBOX, read; its reply filed in Sent, unread, so the
    // row stands for the reply, which the loaded rows do not hold and Enter
    // opened from the store (13.2-01).
    let dir = tempfile::tempdir().expect("a temporary folder");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache, "INBOX");
    let sent = a_folder(&cache, "Sent");
    a_message(&cache, inbox, "origin", 1, 1, true);
    let there = a_message(&cache, sent, "origin", 2, 2, false);
    let state = the_window_after_opening(&cache, inbox, Showing::Conversations, there, there);
    assert!(
        state.messages.iter().all(|m| m.message_id != there),
        "the loaded rows hold INBOX alone"
    );
    assert_eq!(state.what_reading_may_mark(), Some(there));
}

#[test]
fn test_a_message_of_another_conversation_is_never_what_reading_may_mark() {
    // Two conversations of two in INBOX; the cursor on the first's row, the
    // reading begun in the second, as when the cursor moved on after it.
    let dir = tempfile::tempdir().expect("a temporary folder");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache, "INBOX");
    let a = a_message(&cache, inbox, "first", 1, 1, false);
    a_message(&cache, inbox, "first", 2, 2, false);
    let d = a_message(&cache, inbox, "second", 3, 3, false);
    let e = a_message(&cache, inbox, "second", 4, 4, false);
    for began in [d, e] {
        let state = the_window_after_opening(&cache, inbox, Showing::Conversations, a, began);
        assert_eq!(state.what_reading_may_mark(), None, "began on {began}");
    }
}

#[test]
fn test_under_the_message_view_reading_may_mark_the_selected_row_alone() {
    let dir = tempfile::tempdir().expect("a temporary folder");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache, "INBOX");
    let a = a_message(&cache, inbox, "origin", 1, 1, false);
    let b = a_message(&cache, inbox, "origin", 2, 2, false);
    let state = the_window_after_opening(&cache, inbox, Showing::Messages, a, a);
    assert_eq!(state.what_reading_may_mark(), Some(a));
    let state = the_window_after_opening(&cache, inbox, Showing::Messages, a, b);
    assert_eq!(
        state.what_reading_may_mark(),
        None,
        "the cursor moved on from the message read"
    );
}

// ── Under the message view too, for a reading begun in the window, 13.3-13 ─
//
// Enter on a row of a conversation of two or more opens the same window in
// the message view, and a message chosen there began a reading the timer
// never marked, because the rule counted the conversation under the
// conversation view alone. The reading now remembers the conversation it
// began in, and only such a reading counts it: one begun with Space on the
// message's own row, after which the cursor moved to a sibling, still marks
// nothing (#25, LIST-03).

/// The conversation `message` is in, as the window opened on it names it.
fn the_conversation_of(state: &WxUIState, message: i64) -> WhoseConversation {
    let loaded = state
        .messages
        .iter()
        .find(|m| m.message_id == message)
        .expect("the message is loaded");
    WhoseConversation {
        account_id: loaded.account_id.clone(),
        thread_id: loaded
            .thread_id
            .clone()
            .expect("the message is in a conversation"),
    }
}

#[test]
fn test_under_the_message_view_a_member_read_from_the_conversation_window_is_what_reading_may_mark()
{
    // A, B and C of one conversation, nothing read; the cursor on A, where
    // Enter opened the window, and B chosen in it.
    let dir = tempfile::tempdir().expect("a temporary folder");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache, "INBOX");
    let a = a_message(&cache, inbox, "origin", 1, 1, false);
    let b = a_message(&cache, inbox, "origin", 2, 2, false);
    let c = a_message(&cache, inbox, "origin", 3, 3, false);
    let mut state = the_window_after_opening(&cache, inbox, Showing::Messages, a, b);
    state.opened_from_a_conversation = Some(the_conversation_of(&state, a));
    assert_eq!(state.what_reading_may_mark(), Some(b));

    // The cursor may rest on any message of the conversation.
    state.selected_message_index = state.messages.iter().position(|m| m.message_id == c);
    assert_eq!(state.what_reading_may_mark(), Some(b), "the cursor on C");

    // Already read, there is nothing to mark.
    for message in state.messages.iter_mut().filter(|m| m.message_id == b) {
        message.read = true;
    }
    assert_eq!(state.what_reading_may_mark(), None);
}

#[test]
fn test_under_the_message_view_a_reading_begun_elsewhere_never_counts_the_conversation() {
    // B read aloud from its own row and the cursor arrowed to A, or B's
    // reading said to have begun in another conversation's window.
    let dir = tempfile::tempdir().expect("a temporary folder");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache, "INBOX");
    let a = a_message(&cache, inbox, "origin", 1, 1, false);
    let b = a_message(&cache, inbox, "origin", 2, 2, false);
    let d = a_message(&cache, inbox, "second", 3, 3, false);
    a_message(&cache, inbox, "second", 4, 4, false);
    let mut state = the_window_after_opening(&cache, inbox, Showing::Messages, a, b);
    assert_eq!(
        state.what_reading_may_mark(),
        None,
        "begun in no conversation"
    );
    state.opened_from_a_conversation = Some(the_conversation_of(&state, d));
    assert_eq!(
        state.what_reading_may_mark(),
        None,
        "begun in another conversation"
    );
}

#[test]
fn test_under_the_message_view_a_cursor_on_another_conversation_marks_nothing() {
    // B chosen in its conversation's window, then the cursor moved to D,
    // a message of another conversation.
    let dir = tempfile::tempdir().expect("a temporary folder");
    let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
    let inbox = a_folder(&cache, "INBOX");
    let a = a_message(&cache, inbox, "origin", 1, 1, false);
    let b = a_message(&cache, inbox, "origin", 2, 2, false);
    let d = a_message(&cache, inbox, "second", 3, 3, false);
    a_message(&cache, inbox, "second", 4, 4, false);
    let mut state = the_window_after_opening(&cache, inbox, Showing::Messages, d, b);
    state.opened_from_a_conversation = Some(the_conversation_of(&state, a));
    assert_eq!(state.what_reading_may_mark(), None);
}

// ── The timer asks it, and marks what the loaded rows do not hold ─────────

const THE_TIMER: &str = "fn mark_what_was_read(";
const ASKS_WHAT_READING_MAY_MARK: &str = "what_reading_may_mark(";
const THE_STORE_READ: &str = "the_stored_message(";

/// The timer asks the state what reading may mark, keeps the rule and the
/// wait it always read, and reads a message the loaded rows do not hold
/// from the store rather than giving up on it.
fn the_timer_marks_what_reading_may(app: &str) -> Result<(), String> {
    let timer = body_of(app, THE_TIMER)?;
    for (needed, why) in [
        (
            ASKS_WHAT_READING_MAY_MARK,
            "so a message read from an opened conversation is never the one it marks",
        ),
        (
            "whether_to_mark_read(",
            "so the decision is not the rule reading_habits holds",
        ),
        ("s.marks_read", "so the wait is not the one Settings writes"),
        (
            THE_STORE_READ,
            "so a message the loaded rows do not hold, one filed in another folder, is never \
             marked",
        ),
    ] {
        if !timer.contains(needed) {
            return Err(format!("mark_what_was_read does not reach {needed}, {why}"));
        }
    }
    Ok(())
}

#[test]
fn test_the_timer_marks_what_reading_may_even_where_the_loaded_rows_do_not_hold_it() {
    the_timer_marks_what_reading_may(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

/// The timer as it should be, holding every anchor the reading looks for.
fn a_timer_as_it_should_be() -> String {
    "fn mark_what_was_read(app: AppHandles<'_>, cache: &Option<Arc<MessageCache>>) {\n    \
     let selected_unread = s.what_reading_may_mark();\n    \
     let Some(row) = whether_to_mark_read(s.reading_began, selected_unread, now, s.marks_read) else {\n        return;\n    };\n    \
     let loaded = s.messages.iter_mut().find(|m| m.message_id == row);\n    \
     let Some((uid, subject)) = loaded.or_else(|| the_stored_message(cache, row)) else {\n        return;\n    };\n}\n"
        .to_string()
}

#[test]
fn test_the_timer_reading_refuses_a_timer_that_gives_up_on_a_message_not_loaded() {
    let timer = a_timer_as_it_should_be();
    the_timer_marks_what_reading_may(&timer).unwrap_or_else(|why| panic!("{why}"));
    for (planted, needle, said) in [
        (
            "loaded.or_else(|| the_stored_message(cache, row))",
            "loaded",
            "does not reach the_stored_message(",
        ),
        (
            "s.what_reading_may_mark()",
            "s.selected_message_index.and_then(|at| s.the_loaded_message_the_row_stands_for(at))",
            "does not reach what_reading_may_mark(",
        ),
    ] {
        let broken = timer.replacen(planted, needle, 1);
        assert_ne!(
            broken, timer,
            "the plant {planted:?} found nothing to replace"
        );
        let why = the_timer_marks_what_reading_may(&broken)
            .expect_err("a timer left without a part was passed over");
        assert!(why.contains(said), "{planted:?}: {why}");
    }
}

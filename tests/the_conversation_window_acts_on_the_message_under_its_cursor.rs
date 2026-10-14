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
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::presentation::tree_walk;
use wixen_mail::presentation::wx_context_menu::command_for;
use wixen_mail::presentation::wx_thread_view::{self, ThreadChoice, ThreadNode};
use wxdragon::prelude::*;

/// The main window's commands, by the ids the message menu raises for them,
/// so the menu line posted below is the one the menu key's popup delivers.
const ID_DELETE: i32 = command_for(Action::DeleteMessage);
const ID_DELETE_OUTRIGHT: i32 = command_for(Action::DeleteMessageOutright);
const ID_MARK_READ: i32 = command_for(Action::MarkRead);

// ── The window, driven by posted messages ──────────────────────────────────

const WM_KEYDOWN: u32 = 0x0100;
const WM_COMMAND: u32 = 0x0111;
const VK_RETURN: usize = 0x0D;
const VK_DELETE: usize = 0x2E;
const VK_M: usize = 0x4D;
const VK_SHIFT: usize = 0x10;
const VK_LSHIFT: usize = 0xA0;
const PM_REMOVE: u32 = 0x0001;
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

/// What is posted, and to which window.
#[derive(Clone, Copy)]
enum Posted {
    Nothing,
    /// A key-down to the tree, with Shift held or not.
    AKey {
        key: usize,
        shift: bool,
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
                Posted::AKey { key, shift } => {
                    if shift {
                        only_these_modifiers_down(&[VK_SHIFT, VK_LSHIFT]);
                    }
                    PostMessageW(tree_window, WM_KEYDOWN, key, KEY_DOWN_ONCE | extended(key));
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
                let key = |key: usize| Posted::AKey { key, shift: false };
                let harvest = Harvest {
                    the_choice_on_arrival: one_case(&frame, StartOn::RowMessage, Posted::Nothing)?,
                    enter_on_the_root: one_case(&frame, StartOn::Root, key(VK_RETURN))?,
                    delete_on_the_root: one_case(&frame, StartOn::Root, key(VK_DELETE))?,
                    delete_on_a_message: one_case(&frame, StartOn::RowMessage, key(VK_DELETE))?,
                    shift_delete_on_a_message: one_case(
                        &frame,
                        StartOn::RowMessage,
                        Posted::AKey {
                            key: VK_DELETE,
                            shift: true,
                        },
                    )?,
                    m_on_a_message: one_case(&frame, StartOn::RowMessage, key(VK_M))?,
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
        .map(|key| key.written)
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

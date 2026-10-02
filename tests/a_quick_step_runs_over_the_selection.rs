//! A Quick Step runs over the selection, from Action, Quick Steps or from its
//! key (#60, GAP-11, 13-42).
//!
//! 13-40 kept steps and 13-41 made them. Until this plan nothing ran one: the
//! Quick Steps submenu held Manage Quick Steps alone, and Ctrl+Shift+7 to
//! Ctrl+Shift+9, which the manager's Key column already showed, reached
//! nothing.
//!
//! **What is read.** A store built in the test with one account holding four
//! steps, one of them written with an action word no build knows. The real
//! menu bar the window is given, its Quick Steps submenu read item by item as
//! the window starts, after the steps are put on it the way the window puts
//! them, after the same steps are put on again, and after a move written to
//! the store. Whether the menu answers a key at a place, for the key handler's
//! question.
//!
//! **The key, measured.** A message list built on the frame the menu bar is
//! on, with the real key handler bound, and `WM_KEYDOWN` for 9 and for 7
//! posted to it with Ctrl and Shift held in this thread's keyboard state,
//! through the window's own loop so the menu bar's accelerators are asked
//! first as they are for a real key. A probe bound before the handler writes
//! down what each key event that reached the list saw: measured 2026-09-30,
//! handlers bound to one control run in the order they were bound, so a
//! probe bound after the handler never sees a key the handler takes.
//! With two steps Ctrl+Shift+9 has no item: the readings are what the event
//! saw and which place the handler answered. Ctrl+Shift+7 has an item: the
//! reading is the menu id the frame was sent.
//!
//! **Source readings** over `what_ships` of `src/presentation/wx_app.rs`,
//! comment lines left out: `folder_tree_updates` sends the steps, and their
//! arm puts them on the menu. A step's item runs `run_the_quick_step_at`,
//! which reads the selection with the step's reach, meets the Select All
//! bound, checks every chosen message's account and what the step names,
//! then calls 13-24.1's runner once and says one sentence; it writes no mail
//! of its own, since every write is the runner's and goes through the gated
//! paths. Each has a companion that plants the fault into a snippet shaped
//! as the window should be, so a reading that stopped finding its anchor
//! cannot pass by finding nothing.
//!
//! **Undo, read in source.** The runner remembers no action of its own; each
//! of the set commands' do-halves it calls remembers its own for Edit, Undo,
//! so after a step the last write it made is what Undo takes back, and the
//! step as a whole is not one undo.
//!
//! **What is not read.** Whether a screen reader says each item's key when
//! the submenu is open, and whether a step is heard as one act with one
//! sentence. Those are the tester's ear and are in the ledger. The window as
//! a whole is not started, and nothing here reaches a mail server.
//!
//! One window session for the file, every reading sharing it through a
//! `OnceLock`, on the shape of
//! `tests/the_saved_searches_menu_says_the_searches_an_account_has.rs`. Runs
//! under `WIXEN_NO_AUDIO` as CI does, and on a temporary data directory in
//! `WIXEN_MAIL_DATA`, never a person's profile. The frame is never shown.

#![cfg(windows)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::filters::Outcome;
use wixen_mail::application::quick_steps::{QuickStep, key_for};
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::data::message_cache::MessageCache;
use wixen_mail::presentation::wx_app::{
    WxMailApp, a_quick_step_key_the_menu_does_not_answer,
    answer_the_quick_step_keys_the_menu_cannot, put_the_quick_steps_on_the_menu,
};
use wxdragon::prelude::*;

type Harvest = BTreeMap<&'static str, String>;

/// The account the steps are kept under.
const ACCOUNT: &str = "acct-quick";

#[link(name = "user32")]
unsafe extern "system" {
    fn PostMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> i32;
    fn GetKeyboardState(state: *mut u8) -> i32;
    fn SetKeyboardState(state: *const u8) -> i32;
    fn CreateDesktopW(
        name: *const u16,
        device: *const u16,
        mode: *const u8,
        flags: u32,
        access: u32,
        attributes: *const u8,
    ) -> isize;
    fn SetThreadDesktop(desktop: isize) -> i32;
}

const GENERIC_ALL: u32 = 0x1000_0000;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetLastError() -> u32;
}

/// Move the calling thread onto a desktop made for this run, on the window
/// station the process is already on, before its first window.
///
/// A window on the interactive desktop is put in front while nobody uses the
/// machine and is not while somebody does, and nothing a person types
/// reaches a desktop that is not the input desktop. The name carries the
/// process id, so two runs at once never share one. Measured 2026-10-02:
/// this target passed 20 of 20 with its window thread on such a desktop.
/// Not a station of its own, where a posted Alt+letter pressed nothing.
///
/// The handle stays open for the life of the process, because the thread's
/// windows live on it.
fn a_desktop_of_its_own(short: &str) -> Result<(), String> {
    let name: Vec<u16> = format!("wixen-{short}-{}", std::process::id())
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

/// winuser.h.
const VK_SHIFT: usize = 0x10;
const VK_CONTROL: usize = 0x11;
const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
/// A key press, repeat count one.
const PRESSED: isize = 0x0000_0001;
/// Its release: previous state and transition bits.
const RELEASED: isize = 0xC000_0001_u32 as i32 as isize;
/// Timer ticks before the posted keys are read, and the tick's length.
const TICKS_FOR_THE_KEYS: u32 = 10;
const TICK_MS: i32 = 30;

// ── The store ─────────────────────────────────────────────────────────────

fn a_step(id: &str, name: &str, does: Outcome) -> QuickStep {
    QuickStep {
        id: id.to_string(),
        name: name.to_string(),
        does,
    }
}

/// The four steps in the order they are made, which is the order they keep.
fn four_steps() -> Vec<QuickStep> {
    vec![
        a_step(
            "step-archive",
            "Archive and read",
            Outcome {
                read: Some(true),
                move_to: Some("INBOX/Archive".to_string()),
                ..Outcome::default()
            },
        ),
        a_step(
            "step-later",
            "Flag for later",
            Outcome {
                starred: Some(true),
                ..Outcome::default()
            },
        ),
        a_step(
            "step-future",
            "From the future",
            Outcome {
                read: Some(true),
                ..Outcome::default()
            },
        ),
        a_step(
            "step-bin",
            "Bin it",
            Outcome {
                delete: true,
                ..Outcome::default()
            },
        ),
    ]
}

/// The store, with the third step's action rewritten in a word no build
/// knows, the way a newer version would leave it, then opened again.
fn a_store(at: &Path) -> Result<MessageCache, String> {
    {
        let cache = MessageCache::new(at.to_path_buf(), None).map_err(|e| format!("{e}"))?;
        for step in four_steps() {
            cache
                .create_quick_step(ACCOUNT, &step)
                .map_err(|e| format!("the step {}: {e}", step.name))?;
        }
    }
    let written_later = rusqlite::Connection::open(at.join("message_cache.db"))
        .map_err(|e| format!("the store's file: {e}"))?;
    written_later
        .execute(
            "UPDATE quick_step_actions SET action_type = 'sing_a_song' \
             WHERE step_id = 'step-future'",
            [],
        )
        .map_err(|e| format!("the newer version's word: {e}"))?;
    drop(written_later);
    MessageCache::new(at.to_path_buf(), None).map_err(|e| format!("{e}"))
}

/// The account's steps, readable or not, by name in the order kept for them.
fn the_names(cache: &MessageCache) -> Vec<String> {
    cache
        .get_quick_steps_for_account(ACCOUNT)
        .unwrap_or_default()
        .iter()
        .map(|step| step.name().to_string())
        .collect()
}

// ── The menu ──────────────────────────────────────────────────────────────

/// A menu's items, one a line, a separator as "---".
fn the_items(menu: &Menu) -> String {
    menu.get_menu_items()
        .iter()
        .map(|item| match item.get_label() {
            label if label.is_empty() => "---".to_string(),
            label => label,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn submenu_labelled(menu: &Menu, wanted: &str) -> Option<Menu> {
    menu.get_menu_items().iter().find_map(|item| {
        let sub = item.get_sub_menu()?;
        match item.get_label() == wanted {
            true => Some(sub),
            false => submenu_labelled(&sub, wanted),
        }
    })
}

fn the_quick_steps_submenu(bar: &MenuBar) -> Option<Menu> {
    (0..bar.get_menu_count())
        .filter_map(|at| bar.get_menu(at))
        .find_map(|menu| submenu_labelled(&menu, "&Quick Steps"))
}

fn the_menu_now(frame: &Frame) -> String {
    frame
        .get_menu_bar()
        .and_then(|bar| the_quick_steps_submenu(&bar))
        .map(|menu| the_items(&menu))
        .unwrap_or_else(|| "no Quick Steps submenu".to_string())
}

// ── The session ───────────────────────────────────────────────────────────

fn read_the_menus(frame: &Frame, cache: &MessageCache, harvest: &mut Harvest) {
    frame.set_menu_bar(WxMailApp::build_menu_bar());
    harvest.insert("the menu as the window starts", the_menu_now(frame));

    let four = the_names(cache);
    put_the_quick_steps_on_the_menu(frame, &four);
    harvest.insert("four steps' menu", the_menu_now(frame));

    // Steps load again whenever the sidebar is read, on a timer among other
    // times, and the same steps put on again must leave the items where they
    // are: a menu somebody has open is not emptied and refilled under them.
    // The first item is marked through its description, which a menu filled
    // again would give back as the one every step item has.
    const MARK: &str = "marked before the same steps were put on again";
    let the_first_items_description = |frame: &Frame, mark: Option<&str>| {
        let menu = frame
            .get_menu_bar()
            .and_then(|bar| the_quick_steps_submenu(&bar))?;
        let id = menu.find_item_by_position(0)?.get_item_id();
        if let Some(mark) = mark {
            menu.set_help_string(id, mark);
        }
        Some(menu.get_help_string(id))
    };
    let _ = the_first_items_description(frame, Some(MARK));
    put_the_quick_steps_on_the_menu(frame, &four);
    harvest.insert(
        "the same steps put on the menu again",
        match the_first_items_description(frame, None).as_deref() == Some(MARK) {
            true => "left as they were".to_string(),
            false => "emptied and filled again".to_string(),
        },
    );

    // Flag for later moved to the top and Bin it above From the future,
    // written to the store the way the manager's close writes an order.
    let _ = cache.put_quick_steps_in_order(
        ACCOUNT,
        &["step-later", "step-archive", "step-bin", "step-future"].map(str::to_string),
    );
    put_the_quick_steps_on_the_menu(frame, &the_names(cache));
    harvest.insert("the menu after a move", the_menu_now(frame));

    let two = ["Archive and read", "Flag for later"].map(str::to_string);
    put_the_quick_steps_on_the_menu(frame, &two);
    if let Some(bar) = frame.get_menu_bar() {
        harvest.insert(
            "Ctrl+Shift+8 and Ctrl+Shift+9 with two steps",
            format!(
                "{} {}",
                a_quick_step_key_the_menu_does_not_answer(&bar, 2),
                a_quick_step_key_the_menu_does_not_answer(&bar, 3),
            ),
        );
    }
}

/// What the posted keys reached, read once the loop has had time to
/// deliver them.
#[derive(Default)]
struct WhatTheKeysReached {
    /// Each key event the list saw: the modifiers it read and the key.
    list_saw: Vec<String>,
    list_answered: Vec<usize>,
    menu_ids: Vec<i32>,
}

fn press(control: isize, digit: u8) {
    // SAFETY: a live window of this thread; the key goes through the
    // window's own loop, which is what a real key does.
    unsafe {
        PostMessageW(control, WM_KEYDOWN, usize::from(digit), PRESSED);
        PostMessageW(control, WM_KEYUP, usize::from(digit), RELEASED);
    }
}

/// Ctrl and Shift set down in this thread's keyboard state; the state
/// before is handed back so it can be put back.
fn ctrl_and_shift_held_in_this_thread() -> [u8; 256] {
    let mut before = [0u8; 256];
    // SAFETY: the buffer is the 256 bytes the call writes.
    unsafe { GetKeyboardState(before.as_mut_ptr()) };
    let mut held = before;
    held[VK_CONTROL] = 0x80;
    held[VK_SHIFT] = 0x80;
    // SAFETY: the buffer is the 256 bytes the call reads, for this thread.
    unsafe { SetKeyboardState(held.as_ptr()) };
    before
}

fn put_back_the_keyboard_state(before: &[u8; 256]) {
    // SAFETY: as above, the state read before the keys were set down.
    unsafe { SetKeyboardState(before.as_ptr()) };
}

fn the_places(answered: &[usize]) -> String {
    match answered {
        [] => "nothing".to_string(),
        places => places
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(" "),
    }
}

// ── The source ────────────────────────────────────────────────────────────

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

/// Where the steps arrive: the arm, not the place that sends them, which is
/// spelled the same up to its bracket.
const THE_LOADED_ARM: &str = "UIUpdate::QuickStepsLoaded(steps) =>";

/// Where the steps are read for the sidebar and sent.
const THE_TREE_READ: &str = "fn folder_tree_updates(";

fn the_main_window() -> String {
    let whole = fs::read_to_string(THE_MAIN_WINDOW)
        .unwrap_or_else(|why| panic!("{THE_MAIN_WINDOW}: {why}"))
        .replace("\r\n", "\n");
    what_ships(&whole)
}

/// The text of the match arm that starts at `anchor`, up to the next arm's
/// guard or pattern at the same kind of place, without its comment lines.
fn arm_of(source: &str, anchor: &str, next_arm: &str) -> Result<String, String> {
    let at = source.find(anchor).ok_or(format!(
        "{anchor} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at + anchor.len()..];
    let ends = rest.find(next_arm).unwrap_or(rest.len());
    Ok(without_comments(&rest[..ends]))
}

/// One function's text, from its signature to the closing brace at column
/// nought, without its comment lines.
fn body_of(source: &str, signature: &str) -> Result<String, String> {
    let at = source.find(signature).ok_or(format!(
        "{signature} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let ends = rest.find("\n}\n").map_or(rest.len(), |end| end + 2);
    Ok(without_comments(&rest[..ends]))
}

fn without_comments(code: &str) -> String {
    code.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Where `name` is called in `code`: each place `name(` stands after
/// something that cannot end an identifier.
fn calls_of(code: &str, name: &str) -> Vec<usize> {
    let call = format!("{name}(");
    code.match_indices(&call)
        .map(|(at, _)| at)
        .filter(|&at| {
            code[..at]
                .chars()
                .next_back()
                .is_none_or(|before| !(before.is_alphanumeric() || before == '_'))
        })
        .collect()
}

/// `folder_tree_updates` sends the account's steps.
fn the_tree_read_sends_the_steps(app: &str) -> Result<(), String> {
    let body = body_of(app, THE_TREE_READ)?;
    match body.contains("UIUpdate::QuickStepsLoaded(") {
        true => Ok(()),
        false => Err(format!(
            "{THE_TREE_READ} sends no UIUpdate::QuickStepsLoaded(, so the menu never hears of \
             a step"
        )),
    }
}

/// The `QuickStepsLoaded` arm puts the steps on the menu.
fn the_loaded_arm_puts_them_on_the_menu(app: &str) -> Result<(), String> {
    let arm = arm_of(app, THE_LOADED_ARM, "UIUpdate::")?;
    match calls_of(&arm, "put_the_quick_steps_on_the_menu").is_empty() {
        true => Err(format!(
            "the {THE_LOADED_ARM} arm never calls put_the_quick_steps_on_the_menu(, so the \
             menu keeps the steps it was built with"
        )),
        false => Ok(()),
    }
}

/// Where a step is run.
const THE_RUN: &str = "fn run_the_quick_step_at(";

/// Where a step's item on the menu is answered.
const THE_ITEM_ARM: &str = "_ if quick_step_position_of(id).is_some() =>";

/// The one runner a step is carried out through (13-24.1).
const THE_RUNNER: &str = "run_these_actions_over";

/// What a run asks, in the order it must ask it: the step's reach, the
/// selection read with it, the Select All bound, every chosen message's
/// account, what the step names in that account, the runner, and the one
/// sentence.
const THE_CHECKS_IN_ORDER: [&str; 7] = [
    "quick_steps::reach",
    "chosen_messages",
    "too_many",
    "owner_of",
    "what_the_account_lacks",
    THE_RUNNER,
    "what_a_step_did",
];

/// Every path that writes mail, each the runner's to call and never a run's.
const THE_WRITES: [&str; 8] = [
    "spawn_server_change",
    "move_or_copy_here_first",
    "set_says_first",
    "mark_these_read",
    "star_these",
    "label_these",
    "move_these",
    "delete_these",
];

/// A step's item runs the step at the item's place.
fn the_item_runs_the_step(app: &str) -> Result<(), String> {
    let arm = arm_of(app, THE_ITEM_ARM, "_ if id")?;
    match calls_of(&arm, "run_the_quick_step_at").len() {
        1 => Ok(()),
        n => Err(format!(
            "the arm at {THE_ITEM_ARM} calls run_the_quick_step_at( {n} times; it should call \
             it once"
        )),
    }
}

/// A run asks each of its checks, in order, the runner exactly once and the
/// sentence exactly once.
fn the_run_checks_in_order_then_runs_once(app: &str) -> Result<(), String> {
    let run = body_of(app, THE_RUN)?;
    let mut last = 0;
    for check in THE_CHECKS_IN_ORDER {
        let calls = calls_of(&run, check);
        let Some(&first) = calls.first() else {
            return Err(format!("{THE_RUN} never calls {check}("));
        };
        if first < last {
            return Err(format!(
                "{THE_RUN} calls {check}( before a check that should come first"
            ));
        }
        last = first;
    }
    for once in [THE_RUNNER, "what_a_step_did"] {
        let n = calls_of(&run, once).len();
        if n != 1 {
            return Err(format!("{THE_RUN} calls {once}( {n} times; once is a step"));
        }
    }
    Ok(())
}

/// A run writes no mail of its own: every write is the runner's.
fn the_run_writes_nothing_of_its_own(app: &str) -> Result<(), String> {
    let run = body_of(app, THE_RUN)?;
    let written: Vec<&str> = THE_WRITES
        .into_iter()
        .filter(|write| !calls_of(&run, write).is_empty())
        .collect();
    match written.as_slice() {
        [] => Ok(()),
        some => Err(format!(
            "{THE_RUN} calls {some:?} itself, which go round the runner and its gate"
        )),
    }
}

/// After the runner a run says one sentence, at Normal, with one Confirmed
/// signal, whatever the count.
fn the_run_says_one_sentence(app: &str) -> Result<(), String> {
    let run = body_of(app, THE_RUN)?;
    let at = calls_of(&run, THE_RUNNER)
        .first()
        .copied()
        .ok_or(format!("{THE_RUN} never calls {THE_RUNNER}("))?;
    let after = &run[at..];
    let announced = calls_of(after, "announce").len();
    let signalled = calls_of(after, "signal").len();
    let spoken_on_the_status_line = calls_of(after, "send_status").len();
    match (announced, signalled, spoken_on_the_status_line) {
        (1, 1, 0) => Ok(()),
        _ => Err(format!(
            "after the runner, {THE_RUN} announces {announced} times, signals {signalled} times \
             and speaks on the status line {spoken_on_the_status_line} times; a step is one \
             sentence and one Confirmed"
        )),
    }
}

/// The runner remembers nothing for Edit, Undo itself; the do-halves it
/// calls each remember their own, so the last write of a step is what Undo
/// takes back.
fn the_runner_leaves_undo_to_the_do_halves(app: &str) -> Result<(), String> {
    let runner = body_of(app, &format!("fn {THE_RUNNER}("))?;
    if !calls_of(&runner, "remember_the_last_action").is_empty() {
        return Err(format!(
            "{THE_RUNNER} remembers an action of its own, so a step may be one undo now; the \
             pages say it is not"
        ));
    }
    for do_half in ["fn mark_these_read(", "fn move_these("] {
        let body = body_of(app, do_half)?;
        if calls_of(&body, "remember_the_last_action").is_empty() {
            return Err(format!("{do_half} no longer remembers its action for Undo"));
        }
    }
    Ok(())
}

/// Whether `first` is called in `code` before `second` is, or why not.
fn called_before(code: &str, first: &str, second: &str, whose: &str) -> Result<(), String> {
    match (
        calls_of(code, first).first(),
        calls_of(code, second).first(),
    ) {
        (Some(a), Some(b)) if a < b => Ok(()),
        (Some(_), Some(_)) => Err(format!("{whose} calls {first}( after {second}(")),
        (None, _) => Err(format!("{whose} never calls {first}(")),
        (_, None) => Err(format!("{whose} never calls {second}(")),
    }
}

/// The runner names the rows whose move carries their marks before the
/// first mark is made, and sends on their own whatever no move took once
/// the move has been asked for (ledger 688).
fn the_runner_hands_the_marks_to_the_move(app: &str) -> Result<(), String> {
    let runner = body_of(app, &format!("fn {THE_RUNNER}("))?;
    let named = runner.find("the_next_move_carries =").ok_or(format!(
        "{THE_RUNNER} never names the rows a move carries marks for"
    ))?;
    let first_mark = calls_of(&runner, "mark_these_read")
        .first()
        .copied()
        .ok_or(format!("{THE_RUNNER} never calls mark_these_read("))?;
    if named > first_mark {
        return Err(format!(
            "{THE_RUNNER} names the rows a move carries marks for after the first mark is made"
        ));
    }
    called_before(
        &runner,
        "move_these",
        "send_the_marks_no_move_took",
        THE_RUNNER,
    )
}

/// Mark as Read's and Star's do-halves hand a mark over before they would
/// send it on a worker of its own.
fn the_do_halves_hand_their_marks_over(app: &str) -> Result<(), String> {
    for do_half in ["fn mark_these_read(", "fn star_these("] {
        let body = body_of(app, do_half)?;
        called_before(&body, "handed_to_the_move", "spawn_server_change", do_half)?;
    }
    Ok(())
}

/// Labelling's do-half hands a label it puts on over before it would send it
/// on a worker of its own, so a run's label goes with its move (ledger 748).
/// Read from where one label is put on, since taking every label off is a
/// command of its own that no run makes.
fn the_label_do_half_hands_its_label_over(app: &str) -> Result<(), String> {
    let do_half = "fn label_these(";
    let body = body_of(app, do_half)?;
    let putting_one_on = body
        .find("LabelChange::One")
        .map(|at| &body[at..])
        .ok_or(format!("{do_half} no longer puts one label on"))?;
    called_before(
        putting_one_on,
        "handed_to_the_move",
        "spawn_server_change",
        do_half,
    )
}

/// A move made here keeps the marks handed to it before the push that sends
/// it is started.
fn the_move_takes_the_marks_before_the_push(app: &str) -> Result<(), String> {
    let made_here = "fn complete_here_then_tell_the_server(";
    let body = body_of(app, made_here)?;
    called_before(
        &body,
        "the_move_takes_its_marks",
        "spawn_blocking",
        made_here,
    )
}

// ── The window session ────────────────────────────────────────────────────

fn take_the_harvest() -> Result<Harvest, String> {
    a_desktop_of_its_own("quick-step")?;
    let data = tempfile::tempdir().map_err(|e| format!("a data directory: {e}"))?;
    // SAFETY: set before the window session starts any thread, and nothing
    // has read either yet.
    unsafe {
        std::env::set_var("WIXEN_MAIL_DATA", data.path());
        std::env::set_var("WIXEN_NO_AUDIO", "1");
    }
    let store_at = tempfile::tempdir().map_err(|e| format!("a store directory: {e}"))?;
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        let store_at = store_at.path().to_path_buf();
        wxdragon::main(move |app| {
            let settle = move |taken: Result<Harvest, String>| {
                if let Ok(mut slot) = outcome.lock() {
                    *slot = Some(taken);
                }
                wxdragon::call_after(Box::new(move || app.exit_main_loop()));
            };
            let frame = Frame::builder()
                .with_title("A Quick Step runs over the selection, the reading")
                .build();
            let cache = match a_store(&store_at) {
                Ok(cache) => cache,
                Err(why) => return settle(Err(why)),
            };
            let mut harvest = Harvest::new();
            read_the_menus(&frame, &cache, &mut harvest);

            // The keys, with two steps on the menu so Ctrl+Shift+9 has no
            // item and Ctrl+Shift+7 has one.
            let reached: Rc<RefCell<WhatTheKeysReached>> = Rc::default();
            let list = ListCtrl::builder(&frame)
                .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel)
                .build();
            // Bound before the handler, so it runs first and sees every key
            // event that reaches the list, then leaves it to the handler.
            list.bind_internal(EventType::KEY_DOWN, {
                let reached = reached.clone();
                move |event| {
                    event.skip(true);
                    let mut saw = Vec::new();
                    if event.control_down() {
                        saw.push("Ctrl".to_string());
                    }
                    if event.shift_down() {
                        saw.push("Shift".to_string());
                    }
                    if event.alt_down() {
                        saw.push("Alt".to_string());
                    }
                    saw.push(
                        event
                            .get_key_code()
                            .and_then(|key| u8::try_from(key).ok())
                            .map_or("?".to_string(), |key| char::from(key).to_string()),
                    );
                    reached.borrow_mut().list_saw.push(saw.join("+"));
                }
            });
            answer_the_quick_step_keys_the_menu_cannot(&list, frame, {
                let reached = reached.clone();
                move |place| reached.borrow_mut().list_answered.push(place)
            });
            frame.on_menu({
                let reached = reached.clone();
                move |event| reached.borrow_mut().menu_ids.push(event.get_id())
            });
            let the_first_items_id = frame
                .get_menu_bar()
                .and_then(|bar| the_quick_steps_submenu(&bar))
                .and_then(|menu| menu.find_item_by_position(0))
                .map(|item| item.get_item_id());
            // Ctrl and Shift are held in this thread's keyboard state until
            // the keys have been read, because both the accelerators and the
            // key event ask the keyboard state. Never through `SendInput`,
            // which would press a key for the whole desktop.
            let before = ctrl_and_shift_held_in_this_thread();
            let the_list = list.get_handle() as isize;
            press(the_list, b'9');
            press(the_list, b'7');

            let ticks = Rc::new(std::cell::Cell::new(0u32));
            let ticker = Rc::new(Timer::new(&frame));
            let harvest = Rc::new(RefCell::new(Some(harvest)));
            ticker.on_tick({
                let ticker = ticker.clone();
                move |_| {
                    ticks.set(ticks.get() + 1);
                    if ticks.get() < TICKS_FOR_THE_KEYS {
                        return;
                    }
                    ticker.stop();
                    let Some(mut harvest) = harvest.borrow_mut().take() else {
                        return;
                    };
                    put_back_the_keyboard_state(&before);
                    let reached = reached.borrow();
                    harvest.insert(
                        "what the list's key events saw",
                        match reached.list_saw.as_slice() {
                            [] => "nothing".to_string(),
                            saw => saw.join(", "),
                        },
                    );
                    harvest.insert(
                        "the places the list answered",
                        the_places(&reached.list_answered),
                    );
                    harvest.insert(
                        "the menu items the keys ran",
                        match the_first_items_id {
                            Some(first) => reached
                                .menu_ids
                                .iter()
                                .map(|id| match *id == first {
                                    true => "the first step".to_string(),
                                    false => format!("id {id}"),
                                })
                                .collect::<Vec<_>>()
                                .join(", "),
                            None => "the menu had no first item".to_string(),
                        },
                    );
                    // A top-level window is destroyed once the loop is idle
                    // again, so the timer ticking now outlives this tick.
                    frame.destroy();
                    settle(Ok(harvest));
                }
            });
            ticker.start(TICK_MS, false);
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

fn reading(name: &str) -> &'static str {
    static HARVEST: OnceLock<Result<Harvest, String>> = OnceLock::new();
    let harvest = match HARVEST.get_or_init(take_the_harvest) {
        Ok(harvest) => harvest,
        Err(why) => panic!("the window session could not be read: {why}"),
    };
    harvest
        .get(name)
        .map(String::as_str)
        .unwrap_or_else(|| panic!("nothing was read for {name:?}"))
}

// ── The checks ────────────────────────────────────────────────────────────

/// The command the submenu ends with, as 13-41 wrote it.
const MANAGE: &str = "&Manage Quick Steps...";

/// The Quick Steps submenu offers exactly these steps, each with the key the
/// menu's rule gives it, then a separator and Manage Quick Steps; with no
/// steps, Manage Quick Steps alone.
fn the_menu_offers(menu: &str, steps: &[&str]) -> Result<(), String> {
    let mut wanted: Vec<String> = steps
        .iter()
        .enumerate()
        .map(|(at, name)| match key_for(at + 1) {
            Some(key) => format!("{name}\t{key}"),
            None => name.to_string(),
        })
        .collect();
    if !steps.is_empty() {
        wanted.push("---".to_string());
    }
    wanted.push(MANAGE.to_string());
    match menu.lines().eq(wanted.iter().map(String::as_str)) {
        true => Ok(()),
        false => Err(format!(
            "the Quick Steps menu read\n{menu}\nand should have read\n{}",
            wanted.join("\n")
        )),
    }
}

#[test]
fn test_the_menu_as_the_window_starts_holds_manage_quick_steps_alone() {
    the_menu_offers(reading("the menu as the window starts"), &[]).unwrap();
}

#[test]
fn test_the_menu_says_an_accounts_steps_in_their_order_with_their_keys() {
    // The one a newer version wrote is at its own place, with its key: the
    // key is heard beside it and runs it, and the run says it cannot.
    the_menu_offers(
        reading("four steps' menu"),
        &[
            "Archive and read",
            "Flag for later",
            "From the future",
            "Bin it",
        ],
    )
    .unwrap();
}

#[test]
fn test_a_moved_step_is_where_the_menu_says_it_next() {
    the_menu_offers(
        reading("the menu after a move"),
        &[
            "Flag for later",
            "Archive and read",
            "Bin it",
            "From the future",
        ],
    )
    .unwrap();
}

#[test]
fn test_the_same_steps_put_on_the_menu_again_leave_it_as_it_was() {
    assert_eq!(
        reading("the same steps put on the menu again"),
        "left as they were"
    );
}

#[test]
fn test_a_key_past_the_last_step_is_one_the_menu_does_not_answer() {
    // Ctrl+Shift+9 with two steps has no item to carry it, so the list
    // answers it rather than leaving it silent.
    assert_eq!(
        reading("Ctrl+Shift+8 and Ctrl+Shift+9 with two steps"),
        "false true"
    );
}

#[test]
fn test_ctrl_shift_and_a_digit_past_the_last_step_reaches_the_list_whole() {
    // Measured, not assumed: posted through the window's loop with Ctrl and
    // Shift held, Ctrl+Shift+9 with no item on the menu reaches the list's
    // key event with both held and the digit, and the handler answers the
    // third place. Ctrl+Shift+7 has an item, which takes it before the list.
    assert_eq!(reading("what the list's key events saw"), "Ctrl+Shift+9");
    assert_eq!(reading("the places the list answered"), "3");
}

#[test]
fn test_ctrl_shift_and_a_digit_with_a_step_runs_the_menu_item() {
    assert_eq!(reading("the menu items the keys ran"), "the first step");
}

#[test]
fn test_the_tree_read_sends_the_steps() {
    the_tree_read_sends_the_steps(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_loaded_arm_puts_the_steps_on_the_menu() {
    the_loaded_arm_puts_them_on_the_menu(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_steps_item_runs_the_step_at_its_place() {
    the_item_runs_the_step(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_run_checks_the_selection_the_bound_and_the_account_before_the_runner() {
    the_run_checks_in_order_then_runs_once(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_run_writes_no_mail_of_its_own() {
    the_run_writes_nothing_of_its_own(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_run_says_one_sentence_with_one_confirmed() {
    the_run_says_one_sentence(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_undo_takes_back_the_last_write_of_a_step_not_the_step() {
    the_runner_leaves_undo_to_the_do_halves(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_runner_hands_each_moving_messages_marks_to_its_move() {
    the_runner_hands_the_marks_to_the_move(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_mark_and_star_hand_a_carried_mark_over_rather_than_send_it() {
    the_do_halves_hand_their_marks_over(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_move_made_here_keeps_its_marks_before_the_push_starts() {
    the_move_takes_the_marks_before_the_push(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_label_a_run_puts_on_is_handed_to_the_move_rather_than_sent() {
    the_label_do_half_hands_its_label_over(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

// ── Companions ────────────────────────────────────────────────────────────

/// The window's half of carrying a mark with its move, shaped as it should
/// be and cut down to what the readings read.
const MARKS_WITH_THE_MOVE: &str = r#"fn run_these_actions_over(
    app: AppHandles<'_>,
) {
    lock_state(app.state).the_next_move_carries = work.the_marks_that_go_with_the_move();
    mark_these_read(app, &held, list, those, *read);
    move_these(app, list, cache, moving, into, false);
    send_the_marks_no_move_took(app, cache);
}
fn mark_these_read(
    app: AppHandles<'_>,
) {
    if !handed_to_the_move(state, message.row_id, &change) {
        spawn_server_change(app, row, uid, subject, folder, change);
    }
}
fn star_these(
    app: AppHandles<'_>,
) {
    if !handed_to_the_move(state, message.row_id, &change) {
        spawn_server_change(app, row, uid, subject, folder, change);
    }
}
fn complete_here_then_tell_the_server(
    app: AppHandles<'_>,
) {
    the_move_takes_its_marks(state, cache, &made.kept);
    rt.spawn_blocking(move || {});
}
fn label_these(
    app: AppHandles<'_>,
) {
    match change {
        LabelChange::AllOff => spawn_server_change(app, row, uid, subject, folder, change),
        LabelChange::One { label, on } => {
            if !handed_to_the_move(app.state, message.row_id, &change) {
                spawn_server_change(app, row, uid, subject, folder, change);
            }
        }
    }
}
"#;

fn the_marks_with(from: &str, to: &str) -> String {
    let planted = MARKS_WITH_THE_MOVE.replacen(from, to, 1);
    assert_ne!(
        planted, MARKS_WITH_THE_MOVE,
        "the companion lost its anchor: {from}"
    );
    planted
}

#[test]
fn test_the_marks_readings_pass_a_window_shaped_as_it_should_be() {
    the_runner_hands_the_marks_to_the_move(MARKS_WITH_THE_MOVE)
        .unwrap_or_else(|why| panic!("{why}"));
    the_do_halves_hand_their_marks_over(MARKS_WITH_THE_MOVE).unwrap_or_else(|why| panic!("{why}"));
    the_move_takes_the_marks_before_the_push(MARKS_WITH_THE_MOVE)
        .unwrap_or_else(|why| panic!("{why}"));
    the_label_do_half_hands_its_label_over(MARKS_WITH_THE_MOVE)
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_companion_a_label_that_goes_on_its_own_beside_a_move_is_refused() {
    let own = the_marks_with(
        "            if !handed_to_the_move(app.state, message.row_id, &change) {\n",
        "            {\n",
    );

    let said = the_label_do_half_hands_its_label_over(&own).expect_err("sent on its own");

    assert!(
        said.contains("fn label_these( never calls handed_to_the_move("),
        "{said}"
    );
}

#[test]
fn test_companion_a_runner_that_hands_nothing_to_the_move_is_refused() {
    let unnamed = the_marks_with(
        "    lock_state(app.state).the_next_move_carries = work.the_marks_that_go_with_the_move();\n",
        "",
    );

    let said = the_runner_hands_the_marks_to_the_move(&unnamed).expect_err("nothing handed");

    assert!(said.contains("never names the rows"), "{said}");
}

#[test]
fn test_companion_a_runner_that_leaves_marks_nobody_sends_is_refused() {
    let left = the_marks_with("    send_the_marks_no_move_took(app, cache);\n", "");

    let said = the_runner_hands_the_marks_to_the_move(&left).expect_err("left behind");

    assert!(
        said.contains("never calls send_the_marks_no_move_took("),
        "{said}"
    );
}

#[test]
fn test_companion_a_star_that_sends_its_own_mark_is_refused() {
    let own = the_marks_with(
        "fn star_these(\n    app: AppHandles<'_>,\n) {\n    if !handed_to_the_move(state, message.row_id, &change) {\n",
        "fn star_these(\n    app: AppHandles<'_>,\n) {\n    {\n",
    );

    let said = the_do_halves_hand_their_marks_over(&own).expect_err("sent on its own");

    assert!(
        said.contains("fn star_these( never calls handed_to_the_move("),
        "{said}"
    );
}

#[test]
fn test_companion_a_move_that_keeps_its_marks_after_the_push_is_refused() {
    let late = the_marks_with(
        "    the_move_takes_its_marks(state, cache, &made.kept);\n    rt.spawn_blocking(move || {});\n",
        "    rt.spawn_blocking(move || {});\n    the_move_takes_its_marks(state, cache, &made.kept);\n",
    );

    let said = the_move_takes_the_marks_before_the_push(&late).expect_err("after the push");

    assert!(said.contains("after spawn_blocking("), "{said}");
}

/// A run shaped as it should be, cut down to what the readings read, with
/// the runner and two do-halves beside it.
const A_RUN: &str = r#"                        _ if quick_step_position_of(id).is_some() => {
                            if let Some(place) = quick_step_position_of(id) {
                                run_the_quick_step_at(app, &message_cache, &msg_list, &a11y, place);
                            }
                        }
                        _ if id == ID_CHECK_FOR_UPDATES => {}
fn run_the_quick_step_at(
    app: AppHandles<'_>,
) {
    let reach = crate::application::quick_steps::reach(&step.does, setting);
    let chosen = match chosen_messages(state, cache, list, reach) {
        Ok(chosen) => chosen,
        Err(why) => return send_refusal(tx, rt, &why),
    };
    if let Some(why) = too_many(chosen.messages.len()) {
        return send_refusal(tx, rt, &why);
    }
    let elsewhere = chosen.messages.iter().filter(|m| owner_of(&s.messages, &s.accounts, m.row_id, open).is_none()).count();
    if let Some(missing) = what_the_account_lacks(&step.does, &folders, &labels) {
        return send_refusal(tx, rt, &what_is_gone(&step.name, &missing));
    }
    let done = match run_these_actions_over(app, list, cache, &chosen, &step.does) {
        Ok(done) => done,
        Err(why) => return send_refusal(tx, rt, &why),
    };
    let said = what_a_step_did(&step.name, &chosen, &done);
    let _ = a11y.announce(&said, Priority::Normal);
    send_shown(tx, rt, &said);
    let _ = a11y.signal(FeedbackEvent::Confirmed, &step.name);
}
fn run_these_actions_over(
    app: AppHandles<'_>,
) {
    mark_these_read(app, &held, list, those, *read);
    move_these(app, list, cache, moving, into, false);
}
fn mark_these_read(
    app: AppHandles<'_>,
) {
    remember_the_last_action(state, marked);
}
fn move_these(
    app: AppHandles<'_>,
) {
    remember_the_last_action(state, moved);
}
"#;

fn a_run_with(from: &str, to: &str) -> String {
    let planted = A_RUN.replacen(from, to, 1);
    assert_ne!(planted, A_RUN, "the companion lost its anchor: {from}");
    planted
}

#[test]
fn test_the_run_readings_pass_a_run_shaped_as_it_should_be() {
    the_item_runs_the_step(A_RUN).unwrap_or_else(|why| panic!("{why}"));
    the_run_checks_in_order_then_runs_once(A_RUN).unwrap_or_else(|why| panic!("{why}"));
    the_run_writes_nothing_of_its_own(A_RUN).unwrap_or_else(|why| panic!("{why}"));
    the_run_says_one_sentence(A_RUN).unwrap_or_else(|why| panic!("{why}"));
    the_runner_leaves_undo_to_the_do_halves(A_RUN).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_companion_a_run_that_skips_the_bound_is_refused() {
    let unbounded = a_run_with(
        "    if let Some(why) = too_many(chosen.messages.len()) {\n        return send_refusal(tx, rt, &why);\n    }\n",
        "",
    );

    let said = the_run_checks_in_order_then_runs_once(&unbounded).expect_err("no bound");

    assert!(said.contains("never calls too_many("), "{said}");
}

#[test]
fn test_companion_a_run_that_skips_the_account_check_is_refused() {
    let any_account = a_run_with(
        "    let elsewhere = chosen.messages.iter().filter(|m| owner_of(&s.messages, &s.accounts, m.row_id, open).is_none()).count();\n",
        "",
    );

    let said = the_run_checks_in_order_then_runs_once(&any_account).expect_err("no account check");

    assert!(said.contains("never calls owner_of("), "{said}");
}

#[test]
fn test_companion_a_run_that_calls_the_runner_twice_is_refused() {
    let twice = a_run_with(
        "    let said = what_a_step_did(",
        "    let _ = run_these_actions_over(app, list, cache, &chosen, &step.does);\n    let said = what_a_step_did(",
    );

    let said = the_run_checks_in_order_then_runs_once(&twice).expect_err("runner twice");

    assert!(said.contains("run_these_actions_over( 2 times"), "{said}");
}

#[test]
fn test_companion_a_run_that_says_a_second_sentence_is_refused() {
    let twice = a_run_with(
        "    send_shown(tx, rt, &said);\n",
        "    send_shown(tx, rt, &said);\n    let _ = a11y.announce(\"Done\", Priority::Normal);\n",
    );

    let said = the_run_says_one_sentence(&twice).expect_err("second sentence");

    assert!(said.contains("announces 2 times"), "{said}");
}

#[test]
fn test_companion_a_run_that_writes_round_the_runner_is_refused() {
    let round = a_run_with(
        "    let said = what_a_step_did(",
        "    spawn_server_change(app, row, uid, subject, None, change);\n    let said = what_a_step_did(",
    );

    let said = the_run_writes_nothing_of_its_own(&round).expect_err("a write of its own");

    assert!(said.contains("spawn_server_change"), "{said}");
}

#[test]
fn test_companion_a_runner_that_remembers_the_step_is_refused() {
    let one_undo = a_run_with(
        "    move_these(app, list, cache, moving, into, false);\n",
        "    move_these(app, list, cache, moving, into, false);\n    remember_the_last_action(state, the_step);\n",
    );

    let said = the_runner_leaves_undo_to_the_do_halves(&one_undo).expect_err("one undo");

    assert!(said.contains("remembers an action of its own"), "{said}");
}

/// A window shaped as it should be, cut down to what the readings read.
const SHAPED: &str = r#"fn folder_tree_updates(
    cache: &MessageCache,
    account_id: &str,
) -> crate::common::Result<Vec<UIUpdate>> {
    Ok(vec![
        UIUpdate::LabelsLoaded(labels),
        UIUpdate::QuickStepsLoaded(steps),
    ])
}
        UIUpdate::QuickStepsLoaded(steps) => {
            let names = { lock_state(state).quick_steps = steps.clone(); names };
            put_the_quick_steps_on_the_menu(frame, &names);
        }
        UIUpdate::SavedSearchRan { messages, said } => {}
"#;

fn planted(from: &str, to: &str) -> String {
    let planted = SHAPED.replacen(from, to, 1);
    assert_ne!(planted, SHAPED, "the companion lost its anchor: {from}");
    planted
}

#[test]
fn test_the_readings_pass_a_window_shaped_as_it_should_be() {
    the_tree_read_sends_the_steps(SHAPED).unwrap_or_else(|why| panic!("{why}"));
    the_loaded_arm_puts_them_on_the_menu(SHAPED).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_companion_a_tree_read_that_sends_no_steps_is_refused() {
    let silent = planted("        UIUpdate::QuickStepsLoaded(steps),\n", "");

    let said = the_tree_read_sends_the_steps(&silent).expect_err("no steps sent");

    assert!(
        said.contains("sends no UIUpdate::QuickStepsLoaded"),
        "{said}"
    );
}

#[test]
fn test_companion_a_loaded_arm_that_only_stores_is_refused() {
    let stores_only = planted(
        "            put_the_quick_steps_on_the_menu(frame, &names);\n",
        "",
    );

    let said = the_loaded_arm_puts_them_on_the_menu(&stores_only).expect_err("only stored");

    assert!(
        said.contains("never calls put_the_quick_steps_on_the_menu"),
        "{said}"
    );
}

#[test]
fn test_companion_a_menu_of_manage_alone_is_refused_for_an_account_with_steps() {
    // The submenu until 13-42: Manage Quick Steps and nothing else, whatever
    // the account's steps were.
    assert!(the_menu_offers(MANAGE, &["Archive and read", "Flag for later"]).is_err());
}

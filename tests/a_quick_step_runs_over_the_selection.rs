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
//! arm puts them on the menu. Each has a companion that plants the fault into
//! a snippet shaped as the window should be, so a reading that stopped
//! finding its anchor cannot pass by finding nothing.
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

// ── The window session ────────────────────────────────────────────────────

fn take_the_harvest() -> Result<Harvest, String> {
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

// ── Companions ────────────────────────────────────────────────────────────

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

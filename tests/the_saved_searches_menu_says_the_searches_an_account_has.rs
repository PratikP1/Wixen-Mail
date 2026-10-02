//! The Saved Searches submenu says the searches of the account being worked
//! in, in the order the folder tree shows them, the first six with Alt+4 to
//! Alt+9 beside them, and a key past the last search is answered in words
//! (13-38, GAP-09's second clause, #58).
//!
//! Until 2026-09-30 the submenu held three commands, Edit Conditions, Rename
//! and Delete, for whichever search the tree's cursor was on, and a search
//! could only be run by finding its row and pressing Enter.
//!
//! **What is read.** A store built in the test with two accounts: one with
//! three searches, one of them written with a word for "all" or "any" no
//! build knows, so it cannot run, and one with seven. The real menu bar the
//! window is given, its Saved Searches submenu read item by item as the
//! window starts, after each account's searches are put on it the way the
//! window puts them, after the same searches are put on again, and after a
//! rename and a move written to the store. Whether the menu answers a key at
//! a place, for the key handler's question.
//!
//! **The key, measured.** A folder tree and a message list built on the
//! frame the menu bar is on, each with the real key handler bound, and
//! `WM_SYSKEYDOWN` for 7 and for 4 posted to each with the Alt bit set, the
//! way Windows delivers Alt and a digit, through the window's own loop so the
//! menu bar's accelerators are asked first as they are for a real key. Alt is
//! also held in this thread's keyboard state while the keys are delivered:
//! measured 2026-09-30, without it the accelerators saw no Alt, since they
//! ask the keyboard state, and Alt+4 reached the tree as a key, while the key
//! event read Alt from the message's flag either way. With
//! three searches Alt+7 has no item: the reading is which place each
//! control's handler answered. Alt+4 has an item: the reading is the menu id
//! the frame was sent, and whether either handler answered it as well.
//!
//! **Source readings** over `what_ships` of `src/presentation/wx_app.rs`,
//! comment lines left out: the `SavedSearchesLoaded` arm puts the searches on
//! the menu, and the arm that answers the menu's search items puts the tree's
//! cursor on the search's row before it runs the search, by the path Enter
//! takes. Each has a companion that plants the fault into a snippet shaped as
//! the window should be, so a reading that stopped finding its anchor cannot
//! pass by finding nothing.
//!
//! **What is not read.** Whether a screen reader says each item's key when
//! the submenu is open, whether the answer to a key past the last search is
//! heard, and whether the cursor is heard landing on the row a key ran.
//! Those are the tester's ear and are in the ledger. The window as a whole is
//! not started.
//!
//! One window session for the file, every reading sharing it through a
//! `OnceLock`, on the shape `tests/the_label_menu_says_the_labels_an_account_has.rs`
//! uses. Runs under `WIXEN_NO_AUDIO` as CI does, and on a temporary data
//! directory in `WIXEN_MAIL_DATA`, never a person's profile.

#![cfg(windows)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::saved_searches::{self, Join, Question, SavedSearch};
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::data::message_cache::MessageCache;
use wixen_mail::data::message_cache::saved_searches::SavedSearchesRead;
use wixen_mail::presentation::wx_app::{
    WxMailApp, a_saved_search_key_the_menu_does_not_answer,
    answer_the_saved_search_keys_the_menu_cannot, put_the_saved_searches_on_the_menu,
};
use wxdragon::prelude::*;

type Harvest = BTreeMap<&'static str, String>;

/// An account with three searches, one of them written by a newer build.
const THREE: &str = "acct-three";
/// An account with more searches than there are keys.
const SEVEN: &str = "acct-seven";

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
const VK_MENU: usize = 0x12;
const WM_SYSKEYDOWN: u32 = 0x0104;
const WM_SYSKEYUP: u32 = 0x0105;
/// A key press with Alt held, the context bit (29) set, repeat count one.
const ALT_HELD: isize = 0x2000_0001;
/// Its release: context, previous state and transition bits.
const ALT_RELEASED: isize = 0xE000_0001_u32 as i32 as isize;
/// Timer ticks before the posted keys are read, and the tick's length.
const TICKS_FOR_THE_KEYS: u32 = 10;
const TICK_MS: i32 = 30;

// ── The store ─────────────────────────────────────────────────────────────

fn a_search(cache: &MessageCache, account_id: &str, id: &str, name: &str) -> Result<(), String> {
    cache
        .create_saved_search(
            account_id,
            &SavedSearch {
                id: id.to_string(),
                name: name.to_string(),
                join: Join::Any,
                questions: vec![Question {
                    field: "subject".to_string(),
                    match_type: "contains".to_string(),
                    pattern: name.to_string(),
                    case_sensitive: false,
                }],
                folder: None,
            },
        )
        .map_err(|e| format!("a search for {account_id}: {e}"))
}

/// The store, with the third search of the first account given a join word
/// no build knows, as a newer version of the program might write one.
fn a_store(at: &Path) -> Result<MessageCache, String> {
    {
        let cache = MessageCache::new(at.to_path_buf(), None).map_err(|e| format!("{e}"))?;
        a_search(&cache, THREE, "s-invoices", "Invoices")?;
        a_search(&cache, THREE, "s-bills", "Bills")?;
        a_search(&cache, THREE, "s-newer", "From a newer version")?;
        for n in 1..=7 {
            a_search(&cache, SEVEN, &format!("s-{n}"), &format!("Search {n}"))?;
        }
    }
    let written_by_a_newer_build = rusqlite::Connection::open(at.join("message_cache.db"))
        .map_err(|e| format!("the store's file: {e}"))?;
    written_by_a_newer_build
        .execute(
            "UPDATE saved_searches SET all_or_any = 'most' WHERE id = 's-newer'",
            [],
        )
        .map_err(|e| format!("the newer build's search: {e}"))?;
    drop(written_by_a_newer_build);
    MessageCache::new(at.to_path_buf(), None).map_err(|e| format!("{e}"))
}

/// An account's searches, readable or not, by name in the order kept for
/// them, which is the order the folder tree shows.
fn the_names(cache: &MessageCache, account_id: &str) -> Vec<String> {
    let read: SavedSearchesRead = cache
        .get_saved_searches_for_account(account_id)
        .unwrap_or_default();
    let named = |id: &String| {
        read.searches
            .iter()
            .map(|search| (&search.id, &search.name))
            .chain(
                read.saved_by_another_version
                    .iter()
                    .map(|search| (&search.id, &search.name)),
            )
            .find(|(held, _)| *held == id)
            .map(|(_, name)| name.clone())
    };
    read.order.iter().filter_map(named).collect()
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

fn the_saved_search_submenu(bar: &MenuBar) -> Option<Menu> {
    (0..bar.get_menu_count())
        .filter_map(|at| bar.get_menu(at))
        .find_map(|menu| submenu_labelled(&menu, "Saved Searc&hes"))
}

fn the_menu_now(frame: &Frame) -> String {
    frame
        .get_menu_bar()
        .and_then(|bar| the_saved_search_submenu(&bar))
        .map(|menu| the_items(&menu))
        .unwrap_or_else(|| "no Saved Searches submenu".to_string())
}

// ── The session ───────────────────────────────────────────────────────────

fn read_the_menus(frame: &Frame, cache: &MessageCache, harvest: &mut Harvest) {
    frame.set_menu_bar(WxMailApp::build_menu_bar());
    harvest.insert("the menu as the window starts", the_menu_now(frame));

    let three = the_names(cache, THREE);
    put_the_saved_searches_on_the_menu(frame, &three);
    harvest.insert("three searches' menu", the_menu_now(frame));

    // Searches load again whenever the sidebar is read, on a timer among
    // other times, and the same searches put on again must leave the items
    // where they are: a menu somebody has open is not emptied and refilled
    // under them. The first item is marked through its description, which a
    // menu filled again would give back as the one every search item has.
    const MARK: &str = "marked before the same searches were put on again";
    let the_first_items_description = |frame: &Frame, mark: Option<&str>| {
        let menu = frame
            .get_menu_bar()
            .and_then(|bar| the_saved_search_submenu(&bar))?;
        let id = menu.find_item_by_position(0)?.get_item_id();
        if let Some(mark) = mark {
            menu.set_help_string(id, mark);
        }
        Some(menu.get_help_string(id))
    };
    let _ = the_first_items_description(frame, Some(MARK));
    put_the_saved_searches_on_the_menu(frame, &three);
    harvest.insert(
        "the same searches put on the menu again",
        match the_first_items_description(frame, None).as_deref() == Some(MARK) {
            true => "left as they were".to_string(),
            false => "emptied and filled again".to_string(),
        },
    );
    if let Some(bar) = frame.get_menu_bar() {
        harvest.insert(
            "Alt+6 and Alt+7 with three searches",
            format!(
                "{} {}",
                a_saved_search_key_the_menu_does_not_answer(&bar, 3),
                a_saved_search_key_the_menu_does_not_answer(&bar, 4),
            ),
        );
    }

    // Bills renamed Receipts and moved to the top, both written to the store
    // the way the rename and the gesture write them.
    let _ = cache.rename_saved_search("s-bills", "Receipts");
    let _ = cache.put_saved_searches_in_order(
        THREE,
        &["s-bills", "s-invoices", "s-newer"].map(str::to_string),
    );
    put_the_saved_searches_on_the_menu(frame, &the_names(cache, THREE));
    harvest.insert("the menu after a rename and a move", the_menu_now(frame));

    put_the_saved_searches_on_the_menu(frame, &the_names(cache, SEVEN));
    harvest.insert("seven searches' menu", the_menu_now(frame));

    put_the_saved_searches_on_the_menu(frame, &[]);
    harvest.insert("no searches' menu", the_menu_now(frame));
}

/// What the posted keys reached, read once the loop has had time to
/// deliver them.
#[derive(Default)]
struct WhatTheKeysReached {
    tree_answered: Vec<usize>,
    list_answered: Vec<usize>,
    menu_ids: Vec<i32>,
}

fn press_alt_and(control: isize, digit: u8) {
    // SAFETY: a live window of this thread; the key goes through the
    // window's own loop, which is what a real key does.
    unsafe {
        PostMessageW(control, WM_SYSKEYDOWN, usize::from(digit), ALT_HELD);
        PostMessageW(control, WM_SYSKEYUP, usize::from(digit), ALT_RELEASED);
    }
}

/// Alt set down in this thread's keyboard state; the state before is
/// handed back so it can be put back.
fn alt_held_in_this_thread() -> [u8; 256] {
    let mut before = [0u8; 256];
    // SAFETY: the buffer is the 256 bytes the call writes.
    unsafe { GetKeyboardState(before.as_mut_ptr()) };
    let mut held = before;
    held[VK_MENU] = 0x80;
    // SAFETY: the buffer is the 256 bytes the call reads, for this thread.
    unsafe { SetKeyboardState(held.as_ptr()) };
    before
}

fn put_back_the_keyboard_state(before: &[u8; 256]) {
    // SAFETY: as above, the state read before Alt was set down.
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

/// Where the searches arrive from the store: the arm, not the place that
/// sends them, which is spelled the same up to its bracket.
const THE_LOADED_ARM: &str = "UIUpdate::SavedSearchesLoaded(searches) =>";

/// Where a search item on the menu is answered.
const THE_RANGE_ARM: &str = "_ if saved_search_position_of(id).is_some() =>";

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
    Ok(rest[..ends]
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n"))
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

/// The `SavedSearchesLoaded` arm puts the searches on the menu.
fn the_loaded_arm_puts_them_on_the_menu(app: &str) -> Result<(), String> {
    let arm = arm_of(app, THE_LOADED_ARM, "UIUpdate::")?;
    match calls_of(&arm, "put_the_saved_searches_on_the_menu").is_empty() {
        true => Err(format!(
            "the {THE_LOADED_ARM} arm never calls put_the_saved_searches_on_the_menu(, so the \
             menu keeps the searches it was built with"
        )),
        false => Ok(()),
    }
}

/// The arm for a search item puts the cursor on the search's row, then runs
/// it through the path Enter takes, and never runs it before moving there.
fn the_range_arm_lands_then_runs(app: &str) -> Result<(), String> {
    let arm = arm_of(app, THE_RANGE_ARM, "_ if id")?;
    let landed = calls_of(&arm, "select_row");
    let run = calls_of(&arm, "run_a_saved_search");
    let (Some(first_landing), false) = (landed.iter().min(), run.is_empty()) else {
        return Err(format!(
            "the arm at {THE_RANGE_ARM} calls select_row( {} times and run_a_saved_search( {} \
             times; it should call both",
            landed.len(),
            run.len()
        ));
    };
    if calls_of(&arm, "the_search_a_row_names").is_empty() {
        return Err(format!(
            "the arm at {THE_RANGE_ARM} never asks the_search_a_row_names(, so it does not run \
             a search the way Enter on its row does"
        ));
    }
    match run.iter().any(|at| at < first_landing) {
        true => Err(format!(
            "the arm at {THE_RANGE_ARM} calls run_a_saved_search( before select_row("
        )),
        false => Ok(()),
    }
}

// ── The window session ────────────────────────────────────────────────────

fn take_the_harvest() -> Result<Harvest, String> {
    a_desktop_of_its_own("saved-searches")?;
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
                .with_title("The saved searches menu says the searches an account has, the reading")
                .build();
            let cache = match a_store(&store_at) {
                Ok(cache) => cache,
                Err(why) => return settle(Err(why)),
            };
            let mut harvest = Harvest::new();
            read_the_menus(&frame, &cache, &mut harvest);

            // The keys, with the three searches on the menu so Alt+7 has no
            // item and Alt+4 has one.
            put_the_saved_searches_on_the_menu(&frame, &the_names(&cache, THREE));
            let reached: Rc<RefCell<WhatTheKeysReached>> = Rc::default();
            let tree = TreeCtrl::builder(&frame)
                .with_style(
                    TreeCtrlStyle::HideRoot
                        | TreeCtrlStyle::HasButtons
                        | TreeCtrlStyle::LinesAtRoot,
                )
                .build();
            let list = ListCtrl::builder(&frame)
                .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel)
                .build();
            answer_the_saved_search_keys_the_menu_cannot(&tree, frame, {
                let reached = reached.clone();
                move |place| reached.borrow_mut().tree_answered.push(place)
            });
            answer_the_saved_search_keys_the_menu_cannot(&list, frame, {
                let reached = reached.clone();
                move |place| reached.borrow_mut().list_answered.push(place)
            });
            frame.on_menu({
                let reached = reached.clone();
                move |event| reached.borrow_mut().menu_ids.push(event.get_id())
            });
            let the_first_items_id = frame
                .get_menu_bar()
                .and_then(|bar| the_saved_search_submenu(&bar))
                .and_then(|menu| menu.find_item_by_position(0))
                .map(|item| item.get_item_id());
            // Alt is held in this thread's keyboard state until the keys have
            // been read, because the menu's accelerators ask the keyboard
            // state whether Alt is down, while the key event reads the flag
            // the message carries. Never through `SendInput`, which would
            // press a key for the whole desktop.
            let before = alt_held_in_this_thread();
            for control in [tree.get_handle() as isize, list.get_handle() as isize] {
                press_alt_and(control, b'7');
                press_alt_and(control, b'4');
            }

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
                        "the places the tree answered",
                        the_places(&reached.tree_answered),
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
                                    true => "the first search".to_string(),
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

/// The commands the submenu ends with: the three that read on 2026-09-24, and
/// New Saved Search before them since 2026-09-30 (13-39, #58 point 3).
const THE_COMMANDS: [&str; 4] = [
    "&New Saved Search...",
    "Edit &Conditions...",
    "&Rename...",
    "&Delete",
];

/// The Saved Searches submenu offers exactly these searches, each with the
/// key the menu's rule gives it, then a separator and the four commands.
fn the_menu_offers(menu: &str, searches: &[&str]) -> Result<(), String> {
    let mut wanted: Vec<String> = searches
        .iter()
        .enumerate()
        .map(|(at, name)| match saved_searches::key_for(at + 1) {
            Some(key) => format!("{name}\t{key}"),
            None => name.to_string(),
        })
        .collect();
    if !searches.is_empty() {
        wanted.push("---".to_string());
    }
    wanted.extend(THE_COMMANDS.map(str::to_string));
    match menu.lines().eq(wanted.iter().map(String::as_str)) {
        true => Ok(()),
        false => Err(format!(
            "the Saved Searches menu read\n{menu}\nand should have read\n{}",
            wanted.join("\n")
        )),
    }
}

#[test]
fn test_the_menu_as_the_window_starts_holds_the_commands() {
    the_menu_offers(reading("the menu as the window starts"), &[]).unwrap();
}

#[test]
fn test_the_menu_says_an_accounts_searches_in_the_trees_order_with_their_keys() {
    // The one a newer build wrote is at its own place, with its key: Alt+6
    // is heard beside it and runs it, and the run says it cannot.
    the_menu_offers(
        reading("three searches' menu"),
        &["Invoices", "Bills", "From a newer version"],
    )
    .unwrap();
}

#[test]
fn test_a_renamed_and_moved_search_is_what_the_menu_says_next() {
    the_menu_offers(
        reading("the menu after a rename and a move"),
        &["Receipts", "Invoices", "From a newer version"],
    )
    .unwrap();
}

#[test]
fn test_a_seventh_search_is_on_the_menu_without_a_key() {
    let names: Vec<String> = (1..=7).map(|n| format!("Search {n}")).collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    the_menu_offers(reading("seven searches' menu"), &names).unwrap();
}

#[test]
fn test_an_account_with_no_searches_is_offered_the_commands_alone() {
    the_menu_offers(reading("no searches' menu"), &[]).unwrap();
}

#[test]
fn test_the_same_searches_put_on_the_menu_again_leave_it_as_it_was() {
    assert_eq!(
        reading("the same searches put on the menu again"),
        "left as they were"
    );
}

#[test]
fn test_a_key_past_the_last_search_is_one_the_menu_does_not_answer() {
    // Alt+7 with three searches has no item to carry it, so the tree and
    // the list answer it rather than leaving it silent.
    assert_eq!(reading("Alt+6 and Alt+7 with three searches"), "false true");
}

#[test]
fn test_alt_and_a_digit_past_the_last_search_reaches_the_tree_and_the_list() {
    // Measured, not assumed: posted through the window's loop, Alt+7 with
    // no item on the menu reaches the handler bound on each control, which
    // answers the fourth place. Alt+4 has an item and is not answered here.
    assert_eq!(reading("the places the tree answered"), "4");
    assert_eq!(reading("the places the list answered"), "4");
}

#[test]
fn test_alt_and_a_digit_with_a_search_runs_the_menu_item() {
    // Alt+4 from the tree and from the list: the menu's accelerator takes
    // it, the frame is sent the first search item's id, once each.
    assert_eq!(
        reading("the menu items the keys ran"),
        "the first search, the first search"
    );
}

#[test]
fn test_the_loaded_arm_puts_the_searches_on_the_menu() {
    the_loaded_arm_puts_them_on_the_menu(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_search_item_lands_on_its_row_then_runs_it() {
    the_range_arm_lands_then_runs(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

// ── Companions ────────────────────────────────────────────────────────────

/// A window shaped as it should be, cut down to what the readings read.
const SHAPED: &str = r#"        UIUpdate::SavedSearchesLoaded(searches) => {
            let names = { lock_state(state).saved_searches = (**searches).clone(); names };
            put_the_saved_searches_on_the_menu(frame, &names);
        }
        UIUpdate::SavedSearchRan { messages, said } => {}
                        _ if saved_search_position_of(id).is_some() => {
                            let row = the_saved_search_at(&lock_state(&state), id);
                            if let Some(row) = row {
                                select_row(&folder_tree, &rows, &row.stored());
                                let chosen = the_search_a_row_names(&lock_state(&state), &row);
                                if let Some(chosen) = chosen {
                                    run_a_saved_search(&ui_tx, &runtime, chosen);
                                }
                            }
                        }
                        _ if id == ID_NEXT => {}
"#;

fn planted(from: &str, to: &str) -> String {
    let planted = SHAPED.replacen(from, to, 1);
    assert_ne!(planted, SHAPED, "the companion lost its anchor: {from}");
    planted
}

#[test]
fn test_the_readings_pass_a_window_shaped_as_it_should_be() {
    the_loaded_arm_puts_them_on_the_menu(SHAPED).unwrap_or_else(|why| panic!("{why}"));
    the_range_arm_lands_then_runs(SHAPED).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_companion_a_loaded_arm_that_only_stores_is_refused() {
    let stores_only = planted(
        "            put_the_saved_searches_on_the_menu(frame, &names);\n",
        "",
    );

    let said = the_loaded_arm_puts_them_on_the_menu(&stores_only).expect_err("only stored");

    assert!(
        said.contains("never calls put_the_saved_searches_on_the_menu"),
        "{said}"
    );
}

#[test]
fn test_companion_a_search_run_before_the_cursor_moves_is_refused() {
    let early = planted(
        "                                select_row(&folder_tree, &rows, &row.stored());\n",
        "",
    )
    .replacen(
        "                                if let Some(chosen) = chosen {\n                                    run_a_saved_search(&ui_tx, &runtime, chosen);\n                                }\n",
        "                                if let Some(chosen) = chosen {\n                                    run_a_saved_search(&ui_tx, &runtime, chosen);\n                                }\n                                select_row(&folder_tree, &rows, &row.stored());\n",
        1,
    );

    let said = the_range_arm_lands_then_runs(&early).expect_err("run first");

    assert!(said.contains("before select_row"), "{said}");
}

#[test]
fn test_companion_a_search_item_that_never_lands_is_refused() {
    let unlanded = planted(
        "                                select_row(&folder_tree, &rows, &row.stored());\n",
        "",
    );

    let said = the_range_arm_lands_then_runs(&unlanded).expect_err("never lands");

    assert!(said.contains("select_row( 0 times"), "{said}");
}

#[test]
fn test_companion_a_menu_of_the_commands_alone_is_refused_for_an_account_with_searches() {
    // The submenu until 13-38: the commands and nothing else, whatever the
    // account's searches were.
    let the_old_menu = THE_COMMANDS.join("\n");
    assert!(
        the_menu_offers(
            &the_old_menu,
            &["Invoices", "Bills", "From a newer version"]
        )
        .is_err()
    );
}

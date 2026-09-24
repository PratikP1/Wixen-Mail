//! The Label menu says the labels an account has, in the order it keeps, and
//! the keys apply the label the menu shows (#48).
//!
//! The tester on 2026-09-15: "The action menu allows the user to apply
//! labels. Currently there are five preconfigured labels. Four additional
//! slots are available but are not assigned. There is NO UI for creating
//! these labels. Create a UI for this functionality for additional labels as
//! well as for editing and moving labels' order." Until 12-10 the submenu was
//! built once from the five an account starts with while Ctrl and a number
//! applied the stored labels in name order, so Ctrl+2 said Work and applied
//! Later.
//!
//! **What is read.** A store built in the test with three accounts: one
//! whose five labels were made the way the first key makes them, one whose
//! labels were written by a build before labels had a place and are numbered
//! when the store opens, and one with eleven. The real menu bar the window is
//! given, its Label submenu read item by item as the window starts and after
//! each account's labels are put on it the way the window puts them; the
//! label `at_number` answers beside the item the menu shows at that number; a
//! rename and a move written to the store and the submenu read again; the
//! Tools menu's item; the Label Manager built and its rows read off the live
//! list, a move made in it, and the order it hands back stored.
//!
//! **What is not read.** That the window calls this when the labels load and
//! when the Label Manager closes: that is one line in each place, and the
//! scan and NVDA runs of the running program are what reach it.
//!
//! **Companions.** Each check is handed a wrong state, the menu built from
//! the five an account starts with and a key reading the labels by name, and
//! refuses it, so a check that passes is one that could have failed.
//!
//! One window session for the file, every reading sharing it through a
//! `OnceLock`, on the shape `tests/a_signature_follows_the_from_account.rs`
//! uses: the readings are taken once and each test asserts on them. Runs under
//! `WIXEN_NO_AUDIO` as CI does, and on a temporary data directory in
//! `WIXEN_MAIL_DATA`, never a person's profile.

#![cfg(windows)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::reordering::Move;
use wixen_mail::application::tagging::{self, TO_BEGIN_WITH};
use wixen_mail::data::message_cache::{MessageCache, Tag};
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::managers::save_what_the_tag_manager_returned;
use wixen_mail::presentation::wx_app::{
    WxMailApp, a_label_key_the_menu_does_not_answer, put_the_labels_on_the_menu,
};
use wixen_mail::presentation::wx_managers::{
    ManagerState, TagEntry, build_tag_manager, move_the_chosen_row, populate_tags,
};
use wxdragon::prelude::*;

type Harvest = BTreeMap<&'static str, String>;

/// An account whose labels were made the way the first key makes them.
const FRESH: &str = "acct-fresh";
/// An account whose labels were written before labels had a place.
const BEFORE: &str = "acct-before";
/// An account with more labels than there are keys.
const MANY: &str = "acct-many";

#[link(name = "user32")]
unsafe extern "system" {
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
}

/// commctrl.h: `LVM_FIRST + 115`.
const LVM_GETITEMTEXTW: u32 = 0x1000 + 115;

/// commctrl.h's `LVITEMW`, of which `LVM_GETITEMTEXTW` reads the sub-item,
/// the buffer and its length.
#[repr(C)]
struct ListViewItem {
    mask: u32,
    item: i32,
    sub_item: i32,
    state: u32,
    state_mask: u32,
    text: *mut u16,
    text_max: i32,
    image: i32,
    param: isize,
    indent: i32,
    group_id: i32,
    columns: u32,
    column_list: *mut u32,
    column_formats: *mut i32,
    group: i32,
}

/// One cell of a live list, read from the list itself, whole: not through
/// `ListCtrl::get_item_text`, which loses a cell's last character
/// (`tests/manager_dialog_labels.rs` measured it).
fn cell(list: &ListCtrl, row: i64, column: i32) -> String {
    let mut buffer = [0u16; 512];
    let mut item = ListViewItem {
        mask: 0,
        item: row as i32,
        sub_item: column,
        state: 0,
        state_mask: 0,
        text: buffer.as_mut_ptr(),
        text_max: buffer.len() as i32,
        image: 0,
        param: 0,
        indent: 0,
        group_id: 0,
        columns: 0,
        column_list: std::ptr::null_mut(),
        column_formats: std::ptr::null_mut(),
        group: 0,
    };
    // SAFETY: a live list on this thread; the item and its buffer outlive the
    // call, and the length handed over is the buffer's.
    let length = unsafe {
        SendMessageW(
            list.get_handle() as isize,
            LVM_GETITEMTEXTW,
            row as usize,
            &mut item as *mut ListViewItem as isize,
        )
    };
    String::from_utf16_lossy(&buffer[..length.clamp(0, buffer.len() as isize) as usize])
}

/// The Label Manager's rows as "Label | Key", one a line.
fn the_managers_rows(list: &ListCtrl) -> String {
    (0..list.get_item_count() as i64)
        .map(|row| format!("{} | {}", cell(list, row, 0), cell(list, row, 1)))
        .collect::<Vec<_>>()
        .join("\n")
}

// ── The store ─────────────────────────────────────────────────────────────

fn a_label(cache: &MessageCache, account_id: &str, name: &str) -> Result<(), String> {
    cache
        .create_tag(&Tag {
            id: format!("{account_id}:{name}"),
            account_id: account_id.to_string(),
            name: name.to_string(),
            color: "#FF0000".to_string(),
            created_at: "2026-09-24T00:00:00Z".to_string(),
            keyword: None,
        })
        .map_err(|e| format!("a label for {account_id}: {e}"))
}

/// The store, built the way each account's labels came to be, then opened
/// again so the pass that numbers rows from earlier builds has run.
fn a_store(at: &Path) -> Result<MessageCache, String> {
    {
        let cache = MessageCache::new(at.to_path_buf(), None).map_err(|e| format!("{e}"))?;
        for label in TO_BEGIN_WITH {
            a_label(&cache, FRESH, label.name)?;
            a_label(&cache, BEFORE, label.name)?;
        }
        for n in 1..=11 {
            a_label(&cache, MANY, &format!("Label {n}"))?;
        }
    }
    // Written by a build before labels had a place: no position at all.
    let written_before = rusqlite::Connection::open(at.join("message_cache.db"))
        .map_err(|e| format!("the store's file: {e}"))?;
    written_before
        .execute(
            "UPDATE tags SET position = NULL WHERE account_id = ?1",
            [BEFORE],
        )
        .map_err(|e| format!("the rows from before: {e}"))?;
    drop(written_before);
    MessageCache::new(at.to_path_buf(), None).map_err(|e| format!("{e}"))
}

fn the_labels(cache: &MessageCache, account_id: &str) -> Vec<Tag> {
    cache.get_tags_for_account(account_id).unwrap_or_default()
}

fn as_the_window_loads_them(labels: &[Tag]) -> Vec<(String, String)> {
    labels
        .iter()
        .map(|tag| (tag.id.clone(), tag.name.clone()))
        .collect()
}

// ── The menus ─────────────────────────────────────────────────────────────

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

fn the_label_submenu(bar: &MenuBar) -> Option<Menu> {
    (0..bar.get_menu_count())
        .filter_map(|at| bar.get_menu(at))
        .find_map(|menu| submenu_labelled(&menu, "&Label"))
}

fn the_label_menu_now(frame: &Frame) -> String {
    frame
        .get_menu_bar()
        .and_then(|bar| the_label_submenu(&bar))
        .map(|menu| the_items(&menu))
        .unwrap_or_else(|| "no Label submenu".to_string())
}

fn the_tools_menu(bar: &MenuBar) -> String {
    usize::try_from(bar.find_menu("Tools"))
        .ok()
        .and_then(|at| bar.get_menu(at))
        .map(|menu| the_items(&menu))
        .unwrap_or_else(|| "no Tools menu".to_string())
}

fn at_number_says(labels: &[Tag], number: usize) -> String {
    tagging::at_number(labels, number)
        .map(|tag| tag.name.clone())
        .unwrap_or_else(|| "nothing".to_string())
}

// ── The session ───────────────────────────────────────────────────────────

fn read_the_menus(frame: &Frame, cache: &MessageCache, harvest: &mut Harvest) {
    frame.set_menu_bar(WxMailApp::build_menu_bar());
    harvest.insert("the menu as the window starts", the_label_menu_now(frame));
    if let Some(bar) = frame.get_menu_bar() {
        harvest.insert("the Tools menu", the_tools_menu(&bar));
    }

    let fresh = the_labels(cache, FRESH);
    put_the_labels_on_the_menu(frame, &as_the_window_loads_them(&fresh));
    harvest.insert("the fresh account's menu", the_label_menu_now(frame));
    if let Some(bar) = frame.get_menu_bar() {
        harvest.insert(
            "Ctrl+5 and Ctrl+6 with five labels",
            format!(
                "{} {}",
                a_label_key_the_menu_does_not_answer(&bar, 5),
                a_label_key_the_menu_does_not_answer(&bar, 6),
            ),
        );
    }

    let before = the_labels(cache, BEFORE);
    put_the_labels_on_the_menu(frame, &as_the_window_loads_them(&before));
    harvest.insert("the account from before's menu", the_label_menu_now(frame));
    harvest.insert(
        "the account from before's second key",
        at_number_says(&before, 2),
    );

    // Work renamed Employer, and Later moved to the top, both written to the
    // store the way the Label Manager writes them.
    let mut work = fresh
        .iter()
        .find(|tag| tag.name == "Work")
        .cloned()
        .unwrap_or_else(|| fresh[0].clone());
    work.name = "Employer".to_string();
    let _ = cache.update_tag(&work);
    let mut order: Vec<String> = fresh.iter().map(|tag| tag.id.clone()).collect();
    order.rotate_right(1);
    let _ = cache.put_labels_in_order(FRESH, &order);
    let changed = the_labels(cache, FRESH);
    put_the_labels_on_the_menu(frame, &as_the_window_loads_them(&changed));
    harvest.insert(
        "the menu after a rename and a move",
        the_label_menu_now(frame),
    );
    harvest.insert(
        "the second key after a rename and a move",
        at_number_says(&changed, 2),
    );

    let many = the_labels(cache, MANY);
    put_the_labels_on_the_menu(frame, &as_the_window_loads_them(&many));
    harvest.insert("eleven labels' menu", the_label_menu_now(frame));
}

fn read_the_manager(
    frame: &Frame,
    a11y: &Arc<Accessibility>,
    cache: &MessageCache,
    harvest: &mut Harvest,
) {
    let stored = the_labels(cache, BEFORE);
    let rows: Vec<TagEntry> = stored
        .iter()
        .map(|tag| TagEntry {
            id: tag.id.clone(),
            name: tag.name.clone(),
            color: tag.color.clone(),
        })
        .collect();
    let widgets = build_tag_manager(frame, &rows, None);
    harvest.insert(
        "the manager's title",
        widgets.dialog.get_label().unwrap_or_default(),
    );
    harvest.insert("the manager's rows", the_managers_rows(&widgets.list));

    // The cursor on the second row, Later, and Move Down, as Alt+Shift+Down
    // and the Move Down button both do.
    let state = Rc::new(RefCell::new(ManagerState {
        working: rows,
        changed: false,
    }));
    widgets.list.set_item_state(
        1,
        ListItemState::Selected | ListItemState::Focused,
        ListItemState::Selected | ListItemState::Focused,
    );
    move_the_chosen_row(
        &state,
        &widgets.list,
        &widgets.status,
        a11y,
        populate_tags,
        |row: &TagEntry| row.name.clone(),
        Move::Down,
    );
    harvest.insert(
        "the manager's rows after a move",
        the_managers_rows(&widgets.list),
    );
    harvest.insert("what the move said", widgets.status.get_label());

    let handed_back = state.borrow().working.clone();
    let failures = save_what_the_tag_manager_returned(cache, BEFORE, &stored, handed_back);
    harvest.insert(
        "the order stored after the manager closes",
        match failures.is_empty() {
            true => the_labels(cache, BEFORE)
                .iter()
                .map(|tag| tag.name.clone())
                .collect::<Vec<_>>()
                .join(", "),
            false => failures.join("; "),
        },
    );
    widgets.dialog.destroy();
}

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
            let taken = (|| {
                let frame = Frame::builder()
                    .with_title("The label menu says the labels an account has, the reading")
                    .build();
                let a11y = Arc::new(
                    Accessibility::new().map_err(|why| format!("accessibility: {why:?}"))?,
                );
                let cache = a_store(&store_at)?;
                let mut harvest = Harvest::new();
                read_the_menus(&frame, &cache, &mut harvest);
                read_the_manager(&frame, &a11y, &cache, &mut harvest);
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

// ── The checks, each of which a companion hands a wrong state ─────────────

/// The Label submenu offers exactly these labels, each with the key the
/// menu's rule gives it, then Remove every label and Edit Labels.
fn the_menu_offers(menu: &str, labels: &[&str]) -> Result<(), String> {
    let mut wanted: Vec<String> = labels
        .iter()
        .enumerate()
        .map(|(at, name)| match tagging::key_for(at + 1) {
            Some(key) => format!("{name}\t{key}"),
            None => name.to_string(),
        })
        .collect();
    wanted.extend(
        [
            "---",
            "&Remove every label\tCtrl+0",
            "---",
            "&Edit Labels...",
        ]
        .map(str::to_string),
    );
    match menu.lines().eq(wanted.iter().map(String::as_str)) {
        true => Ok(()),
        false => Err(format!(
            "the Label menu read\n{menu}\nand should have read\n{}",
            wanted.join("\n")
        )),
    }
}

/// The name on the menu's line at this number is the label the key applies.
fn the_key_applies_what_the_menu_shows(
    menu: &str,
    number: usize,
    the_key_applies: &str,
) -> Result<(), String> {
    let shown = menu
        .lines()
        .nth(number - 1)
        .and_then(|line| line.split('\t').next())
        .unwrap_or_default();
    match shown == the_key_applies {
        true => Ok(()),
        false => Err(format!(
            "the menu shows {shown:?} at {number} and the key applies {the_key_applies:?}"
        )),
    }
}

// ── The menu and the keys ─────────────────────────────────────────────────

#[test]
fn test_the_menu_as_the_window_starts_offers_the_five_an_account_starts_with() {
    the_menu_offers(
        reading("the menu as the window starts"),
        &["Important", "Work", "Personal", "To Do", "Later"],
    )
    .unwrap();
}

#[test]
fn test_the_menu_says_an_accounts_labels_in_the_order_it_keeps() {
    the_menu_offers(
        reading("the fresh account's menu"),
        &["Important", "Work", "Personal", "To Do", "Later"],
    )
    .unwrap();
}

#[test]
fn test_the_second_key_applies_the_label_the_menu_shows_second() {
    // The account from before, whose labels are not in the order an account
    // starts with: the case where the menu and the keys used to disagree.
    the_key_applies_what_the_menu_shows(
        reading("the account from before's menu"),
        2,
        reading("the account from before's second key"),
    )
    .unwrap();
}

#[test]
fn test_labels_from_before_keep_the_order_their_keys_applied() {
    // A build before this one applied them in name order, so Ctrl+2 goes on
    // applying Later, and the menu now says so.
    the_menu_offers(
        reading("the account from before's menu"),
        &["Important", "Later", "Personal", "To Do", "Work"],
    )
    .unwrap();
}

#[test]
fn test_a_renamed_and_moved_label_is_what_the_menu_says_next() {
    the_menu_offers(
        reading("the menu after a rename and a move"),
        &["Later", "Important", "Employer", "Personal", "To Do"],
    )
    .unwrap();
}

#[test]
fn test_the_key_follows_a_move() {
    the_key_applies_what_the_menu_shows(
        reading("the menu after a rename and a move"),
        2,
        reading("the second key after a rename and a move"),
    )
    .unwrap();
}

#[test]
fn test_a_tenth_label_is_on_the_menu_without_a_key() {
    let names: Vec<String> = (1..=11).map(|n| format!("Label {n}")).collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    the_menu_offers(reading("eleven labels' menu"), &names).unwrap();
}

#[test]
fn test_a_key_past_the_last_label_is_one_the_menu_does_not_answer() {
    // Ctrl+6 with five labels has no item to carry it, so the message list
    // answers it with "There is no label 6" rather than silence.
    assert_eq!(reading("Ctrl+5 and Ctrl+6 with five labels"), "false true");
}

#[test]
fn test_tools_says_labels_and_nothing_says_tags() {
    let tools = reading("the Tools menu");
    assert!(tools.lines().any(|item| item == "Lab&els..."), "{tools}");
    assert!(!tools.to_lowercase().contains("tag"), "{tools}");
}

// ── The Label Manager ─────────────────────────────────────────────────────

#[test]
fn test_the_manager_is_the_label_manager_and_says_each_labels_key() {
    assert_eq!(reading("the manager's title"), "Label Manager");
    assert_eq!(
        reading("the manager's rows"),
        "Important | Ctrl+1\nLater | Ctrl+2\nPersonal | Ctrl+3\nTo Do | Ctrl+4\nWork | Ctrl+5"
    );
}

#[test]
fn test_moving_a_label_in_the_manager_says_where_it_went() {
    assert_eq!(
        reading("the manager's rows after a move"),
        "Important | Ctrl+1\nPersonal | Ctrl+2\nLater | Ctrl+3\nTo Do | Ctrl+4\nWork | Ctrl+5"
    );
    assert_eq!(reading("what the move said"), "Later, 3 of 5.");
}

#[test]
fn test_the_order_the_manager_hands_back_is_the_order_stored() {
    assert_eq!(
        reading("the order stored after the manager closes"),
        "Important, Personal, Later, To Do, Work"
    );
}

// ── Companions ────────────────────────────────────────────────────────────

#[test]
fn test_companion_a_menu_built_from_the_five_an_account_starts_with_is_refused() {
    // The menu until 12-10: the five an account starts with and four empty
    // slots, whatever the account's labels were.
    let the_old_menu = "Important\tCtrl+1\nWork\tCtrl+2\nPersonal\tCtrl+3\nTo Do\tCtrl+4\n\
                        Later\tCtrl+5\nLabel\tCtrl+6\nLabel\tCtrl+7\nLabel\tCtrl+8\n\
                        Label\tCtrl+9\n---\n&Remove every label\tCtrl+0";
    assert!(
        the_menu_offers(
            the_old_menu,
            &["Later", "Important", "Employer", "Personal", "To Do"]
        )
        .is_err()
    );
}

#[test]
fn test_companion_a_key_applying_by_name_order_is_refused() {
    // The keys until 12-10 read the labels by name, so the second was Later
    // while the menu said Work.
    let by_name = "Important\tCtrl+1\nWork\tCtrl+2\nPersonal\tCtrl+3";
    assert!(the_key_applies_what_the_menu_shows(by_name, 2, "Later").is_err());
}

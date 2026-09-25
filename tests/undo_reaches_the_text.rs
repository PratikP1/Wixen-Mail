//! Edit, Undo and Edit, Redo reach the box somebody is typing in, and the
//! Edit menu says when there is nothing for them to do (#47).
//!
//! The tester on 2026-09-15: "There are no general undo/redo commands that
//! provide corresponding functionality." Until 13-01 the Edit menu held Undo
//! Send, the clipboard four and Search, and Ctrl+Z reached the box only
//! because nothing on the menu had taken the key.
//!
//! **What is read.** A frame given the main window's own menu bar and two real
//! boxes, one line and several lines, each keeping a history through
//! `text_history_keys::keep_a_history` as the main window's boxes do. Words
//! typed through `write_text`, which is how this program's Paste writes, then
//! `text_undo::undo` and `redo` as the Edit menu calls them, each answer and
//! each value read back from the box. Then the menu: the box given focus,
//! `WM_INITMENUPOPUP` sent to the frame with the Edit menu's handle, which is
//! what Windows sends as the menu drops, and Undo and Redo read off the bar;
//! then `WM_UNINITMENUPOPUP` and the two read again. The first items on Edit
//! read by their labels.
//!
//! Until 13-05 the boxes were read through their own one step. Two readings
//! changed with the history: after the one step is undone the open menu now
//! greys Undo, because there is nothing further back, where the box's one
//! step offered Undo again as a way to redo; and what the program writes goes
//! through `set_anew`, because a plain write is a change the history keeps.
//! The several steps themselves are read in `tests/several_steps_come_back.rs`.
//!
//! **Measured on a built control, 2026-09-24,** by sending `EM_UNDO` straight
//! to the box, so the measurement does not lean on the module it describes:
//!
//! - Cut as this program made it until 13-01, `replace(from, to, "")` over the
//!   chosen words, then Undo: the words stay gone, in a one-line box and in a
//!   box of several lines. `replace` is a removal and then an empty write, and
//!   a box that remembers one step remembers the empty write, so Undo took
//!   back nothing and would have said "Undone" over a box that did not
//!   change.
//! - Cut as it is made now, `text_undo::remove_the_selection`, which writes
//!   nothing over the chosen words, then Undo: the words come back, in both
//!   boxes. `WM_CLEAR` sent to the box measured the same; the write was kept
//!   because it goes through wxWidgets rather than around it.
//! - Paste as this program makes it, `write_text`, then Undo: the pasted words
//!   go and the words before them stay.
//! - Words the program writes itself with `set_value`, such as a note opened
//!   into the body: the box's own one step has nothing to undo afterwards. The
//!   history keeps no such step either, because those words go through
//!   `set_anew`, so Ctrl+Z after choosing another note does not put the first
//!   note's words back.
//!
//! **What is not read.** That Ctrl+Z in the running window reaches
//! `do_an_edit_command` and speaks: that is the menu's accelerator and one
//! arm, reached by the scan and the NVDA run of the running program.
//!
//! **Companions.** The menu left greyed after it closes, which would swallow
//! Ctrl+Z in silence (`framecmn.cpp` returns early for a greyed item's key),
//! and Redo offered after typing, which would put a step back onto words it
//! was never taken from. Each is
//! handed to its check and refused, so a check that passes is one that could
//! have failed.
//!
//! One window session for the file, every reading sharing it through a
//! `OnceLock`, on the shape `tests/the_label_menu_says_the_labels_an_account_has.rs`
//! uses: the readings are taken once and each test asserts on them. Runs
//! under `WIXEN_NO_AUDIO` as CI does, and on a temporary data directory in
//! `WIXEN_MAIL_DATA`, never a person's profile.

#![cfg(windows)]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::presentation::text_history_keys::{self, keep_a_history};
use wixen_mail::presentation::text_undo::{self, Redone, Undone};
use wixen_mail::presentation::wx_app::{WxMailApp, keep_undo_and_redo_honest_on_the_menu};
use wxdragon::prelude::*;

type Harvest = BTreeMap<&'static str, String>;

#[link(name = "user32")]
unsafe extern "system" {
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
    fn GetMenu(hwnd: isize) -> isize;
    fn GetSubMenu(menu: isize, position: i32) -> isize;
}

/// winuser.h.
const WM_INITMENUPOPUP: u32 = 0x0117;
const WM_UNINITMENUPOPUP: u32 = 0x0125;
/// winuser.h: `EM_UNDO`, for the measurement, which does not go through the
/// module it measures.
const EM_UNDO: u32 = 0x00C7;

const UNDO: &str = "&Undo\tCtrl+Z";
const REDO: &str = "&Redo\tCtrl+Y";

fn the_edit_menu(bar: &MenuBar) -> Option<(usize, Menu)> {
    let at = usize::try_from(bar.find_menu("Edit")).ok()?;
    Some((at, bar.get_menu(at)?))
}

/// The first four items on Edit, one a line, a separator as "---".
fn the_first_items_on_edit(bar: &MenuBar) -> String {
    let Some((_, edit)) = the_edit_menu(bar) else {
        return "no Edit menu".to_string();
    };
    edit.get_menu_items()
        .iter()
        .take(4)
        .map(|item| match item.get_label() {
            label if label.is_empty() => "---".to_string(),
            label => label,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn offered(bar: &MenuBar, label: &str) -> &'static str {
    let item = the_edit_menu(bar).and_then(|(_, edit)| {
        edit.get_menu_items()
            .into_iter()
            .find(|item| item.get_label() == label)
    });
    match item {
        None => "missing",
        Some(item) if bar.is_item_enabled(item.get_item_id()) => "offered",
        Some(_) => "greyed",
    }
}

/// Undo and Redo as the bar has them now, "Undo offered, Redo greyed".
fn undo_and_redo_now(frame: &Frame) -> String {
    match frame.get_menu_bar() {
        Some(bar) => format!("Undo {}, Redo {}", offered(&bar, UNDO), offered(&bar, REDO)),
        None => "no menu bar".to_string(),
    }
}

/// What Windows sends as the Edit menu drops, then as it closes: the two
/// readings, open and closed.
fn open_and_close_edit(frame: &Frame) -> (String, String) {
    let window = frame.get_handle() as isize;
    let at = frame
        .get_menu_bar()
        .and_then(|bar| the_edit_menu(&bar))
        .map_or(0, |(at, _)| at as i32);
    // SAFETY: the frame is a live window on this thread; the two calls take
    // and return handles and no pointer.
    let edit = unsafe { GetSubMenu(GetMenu(window), at) };
    // SAFETY: the same window; the menu handle is a number to the message.
    unsafe { SendMessageW(window, WM_INITMENUPOPUP, edit as usize, at as isize) };
    let open = undo_and_redo_now(frame);
    // SAFETY: as above.
    unsafe { SendMessageW(window, WM_UNINITMENUPOPUP, edit as usize, 0) };
    (open, undo_and_redo_now(frame))
}

fn undone(answer: Undone) -> String {
    match answer {
        Undone::Done => "undone".to_string(),
        Undone::NothingToUndo => "nothing to undo".to_string(),
    }
}

fn redone(answer: Redone) -> String {
    match answer {
        Redone::Done => "redone".to_string(),
        Redone::NothingToRedo => "nothing to redo".to_string(),
    }
}

fn the_box_after_a_raw_undo(box_: &TextCtrl) -> String {
    // SAFETY: a live box on this thread; the message takes no pointer.
    unsafe { SendMessageW(box_.get_handle() as isize, EM_UNDO, 0, 0) };
    box_.get_value()
}

/// "beta" chosen in "alpha beta", then cut the way given, then the box's own
/// undo: what the cut left and what the undo left.
fn cut_then_undo(box_: &TextCtrl, cut: impl Fn(&TextCtrl)) -> String {
    box_.set_value("alpha beta");
    box_.set_selection(6, 10);
    cut(box_);
    let after_cut = box_.get_value();
    format!(
        "cut left {after_cut:?}, undo left {:?}",
        the_box_after_a_raw_undo(box_)
    )
}

/// Cut as it was until 13-01: the chosen words replaced with nothing.
fn the_old_cut(box_: &TextCtrl) {
    let (from, to) = box_.get_selection();
    box_.replace(from, to, "");
}

fn take_the_readings(frame: &Frame, harvest: &mut Harvest) {
    frame.set_menu_bar(WxMailApp::build_menu_bar());
    if let Some(bar) = frame.get_menu_bar() {
        harvest.insert("the first items on Edit", the_first_items_on_edit(&bar));
    }
    let panel = Panel::builder(frame).build();
    let several = TextCtrl::builder(&panel)
        .with_style(TextCtrlStyle::MultiLine)
        .with_pos(Point::new(0, 0))
        .with_size(Size::new(300, 120))
        .build();
    let one = TextCtrl::builder(&panel)
        .with_pos(Point::new(0, 130))
        .with_size(Size::new(300, 30))
        .build();
    keep_a_history(&several);
    keep_a_history(&one);
    keep_undo_and_redo_honest_on_the_menu(frame, vec![several, one]);
    // Shown, because focus is asked of the boxes the way the window asks.
    frame.show(true);

    // Typed, then the menu with nothing undone yet.
    several.set_focus();
    harvest.insert("the box held focus", several.has_focus().to_string());
    several.write_text("one two");
    let (open, closed) = open_and_close_edit(frame);
    harvest.insert("the menu open after typing", open);
    harvest.insert("the menu closed after typing", closed);

    // Undo, the menu, then Redo.
    harvest.insert("undo after typing", undone(text_undo::undo(&several)));
    harvest.insert("the box after undo", several.get_value());
    let (open, closed) = open_and_close_edit(frame);
    harvest.insert("the menu open after an undo", open);
    harvest.insert("the menu closed after an undo", closed);
    harvest.insert("redo after undo", redone(text_undo::redo(&several)));
    harvest.insert("the box after redo", several.get_value());

    // Undo, then typing, then Redo.
    let _ = text_undo::undo(&several);
    several.write_text(" three");
    let before = several.get_value();
    harvest.insert(
        "can redo after typing since the undo",
        text_history_keys::can_redo_in(&several).to_string(),
    );
    harvest.insert(
        "redo after typing since the undo",
        redone(text_undo::redo(&several)),
    );
    harvest.insert(
        "the box kept its typing",
        (several.get_value() == before).to_string(),
    );

    // A fresh box: nothing to undo, and the menu says so.
    one.set_focus();
    harvest.insert("undo on a fresh box", undone(text_undo::undo(&one)));
    let (open, closed) = open_and_close_edit(frame);
    harvest.insert("the menu open on a fresh box", open);
    harvest.insert("the menu closed on a fresh box", closed);

    // What the program writes itself starts a box with nothing to undo.
    text_history_keys::set_anew(&several, "a note opened into the body");
    harvest.insert(
        "can undo after the program writes the box",
        text_history_keys::can_undo_in(&several).to_string(),
    );

    // The measurements.
    harvest.insert(
        "the old cut then undo, one line",
        cut_then_undo(&one, the_old_cut),
    );
    harvest.insert(
        "the old cut then undo, several lines",
        cut_then_undo(&several, the_old_cut),
    );
    harvest.insert(
        "cut then undo, one line",
        cut_then_undo(&one, text_undo::remove_the_selection),
    );
    harvest.insert(
        "cut then undo, several lines",
        cut_then_undo(&several, text_undo::remove_the_selection),
    );
    several.set_value("alpha ");
    several.set_insertion_point_end();
    // Paste as `do_an_edit_command` makes it.
    several.write_text("pasted");
    harvest.insert(
        "paste then undo",
        format!(
            "paste left {:?}, undo left {:?}",
            several.get_value(),
            the_box_after_a_raw_undo(&several)
        ),
    );
}

fn take_the_harvest() -> Result<Harvest, String> {
    let data = tempfile::tempdir().map_err(|e| format!("a data directory: {e}"))?;
    // SAFETY: set before the window session starts any thread, and nothing
    // has read either yet.
    unsafe {
        std::env::set_var("WIXEN_MAIL_DATA", data.path());
        std::env::set_var("WIXEN_NO_AUDIO", "1");
    }
    let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
    let result = {
        let outcome = outcome.clone();
        wxdragon::main(move |app| {
            let frame = Frame::builder()
                .with_title("Undo reaches the text, the reading")
                .with_size(Size::new(420, 260))
                .build();
            let mut harvest = Harvest::new();
            take_the_readings(&frame, &mut harvest);
            frame.destroy();
            if let Ok(mut slot) = outcome.lock() {
                *slot = Some(Ok(harvest));
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

/// Both offered once the menu has closed, so the keys reach the handler and
/// speak.
fn both_offered_once_closed(reading: &str) -> Result<(), String> {
    match reading == "Undo offered, Redo offered" {
        true => Ok(()),
        false => Err(format!(
            "after the menu closed the bar read {reading:?}, and a greyed item's key \
             does nothing and says nothing"
        )),
    }
}

/// Redo refused once something was typed after the Undo.
fn redo_refused_after_typing(can_redo: &str) -> Result<(), String> {
    match can_redo == "false" {
        true => Ok(()),
        false => Err(format!(
            "Redo was offered after typing since the undo ({can_redo}), and it would \
             put a step back onto words it was never taken from"
        )),
    }
}

// ── The box ───────────────────────────────────────────────────────────────

#[test]
fn test_undo_takes_back_what_was_typed() {
    assert_eq!(reading("undo after typing"), "undone");
    assert_eq!(reading("the box after undo"), "");
}

#[test]
fn test_redo_right_after_an_undo_puts_it_back() {
    assert_eq!(reading("redo after undo"), "redone");
    assert_eq!(reading("the box after redo"), "one two");
}

#[test]
fn test_redo_is_refused_once_something_is_typed_after_the_undo() {
    redo_refused_after_typing(reading("can redo after typing since the undo")).unwrap();
    assert_eq!(
        reading("redo after typing since the undo"),
        "nothing to redo"
    );
    assert_eq!(reading("the box kept its typing"), "true");
}

#[test]
fn test_undo_in_a_box_with_nothing_to_undo_says_so() {
    assert_eq!(reading("undo on a fresh box"), "nothing to undo");
}

#[test]
fn test_what_the_program_writes_leaves_nothing_to_undo() {
    // Choosing another note writes its words into the body. Ctrl+Z after that
    // must not put the first note's words back.
    assert_eq!(
        reading("can undo after the program writes the box"),
        "false"
    );
}

// ── The menu ──────────────────────────────────────────────────────────────

#[test]
fn test_edit_opens_with_undo_and_redo_then_undo_send() {
    assert_eq!(
        reading("the first items on Edit"),
        "&Undo\tCtrl+Z\n&Redo\tCtrl+Y\nUndo Se&nd\tCtrl+Shift+Z\n---"
    );
}

#[test]
fn test_the_open_menu_greys_redo_when_nothing_was_undone() {
    assert_eq!(reading("the box held focus"), "true");
    assert_eq!(
        reading("the menu open after typing"),
        "Undo offered, Redo greyed"
    );
}

#[test]
fn test_the_open_menu_offers_redo_right_after_an_undo() {
    // Undo greyed: the one step typed has been taken back and there is
    // nothing further back. The box's own one step offered Undo here, as a
    // second way to redo.
    assert_eq!(
        reading("the menu open after an undo"),
        "Undo greyed, Redo offered"
    );
}

#[test]
fn test_the_open_menu_greys_both_in_a_box_with_nothing_to_undo() {
    assert_eq!(
        reading("the menu open on a fresh box"),
        "Undo greyed, Redo greyed"
    );
}

#[test]
fn test_both_are_offered_again_whenever_the_menu_closes() {
    for closed in [
        "the menu closed after typing",
        "the menu closed after an undo",
        "the menu closed on a fresh box",
    ] {
        both_offered_once_closed(reading(closed)).unwrap_or_else(|why| panic!("{closed}: {why}"));
    }
}

// ── The measurements ──────────────────────────────────────────────────────

#[test]
fn test_cut_then_undo_puts_the_words_back() {
    for box_ in ["one line", "several lines"] {
        assert_eq!(
            reading(&format!("cut then undo, {box_}")),
            "cut left \"alpha \", undo left \"alpha beta\"",
            "{box_}"
        );
    }
}

#[test]
fn test_the_old_cut_left_nothing_for_undo_to_put_back() {
    // Kept measuring, as `tests/text_selection_offsets.rs` keeps measuring the
    // rule it replaced: the day this stops holding, the reason Cut goes
    // through `remove_the_selection` has gone, and that should be said.
    for box_ in ["one line", "several lines"] {
        assert_eq!(
            reading(&format!("the old cut then undo, {box_}")),
            "cut left \"alpha \", undo left \"alpha \"",
            "{box_}"
        );
    }
}

#[test]
fn test_paste_then_undo_takes_the_paste_away() {
    assert_eq!(
        reading("paste then undo"),
        "paste left \"alpha pasted\", undo left \"alpha \""
    );
}

// ── Companions ────────────────────────────────────────────────────────────

#[test]
fn test_companion_a_menu_left_greyed_after_it_closes_is_refused() {
    assert!(both_offered_once_closed("Undo greyed, Redo greyed").is_err());
    assert!(both_offered_once_closed("Undo offered, Redo greyed").is_err());
}

#[test]
fn test_companion_a_redo_offered_after_typing_is_refused() {
    assert!(redo_refused_after_typing("true").is_err());
}

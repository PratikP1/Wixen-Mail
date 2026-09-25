//! A text box keeps several steps of undo, and gives them back one at a time
//! (#47: "a multi-step history where the native control gives one step").
//!
//! **What is read.** A frame with one real box of several lines, keeping a
//! history through `text_history_keys::keep_a_history` as the main window's
//! boxes do. Three words typed a character at a time, each `WM_CHAR` sent to
//! the box, which is what a key press becomes once Windows has translated it;
//! `SendMessageW` returns once the box has taken the character and raised its
//! change, so each is done before the next. Then `undo_in` three times with the
//! value read after each, and `redo_in` three times. Then a value the program
//! writes, through `set_anew`, and whether there is anything to undo after it.
//!
//! **The restore raises the change.** A counter bound on the box's text event
//! after the history's own, read before and after an undo, because the
//! contacts search runs on that event and has to run again on restored words.
//! And whether Redo is still offered after that restore: a history that
//! recorded its own restore as a new step would have nothing left to redo.
//!
//! **Ctrl+Z in the box itself**, which is how a dialog without an Edit menu
//! reaches the history. Control is set down in this thread's keyboard state,
//! which is what wxWidgets reads for a key's modifiers, never through
//! `SendInput`, which would press a key for the whole desktop. Then
//! `WM_KEYDOWN` and the `WM_CHAR` Windows makes of it are sent to the box, the
//! value is read once, and a counter bound on the box's character event says
//! whether the character reached the box. If it had, the box's own one-step
//! undo would have run on top of the history's.
//!
//! **An editable combo box**, as the event form's Category and the contact
//! editor's Prefix and Suffix are, through the same `keep_a_history`. A combo
//! box is two windows, and the keyboard lands in the edit it holds its words
//! in, so the words are typed and Ctrl+Z is pressed at that edit, found by its
//! class under the combo box. Measured 2026-09-25 by 13-06: Ctrl+Z at the edit
//! reaches a key-down bound on the `ComboBox` itself, because wxWidgets
//! forwards the edit's keys to it, and taking it there drops the control
//! character as it does for a text box. So the history binds on the combo box
//! and never on its edit.
//!
//! **Dialogs.** A box in a `Dialog`, which has no menu bar, takes Ctrl+Z and
//! Ctrl+Y at its own key-down. The program's Check Spelling dialog, built by
//! its builder and never shown, opens holding the first suggestion, and four
//! presses of Ctrl+Z after typing walk back a step at a time and stop at the
//! suggestion; Windows' one step would put the typing back on the second.
//! And the contact editor opened on a stored contact: every box it fills
//! before it is shown has nothing to undo, because it is filled with
//! `set_anew`, and a letter typed in one gives it a step.
//!
//! **The note chosen in the list.** Read from `src/presentation/wx_app.rs`,
//! because building the whole main window here is not needed to ask how it
//! writes a note into the editor: every write in the handler goes through
//! `set_anew`, so a chosen note is never a step somebody could undo into the
//! last note's words.
//!
//! **Companions.** A history that recorded its own restore, a key-down passed
//! on after acting, and a note written with `set_value`: each is handed to its
//! check and refused, so a check that passes is one that could have failed.
//!
//! One window session for the file, every reading sharing it through a
//! `OnceLock`, on the shape `tests/undo_reaches_the_text.rs` uses. Runs under
//! `WIXEN_NO_AUDIO` as CI does, and on a temporary data directory in
//! `WIXEN_MAIL_DATA`, never a person's profile.

#![cfg(windows)]

use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use wixen_mail::application::spell_session;
use wixen_mail::application::words::{TextNode, words_in};
use wixen_mail::presentation::accessibility::Accessibility;
use wixen_mail::presentation::text_history_keys::{
    can_redo_in, can_undo_in, keep_a_history, redo_in, set_anew, undo_in,
};
use wixen_mail::presentation::wx_compose;
use wixen_mail::presentation::wx_managers::{self, ContactEntry};
use wxdragon::prelude::*;

type Harvest = BTreeMap<&'static str, String>;

#[link(name = "user32")]
unsafe extern "system" {
    fn SendMessageW(hwnd: isize, message: u32, wparam: usize, lparam: isize) -> isize;
    fn GetKeyboardState(state: *mut u8) -> i32;
    fn SetKeyboardState(state: *const u8) -> i32;
    fn FindWindowExW(parent: isize, after: isize, class: *const u16, title: *const u16) -> isize;
}

/// winuser.h.
const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const WM_CHAR: u32 = 0x0102;
const VK_CONTROL: usize = 0x11;
/// A key press's repeat count and scan code, and its release's flags.
const PRESSED: isize = 0x0000_0001;
const RELEASED: isize = 0xC000_0001_u32 as i32 as isize;

fn handle(box_: &impl WxWidget) -> isize {
    box_.get_handle() as isize
}

/// The edit a combo box holds its words in, which is where the keyboard
/// lands: the combo box's own handle is the frame around it and the list.
fn edit_of(combo: &ComboBox) -> isize {
    let class: Vec<u16> = "Edit\0".encode_utf16().collect();
    // SAFETY: a live combo box on this thread, and a terminated class name.
    unsafe { FindWindowExW(handle(combo), 0, class.as_ptr(), std::ptr::null()) }
}

/// Each character sent to the box as a translated key press would be.
fn type_into(box_: &TextCtrl, words: &str) {
    type_into_window(handle(box_), words);
}

fn type_into_window(window: isize, words: &str) {
    for unit in words.encode_utf16() {
        // SAFETY: a live window on this thread; the message carries a
        // character and no pointer.
        unsafe { SendMessageW(window, WM_CHAR, usize::from(unit), PRESSED) };
    }
}

/// Control held while `letter` is pressed in the box: the key-down and the
/// control character Windows makes of it, then the release. Control is set
/// in this thread's keyboard state and put back afterwards.
fn press_with_control(box_: &TextCtrl, letter: u8) {
    press_with_control_in(handle(box_), letter);
}

fn press_with_control_in(window: isize, letter: u8) {
    let mut before = [0u8; 256];
    // SAFETY: the buffer is the 256 bytes the call writes.
    unsafe { GetKeyboardState(before.as_mut_ptr()) };
    let mut held = before;
    held[VK_CONTROL] = 0x80;
    // SAFETY: the buffer is the 256 bytes the call reads, for this thread.
    unsafe { SetKeyboardState(held.as_ptr()) };
    let control_character = usize::from(letter - b'A' + 1);
    // SAFETY: a live window on this thread; the messages carry numbers only.
    unsafe {
        SendMessageW(window, WM_KEYDOWN, usize::from(letter), PRESSED);
        SendMessageW(window, WM_CHAR, control_character, PRESSED);
        SendMessageW(window, WM_KEYUP, usize::from(letter), RELEASED);
    }
    // SAFETY: as above, the state read at the start.
    unsafe { SetKeyboardState(before.as_ptr()) };
}

fn counter_on(bind: impl FnOnce(Rc<Cell<u32>>)) -> Rc<Cell<u32>> {
    let count = Rc::new(Cell::new(0));
    bind(Rc::clone(&count));
    count
}

fn take_the_readings(frame: &Frame, harvest: &mut Harvest) {
    let panel = Panel::builder(frame).build();
    let box_ = TextCtrl::builder(&panel)
        .with_style(TextCtrlStyle::MultiLine)
        .with_size(Size::new(300, 120))
        .build();
    keep_a_history(&box_);
    let changes = counter_on(|count| {
        box_.on_text_changed(move |_| count.set(count.get() + 1));
    });
    let characters = counter_on(|count| {
        box_.on_char(move |_| count.set(count.get() + 1));
    });
    frame.show(true);

    // Three words, then back a step at a time, then forward.
    type_into(&box_, "one two three");
    harvest.insert("typed", box_.get_value());
    for name in ["undone once", "undone twice", "undone three times"] {
        undo_in(&box_);
        harvest.insert(name, box_.get_value());
        if name == "undone once" {
            let (from, to) = box_.get_selection();
            harvest.insert("the caret after the first undo", format!("{from}..{to}"));
        }
    }
    harvest.insert("a fourth undo did something", undo_in(&box_).to_string());
    for name in ["redone once", "redone twice", "redone three times"] {
        redo_in(&box_);
        harvest.insert(name, box_.get_value());
    }

    // A value the program writes.
    set_anew(&box_, "a note");
    harvest.insert("the box after set_anew", box_.get_value());
    harvest.insert("can undo after set_anew", can_undo_in(&box_).to_string());

    // A restore raises the change, and the history does not keep it.
    box_.set_insertion_point_end();
    type_into(&box_, "s");
    let before = changes.get();
    undo_in(&box_);
    harvest.insert("the box after undoing the s", box_.get_value());
    harvest.insert(
        "the restore raised the change",
        (changes.get() > before).to_string(),
    );
    harvest.insert(
        "redo still offered after the restore",
        can_redo_in(&box_).to_string(),
    );

    // Ctrl+Z and Ctrl+Y pressed in the box itself.
    set_anew(&box_, "");
    type_into(&box_, "alpha beta");
    characters.set(0);
    press_with_control(&box_, b'Z');
    harvest.insert("the box after Ctrl+Z", box_.get_value());
    harvest.insert(
        "the Ctrl+Z character reached the box",
        (characters.get() > 0).to_string(),
    );
    press_with_control(&box_, b'Y');
    harvest.insert("the box after Ctrl+Y", box_.get_value());

    take_the_combo_box_readings(&panel, harvest);
    take_the_dialog_readings(frame, harvest);
    if let Err(why) = take_the_contact_editor_readings(frame, harvest) {
        harvest.insert(
            "the stored contact's boxes with a step to undo",
            format!("not read: {why}"),
        );
    }
}

/// An editable combo box, as the event form's Category and the contact
/// editor's Prefix and Suffix are: typed into and pressed in at its edit,
/// which is where the keyboard lands.
fn take_the_combo_box_readings(panel: &Panel, harvest: &mut Harvest) {
    let combo = ComboBox::builder(panel)
        .with_string_choices(&["Dr."])
        .build();
    let key_downs = counter_on(|count| {
        combo.on_key_down(move |_| count.set(count.get() + 1));
    });
    keep_a_history(&combo);
    let characters = counter_on(|count| {
        combo.on_char(move |_| count.set(count.get() + 1));
    });
    let edit = edit_of(&combo);
    harvest.insert("the combo box has an edit", (edit != 0).to_string());

    type_into_window(edit, "one two three");
    harvest.insert("the combo box typed", combo.get_value());
    for name in [
        "the combo box undone once",
        "the combo box undone twice",
        "the combo box undone three times",
    ] {
        undo_in(&combo);
        harvest.insert(name, combo.get_value());
    }
    redo_in(&combo);
    harvest.insert("the combo box redone once", combo.get_value());

    set_anew(&combo, "");
    type_into_window(edit, "alpha beta");
    key_downs.set(0);
    characters.set(0);
    press_with_control_in(edit, b'Z');
    harvest.insert("the combo box after Ctrl+Z at its edit", combo.get_value());
    harvest.insert(
        "the combo box's key-down saw Ctrl+Z",
        (key_downs.get() > 0).to_string(),
    );
    harvest.insert(
        "the Ctrl+Z character reached the combo box",
        (characters.get() > 0).to_string(),
    );
    press_with_control_in(edit, b'Y');
    harvest.insert("the combo box after Ctrl+Y at its edit", combo.get_value());
}

/// A box in a dialog, which has no menu bar and so no Edit menu to take the
/// keys first: Ctrl+Z and Ctrl+Y reach the box's own key-down. Then the
/// program's own Check Spelling dialog, built by its builder and never shown:
/// its box opens holding the first suggestion, which is where its history
/// starts, and a letter typed after it is one step back to the suggestion.
fn take_the_dialog_readings(frame: &Frame, harvest: &mut Harvest) {
    let dialog = Dialog::builder(frame, "Several steps in a dialog").build();
    let box_ = TextCtrl::builder(&dialog).build();
    keep_a_history(&box_);
    type_into(&box_, "first second");
    press_with_control(&box_, b'Z');
    harvest.insert("the dialog's box after Ctrl+Z", box_.get_value());
    press_with_control(&box_, b'Y');
    harvest.insert("the dialog's box after Ctrl+Y", box_.get_value());

    let spelling =
        wx_compose::build_check_spelling_dialog(&dialog, &one_misspelling_of_world(), None);
    let replacement = spelling.replacement;
    harvest.insert("the spelling box opens holding", replacement.get_value());
    harvest.insert(
        "the spelling box can undo as it opens",
        can_undo_in(&replacement).to_string(),
    );
    replacement.set_insertion_point_end();
    type_into(&replacement, " big bad");
    for name in [
        "the spelling box after one Ctrl+Z",
        "the spelling box after two",
        "the spelling box after three",
        "the spelling box after four",
    ] {
        press_with_control(&replacement, b'Z');
        harvest.insert(name, replacement.get_value());
    }
    spelling.dialog.destroy();
    dialog.destroy();
}

/// The contact editor opened on a stored contact, built by its builder and
/// never shown. Every box it fills is filled with `set_anew`, so none has a
/// step to undo; a box that did would be emptied by the first Ctrl+Z.
fn take_the_contact_editor_readings(frame: &Frame, harvest: &mut Harvest) -> Result<(), String> {
    let a11y = Arc::new(Accessibility::new().map_err(|why| format!("Accessibility::new: {why}"))?);
    let stored = ContactEntry {
        id: "stored".to_string(),
        name: "Grace van der Berg".to_string(),
        given_name: "Grace".to_string(),
        family_name: "van der Berg".to_string(),
        name_prefix: "Dr.".to_string(),
        middle_name: String::new(),
        name_suffix: String::new(),
        nickname: "Gee".to_string(),
        company: String::new(),
        department: String::new(),
        job_title: String::new(),
        emails: Vec::new(),
        phones: Vec::new(),
        addresses: Vec::new(),
        birthday: String::new(),
        website: String::new(),
        relationship: String::new(),
        notes: "Met at the library.".to_string(),
        custom_fields: Vec::new(),
        avatar_url: String::new(),
        favorite: false,
    };
    let editor = wx_managers::build_contact_edit_dialog(frame, Some(&stored), None, &a11y);
    let undoable: Vec<&str> = [
        ("name", can_undo_in(&editor.name_f)),
        ("prefix", can_undo_in(&editor.prefix_f)),
        ("given", can_undo_in(&editor.given_f)),
        ("family", can_undo_in(&editor.family_f)),
        ("nickname", can_undo_in(&editor.nick_f)),
        ("notes", can_undo_in(&editor.notes_f)),
    ]
    .into_iter()
    .filter_map(|(field, can)| can.then_some(field))
    .collect();
    harvest.insert(
        "the stored contact's boxes with a step to undo",
        undoable.join(", "),
    );
    editor.nick_f.set_insertion_point_end();
    type_into(&editor.nick_f, "e");
    harvest.insert(
        "the nickname can undo once typed in",
        can_undo_in(&editor.nick_f).to_string(),
    );
    editor.dialog.destroy();
    Ok(())
}

/// "wrold" found by the real spell check's own path, offering "world".
fn one_misspelling_of_world() -> spell_session::Finding {
    let words = words_in(&[TextNode {
        text: "the wrold turns".to_string(),
        block: 0,
    }]);
    spell_session::findings(
        &words,
        |word| word == "wrold",
        |_| vec!["world".to_string()],
    )
    .into_iter()
    .next()
    .expect("\"wrold\" is misspelled by construction")
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
                .with_title("Several steps come back, the reading")
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

/// Redo still offered after a restore, so the restore was not kept as a step.
fn the_restore_was_not_recorded(redo_offered: &str) -> Result<(), String> {
    match redo_offered == "true" {
        true => Ok(()),
        false => Err(format!(
            "after an undo, Redo was offered: {redo_offered}. The history kept its own \
             restore as a new step, which cleared what Redo could put back"
        )),
    }
}

/// The Ctrl+Z character never reached the box, so its own undo did not run.
fn the_key_was_taken(reached: &str) -> Result<(), String> {
    match reached == "false" {
        true => Ok(()),
        false => Err(format!(
            "the Ctrl+Z character reached the box ({reached}), so its own one-step undo \
             ran on top of the history's"
        )),
    }
}

/// The note selection handler in `wx_app.rs`, from its binding to its close.
fn the_note_selection(source: &str) -> Option<&str> {
    let start = source.find("notes_cp.note_list.on_item_selected({")?;
    let length = source[start..].find("\n            });")?;
    Some(&source[start..start + length])
}

/// Every write of a chosen note goes through `set_anew`.
fn a_chosen_note_is_the_programs_own(handler: &str) -> Result<(), String> {
    let plain = handler.matches(".set_value(").count();
    let anew = handler.matches("set_anew(").count();
    match (plain, anew) {
        (0, 6) => Ok(()),
        _ => Err(format!(
            "the note selection writes {plain} values with set_value and {anew} with \
             set_anew; a note written with set_value is a step Ctrl+Z would undo into the \
             last note's words"
        )),
    }
}

fn wx_app_source() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/presentation/wx_app.rs");
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

// ── The steps ─────────────────────────────────────────────────────────────

#[test]
fn test_three_words_typed_come_back_a_word_at_a_time() {
    assert_eq!(reading("typed"), "one two three");
    assert_eq!(reading("undone once"), "one two ");
    assert_eq!(reading("undone twice"), "one ");
    assert_eq!(reading("undone three times"), "");
    assert_eq!(reading("a fourth undo did something"), "false");
}

#[test]
fn test_the_caret_comes_back_where_the_word_began() {
    assert_eq!(reading("the caret after the first undo"), "8..8");
}

#[test]
fn test_redo_puts_each_word_back_in_turn() {
    assert_eq!(reading("redone once"), "one ");
    assert_eq!(reading("redone twice"), "one two ");
    assert_eq!(reading("redone three times"), "one two three");
}

#[test]
fn test_a_value_the_program_writes_leaves_nothing_to_undo() {
    assert_eq!(reading("the box after set_anew"), "a note");
    assert_eq!(reading("can undo after set_anew"), "false");
}

#[test]
fn test_a_restore_raises_the_change_and_is_not_kept_as_a_step() {
    assert_eq!(reading("the box after undoing the s"), "a note");
    assert_eq!(reading("the restore raised the change"), "true");
    the_restore_was_not_recorded(reading("redo still offered after the restore")).unwrap();
}

#[test]
fn test_ctrl_z_in_the_box_undoes_one_step_and_the_box_does_not_undo_again() {
    assert_eq!(reading("the box after Ctrl+Z"), "alpha ");
    the_key_was_taken(reading("the Ctrl+Z character reached the box")).unwrap();
}

#[test]
fn test_ctrl_y_in_the_box_puts_the_step_back() {
    assert_eq!(reading("the box after Ctrl+Y"), "alpha beta");
}

#[test]
fn test_an_editable_combo_box_gives_back_three_words_a_word_at_a_time() {
    assert_eq!(reading("the combo box has an edit"), "true");
    assert_eq!(reading("the combo box typed"), "one two three");
    assert_eq!(reading("the combo box undone once"), "one two ");
    assert_eq!(reading("the combo box undone twice"), "one ");
    assert_eq!(reading("the combo box undone three times"), "");
    assert_eq!(reading("the combo box redone once"), "one ");
}

#[test]
fn test_ctrl_z_at_a_combo_boxs_edit_reaches_the_history_and_the_edit_does_not_undo_again() {
    assert_eq!(reading("the combo box's key-down saw Ctrl+Z"), "true");
    assert_eq!(reading("the combo box after Ctrl+Z at its edit"), "alpha ");
    the_key_was_taken(reading("the Ctrl+Z character reached the combo box")).unwrap();
    assert_eq!(
        reading("the combo box after Ctrl+Y at its edit"),
        "alpha beta"
    );
}

#[test]
fn test_a_box_in_a_dialog_takes_ctrl_z_and_ctrl_y_with_no_menu() {
    assert_eq!(reading("the dialog's box after Ctrl+Z"), "first ");
    assert_eq!(reading("the dialog's box after Ctrl+Y"), "first second");
}

#[test]
fn test_the_programs_own_dialog_box_walks_back_to_what_it_opened_holding() {
    assert_eq!(reading("the spelling box opens holding"), "world");
    assert_eq!(reading("the spelling box can undo as it opens"), "false");
    // The space typed first ends a step of its own, as a space does.
    assert_eq!(reading("the spelling box after one Ctrl+Z"), "world big ");
    assert_eq!(reading("the spelling box after two"), "world ");
    assert_eq!(reading("the spelling box after three"), "world");
    assert_eq!(reading("the spelling box after four"), "world");
}

#[test]
fn test_a_value_a_dialog_fills_in_before_it_is_shown_is_not_a_step() {
    assert_eq!(
        reading("the stored contact's boxes with a step to undo"),
        ""
    );
    assert_eq!(reading("the nickname can undo once typed in"), "true");
}

#[test]
fn test_a_chosen_note_is_written_as_the_programs_own_value() {
    let source = wx_app_source();
    let handler = the_note_selection(&source).expect("the note selection handler is in wx_app.rs");
    a_chosen_note_is_the_programs_own(handler).unwrap();
}

// ── Companions ────────────────────────────────────────────────────────────

#[test]
fn test_companion_a_history_that_recorded_its_own_restore_is_refused() {
    assert!(the_restore_was_not_recorded("false").is_err());
}

#[test]
fn test_companion_a_key_down_passed_on_after_acting_is_refused() {
    assert!(the_key_was_taken("true").is_err());
}

#[test]
fn test_companion_a_note_written_with_set_value_is_refused() {
    let source = wx_app_source();
    let handler = the_note_selection(&source).expect("the note selection handler is in wx_app.rs");
    let planted = handler.replacen("set_anew(&title_input,", "title_input.set_value(", 1);
    assert!(a_chosen_note_is_the_programs_own(&planted).is_err());
}

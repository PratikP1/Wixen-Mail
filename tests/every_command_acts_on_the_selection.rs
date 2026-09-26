//! The message list selects more than one message, and every command that
//! acts on messages acts on the selection with one sentence (#30, and the
//! thread clause of #27).
//!
//! The tester on 2026-09-15: "Shift+arrow keys should allow the user to
//! select multiple messages." The list was built single-selection, so Shift
//! with an arrow moved the selection instead of extending it, and every
//! command read one index. Two kinds of check here.
//!
//! The built list, `cfg(windows)`, on 11-06.1's shape: a virtual report list
//! without `SingleSel`, wired with a handler on the focus event and one on
//! the selection event, each counting. It measures what the deviation from
//! the plan rests on: growing the selection raises a selection event per
//! row and no focus event, so a cursor handler on the selection event would
//! run once per row of a Select All, five thousand body loads for one key;
//! a focus event is raised once per cursor move, including a move onto a
//! row that was already selected, which is what Shift+Up does when it
//! shrinks the range and which raises no selection event at all. And the
//! walk that reads the selection off the control answers every selected
//! row in order.
//!
//! The source readings, over `what_ships` of `src/presentation/wx_app.rs`:
//! the list built without `SingleSel`; each of the seven commands reading
//! the selection, refusing above the bound and saying one sentence; the
//! cursor commands reading no selection; the cursor handler on the focus
//! event writing the index; the Mark as Read words following the rows; and
//! a set leaving the list landing the cursor once. Each with a companion
//! that plants the fault into a snippet shaped as the window should be, so
//! a reading that stopped finding its anchor cannot pass by finding nothing.
//!
//! One `#[ignore]` timing, run once on a release build for the row on
//! `docs/development/measurements.md`: five thousand messages marked read
//! in the cache through the write the arm uses, one per message.
//!
//! # Why this lives here rather than beside the code
//!
//! `src/presentation/wx_app.rs` is named by 85 guard records on 2026-09-19,
//! so a test added there is 85 builds and 85 library runs at the next commit.
//! This file is named by its own records, whose `suite` couples it to
//! `wx_app.rs`, so it runs on the commits that could break it. The built
//! list is built here for the reason `tests/theme_reach.rs` gives: a window
//! can be built in a test, and what a control did is read from the control.
//!
//! # What this cannot see
//!
//! Whether NVDA says "selected" and "not selected" as the range grows and
//! shrinks, and the count after Ctrl+A, which are the control's own words
//! and the tester's ear; whether one sentence after a command over many is
//! enough by ear; and what a server does with a thousand flag changes,
//! which nothing here has met. The window is not started.

use std::fs;

use wixen_mail::common::what_ships::what_ships;

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

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

// ── The anchors, each a name or a literal ──────────────────────────────────

/// Where the message list is built, and the first line after the builder.
const THE_LIST_BUILT: (&str, &str) = (
    "let msg_list = ListCtrl::builder(&inner)",
    "set_accessible_name(&msg_list, \"Messages\");",
);
const THE_STAR_ARM: &str = "_ if id == ID_TOGGLE_STAR =>";
const THE_DELETE_ARM: &str = "_ if id == ID_DELETE || id == ID_DELETE_OUTRIGHT =>";
const THE_TOGGLE: &str = "fn toggle_read_state(";
const THE_LABELS: &str = "fn label_the_message(";
const THE_MOVE: &str = "fn move_or_copy_message(";
const THE_BATCH: &str = "fn spawn_folder_move(";
const THE_REFRESH: &str = "fn refresh_mark_read_wording(";
const THE_REMOVAL: &str = "fn take_row_out_of_the_list(";
/// The cursor handler, on the focus event, and the next handler on the list.
const THE_CURSOR_HANDLER: (&str, &str) =
    ("msg_list.on_item_focused({", "msg_list.on_column_click({");
/// The handler the cursor used to be read from, which a multi-selection
/// list raises once per selected row.
const THE_OLD_HANDLER: &str = "msg_list.on_item_selected({";
/// The commands that act on the row the cursor is on, by the function or
/// the arm that reads it.
///
/// Answering an invitation is not among them since 13-11: it takes the message
/// it was pressed on, because a reader window's buttons answer the message the
/// window shows, and the Action menu's arm passes the row under the cursor.
const THE_CURSOR_COMMANDS: [&str; 5] = [
    "fn start_reply(",
    "fn msg_info(",
    "fn save_the_message_as(",
    "fn receipt_for_the_open_message(",
    "fn send_receipt_for_the_open_message(",
];
const THE_COPY_TO_ARM: &str = "_ if id == ID_CONTEXT_COPY_TO_TASK";
/// What a command over the set reads and says.
const READS_THE_SELECTION: &str = "chosen_messages(";
const REFUSES_ABOVE_THE_BOUND: &str = "too_many(";
const SAYS_ONE_SENTENCE: &str = "what_was_done(";
const SHOWS_THE_INTENT: &str = "what_is_being_done(";

// ── The readings ───────────────────────────────────────────────────────────

/// The list is built without `SingleSel`, so Shift with an arrow extends
/// the selection the way every Windows list does.
fn the_list_selects_more_than_one(app: &str) -> Result<(), String> {
    let built = between(app, THE_LIST_BUILT.0, THE_LIST_BUILT.1)?;
    if built.contains("SingleSel") {
        return Err(
            "the message list is built with SingleSel, so Shift with an arrow moves the \
             selection instead of extending it"
                .to_string(),
        );
    }
    Ok(())
}

/// Star, Mark as Read and the Labels read the selection, refuse above the
/// bound, and say one sentence over the set.
fn the_flag_commands_act_on_the_selection(app: &str) -> Result<(), String> {
    let star = the_id_arm(app, THE_STAR_ARM)?.to_string();
    let toggle = body_of(app, THE_TOGGLE)?;
    let labels = body_of(app, THE_LABELS)?;
    for (name, text) in [
        ("the Star arm", star),
        ("toggle_read_state", toggle),
        ("label_the_message", labels),
    ] {
        for needed in [
            READS_THE_SELECTION,
            REFUSES_ABOVE_THE_BOUND,
            SAYS_ONE_SENTENCE,
        ] {
            if !text.contains(needed) {
                return Err(format!(
                    "{name} does not reach {needed}, so it acts on the cursor row alone or \
                     says nothing over the set"
                ));
            }
        }
    }
    if !body_of(app, THE_TOGGLE)?.contains("what_mark_read_does(") {
        return Err(
            "toggle_read_state does not ask what_mark_read_does, so a set with one unread \
             message is not marked read as the label says"
                .to_string(),
        );
    }
    Ok(())
}

/// Delete and Delete Permanently read the selection, refuse above the
/// bound, ask when a conversation row is in the set, say the one word once
/// and show the intent line for the set.
fn delete_acts_on_the_selection(app: &str) -> Result<(), String> {
    let arm = the_id_arm(app, THE_DELETE_ARM)?;
    for needed in [
        READS_THE_SELECTION,
        REFUSES_ABOVE_THE_BOUND,
        "deleting_asks(",
        "say_the_one_word(&a11y, \"Delete\")",
        SHOWS_THE_INTENT,
    ] {
        if !arm.contains(needed) {
            return Err(format!(
                "the Delete arm does not reach {needed}, so a delete over a set acts on the \
                 cursor row alone or says something per message"
            ));
        }
    }
    if app.contains("fn delete_the_conversation_row(") {
        return Err(
            "delete_the_conversation_row is still a second delete beside the arm, so a \
             conversation row's delete and a set's delete can drift apart"
                .to_string(),
        );
    }
    Ok(())
}

/// Move to and Copy to read the selection, refuse above the bound, show the
/// intent line for the set, and the batch says one sentence at its end.
fn move_and_copy_act_on_the_selection(app: &str) -> Result<(), String> {
    let asks = body_of(app, THE_MOVE)?;
    for needed in [
        READS_THE_SELECTION,
        REFUSES_ABOVE_THE_BOUND,
        SHOWS_THE_INTENT,
    ] {
        if !asks.contains(needed) {
            return Err(format!(
                "move_or_copy_message does not reach {needed}, so a move over a set moves \
                 the cursor row alone"
            ));
        }
    }
    let batch = body_of(app, THE_BATCH)?;
    if !batch.contains(SAYS_ONE_SENTENCE) {
        return Err(
            "spawn_folder_move never says what_was_done, so a set moved is heard once per \
             message or not at all"
                .to_string(),
        );
    }
    Ok(())
}

/// Reply, Forward, Save As, the read receipt, the invitation answer and Copy
/// to a task, event or note act on the row the cursor is on and read no
/// selection.
fn the_cursor_commands_read_no_selection(app: &str) -> Result<(), String> {
    let copy_to = the_id_arm(app, THE_COPY_TO_ARM)?.to_string();
    let mut texts = vec![("the Copy to arm".to_string(), copy_to)];
    for command in THE_CURSOR_COMMANDS {
        texts.push((command.to_string(), body_of(app, command)?));
    }
    for (name, text) in texts {
        if !text.contains("selected_message_index") {
            return Err(format!(
                "{name} no longer reads selected_message_index, so it does not act on the \
                 row the cursor is on"
            ));
        }
        for must_not in [READS_THE_SELECTION, SAYS_ONE_SENTENCE] {
            if text.contains(must_not) {
                return Err(format!(
                    "{name} reaches {must_not}, so a reply or a save would act on a set"
                ));
            }
        }
    }
    Ok(())
}

/// The cursor is the focused row: the handler is on the focus event, it
/// writes the index, and no handler on the selection event is left, since
/// a multi-selection list raises that one once per selected row.
fn the_cursor_follows_the_focused_row(app: &str) -> Result<(), String> {
    let handler = between(app, THE_CURSOR_HANDLER.0, THE_CURSOR_HANDLER.1)?;
    if !handler.contains("selected_message_index = Some(idx)") {
        return Err(
            "the focus handler does not write selected_message_index, so the cursor commands \
             read a row the cursor left"
                .to_string(),
        );
    }
    if app.contains(THE_OLD_HANDLER) {
        return Err(
            "a handler on the message list's selection event is still wired, and a \
             multi-selection list raises it once per row of a Select All"
                .to_string(),
        );
    }
    Ok(())
}

/// The Mark as Read words follow the selected rows, not the cursor row
/// alone, so the label says what the command will do to the set.
fn the_words_follow_the_selection(app: &str) -> Result<(), String> {
    let refresh = body_of(app, THE_REFRESH)?;
    if !refresh.contains("chosen_rows(") {
        return Err(
            "refresh_mark_read_wording reads the cursor row alone, so the label says the \
             wrong way for a set with an unread message further down"
                .to_string(),
        );
    }
    Ok(())
}

/// A set leaving the list lands the cursor once, after the last row of it
/// has left, through 11-06.1's rule over the set's rows.
fn a_set_leaving_lands_once(app: &str) -> Result<(), String> {
    let removal = body_of(app, THE_REMOVAL)?;
    for needed in ["a_set_leaving", "land_the_cursor_after("] {
        if !removal.contains(needed) {
            return Err(format!(
                "take_row_out_of_the_list does not reach {needed}, so a delete over five \
                 lands the cursor five times or not after the set"
            ));
        }
    }
    let arm = the_id_arm(app, THE_DELETE_ARM)?;
    if !arm.contains("a_set_leaving") {
        return Err(
            "the Delete arm does not record the set that is leaving, so the removal path \
             lands after each row rather than after the set"
                .to_string(),
        );
    }
    Ok(())
}

// ── The readings over the window ───────────────────────────────────────────

#[test]
fn test_the_message_list_is_built_without_single_selection() {
    the_list_selects_more_than_one(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_star_mark_as_read_and_the_labels_act_on_the_selection_with_one_sentence() {
    the_flag_commands_act_on_the_selection(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_delete_acts_on_the_selection_asks_for_a_conversation_and_says_delete_once() {
    delete_acts_on_the_selection(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_move_and_copy_act_on_the_selection_and_the_batch_says_one_sentence() {
    move_and_copy_act_on_the_selection(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_cursor_commands_act_on_the_row_the_cursor_is_on_and_read_no_selection() {
    the_cursor_commands_read_no_selection(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_cursor_follows_the_focused_row_and_the_handler_writes_the_index() {
    the_cursor_follows_the_focused_row(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_mark_as_read_words_follow_the_selected_rows() {
    the_words_follow_the_selection(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_a_set_leaving_the_list_lands_the_cursor_once_after_the_last_row() {
    a_set_leaving_lands_once(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

// ── The companions: the window as it should be, with a fault planted ───────

/// A snippet shaped as the window should be, holding every anchor the
/// readings look for and nothing else.
fn a_window_as_it_should_be() -> String {
    let mut snippet = String::new();
    snippet.push_str(THE_LIST_BUILT.0);
    snippet.push_str(
        "\n    .with_style(ListCtrlStyle::Report | ListCtrlStyle::HRules | ListCtrlStyle::Virtual)\n    .build();\n",
    );
    snippet.push_str(THE_LIST_BUILT.1);
    snippet.push('\n');
    snippet.push_str(THE_CURSOR_HANDLER.0);
    snippet.push_str("\n    s.selected_message_index = Some(idx);\n});\n");
    snippet.push_str(THE_CURSOR_HANDLER.1);
    snippet.push_str("\n});\n");
    for arm in [THE_STAR_ARM, THE_DELETE_ARM] {
        snippet.push_str(arm);
        snippet.push_str(
            " {\n    chosen_messages(state, cache, list, reach);\n    too_many(n);\n    \
             what_was_done(&chosen, &outcome);\n    deleting_asks(&chosen);\n    \
             say_the_one_word(&a11y, \"Delete\");\n    what_is_being_done(\"Deleting\", &chosen);\n    \
             s.a_set_leaving = Some(leaving);\n}\n",
        );
    }
    snippet.push_str(THE_COPY_TO_ARM);
    snippet.push_str(" {\n    s.selected_message_index\n}\n_ if id == ID_OTHER => {}\n");
    for signature in [THE_TOGGLE, THE_LABELS, THE_MOVE, THE_BATCH] {
        snippet.push_str(signature);
        snippet.push_str(
            ") {\n    chosen_messages(state, cache, list, reach);\n    too_many(n);\n    \
             what_was_done(&chosen, &outcome);\n    what_mark_read_does(&chosen);\n    \
             what_is_being_done(\"Moving\", &chosen);\n}\n",
        );
    }
    for command in THE_CURSOR_COMMANDS {
        snippet.push_str(command);
        snippet.push_str(") {\n    s.selected_message_index\n}\n");
    }
    snippet.push_str(THE_REFRESH);
    snippet.push_str(") {\n    chosen_rows(list);\n}\n");
    snippet.push_str(THE_REMOVAL);
    snippet
        .push_str(") {\n    s.a_set_leaving;\n    land_the_cursor_after(list, &rows, len);\n}\n");
    snippet
}

fn every_reading_over(app: &str) -> Result<(), String> {
    the_list_selects_more_than_one(app)?;
    the_flag_commands_act_on_the_selection(app)?;
    delete_acts_on_the_selection(app)?;
    move_and_copy_act_on_the_selection(app)?;
    the_cursor_commands_read_no_selection(app)?;
    the_cursor_follows_the_focused_row(app)?;
    the_words_follow_the_selection(app)?;
    a_set_leaving_lands_once(app)
}

#[test]
fn test_the_readings_pass_a_window_shaped_as_it_should_be() {
    every_reading_over(&a_window_as_it_should_be()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_reading_complains_when_single_selection_is_put_back() {
    let app = a_window_as_it_should_be().replacen(
        "ListCtrlStyle::Report |",
        "ListCtrlStyle::Report | ListCtrlStyle::SingleSel |",
        1,
    );
    let why = the_list_selects_more_than_one(&app).expect_err("SingleSel planted");
    assert!(why.contains("built with SingleSel"), "{why}");
}

#[test]
fn test_the_reading_complains_when_an_arm_acts_on_the_cursor_row_alone() {
    let app = a_window_as_it_should_be();
    let toggle_alone = app.replacen(
        "fn toggle_read_state() {\n    chosen_messages(state, cache, list, reach);",
        "fn toggle_read_state() {\n    s.selected_message_index;",
        1,
    );
    let why = the_flag_commands_act_on_the_selection(&toggle_alone).expect_err("the toggle alone");
    assert!(
        why.contains("toggle_read_state does not reach chosen_messages("),
        "{why}"
    );

    let delete_unbounded = app.replacen(
        "too_many(n);\n    what_was_done(&chosen, &outcome);\n    deleting_asks",
        "what_was_done(&chosen, &outcome);\n    deleting_asks",
        2,
    );
    let why = delete_acts_on_the_selection(&delete_unbounded).expect_err("the bound dropped");
    assert!(why.contains("does not reach too_many("), "{why}");

    let move_alone = app.replacen(
        "fn move_or_copy_message() {\n    chosen_messages(state, cache, list, reach);",
        "fn move_or_copy_message() {\n    s.selected_message_index;",
        1,
    );
    let why = move_and_copy_act_on_the_selection(&move_alone).expect_err("the move alone");
    assert!(why.contains("move_or_copy_message does not reach"), "{why}");
}

#[test]
fn test_the_reading_complains_when_a_cursor_command_reaches_the_set() {
    let app = a_window_as_it_should_be().replacen(
        "fn start_reply() {\n    s.selected_message_index",
        "fn start_reply() {\n    chosen_messages(state, cache, list, reach)",
        1,
    );
    let why = the_cursor_commands_read_no_selection(&app).expect_err("a reply over a set");
    assert!(why.contains("fn start_reply("), "{why}");
}

#[test]
fn test_the_reading_complains_when_the_selection_handler_is_wired_again() {
    let app = a_window_as_it_should_be().replacen(
        THE_CURSOR_HANDLER.1,
        "msg_list.on_item_selected({\n});\nmsg_list.on_column_click({",
        1,
    );
    let why = the_cursor_follows_the_focused_row(&app).expect_err("the selection handler back");
    assert!(why.contains("selection event is still wired"), "{why}");
}

#[test]
fn test_the_reading_complains_when_the_words_or_the_landing_read_the_cursor_row_alone() {
    let app = a_window_as_it_should_be();
    let words = app.replacen("chosen_rows(list);", "s.selected_message_index;", 1);
    let why = the_words_follow_the_selection(&words).expect_err("the words on the cursor row");
    assert!(why.contains("reads the cursor row alone"), "{why}");

    let landing = app.replacen("    s.a_set_leaving;\n", "", 1);
    let why = a_set_leaving_lands_once(&landing).expect_err("the set forgotten");
    assert!(why.contains("does not reach a_set_leaving"), "{why}");
}

// ── The built list ─────────────────────────────────────────────────────────

#[cfg(windows)]
mod the_built_list {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use std::sync::{Arc, Mutex, OnceLock};

    use wixen_mail::presentation::wx_app::chosen_rows;
    use wxdragon::prelude::*;

    /// What the built list answered at each step.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Harvest {
        /// Step 1: the cursor put on row 2 of five, selected and focused, as
        /// arrowing there does. The focus handler's and the selection
        /// handler's counts after it.
        after_the_cursor_landed: (usize, usize),
        /// Step 2: rows 3 and 4 selected without moving the focus, as
        /// growing the range does to each new row. The two counts.
        after_the_range_grew: (usize, usize),
        /// What the walk answered over the three selected rows.
        the_rows_chosen: Vec<usize>,
        /// Step 3: the focus moved to row 3, which was already selected, as
        /// Shift+Up does when it shrinks the range onto a row it kept. The
        /// two counts, and the row the focus handler was given.
        after_the_focus_moved_onto_a_selected_row: (usize, usize),
        the_row_the_focus_handler_was_given: Option<usize>,
    }

    /// Build the list, drive it through the three steps, and read what it
    /// answered; the frame is destroyed before the session ends.
    fn read_the_list() -> Harvest {
        let frame = Frame::builder()
            .with_title("Every command acts on the selection, the reading")
            .with_size(Size::new(320, 240))
            .build();
        let list = ListCtrl::builder(&frame)
            .with_style(ListCtrlStyle::Report | ListCtrlStyle::Virtual)
            .build();
        list.insert_column(0, "Subject", ListColumnFormat::Left, 160);
        list.set_virtual_text_callback(|row, _column| format!("Message {row}"));
        list.set_item_count(5);
        let focused = Rc::new(Cell::new(0usize));
        let selected = Rc::new(Cell::new(0usize));
        let given = Rc::new(RefCell::new(None));
        list.on_item_focused({
            let focused = focused.clone();
            let given = given.clone();
            move |event| {
                focused.set(focused.get() + 1);
                *given.borrow_mut() = Some(event.get_item_index() as usize);
            }
        });
        list.on_item_selected({
            let selected = selected.clone();
            move |_event| selected.set(selected.get() + 1)
        });
        frame.show(true);
        list.set_focus();
        let counts = || (focused.get(), selected.get());
        let both = ListItemState::Selected | ListItemState::Focused;

        list.set_item_state(2, both, both);
        let after_the_cursor_landed = counts();

        for row in [3, 4] {
            list.set_item_state(row, ListItemState::Selected, ListItemState::Selected);
        }
        let after_the_range_grew = counts();
        let the_rows_chosen = chosen_rows(&list);

        *given.borrow_mut() = None;
        list.set_item_state(3, ListItemState::Focused, ListItemState::Focused);
        let after_the_focus_moved_onto_a_selected_row = counts();
        let the_row_the_focus_handler_was_given = *given.borrow();

        frame.destroy();
        Harvest {
            after_the_cursor_landed,
            after_the_range_grew,
            the_rows_chosen,
            after_the_focus_moved_onto_a_selected_row,
            the_row_the_focus_handler_was_given,
        }
    }

    fn take_the_harvest() -> Result<Harvest, String> {
        let outcome: Arc<Mutex<Option<Result<Harvest, String>>>> = Arc::new(Mutex::new(None));
        let result = {
            let outcome = outcome.clone();
            wxdragon::main(move |app| {
                let taken = Ok(read_the_list());
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

    #[test]
    fn test_the_focus_event_fires_once_per_cursor_move_and_the_selection_event_once_per_row() {
        let harvest = the_harvest();
        println!("{harvest:#?}");
        assert_eq!(
            harvest.after_the_cursor_landed,
            (1, 1),
            "landing on a row raises one of each"
        );
        assert_eq!(
            harvest.after_the_range_grew,
            (1, 3),
            "growing the range by two rows raises two more selection events and no focus event"
        );
        assert_eq!(
            harvest.after_the_focus_moved_onto_a_selected_row,
            (2, 3),
            "the focus moving onto a row already selected raises a focus event and no selection event"
        );
        assert_eq!(harvest.the_row_the_focus_handler_was_given, Some(3));
    }

    #[test]
    fn test_the_walk_answers_every_selected_row_in_order() {
        assert_eq!(the_harvest().the_rows_chosen, vec![2, 3, 4]);
    }
}

// ── The timing, run once on a release build for the measurements page ──────

#[cfg(test)]
mod the_cost_at_the_bound {
    use std::time::Instant;

    use wixen_mail::application::editing::MOST_ROWS_WORTH_SELECTING;
    use wixen_mail::common::types::FolderType;
    use wixen_mail::data::message_cache::{CachedFolder, IncomingMessage, MessageCache};
    use wixen_mail::presentation::message_columns::{ColumnLayout, FolderKind};
    use wixen_mail::presentation::sample_mailbox::sample_mailbox;
    use wixen_mail::presentation::ui_types::MessageItem;
    use wixen_mail::presentation::view_state::{self, Showing};
    use wixen_mail::service::safety::Verdict;

    const THE_ACCOUNT: &str = "the-account-at-the-bound";
    const THE_FOLDER: &str = "INBOX";

    /// A generated row as the sync would store it, unread.
    fn as_incoming(folder_id: i64, item: &MessageItem) -> IncomingMessage {
        IncomingMessage {
            folder_id,
            uid: item.uid,
            message_id: format!("<at-the-bound-{}@example.com>", item.uid),
            subject: item.subject.clone(),
            from_addr: item.from.clone(),
            to_addr: item.to.clone(),
            cc: None,
            reply_to: None,
            date: item.date.clone(),
            internal_date: Some(item.date.clone()),
            size_bytes: item.size_bytes,
            refs_header: None,
            read: false,
            starred: item.starred,
            answered: item.answered,
            draft: item.draft,
            deleted: false,
            has_attachments: item.has_attachments,
            safety: Verdict::ordinary(),
            gmail_message_id: None,
            server_thread_id: None,
            labels: None,
            receipt_to: None,
            list_unsubscribe: None,
            pop_uidl: None,
        }
    }

    /// Marking the bound's worth of messages read in the cache, one write
    /// each through the call the arm's write uses, and the elapsed time
    /// printed as the row the measurements page carries.
    ///
    /// Ignored on every ordinary run: a timing asserted on every commit is a
    /// flaky test waiting for a slow machine. Run once, on a release build,
    /// with nothing else building:
    /// `cargo test --release --test every_command_acts_on_the_selection -- --ignored --nocapture --test-threads=1`.
    #[test]
    #[ignore]
    fn test_marking_the_bounds_worth_of_messages_read_in_the_cache_costs_this() {
        let dir = tempfile::tempdir().expect("a folder for the cache");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
        let folder_id = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: THE_ACCOUNT.to_string(),
                name: THE_FOLDER.to_string(),
                path: THE_FOLDER.to_string(),
                folder_type: FolderType::Inbox.as_str().to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("the folder");
        let arriving: Vec<IncomingMessage> = sample_mailbox(MOST_ROWS_WORTH_SELECTING)
            .iter()
            .map(|item| as_incoming(folder_id, item))
            .collect();
        cache.upsert_messages(&arriving).expect("the rows written");
        let order = view_state::order_by(
            Showing::Messages,
            &ColumnLayout::defaults_for(FolderKind::Inbox).sort,
        );
        let rows = cache
            .get_message_list_sorted(folder_id, THE_ACCOUNT, Some(&order), None)
            .expect("the folder read whole");
        assert_eq!(rows.len(), MOST_ROWS_WORTH_SELECTING);

        let started = Instant::now();
        for row in &rows {
            cache
                .update_message_flags(row.id, true, row.starred)
                .expect("one flag write");
        }
        let took = started.elapsed();
        let still_unread = cache
            .get_message_list_sorted(folder_id, THE_ACCOUNT, Some(&order), None)
            .expect("the folder read again")
            .iter()
            .filter(|row| !row.read)
            .count();
        assert_eq!(still_unread, 0, "every row was marked read");
        println!(
            "| marking {} messages read in the cache, one write each | {} ms | \
             `cargo test --release --test every_command_acts_on_the_selection -- --ignored --nocapture --test-threads=1` |",
            MOST_ROWS_WORTH_SELECTING,
            took.as_millis()
        );
    }
}

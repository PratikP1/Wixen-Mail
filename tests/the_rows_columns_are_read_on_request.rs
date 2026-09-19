//! The row under the cursor, read column by column with its headings, on
//! request (#26).
//!
//! The tester on 2026-09-15, under NVDA: "when traversing through the list
//! of messages, column headers should not be announced each time. The user
//! should be able to specifically request the reading of columns and
//! corresponding text by pressing a keyboard command. use control+shift+;
//! if not already assigned." The header before each cell as somebody arrows
//! is NVDA's own reading of a report-view list, from its Row/column headers
//! setting, and nothing an application sets changes it; the pages give the
//! steps for a configuration profile that turns it off for this program
//! alone. What the program owns is the reading on request, and that is what
//! this file holds.
//!
//! Two halves. The composition, `message_rows::the_row_with_its_headings`,
//! is a pure function over the cells the list shows and is held here as
//! cases: the order given, empty cells left out, the six self-describing
//! cells said alone, every other cell said as heading then text, and the
//! whole composed from the very cells the paint callback answers with, for a
//! message row and for a conversation row. The wiring in the window, which
//! needs a frame to run, is read as text over `what_ships` of
//! `src/presentation/wx_app.rs`: the item on the Action menu with its chord,
//! the arm composing the focused row from the visible layout through the
//! same cell function the list paints with, and announcing it once as
//! content, which the mute controls.
//!
//! # Why this lives here rather than beside the code
//!
//! `src/presentation/message_rows.rs` is named by 2 guard records and
//! `src/presentation/wx_app.rs` by 97 on 2026-09-19, so a test added to
//! either is that many builds and library runs at the next commit. This file
//! is named by its own records, whose `suite` couples it to both, so it runs
//! on the commits that could break it.
//!
//! # What this cannot see
//!
//! Whether the reading is heard whole and once under NVDA, whether arrowing
//! is quiet under the profile the page describes, and what Narrator and JAWS
//! do with a report-view list: the tester's ear. The window is not started.

use std::fs;

use wixen_mail::application::conversations::{ConversationItem, RowMessage};
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::presentation::date_display::DateSettings;
use wixen_mail::presentation::message_columns::{ColumnLayout, FolderKind, MessageColumn};
use wixen_mail::presentation::message_rows::the_row_with_its_headings;
use wixen_mail::presentation::ui_types::MessageItem;
use wixen_mail::presentation::view_state::Showing;
use wixen_mail::presentation::virtual_rows::{self, Listed};

/// The six cells whose text already says what the column is, so a heading
/// in front of them would be the same word twice.
const SELF_DESCRIBING: [MessageColumn; 6] = [
    MessageColumn::Unread,
    MessageColumn::Attachment,
    MessageColumn::Flagged,
    MessageColumn::Answered,
    MessageColumn::Draft,
    MessageColumn::Safety,
];

fn cells(given: &[(MessageColumn, &str)]) -> Vec<(MessageColumn, String)> {
    given
        .iter()
        .map(|(column, text)| (*column, text.to_string()))
        .collect()
}

// ── The composition ────────────────────────────────────────────────────────

#[test]
fn test_a_row_reads_each_heading_then_its_text_in_the_order_the_columns_are_shown() {
    let row = cells(&[
        (MessageColumn::Subject, "Quarterly report"),
        (MessageColumn::Correspondent, "Ada Lovelace"),
        (MessageColumn::Unread, "Unread"),
        (MessageColumn::Attachment, ""),
        (MessageColumn::Received, "yesterday"),
    ]);
    assert_eq!(
        the_row_with_its_headings(&row),
        "Subject, Quarterly report. Correspondent, Ada Lovelace. Unread. Received, yesterday."
    );
}

#[test]
fn test_the_order_is_the_order_given_and_not_the_columns_own() {
    // The layout can put Received before Subject; the reading follows the
    // layout, because it is what the list shows that is being read back.
    let row = cells(&[
        (MessageColumn::Received, "yesterday"),
        (MessageColumn::Subject, "Quarterly report"),
    ]);
    assert_eq!(
        the_row_with_its_headings(&row),
        "Received, yesterday. Subject, Quarterly report."
    );
}

#[test]
fn test_an_empty_cell_is_left_out_altogether() {
    // A read message with nothing attached: the two flag cells are empty,
    // and an empty cell said as "Unread, " or as a bare heading would be a
    // word with nothing attached to it, which is the fault the cells were
    // written to avoid.
    let row = cells(&[
        (MessageColumn::Unread, ""),
        (MessageColumn::Attachment, ""),
        (MessageColumn::Subject, "Lunch"),
        (MessageColumn::Snippet, ""),
    ]);
    assert_eq!(the_row_with_its_headings(&row), "Subject, Lunch.");
}

#[test]
fn test_a_cell_that_already_ends_a_sentence_is_not_given_a_second_full_stop() {
    // A snippet is a message's first line and often ends one; "in.." is
    // heard as a stutter. Found by the message-row composition on the day
    // the rule was written.
    let row = cells(&[
        (MessageColumn::Subject, "Any questions?"),
        (MessageColumn::Snippet, "The numbers are in."),
    ]);
    assert_eq!(
        the_row_with_its_headings(&row),
        "Subject, Any questions? Snippet, The numbers are in."
    );
}

#[test]
fn test_no_cells_read_as_nothing() {
    assert_eq!(the_row_with_its_headings(&[]), "");
    let all_empty = cells(&[(MessageColumn::Subject, ""), (MessageColumn::Snippet, "")]);
    assert_eq!(the_row_with_its_headings(&all_empty), "");
}

#[test]
fn test_a_self_describing_cell_is_said_as_its_text_alone() {
    for (column, text) in [
        (MessageColumn::Unread, "Unread"),
        (MessageColumn::Attachment, "Has attachment"),
        (MessageColumn::Flagged, "Flagged"),
        (MessageColumn::Answered, "Answered"),
        (MessageColumn::Draft, "Draft"),
        (MessageColumn::Safety, "Phishing"),
    ] {
        assert_eq!(
            the_row_with_its_headings(&cells(&[(column, text)])),
            format!("{text}."),
            "{column:?} should be said as its text alone"
        );
    }
}

#[test]
fn test_every_other_cell_is_said_as_its_heading_then_its_text() {
    for column in MessageColumn::ALL {
        if SELF_DESCRIBING.contains(&column) {
            continue;
        }
        assert_eq!(
            the_row_with_its_headings(&cells(&[(column, "something")])),
            format!("{}, something.", column.heading()),
            "{column:?} should be said as its heading then its text"
        );
    }
}

// ── The composition over the cells the list paints ─────────────────────────

/// The row's cells as the arm gathers them: the visible columns in the
/// layout's order, each through the function the paint callback answers
/// with, so what is read on request is what the list shows.
fn the_cells_the_list_shows(listed: Listed<'_>, row: i64) -> Vec<(MessageColumn, String)> {
    let columns = ColumnLayout::defaults_for(FolderKind::Inbox).visible();
    columns
        .iter()
        .enumerate()
        .map(|(at, column)| {
            let text = virtual_rows::text_for(
                listed,
                &columns,
                row,
                at as i32,
                DateSettings::default(),
                chrono::Local::now(),
            );
            (*column, text)
        })
        .collect()
}

#[test]
fn test_a_message_row_is_composed_from_the_cells_the_list_paints() {
    let message = MessageItem {
        subject: "Quarterly report".to_string(),
        from: "Ada Lovelace <ada@example.com>".to_string(),
        date: "2026-07-30".to_string(),
        read: false,
        has_attachments: true,
        snippet: Some("The numbers are in.".to_string()),
        ..MessageItem::default()
    };
    let cells = the_cells_the_list_shows(
        Listed {
            showing: Showing::Messages,
            messages: std::slice::from_ref(&message),
            conversations: &[],
        },
        0,
    );
    let read = the_row_with_its_headings(&cells);
    assert!(
        read.starts_with("Unread. Has attachment. Subject, Quarterly report. Correspondent, Ada Lovelace. Received, "),
        "{read:?}"
    );
    assert!(read.ends_with(". Snippet, The numbers are in."), "{read:?}");
}

#[test]
fn test_a_conversation_row_is_composed_from_its_own_cells() {
    // A conversation row's cells are the conversation's: unread when any
    // message is, the sender of the message the row stands for first (#31),
    // the newest date. Nothing here reaches into the flat rows.
    let conversation = ConversationItem {
        thread_id: "gm:5".to_string(),
        subject: "Quarterly report".to_string(),
        messages: 3,
        unread: 0,
        newest_received: "2026-07-30".to_string(),
        newest_sent: String::new(),
        snippet: Some("One question about page four".to_string()),
        senders: "Ada Lovelace <ada@example.com>\nBob <bob@example.com>".to_string(),
        to: String::new(),
        cc: String::new(),
        size_bytes: None,
        any_attachment: false,
        any_flagged: false,
        any_answered: false,
        any_draft: false,
        worst_safety: wixen_mail::service::safety::Safety::Ordinary,
        stands_for: RowMessage {
            id: 2,
            uid: 2,
            from: "Bob <bob@example.com>".to_string(),
        },
        says_first: None,
        labels: String::new(),
    };
    let cells = the_cells_the_list_shows(
        Listed {
            showing: Showing::Conversations,
            messages: &[],
            conversations: std::slice::from_ref(&conversation),
        },
        0,
    );
    let read = the_row_with_its_headings(&cells);
    assert!(
        read.starts_with("Subject, Quarterly report. Correspondent, Bob, Ada Lovelace. Received, "),
        "{read:?}"
    );
    assert!(
        read.ends_with(". Snippet, One question about page four."),
        "{read:?}"
    );
    assert!(
        !read.contains("Unread"),
        "nothing in it is unread: {read:?}"
    );
}

#[test]
fn test_which_headings_are_worth_saying_is_decided_for_every_column() {
    // Walked over ALL so that a column added later is sorted on purpose:
    // the new arm is either in the six or it is not, and this says which.
    for column in MessageColumn::ALL {
        assert_eq!(
            column.heading_is_worth_saying(),
            !SELF_DESCRIBING.contains(&column),
            "{column:?}: a heading is worth saying exactly when the cell's text does not \
             already say what the column is"
        );
    }
}

// ── The window, read as text ───────────────────────────────────────────────

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

/// Everything that builds the Action menu: the builder chain and what is
/// appended to the menu once it has finished, the two halves
/// `tests/wired.rs` reads.
fn the_action_menu(source: &str) -> Result<String, String> {
    let chain_starts = source.find("let message = Menu::builder()").ok_or(
        "the Action menu is no longer built as `let message = Menu::builder()`, so this \
         reads nothing"
            .to_string(),
    )?;
    let chain = &source[chain_starts..];
    let chain_ends = chain
        .find(".build();")
        .map_or(chain.len(), |at| at + ".build();".len());
    let mut block = chain[..chain_ends].to_string();
    for (at, _) in source.match_indices("message.append_") {
        let line_ends = source[at..]
            .find(");")
            .map_or(source.len(), |end| at + end + 2);
        block.push('\n');
        block.push_str(&source[at..line_ends]);
    }
    Ok(block)
}

// ── The anchors ────────────────────────────────────────────────────────────

const THE_ID: &str = "ID_READ_ROW_COLUMNS";
const THE_CHORD: &str = "\\tCtrl+Shift+;";
const THE_ARM: &str = "_ if id == ID_READ_ROW_COLUMNS =>";
const THE_READING: &str = "fn read_the_row_with_its_headings(";
const ASKS_WHETHER_THE_LIST_HOLDS_FOCUS: &str = "has_focus()";
const ASKS_WHICH_ROW_THE_CURSOR_IS_ON: &str = "selected_message_index";
const THE_VISIBLE_LAYOUT: &str = ".visible()";
const THE_CELL_THE_LIST_PAINTS: &str = "virtual_rows::text_for(";
const COMPOSES_THE_ROW: &str = "the_row_with_its_headings(";
const ANNOUNCES_AS_CONTENT: &str = "announce_content(";
const REFUSES_WITH: &str = "\"Nothing is selected in the message list\"";
/// The ways the window says something that are not content under the mute:
/// interface chatter, and the status bar's channels.
const SAYS_IT_ANY_OTHER_WAY: [&str; 3] = [".announce(", "send_status(", "send_shown("];
/// The two cell functions the paint callback dispatches to; the arm must
/// not compose cells of its own from either, or what it reads and what
/// the list shows can come apart.
const COMPOSES_CELLS_OF_ITS_OWN: [&str; 2] = ["cell_text(", "conversation_cell_text("];

// ── The readings ───────────────────────────────────────────────────────────

/// The item is on the Action menu with its chord, so the key reaches the
/// window from wherever focus is and the menu says what the key does.
fn the_item_is_on_the_action_menu_with_its_chord(app: &str) -> Result<(), String> {
    let menu = the_action_menu(app)?;
    let item = menu.find(&format!("{THE_ID},")).ok_or(format!(
        "{THE_ID} is not on the Action menu, so the key has no item and no chord"
    ))?;
    let label_ends = menu[item..].find("\",").map_or(menu.len(), |at| item + at);
    if !menu[item..label_ends].contains(THE_CHORD) {
        return Err(format!(
            "the Action menu's item for {THE_ID} does not carry the chord {}, so the key \
             the tester asked for is not bound",
            THE_CHORD.trim_start_matches("\\t")
        ));
    }
    Ok(())
}

/// The arm reaches the one function that reads the row.
fn the_arm_reaches_the_reading(app: &str) -> Result<(), String> {
    let arm = the_id_arm(app, THE_ARM)?;
    if !arm.contains(THE_READING.trim_start_matches("fn ")) {
        return Err(format!(
            "the {THE_ID} arm does not reach {THE_READING}, so the item and the key read \
             nothing"
        ));
    }
    Ok(())
}

/// The reading composes the row under the cursor from the visible layout,
/// in its order, through the very cell function the list paints with, and
/// refuses when the list does not hold focus or no row is under the cursor.
fn the_reading_composes_the_focused_row_from_what_the_list_shows(app: &str) -> Result<(), String> {
    let reading = body_of(app, THE_READING)?;
    for needed in [
        ASKS_WHETHER_THE_LIST_HOLDS_FOCUS,
        ASKS_WHICH_ROW_THE_CURSOR_IS_ON,
        REFUSES_WITH,
        THE_VISIBLE_LAYOUT,
        THE_CELL_THE_LIST_PAINTS,
        COMPOSES_THE_ROW,
    ] {
        if !reading.contains(needed) {
            return Err(format!(
                "{THE_READING} does not reach {needed}, so what is read is not the row under \
                 the cursor as the list shows it, or a chord pressed off the list reads a row \
                 nobody is on"
            ));
        }
    }
    for forbidden in COMPOSES_CELLS_OF_ITS_OWN {
        if reading.contains(forbidden) {
            return Err(format!(
                "{THE_READING} composes cells of its own through {forbidden} instead of \
                 reading the cell the list paints, so what is read and what is shown can \
                 come apart"
            ));
        }
    }
    Ok(())
}

/// The row goes out once, as content, which the mute controls; nothing on
/// the status bar and nothing through the interface channel.
fn the_reading_is_announced_once_as_content(app: &str) -> Result<(), String> {
    let reading = body_of(app, THE_READING)?;
    match reading.matches(ANNOUNCES_AS_CONTENT).count() {
        1 => {}
        0 => {
            return Err(format!(
                "{THE_READING} never reaches {ANNOUNCES_AS_CONTENT}, so the row is not read \
                 as content and the mute does not apply to it"
            ));
        }
        n => {
            return Err(format!(
                "{THE_READING} reaches {ANNOUNCES_AS_CONTENT} {n} times, so a press can be \
                 heard more than once"
            ));
        }
    }
    for other in SAYS_IT_ANY_OTHER_WAY {
        if reading.contains(other) {
            return Err(format!(
                "{THE_READING} says the row through {other} as well, so it is heard twice \
                 or outside the mute"
            ));
        }
    }
    Ok(())
}

#[test]
fn test_the_item_is_on_the_action_menu_with_its_chord() {
    the_item_is_on_the_action_menu_with_its_chord(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_arm_reaches_the_reading() {
    the_arm_reaches_the_reading(&the_main_window()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_reading_composes_the_focused_row_from_what_the_list_shows() {
    the_reading_composes_the_focused_row_from_what_the_list_shows(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_reading_is_announced_once_as_content() {
    the_reading_is_announced_once_as_content(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

// ── The companions ─────────────────────────────────────────────────────────

/// A window shaped as it should be, so a reading that stopped finding its
/// anchor cannot pass by finding nothing.
fn a_window_as_it_should_be() -> String {
    let mut snippet = String::new();
    snippet.push_str("        let message = Menu::builder()\n");
    snippet
        .push_str("            .append_item(ID_REPLY, \"&Reply\\tCtrl+R\", \"Reply to sender\")\n");
    snippet.push_str("            .append_item(\n                ");
    snippet.push_str(THE_ID);
    snippet.push_str(",\n                \"Read the Row's Headings and Te&xt");
    snippet.push_str(THE_CHORD);
    snippet.push_str(
        "\",\n                \"Read the row under the cursor column by column\",\n            )\n",
    );
    snippet.push_str("            .build();\n");
    snippet.push_str("        message.append_submenu(labels_menu, \"&Label\", \"Put a label on this message\");\n");
    snippet.push_str(THE_ARM);
    snippet.push_str(
        " {\n                            read_the_row_with_its_headings(\n                                \
         &msg_list, &state, &column_layout, date_settings, &a11y, &ui_tx, &runtime,\n                            \
         );\n                        }\n                        _ if id == ID_OTHER => {}\n",
    );
    snippet.push_str(THE_READING);
    snippet.push_str(
        "list: &ListCtrl) {\n    let Some(row) = list\n        .has_focus()\n        \
         .then(|| lock_state(state).selected_message_index)\n        .flatten()\n    else {\n        \
         send_refusal(tx, rt, \"Nothing is selected in the message list\");\n        return;\n    };\n    \
         let columns = layout.borrow().visible();\n    let cells = columns\n        .iter()\n        \
         .enumerate()\n        .map(|(at, column)| (*column, virtual_rows::text_for(listed, &columns, \
         row as i64, at as i32, dates, now)))\n        .collect::<Vec<_>>();\n    \
         let text = message_rows::the_row_with_its_headings(&cells);\n    \
         let _ = a11y.announce_content(&text);\n}\n",
    );
    snippet
}

fn every_reading_over(app: &str) -> Result<(), String> {
    the_item_is_on_the_action_menu_with_its_chord(app)?;
    the_arm_reaches_the_reading(app)?;
    the_reading_composes_the_focused_row_from_what_the_list_shows(app)?;
    the_reading_is_announced_once_as_content(app)
}

#[test]
fn test_the_readings_pass_a_window_shaped_as_it_should_be() {
    every_reading_over(&a_window_as_it_should_be()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_readings_complain_when_the_item_or_the_arm_is_gone() {
    let app = a_window_as_it_should_be();

    let no_chord = app.replacen(THE_CHORD, "", 1);
    let why = the_item_is_on_the_action_menu_with_its_chord(&no_chord).expect_err("no chord");
    assert!(why.contains("does not carry the chord"), "{why}");

    let no_item = app.replacen(&format!("{THE_ID},"), "ID_OTHER_ITEM,", 1);
    let why = the_item_is_on_the_action_menu_with_its_chord(&no_item).expect_err("no item");
    assert!(why.contains("is not on the Action menu"), "{why}");

    let arm_reads_nothing = app.replacen(
        "read_the_row_with_its_headings(\n",
        "read_aloud_the_short_form(\n",
        1,
    );
    let why = the_arm_reaches_the_reading(&arm_reads_nothing).expect_err("the arm reads nothing");
    assert!(why.contains("does not reach fn read_the_row"), "{why}");

    let no_arm = app.replacen(THE_ARM, "_ if id == ID_OTHER_ARM =>", 1);
    let why = the_arm_reaches_the_reading(&no_arm).expect_err("no arm");
    assert!(why.contains("is no longer here"), "{why}");
}

#[test]
fn test_the_readings_complain_when_the_row_is_not_the_lists_own_or_is_said_another_way() {
    let app = a_window_as_it_should_be();

    let reads_off_the_list = app.replacen(
        "    let Some(row) = list\n        .has_focus()\n        .then(|| lock_state(state).selected_message_index)\n        .flatten()\n",
        "    let Some(row) = lock_state(state).selected_message_index\n",
        1,
    );
    let why = the_reading_composes_the_focused_row_from_what_the_list_shows(&reads_off_the_list)
        .expect_err("reads off the list");
    assert!(why.contains("does not reach has_focus()"), "{why}");

    let its_own_cells = app.replacen(
        "virtual_rows::text_for(listed, &columns, row as i64, at as i32, dates, now)",
        "message_rows::cell_text(&message, *column, dates, now)",
        1,
    );
    let why = the_reading_composes_the_focused_row_from_what_the_list_shows(&its_own_cells)
        .expect_err("its own cells");
    assert!(
        why.contains("does not reach virtual_rows::text_for("),
        "{why}"
    );

    let both = app.replacen(
        "let text = message_rows::the_row_with_its_headings(&cells);",
        "let text = message_rows::the_row_with_its_headings(&cells);\n    \
         let _ = message_rows::cell_text(&message, MessageColumn::Subject, dates, now);",
        1,
    );
    let why = the_reading_composes_the_focused_row_from_what_the_list_shows(&both)
        .expect_err("cells of its own beside the list's");
    assert!(why.contains("composes cells of its own"), "{why}");

    let as_chatter = app.replacen(
        "let _ = a11y.announce_content(&text);",
        "let _ = a11y.announce(&text, Priority::Normal);",
        1,
    );
    let why = the_reading_is_announced_once_as_content(&as_chatter).expect_err("as chatter");
    assert!(why.contains("never reaches announce_content("), "{why}");

    let twice = app.replacen(
        "let _ = a11y.announce_content(&text);",
        "let _ = a11y.announce_content(&text);\n    let _ = a11y.announce_content(&text);",
        1,
    );
    let why = the_reading_is_announced_once_as_content(&twice).expect_err("twice");
    assert!(why.contains("2 times"), "{why}");

    let on_the_bar_too = app.replacen(
        "let _ = a11y.announce_content(&text);",
        "let _ = a11y.announce_content(&text);\n    send_status(tx, rt, &text);",
        1,
    );
    let why = the_reading_is_announced_once_as_content(&on_the_bar_too).expect_err("on the bar");
    assert!(why.contains("through send_status( as well"), "{why}");
}

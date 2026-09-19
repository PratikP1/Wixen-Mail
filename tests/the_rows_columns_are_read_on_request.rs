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

use wixen_mail::application::conversations::{ConversationItem, RowMessage};
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

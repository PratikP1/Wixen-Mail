//! The text the message list paints for one cell, computed from memory.
//!
//! The list runs in virtual mode: wxWidgets calls back for cell text while it
//! is painting, once per visible cell, on the interface thread. What that
//! callback does is therefore what every scroll costs, and a callback that
//! reached the database would block every paint on a lock and turn a large
//! mailbox into a list that stalls under the arrow keys.
//!
//! [`text_for`] is the whole of that work. Its inputs are slices of what the
//! window already holds, copies of the settings, and the moment of the paint.
//! There is no connection to reach because none is passed: the message cache
//! lives on the window, not on the state the callback locks, and nothing here
//! has a path to it. That is the type-level half of the check PERF-03's second
//! `[D]` line asks for, that the virtual text callback issues no query.
//!
//! The type alone does not close the question. A function whose inputs hold no
//! connection can still open one from a path literal, and the compiler would
//! not mind. So `tests/the_list_reads_only_memory.rs` reads this file and the
//! closure in `wx_app.rs` that calls it, and refuses either naming
//! `MessageCache`, `rusqlite`, `Connection`, `open(` or `conn`; it also holds
//! the closure to calling this function, so a closure rewritten to do its own
//! work is caught. Neither half alone is the check: the type cannot stop a
//! body opening a database, and a reading cannot see what a slice can reach.
//!
//! The type half cannot be a guard record. Its break is a compile error, an
//! extra parameter carrying the cache, and `scripts/guards.py` measures a
//! break by which tests go red under it; nothing goes red when nothing
//! builds. The reading half is the recordable one and is recorded.

use chrono::{DateTime, Local};

use super::date_display::DateSettings;
use super::message_columns::MessageColumn;
use super::message_rows::{self, PLACEHOLDER};
use super::ui_types::MessageItem;
use super::view_state::Showing;
use crate::application::conversations::ConversationItem;

/// What the list is showing: which view, and the rows of both, borrowed.
///
/// Both slices rather than the one the view needs, because the state holds
/// both and the closure that builds this should not have to choose; choosing
/// is the dispatch, and the dispatch is what [`text_for`] is for.
#[derive(Debug, Clone, Copy)]
pub struct Listed<'a> {
    pub showing: Showing,
    pub messages: &'a [MessageItem],
    pub conversations: &'a [ConversationItem],
}

/// The text for one cell of the message list.
///
/// Reads what is already in memory and never touches the database: every
/// input is a slice, a copy or the moment of the paint, so there is nothing
/// else it could touch. A row past the end of the view answers
/// [`PLACEHOLDER`], because the list may ask for a row the moment before the
/// rows arrive and a blank there would read as an empty message. A column past
/// the visible ones answers nothing, because there is no cell to paint.
///
/// `row` and `column` are the types wxWidgets hands the callback, converted
/// here rather than in the closure so that a negative index, which `as usize`
/// would wrap to a row nobody has, reads as past the end.
pub fn text_for(
    listed: Listed<'_>,
    columns: &[MessageColumn],
    row: i64,
    column: i32,
    dates: DateSettings,
    now: DateTime<Local>,
) -> String {
    let Some(column) = usize::try_from(column)
        .ok()
        .and_then(|at| columns.get(at))
        .copied()
    else {
        return String::new();
    };
    let Ok(row) = usize::try_from(row) else {
        return PLACEHOLDER.to_string();
    };
    match listed.showing {
        Showing::Messages => listed
            .messages
            .get(row)
            .map(|message| message_rows::cell_text(message, column, dates, now)),
        Showing::Conversations => listed.conversations.get(row).map(|conversation| {
            message_rows::conversation_cell_text(conversation, column, dates, now)
        }),
    }
    .unwrap_or_else(|| PLACEHOLDER.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_message(subject: &str) -> MessageItem {
        MessageItem {
            uid: 1,
            message_id: 1,
            subject: subject.to_string(),
            from: "Ada Lovelace <ada@example.com>".to_string(),
            date: "2026-07-30".to_string(),
            read: false,
            starred: false,
            answered: false,
            draft: false,
            has_attachments: false,
            attachments: Vec::new(),
            thread_depth: 0,
            is_thread_parent: false,
            thread_id: None,
            snippet: None,
            size_bytes: None,
            to: "me@example.com".to_string(),
            cc: String::new(),
            reply_to: String::new(),
            header_message_id: String::new(),
            refs_header: None,
            safety: crate::service::safety::Safety::Ordinary,
            safety_reasons: Vec::new(),
            receipt_to: None,
            list_unsubscribe: None,
            account_id: String::new(),
            labels: Vec::new(),
            says_first: None,
        }
    }

    fn a_conversation(subject: &str) -> ConversationItem {
        ConversationItem {
            thread_id: "root@example.com".to_string(),
            subject: subject.to_string(),
            messages: 3,
            unread: 1,
            newest_received: "2026-07-26T10:00:05+00:00".to_string(),
            newest_sent: "2026-07-26T10:00:00+00:00".to_string(),
            snippet: Some("The figures are attached".to_string()),
            senders: "\nAda Lovelace <ada@example.com>".to_string(),
            to: "\nme@example.com".to_string(),
            cc: String::new(),
            size_bytes: Some(4096),
            any_attachment: false,
            any_flagged: false,
            any_answered: false,
            any_draft: false,
            worst_safety: crate::service::safety::Safety::Ordinary,
            stands_for: crate::application::conversations::RowMessage {
                id: 1,
                uid: 1,
                from: "Ada Lovelace <ada@example.com>".to_string(),
            },
            says_first: None,
            labels: String::new(),
        }
    }

    const SUBJECT_ONLY: [MessageColumn; 1] = [MessageColumn::Subject];

    fn painted(listed: Listed<'_>, row: i64, column: i32) -> String {
        text_for(
            listed,
            &SUBJECT_ONLY,
            row,
            column,
            DateSettings::default(),
            Local::now(),
        )
    }

    #[test]
    fn test_a_message_row_gives_its_cell_text() {
        let messages = [a_message("Water bill"), a_message("Roof repair")];
        let listed = Listed {
            showing: Showing::Messages,
            messages: &messages,
            conversations: &[],
        };

        assert_eq!(painted(listed, 1, 0), "Roof repair");
    }

    #[test]
    fn test_a_conversation_row_gives_its_cell_text() {
        let conversations = [a_conversation("Bee swarm"), a_conversation("Allotment")];
        let listed = Listed {
            showing: Showing::Conversations,
            messages: &[],
            conversations: &conversations,
        };

        assert_eq!(painted(listed, 1, 0), "Allotment");
    }

    #[test]
    fn test_a_row_past_the_end_is_the_placeholder_in_both_views() {
        let messages = [a_message("Water bill")];
        let conversations = [a_conversation("Bee swarm")];

        for listed in [
            Listed {
                showing: Showing::Messages,
                messages: &messages,
                conversations: &conversations,
            },
            Listed {
                showing: Showing::Conversations,
                messages: &messages,
                conversations: &conversations,
            },
        ] {
            assert_eq!(painted(listed, 1, 0), PLACEHOLDER, "{:?}", listed.showing);
        }
    }

    #[test]
    fn test_a_column_past_the_visible_ones_is_empty() {
        let messages = [a_message("Water bill")];
        let listed = Listed {
            showing: Showing::Messages,
            messages: &messages,
            conversations: &[],
        };

        assert_eq!(painted(listed, 0, 1), "");
    }

    #[test]
    fn test_a_negative_index_reads_as_past_the_end_rather_than_wrapping() {
        // wxWidgets hands the callback signed integers. `as usize` on a
        // negative one is a row nobody has, which `get` would answer the same
        // way, but the conversion is the function's and this says so.
        let messages = [a_message("Water bill")];
        let listed = Listed {
            showing: Showing::Messages,
            messages: &messages,
            conversations: &[],
        };

        assert_eq!(painted(listed, -1, 0), PLACEHOLDER);
        assert_eq!(painted(listed, 0, -1), "");
    }

    #[test]
    fn test_the_view_decides_which_rows_are_read() {
        // The same slices under the other view answer the other kind of row,
        // which is the dispatch the closure used to do and now delegates.
        let messages = [a_message("Water bill")];
        let conversations = [a_conversation("Bee swarm")];
        let listed = Listed {
            showing: Showing::Conversations,
            messages: &messages,
            conversations: &conversations,
        };

        assert_eq!(painted(listed, 0, 0), "Bee swarm");
        assert_eq!(
            painted(
                Listed {
                    showing: Showing::Messages,
                    ..listed
                },
                0,
                0
            ),
            "Water bill"
        );
    }
}

//! Putting the message list in the order somebody asked for.
//!
//! Its own module rather than a corner of the main window, so that
//! `tests/the_list_at_two_hundred_thousand_rows.rs` can time it over the
//! sample mailbox with no window and `cargo mutants` can reach it. The two
//! tests that hold each order live in `wx_app.rs` beside the menu that
//! offers them, unchanged by the move, and two guard records break lines in
//! here to prove they still notice.
//!
//! The sort runs off the interface thread: `apply_sort` in `wx_app.rs`
//! clones the rows, sorts them here and sends them back through
//! `MessagesLoaded`, so a slow sort is a wait and not a freeze.

use super::ui_types::{MailSortOption, MessageItem};

/// Sort messages in-place according to the given sort option.
pub fn sort_messages(messages: &mut [MessageItem], order: MailSortOption) {
    match order {
        MailSortOption::DateNewestFirst => messages.sort_by(|a, b| b.date.cmp(&a.date)),
        MailSortOption::DateOldestFirst => messages.sort_by(|a, b| a.date.cmp(&b.date)),
        MailSortOption::SenderAZ => messages.sort_by_key(|a| a.from.to_lowercase()),
        MailSortOption::SenderZA => {
            messages.sort_by_key(|a| std::cmp::Reverse(a.from.to_lowercase()))
        }
        MailSortOption::SubjectAZ => messages.sort_by_key(|a| a.subject.to_lowercase()),
        MailSortOption::SubjectZA => {
            messages.sort_by_key(|a| std::cmp::Reverse(a.subject.to_lowercase()))
        }
        MailSortOption::UnreadFirst => messages.sort_by_key(|a| a.read),
    }
}

//! A mailbox large enough to tell whether the list actually scales.
//!
//! Its own module rather than a corner of the main window, because the rows
//! it makes are what the scale numbers on `docs/development/measurements.md`
//! are taken over, and a generator that drifted would move the numbers. The
//! tests here pin its shape, `tests/the_list_at_two_hundred_thousand_rows.rs`
//! builds rows through it with no window, and `cargo mutants` can reach it,
//! which it could not while it sat in `wx_app.rs`.

use super::ui_types::MessageItem;

/// How many messages the sample mailbox generates.
pub const SAMPLE_MAILBOX_SIZE: usize = 200_000;

/// Build a mailbox large enough to tell whether the list actually scales.
///
/// This exists to be tested with a screen reader. Claims about a list holding
/// two hundred thousand rows are worth nothing until someone arrows through one,
/// and waiting for a real mailbox that size to sync is not a reasonable way to
/// find out that it does not work.
///
/// Deliberately reachable from the Help menu rather than hidden behind a build
/// flag, because the people who most need to test it are not the people
/// compiling it.
pub fn sample_mailbox(count: usize) -> Vec<MessageItem> {
    let senders = [
        "Ada Lovelace <ada@example.com>",
        "Grace Hopper <grace@example.com>",
        "Alan Turing <alan@example.com>",
        "no-reply@example.com",
    ];
    let subjects = [
        "Quarterly report",
        "Re: schedule for next week",
        "Invoice 4021",
        "Notes from the accessibility review",
        "",
    ];

    (0..count)
        .map(|i| MessageItem {
            uid: i as u32 + 1,
            message_id: i as i64 + 1,
            subject: subjects[i % subjects.len()].to_string(),
            from: senders[i % senders.len()].to_string(),
            // One day, a minute per row, wrapping after 1,440 rows. This said
            // "descending so the newest is first" from the day it was written
            // until 08-04 read it: the rows climb, and it is the default sort
            // that puts the newest first.
            date: format!("2026-07-26 {:02}:{:02}", (i / 60) % 24, i % 60),
            read: i % 3 != 0,
            starred: i % 17 == 0,
            answered: i % 11 == 0,
            draft: false,
            has_attachments: i % 7 == 0,
            attachments: Vec::new(),
            thread_depth: i % 5,
            is_thread_parent: i % 5 == 0,
            thread_id: (i % 5 != 0).then(|| format!("thread-{}", i / 5)),
            snippet: Some(format!(
                "Sample message {} for testing the list at scale.",
                i + 1
            )),
            size_bytes: Some(((i % 40) as i64 + 1) * 1024),
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
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_count_asked_for_is_the_count_returned() {
        assert_eq!(sample_mailbox(0).len(), 0);
        assert_eq!(sample_mailbox(7).len(), 7);
        assert_eq!(sample_mailbox(1_441).len(), 1_441);
    }

    #[test]
    fn test_the_sample_is_two_hundred_thousand_rows() {
        assert_eq!(SAMPLE_MAILBOX_SIZE, 200_000);
    }

    #[test]
    fn test_uids_and_message_ids_are_one_based_and_dense() {
        let rows = sample_mailbox(12);

        assert_eq!(rows.len(), 12, "a loop over no rows would prove nothing");
        for (i, row) in rows.iter().enumerate() {
            assert_eq!(row.uid as usize, i + 1, "row {i}");
            assert_eq!(row.message_id as usize, i + 1, "row {i}");
        }
    }

    #[test]
    fn test_dates_are_one_day_read_minute_by_minute_and_wrap_after_a_day() {
        // What the generator does, rather than what a comment beside it once
        // said. The hour is `(i / 60) % 24` and the minute `i % 60`, so the
        // first 1,440 rows climb a minute at a time through 2026-07-26 and the
        // row after them repeats the first. The comment used to say the dates
        // descend so the newest is first; they do not, and the default sort
        // is what puts the newest first.
        let rows = sample_mailbox(1_441);

        assert_eq!(rows[0].date, "2026-07-26 00:00");
        assert_eq!(rows[1].date, "2026-07-26 00:01");
        assert_eq!(rows[60].date, "2026-07-26 01:00");
        assert_eq!(rows[1_439].date, "2026-07-26 23:59");
        assert_eq!(rows[1_440].date, rows[0].date);
        for pair in rows[..1_440].windows(2) {
            assert!(
                pair[0].date < pair[1].date,
                "{} then {}",
                pair[0].date,
                pair[1].date
            );
        }
    }

    #[test]
    fn test_every_fifth_row_is_a_thread_parent_and_the_rest_name_a_thread() {
        let rows = sample_mailbox(10);

        assert_eq!(rows.len(), 10, "a loop over no rows would prove nothing");
        for (i, row) in rows.iter().enumerate() {
            if i % 5 == 0 {
                assert!(row.is_thread_parent, "row {i}");
                assert_eq!(row.thread_id, None, "row {i}");
            } else {
                assert!(!row.is_thread_parent, "row {i}");
                assert_eq!(
                    row.thread_id.as_deref(),
                    Some(format!("thread-{}", i / 5).as_str())
                );
            }
            assert_eq!(row.thread_depth, i % 5, "row {i}");
        }
    }

    #[test]
    fn test_the_fifth_subject_is_empty_and_the_first_is_the_filter_word() {
        // The subjects cycle through five, so a search for the first one
        // hits one row in five; the harness at scale relies on that rate.
        let rows = sample_mailbox(10);

        assert_eq!(rows[0].subject, "Quarterly report");
        assert_eq!(rows[5].subject, "Quarterly report");
        assert_eq!(rows[4].subject, "");
        assert_eq!(rows[9].subject, "");
        assert_eq!(
            rows.iter()
                .filter(|r| r.subject == "Quarterly report")
                .count(),
            2
        );
    }

    #[test]
    fn test_one_row_in_three_is_unread() {
        let rows = sample_mailbox(9);

        let unread: Vec<usize> = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.read)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(unread, vec![0, 3, 6]);
    }
}

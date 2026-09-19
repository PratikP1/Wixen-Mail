//! Where the cursor lands on the message list: after rows leave it, whether
//! a re-read of the folder should move it, and when focus arrives on a list
//! with no row under the cursor.
//!
//! The tester on 2026-09-18 (#76), under NVDA: deleting a message puts the
//! cursor at the top of the list. The rule he asked for, the next message or
//! the previous one when the last was deleted, was already written into the
//! window's own record of the selection; nothing set the control's focused
//! and selected item to it after the count changed, and the control is what a
//! screen reader follows. The rules live here, as cases, so the window asks
//! them and a test can hold them without a window.
//!
//! The same day (#87): Tab from the folder tree to the message list lands on
//! the list itself, read as "list" with no row, and Down then lands on the
//! first row. Opening a folder loads the rows and, by 10-02's rule, selects
//! nothing while focus is in the tree; nothing landed a row when focus later
//! arrived. Where it lands is the fourth question here, since it is the same
//! question as the other three: where the cursor lands on the list.
//!
//! Four questions, each one function:
//!
//! - [`where_to_land`]: which row the cursor lands on once `removed` rows
//!   have left, for one row or a set.
//! - [`where_the_same_message_is`]: which row the cursor's message sits on
//!   after the rows were replaced by a re-read, found by its identity and not
//!   its position.
//! - [`whether_to_move`]: whether that is a move at all. 10-02 refused to
//!   re-select on every load, because re-selecting moves focus and takes the
//!   screen reader's cursor with it; that stands for a load that leaves the
//!   cursor's message where it was, and only a load that moved it moves the
//!   cursor.
//! - [`where_to_land_on_arrival`]: which row the cursor lands on when focus
//!   arrives and no row is under it: the remembered row when there is one
//!   and it is still there, else the first row under the sort.
//!
//! Every index is a row of the list. `removed` holds rows as they were
//! numbered before they left; the answer is a row as it is numbered after.

/// The row the cursor lands on after `removed` rows have left a list that
/// now holds `len_after` rows, or nothing when there is no row to land on.
///
/// For one row: the row that followed it, which now sits where it sat, or
/// the row before it when the last row was removed. For a set: the row after
/// the last removed when there is one, else the row before the first, else
/// the last row left. Nothing removed is nothing to land after, so the
/// cursor stays where it is.
pub fn where_to_land(removed: &[usize], len_after: usize) -> Option<usize> {
    let first = *removed.iter().min()?;
    let last = *removed.iter().max()?;
    if len_after == 0 {
        return None;
    }
    let len_before = len_after + removed.len();
    // The row after the set, numbered as it is now: every removed row sat
    // below it, so it moved up by the size of the set.
    let after_the_set = (last + 1 < len_before).then(|| last + 1 - removed.len());
    // The row before the set did not move: nothing removed sat below it.
    let before_the_set = (first > 0).then(|| first - 1);
    // Neither exists only when the set holds both ends of the list; the last
    // row left is the previous message in the sense the last-row case means.
    Some(
        after_the_set
            .or(before_the_set)
            .unwrap_or(len_after - 1)
            .min(len_after - 1),
    )
}

/// The row the cursor's message sits on now, by its identity, or nothing
/// when there was no cursor or the message is no longer listed.
pub fn where_the_same_message_is(cursor: Option<i64>, ids_after: &[i64]) -> Option<usize> {
    let cursor = cursor?;
    ids_after.iter().position(|id| *id == cursor)
}

/// The row to move the cursor to, or nothing when it is already there.
///
/// A load that leaves the cursor's message at the same row moves nothing,
/// which keeps 10-02's rule for the case it was written for; a load after a
/// removal, where the same row is a different message, is the case it did
/// not cover.
pub fn whether_to_move(old_index: Option<usize>, new_index: Option<usize>) -> Option<usize> {
    new_index.filter(|new| Some(*new) != old_index)
}

/// The row the cursor lands on when focus arrives on a list of `len` rows
/// with no row under the cursor (#87): the remembered row when there is one
/// and it is still there, else the first row under the sort, and nothing
/// when the list holds no row.
pub fn where_to_land_on_arrival(remembered: Option<usize>, len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(remembered.filter(|row| *row < len).unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_middle_row_removed_lands_on_the_row_that_followed_it() {
        // Five rows, row 2 deleted: the row that was 3 is 2 now, and that is
        // the next message.
        assert_eq!(where_to_land(&[2], 4), Some(2));
    }

    #[test]
    fn test_the_last_row_removed_lands_on_the_row_before_it() {
        // Row 4 of five deleted: nothing follows, so the previous one.
        assert_eq!(where_to_land(&[4], 4), Some(3));
    }

    #[test]
    fn test_the_only_row_removed_lands_nowhere() {
        assert_eq!(where_to_land(&[0], 0), None);
        assert_eq!(where_to_land(&[0, 1, 2, 3, 4], 0), None);
    }

    #[test]
    fn test_nothing_removed_is_nothing_to_land_after() {
        assert_eq!(where_to_land(&[], 5), None);
    }

    #[test]
    fn test_a_set_at_the_end_lands_on_the_row_before_the_first_removed() {
        // Rows 3 and 4 of five: nothing follows the set, so row 2.
        assert_eq!(where_to_land(&[3, 4], 3), Some(2));
    }

    #[test]
    fn test_a_set_at_the_start_lands_on_the_row_after_the_last_removed() {
        // Rows 0 and 1 of five: the row that was 2 is 0 now.
        assert_eq!(where_to_land(&[0, 1], 3), Some(0));
    }

    #[test]
    fn test_a_set_with_a_gap_lands_after_its_last_row_counted_as_it_is_now() {
        // Rows 1 and 3 of five: the row that was 4 has two removed rows
        // below it and is 2 now.
        assert_eq!(where_to_land(&[1, 3], 3), Some(2));
    }

    #[test]
    fn test_a_set_holding_both_ends_lands_on_the_last_row_left() {
        // Rows 0 and 4 of five: nothing after, nothing before, so the last
        // of the three left.
        assert_eq!(where_to_land(&[0, 4], 3), Some(2));
    }

    #[test]
    fn test_the_cursors_message_is_found_by_its_identity_after_a_re_read() {
        // The message with id 30 was on row 1; after the re-read it is on
        // row 3, because two arrived above it.
        assert_eq!(
            where_the_same_message_is(Some(30), &[10, 40, 50, 30, 20]),
            Some(3)
        );
    }

    #[test]
    fn test_a_message_no_longer_listed_or_no_cursor_is_found_nowhere() {
        assert_eq!(where_the_same_message_is(Some(30), &[10, 20]), None);
        assert_eq!(where_the_same_message_is(None, &[10, 20, 30]), None);
    }

    #[test]
    fn test_a_message_that_moved_rows_is_a_move() {
        assert_eq!(whether_to_move(Some(1), Some(3)), Some(3));
        assert_eq!(whether_to_move(None, Some(0)), Some(0));
    }

    #[test]
    fn test_a_message_still_on_its_row_is_no_move_and_neither_is_one_that_is_gone() {
        // 10-02's rule, kept: a load that changes nothing under the cursor
        // moves nothing.
        assert_eq!(whether_to_move(Some(1), Some(1)), None);
        assert_eq!(whether_to_move(Some(1), None), None);
        assert_eq!(whether_to_move(None, None), None);
    }

    #[test]
    fn test_focus_arriving_on_an_empty_list_lands_nowhere() {
        assert_eq!(where_to_land_on_arrival(None, 0), None);
        assert_eq!(where_to_land_on_arrival(Some(3), 0), None);
    }

    #[test]
    fn test_focus_arriving_with_a_remembered_row_still_there_lands_on_it() {
        // The row somebody was on in this folder, still inside the rows the
        // list holds.
        assert_eq!(where_to_land_on_arrival(Some(3), 5), Some(3));
        assert_eq!(where_to_land_on_arrival(Some(4), 5), Some(4));
    }

    #[test]
    fn test_focus_arriving_with_a_remembered_row_past_the_end_lands_on_the_first() {
        // A shorter list than the one the row was remembered in: the row is
        // gone, so the first row under the sort.
        assert_eq!(where_to_land_on_arrival(Some(5), 5), Some(0));
        assert_eq!(where_to_land_on_arrival(Some(40), 5), Some(0));
    }

    #[test]
    fn test_focus_arriving_with_nothing_remembered_lands_on_the_first_row() {
        // A folder just opened: the newest message under the default sort,
        // the sort's first row under any other.
        assert_eq!(where_to_land_on_arrival(None, 5), Some(0));
        assert_eq!(where_to_land_on_arrival(None, 1), Some(0));
    }
}

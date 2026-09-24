//! How a time moves in blocks: where a new event starts and ends, and where
//! Up, Down, Left and Right take a time in the editors.
//!
//! The tester on 2026-09-15 (#41): "each event should be blocked for 30
//! minutes by default unless the user has indicated otherwise in settings ...
//! The configuration should allow for 15 minutes, 30 minutes, or 1 hour ...
//! Pressing up and down arrow keys when picking time should move in those
//! blocks. However the user should be able to choose other times minutely
//! using left and right arrow keys." His answers the same day are the rules
//! here: a new event starts at the next boundary after now, so 14:37 with
//! half an hour gives 15:00 and not 14:30; the end moves with the start until
//! the person has changed the end; Left and Right move one minute.
//!
//! Every rule is a function over a date and time and a [`Block`], and none
//! touches a control, so each is held at its boundaries here. A time that
//! crosses midnight carries the day change in its date rather than in a second
//! return value.

use chrono::NaiveDateTime;

/// How long a new event lasts, and how far Up and Down move a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Block {
    Fifteen,
    /// The tester's default: "30 minutes by default unless the user has
    /// indicated otherwise in settings".
    #[default]
    Thirty,
    Sixty,
}

impl Block {
    /// Every block, in the order Settings offers them.
    pub const ALL: [Block; 3] = [Block::Fifteen, Block::Thirty, Block::Sixty];

    /// The block's length in minutes, which is also how it is stored.
    pub fn minutes(self) -> u32 {
        15
    }

    /// The block as Settings lists it.
    pub fn label(self) -> &'static str {
        ""
    }

    /// The block a stored number of minutes names.
    pub fn from_setting(minutes: u32) -> Block {
        let _ = minutes;
        Block::Fifteen
    }
}

/// The first block boundary strictly after `now`: where a new event starts.
pub fn next_boundary(now: NaiveDateTime, block: Block) -> NaiveDateTime {
    let _ = block;
    now
}

/// Where Up (`up`) or Down takes a time: one block from a boundary, or to the
/// nearest boundary in that direction from anywhere else.
pub fn step_by_block(at: NaiveDateTime, block: Block, up: bool) -> NaiveDateTime {
    let _ = (block, up);
    at
}

/// Where Right (`forward`) or Left takes a time: one minute.
pub fn step_by_minute(at: NaiveDateTime, forward: bool) -> NaiveDateTime {
    let _ = forward;
    at
}

/// Where a new event ends: one block after it starts.
pub fn end_after(start: NaiveDateTime, block: Block) -> NaiveDateTime {
    let _ = block;
    start
}

/// Where the end goes when the start moves from `old_start` to `new_start`:
/// by the same amount, unless the person has changed the end, and then it
/// stays where they put it.
pub fn follow(
    old_start: NaiveDateTime,
    new_start: NaiveDateTime,
    end: NaiveDateTime,
    end_edited: bool,
) -> NaiveDateTime {
    let _ = (old_start, new_start, end_edited);
    end
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A moment on 24 September 2026, the day this was written.
    fn at(hour: u32, minute: u32, second: u32) -> NaiveDateTime {
        on(24, hour, minute, second)
    }

    fn on(day: u32, hour: u32, minute: u32, second: u32) -> NaiveDateTime {
        chrono::NaiveDate::from_ymd_opt(2026, 9, day)
            .and_then(|date| date.and_hms_opt(hour, minute, second))
            .expect("a real moment")
    }

    #[test]
    fn test_the_three_blocks_are_fifteen_thirty_and_sixty_minutes_in_that_order() {
        let minutes: Vec<u32> = Block::ALL.iter().map(|block| block.minutes()).collect();
        assert_eq!(minutes, [15, 30, 60]);
    }

    #[test]
    fn test_each_block_is_listed_in_words() {
        let labels: Vec<&str> = Block::ALL.iter().map(|block| block.label()).collect();
        assert_eq!(labels, ["15 minutes", "30 minutes", "1 hour"]);
    }

    #[test]
    fn test_a_stored_length_names_its_block_and_anything_else_is_half_an_hour() {
        assert_eq!(Block::from_setting(15), Block::Fifteen);
        assert_eq!(Block::from_setting(30), Block::Thirty);
        assert_eq!(Block::from_setting(60), Block::Sixty);
        for stranger in [0, 1, 45, 90, u32::MAX] {
            assert_eq!(Block::from_setting(stranger), Block::Thirty, "{stranger}");
        }
    }

    #[test]
    fn test_the_stored_default_is_the_blocks_default() {
        assert_eq!(
            crate::data::config::AppConfig::default().event_length_minutes,
            Block::default().minutes()
        );
    }

    #[test]
    fn test_a_new_event_at_14_37_starts_at_15_00_not_14_30() {
        assert_eq!(next_boundary(at(14, 37, 10), Block::Thirty), at(15, 0, 0));
    }

    #[test]
    fn test_a_new_event_on_a_boundary_starts_at_the_next_one() {
        assert_eq!(next_boundary(at(14, 30, 0), Block::Thirty), at(15, 0, 0));
        assert_eq!(next_boundary(at(14, 45, 0), Block::Fifteen), at(15, 0, 0));
    }

    #[test]
    fn test_a_new_event_with_an_hour_starts_on_the_next_hour() {
        assert_eq!(next_boundary(at(14, 5, 0), Block::Sixty), at(15, 0, 0));
        assert_eq!(next_boundary(at(14, 59, 59), Block::Sixty), at(15, 0, 0));
    }

    #[test]
    fn test_a_new_event_late_in_the_evening_starts_at_midnight_the_next_day() {
        assert_eq!(
            next_boundary(at(23, 50, 0), Block::Fifteen),
            on(25, 0, 0, 0)
        );
        assert_eq!(next_boundary(at(23, 30, 0), Block::Thirty), on(25, 0, 0, 0));
    }

    #[test]
    fn test_up_from_a_boundary_moves_one_block() {
        assert_eq!(
            step_by_block(at(15, 0, 0), Block::Thirty, true),
            at(15, 30, 0)
        );
        assert_eq!(
            step_by_block(at(15, 0, 0), Block::Fifteen, true),
            at(15, 15, 0)
        );
        assert_eq!(
            step_by_block(at(15, 0, 0), Block::Sixty, true),
            at(16, 0, 0)
        );
    }

    #[test]
    fn test_down_from_a_boundary_moves_one_block() {
        assert_eq!(
            step_by_block(at(15, 0, 0), Block::Thirty, false),
            at(14, 30, 0)
        );
        assert_eq!(
            step_by_block(at(15, 0, 0), Block::Sixty, false),
            at(14, 0, 0)
        );
    }

    #[test]
    fn test_off_a_boundary_up_and_down_go_to_the_boundaries_either_side() {
        assert_eq!(
            step_by_block(at(14, 37, 0), Block::Thirty, true),
            at(15, 0, 0)
        );
        assert_eq!(
            step_by_block(at(14, 37, 0), Block::Thirty, false),
            at(14, 30, 0)
        );
        assert_eq!(
            step_by_block(at(14, 37, 0), Block::Fifteen, true),
            at(14, 45, 0)
        );
        assert_eq!(
            step_by_block(at(14, 37, 0), Block::Sixty, false),
            at(14, 0, 0)
        );
    }

    #[test]
    fn test_up_and_down_roll_the_hour_and_the_day_at_midnight() {
        assert_eq!(
            step_by_block(at(23, 45, 0), Block::Fifteen, true),
            on(25, 0, 0, 0)
        );
        assert_eq!(
            step_by_block(at(0, 0, 0), Block::Fifteen, false),
            on(23, 23, 45, 0)
        );
        assert_eq!(
            step_by_block(at(9, 30, 0), Block::Thirty, true),
            at(10, 0, 0)
        );
    }

    #[test]
    fn test_up_at_a_quarter_to_noon_reaches_noon() {
        // On a twelve-hour face, 11:45 AM and then 12:00 PM: the face follows
        // the hour, so the rule only has to reach noon.
        assert_eq!(
            step_by_block(at(11, 45, 0), Block::Thirty, true),
            at(12, 0, 0)
        );
        assert_eq!(
            step_by_block(at(12, 0, 0), Block::Thirty, false),
            at(11, 30, 0)
        );
    }

    #[test]
    fn test_left_and_right_move_one_minute_across_the_hour_and_the_day() {
        assert_eq!(step_by_minute(at(14, 30, 0), true), at(14, 31, 0));
        assert_eq!(step_by_minute(at(14, 59, 0), true), at(15, 0, 0));
        assert_eq!(step_by_minute(at(15, 0, 0), false), at(14, 59, 0));
        assert_eq!(step_by_minute(at(0, 0, 0), false), on(23, 23, 59, 0));
    }

    #[test]
    fn test_a_new_event_ends_one_block_after_it_starts() {
        assert_eq!(end_after(at(15, 0, 0), Block::Thirty), at(15, 30, 0));
        assert_eq!(end_after(at(15, 0, 0), Block::Sixty), at(16, 0, 0));
        assert_eq!(end_after(at(23, 30, 0), Block::Sixty), on(25, 0, 30, 0));
    }

    #[test]
    fn test_the_end_moves_with_the_start_by_the_same_amount() {
        assert_eq!(
            follow(at(15, 0, 0), at(15, 30, 0), at(15, 30, 0), false),
            at(16, 0, 0)
        );
        assert_eq!(
            follow(at(15, 0, 0), at(14, 31, 0), at(15, 30, 0), false),
            at(15, 1, 0)
        );
    }

    #[test]
    fn test_an_end_the_person_changed_stays_where_they_put_it() {
        assert_eq!(
            follow(at(15, 0, 0), at(15, 30, 0), at(16, 0, 0), true),
            at(16, 0, 0)
        );
    }
}

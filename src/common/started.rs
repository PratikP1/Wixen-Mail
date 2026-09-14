//! When this process started, and the one line it writes when its list is
//! usable.
//!
//! PERF-02 asks for the time from process start to a usable message list,
//! and nothing in the tree measured either end. This module holds both: the
//! instant `main` takes as its first instruction, and the line the window
//! layer writes, once, at the moment the list can answer for a row.
//!
//! **Usable** means the first `MessagesLoaded` after startup whose rows
//! reached the list control's count and were at least one. Not the window
//! appearing, not the frame being shown: a frame over an empty list is not a
//! mailbox anybody can read. An empty load does not count either, because an
//! empty list is not a usable inbox and PERF-02's own words are "usable
//! message list".
//!
//! A sort or a folder change fires the same handler, and neither is startup.
//! The once-guard is what keeps the line honest: it is written the first time
//! rows reach the list, and a second load says nothing, however many loads
//! follow.
//!
//! The line's shape is fixed, `the message list is usable: N rows, M ms after
//! start`, because a harness parses it. `tests/the_numbers_the_targets_ask_for.rs`
//! builds the line through [`usable_line`] and parses it back, so the two
//! cannot drift apart without a test going red.

use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

/// A start that is marked once and reports once.
///
/// One of these for the process, [`THE_PROCESS`], and one per test, so a test
/// of the once-guard is not sharing its state with every other test in the
/// binary.
pub struct Start {
    at: OnceLock<Instant>,
    said_usable: AtomicBool,
}

impl Start {
    /// Unmarked, and with nothing said.
    pub const fn new() -> Self {
        Self {
            at: OnceLock::new(),
            said_usable: AtomicBool::new(false),
        }
    }

    /// Take the instant, answering whether this call was the first.
    ///
    /// A second call is ignored, not refused: the first instant is the start
    /// and nothing later can move it. `false` says the mark was already there.
    pub fn mark(&self) -> bool {
        let _ = (&self.at, &self.said_usable);
        false
    }

    /// How long since the mark, or `None` when nothing has marked it.
    pub fn elapsed(&self) -> Option<Duration> {
        let _ = self;
        None
    }

    /// The usable line, the first time the list holds a row, and `None` ever
    /// after.
    ///
    /// `None` for a load of no rows, without spending the once: an empty list
    /// is not a usable inbox, and the first load that holds a row still gets
    /// its line. `None` too when nothing marked the start, because a duration
    /// from nowhere is not a measurement.
    pub fn say_usable_once(&self, rows: usize) -> Option<String> {
        let _ = (self, rows);
        None
    }
}

impl Default for Start {
    fn default() -> Self {
        Self::new()
    }
}

/// The process's own start.
static THE_PROCESS: Start = Start::new();

/// Take the process's start instant. The first thing `main` does.
pub fn mark() -> bool {
    THE_PROCESS.mark()
}

/// How long the process has been running, or `None` before `main` marked it.
pub fn elapsed() -> Option<Duration> {
    THE_PROCESS.elapsed()
}

/// The process's usable line, once.
pub fn say_usable_once(rows: usize) -> Option<String> {
    THE_PROCESS.say_usable_once(rows)
}

/// Word the usable line.
///
/// Plain digits and no separators in either number, because the harness
/// parses them and a thousands separator is locale.
pub fn usable_line(rows: usize, since_start: Duration) -> String {
    let _ = (rows, since_start);
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nothing_has_elapsed_before_the_mark_and_something_has_after() {
        let start = Start::new();

        assert_eq!(start.elapsed(), None, "no mark, so no duration to report");

        assert!(start.mark(), "the first mark is the one that sets it");
        assert!(
            start.elapsed().is_some(),
            "marked, so there is a duration to report"
        );
    }

    #[test]
    fn test_a_second_mark_is_ignored_and_says_so() {
        let start = Start::new();
        assert!(start.mark());
        std::thread::sleep(Duration::from_millis(5));

        assert!(
            !start.mark(),
            "a second mark answers false: the first instant is the start"
        );
        assert!(
            start
                .elapsed()
                .is_some_and(|since| since >= Duration::from_millis(5)),
            "the second mark did not move the start forward"
        );
    }

    #[test]
    fn test_the_usable_line_is_held_byte_for_byte() {
        assert_eq!(
            usable_line(1234, Duration::from_millis(5678)),
            "the message list is usable: 1234 rows, 5678 ms after start"
        );
    }

    #[test]
    fn test_the_usable_line_is_said_once_per_start() {
        let start = Start::new();
        start.mark();

        let first = start.say_usable_once(500);
        assert!(
            first
                .as_deref()
                .is_some_and(|line| line.starts_with("the message list is usable: 500 rows, ")),
            "the first load with rows gets the line, got {first:?}"
        );
        assert_eq!(
            start.say_usable_once(500),
            None,
            "a second load says nothing"
        );
        assert_eq!(start.say_usable_once(12), None, "and a third says nothing");
    }

    #[test]
    fn test_an_empty_load_is_not_usable_and_does_not_spend_the_once() {
        let start = Start::new();
        start.mark();

        assert_eq!(
            start.say_usable_once(0),
            None,
            "an empty list is not a usable inbox"
        );
        assert!(
            start.say_usable_once(3).is_some(),
            "the first load that holds a row still gets its line"
        );
    }

    #[test]
    fn test_an_unmarked_start_has_no_usable_line() {
        let start = Start::new();

        assert_eq!(
            start.say_usable_once(3),
            None,
            "a duration from nowhere is not a measurement"
        );
    }
}

//! How long to wait before asking a mail server again, after it said no.
//!
//! One rule, for the two long runs this program makes against somebody
//! else's server: the download that brings every message of every kept
//! folder down (10-05), and the watch that holds a connection open for new
//! mail (10-06). A provider that refuses a fetch and one that drops an IDLE
//! connection are the same fact from this side: the server is not answering,
//! and asking again at once, and again, and again, is the reconnect storm
//! that gets a client blocked. Two waits written apart would drift, and the
//! one that drifted shorter would be the one a provider noticed.
//!
//! # What the rule is
//!
//! The first wait is thirty seconds. Each failure after that doubles it, and
//! it never goes past thirty minutes. A success puts it back to the start, so
//! a watch that ran for an hour and then dropped is not punished for the
//! failures of the morning.
//!
//! **The numbers are a decision of 2026-09-17, not a measurement.** No
//! provider has been observed refusing anything: ledger 64 records that the
//! one immediate sign-in after a dropped connection has met only a loopback
//! server, 65 that nobody knows what a provider does with an idle session,
//! and 72 that nobody knows how a provider treats a whole-folder request.
//! Thirty seconds is short enough that a blip costs nobody more than a
//! moment; the cap is half an hour because RFC 2177 gives an IDLE connection
//! twenty-nine minutes before the server may drop it, so a client that waits
//! longer than that between attempts is waiting longer than the protocol's
//! own silence. A provider observed doing something else would move both.
//!
//! # What it is not
//!
//! It carries no clock and it does no sleeping. It answers a [`Duration`],
//! and whoever asked waits that long, which on the window is the main timer.
//! That is what makes it testable against a list of calls rather than
//! against time, and it is why `service::google_api::with_retry` is not
//! reused here: that helper sleeps inside an async call on a runtime thread,
//! for at most a few seconds, and decides its own number of attempts. A wait
//! of up to thirty minutes between attempts at a mail server has to be served
//! by the main timer from a value handed back, and that value is what this
//! type answers.

use std::time::Duration;

/// The wait after the first failure.
pub const FIRST_WAIT: Duration = Duration::from_secs(30);

/// The wait it never goes past.
pub const LONGEST_WAIT: Duration = Duration::from_secs(30 * 60);

/// How long to wait before asking again, told each failure and each success.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WaitBeforeTryingAgain {
    failures_in_a_row: u32,
}

impl WaitBeforeTryingAgain {
    /// Nothing has failed yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// The server did not answer: how long to wait before asking again.
    ///
    /// Counts the failure, so the next answer is longer than this one until
    /// the cap.
    pub fn next_wait(&mut self) -> Duration {
        // Doubled per failure after the first, and saturating rather than
        // wrapping, because a watch left running for a week would otherwise
        // shift past the width of the number and start again from nothing.
        let doublings = self.failures_in_a_row.min(31);
        self.failures_in_a_row = self.failures_in_a_row.saturating_add(1);
        FIRST_WAIT
            .saturating_mul(1u32 << doublings)
            .min(LONGEST_WAIT)
    }

    /// The server answered, so the next wait starts from the beginning again.
    pub fn tell_it_worked(&mut self) {
        self.failures_in_a_row = 0;
    }

    /// How many times in a row the server has not answered.
    ///
    /// So a caller can say "the server has refused three times" and the
    /// download can stop asking after a bound of its own.
    pub fn how_many_failures_in_a_row(&self) -> u32 {
        self.failures_in_a_row
    }
}

/// What to say before waiting.
///
/// What happened and what happens next, in the words a person uses for a
/// length of time: "in 30 seconds", "in 1 minute", "in 2 minutes", never "in
/// 120 seconds". No protocol vocabulary, because the person hearing this did
/// not choose IMAP and cannot do anything about it.
pub fn what_to_say_before_waiting(wait: Duration, failures_in_a_row: u32) -> String {
    // The first failure is one sentence. A run of them says how long the run
    // is, because "still cannot be reached" and "could not be reached, once"
    // want different decisions from the person hearing them.
    let what_happened = match failures_in_a_row {
        0 | 1 => "The mail server could not be reached.".to_string(),
        times => format!("The mail server could not be reached {times} times in a row."),
    };
    format!(
        "{what_happened} Trying again in {}.",
        said_as_a_person_says_it(wait)
    )
}

/// A length of time in the words a person uses for one.
///
/// Whole minutes as minutes, anything shorter as seconds, and a wait that is
/// both as both. Never a bare count of seconds above a minute, which is a
/// number somebody has to divide in their head.
fn said_as_a_person_says_it(wait: Duration) -> String {
    let seconds = wait.as_secs();
    let (minutes, rest) = (seconds / 60, seconds % 60);
    match (minutes, rest) {
        (0, seconds) => crate::service::caldav::how_many(seconds as usize, "second"),
        (minutes, 0) => crate::service::caldav::how_many(minutes as usize, "minute"),
        (minutes, seconds) => format!(
            "{} {}",
            crate::service::caldav::how_many(minutes as usize, "minute"),
            crate::service::caldav::how_many(seconds as usize, "second")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minutes(count: u64) -> Duration {
        Duration::from_secs(count * 60)
    }

    #[test]
    fn test_the_first_wait_is_thirty_seconds() {
        let mut wait = WaitBeforeTryingAgain::new();

        assert_eq!(wait.next_wait(), Duration::from_secs(30));
    }

    #[test]
    fn test_each_failure_doubles_the_wait_until_the_cap() {
        // The whole shape, as a list, because a rule that doubled once and
        // then stopped would pass a test of the second call alone.
        let mut wait = WaitBeforeTryingAgain::new();

        let answers: Vec<Duration> = (0..9).map(|_| wait.next_wait()).collect();

        assert_eq!(
            answers,
            vec![
                Duration::from_secs(30),
                minutes(1),
                minutes(2),
                minutes(4),
                minutes(8),
                minutes(16),
                minutes(30),
                minutes(30),
                minutes(30),
            ]
        );
    }

    #[test]
    fn test_the_wait_never_goes_past_thirty_minutes() {
        // Asked far more often than any real run would, because the cap is
        // what stops a doubling from overflowing or from waiting a day.
        let mut wait = WaitBeforeTryingAgain::new();

        for _ in 0..100 {
            assert!(wait.next_wait() <= LONGEST_WAIT);
        }
        assert_eq!(wait.next_wait(), LONGEST_WAIT);
    }

    #[test]
    fn test_a_success_puts_the_wait_back_to_the_start() {
        // A watch that ran for an hour and then dropped is not punished for
        // the failures of the morning.
        let mut wait = WaitBeforeTryingAgain::new();
        for _ in 0..5 {
            wait.next_wait();
        }

        wait.tell_it_worked();

        assert_eq!(wait.next_wait(), FIRST_WAIT);
    }

    #[test]
    fn test_the_failures_in_a_row_are_counted_and_a_success_clears_them() {
        let mut wait = WaitBeforeTryingAgain::new();
        assert_eq!(wait.how_many_failures_in_a_row(), 0);

        wait.next_wait();
        wait.next_wait();
        wait.next_wait();
        assert_eq!(wait.how_many_failures_in_a_row(), 3);

        wait.tell_it_worked();
        assert_eq!(wait.how_many_failures_in_a_row(), 0);
    }

    #[test]
    fn test_the_wait_is_said_the_way_a_person_says_it() {
        // Seconds below a minute, minutes from a minute up, and never a bare
        // count of seconds somebody has to divide in their head.
        let cases = [
            (Duration::from_secs(30), "in 30 seconds"),
            (minutes(1), "in 1 minute"),
            (minutes(2), "in 2 minutes"),
            (minutes(30), "in 30 minutes"),
        ];
        for (wait, expected) in cases {
            let said = what_to_say_before_waiting(wait, 1);
            assert!(
                said.contains(expected),
                "a wait of {wait:?} was not said as \"{expected}\": {said}"
            );
        }
    }

    #[test]
    fn test_no_wait_is_said_as_a_bare_number_of_seconds_above_a_minute() {
        for wait in [minutes(1), minutes(2), minutes(16), minutes(30)] {
            let said = what_to_say_before_waiting(wait, 3);
            let seconds = format!("{} seconds", wait.as_secs());
            assert!(
                !said.contains(&seconds),
                "a wait of {wait:?} was said in seconds: {said}"
            );
        }
    }

    #[test]
    fn test_the_sentence_says_what_happened_and_what_happens_next() {
        // Plain words, in that order: what happened, then what will happen.
        // No protocol vocabulary, because the person hearing this did not
        // choose IMAP and cannot do anything about it.
        let said = what_to_say_before_waiting(minutes(2), 1);

        assert!(
            said.starts_with("The mail server could not be reached."),
            "the sentence does not lead with what happened: {said}"
        );
        assert!(
            said.ends_with("Trying again in 2 minutes."),
            "the sentence does not end with what happens next: {said}"
        );
        for word in ["IMAP", "IDLE", "socket", "timeout", "retry"] {
            assert!(
                !said.contains(word),
                "the sentence uses a word a person did not choose, {word}: {said}"
            );
        }
    }

    #[test]
    fn test_a_run_of_failures_is_said_as_a_count_a_person_can_act_on() {
        // The first failure is one sentence; the fourth in a row says it is
        // the fourth, because "still cannot be reached" and "could not be
        // reached, once" want different decisions from the person hearing
        // them.
        let first = what_to_say_before_waiting(Duration::from_secs(30), 1);
        let fourth = what_to_say_before_waiting(minutes(4), 4);

        assert!(
            !first.contains("in a row"),
            "the first failure was said as a run: {first}"
        );
        assert!(
            fourth.contains("4 times in a row"),
            "the fourth failure in a row does not say so: {fourth}"
        );
    }
}

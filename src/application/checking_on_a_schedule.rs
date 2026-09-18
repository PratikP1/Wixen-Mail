//! Which accounts are due a check, whether a watch that ended is tried
//! again, and what the status line says about either.
//!
//! # Why a schedule at all, when there is a watch
//!
//! The inbox watch (`mail_sync::watch_folder`, one IDLE connection per
//! account) learns about new mail as it lands. Four things it cannot cover,
//! and this module is what covers them:
//!
//! - **A dead socket nothing writes to.** RFC 2177 has the client renew IDLE
//!   every twenty-nine minutes, and a connection that died in between is
//!   noticed at the next renewal at the latest. So the longest silence a
//!   watch can hide is twenty-nine minutes, and a check on a schedule is
//!   what bounds it.
//! - **A server without IDLE**, or one that refuses it. The watch cannot
//!   start; after three refusals in a row it is not asked again this
//!   session, and the schedule alone carries the account.
//! - **POP.** No folders, no flags, nothing to select and nothing to watch:
//!   a POP account is checked and never watched.
//! - **The kept folders that are not the inbox.** The watch is on one
//!   folder. Everything else somebody chose to keep up to date is read by a
//!   check, and only a check.
//!
//! # The interval is the account editor's field, made true
//!
//! The editor has offered "Check Interval (min)" since it was written, the
//! account row has stored it, and until 2026-09-17 nothing read it: a
//! setting a person could set that did nothing, which is guardrail 3's shape
//! (#37). It is read here, clamped the way the editor clamps it, and there
//! is no second interval on the Settings screen, because two settings for
//! one interval is the drift `CLAUDE.md`'s settings rule exists to stop.
//!
//! # What is not here
//!
//! No clock, no thread and no connection. [`which_are_due`] is asked over
//! instants somebody hands in, [`whether_to_watch_again`] over a reason and
//! a count, and [`what_the_status_line_says`] over a description of what is
//! running; the window serves the answers from its main timer. That is what
//! makes every decision here testable in milliseconds, and it is why the
//! decisions are here rather than in the window, where a test costs the
//! sixty-six guard records that fingerprint that file.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::common::types::Protocol;
use crate::data::account::Account;

/// The shortest interval the editor accepts, in minutes.
///
/// A stored nought is read as this and not as "every tick", because a
/// check every fifty milliseconds is a reconnect storm with a setting for a
/// cause.
pub const SHORTEST_INTERVAL_MINUTES: u32 = 1;

/// The longest interval the editor accepts, in minutes.
pub const LONGEST_INTERVAL_MINUTES: u32 = 60;

/// How many times in a row a watch may fail to start before the schedule
/// alone carries the account.
///
/// Three, a decision of 2026-09-17 rather than a measurement: one refusal
/// is a blip, two is a coincidence, three is a server that does not offer
/// what was asked. No provider has been observed refusing IDLE.
pub const REFUSALS_BEFORE_THE_SCHEDULE_ALONE: u32 = 3;

/// What the schedule needs to know about one account, and nothing else.
///
/// Built from [`Account`] by `From`, so the schedule cannot read a field
/// the editor does not write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountToCheck {
    pub id: String,
    pub enabled: bool,
    pub protocol: Protocol,
    pub check_interval_minutes: u32,
}

impl From<&Account> for AccountToCheck {
    fn from(account: &Account) -> Self {
        Self {
            id: account.id.clone(),
            enabled: account.enabled,
            protocol: account.protocol(),
            check_interval_minutes: account.check_interval_minutes,
        }
    }
}

/// The interval as a length of time, clamped as the editor clamps it.
pub fn the_interval_of(minutes: u32) -> Duration {
    let minutes = minutes.clamp(SHORTEST_INTERVAL_MINUTES, LONGEST_INTERVAL_MINUTES);
    Duration::from_secs(60 * u64::from(minutes))
}

/// The ids of the enabled accounts whose interval has passed since they were
/// last checked, and of every enabled account never checked this session, in
/// list order.
pub fn which_are_due(
    accounts: &[AccountToCheck],
    last_checked: &HashMap<String, Instant>,
    now: Instant,
) -> Vec<String> {
    accounts
        .iter()
        .filter(|account| account.enabled)
        .filter(|account| {
            last_checked.get(&account.id).is_none_or(|last| {
                now.saturating_duration_since(*last)
                    >= the_interval_of(account.check_interval_minutes)
            })
        })
        .map(|account| account.id.clone())
        .collect()
}

/// Why a watch is not running any more.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhyTheWatchEnded {
    /// It ran, and then `ImapIdleEvent::Stopped` said why it stopped.
    Ended(String),
    /// It never ran: the connection, the sign-in or the select failed before
    /// any watch existed, and `watch_folder` answered an error.
    NeverStarted(String),
}

/// What is left to try a watch again, once the answer is not "after a
/// wait".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Until {
    /// The server could not be reached at all. The next check that reaches
    /// it, or the network coming back, is the moment to try the watch
    /// again.
    TheServerAnswersAgain,
    /// The server was reached and would not watch. Nothing a check can find
    /// out changes that, so the next start of the program is the next try.
    TheProgramStartsAgain,
}

/// Whether to watch again, and if not, what would change the answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchAgain {
    /// After the wait the wait rule answers.
    AfterAWait,
    /// Not from here. Somebody asked for the stop, and whoever asked knows
    /// whether a fresh watch is coming; deciding here would mark an account
    /// whose next watch is already starting.
    Never,
    /// The schedule alone carries the account, until what is named.
    OnTheScheduleAlone(Until),
}

/// What `imap.rs` says when the server was reached and would not watch: a
/// NO to IDLE, or a server without it.
const THE_SERVER_WOULD_NOT_WATCH: &str = "the mail server would not start watching";

/// What `imap.rs` says when the stop was asked for, by the window replacing
/// the watch or closing.
const SOMEBODY_STOPPED_IT: &str = "the watch was stopped";

/// What `imap.rs` says when the window stopped reading events, which is the
/// window having gone.
const NOBODY_WAS_LISTENING: &str = "nobody was listening";

/// Whether a watch that ended for `reason` is tried again, given how many
/// times in a row it has now failed, this failure counted.
///
/// A connection that dropped is tried again after a wait however many times
/// in a row, because the wait rule's cap is what bounds that. A server that
/// was reached and refused to watch, and a server that could not be reached
/// at all, are each given three tries, and then the schedule alone carries
/// the account: for the first until the program starts again, since nothing
/// a check finds out changes what a server offers; for the second until a
/// check reaches the server or the network comes back, since the reason was
/// the reach and not the server.
pub fn whether_to_watch_again(reason: &WhyTheWatchEnded, failures_in_a_row: u32) -> WatchAgain {
    let three_in_a_row = failures_in_a_row >= REFUSALS_BEFORE_THE_SCHEDULE_ALONE;
    match reason {
        WhyTheWatchEnded::Ended(said)
            if said.starts_with(SOMEBODY_STOPPED_IT) || said.starts_with(NOBODY_WAS_LISTENING) =>
        {
            WatchAgain::Never
        }
        WhyTheWatchEnded::Ended(said) if said.starts_with(THE_SERVER_WOULD_NOT_WATCH) => {
            if three_in_a_row {
                WatchAgain::OnTheScheduleAlone(Until::TheProgramStartsAgain)
            } else {
                WatchAgain::AfterAWait
            }
        }
        WhyTheWatchEnded::Ended(_) => WatchAgain::AfterAWait,
        WhyTheWatchEnded::NeverStarted(_) => {
            if three_in_a_row {
                WatchAgain::OnTheScheduleAlone(Until::TheServerAnswersAgain)
            } else {
                WatchAgain::AfterAWait
            }
        }
    }
}

/// What the watch on an account is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheWatch {
    /// A connection is open and the server will say when mail lands.
    Watching,
    /// The last watch ended and the next attempt is this far off.
    Waiting(Duration),
    /// No watch runs and none is coming: the schedule alone.
    NotWatching,
}

/// What is running for one account, as the status line describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatIsRunning<'a> {
    /// The account's name, when more than one account is enabled and the
    /// line has to say whose it is. `None` for a person with one account,
    /// for whom the name is noise.
    pub account: Option<&'a str>,
    /// The folder the watch is on, by the name the tree shows.
    pub folder: &'a str,
    pub watch: TheWatch,
    /// How often the account is checked, from the editor's field.
    pub checking_every: Duration,
}

/// What the status line says about an account: which account, what is
/// happening, how often, in that order, and in the words a person uses.
///
/// "Watching" and "checking" and never "IDLE" or "polling": the person
/// reading this did not choose the protocol and cannot do anything about
/// it.
pub fn what_the_status_line_says(state: &WhatIsRunning<'_>) -> String {
    use super::trying_again::said_as_a_person_says_it;

    let whose = state
        .account
        .map_or(String::new(), |name| format!("{name}: "));
    let folder = state.folder;
    let what_is_happening = match state.watch {
        TheWatch::Watching => format!("Watching {folder} for new mail. "),
        TheWatch::Waiting(wait) => format!(
            "Waiting {} to watch {folder} again. ",
            said_as_a_person_says_it(wait)
        ),
        TheWatch::NotWatching => String::new(),
    };
    // "Every minute" and not "every 1 minute": a count of one is not said
    // as a count.
    let how_often = if state.checking_every == Duration::from_secs(60) {
        "minute".to_string()
    } else {
        said_as_a_person_says_it(state.checking_every)
    };
    format!("{whose}{what_is_happening}Checking every {how_often}.")
}

/// The first line a check of every enabled account says.
///
/// A person with one account is not told how many accounts they have.
pub fn what_a_check_of_them_all_says(how_many: usize) -> String {
    if how_many > 1 {
        format!("Checking {how_many} accounts for new mail...")
    } else {
        "Checking for new mail...".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minutes(count: u64) -> Duration {
        Duration::from_secs(count * 60)
    }

    fn an_account(id: &str, interval: u32) -> AccountToCheck {
        AccountToCheck {
            id: id.to_string(),
            enabled: true,
            protocol: Protocol::Imap,
            check_interval_minutes: interval,
        }
    }

    fn checked(pairs: &[(&str, Instant)]) -> HashMap<String, Instant> {
        pairs.iter().map(|(id, at)| (id.to_string(), *at)).collect()
    }

    // ── Which accounts are due ───────────────────────────────────────────

    #[test]
    fn test_an_account_never_checked_this_session_is_due() {
        let now = Instant::now();
        let accounts = [an_account("work", 5)];

        assert_eq!(
            which_are_due(&accounts, &HashMap::new(), now),
            vec!["work".to_string()]
        );
    }

    #[test]
    fn test_an_account_checked_within_its_interval_is_not_due_yet() {
        let now = Instant::now();
        let accounts = [an_account("work", 5)];
        let last = checked(&[("work", now - minutes(4))]);

        assert!(which_are_due(&accounts, &last, now).is_empty());
    }

    #[test]
    fn test_an_account_whose_interval_has_passed_is_due() {
        let now = Instant::now();
        let accounts = [an_account("work", 5)];
        let last = checked(&[("work", now - minutes(5))]);

        assert_eq!(
            which_are_due(&accounts, &last, now),
            vec!["work".to_string()]
        );
    }

    #[test]
    fn test_a_disabled_account_is_never_due() {
        // Never checked and disabled: the one case where "never checked"
        // does not mean due.
        let now = Instant::now();
        let mut off = an_account("old", 1);
        off.enabled = false;
        let accounts = [off, an_account("work", 5)];

        assert_eq!(
            which_are_due(&accounts, &HashMap::new(), now),
            vec!["work".to_string()]
        );
    }

    #[test]
    fn test_a_stored_interval_of_nought_is_read_as_one_minute_and_not_every_tick() {
        // The editor clamps to 1..=60, but a row can hold nought: written
        // by hand, or by a version that did not clamp. Nought read as "due
        // at once, always" is a check every timer tick against somebody
        // else's server.
        assert_eq!(the_interval_of(0), minutes(1));

        let now = Instant::now();
        let accounts = [an_account("work", 0)];
        let just_checked = checked(&[("work", now - Duration::from_secs(30))]);
        assert!(
            which_are_due(&accounts, &just_checked, now).is_empty(),
            "an interval of nought made an account checked thirty seconds ago due again"
        );
    }

    #[test]
    fn test_a_stored_interval_above_sixty_is_read_as_sixty() {
        assert_eq!(the_interval_of(999), minutes(60));
        assert_eq!(the_interval_of(60), minutes(60));
        assert_eq!(the_interval_of(1), minutes(1));
        assert_eq!(the_interval_of(5), minutes(5));
    }

    #[test]
    fn test_the_due_accounts_come_back_in_list_order() {
        // Three due, one not, and the order is the list's, not the map's.
        let now = Instant::now();
        let accounts = [
            an_account("c", 5),
            an_account("a", 5),
            an_account("fresh", 5),
            an_account("b", 5),
        ];
        let last = checked(&[
            ("c", now - minutes(10)),
            ("a", now - minutes(6)),
            ("fresh", now - minutes(1)),
        ]);

        assert_eq!(
            which_are_due(&accounts, &last, now),
            vec!["c".to_string(), "a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn test_an_account_to_check_is_built_from_the_account_row() {
        // Through `From`, so the schedule reads the field the editor writes
        // and no other.
        let row = Account {
            id: "work".to_string(),
            enabled: false,
            check_interval_minutes: 15,
            protocol: Protocol::Pop3.as_str().to_string(),
            ..Account::default()
        };

        let to_check = AccountToCheck::from(&row);

        assert_eq!(
            to_check,
            AccountToCheck {
                id: "work".to_string(),
                enabled: false,
                protocol: Protocol::Pop3,
                check_interval_minutes: 15,
            }
        );
    }

    // ── Whether a watch is tried again ───────────────────────────────────

    #[test]
    fn test_a_watch_whose_connection_was_lost_is_tried_again_after_a_wait() {
        // However many times in a row: a connection that keeps dropping is
        // what the growing wait is for, and the cap bounds it.
        for reason in [
            "the watch connection was lost",
            "the watch connection failed: broken pipe",
        ] {
            for failures in [1, 3, 20] {
                assert_eq!(
                    whether_to_watch_again(&WhyTheWatchEnded::Ended(reason.to_string()), failures),
                    WatchAgain::AfterAWait,
                    "{reason} after {failures} failures"
                );
            }
        }
    }

    #[test]
    fn test_a_watch_somebody_stopped_is_not_tried_again() {
        // Stopped on purpose, by the window replacing it or closing. And
        // "nobody was listening" is the window having gone: nothing to
        // restart for.
        for reason in ["the watch was stopped", "nobody was listening"] {
            assert_eq!(
                whether_to_watch_again(&WhyTheWatchEnded::Ended(reason.to_string()), 1),
                WatchAgain::Never,
                "{reason}"
            );
        }
    }

    #[test]
    fn test_a_server_that_would_not_start_watching_three_times_is_left_to_the_schedule() {
        // A NO to IDLE, or a server without it. Twice is tried again; the
        // third refusal in a row is a server that does not offer what was
        // asked, and nothing a check can find out changes that.
        let refused = WhyTheWatchEnded::Ended(
            "the mail server would not start watching: IDLE not supported".to_string(),
        );
        let no_reason = WhyTheWatchEnded::Ended(
            "the mail server would not start watching, and gave no reason".to_string(),
        );

        assert_eq!(whether_to_watch_again(&refused, 1), WatchAgain::AfterAWait);
        assert_eq!(whether_to_watch_again(&refused, 2), WatchAgain::AfterAWait);
        assert_eq!(
            whether_to_watch_again(&refused, 3),
            WatchAgain::OnTheScheduleAlone(Until::TheProgramStartsAgain)
        );
        assert_eq!(
            whether_to_watch_again(&no_reason, 3),
            WatchAgain::OnTheScheduleAlone(Until::TheProgramStartsAgain)
        );
    }

    #[test]
    fn test_a_watch_that_never_started_is_tried_again_once_and_left_to_the_schedule_after_three() {
        // The connection, the sign-in or the select failed before any watch
        // existed. Once is a wait; three in a row is a server nobody can
        // reach, and the next check that reaches it is the next try.
        let never = WhyTheWatchEnded::NeverStarted("connection refused".to_string());

        assert_eq!(whether_to_watch_again(&never, 1), WatchAgain::AfterAWait);
        assert_eq!(whether_to_watch_again(&never, 2), WatchAgain::AfterAWait);
        assert_eq!(
            whether_to_watch_again(&never, 3),
            WatchAgain::OnTheScheduleAlone(Until::TheServerAnswersAgain)
        );
        assert_eq!(
            whether_to_watch_again(&never, 7),
            WatchAgain::OnTheScheduleAlone(Until::TheServerAnswersAgain)
        );
    }

    #[test]
    fn test_every_reason_the_watch_can_end_with_has_an_answer_that_is_not_a_guess() {
        // The six reasons `imap.rs` breaks its loop with, each with the
        // answer it should get on a first failure. A reason added there
        // without an arm here is read as a lost connection and tried again,
        // which is the safe side, and this table is where the next person
        // adds it.
        let first_time = [
            ("the watch connection was lost", WatchAgain::AfterAWait),
            (
                "the mail server would not start watching, and gave no reason",
                WatchAgain::AfterAWait,
            ),
            (
                "the mail server would not start watching: NO",
                WatchAgain::AfterAWait,
            ),
            (
                "the watch connection failed: connection reset",
                WatchAgain::AfterAWait,
            ),
            ("the watch was stopped", WatchAgain::Never),
            ("nobody was listening", WatchAgain::Never),
        ];
        for (reason, expected) in first_time {
            assert_eq!(
                whether_to_watch_again(&WhyTheWatchEnded::Ended(reason.to_string()), 1),
                expected,
                "{reason}"
            );
        }
    }

    // ── What the status line says ────────────────────────────────────────

    fn running(watch: TheWatch) -> WhatIsRunning<'static> {
        WhatIsRunning {
            account: None,
            folder: "Inbox",
            watch,
            checking_every: minutes(5),
        }
    }

    #[test]
    fn test_the_status_line_says_watching_and_the_interval() {
        assert_eq!(
            what_the_status_line_says(&running(TheWatch::Watching)),
            "Watching Inbox for new mail. Checking every 5 minutes."
        );
    }

    #[test]
    fn test_the_status_line_says_only_the_interval_when_nothing_watches() {
        // POP, a server without IDLE, or a watch left to the schedule: the
        // line says what is happening and not what is not.
        assert_eq!(
            what_the_status_line_says(&running(TheWatch::NotWatching)),
            "Checking every 5 minutes."
        );
    }

    #[test]
    fn test_the_status_line_says_the_wait_and_the_interval() {
        assert_eq!(
            what_the_status_line_says(&running(TheWatch::Waiting(minutes(2)))),
            "Waiting 2 minutes to watch Inbox again. Checking every 5 minutes."
        );
        assert_eq!(
            what_the_status_line_says(&running(TheWatch::Waiting(Duration::from_secs(30)))),
            "Waiting 30 seconds to watch Inbox again. Checking every 5 minutes."
        );
    }

    #[test]
    fn test_the_account_is_named_first_when_more_than_one_is_enabled() {
        // Which account, what is happening, how often, in that order: the
        // name is what a person with two accounts needs before anything
        // else on the line makes sense.
        let mut state = running(TheWatch::Watching);
        state.account = Some("Work");

        assert_eq!(
            what_the_status_line_says(&state),
            "Work: Watching Inbox for new mail. Checking every 5 minutes."
        );
    }

    #[test]
    fn test_the_interval_is_said_as_one_minute_when_it_is_one() {
        let mut state = running(TheWatch::NotWatching);
        state.checking_every = minutes(1);

        assert_eq!(what_the_status_line_says(&state), "Checking every minute.");
    }

    #[test]
    fn test_the_status_line_uses_no_protocol_words() {
        for watch in [
            TheWatch::Watching,
            TheWatch::Waiting(minutes(1)),
            TheWatch::NotWatching,
        ] {
            let said = what_the_status_line_says(&running(watch));
            assert!(!said.is_empty(), "nothing said for {watch:?}");
            for word in ["IMAP", "IDLE", "poll", "socket", "connection"] {
                assert!(
                    !said.contains(word),
                    "the status line uses a word a person did not choose, {word}: {said}"
                );
            }
        }
    }

    #[test]
    fn test_a_check_of_several_accounts_says_how_many() {
        assert_eq!(what_a_check_of_them_all_says(1), "Checking for new mail...");
        assert_eq!(
            what_a_check_of_them_all_says(2),
            "Checking 2 accounts for new mail..."
        );
    }
}

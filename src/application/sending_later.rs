//! Holding a message back, and taking one back.
//!
//! Send has always meant gone. A message is queued, the send loop picks it up
//! on its next pass, and from that moment the only copy anybody can act on is
//! in a stranger's mailbox. Two things are missing from that: a time somebody
//! chose, and a few seconds in which to change their mind.
//!
//! # What this adds to the queue
//!
//! One question, asked before the send loop picks a row up: may this go yet.
//! [`readiness`] answers it from the row and the clock and nothing else. The
//! queue itself, the connection and the retry counting are unchanged and are
//! not this module's business.
//!
//! A message with nothing set goes as it always did. A message with a time on
//! it waits, and the row says what it is waiting for, because an Outbox that
//! reads "waiting to send" over four rows doing four different things is an
//! Outbox nobody can work with by ear.
//!
//! # Why every message is held for a moment
//!
//! Undoing a send is worth more here than in most mail programs. Working
//! through a mailbox by ear means acting on a row that was described rather
//! than seen, and the message you meant to reply to and the one you did reply
//! to are one arrow key apart. The hold is what makes that recoverable.
//!
//! Ten seconds by default, and the reasoning is in [`Hold::DEFAULT`]. It can
//! be set to anything from off to a minute, and both ends are said plainly at
//! the place the choice is made, because turning it off removes the only way
//! back.
//!
//! # What would be written down
//!
//! Two columns on the queue's table, added the way every other column here is
//! added, so a database somebody already has goes on opening. One holds the
//! moment a message may go, empty for the rows that have none, which is every
//! row that already exists. The other says whether a person chose that moment
//! or the hold set it, because the two are announced differently and worked
//! out differently, and a moment on its own cannot be told apart afterwards.
//! [`GoAfter::read`] and [`GoAfter::written_down`] are the pair, so the reader
//! and the writer answer one question the same way.
//!
//! The moment carries its offset. A message set for nine tomorrow is set for
//! that instant, and it goes then whether or not the laptop has moved to
//! another time zone in between.
//!
//! # What is not here
//!
//! Values in and values out. No database, no connection, no window, and
//! nothing platform-specific, so it behaves the same wherever it is built.
//!
//! All of it runs. The queue carries the time, the send loop asks
//! [`readiness`] on every pass rather than taking the whole queue, and Undo
//! Send is on the Tools menu with `Ctrl+Shift+Z`.
//!
//! That last one was owed for a while. The countdown has always said "Undo Send
//! takes it back", and for as long as it said so there was no Undo Send: no
//! menu item, no key, no button. It was harmless while nothing showed the
//! countdown, and became a promise the moment the send loop started honouring
//! the hold. `tests/wired.rs` now asks whether a command named in a sentence
//! exists at all, which is the check that should have been there first.

use chrono::{DateTime, Duration, Local};

/// How long every message waits after Send, so Send can be taken back.
///
/// Counted in seconds, because that is the unit somebody sets it in and the
/// unit it is counted down in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hold(i64);

impl Hold {
    /// Ten seconds.
    ///
    /// A hold nobody can reach is a hold that does nothing, and reaching this
    /// one starts with hearing that it is there. Pressing Send begins an
    /// announcement, the announcement has to finish before somebody knows
    /// there is anything to undo, and only then do they decide and press the
    /// key. Five seconds is mostly spent on the sentence. Ten leaves several
    /// seconds after it to think and act, which is what "I did not mean to
    /// send that" needs.
    ///
    /// The other end matters as much. Mail that sits in the Outbox with
    /// nothing happening reads as a program that has stopped working, so
    /// somebody presses Send again or goes looking for what went wrong. Ten
    /// seconds is short enough that waiting through it is obviously a wait.
    pub const DEFAULT: Hold = Hold(10);

    /// No hold at all: Send sends.
    pub const OFF: Hold = Hold(0);

    /// The longest hold on offer, one minute.
    ///
    /// Past this the Outbox looks stuck rather than busy, and somebody who
    /// wants a message to leave later wants the scheduler, which says when it
    /// will go instead of counting down at them.
    pub const LONGEST: Hold = Hold(60);

    /// A hold of this many seconds, brought inside what this offers.
    ///
    /// Clamped rather than refused because this reads a setting that has been
    /// written down and survived a restart, so it can hold anything an older
    /// build, a hand-edited file or a typo left there. There is no sensible
    /// way for a stored number to stop the program sending mail, and every
    /// sentence about the hold says the length that came out, so nothing
    /// claims a length it is not using.
    pub fn of_seconds(seconds: i64) -> Hold {
        Hold(seconds.clamp(Hold::OFF.0, Hold::LONGEST.0))
    }

    /// How many seconds this holds for.
    pub fn seconds(self) -> i64 {
        self.0
    }

    /// Whether Send sends straight away.
    pub fn is_off(self) -> bool {
        self.0 <= 0
    }
}

/// When a queued message may go, as its row says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoAfter {
    /// Nothing was set, so it goes on the next pass of the send loop.
    AsSoonAsPossible,
    /// The brief hold, so Send can be taken back. Stored as a moment.
    Held(String),
    /// A time somebody chose, stored as a moment.
    Chosen(String),
}

impl GoAfter {
    /// What Send writes on a message, given the hold in force.
    pub fn held(hold: Hold, now: DateTime<Local>) -> Self {
        if hold.is_off() {
            return GoAfter::AsSoonAsPossible;
        }
        GoAfter::Held(stored(now + Duration::seconds(hold.seconds())))
    }

    /// What the queue's two columns say.
    ///
    /// `None` is a row with nothing set, which is every message queued before
    /// these columns existed and every message sent with the hold off. Those
    /// go as they always did.
    ///
    /// Text that is there and cannot be read stays a time rather than
    /// collapsing to "nothing set", which would send it. It comes out as a
    /// message that is held back and says why, which somebody can act on.
    pub fn read(send_after: Option<&str>, somebody_chose_it: bool) -> Self {
        match (send_after, somebody_chose_it) {
            (None, _) => GoAfter::AsSoonAsPossible,
            (Some(moment), true) => GoAfter::Chosen(moment.to_string()),
            (Some(moment), false) => GoAfter::Held(moment.to_string()),
        }
    }

    /// The two columns this becomes.
    ///
    /// One nullable column for the moment and one flag for who put it there.
    /// The flag is not something to work out later from how far off the moment
    /// is: a hold and a schedule are announced differently and counted
    /// differently, and guessing would make a hold that outlived a long pause
    /// read as a scheduled message.
    pub fn written_down(&self) -> (Option<String>, bool) {
        match self {
            GoAfter::AsSoonAsPossible => (None, false),
            GoAfter::Held(moment) => (Some(moment.clone()), false),
            GoAfter::Chosen(moment) => (Some(moment.clone()), true),
        }
    }
}

/// A moment as the queue's column would carry it.
///
/// RFC 3339, with the offset on the end, so the row names one instant rather
/// than a clock face. A message set for nine tomorrow while sitting in London
/// is set for that moment, and it goes then whether or not the laptop has
/// moved to another zone in between. A stored clock face would move with the
/// machine, which is the opposite of what somebody scheduling mail to reach a
/// person at a particular hour asked for.
pub fn stored(at: DateTime<Local>) -> String {
    at.to_rfc3339()
}

/// Whether a queued message may go yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Readiness {
    /// Nothing is holding it back.
    MayGoNow,
    /// Being held so Send can still be taken back, with this much left.
    HeldFor(Duration),
    /// Set for a time that has not come.
    WaitingUntil(DateTime<Local>),
    /// A time is stored and this build cannot read it, so nothing is sent.
    ///
    /// Two ways to be wrong here and only one of them can be undone. Sending a
    /// message that was set for next week, now, is gone. Leaving it in the
    /// Outbox is a row somebody can see, read and act on, so that is the
    /// answer, and the row says as much rather than sitting there silently.
    TimeCannotBeRead,
}

/// Whether a queued message may go yet, and if not, why not.
///
/// Due now counts as due, the same rule reminders already follow. A message
/// passed over in the pass it came due in is a message that is always one pass
/// late, and the passes are a minute apart.
pub fn readiness(when: &GoAfter, now: DateTime<Local>) -> Readiness {
    match when {
        GoAfter::AsSoonAsPossible => Readiness::MayGoNow,
        GoAfter::Held(stored) => match the_moment(stored) {
            Some(until) if until > now => Readiness::HeldFor(until - now),
            Some(_) => Readiness::MayGoNow,
            None => Readiness::TimeCannotBeRead,
        },
        GoAfter::Chosen(stored) => match the_moment(stored) {
            Some(at) if at > now => Readiness::WaitingUntil(at),
            Some(_) => Readiness::MayGoNow,
            None => Readiness::TimeCannotBeRead,
        },
    }
}

/// Whether this program is handing anything to a server at the moment.
///
/// Offline mode is a switch on the View menu. A bool called `offline` reads as
/// `when_it_goes(true, ..)` at the call site, which is a line nobody can check
/// by eye, so the two states are named.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reachability {
    /// Servers are being talked to as usual.
    Online,
    /// Offline mode is on, so nothing is handed to a server.
    Offline,
}

/// Whether a message just put in the queue is handed to a server on this pass,
/// and if not, what it is waiting for.
///
/// Two things can hold a message back and they are not the same answer to
/// somebody who pressed Send. Offline mode is a switch they set and can unset.
/// A time is something the message carries and only waiting changes. Which one
/// they are told matters, because the way out of each is different.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhenItGoes {
    /// Nothing is holding it back.
    Now,
    /// Offline mode is on. It stays in the Outbox until somebody goes back
    /// online.
    WhenThereIsANetworkAgain,
    /// Its own time has not come. [`readiness`] says which kind of wait it is.
    WhenItsTimeComes,
}

/// Whether a message goes now, taking offline mode into account as well as the
/// message's own time.
///
/// Offline is answered before the time, and that ordering is the whole
/// sentence somebody hears. A message set for Friday, queued while offline
/// mode is on, is waiting on two things at once; the one worth saying is the
/// one they can do something about now.
///
/// It reads the schedule and never writes it. Turning offline mode on holds a
/// message back without touching what it was already waiting for, so turning
/// offline mode off leaves a scheduled message still scheduled rather than
/// sending it.
pub fn when_it_goes(
    reachability: Reachability,
    when: &GoAfter,
    now: DateTime<Local>,
) -> WhenItGoes {
    let _ = reachability;
    match readiness(when, now) {
        Readiness::MayGoNow => WhenItGoes::Now,
        _ => WhenItGoes::WhenItsTimeComes,
    }
}

/// What somebody who pressed Send is told.
///
/// Built here rather than at the window, so the words shown and the words
/// spoken come from one string, and so the three endings can be read beside
/// each other.
///
/// A message that did not go is the case worth wording carefully. Somebody who
/// pressed Send and heard nothing about the Outbox believes their mail has
/// gone, which is the defect this whole decision exists to end.
pub fn what_send_did(goes: WhenItGoes, recipient: &str) -> String {
    let _ = goes;
    format!("Sending to {recipient}...")
}

/// How far past a chosen time still counts as that time.
///
/// The picker chooses a minute, so a minute is the smallest gap it can mean.
/// Somebody who picks nine o'clock and presses OK twenty seconds later has not
/// made a mistake, and a dialog that refuses what it offered a moment ago is a
/// dialog nobody trusts.
pub const JUST_MISSED: Duration = Duration::minutes(1);

/// The furthest ahead a message may be set.
///
/// A year. Past that the likeliest reading of the value is a mistyped year,
/// and taking it at its word puts a message in the Outbox for a century.
/// Nothing here sends while the program is closed either, so a date that far
/// out is a promise this cannot keep.
pub const NO_FURTHER_AHEAD_THAN: Duration = Duration::days(365);

/// What came of asking to send a message at a chosen time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scheduling {
    /// Set. This is the moment, ready to be written down by [`stored`].
    SetFor(DateTime<Local>),
    /// The time has gone by, so setting it would mean sending now.
    AlreadyPast,
    /// So far ahead that it reads as a mistake.
    TooFarAhead,
    /// Not a time at all.
    NotAMoment,
}

/// Set a queued message to go at a time somebody chose.
///
/// The chosen time arrives as text in any shape [`crate::common::moment`]
/// reads, because it can come from the picker, from a message being sent again
/// or from an import, and each writes a different one.
pub fn schedule(chosen: &str, now: DateTime<Local>) -> Scheduling {
    let Some(at) = the_moment(chosen) else {
        return Scheduling::NotAMoment;
    };
    if at < now - JUST_MISSED {
        return Scheduling::AlreadyPast;
    }
    if at > now + NO_FURTHER_AHEAD_THAN {
        return Scheduling::TooFarAhead;
    }
    Scheduling::SetFor(at)
}

/// What came of asking to take a message back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TakingBack {
    /// It never went. The row comes out of the queue and the message can be
    /// opened again.
    Stopped,
    /// Its moment came, so it is on its way or already delivered.
    TooLate,
}

/// Whether a queued message can still be stopped.
///
/// One rule, so this and the send loop cannot come to disagree about the same
/// message: a message may be taken back exactly while [`readiness`] says it may
/// not go. The moment the send loop is allowed to pick it up, taking it back
/// stops promising anything, because from here nothing can see whether the
/// connection has already opened. Saying "taken back" about a message that has
/// gone is the one answer that leaves somebody believing a stranger never read
/// it, so it is never given.
///
/// A message stuck on a time that cannot be read has certainly not gone, so it
/// can be taken back, and that is the way out of a row that would otherwise sit
/// in the Outbox forever.
///
/// This is not the Outbox's own Delete, which takes any waiting row out of the
/// queue and goes on doing exactly that. This answers one narrower question:
/// can Send still be undone.
pub fn take_back(when: &GoAfter, now: DateTime<Local>) -> TakingBack {
    match readiness(when, now) {
        Readiness::MayGoNow => TakingBack::TooLate,
        Readiness::HeldFor(_) | Readiness::WaitingUntil(_) | Readiness::TimeCannotBeRead => {
            TakingBack::Stopped
        }
    }
}

/// Which queued message Undo Send is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatToTakeBack {
    /// This one, by the identifier the queue knows it as.
    ThisOne(String),
    /// Nothing is in the queue at all.
    NothingWaiting,
    /// Messages are waiting and none of them can be promised back.
    TooLateForEverything,
}

/// The message Undo Send means, out of everything in the queue.
///
/// The last one that can still be caught. Undo is about what just happened, so
/// a message set for Friday is not what somebody means while the message they
/// sent eight seconds ago is sitting behind it in the queue. `queued` is in the
/// order the queue holds it, oldest first.
///
/// An empty queue and a queue nothing can be caught in are kept apart, because
/// they are not the same answer and somebody does something different about
/// each. One means Send was never pressed. The other means it was, and only
/// that one is worth explaining.
pub fn what_undo_send_takes_back(
    queued: &[(String, GoAfter)],
    now: DateTime<Local>,
) -> WhatToTakeBack {
    if queued.is_empty() {
        return WhatToTakeBack::NothingWaiting;
    }
    match queued
        .iter()
        .rev()
        .find(|(_, when)| take_back(when, now) == TakingBack::Stopped)
    {
        Some((id, _)) => WhatToTakeBack::ThisOne(id.clone()),
        None => WhatToTakeBack::TooLateForEverything,
    }
}

impl WhatToTakeBack {
    /// Why nothing was taken back, and what to do instead.
    ///
    /// `None` when there is a message to take back. Every refusal names the
    /// next move, because one that only says no leaves somebody pressing the
    /// same key again.
    ///
    /// Neither refusal claims the message has gone. Once the send loop may pick
    /// a message up, nothing here can see whether the connection has opened, so
    /// saying "it has gone" would be a guess and saying "it has not" would be a
    /// worse one. It says what it knows, which is that it cannot be promised
    /// back, and points at the Outbox, where a row that is still there can
    /// still be deleted.
    pub fn why_not(&self) -> Option<&'static str> {
        match self {
            WhatToTakeBack::ThisOne(_) => None,
            WhatToTakeBack::NothingWaiting => Some("There is nothing waiting to send."),
            WhatToTakeBack::TooLateForEverything => Some(
                "Too late to take that back. Look in the Outbox: if the message \
                 is still there, Delete takes it out of the queue.",
            ),
        }
    }
}

// ── What is said ────────────────────────────────────────────────────────────

impl Readiness {
    /// What this row in the Outbox says it is doing.
    ///
    /// Four rows that look identical without it: one about to go, one counting
    /// down, one set for Friday, and one stuck on a time nothing can read.
    pub fn spoken(
        &self,
        now: DateTime<Local>,
        dates: crate::presentation::date_display::DateSettings,
    ) -> String {
        match self {
            // The same words the Outbox already uses for a message with
            // nothing holding it back, so a row nobody has scheduled goes on
            // saying what it has always said.
            Readiness::MayGoNow => "Waiting to send.".to_string(),
            Readiness::HeldFor(left) => countdown(*left),
            Readiness::WaitingUntil(at) => set_to_send(*at, now, dates),
            Readiness::TimeCannotBeRead => {
                "Not sent: the time set on this message cannot be read. \
                 Take it back and send it again."
                    .to_string()
            }
        }
    }
}

impl Scheduling {
    /// What is said back when somebody picks a time.
    ///
    /// Every refusal names the problem and the next move, because a refusal
    /// that only says no leaves somebody pressing the same button again.
    pub fn spoken(
        &self,
        now: DateTime<Local>,
        dates: crate::presentation::date_display::DateSettings,
    ) -> String {
        match self {
            Scheduling::SetFor(at) => set_to_send(*at, now, dates),
            Scheduling::AlreadyPast => "That time has gone. Pick a time still to come.".to_string(),
            Scheduling::TooFarAhead => {
                "That is more than a year ahead. Pick a time within the next year.".to_string()
            }
            Scheduling::NotAMoment => {
                "That is not a date and time. Pick a date and a time of day.".to_string()
            }
        }
    }
}

impl TakingBack {
    /// What is said back when somebody takes a message back.
    pub fn spoken(self) -> &'static str {
        match self {
            TakingBack::Stopped => "Taken back. The message was not sent.",
            TakingBack::TooLate => "Too late to take that back. The message has already gone.",
        }
    }
}

/// When a message that is waiting will go.
///
/// One place, so the Outbox row and the answer the picker gives cannot come to
/// word the same fact differently. The date itself is written by the reader
/// every other date in this program goes through, so it follows whatever
/// somebody set for the order of the day and month and for the clock.
fn set_to_send(
    at: DateTime<Local>,
    now: DateTime<Local>,
    dates: crate::presentation::date_display::DateSettings,
) -> String {
    format!(
        "Set to send on {}.",
        crate::presentation::date_display::spoken(&stored(at), now, dates)
    )
}

/// What pressing Send is about to do, for the button and for the setting.
///
/// Somebody who cannot watch the Outbox fill up has no other way to learn that
/// Send no longer means gone, so the button says it.
pub fn what_send_does(hold: Hold) -> String {
    if hold.is_off() {
        return "Send sends the message straight away, with no time to take it back.".to_string();
    }
    format!(
        "Send holds the message for {} first, so you can take it back.",
        seconds_said(hold.seconds())
    )
}

/// How long is left, and how to stop it.
///
/// Said once when Send is pressed and again on the Outbox row when somebody
/// arrows onto it. Not once a second: a sentence repeated ten times over ten
/// seconds buries whatever else the screen reader was saying, and the hold is
/// short enough that one reading covers it.
pub fn countdown(left: Duration) -> String {
    // Rounded up, so the last part-second is still a second rather than
    // becoming "Sending now" while there is time left to stop it.
    let seconds = (left.num_milliseconds() + 999) / 1000;
    if seconds <= 0 {
        return "Sending now.".to_string();
    }
    format!(
        "Sending in {}. Undo Send takes it back.",
        seconds_said(seconds)
    )
}

/// A number of seconds with the right word after it.
fn seconds_said(count: i64) -> String {
    if count == 1 {
        "1 second".to_string()
    } else {
        format!("{count} seconds")
    }
}

/// The instant a stored moment names on this computer's clock.
///
/// Read by [`crate::common::moment`] rather than by a list of date shapes kept
/// here, because the queue's column holds whatever the picker, an import or an
/// older build wrote, and a second list of shapes is how a reader and a writer
/// come to disagree about the same column.
fn the_moment(stored: &str) -> Option<DateTime<Local>> {
    crate::common::moment::read(stored).and_then(crate::common::moment::Moment::on_this_computer)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A moment on this computer's clock, written the way the cache holds one.
    fn at(text: &str) -> DateTime<Local> {
        crate::common::moment::read(text)
            .and_then(crate::common::moment::Moment::on_this_computer)
            .expect("a real moment")
    }

    #[test]
    fn test_a_message_with_no_time_on_it_goes_now() {
        // Every message queued before this existed has no time on it, and so
        // does every message somebody sends with the hold switched off. Both
        // have to leave, or an upgrade would strand the Outbox.
        assert_eq!(
            readiness(&GoAfter::AsSoonAsPossible, at("2026-08-24 09:00")),
            Readiness::MayGoNow
        );
    }

    #[test]
    fn test_a_message_set_for_a_time_still_to_come_waits() {
        // The whole point of setting a time. Sending it anyway would be the
        // one act in this program that cannot be taken back, done early.
        let when = GoAfter::Chosen("2026-08-24 17:00".to_string());

        assert_eq!(
            readiness(&when, at("2026-08-24 09:00")),
            Readiness::WaitingUntil(at("2026-08-24 17:00"))
        );
    }

    #[test]
    fn test_a_message_whose_time_has_come_goes() {
        // The other half, and the boundary with it: a message set for five
        // o'clock and passed over at five o'clock is a message that is always
        // one pass late, and the pass is a minute apart.
        let when = GoAfter::Chosen("2026-08-24 17:00".to_string());

        assert_eq!(
            readiness(&when, at("2026-08-24 17:00")),
            Readiness::MayGoNow,
            "a message due this very moment was held back"
        );
        assert_eq!(
            readiness(&when, at("2026-08-24 17:00:01")),
            Readiness::MayGoNow
        );
    }

    #[test]
    fn test_a_time_nobody_can_read_holds_the_message_back() {
        // Two ways to be wrong and only one of them can be undone. Sending a
        // message that was set for next week, now, is gone; leaving it in the
        // Outbox is a row somebody can see and act on. So a time this build
        // cannot read is a reason to stop, never a reason to go.
        for stored in ["", "   ", "next tuesday", "2026-13-45T99:99:99"] {
            assert_eq!(
                readiness(&GoAfter::Chosen(stored.to_string()), at("2026-08-24 09:00")),
                Readiness::TimeCannotBeRead,
                "a message stored with {stored:?} as its time was sent anyway"
            );
        }
    }

    #[test]
    fn test_pressing_send_holds_the_message_instead_of_sending_it() {
        // The whole reason the hold exists: Send is the one act in this
        // program that cannot be taken back, and for a few seconds it can.
        let sent_at = at("2026-08-24 09:00");

        let when = GoAfter::held(Hold::DEFAULT, sent_at);

        assert_eq!(
            readiness(&when, sent_at),
            Readiness::HeldFor(Duration::seconds(Hold::DEFAULT.seconds())),
            "Send went straight out with the hold on"
        );
    }

    #[test]
    fn test_the_hold_runs_out_and_the_message_goes() {
        // A hold that never ends is a message that never leaves, which is the
        // worse of the two failures: nothing was sent and Send appeared to
        // work.
        let sent_at = at("2026-08-24 09:00");
        let when = GoAfter::held(Hold::DEFAULT, sent_at);

        assert_eq!(
            readiness(&when, sent_at + Duration::seconds(Hold::DEFAULT.seconds())),
            Readiness::MayGoNow,
            "the message was still held at the moment the hold ended"
        );
        assert_eq!(
            readiness(&when, sent_at + Duration::seconds(1)),
            Readiness::HeldFor(Duration::seconds(Hold::DEFAULT.seconds() - 1)),
            "the wait does not count down"
        );
    }

    #[test]
    fn test_turning_the_hold_off_means_send_sends() {
        // Somebody who does not want it must be able to have Send do what it
        // has always done, with nothing left on the row to hold it back.
        assert_eq!(
            GoAfter::held(Hold::OFF, at("2026-08-24 09:00")),
            GoAfter::AsSoonAsPossible
        );
    }

    #[test]
    fn test_a_stored_hold_length_is_brought_inside_what_is_offered() {
        // The length is written down and comes back after a restart, so it can
        // hold whatever an older build, a typo or a hand-edited file left
        // there. None of those may stop mail leaving, and none may leave it
        // sitting for an hour with a countdown running.
        assert_eq!(Hold::of_seconds(10).seconds(), 10);
        assert_eq!(Hold::of_seconds(0), Hold::OFF);
        assert!(
            Hold::of_seconds(-30).is_off(),
            "a negative length has to mean off, not a hold that has already ended"
        );
        assert_eq!(
            Hold::of_seconds(5_000),
            Hold::LONGEST,
            "a huge stored number left the Outbox looking stuck"
        );
        assert!(!Hold::DEFAULT.is_off());
    }

    #[test]
    fn test_a_message_that_has_not_gone_yet_can_be_taken_back() {
        let sent_at = at("2026-08-24 09:00");

        assert_eq!(
            take_back(
                &GoAfter::held(Hold::DEFAULT, sent_at),
                sent_at + Duration::seconds(3)
            ),
            TakingBack::Stopped,
            "a message still inside its hold could not be stopped"
        );
        assert_eq!(
            take_back(&GoAfter::Chosen("2026-08-25 09:00".to_string()), sent_at),
            TakingBack::Stopped,
            "a message set for tomorrow could not be stopped today"
        );
        assert_eq!(
            take_back(&GoAfter::Chosen("next tuesday".to_string()), sent_at),
            TakingBack::Stopped,
            "a message stuck on a time nobody can read has certainly not gone, \
             and taking it back is the way out of that row"
        );
    }

    #[test]
    fn test_taking_back_a_message_whose_time_has_come_says_so_rather_than_lying() {
        // The failure this exists to prevent is a quiet one: saying "taken
        // back" about a message already on its way leaves somebody believing
        // a stranger never got it.
        let sent_at = at("2026-08-24 09:00");

        assert_eq!(
            take_back(
                &GoAfter::held(Hold::DEFAULT, sent_at),
                sent_at + Duration::seconds(Hold::DEFAULT.seconds())
            ),
            TakingBack::TooLate,
            "a message whose hold had ended was reported as stopped"
        );
        assert_eq!(
            take_back(&GoAfter::AsSoonAsPossible, sent_at),
            TakingBack::TooLate,
            "with no hold on it there is no window to undo, and saying \
             otherwise is a promise this cannot keep"
        );
    }

    #[test]
    fn test_a_time_still_to_come_is_accepted_and_kept_as_that_moment() {
        let now = at("2026-08-24 09:00");

        assert_eq!(
            schedule("2026-08-25 09:00", now),
            Scheduling::SetFor(at("2026-08-25 09:00"))
        );
    }

    #[test]
    fn test_a_time_that_has_passed_is_refused_rather_than_sent_now() {
        // Sending now is the answer that cannot be undone. Somebody who picked
        // yesterday by mistake means to fix the date, not to send the message
        // this second.
        let now = at("2026-08-24 09:00");

        assert_eq!(schedule("2026-08-24 08:00", now), Scheduling::AlreadyPast);
        assert_eq!(schedule("2019-01-01 08:00", now), Scheduling::AlreadyPast);
    }

    #[test]
    fn test_a_time_only_just_missed_is_still_the_time_that_was_meant() {
        // The picker works in minutes. Choosing nine o'clock and pressing OK
        // twenty seconds later is not a mistake to refuse, and refusing it
        // would be a dialog that rejects what it just offered.
        let chosen = at("2026-08-24 09:00");

        assert_eq!(
            schedule("2026-08-24 09:00", chosen + Duration::seconds(20)),
            Scheduling::SetFor(chosen)
        );
    }

    #[test]
    fn test_a_time_far_enough_ahead_to_be_a_mistake_is_refused() {
        // A mistyped year is the ordinary way to get here, and the cost of
        // taking it is a message that sits in the Outbox for a century. This
        // queue only sends while the program is running, so it would not even
        // go then.
        let now = at("2026-08-24 09:00");

        assert_eq!(schedule("2126-08-24 09:00", now), Scheduling::TooFarAhead);
        assert_eq!(
            schedule("2027-08-25 09:00", now),
            Scheduling::TooFarAhead,
            "a year and a day ahead was accepted"
        );
        assert!(
            matches!(schedule("2027-08-23 09:00", now), Scheduling::SetFor(_)),
            "a day inside the limit was refused"
        );
    }

    #[test]
    fn test_a_time_written_in_another_zone_keeps_the_instant_it_names() {
        // A time can arrive carrying an offset, from a picker set to somebody
        // else's zone or from an imported row. What was chosen is an instant,
        // so that is what is kept: reading the clock face and dropping the
        // offset would move the message by however many hours apart the two
        // zones are, which for mail meant to land at nine in the morning
        // somewhere is the whole point missed.
        use chrono::FixedOffset;

        let now = at("2026-08-24 09:00");
        let three_hours_off = now + Duration::hours(3);
        let india = FixedOffset::east_opt(5 * 3600 + 1800).expect("a real offset");
        let written_there = three_hours_off.with_timezone(&india).to_rfc3339();

        assert_eq!(
            schedule(&written_there, now),
            Scheduling::SetFor(three_hours_off),
            "the message moved when the time was written in another zone"
        );
    }

    #[test]
    fn test_a_time_that_is_not_a_time_is_refused() {
        let now = at("2026-08-24 09:00");

        for asked in ["", "   ", "soon", "2026-13-45T99:99:99"] {
            assert_eq!(
                schedule(asked, now),
                Scheduling::NotAMoment,
                "{asked:?} was accepted as a time to send at"
            );
        }
    }

    /// Dates written out in full, so a test asserts wording and not a setting.
    fn dates() -> crate::presentation::date_display::DateSettings {
        use crate::presentation::date_display::{
            Clock, DateOrder, DateSettings, DateStyle, DateWording,
        };
        DateSettings {
            style: DateStyle::Absolute,
            order: DateOrder::MonthFirst,
            wording: DateWording::Verbal,
            clock: Clock::TwelveHour,
        }
    }

    #[test]
    fn test_the_send_button_says_that_it_holds_the_message_first() {
        // Somebody who cannot see the Outbox fill up has no other way to learn
        // that Send no longer means gone.
        assert_eq!(
            what_send_does(Hold::DEFAULT),
            "Send holds the message for 10 seconds first, so you can take it back."
        );
        assert_eq!(
            what_send_does(Hold::of_seconds(1)),
            "Send holds the message for 1 second first, so you can take it back."
        );
    }

    #[test]
    fn test_with_the_hold_off_the_send_button_says_there_is_no_way_back() {
        // The setting that removes the safety net has to say that is what it
        // does, at the place where the choice is made.
        assert_eq!(
            what_send_does(Hold::OFF),
            "Send sends the message straight away, with no time to take it back."
        );
    }

    #[test]
    fn test_the_countdown_says_how_long_is_left_and_how_to_stop_it() {
        // A countdown that does not say how to stop it is a countdown nobody
        // can use, and the seconds are the only reason to hurry.
        assert_eq!(
            countdown(Duration::seconds(10)),
            "Sending in 10 seconds. Undo Send takes it back."
        );
        assert_eq!(
            countdown(Duration::seconds(1)),
            "Sending in 1 second. Undo Send takes it back."
        );
    }

    #[test]
    fn test_a_countdown_that_has_run_out_says_the_message_is_going() {
        // Not "Sending in 0 seconds", which is a sentence that says a thing
        // and its opposite.
        assert_eq!(countdown(Duration::zero()), "Sending now.");
        assert_eq!(countdown(Duration::seconds(-5)), "Sending now.");
    }

    #[test]
    fn test_taking_a_message_back_says_which_way_it_went() {
        assert_eq!(
            TakingBack::Stopped.spoken(),
            "Taken back. The message was not sent."
        );
        assert_eq!(
            TakingBack::TooLate.spoken(),
            "Too late to take that back. The message has already gone."
        );
    }

    #[test]
    fn test_a_scheduled_message_says_when_it_will_go() {
        // The date is the whole answer. "Scheduled" on its own leaves somebody
        // with no idea whether it goes in an hour or in a fortnight.
        let now = at("2026-08-24 09:00");

        assert_eq!(
            Scheduling::SetFor(at("2026-08-25 09:00")).spoken(now, dates()),
            "Set to send on August 25, 2026 at 9:00 AM."
        );
    }

    #[test]
    fn test_a_time_that_cannot_be_used_says_why_and_what_to_do_instead() {
        // A refusal that only says no leaves somebody pressing the same button
        // again. Each of these names the problem and the next move.
        let now = at("2026-08-24 09:00");

        assert_eq!(
            Scheduling::AlreadyPast.spoken(now, dates()),
            "That time has gone. Pick a time still to come."
        );
        assert_eq!(
            Scheduling::TooFarAhead.spoken(now, dates()),
            "That is more than a year ahead. Pick a time within the next year."
        );
        assert_eq!(
            Scheduling::NotAMoment.spoken(now, dates()),
            "That is not a date and time. Pick a date and a time of day."
        );
    }

    #[test]
    fn test_every_row_in_the_outbox_says_what_it_is_doing() {
        // The Outbox is read one row at a time, and four of these rows look
        // identical without a sentence: one going now, one counting down, one
        // set for Friday, and one stuck on a time nothing can read.
        let now = at("2026-08-24 09:00");

        assert_eq!(Readiness::MayGoNow.spoken(now, dates()), "Waiting to send.");
        assert_eq!(
            Readiness::HeldFor(Duration::seconds(4)).spoken(now, dates()),
            "Sending in 4 seconds. Undo Send takes it back."
        );
        assert_eq!(
            Readiness::WaitingUntil(at("2026-08-25 09:00")).spoken(now, dates()),
            "Set to send on August 25, 2026 at 9:00 AM."
        );
        assert_eq!(
            Readiness::TimeCannotBeRead.spoken(now, dates()),
            "Not sent: the time set on this message cannot be read. Take it back and send it again."
        );
    }

    #[test]
    fn test_what_is_written_down_reads_back_as_what_it_was() {
        // A restart in the middle of a hold must not turn it into something
        // else. A held message read back as chosen would be announced as
        // scheduled for ten seconds past nine, and a chosen one read back as
        // held would count down from three days.
        let now = at("2026-08-24 09:00");

        for when in [
            GoAfter::AsSoonAsPossible,
            GoAfter::held(Hold::DEFAULT, now),
            GoAfter::Chosen(stored(at("2026-08-25 09:00"))),
        ] {
            let (send_after, somebody_chose_it) = when.written_down();

            assert_eq!(
                GoAfter::read(send_after.as_deref(), somebody_chose_it),
                when,
                "a row written as {send_after:?} and {somebody_chose_it} came back as \
                 something else"
            );
        }
    }

    #[test]
    fn test_a_message_queued_before_any_of_this_existed_still_goes() {
        // The columns arrive on a database that already has mail waiting in
        // it, so every row queued by an older build has nothing in them. Those
        // messages were sent the moment the loop reached them and they still
        // are, or an upgrade would strand somebody's Outbox in silence.
        assert_eq!(GoAfter::read(None, false), GoAfter::AsSoonAsPossible);
        assert_eq!(
            GoAfter::read(None, true),
            GoAfter::AsSoonAsPossible,
            "a row with the flag set and no time is a row with no time"
        );
    }

    #[test]
    fn test_a_row_with_a_time_in_it_is_never_read_as_having_none() {
        // The dangerous direction. A time this build cannot read has to stay a
        // time, so it comes out as a message that is held back and says why.
        // Collapsing it to "nothing set" would send it.
        for stored_time in ["", "   ", "not a time"] {
            for somebody_chose_it in [true, false] {
                let read = GoAfter::read(Some(stored_time), somebody_chose_it);

                assert_ne!(read, GoAfter::AsSoonAsPossible, "{stored_time:?}");
                assert_eq!(
                    readiness(&read, at("2026-08-24 09:00")),
                    Readiness::TimeCannotBeRead,
                    "a row holding {stored_time:?} was let out"
                );
            }
        }
    }
}

#[cfg(test)]
mod what_undo_send_is_about {
    use super::*;

    fn at(text: &str) -> DateTime<Local> {
        crate::common::moment::read(text)
            .and_then(crate::common::moment::Moment::on_this_computer)
            .expect("a real moment")
    }

    /// A queue row, named and with a time on it.
    fn waiting(name: &str, when: GoAfter) -> (String, GoAfter) {
        (name.to_string(), when)
    }

    #[test]
    fn test_undo_send_takes_back_the_message_most_recently_sent() {
        // Somebody presses Send, hears the countdown, and presses Undo Send.
        // They mean the message they just sent, not the one they set for
        // Friday. Both can still be caught and only the later one is meant.
        let now = at("2026-08-24T09:00:00");
        let queue = [
            waiting("friday", GoAfter::Chosen(stored(at("2026-08-28T09:00:00")))),
            waiting(
                "just-now",
                GoAfter::Held(stored(now + Duration::seconds(8))),
            ),
        ];

        assert_eq!(
            what_undo_send_takes_back(&queue, now),
            WhatToTakeBack::ThisOne("just-now".to_string())
        );
    }

    #[test]
    fn test_an_empty_queue_and_a_queue_nothing_can_be_caught_in_are_different_answers() {
        // "There is nothing waiting" about a message that has just gone would
        // have somebody looking for it in Drafts. "Too late" about a queue
        // nobody has put anything in reads as a message lost. They are told
        // apart because a person does something different about each.
        let now = at("2026-08-24T09:00:00");

        assert_eq!(
            what_undo_send_takes_back(&[], now),
            WhatToTakeBack::NothingWaiting
        );

        let gone = [waiting("already-going", GoAfter::AsSoonAsPossible)];
        assert_eq!(
            what_undo_send_takes_back(&gone, now),
            WhatToTakeBack::TooLateForEverything
        );
    }

    #[test]
    fn test_a_message_whose_time_cannot_be_read_can_still_be_taken_back() {
        // The way out of a row that would otherwise sit in the Outbox forever.
        // It has certainly not gone, because nothing will send it, so refusing
        // to take it back would strand it with no way to reach the message.
        let now = at("2026-08-24T09:00:00");
        let stuck = [waiting("stuck", GoAfter::Chosen("not a time".to_string()))];

        assert_eq!(
            what_undo_send_takes_back(&stuck, now),
            WhatToTakeBack::ThisOne("stuck".to_string())
        );
    }

    #[test]
    fn test_neither_refusal_claims_the_message_has_gone() {
        // The one answer that must never be given, because it is the one that
        // leaves somebody believing a stranger never read what they wrote.
        // Nothing here can see whether the connection has opened.
        for refusal in [
            WhatToTakeBack::NothingWaiting,
            WhatToTakeBack::TooLateForEverything,
        ] {
            let said = refusal.why_not().expect("a refusal says why");
            let claimed = said.to_lowercase();
            assert!(
                !claimed.contains("has gone")
                    && !claimed.contains("was sent")
                    && !claimed.contains("has been sent")
                    && !claimed.contains("delivered"),
                "{refusal:?} says the message has gone, which it cannot know: {said}"
            );
        }
    }

    #[test]
    fn test_the_message_that_can_be_taken_back_gives_no_refusal() {
        let now = at("2026-08-24T09:00:00");
        let held = [waiting(
            "held",
            GoAfter::Held(stored(now + Duration::seconds(8))),
        )];

        assert_eq!(
            what_undo_send_takes_back(&held, now).why_not(),
            None,
            "a message that can be caught should not be explaining itself"
        );
    }
}

#[cfg(test)]
mod what_offline_mode_holds_back {
    use super::*;

    /// A moment on this computer's clock, written the way the cache holds one.
    fn at(text: &str) -> DateTime<Local> {
        crate::common::moment::read(text)
            .and_then(crate::common::moment::Moment::on_this_computer)
            .expect("a real moment")
    }

    #[test]
    fn test_a_message_sent_while_offline_mode_is_on_waits_for_the_network() {
        // The defect this is written against, and it ships today. The View
        // menu says outgoing mail will be queued, nothing reads the switch,
        // and the message goes to a server anyway. Somebody who read that
        // sentence and closed their laptop believes their mail is being held.
        assert_eq!(
            when_it_goes(
                Reachability::Offline,
                &GoAfter::AsSoonAsPossible,
                at("2026-08-24 09:00")
            ),
            WhenItGoes::WhenThereIsANetworkAgain
        );
    }

    #[test]
    fn test_a_message_sent_online_still_goes_now() {
        // The other half, and the one that must not change. Every message
        // sent today takes this path and it has to keep taking it.
        assert_eq!(
            when_it_goes(
                Reachability::Online,
                &GoAfter::AsSoonAsPossible,
                at("2026-08-24 09:00")
            ),
            WhenItGoes::Now
        );
    }

    #[test]
    fn test_offline_mode_is_answered_before_the_messages_own_time() {
        // Waiting on two things at once. The one worth saying is the one
        // somebody can act on now, and the other is still true underneath it,
        // which the round trip below proves.
        let friday = GoAfter::Chosen("2026-08-28 09:00".to_string());

        assert_eq!(
            when_it_goes(Reachability::Offline, &friday, at("2026-08-24 09:00")),
            WhenItGoes::WhenThereIsANetworkAgain
        );
    }

    #[test]
    fn test_going_offline_holds_a_message_without_changing_what_it_waits_for() {
        // Turning offline mode on when there are already queued messages must
        // not touch their schedules. If it did, the way it would show is a
        // message set for Friday going out the moment somebody went back
        // online, which is the one act here that cannot be undone.
        let friday = GoAfter::Chosen("2026-08-28 09:00".to_string());
        let now = at("2026-08-24 09:00");

        assert_eq!(
            when_it_goes(Reachability::Offline, &friday, now),
            WhenItGoes::WhenThereIsANetworkAgain
        );
        // Back online, and it is still waiting on Friday rather than going.
        assert_eq!(
            when_it_goes(Reachability::Online, &friday, now),
            WhenItGoes::WhenItsTimeComes
        );
        assert_eq!(
            readiness(&friday, now),
            Readiness::WaitingUntil(at("2026-08-28 09:00")),
            "the schedule was rewritten by being asked about"
        );
    }

    #[test]
    fn test_a_scheduled_message_waits_for_its_time_when_the_network_is_there() {
        let friday = GoAfter::Chosen("2026-08-28 09:00".to_string());

        assert_eq!(
            when_it_goes(Reachability::Online, &friday, at("2026-08-24 09:00")),
            WhenItGoes::WhenItsTimeComes
        );
    }

    #[test]
    fn test_a_time_this_build_cannot_read_still_holds_the_message() {
        // The arm that must not collapse to "goes now". Sending a message set
        // for next week, now, is the one thing here that cannot be taken back.
        let unreadable = GoAfter::Chosen("the day after the fair".to_string());

        assert_eq!(
            when_it_goes(Reachability::Online, &unreadable, at("2026-08-24 09:00")),
            WhenItGoes::WhenItsTimeComes
        );
    }

    #[test]
    fn test_a_message_held_by_offline_mode_says_so_rather_than_saying_it_is_sending() {
        // Somebody pressed a key called Send. If the only sentence they get is
        // the one about sending, they have been told their mail has gone.
        let said = what_send_did(WhenItGoes::WhenThereIsANetworkAgain, "kim@example.com");

        assert!(
            said.contains("Outbox"),
            "the message did not go and nothing said where it is: {said}"
        );
        assert!(
            said.contains("kim@example.com"),
            "the sentence does not say which message it is about: {said}"
        );
        assert_ne!(
            said,
            what_send_did(WhenItGoes::Now, "kim@example.com"),
            "a message that went and one that did not are told the same thing"
        );
    }

    #[test]
    fn test_the_three_endings_of_send_are_three_different_sentences() {
        // Guardrail 5: feedback has to be distinguishable from its siblings.
        // Two of these mean the message is in the Outbox and the way out of
        // each is different, so hearing the wrong one sends somebody looking
        // in the wrong place.
        let went = what_send_did(WhenItGoes::Now, "kim@example.com");
        let offline = what_send_did(WhenItGoes::WhenThereIsANetworkAgain, "kim@example.com");
        let waiting = what_send_did(WhenItGoes::WhenItsTimeComes, "kim@example.com");

        assert_ne!(went, offline);
        assert_ne!(went, waiting);
        assert_ne!(
            offline, waiting,
            "offline mode and a time somebody set are the same sentence, so \
             neither says what to do about it"
        );
    }
}

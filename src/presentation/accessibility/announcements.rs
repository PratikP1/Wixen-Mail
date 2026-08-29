//! Announcement queue for screen reader messages.
//!
//! A mail client can generate speech faster than anyone can listen to it. A
//! folder sync alone produces a status line per step, and a client that speaks
//! every one of them buries the user until they turn announcements off and miss
//! what mattered. So this queue does four things beyond ordering by priority:
//! it drops repeats, lets a progress counter supersede its own earlier steps,
//! caps how much can be waiting, and caps how much is spoken per second.
//!
//! Anything it discards is counted and reported. Silently dropping speech is a
//! lie about what happened.

use crate::common::{Error, Result};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Most announcements that may be waiting at once. Beyond this the queue is
/// already further behind than anyone will listen to.
const CAPACITY: usize = 32;

/// Most announcements spoken per `WINDOW`.
const MAX_PER_WINDOW: usize = 4;

/// The period the rate limit applies over.
const WINDOW: Duration = Duration::from_secs(1);

/// Priority levels for announcements, lowest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Normal,
    High,
    Urgent,
}

/// What an announcement carries.
///
/// `Content` is message text read aloud, which is the only kind mute silences.
/// Someone presenting their screen needs to stop their mail being read to the
/// room without also losing "connection lost".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Interface,
    Content,
}

/// Whether the words of an announcement may be written to the log.
///
/// Most may, and the log is the better for it: a report that the application
/// said nothing can only be checked against a record of what was released to
/// be spoken. An announcement carrying something a person has typed may not.
/// What somebody is part way through typing into a To line is a person's name,
/// and the log is a file people are asked to attach to bug reports.
///
/// Kept apart from [`Kind`] on purpose. That decides what mute silences, which
/// is about a room with other people in it; this is about a file. Welding them
/// together would mean choosing between a name in a log and a name nobody
/// hears.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InTheLog {
    /// Written down as it was spoken.
    TheWords,
    /// Written down as a length only, so silence can still be told from speech.
    HowManyCharacters,
}

/// One announcement waiting to be spoken.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Announcement {
    pub text: String,
    pub priority: Priority,
    pub kind: Kind,
    /// Announcements sharing a topic supersede one another, so a counter that
    /// climbs from 1 to 500 is spoken once at its final value.
    pub topic: Option<String>,
    /// Whether the words may be written down. See [`InTheLog`].
    pub in_the_log: InTheLog,
}

impl Announcement {
    /// An announcement about the application itself. Never muted.
    pub fn interface(text: impl Into<String>, priority: Priority) -> Self {
        Self {
            text: text.into(),
            priority,
            kind: Kind::Interface,
            topic: None,
            in_the_log: InTheLog::TheWords,
        }
    }

    /// Message text read aloud. Silenced by mute.
    ///
    /// Never written to the log in full: a body read aloud must not end up in
    /// a file on disk.
    pub fn content(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            priority: Priority::Normal,
            kind: Kind::Content,
            topic: None,
            in_the_log: InTheLog::HowManyCharacters,
        }
    }

    /// Group this with others that supersede one another.
    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// Say this without writing the words down. See [`InTheLog`].
    pub fn not_in_the_log(mut self) -> Self {
        self.in_the_log = InTheLog::HowManyCharacters;
        self
    }
}

#[derive(Debug, Clone)]
struct Pending {
    announcement: Announcement,
    sequence: u64,
}

#[derive(Debug, Default)]
struct QueueState {
    pending: Vec<Pending>,
    sequence: u64,
    /// Announcements dropped for capacity since the last drain reported them.
    skipped: usize,
    /// When each recent announcement was spoken, for the rate limit.
    spoken_at: Vec<Instant>,
}

/// An announcement released for speaking, with what the screen reader needs to
/// decide how hard to hold on to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spoken {
    pub text: String,
    pub priority: Priority,
    pub kind: Kind,
    pub topic: Option<String>,
    /// Whether the words may be written down. See [`InTheLog`].
    pub in_the_log: InTheLog,
}

/// Queues and paces screen reader announcements.
pub struct AnnouncementQueue {
    state: Mutex<QueueState>,
    muted: AtomicBool,
}

impl AnnouncementQueue {
    /// Create a new announcement queue.
    pub fn new() -> Result<Self> {
        Ok(Self::default())
    }

    /// Stop or resume reading message content aloud. Interface announcements
    /// are unaffected.
    pub fn set_muted(&self, muted: bool) {
        self.muted.store(muted, Ordering::Relaxed);
    }

    /// Whether message content is currently silenced.
    pub fn is_muted(&self) -> bool {
        self.muted.load(Ordering::Relaxed)
    }

    /// Queue an announcement, applying mute, superseding, deduplication, and
    /// the capacity bound.
    pub fn push(&self, announcement: Announcement) -> Result<()> {
        if announcement.kind == Kind::Content && self.is_muted() {
            return Ok(());
        }

        let mut state = self.lock()?;

        // Superseding: a newer entry on the same topic replaces the older one
        // in place, keeping its queue position so nothing starves.
        if let Some(topic) = announcement.topic.as_deref()
            && let Some(existing) = state
                .pending
                .iter_mut()
                .find(|p| p.announcement.topic.as_deref() == Some(topic))
        {
            existing.announcement = announcement;
            return Ok(());
        }

        // A repeat says nothing new.
        if state
            .pending
            .iter()
            .any(|p| p.announcement.text == announcement.text)
        {
            return Ok(());
        }

        if state.pending.len() >= CAPACITY {
            // Evict the least important thing waiting, which may be this new
            // arrival. Either way it is counted, never hidden.
            let worst = state
                .pending
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    a.announcement
                        .priority
                        .cmp(&b.announcement.priority)
                        .then(b.sequence.cmp(&a.sequence))
                })
                .map(|(index, p)| (index, p.announcement.priority));

            match worst {
                Some((index, worst_priority)) if worst_priority < announcement.priority => {
                    state.pending.remove(index);
                }
                Some(_) => {
                    state.skipped += 1;
                    return Ok(());
                }
                None => {}
            }
            state.skipped += 1;
        }

        let sequence = state.sequence;
        state.sequence += 1;
        state.pending.push(Pending {
            announcement,
            sequence,
        });
        Ok(())
    }

    /// Take the announcements that may be spoken now, most important first.
    ///
    /// Returns at most `MAX_PER_WINDOW` per `WINDOW`, except for `Urgent`,
    /// which is never held back. Anything dropped for capacity is reported as
    /// a final line rather than vanishing.
    pub fn drain(&self, now: Instant) -> Result<Vec<Spoken>> {
        let mut state = self.lock()?;

        state
            .spoken_at
            .retain(|spoken| now.duration_since(*spoken) < WINDOW);
        let mut budget = MAX_PER_WINDOW.saturating_sub(state.spoken_at.len());

        // Most important first; oldest first within a priority.
        state.pending.sort_by(|a, b| {
            b.announcement
                .priority
                .cmp(&a.announcement.priority)
                .then(a.sequence.cmp(&b.sequence))
        });

        let mut spoken = Vec::new();
        let mut held = Vec::new();
        for entry in std::mem::take(&mut state.pending) {
            let urgent = entry.announcement.priority == Priority::Urgent;
            if urgent || budget > 0 {
                if !urgent {
                    budget -= 1;
                }
                state.spoken_at.push(now);
                spoken.push(Spoken {
                    text: entry.announcement.text,
                    priority: entry.announcement.priority,
                    kind: entry.announcement.kind,
                    topic: entry.announcement.topic,
                    in_the_log: entry.announcement.in_the_log,
                });
            } else {
                held.push(entry);
            }
        }
        state.pending = held;

        if state.skipped > 0 && !spoken.is_empty() {
            let skipped = std::mem::take(&mut state.skipped);
            // Heard rather than read, so it has to be a sentence somebody says.
            let word = if skipped == 1 {
                "announcement"
            } else {
                "announcements"
            };
            spoken.push(Spoken {
                text: format!("{} {} skipped", skipped, word),
                priority: Priority::Normal,
                kind: Kind::Interface,
                topic: None,
                in_the_log: InTheLog::TheWords,
            });
        }

        Ok(spoken)
    }

    /// How many announcements are waiting.
    pub fn pending_len(&self) -> Result<usize> {
        Ok(self.lock()?.pending.len())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, QueueState>> {
        self.state
            .lock()
            .map_err(|_| Error::Other("Announcement queue lock poisoned".to_string()))
    }
}

impl Default for AnnouncementQueue {
    fn default() -> Self {
        Self {
            state: Mutex::new(QueueState::default()),
            muted: AtomicBool::new(false),
        }
    }
}

#[cfg(test)]
mod what_may_be_written_down {
    use super::*;

    #[test]
    fn test_an_announcement_may_be_written_to_the_log_unless_it_says_otherwise() {
        // The log is how a report of silence gets checked against whether
        // anything was ever released to be spoken, so the words are in it by
        // default and only the ones that must not be are kept out.
        let ordinary = Announcement::interface("Draft saved", Priority::Normal);

        assert_eq!(ordinary.in_the_log, InTheLog::TheWords);
    }

    #[test]
    fn test_an_announcement_carrying_what_somebody_typed_is_kept_out_of_the_log() {
        // Part of a name typed into a To line is a person's name, and the log
        // is a file people are asked to attach to bug reports.
        let private =
            Announcement::interface("Nobody found for \"smith\"", Priority::Low).not_in_the_log();

        assert_eq!(private.in_the_log, InTheLog::HowManyCharacters);
    }

    #[test]
    fn test_what_is_kept_out_of_the_log_still_reaches_the_screen_reader() {
        // The whole point. Keeping words out of a file must not keep them out
        // of somebody's ears.
        let queue = AnnouncementQueue::new().expect("a queue");
        queue
            .push(
                Announcement::interface("Nobody found for \"smith\"", Priority::Normal)
                    .not_in_the_log(),
            )
            .expect("the announcement to queue");

        let spoken = queue.drain(Instant::now()).expect("something to say");

        assert_eq!(spoken.len(), 1);
        assert_eq!(spoken[0].text, "Nobody found for \"smith\"");
        assert_eq!(spoken[0].in_the_log, InTheLog::HowManyCharacters);
    }

    #[test]
    fn test_being_kept_out_of_the_log_is_not_being_muted() {
        // Mute is for message text being read aloud in a room. This is about a
        // file, and the two must not be welded together: somebody who has not
        // muted anything still hears this.
        let queue = AnnouncementQueue::new().expect("a queue");
        queue.set_muted(true);
        queue
            .push(Announcement::interface("Nobody found", Priority::Normal).not_in_the_log())
            .expect("the announcement to queue");

        assert_eq!(
            queue.drain(Instant::now()).expect("something to say").len(),
            1
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixed starting point so tests never depend on wall-clock time.
    fn t0() -> Instant {
        Instant::now()
    }

    /// Fill the queue right up to capacity with distinct lines, so that the
    /// next push has to decide what to give up. The texts differ because a
    /// repeat is dropped before capacity is ever considered, and they carry no
    /// topic so none of them supersedes another.
    fn fill_to_capacity(queue: &AnnouncementQueue, priority: Priority) {
        for n in 0..CAPACITY {
            queue
                .push(Announcement::interface(format!("chatter {}", n), priority))
                .unwrap();
        }
    }

    /// Just the text, for the tests that only care what was said.
    fn drained(queue: &AnnouncementQueue, now: Instant) -> Vec<String> {
        queue
            .drain(now)
            .unwrap()
            .into_iter()
            .map(|spoken| spoken.text)
            .collect()
    }

    #[test]
    fn test_priority_order() {
        let q = AnnouncementQueue::new().unwrap();
        q.push(Announcement::interface("normal", Priority::Normal))
            .unwrap();
        q.push(Announcement::interface("urgent", Priority::Urgent))
            .unwrap();
        assert_eq!(drained(&q, t0()), vec!["urgent", "normal"]);
    }

    #[test]
    fn test_fifo_within_priority() {
        let q = AnnouncementQueue::new().unwrap();
        q.push(Announcement::interface("first", Priority::Normal))
            .unwrap();
        q.push(Announcement::interface("second", Priority::Normal))
            .unwrap();
        assert_eq!(drained(&q, t0()), vec!["first", "second"]);
    }

    #[test]
    fn test_identical_text_is_not_queued_twice() {
        let q = AnnouncementQueue::new().unwrap();
        q.push(Announcement::interface("Connected", Priority::Normal))
            .unwrap();
        q.push(Announcement::interface("Connected", Priority::Normal))
            .unwrap();
        assert_eq!(drained(&q, t0()), vec!["Connected"]);
    }

    #[test]
    fn test_same_topic_supersedes_rather_than_queues() {
        let q = AnnouncementQueue::new().unwrap();
        for n in 1..=5 {
            q.push(
                Announcement::interface(format!("{} messages loaded", n), Priority::Low)
                    .with_topic("message-count"),
            )
            .unwrap();
        }
        // The user wants the final count, not all five steps.
        assert_eq!(drained(&q, t0()), vec!["5 messages loaded"]);
    }

    #[test]
    fn test_different_topics_both_survive() {
        let q = AnnouncementQueue::new().unwrap();
        q.push(
            Announcement::interface("3 messages loaded", Priority::Normal).with_topic("messages"),
        )
        .unwrap();
        q.push(Announcement::interface("2 folders loaded", Priority::Normal).with_topic("folders"))
            .unwrap();
        assert_eq!(q.drain(t0()).unwrap().len(), 2);
    }

    #[test]
    fn test_queue_is_bounded() {
        let q = AnnouncementQueue::new().unwrap();
        for n in 0..(CAPACITY * 4) {
            q.push(Announcement::interface(
                format!("item {}", n),
                Priority::Normal,
            ))
            .unwrap();
        }
        assert_eq!(q.pending_len().unwrap(), CAPACITY);
    }

    #[test]
    fn test_overflow_drops_low_priority_before_urgent() {
        let q = AnnouncementQueue::new().unwrap();
        q.push(Announcement::interface("critical", Priority::Urgent))
            .unwrap();
        for n in 0..(CAPACITY * 2) {
            q.push(Announcement::interface(
                format!("chatter {}", n),
                Priority::Low,
            ))
            .unwrap();
        }
        let spoken = drained(&q, t0());
        assert_eq!(spoken.first().map(String::as_str), Some("critical"));
    }

    #[test]
    fn test_dropped_announcements_are_reported_not_hidden() {
        let q = AnnouncementQueue::new().unwrap();
        for n in 0..(CAPACITY * 2) {
            q.push(Announcement::interface(
                format!("item {}", n),
                Priority::Normal,
            ))
            .unwrap();
        }
        let spoken = drained(&q, t0());
        assert!(
            spoken.iter().any(|s| s.contains("skipped")),
            "a silent drop is a lie about what happened: {:?}",
            spoken
        );
    }

    #[test]
    fn test_a_full_queue_makes_room_for_an_urgent_line_rather_than_turning_it_away() {
        let q = AnnouncementQueue::new().unwrap();
        fill_to_capacity(&q, Priority::Low);
        q.push(Announcement::interface("connection lost", Priority::Urgent))
            .unwrap();
        // Mid-sync chatter must not be the reason nobody hears the connection
        // drop. Urgent skips the rate limit, so it comes out of the first drain.
        assert_eq!(
            drained(&q, t0()).first().map(String::as_str),
            Some("connection lost")
        );
    }

    #[test]
    fn test_a_full_queue_turns_away_a_newcomer_no_more_important_than_what_is_already_waiting() {
        let q = AnnouncementQueue::new().unwrap();
        fill_to_capacity(&q, Priority::Urgent);
        q.push(Announcement::interface("late arrival", Priority::Urgent))
            .unwrap();
        let spoken = drained(&q, t0());
        assert!(
            !spoken.iter().any(|s| s == "late arrival"),
            "nothing already owed to the user should be displaced for this: {:?}",
            spoken
        );
        assert!(
            spoken.iter().any(|s| s == "chatter 31"),
            "the newest of the equally important is what would have gone: {:?}",
            spoken
        );
    }

    #[test]
    fn test_when_room_must_be_made_the_newest_of_the_least_important_goes_not_the_line_that_has_waited_longest()
     {
        let q = AnnouncementQueue::new().unwrap();
        fill_to_capacity(&q, Priority::Low);
        q.push(Announcement::interface("connection lost", Priority::Urgent))
            .unwrap();

        // The rate limit spreads the rest over several windows, so collect
        // until the queue runs dry.
        let mut all = Vec::new();
        let mut now = t0();
        for _ in 0..12 {
            let batch = drained(&q, now);
            if batch.is_empty() {
                break;
            }
            all.extend(batch);
            now += WINDOW + Duration::from_millis(1);
        }

        assert!(
            all.iter().any(|s| s == "chatter 0"),
            "the line that has waited longest must not be the one sacrificed: {:?}",
            all
        );
        assert!(
            !all.iter().any(|s| s == "chatter 31"),
            "the newest of the least important is what makes room: {:?}",
            all
        );
    }

    #[test]
    fn test_making_room_for_an_urgent_line_still_reports_the_line_it_displaced() {
        let q = AnnouncementQueue::new().unwrap();
        fill_to_capacity(&q, Priority::Low);
        q.push(Announcement::interface("connection lost", Priority::Urgent))
            .unwrap();
        let spoken = drained(&q, t0());
        assert!(
            spoken.iter().any(|s| s.contains("skipped")),
            "a line dropped to make room is still a line the user is owed: {:?}",
            spoken
        );
    }

    #[test]
    fn test_one_skipped_announcement_is_counted_in_the_singular() {
        let q = AnnouncementQueue::new().unwrap();
        fill_to_capacity(&q, Priority::Normal);
        q.push(Announcement::interface("one too many", Priority::Normal))
            .unwrap();
        // This sentence is heard, not read, so the grammar is the feature.
        assert_eq!(
            drained(&q, t0()).last().map(String::as_str),
            Some("1 announcement skipped")
        );
    }

    #[test]
    fn test_a_second_drain_inside_the_same_second_releases_nothing_more() {
        let q = AnnouncementQueue::new().unwrap();
        let now = t0();
        for n in 0..(MAX_PER_WINDOW + 2) {
            q.push(Announcement::interface(
                format!("item {}", n),
                Priority::Normal,
            ))
            .unwrap();
        }
        assert_eq!(drained(&q, now).len(), MAX_PER_WINDOW);
        assert!(
            drained(&q, now + Duration::from_millis(1)).is_empty(),
            "a syncing mailbox must not get around the pace by draining again"
        );
    }

    #[test]
    fn test_the_rate_limit_lifts_once_a_full_second_has_passed() {
        // A boundary test, and a small one: it says only that the window is
        // over at exactly one second rather than just after it.
        let q = AnnouncementQueue::new().unwrap();
        let now = t0();
        for n in 0..(MAX_PER_WINDOW + 2) {
            q.push(Announcement::interface(
                format!("item {}", n),
                Priority::Normal,
            ))
            .unwrap();
        }
        assert_eq!(drained(&q, now).len(), MAX_PER_WINDOW);
        assert_eq!(drained(&q, now + WINDOW).len(), 2);
    }

    #[test]
    fn test_mute_silences_content_but_not_interface() {
        let q = AnnouncementQueue::new().unwrap();
        q.set_muted(true);
        q.push(Announcement::content("Dear Bob, the salary figures are"))
            .unwrap();
        q.push(Announcement::interface("Connected", Priority::Normal))
            .unwrap();
        assert_eq!(drained(&q, t0()), vec!["Connected"]);
    }

    #[test]
    fn test_unmuting_restores_content() {
        let q = AnnouncementQueue::new().unwrap();
        q.set_muted(true);
        q.push(Announcement::content("first body")).unwrap();
        q.set_muted(false);
        q.push(Announcement::content("second body")).unwrap();
        assert_eq!(drained(&q, t0()), vec!["second body"]);
    }

    #[test]
    fn test_rate_limit_holds_the_rest_of_a_burst() {
        let q = AnnouncementQueue::new().unwrap();
        for n in 0..(MAX_PER_WINDOW + 3) {
            q.push(Announcement::interface(
                format!("item {}", n),
                Priority::Normal,
            ))
            .unwrap();
        }
        assert_eq!(drained(&q, t0()).len(), MAX_PER_WINDOW);
    }

    #[test]
    fn test_held_announcements_emit_in_a_later_window() {
        let q = AnnouncementQueue::new().unwrap();
        let now = t0();
        for n in 0..(MAX_PER_WINDOW + 2) {
            q.push(Announcement::interface(
                format!("item {}", n),
                Priority::Normal,
            ))
            .unwrap();
        }
        let first = drained(&q, now);
        assert_eq!(first.len(), MAX_PER_WINDOW);
        let later = drained(&q, now + WINDOW + Duration::from_millis(1));
        assert_eq!(later.len(), 2, "held announcements must not be lost");
    }

    #[test]
    fn test_urgent_is_never_held_back_by_the_rate_limit() {
        let q = AnnouncementQueue::new().unwrap();
        let now = t0();
        for n in 0..MAX_PER_WINDOW {
            q.push(Announcement::interface(
                format!("item {}", n),
                Priority::Normal,
            ))
            .unwrap();
        }
        drained(&q, now);
        q.push(Announcement::interface("connection lost", Priority::Urgent))
            .unwrap();
        assert_eq!(drained(&q, now), vec!["connection lost"]);
    }

    #[test]
    fn test_draining_an_empty_queue_says_nothing() {
        let q = AnnouncementQueue::new().unwrap();
        assert!(drained(&q, t0()).is_empty());
    }
}

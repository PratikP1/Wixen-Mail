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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Most announcements that may be waiting at once. Beyond this the queue is
/// already further behind than anyone will listen to.
pub const CAPACITY: usize = 32;

/// Most announcements spoken per `WINDOW`.
pub const MAX_PER_WINDOW: usize = 4;

/// The period the rate limit applies over.
pub const WINDOW: Duration = Duration::from_secs(1);

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

/// One announcement waiting to be spoken.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Announcement {
    pub text: String,
    pub priority: Priority,
    pub kind: Kind,
    /// Announcements sharing a topic supersede one another, so a counter that
    /// climbs from 1 to 500 is spoken once at its final value.
    pub topic: Option<String>,
}

impl Announcement {
    /// An announcement about the application itself. Never muted.
    pub fn interface(text: impl Into<String>, priority: Priority) -> Self {
        Self {
            text: text.into(),
            priority,
            kind: Kind::Interface,
            topic: None,
        }
    }

    /// Message text read aloud. Silenced by mute.
    pub fn content(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            priority: Priority::Normal,
            kind: Kind::Content,
            topic: None,
        }
    }

    /// Group this with others that supersede one another.
    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
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
        if let Some(topic) = announcement.topic.as_deref() {
            if let Some(existing) = state
                .pending
                .iter_mut()
                .find(|p| p.announcement.topic.as_deref() == Some(topic))
            {
                existing.announcement = announcement;
                return Ok(());
            }
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
                });
            } else {
                held.push(entry);
            }
        }
        state.pending = held;

        if state.skipped > 0 && !spoken.is_empty() {
            let skipped = std::mem::take(&mut state.skipped);
            spoken.push(Spoken {
                text: format!("{} announcements skipped", skipped),
                priority: Priority::Normal,
                kind: Kind::Interface,
                topic: None,
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
mod tests {
    use super::*;

    /// A fixed starting point so tests never depend on wall-clock time.
    fn t0() -> Instant {
        Instant::now()
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

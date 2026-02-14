//! Announcement queue for screen reader messages

use crate::common::Result;
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// Priority levels for announcements
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Normal,
    High,
    Urgent,
}

/// Queues and manages screen reader announcements
pub struct AnnouncementQueue {
    queue: Mutex<BinaryHeap<PrioritizedAnnouncement>>,
    sequence: AtomicU64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct PrioritizedAnnouncement {
    rank: u8,
    sequence: u64,
    text: String,
}

impl Ord for PrioritizedAnnouncement {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Lower rank is higher priority (Urgent=0).
        // For equal rank, lower sequence is older and should win (FIFO).
        other
            .rank
            .cmp(&self.rank)
            .then_with(|| other.sequence.cmp(&self.sequence))
    }
}

impl PartialOrd for PrioritizedAnnouncement {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl AnnouncementQueue {
    /// Create a new announcement queue
    pub fn new() -> Result<Self> {
        Ok(Self {
            queue: Mutex::new(BinaryHeap::new()),
            sequence: AtomicU64::new(0),
        })
    }

    /// Add an announcement to the queue
    pub fn announce(&self, text: &str, priority: Priority) -> Result<()> {
        let mut queue = self
            .queue
            .lock()
            .map_err(|_| crate::common::Error::Other("Announcement queue lock poisoned".to_string()))?;
        let rank = match priority {
            Priority::Urgent => 0,
            Priority::High => 1,
            Priority::Normal => 2,
            Priority::Low => 3,
        };
        let seq = self.sequence.fetch_add(1, Ordering::Relaxed);
        queue.push(PrioritizedAnnouncement {
            rank,
            sequence: seq,
            text: text.to_string(),
        });
        Ok(())
    }

    /// Get next announcement in priority order
    pub fn pop_next(&self) -> Result<Option<String>> {
        let mut queue = self
            .queue
            .lock()
            .map_err(|_| crate::common::Error::Other("Announcement queue lock poisoned".to_string()))?;
        Ok(queue.pop().map(|item| item.text))
    }
}

impl Default for AnnouncementQueue {
    fn default() -> Self {
        Self {
            queue: Mutex::new(BinaryHeap::new()),
            sequence: AtomicU64::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_announcement_priority_order() {
        let queue = AnnouncementQueue::new().unwrap();
        queue.announce("normal", Priority::Normal).unwrap();
        queue.announce("urgent", Priority::Urgent).unwrap();
        assert_eq!(queue.pop_next().unwrap().as_deref(), Some("urgent"));
        assert_eq!(queue.pop_next().unwrap().as_deref(), Some("normal"));
    }

    #[test]
    fn test_announcement_fifo_within_priority() {
        let queue = AnnouncementQueue::new().unwrap();
        queue.announce("first", Priority::Normal).unwrap();
        queue.announce("second", Priority::Normal).unwrap();
        assert_eq!(queue.pop_next().unwrap().as_deref(), Some("first"));
        assert_eq!(queue.pop_next().unwrap().as_deref(), Some("second"));
    }
}

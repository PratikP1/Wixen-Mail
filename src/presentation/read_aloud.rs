//! Reading the item under the cursor aloud, in every module.
//!
//! A screen reader reads a list row: the visible columns, and nothing else. In
//! mail that is a subject and a sender, in tasks a title and a date. Whatever
//! else the record holds, the body, the description, the location, the phone
//! number, is invisible until the item is opened, and opening every item to
//! find out whether it was worth opening is how a long list becomes unusable.
//!
//! Space answers that without leaving the list. The first press reads the short
//! form, the second reads the whole thing, and a third goes back to short.
//! Shift+Space reads the whole thing outright for anyone who does not want to
//! count presses.
//!
//! Cycling on the press rather than on a timer is deliberate. A double-press
//! window is a timing trap (WCAG 2.2.1) and locks out anyone who types slowly,
//! so here the second press does the second thing however long it took.

use super::ui_types::{
    CalendarEventItem, ContactItem, MessageItem, NoteItem, ReminderItem, TaskItem,
};

/// How much of an item to read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Depth {
    /// A line: enough to decide whether to keep going.
    Short,
    /// Everything the record holds.
    Full,
}

/// Which item the cursor is on, so a move resets the cycle.
///
/// Without this, arrowing to a new row and pressing Space would read the new
/// row's full text because the last row had already been pressed once, which
/// is a different answer to the same keystroke depending on history.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Position {
    module: &'static str,
    id: String,
}

/// Tracks what the next Space press should read.
#[derive(Debug, Default)]
pub struct SpaceCycle {
    at: Option<Position>,
    depth_next: Option<Depth>,
}

impl SpaceCycle {
    pub fn new() -> Self {
        Self::default()
    }

    /// What this Space press should read.
    ///
    /// Moving to a different item starts again at the short form.
    pub fn press(&mut self, module: &'static str, id: &str) -> Depth {
        let here = Position {
            module,
            id: id.to_string(),
        };
        let depth = if self.at.as_ref() == Some(&here) {
            match self.depth_next {
                Some(Depth::Full) => Depth::Full,
                _ => Depth::Short,
            }
        } else {
            Depth::Short
        };
        self.at = Some(here);
        self.depth_next = Some(match depth {
            Depth::Short => Depth::Full,
            Depth::Full => Depth::Short,
        });
        depth
    }

    /// Shift+Space: the whole thing, whatever the cycle was doing.
    ///
    /// The cycle is set so the next plain Space goes back to short, because
    /// having just heard everything, the useful next answer is the summary.
    pub fn press_full(&mut self, module: &'static str, id: &str) -> Depth {
        self.at = Some(Position {
            module,
            id: id.to_string(),
        });
        self.depth_next = Some(Depth::Short);
        Depth::Full
    }

    /// Forget where we were, for when a list reloads underneath the cursor.
    pub fn reset(&mut self) {
        self.at = None;
        self.depth_next = None;
    }
}

/// Something a list row can read aloud.
///
/// One trait across six modules so Space behaves identically everywhere.
/// Predictable behaviour across modules is WCAG 3.2.3 and 3.2.4, and it is
/// also the difference between learning one key and learning six.
pub trait ReadAloud {
    /// The identity used to tell one row from the next.
    fn read_id(&self) -> String;
    /// A line.
    fn read_short(&self) -> String;
    /// Everything the record holds.
    fn read_full(&self) -> String;

    /// The text for a depth.
    fn read(&self, depth: Depth) -> String {
        match depth {
            Depth::Short => self.read_short(),
            Depth::Full => self.read_full(),
        }
    }
}

/// Join the parts of a spoken description, dropping the empty ones.
///
/// An empty field read as "Company, , phone" sounds like a stutter, and a
/// literal "none" on every field triples the length of the reading.
fn spoken(parts: &[(&str, &str)]) -> String {
    parts
        .iter()
        .filter(|(_, value)| !value.trim().is_empty())
        .map(|(label, value)| {
            if label.is_empty() {
                value.trim().to_string()
            } else {
                format!("{}: {}", label, value.trim())
            }
        })
        .collect::<Vec<_>>()
        .join(". ")
}

impl ReadAloud for MessageItem {
    fn read_id(&self) -> String {
        self.message_id.to_string()
    }

    fn read_short(&self) -> String {
        let subject = if self.subject.trim().is_empty() {
            "No subject"
        } else {
            self.subject.trim()
        };
        spoken(&[("", subject), ("From", &self.from), ("", &self.snippet)])
    }

    fn read_full(&self) -> String {
        let attachments = if self.has_attachments {
            format!("{} attachments", self.attachments.len().max(1))
        } else {
            String::new()
        };
        let flags = [
            (!self.read).then_some("unread"),
            self.starred.then_some("flagged"),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(", ");

        spoken(&[
            (
                "",
                if self.subject.trim().is_empty() {
                    "No subject"
                } else {
                    self.subject.trim()
                },
            ),
            ("From", &self.from),
            ("To", &self.to),
            ("Cc", &self.cc),
            ("Received", &self.date),
            ("", &flags),
            ("", &attachments),
            ("", &self.snippet),
        ])
    }
}

impl ReadAloud for ContactItem {
    fn read_id(&self) -> String {
        self.id.clone()
    }

    fn read_short(&self) -> String {
        spoken(&[("", &self.name), ("", &self.email)])
    }

    fn read_full(&self) -> String {
        spoken(&[
            ("", &self.name),
            ("Email", &self.email),
            ("Phone", &self.phone),
            ("Company", &self.company),
            ("", if self.favorite { "Favourite" } else { "" }),
        ])
    }
}

impl ReadAloud for NoteItem {
    fn read_id(&self) -> String {
        self.id.clone()
    }

    fn read_short(&self) -> String {
        spoken(&[("", &self.title), ("", &self.body_preview)])
    }

    fn read_full(&self) -> String {
        spoken(&[
            ("", &self.title),
            ("", if self.pinned { "Pinned" } else { "" }),
            ("Updated", &self.updated_at),
            ("", &self.body_preview),
        ])
    }
}

impl ReadAloud for TaskItem {
    fn read_id(&self) -> String {
        self.id.clone()
    }

    fn read_short(&self) -> String {
        spoken(&[
            ("", &self.title),
            ("", if self.is_completed { "done" } else { "" }),
            ("Due", self.due_date.as_deref().unwrap_or("")),
        ])
    }

    fn read_full(&self) -> String {
        spoken(&[
            ("", &self.title),
            (
                "",
                if self.is_completed {
                    "Completed"
                } else {
                    "Not completed"
                },
            ),
            ("Priority", &self.priority),
            ("Due", self.due_date.as_deref().unwrap_or("")),
            ("", self.description.as_deref().unwrap_or("")),
        ])
    }
}

impl ReadAloud for ReminderItem {
    fn read_id(&self) -> String {
        self.id.clone()
    }

    fn read_short(&self) -> String {
        spoken(&[
            ("", &self.title),
            ("", if self.is_completed { "done" } else { "" }),
            ("Due", self.due_datetime.as_deref().unwrap_or("")),
        ])
    }

    fn read_full(&self) -> String {
        spoken(&[
            ("", &self.title),
            (
                "",
                if self.is_completed {
                    "Completed"
                } else {
                    "Not completed"
                },
            ),
            ("Priority", &self.priority),
            ("Due", self.due_datetime.as_deref().unwrap_or("")),
            ("", self.description.as_deref().unwrap_or("")),
        ])
    }
}

impl ReadAloud for CalendarEventItem {
    fn read_id(&self) -> String {
        self.id.clone()
    }

    fn read_short(&self) -> String {
        let when = if self.is_all_day {
            format!("{}, all day", self.start)
        } else {
            self.start.clone()
        };
        spoken(&[("", &self.summary), ("", &when)])
    }

    fn read_full(&self) -> String {
        let when = if self.is_all_day {
            format!("{}, all day", self.start)
        } else {
            format!("{} to {}", self.start, self.end)
        };
        spoken(&[
            ("", &self.summary),
            ("", &when),
            ("Location", &self.location),
            ("Status", &self.status),
            ("Calendar", self.calendar_name.as_deref().unwrap_or("")),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message() -> MessageItem {
        MessageItem {
            uid: 1,
            message_id: 7,
            subject: "Quarterly report".to_string(),
            from: "Ada Lovelace <ada@example.com>".to_string(),
            date: "2026-07-26".to_string(),
            read: false,
            starred: true,
            answered: false,
            draft: false,
            has_attachments: true,
            attachments: Vec::new(),
            thread_depth: 0,
            is_thread_parent: false,
            thread_id: None,
            snippet: "The numbers are attached.".to_string(),
            size_bytes: Some(2048),
            to: "me@example.com".to_string(),
            cc: String::new(),
            reply_to: String::new(),
            safety: crate::service::safety::Safety::Ordinary,
            safety_reasons: Vec::new(),
            receipt_to: None,
        }
    }

    #[test]
    fn test_space_reads_short_then_full_then_short() {
        let mut cycle = SpaceCycle::new();
        assert_eq!(cycle.press("mail", "7"), Depth::Short);
        assert_eq!(cycle.press("mail", "7"), Depth::Full);
        assert_eq!(cycle.press("mail", "7"), Depth::Short);
        assert_eq!(cycle.press("mail", "7"), Depth::Full);
    }

    #[test]
    fn test_moving_to_another_item_starts_again_at_short() {
        // Otherwise the same keystroke gives a different answer depending on
        // what the previous row had already been asked.
        let mut cycle = SpaceCycle::new();
        cycle.press("mail", "7");
        assert_eq!(cycle.press("mail", "8"), Depth::Short);
        assert_eq!(cycle.press("mail", "8"), Depth::Full);
    }

    #[test]
    fn test_the_same_id_in_another_module_is_a_different_item() {
        // Ids are only unique within a module, so "3" in tasks and "3" in
        // notes are different rows.
        let mut cycle = SpaceCycle::new();
        cycle.press("tasks", "3");
        assert_eq!(cycle.press("notes", "3"), Depth::Short);
    }

    #[test]
    fn test_shift_space_reads_everything_without_counting_presses() {
        let mut cycle = SpaceCycle::new();
        assert_eq!(cycle.press_full("mail", "7"), Depth::Full);
        // And the next plain Space gives the summary, which is the useful
        // answer right after hearing the whole thing.
        assert_eq!(cycle.press("mail", "7"), Depth::Short);
    }

    #[test]
    fn test_a_reload_under_the_cursor_starts_the_cycle_again() {
        let mut cycle = SpaceCycle::new();
        cycle.press("mail", "7");
        cycle.reset();
        assert_eq!(cycle.press("mail", "7"), Depth::Short);
    }

    #[test]
    fn test_a_short_message_reading_is_subject_sender_snippet() {
        assert_eq!(
            message().read_short(),
            "Quarterly report. From: Ada Lovelace <ada@example.com>. The numbers are attached."
        );
    }

    #[test]
    fn test_a_full_message_reading_carries_what_the_columns_hide() {
        let full = message().read_full();
        assert!(full.contains("To: me@example.com"));
        assert!(full.contains("unread, flagged"));
        assert!(full.contains("attachments"));
        // And nothing empty leaks in as a stutter.
        assert!(!full.contains(". ."), "empty field left a gap: {}", full);
        assert!(!full.contains("Cc:"), "an empty cc was read: {}", full);
    }

    #[test]
    fn test_a_message_with_no_subject_says_so_in_both_forms() {
        // An empty subject read as nothing is indistinguishable from a row
        // that failed to load.
        let mut m = message();
        m.subject = "   ".to_string();
        assert!(m.read_short().starts_with("No subject"));
        assert!(m.read_full().starts_with("No subject"));
    }

    #[test]
    fn test_every_module_reads_something_in_both_depths() {
        // Space has to answer in all six modules. A module where it returns
        // nothing is a key that appears broken.
        let contact = ContactItem {
            id: "c1".to_string(),
            name: "Grace Hopper".to_string(),
            email: "grace@example.com".to_string(),
            phone: String::new(),
            company: String::new(),
            favorite: false,
        };
        let note = NoteItem {
            id: "n1".to_string(),
            title: "Shopping".to_string(),
            body_preview: "Milk".to_string(),
            pinned: false,
            updated_at: "2026-07-26".to_string(),
            folder_id: None,
        };
        let task = TaskItem {
            id: "t1".to_string(),
            title: "File the report".to_string(),
            description: None,
            due_date: None,
            is_completed: false,
            priority: "normal".to_string(),
            task_list_id: None,
            parent_task_id: None,
        };
        let reminder = ReminderItem {
            id: "r1".to_string(),
            title: "Call the dentist".to_string(),
            description: None,
            due_datetime: None,
            is_completed: false,
            priority: "high".to_string(),
        };
        let event = CalendarEventItem {
            id: "e1".to_string(),
            summary: "Standup".to_string(),
            start: "2026-07-27 09:00".to_string(),
            end: "2026-07-27 09:15".to_string(),
            location: String::new(),
            is_all_day: false,
            status: "confirmed".to_string(),
            provider: "local".to_string(),
            calendar_id: None,
            calendar_name: None,
            calendar_color: None,
        };

        let mail = message();
        let readers: Vec<(&str, &dyn ReadAloud)> = vec![
            ("message", &mail),
            ("contact", &contact),
            ("note", &note),
            ("task", &task),
            ("reminder", &reminder),
            ("event", &event),
        ];
        for (what, reader) in readers {
            for depth in [Depth::Short, Depth::Full] {
                let text = reader.read(depth);
                assert!(
                    !text.trim().is_empty(),
                    "{} read nothing at {:?}",
                    what,
                    depth
                );
                assert!(
                    !text.contains(". ."),
                    "{} left a gap at {:?}: {}",
                    what,
                    depth,
                    text
                );
            }
            assert!(!reader.read_id().is_empty(), "{} has no id", what);
        }
    }

    #[test]
    fn test_an_all_day_event_says_so_rather_than_reading_two_identical_times() {
        let event = CalendarEventItem {
            id: "e2".to_string(),
            summary: "Public holiday".to_string(),
            start: "2026-08-31".to_string(),
            end: "2026-08-31".to_string(),
            location: String::new(),
            is_all_day: true,
            status: "confirmed".to_string(),
            provider: "local".to_string(),
            calendar_id: None,
            calendar_name: None,
            calendar_color: None,
        };
        assert!(event.read_full().contains("all day"));
        assert!(!event.read_full().contains("2026-08-31 to 2026-08-31"));
    }
}

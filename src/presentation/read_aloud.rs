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
use crate::application::long_text;

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
    fn read_short(&self, out: Reading) -> String;
    /// Everything the record holds.
    fn read_full(&self, out: Reading) -> String;

    /// The text for a depth.
    fn read(&self, depth: Depth, out: Reading) -> String {
        match depth {
            Depth::Short => self.read_short(out),
            Depth::Full => self.read_full(out),
        }
    }
}

/// What a reading needs to know beyond the item itself.
///
/// One value rather than two parameters on every method of the trait, so the
/// next thing a reading depends on does not mean touching six implementations
/// again.
///
/// `now` is carried rather than asked for inside, because a reading is one
/// utterance and every date in it should be measured from the same instant.
#[derive(Debug, Clone, Copy)]
pub struct Reading {
    pub dates: crate::presentation::date_display::DateSettings,
    pub now: chrono::DateTime<chrono::Local>,
}

impl Reading {
    /// One stored date, written the way this reader asked for it.
    ///
    /// Reachable by the reader window's composition too, so a date at the top
    /// of a message and the same date in a list column are written one way
    /// rather than two.
    pub(super) fn date(&self, stored: &str) -> String {
        crate::presentation::date_display::spoken(stored, self.now, self.dates)
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

/// The word for whether an item is finished.
///
/// The same word the list column uses, so one state is called one thing
/// wherever it is met (WCAG 3.2.4). The short reading says nothing at all when
/// an item is unfinished, to keep it to a line; the full reading says it,
/// because silence cannot be told apart from a reading that failed.
fn finished_wording(is_completed: bool) -> &'static str {
    if is_completed { "Done" } else { "Not done" }
}

/// The same word, said only when the item is finished, for the short reading.
fn finished_wording_when_it_is(is_completed: bool) -> &'static str {
    if is_completed {
        finished_wording(is_completed)
    } else {
        ""
    }
}

/// A priority worth the words, which is any but the ordinary one.
///
/// Nearly every task and reminder carries the default, so saying it on every
/// row spends two words that never vary and never inform. The unread and
/// flagged wording above already works this way.
///
/// Shared with the list cells rather than decided twice, so a task read out and
/// the same task arrowed past in a list agree about what is worth hearing.
pub(super) fn priority_worth_saying(priority: &str) -> &str {
    if priority.eq_ignore_ascii_case("normal") {
        ""
    } else {
        priority
    }
}

/// A status worth the words. Nearly every event is confirmed; a cancelled one
/// is the reason this is read at all.
///
/// Shared with the list cells for the same reason as the priority above.
pub(super) fn status_worth_saying(status: &str) -> &str {
    if status.eq_ignore_ascii_case("confirmed") {
        ""
    } else {
        status
    }
}

/// What a reading can honestly say about what is attached.
///
/// A list row knows whether anything is attached and not what, because a folder
/// listing that loaded the parts would do a query per row. So a row says the
/// same three words the Attachment column says, and one fact is called one
/// thing wherever it is met (WCAG 3.2.4). Only a reading that was given the
/// parts counts them, and then the count is real.
///
/// This used to be `len().max(1)` on a list whose length is always zero, so
/// every message with anything attached was read as "1 attachments": the wrong
/// number and the wrong plural in the same three words.
fn attachment_wording(has_attachments: bool, named: usize) -> String {
    match named {
        0 if has_attachments => "Has attachment".to_string(),
        0 => String::new(),
        1 => "1 attachment".to_string(),
        many => format!("{many} attachments"),
    }
}

/// What a message's state is worth saying, beyond what the reading already has.
///
/// Whether it is unread, whether it is flagged, and the labels somebody put on
/// it. Shared so the two forms of the fuller reading describe one message the
/// same way: a message read from the row and the same message read with its
/// body must not be two different messages to anyone listening.
///
/// Labels are the reason this is separate rather than inline. They are on no
/// column and in no header block, so this is the only place a label reaches
/// anybody who cannot see the row it sits on.
///
/// Empty when there is nothing to say, so the join above drops it rather than
/// leaving a gap.
pub(super) fn state_worth_saying(message: &MessageItem) -> String {
    let flags = [
        (!message.read).then_some("unread"),
        message.starred.then_some("flagged"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(", ");
    spoken(&[
        ("", &flags),
        (
            "Labels",
            &crate::application::tagging::joined(&message.labels),
        ),
    ])
}

impl ReadAloud for MessageItem {
    fn read_id(&self) -> String {
        self.message_id.to_string()
    }

    fn read_short(&self, _out: Reading) -> String {
        let subject = if self.subject.trim().is_empty() {
            "No subject"
        } else {
            self.subject.trim()
        };
        // Nothing rather than the words a list column uses for text nobody
        // has fetched. A row says "Message text not downloaded" because a
        // blank cell there would read as a message with nothing in it; a
        // reading aloud that has already said the subject and the sender does
        // not need to be told that the part it does not have is not there.
        let first_line = self.snippet.as_deref().unwrap_or_default();
        spoken(&[("", subject), ("From", &self.from), ("", first_line)])
    }

    fn read_full(&self, out: Reading) -> String {
        let attachments = attachment_wording(self.has_attachments, self.attachments.len());
        let state = state_worth_saying(self);

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
            ("Received", &out.date(&self.date)),
            // Said, because a label is not visible from the row it is on and a
            // colour swatch is not a thing everybody can read.
            ("", &state),
            ("", &attachments),
            ("", self.snippet.as_deref().unwrap_or_default()),
        ])
    }
}

impl ReadAloud for ContactItem {
    fn read_id(&self) -> String {
        self.id.clone()
    }

    fn read_short(&self, _out: Reading) -> String {
        spoken(&[("", &self.name), ("", &self.email)])
    }

    fn read_full(&self, out: Reading) -> String {
        // Not `out.date`, deliberately. That measures against now and would
        // read a birthday falling this week as "2 days ago", which is not
        // what anybody asked about a birthday, and it answers nothing at all
        // for one stored without a year.
        let birthday = crate::presentation::date_display::a_day_in_words(&self.birthday, out.dates);
        spoken(&[
            ("", &self.name),
            ("Email", &self.email),
            (&self.phone_label, &self.phone),
            ("Company", &self.company),
            (&self.address_label, &self.address),
            ("Birthday", &birthday),
            // Spelled the way the menu item, the tree node and the detail
            // pane spell it. One record read two ways is two records to
            // anybody listening.
            ("", if self.favorite { "Favorite" } else { "" }),
            // Last, because it is the long one. The short facts come first so
            // somebody who wanted the phone number is not made to sit through
            // half a page of notes to reach it.
            ("Notes", &long_text::spoken(&self.notes)),
        ])
    }
}

impl ReadAloud for NoteItem {
    fn read_id(&self) -> String {
        self.id.clone()
    }

    fn read_short(&self, _out: Reading) -> String {
        spoken(&[("", &self.title), ("", &self.body_preview)])
    }

    fn read_full(&self, out: Reading) -> String {
        spoken(&[
            ("", &self.title),
            ("", if self.pinned { "Pinned" } else { "" }),
            ("Updated", &out.date(&self.updated_at)),
            ("", &long_text::spoken(&self.body)),
        ])
    }
}

impl ReadAloud for TaskItem {
    fn read_id(&self) -> String {
        self.id.clone()
    }

    fn read_short(&self, out: Reading) -> String {
        spoken(&[
            ("", &self.title),
            ("", finished_wording_when_it_is(self.is_completed)),
            ("Due", &out.date(self.due_date.as_deref().unwrap_or(""))),
        ])
    }

    fn read_full(&self, out: Reading) -> String {
        spoken(&[
            ("", &self.title),
            ("", finished_wording(self.is_completed)),
            ("Priority", priority_worth_saying(&self.priority)),
            ("Due", &out.date(self.due_date.as_deref().unwrap_or(""))),
            (
                "",
                &long_text::spoken(self.description.as_deref().unwrap_or("")),
            ),
        ])
    }
}

impl ReadAloud for ReminderItem {
    fn read_id(&self) -> String {
        self.id.clone()
    }

    fn read_short(&self, out: Reading) -> String {
        spoken(&[
            ("", &self.title),
            ("", finished_wording_when_it_is(self.is_completed)),
            ("Due", &out.date(self.due_datetime.as_deref().unwrap_or(""))),
        ])
    }

    fn read_full(&self, out: Reading) -> String {
        spoken(&[
            ("", &self.title),
            ("", finished_wording(self.is_completed)),
            ("Priority", priority_worth_saying(&self.priority)),
            ("Due", &out.date(self.due_datetime.as_deref().unwrap_or(""))),
            (
                "",
                &long_text::spoken(self.description.as_deref().unwrap_or("")),
            ),
        ])
    }
}

impl ReadAloud for CalendarEventItem {
    fn read_id(&self) -> String {
        // Every day of a series carries the stored event's identity, so the day
        // has to come into it or all fifty-two Tuesdays are one item and the
        // Space cycle carries on from whichever one was pressed last.
        if self.repeats.is_empty() {
            return self.id.clone();
        }
        format!("{} {}", self.id, self.start)
    }

    fn read_short(&self, out: Reading) -> String {
        let when = if self.is_all_day {
            format!("{}, all day", out.date(&self.start))
        } else {
            out.date(&self.start)
        };
        // A series that was worked out has its own days on the screen to say it
        // repeats. One that could not be worked out has nothing at all: a
        // single row that looks exactly like an event happening once. That is
        // the only case worth spending a line on without being asked.
        let unreadable = if self.repeats == crate::application::occurrences::CANNOT_BE_READ {
            self.repeats.as_str()
        } else {
            ""
        };
        spoken(&[("", &self.summary), ("", &when), ("", unreadable)])
    }

    fn read_full(&self, out: Reading) -> String {
        let start = out.date(&self.start);
        let end = out.date(&self.end);
        // Joined before the empty parts are dropped, so a missing end has to
        // be handled here: "9:00 AM to" and then silence sounds like the
        // reading was cut off.
        let when = if self.is_all_day {
            format!("{start}, all day")
        } else if end.trim().is_empty() || end == start {
            start
        } else if start.trim().is_empty() {
            end
        } else {
            format!("{start} to {end}")
        };
        spoken(&[
            ("", &self.summary),
            ("", &when),
            ("Location", &self.location),
            ("Status", status_worth_saying(&self.status)),
            ("Calendar", self.calendar_name.as_deref().unwrap_or("")),
            // Both empty on an ordinary event, and `spoken` drops empty parts,
            // so a plain calendar costs nothing to listen to.
            ("", &self.repeats),
            (
                "",
                &crate::application::categories::spoken(&self.categories),
            ),
            ("", &long_text::spoken(&self.description)),
        ])
    }
}

#[cfg(test)]
mod tests {
    // Every test here asserts on the words this module asks to have said. That
    // is all a test can settle. Whether any of it reaches a listener runs
    // through the announcement queue and a Windows notification call, and only
    // a screen reader pass answers that.
    use super::*;

    /// Fixed rather than read from the machine, so these read the same
    /// wherever they run.
    fn aloud() -> Reading {
        use crate::presentation::date_display::{Clock, DateOrder, DateStyle, DateWording};
        use chrono::TimeZone;
        Reading {
            dates: crate::presentation::date_display::DateSettings {
                style: DateStyle::Absolute,
                order: DateOrder::MonthFirst,
                wording: DateWording::Verbal,
                clock: Clock::TwelveHour,
            },
            now: chrono::Local
                .with_ymd_and_hms(2026, 7, 26, 12, 0, 0)
                .single()
                .expect("a real moment"),
        }
    }

    /// The settings people actually get. Nothing changes the date style until
    /// somebody visits the settings screen, so this is the reading every list
    /// ships with. Everything else mirrors [`aloud`], fixed so these read the
    /// same wherever they run.
    fn under_the_shipped_default() -> Reading {
        use crate::presentation::date_display::DateStyle;
        Reading {
            dates: crate::presentation::date_display::DateSettings {
                style: DateStyle::RelativeWithinWeek,
                ..aloud().dates
            },
            ..aloud()
        }
    }

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
            snippet: Some("The numbers are attached.".to_string()),
            size_bytes: Some(2048),
            to: "me@example.com".to_string(),
            cc: String::new(),
            reply_to: String::new(),
            header_message_id: String::new(),
            refs_header: None,
            safety: crate::service::safety::Safety::Ordinary,
            safety_reasons: Vec::new(),
            receipt_to: None,
            list_unsubscribe: None,
            account_id: String::new(),
            labels: Vec::new(),
        }
    }

    fn attachment(filename: &str) -> crate::presentation::ui_types::AttachmentItem {
        crate::presentation::ui_types::AttachmentItem {
            filename: filename.to_string(),
            mime_type: "application/octet-stream".to_string(),
            size: 1024,
            description: crate::service::mime::WhatTheSenderSaid::Nothing,
        }
    }

    fn contact() -> ContactItem {
        ContactItem {
            id: "c1".to_string(),
            name: "Grace Hopper".to_string(),
            email: "grace@example.com".to_string(),
            phone: String::new(),
            phone_label: String::new(),
            company: String::new(),
            address: String::new(),
            address_label: String::new(),
            birthday: String::new(),
            favorite: false,
            notes: String::new(),
        }
    }

    #[test]
    fn test_a_contacts_notes_are_read_with_the_structure_written_into_them() {
        // The one long field that markdown did nothing in. A contact's notes
        // are a multi-line box like a note's body and an event's description,
        // they sync to and from Google and Outlook, and they were read out as
        // nothing at all: the field was not on the item this reads from, so a
        // contact with half a page of notes said only its name and address.
        let mut contact = contact();
        contact.notes = "# Met at\n\n- Grace Hopper Celebration\n- Introduced by Ada".to_string();

        let said = contact.read_full(aloud());

        assert!(said.contains("heading level 1, Met at"), "{said}");
        assert!(said.contains("bullet, Grace Hopper Celebration"), "{said}");
    }

    fn note() -> NoteItem {
        NoteItem {
            id: "n1".to_string(),
            title: "Shopping".to_string(),
            body: "Milk".to_string(),
            body_preview: "Milk".to_string(),
            pinned: false,
            updated_at: "2026-07-26".to_string(),
            folder_id: None,
        }
    }

    fn task() -> TaskItem {
        TaskItem {
            id: "t1".to_string(),
            title: "File the report".to_string(),
            description: None,
            due_date: None,
            is_completed: false,
            priority: "normal".to_string(),
            task_list_id: None,
            parent_task_id: None,
        }
    }

    fn reminder() -> ReminderItem {
        ReminderItem {
            id: "r1".to_string(),
            title: "Call the dentist".to_string(),
            description: None,
            due_datetime: None,
            is_completed: false,
            priority: "high".to_string(),
        }
    }

    fn event() -> CalendarEventItem {
        CalendarEventItem {
            attendees_json: None,
            id: "e1".to_string(),
            summary: "Standup".to_string(),
            description: String::new(),
            start: "2026-07-27 09:00".to_string(),
            end: "2026-07-27 09:15".to_string(),
            location: String::new(),
            is_all_day: false,
            status: "confirmed".to_string(),
            provider: "local".to_string(),
            calendar_id: None,
            calendar_name: None,
            calendar_color: None,
            reminder_minutes: None,
            repeats: String::new(),
            categories: String::new(),
            show_as: String::new(),
            recurrence_rule: None,
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
            message().read_short(aloud()),
            "Quarterly report. From: Ada Lovelace <ada@example.com>. The numbers are attached."
        );
    }

    #[test]
    fn test_a_full_message_reading_carries_what_the_columns_hide() {
        // The words this reading asks to have said, not proof anybody hears
        // them. This is the reading the mail list falls back to when a
        // message's body has not been downloaded.
        let mut m = message();
        m.labels = vec!["Work".to_string()];
        let full = m.read_full(aloud());
        assert!(full.contains("To: me@example.com"));
        assert!(full.contains("unread, flagged"));
        // Labels are on no column and in no header block, so this reading is
        // the only place they reach somebody who cannot see the row.
        assert!(full.contains("Labels: Work"), "{full}");
        // What the row knows, and no count it does not have.
        assert!(full.contains("Has attachment"), "{full}");
        assert!(!full.contains("1 attachments"), "{full}");
        // And nothing empty leaks in as a stutter.
        assert!(!full.contains(". ."), "empty field left a gap: {}", full);
        assert!(!full.contains("Cc:"), "an empty cc was read: {}", full);
    }

    #[test]
    fn test_a_row_that_only_knows_something_is_attached_does_not_claim_a_count() {
        // A list row carries whether anything is attached, not what: a folder
        // listing that loaded the parts would do a query per row. So the
        // reading may only say that much, in the same words the Attachment
        // column uses. This pins the words this code asks to have said; only a
        // screen reader says whether they are heard.
        let only_that_there_is_one = message().read_full(aloud());
        assert!(
            only_that_there_is_one.contains("Has attachment"),
            "{only_that_there_is_one}"
        );
        assert!(
            !only_that_there_is_one.contains("1 attachments"),
            "a count the row does not have: {only_that_there_is_one}"
        );

        let mut one = message();
        one.attachments = vec![attachment("invoice.pdf")];
        let one = one.read_full(aloud());
        assert!(one.contains("1 attachment"), "{one}");
        assert!(!one.contains("1 attachments"), "wrong plural: {one}");

        let mut two = message();
        two.attachments = vec![attachment("invoice.pdf"), attachment("notes.txt")];
        assert!(two.read_full(aloud()).contains("2 attachments"));
    }

    #[test]
    fn test_a_message_with_nothing_attached_says_nothing_about_attachments() {
        let mut bare = message();
        bare.has_attachments = false;
        assert!(!bare.read_full(aloud()).contains("attachment"));
    }

    #[test]
    fn test_a_message_with_no_subject_says_so_in_both_forms() {
        // An empty subject read as nothing is indistinguishable from a row
        // that failed to load.
        let mut m = message();
        m.subject = "   ".to_string();
        assert!(m.read_short(aloud()).starts_with("No subject"));
        assert!(m.read_full(aloud()).starts_with("No subject"));
    }

    #[test]
    fn test_every_module_reads_something_in_both_depths() {
        // Space has to answer in all six modules. A module where it returns
        // nothing is a key that appears broken.
        let contact = contact();
        let note = note();
        let task = task();
        let reminder = reminder();
        let event = event();

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
                let text = reader.read(depth, aloud());
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
    fn test_each_module_reads_the_rows_own_id_so_moving_between_rows_starts_again() {
        // The id is not decoration. SpaceCycle compares it to decide whether
        // the cursor moved, so a row whose identity is a constant would mean
        // arrowing down and pressing Space read the new row's whole text
        // straight away, which is the exact thing the cycle exists to stop.
        assert_eq!(message().read_id(), "7");
        assert_eq!(contact().read_id(), "c1");
        assert_eq!(note().read_id(), "n1");
        assert_eq!(task().read_id(), "t1");
        assert_eq!(reminder().read_id(), "r1");
        assert_eq!(event().read_id(), "e1");

        // Two different rows in one module, which is what the ids are for.
        let mut cycle = SpaceCycle::new();
        cycle.press("tasks", &task().read_id());
        assert_eq!(cycle.press("tasks", &reminder().read_id()), Depth::Short);
    }

    #[test]
    fn test_a_due_date_in_a_reading_is_spoken_in_words_not_as_stored_digits() {
        // The point of the whole module: a date inside a reading goes through
        // the same wording as a date in a column, rather than being read back
        // as the run of digits the database happens to hold.
        let mut task = task();
        task.due_date = Some("2026-07-30".to_string());

        assert_eq!(
            task.read_short(aloud()),
            "File the report. Due: July 30, 2026"
        );
    }

    #[test]
    fn test_a_short_contact_reading_is_the_name_and_the_address() {
        assert_eq!(
            contact().read_short(aloud()),
            "Grace Hopper. grace@example.com"
        );
    }

    #[test]
    fn test_a_full_contact_reading_carries_the_phone_number_the_column_hides() {
        // The phone number has no column, so this is the only way to hear it
        // without opening the record.
        let mut contact = contact();
        contact.phone = "555 0100".to_string();
        contact.phone_label = "Phone".to_string();
        contact.company = "Analytical Engines".to_string();
        contact.favorite = true;

        assert_eq!(
            contact.read_full(aloud()),
            "Grace Hopper. Email: grace@example.com. Phone: 555 0100. \
             Company: Analytical Engines. Favorite"
        );
    }

    #[test]
    fn test_reading_a_contact_aloud_names_a_lone_phones_label_and_the_address() {
        // Spelled the way the detail pane spells it: the file's own comment
        // on `read_full` says a record read two ways is two records to
        // anybody listening, so this pins that the two agree.
        let mut contact = contact();
        contact.phone = "555 0100".to_string();
        contact.phone_label = "Work".to_string();
        contact.address = "1 Main St".to_string();
        contact.address_label = "Home".to_string();

        let reading = contact.read_full(aloud());

        assert!(reading.contains("Work: 555 0100"), "{reading}");
        assert!(!reading.contains("Phone: 555 0100"), "{reading}");
        assert!(reading.contains("Home: 1 Main St"), "{reading}");
    }

    #[test]
    fn test_a_birthday_with_no_year_is_read_aloud_as_a_day_and_a_month() {
        let mut contact = contact();
        contact.birthday = "--03-14".to_string();

        let reading = contact.read_full(aloud());

        assert!(
            reading.contains("Birthday: March 14th"),
            "a birthday nobody gave a year for is still said as words: {reading}"
        );
    }

    #[test]
    fn test_a_birthday_this_week_is_still_a_date_and_not_how_long_ago_it_was() {
        use crate::presentation::date_display::DateStyle;
        let mut contact = contact();
        // Two days before the fixed "now" the reading is measured from.
        contact.birthday = "2026-07-24".to_string();
        let relative = Reading {
            dates: crate::presentation::date_display::DateSettings {
                style: DateStyle::RelativeWithinWeek,
                ..aloud().dates
            },
            ..aloud()
        };

        let reading = contact.read_full(relative);

        assert!(
            reading.contains("Birthday: July 24, 2026"),
            "a birthday is a day, never a count of days: {reading}"
        );
    }

    #[test]
    fn test_an_empty_contact_field_is_left_out_rather_than_read_as_a_gap() {
        assert_eq!(
            contact().read_full(aloud()),
            "Grace Hopper. Email: grace@example.com"
        );
    }

    #[test]
    fn test_the_short_note_reading_is_the_one_line_preview_and_not_the_whole_note() {
        // The mirror of the full reading: full uses the body, short uses the
        // preview, and with only one of the two pinned they could swap.
        let mut note = note();
        note.body = "Milk, bread, and the thing for the tap.".to_string();
        note.body_preview = "Milk, bread, and the\u{2026}".to_string();

        assert_eq!(
            note.read_short(aloud()),
            "Shopping. Milk, bread, and the\u{2026}"
        );
    }

    #[test]
    fn test_a_finished_task_is_called_done_the_same_word_the_column_uses() {
        // One state, one word, wherever it is met. The column says "Done",
        // so Space says "Done" too (WCAG 3.2.4).
        let mut task = task();
        task.is_completed = true;

        assert_eq!(task.read_short(aloud()), "File the report. Done");
        assert_eq!(task.read_full(aloud()), "File the report. Done");
    }

    #[test]
    fn test_an_unfinished_task_says_so_in_full_rather_than_saying_nothing() {
        // Silence cannot be told apart from a reading that failed. The short
        // form stays quiet to keep it to a line; the full form says it.
        assert_eq!(task().read_full(aloud()), "File the report. Not done");
    }

    #[test]
    fn test_a_full_task_reading_carries_the_description_the_column_hides() {
        let mut task = task();
        task.description = Some("Numbers from the third quarter.".to_string());
        task.due_date = Some("2026-07-30".to_string());
        task.priority = "high".to_string();

        assert_eq!(
            task.read_full(aloud()),
            "File the report. Not done. Priority: high. Due: July 30, 2026. \
             Numbers from the third quarter."
        );
    }

    #[test]
    fn test_an_ordinary_priority_is_not_said_on_every_single_row() {
        // Thirty tasks in a list, thirty times "Priority: normal", and none of
        // them told anybody anything. Said only when it is not the ordinary
        // one, which is how the unread and flagged wording already works.
        assert!(!task().read_full(aloud()).contains("Priority"));

        let mut urgent = task();
        urgent.priority = "high".to_string();
        assert!(urgent.read_full(aloud()).contains("Priority: high"));
    }

    #[test]
    fn test_a_reminder_reads_its_own_due_field_in_the_short_form() {
        // A reminder keeps its due date in a differently named field to a
        // task's, and it carries a time where a task's often does not.
        let mut reminder = reminder();
        reminder.due_datetime = Some("2026-07-30 09:15".to_string());

        assert_eq!(
            reminder.read_short(aloud()),
            "Call the dentist. Due: July 30, 2026 at 9:15 AM"
        );
    }

    #[test]
    fn test_a_full_reminder_reading_carries_the_description_the_column_hides() {
        let mut reminder = reminder();
        reminder.due_datetime = Some("2026-07-30 09:15".to_string());
        reminder.description = Some("Ask about the evening appointment.".to_string());

        assert_eq!(
            reminder.read_full(aloud()),
            "Call the dentist. Not done. Priority: high. \
             Due: July 30, 2026 at 9:15 AM. Ask about the evening appointment."
        );
    }

    #[test]
    fn test_a_timed_event_says_when_it_starts_in_the_short_reading() {
        assert_eq!(
            event().read_short(aloud()),
            "Standup. July 27, 2026 at 9:00 AM"
        );
    }

    /// Every shape a stored moment arrives in is read as a date, not spelled
    /// out.
    ///
    /// Asked here rather than of `date_display::parse` because this reading is
    /// the text handed to the announcement, and a parser checked on its own
    /// says nothing about what that text ends up being. Graph writes seven
    /// digits of fraction and this program's own editor writes a `T` and
    /// seconds, and the reading for both was the stored string unchanged.
    ///
    /// The two that carry their own offset are checked by shape rather than by
    /// wording, because the hour they land on is this computer's and the test
    /// has to read the same on every machine it runs on.
    #[test]
    fn test_every_shape_a_stored_moment_takes_is_read_as_a_date() {
        let read = |start: &str| {
            let mut moved = event();
            moved.start = start.to_string();
            moved.end = String::new();
            moved.read_short(aloud())
        };

        for zoneless in [
            "2026-07-27T10:30:00",
            "2026-07-27T10:30:00.0000000",
            "2026-07-27T10:30",
            "2026-07-27 10:30:00",
            "2026-07-27 10:30:00.0000000",
            "2026-07-27 10:30",
        ] {
            assert_eq!(
                read(zoneless),
                "Standup. July 27, 2026 at 10:30 AM",
                "stored as {zoneless}"
            );
        }

        for carries_its_own_offset in ["2026-07-27T10:30:00+05:30", "2026-07-27T10:30:00Z"] {
            let said = read(carries_its_own_offset);
            assert!(
                said.contains(", 2026 at ") && !said.contains('T'),
                "stored as {carries_its_own_offset}, read as {said}"
            );
        }
    }

    /// A whole day is a date and no clock reading, and a value nothing here
    /// understands is still said as it stands rather than dropped or panicked
    /// over.
    #[test]
    fn test_a_whole_day_keeps_its_shape_and_an_unreadable_moment_is_still_said() {
        let read = |start: &str| {
            let mut moved = event();
            moved.start = start.to_string();
            moved.end = String::new();
            moved.read_short(aloud())
        };

        assert_eq!(read("2026-07-27"), "Standup. July 27, 2026");
        assert_eq!(read("not a moment"), "Standup. not a moment");
        assert_eq!(read("2026-13-45T99:99:99"), "Standup. 2026-13-45T99:99:99");
    }

    #[test]
    fn test_a_task_due_today_is_a_date_not_an_hour_count_under_the_shipped_default() {
        // "Due: 12 hours ago" at noon is the reading calling a task due today
        // overdue, in the one channel this product is for. A whole day is a
        // date under every style, the rule the birthday reading already keeps.
        let mut task = task();
        task.due_date = Some("2026-07-26".to_string());

        let said = task.read_short(under_the_shipped_default());

        assert!(said.contains("Due: July 26, 2026"), "{said}");
        assert!(!said.contains("ago"), "{said}");
    }

    #[test]
    fn test_an_event_on_a_bare_date_is_a_date_not_an_hour_count_under_the_shipped_default() {
        let mut moved = event();
        moved.start = "2026-07-26".to_string();
        moved.end = String::new();

        assert_eq!(
            moved.read_short(under_the_shipped_default()),
            "Standup. July 26, 2026"
        );
    }

    #[test]
    fn test_a_reminder_due_on_a_day_is_a_date_not_an_hour_count_under_the_shipped_default() {
        let mut reminder = reminder();
        reminder.due_datetime = Some("2026-07-26".to_string());

        let said = reminder.read_short(under_the_shipped_default());

        assert!(said.contains("Due: July 26, 2026"), "{said}");
        assert!(!said.contains("ago"), "{said}");
    }

    #[test]
    fn test_a_note_updated_on_a_bare_date_is_a_date_under_the_shipped_default() {
        let mut note = note();
        note.updated_at = "2026-07-25".to_string();

        let said = note.read_full(under_the_shipped_default());

        assert!(said.contains("Updated: July 25, 2026"), "{said}");
        assert!(!said.contains("ago"), "{said}");
    }

    #[test]
    fn test_a_note_updated_hours_ago_still_reads_relatively_under_the_shipped_default() {
        // The other half of the rule: a stored value that names an hour keeps
        // the relative wording, which is the style doing its job.
        let mut note = note();
        note.updated_at = "2026-07-26 09:00".to_string();

        let said = note.read_full(under_the_shipped_default());

        assert!(said.contains("Updated: 3 hours ago"), "{said}");
    }

    #[test]
    fn test_an_all_day_event_says_so_in_the_short_reading_too() {
        // The short form is what the first Space press gives, so an all day
        // event that reads a start time it does not have is heard there first.
        let mut holiday = event();
        holiday.summary = "Public holiday".to_string();
        holiday.start = "2026-08-31".to_string();
        holiday.end = "2026-08-31".to_string();
        holiday.is_all_day = true;

        assert_eq!(
            holiday.read_short(aloud()),
            "Public holiday. August 31, 2026, all day"
        );
    }

    #[test]
    fn test_an_event_missing_one_of_its_times_does_not_trail_off_on_the_word_to() {
        // "July 27, 2026 at 9:00 AM to" and then silence sounds like the
        // reading was cut off.
        let mut open_ended = event();
        open_ended.end = String::new();

        assert_eq!(
            open_ended.read_full(aloud()),
            "Standup. July 27, 2026 at 9:00 AM"
        );

        let mut no_start = event();
        no_start.start = String::new();

        assert_eq!(
            no_start.read_full(aloud()),
            "Standup. July 27, 2026 at 9:15 AM"
        );
    }

    #[test]
    fn test_only_an_unusual_status_is_said_about_an_event() {
        // "Status: confirmed" on every event in the list carries nothing. A
        // cancelled one is worth the words.
        assert!(!event().read_full(aloud()).contains("Status"));

        let mut called_off = event();
        called_off.status = "cancelled".to_string();
        assert!(called_off.read_full(aloud()).contains("Status: cancelled"));
    }

    #[test]
    fn test_reading_a_note_in_full_reads_the_note_and_not_the_preview() {
        // The preview is one short line for a list column. Reading it back at
        // somebody who asked for the whole note is answering a different
        // question, and there was no other way to hear a note's contents.
        let note = NoteItem {
            id: "n2".to_string(),
            title: "Shopping".to_string(),
            body: "Milk, bread, and the thing for the tap.".to_string(),
            body_preview: "Milk, bread, and the\u{2026}".to_string(),
            pinned: false,
            updated_at: "2026-07-26".to_string(),
            folder_id: None,
        };

        let said = note.read_full(aloud());

        assert!(said.contains("the thing for the tap"), "{said}");
    }

    #[test]
    fn test_a_heading_in_a_note_is_read_as_a_heading() {
        let note = NoteItem {
            id: "n3".to_string(),
            title: "Trip".to_string(),
            body: "# Packing\n\n- Passport\n- Charger".to_string(),
            body_preview: "Packing".to_string(),
            pinned: false,
            updated_at: "2026-07-26".to_string(),
            folder_id: None,
        };

        let said = note.read_full(aloud());

        assert!(said.contains("heading level 1, Packing"), "{said}");
        assert!(said.contains("bullet, Passport"), "{said}");
    }

    #[test]
    fn test_an_event_that_repeats_says_so_when_it_is_read() {
        // Nothing anywhere said an event repeated. Somebody met a meeting on
        // one day with no way of knowing it was the same meeting as last week.
        let event = CalendarEventItem {
            repeats: "every week, 6 times".to_string(),
            ..event()
        };

        let read = event.read_full(aloud());

        assert!(read.contains("every week, 6 times"), "{read}");
    }

    #[test]
    fn test_a_birthday_is_told_from_a_dentist_appointment_by_ear() {
        // The reason categories exist, and the half that was never delivered:
        // telling one kind of day from another by colour tells this
        // application's own audience nothing.
        let birthday = CalendarEventItem {
            summary: "Ada".to_string(),
            categories: "Birthday".to_string(),
            ..event()
        };

        let read = birthday.read_full(aloud());

        assert!(read.contains("Birthday"), "{read}");
        assert!(
            !event().read_full(aloud()).contains("Birthday"),
            "an event with no category must not gain one"
        );
    }

    #[test]
    fn test_an_event_whose_repeat_rule_cannot_be_read_says_so_without_pressing_space() {
        // The one case somebody has to be told without asking. For a series
        // that was worked out, the days themselves are the evidence; for one
        // that was not, there is no evidence at all and the single row on the
        // screen looks exactly like an event that happens once.
        let unreadable = CalendarEventItem {
            repeats: crate::application::occurrences::CANNOT_BE_READ.to_string(),
            ..event()
        };

        let short = unreadable.read_short(aloud());

        assert!(
            short.contains(crate::application::occurrences::CANNOT_BE_READ),
            "{short}"
        );
        // An ordinary weekly meeting stays a line, because the other rows say it.
        let weekly = CalendarEventItem {
            repeats: "every week".to_string(),
            ..event()
        };
        assert!(!weekly.read_short(aloud()).contains("every week"));
    }

    #[test]
    fn test_two_days_of_the_same_series_are_not_the_same_item_to_read() {
        // Every day of a series carries the stored event's identity, so without
        // the day as well all fifty-two Tuesdays are one item: pressing Space
        // on the second would carry on the cycle begun on the first.
        let first = CalendarEventItem {
            repeats: "every week".to_string(),
            ..event()
        };
        let second = CalendarEventItem {
            start: "2026-08-03 09:00".to_string(),
            ..first.clone()
        };

        assert_ne!(first.read_id(), second.read_id());
        // An event that does not repeat is one row and keeps the plain answer.
        assert_eq!(event().read_id(), "e1");
    }

    #[test]
    fn test_an_events_description_is_read_at_all() {
        // Where the dial-in number and the agenda live. Nothing read it.
        let event = CalendarEventItem {
            attendees_json: None,
            id: "e3".to_string(),
            summary: "Review".to_string(),
            description: "Dial in on 555 0123.".to_string(),
            start: "2026-07-27 09:00".to_string(),
            end: "2026-07-27 09:15".to_string(),
            location: String::new(),
            is_all_day: false,
            status: "confirmed".to_string(),
            provider: "local".to_string(),
            calendar_id: None,
            calendar_name: None,
            calendar_color: None,
            reminder_minutes: None,
            repeats: String::new(),
            categories: String::new(),
            show_as: String::new(),
            recurrence_rule: None,
        };

        assert!(
            event.read_full(aloud()).contains("Dial in on 555 0123."),
            "{}",
            event.read_full(aloud())
        );
    }

    #[test]
    fn test_an_all_day_event_says_so_rather_than_reading_two_identical_times() {
        let event = CalendarEventItem {
            attendees_json: None,
            id: "e2".to_string(),
            summary: "Public holiday".to_string(),
            description: String::new(),
            start: "2026-08-31".to_string(),
            end: "2026-08-31".to_string(),
            location: String::new(),
            is_all_day: true,
            status: "confirmed".to_string(),
            provider: "local".to_string(),
            calendar_id: None,
            calendar_name: None,
            calendar_color: None,
            reminder_minutes: None,
            repeats: String::new(),
            categories: String::new(),
            show_as: String::new(),
            recurrence_rule: None,
        };
        assert!(event.read_full(aloud()).contains("all day"));
        assert!(
            !event
                .read_full(aloud())
                .contains("2026-08-31 to 2026-08-31")
        );
    }
}

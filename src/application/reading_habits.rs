//! Choices about reading that had nowhere to be made.
//!
//! Each of these was either a control in the settings window wired to nothing,
//! or a decision this application had taken on somebody's behalf without
//! offering them a say. They are together because they are the same kind of
//! thing and because a stored value is worth nothing until something reads it.

/// How long a message is looked at before it counts as read.
///
/// The settings window has offered four of these since it was written and
/// none of them was ever read back: the control was built, given a fixed
/// selection, and left out of the list of things saved. So the answer was
/// always "immediately", whatever it said on screen.
///
/// Immediately is the wrong default for somebody working by ear. Arrowing down
/// a list to find something reads every message on the way, and with an instant
/// mark each one of those is marked read, so the unread count empties itself
/// and the one message that mattered is no longer findable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkRead {
    /// The moment it is opened.
    Immediately,
    /// After this many seconds with it open.
    After(u32),
    /// Only when somebody says so.
    Never,
}

impl MarkRead {
    /// The choices offered, in the order they are offered.
    pub const ALL: [MarkRead; 7] = [
        MarkRead::Immediately,
        MarkRead::After(2),
        MarkRead::After(5),
        MarkRead::After(10),
        MarkRead::After(30),
        MarkRead::After(60),
        MarkRead::Never,
    ];

    /// What the choice is called.
    pub fn label(self) -> String {
        match self {
            MarkRead::Immediately => "Immediately".to_string(),
            MarkRead::After(1) => "After 1 second".to_string(),
            MarkRead::After(60) => "After a minute".to_string(),
            MarkRead::After(seconds) => format!("After {seconds} seconds"),
            MarkRead::Never => "Only when I say so".to_string(),
        }
    }

    /// How it is written in the settings file.
    pub fn as_stored(self) -> String {
        match self {
            MarkRead::Immediately => "immediately".to_string(),
            MarkRead::After(seconds) => seconds.to_string(),
            MarkRead::Never => "never".to_string(),
        }
    }

    /// Read the stored preference.
    ///
    /// Anything unrecognised is the default rather than a refusal, because a
    /// settings file somebody has edited by hand should not stop mail working.
    pub fn from_setting(value: &str) -> Self {
        match value.trim() {
            "immediately" => MarkRead::Immediately,
            "never" => MarkRead::Never,
            other => other
                .parse::<u32>()
                .ok()
                .filter(|seconds| *seconds > 0)
                .map(MarkRead::After)
                .unwrap_or_default(),
        }
    }

    /// How long to wait, or `None` for one of the two that never wait.
    pub fn delay(self) -> Option<std::time::Duration> {
        match self {
            MarkRead::After(seconds) => Some(std::time::Duration::from_secs(seconds as u64)),
            _ => None,
        }
    }

    /// Whether opening a message should ever mark it read on its own.
    pub fn marks_at_all(self) -> bool {
        self != MarkRead::Never
    }
}

/// Which entry of the offered list a stored choice selects.
///
/// A stored wait the list does not offer falls back to the default rather than
/// to the first entry. The first entry is "Immediately", which is the most
/// aggressive of the seven and the wrong one to fail towards: somebody who
/// asked to wait would open settings, see no wait at all, and save that back
/// without ever having chosen it.
pub fn offered_index(stored: &str) -> usize {
    let wanted = MarkRead::from_setting(stored);
    MarkRead::ALL
        .iter()
        .position(|choice| *choice == wanted)
        .or_else(|| {
            MarkRead::ALL
                .iter()
                .position(|choice| *choice == MarkRead::default())
        })
        .unwrap_or(0)
}

impl Default for MarkRead {
    /// Two seconds, not instantly.
    ///
    /// Long enough that arrowing past a message does not mark it, short enough
    /// that stopping to read one does.
    fn default() -> Self {
        MarkRead::After(2)
    }
}

/// Whether the Cc and Bcc lines are in the compose window from the start.
///
/// Most messages go to one person, and two empty fields between the recipient
/// and the subject are two stops on the way for anybody working by keyboard,
/// every time they write anything. Anybody who copies people on most of what
/// they send wants the opposite, which is why it is a choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CopyLines {
    /// Always there.
    #[default]
    Shown,
    /// Not there until a reply brings addresses that need them, or somebody
    /// asks for them with Alt+C or Alt+B.
    Hidden,
}

impl CopyLines {
    pub fn from_setting(value: &str) -> Self {
        match value.trim() {
            "hidden" => CopyLines::Hidden,
            _ => CopyLines::Shown,
        }
    }

    pub fn as_stored(self) -> &'static str {
        match self {
            CopyLines::Shown => "shown",
            CopyLines::Hidden => "hidden",
        }
    }

    /// Whether to show them for a message that starts with these lines filled.
    ///
    /// A reply to all arrives with people already in Cc. Hiding a field that
    /// has somebody's address in it hides who is being written to, which is
    /// worse than an extra stop on the way to the subject.
    pub fn shows(self, cc: &str, bcc: &str) -> bool {
        self == CopyLines::Shown || !cc.trim().is_empty() || !bcc.trim().is_empty()
    }
}

/// The hours somebody actually works.
///
/// The calendar showed every hour the same way, so nine in the morning and
/// three in the morning read alike, and an event landing outside the working
/// day said nothing about it. That is the case worth catching: a meeting at
/// seven in the evening is a fact somebody wants to notice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkingDay {
    /// The hour it starts, 0 to 23.
    pub starts: u8,
    /// The hour it ends, 1 to 24. Exclusive: 17 means the day ends at five.
    pub ends: u8,
}

impl Default for WorkingDay {
    fn default() -> Self {
        Self {
            starts: 9,
            ends: 17,
        }
    }
}

impl WorkingDay {
    /// Read the stored pair, keeping the default for anything that makes no
    /// sense. A day that ends before it starts is not a day.
    ///
    /// The first `starts < 24` cannot be swapped for `starts <= 24` and be
    /// told apart: the only value that would disagree is `starts == 24`, and
    /// that value only ever reaches `starts < ends` below by needing
    /// `ends >= 25`, while `(1..=24).contains(&ends)` already caps `ends` at
    /// 24. So `sane` is false at `starts == 24` regardless of which of the
    /// two the first clause uses.
    pub fn from_setting(starts: u8, ends: u8) -> Self {
        // Midnight at the end of a day is hour twenty-four, not hour nought.
        // The settings list offers "Midnight, 00" as an ending and it was
        // refused, so choosing it wrote nine to five back with nothing said.
        let ends = match ends {
            0 => 24,
            other => other,
        };
        // A day that runs past midnight is still refused, and that is a real
        // limit rather than an oversight: `note_for` puts an hour outside the
        // day on one side or the other of it, and a day wrapping round the
        // clock has no such sides. The settings screen says so now instead of
        // writing nine to five back without a word, which is what it did.
        let sane = starts < 24 && (1..=24).contains(&ends) && starts < ends;
        if sane {
            Self { starts, ends }
        } else {
            Self::default()
        }
    }

    /// Whether a pair of hours is one this cannot keep.
    ///
    /// Asked by the settings screen so it can say so. It used to write the
    /// built-in day back over an answer it could not use and say nothing at
    /// all, so somebody choosing a night shift set it, was told nothing, and
    /// found nine to five again the next time they looked.
    pub fn could_not_be_used(starts: u8, ends: u8) -> bool {
        Self::from_setting(starts, ends) == Self::default()
            && (starts, ends) != Self::default_pair()
    }

    /// The built-in day, as the pair of hours somebody would have chosen.
    fn default_pair() -> (u8, u8) {
        let day = Self::default();
        (day.starts, day.ends)
    }

    /// Whether an hour is inside the working day.
    pub fn holds(self, hour: u8) -> bool {
        hour >= self.starts && hour < self.ends
    }

    /// What is added when an event falls outside it.
    ///
    /// Empty inside the working day, so most rows in most calendars cost
    /// nothing to hear. Words rather than a colour, because a colour is not
    /// available to the person this is for.
    pub fn note_for(self, hour: u8) -> &'static str {
        if self.holds(hour) {
            ""
        } else if hour < self.starts {
            "before the working day"
        } else {
            "after the working day"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_stored_mark_read_choice_survives_the_trip() {
        for choice in MarkRead::ALL {
            assert_eq!(MarkRead::from_setting(&choice.as_stored()), choice);
        }
    }

    #[test]
    fn test_the_stored_copy_lines_choice_survives_the_trip() {
        // Anything other than the word this writes reads back as shown, so a
        // wrong stored word reverts the choice at the next start with nothing
        // said anywhere.
        for choice in [CopyLines::Shown, CopyLines::Hidden] {
            assert_eq!(CopyLines::from_setting(choice.as_stored()), choice);
        }
    }

    #[test]
    fn test_an_unreadable_setting_falls_to_the_default_rather_than_refusing() {
        // A settings file somebody has edited by hand should not stop mail
        // from working.
        for nonsense in ["", "   ", "soon", "-4", "0"] {
            assert_eq!(MarkRead::from_setting(nonsense), MarkRead::default());
        }
    }

    #[test]
    fn test_a_stored_choice_the_list_does_not_offer_falls_back_to_the_safe_default() {
        // A settings file can hold a wait the list does not offer. Landing on
        // the first entry means somebody who asked to wait sees "Immediately",
        // and saving writes that back, so every message they arrow past is
        // marked read and the unread count empties itself.
        assert_eq!(MarkRead::ALL[offered_index("1")], MarkRead::default());
        assert_ne!(MarkRead::ALL[offered_index("1")], MarkRead::Immediately);
    }

    #[test]
    fn test_a_stored_choice_the_list_does_offer_is_the_one_selected() {
        assert_eq!(MarkRead::ALL[offered_index("30")], MarkRead::After(30));
        assert_eq!(MarkRead::ALL[offered_index("never")], MarkRead::Never);
        assert_eq!(
            MarkRead::ALL[offered_index("immediately")],
            MarkRead::Immediately
        );
    }

    #[test]
    fn test_the_default_waits_a_moment() {
        // Arrowing down a list reads every message on the way. Marking each
        // one read empties the unread count and loses the one that mattered.
        assert_eq!(MarkRead::default(), MarkRead::After(2));
        assert!(MarkRead::default().delay().is_some());
    }

    #[test]
    fn test_the_two_that_never_wait_have_no_delay() {
        assert_eq!(MarkRead::Immediately.delay(), None);
        assert_eq!(MarkRead::Never.delay(), None);
        assert!(MarkRead::Immediately.marks_at_all());
        assert!(!MarkRead::Never.marks_at_all());
    }

    #[test]
    fn test_every_choice_reads_as_a_phrase() {
        let said: Vec<String> = MarkRead::ALL.iter().map(|c| c.label()).collect();

        assert_eq!(
            said,
            [
                "Immediately",
                "After 2 seconds",
                "After 5 seconds",
                "After 10 seconds",
                "After 30 seconds",
                "After a minute",
                "Only when I say so",
            ]
        );
    }

    #[test]
    fn test_the_copy_lines_can_be_put_away() {
        assert_eq!(CopyLines::from_setting("hidden"), CopyLines::Hidden);
        assert_eq!(CopyLines::from_setting("shown"), CopyLines::Shown);
        assert_eq!(CopyLines::from_setting("nonsense"), CopyLines::Shown);
        assert!(!CopyLines::Hidden.shows("", ""));
        assert!(CopyLines::Shown.shows("", ""));
    }

    #[test]
    fn test_a_reply_that_already_copies_people_shows_the_line_anyway() {
        // Hiding a field with somebody's address in it hides who is being
        // written to, which is worse than an extra stop on the way.
        assert!(CopyLines::Hidden.shows("ada@example.com", ""));
        assert!(CopyLines::Hidden.shows("", "grace@example.com"));
    }

    #[test]
    fn test_the_working_day_is_nine_to_five_unless_told_otherwise() {
        let day = WorkingDay::default();

        assert!(day.holds(9));
        assert!(day.holds(16));
        assert!(!day.holds(8));
        assert!(!day.holds(17), "the day ends at five, so five is outside");
    }

    #[test]
    fn test_a_day_that_ends_before_it_starts_is_not_a_day() {
        assert_eq!(WorkingDay::from_setting(18, 9), WorkingDay::default());
        assert_eq!(WorkingDay::from_setting(9, 9), WorkingDay::default());
        assert_eq!(WorkingDay::from_setting(25, 30), WorkingDay::default());
        assert_eq!(
            WorkingDay::from_setting(7, 15),
            WorkingDay {
                starts: 7,
                ends: 15
            }
        );
    }

    #[test]
    fn test_an_hour_outside_the_day_says_which_side_it_is_on() {
        let day = WorkingDay::default();

        assert_eq!(day.note_for(7), "before the working day");
        assert_eq!(day.note_for(19), "after the working day");
    }

    #[test]
    fn test_an_ordinary_hour_costs_nothing_to_hear() {
        // Most rows in most calendars are inside the working day, and a word
        // on every one of them is a word paid for on all of them.
        assert_eq!(WorkingDay::default().note_for(11), "");
    }

    #[test]
    fn test_an_hour_equal_to_an_impossible_start_still_reads_as_after() {
        // `starts` and `ends` are public fields, so a pair `from_setting`
        // would refuse (start at or after end) can still be built directly.
        // `holds` already says such a day holds nothing; this pins down
        // which side of the day `note_for` puts the start hour itself on.
        let broken = WorkingDay {
            starts: 10,
            ends: 5,
        };
        assert!(!broken.holds(10));
        assert_eq!(broken.note_for(10), "after the working day");
    }

    #[test]
    fn test_a_night_shift_can_be_described() {
        // Not everybody works nine to five, and the setting should not decide
        // that for them.
        let night = WorkingDay::from_setting(22, 24);

        assert!(night.holds(23));
        assert!(!night.holds(9));
        assert_eq!(night.note_for(9), "before the working day");
    }
}

#[cfg(test)]
mod a_working_day_that_is_not_nine_to_five {
    use super::*;

    #[test]
    fn test_a_day_that_ends_at_midnight_is_kept() {
        // "Midnight, 00" is offered in the settings list as the end of the
        // day, and it was refused and quietly written back as nine to five.
        // Midnight is the end of the day rather than the start of it here,
        // which is what an end hour means everywhere else in this type.
        let day = WorkingDay::from_setting(9, 0);

        assert_eq!(day.starts, 9);
        assert_eq!(day.ends, 24, "midnight at the end of a day is hour 24");
        assert!(
            day.holds(23),
            "eleven at night is inside a day ending at midnight"
        );
    }

    #[test]
    fn test_a_day_running_past_midnight_is_still_refused_and_says_which() {
        // Ten at night until six in the morning is a real working day and is
        // not supported: an hour outside the day is described as before it or
        // after it, and a day wrapping round the clock has no such sides.
        // That is a limit worth keeping honest rather than half-supporting.
        //
        // What was wrong was the silence. The settings screen wrote nine to
        // five back over it with nothing said, so the answer was thrown away
        // every time it was set. `could_not_be_used` is what the screen asks
        // so it can say so.
        assert_eq!(WorkingDay::from_setting(22, 6), WorkingDay::default());
        assert!(WorkingDay::could_not_be_used(22, 6));
        assert!(!WorkingDay::could_not_be_used(9, 17));
        assert!(
            !WorkingDay::could_not_be_used(9, 0),
            "a day ending at midnight is used, so nothing is refused"
        );
    }

    #[test]
    fn test_an_answer_that_says_nothing_is_still_refused() {
        // The same hour twice is not a day, it is an instant, and there is no
        // reading of it that holds any hour at all.
        assert_eq!(WorkingDay::from_setting(9, 9), WorkingDay::default());
        assert_eq!(WorkingDay::from_setting(25, 30), WorkingDay::default());
    }
}

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

/// The three ways Mark as read after is offered: at once, after a number of
/// seconds a spin control beside the choice holds, or never.
///
/// Until 12-06 the choice was seven fixed answers, five of them waits. The
/// tester asked for numbers to be spin controls (#35), and a wait is a
/// number, so the choice says which kind of answer and the spin control says
/// how long. The stored value keeps its shape either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkReadWay {
    Immediately,
    AfterSeconds,
    Never,
}

impl MarkReadWay {
    /// The entries of the choice, in the order they are offered.
    pub const ALL: [MarkReadWay; 3] = [
        MarkReadWay::Immediately,
        MarkReadWay::AfterSeconds,
        MarkReadWay::Never,
    ];

    /// What the entry is called.
    pub fn label(self) -> &'static str {
        match self {
            MarkReadWay::Immediately => "Immediately",
            MarkReadWay::AfterSeconds => "After a number of seconds",
            MarkReadWay::Never => "Never",
        }
    }
}

impl MarkRead {
    /// The wait the default holds, and the one the seconds spin control
    /// offers when the answer stored was not a wait.
    pub const DEFAULT_WAIT_SECONDS: u32 = 2;

    /// The longest wait the seconds spin control holds.
    pub const LONGEST_WAIT_SECONDS: u32 = 600;

    /// The entry of the choice and the seconds in the spin control that show
    /// this answer.
    ///
    /// A stored wait longer than the spin control holds shows as the longest
    /// it holds, still a wait. Showing it as anything else, and "Immediately"
    /// above all, would have somebody who asked to wait open Settings, see no
    /// wait, and save that back without ever having chosen it.
    pub fn parts(self) -> (MarkReadWay, u32) {
        match self {
            MarkRead::Immediately => (MarkReadWay::Immediately, Self::DEFAULT_WAIT_SECONDS),
            MarkRead::After(seconds) => (MarkReadWay::AfterSeconds, Self::a_wait_offered(seconds)),
            MarkRead::Never => (MarkReadWay::Never, Self::DEFAULT_WAIT_SECONDS),
        }
    }

    /// The answer the choice and the spin control give together.
    pub fn from_parts(way: MarkReadWay, seconds: u32) -> Self {
        match way {
            MarkReadWay::Immediately => MarkRead::Immediately,
            MarkReadWay::AfterSeconds => MarkRead::After(Self::a_wait_offered(seconds)),
            MarkReadWay::Never => MarkRead::Never,
        }
    }

    fn a_wait_offered(seconds: u32) -> u32 {
        seconds.clamp(1, Self::LONGEST_WAIT_SECONDS)
    }
}

/// Which entry of the choice a stored answer selects.
pub fn offered_index(stored: &str) -> usize {
    let (way, _) = MarkRead::from_setting(stored).parts();
    MarkReadWay::ALL
        .iter()
        .position(|offered| *offered == way)
        .unwrap_or(0)
}

impl Default for MarkRead {
    /// Two seconds, not instantly.
    ///
    /// Counted from the moment the whole message is read aloud or opened,
    /// never from the moment it is selected (#25, 2026-09-18). Until then the
    /// two seconds were counted from selection, on the reasoning that
    /// arrowing past a message takes less than that; hearing a row's sender,
    /// subject and date takes longer, so a walk through a folder by ear
    /// marked every message stopped on. And not from the first Space either:
    /// that press reads subject, sender and snippet, and from the build of
    /// 2026-09-18 until 11-05.1 the same day it started the clock, until #25
    /// was reopened on the tester's word that reading the snippet is not
    /// reading. Counted from the whole reading, two seconds is long enough
    /// that a Space pressed by mistake and left at once does not mark the
    /// message, and short enough that hearing it through does.
    fn default() -> Self {
        MarkRead::After(Self::DEFAULT_WAIT_SECONDS)
    }
}

/// The sentence under the Mark as read after choice in Settings: what the
/// wait is counted from.
///
/// The choice and its seconds cannot say on their own when the counting starts, and until
/// 2026-09-18 it started when a row was selected (#25). Said where somebody
/// meets the choice, on both channels, so a person who has set a wait knows
/// that moving through the list is not what the wait is measured from, and
/// that the first Space, which reads the row a little more fully, is not
/// either: that press started the clock from the build of 2026-09-18 until
/// 11-05.1, when #25 was reopened on the tester's word.
pub const WHAT_MARK_READ_COUNTS_FROM: &str = "Counted from when you read the whole message aloud, \
     with Space twice or Shift+Space, or open it; never from the first Space or from moving \
     onto it.";

/// The sentence under a setting that cannot follow a save: the log level,
/// which is set up once when the program starts and has no way to be
/// changed while it runs.
///
/// Every other setting applies the moment Settings is saved, and since
/// 2026-09-20 (#91) a reading holds that: a setting read once at startup
/// either follows a save or carries this sentence on its control, so a
/// person changing it is told rather than left to find out.
pub const TAKES_EFFECT_AT_THE_NEXT_START: &str = "Takes effect the next time Wixen Mail starts.";

/// The sentence under the default sort order, which applies at the next
/// start and only where no layout was saved: a folder whose columns were
/// arranged with `F8` keeps the sort that arrangement carries, which is the
/// more recent answer.
pub const WHERE_THE_DEFAULT_SORT_ORDER_APPLIES: &str = "Takes effect the next time Wixen Mail \
     starts, and only in folders whose columns you have not arranged.";

/// Whether the message somebody began reading is to be marked read now.
///
/// The clock starts when the whole message is read aloud from the list or
/// opened in its own window, and never when it is selected (#25, decided
/// 2026-09-18): selecting a row is how somebody moves through a folder, and
/// for somebody working by ear hearing the row takes longer than any short
/// wait, so a clock started by selection marked every message stopped on.
/// Nor when the first Space reads the short form, subject, sender and
/// snippet: from the build of 2026-09-18 until 11-05.1 that press started
/// the clock too, and #25 was reopened on the tester's word that reading the
/// snippet is not reading; which press counts is
/// `presentation::read_aloud::what_a_press_starts`, asked where the depth is
/// known, and this rule only sees what was written. `began` is which
/// message was read and when; `selected_unread` is the message under the
/// cursor if it is still unread. The answer is the message to mark, or
/// nothing.
///
/// Nothing began, nothing is marked, whatever is selected and however long
/// it has been. The message that began reading must still be the selected
/// unread one: moving on to another row before the wait has run leaves the
/// first unread and starts nothing for the second. Then the setting: at once
/// under Immediately, once the wait has run under a wait, never under Only
/// when I say so.
pub fn whether_to_mark_read(
    began: Option<(i64, std::time::Instant)>,
    selected_unread: Option<i64>,
    now: std::time::Instant,
    setting: MarkRead,
) -> Option<i64> {
    let (message, since) = began.filter(|(message, _)| Some(*message) == selected_unread)?;
    if !setting.marks_at_all() {
        return None;
    }
    match setting.delay() {
        None => Some(message),
        Some(wait) => (now.saturating_duration_since(since) >= wait).then_some(message),
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
        // Through the settings file, and through the choice and the seconds
        // spin control that show it, so the stored string keeps its shape:
        // `immediately`, `never`, or the number.
        for choice in [
            MarkRead::Immediately,
            MarkRead::After(1),
            MarkRead::After(2),
            MarkRead::After(45),
            MarkRead::After(600),
            MarkRead::Never,
        ] {
            assert_eq!(MarkRead::from_setting(&choice.as_stored()), choice);
            let (way, seconds) = choice.parts();
            assert_eq!(MarkRead::from_parts(way, seconds), choice, "{choice:?}");
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
    fn test_a_stored_wait_longer_than_the_spin_control_holds_shows_the_longest_wait() {
        // A settings file can hold a wait the seconds spin control does not
        // reach. Showing it as "Immediately" would have somebody who asked to
        // wait save that back without choosing it, and every message they
        // arrow past would be marked read. It shows as the longest wait.
        assert_eq!(
            MarkRead::from_setting("5000").parts(),
            (MarkReadWay::AfterSeconds, 600)
        );
    }

    #[test]
    fn test_a_stored_choice_the_list_does_offer_is_the_one_selected() {
        assert_eq!(
            MarkReadWay::ALL.get(offered_index("30")),
            Some(&MarkReadWay::AfterSeconds)
        );
        assert_eq!(
            MarkReadWay::ALL.get(offered_index("never")),
            Some(&MarkReadWay::Never)
        );
        assert_eq!(
            MarkReadWay::ALL.get(offered_index("immediately")),
            Some(&MarkReadWay::Immediately)
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
        // Choosing a wait after either of these offers the default wait.
        assert_eq!(MarkRead::Never.parts(), (MarkReadWay::Never, 2));
        assert_eq!(MarkRead::Immediately.parts(), (MarkReadWay::Immediately, 2));
    }

    #[test]
    fn test_every_choice_reads_as_a_phrase() {
        let said: Vec<&str> = MarkReadWay::ALL.iter().map(|way| way.label()).collect();

        assert_eq!(said, ["Immediately", "After a number of seconds", "Never"]);
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
mod a_message_is_marked_read_after_it_was_read_and_never_after_it_was_selected {
    //! #25, 2026-09-15, the tester under NVDA: "Automatic read/unread status
    //! should not be linked to the list traversal for mail. It should be
    //! either when a message is previewed or when a message is opened."
    //! Hearing a row's sender, subject and date takes longer than two
    //! seconds, so a clock started by selection marked every message stopped
    //! on. The clock starts when a message is read aloud or opened now, and
    //! these cases hold the rule that turns that clock into a mark.

    use super::*;
    use std::time::{Duration, Instant};

    const THE_MESSAGE: i64 = 41;
    const ANOTHER: i64 = 42;

    #[test]
    fn test_nothing_began_means_nothing_is_marked_however_long_a_row_is_selected() {
        // Selecting a row starts nothing, so a row selected for an hour with
        // no reading begun is still unread. This is the whole of #25.
        let long_ago = Instant::now();
        let an_hour_on = long_ago + Duration::from_secs(3600);
        for setting in [
            MarkRead::Immediately,
            MarkRead::After(1),
            MarkRead::After(2),
            MarkRead::After(60),
            MarkRead::After(MarkRead::LONGEST_WAIT_SECONDS),
            MarkRead::Never,
        ] {
            assert_eq!(
                whether_to_mark_read(None, Some(THE_MESSAGE), an_hour_on, setting),
                None,
                "{setting:?} marked a message nobody read"
            );
        }
    }

    #[test]
    fn test_a_message_that_began_reading_is_not_marked_once_another_is_selected() {
        // The clock belongs to the message that was read; moving on to
        // another row before it runs out leaves the first unread and starts
        // nothing for the second.
        let began = Instant::now();
        let later = began + Duration::from_secs(60);
        assert_eq!(
            whether_to_mark_read(
                Some((THE_MESSAGE, began)),
                Some(ANOTHER),
                later,
                MarkRead::Immediately
            ),
            None
        );
        assert_eq!(
            whether_to_mark_read(Some((THE_MESSAGE, began)), None, later, MarkRead::After(2)),
            None,
            "a message that is no longer selected, or is read already, is not marked"
        );
    }

    #[test]
    fn test_immediately_marks_the_message_the_moment_reading_began() {
        let began = Instant::now();
        assert_eq!(
            whether_to_mark_read(
                Some((THE_MESSAGE, began)),
                Some(THE_MESSAGE),
                began,
                MarkRead::Immediately
            ),
            Some(THE_MESSAGE)
        );
    }

    #[test]
    fn test_a_wait_marks_nothing_before_it_has_run() {
        let began = Instant::now();
        let not_yet = began + Duration::from_millis(1999);
        assert_eq!(
            whether_to_mark_read(
                Some((THE_MESSAGE, began)),
                Some(THE_MESSAGE),
                not_yet,
                MarkRead::After(2)
            ),
            None
        );
    }

    #[test]
    fn test_a_wait_marks_the_message_once_it_has_run() {
        let began = Instant::now();
        let just = began + Duration::from_secs(2);
        let well_past = began + Duration::from_secs(90);
        for now in [just, well_past] {
            assert_eq!(
                whether_to_mark_read(
                    Some((THE_MESSAGE, began)),
                    Some(THE_MESSAGE),
                    now,
                    MarkRead::After(2)
                ),
                Some(THE_MESSAGE)
            );
        }
    }

    #[test]
    fn test_only_when_i_say_so_marks_nothing_however_long_ago_reading_began() {
        let began = Instant::now();
        let a_day_on = began + Duration::from_secs(86_400);
        assert_eq!(
            whether_to_mark_read(
                Some((THE_MESSAGE, began)),
                Some(THE_MESSAGE),
                a_day_on,
                MarkRead::Never
            ),
            None
        );
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

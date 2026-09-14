//! The window that opens when something comes due.
//!
//! Reminders were stored, listed, synced and never once went off, so setting
//! one bought nothing over writing a note with a date on it. Since 2026-09-14
//! a task with a due date and a calendar event with an alert come here too,
//! one row each, and each row says what it is before what it is about. The
//! module keeps its name: it is fingerprinted by 06-05's guard records and
//! Pratik calls this the reminder window.
//!
//! Three channels, because one is not enough for anybody: the window itself for
//! anyone looking, the announcement for anyone listening, and a sound for the
//! moment it arrives, which is the part that reaches somebody who is not
//! looking at this application at all. The sound is switchable like every other
//! one, and the window is not, because the window is the thing.
//!
//! What the window offers depends on the kind of the row that is selected. A
//! task and a reminder can be marked done and an event cannot; an event can be
//! opened in its editor to move its time and a task or a reminder cannot yet,
//! because nothing in this program edits an existing one. Where a button is
//! unavailable its own label says why, on the account window's pattern: a
//! greyed button says "unavailable" on every channel and "why" on none, and
//! Windows skips a disabled control in the tab order.

use std::cell::RefCell;
use std::rc::Rc;

use crate::application::due::{Due, Identity, Kind, Snooze};
use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::names::set_accessible_name;
use crate::presentation::theme;
use wxdragon::prelude::*;

/// What was decided about a row that came due.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    /// Leave it alone. It stays due and will not be raised again this session.
    Dismissed,
    /// Come back in this long.
    Snoozed(Snooze),
    /// It is finished. Only for a kind that can be done.
    Done,
    /// Opened in its own editor and no longer due when the editor closed:
    /// moved to another day, or deleted. The editor wrote whatever it wrote;
    /// nothing is written for this answer, and the row is not remembered as
    /// raised, so the thing comes back if its new time arrives this session.
    Edited,
}

/// The editors the Details button can open, one per kind that has one.
///
/// A trait rather than a closure so that which kinds have an editor and the
/// opening are one value, and so a test can hand the window a fake that
/// opens nothing. The window asks `has_one_for` on every selection change
/// to label the button, and calls `open` nested under its own dialog.
pub trait Editors {
    /// Whether this kind has an editor to open.
    fn has_one_for(&self, kind: Kind) -> bool;
    /// Open the row in its editor and answer with the row as it now stands:
    /// `Some` with its fresh sentence when it is still due, `None` when it is
    /// no longer due and the row should go.
    fn open(&mut self, parent: &Dialog, row: &Due) -> Option<Due>;
}

/// No editor for anything, so every Details button is unavailable and says
/// so. What the theme check and the scan fixture hand the window.
pub struct NoEditors;

impl Editors for NoEditors {
    fn has_one_for(&self, _kind: Kind) -> bool {
        false
    }

    fn open(&mut self, _parent: &Dialog, _row: &Due) -> Option<Due> {
        None
    }
}

/// The event this window sounds when a row comes due.
///
/// Pulled out of `raise` so a test can read it. `raise` needs a display and a
/// running application, so which event it asks for was a decision nothing
/// could check, and it was wrong: it asked for the one that means new mail.
/// Since 2026-09-14 it is the event for every kind, and its doc says so.
pub(crate) const ALERT_EVENT: crate::presentation::accessibility::feedback::Event =
    crate::presentation::accessibility::feedback::Event::Reminder;

const ID_SNOOZE: Id = ID_HIGHEST + 401;
const ID_DONE: Id = ID_HIGHEST + 402;
const ID_DISMISS: Id = ID_HIGHEST + 403;
const ID_SNOOZE_ALL: Id = ID_HIGHEST + 404;
const ID_DISMISS_ALL: Id = ID_HIGHEST + 405;
const ID_DETAILS: Id = ID_HIGHEST + 406;
const ID_LIST: Id = ID_HIGHEST + 407;
/// What the dialog ends with when the last row has been answered, which is
/// neither a button nor the close box.
const ID_EVERY_ROW_ANSWERED: Id = ID_HIGHEST + 408;

/// The title, read first by a screen reader. Not "Reminders", which is the
/// name of a module this window is not.
pub const TITLE: &str = "Due now";

/// The buttons' labels, with their Alt keys in them, so each is named on both
/// accessibility channels by the label itself. Six distinct keys: S, A, D, I,
/// L, T.
pub const SNOOZE: &str = "&Snooze";
pub const SNOOZE_ALL: &str = "Snooze &all";
pub const MARK_DONE: &str = "Mark &Done";
pub const DISMISS: &str = "D&ismiss";
pub const DISMISS_ALL: &str = "Dismiss a&ll";
pub const DETAILS: &str = "De&tails";

/// Every label a button here can carry when it is available, for the check
/// that their Alt keys are distinct.
pub const EVERY_BUTTON: [&str; 6] = [SNOOZE, SNOOZE_ALL, MARK_DONE, DISMISS, DISMISS_ALL, DETAILS];

/// The Alt key a label names: the letter after its ampersand, lower case.
pub fn alt_key_of(label: &str) -> Option<char> {
    label
        .split('&')
        .nth(1)?
        .chars()
        .next()
        .map(|key| key.to_ascii_lowercase())
}

/// How many rows the moment's sentence reads out before it counts the rest.
///
/// Three, because the sentence is said on a channel nobody can scroll: the
/// window that follows lists every row and can be arrowed through, so what
/// the sentence has to do is say how many and what the first few are.
pub const MOST_ROWS_SAID: usize = 3;

/// What is said at the moment for these rows.
///
/// One row is that row's sentence, exactly as it always was. Several rows
/// are a count first, then each row's sentence up to [`MOST_ROWS_SAID`], then
/// how many more there are. The count comes first because it changes what
/// the rest means: three things due is a different morning from one.
pub fn sentence_for(
    rows: &[Due],
    now: chrono::DateTime<chrono::Local>,
    dates: crate::presentation::date_display::DateSettings,
) -> String {
    let _ = (rows, now, dates);
    todo!("task 3 green")
}

/// What `say` produced: the sentence, and whether the tone really sounded.
///
/// The tone is reported rather than assumed, for the reason `earcon` gives:
/// a caller that assumes is a caller that reports a sound nobody made. The
/// sentence is returned so the window that follows can carry the same words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Said {
    pub sentence: String,
    pub tone_sounded: bool,
}

/// Say what has come due and sound the tone, without the window.
///
/// This exists because the window is this event's written equivalent, so
/// rows whose window is held back have to say something or the event goes
/// out as nothing at all: no tone, no sentence, no window. Pulled out of
/// `raise` so what is found due while somebody is typing can be said at that
/// look and have its window a look later.
///
/// Said once. Rows are said here at the moment they are found due, whether
/// or not their window can open then. When the window opens after a hold
/// they are not said again: the sentence is the list's own text, and a
/// dialog's text is what a screen reader reads when focus arrives in it.
/// Two announcements a minute apart would be the same fact twice to somebody
/// who cannot skim.
pub fn say(
    rows: &[Due],
    now: chrono::DateTime<chrono::Local>,
    dates: crate::presentation::date_display::DateSettings,
    a11y: &Accessibility,
) -> Said {
    // One sentence, said here and later put on the window, so the channels
    // cannot say different things.
    let sentence = sentence_for(rows, now, dates);
    let tone_sounded = a11y.earcon(ALERT_EVENT).unwrap_or(false);
    let _ = a11y.announce(
        &sentence,
        crate::presentation::accessibility::announcements::Priority::Urgent,
    );
    Said {
        sentence,
        tone_sounded,
    }
}

/// How long between one tone and the next while the window waits for focus.
///
/// A minute, because that is the unit the whole feature counts in: a reminder
/// is set to the minute and the look that finds one runs once a minute.
pub const BETWEEN_TONES: std::time::Duration = std::time::Duration::from_secs(60);

/// How many tones the window sounds before it falls silent with the window
/// still on screen.
///
/// Feedback must be bounded. A tone every minute until somebody comes back
/// from lunch is a flood in an empty room. Ten minutes is long enough to come
/// back from the kettle and short enough that a machine left running does not
/// sound all afternoon; after the tenth the window is still there with its
/// rows on it, and nothing is lost.
pub const MOST_TONES: u32 = 10;

/// What the repeat rule answers when asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToneNow {
    /// Sound the tone now.
    Sound,
    /// Not yet; ask again later.
    Wait,
    /// Never again for this window: focus has reached it, or the ceiling has.
    Finished,
}

/// The tone that comes back until focus reaches the window.
///
/// Pure: it is handed instants and the focus answer rather than reading a
/// clock or a window, so the ceiling and the latch can be asserted exactly.
/// The first time focus reaches the window the tone stops for good, and it
/// does not resume if focus leaves again, because somebody who has heard it
/// and gone back to what they were doing has chosen to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepeatingTone {
    last_sounded: std::time::Instant,
    sounded: u32,
    focus_has_arrived: bool,
}

impl RepeatingTone {
    /// Start counting from the moment the window opened. The first tone was
    /// the rows' own, sounded by `say`; these are the ones after it.
    pub fn from_the_window_opening(at: std::time::Instant) -> Self {
        Self {
            last_sounded: at,
            sounded: 0,
            focus_has_arrived: false,
        }
    }

    /// Whether the tone sounds now, waits, or is finished for good.
    pub fn asked(&mut self, now: std::time::Instant, window_has_focus: bool) -> ToneNow {
        // Latched, not read: focus leaving again does not unlatch it.
        self.focus_has_arrived |= window_has_focus;
        if self.focus_has_arrived || self.sounded >= MOST_TONES {
            return ToneNow::Finished;
        }
        if now.duration_since(self.last_sounded) < BETWEEN_TONES {
            return ToneNow::Wait;
        }
        self.last_sounded = now;
        self.sounded += 1;
        ToneNow::Sound
    }
}

/// How often the window asks the repeat rule, in milliseconds.
///
/// Once a second rather than once a minute, and the rule owns the spacing: a
/// timer set to the minute and a rule wanting a full minute would miss each
/// other by a few milliseconds every other tick and sound every two minutes
/// instead. Asking often costs nothing and the rule still answers Wait.
const HOW_OFTEN_TO_ASK_THE_TONE: i32 = 1000;

/// What answering a row came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answered {
    /// The row is answered and no longer listed.
    Taken,
    /// The answer means nothing for that row's kind, so nothing changed:
    /// done on an event.
    NotForThisKind,
    /// No row at that place.
    NoSuchRow,
}

/// The rows on the window and the answers given so far, with no window.
///
/// The bookkeeping the handlers push into, kept apart from the controls so
/// that what is left after each answer and what the window returns can be
/// asserted without a display, the way 06-02 drove the settings panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rows {
    listed: Vec<Due>,
    answered: Vec<(Identity, Answer)>,
}

impl Rows {
    /// The rows, in the order the window lists them.
    pub fn new(listed: Vec<Due>) -> Self {
        Self {
            listed,
            answered: Vec::new(),
        }
    }

    /// What is still on the window.
    pub fn listed(&self) -> &[Due] {
        &self.listed
    }

    /// Whether every row has been answered.
    pub fn is_empty(&self) -> bool {
        self.listed.is_empty()
    }

    /// Answer the row at a place and take it off the window. Done is refused
    /// for a kind that cannot be done and the row stays.
    pub fn answer(&mut self, index: usize, answer: Answer) -> Answered {
        let _ = (index, answer);
        todo!("task 3 green")
    }

    /// Answer every row still listed the same way: snooze all, dismiss all.
    /// Done is not an answer for all of them, because one of them may be an
    /// event, and is refused with nothing changed.
    pub fn answer_every_row(&mut self, answer: Answer) -> Answered {
        let _ = answer;
        todo!("task 3 green")
    }

    /// The row at a place as it stands after its editor closed: replaced when
    /// it is still due, answered [`Answer::Edited`] and taken off when it is
    /// not.
    pub fn now_stands(&mut self, index: usize, row: Option<Due>) -> Answered {
        let _ = (index, row);
        todo!("task 3 green")
    }

    /// The window is closing. Whatever is still listed is dismissed, as
    /// closing the old window with Escape or the title bar always was: the
    /// rows are left as they are and not raised again this session.
    pub fn closing(self) -> Vec<(Identity, Answer)> {
        todo!("task 3 green")
    }
}

/// What a button says and whether it can be pressed, for the selected row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonState {
    pub label: String,
    pub enabled: bool,
}

/// What Mark Done says for a row of this kind.
///
/// Available for a kind that can be done. For an event the label itself
/// says it is not for an event, rather than the same label greyed: a
/// greyed button says "unavailable" on every channel and "why" on none.
pub fn what_mark_done_says(kind: Kind) -> ButtonState {
    let _ = kind;
    todo!("task 3 green")
}

/// What Details says for a row of this kind, given which kinds have an
/// editor to open.
pub fn what_details_says(kind: Kind, has_an_editor: bool) -> ButtonState {
    let _ = (kind, has_an_editor);
    todo!("task 3 green")
}

/// The accessible name of the list: how many, before anything else.
pub fn what_the_list_is_called(rows: usize) -> String {
    let _ = rows;
    todo!("task 3 green")
}

/// The window, built and not shown, with what a caller reads back.
pub struct DueWindow {
    pub dialog: Dialog,
    pub list: ListBox,
    pub snooze_choice: Choice,
    /// The rows and answers the handlers push into, shared with them.
    pub rows: Rc<RefCell<Rows>>,
}

impl DueWindow {
    /// Select a row and relabel the buttons for its kind, as the selection
    /// handler does. Here so a test can drive the real controls through the
    /// same function, since the toolkit offers no way to raise the event.
    pub fn select(&self, index: u32) {
        let _ = index;
        todo!("task 3 green")
    }

    /// The button with this id, for a test or the timer to read.
    pub fn button(&self, id: Id) -> Option<Button> {
        let _ = id;
        todo!("task 3 green")
    }

    /// The six buttons' ids, in tab order.
    pub const BUTTONS: [Id; 6] = [
        ID_SNOOZE,
        ID_SNOOZE_ALL,
        ID_DONE,
        ID_DISMISS,
        ID_DISMISS_ALL,
        ID_DETAILS,
    ];
}

/// Raise everything that came due at one look and wait for an answer to each.
///
/// Modal on purpose. A window that can be left sitting behind the one it
/// interrupted is one somebody will find tomorrow, and the whole point of
/// asking to be told is being told at the time.
///
/// Nothing is said here: the caller says the rows through [`say`] at the look
/// that finds them, whether or not the window can open then, and the
/// sentence is the list's own text when it does. One answer per identity
/// given, and dismissed for anything still listed when the window closes by
/// Escape or the title bar.
///
/// While the window is open, its tone comes back once a minute until focus
/// reaches it, on the rule in [`RepeatingTone`]. That is for somebody in
/// another application when the rows were due: the sentence may have gone
/// unheard there, and the tone is what reaches across. The tone alone; its
/// written equivalent is the window on screen.
pub fn raise(
    parent: &Frame,
    rows: Vec<Due>,
    now: chrono::DateTime<chrono::Local>,
    dates: crate::presentation::date_display::DateSettings,
    a11y: &std::sync::Arc<Accessibility>,
    default_snooze: Snooze,
    editors: Box<dyn Editors>,
) -> Vec<(Identity, Answer)> {
    let _ = (
        parent,
        rows,
        now,
        dates,
        a11y,
        default_snooze,
        editors,
        HOW_OFTEN_TO_ASK_THE_TONE,
    );
    todo!("task 3 green")
}

/// Build the window without showing it.
///
/// Everything `raise` does from its own window onward, split out the same
/// way [`crate::presentation::wx_settings::build_settings_dialog`] splits
/// Settings: a test can build the real dialog, read the rows back, drive
/// the selection and read what each button says, and never call
/// `.show_modal()`, sound an earcon, or make an announcement at all.
///
/// The rows are listed in the order given, each as its own sentence, because
/// the sentence is what the three channels agree on. Returns what the caller
/// reads back after a real `.show_modal()`: the answers, and how long a
/// snooze was chosen to be.
pub fn build_reminder_alert_dialog(
    parent: &Frame,
    rows: Vec<Due>,
    now: chrono::DateTime<chrono::Local>,
    dates: crate::presentation::date_display::DateSettings,
    default_snooze: Snooze,
    palette: Option<theme::Palette>,
    editors: Box<dyn Editors>,
) -> DueWindow {
    let _ = (
        parent,
        rows,
        now,
        dates,
        default_snooze,
        palette,
        editors,
        ID_LIST,
        ID_EVERY_ROW_ANSWERED,
        TITLE,
    );
    let _ = |what: &StaticText, name: &str| set_accessible_name(what, name);
    todo!("task 3 green")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::accessibility::feedback::Event;

    fn row(kind: Kind, id: &str, title: &str, when: &str, late: bool) -> Due {
        Due {
            identity: Identity {
                kind,
                id: id.to_string(),
            },
            title: title.to_string(),
            when: when.to_string(),
            late,
        }
    }

    fn three() -> Vec<Due> {
        vec![
            row(Kind::Task, "t1", "File the report", "2026-09-14", false),
            row(
                Kind::Reminder,
                "r1",
                "Call the bank",
                "2026-09-14T09:00:00",
                false,
            ),
            row(
                Kind::Event,
                "e1|2026-09-14T09:20:00",
                "Standup",
                "2026-09-14T09:20:00",
                false,
            ),
        ]
    }

    fn at(text: &str) -> chrono::DateTime<chrono::Local> {
        crate::common::moment::read(text)
            .and_then(crate::common::moment::Moment::on_this_computer)
            .expect("a real moment")
    }

    fn dates() -> crate::presentation::date_display::DateSettings {
        crate::presentation::date_display::DateSettings::default()
    }

    #[test]
    fn test_the_reminder_window_asks_for_the_reminder_event_not_new_mail() {
        // This pins which event the window ASKS for, and nothing beyond that.
        // No test here plays a tone, hears one, or can tell you whether the
        // two are tellable apart at somebody's speakers. That is a listening
        // pass, not an assertion.
        //
        // The decision is pulled out of `raise` into a constant precisely so a
        // test can read it: `raise` needs a display and a running application,
        // so nothing inside it is reachable from here.
        assert_eq!(ALERT_EVENT, Event::Reminder);
        assert_ne!(ALERT_EVENT.tone(), Event::NewMail.tone());
    }

    #[test]
    fn test_a_reminder_can_be_said_and_sounded_without_its_window() {
        // No display, no dialog, no `raise`. The sentence reaches the bridge
        // and the tone is reported as sounded, which is what a reminder held
        // back while somebody types needs: the event must not go out as
        // nothing at all while its window waits.
        //
        // Proves the words reached the bridge and the player played, not that
        // anybody heard either. Under `WIXEN_NO_AUDIO` the tone goes to a
        // mixer nothing listens to and is still reported as sounded.
        //
        // Sounds are off by default, so the first half switches them on. The
        // fixture as first written assumed the default was on and was red
        // against working code for that reason; the second half is what that
        // taught, that the sentence goes out whatever the sound setting says.
        let a11y = Accessibility::new().expect("accessibility");
        let mut settings = a11y.feedback_settings();
        settings.set_channel_enabled(
            crate::presentation::accessibility::feedback::Channel::Earcon,
            true,
        );
        a11y.set_feedback_settings(settings);
        let item = row(
            Kind::Reminder,
            "r1",
            "Ring the bank",
            "2026-09-14T10:00:00",
            false,
        );
        let now = chrono::Local::now();
        let dates = dates();

        let said = say(std::slice::from_ref(&item), now, dates, &a11y);

        assert_eq!(
            said.sentence,
            item.spoken(now, dates),
            "the sentence said is not the one the window will show"
        );
        assert_eq!(
            a11y.last_announcement().as_deref(),
            Some(said.sentence.as_str()),
            "the sentence never reached the screen reader bridge"
        );
        assert!(said.tone_sounded, "the tone did not sound with sounds on");

        // Sounds off: the tone is reported as not sounded rather than
        // assumed, and the sentence still goes out, because the sentence is
        // the written half and a sound switched off does not switch it off.
        let quiet = Accessibility::new().expect("accessibility");
        let said_quietly = say(std::slice::from_ref(&item), now, dates, &quiet);
        assert!(
            !said_quietly.tone_sounded,
            "a tone was reported with the sound channel off"
        );
        assert_eq!(
            quiet.last_announcement().as_deref(),
            Some(said_quietly.sentence.as_str()),
            "the sentence was held back because the sound was off"
        );
    }

    #[test]
    fn test_one_row_is_said_as_its_own_sentence_exactly_as_before() {
        let one = three().remove(1);
        let now = at("2026-09-14T09:00:30");
        assert_eq!(
            sentence_for(std::slice::from_ref(&one), now, dates()),
            one.spoken(now, dates())
        );
    }

    #[test]
    fn test_several_rows_are_said_as_a_count_then_each_then_how_many_more() {
        // The count first, because it changes what the rest means. Three
        // rows read out in full; five read the ceiling and count the rest,
        // so a machine back from a day asleep does not read twenty sentences
        // on a channel nobody can scroll.
        let now = at("2026-09-14T09:05:00");
        let rows = three();
        let said = sentence_for(&rows, now, dates());
        let each: Vec<String> = rows.iter().map(|row| row.spoken(now, dates())).collect();
        assert_eq!(
            said,
            format!("3 things due. {}. {}. {}.", each[0], each[1], each[2])
        );

        let mut five = three();
        five.push(row(Kind::Task, "t2", "Pay the gas", "2026-09-14", false));
        five.push(row(
            Kind::Task,
            "t3",
            "Book the car in",
            "2026-09-14",
            false,
        ));
        let said = sentence_for(&five, now, dates());
        assert_eq!(
            said,
            format!(
                "5 things due. {}. {}. {}. And 2 more.",
                each[0], each[1], each[2]
            )
        );
        assert_eq!(MOST_ROWS_SAID, 3, "the ceiling the summary records");
    }

    #[test]
    fn test_dismissing_one_row_leaves_the_rest_listed() {
        let mut rows = Rows::new(three());

        assert_eq!(rows.answer(1, Answer::Dismissed), Answered::Taken);

        let left: Vec<&str> = rows
            .listed()
            .iter()
            .map(|r| r.identity.id.as_str())
            .collect();
        assert_eq!(left, vec!["t1", "e1|2026-09-14T09:20:00"]);
        assert_eq!(rows.answer(5, Answer::Dismissed), Answered::NoSuchRow);
    }

    #[test]
    fn test_snooze_all_answers_for_every_row_still_listed_and_not_only_the_selected_one() {
        let mut rows = Rows::new(three());
        assert_eq!(rows.answer(0, Answer::Dismissed), Answered::Taken);

        assert_eq!(
            rows.answer_every_row(Answer::Snoozed(Snooze(30))),
            Answered::Taken
        );

        assert!(rows.is_empty(), "snooze all left rows on the window");
        let mut answers = rows.closing();
        answers.sort_by(|a, b| a.0.id.cmp(&b.0.id));
        assert_eq!(
            answers,
            vec![
                (
                    Identity {
                        kind: Kind::Event,
                        id: "e1|2026-09-14T09:20:00".to_string()
                    },
                    Answer::Snoozed(Snooze(30))
                ),
                (
                    Identity {
                        kind: Kind::Reminder,
                        id: "r1".to_string()
                    },
                    Answer::Snoozed(Snooze(30))
                ),
                (
                    Identity {
                        kind: Kind::Task,
                        id: "t1".to_string()
                    },
                    Answer::Dismissed
                ),
            ]
        );
    }

    #[test]
    fn test_done_is_refused_on_an_event_row_and_taken_on_a_task_row() {
        let mut rows = Rows::new(three());

        assert_eq!(rows.answer(2, Answer::Done), Answered::NotForThisKind);
        assert_eq!(rows.listed().len(), 3, "a refused answer took the row off");
        assert_eq!(rows.answer(0, Answer::Done), Answered::Taken);
        assert_eq!(
            rows.answer_every_row(Answer::Done),
            Answered::NotForThisKind,
            "done for every row would mark an event done"
        );
        assert_eq!(rows.listed().len(), 2);
    }

    #[test]
    fn test_closing_the_window_dismisses_whatever_is_still_listed() {
        let mut rows = Rows::new(three());
        assert_eq!(rows.answer(0, Answer::Snoozed(Snooze(5))), Answered::Taken);

        let answers = rows.closing();

        assert_eq!(answers.len(), 3, "an answer per identity given");
        assert_eq!(answers[0].1, Answer::Snoozed(Snooze(5)));
        assert!(
            answers[1..]
                .iter()
                .all(|(_, answer)| *answer == Answer::Dismissed),
            "a row still listed at close was not dismissed: {answers:?}"
        );
    }

    #[test]
    fn test_a_row_that_is_no_longer_due_after_its_editor_closes_is_answered_edited() {
        let mut rows = Rows::new(three());

        // Still due, renamed: the row stays, with the fresh sentence.
        let renamed = row(
            Kind::Event,
            "e1|2026-09-14T09:20:00",
            "Stand-up",
            "2026-09-14T09:20:00",
            false,
        );
        assert_eq!(rows.now_stands(2, Some(renamed.clone())), Answered::Taken);
        assert_eq!(rows.listed()[2], renamed);

        // Moved to next week: the row goes, and the answer says so rather
        // than dismissing it, so it is not remembered as raised.
        assert_eq!(rows.now_stands(2, None), Answered::Taken);
        assert_eq!(rows.listed().len(), 2);
        assert_eq!(rows.now_stands(7, None), Answered::NoSuchRow);
        let answers = rows.closing();
        assert!(
            answers.contains(&(renamed.identity, Answer::Edited)),
            "{answers:?}"
        );
    }

    #[test]
    fn test_mark_done_and_details_say_why_they_are_unavailable() {
        assert_eq!(
            what_mark_done_says(Kind::Task),
            ButtonState {
                label: MARK_DONE.to_string(),
                enabled: true
            }
        );
        assert_eq!(
            what_mark_done_says(Kind::Event),
            ButtonState {
                label: "Mark &Done: not for an event".to_string(),
                enabled: false
            }
        );
        assert_eq!(
            what_details_says(Kind::Event, true),
            ButtonState {
                label: DETAILS.to_string(),
                enabled: true
            }
        );
        assert_eq!(
            what_details_says(Kind::Task, false),
            ButtonState {
                label: "De&tails: not for a task yet".to_string(),
                enabled: false
            }
        );
        assert_eq!(
            what_details_says(Kind::Reminder, false).label,
            "De&tails: not for a reminder yet"
        );
        // The Alt key stays in the label whatever it says, so the key is
        // announced with the reason.
        for kind in Kind::ALL {
            assert_eq!(alt_key_of(&what_mark_done_says(kind).label), Some('d'));
            assert_eq!(alt_key_of(&what_details_says(kind, false).label), Some('t'));
        }
    }

    #[test]
    fn test_every_button_carries_its_own_alt_key_and_the_list_says_how_many() {
        let keys: std::collections::HashSet<char> = EVERY_BUTTON
            .iter()
            .map(|label| alt_key_of(label).expect("an Alt key in every label"))
            .collect();
        assert_eq!(keys.len(), EVERY_BUTTON.len(), "two buttons share a key");
        assert_eq!(what_the_list_is_called(1), "1 thing due");
        assert_eq!(what_the_list_is_called(3), "3 things due");
        // Read first by a screen reader, and not the name of a module this
        // window is not.
        assert_eq!(TITLE, "Due now");
    }

    fn minutes(n: u64) -> std::time::Duration {
        std::time::Duration::from_secs(60 * n)
    }

    #[test]
    fn test_the_tone_comes_back_once_a_minute_while_focus_has_not_arrived() {
        let opened = std::time::Instant::now();
        let mut tone = RepeatingTone::from_the_window_opening(opened);

        assert_eq!(tone.asked(opened, false), ToneNow::Wait, "sounded at once");
        assert_eq!(
            tone.asked(
                opened + minutes(1) - std::time::Duration::from_secs(1),
                false
            ),
            ToneNow::Wait,
            "sounded before a minute had passed"
        );
        assert_eq!(tone.asked(opened + minutes(1), false), ToneNow::Sound);
        assert_eq!(
            tone.asked(
                opened + minutes(1) + std::time::Duration::from_secs(1),
                false
            ),
            ToneNow::Wait,
            "sounded twice in one minute"
        );
        assert_eq!(tone.asked(opened + minutes(2), false), ToneNow::Sound);
    }

    #[test]
    fn test_the_tone_stops_for_good_once_focus_has_arrived_even_if_it_leaves() {
        let opened = std::time::Instant::now();
        let mut tone = RepeatingTone::from_the_window_opening(opened);
        assert_eq!(tone.asked(opened + minutes(1), false), ToneNow::Sound);

        // Focus arrives between tones, then leaves again. Somebody who has
        // heard it and gone back to what they were doing has chosen to.
        assert_eq!(
            tone.asked(opened + minutes(1) + minutes(0), true),
            ToneNow::Finished
        );
        assert_eq!(
            tone.asked(opened + minutes(2), false),
            ToneNow::Finished,
            "the tone came back after focus had reached the window and left"
        );
        assert_eq!(tone.asked(opened + minutes(30), false), ToneNow::Finished);
    }

    #[test]
    fn test_a_window_that_opens_with_focus_never_sounds_again() {
        // The ordinary case: the dialog is modal and takes focus as it opens,
        // so the first ask finds focus and nothing repeats.
        let opened = std::time::Instant::now();
        let mut tone = RepeatingTone::from_the_window_opening(opened);
        assert_eq!(tone.asked(opened, true), ToneNow::Finished);
        assert_eq!(tone.asked(opened + minutes(1), false), ToneNow::Finished);
    }

    #[test]
    fn test_the_tone_stops_after_the_ceiling_whatever_focus_does() {
        // Guardrail 5: bounded. Ten, then silence with the window still on
        // screen, and a machine left running does not sound all afternoon.
        let opened = std::time::Instant::now();
        let mut tone = RepeatingTone::from_the_window_opening(opened);

        let mut sounded = 0;
        for minute in 1..=60u64 {
            if tone.asked(opened + minutes(minute), false) == ToneNow::Sound {
                sounded += 1;
            }
        }
        assert_eq!(sounded, MOST_TONES, "the ceiling is not the ceiling");
        assert_eq!(
            tone.asked(opened + minutes(61), false),
            ToneNow::Finished,
            "past the ceiling the rule still asks to be asked again"
        );
    }
}

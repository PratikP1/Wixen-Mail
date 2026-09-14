//! The window that opens when a reminder comes due.
//!
//! Reminders were stored, listed, synced and never once went off, so setting
//! one bought nothing over writing a note with a date on it.
//!
//! Three channels, because one is not enough for anybody: the window itself for
//! anyone looking, the announcement for anyone listening, and a sound for the
//! moment it arrives, which is the part that reaches somebody who is not
//! looking at this application at all. The sound is switchable like every other
//! one, and the window is not, because the window is the thing.

use crate::application::due::{Due, Snooze};
use crate::presentation::accessibility::Accessibility;
use crate::presentation::accessibility::names::set_accessible_name;
use crate::presentation::theme;
use wxdragon::prelude::*;

/// What was decided about a reminder that went off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    /// Leave it alone. It stays due and will not be raised again this session.
    Dismissed,
    /// Come back in this long.
    Snoozed(Snooze),
    /// It is finished.
    Done,
}

/// The event this window sounds when a reminder goes off.
///
/// Pulled out of `raise` so a test can read it. `raise` needs a display and a
/// running application, so which event it asks for was a decision nothing
/// could check, and it was wrong: it asked for the one that means new mail.
pub(crate) const ALERT_EVENT: crate::presentation::accessibility::feedback::Event =
    crate::presentation::accessibility::feedback::Event::Reminder;

const ID_SNOOZE: Id = ID_HIGHEST + 401;
const ID_DONE: Id = ID_HIGHEST + 402;
const ID_DISMISS: Id = ID_HIGHEST + 403;

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

/// Say a reminder and sound its tone, without its window.
///
/// This exists because the window is this event's written equivalent, so a
/// reminder whose window is held back has to say something or the event goes
/// out as nothing at all: no tone, no sentence, no window. Pulled out of
/// `raise` so a reminder found due while somebody is typing can be said at
/// that look and have its window a look later.
///
/// Said once. A reminder is said here at the moment it is found due, whether
/// or not its window can open then. When the window opens after a hold it is
/// not said again: the sentence is the window's own text and accessible name,
/// and a dialog's text is what a screen reader reads when focus arrives in it.
/// Two announcements a minute apart would be the same fact twice to somebody
/// who cannot skim.
pub fn say(
    item: &Due,
    now: chrono::DateTime<chrono::Local>,
    dates: crate::presentation::date_display::DateSettings,
    a11y: &Accessibility,
) -> Said {
    // One sentence, said here and later shown and put on the window's own
    // name, so the three channels cannot say different things.
    let sentence = item.spoken(now, dates);
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

/// Whether `say` has already happened for the reminder `raise` is opening.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spoken {
    /// Not yet. `raise` says it before the window, as it always did.
    NotYet,
    /// Said at an earlier look, while the window was held. Not said again.
    Already,
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
/// sentence on it, and the reminder is not lost.
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

/// The tone that comes back until focus reaches the reminder window.
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
    /// the reminder's own, sounded by `say`; these are the ones after it.
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

/// Raise one reminder and wait for an answer.
///
/// Modal on purpose. A reminder that can be left sitting behind the window it
/// interrupted is one somebody will find tomorrow, and the whole point of
/// asking to be told is being told at the time.
///
/// `spoken` says whether [`say`] already happened at an earlier look, while
/// the window was held back; if so it is not said again. Either way the
/// sentence on the window is the one `say` produces, so the channels agree.
///
/// While the window is open, its tone comes back once a minute until focus
/// reaches it, on the rule in [`RepeatingTone`]. That is for somebody in
/// another application when the reminder was due: the sentence may have gone
/// unheard there, and the tone is what reaches across. The tone alone; its
/// written equivalent is the window on screen.
pub fn raise(
    parent: &Frame,
    item: &Due,
    now: chrono::DateTime<chrono::Local>,
    dates: crate::presentation::date_display::DateSettings,
    a11y: &std::sync::Arc<Accessibility>,
    default_snooze: Snooze,
    spoken: Spoken,
) -> Answer {
    // Before the window, so it arrives with the window rather than after
    // somebody has already started reading it.
    let said = match spoken {
        Spoken::NotYet => say(item, now, dates, a11y).sentence,
        Spoken::Already => item.spoken(now, dates),
    };
    let said = said.as_str();

    let (dialog, snooze_choice) = build_reminder_alert_dialog(
        parent,
        said,
        default_snooze,
        theme::current_from_stored_config(),
    );

    // Held until after `show_modal` returns: dropping a timer stops it, so one
    // that fell out of scope here would never tick. Dropped before `destroy`,
    // because its owner is the dialog. Focus is asked of the four controls
    // rather than the dialog, because on Windows a dialog has focus only when
    // none of its children does, which is never while somebody is in it.
    //
    // The tick does not stop the timer once the rule says Finished. Stopping
    // it from inside its own handler means the handler owning the timer,
    // which is a cycle across the toolkit boundary; a tick that asks and is
    // told Finished costs nothing, and the drop below ends it.
    let watching = Timer::new(&dialog);
    watching.on_tick({
        let a11y = a11y.clone();
        let mut tone = RepeatingTone::from_the_window_opening(std::time::Instant::now());
        move |_| {
            let focus_is_here = snooze_choice.has_focus()
                || [ID_SNOOZE, ID_DONE, ID_DISMISS].iter().any(|id| {
                    dialog
                        .find_window_by_id(*id)
                        .is_some_and(|button| button.has_focus())
                });
            match tone.asked(std::time::Instant::now(), focus_is_here) {
                ToneNow::Sound => {
                    let _ = a11y.earcon(ALERT_EVENT);
                }
                ToneNow::Wait | ToneNow::Finished => (),
            }
        }
    });
    if !watching.start(HOW_OFTEN_TO_ASK_THE_TONE, false) {
        // The window still opens and the reminder is still on it. What is
        // lost is the tone coming back, which is said rather than swallowed.
        tracing::warn!("The reminder window's timer refused to start; its tone will not repeat");
    }

    let answer = dialog.show_modal();
    drop(watching);
    let chosen = Snooze::ALL
        .get(snooze_choice.get_selection().unwrap_or(0) as usize)
        .copied()
        .unwrap_or(default_snooze);
    dialog.destroy();

    match answer {
        id if id == ID_SNOOZE => Answer::Snoozed(chosen),
        id if id == ID_DONE => Answer::Done,
        // Closing the window with Escape or the title bar is the same as
        // dismissing: the reminder is left as it is and not raised again this
        // session. Treating a closed window as a snooze would bring it back at
        // somebody who had just decided they were finished with it.
        _ => Answer::Dismissed,
    }
}

/// Build the reminder alert dialog without showing it.
///
/// Everything `raise` used to do from its own window onward, split out the
/// same way [`crate::presentation::wx_settings::build_settings_dialog`]
/// splits Settings: a test can build the real dialog and read back the real
/// colour a live control holds, and never call `.show_modal()`, sound an
/// earcon, or make an announcement at all.
///
/// `said` is the sentence `raise` has already spoken and shown once; this
/// only ever puts it on screen. Returns the snooze choice alongside the
/// dialog, the same way the caller needs it after a real `.show_modal()`: to
/// read how long was chosen.
pub fn build_reminder_alert_dialog(
    parent: &Frame,
    said: &str,
    default_snooze: Snooze,
    palette: Option<theme::Palette>,
) -> (Dialog, Choice) {
    let dialog = Dialog::builder(parent, "Reminder")
        .with_size(420, 220)
        .build();

    let sizer = BoxSizer::builder(Orientation::Vertical).build();

    // The whole sentence, not just the title. It is what the window is for,
    // and it is what a braille display shows off the first line.
    let what = StaticText::builder(&dialog).with_label(said).build();
    set_accessible_name(&what, said);
    sizer.add(&what, 0, SizerFlag::Expand | SizerFlag::All, 12);

    let snooze_row = BoxSizer::builder(Orientation::Horizontal).build();
    let snooze_label = StaticText::builder(&dialog)
        .with_label("Come back in:")
        .build();
    let snooze_choice = Choice::builder(&dialog)
        .with_choices(Snooze::ALL.iter().map(|s| s.label()).collect())
        .with_selection(Some(
            Snooze::ALL
                .iter()
                .position(|s| *s == default_snooze)
                .unwrap_or(0) as u32,
        ))
        .build();
    set_accessible_name(&snooze_choice, "Come back in");
    snooze_row.add(
        &snooze_label,
        0,
        SizerFlag::AlignCenterVertical | SizerFlag::All,
        4,
    );
    snooze_row.add(&snooze_choice, 1, SizerFlag::Expand | SizerFlag::All, 4);
    sizer.add_sizer(&snooze_row, 0, SizerFlag::Expand | SizerFlag::All, 8);

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    // Snooze first and given the focus, because it is the answer that keeps the
    // reminder. Dismiss is the one that loses it, and a destructive default is
    // how somebody dismisses by reflex what they meant to keep.
    let snooze_btn = Button::builder(&dialog)
        .with_label("&Snooze")
        .with_id(ID_SNOOZE)
        .build();
    let done_btn = Button::builder(&dialog)
        .with_label("Mark &Done")
        .with_id(ID_DONE)
        .build();
    let dismiss_btn = Button::builder(&dialog)
        .with_label("D&ismiss")
        .with_id(ID_DISMISS)
        .build();
    buttons.add_spacer(0);
    buttons.add(&snooze_btn, 0, SizerFlag::All, 4);
    buttons.add(&done_btn, 0, SizerFlag::All, 4);
    buttons.add(&dismiss_btn, 0, SizerFlag::All, 4);
    sizer.add_sizer(&buttons, 0, SizerFlag::AlignRight | SizerFlag::All, 8);

    dialog.set_sizer(sizer, true);

    for (button, id) in [
        (&snooze_btn, ID_SNOOZE),
        (&done_btn, ID_DONE),
        (&dismiss_btn, ID_DISMISS),
    ] {
        button.on_click(move |_| dialog.end_modal(id));
    }

    snooze_btn.set_focus();

    // Painted last. No `TextCtrl`, `ListCtrl` or `TreeCtrl` anywhere in this
    // dialog (`StaticText`, `Choice` and buttons only), so the dialog itself
    // is the only site: the snooze `Choice`, like every other `Choice` this
    // round paints around, is left to Windows. `None` means high contrast is
    // on, or the system is set up in a way this application should not paint
    // over, so nothing is set here and Windows decides.
    if let Some(palette) = palette {
        theme::paint(&dialog, palette.main_surface());
    }

    (dialog, snooze_choice)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::accessibility::feedback::Event;

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
        let item = Due {
            identity: crate::application::due::Identity {
                kind: crate::application::due::Kind::Reminder,
                id: "r1".to_string(),
            },
            title: "Ring the bank".to_string(),
            when: "2026-09-14T10:00:00".to_string(),
            late: false,
        };
        let now = chrono::Local::now();
        let dates = crate::presentation::date_display::DateSettings::default();

        let said = say(&item, now, dates, &a11y);

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
        let said_quietly = say(&item, now, dates, &quiet);
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

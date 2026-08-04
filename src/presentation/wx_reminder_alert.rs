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

/// Raise one reminder and wait for an answer.
///
/// Modal on purpose. A reminder that can be left sitting behind the window it
/// interrupted is one somebody will find tomorrow, and the whole point of
/// asking to be told is being told at the time.
pub fn raise(
    parent: &Frame,
    item: &Due,
    now: chrono::DateTime<chrono::Local>,
    dates: crate::presentation::date_display::DateSettings,
    a11y: &Accessibility,
    default_snooze: Snooze,
) -> Answer {
    // One sentence, said, shown and put on the window's own name, so the three
    // channels cannot say different things.
    let said = item.spoken(now, dates);
    let said = said.as_str();

    // Before the window, so it arrives with the window rather than after
    // somebody has already started reading it.
    let _ = a11y.earcon(ALERT_EVENT);
    let _ = a11y.announce(
        said,
        crate::presentation::accessibility::announcements::Priority::Urgent,
    );

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
    let answer = dialog.show_modal();
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
}

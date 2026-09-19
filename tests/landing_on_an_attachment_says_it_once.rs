//! Landing on a message with an attachment says the word once (#77), and the
//! sounds are on for everybody from the start.
//!
//! The tester on 2026-09-18, under NVDA: landing on a message with an
//! attachment said "attachment" more than once, from the Attachment column
//! NVDA reads in the row, from the spoken "Has attachment" event, and from
//! the earcon when it was on. Pratik's decision the same day: the earcon on by
//! default for this event, the spoken announcement off by default, the column
//! kept, since it carries the fact in the row. And, at 12:24Z on the issue,
//! the Earcon channel enabled by default for every event, with per-event
//! exceptions still allowed and this event's default being the exception.
//!
//! Two halves. The model, `FeedbackSettings` and `Event`, is held here as
//! cases: what a fresh profile hears, what the attachment event reaches under
//! the default, what the tester's own stored profile reaches, a profile that
//! chose silence keeping it, the fallback that never speaks for an event whose
//! text is already on the row, and speech turned back on for it. The window,
//! which needs a frame to run, is read as text over `what_ships` of
//! `src/presentation/wx_app.rs`: the landing arm signals the event and speaks
//! nothing of its own about attachments.
//!
//! # Braille, and why the decision's "and braille" is not sent
//!
//! Pratik's words for this event were the earcon plus the status bar and
//! braille, no speech. This program has no braille route apart from speech:
//! a set holding `Braille` calls `announce_topic`, the screen reader speaks
//! that notification, and the same row's Attachment column already sits on
//! the braille display as it sits in speech. So "braille, no speech" is not a
//! set this program can send, and a set naming Braille reinstates the
//! duplicate the issue exists to stop. The default is the earcon and the
//! status bar; one channel added to it is the overrule, and the close comment
//! on #77 says so.
//!
//! # Why this lives here rather than beside the code
//!
//! `src/presentation/accessibility/feedback.rs` is named by 3 guard records
//! and `src/presentation/wx_app.rs` by 98 on 2026-09-19, so a test added to
//! either is that many builds and library runs at the next commit. This file
//! is named by its own records, whose `suite` couples it to both, so it runs
//! on the commits that could break it.
//!
//! # What this cannot see
//!
//! Whether the row is heard once with the tone, and the tone alone when the
//! status bar is off: the tester's ear. The window is not started.

use std::collections::BTreeSet;

use wixen_mail::presentation::accessibility::feedback::{Channel, Event, FeedbackSettings};

fn set(channels: &[Channel]) -> BTreeSet<Channel> {
    channels.iter().copied().collect()
}

/// The stored form the tester's own profile held on 2026-09-18, read through
/// Python from `app_config.json`: every channel on, no per-event answer.
const THE_TESTERS_PROFILE: &str = "off=";

// ── The default ────────────────────────────────────────────────────────────

#[test]
fn test_a_fresh_profile_hears_every_tone() {
    // Pratik's decision on #77: the Earcon channel is on by default for every
    // event. Until 2026-09-18 it was off, for phase 6's reason that an
    // application which starts making noises nobody asked for is one people
    // switch the sounds off in for good; the tester's own profile had turned
    // them on, and the decision made that the default.
    let settings = FeedbackSettings::default();
    assert!(settings.is_channel_enabled(Channel::Earcon));
    for channel in Channel::ALL {
        assert!(
            settings.is_channel_enabled(channel),
            "{channel} is off for a fresh profile"
        );
    }
    for event in Event::ALL {
        assert!(
            settings.channels_for(event).contains(&Channel::Earcon),
            "{event:?} has no tone for a fresh profile"
        );
    }
}

#[test]
fn test_the_attachment_event_reaches_the_tone_and_the_status_bar_and_no_words() {
    // The column in the row is what NVDA reads; the event's own words would
    // be the same word a second time, so under the default they are not
    // spoken and not brailled. The sound and the status bar carry it.
    let settings = FeedbackSettings::default();
    assert_eq!(
        settings.channels_for(Event::HasAttachment),
        set(&[Channel::Earcon, Channel::Visual])
    );
}

#[test]
fn test_every_other_event_still_reaches_every_channel_under_the_default() {
    // The exception is this one event. A default that quietened others on
    // the way would take words away from somebody who never chose that.
    let settings = FeedbackSettings::default();
    let every_channel: BTreeSet<Channel> = Channel::ALL.into_iter().collect();
    for event in Event::ALL
        .into_iter()
        .filter(|e| *e != Event::HasAttachment)
    {
        assert_eq!(
            settings.channels_for(event),
            every_channel,
            "{event:?} lost a channel it never had an exception for"
        );
    }
}

#[test]
fn test_nobody_has_chosen_anything_for_a_fresh_profile() {
    // The exception is the event's own default, not an answer somebody gave.
    // The Feedback tab tells those two apart in words ("using the default"
    // against "an answer of its own"), and a fresh profile that read as
    // having chosen something for this event would be the screen telling
    // somebody they did something they did not.
    let settings = FeedbackSettings::default();
    for event in Event::ALL {
        assert_eq!(
            settings.what_was_chosen_for(event),
            None,
            "{event:?} reads as chosen on a profile nobody has touched"
        );
    }
    assert_eq!(settings.to_stored(), "off=");
}

// ── Stored profiles ────────────────────────────────────────────────────────

#[test]
fn test_the_profile_that_filed_the_issue_hears_the_attachment_event_the_same_way() {
    // The tester's profile has every channel on and no per-event answer. If
    // the exception lived only in what a fresh profile stores, his would go
    // on speaking "Has attachment" over the row until he opened Settings,
    // and the fix would not reach the person who reported it.
    let restored = FeedbackSettings::from_stored(THE_TESTERS_PROFILE);
    assert_eq!(
        restored.channels_for(Event::HasAttachment),
        set(&[Channel::Earcon, Channel::Visual])
    );
    assert!(restored.is_channel_enabled(Channel::Earcon));
    assert_eq!(
        restored.channels_for(Event::NewMail).len(),
        Channel::ALL.len()
    );
}

#[test]
fn test_a_profile_that_chose_silence_keeps_it() {
    // Somebody who turned the sounds off before the default moved has said
    // what they want, and a default that moves must not move them (T-11-51).
    // For the attachment event that leaves the status bar alone, since the
    // rule below never adds speech for it.
    let restored = FeedbackSettings::from_stored("off=earcon");
    assert!(!restored.is_channel_enabled(Channel::Earcon));
    for event in Event::ALL {
        assert!(
            !restored.channels_for(event).contains(&Channel::Earcon),
            "{event:?} still sounds on a profile that turned the sounds off"
        );
    }
    assert_eq!(
        restored.channels_for(Event::HasAttachment),
        set(&[Channel::Visual])
    );
}

#[test]
fn test_a_stored_string_with_nothing_in_it_answers_the_new_default() {
    // An empty `feedback_channels` is what every settings file written before
    // the Feedback tab existed holds, and what the window reads as "nothing
    // stored". It is the new default, not the old one.
    let restored = FeedbackSettings::from_stored("");
    assert_eq!(restored, FeedbackSettings::default());
    assert!(restored.is_channel_enabled(Channel::Earcon));
}

#[test]
fn test_speech_can_be_turned_back_on_for_the_attachment_event_and_survives_a_restart() {
    // The Feedback tab's One event at a time section writes an answer for
    // the event; an answer naming every channel puts the words back, and it
    // has to come back from the settings file the same way.
    let mut settings = FeedbackSettings::default();
    settings.set_event_channels(Event::HasAttachment, Channel::ALL.into_iter().collect());
    assert!(
        settings
            .channels_for(Event::HasAttachment)
            .contains(&Channel::Speech)
    );

    let restored = FeedbackSettings::from_stored(&settings.to_stored());
    assert_eq!(restored, settings);
    assert!(
        restored
            .channels_for(Event::HasAttachment)
            .contains(&Channel::Speech)
    );
    assert_eq!(
        restored.what_was_chosen_for(Event::HasAttachment),
        Some(Channel::ALL.into_iter().collect())
    );
}

// ── The fallback ───────────────────────────────────────────────────────────

#[test]
fn test_only_the_attachment_event_says_its_text_is_already_on_the_row() {
    // The mark is what the default and the fallback both read. Decided for
    // every event on purpose, so a later event whose text is on the row is
    // added here rather than found by ear.
    for event in Event::ALL {
        assert_eq!(
            event.text_is_already_on_the_row(),
            event == Event::HasAttachment,
            "{event:?}: the text is already on the row exactly for the attachment event, \
             whose Attachment column NVDA reads in the row"
        );
    }
}

#[test]
fn test_the_fallback_never_speaks_for_an_event_whose_text_is_on_the_row() {
    // The never-sound-alone rule adds a written channel to a sound-only set.
    // For this event the written channel is the status bar and nothing else:
    // braille rides the same screen reader notification as speech here, so
    // adding either puts the word back over the row (T-11-53).
    let mut settings = FeedbackSettings::default();
    settings.set_event_channels(Event::HasAttachment, set(&[Channel::Earcon]));
    assert_eq!(
        settings.channels_for(Event::HasAttachment),
        set(&[Channel::Earcon, Channel::Visual])
    );

    // With the status bar off too, the sound goes out alone rather than as
    // speech. That is the one place the rule yields, and it yields to the
    // row's own words, which are already there.
    settings.set_channel_enabled(Channel::Visual, false);
    assert_eq!(
        settings.channels_for(Event::HasAttachment),
        set(&[Channel::Earcon])
    );
    settings.use_the_default_for(Event::HasAttachment);
    assert_eq!(
        settings.channels_for(Event::HasAttachment),
        set(&[Channel::Earcon])
    );
}

#[test]
fn test_the_fallback_still_adds_braille_first_for_an_event_whose_text_is_not_on_the_row() {
    // The companion: the order Braille, then the status bar, then speech is
    // unchanged for every event without the mark, so the case above cannot
    // pass by the rule having stopped adding anything.
    let mut settings = FeedbackSettings::default();
    settings.set_event_channels(Event::NewMail, set(&[Channel::Earcon]));
    assert_eq!(
        settings.channels_for(Event::NewMail),
        set(&[Channel::Earcon, Channel::Braille])
    );
    settings.set_channel_enabled(Channel::Braille, false);
    assert_eq!(
        settings.channels_for(Event::NewMail),
        set(&[Channel::Earcon, Channel::Visual])
    );
    settings.set_channel_enabled(Channel::Visual, false);
    assert_eq!(
        settings.channels_for(Event::NewMail),
        set(&[Channel::Earcon, Channel::Speech])
    );
}

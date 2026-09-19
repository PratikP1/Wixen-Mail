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
use std::fs;

use wixen_mail::common::what_ships::what_ships;
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

// ── The window, read as text ───────────────────────────────────────────────

const THE_MAIN_WINDOW: &str = "src/presentation/wx_app.rs";

fn the_main_window() -> String {
    let whole = fs::read_to_string(THE_MAIN_WINDOW)
        .unwrap_or_else(|why| panic!("{THE_MAIN_WINDOW}: {why}"))
        .replace("\r\n", "\n");
    what_ships(&whole)
}

/// One function's text, from its signature to the closing brace at column
/// nought, or a complaint when the signature is gone.
fn body_of(source: &str, signature: &str) -> Result<String, String> {
    let at = source.find(signature).ok_or(format!(
        "{signature} is no longer in this file, so this reads nothing"
    ))?;
    let rest = &source[at..];
    let ends = rest.find("\n}\n").map_or(rest.len(), |end| end + 2);
    Ok(rest[..ends].to_string())
}

/// The message list's cursor handler, cut between two anchors: where it is
/// wired and the handler wired after it.
fn the_cursor_handler(source: &str) -> Result<&str, String> {
    let after = source
        .split_once(THE_CURSOR_HANDLER)
        .ok_or(format!(
            "{THE_CURSOR_HANDLER} is no longer in this file, so this reads nothing"
        ))?
        .1;
    let end = after.find(THE_HANDLER_AFTER_IT).ok_or(format!(
        "{THE_HANDLER_AFTER_IT} no longer follows the cursor handler, so this cannot tell \
         where the handler ends"
    ))?;
    Ok(&after[..end])
}

/// Every string literal in `text`, read line by line with the comment at
/// the end of a line cut off first.
///
/// A reader of source text, with the blind spot said: a `//` inside a
/// literal ends the line early, and a literal spanning lines is read as
/// two. Neither shape is in the handler this reads, and the companions
/// below prove a planted literal is seen.
fn string_literals_in(text: &str) -> Vec<String> {
    text.lines()
        .map(|line| line.split("//").next().unwrap_or_default())
        .flat_map(|code| {
            code.split('"')
                .enumerate()
                .filter(|(at, _)| at % 2 == 1)
                .map(|(_, literal)| literal.to_string())
                .collect::<Vec<_>>()
        })
        .collect()
}

// ── The anchors ────────────────────────────────────────────────────────────

const THE_CURSOR_HANDLER: &str = "msg_list.on_item_focused({";
const THE_HANDLER_AFTER_IT: &str = "msg_list.on_column_click({";
const THE_LANDING_LOOP: &str = "for landing_event in landing_events {";
const SIGNALS_THE_EVENT: &str = "a11y.signal(landing_event";
const THE_MESSAGE_LANDING: &str = "fn feedback_events_for_landing(";
const THE_CONVERSATION_LANDING: &str = "fn feedback_events_for_landing_on_a_conversation(";
const THE_EVENT: &str = "FeedbackEvent::HasAttachment";
const THE_WORD: &str = "attachment";

// ── The readings ───────────────────────────────────────────────────────────

/// Landing on a message or a conversation with an attachment is decided by
/// the two pure functions and signalled, once per event, by the cursor
/// handler; the channels are the settings' and never the handler's.
fn the_cursor_handler_signals_what_the_landing_decides(app: &str) -> Result<(), String> {
    for landing in [THE_MESSAGE_LANDING, THE_CONVERSATION_LANDING] {
        if !body_of(app, landing)?.contains(THE_EVENT) {
            return Err(format!(
                "{landing} never answers {THE_EVENT}, so landing on an attachment signals \
                 nothing and the tone is never played"
            ));
        }
    }
    let handler = the_cursor_handler(app)?;
    let after_the_loop = handler.split_once(THE_LANDING_LOOP).ok_or(format!(
        "the cursor handler no longer holds {THE_LANDING_LOOP:?}, so what the landing \
         decides is never signalled"
    ))?;
    if !after_the_loop
        .1
        .trim_start()
        .starts_with(&format!("let _ = {SIGNALS_THE_EVENT}"))
    {
        return Err(format!(
            "the landing loop does not begin by signalling the event through \
             {SIGNALS_THE_EVENT}, so the channels an event reaches are not the settings'"
        ));
    }
    Ok(())
}

/// The cursor handler adds nothing spoken about attachments: no string it
/// holds names the word. The row's own Attachment column and the signalled
/// event are the only two ways the fact goes out, and the event's channels
/// are the settings' (#77).
fn the_cursor_handler_speaks_nothing_of_its_own_about_attachments(app: &str) -> Result<(), String> {
    let handler = the_cursor_handler(app)?;
    if let Some(literal) = string_literals_in(handler)
        .into_iter()
        .find(|literal| literal.to_lowercase().contains(THE_WORD))
    {
        return Err(format!(
            "the cursor handler holds the words {literal:?}, so landing on a message with an \
             attachment says the word a second time beside the row's own column and the \
             signalled event, whatever the Feedback tab says"
        ));
    }
    Ok(())
}

#[test]
fn test_the_cursor_handler_signals_what_the_landing_decides() {
    the_cursor_handler_signals_what_the_landing_decides(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_cursor_handler_speaks_nothing_of_its_own_about_attachments() {
    the_cursor_handler_speaks_nothing_of_its_own_about_attachments(&the_main_window())
        .unwrap_or_else(|why| panic!("{why}"));
}

// ── The companions ─────────────────────────────────────────────────────────

/// A window shaped as it should be, so a reading that stopped finding its
/// anchor cannot pass by finding nothing.
fn a_window_as_it_should_be() -> String {
    let mut snippet = String::new();
    snippet.push_str("            ");
    snippet.push_str(THE_CURSOR_HANDLER);
    snippet.push_str(
        "\n                let a11y = a11y.clone();\n                move |event| {\n                    \
         let idx = event.get_item_index() as usize;\n                    \
         let landing_events = feedback_events_for_landing(&row);\n                    ",
    );
    snippet.push_str(THE_LANDING_LOOP);
    snippet.push_str("\n                        let _ = ");
    snippet.push_str(SIGNALS_THE_EVENT);
    snippet.push_str(
        ", \"\");\n                    }\n                    // Landing on a row with an attachment\n                    \
         let _ = a11y.announce_topic(PRESS_ENTER_TO_RUN, Priority::Low, \"saved search\");\n                }\n            });\n\n            ",
    );
    snippet.push_str(THE_HANDLER_AFTER_IT);
    snippet.push_str("\n                move |_| {}\n            });\n");
    snippet.push_str(THE_MESSAGE_LANDING);
    snippet.push_str(
        "message: &MessageItem) -> Vec<FeedbackEvent> {\n    let mut events = Vec::new();\n    \
         if message.has_attachments {\n        events.push(FeedbackEvent::HasAttachment);\n    }\n    events\n}\n",
    );
    snippet.push_str(THE_CONVERSATION_LANDING);
    snippet.push_str(
        "conversation: &ConversationItem) -> Vec<FeedbackEvent> {\n    let mut events = Vec::new();\n    \
         if conversation.any_attachment {\n        events.push(FeedbackEvent::HasAttachment);\n    }\n    events\n}\n",
    );
    snippet
}

fn every_window_reading_over(app: &str) -> Result<(), String> {
    the_cursor_handler_signals_what_the_landing_decides(app)?;
    the_cursor_handler_speaks_nothing_of_its_own_about_attachments(app)
}

#[test]
fn test_the_window_readings_pass_a_window_shaped_as_it_should_be() {
    every_window_reading_over(&a_window_as_it_should_be()).unwrap_or_else(|why| panic!("{why}"));
}

#[test]
fn test_the_window_readings_complain_when_the_handler_speaks_or_stops_signalling() {
    let app = a_window_as_it_should_be();

    let speaks = app.replacen(
        "let _ = a11y.announce_topic(PRESS_ENTER_TO_RUN, Priority::Low, \"saved search\");",
        "let _ = a11y.announce(\"Has attachment\", Priority::Low);",
        1,
    );
    let why = the_cursor_handler_speaks_nothing_of_its_own_about_attachments(&speaks)
        .expect_err("the handler speaks");
    assert!(why.contains("holds the words \"Has attachment\""), "{why}");

    // A comment naming the word is not a string, and a string outside the
    // handler is not the handler's.
    let comments_and_elsewhere = format!(
        "{}\nfn elsewhere() {{ let _ = \"Attachments, 3\"; }}\n",
        app.replacen(
            "// Landing on a row with an attachment",
            "// Landing on a row with an attachment says \"Has attachment\" nowhere here",
            1
        )
    );
    every_window_reading_over(&comments_and_elsewhere).unwrap_or_else(|why| panic!("{why}"));

    let never_signals = app.replacen(
        "let _ = a11y.signal(landing_event, \"\");",
        "let _ = landing_event;",
        1,
    );
    let why = the_cursor_handler_signals_what_the_landing_decides(&never_signals)
        .expect_err("never signals");
    assert!(why.contains("does not begin by signalling"), "{why}");

    let no_loop = app.replacen(THE_LANDING_LOOP, "for other in others {", 1);
    let why = the_cursor_handler_signals_what_the_landing_decides(&no_loop).expect_err("no loop");
    assert!(why.contains("no longer holds"), "{why}");

    let no_event = app.replacen(
        "if conversation.any_attachment {\n        events.push(FeedbackEvent::HasAttachment);\n    }\n",
        "",
        1,
    );
    let why = the_cursor_handler_signals_what_the_landing_decides(&no_event)
        .expect_err("no event for a conversation");
    assert!(
        why.contains("never answers FeedbackEvent::HasAttachment"),
        "{why}"
    );

    let no_handler = app.replacen(THE_CURSOR_HANDLER, "msg_list.on_item_selected({", 1);
    let why = the_cursor_handler_speaks_nothing_of_its_own_about_attachments(&no_handler)
        .expect_err("no handler");
    assert!(why.contains("is no longer in this file"), "{why}");
}

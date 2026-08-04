//! Signalling events on more than one channel.
//!
//! Speech is not the only way to tell someone something happened, and for some
//! people it is not a way at all. A deaf-blind user reading braille hears
//! nothing; someone working in an open office may want a short sound where a
//! sentence of speech would be intrusive; someone with a hearing loss needs
//! every sound to have a written equivalent.
//!
//! So an event is not "an announcement". It is a fact, and the channels it
//! reaches are a preference. This module holds the routing.
//!
//! Two rules are enforced here rather than left to callers:
//!
//! 1. **Nothing is signalled by sound alone** unless the user has switched the
//!    text channels off themselves. A sound with no written equivalent is
//!    invisible to a deaf-blind user and meaningless to anyone who has not
//!    learned it yet.
//! 2. **Sibling events sound different.** An earcon that cannot be told apart
//!    from the one before it carries no information, so each event maps to its
//!    own tone rather than to a generic "something happened".

use super::announcements::Priority;
use std::collections::BTreeSet;
use std::fmt;

/// Something worth telling the user about.
///
/// Deliberately a closed set. An open "signal this string" call is how a
/// codebase ends up with forty near-identical sounds that nobody can tell
/// apart, which is the failure this enum exists to prevent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Event {
    /// The cursor landed on a message that is part of a conversation.
    ThreadLanded,
    /// The cursor tried to move past the first or last row.
    EdgeOfList,
    /// New mail arrived.
    NewMail,
    /// A message was sent.
    MessageSent,
    /// A send failed.
    SendFailed,
    /// The connection to the server dropped.
    ConnectionLost,
    /// The connection came back.
    ConnectionRestored,
    /// A long operation finished.
    SyncComplete,
    /// Something the user asked for could not be done.
    ActionRefused,
    /// A message that was opened is spam, or looks like a phishing attempt.
    ///
    /// Configurable like every other event, and switchable off. Somebody who
    /// only ever reads their junk folder deliberately does not need to be told
    /// each time, and somebody who has heard it once about a message they are
    /// still reading does not need it again.
    UnsafeMessage,
    /// A word was finished and the dictionary does not have it.
    ///
    /// The only event whose written equivalent is not ours. The engine marks
    /// the word with a real spelling annotation, which every screen reader
    /// reads as the caret crosses it and which a braille display shows; that
    /// is the written form, and it is better than anything this could say
    /// because it names the word in place. What the sound adds is the moment:
    /// knowing at the end of the word rather than on the way back past it.
    MisspelledWord,
    /// A reminder came due.
    ///
    /// The second event whose written equivalent is not this module's. The
    /// reminder window says what is due and when, in the same sentence it
    /// announces, and that names the reminder where "Reminder" alone would
    /// not. So this goes out through `earcon` rather than `signal`, and the
    /// window is the visible equivalent the sound needs.
    ///
    /// It has its own tone because it is its own fact. It used to borrow the
    /// one for new mail, which meant a reminder sent people to an empty inbox,
    /// and meant switching the new mail sound off switched reminders off too.
    Reminder,
}

impl Event {
    /// Every event, so settings and tests can cover the whole set.
    pub const ALL: [Event; 12] = [
        Event::ThreadLanded,
        Event::EdgeOfList,
        Event::NewMail,
        Event::MessageSent,
        Event::SendFailed,
        Event::ConnectionLost,
        Event::ConnectionRestored,
        Event::SyncComplete,
        Event::ActionRefused,
        Event::UnsafeMessage,
        Event::MisspelledWord,
        Event::Reminder,
    ];

    /// The identifier used when preferences are stored.
    pub fn key(&self) -> &'static str {
        match self {
            Event::ThreadLanded => "thread_landed",
            Event::EdgeOfList => "edge_of_list",
            Event::NewMail => "new_mail",
            Event::MessageSent => "message_sent",
            Event::SendFailed => "send_failed",
            Event::ConnectionLost => "connection_lost",
            Event::ConnectionRestored => "connection_restored",
            Event::SyncComplete => "sync_complete",
            Event::ActionRefused => "action_refused",
            Event::UnsafeMessage => "unsafe_message",
            Event::MisspelledWord => "misspelled_word",
            Event::Reminder => "reminder",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        Event::ALL.into_iter().find(|e| e.key() == key)
    }

    /// The written equivalent, used for speech, braille, and the status line.
    ///
    /// Every event has one whether or not the user has speech switched on,
    /// because an earcon without a written equivalent is a sound a deaf-blind
    /// user cannot perceive at all.
    pub fn text(&self) -> &'static str {
        match self {
            Event::ThreadLanded => "Conversation",
            Event::EdgeOfList => "End of list",
            Event::NewMail => "New mail",
            Event::MessageSent => "Message sent",
            Event::SendFailed => "Message not sent",
            Event::ConnectionLost => "Disconnected",
            Event::ConnectionRestored => "Connected",
            Event::SyncComplete => "Sync finished",
            Event::ActionRefused => "Not available",
            Event::UnsafeMessage => "Unsafe message",
            Event::MisspelledWord => "Misspelled word",
            Event::Reminder => "Reminder",
        }
    }

    /// The written form with `detail` added, where the detail adds anything.
    ///
    /// A detail that only repeats the event's own words is dropped rather than
    /// said twice. Every arrival of mail was announced as "New mail, New mail",
    /// because the call site passed the event's own wording as the detail, and
    /// the rule lives here so that no other call site can make the same
    /// mistake again.
    ///
    /// Comparison ignores case, because a detail is written by a person and
    /// "new mail" is the same repetition as "New mail".
    pub fn text_with(&self, detail: &str) -> String {
        let detail = detail.trim();
        if detail.is_empty() || detail.eq_ignore_ascii_case(self.text()) {
            self.text().to_string()
        } else {
            format!("{}, {}", self.text(), detail)
        }
    }

    /// How hard the announcement queue should hold on to this.
    ///
    /// Failures outrank arrivals: missing "new mail" costs a moment, missing
    /// "message not sent" costs the message.
    pub fn priority(&self) -> Priority {
        match self {
            // Urgent alongside a failed send. Somebody is about to read a
            // message that is trying to deceive them, and an announcement that
            // waits its turn behind a sync notice arrives after they have
            // started reading it.
            Event::SendFailed | Event::ConnectionLost | Event::UnsafeMessage | Event::Reminder => {
                Priority::Urgent
            }
            Event::ActionRefused | Event::ConnectionRestored => Priority::High,
            Event::NewMail | Event::MessageSent | Event::SyncComplete => Priority::Normal,
            Event::ThreadLanded | Event::EdgeOfList | Event::MisspelledWord => Priority::Low,
        }
    }

    /// The tone this event plays, as frequency in hertz and length in
    /// milliseconds.
    ///
    /// Distinct per event on purpose. Rising pairs mean something arrived or
    /// succeeded, falling pairs mean something went wrong, and the two
    /// navigation events are single short ticks so they do not sound like
    /// status at all.
    pub fn tone(&self) -> Tone {
        match self {
            Event::ThreadLanded => Tone::new(880, 40),
            Event::EdgeOfList => Tone::new(440, 40),
            Event::NewMail => Tone::new(660, 90),
            Event::MessageSent => Tone::new(990, 90),
            Event::SendFailed => Tone::new(220, 220),
            Event::ConnectionLost => Tone::new(280, 180),
            Event::ConnectionRestored => Tone::new(740, 120),
            Event::SyncComplete => Tone::new(560, 110),
            Event::ActionRefused => Tone::new(330, 140),
            // The lowest and the longest in the set, which nothing else is, so
            // it cannot be mistaken for a status noise. Not lower still: the
            // floor here is 200 Hz, because a tone below that is felt more than
            // heard on the speakers most laptops have.
            Event::UnsafeMessage => Tone::new(240, 280),
            // Shorter than anything else, because it happens while somebody is
            // typing and a tone they have to wait out is one they turn off. Low
            // enough not to be mistaken for the two navigation ticks, which are
            // the only other sounds this brief.
            Event::MisspelledWord => Tone::new(380, 35),
            // The highest in the set and more than twice the length of the two
            // arrival sounds, which is what keeps it off new mail: a fourth
            // above the higher of them, and 200 ms against their 90.
            //
            // Not lower: everything from 220 to 330 Hz is the failure family,
            // and a reminder must not sound like a send that failed. Not
            // higher: age-related hearing loss takes the top of the range
            // first, and a reminder is the wrong event to put up there. Not
            // longer: the sound is played on the thread that then builds the
            // window, so its length is time the window is not appearing in.
            //
            // These are proposed numbers. Whether the two are tellable apart
            // by ear is a listening pass, and the test beside this one is
            // written as separation so that pass can move them.
            Event::Reminder => Tone::new(1320, 200),
        }
    }
}

/// A single tone: what an earcon is, before anything plays it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tone {
    pub hertz: u32,
    pub millis: u32,
}

impl Tone {
    const fn new(hertz: u32, millis: u32) -> Self {
        Self { hertz, millis }
    }
}

/// A way of reaching the user.
///
/// Braille is listed separately from speech even though a screen reader
/// usually drives both from one notification. The distinction is a preference,
/// not a transport: someone can want the written form without the spoken one,
/// and saying so here is what lets the visual channel stand in when speech is
/// off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Channel {
    Speech,
    Braille,
    Earcon,
    Visual,
}

impl Channel {
    pub const ALL: [Channel; 4] = [
        Channel::Speech,
        Channel::Braille,
        Channel::Earcon,
        Channel::Visual,
    ];

    /// Whether this channel carries words rather than a sound.
    ///
    /// The never-sound-alone rule is written in terms of this.
    fn carries_text(&self) -> bool {
        !matches!(self, Channel::Earcon)
    }

    pub fn key(&self) -> &'static str {
        match self {
            Channel::Speech => "speech",
            Channel::Braille => "braille",
            Channel::Earcon => "earcon",
            Channel::Visual => "visual",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        Channel::ALL.into_iter().find(|c| c.key() == key)
    }

    /// The wording beside this channel's checkbox in Settings.
    ///
    /// Here rather than in the dialog so that a label and the channel it
    /// switches cannot come apart. They used to be two arrays paired by
    /// position, and the cost of them drifting is specific: somebody ticks
    /// "Play a short sound for each event", the application switches their
    /// speech off instead, and nothing says so, because the thing that would
    /// say so is what was just switched off.
    ///
    /// The ampersand marks the keyboard accelerator, as everywhere else in the
    /// dialog. The accessible name is derived from this by dropping it, rather
    /// than written out a second time by hand.
    pub fn setting_label(&self) -> &'static str {
        match self {
            Channel::Speech => "&Speak events through the screen reader",
            Channel::Braille => "Send events to a &braille display",
            Channel::Earcon => "Play a short &sound for each event",
            Channel::Visual => "Show events in the s&tatus bar",
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.key())
    }
}

/// Which channels each event reaches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackSettings {
    /// Channels the user has switched off everywhere.
    ///
    /// This is what makes sound-only legitimate: someone who has deliberately
    /// turned speech, braille and the status line off has said they want the
    /// sounds and nothing else, and the rule below stops fighting them.
    disabled: BTreeSet<Channel>,
    /// Per-event overrides. An event with no entry uses the default.
    per_event: Vec<(Event, BTreeSet<Channel>)>,
}

impl Default for FeedbackSettings {
    /// Speech, braille, and the status line on; earcons off.
    ///
    /// Off by default because an application that starts making noises nobody
    /// asked for is one people switch the sounds off in, permanently, before
    /// they ever find out which sound meant what.
    fn default() -> Self {
        Self {
            disabled: [Channel::Earcon].into_iter().collect(),
            per_event: Vec::new(),
        }
    }
}

impl FeedbackSettings {
    /// Switch a channel on or off across every event.
    pub fn set_channel_enabled(&mut self, channel: Channel, enabled: bool) {
        if enabled {
            self.disabled.remove(&channel);
        } else {
            self.disabled.insert(channel);
        }
    }

    pub fn is_channel_enabled(&self, channel: Channel) -> bool {
        !self.disabled.contains(&channel)
    }

    /// Choose the channels for one event, overriding the global setting.
    fn set_event_channels(&mut self, event: Event, channels: BTreeSet<Channel>) {
        self.per_event.retain(|(e, _)| *e != event);
        self.per_event.push((event, channels));
    }

    /// The channels an event actually reaches.
    ///
    /// The never-sound-alone rule lives here rather than at the call sites, so
    /// no future caller can signal something by sound alone by forgetting it.
    pub fn channels_for(&self, event: Event) -> BTreeSet<Channel> {
        let chosen: BTreeSet<Channel> = match self.per_event.iter().find(|(e, _)| *e == event) {
            Some((_, channels)) => channels.clone(),
            None => Channel::ALL.into_iter().collect(),
        };
        let mut active: BTreeSet<Channel> = chosen
            .into_iter()
            .filter(|c| self.is_channel_enabled(*c))
            .collect();

        // Sound with nothing written alongside it. If any text channel is
        // available at all, add the quietest one rather than let the event go
        // out as a noise with no meaning.
        let sound_only = !active.is_empty() && !active.iter().any(Channel::carries_text);
        if sound_only
            && let Some(fallback) = [Channel::Braille, Channel::Visual, Channel::Speech]
                .into_iter()
                .find(|c| self.is_channel_enabled(*c))
        {
            active.insert(fallback);
        }
        active
    }

    /// The stored form, one `event:channel+channel` group per event.
    pub fn to_stored(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        let off: Vec<&str> = Channel::ALL
            .into_iter()
            .filter(|c| !self.is_channel_enabled(*c))
            .map(|c| c.key())
            .collect();
        parts.push(format!("off={}", off.join("+")));
        for (event, channels) in &self.per_event {
            let names: Vec<&str> = channels.iter().map(|c| c.key()).collect();
            parts.push(format!("{}={}", event.key(), names.join("+")));
        }
        parts.join(",")
    }

    /// Read stored preferences, ignoring anything unrecognised.
    ///
    /// A newer version may have written an event or channel this build has
    /// never heard of, and that must not throw away the rest of someone's
    /// settings.
    pub fn from_stored(stored: &str) -> Self {
        let mut settings = Self {
            disabled: BTreeSet::new(),
            per_event: Vec::new(),
        };
        let mut saw_off = false;
        for group in stored.split(',').filter(|g| !g.is_empty()) {
            let Some((name, values)) = group.split_once('=') else {
                continue;
            };
            let channels: BTreeSet<Channel> = values
                .split('+')
                .filter_map(Channel::from_key)
                .collect::<BTreeSet<_>>();
            if name == "off" {
                saw_off = true;
                settings.disabled = channels;
            } else if let Some(event) = Event::from_key(name) {
                settings.set_event_channels(event, channels);
            }
        }
        if !saw_off && settings.per_event.is_empty() {
            return Self::default();
        }
        settings
    }
}

/// Plays earcons, one at a time and never faster than the ear can separate.
///
/// Guardrail: feedback must be bounded. A syncing mailbox can raise the same
/// event forty times a second, and forty overlapping tones is not information,
/// it is noise that drives people to switch sound off for good.
#[derive(Debug)]
pub struct EarconPlayer {
    last_played: std::sync::Mutex<Option<std::time::Instant>>,
}

/// Shortest gap between two earcons.
///
/// Below about a tenth of a second two tones stop being heard as two events.
const EARCON_GAP: std::time::Duration = std::time::Duration::from_millis(120);

impl Default for EarconPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl EarconPlayer {
    pub fn new() -> Self {
        Self {
            last_played: std::sync::Mutex::new(None),
        }
    }

    /// Play a tone if enough time has passed since the last one.
    ///
    /// Returns whether it played, so a caller can tell the difference between
    /// "sounded" and "suppressed" rather than assuming.
    pub fn play(&self, tone: Tone) -> bool {
        self.play_at(tone, std::time::Instant::now())
    }

    /// The same decision with the clock passed in, so it can be tested.
    fn play_at(&self, tone: Tone, now: std::time::Instant) -> bool {
        let Ok(mut last) = self.last_played.lock() else {
            // A poisoned lock means another thread panicked mid-play. Staying
            // silent is the safe answer; a stuck tone is worse than none.
            return false;
        };
        if let Some(previous) = *last
            && now.duration_since(previous) < EARCON_GAP
        {
            return false;
        }
        *last = Some(now);
        emit(tone);
        true
    }
}

/// Sound one tone.
///
/// `Beep` is synchronous and blocks its thread for the tone's duration, so
/// every earcon here is short by design. It is also the one audio path that
/// needs no device setup, no extra dependency, and no permission, which
/// matters for a feature that has to work on a locked-down machine.
#[cfg(target_os = "windows")]
fn emit(tone: Tone) {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn Beep(frequency: u32, duration: u32) -> i32;
    }
    // Safety: Beep takes two integers and touches nothing we own.
    unsafe {
        Beep(tone.hertz, tone.millis);
    }
}

/// No earcons off Windows yet.
///
/// Said plainly rather than papered over: on macOS and Linux this is silent,
/// and the text channels carry the event on their own. A port needs its own
/// audio path, not a framework change.
#[cfg(not(target_os = "windows"))]
fn emit(_tone: Tone) {}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(channels: &[Channel]) -> BTreeSet<Channel> {
        channels.iter().copied().collect()
    }

    #[test]
    fn test_an_event_asked_to_repeat_itself_produces_its_words_once() {
        // Pins the string this builds, and nothing about it being spoken.
        //
        // The last case is the one worth keeping: a detail that genuinely adds
        // something is still added, so the rule that drops repetition cannot
        // quietly start dropping information.
        assert_eq!(Event::NewMail.text_with("New mail"), "New mail");
        assert_eq!(Event::NewMail.text_with("new mail"), "New mail");
        assert_eq!(Event::NewMail.text_with("   New mail  "), "New mail");
        assert_eq!(Event::NewMail.text_with(""), "New mail");
        assert_eq!(Event::NewMail.text_with("   "), "New mail");
        assert_eq!(
            Event::NewMail.text_with("3 messages"),
            "New mail, 3 messages"
        );
    }

    #[test]
    fn test_earcons_do_not_stack_up_under_a_burst() {
        // A syncing mailbox can raise the same event forty times a second.
        // Forty overlapping tones is noise, not information.
        let player = EarconPlayer::new();
        let start = std::time::Instant::now();
        assert!(player.play_at(Event::NewMail.tone(), start));
        assert!(!player.play_at(Event::NewMail.tone(), start));
        assert!(!player.play_at(
            Event::NewMail.tone(),
            start + std::time::Duration::from_millis(50)
        ));
        assert!(player.play_at(
            Event::NewMail.tone(),
            start + std::time::Duration::from_millis(200)
        ));
    }

    #[test]
    fn test_a_tone_exactly_one_gap_later_still_sounds() {
        // The gap is the shortest interval at which two tones are still heard
        // as two events, so the boundary belongs to the second tone rather
        // than to the suppression. The burst test above steps over it in both
        // directions and never lands on it.
        //
        // Named rather than written as a number, so it keeps meaning the
        // boundary if the constant moves.
        let player = EarconPlayer::new();
        let start = std::time::Instant::now();
        assert!(player.play_at(Event::MisspelledWord.tone(), start));
        assert!(player.play_at(Event::MisspelledWord.tone(), start + EARCON_GAP));
    }

    #[test]
    fn test_the_gap_between_tones_holds_when_the_clock_is_the_real_one() {
        // Everything above tests `play_at`, and nothing outside this file
        // calls it: the application calls `play`, which is the same decision
        // with the clock read from the system. This says that reading the
        // clock keeps the decision rather than replacing it.
        //
        // Two adjacent statements cannot be a tenth of a second apart, so the
        // second call is inside the gap. On Windows the first one really does
        // sound, which is why the shortest tone in the set is the one used;
        // the suite already beeps for the burst test above.
        let player = EarconPlayer::new();
        assert!(player.play(Event::MisspelledWord.tone()));
        assert!(!player.play(Event::MisspelledWord.tone()));
    }

    #[test]
    fn test_nothing_is_signalled_by_sound_alone() {
        // A sound with no written equivalent is invisible to a deaf-blind user
        // and meaningless to anyone who has not learned it yet.
        //
        // The channels are named here rather than asked through
        // `carries_text`. Calling the predicate under test as its own witness
        // made this assertion pass for any answer that predicate gave.
        let mut settings = FeedbackSettings::default();
        settings.set_channel_enabled(Channel::Earcon, true);
        for event in Event::ALL {
            settings.set_event_channels(event, set(&[Channel::Earcon]));
            let channels = settings.channels_for(event);
            assert!(
                channels
                    .iter()
                    .any(|c| matches!(c, Channel::Speech | Channel::Braille | Channel::Visual)),
                "{:?} would have been sound only",
                event
            );
        }
    }

    #[test]
    fn test_an_event_set_to_sound_only_still_gets_a_written_channel() {
        // The same rule with the answer written out, so a change that stops
        // adding the written channel back cannot pass by agreeing with itself.
        let mut settings = FeedbackSettings::default();
        settings.set_channel_enabled(Channel::Earcon, true);
        settings.set_event_channels(Event::NewMail, set(&[Channel::Earcon]));
        assert_eq!(
            settings.channels_for(Event::NewMail),
            set(&[Channel::Earcon, Channel::Braille])
        );
    }

    #[test]
    fn test_the_written_channel_added_back_is_braille_before_the_status_line() {
        // Which one is added back is a deliberate order and not an accident of
        // how the enum is written: braille first because it is the quietest
        // way to say something to somebody who is already reading it, then the
        // status line, then speech. With braille switched off the status line
        // is what stands in.
        let mut settings = FeedbackSettings::default();
        settings.set_channel_enabled(Channel::Earcon, true);
        settings.set_channel_enabled(Channel::Braille, false);
        settings.set_event_channels(Event::NewMail, set(&[Channel::Earcon]));
        assert_eq!(
            settings.channels_for(Event::NewMail),
            set(&[Channel::Earcon, Channel::Visual])
        );
    }

    #[test]
    fn test_choosing_channels_for_one_event_leaves_the_others_alone() {
        // Somebody who configures two events keeps both. The existing
        // round-trip test only ever sets one, so nothing said this.
        let mut settings = FeedbackSettings::default();
        settings.set_event_channels(Event::NewMail, set(&[Channel::Speech]));
        settings.set_event_channels(Event::SendFailed, set(&[Channel::Braille]));
        assert_eq!(
            settings.channels_for(Event::NewMail),
            set(&[Channel::Speech])
        );
        assert_eq!(
            settings.channels_for(Event::SendFailed),
            set(&[Channel::Braille])
        );
    }

    #[test]
    fn test_setting_an_event_twice_replaces_rather_than_stacks() {
        // Changing your mind about one event has to overwrite the old answer.
        // A second entry left behind the first is the one that gets found.
        let mut settings = FeedbackSettings::default();
        settings.set_event_channels(Event::NewMail, set(&[Channel::Speech]));
        settings.set_event_channels(Event::NewMail, set(&[Channel::Braille]));
        assert_eq!(
            settings.channels_for(Event::NewMail),
            set(&[Channel::Braille])
        );
    }

    #[test]
    fn test_switching_one_channel_off_survives_a_restart_on_its_own() {
        // What gets stored when somebody unticks the speech box and changes
        // nothing else. Thrown away, they switch speech off again every
        // launch and never work out why it comes back.
        let restored = FeedbackSettings::from_stored("off=speech");
        assert!(!restored.is_channel_enabled(Channel::Speech));
        // Earcons are off in the default and on in what was stored, so this is
        // the assertion that says the stored value was kept rather than
        // quietly replaced by the default.
        assert!(restored.is_channel_enabled(Channel::Earcon));
    }

    #[test]
    fn test_a_stored_choice_for_one_event_survives_on_its_own() {
        // The same discard the other way round: a per-event choice with no
        // switched-off channels beside it.
        let restored = FeedbackSettings::from_stored("thread_landed=braille");
        assert_eq!(
            restored.channels_for(Event::ThreadLanded),
            set(&[Channel::Braille])
        );
    }

    #[test]
    fn test_no_two_events_say_the_same_words() {
        // The written form is what a deaf-blind user gets, and two events that
        // read alike are one event as far as they are concerned. Nothing in
        // the mutation run can reach a string, so this is written out.
        let mut seen: Vec<&str> = Vec::new();
        for event in Event::ALL {
            let text = event.text();
            assert!(
                !seen.contains(&text),
                "{:?} says the same as an earlier event: {}",
                event,
                text
            );
            seen.push(text);
        }
    }

    #[test]
    fn test_every_event_has_its_own_stored_name() {
        // These are the names somebody's preferences are filed under. Two
        // events sharing one moves their settings to the wrong event with
        // nothing said about it.
        let mut seen: Vec<&str> = Vec::new();
        for event in Event::ALL {
            let key = event.key();
            assert!(!seen.contains(&key), "{:?} reuses the name {}", event, key);
            seen.push(key);
        }
        let mut channel_names: Vec<&str> = Vec::new();
        for channel in Channel::ALL {
            let key = channel.key();
            assert!(
                !channel_names.contains(&key),
                "{:?} reuses the name {}",
                channel,
                key
            );
            channel_names.push(key);
        }
    }

    #[test]
    fn test_a_stored_name_never_contains_the_characters_that_separate_them() {
        // `to_stored` joins groups with a comma, names to values with an
        // equals sign, and values to each other with a plus. A name carrying
        // any of those cannot be read back.
        for name in Event::ALL
            .into_iter()
            .map(|e| e.key())
            .chain(Channel::ALL.into_iter().map(|c| c.key()))
        {
            for separator in [',', '=', '+'] {
                assert!(
                    !name.contains(separator),
                    "{} carries the {} that separates stored settings",
                    name,
                    separator
                );
            }
        }
    }

    #[test]
    fn test_sound_only_is_allowed_when_the_user_asked_for_it() {
        // Someone who has switched every text channel off has said what they
        // want. The rule protects people; it does not overrule them.
        let mut settings = FeedbackSettings::default();
        settings.set_channel_enabled(Channel::Earcon, true);
        for channel in [Channel::Speech, Channel::Braille, Channel::Visual] {
            settings.set_channel_enabled(channel, false);
        }
        assert_eq!(
            settings.channels_for(Event::NewMail),
            set(&[Channel::Earcon])
        );
    }

    #[test]
    fn test_a_disabled_channel_is_never_used() {
        let mut settings = FeedbackSettings::default();
        settings.set_channel_enabled(Channel::Speech, false);
        for event in Event::ALL {
            assert!(
                !settings.channels_for(event).contains(&Channel::Speech),
                "{:?} still spoke",
                event
            );
        }
    }

    #[test]
    fn test_earcons_are_off_until_someone_turns_them_on() {
        // An application that starts making noises nobody asked for is one
        // people mute permanently before learning what the sounds mean.
        let settings = FeedbackSettings::default();
        assert!(!settings.is_channel_enabled(Channel::Earcon));
        assert!(
            !settings
                .channels_for(Event::NewMail)
                .contains(&Channel::Earcon)
        );
    }

    #[test]
    fn test_braille_survives_speech_being_switched_off() {
        // The case that matters most: a deaf-blind user reading braille, who
        // does not want the speech and cannot hear the earcon.
        let mut settings = FeedbackSettings::default();
        settings.set_channel_enabled(Channel::Speech, false);
        let channels = settings.channels_for(Event::SendFailed);
        assert!(channels.contains(&Channel::Braille));
        assert!(!channels.contains(&Channel::Speech));
    }

    #[test]
    fn test_every_event_has_its_own_tone() {
        // An earcon that cannot be told apart from its sibling carries no
        // information at all.
        let mut seen: Vec<Tone> = Vec::new();
        for event in Event::ALL {
            let tone = event.tone();
            assert!(
                !seen.contains(&tone),
                "{:?} sounds the same as an earlier event",
                event
            );
            assert!(
                tone.hertz >= 200 && tone.hertz <= 2000,
                "{:?} out of range",
                event
            );
            assert!(
                tone.millis > 0 && tone.millis <= 300,
                "{:?} too long",
                event
            );
            seen.push(tone);
        }
    }

    #[test]
    fn test_the_reminder_tone_is_separated_from_new_mail_in_both_pitch_and_length() {
        // A reminder used to borrow the new mail tone, so the two facts made
        // one sound. Written as separation rather than as fixed numbers, so a
        // listening pass can move the reminder inside the envelope without
        // fighting a test.
        //
        // What this checks is that the numbers are far apart. Whether an ear
        // separates them, at the volume and on the speakers somebody actually
        // has, is a listening question and no test here settles it.
        let reminder = Event::Reminder.tone();
        for arrival in [Event::NewMail, Event::MessageSent] {
            let other = arrival.tone();
            let (high, low) = if reminder.hertz > other.hertz {
                (reminder.hertz, other.hertz)
            } else {
                (other.hertz, reminder.hertz)
            };
            assert!(
                high * 10 >= low * 13,
                "{:?} at {} Hz sits too close to the reminder at {} Hz",
                arrival,
                other.hertz,
                reminder.hertz
            );
            assert!(
                reminder.millis.abs_diff(other.millis) >= 80,
                "{:?} at {} ms is too near the reminder's {} ms",
                arrival,
                other.millis,
                reminder.millis
            );
        }
    }

    #[test]
    fn test_each_channel_carries_its_own_wording() {
        // The wording lives on the channel so that the tick somebody puts on a
        // line cannot land on a different channel. Before this, the labels sat
        // in one array in the settings dialog and the channels in another, and
        // the only thing pairing them was that both happened to be written in
        // the same order.
        //
        // The answers are written out rather than asked of the code, because a
        // test that asks the code under test for its own expected answer
        // passes whatever that answer is. That mistake is already recorded
        // higher up this file.
        //
        // The marker is taken out here with a plain replace rather than by
        // calling the code that strips it, so this test does not depend on
        // that code being right. It sits in the middle of a word for the
        // status line, which is why it has to come out at all.
        let wording = |channel: Channel| channel.setting_label().replace('&', "");
        assert!(wording(Channel::Speech).contains("Speak events"));
        assert!(wording(Channel::Braille).contains("braille display"));
        assert!(wording(Channel::Earcon).contains("short sound"));
        assert!(wording(Channel::Visual).contains("status bar"));
    }

    #[test]
    fn test_the_name_passed_for_a_channel_checkbox_drops_the_accelerator_marker() {
        // "Passed", not "announced". `set_accessible_name` writes MSAA, and
        // this project has already shipped sixteen names that were set and
        // never heard, so nothing here claims anything downstream of that
        // call. What this checks is the string handed to it.
        for channel in Channel::ALL {
            let name =
                crate::presentation::accessibility::names::name_from_label(channel.setting_label());
            assert!(!name.is_empty(), "{:?} has no name to pass", channel);
            assert!(
                !name.contains('&'),
                "{:?} would be named with the accelerator marker still in it",
                channel
            );
        }
    }

    #[test]
    fn test_every_event_has_a_written_equivalent() {
        for event in Event::ALL {
            assert!(!event.text().trim().is_empty(), "{:?} has no text", event);
        }
    }

    #[test]
    fn test_failures_outrank_arrivals() {
        // Missing "new mail" costs a moment. Missing "message not sent" costs
        // the message.
        assert!(Event::SendFailed.priority() > Event::NewMail.priority());
        assert!(Event::ConnectionLost.priority() > Event::SyncComplete.priority());
        assert!(Event::NewMail.priority() > Event::ThreadLanded.priority());
    }

    #[test]
    fn test_settings_survive_a_round_trip() {
        let mut settings = FeedbackSettings::default();
        settings.set_channel_enabled(Channel::Earcon, true);
        settings.set_channel_enabled(Channel::Speech, false);
        settings.set_event_channels(
            Event::ThreadLanded,
            set(&[Channel::Earcon, Channel::Braille]),
        );

        let restored = FeedbackSettings::from_stored(&settings.to_stored());
        assert_eq!(restored, settings);
    }

    #[test]
    fn test_unreadable_settings_fall_back_to_the_default() {
        let restored = FeedbackSettings::from_stored("nonsense");
        assert_eq!(restored, FeedbackSettings::default());
    }

    #[test]
    fn test_stored_settings_ignore_names_they_do_not_recognise() {
        // Forward compatibility: a newer version may know an event this build
        // does not, and that must not lose the rest.
        let restored =
            FeedbackSettings::from_stored("off=earcon,invented_event=speech,thread_landed=braille");
        assert!(!restored.is_channel_enabled(Channel::Earcon));
        assert_eq!(
            restored.channels_for(Event::ThreadLanded),
            set(&[Channel::Braille])
        );
    }

    #[test]
    fn test_an_event_with_no_channels_at_all_signals_nothing() {
        // Not a panic and not a silent fallback to speech. If the user
        // switched everything off for one event, that event is off.
        let mut settings = FeedbackSettings::default();
        settings.set_event_channels(Event::EdgeOfList, BTreeSet::new());
        assert!(settings.channels_for(Event::EdgeOfList).is_empty());
    }
}

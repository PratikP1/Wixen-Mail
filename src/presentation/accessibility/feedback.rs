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
}

impl Event {
    /// Every event, so settings and tests can cover the whole set.
    pub const ALL: [Event; 9] = [
        Event::ThreadLanded,
        Event::EdgeOfList,
        Event::NewMail,
        Event::MessageSent,
        Event::SendFailed,
        Event::ConnectionLost,
        Event::ConnectionRestored,
        Event::SyncComplete,
        Event::ActionRefused,
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
        }
    }

    /// How hard the announcement queue should hold on to this.
    ///
    /// Failures outrank arrivals: missing "new mail" costs a moment, missing
    /// "message not sent" costs the message.
    pub fn priority(&self) -> Priority {
        match self {
            Event::SendFailed | Event::ConnectionLost => Priority::Urgent,
            Event::ActionRefused | Event::ConnectionRestored => Priority::High,
            Event::NewMail | Event::MessageSent | Event::SyncComplete => Priority::Normal,
            Event::ThreadLanded | Event::EdgeOfList => Priority::Low,
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
    pub fn carries_text(&self) -> bool {
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
    pub fn set_event_channels(&mut self, event: Event, channels: BTreeSet<Channel>) {
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
    pub fn play_at(&self, tone: Tone, now: std::time::Instant) -> bool {
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
    fn test_nothing_is_signalled_by_sound_alone() {
        // A sound with no written equivalent is invisible to a deaf-blind user
        // and meaningless to anyone who has not learned it yet.
        let mut settings = FeedbackSettings::default();
        settings.set_channel_enabled(Channel::Earcon, true);
        for event in Event::ALL {
            settings.set_event_channels(event, set(&[Channel::Earcon]));
            let channels = settings.channels_for(event);
            assert!(
                channels.iter().any(Channel::carries_text),
                "{:?} would have been sound only",
                event
            );
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

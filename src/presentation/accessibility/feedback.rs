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
use rodio::Source;
use rodio::source::SineWave;
use std::collections::BTreeSet;
use std::fmt;
use std::time::Duration;

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
    /// The cursor landed on a message that carries an attachment.
    ///
    /// The same shape as [`Event::ThreadLanded`]: a fact a sighted user gets
    /// at a glance from a paperclip icon, with nothing before this giving an
    /// equally fast signal without reading the row.
    HasAttachment,
    /// An account's sign-in needs attention: an expired token, a revoked
    /// authorisation, credentials that were never finished.
    ///
    /// Deliberately not [`Event::ConnectionLost`]. A dropped connection asks
    /// somebody to wait; this asks them to act, and conflating the two tells
    /// the wrong story about what to do next.
    AccountNeedsAttention,
    /// A toggle or a small action completed the way it was asked to.
    ///
    /// One shared event for flag/unflag, mark done, pin, and anything else
    /// like them, rather than one event per toggle. Five toggle actions
    /// could easily become five near-identical "did it" tones, which is
    /// exactly the failure this module's own rule against an open "signal
    /// this string" call exists to prevent.
    Confirmed,
    /// A search or a filter completed and matched nothing.
    ///
    /// Distinct from [`Event::SyncComplete`]'s neutral "an operation
    /// finished": coming up empty is a meaningfully different fact, gentle
    /// rather than alarming, and its own tone means somebody does not have
    /// to listen to a whole sentence to know a search came back empty.
    NothingFound,
}

impl Event {
    /// Every event, so settings and tests can cover the whole set.
    pub const ALL: [Event; 16] = [
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
        Event::HasAttachment,
        Event::AccountNeedsAttention,
        Event::Confirmed,
        Event::NothingFound,
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
            Event::HasAttachment => "has_attachment",
            Event::AccountNeedsAttention => "account_needs_attention",
            Event::Confirmed => "confirmed",
            Event::NothingFound => "nothing_found",
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
            Event::HasAttachment => "Has attachment",
            Event::AccountNeedsAttention => "Sign-in needs attention",
            Event::Confirmed => "Confirmed",
            Event::NothingFound => "Nothing found",
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
            // An account needing re-authorisation sits here too: a dropped
            // connection asks somebody to wait, this asks them to act, and
            // the one that needs a person to do something should not lose a
            // race to the one that does not.
            Event::SendFailed
            | Event::ConnectionLost
            | Event::UnsafeMessage
            | Event::Reminder
            | Event::AccountNeedsAttention => Priority::Urgent,
            Event::ActionRefused | Event::ConnectionRestored => Priority::High,
            Event::NewMail
            | Event::MessageSent
            | Event::SyncComplete
            | Event::Confirmed
            | Event::NothingFound => Priority::Normal,
            Event::ThreadLanded
            | Event::EdgeOfList
            | Event::MisspelledWord
            | Event::HasAttachment => Priority::Low,
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
            // first, and a reminder is the wrong event to put up there.
            //
            // These are proposed numbers. Whether the two are tellable apart
            // by ear is a listening pass, and the test beside this one is
            // written as separation so that pass can move them.
            Event::Reminder => Tone::new(1320, 200),
            // A third short navigation tick, alongside ThreadLanded and
            // EdgeOfList, pitched clearly apart from both so three ticks the
            // same length still tell apart by ear alone.
            Event::HasAttachment => Tone::new(1046, 40),
            // Deliberately not in the 220-330 Hz failure band SendFailed,
            // ConnectionLost, ActionRefused and UnsafeMessage already
            // crowd: this is not a failure, it is a specific, actionable
            // fact, and sounding like one more failure tone would bury it
            // among them rather than stand apart.
            Event::AccountNeedsAttention => Tone::new(470, 160),
            // Short and clean, in the same rising-arrival character as
            // MessageSent and NewMail without landing on either one.
            Event::Confirmed => Tone::new(830, 70),
            // Gentle, not alarming: coming up empty is a different fact
            // from a failure, and this sits below SyncComplete's neutral
            // tone rather than beside the failure family under it.
            Event::NothingFound => Tone::new(300, 130),
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

/// How long the start and end of a tone fades rather than jumping straight
/// to full volume or straight to silence.
///
/// A sine wave starting or stopping at full amplitude is an audible click,
/// not part of the note. Long enough to remove it, short enough that even
/// the shortest tone in the set (35ms) still has a real body between the
/// two fades rather than becoming two fades meeting in the middle.
const TONE_EDGE: Duration = Duration::from_millis(3);

/// How loud a tone plays, in `amplify_normalized`'s own 0.0-1.0 range.
///
/// That range is perceptual, not linear, curved so it matches how loudness
/// actually sounds to a human ear rather than how a raw sample multiplier
/// would: 0.5 here is not half volume, it comes out closer to a thirtieth of
/// full linear amplitude. Chosen for a real, comfortably audible peak rather
/// than the midpoint of the number line. A proposed number, not a measured
/// one, the same as every hertz/millis pair on [`Event::tone`]: revisit
/// alongside a real listening pass.
const TONE_VOLUME: f32 = 0.85;

/// A hand-written fade, in place of `Source::fade_in`/`fade_out`.
///
/// Those two ramp from wherever they are inserted in the chain, not from
/// the true end of a bounded source underneath them: chaining `fade_in`
/// straight into `fade_out` fades the first instant twice, at the same
/// time, and then holds silence for everything after, because `fade_out`
/// clamps to its own end gain once its own local duration has passed with
/// no idea the tone continues underneath it. Proven by three of the tests
/// beside this file's tone tests going red for exactly that reason before
/// this replaced them.
///
/// A tone's own total length is already known here, in a way it might not
/// be to a general-purpose combinator, which is what makes counting real
/// samples against it the simpler and the correct answer.
struct Faded<I> {
    input: I,
    edge_samples: u64,
    total_samples: u64,
    position: u64,
}

impl<I: Iterator<Item = f32>> Iterator for Faded<I> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let sample = self.input.next()?;
        let from_start = self.position;
        let from_end = self.total_samples.saturating_sub(self.position + 1);
        let gain =
            from_start.min(from_end).min(self.edge_samples) as f32 / self.edge_samples as f32;
        self.position += 1;
        Some(sample * gain.clamp(0.0, 1.0))
    }
}

impl<I: Source<Item = f32>> Source for Faded<I> {
    fn current_span_len(&self) -> Option<usize> {
        self.input.current_span_len()
    }

    fn channels(&self) -> rodio::ChannelCount {
        self.input.channels()
    }

    fn sample_rate(&self) -> rodio::SampleRate {
        self.input.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        self.input.try_seek(pos)
    }
}

/// The rate every tone here is produced at, in samples per second.
///
/// `SineWave` is fixed at 48kHz mono. This was a `const` inside `sound_for`
/// while its own doc comment said every caller gets the rate from here rather
/// than assuming it, which no caller could: it was not in scope for one. It
/// is now, and `EarconPlayer::detached` builds its mixer at this rate so a
/// test reads back the samples that were written rather than resampled ones.
const TONE_SAMPLE_RATE: u32 = 48_000;

/// A tone as a real sound rather than a fact about frequency and length.
///
/// `SineWave` is fixed at 48kHz mono, which is why nothing here takes a
/// sample rate as a parameter: there is only the one, and every caller gets
/// it from `TONE_SAMPLE_RATE` rather than assuming it.
fn sound_for(tone: Tone) -> impl Source<Item = f32> {
    const SAMPLE_RATE: u64 = TONE_SAMPLE_RATE as u64;
    let total_samples = SAMPLE_RATE * tone.millis as u64 / 1000;
    let edge_samples = (SAMPLE_RATE * TONE_EDGE.as_millis() as u64 / 1000).max(1);
    Faded {
        input: SineWave::new(tone.hertz as f32)
            .take_duration(Duration::from_millis(tone.millis as u64))
            .amplify_normalized(TONE_VOLUME),
        edge_samples,
        total_samples,
        position: 0,
    }
}

/// Plays earcons, one at a time and never faster than the ear can separate.
///
/// Guardrail: feedback must be bounded. A syncing mailbox can raise the same
/// event forty times a second, and forty overlapping tones is not information,
/// it is noise that drives people to switch sound off for good.
pub struct EarconPlayer {
    last_played: std::sync::Mutex<Option<std::time::Instant>>,
    /// The open output device. Held for as long as the player exists and read
    /// by nothing: `rodio` stops playing the moment this is dropped, so there
    /// is nowhere shorter-lived it could live. `None` on a machine with no
    /// audio device to open, and `None` in a test, which plays into a mixer
    /// of its own.
    _device: Option<rodio::MixerDeviceSink>,
    /// Where a sound goes, which is the device's own mixer in the running
    /// program. `None` when there is nothing to play through, and that is
    /// what makes `play` answer false rather than pretend.
    ///
    /// Separate from the device because a mixer does not need one. A test
    /// gets a detached mixer and can read back what was played, which is a
    /// stronger question than the boolean, and it asks that question on a
    /// machine with no sound card. GitHub's Windows runners have none, and
    /// `rodio` 0.22.2 does not fail cleanly there: it opens something and
    /// then faults on the first write, taking the whole test binary with it.
    mixer: Option<rodio::mixer::Mixer>,
}

/// Written out because `rodio::mixer::Mixer` has no `Debug`, and derived
/// `Debug` on a struct holding one does not compile.
impl std::fmt::Debug for EarconPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EarconPlayer")
            .field("last_played", &self.last_played)
            .field("has_somewhere_to_play", &self.mixer.is_some())
            .finish()
    }
}

/// Shortest gap between two earcons.
///
/// Below about a tenth of a second two tones stop being heard as two events.
const EARCON_GAP: std::time::Duration = std::time::Duration::from_millis(120);

/// Says this machine cannot play sound, whatever its audio device claims.
///
/// Set it where a device opens and then does not work, which is not the same
/// as having none and is the case nothing can detect. GitHub's Windows
/// runners ship no audio driver at all (runner-images#6983); `rodio` 0.22.2
/// opens something there regardless and faults on the first write, killing
/// the process. A crash is not a failing test: it takes down every test
/// scheduled after it, and about 3,500 of ours went with it, unreported.
///
/// This is not a way to skip the sound tests. They all still run and all
/// still assert; the sound goes to a mixer nothing listens to instead of to
/// a card. What stops being exercised is `rodio`'s own device handling,
/// which no test here covered anyway.
///
/// Setting it in ordinary use turns earcons off, which is why `new` says so
/// through `tracing` rather than going quiet with no explanation.
const NO_AUDIO: &str = "WIXEN_NO_AUDIO";

fn audio_is_declared_unusable() -> bool {
    std::env::var_os(NO_AUDIO).is_some_and(|value| !value.is_empty())
}

impl Default for EarconPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl EarconPlayer {
    pub fn new() -> Self {
        if audio_is_declared_unusable() {
            tracing::info!(
                "{NO_AUDIO} is set, so earcons play into a mixer with nothing \
                 listening. Every decision about whether to play still runs.",
            );
            return Self::into_the_air().0;
        }
        // A machine with no audio device, or one none of rodio's own
        // fallback attempts could open, gets a silent player rather
        // than a construction failure: a missing sound is a smaller
        // problem than an application that will not start over one.
        let device = rodio::DeviceSinkBuilder::open_default_sink().ok();
        Self {
            last_played: std::sync::Mutex::new(None),
            mixer: device.as_ref().map(|sink| sink.mixer().clone()),
            _device: device,
        }
    }

    /// A player that mixes somewhere real, with no device behind it.
    ///
    /// The returned source is where the sound goes. Dropping it is safe and
    /// is what `new` does: `rodio::mixer::Mixer::add` ignores the send when
    /// nothing is listening, so a player whose source has gone is silent
    /// rather than broken.
    fn into_the_air() -> (Self, rodio::mixer::MixerSource) {
        // Both are `NonZero` in rodio's types, and neither can be zero here:
        // mono, at the rate `sound_for` produces, so a sample read back is
        // the sample that was written rather than a resampled one.
        let channels = std::num::NonZero::new(1).unwrap_or(std::num::NonZero::<u16>::MIN);
        let rate =
            std::num::NonZero::new(TONE_SAMPLE_RATE).unwrap_or(std::num::NonZero::<u32>::MIN);
        let (mixer, source) = rodio::mixer::mixer(channels, rate);
        (
            Self {
                last_played: std::sync::Mutex::new(None),
                _device: None,
                mixer: Some(mixer),
            },
            source,
        )
    }

    /// A player that mixes into the returned source instead of a sound card.
    ///
    /// So a test can ask the stronger question: not whether playing answered
    /// true, but whether a sound really arrived. Every test below that plays
    /// uses this. None of them used to, and on a machine with no sound card
    /// they did not fail, they crashed the process and took about 3,500
    /// unrelated library tests down with them.
    #[cfg(test)]
    fn detached() -> (Self, rodio::mixer::MixerSource) {
        Self::into_the_air()
    }

    /// Play an event's sound, under `scheme`, if enough time has passed
    /// since the last one.
    ///
    /// Returns whether it played, so a caller can tell the difference between
    /// "sounded" and "suppressed" rather than assuming. Also false, the same
    /// as any other silent outcome, when there is no device to play through.
    pub fn play(&self, event: Event, scheme: &super::sound_scheme::SoundScheme) -> bool {
        self.play_at(event, scheme, std::time::Instant::now())
    }

    /// The same decision with the clock passed in, so it can be tested.
    fn play_at(
        &self,
        event: Event,
        scheme: &super::sound_scheme::SoundScheme,
        now: std::time::Instant,
    ) -> bool {
        let Some(mixer) = &self.mixer else {
            return false;
        };
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
        // Fire and forget: rodio plays this on its own thread and returns
        // immediately, unlike the Beep() this replaced, which blocked the
        // calling thread for the tone's own length. A caller that needs the
        // old wait-for-it-to-finish behaviour has to ask for that itself now
        // rather than getting it as a side effect of playing a sound.
        if !self.play_file(mixer, scheme, event) {
            mixer.add(sound_for(event.tone()));
        }
        true
    }

    /// Play a scheme's own file for this event, if it names one and the
    /// file is really there and really a sound. Returns whether it did.
    ///
    /// A scheme naming a file that has since moved, been deleted, or never
    /// was a real sound file falls back to the built-in tone rather than
    /// going silent: a scheme is a preference about which sound plays, not
    /// a promise that every file it names still exists on this machine.
    fn play_file(
        &self,
        mixer: &rodio::mixer::Mixer,
        scheme: &super::sound_scheme::SoundScheme,
        event: Event,
    ) -> bool {
        let super::sound_scheme::SoundSource::File(path) = scheme.source_for(event) else {
            return false;
        };
        let file = match std::fs::File::open(&path) {
            Ok(file) => file,
            Err(err) => {
                tracing::warn!(
                    "{}'s sound for {event:?} could not be opened ({err}); \
                     playing the built-in tone instead",
                    path.display()
                );
                return false;
            }
        };
        match rodio::Decoder::new(std::io::BufReader::new(file)) {
            Ok(decoded) => {
                mixer.add(decoded);
                true
            }
            Err(err) => {
                tracing::warn!(
                    "{} does not look like a sound {event:?} could play ({err}); \
                     playing the built-in tone instead",
                    path.display()
                );
                false
            }
        }
    }
}

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
        let (player, _sound) = EarconPlayer::detached();
        let scheme = super::super::sound_scheme::SoundScheme::generated();
        let start = std::time::Instant::now();
        assert!(player.play_at(Event::NewMail, &scheme, start));
        assert!(!player.play_at(Event::NewMail, &scheme, start));
        assert!(!player.play_at(
            Event::NewMail,
            &scheme,
            start + std::time::Duration::from_millis(50)
        ));
        assert!(player.play_at(
            Event::NewMail,
            &scheme,
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
        let (player, _sound) = EarconPlayer::detached();
        let scheme = super::super::sound_scheme::SoundScheme::generated();
        let start = std::time::Instant::now();
        assert!(player.play_at(Event::MisspelledWord, &scheme, start));
        assert!(player.play_at(Event::MisspelledWord, &scheme, start + EARCON_GAP));
    }

    #[test]
    fn test_a_player_with_no_device_behind_it_still_plays_and_still_suppresses() {
        // What `WIXEN_NO_AUDIO` buys, asked of the thing it produces rather
        // than of the environment variable, because a test that sets one is
        // racing every other test in the process.
        //
        // The point is that declaring the machine deaf changes where a sound
        // goes and nothing else. Both answers still come out of `play_at`, so
        // no assertion anywhere is weakened by setting it, and a decision that
        // stopped working would still be caught.
        let (player, _sound) = EarconPlayer::into_the_air();
        let scheme = super::super::sound_scheme::SoundScheme::generated();
        let start = std::time::Instant::now();

        assert!(
            player.play_at(Event::NewMail, &scheme, start),
            "a player with nowhere to play should still make the decision"
        );
        assert!(
            !player.play_at(Event::NewMail, &scheme, start),
            "and should still suppress the one inside the gap"
        );
    }

    #[test]
    fn test_a_player_that_has_lost_its_listener_is_silent_rather_than_broken() {
        // `new` drops the source when `WIXEN_NO_AUDIO` is set, so this is the
        // shape the running program is in under that flag. It holds because
        // `Mixer::add` ignores the send when nothing is listening. If a
        // future rodio panicked there instead, every earcon on such a machine
        // would take the application down, and this is what would say so.
        let (player, sound) = EarconPlayer::into_the_air();
        let scheme = super::super::sound_scheme::SoundScheme::generated();
        drop(sound);

        assert!(player.play(Event::NewMail, &scheme));
    }

    #[test]
    fn test_playing_puts_a_real_sound_where_the_device_would_hear_it() {
        // Every other test here asks whether `play` answered true, which is a
        // question about the decision and not about the sound. Answering true
        // and adding nothing to the mixer would pass all of them.
        //
        // This reads back what arrived. A tone is not silence, so somewhere in
        // the first tenth of a second there is a sample away from zero.
        let (player, sound) = EarconPlayer::detached();
        let scheme = super::super::sound_scheme::SoundScheme::generated();

        assert!(player.play(Event::NewMail, &scheme));

        let loudest = sound
            .take(TONE_SAMPLE_RATE as usize / 10)
            .fold(0.0f32, |loudest, sample| loudest.max(sample.abs()));
        assert!(
            loudest > 0.0,
            "playing said it played and the mixer got silence"
        );
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
        let (player, _sound) = EarconPlayer::detached();
        let scheme = super::super::sound_scheme::SoundScheme::generated();
        assert!(player.play(Event::MisspelledWord, &scheme));
        assert!(!player.play(Event::MisspelledWord, &scheme));
    }

    #[test]
    fn test_a_scheme_naming_a_file_that_does_not_exist_falls_back_to_the_tone() {
        // A scheme is a preference about which sound plays, not a promise
        // that every file it names still exists on this machine: a moved or
        // deleted file falls back to the built-in tone rather than going
        // silent, and playing still counts as playing.
        let manifest = "name = \"Broken Pack\"\n\n[sounds]\nnew_mail = \"missing.wav\"\n";
        let scheme = super::super::sound_scheme::SoundScheme::from_manifest(
            "broken",
            manifest,
            std::path::Path::new("/nowhere/that/exists"),
        )
        .expect("a valid manifest, even if its file is not real");
        let (player, _sound) = EarconPlayer::detached();
        assert!(player.play(Event::NewMail, &scheme));
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
    fn test_a_channel_displays_as_its_stored_key() {
        // `Display` exists so a channel can be written into a format string
        // the way `key()` already writes it into stored preferences; the two
        // must never drift apart from each other.
        for channel in Channel::ALL {
            assert_eq!(channel.to_string(), channel.key());
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
    fn test_the_four_new_events_each_round_trip_their_own_key() {
        // The stored-preference identifier is what a setting is filed under,
        // so a key that does not round trip through from_key silently
        // detaches a new event from anything a user chose for it.
        for (event, key) in [
            (Event::HasAttachment, "has_attachment"),
            (Event::AccountNeedsAttention, "account_needs_attention"),
            (Event::Confirmed, "confirmed"),
            (Event::NothingFound, "nothing_found"),
        ] {
            assert_eq!(event.key(), key);
            assert_eq!(Event::from_key(key), Some(event));
        }
    }

    #[test]
    fn test_account_needs_attention_outranks_a_dropped_connection() {
        // A dropped connection asks somebody to wait; an account that needs
        // re-authorising asks them to act. The one that needs a person to do
        // something is the one that should not lose a race to the other.
        assert!(Event::AccountNeedsAttention.priority() >= Event::ConnectionLost.priority());
    }

    #[test]
    fn test_confirmed_is_not_the_same_fact_as_action_refused() {
        // The two are opposites, not variations: one says the toggle you
        // asked for happened, the other says nothing happened at all.
        // Sharing a priority is fine; sharing a tone or a sentence is not.
        assert_ne!(Event::Confirmed.tone(), Event::ActionRefused.tone());
        assert_ne!(Event::Confirmed.text(), Event::ActionRefused.text());
    }

    #[test]
    fn test_a_tones_own_samples_run_for_its_stated_length() {
        // SineWave is fixed at 48kHz, so a tone's own millis says exactly
        // how many samples it should produce, independent of the fade or
        // the volume layered on top.
        let tone = Tone::new(440, 100);
        let samples: Vec<f32> = sound_for(tone).collect();
        let expected = 48_000 * tone.millis as usize / 1000;
        assert_eq!(
            samples.len(),
            expected,
            "100ms at 48kHz should be {expected} samples, not {}",
            samples.len()
        );
    }

    #[test]
    fn test_a_tones_samples_never_clip() {
        // Amplified, faded, still bounded: a sample outside [-1.0, 1.0] is
        // what a speaker reads as distortion, not a louder tone.
        let tone = Tone::new(660, 90);
        for sample in sound_for(tone) {
            assert!(
                (-1.0..=1.0).contains(&sample),
                "{sample} is outside what a speaker can play cleanly"
            );
        }
    }

    #[test]
    fn test_a_tone_fades_in_rather_than_starting_at_full_volume() {
        // A sine wave starting at full amplitude is a click, not a note.
        // The first sample of a fade-in has to be quieter than a sample
        // safely inside the tone's body.
        let tone = Tone::new(440, 100);
        let samples: Vec<f32> = sound_for(tone).collect();
        let first = samples[0].abs();
        let middle = samples[samples.len() / 2].abs();
        assert!(
            first < middle,
            "the first sample ({first}) is not quieter than the middle one ({middle}), so nothing faded in"
        );
    }

    #[test]
    fn test_a_tone_fades_out_rather_than_stopping_at_full_volume() {
        // The same shape at the other end: the last sample has to be
        // quieter than the middle, or the tone stops on a click instead of
        // trailing off.
        let tone = Tone::new(440, 100);
        let samples: Vec<f32> = sound_for(tone).collect();
        let last = samples[samples.len() - 1].abs();
        let middle = samples[samples.len() / 2].abs();
        assert!(
            last < middle,
            "the last sample ({last}) is not quieter than the middle one ({middle}), so nothing faded out"
        );
    }

    #[test]
    fn test_even_the_shortest_tone_still_has_an_audible_body() {
        // The fade at each edge has to leave something between them, or a
        // short tone becomes two fades meeting in the middle with nothing
        // ever reaching real volume.
        let shortest = Event::ALL
            .iter()
            .map(|e| e.tone())
            .min_by_key(|t| t.millis)
            .expect("at least one event exists");
        let samples: Vec<f32> = sound_for(shortest).collect();
        let peak = samples.iter().fold(0.0_f32, |a, &b| a.max(b.abs()));
        assert!(
            peak > 0.1,
            "the shortest tone ({}ms) never gets louder than {peak}, so its fades ate the whole thing",
            shortest.millis
        );
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

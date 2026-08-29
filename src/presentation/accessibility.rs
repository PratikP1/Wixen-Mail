//! Accessibility layer for screen reader support
//!
//! Provides interfaces for screen readers (NVDA, JAWS, Narrator) and
//! keyboard navigation support.

pub mod announcements;
pub mod automation;
pub mod feedback;
pub mod focus;
pub mod keyboard;
pub mod names;
pub mod screen_reader;
pub mod sound_scheme;
pub mod sound_scheme_import;

use crate::common::Result;

/// Main accessibility manager
pub struct Accessibility {
    screen_reader: screen_reader::ScreenReaderBridge,
    keyboard: keyboard::KeyboardHandler,
    focus: focus::FocusManager,
    announcements: announcements::AnnouncementQueue,
    automation: automation::AutomationStore,
    /// Which channels each event reaches, and the thing that plays the tones.
    feedback: std::sync::Mutex<feedback::FeedbackSettings>,
    earcons: feedback::EarconPlayer,
    /// Which sound plays for each event, when the earcon channel is one of
    /// the ones an event reaches. A separate setting from `feedback`
    /// itself: that decides which channels an event reaches at all, this
    /// decides what the sound channel actually sounds like.
    scheme: std::sync::Mutex<sound_scheme::SoundScheme>,
    /// The most recent event's written form, for the status line to pick up.
    ///
    /// The visual channel has no API of its own here: the window owns the
    /// status bar. Leaving the text where the caller can read it keeps this
    /// layer free of widget handles.
    visual: std::sync::Mutex<Option<String>>,
}

impl Accessibility {
    /// Create a new accessibility instance
    pub fn new() -> Result<Self> {
        Ok(Self {
            screen_reader: screen_reader::ScreenReaderBridge::new()?,
            keyboard: keyboard::KeyboardHandler::new()?,
            focus: focus::FocusManager::new()?,
            announcements: announcements::AnnouncementQueue::new()?,
            automation: automation::AutomationStore::new()?,
            feedback: std::sync::Mutex::new(feedback::FeedbackSettings::default()),
            earcons: feedback::EarconPlayer::new(),
            scheme: std::sync::Mutex::new(sound_scheme::SoundScheme::generated()),
            visual: std::sync::Mutex::new(None),
        })
    }

    /// Initialize accessibility features
    pub fn initialize(&self) -> Result<()> {
        self.keyboard
            .register_shortcut("Ctrl+N", "compose_new_message")?;
        // Not "search_messages" any more: Find searches whichever module is
        // showing. Corrected rather than left, even though nothing reads this
        // map back today, because a wrong fact sitting in the tree is what
        // somebody wiring it up later would believe.
        self.keyboard
            .register_shortcut("Ctrl+F", "search_whatever_is_showing")?;
        self.keyboard.register_shortcut("F1", "open_help")?;
        // Module navigation shortcuts
        self.keyboard
            .register_shortcut("Ctrl+Shift+1", "switch_to_mail")?;
        self.keyboard
            .register_shortcut("Ctrl+Shift+2", "switch_to_contacts")?;
        self.keyboard
            .register_shortcut("Ctrl+Shift+3", "switch_to_calendar")?;
        self.keyboard
            .register_shortcut("Ctrl+Shift+4", "switch_to_reminders")?;
        self.keyboard
            .register_shortcut("Ctrl+Shift+5", "switch_to_tasks")?;
        self.keyboard
            .register_shortcut("Ctrl+Shift+6", "switch_to_notes")?;
        self.register_node(automation::AutomationNode {
            id: "main_window".to_string(),
            parent_id: None,
            role: automation::AutomationRole::Window,
            name: "Wixen Mail".to_string(),
            description: Some("Primary application window".to_string()),
            state: automation::AutomationState {
                enabled: true,
                ..automation::AutomationState::default()
            },
        })?;
        self.register_node(automation::AutomationNode {
            id: "folder_tree".to_string(),
            parent_id: Some("main_window".to_string()),
            role: automation::AutomationRole::List,
            name: "Folders".to_string(),
            description: Some("Folder navigation".to_string()),
            state: automation::AutomationState {
                enabled: true,
                ..automation::AutomationState::default()
            },
        })?;
        // Register module panel nodes
        let modules = [
            ("module_mail", "Mail"),
            ("module_contacts", "Contacts"),
            ("module_calendar", "Calendar"),
            ("module_reminders", "Reminders"),
            ("module_tasks", "Tasks"),
            ("module_notes", "Notes"),
        ];
        for (id, name) in modules {
            self.register_node(automation::AutomationNode {
                id: id.to_string(),
                parent_id: Some("main_window".to_string()),
                role: automation::AutomationRole::Pane,
                name: name.to_string(),
                description: Some(format!("{} module panel", name)),
                state: automation::AutomationState {
                    enabled: true,
                    ..automation::AutomationState::default()
                },
            })?;
        }
        self.set_focus("folder_tree")?;
        // Said out loud, so it is a sentence somebody would say. "Accessibility
        // initialized" told the person hearing it nothing about their mail.
        self.announcements
            .push(announcements::Announcement::interface(
                "Wixen Mail is ready",
                announcements::Priority::Normal,
            ))?;
        self.flush_announcements()?;
        Ok(())
    }

    /// Register or update automation node.
    fn register_node(&self, node: automation::AutomationNode) -> Result<()> {
        let node_id = node.id.clone();
        self.automation.upsert_node(node)?;
        self.screen_reader
            .notify_event(automation::AutomationEvent::NodeAdded(node_id))?;
        Ok(())
    }

    /// Update focus in automation framework and bridge.
    pub fn set_focus(&self, element_id: &str) -> Result<()> {
        self.focus.set_focus(element_id)?;
        self.screen_reader
            .notify_event(automation::AutomationEvent::FocusChanged(
                element_id.to_string(),
            ))?;
        let focus_label = self
            .automation
            .get_node(element_id)?
            .map(|node| node.name)
            .unwrap_or_else(|| element_id.to_string());
        // Focus moves constantly while navigating, so they share a topic:
        // the latest one supersedes the rest instead of queueing behind them.
        self.announcements.push(
            announcements::Announcement::interface(
                format!("Focus moved to {}", focus_label),
                announcements::Priority::Low,
            )
            .with_topic("focus"),
        )?;
        Ok(())
    }

    /// Update state of existing node.
    pub fn update_node_state(
        &self,
        node_id: &str,
        state: automation::AutomationState,
    ) -> Result<()> {
        self.automation.update_state(node_id, state)?;
        self.screen_reader
            .notify_event(automation::AutomationEvent::NodeUpdated(
                node_id.to_string(),
            ))?;
        Ok(())
    }

    /// Play an event's tone on its own, where the written form is elsewhere.
    ///
    /// Everywhere else, nothing is signalled by sound alone: a tone reaches
    /// nobody who cannot hear it, and [`Self::signal`] adds a text channel
    /// rather than let one go out as a noise with no meaning.
    ///
    /// This is the exception and it is narrow. It is for events whose written
    /// equivalent is already there, put there by something better placed to
    /// say it. So far that is two events. A misspelled word, which the browser
    /// engine marks with a real spelling annotation that every screen reader
    /// reads as the caret crosses it; routing that through the announcement
    /// queue as well would speak a word over the top of the screen reader
    /// echoing what was just typed. And a reminder coming due, where the
    /// window that opens names what is due and when, and announces the same
    /// sentence itself; adding this event's own words would put "Reminder"
    /// in front of "Reminder: call the bank".
    ///
    /// A third one has to earn it the same way. An event whose written form
    /// exists only here belongs in [`Self::signal`].
    ///
    /// Obeys the earcon channel setting like every other sound, so somebody who
    /// has sounds off never hears it.
    ///
    /// Returns whether the tone sounded. The player already knows the
    /// difference between sounded and held back by the pace limit, and says so
    /// for the reason written on it: a caller that assumes instead is a caller
    /// that reports a sound nobody made.
    pub fn earcon(&self, event: feedback::Event) -> Result<bool> {
        let channels = match self.feedback.lock() {
            Ok(settings) => settings.channels_for(event),
            Err(_) => return Ok(false),
        };
        if !channels.contains(&feedback::Channel::Earcon) {
            return Ok(false);
        }
        let scheme = self.scheme.lock().map(|s| s.clone()).unwrap_or_default();
        Ok(self.earcons.play(event, &scheme))
    }

    /// Signal an event on whichever channels the user has chosen.
    ///
    /// Callers name the fact, not the medium. That is what keeps the
    /// never-sound-alone rule in one place instead of depending on every call
    /// site remembering it, and what lets someone switch the whole application
    /// from speech to earcons without touching any of this code.
    ///
    /// `detail` is appended to the event's own wording when there is something
    /// specific worth saying, such as which message or how many. Pass an empty
    /// string when the event says it all.
    pub fn signal(&self, event: feedback::Event, detail: &str) -> Result<()> {
        let channels = match self.feedback.lock() {
            Ok(settings) => settings.channels_for(event),
            Err(_) => return Ok(()),
        };
        if channels.is_empty() {
            return Ok(());
        }

        let text = event.text_with(detail);

        if channels.contains(&feedback::Channel::Earcon) {
            let scheme = self.scheme.lock().map(|s| s.clone()).unwrap_or_default();
            self.earcons.play(event, &scheme);
        }
        // Speech and braille both ride the one screen reader notification, so
        // announcing once serves either. Announcing twice would double the
        // speech for anyone who has both.
        if channels.contains(&feedback::Channel::Speech)
            || channels.contains(&feedback::Channel::Braille)
        {
            self.announce_topic(&text, event.priority(), event.key())?;
        }
        if channels.contains(&feedback::Channel::Visual)
            && let Ok(mut visual) = self.visual.lock()
        {
            *visual = Some(text);
        }
        Ok(())
    }

    /// Take the last event's text for the status line, if there is one.
    ///
    /// Taking rather than reading, so the status line shows each event once
    /// and does not redisplay an old one on the next timer tick.
    pub fn take_visual_feedback(&self) -> Option<String> {
        self.visual.lock().ok().and_then(|mut v| v.take())
    }

    /// Read the current feedback preferences.
    pub fn feedback_settings(&self) -> feedback::FeedbackSettings {
        self.feedback.lock().map(|s| s.clone()).unwrap_or_default()
    }

    /// Replace the feedback preferences.
    pub fn set_feedback_settings(&self, settings: feedback::FeedbackSettings) {
        if let Ok(mut current) = self.feedback.lock() {
            *current = settings;
        }
    }

    /// Read which sound scheme is active.
    pub fn sound_scheme(&self) -> sound_scheme::SoundScheme {
        self.scheme.lock().map(|s| s.clone()).unwrap_or_default()
    }

    /// Switch to a different sound scheme.
    pub fn set_sound_scheme(&self, scheme: sound_scheme::SoundScheme) {
        if let Ok(mut current) = self.scheme.lock() {
            *current = scheme;
        }
    }

    /// Queue an announcement about the application and speak what is due now.
    ///
    /// The queue paces itself, so a burst leaves a remainder behind. The UI
    /// timer calls `flush_announcements` to pick that up.
    pub fn announce(&self, text: &str, priority: announcements::Priority) -> Result<()> {
        self.announcements
            .push(announcements::Announcement::interface(text, priority))?;
        self.flush_announcements()
    }

    /// Queue an announcement that supersedes earlier ones on the same topic.
    ///
    /// Use for anything that updates in place, such as a message count that
    /// climbs during a sync. Only the latest value is worth hearing.
    pub fn announce_topic(
        &self,
        text: &str,
        priority: announcements::Priority,
        topic: &str,
    ) -> Result<()> {
        self.announcements
            .push(announcements::Announcement::interface(text, priority).with_topic(topic))?;
        self.flush_announcements()
    }

    /// Queue an announcement that carries something a person typed.
    ///
    /// Spoken like any other and never muted, because it is the application
    /// answering a keystroke rather than mail being read out. Its words are
    /// not written to the log: what somebody is part way through typing into a
    /// recipient line is a person's name, and the log is a file people are
    /// asked to attach to bug reports.
    ///
    /// Carries a topic for the same reason [`Self::announce_topic`] does: this
    /// is answering something typed, so a fast typist produces several and
    /// only the newest is worth hearing.
    pub fn announce_what_was_typed(
        &self,
        text: &str,
        priority: announcements::Priority,
        topic: &str,
    ) -> Result<()> {
        self.announcements.push(
            announcements::Announcement::interface(text, priority)
                .with_topic(topic)
                .not_in_the_log(),
        )?;
        self.flush_announcements()
    }

    /// Read message content aloud. Silenced when content is muted.
    pub fn announce_content(&self, text: &str) -> Result<()> {
        self.announcements
            .push(announcements::Announcement::content(text))?;
        self.flush_announcements()
    }

    /// Register the control that announcements are carried on.
    ///
    /// Called once the main window exists, because the handle does not exist
    /// before that.
    pub fn register_live_region(&self, handle: isize) {
        self.screen_reader.set_live_region(handle);
    }

    /// Stop or resume reading message content aloud.
    ///
    /// Interface announcements keep working, so muting before a screen share
    /// does not also cost the user their error messages.
    pub fn set_content_muted(&self, muted: bool) {
        self.announcements.set_muted(muted);
    }

    /// Whether message content is currently silenced.
    pub fn is_content_muted(&self) -> bool {
        self.announcements.is_muted()
    }

    /// Emit live region update.
    pub fn live_region_update(&self, region_id: &str, text: &str) -> Result<()> {
        self.screen_reader
            .notify_event(automation::AutomationEvent::LiveRegion(
                region_id.to_string(),
                text.to_string(),
            ))?;
        self.screen_reader.announce(text)
    }

    /// Speak whatever the queue will release now, most important first.
    ///
    /// Called both after queueing and from the UI timer, so announcements held
    /// back by the rate limit are picked up rather than stranded.
    pub fn flush_announcements(&self) -> Result<()> {
        for spoken in self.announcements.drain(std::time::Instant::now())? {
            // Logged so a report of silence can be checked against whether
            // anything was ever released to be spoken. Message content is
            // logged by length only: a body read aloud must not be written to
            // a file on disk. So is anything else carrying what a person
            // typed, such as the part of a name being looked up in a
            // recipient line.
            match spoken.in_the_log {
                announcements::InTheLog::TheWords => {
                    tracing::info!(topic = ?spoken.topic, "Speaking: {}", spoken.text)
                }
                announcements::InTheLog::HowManyCharacters => {
                    tracing::info!(
                        topic = ?spoken.topic,
                        "Speaking {} characters that are not written down here",
                        spoken.text.len()
                    )
                }
            }
            self.screen_reader.announce_with(
                &spoken.text,
                match spoken.priority {
                    announcements::Priority::Urgent => screen_reader::Urgency::Urgent,
                    announcements::Priority::High => screen_reader::Urgency::Important,
                    announcements::Priority::Normal | announcements::Priority::Low => {
                        screen_reader::Urgency::Routine
                    }
                },
                spoken.topic.as_deref().unwrap_or(""),
            )?;
        }
        Ok(())
    }

    /// Diagnostic snapshot of automation tree.
    pub fn automation_snapshot(&self) -> Result<Vec<automation::AutomationNode>> {
        self.automation.snapshot()
    }

    /// Return screen reader bridge status.
    pub fn native_bridge_status(&self) -> screen_reader::NativeBridgeStatus {
        self.screen_reader.status()
    }
}

impl Default for Accessibility {
    fn default() -> Self {
        Self::new().unwrap_or(Self {
            screen_reader: screen_reader::ScreenReaderBridge::default(),
            keyboard: keyboard::KeyboardHandler::default(),
            focus: focus::FocusManager::default(),
            announcements: announcements::AnnouncementQueue::default(),
            automation: automation::AutomationStore::default(),
            feedback: std::sync::Mutex::new(feedback::FeedbackSettings::default()),
            earcons: feedback::EarconPlayer::new(),
            scheme: std::sync::Mutex::new(sound_scheme::SoundScheme::generated()),
            visual: std::sync::Mutex::new(None),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every line the code released to the bridge, oldest first.
    ///
    /// This says what was asked for. It is not evidence that anybody heard
    /// anything: only a run with a real screen reader is that, and no
    /// assertion below should be read as claiming otherwise.
    fn lines_released(a11y: &Accessibility) -> Vec<String> {
        a11y.screen_reader
            .events()
            .expect("events")
            .into_iter()
            .filter_map(|event| match event {
                automation::AutomationEvent::LiveRegion(_, text) => Some(text),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn test_the_words_still_go_out_when_speech_is_off_and_braille_is_on() {
        // The case this module was written for. Speech and braille ride one
        // notification, so either one on its own has to release it; requiring
        // both leaves a deaf-blind user with nothing at all when a send fails.
        //
        // Proves the text was released to the bridge, not that a braille
        // display showed it. Only a screen reader run says the second.
        let a11y = Accessibility::new().expect("accessibility");
        let mut settings = a11y.feedback_settings();
        settings.set_channel_enabled(feedback::Channel::Speech, false);
        a11y.set_feedback_settings(settings);

        a11y.signal(feedback::Event::SendFailed, "")
            .expect("signal");

        let released = lines_released(&a11y);
        assert!(
            released.iter().any(|line| line == "Message not sent"),
            "nothing went out with speech off and braille on: {:?}",
            released
        );
    }

    #[test]
    fn test_an_announcement_is_queued_and_released_in_the_one_call() {
        // Forty-odd call sites announce and none of them flush afterwards, so
        // queueing and releasing have to be the same call. Nothing covered
        // this: the ordering test pushes onto the queue directly and goes
        // round the outside of it.
        //
        // Pins that the text reached the bridge. Whether it is spoken is a
        // screen reader question.
        let a11y = Accessibility::new().expect("accessibility");
        a11y.announce("Message moved to Archive", announcements::Priority::Normal)
            .expect("announce");
        assert_eq!(
            a11y.screen_reader
                .last_announcement()
                .expect("last announcement")
                .as_deref(),
            Some("Message moved to Archive")
        );
    }

    #[test]
    fn test_an_event_that_replaces_its_earlier_self_still_says_something() {
        // Every signalled event with a text channel goes out through this
        // call, so if it went quiet the tone and the status line would survive
        // and the words would not. That is the sound-alone failure the module
        // exists to prevent, arriving by the back door.
        let a11y = Accessibility::new().expect("accessibility");
        a11y.announce_topic(
            "12 of 40 messages",
            announcements::Priority::Low,
            "message-count",
        )
        .expect("announce");
        assert_eq!(
            a11y.screen_reader
                .last_announcement()
                .expect("last announcement")
                .as_deref(),
            Some("12 of 40 messages")
        );
    }

    #[test]
    fn test_muting_stops_the_message_text_and_leaves_the_applications_own_words() {
        // The fast mute for private mail: it silences what a message says and
        // nothing else, so muting before a screen share does not also cost
        // somebody their error messages.
        //
        // The lines here are obviously made up. A test fixture must never be
        // a way to get something that looks like real mail into a log file.
        let a11y = Accessibility::new().expect("accessibility");
        a11y.announce_content("first placeholder line")
            .expect("content");
        assert_eq!(
            a11y.screen_reader
                .last_announcement()
                .expect("last announcement")
                .as_deref(),
            Some("first placeholder line")
        );

        a11y.set_content_muted(true);
        a11y.announce_content("second placeholder line")
            .expect("content");
        a11y.announce("Connected", announcements::Priority::Normal)
            .expect("announce");

        let released = lines_released(&a11y);
        assert!(
            !released
                .iter()
                .any(|line| line == "second placeholder line"),
            "muted message text was still released: {:?}",
            released
        );
        assert!(
            released.iter().any(|line| line == "Connected"),
            "mute took the application's own words with it: {:?}",
            released
        );
    }

    #[test]
    fn test_the_mute_switch_remembers_which_way_it_is_set() {
        // The menu item toggles whatever this reports, and the stored
        // preference is put back through it at startup. Stuck at one answer,
        // the menu item inverts nothing and the preference disappears.
        let a11y = Accessibility::new().expect("accessibility");
        assert!(!a11y.is_content_muted());
        a11y.set_content_muted(true);
        assert!(a11y.is_content_muted());
        a11y.set_content_muted(false);
        assert!(!a11y.is_content_muted());
    }

    #[test]
    fn test_a_tone_is_reported_as_suppressed_when_the_sound_channel_is_off() {
        // Sounded and suppressed are different answers, and this used to give
        // the same one for both. A tone the pace limit swallowed, or that the
        // sound channel is switched off for, now says so instead of leaving
        // the caller to assume.
        //
        // The shortest tone in the set, because on Windows the middle call
        // here really does sound.
        let a11y = Accessibility::new().expect("accessibility");
        assert!(
            !a11y
                .earcon(feedback::Event::MisspelledWord)
                .expect("earcon"),
            "sounds are off by default, so nothing should have played"
        );

        let mut settings = a11y.feedback_settings();
        settings.set_channel_enabled(feedback::Channel::Earcon, true);
        a11y.set_feedback_settings(settings);

        assert!(
            a11y.earcon(feedback::Event::MisspelledWord)
                .expect("earcon"),
            "with sounds on the first tone should play"
        );
        assert!(
            !a11y
                .earcon(feedback::Event::MisspelledWord)
                .expect("earcon"),
            "a second tone in the same instant should be held back"
        );
    }

    #[test]
    fn test_the_active_sound_scheme_round_trips() {
        let a11y = Accessibility::new().expect("accessibility");
        assert_eq!(a11y.sound_scheme(), sound_scheme::SoundScheme::generated());

        let manifest = "name = \"Soft Chimes\"\n";
        let custom = sound_scheme::SoundScheme::from_manifest(
            "soft-chimes",
            manifest,
            std::path::Path::new("/x"),
        )
        .expect("a valid manifest");
        a11y.set_sound_scheme(custom.clone());
        assert_eq!(a11y.sound_scheme(), custom);
    }

    #[test]
    fn test_an_earcon_still_plays_when_the_active_schemes_file_is_missing() {
        // Proves the scheme actually reaches EarconPlayer rather than sitting
        // unused: a scheme naming a file that is not really there still
        // counts as played, because the fallback to the built-in tone is
        // inside play() itself, not a second decision this layer has to make.
        let a11y = Accessibility::new().expect("accessibility");
        let mut settings = a11y.feedback_settings();
        settings.set_channel_enabled(feedback::Channel::Earcon, true);
        a11y.set_feedback_settings(settings);

        let manifest = "name = \"Broken Pack\"\n\n[sounds]\nmisspelled_word = \"missing.wav\"\n";
        let scheme = sound_scheme::SoundScheme::from_manifest(
            "broken",
            manifest,
            std::path::Path::new("/nowhere/that/exists"),
        )
        .expect("a valid manifest, even if its file is not real");
        a11y.set_sound_scheme(scheme);

        assert!(
            a11y.earcon(feedback::Event::MisspelledWord)
                .expect("earcon"),
            "a missing file should still fall back to the tone rather than go silent"
        );
    }

    #[test]
    fn test_nothing_said_at_startup_is_wording_from_inside_the_program() {
        // Heard, not read. A line about something being initialized tells the
        // person hearing it nothing about their mail, and it is exactly the
        // sort of phrase this project's rule on machine words is there to keep
        // out of the ones people hear.
        //
        // This pins what the code asks to say. Whether anybody hears it is a
        // separate question, and a real one: the window that carries
        // announcements is registered after this runs.
        let a11y = Accessibility::new().expect("accessibility");
        a11y.initialize().expect("initialize");
        let released = lines_released(&a11y);
        assert!(
            released.iter().any(|line| line == "Wixen Mail is ready"),
            "startup released {:?}",
            released
        );
        assert!(
            !released
                .iter()
                .any(|line| line.to_lowercase().contains("initial")),
            "startup released {:?}",
            released
        );
    }

    #[test]
    fn test_the_first_lines_of_a_session_are_still_waiting_when_the_window_arrives() {
        // The window that carries announcements does not exist until the event
        // loop starts, and this runs before that. Every line the application
        // opened with was going out on a path this codebase records as
        // reporting success and delivering nothing, so the first thing it ever
        // said reached nobody.
        //
        // This pins that the lines are kept until there is somewhere to put
        // them, and that handing over a window empties the queue of them. It
        // does not pin that they are then spoken: whether a live region raised
        // before the frame is shown reaches NVDA is exactly the sort of thing
        // only a screen reader run answers.
        let a11y = Accessibility::new().expect("accessibility");
        a11y.initialize().expect("initialize");

        let waiting = a11y.screen_reader.held();
        assert!(
            waiting.iter().any(|line| line == "Wixen Mail is ready"),
            "the startup line was not waiting for a window: {:?}",
            waiting
        );
        assert!(
            waiting.iter().any(|line| line == "Focus moved to Folders"),
            "where the cursor landed was not waiting for a window: {:?}",
            waiting
        );

        // Zero, deliberately: a real handle would poke whatever window owns it
        // on the machine running the tests. Zero takes the same path these
        // lines take today, so nothing new happens to them here.
        a11y.register_live_region(0);
        assert!(
            a11y.screen_reader.held().is_empty(),
            "lines were still waiting after a window was handed over: {:?}",
            a11y.screen_reader.held()
        );
    }

    #[test]
    fn test_signalling_an_event_reaches_the_visual_channel() {
        // The status line is what someone sees when speech is off, and it is
        // the only channel that survives both mute and a missing screen
        // reader.
        let a11y = Accessibility::new().expect("accessibility");
        a11y.signal(feedback::Event::NewMail, "3 messages")
            .expect("signal");
        assert_eq!(
            a11y.take_visual_feedback().as_deref(),
            Some("New mail, 3 messages")
        );
    }

    #[test]
    fn test_visual_feedback_is_taken_once_and_not_repeated() {
        // A status line that redisplays an old event on every timer tick is
        // saying something happened when nothing did.
        let a11y = Accessibility::new().expect("accessibility");
        a11y.signal(feedback::Event::SyncComplete, "")
            .expect("signal");
        assert!(a11y.take_visual_feedback().is_some());
        assert!(a11y.take_visual_feedback().is_none());
    }

    #[test]
    fn test_a_detail_that_only_repeats_the_events_own_words_is_not_added_twice() {
        // Every arrival of mail read as "New mail, New mail", because the call
        // site passed the event's own words as the detail. Repetition like that
        // reads as a fault in the program, costs twice the braille cells, and
        // is what teaches somebody to switch announcements off, after which
        // they miss the ones that matter.
        //
        // This pins the text the code built. Whether anybody hears it is a
        // separate question that only a screen reader run answers.
        let a11y = Accessibility::new().expect("accessibility");
        a11y.signal(feedback::Event::NewMail, "New mail")
            .expect("signal");
        assert_eq!(a11y.take_visual_feedback().as_deref(), Some("New mail"));
    }

    #[test]
    fn test_an_event_with_no_detail_says_only_its_own_words() {
        let a11y = Accessibility::new().expect("accessibility");
        a11y.signal(feedback::Event::ConnectionLost, "   ")
            .expect("signal");
        assert_eq!(a11y.take_visual_feedback().as_deref(), Some("Disconnected"));
    }

    #[test]
    fn test_switching_the_visual_channel_off_leaves_nothing_to_display() {
        let a11y = Accessibility::new().expect("accessibility");
        let mut settings = a11y.feedback_settings();
        settings.set_channel_enabled(feedback::Channel::Visual, false);
        a11y.set_feedback_settings(settings);

        a11y.signal(feedback::Event::NewMail, "").expect("signal");
        assert!(a11y.take_visual_feedback().is_none());
    }

    #[test]
    fn test_feedback_settings_reports_what_was_just_stored_rather_than_the_default() {
        // Every other test that calls `feedback_settings()` reads it once,
        // right after `Accessibility::new()`, while the stored value still
        // happens to equal the default. None of them would notice this getter
        // ignoring the mutex and handing back `Default::default()` on every
        // call. This one changes the setting first, so default and stored
        // disagree, and only reads the getter after that.
        let a11y = Accessibility::new().expect("accessibility");
        let mut changed = a11y.feedback_settings();
        changed.set_channel_enabled(feedback::Channel::Earcon, true);
        a11y.set_feedback_settings(changed.clone());

        assert_eq!(a11y.feedback_settings(), changed);
        assert_ne!(
            a11y.feedback_settings(),
            feedback::FeedbackSettings::default(),
            "earcons are off by default, so a settings value with them on must not read back as the default"
        );
    }

    #[test]
    fn test_live_region_update_notifies_the_tree_and_announces_the_text() {
        let a11y = Accessibility::new().expect("accessibility");
        a11y.live_region_update("message-count", "12 unread")
            .expect("live_region_update");

        assert_eq!(
            a11y.screen_reader.last_announcement().unwrap().as_deref(),
            Some("12 unread")
        );
        assert!(
            a11y.screen_reader
                .events()
                .unwrap()
                .iter()
                .any(|event| matches!(
                    event,
                    automation::AutomationEvent::LiveRegion(region, text)
                        if region == "message-count" && text == "12 unread"
                )),
            "no LiveRegion automation event was raised"
        );
    }

    #[test]
    fn test_accessibility_creation() {
        let a11y = Accessibility::new();
        assert!(a11y.is_ok());
    }

    #[test]
    fn test_every_node_registered_at_startup_is_enabled() {
        // The main window, the folder tree and every module panel exist for as
        // long as the application runs, so a screen reader asking whether one
        // can be interacted with should always hear yes. `AutomationState`
        // defaults every field to false, `enabled` included, so leaving the
        // field out of any one of the three registrations here would silently
        // report that node as disabled.
        let a11y = Accessibility::new().unwrap();
        a11y.initialize().unwrap();
        let snapshot = a11y.automation_snapshot().unwrap();
        assert!(
            !snapshot.is_empty(),
            "initialize registered no nodes at all"
        );
        for node in &snapshot {
            assert!(node.state.enabled, "{} was registered as disabled", node.id);
        }
    }

    #[test]
    fn test_updating_a_nodes_state_reaches_the_tree_and_tells_the_bridge() {
        // `update_node_state` promises two things: the automation tree carries
        // the new state, and the screen reader bridge hears about the change.
        // Stubbing the function to a bare `Ok(())` would satisfy neither.
        let a11y = Accessibility::new().unwrap();
        a11y.initialize().unwrap();

        let ticked = automation::AutomationState {
            checked: Some(true),
            ..automation::AutomationState::default()
        };
        a11y.update_node_state("folder_tree", ticked.clone())
            .expect("update_node_state");

        let snapshot = a11y.automation_snapshot().unwrap();
        let folder_tree = snapshot
            .iter()
            .find(|n| n.id == "folder_tree")
            .expect("folder_tree was registered by initialize");
        assert_eq!(
            folder_tree.state, ticked,
            "the new state never reached the automation tree"
        );

        assert!(
            a11y.screen_reader
                .events()
                .unwrap()
                .iter()
                .any(|event| matches!(
                    event,
                    automation::AutomationEvent::NodeUpdated(id) if id == "folder_tree"
                )),
            "no NodeUpdated event reached the screen reader bridge"
        );
    }

    #[test]
    fn test_accessibility_initialize() {
        let a11y = Accessibility::new().unwrap();
        assert!(a11y.initialize().is_ok());
        let snapshot = a11y.automation_snapshot().unwrap();
        assert!(snapshot.iter().any(|n| n.id == "main_window"));
        assert!(snapshot.iter().any(|n| n.id == "folder_tree"));
        let compose_shortcut = a11y.keyboard.action_for_key("Ctrl+N").unwrap();
        assert_eq!(compose_shortcut.as_deref(), Some("compose_new_message"));
    }

    #[test]
    fn test_focus_event_and_announce() {
        let a11y = Accessibility::new().unwrap();
        a11y.set_focus("message_list").unwrap();
        a11y.flush_announcements().unwrap();
        assert!(a11y.screen_reader.events().unwrap().iter().any(|event| {
            matches!(
                event,
                automation::AutomationEvent::FocusChanged(id) if id == "message_list"
            )
        }));
    }

    #[test]
    fn test_flush_announcements_priority_order() {
        const GLOBAL_REGION_ID: &str = "global";
        let a11y = Accessibility::new().unwrap();
        a11y.announcements
            .push(announcements::Announcement::interface(
                "normal",
                announcements::Priority::Normal,
            ))
            .unwrap();
        a11y.announcements
            .push(announcements::Announcement::interface(
                "urgent",
                announcements::Priority::Urgent,
            ))
            .unwrap();
        a11y.announcements
            .push(announcements::Announcement::interface(
                "high",
                announcements::Priority::High,
            ))
            .unwrap();

        a11y.flush_announcements().unwrap();

        let spoken: Vec<String> = a11y
            .screen_reader
            .events()
            .unwrap()
            .into_iter()
            .filter_map(|event| match event {
                automation::AutomationEvent::LiveRegion(region, text)
                    if region == GLOBAL_REGION_ID =>
                {
                    Some(text)
                }
                _ => None,
            })
            .collect();
        assert_eq!(spoken, vec!["urgent", "high", "normal"]);
    }
}

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
pub mod platform_bridge;
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
    /// How much is said while mail and the other modules are fetched. Kept
    /// here beside the other routing choices, set at startup and on save,
    /// because the arms that speak a fetch's lines ask this layer rather
    /// than reading the settings file per line.
    how_much_to_say:
        std::sync::Mutex<crate::application::what_is_said_while_fetching::HowMuchToSay>,
    /// Every sentence shown since the status line last took them, for it to
    /// pick up joined, most urgent first, the way the speech said them.
    ///
    /// The visual channel has no API of its own here: the window owns the
    /// status bar. Leaving the text where the caller can read it keeps this
    /// layer free of widget handles.
    visual: std::sync::Mutex<Vec<Shown>>,
    /// How many turns are open. While any is, an announcement is queued and
    /// not released, and the end of the outermost releases what they said as
    /// one line. See [`Self::as_one_turn`].
    turns: std::sync::atomic::AtomicUsize,
}

/// A sentence waiting for the status line, with what decides its place.
#[derive(Debug)]
struct Shown {
    text: String,
    priority: announcements::Priority,
    /// The event it is about, so a later sentence about the same event
    /// replaces it rather than growing the line, as the speech queue's topics
    /// do. `None` for a sentence about no event, such as the player's
    /// complaint, which is kept beside the rest.
    event: Option<feedback::Event>,
}

impl Shown {
    /// An event's sentence, ranked as the event is.
    fn about(event: feedback::Event, text: String) -> Self {
        Self {
            text,
            priority: event.priority(),
            event: Some(event),
        }
    }

    /// A sentence about no event, ranked with the ordinary ones.
    fn about_no_event(text: String) -> Self {
        Self {
            text,
            priority: announcements::Priority::Normal,
            event: None,
        }
    }
}

/// One open turn, closed when it is dropped, whether the work in it finished
/// or panicked: a turn left open would silence everything said after it.
struct OpenTurn<'a>(&'a std::sync::atomic::AtomicUsize);

impl<'a> OpenTurn<'a> {
    /// Open a turn, saying whether it is the outermost one open.
    fn open(turns: &'a std::sync::atomic::AtomicUsize) -> (Self, bool) {
        let already_open = turns.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        (Self(turns), already_open == 0)
    }
}

impl Drop for OpenTurn<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
    }
}

impl From<announcements::Priority> for screen_reader::Urgency {
    /// The queue's priority as the screen reader is told it: urgent is never
    /// to be dropped, high keeps the latest of its kind, and the rest may be
    /// replaced by a newer line on the same topic.
    fn from(priority: announcements::Priority) -> Self {
        match priority {
            announcements::Priority::Urgent => Self::Urgent,
            announcements::Priority::High => Self::Important,
            announcements::Priority::Normal | announcements::Priority::Low => Self::Routine,
        }
    }
}

/// Write one released line to the log, so a report of silence can be checked
/// against whether anything was ever released to be spoken.
///
/// Message content is logged by length only: a body read aloud must not be
/// written to a file on disk. So is anything else carrying what a person
/// typed, such as the part of a name being looked up in a recipient line.
fn log_what_is_said(spoken: &announcements::Spoken) {
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
}

impl From<announcements::Spoken> for screen_reader::Line {
    fn from(spoken: announcements::Spoken) -> Self {
        Self {
            urgency: spoken.priority.into(),
            topic: spoken.topic.unwrap_or_default(),
            text: spoken.text,
        }
    }
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
            how_much_to_say: std::sync::Mutex::new(
                crate::application::what_is_said_while_fetching::HowMuchToSay::default(),
            ),
            visual: std::sync::Mutex::new(Vec::new()),
            turns: std::sync::atomic::AtomicUsize::new(0),
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
        let played = self.earcons.play(event, &scheme);
        if let Some(complaint) = self.earcons.take_complaint() {
            self.show(Shown::about_no_event(complaint));
        }
        Ok(played)
    }

    /// Put a sentence where the status line picks it up, beside the others
    /// shown since it last looked, or in place of an earlier one about the
    /// same event.
    fn show(&self, shown: Shown) {
        if let Ok(mut visual) = self.visual.lock() {
            let same_event = visual
                .iter_mut()
                .find(|earlier| shown.event.is_some() && earlier.event == shown.event);
            match same_event {
                Some(earlier) => *earlier = shown,
                None => visual.push(shown),
            }
        }
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

        let mut shown = channels
            .contains(&feedback::Channel::Visual)
            .then(|| Shown::about(event, text.clone()));
        if channels.contains(&feedback::Channel::Earcon) {
            let scheme = self.scheme.lock().map(|s| s.clone()).unwrap_or_default();
            self.earcons.play(event, &scheme);
            // The player's one sentence for an outage goes to the status bar
            // in place of the event's words, whatever channels the event
            // reaches (#81): the silence is explained where it is noticed,
            // and the event's words still go out below.
            if let Some(complaint) = self.earcons.take_complaint() {
                shown = Some(Shown::about_no_event(complaint));
            }
        }
        // Speech and braille both ride the one screen reader notification, so
        // announcing once serves either. Announcing twice would double the
        // speech for anyone who has both.
        if channels.contains(&feedback::Channel::Speech)
            || channels.contains(&feedback::Channel::Braille)
        {
            self.announce_topic(&text, event.priority(), event.key())?;
        }
        if let Some(shown) = shown {
            self.show(shown);
        }
        Ok(())
    }

    /// Take what was shown since the status line last looked, as one line,
    /// most urgent first and oldest first within an urgency, joined the way
    /// the speech for the same moment is, if anything was shown.
    ///
    /// Taking rather than reading, so the status line shows each event once
    /// and does not redisplay an old one on the next timer tick. It used to
    /// keep the last sentence alone, so a failed send followed by the lost
    /// connection ended on "Disconnected" and the reason never reached a
    /// braille display (audit finding #32).
    pub fn take_visual_feedback(&self) -> Option<String> {
        let mut shown = self
            .visual
            .lock()
            .map(|mut visual| std::mem::take(&mut *visual))
            .unwrap_or_default();
        // A stable sort, so sentences of one urgency stay in the order shown.
        shown.sort_by_key(|one| std::cmp::Reverse(one.priority));
        let line = screen_reader::said_together(shown.iter().map(|one| one.text.as_str()));
        (!line.is_empty()).then_some(line)
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

    /// Read how much is said while things are fetched.
    pub fn how_much_to_say(&self) -> crate::application::what_is_said_while_fetching::HowMuchToSay {
        self.how_much_to_say
            .lock()
            .map(|level| *level)
            .unwrap_or_default()
    }

    /// Replace how much is said while things are fetched.
    pub fn set_how_much_to_say(
        &self,
        level: crate::application::what_is_said_while_fetching::HowMuchToSay,
    ) {
        if let Ok(mut current) = self.how_much_to_say.lock() {
            *current = level;
        }
    }

    /// Queue an announcement about the application and speak what is due now,
    /// or, inside [`Self::as_one_turn`], when the turn ends.
    ///
    /// The queue paces itself, so a burst leaves a remainder behind. The UI
    /// timer calls `flush_announcements` to pick that up.
    pub fn announce(&self, text: &str, priority: announcements::Priority) -> Result<()> {
        self.announcements
            .push(announcements::Announcement::interface(text, priority))?;
        self.release_unless_a_turn_is_open()
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
        self.release_unless_a_turn_is_open()
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
        self.release_unless_a_turn_is_open()
    }

    /// Read message content aloud. Silenced when content is muted.
    pub fn announce_content(&self, text: &str) -> Result<()> {
        self.announcements
            .push(announcements::Announcement::content(text))?;
        self.release_unless_a_turn_is_open()
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

    /// Speak whatever the queue will release now as one line, most important
    /// first.
    ///
    /// Called after queueing outside a turn, at the end of a turn, and from
    /// the UI timer, so announcements held back by the rate limit, or said
    /// inside a turn that is still open while a dialog it opened waits, are
    /// picked up rather than stranded.
    ///
    /// One line because each line ends as one write to the one control a
    /// screen reader reads, so lines handed over one after another are heard
    /// as the last (audit finding #32). Each is still logged on its own, and
    /// the joined line never is.
    pub fn flush_announcements(&self) -> Result<()> {
        let released = self.announcements.drain(std::time::Instant::now())?;
        for spoken in &released {
            log_what_is_said(spoken);
        }
        let lines: Vec<screen_reader::Line> = released.into_iter().map(Into::into).collect();
        match screen_reader::handed_over_together(&lines) {
            Some(line) => self
                .screen_reader
                .announce_with(&line.text, line.urgency, &line.topic),
            None => Ok(()),
        }
    }

    /// Release what is due now, unless a turn is open, in which case its end
    /// releases it with everything else the turn says.
    fn release_unless_a_turn_is_open(&self) -> Result<()> {
        if self.turns.load(std::sync::atomic::Ordering::Acquire) > 0 {
            return Ok(());
        }
        self.flush_announcements()
    }

    /// Run `work` as one turn: everything announced while it runs is said as
    /// one line when it ends, most urgent first.
    ///
    /// For the places where several things are said for one reason: the UI
    /// tick handling every update the workers sent since the last one, and
    /// one menu, accelerator or tray command. A turn inside a turn releases
    /// nothing; the outermost releases once. A turn whose work panics is
    /// closed all the same and releases nothing, so the next announcement is
    /// spoken. While a turn waits on a dialog it opened, the UI timer's own
    /// flush releases what has been said so far, so nothing waits for the
    /// dialog to close.
    pub fn as_one_turn<T>(&self, work: impl FnOnce() -> T) -> T {
        let (open, is_outermost) = OpenTurn::open(&self.turns);
        let answer = work();
        drop(open);
        if is_outermost && let Err(why) = self.flush_announcements() {
            tracing::warn!("What one turn said could not be released: {why}");
        }
        answer
    }

    /// Diagnostic snapshot of automation tree.
    pub fn automation_snapshot(&self) -> Result<Vec<automation::AutomationNode>> {
        self.automation.snapshot()
    }

    /// The last sentence released to the screen reader bridge, for a test in
    /// another module that has no other way to see what was said.
    ///
    /// Proves the words reached the bridge, not that anything spoke them.
    #[cfg(test)]
    pub(crate) fn last_announcement(&self) -> Option<String> {
        self.screen_reader.last_announcement().ok().flatten()
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
            how_much_to_say: std::sync::Mutex::new(
                crate::application::what_is_said_while_fetching::HowMuchToSay::default(),
            ),
            visual: std::sync::Mutex::new(Vec::new()),
            turns: std::sync::atomic::AtomicUsize::new(0),
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
        //
        // The sounds are switched off by hand first rather than relied on
        // being off: they have been on by default since 2026-09-18 (#77),
        // and this test is about the answer with them off, whatever the
        // default is.
        let a11y = Accessibility::new().expect("accessibility");
        let mut settings = a11y.feedback_settings();
        settings.set_channel_enabled(feedback::Channel::Earcon, false);
        a11y.set_feedback_settings(settings);
        assert!(
            !a11y
                .earcon(feedback::Event::MisspelledWord)
                .expect("earcon"),
            "with the sounds off nothing should have played"
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
        // Said together with where the cursor landed since 13.2-22, so the
        // startup sentence is one sentence of the one line released.
        let released = lines_released(&a11y);
        assert!(
            released
                .iter()
                .any(|line| line.split(". ").any(|said| said == "Wixen Mail is ready")),
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

        // One line since 13.2-22: the two are released by one flush, and so
        // joined there rather than at the handover.
        let waiting = a11y.screen_reader.held();
        let says = |sentence: &str| {
            waiting
                .iter()
                .any(|line| line.split(". ").any(|said| said == sentence))
        };
        assert!(
            says("Wixen Mail is ready"),
            "the startup line was not waiting for a window: {:?}",
            waiting
        );
        assert!(
            says("Focus moved to Folders"),
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
    fn test_a_player_that_cannot_open_a_device_puts_its_complaint_where_the_eye_reads() {
        // The tester's earcons went silent after hours and nothing said so
        // (#81). When no output device can be opened, the player's one
        // sentence for the outage goes to the status bar in place of the
        // event's own words, whatever channels the event reaches, so the
        // silence is explained where it is noticed. The event's words still
        // go to the screen reader: the event happened, and only the sound
        // did not.
        let a11y = Accessibility {
            earcons: feedback::EarconPlayer::whose_device_has_gone_for_good(),
            ..Accessibility::new().expect("accessibility")
        };

        a11y.signal(feedback::Event::NewMail, "3 messages")
            .expect("signal");

        let shown = a11y.take_visual_feedback().expect("a sentence for the eye");
        assert!(
            shown.starts_with("The sounds have stopped"),
            "the status bar got {shown:?} rather than the outage"
        );
        assert!(
            !shown.contains("New mail"),
            "the event's words displaced the outage: {shown:?}"
        );
        assert!(
            lines_released(&a11y)
                .iter()
                .any(|line| line == "New mail, 3 messages"),
            "the event's own words were lost with the sound"
        );
        assert_eq!(
            a11y.take_visual_feedback(),
            None,
            "the outage is shown once"
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
        //
        // Speech is the channel switched, because it is on in the default
        // and this test needs a value that differs from it. The sounds used
        // to be that channel until they went on by default too (#77).
        let a11y = Accessibility::new().expect("accessibility");
        let mut changed = a11y.feedback_settings();
        changed.set_channel_enabled(feedback::Channel::Speech, false);
        a11y.set_feedback_settings(changed.clone());

        assert_eq!(a11y.feedback_settings(), changed);
        assert_ne!(
            a11y.feedback_settings(),
            feedback::FeedbackSettings::default(),
            "speech is on by default, so a settings value with it off must not read back as the default"
        );
    }

    #[test]
    fn test_how_much_to_say_reports_what_was_just_set_rather_than_the_default() {
        // The same shape as the feedback settings test above, for the same
        // reason: a getter that ignored its mutex and handed back the
        // default would pass every reading taken before anything was set.
        use crate::application::what_is_said_while_fetching::HowMuchToSay;
        let a11y = Accessibility::new().expect("accessibility");
        let not_the_default = HowMuchToSay::ALL
            .into_iter()
            .find(|level| *level != HowMuchToSay::default())
            .expect("a level other than the default");
        a11y.set_how_much_to_say(not_the_default);

        assert_eq!(a11y.how_much_to_say(), not_the_default);
        assert_ne!(a11y.how_much_to_say(), HowMuchToSay::default());
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
        // One line, most urgent first. Three lines handed over one after
        // another would each be written to the one control the screen reader
        // reads, so only "normal", the least urgent, would still be there to
        // be read (audit finding #32).
        assert_eq!(spoken, vec!["urgent. high. normal"]);
    }

    #[test]
    fn test_lines_released_together_go_out_as_urgent_as_the_most_urgent_with_no_topic() {
        // The joined line cannot be less urgent than the most urgent thing in
        // it, and no one topic is true of it: a later line on "message-count"
        // must not be allowed to replace a failure that rode along with it.
        let a11y = Accessibility::new().expect("accessibility");
        a11y.announcements
            .push(
                announcements::Announcement::interface(
                    "12 of 40 messages",
                    announcements::Priority::Normal,
                )
                .with_topic("message-count"),
            )
            .expect("push");
        a11y.announcements
            .push(announcements::Announcement::interface(
                "Message not sent",
                announcements::Priority::Urgent,
            ))
            .expect("push");

        a11y.flush_announcements().expect("flush");

        assert_eq!(
            a11y.screen_reader.held_as_handed_over(),
            vec![(
                "Message not sent. 12 of 40 messages".to_string(),
                screen_reader::Urgency::Urgent,
                String::new(),
            )]
        );
    }

    #[test]
    fn test_a_line_ending_in_a_question_or_an_exclamation_keeps_its_mark_when_joined() {
        // "Is it sent?. Stop!. Done" reads as a fault and is spoken with a
        // stop where none belongs. A question or an exclamation is already
        // the end of a sentence, so only a space follows it.
        let a11y = Accessibility::new().expect("accessibility");
        for (text, priority) in [
            ("Is it sent?", announcements::Priority::High),
            ("Stop!", announcements::Priority::Normal),
            ("Done.", announcements::Priority::Low),
        ] {
            a11y.announcements
                .push(announcements::Announcement::interface(text, priority))
                .expect("push");
        }

        a11y.flush_announcements().expect("flush");

        assert_eq!(lines_released(&a11y), vec!["Is it sent? Stop! Done"]);
    }

    #[test]
    fn test_one_line_goes_out_exactly_as_it_was_said_and_an_empty_flush_says_nothing() {
        // A pin, green before the join existed: the join is for lines said
        // together, and a line said alone keeps its own full stop, its own
        // urgency and its own topic, so the screen reader may still let the
        // next count replace this one.
        let a11y = Accessibility::new().expect("accessibility");
        a11y.flush_announcements().expect("an empty flush");
        assert!(
            lines_released(&a11y).is_empty(),
            "an empty flush released {:?}",
            lines_released(&a11y)
        );

        a11y.announce_topic(
            "12 of 40 messages.",
            announcements::Priority::Low,
            "message-count",
        )
        .expect("announce");

        assert_eq!(
            a11y.screen_reader.held_as_handed_over(),
            vec![(
                "12 of 40 messages.".to_string(),
                screen_reader::Urgency::Routine,
                "message-count".to_string(),
            )]
        );
    }

    #[test]
    fn test_each_line_joined_is_still_logged_on_its_own_and_message_text_by_its_length() {
        // A pin, green before the join existed. The log is a file people
        // attach to bug reports, so message text read aloud is written there
        // by its length alone. Joining it to an interface line must not
        // smuggle it into the log under the interface line's rule.
        let a11y = Accessibility::new().expect("accessibility");
        let captured = screen_reader::tests::CapturedLogs::default();
        let _listening = captured.as_the_default();

        a11y.announcements
            .push(announcements::Announcement::interface(
                "Connected",
                announcements::Priority::Normal,
            ))
            .expect("push");
        a11y.announcements
            .push(announcements::Announcement::content(
                "first placeholder line",
            ))
            .expect("push");
        a11y.flush_announcements().expect("flush");

        assert!(
            captured.has(tracing::Level::INFO, "Speaking: Connected"),
            "the interface line was not logged on its own: {:?}",
            captured.events()
        );
        assert!(
            captured.has(
                tracing::Level::INFO,
                "Speaking 22 characters that are not written down here"
            ),
            "the message text was not logged by its length: {:?}",
            captured.events()
        );
        assert!(
            !captured
                .events()
                .iter()
                .any(|(_, message)| message.contains("placeholder")),
            "the message text reached the log: {:?}",
            captured.events()
        );
    }

    #[test]
    fn test_a_failed_send_the_lost_connection_and_the_outbox_said_in_one_turn_are_one_line() {
        // The audit's case (#32): a POP account with a wrong outgoing
        // password. The Outbox pass sends its result, the connection change
        // and its closing count back to back, the tick handles all three, and
        // each used to be written over the one before, so only "The Outbox is
        // done" was there to be read and the reason never was.
        let a11y = Accessibility::new().expect("accessibility");

        a11y.as_one_turn(|| {
            a11y.signal(
                feedback::Event::SendFailed,
                "the server refused the password",
            )
            .expect("signal");
            a11y.signal(feedback::Event::ConnectionLost, "")
                .expect("signal");
            a11y.announce(
                "The Outbox is done: 0 sent, 1 not sent.",
                announcements::Priority::Normal,
            )
            .expect("announce");
        });

        assert_eq!(
            lines_released(&a11y),
            vec![
                "Message not sent, the server refused the password. Disconnected. \
                 The Outbox is done: 0 sent, 1 not sent"
            ]
        );
    }

    #[test]
    fn test_the_status_line_shows_the_failure_before_the_lost_connection_not_the_last_alone() {
        // The status bar is what a braille display reads with speech off. It
        // kept the last sentence shown, so the same case ended on
        // "Disconnected" and the reason was lost there too.
        let a11y = Accessibility::new().expect("accessibility");

        a11y.as_one_turn(|| {
            a11y.signal(
                feedback::Event::SendFailed,
                "the server refused the password",
            )
            .expect("signal");
            a11y.signal(feedback::Event::ConnectionLost, "")
                .expect("signal");
        });

        assert_eq!(
            a11y.take_visual_feedback().as_deref(),
            Some("Message not sent, the server refused the password. Disconnected")
        );
    }

    #[test]
    fn test_an_event_shown_again_before_the_status_line_looks_is_shown_once_at_its_latest() {
        // The status line now keeps every sentence shown since it last
        // looked, so it needs the bound the speech queue's topics give it: a
        // tick that hears of new mail three times shows the latest count
        // once, not three counts in a row, however many arrive (guardrail 5).
        // A pin against the last-sentence-only status line, which passed it
        // by keeping one sentence; taken red by hand against the join with no
        // replacement, and held by its guard record.
        let a11y = Accessibility::new().expect("accessibility");
        for detail in ["1 message", "2 messages", "3 messages"] {
            a11y.signal(feedback::Event::NewMail, detail)
                .expect("signal");
        }
        a11y.signal(feedback::Event::ConnectionLost, "")
            .expect("signal");

        assert_eq!(
            a11y.take_visual_feedback().as_deref(),
            Some("Disconnected. New mail, 3 messages")
        );
    }

    #[test]
    fn test_an_announcement_outside_a_turn_is_spoken_at_once() {
        // A pin, green before turns existed. Forty-odd call sites announce
        // from handlers no turn wraps, and each of them must still be said
        // the moment it is made.
        let a11y = Accessibility::new().expect("accessibility");
        a11y.announce("Connected", announcements::Priority::Normal)
            .expect("announce");
        assert_eq!(lines_released(&a11y), vec!["Connected"]);
        a11y.announce("Message moved to Archive", announcements::Priority::Normal)
            .expect("announce");
        assert_eq!(
            lines_released(&a11y),
            vec!["Connected", "Message moved to Archive"]
        );
    }

    #[test]
    fn test_a_turn_whose_work_panics_leaves_the_next_announcement_spoken() {
        // A pin until the hold exists, measured by its guard record. A hold
        // left set by a command that panicked would silence every line after
        // it for the rest of the session, which is worse than the overwrite
        // the hold exists to stop.
        let a11y = Accessibility::new().expect("accessibility");
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            a11y.as_one_turn(|| panic!("a command that fails part way"))
        }));
        assert!(outcome.is_err(), "the work did not panic");

        a11y.announce("After the fault", announcements::Priority::Normal)
            .expect("announce");

        assert_eq!(
            lines_released(&a11y).last().map(String::as_str),
            Some("After the fault")
        );
    }

    #[test]
    fn test_a_turn_inside_a_turn_releases_once_at_the_end_of_the_outer_one() {
        // A command that runs another command is still one thing somebody
        // did, so what both say is one line, and nothing goes out before the
        // outer one ends.
        let a11y = Accessibility::new().expect("accessibility");

        a11y.as_one_turn(|| {
            a11y.announce("Moved to Archive", announcements::Priority::Normal)
                .expect("announce");
            a11y.as_one_turn(|| {
                a11y.announce("Marked as read", announcements::Priority::Normal)
                    .expect("announce");
            });
            assert!(
                lines_released(&a11y).is_empty(),
                "a line went out before the outer turn ended: {:?}",
                lines_released(&a11y)
            );
        });

        assert_eq!(
            lines_released(&a11y),
            vec!["Moved to Archive. Marked as read"]
        );
    }
}

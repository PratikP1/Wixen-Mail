//! Accessibility layer for screen reader support
//!
//! Provides interfaces for screen readers (NVDA, JAWS, Narrator) and
//! keyboard navigation support.

pub mod announcements;
pub mod automation;
pub mod focus;
pub mod keyboard;
pub mod names;
pub mod screen_reader;
pub mod shortcuts;

use crate::common::Result;

/// Main accessibility manager
#[allow(dead_code)]
pub struct Accessibility {
    screen_reader: screen_reader::ScreenReaderBridge,
    keyboard: keyboard::KeyboardHandler,
    focus: focus::FocusManager,
    announcements: announcements::AnnouncementQueue,
    automation: automation::AutomationStore,
    shortcuts: shortcuts::ShortcutManager,
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
            shortcuts: shortcuts::ShortcutManager::new(),
        })
    }

    /// Initialize accessibility features
    pub fn initialize(&self) -> Result<()> {
        self.keyboard
            .register_shortcut("Ctrl+N", "compose_new_message")?;
        self.keyboard
            .register_shortcut("Ctrl+F", "search_messages")?;
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
        self.announcements
            .push(announcements::Announcement::interface(
                "Accessibility initialized",
                announcements::Priority::Normal,
            ))?;
        self.flush_announcements()?;
        Ok(())
    }

    /// Register or update automation node.
    pub fn register_node(&self, node: automation::AutomationNode) -> Result<()> {
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

    /// Read message content aloud. Silenced when content is muted.
    pub fn announce_content(&self, text: &str) -> Result<()> {
        self.announcements
            .push(announcements::Announcement::content(text))?;
        self.flush_announcements()
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
            // a file on disk.
            match spoken.kind {
                announcements::Kind::Interface => {
                    tracing::info!(topic = ?spoken.topic, "Speaking: {}", spoken.text)
                }
                announcements::Kind::Content => {
                    tracing::info!("Speaking message content, {} characters", spoken.text.len())
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

    /// Get shortcut manager
    pub fn shortcuts(&self) -> &shortcuts::ShortcutManager {
        &self.shortcuts
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
            shortcuts: shortcuts::ShortcutManager::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accessibility_creation() {
        let a11y = Accessibility::new();
        assert!(a11y.is_ok());
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

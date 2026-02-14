//! Accessibility layer for screen reader support
//!
//! Provides interfaces for screen readers (NVDA, JAWS, Narrator) and
//! keyboard navigation support.

pub mod announcements;
pub mod automation;
pub mod focus;
pub mod keyboard;
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
        self.keyboard.register_shortcut("Ctrl+N", "compose_new_message")?;
        self.keyboard.register_shortcut("Ctrl+F", "search_messages")?;
        self.keyboard.register_shortcut("F1", "open_help")?;
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
        self.set_focus("folder_tree")?;
        self.announcements
            .announce("Accessibility initialized", announcements::Priority::Normal)?;
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
            .notify_event(automation::AutomationEvent::FocusChanged(element_id.to_string()))?;
        let focus_label = self
            .automation
            .get_node(element_id)?
            .map(|node| node.name)
            .unwrap_or_else(|| element_id.to_string());
        self.announcements.announce(
            &format!("Focus moved to {}", focus_label),
            announcements::Priority::Low,
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
            .notify_event(automation::AutomationEvent::NodeUpdated(node_id.to_string()))?;
        Ok(())
    }

    /// Queue and flush announcement through native bridge.
    pub fn announce(&self, text: &str, priority: announcements::Priority) -> Result<()> {
        self.announcements.announce(text, priority)?;
        self.flush_announcements()
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

    /// Flush queued announcements in priority order.
    pub fn flush_announcements(&self) -> Result<()> {
        while let Some(message) = self.announcements.pop_next()? {
            self.screen_reader.announce(&message)?;
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
}

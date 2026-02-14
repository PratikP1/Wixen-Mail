//! Accessibility layer for screen reader support
//!
//! Provides interfaces for screen readers (NVDA, JAWS, Narrator) and
//! keyboard navigation support.

pub mod announcements;
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
            shortcuts: shortcuts::ShortcutManager::new(),
        })
    }

    /// Initialize accessibility features
    pub fn initialize(&self) -> Result<()> {
        self.keyboard.register_shortcut("Ctrl+N", "compose_new_message")?;
        self.keyboard.register_shortcut("Ctrl+F", "search_messages")?;
        self.keyboard.register_shortcut("F1", "open_help")?;
        self.focus.set_focus("folder_tree")?;
        self.announcements
            .announce("Accessibility initialized", announcements::Priority::Normal)?;
        self.screen_reader.announce("Accessibility initialized")?;
        Ok(())
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
    }
}

//! Screen reader bridge for Windows UI Automation

use crate::common::Result;
use std::sync::Mutex;

/// Bridge to Windows UI Automation for screen readers
pub struct ScreenReaderBridge {
    last_announcement: Mutex<Option<String>>,
}

impl ScreenReaderBridge {
    /// Create a new screen reader bridge
    pub fn new() -> Result<Self> {
        Ok(Self {
            last_announcement: Mutex::new(None),
        })
    }

    /// Announce text to screen reader
    pub fn announce(&self, text: &str) -> Result<()> {
        let mut last = self
            .last_announcement
            .lock()
            .map_err(|_| crate::common::Error::Other("Screen reader lock poisoned".to_string()))?;
        *last = Some(text.to_string());
        Ok(())
    }

    /// Return last announced text (for diagnostics/testing)
    pub fn last_announcement(&self) -> Result<Option<String>> {
        let last = self
            .last_announcement
            .lock()
            .map_err(|_| crate::common::Error::Other("Screen reader lock poisoned".to_string()))?;
        Ok(last.clone())
    }
}

impl Default for ScreenReaderBridge {
    fn default() -> Self {
        Self {
            last_announcement: Mutex::new(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_announce_updates_last_value() {
        let bridge = ScreenReaderBridge::new().unwrap();
        bridge.announce("Hello").unwrap();
        assert_eq!(bridge.last_announcement().unwrap().as_deref(), Some("Hello"));
    }
}

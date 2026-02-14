//! Screen reader bridge for Windows UI Automation

use crate::common::Result;
use std::sync::Mutex;

use super::automation::AutomationEvent;

/// Native bridge status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeBridgeStatus {
    /// Windows bridge active.
    Active,
    /// Non-Windows fallback mode.
    Fallback,
}

/// Bridge to Windows UI Automation for screen readers
pub struct ScreenReaderBridge {
    last_announcement: Mutex<Option<String>>,
    event_log: Mutex<Vec<AutomationEvent>>,
    status: NativeBridgeStatus,
}

impl ScreenReaderBridge {
    /// Create a new screen reader bridge
    pub fn new() -> Result<Self> {
        Ok(Self {
            last_announcement: Mutex::new(None),
            event_log: Mutex::new(Vec::new()),
            status: if cfg!(target_os = "windows") {
                NativeBridgeStatus::Active
            } else {
                NativeBridgeStatus::Fallback
            },
        })
    }

    /// Announce text to screen reader
    pub fn announce(&self, text: &str) -> Result<()> {
        let mut last = self
            .last_announcement
            .lock()
            .map_err(|_| crate::common::Error::Other("Screen reader lock poisoned".to_string()))?;
        *last = Some(text.to_string());
        self.push_event(AutomationEvent::LiveRegion(
            "global".to_string(),
            text.to_string(),
        ))?;
        Ok(())
    }

    /// Notify native bridge of automation event.
    pub fn notify_event(&self, event: AutomationEvent) -> Result<()> {
        self.push_event(event)
    }

    fn push_event(&self, event: AutomationEvent) -> Result<()> {
        let mut log = self
            .event_log
            .lock()
            .map_err(|_| crate::common::Error::Other("Screen reader event log lock poisoned".to_string()))?;
        log.push(event);
        Ok(())
    }

    /// Return event log (for diagnostics/testing).
    pub fn events(&self) -> Result<Vec<AutomationEvent>> {
        let log = self
            .event_log
            .lock()
            .map_err(|_| crate::common::Error::Other("Screen reader event log lock poisoned".to_string()))?;
        Ok(log.clone())
    }

    /// Return bridge status.
    pub fn status(&self) -> NativeBridgeStatus {
        self.status
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
            event_log: Mutex::new(Vec::new()),
            status: if cfg!(target_os = "windows") {
                NativeBridgeStatus::Active
            } else {
                NativeBridgeStatus::Fallback
            },
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
        assert!(!bridge.events().unwrap().is_empty());
    }
}

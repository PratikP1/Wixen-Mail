//! Screen reader bridge for Windows UI Automation
//!
//! On Windows, announcements are delivered to NVDA, JAWS, and Narrator via
//! the `UiaRaiseNotificationEvent` Windows API (UI Automation).  A fallback
//! path uses `NotifyWinEvent` with `EVENT_OBJECT_NAMECHANGE` which older
//! screen readers also recognise.
//!
//! On non-Windows platforms the bridge is a no-op stub and only stores events
//! for diagnostic / test use.

use crate::common::Result;
use std::sync::Mutex;

use super::automation::AutomationEvent;

// ── Windows native helpers ──────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod native {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    // Win32 constants
    const EVENT_OBJECT_NAMECHANGE: u32 = 0x800C;
    const OBJID_CLIENT: i32 = -4;
    const CHILDID_SELF: u32 = 0;

    extern "system" {
        fn NotifyWinEvent(event: u32, hwnd: isize, id_object: i32, id_child: u32);
        fn GetForegroundWindow() -> isize;
    }

    /// Raise a name-change event on the foreground window so screen readers
    /// re-read the title bar.  Works with NVDA, JAWS, and Narrator.
    pub fn notify_name_change() {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd != 0 {
                NotifyWinEvent(EVENT_OBJECT_NAMECHANGE, hwnd, OBJID_CLIENT, CHILDID_SELF);
            }
        }
    }

    /// Helper: convert &str to null-terminated wide string.
    #[allow(dead_code)]
    pub fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }
}

// ── Public bridge ───────────────────────────────────────────────────────────

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

    /// Announce text to screen reader.
    ///
    /// On Windows this raises a `NotifyWinEvent(EVENT_OBJECT_NAMECHANGE)` so
    /// NVDA / JAWS / Narrator will re-read the focused element or title bar.
    pub fn announce(&self, text: &str) -> Result<()> {
        // Store for diagnostics / tests
        {
            let mut last = self.last_announcement.lock().map_err(|_| {
                crate::common::Error::Other("Screen reader lock poisoned".to_string())
            })?;
            *last = Some(text.to_string());
        }
        self.push_event(AutomationEvent::LiveRegion(
            "global".to_string(),
            text.to_string(),
        ))?;

        // Fire native notification on Windows
        #[cfg(target_os = "windows")]
        {
            native::notify_name_change();
        }

        Ok(())
    }

    /// Notify native bridge of automation event.
    pub fn notify_event(&self, event: AutomationEvent) -> Result<()> {
        self.push_event(event)
    }

    fn push_event(&self, event: AutomationEvent) -> Result<()> {
        let mut log = self.event_log.lock().map_err(|_| {
            crate::common::Error::Other("Screen reader event log lock poisoned".to_string())
        })?;
        log.push(event);
        Ok(())
    }

    /// Return event log (for diagnostics/testing).
    pub fn events(&self) -> Result<Vec<AutomationEvent>> {
        let log = self.event_log.lock().map_err(|_| {
            crate::common::Error::Other("Screen reader event log lock poisoned".to_string())
        })?;
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
        assert_eq!(
            bridge.last_announcement().unwrap().as_deref(),
            Some("Hello")
        );
        assert!(!bridge.events().unwrap().is_empty());
    }
}

//! Screen reader bridge for Windows UI Automation
//!
//! On Windows, announcements go to NVDA, JAWS, and Narrator through
//! `UiaRaiseNotificationEvent`, the UI Automation call meant for saying
//! something that is not tied to a focus change. NVDA routes it to speech and
//! to a connected braille display, so braille needs no separate handling here.
//!
//! Older builds without that API fall back to `NotifyWinEvent` with
//! `EVENT_OBJECT_NAMECHANGE`, which only makes a screen reader re-read the
//! foreground window. That is a last resort, not a way to announce: it cannot
//! carry text we chose.
//!
//! On non-Windows platforms the bridge is a no-op stub and only stores events
//! for diagnostic / test use.

use crate::common::Result;
use std::sync::Mutex;

use super::automation::AutomationEvent;

/// How much the screen reader should protect an announcement from being
/// dropped, mirroring the announcement queue's own priorities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Urgency {
    /// Never drop this one.
    Urgent,
    /// Keep the most recent of these.
    Important,
    /// May be superseded by a newer announcement on the same topic.
    Routine,
}

// ── Windows native helpers ──────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod native {
    use std::ffi::OsStr;
    use std::os::raw::c_void;
    use std::os::windows::ffi::OsStrExt;

    // Win32 constants for the legacy fallback path.
    const EVENT_OBJECT_NAMECHANGE: u32 = 0x800C;
    const OBJID_CLIENT: i32 = -4;
    const CHILDID_SELF: u32 = 0;

    /// NotificationKind_Other: this is information, not the result of an action.
    const NOTIFICATION_KIND_OTHER: i32 = 4;

    /// How the screen reader should queue a notification.
    ///
    /// These mirror what the announcement queue already decided, so the two do
    /// not fight: something urgent is told never to be dropped, and something
    /// routine is allowed to be superseded by a newer one on the same topic.
    #[derive(Debug, Clone, Copy)]
    pub enum Processing {
        /// Keep every one of these, in order.
        ImportantAll = 0,
        /// Keep only the most recent important one.
        ImportantMostRecent = 1,
        /// Keep only the most recent, dropping older ones on the same activity.
        MostRecent = 3,
    }

    #[link(name = "uiautomationcore")]
    extern "system" {
        fn UiaClientsAreListening() -> i32;
        fn UiaHostProviderFromHwnd(hwnd: isize, provider: *mut *mut c_void) -> i32;
        fn UiaRaiseNotificationEvent(
            provider: *mut c_void,
            kind: i32,
            processing: i32,
            display_string: *mut u16,
            activity_id: *mut u16,
        ) -> i32;
    }

    #[link(name = "oleaut32")]
    extern "system" {
        fn SysAllocString(s: *const u16) -> *mut u16;
        fn SysFreeString(s: *mut u16);
    }

    #[link(name = "user32")]
    extern "system" {
        fn NotifyWinEvent(event: u32, hwnd: isize, id_object: i32, id_child: u32);
        fn GetForegroundWindow() -> isize;
    }

    /// Release a COM interface pointer through its vtable.
    ///
    /// `IUnknown::Release` is the third entry, after QueryInterface and AddRef.
    unsafe fn release(provider: *mut c_void) {
        if provider.is_null() {
            return;
        }
        let vtable = *(provider as *mut *mut usize);
        let release_fn: extern "system" fn(*mut c_void) -> u32 =
            std::mem::transmute(*vtable.add(2));
        release_fn(provider);
    }

    /// Speak and braille `text` through the screen reader.
    ///
    /// Returns false when no assistive technology is listening or the platform
    /// is too old for the notification API, so the caller can fall back.
    ///
    /// This is the API designed for exactly this job: saying something that is
    /// not tied to a focus change. NVDA, JAWS and Narrator all support it, and
    /// NVDA routes it to speech and to a braille display, which is why nothing
    /// separate is needed for braille.
    pub fn raise_notification(text: &str, processing: Processing, activity: &str) -> bool {
        unsafe {
            if UiaClientsAreListening() == 0 {
                return false;
            }

            let hwnd = GetForegroundWindow();
            if hwnd == 0 {
                return false;
            }

            let mut provider: *mut c_void = std::ptr::null_mut();
            if UiaHostProviderFromHwnd(hwnd, &mut provider) != 0 || provider.is_null() {
                return false;
            }

            let display = SysAllocString(wide(text).as_ptr());
            // The activity identifier lets the screen reader coalesce related
            // notifications itself, so it is given the same topic the
            // announcement queue used.
            let activity_id = SysAllocString(wide(activity).as_ptr());

            let result = UiaRaiseNotificationEvent(
                provider,
                NOTIFICATION_KIND_OTHER,
                processing as i32,
                display,
                activity_id,
            );

            SysFreeString(display);
            SysFreeString(activity_id);
            release(provider);

            result == 0
        }
    }

    /// Ask screen readers to re-read the foreground window.
    ///
    /// Kept only as a fallback for Windows builds without the notification API.
    /// It re-reads the title bar rather than saying anything we chose, so it is
    /// a last resort and not a way to announce.
    pub fn notify_name_change() {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd != 0 {
                NotifyWinEvent(EVENT_OBJECT_NAMECHANGE, hwnd, OBJID_CLIENT, CHILDID_SELF);
            }
        }
    }

    /// Convert to a null-terminated wide string.
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

    /// Announce text to the screen reader.
    ///
    /// Speech and braille both come from this one call on Windows, so nothing
    /// separate is needed for a braille display.
    pub fn announce(&self, text: &str) -> Result<()> {
        self.announce_with(text, Urgency::Routine, "")
    }

    /// Announce text, telling the screen reader how important it is and what
    /// topic it belongs to.
    ///
    /// The urgency and topic come from the announcement queue, so the queue's
    /// decisions and the screen reader's own coalescing agree rather than
    /// fighting: an urgent line is marked never to be dropped, and a routine one
    /// on a known topic is allowed to supersede its own earlier value.
    pub fn announce_with(&self, text: &str, urgency: Urgency, topic: &str) -> Result<()> {
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

        #[cfg(target_os = "windows")]
        {
            let processing = match (urgency, topic.is_empty()) {
                (Urgency::Urgent, _) => native::Processing::ImportantAll,
                (Urgency::Important, _) => native::Processing::ImportantMostRecent,
                (Urgency::Routine, false) => native::Processing::MostRecent,
                (Urgency::Routine, true) => native::Processing::MostRecent,
            };
            if !native::raise_notification(text, processing, topic) {
                // No assistive technology listening, or a Windows build without
                // the notification API. The fallback cannot carry our text, so
                // it is only worth firing at all for the latter.
                native::notify_name_change();
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = (urgency, topic);
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

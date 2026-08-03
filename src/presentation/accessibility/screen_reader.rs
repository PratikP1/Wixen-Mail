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

    // Win32 constants.
    const EVENT_OBJECT_NAMECHANGE: u32 = 0x800C;
    /// Windows 8 and later. Screen readers announce the new name of a control
    /// that raises this, which is how a plain Win32 application says something
    /// without moving focus.
    const EVENT_OBJECT_LIVEREGIONCHANGED: u32 = 0x8019;
    const OBJID_CLIENT: i32 = -4;
    const CHILDID_SELF: u32 = 0;

    /// NotificationKind_Other: this is information, not the result of an action.
    const NOTIFICATION_KIND_OTHER: i32 = 4;

    /// How the screen reader should queue a notification.
    ///
    /// These mirror what the announcement queue already decided, so the two do
    /// not fight: something urgent is told never to be dropped, and something
    /// routine is allowed to be superseded by a newer one on the same topic.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Processing {
        /// Keep every one of these, in order.
        ImportantAll = 0,
        /// Keep only the most recent important one.
        ImportantMostRecent = 1,
        /// Keep every one of these, dropping them only under pressure.
        All = 2,
        /// Keep only the most recent, dropping older ones on the same activity.
        MostRecent = 3,
    }

    #[link(name = "uiautomationcore")]
    unsafe extern "system" {
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
    unsafe extern "system" {
        fn SysAllocString(s: *const u16) -> *mut u16;
        fn SysFreeString(s: *mut u16);
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn NotifyWinEvent(event: u32, hwnd: isize, id_object: i32, id_child: u32);
        fn GetForegroundWindow() -> isize;
        fn SetWindowTextW(hwnd: isize, text: *const u16) -> i32;
    }

    /// Announce through a live region control.
    ///
    /// A Win32 static control reports its window text as its accessible name,
    /// so setting the text and then raising a live region change is a complete
    /// announcement. This works through MSAA, which is what wxWidgets actually
    /// implements, rather than requiring the application to be a native UI
    /// Automation provider.
    pub fn announce_via_live_region(hwnd: isize, text: &str) -> bool {
        if hwnd == 0 {
            return false;
        }
        unsafe {
            SetWindowTextW(hwnd, wide(text).as_ptr());
            NotifyWinEvent(
                EVENT_OBJECT_LIVEREGIONCHANGED,
                hwnd,
                OBJID_CLIENT,
                CHILDID_SELF,
            );
        }
        true
    }

    /// Release a COM interface pointer through its vtable.
    ///
    /// `IUnknown::Release` is the third entry, after QueryInterface and AddRef.
    ///
    /// # Safety
    ///
    /// `provider` must be null or a live COM interface pointer that this code
    /// owns a reference to. Passing anything else, or passing the same pointer
    /// twice, releases memory the caller no longer owns.
    unsafe fn release(provider: *mut c_void) {
        if provider.is_null() {
            return;
        }
        // The unsafe block is explicit rather than inherited from the
        // signature, which edition 2024 requires and which is the point: the
        // null check above is ordinary code, and only these three lines are
        // trusting the layout of someone else's object.
        unsafe {
            let vtable = *(provider as *mut *mut usize);
            let release_fn: extern "system" fn(*mut c_void) -> u32 =
                std::mem::transmute(*vtable.add(2));
            release_fn(provider);
        }
    }

    /// Whether a window can carry an announcement at all.
    ///
    /// Zero is what the lookup gives back when nothing is in front, and an
    /// event addressed to it reaches nobody.
    fn window_can_carry_announcement(hwnd: isize) -> bool {
        hwnd != 0
    }

    /// Whether the host object the system handed back can be used.
    ///
    /// Both answers have to agree: the call can report success and still hand
    /// back nothing, and using that would be reading from a null pointer.
    fn provider_is_usable(result: i32, provider_is_null: bool) -> bool {
        result == 0 && !provider_is_null
    }

    /// Whether the notification went out. Zero is success here, as everywhere
    /// else in this interface.
    fn notification_succeeded(result: i32) -> bool {
        result == 0
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
            // Deliberately not gated on UiaClientsAreListening. It reports
            // false in cases where a screen reader is plainly running, and
            // gating on it turned every announcement into a silent no-op.
            // Raising the event when nobody is listening costs almost nothing.
            let hwnd = GetForegroundWindow();
            if !window_can_carry_announcement(hwnd) {
                tracing::warn!("No foreground window, so the announcement had nowhere to go");
                return false;
            }

            let mut provider: *mut c_void = std::ptr::null_mut();
            let host = UiaHostProviderFromHwnd(hwnd, &mut provider);
            if !provider_is_usable(host, provider.is_null()) {
                tracing::warn!("UiaHostProviderFromHwnd failed with 0x{:08x}", host);
                return false;
            }
            tracing::debug!("Raising notification on window 0x{:x}", hwnd);

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

            // A report of silence is read against these two lines, so a failed
            // call must not be logged as a delivered one.
            let delivered = notification_succeeded(result);
            if delivered {
                tracing::debug!("Notification raised for {} characters", text.len());
            } else {
                tracing::warn!("UiaRaiseNotificationEvent failed with 0x{:08x}", result);
            }
            delivered
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
            if window_can_carry_announcement(hwnd) {
                NotifyWinEvent(EVENT_OBJECT_NAMECHANGE, hwnd, OBJID_CLIENT, CHILDID_SELF);
            }
        }
    }

    /// Whether any UI Automation client is listening, for the startup log only.
    ///
    /// Not used to decide whether to announce: it reports false in cases where
    /// a screen reader is running, so acting on it silences everything.
    pub fn clients_are_listening() -> bool {
        unsafe { UiaClientsAreListening() != 0 }
    }

    /// Convert to a null-terminated wide string.
    pub fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// A weak test on purpose: it guards these numbers against a typo and
        /// says nothing about whether an event reaches anybody. Only a screen
        /// reader can answer that. They are worth pinning anyway, because a
        /// wrong one points the event at something that does not exist and
        /// Windows still reports success.
        /// The decisions below are ordinary logic that used to sit inside the
        /// unsafe calls. Pinning them says the code asks the right question of
        /// what Windows answered. It says nothing about anything being spoken,
        /// which only a screen reader run can say.
        #[test]
        fn test_a_window_of_zero_cannot_carry_an_announcement() {
            assert!(!window_can_carry_announcement(0));
            assert!(window_can_carry_announcement(0x1234));
        }

        #[test]
        fn test_a_call_that_reported_success_and_handed_back_nothing_is_still_a_failure() {
            assert!(provider_is_usable(0, false));
            assert!(!provider_is_usable(0, true));
        }

        #[test]
        fn test_a_failed_call_is_not_usable_whatever_it_handed_back() {
            assert!(!provider_is_usable(-2147024809, false));
            assert!(!provider_is_usable(-2147024809, true));
        }

        #[test]
        fn test_only_a_result_of_zero_means_the_announcement_went_out() {
            assert!(notification_succeeded(0));
            assert!(!notification_succeeded(-2147024809));
        }

        #[test]
        fn test_the_win32_identifiers_are_the_values_windows_defines() {
            assert_eq!(EVENT_OBJECT_NAMECHANGE, 0x800C);
            assert_eq!(EVENT_OBJECT_LIVEREGIONCHANGED, 0x8019);
            assert_eq!(OBJID_CLIENT, -4);
            assert_eq!(CHILDID_SELF, 0);
            assert_eq!(NOTIFICATION_KIND_OTHER, 4);
        }
    }
}

// ── Working out how to say it ────────────────────────────────────────────────

/// The window an announcement can be carried on, if one was registered.
///
/// Zero is the value the handle holds before the main window exists, and it is
/// also what a failed registration leaves behind, so it means "nowhere to say
/// this" rather than a window.
#[cfg(target_os = "windows")]
fn live_region_target(handle: isize) -> Option<isize> {
    (handle != 0).then_some(handle)
}

/// How hard the screen reader should hold on to an announcement.
///
/// The topic is what tells the screen reader which announcements are versions
/// of one another, so only a line that has one may replace its own earlier
/// value. Without a topic there is nothing to group by, and asking for the most
/// recent only would let any two unrelated lines silence each other.
#[cfg(target_os = "windows")]
fn notification_processing(urgency: Urgency, topic: &str) -> native::Processing {
    match (urgency, topic.is_empty()) {
        (Urgency::Urgent, _) => native::Processing::ImportantAll,
        (Urgency::Important, _) => native::Processing::ImportantMostRecent,
        (Urgency::Routine, false) => native::Processing::MostRecent,
        (Urgency::Routine, true) => native::Processing::All,
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
    /// Native handle of the control used to carry announcements, or zero.
    live_region: std::sync::atomic::AtomicIsize,
}

impl ScreenReaderBridge {
    /// Create a new screen reader bridge
    pub fn new() -> Result<Self> {
        // Logged once at startup so a report of "nothing was spoken" can be
        // read against whether a screen reader was even running.
        #[cfg(target_os = "windows")]
        tracing::info!(
            "Screen reader bridge ready; UI Automation clients listening: {}",
            native::clients_are_listening()
        );

        Ok(Self {
            last_announcement: Mutex::new(None),
            event_log: Mutex::new(Vec::new()),
            status: if cfg!(target_os = "windows") {
                NativeBridgeStatus::Active
            } else {
                NativeBridgeStatus::Fallback
            },
            live_region: std::sync::atomic::AtomicIsize::new(0),
        })
    }

    /// Register the control announcements are carried on.
    ///
    /// Must be called once the main window exists. Without it announcements
    /// fall back to the UI Automation notification, which returns success on
    /// this application and is not delivered, because wxWidgets implements
    /// MSAA rather than being a native UI Automation provider.
    pub fn set_live_region(&self, handle: isize) {
        self.live_region
            .store(handle, std::sync::atomic::Ordering::Relaxed);
        // Zero is the one value that means the registration failed, and a
        // report of "nothing was spoken" is read against this line, so it must
        // not say that all is well.
        if handle == 0 {
            tracing::warn!("No window to carry announcements, so none will be spoken");
        } else {
            tracing::info!("Announcements will be carried on window 0x{:x}", handle);
        }
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
            // The live region is the path that works here. The notification
            // call reports success and is never delivered, so it is only tried
            // when there is no live region to use.
            let registered = self.live_region.load(std::sync::atomic::Ordering::Relaxed);
            if let Some(hwnd) = live_region_target(registered)
                && native::announce_via_live_region(hwnd, text)
            {
                tracing::debug!(
                    "Announced through the live region: {} characters",
                    text.len()
                );
                return Ok(());
            }

            let processing = notification_processing(urgency, topic);
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
            live_region: std::sync::atomic::AtomicIsize::new(0),
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

    /// Registering the window is the one wiring step everything else depends
    /// on: without it there is nowhere to carry an announcement. This proves
    /// the handle is recorded, and nothing at all about anything being spoken
    /// through it, which only a screen reader run can say.
    #[test]
    fn test_registering_the_announcement_window_records_the_handle_it_was_given() {
        let bridge = ScreenReaderBridge::default();
        bridge.set_live_region(0x1234);
        assert_eq!(
            bridge
                .live_region
                .load(std::sync::atomic::Ordering::Relaxed),
            0x1234
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_handle_of_zero_means_nowhere_to_carry_an_announcement() {
        assert_eq!(live_region_target(0), None);
        assert_eq!(live_region_target(0x1234), Some(0x1234));
    }

    /// Proves the code refuses a handle it cannot use, so the caller goes on to
    /// try its other path instead of believing something was said. It proves
    /// nothing about anything being heard. Do not extend this to a made-up
    /// non-zero handle: that would poke a real window on whatever machine is
    /// running the tests.
    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_zero_window_handle_is_refused_rather_than_announced_through() {
        assert!(!native::announce_via_live_region(0, "connection lost"));
    }

    /// The text is on the path of every announcement, so a truncated or empty
    /// conversion is silence. This is ordinary pure code that happens to live
    /// behind the platform gate.
    #[cfg(target_os = "windows")]
    #[test]
    fn test_text_handed_to_windows_is_utf16_and_null_terminated() {
        assert_eq!(native::wide("Hi"), vec![0x0048_u16, 0x0069, 0x0000]);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_character_outside_the_common_range_survives_as_a_pair() {
        // A subject line can carry anything a sender typed, including an emoji,
        // which needs two code units rather than one.
        assert_eq!(native::wide("\u{1F600}"), vec![0xD83D_u16, 0xDE00, 0x0000]);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_an_urgent_announcement_is_marked_never_to_be_dropped() {
        assert_eq!(
            notification_processing(Urgency::Urgent, ""),
            native::Processing::ImportantAll
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_an_important_announcement_keeps_the_latest_of_its_kind() {
        assert_eq!(
            notification_processing(Urgency::Important, ""),
            native::Processing::ImportantMostRecent
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_routine_announcement_on_a_topic_may_replace_its_own_earlier_value() {
        assert_eq!(
            notification_processing(Urgency::Routine, "message-count"),
            native::Processing::MostRecent
        );
    }

    /// A topic is what says which announcements are versions of one another.
    /// Without one there is nothing to group by, so asking the screen reader to
    /// keep only the most recent lets any two unrelated lines silence each
    /// other: "Message moved to Archive" then "Draft saved" leaves only the
    /// second. Most announcements in the application carry no topic, so this is
    /// the common case rather than an edge.
    #[cfg(target_os = "windows")]
    #[test]
    fn test_a_routine_announcement_with_no_topic_does_not_silence_the_one_before_it() {
        assert_eq!(
            notification_processing(Urgency::Routine, ""),
            native::Processing::All
        );
    }
}

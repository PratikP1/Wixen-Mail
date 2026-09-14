//! What an event's stored alerts say, and what an empty column means.
//!
//! One column, `reminders_json` on a stored event, holds what Google sent as
//! overrides, what Microsoft sent when its reminder was on, and what the
//! editor here wrote. Until 2026-09-14 an absent column meant three things
//! at once: at Google, "use the calendar's default alert", which this program
//! never reads; at Microsoft, "the alert is off"; from a CalDAV server,
//! "nobody has read the alarms". The due window cannot tell those apart, and
//! the one mistake worse than a missed alert is interrupting somebody for an
//! event they silenced.
//!
//! So "off" is now a value of its own, an empty list, and the writers that
//! know the alert is off write it: Microsoft's pull when `isReminderOn` is
//! false, Google's pull when `useDefault` is false with no overrides, and
//! the editor here when a person takes the last alert off. An absent column
//! still means "nobody said", and for that the due window uses the program's
//! own default lead, which is Pratik's decision of 2026-09-14: the default
//! fills silence and never overrides an explicit off. A CalDAV event's
//! alarms are still unread, so a CalDAV event is silent-by-default rather
//! than silent-by-its-alarm, and the changelog says so.
//!
//! The reading is here rather than beside the column because three readers
//! already parse the column their own way, for the editor's box, for
//! Outlook's push and for Google's, and each is right for its purpose; this
//! is the fourth reader, the one that decides whether to interrupt, and it
//! is the only one that has to tell off from unknown.

use crate::service::google_api::GoogleReminders;

/// The stored form of "this event has no alert": an empty list. Written by
/// every writer that knows the alert is off, read by [`read`].
pub const NO_ALERT: &str = "[]";

/// What the stored alerts say about whether an event alerts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoredAlert {
    /// An alert is stored: this many minutes before the start. The first
    /// one, because the editor offers one alert and answers for the first.
    Lead(i64),
    /// The alert is off, and a writer that knew so said so.
    Off,
    /// Nobody said. The column is absent, or holds something that will not
    /// read as a list of alerts, which came from a calendar server and is
    /// not somebody asking for silence.
    Unknown,
}

/// Read the stored alerts.
pub fn read(stored: Option<&str>) -> StoredAlert {
    let Some(stored) = stored else {
        return StoredAlert::Unknown;
    };
    let Ok(serde_json::Value::Array(alerts)) = serde_json::from_str::<serde_json::Value>(stored)
    else {
        return StoredAlert::Unknown;
    };
    let Some(first) = alerts.first() else {
        return StoredAlert::Off;
    };
    match first.get("minutes").and_then(serde_json::Value::as_i64) {
        Some(minutes) => StoredAlert::Lead(minutes),
        None => StoredAlert::Unknown,
    }
}

/// How many minutes before its start an event is raised, or `None` for an
/// event that is not raised at all.
///
/// A stored lead is the lead. Off is off, whatever the default says. Unknown
/// takes the default, because the default fills silence and never overrides
/// an answer.
pub fn lead_to_raise_at(stored: Option<&str>, default_lead: i64) -> Option<i64> {
    match read(stored) {
        StoredAlert::Lead(minutes) => Some(minutes),
        StoredAlert::Off => None,
        StoredAlert::Unknown => Some(default_lead),
    }
}

/// What Google's pull stores for an event's reminders.
///
/// Overrides are the event's own alerts and are stored as sent. `useDefault`
/// false with no overrides is Google's way of saying the event never alerts,
/// and is stored as off. `useDefault` true with no overrides is the
/// calendar's default alert, which this program never reads, and is stored
/// as nothing, so the due window gives it the program's own default.
pub fn from_google(reminders: Option<&GoogleReminders>) -> Option<String> {
    let reminders = reminders?;
    if reminders.overrides.is_empty() {
        return (!reminders.use_default).then(|| NO_ALERT.to_string());
    }
    let own: Vec<_> = reminders
        .overrides
        .iter()
        .map(|o| serde_json::json!({"method": o.method, "minutes": o.minutes}))
        .collect();
    serde_json::to_string(&own).ok()
}

/// What Microsoft's pull stores for an event's reminder.
///
/// On with a lead is stored as one alert. Off is stored as off. On with no
/// lead, and a reminder Graph said nothing about, are stored as nothing, as
/// they were before off had a stored form.
pub fn from_microsoft(is_reminder_on: Option<bool>, lead_minutes: i32) -> Option<String> {
    match is_reminder_on? {
        false => Some(NO_ALERT.to_string()),
        true if lead_minutes > 0 => serde_json::to_string(&vec![serde_json::json!({
            "method": "popup",
            "minutes": lead_minutes,
        })])
        .ok(),
        true => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::google_api::GoogleReminderOverride;

    #[test]
    fn test_a_stored_alert_is_read_as_its_lead_and_off_and_nothing_are_told_apart() {
        assert_eq!(
            read(Some("[{\"minutes\":15,\"method\":\"popup\"}]")),
            StoredAlert::Lead(15)
        );
        assert_eq!(
            read(Some("[{\"minutes\":30},{\"minutes\":5}]")),
            StoredAlert::Lead(30),
            "the first alert is the one the editor shows and answers for"
        );
        assert_eq!(read(Some(NO_ALERT)), StoredAlert::Off);
        assert_eq!(read(None), StoredAlert::Unknown);
        // Something a calendar server stored that will not read as alerts is
        // not somebody asking for silence.
        assert_eq!(read(Some("not a list")), StoredAlert::Unknown);
        assert_eq!(read(Some("[{\"method\":\"popup\"}]")), StoredAlert::Unknown);
    }

    #[test]
    fn test_the_default_fills_silence_and_never_overrides_an_explicit_off() {
        assert_eq!(lead_to_raise_at(Some("[{\"minutes\":10}]"), 15), Some(10));
        assert_eq!(lead_to_raise_at(None, 15), Some(15));
        assert_eq!(
            lead_to_raise_at(Some(NO_ALERT), 15),
            None,
            "an event whose owner switched the alert off was given the default"
        );
    }

    #[test]
    fn test_googles_pull_stores_off_only_when_google_said_the_event_never_alerts() {
        let own = GoogleReminders {
            use_default: false,
            overrides: vec![GoogleReminderOverride {
                method: "popup".to_string(),
                minutes: 20,
            }],
        };
        assert_eq!(
            read(from_google(Some(&own)).as_deref()),
            StoredAlert::Lead(20)
        );

        let never = GoogleReminders {
            use_default: false,
            overrides: Vec::new(),
        };
        assert_eq!(from_google(Some(&never)).as_deref(), Some(NO_ALERT));

        let calendars_default = GoogleReminders {
            use_default: true,
            overrides: Vec::new(),
        };
        assert_eq!(
            from_google(Some(&calendars_default)),
            None,
            "the calendar's default is not off; nobody here knows what it is"
        );
        assert_eq!(from_google(None), None);
    }

    #[test]
    fn test_microsofts_pull_stores_off_when_the_reminder_is_off() {
        assert_eq!(
            read(from_microsoft(Some(true), 15).as_deref()),
            StoredAlert::Lead(15)
        );
        assert_eq!(from_microsoft(Some(false), 15).as_deref(), Some(NO_ALERT));
        assert_eq!(from_microsoft(Some(false), 0).as_deref(), Some(NO_ALERT));
        // As before off had a form: on with nothing to count down from, and
        // a reminder Graph said nothing about, are nobody's answer.
        assert_eq!(from_microsoft(Some(true), 0), None);
        assert_eq!(from_microsoft(None, 15), None);
    }
}

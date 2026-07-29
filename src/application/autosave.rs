//! How often a draft is saved while somebody is writing.
//!
//! Set in minutes, from nought to ten, where nought means never. Minutes
//! rather than a free number of seconds because the choice is coarse by
//! nature: the difference between saving every ninety seconds and every two
//! minutes is not a decision anybody can make usefully, and a spin box that
//! steps a minute at a time is two key presses to change rather than a text
//! field to type into and get wrong.

use std::time::Duration;

/// How often a draft is saved, in minutes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutosaveInterval {
    minutes: u32,
}

impl AutosaveInterval {
    /// The most anybody can ask for.
    ///
    /// Past ten minutes the feature stops being a safety net: an hour of
    /// writing lost is the same disaster whether the last save was eleven
    /// minutes ago or fifty. Somebody who wants it off has nought.
    pub const MAX_MINUTES: u32 = 10;

    /// The setting as stored, brought into range.
    ///
    /// Clamped rather than rejected. A settings file edited by hand, or
    /// written by a future version with a wider range, should not stop the
    /// application from starting.
    pub fn from_setting(minutes: u32) -> Self {
        Self {
            minutes: minutes.min(Self::MAX_MINUTES),
        }
    }

    /// Never save automatically.
    pub fn off() -> Self {
        Self { minutes: 0 }
    }

    /// Whether automatic saving is switched off.
    fn is_off(self) -> bool {
        self.minutes == 0
    }

    /// The setting, for storing and for a spin box.
    pub fn minutes(self) -> u32 {
        self.minutes
    }

    /// How long to wait between saves, or `None` when it is off.
    pub fn interval(self) -> Option<Duration> {
        (!self.is_off()).then(|| Duration::from_secs(u64::from(self.minutes) * 60))
    }

    /// What the setting says when it is read out.
    pub fn spoken(self) -> String {
        match self.minutes {
            0 => "Drafts are not saved automatically".to_string(),
            1 => "Drafts are saved automatically every minute".to_string(),
            n => format!("Drafts are saved automatically every {n} minutes"),
        }
    }
}

impl Default for AutosaveInterval {
    /// Every two minutes.
    ///
    /// On by default, because the people most helped by it are the ones who
    /// would never find the setting. Two rather than one: a save is a write to
    /// the database and a status line announcement, and once a minute is often
    /// enough to be heard as interruption while writing.
    fn default() -> Self {
        Self { minutes: 2 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nought_means_never() {
        let interval = AutosaveInterval::from_setting(0);

        assert!(interval.is_off());
        assert_eq!(interval.interval(), None);
    }

    #[test]
    fn test_a_setting_beyond_the_range_is_brought_back_rather_than_refused() {
        // A settings file edited by hand should not stop the application
        // starting, and neither should one written by a later version.
        assert_eq!(
            AutosaveInterval::from_setting(999).minutes(),
            AutosaveInterval::MAX_MINUTES
        );
    }

    #[test]
    fn test_the_interval_is_the_number_of_minutes_asked_for() {
        let interval = AutosaveInterval::from_setting(3);

        assert_eq!(interval.interval(), Some(Duration::from_secs(180)));
    }

    #[test]
    fn test_every_step_in_the_range_is_usable() {
        // The spin box steps one at a time from nought to ten, so every one of
        // those eleven values has to mean something.
        for minutes in 0..=AutosaveInterval::MAX_MINUTES {
            let interval = AutosaveInterval::from_setting(minutes);

            assert_eq!(interval.minutes(), minutes);
            assert_eq!(interval.is_off(), minutes == 0);
            assert!(!interval.spoken().is_empty());
        }
    }

    #[test]
    fn test_one_minute_is_not_read_as_one_minutes() {
        assert!(
            AutosaveInterval::from_setting(1)
                .spoken()
                .ends_with("minute")
        );
        assert!(
            AutosaveInterval::from_setting(2)
                .spoken()
                .ends_with("minutes")
        );
    }

    #[test]
    fn test_off_says_so_rather_than_saying_nothing() {
        // This is read out when the setting is reached, and silence would be
        // indistinguishable from a control that failed to announce.
        let spoken = AutosaveInterval::off().spoken();

        assert!(spoken.contains("not saved automatically"), "got {spoken}");
    }

    #[test]
    fn test_it_is_on_unless_somebody_turns_it_off() {
        // The people this protects most are the ones who would never find the
        // setting to switch it on.
        assert!(!AutosaveInterval::default().is_off());
    }
}

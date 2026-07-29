//! What Wixen Mail may change at somebody's provider.
//!
//! Reading mail into a local cache cannot hurt anybody. Sending a message,
//! removing one from a server, or deleting a task at a provider can, and none
//! of those paths has run for real yet. So they are switched off by default
//! and turned on deliberately.
//!
//! # Three places can say no, and any one of them is enough
//!
//! The command line, the application's own setting, and the account. A change
//! goes out only when all three allow it. That is the whole design: a safety
//! catch that something else can quietly arm is not a safety catch, and
//! somebody who marked one account read-only should not have that undone by a
//! setting they changed for a different reason.
//!
//! It follows that the command line can only ever restrict. `--read-only`
//! stops everything for that run whatever is stored; there is deliberately no
//! flag that forces writes on, because the flag that turns off a safety catch
//! is the flag somebody leaves in a shortcut and forgets.
//!
//! # Split by what it costs to get wrong
//!
//! Mail is separate from everything else. Losing a task is annoying and can be
//! typed again; a message deleted from a server, or sent to the wrong people,
//! is gone. So they are two answers rather than one.

use serde::{Deserialize, Serialize};

/// What may be changed at a provider.
///
/// Two answers rather than one boolean, because the two cost different amounts
/// to get wrong. `Default` is the safe end of both, so anything constructed
/// without a decision changes nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Allowed {
    /// Sending a message, and changing or deleting one on the server.
    ///
    /// The one that cannot be undone. A message that has gone has gone, and a
    /// message deleted from a server may be the only copy.
    pub mail: bool,
    /// Tasks, contacts and calendar events at a provider.
    ///
    /// Recoverable by hand, mostly, and this is the least proven code in the
    /// application: none of the three sync paths has met a live account.
    pub personal_information: bool,
}

impl Allowed {
    /// Nothing may be changed.
    pub const NOTHING: Self = Self {
        mail: false,
        personal_information: false,
    };

    /// Everything may be changed, which is what a finished mail client does.
    pub const EVERYTHING: Self = Self {
        mail: true,
        personal_information: true,
    };

    /// What an alpha tester starts with.
    ///
    /// Tasks, contacts and the calendar can be changed; mail cannot. Somebody
    /// can point this at their real account, use it all day, and the worst
    /// that happens is a task in the wrong place. Sending is the deliberate
    /// step afterwards.
    pub const FOR_TESTING: Self = Self {
        mail: false,
        personal_information: true,
    };

    /// What both of these allow, which is what actually happens.
    ///
    /// The safest answer wins on each half independently, so a person who
    /// allows everything globally and marks one account read-only gets what
    /// they asked for on both.
    pub const fn and(self, other: Self) -> Self {
        Self {
            mail: self.mail && other.mail,
            personal_information: self.personal_information && other.personal_information,
        }
    }

    /// Whether anything at all may be changed.
    pub const fn anything(self) -> bool {
        self.mail || self.personal_information
    }

    /// How this reads in a settings screen or a status line.
    pub fn spoken(self) -> &'static str {
        match (self.mail, self.personal_information) {
            (true, true) => "Everything, including sending mail",
            (false, true) => "Tasks, contacts and calendar only. Mail is read only",
            (true, false) => "Mail only",
            (false, false) => "Nothing. This account is read only",
        }
    }
}

/// Everything that has an opinion about what may be changed.
///
/// Kept as one value so the answer is worked out in one place and every
/// transport asks the same question. Threading three booleans through instead
/// is how one of them comes to be forgotten at a call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permission {
    /// What the command line said, which can only ever narrow the others.
    pub command_line: Allowed,
    /// What this installation is set to.
    pub setting: Allowed,
    /// What this account is set to.
    pub account: Allowed,
}

impl Permission {
    /// The answer: what all three allow.
    pub const fn allowed(self) -> Allowed {
        self.command_line.and(self.setting).and(self.account)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nothing_is_allowed_until_something_says_otherwise() {
        // The important default. A value built without anybody deciding should
        // be the one that cannot damage somebody's mail.
        assert_eq!(Allowed::default(), Allowed::NOTHING);
        assert!(!Allowed::default().anything());
    }

    #[test]
    fn test_an_alpha_tester_can_change_their_tasks_and_not_their_mail() {
        // Somebody can point this at their real account, use it all day, and
        // the worst that happens is a task in the wrong place.
        assert_eq!(
            Allowed::FOR_TESTING,
            Allowed {
                mail: false,
                personal_information: true,
            }
        );
    }

    #[test]
    fn test_one_no_is_enough() {
        // The rule the whole thing rests on. Somebody who marked an account
        // read-only must not have that undone by a setting they changed for a
        // different account, and the command line must be able to stop
        // everything without editing any stored setting at all.
        let stopped_at_the_command_line = Permission {
            command_line: Allowed::NOTHING,
            setting: Allowed::EVERYTHING,
            account: Allowed::EVERYTHING,
        };
        let stopped_by_the_setting = Permission {
            command_line: Allowed::EVERYTHING,
            setting: Allowed::NOTHING,
            account: Allowed::EVERYTHING,
        };
        let stopped_by_the_account = Permission {
            command_line: Allowed::EVERYTHING,
            setting: Allowed::EVERYTHING,
            account: Allowed::NOTHING,
        };

        for stopped in [
            stopped_at_the_command_line,
            stopped_by_the_setting,
            stopped_by_the_account,
        ] {
            assert_eq!(stopped.allowed(), Allowed::NOTHING, "{stopped:?}");
        }
    }

    #[test]
    fn test_the_two_halves_are_decided_separately() {
        // Somebody who allows everything but marks one account mail-read-only
        // should still be able to sync that account's tasks. One boolean would
        // have taken both away.
        let permission = Permission {
            command_line: Allowed::EVERYTHING,
            setting: Allowed::EVERYTHING,
            account: Allowed::FOR_TESTING,
        };

        assert_eq!(permission.allowed(), Allowed::FOR_TESTING);
    }

    #[test]
    fn test_everything_allowed_everywhere_allows_everything() {
        let permission = Permission {
            command_line: Allowed::EVERYTHING,
            setting: Allowed::EVERYTHING,
            account: Allowed::EVERYTHING,
        };

        assert_eq!(permission.allowed(), Allowed::EVERYTHING);
        assert!(permission.allowed().mail);
    }

    #[test]
    fn test_what_is_allowed_can_be_said_out_loud() {
        // It goes in a settings screen and in the refusal somebody hears, so
        // each of the four has to be a sentence rather than two booleans read
        // back at them.
        assert!(Allowed::NOTHING.spoken().contains("read only"));
        assert!(Allowed::FOR_TESTING.spoken().contains("Mail is read only"));
        assert!(Allowed::EVERYTHING.spoken().contains("sending mail"));
        assert!(Allowed::EVERYTHING.spoken() != Allowed::FOR_TESTING.spoken());
    }

    #[test]
    fn test_combining_is_order_independent() {
        // It reads as though order might matter, and it must not: the three
        // are asked in whatever order a call site happens to write them.
        let a = Allowed::FOR_TESTING;
        let b = Allowed {
            mail: true,
            personal_information: false,
        };

        assert_eq!(a.and(b), b.and(a));
        assert_eq!(a.and(b), Allowed::NOTHING);
    }
}

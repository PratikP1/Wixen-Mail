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
}

/// The warning shown beside the two Allowed Changes boxes.
///
/// Kept here rather than typed into the settings screen, because it was typed
/// there and the copy drifted: the visible label and the accessible name were
/// two hand-written strings that differed from each other, and both had lost
/// their line continuations, so each carried a run of stray spaces in the
/// middle of a sentence somebody hears.
///
/// One sentence per idea, and the irreversible parts last, because that is the
/// part somebody has to still be listening for.
pub const EXPERIMENTAL_WARNING: &str = "Both are experimental: none of this has been run against a real account yet, \
     so expect bugs. Reading your mail is the part that has been used. A message \
     that has been sent cannot be recalled, and a message deleted from a server \
     may have been the only copy.";

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
    fn test_anything_that_writes_says_it_is_experimental() {
        // The warning has to reach the person using it, not sit in a note.
        // None of these paths has run against a real account, and this is the
        // sentence beside the two boxes that turn them on.
        assert!(EXPERIMENTAL_WARNING.contains("experimental"));
        assert!(EXPERIMENTAL_WARNING.contains("real account"));
    }

    #[test]
    fn test_the_experimental_warning_reads_as_sentences_rather_than_a_wrapped_literal() {
        // A wrapped string literal that loses its continuations keeps every
        // space of the indenting, and this one is read aloud. Runs of stray
        // spaces are silence in the middle of a sentence somebody is relying
        // on to tell them what cannot be undone.
        assert!(
            !EXPERIMENTAL_WARNING.contains("  "),
            "{EXPERIMENTAL_WARNING}"
        );
        assert!(EXPERIMENTAL_WARNING.contains("cannot be recalled"));
        assert!(EXPERIMENTAL_WARNING.contains("only copy"));
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

/// What the command line narrowed this run to.
///
/// Written once, before anything opens, and read from wherever a client is
/// built. A global is worth justifying: this one is set exactly once at
/// startup, is never changed afterwards, and can only ever take permissions
/// away. Threading it instead would mean carrying it through seven
/// constructions in the window layer, several inside spawned tasks, which is
/// seven chances to drop it, and dropping it fails towards writing.
static FROM_COMMAND_LINE: std::sync::OnceLock<Allowed> = std::sync::OnceLock::new();

/// Record what the command line allowed. Call once, before anything opens.
///
/// A second call is ignored rather than being an error: the first answer is
/// the one from the arguments, and nothing later should be able to widen it.
pub fn narrow_this_run_to(allowed: Allowed) {
    let _ = FROM_COMMAND_LINE.set(allowed);
}

/// What this account may actually change, with all three asked.
///
/// The one function every client should call before it is built. Returns the
/// narrowest of the command line, the application-wide setting and the
/// account's own, so no caller has to remember there are three.
///
/// A settings file that cannot be read counts as allowing nothing. That is the
/// safe direction: somebody whose config is corrupt should find that nothing
/// syncs, not that everything is permitted.
pub fn allowed_for(account_id: &str) -> Allowed {
    let stored = crate::data::config::ConfigManager::load_stored()
        .map(|config| config.app_config().allowed_for(account_id))
        .unwrap_or(Allowed::NOTHING);

    narrowed_by(FROM_COMMAND_LINE.get().copied(), stored)
}

/// What the command line leaves of a stored answer.
///
/// Split out so it can be tested without touching the process-wide value.
/// That value is set once, so a test for the unset case and a test that sets
/// it cannot both live in the same process: Rust runs them in parallel and
/// whichever went first decided the answer for the other. Not hypothetical,
/// it is what happened, and it failed about one run in three.
///
/// `None` means the command line had no opinion, which is different from it
/// having said "nothing": a window opened by a test or a tool that never
/// parsed arguments is governed by the settings alone.
const fn narrowed_by(command_line: Option<Allowed>, stored: Allowed) -> Allowed {
    match command_line {
        Some(narrowing) => narrowing.and(stored),
        None => stored,
    }
}

#[cfg(test)]
mod resolving {
    use super::*;

    #[test]
    fn test_no_opinion_from_the_command_line_leaves_the_settings_alone() {
        // A window can be opened by a test or a tool that never parsed
        // arguments. That must not silently mean "allow everything" nor
        // "allow nothing": it means the command line said nothing, and the
        // settings decide on their own.
        assert_eq!(narrowed_by(None, Allowed::EVERYTHING), Allowed::EVERYTHING);
        assert_eq!(
            narrowed_by(None, Allowed::FOR_TESTING),
            Allowed::FOR_TESTING
        );
        assert_eq!(narrowed_by(None, Allowed::NOTHING), Allowed::NOTHING);
    }

    #[test]
    fn test_the_command_line_can_only_take_permissions_away() {
        assert_eq!(
            narrowed_by(Some(Allowed::NOTHING), Allowed::EVERYTHING),
            Allowed::NOTHING
        );
        assert_eq!(
            narrowed_by(Some(Allowed::EVERYTHING), Allowed::FOR_TESTING),
            Allowed::FOR_TESTING,
            "saying everything on the command line must not widen a setting"
        );
    }

    #[test]
    fn test_recording_it_twice_keeps_the_first_answer() {
        // Set once at startup. Anything later trying to widen it is ignored,
        // which is the property that makes a global defensible here.
        narrow_this_run_to(Allowed::NOTHING);
        narrow_this_run_to(Allowed::EVERYTHING);

        assert_eq!(FROM_COMMAND_LINE.get().copied(), Some(Allowed::NOTHING));
    }
}

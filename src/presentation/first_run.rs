//! The first thing an alpha tester sees.
//!
//! Two things, in one place, before any mail is touched: what is worth testing
//! and what is known to be broken, and a choice about what Wixen Mail may
//! change at their provider.
//!
//! It exists because most of what writes has never run against a real account.
//! Somebody deciding whether to point this at their own mail needs to know
//! that before they do it, not afterwards, and a warning that lives only in a
//! release note is a warning most people never read.
//!
//! # Why it is a choice rather than a notice
//!
//! A notice is dismissed. A question has to be answered, and the answer is
//! stored, so the decision was actually made by the person it affects. The
//! default is the safe one, so somebody who presses Enter without reading gets
//! the cautious answer rather than the permissive one.
//!
//! The wording lives here, apart from the window, so it can be read in a test.
//! What it says is the part that matters: the window is a list of radio
//! buttons.

use crate::application::allowed::Allowed;

/// What the first-run screen offers.
///
/// Three, not four. "Mail only" is a real combination and not one anybody
/// wants on purpose, so offering it would be a fourth thing to read past.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    /// Read everything, change nothing.
    ReadOnly,
    /// Tasks, contacts and calendar, but not mail.
    TasksAndContacts,
    /// Everything, including sending.
    Everything,
}

impl Choice {
    /// The three, in the order they are offered.
    ///
    /// Safest first, so the default is the first thing focus lands on and
    /// somebody arrowing down is widening rather than narrowing.
    pub const ALL: [Choice; 3] = [
        Choice::ReadOnly,
        Choice::TasksAndContacts,
        Choice::Everything,
    ];

    /// What the screen starts on.
    pub const DEFAULT: Choice = Choice::TasksAndContacts;

    /// What this choice permits.
    pub const fn allows(self) -> Allowed {
        match self {
            Choice::ReadOnly => Allowed::NOTHING,
            Choice::TasksAndContacts => Allowed::FOR_TESTING,
            Choice::Everything => Allowed::EVERYTHING,
        }
    }

    /// The label on the button.
    pub const fn label(self) -> &'static str {
        match self {
            Choice::ReadOnly => "Read my mail, change nothing",
            Choice::TasksAndContacts => "Also let it change my tasks, contacts and calendar",
            Choice::Everything => "Let it do everything, including sending mail",
        }
    }

    /// What that choice means, said plainly.
    ///
    /// Each says what it costs as well as what it does, because the point of
    /// asking is that the person can weigh it. "Recommended" on its own is
    /// something to click past.
    pub const fn explanation(self) -> &'static str {
        match self {
            Choice::ReadOnly => {
                "Nothing you do here reaches your provider. Safe to point at \
                 your real mail. You will not be able to send, move, delete, or \
                 sync changes to your tasks."
            }
            Choice::TasksAndContacts => {
                "Sending, moving and deleting mail stay off. Changes to tasks, \
                 contacts and the calendar go up to your provider. This has \
                 never been run against a real account, so a task may end up in \
                 the wrong place, but nothing here can lose an email."
            }
            Choice::Everything => {
                "Everything works, including sending. None of it has been run \
                 against a real account. A message that goes out cannot be \
                 recalled, and a message deleted from a server may have been \
                 the only copy. Worth choosing on an account you do not mind \
                 breaking, rather than on the one you rely on."
            }
        }
    }
}

/// The heading at the top of the screen.
pub const TITLE: &str = "Before you start";

/// What the screen says before the choice.
///
/// Short on purpose. It is read out in full by a screen reader before the
/// person reaches the buttons, so anything not worth hearing every time does
/// not belong here; the longer version is the testing page.
pub const INTRODUCTION: &str = "\
Wixen Mail is an alpha. Reading your mail is the part that has been used.

Everything that writes is experimental: sending, moving, deleting, filing a \
copy in Sent, and sending your changes to tasks, contacts and the calendar \
back to your provider. None of that has been run against a real account yet, \
so expect it to have bugs.

Choose what Wixen Mail may change. You can change this later in Settings, and \
you can set it per account.";

/// The button that opens the fuller page.
pub const READ_MORE: &str = "What to test, and what is known to be broken";

/// Which of the shipped documents that button opens.
///
/// Finding it and turning it into something readable is
/// [`crate::presentation::help_page`]'s job.
pub const TESTING_PAGE: &str = "ALPHA_TESTING.md";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_safe_choice_is_the_one_it_starts_on() {
        // Somebody who presses Enter without reading gets the cautious
        // answer. That is the whole reason the default is not "everything".
        assert_eq!(Choice::DEFAULT, Choice::TasksAndContacts);
        assert!(!Choice::DEFAULT.allows().mail);
    }

    #[test]
    fn test_the_choices_go_from_safest_to_riskiest() {
        // Focus lands on the first, so arrowing down widens. The other order
        // would mean arrowing down to become safer, having started on the
        // riskiest thing.
        let mail: Vec<bool> = Choice::ALL.iter().map(|c| c.allows().mail).collect();
        let pim: Vec<bool> = Choice::ALL
            .iter()
            .map(|c| c.allows().personal_information)
            .collect();

        assert_eq!(mail, [false, false, true]);
        assert_eq!(pim, [false, true, true]);
    }

    #[test]
    fn test_each_choice_says_what_it_costs_and_not_only_what_it_does() {
        // The point of asking is that somebody can weigh it, and they cannot
        // weigh "recommended".
        assert!(
            Choice::ReadOnly
                .explanation()
                .contains("will not be able to")
        );
        assert!(
            Choice::TasksAndContacts
                .explanation()
                .contains("never been run against a real account")
        );
        assert!(
            Choice::Everything
                .explanation()
                .contains("cannot be recalled")
        );
    }

    #[test]
    fn test_the_introduction_says_writing_is_experimental() {
        assert!(INTRODUCTION.contains("experimental"));
        assert!(INTRODUCTION.contains("expect it to have bugs"));
    }

    #[test]
    fn test_the_introduction_says_the_choice_can_be_changed_later() {
        // Otherwise it reads as a one-off decision made under pressure at
        // startup, which is when somebody picks whatever gets them past it.
        assert!(INTRODUCTION.contains("change this later in Settings"));
    }

    #[test]
    fn test_every_choice_has_a_label_and_an_explanation() {
        for choice in Choice::ALL {
            assert!(!choice.label().is_empty(), "{choice:?}");
            assert!(
                choice.explanation().len() > 40,
                "{choice:?} is explained too thinly to weigh"
            );
        }
    }

    #[test]
    fn test_the_page_it_points_at_exists_in_the_repository() {
        // A button offering to open a document that is not there is worse
        // than no button. This only proves the file is in the repository; the
        // test below is the one about the installed copy.
        let in_repository = std::path::Path::new("docs").join(TESTING_PAGE);

        assert!(
            in_repository.exists(),
            "{} is missing, so the button opens nothing",
            in_repository.display()
        );
    }

    #[test]
    fn test_the_page_is_looked_for_beside_the_program() {
        // The bug this was written for: a relative path is relative to the
        // working directory, and an installed copy starts from wherever the
        // shortcut points. It has to look beside the executable first.
        //
        // Under `cargo test` nothing sits beside the test binary, so this
        // falls back, and what is checked is that the fallback is the
        // repository path rather than something absolute and wrong.
        let looked_for = crate::presentation::help_page::shipped(TESTING_PAGE);

        assert!(
            looked_for.ends_with("ALPHA_TESTING.md"),
            "{}",
            looked_for.display()
        );
    }

    #[test]
    fn test_the_installer_ships_the_page_beside_the_program() {
        // The other half, and the half a Rust test cannot check by running:
        // the page has to be in the installer or the button opens nothing on
        // every machine except this one. The wildcard is what puts it there.
        let installer = std::fs::read_to_string("installer/Wixen-Mail-Setup.iss")
            .expect("the installer script");

        assert!(
            installer.contains(r"docs\*.md"),
            "the installer no longer ships the docs folder, so the button opens nothing"
        );
    }
}

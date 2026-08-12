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
//! stored, so the decision was actually made by the person it affects.
//! Somebody who presses Enter without reading gets the middle answer, which
//! leaves mail alone and sends changes to tasks, contacts and the calendar up
//! to their provider. Focus lands on that middle answer too, so the answer read
//! out is the answer Continue takes.
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
    /// Safest first, so arrowing down widens what is allowed and arrowing up
    /// narrows it. The one selected is the second, so somebody who presses
    /// Enter without reading allows changes to tasks, contacts and the
    /// calendar. Focus lands on the second as well, so the answer read out is
    /// the answer Continue takes.
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
            // "Allow", the word the settings screen and every sync sentence
            // use for this. Somebody who answers here and later goes looking
            // for the setting is looking for the same word.
            Choice::TasksAndContacts => "Also allow it to change my tasks, contacts and calendar",
            Choice::Everything => "Allow it to do everything, including sending mail",
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
the answer covers every account.";

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
    fn test_the_screen_starts_on_the_second_choice_and_not_the_safest() {
        // Somebody who presses Enter without reading gets the second of the
        // three: mail is left alone, and changes to tasks, contacts and the
        // calendar go up to their provider. That is the whole reason it is not
        // "everything", and it is not the safest answer either.
        assert_eq!(Choice::DEFAULT, Choice::TasksAndContacts);
        assert_eq!(Choice::ALL[1], Choice::DEFAULT);
        assert!(!Choice::DEFAULT.allows().mail);
        assert!(Choice::DEFAULT.allows().personal_information);
    }

    /// The window that draws this screen, read as text.
    ///
    /// The same thing `test_the_installer_ships_the_page_beside_the_program`
    /// does with the installer script, and for the same reason: the fact is
    /// real and it matters, and a test cannot reach it by running because
    /// reaching it needs a window on a screen.
    fn the_window_that_draws_it() -> String {
        std::fs::read_to_string("src/presentation/wx_first_run.rs")
            .expect("the first-run window to be readable")
            .replace("\r\n", "\n")
    }

    #[test]
    fn test_the_screen_puts_focus_on_the_answer_it_ticks() {
        // The rule this screen has to keep: what is read out and what Continue
        // does are one answer. They were two. Focus was put on one answer and
        // the tick on another, so somebody using a screen reader heard one
        // answer, pressed Enter on it, and had writing to their real address
        // book, calendar and tasks switched on without being told.
        //
        // Which answer it starts on is a separate question, and this is not it.
        //
        // What this cannot see: whether a screen reader really reads the
        // ticked answer out when the screen opens. Only a run with one says
        // that. What is pinned here is that the code asks for the right thing,
        // in the right order, in one place.
        let window = the_window_that_draws_it();

        assert!(
            window.contains("button.set_value(*choice == Choice::DEFAULT)"),
            "the window no longer ticks the answer DEFAULT names, so nothing \
             here knows which one it starts on"
        );
        assert!(
            window.contains(
                "if let Some((_, ticked)) = buttons.iter().find(|(_, button)| \
                 button.get_value()) {\n        ticked.set_focus();\n    }"
            ),
            "the window no longer puts focus on the answer it ticks"
        );
        assert_eq!(
            window.matches("set_focus").count(),
            1,
            "something else in the window takes focus as well, so what is heard \
             is no longer the answer that is ticked"
        );
        assert_eq!(
            window.matches("set_value").count(),
            1,
            "the tick is put on in more than one place, so the answer focus \
             found is not the one left ticked"
        );
        assert!(
            window.find("button.set_value(*choice == Choice::DEFAULT)")
                < window.find("ticked.set_focus()"),
            "the tick is put on after focus goes looking for it, so focus finds \
             nothing ticked and lands wherever the window puts it"
        );
    }

    #[test]
    fn test_what_each_answer_costs_rides_on_the_button_and_on_the_screen() {
        // Both, deliberately, because the two reach different people and
        // dropping either takes the sentence away from somebody.
        //
        // The description is what a screen reader working through Microsoft
        // Active Accessibility reads when the button takes focus, which is the
        // moment somebody is deciding. It was missing once: three radio
        // buttons that read correctly and three explanations of what each one
        // costs that nobody ever heard.
        //
        // The words on screen are the only copy a sighted reader gets, and the
        // only copy a reader working through UI Automation gets, because a
        // description set here never reaches that tree at all.
        //
        // Read from the source, the same way the focus rule above is, because
        // reaching the real answer needs a window. Whether a screen reader
        // speaks either of them is a separate question and this is not it.
        //
        // What this cannot see beyond that: whether either sentence is the
        // right one for the answer it sits on. It asks that both calls are
        // written and handed the explanation. A screen that puts every
        // explanation on the same button keeps this green.
        let window = the_window_that_draws_it();

        assert!(
            window.contains(
                "set_accessible_name_and_description(&button, choice.label(), choice.explanation())"
            ),
            "what an answer costs is no longer the button's description, so \
             nobody hears it at the moment they are choosing"
        );
        assert!(
            window.contains(".with_label(choice.explanation())"),
            "what an answer costs is no longer on the screen, so a sighted \
             reader and a reader working through UI Automation have no copy \
             of it at all"
        );
    }

    #[test]
    fn test_the_choices_go_from_safest_to_riskiest() {
        // Safest at the top, so arrowing down widens. The other order would
        // mean arrowing down to become safer, having started on the riskiest
        // thing. Where the screen starts is a separate question, and the test
        // above is the one about that.
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
    fn test_the_introduction_offers_only_the_control_that_exists() {
        // It offered a setting per account. Nothing writes one: the settings
        // screen writes the application-wide answer and every account gets
        // that. This screen is read out in full before somebody reaches the
        // buttons, so a control named here that is not there is a search
        // somebody makes with a screen reader for nothing.
        assert!(INTRODUCTION.contains("the answer covers every account"));
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
    fn test_each_choice_says_in_its_label_what_it_will_and_will_not_change() {
        // The label is the whole of what somebody in a hurry reads before
        // answering, so a wrong one is a person agreeing to something they did
        // not agree to. The test above only asks that a label exists.
        //
        // This pins the words the code produces. Whether they reach the
        // accessibility tree as the radio buttons' names is a screen reader
        // question, and it is not answered here.
        assert_eq!(Choice::ReadOnly.label(), "Read my mail, change nothing");
        assert_eq!(
            Choice::TasksAndContacts.label(),
            "Also allow it to change my tasks, contacts and calendar"
        );
        assert_eq!(
            Choice::Everything.label(),
            "Allow it to do everything, including sending mail"
        );
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

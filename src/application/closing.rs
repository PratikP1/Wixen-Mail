//! What closing the window should actually do.
//!
//! Three things can ask this application to close and they do not all mean the
//! same thing, which is the whole reason this is a decision rather than a line
//! of code in the close handler.
//!
//! # Quit means quit
//!
//! On Windows, File then Quit and the window's close button arrive at the same
//! handler. So a handler that hides the window instead of closing it turns Quit
//! into "hide", and somebody who chose Quit from a menu is left with a running
//! program they were certain they had ended. The menu says what it does, so it
//! has to do it.
//!
//! # Hiding without somewhere to go is a trap
//!
//! Hiding the only window is safe only when there is a tray icon to bring it
//! back. If the icon failed to install, hiding leaves a process with no window,
//! no icon and no way in: it is still running, still holding the single-instance
//! lock, and starting the application again will do nothing visible. That is
//! unrecoverable without Task Manager, which is not a thing to ask of somebody
//! who cannot see the screen.
//!
//! So the icon has to be there already, checked rather than assumed. An icon
//! that was asked for and did not appear is exactly the case this guards.

/// Why the window is being closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asked {
    /// The window's close button, or anything else that means "put this away".
    CloseTheWindow,
    /// Quit, from the menu, the key, or the tray. This ends the program.
    Quit,
}

/// What to do about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Closing {
    /// Really close. The program ends.
    LetItClose,
    /// Keep running with no window, reachable from the notification area.
    HideToTheTray,
}

/// What closing should do, given who asked and what is available.
///
/// `tray_icon_is_installed` is deliberately the state of the icon rather than
/// the setting that asked for one. The setting says what somebody wanted; only
/// the icon says whether there is a way back.
pub fn what_closing_should_do(
    asked: Asked,
    keep_running_in_the_tray: bool,
    tray_icon_is_installed: bool,
) -> Closing {
    match asked {
        Asked::Quit => Closing::LetItClose,
        Asked::CloseTheWindow if keep_running_in_the_tray && tray_icon_is_installed => {
            Closing::HideToTheTray
        }
        Asked::CloseTheWindow => Closing::LetItClose,
    }
}

/// What to say the first time the window goes to the tray instead of closing.
///
/// Said once per run rather than every time, because somebody who has learned
/// this does not need telling on every close, and a message that appears every
/// time is one that gets dismissed without reading.
///
/// It exists at all because hiding is indistinguishable from quitting to
/// somebody who cannot see the screen: the window goes, the reading stops, and
/// nothing says the program is still there.
pub fn what_hiding_should_say() -> &'static str {
    "Wixen Mail is still running, in the notification area. \
     Open it from there, or choose Quit to close it properly."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quit_ends_the_program_even_when_the_tray_is_asked_for() {
        // File then Quit and the window's close button reach the same handler
        // on Windows. A handler that only asked about the setting would turn
        // Quit into hide, and somebody who chose Quit from a menu would be
        // left with a program they were sure they had ended.
        assert_eq!(
            what_closing_should_do(Asked::Quit, true, true),
            Closing::LetItClose
        );
    }

    #[test]
    fn test_closing_the_window_hides_it_when_that_was_asked_for_and_there_is_a_way_back() {
        assert_eq!(
            what_closing_should_do(Asked::CloseTheWindow, true, true),
            Closing::HideToTheTray
        );
    }

    #[test]
    fn test_the_window_really_closes_when_the_tray_icon_is_not_there() {
        // The trap this exists for. Hiding the only window with no icon to
        // bring it back leaves a running program with no way in, still holding
        // the single-instance lock, so starting it again does nothing visible.
        // Recovering from that means Task Manager, which is not something to
        // ask of somebody who cannot see the screen.
        assert_eq!(
            what_closing_should_do(Asked::CloseTheWindow, true, false),
            Closing::LetItClose
        );
    }

    #[test]
    fn test_closing_the_window_closes_it_when_the_tray_was_never_asked_for() {
        // The default, and what every version before this did.
        assert_eq!(
            what_closing_should_do(Asked::CloseTheWindow, false, true),
            Closing::LetItClose
        );
        assert_eq!(
            what_closing_should_do(Asked::CloseTheWindow, false, false),
            Closing::LetItClose
        );
    }

    #[test]
    fn test_only_one_combination_of_the_four_hides() {
        // Stated as its own test because the failure that matters is a rule
        // that hides in a case it should not: every one of those cases is a
        // window somebody cannot get back.
        let mut hid = Vec::new();
        for asked in [Asked::CloseTheWindow, Asked::Quit] {
            for wanted in [true, false] {
                for installed in [true, false] {
                    if what_closing_should_do(asked, wanted, installed) == Closing::HideToTheTray {
                        hid.push((asked, wanted, installed));
                    }
                }
            }
        }
        assert_eq!(
            hid,
            vec![(Asked::CloseTheWindow, true, true)],
            "something other than a close with a working tray icon hides the window"
        );
    }

    #[test]
    fn test_hiding_says_the_program_is_still_running_and_how_to_end_it() {
        // Hiding is indistinguishable from quitting to somebody who cannot see
        // the screen: the window goes and nothing says the program is still
        // there. It also has to say how to really quit, or the answer to
        // "I want this closed" is to keep pressing something that hides it.
        let said = what_hiding_should_say();

        assert!(said.contains("still running"), "{said}");
        assert!(said.contains("notification area"), "{said}");
        assert!(said.contains("Quit"), "{said}");
    }
}

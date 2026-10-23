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

/// What has to happen to the icon after somebody changes the setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheIconShould {
    /// It was asked for and is not there.
    BePutThere,
    /// It is there and is no longer wanted.
    BeTakenAway,
    /// It already matches what was asked for.
    StayAsItIs,
}

/// Bring the icon into line with the setting, without waiting for a restart.
///
/// The icon used to be made once, at startup, from whatever the setting said
/// then. So somebody could tick the box, press OK, close the window, and have
/// the program end: the setting was on, no icon had ever been made, and the
/// rule above correctly refuses to hide a window with no way back to it. The
/// setting looked like it did nothing, and nothing said why.
///
/// Refusing to hide was right and is not the fault. The fault was making the
/// icon in one place and reading the setting in two.
pub fn what_the_setting_change_means(wanted: bool, icon_is_there: bool) -> TheIconShould {
    match (wanted, icon_is_there) {
        (true, false) => TheIconShould::BePutThere,
        (false, true) => TheIconShould::BeTakenAway,
        _ => TheIconShould::StayAsItIs,
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
    fn test_ticking_the_box_puts_the_icon_there_without_waiting_for_a_restart() {
        // The bug this was written for. The icon was made once at startup from
        // whatever the setting said then, so ticking the box, pressing OK and
        // closing the window ended the program: the setting was on, no icon
        // had ever been made, and the rule above correctly refused to hide a
        // window with no way back. The refusal was right; making the icon in
        // one place while reading the setting in two was not.
        assert_eq!(
            what_the_setting_change_means(true, false),
            TheIconShould::BePutThere
        );
    }

    #[test]
    fn test_unticking_the_box_takes_the_icon_away_rather_than_leaving_it_there() {
        // An icon left behind after the setting is off is one somebody clicks
        // expecting the program to be in the notification area, when closing
        // the window now really closes it.
        assert_eq!(
            what_the_setting_change_means(false, true),
            TheIconShould::BeTakenAway
        );
    }

    #[test]
    fn test_nothing_happens_when_the_icon_already_matches_the_setting() {
        // Opening settings and pressing OK without touching this must not
        // rebuild the icon: rebuilding it makes it disappear and come back in
        // the notification area for no reason.
        assert_eq!(
            what_the_setting_change_means(true, true),
            TheIconShould::StayAsItIs
        );
        assert_eq!(
            what_the_setting_change_means(false, false),
            TheIconShould::StayAsItIs
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

    /// The window's source as a release build compiles it.
    fn the_window_as_shipped() -> String {
        let whole = std::fs::read_to_string("src/presentation/wx_app.rs")
            .expect("the window's source to be readable")
            .replace("\r\n", "\n");
        crate::common::what_ships::what_ships(&whole)
    }

    /// The text from `opening` to the first of `endings` after it, or to the
    /// end of `source` when none follows.
    fn from_to<'a>(source: &'a str, opening: &str, endings: &[&str]) -> Option<&'a str> {
        let (_, after) = source.split_once(opening)?;
        let end = endings
            .iter()
            .filter_map(|ending| after.find(ending))
            .min()
            .unwrap_or(after.len());
        Some(&after[..end])
    }

    /// Whether the window's source closes for an installed update the way it
    /// closes for Quit, or why not.
    ///
    /// Asked of source because the handover cannot be reached from a test:
    /// its arm takes a `Verified`, which only `update_download::verify` makes,
    /// out of an installer signed by this project, and no test holds one.
    /// What this asks is the shape that makes the two the same path. One
    /// function sets the flag the close handler reads as a Quit and only then
    /// closes the frame; the Quit arm and the arm for a ready update both call
    /// it, and neither closes the frame on its own.
    fn whether_the_handover_quits_the_way_quit_does(window: &str) -> Result<(), String> {
        let shared = from_to(window, "\nfn really_quit(", &["\n}\n"])
            .ok_or("the window has no `fn really_quit(` for Quit and the handover to share")?;
        let flagged = shared
            .find(".set(true)")
            .ok_or("`really_quit` never tells the close handler this is a Quit")?;
        let closed = shared
            .find(".close(")
            .ok_or("`really_quit` never closes the frame")?;
        if closed < flagged {
            return Err(
                "`really_quit` closes the frame before it says this is a Quit, so \
                        the close handler reads a close of the window"
                    .into(),
            );
        }

        let quit = from_to(window, "_ if id == ID_QUIT =>", &["_ if id =="])
            .ok_or("the window has no arm for ID_QUIT")?;
        if !quit.contains("really_quit(") || quit.contains(".close(") {
            return Err("the Quit arm does not close through `really_quit` alone".into());
        }

        let handler = from_to(
            window,
            "fn handle_update(update: &UIUpdate, targets: UpdateTargets<'_>) {",
            &["\n}\n"],
        )
        .ok_or("the window has no update handler")?;
        let ready = from_to(
            handler,
            "\n        UIUpdate::AnUpdateIsReady",
            &["\n        UIUpdate::"],
        )
        .ok_or("the update handler has no arm for AnUpdateIsReady")?;
        if !ready.contains("really_quit(") || ready.contains(".close(") {
            return Err(
                "the arm for a ready update does not close through `really_quit` \
                        alone, so with the notification area setting on the close handler \
                        hides the window instead of quitting"
                    .into(),
            );
        }
        Ok(())
    }

    #[test]
    fn test_installing_an_update_quits_the_way_quit_does() {
        // Gap 2 of the research into the update mechanism (13.3-24). The
        // handover closed the window without saying it was a Quit, so with
        // "keep running in the notification area" on and the icon showing,
        // the close handler hid the window and said Wixen Mail was still
        // running, a moment after saying it was closing for the installer,
        // and skipped Quit's own work on the way out.
        assert_eq!(
            whether_the_handover_quits_the_way_quit_does(&the_window_as_shipped()),
            Ok(())
        );
    }

    #[test]
    fn test_that_reading_can_see_a_handover_that_closes_on_its_own() {
        // A reading that answers yes to everything would pass above against
        // the defect it is for, so it is shown a window that is right and two
        // that each carry one of the two ways to be wrong.
        let shaped = |shared: &str, handover: &str| {
            format!(
                "\nfn really_quit(really_quitting: &Cell<bool>, frame: &Frame) {{\n{shared}\n}}\n\
                 \x20                       _ if id == ID_QUIT => {{\n\
                 \x20                           really_quit(&really_quitting, &frame);\n\
                 \x20                       }}\n\
                 \x20                       _ if id == ID_OPEN_WINDOW => {{}}\n\
                 fn handle_update(update: &UIUpdate, targets: UpdateTargets<'_>) {{\n\
                 \x20   match update {{\n\
                 \x20       UIUpdate::AnUpdateIsReady {{ installer }} => {{\n\
                 \x20           {handover}\n\
                 \x20       }}\n\
                 \x20       UIUpdate::CommandRefused(why) => {{}}\n\
                 \x20   }}\n\
                 }}\n"
            )
        };
        let flag_then_close = "    really_quitting.set(true);\n    frame.close(false);";
        let close_then_flag = "    frame.close(false);\n    really_quitting.set(true);";

        assert_eq!(
            whether_the_handover_quits_the_way_quit_does(&shaped(
                flag_then_close,
                "really_quit(really_quitting, frame);"
            )),
            Ok(()),
            "a window that is right was refused"
        );
        assert!(
            whether_the_handover_quits_the_way_quit_does(&shaped(
                flag_then_close,
                "frame.close(true);"
            ))
            .is_err(),
            "a handover that closes the frame on its own was passed"
        );
        assert!(
            whether_the_handover_quits_the_way_quit_does(&shaped(
                close_then_flag,
                "really_quit(really_quitting, frame);"
            ))
            .is_err(),
            "a shared function that closes before it says Quit was passed"
        );
    }
}

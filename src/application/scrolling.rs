//! How a view moves: whether it slides or jumps, and whether it follows you.
//!
//! # Why the system setting wins
//!
//! Animated scrolling makes some people ill. Vestibular disorders are not a
//! preference and the reaction is not mild, which is why WCAG 2.3.3 exists and
//! why every operating system carries a switch for it. Windows calls that
//! switch "Show animations in Windows" and answers for it through
//! `SPI_GETCLIENTAREAANIMATION`.
//!
//! So this module has one rule that is not negotiable: when the machine says
//! reduce motion, motion is reduced, whatever this application's own setting
//! says. An application preference that could override the operating system
//! would mean somebody who has already told their computer once has to find
//! and turn off the same thing again in every program, and would only discover
//! this one by being made unwell by it.
//!
//! The application setting can therefore only ever turn smooth scrolling
//! *off*. It cannot turn it on against the system. That asymmetry is
//! deliberate and is what the tests here pin down.
//!
//! # Why this is not in the presentation layer
//!
//! Two places need the answer and they are not near each other: the reader's
//! web view, which scrolls with CSS, and the message list, which scrolls
//! through wxWidgets. One decision, made here, read by both.

/// How a view should move when it scrolls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    /// Slide to the new position.
    Smooth,
    /// Arrive at it. What everything did before this existed.
    Immediate,
}

impl Motion {
    /// The CSS value for the reader, which is a web view.
    ///
    /// `auto` rather than an empty string, because `scroll-behavior` has to say
    /// something: leaving the property out inherits whatever the page above set,
    /// and a message is a stranger's document.
    pub fn css_scroll_behavior(self) -> &'static str {
        match self {
            Motion::Smooth => "smooth",
            Motion::Immediate => "auto",
        }
    }
}

/// What the machine says about animation, kept apart from what it is used for.
///
/// A named type rather than a bare `bool`, because `how_to_scroll(true, false)`
/// at the call site says nothing about which `true` is which, and getting the
/// two the wrong way round would silently invert the one rule this module has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemMotion {
    /// Animation is allowed. The ordinary setting.
    Allowed,
    /// The machine has been told to reduce motion.
    Reduced,
}

/// How to scroll, given what was asked for and what the machine says.
///
/// The system's answer wins whenever it says to reduce motion. See the module
/// comment: this asymmetry is the whole point.
pub fn how_to_scroll(asked_for_smooth: bool, system: SystemMotion) -> Motion {
    match (asked_for_smooth, system) {
        (_, SystemMotion::Reduced) => Motion::Immediate,
        (true, SystemMotion::Allowed) => Motion::Smooth,
        (false, SystemMotion::Allowed) => Motion::Immediate,
    }
}

/// What to say in the settings screen about a switch the machine has overruled.
///
/// Empty when nothing has been overruled. A setting that silently does nothing
/// is worse than one that is absent, and somebody who ticks smooth scrolling
/// and sees no change deserves to be told why rather than left to conclude the
/// application is broken.
pub fn what_the_machine_has_overruled(asked_for_smooth: bool, system: SystemMotion) -> String {
    if asked_for_smooth && system == SystemMotion::Reduced {
        "Windows is set to reduce animation, so scrolling stays immediate. \
         Change it in Windows Settings, Accessibility, Visual effects."
            .to_string()
    } else {
        String::new()
    }
}

/// Whether the message list should bring the chosen row back into view when
/// the list is rebuilt underneath it.
///
/// Separate from motion, because it answers a different question: not how the
/// view moves but whether it moves at all without being asked. A sync finishing
/// while somebody reads down a list is the case, and both answers are
/// reasonable, which is what makes it a setting rather than a decision.
///
/// The selection itself never moves either way. Only the viewport does, so
/// turning this off cannot lose somebody their place, it can only leave that
/// place off screen until they arrow again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Following {
    /// Bring the chosen row back into view.
    KeepInView,
    /// Leave the view where it is.
    LeaveAlone,
}

impl Following {
    pub fn from_setting(keep_in_view: bool) -> Self {
        if keep_in_view {
            Following::KeepInView
        } else {
            Following::LeaveAlone
        }
    }

    pub fn should_scroll(self) -> bool {
        self == Following::KeepInView
    }
}

/// What the machine says about animation right now.
///
/// `SPI_GETCLIENTAREAANIMATION` is the Windows answer to "should things
/// animate", and it is what the accessibility setting writes to. A machine
/// that cannot be asked is treated as allowing animation, because that is what
/// every version of this application did before the question was asked at all,
/// and because guessing "reduced" would take smooth scrolling away from people
/// who never asked for it to go.
#[cfg(target_os = "windows")]
pub fn system_motion() -> SystemMotion {
    match ask_the_machine_about_animation() {
        Some(false) => SystemMotion::Reduced,
        // Animation allowed, or the machine could not be asked. Both answer
        // the same way and they are not the same fact, which is why the ask
        // is its own function: a test can see the difference even though the
        // caller does not need to.
        _ => SystemMotion::Allowed,
    }
}

/// Whether the machine wants things animated, or `None` if it would not say.
///
/// Split out from [`system_motion`] so the failure can be seen. Folded into the
/// answer, a call that never succeeds is indistinguishable from a machine that
/// allows animation, and this application would report "no reduced motion
/// setting found" forever while looking like it had checked.
#[cfg(target_os = "windows")]
fn ask_the_machine_about_animation() -> Option<bool> {
    /// `SPI_GETCLIENTAREAANIMATION`.
    const ASK_ABOUT_ANIMATION: u32 = 0x1042;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn SystemParametersInfoW(
            action: u32,
            param: u32,
            value: *mut core::ffi::c_void,
            update: u32,
        ) -> i32;
    }

    let mut animations_on: i32 = 1;
    let answered = unsafe {
        SystemParametersInfoW(ASK_ABOUT_ANIMATION, 0, (&raw mut animations_on).cast(), 0)
    };
    (answered != 0).then_some(animations_on != 0)
}

/// Everywhere else. The switch is a Windows one and this is not Windows.
///
/// Said rather than silently assumed: a port needs its own answer here, and
/// `prefers-reduced-motion` is what the other two platforms call it.
#[cfg(not(target_os = "windows"))]
pub fn system_motion() -> SystemMotion {
    SystemMotion::Allowed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_machine_set_to_reduce_motion_is_obeyed_over_this_applications_setting() {
        // The rule this module exists for. Somebody who has told Windows to
        // stop animating has said it once and should not have to say it again
        // here, and the way they would find out this application ignored them
        // is by being made unwell.
        assert_eq!(
            how_to_scroll(true, SystemMotion::Reduced),
            Motion::Immediate
        );
    }

    #[test]
    fn test_smooth_scrolling_happens_when_it_is_asked_for_and_the_machine_allows_it() {
        // The other half. A rule that only ever answered "immediate" would
        // pass the test above and make the setting do nothing at all.
        assert_eq!(how_to_scroll(true, SystemMotion::Allowed), Motion::Smooth);
    }

    #[test]
    fn test_not_asking_for_smooth_scrolling_leaves_it_immediate() {
        assert_eq!(
            how_to_scroll(false, SystemMotion::Allowed),
            Motion::Immediate
        );
    }

    #[test]
    fn test_the_setting_can_turn_motion_off_and_never_on_against_the_machine() {
        // Stated as its own test because it is the asymmetry, and the
        // asymmetry is the design. Every combination, so no arm can be
        // changed without something going red.
        for system in [SystemMotion::Allowed, SystemMotion::Reduced] {
            for asked in [true, false] {
                let got = how_to_scroll(asked, system);
                if system == SystemMotion::Reduced {
                    assert_eq!(got, Motion::Immediate, "{system:?} with asked={asked}");
                }
                if !asked {
                    assert_eq!(got, Motion::Immediate, "{system:?} with asked={asked}");
                }
            }
        }
    }

    #[test]
    fn test_somebody_overruled_by_their_machine_is_told_why() {
        // A ticked box that does nothing reads as a broken application. The
        // sentence says which switch won and where to find it.
        let said = what_the_machine_has_overruled(true, SystemMotion::Reduced);

        assert!(said.contains("reduce animation"), "{said}");
        assert!(said.contains("Windows Settings"), "{said}");
    }

    #[test]
    fn test_nothing_is_said_when_nothing_has_been_overruled() {
        // Three ways to have nothing to report, and a sentence in any of them
        // would be a warning about a problem somebody does not have.
        assert!(what_the_machine_has_overruled(true, SystemMotion::Allowed).is_empty());
        assert!(what_the_machine_has_overruled(false, SystemMotion::Reduced).is_empty());
        assert!(what_the_machine_has_overruled(false, SystemMotion::Allowed).is_empty());
    }

    #[test]
    fn test_the_reader_is_given_a_scroll_behaviour_either_way() {
        // A message is somebody else's document. Leaving the property out
        // would inherit whatever the page around it set, so both answers name
        // a value.
        assert_eq!(Motion::Smooth.css_scroll_behavior(), "smooth");
        assert_eq!(Motion::Immediate.css_scroll_behavior(), "auto");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_this_machine_really_answers_the_animation_question() {
        // Proves the check can see something before anything trusts it saying
        // nothing. A call that always failed would report "animation allowed"
        // on every machine on earth, including the ones whose owners have
        // turned animation off, and would look exactly like a working check.
        assert!(
            super::ask_the_machine_about_animation().is_some(),
            "Windows would not answer SPI_GETCLIENTAREAANIMATION, so the \
             reduced-motion setting is being guessed rather than read"
        );
    }

    #[test]
    fn test_following_the_chosen_row_is_what_the_setting_says() {
        assert!(Following::from_setting(true).should_scroll());
        assert!(!Following::from_setting(false).should_scroll());
    }
}

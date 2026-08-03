//! The visual design system: colour, spacing, and type.
//!
//! A mail client for blind users is not a reason to look unfinished. The people
//! this is for share screens, sit beside sighted colleagues, and are sighted
//! themselves often enough that "screen reader user" and "cannot see anything"
//! are not the same set. Low vision is the largest group of all, and it is
//! served by good visual design rather than in spite of it. So this application
//! gets a designed interface, and the design obeys the same rules as everything
//! else here.
//!
//! # The constraint that shapes everything
//!
//! wxWidgets draws native Windows controls, and those controls report
//! themselves to the accessibility API for free. A custom-drawn control does
//! not: it is a rectangle that has to be taught its own name, role, value and
//! state by hand, and this project has already shipped sixteen widgets that
//! looked named and were not.
//!
//! So the look does not come from replacing controls. It comes from the four
//! things that can be changed without giving up what the platform provides:
//! colour, spacing, type, and iconography. That is a tighter brief than a web
//! application has, and tight briefs are where the good work is.
//!
//! # Why the borders are darker than fashion
//!
//! Most design systems draw a border at around 1.3:1 against its background,
//! because it looks refined. WCAG 1.4.11 wants 3:1 for anything that is the
//! only thing identifying a component, which a text field's border is. Ours
//! meet 3:1, and the tests below refuse to let anybody soften them.
//!
//! Nothing here is carried by colour alone (1.4.1). Colour is the second
//! signal; the first is always a word, a shape, or a position.

/// A colour, as the platform wants it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// The relative luminance WCAG defines, in 0.0 to 1.0.
    fn luminance(self) -> f64 {
        fn channel(value: u8) -> f64 {
            let v = f64::from(value) / 255.0;
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * channel(self.r) + 0.7152 * channel(self.g) + 0.0722 * channel(self.b)
    }
}

/// The contrast between two colours, from 1.0 to 21.0.
///
/// The figure WCAG 1.4.3 and 1.4.11 are written in terms of. Text needs 4.5,
/// large text and user interface components need 3.
pub fn contrast(a: Rgb, b: Rgb) -> f64 {
    let (light, dark) = {
        let (la, lb) = (a.luminance(), b.luminance());
        if la >= lb { (la, lb) } else { (lb, la) }
    };
    (light + 0.05) / (dark + 0.05)
}

/// The colours of one theme.
///
/// Named by role rather than by shade, so a change of palette is a change in
/// one place and no call site has an opinion about which grey it wanted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    /// The main background: message lists, panels, dialogs.
    pub surface: Rgb,
    /// The second surface: sidebars, the folder tree, headers.
    ///
    /// Darker than [`Self::surface`] in a light theme and lighter in a dark
    /// one, which is not a quirk but how depth works when the background is
    /// already near black. Naming it "sunken" and subtracting produced two
    /// dark surfaces a fifth of a stop apart, which the tests caught.
    pub surface_alt: Rgb,
    /// Ordinary text.
    pub text: Rgb,
    /// Secondary text: snippets, dates, counts. Still meets 4.5:1.
    pub text_muted: Rgb,
    /// The brand colour. Links, selection, and the focus ring.
    pub accent: Rgb,
    /// Something is wrong: a failed send, an unreadable password.
    pub danger: Rgb,
    /// Something needs attention: spam, a suspicious message.
    pub warning: Rgb,
    /// Something worked.
    pub success: Rgb,
    /// The line around a control that is the only thing identifying it.
    pub border: Rgb,
}

impl Palette {
    /// Warm light. Off-white rather than white, which is easier to sit in
    /// front of for the hours a working day puts into a mail client.
    const LIGHT: Palette = Palette {
        surface: Rgb::new(0xFB, 0xFA, 0xF9),
        surface_alt: Rgb::new(0xF3, 0xEF, 0xEA),
        text: Rgb::new(0x1A, 0x18, 0x17),
        text_muted: Rgb::new(0x5A, 0x53, 0x50),
        accent: Rgb::new(0x5B, 0x21, 0xB6),
        danger: Rgb::new(0xA3, 0x1D, 0x1D),
        warning: Rgb::new(0x7C, 0x4A, 0x03),
        success: Rgb::new(0x1B, 0x5E, 0x20),
        border: Rgb::new(0x8C, 0x83, 0x7B),
    };

    /// Warm dark. Not black: a pure black surface with light text produces
    /// halation for a lot of people, which is the opposite of the point.
    const DARK: Palette = Palette {
        surface: Rgb::new(0x16, 0x13, 0x0F),
        surface_alt: Rgb::new(0x22, 0x1E, 0x19),
        text: Rgb::new(0xF5, 0xF0, 0xEA),
        text_muted: Rgb::new(0xB0, 0xA6, 0x9D),
        accent: Rgb::new(0xC4, 0xB5, 0xFD),
        danger: Rgb::new(0xF2, 0xB8, 0xB5),
        warning: Rgb::new(0xE8, 0xC4, 0x6A),
        success: Rgb::new(0x9C, 0xD6, 0xA0),
        border: Rgb::new(0x7A, 0x71, 0x6A),
    };

    /// Every colour meant to be read as text on this palette's surfaces.
    ///
    /// Only the contrast tests need this. It exists so they can iterate the
    /// roles rather than listing them and quietly missing one added later.
    #[cfg(test)]
    fn text_roles(&self) -> [(&'static str, Rgb); 6] {
        [
            ("text", self.text),
            ("text_muted", self.text_muted),
            ("accent", self.accent),
            ("danger", self.danger),
            ("warning", self.warning),
            ("success", self.success),
        ]
    }
}

/// The colours of the mark, which are not the colours of the interface.
///
/// Wixen Mail is one of a family, so the mark belongs to the family and not to
/// this application. It has to hold up in places the palette never goes: a
/// README on a white page, a GitHub avatar on a dark page, a favicon at sixteen
/// pixels, a printed page with no colour at all.
///
/// So it is two colours and one silhouette, and both colours are chosen to
/// clear 3:1 against every surface either theme can put behind them. That is
/// WCAG 1.4.11's floor for a meaningful graphic, and a logo that identifies the
/// application is meaningful by definition.
pub mod brand {
    use super::Rgb;

    /// The fox's coat. Deep enough to read as text if it ever has to.
    pub const FOX: Rgb = Rgb::new(0xC2, 0x41, 0x0C);
    /// The blindfold, the nose, and the wordmark on a light page.
    pub const INK: Rgb = Rgb::new(0x1C, 0x19, 0x17);
    /// The wordmark on a dark page, and the field behind the mark on a badge.
    pub const PAPER: Rgb = Rgb::new(0xFB, 0xFA, 0xF9);
}

/// Which palette to draw with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    /// Follow Windows.
    #[default]
    System,
    Light,
    Dark,
    /// Hand everything to Windows high contrast and draw nothing of our own.
    HighContrast,
}

impl Theme {
    /// The setting as stored.
    pub fn from_setting(stored: &str) -> Self {
        match stored {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            "high_contrast" => Theme::HighContrast,
            _ => Theme::System,
        }
    }

    /// How it is stored.
    pub fn as_str(self) -> &'static str {
        match self {
            Theme::System => "default",
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::HighContrast => "high_contrast",
        }
    }

    /// The palette to use, given whether Windows is currently dark.
    ///
    /// `None` for high contrast, which is not a palette we are entitled to an
    /// opinion about: somebody running high contrast has chosen their colours
    /// deliberately, often because nothing else is legible to them, and an
    /// application that paints over that has taken away the reason they set it.
    pub fn palette(self, system_is_dark: bool) -> Option<Palette> {
        match self {
            Theme::HighContrast => None,
            Theme::Light => Some(Palette::LIGHT),
            Theme::Dark => Some(Palette::DARK),
            Theme::System if system_is_dark => Some(Palette::DARK),
            Theme::System => Some(Palette::LIGHT),
        }
    }
}

/// Spacing, in device-independent pixels.
///
/// A four step scale. Anything not on it is a number somebody picked because it
/// looked right on their monitor, which is how a layout ends up with seven
/// different gaps that are all nearly the same.
pub mod space {
    /// Between a label and its control.
    pub const TIGHT: i32 = 4;
    /// Between controls in a row.
    pub const SNUG: i32 = 8;
    /// Between groups of controls.
    pub const ROOMY: i32 = 16;
    /// Between major regions, and a dialog's outer margin.
    pub const OPEN: i32 = 24;
}

/// The smallest a control may be, in device-independent pixels.
///
/// WCAG 2.5.8 asks for 24 by 24 and 2.5.5 asks for 44. Windows convention is
/// smaller than either, so this is the floor rather than the target: a toolbar
/// button gets the target, an inline control gets at least the floor.
pub const MIN_TARGET: i32 = 24;
/// What a control that is a primary action should be.
pub const COMFORTABLE_TARGET: i32 = 44;

// ── Putting it on the screen ────────────────────────────────────────────────

impl Rgb {
    /// The colour as wxWidgets wants it.
    fn wx(self) -> wxdragon::prelude::Colour {
        wxdragon::prelude::Colour::rgb(self.r, self.g, self.b)
    }
}

/// The palette to draw with right now.
///
/// `None` means draw nothing of our own and let Windows decide, which is the
/// answer both for high contrast and for anybody whose system is set up in a
/// way we have not thought of.
pub fn current(setting: &str) -> Option<Palette> {
    palette_for(
        setting,
        wxdragon::is_system_dark_mode(),
        windows_high_contrast(),
    )
}

/// Which palette follows from the setting and what the machine reports.
///
/// Split from [`current`] so the rule can be stated without asking the machine
/// anything. High contrast wins whatever the setting says: somebody running it
/// has chosen their colours, usually because nothing else is legible to them,
/// and an application that paints over that has removed the reason they set it.
/// That beats an explicit Light or Dark for the same reason.
fn palette_for(setting: &str, system_is_dark: bool, high_contrast: bool) -> Option<Palette> {
    if high_contrast {
        return None;
    }
    Theme::from_setting(setting).palette(system_is_dark)
}

/// Whether Windows is in a high contrast theme.
///
/// `SystemParametersInfo` with `SPI_GETHIGHCONTRAST`, because there is no
/// cross-platform way to ask and this is a Windows-first application. Anywhere
/// else the answer is no, which leaves the palette in charge, and that is
/// correct on a platform with no such mode.
fn windows_high_contrast() -> bool {
    #[cfg(target_os = "windows")]
    {
        #[repr(C)]
        struct HighContrast {
            size: u32,
            flags: u32,
            scheme: *mut u16,
        }
        const SPI_GETHIGHCONTRAST: u32 = 0x0042;

        #[link(name = "user32")]
        unsafe extern "system" {
            fn SystemParametersInfoW(
                action: u32,
                param: u32,
                data: *mut core::ffi::c_void,
                update: u32,
            ) -> i32;
        }

        let mut info = HighContrast {
            size: size_of::<HighContrast>() as u32,
            flags: 0,
            scheme: std::ptr::null_mut(),
        };
        // Safe: the struct is the shape the API documents, its size field is
        // set as the API requires, and nothing is read back unless the call
        // reported success.
        let ok = unsafe {
            SystemParametersInfoW(
                SPI_GETHIGHCONTRAST,
                size_of::<HighContrast>() as u32,
                (&raw mut info).cast(),
                0,
            )
        };
        high_contrast_from(ok, info.flags)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

/// The one bit in that flags word that says high contrast is on.
///
/// The word carries others, such as whether a high contrast scheme is merely
/// available, so it cannot be read as a whole.
const HCF_HIGHCONTRASTON: u32 = 0x0000_0001;

/// Read what Windows answered, separately from asking it.
///
/// Two decisions, and getting either wrong paints over the colours of somebody
/// who cannot read anything else. A failed call is not an answer: when `ok` is
/// zero the flags word was never filled in, so reading it is reading nothing,
/// and the safe reading is that high contrast is off and our palette stays in
/// charge. And only [`HCF_HIGHCONTRASTON`] means on, whatever else is set
/// beside it.
///
/// Outside the platform gate on purpose, so it is compiled and tested
/// everywhere rather than only where it runs.
const fn high_contrast_from(ok: i32, flags: u32) -> bool {
    ok != 0 && flags & HCF_HIGHCONTRASTON != 0
}

/// Paint a container, and only a container.
///
/// Colours go on panels rather than on the controls inside them. A control
/// given explicit colours stops following Windows high contrast, which is the
/// one setting that must always win, and stops following whatever else the
/// user has done to their system theme.
pub fn paint(window: &impl wxdragon::prelude::WxWidget, colour: Rgb) {
    window.set_background_color(colour.wx());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// WCAG's own worked example, and the two ends of the scale.
    #[test]
    fn test_the_contrast_figure_is_the_one_wcag_defines() {
        let black = Rgb::new(0, 0, 0);
        let white = Rgb::new(255, 255, 255);

        assert!((contrast(black, white) - 21.0).abs() < 0.01);
        assert!((contrast(white, white) - 1.0).abs() < 0.01);
        // Order does not matter: it is a ratio between two colours, not a
        // direction.
        assert!((contrast(black, white) - contrast(white, black)).abs() < 0.001);
    }

    #[test]
    fn test_a_channel_inside_the_linear_part_of_the_curve_is_divided_not_scaled() {
        // WCAG's transfer function is a straight line below 0.03928 of full
        // scale and a power curve above it, and dividing by 12.92 is that
        // line. Real palette colours go through it: the warning brown in the
        // light theme has a blue channel of 3. The other tests here only ever
        // push pure black through this branch, and zero comes back as zero
        // whatever the arithmetic is, so the line itself went unchecked.
        //
        // 10 is the largest channel value still inside the linear segment
        // (10/255 is 0.03922); 11 is outside it.
        let ratio = contrast(Rgb::new(10, 10, 10), Rgb::new(0, 0, 0));
        assert!(
            (ratio - 1.0607).abs() < 0.001,
            "near black against black came out at {ratio:.4}:1"
        );
    }

    #[test]
    fn test_green_carries_most_of_the_luminance() {
        // The three coefficients say how much each primary contributes to how
        // bright a colour looks, and they are not interchangeable. Swapping
        // red and green leaves black against white at 21:1 and white against
        // white at 1:1, so the formula test above would not notice, and every
        // palette figure has enough margin to stay green as well.
        let black = Rgb::new(0, 0, 0);
        let green = contrast(Rgb::new(0, 255, 0), black);
        let red = contrast(Rgb::new(255, 0, 0), black);
        let blue = contrast(Rgb::new(0, 0, 255), black);
        assert!(green > red, "green {green:.2} is not above red {red:.2}");
        assert!(red > blue, "red {red:.2} is not above blue {blue:.2}");
    }

    #[test]
    fn test_a_failed_question_to_windows_is_not_read_as_high_contrast() {
        // When the call reports failure the flags word was never filled in, so
        // whatever is sitting in it means nothing. Believing it would hand the
        // whole interface to a setting nobody asked for.
        assert!(!high_contrast_from(0, HCF_HIGHCONTRASTON));
        assert!(high_contrast_from(1, HCF_HIGHCONTRASTON));
    }

    #[test]
    fn test_only_the_high_contrast_on_bit_means_high_contrast_is_on() {
        // 0x02 is the neighbouring bit, which says a high contrast scheme is
        // available rather than in use. Reading the word as a whole, or with
        // the wrong operator, either paints over somebody's high contrast
        // colours or refuses to paint for everybody else.
        const HCF_AVAILABLE: u32 = 0x0000_0002;
        assert!(!high_contrast_from(1, 0));
        assert!(!high_contrast_from(1, HCF_AVAILABLE));
        assert!(high_contrast_from(1, HCF_HIGHCONTRASTON | HCF_AVAILABLE));
    }

    #[test]
    fn test_high_contrast_beats_a_theme_somebody_chose_by_hand() {
        // The precedence the comment on `current` states and nothing checked.
        // Somebody who picked Dark last year and switched high contrast on
        // this morning meant the high contrast.
        assert_eq!(palette_for("dark", false, true), None);
        assert_eq!(palette_for("light", true, true), None);
    }

    #[test]
    fn test_without_high_contrast_the_chosen_theme_is_what_gets_drawn() {
        assert_eq!(palette_for("light", true, false), Some(Palette::LIGHT));
        assert_eq!(palette_for("dark", false, false), Some(Palette::DARK));
        assert_eq!(palette_for("default", true, false), Some(Palette::DARK));
    }

    #[test]
    fn test_every_readable_colour_meets_four_and_a_half_to_one() {
        // 1.4.3. This is the test that stops the palette drifting towards
        // whatever looked nice in a screenshot.
        for (name, palette) in [("light", Palette::LIGHT), ("dark", Palette::DARK)] {
            for (role, colour) in palette.text_roles() {
                for (surface_name, surface) in [
                    ("surface", palette.surface),
                    ("surface_alt", palette.surface_alt),
                ] {
                    let ratio = contrast(colour, surface);
                    assert!(
                        ratio >= 4.5,
                        "{name}: {role} on {surface_name} is {ratio:.2}:1, needs 4.5"
                    );
                }
            }
        }
    }

    #[test]
    fn test_borders_meet_three_to_one() {
        // 1.4.11. A text field's border is the only thing saying there is a
        // text field there, so it is a user interface component and not
        // decoration. Most design systems get this wrong and look better for
        // it; we do not get to.
        for (name, palette) in [("light", Palette::LIGHT), ("dark", Palette::DARK)] {
            for (surface_name, surface) in [
                ("surface", palette.surface),
                ("surface_alt", palette.surface_alt),
            ] {
                let ratio = contrast(palette.border, surface);
                assert!(
                    ratio >= 3.0,
                    "{name}: border on {surface_name} is {ratio:.2}:1, needs 3.0"
                );
            }
        }
    }

    #[test]
    fn test_the_focus_ring_is_visible_against_both_surfaces() {
        // 2.4.11 and 2.4.13. A focus indicator nobody can see is the single
        // fastest way to make a keyboard-only interface unusable.
        for (name, palette) in [("light", Palette::LIGHT), ("dark", Palette::DARK)] {
            for surface in [palette.surface, palette.surface_alt] {
                let ratio = contrast(palette.accent, surface);
                assert!(
                    ratio >= 3.0,
                    "{name}: focus ring is {ratio:.2}:1, needs 3.0"
                );
            }
        }
    }

    #[test]
    fn test_the_two_surfaces_can_be_told_apart() {
        // Not an accessibility floor, since nothing is identified by the
        // difference, but a sidebar indistinguishable from the list beside it
        // is a design that failed at its own job.
        for (name, palette) in [("light", Palette::LIGHT), ("dark", Palette::DARK)] {
            let ratio = contrast(palette.surface, palette.surface_alt);
            assert!(ratio > 1.05, "{name}: the surfaces are the same colour");
        }
    }

    /// Every surface the mark can be asked to sit on.
    ///
    /// Both themes, plus plain white and plain black, because a README, a
    /// GitHub avatar and a printed page are none of our surfaces and the mark
    /// still has to work on them.
    #[cfg(test)]
    const MARK_BACKGROUNDS: [(&str, Rgb); 6] = [
        ("light surface", Palette::LIGHT.surface),
        ("light surface_alt", Palette::LIGHT.surface_alt),
        ("dark surface", Palette::DARK.surface),
        ("dark surface_alt", Palette::DARK.surface_alt),
        ("white", Rgb::new(0xFF, 0xFF, 0xFF)),
        ("black", Rgb::new(0x00, 0x00, 0x00)),
    ];

    #[test]
    fn test_the_mark_is_visible_wherever_it_is_put() {
        // 1.4.11. A logo that identifies the application is a meaningful
        // graphic, so 3:1 is a floor and not a preference. The dark surface is
        // the tight one: a fox orange light enough for black backgrounds is too
        // light for white ones, and this sits where both hold.
        for (name, background) in MARK_BACKGROUNDS {
            let ratio = contrast(brand::FOX, background);
            assert!(
                ratio >= 3.0,
                "the coat on {name} is {ratio:.2}:1, needs 3.0"
            );
        }
    }

    #[test]
    fn test_the_blindfold_can_be_told_from_the_coat() {
        // The band is the one piece of the mark carried by colour rather than
        // by outline, because it sits inside the silhouette. If it does not
        // separate from the coat it is not a blindfold, it is a smudge.
        let ratio = contrast(brand::INK, brand::FOX);
        assert!(ratio >= 3.0, "the band on the coat is {ratio:.2}:1");
    }

    #[test]
    fn test_the_knocked_out_form_holds_up_too() {
        // On a badge the mark inverts: a coloured field, the fox in cream, the
        // band still in ink. That is the form a favicon, an avatar and a
        // taskbar button all use, so it is the one most people see, and each
        // of its three layers has to separate from the one under it.
        assert!(
            contrast(brand::PAPER, brand::FOX) >= 3.0,
            "the cream fox does not separate from the field it sits on"
        );
        assert!(
            contrast(brand::INK, brand::PAPER) >= 3.0,
            "the band does not separate from the cream fox"
        );
    }

    #[test]
    fn test_the_wordmark_is_readable_as_text_on_either_page() {
        // It is a word, so 4.5:1 rather than 3:1, whichever way round it runs.
        for (light, dark) in [
            (Palette::LIGHT.surface, Palette::DARK.surface),
            (Rgb::new(0xFF, 0xFF, 0xFF), Rgb::new(0x00, 0x00, 0x00)),
        ] {
            assert!(contrast(brand::INK, light) >= 4.5, "ink on a light page");
            assert!(contrast(brand::PAPER, dark) >= 4.5, "paper on a dark page");
        }
    }

    #[test]
    fn test_the_mark_still_works_with_the_colour_taken_away() {
        // Printed in one colour, or faxed, or rendered by something that only
        // does black and white, the two parts collapse into one. That is fine
        // as long as what is left is a silhouette rather than a blank: both
        // brand colours are dark, so both land on the same side of any
        // sensible threshold and the fox stays a fox.
        for (name, colour) in [("coat", brand::FOX), ("band", brand::INK)] {
            assert!(
                contrast(colour, Rgb::new(0xFF, 0xFF, 0xFF))
                    > contrast(colour, Rgb::new(0x00, 0x00, 0x00)),
                "{name} is closer to white than to black, so it drops out when \
                 the colour goes"
            );
        }
    }

    #[test]
    fn test_high_contrast_gets_no_palette_of_ours() {
        // Somebody running high contrast has chosen their colours, often
        // because nothing else is legible. Painting over that removes the
        // reason they set it.
        assert_eq!(Theme::HighContrast.palette(false), None);
        assert_eq!(Theme::HighContrast.palette(true), None);
    }

    #[test]
    fn test_the_system_theme_follows_the_system() {
        assert_eq!(Theme::System.palette(true), Some(Palette::DARK));
        assert_eq!(Theme::System.palette(false), Some(Palette::LIGHT));
    }

    #[test]
    fn test_a_chosen_theme_ignores_the_system() {
        // Somebody who picked dark meant dark, whatever Windows is doing.
        assert_eq!(Theme::Light.palette(true), Some(Palette::LIGHT));
        assert_eq!(Theme::Dark.palette(false), Some(Palette::DARK));
    }

    #[test]
    fn test_every_theme_survives_being_stored() {
        for theme in [
            Theme::System,
            Theme::Light,
            Theme::Dark,
            Theme::HighContrast,
        ] {
            assert_eq!(Theme::from_setting(theme.as_str()), theme);
        }
    }

    #[test]
    fn test_an_unknown_theme_falls_back_to_following_the_system() {
        assert_eq!(Theme::from_setting("chartreuse"), Theme::System);
        assert_eq!(Theme::from_setting(""), Theme::System);
    }

    /// Ascending and distinct, or it is not a scale, it is four numbers.
    ///
    /// A compile-time check rather than a test: these are constants, so a
    /// runtime assertion would only ever confirm what the compiler already
    /// knew. This one fails the build instead.
    const _SCALE_ASCENDS: () = {
        assert!(space::TIGHT < space::SNUG);
        assert!(space::SNUG < space::ROOMY);
        assert!(space::ROOMY < space::OPEN);
    };

    /// The floors WCAG 2.5.8 and 2.5.5 ask for.
    const _TARGETS_MEET_WCAG: () = {
        assert!(MIN_TARGET >= 24);
        assert!(COMFORTABLE_TARGET >= 44);
    };
}

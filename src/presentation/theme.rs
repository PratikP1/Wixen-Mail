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
    pub const LIGHT: Palette = Palette {
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
    pub const DARK: Palette = Palette {
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
    pub fn wx(self) -> wxdragon::prelude::Colour {
        wxdragon::prelude::Colour::rgb(self.r, self.g, self.b)
    }
}

/// The palette to draw with right now.
///
/// `None` means draw nothing of our own and let Windows decide, which is the
/// answer both for high contrast and for anybody whose system is set up in a
/// way we have not thought of.
pub fn current(setting: &str) -> Option<Palette> {
    if windows_high_contrast() {
        // Whatever the setting says. Somebody running high contrast has chosen
        // their colours, usually because nothing else is legible to them, and
        // an application that paints over that has removed the reason they set
        // it. This wins over an explicit Light or Dark for the same reason.
        return None;
    }
    Theme::from_setting(setting).palette(wxdragon::is_system_dark_mode())
}

/// Whether Windows is in a high contrast theme.
///
/// `SystemParametersInfo` with `SPI_GETHIGHCONTRAST`, because there is no
/// cross-platform way to ask and this is a Windows-first application. Anywhere
/// else the answer is no, which leaves the palette in charge, and that is
/// correct on a platform with no such mode.
pub fn windows_high_contrast() -> bool {
    #[cfg(target_os = "windows")]
    {
        #[repr(C)]
        struct HighContrast {
            size: u32,
            flags: u32,
            scheme: *mut u16,
        }
        const SPI_GETHIGHCONTRAST: u32 = 0x0042;
        const HCF_HIGHCONTRASTON: u32 = 0x0000_0001;

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
        ok != 0 && info.flags & HCF_HIGHCONTRASTON != 0
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
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

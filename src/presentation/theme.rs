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

/// A background and the text colour that has been tested against it.
///
/// The two travel together because separating them is how the folder list came
/// to be painted a near-black grey while its text stayed the near-black
/// Windows had given it. Setting a background on a control and leaving its text
/// alone is a colour choice about text, made by not making one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Surface {
    pub background: Rgb,
    pub text: Rgb,
}

impl Palette {
    /// The main reading surface: message lists, panels, dialogs.
    pub fn main_surface(self) -> Surface {
        Surface {
            background: self.surface,
            text: self.text,
        }
    }

    /// The second surface: sidebars, the folder tree, headers.
    pub fn second_surface(self) -> Surface {
        Surface {
            background: self.surface_alt,
            text: self.text,
        }
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
/// A four step scale, and nothing lays out to it yet: the window passes its own
/// literals to every sizer. This is the scale a layout pass would adopt, not a
/// description of the one on the screen.
///
/// The point of the scale, when something does adopt it, is that anything off
/// it is a number somebody picked because it looked right on their monitor,
/// which is how a layout ends up with seven different gaps that are all nearly
/// the same.
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
/// NOTHING IS SIZED TO THIS. No control in the window asks for it, so what is
/// on the screen is whatever wxWidgets chose, and whether it clears 24 by 24 is
/// unmeasured. This is the floor a sizing pass would work to, written down so
/// the number is not invented twice.
///
/// WCAG 2.5.8 asks for 24 by 24 and 2.5.5 asks for 44. Windows convention is
/// smaller than either, so this is a floor rather than a target: the intent is
/// that a toolbar button gets the target and an inline control gets at least
/// the floor.
pub const MIN_TARGET: i32 = 24;
/// What a control that is a primary action should be. Also unused, for the same
/// reason.
pub const COMFORTABLE_TARGET: i32 = 44;

// ── Putting it on the screen ────────────────────────────────────────────────

impl Rgb {
    /// The colour as wxWidgets wants it.
    fn wx(self) -> wxdragon::prelude::Colour {
        wxdragon::prelude::Colour::rgb(self.r, self.g, self.b)
    }
}

/// How far the theme reaches, in the words a person reads.
///
/// The setting used to be described as though picking Dark made the
/// application dark. It applies the palette to the sidebar and content area
/// of every module and to the windows a message can open into, and a note
/// that does not say so leaves somebody hunting for the reason the rest of
/// the window did not change.
///
/// It lives here rather than in the settings dialog so the sentence and the
/// code it describes sit together, and so a test can read it. What no test
/// here can say is whether anybody sees or hears it.
pub const REACH: &str = "Colour is applied to the sidebar and content area of \
     every module: Mail, Calendar, Contacts, Reminders, Tasks and Notes. It \
     also reaches the window a message opens into for reading, and the \
     window that shows a conversation as headings. Everything else follows \
     Windows. Changing it here recolours them immediately, with nothing to \
     restart. Default now matches whether Windows itself is set to light \
     or dark.";

/// The palette to draw with right now.
///
/// `None` means draw nothing of our own and let Windows decide, which is the
/// answer both for high contrast and for anybody whose system is set up in a
/// way we have not thought of.
pub fn current(setting: &str) -> Option<Palette> {
    palette_for(
        setting,
        // Read straight from Windows, through the registry, rather than
        // through wxWidgets: see `windows_prefers_dark` and `dark_mode_from`
        // for the detail. This still never calls `AppAppearance::set_appearance`.
        // That call recolours every native control in the application at
        // once, which is a change only eyes on a running build can accept or
        // reject, so it stays out of here; all this chooses is which of our
        // own two palettes to paint over the three surfaces `REACH` names,
        // and Windows keeps drawing everything else however it always has.
        // A read that fails, or answers something that is neither light nor
        // dark, falls back to light, the same safe default as before.
        windows_prefers_dark(),
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

/// Whether Windows itself prefers a dark or a light app appearance.
///
/// `SystemParametersInfo` has no question for this; it lives in the registry
/// instead, as `AppsUseLightTheme`. This composition mirrors
/// [`windows_high_contrast`] exactly: [`ask_windows_about_light_or_dark`]
/// does the asking and holds no decision, [`dark_mode_from`] does the reading
/// and is compiled and tested everywhere.
fn windows_prefers_dark() -> bool {
    let (status, apps_use_light_theme) = ask_windows_about_light_or_dark();
    dark_mode_from(status, apps_use_light_theme)
}

/// Nobody to ask off Windows, so nothing was answered.
///
/// The registry call this stands in for reports success as `0`, the opposite
/// of `SystemParametersInfoW` in [`ask_windows_about_high_contrast`], where
/// `0` is failure. A refused call has to look like a refused call here too,
/// so this answers a status [`dark_mode_from`] does not read as success,
/// rather than the `(0, 0)` a copy of that sibling stub would reach for.
#[cfg(not(target_os = "windows"))]
fn ask_windows_about_light_or_dark() -> (i32, u32) {
    (1, 0)
}

/// Put the question to Windows, and answer with what came back untouched.
///
/// Reads `AppsUseLightTheme`, under
/// `Software\Microsoft\Windows\CurrentVersion\Themes\Personalize` in the
/// current user's part of the registry, which is where Windows itself keeps
/// this preference; there is no dedicated Win32 call for it the way there is
/// for high contrast. Holds no decision, the way `date_display::read_locale`
/// holds none: the reading is [`dark_mode_from`], which is compiled and
/// tested everywhere.
///
/// A read. Nothing here writes a value, and nothing here can change what
/// Windows is set to.
///
/// Split out so a test can say the call still works. Whatever comes back is
/// returned untouched, matching [`ask_windows_about_high_contrast`]'s own
/// doc: interpreting it is [`dark_mode_from`]'s job, not this one's.
#[cfg(target_os = "windows")]
fn ask_windows_about_light_or_dark() -> (i32, u32) {
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn RegGetValueW(
            hkey: isize,
            sub_key: *const u16,
            value: *const u16,
            flags: u32,
            out_type: *mut u32,
            out_data: *mut core::ffi::c_void,
            out_size: *mut u32,
        ) -> i32;
    }

    // Windows documents `HKEY_CURRENT_USER` as `0x80000001` read as a signed
    // 32 bit value and then sign-extended to pointer width. Casting straight
    // from `u32` to `isize` would zero-extend instead and hand the API a
    // handle it has never heard of.
    const HKEY_CURRENT_USER: isize = 0x8000_0001u32 as i32 as isize;
    const RRF_RT_REG_DWORD: u32 = 0x0000_0010;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    let sub_key = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
    let value_name = wide("AppsUseLightTheme");
    let mut data: u32 = 0;
    let mut size: u32 = size_of::<u32>() as u32;
    // Safe: `sub_key` and `value_name` are null-terminated UTF-16 buffers
    // kept alive for the whole call, `data` is a plain four byte buffer that
    // matches `RRF_RT_REG_DWORD`, and `size` tells the API exactly how big
    // it is. A refused call leaves `data` at the zero it was set to here,
    // and `dark_mode_from` reads the status before it ever reads `data`.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            sub_key.as_ptr(),
            value_name.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            (&raw mut data).cast(),
            &raw mut size,
        )
    };
    (status, data)
}

/// Read what Windows answered, separately from asking it.
///
/// The registry call this reads is an `LSTATUS`, where zero is success, the
/// opposite convention from `SystemParametersInfoW` in [`high_contrast_from`],
/// where nonzero is success. Getting this backwards reads every real answer
/// as a failure and Default silently stays light forever: the exact bug this
/// function exists to fix, reached again through code that looks correct and
/// compiles clean.
///
/// A failed call never touched `apps_use_light_theme`, so it is read as
/// light, the same safe default as before. And only an exact `0` reads as
/// dark: Microsoft documents this value as `0` for dark and `1` for light, so
/// anything else, `2`, `0xFFFF_FFFF`, whatever a future Windows release
/// invents, is an answer nobody has written down and falls back to light
/// rather than being guessed at.
///
/// Outside the platform gate on purpose, so it is compiled and tested
/// everywhere rather than only where it runs.
const fn dark_mode_from(status: i32, apps_use_light_theme: u32) -> bool {
    status == 0 && apps_use_light_theme == 0
}

/// Whether Windows is in a high contrast theme.
///
/// `SystemParametersInfo` with `SPI_GETHIGHCONTRAST`, because there is no
/// cross-platform way to ask and this is a Windows-first application. Anywhere
/// else there is nobody to ask, and the answer works out as no, which leaves
/// the palette in charge and is correct on a platform with no such mode.
///
/// No platform gate in here. It used to answer a hand-written `false` off
/// Windows, which is a second place for the same decision to be made and the
/// only one of the two a test on this machine cannot reach. The gate is on the
/// asking now, so this composition is the same code everywhere and one test
/// covers it everywhere.
fn windows_high_contrast() -> bool {
    let (ok, flags) = ask_windows_about_high_contrast();
    high_contrast_from(ok, flags)
}

/// Nobody to ask off Windows, so nothing was answered.
///
/// `(0, 0)` is what a refused call looks like, and [`high_contrast_from`] reads
/// a refused call as high contrast off. That is the right answer on a platform
/// with no such mode, and it is reached through the same tested reading rather
/// than through a `false` written out a second time.
#[cfg(not(target_os = "windows"))]
fn ask_windows_about_high_contrast() -> (i32, u32) {
    (0, 0)
}

/// Put the question to Windows, and answer with what came back untouched.
///
/// Whether the call reported success, and the flags word it filled in. Holds no
/// decision, the way `date_display::read_locale` holds none: the reading is
/// [`high_contrast_from`], which is compiled and tested everywhere.
///
/// Split out so a test can say the call still works. Everything about this call
/// that can be wrong is invisible from its answer: the struct has a pointer in
/// it, so its size differs between 32 and 64 bit builds and the API rejects the
/// call outright if the size field disagrees with what it was handed. A rejected
/// call answers zero, [`high_contrast_from`] correctly reads that as "no
/// answer", and the palette stays in charge forever. That is our colours painted
/// over the colours of somebody who turned high contrast on because nothing else
/// is legible to them, and it would look exactly like working code.
#[cfg(target_os = "windows")]
fn ask_windows_about_high_contrast() -> (i32, u32) {
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
    (ok, info.flags)
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

/// Give a window a background and the text colour tested against it.
///
/// Both, always, which is what taking a [`Surface`] rather than two colours is
/// for. A control handed only a background keeps whatever text colour Windows
/// gave it, and the two were chosen by different people for different
/// backgrounds.
///
/// This is only ever reached with a palette in hand, and [`current`] returns
/// none under Windows high contrast, so a person running high contrast gets
/// their own colours and no call is made here at all. That is the one setting
/// that must always win.
///
/// Whether a given control obeys either colour is a question about the native
/// control underneath it. wxWidgets forwards both to Windows and some controls
/// ignore one or the other, so a call here is a request and not a result.
///
/// `?Sized` so a `&dyn WxWidget` can be handed in as well as a concrete
/// control: repainting an open window when the Theme setting changes means
/// calling this once for each of a mixture of widget types, and a trait
/// object is how that is held in one list.
pub fn paint(window: &(impl wxdragon::prelude::WxWidget + ?Sized), surface: Surface) {
    window.set_background_color(surface.background.wx());
    window.set_foreground_color(surface.text.wx());
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

    /// The question to Windows really is asked and really is answered.
    ///
    /// This asserts the call succeeded and that Windows filled the value in
    /// with one of the two answers Microsoft documents. It must never assert
    /// which one: that is the state of the machine running the suite, and a
    /// test that pinned it would go red the moment Pratik switches his own
    /// machine's theme to do an accessibility pass, which is the one time
    /// the suite must not be lying to him.
    ///
    /// What it does not prove: nothing at all about what a person sees.
    /// Only a pass on a machine actually set to each answer proves that.
    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_still_answers_when_asked_about_light_or_dark() {
        let (status, value) = ask_windows_about_light_or_dark();

        assert!(
            status == 0,
            "Windows refused the light or dark registry read, so Default could never follow it"
        );
        assert!(
            value == 0 || value == 1,
            "AppsUseLightTheme came back as {value}, which is not a documented value"
        );
    }

    /// The answer handed to the palette is the machine's, not a constant.
    ///
    /// The expected side works the bit out longhand rather than calling
    /// [`dark_mode_from`], so the two sides are not the same code and a wrong
    /// reading cannot move them together.
    ///
    /// It asserts nothing about whether the machine is light or dark, so it
    /// stays green whichever way Pratik's own machine is set.
    #[test]
    fn test_the_light_or_dark_answer_is_the_machines_own_and_not_a_constant() {
        let (status, value) = ask_windows_about_light_or_dark();
        let machine_says_dark = status == 0 && value == 0;

        assert_eq!(
            windows_prefers_dark(),
            machine_says_dark,
            "Windows answered {status} with AppsUseLightTheme {value} and the palette was told otherwise"
        );
    }

    #[test]
    fn test_the_registry_answer_zero_means_dark_and_one_means_light() {
        // Microsoft documents `AppsUseLightTheme` as `0` for dark and `1` for
        // light, the opposite sense of a name like `HCF_HIGHCONTRASTON` where
        // the bit itself is the "on" state.
        assert!(dark_mode_from(0, 0));
        assert!(!dark_mode_from(0, 1));
    }

    #[test]
    fn test_a_failed_or_unrecognised_registry_read_is_not_read_as_dark() {
        // A failed call never touched `apps_use_light_theme`, so whatever is
        // sitting in it means nothing. Believing a stray zero there would
        // hand the whole interface to a setting nobody asked for, the same
        // mistake a failed high contrast call would make if it were read the
        // same way.
        assert!(!dark_mode_from(1, 0));
        // A successful read of a value nobody has documented is not a
        // documented answer either.
        assert!(!dark_mode_from(0, 2));
        assert!(!dark_mode_from(0, 0xFFFF_FFFF));
    }

    /// The neighbouring bit: a high contrast scheme is available, as opposed to
    /// in use. Windows sets it on any desktop install and leaves it set when
    /// somebody switches high contrast on.
    const HCF_AVAILABLE: u32 = 0x0000_0002;

    /// The question to Windows really is asked and really is answered.
    ///
    /// Everything that can be wrong with this call is invisible from its
    /// answer. The struct carries a pointer, so it is a different size in a 32
    /// and a 64 bit build, and Windows refuses the call outright when the size
    /// field disagrees with the struct it was handed. A refused call answers
    /// zero, which is correctly read as "no answer", and the palette then stays
    /// in charge for good. Somebody running high contrast gets our colours
    /// painted over theirs and nothing anywhere says so.
    ///
    /// This asserts the call succeeded and that Windows filled the word in. It
    /// must never assert anything about `HCF_HIGHCONTRASTON`. That bit is the
    /// state of the machine running the suite, so a test that reads it goes red
    /// the moment Pratik switches high contrast on to do an accessibility pass,
    /// which is the one time the suite must not be lying to him. `HCF_AVAILABLE`
    /// is a different kind of fact: it says the platform has the feature, and it
    /// stays set whether high contrast is on or off.
    ///
    /// What it does not prove: nothing at all about what a person running high
    /// contrast sees. Only a pass with it switched on answers that.
    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_still_answers_when_asked_about_high_contrast() {
        let (ok, flags) = ask_windows_about_high_contrast();

        assert!(
            ok != 0,
            "Windows refused the high contrast question, so we would never know it was on"
        );
        assert!(
            flags & HCF_AVAILABLE != 0,
            "the flags word came back as {flags:#010x}, which does not name a scheme as available"
        );
    }

    /// The answer handed to the palette is the machine's, not a constant.
    ///
    /// The leaf that asks Windows has a test and the reading of its answer has
    /// two, and the composition that joins them had none, so both of its
    /// mutants lived: a body replaced by `true` or by `false` passed the whole
    /// suite. This calls it and holds it against what Windows said a moment
    /// ago.
    ///
    /// The expected side works the bit out longhand rather than calling
    /// [`high_contrast_from`], so the two sides are not the same code and a
    /// wrong reading cannot move them together.
    ///
    /// It asserts nothing about whether high contrast is on, so it stays green
    /// when Pratik switches it on for an accessibility pass, which is the one
    /// time the suite must not be lying to him.
    ///
    /// What it cannot do. It kills only the constant that disagrees with this
    /// machine right now: with high contrast off, "always on" dies here and
    /// "always off" does not, and "always off" is the harmful one, because that
    /// is the answer that paints our palette over the colours of somebody who
    /// turned high contrast on because nothing else is legible to them. Killing
    /// that one needs the suite run with high contrast switched on, which is a
    /// decision about somebody's own machine and not one a test may make.
    #[test]
    fn test_the_high_contrast_answer_is_the_machines_own_and_not_a_constant() {
        let (ok, flags) = ask_windows_about_high_contrast();
        let machine_says_on = ok != 0 && flags & HCF_HIGHCONTRASTON != 0;

        assert_eq!(
            windows_high_contrast(),
            machine_says_on,
            "Windows answered {ok} with flags {flags:#010x} and the palette was told otherwise"
        );
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

    /// `palette_for` takes two bare `bool`s in a row, and a swap at
    /// [`current`]'s call site compiles and passes every test that calls
    /// `palette_for` directly, since none of them go through `current` at
    /// all. The expected side is worked out longhand here, not by calling
    /// `palette_for`, so the same swap cannot move both sides together.
    ///
    /// It asserts nothing about whether the machine is light, dark, or in
    /// high contrast, so it stays green whichever way Pratik's own machine
    /// is set.
    #[test]
    fn test_current_picks_the_right_palette_from_the_dark_and_high_contrast_readings() {
        let expected = if windows_high_contrast() {
            None
        } else if windows_prefers_dark() {
            Some(Palette::DARK)
        } else {
            Some(Palette::LIGHT)
        };
        assert_eq!(current("default"), expected);
    }

    #[test]
    fn test_every_readable_colour_in_the_palette_meets_four_and_a_half_to_one() {
        // 1.4.3. This is the test that stops the palette drifting towards
        // whatever looked nice in a screenshot.
        //
        // It checks the palette. What reaches the screen is a separate
        // question and only looking at a running build answers it. The palette
        // is applied to three places, listed in `REACH`; everywhere else in
        // the application these colours are arithmetic and nothing more.
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
    fn test_the_theme_note_names_how_far_the_colour_reaches_and_when_it_arrives() {
        // The sentence shown under the Theme setting. It has to name every
        // module the palette reaches, the two windows a message can open
        // into, say the rest is left to Windows, and say the change is
        // immediate, because somebody who is told none of that reads
        // the setting as broken and changes it again.
        //
        // This checks the sentence. Whether it is displayed, whether it is in
        // the accessibility tree, and whether a screen reader reaches it when
        // focus lands on the Theme choice are three separate questions, and
        // none of them is answered here.
        for place in [
            "Mail",
            "Calendar",
            "Contacts",
            "Reminders",
            "Tasks",
            "Notes",
            "reading",
            "conversation as headings",
        ] {
            assert!(
                REACH.contains(place),
                "the note does not name {place}: {REACH}"
            );
        }
        assert!(REACH.contains("follows Windows"), "{REACH}");
        assert!(REACH.contains("immediately"), "{REACH}");
        // A wrapped string literal that loses its continuations keeps every
        // space of the indenting, and this one is read aloud. Runs of stray
        // spaces are silence in the middle of a sentence.
        assert!(!REACH.contains("  "), "{REACH}");
    }

    #[test]
    fn test_the_theme_note_says_default_follows_the_real_system_preference() {
        // Default used to mean light always, because nothing here had ever
        // asked Windows. `current` asks now, so the sentence has to say what
        // is true today rather than what used to be true.
        assert!(!REACH.contains("has not yet asked"), "{REACH}");
        assert!(REACH.contains("Default"), "{REACH}");
        assert!(REACH.contains("light or dark"), "{REACH}");
    }

    #[test]
    fn test_a_palette_surface_is_only_handed_out_with_the_text_colour_tested_against_it() {
        // A background without its text colour is how the folder list came to
        // be dark grey with near-black text on it. The pair is a type here so
        // that nobody can ask for half of it, and this holds the pair to the
        // floor for reading text.
        //
        // It checks the palette: the two colours the code asks for are a legal
        // pair. Whether the control underneath honours either of them is a
        // question about wxWidgets and Windows that only a running build
        // answers.
        for (name, palette) in [("light", Palette::LIGHT), ("dark", Palette::DARK)] {
            for (which, surface) in [
                ("main", palette.main_surface()),
                ("second", palette.second_surface()),
            ] {
                let ratio = contrast(surface.background, surface.text);
                assert!(
                    ratio >= 4.5,
                    "{name}: the {which} surface reads at {ratio:.2}:1, needs 4.5"
                );
            }
        }
    }

    #[test]
    fn test_a_dark_palette_surface_under_the_windows_text_colour_would_be_unreadable() {
        // Why the pair has to travel together. This is not a regression test:
        // it is arithmetic on two constants, it was true before the pair
        // existed and it is true after, and it never touches a control.
        //
        // Wixen Mail has never asked Windows to give it dark mode, so a
        // control's inherited text colour is the light theme's near-black.
        // Hand such a control the dark second surface and nothing else, and
        // its text is near-black on a near-black ground.
        const WINDOWS_TEXT_UNTIL_WE_ASK_FOR_DARK_MODE: Rgb = Rgb::new(0, 0, 0);
        let ratio = contrast(
            Palette::DARK.surface_alt,
            WINDOWS_TEXT_UNTIL_WE_ASK_FOR_DARK_MODE,
        );
        assert!(
            ratio < 3.0,
            "the dark second surface reads at {ratio:.2}:1 against the text \
             colour Windows supplies, which is above the floor this test \
             exists to record it falling below"
        );
    }

    #[test]
    fn test_the_palette_borders_meet_three_to_one() {
        // 1.4.11. A text field's border is the only thing saying there is a
        // text field there, so it is a user interface component and not
        // decoration. Most design systems get this wrong and look better for
        // it; we do not get to.
        //
        // It checks the palette, and `border` is painted on nothing at all:
        // every border on the screen is drawn by Windows. So this keeps a
        // colour ready for the day something uses it, and says nothing about
        // any border a person can see today.
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
    fn test_the_palette_focus_colour_clears_three_to_one_on_both_surfaces() {
        // 2.4.11 and 2.4.13. A focus indicator nobody can see is the single
        // fastest way to make a keyboard-only interface unusable.
        //
        // It checks the palette, and nothing draws a focus ring from `accent`:
        // the ring on the screen is the one Windows draws. This is the colour
        // we would use, held to the floor, and not a measurement of the ring
        // anybody currently sees.
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
    fn test_the_two_palette_surfaces_can_be_told_apart() {
        // Not an accessibility floor, since nothing is identified by the
        // difference, but a sidebar indistinguishable from the list beside it
        // is a design that failed at its own job.
        //
        // It checks the palette. Both surfaces are painted somewhere, so this
        // one is closer to the screen than its neighbours here, and still only
        // a running build shows whether the two read as different.
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
    fn test_the_marks_blindfold_can_be_told_from_its_coat() {
        // The band is the one piece of the mark carried by colour rather than
        // by outline, because it sits inside the silhouette. If it does not
        // separate from the coat it is not a blindfold, it is a smudge.
        //
        // These constants are a second copy of the colours in the artwork
        // under assets/brand, kept in step by hand. So this checks the copy.
        let ratio = contrast(brand::INK, brand::FOX);
        assert!(ratio >= 3.0, "the band on the coat is {ratio:.2}:1");
    }

    #[test]
    fn test_the_knocked_out_form_of_the_mark_holds_up_too() {
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

    #[test]
    fn test_every_palette_contrast_check_is_named_after_the_palette() {
        // The mistake this file is one rename away from making. A test called
        // "the focus ring is visible against both surfaces" reads aloud as a
        // claim about the running window, and every assertion in this file is
        // arithmetic on two constants. So any contrast check on a palette or
        // brand colour has to say "palette" or "mark" in its name, and this
        // fails the build when one does not.
        //
        // What it checks is the names in this file. It cannot check what any
        // of them prove, and neither this test nor any of the ones it guards
        // says anything about what reaches a screen.
        const SOURCE: &str = include_str!("theme.rs");
        // Joined at compile time so the needle does not appear whole in the
        // file it searches. Written as one literal, this test finds itself and
        // reports a fragment of its own body as a test name.
        const START_OF_A_TEST: &str = concat!("    fn ", "test_");
        let mut misnamed: Vec<&str> = Vec::new();
        for chunk in SOURCE.split(START_OF_A_TEST).skip(1) {
            let Some((name, body)) = chunk.split_once('(') else {
                continue;
            };
            let checks_a_designed_colour = body.contains("Palette::") || body.contains("brand::");
            if !checks_a_designed_colour || !body.contains("contrast(") {
                continue;
            }
            if !name.contains("palette") && !name.contains("mark") {
                misnamed.push(name);
            }
        }
        assert!(
            misnamed.is_empty(),
            "these check a designed colour and are not named after the palette: {misnamed:?}"
        );
    }

    /// The two numbers are the ones WCAG 2.5.8 and 2.5.5 name.
    ///
    /// Named for what it checks and no more. Nothing is sized to either
    /// constant, so this says the numbers written down are the right numbers.
    /// It says nothing about any control on the screen, and a name like
    /// "targets meet WCAG" would have claimed exactly that.
    const _THE_TARGET_NUMBERS_ARE_THE_ONES_WCAG_NAMES: () = {
        assert!(MIN_TARGET >= 24);
        assert!(COMFORTABLE_TARGET >= 44);
    };
}

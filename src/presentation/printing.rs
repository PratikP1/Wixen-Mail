//! Drawing laid-out pages on a printer.
//!
//! #45, the tester on 2026-09-15: "Add print functionality." What a page holds
//! and where it breaks is [`crate::application::printing`]'s. This is the
//! device half: the font a page is measured and drawn in, the drawing, and the
//! job that carries the pages to Windows' print spooler.
//!
//! # Why this is Win32 and not wxWidgets
//!
//! wxdragon 0.9.17's printing never starts a print job on Windows. Its C++
//! shim overrides `wxPrintout::OnBeginDocument` and returns without calling
//! the base class, which is where `StartDoc` is called, so the pages are drawn
//! into a device context with no document behind it, nothing is spooled, and
//! the call reports success (phase 13's research, section 1.2). So the calls
//! here are the ones wxWidgets itself would have made.
//!
//! # Why one font measures and draws
//!
//! The layout breaks a line where the font says a run is too wide. A page
//! drawn in any other font would overrun those breaks or fall short of them,
//! so a [`Sheet`] makes the one font and both measures and draws with it, and
//! `tests/printing_draws_what_the_layout_says.rs` reads every line back.
//!
//! Nothing here writes a line of text or a title to the log. A message body is
//! private, and the log goes wherever somebody sends it; what is logged is the
//! step that failed.

#[cfg(target_os = "windows")]
pub use on_windows::{Sheet, draw_page, print};

#[cfg(target_os = "windows")]
mod on_windows {
    use std::path::Path;

    use windows::Win32::Graphics::Gdi::HDC;

    use crate::application::printing::{NotPrinted, Page, Printed};

    /// A device context with the page font chosen into it, ready to measure
    /// and draw.
    pub struct Sheet(());

    impl Sheet {
        /// Chooses the page font into `hdc` and reads how much a page holds.
        ///
        /// `hdc` stays the caller's and must outlive the sheet.
        pub fn on(_hdc: HDC) -> Result<Sheet, NotPrinted> {
            Err(NotPrinted::Failed("nothing is drawn yet".to_string()))
        }

        /// How wide `text` is in the page font, in the device's units.
        pub fn measure(&self, _text: &str) -> u32 {
            0
        }

        /// How wide a line may be, in the same units as [`Self::measure`].
        pub fn width(&self) -> u32 {
            0
        }

        /// How many lines fit under the stamp.
        pub fn lines_per_page(&self) -> usize {
            0
        }
    }

    /// Draws one page: its stamp, a gap, and each of its lines once.
    pub fn draw_page(_sheet: &Sheet, _page: &Page) -> Result<(), NotPrinted> {
        Err(NotPrinted::Failed("nothing is drawn yet".to_string()))
    }

    /// Sends `pages` to the printer behind `sheet` as one job named
    /// `job_name`, into `into_file` when one is given.
    pub fn print(
        _sheet: &Sheet,
        _job_name: &str,
        _into_file: Option<&Path>,
        _pages: &[Page],
    ) -> Result<Printed, NotPrinted> {
        Err(NotPrinted::Failed("nothing is drawn yet".to_string()))
    }
}

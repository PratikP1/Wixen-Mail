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
pub use on_windows::{ChosenPrinter, Sheet, ask_for_a_printer, draw_page, print, print_on};

#[cfg(not(target_os = "windows"))]
pub use elsewhere::{ChosenPrinter, ask_for_a_printer, print_on};

use crate::application::printing::{AfterPrinting, NotPrinted, Paper, after_printing};

/// Windows' print dialog owned by `owner`, then `paper` on the printer chosen
/// there, and the one sentence that says what came of it.
///
/// The one path every surface that prints goes through: the main window, the
/// reader window and the formatted conversation window. Each says the sentence
/// through its own channel and none of them words it, so the surfaces cannot
/// come to print or answer differently.
///
/// On the window's thread, because the dialog is modal to the window, and a
/// message's pages spool in under a second on Microsoft Print to PDF
/// (`tests/printing_spools_a_document.rs`). The log gets the printer's name
/// and the page count, never the title or the text.
pub fn print_through_the_dialog(owner: &wxdragon::prelude::Frame, paper: &Paper) -> AfterPrinting {
    let printed = ask_for_a_printer(owner).and_then(|chosen| match chosen {
        Some(chosen) => print_on(&chosen, &paper.printable, paper.kind)
            .map(|printed| (chosen.printer_name().to_string(), printed)),
        None => Err(NotPrinted::Cancelled),
    });
    if let Ok((printer, printed)) = &printed {
        tracing::info!("Printed {} pages on {printer}", printed.pages);
    }
    after_printing(&paper.printable.title, printed)
}

/// The sentence after Print, said by a window with no status bar of its own:
/// an answer at the ordinary priority and a refusal at the high one, as the
/// main window's status line and refusals are.
pub fn say_what_came_of_it(
    a11y: &crate::presentation::accessibility::Accessibility,
    said: AfterPrinting,
) {
    use crate::presentation::accessibility::announcements::Priority;
    let _ = match said {
        AfterPrinting::Answer(said) => a11y.announce(&said, Priority::Normal),
        AfterPrinting::Refusal(said) => a11y.announce(&said, Priority::High),
    };
}

#[cfg(target_os = "windows")]
mod on_windows {
    use std::ffi::{OsStr, c_void};
    use std::os::windows::ffi::OsStrExt;
    use std::path::{Path, PathBuf};

    use windows::Win32::Foundation::{
        COLORREF, ERROR_CANCELLED, GetLastError, GlobalFree, HGLOBAL, HWND, RECT, SIZE,
    };
    use windows::Win32::Graphics::Gdi::{
        CreateDCW, CreateFontIndirectW, DEFAULT_CHARSET, DT_EXPANDTABS, DT_NOPREFIX, DT_SINGLELINE,
        DeleteDC, DeleteObject, DrawTextW, FW_NORMAL, GetDeviceCaps, GetTextExtentPoint32W,
        GetTextMetricsW, HDC, HFONT, HGDIOBJ, HORZRES, LOGFONTW, LOGPIXELSX, LOGPIXELSY,
        SelectObject, SetBkMode, SetTextColor, TEXTMETRICW, TRANSPARENT, VERTRES,
    };
    use windows::Win32::Storage::Xps::{AbortDoc, DOCINFOW, EndDoc, EndPage, StartDocW, StartPage};
    use windows::Win32::UI::Controls::Dialogs::{
        DEVNAMES, PD_HIDEPRINTTOFILE, PD_NOCURRENTPAGE, PD_NOSELECTION, PD_PAGENUMS,
        PD_RESULT_PRINT, PD_RETURNDC, PD_USEDEVMODECOPIESANDCOLLATE, PRINTDLGEXW, PRINTPAGERANGE,
        PrintDlgExW, START_PAGE_GENERAL,
    };
    use windows::core::PCWSTR;
    use wxdragon::prelude::{Frame, WxWidget};

    use crate::application::printing::{
        Kind, NotPrinted, Page, Printable, Printed, job_name, lay_out, pages_chosen,
    };

    /// The page's size, fixed whatever the screen's reading size: decision 6
    /// of phase 13, taken so there is no setting to find.
    const POINTS: i32 = 11;

    const FACE: &str = "Segoe UI";

    /// Black, whatever the screen's theme, because a dark theme printed wastes
    /// ink and reads badly.
    const BLACK: COLORREF = COLORREF(0);

    /// The rows above a page's own lines: the stamp and a gap under it.
    const ABOVE_THE_TEXT: i32 = 2;

    /// A step that failed, logged by what it was and never by what it held.
    fn failed(step: &str) -> NotPrinted {
        tracing::warn!("Printing stopped: {step}");
        NotPrinted::Failed(step.to_string())
    }

    /// Text as the drawing calls take it: UTF-16, counted rather than ended.
    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    /// Text as the job calls take it: UTF-16 ended by a nought.
    fn ended(text: &OsStr) -> Vec<u16> {
        text.encode_wide().chain(std::iter::once(0)).collect()
    }

    /// The page font, chosen into a device context. Dropped, it puts back the
    /// font it replaced and deletes itself, so an early return leaks nothing.
    struct PageFont {
        hdc: HDC,
        font: HFONT,
        replaced: HGDIOBJ,
    }

    impl PageFont {
        fn chosen_into(hdc: HDC, dots_per_inch: i32) -> Result<PageFont, NotPrinted> {
            let mut face = [0u16; 32];
            face.iter_mut()
                .zip(FACE.encode_utf16())
                .for_each(|(slot, letter)| *slot = letter);
            let design = LOGFONTW {
                // Negative asks for the height of the letters rather than of
                // the cell, which is what a point size means in a word
                // processor. Rounded to the nearest device unit.
                lfHeight: -((POINTS * dots_per_inch + 36) / 72),
                lfWeight: FW_NORMAL.0 as i32,
                lfCharSet: DEFAULT_CHARSET,
                lfFaceName: face,
                ..Default::default()
            };
            // SAFETY: `design` is a whole LOGFONTW that lives across the call.
            let font = unsafe { CreateFontIndirectW(&design) };
            if font.is_invalid() {
                return Err(failed("the page font could not be made"));
            }
            // SAFETY: both handles are live; what SelectObject answers is the
            // font it replaced, which the drop puts back.
            let replaced = unsafe { SelectObject(hdc, font.into()) };
            Ok(PageFont {
                hdc,
                font,
                replaced,
            })
        }

        /// Chooses the font again, for a device context that may have been
        /// handed back with its defaults at the start of a page.
        fn choose_again(&self) {
            // SAFETY: both handles are live for as long as `self` is.
            unsafe { SelectObject(self.hdc, self.font.into()) };
        }
    }

    impl Drop for PageFont {
        fn drop(&mut self) {
            // SAFETY: the font replaced is chosen back first, so the one made
            // here is no longer in use when it is deleted, once, here.
            unsafe {
                SelectObject(self.hdc, self.replaced);
                let _ = DeleteObject(self.font.into());
            }
        }
    }

    /// A device context with the page font chosen into it, ready to measure
    /// and draw.
    pub struct Sheet {
        font: PageFont,
        left: i32,
        top: i32,
        width: i32,
        line_height: i32,
        lines_per_page: usize,
    }

    impl Sheet {
        /// Chooses the page font into `hdc` and reads how much a page holds.
        ///
        /// `hdc` stays the caller's and must outlive the sheet. The page is
        /// what the device can reach, less half an inch on every side.
        pub fn on(hdc: HDC) -> Result<Sheet, NotPrinted> {
            // SAFETY: GetDeviceCaps reads a number from a device context the
            // caller holds open.
            let caps = |index| unsafe { GetDeviceCaps(Some(hdc), index) };
            let (across, down) = (caps(HORZRES), caps(VERTRES));
            let (inch_across, inch_down) = (caps(LOGPIXELSX), caps(LOGPIXELSY));
            if [across, down, inch_across, inch_down]
                .iter()
                .any(|n| *n <= 0)
            {
                return Err(failed("the printer did not say how big its page is"));
            }
            let font = PageFont::chosen_into(hdc, inch_down)?;
            let mut metrics = TEXTMETRICW::default();
            // SAFETY: the page font is chosen into `hdc`, and `metrics` lives
            // across the call.
            if !unsafe { GetTextMetricsW(hdc, &mut metrics) }.as_bool() {
                return Err(failed("the page font could not be measured"));
            }
            let line_height = (metrics.tmHeight + metrics.tmExternalLeading).max(1);
            let (left, top) = (inch_across / 2, inch_down / 2);
            let width = across - 2 * left;
            let rows = (down - 2 * top) / line_height - ABOVE_THE_TEXT;
            if width <= 0 || rows <= 0 {
                return Err(failed("the page is too small to print on"));
            }
            Ok(Sheet {
                font,
                left,
                top,
                width,
                line_height,
                lines_per_page: rows as usize,
            })
        }

        /// How wide `text` is in the page font, in the device's units.
        ///
        /// A run that cannot be measured is as wide as nothing fits, so the
        /// failure shows as short lines rather than as words lost off the
        /// page's edge.
        pub fn measure(&self, text: &str) -> u32 {
            let letters = utf16(text);
            let mut size = SIZE::default();
            // SAFETY: the page font is chosen into the device context, and
            // both buffers live across the call.
            let measured = unsafe { GetTextExtentPoint32W(self.font.hdc, &letters, &mut size) };
            match measured.as_bool() {
                true => size.cx.max(0) as u32,
                false => u32::MAX,
            }
        }

        /// How wide a line may be, in the same units as [`Self::measure`].
        pub fn width(&self) -> u32 {
            self.width as u32
        }

        /// How many lines fit under the stamp.
        pub fn lines_per_page(&self) -> usize {
            self.lines_per_page
        }

        fn hdc(&self) -> HDC {
            self.font.hdc
        }

        /// The font, black text and no background box behind a line, chosen
        /// again for every page.
        fn ready_to_draw(&self) {
            self.font.choose_again();
            // SAFETY: the device context is live for as long as `self` is.
            unsafe {
                SetTextColor(self.hdc(), BLACK);
                SetBkMode(self.hdc(), TRANSPARENT);
            }
        }

        /// One line in row `row` from the top, once. A row with no words on
        /// it draws nothing.
        fn draw_line(&self, row: usize, text: &str) -> Result<(), NotPrinted> {
            if text.trim().is_empty() {
                return Ok(());
            }
            let top = self.top + row as i32 * self.line_height;
            let mut bounds = RECT {
                left: self.left,
                top,
                right: self.left + self.width,
                bottom: top + self.line_height,
            };
            let mut letters = utf16(text);
            // SAFETY: the device context is live, `letters` and `bounds` live
            // across the call, and no flag asks Windows to write into the text.
            // DT_NOPREFIX because an ampersand in a stranger's text is text and
            // not a mnemonic.
            let drawn = unsafe {
                DrawTextW(
                    self.hdc(),
                    &mut letters,
                    &mut bounds,
                    DT_SINGLELINE | DT_NOPREFIX | DT_EXPANDTABS,
                )
            };
            match drawn {
                0 => Err(failed("a line could not be drawn")),
                _ => Ok(()),
            }
        }
    }

    /// Draws one page: its stamp, a gap, and each of its lines once.
    pub fn draw_page(sheet: &Sheet, page: &Page) -> Result<(), NotPrinted> {
        sheet.ready_to_draw();
        std::iter::once(page.stamp.as_str())
            .chain(std::iter::once(""))
            .chain(page.lines.iter().map(String::as_str))
            .enumerate()
            .try_for_each(|(row, text)| sheet.draw_line(row, text))
    }

    /// Sends `pages` to the printer behind `sheet` as one job named
    /// `job_name`, into `into_file` when one is given.
    ///
    /// A job that fails part way is abandoned rather than ended, so half a
    /// message does not come out of the printer looking like the whole of it.
    /// A file printer's Save dialog closed without a name is
    /// [`NotPrinted::Cancelled`], told apart from a failure the way wxWidgets
    /// tells it, by the error `StartDocW` leaves behind.
    pub fn print(
        sheet: &Sheet,
        job_name: &str,
        into_file: Option<&Path>,
        pages: &[Page],
    ) -> Result<Printed, NotPrinted> {
        let name = ended(OsStr::new(job_name));
        let output = into_file.map(|file| ended(file.as_os_str()));
        let job = DOCINFOW {
            cbSize: std::mem::size_of::<DOCINFOW>() as i32,
            lpszDocName: PCWSTR(name.as_ptr()),
            lpszOutput: output
                .as_ref()
                .map_or(PCWSTR::null(), |file| PCWSTR(file.as_ptr())),
            ..Default::default()
        };
        // SAFETY: `job` and the strings it points at live across the call.
        if unsafe { StartDocW(sheet.hdc(), &job) } <= 0 {
            // SAFETY: read straight after the call that set it.
            return Err(match unsafe { GetLastError() } {
                ERROR_CANCELLED => NotPrinted::Cancelled,
                _ => failed("the printer did not accept the job"),
            });
        }
        let sent = pages
            .iter()
            .try_for_each(|page| one_page(sheet, page))
            .and_then(|()| {
                // SAFETY: ends the job StartDocW began on this device context.
                match unsafe { EndDoc(sheet.hdc()) } > 0 {
                    true => Ok(()),
                    false => Err(failed("the printer did not finish the job")),
                }
            });
        match sent {
            Ok(()) => Ok(Printed { pages: pages.len() }),
            Err(why) => {
                // SAFETY: abandons the job StartDocW began, whatever state it
                // reached.
                unsafe { AbortDoc(sheet.hdc()) };
                Err(why)
            }
        }
    }

    /// One page of a job: begun, drawn and ended.
    fn one_page(sheet: &Sheet, page: &Page) -> Result<(), NotPrinted> {
        // SAFETY: begins a page of the job begun on this device context.
        if unsafe { StartPage(sheet.hdc()) } <= 0 {
            return Err(failed("a page could not be started"));
        }
        draw_page(sheet, page)?;
        // SAFETY: ends the page begun above.
        match unsafe { EndPage(sheet.hdc()) } > 0 {
            true => Ok(()),
            false => Err(failed("a page could not be finished")),
        }
    }

    /// A device context made here or handed back by the dialog, deleted once
    /// when dropped.
    struct OwnedDc(HDC);

    impl Drop for OwnedDc {
        fn drop(&mut self) {
            if !self.0.is_invalid() {
                // SAFETY: this value owns the device context and deletes it
                // once, here.
                unsafe {
                    let _ = DeleteDC(self.0);
                }
            }
        }
    }

    // Declared by hand rather than by turning on `Win32_System_Memory` for two
    // calls, the pattern `date_display.rs` uses for a flat call no feature
    // gives. Both are kernel32's and have had these signatures since Windows
    // 3.1.
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GlobalLock(memory: HGLOBAL) -> *mut c_void;
        fn GlobalUnlock(memory: HGLOBAL) -> i32;
    }

    /// Longer than any printer name Windows allows, so a block missing its
    /// ending nought is read no further than this.
    const LONGEST_PRINTER_NAME: usize = 260;

    /// The two blocks of memory the print dialog hands back, freed once when
    /// dropped, on every path out of the dialog.
    struct DialogMemory {
        dev_mode: HGLOBAL,
        dev_names: HGLOBAL,
    }

    impl DialogMemory {
        /// The chosen printer's name, out of the dialog's `DEVNAMES` block.
        fn printer_name(&self) -> Option<String> {
            if self.dev_names.0.is_null() {
                return None;
            }
            // SAFETY: the block is the dialog's `DEVNAMES`, not yet freed, and
            // it is unlocked below before anything returns.
            let base = unsafe { GlobalLock(self.dev_names) } as *const u16;
            if base.is_null() {
                return None;
            }
            // SAFETY: a `DEVNAMES` begins the block, and its device offset
            // counts UTF-16 units from the block's start to a name ended by a
            // nought, read no further than the longest name there can be.
            let name = unsafe {
                let names = std::ptr::read_unaligned(base as *const DEVNAMES);
                let start = base.add(names.wDeviceOffset as usize);
                let length = (0..LONGEST_PRINTER_NAME)
                    .take_while(|&at| *start.add(at) != 0)
                    .count();
                String::from_utf16_lossy(std::slice::from_raw_parts(start, length))
            };
            // SAFETY: balances the lock above.
            unsafe { GlobalUnlock(self.dev_names) };
            Some(name).filter(|name| !name.trim().is_empty())
        }
    }

    impl Drop for DialogMemory {
        fn drop(&mut self) {
            for memory in [self.dev_mode, self.dev_names] {
                if !memory.0.is_null() {
                    // SAFETY: the dialog handed these over to be freed by the
                    // caller, and this frees each once, here.
                    unsafe {
                        let _ = GlobalFree(Some(memory));
                    }
                }
            }
        }
    }

    /// A printer somebody chose, with the pages they asked for.
    pub struct ChosenPrinter {
        dc: OwnedDc,
        printer: String,
        ranges: Vec<(u32, u32)>,
        all: bool,
        into_file: Option<PathBuf>,
    }

    impl ChosenPrinter {
        /// The printer called `name`, every page, into `into_file` when one is
        /// given: the dialog's answer for a caller that already knows which
        /// printer it wants.
        pub fn the_printer_named(
            name: &str,
            into_file: Option<&Path>,
        ) -> Result<ChosenPrinter, NotPrinted> {
            let driver = ended(OsStr::new("WINSPOOL"));
            let device = ended(OsStr::new(name));
            // SAFETY: both strings are ended and live across the call, and no
            // DEVMODE is passed, so the printer's own defaults are used.
            let hdc = unsafe {
                CreateDCW(
                    PCWSTR(driver.as_ptr()),
                    PCWSTR(device.as_ptr()),
                    PCWSTR::null(),
                    None,
                )
            };
            if hdc.is_invalid() {
                return Err(failed("the printer could not be opened"));
            }
            Ok(ChosenPrinter {
                dc: OwnedDc(hdc),
                printer: name.to_string(),
                ranges: Vec::new(),
                all: true,
                into_file: into_file.map(Path::to_path_buf),
            })
        }

        /// The printer's name, as the sentence after printing says it.
        pub fn printer_name(&self) -> &str {
            &self.printer
        }

        /// A sheet on the chosen printer, to measure and draw with.
        pub fn sheet(&self) -> Result<Sheet, NotPrinted> {
            Sheet::on(self.dc.0)
        }
    }

    /// The most page ranges the dialog takes: "1-2, 4, 6-9" is three.
    const MOST_RANGES: usize = 32;

    /// The highest page number the dialog accepts. How many pages there are
    /// is not known until the printer is chosen, because it depends on the
    /// printer, so this is a bound and a range past the end stops at the last
    /// page.
    const HIGHEST_PAGE: u32 = 9999;

    /// Windows' own print dialog, owned by `owner` so focus goes back there
    /// when it closes. `None` when it was closed without printing.
    ///
    /// The printer, the copies and the pages are chosen there, as in every
    /// other program. Copies are the printer driver's to make
    /// (`PD_USEDEVMODECOPIESANDCOLLATE`). Print to File is hidden, because a
    /// file printer such as Microsoft Print to PDF already asks where.
    pub fn ask_for_a_printer(owner: &Frame) -> Result<Option<ChosenPrinter>, NotPrinted> {
        let mut ranges = [PRINTPAGERANGE::default(); MOST_RANGES];
        let mut asking = PRINTDLGEXW {
            lStructSize: std::mem::size_of::<PRINTDLGEXW>() as u32,
            hwndOwner: HWND(owner.get_handle()),
            Flags: PD_RETURNDC
                | PD_NOSELECTION
                | PD_NOCURRENTPAGE
                | PD_USEDEVMODECOPIESANDCOLLATE
                | PD_HIDEPRINTTOFILE,
            nMaxPageRanges: MOST_RANGES as u32,
            lpPageRanges: ranges.as_mut_ptr(),
            nMinPage: 1,
            nMaxPage: HIGHEST_PAGE,
            nCopies: 1,
            nStartPage: START_PAGE_GENERAL,
            ..Default::default()
        };
        // SAFETY: `asking` carries its own size, its page ranges point at
        // `ranges`, which outlives the call, and the owner is a live window on
        // this thread, where wxWidgets has already started OLE.
        let asked = unsafe { PrintDlgExW(&mut asking) };
        let memory = DialogMemory {
            dev_mode: asking.hDevMode,
            dev_names: asking.hDevNames,
        };
        let dc = OwnedDc(asking.hDC);
        if let Err(error) = asked {
            tracing::warn!(
                "Printing stopped: the print dialog answered {:?}",
                error.code()
            );
            return Err(NotPrinted::Failed(
                "Windows' print dialog could not be opened".to_string(),
            ));
        }
        if asking.dwResultAction != PD_RESULT_PRINT {
            return Ok(None);
        }
        if dc.0.is_invalid() {
            return Err(failed("the printer could not be opened"));
        }
        let ranges = ranges[..(asking.nPageRanges as usize).min(MOST_RANGES)]
            .iter()
            .map(|range| (range.nFromPage, range.nToPage))
            .collect();
        Ok(Some(ChosenPrinter {
            dc,
            printer: memory
                .printer_name()
                .unwrap_or_else(|| "the printer you chose".to_string()),
            ranges,
            all: (asking.Flags & PD_PAGENUMS).0 == 0,
            into_file: None,
        }))
    }

    /// `printable` laid out for the chosen printer, and the pages chosen of it
    /// sent as one job named for its `kind`.
    pub fn print_on(
        chosen: &ChosenPrinter,
        printable: &Printable,
        kind: Kind,
    ) -> Result<Printed, NotPrinted> {
        let sheet = chosen.sheet()?;
        let pages = lay_out(
            printable,
            |text| sheet.measure(text),
            sheet.width(),
            sheet.lines_per_page(),
        );
        let wanted: Vec<Page> = pages_chosen(&chosen.ranges, chosen.all, pages.len())
            .into_iter()
            .filter_map(|number| number.checked_sub(1).and_then(|at| pages.get(at)))
            .cloned()
            .collect();
        if wanted.is_empty() {
            return Err(NotPrinted::PastTheLastPage { pages: pages.len() });
        }
        print(&sheet, job_name(kind), chosen.into_file.as_deref(), &wanted)
    }
}

/// Where there is no Win32 there is no printing, and saying so is the one
/// thing to do. The chosen printer cannot exist here, so nothing that takes
/// one can be reached.
#[cfg(not(target_os = "windows"))]
mod elsewhere {
    use crate::application::printing::{Kind, NotPrinted, Printable, Printed};

    /// A printer somebody chose, which off Windows nobody can.
    pub struct ChosenPrinter {
        never: std::convert::Infallible,
    }

    impl ChosenPrinter {
        pub fn printer_name(&self) -> &str {
            match self.never {}
        }
    }

    pub fn ask_for_a_printer(
        _owner: &wxdragon::prelude::Frame,
    ) -> Result<Option<ChosenPrinter>, NotPrinted> {
        Err(NotPrinted::NotOnThisPlatform)
    }

    pub fn print_on(
        chosen: &ChosenPrinter,
        _printable: &Printable,
        _kind: Kind,
    ) -> Result<Printed, NotPrinted> {
        match chosen.never {}
    }
}

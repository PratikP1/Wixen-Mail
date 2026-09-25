//! Every line the layout placed is drawn on its page.
//!
//! `application::printing` decides where a line breaks by asking a font how
//! wide a run of text is, and `presentation::printing` draws the pages in that
//! same font. This reads the drawing back. Each page is drawn into an enhanced
//! metafile, which records every string GDI was asked to draw as a record of
//! its own, and the strings are read out of those records and compared with
//! the page the layout gave: the stamp, then each line, in order, as written.
//!
//! A metafile rather than a printer, because it needs no printer and reads
//! the words: the spool through a real printer driver is
//! `tests/printing_spools_a_document.rs`, which counts pages and cannot read
//! the text of the PDF it makes, and which asserts rather than spools on a
//! machine with no PDF printer. This runs on every machine.
//!
//! What this cannot see: how a printer's own driver renders the page, whether
//! the words are legible on paper, or where the page's edges fall on a sheet.
//! Those are a person's to look at.

#![cfg(windows)]

use windows::Win32::Foundation::LPARAM;
use windows::Win32::Graphics::Gdi::{
    CloseEnhMetaFile, CreateEnhMetaFileW, DeleteEnhMetaFile, EMR_EXTTEXTOUTW, EMRTEXT,
    ENHMETARECORD, ETO_GLYPH_INDEX, EnumEnhMetaFile, HANDLETABLE, HDC, HENHMETAFILE,
};
use windows::core::PCWSTR;
use wixen_mail::application::printing::{Page, Printable, lay_out};
use wixen_mail::presentation::printing::{Sheet, draw_page};

/// A message long enough for several pages, with an ampersand, accents and a
/// line longer than a page is wide.
fn a_message() -> Printable {
    let mut lines = vec![
        "Subject: Café & croissants".to_string(),
        "From: Zoë Ångström <zoe@example.com>".to_string(),
        "To: me@example.com".to_string(),
        "Date: July 24, 2026 at 10:00 AM".to_string(),
        String::new(),
    ];
    lines.extend((1..=120).map(|n| format!("Line {n}: Tom & Jerry had crème brûlée at the café.")));
    lines.push(String::new());
    lines.push("a run of words long enough to wrap ".repeat(60));
    Printable {
        title: "Café & croissants".to_string(),
        lines,
        header_lines: 4,
        warning: None,
    }
}

/// A metafile being recorded. Dropped unfinished, it is closed and deleted.
struct Recording(HDC);

impl Recording {
    fn start() -> Recording {
        // SAFETY: no reference device, no file and no description, so the
        // metafile is kept in memory and measured against the screen.
        let hdc = unsafe { CreateEnhMetaFileW(None, PCWSTR::null(), None, PCWSTR::null()) };
        assert!(
            !hdc.is_invalid(),
            "no metafile could be started, so nothing here can be read"
        );
        Recording(hdc)
    }

    /// The strings drawn into the recording, in the order they were drawn,
    /// leaving out any that are whitespace alone.
    fn strings(self) -> Vec<String> {
        let hdc = self.0;
        std::mem::forget(self);
        // SAFETY: the recording's own device context, closed once, here.
        let played = Played(unsafe { CloseEnhMetaFile(hdc) });
        played.strings()
    }
}

impl Drop for Recording {
    fn drop(&mut self) {
        // SAFETY: the recording's own device context, closed once, here.
        drop(Played(unsafe { CloseEnhMetaFile(self.0) }));
    }
}

/// A finished metafile, deleted when dropped.
struct Played(HENHMETAFILE);

impl Played {
    fn strings(&self) -> Vec<String> {
        let mut drawn: Vec<String> = Vec::new();
        // SAFETY: the metafile is live for the call, and `drawn` outlives it;
        // the callback reads records only inside the call.
        let read = unsafe {
            EnumEnhMetaFile(
                None,
                self.0,
                Some(each_record),
                Some(&mut drawn as *mut Vec<String> as *const std::ffi::c_void),
                None,
            )
        };
        assert!(read.as_bool(), "the metafile could not be read back");
        drawn
            .into_iter()
            .filter(|string| !string.trim().is_empty())
            .collect()
    }
}

impl Drop for Played {
    fn drop(&mut self) {
        // SAFETY: the metafile is this value's, deleted once, here.
        unsafe {
            let _ = DeleteEnhMetaFile(Some(self.0));
        }
    }
}

/// Where an `EMREXTTEXTOUTW`'s `EMRTEXT` begins: after the record's type and
/// size, its bounds, its graphics mode and its two scales, 8 + 16 + 4 + 4 + 4
/// bytes. The `windows` crate has `EMRTEXT` and not the record around it.
const EMRTEXT_AT: usize = 36;

/// Keeps the string of every text record, in order.
unsafe extern "system" fn each_record(
    _hdc: HDC,
    _table: *const HANDLETABLE,
    record: *const ENHMETARECORD,
    _handles: i32,
    data: LPARAM,
) -> i32 {
    // SAFETY: `data` is the `Vec<String>` handed to EnumEnhMetaFile above,
    // and the record is valid for the length of this call.
    unsafe {
        let drawn = &mut *(data.0 as *mut Vec<String>);
        if (*record).iType != EMR_EXTTEXTOUTW {
            return 1;
        }
        let bytes = record as *const u8;
        let text: EMRTEXT = std::ptr::read_unaligned(bytes.add(EMRTEXT_AT) as *const EMRTEXT);
        if text.fOptions & ETO_GLYPH_INDEX.0 != 0 {
            drawn.push("(drawn as glyph numbers, which this cannot read)".to_string());
            return 1;
        }
        let letters: Vec<u16> = (0..text.nChars as usize)
            .map(|at| {
                std::ptr::read_unaligned(bytes.add(text.offString as usize + at * 2) as *const u16)
            })
            .collect();
        drawn.push(String::from_utf16_lossy(&letters));
    }
    1
}

/// The pages the layout gives `printable` when it measures in the page font.
fn laid_out(printable: &Printable) -> Vec<Page> {
    let measuring = Recording::start();
    let sheet = Sheet::on(measuring.0).expect("a sheet on a metafile");
    lay_out(
        printable,
        |text| sheet.measure(text),
        sheet.width(),
        sheet.lines_per_page(),
    )
}

/// What drawing `page` put on paper, read back.
///
/// The sheet is let go before the recording is closed, so the font it chose
/// into the metafile is put back first.
fn drawn(page: &Page) -> Vec<String> {
    let recording = Recording::start();
    {
        let sheet = Sheet::on(recording.0).expect("a sheet on a metafile");
        draw_page(&sheet, page).expect("the page is drawn");
    }
    recording.strings()
}

/// What should have been drawn: the stamp, then every line with words on it.
fn placed(page: &Page) -> Vec<String> {
    std::iter::once(&page.stamp)
        .chain(&page.lines)
        .filter(|line| !line.trim().is_empty())
        .cloned()
        .collect()
}

/// Whether what was drawn is exactly what was placed, said as a complaint
/// naming the first line that differs.
fn drawn_as_placed(placed: &[String], drawn: &[String]) -> Result<(), String> {
    for (at, line) in placed.iter().enumerate() {
        match drawn.get(at) {
            Some(read) if read == line => {}
            Some(read) => {
                return Err(format!(
                    "line {} was placed as {line:?} and drawn as {read:?}",
                    at + 1
                ));
            }
            None => {
                return Err(format!(
                    "line {} was placed as {line:?} and never drawn",
                    at + 1
                ));
            }
        }
    }
    match drawn.len() > placed.len() {
        true => Err(format!(
            "{} strings were drawn that nothing placed, the first {:?}",
            drawn.len() - placed.len(),
            drawn[placed.len()]
        )),
        false => Ok(()),
    }
}

#[test]
fn test_every_line_the_layout_placed_is_drawn_on_its_page() {
    let pages = laid_out(&a_message());
    assert!(
        pages.len() >= 3,
        "the message laid out as {} pages, too few to read a page turn",
        pages.len()
    );

    for (at, page) in pages.iter().enumerate() {
        if let Err(why) = drawn_as_placed(&placed(page), &drawn(page)) {
            panic!("page {} of {}: {why}", at + 1, pages.len());
        }
    }
}

#[test]
fn test_an_ampersand_and_accents_are_drawn_as_written() {
    // An ampersand drawn without DT_NOPREFIX is taken as a mnemonic: it
    // vanishes and the letter after it is underlined, so "Tom & Jerry" comes
    // out as "Tom  Jerry". A stranger's text is text, not markup.
    let pages = laid_out(&a_message());
    let first = drawn(&pages[0]);

    assert!(
        first
            .iter()
            .any(|line| line == "Subject: Café & croissants"),
        "the subject line was not drawn as written: {first:?}"
    );
    assert!(
        first
            .iter()
            .any(|line| line == "Line 1: Tom & Jerry had crème brûlée at the café."),
        "the first line of text was not drawn as written: {first:?}"
    );
}

#[test]
fn test_the_reading_refuses_a_page_with_a_line_dropped() {
    // The companion. A reading that compared nothing, or compared only as
    // far as the shorter list, would pass a page missing its last line.
    let pages = laid_out(&a_message());
    let mut read = drawn(&pages[0]);
    let dropped = read.pop().expect("the first page drew something");

    let complaint = drawn_as_placed(&placed(&pages[0]), &read)
        .expect_err("a line was dropped from the page and the reading did not notice");
    assert!(
        complaint.contains(&format!("{dropped:?}")) && complaint.contains("never drawn"),
        "the reading complained about something other than the dropped line: {complaint}"
    );
}

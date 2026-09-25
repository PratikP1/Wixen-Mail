//! A message printed through a real printer driver comes out as the pages the
//! layout gave.
//!
//! `tests/printing_draws_what_the_layout_says.rs` reads the drawing back from
//! a metafile, which proves every line is drawn and needs no printer. This is
//! the other half: the job itself, `StartDocW` to `EndDoc`, through Windows'
//! print spooler and a real driver, Microsoft Print to PDF, into a file under a
//! temporary folder. Naming the file is what keeps the driver from opening its
//! Save dialog, so nothing here waits on a person and nothing reaches a
//! printer with paper in it.
//!
//! The PDF is read for its page count and nothing else. `pdfpurr` 0.4.0 does
//! not apply the character map Microsoft Print to PDF writes, so its text reads
//! as glyph numbers (phase 13's research, section 1.3); the metafile reading is
//! the one that checks words.
//!
//! GitHub's Windows runners have no Microsoft Print to PDF: it was taken off
//! the Windows Server 2025 image (actions/runner-images#12328). CI says so by
//! setting `WIXEN_NO_PDF_PRINTER`, the way it says there is no sound card with
//! `WIXEN_NO_AUDIO`. With the flag set this does not skip: it asserts the
//! printer really cannot be opened, so the flag cannot hide one that works.
//!
//! What this cannot see: Windows' print dialog, which needs a person, and a
//! page on paper.

#![cfg(windows)]

use std::path::Path;
use std::time::{Duration, Instant};

use wixen_mail::application::printing::{Kind, Printable, Printed, lay_out};
use wixen_mail::presentation::printing::{Chosen, print_on};

const THE_PDF_PRINTER: &str = "Microsoft Print to PDF";

/// How long the spooler is given to finish writing the file after the job
/// ends. Measured in seconds when it works; this is the bound on a failure.
const THE_SPOOLER_IS_GIVEN: Duration = Duration::from_secs(60);

/// A message with a header block and `lines` lines of text under it.
fn a_message_of(lines: usize) -> Printable {
    let mut text = vec![
        "Subject: Quarterly report".to_string(),
        "From: Ada Lovelace <ada@example.com>".to_string(),
        "Date: July 24, 2026 at 10:00 AM".to_string(),
        String::new(),
    ];
    text.extend((1..=lines).map(|n| format!("Line {n} of the report.")));
    Printable {
        title: "Quarterly report".to_string(),
        lines: text,
        header_lines: 3,
        warning: None,
    }
}

/// How many pages the PDF at `file` holds, once the spooler has written it.
fn pages_in(file: &Path) -> usize {
    let started = Instant::now();
    loop {
        let counted = std::fs::read(file)
            .ok()
            .and_then(|bytes| pdfpurr::Document::from_bytes(&bytes).ok())
            .and_then(|document| document.page_count().ok());
        match counted {
            Some(pages) => return pages,
            None if started.elapsed() > THE_SPOOLER_IS_GIVEN => panic!(
                "the spooler did not write a PDF that can be read within {} seconds",
                THE_SPOOLER_IS_GIVEN.as_secs()
            ),
            None => std::thread::sleep(Duration::from_millis(250)),
        }
    }
}

#[test]
fn test_a_message_spooled_to_the_pdf_printer_comes_out_as_the_pages_the_layout_gave() {
    if std::env::var_os("WIXEN_NO_PDF_PRINTER").is_some() {
        assert!(
            Chosen::the_printer_named(THE_PDF_PRINTER, None).is_err(),
            "WIXEN_NO_PDF_PRINTER is set and {THE_PDF_PRINTER} opened, so the flag is \
             hiding a working printer and this reading is not being made"
        );
        return;
    }
    let folder = tempfile::tempdir().expect("a temporary folder");
    let file = folder.path().join("spooled.pdf");
    let chosen = Chosen::the_printer_named(THE_PDF_PRINTER, Some(&file)).unwrap_or_else(|why| {
        panic!(
            "{THE_PDF_PRINTER} could not be opened ({why:?}). Where it is absent, set \
             WIXEN_NO_PDF_PRINTER=1 and this asserts that instead"
        )
    });

    // Two and a half pages of text under the header: three pages, whatever
    // the printer's resolution makes a page hold.
    let per_page = chosen
        .sheet()
        .expect("a sheet on the printer")
        .lines_per_page();
    let message = a_message_of(per_page * 5 / 2);
    let laid_out = {
        let sheet = chosen.sheet().expect("a sheet on the printer");
        lay_out(
            &message,
            |text| sheet.measure(text),
            sheet.width(),
            sheet.lines_per_page(),
        )
    };
    assert_eq!(
        laid_out.len(),
        3,
        "the message did not lay out as three pages"
    );

    let printed = print_on(&chosen, &message, Kind::Message).expect("the job was spooled");

    assert_eq!(printed, Printed { pages: 3 });
    assert_eq!(
        pages_in(&file),
        3,
        "the PDF the driver wrote does not hold the three pages the layout gave"
    );
}

//! Reading a PDF attachment, and saying how much of it is really there.
//!
//! A PDF arrives from a stranger and there is no telling what is in it. It may
//! be a tagged document with a heading structure its author wrote, or it may be
//! a scan with no text at all, or anything between. Those are very different
//! things to be handed, and the difference is invisible unless somebody says
//! it out loud.
//!
//! Most applications do not. They render whatever they can and let the reader
//! work out from the silence that there was nothing there. That is the failure
//! this module exists to avoid: what comes back always says which of the three
//! it got, and where the headings came from.
//!
//! # Where the structure comes from
//!
//! Headings are worked out from the size and position of the text on the page,
//! not from the document's own tags. That is true whether or not the document
//! is tagged, because a PDF's tag tree names its elements without holding their
//! text, so there is no way to place a tagged heading in the extracted text
//! without matching it back anyway.
//!
//! What the tags do decide is what the reader is told. A tagged document's
//! headings agree with the author's structure and can be trusted; an untagged
//! one's are a guess made from typography, and the note at the top says so.
//!
//! # What it will not do
//!
//! It does not render pages, run OCR, or open anything. It turns bytes into
//! text and offsets, which is the whole of what the reading window needs, and
//! it is all pure so it can be tested against real files without a window.

use crate::common::{Error, Result};

/// The most pages to read from one attachment.
///
/// A long PDF is read a page at a time and each page costs real work. Two
/// hundred is more than any attachment a person is sent, and stopping is said
/// out loud rather than leaving somebody to notice the document ends early.
const PAGE_LIMIT: usize = 200;

/// A heading found in a PDF, as an offset into the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfHeading {
    /// Character offset into [`PdfReading::text`].
    pub offset: usize,
    /// One to six.
    pub level: usize,
    pub label: String,
}

/// Where a document's structure came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Structure {
    /// The author tagged it and the tags hold up.
    Tagged,
    /// The author tagged it and the tagging is incomplete.
    TaggedWithGaps,
    /// No tags at all. Everything below is worked out from typography.
    Inferred,
}

/// A PDF, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfReading {
    /// Every page's text, in order, with a line naming each page.
    pub text: String,
    /// Page markers and detected headings, in the order they appear.
    pub headings: Vec<PdfHeading>,
    /// Where the structure came from.
    pub structure: Structure,
    /// What is worth saying about this document before it is read.
    ///
    /// Always something. A reader handed a document with no word about it has
    /// to work out for themselves whether the silence means the document is
    /// fine or means nothing could be read.
    pub note: String,
    /// How many pages the document has.
    pub pages: usize,
    /// How many were read, which is fewer when the document is very long.
    pub pages_read: usize,
}

/// Read a PDF into text and structure.
///
/// Fails only when the bytes are not a PDF this can open at all: encrypted with
/// a password, truncated, or not a PDF. A document that opens and has no text
/// is not a failure, it is a scan, and saying so is more use than an error.
pub fn read(bytes: &[u8]) -> Result<PdfReading> {
    let document = pdfpurr::Document::from_bytes(bytes)
        .map_err(|e| Error::Other(format!("The PDF could not be opened: {e}")))?;

    let pages = document
        .page_count()
        .map_err(|e| Error::Other(format!("The PDF has no readable page list: {e}")))?;
    let pages_read = pages.min(PAGE_LIMIT);

    let structure = structure_of(&document);

    let mut text = String::new();
    let mut headings = Vec::new();
    for page in 0..pages_read {
        let marker = format!("Page {} of {}", page + 1, pages);
        headings.push(PdfHeading {
            offset: text.chars().count(),
            level: 2,
            label: marker.clone(),
        });
        text.push_str(&marker);
        text.push('\n');

        let page_start = text.chars().count();
        let page_text = document.extract_page_text(page).unwrap_or_default();
        // Headings are placed by finding their text in the page's text rather
        // than by rebuilding the page out of the blocks. Detection can miss a
        // block, and a missed block would silently drop that text from the
        // document. This way the worst that happens is a heading nobody can
        // jump to, and the words are all still there.
        headings.extend(headings_in(&document, page, &page_text, page_start));

        text.push_str(page_text.trim_end());
        text.push_str("\n\n");
    }

    Ok(PdfReading {
        note: note_for(structure, pages, pages_read, &text, &headings),
        text,
        headings,
        structure,
        pages,
        pages_read,
    })
}

/// Whether the author tagged this document, and whether the tagging holds up.
fn structure_of(document: &pdfpurr::Document) -> Structure {
    if document.structure_tree().is_none() {
        return Structure::Inferred;
    }
    if document.accessibility_report().is_compliant() {
        Structure::Tagged
    } else {
        Structure::TaggedWithGaps
    }
}

/// The headings on one page, placed in the page's extracted text.
///
/// A forward-only cursor, so a heading whose words appear twice on a page is
/// matched to the one that comes next rather than always to the first.
fn headings_in(
    document: &pdfpurr::Document,
    page: usize,
    page_text: &str,
    page_start: usize,
) -> Vec<PdfHeading> {
    use pdfpurr::content::structure_detection::BlockRole;

    let Ok(blocks) = document.analyze_page_structure(page) else {
        return Vec::new();
    };
    let characters: Vec<char> = page_text.chars().collect();
    let mut cursor = 0usize;
    let mut found = Vec::new();

    for block in blocks {
        let BlockRole::Heading(level) = block.role else {
            continue;
        };
        let label = block
            .runs
            .iter()
            .map(|run| run.text.as_str())
            .collect::<String>();
        let label = label.split_whitespace().collect::<Vec<_>>().join(" ");
        if label.is_empty() {
            continue;
        }
        let Some(at) = find_from(&characters, &label, cursor) else {
            // Extraction and detection disagreed about this block's text,
            // which happens with ligatures and odd encodings. Dropping the
            // landmark loses a jump target; inventing an offset would put the
            // caret somewhere arbitrary, which is worse.
            continue;
        };
        cursor = at + label.chars().count();
        found.push(PdfHeading {
            // Page markers are level 2, so a document's own headings start at
            // 3 and never compete with them. Capped at six the same way the
            // HTML side caps, so the two agree about what a deep heading is.
            offset: page_start + at,
            level: (usize::from(level) + 2).min(6),
            label,
        });
    }
    found
}

/// The character offset of `needle` in `haystack` at or after `from`.
fn find_from(haystack: &[char], needle: &str, from: usize) -> Option<usize> {
    let needle: Vec<char> = needle.chars().collect();
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    (from..=haystack.len() - needle.len()).find(|&start| haystack[start..].starts_with(&needle))
}

/// The sentence read before the document itself.
///
/// Never empty. A reader handed a document with nothing said about it has to
/// work out from the silence whether it is fine or whether nothing could be
/// read, and those are the two things most worth telling apart.
fn note_for(
    structure: Structure,
    pages: usize,
    pages_read: usize,
    text: &str,
    headings: &[PdfHeading],
) -> String {
    let mut parts = Vec::new();

    // The page markers are always there, so anything above that count is a
    // heading that was actually found in the document.
    let real_headings = headings.len().saturating_sub(pages_read);
    // Whether there is any text at all, ignoring the page markers this added.
    let has_text = text
        .lines()
        .filter(|line| !line.starts_with("Page "))
        .any(|line| !line.trim().is_empty());

    if !has_text {
        // The single most useful thing this module says. A scanned PDF is a
        // picture of a document, and every other application presents it as
        // though it were a document.
        parts.push(
            "There is no text in this PDF. It is most likely a scan, which is a \
             picture of a page rather than words, and nothing here can read it \
             aloud. Ask the sender for a copy with real text in it."
                .to_string(),
        );
    } else {
        match structure {
            Structure::Tagged => parts.push(
                "This PDF is tagged, so its headings are the ones its author \
                 marked."
                    .to_string(),
            ),
            Structure::TaggedWithGaps => parts.push(
                "This PDF is tagged, but the tagging is incomplete, so some of \
                 what follows is worked out rather than declared."
                    .to_string(),
            ),
            Structure::Inferred => parts.push(
                "This PDF has no structure of its own. Any headings below were \
                 worked out from the size and position of the text, so they are \
                 a guess rather than the author's."
                    .to_string(),
            ),
        }
        if real_headings == 0 {
            parts.push("No headings were found, so this reads from the top.".to_string());
        }
    }

    if pages_read < pages {
        parts.push(format!(
            "Only the first {pages_read} of {pages} pages are shown. Save the \
             file to read the rest."
        ));
    }

    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The smallest thing that is a PDF, with one line of text on one page.
    ///
    /// Written by hand rather than checked in as a file, so the test says what
    /// it is testing. Offsets in the cross-reference table are wrong on
    /// purpose-free grounds: the parser rebuilds them, which is what it has to
    /// do for the real mail that arrives with them wrong.
    fn one_page_pdf(body: &str) -> Vec<u8> {
        let content = format!("BT /F1 12 Tf 72 700 Td ({body}) Tj ET");
        pdf_from_content(&content)
    }

    /// A one-page PDF built from lines that can each choose their own font
    /// size, so a page can hold a genuine heading next to body text.
    ///
    /// `dx`/`dy` are the `Td` operator's own arguments: the first line's are
    /// an absolute position on the page, since `BT` starts the text matrix at
    /// the origin, and every line after that moves relative to the one
    /// before it, the same way `Td` does in a real content stream.
    fn pdf_with_lines(lines: &[(f64, f64, f64, &str)]) -> Vec<u8> {
        let mut content = String::from("BT\n");
        for (size, dx, dy, text) in lines {
            content.push_str(&format!("/F1 {size} Tf {dx} {dy} Td ({text}) Tj\n"));
        }
        content.push_str("ET");
        pdf_from_content(&content)
    }

    /// Wraps a content stream in the minimal PDF structure the tests need.
    ///
    /// Offsets in the cross-reference table are wrong on purpose: the parser
    /// rebuilds them, which is what it has to do for the real mail that
    /// arrives with them wrong.
    fn pdf_from_content(content: &str) -> Vec<u8> {
        let mut pdf = String::from("%PDF-1.4\n");
        pdf.push_str("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
        pdf.push_str("2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
        pdf.push_str(
            "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] \
             /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>\nendobj\n",
        );
        pdf.push_str(&format!(
            "4 0 obj\n<< /Length {} >>\nstream\n{content}\nendstream\nendobj\n",
            content.len()
        ));
        pdf.push_str("5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n");
        pdf.push_str("trailer\n<< /Size 6 /Root 1 0 R >>\n%%EOF\n");
        pdf.into_bytes()
    }

    #[test]
    fn test_something_that_is_not_a_pdf_is_refused_rather_than_read() {
        // An attachment's type is written by whoever sent the message, so a
        // part labelled application/pdf is a claim about the bytes and not a
        // fact about them.
        let error = read(b"This is not a PDF at all.").expect_err("not a PDF");

        assert!(
            error.to_string().contains("PDF"),
            "unhelpful message: {error}"
        );
    }

    #[test]
    fn test_an_empty_attachment_is_refused_rather_than_read() {
        assert!(read(&[]).is_err());
    }

    #[test]
    fn test_a_pdf_with_text_gives_its_text_back() {
        let reading =
            read(&one_page_pdf("The quarterly numbers are attached.")).expect("a one page PDF");

        assert!(
            reading.text.contains("quarterly numbers"),
            "the text was lost: {:?}",
            reading.text
        );
        assert_eq!(reading.pages, 1);
        assert_eq!(reading.pages_read, 1);
    }

    #[test]
    fn test_every_page_gets_a_marker_to_jump_to() {
        // A page number is the only landmark a plain document has, and moving
        // through a long report a page at a time is the difference between
        // reading it and listening to all of it.
        let reading = read(&one_page_pdf("Hello.")).expect("a one page PDF");

        assert!(reading.text.contains("Page 1 of 1"), "{:?}", reading.text);
        assert!(
            reading.headings.iter().any(|h| h.label == "Page 1 of 1"),
            "{:?}",
            reading.headings
        );
    }

    #[test]
    fn test_an_untagged_pdf_says_its_headings_are_a_guess() {
        // The requirement this module exists for. Presenting inferred
        // structure as though it were the author's is telling somebody their
        // document is accessible when it is not.
        let reading = read(&one_page_pdf("Hello.")).expect("a one page PDF");

        assert_eq!(reading.structure, Structure::Inferred);
        assert!(
            reading.note.contains("no structure of its own"),
            "did not say the structure was inferred: {}",
            reading.note
        );
    }

    #[test]
    fn test_a_landmark_points_at_its_heading_in_the_text() {
        // An offset that is close is an offset that puts the caret in the
        // wrong place, and somebody listening cannot tell that it has.
        let reading = read(&one_page_pdf("Hello.")).expect("a one page PDF");
        let characters: Vec<char> = reading.text.chars().collect();

        for heading in &reading.headings {
            let at: String = characters[heading.offset..]
                .iter()
                .take(heading.label.chars().count())
                .collect();
            assert_eq!(at, heading.label, "landmark {:?} is misplaced", heading);
        }
    }

    #[test]
    fn test_headings_are_in_the_order_they_are_read_in() {
        // The reader moves through them with a key, so out of order means the
        // key goes backwards partway down the document.
        let reading = read(&one_page_pdf("Hello.")).expect("a one page PDF");

        let offsets: Vec<usize> = reading.headings.iter().map(|h| h.offset).collect();
        let mut sorted = offsets.clone();
        sorted.sort_unstable();
        assert_eq!(offsets, sorted, "{:?}", reading.headings);
    }

    #[test]
    fn test_a_heading_level_never_goes_past_six() {
        // The same cap the HTML side uses, so the two agree about what a deep
        // heading is rather than each having their own idea.
        let reading = read(&one_page_pdf("Hello.")).expect("a one page PDF");

        assert!(reading.headings.iter().all(|h| (1..=6).contains(&h.level)));
    }

    #[test]
    fn test_there_is_always_something_to_say_about_the_document() {
        // Silence is the one answer a reader cannot act on: it does not
        // distinguish a document that is fine from one nothing could be read
        // from.
        let reading = read(&one_page_pdf("Hello.")).expect("a one page PDF");

        assert!(!reading.note.trim().is_empty());
    }

    #[test]
    fn test_a_forward_search_finds_the_next_one_not_the_first() {
        // Two headings with the same words on one page have to land on
        // different offsets, or the second sends the caret backwards.
        let text: Vec<char> = "Summary and then Summary again".chars().collect();

        let first = find_from(&text, "Summary", 0).expect("first");
        let second = find_from(&text, "Summary", first + 7).expect("second");

        assert_eq!(first, 0);
        assert_eq!(second, 17);
    }

    #[test]
    fn test_a_heading_that_is_the_whole_page_is_still_found() {
        // A one line document is a real thing to be sent, and a landmark that
        // is the entire text is exactly as useful as any other.
        let text: Vec<char> = "Chapter one".chars().collect();

        assert_eq!(find_from(&text, "Chapter one", 0), Some(0));
    }

    #[test]
    fn test_looking_for_a_heading_that_is_not_there_ends_rather_than_running_on() {
        // The search has to stop at the last place the text could still hold
        // the whole heading. Stopping anywhere later reads past the end, and
        // this runs over text extracted from a stranger's attachment.
        let text: Vec<char> = "Chapter one and then some more".chars().collect();

        assert_eq!(find_from(&text, "absent", 0), None);
        assert_eq!(
            find_from(&text, "Chapter", 5),
            None,
            "the only match is before where the search began"
        );
    }

    #[test]
    fn test_the_note_says_no_headings_only_when_there_are_none() {
        // Every page contributes a marker, so a document with headings of its
        // own has more than one per page. Getting this the wrong way round
        // tells a reader there is nothing to jump to when there is, which is
        // the difference between reading the part they want and reading all of
        // it.
        let marker = PdfHeading {
            offset: 0,
            level: 2,
            label: "Page 1".to_string(),
        };
        let real = PdfHeading {
            offset: 7,
            level: 3,
            label: "Introduction".to_string(),
        };
        let text = "Page 1\nSome words\n";

        let with_one = note_for(Structure::Inferred, 1, 1, text, &[marker.clone(), real]);
        assert!(
            !with_one.contains("No headings were found"),
            "a document with a heading was said to have none: {with_one}"
        );

        let with_none = note_for(Structure::Inferred, 1, 1, text, &[marker]);
        assert!(
            with_none.contains("No headings were found"),
            "a document with no headings did not say so: {with_none}"
        );
    }

    #[test]
    fn test_a_document_only_partly_read_says_how_much_is_missing() {
        // Silence here reads as "that was the whole thing", and somebody acts
        // on a document having seen a fifth of it.
        let text = "Page 1\nSome words\n";

        let whole = note_for(Structure::Tagged, 3, 3, text, &[]);
        assert!(
            !whole.contains("Only the first"),
            "a document read in full claimed to be cut short: {whole}"
        );

        let part = note_for(Structure::Tagged, 10, 3, text, &[]);
        assert!(
            part.contains("Only the first 3 of 10 pages"),
            "a document cut short did not say so: {part}"
        );
    }

    #[test]
    fn test_a_needle_longer_than_the_text_is_not_found_rather_than_a_panic() {
        let text: Vec<char> = "short".chars().collect();

        assert_eq!(find_from(&text, "much longer than that", 0), None);
        assert_eq!(find_from(&text, "", 0), None);
    }

    #[test]
    fn test_a_pdf_with_no_text_says_it_is_probably_a_scan() {
        // The most useful sentence in the module. Every other application
        // presents a scan as though it were a document, and the reader finds
        // out by hearing nothing.
        let reading = read(&one_page_pdf("")).expect("a one page PDF");

        assert!(
            reading.note.contains("no text in this PDF"),
            "did not say it was empty: {}",
            reading.note
        );
        assert!(reading.note.contains("scan"), "{}", reading.note);
    }

    #[test]
    fn test_a_real_heading_is_found_and_placed_correctly_in_the_text() {
        // one_page_pdf only ever writes one font size, so a document's own
        // heading never exists in it: nothing distinguishes a heading line
        // from a paragraph without at least two sizes on the page. Every
        // other test in this file reads only page markers through
        // `headings_in`; this is the one that reads a heading the document
        // actually wrote.
        let reading = read(&pdf_with_lines(&[
            (
                12.0,
                72.0,
                760.0,
                "A short line sits before the heading on this page.",
            ),
            (20.0, 0.0, -30.0, "Quarterly Results"),
            (
                12.0,
                0.0,
                -30.0,
                "The numbers below cover the third quarter in detail.",
            ),
        ]))
        .expect("a page with a real heading");

        let heading = reading
            .headings
            .iter()
            .find(|h| h.label == "Quarterly Results")
            .unwrap_or_else(|| panic!("the document's own heading was not found: {:?}", reading));
        // Page markers are level 2, so a document's own heading starts at 3.
        assert_eq!(heading.level, 3, "{:?}", reading.headings);

        // The offset has to point at the label itself, not somewhere near it.
        let characters: Vec<char> = reading.text.chars().collect();
        let at: String = characters[heading.offset..]
            .iter()
            .take(heading.label.chars().count())
            .collect();
        assert_eq!(
            at, heading.label,
            "landmark is misplaced: {:?}",
            reading.headings
        );
    }

    #[test]
    fn test_the_same_heading_twice_on_a_page_is_matched_to_each_occurrence_in_turn() {
        // headings_in keeps a forward-only cursor precisely so the second
        // "Overview" is not matched back to the first one's offset. Nothing
        // before this exercised that cursor: every other test's page has at
        // most one real heading.
        let reading = read(&pdf_with_lines(&[
            (
                12.0,
                72.0,
                760.0,
                "A short line sits before the first heading.",
            ),
            (20.0, 0.0, -30.0, "Overview"),
            (
                12.0,
                0.0,
                -30.0,
                "The first section covers what changed this quarter.",
            ),
            (
                12.0,
                0.0,
                -20.0,
                "It continues for a second line of the same paragraph.",
            ),
            (20.0, 0.0, -30.0, "Overview"),
            (
                12.0,
                0.0,
                -20.0,
                "The second section covers what is planned next.",
            ),
        ]))
        .expect("a page with a repeated heading");

        let overviews: Vec<&PdfHeading> = reading
            .headings
            .iter()
            .filter(|h| h.label == "Overview")
            .collect();
        assert_eq!(overviews.len(), 2, "{:?}", reading.headings);
        assert!(
            overviews[0].offset < overviews[1].offset,
            "the second Overview was matched back to the first: {:?}",
            reading.headings
        );

        let characters: Vec<char> = reading.text.chars().collect();
        for heading in &overviews {
            let at: String = characters[heading.offset..]
                .iter()
                .take(heading.label.chars().count())
                .collect();
            assert_eq!(
                at, heading.label,
                "landmark is misplaced: {:?}",
                reading.headings
            );
        }
    }
}

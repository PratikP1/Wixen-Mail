//! What a printed page holds, and where it breaks.
//!
//! #45, the tester on 2026-09-15: "Add print functionality." The issue asks
//! for the header lines and the text, and calls plain text with header lines
//! the honest first version. This is the layout half of that: a message, a
//! conversation or an item in, pages out, decided here where it can be tested
//! without a window or a printer. Drawing the pages through Windows' print
//! dialog is `presentation`'s half.
//!
//! # Why paper reuses the reader's composition
//!
//! The reader window already decides what a message's header lines are and
//! in what order, and what its text is once the markup is gone. A second
//! header builder for paper would be one fact written twice, and the two would
//! drift. So a page is the reader's document laid out on paper, and the one
//! thing paper needs differently is asked for through the reading it is
//! composed with: [`on_paper`] answers the same reading with every date in
//! full, because a printed "2 days ago" is wrong the day after.
//!
//! The five other kinds work the same way. Each item's reading says its fields
//! in an order, and [`from_item`] prints those same fields one to a line, so
//! what is spoken and what is printed name the same things.
//!
//! # Why the layout takes a measuring function
//!
//! Where a line breaks depends on the font, and the font is the printer's
//! business: its resolution, its face, its size. The words are this module's.
//! So [`lay_out`] is handed a function that says how wide a run of text is and
//! how wide a line may be, and decides everything else. A test hands it a fixed
//! width per letter, and every break is arithmetic.

use crate::presentation::read_aloud::Reading;
use crate::presentation::reader_text::ReaderDocument;

/// What is being printed, for the name the print job carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Message,
    Conversation,
    Event,
    Contact,
    Task,
    Note,
    Reminder,
}

impl Kind {
    /// Every kind, so a test can ask all of them the same question.
    pub const ALL: [Kind; 7] = [
        Kind::Message,
        Kind::Conversation,
        Kind::Event,
        Kind::Contact,
        Kind::Task,
        Kind::Note,
        Kind::Reminder,
    ];
}

/// A thing ready to be laid out: its title and its lines, as plain text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Printable {
    /// What the stamp on every page calls it.
    pub title: String,
    /// The text, one entry per line as written, before any wrapping.
    pub lines: Vec<String>,
    /// How many of `lines`, from the top, are the header block, which is
    /// kept on one page.
    pub header_lines: usize,
    /// What is wrong with the message, printed above it when something is.
    pub warning: Option<String>,
}

/// One printed page: the line stamped at its top and the lines under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub stamp: String,
    pub lines: Vec<String>,
}

/// The same reading, with every date written in full.
pub fn on_paper(out: Reading) -> Reading {
    out
}

/// A document the reader composed, as a thing to print.
pub fn from_document(document: &ReaderDocument) -> Printable {
    Printable {
        title: document.title.clone(),
        lines: Vec::new(),
        header_lines: 0,
        warning: None,
    }
}

/// The pages a printable fills.
pub fn lay_out(
    _printable: &Printable,
    _measure: impl Fn(&str) -> u32,
    _width: u32,
    _lines_per_page: usize,
) -> Vec<Page> {
    Vec::new()
}

/// The name the print job carries in Windows' print queue.
pub fn job_name(_kind: Kind) -> &'static str {
    ""
}

/// What is said once the pages are with the printer.
pub fn sent_to_the_printer(_title: &str, _printer: &str, _pages: usize) -> String {
    String::new()
}

/// What is said when the print dialog was closed without printing.
pub fn nothing_was_printed() -> String {
    String::new()
}

/// What is said when the pages could not be printed.
pub fn printing_failed(_why: &str) -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::status_sentences::{Voice, reads_as_a_persons_sentence};
    use crate::common::types::MessageBody;
    use crate::presentation::date_display::{
        Clock, DateOrder, DateSettings, DateStyle, DateWording,
    };
    use crate::presentation::reader_text::{ConversationPart, conversation, single_message};
    use crate::presentation::ui_types::{AttachmentItem, MessageItem};

    /// Every letter ten units wide, so a line of `width` 100 holds ten.
    fn ten_units_a_letter(text: &str) -> u32 {
        text.chars().count() as u32 * 10
    }

    const TEN_LETTERS: u32 = 100;

    /// The reading the list ships with: "2 days ago" within the week.
    fn reading() -> Reading {
        use chrono::TimeZone;
        Reading {
            dates: DateSettings {
                style: DateStyle::RelativeWithinWeek,
                order: DateOrder::MonthFirst,
                wording: DateWording::Verbal,
                clock: Clock::TwelveHour,
            },
            now: chrono::Local
                .with_ymd_and_hms(2026, 7, 26, 12, 0, 0)
                .single()
                .expect("a real moment"),
        }
    }

    fn message() -> MessageItem {
        MessageItem {
            message_id: 7,
            subject: "Quarterly report".to_string(),
            from: "Ada Lovelace <ada@example.com>".to_string(),
            to: "me@example.com".to_string(),
            cc: "grace@example.com".to_string(),
            date: "2026-07-24 10:00".to_string(),
            has_attachments: true,
            attachments: vec![AttachmentItem {
                filename: "numbers.xlsx".to_string(),
                mime_type: "application/octet-stream".to_string(),
                size: 1024,
                description: crate::service::mime::WhatTheSenderSaid::Nothing,
            }],
            ..MessageItem::default()
        }
    }

    fn body(text: &str) -> MessageBody {
        MessageBody::Plain(text.to_string())
    }

    fn printable(title: &str, lines: &[&str], header_lines: usize) -> Printable {
        Printable {
            title: title.to_string(),
            lines: lines.iter().map(|line| line.to_string()).collect(),
            header_lines,
            warning: None,
        }
    }

    fn every_line(pages: &[Page]) -> Vec<String> {
        pages.iter().flat_map(|page| page.lines.clone()).collect()
    }

    #[test]
    fn test_a_message_prints_its_header_lines_before_its_text() {
        let document = single_message(
            &message(),
            &body("The numbers are attached."),
            on_paper(reading()),
        );
        let printable = from_document(&document);

        let pages = lay_out(&printable, ten_units_a_letter, 1000, 60);

        assert_eq!(printable.header_lines, 6, "{printable:#?}");
        assert_eq!(pages.len(), 1, "{pages:#?}");
        assert_eq!(
            pages[0].lines,
            [
                "Subject: Quarterly report",
                "From: Ada Lovelace <ada@example.com>",
                "To: me@example.com",
                "Cc: grace@example.com",
                "Date: July 24, 2026 at 10:00 AM",
                "Attachments: numbers.xlsx",
                "",
                "The numbers are attached.",
            ]
        );
    }

    #[test]
    fn test_the_date_on_paper_is_the_full_date_and_time() {
        // The control first: the reading the list ships with says how long
        // ago, so the case below is about the reading paper asks for.
        let on_screen = single_message(&message(), &body("Hello."), reading());
        assert!(on_screen.text.contains("ago"), "{}", on_screen.text);

        let printed = single_message(&message(), &body("Hello."), on_paper(reading()));

        let date = printed
            .text
            .lines()
            .find(|line| line.starts_with("Date: "))
            .expect("a date line");
        assert_eq!(date, "Date: July 24, 2026 at 10:00 AM");
        assert!(!printed.text.contains("ago"), "{}", printed.text);
    }

    #[test]
    fn test_a_line_longer_than_the_page_breaks_at_the_last_space_that_fits() {
        let pages = lay_out(
            &printable("Words", &["the quick brown fox jumps"], 0),
            ten_units_a_letter,
            TEN_LETTERS,
            60,
        );

        assert_eq!(every_line(&pages), ["the quick", "brown fox", "jumps"]);
    }

    #[test]
    fn test_a_run_with_no_space_breaks_at_the_last_character_that_fits() {
        // Longer than two lines, so the run is broken more than once and the
        // break is not a special case of the first line.
        let pages = lay_out(
            &printable("Run", &["abcdefghijklmnopqrstuvwxyz"], 0),
            ten_units_a_letter,
            TEN_LETTERS,
            60,
        );

        assert_eq!(every_line(&pages), ["abcdefghij", "klmnopqrst", "uvwxyz"]);
    }

    #[test]
    fn test_the_header_block_moves_whole_to_the_next_page() {
        let mut two_lines_of_warning = printable(
            "Moved",
            &["Subject: Moved", "From: Ada", "Date: today", "", "Body."],
            3,
        );
        two_lines_of_warning.warning = Some("Be careful\nwith this one".to_string());

        let pages = lay_out(&two_lines_of_warning, ten_units_a_letter, 1000, 4);

        assert_eq!(pages.len(), 2, "{pages:#?}");
        assert_eq!(pages[0].lines, ["Be careful", "with this one"]);
        assert_eq!(
            pages[1].lines,
            ["Subject: Moved", "From: Ada", "Date: today", "Body."],
            "the header block goes whole to the next page, and the empty \
             line at the top of what is left is dropped"
        );
    }

    #[test]
    fn test_no_page_holds_only_its_stamp() {
        let pages = lay_out(
            &printable("Blank tail", &["one", "two", "", "", ""], 0),
            ten_units_a_letter,
            TEN_LETTERS,
            2,
        );

        assert_eq!(pages.len(), 1, "{pages:#?}");
        assert!(
            pages.iter().all(|page| !page.lines.is_empty()),
            "{pages:#?}"
        );
    }

    #[test]
    fn test_the_last_page_says_page_n_of_n() {
        let pages = lay_out(
            &printable("Five lines", &["1", "2", "3", "4", "5"], 0),
            ten_units_a_letter,
            TEN_LETTERS,
            2,
        );

        let stamps: Vec<&str> = pages.iter().map(|page| page.stamp.as_str()).collect();
        assert_eq!(
            stamps,
            [
                "Wixen Mail, Five lines, page 1 of 3",
                "Wixen Mail, Five lines, page 2 of 3",
                "Wixen Mail, Five lines, page 3 of 3",
            ]
        );
    }

    #[test]
    fn test_a_warning_prints_above_the_message() {
        let mut suspicious = message();
        suspicious.safety = crate::service::safety::Safety::Phishing;
        suspicious.safety_reasons = vec!["The sender's name and address do not match".to_string()];
        let document = single_message(&suspicious, &body("Click here."), on_paper(reading()));
        let warning = document
            .warning
            .clone()
            .expect("a phishing message is warned about");

        let lines = every_line(&lay_out(
            &from_document(&document),
            ten_units_a_letter,
            100_000,
            60,
        ));

        let warned: Vec<&str> = warning.lines().collect();
        assert_eq!(lines[..warned.len()], warned[..], "{lines:#?}");
        assert_eq!(lines[warned.len()], "", "{lines:#?}");
        assert_eq!(
            lines[warned.len() + 1],
            "Subject: Quarterly report",
            "{lines:#?}"
        );
    }

    #[test]
    fn test_a_conversation_prints_every_message_in_order() {
        let parts: Vec<ConversationPart> = ["First words.", "Second words.", "Third words."]
            .iter()
            .enumerate()
            .map(|(at, words)| {
                let mut said_by = message();
                said_by.from = format!("Person {at} <p{at}@example.com>");
                ConversationPart {
                    message: said_by,
                    body: body(words),
                    said: crate::application::reading_a_message::WhatIsSaidAboutIt::nothing(),
                    depth: at,
                }
            })
            .collect();
        let printable = from_document(&conversation("Plans", &parts));

        let lines = every_line(&lay_out(&printable, ten_units_a_letter, 100_000, 60));

        assert_eq!(printable.header_lines, 2, "{printable:#?}");
        assert_eq!(lines[0], "Plans");
        let at = |words: &str| {
            lines
                .iter()
                .position(|line| line == words)
                .unwrap_or_else(|| panic!("{words} is printed: {lines:#?}"))
        };
        assert!(at("First words.") < at("Second words."), "{lines:#?}");
        assert!(at("Second words.") < at("Third words."), "{lines:#?}");
    }

    #[test]
    fn test_the_print_job_is_named_by_kind_and_never_by_subject() {
        // `job_name` takes a kind and nothing else, so no subject can reach a
        // shared printer's queue, which is a room other people are in.
        let names: Vec<&str> = Kind::ALL.into_iter().map(job_name).collect();

        assert_eq!(
            names,
            [
                "Wixen Mail message",
                "Wixen Mail conversation",
                "Wixen Mail event",
                "Wixen Mail contact",
                "Wixen Mail task",
                "Wixen Mail note",
                "Wixen Mail reminder",
            ]
        );
    }

    #[test]
    fn test_every_printing_sentence_reads_as_a_persons_sentence() {
        let said = [
            sent_to_the_printer("Quarterly report", "HP LaserJet 1022", 2),
            sent_to_the_printer("Quarterly report", "HP LaserJet 1022", 1),
            nothing_was_printed(),
            printing_failed("the printer did not answer"),
        ];

        assert_eq!(
            said,
            [
                "Sent Quarterly report to HP LaserJet 1022, 2 pages.",
                "Sent Quarterly report to HP LaserJet 1022, 1 page.",
                "Printing was cancelled, so nothing was printed.",
                "Nothing was printed, because the printer did not answer. Check that the \
                 printer is on and connected, then print again.",
            ]
        );
        for sentence in &said {
            if let Err(why) = reads_as_a_persons_sentence(sentence, Voice::Answer) {
                panic!("{why}");
            }
        }
    }

    #[test]
    fn test_no_stamp_holds_a_dash_character() {
        let title = "Q3 report \u{2014} draft - final";
        let pages = lay_out(
            &printable(title, &[title, "second line"], 0),
            ten_units_a_letter,
            100_000,
            1,
        );

        assert_eq!(pages.len(), 2, "{pages:#?}");
        for page in &pages {
            assert!(
                !page
                    .stamp
                    .contains(['\u{2012}', '\u{2013}', '\u{2014}', '\u{2015}'])
                    && !page.stamp.contains(" - "),
                "{:?}",
                page.stamp
            );
        }
        assert_eq!(
            pages[0].stamp,
            "Wixen Mail, Q3 report, draft, final, page 1 of 2"
        );
        assert_eq!(
            pages[0].lines,
            [title],
            "the sender's own dashes are printed as written"
        );
    }
}

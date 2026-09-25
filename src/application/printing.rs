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

use crate::presentation::date_display::{DateSettings, DateStyle};
use crate::presentation::read_aloud::{Field, Reading};
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

    /// What a page says it is when the thing on it has no title, so the
    /// first line and the stamp are never blank. A message says what the
    /// reader window says for one.
    fn untitled(self) -> &'static str {
        match self {
            Kind::Message | Kind::Conversation => "No subject",
            Kind::Event => "Event with no title",
            Kind::Contact => "Contact with no name",
            Kind::Task => "Task with no title",
            Kind::Note => "Note with no title",
            Kind::Reminder => "Reminder with no title",
        }
    }
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
///
/// Whatever the reader chose for the screen: a list that says "2 days ago"
/// is right while it is on the screen and wrong on paper the day after.
pub fn on_paper(out: Reading) -> Reading {
    Reading {
        dates: DateSettings {
            style: DateStyle::Absolute,
            ..out.dates
        },
        ..out
    }
}

/// A document the reader composed, as a thing to print.
///
/// The header block is every line above the first empty one, which is where
/// the reader's composition puts the gap between the header lines and the
/// text. With no empty line there is no telling where it ends, so nothing is
/// kept together.
pub fn from_document(document: &ReaderDocument) -> Printable {
    let lines: Vec<String> = document.text.lines().map(as_text).collect();
    let header_lines = lines
        .iter()
        .position(|line| line.trim().is_empty())
        .unwrap_or(0);
    Printable {
        title: document.title.clone(),
        lines,
        header_lines,
        warning: document
            .warning
            .as_deref()
            .map(|warning| warning.lines().map(as_text).collect::<Vec<_>>().join("\n")),
    }
}

/// An item's fields, one to a line, as a thing to print.
///
/// The fields are the ones the item's full reading says, in its order, so
/// paper and speech name the same things. The title is the first line and the
/// short fields follow it, labelled, as the header block; the long text comes
/// last, after a gap, as it was written rather than as it is spoken. An empty
/// field prints no line, the way it says no word.
pub fn from_item(kind: Kind, title: &str, fields: &[Field]) -> Printable {
    let title = match title.trim() {
        "" => kind.untitled().to_string(),
        named => named.to_string(),
    };
    let said: Vec<&Field> = fields
        .iter()
        .filter(|field| !field.value.trim().is_empty())
        .collect();
    // Every reading opens with the item's name, unlabelled, because a row is
    // heard by its name first. On paper that name is the title line, and
    // printing it again under itself would be a stutter.
    let said = match said.first() {
        Some(name) if name.label.is_empty() && name.value.trim() == title => &said[1..],
        _ => &said[..],
    };
    let (long, short): (Vec<&Field>, Vec<&Field>) = said.iter().partition(|field| field.long);

    let mut lines = vec![title.clone()];
    for field in short {
        lines.extend(labelled(field).lines().map(as_text));
    }
    let header_lines = lines.len();
    for field in long {
        lines.push(String::new());
        if !field.label.is_empty() {
            lines.push(format!("{}:", field.label));
        }
        lines.extend(
            field
                .value
                .trim_end()
                .trim_start_matches(['\r', '\n'])
                .lines()
                .map(as_text),
        );
    }
    Printable {
        title,
        lines,
        header_lines,
        warning: None,
    }
}

/// A short field as its line: "Label: value", or the value alone.
fn labelled(field: &Field) -> String {
    match field.label.trim() {
        "" => field.value.trim().to_string(),
        label => format!("{label}: {}", field.value.trim()),
    }
}

/// How many columns a tab reaches to.
const TAB_STOP: usize = 4;

/// One line as characters a page can hold: tabs as spaces, and no control
/// characters, which a printer would draw as boxes or act on.
///
/// Nothing else is interpreted. A message body is a stranger's text, and on
/// paper it is text and nothing more.
fn as_text(line: &str) -> String {
    let mut text = String::with_capacity(line.len());
    let mut column = 0;
    for letter in line.chars() {
        match letter {
            '\t' => {
                let spaces = TAB_STOP - column % TAB_STOP;
                text.push_str(&" ".repeat(spaces));
                column += spaces;
            }
            letter if letter.is_control() => {}
            letter => {
                text.push(letter);
                column += 1;
            }
        }
    }
    text
}

/// The pages a printable fills.
///
/// `measure` says how wide a run of text is, `width` how wide a line may be,
/// in the same units, and `lines_per_page` how many lines fit under the stamp.
/// The warning comes first, then the header block, kept on one page unless it
/// is longer than a page, then the rest. A page never starts with an empty
/// line and never holds nothing but its stamp.
pub fn lay_out(
    printable: &Printable,
    measure: impl Fn(&str) -> u32,
    width: u32,
    lines_per_page: usize,
) -> Vec<Page> {
    let wrapped = |line: &str| wrap(line, &measure, width);
    let mut pages = Filling::new(lines_per_page);
    if let Some(warning) = &printable.warning {
        warning
            .lines()
            .for_each(|line| pages.place_all(wrapped(line)));
        pages.place(String::new());
    }
    let (header, text) = printable
        .lines
        .split_at(printable.header_lines.min(printable.lines.len()));
    pages.keep_together(header.iter().flat_map(|line| wrapped(line)).collect());
    text.iter().for_each(|line| pages.place_all(wrapped(line)));
    pages.stamped(&printable.title)
}

/// One line broken into the lines a page is wide enough for.
///
/// At the last space that fits, so words stay whole; where no space fits, at
/// the last character that does, so a long address or a run of symbols still
/// reaches the page. Every line holds at least one character, so a page too
/// narrow for a letter still moves on rather than stopping.
fn wrap(line: &str, measure: &impl Fn(&str) -> u32, width: u32) -> Vec<String> {
    let mut wrapped = Vec::new();
    let mut rest = line.trim_end();
    loop {
        let fits = longest_start_that_fits(rest, measure, width);
        if fits == rest.len() {
            wrapped.push(rest.to_string());
            return wrapped;
        }
        // The character that did not fit counts as a place to break when it
        // is a space: "brown fox" fits and the space after it is where it ends.
        let reach = fits + rest[fits..].chars().next().map_or(0, char::len_utf8);
        let last_space_that_fits = rest[..reach].rfind(' ');
        let (taken, left) = match last_space_that_fits {
            Some(space) if !rest[..space].trim().is_empty() => {
                (rest[..space].trim_end(), rest[space..].trim_start())
            }
            _ => (&rest[..fits], &rest[fits..]),
        };
        wrapped.push(taken.to_string());
        rest = left;
    }
}

/// How many bytes from the start of `text` fit in `width`, and never less
/// than one character.
///
/// Measured a character at a time, so the work is the width of a line and not
/// the length of the text: a run of thousands of characters is measured a line
/// at a time.
fn longest_start_that_fits(text: &str, measure: &impl Fn(&str) -> u32, width: u32) -> usize {
    let mut fits = 0;
    for (at, letter) in text.char_indices() {
        let end = at + letter.len_utf8();
        if measure(&text[..end]) > width {
            break;
        }
        fits = end;
    }
    match fits {
        0 => text.chars().next().map_or(0, char::len_utf8),
        fits => fits,
    }
}

/// Pages being filled, a line at a time.
struct Filling {
    room: usize,
    filled: Vec<Vec<String>>,
    page: Vec<String>,
}

impl Filling {
    fn new(lines_per_page: usize) -> Self {
        Filling {
            room: lines_per_page.max(1),
            filled: Vec::new(),
            page: Vec::new(),
        }
    }

    fn place(&mut self, line: String) {
        if self.page.len() == self.room {
            self.turn();
        }
        if self.page.is_empty() && line.trim().is_empty() {
            return;
        }
        self.page.push(line);
    }

    fn place_all(&mut self, lines: Vec<String>) {
        lines.into_iter().for_each(|line| self.place(line));
    }

    /// Lines that go on one page when a page can hold them all.
    fn keep_together(&mut self, block: Vec<String>) {
        let left = self.room - self.page.len();
        if block.len() > left && block.len() <= self.room {
            self.turn();
        }
        self.place_all(block);
    }

    /// Finish the page being filled, without the empty lines at its foot.
    fn turn(&mut self) {
        let mut page = std::mem::take(&mut self.page);
        while page.last().is_some_and(|line| line.trim().is_empty()) {
            page.pop();
        }
        if !page.is_empty() {
            self.filled.push(page);
        }
    }

    fn stamped(mut self, title: &str) -> Vec<Page> {
        self.turn();
        let pages = self.filled.len();
        self.filled
            .into_iter()
            .enumerate()
            .map(|(at, lines)| Page {
                stamp: stamp(title, at + 1, pages),
                lines,
            })
            .collect()
    }
}

/// The line at the top of every page.
///
/// Commas and no dash, the house style, and the title's own dashes turned into
/// commas too, because the stamp is this program's sentence even where the
/// title is somebody else's. The text under it keeps every dash it was
/// written with.
fn stamp(title: &str, page: usize, pages: usize) -> String {
    match without_dashes(title) {
        title if title.is_empty() => format!("Wixen Mail, page {page} of {pages}"),
        title => format!("Wixen Mail, {title}, page {page} of {pages}"),
    }
}

/// The dash characters, from the figure dash to the horizontal bar.
const DASHES: [char; 4] = ['\u{2012}', '\u{2013}', '\u{2014}', '\u{2015}'];

/// A title with each dash, and each hyphen standing alone between words
/// where a dash was meant, turned into a comma. A hyphen inside a word stays.
fn without_dashes(title: &str) -> String {
    let spaced: String = title
        .chars()
        .map(|letter| match DASHES.contains(&letter) {
            true => " - ".to_string(),
            false => letter.to_string(),
        })
        .collect();
    let words: Vec<&str> = spaced.split_whitespace().collect();
    words
        .split(|word| word.chars().all(|letter| letter == '-'))
        .filter(|piece| !piece.is_empty())
        .map(|piece| piece.join(" "))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The name the print job carries in Windows' print queue.
///
/// By kind and never by subject. The queue of a shared printer is read by
/// whoever else is using it, and a subject is private; this takes a kind and
/// nothing else, so no subject can reach it.
pub fn job_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Message => "Wixen Mail message",
        Kind::Conversation => "Wixen Mail conversation",
        Kind::Event => "Wixen Mail event",
        Kind::Contact => "Wixen Mail contact",
        Kind::Task => "Wixen Mail task",
        Kind::Note => "Wixen Mail note",
        Kind::Reminder => "Wixen Mail reminder",
    }
}

/// What is said once the pages are with the printer: once, and not a word
/// per page, so a long message is not a long announcement.
pub fn sent_to_the_printer(title: &str, printer: &str, pages: usize) -> String {
    format!(
        "Sent {} to {}, {}.",
        title.trim(),
        printer.trim(),
        crate::service::caldav::how_many(pages, "page")
    )
}

/// What is said when the print dialog was closed without printing.
///
/// Said rather than left to silence, because a dialog closing without a word
/// cannot be told apart from a print that failed.
pub fn nothing_was_printed() -> String {
    "Printing was cancelled, so nothing was printed.".to_string()
}

/// What is said when the pages could not be printed: what went wrong, and
/// what to try.
pub fn printing_failed(why: &str) -> String {
    format!(
        "Nothing was printed, because {}. Check that the printer is on and connected, \
         then print again.",
        why.trim().trim_end_matches('.')
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::status_sentences::{Voice, reads_as_a_persons_sentence};
    use crate::common::types::MessageBody;
    use crate::presentation::date_display::{
        Clock, DateOrder, DateSettings, DateStyle, DateWording,
    };
    use crate::presentation::read_aloud::ReadAloud;
    use crate::presentation::reader_text::{ConversationPart, conversation, single_message};
    use crate::presentation::ui_types::{
        AttachmentItem, CalendarEventItem, ContactItem, MessageItem, NoteItem, ReminderItem,
        TaskItem,
    };

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

        // Four lines a page: the warning and the gap under it take three, so
        // the header's three go to the second page rather than one of them
        // staying behind, and the gap under the header is the fourth line of
        // that page, which is its foot and is not printed.
        assert_eq!(pages.len(), 3, "{pages:#?}");
        assert_eq!(pages[0].lines, ["Be careful", "with this one"]);
        assert_eq!(
            pages[1].lines,
            ["Subject: Moved", "From: Ada", "Date: today"]
        );
        assert_eq!(pages[2].lines, ["Body."]);
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

    // ── The other five kinds, printed from the fields their reading says ──

    fn event() -> CalendarEventItem {
        CalendarEventItem {
            attendees_json: None,
            id: "e1".to_string(),
            summary: "Standup".to_string(),
            description: "Agenda:\n1. Numbers\n2. Plans".to_string(),
            start: "2026-07-27 09:00".to_string(),
            end: "2026-07-27 09:15".to_string(),
            location: "Room 4".to_string(),
            is_all_day: false,
            status: "confirmed".to_string(),
            provider: "local".to_string(),
            calendar_id: None,
            calendar_name: Some("Work".to_string()),
            calendar_color: None,
            reminder_minutes: None,
            repeats: String::new(),
            categories: String::new(),
            show_as: String::new(),
            recurrence_rule: None,
            changed_on_its_own: false,
        }
    }

    fn contact() -> ContactItem {
        ContactItem {
            id: "c1".to_string(),
            name: "Grace Hopper".to_string(),
            email: "grace@example.com".to_string(),
            phone: "555 0100".to_string(),
            phone_label: "Mobile".to_string(),
            company: "Navy".to_string(),
            address: "1 Main St".to_string(),
            address_label: "Home".to_string(),
            birthday: "--03-14".to_string(),
            favorite: true,
            notes: "Met at a conference.".to_string(),
        }
    }

    fn task() -> TaskItem {
        TaskItem {
            id: "t1".to_string(),
            title: "File the report".to_string(),
            description: None,
            due_date: Some("2026-07-24 17:00".to_string()),
            is_completed: false,
            priority: "high".to_string(),
            task_list_id: None,
            parent_task_id: None,
        }
    }

    fn note() -> NoteItem {
        NoteItem {
            id: "n1".to_string(),
            title: "Shopping".to_string(),
            body: "# Plans\n\n- Milk\n- Bread".to_string(),
            body_preview: "Plans".to_string(),
            pinned: true,
            updated_at: "2026-07-20 08:00".to_string(),
            folder_id: None,
        }
    }

    fn reminder() -> ReminderItem {
        ReminderItem {
            id: "r1".to_string(),
            title: "Call the dentist".to_string(),
            description: None,
            due_datetime: Some("2026-07-20 09:30".to_string()),
            is_completed: true,
            priority: "normal".to_string(),
        }
    }

    #[test]
    fn test_an_event_prints_its_start_end_place_and_description_one_to_a_line() {
        let event = event();

        let printed = from_item(
            Kind::Event,
            &event.summary,
            &event.fields(on_paper(reading())),
        );

        assert_eq!(
            printed.lines,
            [
                "Standup",
                "July 27, 2026 at 9:00 AM to July 27, 2026 at 9:15 AM",
                "Location: Room 4",
                "Calendar: Work",
                "",
                "Agenda:",
                "1. Numbers",
                "2. Plans",
            ]
        );
        assert_eq!(printed.header_lines, 4, "{printed:#?}");
        // One list of fields for both: the place printed is the place said.
        let said = event.read_full(on_paper(reading()));
        assert!(
            said.split(". ").any(|part| part == "Location: Room 4"),
            "{said}"
        );
    }

    #[test]
    fn test_a_contact_prints_every_field_its_reading_names() {
        let contact = contact();

        let printed = from_item(
            Kind::Contact,
            &contact.name,
            &contact.fields(on_paper(reading())),
        );

        assert_eq!(
            printed.lines,
            [
                "Grace Hopper",
                "Email: grace@example.com",
                "Mobile: 555 0100",
                "Company: Navy",
                "Home: 1 Main St",
                "Birthday: March 14",
                "Favorite",
                "",
                "Notes:",
                "Met at a conference.",
            ],
            "the name once, as the title, and every field the reading says after it"
        );
    }

    #[test]
    fn test_a_task_prints_its_due_date_in_full() {
        let task = task();
        // The control: the list's reading says how long ago it was due.
        let on_screen = from_item(Kind::Task, &task.title, &task.fields(reading()));
        assert!(
            on_screen.lines.iter().any(|line| line.contains("ago")),
            "{on_screen:#?}"
        );

        let printed = from_item(Kind::Task, &task.title, &task.fields(on_paper(reading())));

        assert_eq!(
            printed.lines,
            [
                "File the report",
                "Not done",
                "Priority: high",
                "Due: July 24, 2026 at 5:00 PM",
            ]
        );
    }

    #[test]
    fn test_a_note_prints_its_body_as_written_not_as_spoken() {
        let note = note();

        let printed = from_item(Kind::Note, &note.title, &note.fields(on_paper(reading())));

        assert_eq!(
            printed.lines,
            [
                "Shopping",
                "Pinned",
                "Updated: July 20, 2026 at 8:00 AM",
                "",
                "# Plans",
                "",
                "- Milk",
                "- Bread",
            ]
        );
        // The control: the same body spoken says its structure in words, so
        // the lines above are the text as written and not the reading.
        let said = note.read_full(on_paper(reading()));
        assert!(said.contains("heading level 1, Plans"), "{said}");
    }

    #[test]
    fn test_a_reminder_prints_whether_it_is_done() {
        let mut reminder = reminder();
        let done = from_item(
            Kind::Reminder,
            &reminder.title,
            &reminder.fields(on_paper(reading())),
        );
        reminder.is_completed = false;
        let not_done = from_item(
            Kind::Reminder,
            &reminder.title,
            &reminder.fields(on_paper(reading())),
        );

        assert_eq!(
            done.lines,
            ["Call the dentist", "Done", "Due: July 20, 2026 at 9:30 AM"]
        );
        assert_eq!(not_done.lines[1], "Not done", "{not_done:#?}");
    }

    #[test]
    fn test_an_empty_field_prints_no_line() {
        let mut contact = contact();
        contact.phone.clear();
        contact.company.clear();
        contact.address.clear();
        contact.birthday.clear();
        contact.favorite = false;
        contact.notes.clear();

        let printed = from_item(
            Kind::Contact,
            &contact.name,
            &contact.fields(on_paper(reading())),
        );

        assert_eq!(printed.lines, ["Grace Hopper", "Email: grace@example.com"]);
        assert_eq!(printed.header_lines, 2, "{printed:#?}");

        // An empty title is an empty field too, and the page still says what
        // it is rather than starting blank.
        let titles: Vec<String> = Kind::ALL
            .into_iter()
            .map(|kind| from_item(kind, " ", &[]).title)
            .collect();
        assert_eq!(
            titles,
            [
                "No subject",
                "No subject",
                "Event with no title",
                "Contact with no name",
                "Task with no title",
                "Note with no title",
                "Reminder with no title",
            ]
        );
    }
}

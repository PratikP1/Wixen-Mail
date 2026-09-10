//! A note as the document a calendar server exchanges, read and written here.
//!
//! Pure. It turns a note into a document and a document into a note and
//! touches no network, which is this project's thin-transport rule and is what
//! makes the parsing testable without a server:
//!
//! ```text
//! grep -rn "reqwest\|Outward\|http" src/service/note_document.rs
//! ```
//!
//! returns nothing, and that is an acceptance criterion of the plan that wrote
//! this file rather than a habit.
//!
//! # It shares the calendar's routines rather than writing its own
//!
//! Folding a long line, putting a folded one back together, reading a
//! property's value off a line and escaping a value are all done by
//! [`crate::service::caldav`]'s own routines. A journal entry is the same
//! format as an event, so a second set of them here would be a second answer
//! to a question that already has one. `days_called_off`'s comment records what
//! happened the one time this project had two: a cancellation went back to a
//! server with its own property name written in front of it twice.
//!
//! # A round trip cannot be byte-identical, and this is where it stops
//!
//! Measured rather than assumed, and it is the first thing this implementation
//! found that the seam's contract had not thought about.
//!
//! The format has one escape for a line break and no way to write a carriage
//! return inside a value at all: RFC 5545 section 3.3.11 gives `\n`, and a
//! literal carriage return would end the content line. So a body typed on a
//! Windows machine, holding CRLF, goes out as `\n` and comes back as LF. Every
//! other byte survives: a trailing space, a backslash, a comma, an emoji, and a
//! line inside the body that looks like a property name.
//!
//! That is a property of the format rather than of this code, and no backend
//! that speaks it can do better. It matters because the phase README says this
//! backend "is the only candidate whose round trip can be byte-identical", and
//! that is now known to be false for one specific case.

use crate::service::caldav::{
    as_one_value, as_typed, ical_utc_stamp, property_name, unfolded, value_named_on, written_out,
};

/// One note, as a journal document carries it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ANoteInADocument {
    /// What the document calls itself. Opaque, and never built here.
    pub uid: String,
    /// What the note is called.
    pub title: String,
    /// What it says.
    pub body: String,
}

/// Why a document could not be read as a note.
///
/// Named rather than a string, for the reason
/// [`crate::application::notes_backend::WhatTheBackendSaid`] gives about its
/// own variants: what reaches somebody is read aloud, and a reason a parser
/// wrote for a developer is not a reason to speak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotANote {
    /// Nothing in it is a journal entry. A calendar document holding only
    /// events is this, and so is a truncated one whose `END` never arrived.
    NoJournalEntryInIt,
    /// A journal entry with no identifier on it.
    ///
    /// Refused rather than given one here. The identifier is the backend's own
    /// word for the note and this program never builds one, which is the first
    /// requirement the seam's contract puts on any backend.
    NoIdentifierOnIt,
}

/// The component a note lives in.
const A_JOURNAL_ENTRY: &str = "VJOURNAL";

/// What this program writes as the producer of a document.
const WRITTEN_BY: &str = "PRODID:-//Wixen Mail//NONSGML v1.0//EN";

/// The note one document holds, or why it holds none.
///
/// Reads the first journal entry and only the first. A document holding more
/// than one is not something either candidate backend produces, and taking the
/// first is the same answer `parse_ical_vevent` gives for the same shape.
pub fn the_note_in(document: &str) -> Result<ANoteInADocument, NotANote> {
    let lines = unfolded(document);
    let inside = the_lines_inside(&lines, A_JOURNAL_ENTRY).ok_or(NotANote::NoJournalEntryInIt)?;

    let uid = inside
        .iter()
        .find_map(|line| value_named_on(line, "UID"))
        .ok_or(NotANote::NoIdentifierOnIt)?
        .to_string();

    Ok(ANoteInADocument {
        uid,
        // A journal entry with no summary is a note nobody named, which is
        // ordinary: this program's own note editor allows an empty title, and
        // so does every other program that writes one.
        title: inside
            .iter()
            .find_map(|line| the_whole_value_on(line, "SUMMARY"))
            .map(as_typed)
            .unwrap_or_default(),
        body: inside
            .iter()
            .find_map(|line| the_whole_value_on(line, "DESCRIPTION"))
            .map(as_typed)
            .unwrap_or_default(),
    })
}

/// What one line carries for a property, exactly as it was written.
///
/// [`value_named_on`] is the calendar's reader and it trims. That is right for
/// a zone name or a status word, where surrounding space is noise, and it is
/// wrong for a note: a body is somebody's text and a trailing space is part of
/// it. Measured rather than assumed, by writing a body ending in a space and
/// watching it come back one byte shorter.
///
/// The format is on this side of the argument. RFC 5545 section 3.1 says a
/// property's value begins immediately after the delimiter colon, so there is
/// no leading space to strip that was not in the value, and the same section
/// puts no restriction on the space inside one.
///
/// An empty value is `Some("")` rather than `None`, which is the other place
/// the calendar's reader answers a different question: for it, a property with
/// nothing after the colon is a property that says nothing, and for a note it
/// is a note with nothing in it.
fn the_whole_value_on<'a>(line: &'a str, property: &str) -> Option<&'a str> {
    if !property_name(line).is_some_and(|name| name.eq_ignore_ascii_case(property)) {
        return None;
    }
    Some(&line[crate::service::caldav::delimiter_colon(line)? + 1..])
}

/// A whole document for a note no backend has ever held.
pub fn a_document_saying(uid: &str, title: &str, body: &str) -> String {
    written_out(&[
        "BEGIN:VCALENDAR".to_string(),
        "VERSION:2.0".to_string(),
        WRITTEN_BY.to_string(),
        format!("BEGIN:{A_JOURNAL_ENTRY}"),
        format!("UID:{uid}"),
        format!("SUMMARY:{}", as_one_value(title)),
        format!("DESCRIPTION:{}", as_one_value(body)),
        format!("DTSTAMP:{}", ical_utc_stamp(chrono::Utc::now())),
        format!("END:{A_JOURNAL_ENTRY}"),
        "END:VCALENDAR".to_string(),
    ])
}

/// The document the server already holds, saying something else.
///
/// Only the two properties this program owns are replaced, and everything else
/// the server had is left exactly where it was. A whole document written from
/// nothing would be the shorter code and would throw away every property this
/// program has never modelled: categories somebody set in another program, the
/// class, the status, an attachment.
///
/// `build_ical_vevent` and `ical_with_the_event_changed` are the same pair for
/// an event and were split for the same reason, which their own comments give:
/// one list of what this program owns, written twice, would be two answers, and
/// the second change to somebody's calendar would disagree with the first.
///
/// A document with no journal entry in it is left alone. Writing a note into
/// something that is not one would put a second component into a stranger's
/// document.
pub fn the_document_with_the_note_changed(document: &str, title: &str, body: &str) -> String {
    let lines = unfolded(document);
    if the_lines_inside(&lines, A_JOURNAL_ENTRY).is_none() {
        return document.to_string();
    }
    let mut written: Vec<String> = Vec::with_capacity(lines.len() + 2);
    let mut inside = false;
    let mut said = false;
    for line in &lines {
        if opens(line, A_JOURNAL_ENTRY) {
            inside = true;
            written.push(line.clone());
            continue;
        }
        if closes(line, A_JOURNAL_ENTRY) {
            if !said {
                written.push(format!("SUMMARY:{}", as_one_value(title)));
                written.push(format!("DESCRIPTION:{}", as_one_value(body)));
                said = true;
            }
            inside = false;
            written.push(line.clone());
            continue;
        }
        // The two this program owns come out wherever the server put them, and
        // go back in together at the end. Written in place one at a time, a
        // document that carried a summary and no description would grow the
        // description somewhere the summary is not, which is legal and is
        // harder to read back.
        if inside && is_one_of_ours(line) {
            continue;
        }
        written.push(line.clone());
    }
    written_out(&written)
}

/// Whether this line carries a property this program decides the value of.
fn is_one_of_ours(line: &str) -> bool {
    property_name(line).is_some_and(|name| {
        name.eq_ignore_ascii_case("SUMMARY") || name.eq_ignore_ascii_case("DESCRIPTION")
    })
}

/// Whether this line opens that component, however it is written.
///
/// Matched without regard to case, which the calendar standard requires of
/// component and property names. A reader comparing exact strings reads a
/// server that writes `Begin:VJournal` as having sent no journal entry at all,
/// and the sender never hears about it.
fn opens(line: &str, component: &str) -> bool {
    value_named_on(line, "BEGIN").is_some_and(|named| named.eq_ignore_ascii_case(component))
}

/// Whether this line closes that component, however it is written.
fn closes(line: &str, component: &str) -> bool {
    value_named_on(line, "END").is_some_and(|named| named.eq_ignore_ascii_case(component))
}

/// The lines between that component's `BEGIN` and its `END`.
///
/// `None` where it never begins, and `None` where it begins and never ends,
/// which is what a document cut short in the middle looks like. Reading a
/// truncated document as a note whose body simply stops is the half-filled
/// answer this refuses to give.
fn the_lines_inside<'a>(lines: &'a [String], component: &str) -> Option<Vec<&'a str>> {
    let from = lines.iter().position(|line| opens(line, component))?;
    let to = lines
        .iter()
        .skip(from)
        .position(|line| closes(line, component))?
        + from;
    Some(lines[from + 1..to].iter().map(String::as_str).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A note written here and read back is the same note.
    ///
    /// Both halves of this are written in this repository, so what it proves is
    /// that the reader and the writer agree with each other. It is not evidence
    /// that a real server's document parses, and saying so here rather than in
    /// a summary is the point: `04-09` names the same limit for constructed DER
    /// and it is the same limit.
    #[test]
    fn test_a_note_written_as_a_document_comes_back_the_bytes_it_went_in_as() {
        let fixtures = [
            (
                "a trailing space, which a trim added while tidying would take",
                "Remember the space at the end ",
            ),
            (
                "a comma and a semicolon, which the format marks and a reader \
                 has to unmark exactly once",
                "Milk, eggs; and bread",
            ),
            (
                "a backslash, which grows on every save if the marking is not \
                 undone on the way in",
                "C:\\Users\\notes and the words after it",
            ),
            (
                "a character outside the basic multilingual plane, which a \
                 byte-length assumption cuts in half when a line is folded",
                "Dinner at seven and a walk after, all of it worth remembering \
                 because the line has to be long enough to fold before it can be \
                 cut in the wrong place",
            ),
            (
                "a line that looks like a property name, which a note is free \
                 text and somebody really types",
                "BEGIN:VEVENT\nis what the file started with\nEND:VEVENT",
            ),
            ("nothing at all", ""),
        ];

        for (aimed_at, body) in fixtures {
            let document = a_document_saying("uid-1", "A note", body);
            let read_back = the_note_in(&document).expect("the note just written");

            assert_eq!(
                read_back.body, body,
                "a body with {aimed_at} did not come back as it went in"
            );
            assert_eq!(read_back.title, "A note");
            assert_eq!(read_back.uid, "uid-1");
        }
    }

    /// Windows line endings do not survive, and this says so rather than
    /// leaving somebody to find it.
    ///
    /// The format has one escape for a line break, `\n`, and no way to write a
    /// carriage return inside a value at all: a literal one ends the content
    /// line. So CRLF goes out as one break and comes back as LF. No backend
    /// speaking this format can do better, and the phase README's claim that
    /// this is the candidate whose round trip can be byte-identical is wrong
    /// for exactly this case.
    #[test]
    fn test_windows_line_endings_come_back_as_plain_ones_and_nothing_else_changes() {
        let document = a_document_saying("uid-1", "A note", "one\r\ntwo\r\nthree");
        let read_back = the_note_in(&document).expect("the note");

        assert_eq!(
            read_back.body, "one\ntwo\nthree",
            "the line endings changed in some way this does not describe"
        );
    }

    /// Property names mean the same however they are written.
    ///
    /// The case here is one this build never writes, which is what makes the
    /// fixture worth having: a reader comparing exact strings passes against
    /// every document this program produced and drops every property from a
    /// server that writes them differently.
    #[test]
    fn test_a_document_written_in_a_case_this_build_does_not_use_is_read() {
        let document = "Begin:VCalendar\r\n\
                        Version:2.0\r\n\
                        Begin:VJournal\r\n\
                        Uid:uid-1\r\n\
                        Summary:Wiring colours\r\n\
                        Description:Brown is live\r\n\
                        End:VJournal\r\n\
                        End:VCalendar\r\n";

        let read_back = the_note_in(document).expect("a note written in another case");

        assert_eq!(read_back.uid, "uid-1");
        assert_eq!(read_back.title, "Wiring colours");
        assert_eq!(read_back.body, "Brown is live");
    }

    /// A property this build does not know does not cost the ones it does.
    #[test]
    fn test_a_property_this_build_does_not_know_is_not_a_reason_to_refuse_a_note() {
        let document = "BEGIN:VCALENDAR\r\n\
                        VERSION:2.0\r\n\
                        BEGIN:VJOURNAL\r\n\
                        UID:uid-1\r\n\
                        SOMETHING-LATER:whatever it means\r\n\
                        SUMMARY:Wiring colours\r\n\
                        CATEGORIES:House,Wiring\r\n\
                        DESCRIPTION:Brown is live\r\n\
                        END:VJOURNAL\r\n\
                        END:VCALENDAR\r\n";

        let read_back =
            the_note_in(document).expect("a note with more on it than this build knows");

        assert_eq!(read_back.title, "Wiring colours");
        assert_eq!(read_back.body, "Brown is live");
    }

    /// A document that cannot be read is refused with a reason.
    #[test]
    fn test_a_document_that_is_not_a_note_is_refused_rather_than_half_read() {
        // Cut short in the middle, which is what a connection dropping leaves.
        // The identifier and the title are both there, so a reader that took
        // what it could find would hand back a note whose body simply stops.
        let cut_short = "BEGIN:VCALENDAR\r\n\
                         BEGIN:VJOURNAL\r\n\
                         UID:uid-1\r\n\
                         SUMMARY:Wiring colours\r\n\
                         DESCRIPTION:Brown is";
        assert_eq!(the_note_in(cut_short), Err(NotANote::NoJournalEntryInIt));

        // A calendar with an event in it and no journal entry at all.
        let an_event = "BEGIN:VCALENDAR\r\n\
                        BEGIN:VEVENT\r\n\
                        UID:uid-1\r\n\
                        SUMMARY:Lunch\r\n\
                        END:VEVENT\r\n\
                        END:VCALENDAR\r\n";
        assert_eq!(the_note_in(an_event), Err(NotANote::NoJournalEntryInIt));

        // A journal entry with nothing naming it. The name is the backend's own
        // word for the note and nothing here builds one.
        let unnamed = "BEGIN:VCALENDAR\r\n\
                       BEGIN:VJOURNAL\r\n\
                       SUMMARY:Wiring colours\r\n\
                       END:VJOURNAL\r\n\
                       END:VCALENDAR\r\n";
        assert_eq!(the_note_in(unnamed), Err(NotANote::NoIdentifierOnIt));

        assert_eq!(the_note_in(""), Err(NotANote::NoJournalEntryInIt));
    }

    /// Changing a note leaves everything else in the server's document alone.
    #[test]
    fn test_changing_a_note_keeps_what_this_program_does_not_own() {
        let theirs = "BEGIN:VCALENDAR\r\n\
                      VERSION:2.0\r\n\
                      PRODID:-//Somebody Else//Their Program//EN\r\n\
                      BEGIN:VJOURNAL\r\n\
                      UID:uid-1\r\n\
                      SUMMARY:Old title\r\n\
                      CATEGORIES:House,Wiring\r\n\
                      CLASS:PRIVATE\r\n\
                      DESCRIPTION:Old words\r\n\
                      END:VJOURNAL\r\n\
                      END:VCALENDAR\r\n";

        let changed = the_document_with_the_note_changed(theirs, "New title", "New words");

        let read_back = the_note_in(&changed).expect("the changed note");
        assert_eq!(read_back.title, "New title");
        assert_eq!(read_back.body, "New words");
        assert_eq!(read_back.uid, "uid-1", "the identifier was rewritten");
        assert!(
            changed.contains("CATEGORIES:House,Wiring"),
            "a property this program has never modelled was thrown away: {changed}"
        );
        assert!(changed.contains("CLASS:PRIVATE"), "{changed}");
        assert!(
            changed.contains("PRODID:-//Somebody Else//Their Program//EN"),
            "the document was rewritten from nothing rather than changed: {changed}"
        );
        assert_eq!(
            changed.matches("SUMMARY:").count(),
            1,
            "the old title was left beside the new one: {changed}"
        );
    }

    /// A document with no journal entry in it is not given one.
    #[test]
    fn test_changing_a_note_in_a_document_that_holds_none_leaves_it_alone() {
        let an_event = "BEGIN:VCALENDAR\r\n\
                        BEGIN:VEVENT\r\n\
                        UID:uid-1\r\n\
                        SUMMARY:Lunch\r\n\
                        END:VEVENT\r\n\
                        END:VCALENDAR\r\n";

        assert_eq!(
            the_document_with_the_note_changed(an_event, "New title", "New words"),
            an_event,
            "a note was written into a document that is not a note"
        );
    }

    /// A summary the server wrote in another case is replaced, not doubled.
    ///
    /// The failure this is about really happened here, to the calendar writer:
    /// it read `SUMMARY ` as a different name from `SUMMARY`, left the server's
    /// own title in the document and wrote the new one beside it, and two titles
    /// went to somebody's calendar.
    #[test]
    fn test_a_property_the_server_wrote_in_another_case_is_replaced_rather_than_doubled() {
        let theirs = "BEGIN:VCALENDAR\r\n\
                      BEGIN:VJOURNAL\r\n\
                      UID:uid-1\r\n\
                      Summary:Old title\r\n\
                      Description:Old words\r\n\
                      END:VJOURNAL\r\n\
                      END:VCALENDAR\r\n";

        let changed = the_document_with_the_note_changed(theirs, "New title", "New words");

        assert!(
            !changed.contains("Old title"),
            "the server's own title was kept beside the new one: {changed}"
        );
        assert!(!changed.contains("Old words"), "{changed}");
        assert_eq!(
            the_note_in(&changed).expect("the changed note").title,
            "New title"
        );
    }

    /// A body long enough to be folded comes back whole.
    #[test]
    fn test_a_body_long_enough_to_be_broken_across_lines_is_put_back_together() {
        let long = "words ".repeat(60);
        let document = a_document_saying("uid-1", "A note", &long);

        assert!(
            document.contains("\r\n "),
            "the document was never folded, so this proves nothing about \
             putting one back together"
        );
        assert_eq!(the_note_in(&document).expect("the note").body, long);
    }
}

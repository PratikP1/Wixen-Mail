//! Reading a text attachment, and saying how much of it is really there.
//!
//! A text attachment arrives from a stranger with a label its sender chose, and
//! the label is the only claim anybody has made about it. It may be the prose
//! it says it is; it may be a log written in an encoding nobody records any
//! more; it may be a compiled program somebody called `readme.txt`. Those are
//! very different things to be handed, and the difference is invisible in a
//! reading window unless something says it out loud.
//!
//! So what comes back always carries a note saying which of them it got, and
//! the note goes above the words rather than below them, because it changes how
//! the words should be taken. `pdf.rs` answers the same question the same way
//! for the same reason, and this is written as its sibling.
//!
//! # What it will not do
//!
//! It does not guess at an encoding. Anything that is not UTF-8 is shown with
//! the parts that would not decode replaced, and the note says so, rather than
//! this trying several encodings and presenting whichever produced the fewest
//! odd characters. A guess dressed as a reading is worse than a reading that
//! admits what it could not do.
//!
//! It renders nothing and opens nothing. Bytes in, characters out, which is the
//! whole of what a reading window needs, and it is all pure so it can be tested
//! against real files without one.

use crate::common::{Error, Result};

/// The most of one text attachment that is read into the reading window.
///
/// Not the attachment store's own limit, which is twenty-five megabytes: that
/// one answers whether a file is worth keeping on disk, and this one answers
/// how much of it a person can be handed at once. Twenty-five megabytes of
/// characters in a read-only control is a window that stops answering while it
/// is filled and a screen reader with a document it cannot move around in. A
/// megabyte is a long novel, which is already more than any attachment is, and
/// stopping is said out loud rather than leaving somebody to notice the file
/// ends early. `pdf.rs` bounds the same risk the same way with `PAGE_LIMIT`.
pub const LONGEST_READ_BYTES: usize = 1024 * 1024;

/// How much of the start of a file decides whether it is text at all.
///
/// The question is what kind of file this is, and the first few thousand bytes
/// answer that as well as all of them do. Reading twenty-five megabytes to
/// decide whether to read twenty-five megabytes is a pass over the file for
/// nothing.
const SAMPLE_BYTES: usize = 8 * 1024;

/// One character in this many may be something other than writing before the
/// file stops being worth rendering as text.
///
/// A judgement rather than a measurement, and it is the whole difference
/// between a useful preview and a screenful of nonsense, so the reasoning is
/// worth having in the open. Real text has almost none of these: a stray form
/// feed in a printout, an escape sequence in a captured terminal session. A
/// file that is a tenth control characters and replacement characters is a file
/// whose text is incidental, and `wx_reader.rs` already gives the reason for
/// refusing rather than rendering one: opening a spreadsheet in a text control
/// is worse than saying it cannot be read.
///
/// Erring towards showing it, deliberately. Something shown that should not
/// have been is a screenful somebody closes; something refused that should have
/// been shown is a file they cannot read at all and no way to find out why.
const ONE_CHARACTER_IN: usize = 10;

/// A text file, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextReading {
    /// The words, with anything that would confuse a reading window taken out.
    pub text: String,
    /// What is worth saying about this file before it is read.
    ///
    /// Always something. A reader handed a file with no word about it has to
    /// work out for themselves whether the silence means the file is fine.
    pub note: String,
    /// How many bytes arrived.
    pub bytes: usize,
    /// How many were read, which is fewer when the file is very long.
    pub bytes_read: usize,
    /// Whether every byte decoded, or some had to be replaced.
    pub entirely_text: bool,
}

/// Read bytes into text.
///
/// Fails only when the bytes are not text at all, which is a file a sender
/// labelled wrongly or labelled honestly and this cannot show. That refusal is
/// the point rather than an inconvenience: the alternative is a reading window
/// full of nonsense that a screen reader will read out character by character.
pub fn read(bytes: &[u8]) -> Result<TextReading> {
    if mostly_not_text(&bytes[..bytes.len().min(SAMPLE_BYTES)]) {
        return Err(Error::Other(
            "it is not text, whatever it is labelled as".to_string(),
        ));
    }

    let kept = up_to_a_character_boundary(bytes, LONGEST_READ_BYTES);
    let decoded = String::from_utf8_lossy(kept);
    // `from_utf8_lossy` borrows when it changed nothing and owns when it had to
    // replace something, so the answer is already here rather than needing a
    // second pass counting replacement characters.
    let entirely_text = matches!(decoded, std::borrow::Cow::Borrowed(_));

    Ok(TextReading {
        text: without_anything_that_is_not_writing(&decoded),
        note: note_for(bytes.len(), kept.len(), entirely_text),
        bytes: bytes.len(),
        bytes_read: kept.len(),
        entirely_text,
    })
}

/// Whether these bytes are too far from text to be worth rendering as text.
///
/// Two questions. A NUL byte settles it on its own: no encoding this reads puts
/// one inside a file, and the binary formats a sender might mislabel are full
/// of them. Short of that it is the proportion of characters that are not
/// writing, judged after decoding so that a file which is not UTF-8 at all is
/// caught by its replacement characters rather than slipping through for having
/// no control bytes in it.
fn mostly_not_text(sample: &[u8]) -> bool {
    if sample.is_empty() {
        // An empty file is a text file with nothing in it, which the note says
        // rather than the refusal.
        return false;
    }
    if sample.contains(&0) {
        return true;
    }
    let decoded = String::from_utf8_lossy(sample);
    let characters = decoded.chars().count();
    let not_writing = decoded.chars().filter(|c| !is_writing(*c)).count();
    not_writing * ONE_CHARACTER_IN > characters
}

/// Whether a character is one somebody meant to write.
///
/// A tab and a newline are how a text file is laid out and a carriage return is
/// how half the world ends a line, so those three stay. Everything else the
/// standard calls a control character is not writing: at best it does nothing
/// audible, and at worst it is an escape sequence, which is a stranger deciding
/// what a window does. The replacement character counts as not writing too,
/// because it is this program's own mark for a byte it could not read.
fn is_writing(character: char) -> bool {
    matches!(character, '\t' | '\n' | '\r')
        || !(character.is_control() || character == char::REPLACEMENT_CHARACTER)
}

/// The text with everything that is not writing taken out.
///
/// The replacement characters stay: they are the visible evidence for what the
/// note says about the file not being entirely text, and taking them out would
/// leave words silently joined together with nothing showing where the gap was.
fn without_anything_that_is_not_writing(text: &str) -> String {
    text.chars()
        .filter(|c| is_writing(*c) || *c == char::REPLACEMENT_CHARACTER)
        .collect()
}

/// At most `most` bytes, ending where a character ends.
///
/// Cutting in the middle of a multi-byte character would turn the last one into
/// a replacement character and, with it, turn every whole file that happens to
/// be long into one this reports as "not entirely text". Backing off at most
/// three bytes is enough, because that is the longest a UTF-8 character is
/// after its first byte.
fn up_to_a_character_boundary(bytes: &[u8], most: usize) -> &[u8] {
    if bytes.len() <= most {
        return bytes;
    }
    let mut end = most;
    for _ in 0..3 {
        // A continuation byte is `10xxxxxx`, so an end sitting on one is an end
        // in the middle of a character.
        if end == 0 || bytes[end] & 0b1100_0000 != 0b1000_0000 {
            break;
        }
        end -= 1;
    }
    &bytes[..end]
}

/// What to say about this file before any of it is read.
fn note_for(bytes: usize, bytes_read: usize, entirely_text: bool) -> String {
    let mut parts = Vec::new();

    if bytes_read < bytes {
        parts.push(format!(
            "This file is {} kilobytes and the first {} of them are below. It \
             was cut there, so the end of the file is not here. Control S saves \
             the whole of it.",
            bytes.div_ceil(1024),
            bytes_read / 1024,
        ));
    }

    if !entirely_text {
        // The most useful thing this module says after the refusal. A word that
        // came out wrong is otherwise read as the sender's, and there is
        // nothing in the window to suggest otherwise.
        parts.push(
            "This file is not entirely text. What could not be read is shown as \
             a replacement character, so a word that looks wrong here is one \
             this could not read rather than one the sender wrote."
                .to_string(),
        );
    }

    if parts.is_empty() {
        parts.push("Read as text, the whole file.".to_string());
    }
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_text_file_comes_back_as_its_own_words() {
        let read = read(b"Hello, world.\nSecond line.\n").expect("plain text");

        assert_eq!(read.text, "Hello, world.\nSecond line.\n");
        assert!(read.entirely_text);
    }

    #[test]
    fn test_a_file_that_is_not_entirely_text_says_so_rather_than_being_refused() {
        // One byte that is not valid UTF-8 in a file that is otherwise prose.
        // Refusing the whole file over it would lose the prose; rendering it
        // silently would leave somebody puzzling over a word that came out
        // wrong with nothing saying why.
        let mut bytes = b"A perfectly ordinary sentence, and then ".to_vec();
        bytes.push(0xff);
        bytes.extend_from_slice(b" carries on afterwards.");

        let read = read(&bytes).expect("mostly text");

        assert!(!read.entirely_text);
        assert!(
            read.note.to_lowercase().contains("not entirely text"),
            "{}",
            read.note
        );
        assert!(read.text.contains("carries on afterwards"));
    }

    #[test]
    fn test_a_binary_file_labelled_as_text_is_refused_rather_than_rendered() {
        // A sender can label anything text/plain. Rendering a compiled program
        // in a reading window is a screenful of nonsense, which is worse than
        // saying it cannot be read.
        let mut bytes = b"MZ\x90\x00\x03\x00\x00\x00".to_vec();
        bytes.extend(vec![0u8; 500]);

        assert!(read(&bytes).is_err());
    }

    #[test]
    fn test_a_file_of_bytes_that_are_not_text_and_hold_no_nul_is_still_refused() {
        // The NUL check alone would let this through: nothing in it is a
        // control byte, and every byte of it is meaningless as UTF-8. It is the
        // proportion of replacement characters after decoding that catches it,
        // which is why the rule is asked after decoding rather than before.
        let bytes: Vec<u8> = (0..500).map(|i| 0x80 + (i % 0x40) as u8).collect();
        assert!(!bytes.contains(&0));

        assert!(read(&bytes).is_err());
    }

    #[test]
    fn test_a_file_over_the_bound_comes_back_cut_and_says_so() {
        // Cut is fine. Silently short is not: somebody who reaches the end
        // believes they have read the file.
        let bytes = "All work and no play. ".repeat(200_000).into_bytes();
        assert!(bytes.len() > LONGEST_READ_BYTES);

        let read = read(&bytes).expect("long prose");

        assert!(read.bytes_read < read.bytes);
        assert_eq!(read.bytes, bytes.len());
        assert!(read.text.len() <= LONGEST_READ_BYTES);
        assert!(read.note.to_lowercase().contains("cut"), "{}", read.note);
    }

    #[test]
    fn test_a_long_file_is_not_reported_as_not_entirely_text_for_having_been_cut() {
        // The cut lands mid-character unless something stops it, and the
        // replacement character that produces would say the sender's file was
        // damaged when the only thing that happened is that it was long.
        // Built so the cut really lands inside a character rather than
        // happening to miss one: the accented letter starts one byte before the
        // bound, so the byte at the bound is the second half of it. A fixture
        // that is merely long does not test this, which was measured by taking
        // the backing-off away and watching an earlier version stay green.
        let mut bytes = vec![b'a'; LONGEST_READ_BYTES - 1];
        bytes.extend_from_slice("éééé".as_bytes());
        assert_eq!(bytes[LONGEST_READ_BYTES] & 0b1100_0000, 0b1000_0000);

        let read = read(&bytes).expect("long prose");

        assert!(read.bytes_read < read.bytes);
        assert!(read.entirely_text);
        assert!(!read.text.contains(char::REPLACEMENT_CHARACTER));
    }

    #[test]
    fn test_control_characters_are_removed_and_tabs_and_newlines_are_not() {
        // An escape sequence reaching a reading window is a stranger deciding
        // what it does. A tab and a newline are how a text file is laid out.
        let read = read(b"Name\tValue\n\x1b[31mred\x1b[0m\nDone\n").expect("text with an escape");

        assert!(read.text.contains('\t'));
        assert!(read.text.contains('\n'));
        assert!(!read.text.contains('\u{1b}'), "{:?}", read.text);
        assert!(read.text.contains("red"));
    }

    #[test]
    fn test_a_note_is_always_something() {
        // A reader handed a file with no word about it has to work out for
        // themselves whether the silence means the file is fine.
        let read = read(b"Two words.").expect("plain text");

        assert!(!read.note.trim().is_empty());
    }

    #[test]
    fn test_an_empty_file_is_a_text_file_with_nothing_in_it() {
        // Refusing it would say the file could not be read, which is not what
        // happened: it was read and there was nothing there.
        let read = read(b"").expect("an empty file");

        assert!(read.text.is_empty());
        assert!(!read.note.trim().is_empty());
    }
}

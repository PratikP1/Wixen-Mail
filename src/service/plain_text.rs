//! Reading a text attachment, and saying how much of it is really there.
//!
//! RED half. `read` answers one outcome for everything so the tests below fail
//! on the answer rather than on a build that never happened. The real reading
//! arrives with the green commit.

use crate::common::Result;

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

/// A text file, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextReading {
    /// The words, with anything that would confuse a reading window taken out.
    pub text: String,
    /// What is worth saying about this file before it is read.
    pub note: String,
    /// How many bytes arrived.
    pub bytes: usize,
    /// How many were read, which is fewer when the file is very long.
    pub bytes_read: usize,
    /// Whether every byte decoded, or some were replaced.
    pub entirely_text: bool,
}

/// Read bytes into text.
pub fn read(bytes: &[u8]) -> Result<TextReading> {
    Ok(TextReading {
        text: String::new(),
        note: String::new(),
        bytes: bytes.len(),
        bytes_read: 0,
        entirely_text: true,
    })
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
}

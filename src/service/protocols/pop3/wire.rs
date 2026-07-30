//! Reading what a POP3 server says.
//!
//! POP3 is a line protocol with two shapes of answer. Every command gets a
//! status line beginning `+OK` or `-ERR`, and some commands follow it with
//! lines of data ending in a line that is a single full stop.
//!
//! The parsing lives here, away from the socket, because it is the half that
//! can be wrong in ways a connection test would not show: a UIDL line split on
//! the wrong space attributes a message to the wrong identifier, and a body
//! whose dots are not unstuffed loses a line from every message that happens to
//! start one with a full stop.

use crate::common::{Error, Result};

/// What the server said after a command.
///
/// The text after `+OK` is informational and differs between servers, so it is
/// carried rather than parsed. The one place it matters is a failure, where it
/// is the only thing that says why.
pub fn status(line: &str) -> Result<String> {
    let line = line.trim_end_matches(['\r', '\n']);
    if let Some(rest) = line.strip_prefix("+OK") {
        return Ok(rest.trim().to_string());
    }
    if let Some(rest) = line.strip_prefix("-ERR") {
        return Err(Error::Protocol(format!(
            "The mail server refused: {}",
            rest.trim()
        )));
    }
    Err(Error::Protocol(format!(
        "The mail server said something POP3 does not allow: {line}"
    )))
}

/// Whether a line ends a multi-line answer.
///
/// A single full stop, and nothing else. A line of `..` is a body line of `.`
/// that has been stuffed, not the end.
pub fn is_end_of_data(line: &str) -> bool {
    line.trim_end_matches(['\r', '\n']) == "."
}

/// Undo the dot-stuffing a server applies to body lines.
///
/// RFC 1939 says a line beginning with a full stop is sent with an extra one in
/// front, so the terminator cannot appear inside the data. Not undoing it
/// leaves a stray dot at the start of any line that began with one, which in a
/// message quoting a configuration file or a diff is silently wrong text.
pub fn unstuff(line: &str) -> &str {
    line.strip_prefix('.').unwrap_or(line)
}

/// One line of a UIDL listing: the message's number and its identifier.
///
/// The identifier is what makes POP3 usable at all. Message numbers are
/// assigned per session and shift as messages are deleted, so a number means
/// nothing between one connection and the next; the identifier is stable for
/// as long as the message is on the server, and is the only way to tell mail
/// that has already been downloaded from mail that has not.
pub fn uidl_line(line: &str) -> Option<(u32, String)> {
    let line = line.trim();
    let (number, id) = line.split_once(' ')?;
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    Some((number.parse().ok()?, id.to_string()))
}

/// One line of a LIST listing: the message's number and its size in bytes.
pub fn list_line(line: &str) -> Option<(u32, usize)> {
    let line = line.trim();
    let (number, size) = line.split_once(' ')?;
    Some((number.parse().ok()?, size.trim().parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_success_carries_what_the_server_said() {
        assert_eq!(status("+OK 3 messages\r\n").unwrap(), "3 messages");
        assert_eq!(status("+OK\r\n").unwrap(), "");
    }

    #[test]
    fn test_a_refusal_says_why() {
        // The server's own words are the only thing that tells somebody
        // whether it was the password, the mailbox being locked, or a quota.
        let refused = status("-ERR [AUTH] Invalid credentials\r\n").unwrap_err();

        assert!(
            refused.to_string().contains("Invalid credentials"),
            "{refused}"
        );
    }

    #[test]
    fn test_anything_that_is_neither_is_an_error_rather_than_a_success() {
        // A half-open connection returning rubbish must not read as +OK.
        assert!(status("HTTP/1.1 200 OK").is_err());
        assert!(status("").is_err());
    }

    #[test]
    fn test_a_lone_full_stop_ends_the_data() {
        assert!(is_end_of_data(".\r\n"));
        assert!(is_end_of_data("."));
    }

    #[test]
    fn test_a_stuffed_dot_is_not_the_end() {
        // A body line that is itself a full stop arrives as two. Reading it as
        // the terminator truncates the message there.
        assert!(!is_end_of_data("..\r\n"));
        assert!(!is_end_of_data(". \r\n"));
        assert!(!is_end_of_data(".signature"));
    }

    #[test]
    fn test_a_leading_dot_is_removed_from_a_body_line() {
        // A message quoting a configuration file or a diff has lines starting
        // with a full stop, and leaving the extra one in is silently wrong text.
        assert_eq!(unstuff("..hidden"), ".hidden");
        assert_eq!(unstuff(".."), ".");
        assert_eq!(unstuff("ordinary"), "ordinary");
    }

    #[test]
    fn test_a_uidl_line_gives_the_number_and_the_identifier() {
        assert_eq!(
            uidl_line("2 QhdPYR:00WBw1Ph7x7\r\n"),
            Some((2, "QhdPYR:00WBw1Ph7x7".to_string()))
        );
    }

    #[test]
    fn test_a_uidl_identifier_may_contain_anything_printable() {
        // RFC 1939 allows any printable character except space, so an
        // identifier with punctuation in it is ordinary rather than suspect.
        assert_eq!(
            uidl_line("7 <1234.5678@mail.example.com>"),
            Some((7, "<1234.5678@mail.example.com>".to_string()))
        );
    }

    #[test]
    fn test_a_uidl_line_that_makes_no_sense_is_skipped_rather_than_guessed() {
        // Better to miss one message than to file every message under an
        // identifier that came from a misread line.
        assert_eq!(uidl_line("not a number here"), None);
        assert_eq!(uidl_line("3"), None);
        assert_eq!(uidl_line("3 "), None);
        assert_eq!(uidl_line(""), None);
    }

    #[test]
    fn test_a_list_line_gives_the_number_and_the_size() {
        assert_eq!(list_line("1 2048\r\n"), Some((1, 2048)));
        assert_eq!(list_line("bad"), None);
        assert_eq!(list_line("1 enormous"), None);
    }
}

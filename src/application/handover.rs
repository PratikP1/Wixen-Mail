//! What one copy of Wixen Mail hands to the copy already running.
//!
//! # Why this exists
//!
//! Windows starts a fresh copy of a program every time somebody clicks a
//! `mailto:` link or opens an `.ics` file with it. Nothing about being the
//! registered handler makes Windows find the copy already open. So without
//! this, being chosen as the default mail program means every link on the
//! machine opens another whole mail client, all of them sharing one database.
//!
//! That is not only untidy. The outbox is a table with no notion of who is
//! sending: two copies both read the queued message, both send it, and the
//! person on the other end receives it twice. A sent message cannot be
//! recalled, which is what makes this worth solving properly rather than
//! living with.
//!
//! # What is sent
//!
//! The argument exactly as Windows gave it, and nothing else. Not the parsed
//! result: parsing happens once, in the copy that is going to act on it, so
//! there is one parser and one set of rules about what a link may ask for. A
//! second copy that parsed and sent the pieces would be a second place for
//! those rules to live and drift.
//!
//! An empty argument is a real message meaning "somebody started me with
//! nothing, so show yourself". That is what double-clicking the icon of an
//! already-running program should do.
//!
//! # Why it is checked rather than trusted
//!
//! Anything on the machine can connect to a named pipe. So the bytes carry a
//! marker and a version, and anything that does not begin with them is refused
//! without being looked at further. The argument itself stays untrusted after
//! that: it goes through the same parser as a command line, which is where the
//! rules about what a link may ask for already live.

use crate::common::{Error, Result};

/// The first bytes of anything Wixen Mail sends to itself.
///
/// A version rather than a bare marker, so a copy left running through an
/// upgrade meets a message it does not understand and says so, rather than
/// reading new bytes with old rules.
pub const MARKER: &[u8] = b"wixen-mail handover 1\n";

/// The most an argument may be.
///
/// Windows will not pass a command line longer than about 32,000 characters,
/// so anything past this did not come from Windows starting a program. Twice
/// that, because the cap is here to stop a stranger sending megabytes rather
/// than to police a length this application decides.
pub const MOST_AN_ARGUMENT_MAY_HOLD: usize = 64 * 1024;

/// The bytes to send.
pub fn encode(argument: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(MARKER.len() + argument.len());
    bytes.extend_from_slice(MARKER);
    bytes.extend_from_slice(argument.as_bytes());
    bytes
}

/// The argument inside a message, or why it is not one.
///
/// Refuses rather than guesses. A pipe with a well-known name is reachable by
/// anything running as this person, so the first question about a message is
/// always whether it is one of ours.
pub fn decode(bytes: &[u8]) -> Result<String> {
    let Some(rest) = bytes.strip_prefix(MARKER) else {
        return Err(Error::Other(
            "Something connected to Wixen Mail and sent it a message it did not recognise"
                .to_string(),
        ));
    };
    if rest.len() > MOST_AN_ARGUMENT_MAY_HOLD {
        return Err(Error::Other(format!(
            "A handover carried {} bytes, which is more than Windows could have \
             given a program on a command line",
            rest.len()
        )));
    }
    // Not lossy. A handover that is not text is not something this application
    // sent, and turning it into replacement characters would hand the rest of
    // the program a string somebody never typed.
    String::from_utf8(rest.to_vec())
        .map_err(|_| Error::Other("A handover was not text".to_string()))
}

/// What this start should do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HowToStart {
    /// Be the copy that runs. Nothing else is up, or nothing could be reached.
    CarryOn,
    /// Give this to the copy already running, then stop.
    HandOver(String),
}

/// Whether to run, or to hand what this start was given to the copy already up.
///
/// Handing over with nothing to hand is deliberate and is not a special case:
/// somebody who starts a program that is already running means "show me the
/// one that is running", which is what every other Windows application does.
pub fn how_to_start(another_is_running: bool, argument: Option<&str>) -> HowToStart {
    if !another_is_running {
        return HowToStart::CarryOn;
    }
    HowToStart::HandOver(argument.unwrap_or_default().to_string())
}

/// What a copy that has just been handed something should do about its window.
///
/// Always shown and raised, whether or not anything came with it. A handover
/// that opened a composer behind another program's window would look exactly
/// like a link that did nothing.
pub fn what_to_say_when_handed(argument: &str) -> &'static str {
    if argument.trim().is_empty() {
        "Wixen Mail is already running, and this is it."
    } else {
        "Wixen Mail is already running, and opened this here."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_an_argument_survives_being_sent_and_read_back() {
        let sent = encode("mailto:someone@example.com?subject=Hello%20there");

        let read = decode(&sent).expect("what this application sent to itself");

        assert_eq!(read, "mailto:someone@example.com?subject=Hello%20there");
    }

    #[test]
    fn test_nothing_to_hand_over_is_itself_a_message() {
        // Starting a program that is already running means "show me the one
        // that is running". An empty argument has to survive the trip for that
        // to be distinguishable from no message at all.
        let read = decode(&encode("")).expect("an empty handover");

        assert!(read.is_empty());
    }

    #[test]
    fn test_something_that_did_not_come_from_this_application_is_refused() {
        // A named pipe is reachable by anything running as this person. The
        // first question about a message is whether it is one of ours, and the
        // answer has to be no before any of it is read as a link.
        let refused = decode(b"mailto:someone@example.com");

        assert!(refused.is_err(), "a bare link was accepted as a handover");
    }

    #[test]
    fn test_a_message_from_a_different_version_is_refused_rather_than_guessed_at() {
        // A copy left running through an upgrade. Reading new bytes with old
        // rules is how a handover turns into something nobody sent.
        let from_later = b"wixen-mail handover 2\nmailto:someone@example.com";

        assert!(decode(from_later).is_err());
    }

    #[test]
    fn test_more_than_windows_could_have_given_is_refused() {
        // Past this, it did not come from Windows starting a program, so it
        // came from something else and there is no reason to read it.
        let mut enormous = MARKER.to_vec();
        enormous.extend(std::iter::repeat_n(b'x', MOST_AN_ARGUMENT_MAY_HOLD + 1));

        assert!(decode(&enormous).is_err());
    }

    #[test]
    fn test_a_handover_that_is_not_text_is_refused_rather_than_patched_up() {
        // Lossy decoding would hand the rest of the program a string somebody
        // never typed, made of replacement characters.
        let mut bytes = MARKER.to_vec();
        bytes.extend_from_slice(&[0xff, 0xfe, 0xfd]);

        assert!(decode(&bytes).is_err());
    }

    #[test]
    fn test_the_first_copy_runs() {
        assert_eq!(
            how_to_start(false, Some("mailto:a@b.com")),
            HowToStart::CarryOn
        );
        assert_eq!(how_to_start(false, None), HowToStart::CarryOn);
    }

    #[test]
    fn test_a_second_copy_hands_over_what_it_was_given() {
        assert_eq!(
            how_to_start(true, Some("mailto:a@b.com")),
            HowToStart::HandOver("mailto:a@b.com".to_string())
        );
    }

    #[test]
    fn test_a_second_copy_with_nothing_to_hand_over_still_hands_over() {
        // Otherwise double-clicking the icon of a running program starts a
        // second one, which is the whole thing this exists to stop.
        assert_eq!(
            how_to_start(true, None),
            HowToStart::HandOver(String::new())
        );
    }

    #[test]
    fn test_what_is_said_tells_the_two_kinds_of_handover_apart() {
        // A window coming to the front with nothing else happening, against a
        // window coming to the front with a message now open in it. Somebody
        // who cannot see the screen has only the sentence to go on.
        assert_ne!(
            what_to_say_when_handed(""),
            what_to_say_when_handed("mailto:a@b.com")
        );
        assert!(what_to_say_when_handed("").contains("already running"));
    }
}

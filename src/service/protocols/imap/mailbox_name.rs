//! Mailbox names on the wire, and mailbox names a person can read.
//!
//! IMAP4rev1 carries non-ASCII mailbox names in a modified UTF-7 encoding
//! (RFC 3501 section 5.1.3), so a German user's Drafts folder arrives as
//! `Entw&APw-rfe` and a Japanese one as `&ZeVnLIqe-`. Left alone, that is what
//! the folder tree announces, which for a screen reader user is not a slightly
//! ugly label but an unreadable one: the synthesiser spells out punctuation.
//!
//! Two differences from ordinary Base64 UTF-7: `/` is written as `,` because
//! `/` is a common hierarchy delimiter, and the padding `=` is omitted.
//!
//! A name the server spelled goes back to the server exactly as it arrived,
//! which is the only way a name we could not decode stays reachable. That was
//! once the whole story, and [`decode`] was the whole module: nothing sent a
//! mailbox name this program made up. Creating a folder is the day that
//! changed, so [`encode`] is here now, sharing the engine below rather than
//! configuring a second one, because two engines is how a name that decodes
//! one way comes to encode another.
//!
//! RFC 6855 lets a client ask for UTF-8 mailbox names with `ENABLE UTF8=ACCEPT`,
//! and plenty of servers still do not offer it, so decoding stays necessary.

use base64::Engine as _;
use base64::alphabet::Alphabet;
use base64::engine::DecodePaddingMode;
use base64::engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig};
use std::sync::LazyLock;

/// Base64 as RFC 3501 spells it: the standard alphabet with `,` for 63.
static MODIFIED_BASE64: LazyLock<GeneralPurpose> = LazyLock::new(|| {
    let alphabet =
        Alphabet::new("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+,")
            .unwrap_or(base64::alphabet::STANDARD);
    let config = GeneralPurposeConfig::new()
        .with_encode_padding(false)
        .with_decode_padding_mode(DecodePaddingMode::RequireNone)
        // A final group can leave stray low bits. The specification says they
        // are zero; a server that sets them anyway is not a reason to hide a
        // folder from the person who owns it.
        .with_decode_allow_trailing_bits(true);
    GeneralPurpose::new(&alphabet, config)
});

/// Turn a mailbox name from the wire into text.
///
/// A name that is not valid modified UTF-7 comes back as it arrived. A folder
/// with an odd name should still be listed and still be selectable: the name is
/// the server's identifier for it, so refusing to show it loses the folder.
pub fn decode(encoded: &str) -> String {
    let mut out = String::with_capacity(encoded.len());
    let mut rest = encoded;

    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let after = &rest[amp + 1..];
        let Some(end) = after.find('-') else {
            // An unterminated run. Everything from here is not decodable, so
            // keep it verbatim rather than dropping the tail.
            out.push_str(&rest[amp..]);
            return out;
        };
        let chunk = &after[..end];
        if chunk.is_empty() {
            // `&-` is how a literal ampersand is written.
            out.push('&');
        } else {
            match decode_run(chunk) {
                Some(text) => out.push_str(&text),
                // Not decodable: keep the original run, delimiters and all.
                None => out.push_str(&rest[amp..amp + 2 + end]),
            }
        }
        rest = &after[end + 1..];
    }

    out.push_str(rest);
    out
}

/// Decode one `&...-` run: modified Base64 over UTF-16BE code units.
fn decode_run(chunk: &str) -> Option<String> {
    let bytes = MODIFIED_BASE64.decode(chunk).ok()?;
    if bytes.len() % 2 != 0 {
        return None;
    }
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
        .collect();
    // An unpaired surrogate is a malformed name, not a character we can show.
    char::decode_utf16(units)
        .collect::<Result<String, _>>()
        .ok()
}

/// Turn a mailbox name into the spelling the wire carries.
///
/// The inverse of [`decode`], and it has to be exact rather than close. The
/// IMAP library sends whatever it is handed: `validate_str` quotes the name and
/// refuses a line break and does nothing else, so a folder created as
/// `Entwürfe` on a server that speaks RFC 3501 is a folder literally called
/// that, which this client and every other one then shows as punctuation a
/// synthesiser reads out one character at a time. Working in English and
/// corrupting every other alphabet is the failure this exists to stop.
///
/// Only printable US-ASCII stands for itself, so a control character goes out
/// escaped rather than as a byte that could end one command and begin another.
pub fn encode(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut run = String::new();

    for ch in name.chars() {
        if !is_printable_ascii(ch) {
            // Neighbours share one escape. Closing and reopening per character
            // decodes to the same name and is a different string, and a server
            // comparing mailbox names compares strings.
            run.push(ch);
            continue;
        }
        if !run.is_empty() {
            out.push_str(&encode_run(&run));
            run.clear();
        }
        if ch == '&' {
            // The inverse of the case `decode` names above.
            out.push_str("&-");
        } else {
            out.push(ch);
        }
    }
    if !run.is_empty() {
        out.push_str(&encode_run(&run));
    }
    out
}

/// Whether a character can appear on the wire as itself.
///
/// RFC 3501 section 5.1.3: printable US-ASCII, and everything else lives inside
/// an escape. `&` is printable and still does not stand for itself, so the
/// caller asks about it separately rather than this answering two questions.
fn is_printable_ascii(ch: char) -> bool {
    matches!(ch, ' '..='~')
}

/// Encode one run as `&...-`: modified Base64 over UTF-16BE code units.
///
/// `encode_utf16` is what puts a character outside the basic plane out as the
/// surrogate pair the wire format is defined in terms of.
fn encode_run(chunk: &str) -> String {
    let mut bytes = Vec::with_capacity(chunk.len() * 2);
    for unit in chunk.encode_utf16() {
        bytes.extend_from_slice(&unit.to_be_bytes());
    }
    format!("&{}-", MODIFIED_BASE64.encode(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_an_ascii_name_is_unchanged() {
        assert_eq!(decode("INBOX"), "INBOX");
        assert_eq!(decode("INBOX/Work/2026"), "INBOX/Work/2026");
    }

    #[test]
    fn test_a_german_drafts_folder_reads_as_words() {
        // The label a screen reader would otherwise spell out as
        // "Entw ampersand A P w hyphen r f e".
        assert_eq!(decode("Entw&APw-rfe"), "Entw\u{fc}rfe");
    }

    #[test]
    fn test_the_examples_from_rfc_3501() {
        assert_eq!(
            decode("~peter/mail/&U,BTFw-/&ZeVnLIqe-"),
            "~peter/mail/\u{53f0}\u{5317}/\u{65e5}\u{672c}\u{8a9e}"
        );
    }

    #[test]
    fn test_an_undecodable_name_is_shown_rather_than_lost() {
        // The folder still exists on the server and the user still needs to
        // reach it, so a name we cannot read is better than no row at all.
        assert_eq!(decode("Broken&"), "Broken&");
        assert_eq!(decode("Broken&%%%-tail"), "Broken&%%%-tail");
        // An odd number of octets cannot be UTF-16.
        assert_eq!(decode("&AQ-"), "&AQ-");
    }

    #[test]
    fn test_an_unpaired_surrogate_is_not_invented_into_a_character() {
        // 0xD800 alone is not a character. Guessing one would put a wrong
        // label on a real folder.
        assert_eq!(decode("&2AA-"), "&2AA-");
    }

    #[test]
    fn test_decoding_never_panics_on_arbitrary_input() {
        // Names arrive from a server, so they are arbitrary bytes that happened
        // to be valid UTF-8.
        for name in [
            "",
            "&",
            "&-",
            "&&",
            "&&&&-",
            "-",
            "---",
            "&-&-&-",
            "\u{4f60}\u{597d}",
            "&\u{1f600}-",
            "&AAAAAAAAAAAA-",
            "a&b&c&d",
        ] {
            let decoded = decode(name);
            // Valid UTF-8 by construction; the round trip proves nothing was
            // cut mid-character.
            assert_eq!(decoded, String::from_utf8_lossy(decoded.as_bytes()));
        }
    }

    // ── Going the other way: a name this program made up, on the wire ────────

    #[test]
    fn test_an_ascii_name_goes_out_unchanged() {
        assert_eq!(encode("Work"), "Work");
        assert_eq!(encode("INBOX/Work/2026"), "INBOX/Work/2026");
    }

    #[test]
    fn test_a_literal_ampersand_is_escaped() {
        // The inverse of the case `decode` names: `&-` is how a name that
        // really carries an ampersand is written, and a bare `&` would open an
        // escape the server then never sees closed.
        assert_eq!(encode("R&D"), "R&-D");
        assert_eq!(encode("&"), "&-");
        assert_eq!(encode("Tom & Jerry"), "Tom &- Jerry");
    }

    #[test]
    fn test_a_german_drafts_folder_goes_out_as_the_wire_spells_it() {
        assert_eq!(encode("Entw\u{fc}rfe"), "Entw&APw-rfe");
    }

    #[test]
    fn test_each_run_of_non_ascii_gets_its_own_escape() {
        // Two runs with readable text between them, so the scanner has to
        // close one escape and open another rather than swallowing the ASCII
        // in the middle.
        assert_eq!(
            encode("Bo\u{ee}te de r\u{e9}ception"),
            "Bo&AO4-te de r&AOk-ception"
        );
    }

    #[test]
    fn test_neighbouring_non_ascii_characters_share_one_escape() {
        // Not one escape each. Both spellings decode to the same name, so only
        // a test reading the wire form can tell them apart, and a server
        // comparing mailbox names as strings cannot.
        assert_eq!(encode("\u{53f0}\u{5317}"), "&U,BTFw-");
    }

    #[test]
    fn test_a_character_outside_the_basic_plane_becomes_a_surrogate_pair() {
        // A run is UTF-16BE code units, so an emoji is two of them inside one
        // escape rather than a character the encoder has no room for.
        let encoded = encode("\u{1f600}");
        assert_eq!(encoded, "&2D3eAA-");
        assert_eq!(encoded.matches('&').count(), 1);
        assert_eq!(decode(&encoded), "\u{1f600}");
    }

    #[test]
    fn test_a_control_character_is_never_sent_as_itself() {
        // Only printable US-ASCII stands for itself. A name carrying anything
        // below a space goes out escaped rather than as bytes that could end
        // one command and begin another.
        assert_eq!(encode("a\u{1}b"), "a&AAE-b");
        assert!(!encode("a\rb").contains('\r'));
        assert!(!encode("a\nb").contains('\n'));
    }

    #[test]
    fn test_encoding_and_decoding_are_inverses() {
        // The pair is what makes the encoder trustworthy. `decode` is already
        // tested against RFC 3501's own examples, so a name that survives the
        // round trip was spelled the way the server spells it.
        for name in [
            "INBOX",
            "Work",
            "R&D",
            "Entw\u{fc}rfe",
            "~peter/mail/\u{53f0}\u{5317}/\u{65e5}\u{672c}\u{8a9e}",
            "Bo\u{ee}te de r\u{e9}ception",
            "\u{1f600}",
            "&-&-",
            "",
        ] {
            assert_eq!(
                decode(&encode(name)),
                name,
                "round trip failed for {name:?}"
            );
        }
    }

    #[test]
    fn test_the_examples_from_rfc_3501_survive_the_round_trip() {
        // The same name `test_the_examples_from_rfc_3501` decodes, encoded
        // back. This is the assertion that would notice the alphabet losing
        // its comma for 63.
        let readable = "~peter/mail/\u{53f0}\u{5317}/\u{65e5}\u{672c}\u{8a9e}";
        let on_the_wire = "~peter/mail/&U,BTFw-/&ZeVnLIqe-";
        assert_eq!(encode(readable), on_the_wire);
        assert_eq!(decode(on_the_wire), readable);
    }
}

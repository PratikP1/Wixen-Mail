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

/// Turn readable text into a mailbox name for the wire.
///
/// Needed whenever a name goes back to the server: SELECT, CREATE, RENAME, and
/// the folder argument of a COPY.
pub fn encode(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut pending: Vec<u16> = Vec::new();

    for ch in name.chars() {
        if is_direct(ch) {
            flush(&mut pending, &mut out);
            if ch == '&' {
                out.push_str("&-");
            } else {
                out.push(ch);
            }
        } else {
            let mut buf = [0u16; 2];
            pending.extend_from_slice(ch.encode_utf16(&mut buf));
        }
    }
    flush(&mut pending, &mut out);
    out
}

/// Whether a character stands for itself on the wire.
///
/// Printable ASCII only. `&` is printable and stands for itself as `&-`, which
/// the caller handles; everything else here is copied through.
fn is_direct(ch: char) -> bool {
    matches!(ch, ' '..='~')
}

/// Emit the pending UTF-16 run as one `&...-` group.
fn flush(pending: &mut Vec<u16>, out: &mut String) {
    if pending.is_empty() {
        return;
    }
    let mut bytes = Vec::with_capacity(pending.len() * 2);
    for unit in pending.drain(..) {
        bytes.extend_from_slice(&unit.to_be_bytes());
    }
    out.push('&');
    out.push_str(&MODIFIED_BASE64.encode(&bytes));
    out.push('-');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_an_ascii_name_is_unchanged_both_ways() {
        assert_eq!(decode("INBOX"), "INBOX");
        assert_eq!(encode("INBOX"), "INBOX");
        assert_eq!(decode("INBOX/Work/2026"), "INBOX/Work/2026");
        assert_eq!(encode("INBOX/Work/2026"), "INBOX/Work/2026");
    }

    #[test]
    fn test_a_german_drafts_folder_reads_as_words() {
        // The label a screen reader would otherwise spell out as
        // "Entw ampersand A P w hyphen r f e".
        assert_eq!(decode("Entw&APw-rfe"), "Entw\u{fc}rfe");
        assert_eq!(encode("Entw\u{fc}rfe"), "Entw&APw-rfe");
    }

    #[test]
    fn test_the_examples_from_rfc_3501() {
        assert_eq!(
            decode("~peter/mail/&U,BTFw-/&ZeVnLIqe-"),
            "~peter/mail/\u{53f0}\u{5317}/\u{65e5}\u{672c}\u{8a9e}"
        );
        assert_eq!(
            encode("~peter/mail/\u{53f0}\u{5317}/\u{65e5}\u{672c}\u{8a9e}"),
            "~peter/mail/&U,BTFw-/&ZeVnLIqe-"
        );
    }

    #[test]
    fn test_a_literal_ampersand_survives_the_round_trip() {
        assert_eq!(decode("R&-D"), "R&D");
        assert_eq!(encode("R&D"), "R&-D");
        assert_eq!(decode(&encode("Bed & Breakfast")), "Bed & Breakfast");
    }

    #[test]
    fn test_a_name_outside_the_basic_plane_survives() {
        // Emoji are two UTF-16 units, and a folder named with one is a real
        // thing people do.
        let name = "Fun \u{1f600}";
        assert_eq!(decode(&encode(name)), name);
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

    #[test]
    fn test_every_name_we_encode_decodes_back_to_itself() {
        for name in [
            "INBOX",
            "Entw\u{fc}rfe",
            "\u{53f0}\u{5317}",
            "R&D",
            "&",
            "Fun \u{1f600}",
            "Mixed \u{4f60}\u{597d} and ASCII",
            "INBOX/\u{440}\u{430}\u{431}\u{43e}\u{442}\u{430}",
            " leading and trailing ",
        ] {
            assert_eq!(decode(&encode(name)), name, "round trip failed for {name}");
        }
    }
}

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

/// Where this mailbox's own name starts inside its path.
///
/// Nothing stores what a server separates folders with. It does not have to:
/// the folder above is a prefix of this one, so whatever sits between the end
/// of that prefix and the end of this path is the separator the server gave for
/// this mailbox, read rather than guessed. A slash on one server and a dot on
/// the next, and neither assumed for the other.
///
/// `inside` is the stored path of the folder this one sits under, or `None` for
/// one at the top level, whose whole path is its name. A parent that is not a
/// prefix is a stored tree disagreeing with itself, and the answer is the same
/// as no parent at all: the whole path is the name, so a rename replaces the
/// folder whole rather than slicing it at an offset taken from a string it has
/// nothing to do with.
fn where_the_leaf_starts(path: &str, inside: Option<&str>) -> usize {
    match inside {
        Some(parent) if !parent.is_empty() && path.len() > parent.len() => path
            .strip_prefix(parent)
            .map_or(0, |after| path.len() - after.len() + separator_width(after)),
        _ => 0,
    }
}

/// How much of what follows the parent's path is separator rather than name.
///
/// A separator is one or more characters of the server's choosing and never
/// empty, since a mailbox `LIST` line carrying no separator is a flat namespace
/// with nothing under it. What decides where it ends is the encoding: a
/// separator is ASCII punctuation, and a name that is not ASCII opens with `&`,
/// so the separator is everything up to the first character a name could start
/// with. In practice servers use one character, and taking the first is what
/// every reading of this has to agree on.
fn separator_width(after_the_parent: &str) -> usize {
    after_the_parent.chars().next().map_or(0, char::len_utf8)
}

/// The path this mailbox would have if its own name were changed to `typed`.
///
/// D-26's rename, and the whole of its safety argument: everything in front of
/// the last segment is carried over exactly, so a name typed into a text box
/// cannot move a folder.
///
/// `typed` is a name this program was handed by a person, so it goes out
/// encoded. The path in front of it is the server's own spelling and goes back
/// as it arrived. Encoding that again spells a mailbox the server has not got,
/// which is the failure [`decode`]'s own note above is about from the other
/// direction.
pub fn the_path_after_a_rename(path: &str, inside: Option<&str>, typed: &str) -> String {
    format!(
        "{}{}",
        &path[..where_the_leaf_starts(path, inside)],
        encode(typed)
    )
}

/// The path this mailbox would have under a different folder.
///
/// D-26's move, which is the opposite half: the name is untouched and only what
/// is in front of it changes. So the leaf is carried over verbatim, including
/// the one this client could not decode, and nothing here encodes anything.
///
/// `into` is the new parent's path as the server spells it, and `separator` is
/// what that server puts between a folder and the one it is in, read from the
/// server rather than assumed. RFC 9051 section 6.3.6 has the server rename
/// every inferior name along with this one, so the children need no path of
/// their own.
pub fn the_path_after_a_move(
    path: &str,
    inside: Option<&str>,
    into: &str,
    separator: &str,
) -> String {
    format!(
        "{into}{separator}{}",
        &path[where_the_leaf_starts(path, inside)..]
    )
}

/// What this server puts between a folder and the one it sits in.
///
/// Read from the pair rather than stored or guessed, the same way
/// [`where_the_leaf_starts`] reads it and for the same reason: a `LIST` line
/// carries a separator per mailbox, so one taken from the first line and used
/// for the rest splits the wrong names.
///
/// Empty for a folder at the top level, and empty where the stored tree
/// disagrees with itself and the folder above is not a prefix of this one. Both
/// mean the same thing to a caller: there is no join here to copy.
pub fn the_separator_between<'a>(path: &'a str, inside: Option<&str>) -> &'a str {
    let leaf = where_the_leaf_starts(path, inside);
    match inside {
        Some(parent) if leaf > parent.len() => &path[parent.len()..leaf],
        _ => "",
    }
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

    // ── Taking a path apart without the separator being stored ──────────────

    #[test]
    fn test_a_folder_at_the_top_level_is_all_leaf() {
        assert_eq!(where_the_leaf_starts("Archive", None), 0);
    }

    #[test]
    fn test_the_separator_is_read_from_the_gap_between_the_two_paths() {
        // Nothing anywhere stores what this server separates folders with. It
        // does not have to: the folder above is a prefix of this one, so
        // whatever sits between the two is the server's own answer for this
        // mailbox. A slash here, a dot on the next server, and neither guessed.
        assert_eq!(where_the_leaf_starts("Archive/2026", Some("Archive")), 8);
        assert_eq!(where_the_leaf_starts("Archive.2026", Some("Archive")), 8);
        assert_eq!(
            where_the_leaf_starts("INBOX.Work.2026", Some("INBOX.Work")),
            11
        );
    }

    #[test]
    fn test_a_parent_that_is_not_a_prefix_leaves_the_whole_path_as_the_leaf() {
        // A stored tree that disagrees with itself. Answering "all of it is
        // the name" renames the folder whole rather than slicing a path at an
        // offset taken from a string it has nothing to do with.
        assert_eq!(where_the_leaf_starts("Archive/2026", Some("Other")), 0);
        // A parent recorded as the folder itself is the same disagreement.
        assert_eq!(
            where_the_leaf_starts("Archive/2026", Some("Archive/2026")),
            0
        );
    }

    #[test]
    fn test_a_rename_changes_the_last_segment_and_nothing_before_it() {
        // The whole of D-26's safety argument, in one assertion.
        assert_eq!(
            the_path_after_a_rename("Archive/2026", Some("Archive"), "2025"),
            "Archive/2025"
        );
        assert_eq!(
            the_path_after_a_rename("INBOX.Work.Old", Some("INBOX.Work"), "Older"),
            "INBOX.Work.Older"
        );
    }

    #[test]
    fn test_a_rename_at_the_top_level_replaces_the_whole_path() {
        assert_eq!(the_path_after_a_rename("Archive", None, "Old"), "Old");
    }

    #[test]
    fn test_a_rename_encodes_the_new_name_and_leaves_the_rest_of_the_path_alone() {
        // The half an implementation forgets. The name somebody typed is a
        // name this program made up, so it is encoded on the way out. The
        // path in front of it is the server's own spelling and goes back
        // exactly as it arrived: encoding that again spells a mailbox the
        // server has not got.
        assert_eq!(
            the_path_after_a_rename("Entw&APw-rfe/Alt", Some("Entw&APw-rfe"), "Entw\u{fc}rfe"),
            "Entw&APw-rfe/Entw&APw-rfe"
        );
    }

    #[test]
    fn test_a_move_keeps_the_name_exactly_and_changes_only_what_is_in_front() {
        // The other half of D-26. A move does not rename, so the leaf is the
        // server's own spelling and is carried over untouched, including the
        // one this client could never decode.
        assert_eq!(
            the_path_after_a_move("Archive/&ZeVnLIqe-", Some("Archive"), "Old", "/"),
            "Old/&ZeVnLIqe-"
        );
        assert_eq!(
            the_path_after_a_move("Archive.2026", Some("Archive"), "Old.Kept", "."),
            "Old.Kept.2026"
        );
    }

    #[test]
    fn test_a_move_of_a_top_level_folder_puts_the_whole_path_under_the_new_parent() {
        assert_eq!(
            the_path_after_a_move("Work", None, "Archive", "/"),
            "Archive/Work"
        );
    }

    #[test]
    fn test_the_separator_is_whatever_this_server_put_between_the_two() {
        assert_eq!(the_separator_between("Archive/2026", Some("Archive")), "/");
        assert_eq!(the_separator_between("Archive.2026", Some("Archive")), ".");
        assert_eq!(
            the_separator_between("INBOX.Work.Old", Some("INBOX.Work")),
            "."
        );
    }

    #[test]
    fn test_a_folder_at_the_top_level_has_nothing_in_front_of_it_to_copy() {
        assert_eq!(the_separator_between("Archive", None), "");
    }

    #[test]
    fn test_a_folder_above_that_is_not_a_prefix_offers_no_separator() {
        // A stored tree disagreeing with itself. Answering "there is no join
        // here" is what stops a caller copying a character out of a string
        // this folder has nothing to do with.
        assert_eq!(the_separator_between("Archive/2026", Some("Work")), "");
    }

    #[test]
    fn test_a_folder_inside_a_renamed_one_follows_it_by_the_two_halves_it_is_made_of() {
        // RFC 9051 section 6.3.6 has the server rename every folder inside the
        // one it was asked about, in the same command and without being told.
        // Nothing else goes out, so the rows stored here have to be moved to
        // match, and this is how one is worked out: the folder above it has
        // already moved, and this keeps its own name and the join in front of
        // it. No prefix is swapped and no separator is assumed, so `Archived`
        // cannot be dragged along by a rename of `Archive`.
        let separator = the_separator_between("Archive/2026", Some("Archive"));
        assert_eq!(
            the_path_after_a_move("Archive/2026", Some("Archive"), "Old", separator),
            "Old/2026"
        );
    }
}

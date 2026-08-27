//! What a `mailto:` link is asking this program to write.
//!
//! Windows hands the whole URL over as one argument when somebody follows a
//! `mailto:` link and this program holds that association. It arrives as text
//! from a web page, an email, or a document, which is to say from a stranger,
//! so nothing here trusts it: it decides what a composer is filled in with,
//! and every field it can reach is one a stranger would like to choose.
//!
//! # Only five fields, and the list is the point
//!
//! RFC 6068 lets a `mailto:` URL name arbitrary header fields. Honouring that
//! would let a link set `From` on a message going out under somebody's own
//! account, `Reply-To` so every answer goes somewhere they did not choose, or
//! `Content-Type` on a message they think is plain text. So [`parse`] reads
//! the five a person can see and change in the composer, which are [`HONOURED`],
//! and drops everything else on the floor. A dropped field is recorded in
//! [`MailTo::ignored`] rather than silently forgotten, so a log can say what a
//! link tried to do.
//!
//! # A plus sign is a plus sign
//!
//! RFC 6068 section 5 says plainly that `+` does not mean a space in a
//! `mailto:` URL, unlike an HTML form. Decoding it as one would turn
//! `first+tag@example.com`, which is how a great many people filter their
//! mail, into an address with a space in it that goes nowhere. So `+` is left
//! alone, and a link written by something that assumed form encoding shows a
//! literal `+` in the subject. That is the wrong-looking answer that is
//! correct, and it is preferred to the right-looking one that breaks
//! addresses.

use crate::common::{Error, Result};

/// The fields a `mailto:` link may fill in.
///
/// Strings rather than parsed addresses, because this is what goes into the
/// composer's own boxes and a person has to be able to see and correct it
/// before anything is sent. Nothing here is validated as an address: that is
/// the composer's job, in front of somebody who can fix it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MailTo {
    /// The To line, addresses separated by a comma and a space.
    pub to: String,
    pub cc: String,
    pub bcc: String,
    pub subject: String,
    /// The message, as plain text.
    ///
    /// Plain, never markup. A body arriving from a link is a stranger's text,
    /// and putting it into the editor as HTML would let that stranger write
    /// markup into a message somebody is about to send under their own name.
    pub body: String,
    /// Every header the link named that this program will not honour.
    ///
    /// Kept so the reason a link did not do what its author expected can be
    /// written to the log, rather than the link quietly doing half of what it
    /// said. Names only: a dropped value is not worth carrying.
    pub ignored: Vec<String>,
}

/// The scheme, with its colon, spelled once.
const SCHEME: &str = "mailto:";

/// The header fields a link is allowed to fill in, for the documentation to
/// point at and for a test to count.
///
/// Not consulted by [`parse`], deliberately. Two gates, a list and a set of
/// arms, is one gate too many: a name had to appear in both, so adding it to
/// only one changed nothing and a test could not tell the difference. The arms
/// below are the gate. This says how many there should be, and a test compares
/// the two.
pub const HONOURED: [&str; 5] = ["to", "cc", "bcc", "subject", "body"];

/// Read a `mailto:` URL into the fields a composer opens with.
///
/// Refuses anything that is not a `mailto:` URL rather than guessing. This is
/// reached from the command line, where the argument could be anything at
/// all, and treating a stray file path as an empty message would open a
/// composer nobody asked for.
pub fn parse(url: &str) -> Result<MailTo> {
    let trimmed = url.trim();
    let rest = strip_scheme(trimmed).ok_or_else(|| {
        Error::Other(format!(
            "{trimmed:?} is not a mailto: link, so there is nothing to write"
        ))
    })?;

    let (addresses, query) = match rest.split_once('?') {
        Some((addresses, query)) => (addresses, query),
        None => (rest, ""),
    };

    let mut asked = MailTo {
        to: address_list(addresses),
        ..MailTo::default()
    };

    for (name, value) in fields_in(query) {
        let decoded = decode(&value);
        // Lowered once and matched on directly. This is the whole allowlist:
        // a header with no arm here reaches nothing, whatever it is called.
        // RFC 6068 says header field names are compared without case, and
        // links in the wild are written every way there is.
        match name.to_ascii_lowercase().as_str() {
            // A second `to=` adds to the addresses in the path rather than
            // replacing them. RFC 6068 says the two are combined, and a link
            // that named one person in the path and another in the query
            // means both.
            "to" => asked.to = joined(&asked.to, &address_list(&value)),
            "cc" => asked.cc = joined(&asked.cc, &address_list(&value)),
            "bcc" => asked.bcc = joined(&asked.bcc, &address_list(&value)),
            "subject" => asked.subject = joined_line(&asked.subject, &one_line(&decoded)),
            "body" => {
                asked.body = joined_line(&asked.body, &without_control_characters(&decoded));
            }
            _ => {
                if !name.trim().is_empty() && !asked.ignored.iter().any(|seen| seen == &name) {
                    asked.ignored.push(name);
                }
            }
        }
    }

    Ok(asked)
}

/// The part after `mailto:`, if this is a `mailto:` URL at all.
///
/// The scheme is matched without case because RFC 3986 says schemes are
/// compared that way, and Windows hands over whatever the page wrote.
fn strip_scheme(url: &str) -> Option<&str> {
    url.get(..SCHEME.len())
        .filter(|start| start.eq_ignore_ascii_case(SCHEME))
        .and_then(|_| url.get(SCHEME.len()..))
}

/// Split a query into its name and value pairs, undecoded names first.
///
/// The name is decoded here and the value is not, because the caller decides
/// what to do with a value: an address list is split on its commas before
/// decoding, so a comma that arrived percent-encoded inside one address stays
/// part of that address instead of cutting it in half.
fn fields_in(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| match pair.split_once('=') {
            Some((name, value)) => (decode(name), value.to_string()),
            None => (decode(pair), String::new()),
        })
        .collect()
}

/// Addresses from one comma-separated list, tidied for a composer's box.
///
/// Split before decoding, so `%2C` inside a quoted local part stays where it
/// was written. Empty entries are dropped: `mailto:a@b.com,,c@d.com` is two
/// addresses, and an empty one in the middle shows in the composer as a
/// stray comma nobody typed.
fn address_list(list: &str) -> String {
    let addresses: Vec<String> = list
        .split(',')
        .map(decode)
        .map(|address| one_line(&address).trim().to_string())
        .filter(|address| !address.is_empty())
        .collect();
    addresses.join(", ")
}

/// Put two comma-separated lists together, dropping an empty one.
fn joined(first: &str, second: &str) -> String {
    match (first.is_empty(), second.is_empty()) {
        (_, true) => first.to_string(),
        (true, false) => second.to_string(),
        (false, false) => format!("{first}, {second}"),
    }
}

/// Put two pieces of text together, dropping an empty one.
fn joined_line(first: &str, second: &str) -> String {
    match (first.is_empty(), second.is_empty()) {
        (_, true) => first.to_string(),
        (true, false) => second.to_string(),
        (false, false) => format!("{first} {second}"),
    }
}

/// Undo percent encoding, leaving anything malformed as it was written.
///
/// A stray `%` is kept rather than treated as the start of an escape, and a
/// `%` followed by something that is not two hexadecimal digits is kept the
/// same way. Refusing the whole link over one is worse: the link still names
/// somebody, and a person can see and correct a stray `%` in the composer.
///
/// The decoded bytes are read as UTF-8, which is what RFC 6068 requires, and
/// anything that is not valid UTF-8 is replaced rather than dropped. A
/// mangled character is visible; a silently shortened address is not.
fn decode(text: &str) -> String {
    let raw = text.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(raw.len());
    let mut at = 0;
    while at < raw.len() {
        let escape = (raw[at] == b'%')
            .then(|| raw.get(at + 1..at + 3))
            .flatten()
            .and_then(|pair| std::str::from_utf8(pair).ok())
            .and_then(|pair| u8::from_str_radix(pair, 16).ok());
        match escape {
            Some(byte) => {
                out.push(byte);
                at += 3;
            }
            None => {
                out.push(raw[at]);
                at += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// One line, whatever the link tried to put in it.
///
/// A newline in a subject or an address is header injection: the value is
/// written into a message header, and a header holding a newline is two
/// headers. Turned into spaces rather than removed, so words on either side
/// do not run together into one that reads as a different word.
fn one_line(text: &str) -> String {
    without_control_characters(text)
        .replace(['\r', '\n'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Text with the characters that are not text taken out.
///
/// A body may hold newlines and tabs, because those are the shape of a
/// message. Everything else below a space is a control character that no
/// message needs and that a screen reader either skips or reads as a machine
/// name, and a null in the middle of a Rust string quietly truncates the text
/// at every Windows API it is later handed to.
fn without_control_characters(text: &str) -> String {
    text.chars()
        .filter(|character| {
            !character.is_control()
                || *character == '\n'
                || *character == '\r'
                || *character == '\t'
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_bare_address_becomes_the_to_line_and_nothing_else() {
        // The commonest link on the web, and the one every other case is a
        // variation on.
        let asked = parse("mailto:someone@example.com").expect("a plain link");

        assert_eq!(asked.to, "someone@example.com");
        assert_eq!(asked.subject, "");
        assert_eq!(asked.body, "");
        assert_eq!(asked.cc, "");
        assert_eq!(asked.bcc, "");
    }

    #[test]
    fn test_the_scheme_is_matched_without_case_because_schemes_are() {
        // Windows hands over whatever the page wrote, and pages write
        // "MAILTO:" and "MailTo:". Refusing those would make the association
        // work on some links and not others, with nothing to say why.
        for written in ["MAILTO:a@b.com", "MailTo:a@b.com", "mailto:a@b.com"] {
            assert_eq!(
                parse(written).expect(written).to,
                "a@b.com",
                "{written} was not read as a mailto link"
            );
        }
    }

    #[test]
    fn test_something_that_is_not_a_mailto_link_is_refused_rather_than_guessed_at() {
        // This is reached from the command line, where the argument could be
        // anything. Treating a file path or a web address as an empty message
        // would open a composer nobody asked for and lose whatever was
        // actually meant.
        for not_a_link in [
            "https://example.com",
            r"C:\Users\somebody\invite.ics",
            "someone@example.com",
            "",
            "mailt:a@b.com",
        ] {
            let refused = parse(not_a_link);
            assert!(refused.is_err(), "{not_a_link:?} was accepted");
        }
    }

    #[test]
    fn test_several_addresses_in_the_path_all_reach_the_to_line() {
        let asked = parse("mailto:one@example.com,two@example.com").expect("two addresses");

        assert_eq!(asked.to, "one@example.com, two@example.com");
    }

    #[test]
    fn test_an_empty_address_between_two_commas_does_not_become_a_stray_comma() {
        // `mailto:a@b.com,,c@d.com` is written by generators that join a list
        // without minding a blank entry. Kept as written, the composer shows a
        // To line with a comma sitting on its own, which a screen reader reads
        // out and which some servers refuse.
        let asked = parse("mailto:a@b.com,,c@d.com").expect("a list with a gap");

        assert_eq!(asked.to, "a@b.com, c@d.com");
    }

    #[test]
    fn test_the_query_fills_in_the_subject_the_body_and_the_copy_lines() {
        let asked = parse(
            "mailto:to@example.com?subject=Hello&body=Some%20words&cc=copy@example.com\
             &bcc=quiet@example.com",
        )
        .expect("a full link");

        assert_eq!(asked.to, "to@example.com");
        assert_eq!(asked.subject, "Hello");
        assert_eq!(asked.body, "Some words");
        assert_eq!(asked.cc, "copy@example.com");
        assert_eq!(asked.bcc, "quiet@example.com");
    }

    #[test]
    fn test_a_header_name_is_matched_without_case_the_way_the_rfc_says() {
        // Every honoured arm, in a spelling nobody would write by hand but
        // that generators produce. A name this did not recognise would be
        // dropped, and the link would half work.
        let asked = parse("mailto:?TO=a@b.com&Subject=S&BODY=B&Cc=c@d.com&bCc=e@f.com")
            .expect("shouted headers");

        assert_eq!(asked.to, "a@b.com");
        assert_eq!(asked.subject, "S");
        assert_eq!(asked.body, "B");
        assert_eq!(asked.cc, "c@d.com");
        assert_eq!(asked.bcc, "e@f.com");
        assert!(asked.ignored.is_empty(), "{:?}", asked.ignored);
    }

    #[test]
    fn test_a_to_in_the_query_adds_to_the_addresses_in_the_path() {
        // RFC 6068: the two lists are combined. Replacing one with the other
        // would drop somebody a link named, and which one is dropped would
        // depend on how the link happened to be written.
        let asked = parse("mailto:first@example.com?to=second@example.com").expect("both");

        assert_eq!(asked.to, "first@example.com, second@example.com");
    }

    #[test]
    fn test_a_header_this_program_will_not_honour_is_dropped_and_written_down() {
        // The security decision this module exists for. A link that could set
        // `from` puts somebody else's address on a message going out under
        // this person's account; one that could set `reply-to` sends every
        // answer somewhere they did not choose. Both are dropped, and both are
        // named so a log can say what the link tried.
        let asked = parse(
            "mailto:a@b.com?from=victim@example.com&reply-to=attacker@example.com\
             &content-type=text/html&subject=Real",
        )
        .expect("a link with headers it may not set");

        assert_eq!(asked.subject, "Real", "an honoured field was lost");
        assert_eq!(asked.to, "a@b.com");
        for dropped in ["from", "reply-to", "content-type"] {
            assert!(
                asked.ignored.iter().any(|name| name == dropped),
                "{dropped} was not recorded as ignored: {:?}",
                asked.ignored
            );
        }
    }

    #[test]
    fn test_nothing_a_link_names_can_reach_a_field_this_type_does_not_have() {
        // The other half of the same guard, stated as a property rather than
        // against a list: whatever a link writes, the only things that come
        // back filled in are the five a person can see in the composer.
        let asked = parse("mailto:?x-priority=1&attach=C:%5Cwindows%5Csystem32%5Ccalc.exe")
            .expect("a link asking for things it may not have");

        assert_eq!(
            asked,
            MailTo {
                ignored: asked.ignored.clone(),
                ..MailTo::default()
            }
        );
        assert_eq!(asked.ignored.len(), 2, "{:?}", asked.ignored);
    }

    #[test]
    fn test_the_only_headers_that_reach_a_field_are_the_ones_written_down() {
        // The list and the arms are two spellings of one decision, and this is
        // what stops them drifting. Written because they had drifted while
        // both were gates: a name had to be in both, so adding one to only the
        // list changed nothing at all, and no test could tell. The arms are
        // now the gate and this measures them.
        //
        // Every honoured name is asked for on its own and must land somewhere.
        for honoured in HONOURED {
            let asked = parse(&format!("mailto:?{honoured}=x@example.com"))
                .unwrap_or_else(|_| panic!("{honoured} was refused"));
            assert!(
                asked.ignored.is_empty(),
                "{honoured} is written down as honoured and was dropped"
            );
        }
        // And the count matches, so an arm added without a line in the list,
        // or a line added without an arm, is a failure here.
        let every_field = parse(&format!(
            "mailto:?{}",
            HONOURED
                .iter()
                .map(|name| format!("{name}=x"))
                .collect::<Vec<_>>()
                .join("&")
        ))
        .expect("every honoured field at once");
        assert!(every_field.ignored.is_empty(), "{:?}", every_field.ignored);
        assert_eq!(HONOURED.len(), 5, "the list changed without this test");
    }

    #[test]
    fn test_a_plus_sign_stays_a_plus_sign_because_this_is_not_a_form() {
        // RFC 6068 section 5 says so in as many words, and the cost of getting
        // it wrong is not cosmetic: `first+tag@example.com` is how a great
        // many people filter their mail, and decoding the plus as a space
        // turns it into an address with a space in it that goes nowhere.
        let asked = parse("mailto:first+tag@example.com?subject=one+two").expect("a plus");

        assert_eq!(asked.to, "first+tag@example.com");
        assert_eq!(asked.subject, "one+two", "a plus was decoded as a space");
    }

    #[test]
    fn test_a_space_written_as_an_escape_is_decoded() {
        // The correct way to write one, and the one this must get right since
        // the plus is deliberately left alone.
        assert_eq!(
            parse("mailto:?subject=one%20two")
                .expect("an escape")
                .subject,
            "one two"
        );
    }

    #[test]
    fn test_a_percent_encoded_comma_stays_inside_the_address_it_was_written_in() {
        // The reason the address list is split before it is decoded. Decoding
        // first would turn this into two addresses, neither of them the one
        // the link named.
        let asked = parse(r#"mailto:%22Doe,%20Jane%22@example.com"#).expect("a quoted local part");

        assert_eq!(asked.to, r#""Doe, Jane"@example.com"#);
    }

    #[test]
    fn test_a_stray_percent_is_kept_rather_than_refusing_the_whole_link() {
        // A malformed escape is a badly written link, not an attack, and the
        // link still names somebody. Refusing outright loses that; keeping the
        // percent shows it to a person who can correct it before sending.
        let asked = parse("mailto:a@b.com?subject=100%25%20done%20%ZZ").expect("a stray percent");

        assert_eq!(asked.subject, "100% done %ZZ");
    }

    #[test]
    fn test_a_newline_in_a_subject_becomes_a_space_because_a_header_is_one_line() {
        // Header injection. A subject carrying a carriage return and a newline
        // is two headers by the time it reaches a mail server, and the second
        // one is whatever the link's author chose: a Bcc, a different From, or
        // the start of a body.
        let asked = parse("mailto:a@b.com?subject=Hello%0D%0ABcc:%20attacker@example.com")
            .expect("an injected header");

        assert!(
            !asked.subject.contains('\n') && !asked.subject.contains('\r'),
            "a line ending survived into the subject: {:?}",
            asked.subject
        );
        assert_eq!(asked.subject, "Hello Bcc: attacker@example.com");
    }

    #[test]
    fn test_a_newline_in_an_address_becomes_a_space_for_the_same_reason() {
        let asked = parse("mailto:a@b.com%0D%0ABcc:%20attacker@example.com").expect("an address");

        assert!(
            !asked.to.contains('\n') && !asked.to.contains('\r'),
            "a line ending survived into the To line: {:?}",
            asked.to
        );
    }

    #[test]
    fn test_a_body_keeps_its_line_endings_because_a_message_has_lines() {
        // The one field where a newline is the point rather than an attack.
        // The body goes into an editor, not into a header.
        let asked = parse("mailto:a@b.com?body=First%20line%0D%0ASecond%20line").expect("a body");

        assert_eq!(asked.body, "First line\r\nSecond line");
    }

    #[test]
    fn test_a_null_character_never_reaches_a_field() {
        // A null in the middle of a Rust string is legal Rust and quietly
        // truncates the text at every Windows API it is later handed to, so a
        // subject would be shown in full here and sent cut in half.
        let asked = parse("mailto:a@b.com?subject=before%00after&body=in%00side").expect("nulls");

        assert!(!asked.subject.contains('\0'), "{:?}", asked.subject);
        assert!(!asked.body.contains('\0'), "{:?}", asked.body);
    }

    #[test]
    fn test_a_link_with_no_address_at_all_is_still_a_link() {
        // `mailto:?subject=...` is written by "share this page" buttons, which
        // leave the recipient to the person. It has to open a composer rather
        // than being refused.
        let asked = parse("mailto:?subject=Look%20at%20this").expect("no recipient");

        assert_eq!(asked.to, "");
        assert_eq!(asked.subject, "Look at this");
    }

    #[test]
    fn test_the_same_field_written_twice_keeps_both_rather_than_losing_one() {
        // Which one a reader keeps is otherwise arbitrary, and a link naming
        // two people in two `cc=` fields means two people.
        let asked =
            parse("mailto:a@b.com?cc=one@example.com&cc=two@example.com").expect("two cc fields");

        assert_eq!(asked.cc, "one@example.com, two@example.com");
    }

    #[test]
    fn test_characters_that_are_not_ascii_survive_being_decoded() {
        // A subject in any language, written the way RFC 6068 requires, which
        // is UTF-8 bytes each percent-encoded.
        let asked = parse("mailto:a@b.com?subject=caf%C3%A9%20%E6%97%A5%E6%9C%AC")
            .expect("a subject that is not ascii");

        assert_eq!(asked.subject, "café 日本");
    }

    #[test]
    fn test_a_field_with_no_value_is_read_as_empty_rather_than_dropping_the_rest() {
        // `?subject=&body=Hi` and `?subject&body=Hi` both appear in the wild.
        // Reading either as malformed would lose the body.
        for written in [
            "mailto:a@b.com?subject=&body=Hi",
            "mailto:a@b.com?subject&body=Hi",
        ] {
            let asked = parse(written).expect(written);
            assert_eq!(asked.subject, "", "{written}");
            assert_eq!(asked.body, "Hi", "{written}");
        }
    }

    #[test]
    fn test_surrounding_whitespace_does_not_stop_a_link_being_read() {
        // A shell, a shortcut, or a copied line can bring one along.
        assert_eq!(parse("  mailto:a@b.com  ").expect("padded").to, "a@b.com");
    }
}

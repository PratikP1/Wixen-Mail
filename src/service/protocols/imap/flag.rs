//! The five flags a mail server keeps on a message, spelled once.
//!
//! These names were written out at every place that reads or writes one, and
//! they have to agree: the name sent to the server, the name the reply is
//! matched against, and the name the cache turns into a column. Nothing checked
//! that they did.
//!
//! That is the kind of mistake that does not fail. Star a message and the list
//! writes its own column, so the star appears and the row reads back starred.
//! The name that went to the server is a separate string, and a typo in it
//! means the server was never told. The person finds out from another device,
//! days later, when the star is not there and there is nothing to say why.
//!
//! Constants rather than a set of values, because the same calls also carry
//! keywords a provider invented, which no fixed list can hold.

/// The message has been read.
pub const SEEN: &str = "\\Seen";

/// The message is flagged for attention, which is what starring sets.
pub const FLAGGED: &str = "\\Flagged";

/// The message has been answered.
pub const ANSWERED: &str = "\\Answered";

/// The message is a draft.
pub const DRAFT: &str = "\\Draft";

/// The message is marked for removal.
pub const DELETED: &str = "\\Deleted";

/// The message is junk, in the keyword IANA registered and RFC 9051 names
/// (section 2.3.2). A keyword rather than a system flag, so it is only kept
/// where the folder says it keeps it; see [`keeps_keyword`].
pub const JUNK: &str = "$Junk";

/// The message is not junk. RFC 9051 says a message carrying this and
/// [`JUNK`] together is to be read as carrying neither, so the one is taken
/// off before the other is put on.
pub const NOT_JUNK: &str = "$NotJunk";

/// Whether a folder keeps this keyword, from the PERMANENTFLAGS it named when
/// it was opened.
///
/// RFC 9051's SELECT: a keyword is kept where the list names it, or where it
/// holds `\*`, which lets a client make keywords of its own. A server that sent
/// no list at all has said nothing about what it keeps, so nothing is claimed,
/// and a keyword sent into that may be gone when the session ends. Keywords are
/// matched without case, as IMAP matches them.
pub fn keeps_keyword(permanent: Option<&[String]>, keyword: &str) -> bool {
    permanent.is_some_and(|flags| {
        flags
            .iter()
            .any(|flag| flag == "\\*" || flag.eq_ignore_ascii_case(keyword))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_names_are_the_ones_the_protocol_uses() {
        // Written out a second time on purpose, and only here. A typo in one of
        // these changes what reaches every mail server at once, so changing one
        // should mean changing this line as well.
        assert_eq!(SEEN, "\\Seen");
        assert_eq!(FLAGGED, "\\Flagged");
        assert_eq!(ANSWERED, "\\Answered");
        assert_eq!(DRAFT, "\\Draft");
        assert_eq!(DELETED, "\\Deleted");
    }

    #[test]
    fn test_a_flag_name_is_what_both_readers_accept() {
        // The point of the one definition. A name that the writer sends and the
        // readers do not recognise is a change that never leaves this computer.
        let message = crate::service::protocols::imap::ImapMessage {
            flags: vec![
                FLAGGED.to_string(),
                SEEN.to_string(),
                ANSWERED.to_string(),
                DRAFT.to_string(),
                DELETED.to_string(),
            ],
            ..Default::default()
        };

        assert!(message.flagged());
        assert!(message.seen());
        assert!(message.answered());
        assert!(message.draft());
        assert!(message.deleted());
    }

    #[test]
    fn test_the_junk_keywords_are_the_ones_the_registry_names() {
        // Written out a second time for the reason the five above are: a typo
        // here is a keyword no server's filter has ever heard of.
        assert_eq!(JUNK, "$Junk");
        assert_eq!(NOT_JUNK, "$NotJunk");
    }

    fn listed(flags: &[&str]) -> Vec<String> {
        flags.iter().map(|flag| flag.to_string()).collect()
    }

    #[test]
    fn test_a_folder_keeps_a_new_keyword_where_it_says_star() {
        // RFC 9051, SELECT: "\*" in PERMANENTFLAGS means a client may create
        // a keyword of its own, and have it kept.
        let permanent = listed(&["\\Seen", "\\Deleted", "\\*"]);
        assert!(keeps_keyword(Some(&permanent), JUNK));
    }

    #[test]
    fn test_a_folder_keeps_a_keyword_it_names() {
        let permanent = listed(&["\\Seen", "$Junk", "$NotJunk"]);
        assert!(keeps_keyword(Some(&permanent), JUNK));
    }

    #[test]
    fn test_a_keyword_is_matched_without_case() {
        // Keywords are case-insensitive in IMAP, so a server naming "$junk"
        // keeps what this program sends as "$Junk".
        let permanent = listed(&["\\Seen", "$junk"]);
        assert!(keeps_keyword(Some(&permanent), JUNK));
    }

    #[test]
    fn test_a_folder_naming_neither_the_keyword_nor_star_keeps_nothing_new() {
        let permanent = listed(&["\\Answered", "\\Flagged", "\\Deleted", "\\Seen", "\\Draft"]);
        assert!(!keeps_keyword(Some(&permanent), JUNK));
    }

    #[test]
    fn test_the_other_junk_keyword_is_not_this_one() {
        let permanent = listed(&["\\Seen", "$NotJunk"]);
        assert!(!keeps_keyword(Some(&permanent), JUNK));
    }

    #[test]
    fn test_a_server_that_sent_no_permanent_flags_keeps_nothing_new() {
        // No PERMANENTFLAGS at all is the server saying nothing about what it
        // keeps, and a keyword sent into that silence may be dropped at the
        // end of the session. Nothing is claimed.
        assert!(!keeps_keyword(None, JUNK));
        assert!(!keeps_keyword(Some(&[]), JUNK));
    }
}

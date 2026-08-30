//! What a conversation is called, and whether a subject already says it is a
//! reply.
//!
//! Two questions with one answer between them, deliberately. A conversation is
//! named by the oldest message present with its reply and forward markers taken
//! off (D-04), and the compose box decides whether to write a marker on the way
//! out. Those are the same question asked from two ends, and this project has
//! been bitten before by two functions answering one question: the compose box
//! used to recognise the exact ASCII `"Re: "` and nothing else, so replying to
//! `AW: Angebot` produced `Re: AW: Angebot` and the chain grew every time.
//!
//! Neither answer is written here. `mail_parser` already implements RFC 5256's
//! base-subject algorithm with nineteen reply markers and twenty-two forward
//! markers across seventeen languages, and a list of markers kept here would be
//! wrong in a language nobody here reads.
//!
//! # This is a label rule, not a threading rule
//!
//! [`crate::application::threading`] argues at length against matching messages
//! by subject, and it is right: `Re: lunch` collides across years and strangers.
//! The stripped subject names a conversation somebody is already in. It never
//! decides which conversation a message belongs to; that is
//! [`crate::application::thread_identity::conversation_root`], from the
//! `References` chain.
//!
//! # The name moves
//!
//! D-04 names a conversation after the oldest message **present**, and present
//! is a moving target: `Get Older Messages` can bring in something earlier and
//! the name changes. So this takes the oldest subject as an argument every time
//! rather than remembering an answer against a thread id.

use mail_parser::parsers::fields::thread::{thread_name, trim_trailing_fwd};

/// What a conversation with nothing to be called is called.
///
/// The same words a message with no subject gets, from the one place that
/// already decides it, so a conversation and the message it holds are not given
/// two different names for the same emptiness.
pub use crate::application::from_message::NO_SUBJECT;

/// The conversation's own name, from the subject of the oldest message present.
///
/// Every reply and forward marker chain taken off, in any of the seventeen
/// languages `mail_parser` knows, so a conversation is called what it is about
/// rather than `Re: Fwd: Re:` followed by what it is about.
pub fn name_of(oldest_subject: &str) -> String {
    let base = thread_name(oldest_subject).trim();
    if base.is_empty() {
        NO_SUBJECT.to_string()
    } else {
        base.to_string()
    }
}

/// Whether this subject already opens with a reply marker.
///
/// Asked so the compose box can write `Re: ` exactly once. A subject that
/// already says it is a reply, in any language and any case, does not get a
/// second marker.
pub fn opens_with_a_reply_marker(subject: &str) -> bool {
    match opening_marker(subject) {
        // Every marker `thread_name` removes is either a reply marker or a
        // forward marker, and the two sets do not overlap, so a marker that is
        // not a forward is a reply. Asked this way round because the forward
        // set is the one that can be asked about directly.
        Some(marker) => !is_a_forward_marker(marker),
        None => false,
    }
}

/// Whether this subject already opens with a forward marker.
pub fn opens_with_a_forward_marker(subject: &str) -> bool {
    opening_marker(subject).is_some_and(is_a_forward_marker)
}

/// The token this subject opens with, when that token is a marker at all.
///
/// A marker is a word ending in a colon, optionally carrying the RFC's
/// bracketed count: `Re:`, `AW:`, `Re[2]:`, `fwd[5]:`. Finding where it ends is
/// reading the shape RFC 5256 defines; deciding whether the word inside it is a
/// marker is the part that needs a list of forty-one words in seventeen
/// languages, and that part is asked of `mail_parser` rather than answered
/// here.
///
/// `None` for a subject that opens with anything else, which includes a
/// bracketed list tag: `[mailing-list] hello` is not a reply, so replying to it
/// still writes `Re: `. `thread_name` takes that tag off as well, which is why
/// "would stripping change the subject" is the wrong question to ask here and
/// this one is asked instead.
fn opening_marker(subject: &str) -> Option<&str> {
    let (token, _) = subject.trim_start().split_once(':')?;
    let word = token.split('[').next().unwrap_or(token).trim();
    (!word.is_empty() && is_a_marker(word)).then_some(word)
}

/// Whether `mail_parser` reads this word as a marker of either kind.
///
/// Asked by handing it a subject made of that word and one other, and seeing
/// whether the other word comes back alone. That keeps the list of markers in
/// the dependency that maintains it.
fn is_a_marker(word: &str) -> bool {
    thread_name(&format!("{word}: {A_WORD_THAT_IS_NOT_A_MARKER}")) == A_WORD_THAT_IS_NOT_A_MARKER
}

/// Whether `mail_parser` reads this word as a forward marker specifically.
///
/// The forward set is the only one of the two that can be asked about on its
/// own: `trim_trailing_fwd` removes a trailing `(fwd)` for exactly the words in
/// it and leaves every other word alone.
///
/// One marker is read wrongly by this, and it is worth saying rather than
/// hiding: `trim_trailing_fwd` ignores a parenthesised word of a single
/// character, so Hungarian's `I:` is taken for a reply marker. The cost is that
/// forwarding a Hungarian forward writes `Fwd: I: ...` instead of leaving it
/// alone. Forty of the forty-one markers are read correctly, against one that
/// was read correctly before this change.
fn is_a_forward_marker(word: &str) -> bool {
    trim_trailing_fwd(&format!("{A_WORD_THAT_IS_NOT_A_MARKER} ({word})"))
        == A_WORD_THAT_IS_NOT_A_MARKER
}

/// The other word in both probes above.
///
/// Anything that is not itself a marker in any of the seventeen languages. A
/// probe built on a word that was one would answer yes about everything.
const A_WORD_THAT_IS_NOT_A_MARKER: &str = "qzqz";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_an_ordinary_subject_is_its_own_name() {
        // The ordinary case, first. A fixture of nothing but replies and
        // forwards cannot tell a working rule from one that returns whatever
        // it was handed.
        assert_eq!(name_of("Quarterly report"), "Quarterly report");
        assert_eq!(name_of("lunch"), "lunch");
        assert_eq!(name_of("Notes on the engine"), "Notes on the engine");
    }

    #[test]
    fn test_a_reply_marker_is_not_part_of_the_name() {
        assert_eq!(name_of("Re: Quarterly report"), "Quarterly report");
    }

    #[test]
    fn test_a_whole_chain_of_markers_comes_off() {
        assert_eq!(name_of("Re: Fwd: Re: Quarterly report"), "Quarterly report");
        assert_eq!(
            name_of("fwd[5]:re[5]: Quarterly report"),
            "Quarterly report"
        );
    }

    #[test]
    fn test_the_markers_other_languages_write_come_off_too() {
        // Seventeen languages is the whole reason this is taken from a
        // dependency rather than written here, so the test asks in more than
        // one of them: German, Swedish, Dutch, Italian, Polish and Chinese.
        assert_eq!(name_of("AW: Angebot"), "Angebot");
        assert_eq!(name_of("SV: Kvartalsrapport"), "Kvartalsrapport");
        assert_eq!(name_of("Antw: Vergadering"), "Vergadering");
        assert_eq!(name_of("R: Bilancio"), "Bilancio");
        assert_eq!(name_of("Odp: Spotkanie"), "Spotkanie");
        assert_eq!(name_of("WG: Angebot"), "Angebot");
        assert_eq!(name_of("回复: 轉寄: 報告"), "報告");
    }

    #[test]
    fn test_a_marker_in_any_case_comes_off() {
        assert_eq!(name_of("re: lunch"), "lunch");
        assert_eq!(name_of("RE: lunch"), "lunch");
        assert_eq!(name_of("aW: Angebot"), "Angebot");
    }

    #[test]
    fn test_a_subject_that_is_only_a_marker_gets_the_name_an_empty_one_gets() {
        // Not an empty label. A row reading nothing at all is a row somebody
        // arrows onto and hears silence from.
        assert_eq!(name_of("Re:"), NO_SUBJECT);
        assert_eq!(name_of("Fwd: Re: "), NO_SUBJECT);
        assert_eq!(name_of(""), NO_SUBJECT);
        assert_eq!(name_of("   "), NO_SUBJECT);
    }

    #[test]
    fn test_the_name_is_a_function_of_the_subject_it_is_handed_and_nothing_else() {
        // D-04: the oldest message present names the conversation, and present
        // moves. Nothing here may hold an answer against a conversation, or
        // the label would be pinned at first sight and an earlier message
        // arriving would change nothing.
        //
        // The same conversation, named twice, from two different oldest
        // messages. What `conversations_in` does with this is tested against a
        // real database in `message_cache::messages`.
        assert_eq!(name_of("Re: Quarterly report"), "Quarterly report");
        assert_eq!(name_of("Quarterly figures"), "Quarterly figures");
    }

    #[test]
    fn test_a_bracketed_list_tag_is_not_a_marker() {
        // A mailing list writes its name into every subject, so it says
        // nothing about which message in the conversation this is, and it is
        // not a reply marker either: replying to one still writes `Re: `,
        // which is the half `opens_with_a_reply_marker` answers.
        assert_eq!(name_of("[mailing-list] hello world"), "hello world");
        assert!(!opens_with_a_reply_marker("[mailing-list] hello world"));
        assert!(!opens_with_a_forward_marker("[mailing-list] hello world"));
    }

    #[test]
    fn test_a_subject_with_no_marker_opens_with_neither() {
        assert!(!opens_with_a_reply_marker("Quarterly report"));
        assert!(!opens_with_a_forward_marker("Quarterly report"));
        // A colon in the middle of a subject is not a marker either.
        assert!(!opens_with_a_reply_marker("Engine notes: the second pass"));
        assert!(!opens_with_a_forward_marker(
            "Engine notes: the second pass"
        ));
        // Nor is a word nobody replies with.
        assert!(!opens_with_a_reply_marker("Warning: low disk"));
        // Paired with the positive case in the same fixture, because a test
        // made only of "not this" passes against a function that says no to
        // everything, which is the emptiest thing it could do.
        assert!(opens_with_a_reply_marker("Re: Quarterly report"));
        assert!(opens_with_a_forward_marker("Fwd: Quarterly report"));
    }

    #[test]
    fn test_a_reply_marker_is_recognised_whatever_its_case_or_language() {
        for subject in [
            "Re: lunch",
            "re: lunch",
            "RE: lunch",
            "Re[2]: lunch",
            "AW: Angebot",
            "SV: Kvartalsrapport",
            "Antw: Vergadering",
            "Odp: Spotkanie",
        ] {
            assert!(
                opens_with_a_reply_marker(subject),
                "{subject:?} opens with a reply marker"
            );
            assert!(
                !opens_with_a_forward_marker(subject),
                "{subject:?} is a reply, not a forward"
            );
        }
    }

    #[test]
    fn test_a_forward_marker_is_recognised_whatever_its_case_or_language() {
        for subject in [
            "Fwd: lunch",
            "fwd: lunch",
            "FW: lunch",
            "fwd[5]: lunch",
            "WG: Angebot",
            "VS: Kvartalsrapport",
            "Doorst: Vergadering",
        ] {
            assert!(
                opens_with_a_forward_marker(subject),
                "{subject:?} opens with a forward marker"
            );
            assert!(
                !opens_with_a_reply_marker(subject),
                "{subject:?} is a forward, not a reply"
            );
        }
    }
}

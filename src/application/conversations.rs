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

/// What a conversation with nothing to be called is called.
///
/// The same words a message with no subject gets, from the one place that
/// already decides it, so a conversation and the message it holds are not given
/// two different names for the same emptiness.
pub use crate::application::from_message::NO_SUBJECT;

/// The conversation's own name, from the subject of the oldest message present.
pub fn name_of(_oldest_subject: &str) -> String {
    String::new()
}

/// Whether this subject already opens with a reply marker.
pub fn opens_with_a_reply_marker(_subject: &str) -> bool {
    false
}

/// Whether this subject already opens with a forward marker.
pub fn opens_with_a_forward_marker(_subject: &str) -> bool {
    false
}

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

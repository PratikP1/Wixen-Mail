//! Labelling a message, and which key does it.
//!
//! Tags could be made, named, coloured, edited and deleted, and none of that
//! ever reached a message: the table, the join table and the manager were all
//! there, nothing put a tag on anything, and nothing read one back. The third
//! feature in this application built up to its last step and left there, after
//! signatures and filter rules.
//!
//! # Why it is worth having
//!
//! It is the fastest triage gesture there is. Working through an inbox by ear
//! means deciding one thing per message, quickly, and a keystroke that says
//! "this one is work" without opening it, moving it or leaving the row is the
//! difference between sorting a hundred messages and giving up on them.
//!
//! # Which keys
//!
//! Thunderbird uses the bare number keys, `1` to `9`, with `0` to take them all
//! off. This uses Ctrl and the number instead, for a reason and not by
//! preference: a bare digit in a list is also a character, and a list that
//! jumps to what you type cannot tell "label this work" from somebody
//! spelling their way to a message about invoice 4021. The modifier makes the
//! two unambiguous.
//!
//! The names and colours are Thunderbird's, because somebody arriving from
//! there should find their own labels rather than have to rebuild them.
//!
//! # How a label travels
//!
//! As an IMAP keyword, which is what every other client reads. The five an
//! account starts with carry the keywords Thunderbird uses, so a message
//! labelled Work here is labelled Work there. A label somebody makes carries a
//! keyword built from the letters of its name, because a keyword is an atom and
//! cannot hold a space.
//!
//! The keyword is stored beside the name rather than worked out at send time.
//! Renaming a label must not change what it was already sent under: that would
//! leave the old keyword on every message on the server with nothing here
//! recognising it.
//!
//! A name with no letters or digits in it, like "!!!", has no keyword that
//! could be sent. That label works here and goes no further, which the log says
//! rather than pretending it was sent.
//!
//! Writing a keyword to the server is a write, so it is gated exactly like
//! every other change to a mailbox and does not happen at all until somebody
//! allows it.

/// A label somebody can put on a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Label {
    pub name: &'static str,
    /// What it looks like, for the list and for anyone who can see it.
    ///
    /// Never the only signal. Colour alone says nothing to most of the people
    /// this is for, so the name is what is announced and what a column shows.
    pub colour: &'static str,
    /// The IMAP keyword, which is what other clients read.
    ///
    /// A keyword is alphanumeric with no spaces, so "To Do" cannot be one and
    /// travels as `$todo`. The dollar prefix marks a keyword that is meant to
    /// be shared between clients rather than private to one.
    pub keyword: &'static str,
}

/// The labels an account starts with.
///
/// Thunderbird's five, in its order, so the number that applies each one is the
/// number somebody already knows.
pub const TO_BEGIN_WITH: [Label; 5] = [
    Label {
        name: "Important",
        colour: "#FF0000",
        keyword: "$label1",
    },
    Label {
        name: "Work",
        colour: "#FF9900",
        keyword: "$label2",
    },
    Label {
        name: "Personal",
        colour: "#009900",
        keyword: "$label3",
    },
    Label {
        name: "To Do",
        colour: "#3333FF",
        keyword: "$label4",
    },
    Label {
        name: "Later",
        colour: "#993399",
        keyword: "$label5",
    },
];

/// The keyword a label somebody made themselves travels as.
///
/// An IMAP keyword is an atom: no spaces and none of the characters the
/// protocol reserves. A label called "Follow up" cannot be sent as it is, so
/// its keyword is the letters and digits of its name with the rest taken out.
///
/// The name is kept beside it in the database, so this is a wire format rather
/// than a rename: "Follow up" is still called "Follow up" everywhere somebody
/// reads it.
///
/// `None` for a name with nothing usable in it. A label called "!!!" has no
/// keyword that could be sent, and inventing one would put a label on somebody's
/// mailbox under a name they never chose.
pub fn keyword_from(name: &str) -> Option<String> {
    let usable: String = name.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    if usable.is_empty() {
        return None;
    }
    // Not prefixed with a dollar. That prefix is for keywords with an agreed
    // meaning across clients, and a label somebody invented has none.
    Some(usable)
}

/// The keyword one of the starting labels travels as.
pub fn keyword_for(name: &str) -> Option<&'static str> {
    TO_BEGIN_WITH
        .iter()
        .find(|label| label.name.eq_ignore_ascii_case(name))
        .map(|label| label.keyword)
}

/// Whether a keyword is one this application would have written.
///
/// A mailbox carries keywords from every client that has touched it, including
/// ones this knows nothing about. Those are left alone rather than turned into
/// labels nobody made.
pub fn is_a_label_keyword(keyword: &str, known: &[String]) -> bool {
    known.iter().any(|held| held == keyword)
}

/// How many labels the number keys can reach.
///
/// Nine, because there are nine digits that are not zero and zero means "take
/// them all off". Somebody with more labels than this reaches the rest from the
/// menu, which is what every other client does too.
pub const REACHABLE_BY_KEY: usize = 9;

/// Which label a number key means, out of the ones this account has.
///
/// `None` for a number past the end, which is a key press worth answering with
/// "there is no ninth label" rather than with silence.
pub fn at_number<T>(labels: &[T], number: usize) -> Option<&T> {
    if number == 0 || number > REACHABLE_BY_KEY {
        return None;
    }
    labels.get(number - 1)
}

/// Whether pressing a number puts the label on or takes it off.
///
/// The same key both ways, because a label applied by mistake should come off
/// the way it went on. Anything else means learning a second key for undoing
/// the first.
pub fn turns_on(already_on: &[String], name: &str) -> bool {
    !already_on.iter().any(|held| held == name)
}

/// What to say when a label goes on or comes off.
///
/// Names the label and which way it went. "Tagged" alone leaves somebody who
/// pressed the wrong number with no idea what they just did, and a label is not
/// visible from the row it is on.
pub fn spoken(name: &str, now_on: bool) -> String {
    if now_on {
        format!("{name} added")
    } else {
        format!("{name} removed")
    }
}

/// What to say when there is no label on that number.
pub fn nothing_there(number: usize) -> String {
    format!("There is no label {number}")
}

/// What to say when every label comes off at once.
///
/// Counted, because "labels cleared" on a message that had none is a keystroke
/// that appears to have done something and did not.
pub fn all_removed(how_many: usize) -> String {
    match how_many {
        0 => "There were no labels on it".to_string(),
        1 => "1 label removed".to_string(),
        many => format!("{many} labels removed"),
    }
}

/// A message's labels, as a list column and as something to read out.
///
/// Empty for a message with none, so the column is blank rather than saying
/// "no labels" on every row of an untagged mailbox.
pub fn joined(names: &[String]) -> String {
    names.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> Vec<String> {
        TO_BEGIN_WITH
            .iter()
            .map(|label| label.name.to_string())
            .collect()
    }

    #[test]
    fn test_a_label_with_a_space_in_it_still_has_a_keyword() {
        // An IMAP keyword is an atom, so "Follow up" cannot be sent as it is.
        assert_eq!(keyword_from("Follow up").as_deref(), Some("Followup"));
        assert_eq!(keyword_from("Q4 / 2026").as_deref(), Some("Q42026"));
    }

    #[test]
    fn test_a_name_with_nothing_usable_in_it_has_no_keyword() {
        // Inventing one would put a label on somebody's mailbox under a name
        // they never chose.
        assert_eq!(keyword_from("!!!"), None);
        assert_eq!(keyword_from("   "), None);
    }

    #[test]
    fn test_the_starting_labels_keep_the_keywords_other_clients_know() {
        // These are the ones with an agreed meaning, so they carry the dollar
        // prefix and are not derived from their names.
        assert_eq!(keyword_for("Important"), Some("$label1"));
        assert_eq!(keyword_for("later"), Some("$label5"));
        assert_eq!(keyword_for("Follow up"), None);
    }

    #[test]
    fn test_a_keyword_from_another_client_is_not_turned_into_a_label() {
        // A mailbox carries keywords from everything that has touched it.
        let ours = vec!["$label1".to_string(), "Followup".to_string()];

        assert!(is_a_label_keyword("$label1", &ours));
        assert!(!is_a_label_keyword("$MailFlagBit0", &ours));
        assert!(!is_a_label_keyword("NonJunk", &ours));
    }

    #[test]
    fn test_the_first_number_means_the_first_label() {
        let labels = names();

        assert_eq!(at_number(&labels, 1), Some(&"Important".to_string()));
        assert_eq!(at_number(&labels, 5), Some(&"Later".to_string()));
    }

    #[test]
    fn test_a_number_past_the_end_is_answerable_rather_than_silent() {
        // A key that does nothing silently is indistinguishable from a key
        // that is broken.
        let labels = names();

        assert_eq!(at_number(&labels, 6), None);
        assert!(nothing_there(6).contains("no label 6"));
    }

    #[test]
    fn test_zero_is_not_a_label() {
        // It means take them all off, which is a different command.
        assert_eq!(at_number(&names(), 0), None);
    }

    #[test]
    fn test_nothing_past_nine_is_reachable_by_key() {
        // Ten labels, and the tenth has no digit to reach it.
        let many: Vec<String> = (1..=10).map(|n| format!("Label {n}")).collect();

        assert_eq!(at_number(&many, 9), Some(&"Label 9".to_string()));
        assert_eq!(at_number(&many, 10), None);
    }

    #[test]
    fn test_the_same_key_takes_a_label_off_again() {
        // Applied by mistake, removed the way it went on. A second key for
        // undoing the first is a second thing to learn.
        let on = vec!["Work".to_string()];

        assert!(!turns_on(&on, "Work"));
        assert!(turns_on(&on, "Personal"));
    }

    #[test]
    fn test_what_is_said_names_the_label_and_which_way_it_went() {
        // A label is not visible from the row it is on, and somebody who
        // pressed the wrong number has no other way to find out.
        assert_eq!(spoken("Work", true), "Work added");
        assert_eq!(spoken("Work", false), "Work removed");
    }

    #[test]
    fn test_clearing_a_message_with_no_labels_says_so() {
        // Otherwise the keystroke appears to have done something.
        assert_eq!(all_removed(0), "There were no labels on it");
        assert_eq!(all_removed(1), "1 label removed");
        assert_eq!(all_removed(3), "3 labels removed");
    }

    #[test]
    fn test_an_untagged_message_has_an_empty_column_rather_than_a_word() {
        assert_eq!(joined(&[]), "");
        assert_eq!(
            joined(&["Work".to_string(), "Later".to_string()]),
            "Work, Later"
        );
    }

    #[test]
    fn test_every_starting_label_travels_as_a_keyword_other_clients_read() {
        // A keyword is alphanumeric with no spaces, so "To Do" cannot be one.
        // Shared rather than private, so a message labelled here is labelled
        // in Thunderbird too.
        for label in TO_BEGIN_WITH {
            assert!(label.keyword.starts_with('$'), "{}", label.keyword);
            assert!(
                label.keyword[1..]
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric()),
                "{} is not a usable keyword",
                label.keyword
            );
        }
    }

    #[test]
    fn test_no_starting_label_is_told_apart_by_colour_alone() {
        // Colour says nothing to most of the people this is for. Every label
        // has a name, and the name is what is announced.
        for label in TO_BEGIN_WITH {
            assert!(!label.name.trim().is_empty());
        }
        let mut names: Vec<&str> = TO_BEGIN_WITH.iter().map(|l| l.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), TO_BEGIN_WITH.len(), "two labels share a name");
    }
}

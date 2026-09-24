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

/// Which label a number means, out of the ones this account has, in the
/// order the Label menu shows them.
///
/// The number a menu line carries, key or not: the first nine are reached by
/// Ctrl and their digit as well as from the menu, and a tenth is reached from
/// the menu alone. `None` for a number past the end, which is a key press
/// worth answering with "there is no ninth label" rather than with silence.
pub fn at_number<T>(labels: &[T], number: usize) -> Option<&T> {
    labels.get(number.checked_sub(1)?)
}

/// One line of the Label submenu: where it sits, and what it says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuLine {
    /// Its place in the account's order, counted from one, which is also the
    /// number its key carries while there is a key.
    pub position: usize,
    /// The item's text, with the key after a tab where it has one.
    pub text: String,
}

/// What the Label submenu says, one line per label in the account's order.
///
/// Read from the account's own labels, the same list and the same order
/// [`at_number`] reads when a key is pressed, so the key beside a name is the
/// key that applies it. Until 2026-09-24 the menu was built once from
/// [`TO_BEGIN_WITH`] while the keys read the stored labels by name, and Ctrl+2
/// said Work and applied Later (#48).
///
/// An account with no labels yet is offered the five it starts with, because
/// that is what the first press of any of these keys makes, in this order.
/// A lone ampersand in a name is doubled, since a menu reads one as the mark
/// before an access letter.
pub fn what_the_menu_says(names: &[String]) -> Vec<MenuLine> {
    let starting: Vec<String> = TO_BEGIN_WITH
        .iter()
        .map(|label| label.name.to_string())
        .collect();
    let names = if names.is_empty() {
        &starting[..]
    } else {
        names
    };
    names
        .iter()
        .enumerate()
        .map(|(at, name)| {
            let position = at + 1;
            let shown = name.replace('&', "&&");
            let text = match key_for(position) {
                Some(key) => format!("{shown}\t{key}"),
                None => shown,
            };
            MenuLine { position, text }
        })
        .collect()
}

/// The key that applies the label at this place in the order, if it has one.
///
/// The menu writes it beside the name and the Label Manager's Key column
/// shows it, from this one answer. `None` past the ninth.
pub fn key_for(position: usize) -> Option<String> {
    (1..=REACHABLE_BY_KEY)
        .contains(&position)
        .then(|| format!("Ctrl+{position}"))
}

/// What is said when the cursor is on no label in the Label Manager.
pub const WHICH_LABEL: &str = "Choose a label first. Move Up and Move Down act on the row \
                               the cursor is on.";

/// Move one label up or down the account's order.
///
/// `labels` is every label as `(id, name)` in the order they sit in now. The
/// gesture and its wording are the ones accounts and pinned folders use, so a
/// move is said the same way whatever moved.
pub fn moved(labels: &[(String, String)], which: &str, direction: Move) -> Moved {
    crate::application::reordering::moved(labels, which, direction, WHICH_LABEL)
}

pub use crate::application::reordering::{Move, Moved};

/// Whether pressing a number puts the label on or takes it off.
///
/// The same key both ways, because a label applied by mistake should come off
/// the way it went on. Anything else means learning a second key for undoing
/// the first.
pub fn turns_on(already_on: &[String], name: &str) -> bool {
    !already_on.iter().any(|held| held == name)
}

/// What to say when there is no label on that number.
///
/// What is said when a label goes on or comes off, or every label comes
/// off, is `choosing_messages::what_was_done` since 2026-09-19 (#30), which
/// names the label and counts the messages; the two sentences that lived
/// here for one message went with the arm that said them.
pub fn nothing_there(number: usize) -> String {
    format!("There is no label {number}")
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
    fn test_nothing_past_nine_has_a_key() {
        // Ten labels, and the tenth has no digit to reach it, so its line on
        // the menu carries no key while the ninth's does.
        let many: Vec<String> = (1..=10).map(|n| format!("Label {n}")).collect();

        let lines = what_the_menu_says(&many);

        assert_eq!(lines[8].text, "Label 9\tCtrl+9");
        assert_eq!(lines[9].text, "Label 10");
    }

    #[test]
    fn test_the_tenth_label_is_reached_from_the_menu() {
        // The menu is how a label past the ninth is put on, and the number a
        // menu line carries is its place in the order, key or not.
        let many: Vec<String> = (1..=10).map(|n| format!("Label {n}")).collect();

        assert_eq!(at_number(&many, 10), Some(&"Label 10".to_string()));
        assert_eq!(at_number(&many, 11), None);
    }

    #[test]
    fn test_the_menu_says_each_label_in_its_order_with_its_key() {
        // The order the account keeps, not the order an account starts with:
        // the key beside a name is the key that applies it (#48).
        let held: Vec<String> = ["Later", "Important", "Invoices"]
            .iter()
            .map(|name| name.to_string())
            .collect();

        assert_eq!(
            what_the_menu_says(&held),
            vec![
                MenuLine {
                    position: 1,
                    text: "Later\tCtrl+1".to_string(),
                },
                MenuLine {
                    position: 2,
                    text: "Important\tCtrl+2".to_string(),
                },
                MenuLine {
                    position: 3,
                    text: "Invoices\tCtrl+3".to_string(),
                },
            ]
        );
    }

    #[test]
    fn test_an_account_with_no_labels_is_offered_the_five_the_first_key_makes() {
        // The first press of a key makes the five an account starts with, in
        // this order, so a menu read before then says what that press will
        // find rather than being empty and leaving every key unbound.
        let lines = what_the_menu_says(&[]);

        let texts: Vec<&str> = lines.iter().map(|line| line.text.as_str()).collect();
        assert_eq!(
            texts,
            [
                "Important\tCtrl+1",
                "Work\tCtrl+2",
                "Personal\tCtrl+3",
                "To Do\tCtrl+4",
                "Later\tCtrl+5",
            ]
        );
    }

    #[test]
    fn test_an_ampersand_in_a_labels_name_is_shown_and_claims_no_letter() {
        // A menu reads a lone ampersand as the mark before an access letter,
        // so "R&D" would show as "RD" and take D from the items beside it.
        // Doubled, it is shown as itself.
        let lines = what_the_menu_says(&["R&D".to_string()]);

        assert_eq!(lines[0].text, "R&&D\tCtrl+1");
    }

    fn five_as_stored() -> Vec<(String, String)> {
        ["Important", "Work", "Personal", "To Do", "Later"]
            .iter()
            .map(|name| (format!("acct:{name}"), name.to_string()))
            .collect()
    }

    #[test]
    fn test_moving_a_label_says_where_it_is_now() {
        // The gesture accounts and pinned folders already use, worded the way
        // they word it, and the whole order handed back to be written.
        let after = moved(&five_as_stored(), "acct:Work", Move::Down);

        assert_eq!(after.say, "Work, 3 of 5.");
        assert!(after.moved);
        assert_eq!(
            after.order,
            [
                "acct:Important",
                "acct:Personal",
                "acct:Work",
                "acct:To Do",
                "acct:Later",
            ]
        );
    }

    #[test]
    fn test_the_first_label_does_not_move_up_and_says_so() {
        let after = moved(&five_as_stored(), "acct:Important", Move::Up);

        assert_eq!(after.say, "Important is already first of 5.");
        assert!(!after.moved);
    }

    #[test]
    fn test_moving_with_no_label_chosen_says_to_choose_one() {
        let after = moved(&five_as_stored(), "", Move::Down);

        assert_eq!(after.say, WHICH_LABEL);
        assert!(!after.moved);
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

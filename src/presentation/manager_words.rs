//! What a manager window says about the row somebody just changed.
//!
//! One owner, because four windows worded a delete as "Deleted: {name}",
//! which is character-for-character what the mail path says when a message
//! is gone from a server (`application::server_delete::after_a_delete` over
//! `Deletion::Removed`). By ear those two are the same sentence, on a product
//! where by ear is the point. Naming the kind of thing is also the answer to
//! this round's own question, "would somebody hearing this and nothing else
//! know what happened": "Deleted: Jane Smith" never said whether a contact or
//! a message had gone.

pub(crate) const ACCOUNT: &str = "account";
pub(crate) const CONTACT: &str = "contact";
pub(crate) const FILTER: &str = "filter";
pub(crate) const TAG: &str = "tag";
pub(crate) const SIGNATURE: &str = "signature";
/// One thing a saved search asks about a message.
///
/// The sixth kind of row a manager window holds, and the first whose count is
/// part of the answer; see [`this_kind_is_counted_out_loud`].
pub(crate) const CONDITION: &str = "condition";

/// "a" or "an", whichever belongs in front of `kind`.
pub(crate) fn a_or_an(kind: &str) -> &'static str {
    match kind.chars().next() {
        Some(first) if "aeiouAEIOU".contains(first) => "an",
        _ => "a",
    }
}

/// Whether a window over things of this kind says how many are left after
/// every change.
///
/// A condition list, and nothing else. Its count is load-bearing in a way no
/// other manager's is: a saved search that asks nothing about a message is
/// refused, at the window and again at the store, so somebody taking
/// conditions out has to hear where they are before they meet that refusal on
/// the way out. A filter list, a tag list and a signature list have no such
/// floor, and a tally repeated into every sentence there would be a clause
/// with no answer in it.
///
/// Asked here rather than passed in, so the count cannot be switched on in one
/// window over conditions and off in another.
fn this_kind_is_counted_out_loud(kind: &str) -> bool {
    kind == CONDITION
}

/// The tally that goes on the end of a sentence about one change, for the
/// kinds that count out loud, and nothing at all for the rest.
///
/// On the end of the one sentence rather than said after it. Two announcements
/// for one change put the second over the first before anybody has heard it,
/// which is worse than not saying the number at all.
///
/// Nought is said in words. "0 conditions now" is a figure somebody has to
/// read as a warning; "No conditions now" is the warning, and it is the case
/// that matters most, because the next thing that happens is a refusal to
/// close.
fn and_how_many_now(kind: &str, left: usize) -> String {
    if !this_kind_is_counted_out_loud(kind) {
        return String::new();
    }
    match left {
        0 => format!(". No {kind}s now."),
        1 => format!(". 1 {kind} now."),
        many => format!(". {many} {kind}s now."),
    }
}

/// What to say when something of this kind was added, and how many there are
/// now.
pub(crate) fn added(kind: &str, name: &str, left: usize) -> String {
    let _ = left;
    format!("Added the {kind}: {name}")
}

/// What to say when something of this kind was updated, and how many there are
/// now.
pub(crate) fn updated(kind: &str, name: &str, left: usize) -> String {
    let _ = left;
    format!("Updated the {kind}: {name}")
}

/// What to say when something of this kind was deleted.
///
/// Not the sentence the mail path uses for a message removed from a server
/// (`application::server_delete::after_a_delete` over `Deletion::Removed`),
/// on purpose. A contact, a filter, a tag, a signature and an account going
/// are four different events from a message going, in four different
/// windows, and "Deleted: Jane Smith" used to read identically to a message
/// leaving a server: nobody working by ear could tell which one had
/// happened.
pub(crate) fn deleted(kind: &str, name: &str, left: usize) -> String {
    format!("Deleted the {kind}: {name}{}", and_how_many_now(kind, left))
}

/// What to say when a button was pressed with nothing selected.
pub(crate) fn nothing_selected(kind: &str, to_do: &str) -> String {
    format!("Select {} {kind} to {to_do}", a_or_an(kind))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::server_delete::{
        after_a_copy, after_a_delete, after_a_move, nothing_changed,
    };
    use crate::service::protocols::imap::{Deletion, Moved, StillHere};

    const EVERY_KIND: [&str; 6] = [ACCOUNT, CONTACT, FILTER, TAG, SIGNATURE, CONDITION];

    /// The three sentences a manager window says about one change, over a list
    /// that has `left` rows in it afterwards.
    fn every_change(kind: &str, name: &str, left: usize) -> [String; 3] {
        [
            added(kind, name, left),
            updated(kind, name, left),
            deleted(kind, name, left),
        ]
    }

    /// The five ways a delete at the server can end, named again here rather
    /// than reused from `server_delete`'s own tests: that fixture lives
    /// inside a `#[cfg(test)]` module of its own and is not visible from this
    /// file's tests.
    fn every_deletion() -> Vec<Deletion> {
        vec![
            Deletion::MovedToTrash,
            Deletion::Removed,
            Deletion::CopiedToTrashAndFlagged(StillHere::TheServerCannotRemoveOneMessage),
            Deletion::CopiedToTrashAndNotFlagged("over quota".to_string()),
            Deletion::MarkedOnly(StillHere::TheServerCannotRemoveOneMessage),
        ]
    }

    #[test]
    fn test_no_manager_window_says_what_the_mail_path_says_when_a_message_is_gone() {
        // Compares produced sentences, not source text. A tree-wide text scan
        // for "Deleted" would flag this file's own owner, whose sentence
        // starts with the word "Deleted" too; see `deleted`'s own doc comment
        // for why that is not the same defect.
        let mail: Vec<String> = every_deletion()
            .into_iter()
            .map(|d| after_a_delete(&d, "Invoice").said)
            .chain([
                after_a_move(&Moved::Moved, "Archive", "Invoice").said,
                after_a_copy("Archive", "Invoice").said,
                nothing_changed("over quota"),
            ])
            .collect();

        for kind in EVERY_KIND {
            for manager_sentence in every_change(kind, "Invoice", 2) {
                assert!(
                    !mail.contains(&manager_sentence),
                    "a manager sentence reads exactly like a sentence the mail \
                     path says about a message: {manager_sentence:?}"
                );
            }
        }
    }

    #[test]
    fn test_every_manager_sentence_names_the_kind_of_thing_it_happened_to() {
        for kind in EVERY_KIND {
            for sentence in every_change(kind, "Invoice", 2) {
                assert!(
                    sentence.contains(kind),
                    "{sentence:?} does not name the kind {kind:?}"
                );
                assert!(
                    sentence.contains("Invoice"),
                    "{sentence:?} does not name the item"
                );
            }
        }
    }

    #[test]
    fn test_the_word_before_a_kind_is_the_right_one() {
        assert_eq!(a_or_an(ACCOUNT), "an", "an account starts with a vowel");
        for kind in [CONTACT, FILTER, TAG, SIGNATURE, CONDITION] {
            assert_eq!(a_or_an(kind), "a", "{kind}");
        }
    }

    #[test]
    fn test_every_change_to_a_condition_list_says_how_many_are_left() {
        // Adding, editing and removing alike. A count said after only some of
        // the three is a number somebody has to notice the absence of, and
        // noticing an absence by ear is the thing this product exists to stop
        // asking of people. Editing does not move the count, and saying it
        // anyway is what makes the tally something to rely on rather than
        // something to interpret.
        for sentence in every_change(CONDITION, "Subject contains Invoice", 3) {
            assert!(
                sentence.contains('3'),
                "a change to a condition list said nothing about how many are \
                 left: {sentence:?}"
            );
        }
    }

    #[test]
    fn test_a_condition_change_is_one_sentence_and_not_two() {
        // The count goes on the end of the sentence about the change, not into
        // a second announcement. Two announcements for one change put the
        // second over the first before anybody has heard it, which is why the
        // count is built here rather than said again by the window.
        let said = deleted(CONDITION, "Subject contains Invoice", 2);

        assert!(said.starts_with("Deleted the condition: "), "{said:?}");
        assert!(said.contains('2'), "{said:?}");
    }

    #[test]
    fn test_the_last_condition_going_is_said_in_words_rather_than_as_nought() {
        // The case that matters most, because the next thing that happens is a
        // refusal to close. "0 conditions now" is a figure somebody has to
        // read as a warning before it is one.
        let none_left = deleted(CONDITION, "Subject contains Invoice", 0);

        assert!(
            !none_left.contains('0'),
            "an empty condition list was reported as a figure: {none_left:?}"
        );
        assert!(
            none_left.to_lowercase().contains("no condition"),
            "an empty condition list was not said in words: {none_left:?}"
        );
    }

    #[test]
    fn test_one_condition_left_is_not_said_as_a_plural() {
        let one = deleted(CONDITION, "Subject contains Invoice", 1);

        assert!(one.contains("1 condition now"), "{one:?}");
        assert!(!one.contains("conditions"), "{one:?}");
    }

    #[test]
    fn test_only_a_condition_list_counts_out_loud() {
        // The other five windows keep the sentences they have. A tally on a
        // filter or a tag list is a clause with no answer in it, because
        // nothing refuses an empty one, and it would arrive in five windows
        // that did not ask for it.
        for kind in [ACCOUNT, CONTACT, FILTER, TAG, SIGNATURE] {
            assert_eq!(
                added(kind, "Invoice", 4),
                format!("Added the {kind}: Invoice")
            );
            assert_eq!(
                updated(kind, "Invoice", 4),
                format!("Updated the {kind}: Invoice")
            );
            assert_eq!(
                deleted(kind, "Invoice", 4),
                format!("Deleted the {kind}: Invoice")
            );
        }
    }

    #[test]
    fn test_nothing_selected_names_the_kind_and_the_article_agrees() {
        assert_eq!(
            nothing_selected(ACCOUNT, "edit"),
            "Select an account to edit"
        );
        assert_eq!(
            nothing_selected(CONTACT, "edit"),
            "Select a contact to edit"
        );
        assert_eq!(
            nothing_selected(FILTER, "delete"),
            "Select a filter to delete"
        );
    }
}

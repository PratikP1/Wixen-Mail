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

/// "a" or "an", whichever belongs in front of `kind`.
pub(crate) fn a_or_an(kind: &str) -> &'static str {
    match kind.chars().next() {
        Some(first) if "aeiouAEIOU".contains(first) => "an",
        _ => "a",
    }
}

/// What to say when something of this kind was added.
pub(crate) fn added(kind: &str, name: &str) -> String {
    format!("Added the {kind}: {name}")
}

/// What to say when something of this kind was updated.
pub(crate) fn updated(kind: &str, name: &str) -> String {
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
pub(crate) fn deleted(kind: &str, name: &str) -> String {
    format!("Deleted the {kind}: {name}")
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

    const EVERY_KIND: [&str; 5] = [ACCOUNT, CONTACT, FILTER, TAG, SIGNATURE];

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
            for manager_sentence in [
                added(kind, "Invoice"),
                updated(kind, "Invoice"),
                deleted(kind, "Invoice"),
            ] {
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
            for sentence in [
                added(kind, "Invoice"),
                updated(kind, "Invoice"),
                deleted(kind, "Invoice"),
            ] {
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
        for kind in [CONTACT, FILTER, TAG, SIGNATURE] {
            assert_eq!(a_or_an(kind), "a", "{kind}");
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

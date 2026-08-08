//! What importing a file of contact cards did, in the words somebody hears.
//!
//! Named here rather than built where it is spoken, so it can be argued about
//! in a test. Reading a card file is the one place in this application where a
//! file somebody chose is turned into contacts, and until now the only thing
//! said about it was how many arrived. A folder whose cards were all turned
//! away and a folder with no card files in it both came back as "Imported 0
//! contacts", and nought with no reason beside it is what sends somebody
//! looking for a broken program.

use crate::application::summing_up::SummingUp;
use crate::data::message_cache::CardsRead;

/// What to say when a card file is chosen and no account is.
///
/// A contact belongs to an account. That account's list shows it, that
/// account's address books hold it, and that account's sync sends it. Import
/// and Export used to name a fixed word instead of the account being looked
/// at, so with any account signed in a file was read into a corner of the
/// database no panel and no sync ever looks at: the count said contacts had
/// arrived, the list stayed as it was, and nothing was ever sent anywhere.
/// Export had the matching half and wrote out an empty file.
pub const CHOOSE_AN_ACCOUNT_FIRST: &str =
    "Choose an account first. Contacts belong to the account whose address book holds them.";

/// What an import put into the address book and what it left out.
///
/// The counts that are not zero are the ones worth saying. A card turned away
/// gets a whole sentence rather than another count, because it names a rule
/// somebody has to act on: the card is still in the file and the contact is
/// still not here.
pub fn what_the_card_import_did(read: &CardsRead) -> String {
    let mut said = SummingUp::opening(match read.added {
        0 => "No contacts were imported".to_string(),
        1 => "Imported 1 contact".to_string(),
        many => format!("Imported {many} contacts"),
    });
    if read.with_no_email_address > 0 {
        // Two sentences written out rather than one built from parts: three
        // words have to agree in number and a sentence assembled from
        // fragments reads like one.
        said.sentence(if read.with_no_email_address == 1 {
            "1 card named no email address and was left out, because a contact \
             here needs an address to write to"
                .to_string()
        } else {
            format!(
                "{} cards named no email address and were left out, because a \
                 contact here needs an address to write to",
                read.with_no_email_address
            )
        });
    }
    if read.not_written_down > 0 {
        said.sentence(if read.not_written_down == 1 {
            "1 contact was read from the file and could not be saved on this computer".to_string()
        } else {
            format!(
                "{} contacts were read from the file and could not be saved on \
                 this computer",
                read.not_written_down
            )
        });
    }
    if read.waiting_to_be_sent > 0 {
        // The part of an import that leaves this computer, said at the moment
        // it is set going rather than in the sync summary afterwards.
        // Somebody importing an old backup to look through it is entitled to
        // know their real address book is about to hold all of it.
        said.sentence(if read.waiting_to_be_sent == 1 {
            "1 contact is waiting to be sent to your address book".to_string()
        } else {
            format!(
                "{} contacts are waiting to be sent to your address book",
                read.waiting_to_be_sent
            )
        });
    }
    said.spoken()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_an_import_that_added_nothing_and_turned_nothing_away_says_only_that() {
        // A folder with no card files in it. Nothing went wrong and nothing is
        // owed an explanation.
        assert_eq!(
            what_the_card_import_did(&CardsRead::default()),
            "No contacts were imported"
        );
    }

    #[test]
    fn test_an_import_that_turned_every_card_away_says_how_many_and_why() {
        // The reason this module exists. Before it, an address book of two
        // hundred contacts kept without email addresses imported as "Imported
        // 0 contacts" and there was nothing to act on.
        let said = what_the_card_import_did(&CardsRead {
            with_no_email_address: 3,
            ..CardsRead::default()
        });

        assert_eq!(
            said,
            "No contacts were imported. 3 cards named no email address and were \
             left out, because a contact here needs an address to write to."
        );
    }

    #[test]
    fn test_one_card_turned_away_is_said_in_the_singular() {
        // Read aloud, "1 cards named no email address" is the sentence that
        // was worth interrupting somebody for arriving broken.
        let said = what_the_card_import_did(&CardsRead {
            added: 4,
            with_no_email_address: 1,
            ..CardsRead::default()
        });

        assert_eq!(
            said,
            "Imported 4 contacts. 1 card named no email address and was left \
             out, because a contact here needs an address to write to."
        );
    }

    #[test]
    fn test_one_contact_imported_is_said_in_the_singular() {
        assert_eq!(
            what_the_card_import_did(&CardsRead {
                added: 1,
                ..CardsRead::default()
            }),
            "Imported 1 contact"
        );
    }

    #[test]
    fn test_a_contact_that_could_not_be_saved_is_said_rather_than_left_to_the_log() {
        let said = what_the_card_import_did(&CardsRead {
            added: 2,
            not_written_down: 1,
            ..CardsRead::default()
        });

        assert_eq!(
            said,
            "Imported 2 contacts. 1 contact was read from the file and could not \
             be saved on this computer."
        );
    }

    #[test]
    fn test_both_things_that_can_go_wrong_are_said_and_neither_runs_into_the_other() {
        // The punctuation is `summing_up`'s job and this is where it is asked
        // for: two sentences in a row used to arrive as "write to.. 2
        // contacts", which is a stutter and then a fragment.
        let said = what_the_card_import_did(&CardsRead {
            added: 5,
            with_no_email_address: 2,
            not_written_down: 2,
            waiting_to_be_sent: 0,
        });

        assert_eq!(
            said,
            "Imported 5 contacts. 2 cards named no email address and were left \
             out, because a contact here needs an address to write to. 2 \
             contacts were read from the file and could not be saved on this \
             computer."
        );
    }

    #[test]
    fn test_an_import_says_how_many_contacts_are_on_their_way_to_the_address_book() {
        // The part of an import that leaves this computer. Said here rather
        // than in the sync summary afterwards, because afterwards is too late
        // to decide against it.
        let said = what_the_card_import_did(&CardsRead {
            added: 40,
            waiting_to_be_sent: 40,
            ..CardsRead::default()
        });

        assert_eq!(
            said,
            "Imported 40 contacts. 40 contacts are waiting to be sent to your \
             address book."
        );
    }

    #[test]
    fn test_one_contact_on_its_way_to_the_address_book_is_said_in_the_singular() {
        let said = what_the_card_import_did(&CardsRead {
            added: 1,
            waiting_to_be_sent: 1,
            ..CardsRead::default()
        });

        assert_eq!(
            said,
            "Imported 1 contact. 1 contact is waiting to be sent to your address book."
        );
    }

    #[test]
    fn test_an_import_that_sends_nothing_onward_says_nothing_about_sending() {
        // An import that changed nobody queues nothing, and a sentence about
        // nought contacts on their way is a sentence that teaches people to
        // stop listening to this one.
        let said = what_the_card_import_did(&CardsRead {
            added: 3,
            ..CardsRead::default()
        });

        assert_eq!(said, "Imported 3 contacts");
    }
}

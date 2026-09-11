//! An address book somebody added by typing the address it lives at.
//!
//! A CalDAV calendar keeps its address, its change marker and the id its
//! sign-in is filed under on the calendars row. A CardDAV address book had
//! nowhere at all: `contact_identities` records what one address book calls one
//! contact, which is the contact's side of the relationship, and no table
//! anywhere held the address book itself. So this is the table, and it is
//! additive: `CREATE TABLE IF NOT EXISTS`, no column dropped and none renamed.
//!
//! # The word an address book's contacts are filed under has to be per book
//!
//! Every question the contact merge asks is keyed on the word
//! `AddressBook::as_stored` returns. `this_address_book_is_still_owed_the_change`
//! compares it, and so do `ContactEntry::id_in`, `told` and `no_longer_in`. Two
//! address books both filed under the bare word `carddav` are therefore one
//! address book as far as all of those are concerned: a change waiting for one
//! reads as waiting for the other, and a contact in both carries one identity
//! where it needs two. That is the failure `ProviderIdentity`'s own comment
//! records, from when the flag was kept on the contact rather than on the
//! identity: a failure at one address book either lost the change at the other
//! or resent it to both for ever.
//!
//! So the word is built from the address book's own id, which is per book by
//! construction. [`AddressBookContainer::the_word_its_contacts_are_filed_under`]
//! is the one place that says so. It begins with `carddav-` so that somebody
//! reading a contact's row in the database can tell what kind of address book
//! filed it.
//!
//! # Nothing here holds a sign-in
//!
//! The user name and the password go to the operating system's credential
//! store, under [`crate::service::carddav::keyring_service`]. The database is
//! copied with a profile and restored from a backup, so a password travelling
//! in it is a password on somebody else's disk. A test below stores a sign-in
//! through the real path and then reads every column of the row back to check
//! that none of them holds it.

use super::{AddressBook, MessageCache};
use crate::common::Result;

/// One CardDAV address book somebody added by its own address.
///
/// Five columns and every one of them is read. The address is what the sync
/// asks; the change marker is what it compares before asking again; the id is
/// both the credential store owner and the word contacts are filed under; the
/// account says whose contacts these are; and the name is what a person is told
/// they added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressBookContainer {
    /// This address book and no other, on this computer.
    ///
    /// The credential store entry holding its sign-in is named from this, and
    /// so is the word its contacts are filed under, so it is permanent for the
    /// life of the row.
    pub id: String,
    /// The account it is kept with. Only an account's address books are synced.
    pub account_id: String,
    /// What to call it, as the server named it.
    pub name: String,
    /// Where it lives, whole, so it can be asked without being rebuilt.
    pub url: String,
    /// The marker the server moves when anything in the address book changes,
    /// as it stood at the end of the last sync. Nothing before the first one,
    /// and nothing from a server that gives none.
    pub ctag: Option<String>,
}

impl AddressBookContainer {
    /// A new address book, with an id no other row will be given.
    ///
    /// Time based rather than random, so address books added in one session
    /// sort in the order they were added.
    pub fn new(account_id: &str, name: &str, url: &str) -> Self {
        Self {
            id: an_id_of_its_own(),
            account_id: account_id.to_string(),
            name: name.to_string(),
            url: url.to_string(),
            ctag: None,
        }
    }

    /// The word this address book's contacts are filed under.
    ///
    /// Per address book, for the reason the module header gives: every question
    /// the contact merge asks is keyed on this word, so two address books
    /// sharing one are one address book to all of them.
    pub fn the_word_its_contacts_are_filed_under(&self) -> AddressBook {
        AddressBook::Other(format!("carddav-{}", self.id))
    }
}

/// An identifier for an address book this program added.
fn an_id_of_its_own() -> String {
    // RED: every address book gets the same one, so every address book's
    // contacts are filed under the same word, which is what `05-RESEARCH.md`
    // assumed would do. The tests below say what it costs.
    "the-address-book".to_string()
}

impl MessageCache {
    /// Write an address book down, or replace the one already under its id.
    pub fn save_address_book(&self, book: &AddressBookContainer) -> Result<()> {
        let _ = book;
        Ok(())
    }

    /// Every address book kept with this account.
    pub fn get_address_books_for_account(
        &self,
        account_id: &str,
    ) -> Result<Vec<AddressBookContainer>> {
        let _ = account_id;
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::paths::AppPaths;
    use crate::common::temp_home::TempHome;
    use crate::service::carddav::{KEYRING_PASSWORD, KEYRING_USERNAME, keyring_service, sign_in};

    fn paths_for(label: &str) -> TempHome<AppPaths> {
        TempHome::named(label, |dir| AppPaths::under(dir.to_path_buf()))
    }

    fn cache_for(label: &str) -> (TempHome<AppPaths>, MessageCache) {
        let paths = paths_for(label);
        let cache = MessageCache::new(paths.cache_dir(), None).expect("a cache of its own");
        (paths, cache)
    }

    #[test]
    fn test_an_address_book_added_by_its_address_comes_back_the_same() {
        let (_paths, cache) = cache_for("address-book-round-trip");
        let book = AddressBookContainer::new("a1", "Work", "https://dav.example.com/books/work/");

        cache.save_address_book(&book).expect("it saves");

        assert_eq!(
            cache.get_address_books_for_account("a1").expect("it reads"),
            vec![book]
        );
    }

    #[test]
    fn test_an_address_book_starts_with_no_change_marker_and_keeps_the_one_it_is_given() {
        // The marker is what the next sync compares against to find out whether
        // anything moved. A first sync has nothing to compare, and a marker
        // written down and not kept means every sync reads the whole book.
        let (_paths, cache) = cache_for("address-book-marker");
        let mut book =
            AddressBookContainer::new("a1", "Work", "https://dav.example.com/books/work/");
        assert_eq!(book.ctag, None, "a new address book has no marker yet");

        book.ctag = Some("\"ctag-7\"".to_string());
        cache.save_address_book(&book).expect("it saves");

        let stored = cache.get_address_books_for_account("a1").expect("it reads");
        assert_eq!(
            stored.first().and_then(|held| held.ctag.clone()),
            Some("\"ctag-7\"".to_string())
        );
    }

    #[test]
    fn test_two_address_books_on_one_account_are_two_address_books() {
        // The failure this exists to stop. Both filed under the bare word
        // `carddav`, every question the contact merge asks about one answers
        // for the other: a change waiting to go to this book reads as waiting
        // for that one, and a contact in both carries one identity where it
        // needs two.
        let (_paths, cache) = cache_for("two-address-books");
        let work = AddressBookContainer::new("a1", "Work", "https://dav.example.com/books/work/");
        let home = AddressBookContainer::new("a1", "Home", "https://dav.example.com/books/home/");

        assert_ne!(work.id, home.id, "two address books share one id");
        assert_ne!(
            work.the_word_its_contacts_are_filed_under(),
            home.the_word_its_contacts_are_filed_under(),
            "two address books' contacts are filed under one word"
        );

        cache.save_address_book(&work).expect("the first saves");
        cache.save_address_book(&home).expect("the second saves");
        assert_eq!(
            cache
                .get_address_books_for_account("a1")
                .expect("they read")
                .len(),
            2
        );
    }

    #[test]
    fn test_the_word_an_address_books_contacts_are_filed_under_survives_the_database() {
        // It is written into every one of its contacts' rows and read back by
        // `AddressBook::from_stored`, so a word that does not survive that trip
        // is a contact nobody can match to the address book it came from.
        let book = AddressBookContainer::new("a1", "Work", "https://dav.example.com/books/work/");
        let filed_under = book.the_word_its_contacts_are_filed_under();

        assert_eq!(
            AddressBook::from_stored(filed_under.as_stored()),
            filed_under
        );
        assert!(
            filed_under.as_stored().starts_with("carddav-"),
            "the word says nothing about what kind of address book it is: {}",
            filed_under.as_stored()
        );
    }

    #[test]
    fn test_an_address_book_is_only_listed_for_the_account_it_is_kept_with() {
        let (_paths, cache) = cache_for("address-book-per-account");
        cache
            .save_address_book(&AddressBookContainer::new(
                "a1",
                "Work",
                "https://dav.example.com/books/work/",
            ))
            .expect("it saves");

        assert!(
            cache
                .get_address_books_for_account("a2")
                .expect("it reads")
                .is_empty()
        );
    }

    #[test]
    fn test_a_database_with_no_address_books_table_opens_and_gains_one() {
        // Somebody upgrading has a database written before this table existed.
        // Opening it has to add the table and leave everything already in it
        // alone, which is this project's schema rule.
        let paths = paths_for("address-book-older-database");
        let cache_dir = paths.cache_dir();
        std::fs::create_dir_all(&cache_dir).expect("the folder");
        let older = cache_dir.join("message_cache.db");
        {
            let conn = rusqlite::Connection::open(&older).expect("an older database");
            conn.execute("CREATE TABLE what_was_here_before (word TEXT)", [])
                .expect("the older table");
            conn.execute(
                "INSERT INTO what_was_here_before (word) VALUES ('still here')",
                [],
            )
            .expect("the older row");
        }

        let cache = MessageCache::new(cache_dir.clone(), None).expect("the older database opens");
        let book = AddressBookContainer::new("a1", "Work", "https://dav.example.com/books/work/");
        cache
            .save_address_book(&book)
            .expect("the table it did not have");
        assert_eq!(
            cache.get_address_books_for_account("a1").expect("it reads"),
            vec![book]
        );
        drop(cache);

        let conn = rusqlite::Connection::open(&older).expect("it opens again");
        let kept: String = conn
            .query_row("SELECT word FROM what_was_here_before", [], |row| {
                row.get(0)
            })
            .expect("what was there before");
        assert_eq!(kept, "still here");
    }

    // ── The sign-in, which is not in any of that ─────────────────────────

    #[test]
    fn test_the_service_name_a_sign_in_is_kept_under_is_the_one_uninstalling_removes() {
        // Pinned to the literal, the same as `credentials::KEYRING_SERVICE` is.
        // The code that erases secrets names entries by this exact string, so
        // changing it orphans a password on every machine that has one.
        assert_eq!(
            keyring_service("address-book-17"),
            "wixen-mail-carddav-address-book-17"
        );
        assert_eq!(KEYRING_USERNAME, "username");
        assert_eq!(KEYRING_PASSWORD, "password");
    }

    #[test]
    fn test_two_address_books_sign_ins_do_not_share_one_entry() {
        // One service name for both is one password for both: signing in to the
        // second overwrites the first, and the first address book then fails to
        // sign in with somebody else's details.
        let (_paths, _cache) = cache_for("two-sign-ins");
        sign_in::store("address-book-1", "ann", "one").expect("the first stores");
        sign_in::store("address-book-2", "bob", "two").expect("the second stores");

        assert_eq!(
            sign_in::load("address-book-1"),
            Some(("ann".to_string(), "one".to_string()))
        );
        assert_eq!(
            sign_in::load("address-book-2"),
            Some(("bob".to_string(), "two".to_string()))
        );
    }

    #[test]
    fn test_half_a_stored_sign_in_is_not_a_sign_in() {
        // The same rule the calendar's has, for the same reason: a blank
        // password sent to a server gets a refusal that reads as a broken
        // account, so an address book with only one half is left alone until
        // somebody types the other.
        sign_in::store("address-book-half", "ann", "").expect("it stores");

        assert_eq!(sign_in::load("address-book-half"), None);
    }

    #[test]
    fn test_a_sign_in_stored_through_the_real_path_is_in_no_column_of_the_row() {
        // A test that only checked the credential store would be green against
        // code that wrote the password into the row as well. This stores one
        // the way the screen does and then reads every column back.
        let (_paths, cache) = cache_for("no-password-in-the-row");
        let book = AddressBookContainer::new("a1", "Work", "https://dav.example.com/books/work/");
        sign_in::store(&book.id, "ann", "hunter2").expect("the sign-in stores");
        cache.save_address_book(&book).expect("the row saves");

        let stored = cache.get_address_books_for_account("a1").expect("it reads");
        let row = stored.first().expect("the row");
        for column in [
            row.id.as_str(),
            row.account_id.as_str(),
            row.name.as_str(),
            row.url.as_str(),
            row.ctag.as_deref().unwrap_or(""),
        ] {
            assert!(
                !column.contains("hunter2"),
                "the password is in the database: {column}"
            );
            assert!(
                !column.contains("ann"),
                "the user name is in the database: {column}"
            );
        }
    }

    #[test]
    fn test_forgetting_an_address_books_sign_in_takes_both_halves() {
        sign_in::store("address-book-forget", "ann", "hunter2").expect("it stores");
        sign_in::forget("address-book-forget").expect("it goes");

        assert_eq!(sign_in::load("address-book-forget"), None);
    }
}

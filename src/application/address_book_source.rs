//! Adding an address book by the address it lives at.
//!
//! One kind, unlike a calendar: a CardDAV address book is signed in to and
//! written back to, and there is no such thing as a published address book feed
//! that this only ever reads. So there is no choice to make on the screen, and
//! the sign-in boxes are never the ones that are greyed out.
//!
//! Everything a screen would have to decide is here rather than in the window,
//! because a window needs a display and a running application and nothing about
//! it can be tested. What somebody typed becomes an address, an address is
//! usable or it is not, a server's answer becomes a list somebody can choose
//! from, and a failure becomes a sentence. The window is left with the widgets.
//!
//! # The checks are the calendar's; the sentences are not
//!
//! An address typed here has exactly the problems a calendar's address has, for
//! exactly the same reasons, so [`crate::application::calendar_source::address_for`]
//! is what checks it. Two lists of reasons would drift, and the second one
//! would be the one nobody updated.
//!
//! The words are a different question. Four of `Unusable`'s six sentences name
//! a calendar in so many words, and one of those four ends by offering to add
//! the calendar as a feed instead, which for an address book is a way out that
//! does not exist. So the two that name nothing are reused exactly as they
//! stand, and the four that do are answered here. [`said_about_an_address_book`]
//! is the whole of that, and a test walks every reason so none can be left
//! without words.
//!
//! # Nothing here changes anything at a server
//!
//! Discovery asks a server what it has. It goes out through
//! [`CardDavClient::new`], which holds the refusing client, so a request that
//! changed something would be stopped before the network even if somebody later
//! moved discovery onto a writing method. The two things this does write are on
//! this computer: a row in the address books table, and the sign-in in the
//! credential store.

use crate::application::calendar_source::Unusable;
use crate::common::Error;
use crate::data::message_cache::{AddressBookContainer, MessageCache};
use crate::service::carddav::{CardDavAddressBook, CardDavClient};

/// What the window says about itself, where somebody adding an address book
/// reads it.
///
/// Not in a report and not in a log line. This is the first way in the product
/// to reach an address book server at all, and nobody has pointed it at a real
/// one. Changes do go back, which raises the stakes rather than lowering them:
/// what was an address book this program could only get wrong on this computer
/// is now one it can get wrong on somebody's server.
///
/// Its own words rather than a share of `calendar_source::NOT_TRIED_FOR_REAL`,
/// which says "real calendar server". A shared constant would say the wrong
/// noun to everybody who reads this screen, and a sentence read aloud that
/// names the wrong kind of thing is worse than a longer file.
pub const NOT_TRIED_FOR_REAL: &str =
    // RED: the calendar's own sentence, shared. It says "real calendar
    // server" to somebody adding an address book.
    crate::application::calendar_source::NOT_TRIED_FOR_REAL;

/// What somebody is told about an address they cannot use.
///
/// The checking is the calendar's and the wording is not, and the module header
/// says why. Four of the six name a calendar in so many words, and the worst of
/// those offers to add it as a feed instead, which for an address book is a way
/// out that does not exist. The two that name nothing are the calendar's own
/// words, reached through `Unusable::said`, so a later correction to either of
/// them is a correction to both screens.
pub fn said_about_an_address_book(why: Unusable) -> &'static str {
    // RED: the check is reused and so are its words, which is the reading
    // of "reuse the address checks" that puts the word calendar in front of
    // somebody adding an address book four times, one of those offering a
    // feed that does not exist.
    why.said()
}

/// What somebody typed, turned into an address that can be asked, or a reason
/// it cannot.
///
/// The calendar's check, asked as a server rather than as a feed: an address
/// book is always signed in to, so an unencrypted one is always a password on
/// the wire.
pub fn address_for(typed: &str) -> std::result::Result<String, Unusable> {
    // RED: the check is written again here rather than reused, and it is the
    // shorter version somebody writes from memory.
    let typed = typed.trim();
    if typed.is_empty() {
        return Err(Unusable::Nothing);
    }
    Ok(format!("https://{typed}"))
}

/// One line somebody can choose from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    /// What is read out. Distinguishable from its neighbours, always.
    pub name: String,
    /// The address the server itself gave, kept as it came.
    pub address: String,
}

/// What a server's answer becomes: the address books it really has, each with
/// something a person can tell apart from the one next to it.
///
/// A picker read aloud is only its lines. Two address books called Contacts,
/// which one account holding one of somebody's own and one shared to them
/// ordinarily has, are two identical lines and nobody knows which they chose.
/// So a name shared with another line has the address book's own address added
/// to it.
pub fn offered(found: &[CardDavAddressBook]) -> Vec<Offer> {
    // RED: whatever the server said, with nothing done about two of them
    // saying the same thing.
    found
        .iter()
        .map(|book| Offer {
            name: book.display_name.clone(),
            address: book.url.clone(),
        })
        .collect()
}

/// What somebody is told when an address book server could not be added.
///
/// One sentence per case, each saying what happened and what to do next. Never
/// the words underneath: a transport failure read back at somebody tells them
/// nothing they can act on and quite a lot they did not ask about.
pub fn what_went_wrong(failure: &Error) -> String {
    match failure {
        Error::Api { status: 401, .. } | Error::Api { status: 403, .. } => {
            "The address book server did not accept that user name and password. Check \
             both and try again."
        }
        Error::Api { status: 404, .. } => {
            "There is no address book at that address. Check the address with whoever \
             gave it to you and try again."
        }
        Error::Api { .. } => {
            "The address book server could not answer just now. Try again in a few \
             minutes, and if it keeps happening ask whoever looks after the server."
        }
        Error::Network(_) => {
            "That address could not be reached. Check the address and your connection, \
             then try again."
        }
        _ => {
            "The address book could not be added. Check the address and the sign-in \
             details, then try again."
        }
    }
    .to_string()
}

/// What somebody is told when a server answered and had nothing to offer.
const NOTHING_OFFERED: &str = "That server has no address books on it for this sign-in. Check \
                               the address, or sign in as somebody an address book is shared \
                               with.";

/// What somebody is told when an address book would be filed where nothing
/// visits.
const NO_ACCOUNT_TO_FILE_IT_UNDER: &str = "Set up an account first. An address book is kept with \
                                           an account, and only an account's address books are \
                                           brought up to date.";

/// How long an address book server is given to answer before somebody is told
/// it did not.
///
/// The same half minute a calendar server gets, and for the same reasons: long
/// enough for a server on the other side of the world on a bad line, short
/// enough that nobody sits through it twice.
pub const HOW_LONG_A_SERVER_IS_GIVEN: std::time::Duration = std::time::Duration::from_secs(30);

/// What somebody is told the moment the asking starts.
///
/// Said, not only shown. A button pressed with nothing said after it is a
/// button somebody presses again. The wait is bounded, so the sentence says by
/// how much rather than leaving somebody to guess, and it names the way out.
pub fn looking_for_address_books() -> String {
    format!(
        "Looking for address books on that server. This can take up to {} seconds. \
         Choose Stop looking if you would rather not wait.",
        HOW_LONG_A_SERVER_IS_GIVEN.as_secs()
    )
}

/// What the list of address books is labelled once the server has answered.
///
/// The count comes first because it is the part nobody can get any other way: a
/// list that fills in silence tells a listener one row and nothing about how
/// many there are. It carries the mnemonic for the list it labels.
pub fn how_many_were_found(count: usize) -> String {
    match count {
        1 => "1 &address book found on that server. Choose it to add it:".to_string(),
        many => format!("{many} &address books found on that server. Choose the one to add:"),
    }
}

/// What somebody is told when they stopped the looking themselves.
pub const LOOKING_WAS_STOPPED: &str = "Looking for address books was stopped. Nothing was added.";

/// What somebody is told when the server never answered.
pub const NO_ANSWER_IN_TIME: &str = "That server did not answer in time. Check the address and \
                                     your connection, then try again.";

/// Whether an address book can be filed under this account at all, or the
/// sentence saying why not.
///
/// Asked before a server is, so nobody types a password and waits for an answer
/// to a question that could never have been kept.
pub fn can_be_filed_under(account_id: &str) -> std::result::Result<(), String> {
    if account_id == crate::application::new_item::LOCAL_ACCOUNT_ID {
        return Err(NO_ACCOUNT_TO_FILE_IT_UNDER.to_string());
    }
    Ok(())
}

/// Ask an address book server what address books it has.
///
/// Nothing on this computer is touched, which is what lets this run away from
/// the thread that draws the window while that window stays alive and able to
/// speak. Somebody who gives up, or closes the window, is left with exactly
/// what they started with.
pub async fn what_a_server_has(
    typed: &str,
    user_name: &str,
    password: &str,
) -> std::result::Result<Vec<Offer>, String> {
    // Before anything is sent. A refusal after the request is a password
    // already on the wire.
    let address = address_for(typed).map_err(said_about_an_address_book)?;

    // The refusing client, not the one built from what an account is allowed.
    // Asking a server what it has changes nothing, and building it this way
    // means a later edit that moved discovery onto a changing method would be
    // stopped here rather than at somebody's address book.
    let found = CardDavClient::new()
        .discover_address_books(&address, user_name, password)
        .await
        .map_err(|failure| what_went_wrong(&failure))?;

    let offers = offered(&found);
    if offers.is_empty() {
        return Err(NOTHING_OFFERED.to_string());
    }
    Ok(offers)
}

/// Add the address book somebody chose: keep the sign-in where passwords go,
/// and write the row.
///
/// Local work only, so it is the half that can happen on the thread the window
/// is on without holding anything up.
///
/// The sign-in goes first, so an address book never exists with no way to reach
/// it. If the row cannot be written the sign-in is taken back out again, which
/// is the shape `calendar_source::add_the_chosen` already has and for the same
/// reason: nothing about a store that will not answer should end in a row
/// somebody can see and nothing can sign in to.
pub fn add_the_chosen(
    cache: &MessageCache,
    account_id: &str,
    chosen: &Offer,
    user_name: &str,
    password: &str,
) -> std::result::Result<AddressBookContainer, String> {
    can_be_filed_under(account_id)?;
    let row = AddressBookContainer::new(account_id, &chosen.name, &chosen.address);
    // RED: the sign-in is written into the row, which is the shortcut somebody
    // takes when the credential store is awkward on their machine.
    let mut row = row;
    row.url = format!("{user_name}:{password}@{}", row.url);
    if let Err(failure) = cache.save_address_book(&row) {
        tracing::error!("An address book added by address could not be saved: {failure}");
        return Err("The address book could not be saved on this computer. Try again.".to_string());
    }
    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::calendar_source::Source;
    use crate::common::paths::AppPaths;
    use crate::common::temp_home::TempHome;
    use crate::service::carddav::sign_in;

    fn cache_for(label: &str) -> (TempHome<AppPaths>, MessageCache) {
        let paths = TempHome::named(label, |dir| AppPaths::under(dir.to_path_buf()));
        let cache = MessageCache::new(paths.cache_dir(), None).expect("a cache of its own");
        (paths, cache)
    }

    fn a_book(name: &str, url: &str) -> CardDavAddressBook {
        CardDavAddressBook {
            url: url.to_string(),
            display_name: name.to_string(),
            ctag: None,
        }
    }

    // ── The address, checked by the calendar's own rules ─────────────────

    #[test]
    fn test_the_checks_are_the_calendars_rather_than_a_second_set() {
        // Two lists of reasons drift, and the second one is the one nobody
        // updates. Every typed address either gets the same answer from both,
        // or this file has grown a rule the calendar does not have.
        for typed in [
            "",
            "dav.example.com/books/",
            "http://dav.example.com/books/",
            "https://sam@dav.example.com/books/",
            "https://dav.example.com/a book/",
            "ftp://dav.example.com/books/",
            "https://127.0.0.1:8008/books/",
            "http://localhost:8008/books/",
            "not an address at all",
        ] {
            assert_eq!(
                address_for(typed),
                crate::application::calendar_source::address_for(typed, Source::Server),
                "the address book screen and the calendar screen disagree about {typed:?}"
            );
        }
    }

    #[test]
    fn test_a_sign_in_is_never_sent_over_an_unencrypted_connection() {
        // An address book is always signed in to, so an unencrypted address is
        // always a password on the wire. There is no feed half here to excuse
        // one.
        assert_eq!(
            address_for("http://dav.example.com/books/"),
            Err(Unusable::NotEncrypted)
        );
    }

    // ── The words, which are this screen's own where they name a thing ───

    #[test]
    fn test_no_reason_an_address_cannot_be_used_says_calendar_to_somebody_adding_contacts() {
        // Every reason, so none can be left with the wrong noun. Sixteen
        // widgets in this project were once named with a call that reaches
        // nothing; a sentence that names the wrong kind of thing is the same
        // shape of defect, and it is read aloud.
        for why in Unusable::ALL {
            let said = said_about_an_address_book(why);
            assert!(
                !said.to_ascii_lowercase().contains("calendar"),
                "{why:?} tells somebody adding an address book about a calendar: {said}"
            );
            assert!(
                !said.to_ascii_lowercase().contains("feed"),
                "{why:?} offers a way out that does not exist for an address book: {said}"
            );
            assert!(said.ends_with('.'), "{why:?}: {said}");
            // Read aloud, so a wrapped literal that lost a continuation is a
            // run of silence in the middle of it.
            assert!(!said.contains("  "), "{why:?}: {said}");
        }
    }

    #[test]
    fn test_the_two_reasons_that_never_named_a_calendar_are_the_calendars_own_words() {
        // Reused rather than copied, so a later correction to either is a
        // correction to both screens.
        for why in [Unusable::CarriesASignIn, Unusable::HasStrayCharacters] {
            assert_eq!(said_about_an_address_book(why), why.said());
        }
    }

    #[test]
    fn test_the_four_reasons_that_named_a_calendar_say_something_else_here() {
        // The other half of the pair. Without it, a version that reused all six
        // would pass the test above only because `Unusable::said` had changed
        // under it.
        for why in [
            Unusable::Nothing,
            Unusable::NotAnAddress,
            Unusable::NotTheWeb,
            Unusable::NotEncrypted,
        ] {
            assert_ne!(said_about_an_address_book(why), why.said());
        }
    }

    #[test]
    fn test_the_screen_says_it_has_never_been_tried_against_a_real_server() {
        // Said in the window before anybody types a password into it.
        assert!(
            NOT_TRIED_FOR_REAL.contains("experimental"),
            "{NOT_TRIED_FOR_REAL}"
        );
        assert!(
            NOT_TRIED_FOR_REAL.contains("address book server"),
            "{NOT_TRIED_FOR_REAL}"
        );
        assert!(
            !NOT_TRIED_FOR_REAL.to_ascii_lowercase().contains("calendar"),
            "{NOT_TRIED_FOR_REAL}"
        );
        assert!(
            NOT_TRIED_FOR_REAL.contains("Allow Changes"),
            "the setting that lets a change out is not named: {NOT_TRIED_FOR_REAL}"
        );
        assert!(!NOT_TRIED_FOR_REAL.contains("  "), "{NOT_TRIED_FOR_REAL}");
    }

    // ── The list somebody chooses from ───────────────────────────────────

    #[test]
    fn test_two_address_books_with_one_name_are_told_apart() {
        // A picker read aloud is only its lines. One of somebody's own and one
        // shared to them, both called Contacts, are two identical lines and
        // nobody knows which they chose.
        let offers = offered(&[
            a_book("Contacts", "https://dav.example.com/sam/contacts/"),
            a_book("Contacts", "https://dav.example.com/pat/contacts/"),
        ]);

        assert_ne!(offers[0].name, offers[1].name);
        assert!(offers[0].name.contains("sam"), "{:?}", offers[0]);
        assert!(offers[1].name.contains("pat"), "{:?}", offers[1]);
    }

    #[test]
    fn test_an_address_book_with_a_name_of_its_own_is_offered_under_it() {
        // The other half. A version that put the address on every line would
        // pass the test above and read out a URL to everybody.
        let offers = offered(&[a_book("Work", "https://dav.example.com/sam/work/")]);

        assert_eq!(offers[0].name, "Work");
    }

    #[test]
    fn test_an_address_book_with_nowhere_to_be_asked_is_not_offered() {
        // An address book with no address is worse than none: every later
        // request for it resolves the empty address against the base and goes
        // somewhere nobody chose.
        assert!(offered(&[a_book("Contacts", "")]).is_empty());
    }

    // ── Adding the one that was chosen ───────────────────────────────────

    #[test]
    fn test_adding_the_chosen_one_keeps_the_sign_in_out_of_the_database() {
        // Through the real path, and then every column of the row is read. A
        // test that only asked the credential store would be green against
        // code that wrote the password into the row as well.
        let (_paths, cache) = cache_for("adding-keeps-the-sign-in-out");
        let chosen = Offer {
            name: "Work".to_string(),
            address: "https://dav.example.com/sam/work/".to_string(),
        };

        let added = add_the_chosen(&cache, "a1", &chosen, "sam", "hunter2").expect("it is added");

        for column in [
            added.id.as_str(),
            added.account_id.as_str(),
            added.name.as_str(),
            added.url.as_str(),
            added.ctag.as_deref().unwrap_or(""),
        ] {
            assert!(
                !column.contains("hunter2"),
                "the password is in the row: {column}"
            );
            assert!(
                !column.contains("sam"),
                "the user name is in the row: {column}"
            );
        }
        assert_eq!(
            sign_in::load(&added.id),
            Some(("sam".to_string(), "hunter2".to_string())),
            "the sign-in did not reach the credential store"
        );
    }

    #[test]
    fn test_an_address_book_added_this_way_is_one_the_sync_can_find() {
        let (_paths, cache) = cache_for("adding-is-found");
        let chosen = Offer {
            name: "Work".to_string(),
            address: "https://dav.example.com/sam/work/".to_string(),
        };

        let added = add_the_chosen(&cache, "a1", &chosen, "sam", "hunter2").expect("it is added");

        assert_eq!(
            cache
                .get_address_books_for_account("a1")
                .expect("they read"),
            vec![added]
        );
    }

    #[test]
    fn test_an_address_book_cannot_be_filed_under_no_account() {
        // Asked before a server is, so nobody types a password and waits for
        // an answer to a question that could never have been kept.
        let (_paths, cache) = cache_for("adding-with-no-account");
        let chosen = Offer {
            name: "Work".to_string(),
            address: "https://dav.example.com/sam/work/".to_string(),
        };

        let refused = add_the_chosen(
            &cache,
            crate::application::new_item::LOCAL_ACCOUNT_ID,
            &chosen,
            "sam",
            "hunter2",
        );

        assert_eq!(refused, Err(NO_ACCOUNT_TO_FILE_IT_UNDER.to_string()));
    }

    // ── The wiring, read out of the window layer ─────────────────────────

    /// The body of one function in a source file, as text.
    fn the_body_of(path: &str, signature: &str) -> String {
        let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let start = source
            .find(signature)
            .unwrap_or_else(|| panic!("{path} no longer holds {signature}"));
        let from_there = &source[start..];
        let end = from_there
            .find("\n}\n")
            .unwrap_or_else(|| panic!("{signature} has no end in {path}"));
        from_there[..end].to_string()
    }

    #[test]
    fn test_adding_an_address_book_really_goes_and_looks() {
        // The stub shape this project has met before: the handler writes the
        // row, says "added", and nothing ever asks the server what is there.
        // Everything then reads as working until the first sync finds an
        // address nobody checked.
        let body = the_body_of(
            "src/presentation/managers.rs",
            "fn ask_an_address_book_server_and_add_what_was_chosen",
        );

        assert!(
            body.contains("what_a_server_has"),
            "nothing asks the server what address books it has"
        );
        assert!(
            body.contains("add_the_chosen"),
            "nothing writes the address book somebody chose"
        );
    }

    #[test]
    fn test_the_window_is_never_held_while_an_address_book_server_is_asked() {
        // Half a minute in which the window cannot repaint, cannot answer a key
        // and cannot speak. Silence with no way to tell working from dead is
        // the worst failure this program has.
        for signature in [
            "pub fn add_address_book_by_address",
            "fn ask_an_address_book_server_and_add_what_was_chosen",
        ] {
            let body = the_body_of("src/presentation/managers.rs", signature);
            assert!(
                !body.contains("block_on"),
                "{signature} still waits on the thread that draws the window"
            );
        }
        let asking = the_body_of(
            "src/presentation/managers.rs",
            "fn ask_an_address_book_server_and_add_what_was_chosen",
        );
        assert!(
            asking.contains("rt.spawn"),
            "nothing asks the server away from this thread"
        );
        assert!(
            asking.contains("wait_for_an_answer"),
            "nothing keeps the window alive while the answer is waited for"
        );
        assert!(
            asking.contains("tokio::time::timeout"),
            "nothing bounds how long a server is waited for, so one that never answers \
             would be waited for forever"
        );
    }

    #[test]
    fn test_a_contacts_sync_really_syncs_the_address_books_somebody_added() {
        // The guardrail this project's first line is about: no feature is done
        // until a non-test path reaches it. Everything under this screen is
        // reachable only from tests until the contacts sync asks for it.
        let body = the_body_of("src/presentation/wx_app.rs", "fn spawn_contacts_sync");

        assert!(
            body.contains("sync_carddav_address_book"),
            "nothing in the running program ever syncs a CardDAV address book"
        );
        assert!(
            body.contains("get_address_books_for_account"),
            "nothing looks for the address books somebody added"
        );
    }

    #[test]
    fn test_the_settings_screen_says_this_path_has_never_met_a_server() {
        // Where somebody looking for what this program may change would find
        // it, rather than only on the screen that adds one.
        let settings = std::fs::read_to_string("src/presentation/wx_settings.rs")
            .expect("the settings screen");

        assert!(
            settings.contains("address_book_source::NOT_TRIED_FOR_REAL")
                || settings.contains("NOT_TRIED_FOR_REAL"),
            "the Permissions tab says nothing about an address book path that has never \
             met a server"
        );
    }
}

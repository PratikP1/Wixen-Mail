//! Contacts going both ways with a CardDAV address book.
//!
//! The third address book, after Google and Microsoft, and the first that is
//! not a company's own API. What it adds is a place to keep contacts for
//! everybody whose mail is not at one of those two.
//!
//! # This decides nothing about whose copy wins
//!
//! `contacts_sync` already holds that decision and it was built to take any
//! number of address books: `whose_copy_wins` compares the marker an address
//! book gave at the end of the last sync against the one on the copy that has
//! just arrived, rather than comparing clocks, and `ProviderIdentity` keeps the
//! waiting flag per address book rather than per contact. Its own comment
//! records what keeping it on the contact cost: a failure at one address book
//! either lost the change at the other or resent it to both for ever.
//!
//! So this file reads cards, writes cards, and hands every question about which
//! copy survives to the functions that already answer it. A second merge here
//! would be two answers to one question, which is the defect this project has
//! paid for in the calendar already.
//!
//! # The sign-in never comes through here
//!
//! [`CardsOnAServer`] takes no user name and no password. Whatever implements
//! it holds the sign-in, which for the real client means it was loaded from the
//! credential store once, where the screen put it. Nothing in the sync, in a
//! `SyncResult` or in an error message can carry a password, because the sync
//! never has one.

use crate::application::contacts_sync::{
    self, Contacts, HowFarAChangeGoes, SyncResult, WhoseCopyWins,
};
use crate::common::Result;
use crate::data::message_cache::{AddressBook, AddressBookContainer, ContactEntry, MessageCache};
use crate::service::carddav::{CardDavClient, CardOnAServer, CardsFromAServer};

/// What a contacts sync asks of a CardDAV address book.
///
/// Named for what it is rather than for the HTTP underneath, which is what lets
/// the deciding be tested: which copy wins, what is held and what the counts
/// mean are all decisions, and none of them can be run at all without an
/// account unless the asking can be faked.
///
/// Four methods, which are the four things CardDAV needs: has anything moved,
/// what is in there, put this card, take that card away.
///
/// Written out as futures rather than as `async fn`, because `async fn` in a
/// trait anybody outside this crate can see cannot say that its futures are
/// `Send`, and the sync runs on the runtime rather than on the thread that
/// draws the window. `tasks_sync::TaskService` writes `async fn` and is
/// `pub(crate)`, where the lint does not apply; this one is `pub` because
/// nothing outside its own tests calls it until the screen exists, and a
/// `pub(crate)` item with no caller is dead code that `-D warnings` refuses.
pub trait CardsOnAServer {
    /// The marker the server moves whenever anything in the address book
    /// changes, or nothing where it gives none.
    fn change_marker(
        &self,
        address_book_url: &str,
    ) -> impl std::future::Future<Output = Result<Option<String>>> + Send;
    /// Every card in the address book, read into contacts.
    fn whats_in(
        &self,
        address_book_url: &str,
        account_id: &str,
    ) -> impl std::future::Future<Output = Result<CardsFromAServer>> + Send;
    /// Write one card, and answer with the version marker the server gave it.
    ///
    /// `version` names the copy this change was built on, where one is known.
    /// Nothing means the card is new here.
    fn write(
        &self,
        card_url: &str,
        vcard: &str,
        version: Option<&str>,
    ) -> impl std::future::Future<Output = Result<Option<String>>> + Send;
    /// Take one card away.
    ///
    /// No version is passed, deliberately, and `caldav::delete_event`'s comment
    /// says why: somebody asked for the contact to go, and a version that had
    /// moved on would make the deletion fail for ever.
    fn remove(&self, card_url: &str) -> impl std::future::Future<Output = Result<()>> + Send;
}

/// A real address book server, with the sign-in it was given.
///
/// The sign-in is held here and nowhere else in the sync. Loaded once, from the
/// credential store, where the screen that added the address book put it.
pub struct AnAddressBookServer {
    client: CardDavClient,
    user_name: String,
    password: String,
}

impl AnAddressBookServer {
    /// The server one address book lives on, or nothing when this computer has
    /// no whole sign-in for it.
    ///
    /// Half a sign-in is not a sign-in: sending a blank password gets a refusal
    /// that reads as a broken account, so an address book with only one half
    /// stored is left alone until somebody types the other.
    pub fn for_the(book: &AddressBookContainer) -> Option<Self> {
        let (user_name, password) = crate::service::carddav::sign_in::load(&book.id)?;
        Some(Self {
            client: CardDavClient::for_account(&book.account_id),
            user_name,
            password,
        })
    }
}

impl CardsOnAServer for AnAddressBookServer {
    async fn change_marker(&self, address_book_url: &str) -> Result<Option<String>> {
        self.client
            .change_marker(address_book_url, &self.user_name, &self.password)
            .await
    }

    async fn whats_in(&self, address_book_url: &str, account_id: &str) -> Result<CardsFromAServer> {
        self.client
            .cards(
                address_book_url,
                &self.user_name,
                &self.password,
                account_id,
            )
            .await
    }

    async fn write(
        &self,
        card_url: &str,
        vcard: &str,
        version: Option<&str>,
    ) -> Result<Option<String>> {
        self.client
            .write_card(card_url, &self.user_name, &self.password, vcard, version)
            .await
    }

    async fn remove(&self, card_url: &str) -> Result<()> {
        self.client
            .delete_card(card_url, &self.user_name, &self.password)
            .await
    }
}

/// Sync contacts both ways with one CardDAV address book.
///
/// The order is the one `sync_google_contacts` uses and it is not arbitrary.
/// Deletions go first, because deleting a contact takes the row and any change
/// waiting in it, so there is nothing left for the change push to find. The
/// change push goes before the read, because the other order sends a value the
/// read has just written and the push undoes the thing it was told to accept.
///
/// Every decision about whose copy survives is `contacts_sync`'s. Nothing here
/// answers one.
pub async fn sync_carddav_address_book<B: CardsOnAServer>(
    cache: &MessageCache,
    server: &B,
    book: &AddressBookContainer,
) -> Result<SyncResult> {
    let filed_under = book.the_word_its_contacts_are_filed_under();
    let account_id = book.account_id.as_str();
    let mut result = SyncResult::default();

    contacts_sync::forget_the_deletions_remembered_long_enough(cache, &mut result);

    for note in contacts_sync::deletions_waiting_for(cache, account_id, &filed_under, &mut result) {
        // The address book's own name for this card is its whole address, so
        // this is the card to take away.
        let sent = server.remove(&note.provider_contact_id).await;
        contacts_sync::count_the_deletion(cache, &sent, &note, &book.name, &mut result);
    }
    let everybody_deleted_here =
        contacts_sync::contacts_deleted_here(cache, account_id, &filed_under, &mut result);

    let still_built_on_an_old_copy =
        push_changes_to(cache, server, book, &filed_under, &mut result).await;

    // Asked before the cards are. The marker is what the server moves whenever
    // anything in the address book changes, so an unmoved one means every card
    // is the card this computer already has, and asking for them is the whole
    // address book over the wire to learn that nothing happened.
    //
    // A server that gives no marker answers `None`, and `the_marker_moved`
    // reads that as moved, so those are read in full every time. That is the
    // answer that claims least: a missing marker is no evidence, not evidence
    // of stillness.
    let marker_now = server.change_marker(&book.url).await?;
    if !contacts_sync::the_marker_moved(marker_now.as_deref(), book.ctag.as_deref()) {
        return Ok(result);
    }

    let arrived = server.whats_in(&book.url, account_id).await?;
    if arrived.could_not_be_read > 0 {
        // Said rather than swallowed. An address book that really is empty and
        // one whose cards could none of them be read are different facts, and
        // the second is the one somebody goes looking for a broken program
        // over.
        result.errors.push(format!(
            "{} card(s) in {} could not be read and were passed over.",
            arrived.could_not_be_read, book.name
        ));
    }

    for card in &arrived.cards {
        if card.url.is_empty() {
            continue;
        }
        // Somebody this computer deleted. Whether the address book has taken
        // the deletion or not, writing her back down puts a contact somebody
        // deleted on the screen again.
        if everybody_deleted_here.holds(&card.url) {
            continue;
        }
        take_in_one_card(
            cache,
            card,
            &filed_under,
            &still_built_on_an_old_copy,
            &mut result,
        )?;
    }

    // Written down only after the read succeeded. Moved on after a read that
    // failed, everything that changed in between is never asked for again.
    let mut seen = book.clone();
    seen.ctag = marker_now;
    cache.save_address_book(&seen)?;

    Ok(result)
}

/// Send the address book everything made or changed here that it has not had.
///
/// Answers with the changes it offered and could not get in, which
/// [`contacts_sync::keep_a_change_this_sync_could_not_send`] needs so that the
/// read in the same sync does not destroy them.
async fn push_changes_to<B: CardsOnAServer>(
    cache: &MessageCache,
    server: &B,
    book: &AddressBookContainer,
    filed_under: &AddressBook,
    result: &mut SyncResult,
) -> Contacts {
    let mut still_built_on_an_old_copy = Contacts::default();

    // Changes to contacts this address book already holds.
    //
    // To every address book that knows the contact, because a CardDAV address
    // book is not where a contact came from in the sense the other setting
    // means: a card is the whole contact, and there is no field this address
    // book owns and the others do not.
    for (contact, card_url) in contacts_sync::changes_waiting_for(
        cache,
        &book.account_id,
        filed_under,
        HowFarAChangeGoes::ToEveryAddressBookThatKnowsThem,
        result,
    ) {
        let version = contacts_sync::version_given_by(&contact, filed_under);
        let sent = server
            .write(
                &card_url,
                &MessageCache::vcard_block_from_contact(&contact),
                version.as_deref(),
            )
            .await;
        match &sent {
            Ok(now) => {
                let told = contact.told(filed_under, now.as_deref());
                if let Err(unwritten) = cache.save_contact(&told) {
                    result.errors.push(format!(
                        "Contact {} was sent to {} and the note saying so could not be \
                         written: {unwritten}",
                        contact.id, book.name
                    ));
                }
            }
            Err(refused) if the_address_book_had_moved_past_it(refused) => {
                still_built_on_an_old_copy.note(&contact.id);
            }
            Err(_) => {}
        }
        contacts_sync::count_the_attempt(&sent.map(|_| ()), &contact.id, &book.name, result);
    }

    // Contacts made here that no address book knows yet.
    let locals = match cache.get_contacts_for_account(&book.account_id) {
        Ok(locals) => locals,
        Err(unreadable) => {
            result.errors.push(format!(
                "The contacts waiting to be sent could not be read: {unreadable}"
            ));
            return still_built_on_an_old_copy;
        }
    };
    for local in locals.iter().filter(|local| made_here(local)) {
        // The address is chosen here rather than by the server, which is what
        // CardDAV asks for: a PUT to an address nothing is at makes the card.
        // The contact's own identifier goes in the path, escaped, because one
        // holding a space breaks the request line and one holding a hash
        // truncates the address at the fragment.
        let card_url = format!(
            "{}/{}.vcf",
            book.url.trim_end_matches('/'),
            crate::service::outward::in_a_path(&local.id)
        );
        let sent = server
            .write(
                &card_url,
                &MessageCache::vcard_block_from_contact(local),
                None,
            )
            .await;
        match sent {
            Ok(now) => {
                let mut told = local
                    .also_known_to(filed_under.clone(), &card_url, now.as_deref())
                    // The address book has it, and no other is owed it: a
                    // create only happens for a contact no address book knew.
                    // Left marked as waiting, the next read counts the address
                    // book's own copy as having replaced an edit and tells
                    // somebody they lost work they still have.
                    .told(filed_under, now.as_deref());
                told.source_provider = Some(filed_under.as_stored().to_string());
                told.last_synced_at = Some(chrono::Utc::now().to_rfc3339());
                if let Err(unwritten) = cache.save_contact(&told) {
                    // Never given up on. The card is already at the server, so
                    // stopping here abandons the contacts behind this one.
                    result.errors.push(format!(
                        "Contact {} was added to {} and the note saying so could not be \
                         written: {unwritten}",
                        local.id, book.name
                    ));
                }
                result.created_remote.note(&local.id);
            }
            Err(refused) if crate::service::outward::was_refused_by_the_gate(&refused) => {
                result.waiting_on_the_setting.note(&local.id);
            }
            Err(failed) => result.errors.push(format!(
                "Contact {} could not be added to {}: {failed}",
                local.id, book.name
            )),
        }
    }

    still_built_on_an_old_copy
}

/// Whether this contact was made on this computer and no address book has it.
///
/// `pending` is what says made here. Without it every address harvested from a
/// message header goes into somebody's real address book, and nobody asked for
/// their mail history to be copied there.
fn made_here(contact: &ContactEntry) -> bool {
    contact.pending && contact.known_to.is_empty()
}

/// Whether the server turned a write down because the copy it was built on has
/// moved.
///
/// A CardDAV server answers a failed `If-Match` with 412. It is not a failure
/// somebody can act on: the next sync builds the change again on whatever the
/// address book holds by then.
fn the_address_book_had_moved_past_it(refused: &crate::common::Error) -> bool {
    matches!(refused, crate::common::Error::Api { status: 412, .. })
}

/// Fold one card the address book sent into what is stored here.
///
/// Every decision in here is made somewhere else. This is the wiring between
/// them and the order they are asked in, which is `sync_google_contacts`'s
/// order because it is the same set of questions.
fn take_in_one_card(
    cache: &MessageCache,
    card: &CardOnAServer,
    filed_under: &AddressBook,
    still_built_on_an_old_copy: &Contacts,
    result: &mut SyncResult,
) -> Result<()> {
    let locals = cache.get_contacts_for_account(&card.contact.account_id)?;
    let Some(local) =
        contacts_sync::the_stored_contact_this_is(&locals, filed_under, &card.url, &card.contact)
    else {
        cache.save_contact(&card.contact.also_known_to(
            filed_under.clone(),
            &card.url,
            card.version.as_deref(),
        ))?;
        result.created_local.note(&card.contact.id);
        return Ok(());
    };

    // A contact waiting on somebody's choice is left exactly as it is. Without
    // this the sync after the one that raised the question answers it, and
    // keeping both copies buys nothing but a slower overwrite.
    if cache.is_held_for_a_choice(&local.id)? {
        return Ok(());
    }

    let last_seen = contacts_sync::version_given_by(local, filed_under);
    let answer = contacts_sync::keep_a_change_this_sync_could_not_send(
        contacts_sync::whose_copy_wins(
            contacts_sync::the_copy_here_holds_work_nobody_has_sent(local),
            card.version.as_deref(),
            last_seen.as_deref(),
        ),
        local,
        result,
        still_built_on_an_old_copy,
    );
    if answer == WhoseCopyWins::KeepWhatIsHere {
        return Ok(());
    }
    if answer == WhoseCopyWins::NeitherCopyMoved {
        result.unchanged.note(&local.id);
        return Ok(());
    }
    // Both copies moved and one of them is somebody's own work. Keep both and
    // ask, rather than writing one over the other and saying afterwards which
    // was thrown away.
    if answer == WhoseCopyWins::TakeTheAddressBooksOverAChangeMadeHere
        && contacts_sync::the_copy_here_was_written_here(local)
    {
        return contacts_sync::hold_both_copies_of(
            cache,
            local,
            &card.contact,
            filed_under,
            card.version.as_deref(),
            result,
        );
    }

    let merged = the_cards_fields_over(local, &card.contact).also_known_to(
        filed_under.clone(),
        &card.url,
        card.version.as_deref(),
    );
    cache.save_contact(&contacts_sync::a_change_here_that_lost(
        merged,
        local,
        filed_under,
        answer,
        result,
    ))?;
    result.updated_local.note(&local.id);
    Ok(())
}

/// The stored contact with the card's copy of it folded in.
///
/// Wholesale, which Google's and Microsoft's are not, and the difference is in
/// the format rather than in the decision. A Google person names the fields it
/// carries and says nothing about the rest, so a merge there has to name each
/// one. A card is the whole contact: every field this program knows is either
/// written on it or absent from it, and absent means the contact does not have
/// that field any more.
///
/// What is kept from here is only what is not the card's to say: the row's own
/// identifier, which address books know this person, and whether a change is
/// still owed to one of them.
fn the_cards_fields_over(local: &ContactEntry, arriving: &ContactEntry) -> ContactEntry {
    ContactEntry {
        id: local.id.clone(),
        account_id: local.account_id.clone(),
        created_at: local.created_at.clone(),
        known_to: local.known_to.clone(),
        pending: local.pending,
        favorite: local.favorite,
        last_synced_at: Some(chrono::Utc::now().to_rfc3339()),
        ..arriving.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::contacts_sync::whose_copy_wins;
    use crate::common::paths::AppPaths;
    use crate::common::temp_home::TempHome;
    use crate::common::{Error, Result};
    use crate::data::account::Account;
    use crate::data::message_cache::ProviderIdentity;
    use crate::service::carddav::{CardOnAServer, CardsFromAServer};
    use std::sync::Mutex;

    const THE_ADDRESS_BOOK: &str = "https://dav.example.com/books/work/";

    /// What a test's address book server does with a card sent to it.
    ///
    /// The three answers the push has to tell apart, following the shape
    /// `tasks_sync::Writes` already uses. Refusing everything is the default,
    /// so a test that never meant to send anything fails loudly if it reaches a
    /// write by accident.
    #[derive(Default, Clone, Copy, PartialEq, Eq)]
    enum Writes {
        /// Refused for an ordinary reason.
        #[default]
        NothingIsSent,
        /// Taken, and the server gives a new marker back.
        Accepted,
        /// Refused by Allow Changes before anything left the machine, which is
        /// a setting rather than a fault.
        RefusedByTheGate,
    }

    /// An address book server written out by hand, so the deciding can be run.
    ///
    /// It can refuse a write, answer an empty list, and give back a marker that
    /// has moved on, which are the three things a fake that always succeeds
    /// cannot do and which are where every interesting decision lives.
    ///
    /// `Mutex` rather than `RefCell` because these methods take `&self` inside
    /// futures the sync holds across its own awaits.
    #[derive(Default)]
    struct Scripted {
        /// What the server says has changed. Nothing means it gives no marker.
        marker: Option<String>,
        /// What is in the address book right now.
        cards: Vec<CardOnAServer>,
        /// How many of its cards could not be read.
        could_not_be_read: usize,
        /// Whether the address book itself refuses to be read.
        unreadable: bool,
        /// What it does with a card written to it.
        writes: Writes,
        /// The marker it hands back for a card it accepted.
        marker_after_a_write: Option<String>,
        /// Every card it was asked to write, in the order asked, recorded
        /// before the answer: the question is whether the call reached the
        /// server at all, and a refusal is still a call that reached it.
        written: Mutex<Vec<(String, String)>>,
        /// Every card it was asked to take away.
        removed: Mutex<Vec<String>>,
        /// How many times its cards were asked for, so a test can say that a
        /// read was skipped rather than that it found nothing.
        times_read: Mutex<usize>,
    }

    impl CardsOnAServer for Scripted {
        async fn change_marker(&self, _address_book_url: &str) -> Result<Option<String>> {
            Ok(self.marker.clone())
        }

        async fn whats_in(
            &self,
            _address_book_url: &str,
            _account_id: &str,
        ) -> Result<CardsFromAServer> {
            *self.times_read.lock().expect("the count") += 1;
            if self.unreadable {
                return Err(Error::Network("that address book did not answer".into()));
            }
            Ok(CardsFromAServer {
                cards: self.cards.clone(),
                could_not_be_read: self.could_not_be_read,
            })
        }

        async fn write(
            &self,
            card_url: &str,
            vcard: &str,
            _version: Option<&str>,
        ) -> Result<Option<String>> {
            self.written
                .lock()
                .expect("what was written")
                .push((card_url.to_string(), vcard.to_string()));
            match self.writes {
                Writes::Accepted => Ok(self.marker_after_a_write.clone()),
                Writes::RefusedByTheGate => Err(Error::Security(
                    "Allow Changes is off for this account".into(),
                )),
                Writes::NothingIsSent => Err(Error::Api {
                    status: 500,
                    provider: "address book server".to_string(),
                    message: "nothing is sent".to_string(),
                }),
            }
        }

        async fn remove(&self, card_url: &str) -> Result<()> {
            self.removed
                .lock()
                .expect("what was removed")
                .push(card_url.to_string());
            match self.writes {
                Writes::Accepted => Ok(()),
                Writes::RefusedByTheGate => Err(Error::Security(
                    "Allow Changes is off for this account".into(),
                )),
                Writes::NothingIsSent => Err(Error::Api {
                    status: 500,
                    provider: "address book server".to_string(),
                    message: "nothing is sent".to_string(),
                }),
            }
        }
    }

    fn cache_for(label: &str) -> (TempHome<AppPaths>, MessageCache) {
        let paths = TempHome::named(label, |dir| AppPaths::under(dir.to_path_buf()));
        let cache = MessageCache::new(paths.cache_dir(), None).expect("a cache of its own");
        let mut account = Account::new("Test".to_string(), "me@example.com".to_string());
        account.id = "a1".to_string();
        cache.save_account(&account).expect("the account saves");
        (paths, cache)
    }

    fn an_address_book(cache: &MessageCache) -> AddressBookContainer {
        let book = AddressBookContainer::new("a1", "Work", THE_ADDRESS_BOOK);
        cache.save_address_book(&book).expect("it saves");
        book
    }

    /// A contact on this computer, with an id of its own.
    ///
    /// Written out rather than taken from a shared builder, because the
    /// builders in `contacts_sync` are in that file's test module and nothing
    /// here may add a `#[test]` to it: 77 guard records fingerprint it.
    fn a_contact(id: &str, name: &str, email: &str) -> ContactEntry {
        ContactEntry {
            id: id.to_string(),
            account_id: "a1".to_string(),
            name: name.to_string(),
            given_name: None,
            family_name: None,
            name_prefix: None,
            middle_name: None,
            name_suffix: None,
            email: email.to_string(),
            phone: None,
            company: None,
            job_title: None,
            website: None,
            address: None,
            birthday: None,
            avatar_url: None,
            avatar_data_base64: None,
            source_provider: None,
            last_synced_at: None,
            vcard_raw: None,
            notes: None,
            favorite: false,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            nickname: None,
            department: None,
            relationship: None,
            emails_json: None,
            phones_json: None,
            addresses_json: None,
            custom_fields_json: None,
            pending: false,
            known_to: Vec::new(),
        }
    }

    /// A contact this address book already holds, changed here since it last
    /// had it.
    ///
    /// The waiting flag is set by hand. `also_known_to` deliberately does not
    /// raise it: whether a change is owed to an address book is not something a
    /// read can answer, so a pull naming the contact again must not clear a
    /// push that has not gone through.
    fn changed_here(
        name: &str,
        email: &str,
        filed_under: &AddressBook,
        card_url: &str,
        version: &str,
    ) -> ContactEntry {
        let mut contact = a_contact("here-1", name, email);
        contact.pending = true;
        contact.known_to = vec![ProviderIdentity {
            address_book: filed_under.clone(),
            provider_contact_id: card_url.to_string(),
            provider_version: Some(version.to_string()),
            change_is_waiting: true,
        }];
        contact
    }

    /// A contact made on this computer that no address book knows yet.
    fn made_here(name: &str, email: &str) -> ContactEntry {
        let mut contact = a_contact("here-1", name, email);
        contact.pending = true;
        contact
    }

    /// One card the server holds, for a contact with this name and address.
    fn a_card_for(url: &str, version: &str, name: &str, email: &str) -> CardOnAServer {
        CardOnAServer {
            url: url.to_string(),
            version: Some(version.to_string()),
            contact: a_contact("from-the-server", name, email),
        }
    }

    // ── Sending what was made or changed here ────────────────────────────

    #[tokio::test]
    async fn test_a_contact_made_here_is_sent_to_the_address_book_once() {
        // And not again after the address book has taken it. Left waiting, the
        // next sync sends the address book its own copy back and counts it on
        // the status line as one of yours sent, which is not true.
        let (_paths, cache) = cache_for("carddav-create");
        let book = an_address_book(&cache);
        let ann = made_here("Ann Example", "ann@example.com");
        cache.save_contact(&ann).expect("she saves");
        let server = Scripted {
            writes: Writes::Accepted,
            marker_after_a_write: Some("\"v1\"".to_string()),
            ..Scripted::default()
        };

        let first = sync_carddav_address_book(&cache, &server, &book)
            .await
            .expect("the first sync");
        assert_eq!(first.created_remote.count(), 1);
        assert_eq!(server.written.lock().expect("written").len(), 1);

        let second = sync_carddav_address_book(&cache, &server, &book)
            .await
            .expect("the second sync");
        assert_eq!(second.created_remote.count(), 0);
        assert_eq!(
            server.written.lock().expect("written").len(),
            1,
            "the address book was sent the same contact twice"
        );
    }

    #[tokio::test]
    async fn test_a_write_the_setting_refused_is_counted_and_names_the_setting() {
        // Counted rather than reported as an error. One error per waiting
        // change on every sync from now on is how a warning somebody needs
        // stops being read.
        let (_paths, cache) = cache_for("carddav-refused");
        let book = an_address_book(&cache);
        cache
            .save_contact(&made_here("Ann Example", "ann@example.com"))
            .expect("she saves");
        let server = Scripted {
            writes: Writes::RefusedByTheGate,
            ..Scripted::default()
        };

        let result = sync_carddav_address_book(&cache, &server, &book)
            .await
            .expect("the sync");

        assert_eq!(result.waiting_on_the_setting.count(), 1);
        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let said = crate::application::contacts_sync::what_the_contacts_sync_did(&result);
        assert!(
            said.contains(&crate::application::allowed::changes_waiting_here(1)),
            "the setting to turn on was not named: {said}"
        );
    }

    #[tokio::test]
    async fn test_a_sync_that_sent_everything_does_not_name_the_setting() {
        // The other half of the pair. A summary that always named the setting
        // would satisfy the test above while telling everybody to turn on
        // something that stopped nothing.
        let (_paths, cache) = cache_for("carddav-not-refused");
        let book = an_address_book(&cache);
        cache
            .save_contact(&made_here("Ann Example", "ann@example.com"))
            .expect("she saves");
        let server = Scripted {
            writes: Writes::Accepted,
            ..Scripted::default()
        };

        let result = sync_carddav_address_book(&cache, &server, &book)
            .await
            .expect("the sync");

        // That something really was sent is asserted first. Without it this
        // passes against a sync that sends nothing at all, which is what it did
        // in the red half: "nothing was refused" and "nothing was attempted"
        // are the same silence.
        assert_eq!(result.created_remote.count(), 1);
        // That something really was sent is asserted first. Without it this
        // passes against a sync that sends nothing at all, which is what it did
        // in the red half: "nothing was refused" and "nothing was attempted"
        // are the same silence.
        assert_eq!(result.created_remote.count(), 1);
        assert_eq!(result.waiting_on_the_setting.count(), 0);
        let said = crate::application::contacts_sync::what_the_contacts_sync_did(&result);
        assert!(
            !said.contains("Allow Changes"),
            "a sync that sent everything told somebody to turn on a setting: {said}"
        );
    }

    // ── Reading what the address book has ────────────────────────────────

    #[tokio::test]
    async fn test_a_card_this_address_book_has_never_seen_becomes_a_contact_here() {
        let (_paths, cache) = cache_for("carddav-new-card");
        let book = an_address_book(&cache);
        let server = Scripted {
            marker: Some("\"ctag-1\"".to_string()),
            cards: vec![a_card_for(
                "https://dav.example.com/books/work/bob.vcf",
                "\"v1\"",
                "Bob Example",
                "bob@example.com",
            )],
            ..Scripted::default()
        };

        let result = sync_carddav_address_book(&cache, &server, &book)
            .await
            .expect("the sync");

        assert_eq!(result.created_local.count(), 1);
        let here = cache.get_contacts_for_account("a1").expect("the contacts");
        assert_eq!(here.len(), 1);
        assert_eq!(
            here[0].id_in(&book.the_word_its_contacts_are_filed_under()),
            Some("https://dav.example.com/books/work/bob.vcf")
        );
    }

    #[tokio::test]
    async fn test_a_contact_changed_at_the_address_book_and_not_here_is_taken() {
        let (_paths, cache) = cache_for("carddav-theirs-wins");
        let book = an_address_book(&cache);
        let filed_under = book.the_word_its_contacts_are_filed_under();
        let card_url = "https://dav.example.com/books/work/ann.vcf";
        let here = made_here("Ann Example", "ann@example.com")
            .also_known_to(filed_under.clone(), card_url, Some("\"v1\""))
            .told(&filed_under, Some("\"v1\""));
        cache.save_contact(&here).expect("she saves");
        let server = Scripted {
            marker: Some("\"ctag-2\"".to_string()),
            cards: vec![a_card_for(
                card_url,
                "\"v2\"",
                "Ann Elsewhere",
                "ann@example.com",
            )],
            ..Scripted::default()
        };

        let result = sync_carddav_address_book(&cache, &server, &book)
            .await
            .expect("the sync");

        assert_eq!(result.updated_local.count(), 1);
        let stored = cache.get_contacts_for_account("a1").expect("the contacts");
        assert_eq!(stored.len(), 1, "a second row was made for one person");
        assert_eq!(stored[0].name, "Ann Elsewhere");
    }

    #[tokio::test]
    async fn test_a_contact_changed_only_here_keeps_what_is_here() {
        // The half a test that held everything would get wrong. Nothing is
        // held, nothing is written over, and the change is still owed.
        let (_paths, cache) = cache_for("carddav-mine-wins");
        let book = an_address_book(&cache);
        let filed_under = book.the_word_its_contacts_are_filed_under();
        let card_url = "https://dav.example.com/books/work/ann.vcf";
        let here = changed_here(
            "Ann Here",
            "ann@example.com",
            &filed_under,
            card_url,
            "\"v1\"",
        );
        cache.save_contact(&here).expect("she saves");
        let server = Scripted {
            marker: Some("\"ctag-2\"".to_string()),
            writes: Writes::Accepted,
            marker_after_a_write: Some("\"v2\"".to_string()),
            // What the address book holds once the push has gone in: this
            // computer's copy, under the marker the write gave it. A fake that
            // went on serving the old card would be a server that took a write
            // and did not keep it, and the read that follows would then read
            // its own push as somebody else's change.
            cards: vec![a_card_for(
                card_url,
                "\"v2\"",
                "Ann Here",
                "ann@example.com",
            )],
            ..Scripted::default()
        };

        let result = sync_carddav_address_book(&cache, &server, &book)
            .await
            .expect("the sync");

        // That the sync did anything at all is asserted first, for the reason
        // `test_a_sync_that_sent_everything_does_not_name_the_setting` gives:
        // "not held and not written over" is true of a sync that never ran.
        assert_eq!(result.updated_remote.count(), 1);
        // That the sync did anything at all is asserted first, for the reason
        // `test_a_sync_that_sent_everything_does_not_name_the_setting` gives:
        // "not held and not written over" is true of a sync that never ran.
        assert_eq!(result.updated_remote.count(), 1);
        assert_eq!(result.held_for_you_to_choose.count(), 0);
        // Neither copy moved, because the marker the push was given is the one
        // the read found. Counted as unchanged rather than as another update,
        // which is what says the push wrote down the marker it was handed.
        assert_eq!(result.unchanged.count(), 1);
        let stored = cache.get_contacts_for_account("a1").expect("the contacts");
        assert_eq!(stored[0].name, "Ann Here");
    }

    #[tokio::test]
    async fn test_a_contact_changed_in_both_places_is_held_for_somebody_to_choose() {
        // Both copies moved since the last sync and one of them is somebody's
        // own work. Neither is written over and neither is sent.
        let (_paths, cache) = cache_for("carddav-both-moved");
        let book = an_address_book(&cache);
        let filed_under = book.the_word_its_contacts_are_filed_under();
        let card_url = "https://dav.example.com/books/work/ann.vcf";
        let here = changed_here(
            "Ann Here",
            "ann@example.com",
            &filed_under,
            card_url,
            "\"v1\"",
        );
        cache.save_contact(&here).expect("she saves");
        let server = Scripted {
            marker: Some("\"ctag-2\"".to_string()),
            // The push is refused for an ordinary reason, so the edit is still
            // waiting when the read arrives with a marker that has moved on.
            writes: Writes::NothingIsSent,
            cards: vec![a_card_for(
                card_url,
                "\"v9\"",
                "Ann There",
                "ann@example.com",
            )],
            ..Scripted::default()
        };

        let result = sync_carddav_address_book(&cache, &server, &book)
            .await
            .expect("the sync");

        assert_eq!(result.held_for_you_to_choose.count(), 1);
        let stored = cache.get_contacts_for_account("a1").expect("the contacts");
        assert_eq!(
            stored[0].name, "Ann Here",
            "the address book's copy was written over a change made here"
        );
        assert_eq!(
            cache.conflicts_held_for("a1").expect("what is held").len(),
            1
        );
    }

    /// Whether that source really calls the shared decision.
    ///
    /// The call and not the name. Written as a search for the bare word, this
    /// was answered by this file's own module header, which says in prose that
    /// `whose_copy_wins` is where the decision lives: the assertion was
    /// satisfied by its own quotation and would have stayed green against a
    /// file that decided everything inline. `05.1-05` met the same shape and
    /// reworded a doc comment to get out of it.
    fn asks_the_shared_decision(shipped: &str) -> bool {
        shipped.contains("whose_copy_wins(") && shipped.contains("hold_both_copies_of(")
    }

    #[test]
    fn test_the_reading_that_says_the_shared_decision_is_asked_can_see_it_missing() {
        // The companion this project asks any source-reading guard to carry.
        assert!(asks_the_shared_decision(
            "whose_copy_wins(a, b, c); hold_both_copies_of(d)"
        ));
        assert!(
            !asks_the_shared_decision(
                "//! whose_copy_wins is where the decision lives. \
                 //! hold_both_copies_of is what holds it."
            ),
            "prose naming the shared decision reads as calling it"
        );
    }

    #[tokio::test]
    async fn test_nothing_here_decides_a_winner_of_its_own() {
        // The same answers the shared decision gives, asked of the shared
        // decision. If this file ever grew its own, this is what would stop
        // agreeing with it.
        assert_eq!(
            whose_copy_wins(false, Some("\"v2\""), Some("\"v1\"")),
            crate::application::contacts_sync::WhoseCopyWins::TakeTheAddressBooks
        );
        assert_eq!(
            whose_copy_wins(true, Some("\"v1\""), Some("\"v1\"")),
            crate::application::contacts_sync::WhoseCopyWins::KeepWhatIsHere
        );
        let shipped = crate::common::what_ships::what_ships(
            &std::fs::read_to_string("src/application/carddav_sync.rs").expect("this file"),
        );
        assert!(
            asks_the_shared_decision(&shipped),
            "the CardDAV sync no longer asks the shared decision who wins"
        );
    }

    // ── Deletions ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_a_contact_deleted_here_is_not_written_back_down() {
        // The case `application::deletions` is about: the address book still
        // names her, because its own list has not caught up. Written back down
        // she is on the screen again, under a new identifier, with nothing left
        // to say she was ever deleted.
        let (_paths, cache) = cache_for("carddav-deleted-here");
        let book = an_address_book(&cache);
        let filed_under = book.the_word_its_contacts_are_filed_under();
        let card_url = "https://dav.example.com/books/work/ann.vcf";
        let here = made_here("Ann Example", "ann@example.com")
            .also_known_to(filed_under.clone(), card_url, Some("\"v1\""))
            .told(&filed_under, Some("\"v1\""));
        cache.save_contact(&here).expect("she saves");
        cache.delete_contact(&here.id).expect("she is deleted");
        let server = Scripted {
            marker: Some("\"ctag-2\"".to_string()),
            writes: Writes::Accepted,
            cards: vec![a_card_for(
                card_url,
                "\"v1\"",
                "Ann Example",
                "ann@example.com",
            )],
            ..Scripted::default()
        };

        sync_carddav_address_book(&cache, &server, &book)
            .await
            .expect("the sync");

        assert!(
            cache
                .get_contacts_for_account("a1")
                .expect("the contacts")
                .is_empty(),
            "a contact deleted here came back"
        );
        assert_eq!(
            server.removed.lock().expect("removed").as_slice(),
            [card_url.to_string()]
        );
    }

    // ── Asking again only when something moved ───────────────────────────

    #[tokio::test]
    async fn test_the_cards_are_not_asked_for_again_when_the_change_marker_has_not_moved() {
        // What the marker is for. Reading every card on every sync is the whole
        // address book over the wire to learn that nothing happened.
        let (_paths, cache) = cache_for("carddav-marker-still");
        let mut book = an_address_book(&cache);
        book.ctag = Some("\"ctag-1\"".to_string());
        cache.save_address_book(&book).expect("the marker saves");
        let server = Scripted {
            marker: Some("\"ctag-1\"".to_string()),
            ..Scripted::default()
        };

        sync_carddav_address_book(&cache, &server, &book)
            .await
            .expect("the sync");

        assert_eq!(*server.times_read.lock().expect("the count"), 0);
    }

    #[tokio::test]
    async fn test_the_cards_are_asked_for_when_the_change_marker_has_moved() {
        let (_paths, cache) = cache_for("carddav-marker-moved");
        let mut book = an_address_book(&cache);
        book.ctag = Some("\"ctag-1\"".to_string());
        cache.save_address_book(&book).expect("the marker saves");
        let server = Scripted {
            marker: Some("\"ctag-2\"".to_string()),
            ..Scripted::default()
        };

        sync_carddav_address_book(&cache, &server, &book)
            .await
            .expect("the sync");

        assert_eq!(*server.times_read.lock().expect("the count"), 1);
        let stored = cache
            .get_address_books_for_account("a1")
            .expect("the address books");
        assert_eq!(
            stored[0].ctag,
            Some("\"ctag-2\"".to_string()),
            "the marker this sync saw was not written down, so the next one reads everything again"
        );
    }

    #[tokio::test]
    async fn test_a_read_that_failed_does_not_move_the_change_marker_on() {
        // The marker says what this computer has seen. Moved on after a read
        // that failed, everything that changed in between is never asked for
        // again.
        let (_paths, cache) = cache_for("carddav-read-failed");
        let mut book = an_address_book(&cache);
        book.ctag = Some("\"ctag-1\"".to_string());
        cache.save_address_book(&book).expect("the marker saves");
        let server = Scripted {
            marker: Some("\"ctag-2\"".to_string()),
            unreadable: true,
            ..Scripted::default()
        };

        let sync = sync_carddav_address_book(&cache, &server, &book).await;

        assert!(sync.is_err(), "a read that failed reported a clean sync");
        let stored = cache
            .get_address_books_for_account("a1")
            .expect("the address books");
        assert_eq!(stored[0].ctag, Some("\"ctag-1\"".to_string()));
    }

    #[tokio::test]
    async fn test_a_card_that_could_not_be_read_is_counted_rather_than_dropped() {
        let (_paths, cache) = cache_for("carddav-unreadable-card");
        let book = an_address_book(&cache);
        let server = Scripted {
            marker: Some("\"ctag-1\"".to_string()),
            could_not_be_read: 2,
            ..Scripted::default()
        };

        let result = sync_carddav_address_book(&cache, &server, &book)
            .await
            .expect("the sync");

        assert_eq!(result.errors.len(), 1, "{:?}", result.errors);
        assert!(
            result.errors[0].contains('2'),
            "how many could not be read was not said: {:?}",
            result.errors
        );
    }
}

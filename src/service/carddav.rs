//! What a CardDAV server exchanges, read and written without a server.
//!
//! CardDAV is how an address book lives on a server: a collection of cards at
//! an address, asked about with a PROPFIND and read with a REPORT, each card
//! written in the same vCard format a `.vcf` file uses. This file holds the
//! request bodies and the readers for the two answers, and nothing else.
//!
//! **Nothing here speaks to a network.** The transport lives elsewhere, the
//! way it does for the calendar, so that everything worth testing can be
//! tested from text. That is this project's thin transport rule, and it is
//! also what makes the tests below mean something: they are about parsing and
//! about the card format, and a machine with no account and no server can
//! settle every one of them.
//!
//! **The reading is a hand written scan and that is a security decision, not
//! a shortcut.** A general purpose reader for this kind of document will
//! resolve an entity declared in the document itself, and a server's answer is
//! a stranger's bytes: an entity pointing at a file on this computer turns
//! somebody's address book into a way to read it and send it on. A scan that
//! looks for elements by name cannot be made to fetch anything, because there
//! is nothing in it that fetches. The calendar's reader was written the same
//! way for the same reason and this agrees with it.
//!
//! **A card is read and written by the code a file import already uses.**
//! `MessageCache::contact_from_vcard_block` turns one card into one contact
//! and `MessageCache::vcard_block_from_contact` turns one contact into one
//! card. Neither is reimplemented here. Two answers to what a card looks like
//! is the defect this project has already paid for once in the calendar,
//! where a reader and the writer beside it disagreed and a cancellation went
//! back to a server with its own property name written in front of it twice.
//!
//! **Nothing here has met a server.** Every fixture below was written in this
//! repository, so a card written here and read back here proves that this
//! reader and this writer agree with each other. Whether a real server's
//! answer parses, and whether a card this writes is accepted by anybody
//! else's client, are two further questions and neither is answered here.

use crate::common::{Error, Result};
use crate::data::message_cache::{ContactEntry, MessageCache};
use crate::service::caldav::{extract_xml_value, resolved_against, response_blocks};

/// What this program asks a server when it wants to know which address books
/// are there.
///
/// Beside the reader of the answer on purpose, the way the calendar keeps its
/// own pair together. A request asking for one set of properties and a reader
/// looking for another is a silence nobody can see: the server answers
/// correctly, the reader finds nothing, and the address book reads as empty.
pub const ASKING_WHICH_ADDRESS_BOOKS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<d:propfind xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/" xmlns:card="urn:ietf:params:xml:ns:carddav">
  <d:prop>
    <d:displayname/>
    <d:resourcetype/>
    <cs:getctag/>
  </d:prop>
</d:propfind>"#;

/// One address book a server said it holds.
///
/// Its own type rather than the calendar's. The two will grow apart, and a
/// shared one would make each of them carry a field the other never fills.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardDavAddressBook {
    /// Where it lives, whole, so it can be asked for without being rebuilt.
    pub url: String,
    /// What the server calls it, or "Untitled" when it said nothing.
    pub display_name: String,
    /// The marker a server moves when anything in the address book changes,
    /// where the server gives one. Nothing where it does not.
    pub ctag: Option<String>,
}

/// The address books in a server's answer to [`ASKING_WHICH_ADDRESS_BOOKS`].
///
/// A hand written scan over the answer's response blocks, and that is a
/// security decision rather than a shortcut. A general purpose reader for this
/// kind of document resolves an entity the document declares, and this
/// document came from a server: an entity naming a file on this computer turns
/// somebody's address book into a way to read that file and send it onward.
/// This cannot be made to fetch anything, because there is nothing in it that
/// fetches. The calendar's reader was written the same way for the same reason
/// and this shares its parts rather than copying them.
///
/// An answer that is not a multistatus is refused with a sentence rather than
/// read as a server holding no address books. The two are different facts and a
/// screen that cannot tell them apart says "none found" to somebody whose
/// address is wrong. That follows `parse_report_events` and departs from
/// `parse_propfind_calendars`, which refuses nothing; the departure is
/// deliberate, so do not correct it back.
///
/// Two limits, said plainly rather than left to be discovered. The `d:` prefix
/// on the DAV elements is assumed, so a server that declares the DAV namespace
/// for unprefixed names is read as holding none; the calendar assumes the same
/// and this has never been tried against a real server either way. And the scan
/// for the address book element runs over the whole response block rather than
/// over the resource type alone, the way the calendar's does, so a server
/// sending a raw `<` inside a name it should have escaped could be read as
/// offering an address book it does not have. The cost of that is one extra row
/// in a list somebody chooses from.
pub fn address_books_in(xml: &str, base_url: &str) -> Result<Vec<CardDavAddressBook>> {
    if !xml.to_ascii_lowercase().contains("multistatus") {
        return Err(Error::Protocol(
            "That address did not answer with an address book. Nothing was added here.".to_string(),
        ));
    }

    let mut address_books = Vec::new();
    for block in response_blocks(xml) {
        let href = value_in(block, "d:href").unwrap_or_default();
        // An address book with nowhere to ask is worse than none: every later
        // request for it resolves the empty address against the base and goes
        // somewhere nobody chose.
        if href.is_empty() {
            continue;
        }
        if !names_an_address_book(block) {
            continue;
        }
        address_books.push(CardDavAddressBook {
            url: resolved_against(&href, base_url),
            display_name: value_in(block, "d:displayname")
                .unwrap_or_else(|| "Untitled".to_string()),
            ctag: value_in(block, "cs:getctag"),
        });
    }
    Ok(address_books)
}

/// Whether a block describes an address book rather than something else the
/// server keeps beside them.
///
/// The local name is matched and the namespace prefix is stepped over, because
/// the prefix is the document's own choice: one server writes
/// `card:addressbook`, another writes `CR:addressbook`, and a server that
/// declares the CardDAV namespace for unprefixed names writes `addressbook`.
/// The calendar's reader matches two spellings and records in its own doc that
/// a third reads as offering none, which is a gap this does not repeat.
///
/// The character after the name has to end it, so `addressbook-home-set`, which
/// the standard also defines, is not read as an address book.
fn names_an_address_book(block: &str) -> bool {
    const NAME: &str = "addressbook";
    block.match_indices(NAME).any(|(at, _)| {
        an_element_opens_at(block, at) && ends_a_name(block[at + NAME.len()..].chars().next())
    })
}

/// Whether an element name begins at that position: a `<` immediately before
/// it, or a namespace prefix and its colon with a `<` in front of those.
fn an_element_opens_at(text: &str, name_at: usize) -> bool {
    let before = &text[..name_at];
    if before.ends_with('<') {
        return true;
    }
    let Some(before_colon) = before.strip_suffix(':') else {
        return false;
    };
    match before_colon.rfind('<') {
        Some(opening) => {
            let prefix = &before_colon[opening + 1..];
            !prefix.is_empty() && prefix.chars().all(a_name_character)
        }
        None => false,
    }
}

/// Whether that character ends an element name rather than continuing it.
fn ends_a_name(next: Option<char>) -> bool {
    matches!(next, Some('/') | Some('>')) || next.is_some_and(char::is_whitespace)
}

/// Whether that character may appear in a namespace prefix.
fn a_name_character(letter: char) -> bool {
    letter.is_ascii_alphanumeric() || matches!(letter, '-' | '_' | '.')
}

/// What this program asks for the cards in one address book.
///
/// Beside its reader for the reason [`ASKING_WHICH_ADDRESS_BOOKS`] is beside
/// its own.
pub const ASKING_FOR_THE_CARDS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<card:addressbook-query xmlns:d="DAV:" xmlns:card="urn:ietf:params:xml:ns:carddav">
  <d:prop>
    <d:getetag/>
    <card:address-data/>
  </d:prop>
</card:addressbook-query>"#;

/// One card a server holds, and the contact it says.
#[derive(Debug, Clone)]
pub struct CardOnAServer {
    /// Where this one card lives, whole.
    pub url: String,
    /// The marker the server moves when this card changes, where it gives one.
    /// Nothing where it does not, and an empty one is the same answer, which is
    /// decided once at [`CardDavAddressBook::ctag`] for both markers.
    pub version: Option<String>,
    /// What the card says, read by the same code a file import uses. The
    /// card's own text is on `contact.vcard_raw`, put there by that reader, so
    /// it is not carried here a second time.
    pub contact: ContactEntry,
}

/// What a server sent when asked for the cards in an address book.
#[derive(Debug, Clone, Default)]
pub struct CardsFromAServer {
    /// The cards that could be read, in the order the server sent them.
    pub cards: Vec<CardOnAServer>,
    /// How many arrived that could not be read.
    ///
    /// Counted rather than dropped in silence, and not turned into a failure
    /// here. An address book that really is empty and one whose cards could
    /// none of them be read are different facts, and the second is the one
    /// somebody goes looking for a broken program over. The file import makes
    /// the same distinction, and whoever calls this has the count to say it
    /// with.
    pub could_not_be_read: usize,
}

/// The cards in a server's answer to [`ASKING_FOR_THE_CARDS`].
///
/// A hand written scan over response blocks, for the reason
/// [`address_books_in`] gives, and every card goes through
/// `MessageCache::contact_from_vcard_block`, which is the same code a file
/// import uses. Nothing here reads a property off a card.
pub fn cards_in(xml: &str, base_url: &str, account_id: &str) -> Result<CardsFromAServer> {
    if !xml.to_ascii_lowercase().contains("multistatus") {
        return Err(Error::Protocol(
            "That address did not answer with the cards in an address book. \
             Nothing here was changed."
                .to_string(),
        ));
    }

    let mut sent = CardsFromAServer::default();
    for block in response_blocks(xml) {
        let Some(card) = the_card_in(block) else {
            continue;
        };
        let Some(contact) = MessageCache::contact_from_vcard_block(account_id, &card) else {
            sent.could_not_be_read += 1;
            continue;
        };
        let href = value_in(block, "d:href").unwrap_or_default();
        sent.cards.push(CardOnAServer {
            // Nothing rather than the address book's own address when the
            // server said none. An empty address resolved against the base is
            // the collection itself, and a change written there is written
            // over every card in it.
            url: match href.is_empty() {
                true => String::new(),
                false => resolved_against(&href, base_url),
            },
            version: value_in(block, "d:getetag"),
            contact,
        });
    }
    Ok(sent)
}

/// The card's own text in a response block.
///
/// Three spellings of the element are tried, because the namespace prefix is
/// the document's own choice and this cannot step over it the way
/// [`names_an_address_book`] does: an element's value is taken by name here,
/// not found by scanning. A server writing a fourth spelling is read as having
/// sent no card, which is a real gap and has never been tried against one.
fn the_card_in(block: &str) -> Option<String> {
    ["card:address-data", "C:address-data", "address-data"]
        .iter()
        .find_map(|named| value_in(block, named))
}

/// One element's value, with XML's own escaping taken off.
///
/// Everything this file takes out of a document comes through here, so a value
/// reaches the rest of the program as the text it is rather than as the text
/// XML needed to carry it. An address book called `Sam &amp; Co` is read out to
/// somebody, and that is not a name.
///
/// The calendar's reader does not do this: it hands the document it takes out
/// straight to the calendar parser. That is a gap there rather than a decision,
/// and this plan does not reach into that file to fix it.
fn value_in(block: &str, tag: &str) -> Option<String> {
    extract_xml_value(block, tag).map(|value| xml_unescaped(&value))
}

/// Text with the five references XML defines turned back into the characters
/// they stand for.
///
/// Only those five, and only once. A reference this does not know is left
/// exactly as it arrived, which is what keeps an entity a document declared for
/// itself from ever being expanded: the document may declare what it likes and
/// this reads the declaration as text and the reference as text. Reading once
/// rather than until nothing changes is the other half of that, so a document
/// writing `&amp;lt;` gets back `&lt;` and not `<`.
fn xml_unescaped(text: &str) -> String {
    const REFERENCES: [(&str, char); 5] = [
        ("&amp;", '&'),
        ("&lt;", '<'),
        ("&gt;", '>'),
        ("&quot;", '"'),
        ("&apos;", '\''),
    ];
    let mut plain = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        plain.push_str(&rest[..at]);
        let from_the_ampersand = &rest[at..];
        match REFERENCES
            .iter()
            .find(|(reference, _)| from_the_ampersand.starts_with(reference))
        {
            Some((reference, letter)) => {
                plain.push(*letter);
                rest = &from_the_ampersand[reference.len()..];
            }
            None => {
                plain.push('&');
                rest = &from_the_ampersand[1..];
            }
        }
    }
    plain.push_str(rest);
    plain
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{AddressEntry, ContactEntry, CustomFieldEntry, MessageCache};

    const SOMEBODY: &str = "sam@example.com";

    /// A contact with an address and a name and nothing else filled in, so a
    /// test says only what it is about.
    fn a_contact(name: &str) -> ContactEntry {
        ContactEntry {
            id: "contact-1".to_string(),
            account_id: SOMEBODY.to_string(),
            name: name.to_string(),
            given_name: None,
            family_name: None,
            email: "grace@example.com".to_string(),
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
            created_at: "2026-09-10T00:00:00Z".to_string(),
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

    /// One custom field, which is the structured property easiest to fill in
    /// with a value somebody really typed.
    fn one_custom_field(label: &str, value: &str) -> String {
        serde_json::to_string(&vec![CustomFieldEntry {
            label: label.to_string(),
            value: value.to_string(),
        }])
        .expect("a list of one field to be written")
    }

    /// One postal address, which is the other structured property a person
    /// fills in by hand.
    fn one_address(street: &str) -> String {
        serde_json::to_string(&vec![AddressEntry {
            label: "Home".to_string(),
            street: street.to_string(),
            city: "Newcastle".to_string(),
            state: "Tyne and Wear".to_string(),
            zip: "NE1 1AA".to_string(),
            country: "United Kingdom".to_string(),
        }])
        .expect("a list of one address to be written")
    }

    /// The contact a card comes back as, or a failure saying the card was
    /// turned away.
    fn read_back(card: &str) -> ContactEntry {
        MessageCache::contact_from_vcard_block(SOMEBODY, card)
            .expect("the card this code wrote to be read back")
    }

    /// The one address in a contact's list of them.
    fn the_one_address(contact: &ContactEntry) -> AddressEntry {
        let json = contact
            .addresses_json
            .as_deref()
            .expect("the contact to carry an address");
        serde_json::from_str::<Vec<AddressEntry>>(json)
            .expect("the addresses to be readable")
            .first()
            .cloned()
            .expect("one address")
    }

    /// The one custom field in a contact's list of them.
    fn the_one_custom_field(contact: &ContactEntry) -> CustomFieldEntry {
        let json = contact
            .custom_fields_json
            .as_deref()
            .expect("the contact to carry a custom field");
        serde_json::from_str::<Vec<CustomFieldEntry>>(json)
            .expect("the custom fields to be readable")
            .first()
            .cloned()
            .expect("one custom field")
    }

    #[test]
    fn test_a_card_this_code_wrote_is_read_back_as_the_contact_it_was() {
        let mut she = a_contact("Grace Hopper");
        she.nickname = Some("Amazing Grace".to_string());
        she.phone = Some("+44 191 496 0000".to_string());
        she.notes = Some("Met at the harbour".to_string());
        she.company = Some("Univac".to_string());
        she.department = Some("Research".to_string());
        she.custom_fields_json = Some(one_custom_field("Blood type", "O negative"));

        let card = MessageCache::vcard_block_from_contact(&she);
        let back = read_back(&card);

        assert_eq!(back.name, "Grace Hopper");
        assert_eq!(back.email, "grace@example.com");
        assert_eq!(back.nickname.as_deref(), Some("Amazing Grace"));
        assert_eq!(back.phone.as_deref(), Some("+44 191 496 0000"));
        assert_eq!(back.notes.as_deref(), Some("Met at the harbour"));
        assert_eq!(back.company.as_deref(), Some("Univac"));
        assert_eq!(back.department.as_deref(), Some("Research"));
        let field = the_one_custom_field(&back);
        assert_eq!(field.label, "Blood type");
        assert_eq!(field.value, "O negative");
    }

    #[test]
    fn test_a_value_too_long_for_one_line_is_folded_and_comes_back_whole() {
        // Seventy characters, then a space, so the fold lands exactly beside
        // it: `NOTE:` is five octets and the first piece of a folded line is
        // seventy five. That is the case which once ran two words together,
        // because unfolding took off every space at the join rather than the
        // one the fold put there, and a county of "Tyne and Wear" came back
        // "Tyneand Wear".
        let note = format!("{} {}", "w".repeat(70), "and what she said after it");
        let mut she = a_contact("Grace Hopper");
        she.notes = Some(note.clone());

        let card = MessageCache::vcard_block_from_contact(&she);

        assert!(
            card.contains("\r\n "),
            "this test proves nothing unless the writer really folded: {card}"
        );
        assert_eq!(read_back(&card).notes.as_deref(), Some(note.as_str()));
    }

    #[test]
    fn test_a_comma_is_escaped_in_a_plain_text_property_and_in_a_structured_one() {
        // A comma separates one value from the next in this format, so a
        // comma somebody typed is escaped on the way out. A round trip alone
        // would not notice the escape being dropped from a plain text
        // property, because nothing splits an unescaped comma there, which is
        // why the card text is asserted on as well.
        let mut she = a_contact("Grace Hopper");
        she.notes = Some("Numbers, then words".to_string());
        she.custom_fields_json = Some(one_custom_field("Seen at", "Newcastle, twice"));

        let card = MessageCache::vcard_block_from_contact(&she);

        assert!(
            card.contains("NOTE:Numbers\\, then words"),
            "the comma in a plain text property was not escaped: {card}"
        );
        assert!(
            card.contains("Newcastle\\, twice"),
            "the comma in a structured property was not escaped: {card}"
        );
        let back = read_back(&card);
        assert_eq!(back.notes.as_deref(), Some("Numbers, then words"));
        assert_eq!(the_one_custom_field(&back).value, "Newcastle, twice");
    }

    #[test]
    fn test_a_semicolon_is_escaped_so_a_structured_field_does_not_become_two() {
        // A semicolon ends a field inside a structured property, so one left
        // unescaped in a street turns a street into two fields and shifts the
        // town, county, postcode and country each one along, dropping the
        // country off the end. That regression is what the doc comment on
        // `structured_parts` records.
        let mut she = a_contact("Grace Hopper");
        she.notes = Some("First; then second".to_string());
        she.addresses_json = Some(one_address("12 High Street; Flat 2"));

        let card = MessageCache::vcard_block_from_contact(&she);

        assert!(
            card.contains("NOTE:First\\; then second"),
            "the semicolon in a plain text property was not escaped: {card}"
        );
        assert!(
            card.contains("12 High Street\\; Flat 2"),
            "the semicolon in a structured property was not escaped: {card}"
        );
        let back = read_back(&card);
        assert_eq!(back.notes.as_deref(), Some("First; then second"));
        let address = the_one_address(&back);
        assert_eq!(address.street, "12 High Street; Flat 2");
        assert_eq!(address.city, "Newcastle");
        assert_eq!(address.country, "United Kingdom");
    }

    #[test]
    fn test_a_backslash_is_doubled_on_the_way_out_and_comes_back_single() {
        // A backslash begins an escape, so one somebody typed is doubled. Left
        // alone, "C:\new" comes back as "C:" and then a line break and then
        // "ew", because the reader takes the backslash and the letter after it
        // as an escape for a line break.
        let mut she = a_contact("Grace Hopper");
        she.notes = Some("Kept in C:\\new folder".to_string());
        she.custom_fields_json = Some(one_custom_field("Path", "D:\\notes\\old"));

        let card = MessageCache::vcard_block_from_contact(&she);

        assert!(
            card.contains("Kept in C:\\\\new folder"),
            "the backslash in a plain text property was not doubled: {card}"
        );
        assert!(
            card.contains("D:\\\\notes\\\\old"),
            "the backslash in a structured property was not doubled: {card}"
        );
        let back = read_back(&card);
        assert_eq!(back.notes.as_deref(), Some("Kept in C:\\new folder"));
        assert_eq!(the_one_custom_field(&back).value, "D:\\notes\\old");
    }

    #[test]
    fn test_a_line_break_in_a_value_is_written_as_an_escape_rather_than_a_new_line() {
        // A line break left alone ends the property and the rest of the value
        // becomes a line naming no property at all, which the reader passes
        // over. The value comes back with its tail silently missing.
        let mut she = a_contact("Grace Hopper");
        she.notes = Some("First line\nsecond line".to_string());
        she.custom_fields_json = Some(one_custom_field("Directions", "Left\nthen right"));

        let card = MessageCache::vcard_block_from_contact(&she);

        assert!(
            card.contains("NOTE:First line\\nsecond line"),
            "the line break in a plain text property was not escaped: {card}"
        );
        assert!(
            card.contains("Left\\nthen right"),
            "the line break in a structured property was not escaped: {card}"
        );
        let back = read_back(&card);
        assert_eq!(back.notes.as_deref(), Some("First line\nsecond line"));
        assert_eq!(the_one_custom_field(&back).value, "Left\nthen right");
    }

    #[test]
    fn test_a_card_holding_a_property_this_build_does_not_know_keeps_the_ones_it_does() {
        // Somebody else's client is free to write properties this one has
        // never heard of, and a card carrying one is an ordinary card rather
        // than a broken one.
        let card = "BEGIN:VCARD\r\n\
                    VERSION:3.0\r\n\
                    FN:Grace Hopper\r\n\
                    X-SOMETHING-ELSE:whatever this means\r\n\
                    EMAIL:grace@example.com\r\n\
                    NOTE:Met at the harbour\r\n\
                    END:VCARD\r\n";

        let back = read_back(card);

        assert_eq!(back.name, "Grace Hopper");
        assert_eq!(back.email, "grace@example.com");
        assert_eq!(back.notes.as_deref(), Some("Met at the harbour"));
    }

    #[test]
    fn test_a_card_naming_no_address_this_program_could_write_to_is_turned_away() {
        // The one rule that turns a card away. A contact with no address is a
        // contact nothing here can write to, and a card is where this program
        // learns the address.
        let no_address = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Grace Hopper\r\nEND:VCARD\r\n";
        let not_an_address =
            "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Grace Hopper\r\nEMAIL:grace\r\nEND:VCARD\r\n";

        assert!(MessageCache::contact_from_vcard_block(SOMEBODY, no_address).is_none());
        assert!(MessageCache::contact_from_vcard_block(SOMEBODY, not_an_address).is_none());
    }

    #[test]
    fn test_a_card_with_no_name_on_it_is_read_rather_than_turned_away() {
        // A card with no FN is imported and named from its address, so the
        // reader answers with a contact whose name is empty rather than with
        // nothing at all. Asserting the opposite would fail against correct
        // behaviour: three tests beside the file import already pin this.
        let card = "BEGIN:VCARD\r\nVERSION:3.0\r\nEMAIL:grace@example.com\r\nEND:VCARD\r\n";

        let back = read_back(card);

        assert_eq!(back.name, "");
        assert_eq!(back.email, "grace@example.com");
    }

    // ── A server's answer about which address books it has ──────────────
    //
    // Every fixture below is written the way a server really answers: a
    // multistatus carrying response blocks, each with an href, a propstat, a
    // prop, a resourcetype and a displayname. A fixture built by printing what
    // this code would produce tests nothing at all.

    const AT: &str = "https://dav.example.com/carddav/sam/";

    /// A whole answer with those blocks in it.
    fn a_multistatus(blocks: &str) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <d:multistatus xmlns:d=\"DAV:\" \
             xmlns:card=\"urn:ietf:params:xml:ns:carddav\" \
             xmlns:cs=\"http://calendarserver.org/ns/\">\n\
             {blocks}</d:multistatus>\n"
        )
    }

    /// One block describing an address book, with the resource type saying so.
    fn an_address_book(href: &str, name: &str, ctag: &str) -> String {
        one_block(
            href,
            name,
            "<d:collection/><card:addressbook/>",
            &format!("<cs:getctag>{ctag}</cs:getctag>"),
        )
    }

    /// One block describing something else the server keeps beside them.
    fn a_calendar(href: &str, name: &str) -> String {
        one_block(href, name, "<d:collection/><c:calendar/>", "")
    }

    fn one_block(href: &str, name: &str, resource_type: &str, extra: &str) -> String {
        format!(
            "  <d:response>\n    \
             <d:href>{href}</d:href>\n    \
             <d:propstat>\n      \
             <d:prop>\n        \
             <d:displayname>{name}</d:displayname>\n        \
             <d:resourcetype>{resource_type}</d:resourcetype>\n        \
             {extra}\n      \
             </d:prop>\n      \
             <d:status>HTTP/1.1 200 OK</d:status>\n    \
             </d:propstat>\n  \
             </d:response>\n"
        )
    }

    #[test]
    fn test_two_address_books_come_back_with_their_addresses_and_their_names() {
        let answer = a_multistatus(&format!(
            "{}{}",
            an_address_book("/carddav/sam/contacts/", "Contacts", "12"),
            an_address_book("/carddav/sam/work/", "Work", "34")
        ));

        let found = address_books_in(&answer, AT).expect("a multistatus to be read");

        assert_eq!(found.len(), 2);
        assert_eq!(
            found[0].url,
            "https://dav.example.com/carddav/sam/contacts/"
        );
        assert_eq!(found[0].display_name, "Contacts");
        assert_eq!(found[1].url, "https://dav.example.com/carddav/sam/work/");
        assert_eq!(found[1].display_name, "Work");
    }

    #[test]
    fn test_a_collection_that_is_not_an_address_book_is_left_out() {
        // One of each in the same answer, and both the count and the identity
        // asserted. Counting alone is green against a reader that returns
        // nothing at all, which is the reader somebody writes by accident.
        let answer = a_multistatus(&format!(
            "{}{}",
            a_calendar("/carddav/sam/diary/", "Diary"),
            an_address_book("/carddav/sam/contacts/", "Contacts", "12")
        ));

        let found = address_books_in(&answer, AT).expect("a multistatus to be read");

        assert_eq!(found.len(), 1, "only the address book is one");
        assert_eq!(found[0].display_name, "Contacts");
    }

    #[test]
    fn test_a_block_with_no_address_is_left_out() {
        // An address book with nowhere to ask is worse than none: every
        // request for it would go to whatever the empty address resolves to.
        let answer = a_multistatus(&format!(
            "{}{}",
            an_address_book("", "Nowhere", "12"),
            an_address_book("/carddav/sam/contacts/", "Contacts", "34")
        ));

        let found = address_books_in(&answer, AT).expect("a multistatus to be read");

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].display_name, "Contacts");
    }

    #[test]
    fn test_an_answer_that_is_not_a_multi_status_is_refused_rather_than_read_as_empty() {
        // A captive portal, a sign in page or a proxy's error page arrives
        // with an ordinary 200 and holds no response block. Read as an answer
        // it says the server has no address books, and somebody whose address
        // is wrong is told their address book is empty.
        let page = "<html><body>Please sign in to the network</body></html>";

        let refusal = address_books_in(page, AT).expect_err("a page that is not an answer");

        let said = refusal.to_string();
        assert!(
            said.contains("did not answer with an address book"),
            "the refusal says what went wrong: {said}"
        );
    }

    #[test]
    fn test_an_address_book_named_with_a_prefix_this_code_does_not_expect_is_still_found() {
        // A namespace prefix is the document's own choice. Apple's server
        // writes `card:`, SabreDAV writes `card:` too, and Radicale writes
        // `CR:`. A server may also declare the CardDAV namespace as the one
        // unprefixed names belong to, so the element arrives with a bare name.
        // A reader that matches one spelling reads every other server's answer
        // as an empty list.
        let answer = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <multistatus xmlns=\"DAV:\" xmlns:CR=\"urn:ietf:params:xml:ns:carddav\">\n  \
             <d:response>\n    \
             <d:href>/carddav/sam/contacts/</d:href>\n    \
             <d:prop>\n      \
             <d:displayname>Contacts</d:displayname>\n      \
             <d:resourcetype><collection/><CR:addressbook/></d:resourcetype>\n    \
             </d:prop>\n  \
             </d:response>\n\
             </multistatus>\n";

        let found = address_books_in(answer, AT).expect("a multistatus to be read");

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].display_name, "Contacts");
    }

    #[test]
    fn test_an_entity_in_the_answer_is_neither_fetched_nor_expanded() {
        // The fixture really carries a declaration and a reference to it, or
        // it would pass against a reader that expands one.
        let answer = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE d:multistatus [\n  \
             <!ENTITY somewhere SYSTEM \"file:///c:/windows/win.ini\">\n\
             ]>\n\
             <d:multistatus xmlns:d=\"DAV:\" \
             xmlns:card=\"urn:ietf:params:xml:ns:carddav\">\n{}\
             </d:multistatus>\n",
            one_block(
                "/carddav/sam/contacts/",
                "&somewhere;",
                "<d:collection/><card:addressbook/>",
                ""
            )
        );

        let found = address_books_in(&answer, AT).expect("a multistatus to be read");

        assert_eq!(found.len(), 1);
        assert_eq!(
            found[0].display_name, "&somewhere;",
            "the reference came through as the text it is, unexpanded"
        );
        assert!(
            !found[0].display_name.contains("fonts"),
            "nothing on this computer was read"
        );
    }

    #[test]
    fn test_an_address_books_change_marker_is_kept_where_the_server_gives_one() {
        let answer = a_multistatus(&an_address_book("/carddav/sam/contacts/", "Contacts", "99"));

        let found = address_books_in(&answer, AT).expect("a multistatus to be read");

        assert_eq!(found[0].ctag.as_deref(), Some("99"));
    }

    #[test]
    fn test_a_change_marker_that_is_empty_and_one_that_is_absent_are_the_same_answer() {
        // Decided in writing rather than discovered later: an empty marker and
        // an absent one both come back as nothing. The one extractor this
        // program has cannot tell them apart, a server sending an empty marker
        // is sending no useful marker, and both mean the same to whoever asks
        // next. Growing a second extractor to tell them apart would be a
        // second thing to keep working for a distinction nobody acts on. The
        // same answer is given to a card's version marker.
        let empty = a_multistatus(&an_address_book("/carddav/sam/contacts/", "Contacts", ""));
        let absent = a_multistatus(&one_block(
            "/carddav/sam/contacts/",
            "Contacts",
            "<d:collection/><card:addressbook/>",
            "",
        ));

        let from_empty = address_books_in(&empty, AT).expect("a multistatus to be read");
        let from_absent = address_books_in(&absent, AT).expect("a multistatus to be read");

        assert_eq!(from_empty[0].ctag, None);
        assert_eq!(from_absent[0].ctag, None);
    }

    #[test]
    fn test_the_request_asks_for_every_property_the_reader_reads() {
        // The pair that cannot be allowed to drift. A request that stops
        // asking for a property leaves the reader looking for something the
        // server was never asked to send, and the answer is a silence rather
        // than a failure.
        for property in ["displayname", "resourcetype", "getctag"] {
            assert!(
                ASKING_WHICH_ADDRESS_BOOKS.contains(property),
                "the request does not ask for {property}, which the reader reads"
            );
        }
    }

    // ── A server's answer with the cards in an address book ─────────────

    /// One response block carrying a card, the way a server sends it: the
    /// card's text is XML escaped inside the element.
    fn a_card_block(href: &str, etag: Option<&str>, card: &str) -> String {
        let marker = match etag {
            None => String::new(),
            Some(given) => format!("<d:getetag>{given}</d:getetag>"),
        };
        format!(
            "  <d:response>\n    \
             <d:href>{href}</d:href>\n    \
             <d:propstat>\n      \
             <d:prop>\n        \
             {marker}\n        \
             <card:address-data>{card}</card:address-data>\n      \
             </d:prop>\n      \
             <d:status>HTTP/1.1 200 OK</d:status>\n    \
             </d:propstat>\n  \
             </d:response>\n"
        )
    }

    /// A card as a server would carry it in an answer: real line breaks, and
    /// the characters XML reserves written as references.
    fn a_readable_card(name: &str, address: &str, note: &str) -> String {
        format!("BEGIN:VCARD\nVERSION:3.0\nFN:{name}\nEMAIL:{address}\nNOTE:{note}\nEND:VCARD\n")
    }

    #[test]
    fn test_an_answer_holding_two_cards_is_read_into_two_contacts() {
        let answer = a_multistatus(&format!(
            "{}{}",
            a_card_block(
                "/carddav/sam/contacts/grace.vcf",
                Some("\"one\""),
                &a_readable_card("Grace Hopper", "grace@example.com", "Met at the harbour")
            ),
            a_card_block(
                "/carddav/sam/contacts/ada.vcf",
                Some("\"two\""),
                &a_readable_card("Ada Lovelace", "ada@example.com", "Wrote it down first")
            )
        ));

        let sent = cards_in(&answer, AT, SOMEBODY).expect("a multistatus to be read");

        assert_eq!(sent.cards.len(), 2);
        assert_eq!(sent.could_not_be_read, 0);
        assert_eq!(
            sent.cards[0].url,
            "https://dav.example.com/carddav/sam/contacts/grace.vcf"
        );
        assert_eq!(sent.cards[0].contact.name, "Grace Hopper");
        assert_eq!(sent.cards[0].contact.email, "grace@example.com");
        assert_eq!(sent.cards[1].contact.name, "Ada Lovelace");
    }

    #[test]
    fn test_a_cards_text_goes_through_the_reader_a_file_import_uses() {
        // Two things no reader written here would get right, and both are
        // ordinary: a line the format broke in two, and a semicolon inside a
        // structured field. Getting them back proves the card went through the
        // shared reader rather than through something written beside it.
        let street = "12 High Street\\; Flat 2";
        // Two spaces after the break: the format's own continuation space, and
        // then the space that belongs to "United Kingdom". Unfolding takes off
        // exactly one, which is the rule that once ran two words together here.
        let card = format!(
            "BEGIN:VCARD\nVERSION:3.0\nFN:Grace Hopper\nEMAIL:grace@example.com\n\
             ADR;TYPE=HOME:;;{street};Newcastle;Tyne and Wear;NE1 1AA;United\n  Kingdom\n\
             END:VCARD\n"
        );
        let answer = a_multistatus(&a_card_block(
            "/carddav/sam/contacts/grace.vcf",
            Some("\"1\""),
            &card,
        ));

        let sent = cards_in(&answer, AT, SOMEBODY).expect("a multistatus to be read");

        let address = the_one_address(&sent.cards[0].contact);
        assert_eq!(address.street, "12 High Street; Flat 2");
        assert_eq!(address.country, "United Kingdom");
    }

    #[test]
    fn test_a_card_that_cannot_be_read_is_counted_while_the_others_come_back() {
        // Paired on purpose. A test with only the unreadable card in it is
        // green against a reader that passes over everything.
        let answer = a_multistatus(&format!(
            "{}{}",
            a_card_block(
                "/carddav/sam/contacts/nobody.vcf",
                Some("\"one\""),
                "BEGIN:VCARD\nVERSION:3.0\nFN:Nobody\nEND:VCARD\n"
            ),
            a_card_block(
                "/carddav/sam/contacts/grace.vcf",
                Some("\"two\""),
                &a_readable_card("Grace Hopper", "grace@example.com", "Met at the harbour")
            )
        ));

        let sent = cards_in(&answer, AT, SOMEBODY).expect("a multistatus to be read");

        assert_eq!(sent.cards.len(), 1, "the readable card came back");
        assert_eq!(sent.cards[0].contact.name, "Grace Hopper");
        assert_eq!(sent.could_not_be_read, 1, "the other was counted");
    }

    #[test]
    fn test_a_cards_version_marker_is_kept_as_the_server_gave_it() {
        // Kept byte for byte, quotation marks and all. A server compares what
        // it is handed back against what it sent, so a marker tidied on the way
        // in is a marker the server does not recognise on the way out.
        let answer = a_multistatus(&a_card_block(
            "/carddav/sam/contacts/grace.vcf",
            Some("\"abc-123\""),
            &a_readable_card("Grace Hopper", "grace@example.com", "Met at the harbour"),
        ));

        let sent = cards_in(&answer, AT, SOMEBODY).expect("a multistatus to be read");

        assert_eq!(sent.cards[0].version.as_deref(), Some("\"abc-123\""));
    }

    #[test]
    fn test_a_version_marker_that_is_empty_and_one_that_is_absent_are_the_same_answer() {
        // The same decision as the address book's change marker, taken once
        // for both rather than twice: an element that is there and holds
        // nothing, and an element that is not there, are one answer.
        let card = a_readable_card("Grace Hopper", "grace@example.com", "Met at the harbour");
        let empty = a_multistatus(&a_card_block(
            "/carddav/sam/contacts/grace.vcf",
            Some(""),
            &card,
        ));
        let absent = a_multistatus(&a_card_block(
            "/carddav/sam/contacts/grace.vcf",
            None,
            &card,
        ));

        let from_empty = cards_in(&empty, AT, SOMEBODY).expect("a multistatus to be read");
        let from_absent = cards_in(&absent, AT, SOMEBODY).expect("a multistatus to be read");

        assert_eq!(from_empty.cards[0].version, None);
        assert_eq!(from_absent.cards[0].version, None);
    }

    #[test]
    fn test_an_answer_that_is_not_a_multi_status_is_refused_when_asking_for_cards() {
        let page = "<html><body>Please sign in to the network</body></html>";

        let refusal = cards_in(page, AT, SOMEBODY).expect_err("a page that is not an answer");

        let said = refusal.to_string();
        assert!(
            said.contains("did not answer with"),
            "the refusal says what went wrong: {said}"
        );
    }

    #[test]
    fn test_card_text_arriving_with_escaped_xml_is_unescaped_before_the_card_reader_sees_it() {
        // A card containing an ampersand or an angle bracket arrives with
        // those characters written as references, because the card is text
        // inside an XML element. A fixture with plain card text passes against
        // a reader that does no unescaping at all.
        let card = a_readable_card(
            "Grace &amp; Ada",
            "grace@example.com",
            "Bells &amp; whistles, 3 &lt; 4",
        );
        let answer = a_multistatus(&a_card_block(
            "/carddav/sam/contacts/grace.vcf",
            Some("\"1\""),
            &card,
        ));

        let sent = cards_in(&answer, AT, SOMEBODY).expect("a multistatus to be read");

        assert_eq!(sent.cards[0].contact.name, "Grace & Ada");
        assert_eq!(
            sent.cards[0].contact.notes.as_deref(),
            Some("Bells & whistles, 3 < 4")
        );
    }

    #[test]
    fn test_a_reference_this_reader_does_not_know_is_left_as_the_text_it_is() {
        // The other half of unescaping, and the one that matters for safety:
        // the five references XML defines are turned back into their
        // characters and anything else is left alone. An entity a document
        // declared for itself is therefore never expanded.
        let card = a_readable_card(
            "Grace Hopper",
            "grace@example.com",
            "&somewhere; and &#65; too",
        );
        let answer = a_multistatus(&a_card_block(
            "/carddav/sam/contacts/grace.vcf",
            Some("\"1\""),
            &card,
        ));

        let sent = cards_in(&answer, AT, SOMEBODY).expect("a multistatus to be read");

        assert_eq!(
            sent.cards[0].contact.notes.as_deref(),
            Some("&somewhere; and &#65; too")
        );
    }

    #[test]
    fn test_a_line_in_a_card_that_looks_like_a_tag_is_read_as_card_text() {
        let card = a_readable_card(
            "Grace Hopper",
            "grace@example.com",
            "&lt;d:href&gt;not a tag&lt;/d:href&gt;",
        );
        let answer = a_multistatus(&a_card_block(
            "/carddav/sam/contacts/grace.vcf",
            Some("\"1\""),
            &card,
        ));

        let sent = cards_in(&answer, AT, SOMEBODY).expect("a multistatus to be read");

        assert_eq!(
            sent.cards[0].contact.notes.as_deref(),
            Some("<d:href>not a tag</d:href>")
        );
    }

    #[test]
    fn test_an_escaped_ampersand_in_an_address_books_name_comes_back_as_an_ampersand() {
        // A name is read out to somebody. "Sam &amp; Co" is not a name.
        let answer = a_multistatus(&an_address_book(
            "/carddav/sam/contacts/",
            "Sam &amp; Co",
            "12",
        ));

        let found = address_books_in(&answer, AT).expect("a multistatus to be read");

        assert_eq!(found[0].display_name, "Sam & Co");
    }

    #[test]
    fn test_the_request_for_cards_asks_for_every_property_the_reader_reads() {
        for property in ["getetag", "address-data"] {
            assert!(
                ASKING_FOR_THE_CARDS.contains(property),
                "the request does not ask for {property}, which the reader reads"
            );
        }
    }

    #[test]
    fn test_the_whole_file_export_and_the_one_card_writer_give_the_same_card() {
        // The two callers share one answer to what a card looks like. Written
        // twice they disagree the first time either is corrected, and the one
        // that is not corrected goes on sending the old card to a server.
        let cache = TempHome::named("one card and a whole file agree", |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache to open")
        });
        let mut she = a_contact("Grace Hopper");
        she.nickname = Some("Amazing Grace".to_string());
        she.notes = Some("Numbers, then words".to_string());
        she.addresses_json = Some(one_address("12 High Street; Flat 2"));
        cache.save_contact(&she).expect("the contact to be saved");

        let stored = cache
            .get_contacts_for_account(SOMEBODY)
            .expect("the contacts to be read back");
        let whole_file = cache
            .export_contacts_to_vcard(SOMEBODY)
            .expect("the file to be written");

        assert_eq!(stored.len(), 1, "one contact was saved");
        assert_eq!(
            whole_file,
            MessageCache::vcard_block_from_contact(&stored[0]),
            "the whole file export and the one card writer disagree"
        );
    }
}

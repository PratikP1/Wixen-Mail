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

#[cfg(test)]
mod tests {
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

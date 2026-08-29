//! Finding somebody to write to while a recipient line is being typed.
//!
//! Everything about the feature that can be decided without a window and
//! without a network: which part of the line is being completed, whether it is
//! worth looking anybody up for yet, how the people found are put in one list,
//! and what is said about them.

use crate::common::types::EmailAddress;
use crate::data::message_cache::ContactEntry;
use crate::service::directory::FROM_A_DIRECTORY;

/// How many letters have to be typed before anybody is looked up.
///
/// Every letter below this is a question that would go to an organisation's
/// directory, and one or two letters match most of one. Three is the point
/// where the answer is short enough to be worth reading and the question is
/// specific enough to be worth asking.
pub const BEFORE_LOOKING_ANYBODY_UP: usize = 3;

/// Which of the two places a person was found in.
///
/// Written out in the row rather than shown as a colour or implied by the
/// order, because the row is the whole of what a screen reader reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Whose {
    /// The address book on this computer.
    YourContacts,
    /// The organisation's directory, which was asked over the network.
    TheDirectory,
}

impl Whose {
    /// Where this entry came from, read off the entry itself.
    ///
    /// Off the mark a directory lookup puts on what it finds, rather than
    /// told to this by whoever is building the list: a caller that passed the
    /// wrong answer would label a row with somewhere the person is not.
    pub fn of(contact: &ContactEntry) -> Self {
        match contact.source_provider.as_deref() {
            Some(FROM_A_DIRECTORY) => Self::TheDirectory,
            _ => Self::YourContacts,
        }
    }

    /// The words that follow a name and address in the list.
    pub const fn said(self) -> &'static str {
        match self {
            Self::YourContacts => "from your contacts",
            Self::TheDirectory => "from the directory",
        }
    }
}

/// One person who could be written to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Somebody {
    /// What to call them, or their address when nothing else is recorded.
    pub name: String,
    pub address: String,
    pub whose: Whose,
}

impl Somebody {
    /// One person, or nothing when there is no address to write to.
    ///
    /// A contact holding only a phone number is an ordinary contact and not an
    /// answer to the question a recipient line is asking.
    pub fn from_contact(contact: &ContactEntry) -> Option<Self> {
        let address = contact.email.trim();
        if address.is_empty() {
            return None;
        }
        Some(Self {
            name: contact.name.trim().to_string(),
            address: address.to_string(),
            whose: Whose::of(contact),
        })
    }

    /// The line shown in the list and read out by a screen reader.
    pub fn row(&self) -> String {
        match self.name.is_empty() {
            true => format!("{}, {}", self.address, self.whose.said()),
            false => format!("{}, {}, {}", self.name, self.address, self.whose.said()),
        }
    }

    /// The text that goes into the recipient line when this one is chosen.
    ///
    /// Written by the type that already writes an address with a name in front
    /// of it, so a name holding a comma comes out quoted. Written by hand, the
    /// comma in "Babbage, Charles" reads back as the separator between two
    /// recipients and the message goes to an address nobody has.
    pub fn as_a_recipient(&self) -> String {
        EmailAddress::new(
            self.address.clone(),
            Some(self.name.clone()).filter(|name| !name.is_empty()),
        )
        .to_string()
    }
}

/// Whether what has been typed is enough to look anybody up for.
///
/// Counted in characters rather than bytes, or a name written in an alphabet
/// whose letters take more than one byte would be looked up sooner than one
/// written in English.
pub fn worth_looking_up(name: &str) -> bool {
    name.trim().chars().count() >= BEFORE_LOOKING_ANYBODY_UP
}

/// The most people worth putting in the list from either place.
///
/// The directory's own answer to the same question, not a second one. A list
/// of fifty from one place and a different limit from the other would be a
/// list nobody could be told the size of.
pub const AT_MOST_TO_READ_THROUGH: usize = crate::service::directory::AT_MOST;

/// Turn what the address book on this computer answered into people to write
/// to.
///
/// `Err` with a sentence when there are more than anybody can read through.
/// The same answer the directory gives to the same question: showing the first
/// fifty and saying nothing about the rest hides the person being looked for
/// somewhere nobody will look.
pub fn from_your_contacts(found: &[ContactEntry]) -> std::result::Result<Vec<Somebody>, String> {
    if found.len() > AT_MOST_TO_READ_THROUGH {
        return Err(format!(
            "More than {AT_MOST_TO_READ_THROUGH} of your contacts match, so none of them are \
             shown. Type more of the name to narrow the search down."
        ));
    }
    Ok(found.iter().filter_map(Somebody::from_contact).collect())
}

/// Both lists as one, without offering the same address twice.
///
/// The address book on this computer comes first: it is the shorter list, it
/// is the one somebody chose to keep, and it is where the more likely answer
/// is. Where the same address is in both, the entry here is the one kept,
/// because the spelling of the name in it is the one this person chose.
///
/// Compared without case, because an address book and a directory disagree
/// about capitals constantly and two rows for one mailbox make the list longer
/// and the choice harder for nothing.
pub fn everybody_found(
    from_your_contacts: Vec<Somebody>,
    from_the_directory: Vec<Somebody>,
) -> Vec<Somebody> {
    let mut together: Vec<Somebody> = Vec::new();
    for person in from_your_contacts.into_iter().chain(from_the_directory) {
        let already_there = together
            .iter()
            .any(|kept| kept.address.eq_ignore_ascii_case(&person.address));
        if !already_there {
            together.push(person);
        }
    }
    together
}

/// Which search this is, so an answer to an older one can be dropped.
///
/// Somebody types "sm" and then "smith", and the two questions are answered by
/// a server in whatever order it manages. Showing the answer to "sm" under the
/// word "smith" is a list of the wrong people with nothing saying so.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Search(u64);

impl Search {
    /// The next search after this one.
    pub const fn and_then_another(self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    /// Whether an answer to this search is still the one being waited for.
    pub const fn is_still_wanted(self, latest: Self) -> bool {
        self.0 == latest.0
    }
}

/// One request to look somebody up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LookFor {
    /// Which search this is. It comes back on the answer.
    pub search: Search,
    /// The part of the recipient line being typed.
    pub name: String,
    /// Which account the message is being sent from, as the From list numbers
    /// them.
    ///
    /// The directory belongs to an account, so changing the account a message
    /// is sent from changes which organisation is asked.
    pub from_account: Option<u32>,
}

/// What one search found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhoWasFound {
    /// Which search this answers.
    pub search: Search,
    /// What was being looked for, so the sentence can name it.
    pub name: String,
    pub everybody: Vec<Somebody>,
    /// Why the directory added nothing, when that is worth saying.
    pub trouble: Option<String>,
}

/// The label on the list of people found, with the ampersand marking the
/// letter Alt reaches it by.
pub const THE_LIST_LABEL: &str = "P&eople found:";

/// The key that label binds.
///
/// Said out loud when people are found, because a list nobody can find is a
/// list nobody has. A test holds this and the label above to the same letter.
pub const REACHES_THE_LIST: &str = "Alt+E";

/// What these announcements are about, so one supersedes the last.
///
/// Somebody typing quickly finishes several searches, and each has something
/// to say. Sharing a topic means the queue keeps only the newest, so what is
/// heard is the answer for what is in the box now rather than a recital of
/// every answer on the way to it.
pub const WHILE_LOOKING_SOMEBODY_UP: &str = "looking somebody up";

/// The recipient line with the part being typed replaced by the person chosen.
pub fn with_the_chosen(field: &str, chosen: &Somebody) -> String {
    let before = field[..where_the_last_entry_starts(field)].trim_end();
    match before.is_empty() {
        true => chosen.as_a_recipient(),
        false => format!("{before} {}", chosen.as_a_recipient()),
    }
}

/// Whether choosing somebody from the list still has to be explained.
///
/// Said out loud, once, with the first list a window shows, and then not
/// again. Out loud because a description set on a list does not reach the
/// accessibility tree for a native list in this toolkit, so a hint left there
/// is a hint nobody gets. Once, because a sentence repeated after every third
/// letter somebody types is the flood this project has a guardrail against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HowToChoose {
    /// This window has not shown a list of people yet.
    SayIt,
    /// It has, and the explanation went with it.
    AlreadySaid,
}

/// What to say when a search comes back.
pub fn what_was_found(
    name: &str,
    found: &[Somebody],
    trouble: Option<&str>,
    explaining: HowToChoose,
) -> String {
    let name = name.trim();
    let reaching_it = match explaining {
        HowToChoose::SayIt => {
            format!("Press {REACHES_THE_LIST} for the list, then Enter on the person you want.")
        }
        HowToChoose::AlreadySaid => format!("Press {REACHES_THE_LIST} for the list."),
    };
    let opening = match found.len() {
        // Nothing to reach and nothing to choose from, so neither is offered.
        0 => format!("Nobody found for \"{name}\"."),
        1 => format!("1 person found for \"{name}\". {reaching_it}"),
        many => format!("{many} people found for \"{name}\". {reaching_it}"),
    };
    match trouble {
        Some(why) => format!("{opening} {why}"),
        None => opening,
    }
}

/// Where in a recipient line the entry still being typed begins.
///
/// The same rules [`crate::application::reply::split_addresses`] uses, because
/// that is the one thing in this program that decides where one recipient ends
/// and the next begins: a comma or a semicolon separates them unless it is
/// inside a quoted name or inside the angle brackets around an address. Two
/// readings of one line would mean looking somebody up by a fragment of a
/// recipient they had already finished typing.
fn where_the_last_entry_starts(field: &str) -> usize {
    let mut inside_quotes = false;
    let mut inside_angles = false;
    let mut starts_at = 0;
    for (at, character) in field.char_indices() {
        match character {
            '"' => inside_quotes = !inside_quotes,
            '<' if !inside_quotes => inside_angles = true,
            '>' if !inside_quotes => inside_angles = false,
            ',' | ';' if !inside_quotes && !inside_angles => {
                starts_at = at + character.len_utf8();
            }
            _ => {}
        }
    }
    starts_at
}

/// The part of a recipient line that is still being typed.
pub fn the_name_being_typed(field: &str) -> &str {
    field[where_the_last_entry_starts(field)..].trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_name_being_typed_is_the_part_after_the_last_recipient() {
        // Somebody with one address already in the line is typing a second
        // person, and looking up the whole line would find nobody.
        assert_eq!(
            the_name_being_typed("ada@example.com, lovel"),
            "lovel",
            "the part already finished was looked up as well"
        );
    }

    #[test]
    fn test_a_semicolon_separates_recipients_too() {
        // Both separators, because both are what the one splitter in this
        // program accepts, and somebody who types the other one would
        // otherwise have the whole line looked up as a name.
        assert_eq!(the_name_being_typed("ada@example.com; lovel"), "lovel");
    }

    #[test]
    fn test_a_comma_inside_a_name_is_part_of_that_name_and_not_a_separator() {
        // "Babbage, Charles" is an ordinary directory-style name, and it comes
        // back quoted from every other part of this program. Read as a
        // separator it leaves "Charles" <address> looking like the part being
        // typed, and every keystroke afterwards searches for a fragment of a
        // recipient that is already finished.
        assert_eq!(
            the_name_being_typed("\"Babbage, Charles\" <charles@example.com>, love"),
            "love"
        );
        assert_eq!(
            the_name_being_typed("\"Babbage, Charles\" <charles@example.com>"),
            "\"Babbage, Charles\" <charles@example.com>"
        );
    }

    #[test]
    fn test_the_last_entry_is_the_same_one_the_recipient_splitter_finds() {
        // The two must not be able to disagree. One decides who a message is
        // sent to and the other decides what is looked up, and a line where
        // they part company looks up something nobody typed.
        for line in [
            "ada@example.com, lovel",
            "ada@example.com; lovel",
            "\"Babbage, Charles\" <charles@example.com>, love",
            "lovel",
            "  Ada Lovelace <ada@example.com>  ",
        ] {
            let last = crate::application::reply::split_addresses(line)
                .last()
                .cloned()
                .unwrap_or_default();

            assert_eq!(
                the_name_being_typed(line),
                last,
                "the two disagree about the last recipient in {line:?}"
            );
        }
    }

    #[test]
    fn test_an_empty_line_is_nothing_being_typed() {
        assert_eq!(the_name_being_typed(""), "");
        assert_eq!(the_name_being_typed("ada@example.com, "), "");
    }

    // ── Whether to look at all ──────────────────────────────────────────

    #[test]
    fn test_one_or_two_letters_are_not_enough_to_look_anybody_up() {
        // Every letter typed would otherwise be a question asked of an
        // organisation's directory, and one letter matches most of it.
        assert!(!worth_looking_up("a"));
        assert!(!worth_looking_up("ad"));
        assert!(worth_looking_up("ada"));
    }

    #[test]
    fn test_spaces_do_not_count_towards_being_enough_to_look_up() {
        assert!(!worth_looking_up("  a  "));
        assert!(!worth_looking_up("   "));
        assert!(!worth_looking_up(""));
    }

    #[test]
    fn test_three_letters_of_a_language_that_does_not_use_bytes_are_still_three() {
        // Counted as characters and not as bytes, or a name written in Greek
        // or Japanese would be looked up a letter earlier than one written in
        // English, and one written in an alphabet of four-byte characters
        // earlier still.
        assert!(worth_looking_up("\u{3b1}\u{3b2}\u{3b3}"));
        assert!(!worth_looking_up("\u{3b1}\u{3b2}"));
    }

    // ── One person, from either place ───────────────────────────────────

    fn a_contact(name: &str, address: &str, came_from: Option<&str>) -> ContactEntry {
        ContactEntry {
            id: format!("id-{name}"),
            account_id: "acct".to_string(),
            name: name.to_string(),
            given_name: None,
            family_name: None,
            email: address.to_string(),
            phone: None,
            company: None,
            job_title: None,
            website: None,
            address: None,
            birthday: None,
            avatar_url: None,
            avatar_data_base64: None,
            source_provider: came_from.map(str::to_string),
            last_synced_at: None,
            vcard_raw: None,
            notes: None,
            favorite: false,
            created_at: "t".to_string(),
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

    #[test]
    fn test_where_somebody_came_from_is_read_off_the_entry_rather_than_told_to_it() {
        // The one mark a directory entry carries is the one this reads, so a
        // caller cannot label a list wrongly by passing the wrong answer.
        let looked_up = a_contact(
            "Ada Lovelace",
            "ada@example.com",
            Some(crate::service::directory::FROM_A_DIRECTORY),
        );
        let held_here = a_contact("Ada Lovelace", "ada@example.com", Some("google"));
        let from_nowhere = a_contact("Ada Lovelace", "ada@example.com", None);

        assert_eq!(Whose::of(&looked_up), Whose::TheDirectory);
        assert_eq!(Whose::of(&held_here), Whose::YourContacts);
        assert_eq!(Whose::of(&from_nowhere), Whose::YourContacts);
    }

    #[test]
    fn test_a_row_says_which_of_the_two_places_the_person_came_from() {
        // Never by colour, and never left to be worked out from the order: a
        // screen reader reads the row and nothing else.
        let from_the_directory = Somebody::from_contact(&a_contact(
            "Ada Lovelace",
            "ada@example.com",
            Some(crate::service::directory::FROM_A_DIRECTORY),
        ))
        .expect("somebody to write to");
        let from_contacts =
            Somebody::from_contact(&a_contact("Ada Lovelace", "ada@example.com", None))
                .expect("somebody to write to");

        assert!(
            from_the_directory.row().contains("directory"),
            "{}",
            from_the_directory.row()
        );
        assert!(
            from_contacts.row().contains("contacts"),
            "{}",
            from_contacts.row()
        );
        assert_ne!(from_the_directory.row(), from_contacts.row());
    }

    #[test]
    fn test_a_row_says_the_name_and_the_address() {
        // Two people at an organisation share a name often enough that the
        // name alone is not a choice anybody can make.
        let row = Somebody::from_contact(&a_contact("Ada Lovelace", "ada@example.com", None))
            .expect("somebody to write to")
            .row();

        assert!(row.contains("Ada Lovelace"), "{row}");
        assert!(row.contains("ada@example.com"), "{row}");
    }

    #[test]
    fn test_somebody_with_no_address_is_not_somebody_to_write_to() {
        // A contact with only a phone number is an ordinary contact and not an
        // answer to "who is this message going to".
        assert!(Somebody::from_contact(&a_contact("Ada Lovelace", "", None)).is_none());
        assert!(Somebody::from_contact(&a_contact("Ada Lovelace", "   ", None)).is_none());
    }

    #[test]
    fn test_a_person_with_no_name_shows_as_their_address_rather_than_as_nothing() {
        let row = Somebody::from_contact(&a_contact("", "ada@example.com", None))
            .expect("somebody to write to")
            .row();

        assert!(row.contains("ada@example.com"), "{row}");
    }

    #[test]
    fn test_the_text_a_chosen_person_puts_in_the_line_is_written_the_one_way() {
        // Through the type that already writes an address with a name in front
        // of it, so a name holding a comma comes out quoted. Written by hand
        // here, "Babbage, Charles <charles@example.com>" reads back as two
        // recipients and the message goes to an address that does not exist.
        let chosen =
            Somebody::from_contact(&a_contact("Babbage, Charles", "charles@example.com", None))
                .expect("somebody to write to");

        assert_eq!(
            chosen.as_a_recipient(),
            crate::common::types::EmailAddress::new(
                "charles@example.com".to_string(),
                Some("Babbage, Charles".to_string()),
            )
            .to_string()
        );
    }

    #[test]
    fn test_a_person_with_no_name_goes_into_the_line_as_a_bare_address() {
        let chosen = Somebody::from_contact(&a_contact("  ", "ada@example.com", None))
            .expect("somebody to write to");

        assert_eq!(chosen.as_a_recipient(), "ada@example.com");
    }

    // ── Two places, one list ────────────────────────────────────────────

    fn from_contacts(name: &str, address: &str) -> Somebody {
        Somebody::from_contact(&a_contact(name, address, None)).expect("somebody to write to")
    }

    fn from_the_directory(name: &str, address: &str) -> Somebody {
        Somebody::from_contact(&a_contact(
            name,
            address,
            Some(crate::service::directory::FROM_A_DIRECTORY),
        ))
        .expect("somebody to write to")
    }

    #[test]
    fn test_the_people_here_come_before_the_people_at_the_organisation() {
        // The address book on this computer is the one somebody chose to keep,
        // so it is the more likely answer and the shorter list to read through.
        let together = everybody_found(
            vec![from_contacts("Ada Lovelace", "ada@example.com")],
            vec![from_the_directory("Adam Smith", "adam@example.com")],
        );

        assert_eq!(together.len(), 2);
        assert_eq!(together[0].whose, Whose::YourContacts);
        assert_eq!(together[1].whose, Whose::TheDirectory);
    }

    #[test]
    fn test_one_person_in_both_places_is_one_row_and_not_two() {
        // Somebody who has saved a colleague is in both, and offering the same
        // address twice makes the list longer and the choice harder for no
        // gain. The one already saved wins, because that is the spelling of
        // their name this person chose.
        let together = everybody_found(
            vec![from_contacts("Ada", "ada@example.com")],
            vec![from_the_directory("Ada Lovelace", "ADA@EXAMPLE.COM")],
        );

        assert_eq!(together.len(), 1, "{together:?}");
        assert_eq!(together[0].name, "Ada");
        assert_eq!(together[0].whose, Whose::YourContacts);
    }

    #[test]
    fn test_one_address_twice_in_the_same_place_is_still_one_row() {
        // An address book holding the same person on two cards is ordinary,
        // and so is a directory with two entries pointing at one mailbox.
        let together = everybody_found(
            vec![
                from_contacts("Ada", "ada@example.com"),
                from_contacts("Ada Lovelace", "ada@example.com"),
            ],
            Vec::new(),
        );

        assert_eq!(together.len(), 1, "{together:?}");
    }

    // ── An answer that arrives after the question changed ───────────────

    #[test]
    fn test_the_answer_to_an_older_search_is_not_the_answer_to_this_one() {
        // Somebody types "sm", then "smith". The answer to "sm" can arrive
        // second, and showing it would put the wrong people under the wrong
        // name with nothing to say so.
        let first = Search::default();
        let second = first.and_then_another();

        assert!(second.is_still_wanted(second));
        assert!(!first.is_still_wanted(second));
    }

    #[test]
    fn test_each_search_is_a_different_one_from_the_last() {
        let mut search = Search::default();
        let mut seen = vec![search];
        for _ in 0..5 {
            search = search.and_then_another();
            assert!(!seen.contains(&search), "a search number came round again");
            seen.push(search);
        }
    }

    // ── Putting the chosen person in the line ───────────────────────────

    #[test]
    fn test_choosing_somebody_replaces_only_the_part_being_typed() {
        let line = with_the_chosen(
            "bob@example.com, lovel",
            &from_contacts("Ada Lovelace", "ada@example.com"),
        );

        assert_eq!(line, "bob@example.com, Ada Lovelace <ada@example.com>");
    }

    #[test]
    fn test_choosing_somebody_into_an_empty_line_leaves_no_stray_separator() {
        let line = with_the_chosen("lov", &from_contacts("Ada Lovelace", "ada@example.com"));

        assert_eq!(line, "Ada Lovelace <ada@example.com>");
    }

    #[test]
    fn test_the_line_a_choice_makes_reads_back_as_the_people_who_are_in_it() {
        // The whole point of putting text in this box: what goes in has to
        // come back out as the same recipients when the message is sent.
        let line = with_the_chosen(
            "bob@example.com, bab",
            &from_the_directory("Babbage, Charles", "charles@example.com"),
        );

        let back = crate::application::reply::split_addresses(&line);
        assert_eq!(back.len(), 2, "{line}");
        assert!(back[1].contains("charles@example.com"), "{line}");
    }

    // ── What is said when the answers arrive ────────────────────────────

    #[test]
    fn test_the_number_of_people_found_is_said_rather_than_left_to_be_discovered() {
        let two = what_was_found(
            "lovel",
            &[from_contacts("Ada", "a@x.com")],
            None,
            HowToChoose::AlreadySaid,
        );
        assert!(two.contains('1'), "{two}");
        assert!(two.contains("person"), "{two}");

        let more = what_was_found(
            "lovel",
            &[
                from_contacts("Ada", "a@x.com"),
                from_contacts("Adam", "b@x.com"),
            ],
            None,
            HowToChoose::AlreadySaid,
        );
        assert!(more.contains('2'), "{more}");
        assert!(more.contains("people"), "{more}");
    }

    #[test]
    fn test_the_sentence_says_how_to_reach_the_list() {
        // A list nobody can find is a list nobody has, and the key is not
        // written anywhere else somebody working by ear would come across.
        let said = what_was_found(
            "lovel",
            &[from_contacts("Ada", "a@x.com")],
            None,
            HowToChoose::AlreadySaid,
        );

        assert!(said.contains(REACHES_THE_LIST), "{said}");
    }

    #[test]
    fn test_how_to_choose_somebody_is_said_with_the_first_list_and_then_not_again() {
        // A description set on a list does not reach the accessibility tree
        // for a native list in this toolkit, so it has to be said out loud or
        // it is not said at all. Once, though: on every search it would be a
        // sentence repeated after every third letter somebody types.
        let people = [from_contacts("Ada", "a@x.com")];
        let first = what_was_found("lovel", &people, None, HowToChoose::SayIt);
        let after = what_was_found("lovel", &people, None, HowToChoose::AlreadySaid);

        assert!(first.contains("Enter"), "{first}");
        assert!(!after.contains("Enter"), "{after}");
        assert!(first.len() > after.len(), "{first}");
    }

    #[test]
    fn test_finding_nobody_does_not_explain_a_list_that_is_not_there() {
        let said = what_was_found("qqq", &[], None, HowToChoose::SayIt);

        assert!(!said.contains("Enter"), "{said}");
        assert!(!said.contains(REACHES_THE_LIST), "{said}");
    }

    #[test]
    fn test_finding_nobody_is_said_rather_than_left_as_silence() {
        let said = what_was_found("qqq", &[], None, HowToChoose::AlreadySaid);

        assert!(said.to_lowercase().contains("nobody"), "{said}");
        assert!(said.contains("qqq"), "{said}");
        assert!(
            !said.contains(REACHES_THE_LIST),
            "an empty list was offered as somewhere to go: {said}"
        );
    }

    #[test]
    fn test_a_reason_the_directory_could_not_help_is_carried_into_the_sentence() {
        // Otherwise a directory nobody can reach is indistinguishable from an
        // organisation with nobody by that name in it.
        let said = what_was_found(
            "lovel",
            &[],
            Some("The directory did not answer."),
            HowToChoose::AlreadySaid,
        );

        assert!(said.contains("The directory did not answer."), "{said}");
    }

    #[test]
    fn test_a_reason_is_still_said_when_the_contacts_here_found_somebody() {
        // Finding two people here does not mean the search worked. Staying
        // quiet about a directory that refused the sign-in leaves somebody
        // believing their organisation holds nobody else by that name.
        let said = what_was_found(
            "lovel",
            &[from_contacts("Ada", "a@x.com")],
            Some("The directory would not accept the sign-in."),
            HowToChoose::AlreadySaid,
        );

        assert!(said.contains("would not accept the sign-in"), "{said}");
        assert!(said.contains('1'), "{said}");
    }

    #[test]
    fn test_the_label_and_the_key_it_binds_say_the_same_letter() {
        // The label's ampersand is what actually binds the key, and the
        // sentence is what tells somebody the key exists. Written out twice,
        // they can part company and leave a spoken instruction for a key that
        // does nothing.
        let underlined = THE_LIST_LABEL
            .split('&')
            .nth(1)
            .and_then(|rest| rest.chars().next())
            .expect("the label marks a letter with an ampersand");

        assert_eq!(
            REACHES_THE_LIST,
            format!("Alt+{}", underlined.to_ascii_uppercase()),
            "the label underlines {underlined:?} and the sentence says \
             {REACHES_THE_LIST}"
        );
    }

    // ── What the address book here answered ─────────────────────────────

    #[test]
    fn test_the_contacts_here_become_people_to_write_to() {
        let answered = vec![
            a_contact("Ada Lovelace", "ada@example.com", None),
            a_contact("Adam Smith", "adam@example.com", None),
        ];

        let found = from_your_contacts(&answered).expect("two people");

        assert_eq!(found.len(), 2);
        assert_eq!(found[0].whose, Whose::YourContacts);
    }

    #[test]
    fn test_a_contact_with_only_a_phone_number_is_not_one_of_the_people_found() {
        let answered = vec![
            a_contact("Ada Lovelace", "ada@example.com", None),
            a_contact("The Plumber", "", None),
        ];

        let found = from_your_contacts(&answered).expect("one person");

        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_more_contacts_than_anybody_can_read_through_asks_for_a_narrower_search() {
        // The same answer the directory gives to the same question. Showing
        // the first fifty and saying nothing about the rest hides the one
        // being looked for somewhere nobody will find it.
        let crowd: Vec<ContactEntry> = (0..=AT_MOST_TO_READ_THROUGH)
            .map(|n| a_contact(&format!("Person {n}"), &format!("p{n}@example.com"), None))
            .collect();

        let refused = from_your_contacts(&crowd).expect_err("a refusal");

        assert!(
            refused.contains(&AT_MOST_TO_READ_THROUGH.to_string()),
            "{refused}"
        );
        assert!(refused.contains("more"), "{refused}");
    }

    #[test]
    fn test_exactly_as_many_as_can_be_read_through_are_all_shown() {
        // The boundary, both ways round: one more than the limit is too many
        // and the limit itself is not.
        let just_enough: Vec<ContactEntry> = (0..AT_MOST_TO_READ_THROUGH)
            .map(|n| a_contact(&format!("Person {n}"), &format!("p{n}@example.com"), None))
            .collect();

        let found = from_your_contacts(&just_enough).expect("a full list");

        assert_eq!(found.len(), AT_MOST_TO_READ_THROUGH);
    }

    #[test]
    fn test_how_many_is_too_many_is_the_one_answer_this_program_has() {
        // Two limits would mean a list of fifty from one place and eighty from
        // the other, read through by somebody who was told fifty.
        assert_eq!(AT_MOST_TO_READ_THROUGH, crate::service::directory::AT_MOST);
    }

    #[test]
    fn test_every_sentence_reads_as_sentences_rather_than_a_wrapped_literal() {
        // A wrapped string literal that loses its continuations keeps every
        // space of the indenting, and these are read aloud.
        let said = [
            what_was_found(
                "lovel",
                &[from_contacts("Ada", "a@x.com")],
                None,
                HowToChoose::SayIt,
            ),
            what_was_found(
                "lovel",
                &[from_contacts("Ada", "a@x.com")],
                None,
                HowToChoose::AlreadySaid,
            ),
            what_was_found("qqq", &[], None, HowToChoose::AlreadySaid),
            what_was_found(
                "qqq",
                &[],
                Some("The directory did not answer."),
                HowToChoose::AlreadySaid,
            ),
        ];

        for sentence in said {
            assert!(!sentence.contains("  "), "{sentence}");
        }
    }
}

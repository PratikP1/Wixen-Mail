//! Who is coming to a meeting, as somebody typed them into the event form.
//!
//! The guest list is one box of text, because that is what somebody arranging
//! a meeting has in their head and in the message they were sent. This turns
//! it into people, and turns people back into the box and into the column the
//! event is stored in.
//!
//! # One address parser, not a second one
//!
//! Nothing here reads an address itself. Splitting a guest list looks like a
//! job for `split(',')` and is not: a name written `"Smith, John"
//! <john@example.com>` is one person, and a splitter that does not know a
//! comma inside quotes is not a separator turns them into two, one of whom has
//! no address and neither of whom is the person who was invited. This program
//! has already been bitten by exactly that shape, in a calendar parser that
//! split on every semicolon and truncated a name written the same way.
//!
//! So the reading is `service::mime::parse_addresses`, which is the RFC 5322
//! parser the mail side already uses, escapes and all. One parser cannot come
//! to disagree with itself.

use crate::common::types::EmailAddress;

/// One person invited to a meeting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coming {
    /// What they are called, as it will be read out. Never empty: somebody
    /// written down as an address alone is called by their address, because a
    /// list read aloud with a silence in it is a list nobody can follow.
    pub called: String,
    /// Their address, as it was written, without a name around it.
    pub address: String,
}

/// The people a typed guest list names, each once, in the order they were
/// written.
pub fn typed_in(typed: &str) -> Vec<Coming> {
    let mut invited: Vec<Coming> = Vec::new();
    for person in crate::service::mime::parse_addresses(&all_on_one_line(typed)) {
        let person = one_guest(person);
        if !invited
            .iter()
            .any(|already| the_same_person(&already.address, &person.address))
        {
            invited.push(person);
        }
    }
    invited
}

/// A guest list typed over several lines, as one line of addresses.
///
/// An address header is one line by definition, and the parser stops at the
/// first line ending it meets that is not folded. So a guest list typed one
/// person to a line, which is how this box invites somebody to type it and how
/// [`as_typed`] writes it back, would be read as the first person alone and
/// everybody underneath would be dropped without a word.
///
/// A line's own trailing comma is taken off before the lines are joined by
/// one, so a list written either way, or both ways at once, comes out as the
/// same list rather than as one with a gap between every pair of names.
fn all_on_one_line(typed: &str) -> String {
    typed
        .lines()
        .map(|line| line.trim().trim_end_matches(',').trim_end())
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join(",")
}

/// Whether two written addresses name the same person.
///
/// Without case. A domain is case-insensitive by definition and in practice so
/// is the half in front of it, and this is the same comparison
/// `service::free_busy` makes when matching a server's answer back to the
/// person it was asked about.
fn the_same_person(one: &str, other: &str) -> bool {
    one.eq_ignore_ascii_case(other)
}

/// One guest, in the shape the event's own column holds.
///
/// The same keys `application::calendar` already writes when a guest list
/// arrives from a provider, so an event edited here and one synced down are
/// read by one reader rather than two that could come to disagree.
#[derive(serde::Serialize, serde::Deserialize)]
struct StoredGuest {
    #[serde(default)]
    email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// The answer this person has already sent, where a provider recorded one.
    ///
    /// Carried across an edit rather than written from the form, because the
    /// form never asks for it: rewritten from a box of names alone, every
    /// "yes" already sent would come back a blank and the meeting would look
    /// like one nobody had answered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    status: Option<String>,
}

/// The guest list, as the event's own column holds it.
///
/// Nothing at all for a meeting nobody is coming to, because that column is
/// empty for every event that has no guests and an empty list written into it
/// would be a change where there was none.
pub fn as_stored(invited: &[Coming], held: Option<&str>) -> Option<String> {
    if invited.is_empty() {
        return None;
    }
    let answered = what_was_already_answered(held);
    let written: Vec<StoredGuest> = invited
        .iter()
        .map(|person| StoredGuest {
            email: person.address.clone(),
            // Nothing for somebody nobody gave a name for. [`typed_in`] calls
            // them by their address so a list read aloud has no silence in it,
            // and writing that back as their name would be this program
            // inventing one: stored, it goes to the provider on the next sync
            // as what that person is called.
            name: (!the_same_person(&person.called, &person.address))
                .then(|| person.called.clone()),
            status: answered
                .iter()
                .find(|(address, _)| the_same_person(address, &person.address))
                .and_then(|(_, answer)| answer.clone()),
        })
        .collect();
    serde_json::to_string(&written).ok()
}

/// What each person the column already named had answered.
///
/// Matched back by address rather than by position, because the list somebody
/// just typed is not the list that was stored: a name added at the top would
/// otherwise be handed the answer of whoever used to be first.
fn what_was_already_answered(held: Option<&str>) -> Vec<(String, Option<String>)> {
    let Some(held) = held else {
        return Vec::new();
    };
    let read: Vec<StoredGuest> = serde_json::from_str(held).unwrap_or_default();
    read.into_iter()
        .map(|guest| (guest.email, guest.status))
        .collect()
}

/// The guest list as it goes back into the box somebody types it in.
///
/// One person to a line, because the box holds several lines and a screen
/// reader moves through it a line at a time: a guest list on one long line is
/// one long sentence to check a name in. The comma is still there, so what
/// comes back is an ordinary address list whatever it is read by.
pub fn as_typed(invited: &[Coming]) -> String {
    invited
        .iter()
        .map(as_one_person_is_written)
        .collect::<Vec<String>>()
        .join(",\n")
}

/// The guest list an event already holds, as it goes into the box.
///
/// One function rather than the two steps written out wherever the box is
/// filled. Two places work out what that box shows: the one that fills it, and
/// the one that asks afterwards whether anything was edited. Written out twice
/// they drift, and what that looks like is either an event that reports itself
/// changed every time it is opened, or a guest list somebody edited and lost.
pub fn in_the_box(stored: Option<&str>) -> String {
    as_typed(&already_on(stored))
}

/// One person, written the way an address with a name on it is written.
///
/// The address by itself where that is all anybody gave, so reopening an event
/// does not fill the box with `ada@example.com <ada@example.com>`.
///
/// Not called `written_out`, which is what `service::caldav` calls the function
/// that folds a calendar line: `tests/house_style.rs` counts the definitions of
/// each calendar-format helper and requires exactly one, and a second function
/// of that name here reads like a second answer to that question when it is not
/// about calendar lines at all.
fn as_one_person_is_written(person: &Coming) -> String {
    if the_same_person(&person.called, &person.address) {
        return person.address.clone();
    }
    format!(
        "{} <{}>",
        quoted_where_it_has_to_be(&person.called),
        person.address
    )
}

/// A name, with quotation marks round it where leaving them off would change
/// who is named.
///
/// RFC 5322 section 3.2.3: a name holding any of the characters that separate
/// one address from the next is not a plain run of words, and written bare it
/// is read back as two people. A quotation mark or a backslash inside the name
/// is escaped, or the quoting ends early and the same thing happens.
fn quoted_where_it_has_to_be(called: &str) -> String {
    if !called
        .chars()
        .any(|letter| SPECIAL_IN_AN_ADDRESS.contains(&letter))
    {
        return called.to_string();
    }
    format!("\"{}\"", called.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The characters RFC 5322 section 3.2.3 calls special.
const SPECIAL_IN_AN_ADDRESS: [char; 13] = [
    '(', ')', '<', '>', '[', ']', ':', ';', '@', '\\', ',', '.', '"',
];

/// The people an event already holds, for filling the box back in.
///
/// Anybody the column names without an address is left out: they cannot be
/// asked about and they cannot be written back, so carrying them would put a
/// name in the box that does nothing.
pub fn already_on(stored: Option<&str>) -> Vec<Coming> {
    let Some(stored) = stored else {
        return Vec::new();
    };
    let read: Vec<StoredGuest> = serde_json::from_str(stored).unwrap_or_default();
    read.into_iter()
        .filter(|guest| !guest.email.trim().is_empty())
        .map(|guest| Coming {
            called: guest
                .name
                .map(|name| name.trim().to_string())
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| guest.email.clone()),
            address: guest.email,
        })
        .collect()
}

/// One parsed address, as somebody to ask about.
fn one_guest(found: EmailAddress) -> Coming {
    let address = found.address;
    Coming {
        called: found.name.unwrap_or_else(|| address.clone()),
        address,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A guest list, as a test can read at a glance.
    fn people(typed: &str) -> Vec<(String, String)> {
        typed_in(typed)
            .into_iter()
            .map(|person| (person.called, person.address))
            .collect()
    }

    #[test]
    fn test_a_guest_list_typed_one_to_a_line_is_everybody_on_it() {
        // The box holds several lines and says one person to a line, so this
        // is how it will really be filled in. An address header is one line by
        // definition and its parser stops at the first unfolded line ending,
        // so read straight this is Ada and nobody else: everybody underneath
        // is dropped in silence, and the meeting is arranged around one person
        // out of three.
        let one_to_a_line = "Ada Lovelace <ada@example.com>\n\
                             bob@example.com\n\
                             \"Smith, John\" <john@example.com>";

        assert_eq!(
            people(one_to_a_line),
            [
                ("Ada Lovelace".to_string(), "ada@example.com".to_string()),
                ("bob@example.com".to_string(), "bob@example.com".to_string()),
                ("Smith, John".to_string(), "john@example.com".to_string()),
            ]
        );
    }

    #[test]
    fn test_a_meeting_nobody_is_coming_to_stores_nothing_at_all() {
        // That column is empty for every event with no guests on it. An empty
        // list written into it is a change where there was none, and a
        // difference every sync afterwards would carry to the server.
        assert_eq!(as_stored(&[], None), None);
        assert_eq!(typed_in("   \n  \n"), Vec::new());
    }

    #[test]
    fn test_a_guest_list_survives_the_column_it_is_stored_in() {
        // A lenient reader beside a strict writer is the shape that has cost
        // this program data more than once: both look right on their own and
        // they answer one question two ways. Asked as a round trip, a guest
        // list that comes back shorter than it went in cannot hide.
        let invited = typed_in("\"Smith, John\" <john@example.com>, ada@example.com");

        let stored = as_stored(&invited, None);

        assert_eq!(already_on(stored.as_deref()), invited);
    }

    #[test]
    fn test_the_guest_list_goes_back_into_the_box_it_was_typed_in() {
        // Reopening an event has to show who is already coming, or somebody
        // changes the title, presses Save, and the guest list is written back
        // from an empty box.
        //
        // A round trip rather than a comparison against a written-out string,
        // because the failure worth catching is a writer that puts a name
        // holding a comma back without its quotation marks: read again, one
        // person becomes two and one of them has no address.
        let invited = typed_in("\"Smith, John\" <john@example.com>, ada@example.com");

        assert_eq!(typed_in(&as_typed(&invited)), invited);
    }

    #[test]
    fn test_nobody_is_given_a_name_they_were_never_written_down_with() {
        // Somebody stored as an address alone is read out by their address,
        // because a list read aloud with a silence in it is a list nobody can
        // follow. Written back that way it stops being a stand-in and becomes
        // what this program says that person is called, and the next sync
        // tells their provider so.
        let held = "[{\"email\":\"sam@example.com\"}]";

        let stored = as_stored(&already_on(Some(held)), Some(held));

        assert_eq!(stored.as_deref(), Some(held));
    }

    #[test]
    fn test_editing_the_guest_list_keeps_the_answers_people_already_sent() {
        // A provider writes each guest's reply into this column. Rewritten
        // from a box of names alone, every "yes" already sent becomes a blank
        // and the organiser is looking at a meeting nobody has answered.
        let held = "[{\"email\":\"ada@example.com\",\"name\":\"Ada\",\"status\":\"accepted\"}]";
        let invited = typed_in("Ada <ada@example.com>, bob@example.com");

        let stored = as_stored(&invited, Some(held)).expect("a guest list is stored");

        let read: serde_json::Value = serde_json::from_str(&stored).expect("what was stored");
        assert_eq!(read[0]["status"], serde_json::json!("accepted"), "{stored}");
        // And somebody only just added has no answer yet, rather than the
        // answer of whoever used to be in that position.
        assert_eq!(read[1]["status"], serde_json::Value::Null, "{stored}");
    }

    #[test]
    fn test_the_same_person_written_twice_is_asked_about_once() {
        // Somebody pasted from a message and then typed a name they thought
        // was missing. Left as two, their server is asked about them twice and
        // every sentence in the answer says their name twice, which sounds
        // like two people who happen to share a diary.
        //
        // The first spelling is the one kept, because the answer names people
        // in the order they were invited and the first is where they were
        // invited.
        assert_eq!(
            people("Ada Lovelace <ada@example.com>, ADA@Example.com"),
            [("Ada Lovelace".to_string(), "ada@example.com".to_string())]
        );
    }

    #[test]
    fn test_a_name_with_a_comma_in_it_is_one_person_and_not_two() {
        // The whole reason nothing here splits on a comma itself. Read that
        // way, this guest list is three people: "Smith, then John" with no
        // address at all, then a stray fragment, then Ada. The meeting is
        // then arranged around somebody who does not exist, and the person
        // who was really invited is never asked about.
        assert_eq!(
            people("\"Smith, John\" <john@example.com>, ada@example.com"),
            [
                ("Smith, John".to_string(), "john@example.com".to_string()),
                ("ada@example.com".to_string(), "ada@example.com".to_string()),
            ]
        );
    }
}

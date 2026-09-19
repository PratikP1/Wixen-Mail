//! What the accessibility scan opens each window on.
//!
//! A fresh profile has no mail, no contacts, no events and no reminders, and
//! several windows refuse to open on nothing: `wx_destination::ask` returns at
//! once with no branches, `wx_thread_view::show_thread_dialog` with no
//! messages, `wx_which_days::which_days_are_meant` for an event that does not
//! repeat. The scan needs those windows open, so this is the made-up data they
//! open on, the way `ScanTarget::Reader` opens on a made-up message. Nothing
//! here is written anywhere, and nothing reads it but `open_for_scanning`.
//!
//! Each fixture has a test for the one property that decides whether its
//! window opens at all, because a fixture that opens nothing is a scan of the
//! main window reported as a pass for a dialog nobody looked at. That is the
//! failure `scan_target.rs`'s own doc comment names, and it is the one a
//! fixture can cause without anything failing.

use crate::application::calendar::{WhatTheCalendarAllows, WhereAChangeGoes};
use crate::application::conflict_choice::{AField, BothCopies, TheOtherCopy};
use crate::application::destinations::{Branch, Destination};
use crate::application::due::Due;
use crate::application::saved_searches::Question;
use crate::presentation::wx_folder_choice::FolderRow;
use crate::presentation::wx_managers::{
    AddressItem, ContactEntry, CustomFieldItem, EmailItem, FilterRule, PhoneItem, SignatureEntry,
};
use crate::presentation::wx_thread_view::ThreadNode;

/// A conversation with a reply in it, so the tree has a second level.
pub fn conversation() -> Vec<ThreadNode> {
    let message = |message_id: i64, sender: &str, depth: usize, parent: Option<usize>| ThreadNode {
        message_id,
        uid: u32::try_from(message_id).unwrap_or(0),
        sender: sender.to_string(),
        subject: "Scan target".to_string(),
        date: "2026-01-01T00:00:00+00:00".to_string(),
        read: depth == 0,
        depth,
        parent,
    };
    vec![
        message(1, "Somebody <somebody@example.com>", 0, None),
        message(2, "Me <me@example.com>", 1, Some(0)),
        message(3, "Somebody <somebody@example.com>", 2, Some(1)),
    ]
}

/// Two copies of one contact that disagree, so the window has a row to show.
///
/// One field both name differently and one only the provider names, because
/// those are the two shapes of disagreement the window reads out and the
/// second is the one that hid a telephone number once.
pub fn both_copies() -> BothCopies {
    BothCopies {
        what_it_is_called: "Scan target".to_string(),
        other_copy: TheOtherCopy::AnAddressBook,
        here: vec![
            AField::new("Name", "Scan Target"),
            AField::new("Email", "scan-target@example.com"),
        ],
        theirs: vec![
            AField::new("Name", "Scan Target"),
            AField::new("Email", "somebody@example.com"),
            AField::new("Telephone", "+1 555 0100"),
        ],
    }
}

/// Two accounts with an Archive each, so the tree has branches to build from
/// and two rows that would read alike in a flat list.
pub fn branches() -> Vec<Branch> {
    let account = |account_id: &str, account_name: &str| Branch {
        account_id: account_id.to_string(),
        account_name: account_name.to_string(),
        places: ["Inbox", "Archive"]
            .into_iter()
            .map(|name| Destination {
                name: name.to_string(),
                id: name.to_string(),
                account_id: account_id.to_string(),
                depth: 0,
            })
            .collect(),
    };
    vec![
        account("scan-work", "work@example.com"),
        account("scan-home", "home@example.com"),
    ]
}

/// A server's folder list, so the checked list has rows, with the one row
/// that holds a copy of every message.
pub fn folders() -> Vec<FolderRow> {
    let folder = |path: &str, syncing: bool, holds_all_mail: bool, total: usize| FolderRow {
        path: path.to_string(),
        name: path.to_string(),
        parent: None,
        syncing,
        subscribed: syncing,
        holds_all_mail,
        total,
    };
    vec![
        folder("INBOX", true, false, 12),
        folder("Archive", true, false, 340),
        folder("[Gmail]/All Mail", false, true, 352),
    ]
}

/// One row of each kind that can be due, so the scan meets the list with
/// three rows and every button the window has: a task due on a day, a
/// reminder that came due, and an event that has started.
pub fn due_rows() -> Vec<Due> {
    use crate::application::due::{Identity, Kind};
    let row = |kind: Kind, id: &str, when: &str| Due {
        identity: Identity {
            kind,
            id: id.to_string(),
        },
        title: "Scan target".to_string(),
        when: when.to_string(),
        late: true,
    };
    vec![
        row(Kind::Task, "scan-task", "2026-01-01"),
        row(Kind::Reminder, "scan-reminder", "2026-01-01T09:00:00"),
        row(
            Kind::Event,
            "scan-event|2026-01-01T09:30:00",
            "2026-01-01T09:30:00",
        ),
    ]
}

/// An event that repeats, and what its calendar allows, for the window that
/// asks whether a change is meant for one day or all of them.
pub struct RepeatingEvent {
    pub summary: String,
    pub repeats: String,
    pub allows: WhatTheCalendarAllows,
}

/// The repeating event the "which days" question is asked about.
///
/// Kept on this computer, since a fresh profile has no calendar server, which
/// is the calendar every arm of the question is offered for.
pub fn repeating_event() -> RepeatingEvent {
    RepeatingEvent {
        summary: "Scan target".to_string(),
        repeats: "every week".to_string(),
        allows: WhatTheCalendarAllows::just(WhereAChangeGoes::KeptHere),
    }
}

/// A contact with something on every list, for the contact editor.
///
/// The editor opens on nothing, so no property decides whether the window
/// appears; what the scan meets is the point. Opened on a stored contact the
/// editor is "Edit Contact" with every field filled and a row on each of its
/// four lists, which is the shape somebody editing a real contact meets, and
/// the shape the tester met the unnamed checkbox in (#40).
pub fn contact() -> ContactEntry {
    ContactEntry {
        id: "scan-contact".to_string(),
        name: "Scan Target".to_string(),
        given_name: "Scan".to_string(),
        family_name: "Target".to_string(),
        nickname: "Scan".to_string(),
        company: "Example".to_string(),
        department: "Scanning".to_string(),
        job_title: "Fixture".to_string(),
        emails: vec![EmailItem {
            label: "Work".to_string(),
            address: "scan-target@example.com".to_string(),
        }],
        phones: vec![PhoneItem {
            label: "Work".to_string(),
            number: "+1 555 0100".to_string(),
        }],
        addresses: vec![AddressItem {
            label: "Work".to_string(),
            street: "1 Example Street".to_string(),
            city: "Example".to_string(),
            state: "EX".to_string(),
            zip: "00000".to_string(),
            country: "Nowhere".to_string(),
        }],
        birthday: "2000-01-01".to_string(),
        website: "https://example.com".to_string(),
        relationship: "Colleague".to_string(),
        notes: "Opened for the accessibility scan.".to_string(),
        custom_fields: vec![CustomFieldItem {
            label: "Opened by".to_string(),
            value: "the accessibility scan".to_string(),
        }],
        avatar_url: String::new(),
        favorite: true,
    }
}

/// A stored condition the condition editor can be opened on.
///
/// The editor refuses, before it is built, a stored condition whose field or
/// way of matching it has no words for, so the fixture has to name ones it
/// does, or the scan meets a refusal box rather than the editor.
pub fn condition() -> Question {
    Question {
        field: "subject".to_string(),
        match_type: "contains".to_string(),
        pattern: "scan target".to_string(),
        case_sensitive: true,
    }
}

/// A stored filter rule the filter editor can be opened on, with the same
/// refusal to get past as the condition above.
pub fn filter() -> FilterRule {
    FilterRule {
        id: "scan-filter".to_string(),
        name: "Scan target".to_string(),
        field: "subject".to_string(),
        match_type: "contains".to_string(),
        pattern: "scan target".to_string(),
        case_sensitive: true,
        action_type: "add_tag".to_string(),
        action_value: "scanned".to_string(),
        enabled: true,
        plays_a_sound: false,
    }
}

/// A stored signature the signature editor can be opened on, the default
/// one, so the checkbox the tester met unnamed (#42) is scanned ticked and
/// the walk reads its state as well as its name.
pub fn signature() -> SignatureEntry {
    SignatureEntry {
        id: "scan-signature".to_string(),
        name: "Scan target".to_string(),
        content_plain: "Sent from the accessibility scan.".to_string(),
        content_html: None,
        is_default: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::manager_words;
    use crate::presentation::wx_managers::what_stops_this_being_shown;

    #[test]
    fn test_the_contact_has_a_row_on_every_list() {
        // The editor's three other tabs are four lists with Add and Remove
        // beside each. A contact with nothing on them is scanned as four empty
        // lists, and an empty list is the state that says nothing about its
        // rows.
        let contact = contact();

        for (what, rows) in [
            ("emails", contact.emails.len()),
            ("phones", contact.phones.len()),
            ("addresses", contact.addresses.len()),
            ("custom fields", contact.custom_fields.len()),
        ] {
            assert!(
                rows > 0,
                "the contact has no {what}, so that list is scanned empty"
            );
        }
    }

    #[test]
    fn test_the_condition_is_one_the_editor_agrees_to_show() {
        // `show_rule_edit` refuses before building, so a fixture the editor
        // cannot show is a refusal box scanned in place of the editor.
        let condition = condition();

        assert_eq!(
            what_stops_this_being_shown(
                manager_words::CONDITION,
                &condition.field,
                &condition.match_type
            ),
            None,
            "the editor would refuse this condition before opening"
        );
    }

    #[test]
    fn test_the_filter_is_one_the_editor_agrees_to_show() {
        // The same refusal, made by `show_filter_edit`.
        let filter = filter();

        assert_eq!(
            what_stops_this_being_shown(manager_words::FILTER, &filter.field, &filter.match_type),
            None,
            "the editor would refuse this filter before opening"
        );
    }

    #[test]
    fn test_the_signature_is_the_default_so_the_checkbox_is_scanned_ticked() {
        // The checkbox the tester met is "Default signature". Ticked, the
        // walk reads a state as well as a name; unticked it reads only the
        // absence of one.
        assert!(
            signature().is_default,
            "the signature is not the default, so the checkbox is scanned unticked"
        );
    }

    #[test]
    fn test_the_conversation_has_a_reply_under_its_first_message() {
        // `show_thread_dialog` returns at once on an empty slice, and a flat
        // list of one message is a tree with no levels, which is the thing
        // the window exists to announce.
        let nodes = conversation();

        assert!(!nodes.is_empty(), "no conversation, so no window");
        assert!(
            nodes
                .iter()
                .any(|node| node.parent == Some(0) && node.depth == 1),
            "no message is a reply to the first, so the tree has one level"
        );
    }

    #[test]
    fn test_the_two_copies_disagree_about_something() {
        // A window asking which copy to keep, over two copies that agree, has
        // no row in it.
        assert!(
            !both_copies().fields_that_differ().is_empty(),
            "the two copies agree, so the window has nothing to show"
        );
    }

    #[test]
    fn test_the_branches_offer_two_accounts_with_a_place_each() {
        // `ask` returns at once with no branches. Two accounts, because the
        // window's own reason for being a tree is that two accounts can both
        // have an Archive; and every place filed under the account it says
        // it is in, or the tree is built wrong before anybody reads it.
        let branches = branches();

        assert!(
            branches.len() >= 2,
            "one account is a flat list, not a tree"
        );
        for branch in &branches {
            assert!(
                !branch.places.is_empty(),
                "{} has nowhere to go",
                branch.account_name
            );
            for place in &branch.places {
                assert_eq!(
                    place.account_id, branch.account_id,
                    "{} is filed under the wrong account",
                    place.name
                );
            }
        }
    }

    #[test]
    fn test_the_folders_include_one_holding_every_message() {
        // `ask` returns at once on an empty slice, and the row that says a
        // folder holds a copy of every message is the wording the window
        // exists for.
        let folders = folders();

        assert!(!folders.is_empty(), "no folders, so no window");
        assert!(
            folders.iter().any(|folder| folder.holds_all_mail),
            "no folder holds every message, so that wording is never on screen"
        );
    }

    #[test]
    fn test_the_reminder_is_late() {
        // Late is said first because it changes what somebody does next, so
        // the window is scanned in the state with the most to say. One row
        // of each kind, so the scan meets the list and every button.
        let rows = due_rows();
        assert!(
            rows.iter().all(|row| row.late),
            "a row is on time, so the late wording is never on screen"
        );
        let kinds: Vec<_> = rows.iter().map(|row| row.identity.kind).collect();
        assert_eq!(
            kinds.len(),
            crate::application::due::Kind::ALL.len(),
            "a kind has no row, so its buttons and wording are never scanned"
        );
    }

    #[test]
    fn test_the_event_repeats_so_the_question_is_asked() {
        // `which_days_are_meant` answers for the whole series without a
        // window when the event does not repeat.
        assert!(
            crate::application::calendar::asking_is_needed(&repeating_event().repeats),
            "the event does not repeat, so the question is never asked"
        );
    }
}

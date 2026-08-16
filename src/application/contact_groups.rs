//! What a contact group is called, and what writing to one comes to.
//!
//! A group is a name kept on this computer over people in the address book.
//! It is made here, it is read here, and it goes nowhere: nothing sends one to
//! Google or to Microsoft, and nothing reads theirs in. That is said in the
//! product through [`STAYS_ON_THIS_COMPUTER`] rather than only in the
//! changelog, because somebody who already keeps groups in Gmail will look for
//! them here and find an empty sidebar.
//!
//! # Why the words live here
//!
//! Two of them are heard rather than read: the sidebar says what a group is
//! and how many people are in it, and writing to one says who it is about to
//! reach. Both are sentences somebody acts on, so both are tested, and a
//! sentence built inline in a window handler cannot be.
//!
//! The counting matters more than it looks. A group resolves through the
//! address on each contact's own line, so a member whose only address is one
//! of their extra ones contributes nothing. Reaching seven people out of ten
//! in silence is the version of that nobody finds out about until somebody is
//! left off a thread, so [`writing_to`] says both numbers.

/// What a group is called when it is read out.
///
/// The count is part of the name rather than a second announcement: the tree
/// reads a row when focus lands on it and says nothing more, so anything not
/// in the row is not said at all.
pub fn spoken(name: &str, members: usize) -> String {
    match members {
        // Not "0 people". A group somebody has just made is waiting for them
        // to put somebody in it, and that is what the row should say.
        0 => format!("{name}, nobody in it yet"),
        1 => format!("{name}, 1 person"),
        many => format!("{name}, {many} people"),
    }
}

/// The branch the groups hang under in the contacts sidebar.
///
/// A level of its own, so the tree says these are groups and each row only has
/// to say which group and how many people are in it. Always present, even with
/// nothing under it, because a branch that appears once somebody has made
/// their first group is a branch nobody knows to look for before then.
pub fn the_groups_branch(groups: usize) -> String {
    match groups {
        0 => "Groups, none yet".to_string(),
        some => format!("Groups, {some}"),
    }
}

/// Addresses as one To line, with the blanks and the repeats taken out.
///
/// Joined with ", ", which is what `mail_controller::addresses` splits on.
/// Repeats are compared without case, because two contacts holding one address
/// spelled two ways is ordinary and sending somebody two copies of everything
/// is how people stop reading a group.
///
/// The first spelling is the one kept, because it is the one somebody typed.
pub fn recipients(addresses: &[String]) -> String {
    let mut kept: Vec<&str> = Vec::new();
    for address in addresses {
        let address = address.trim();
        if address.is_empty() {
            continue;
        }
        if kept.iter().any(|seen| seen.eq_ignore_ascii_case(address)) {
            continue;
        }
        kept.push(address);
    }
    kept.join(", ")
}

/// What came of asking to write to a group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Writing {
    /// There is nobody to write to, and this says what would fix it.
    Refused(String),
    /// Open a message to these people, saying this first.
    Opens {
        /// The To line, ready to put in the window.
        to: String,
        /// What is announced as the window opens.
        said: String,
    },
}

/// Whether a group can be written to, and to whom.
///
/// `members` is how many people are in the group and `addresses` is what the
/// address lookup came back with, which is never more and is often fewer.
pub fn writing_to(name: &str, members: usize, addresses: &[String]) -> Writing {
    if members == 0 {
        // A different problem from the one below, and a different fix, so a
        // different sentence. One sentence for both would send half the people
        // who hear it to the wrong place.
        return Writing::Refused(format!(
            "{name} has nobody in it yet. Put somebody in the group first."
        ));
    }
    let to = recipients(addresses);
    if to.is_empty() {
        return Writing::Refused(format!(
            "Nobody in {name} has an email address, so there is nowhere to write."
        ));
    }
    // Counted off the To line rather than off what came back, so the number
    // said and the number of addresses in the window are the same number.
    let reaching = to.split(", ").count();
    Writing::Opens {
        said: about_to_write(name, members, reaching),
        to,
    }
}

/// What is said as the window opens.
fn about_to_write(name: &str, members: usize, reaching: usize) -> String {
    if reaching < members {
        return format!(
            "Writing to {name}, {reaching} of {members} people. The others have no email address."
        );
    }
    match reaching {
        1 => format!("Writing to {name}, 1 person."),
        many => format!("Writing to {name}, {many} people."),
    }
}

/// Said when somebody has been put in a group.
///
/// The count is the group as it stands now, through [`spoken`], because that
/// is the number somebody acts on.
pub fn put_in(person: &str, group: &str, members: usize) -> String {
    format!("{person} put in {}.", spoken(group, members))
}

/// Said when they were already in it.
///
/// Not a failure and not the same as having just put them in. Somebody who
/// hears this knows the group is right and there is nothing left to do.
pub fn already_in(person: &str, group: &str) -> String {
    format!("{person} is already in {group}.")
}

/// Said when somebody has been taken out of a group.
///
/// The second sentence is not padding. This sits one line from Delete on the
/// same menu, one of the two cannot be undone, and the difference between them
/// is the whole reason for saying anything at all.
pub fn taken_out(person: &str, group: &str, members: usize) -> String {
    format!(
        "{person} taken out of {}. {person} is still in your address book.",
        spoken(group, members)
    )
}

/// Said when they were not in it to begin with.
pub fn not_in(person: &str, group: &str) -> String {
    format!("{person} is not in {group}.")
}

/// What is said once a group has been made.
///
/// Shorter than [`STAYS_ON_THIS_COMPUTER`], which the window that asked for
/// the name has already shown. This is the status line, and a status line that
/// reads out three sentences is one people learn to talk over.
pub fn made(name: &str) -> String {
    format!("Contact group \"{name}\" made, and kept on this computer.")
}

/// Where a contact group lives, said to the person making one.
///
/// On the dialog that makes a group and in the sentence said afterwards. A
/// group is the one container here that looks like something a provider would
/// carry, so somebody who keeps groups in Gmail will reasonably expect these
/// to be the same groups. They are not, and being told once at the moment of
/// making one is cheaper than finding out on a second device.
pub const STAYS_ON_THIS_COMPUTER: &str = "Contact groups are kept on this computer. A group made here is not sent to Google or \
     Outlook, and a group you already keep there does not appear in this address book.";

/// What the contacts sidebar's current selection narrows the list to.
///
/// A group could have been named by its id alone and left the sidebar to
/// remember what "no group" means, but that is two states (all contacts,
/// favorites) with no group id to carry, against one where every state
/// carries exactly what it needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shown {
    /// Every contact, the sidebar's own resting state.
    Everyone,
    /// Only the contacts marked as a favorite.
    Favorites,
    /// Only the contacts recorded as members of this group, by id.
    Group(String),
}

/// Whether a contact belongs under the sidebar's current selection.
///
/// `member_ids` is who the selected group holds, resolved by the caller
/// before asking; this does not go looking for a group by the id
/// [`Shown::Group`] carries, so a group that has since been renamed or
/// deleted cannot make this answer something it was never asked.
///
/// An empty `member_ids` under [`Shown::Group`] shows nobody rather than
/// falling back to everyone. A group with nobody in it is a real, empty
/// answer, and reading an empty list as "no filter" would make it
/// indistinguishable from All Contacts instead of the empty group it is.
pub fn belongs(contact_id: &str, is_favorite: bool, shown: &Shown, member_ids: &[String]) -> bool {
    match shown {
        Shown::Everyone => true,
        Shown::Favorites => is_favorite,
        Shown::Group(_) => member_ids.iter().any(|member| member == contact_id),
    }
}

/// Said when the sidebar selection changes what the contact list shows.
///
/// Deliberately not "loaded": nothing was fetched over the network or read
/// off disk here, the list already held every contact and this only narrows
/// which of them are shown, so claiming a load would describe work that did
/// not happen.
pub fn now_showing(label: &str, count: usize) -> String {
    match count {
        // Matches the group row's own "nobody in it yet": a selection that
        // narrows the list to nothing is a real, complete answer, not an
        // error, and reads the same way the row that led to it does.
        0 => format!("{label}, nobody to show"),
        some => format!(
            "{label}, {} shown",
            crate::service::caldav::how_many(some, "contact")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_group_says_how_many_people_are_in_it() {
        assert_eq!(spoken("Team A", 3), "Team A, 3 people");
        assert_eq!(spoken("Team A", 1), "Team A, 1 person");
        assert_eq!(spoken("Team A", 0), "Team A, nobody in it yet");
    }

    #[test]
    fn test_the_same_address_twice_goes_out_once() {
        // Two contacts can hold one address, and a mailing list that sends
        // somebody two copies of everything is how people stop reading it.
        // The first spelling is kept, because that is the one somebody typed.
        let lines = [
            " ada@x.example ".to_string(),
            "Ada@X.example".to_string(),
            "bob@x.example".to_string(),
        ];

        assert_eq!(recipients(&lines), "ada@x.example, bob@x.example");
    }

    #[test]
    fn test_an_address_that_is_nothing_but_space_is_not_a_recipient() {
        let lines = ["   ".to_string(), "bob@x.example".to_string()];

        assert_eq!(recipients(&lines), "bob@x.example");
    }

    #[test]
    fn test_nobody_to_write_to_leaves_the_line_empty_rather_than_full_of_commas() {
        assert_eq!(recipients(&[]), "");
        assert_eq!(recipients(&["".to_string(), " ".to_string()]), "");
    }

    #[test]
    fn test_an_empty_group_and_a_group_with_no_addresses_are_different_problems() {
        // One is fixed by putting somebody in the group and the other by
        // giving somebody an address, so one sentence for both would send
        // half the people who read it to the wrong place.
        let empty = writing_to("Team A", 0, &[]);
        let no_addresses = writing_to("Team A", 2, &[]);

        let Writing::Refused(empty) = empty else {
            panic!("an empty group should refuse");
        };
        let Writing::Refused(no_addresses) = no_addresses else {
            panic!("a group with no addresses should refuse");
        };

        assert_ne!(empty, no_addresses);
        assert!(empty.contains("nobody in it"), "{empty}");
        assert!(no_addresses.contains("email address"), "{no_addresses}");
    }

    #[test]
    fn test_writing_to_a_group_says_who_it_reaches() {
        let all = writing_to("Team A", 2, &["a@x.example".into(), "b@x.example".into()]);

        let Writing::Opens { to, said } = all else {
            panic!("a group with addresses should open a message");
        };
        assert_eq!(to, "a@x.example, b@x.example");
        assert_eq!(said, "Writing to Team A, 2 people.");
    }

    #[test]
    fn test_the_people_it_cannot_reach_are_counted_out_loud() {
        // The address it resolves is the one on the contact's own line, so
        // somebody whose only address is in their extra addresses is missed.
        // Reaching seven of ten silently is the version of this that nobody
        // finds out about until somebody is left off a thread.
        let some = writing_to("Team A", 3, &["a@x.example".into()]);

        let Writing::Opens { said, .. } = some else {
            panic!("one address is still somebody to write to");
        };
        assert_eq!(
            said,
            "Writing to Team A, 1 of 3 people. The others have no email address."
        );
    }

    #[test]
    fn test_one_person_is_not_counted_in_the_plural() {
        let one = writing_to("Team A", 1, &["a@x.example".into()]);

        let Writing::Opens { said, .. } = one else {
            panic!("one person is somebody to write to");
        };
        assert_eq!(said, "Writing to Team A, 1 person.");
    }

    #[test]
    fn test_the_same_address_twice_is_counted_once_when_it_is_announced() {
        // Otherwise the sentence says it reaches two people and the To line
        // carries one address, and the person hearing it has no way to tell
        // which is right.
        let twice = writing_to("Team A", 2, &["a@x.example".into(), "A@X.example".into()]);

        let Writing::Opens { to, said } = twice else {
            panic!("a group with an address should open a message");
        };
        assert_eq!(to, "a@x.example");
        assert_eq!(
            said,
            "Writing to Team A, 1 of 2 people. The others have no email address."
        );
    }

    #[test]
    fn test_the_branch_says_when_there_are_no_groups_yet() {
        // An empty branch in a tree is a stop somebody opens and learns
        // nothing from. Saying there are none is what stops them looking for
        // a group they think they have lost.
        assert_eq!(the_groups_branch(0), "Groups, none yet");
        assert_eq!(the_groups_branch(1), "Groups, 1");
        assert_eq!(the_groups_branch(4), "Groups, 4");
    }

    #[test]
    fn test_each_answer_about_membership_says_something_different() {
        // Four answers rather than "done" and "not done". Somebody who hears
        // "already in Team A" knows the group is right and no work is needed;
        // somebody who hears "put in Team A" knows it just changed. One
        // sentence for both makes the second indistinguishable from the first.
        let said = [
            put_in("Ada Lovelace", "Team A", 3),
            already_in("Ada Lovelace", "Team A"),
            taken_out("Ada Lovelace", "Team A", 2),
            not_in("Ada Lovelace", "Team A"),
        ];

        for one in &said {
            assert!(one.contains("Ada Lovelace"), "{one}");
            assert!(one.contains("Team A"), "{one}");
        }
        let mut unique = said.to_vec();
        unique.sort();
        unique.dedup();
        assert_eq!(
            unique.len(),
            said.len(),
            "two answers read the same: {said:?}"
        );
    }

    #[test]
    fn test_taking_somebody_out_of_a_group_says_they_are_still_in_the_address_book() {
        // It sits one line from Delete on the same menu, and one of the two
        // cannot be undone. Saying which this was is what stops the panic.
        let said = taken_out("Ada Lovelace", "Team A", 2);

        assert!(said.contains("still in your address book"), "{said}");
    }

    #[test]
    fn test_membership_counts_the_group_after_the_change() {
        // The number people act on is what the group holds now, not what it
        // held a moment ago.
        assert!(put_in("Ada Lovelace", "Team A", 1).contains("1 person"));
        assert!(put_in("Ada Lovelace", "Team A", 3).contains("3 people"));
        assert!(taken_out("Ada Lovelace", "Team A", 0).contains("nobody in it"));
    }

    #[test]
    fn test_making_a_group_does_not_name_an_account_that_will_never_see_it() {
        // It used to say "Contact group \"Team A\" created in me@gmail.com",
        // because the group was filed under the default account and the
        // sentence read that account's name back out. Nothing ever sent it
        // there.
        let said = made("Team A");

        assert!(said.contains("Team A"), "{said}");
        assert!(said.contains("this computer"), "{said}");
        assert!(!said.contains('@'), "{said} names an account");
    }

    #[test]
    fn test_the_warning_is_in_the_product_and_not_only_in_the_changelog() {
        // What this cannot see: whether anybody is shown the warning. It reads
        // the window's source for the sentence. The sentence could sit in a
        // branch nothing reaches and this would not know.
        // A warning that lives only in release notes is a warning nobody
        // gets. Somebody who keeps groups in Gmail has to be told where these
        // ones live at the moment they make one.
        let managers =
            std::fs::read_to_string("src/presentation/managers.rs").expect("the commands");
        assert!(
            managers.contains("STAYS_ON_THIS_COMPUTER"),
            "nothing in the product says where a group is kept"
        );

        let privacy = std::fs::read_to_string("docs/privacy.md").expect("the privacy page");
        assert!(
            privacy.to_lowercase().contains("contact group"),
            "the privacy page does not mention contact groups"
        );
    }

    #[test]
    fn test_there_is_one_way_to_resolve_a_group() {
        // Two resolvers is how the wrong one gets used. There was an unwired
        // second copy in an unreachable module for as long as groups have
        // existed, formatting its own comma-joined string from a different
        // set of contacts.
        // Assembled rather than written out, so this file does not match its
        // own check and report a resolver that is only the test looking for
        // one. The first version did exactly that.
        let looking_for = format!("fn resolve_group_{}", "emails");
        let mut found: Vec<String> = Vec::new();
        walk("src".as_ref(), &mut |path: &std::path::Path| {
            let text = std::fs::read_to_string(path).unwrap_or_default();
            if text.contains(&looking_for) {
                found.push(path.display().to_string());
            }
        });

        assert_eq!(
            found.len(),
            1,
            "a group can be resolved more than one way: {found:?}"
        );
    }

    /// Every Rust file under a directory, so a check covers the whole tree
    /// rather than the files somebody remembered.
    #[cfg(test)]
    fn walk(dir: &std::path::Path, seen: &mut impl FnMut(&std::path::Path)) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, seen);
            } else if path.extension().is_some_and(|e| e == "rs") {
                seen(&path);
            }
        }
    }

    #[test]
    fn test_the_sentence_about_staying_here_names_no_mechanism() {
        // Read aloud in a dialog. It has to say where a group lives and what
        // that means for somebody who already has groups somewhere else.
        assert!(STAYS_ON_THIS_COMPUTER.contains("this computer"));
        assert!(STAYS_ON_THIS_COMPUTER.contains("address book"));
        for machine in ['_', '{'] {
            assert!(
                !STAYS_ON_THIS_COMPUTER.contains(machine),
                "{STAYS_ON_THIS_COMPUTER} reads as an identifier"
            );
        }
        for jargon in ["API", "contactGroups", "sync"] {
            assert!(
                !STAYS_ON_THIS_COMPUTER.contains(jargon),
                "{jargon} is a word about the machine, not about the person"
            );
        }
    }

    #[test]
    fn test_everyone_is_shown_under_the_all_contacts_selection() {
        assert!(belongs("c1", false, &Shown::Everyone, &[]));
        assert!(belongs("c1", true, &Shown::Everyone, &[]));
    }

    #[test]
    fn test_only_favorites_are_shown_under_the_favorites_selection() {
        assert!(belongs("c1", true, &Shown::Favorites, &[]));
        assert!(!belongs("c1", false, &Shown::Favorites, &[]));
    }

    #[test]
    fn test_a_named_group_shows_only_the_contacts_recorded_as_its_members() {
        let members = ["c1".to_string(), "c2".to_string()];
        let shown = Shown::Group("g1".to_string());

        assert!(belongs("c1", false, &shown, &members));
        assert!(belongs("c2", false, &shown, &members));
        assert!(!belongs("c3", false, &shown, &members));
    }

    #[test]
    fn test_a_group_with_no_members_shows_nobody_rather_than_falling_back_to_everyone() {
        // The bug this guards against: mistaking an empty member list for "no
        // filter", which would make an empty group indistinguishable from All
        // Contacts instead of the empty group it really is.
        let shown = Shown::Group("g1".to_string());

        assert!(!belongs("c1", false, &shown, &[]));
        assert!(!belongs("c1", true, &shown, &[]));
    }

    #[test]
    fn test_membership_ignores_favorite_status() {
        // Being a favorite and being in a named group are two different
        // questions; a favorite who is not in the group does not sneak in.
        let members = ["c2".to_string()];
        let shown = Shown::Group("g1".to_string());

        assert!(!belongs("c1", true, &shown, &members));
    }

    #[test]
    fn test_the_sidebar_selection_is_named_when_the_list_changes_to_match_it() {
        assert_eq!(
            now_showing("All Contacts", 3),
            "All Contacts, 3 contacts shown"
        );
        assert_eq!(now_showing("Favorites", 1), "Favorites, 1 contact shown");
    }

    #[test]
    fn test_the_sidebar_selection_says_nobody_rather_than_zero_contacts_shown_is_awkward() {
        // Not wrong either way, but this is the same shape every other count
        // in this module already takes, and a screen reader user hears this
        // one right after the group's own "nobody in it yet".
        assert_eq!(now_showing("Empty Group", 0), "Empty Group, nobody to show");
    }
}

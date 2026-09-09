//! A contact taken out of one group and put in another, against a real store.
//!
//! A move here is not the move the other four kinds get. An event, a task or a
//! note is in one container, so moving it is one write that names the new one.
//! A contact is in as many groups as somebody puts it in, so a move is two
//! writes over a join table, and the whole risk is what happens between them.
//! If the take-out runs first and the put-in fails, the contact is in neither
//! group, which nobody can see and nobody can correct. So the put-in runs first
//! and the take-out second, and the pair is one transaction.
//!
//! The other risk is the move quietly becoming the put-in that already ships.
//! Adding somebody to a second group without taking them out of the first is a
//! copy, and it is a real act this program offers under its own name. A move
//! that forgot its second half would look exactly like it, report success, and
//! leave the contact where it was.
//!
//! An integration test rather than unit tests beside the code. A `#[test]` in
//! `src/data/message_cache/contacts.rs` costs 34 guard records a re-measurement
//! each, which is a build and a full library run apiece, on the critical path of
//! every commit that adds one. The question also spans the write and the read:
//! only `load_contact_groups` says what the store holds afterwards.
//!
//! What this cannot see. It opens no chooser, so it says nothing about which
//! groups somebody is offered or how the question reads; that is
//! `application::contact_groups`'s and `tests/wired.rs`'s to answer. It presses
//! no key. And nothing here has been heard by a screen reader.

use wixen_mail::application::destinations::Filing;
use wixen_mail::application::new_item::ItemKind;
use wixen_mail::common::what_ships::what_ships;
use wixen_mail::data::message_cache::{ContactGroup, MessageCache, MovedBetweenGroups};
use wixen_mail::presentation::managers::file_under;

/// The account every fixture here stores under.
const ACCOUNT: &str = "acct";

/// The three groups, and the one person moved between them.
const HOME: &str = "group-home";
const ELSEWHERE: &str = "group-elsewhere";
const UNTOUCHED: &str = "group-untouched";
const ADA: &str = "contact-ada";
const GRACE: &str = "contact-grace";

fn a_store(dir: &tempfile::TempDir) -> MessageCache {
    MessageCache::new(dir.path().join("groups.db"), None).expect("a store to write into")
}

fn a_group(id: &str, name: &str) -> ContactGroup {
    ContactGroup {
        id: id.to_string(),
        account_id: ACCOUNT.to_string(),
        name: name.to_string(),
        description: None,
        created_at: String::new(),
        member_ids: Vec::new(),
    }
}

/// Three groups, with nobody in them yet.
///
/// No contact rows are made. Membership is recorded by identifier and the
/// membership write never reads the contact, so a contact row would only be
/// fixture that proves nothing. The layer that reads a name to say a sentence
/// is `presentation::managers`, and it is not what this file is about.
fn three_groups(cache: &MessageCache) {
    for (id, name) in [
        (HOME, "Team A"),
        (ELSEWHERE, "Team B"),
        (UNTOUCHED, "Everybody else"),
    ] {
        cache
            .create_contact_group(&a_group(id, name))
            .expect("a group to move between");
    }
}

/// Who is in that group, as the store holds it now.
fn who_is_in(cache: &MessageCache, group_id: &str) -> Vec<String> {
    let mut members = cache
        .load_contact_groups(ACCOUNT)
        .expect("the groups to read back")
        .into_iter()
        .find(|group| group.id == group_id)
        .unwrap_or_else(|| panic!("a group called {group_id}"))
        .member_ids;
    members.sort();
    members
}

#[test]
fn test_a_contact_moved_from_one_group_to_another_is_in_the_new_one_and_not_the_old() {
    // The whole act, in the one shape somebody asks for it. Both halves have
    // to happen: a move that only put her in the new group is the put-in this
    // program already had, under a name that says something else happened.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    three_groups(&cache);
    cache
        .add_contact_to_group(HOME, ADA)
        .expect("Ada in the group she is leaving");

    let answer = cache
        .move_contact_between_groups(ADA, HOME, ELSEWHERE)
        .expect("the move to be written");

    assert_eq!(answer, MovedBetweenGroups::Moved);
    assert_eq!(
        who_is_in(&cache, ELSEWHERE),
        vec![ADA.to_string()],
        "she is not in the group she was moved into"
    );
    assert!(
        who_is_in(&cache, HOME).is_empty(),
        "she is still in the group she was moved out of, so this was a copy \
         wearing a move's name"
    );
}

#[test]
fn test_a_contact_that_is_not_in_the_group_it_would_leave_is_not_put_in_the_other_one() {
    // This is the reason there is a transaction, and the only failure anybody
    // can reach from outside. Whether she is really in the group she is
    // leaving is answered by the take-out itself, by how many rows it removed,
    // because a read before the write can be stale by the time the write runs.
    // That answer arrives after the put-in has already happened, so the put-in
    // has to be undone, and the only thing that undoes it is the transaction.
    //
    // Two loose statements pass every other test in this file and fail this
    // one: she ends up in the group she was going to, and nothing says the move
    // did not happen.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    three_groups(&cache);

    let answer = cache
        .move_contact_between_groups(ADA, HOME, ELSEWHERE)
        .expect("the store to answer rather than fail");

    assert_eq!(answer, MovedBetweenGroups::NotInTheGroupItWouldLeave);
    assert!(
        who_is_in(&cache, ELSEWHERE).is_empty(),
        "she was put in the group she was moving to although there was nothing \
         to move, so the put-in was not undone"
    );
    assert!(
        who_is_in(&cache, HOME).is_empty(),
        "the group she was not in gained her"
    );
}

#[test]
fn test_a_move_into_the_group_it_is_leaving_writes_nothing() {
    // Carrying this one out would put her in the group and then take her out of
    // it, leaving her in no group at all and saying she had been moved. The
    // chooser cannot offer it, which is exactly why the write is asked as well:
    // the chooser is one route in and this is the other.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    three_groups(&cache);
    cache
        .add_contact_to_group(HOME, ADA)
        .expect("Ada in one group");

    let answer = cache
        .move_contact_between_groups(ADA, HOME, HOME)
        .expect("the store to answer rather than fail");

    assert_eq!(answer, MovedBetweenGroups::IntoTheOneItIsLeaving);
    assert_eq!(
        who_is_in(&cache, HOME),
        vec![ADA.to_string()],
        "she was taken out of the group she was going to stay in"
    );
}

#[test]
fn test_a_move_into_a_group_it_is_already_in_still_takes_it_out_of_the_one_it_left() {
    // The chooser leaves out the groups she is already in, so this arrives only
    // by a route that did not go through it. It still has to be right: the
    // put-in is ignored because she is there, and the take-out is the half that
    // matters. Getting this wrong by refusing the whole move would leave her in
    // the group she asked to leave.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    three_groups(&cache);
    cache.add_contact_to_group(HOME, ADA).expect("Ada in one");
    cache
        .add_contact_to_group(ELSEWHERE, ADA)
        .expect("Ada in the other");

    let answer = cache
        .move_contact_between_groups(ADA, HOME, ELSEWHERE)
        .expect("the move to be written");

    assert_eq!(answer, MovedBetweenGroups::Moved);
    assert_eq!(
        who_is_in(&cache, ELSEWHERE),
        vec![ADA.to_string()],
        "she is in the group she was already in twice, or not at all"
    );
    assert!(
        who_is_in(&cache, HOME).is_empty(),
        "she is still in the group she was moved out of"
    );
}

#[test]
fn test_a_move_leaves_every_other_contact_and_group_where_they_were() {
    // What a take-out written with one condition too few would break: a DELETE
    // naming only the group takes everybody out of it, and a DELETE naming only
    // the contact takes her out of every group she is in. Neither shows up in a
    // test that watches one person and two groups.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    three_groups(&cache);
    cache.add_contact_to_group(HOME, ADA).expect("Ada here");
    cache.add_contact_to_group(HOME, GRACE).expect("Grace here");
    cache
        .add_contact_to_group(UNTOUCHED, ADA)
        .expect("Ada somewhere else too");

    cache
        .move_contact_between_groups(ADA, HOME, ELSEWHERE)
        .expect("the move to be written");

    assert_eq!(
        who_is_in(&cache, HOME),
        vec![GRACE.to_string()],
        "somebody else was taken out of the group the move emptied"
    );
    assert_eq!(
        who_is_in(&cache, UNTOUCHED),
        vec![ADA.to_string()],
        "she was taken out of a group the move was never about"
    );
}

#[test]
fn test_filing_a_contact_through_file_under_is_refused_rather_than_reported_as_done() {
    // The trap this whole plan is written around, and the compiler says
    // nothing about it. `file_under`'s last arm answered the three kinds with
    // no container by handing back the identifier it was given, which is
    // success with nothing written. Its comment said a new kind of item would
    // be a compile error there, which is true of a new `ItemKind` variant and
    // false of an existing kind moving off that list, which is exactly what
    // giving a contact a move does.
    //
    // So widening the command to a contact and leaving this arm alone
    // compiles, runs, says "Ada Lovelace moved to Team B", and writes nothing.
    // This is the test that would notice.
    let dir = tempfile::tempdir().expect("somewhere to put the store");
    let cache = a_store(&dir);
    three_groups(&cache);
    cache.add_contact_to_group(HOME, ADA).expect("Ada in one");

    let answer = file_under(
        &cache,
        ItemKind::Contact,
        ADA,
        ELSEWHERE,
        ACCOUNT,
        Filing::Moving,
    );

    assert!(
        answer.is_err(),
        "filing a contact into a group here reported success: {answer:?}"
    );
    assert!(
        who_is_in(&cache, ELSEWHERE).is_empty(),
        "it wrote something after all"
    );
    assert_eq!(
        who_is_in(&cache, HOME),
        vec![ADA.to_string()],
        "it took her out of the group she was in"
    );
}

#[test]
fn test_a_contact_is_filed_by_its_groups_rather_than_by_the_path_that_names_one_container() {
    // What this cannot see: whether the key is pressed, whether the window
    // opens, or whether anybody hears the answer. It reads the source of the
    // arm the two filing commands share and asks where a contact goes from
    // there.
    //
    // Why it exists. `where_it_could_go` opens with `kind.kept_in()?` and a
    // contact answers `None`, so a contact that reaches the ordinary filing
    // path falls out of it with `None`, which the dispatcher treats as
    // "somebody left the window without choosing" and says nothing at all. A
    // key that does nothing and says nothing is indistinguishable from a
    // broken keyboard, and no test in this repository reaches that arm: it is
    // in `managers.rs`, which 42 guard records fingerprint, so it has no unit
    // tests of its own.
    //
    // It lives in this file rather than in `tests/wired.rs` for the same
    // reason. Fourteen records name `wired.rs`, so a test added there costs
    // fourteen re-measurements inside the commit gate; this file is named by
    // one.
    // Read as the text between the two arms the filing commands share. There
    // have to be two: one that answers a contact and one that answers the three
    // kinds kept in a single container. Merging them back into one is the
    // regression, and it takes the first `expect` below with it.
    const SHARED: &str = "PimCommand::Move | PimCommand::Copy";
    let managers =
        std::fs::read_to_string("src/presentation/managers.rs").expect("the manager sources");
    let ships = what_ships(&managers);
    let below_the_first = ships
        .split_once(SHARED)
        .expect("the arm the two filing commands share")
        .1;
    let arm = below_the_first
        .split_once(SHARED)
        .expect("a second filing arm, for the kinds that are kept in one container")
        .0;

    assert!(
        arm.contains("ItemKind::Contact"),
        "the filing arm no longer sends a contact anywhere of its own, so it \
         goes to the path that asks which single container holds it, gets \
         nothing back, and says nothing"
    );
    assert!(
        arm.contains("move_a_contact_between_groups"),
        "nothing raises the move between groups, so Ctrl+Shift+V on a contact \
         is a key that does nothing"
    );
    assert!(
        arm.contains("change_the_group_a_contact_is_in"),
        "the copy of a contact no longer reaches the put-in that already \
         ships, so either it does nothing or somebody has written a second one"
    );
}

#[test]
fn test_each_group_question_is_asked_of_the_groups_that_question_has_an_answer_in() {
    // What this cannot see: whether the window opens, what it reads like, or
    // whether the two questions are told apart by ear. It reads which list each
    // chooser is handed.
    //
    // Why it exists, and why it is a second check rather than part of the one
    // above. `could_leave` and `could_join` have unit tests, and those stay
    // green when the chooser stops being handed their answers: the regression
    // is one word at the call site, `&groups` where `&leaving` was, and it
    // leaves the pure functions untouched. What it costs is somebody hearing
    // every group in the account read out where they expected the two they are
    // in, and choosing one the contact is not in asks for a move that cannot
    // happen. The wiring lives in `managers.rs`, which 42 guard records
    // fingerprint, so it has no unit tests of its own.
    let managers =
        std::fs::read_to_string("src/presentation/managers.rs").expect("the manager sources");
    let ships = what_ships(&managers);
    let rest = ships
        .split_once("pub fn move_a_contact_between_groups")
        .expect("the move between groups")
        .1;
    let body: String = rest[..rest.find("\n}\n").unwrap_or(rest.len())]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    assert!(
        body.contains("which_of_these_groups(frame,&leaving,"),
        "the question about which group is being left is asked of some other \
         list than the groups the contact is in"
    );
    assert!(
        body.contains("which_of_these_groups(frame,&joining,"),
        "the question about where the contact is going is asked of some other \
         list than the groups it is not in"
    );
    assert!(
        !body.contains("which_group("),
        "a contact's move asks the chooser that offers every group in the \
         account, so both questions are the whole sidebar read out"
    );
}

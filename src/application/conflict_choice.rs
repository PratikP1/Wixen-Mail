//! Both copies of one thing, kept because they disagree and nobody has chosen
//! yet.
//!
//! # What this is for
//!
//! A contact or a calendar item can be changed here and changed at the provider
//! between two syncs. One of those two copies is about to be lost, and until
//! this module existed nothing asked which. Contacts decided on somebody's
//! behalf and told them afterwards, in a count that used to be called
//! `replaced`; the calendar decided and told them nothing at all.
//!
//! Telling somebody after the fact is not the same as asking. An edit that
//! disappeared with a sentence about it in a summary they may not have heard is
//! an edit that disappeared.
//!
//! # What is in here and what is not
//!
//! Values in and values out. No window type, no sync, no database. That is what
//! lets the sentences somebody hears be tested without a running window, and it
//! is why the fields, the wording and the choice all live together here rather
//! than beside the code that stores them.
//!
//! Who wins is decided elsewhere and is not decided again here.
//! [`crate::application::contacts_sync::whose_copy_wins`] already answers it for
//! contacts, comparing version markers rather than clocks, and this module
//! changes what happens in one of its arms rather than adding a second opinion.
//! Two decisions about who wins disagree the first time either one changes.
//!
//! # What has never been checked
//!
//! No account and no calendar server has ever been used with this program, so
//! every conflict here is two divergent local states driven through the same
//! code path a sync uses. Whether the labelled pair below is understood by ear
//! is recorded in the broken windows ledger rather than claimed.

/// One named value out of one copy of a contact or a calendar item.
///
/// A name somebody would recognise rather than a column name: "Telephone", not
/// `phones_json`. These are read aloud.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AField {
    /// What to call it when it is read out.
    pub called: String,
    /// What this copy holds, or empty where this copy holds nothing.
    pub value: String,
}

impl AField {
    /// A field with a name and a value.
    pub fn new(called: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            called: called.into(),
            value: value.into(),
        }
    }
}

/// Which of the two copies somebody is talking about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhichCopy {
    /// The one on this computer, holding work nobody has sent.
    Here,
    /// The one the address book or the calendar server holds.
    TheProviders,
}

/// What the copy that is not on this computer should be called.
///
/// A parameter rather than a second set of sentences. "The address book" is
/// wrong for a calendar and "the calendar" is wrong for a contact, and the two
/// would come to differ the first time either was reworded. One set of words
/// with a hole in it cannot drift from itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheOtherCopy {
    /// A contact, in Google Contacts or Outlook.
    AnAddressBook,
    /// A calendar item, at a CalDAV server.
    ACalendar,
}

impl TheOtherCopy {
    /// What to call it in the middle of a sentence.
    pub fn called(self) -> &'static str {
        match self {
            TheOtherCopy::AnAddressBook => "your address book",
            TheOtherCopy::ACalendar => "your calendar",
        }
    }

    /// What to call the kind of thing itself.
    pub fn the_thing(self) -> &'static str {
        match self {
            TheOtherCopy::AnAddressBook => "contact",
            TheOtherCopy::ACalendar => "calendar item",
        }
    }
}

/// Both copies of one thing, and enough to show them.
///
/// Kept whole rather than as a difference, because somebody choosing needs to
/// hear what each copy actually says. A difference alone reads as an
/// instruction to reconstruct the two copies in your head, which is the memory
/// load the cognitive rule in `CLAUDE.md` is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BothCopies {
    /// What the thing is called, so the question names it.
    pub what_it_is_called: String,
    /// What to call the copy that is not on this computer.
    pub other_copy: TheOtherCopy,
    /// This computer's copy.
    pub here: Vec<AField>,
    /// The provider's copy.
    pub theirs: Vec<AField>,
}

impl BothCopies {
    /// The fields the two copies do not agree about, by the name each is read
    /// out under.
    ///
    /// Every field either copy names, not only the ones both name: a value one
    /// side holds and the other does not is a disagreement, and reading only
    /// the intersection hides exactly the case where somebody added a telephone
    /// number in one place.
    ///
    /// In the order this computer's copy lists them, then whatever only the
    /// provider's copy names. An order somebody can predict, rather than
    /// whatever a set happened to hand back, because this is read aloud.
    pub fn fields_that_differ(&self) -> Vec<String> {
        let mut differ: Vec<String> = Vec::new();
        for field in &self.here {
            if self.value_of(&field.called, WhichCopy::TheProviders) != Some(&field.value) {
                differ.push(field.called.clone());
            }
        }
        for field in &self.theirs {
            if self.here.iter().all(|held| held.called != field.called) {
                differ.push(field.called.clone());
            }
        }
        differ
    }

    /// What one copy holds under that name, where it names it at all.
    fn value_of(&self, called: &str, which: WhichCopy) -> Option<&String> {
        self.values_in(which)
            .iter()
            .find(|field| field.called == called)
            .map(|field| &field.value)
    }

    /// One copy's values.
    pub fn values_in(&self, which: WhichCopy) -> &[AField] {
        match which {
            WhichCopy::Here => &self.here,
            WhichCopy::TheProviders => &self.theirs,
        }
    }

    /// What each copy is introduced by.
    ///
    /// Every version somebody hears is introduced by which copy it is. Two
    /// unlabelled blocks read out one after the other are two blocks nobody can
    /// tell apart, which is the whole failure this is here to prevent.
    pub fn label_for(&self, which: WhichCopy) -> String {
        match which {
            WhichCopy::Here => "What is on this computer".to_string(),
            WhichCopy::TheProviders => format!("What {} has", self.other_copy.called()),
        }
    }

    /// The one sentence said on arrival: what is being asked, and how much
    /// disagrees.
    ///
    /// The count rather than the list, because the list is read out underneath
    /// and saying it twice is the flooding guardrail 5 forbids. Whether the
    /// count is useful or is a sentence somebody stops hearing is unverified by
    /// ear and is in the ledger.
    pub fn what_is_being_asked(&self) -> String {
        let differ = self.fields_that_differ();
        let thing = self.other_copy.the_thing();
        let named = &self.what_it_is_called;
        let provider = self.other_copy.called();
        match differ.as_slice() {
            [only] => format!(
                "You changed this {thing} here and it changed in {provider} as well. \
                 1 field is different: {only}. Choose which copy to keep."
            ),
            several => format!(
                "You changed {named} here and it changed in {provider} as well. \
                 {} fields are different: {}. Choose which copy to keep.",
                several.len(),
                several.join(", ")
            ),
        }
    }

    /// What is said once somebody has chosen.
    pub fn what_was_chosen(&self, which: WhichCopy) -> String {
        match which {
            WhichCopy::Here => format!(
                "Kept what is on this computer for {}. It goes to {} on the next sync.",
                self.what_it_is_called,
                self.other_copy.called()
            ),
            WhichCopy::TheProviders => format!(
                "Kept what {} has for {}. The change made here is gone.",
                self.other_copy.called(),
                self.what_it_is_called
            ),
        }
    }
}

/// What choosing one copy calls for.
///
/// Named rather than boolean because the two are not opposites of one act. One
/// of them still has to reach the provider and the other has already arrived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatChoosingCallsFor {
    /// Keep this computer's copy. It is still owed to the provider, so the hold
    /// clears and the change goes back to waiting to be sent.
    KeepWhatIsHereAndSendIt,
    /// Take the provider's copy. Nothing is sent: it is already what they hold,
    /// and sending it back would be this computer telling a provider its own
    /// words.
    TakeTheirsAndSendNothing,
}

/// What a sync says at the end about the disagreements it found.
///
/// One sentence for both syncs, with a hole for what kind of thing and whose
/// copy, so the address book's version and the calendar's cannot come to
/// differ. `allowed::changes_waiting_here` is shared between the two syncs for
/// the same reason, and its own comment records that two copies of it drifted
/// and only one was corrected.
///
/// It names the menu item rather than a module, because a sentence saying
/// something is waiting and not saying where it is waiting sends somebody
/// looking. Whether the count is useful or is a sentence somebody stops
/// hearing is unverified by ear.
pub fn how_many_are_waiting_to_be_chosen(count: usize, other: TheOtherCopy) -> String {
    let provider = other.called();
    // Written out both ways rather than built from parts, because three words
    // have to agree in number and a sentence assembled from fragments reads
    // like one. `allowed::changes_waiting_here` says the same thing about
    // itself.
    match count {
        1 => format!(
            "1 {} changed here and in {provider} as well. Use Choose Which Copy \
             to Keep, on the Tools menu, to say which copy to keep; nothing is \
             sent until you do",
            other.the_thing()
        ),
        many => format!(
            "{many} {}s changed here and in {provider} as well. Use Choose Which \
             Copy to Keep, on the Tools menu, to say which copy to keep; nothing \
             is sent until you do",
            other.the_thing()
        ),
    }
}

/// What somebody did with the window that asked.
///
/// Named rather than a bare identifier, so the arm that means "they did not
/// answer" cannot be reached by falling off the end of a match on numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatWasPressed {
    /// The button for this computer's copy.
    KeepWhatIsHere,
    /// The button for the provider's copy.
    KeepTheirs,
    /// Escape, the close box, or the button that says so. Anything that is not
    /// one of the two answers is this.
    LeftWithoutChoosing,
}

/// Which copy that means to keep, where it means one at all.
///
/// `None` is a real answer and the important one: somebody who closed the
/// window has not chosen, the hold stays, and nothing is written or sent. A
/// closed window read as either copy winning is a choice made by whoever wrote
/// the match rather than by the person.
pub fn what_that_means(pressed: WhatWasPressed) -> Option<WhichCopy> {
    match pressed {
        WhatWasPressed::KeepWhatIsHere => Some(WhichCopy::Here),
        WhatWasPressed::KeepTheirs => Some(WhichCopy::TheProviders),
        WhatWasPressed::LeftWithoutChoosing => None,
    }
}

/// What choosing that copy calls for.
pub fn choosing(which: WhichCopy) -> WhatChoosingCallsFor {
    match which {
        WhichCopy::Here => WhatChoosingCallsFor::KeepWhatIsHereAndSendIt,
        WhichCopy::TheProviders => WhatChoosingCallsFor::TakeTheirsAndSendNothing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn both(here: &[(&str, &str)], theirs: &[(&str, &str)]) -> BothCopies {
        BothCopies {
            what_it_is_called: "Ada Lovelace".to_string(),
            other_copy: TheOtherCopy::AnAddressBook,
            here: here.iter().map(|(n, v)| AField::new(*n, *v)).collect(),
            theirs: theirs.iter().map(|(n, v)| AField::new(*n, *v)).collect(),
        }
    }

    #[test]
    fn test_one_field_apart_is_named_on_its_own() {
        let pair = both(
            &[("Name", "Ada Lovelace"), ("Telephone", "01234")],
            &[("Name", "Ada Lovelace"), ("Telephone", "05678")],
        );
        assert_eq!(pair.fields_that_differ(), vec!["Telephone".to_string()]);
    }

    #[test]
    fn test_several_fields_apart_are_all_named_in_the_order_this_computer_lists_them() {
        let pair = both(
            &[
                ("Name", "Ada Lovelace"),
                ("Telephone", "01234"),
                ("Company", "Analytical Engines"),
            ],
            &[
                ("Name", "Ada King"),
                ("Telephone", "01234"),
                ("Company", "Difference Engines"),
            ],
        );
        assert_eq!(
            pair.fields_that_differ(),
            vec!["Name".to_string(), "Company".to_string()]
        );
    }

    #[test]
    fn test_two_copies_that_agree_name_no_field() {
        let pair = both(&[("Name", "Ada")], &[("Name", "Ada")]);
        assert!(pair.fields_that_differ().is_empty());
    }

    #[test]
    fn test_a_field_only_one_copy_holds_is_a_difference() {
        // The case reading only the fields both copies name would hide, which
        // is the common one: somebody adds a telephone number in one place.
        let added_here = both(
            &[("Name", "Ada"), ("Telephone", "01234")],
            &[("Name", "Ada")],
        );
        assert_eq!(
            added_here.fields_that_differ(),
            vec!["Telephone".to_string()]
        );
        let added_there = both(
            &[("Name", "Ada")],
            &[("Name", "Ada"), ("Telephone", "01234")],
        );
        assert_eq!(
            added_there.fields_that_differ(),
            vec!["Telephone".to_string()]
        );
    }

    #[test]
    fn test_each_copy_is_introduced_by_which_copy_it_is() {
        let pair = both(&[("Name", "Ada")], &[("Name", "Ada King")]);
        assert_eq!(
            pair.label_for(WhichCopy::Here),
            "What is on this computer",
            "this computer's copy has to say it is this computer's copy"
        );
        assert_eq!(
            pair.label_for(WhichCopy::TheProviders),
            "What your address book has",
            "the other copy has to say whose it is"
        );
    }

    #[test]
    fn test_a_calendar_is_not_called_an_address_book() {
        let mut pair = both(&[("Summary", "Standup")], &[("Summary", "Stand-up")]);
        pair.other_copy = TheOtherCopy::ACalendar;
        pair.what_it_is_called = "Standup".to_string();
        assert_eq!(
            pair.label_for(WhichCopy::TheProviders),
            "What your calendar has"
        );
        assert!(
            pair.what_is_being_asked().contains("calendar item"),
            "a calendar item is not a contact: {}",
            pair.what_is_being_asked()
        );
    }

    #[test]
    fn test_the_opening_sentence_says_what_is_asked_and_how_many_fields_differ() {
        let pair = both(
            &[("Name", "Ada"), ("Telephone", "01234"), ("Company", "AE")],
            &[
                ("Name", "Ada King"),
                ("Telephone", "01234"),
                ("Company", "DE"),
            ],
        );
        let said = pair.what_is_being_asked();
        assert!(
            said.contains("2 fields are different"),
            "the count has to be in it: {said}"
        );
        assert!(
            said.contains("Name, Company"),
            "the fields have to be named: {said}"
        );
        assert!(
            said.contains("Choose which copy to keep"),
            "it has to say what is being asked of somebody: {said}"
        );
    }

    #[test]
    fn test_one_differing_field_is_not_said_in_the_plural() {
        let pair = both(&[("Telephone", "01234")], &[("Telephone", "05678")]);
        let said = pair.what_is_being_asked();
        assert!(said.contains("1 field is different"), "{said}");
        assert!(!said.contains("fields are"), "{said}");
    }

    #[test]
    fn test_choosing_this_computers_copy_still_owes_it_to_the_provider() {
        assert_eq!(
            choosing(WhichCopy::Here),
            WhatChoosingCallsFor::KeepWhatIsHereAndSendIt
        );
    }

    #[test]
    fn test_choosing_the_providers_copy_sends_nothing_back_to_them() {
        assert_eq!(
            choosing(WhichCopy::TheProviders),
            WhatChoosingCallsFor::TakeTheirsAndSendNothing
        );
    }

    #[test]
    fn test_what_was_chosen_says_which_copy_survived_and_what_happens_next() {
        let pair = both(&[("Name", "Ada")], &[("Name", "Ada King")]);
        let kept_here = pair.what_was_chosen(WhichCopy::Here);
        assert!(kept_here.contains("on this computer"), "{kept_here}");
        assert!(
            kept_here.contains("goes to your address book"),
            "somebody has to be told the change still has to travel: {kept_here}"
        );
        let took_theirs = pair.what_was_chosen(WhichCopy::TheProviders);
        assert!(
            took_theirs.contains("your address book has"),
            "{took_theirs}"
        );
        assert!(
            took_theirs.contains("change made here is gone"),
            "losing work is said plainly rather than implied: {took_theirs}"
        );
    }

    #[test]
    fn test_the_sync_sentence_says_where_the_choice_is_made() {
        let said = how_many_are_waiting_to_be_chosen(1, TheOtherCopy::AnAddressBook);
        assert!(
            said.contains("Choose Which Copy to Keep"),
            "a sentence saying something is waiting and not saying where sends \
             somebody looking: {said}"
        );
        assert!(
            said.contains("nothing is sent until you do"),
            "somebody has to be told their change is not going anywhere while \
             they decide: {said}"
        );
    }

    #[test]
    fn test_the_sync_sentence_names_the_kind_of_thing_and_whose_copy() {
        let contacts = how_many_are_waiting_to_be_chosen(2, TheOtherCopy::AnAddressBook);
        assert!(contacts.contains("2 contacts"), "{contacts}");
        assert!(contacts.contains("your address book"), "{contacts}");
        let calendar = how_many_are_waiting_to_be_chosen(2, TheOtherCopy::ACalendar);
        assert!(calendar.contains("2 calendar items"), "{calendar}");
        assert!(calendar.contains("your calendar"), "{calendar}");
        assert!(
            !calendar.contains("address book"),
            "a calendar item read out under the address book's words: {calendar}"
        );
    }

    #[test]
    fn test_one_thing_waiting_is_not_said_in_the_plural() {
        let said = how_many_are_waiting_to_be_chosen(1, TheOtherCopy::ACalendar);
        assert!(said.contains("1 calendar item changed"), "{said}");
        assert!(!said.contains("items"), "{said}");
    }

    #[test]
    fn test_leaving_without_choosing_chooses_nothing() {
        assert_eq!(
            what_that_means(WhatWasPressed::LeftWithoutChoosing),
            None,
            "a window somebody closed was read as an answer, so the hold went \
             and one of the two copies with it"
        );
    }

    #[test]
    fn test_each_button_means_the_copy_it_names() {
        assert_eq!(
            what_that_means(WhatWasPressed::KeepWhatIsHere),
            Some(WhichCopy::Here)
        );
        assert_eq!(
            what_that_means(WhatWasPressed::KeepTheirs),
            Some(WhichCopy::TheProviders)
        );
    }

    #[test]
    fn test_a_copys_own_values_are_handed_back_under_the_copy_they_belong_to() {
        let pair = both(&[("Name", "Ada")], &[("Name", "Ada King")]);
        assert_eq!(pair.values_in(WhichCopy::Here)[0].value, "Ada");
        assert_eq!(pair.values_in(WhichCopy::TheProviders)[0].value, "Ada King");
    }
}

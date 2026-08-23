//! What a delete or a toggle does, and what somebody is asked first.
//!
//! Six panels can each make something and none of them can remove one. The
//! cache has had the methods all along; nothing ever called them. A note folder
//! you can create and cannot delete is a gap in a shipped feature rather than
//! leftover code, which is why they were kept when the visibility pass found
//! them.
//!
//! This is the pure half: which item a command lands on, whether it needs
//! confirming, and the exact words used. The words matter more here than in
//! most places. A confirmation is read aloud in full before the buttons are
//! reached, so it has to name the thing being destroyed rather than say "this
//! item", and it has to be short enough that nobody learns to answer before it
//! finishes.

use crate::application::new_item::{ContainerKind, ItemKind};

/// A command that acts on whatever is selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PimCommand {
    /// Remove it. Always confirmed.
    Delete,
    /// Done or not done, for tasks and reminders.
    ToggleComplete,
    /// Pinned or not, for notes.
    TogglePin,
    /// Put it in another calendar, list, folder or group.
    ///
    /// Filed in the wrong place is the ordinary case, not the unusual one: a
    /// task typed in a hurry lands on whichever list was open, and without this
    /// the only way to correct it was to delete it and type it again.
    Move,
}

impl PimCommand {
    /// Whether this command means anything for that kind of item.
    ///
    /// A menu item that does nothing on the panel you are looking at is worse
    /// than one that is not there: it is a stop in the menu that teaches
    /// nothing and costs a moment every time it is passed.
    pub const fn applies_to(self, kind: ItemKind) -> bool {
        match self {
            // Mail has its own Delete, on the message list, with the server
            // semantics that go with it. This is the personal information side
            // only.
            Self::Delete => !matches!(kind, ItemKind::Mail),
            Self::ToggleComplete => matches!(kind, ItemKind::Task | ItemKind::Reminder),
            Self::TogglePin => matches!(kind, ItemKind::Note),
            // Not a contact: a contact is in as many groups as somebody puts
            // it in, so there is no one home to move it out of. Not a
            // reminder: the module holds buckets worked out from when each one
            // is due, and there is nothing to move it to. Mail moves between
            // folders by its own path, which has to talk to the server.
            Self::Move => matches!(kind, ItemKind::Event | ItemKind::Task | ItemKind::Note),
        }
    }
}

/// One command, and the thing it lands on.
///
/// Bundled because they are one idea: what to do, to which kind of item, at
/// which row. Passing them separately made the function that carries them out
/// take eight arguments, which clippy objected to and was right about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PimAction {
    pub command: PimCommand,
    pub kind: ItemKind,
    /// The selected row in that panel's list, or `None` when nothing is chosen.
    pub row: Option<usize>,
}

/// What to ask before destroying something.
///
/// Named, not "this item". Somebody who has arrowed to the wrong row and
/// pressed Delete finds out from the question, and only if the question says
/// which row.
///
/// One sentence. A confirmation is read in full before its buttons are reached,
/// and a long one teaches people to answer before it has finished.
pub fn confirm_delete(kind: ItemKind, name: &str) -> String {
    let named = match name.trim() {
        // Both are possible: an untitled note, or a row whose title never
        // loaded. "Delete the note?" is still answerable; "Delete ?" is not.
        "" => format!("this {}", thing(kind)),
        title => format!("\"{title}\""),
    };
    format!("Delete {named}? This cannot be undone.")
}

/// What one of these is called in a sentence.
const fn thing(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Mail => "message",
        ItemKind::Contact => "contact",
        ItemKind::Event => "event",
        ItemKind::Reminder => "reminder",
        ItemKind::Task => "task",
        ItemKind::Note => "note",
    }
}

/// The same word with the article that belongs in front of it.
///
/// One of the six starts with a vowel, and every sentence that wrote "a" and
/// then the word got that one wrong. Written once here so the next sentence to
/// name a kind cannot get it wrong again.
const fn a_thing(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Mail => "a message",
        ItemKind::Contact => "a contact",
        ItemKind::Event => "an event",
        ItemKind::Reminder => "a reminder",
        ItemKind::Task => "a task",
        ItemKind::Note => "a note",
    }
}

/// What to say once it is gone.
///
/// Said rather than left silent. A row disappearing is not something somebody
/// listening can see, and silence after a confirmed delete is indistinguishable
/// from a delete that failed.
pub fn deleted(kind: ItemKind, name: &str) -> String {
    match name.trim() {
        "" => format!("{} deleted", capitalise(thing(kind))),
        title => format!("{title} deleted"),
    }
}

/// What to say once something has been moved.
///
/// Names where it went. "Moved" alone leaves somebody who chose from a tree of
/// twenty calendars with no way to know which one they landed on, and the whole
/// reason to move a thing is to put it somewhere in particular.
pub fn moved(name: &str, into: &str) -> String {
    match name.trim() {
        "" => format!("Moved to {into}"),
        title => format!("{title} moved to {into}"),
    }
}

/// What to say when the item is one a provider holds and the move cannot be sent.
///
/// Neither Google nor Microsoft is asked here to move a task to another list or
/// an event to another calendar. Doing it means deleting the item where it is,
/// creating it again where it is going, and writing the identity that comes
/// back over the old one, and none of that is built. Filing it in the new
/// container on this computer alone leaves the two ends disagreeing: the next
/// push would ask the provider to update an item in a container it is not in,
/// which is refused every time, and the next pull would put it back where the
/// provider still has it.
///
/// So the move is refused and the reason said. It names what does work, because
/// "not yet" on its own leaves somebody with nothing to try.
pub fn cannot_be_moved(kind: ItemKind, holder: ContainerKind, name: &str) -> String {
    let named = match name.trim() {
        "" => format!("This {}", thing(kind)),
        title => format!("\"{title}\""),
    };
    format!(
        "{named} is held by the account it came from, and moving one of those to another \
         {} is not something this can do yet. Nothing has been moved. {} made on this \
         computer can be moved.",
        holder.label().to_lowercase(),
        capitalise(a_thing(kind))
    )
}

/// What to say when the container chosen is one this program can only read.
///
/// The other half of [`cannot_be_moved`], on the axis that one does not ask
/// about. That one asks whether the item can be moved; this one asks whether
/// the place it is going could ever hold it. A calendar somebody subscribed to,
/// or one a calendar server marks as read-only, takes nothing: the row would be
/// filed there on this computer, marked as waiting to be sent, and every sync
/// from then on would look at it, find nothing that could send it, and leave it
/// exactly where it was. The move was offered, accepted and announced as done,
/// and nothing ever happened.
///
/// Same shape as the item refusal: what is true, then that nothing was moved,
/// then what does work. Two sentences of the same shape are one thing to learn
/// rather than two.
pub fn cannot_be_moved_into(kind: ItemKind, holder: ContainerKind, container_name: &str) -> String {
    let holder_name = holder.label().to_lowercase();
    let named = match container_name.trim() {
        // A row whose name never loaded. "That calendar" is still answerable.
        "" => format!("That {holder_name}"),
        name => format!("\"{name}\""),
    };
    format!(
        "{named} is a {holder_name} this program can only read, and {} moved into it could \
         never be sent. Nothing has been moved. {} you can change can hold it.",
        a_thing(kind),
        capitalise(&format!("a {holder_name}")),
    )
}

/// What to say when the row a confirmed command was about has gone.
///
/// Between the question and the answer somebody else's sync, or another window,
/// can take the row away. Returning quietly at that point is the worst of both:
/// the question was answered, so something is expected to have happened, and
/// nothing is said either way. Somebody listening then presses the key again on
/// whichever row moved up into the selection.
///
/// It does not say deleted, because nothing was. It says the row has gone and
/// that nothing was changed, which are the two facts worth having.
pub fn no_longer_there(kind: ItemKind, name: &str) -> String {
    something_no_longer_there(thing(kind), name)
}

/// The same sentence for something [`ItemKind`] has no word for.
///
/// A contact group is the case today. No command here acts on one, so there is
/// no kind for it, and adding one so that a sentence could name it would put a
/// variant into every match that switches on a kind. The words are the same
/// words either way; only what is being named differs, and that is the only
/// thing this takes.
///
/// Eight places wrote a shorter version of this for themselves, so which
/// sentence somebody heard depended on which panel they were in, and half of
/// those left off the part saying nothing had happened.
pub fn something_no_longer_there(what: &str, name: &str) -> String {
    let named = match name.trim() {
        // A row whose title never loaded, the same case the question itself
        // copes with. "That event is no longer there" is still a sentence.
        "" => format!("That {what}"),
        title => format!("\"{title}\""),
    };
    format!("{named} is no longer there. Nothing has been changed.")
}

/// What to say when a command was carried out and the storage refused it.
///
/// The whole family used to share one sentence, "That did not work", with
/// whatever the storage layer said appended. Read aloud, that names nothing: a
/// failed delete and a failed move are the same words, the row is not named,
/// and there is nothing in it to act on. So this says what was being done, to
/// which row, why it did not happen, that the row is as it was, and what to try.
///
/// The reason arrives as it comes, with a full stop or without one, so the stop
/// is put on here. Two in a row is heard as a stumble rather than seen as a
/// typo.
pub fn did_not_happen(command: PimCommand, kind: ItemKind, name: &str, reason: &str) -> String {
    let named = match name.trim() {
        "" => format!("that {}", thing(kind)),
        title => format!("\"{title}\""),
    };
    // Worded so that it is true whichever way a toggle was going. Nothing here
    // knows which way it went, and "Ticking off" over an untick would be a
    // sentence saying the opposite of what was asked for.
    let tried = match command {
        PimCommand::Delete => format!("Deleting {named}"),
        PimCommand::ToggleComplete => format!("Changing whether {named} is done"),
        PimCommand::TogglePin => format!("Changing whether {named} is pinned"),
        PimCommand::Move => format!("Moving {named}"),
    };
    format!(
        "{tried} did not work: {}. Nothing has been changed. Try it again, and if it \
         keeps happening, close Wixen Mail and open it again.",
        reason.trim().trim_end_matches('.')
    )
}

/// What to say after a toggle, which has to name the new state.
///
/// "Done" rather than "toggled". The whole point of a toggle is that you
/// cannot tell which way it went without being told, and somebody listening
/// has no tick box to glance at.
pub fn toggled(command: PimCommand, name: &str, now: bool) -> String {
    let state = match (command, now) {
        (PimCommand::ToggleComplete, true) => "done",
        (PimCommand::ToggleComplete, false) => "not done",
        (PimCommand::TogglePin, true) => "pinned",
        (PimCommand::TogglePin, false) => "unpinned",
        (PimCommand::Delete, _) => "deleted",
        // Not a state something is now in. Where a thing went is the whole of
        // what is worth saying about a move, and that needs the destination's
        // name, which this does not have. Said by `moved` instead.
        (PimCommand::Move, _) => "moved",
    };
    match name.trim() {
        "" => capitalise(state),
        title => format!("{title}, {state}"),
    }
}

/// The short word `Event::Confirmed`'s tone carries, when this command is one
/// of the toggles it covers.
///
/// `None` for [`PimCommand::Delete`] and [`PimCommand::Move`]: a delete asks
/// first and is announced by [`deleted`], and a move needs the destination's
/// name, which [`moved`] already carries. Neither is the "did the small
/// thing I asked for happen" fact `Event::Confirmed` exists for.
///
/// Deliberately not [`toggled`]'s sentence. That names the item, which
/// somebody who just acted on the row they were sitting on already knows;
/// this is the one short word every toggle site shares, so flag, mark done
/// and pin all sound like the same fact rather than three near-identical
/// ones.
pub fn confirmed_detail(command: PimCommand, now: bool) -> Option<&'static str> {
    match command {
        PimCommand::ToggleComplete => Some(if now {
            "Marked done"
        } else {
            "Marked not done"
        }),
        PimCommand::TogglePin => Some(if now { "Pinned" } else { "Unpinned" }),
        PimCommand::Delete | PimCommand::Move => None,
    }
}

fn capitalise(value: &str) -> String {
    let mut characters = value.chars();
    match characters.next() {
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_confirmation_names_what_it_will_destroy() {
        // Somebody who arrowed to the wrong row finds out from the question,
        // and only if the question says which row.
        let asked = confirm_delete(ItemKind::Task, "File the tax return");

        assert!(asked.contains("File the tax return"), "{asked}");
        assert!(asked.contains("cannot be undone"), "{asked}");
    }

    #[test]
    fn test_an_untitled_item_is_still_asked_about_answerably() {
        // An untitled note is a real thing, and so is a row whose title never
        // loaded. "Delete the note?" can be answered; "Delete ?" cannot.
        let asked = confirm_delete(ItemKind::Note, "   ");

        assert!(asked.contains("this note"), "{asked}");
        assert!(!asked.contains("\"\""), "{asked}");
    }

    #[test]
    fn test_a_confirmation_is_one_sentence_long() {
        // It is read in full before its buttons are reached. A long one
        // teaches people to answer before it has finished, which is how a
        // confirmation stops being one.
        let asked = confirm_delete(ItemKind::Event, "Dentist");

        assert!(asked.len() < 80, "{} characters: {asked}", asked.len());
    }

    #[test]
    fn test_a_toggle_says_which_way_it_went() {
        // The whole point of a toggle is that you cannot tell without being
        // told, and there is no tick box to glance at.
        assert!(toggled(PimCommand::ToggleComplete, "Buy milk", true).contains("done"));
        assert!(toggled(PimCommand::ToggleComplete, "Buy milk", false).contains("not done"));
        assert!(toggled(PimCommand::TogglePin, "Ideas", true).contains("pinned"));
        assert!(toggled(PimCommand::TogglePin, "Ideas", false).contains("unpinned"));
    }

    #[test]
    fn test_not_done_is_not_read_as_done() {
        // "done" is a substring of "not done", so a careless check would pass
        // on the wrong one. This is the assertion that catches it.
        let off = toggled(PimCommand::ToggleComplete, "Buy milk", false);

        assert!(off.ends_with("not done"), "{off}");
    }

    #[test]
    fn test_a_deletion_is_announced_because_a_row_vanishing_is_invisible() {
        assert_eq!(deleted(ItemKind::Task, "Buy milk"), "Buy milk deleted");
        assert_eq!(deleted(ItemKind::Task, ""), "Task deleted");
    }

    #[test]
    fn test_a_move_says_what_moved_and_where_it_went() {
        // Silence after a move is indistinguishable from a move that failed,
        // and "moved" without a destination leaves somebody who chose from a
        // tree of twenty lists no way to know where they landed.
        assert_eq!(moved("Buy milk", "Shopping"), "Buy milk moved to Shopping");
    }

    #[test]
    fn test_a_refused_move_says_it_did_not_happen_and_what_does_work() {
        // "Not yet" on its own leaves somebody trying it again on the next
        // task, and a refusal that does not say the row is untouched reads
        // exactly like a move that half worked.
        let said = cannot_be_moved(ItemKind::Task, ContainerKind::TaskList, "Book the dentist");

        assert!(said.contains("Book the dentist"), "{said}");
        assert!(said.contains("task list"), "{said}");
        assert!(said.contains("Nothing has been moved"), "{said}");
        assert!(
            said.contains("made on this computer can be moved"),
            "{said}"
        );
    }

    #[test]
    fn test_a_refused_move_of_an_untitled_row_is_still_a_sentence() {
        let said = cannot_be_moved(ItemKind::Event, ContainerKind::Calendar, "  ");

        assert!(said.starts_with("This event is held"), "{said}");
        assert!(said.contains("another calendar"), "{said}");
    }

    #[test]
    fn test_a_move_into_a_container_that_can_only_be_read_says_so_and_what_works() {
        // The other axis of the same refusal. The item was fine to move and
        // the destination was not, and the sentence has to name the
        // destination, because that is the part somebody chose.
        let said = cannot_be_moved_into(ItemKind::Event, ContainerKind::Calendar, "Term dates");

        assert!(said.contains("Term dates"), "{said}");
        assert!(said.contains("can only read"), "{said}");
        assert!(said.contains("Nothing has been moved"), "{said}");
        assert!(said.contains("A calendar you can change"), "{said}");
    }

    #[test]
    fn test_the_refusal_names_the_kind_with_the_article_that_belongs_with_it() {
        // Read aloud, so "a event" is heard rather than skimmed past. An event
        // is the one of the six kinds that starts with a vowel, so it is the
        // only one any of these sentences can get wrong.
        let said = cannot_be_moved(ItemKind::Event, ContainerKind::Calendar, "Dentist");

        assert!(said.contains("An event made on this computer"), "{said}");
        assert!(!said.contains("A event"), "{said}");
    }

    #[test]
    fn test_a_move_of_an_untitled_row_still_says_where_it_went() {
        // A row whose title never loaded still went somewhere, and where it
        // went is the part worth hearing.
        assert_eq!(moved("   ", "Shopping"), "Moved to Shopping");
    }

    #[test]
    fn test_a_command_is_only_offered_where_it_means_something() {
        // A menu item that does nothing on the panel you are on is a stop that
        // teaches nothing and costs a moment every time it is passed.
        assert!(PimCommand::ToggleComplete.applies_to(ItemKind::Task));
        assert!(PimCommand::ToggleComplete.applies_to(ItemKind::Reminder));
        assert!(!PimCommand::ToggleComplete.applies_to(ItemKind::Note));
        assert!(!PimCommand::ToggleComplete.applies_to(ItemKind::Contact));

        assert!(PimCommand::TogglePin.applies_to(ItemKind::Note));
        assert!(!PimCommand::TogglePin.applies_to(ItemKind::Task));
    }

    #[test]
    fn test_a_row_that_went_away_between_the_question_and_the_answer_is_said_not_left_silent() {
        // Somebody has answered a question about destroying something named,
        // and then a second question about which days it meant. Silence after
        // that is indistinguishable from a delete that worked, and the next
        // thing they do is press Delete again on whatever row moved up.
        let said = no_longer_there(ItemKind::Event, "Stand-up");

        assert!(said.contains("Stand-up"), "the row is not named: {said}");
        assert!(
            said.contains("Nothing has been changed"),
            "it does not say nothing happened: {said}"
        );
        assert!(
            !said.to_lowercase().contains("delet"),
            "it says something was deleted, and nothing was: {said}"
        );

        let untitled = no_longer_there(ItemKind::Event, "   ");
        assert_eq!(
            untitled, "That event is no longer there. Nothing has been changed.",
            "a row whose title never loaded left the sentence unfinished"
        );

        // A contact group and an attachment are the same fact about a different
        // thing, and there is no kind for either. Same sentence, so nobody has
        // to learn two.
        assert_eq!(
            something_no_longer_there("group", ""),
            "That group is no longer there. Nothing has been changed."
        );
        assert_eq!(
            something_no_longer_there("attachment", "Invoice.pdf"),
            "\"Invoice.pdf\" is no longer there. Nothing has been changed."
        );
        assert_eq!(
            no_longer_there(ItemKind::Task, "Buy milk"),
            something_no_longer_there("task", "Buy milk"),
            "the kind that has a word for itself gets a different sentence"
        );
    }

    #[test]
    fn test_a_command_that_failed_says_what_it_was_and_what_to_try() {
        // Every one of these commands used to fail into one sentence, "That did
        // not work", followed by whatever the storage layer had said. It named
        // neither the row nor what had been attempted, so somebody working by
        // ear could not tell a failed delete from a failed move, and it left
        // them with nothing to do about it.
        let said = did_not_happen(
            PimCommand::Delete,
            ItemKind::Task,
            "Buy milk",
            "the file is in use",
        );

        assert!(said.contains("Buy milk"), "the row is not named: {said}");
        assert!(
            said.contains("Deleting"),
            "what was tried is not named: {said}"
        );
        assert!(
            said.contains("the file is in use"),
            "the reason was dropped: {said}"
        );
        assert!(
            said.contains("Nothing has been changed"),
            "it does not say the row is untouched: {said}"
        );
        assert!(said.contains("Try it again"), "nothing to do next: {said}");

        // Each command names itself, so a failed toggle does not read as a
        // failed delete.
        let every: std::collections::BTreeSet<String> = [
            PimCommand::Delete,
            PimCommand::ToggleComplete,
            PimCommand::TogglePin,
            PimCommand::Move,
        ]
        .into_iter()
        .map(|command| did_not_happen(command, ItemKind::Task, "Buy milk", "the file is in use"))
        .collect();
        assert_eq!(
            every.len(),
            4,
            "two commands fail in the same words: {every:?}"
        );

        // A row whose title never loaded, the case every sentence here copes
        // with.
        let untitled = did_not_happen(PimCommand::Delete, ItemKind::Note, "   ", "no room left");
        assert!(untitled.contains("that note"), "{untitled}");
        assert!(!untitled.contains("\"\""), "{untitled}");

        // The reason arrives with or without a full stop on the end, and two in
        // a row is heard as a stumble.
        let stopped = did_not_happen(PimCommand::Move, ItemKind::Event, "Dentist", "it broke.");
        assert!(!stopped.contains(".."), "{stopped}");
    }

    #[test]
    fn test_mail_keeps_its_own_delete() {
        // The message list's Delete has server semantics behind it, and this
        // is the personal information side only. Two commands on one key would
        // be one of them doing the wrong thing.
        assert!(!PimCommand::Delete.applies_to(ItemKind::Mail));
        for kind in [
            ItemKind::Contact,
            ItemKind::Event,
            ItemKind::Reminder,
            ItemKind::Task,
            ItemKind::Note,
        ] {
            assert!(PimCommand::Delete.applies_to(kind), "{kind:?}");
        }
    }

    #[test]
    fn test_confirmed_detail_names_which_way_a_toggle_went() {
        // Short and generic on purpose, unlike `toggled`'s sentence: this is
        // the one word every toggle site shares with `Event::Confirmed`, not
        // a sentence naming the row, which somebody who just acted on the row
        // they were sitting on already knows.
        assert_eq!(
            confirmed_detail(PimCommand::ToggleComplete, true),
            Some("Marked done")
        );
        assert_eq!(
            confirmed_detail(PimCommand::ToggleComplete, false),
            Some("Marked not done")
        );
        assert_eq!(
            confirmed_detail(PimCommand::TogglePin, true),
            Some("Pinned")
        );
        assert_eq!(
            confirmed_detail(PimCommand::TogglePin, false),
            Some("Unpinned")
        );
    }

    #[test]
    fn test_confirmed_detail_is_silent_for_delete_and_move() {
        // Neither is the "did the small thing I asked for happen" fact
        // `Event::Confirmed` exists for: a delete asks first and is announced
        // by `deleted`, and a move needs the destination `moved` already
        // names.
        assert_eq!(confirmed_detail(PimCommand::Delete, true), None);
        assert_eq!(confirmed_detail(PimCommand::Move, false), None);
    }
}

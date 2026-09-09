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

use crate::application::destinations::Filing;
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
    /// Put a second one in another calendar, list or folder, leaving the first
    /// where it is.
    ///
    /// The same shape twice is the ordinary case too: a task that is on this
    /// week's list and next week's, an event that belongs to two calendars. The
    /// only way to make one was to type it again.
    Copy,
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
            // A contact is in as many groups as somebody puts it in, so there
            // is no one home to move it out of. That was the reason for
            // leaving it out and it is still true; what changed is that the
            // program now asks which group is being left, and the answer to
            // that question is the move. It does not come through the path
            // below: `kept_in` still answers `None` for a contact, and
            // `managers::pim_command` sends a contact to the group-membership
            // path before the chooser that names one container is reached.
            //
            // A reminder was left out because the module holds buckets worked
            // out from when each one is due, and a bucket is not a place. That
            // was true and it was about the wrong container. The one a reminder
            // really has, and has had since the table was written, is the
            // account, so moving one means sending it to another account. It
            // does not come through the path below either: `kept_in` answers
            // `None` for a reminder, and `managers::pim_command` sends one to
            // the account path before the chooser that names one container is
            // reached.
            //
            // Mail is still out. It moves between folders by its own path,
            // which has to talk to the server.
            //
            // A copy answers the same, and for a contact it is the put-in that
            // already ships. A second contact would be a second person, which
            // is not something anybody wants; a second group membership is
            // exactly what copying one means, and it had no way in from this
            // family of commands. A second reminder in another account is an
            // ordinary thing to want, and a copy is offered the account the
            // reminder is in as well, where a move is not.
            //
            // Written out rather than as "anything but mail", although the two
            // answer alike today. The five are here for five reasons and a
            // sixth kind of item should have to give its own.
            Self::Move | Self::Copy => matches!(
                kind,
                ItemKind::Event
                    | ItemKind::Task
                    | ItemKind::Note
                    | ItemKind::Contact
                    | ItemKind::Reminder
            ),
        }
    }

    /// Which of the two filing acts this is, for the commands that put
    /// something somewhere.
    ///
    /// `None` for the three that put nothing anywhere. Asked rather than
    /// decided at the call site, so the pairing lives beside the enum it is
    /// about and a fourth filing command cannot be wired to the wrong act.
    pub const fn filing(self) -> Option<Filing> {
        match self {
            Self::Move => Some(Filing::Moving),
            Self::Copy => Some(Filing::Copying),
            Self::Delete | Self::ToggleComplete | Self::TogglePin => None,
        }
    }
}

/// One account, named the way somebody hears it.
///
/// Not [`crate::application::destinations::Branch`], which is an account and
/// the places inside it. A reminder is kept in no place, so every branch here
/// would carry an empty `places` and a reader would have to work out that the
/// emptiness meant something rather than nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnAccount {
    /// The identifier the row is written with.
    pub id: String,
    /// What it is called when it is read out.
    ///
    /// Its name, with its address after it only where two accounts read alike.
    /// `presentation::folder_tree` decides that, because only it can see the
    /// whole set being offered, and this takes the answer rather than making a
    /// second rule for when an address is spoken.
    pub spoken: String,
}

/// The accounts a reminder can be filed into, given the one it is in now.
///
/// The account is the only container a reminder has ever had, so this is the
/// whole of the question a reminder's move asks.
///
/// The act decides whether the account it is in is one of them, and it is asked
/// rather than assumed: [`Filing::leaves_out_where_it_is`] is the same
/// predicate `where_it_could_go` reads for the three kinds that are kept in a
/// container, so a copy of a reminder is offered the account it is in for
/// exactly the reason a copy of a task is offered the list it is on. Offering
/// somebody the account a thing is already in is offering a move that does
/// nothing; putting a second reminder there is a real act somebody may want.
pub fn accounts_a_reminder_could_go_to(
    filing: Filing,
    accounts: &[AnAccount],
    in_now: &str,
) -> Vec<AnAccount> {
    accounts
        .iter()
        .filter(|account| !filing.leaves_out_where_it_is() || account.id != in_now)
        .cloned()
        .collect()
}

/// What to say when a move has nowhere to go, because there is one account.
///
/// Said before any window opens. A chooser with nothing in it is a window
/// somebody arrows through to find out there was never an answer, and a key
/// that opens nothing and says nothing is indistinguishable from a broken one.
///
/// Deliberately not [`is_not_kept_in_a_container`], which is the nearest
/// existing sentence and says the wrong thing. That one says a reminder cannot
/// be filed at all, and somebody who hears it stops trying: what is true is
/// that this computer has one account today and a second one gives the command
/// somewhere to go.
pub fn the_only_account_there_is(name: &str) -> String {
    let named = match name.trim() {
        "" => "That reminder".to_string(),
        title => format!("\"{title}\""),
    };
    format!(
        "{named} is in the one account set up on this computer, so there is nowhere else to \
         move it to. Nothing has been moved. Setting up a second account gives a reminder \
         somewhere to go."
    )
}

/// What to say when the account chosen turns out to be the one it is in.
///
/// The chooser cannot offer this, because a move leaves out the account the
/// reminder is in. It arrives when something moves the reminder between the
/// question and the answer, and by a route that never opened a chooser at all.
///
/// Not [`filed`], which is what the successful move says and is the sentence
/// nearest to hand here. Saying "moved to Work" about a reminder that was in
/// Work before the key was pressed is a report of an act that did not happen,
/// and somebody working by ear has no list to glance at to find out.
pub fn already_in_that_account(name: &str, account: &str) -> String {
    let named = match name.trim() {
        "" => "That reminder".to_string(),
        title => format!("\"{title}\""),
    };
    format!("{named} is already in {account}. Nothing has been moved.")
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

/// What is still owed to the account once the row here has been written.
///
/// A filing is two things that happen at different times and used to be
/// announced as one. The row moves on this computer at the moment somebody
/// presses the key; whether anything reaches the account is the next sync's
/// business, and for a task in a list nobody syncs it is nobody's business at
/// all. Saying only "moved to Work" is true of the first and silent about the
/// second, and somebody who cannot see the list has that sentence and nothing
/// else.
///
/// Three answers and not a boolean, because a change held by a setting is not
/// the same as one waiting for the next sync. The first has something the
/// person can do about it and the second has not, and telling somebody to go
/// and turn on a setting that is already on is worse than saying nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waiting {
    /// Nothing is. The item stays on this computer and no account is owed it.
    StaysHere,
    /// It is written here and the next sync sends it.
    ToBeSent,
    /// It is written here, and Allow Changes is what is keeping it here.
    HeldByTheSetting,
}

/// What is owed to the account, given whether anything will be sent at all.
///
/// Asked here rather than at the call site so the one case that matters cannot
/// be got wrong by being forgotten: the setting only decides anything when
/// something was going to be sent. A note filed in a folder is not held by
/// Allow Changes, it simply has nowhere to go, and reporting the setting for it
/// would send somebody to turn on a switch that would change nothing.
pub const fn what_is_waiting(_will_be_sent: bool, _changes_are_allowed: bool) -> Waiting {
    Waiting::StaysHere
}

/// What to say once something has been moved, or copied.
///
/// Names where it went. "Moved" alone leaves somebody who chose from a tree of
/// twenty calendars with no way to know which one they landed on, and the whole
/// reason to file a thing somewhere is to put it somewhere in particular.
///
/// One sentence taking the act rather than two sentences of the same shape.
/// Somebody who cannot see the two lists has only this to tell them which of
/// the two happened, so the two must differ; and a second sentence written
/// beside this one is a second sentence to keep in step with it.
///
/// And one sentence however much it has to carry. This is heard after every
/// single filing, so the clause about the account is joined on with a comma
/// rather than being a sentence of its own, and where the setting is named the
/// join is a colon. A paragraph read out after every move is the failure
/// guardrail 5 is about: feedback has to be bounded as well as distinct.
///
/// Nothing is added where nothing is waiting, which is most filings. A note
/// goes nowhere, and a task filed into a list made on this computer goes
/// nowhere either, whatever kind of account it is sitting on.
pub fn filed(filing: Filing, name: &str, into: &str, _waiting: Waiting) -> String {
    let done = did(filing);
    match name.trim() {
        "" => format!("{} to {into}", capitalise(done)),
        title => format!("{title} {done} to {into}"),
    }
}

/// The past tense of the act, for the middle of a sentence.
///
/// Apart from [`Filing::act`], which is the capitalised word a button and a
/// window title carry.
const fn did(filing: Filing) -> &'static str {
    match filing {
        Filing::Moving => "moved",
        Filing::Copying => "copied",
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
/// exactly where it was. The act was offered, accepted and announced as done,
/// and nothing ever happened.
///
/// This one asks about the destination, so it refuses a copy for exactly the
/// reason it refuses a move and takes the act as a parameter rather than being
/// written out twice. A copy announced with the word "moved" would tell
/// somebody who cannot see the list that their original had gone.
///
/// Same shape as the item refusal: what is true, then that nothing happened,
/// then what does work. Two sentences of the same shape are one thing to learn
/// rather than two.
pub fn cannot_be_filed_into(
    filing: Filing,
    kind: ItemKind,
    holder: ContainerKind,
    container_name: &str,
) -> String {
    let holder_name = holder.label().to_lowercase();
    let named = match container_name.trim() {
        // A row whose name never loaded. "That calendar" is still answerable.
        "" => format!("That {holder_name}"),
        name => format!("\"{name}\""),
    };
    let done = did(filing);
    format!(
        "{named} is a {holder_name} this program can only read, and {} {done} into it could \
         never be sent. Nothing has been {done}. {} you can change can hold it.",
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

/// What to say when something is filed into a container and its kind is not
/// kept in one.
///
/// The refusal for the arm nothing should reach. `file_under` used to answer
/// the three kinds with no container by returning the identifier it was given,
/// which is success with nothing written: a route that reached it was told the
/// filing had happened, and the row was exactly as before. A comment there said
/// a new kind of item would be a compile error, which is true of a new
/// [`ItemKind`] variant and false of an existing kind moving from the
/// never-reached list to the reached one, which is what giving a contact a move
/// does.
///
/// Deliberately not [`something_no_longer_there`], which was the first sentence
/// reached for and says the wrong thing: the row is there, and what is missing
/// is a container to file it in. Somebody who hears that a contact has gone
/// goes looking for a contact that never moved.
pub fn is_not_kept_in_a_container(kind: ItemKind) -> String {
    format!(
        "{} is not kept in one container, so there is nowhere here to file it. \
         Nothing has been changed.",
        capitalise(a_thing(kind))
    )
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
        PimCommand::Copy => format!("Copying {named}"),
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
        // what is worth saying about either of these, and that needs the
        // destination's name, which this does not have. Said by `filed`
        // instead.
        (PimCommand::Move, _) => "moved",
        (PimCommand::Copy, _) => "copied",
    };
    match name.trim() {
        "" => capitalise(state),
        title => format!("{title}, {state}"),
    }
}

/// The short word `Event::Confirmed`'s tone carries, when this command is one
/// of the toggles it covers.
///
/// `None` for [`PimCommand::Delete`], [`PimCommand::Move`] and
/// [`PimCommand::Copy`]: a delete asks first and is announced by [`deleted`],
/// and the two filing commands need the destination's name, which [`filed`]
/// already carries. None of the three is the "did the small thing I asked for
/// happen" fact `Event::Confirmed` exists for.
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
        PimCommand::Delete | PimCommand::Move | PimCommand::Copy => None,
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
        assert_eq!(
            filed(Filing::Moving, "Buy milk", "Shopping", Waiting::StaysHere),
            "Buy milk moved to Shopping"
        );
    }

    #[test]
    fn test_a_copy_says_it_was_copied_and_never_that_it_moved() {
        // The one thing somebody who cannot see the two lists has to go on.
        // Heard as "moved", a copy says the original has gone from the list
        // they were sitting on, and the next thing they do is go looking for
        // it.
        let said = filed(Filing::Copying, "Buy milk", "Shopping", Waiting::StaysHere);

        assert_eq!(said, "Buy milk copied to Shopping");
        assert!(!said.contains("moved"), "{said}");
    }

    #[test]
    fn test_a_filing_the_account_is_owed_says_it_has_not_got_there_yet() {
        // The half a move used to leave out. The row moves on this computer at
        // the moment the key is pressed and nothing has reached the account,
        // and somebody who cannot see the list had "Buy milk moved to Shopping"
        // and no way to tell those two apart. They found out at the next sync
        // or not at all.
        assert_eq!(
            filed(Filing::Moving, "Buy milk", "Shopping", Waiting::ToBeSent),
            "Buy milk moved to Shopping, and has not reached the account yet"
        );
    }

    #[test]
    fn test_a_filing_that_stays_on_this_computer_says_nothing_about_waiting() {
        // Most filings, and the reason the clause is not simply always said. A
        // note goes nowhere, and a task filed into a list made on this computer
        // goes nowhere either. "Has not reached the account yet" said about
        // those is not a caution, it is untrue: nothing is ever going to reach
        // an account, and there is nothing to wait for.
        let said = filed(Filing::Moving, "Shopping list", "Home", Waiting::StaysHere);

        assert_eq!(said, "Shopping list moved to Home");
        assert!(!said.contains("account"), "{said}");
        assert!(!said.contains("waiting"), "{said}");
    }

    #[test]
    fn test_a_filing_the_setting_is_holding_names_the_setting_to_turn_on() {
        // Allow Changes off is the one case where the person can do something
        // about it, so it is the one case that says what. In the same words the
        // task, calendar and contacts syncs use for the same fact, out of the
        // same function, because two hand-written copies of it drifted once and
        // only one was corrected.
        let said = filed(
            Filing::Moving,
            "Buy milk",
            "Shopping",
            Waiting::HeldByTheSetting,
        );

        assert_eq!(
            said,
            "Buy milk moved to Shopping, and has not reached the account: turn on \
             Allow Changes in Settings to send it"
        );
        assert!(
            said.ends_with(&crate::application::allowed::turn_the_setting_on()),
            "the move and the syncs no longer name the setting alike: {said}"
        );
    }

    #[test]
    fn test_the_setting_is_named_only_while_it_is_the_thing_holding_the_change() {
        // Naming a setting that is already on teaches nothing and is heard
        // after every single filing, which is how a useful sentence becomes
        // noise. Guardrail 5: bounded is the half that is easy to lose.
        for waiting in [Waiting::StaysHere, Waiting::ToBeSent] {
            let said = filed(Filing::Moving, "Buy milk", "Shopping", waiting);
            assert!(
                !said.contains("Allow Changes"),
                "the setting is named where it is not what is holding the change: {said}"
            );
        }
    }

    #[test]
    fn test_what_a_filing_says_is_one_sentence_however_much_it_carries() {
        // Heard after every move somebody makes. The longest this can produce
        // is the setting case, and it is joined with a comma and a colon rather
        // than being two sentences, because a paragraph read out after every
        // move is the failure guardrail 5 names.
        let longest = filed(
            Filing::Copying,
            "Ring the clinic about the results",
            "Next week",
            Waiting::HeldByTheSetting,
        );

        assert_eq!(
            longest.matches(". ").count(),
            0,
            "the filing sentence has become a paragraph: {longest}"
        );
        assert!(!longest.ends_with('.'), "{longest}");
    }

    #[test]
    fn test_a_copy_that_the_account_is_owed_still_says_copied() {
        // The clause must not cost the one distinction this sentence exists to
        // make. A copy waiting to be sent is still a copy, and heard as "moved"
        // it says the original has gone.
        let said = filed(Filing::Copying, "Buy milk", "Shopping", Waiting::ToBeSent);

        assert!(said.starts_with("Buy milk copied to Shopping"), "{said}");
        assert!(!said.contains("moved"), "{said}");
    }

    #[test]
    fn test_nothing_is_waiting_when_nothing_was_going_to_be_sent() {
        // The setting only decides anything about a change something was going
        // to send. A note is not held by Allow Changes, it has nowhere to go,
        // and sending somebody to turn on a switch that would change nothing
        // for it is worse than saying nothing at all.
        assert_eq!(what_is_waiting(false, false), Waiting::StaysHere);
        assert_eq!(what_is_waiting(false, true), Waiting::StaysHere);
    }

    #[test]
    fn test_a_change_that_will_be_sent_names_the_setting_only_when_it_is_off() {
        assert_eq!(what_is_waiting(true, true), Waiting::ToBeSent);
        assert_eq!(what_is_waiting(true, false), Waiting::HeldByTheSetting);
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
        let said = cannot_be_filed_into(
            Filing::Moving,
            ItemKind::Event,
            ContainerKind::Calendar,
            "Term dates",
        );

        assert!(said.contains("Term dates"), "{said}");
        assert!(said.contains("can only read"), "{said}");
        assert!(said.contains("Nothing has been moved"), "{said}");
        assert!(said.contains("A calendar you can change"), "{said}");
    }

    #[test]
    fn test_a_copy_into_a_container_that_can_only_be_read_is_refused_saying_copy() {
        // The same refusal on the same axis, because the destination is the
        // half being refused and a copy filed in a calendar nothing can send
        // to would sit there waiting for ever exactly as a move would.
        //
        // Worded as a copy, though. "Nothing has been moved" answering a copy
        // is an answer to a question nobody asked, and somebody working by ear
        // has no list to glance at to find out that their original is still
        // there.
        let said = cannot_be_filed_into(
            Filing::Copying,
            ItemKind::Event,
            ContainerKind::Calendar,
            "Term dates",
        );

        assert!(said.contains("Term dates"), "{said}");
        assert!(
            said.contains("copied into it could never be sent"),
            "{said}"
        );
        assert!(said.contains("Nothing has been copied"), "{said}");
        assert!(!said.contains("moved"), "{said}");
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
        assert_eq!(
            filed(Filing::Moving, "   ", "Shopping", Waiting::StaysHere),
            "Moved to Shopping"
        );
        assert_eq!(
            filed(Filing::Copying, "   ", "Shopping", Waiting::StaysHere),
            "Copied to Shopping"
        );
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
    fn test_a_copy_means_something_exactly_where_a_move_does() {
        // The five kinds that have somewhere to be copied into. A contact was
        // out of this list until 05-04 on the grounds that a second contact is
        // a second person rather than a second filing, which is true and was
        // about the wrong thing: a copy of a contact is a second group
        // membership, and it is the put-in that has always been on the contacts
        // menu. A reminder was out until 05-05 on the grounds that its buckets
        // are worked out from when it is due and are not places, which is also
        // true and was also about the wrong container: the one a reminder has
        // is the account. Mail copies between folders by its own path, which
        // has to talk to a server.
        //
        // Held to Move's own answer rather than to a list written out again,
        // because the two questions have the same answer for the same reason
        // and a list written twice is a list that comes apart.
        for kind in ItemKind::ALL {
            assert_eq!(
                PimCommand::Copy.applies_to(kind),
                PimCommand::Move.applies_to(kind),
                "copy and move disagree about {kind:?}, and nothing here is a \
                 reason for them to"
            );
        }

        assert!(PimCommand::Copy.applies_to(ItemKind::Task));
        assert!(PimCommand::Copy.applies_to(ItemKind::Contact));
        assert!(PimCommand::Copy.applies_to(ItemKind::Reminder));
        assert!(!PimCommand::Copy.applies_to(ItemKind::Mail));
    }

    #[test]
    fn test_only_the_two_filing_commands_name_an_act() {
        // The pairing lives here so that a command that puts something
        // somewhere cannot be wired to the other act at a call site. The other
        // three put nothing anywhere and say so.
        assert_eq!(PimCommand::Move.filing(), Some(Filing::Moving));
        assert_eq!(PimCommand::Copy.filing(), Some(Filing::Copying));
        assert_eq!(PimCommand::Delete.filing(), None);
        assert_eq!(PimCommand::ToggleComplete.filing(), None);
        assert_eq!(PimCommand::TogglePin.filing(), None);
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
            PimCommand::Copy,
        ]
        .into_iter()
        .map(|command| did_not_happen(command, ItemKind::Task, "Buy milk", "the file is in use"))
        .collect();
        assert_eq!(
            every.len(),
            5,
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
    fn test_confirmed_detail_is_silent_for_delete_and_for_filing() {
        // None of the three is the "did the small thing I asked for happen"
        // fact `Event::Confirmed` exists for: a delete asks first and is
        // announced by `deleted`, and a move and a copy each need the
        // destination `filed` already names.
        assert_eq!(confirmed_detail(PimCommand::Delete, true), None);
        assert_eq!(confirmed_detail(PimCommand::Move, false), None);
        assert_eq!(confirmed_detail(PimCommand::Copy, false), None);
    }

    #[test]
    fn test_a_contact_and_a_reminder_can_both_be_filed_although_neither_has_one_home() {
        // Both reasons for leaving them out were right and both are now
        // answered rather than overruled. A contact is in as many groups as
        // somebody puts it in, so there is no one home to move it out of, and
        // the program asks which one. A reminder is in no bucket anybody put it
        // in, so there was nothing to move it between, and the container it
        // does have is the account it has always belonged to.
        //
        // Mail is still out, for a reason that has not changed: it moves
        // between folders by a path that has to talk to a server.
        for command in [PimCommand::Move, PimCommand::Copy] {
            assert!(
                command.applies_to(ItemKind::Contact),
                "{command:?} does not reach a contact"
            );
            assert!(
                command.applies_to(ItemKind::Reminder),
                "{command:?} does not reach a reminder"
            );
            assert!(!command.applies_to(ItemKind::Mail), "{command:?}");
        }
    }

    /// Two accounts, in the order a chooser would read them out.
    fn two_accounts() -> Vec<AnAccount> {
        vec![
            AnAccount {
                id: "acct-work".to_string(),
                spoken: "Work".to_string(),
            },
            AnAccount {
                id: "acct-home".to_string(),
                spoken: "Home".to_string(),
            },
        ]
    }

    #[test]
    fn test_a_reminder_is_not_offered_the_account_it_is_already_in() {
        // Offering it would be offering a command that silently does nothing:
        // the storage answers AlreadyThere and writes no row, so somebody who
        // worked through the list to the account they were already in would
        // have arrowed through a question with no answer in it.
        let offered = accounts_a_reminder_could_go_to(Filing::Moving, &two_accounts(), "acct-work");

        assert_eq!(offered.len(), 1, "{offered:?}");
        assert_eq!(offered[0].id, "acct-home");
    }

    #[test]
    fn test_a_reminder_being_copied_is_offered_the_account_it_is_in() {
        // The other half of the same rule, and the one a filter written once
        // for both acts gets wrong. A second reminder in the account the first
        // is in is a real thing to want, which is why `Filing` carries
        // `leaves_out_where_it_is` rather than every caller deciding.
        let offered =
            accounts_a_reminder_could_go_to(Filing::Copying, &two_accounts(), "acct-work");

        assert_eq!(offered.len(), 2, "{offered:?}");
        assert!(
            offered.iter().any(|account| account.id == "acct-work"),
            "the copy was refused the account the reminder is in: {offered:?}"
        );
    }

    #[test]
    fn test_a_reminder_on_a_machine_with_one_account_has_nowhere_to_move_to_and_somewhere_to_copy_to()
     {
        // The case the sentence below exists for, and the case that separates
        // the two acts most plainly. There is nowhere else to move it, and
        // copying it where it is still makes a second reminder.
        let only = vec![AnAccount {
            id: "acct-work".to_string(),
            spoken: "Work".to_string(),
        }];

        assert!(accounts_a_reminder_could_go_to(Filing::Moving, &only, "acct-work").is_empty());
        assert_eq!(
            accounts_a_reminder_could_go_to(Filing::Copying, &only, "acct-work").len(),
            1
        );
    }

    #[test]
    fn test_a_reminder_with_nowhere_to_go_is_told_that_rather_than_that_it_cannot_be_filed() {
        // The nearest existing sentence says a reminder is not kept in one
        // container, which is true and is not the answer to this. Somebody who
        // hears it concludes the command does not work on reminders and never
        // tries again after setting up a second account. What is true is that
        // there is one account today.
        let said = the_only_account_there_is("Ring the dentist");

        assert!(said.contains("Ring the dentist"), "{said}");
        assert!(
            said.contains("one account"),
            "it does not say what is actually in the way: {said}"
        );
        assert!(
            said.contains("Nothing has been moved"),
            "it does not say the reminder is where it was: {said}"
        );
        assert!(
            !said.contains("not kept in one container"),
            "the refusal says a reminder can never be filed, which is a \
             different fact and stops somebody trying again: {said}"
        );
        assert_ne!(said, is_not_kept_in_a_container(ItemKind::Reminder));
    }

    #[test]
    fn test_a_reminder_with_nowhere_to_go_and_no_title_is_still_a_sentence() {
        // A row whose title never loaded, the case every sentence here copes
        // with.
        let said = the_only_account_there_is("   ");

        assert!(said.starts_with("That reminder"), "{said}");
        assert!(!said.contains("\"\""), "{said}");
    }

    #[test]
    fn test_a_move_into_the_account_it_was_already_in_does_not_report_a_move() {
        // The sentence nearest to hand is the one a successful move says, and
        // it is a report of something that did not happen. Somebody working by
        // ear has no list to glance at, so "Ring the dentist moved to Work"
        // for a reminder that was in Work before the key was pressed is the
        // only thing they have and it is false.
        let said = already_in_that_account("Ring the dentist", "Work");

        assert!(said.contains("Ring the dentist"), "{said}");
        assert!(said.contains("Work"), "{said}");
        assert!(
            said.contains("already"),
            "it does not say what was actually true: {said}"
        );
        assert!(
            said.contains("Nothing has been moved"),
            "it does not say the reminder is where it was: {said}"
        );
        // "moved to", which is the phrase a successful move says, rather than
        // "moved", which every refusal here has to be free to use in the
        // sentence saying nothing happened. The first version of this assertion
        // forbade the word outright and reddened the correct sentence.
        assert!(
            !said.contains("moved to"),
            "a move that did not happen is reported as one: {said}"
        );
        assert_ne!(
            said,
            filed(
                Filing::Moving,
                "Ring the dentist",
                "Work",
                Waiting::StaysHere
            )
        );
    }

    #[test]
    fn test_a_kind_with_no_container_is_refused_rather_than_told_its_filing_worked() {
        // The sentence for the arm nothing should reach. It must not be the
        // one about a row that has gone: the row is there, and saying it is
        // not sends somebody looking for something that never moved.
        let said = is_not_kept_in_a_container(ItemKind::Contact);

        assert!(said.contains("contact"), "{said}");
        assert!(
            !said.contains("no longer there"),
            "a refusal about having no container says the row has gone: {said}"
        );
        assert!(said.contains("Nothing has been changed."), "{said}");
        assert_ne!(said, is_not_kept_in_a_container(ItemKind::Reminder));
    }
}

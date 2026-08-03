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

use crate::application::new_item::ItemKind;

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
}

//! A search kept under a name.
//!
//! A search runs, shows what it found, and the results are gone the moment
//! anything else is opened. So a question somebody asks every morning is typed
//! out every morning. That costs more working by keyboard than working by
//! mouse: there is no saved click to go back to, no result list left open in
//! another window, and getting back to the answer means retyping the question
//! and then finding the results again.
//!
//! # Why it is written in the filter engine's words
//!
//! A saved search is a filter rule that selects instead of acting. The fields
//! and the ways of matching are the same ones, holding the same values, and
//! they are answered by [`crate::application::filters::FilterEngine`] itself
//! rather than by a matcher kept here. That is the whole reason the questions
//! look the way they do: the two are written in the same words in front of the
//! same person, and a second matcher is how "contains", "is empty" or a field
//! this build does not know would come to mean one thing in a rule and
//! something else in a search.
//!
//! The other vocabulary in this layer, [`crate::application::search`], is text
//! and a folder. That cannot say "unread mail from Ann", which is most of what
//! anybody saves. Its folder is kept here, because where to look is a real
//! part of the question.
//!
//! # Where it appears
//!
//! In the folder tree, under one heading, beside real folders. Two things
//! follow from that and both are here. A folder is found by its path, so a
//! saved search is given one under a prefix no mailbox name can produce, and
//! it is a different prefix from the one folders on this computer use, because
//! those hold mail and these hold a question. And every row says it is a
//! search when it is read out, so somebody arrowing down the tree knows what
//! Enter is about to open. That is why a search may be called after a mailbox:
//! the two are never read out the same way.
//!
//! # What would be written down
//!
//! One row for the search: an identifier, which account it belongs to, its
//! name, whether it wants every question answered or any one of them, the
//! folder to look in, and where it sits in the list. Then one row for each
//! question, carrying the same four columns a stored filter rule already
//! carries and holding the same values, so one reader serves both and neither
//! can drift. Those four are text and a flag, so there is nothing to convert;
//! the one word that is not, "all" or "any", is read by [`Join::read`] and
//! written by [`Join::written_down`], and a word neither knows is a search
//! that says it could not run rather than one that guesses.
//!
//! The row in the tree is found by the identifier rather than the name, so
//! renaming a search does not leave whatever had it open pointing at nothing.
//!
//! # What is not here
//!
//! Values in and values out. No database, no query, no window, and nothing
//! platform-specific, so it behaves the same wherever it is built. Nothing
//! calls it yet: there is no table, no row in the tree, and no search runs.
//!
//! One gap worth knowing about. A question naming a field this build does not
//! know is answered "no" by the filter engine, one message at a time, so a
//! search carrying one reads as a search that found nothing rather than as one
//! that could not run. [`Found::CouldNotRun`] is the honest answer and nothing
//! can reach it for that cause, because the engine does not say which fields
//! it knows. Telling the two apart means asking it, which is a change to the
//! engine rather than to this.

use crate::application::filters::{FilterAction, FilterEngine, FilterRule};
use crate::data::message_cache::CachedMessage;

/// One thing a saved search asks about a message.
///
/// The same four fields a filter rule carries, holding the same values, so a
/// question written here means exactly what the same words mean in a rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// Which part of the message to look at: "subject", "from", "body_plain".
    pub field: String,
    /// How to compare: "contains", "equals", "is_true".
    pub match_type: String,
    /// What to compare it against.
    pub pattern: String,
    /// Whether upper and lower case have to match.
    pub case_sensitive: bool,
}

impl Question {
    /// This question as the rule the filter engine already knows how to answer.
    ///
    /// A saved search selects rather than acts, so the action is never read.
    /// It is filled in with the one action that changes nothing, because the
    /// rule type demands one and inventing a second matcher to avoid it would
    /// be the second vocabulary this module exists to not have.
    fn as_a_rule(&self) -> FilterRule {
        FilterRule {
            id: String::new(),
            name: String::new(),
            field: self.field.clone(),
            match_type: self.match_type.clone(),
            pattern: self.pattern.clone(),
            case_sensitive: self.case_sensitive,
            action: FilterAction::MarkAsRead,
            enabled: true,
        }
    }
}

/// Whether a search wants every question answered or any one of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Join {
    All,
    Any,
}

impl Join {
    /// What the stored word means.
    ///
    /// `None` for a word this build does not know, which is a search saved by
    /// a newer one. Nothing is guessed: reading it as every question would
    /// narrow somebody's search in silence and reading it as any one of them
    /// would flood it, and both come back as an answer somebody acts on. The
    /// caller says [`Found::CouldNotRun`] instead.
    pub fn read(stored: &str) -> Option<Join> {
        match stored {
            "all" => Some(Join::All),
            "any" => Some(Join::Any),
            _ => None,
        }
    }

    /// The word this is written down as.
    pub fn written_down(self) -> &'static str {
        match self {
            Join::All => "all",
            Join::Any => "any",
        }
    }
}

/// What every saved search's row in the folder tree starts with.
///
/// A saved search sits in the tree beside real folders, and a folder is found
/// by its path, so the two must never be able to produce the same one. The
/// prefix opens with a character a mailbox name cannot carry, which is the same
/// guard [`crate::application::local_folders`] uses and for the same reason: a
/// server can name a mailbox anything, including "Saved Searches".
///
/// It is deliberately not the prefix a folder on this computer uses. A folder
/// there holds mail, and deleting from one means taking the only copy off the
/// machine. A message listed inside a search lives in a real folder somewhere
/// else, and deleting it has to mean whatever it means there.
pub const SEARCH_PREFIX: &str = "\u{1}Search";

/// Whether a path names a saved search rather than a folder.
///
/// Anything that opens a folder path has to ask this first. What is behind
/// this path is a question, not a mailbox: there is nothing to select on a
/// server and nothing to file into.
pub fn is_a_saved_search(path: &str) -> bool {
    path.starts_with(SEARCH_PREFIX)
}

/// The tree row every saved search sits under.
///
/// Named here so the sentence that says where a new one went and the row that
/// holds it cannot come to say different things.
pub const THE_HEADING: &str = "Saved Searches";

/// What a row is called when its record has no name left in it.
///
/// Naming refuses a blank name, so this is only reached by a record that went
/// wrong. It still has to say something: a row read out as ", saved search"
/// is a row somebody cannot tell from the one above it.
const NO_NAME: &str = "Saved search with no name";

/// What is said when there is no mail here to search yet.
///
/// Named rather than written where it is needed, so the row that could not run
/// says the same thing wherever it was asked from.
pub const NO_MAIL_HERE_YET: &str = "there is no mail on this computer to search yet.";

/// What is said about a search this build cannot make sense of.
///
/// Reached when [`Join::read`] does not know the stored word, which is a
/// search written by a newer version. Saying it could not run is the honest
/// answer; the alternative is guessing at the question and handing back a
/// result under somebody's own name for it.
pub const SAVED_BY_ANOTHER_VERSION: &str =
    "it was saved by a version of this program that this one does not understand.";

/// What a saved search turned up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Found {
    /// It ran and found this many. Never none, which is [`Found::Nothing`].
    Messages(usize),
    /// It ran and there was nothing.
    Nothing,
    /// It did not run, and this is why, ending as a sentence.
    ///
    /// Kept apart from finding nothing because they are not the same answer
    /// and somebody acts differently on each. "No messages" about a search
    /// that never ran is how a person stops waiting for mail that is there.
    CouldNotRun(String),
}

impl Found {
    /// What a count of results means, with none named rather than counted.
    pub fn of(how_many: usize) -> Found {
        match how_many {
            0 => Found::Nothing,
            found => Found::Messages(found),
        }
    }
}

/// What is said when a saved search is made.
///
/// It says where the row went, because a row appearing somewhere in a tree is
/// not something a person working by ear notices.
pub fn created(name: &str) -> String {
    format!("{name} saved. It is in the folder tree under {THE_HEADING}.")
}

/// The most a saved search's name may run to.
///
/// A hundred characters, counted in characters so a name written in another
/// alphabet is not refused for how it is stored. The name is read out every
/// time somebody passes the row on the way to something else, so its length is
/// a toll paid on every trip through the tree, and a hundred is already longer
/// than anybody names a search.
pub const LONGEST_NAME: usize = 100;

/// What came of naming a saved search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Naming {
    /// Good. This is the name to keep, tidied.
    Accepted(String),
    /// Nothing was typed.
    Nothing,
    /// Another saved search is already called that.
    Taken,
    /// Longer than a row anybody wants to listen to.
    TooLong,
}

impl Naming {
    /// Why the name was refused, and what to do instead.
    ///
    /// `None` when it was accepted. Every refusal names the problem and the
    /// next move, because one that only says no leaves somebody pressing the
    /// same button with the same name still in the box.
    pub fn why_not(&self) -> Option<String> {
        match self {
            Naming::Accepted(_) => None,
            Naming::Nothing => {
                Some("A saved search needs a name. Type one and try again.".to_string())
            }
            Naming::Taken => Some(
                "You already have a saved search with that name. Pick a different one.".to_string(),
            ),
            Naming::TooLong => Some(format!(
                "That name is too long. Use {LONGEST_NAME} characters or fewer."
            )),
        }
    }
}

/// Whether a name can be kept, and the name to keep if it can.
///
/// `already_used` is the names of the other saved searches, not the folders.
/// A saved search may be called after a mailbox: refusing "Inbox" would be
/// refusing a name because of a folder on a server, which another account does
/// not have and this one may not have tomorrow. What keeps those two apart is
/// that every search row says it is a search when it is read out.
///
/// Two saved searches may not share a name whatever the case, because a screen
/// reader says "Work" and "work" the same way, and two rows nobody can tell
/// apart means the one you want is whichever you did not open.
pub fn name_for(asked: &str, already_used: &[String]) -> Naming {
    let tidied = tidied(asked);
    if tidied.is_empty() {
        return Naming::Nothing;
    }
    if tidied.chars().count() > LONGEST_NAME {
        return Naming::TooLong;
    }
    let said_the_same_way = tidied.to_lowercase();
    if already_used
        .iter()
        .any(|held| tidied_lowercase(held) == said_the_same_way)
    {
        return Naming::Taken;
    }
    Naming::Accepted(tidied)
}

/// A typed name with what nobody typed taken off it.
///
/// Characters that are not really characters go first: they cannot be typed on
/// purpose, they are read out as nothing, and one of them is the character
/// that marks a row in the tree as a saved search rather than a folder. Spaces
/// at either end go too, because those are typing rather than naming.
fn tidied(asked: &str) -> String {
    asked
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .to_string()
}

/// A stored name as it would be compared against a typed one.
fn tidied_lowercase(name: &str) -> String {
    tidied(name).to_lowercase()
}

/// A search kept under a name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedSearch {
    pub id: String,
    /// What it is called, and what the tree row announces.
    pub name: String,
    /// Whether every question has to be answered or any one of them will do.
    pub join: Join,
    pub questions: Vec<Question>,
    /// Which folder to look in, or `None` for everywhere.
    ///
    /// This narrows the query that gathers the messages. It is not answered by
    /// [`SavedSearch::selects`], which sees one message at a time and a message
    /// carries the number of the folder it is in rather than its path. Handing
    /// every message in the mailbox to `selects` and expecting this to be
    /// honoured would quietly widen the search to everywhere.
    pub folder: Option<String>,
}

impl SavedSearch {
    /// The path its row in the folder tree is found by.
    ///
    /// Built from the identifier rather than the name, because a name can be
    /// edited and anything holding the old path would then point at nothing:
    /// the folder somebody had open, a column layout, whatever is restored at
    /// startup. The identifier is what the row is.
    pub fn path(&self) -> String {
        format!("{SEARCH_PREFIX}/{}", self.id)
    }

    /// What the tree row is called when it is read out.
    ///
    /// It says what it is, every time, because a row that sounds like a folder
    /// and is not is the whole risk of putting these in the tree: somebody
    /// arrowing down it has to know what Enter will open before pressing it.
    /// The words are part of the name rather than a separate description
    /// because a tree item here has one role, "tree item", for a folder and a
    /// search alike, and there is nothing else a screen reader would read. If
    /// the tree ever carries descriptions, this belongs in one.
    pub fn announced(&self) -> String {
        match self.name.trim() {
            "" => NO_NAME.to_string(),
            named => format!("{named}, saved search"),
        }
    }

    /// What the row says once the search has run.
    ///
    /// A label rather than a sentence, the same shape the Outbox uses for a
    /// waiting message, because it is read as one row among many. The count is
    /// the whole reason to look at it, and having none is said in words rather
    /// than left as silence.
    pub fn what_it_found(&self, found: &Found) -> String {
        let name = self.name_to_say();
        match found {
            Found::Messages(1) => format!("{name}, 1 message"),
            Found::Messages(how_many) => format!("{name}, {how_many} messages"),
            Found::Nothing => format!("{name}, no messages"),
            Found::CouldNotRun(why) => format!("{name} could not run: {why}"),
        }
    }

    /// The name to read out, whatever the record holds.
    fn name_to_say(&self) -> String {
        match self.name.trim() {
            "" => NO_NAME.to_string(),
            named => named.to_string(),
        }
    }

    /// Whether one message belongs in this search's results.
    ///
    /// A search with no question in it takes nothing. A list of conditions
    /// that all have to match is true of every message when the list is empty,
    /// which would turn a row somebody opened expecting a handful of messages
    /// into the whole mailbox. Nothing was asked, so nothing is the answer.
    pub fn selects(&self, message: &CachedMessage) -> bool {
        if self.questions.is_empty() {
            return false;
        }
        let mut answered = self
            .questions
            .iter()
            .map(|question| FilterEngine::matches(&question.as_a_rule(), message));
        match self.join {
            Join::All => answered.all(|yes| yes),
            Join::Any => answered.any(|yes| yes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::message_cache::CachedMessage;

    fn a_message() -> CachedMessage {
        CachedMessage {
            id: 1,
            uid: 1,
            folder_id: 1,
            message_id: "msg-1".to_string(),
            subject: "Quarterly report".to_string(),
            from_addr: "ann@example.com".to_string(),
            to_addr: "me@example.com".to_string(),
            cc: None,
            date: "2026-08-24T09:00:00Z".to_string(),
            body_plain: None,
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
        }
    }

    fn asking(field: &str, match_type: &str, pattern: &str) -> Question {
        Question {
            field: field.to_string(),
            match_type: match_type.to_string(),
            pattern: pattern.to_string(),
            case_sensitive: false,
        }
    }

    fn a_search(name: &str, questions: Vec<Question>) -> SavedSearch {
        SavedSearch {
            id: format!("id-{name}"),
            name: name.to_string(),
            join: Join::All,
            questions,
            folder: None,
        }
    }

    #[test]
    fn test_a_saved_search_picks_out_the_messages_that_answer_it() {
        // The same words a filter rule is written in, answered by the same
        // matcher, so "from contains" cannot come to mean one thing in a rule
        // and another in a search.
        let search = a_search("From Ann", vec![asking("from", "contains", "ann@")]);

        let mut from_someone_else = a_message();
        from_someone_else.from_addr = "bob@example.com".to_string();

        assert!(search.selects(&a_message()));
        assert!(!search.selects(&from_someone_else));
    }

    #[test]
    fn test_a_search_wanting_every_question_answered_needs_all_of_them() {
        // "Unread mail from Ann" is the shape almost every saved search takes,
        // and it is two questions rather than one.
        let search = a_search(
            "Unread from Ann",
            vec![
                asking("from", "contains", "ann@"),
                asking("read", "is_false", ""),
            ],
        );

        assert!(search.selects(&a_message()));

        let mut already_read = a_message();
        already_read.read = true;
        assert!(
            !search.selects(&already_read),
            "a message answering one question of two was taken"
        );
    }

    #[test]
    fn test_a_search_wanting_any_of_its_questions_takes_one_answer() {
        // "From Ann or from Bob", which is the other half of how people write
        // these and cannot be said with a list that has to match in full.
        let mut search = a_search(
            "From Ann or Bob",
            vec![
                asking("from", "contains", "ann@"),
                asking("from", "contains", "bob@"),
            ],
        );
        search.join = Join::Any;

        let mut from_bob = a_message();
        from_bob.from_addr = "bob@example.com".to_string();
        let mut from_carol = a_message();
        from_carol.from_addr = "carol@example.com".to_string();

        assert!(search.selects(&a_message()));
        assert!(search.selects(&from_bob));
        assert!(!search.selects(&from_carol));
    }

    #[test]
    fn test_a_search_with_no_question_in_it_takes_nothing_rather_than_everything() {
        // A list of conditions that all have to match is true of every message
        // when the list is empty, which would quietly turn a row somebody
        // opened expecting a handful of messages into the whole mailbox. A
        // search with no question is a broken record, and the honest answer to
        // a broken record is nothing.
        for join in [Join::All, Join::Any] {
            let mut search = a_search("Nothing asked", Vec::new());
            search.join = join;

            assert!(!search.selects(&a_message()), "{join:?} took everything");
        }
    }

    #[test]
    fn test_a_saved_search_can_never_take_a_real_folders_place() {
        // These sit in the tree beside mailboxes, and a folder is found by its
        // path. Two rows sharing one path is one row too few, and the one that
        // loses is somebody's mail. Named after a mailbox on purpose here,
        // because that is the case where it would happen.
        let search = a_search("INBOX", vec![asking("from", "contains", "ann@")]);

        let path = search.path();

        assert!(is_a_saved_search(&path), "{path:?} is not recognisable");
        for real in [
            "INBOX",
            "[Gmail]/All Mail",
            "Archive/2026",
            "Saved Searches",
        ] {
            assert_ne!(path, real, "a search took a mailbox's path");
            assert!(!is_a_saved_search(real), "{real} was read as a search");
        }
        // And not a folder on this computer either. Those hold mail, and
        // deleting from one means taking it off this machine, which is not
        // what deleting a message listed inside a search may ever mean.
        assert!(
            !crate::application::local_folders::is_local(&path),
            "a search would be treated as a folder holding mail"
        );
        assert!(!is_a_saved_search(
            crate::application::local_folders::LOCAL_PREFIX
        ));
    }

    #[test]
    fn test_two_saved_searches_do_not_share_a_row() {
        let ann = a_search("From Ann", vec![asking("from", "contains", "ann@")]);
        let bob = a_search("From Bob", vec![asking("from", "contains", "bob@")]);

        assert_ne!(ann.path(), bob.path());
    }

    #[test]
    fn test_the_tree_row_says_it_is_a_search_rather_than_a_folder() {
        // Somebody arrowing down the tree hears one row after another and has
        // to know what pressing Enter will open. A row that sounds like a
        // folder and is not is the whole risk of putting these here.
        let search = a_search("Unread from Ann", vec![asking("read", "is_false", "")]);

        assert_eq!(search.announced(), "Unread from Ann, saved search");
    }

    #[test]
    fn test_a_search_named_after_a_folder_still_does_not_sound_like_one() {
        // The name is allowed. What is not allowed is the two being
        // indistinguishable when they are read out one after the other.
        let named_like_a_mailbox = a_search("Inbox", vec![asking("read", "is_false", "")]);

        assert_eq!(named_like_a_mailbox.announced(), "Inbox, saved search");
        assert_ne!(named_like_a_mailbox.announced(), "Inbox");
    }

    #[test]
    fn test_renaming_a_saved_search_does_not_move_its_row() {
        // Whatever is holding the path, the folder open at the moment or the
        // one to restore at startup, must not be left pointing at nothing
        // because somebody tidied a name.
        let mut search = a_search("From Ann", vec![asking("from", "contains", "ann@")]);
        let before = search.path();

        search.name = "Mail from Ann".to_string();

        assert_eq!(search.path(), before);
    }

    #[test]
    fn test_a_saved_search_with_no_name_is_refused() {
        // A row in the tree with no name is a row somebody arrows onto and
        // hears nothing about.
        for typed in ["", "   ", "\t\r\n"] {
            assert_eq!(name_for(typed, &[]), Naming::Nothing, "{typed:?}");
        }
    }

    #[test]
    fn test_a_name_is_tidied_before_it_is_kept() {
        // Spaces on the end are typing, not naming. Characters that are not
        // really characters are worse: they cannot be typed on purpose, they
        // are read out as nothing, and one of them is the character that marks
        // a row as a saved search rather than a folder.
        assert_eq!(
            name_for("  Unread from Ann  ", &[]),
            Naming::Accepted("Unread from Ann".to_string())
        );
        assert_eq!(
            name_for("Unread\u{1}from Ann", &[]),
            Naming::Accepted("Unreadfrom Ann".to_string())
        );
    }

    #[test]
    fn test_a_name_another_saved_search_already_has_is_refused() {
        // Two rows reading the same is two rows nobody can tell apart, and the
        // one you want is whichever you did not open.
        let already = ["Unread from Ann".to_string(), "Invoices".to_string()];

        assert_eq!(name_for("Invoices", &already), Naming::Taken);
        assert_eq!(
            name_for("  invoices ", &already),
            Naming::Taken,
            "two names that are read out identically were both allowed"
        );
        assert!(matches!(
            name_for("Receipts", &already),
            Naming::Accepted(_)
        ));
    }

    #[test]
    fn test_a_name_too_long_to_listen_to_is_refused() {
        // The name is read out every time somebody passes the row on the way
        // to something else, so an essay in the tree is a toll on every trip
        // through it.
        let essay = "a".repeat(LONGEST_NAME + 1);
        let just_fits = "a".repeat(LONGEST_NAME);

        assert_eq!(name_for(&essay, &[]), Naming::TooLong);
        assert!(matches!(name_for(&just_fits, &[]), Naming::Accepted(_)));
    }

    #[test]
    fn test_a_long_name_is_measured_in_characters_and_not_in_bytes() {
        // A name written in another alphabet is not two or three times too
        // long because of how it is stored.
        let just_fits = "\u{4f60}".repeat(LONGEST_NAME);

        assert!(matches!(name_for(&just_fits, &[]), Naming::Accepted(_)));
    }

    #[test]
    fn test_a_name_a_real_folder_already_has_is_allowed() {
        // Refusing it would mean refusing a name because of a mailbox on a
        // server, which may not be there tomorrow and is not there at all for
        // another account. The row says it is a search, every time it is read,
        // so the two are never mistaken for each other.
        assert_eq!(
            name_for("Inbox", &[]),
            Naming::Accepted("Inbox".to_string())
        );
    }

    #[test]
    fn test_every_refusal_says_what_happened_and_what_to_do() {
        // A refusal that only says no leaves somebody pressing the same button
        // with the same name in the box.
        assert_eq!(
            Naming::Nothing.why_not().as_deref(),
            Some("A saved search needs a name. Type one and try again.")
        );
        assert_eq!(
            Naming::Taken.why_not().as_deref(),
            Some("You already have a saved search with that name. Pick a different one.")
        );
        assert_eq!(
            Naming::TooLong.why_not().as_deref(),
            Some("That name is too long. Use 100 characters or fewer.")
        );
        assert_eq!(
            Naming::Accepted("Invoices".to_string()).why_not(),
            None,
            "a name that was accepted was refused out loud"
        );
    }

    #[test]
    fn test_a_row_with_no_name_left_in_it_still_says_something() {
        // Naming refuses a blank one, so this is a row from a record that went
        // wrong. A row announced as ", saved search" is a row somebody cannot
        // tell from the one above it.
        let unnamed = a_search("   ", vec![asking("read", "is_false", "")]);

        assert_eq!(unnamed.announced(), "Saved search with no name");
    }

    #[test]
    fn test_making_a_saved_search_says_so_and_says_where_it_went() {
        // A row appearing somewhere in a tree is not something a person
        // working by ear will notice. Saying where it is is the difference
        // between finding it now and hunting for it.
        assert_eq!(
            created("Unread from Ann"),
            "Unread from Ann saved. It is in the folder tree under Saved Searches."
        );
    }

    #[test]
    fn test_a_saved_search_says_its_name_and_how_much_it_found() {
        // What the row says once it has run. The count is the whole reason to
        // look, and one message is not "1 messages".
        let search = a_search("Unread from Ann", vec![asking("read", "is_false", "")]);

        assert_eq!(
            search.what_it_found(&Found::of(12)),
            "Unread from Ann, 12 messages"
        );
        assert_eq!(
            search.what_it_found(&Found::of(1)),
            "Unread from Ann, 1 message"
        );
    }

    #[test]
    fn test_a_saved_search_that_found_nothing_says_so_plainly() {
        // Silence here reads as a search that is still running, or as one that
        // is broken. Neither is true and both send somebody looking.
        let search = a_search("Unread from Ann", vec![asking("read", "is_false", "")]);

        assert_eq!(
            search.what_it_found(&Found::of(0)),
            "Unread from Ann, no messages"
        );
        assert_eq!(Found::of(0), Found::Nothing);
    }

    #[test]
    fn test_a_saved_search_that_could_not_run_says_that_rather_than_none() {
        // The distinction this exists for. "No messages" about a search that
        // never ran is an answer to a question nobody asked, and somebody
        // acts on it: they stop expecting the mail they were waiting for.
        let search = a_search("Unread from Ann", vec![asking("read", "is_false", "")]);

        assert_eq!(
            search.what_it_found(&Found::CouldNotRun(NO_MAIL_HERE_YET.to_string())),
            "Unread from Ann could not run: there is no mail on this computer to search yet."
        );
    }

    #[test]
    fn test_a_saved_search_and_a_filter_rule_asking_the_same_thing_agree() {
        // The guard on the whole decision to reuse the filter engine's words.
        // The moment somebody writes a second matcher here, "contains" or
        // "is_empty" or an absent field starts meaning one thing in a rule and
        // another in a search, and the two are written in the same words in
        // the same settings screen.
        use crate::application::filters::FilterRule;

        let mut message = a_message();
        message.cc = None;
        message.subject = "Quarterly report".to_string();

        for (field, match_type, pattern) in [
            ("subject", "contains", "Quarterly"),
            ("subject", "not_contains", "Annual"),
            ("subject", "equals", "Quarterly report"),
            ("subject", "starts_with", "Quarterly"),
            ("subject", "ends_with", "report"),
            ("subject", "regex", "^Quarterly"),
            ("from", "contains", "ann@"),
            ("cc", "is_empty", ""),
            ("read", "is_false", ""),
            ("body_plain", "is_empty", ""),
            ("invented_field", "contains", "anything"),
            ("subject", "wishful_thinking", "Quarterly"),
        ] {
            let question = asking(field, match_type, pattern);
            let rule = FilterRule {
                id: "r1".to_string(),
                name: "the same question".to_string(),
                field: field.to_string(),
                match_type: match_type.to_string(),
                pattern: pattern.to_string(),
                case_sensitive: false,
                action: crate::application::filters::FilterAction::MarkAsRead,
                enabled: true,
            };

            assert_eq!(
                a_search("One question", vec![question]).selects(&message),
                FilterEngine::matches(&rule, &message),
                "a search and a rule disagreed about {field} {match_type} {pattern:?}"
            );
        }
    }

    #[test]
    fn test_whether_a_search_wants_all_or_any_survives_being_written_down() {
        // A search read back the other way round is a different question under
        // the same name. "From Ann and unread" turning into "from Ann or
        // unread" is a row that fills with mail nobody asked about.
        for join in [Join::All, Join::Any] {
            assert_eq!(Join::read(join.written_down()), Some(join));
        }
    }

    #[test]
    fn test_a_search_saved_by_a_build_this_one_does_not_understand_says_so() {
        // The safe answer is not a guess. Reading an unknown word as "all"
        // would narrow somebody's search silently and reading it as "any"
        // would flood it, and both come back as an answer somebody acts on.
        for stored in ["", "either", "ALL", "both of them"] {
            assert_eq!(Join::read(stored), None, "{stored:?} was guessed at");
        }

        let search = a_search("Unread from Ann", vec![asking("read", "is_false", "")]);
        assert_eq!(
            search.what_it_found(&Found::CouldNotRun(SAVED_BY_ANOTHER_VERSION.to_string())),
            "Unread from Ann could not run: it was saved by a version of this program that \
             this one does not understand."
        );
    }

    #[test]
    fn test_where_a_search_looks_is_carried_rather_than_answered_here() {
        // Worth pinning, because it is the easy thing to assume wrongly. A
        // message on its own says which folder number it is in, not which
        // folder path, so narrowing to a folder is done by the query that
        // gathers the messages. Handing every message in the mailbox to this
        // and expecting the folder to be honoured would quietly widen the
        // search to everywhere.
        let mut anywhere = a_search("From Ann", vec![asking("from", "contains", "ann@")]);
        let taken_anywhere = anywhere.selects(&a_message());

        anywhere.folder = Some("Archive/2026".to_string());

        assert_eq!(
            anywhere.selects(&a_message()),
            taken_anywhere,
            "the folder was answered here, so the query that narrows it would \
             be applying it twice or not at all"
        );
    }
}

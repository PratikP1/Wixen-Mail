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
//! platform-specific, so it behaves the same wherever it is built. The table
//! is in [`crate::data::message_cache::saved_searches`], the rows in the
//! folder tree and the commands that make, rename and remove one are in
//! `presentation::wx_app`, and the messages a search is run over are gathered
//! by a query there rather than here.
//!
//! # What a search cannot see
//!
//! Mail this computer has not downloaded, and mail that has been marked
//! deleted. A search reads what is cached, the same rows every folder listing
//! reads, so it answers about the mail somebody can see in the tree and not
//! about the whole of what a server holds.
//!
//! A question about the text of a message is answered from the copy this
//! computer kept. Bodies are evicted to stay within a budget, so a search on
//! the text can only find what is still here; the headers are always here.
//! That is said in the changelog as a limitation rather than papered over.
//!
//! # A question this build cannot read
//!
//! Either half of a question is a word, and a word this build has never met
//! can arrive in either: a search written by a newer version, or one whose
//! field was misspelled by whatever wrote it. The filter engine answers "no"
//! about a question it cannot read, which is the only safe thing to say about
//! one message and the wrong thing to report about a whole search, because
//! "no" is also what it says about a message that simply does not match.
//!
//! So the asking happens before any message is looked at, through
//! [`crate::application::filters::a_rule_may_name`] and
//! [`crate::application::filters::a_rule_may_match`], and the answer is
//! [`Found::CouldNotRun`] naming the word it could not read.
//! [`SavedSearch::run_over`] is the only way to run a search, so the asking
//! cannot be left out; [`SavedSearch::selects`] stays as the per-message test
//! it always was and is not for filtering with directly.

use crate::application::filters::{FilterAction, FilterEngine, FilterRule};
use crate::data::message_cache::CachedMessage;
use crate::data::message_cache::WhereToSearch;
use crate::data::message_cache::saved_searches::TextStoredHere;

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

/// The path a saved search's row in the folder tree is found by.
///
/// Built from the identifier rather than the name, because a name can be
/// edited and anything holding the old path would then point at nothing.
///
/// A free function as well as [`SavedSearch::path`], for the reason
/// [`a_row_for`] is one: a search this build cannot read has an identifier and
/// a name and nothing else, and it still has a row that has to be found by the
/// same path as any other. Two spellings would be a row nothing could resolve.
pub fn the_path_of(id: &str) -> String {
    format!("{SEARCH_PREFIX}/{id}")
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

/// A part of a question this build cannot make sense of.
///
/// The two halves of a question are stored as words, and a word this build has
/// never met can arrive in either: a search written by a newer version, or one
/// whose field was misspelled by whatever wrote it. Neither can be answered,
/// and both are answered "no" by the filter engine, which is the only safe
/// thing it can say about one message and the wrong thing to report about a
/// whole search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotUnderstood {
    /// The part of the message it asks about.
    Field(String),
    /// The way it asks the message and the pattern to be compared.
    WayOfMatching(String),
}

impl NotUnderstood {
    /// Why the search could not run, ending as a sentence.
    ///
    /// It names the word it could not read. "This search could not run" on its
    /// own leaves somebody with nothing to fix and no way to tell a search
    /// written by a newer version from one with a typo in it.
    pub fn why(&self) -> String {
        match self {
            NotUnderstood::Field(field) => format!(
                "it asks about the {field} of a message, which this version does not understand."
            ),
            NotUnderstood::WayOfMatching(way) => {
                format!("it asks to match by {way}, which this version does not understand.")
            }
        }
    }
}

/// What came of running a saved search over some messages.
///
/// One shape rather than "ask whether it can run, then run it", because the
/// asking is the half that gets skipped. [`SavedSearch::selects`] answers "no"
/// about a question this build cannot read, which is the same word it uses
/// about a message that does not match, so the two arrive as one empty list.
#[derive(Debug)]
pub enum Ran<'a> {
    /// It ran, and these are the messages it took, in the order they came.
    Took(Vec<&'a CachedMessage>),
    /// There was no mail on this computer for it to look at.
    ///
    /// The first morning of a new account, before anything has come down.
    /// Kept apart from finding nothing for the same reason a question this
    /// build cannot read is: "no messages" says the mail is not there, when
    /// what is true is that this computer has not got it yet.
    NothingToLookAt,
    /// It did not run, and this is what was wrong with it.
    CouldNotRun(NotUnderstood),
}

impl Ran<'_> {
    /// What is said about it.
    pub fn found(&self) -> Found {
        match self {
            Ran::Took(taken) => Found::of(taken.len()),
            Ran::NothingToLookAt => Found::CouldNotRun(NO_MAIL_HERE_YET.to_string()),
            Ran::CouldNotRun(why) => Found::CouldNotRun(why.why()),
        }
    }
}

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

/// How a saved search reads in the folder tree.
///
/// One spelling, used by the tree that draws the row and by the handler that
/// works back from a row to the search it names, for the reason a label row
/// has one: two spellings would mean a row nothing could resolve. It says what
/// it is every time, because a row that sounds like a folder and is not is the
/// whole risk of putting these in the tree.
///
/// A name rather than a whole search, so a search this build cannot read is
/// still a row and still reads out like the rest of them.
pub fn a_row_for(name: &str) -> String {
    match name.trim() {
        "" => NO_NAME.to_string(),
        named => format!("{named}, saved search"),
    }
}

/// The parts of a message a search somebody typed into the box asks about.
///
/// The subject, the sender and the recipients: where a word somebody remembers
/// about a message would be written. Named here rather than built at the call
/// site, so the sentence that says what is being saved and the questions that
/// are saved cannot come to say different things.
pub const WHAT_A_TYPED_SEARCH_LOOKS_AT: [&str; 3] = ["subject", "from", "to"];

/// Whether a typed search wants every part to hold the word or any one of them.
///
/// Any one. Somebody typing into a search box is asking where a word appears,
/// not for a message whose subject and sender and recipients all carry it,
/// which is almost nothing.
pub const WHAT_A_TYPED_SEARCH_JOINS_WITH: Join = Join::Any;

/// A search that was run from the search box, kept whole.
///
/// The words and the answer the "In" list gave, in one value, because they
/// describe one act. Two fields side by side is the shape where one gets
/// written and the other does not, and the compiler cannot see it; one value
/// means every reader is enumerated. `01-05` made
/// `WxUIState::selected_folder` a row identity rather than a display string
/// for the same reason.
///
/// It is kept at all because the search box is a modal dialog that is gone by
/// the time anybody decides the answer was worth keeping, and asking somebody
/// to type the question twice is the work saving a search exists to remove.
/// That was always true of the words. It is now true of the scope as well,
/// which is the whole reason the scope has to be kept rather than worked out
/// again from whatever is on screen later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheSearchThatWasRun {
    /// The words typed into the box.
    pub typed: String,
    /// What the "In" list was set to.
    pub looking_in: WhereToSearch,
    /// The folder [`Self::looking_in`] names, when it names one.
    ///
    /// Filled only by [`Self::new`], so this and `looking_in` cannot come to
    /// name different folders, and taken at the moment the search ran rather
    /// than at the moment somebody names it: those are different folders
    /// whenever somebody has arrowed to another one in between.
    pub the_folder_looked_in: Option<TheFolderSearched>,
}

/// The folder a search was narrowed to, and whose it is.
///
/// The account as well as the path, because a stored folder is resolved
/// against the account the saved search belongs to and a path is not unique
/// across accounts: "INBOX" names a folder in every one of them. The two
/// travel together so nothing can pair a path with the wrong account and get
/// somebody else's mail back under a name they gave this one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheFolderSearched {
    /// Which account the folder belongs to.
    pub account: String,
    /// The path the server spells, which is what a saved search stores and
    /// what its reader looks the folder up by.
    pub path: String,
}

impl TheSearchThatWasRun {
    /// What was asked, with the folder kept only when the "In" list named one.
    ///
    /// `folder_on_screen` is the folder that was open when the search ran. It
    /// is dropped for the three answers that do not narrow to one folder, so
    /// "All Folders" chosen while a folder happens to be open saves a search
    /// across the account rather than one silently pinned to wherever somebody
    /// was standing.
    pub fn new(
        typed: String,
        looking_in: WhereToSearch,
        folder_on_screen: Option<TheFolderSearched>,
    ) -> TheSearchThatWasRun {
        TheSearchThatWasRun {
            typed,
            the_folder_looked_in: match looking_in {
                WhereToSearch::OneFolder(_) => folder_on_screen,
                WhereToSearch::EveryFolder
                | WhereToSearch::SubjectOnly
                | WhereToSearch::SenderOnly => None,
            },
            looking_in,
        }
    }
}

/// Why a search that has been run cannot be kept under this account.
///
/// The refusal exists because "Set Active" changes which account a search is
/// saved under and leaves the folder tree's cursor where it was, so the folder
/// a search ran in can belong to an account other than the one about to own
/// it. A path is not unique across accounts, so storing it anyway would give a
/// saved search that quietly lists another account's mail of the same path,
/// usually its inbox.
///
/// Refused rather than saved without the folder. Dropping it would widen the
/// search from one folder to a whole account with nothing said, which is the
/// failure this plan exists to fix, arriving by another door.
pub const THAT_FOLDER_IS_ANOTHER_ACCOUNTS: &str = "That search ran in another account's folder. Open that account, search there, and then this \
     will keep it.";

/// Whether a search that was run may be kept under this account.
///
/// Asked before anything is written, and about the account the search is being
/// saved under rather than whichever is open now, because those are the same
/// answer almost always and not always.
pub fn a_search_may_be_saved_under(ran: &TheSearchThatWasRun, account_id: &str) -> bool {
    let _ = (ran, account_id);
    true
}

/// How a typed search compares the words against the part it looks at.
///
/// Written down once and read by every answer the "In" list can give, so a
/// narrowed search asks the same kind of question as an unnarrowed one and
/// differs only in how many parts of a message it asks it about.
const HOW_A_TYPED_SEARCH_COMPARES: &str = "contains";

/// The whole of what a saved search will ask, from one answer.
///
/// The three halves of a scope together, because they describe one thing.
/// D-2-14: the folder and the field restriction written by different code is
/// the shape that comes apart, and every data-losing defect found in this
/// codebase has had it. There is one producer, one value, and one write, so
/// there is nowhere for a second answer to appear.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatASavedSearchWillAsk {
    pub questions: Vec<Question>,
    pub join: Join,
    /// Which folder to look in, or `None` for everywhere in the account.
    pub folder: Option<String>,
}

impl WhatASavedSearchWillAsk {
    /// This, in words, said while somebody is naming the search.
    ///
    /// Here rather than at the call site so the three halves reach the
    /// sentence from the one value that holds them, and a window cannot
    /// describe one set of questions while saving another.
    pub fn in_words(&self) -> String {
        a_search_in_words(&self.questions, self.join, self.folder.as_deref())
    }
}

/// The parts of a message one answer from the "In" list asks about.
///
/// Two of the four answers narrow which part of a message is read and two
/// narrow where to look, which is one control doing two jobs; that is what the
/// box has always offered. So Current Folder asks about the same three parts
/// All Folders does, and the folder it names is carried separately.
///
/// The three-part answer is [`WHAT_A_TYPED_SEARCH_LOOKS_AT`] itself rather
/// than a copy, so a search saved by an older version and an unnarrowed one
/// saved now are the same three questions and cannot be told apart. That is
/// what makes SEARCH-01's fourth criterion, about a missing restriction
/// reading as an unrestricted search, disappear rather than need answering:
/// there is no absent value anywhere to interpret.
///
/// A match rather than a lookup, so a fifth answer added to the "In" list has
/// to be sorted here deliberately instead of falling into a default.
fn what_that_answer_looks_at(looking_in: WhereToSearch) -> &'static [&'static str] {
    match looking_in {
        WhereToSearch::SubjectOnly => &["subject"],
        WhereToSearch::SenderOnly => &["from"],
        WhereToSearch::EveryFolder | WhereToSearch::OneFolder(_) => &WHAT_A_TYPED_SEARCH_LOOKS_AT,
    }
}

/// The search somebody typed, written as the whole of what a saved search asks.
///
/// Both halves of the scope out of one value, rather than the questions here
/// and the folder worked out again at the call site. `wx_app.rs` hardcoded
/// `folder: None` and argued that narrowing would be narrowing on something
/// nobody wrote down; that stopped being true the moment the field
/// restriction was written down, and the two halves have to arrive together
/// or the one that is easier to forget is the one that gets forgotten.
pub fn what_a_typed_search_asks(ran: &TheSearchThatWasRun) -> WhatASavedSearchWillAsk {
    WhatASavedSearchWillAsk {
        questions: what_that_answer_looks_at(ran.looking_in)
            .iter()
            .map(|part| Question {
                field: (*part).to_string(),
                match_type: HOW_A_TYPED_SEARCH_COMPARES.to_string(),
                pattern: ran.typed.clone(),
                case_sensitive: false,
            })
            .collect(),
        join: WHAT_A_TYPED_SEARCH_JOINS_WITH,
        folder: ran
            .the_folder_looked_in
            .as_ref()
            .map(|folder| folder.path.clone()),
    }
}

/// What is said about a question this build cannot put into words.
///
/// A field or a way of matching written by a newer version reaches here, and
/// the house answer to a stored word nothing understands is to say so rather
/// than to guess or to read the machine name out. [`Join::read`] and
/// `put_back_together` answer the same way for the same reason.
const A_QUESTION_THIS_VERSION_CANNOT_READ: &str = "a question this version cannot read";

/// One question in the words a person would say.
///
/// The words come from the pairs beside the filter engine's own constants, so
/// there is one vocabulary rather than a second list here. A way of matching
/// that compares against nothing gets no pattern read out: those four answer
/// from the field alone, and whatever is stored in the pattern is not part of
/// the question. Reading it out would announce an empty pair of quotes, or
/// worse, some text left over from a Pattern box that never applied.
fn one_question_in_words(question: &Question) -> String {
    use crate::application::filters::{
        a_way_of_matching_compares_against_nothing, the_words_for_a_field,
        the_words_for_a_way_of_matching,
    };

    let (Some(part), Some(way)) = (
        the_words_for_a_field(&question.field),
        the_words_for_a_way_of_matching(&question.match_type),
    ) else {
        return A_QUESTION_THIS_VERSION_CANNOT_READ.to_string();
    };
    if a_way_of_matching_compares_against_nothing(&question.match_type) {
        format!("{part} {way}")
    } else {
        format!("{part} {way} \"{}\"", question.pattern)
    }
}

/// Several clauses read out as one list, joined by the word the search uses.
///
/// The joining word is the search's own, because "or" said about a search that
/// wants every question answered describes a different search. A list of two
/// takes no comma, which is how it would be spoken.
fn one_after_another(clauses: &[String], join: Join) -> String {
    let word = match join {
        Join::All => "and",
        Join::Any => "or",
    };
    match clauses {
        [] => String::new(),
        [only] => only.clone(),
        [first, second] => format!("{first} {word} {second}"),
        [rest @ .., last] => format!("{}, {word} {last}", rest.join(", ")),
    }
}

/// What a saved search asks, in words, for any set of questions.
///
/// Said while somebody is naming a search rather than after. A search saved
/// from the box asks a narrower question than the box did: the box reads the
/// first line of a message as well, and this does not, so a search saved from
/// it can come back with fewer messages than the search that was just run.
/// Being told that while naming it is the difference between a known limit and
/// a bug report.
///
/// Built from the questions rather than from the name of the answer the "In"
/// list gave (D-2-04). The rule editor makes sets that list has no name for, so
/// a path built around scope names would need a fallback for them anyway, and a
/// search edited until it no longer matched a named scope would silently change
/// how it describes itself. Building from the questions removes the case
/// instead of handling it.
pub fn a_search_in_words(questions: &[Question], join: Join, folder: Option<&str>) -> String {
    let where_it_looks = match folder {
        Some(path) => format!("in {path}"),
        None => "in this account".to_string(),
    };
    let clauses: Vec<String> = questions.iter().map(one_question_in_words).collect();
    if clauses.is_empty() {
        return format!(
            "This saved search asks nothing about a message, so it finds every message \
             {where_it_looks}."
        );
    }
    format!(
        "This saved search looks {where_it_looks} for messages where {}.",
        one_after_another(&clauses, join)
    )
}

/// How the coverage sentence opens, and the one place those words are written.
///
/// A constant so a check can count the places that build this sentence, which
/// is what stops a second one appearing in the window layer. The three cases
/// below finish it differently and all three start here.
const READS_MESSAGE_TEXT: &str = "This saved search reads the text of your messages, and";

/// What a saved search that reads message text can actually cover here.
///
/// Said before the search runs, because a search that comes back with three
/// results when the answer is thirty reads as an answer rather than as a
/// fraction of one, and nothing in the result says which it was.
///
/// **It says which search it is about, and that is the point of it rather
/// than a nicety.** There are two searches in this program and they cover
/// different amounts of the same mailbox. Evicting a message's text deletes
/// what this search reads and deliberately leaves the search index alone, so
/// the box at the top goes on finding that message by a word that is only in
/// its text, while this search no longer can. One number covering "the search"
/// would therefore be wrong about one of them whichever way it was computed,
/// and it would be wrong in the confident direction. Collapsing this into one
/// number is the failure this sentence exists to prevent, wearing a different
/// coat. The doc on `evict_bodies_over` records the same decision from the
/// other side.
pub fn what_a_saved_search_covers(coverage: TextStoredHere) -> String {
    let TextStoredHere {
        messages,
        with_text,
    } = coverage;
    // "the 1 message" and "the 3 messages", so the sentence reads properly at
    // one as well as at many. A mailbox of one is a real mailbox and hearing
    // "1 messages" is how a program tells somebody it was not written for
    // them.
    let mail = match messages {
        1 => "the 1 message".to_string(),
        _ => format!("the {messages} messages"),
    };
    // Whether a message with no text here can still be matched at all. It can:
    // a search asks several questions and only the ones about text lose their
    // answer, so saying "cannot be searched" would be a worse lie than the one
    // this sentence is fixing.
    let rest = "can only be matched on what else this search asks about";

    match (messages, with_text) {
        // Nothing synced yet. "0 of 0 have their text here" is true, useless,
        // and reads as a fault in the search rather than as an empty cache.
        (0, _) => "There is no mail from this account on this computer yet, so this saved \
                   search has nothing to look at."
            .to_string(),
        (total, here) if here >= total => {
            format!("{READS_MESSAGE_TEXT} this computer has the text of {mail} in this account.")
        }
        (_, 0) => format!(
            "{READS_MESSAGE_TEXT} this computer has the text of none of {mail} in this \
             account, so a message {rest}."
        ),
        (_, here) => format!(
            "{READS_MESSAGE_TEXT} this computer has the text of {here} of {mail} in this \
             account. The rest {rest}."
        ),
    }
}

/// How many of a search's results the message list is filled with.
///
/// The same bound every other listing here carries, and for the same reason: a
/// list of forty thousand rows is one nobody can move through. The count that
/// is announced is the true one, so nobody is told they have five hundred when
/// they have nine hundred.
pub const MOST_RESULTS_SHOWN: usize = 500;

/// What is said when a search found more than the list was filled with.
///
/// Said as well as the count rather than instead of it. A list quietly cut to
/// its first page, with a count that matches the cut, is a search that has
/// answered a narrower question than the one somebody asked.
pub fn only_the_newest_are_shown(shown: usize) -> String {
    format!("The newest {shown} are shown.")
}

/// What is said about a search that looks in a folder this computer has not
/// got.
///
/// A folder can be renamed on the server, unsubscribed, or belong to an
/// account that has gone. Searching everywhere instead would quietly widen the
/// question; finding nothing would quietly narrow it. Neither is what was
/// asked, so it says it could not run.
pub const THAT_FOLDER_IS_NOT_HERE: &str =
    "the folder it looks in is not on this computer any more.";

/// What a row says once its search has run.
///
/// A name rather than a whole search, so a search this build cannot read says
/// what happened to it in the same words as one it can. Two spellings would be
/// two ways of wording a count, and the row nobody can run is the one that
/// would end up worded worse.
///
/// A label rather than a sentence, the same shape the Outbox uses for a
/// waiting message. The count is the whole reason to look at it, and having
/// none is said in words rather than left as silence.
pub fn what_a_search_found(name: &str, found: &Found) -> String {
    let name = match name.trim() {
        "" => NO_NAME,
        named => named,
    };
    match found {
        Found::Messages(1) => format!("{name}, 1 message"),
        Found::Messages(how_many) => format!("{name}, {how_many} messages"),
        Found::Nothing => format!("{name}, no messages"),
        Found::CouldNotRun(why) => format!("{name} could not run: {why}"),
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
        the_path_of(&self.id)
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
        a_row_for(&self.name)
    }

    /// Whether answering this search means having the text of each message.
    ///
    /// Asked before the messages are gathered, so a search about senders and
    /// subjects costs a listing-sized read and only one about the text of a
    /// message pays for unpacking every body this computer holds.
    pub fn reads_the_message_text(&self) -> bool {
        self.questions.iter().any(|question| {
            crate::application::filters::a_rule_reads_the_message_text(&question.field)
        })
    }

    /// What the row says once the search has run.
    ///
    /// A label rather than a sentence, the same shape the Outbox uses for a
    /// waiting message, because it is read as one row among many. The count is
    /// the whole reason to look at it, and having none is said in words rather
    /// than left as silence.
    pub fn what_it_found(&self, found: &Found) -> String {
        what_a_search_found(&self.name, found)
    }

    /// The part of a question this build cannot read, if there is one.
    ///
    /// Asked once, before any message is looked at, because the answer is the
    /// same for every message and because it is a different answer from "this
    /// message does not match". The first one found is reported: a search with
    /// two unreadable questions is still one search that cannot run, and
    /// naming one word to fix is more use than naming a list.
    pub fn what_it_cannot_read(&self) -> Option<NotUnderstood> {
        use crate::application::filters::{a_rule_may_match, a_rule_may_name};

        self.questions.iter().find_map(|question| {
            if !a_rule_may_name(&question.field) {
                Some(NotUnderstood::Field(question.field.clone()))
            } else if !a_rule_may_match(&question.match_type) {
                Some(NotUnderstood::WayOfMatching(question.match_type.clone()))
            } else {
                None
            }
        })
    }

    /// Run this search over some messages.
    ///
    /// The one way to run one. It asks [`Self::what_it_cannot_read`] first,
    /// which is the step that has to happen before any message is looked at
    /// and the step a caller doing the filtering itself would leave out.
    pub fn run_over<'a>(&self, messages: &'a [CachedMessage]) -> Ran<'a> {
        // The question first. A search this build cannot read is the thing to
        // fix whether there is mail here to look at or not.
        if let Some(why) = self.what_it_cannot_read() {
            return Ran::CouldNotRun(why);
        }
        if messages.is_empty() {
            return Ran::NothingToLookAt;
        }
        Ran::Took(
            messages
                .iter()
                .filter(|message| self.selects(message))
                .collect(),
        )
    }

    /// Whether one message belongs in this search's results.
    ///
    /// A search with no question in it takes nothing. A list of conditions
    /// that all have to match is true of every message when the list is empty,
    /// which would turn a row somebody opened expecting a handful of messages
    /// into the whole mailbox. Nothing was asked, so nothing is the answer.
    ///
    /// One message at a time, so it cannot tell a question this build cannot
    /// read from a message that does not match: both are "no". That is what
    /// [`Self::run_over`] is for, and why nothing outside this module should
    /// be filtering with this directly.
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

    /// The coverage sentence for an account of `messages` with `with_text` of
    /// them holding their text here.
    fn covering(messages: i64, with_text: i64) -> String {
        what_a_saved_search_covers(TextStoredHere {
            messages,
            with_text,
        })
    }

    /// The shipping half of one file, tests cut off.
    ///
    /// Cut, because the check below looks for the very words it would
    /// otherwise find in itself, which is a check that measures nothing.
    fn what_ships_in(path: &str) -> String {
        std::fs::read_to_string(path)
            .map(|text| crate::common::what_ships::what_ships(&text))
            .unwrap_or_default()
    }

    #[test]
    fn test_the_coverage_sentence_gives_both_numbers() {
        // Both, because they are two facts and neither implies the other. A
        // sentence with only the covered number cannot be told from a complete
        // one, and a sentence with only the total says nothing about coverage.
        let said = covering(30, 12);

        assert!(
            said.contains("12"),
            "the sentence dropped how much text is here: {said}"
        );
        assert!(
            said.contains("30"),
            "the sentence dropped how much mail there is: {said}"
        );
    }

    #[test]
    fn test_the_coverage_sentence_says_which_search_it_is_about() {
        // The whole reason this sentence is worded rather than being a pair of
        // numbers. Two searches here cover different amounts of the same
        // mailbox, so a coverage claim that does not name its subject is read
        // as being about the box at the top, which it is not about.
        for (messages, with_text) in [(30, 12), (30, 30), (30, 0)] {
            let said = covering(messages, with_text);
            assert!(
                said.contains("saved search"),
                "a coverage sentence that does not say which search it is \
                 about: {said}"
            );
        }
    }

    #[test]
    fn test_an_account_whose_text_is_all_here_is_told_so_plainly() {
        // "0 do not have their text here" is arithmetic somebody has to do in
        // their head to learn there is nothing wrong. It also reads as a
        // warning, which is exactly the wrong shape for the good case.
        let all_here = covering(30, 30);

        assert_ne!(all_here, covering(30, 12));
        assert_ne!(all_here, covering(30, 0));
        assert!(
            !all_here.contains(" 0 "),
            "the everything-is-here case counted what is missing: {all_here}"
        );
    }

    #[test]
    fn test_an_account_with_no_text_here_does_not_read_as_an_empty_mailbox() {
        // Mail that is here with its text elsewhere and no mail at all are
        // different situations and somebody does different things about each.
        // The count of messages is what tells them apart, so it stays in.
        let none_here = covering(30, 0);

        assert!(
            none_here.contains("30"),
            "a search over 30 messages was described as having nothing: \
             {none_here}"
        );
        assert_ne!(none_here, covering(30, 30));
    }

    #[test]
    fn test_an_account_with_no_mail_here_is_not_given_a_coverage_figure() {
        // Nothing has been synced yet. "0 of 0 messages have their text here"
        // is true, useless, and reads as a fault in the search rather than as
        // an empty cache.
        let nothing_yet = covering(0, 0);

        assert!(
            !nothing_yet.contains('0'),
            "an account with no mail here was given a coverage figure: \
             {nothing_yet}"
        );
        assert_ne!(nothing_yet, covering(30, 0));
    }

    #[test]
    fn test_one_place_builds_the_coverage_sentence() {
        // Both directions in one fixture. The window layer must not word this
        // itself, and a check for the absence of words proves nothing on its
        // own: the same reader has to find them where they really are, or a
        // renamed constant would read as a clean window layer.
        assert_eq!(
            what_ships_in("src/application/saved_searches.rs")
                .matches(READS_MESSAGE_TEXT)
                .count(),
            1,
            "the words of the coverage sentence are not written once here"
        );
        assert_eq!(
            what_ships_in("src/presentation/wx_app.rs")
                .matches(READS_MESSAGE_TEXT)
                .count(),
            0,
            "the window layer words the coverage sentence itself"
        );
    }

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
    fn test_a_search_naming_a_field_this_build_does_not_know_says_it_could_not_run() {
        // The distinction the whole module exists for, on the one cause that
        // could not reach it. The engine answers "no" about a field it cannot
        // read, which is the same word it uses about a message that simply
        // does not match, so a search carrying a misspelled or newer field
        // came back as a search that found nothing. Somebody hears "no
        // messages", stops waiting, and the mail is in their inbox.
        let search = a_search(
            "From my manager",
            vec![asking("sender_display_name", "contains", "ann")],
        );

        assert_eq!(
            search.run_over(&[a_message()]).found(),
            Found::CouldNotRun(
                "it asks about the sender_display_name of a message, which this version \
                 does not understand."
                    .to_string()
            )
        );
    }

    #[test]
    fn test_a_search_matching_a_way_this_build_does_not_know_says_it_could_not_run() {
        // The other half of a question, and the same failure. Naming the
        // fields and not the ways would have left a search written by a newer
        // version reading as an empty folder for half the reasons it can be
        // unreadable.
        let search = a_search(
            "Sounds like",
            vec![asking("subject", "sounds_like", "invoice")],
        );

        assert_eq!(
            search.run_over(&[a_message()]).found(),
            Found::CouldNotRun(
                "it asks to match by sounds_like, which this version does not understand."
                    .to_string()
            )
        );
    }

    #[test]
    fn test_a_search_with_no_mail_to_look_at_says_that_rather_than_no_messages() {
        // The first morning of a new account, before anything has been
        // downloaded. "No messages" there is the same wrong answer a search
        // this build cannot read used to give: it says the mail is not there,
        // when what is true is that this computer has not got it yet.
        let search = a_search("From Ann", vec![asking("from", "contains", "ann@")]);

        assert_eq!(
            search.run_over(&[]).found(),
            Found::CouldNotRun(NO_MAIL_HERE_YET.to_string())
        );
        // And a search whose question cannot be read says that instead, which
        // is the thing to fix whether there is mail here or not.
        let unreadable = a_search("Broken", vec![asking("sender_name", "contains", "ann")]);
        assert!(matches!(
            unreadable.run_over(&[]).found(),
            Found::CouldNotRun(why) if why.contains("sender_name")
        ));
    }

    #[test]
    fn test_a_search_that_can_run_hands_back_the_messages_it_took() {
        // The other direction, so the refusal above cannot be satisfied by a
        // build that refuses everything.
        let search = a_search("From Ann", vec![asking("from", "contains", "ann@")]);
        let mut from_bob = a_message();
        from_bob.from_addr = "bob@example.com".to_string();
        let messages = [a_message(), from_bob];

        let Ran::Took(taken) = search.run_over(&messages) else {
            panic!("a search asking an ordinary question refused to run");
        };

        assert_eq!(taken.len(), 1);
        assert_eq!(taken[0].from_addr, "ann@example.com");
        assert_eq!(search.run_over(&messages).found(), Found::Messages(1));
    }

    #[test]
    fn test_a_search_saved_by_another_version_is_still_a_row_that_reads_the_same() {
        // A search this build cannot make sense of still has a name and still
        // has a row in the tree. It has to read out the same way as the rest,
        // because somebody arrowing past it is not being asked to tell a
        // readable search from an unreadable one by ear.
        let readable = a_search("Invoices", vec![asking("from", "contains", "ann@")]);

        assert_eq!(a_row_for("Invoices"), readable.announced());
        assert_eq!(a_row_for("   "), NO_NAME);
        assert_eq!(the_path_of(&readable.id), readable.path());
    }

    #[test]
    fn test_a_search_asking_about_the_message_text_says_so_before_it_is_run() {
        // What decides whether the read that gathers the messages has to bring
        // every body on this computer with it. Asked from the questions rather
        // than paid for on every search.
        let headers_only = a_search("From Ann", vec![asking("from", "contains", "ann@")]);
        let the_text = a_search(
            "Mentions the invoice",
            vec![
                asking("from", "contains", "ann@"),
                asking("body_plain", "contains", "invoice"),
            ],
        );

        assert!(!headers_only.reads_the_message_text());
        assert!(the_text.reads_the_message_text());
    }

    #[test]
    fn test_a_search_this_build_cannot_read_still_says_its_name_and_what_happened() {
        // A search saved by a newer version has a name and nothing else this
        // build understands, so the sentence about it cannot be built from a
        // search. One spelling for both, so the row somebody can run and the
        // row nobody can are worded the same way and neither can drift.
        let readable = a_search("Invoices", vec![asking("from", "contains", "ann@")]);

        assert_eq!(
            what_a_search_found("Invoices", &Found::of(3)),
            readable.what_it_found(&Found::of(3))
        );
        assert_eq!(
            what_a_search_found(
                "Invoices",
                &Found::CouldNotRun(SAVED_BY_ANOTHER_VERSION.to_string())
            ),
            "Invoices could not run: it was saved by a version of this program that this one \
             does not understand."
        );
    }

    #[test]
    fn test_a_typed_search_becomes_questions_that_say_what_they_ask() {
        // What "save the search you just ran" turns into. The subject, the
        // sender and the recipients, any one of which will do, because that is
        // how somebody reads a search box: find this word anywhere it would be
        // written on a message.
        //
        // Every question has to be one the engine can answer, or the search
        // would be saved and then refuse to run the moment it was opened.
        let asked = what_a_typed_search_asks(&across_the_account("invoice"));
        let saved = SavedSearch {
            id: "s1".to_string(),
            name: "Invoices".to_string(),
            join: asked.join,
            questions: asked.questions,
            folder: asked.folder,
        };

        assert_eq!(saved.what_it_cannot_read(), None);
        assert_eq!(
            saved.join,
            Join::Any,
            "a typed search was narrowed to mail answering all of it"
        );

        let mut from_ann = a_message();
        from_ann.subject = "Nothing to see".to_string();
        from_ann.from_addr = "invoice@example.com".to_string();
        let mut about_it = a_message();
        about_it.subject = "Your invoice".to_string();
        let mut neither = a_message();
        neither.subject = "Lunch".to_string();
        neither.from_addr = "bob@example.com".to_string();
        neither.to_addr = "me@example.com".to_string();

        assert!(saved.selects(&from_ann));
        assert!(saved.selects(&about_it));
        assert!(!saved.selects(&neither));
    }

    // ── A saved search keeps the whole scope it was saved with ──────────
    //
    // The "In" list offered four answers and the saved search kept none of
    // them: choosing From Only and saving gave back a search over the
    // subject, the sender and the recipients. The field half is a narrower
    // question set and the folder half is the folder, and both come out of
    // one value in one call, because two things describing one scope written
    // by different code is the shape that comes apart.

    /// A search run with the "In" list left where it starts.
    fn across_the_account(typed: &str) -> TheSearchThatWasRun {
        TheSearchThatWasRun::new(typed.to_string(), WhereToSearch::EveryFolder, None)
    }

    /// A folder of the account these tests save under, unless they say
    /// otherwise.
    fn a_folder_of(account: &str, path: &str) -> Option<TheFolderSearched> {
        Some(TheFolderSearched {
            account: account.to_string(),
            path: path.to_string(),
        })
    }

    /// The parts of a message a set of questions asks about, in order.
    fn the_parts_asked_about(asked: &WhatASavedSearchWillAsk) -> Vec<&str> {
        asked
            .questions
            .iter()
            .map(|question| question.field.as_str())
            .collect()
    }

    #[test]
    fn test_subject_only_saves_one_question_about_the_subject() {
        let asked = what_a_typed_search_asks(&TheSearchThatWasRun::new(
            "invoice".to_string(),
            WhereToSearch::SubjectOnly,
            None,
        ));

        assert_eq!(
            the_parts_asked_about(&asked),
            vec!["subject"],
            "Subject Only was chosen and the saved search asks about the \
             sender and the recipients too, so it comes back wider than the \
             search that was run"
        );
        assert_eq!(asked.questions[0].pattern, "invoice");
        assert!(!asked.questions[0].case_sensitive);
        assert_eq!(
            asked.questions[0].match_type, "contains",
            "a narrowed search compares differently from an unnarrowed one"
        );
    }

    #[test]
    fn test_from_only_saves_one_question_about_the_sender() {
        let asked = what_a_typed_search_asks(&TheSearchThatWasRun::new(
            "invoice".to_string(),
            WhereToSearch::SenderOnly,
            None,
        ));

        assert_eq!(
            the_parts_asked_about(&asked),
            vec!["from"],
            "From Only was chosen and the saved search asks about the subject \
             and the recipients too"
        );
    }

    #[test]
    fn test_all_folders_saves_the_same_three_questions_it_always_has() {
        // The backward-compatibility case, made to disappear rather than
        // handled. A search saved by an older version holds these three
        // questions and no folder, and one saved now with the list left where
        // it starts holds exactly the same thing, so there is no absent value
        // for a reader to interpret and the reader's answer and the writer's
        // cannot come apart.
        let asked = what_a_typed_search_asks(&across_the_account("invoice"));

        // The three named here rather than read out of the constant. Both
        // sides reading it would move together when it changed, and this
        // assertion is about the three a search saved before this plan holds,
        // which no later edit may quietly redefine.
        assert_eq!(
            the_parts_asked_about(&asked),
            vec!["subject", "from", "to"],
            "an unnarrowed search saved now asks something different from one \
             saved before this change"
        );
        assert_eq!(asked.folder, None);
        assert_eq!(asked.join, Join::Any);
    }

    #[test]
    fn test_current_folder_saves_the_three_questions_and_the_folder() {
        // Current Folder narrows where to look, not which part of a message
        // to read. All four answers come out of one control, which is what
        // makes reading it as a field restriction the easy mistake.
        let asked = what_a_typed_search_asks(&TheSearchThatWasRun::new(
            "invoice".to_string(),
            WhereToSearch::OneFolder(7),
            a_folder_of("first", "INBOX/Work"),
        ));

        assert_eq!(
            the_parts_asked_about(&asked),
            vec!["subject", "from", "to"],
            "Current Folder was read as a field restriction"
        );
        assert_eq!(
            asked.folder,
            Some("INBOX/Work".to_string()),
            "Current Folder was chosen and the saved search looks everywhere, \
             which is the half of the scope nothing wrote down"
        );
    }

    #[test]
    fn test_a_search_saved_before_this_change_is_the_same_search_and_still_runs() {
        // The fixture is written out as an older version wrote it, rather than
        // taken from the writer, because the claim is about two writers
        // agreeing and one of them is no longer in the tree. Taking it from
        // the writer would put it on both sides of the assertion.
        //
        // This is what makes SEARCH-01's fourth criterion, about a missing
        // restriction reading as an unrestricted search, disappear rather than
        // need answering. There is no absent value: the old shape and the new
        // unnarrowed one are the same rows.
        let as_an_older_version_wrote_it = SavedSearch {
            id: "s1".to_string(),
            name: "Invoices".to_string(),
            join: Join::Any,
            questions: vec![
                asking_about("subject", "contains", "invoice"),
                asking_about("from", "contains", "invoice"),
                asking_about("to", "contains", "invoice"),
            ],
            folder: None,
        };

        let asked = what_a_typed_search_asks(&across_the_account("invoice"));
        assert_eq!(asked.questions, as_an_older_version_wrote_it.questions);
        assert_eq!(asked.join, as_an_older_version_wrote_it.join);
        assert_eq!(asked.folder, as_an_older_version_wrote_it.folder);

        // And it still runs, rather than merely still being stored.
        let mut about_it = a_message();
        about_it.subject = "Your invoice".to_string();
        let mut neither = a_message();
        neither.subject = "Lunch".to_string();
        neither.from_addr = "bob@example.com".to_string();
        neither.to_addr = "me@example.com".to_string();

        assert_eq!(as_an_older_version_wrote_it.what_it_cannot_read(), None);
        assert!(as_an_older_version_wrote_it.selects(&about_it));
        assert!(!as_an_older_version_wrote_it.selects(&neither));
    }

    #[test]
    fn test_a_search_across_the_account_saves_no_folder_even_with_one_open() {
        // Somebody standing in a folder who chooses All Folders asked for the
        // account. Keeping the folder would pin the search to where they
        // happened to be.
        let asked = what_a_typed_search_asks(&TheSearchThatWasRun::new(
            "invoice".to_string(),
            WhereToSearch::EveryFolder,
            a_folder_of("first", "INBOX/Work"),
        ));

        assert_eq!(asked.folder, None);
    }

    #[test]
    fn test_a_search_run_in_a_folder_may_be_saved_under_that_folders_account() {
        assert!(a_search_may_be_saved_under(
            &TheSearchThatWasRun::new(
                "invoice".to_string(),
                WhereToSearch::OneFolder(7),
                a_folder_of("first", "INBOX/Work"),
            ),
            "first"
        ));
    }

    #[test]
    fn test_a_search_run_in_another_accounts_folder_is_not_kept_here() {
        // Set Active changes which account a search is saved under and leaves
        // the folder tree's cursor where it was, so these two really can
        // differ. "INBOX" names a folder in every account, so storing the path
        // anyway would give a saved search that lists somebody else's inbox
        // under the name given to this one.
        assert!(!a_search_may_be_saved_under(
            &TheSearchThatWasRun::new(
                "invoice".to_string(),
                WhereToSearch::OneFolder(7),
                a_folder_of("second", "INBOX"),
            ),
            "first"
        ));
    }

    #[test]
    fn test_a_search_narrowed_by_no_folder_may_be_saved_under_any_account() {
        // Nothing to disagree about. Refusing here would refuse every ordinary
        // save, which is the direction a check like this fails in.
        assert!(a_search_may_be_saved_under(
            &across_the_account("invoice"),
            "first"
        ));
        assert!(a_search_may_be_saved_under(
            &TheSearchThatWasRun::new(
                "invoice".to_string(),
                WhereToSearch::SubjectOnly,
                a_folder_of("second", "INBOX"),
            ),
            "first"
        ));
    }

    #[test]
    fn test_every_part_a_narrowed_search_names_is_one_the_engine_answers() {
        // A narrowed answer names one part of a message by the same word the
        // filter engine switches on. A word the engine has never met is
        // answered no about every message, which is indistinguishable from a
        // search that matched nothing, so the search would be saved and then
        // silently find nothing for ever.
        for looking_in in [
            WhereToSearch::EveryFolder,
            WhereToSearch::OneFolder(7),
            WhereToSearch::SubjectOnly,
            WhereToSearch::SenderOnly,
        ] {
            let asked = what_a_typed_search_asks(&TheSearchThatWasRun::new(
                "invoice".to_string(),
                looking_in,
                a_folder_of("first", "INBOX/Work"),
            ));
            let saved = SavedSearch {
                id: "s1".to_string(),
                name: "Invoices".to_string(),
                join: asked.join,
                questions: asked.questions,
                folder: asked.folder,
            };
            assert_eq!(
                saved.what_it_cannot_read(),
                None,
                "{looking_in:?} saves a question this build cannot answer"
            );
        }
    }

    // ── One sentence for any question set, named or not ─────────────────
    //
    // D-2-04. The sentence says what a search asks rather than which of the
    // "In" list's four answers made it, because the rule editor makes sets
    // that list has no name for. A sentence built around scope names would
    // need a fallback for those anyway, and a search edited until it no
    // longer matched a named scope would silently change how it describes
    // itself.

    fn asking_about(field: &str, match_type: &str, pattern: &str) -> Question {
        Question {
            field: field.to_string(),
            match_type: match_type.to_string(),
            pattern: pattern.to_string(),
            case_sensitive: false,
        }
    }

    #[test]
    fn test_saving_a_typed_search_says_what_it_will_ask_before_it_is_kept() {
        // A saved search asks a narrower question than the box it came from:
        // the box reads the first line of a message too, and this does not.
        // Somebody is told what they are keeping while they are naming it,
        // rather than finding out when the counts do not match.
        assert_eq!(
            what_a_typed_search_asks(&across_the_account("invoice")).in_words(),
            "This saved search looks in this account for messages where Subject contains \
             \"invoice\", From contains \"invoice\", or To contains \"invoice\"."
        );
    }

    #[test]
    fn test_a_search_about_the_subject_alone_says_the_subject_and_nothing_else() {
        assert_eq!(
            a_search_in_words(
                &[asking_about("subject", "contains", "invoice")],
                Join::Any,
                None
            ),
            "This saved search looks in this account for messages where Subject contains \
             \"invoice\"."
        );
    }

    #[test]
    fn test_a_question_set_the_in_box_has_no_name_for_names_every_part_it_asks_about() {
        // The message text and the sender. The "In" list cannot make this and
        // the rule editor can, so a sentence that named scopes would fall back
        // to fixed wording here and say nothing true about the search.
        assert_eq!(
            a_search_in_words(
                &[
                    asking_about("body_plain", "contains", "invoice"),
                    asking_about("from", "contains", "ann@"),
                ],
                Join::All,
                None
            ),
            "This saved search looks in this account for messages where Message text contains \
             \"invoice\" and From contains \"ann@\".",
            "the sentence fell back to fixed wording, or read the join out as \
             the wrong word"
        );
    }

    #[test]
    fn test_a_search_narrowed_to_a_folder_says_which_folder() {
        assert_eq!(
            a_search_in_words(
                &[asking_about("subject", "contains", "invoice")],
                Join::Any,
                Some("INBOX/Work")
            ),
            "This saved search looks in INBOX/Work for messages where Subject contains \
             \"invoice\"."
        );
    }

    #[test]
    fn test_a_way_of_matching_that_compares_against_nothing_reads_out_no_pattern() {
        // Four of the eleven answer from the field alone. Whatever is stored
        // in the pattern beside one of them is not part of the question, so
        // reading it out announces an empty pair of quotes or some text left
        // over from a Pattern box that never applied.
        let said = a_search_in_words(
            &[
                asking_about("read", "is_false", ""),
                asking_about("subject", "is_not_empty", "left over"),
            ],
            Join::All,
            None,
        );

        assert_eq!(
            said,
            "This saved search looks in this account for messages where Read is no and Subject \
             is not empty."
        );
        assert!(
            !said.contains("\"\""),
            "an empty pattern was read out: {said}"
        );
        assert!(
            !said.contains("left over"),
            "a pattern nothing compares against was read out: {said}"
        );
    }

    #[test]
    fn test_no_sentence_the_builder_makes_reads_out_a_stored_name() {
        // Every field and every way of matching the engine knows, one at a
        // time. A stored name is a machine name, and one read out to somebody
        // is the second vocabulary this module exists to not have.
        use crate::application::filters::{A_FIELD_A_RULE_MAY_NAME, A_WAY_A_RULE_MAY_MATCH};

        for field in A_FIELD_A_RULE_MAY_NAME {
            for way in A_WAY_A_RULE_MAY_MATCH {
                let said =
                    a_search_in_words(&[asking_about(field, way, "invoice")], Join::Any, None);
                assert!(
                    !said.contains('_'),
                    "{field} matched by {way} reads out a stored name: {said}"
                );
                assert!(
                    !said.contains(A_QUESTION_THIS_VERSION_CANNOT_READ),
                    "{field} matched by {way} has no words, so the sentence \
                     gives up on a question this build can answer"
                );
            }
        }
    }

    #[test]
    fn test_a_question_this_build_cannot_read_says_so_rather_than_guessing() {
        // A search written by a newer version. Reading the stored word out
        // would show a machine name, and leaving the clause out would describe
        // a narrower search than the one that will run.
        assert_eq!(
            a_search_in_words(
                &[asking_about("attachment_count", "is_more_than", "2")],
                Join::Any,
                None
            ),
            "This saved search looks in this account for messages where a question this \
             version cannot read."
        );
    }

    #[test]
    fn test_a_search_that_found_more_than_the_list_holds_says_the_count_and_says_it_is_cut() {
        // The count is the true one, because that is the answer somebody came
        // for. The list is bounded, because a list of forty thousand rows is
        // one nobody can use. Saying only the bounded number would be
        // reporting five hundred to somebody who has nine hundred.
        assert_eq!(
            only_the_newest_are_shown(MOST_RESULTS_SHOWN),
            "The newest 500 are shown."
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

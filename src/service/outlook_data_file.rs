//! Reading an Outlook data file: mail, calendar, contacts, tasks and notes.
//!
//! Somebody leaving Outlook arrives with one file holding everything they have:
//! twenty years of mail, every appointment, the address book, the tasks and the
//! notes, all in the folders they kept them in. This opens that file and hands
//! its contents over one at a time.
//!
//! [`crate::service::mailbox_archive`] is the same job for a zip file or a
//! folder of saved messages, and this is its sibling: a file is opened, what it
//! holds is offered as names, one thing at a time is read through, and anything
//! that would exhaust this computer is refused with a sentence saying why.
//!
//! # Both kinds of file
//!
//! Outlook has written two forms of this file. The older one dates from Outlook
//! 97 and holds its text in whatever alphabet the computer that wrote it was
//! set to; the newer one arrived with Outlook 2003 and is Unicode throughout.
//! Both open here and nothing above this has to ask which it got.
//! [`OutlookDataFile::is_the_older_kind`] answers for anything that has to say
//! so out loud.
//!
//! # A password on the file is not a lock on the mail
//!
//! Outlook will ask for a password before it opens a file somebody set one on,
//! and that password never encrypted anything: it is a number written in the
//! file that Outlook checks and every other reader ignores. So a file with one
//! opens here, and [`OutlookDataFile::has_a_password_on_it`] says so, because
//! somebody who has forgotten a password Outlook is asking for needs to hear
//! that their mail is not locked away rather than watch this fail obscurely.
//!
//! # Memory, which is the shape of this module
//!
//! Looking the file over keeps a name and a count for each folder and nothing
//! else, so a file larger than this computer's memory can be planned from.
//! Filling a folder then reads one item through and lets it go again.
//!
//! What that cannot bound is a single item larger than memory, because the
//! reader underneath reads one property whole before anything here sees a byte
//! of it. Asking for only the properties this program uses keeps the largest
//! things in a data file out of that read, and what really came out is counted
//! afterwards against [`HowMuchToAllow`]. No size the file states about itself
//! is ever believed.
//!
//! # What this does not bring across yet, and says so
//!
//! [`OutlookDataFile::what_stayed_behind`] counts four things, so that none of
//! them is lost in silence:
//!
//! - **Files carried on anything.** Mail comes across with its words and its
//!   markup, and everything else with what it said. Whatever was attached, from
//!   an invoice on a message to a photograph on a contact, stays in the file.
//! - **An appointment that repeats.** It comes across as the single appointment
//!   it first was, because the pattern Outlook writes is a shape of its own that
//!   nothing here reads yet.
//! - **Anything that is none of these five kinds.** A meeting reply, a delivery
//!   report, a form somebody's company wrote.
//! - **Anything the file would not give up.** A damaged folder or item is
//!   counted and passed over, and the rest of the file still comes across.
//!
//! Three more are not counted, because nothing here can tell which items had
//! them without reading more than it reads. An appointment's reminder, its
//! guests, and the words somebody filed things under all stay in the file.
//! Whatever puts this in front of somebody has to say so in a sentence.
//!
//! # One thread
//!
//! The reader underneath keeps its file behind [`std::rc::Rc`], so an open data
//! file cannot be moved to another thread. Whatever imports one has to do it on
//! the thread that opened it.
//!
//! # This has never read a real Outlook data file
//!
//! An Outlook data file cannot be written by this program or by the reader
//! underneath it, so no test here builds one. What is tested is everything a
//! file is not needed for: the sentences, the limits and the refusals, the
//! reading of a moment, which of the five kinds a thing is, the alphabets, and
//! every one of the five turned into what this program keeps. What is not
//! tested is the walk through a real file's folders and the reading of a real
//! item out of one. Anything that puts this in front of somebody should say it
//! is experimental where they will see it.

use crate::common::types::EmailAddress;
use crate::common::{Error, Result};
use crate::data::message_cache::{
    AddressEntry, CalendarEventEntry, ContactEntry, EmailEntry, NoteEntry, PhoneEntry, TaskEntry,
};
use crate::service::mime::ParsedMessage;
use outlook_pst::messaging::folder::Folder;
use outlook_pst::messaging::store::Store;
use outlook_pst::ndb::node_id::NodeId;
use outlook_pst::{AnsiPstFile, UnicodePstFile};
use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// How every Outlook data file begins, whichever of the two kinds it is.
///
/// Only ever asked once opening one has already failed, and only to choose
/// between two sentences: a file that never was an Outlook file, and one that
/// is and stops partway through. Those ask different things of the person in
/// front of the screen.
const HOW_ONE_BEGINS: &[u8] = b"!BDN";

/// What Outlook writes a little further in to say which of its files this is.
///
/// Two letters, eight bytes in. The file somebody keeps their own mail in says
/// one thing here and the copy Outlook keeps of a mailbox living on a server
/// says another, and the two are alike in every other way this far in.
const WHICH_OUTLOOK_FILE_IT_IS: std::ops::Range<usize> = 8..10;

/// What the file Outlook keeps somebody's own mail in says there.
const A_FILE_OF_SOMEBODYS_OWN_MAIL: &[u8] = b"SM";

/// What the copy of a mailbox living on a server says there.
const A_COPY_OF_A_MAILBOX_ON_A_SERVER: &[u8] = b"SO";

// ── Limits ──────────────────────────────────────────────────────────────────

/// How much of a data file this program will read before refusing it.
///
/// A data file is a stranger's file, and a small one can claim to hold more
/// than every disk ever made. Reading it as far as it says fills this
/// computer's memory and the program stops. So there is a limit on every part
/// of it that can grow: the first two bound what looking a file over costs, and
/// the last two bound what importing one costs.
///
/// A value rather than four constants, so a test can hand this small numbers
/// and watch a refusal really happen. A limit nothing has ever reached is a
/// limit nobody has watched work.
///
/// None of these is worked out from what the file says about itself. Every
/// count and size recorded in a data file was written by whoever built it, so
/// what is counted here is what really came out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HowMuchToAllow {
    /// How many folders a data file may hold.
    ///
    /// Every one of them is a name to keep, a folder to open and a row in
    /// whatever comes next, so a file claiming a million of them is work nobody
    /// asked for before an item has been looked at.
    pub most_folders_in_it: usize,
    /// How many items one folder may hold.
    ///
    /// Filling a folder holds the place of every item in it at once, which is
    /// the one part of reading a folder that grows with the folder.
    pub most_items_in_one_folder: usize,
    /// How much any one item may come to once it has been read.
    ///
    /// A whole item is held while it is handed over, so this is a limit of how
    /// this program reads rather than of what a data file may contain: a single
    /// message larger than this is left where it is, with a sentence saying so,
    /// rather than handed over half read.
    pub most_one_item_comes_to: u64,
    /// How much the whole file may come to, added up as it is read.
    ///
    /// The limit above is about one item, and a thousand items each just under
    /// it add up to something no disk holds.
    pub most_the_whole_of_it_comes_to: u64,
}

impl Default for HowMuchToAllow {
    /// The limits this program ships with.
    ///
    /// Each is generous for the largest mailbox anybody has kept and nowhere
    /// near what a hostile file asks for. A mailbox somebody has run for twenty
    /// years is a few hundred folders, so ten thousand is past all of that; one
    /// folder of two hundred thousand messages is larger than any real inbox;
    /// a quarter of a gigabyte is already more than one message this program
    /// can hold and file comfortably.
    fn default() -> Self {
        Self {
            most_folders_in_it: 10_000,
            most_items_in_one_folder: 200_000,
            most_one_item_comes_to: 256 * 1024 * 1024,
            most_the_whole_of_it_comes_to: 20 * 1024 * 1024 * 1024,
        }
    }
}

/// How much has really come out of one data file so far.
///
/// Kept for the life of the open file rather than reset at each item, because
/// the limit it is counted against is about the whole import.
#[derive(Debug, Clone, Copy)]
struct HowMuchHasComeOut {
    allowed: HowMuchToAllow,
    so_far: u64,
}

impl HowMuchHasComeOut {
    /// Nothing out yet, from a file that may be read this much.
    fn of(allowed: HowMuchToAllow) -> Self {
        Self { allowed, so_far: 0 }
    }

    /// Take one more item of this size, or refuse the file for going past what
    /// it may come to.
    ///
    /// The one place either limit on reading a file through is applied, so
    /// every kind of item is held to the same one. What is counted is what
    /// really came out, never a size the file stated about itself.
    ///
    /// Asked after the thing has been read, because the reader underneath reads
    /// one property whole and nothing here sees a byte before it has. So this
    /// does not stop one absurd item arriving in memory once; what it stops is
    /// that item being copied on into a message and a row, and the thousand
    /// after it arriving as well.
    fn and_now(&mut self, at: &Path, folder: &str, came_to: u64) -> Result<()> {
        if came_to > self.allowed.most_one_item_comes_to {
            return Err(one_item_comes_to_more_than_will_be_read(
                at,
                folder,
                self.allowed.most_one_item_comes_to,
            ));
        }
        self.so_far = self.so_far.saturating_add(came_to);
        if self.so_far > self.allowed.most_the_whole_of_it_comes_to {
            return Err(the_whole_of_it_comes_to_more_than_will_be_read(
                at,
                self.allowed.most_the_whole_of_it_comes_to,
            ));
        }
        Ok(())
    }
}

// ── The sentences ───────────────────────────────────────────────────────────

/// One size, in the largest unit that leaves a number somebody can hold.
///
/// These go into sentences that get read out. Two hundred and sixty eight
/// million four hundred and thirty five thousand four hundred and fifty six is
/// a number nobody can hold, and it is 256 megabytes.
fn said_as_a_size(bytes: u64) -> String {
    const ONE_MEGABYTE: u64 = 1024 * 1024;
    const ONE_GIGABYTE: u64 = ONE_MEGABYTE * 1024;
    match bytes {
        huge if huge >= ONE_GIGABYTE => format!("{} gigabytes", huge / ONE_GIGABYTE),
        large if large >= ONE_MEGABYTE => format!("{} megabytes", large / ONE_MEGABYTE),
        small => format!("{small} bytes"),
    }
}

/// What to say about a file that is not an Outlook data file at all.
///
/// The commonest thing that goes wrong here, and what somebody needs to hear is
/// which file to look for instead, rather than what this program could not
/// read.
fn is_not_an_outlook_data_file(at: &Path) -> Error {
    Error::Other(format!(
        "{} is not an Outlook data file this program can open. Choose the file Outlook keeps \
         your mail in, whose name ends in .pst.",
        at.display()
    ))
}

/// What to say about a data file that is one and stops before its end.
///
/// A copy off a failing disk, half a file restored from a backup, a transfer
/// that stopped. Told it is not an Outlook file at all, somebody goes looking
/// for a different file; told it stops partway through, they go and fetch
/// another copy of the one they already have.
fn stops_partway_through(at: &Path) -> Error {
    Error::Other(format!(
        "{} stops partway through, so what is in it cannot be read. Ask Outlook for another \
         copy of it.",
        at.display()
    ))
}

/// What to say about the copy Outlook keeps of a mailbox living on a server.
///
/// It begins exactly the way the file somebody's own mail is in begins, so
/// without this it is told it stops partway through, and somebody goes looking
/// for another copy of a file that was never the one they wanted. What they
/// need is either the account itself or a real data file written out of it.
fn is_a_copy_of_a_mailbox_on_a_server(at: &Path) -> Error {
    Error::Other(format!(
        "{} is the copy Outlook keeps of a mailbox that lives on a server, not the file it keeps \
         your own mail in. Add that mailbox here as an account, or ask Outlook to export it to an \
         Outlook data file first.",
        at.display()
    ))
}

/// What to say about a data file with nothing in it.
///
/// An import of nothing that says nothing looks exactly like a broken import,
/// and the difference is the whole of what somebody needs in order to decide
/// what to do next.
fn there_is_nothing_in(at: &Path) -> Error {
    Error::Other(format!(
        "There is nothing in {}. Choose the Outlook data file your mail is really in.",
        at.display()
    ))
}

/// What to say about a data file with more folders in it than will be opened.
fn holds_more_folders_than_this_program_will_open(at: &Path, most: usize) -> Error {
    Error::Other(format!(
        "{} holds more than {most} folders, which is more than this program will open at once. \
         Import your mail a few folders at a time instead.",
        at.display()
    ))
}

/// What to say about one folder with more in it than will be read.
fn one_folder_holds_more_than_this_program_will_open(
    at: &Path,
    folder: &str,
    most: usize,
) -> Error {
    Error::Other(format!(
        "{folder} in {} holds more than {most} things, which is more than this program will \
         read from one folder. Nothing was imported from it.",
        at.display()
    ))
}

/// What to say about one item that comes to more than will be read.
fn one_item_comes_to_more_than_will_be_read(at: &Path, folder: &str, most: u64) -> Error {
    Error::Other(format!(
        "Something in {folder} in {} comes to more than {}, which is more than this program \
         will read in one piece. It is still in there and nothing was imported from it.",
        at.display(),
        said_as_a_size(most)
    ))
}

/// What to say about a whole data file that comes to more than will be read.
fn the_whole_of_it_comes_to_more_than_will_be_read(at: &Path, most: u64) -> Error {
    Error::Other(format!(
        "{} comes to more than {}, which is more than this program will read from one file. \
         What was imported before this point is on this computer; the rest is still in there.",
        at.display(),
        said_as_a_size(most)
    ))
}

// ── Opening one ─────────────────────────────────────────────────────────────

/// Open the Outlook data file at this path, of either kind.
pub fn opened(at: &Path) -> Result<OutlookDataFile> {
    opened_allowing(at, HowMuchToAllow::default())
}

/// The same, with the limits said out loud rather than taken as they ship.
pub fn opened_allowing(at: &Path, allowed: HowMuchToAllow) -> Result<OutlookDataFile> {
    let (store, the_older_kind) = a_store_read_from(at)?;

    // Before the folders, because a file somebody cannot get into is the first
    // thing they need to hear about and it costs one property to find out.
    let a_password_is_set = a_password_is_set_on(&store);
    if a_password_is_set {
        tracing::warn!(
            "{} has a password on it, which Outlook checks and nothing else does",
            at.display()
        );
    }

    let named = WhatTheNamesAreHere::read_from(&store);
    let mut found = WhatWasFound::of(at, allowed);
    every_folder_in(&store, &mut found)?;
    found.nothing_at_all()?;

    Ok(OutlookDataFile {
        opened_at: at.to_path_buf(),
        store,
        the_older_kind,
        a_password_is_set,
        worth_reading: what_is_worth_reading(&named),
        named,
        in_name_order: found.in_name_order(),
        folders: found.folders,
        come_out: HowMuchHasComeOut::of(allowed),
        allowed,
        stayed_behind: WhatStayedBehind {
            could_not_be_read: found.could_not_be_read,
            ..WhatStayedBehind::default()
        },
    })
}

/// One data file read far enough to ask it what it holds, and which kind it is.
///
/// The newer form is tried first because it is the one almost everybody has.
/// Only when both fail does anything look at the file's own bytes, and then
/// only to choose between two sentences.
fn a_store_read_from(at: &Path) -> Result<(Rc<dyn Store>, bool)> {
    if let Ok(store) = UnicodePstFile::open(at)
        .and_then(|file| outlook_pst::messaging::store::UnicodeStore::read(Rc::new(file)))
    {
        return Ok((store, false));
    }
    if let Ok(store) = AnsiPstFile::open(at)
        .and_then(|file| outlook_pst::messaging::store::AnsiStore::read(Rc::new(file)))
    {
        return Ok((store, true));
    }
    match how_it_begins(at) {
        Ok(HowItBegins::SomebodysOwnMail) => Err(stops_partway_through(at)),
        Ok(HowItBegins::ACopyOfAMailboxOnAServer) => Err(is_a_copy_of_a_mailbox_on_a_server(at)),
        Ok(HowItBegins::SomethingElse) => Err(is_not_an_outlook_data_file(at)),
        // The file could not be read at all: it is gone, or held open by
        // something else, or on a disk that stopped answering. That is a
        // different thing from any sentence above and says so.
        Err(why) => Err(Error::Other(format!(
            "{} could not be opened: {why}.",
            at.display()
        ))),
    }
}

/// What the first few bytes of a file say it is.
///
/// Only ever asked once opening it has already failed, and only to choose
/// between sentences. Each of them asks something different of the person in
/// front of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HowItBegins {
    /// Like the file Outlook keeps somebody's own mail in.
    SomebodysOwnMail,
    /// Like the copy Outlook keeps of a mailbox that lives on a server.
    ACopyOfAMailboxOnAServer,
    /// Like nothing this program opens.
    SomethingElse,
}

/// How a file begins, as far as choosing what to say about it.
fn how_it_begins(at: &Path) -> std::io::Result<HowItBegins> {
    let mut opening = [0u8; 10];
    let mut file = std::fs::File::open(at)?;
    if let Err(why) = file.read_exact(&mut opening) {
        // A file too short to hold the opening is too short to be one of
        // Outlook's, and that is an answer rather than a failure to read.
        if why.kind() == std::io::ErrorKind::UnexpectedEof {
            return Ok(HowItBegins::SomethingElse);
        }
        return Err(why);
    }
    if opening[..HOW_ONE_BEGINS.len()] != *HOW_ONE_BEGINS {
        return Ok(HowItBegins::SomethingElse);
    }
    match &opening[WHICH_OUTLOOK_FILE_IT_IS] {
        which if which == A_COPY_OF_A_MAILBOX_ON_A_SERVER => {
            Ok(HowItBegins::ACopyOfAMailboxOnAServer)
        }
        which if which == A_FILE_OF_SOMEBODYS_OWN_MAIL => Ok(HowItBegins::SomebodysOwnMail),
        // It begins the way Outlook's files do and says it is neither of them.
        // Damaged, then, rather than something else altogether.
        _ => Ok(HowItBegins::SomebodysOwnMail),
    }
}

/// Where Outlook records the password somebody set on a data file.
///
/// `PidTagPstPassword`. What is written there is a number worked out from the
/// password rather than the password, and nothing in the file is encrypted with
/// it, so anything other than nought means Outlook will ask and this will not.
const WHERE_A_PASSWORD_IS_RECORDED: u16 = 0x67FF;

/// Whether somebody set a password on this file.
fn a_password_is_set_on(store: &Rc<dyn Store>) -> bool {
    matches!(
        store.properties().get(WHERE_A_PASSWORD_IS_RECORDED),
        Some(outlook_pst::ltp::prop_context::PropertyValue::Integer32(set)) if *set != 0
    )
}

// ── Looking the folders over ────────────────────────────────────────────────

/// The most folders deep a data file is followed.
///
/// A real mailbox is a handful of folders deep. Anything past this is either a
/// mistake or a file whose folders point at each other in a circle, and the
/// path it would build is one nothing can open and nobody could hear read out.
///
/// The same depth [`crate::application::import_tree`] will accept, so nothing
/// this stops at could have been imported in any case. It is here to stop a
/// walk that never ends rather than to make a decision about somebody's mail,
/// and a folder left behind is said in the log rather than left in silence.
const MOST_FOLDERS_DEEP: usize = 24;

/// One folder in a data file: what Outlook calls it and how much is in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderInTheDataFile {
    /// Where it sits in the file, with a slash between each folder and the one
    /// inside it, spelled however Outlook spelled it.
    ///
    /// The same shape [`crate::application::import_tree`] reads out of a zip
    /// file, so the same rules decide where it lands on this computer.
    ///
    /// Outlook lets somebody put a slash in a folder's own name, and one of
    /// those reads here as two folders and lands as two on this computer.
    /// Nothing is lost by that: the name still names the one folder it came
    /// from and everything in it is still read out of it.
    pub named: String,
    /// How many things Outlook says are in it.
    ///
    /// The file's own count, so it is a number to show and never a number to
    /// read as far as. What is really read is whatever is really there.
    pub how_many_things_in_it: usize,
    /// Which folder it is, for finding it again.
    which: u32,
}

/// What looking a data file over turned up, and what it could not.
struct WhatWasFound {
    /// What was pointed at, for anything that has to be said about it.
    opened_at: PathBuf,
    /// How much of it may be read before it is refused.
    allowed: HowMuchToAllow,
    /// Every folder that could be named.
    folders: Vec<FolderInTheDataFile>,
    /// How many folders could not be read, whatever the reason.
    could_not_be_read: usize,
    /// The folders already walked, so a file whose folders point at each other
    /// in a circle is walked once rather than forever.
    already_walked: HashSet<u32>,
}

impl WhatWasFound {
    /// Nothing found yet, in a file that may be read this much.
    fn of(opened_at: &Path, allowed: HowMuchToAllow) -> Self {
        Self {
            opened_at: opened_at.to_path_buf(),
            allowed,
            folders: Vec::new(),
            could_not_be_read: 0,
            already_walked: HashSet::new(),
        }
    }

    /// Take one more folder, or refuse the file for holding too many.
    fn one_more_folder(
        &mut self,
        named: String,
        which: u32,
        how_many_things_in_it: usize,
    ) -> Result<()> {
        if self.folders.len() >= self.allowed.most_folders_in_it {
            return Err(holds_more_folders_than_this_program_will_open(
                &self.opened_at,
                self.allowed.most_folders_in_it,
            ));
        }
        self.folders.push(FolderInTheDataFile {
            named,
            how_many_things_in_it,
            which,
        });
        Ok(())
    }

    /// Count one folder that could not be read, and say in the log which.
    ///
    /// Counted so it can be said out loud at the end, and logged so somebody
    /// helping can find out which folder it was. A count on its own says mail
    /// was lost and not which; a log line on its own is a warning nobody gets.
    fn one_could_not_be_read(&mut self, which: &str) {
        self.could_not_be_read += 1;
        tracing::warn!("{which} could not be read, so it was left out of the import");
    }

    /// Refuse a data file that turned out to hold nothing at all.
    ///
    /// A file whose every folder was unreadable is a damaged file rather than
    /// an empty one, and telling somebody to choose a different file when the
    /// one they chose is the right file and damaged sends them looking in the
    /// wrong place.
    fn nothing_at_all(&self) -> Result<()> {
        if !self.folders.is_empty() {
            return Ok(());
        }
        if self.could_not_be_read > 0 {
            return Err(stops_partway_through(&self.opened_at));
        }
        Err(there_is_nothing_in(&self.opened_at))
    }

    /// The places in the folder list, in the order their names sort.
    ///
    /// A second order rather than sorting the list itself, because what a
    /// caller is given is in Outlook's own order and an import says what it did
    /// folder by folder: somebody listening to a long one hears their folders
    /// in the order they know them in. A search rather than a walk, because a
    /// file of forty thousand messages is looked up folder by folder and a walk
    /// each time is a walk for each.
    fn in_name_order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.folders.len()).collect();
        order.sort_by(|one, other| self.folders[*one].named.cmp(&self.folders[*other].named));
        order
    }
}

/// Every folder in one data file, from the top of somebody's folders down.
///
/// The top of the tree is Outlook's own root, which holds no mail and is named
/// something nobody chose, so what is recorded is everything under it.
fn every_folder_in(store: &Rc<dyn Store>, found: &mut WhatWasFound) -> Result<()> {
    let top = store.properties().ipm_sub_tree_entry_id().map_err(|why| {
        Error::Other(format!(
            "{} could not be read: its folders are not where an Outlook data file keeps them \
             ({why}).",
            found.opened_at.display()
        ))
    })?;
    let Ok(root) = store.open_folder(&top) else {
        // The one folder there is nothing to go on without, so this is the file
        // being damaged rather than one folder of it.
        return Err(stops_partway_through(&found.opened_at));
    };
    walked(store, &root, "", 0, found)
}

/// One folder walked, with `named_under` the name every folder in it carries in
/// front of its own.
///
/// The name is built on the way down rather than worked back out afterwards, so
/// it is made of the names Outlook gave and nothing else. A folder that cannot
/// be opened is counted and the walk goes on: a damaged folder in the middle of
/// a file must not cost somebody the other eleven folders of mail, and say
/// nothing about which one was the problem.
fn walked(
    store: &Rc<dyn Store>,
    folder: &Rc<dyn Folder>,
    named_under: &str,
    depth: usize,
    found: &mut WhatWasFound,
) -> Result<()> {
    let Some(hierarchy) = folder.hierarchy_table() else {
        return Ok(());
    };
    let inside: Vec<u32> = hierarchy
        .rows_matrix()
        .map(|row| u32::from(row.id()))
        .collect();

    for which in inside {
        // A file whose folders point at each other in a circle would otherwise
        // be walked until this computer stopped. The depth below bounds how far
        // down one path goes; this bounds how many times one folder is opened.
        if !found.already_walked.insert(which) {
            continue;
        }
        let Ok(entry_id) = store.properties().make_entry_id(NodeId::from(which)) else {
            found.one_could_not_be_read(&format!("a folder in {}", found.opened_at.display()));
            continue;
        };
        let Ok(inner) = store.open_folder(&entry_id) else {
            found.one_could_not_be_read(&format!("a folder in {}", found.opened_at.display()));
            continue;
        };
        let Ok(name) = inner.properties().display_name() else {
            found.one_could_not_be_read(&format!("a folder in {}", found.opened_at.display()));
            continue;
        };
        let named = if named_under.is_empty() {
            name
        } else {
            format!("{named_under}/{name}")
        };
        // Outlook's own count, kept to show and never read as far as. It was
        // written by whoever built the file and what is really read is whatever
        // is really there.
        let how_many = inner
            .properties()
            .content_count()
            .unwrap_or(0)
            .try_into()
            .unwrap_or(0usize);
        found.one_more_folder(named.clone(), which, how_many)?;

        if depth + 1 > MOST_FOLDERS_DEEP {
            tracing::warn!("{named} sits deeper than this program follows a folder");
            continue;
        }
        walked(store, &inner, &named, depth + 1, found)?;
    }
    Ok(())
}

// ── The properties Outlook names rather than numbers ────────────────────────

/// The sets of properties Outlook names rather than numbers.
///
/// Most of what a message holds sits at a number every Outlook file agrees on.
/// Almost everything that makes an appointment an appointment, a task a task or
/// a contact's email address an email address does not: those live at numbers
/// each file hands out for itself, and the file carries a list saying which
/// number it gave to which name. So the list is read once when the file is
/// opened and everything after that asks it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WhichSet {
    /// Everything about an appointment.
    Appointment,
    /// Everything about a task.
    Task,
    /// Everything about a contact that is not already at a fixed number.
    Address,
}

/// The tail every one of these sets ends with.
///
/// Outlook's own sets are alike but for the first of their four parts, so this
/// is checked once and that first part tells them apart.
const HOW_OUTLOOKS_OWN_SETS_END: [u8; 8] = [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46];

/// Which of the sets this program knows a name belongs to, if any.
fn which_set_is(guid: &outlook_pst::ltp::prop_context::GuidValue) -> Option<WhichSet> {
    if guid.data2() != 0 || guid.data3() != 0 || guid.data4() != &HOW_OUTLOOKS_OWN_SETS_END {
        return None;
    }
    match guid.data1() {
        0x0006_2002 => Some(WhichSet::Appointment),
        0x0006_2003 => Some(WhichSet::Task),
        0x0006_2004 => Some(WhichSet::Address),
        _ => None,
    }
}

/// What each of the properties Outlook names is numbered in one data file.
///
/// Empty is an answer rather than a failure: a file whose list of names cannot
/// be read still gives up its mail, its subjects and its bodies, and everything
/// that needed a name comes out missing rather than wrong.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct WhatTheNamesAreHere {
    numbered: std::collections::BTreeMap<(WhichSet, u32), u16>,
}

impl WhatTheNamesAreHere {
    /// Read the list of names one data file carries.
    fn read_from(store: &Rc<dyn Store>) -> Self {
        use outlook_pst::messaging::named_prop::{NamedPropertyGuid, NamedPropertyId};

        let mut here = Self::default();
        let Ok(map) = store.named_property_map() else {
            tracing::warn!(
                "this Outlook data file does not say what it numbered its own names, so \
                 appointments, tasks and contacts will come across with less on them"
            );
            return here;
        };
        let properties = map.properties();
        let Ok(entries) = properties.stream_entry() else {
            tracing::warn!("the list of names in this Outlook data file could not be read");
            return here;
        };
        for entry in entries {
            // A name spelled out in words rather than numbered. Categories are
            // the one of those this program would want, and reading them means
            // reading a second list, so they stay behind for now.
            let NamedPropertyId::Number(number) = entry.id() else {
                continue;
            };
            let NamedPropertyGuid::GuidIndex(index) = entry.guid() else {
                continue;
            };
            // The reader underneath takes the index already counted from the
            // start of the list rather than the raw number in the file, and
            // turns it back itself. Written the way its own example writes it.
            let Ok(place) = NamedPropertyGuid::try_from(index) else {
                continue;
            };
            let Ok(guid) = properties.lookup_guid(place) else {
                continue;
            };
            if let Some(set) = which_set_is(&guid) {
                here.numbered.insert((set, number), entry.prop_id());
            }
        }
        here
    }

    /// What one of the properties Outlook names is numbered in this file.
    fn id_of(&self, set: WhichSet, number: u32) -> Option<u16> {
        self.numbered.get(&(set, number)).copied()
    }
}

// ── Where in an item each thing is written ──────────────────────────────────
//
// Outlook writes most of what a message holds at a number every one of its
// files agrees on. These are those numbers, named for what they hold rather
// than for what Outlook's own documentation calls them.

/// What Outlook calls this thing: mail, an appointment, a contact and so on.
const MESSAGE_CLASS: u16 = 0x001A;
/// The subject, with the mark Outlook puts in front of it.
const SUBJECT: u16 = 0x0037;
/// The words of it.
const BODY: u16 = 0x1000;
/// The markup of it, when it has any.
const HTML_BODY: u16 = 0x1013;
/// Which alphabet the markup was written in.
const CODE_PAGE_OF_THE_MARKUP: u16 = 0x3FDE;
/// Which alphabet the rest of it was written in.
const CODE_PAGE_OF_THE_TEXT: u16 = 0x3FFD;
/// How important the person who made it said it was.
const IMPORTANCE: u16 = 0x0017;
/// When it was made.
const CREATED: u16 = 0x3007;
/// When it was last changed.
const LAST_CHANGED: u16 = 0x3008;
/// When it was sent.
const SENT_AT: u16 = 0x0039;
/// When it arrived.
const ARRIVED_AT: u16 = 0x0E06;
/// When an appointment starts, at the number every Outlook file agrees on.
///
/// Outlook writes this beside the one it numbers for itself. It is the answer
/// for a file that never said what it numbered its own names, and getting it
/// from anywhere else would put an appointment at an hour nobody chose.
const APPOINTMENT_STARTS_PLAINLY: u16 = 0x0060;
/// When an appointment ends, at the number every Outlook file agrees on.
const APPOINTMENT_ENDS_PLAINLY: u16 = 0x0061;
/// What every other mail program calls this message.
const INTERNET_MESSAGE_ID: u16 = 0x1035;
/// What the message this one answers is called.
const ANSWERS: u16 = 0x1042;
/// Everything this message is an answer to, oldest first.
const THE_CHAIN_BEHIND_IT: u16 = 0x1039;
/// Whether it carries any files.
const CARRIES_FILES: u16 = 0x0E1B;
/// The name of whoever the message says it is from.
const SENDER_NAME: u16 = 0x0042;
/// The address of whoever the message says it is from.
const SENDER_ADDRESS: u16 = 0x0065;
/// Which kind of address that is.
const SENDER_ADDRESS_KIND: u16 = 0x0064;
/// The internet address of whoever the message says it is from.
const SENDER_SMTP_ADDRESS: u16 = 0x5D02;
/// The name of whoever really sent it.
const WHO_REALLY_SENT_IT_NAME: u16 = 0x0C1A;
/// The address of whoever really sent it.
const WHO_REALLY_SENT_IT_ADDRESS: u16 = 0x0C1F;
/// Which kind of address that is.
const WHO_REALLY_SENT_IT_ADDRESS_KIND: u16 = 0x0C1E;
/// The internet address of whoever really sent it.
const WHO_REALLY_SENT_IT_SMTP_ADDRESS: u16 = 0x5D01;
/// Whether a recipient was written to, copied in, or copied in blind.
const RECIPIENT_KIND: u16 = 0x0C15;
/// The name of a folder, a contact or a recipient.
const DISPLAY_NAME: u16 = 0x3001;
/// The address of a recipient or a contact.
const EMAIL_ADDRESS: u16 = 0x3003;
/// Which kind of address that is.
const EMAIL_ADDRESS_KIND: u16 = 0x3002;
/// The internet address of a recipient or a contact.
const SMTP_ADDRESS: u16 = 0x39FE;

/// What a recipient number means when it says the message was written to them.
const WRITTEN_TO: i64 = 1;
/// What a recipient number means when it says they were copied in.
const COPIED_IN: i64 = 2;

// Contacts.
/// A contact's first name.
const GIVEN_NAME: u16 = 0x3A06;
/// A contact's last name.
const SURNAME: u16 = 0x3A11;
/// What a contact is called by people who know them.
const NICKNAME: u16 = 0x3A4F;
/// Who a contact works for.
const COMPANY: u16 = 0x3A16;
/// What a contact does there.
const JOB_TITLE: u16 = 0x3A17;
/// Which part of it they work in.
const DEPARTMENT: u16 = 0x3A18;
/// A contact's page on the web.
const WEB_PAGE: u16 = 0x3A51;
/// When a contact was born.
const BIRTHDAY: u16 = 0x3A42;
/// A contact's number at work.
const BUSINESS_PHONE: u16 = 0x3A08;
/// A contact's number at home.
const HOME_PHONE: u16 = 0x3A09;
/// A contact's number in their pocket.
const MOBILE_PHONE: u16 = 0x3A1C;
/// A contact's fax at work.
const BUSINESS_FAX: u16 = 0x3A24;
/// A contact's fax at home.
const HOME_FAX: u16 = 0x3A25;
/// A contact's other number.
const OTHER_PHONE: u16 = 0x3A1F;
/// The street a contact works in.
const WORK_STREET: u16 = 0x3A29;
/// The town a contact works in.
const WORK_CITY: u16 = 0x3A27;
/// The county a contact works in.
const WORK_STATE: u16 = 0x3A28;
/// The postal code a contact works at.
const WORK_POSTAL_CODE: u16 = 0x3A2A;
/// The country a contact works in.
const WORK_COUNTRY: u16 = 0x3A26;
/// The street a contact lives in.
const HOME_STREET: u16 = 0x3A5D;
/// The town a contact lives in.
const HOME_CITY: u16 = 0x3A59;
/// The county a contact lives in.
const HOME_STATE: u16 = 0x3A5C;
/// The postal code a contact lives at.
const HOME_POSTAL_CODE: u16 = 0x3A5B;
/// The country a contact lives in.
const HOME_COUNTRY: u16 = 0x3A5A;

// The names Outlook numbers for itself, by the number it gives them inside
// each set rather than by the number they end up at in any one file.

/// When an appointment starts.
const APPOINTMENT_STARTS: u32 = 0x820D;
/// When an appointment ends.
const APPOINTMENT_ENDS: u32 = 0x820E;
/// Where an appointment is.
const APPOINTMENT_LOCATION: u32 = 0x8208;
/// Whether an appointment takes the whole day.
const APPOINTMENT_IS_ALL_DAY: u32 = 0x8215;
/// How an appointment shows on somebody's calendar to other people.
const APPOINTMENT_BUSY_STATUS: u32 = 0x8205;
/// Whether an appointment has been called off, and other facts about it.
const APPOINTMENT_STATE: u32 = 0x8217;
/// Whether an appointment repeats.
const APPOINTMENT_REPEATS: u32 = 0x8223;
/// When a task is due.
const TASK_DUE_DATE: u32 = 0x8105;
/// Whether a task is finished.
const TASK_IS_DONE: u32 = 0x811C;
/// When a task was finished.
const TASK_DONE_AT: u32 = 0x810F;
/// How far along a task is, as Outlook's own numbers.
const TASK_STATUS: u32 = 0x8101;
/// A contact's first email address.
const FIRST_EMAIL_ADDRESS: u32 = 0x8083;
/// A contact's second email address.
const SECOND_EMAIL_ADDRESS: u32 = 0x8093;
/// A contact's third email address.
const THIRD_EMAIL_ADDRESS: u32 = 0x80A3;

// ── What one item said ──────────────────────────────────────────────────────

/// One property of an item, in a value this program has rather than the file's.
///
/// The reader underneath hands back its own values, and two of them cannot be
/// built outside it. So everything crosses into this shape at the one place
/// that touches the file, and every decision after that can be tested without
/// a data file to read.
#[derive(Debug, Clone, PartialEq)]
enum WhatItSaid {
    /// Text, already read in whichever alphabet the file wrote it in.
    Words(String),
    /// A whole number.
    Whole(i64),
    /// A moment, as hundred-nanosecond steps from the first of January 1601.
    When(i64),
    /// Yes or no.
    YesOrNo(bool),
    /// Bytes, for the one thing Outlook writes as bytes and means as text.
    Bytes(Vec<u8>),
}

/// Everything one item said about itself.
#[derive(Debug, Clone, Default, PartialEq)]
struct WhatTheItemSaid {
    said: std::collections::BTreeMap<u16, WhatItSaid>,
}

/// One item, and what the file it came out of numbered the names it uses.
///
/// The two travel together because half of what makes an appointment an
/// appointment sits at a number that file chose, and asking for one without
/// the other is asking the wrong file.
struct TheItem<'a> {
    said: &'a WhatTheItemSaid,
    names: &'a WhatTheNamesAreHere,
}

impl<'a> TheItem<'a> {
    /// One item read out of one data file.
    fn of(said: &'a WhatTheItemSaid, names: &'a WhatTheNamesAreHere) -> Self {
        Self { said, names }
    }

    /// The text at this number, when there is any.
    ///
    /// Blank is nothing. Outlook writes an empty string for a field somebody
    /// left alone, and a contact whose company is one space is a contact with
    /// no company.
    fn words(&self, id: u16) -> Option<&str> {
        match self.said.said.get(&id) {
            Some(WhatItSaid::Words(text)) if !text.trim().is_empty() => Some(text.as_str()),
            _ => None,
        }
    }

    /// The whole number at this number.
    fn whole(&self, id: u16) -> Option<i64> {
        match self.said.said.get(&id) {
            Some(WhatItSaid::Whole(number)) => Some(*number),
            _ => None,
        }
    }

    /// The moment at this number, as Outlook counts one.
    fn when(&self, id: u16) -> Option<i64> {
        match self.said.said.get(&id) {
            Some(WhatItSaid::When(steps)) => Some(*steps),
            _ => None,
        }
    }

    /// The yes or no at this number.
    fn yes(&self, id: u16) -> Option<bool> {
        match self.said.said.get(&id) {
            Some(WhatItSaid::YesOrNo(answer)) => Some(*answer),
            _ => None,
        }
    }

    /// The bytes at this number.
    fn bytes(&self, id: u16) -> Option<&[u8]> {
        match self.said.said.get(&id) {
            Some(WhatItSaid::Bytes(bytes)) => Some(bytes.as_slice()),
            _ => None,
        }
    }

    /// Where one of the names Outlook numbers for itself sits in this file.
    ///
    /// Nought when this file never numbered that name, which is not a number
    /// any property sits at, so asking for it finds nothing. That is the whole
    /// answer for a file whose list of names could not be read: everything that
    /// needed one comes out missing rather than wrong.
    fn named(&self, set: WhichSet, number: u32) -> u16 {
        self.names.id_of(set, number).unwrap_or(0)
    }
}

// ── Moments ─────────────────────────────────────────────────────────────────

/// How many seconds separate the day Outlook counts from and the day
/// everything else counts from.
///
/// Outlook counts from the first of January 1601 and the rest of the world from
/// the first of January 1970.
const FROM_1601_TO_1970: i64 = 11_644_473_600;

/// How many of Outlook's steps make one second.
const STEPS_IN_ONE_SECOND: i64 = 10_000_000;

/// How many nanoseconds one of Outlook's steps is.
const NANOSECONDS_IN_ONE_STEP: i64 = 100;

/// The first year nobody ever means.
///
/// Outlook marks a date somebody never set by writing a day in the year 4501
/// rather than by leaving the field out, so a task with no due date sorts to
/// the end of time instead of to the end of the list.
const NOBODY_MEANS_A_YEAR_THIS_FAR_OFF: i32 = 4000;

/// The moment one of Outlook's counts names, or nothing when it names none.
///
/// Universal time, always. Outlook's count is not in anybody's local clock and
/// reading it as though it were moves every appointment in the file by the
/// difference between two places.
fn the_moment_of(steps: i64) -> Option<chrono::DateTime<chrono::Utc>> {
    use chrono::Datelike;

    // Nought is a field nobody filled in, and anything below it is a date
    // before Outlook itself could count, which is a damaged value rather than
    // a moment in the seventeenth century.
    if steps <= 0 {
        return None;
    }
    let seconds = steps.div_euclid(STEPS_IN_ONE_SECOND) - FROM_1601_TO_1970;
    let nanoseconds = (steps.rem_euclid(STEPS_IN_ONE_SECOND) * NANOSECONDS_IN_ONE_STEP) as u32;
    let when = chrono::DateTime::from_timestamp(seconds, nanoseconds)?;
    (when.year() < NOBODY_MEANS_A_YEAR_THIS_FAR_OFF).then_some(when)
}

/// One of Outlook's counts as this program stores an instant.
fn said_as_a_moment(steps: i64) -> Option<String> {
    the_moment_of(steps).map(|when| when.to_rfc3339())
}

/// One of Outlook's counts as this program stores a whole day.
///
/// A whole day is midnight to midnight where the person was, and Outlook writes
/// the universal time that their midnight fell at. So a holiday somebody
/// entered in Berlin is stored at eleven the evening before, and taking the day
/// straight off that instant puts it on the wrong date.
///
/// Half a day is added before the day is taken, which lands on the right date
/// for everywhere whose clocks are less than twelve hours from universal time.
/// The two or three places further out than that are a day early, and there is
/// no way to do better without the name of the zone the file was written in,
/// which Outlook records in its own names for zones rather than in the ones
/// everything else uses.
fn said_as_a_day(steps: i64) -> Option<String> {
    let when = the_moment_of(steps)?;
    let nearest = when.checked_add_signed(chrono::TimeDelta::hours(12))?;
    Some(nearest.format(crate::common::moment::WHOLE_DAY).to_string())
}

// ── Which of the five kinds an item is ──────────────────────────────────────

/// The five kinds of thing this program keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WhatKind {
    Mail,
    Appointment,
    Contact,
    Task,
    Note,
}

/// Which of the five kinds Outlook's own name for a thing makes it, if any.
///
/// A data file holds all five mixed together in the same folders, and a good
/// deal besides: meeting replies, delivery reports, and forms whoever ran the
/// mail server wrote. Anything that is none of the five is counted and left,
/// because inventing a shape for it would put something nobody can read into
/// somebody's mailbox.
fn the_kind_of(class: &str) -> Option<WhatKind> {
    let called = class.to_ascii_lowercase();
    let is = |name: &str| called == name || called.starts_with(&format!("{name}."));
    if is("ipm.note") {
        return Some(WhatKind::Mail);
    }
    if is("ipm.appointment") {
        return Some(WhatKind::Appointment);
    }
    if is("ipm.contact") || is("ipm.distlist") {
        return Some(WhatKind::Contact);
    }
    if is("ipm.task") {
        return Some(WhatKind::Task);
    }
    if is("ipm.stickynote") {
        return Some(WhatKind::Note);
    }
    None
}

/// A subject with the mark Outlook writes in front of it taken off.
///
/// Outlook writes one character saying a prefix follows and one saying how long
/// it is, then the whole subject including the prefix. Handed over as it
/// stands, every reply in somebody's mailbox reads aloud beginning with a
/// character that is not a character.
fn without_the_subject_marker(subject: &str) -> &str {
    const THE_MARK: char = '\u{1}';
    let mut letters = subject.chars();
    if letters.next() != Some(THE_MARK) {
        return subject;
    }
    // The mark on its own, with nothing after it to say how long the prefix is,
    // is not the pair Outlook writes. Left alone rather than half taken off.
    match letters.next() {
        Some(_) => letters.as_str(),
        None => subject,
    }
}

// ── Text ────────────────────────────────────────────────────────────────────

/// Which alphabet a data file means when it says nothing.
///
/// Western European, which is what Outlook wrote on the computers the older
/// kind of file came from unless it was set otherwise.
const WESTERN_EUROPEAN: u16 = 1252;

/// Text out of a data file, read in the alphabet it was written in.
///
/// The newer kind of file is Unicode throughout and never reaches here. The
/// older kind holds its text as one byte a letter in whatever alphabet the
/// computer that wrote it was set to, and says which beside it. Read as though
/// every byte were a letter, a German subject comes out with a stand-in
/// character where each accent was, and a Japanese one comes out as nothing
/// anybody can read.
fn text_in(bytes: &[u8], code_page: Option<u16>) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    let page = code_page.unwrap_or(WESTERN_EUROPEAN);
    match text_the_system_reads(bytes, page) {
        Some(text) => text,
        None => as_western_european(bytes),
    }
}

/// Text read by Windows, which knows every alphabet Outlook ever wrote in.
///
/// Nothing rather than a guess when Windows will not read it, so the fallback
/// below is what answers rather than a half-decoded string.
#[cfg(target_os = "windows")]
fn text_the_system_reads(bytes: &[u8], code_page: u16) -> Option<String> {
    use windows::Win32::Globalization::{MULTI_BYTE_TO_WIDE_CHAR_FLAGS, MultiByteToWideChar};

    // Asked twice on purpose: once for how much room the answer needs and once
    // for the answer. Guessing the room is how a call like this writes past the
    // end of what it was given.
    let how_much = unsafe {
        MultiByteToWideChar(
            u32::from(code_page),
            MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0),
            bytes,
            None,
        )
    };
    let how_much = usize::try_from(how_much).ok().filter(|room| *room > 0)?;
    let mut room = vec![0u16; how_much];
    let written = unsafe {
        MultiByteToWideChar(
            u32::from(code_page),
            MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0),
            bytes,
            Some(&mut room),
        )
    };
    let written = usize::try_from(written).ok().filter(|count| *count > 0)?;
    room.truncate(written);
    Some(String::from_utf16_lossy(&room))
}

/// The same, anywhere else, where there is nothing to ask.
#[cfg(not(target_os = "windows"))]
fn text_the_system_reads(_bytes: &[u8], _code_page: u16) -> Option<String> {
    None
}

/// The thirty-two letters Western European puts where Latin-1 puts nothing.
///
/// Everything else in that alphabet is the letter with the same number, so this
/// is the whole of the difference between the two. A stand-in character stands
/// for the five numbers the alphabet leaves unused.
const WHAT_WESTERN_EUROPEAN_PUTS_IN_THE_GAP: [char; 32] = [
    '\u{20AC}', '\u{FFFD}', '\u{201A}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\u{FFFD}', '\u{017D}', '\u{FFFD}',
    '\u{FFFD}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}', '\u{0153}', '\u{FFFD}', '\u{017E}', '\u{0178}',
];

/// Text read as Western European, which is what is left when nothing else can.
fn as_western_european(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| match byte {
            0x80..=0x9F => WHAT_WESTERN_EUROPEAN_PUTS_IN_THE_GAP[usize::from(byte - 0x80)],
            other => char::from(*other),
        })
        .collect()
}

// ── Turning one item into what this program keeps ───────────────────────────

/// The one thing an Outlook data file cannot know about what is in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhereItIsGoing<'a> {
    /// The account everything read out of the file will belong to.
    pub account: &'a str,
}

/// A name for one imported thing that nothing else on this computer has.
fn a_name_of_its_own() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// When an item was made and when it was last changed, as this program stores
/// them.
///
/// Taken from the file, so twenty years of somebody's history arrives with its
/// own dates on it rather than all made today. Only a file that says nothing
/// falls back to now, and then the two agree, because a thing changed before it
/// was made is a row that sorts wrongly wherever it is shown.
fn made_and_changed(item: &TheItem<'_>) -> (String, String) {
    let made = item
        .when(CREATED)
        .and_then(said_as_a_moment)
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let changed = item
        .when(LAST_CHANGED)
        .and_then(said_as_a_moment)
        .unwrap_or_else(|| made.clone());
    (made, changed)
}

/// The words of an item, whether Outlook wrote them as text or as bytes.
fn the_words_of(item: &TheItem<'_>) -> Option<String> {
    item.words(BODY).map(str::to_string)
}

/// The markup of an item, when it has any.
///
/// Outlook writes this as bytes rather than as text and says which alphabet
/// beside it, so it is read here rather than where everything else is.
fn the_markup_of(item: &TheItem<'_>) -> Option<String> {
    if let Some(text) = item.words(HTML_BODY) {
        return Some(text.to_string());
    }
    let bytes = item.bytes(HTML_BODY)?;
    let page = item
        .whole(CODE_PAGE_OF_THE_MARKUP)
        .and_then(|page| u16::try_from(page).ok());
    let markup = text_in(bytes, page);
    (!markup.trim().is_empty()).then_some(markup)
}

/// One appointment, as this program keeps a calendar event.
///
/// The hours are stored as instants with universal time on them rather than as
/// a clock face beside the name of a zone. Outlook records the zone in its own
/// names for zones, and turning one of those into the name everything else uses
/// needs a list this program does not have, so a wrong guess would move
/// somebody's appointments by hours. An instant cannot be moved by a guess.
fn an_appointment_from(item: &TheItem<'_>, going_to: WhereItIsGoing<'_>) -> CalendarEventEntry {
    let (made, changed) = made_and_changed(item);
    // The one Outlook numbers for itself first, and the one every file agrees
    // on second. The second is what a file that never said what it numbered its
    // own names comes across on, and it is the same instant rather than a near
    // one, so nothing here has to guess an hour.
    let starts = item
        .when(item.named(WhichSet::Appointment, APPOINTMENT_STARTS))
        .or_else(|| item.when(APPOINTMENT_STARTS_PLAINLY));
    let ends = item
        .when(item.named(WhichSet::Appointment, APPOINTMENT_ENDS))
        .or_else(|| item.when(APPOINTMENT_ENDS_PLAINLY))
        .or(starts);
    let all_day = item
        .yes(item.named(WhichSet::Appointment, APPOINTMENT_IS_ALL_DAY))
        .unwrap_or(false);

    CalendarEventEntry {
        id: a_name_of_its_own(),
        account_id: going_to.account.to_string(),
        // It came out of a file rather than off a server. Anything that named a
        // provider here would be taken for a copy of something that provider
        // owns, and the next sync would delete it as one the server no longer
        // has.
        provider_event_id: None,
        calendar_id: None,
        summary: item
            .words(SUBJECT)
            .map(without_the_subject_marker)
            .unwrap_or_default()
            .to_string(),
        description: the_words_of(item),
        location: item
            .words(item.named(WhichSet::Appointment, APPOINTMENT_LOCATION))
            .map(str::to_string),
        // Empty when the file said nothing, rather than a moment nobody meant.
        // Whatever shows it says the characters as they stand, which is poor to
        // listen to and better than an invented hour.
        start_datetime: starts.and_then(said_as_a_moment).unwrap_or_default(),
        end_datetime: ends.and_then(said_as_a_moment).unwrap_or_default(),
        start_date: all_day.then(|| starts.and_then(said_as_a_day)).flatten(),
        end_date: all_day.then(|| ends.and_then(said_as_a_day)).flatten(),
        is_all_day: all_day,
        time_zone: None,
        status: if it_was_called_off(item) {
            "cancelled".to_string()
        } else {
            "confirmed".to_string()
        },
        // Outlook keeps what an appointment repeats as in a shape of its own
        // that nothing here reads yet, so a repeating appointment arrives as
        // the single one it first was and is counted as having done so.
        recurrence_rule: None,
        categories: String::new(),
        source_provider: None,
        etag: None,
        web_link: None,
        show_as: how_it_shows(item).to_string(),
        last_modified_remote: None,
        last_synced_at: None,
        attendees_json: None,
        reminders_json: None,
        created_at: made,
        updated_at: changed,
        pending: false,
        exception_dates: None,
        cut_from_event_id: None,
        provider_recurrence_id: None,
    }
}

/// Which bit Outlook sets on an appointment somebody called off.
const CALLED_OFF: i64 = 0x4;

/// Whether an appointment was called off.
fn it_was_called_off(item: &TheItem<'_>) -> bool {
    item.whole(item.named(WhichSet::Appointment, APPOINTMENT_STATE))
        .is_some_and(|state| state & CALLED_OFF != 0)
}

/// Whether an appointment repeats.
fn it_repeats(item: &TheItem<'_>) -> bool {
    item.yes(item.named(WhichSet::Appointment, APPOINTMENT_REPEATS))
        .unwrap_or(false)
}

/// How an appointment shows on somebody's calendar to other people.
fn how_it_shows(item: &TheItem<'_>) -> &'static str {
    match item.whole(item.named(WhichSet::Appointment, APPOINTMENT_BUSY_STATUS)) {
        Some(0) => "free",
        Some(1) => "tentative",
        Some(3) => "oof",
        // Busy is what Outlook does when nothing was chosen, and it is the
        // answer that keeps somebody from being booked over.
        _ => "busy",
    }
}

/// One contact, as this program keeps an address book entry.
fn a_contact_from(item: &TheItem<'_>, going_to: WhereItIsGoing<'_>) -> ContactEntry {
    let (made, _) = made_and_changed(item);
    let addresses = every_address_on(item);
    let numbers = every_number_on(item);
    let where_they_are = everywhere_on(item);

    ContactEntry {
        id: a_name_of_its_own(),
        account_id: going_to.account.to_string(),
        name: item.words(DISPLAY_NAME).unwrap_or_default().to_string(),
        given_name: item.words(GIVEN_NAME).map(str::to_string),
        family_name: item.words(SURNAME).map(str::to_string),
        // The one a row shows, which is the first the file listed.
        email: addresses
            .first()
            .map(|one| one.address.clone())
            .unwrap_or_default(),
        phone: numbers.first().map(|one| one.number.clone()),
        company: item.words(COMPANY).map(str::to_string),
        job_title: item.words(JOB_TITLE).map(str::to_string),
        website: item.words(WEB_PAGE).map(str::to_string),
        address: where_they_are.first().map(AddressEntry::on_one_line),
        birthday: item.when(BIRTHDAY).and_then(said_as_a_day),
        avatar_url: None,
        avatar_data_base64: None,
        source_provider: None,
        last_synced_at: None,
        vcard_raw: None,
        notes: the_words_of(item),
        favorite: false,
        created_at: made,
        nickname: item.words(NICKNAME).map(str::to_string),
        department: item.words(DEPARTMENT).map(str::to_string),
        relationship: None,
        emails_json: as_a_list(&addresses),
        phones_json: as_a_list(&numbers),
        addresses_json: as_a_list(&where_they_are),
        custom_fields_json: None,
        pending: false,
        // It came out of a file rather than off a server, so no address book
        // anywhere knows it and nothing may sync it away again.
        known_to: Vec::new(),
    }
}

/// A list of one contact's several somethings, as this program stores them.
///
/// Nothing rather than an empty list, so a contact with no numbers is stored
/// the same way as one whose numbers nothing has looked at yet.
fn as_a_list<T: serde::Serialize>(several: &[T]) -> Option<String> {
    if several.is_empty() {
        return None;
    }
    serde_json::to_string(several).ok()
}

/// Every email address on one contact, in the order Outlook lists them.
///
/// Only ones that are email addresses. A contact somebody kept inside a company
/// carries an address only that company's own server understood, and unlike the
/// same thing on a message it is worth nothing here: a message's sender address
/// is kept because dropping it drops the sender's name with it, and a contact's
/// name is its own field. What is left is an address the compose window would
/// take and nothing could deliver to, and the contact still arrives with their
/// name, their company and their telephone numbers.
fn every_address_on(item: &TheItem<'_>) -> Vec<EmailEntry> {
    let named = |number| item.named(WhichSet::Address, number);
    [
        ("Email", named(FIRST_EMAIL_ADDRESS)),
        ("Email 2", named(SECOND_EMAIL_ADDRESS)),
        ("Email 3", named(THIRD_EMAIL_ADDRESS)),
        ("Email", SMTP_ADDRESS),
        ("Email", EMAIL_ADDRESS),
    ]
    .into_iter()
    .filter_map(|(label, at)| {
        let address = item.words(at).filter(|written| written.contains('@'))?;
        Some(EmailEntry {
            label: label.to_string(),
            address: address.to_string(),
            name: item.words(DISPLAY_NAME).unwrap_or_default().to_string(),
        })
    })
    .fold(Vec::new(), |mut kept, one| {
        // The last two above are where Outlook writes the same address again on
        // a contact it made from a message, so a contact would otherwise arrive
        // with its one address listed twice.
        if !kept
            .iter()
            .any(|already: &EmailEntry| already.address.eq_ignore_ascii_case(&one.address))
        {
            kept.push(one);
        }
        kept
    })
}

/// Every telephone number on one contact, in the order a person would say them.
fn every_number_on(item: &TheItem<'_>) -> Vec<PhoneEntry> {
    [
        ("Work", BUSINESS_PHONE),
        ("Mobile", MOBILE_PHONE),
        ("Home", HOME_PHONE),
        ("Work fax", BUSINESS_FAX),
        ("Home fax", HOME_FAX),
        ("Other", OTHER_PHONE),
    ]
    .into_iter()
    .filter_map(|(label, at)| {
        Some(PhoneEntry {
            label: label.to_string(),
            number: item.words(at)?.to_string(),
        })
    })
    .collect()
}

/// Every postal address on one contact.
fn everywhere_on(item: &TheItem<'_>) -> Vec<AddressEntry> {
    [
        (
            "Work",
            WORK_STREET,
            WORK_CITY,
            WORK_STATE,
            WORK_POSTAL_CODE,
            WORK_COUNTRY,
        ),
        (
            "Home",
            HOME_STREET,
            HOME_CITY,
            HOME_STATE,
            HOME_POSTAL_CODE,
            HOME_COUNTRY,
        ),
    ]
    .into_iter()
    .filter_map(|(label, street, city, state, zip, country)| {
        let one = AddressEntry {
            label: label.to_string(),
            street: item.words(street).unwrap_or_default().to_string(),
            city: item.words(city).unwrap_or_default().to_string(),
            state: item.words(state).unwrap_or_default().to_string(),
            zip: item.words(zip).unwrap_or_default().to_string(),
            country: item.words(country).unwrap_or_default().to_string(),
        };
        // A contact who filled in neither address has two addresses of nothing
        // but a label, which read aloud as two rows saying "Work" and "Home".
        let anything = [&one.street, &one.city, &one.state, &one.zip, &one.country]
            .iter()
            .any(|part| !part.is_empty());
        anything.then_some(one)
    })
    .collect()
}

/// One task, as this program keeps one.
fn a_task_from(item: &TheItem<'_>, going_to: WhereItIsGoing<'_>) -> TaskEntry {
    let (made, changed) = made_and_changed(item);
    let done_at = item
        .when(item.named(WhichSet::Task, TASK_DONE_AT))
        .and_then(said_as_a_moment);
    let done = item
        .yes(item.named(WhichSet::Task, TASK_IS_DONE))
        .or_else(|| {
            item.whole(item.named(WhichSet::Task, TASK_STATUS))
                .map(|status| status == OUTLOOK_CALLS_A_TASK_DONE)
        })
        .unwrap_or(done_at.is_some());

    TaskEntry {
        id: a_name_of_its_own(),
        account_id: going_to.account.to_string(),
        task_list_id: None,
        title: item
            .words(SUBJECT)
            .map(without_the_subject_marker)
            .unwrap_or_default()
            .to_string(),
        description: the_words_of(item),
        // A day rather than an hour, because that is what a due date is and
        // what this program stores. Outlook writes it as the midnight that
        // began the day where the person was.
        due_date: item
            .when(item.named(WhichSet::Task, TASK_DUE_DATE))
            .and_then(said_as_a_day),
        is_completed: done,
        completed_at: done_at,
        priority: how_important(item).to_string(),
        display_order: 0,
        parent_task_id: None,
        created_at: made,
        updated_at: changed,
        remote_updated: None,
        pending: false,
        // It came out of a file rather than off a server, so there is no
        // provider whose words this would be in.
        remote_status: None,
    }
}

/// The number Outlook writes on a task somebody has finished.
const OUTLOOK_CALLS_A_TASK_DONE: i64 = 2;

/// How important the person who made an item said it was.
fn how_important(item: &TheItem<'_>) -> &'static str {
    match item.whole(IMPORTANCE) {
        Some(0) => "low",
        Some(2) => "high",
        _ => "normal",
    }
}

/// One note, as this program keeps one.
fn a_note_from(item: &TheItem<'_>, going_to: WhereItIsGoing<'_>) -> NoteEntry {
    let (made, changed) = made_and_changed(item);
    let body = the_words_of(item).unwrap_or_default();

    NoteEntry {
        id: a_name_of_its_own(),
        account_id: going_to.account.to_string(),
        folder_id: None,
        title: item
            .words(SUBJECT)
            .map(without_the_subject_marker)
            .map(str::to_string)
            .unwrap_or_else(|| the_first_line_of(&body)),
        body,
        // Outlook's notes are words and nothing else. Anything else here would
        // send the markup reader at text that is not markup.
        format: "plain".to_string(),
        pinned: false,
        created_at: made,
        updated_at: changed,
    }
}

/// The first line of a note, for one Outlook never asked a title for.
///
/// Outlook does not have a title on a note at all, so most of them arrive with
/// nothing to call them and a list of rows that all read aloud as nothing is a
/// list nobody can move through.
fn the_first_line_of(body: &str) -> String {
    body.lines().next().unwrap_or_default().trim().to_string()
}

// ── Turning one message into mail ───────────────────────────────────────────

/// Who a message went to.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct WhoItWentTo {
    /// Everybody it was written to.
    to: Vec<EmailAddress>,
    /// Everybody copied in.
    cc: Vec<EmailAddress>,
}

/// One message, as the raw bytes [`crate::service::mime::parse`] reads.
///
/// A data file does not hold a message the way a server hands one over: it
/// holds the pieces separately and Outlook puts them together when it shows
/// one. So the pieces are put back together here, by the same writer that
/// exports a message to a file, and what comes out reads back through this
/// program's own reader. There is a test that says so.
fn mail_written_from(item: &TheItem<'_>, went_to: &WhoItWentTo) -> Vec<u8> {
    crate::application::message_files::written_as_one_message(&a_message_from(item, went_to), &[])
}

/// One message, in the shape the rest of this program reads a message in.
fn a_message_from(item: &TheItem<'_>, went_to: &WhoItWentTo) -> ParsedMessage {
    ParsedMessage {
        // A data file holds a message in pieces and this puts them back
        // together; the transport headers are not among the pieces read here,
        // so there is nothing to say about a mailing list. Saying nothing is
        // the truthful answer rather than a gap being papered over: a message
        // imported from Outlook loses the warning it would have had, and
        // getting it back means reading the header property out of the file
        // and writing it into the bytes below, which is its own change.
        list_unsubscribe: None,
        subject: item
            .words(SUBJECT)
            .map(without_the_subject_marker)
            .unwrap_or_default()
            .to_string(),
        from: who_sent_it(item).into_iter().collect(),
        to: went_to.to.clone(),
        cc: went_to.cc.clone(),
        reply_to: Vec::new(),
        date: item
            .when(SENT_AT)
            .or_else(|| item.when(ARRIVED_AT))
            .or_else(|| item.when(CREATED))
            .and_then(said_as_a_moment),
        message_id: item.words(INTERNET_MESSAGE_ID).map(without_the_brackets),
        in_reply_to: item.words(ANSWERS).map(without_the_brackets),
        references: item
            .words(THE_CHAIN_BEHIND_IT)
            .map(|chain| chain.split_whitespace().map(without_the_brackets).collect())
            .unwrap_or_default(),
        body_plain: the_words_of(item),
        body_html: the_markup_of(item),
        // The files a message carried stay in the data file. Counted where the
        // message is read, so nobody has to find out by opening one.
        attachments: Vec::new(),
        receipt_to: None,
    }
}

/// One message name with the brackets a header wraps it in taken off.
///
/// The writer puts them back on, so leaving them here writes them twice and
/// the name that comes back out is a name no other message ever refers to.
fn without_the_brackets(named: &str) -> String {
    named
        .trim()
        .trim_start_matches('<')
        .trim_end_matches('>')
        .to_string()
}

/// Who a message says it is from.
///
/// Outlook records two senders: whoever the message says it is from, and
/// whoever really put it in the outbox. They differ when one person sends on
/// another's behalf, and the first is the one a reader expects to hear.
fn who_sent_it(item: &TheItem<'_>) -> Option<EmailAddress> {
    let said_to_be_from = one_person(
        item,
        SENDER_NAME,
        SENDER_SMTP_ADDRESS,
        SENDER_ADDRESS,
        SENDER_ADDRESS_KIND,
    );
    said_to_be_from.or_else(|| {
        one_person(
            item,
            WHO_REALLY_SENT_IT_NAME,
            WHO_REALLY_SENT_IT_SMTP_ADDRESS,
            WHO_REALLY_SENT_IT_ADDRESS,
            WHO_REALLY_SENT_IT_ADDRESS_KIND,
        )
    })
}

/// One person named on a message, from wherever their name and address sit.
///
/// The internet address first, then whatever address Outlook recorded. Mail
/// that never left a company carries an address only that company's server
/// understands, and it is kept as it was written rather than dropped: the
/// reader this program is for hears the sender's name because there is an
/// address beside it, and a message that reads aloud as being from nobody is
/// the worse of the two. Nothing is invented to fill the gap, because an
/// invented address is one somebody else may really have.
fn one_person(
    item: &TheItem<'_>,
    name_at: u16,
    smtp_at: u16,
    address_at: u16,
    kind_at: u16,
) -> Option<EmailAddress> {
    let address = item
        .words(smtp_at)
        .or_else(|| item.words(address_at))
        .unwrap_or_default();
    // A name and an address that are the same thing read aloud twice.
    let name = item.words(name_at).filter(|named| *named != address);

    // The kind is not used to turn an address away, only to say in the log that
    // one is not an internet address, because turning it away is what loses the
    // name with it.
    if let Some(kind) = item.words(kind_at) {
        if !kind.eq_ignore_ascii_case("SMTP") && !address.is_empty() {
            tracing::debug!("an address of kind {kind} was kept as it was written");
        }
    }

    match (address.is_empty(), name) {
        // Nothing at all. A message written with neither a name nor an address
        // is a message with no sender, and that is what it says.
        (true, None) => None,
        // A name and nothing else, which the file does give for some messages.
        // The name is written in both places, and that is a decision taken
        // after watching the other two fail.
        //
        // `From: Ada Lovelace <>` is the honest way to say there was no
        // address, and what reads a message back drops a sender whose address
        // is empty, so the name goes with it. `From: Ada Lovelace` is read as a
        // name with no address and goes the same way. Written in both places it
        // survives, and the message reads aloud as being from the one person
        // the file named rather than from nobody.
        //
        // Nothing is invented by it. A name with a space in it can never be
        // anybody's real address, and one without a space is still not one
        // while it has no `@` in it, so this cannot come to name a stranger.
        // Replying to it fails at once and says so, which is the visible half
        // of a trade whose other half would have been silent.
        (true, Some(named)) => Some(EmailAddress::new(
            named.to_string(),
            Some(named.to_string()),
        )),
        (false, name) => Some(EmailAddress::new(
            address.to_string(),
            name.map(str::to_string),
        )),
    }
}

// ── What was left behind ────────────────────────────────────────────────────

/// What reading a data file could not bring across.
///
/// Every one of these is somebody's mail, appointment or file still sitting in
/// their old program, so each is counted and offered rather than left in a log.
/// A count nobody is given is a gap nobody finds until they go looking for the
/// thing that is missing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WhatStayedBehind {
    /// Folders and items the file would not give up at all.
    pub could_not_be_read: usize,
    /// Things that are none of the five kinds this program keeps.
    pub not_one_of_these_kinds: usize,
    /// Things that carried files, which stayed in the data file.
    ///
    /// A message with an invoice on it, a contact with a photograph, a meeting
    /// with an agenda: all three are the same thing in a data file.
    pub things_that_carried_files: usize,
    /// Appointments that repeat, which came across as a single one.
    pub appointments_that_repeat: usize,
}

// ── The open file ───────────────────────────────────────────────────────────

/// An Outlook data file somebody pointed at, looked over and ready to be read.
pub struct OutlookDataFile {
    /// What was pointed at, for anything that has to be said about it.
    opened_at: PathBuf,
    /// The file itself, open.
    store: Rc<dyn Store>,
    /// Whether it is the older of the two forms Outlook has written.
    the_older_kind: bool,
    /// Whether somebody set a password on it.
    a_password_is_set: bool,
    /// What the properties Outlook names rather than numbers are called here.
    named: WhatTheNamesAreHere,
    /// Everything read off one thing in this file, worked out once.
    worth_reading: Vec<u16>,
    /// Every folder in it, in Outlook's own order.
    folders: Vec<FolderInTheDataFile>,
    /// The places in that list, in the order their names sort.
    in_name_order: Vec<usize>,
    /// How much has really come out of it so far.
    come_out: HowMuchHasComeOut,
    /// How much of it may be read before it is refused.
    allowed: HowMuchToAllow,
    /// What could not be brought across.
    stayed_behind: WhatStayedBehind,
}

impl OutlookDataFile {
    /// Every folder in the file, in Outlook's own order.
    pub fn what_it_holds(&self) -> &[FolderInTheDataFile] {
        &self.folders
    }

    /// What was pointed at.
    pub fn opened_at(&self) -> &Path {
        &self.opened_at
    }

    /// Whether this is the older of the two forms Outlook has written.
    ///
    /// Worth asking because the older one holds its text in whatever alphabet
    /// the computer that wrote it was set to, so a name that comes out wrong
    /// has an explanation somebody can be given.
    pub fn is_the_older_kind(&self) -> bool {
        self.the_older_kind
    }

    /// Whether somebody set a password on this file.
    pub fn has_a_password_on_it(&self) -> bool {
        self.a_password_is_set
    }

    /// What to say about the password, when there is one.
    ///
    /// Not a refusal, because the password never locked anything: what somebody
    /// needs to hear is that the mail Outlook is keeping from them is readable
    /// here, not that this program has hit the same wall.
    pub fn what_to_say_about_its_password(&self) -> Option<String> {
        self.a_password_is_set.then(|| {
            format!(
                "{} has a password on it. Outlook asks for that password and nothing in the \
                 file is locked by it, so your mail can still be imported.",
                self.opened_at.display()
            )
        })
    }

    /// What could not be brought across, for the sentence at the end.
    pub fn what_stayed_behind(&self) -> WhatStayedBehind {
        self.stayed_behind
    }

    /// Everything in one folder, one thing at a time.
    ///
    /// Only under a name the file itself offered, so a name that came from
    /// somewhere else finds nothing rather than something.
    ///
    /// What comes back holds one item at a time and lets it go again, which is
    /// what lets a mailbox larger than this computer's memory be imported at
    /// all. Anything the file will not give up is counted and passed over, so
    /// one damaged message does not cost somebody the rest of the folder. A
    /// refusal for size ends the folder and says why: the caller can go on to
    /// the next folder, and what it has already been handed is good.
    pub fn each_item_in(
        &mut self,
        folder: &str,
        going_to: WhereItIsGoing<'_>,
    ) -> Result<ItemsInAFolder<'_>> {
        let places = every_folder_named(&self.folders, &self.in_name_order, folder);
        let Some(first) = places.first() else {
            return Err(Error::Other(format!(
                "There is nothing called {folder} in {}.",
                self.opened_at.display()
            )));
        };
        let named = self.folders[*first].named.clone();
        let mut inside = Vec::new();
        for at in places {
            let node = self.folders[at].which;
            self.what_is_really_in(node, &named, &mut inside)?;
        }
        Ok(ItemsInAFolder {
            reading: self,
            named,
            account: going_to.account.to_string(),
            left: inside.into_iter(),
            refused: false,
        })
    }

    /// Which things one folder really holds, and what it says each comes to.
    ///
    /// Outlook's own count of what is in a folder is not read as far as. What
    /// is read is whatever is really listed, and the stated size beside each is
    /// believed in one direction only: a thing that says it is too large is
    /// refused before it is read, and a thing that says it is small enough is
    /// still measured after it has been.
    ///
    /// Added to what is already there rather than handed back on its own,
    /// because two folders of one name are read as the one folder they will
    /// become, and the limit on how much one folder holds is about that one.
    fn what_is_really_in(
        &mut self,
        node: u32,
        named: &str,
        inside: &mut Vec<(u32, u64)>,
    ) -> Result<()> {
        let store = self.store.clone();
        let entry_id = store
            .properties()
            .make_entry_id(NodeId::from(node))
            .map_err(|why| self.would_not_open(named, &why.to_string()))?;
        let folder = store
            .open_folder(&entry_id)
            .map_err(|why| self.would_not_open(named, &why.to_string()))?;
        let Some(contents) = folder.contents_table() else {
            // A folder with nothing in it has no list of contents at all, which
            // is an answer rather than a failure.
            return Ok(());
        };

        let context = contents.context();
        let says_how_large = context
            .columns()
            .iter()
            .position(|column| column.prop_id() == HOW_LARGE_IT_SAYS_IT_IS);
        for row in contents.rows_matrix() {
            if inside.len() >= self.allowed.most_items_in_one_folder {
                return Err(one_folder_holds_more_than_this_program_will_open(
                    &self.opened_at,
                    named,
                    self.allowed.most_items_in_one_folder,
                ));
            }
            let says = says_how_large
                .and_then(|at| one_column_of(contents.as_ref(), row, at))
                .and_then(|said| match said {
                    WhatItSaid::Whole(size) => u64::try_from(size).ok(),
                    _ => None,
                })
                .unwrap_or(0);
            inside.push((u32::from(row.id()), says));
        }
        Ok(())
    }

    /// What to say about a folder in the file that would not open.
    fn would_not_open(&self, named: &str, why: &str) -> Error {
        Error::Other(format!(
            "{named} in {} could not be read: {why}.",
            self.opened_at.display()
        ))
    }

    /// One thing in the file, read through and turned into what this program
    /// keeps.
    ///
    /// Nothing rather than a failure for something that could not be read or is
    /// none of the five kinds: both are counted and the rest of the folder goes
    /// on. A failure here is a refusal for size, and it ends the folder.
    fn one_item(
        &mut self,
        which: u32,
        says_it_comes_to: u64,
        folder: &str,
        account: &str,
    ) -> Result<Option<ItemInTheDataFile>> {
        // Believed only because it refuses. A file claiming one of its messages
        // is larger than this program will read is a file whose message is not
        // read, and a file lying the other way is caught by the measurement
        // below, so neither answer costs anything.
        if says_it_comes_to > self.allowed.most_one_item_comes_to {
            return Err(one_item_comes_to_more_than_will_be_read(
                &self.opened_at,
                folder,
                self.allowed.most_one_item_comes_to,
            ));
        }

        let store = self.store.clone();
        let Ok(entry_id) = store.properties().make_entry_id(NodeId::from(which)) else {
            self.one_could_not_be_read(folder);
            return Ok(None);
        };
        let Ok(message) = store.open_message(&entry_id, Some(&self.worth_reading)) else {
            self.one_could_not_be_read(folder);
            return Ok(None);
        };

        let said = what_was_said(message.properties().iter());
        let came_to = how_much_it_came_to(&said);
        self.come_out.and_now(&self.opened_at, folder, came_to)?;

        let item = TheItem::of(&said, &self.named);
        let Some(kind) = item.words(MESSAGE_CLASS).and_then(the_kind_of) else {
            self.stayed_behind.not_one_of_these_kinds += 1;
            return Ok(None);
        };
        // Every kind can carry a file, not only mail: a contact's photograph is
        // one, and so is the agenda somebody attached to a meeting.
        if item.yes(CARRIES_FILES).unwrap_or(false) {
            self.stayed_behind.things_that_carried_files += 1;
        }
        if kind == WhatKind::Appointment && it_repeats(&item) {
            self.stayed_behind.appointments_that_repeat += 1;
        }
        let going_to = WhereItIsGoing { account };
        let made = match kind {
            WhatKind::Mail => ItemInTheDataFile::Mail(mail_written_from(
                &item,
                // In the same alphabet the message itself was written in. A
                // recipient's name read in the wrong one is a name nobody in
                // the address book matches and nobody hears correctly.
                &who_it_went_to(message.as_ref(), which_alphabet(&said)),
            )),
            WhatKind::Appointment => {
                ItemInTheDataFile::Appointment(Box::new(an_appointment_from(&item, going_to)))
            }
            WhatKind::Contact => {
                ItemInTheDataFile::Contact(Box::new(a_contact_from(&item, going_to)))
            }
            WhatKind::Task => ItemInTheDataFile::Task(Box::new(a_task_from(&item, going_to))),
            WhatKind::Note => ItemInTheDataFile::Note(Box::new(a_note_from(&item, going_to))),
        };
        Ok(Some(made))
    }

    /// Count one thing the file would not give up, and say in the log where.
    fn one_could_not_be_read(&mut self, folder: &str) {
        self.stayed_behind.could_not_be_read += 1;
        tracing::warn!(
            "something in {folder} in {} could not be read, so it was left out of the import",
            self.opened_at.display()
        );
    }
}

/// Everything in one folder, one thing at a time.
///
/// The whole file is never held. What this carries is the place of each thing
/// in the folder, and each is read through and let go again as it is asked for.
pub struct ItemsInAFolder<'a> {
    /// The file it is coming out of.
    reading: &'a mut OutlookDataFile,
    /// The folder, for anything that has to be said about it.
    named: String,
    /// The account everything read out of it belongs to.
    account: String,
    /// The places of everything not handed over yet, and what each says it
    /// comes to.
    left: std::vec::IntoIter<(u32, u64)>,
    /// Whether the file has been refused, so nothing more is handed over.
    refused: bool,
}

impl Iterator for ItemsInAFolder<'_> {
    type Item = Result<ItemInTheDataFile>;

    fn next(&mut self) -> Option<Self::Item> {
        // Nothing after a refusal. A refusal is about the file rather than
        // about one thing in it, so going on would be reading past the point
        // the limits said to stop.
        if self.refused {
            return None;
        }
        loop {
            let (which, says_it_comes_to) = self.left.next()?;
            match self
                .reading
                .one_item(which, says_it_comes_to, &self.named, &self.account)
            {
                Ok(Some(item)) => return Some(Ok(item)),
                // Counted already, and the rest of the folder is still there.
                Ok(None) => continue,
                Err(why) => {
                    self.refused = true;
                    return Some(Err(why));
                }
            }
        }
    }
}

/// One thing out of an Outlook data file.
///
/// Every one of these is a shape this program already keeps, so nothing that
/// receives one has to know a data file exists.
#[derive(Debug, Clone)]
pub enum ItemInTheDataFile {
    /// A message, as the raw bytes [`crate::service::mime::parse`] reads.
    Mail(Vec<u8>),
    /// An appointment.
    Appointment(Box<CalendarEventEntry>),
    /// A contact.
    Contact(Box<ContactEntry>),
    /// A task.
    Task(Box<TaskEntry>),
    /// A note.
    Note(Box<NoteEntry>),
}

// ── Reading what the file says ──────────────────────────────────────────────

/// What a folder's list of contents says one thing in it comes to.
const HOW_LARGE_IT_SAYS_IT_IS: u16 = 0x0E08;

/// Every folder in the file with this name, in the order the file listed them.
///
/// A name is what a caller has to go on, and the one name that must never
/// quietly find something is one the file never offered, so nothing comes back
/// for a name that was not in the list.
///
/// All of them rather than one, because Outlook lets two folders in one place
/// carry one name, and [`crate::application::import_tree`] files two folders of
/// one name together anyway. Reading whichever of them a search happened to
/// land on would leave a whole folder of somebody's mail in the file with
/// nothing said about it.
fn every_folder_named(
    folders: &[FolderInTheDataFile],
    in_name_order: &[usize],
    named: &str,
) -> Vec<usize> {
    let Ok(found) = in_name_order.binary_search_by(|at| folders[*at].named.as_str().cmp(named))
    else {
        return Vec::new();
    };
    // Out from wherever the search landed, because two of one name sort beside
    // each other and the search may land on either.
    let mut first = found;
    while first > 0 && folders[in_name_order[first - 1]].named == named {
        first -= 1;
    }
    let mut last = found;
    while last + 1 < in_name_order.len() && folders[in_name_order[last + 1]].named == named {
        last += 1;
    }
    let mut places = in_name_order[first..=last].to_vec();
    // Back into the order the file listed them, so an import that says what it
    // did says it in the order somebody knows their folders in.
    places.sort_unstable();
    places
}

/// One column of one row of a table, in this program's own value.
fn one_column_of(
    table: &dyn outlook_pst::ltp::table_context::TableContext,
    row: &outlook_pst::ltp::table_context::TableRowData,
    at: usize,
) -> Option<WhatItSaid> {
    let context = table.context();
    let column = context.columns().get(at)?;
    let value = row.columns(context).ok()?.get(at)?.clone()?;
    let read = table.read_column(&value, column.prop_type()).ok()?;
    what_one_property_said(&read, None)
}

/// Everything one thing in the file said about itself.
///
/// The one place the reader's own values cross into this program's, so
/// everything after it can be tested without a data file to read.
fn what_was_said<'a>(
    properties: impl Iterator<Item = (&'a u16, &'a outlook_pst::ltp::prop_context::PropertyValue)>,
) -> WhatTheItemSaid {
    use outlook_pst::ltp::prop_context::PropertyValue;

    let all: Vec<(u16, &PropertyValue)> = properties.map(|(id, value)| (*id, value)).collect();
    // Which alphabet the text was written in, found before any of it is read,
    // because reading it in the wrong one is what turns a German subject into
    // a row of stand-in characters. The number itself is a number and needs no
    // alphabet, so it can be found first.
    let code_page = all.iter().find_map(|(id, value)| match (*id, value) {
        (CODE_PAGE_OF_THE_TEXT, PropertyValue::Integer32(page)) => u16::try_from(*page).ok(),
        _ => None,
    });

    WhatTheItemSaid {
        said: all
            .into_iter()
            .filter_map(|(id, value)| Some((id, what_one_property_said(value, code_page)?)))
            .collect(),
    }
}

/// One property of one thing, in this program's own value.
///
/// The kinds of value a data file holds that nothing here reads come back as
/// nothing at all, rather than as an empty one of something else.
fn what_one_property_said(
    value: &outlook_pst::ltp::prop_context::PropertyValue,
    code_page: Option<u16>,
) -> Option<WhatItSaid> {
    use outlook_pst::ltp::prop_context::PropertyValue;

    Some(match value {
        // Already Unicode, so the alphabet above has nothing to do with it.
        PropertyValue::Unicode(text) => WhatItSaid::Words(String::from_utf16_lossy(text.buffer())),
        // One byte a letter, in whichever alphabet the file named.
        PropertyValue::String8(text) => WhatItSaid::Words(text_in(text.buffer(), code_page)),
        PropertyValue::Integer16(number) => WhatItSaid::Whole(i64::from(*number)),
        PropertyValue::Integer32(number) => WhatItSaid::Whole(i64::from(*number)),
        PropertyValue::Integer64(number) => WhatItSaid::Whole(*number),
        PropertyValue::Boolean(answer) => WhatItSaid::YesOrNo(*answer),
        PropertyValue::Time(steps) => WhatItSaid::When(*steps),
        PropertyValue::Binary(bytes) => WhatItSaid::Bytes(bytes.buffer().to_vec()),
        _ => return None,
    })
}

/// Which alphabet one thing's text was written in, once it has been read.
///
/// Asked again here rather than carried out of the reading above, because the
/// recipients of a message sit beside it rather than on it and are read in a
/// second pass that has to be told the same answer.
fn which_alphabet(said: &WhatTheItemSaid) -> Option<u16> {
    match said.said.get(&CODE_PAGE_OF_THE_TEXT) {
        Some(WhatItSaid::Whole(page)) => u16::try_from(*page).ok(),
        _ => None,
    }
}

/// How much one thing really came to, once it had been read.
///
/// What is counted is what is really held: the text and the bytes. Everything
/// else is a number of a size known in advance, counted as the eight bytes it
/// is so that a thing made only of numbers is not counted as nothing at all.
///
/// This is the one measure the limits are applied to, and it is the same for
/// all five kinds, so no kind can grow past what the others are held to.
fn how_much_it_came_to(said: &WhatTheItemSaid) -> u64 {
    const A_NUMBER_IS_THIS_MANY_BYTES: u64 = 8;
    said.said
        .values()
        .map(|one| match one {
            WhatItSaid::Words(text) => text.len() as u64,
            WhatItSaid::Bytes(bytes) => bytes.len() as u64,
            _ => A_NUMBER_IS_THIS_MANY_BYTES,
        })
        .sum()
}

/// Everything this program reads off one thing in a data file.
///
/// Named out loud rather than taking whatever the file holds, and that is the
/// first half of what keeps a large file from filling this computer: the
/// largest things in a data file, the older markup Outlook keeps beside the
/// words and the files carried on a message, are never read at all.
fn what_is_worth_reading(names: &WhatTheNamesAreHere) -> Vec<u16> {
    let mut worth = vec![
        MESSAGE_CLASS,
        SUBJECT,
        BODY,
        HTML_BODY,
        CODE_PAGE_OF_THE_MARKUP,
        CODE_PAGE_OF_THE_TEXT,
        IMPORTANCE,
        CREATED,
        LAST_CHANGED,
        SENT_AT,
        ARRIVED_AT,
        APPOINTMENT_STARTS_PLAINLY,
        APPOINTMENT_ENDS_PLAINLY,
        INTERNET_MESSAGE_ID,
        ANSWERS,
        THE_CHAIN_BEHIND_IT,
        CARRIES_FILES,
        SENDER_NAME,
        SENDER_ADDRESS,
        SENDER_ADDRESS_KIND,
        SENDER_SMTP_ADDRESS,
        WHO_REALLY_SENT_IT_NAME,
        WHO_REALLY_SENT_IT_ADDRESS,
        WHO_REALLY_SENT_IT_ADDRESS_KIND,
        WHO_REALLY_SENT_IT_SMTP_ADDRESS,
        DISPLAY_NAME,
        EMAIL_ADDRESS,
        EMAIL_ADDRESS_KIND,
        SMTP_ADDRESS,
        GIVEN_NAME,
        SURNAME,
        NICKNAME,
        COMPANY,
        JOB_TITLE,
        DEPARTMENT,
        WEB_PAGE,
        BIRTHDAY,
        BUSINESS_PHONE,
        HOME_PHONE,
        MOBILE_PHONE,
        BUSINESS_FAX,
        HOME_FAX,
        OTHER_PHONE,
        WORK_STREET,
        WORK_CITY,
        WORK_STATE,
        WORK_POSTAL_CODE,
        WORK_COUNTRY,
        HOME_STREET,
        HOME_CITY,
        HOME_STATE,
        HOME_POSTAL_CODE,
        HOME_COUNTRY,
    ];
    for (set, number) in [
        (WhichSet::Appointment, APPOINTMENT_STARTS),
        (WhichSet::Appointment, APPOINTMENT_ENDS),
        (WhichSet::Appointment, APPOINTMENT_LOCATION),
        (WhichSet::Appointment, APPOINTMENT_IS_ALL_DAY),
        (WhichSet::Appointment, APPOINTMENT_BUSY_STATUS),
        (WhichSet::Appointment, APPOINTMENT_STATE),
        (WhichSet::Appointment, APPOINTMENT_REPEATS),
        (WhichSet::Task, TASK_DUE_DATE),
        (WhichSet::Task, TASK_IS_DONE),
        (WhichSet::Task, TASK_DONE_AT),
        (WhichSet::Task, TASK_STATUS),
        (WhichSet::Address, FIRST_EMAIL_ADDRESS),
        (WhichSet::Address, SECOND_EMAIL_ADDRESS),
        (WhichSet::Address, THIRD_EMAIL_ADDRESS),
    ] {
        if let Some(at) = names.id_of(set, number) {
            worth.push(at);
        }
    }
    worth
}

/// Who one message went to, out of its own list of recipients.
///
/// A data file keeps the recipients beside the message rather than on it, so
/// this is where a message's To and Cc lines come from. Anyone the file will
/// not give up is passed over: a message that arrives naming three of its four
/// recipients is worth more than one that does not arrive.
fn who_it_went_to(
    message: &dyn outlook_pst::messaging::message::Message,
    in_this_alphabet: Option<u16>,
) -> WhoItWentTo {
    let mut went_to = WhoItWentTo::default();
    let Some(table) = message.recipient_table() else {
        return went_to;
    };
    let names = WhatTheNamesAreHere::default();
    for row in table.rows_matrix() {
        let said = what_a_row_said(table.as_ref(), row, in_this_alphabet);
        let one = TheItem::of(&said, &names);
        let Some(who) = one_person(
            &one,
            DISPLAY_NAME,
            SMTP_ADDRESS,
            EMAIL_ADDRESS,
            EMAIL_ADDRESS_KIND,
        ) else {
            continue;
        };
        match one.whole(RECIPIENT_KIND) {
            Some(WRITTEN_TO) => went_to.to.push(who),
            Some(COPIED_IN) => went_to.cc.push(who),
            // A blind copy, and anything else the file lists. What this program
            // reads a message as has no blind-copy line to put one on, and
            // writing one onto a message received would tell everybody who
            // reads it afterwards something the sender chose not to say.
            _ => {}
        }
    }
    went_to
}

/// Everything one row of a table said.
fn what_a_row_said(
    table: &dyn outlook_pst::ltp::table_context::TableContext,
    row: &outlook_pst::ltp::table_context::TableRowData,
    in_this_alphabet: Option<u16>,
) -> WhatTheItemSaid {
    let context = table.context();
    let Ok(values) = row.columns(context) else {
        return WhatTheItemSaid::default();
    };
    WhatTheItemSaid {
        said: context
            .columns()
            .iter()
            .zip(values)
            .filter_map(|(column, value)| {
                let read = table.read_column(&value?, column.prop_type()).ok()?;
                Some((
                    column.prop_id(),
                    what_one_property_said(&read, in_this_alphabet)?,
                ))
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    /// A somewhere to build test files in, thrown away with the test.
    fn a_place_to_work() -> tempfile::TempDir {
        tempfile::tempdir().expect("a temporary folder to build test files in")
    }

    /// What was said about a data file that would not open.
    fn refused(at: &Path) -> String {
        match opened(at) {
            Err(why) => why.to_string(),
            Ok(open) => panic!(
                "{} opened and holds {} folders",
                at.display(),
                open.what_it_holds().len()
            ),
        }
    }

    /// A file that begins the way one of Outlook's does and then stops.
    fn half_an_outlook_file(inside: &Path, named: &str, which: &[u8]) -> PathBuf {
        let at = inside.join(named);
        let mut bytes = HOW_ONE_BEGINS.to_vec();
        // The four that come between, which say nothing about which file it is.
        bytes.extend([0u8; 4]);
        bytes.extend(which);
        bytes.extend(std::iter::repeat_n(0u8, 400));
        std::fs::write(&at, &bytes).expect("write half an Outlook file");
        at
    }

    /// Half of the file Outlook keeps somebody's own mail in.
    fn half_a_data_file(inside: &Path) -> PathBuf {
        half_an_outlook_file(inside, "half-a-data-file.pst", A_FILE_OF_SOMEBODYS_OWN_MAIL)
    }

    /// Half of the copy Outlook keeps of a mailbox living on a server.
    fn a_copy_of_a_mailbox(inside: &Path) -> PathBuf {
        half_an_outlook_file(
            inside,
            "a-mailbox-copy.ost",
            A_COPY_OF_A_MAILBOX_ON_A_SERVER,
        )
    }

    /// A file that is not an Outlook data file at all.
    fn not_a_data_file(inside: &Path) -> PathBuf {
        let at = inside.join("notes.txt");
        std::fs::write(&at, b"Dear Charles,\r\n\r\nThe engine.\r\n").expect("write it");
        at
    }

    #[test]
    fn test_a_file_that_is_not_a_data_file_at_all_says_which_file_to_choose() {
        // Somebody picks the wrong file, which is the commonest thing that goes
        // wrong here. A message, a spreadsheet, a photograph: none of them is
        // an Outlook data file, and what they need to hear is which file to
        // look for instead, not what this program failed to read.
        let place = a_place_to_work();

        let said = refused(&not_a_data_file(place.path()));

        assert!(said.contains("is not an Outlook data file"), "{said}");
        assert!(said.contains("Choose"), "{said}");
    }

    #[test]
    fn test_a_data_file_that_stops_partway_through_says_so_rather_than_saying_it_is_not_one() {
        // A copy off a failing disk, a file half restored from a backup, a
        // transfer that stopped. Told it is not an Outlook file, somebody goes
        // looking for a different file; told it stops partway through, they go
        // and fetch another copy of the one they already have.
        let place = a_place_to_work();

        let said = refused(&half_a_data_file(place.path()));

        assert!(said.contains("stops partway through"), "{said}");
    }

    #[test]
    fn test_the_copy_of_a_mailbox_on_a_server_is_told_apart_from_a_damaged_data_file() {
        // The file Outlook keeps of a mailbox that lives on a server begins
        // exactly the way the file of somebody's own mail begins, and this
        // program cannot open it. Told it stops partway through, somebody goes
        // looking for another copy of a file that was never the one they
        // wanted. It is a very common file to have and to pick by mistake.
        let place = a_place_to_work();

        let said = refused(&a_copy_of_a_mailbox(place.path()));

        assert!(said.contains("lives on a server"), "{said}");
        assert!(said.contains("account"), "{said}");
        assert_ne!(said, refused(&half_a_data_file(place.path())));
    }

    #[test]
    fn test_a_data_file_with_nothing_in_it_says_so_rather_than_importing_nothing_in_silence() {
        // An Outlook file somebody made and never used. An import of nothing
        // that says nothing looks exactly like a broken import, and the
        // difference is the whole of what somebody needs in order to decide
        // what to do next.
        let nothing_in_it = WhatWasFound::of(Path::new("empty.pst"), HowMuchToAllow::default());

        let said = nothing_in_it
            .nothing_at_all()
            .err()
            .map(|why| why.to_string())
            .expect("a file with nothing in it is refused");

        assert!(said.contains("There is nothing in"), "{said}");
        assert!(said.contains("Choose"), "{said}");
    }

    #[test]
    fn test_a_file_whose_every_folder_was_unreadable_is_damaged_rather_than_empty() {
        // Telling somebody to choose a different file when the one they chose
        // is the right file and damaged sends them looking in the wrong place.
        let mut all_broken = WhatWasFound::of(Path::new("broken.pst"), HowMuchToAllow::default());
        all_broken.one_could_not_be_read("a folder in broken.pst");

        let said = all_broken
            .nothing_at_all()
            .err()
            .map(|why| why.to_string())
            .expect("a file nothing could be read from is refused");

        assert!(said.contains("stops partway through"), "{said}");
    }

    #[test]
    fn test_each_thing_that_can_go_wrong_is_told_apart_from_the_others() {
        // Four situations, and each of them asks something different of the
        // person in front of the screen: find another file, fetch another copy
        // of this one, choose a file with something in it, import a few folders
        // at a time. One sentence covering all of them tells them none of that.
        let place = a_place_to_work();
        let mut all_broken = WhatWasFound::of(Path::new("broken.pst"), HowMuchToAllow::default());
        all_broken.one_could_not_be_read("a folder in broken.pst");

        let said = [
            refused(&not_a_data_file(place.path())),
            refused(&half_a_data_file(place.path())),
            refused(&a_copy_of_a_mailbox(place.path())),
            WhatWasFound::of(Path::new("empty.pst"), HowMuchToAllow::default())
                .nothing_at_all()
                .err()
                .map(|why| why.to_string())
                .expect("a file with nothing in it is refused"),
            holds_more_folders_than_this_program_will_open(Path::new("huge.pst"), 3).to_string(),
        ];

        for (which, one) in said.iter().enumerate() {
            assert!(one.ends_with('.'), "not a sentence: {one}");
            for other in said.iter().skip(which + 1) {
                assert_ne!(one, other, "two of these say the same thing");
            }
        }
    }

    #[test]
    fn test_a_file_listing_more_folders_than_this_program_will_open_is_refused() {
        // A small file can say it holds a million folders. Every one of them is
        // a name to hold, a read to make and a row in whatever comes next, so a
        // file nobody could have made is turned away rather than worked through.
        let mut found = WhatWasFound::of(
            Path::new("huge.pst"),
            HowMuchToAllow {
                most_folders_in_it: 2,
                ..HowMuchToAllow::default()
            },
        );

        assert!(found.one_more_folder("Work".to_string(), 1, 0).is_ok());
        assert!(found.one_more_folder("Home".to_string(), 2, 0).is_ok());
        let refused = found
            .one_more_folder("Family".to_string(), 3, 0)
            .err()
            .map(|why| why.to_string())
            .expect("a file listing more folders than it may is refused");

        assert!(refused.contains("more than 2 folders"), "{refused}");
    }

    #[test]
    fn test_one_item_larger_than_will_be_read_is_refused_rather_than_cut_short() {
        // Refused rather than handed over half read. A message given to the
        // import with its body missing is imported without complaint and nobody
        // finds out until they go looking for what it said.
        let mut come_out = HowMuchHasComeOut::of(HowMuchToAllow {
            most_one_item_comes_to: 100,
            ..HowMuchToAllow::default()
        });

        let refused = come_out
            .and_now(Path::new("mail.pst"), "Work", 101)
            .err()
            .map(|why| why.to_string())
            .expect("an item past the limit is refused");

        assert!(refused.contains("Work"), "{refused}");
        assert!(refused.contains("more than"), "{refused}");
    }

    #[test]
    fn test_the_whole_file_stops_at_the_total_it_is_allowed_to_come_to() {
        // The limit above is about one item, and a thousand items each just
        // under it add up to something no disk holds. This is the count that
        // has to be kept across the whole of an import rather than reset at
        // every item.
        let mut come_out = HowMuchHasComeOut::of(HowMuchToAllow {
            most_the_whole_of_it_comes_to: 250,
            ..HowMuchToAllow::default()
        });

        assert!(come_out.and_now(Path::new("mail.pst"), "Work", 100).is_ok());
        assert!(come_out.and_now(Path::new("mail.pst"), "Work", 100).is_ok());
        let refused = come_out
            .and_now(Path::new("mail.pst"), "Work", 100)
            .err()
            .map(|why| why.to_string())
            .expect("a file past its total is refused");

        assert!(refused.contains("more than"), "{refused}");
        assert!(refused.contains("was imported before"), "{refused}");
    }

    #[test]
    fn test_what_really_came_out_is_counted_rather_than_what_the_file_said() {
        // Every size a data file states about itself was written by whoever
        // built it, so nothing here believes one. What is counted is what
        // really came out, which is the only number a hostile file cannot
        // choose.
        let mut come_out = HowMuchHasComeOut::of(HowMuchToAllow {
            most_one_item_comes_to: 1000,
            most_the_whole_of_it_comes_to: 1000,
            ..HowMuchToAllow::default()
        });

        assert!(come_out.and_now(Path::new("a.pst"), "Work", 600).is_ok());
        assert!(
            come_out.and_now(Path::new("a.pst"), "Work", 600).is_err(),
            "the running total was not counted from what really came out"
        );
    }

    #[test]
    fn test_the_limits_this_program_ships_with_are_the_ones_written_down() {
        // The tests above hand this small numbers, because a limit nothing has
        // ever reached is a limit nobody has watched work. What they cannot say
        // is whether the numbers really shipped are the ones somebody chose, so
        // that is pinned here: generous for the largest mailbox anybody has
        // kept, and nowhere near what a hostile file asks for.
        let shipped = HowMuchToAllow::default();

        assert_eq!(shipped.most_folders_in_it, 10_000);
        assert_eq!(shipped.most_items_in_one_folder, 200_000);
        assert_eq!(shipped.most_one_item_comes_to, 256 * 1024 * 1024);
        assert_eq!(
            shipped.most_the_whole_of_it_comes_to,
            20 * 1024 * 1024 * 1024
        );
    }

    // ── What is in one item ─────────────────────────────────────────────────

    /// One item's properties, built by hand, so everything that turns an item
    /// into something this program keeps can be tested without a data file.
    fn an_item(said: &[(u16, WhatItSaid)]) -> WhatTheItemSaid {
        WhatTheItemSaid {
            said: said.iter().cloned().collect(),
        }
    }

    /// What one data file numbered the names it uses, built by hand.
    fn names_numbered(here: &[((WhichSet, u32), u16)]) -> WhatTheNamesAreHere {
        WhatTheNamesAreHere {
            numbered: here.iter().copied().collect(),
        }
    }

    /// Some words, for a property that holds text.
    fn words(text: &str) -> WhatItSaid {
        WhatItSaid::Words(text.to_string())
    }

    /// The way Outlook counts to a moment: hundred-nanosecond steps from the
    /// first of January 1601.
    fn as_outlook_counts(rfc3339: &str) -> i64 {
        let when = chrono::DateTime::parse_from_rfc3339(rfc3339).expect("a moment a test named");
        (when.timestamp() + 11_644_473_600) * 10_000_000
    }

    /// The account an import is going into, for tests that need one.
    fn going_to() -> WhereItIsGoing<'static> {
        WhereItIsGoing { account: "acct-1" }
    }

    #[test]
    fn test_a_time_in_an_outlook_file_is_read_as_the_instant_it_names() {
        // Getting this wrong moves somebody's appointments. Outlook counts
        // hundred-nanosecond steps from the first of January 1601 and the count
        // is universal time, never anybody's local clock, so there is one right
        // answer and it does not depend on where the file is opened.
        //
        // The first of January 1970 is the one value everybody has written
        // down, so it is pinned here rather than worked out from the same
        // arithmetic the code uses.
        assert_eq!(
            the_moment_of(116_444_736_000_000_000).map(|when| when.to_rfc3339()),
            Some("1970-01-01T00:00:00+00:00".to_string())
        );
        assert_eq!(
            the_moment_of(as_outlook_counts("2026-08-24T10:00:00Z")).map(|when| when.to_rfc3339()),
            Some("2026-08-24T10:00:00+00:00".to_string())
        );
    }

    #[test]
    fn test_the_dates_outlook_writes_to_mean_no_date_are_read_as_no_date() {
        // Outlook leaves a date unset by writing nothing, and marks a task with
        // no due date by writing a day in the year 4501. Read as dates, the
        // first files everything in 1601 and the second in 4501, and both are
        // wrong in a way somebody only notices when they sort by date.
        assert_eq!(the_moment_of(0), None);
        assert_eq!(
            the_moment_of(as_outlook_counts("4501-01-01T00:00:00Z")),
            None
        );
        assert_eq!(the_moment_of(-1), None);
    }

    #[test]
    fn test_a_whole_day_is_the_day_it_was_written_in_wherever_that_was() {
        // An all-day appointment is midnight to midnight where the person was,
        // and Outlook writes it as the universal time that midnight fell at. So
        // an all-day event written in Berlin is stored at eleven the evening
        // before, and taking the day off that instant puts somebody's birthday
        // on the wrong date.
        let in_berlin = as_outlook_counts("2026-08-23T23:00:00Z");
        let in_new_york = as_outlook_counts("2026-08-24T05:00:00Z");
        let at_greenwich = as_outlook_counts("2026-08-24T00:00:00Z");

        assert_eq!(said_as_a_day(in_berlin).as_deref(), Some("2026-08-24"));
        assert_eq!(said_as_a_day(in_new_york).as_deref(), Some("2026-08-24"));
        assert_eq!(said_as_a_day(at_greenwich).as_deref(), Some("2026-08-24"));
    }

    #[test]
    fn test_what_an_item_is_comes_from_what_outlook_calls_it() {
        // The one decision everything else hangs off. Outlook writes what each
        // thing is on the thing itself, and a file holds all five kinds mixed
        // together in the same folders.
        for (called, expected) in [
            ("IPM.Note", Some(WhatKind::Mail)),
            ("IPM.Note.SMIME", Some(WhatKind::Mail)),
            ("ipm.note", Some(WhatKind::Mail)),
            ("IPM.Appointment", Some(WhatKind::Appointment)),
            ("IPM.Contact", Some(WhatKind::Contact)),
            ("IPM.DistList", Some(WhatKind::Contact)),
            ("IPM.Task", Some(WhatKind::Task)),
            ("IPM.StickyNote", Some(WhatKind::Note)),
            ("IPM.Schedule.Meeting.Request", None),
            ("REPORT.IPM.Note.NDR", None),
            ("", None),
        ] {
            assert_eq!(the_kind_of(called), expected, "for {called:?}");
        }
    }

    #[test]
    fn test_a_subject_keeps_its_words_and_loses_the_mark_outlook_puts_in_front_of_it() {
        // Outlook writes a subject with two bytes in front of it saying where
        // the "Re:" ends. Handed over as it stands, every reply in somebody's
        // mailbox reads aloud starting with a character that is not a
        // character.
        assert_eq!(
            without_the_subject_marker("\u{1}\u{4}Re: Dinner"),
            "Re: Dinner"
        );
        assert_eq!(without_the_subject_marker("Dinner"), "Dinner");
        assert_eq!(without_the_subject_marker(""), "");
        assert_eq!(without_the_subject_marker("\u{1}"), "\u{1}");
    }

    #[test]
    fn test_text_from_an_older_file_is_read_in_the_alphabet_it_was_written_in() {
        // The older kind of Outlook file holds its text in whatever alphabet
        // the computer that wrote it was set to, and says which beside it. Read
        // as though every byte were a letter, a German subject comes out with a
        // replacement character where each accent was.
        assert_eq!(text_in(b"Sch\xf6nen Gru\xdf", Some(1252)), "Schönen Gruß");
        assert_eq!(text_in(b"\x80 5", Some(1252)), "€ 5");
        assert_eq!(text_in(b"plain words", None), "plain words");
        assert_eq!(text_in(b"", Some(1252)), "");
    }

    // ── Turning one item into what this program keeps ───────────────────────

    #[test]
    fn test_mail_comes_out_as_a_message_this_program_can_read_back() {
        // The whole promise of the mail half of this module. What comes out is
        // a message, so everything this program already does with a message
        // works on it without knowing where it came from.
        let said = an_item(&[
            (SUBJECT, words("\u{1}\u{4}Re: The engine")),
            (BODY, words("The engine weaves algebraic patterns.")),
            (SENDER_NAME, words("Ada Lovelace")),
            (SENDER_SMTP_ADDRESS, words("ada@example.com")),
            (INTERNET_MESSAGE_ID, words("<one@example.com>")),
            (
                SENT_AT,
                WhatItSaid::When(as_outlook_counts("2026-08-24T10:00:00Z")),
            ),
        ]);
        let names = WhatTheNamesAreHere::default();
        let went_to = WhoItWentTo {
            to: vec![EmailAddress::new(
                "charles@example.com".to_string(),
                Some("Charles Babbage".to_string()),
            )],
            cc: Vec::new(),
        };

        let raw = mail_written_from(&TheItem::of(&said, &names), &went_to);
        let read_back = crate::service::mime::parse(&raw).expect("what came out is a message");

        assert_eq!(read_back.subject, "Re: The engine");
        assert_eq!(
            read_back.from,
            vec![EmailAddress::new(
                "ada@example.com".to_string(),
                Some("Ada Lovelace".to_string())
            )]
        );
        assert_eq!(
            read_back.to,
            vec![EmailAddress::new(
                "charles@example.com".to_string(),
                Some("Charles Babbage".to_string())
            )]
        );
        assert_eq!(
            read_back.body_plain.as_deref(),
            Some("The engine weaves algebraic patterns.")
        );
        assert_eq!(read_back.message_id.as_deref(), Some("one@example.com"));
        // Universal time spelled with a Z rather than with an offset of
        // nothing, which is how the reader writes it back after the round
        // trip. Both are the same instant and this program reads both.
        assert_eq!(read_back.date.as_deref(), Some("2026-08-24T10:00:00Z"));
    }

    #[test]
    fn test_mail_written_inside_one_company_still_says_who_sent_it() {
        // Mail that never left a company carries an address only that company's
        // own server understands, and it is not an email address at all. It is
        // kept exactly as Outlook wrote it, and that is a decision rather than
        // an oversight.
        //
        // What reads a message back drops a sender with no address on it, so a
        // message written with the name alone reads aloud as being from nobody.
        // A whole Sent folder of those is worse than a sender whose address
        // cannot be replied to, and replying fails at once and visibly where
        // the missing name is silent. Nothing is invented to fill the gap,
        // because an invented address is one somebody else may really have.
        let said = an_item(&[
            (SUBJECT, words("The quarterly figures")),
            (BODY, words("Attached.")),
            (SENDER_NAME, words("Ada Lovelace")),
            (
                SENDER_ADDRESS,
                words("/O=ENGINE/OU=EXCHANGE/CN=RECIPIENTS/CN=ADA"),
            ),
            (SENDER_ADDRESS_KIND, words("EX")),
        ]);
        let names = WhatTheNamesAreHere::default();

        let raw = mail_written_from(&TheItem::of(&said, &names), &WhoItWentTo::default());
        let read_back = crate::service::mime::parse(&raw).expect("what came out is a message");

        assert_eq!(read_back.from.len(), 1, "{:?}", read_back.from);
        assert_eq!(
            read_back.from[0].name.as_deref(),
            Some("Ada Lovelace"),
            "the sender's name was lost with their address"
        );
        assert_eq!(
            read_back.from[0].address, "/O=ENGINE/OU=EXCHANGE/CN=RECIPIENTS/CN=ADA",
            "the address Outlook recorded was changed into something else"
        );
    }

    #[test]
    fn test_a_sender_the_file_named_and_gave_no_address_is_still_heard() {
        // Some messages in a data file carry a name for their sender and no
        // address of any kind. What reads a message back drops a sender with
        // nothing in its address, so writing the name only as a display name
        // loses it altogether and the message reads aloud as being from nobody.
        //
        // Both of the other ways were tried and this one is what is left.
        // `From: Ada Lovelace <>` is dropped on the way back in, and so is
        // `From: Ada Lovelace`, which is read as a name with no address.
        let said = an_item(&[
            (SUBJECT, words("The plates")),
            (BODY, words("They arrived.")),
            (SENDER_NAME, words("Ada Lovelace")),
        ]);

        let raw = mail_written_from(
            &TheItem::of(&said, &WhatTheNamesAreHere::default()),
            &WhoItWentTo::default(),
        );
        let read_back = crate::service::mime::parse(&raw).expect("what came out is a message");

        assert_eq!(read_back.from.len(), 1, "{:?}", read_back.from);
        let heard = format!(
            "{} {}",
            read_back.from[0].address,
            read_back.from[0].name.as_deref().unwrap_or_default()
        );
        assert!(
            heard.contains("Ada Lovelace"),
            "the only thing the file said about the sender was lost: {heard:?}"
        );
    }

    #[test]
    fn test_mail_with_no_sender_at_all_is_still_a_message() {
        // A draft nobody had addressed, a message whose sender the file lost.
        // Writing an empty sender onto it would be a header saying the message
        // came from nobody in particular, which is not the same as saying
        // nothing.
        let said = an_item(&[(SUBJECT, words("A draft")), (BODY, words("Half written."))]);

        let raw = mail_written_from(
            &TheItem::of(&said, &WhatTheNamesAreHere::default()),
            &WhoItWentTo::default(),
        );
        let read_back = crate::service::mime::parse(&raw).expect("what came out is a message");

        assert!(read_back.from.is_empty(), "{:?}", read_back.from);
        assert_eq!(read_back.subject, "A draft");
        assert_eq!(read_back.body_plain.as_deref(), Some("Half written."));
    }

    #[test]
    fn test_an_appointment_comes_out_as_an_event_at_the_hours_it_was_kept_at() {
        // An appointment written in one place and read in another has to mean
        // the same instant, so what is stored is the instant with its offset on
        // it rather than a clock face and the name of a zone this program would
        // have to map from Outlook's names to everybody else's.
        let names = names_numbered(&[
            ((WhichSet::Appointment, APPOINTMENT_STARTS), 0x8001),
            ((WhichSet::Appointment, APPOINTMENT_ENDS), 0x8002),
            ((WhichSet::Appointment, APPOINTMENT_LOCATION), 0x8003),
            ((WhichSet::Appointment, APPOINTMENT_BUSY_STATUS), 0x8004),
        ]);
        let said = an_item(&[
            (SUBJECT, words("Analytical engine review")),
            (BODY, words("Bring the cards.")),
            (
                0x8001,
                WhatItSaid::When(as_outlook_counts("2026-08-24T09:00:00Z")),
            ),
            (
                0x8002,
                WhatItSaid::When(as_outlook_counts("2026-08-24T10:30:00Z")),
            ),
            (0x8003, words("The drawing room")),
            (0x8004, WhatItSaid::Whole(2)),
            (
                CREATED,
                WhatItSaid::When(as_outlook_counts("2026-01-05T08:00:00Z")),
            ),
        ]);

        let event = an_appointment_from(&TheItem::of(&said, &names), going_to());

        assert_eq!(event.summary, "Analytical engine review");
        assert_eq!(event.description.as_deref(), Some("Bring the cards."));
        assert_eq!(event.location.as_deref(), Some("The drawing room"));
        assert_eq!(event.start_datetime, "2026-08-24T09:00:00+00:00");
        assert_eq!(event.end_datetime, "2026-08-24T10:30:00+00:00");
        assert!(!event.is_all_day);
        assert_eq!(event.start_date, None);
        assert_eq!(event.show_as, "busy");
        assert_eq!(event.status, "confirmed");
        assert_eq!(event.account_id, "acct-1");
        assert_eq!(event.created_at, "2026-01-05T08:00:00+00:00");
        // It came out of a file rather than off a server, so nothing may take
        // it for a copy of something a provider owns and sync it away again.
        assert_eq!(event.source_provider, None);
        assert_eq!(event.provider_event_id, None);
        assert!(!event.id.is_empty());
    }

    #[test]
    fn test_an_appointment_in_a_file_that_numbered_no_names_still_has_its_hours() {
        // A file whose list of its own names could not be read. Everything that
        // needed a name comes out missing rather than wrong, but the hours are
        // the one thing an appointment cannot do without: Outlook writes them at
        // a second number every one of its files agrees on, and that is what is
        // read here. Anything else would put the appointment at an hour nobody
        // chose, which is the failure this whole module is careful about.
        let said = an_item(&[
            (SUBJECT, words("Engine review")),
            (
                APPOINTMENT_STARTS_PLAINLY,
                WhatItSaid::When(as_outlook_counts("2026-08-24T09:00:00Z")),
            ),
            (
                APPOINTMENT_ENDS_PLAINLY,
                WhatItSaid::When(as_outlook_counts("2026-08-24T10:30:00Z")),
            ),
        ]);

        let event = an_appointment_from(
            &TheItem::of(&said, &WhatTheNamesAreHere::default()),
            going_to(),
        );

        assert_eq!(event.start_datetime, "2026-08-24T09:00:00+00:00");
        assert_eq!(event.end_datetime, "2026-08-24T10:30:00+00:00");
        assert_eq!(event.summary, "Engine review");
        // The name-numbered fields are missing rather than filled from
        // somewhere they do not belong.
        assert_eq!(event.location, None);
        assert!(!event.is_all_day);
    }

    #[test]
    fn test_an_all_day_appointment_comes_out_as_a_day_rather_than_an_hour() {
        // Somebody's holiday is a day, not a stretch of hours, and this program
        // keeps the two apart. Stored as hours it reads aloud as "midnight to
        // midnight" and moves by an hour twice a year.
        let names = names_numbered(&[
            ((WhichSet::Appointment, APPOINTMENT_STARTS), 0x8001),
            ((WhichSet::Appointment, APPOINTMENT_ENDS), 0x8002),
            ((WhichSet::Appointment, APPOINTMENT_IS_ALL_DAY), 0x8005),
        ]);
        let said = an_item(&[
            (SUBJECT, words("Away")),
            (
                0x8001,
                WhatItSaid::When(as_outlook_counts("2026-08-23T22:00:00Z")),
            ),
            (
                0x8002,
                WhatItSaid::When(as_outlook_counts("2026-08-25T22:00:00Z")),
            ),
            (0x8005, WhatItSaid::YesOrNo(true)),
        ]);

        let event = an_appointment_from(&TheItem::of(&said, &names), going_to());

        assert!(event.is_all_day);
        assert_eq!(event.start_date.as_deref(), Some("2026-08-24"));
        assert_eq!(event.end_date.as_deref(), Some("2026-08-26"));
    }

    #[test]
    fn test_a_contact_comes_out_with_the_names_numbers_and_addresses_it_had() {
        // An address book is the part of an old mail program somebody misses
        // first, and a contact with only a display name on it is a row that
        // reads aloud as a name and cannot be written to.
        let names = names_numbered(&[((WhichSet::Address, FIRST_EMAIL_ADDRESS), 0x8083)]);
        let said = an_item(&[
            (DISPLAY_NAME, words("Ada Lovelace")),
            (GIVEN_NAME, words("Ada")),
            (SURNAME, words("Lovelace")),
            (COMPANY, words("The Analytical Society")),
            (JOB_TITLE, words("Mathematician")),
            (0x8083, words("ada@example.com")),
            (MOBILE_PHONE, words("+44 20 7946 0000")),
            (BUSINESS_PHONE, words("+44 20 7946 0001")),
            (HOME_STREET, words("12 Engine Row")),
            (HOME_CITY, words("London")),
            (
                BIRTHDAY,
                WhatItSaid::When(as_outlook_counts("1815-12-10T00:00:00Z")),
            ),
            (BODY, words("Met at the exhibition.")),
        ]);

        let contact = a_contact_from(&TheItem::of(&said, &names), going_to());

        assert_eq!(contact.name, "Ada Lovelace");
        assert_eq!(contact.given_name.as_deref(), Some("Ada"));
        assert_eq!(contact.family_name.as_deref(), Some("Lovelace"));
        assert_eq!(contact.email, "ada@example.com");
        assert_eq!(contact.company.as_deref(), Some("The Analytical Society"));
        assert_eq!(contact.job_title.as_deref(), Some("Mathematician"));
        assert_eq!(contact.birthday.as_deref(), Some("1815-12-10"));
        assert_eq!(contact.notes.as_deref(), Some("Met at the exhibition."));
        assert_eq!(contact.phone.as_deref(), Some("+44 20 7946 0001"));
        assert_eq!(contact.account_id, "acct-1");
        assert_eq!(contact.source_provider, None);
        assert!(contact.known_to.is_empty());

        // The numbers and addresses this program keeps several of are kept as
        // several rather than flattened onto the one field a row shows.
        let numbers: Vec<PhoneEntry> =
            serde_json::from_str(contact.phones_json.as_deref().unwrap_or("[]"))
                .expect("the numbers come back out");
        assert_eq!(numbers.len(), 2, "{numbers:?}");
        assert!(
            numbers.iter().any(|one| one.label == "Mobile"),
            "{numbers:?}"
        );
        let where_they_live: Vec<AddressEntry> =
            serde_json::from_str(contact.addresses_json.as_deref().unwrap_or("[]"))
                .expect("the addresses come back out");
        assert_eq!(where_they_live.len(), 1, "{where_they_live:?}");
        assert_eq!(where_they_live[0].city, "London");
    }

    #[test]
    fn test_a_contact_from_inside_one_company_keeps_everything_but_an_unusable_address() {
        // The other side of the decision made for a message's sender, and it
        // goes the other way on purpose. A message keeps an address only one
        // company's server understood, because dropping it drops the sender's
        // name with it. A contact's name is its own field, so what would be
        // left is an address the compose window takes and nothing delivers to.
        let names = names_numbered(&[((WhichSet::Address, FIRST_EMAIL_ADDRESS), 0x8083)]);
        let said = an_item(&[
            (DISPLAY_NAME, words("Ada Lovelace")),
            (COMPANY, words("The Analytical Society")),
            (BUSINESS_PHONE, words("+44 20 7946 0001")),
            (0x8083, words("/O=ENGINE/OU=EXCHANGE/CN=RECIPIENTS/CN=ADA")),
        ]);

        let contact = a_contact_from(&TheItem::of(&said, &names), going_to());

        assert_eq!(contact.email, "");
        assert_eq!(contact.emails_json, None);
        assert_eq!(contact.name, "Ada Lovelace");
        assert_eq!(contact.company.as_deref(), Some("The Analytical Society"));
        assert_eq!(contact.phone.as_deref(), Some("+44 20 7946 0001"));
    }

    #[test]
    fn test_a_task_comes_out_with_its_due_date_and_whether_it_is_done() {
        // A task list where everything reads as still to do, or where the due
        // dates are all a day out, is a task list nobody trusts twice.
        let names = names_numbered(&[
            ((WhichSet::Task, TASK_DUE_DATE), 0x8105),
            ((WhichSet::Task, TASK_IS_DONE), 0x811C),
            ((WhichSet::Task, TASK_DONE_AT), 0x810F),
        ]);
        let said = an_item(&[
            (SUBJECT, words("Finish the notes")),
            (BODY, words("Section G especially.")),
            (
                0x8105,
                WhatItSaid::When(as_outlook_counts("2026-09-01T00:00:00Z")),
            ),
            (0x811C, WhatItSaid::YesOrNo(true)),
            (
                0x810F,
                WhatItSaid::When(as_outlook_counts("2026-08-30T14:00:00Z")),
            ),
            (IMPORTANCE, WhatItSaid::Whole(2)),
        ]);

        let task = a_task_from(&TheItem::of(&said, &names), going_to());

        assert_eq!(task.title, "Finish the notes");
        assert_eq!(task.description.as_deref(), Some("Section G especially."));
        assert_eq!(task.due_date.as_deref(), Some("2026-09-01"));
        assert!(task.is_completed);
        assert_eq!(
            task.completed_at.as_deref(),
            Some("2026-08-30T14:00:00+00:00")
        );
        assert_eq!(task.priority, "high");
        assert_eq!(task.account_id, "acct-1");
        // It came out of a file, so nothing may take it for a copy of a task a
        // provider owns.
        assert_eq!(task.remote_status, None);
        assert!(!task.pending);
    }

    #[test]
    fn test_a_note_comes_out_with_its_words_and_when_it_was_written() {
        // A note is the smallest of the five and the easiest to lose: its whole
        // content is a body with no headers on it.
        let said = an_item(&[
            (SUBJECT, words("Bernoulli numbers")),
            (BODY, words("The seventh is the one to check.")),
            (
                CREATED,
                WhatItSaid::When(as_outlook_counts("2026-02-11T09:15:00Z")),
            ),
            (
                LAST_CHANGED,
                WhatItSaid::When(as_outlook_counts("2026-02-12T16:40:00Z")),
            ),
        ]);

        let note = a_note_from(
            &TheItem::of(&said, &WhatTheNamesAreHere::default()),
            going_to(),
        );

        assert_eq!(note.title, "Bernoulli numbers");
        assert_eq!(note.body, "The seventh is the one to check.");
        assert_eq!(note.format, "plain");
        assert_eq!(note.created_at, "2026-02-11T09:15:00+00:00");
        assert_eq!(note.updated_at, "2026-02-12T16:40:00+00:00");
        assert_eq!(note.account_id, "acct-1");
        assert!(!note.pinned);
    }

    #[test]
    fn test_a_note_with_no_subject_is_still_called_something() {
        // Outlook does not ask for a title on a note, so most of them have
        // none. A list of rows that all read aloud as nothing is a list nobody
        // can move through, so the first line of the note becomes its name.
        let said = an_item(&[(
            BODY,
            words("Ring the printer about the plates.\r\nThey have the wrong dates."),
        )]);

        let note = a_note_from(
            &TheItem::of(&said, &WhatTheNamesAreHere::default()),
            going_to(),
        );

        assert_eq!(note.title, "Ring the printer about the plates.");
    }

    #[test]
    fn test_every_date_written_out_is_one_this_programs_own_reader_reads_back() {
        // The other end of the promise. A date this module stores that the one
        // reader of stored dates cannot read is a date whatever shows it says
        // out loud character by character, and one nothing can sort by.
        use crate::common::moment::{Moment, read};

        let names = names_numbered(&[
            ((WhichSet::Appointment, APPOINTMENT_STARTS), 0x8001),
            ((WhichSet::Appointment, APPOINTMENT_ENDS), 0x8002),
            ((WhichSet::Appointment, APPOINTMENT_IS_ALL_DAY), 0x8005),
            ((WhichSet::Task, TASK_DUE_DATE), 0x8105),
        ]);
        let at_an_hour = an_item(&[
            (
                0x8001,
                WhatItSaid::When(as_outlook_counts("2026-08-24T09:00:00Z")),
            ),
            (
                0x8002,
                WhatItSaid::When(as_outlook_counts("2026-08-24T10:00:00Z")),
            ),
            (
                CREATED,
                WhatItSaid::When(as_outlook_counts("2026-01-05T08:00:00Z")),
            ),
        ]);
        let all_day = an_item(&[
            (
                0x8001,
                WhatItSaid::When(as_outlook_counts("2026-08-23T22:00:00Z")),
            ),
            (
                0x8002,
                WhatItSaid::When(as_outlook_counts("2026-08-24T22:00:00Z")),
            ),
            (0x8005, WhatItSaid::YesOrNo(true)),
        ]);
        let a_task = an_item(&[(
            0x8105,
            WhatItSaid::When(as_outlook_counts("2026-09-01T00:00:00Z")),
        )]);

        let hours = an_appointment_from(&TheItem::of(&at_an_hour, &names), going_to());
        let day = an_appointment_from(&TheItem::of(&all_day, &names), going_to());
        let task = a_task_from(&TheItem::of(&a_task, &names), going_to());

        // An hour is an instant with its offset on it, so nothing has to guess
        // which zone it meant.
        assert!(
            matches!(read(&hours.start_datetime), Some(Moment::Fixed(_))),
            "{}",
            hours.start_datetime
        );
        assert!(matches!(read(&hours.end_datetime), Some(Moment::Fixed(_))));
        assert!(matches!(read(&hours.created_at), Some(Moment::Fixed(_))));
        // A whole day is a day, with nothing said about the time of day.
        assert!(
            matches!(
                read(day.start_date.as_deref().unwrap_or_default()),
                Some(Moment::WholeDay(_))
            ),
            "{:?}",
            day.start_date
        );
        assert!(matches!(
            read(task.due_date.as_deref().unwrap_or_default()),
            Some(Moment::WholeDay(_))
        ));
    }

    #[test]
    fn test_only_what_this_program_uses_is_ever_read_off_a_thing_in_the_file() {
        // The first half of what keeps a large file from filling this computer.
        // The largest things in a data file are the older markup Outlook keeps
        // beside the words and the files carried on a message, and neither is
        // ever asked for, so neither is ever read.
        let numbered_here = names_numbered(&[
            ((WhichSet::Appointment, APPOINTMENT_STARTS), 0x8001),
            ((WhichSet::Task, TASK_DUE_DATE), 0x8105),
        ]);

        let worth = what_is_worth_reading(&numbered_here);

        assert!(worth.contains(&SUBJECT));
        assert!(worth.contains(&BODY));
        // The names this file numbered for itself are asked for at the numbers
        // this file gave them, not at the numbers any other file gave them.
        assert!(worth.contains(&0x8001), "{worth:?}");
        assert!(worth.contains(&0x8105), "{worth:?}");
        // The older markup, which Outlook keeps beside the words and which is
        // several times their size.
        assert!(!worth.contains(&0x1009), "{worth:?}");

        // A file that never said what it numbered its own names asks for the
        // fixed numbers and nothing else, rather than for numbers it guessed.
        let nothing_numbered = what_is_worth_reading(&WhatTheNamesAreHere::default());
        assert!(
            nothing_numbered.iter().all(|at| *at < 0x8000),
            "{nothing_numbered:?}"
        );
    }

    #[test]
    fn test_what_a_thing_came_to_is_what_it_really_holds() {
        // The measure the limits are applied to, and the same one for all five
        // kinds, so no kind can grow past what the others are held to. What is
        // counted is what is really held rather than anything the file said.
        let nothing = an_item(&[]);
        let some_words = an_item(&[(BODY, words("12345678901234567890"))]);
        let some_bytes = an_item(&[(HTML_BODY, WhatItSaid::Bytes(vec![0u8; 500]))]);
        let one_number = an_item(&[(IMPORTANCE, WhatItSaid::Whole(2))]);

        assert_eq!(how_much_it_came_to(&nothing), 0);
        assert_eq!(how_much_it_came_to(&some_words), 20);
        assert_eq!(how_much_it_came_to(&some_bytes), 500);
        // A thing made only of numbers is not a thing of no size.
        assert_eq!(how_much_it_came_to(&one_number), 8);
    }

    #[test]
    fn test_what_the_file_holds_crosses_into_this_programs_own_values() {
        // The one place the reader's own values are turned into this
        // program's, and the reason everything after it can be tested at all.
        // The kinds nothing here reads come back as nothing rather than as an
        // empty one of something else, so a property this program does not
        // understand is never mistaken for one it does.
        use outlook_pst::ltp::prop_context::{BinaryValue, PropertyValue};

        assert_eq!(
            what_one_property_said(&PropertyValue::Integer32(2), None),
            Some(WhatItSaid::Whole(2))
        );
        assert_eq!(
            what_one_property_said(&PropertyValue::Boolean(true), None),
            Some(WhatItSaid::YesOrNo(true))
        );
        assert_eq!(
            what_one_property_said(&PropertyValue::Time(12345), None),
            Some(WhatItSaid::When(12345))
        );
        assert_eq!(
            what_one_property_said(
                &PropertyValue::Binary(BinaryValue::new(vec![1, 2, 3])),
                None
            ),
            Some(WhatItSaid::Bytes(vec![1, 2, 3]))
        );
        assert_eq!(what_one_property_said(&PropertyValue::Null, None), None);
        assert_eq!(
            what_one_property_said(&PropertyValue::Floating64(1.5), None),
            None
        );
    }

    #[test]
    fn test_only_outlooks_own_sets_of_names_are_recognised() {
        // Anything that names its properties can put its own set of names in a
        // data file, and a set some other program wrote uses the same numbers
        // for entirely different things. Read as Outlook's, a plug-in's own
        // field becomes the hour somebody's appointment starts at.
        use outlook_pst::ltp::prop_context::GuidValue;

        let outlooks_appointments = GuidValue::new(0x0006_2002, 0, 0, HOW_OUTLOOKS_OWN_SETS_END);
        let somebody_elses = GuidValue::new(0x0006_2002, 0, 0, [0; 8]);
        let one_nobody_here_knows = GuidValue::new(0x1234_5678, 0, 0, HOW_OUTLOOKS_OWN_SETS_END);

        assert_eq!(
            which_set_is(&outlooks_appointments),
            Some(WhichSet::Appointment)
        );
        assert_eq!(which_set_is(&somebody_elses), None);
        assert_eq!(which_set_is(&one_nobody_here_knows), None);
    }

    /// A folder in a data file, built by hand.
    fn a_folder(named: &str, which: u32) -> FolderInTheDataFile {
        FolderInTheDataFile {
            named: named.to_string(),
            how_many_things_in_it: 0,
            which,
        }
    }

    /// The places in a list of folders, in the order their names sort.
    fn in_name_order(folders: &[FolderInTheDataFile]) -> Vec<usize> {
        let mut order: Vec<usize> = (0..folders.len()).collect();
        order.sort_by(|one, other| folders[*one].named.cmp(&folders[*other].named));
        order
    }

    #[test]
    fn test_two_folders_of_one_name_are_read_as_the_one_folder_they_will_become() {
        // Outlook lets two folders sitting in the same place carry one name, and
        // this program's own placer files two of one name together anyway.
        // Reading whichever of them a search happened to land on would leave a
        // whole folder of somebody's mail in the file with nothing said about
        // it, which is the quietest way this module could lose anything.
        let folders = [
            a_folder("Work", 10),
            a_folder("Work/Invoices", 11),
            a_folder("Work/Invoices", 12),
            a_folder("Home", 13),
            a_folder("Work/Invoices", 14),
        ];
        let order = in_name_order(&folders);

        assert_eq!(
            every_folder_named(&folders, &order, "Work/Invoices"),
            vec![1, 2, 4],
            "one of three folders of a name was going to be read"
        );
        assert_eq!(every_folder_named(&folders, &order, "Home"), vec![3]);
    }

    #[test]
    fn test_a_folder_name_the_file_never_offered_finds_nothing_rather_than_something() {
        // A name is what a caller has to go on, and a name that came from
        // somewhere else must find nothing rather than whichever folder it
        // happened to sort next to.
        let folders = [a_folder("Work", 10), a_folder("Work/Invoices", 11)];
        let order = in_name_order(&folders);

        for asked in [
            "Work/Nothing",
            "work",
            "Work/",
            "../Work",
            "",
            "Work/Invoices/Deeper",
        ] {
            assert!(
                every_folder_named(&folders, &order, asked).is_empty(),
                "{asked:?} found a folder"
            );
        }
    }

    #[test]
    fn test_everything_this_module_says_is_a_sentence_and_names_no_machinery() {
        // All of it is read aloud. A fragment with no stop on the end runs into
        // whatever is spoken next, and a sentence naming a mechanism tells
        // somebody about this program's insides instead of about their mail.
        //
        // The few sentences that carry the reader's own words about a damaged
        // file are not here: those are that reader's, and quoting them is what
        // lets somebody helping find out what really went wrong.
        let at = Path::new("mailbox.pst");
        let everything = [
            is_not_an_outlook_data_file(at).to_string(),
            stops_partway_through(at).to_string(),
            is_a_copy_of_a_mailbox_on_a_server(at).to_string(),
            there_is_nothing_in(at).to_string(),
            holds_more_folders_than_this_program_will_open(at, 10_000).to_string(),
            one_folder_holds_more_than_this_program_will_open(at, "Work", 200_000).to_string(),
            one_item_comes_to_more_than_will_be_read(at, "Work", 256 * 1024 * 1024).to_string(),
            the_whole_of_it_comes_to_more_than_will_be_read(at, 20 * 1024 * 1024 * 1024)
                .to_string(),
        ];

        for said in &everything {
            assert!(said.ends_with('.'), "not a sentence: {said}");
            let lowered = said.to_lowercase();
            for machinery in [
                "property",
                "node",
                "b-tree",
                "heap",
                "parse",
                "code page",
                "codepage",
                "utf",
                "entry id",
                "buffer",
                // Not "byte", which is the plain word for a size and is how
                // megabytes and gigabytes are spelled.
            ] {
                assert!(
                    !lowered.contains(machinery),
                    "this names {machinery}, which is a mechanism and not what happens: {said}"
                );
            }
        }
    }

    #[test]
    fn test_a_size_is_said_in_a_unit_somebody_can_hear() {
        // These numbers go into a sentence that gets read out. Two hundred and
        // sixty eight million four hundred and thirty five thousand four
        // hundred and fifty six is a number nobody can hold, and it is 256
        // megabytes.
        assert_eq!(said_as_a_size(256 * 1024 * 1024), "256 megabytes");
        assert_eq!(said_as_a_size(20 * 1024 * 1024 * 1024), "20 gigabytes");
        assert_eq!(said_as_a_size(1000), "1000 bytes");
    }
}

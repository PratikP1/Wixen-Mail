//! Opening an archive of somebody's mailbox, and writing one.
//!
//! [`crate::application::import_tree`] decides where every entry of an archive
//! lands, and [`crate::application::export_tree`] decides what an export is
//! made of. Neither of them opens a file. This does, and it is the only part of
//! that work that touches the disk.
//!
//! Nothing here decides where any mail goes. What comes out is the archive's
//! own names and the archive's own bytes; where they land is somebody else's
//! answer.
//!
//! # One file or a folder, and the caller cannot tell which
//!
//! Somebody leaving another mail program arrives with the file that program
//! wrote or with the folder it wrote into, and which of the two it is has
//! nothing to do with where their mail belongs. So both open into the same
//! thing, and nothing above this ever asks. Anything that had to ask would have
//! to ask in the layer that decides, and that is the layer that is not allowed
//! to open a file at all.
//!
//! # A name is handed over exactly as the archive wrote it
//!
//! The name in an archive is the one thing in it that decides where bytes get
//! written, and it was written by somebody else. Nothing here repairs one. A
//! name with its climbing steps taken out would arrive at the placer looking
//! ordinary, and the placer would file a stranger's mail under a name they
//! chose, in silence. Refusing is [`crate::application::import_tree`]'s to do,
//! and it can only refuse what it really sees.
//!
//! # Memory, which is the whole shape of this module
//!
//! A mailbox somebody has kept for twenty years is the largest thing this
//! program opens. Looking an archive over reads only the opening of each entry,
//! because that is where the placer stops looking, so a plan for the whole
//! archive can be made on a computer that could not hold it. Filling a folder
//! then reads one entry and lets it go again, and writing an export takes one
//! message at a time, so the mail is in memory once on its way past rather than
//! twice at rest.
//!
//! An entry is read one of two ways, and the difference is what is held.
//! [`MailboxArchive::one_entry_read_in_pieces`] holds none of it: it hands over
//! somewhere to read from, a piece at a time, so the entry may be any size at
//! all. That is what a folder of twenty years of mail saved as one file needs,
//! and it is the way to read one.
//! [`MailboxArchive::one_entry_read_through`] hands back the whole entry as
//! bytes, which is simpler for a small one and is why
//! [`HowMuchToAllow::most_one_thing_unpacks_to`] exists: it is a bound on that
//! call rather than on what an archive may contain.
//!
//! # An archive is a stranger's file
//!
//! A file of a few kilobytes can say it unpacks to more than every disk ever
//! made, and reading it as far as it says fills this computer and stops the
//! program. Every size an archive states about itself was written by whoever
//! built it, so nothing here believes one: what is counted is what really came
//! out, and the reading stops at the limit rather than at whatever the archive
//! said would happen. [`HowMuchToAllow`] is where those limits are.

use crate::application::import_tree::{ENOUGH_TO_TELL_WHAT_IT_HOLDS, EntryInTheArchive};
use crate::common::{Error, Result};
use std::io::Read;
use std::path::{Path, PathBuf};

/// An archive somebody pointed at, looked over and ready to be read.
pub struct MailboxArchive {
    /// What was pointed at, for anything that has to be said about it.
    opened_at: PathBuf,
    /// Where its bytes come from.
    holding: Holding,
    /// What it holds: each entry's name and how it begins.
    looked_over: Vec<EntryOpening>,
    /// The places in that list, in the order their names sort.
    ///
    /// A second order rather than sorting the list itself, because what a
    /// caller is given is in the archive's own order and an import says what it
    /// did folder by folder: somebody listening to a long one hears their
    /// folders in the order they know them in.
    ///
    /// And a search rather than a walk, because an archive of forty thousand
    /// messages is looked up forty thousand times, and a walk each time is
    /// forty thousand walks.
    in_name_order: Vec<usize>,
    /// How many things in it could not be read at all.
    could_not_be_read: usize,
    /// How many folders in it sat deeper than this program follows.
    too_deep_to_follow: usize,
    /// How much of it may be read before it is refused.
    allowed: HowMuchToAllow,
    /// How much has really come out of it so far.
    ///
    /// Kept for the life of the archive rather than reset at each entry,
    /// because the limit it is counted against is about the whole import: a
    /// thousand entries each just under the limit for one add up to something
    /// no disk holds.
    unpacked_so_far: u64,
}

/// The two things an archive can be, once it is open.
enum Holding {
    /// One file somebody chose.
    AZipFile(Box<zip::ZipArchive<std::fs::File>>),
    /// A folder somebody chose, with every name below relative to it.
    AFolder(PathBuf),
}

/// One entry, named and opened far enough to say what it holds.
struct EntryOpening {
    /// The path inside the archive, spelled however the archive spelled it.
    named: String,
    /// The beginning of it, or all of it when it is shorter than that.
    opens_with: Vec<u8>,
}

/// How much of an archive this program will read before refusing it.
///
/// An archive is a stranger's file, and a small one can claim to unpack to more
/// than every disk ever made. Reading it as far as it says fills this computer's
/// memory or its disk, and the program stops. So there is a limit on every part
/// of it that can grow, and each of these bounds something different: the first
/// two bound what looking over an archive costs, the third bounds one way of
/// reading an entry, and the last bounds what importing a whole archive costs
/// however it is read.
///
/// A value rather than four constants, so a test can hand this small numbers and
/// watch a refusal really happen. A limit nothing has ever reached is a limit
/// nobody has watched work, and every one of these is out of reach of any test
/// that had to build a file to reach it.
///
/// None of these is worked out from what the archive says about itself. A size
/// recorded in an archive was written by whoever built the archive, so what is
/// counted here is what really came out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HowMuchToAllow {
    /// How many things an archive may hold.
    ///
    /// Every one of them is a name to keep, a read to make and a decision to
    /// take, so a file claiming a million of them is work nobody asked for
    /// before a byte of mail has been looked at.
    pub most_things_in_it: usize,
    /// How much of an archive may be held at once while looking it over.
    ///
    /// The opening of every entry is kept until the whole archive has been
    /// placed, which is the one part of this that grows with the archive.
    pub most_held_while_looking_it_over: u64,
    /// How much any one entry may unpack to when the whole of it is asked for
    /// at once.
    ///
    /// A bound on one call rather than on what an archive may hold.
    /// [`MailboxArchive::one_entry_read_through`] hands back the entry as
    /// bytes, so the entry is in memory, and this is what keeps that from being
    /// the whole of a mailbox.
    /// [`MailboxArchive::one_entry_read_in_pieces`] holds none of the entry and
    /// is not measured against this at all, which is how a folder larger than
    /// this program's memory is imported rather than refused.
    pub most_one_thing_unpacks_to: u64,
    /// How much the whole archive may unpack to, added up as it is read.
    ///
    /// The limit above is about one entry, and a thousand entries each just
    /// under it add up to something no disk holds.
    pub most_the_whole_of_it_unpacks_to: u64,
}

impl Default for HowMuchToAllow {
    /// The limits this program ships with.
    ///
    /// Each is generous for the largest mailbox anybody has kept and nowhere
    /// near what a hostile file asks for. A mailbox saved as one file per
    /// message runs to tens of thousands of them, so a hundred thousand is past
    /// all of that; the opening of every entry in one of those is a few hundred
    /// megabytes, so half a gigabyte covers it; a gigabyte is more than enough
    /// for the entries small enough to be worth asking for whole, and the
    /// entries larger than that are read a piece at a time and never measured
    /// against it.
    fn default() -> Self {
        Self {
            most_things_in_it: 100_000,
            most_held_while_looking_it_over: 512 * 1024 * 1024,
            most_one_thing_unpacks_to: 1024 * 1024 * 1024,
            most_the_whole_of_it_unpacks_to: 20 * 1024 * 1024 * 1024,
        }
    }
}

/// Open the archive at this path, whether it is one file or a folder.
///
/// Both come back as the same thing, because which of the two somebody
/// arrived with has nothing to do with where their mail goes, and the layer
/// that decides that is the layer not allowed to touch a disk.
pub fn opened(at: &Path) -> Result<MailboxArchive> {
    opened_allowing(at, HowMuchToAllow::default())
}

/// The same, with the limits said out loud rather than taken as they ship.
pub fn opened_allowing(at: &Path, allowed: HowMuchToAllow) -> Result<MailboxArchive> {
    if at.is_dir() {
        return a_folder_looked_over(at, allowed);
    }
    a_zip_looked_over(at, allowed)
}

/// One size, in the largest unit that leaves a number somebody can hold.
///
/// These go into sentences that get read out. Five hundred and thirty six
/// million eight hundred and seventy thousand nine hundred and twelve is a
/// number nobody can hold, and it is 512 megabytes.
fn said_as_a_size(bytes: u64) -> String {
    const ONE_MEGABYTE: u64 = 1024 * 1024;
    const ONE_GIGABYTE: u64 = ONE_MEGABYTE * 1024;
    match bytes {
        huge if huge >= ONE_GIGABYTE => format!("{} gigabytes", huge / ONE_GIGABYTE),
        large if large >= ONE_MEGABYTE => format!("{} megabytes", large / ONE_MEGABYTE),
        small => format!("{small} bytes"),
    }
}

/// What to say about an archive with more in it than this program will open.
fn holds_more_things_than_this_program_will_open(at: &Path, most: usize) -> Error {
    Error::Other(format!(
        "{} holds more than {most} things, which is more than this program will open at once. \
         Import your mail a few folders at a time instead.",
        at.display()
    ))
}

/// What to say about an archive too large to be looked over at all.
fn too_large_to_look_over(at: &Path, most: u64) -> Error {
    Error::Other(format!(
        "{} is too large for this program to open. Seeing what is in it would take more than \
         {}, which is more memory than it will use. Import your mail a few folders at a time \
         instead.",
        at.display(),
        said_as_a_size(most)
    ))
}

/// What to say about one entry that unpacks to more than will be read.
fn unpacks_to_more_than_will_be_read(at: &Path, named: &str, most: u64) -> Error {
    Error::Other(format!(
        "{named} in {} unpacks to more than {}, which is more than this program will read in \
         one piece. Its mail is still in there and nothing was imported from it.",
        at.display(),
        said_as_a_size(most)
    ))
}

/// What to say about a whole archive that unpacks to more than will be read.
fn the_whole_of_it_unpacks_to_more_than_will_be_read(at: &Path, most: u64) -> Error {
    Error::Other(format!(
        "{} unpacks to more than {}, which is more than this program will read from one \
         archive. What was imported before this point is on this computer; the rest is still \
         in there.",
        at.display(),
        said_as_a_size(most)
    ))
}

/// What to say about a file that is not an archive at all.
///
/// The commonest thing that goes wrong here, and what somebody needs to hear is
/// which file to look for instead, rather than what this program could not
/// read.
fn is_not_a_mailbox_archive(at: &Path) -> Error {
    Error::Other(format!(
        "{} is not a mailbox archive this program can open. Choose the file your old mail \
         program wrote your mail out to, or the folder it wrote it into.",
        at.display()
    ))
}

/// What to say about an archive that is one and stops before its end.
///
/// A download that stopped, a copy off a failing disk, half a file recovered
/// from a backup. Told it is not an archive at all, somebody goes looking for a
/// different file; told it stops partway through, they go and fetch another
/// copy of the one they already have.
fn stops_partway_through(at: &Path) -> Error {
    Error::Other(format!(
        "{} stops partway through, so the mail in it cannot be read. Ask the program that \
         wrote it for another copy.",
        at.display()
    ))
}

/// What to say about an archive with nothing in it.
///
/// An import of nothing that says nothing looks exactly like a broken import,
/// and the difference is the whole of what somebody needs in order to decide
/// what to do next. The second sentence differs by what was pointed at, because
/// telling somebody to choose a folder when they chose a file is telling them
/// to do the thing they just did.
fn there_is_nothing_in(at: &Path, choose_instead: &str) -> Error {
    Error::Other(format!(
        "There is nothing in {}. {choose_instead}",
        at.display()
    ))
}

/// What to tell somebody who pointed at a file with nothing in it.
const CHOOSE_ANOTHER_FILE: &str = "Choose the file your old mail program wrote your mail out to.";

/// What to tell somebody who pointed at a folder with nothing in it.
const CHOOSE_ANOTHER_FOLDER: &str = "Choose the folder your old mail program wrote your mail into.";

/// One zip file, looked over far enough to say what is in it.
fn a_zip_looked_over(at: &Path, allowed: HowMuchToAllow) -> Result<MailboxArchive> {
    let mut file = std::fs::File::open(at)
        .map_err(|why| Error::Other(format!("{} could not be opened: {why}.", at.display())))?;
    let began_like_one = it_begins_like_an_archive(&mut file)
        .map_err(|why| Error::Other(format!("{} could not be read: {why}.", at.display())))?;
    let mut zip = zip::ZipArchive::new(file).map_err(|_| {
        if began_like_one {
            stops_partway_through(at)
        } else {
            is_not_a_mailbox_archive(at)
        }
    })?;

    // Before anything is read. The count below only rises for entries that are
    // kept, and an archive listing a million folder marks and one file would
    // otherwise be a million turns of this loop before it reached the one.
    if zip.len() > allowed.most_things_in_it {
        return Err(holds_more_things_than_this_program_will_open(
            at,
            allowed.most_things_in_it,
        ));
    }

    let mut found = WhatWasFound::of(at, allowed);
    for which in 0..zip.len() {
        let Ok(mut entry) = zip.by_index(which) else {
            // The rest of the archive is still there. Abandoning it here would
            // lose somebody eleven folders of mail over one, and say nothing
            // about which one was the problem.
            found.one_could_not_be_read(&format!("entry {which} of {}", at.display()));
            continue;
        };
        // A zip marks each of its own folders with an entry of no bytes, and a
        // link is a name pointing at a file somewhere else rather than mail.
        // Neither holds any mail, and an archive of fifty folders carries fifty
        // of the first kind.
        if !entry.is_file() {
            continue;
        }
        let named = entry.name().to_string();
        // A name in an archive is whatever bytes somebody wrote and need not be
        // text at all, and what comes back for one of those has a stand-in
        // character where each unreadable byte was. Handed over as it is, since
        // the same stand-in name finds the entry again, but said out loud
        // because a folder that reads aloud as nothing is worth a line in the
        // log when somebody asks why.
        if named.contains(char::REPLACEMENT_CHARACTER) {
            tracing::warn!(
                "the name {named} in {} is not text, so it will read out as very little",
                at.display()
            );
        }
        match how_it_begins(&mut entry) {
            Ok(opens_with) => found.one_more(named, opens_with)?,
            Err(why) => {
                found.one_could_not_be_read(&format!("{named} in {}: {why}", at.display()));
            }
        }
    }

    found.nothing_at_all(at, CHOOSE_ANOTHER_FILE)?;
    Ok(MailboxArchive::of(
        at,
        Holding::AZipFile(Box::new(zip)),
        found,
        allowed,
    ))
}

/// Whether a file at least begins the way an archive does.
///
/// Only ever asked once opening it as one has already failed, and only to
/// choose between two sentences: a file that never was an archive, and one that
/// is an archive and stops partway through. Those ask different things of the
/// person in front of the screen.
///
/// The file is put back to its beginning afterwards, because what reads it next
/// is handed the same open file.
fn it_begins_like_an_archive(file: &mut std::fs::File) -> std::io::Result<bool> {
    let mut opening = [0u8; 2];
    let begins_like_one = match file.read_exact(&mut opening) {
        Ok(()) => opening == *b"PK",
        // A file too short to hold two bytes is too short to be an archive, and
        // that is an answer rather than a failure to read.
        Err(why) if why.kind() == std::io::ErrorKind::UnexpectedEof => false,
        Err(why) => return Err(why),
    };
    std::io::Seek::rewind(file)?;
    Ok(begins_like_one)
}

/// The most folders deep a chosen folder is followed.
///
/// Past the depth the placer will accept anyway, so nothing this stops at could
/// have been imported in any case. It is here to stop a walk that never ends
/// rather than to make a decision about somebody's mail, and what it counts is
/// said out loud so a folder left behind is never left behind in silence.
const MOST_FOLDERS_DEEP: usize = 32;

/// One folder somebody pointed at, walked far enough to say what is in it.
fn a_folder_looked_over(at: &Path, allowed: HowMuchToAllow) -> Result<MailboxArchive> {
    let mut found = WhatWasFound::of(at, allowed);
    found.walked(at, "", 0)?;
    // The order a folder is read back in belongs to the filesystem and is not
    // promised to be the same twice. An import says what it did folder by
    // folder, so two runs over one archive have to name them in one order.
    found
        .entries
        .sort_by(|one, other| one.named.cmp(&other.named));

    found.nothing_at_all(at, CHOOSE_ANOTHER_FOLDER)?;
    Ok(MailboxArchive::of(
        at,
        Holding::AFolder(at.to_path_buf()),
        found,
        allowed,
    ))
}

/// What looking over an archive turned up, and what it could not.
struct WhatWasFound {
    /// What was pointed at, for anything that has to be said about it.
    opened_at: PathBuf,
    /// How much of it may be read before it is refused.
    allowed: HowMuchToAllow,
    /// Every entry that could be named and opened.
    entries: Vec<EntryOpening>,
    /// How much the entries above take up together.
    held_while_looking: u64,
    /// How many could not be read, whatever the reason.
    could_not_be_read: usize,
    /// How many folders sat deeper than this program follows.
    too_deep_to_follow: usize,
}

impl WhatWasFound {
    /// Nothing found yet, in an archive that may be read this much.
    fn of(opened_at: &Path, allowed: HowMuchToAllow) -> Self {
        Self {
            opened_at: opened_at.to_path_buf(),
            allowed,
            entries: Vec::new(),
            held_while_looking: 0,
            could_not_be_read: 0,
            too_deep_to_follow: 0,
        }
    }

    /// Take one more entry, or refuse the archive for having too much in it.
    ///
    /// The one place either limit on looking an archive over is applied, so a
    /// zip and a folder are held to the same one. What is counted is the name
    /// and the opening really read, never a size the archive stated about
    /// itself.
    fn one_more(&mut self, named: String, opens_with: Vec<u8>) -> Result<()> {
        if self.entries.len() >= self.allowed.most_things_in_it {
            return Err(holds_more_things_than_this_program_will_open(
                &self.opened_at,
                self.allowed.most_things_in_it,
            ));
        }
        let taking = (named.len() + opens_with.len()) as u64;
        self.held_while_looking = self.held_while_looking.saturating_add(taking);
        if self.held_while_looking > self.allowed.most_held_while_looking_it_over {
            return Err(too_large_to_look_over(
                &self.opened_at,
                self.allowed.most_held_while_looking_it_over,
            ));
        }
        self.entries.push(EntryOpening { named, opens_with });
        Ok(())
    }

    /// Count one thing that could not be read, and say in the log which.
    ///
    /// Counted so it can be said out loud at the end, and logged so somebody
    /// helping can find out which file it was. A count on its own says a file
    /// was lost and not which one; a log line on its own is a warning nobody
    /// gets.
    fn one_could_not_be_read(&mut self, which: &str) {
        self.could_not_be_read += 1;
        tracing::warn!("{which} could not be read, so it was left out of the import");
    }

    /// Refuse an archive that turned out to hold nothing at all.
    ///
    /// An archive whose every entry was unreadable is a damaged archive rather
    /// than an empty one, and telling somebody to choose a different file when
    /// the one they chose is the right file and broken sends them looking in
    /// the wrong place.
    fn nothing_at_all(&self, at: &Path, choose_instead: &str) -> Result<()> {
        if !self.entries.is_empty() {
            return Ok(());
        }
        if self.could_not_be_read > 0 {
            return Err(stops_partway_through(at));
        }
        Err(there_is_nothing_in(at, choose_instead))
    }

    /// Walk one folder, with `named_under` the name every entry in it carries
    /// in front of its own.
    ///
    /// The name is built on the way down rather than worked back out of the
    /// path afterwards, so it is made of the parts the filesystem gave and
    /// nothing else, and it separates its folders the way an archive does
    /// whichever platform is running.
    ///
    /// A failure comes back for the folder somebody chose itself, and for an
    /// archive with too much in it. A file that cannot be read, or a folder
    /// inside this one that cannot be listed, is counted and the walk goes on:
    /// a folder somebody points at is a live part of their computer, where a
    /// file can be held open by the program that wrote it or be gone between
    /// the listing and the reading, and one of those must not cost them the
    /// other eleven folders of mail.
    fn walked(&mut self, folder: &Path, named_under: &str, depth: usize) -> Result<()> {
        let reading = match std::fs::read_dir(folder) {
            Ok(reading) => reading,
            // The folder somebody chose. Nothing was read, so there is nothing
            // to go on with and no count that would ever be said out loud.
            Err(why) if depth == 0 => {
                return Err(Error::Other(format!(
                    "{} could not be read: {why}.",
                    folder.display()
                )));
            }
            Err(why) => {
                self.one_could_not_be_read(&format!("{named_under}: {why}"));
                return Ok(());
            }
        };
        for found in reading {
            let Ok(found) = found else {
                self.one_could_not_be_read(&format!("something in {}", folder.display()));
                continue;
            };
            let Some(named) = found.file_name().to_str().map(|part| {
                if named_under.is_empty() {
                    part.to_string()
                } else {
                    format!("{named_under}/{part}")
                }
            }) else {
                // A name this computer holds and Rust cannot read as text. It
                // could not be handed over, asked for again, or read out loud,
                // so it is left where it is and counted.
                self.one_could_not_be_read(&format!("a file in {}", folder.display()));
                continue;
            };
            let Ok(what_it_is) = found.file_type() else {
                self.one_could_not_be_read(&named);
                continue;
            };
            if what_it_is.is_dir() {
                if depth + 1 > MOST_FOLDERS_DEEP {
                    self.too_deep_to_follow += 1;
                    tracing::warn!("{named} sits deeper than this program follows a folder");
                    continue;
                }
                self.walked(&found.path(), &named, depth + 1)?;
                continue;
            }
            // Anything that is neither a file nor a folder is a link, a pipe or
            // a device. None of them holds mail, and following a link is how a
            // walk comes to go round in a circle or to read a file somewhere
            // else on this computer.
            if !what_it_is.is_file() {
                continue;
            }
            match std::fs::File::open(found.path()).and_then(|mut file| how_it_begins(&mut file)) {
                Ok(opens_with) => self.one_more(named, opens_with)?,
                Err(why) => self.one_could_not_be_read(&format!("{named}: {why}")),
            }
        }
        Ok(())
    }
}

/// How one entry begins, and no more of it than that.
///
/// [`ENOUGH_TO_TELL_WHAT_IT_HOLDS`] is where the placer stops looking, so
/// reading further would be reading somebody's mail into memory to answer a
/// question that was already settled.
fn how_it_begins(entry: &mut impl Read) -> std::io::Result<Vec<u8>> {
    let mut opens_with = Vec::new();
    entry
        .take(ENOUGH_TO_TELL_WHAT_IT_HOLDS as u64)
        .read_to_end(&mut opens_with)?;
    Ok(opens_with)
}

/// One entry unpacked as far as it is allowed to go, or nothing when there is
/// more of it than that.
///
/// One byte past the limit is read on purpose, because that is the difference
/// between an entry that fits exactly and one that does not, and there is no
/// other way to tell them apart without believing what the archive says about
/// itself.
fn as_far_as_allowed(entry: &mut impl Read, allowed: u64) -> std::io::Result<Option<Vec<u8>>> {
    let mut unpacked = Vec::new();
    entry
        .take(allowed.saturating_add(1))
        .read_to_end(&mut unpacked)?;
    if unpacked.len() as u64 > allowed {
        return Ok(None);
    }
    Ok(Some(unpacked))
}

/// What to say when an archive has unpacked as much as it is allowed to.
///
/// Said as a reading that failed rather than as a short one, because a folder
/// of mail handed over with its last thousand messages missing is imported
/// without complaint and nobody finds out until they go looking for one of
/// them. What the person in front of the screen is told is the sentence
/// [`the_whole_of_it_unpacks_to_more_than_will_be_read`] writes; this only has
/// to stop the reading.
const THERE_IS_NO_ROOM_LEFT: &str = "this archive has unpacked as much as it is allowed to";

/// One entry being read, counting what really comes out of it.
///
/// A size recorded in an archive was written by whoever built the archive, so
/// what is counted here is what really arrived. One byte past what is left is
/// read on purpose, because that is the difference between an entry that fits
/// exactly and one that does not.
struct AsFarAsThereIsRoom<R> {
    /// Where the bytes come from.
    reading: R,
    /// How much may still come out of this archive altogether.
    room: u64,
    /// How much really has.
    came_out: u64,
    /// Whether it asked for more than there was room for.
    ran_out: bool,
}

impl<R: Read> AsFarAsThereIsRoom<R> {
    /// One entry, with this much room left in the archive it came from.
    fn of(reading: R, room: u64) -> Self {
        Self {
            reading,
            room,
            came_out: 0,
            ran_out: false,
        }
    }
}

impl<R: Read> Read for AsFarAsThereIsRoom<R> {
    fn read(&mut self, into: &mut [u8]) -> std::io::Result<usize> {
        let one_more_than_is_left = self.room.saturating_sub(self.came_out).saturating_add(1);
        let asking = into
            .len()
            .min(usize::try_from(one_more_than_is_left).unwrap_or(usize::MAX));
        let read = self.reading.read(&mut into[..asking])?;
        self.came_out = self.came_out.saturating_add(read as u64);
        if self.came_out > self.room {
            self.ran_out = true;
            return Err(std::io::Error::other(THERE_IS_NO_ROOM_LEFT));
        }
        Ok(read)
    }
}

impl MailboxArchive {
    /// One archive, with its entries put into a second order they can be found
    /// again in.
    fn of(
        opened_at: &Path,
        holding: Holding,
        found: WhatWasFound,
        allowed: HowMuchToAllow,
    ) -> Self {
        let looked_over = found.entries;
        let mut in_name_order: Vec<usize> = (0..looked_over.len()).collect();
        in_name_order.sort_by(|one, other| looked_over[*one].named.cmp(&looked_over[*other].named));
        Self {
            opened_at: opened_at.to_path_buf(),
            holding,
            looked_over,
            in_name_order,
            could_not_be_read: found.could_not_be_read,
            too_deep_to_follow: found.too_deep_to_follow,
            allowed,
            unpacked_so_far: 0,
        }
    }

    /// How many things in the archive could not be read at all.
    ///
    /// Not nought is somebody's mail still sitting in their archive, so it is
    /// the caller's job to say so at the end rather than leave it in a log.
    pub fn how_many_could_not_be_read(&self) -> usize {
        self.could_not_be_read
    }

    /// How many folders sat deeper than this program follows a folder.
    pub fn how_many_were_too_deep_to_follow(&self) -> usize {
        self.too_deep_to_follow
    }

    /// One entry read the whole way through, handed back as bytes.
    ///
    /// The whole entry is in memory when this returns, so it is bounded by
    /// [`HowMuchToAllow::most_one_thing_unpacks_to`] and refuses anything
    /// larger. For a folder of mail saved as one file, which is the largest
    /// thing an archive holds, use
    /// [`MailboxArchive::one_entry_read_in_pieces`] instead: it holds none of
    /// the entry and has no such limit.
    ///
    /// Only under a name the archive itself offered. For a folder that is the
    /// whole of the safety of this call: the name is joined to the folder
    /// somebody chose, and a name that climbed out of it would read a file
    /// somewhere else on this computer and file its contents as mail. Nothing
    /// here repairs such a name, because a name the archive never offered is
    /// not a name this archive has.
    pub fn one_entry_read_through(&mut self, named: &str) -> Result<Vec<u8>> {
        if !self.holds(named) {
            return Err(Error::Other(format!(
                "There is nothing called {named} in {}.",
                self.opened_at.display()
            )));
        }
        let opened_at = self.opened_at.clone();
        let allowed = self.allowed;
        // How much is left of what this archive may unpack to altogether, and
        // never more than one entry may unpack to on its own. The reading stops
        // at whichever comes first rather than at whatever the archive said
        // would happen.
        let room_left = allowed
            .most_the_whole_of_it_unpacks_to
            .saturating_sub(self.unpacked_so_far)
            .min(allowed.most_one_thing_unpacks_to);

        let all_of_it = match &mut self.holding {
            Holding::AZipFile(zip) => {
                let mut entry = zip.by_name(named).map_err(|why| {
                    Error::Other(format!(
                        "{named} in {} could not be read: {why}.",
                        opened_at.display()
                    ))
                })?;
                as_far_as_allowed(&mut entry, room_left)
            }
            Holding::AFolder(root) => std::fs::File::open(root.join(named))
                .and_then(|mut file| as_far_as_allowed(&mut file, room_left)),
        }
        .map_err(|why| {
            Error::Other(format!(
                "{named} in {} could not be read: {why}.",
                opened_at.display()
            ))
        })?;

        // Refused rather than cut short. A folder of mail handed over with its
        // last thousand messages missing is imported without complaint and
        // nobody finds out until they go looking for one of them.
        let Some(all_of_it) = all_of_it else {
            if room_left < allowed.most_one_thing_unpacks_to {
                return Err(the_whole_of_it_unpacks_to_more_than_will_be_read(
                    &opened_at,
                    allowed.most_the_whole_of_it_unpacks_to,
                ));
            }
            return Err(unpacks_to_more_than_will_be_read(
                &opened_at,
                named,
                allowed.most_one_thing_unpacks_to,
            ));
        };
        self.unpacked_so_far = self.unpacked_so_far.saturating_add(all_of_it.len() as u64);
        Ok(all_of_it)
    }

    /// One entry handed over a piece at a time, for mail too large to hold.
    ///
    /// The same entry as [`MailboxArchive::one_entry_read_through`] and the
    /// same name rule, and it is never held: what the caller is given is
    /// somewhere to read from, and it reads as much as it wants at a time and
    /// lets each piece go. So there is no limit here on how large one entry may
    /// be, because there is nothing that grows with it. A folder somebody has
    /// kept for twenty years and exported as one file is exactly the entry that
    /// could not be read before and can be now.
    ///
    /// What the whole archive may unpack to still holds. That one is not about
    /// memory: it is what stops a small hostile file filling this computer's
    /// disk with what it unpacks to, an entry at a time. Reaching it partway
    /// through an entry says so, and says that what was imported before that
    /// point is on this computer.
    ///
    /// The reading is handed to a caller inside a call rather than given out as
    /// a value, because the archive has to know how much really came out of it
    /// before it is asked for anything else.
    pub fn one_entry_read_in_pieces<T>(
        &mut self,
        named: &str,
        take: impl FnOnce(&mut dyn Read) -> Result<T>,
    ) -> Result<T> {
        if !self.holds(named) {
            return Err(Error::Other(format!(
                "There is nothing called {named} in {}.",
                self.opened_at.display()
            )));
        }
        let opened_at = self.opened_at.clone();
        let allowed = self.allowed;
        let could_not_be_read = |why: &dyn std::fmt::Display| {
            Error::Other(format!(
                "{named} in {} could not be read: {why}.",
                opened_at.display()
            ))
        };
        let room_left = allowed
            .most_the_whole_of_it_unpacks_to
            .saturating_sub(self.unpacked_so_far);

        let (what_the_caller_made, came_out, ran_out) = match &mut self.holding {
            Holding::AZipFile(zip) => {
                let entry = zip.by_name(named).map_err(|why| could_not_be_read(&why))?;
                let mut counting = AsFarAsThereIsRoom::of(entry, room_left);
                let made = take(&mut counting);
                (made, counting.came_out, counting.ran_out)
            }
            Holding::AFolder(root) => {
                let file =
                    std::fs::File::open(root.join(named)).map_err(|why| could_not_be_read(&why))?;
                let mut counting = AsFarAsThereIsRoom::of(file, room_left);
                let made = take(&mut counting);
                (made, counting.came_out, counting.ran_out)
            }
        };

        self.unpacked_so_far = self.unpacked_so_far.saturating_add(came_out);
        if ran_out {
            return Err(the_whole_of_it_unpacks_to_more_than_will_be_read(
                &opened_at,
                allowed.most_the_whole_of_it_unpacks_to,
            ));
        }
        what_the_caller_made
    }

    /// Whether one of the names the archive offered is this one.
    ///
    /// A name is what a caller has to go on, so two entries under one name
    /// would be one entry read twice. In a zip that cannot arrive here: the
    /// reader underneath keeps entries by name and has already made two of them
    /// into one before anything in this file sees either. That is worth knowing
    /// rather than being surprised by, and nothing at this level can see it
    /// happen or say so. A folder cannot hold two files under one name at all.
    fn holds(&self, named: &str) -> bool {
        self.in_name_order
            .binary_search_by(|at| self.looked_over[*at].named.as_str().cmp(named))
            .is_ok()
    }

    /// Everything the archive holds: each entry's name and how it begins.
    pub fn what_it_holds(&self) -> Vec<EntryInTheArchive<'_>> {
        self.looked_over
            .iter()
            .map(|entry| EntryInTheArchive {
                named: &entry.named,
                opens_with: &entry.opens_with,
            })
            .collect()
    }

    /// What was pointed at.
    pub fn opened_at(&self) -> &Path {
        &self.opened_at
    }

    /// Whether what was pointed at was a folder rather than a file.
    ///
    /// Worth asking because what somebody is told about an archive with nothing
    /// in it depends on it: telling them to choose a folder when they chose a
    /// folder is telling them to do the thing they just did.
    pub fn is_a_folder(&self) -> bool {
        matches!(self.holding, Holding::AFolder(_))
    }
}

// ── Writing one out ─────────────────────────────────────────────────────────

/// An archive being written, one entry at a time.
///
/// A folder of forty thousand messages does not fit in this computer's memory
/// twice, so it is never gathered before it is written. The caller starts a
/// file, hands over one message, lets it go, and hands over the next: the mail
/// is in memory once on its way past rather than twice at rest.
pub struct ArchiveBeingWritten {
    /// The file being written, and where it goes.
    writing: zip::ZipWriter<std::fs::File>,
    /// What the caller named, for anything that has to be said about it.
    at: PathBuf,
    /// Whether a file has been started for what is handed over next.
    started_a_file: bool,
}

/// Start writing an archive at the path the caller named, and nowhere else.
///
/// Opened now rather than at the first message, so a folder that is not there
/// or a disk that is full is found before somebody watches a long export run
/// and then hears it failed.
pub fn written_to(at: &Path) -> Result<ArchiveBeingWritten> {
    let file = std::fs::File::create(at)
        .map_err(|why| Error::Other(format!("{} could not be written: {why}.", at.display())))?;
    Ok(ArchiveBeingWritten {
        writing: zip::ZipWriter::new(file),
        at: at.to_path_buf(),
        started_a_file: false,
    })
}

impl ArchiveBeingWritten {
    /// How each entry is written.
    ///
    /// Packed, because mail compresses unusually well and an export of a
    /// mailbox is the largest file this program writes.
    ///
    /// The large-file form on every entry, which costs twenty bytes each and is
    /// not optional: one folder of somebody's mail can pass four gigabytes, and
    /// without it the write fails at that point with the mail already in the
    /// file. Twenty bytes an entry against a whole export lost is not a trade
    /// worth thinking about twice.
    fn how_entries_are_written() -> zip::write::SimpleFileOptions {
        zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .large_file(true)
    }

    /// Start one file inside the archive. What is handed over next goes in it.
    pub fn start_a_file(&mut self, named: &str) -> Result<()> {
        self.writing
            .start_file(named, Self::how_entries_are_written())
            .map_err(|why| self.went_wrong(&format!("{named} could not be written: {why}")))?;
        self.started_a_file = true;
        Ok(())
    }

    /// Put a folder in the archive with nothing inside it.
    ///
    /// An empty folder somebody has kept for years is part of the shape an
    /// export exists to keep. It cannot go in as an archive of no messages,
    /// because that is a file this program's own import turns away as not mail,
    /// so it goes in as the folder and nothing else.
    pub fn add_a_folder_with_nothing_in_it(&mut self, named: &str) -> Result<()> {
        self.writing
            .add_directory(named, Self::how_entries_are_written())
            .map_err(|why| self.went_wrong(&format!("{named} could not be written: {why}")))?;
        self.started_a_file = false;
        Ok(())
    }

    /// Write the next piece of the file that was started.
    ///
    /// Refused when no file has been started, rather than taken and dropped.
    /// Mail handed to a writer with nowhere to put it goes nowhere, the export
    /// says it worked, and the messages are missing from a file nobody opens
    /// again for a year.
    pub fn write_into_it(&mut self, bytes: &[u8]) -> Result<()> {
        if !self.started_a_file {
            return Err(self.went_wrong(
                "there is no file in it yet for this mail to go into, so nothing was written",
            ));
        }
        std::io::Write::write_all(&mut self.writing, bytes)
            .map_err(|why| self.went_wrong(&format!("it could not be written: {why}")))
    }

    /// Close the archive, so what is in it can be opened again.
    ///
    /// Said out loud rather than left to happen when this is dropped. An
    /// archive closed on the way out closes without a word, and a failure at
    /// that moment is the one that loses the whole export.
    pub fn finish(self) -> Result<()> {
        let at = self.at;
        let file = self.writing.finish().map_err(|why| {
            Error::Other(format!("{} could not be written: {why}.", at.display()))
        })?;
        // Onto the disk rather than into whatever is holding it. An export
        // somebody is about to copy to another computer has to be there.
        file.sync_all()
            .map_err(|why| Error::Other(format!("{} could not be written: {why}.", at.display())))
    }

    /// What to say when something in this archive could not be written.
    fn went_wrong(&self, why: &str) -> Error {
        Error::Other(format!("In {}, {why}.", self.at.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Two messages in the format an archive of one folder uses.
    fn a_folder_of_mail() -> &'static [u8] {
        concat!(
            "From ada@example.com Mon Jul 20 10:00:00 2026\r\n",
            "From: Ada Lovelace <ada@example.com>\r\n",
            "Subject: The first one\r\n",
            "\r\n",
            "One.\r\n",
            "\r\n",
            "From charles@example.com Tue Jul 21 11:00:00 2026\r\n",
            "From: Charles Babbage <charles@example.com>\r\n",
            "Subject: The second one\r\n",
            "\r\n",
            "Two.\r\n",
        )
        .as_bytes()
    }

    /// One ordinary message, as a mail program saves a single one.
    fn one_message() -> &'static [u8] {
        concat!(
            "From: Ada Lovelace <ada@example.com>\r\n",
            "Subject: Notes on the engine\r\n",
            "\r\n",
            "The engine weaves algebraic patterns.\r\n",
        )
        .as_bytes()
    }

    /// What a small archive holds, for a test that builds one both ways.
    fn some_mail() -> Vec<(&'static str, &'static [u8])> {
        vec![
            ("Work/Invoices.mbox", a_folder_of_mail()),
            ("Work/2024/one.eml", one_message()),
        ]
    }

    /// A somewhere to build test archives in, thrown away with the test.
    fn a_place_to_work() -> tempfile::TempDir {
        tempfile::tempdir().expect("a temporary folder to build test archives in")
    }

    /// One zip file holding these entries, built with this project's own zip
    /// dependency rather than a checked-in binary, so every entry a test cares
    /// about is readable in the diff.
    fn a_zip_of(inside: &Path, entries: &[(&str, &[u8])]) -> PathBuf {
        let at = inside.join("mailbox.zip");
        let file = std::fs::File::create(&at).expect("create the test zip");
        let mut writing = zip::ZipWriter::new(file);
        for (named, bytes) in entries {
            writing
                .start_file(*named, zip::write::SimpleFileOptions::default())
                .expect("start an entry");
            writing.write_all(bytes).expect("write an entry");
        }
        writing.finish().expect("finish the test zip");
        at
    }

    /// One folder holding the same entries as files, folders and all.
    fn a_folder_of(inside: &Path, entries: &[(&str, &[u8])]) -> PathBuf {
        let at = inside.join("mailbox");
        for (named, bytes) in entries {
            let file = at.join(named);
            if let Some(holding) = file.parent() {
                std::fs::create_dir_all(holding).expect("make the folders above a test file");
            }
            std::fs::write(&file, bytes).expect("write a test file");
        }
        std::fs::create_dir_all(&at).expect("make the test folder even when it holds nothing");
        at
    }

    /// What an archive holds, in a shape two archives can be compared in.
    fn named_and_opening(archive: &MailboxArchive) -> Vec<(String, Vec<u8>)> {
        let mut held: Vec<(String, Vec<u8>)> = archive
            .what_it_holds()
            .iter()
            .map(|entry| (entry.named.to_string(), entry.opens_with.to_vec()))
            .collect();
        held.sort();
        held
    }

    #[test]
    fn test_a_zip_and_a_folder_holding_the_same_mail_look_the_same_to_a_caller() {
        // The whole point of this module. Somebody leaving another mail program
        // arrives with one file or with the folder that program wrote, and
        // which of the two it is has nothing to do with where their mail goes.
        // Anything that had to ask would have to ask in the layer that decides,
        // which is the layer that is not allowed to touch a disk.
        let place = a_place_to_work();
        let as_a_file = a_zip_of(place.path(), &some_mail());
        let as_a_folder = a_folder_of(place.path(), &some_mail());

        let from_the_file = opened(&as_a_file).expect("a zip of mail opens");
        let from_the_folder = opened(&as_a_folder).expect("a folder of mail opens");

        assert_eq!(
            named_and_opening(&from_the_file),
            named_and_opening(&from_the_folder)
        );
        assert_eq!(
            named_and_opening(&from_the_file),
            vec![
                ("Work/2024/one.eml".to_string(), one_message().to_vec()),
                (
                    "Work/Invoices.mbox".to_string(),
                    a_folder_of_mail().to_vec()
                ),
            ]
        );
        assert!(!from_the_file.is_a_folder());
        assert!(from_the_folder.is_a_folder());
    }

    /// A mailbox longer than the opening this module hands over.
    fn a_long_mailbox() -> Vec<u8> {
        let mut long = a_folder_of_mail().to_vec();
        long.extend(
            "A line of somebody's message.\r\n"
                .repeat(4000)
                .into_bytes(),
        );
        assert!(long.len() > ENOUGH_TO_TELL_WHAT_IT_HOLDS);
        long
    }

    /// Both kinds of archive holding the same entries, so a test can ask each
    /// of them the same question and know it got one answer.
    fn both_kinds(place: &tempfile::TempDir, entries: &[(&str, &[u8])]) -> Vec<MailboxArchive> {
        vec![
            opened(&a_zip_of(place.path(), entries)).expect("a zip of mail opens"),
            opened(&a_folder_of(place.path(), entries)).expect("a folder of mail opens"),
        ]
    }

    #[test]
    fn test_only_the_opening_of_an_entry_is_kept_rather_than_the_whole_of_it() {
        // What lets somebody look over an archive of twenty years of mail on a
        // computer that could not hold it. Every entry is placed from its
        // opening bytes, and the placer stops looking at exactly this point, so
        // reading further is reading somebody's mail into memory to answer a
        // question that has already been answered.
        let place = a_place_to_work();
        let long = a_long_mailbox();
        let held: Vec<(&str, &[u8])> = vec![("Work/Invoices.mbox", &long)];

        for archive in both_kinds(&place, &held) {
            let entries = archive.what_it_holds();

            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].opens_with.len(), ENOUGH_TO_TELL_WHAT_IT_HOLDS);
            assert_eq!(
                entries[0].opens_with,
                &long[..ENOUGH_TO_TELL_WHAT_IT_HOLDS],
                "the opening handed over is not the entry's own opening"
            );
        }
    }

    #[test]
    fn test_an_entry_is_read_through_under_the_name_the_archive_gave_it() {
        // The second half of the bargain. An archive is looked over from
        // openings alone, and then, one folder at a time, the entries that fill
        // each folder are read through and let go again.
        let place = a_place_to_work();
        let long = a_long_mailbox();
        let held: Vec<(&str, &[u8])> = vec![
            ("Work/Invoices.mbox", &long),
            ("Work/2024/one.eml", one_message()),
        ];

        for mut archive in both_kinds(&place, &held) {
            assert_eq!(
                archive
                    .one_entry_read_through("Work/Invoices.mbox")
                    .expect("the archive holds this entry"),
                long
            );
            assert_eq!(
                archive
                    .one_entry_read_through("Work/2024/one.eml")
                    .expect("the archive holds this entry"),
                one_message()
            );
        }
    }

    /// One entry read the whole way through, a piece at a time, gathered here
    /// so a test can compare it with what went in.
    ///
    /// What the import does instead is hand each piece to a reader that gives
    /// up one message at a time and keeps none of them, which is the point of
    /// the call. Gathering it is what a test needs and what nothing else should
    /// do.
    fn gathered_a_piece_at_a_time(archive: &mut MailboxArchive, named: &str) -> Result<Vec<u8>> {
        archive.one_entry_read_in_pieces(named, |reading| {
            let mut gathered = Vec::new();
            reading
                .read_to_end(&mut gathered)
                .map_err(|why| Error::Other(format!("{named} could not be read: {why}.")))?;
            Ok(gathered)
        })
    }

    #[test]
    fn test_an_entry_larger_than_will_be_held_is_still_read_a_piece_at_a_time() {
        // The reason any of this exists. A folder somebody has used for twenty
        // years, exported as one file, is larger than what this program will
        // hold in memory at once, and holding it was the only way an entry
        // could be read. So the person with the most mail to bring was the one
        // who could not bring it.
        //
        // Read a piece at a time nothing holds the entry, so the limit that
        // said no does not apply and is not asked.
        let place = a_place_to_work();
        let long = a_long_mailbox();
        let held: Vec<(&str, &[u8])> = vec![("Work/Invoices.mbox", &long)];
        let far_less_than_the_entry = HowMuchToAllow {
            most_one_thing_unpacks_to: 64,
            ..HowMuchToAllow::default()
        };

        for at in [
            a_zip_of(place.path(), &held),
            a_folder_of(place.path(), &held),
        ] {
            let mut archive = opened_allowing(&at, far_less_than_the_entry).expect("it opens");

            // Held whole, it is refused, and that refusal is the door this
            // opens another way through rather than one it takes off.
            assert!(
                archive
                    .one_entry_read_through("Work/Invoices.mbox")
                    .is_err(),
                "{} was held whole after all",
                at.display()
            );
            assert_eq!(
                gathered_a_piece_at_a_time(&mut archive, "Work/Invoices.mbox")
                    .expect("an entry read a piece at a time is not refused for its size"),
                long,
                "{} came back changed",
                at.display()
            );
        }
    }

    #[test]
    fn test_reading_a_piece_at_a_time_still_stops_at_what_the_whole_archive_may_unpack_to() {
        // The limit that still holds when nothing is held. It was never about
        // memory: it is what stops a small hostile file filling this computer's
        // disk with what it unpacks to, an entry at a time, and reading in
        // pieces does not change that.
        //
        // Reached partway through an entry, what came out before it really did
        // come out, and the sentence somebody hears says so rather than
        // pretending the entry was refused whole.
        let place = a_place_to_work();
        let long = a_long_mailbox();
        let held: Vec<(&str, &[u8])> = vec![("Work/Invoices.mbox", &long)];
        let allowing_a_little = HowMuchToAllow {
            most_the_whole_of_it_unpacks_to: 1000,
            ..HowMuchToAllow::default()
        };

        const A_PIECE_AT_A_TIME: usize = 128;
        let mut archive =
            opened_allowing(&a_zip_of(place.path(), &held), allowing_a_little).expect("it opens");
        let mut how_much_came_out = 0;
        let refused = archive
            .one_entry_read_in_pieces("Work/Invoices.mbox", |reading| {
                let mut piece = [0u8; A_PIECE_AT_A_TIME];
                while let Ok(read) = reading.read(&mut piece) {
                    if read == 0 {
                        break;
                    }
                    how_much_came_out += read;
                }
                Ok(())
            })
            .err()
            .map(|why| why.to_string())
            .expect("an archive that unpacks past its total is refused");

        assert!(refused.contains("unpacks to more than"), "{refused}");
        assert!(
            refused.contains("was imported before this point"),
            "{refused}"
        );
        // Never more than the archive was allowed to unpack to, and within one
        // read of it, because the read that went past is refused rather than
        // handed over in part.
        assert!(
            how_much_came_out <= 1000 && how_much_came_out + A_PIECE_AT_A_TIME >= 1000,
            "{how_much_came_out} bytes came out where a thousand were allowed"
        );
    }

    #[test]
    fn test_asking_for_an_entry_no_archive_holds_says_so_rather_than_reading_something_else() {
        // A name is what a caller has to go on, and the one name that must
        // never quietly find a file is one the archive never offered. For a
        // folder that is the whole of the safety of this call: a name is joined
        // to the folder somebody chose, and a name that climbs out of it would
        // read a file somewhere else on this computer and file it as mail.
        let place = a_place_to_work();
        let held = some_mail();

        for mut archive in both_kinds(&place, &held) {
            for asked in [
                "Work/Nothing.mbox",
                "../../../Windows/System32/drivers/etc/hosts",
                r"..\..\somewhere",
                "/etc/passwd",
                r"C:\Windows\System32\drivers\etc\hosts",
                "",
                "Work",
            ] {
                let refused = archive.one_entry_read_through(asked);

                assert!(
                    refused.is_err(),
                    "{asked:?} found something in {}",
                    archive.opened_at().display()
                );
            }
        }
    }

    /// What was said about an archive that would not open.
    fn refused(at: &Path) -> String {
        match opened(at) {
            Err(why) => why.to_string(),
            Ok(open) => panic!(
                "{} opened and holds {} entries",
                at.display(),
                open.what_it_holds().len()
            ),
        }
    }

    #[test]
    fn test_a_file_that_is_not_an_archive_at_all_says_which_file_to_choose() {
        // Somebody picks the wrong file, which is the commonest thing that goes
        // wrong here. A message, a spreadsheet, a photograph: none of them is
        // an archive, and what they need to hear is which file to look for
        // instead, not what this program failed to parse.
        let place = a_place_to_work();
        let not_an_archive = place.path().join("notes.txt");
        std::fs::write(&not_an_archive, b"Dear Charles,\r\n\r\nThe engine.\r\n")
            .expect("write a file that is not an archive");

        let said = refused(&not_an_archive);

        assert!(said.contains("is not a mailbox archive"), "{said}");
        assert!(said.contains("Choose"), "{said}");
    }

    #[test]
    fn test_an_archive_that_stops_partway_through_says_so_rather_than_saying_it_is_not_one() {
        // A download that stopped, a copy off a failing disk, half a file
        // recovered from a backup. Told it is not an archive, somebody goes
        // looking for a different file; told it stops partway through, they go
        // and fetch another copy of the one they have.
        let place = a_place_to_work();
        let whole = a_zip_of(place.path(), &some_mail());
        let mut bytes = std::fs::read(&whole).expect("read the whole test zip back");
        bytes.truncate(bytes.len() / 2);
        let cut_short = place.path().join("half-a-mailbox.zip");
        std::fs::write(&cut_short, &bytes).expect("write half a zip");

        let said = refused(&cut_short);

        assert!(said.contains("stops partway through"), "{said}");
    }

    #[test]
    fn test_an_archive_with_nothing_in_it_says_so_rather_than_importing_nothing_in_silence() {
        // A folder somebody pointed at before their old program had written
        // anything into it, or a file that packed up an empty mailbox. An
        // import of nothing that says nothing looks exactly like a broken
        // import, and the difference is the whole of what somebody needs in
        // order to decide what to do next.
        let place = a_place_to_work();
        let an_empty_folder = a_folder_of(place.path(), &[]);
        let an_empty_file = a_zip_of(place.path(), &[]);

        for at in [&an_empty_folder, &an_empty_file] {
            let said = refused(at);

            assert!(said.contains("There is nothing in"), "{said}");
            assert!(said.contains("Choose"), "{said}");
        }
    }

    #[test]
    fn test_each_thing_that_can_go_wrong_is_told_apart_from_the_others() {
        // Four situations, and each of them asks something different of the
        // person in front of the screen: find another file, fetch another copy
        // of this one, point at a folder with mail in it. One sentence covering
        // all of them tells them none of that.
        let place = a_place_to_work();
        let not_an_archive = place.path().join("notes.txt");
        std::fs::write(&not_an_archive, b"Dear Charles").expect("write a file that is not one");
        let whole = std::fs::read(a_zip_of(place.path(), &some_mail())).expect("read a test zip");
        let cut_short = place.path().join("half.zip");
        std::fs::write(&cut_short, &whole[..whole.len() / 2]).expect("write half a zip");

        let said = [
            refused(&not_an_archive),
            refused(&cut_short),
            refused(&a_folder_of(place.path(), &[])),
            refused(&a_zip_of(place.path(), &[])),
        ];

        for (which, one) in said.iter().enumerate() {
            assert!(one.ends_with('.'), "not a sentence: {one}");
            for other in said.iter().skip(which + 1) {
                assert_ne!(one, other, "two of these say the same thing");
            }
        }
    }

    #[test]
    fn test_one_entry_that_cannot_be_read_does_not_take_the_rest_of_the_archive_with_it() {
        // A damaged entry in the middle of an archive is one folder of mail
        // somebody has lost. Abandoning the read there loses them the other
        // eleven folders as well, and says nothing about which one was the
        // problem.
        let place = a_place_to_work();
        let held: Vec<(&str, &[u8])> = vec![
            ("Work/Broken.mbox", a_folder_of_mail()),
            ("Work/Invoices.mbox", a_folder_of_mail()),
            ("Work/2024/one.eml", one_message()),
        ];
        let at = a_zip_of_stored_bytes(place.path(), &held);
        let mut bytes = std::fs::read(&at).expect("read the test zip back");
        // The first entry is stored rather than packed, so its own bytes are in
        // the file as they were written. One of them changed is an entry whose
        // check no longer matches, which is what a damaged archive gives.
        let broken = bytes
            .windows(4)
            .position(|four| four == b"One.")
            .expect("the first entry's own bytes are in the file");
        bytes[broken] = b'X';
        std::fs::write(&at, &bytes).expect("write the damaged zip");

        let archive = opened(&at).expect("an archive with one damaged entry still opens");

        assert_eq!(
            archive
                .what_it_holds()
                .iter()
                .map(|entry| entry.named)
                .collect::<Vec<_>>(),
            vec!["Work/Invoices.mbox", "Work/2024/one.eml"]
        );
        assert_eq!(archive.how_many_could_not_be_read(), 1);
    }

    /// One zip file whose entries are stored rather than packed, so a test can
    /// find an entry's own bytes in the file and change one of them.
    fn a_zip_of_stored_bytes(inside: &Path, entries: &[(&str, &[u8])]) -> PathBuf {
        let at = inside.join("stored.zip");
        let file = std::fs::File::create(&at).expect("create the test zip");
        let mut writing = zip::ZipWriter::new(file);
        let stored = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for (named, bytes) in entries {
            writing.start_file(*named, stored).expect("start an entry");
            writing.write_all(bytes).expect("write an entry");
        }
        writing.finish().expect("finish the test zip");
        at
    }

    /// Several small entries, enough of them to go past a small limit.
    fn a_few_entries(how_many: usize) -> Vec<(String, Vec<u8>)> {
        (0..how_many)
            .map(|which| (format!("Work/{which}.eml"), one_message().to_vec()))
            .collect()
    }

    /// The same, in the shape the archive builders take.
    fn as_entries(held: &[(String, Vec<u8>)]) -> Vec<(&str, &[u8])> {
        held.iter()
            .map(|(named, bytes)| (named.as_str(), bytes.as_slice()))
            .collect()
    }

    #[test]
    fn test_an_archive_listing_more_things_than_this_program_will_open_is_refused() {
        // A small file can say it holds a million things. Every one of them is
        // a name to hold, a read to make and a row in whatever comes next, so
        // an archive nobody could have made is turned away before any of that
        // rather than partway through it.
        let place = a_place_to_work();
        let held = a_few_entries(6);
        let allowing_three = HowMuchToAllow {
            most_things_in_it: 3,
            ..HowMuchToAllow::default()
        };

        for at in [
            a_zip_of(place.path(), &as_entries(&held)),
            a_folder_of(place.path(), &as_entries(&held)),
        ] {
            let refused = opened_allowing(&at, allowing_three)
                .err()
                .map(|why| why.to_string())
                .unwrap_or_else(|| panic!("{} opened anyway", at.display()));

            assert!(refused.contains("more than 3 things"), "{refused}");
        }
    }

    /// One zip that marks a run of folders and then holds a single file.
    fn a_zip_of_folder_marks(inside: &Path, how_many: usize) -> PathBuf {
        let at = inside.join("marks.zip");
        let file = std::fs::File::create(&at).expect("create the test zip");
        let mut writing = zip::ZipWriter::new(file);
        let plain = zip::write::SimpleFileOptions::default();
        for which in 0..how_many {
            writing
                .add_directory(format!("Work/{which}"), plain)
                .expect("mark a folder");
        }
        writing
            .start_file("Work/one.eml", plain)
            .expect("start the one real entry");
        writing.write_all(one_message()).expect("write it");
        writing.finish().expect("finish the test zip");
        at
    }

    #[test]
    fn test_the_marks_an_archive_puts_on_its_own_folders_are_not_things_it_holds() {
        // A zip writes an entry with no bytes for each folder in it, so an
        // archive of fifty folders carries fifty of them. None holds any mail,
        // and each one kept would be a name held, a decision taken and a share
        // of what this program will open, spent on nothing.
        let place = a_place_to_work();

        let archive =
            opened(&a_zip_of_folder_marks(place.path(), 5)).expect("an archive of marks opens");

        assert_eq!(
            archive
                .what_it_holds()
                .iter()
                .map(|entry| entry.named)
                .collect::<Vec<_>>(),
            vec!["Work/one.eml"]
        );
    }

    #[test]
    fn test_an_archive_listing_more_things_than_will_be_opened_is_refused_before_reading_any() {
        // The count above only rises for entries that are kept, and the marks a
        // zip puts on its own folders are not kept. So a file listing a million
        // of those and one message would be a million turns of the loop before
        // it reached the one, which is the wait this refusal exists to prevent.
        let place = a_place_to_work();
        let allowing_three = HowMuchToAllow {
            most_things_in_it: 3,
            ..HowMuchToAllow::default()
        };

        let refused = opened_allowing(&a_zip_of_folder_marks(place.path(), 5), allowing_three)
            .err()
            .map(|why| why.to_string())
            .expect("an archive listing more than it may is refused");

        assert!(refused.contains("more than 3 things"), "{refused}");
    }

    #[test]
    fn test_an_archive_too_large_to_look_over_is_refused_rather_than_taken_into_memory() {
        // Looking over an archive keeps the opening of every entry in it at
        // once, which is the one part of reading an archive that grows with the
        // archive. A file with enough in it would fill this computer's memory
        // before anything had decided anything.
        let place = a_place_to_work();
        let held = a_few_entries(6);
        let allowing_a_little = HowMuchToAllow {
            most_held_while_looking_it_over: 200,
            ..HowMuchToAllow::default()
        };

        for at in [
            a_zip_of(place.path(), &as_entries(&held)),
            a_folder_of(place.path(), &as_entries(&held)),
        ] {
            let refused = opened_allowing(&at, allowing_a_little)
                .err()
                .map(|why| why.to_string())
                .unwrap_or_else(|| panic!("{} opened anyway", at.display()));

            assert!(refused.contains("too large"), "{refused}");
        }
    }

    /// One zip whose declared size for `named` is a lie, built by writing the
    /// entry properly and then changing only the number recorded beside it.
    ///
    /// What a check that believes the declared size cannot see coming. The
    /// entry looks whatever size the writer of the archive wanted it to look,
    /// and only unpacking it, a bounded amount at a time, says what it really
    /// is.
    fn a_zip_lying_about_a_size(
        inside: &Path,
        named: &str,
        real_bytes: &[u8],
        claiming: u64,
    ) -> PathBuf {
        let at = inside.join("lying.zip");
        let file = std::fs::File::create(&at).expect("create the test zip");
        let mut writing = zip::ZipWriter::new(file);
        writing
            .start_file(named, zip::write::SimpleFileOptions::default())
            .expect("start the entry");
        writing.write_all(real_bytes).expect("write the real bytes");
        // The bytes above were written and packed by this project's own writer,
        // so what is in the file is genuine. Only the size and the check
        // recorded beside them are overwritten, on purpose, which is the one
        // shape this test exists to catch. The check is the real one, so an
        // entry read the whole way through still reads back as itself.
        let mut check = flate2::Crc::new();
        check.update(real_bytes);
        unsafe {
            writing
                .set_file_metadata(claiming, check.sum())
                .expect("record a size the entry does not have");
        }
        writing.finish().expect("finish the test zip");
        at
    }

    #[test]
    fn test_an_entry_claiming_to_be_small_is_still_only_read_as_far_as_it_is_allowed() {
        // The attack this whole set of limits is about. A file of a few
        // kilobytes can unpack to more than any disk holds, and every size it
        // states about itself was written by whoever built it. So what is
        // counted is what really came out, and the reading stops at the limit
        // rather than at whatever the archive said would happen.
        let place = a_place_to_work();
        let far_too_much = vec![b'A'; 200_000];
        let at = a_zip_lying_about_a_size(place.path(), "Work/Bomb.mbox", &far_too_much, 10);
        let allowing_a_little = HowMuchToAllow {
            most_one_thing_unpacks_to: 1000,
            ..HowMuchToAllow::default()
        };

        let mut archive = opened_allowing(&at, allowing_a_little).expect("the archive opens");
        let refused = archive
            .one_entry_read_through("Work/Bomb.mbox")
            .err()
            .map(|why| why.to_string())
            .expect("an entry that unpacks past the limit is refused");

        assert!(refused.contains("Work/Bomb.mbox"), "{refused}");
        assert!(refused.contains("unpacks to more than"), "{refused}");
    }

    #[test]
    fn test_a_size_no_entry_could_have_is_not_believed_either_way() {
        // The same lie told the other way round, and the one that turns a
        // careless reader into a crash: an entry a few bytes long saying it
        // unpacks to more than this computer has. Believed, the number becomes
        // a refusal for an archive that is fine, or a request for that much
        // memory before a byte has been read.
        let place = a_place_to_work();
        let really = one_message();
        let at = a_zip_lying_about_a_size(place.path(), "Work/one.eml", really, u64::MAX / 2);

        let mut archive = opened_allowing(&at, HowMuchToAllow::default()).expect("it opens");

        assert_eq!(archive.what_it_holds()[0].opens_with, really);
        assert_eq!(
            archive
                .one_entry_read_through("Work/one.eml")
                .expect("an entry that fits is read"),
            really
        );
    }

    #[test]
    fn test_a_whole_archive_stops_at_the_total_it_is_allowed_to_unpack() {
        // The limit above is about one entry, and a thousand entries each just
        // under it add up to something no disk holds. This is the count that
        // has to be kept across the whole of an import rather than reset at
        // every entry.
        let place = a_place_to_work();
        let held = a_few_entries(6);
        let at = a_zip_of(place.path(), &as_entries(&held));
        let allowing_two_entries = HowMuchToAllow {
            most_the_whole_of_it_unpacks_to: (one_message().len() * 2) as u64,
            ..HowMuchToAllow::default()
        };

        let mut archive = opened_allowing(&at, allowing_two_entries).expect("the archive opens");
        let mut read_through = 0;
        let mut refused = String::new();
        for which in 0..6 {
            match archive.one_entry_read_through(&format!("Work/{which}.eml")) {
                Ok(_) => read_through += 1,
                Err(why) => {
                    refused = why.to_string();
                    break;
                }
            }
        }

        assert_eq!(read_through, 2, "{refused}");
        assert!(refused.contains("unpacks to more than"), "{refused}");
    }

    #[test]
    fn test_the_limits_this_program_ships_with_are_the_ones_written_down() {
        // The tests above hand this small numbers, because a limit nothing has
        // ever reached is a limit nobody has watched work. What they cannot say
        // is whether the numbers really shipped are the ones somebody chose, so
        // that is pinned here: generous for the largest mailbox anybody has
        // kept, and nowhere near what a hostile file asks for.
        let shipped = HowMuchToAllow::default();

        assert_eq!(shipped.most_things_in_it, 100_000);
        assert_eq!(shipped.most_held_while_looking_it_over, 512 * 1024 * 1024);
        assert_eq!(shipped.most_one_thing_unpacks_to, 1024 * 1024 * 1024);
        assert_eq!(
            shipped.most_the_whole_of_it_unpacks_to,
            20 * 1024 * 1024 * 1024
        );
    }

    #[test]
    fn test_a_size_is_said_in_a_unit_somebody_can_hear() {
        // These numbers go into a sentence that gets read out. Five hundred and
        // thirty six million eight hundred and seventy thousand nine hundred
        // and twelve is a number nobody can hold, and it is 512 megabytes.
        assert_eq!(said_as_a_size(512 * 1024 * 1024), "512 megabytes");
        assert_eq!(said_as_a_size(20 * 1024 * 1024 * 1024), "20 gigabytes");
        assert_eq!(said_as_a_size(1000), "1000 bytes");
    }

    #[test]
    fn test_an_archive_is_written_a_piece_at_a_time_and_reads_back_as_what_went_into_it() {
        // What lets a folder of forty thousand messages be written out on a
        // computer that could not hold two copies of it. The caller builds one
        // message, hands it over, and lets it go, so the mail is in memory once
        // on its way past rather than twice at rest.
        //
        // Read back through this module's own reader rather than looked at as
        // bytes, because what is being asked is whether the two agree: an
        // export nothing can open again is not an export.
        let place = a_place_to_work();
        let at = place.path().join("mailbox.zip");
        let long = a_long_mailbox();

        let mut writing = written_to(&at).expect("an archive can be started");
        writing
            .start_a_file("INBOX.mbox")
            .expect("a file can be started");
        for piece in long.chunks(1000) {
            writing.write_into_it(piece).expect("a piece goes in");
        }
        writing
            .start_a_file("Work/2024/one.eml")
            .expect("the next file can be started");
        writing
            .write_into_it(one_message())
            .expect("the next file goes in");
        writing.finish().expect("the archive can be finished");

        let mut written = opened(&at).expect("what was written opens again");
        assert_eq!(
            written
                .what_it_holds()
                .iter()
                .map(|entry| entry.named)
                .collect::<Vec<_>>(),
            vec!["INBOX.mbox", "Work/2024/one.eml"]
        );
        assert_eq!(
            written
                .one_entry_read_through("INBOX.mbox")
                .expect("the mail written out is read back"),
            long
        );
        assert_eq!(
            written
                .one_entry_read_through("Work/2024/one.eml")
                .expect("the mail written out is read back"),
            one_message()
        );
    }

    #[test]
    fn test_a_folder_with_nothing_in_it_is_written_as_a_folder_rather_than_left_out() {
        // An empty folder somebody has kept for years is part of the shape an
        // export exists to keep, and it cannot be written as an archive holding
        // no messages, because that is a file this program's own import turns
        // away as not mail. So it goes in as the folder and nothing else.
        let place = a_place_to_work();
        let at = place.path().join("mailbox.zip");

        let mut writing = written_to(&at).expect("an archive can be started");
        writing
            .add_a_folder_with_nothing_in_it("Receipts 2019")
            .expect("an empty folder goes in");
        writing
            .start_a_file("INBOX.mbox")
            .expect("a file can be started");
        writing
            .write_into_it(a_folder_of_mail())
            .expect("the mail goes in");
        writing.finish().expect("the archive can be finished");

        let read_back =
            zip::ZipArchive::new(std::fs::File::open(&at).expect("open what was written"))
                .expect("what was written is an archive");
        assert_eq!(
            read_back
                .file_names()
                .collect::<std::collections::BTreeSet<_>>(),
            ["INBOX.mbox", "Receipts 2019/"]
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>()
        );
    }

    #[test]
    fn test_nothing_is_written_anywhere_but_the_file_the_caller_named() {
        // The one promise a caller cannot check for itself. An export writes
        // where somebody chose, and nowhere else: no working copy beside it, no
        // half-written file left behind, nothing in a folder for temporary
        // things that fills a disk somebody never looks at.
        let place = a_place_to_work();
        let at = place.path().join("mailbox.zip");

        let mut writing = written_to(&at).expect("an archive can be started");
        writing.start_a_file("INBOX.mbox").expect("a file starts");
        writing
            .write_into_it(a_folder_of_mail())
            .expect("the mail goes in");
        writing.finish().expect("the archive can be finished");

        let left_behind: Vec<String> = std::fs::read_dir(place.path())
            .expect("list what the export left")
            .filter_map(|found| Some(found.ok()?.file_name().to_str()?.to_string()))
            .collect();
        assert_eq!(left_behind, vec!["mailbox.zip".to_string()]);
    }

    #[test]
    fn test_an_archive_that_cannot_be_written_says_so_before_anything_is_handed_to_it() {
        // A folder that is not there, a disk that is full, a name the
        // filesystem will not take. Found when the export is started rather
        // than when the last message goes in, so nobody watches a long export
        // run and then hears it failed.
        let place = a_place_to_work();
        let nowhere = place.path().join("no-such-folder").join("mailbox.zip");

        let refused = written_to(&nowhere)
            .err()
            .map(|why| why.to_string())
            .expect("an archive nothing can write is refused");

        assert!(refused.contains("could not be written"), "{refused}");
    }

    #[test]
    fn test_mail_handed_over_before_a_file_is_started_is_refused_rather_than_dropped() {
        // Somebody's mail handed to a writer with nowhere to put it. Taken
        // quietly, it goes nowhere, the export says it worked, and the messages
        // are missing from a file nobody opens again for a year.
        let place = a_place_to_work();
        let at = place.path().join("mailbox.zip");

        let mut writing = written_to(&at).expect("an archive can be started");
        let refused = writing
            .write_into_it(a_folder_of_mail())
            .err()
            .map(|why| why.to_string())
            .expect("mail with nowhere to go is refused");

        assert!(refused.contains("no file"), "{refused}");
    }

    #[test]
    fn test_the_name_an_archive_wrote_is_handed_over_as_it_stands_and_never_repaired() {
        // The one thing in an archive that decides where bytes get written, and
        // it is decided somewhere else. A name with its climbing steps taken
        // out here would arrive at the placer looking ordinary, and the placer
        // would file a stranger's mail under a name they chose, in silence.
        //
        // So every one of these comes out of here exactly as it went in, and
        // the refusals happen where the refusing is done.
        let place = a_place_to_work();
        let hostile: Vec<(&str, &[u8])> = vec![
            ("../../../Windows/System32/evil.mbox", one_message()),
            (r"..\..\somewhere.mbox", one_message()),
            ("Work/../../evil.mbox", one_message()),
            ("Work//Invoices.mbox", one_message()),
            ("NUL.mbox", one_message()),
            ("Work/In\u{202E}voices.mbox", one_message()),
        ];

        let archive = opened(&a_zip_of(place.path(), &hostile)).expect("it opens");

        assert_eq!(
            archive
                .what_it_holds()
                .iter()
                .map(|entry| entry.named)
                .collect::<Vec<_>>(),
            hostile.iter().map(|(named, _)| *named).collect::<Vec<_>>()
        );
    }

    /// One zip whose entry name is not text, built by writing a name with a
    /// letter outside plain English and then putting bytes in its place that
    /// are not a letter at all.
    ///
    /// Written that way because the writer will not take a name that is not
    /// text, and an archive written by something else can carry one.
    fn a_zip_with_a_name_that_is_not_text(inside: &Path) -> PathBuf {
        let at = inside.join("not-text.zip");
        let written = a_zip_of(inside, &[("W\u{f6}rk.mbox", a_folder_of_mail())]);
        let mut bytes = std::fs::read(&written).expect("read the test zip back");
        // The letter is two bytes and so is what replaces it, so every offset
        // recorded in the file still points where it did.
        let letter = "\u{f6}".as_bytes().to_vec();
        let mut from = 0;
        while let Some(found) = bytes[from..]
            .windows(letter.len())
            .position(|two| two == letter.as_slice())
        {
            let at = from + found;
            bytes[at] = 0xFF;
            bytes[at + 1] = 0xFE;
            from = at + letter.len();
        }
        std::fs::write(&at, &bytes).expect("write the zip with a name that is not text");
        at
    }

    #[test]
    fn test_an_entry_whose_name_is_not_text_can_still_be_asked_for_by_the_name_given_back() {
        // A name in an archive is whatever bytes somebody wrote, and it need
        // not be text at all. What matters is that the name handed over and the
        // name that finds the entry again are the same name: anything else is a
        // folder that appears in the plan and holds no mail when the time comes
        // to fill it.
        let place = a_place_to_work();
        let at = a_zip_with_a_name_that_is_not_text(place.path());

        let mut archive = opened(&at).expect("an archive with such a name still opens");
        let named = archive.what_it_holds()[0].named.to_string();

        assert!(
            named.contains('\u{fffd}'),
            "the name came back as text after all: {named:?}"
        );
        assert_eq!(
            archive
                .one_entry_read_through(&named)
                .expect("the name handed over finds the entry again"),
            a_folder_of_mail()
        );
    }

    /// One folder nested this many folders deep, with a message at the bottom.
    fn a_folder_nested(inside: &Path, deep: usize) -> PathBuf {
        let at = inside.join("mailbox");
        let mut down = at.clone();
        for _ in 0..deep {
            down.push("a");
        }
        std::fs::create_dir_all(&down).expect("make the nested folders");
        std::fs::write(down.join("deep.eml"), one_message()).expect("write the deep message");
        std::fs::write(at.join("shallow.eml"), one_message()).expect("write the shallow message");
        at
    }

    #[test]
    fn test_a_folder_deeper_than_this_program_follows_is_counted_rather_than_walked_forever() {
        // Both sides of the limit, because a limit tested from one side only is
        // one that could be anywhere. A real mailbox is a handful of folders
        // deep, and the mail below this could not have been imported in any
        // case, because the placer refuses a name that deep. What matters is
        // that a folder left behind is counted rather than left behind in
        // silence.
        let place = a_place_to_work();
        let as_deep_as_followed = a_folder_nested(&place.path().join("shallow"), MOST_FOLDERS_DEEP);
        let one_deeper = a_folder_nested(&place.path().join("deep"), MOST_FOLDERS_DEEP + 1);
        std::fs::create_dir_all(place.path().join("shallow")).expect("a place for each");

        let followed = opened(&as_deep_as_followed).expect("a folder at the limit opens");
        let stopped = opened(&one_deeper).expect("a folder past the limit still opens");

        assert_eq!(followed.what_it_holds().len(), 2);
        assert_eq!(followed.how_many_were_too_deep_to_follow(), 0);
        // The shallow message is still imported, and the one below the limit is
        // counted rather than quietly missing.
        assert_eq!(
            stopped
                .what_it_holds()
                .iter()
                .map(|entry| entry.named)
                .collect::<Vec<_>>(),
            vec!["shallow.eml"]
        );
        assert_eq!(stopped.how_many_were_too_deep_to_follow(), 1);
    }

    #[cfg(windows)]
    #[test]
    fn test_one_file_nothing_can_open_does_not_take_the_rest_of_the_folder_with_it() {
        // A folder somebody points at is a live part of their computer. A file
        // in it can be held open by the program that wrote it, be halfway
        // through a copy, or be gone between the moment the folder was listed
        // and the moment this got to it. Any of those abandoning the walk loses
        // the other eleven folders of mail with nothing said about which file
        // was the problem.
        //
        // Windows only, because holding a file open so nothing else can read it
        // is done differently on each platform, and this is the one this
        // program ships on. The code it exercises has nothing platform-specific
        // in it.
        use std::os::windows::fs::OpenOptionsExt;

        let place = a_place_to_work();
        let at = a_folder_of(place.path(), &some_mail());
        let held_open = at.join("Work").join("Locked.mbox");
        std::fs::write(&held_open, a_folder_of_mail()).expect("write the file to be held open");
        // No sharing at all, which is what a program that has a file open for
        // its own use asks for, and what makes every other open of it fail.
        let holding_it = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&held_open)
            .expect("hold the file open");

        let archive = opened(&at).expect("a folder with one unreadable file in it still opens");

        assert_eq!(
            archive
                .what_it_holds()
                .iter()
                .map(|entry| entry.named)
                .collect::<Vec<_>>(),
            vec!["Work/2024/one.eml", "Work/Invoices.mbox"]
        );
        assert_eq!(archive.how_many_could_not_be_read(), 1);
        drop(holding_it);
    }
}

#[cfg(test)]
mod checked_a_second_time {
    use super::*;

    /// A zip holding one entry of this many identical bytes.
    ///
    /// Written with the project's own writer, so what is on disk is a real
    /// archive rather than a hand-built fixture that only looks like one.
    fn a_zip_holding(bytes: usize) -> (tempfile::TempDir, std::path::PathBuf) {
        let folder = tempfile::tempdir().expect("a temporary folder");
        let at = folder.path().join("big.zip");
        let mut writing = written_to(&at).expect("a zip to write");
        writing.start_a_file("Inbox.mbox").expect("an entry");
        // Written in pieces so the test itself never holds it all at once.
        let piece = vec![b'A'; 64 * 1024];
        let mut left = bytes;
        while left > 0 {
            let this_time = left.min(piece.len());
            writing.write_into_it(&piece[..this_time]).expect("bytes");
            left -= this_time;
        }
        writing.finish().expect("a finished zip");
        (folder, at)
    }

    #[test]
    fn test_an_archive_that_would_not_fit_is_refused_rather_than_read() {
        // A zip of one kind of byte packs to almost nothing and unpacks to
        // whatever it likes, which is how a small file fills a disk. The
        // refusal has to happen while reading rather than from what the
        // archive says about itself, because a hostile file says whatever
        // suits it.
        //
        // Written against the rule rather than by whoever wrote it, and with
        // a real archive rather than one whose recorded sizes were edited.
        let (_folder, at) = a_zip_holding(4 * 1024 * 1024);
        let barely_anything = HowMuchToAllow {
            most_one_thing_unpacks_to: 64 * 1024,
            ..HowMuchToAllow::default()
        };

        let mut archive = opened_allowing(&at, barely_anything).expect("the zip opens");
        let named = archive.what_it_holds()[0].named.to_string();
        let read = archive.one_entry_read_through(&named);

        let why =
            read.expect_err("four megabytes were read where sixty four kilobytes were allowed");
        let said = why.to_string();
        assert!(
            said.to_lowercase().contains("large") || said.to_lowercase().contains("big"),
            "the refusal does not say what was wrong: {said}"
        );
    }

    #[test]
    fn test_an_archive_within_the_limits_is_still_read_whole() {
        // The other half. A rule that refuses everything is perfectly safe and
        // imports nothing, and this test is what stops the one above passing
        // that way.
        let (_folder, at) = a_zip_holding(128 * 1024);

        let mut archive = opened_allowing(&at, HowMuchToAllow::default()).expect("the zip opens");
        let named = archive.what_it_holds()[0].named.to_string();
        let read = archive
            .one_entry_read_through(&named)
            .expect("an ordinary entry was refused");

        assert_eq!(read.len(), 128 * 1024, "the entry came back short");
    }
}

//! An archive of somebody's whole mailbox, folders and all, and where its
//! folders land on this computer.
//!
//! [`crate::application::message_files`] reads one file of mail, and
//! [`crate::application::importing_messages`] decides where that one file's
//! messages go. Both answer about a single file, so an import files everything
//! into whichever folder happens to be on screen. That is the wrong answer for
//! the case most people arrive with: somebody leaving another mail program
//! exports the lot, and the shape their mail was in is half of what they are
//! bringing with them. Twelve folders arriving as one folder of forty thousand
//! messages is a mailbox nobody can find anything in, and for somebody working
//! through it by ear it is worse than not importing it at all.
//!
//! Values in, values out. Nothing here opens a file, unpacks an archive, talks
//! to a server or touches the database.
//!
//! # The archive is a stranger's file
//!
//! Every name in it was written by somebody else, and a name is the one thing
//! in an archive that decides where bytes get written. So a name here is
//! untrusted input that happens to look like a name, and the answer to one that
//! cannot be used is to refuse it, count it, and say so at the end.
//!
//! Refusing rather than repairing is the decision the safety of this module
//! rests on. A name with its climbing steps taken out still writes a stranger's
//! mail into a folder under a name they chose, and it does it in silence, so
//! the repaired form is not the safe form of the same import: it is a different
//! import nobody asked for. [`is_a_name_that_can_be_used`] is where that is
//! decided, and it asks the one function in this program that already knows
//! what a name Windows will take looks like, rather than answering it a second
//! time.
//!
//! # Where the folders go, and why not where somebody chose
//!
//! Under [`crate::application::local_folders::LOCAL_PREFIX`], so the mail
//! belongs to nobody's server and no sync can take it away again, and inside
//! one folder of its own under that. The folder of its own is not tidiness. An
//! account already keeps folders under that prefix, and an archive naming a
//! folder `Outbox` would otherwise land in the one folder mail must never be
//! imported into, where what is listed comes from the send queue and a message
//! filed there is written down, counted and never seen again. `Inbox` would tip
//! twenty years of somebody's archive in among the mail they have not read yet.
//!
//! Inside that folder the archive keeps its shape: `Work/Invoices` in the
//! archive is inside `Work` afterwards.
//!
//! # What a large archive costs
//!
//! A mailbox somebody has kept for twenty years is the largest thing this
//! program opens, and an archive of one holds several of them. Nothing here
//! holds any of it. An entry is placed from its opening bytes alone, and what
//! comes back names the entries that fill each folder rather than carrying
//! them, so whatever unpacks the archive works down one folder at a time and
//! lets each entry go again.

use crate::application::importing_messages::ReadAs;
use crate::application::local_folders::LOCAL_PREFIX;
use crate::application::message_files::{self, FileHolds};
use crate::application::summing_up::SummingUp;

/// What the folder holding every imported archive is called.
const IMPORTED_FOLDERS_ARE_UNDER: &str = "Imported";

/// The one folder imported archives are allowed to write inside.
///
/// Under the local prefix, so no server owns it and no sync can take the mail
/// away again, and inside a folder of its own rather than beside the folders an
/// account already keeps. An archive naming a folder `Outbox` would otherwise
/// land in the one folder mail must never be imported into, and one naming
/// `Inbox` would tip twenty years of somebody's archive in among the mail they
/// have not read yet.
pub fn where_imported_folders_go() -> String {
    format!("{LOCAL_PREFIX}/{IMPORTED_FOLDERS_ARE_UNDER}")
}

// ── What an archive of folders looks like ───────────────────────────────────

/// One thing an archive holds: what the archive calls it, and how it begins.
///
/// A zip file and a folder somebody points at both give these two facts and
/// nothing else that matters here, so the rest of this module never asks which
/// it came from.
///
/// Only the beginning of the entry, because that is all that is needed to tell
/// what it holds, and because the whole of an entry is somebody's mail: a
/// mailbox kept for twenty years does not fit in memory once, let alone all at
/// once beside a plan describing it. Whatever opens the archive reads
/// [`ENOUGH_TO_TELL_WHAT_IT_HOLDS`] bytes of each entry to work out where it
/// goes, then goes back and reads each entry through when the time comes to
/// file its mail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryInTheArchive<'a> {
    /// The path inside the archive, spelled however the archive spelled it.
    pub named: &'a str,
    /// The beginning of the entry, or all of it when it is shorter than that.
    pub opens_with: &'a [u8],
}

/// How much of an entry has to be read before it can be placed.
///
/// A message's headers come before its body, so what an entry holds is decided
/// within its opening bytes or not at all, and
/// [`message_files::what_the_file_holds`] stops looking at exactly this point.
/// The two numbers have to stay together: if that one ever grows, this one has
/// to grow with it, or an entry whose headers sit past this would be called mail
/// when read whole and not mail when read as far as here. There is a test that
/// an entry answers the same either way, and it can only ask that of the entries
/// it is given.
pub const ENOUGH_TO_TELL_WHAT_IT_HOLDS: usize = 64 * 1024;

/// Where the mail in one entry of an archive goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhereItGoes {
    /// Into this folder on this computer, read this much at a time.
    Into { folder: String, read: ReadAs },
    /// Nowhere. There is no mail in it.
    ///
    /// An export carries its own index, settings and address book in among the
    /// mail. Counted and said at the end rather than passed over in silence,
    /// because an archive whose mail this program could not recognise and one
    /// holding no mail at all look identical from the outside.
    NotMail,
    /// Nowhere. The archive's name for it is not one this program will write.
    ///
    /// Refused rather than repaired, and that is the decision the whole of the
    /// safety of this module rests on. A name with its climbing steps taken out
    /// still writes a stranger's mail into a folder under a name they chose,
    /// and it does it in silence. So the mail stays in the archive, the entry
    /// is counted, and the count is said at the end.
    NameRefused,
}

/// The two characters that put one folder inside another.
///
/// Both, whichever built this program. An archive is a stranger's file and the
/// name inside it is whatever they wrote, so a name carrying either character
/// has to be read as a path here, on any platform, rather than left for the one
/// place that happens to be running to decide.
const SEPARATES_FOLDERS: [char; 2] = ['/', '\\'];

/// The endings a file holding a whole folder of mail is usually given.
///
/// Taken off so that `Invoices.mbox` becomes a folder called Invoices rather
/// than one called Invoices.mbox, which is a name that means nothing to anybody
/// once the mail is out of the file it came in. Only these two, and only on the
/// last part of the name: an ending nobody recognises is left alone, because a
/// folder somebody called `Notes.old` is called that on purpose.
const A_MAILBOX_FILE_ENDS_WITH: [&str; 2] = [".mbox", ".mbx"];

/// Where one entry of an archive lands, from its name and its opening bytes.
///
/// What the entry holds is decided from the bytes, which is
/// [`message_files`]' rule and holds here for the same reason: an export names
/// its files whatever it likes, and its index and settings files sit in the
/// same folders as the mail.
pub fn where_one_entry_lands(named: &str, opens_with: &[u8]) -> WhereItGoes {
    // What it holds first, so that an archive's own furniture is reported as
    // furniture. A zip marks its folders with an entry of no bytes and a name
    // ending in a separator, and that name would otherwise be counted as one
    // somebody has to be warned about.
    let read = match message_files::what_the_file_holds(opens_with) {
        FileHolds::ManyMessages => ReadAs::OneAtATimeFromAnArchive,
        FileHolds::OneMessage => ReadAs::OneMessage,
        FileHolds::NotMail => return WhereItGoes::NotMail,
    };
    let Some(parts) = the_folder_named_by(named, read) else {
        return WhereItGoes::NameRefused;
    };
    WhereItGoes::Into {
        folder: the_path_under_the_import_area(&parts),
        read,
    }
}

/// The most folders deep an archive is followed.
///
/// A real mailbox is a handful of folders deep. Anything past this is either a
/// mistake or somebody seeing what happens, and the path it would build is one
/// nothing can open and nobody could read out.
const MOST_FOLDERS_DEEP: usize = 24;

/// The longest name an archive may give one of its entries, all of it together.
///
/// Each part of a name has a limit of its own, and a name made of two dozen
/// short parts passes every one of them and still comes out longer than the two
/// hundred and sixty characters Windows will open without being asked specially.
/// Two hundred and forty leaves room under that for the folder somebody later
/// writes this mail out into.
const MOST_IN_A_NAME_ALTOGETHER: usize = 240;

/// The parts of the folder an entry's mail goes in, or nothing when the name is
/// one this program will not write.
///
/// No parts at all is an answer rather than a refusal: it is one saved message
/// sitting at the top of the archive, and it goes into the import folder
/// itself.
fn the_folder_named_by(named: &str, read: ReadAs) -> Option<Vec<&str>> {
    // An archive that names an entry nothing at all has nothing to file it
    // under, and every later rule here is about the parts of a name.
    if named.is_empty() || named.chars().count() > MOST_IN_A_NAME_ALTOGETHER {
        return None;
    }
    let mut parts: Vec<&str> = named.split(SEPARATES_FOLDERS).collect();
    if parts.len() > MOST_FOLDERS_DEEP {
        return None;
    }
    // A step out of a folder is refused wherever it stands in the name,
    // including in the part that names the file itself. That part is dropped
    // rather than used to build anything for a saved message, so a step hidden
    // there is one nothing further down would ever look at, and `Work/..` would
    // quietly file its message in Work instead of where its name really points.
    if parts.iter().any(|part| is_a_step_out_of_a_folder(part)) {
        return None;
    }
    match read {
        // A file of many messages is a whole folder, so it becomes one.
        ReadAs::OneAtATimeFromAnArchive => {
            if let Some(last) = parts.last_mut() {
                *last = without_a_mailbox_ending(last);
            }
        }
        // A file of one message is a message in the folder it was sitting in,
        // and not a folder of its own. An export of separate messages is a
        // folder of them, and the other reading gives somebody four hundred
        // folders holding one message each.
        ReadAs::OneMessage => {
            parts.pop();
        }
    }
    // Every part as it will be written, after the ending came off rather than
    // before. `..mbox` loses its ending and is left as `..`, which is a step
    // out of the import area that was not there in the name the archive gave.
    parts
        .iter()
        .all(|part| is_a_name_that_can_be_used(part))
        .then_some(parts)
}

/// Whether one part of a name is a step out of the folder it sits in.
///
/// `.` is the folder itself and `..` is the one above it, and a run of dots of
/// any length is one or the other on some filesystem somewhere. Asked of the
/// name the archive gave, before the part naming the file is dropped, which is
/// the one place the rule below cannot reach.
fn is_a_step_out_of_a_folder(part: &str) -> bool {
    !part.is_empty() && part.chars().all(|letter| letter == '.')
}

/// Whether one part of a name is usable exactly as the archive wrote it.
///
/// Asked of the one function in this program that already knows what a name
/// Windows will accept looks like, rather than answered a second time here. That
/// function repairs a name; what is wanted here is whether a name needed
/// repairing, and a name that came back unchanged is one that needed none. So a
/// step out of a folder, a device like `NUL`, a trailing dot the filesystem
/// would strip after this had finished checking, a character Windows will not
/// take, a name written backwards by an override, and a name too long to write
/// are all one question with one answer.
///
/// The names this refuses are pinned by tests here as well, because this
/// module's promise has to hold whatever that function is later asked to allow.
///
/// Reachable from the rest of the crate because a folder somebody makes on this
/// computer asks the same question, and a second answer to it is how the two
/// drift apart. It is asked there of one part of a name at a time, because that
/// module lets a name hold the character it nests with and this one does not.
pub(crate) fn is_a_name_that_can_be_used(part: &str) -> bool {
    !part.is_empty() && crate::service::attachment_name::safe_file_name(part) == part
}

/// A name with the ending a mailbox file carries taken off.
///
/// The comparison folds ASCII only, and it folds the ending in place rather than
/// lowercasing the name first. Lowercasing can change how many bytes a name
/// takes, so a name lowercased to find the ending and then cut by the ending's
/// length is a cut that can land in the middle of a character.
///
/// A name that is nothing but the ending keeps it, since a folder has to be
/// called something.
fn without_a_mailbox_ending(part: &str) -> &str {
    for ending in A_MAILBOX_FILE_ENDS_WITH {
        let Some(before_the_ending) = part.len().checked_sub(ending.len()) else {
            continue;
        };
        if before_the_ending > 0
            && part.is_char_boundary(before_the_ending)
            && part[before_the_ending..].eq_ignore_ascii_case(ending)
        {
            return &part[..before_the_ending];
        }
    }
    part
}

// ── The whole archive, a folder at a time ───────────────────────────────────

/// One folder to make, and the entries whose mail fills it.
///
/// The entries are named rather than carried. Whatever opens the archive reads
/// each one when it reaches it and lets it go again, so a mailbox larger than
/// this computer's memory is imported by working down this list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderToFill {
    /// Where it goes on this computer.
    pub path: String,
    /// The entries that fill it, in the order the archive held them.
    pub entries: Vec<EntryToRead>,
}

/// One entry to read, and how much of it to read at a time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryToRead {
    /// What the archive calls it, so it can be found again.
    pub named: String,
    /// How much of it is read at once.
    pub read: ReadAs,
}

/// What importing an archive of folders did.
///
/// Filled in twice: everything but the messages is known once the archive has
/// been looked over, and the messages are counted as the folders are filled.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FoldersImported {
    /// Folders made on this computer.
    pub folders: usize,
    /// Messages written into them.
    pub messages: usize,
    /// Files in the archive that held no mail.
    pub held_no_mail: usize,
    /// Files left in the archive, because of the name it gave them.
    pub names_refused: usize,
    /// Folders in the archive that would have had the same name here as one
    /// already made, so their mail was filed into that one.
    pub filed_together: usize,
}

/// The folders an archive turns into, and what to say about the rest of it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HowTheFoldersLand {
    /// The folders to make and fill, in the order the archive named them.
    pub folders: Vec<FolderToFill>,
    /// Everything worked out so far, for the sentence at the end.
    pub counted: FoldersImported,
}

/// Where every folder in an archive lands, and what is left behind.
///
/// The order is the archive's own, so somebody watching a long import hears
/// their folders in the order they know them in.
pub fn where_the_folders_land(entries: &[EntryInTheArchive<'_>]) -> HowTheFoldersLand {
    let mut folders: Vec<FolderToFill> = Vec::new();
    let mut which_folder: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut spellings: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut counted = FoldersImported::default();

    for entry in entries {
        // A zip marks each of its folders with an entry of no bytes, so an
        // archive of fifty folders carries fifty of these. Passed over rather
        // than counted: told that fifty files in their archive were not mail,
        // somebody would go looking for fifty missing folders.
        if entry.opens_with.is_empty() {
            continue;
        }
        match where_one_entry_lands(entry.named, entry.opens_with) {
            WhereItGoes::NotMail => counted.held_no_mail += 1,
            WhereItGoes::NameRefused => counted.names_refused += 1,
            WhereItGoes::Into { folder, read } => {
                let reading = EntryToRead {
                    named: entry.named.to_string(),
                    read,
                };
                let same_folder_however_spelled = however_it_is_spelled(&folder);
                spellings.insert(folder.clone());
                match which_folder.get(&same_folder_however_spelled) {
                    Some(already) => folders[*already].entries.push(reading),
                    None => {
                        which_folder.insert(same_folder_however_spelled, folders.len());
                        folders.push(FolderToFill {
                            path: folder,
                            entries: vec![reading],
                        });
                    }
                }
            }
        }
    }

    counted.folders = folders.len();
    // Every folder made contributed the first spelling of its own name, so
    // anything past that count is a second spelling of a name already here.
    // Never negative, and `saturating_sub` rather than a subtraction that would
    // be a panic in a debug build and a vast number in a release one if that
    // reasoning were ever wrong.
    counted.filed_together = spellings.len().saturating_sub(folders.len());
    HowTheFoldersLand { folders, counted }
}

/// A folder path with the differences that are not differences taken out.
///
/// Case only. `Work` and `work` are one folder to Windows and, read aloud, one
/// folder to anybody: two of them in the tree is two rows nobody listening can
/// tell apart, so the mail goes into the first of them and the count says how
/// many were filed that way.
///
/// Named for what it answers rather than for what it does to the text. It was
/// called `folded`, which is what the calendar service calls breaking a line
/// too long to send, and one word for two questions in a program that asks
/// both is a word that will be read as the wrong one.
fn however_it_is_spelled(path: &str) -> String {
    path.to_lowercase()
}

/// The path on this computer these parts of an archive's name make.
///
/// The only place a folder path is built here, and it always starts from the
/// import area, so there is nowhere for a path outside that area to come from.
fn the_path_under_the_import_area(parts: &[&str]) -> String {
    let mut path = where_imported_folders_go();
    for part in parts {
        path.push('/');
        path.push_str(part);
    }
    path
}

// ── Saying what the import did ──────────────────────────────────────────────

/// What importing an archive of folders did, in the words somebody hears.
///
/// The counts that are not nought are the ones worth saying, and each one that
/// is gets a whole sentence, because each names something somebody has to decide
/// what to do about: mail still sitting in their archive, or a folder they will
/// go looking for and not find.
pub fn what_the_folder_import_did(imported: &FoldersImported) -> String {
    let mut said = SummingUp::opening(match imported.folders {
        0 => "No folders were imported".to_string(),
        1 => "Imported 1 folder".to_string(),
        many => format!("Imported {many} folders"),
    });
    // Only beside a folder count that means something. "No folders were
    // imported, 0 messages" says one thing twice and buries the sentence after
    // it that says why.
    if imported.folders > 0 {
        said.count(match imported.messages {
            1 => "1 message".to_string(),
            many => format!("{many} messages"),
        });
    }
    // Before the two counts of what was left out, because this one is about
    // folders that did arrive and is the answer to "why are there fewer folders
    // than I had".
    if imported.filed_together > 0 {
        // Two sentences written out rather than one built from parts. Several
        // words have to agree in number, and a sentence assembled from
        // fragments reads like one.
        said.sentence(match imported.filed_together {
            1 => "1 folder in the archive would have had the same name here as \
                  another, so their mail was filed together"
                .to_string(),
            many => format!(
                "{many} folders in the archive would have had the same name here \
                 as others, so their mail was filed together"
            ),
        });
    }
    if imported.held_no_mail > 0 {
        said.sentence(match imported.held_no_mail {
            1 => "1 file in the archive was not mail and was left out".to_string(),
            many => format!("{many} files in the archive were not mail and were left out"),
        });
    }
    if imported.names_refused > 0 {
        said.sentence(match imported.names_refused {
            1 => "1 file was left out, because the archive gives it a name that \
                  cannot be used on this computer"
                .to_string(),
            many => format!(
                "{many} files were left out, because the archive gives them names \
                 that cannot be used on this computer"
            ),
        });
    }
    // An archive nothing at all was found in. "No folders were imported" on its
    // own is what a broken import says too, and somebody who cannot tell those
    // apart goes looking for a broken program rather than at their file.
    if imported.folders == 0 && imported.held_no_mail == 0 && imported.names_refused == 0 {
        said.sentence("There is nothing in this archive that reads as mail");
    }
    said.spoken()
}

/// What somebody pointed at when they asked to import.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatWasChosen {
    /// A zip file or a folder: folders of mail laid out inside it.
    AnArchive,
    /// One file holding mail, which is a folder of one.
    MailInOneFile,
}

/// Which of the two this is, decided from how it begins.
///
/// From the bytes rather than the name, which is this module's rule throughout
/// and holds here for the same reason: mail is saved with every ending there
/// is and with none, and a zip somebody renamed is still a zip.
///
/// This exists because the two readers refuse each other. A single saved
/// message handed to the archive reader is a file that does not begin the way
/// a zip begins, so it comes back as not an archive, which is true and is not
/// what somebody who picked one message needs to hear.
pub fn what_was_chosen(a_folder: bool, opens_with: &[u8]) -> WhatWasChosen {
    if a_folder {
        return WhatWasChosen::AnArchive;
    }
    match message_files::what_the_file_holds(opens_with) {
        FileHolds::NotMail => WhatWasChosen::AnArchive,
        FileHolds::OneMessage | FileHolds::ManyMessages => WhatWasChosen::MailInOneFile,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_saved_message_is_not_sent_to_the_archive_reader() {
        // The regression this was written for. Handing a single saved message
        // to the reader that opens zip files gets "this is not an archive",
        // which is true and useless: somebody picked one message and is told
        // their file is the wrong kind.
        assert_eq!(
            what_was_chosen(false, b"From: a@example.com\r\nSubject: Hi\r\n\r\nBody\r\n"),
            WhatWasChosen::MailInOneFile
        );
    }

    #[test]
    fn test_a_zip_and_a_folder_both_go_to_the_archive_reader() {
        // A zip opens `PK`, which is not how mail opens, so it falls through
        // to the archive reader without anything here knowing what a zip is.
        assert_eq!(
            what_was_chosen(false, b"PK\x03\x04rest of a zip"),
            WhatWasChosen::AnArchive
        );
        assert_eq!(what_was_chosen(true, b""), WhatWasChosen::AnArchive);
    }

    #[test]
    fn test_something_that_is_neither_goes_to_the_archive_reader() {
        // A picture, a document, an empty file. The archive reader is the one
        // with a sentence for each way a file can fail to be an archive, and
        // saying "not mail" here would throw that away.
        assert_eq!(
            what_was_chosen(false, &[0x89, b'P', b'N', b'G']),
            WhatWasChosen::AnArchive
        );
    }

    /// One ordinary message, as a file saved from a mail program holds it.
    fn one_message() -> &'static str {
        concat!(
            "From: Ada Lovelace <ada@example.com>\r\n",
            "To: Charles Babbage <charles@example.com>\r\n",
            "Subject: Notes on the engine\r\n",
            "\r\n",
            "The engine weaves algebraic patterns.\r\n",
        )
    }

    /// Two messages in the format an archive of one folder uses.
    fn a_folder_of_mail() -> &'static str {
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
    }

    /// The folder an entry lands in, for a test that expects it to land.
    fn lands_in(named: &str, bytes: &[u8]) -> String {
        match where_one_entry_lands(named, bytes) {
            WhereItGoes::Into { folder, .. } => folder,
            went_nowhere => panic!("{named} went nowhere: {went_nowhere:?}"),
        }
    }

    #[test]
    fn test_a_name_that_tries_to_climb_out_of_the_import_area_is_refused() {
        // The one thing in this module that can do real harm. An archive is a
        // stranger's file and the names inside it are whatever they wrote, so
        // every one of these has to stop here rather than at whatever writes
        // the mail down.
        //
        // Refused, and never repaired into something else. A name that has had
        // its climbing steps taken out still writes a stranger's mail into a
        // folder under a name they chose, and it does it silently.
        for hostile in [
            "../../../Windows/System32/evil",
            "/etc/passwd",
            r"..\..\somewhere",
            "..",
            ".",
            "Work/../../evil",
            r"C:\Windows\System32\evil",
            "C:evil",
            r"\\server\share\evil",
            "Work//Invoices",
            "Work/",
            "\u{0}evil",
            "",
        ] {
            assert_eq!(
                where_one_entry_lands(hostile, a_folder_of_mail().as_bytes()),
                WhereItGoes::NameRefused,
                "{hostile:?} was not refused"
            );
        }
    }

    #[test]
    fn test_an_archive_worked_through_from_end_to_end_says_what_it_did() {
        // The pieces together rather than one at a time. An archive is looked
        // over, each folder is filled from the entries the plan names for it,
        // and the counts that come out are the sentence somebody hears. Nothing
        // in this loop holds more than one entry's mail at once, which is the
        // shape whatever unpacks a real archive has to keep to.
        let an_index_file = &b"// <mdb:mork:z v=\"1.4\"/>\n"[..];
        let archive = [
            entry("Work/", b""),
            entry("Work/Invoices.mbox", a_folder_of_mail().as_bytes()),
            entry("Work/2024/one.eml", one_message().as_bytes()),
            entry("work/Invoices.mbox", a_folder_of_mail().as_bytes()),
            entry("Work/INBOX.msf", an_index_file),
            entry(
                "../../../Windows/System32/evil.mbox",
                a_folder_of_mail().as_bytes(),
            ),
        ];

        let mut landing = where_the_folders_land(&archive);
        for folder in &landing.folders {
            for to_read in &folder.entries {
                let bytes = archive
                    .iter()
                    .find(|held| held.named == to_read.named)
                    .expect("the plan names entries the archive holds")
                    .opens_with;
                landing.counted.messages += match to_read.read {
                    ReadAs::OneMessage => {
                        usize::from(message_files::read_one_message(bytes).is_ok())
                    }
                    ReadAs::OneAtATimeFromAnArchive => message_files::each_message_read_from(bytes)
                        .filter(|message| message.is_ok())
                        .count(),
                };
            }
        }

        let where_imported = where_imported_folders_go();
        assert_eq!(
            landing
                .folders
                .iter()
                .map(|folder| folder.path.as_str())
                .collect::<Vec<_>>(),
            vec![
                format!("{where_imported}/Work/Invoices"),
                format!("{where_imported}/Work/2024"),
            ]
        );
        assert_eq!(
            what_the_folder_import_did(&landing.counted),
            "Imported 2 folders, 5 messages. 1 folder in the archive would have \
             had the same name here as another, so their mail was filed together. \
             1 file in the archive was not mail and was left out. 1 file was left \
             out, because the archive gives it a name that cannot be used on this \
             computer."
        );
    }

    #[test]
    fn test_an_archive_with_no_mail_in_it_says_so_rather_than_reporting_nothing() {
        // Somebody picks the wrong file. "No folders were imported" on its own
        // is what a broken import says as well, and the difference between the
        // two is the whole of what somebody needs to decide what to do next.
        assert_eq!(
            what_the_folder_import_did(&FoldersImported::default()),
            "No folders were imported. There is nothing in this archive that reads as mail."
        );
    }

    #[test]
    fn test_one_folder_and_one_message_are_said_in_the_singular() {
        // Read aloud, "Imported 1 folders, 1 messages" is the kind of sentence
        // that makes somebody stop trusting the ones that matter.
        assert_eq!(
            what_the_folder_import_did(&FoldersImported {
                folders: 1,
                messages: 1,
                ..FoldersImported::default()
            }),
            "Imported 1 folder, 1 message"
        );
        assert_eq!(
            what_the_folder_import_did(&FoldersImported {
                folders: 12,
                messages: 4300,
                ..FoldersImported::default()
            }),
            "Imported 12 folders, 4300 messages"
        );
    }

    #[test]
    fn test_files_that_held_no_mail_are_counted_out_loud() {
        // An export carries its own index, settings and address book among the
        // mail. Passed over in silence, an archive whose mail this program could
        // not recognise looks exactly like one holding no mail at all.
        assert_eq!(
            what_the_folder_import_did(&FoldersImported {
                folders: 3,
                messages: 40,
                held_no_mail: 1,
                ..FoldersImported::default()
            }),
            "Imported 3 folders, 40 messages. 1 file in the archive was not mail \
             and was left out."
        );
    }

    #[test]
    fn test_a_name_that_was_refused_is_said_rather_than_left_to_the_log() {
        // The mail is still in their archive and nothing here can get at it, so
        // this is the sentence that tells them to rename the folder and try
        // again rather than wonder where it went.
        assert_eq!(
            what_the_folder_import_did(&FoldersImported {
                folders: 3,
                messages: 40,
                names_refused: 1,
                ..FoldersImported::default()
            }),
            "Imported 3 folders, 40 messages. 1 file was left out, because the \
             archive gives it a name that cannot be used on this computer."
        );
    }

    #[test]
    fn test_folders_filed_together_are_said_so_the_missing_ones_are_accounted_for() {
        // Somebody counts twelve folders in their archive and eleven here.
        // Without this they have no way to find out which one went and why.
        assert_eq!(
            what_the_folder_import_did(&FoldersImported {
                folders: 11,
                messages: 40,
                filed_together: 1,
                ..FoldersImported::default()
            }),
            "Imported 11 folders, 40 messages. 1 folder in the archive would have \
             had the same name here as another, so their mail was filed together."
        );
    }

    #[test]
    fn test_everything_that_can_be_said_at_once_is_still_one_run_of_sentences() {
        // The punctuation belongs to `summing_up` and this is where it is asked
        // for. Sentences pushed on to each other arrive as "computer.. 2 files",
        // which on screen is a typo and read aloud is a stutter followed by a
        // fragment.
        let said = what_the_folder_import_did(&FoldersImported {
            folders: 11,
            messages: 4300,
            held_no_mail: 2,
            names_refused: 3,
            filed_together: 2,
        });

        assert_eq!(
            said,
            "Imported 11 folders, 4300 messages. 2 folders in the archive would \
             have had the same name here as others, so their mail was filed \
             together. 2 files in the archive were not mail and were left out. \
             3 files were left out, because the archive gives them names that \
             cannot be used on this computer."
        );
    }

    #[test]
    fn test_everything_this_module_says_is_a_sentence_and_names_no_machinery() {
        // All of it is read aloud. A fragment with no stop on the end runs into
        // whatever is spoken next, and a sentence naming a mechanism tells
        // somebody about this program's insides instead of about their mail.
        let everything = [
            what_the_folder_import_did(&FoldersImported::default()),
            what_the_folder_import_did(&FoldersImported {
                folders: 1,
                messages: 1,
                held_no_mail: 1,
                names_refused: 1,
                filed_together: 1,
            }),
            what_the_folder_import_did(&FoldersImported {
                folders: 11,
                messages: 4300,
                held_no_mail: 2,
                names_refused: 3,
                filed_together: 2,
            }),
        ];

        for said in &everything {
            assert!(!said.trim().is_empty(), "something said nothing at all");
            assert!(said.ends_with('.'), "not a sentence: {said}");
            let lowered = said.to_lowercase();
            for machinery in [
                "imap",
                "pop3",
                "uid",
                "database",
                "cache",
                "sync",
                "parse",
                "header",
                "mbox",
                ".eml",
                "row",
                "zip",
                "path",
                "directory",
                "entry",
                "prefix",
            ] {
                assert!(
                    !lowered.contains(machinery),
                    "this names {machinery}, which is a mechanism and not what happens: {said}"
                );
            }
        }
    }

    /// One entry of an archive, for a test that does not care about its bytes
    /// beyond what they hold.
    fn entry<'a>(named: &'a str, opens_with: &'a [u8]) -> EntryInTheArchive<'a> {
        EntryInTheArchive { named, opens_with }
    }

    #[test]
    fn test_an_archive_becomes_a_folder_at_a_time_with_the_entries_that_fill_each() {
        // What a caller works through. A folder and the names of the entries
        // that fill it, so the mail itself is opened one entry at a time and let
        // go again: a mailbox somebody has kept for twenty years does not fit in
        // memory, and a plan holding all of it would be the same mistake as
        // reading it all in.
        let index_file = &b"// <mdb:mork:z v=\"1.4\"/>\n"[..];
        let archive = [
            entry("Work/Invoices.mbox", a_folder_of_mail().as_bytes()),
            entry("Work/2024/one.eml", one_message().as_bytes()),
            entry("Work/INBOX.msf", index_file),
            entry("Work/2024/two.eml", one_message().as_bytes()),
        ];

        let landing = where_the_folders_land(&archive);

        let where_imported = where_imported_folders_go();
        assert_eq!(
            landing.folders,
            vec![
                FolderToFill {
                    path: format!("{where_imported}/Work/Invoices"),
                    entries: vec![EntryToRead {
                        named: "Work/Invoices.mbox".to_string(),
                        read: ReadAs::OneAtATimeFromAnArchive,
                    }],
                },
                FolderToFill {
                    path: format!("{where_imported}/Work/2024"),
                    entries: vec![
                        EntryToRead {
                            named: "Work/2024/one.eml".to_string(),
                            read: ReadAs::OneMessage,
                        },
                        EntryToRead {
                            named: "Work/2024/two.eml".to_string(),
                            read: ReadAs::OneMessage,
                        },
                    ],
                },
            ]
        );
        assert_eq!(
            landing.counted,
            FoldersImported {
                folders: 2,
                held_no_mail: 1,
                ..FoldersImported::default()
            }
        );
    }

    #[test]
    fn test_the_marks_an_archive_puts_on_its_own_folders_are_passed_over_in_silence() {
        // A zip writes an entry with no bytes and a name ending in a separator
        // for each folder it holds, so an archive of fifty folders carries fifty
        // of them. Counted as files that held no mail, the sentence at the end
        // would send somebody looking for fifty folders of missing mail.
        let archive = [
            entry("Work/", b""),
            entry("Work/2024/", b""),
            entry("Work/Invoices.mbox", a_folder_of_mail().as_bytes()),
        ];

        let landing = where_the_folders_land(&archive);

        assert_eq!(
            landing.counted,
            FoldersImported {
                folders: 1,
                ..FoldersImported::default()
            }
        );
    }

    #[test]
    fn test_the_opening_of_an_entry_places_it_where_the_whole_of_it_would() {
        // What lets a caller look over an archive without reading it. Every
        // entry is placed from its first few thousand bytes, so the mail itself
        // is opened once, when its folder is filled, rather than once to decide
        // and once to file.
        let mut a_long_mailbox = a_folder_of_mail().to_string();
        a_long_mailbox.push_str(&"A line of somebody's message.\r\n".repeat(4000));
        let mut a_long_message = one_message().to_string();
        a_long_message.push_str(&"A line of somebody's message.\r\n".repeat(4000));

        for whole in [a_long_mailbox, a_long_message] {
            let whole = whole.as_bytes();
            assert!(whole.len() > ENOUGH_TO_TELL_WHAT_IT_HOLDS);

            assert_eq!(
                where_one_entry_lands("Work/Invoices.mbox", &whole[..ENOUGH_TO_TELL_WHAT_IT_HOLDS]),
                where_one_entry_lands("Work/Invoices.mbox", whole),
                "the opening and the whole entry disagree about where it goes"
            );
        }
    }

    #[test]
    fn test_folders_whose_names_differ_only_in_capitals_are_filed_as_one() {
        // A decision rather than an accident. Kept apart, `Work` and `work` are
        // two rows in the tree that read aloud as the same words, and the person
        // this program is for has no way to tell which of them holds what.
        // Windows would make them one folder anyway.
        //
        // The first spelling the archive used is the one kept, and how many were
        // folded into it is counted, so nobody has to work out afterwards why
        // there are fewer folders than there were.
        let archive = [
            entry("Work/Invoices.mbox", a_folder_of_mail().as_bytes()),
            entry("work/Invoices.mbox", a_folder_of_mail().as_bytes()),
            entry("WORK/Invoices.mbox", a_folder_of_mail().as_bytes()),
            entry("work/Invoices/extra.eml", one_message().as_bytes()),
        ];

        let landing = where_the_folders_land(&archive);

        assert_eq!(
            landing.folders.len(),
            1,
            "{:?}",
            landing
                .folders
                .iter()
                .map(|to| &to.path)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            landing.folders[0].path,
            format!("{}/Work/Invoices", where_imported_folders_go())
        );
        assert_eq!(landing.folders[0].entries.len(), 4);
        assert_eq!(landing.counted.folders, 1);
        assert_eq!(landing.counted.filed_together, 2);
    }

    #[test]
    fn test_a_mailbox_file_and_a_folder_of_messages_sharing_a_name_are_one_folder() {
        // The other way two things in an archive come to one folder here, and it
        // is not a fold: an export writes `Invoices.mbox` beside an `Invoices`
        // folder of separate messages, and once the ending is off they are the
        // same name rather than two names that could not both be kept. So the
        // mail goes together and nothing is counted as filed together, because
        // nothing was.
        let archive = [
            entry("Invoices.mbox", a_folder_of_mail().as_bytes()),
            entry("Invoices/one.eml", one_message().as_bytes()),
        ];

        let landing = where_the_folders_land(&archive);

        assert_eq!(landing.folders.len(), 1);
        assert_eq!(
            landing.folders[0].path,
            format!("{}/Invoices", where_imported_folders_go())
        );
        assert_eq!(landing.counted.filed_together, 0);
    }

    /// Every name a stranger might put in an archive, built from the pieces
    /// that have each broken something somewhere.
    ///
    /// Built rather than listed, because the pieces are dangerous in
    /// combination and a list written by hand only holds the combinations
    /// somebody thought of. Two of these were a surprise: an ending taken off
    /// `..mbox` leaves a step out of the import area that was not in the name
    /// the archive gave, and a single message named `..` files itself in the
    /// folder above the one its own name asked for.
    fn names_a_stranger_might_write() -> Vec<String> {
        const PIECES: [&str; 14] = [
            "..",
            ".",
            "",
            "Work",
            "/",
            "\\",
            "C:",
            "NUL",
            "con.mbox",
            " ",
            "x.",
            "\u{0}",
            "\u{202E}",
            "Invoices.mbox",
        ];
        let mut names = Vec::new();
        for first in PIECES {
            for second in PIECES {
                for third in PIECES {
                    names.push(format!("{first}{second}{third}"));
                    names.push(format!("{first}/{second}/{third}"));
                }
            }
        }
        names
    }

    #[test]
    fn test_no_name_an_archive_can_carry_reaches_outside_the_area_imports_write_to() {
        // The promise everything wiring this up depends on, asked of thousands
        // of names at once rather than of the handful anybody would think to
        // write down. Whatever the archive says, the answer is either that the
        // mail stays where it is or a path inside one folder, made only of
        // names this computer will accept.
        let where_imported = where_imported_folders_go();
        let inside_it = format!("{where_imported}/");

        for named in names_a_stranger_might_write() {
            for opens_with in [
                a_folder_of_mail().as_bytes(),
                one_message().as_bytes(),
                &b"\x89PNG\r\n\x1a\n"[..],
            ] {
                let WhereItGoes::Into { folder, .. } = where_one_entry_lands(&named, opens_with)
                else {
                    continue;
                };

                assert!(
                    folder == where_imported || folder.starts_with(&inside_it),
                    "{named:?} reached {folder:?}"
                );
                assert!(
                    crate::application::local_folders::is_local(&folder),
                    "{named:?} reached {folder:?}, which a server could claim"
                );
                let below = folder.strip_prefix(&inside_it).unwrap_or_default();
                for part in below.split('/').filter(|part| !part.is_empty()) {
                    assert!(
                        is_a_name_that_can_be_used(part),
                        "{named:?} reached {folder:?}, whose part {part:?} is not a name"
                    );
                }
            }
        }
    }

    #[test]
    fn test_an_archive_cannot_reach_a_folder_this_program_already_keeps() {
        // Why imported folders sit inside a folder of their own rather than
        // beside the ones an account already has. An archive naming a folder
        // `Outbox` would otherwise land in the one folder mail must never be
        // imported into, where what is listed comes from the send queue and a
        // message filed there is written down, counted and never seen again.
        use crate::common::types::Protocol;

        for named in [
            "Outbox.mbox",
            "Inbox.mbox",
            "Trash.mbox",
            "Drafts.mbox",
            "Sent.mbox",
            "Junk.mbox",
        ] {
            let folder = lands_in(named, a_folder_of_mail().as_bytes());

            for protocol in [Protocol::Imap, Protocol::Pop3] {
                for already_kept in crate::application::local_folders::for_account(protocol) {
                    assert_ne!(
                        folder,
                        already_kept.path(),
                        "{named} landed on a folder this program keeps for itself"
                    );
                }
            }
        }
    }

    /// Whether an entry with mail in it went nowhere because of its name.
    fn is_refused(named: &str) -> bool {
        where_one_entry_lands(named, a_folder_of_mail().as_bytes()) == WhereItGoes::NameRefused
    }

    #[test]
    fn test_the_ending_of_a_mailbox_file_comes_off_whatever_case_it_was_written_in() {
        // Some programs write `.MBOX`. Left on, the folder in the tree is read
        // out as "Invoices dot m box", which is a name from inside a file rather
        // than the name of somebody's mail.
        let where_imported = where_imported_folders_go();
        let mail = a_folder_of_mail().as_bytes();

        for named in ["Invoices.mbox", "Invoices.MBOX", "Invoices.Mbx"] {
            assert_eq!(
                lands_in(named, mail),
                format!("{where_imported}/Invoices"),
                "for {named}"
            );
        }

        // A name not written in English keeps every letter of it. Lowercasing a
        // whole name to find the ending and then cutting it by the ending's
        // length is a cut that can land in the middle of a character.
        assert_eq!(
            lands_in("Rechnungen f\u{fc}r 2024.mbox", mail),
            format!("{where_imported}/Rechnungen f\u{fc}r 2024")
        );

        // A name that is nothing but the ending keeps it, because a folder has
        // to be called something.
        assert_eq!(lands_in(".mbox", mail), format!("{where_imported}/.mbox"));
    }

    #[test]
    fn test_a_saved_message_at_the_top_of_the_archive_goes_into_the_import_folder_itself() {
        // The one name with no folder in it at all, and the only case where
        // having nothing left after the file's own name is an answer rather
        // than a refusal.
        assert_eq!(
            lands_in("note.eml", one_message().as_bytes()),
            where_imported_folders_go()
        );
    }

    #[test]
    fn test_a_step_out_of_a_folder_is_refused_even_where_the_name_is_about_to_be_dropped() {
        // The last part of an entry naming one saved message is the file's own
        // name, and it is dropped rather than used to build anything. So a step
        // hidden there is one nothing later in this module would ever look at,
        // and `Work/..` would quietly file its message in Work rather than in
        // the folder its name really asks for.
        for climbing in ["..", ".", "Work/..", "Work/.", "Work/.../one.eml"] {
            assert_eq!(
                where_one_entry_lands(climbing, one_message().as_bytes()),
                WhereItGoes::NameRefused,
                "{climbing:?} was not refused"
            );
        }
    }

    #[test]
    fn test_a_folder_named_after_a_device_on_windows_is_refused() {
        // Opening a file called `NUL` does something other than opening a file,
        // whatever is on the end of it, and a folder called that is a folder
        // nothing can later be written into or read out of.
        for device in ["CON", "nul", "Work/COM1", "Work/LPT9.mbox", "aux", "PRN"] {
            assert!(is_refused(device), "{device} stayed a device");
        }
    }

    #[test]
    fn test_a_name_windows_would_quietly_change_is_refused_rather_than_used() {
        // A trailing dot or space is stripped by the filesystem after any check
        // has been made, so `Invoices.` and `Invoices` are one folder and only
        // one of them was ever looked at. A leading space goes the same way.
        for awkward in [
            "Work/Invoices.",
            "Work/Invoices ",
            "Work /Invoices",
            "Work/ Invoices",
            "Work/Invoices.mbox.",
            "Work/In<voices>",
            "Work/In|voices",
            "Work/In\u{202E}voices",
        ] {
            assert!(is_refused(awkward), "{awkward:?} was used as it stands");
        }
    }

    #[test]
    fn test_an_absurdly_deeply_nested_name_is_refused_and_an_ordinary_one_is_not() {
        // Both sides of the limit, because a limit only tested from one side is
        // one that could be anywhere. A real mailbox is a handful of folders
        // deep; the path a thousand would build is one nothing can open and
        // nobody could hear read out.
        let as_deep_as_allowed = format!("{}Invoices.mbox", "a/".repeat(MOST_FOLDERS_DEEP - 1));
        let one_deeper = format!("{}Invoices.mbox", "a/".repeat(MOST_FOLDERS_DEEP));

        assert!(!is_refused(&as_deep_as_allowed), "{as_deep_as_allowed}");
        assert!(is_refused(&one_deeper), "{one_deeper}");
    }

    #[test]
    fn test_a_name_too_long_to_write_is_refused_however_it_got_that_long() {
        // Two ways to be too long, and each has its own limit because neither
        // catches the other. One enormous part passes every rule about how many
        // parts there are; two dozen short parts pass every rule about how long
        // one part may be, and the path they build together is still longer than
        // Windows will open.
        let one_long_part = format!("{}.mbox", "a".repeat(150));
        let many_short_parts = format!("{}Invoices.mbox", "abcdefghij/".repeat(22));

        // The one long part is short enough altogether that only the rule about
        // a single name can turn it away, and the many short parts are each
        // short enough that only the rule about the whole name can. Written
        // that way on purpose: with either one made to catch both, taking the
        // other out would change nothing here.
        assert!(one_long_part.chars().count() < MOST_IN_A_NAME_ALTOGETHER);
        assert!(many_short_parts.chars().count() > MOST_IN_A_NAME_ALTOGETHER);
        assert!(is_refused(&one_long_part), "{one_long_part}");
        assert!(is_refused(&many_short_parts), "{many_short_parts}");
    }

    #[test]
    fn test_what_an_entry_holds_is_decided_from_its_bytes_and_not_from_its_name() {
        // An export carries its own index, settings and address book in among
        // the mail, and it names them whatever it likes. Going by the name
        // makes a folder called Invoices out of the index file that happens to
        // sit beside the invoices, and turns away the mailbox somebody's old
        // program saved with no ending at all.
        for (named, bytes) in [
            ("Work/Invoices.mbox", &b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR"[..]),
            (
                "Work/msgFilterRules.dat",
                b"name=\"Junk\"\nenabled=\"yes\"\n",
            ),
            ("Work/INBOX.msf", b"// <mdb:mork:z v=\"1.4\"/>\n"),
        ] {
            assert_eq!(
                where_one_entry_lands(named, bytes),
                WhereItGoes::NotMail,
                "{named} was taken for mail"
            );
        }

        // And the other way, which is the half that loses mail: a mailbox saved
        // with an ending nothing recognises is still a mailbox.
        assert_eq!(
            lands_in("Work/Sent Items", a_folder_of_mail().as_bytes()),
            format!("{}/Work/Sent Items", where_imported_folders_go())
        );
    }

    #[test]
    fn test_a_saved_message_goes_in_the_folder_it_sat_in_and_a_mailbox_is_a_folder_of_its_own() {
        // Both directions, because they are one rule and getting either half
        // wrong is silent. An export of separate messages is a folder of `.eml`
        // files, and reading each one as a folder of its own gives somebody
        // four hundred folders holding one message each. An export of whole
        // mailboxes is one file per folder, and reading each as a message in
        // its parent throws the folders away and heaps everything together.
        let where_imported = where_imported_folders_go();

        assert_eq!(
            lands_in("Work/2024/note.eml", one_message().as_bytes()),
            format!("{where_imported}/Work/2024")
        );
        assert_eq!(
            lands_in("Work/Invoices.mbox", a_folder_of_mail().as_bytes()),
            format!("{where_imported}/Work/Invoices")
        );
    }

    #[test]
    fn test_a_folder_of_mail_lands_in_a_folder_on_this_computer() {
        // The whole point of the module. Mail read out of a stranger's archive
        // has never been on anybody's server, so it goes where no sync can
        // reach it and take it away again.
        let where_it_goes = where_one_entry_lands("Invoices.mbox", a_folder_of_mail().as_bytes());

        assert_eq!(
            where_it_goes,
            WhereItGoes::Into {
                folder: format!("{}/Invoices", where_imported_folders_go()),
                read: crate::application::importing_messages::ReadAs::OneAtATimeFromAnArchive,
            }
        );
    }
}

#[cfg(test)]
mod checked_a_second_time {
    use super::*;

    /// A message, so an entry is mail and gets as far as being placed.
    fn some_mail() -> &'static [u8] {
        b"From: a@example.com\r\nSubject: Hello\r\n\r\nBody\r\n"
    }

    #[test]
    fn test_no_name_a_stranger_can_write_reaches_outside_the_imported_folder() {
        // Written against the rule rather than by whoever wrote the rule, and
        // with names chosen to go at it from a different direction: separators
        // doubled and mixed, climbing steps buried in the middle rather than at
        // the front, Windows drive letters, the character that ends a name
        // early, and the dots hidden behind an ending the placer takes off.
        //
        // A zip is a stranger's file and its names are the one thing in it that
        // decides where bytes are written, so this is the check worth having
        // twice.
        let inside = where_imported_folders_go();
        for hostile in [
            "../outside.mbox",
            r"..\outside.mbox",
            "Work/../../outside.mbox",
            r"Work\..\..\outside.mbox",
            "Work//..//outside.mbox",
            "Work/./../outside.mbox",
            "/absolute/outside.mbox",
            r"\\server\share\outside.mbox",
            "C:/Windows/System32/outside.mbox",
            r"C:\Windows\outside.mbox",
            "....//outside.mbox",
            "..;/outside.mbox",
            "Work/..%2f..%2foutside.mbox",
            "..\u{0000}/outside.mbox",
            "Work/\u{202E}gpj.exe.mbox",
            "CON/mail.mbox",
            "Work/NUL.mbox",
            "..mbox",
            "...mbox",
            "Work/..mbox",
            "   ../outside.mbox",
            "../outside.mbox   ",
            "",
            "/",
            "\\",
            "//",
            "../",
            "./",
        ] {
            let landed = where_one_entry_lands(hostile, some_mail());

            if let WhereItGoes::Into { folder, .. } = &landed {
                assert!(
                    folder == &inside || folder.starts_with(&format!("{inside}/")),
                    "{hostile:?} was placed at {folder:?}, which is outside {inside:?}"
                );
                assert!(
                    crate::application::local_folders::is_local(folder),
                    "{hostile:?} was placed at {folder:?}, which is not a folder on this computer"
                );
                // No part of it is a climbing step. A part that merely
                // contains two dots, like `..;`, is a strange folder name and
                // is not one: these paths name rows in a table and never a
                // place on the disk, which the prefix settles on its own by
                // opening with a character Windows will not take in a file
                // name. So what matters is that no part is a step, not that
                // the text nowhere holds two dots.
                assert!(
                    !folder
                        .split('/')
                        .any(|part| !part.is_empty() && part.chars().all(|c| c == '.')),
                    "{hostile:?} was placed at {folder:?}, which carries a climbing step"
                );
            }
        }
    }

    #[test]
    fn test_an_ordinary_archive_still_keeps_the_shape_it_had() {
        // The other half, and the one that stops the check above being passed
        // by refusing everything. A rule that refuses every name is perfectly
        // safe and imports nothing.
        //
        // A file of several messages is a folder of its own; a file of one is
        // a message in the folder holding it. So the archive of a folder is
        // what keeps its place, and that is what is asked here.
        let inside = where_imported_folders_go();
        let several = concat!(
            "From nobody Mon Jul 20 10:00:00 2026\r\n",
            "From: a@example.com\r\nSubject: One\r\n\r\nFirst\r\n",
            "\r\n",
            "From nobody Tue Jul 21 11:00:00 2026\r\n",
            "From: b@example.com\r\nSubject: Two\r\n\r\nSecond\r\n",
        )
        .as_bytes();

        let landed = where_one_entry_lands("Work/Invoices.mbox", several);

        assert_eq!(
            landed,
            WhereItGoes::Into {
                folder: format!("{inside}/Work/Invoices"),
                read: ReadAs::OneAtATimeFromAnArchive,
            },
            "an ordinary archive folder no longer keeps its place"
        );
    }
}

//! Writing folders of mail out to one file, keeping the shape they were in.
//!
//! [`crate::application::message_files`] writes one message out and several
//! into an archive. This decides what an export of a whole mailbox is made of:
//! what each message has to be built from, where each folder goes inside the
//! file, what the folders are called once they are in it, and what somebody is
//! told at the end.
//!
//! Values in, values out. Nothing here opens a file, packs one, talks to a
//! server or touches the database: it takes what is stored and gives back the
//! bytes and the names whatever writes the file puts down.
//!
//! # Nothing kept the message as it arrived
//!
//! What is stored is a row of columns and, for the messages somebody has
//! opened at least once, the text. The message as the server handed it over
//! was never kept, so exporting one means building it again out of those
//! columns.
//!
//! Two of them are the trap. `from_addr` and `to_addr` hold addresses already
//! written out as text, display names and all: `Ada Lovelace
//! <ada@example.com>`, and `"Babbage, Charles" <charles@example.com>` once the
//! name carries something that would otherwise read as the separator between
//! two people. Reading that back is a parse, writing it out again is a
//! spelling, and where the two disagree one recipient becomes two or two
//! become one. The count still looks right and nothing says anything. That is
//! the shape every data-losing defect in this program has had.
//!
//! So neither half is written again here. The reading goes through the parser
//! that already reads a real `From` or `To` header off the wire, and the
//! writing goes through the spelling the stored column was written with in the
//! first place. `test_every_name_in_a_stored_column_is_the_same_name_when_it_comes_back`
//! is the round trip: out through this program's own writer, back through its
//! own reader, and into a row again through the one place that files a message
//! this program has read.
//!
//! # What cannot come back
//!
//! Said plainly, because an export that quietly drops something is worse than
//! one that says what it cannot carry:
//!
//! - **The files a message came with.** What is stored about one is its name,
//!   its type and its size. The file itself was never kept, so a message that
//!   arrived carrying three of them exports as the message and none of them.
//! - **Every header nobody displays.** The message is built from the columns,
//!   so what it arrived with and this program does not show is gone: the route
//!   it took, the name of the program that sent it, whatever a mailing list
//!   added on the way.
//! - **Read, starred, answered, flagged.** Those are columns rather than parts
//!   of the message, and the file this writes has nowhere to put them, so mail
//!   brought back somewhere else comes back unread.
//! - **Which of two headers an identifier came from.** `References` and
//!   `In-Reply-To` go into one column. See [`the_message_this_one_answers`].
//! - **A body's missing last line ending.** See [`ending_where_a_line_ends`].
//!
//! # The shape of the file
//!
//! One file holding many folders, each folder's mail in an archive of its own
//! inside it, under the folder's own path, so a folder inside a folder is
//! still inside it when somebody unpacks the export. A folder that holds
//! nothing is in the file as the folder and nothing else, never as an archive
//! with no messages in it.
//!
//! Which folders go in is whatever asks. Worth knowing before choosing them:
//! the Outbox lists the queue of mail waiting to go out rather than messages
//! filed in it, so it exports as a folder with nothing in it however much is
//! waiting there.

use crate::application::importing_messages::{MessagesExported, WritingOut};
use crate::application::message_files::{self, FileOnTheMessage};
use crate::application::summing_up::SummingUp;
use crate::common::types::EmailAddress;
use crate::data::message_cache::MessageListRow;
use crate::data::message_cache::attachment_content::AttachmentWithContent;
use crate::data::message_cache::bodies::MessageBody;
use crate::data::message_cache::signed_original::SignedOriginal;
use crate::service::attachment_name::safe_file_name;
use crate::service::mime::ParsedMessage;
use std::collections::HashSet;

// ── Rebuilding one message from what is stored ──────────────────────────────

/// One stored message, put back into the shape a message file holds.
///
/// Exactly what is stored and nothing invented: the message the file gets is
/// the columns and the text, and the headers that were never kept are not
/// guessed at. What cannot come back at all is listed at the top of this file.
///
/// The body is passed as it is stored, so a message written out on its own is
/// the message that was stored. The archive path has one more rule to keep,
/// and [`ending_where_a_line_ends`] is where it is.
pub fn rebuilt_from_what_is_stored(stored: &MessageListRow, text: &MessageBody) -> ParsedMessage {
    // Read once. Both headers that put a message in its conversation come out
    // of this one column, and reading it twice is two chances to read it two
    // ways.
    let conversation = the_conversation_before_it(as_stored(&stored.refs_header));
    ParsedMessage {
        subject: stored.subject.clone(),
        from: everyone_named_in(&stored.from_addr),
        to: everyone_named_in(&stored.to_addr),
        cc: everyone_named_in(as_stored(&stored.cc)),
        reply_to: everyone_named_in(as_stored(&stored.reply_to)),
        date: something_written_in(&stored.date),
        message_id: something_written_in(&stored.message_id),
        in_reply_to: the_message_this_one_answers(&conversation),
        references: conversation,
        body_plain: text.body_plain.clone(),
        body_html: text.body_html.clone(),
        ..ParsedMessage::default()
    }
}

/// What a column that may hold nothing holds.
///
/// A column with nothing in it and a column that was never written are the
/// same thing to everything below: no address, no conversation. The difference
/// is worth keeping in the database, where it says whether anybody has looked,
/// and means nothing in a message.
fn as_stored(column: &Option<String>) -> &str {
    column.as_deref().unwrap_or_default()
}

/// A column's value, or nothing when the column holds nothing.
///
/// The difference matters on the way out. An empty subject is a subject
/// somebody left blank; an empty identifier is not an identifier, and written
/// down it becomes `Message-ID: <>`, which is a malformed value some servers
/// refuse and every client ignores.
fn something_written_in(column: &str) -> Option<String> {
    Some(column.trim())
        .filter(|written| !written.is_empty())
        .map(str::to_string)
}

/// Every message in the conversation before this one, oldest first.
///
/// One column holds the whole chain, identifiers bare and separated by spaces,
/// which is how [`crate::application::threading::as_stored`] writes it and how
/// [`crate::application::threading::continuing`] reads it back.
fn the_conversation_before_it(chain: &str) -> Vec<String> {
    chain.split_whitespace().map(str::to_string).collect()
}

/// The message this one answers, as far as the stored chain can say.
///
/// The last of the chain, because that is where the message being answered
/// goes: `threading::as_stored` puts the chain down oldest first and adds the
/// one being answered on the end when it is not already there, and
/// `threading::continuing` reads the last one back as the parent.
///
/// What no reading of that column can recover is a message whose own
/// `In-Reply-To` named something other than the last of its `References`.
/// Both headers went into one column, so which one an identifier came from is
/// not written down anywhere. The chain itself is exact either way, and the
/// chain is what a conversation is built from.
fn the_message_this_one_answers(conversation: &[String]) -> Option<String> {
    conversation.last().cloned()
}

/// What became of one stored message an export reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatBecameOfIt {
    /// Written into the folder's archive, leaving `files_left_out` of the files
    /// it carries behind because this computer does not have them.
    ///
    /// The message still goes in, files or no files. An attachment nobody kept
    /// is listed on the message by name, type and size wherever it is read, and
    /// leaving the whole message out over one missing file would lose the
    /// twenty things about it that are here.
    WrittenOut {
        /// How many of its files this computer does not have. Usually nought.
        files_left_out: usize,
        /// Whether it says it is signed and the bytes that would prove it were
        /// not kept, so what went into the file cannot carry the signature.
        ///
        /// Not a failure and not a reason to leave the message out. It is a
        /// fact about the export that somebody has to be told, in the same way
        /// and for the same reason as a file that could not go with its
        /// message.
        signature_could_not_be_kept: bool,
    },
    /// Left out, because its text has never been downloaded to this computer.
    ///
    /// Not a failure and not a message to write out empty. Opening it once
    /// downloads the text, and then it exports like any other.
    LeftOutUntilItIsDownloaded,
}

impl WhatBecameOfIt {
    /// Count this message in what the export says at the end.
    ///
    /// Exactly one of the message counts moves, which is what makes the total
    /// worth saying: the two of them have to add up to the number of messages
    /// the folder held. A message that moved no count is one nobody is ever
    /// told about, and one that moved two reports a mailbox as holding more
    /// than it does.
    ///
    /// Into the counts an export of a single folder already keeps, rather than
    /// a second pair meaning the same thing. Two counts of one fact is how the
    /// same export comes to be described two ways. The files are counted here
    /// too, for that reason: the caller adding them up itself would be a second
    /// place that has to remember.
    pub fn counted_in(self, exported: &mut FoldersExported) {
        match self {
            Self::WrittenOut {
                files_left_out,
                signature_could_not_be_kept,
            } => {
                exported.messages.written += 1;
                exported.files_not_on_this_computer += files_left_out;
                exported.signatures_that_could_not_be_kept +=
                    usize::from(signature_could_not_be_kept);
            }
            Self::LeftOutUntilItIsDownloaded => exported.messages.not_on_this_computer += 1,
        }
    }

    /// Whether this message went into the archive.
    ///
    /// Asked by whatever writes the file, which starts the folder's archive
    /// when the first message goes into it rather than before. See
    /// [`FolderInTheFile::once_written`].
    pub fn was_written(self) -> bool {
        matches!(self, Self::WrittenOut { .. })
    }
}

/// Add one stored message to the archive being written for its folder.
///
/// One message at a time rather than a folder at once, because a folder is the
/// largest thing this program handles: a function that took every message and
/// gave back the finished archive would hold the whole folder in memory a
/// second time. The caller adds one message, writes out what has accumulated,
/// empties the buffer and keeps none of it.
///
/// Nothing goes into the archive for a message whose text was never
/// downloaded, and the answer says so. That is what stops a folder somebody
/// has only ever scrolled past from becoming a file of senders and subjects
/// with nothing under them, counted and reported as an export that worked.
///
/// A folder where that happens to every message leaves the buffer as empty as
/// it started, so whatever is writing should start the archive when the first
/// message goes into it rather than before: an archive holding no messages is
/// a file this program's own import refuses to open.
/// The files a message carries go in with it, where this computer has them.
/// They are kept when a message is read, so ordinarily it does. The ones it
/// does not have are counted and come back in the answer, because an archive
/// that quietly leaves files out is a backup that looks complete and is not.
/// Nothing is fetched here: an export of forty thousand messages is not the
/// moment to open forty thousand connections.
///
/// `arrived_as` is what the store holds of the form the message came in. A
/// signed message is written from those bytes rather than rebuilt, because a
/// signature is a statement about bytes and rebuilding a message reorders its
/// headers and rewraps its lines: what came out was ordinary mail carrying a
/// loose signature file, and importing it again said nothing about a signature
/// at all. A signed message whose bytes were not kept is still written, from
/// the columns like any other, and counted. Losing the message to save the
/// signature would be the worse trade, and saying nothing would be the worst.
pub fn added_to_the_archive(
    archive: &mut Vec<u8>,
    stored: &MessageListRow,
    text: Option<&MessageBody>,
    files: &[AttachmentWithContent],
    arrived_as: &SignedOriginal,
) -> WhatBecameOfIt {
    let Some(text) = text.filter(|text| is_really_there(text)) else {
        return WhatBecameOfIt::LeftOutUntilItIsDownloaded;
    };
    let carried = what_can_be_written_of(files);
    let rebuilt = ending_where_a_line_ends(rebuilt_from_what_is_stored(stored, text));
    match arrived_as {
        SignedOriginal::Kept(raw) => message_files::written_into_an_archive(
            archive,
            &rebuilt,
            message_files::WhatToWrite::ExactlyAsItArrived(raw),
        ),
        SignedOriginal::NotSigned | SignedOriginal::NotKept => {
            message_files::written_into_an_archive(
                archive,
                &rebuilt,
                message_files::WhatToWrite::RebuiltFromWhatIsStored(&carried.to_write),
            );
        }
    }
    WhatBecameOfIt::WrittenOut {
        files_left_out: match arrived_as {
            // Written as it arrived carries its own files, so none of them was
            // left behind whatever the store has. Counting the store's answer
            // here would report files missing from a message that has all of
            // them.
            SignedOriginal::Kept(_) => 0,
            _ => carried.not_here,
        },
        signature_could_not_be_kept: matches!(arrived_as, SignedOriginal::NotKept),
    }
}

/// What an export can write of one message's files, and what it cannot.
struct TheFilesOnIt {
    /// The ones this computer has, in the order the message carries them.
    to_write: Vec<FileOnTheMessage>,
    /// How many it does not have.
    not_here: usize,
}

/// One message's files, split into the ones that can be written and a count of
/// the ones that cannot.
///
/// One walk rather than a list built here and a count taken somewhere else, so
/// the number somebody is told cannot come to disagree with what really went
/// into the file.
///
/// The name is passed through exactly as it is stored, and the order is the
/// order the files are recorded in, which is the order the parser found them
/// and the position everything else counts to.
fn what_can_be_written_of(files: &[AttachmentWithContent]) -> TheFilesOnIt {
    let mut carried = TheFilesOnIt {
        to_write: Vec::new(),
        not_here: 0,
    };
    for file in files {
        match &file.content {
            Some(bytes) => carried.to_write.push(FileOnTheMessage {
                named: Some(file.described.filename.clone()),
                kind: file.described.mime_type.clone(),
                bytes: bytes.clone(),
            }),
            None => carried.not_here += 1,
        }
    }
    carried
}

/// The same message, with each half of its body ending where a line ends.
///
/// This is no longer what stops the archive losing messages. It was written
/// when it was: the writer assumed a message already ended where a line ends,
/// and given one that stopped mid-line the separator after it had the end of
/// somebody's sentence in front of it instead of an empty line, so every
/// message after the first was read back as part of the first one's body.
/// Three went in and one came out. `message_files::written_into_an_archive`
/// now sees to that itself, for every caller rather than this one.
///
/// What is still here is the difference between doing it well and doing it
/// safely. The writer adds the ending this program writes; this adds the one
/// the body itself already uses, so a body whose lines end the other way is
/// not left with one line ending unlike all the others. Stored bodies land on
/// the wrong side of that more often than they look: text kept from a message
/// composed here, and markup ending `</html>`, both stop mid-line.
///
/// This is what a trip through an archive changes, and it is the only thing:
/// a body that did not end with a line break comes back with one. There is no
/// way to say otherwise in this format, where a message is a run of lines and
/// the line after the last of them is where the next message starts. Only for
/// an archive: a message written out on its own keeps its body exactly as it
/// was stored.
fn ending_where_a_line_ends(message: ParsedMessage) -> ParsedMessage {
    ParsedMessage {
        body_plain: message.body_plain.map(with_the_last_line_ended),
        body_html: message.body_html.map(with_the_last_line_ended),
        ..message
    }
}

/// One body whose last line ends the way its other lines do.
///
/// A body with nothing in it is left alone. It writes as the empty line that
/// closes the headers and nothing after it, so the archive already ends where
/// a line ends and adding another would put a blank line into a message that
/// had none.
fn with_the_last_line_ended(body: String) -> String {
    if body.is_empty() || body.ends_with('\n') {
        return body;
    }
    let ending = how_this_body_ends_its_lines(&body);
    format!("{body}{ending}")
}

/// How a body ends its lines, read from the body itself.
///
/// A message composed on this computer ends its lines with the newline alone,
/// and giving its last line a different ending from all the others is the kind
/// of difference nobody sees until they compare two exports of one mailbox.
/// With nothing to go on, both bytes: that is what a message file ends a line
/// with, and the readers that refuse the newline alone are the servers.
fn how_this_body_ends_its_lines(body: &str) -> &'static str {
    if body.contains('\n') && !body.contains("\r\n") {
        return "\n";
    }
    "\r\n"
}

/// Whether a stored body is really a body.
///
/// Neither half there is what the database gives back for a message nobody has
/// opened, and it is the same answer `get_message_body` gives about the same
/// row. An empty half is a different thing and is kept: a message somebody
/// sent with nothing in it is a real message.
fn is_really_there(text: &MessageBody) -> bool {
    text.body_plain.is_some() || text.body_html.is_some()
}

/// The people one stored column names.
///
/// The columns hold addresses already written out as text, display names and
/// all, so reading one back is a parse and not a split. It goes through the
/// parser this program already reads a real `From` or `To` header with, rather
/// than a second one written here: a name carrying a comma or a semicolon
/// reads as two people to anything that splits on punctuation, and one
/// recipient becoming two, or two becoming one, is the shape every
/// data-losing defect in this program has had.
fn everyone_named_in(column: &str) -> Vec<EmailAddress> {
    crate::service::mime::parse_addresses(column)
}

// ── Laying out an export that keeps its folders ─────────────────────────────

/// What stands between a folder and the folder inside it.
///
/// The same mark in a stored path and in the file this writes, which is what
/// lets a folder inside a folder still be inside it once the export is
/// unpacked.
const SEPARATES_FOLDERS: &str = "/";

/// What one folder's archive inside the exported file is named to end with.
///
/// Taken from the export that writes a single archive rather than written down
/// a second time, so a folder inside this file is named the way a folder
/// written out on its own is and the two cannot drift apart. A refusal is the
/// only answer that names no ending, and this does not ask about a refusal, so
/// nothing reaches the second arm.
const AN_ARCHIVE_ENDS_WITH: &str = match WritingOut::AnArchive.the_file_ends_with() {
    Some(ending) => ending,
    None => ".mbox",
};

/// One folder as the exported file holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderInTheFile {
    /// The folder as this computer stores it.
    ///
    /// What the caller reads the messages of. Kept beside the name below
    /// rather than worked back out of it, because the name is not reversible:
    /// it has had characters taken out of it that a file cannot carry.
    pub stored_at: String,
    /// The folder itself inside the exported file, the folders above it and
    /// all.
    pub named: String,
}

impl FolderInTheFile {
    /// The name an archive of this folder's mail goes into the file under.
    ///
    /// The ending comes from the export that writes one archive on its own, so
    /// a folder inside this file is named the way a folder written out by
    /// itself is and the two cannot drift apart.
    ///
    /// Asked at the moment the first message goes in rather than once the
    /// folder is finished. Whatever writes the file starts the archive then, so
    /// that a folder of forty thousand messages carrying their files is never
    /// held in memory whole, and it has to name what it is starting.
    pub fn an_archive_of_its_mail(&self) -> String {
        format!("{}{AN_ARCHIVE_ENDS_WITH}", self.named)
    }

    /// The name the folder itself goes into the file under, with nothing in it.
    ///
    /// For a folder that holds no mail at all, and for one whose every message
    /// was left out for want of its text. Both are kept, because an empty
    /// folder somebody has had for years is part of the shape this export
    /// exists to keep and dropping it would lose it with nothing said.
    ///
    /// Neither is written as an archive holding no messages. An archive with
    /// nothing in it is a file this program's own import turns away as not
    /// mail, so a folder written that way would go into the export and be one
    /// nobody could bring back. Which of the two a folder gets is decided by
    /// what really went in rather than by what the folder holds: those differ
    /// whenever a message's text was never downloaded, and asking the wrong
    /// one opens an archive for a folder and then puts nothing in it.
    pub fn the_folder_and_nothing_in_it(&self) -> &str {
        &self.named
    }
}

/// Where each folder goes inside one exported file.
///
/// All of them in one call rather than one at a time, because the names have
/// to be decided against each other: two folders whose names differ only by
/// something a file name cannot carry arrive at the same name, and a function
/// answering about one folder has no way to know.
pub fn where_each_folder_goes(folders: &[String]) -> Vec<FolderInTheFile> {
    let mut taken = HashSet::new();
    folders
        .iter()
        .map(|stored_at| FolderInTheFile {
            stored_at: stored_at.clone(),
            named: one_nothing_else_has_taken(&as_a_path_in_the_file(stored_at), &mut taken),
        })
        .collect()
}

/// One folder's stored path, written the way a file can carry it.
///
/// Every part of the path on its own, so a folder inside a folder stays inside
/// it. Each part goes through the rule this program already cleans a name a
/// stranger chose with: no separators, no characters Windows refuses, nothing
/// that reads backwards to a person and forwards to the filesystem, no device
/// name, no trailing dot or space, and a length a path can carry.
///
/// It is also what takes this program's own mark off a folder that lives here.
/// The reserved prefix opens with a control character, and a name a file can
/// carry has no control characters in it, so the mark goes and the word after
/// it stays.
fn as_a_path_in_the_file(stored_at: &str) -> String {
    let folders: Vec<String> = stored_at
        .split(SEPARATES_FOLDERS)
        .filter_map(as_one_folder_in_the_path)
        .collect();
    if folders.is_empty() {
        return A_FOLDER_WITH_NO_USABLE_NAME.to_string();
    }
    folders.join(SEPARATES_FOLDERS)
}

/// What a folder is called when its own name is nothing a file can carry.
///
/// Plain, and a word somebody hears and understands. The names in a run are
/// told apart afterwards, so several of these in one export do not become one.
const A_FOLDER_WITH_NO_USABLE_NAME: &str = "Folder";

/// One folder of a path, or nothing where the path named no folder there.
///
/// A separator at the front of a path, or two in a row, names no folder
/// between them: `/INBOX` is the inbox, not an unnamed folder holding it, and
/// standing in for the nothing in front would put every folder in the export
/// one level deeper than it is.
///
/// A part that had something in it and nothing a file name can keep is a
/// different case and gets a name. Dropping it would move every folder under
/// it up a level, which changes the shape the export exists to keep.
fn as_one_folder_in_the_path(part: &str) -> Option<String> {
    if part.trim().is_empty() {
        return None;
    }
    if !carries_anything_a_name_can_keep(part) {
        return Some(A_FOLDER_WITH_NO_USABLE_NAME.to_string());
    }
    Some(safe_file_name(part))
}

/// Whether a name has anything in it a file name could keep.
///
/// The one question the rule that cleans a name cannot answer, because what it
/// gives back when the answer is no is the name it gives an attachment nobody
/// named: a folder called that tells somebody opening the file that their mail
/// folder is a file somebody sent them. Asked of what was there rather than of
/// what came back, so a folder really called that keeps its own name.
///
/// Dots, spaces, and the characters that are not really characters all come
/// off, and a name that was only those had nothing in it to begin with. A name
/// of nothing but punctuation a file cannot carry is not one of these: those
/// characters become underscores, and the shape of the name survives.
fn carries_anything_a_name_can_keep(part: &str) -> bool {
    part.chars().any(|letter| {
        !letter.is_control()
            && !letter.is_whitespace()
            && letter != '.'
            && !matches!(letter, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
    })
}

/// A name nothing else in this file has taken.
///
/// Capitals do not count towards telling two names apart. A mail server is
/// happy to hold "Work" and "work" as two mailboxes and Windows is not, so
/// side by side in one folder they are one file: whatever unpacks the export
/// writes one folder's mail over the other's, long after this program has said
/// the export worked. Nothing later gets the chance to notice, so it is
/// settled here.
///
/// Terminates: each attempt is a different name, and a run of folders is
/// finite, so there are always names left.
fn one_nothing_else_has_taken(wanted: &str, taken: &mut HashSet<String>) -> String {
    let mut attempt = 1;
    loop {
        let named = numbered(wanted, attempt);
        if taken.insert(named.to_lowercase()) {
            return named;
        }
        attempt += 1;
    }
}

/// The first name is the one asked for; the rest carry a number.
///
/// The number goes on the end of the folder's own name rather than in front of
/// it, so the two folders still sort together and still read as what they are.
fn numbered(wanted: &str, attempt: usize) -> String {
    if attempt == 1 {
        return wanted.to_string();
    }
    format!("{wanted} ({attempt})")
}

// ── Saying what the export did ──────────────────────────────────────────────

/// What writing a mailbox out, folders and all, did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FoldersExported {
    /// How many folders the file holds.
    ///
    /// Every folder that was asked for, including the ones that turned out to
    /// hold nothing: they are in the file as folders, so they are part of what
    /// came out.
    pub folders: usize,
    /// The messages, counted the way an export of one folder already counts
    /// them.
    ///
    /// The same two counts rather than a second pair meaning the same thing,
    /// so somebody who exports one folder and then their whole mailbox hears
    /// one fact worded one way.
    pub messages: MessagesExported,
    /// How many files the export could not write, because this computer does
    /// not have them.
    ///
    /// Counted in files rather than in messages, because that is the thing
    /// somebody is missing: one message carrying six of them is six files that
    /// are not in the backup. An attachment is here whenever the message has
    /// been opened, so ordinarily this is nought.
    pub files_not_on_this_computer: usize,
    /// How many messages went into the file saying they are signed, with
    /// nothing in the export that could prove it.
    ///
    /// The bytes a signature is made over are kept for signed mail and dropped
    /// under a budget, and mail signed before this program kept them has none.
    /// Those messages export whole and their signatures do not survive the
    /// trip, which is worth a sentence for the same reason a missing file is.
    pub signatures_that_could_not_be_kept: usize,
}

/// What an export of a whole mailbox did, in the words somebody hears.
///
/// The count that is not zero is the one worth saying, and it gets a whole
/// sentence rather than another number, because it names something somebody
/// has to decide what to do about.
///
/// The sentence about messages left out is written in the same words the
/// export of one folder uses about the same fact. A test holds the two
/// together, so neither can be reworded on its own and leave somebody hearing
/// one thing on the status line and another in the log.
pub fn what_the_folder_export_did(written: &FoldersExported) -> String {
    let mut said = SummingUp::opening(how_much_came_out(written));
    if written.messages.not_on_this_computer > 0 {
        // Two sentences written out rather than one built from parts. Several
        // words have to agree in number, and a sentence assembled from
        // fragments reads like one.
        said.sentence(match written.messages.not_on_this_computer {
            1 => "1 message was left out, because it has not been downloaded to \
                  this computer: open it once, then export again"
                .to_string(),
            many => format!(
                "{many} messages were left out, because they have not been \
                 downloaded to this computer: open each one once, then export again"
            ),
        });
    }
    if written.files_not_on_this_computer > 0 {
        // Written out in full for the same reason as the sentence above, and
        // said at all because an archive that quietly leaves files out is a
        // backup that looks complete and is not. The messages themselves are in
        // the file: only the files are missing, and saying which it is saves
        // somebody opening the export to find out.
        said.sentence(match written.files_not_on_this_computer {
            1 => "1 file could not go with its message, because it is not on \
                  this computer: open that message once, then export again"
                .to_string(),
            many => format!(
                "{many} files could not go with their messages, because they \
                 are not on this computer: open those messages once, then \
                 export again"
            ),
        });
    }
    if written.signatures_that_could_not_be_kept > 0 {
        // The message is in the file and the proof of who wrote it is not.
        // Said rather than left to be discovered, because the moment somebody
        // needs a signature is years later and by then there is no getting it
        // back. No advice on the end of it: unlike a missing file, there is
        // nothing to do about this one, and inventing a step would be worse
        // than saying so.
        said.sentence(match written.signatures_that_could_not_be_kept {
            1 => "1 message is signed and went in without what proves it, \
                  because this computer no longer has the form it arrived in"
                .to_string(),
            many => format!(
                "{many} messages are signed and went in without what proves \
                 them, because this computer no longer has the form they \
                 arrived in"
            ),
        });
    }
    said.spoken()
}

/// The opening line: how many messages, and how many folders they came from.
///
/// The count of messages comes from the export that writes one archive, rather
/// than being written out again here, so the two cannot come to word the same
/// fact differently.
fn how_much_came_out(written: &FoldersExported) -> String {
    let messages = message_files::what_the_export_did(written.messages.written);
    match written.folders {
        // An export of nothing has no folders for a sentence about folders to
        // be about, and "from 0 folders" is a phrase nobody needs to hear.
        0 => messages,
        1 => format!("{messages} from 1 folder"),
        many => format!("{messages} from {many} folders"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::filing::{AlreadyRead, a_row_filed_here};
    use crate::application::importing_messages::what_the_mail_export_did;
    use crate::application::local_folders::{LOCAL_PREFIX, local_sent};
    use crate::common::types::Protocol;
    use crate::data::message_cache::IncomingMessage;
    use crate::service::safety::Safety;
    use base64::Engine as _;

    /// One stored message, as a folder listing hands it over.
    fn a_stored_message() -> MessageListRow {
        MessageListRow {
            id: 1,
            uid: 42,
            account_id: "acct".to_string(),
            message_id: "note-1@example.com".to_string(),
            refs_header: None,
            subject: "Notes on the engine".to_string(),
            from_addr: "Ada Lovelace <ada@example.com>".to_string(),
            to_addr: "Charles Babbage <charles@example.com>".to_string(),
            cc: None,
            reply_to: None,
            date: "2026-07-20T10:00:00Z".to_string(),
            snippet: None,
            size_bytes: None,
            read: false,
            starred: false,
            answered: false,
            draft: false,
            has_attachments: false,
            safety: Safety::default(),
            safety_reasons: Vec::new(),
            receipt_to: None,
        }
    }

    /// One attachment as the store hands it over: always described, and
    /// carrying its file only when this computer kept one.
    fn a_file_kept(named: &str, file: Option<&[u8]>) -> AttachmentWithContent {
        AttachmentWithContent {
            described: crate::data::message_cache::CachedAttachment {
                id: 0,
                message_id: 1,
                filename: named.to_string(),
                mime_type: "application/octet-stream".to_string(),
                size: file.map_or(4096, <[u8]>::len) as i64,
                content_id: None,
                description: crate::service::mime::WhatTheSenderSaid::Nothing,
            },
            content: file.map(<[u8]>::to_vec),
        }
    }

    /// One message into an archive, from mail that never claimed a signature.
    ///
    /// Which is what every test here is about except the three that are about
    /// signed mail. Named once rather than written out at each of them, so what
    /// each test is really asking stays legible.
    fn ordinary_mail_added(
        archive: &mut Vec<u8>,
        stored: &MessageListRow,
        text: Option<&MessageBody>,
        files: &[AttachmentWithContent],
    ) -> WhatBecameOfIt {
        added_to_the_archive(archive, stored, text, files, &SignedOriginal::NotSigned)
    }

    /// What a message that went into the file whole comes back as.
    const WENT_IN_WHOLE: WhatBecameOfIt = WhatBecameOfIt::WrittenOut {
        files_left_out: 0,
        signature_could_not_be_kept: false,
    };

    /// The text of a message somebody has opened at least once.
    fn some_text() -> MessageBody {
        MessageBody {
            body_plain: Some("The engine weaves algebraic patterns.".to_string()),
            body_html: None,
        }
    }

    /// One stored message written out and read straight back into a row.
    ///
    /// Through this program's own writer, its own reader, and the one place
    /// that turns a message it has read into a row, so the question is asked
    /// the way an export and an import really ask it rather than through a
    /// stand-in for either.
    fn there_and_back(stored: &MessageListRow) -> IncomingMessage {
        let rebuilt = rebuilt_from_what_is_stored(stored, &some_text());
        let written = message_files::written_as_one_message(&rebuilt, &[]);
        let read = message_files::read_one_message(&written).unwrap_or_else(|refused| {
            panic!(
                "what was written a line ago would not read back: {refused}\n{}",
                String::from_utf8_lossy(&written)
            )
        });
        a_row_filed_here(
            &read,
            stored.id,
            stored.uid,
            written.len(),
            AlreadyRead::No,
            "2026-08-28T09:00:00Z",
        )
    }

    /// Several people, written the way a stored column holds them.
    fn written_as_a_column(people: &[EmailAddress]) -> String {
        people
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join(", ")
    }

    /// One person whose name a header cannot carry as it stands.
    fn named(name: &str, address: &str) -> EmailAddress {
        EmailAddress::new(address.to_string(), Some(name.to_string()))
    }

    /// The names a stored column can really hold.
    ///
    /// Every column is written by spelling addresses out one way, so the
    /// shapes that can turn up in one are the shapes that spelling produces.
    /// Nothing here is padded with spaces, because the parser that writes
    /// these columns trims a name before it ever reaches one.
    const AWKWARD_NAMES: [&str; 12] = [
        "Ada Lovelace",
        "A. Lovelace",
        "Babbage, Charles",
        "Smith; Jane",
        "Bob <VIP>",
        "She said \"hello\"",
        "back\\slash",
        "quote\"and,comma",
        "J\u{fc}rgen M\u{fc}ller",
        "M\u{fc}ller, J\u{fc}rgen",
        "\u{4f1a}\u{8b70}\u{306e}\u{4ef6}",
        "\u{202E}reversed",
    ];

    #[test]
    fn test_any_name_a_stored_column_can_hold_comes_back_as_itself() {
        // Every shape at once rather than the three somebody thought to write
        // a test for. The name that breaks a reader is always the one nobody
        // put in a test, and here breaking it means a recipient turning into
        // two, or two into one, with the count still looking right.
        for name in AWKWARD_NAMES {
            let mut stored = a_stored_message();
            stored.from_addr = named(name, "someone@example.com").to_string();

            assert_eq!(
                there_and_back(&stored).from_addr,
                stored.from_addr,
                "the name {name:?} changed on the way through"
            );
        }
    }

    #[test]
    fn test_a_list_of_awkward_names_keeps_its_length_and_its_order() {
        // One column holding all of them, because a name that reads back
        // correctly on its own can still swallow the person after it or split
        // itself in two once there is a list around it, and the separator
        // between two people is a comma.
        let everyone: Vec<EmailAddress> = AWKWARD_NAMES
            .iter()
            .enumerate()
            .map(|(which, name)| named(name, &format!("person{which}@example.com")))
            .collect();
        let mut stored = a_stored_message();
        stored.to_addr = written_as_a_column(&everyone);

        let back = there_and_back(&stored);

        assert_eq!(back.to_addr, stored.to_addr);
        assert_eq!(
            back.to_addr.matches("@example.com").count(),
            everyone.len(),
            "the list changed length: {}",
            back.to_addr
        );
    }

    #[test]
    fn test_every_name_in_a_stored_column_is_the_same_name_when_it_comes_back() {
        // The one question in this module that loses somebody's mail quietly.
        // The columns hold addresses already written out as text, names and
        // all. Reading them back is one answer and writing them out again is
        // another, and where the two disagree the message changes on the way
        // through and nothing says so.
        //
        // A comma and a semicolon both read as the separator between two
        // people, so a name carrying either has to come back as one person and
        // not as two. A name outside plain English cannot go into a header as
        // it stands and is encoded on the way out, so it has to come back
        // decoded. And a list has to keep its order and its length.
        let mut stored = a_stored_message();
        stored.from_addr = named("Babbage, Charles", "charles@example.com").to_string();
        stored.to_addr = written_as_a_column(&[
            named("Smith; Jane", "jane@example.com"),
            named("J\u{fc}rgen M\u{fc}ller", "jurgen@example.com"),
            EmailAddress::new("ada@example.com".to_string(), None),
        ]);
        stored.cc = Some(written_as_a_column(&[
            named("M\u{fc}ller, J\u{fc}rgen", "both@example.com"),
            named("Bob <VIP>", "bob@example.com"),
        ]));
        stored.reply_to = Some(named("Lovelace, Ada", "ada@example.com").to_string());

        let back = there_and_back(&stored);

        assert_eq!(back.from_addr, stored.from_addr);
        assert_eq!(back.to_addr, stored.to_addr);
        assert_eq!(back.cc, stored.cc);
        assert_eq!(back.reply_to, stored.reply_to);
    }

    #[test]
    fn test_a_message_whose_text_was_never_downloaded_is_left_out_rather_than_written_empty() {
        // A folder lists every message it knows about and keeps the text of
        // only the ones somebody has opened, so exporting a whole folder
        // reaches messages this computer has the columns of and nothing else.
        // Written out anyway they are a sender, a subject and a date with
        // nothing under them: a file of empty messages that looks from the
        // outside exactly like an export that worked.
        let mut archive = Vec::new();

        let what = ordinary_mail_added(&mut archive, &a_stored_message(), None, &[]);

        assert_eq!(what, WhatBecameOfIt::LeftOutUntilItIsDownloaded);
        assert!(
            archive.is_empty(),
            "a message with no text went into the file anyway: {}",
            String::from_utf8_lossy(&archive)
        );
    }

    #[test]
    fn test_a_message_stored_with_neither_half_of_its_text_is_treated_as_not_downloaded() {
        // The database gives a row with neither half back as no body at all,
        // and this has to read it the same way. Taken for a body it becomes an
        // empty message counted as one that worked, which is the one thing the
        // count above exists to prevent.
        let mut archive = Vec::new();
        let nothing_in_it = MessageBody {
            body_plain: None,
            body_html: None,
        };

        let what =
            ordinary_mail_added(&mut archive, &a_stored_message(), Some(&nothing_in_it), &[]);

        assert_eq!(what, WhatBecameOfIt::LeftOutUntilItIsDownloaded);
        assert!(archive.is_empty(), "an empty message went into the file");
    }

    #[test]
    fn test_a_message_somebody_sent_with_nothing_in_it_is_still_written_out() {
        // The other side of the rule above, and the reason it is about which
        // halves are stored rather than about whether there are any words. A
        // message with an empty body is a real message: an appointment
        // carrying only its subject, or a note sent with nothing in it.
        // Counting those as never downloaded would leave them out of every
        // export somebody ever ran and say they had not been opened.
        let mut archive = Vec::new();
        let empty_but_downloaded = MessageBody {
            body_plain: Some(String::new()),
            body_html: None,
        };

        let what = ordinary_mail_added(
            &mut archive,
            &a_stored_message(),
            Some(&empty_but_downloaded),
            &[],
        );

        assert_eq!(what, WENT_IN_WHOLE);
        assert!(!archive.is_empty(), "the message was left out of the file");
    }

    #[test]
    fn test_a_folder_of_mail_goes_into_one_archive_a_message_at_a_time() {
        // A folder somebody has kept for twenty years does not fit in memory
        // twice, so the archive is built one message at a time: the caller
        // adds a message, writes out what has accumulated, and lets it go.
        //
        // Read back through the archive reader rather than looked at as bytes,
        // because what is being checked is that the two agree: an archive
        // whose separators this reader does not recognise holds every message
        // and gives back one.
        let mut archive = Vec::new();
        let mut sent = Vec::new();
        for which in 1..=3 {
            let mut stored = a_stored_message();
            stored.subject = format!("Message number {which}");
            stored.message_id = format!("note-{which}@example.com");

            let what = ordinary_mail_added(&mut archive, &stored, Some(&some_text()), &[]);

            assert_eq!(what, WENT_IN_WHOLE);
            sent.push(stored.subject);
        }

        let read = message_files::read_many_messages(&archive);

        let arrived: Vec<String> = read
            .messages
            .iter()
            .map(|message| message.subject.clone())
            .collect();
        assert_eq!(arrived, sent);
        assert_eq!(read.could_not_be_read, 0);
    }

    #[test]
    fn test_a_body_that_stops_mid_line_does_not_swallow_the_messages_after_it() {
        // An archive finds the message after this one by looking for a
        // separator with an empty line in front of it. A stored body whose
        // last line has no ending leaves the end of somebody's sentence there
        // instead, so the separator is not recognised and every message after
        // this one is read back inside this one's body. Three messages in, one
        // out, and the file opens without complaint.
        //
        // Both halves, because either can be the one written last: text kept
        // from a message composed here stops mid-line, and markup ending
        // `</html>` always does.
        for stops_mid_line in [
            MessageBody {
                body_plain: Some("No line break after this.".to_string()),
                body_html: None,
            },
            MessageBody {
                body_plain: None,
                body_html: Some("<p>Nor after this.</p>".to_string()),
            },
        ] {
            let mut archive = Vec::new();
            for which in 1..=3 {
                let mut stored = a_stored_message();
                stored.subject = format!("Message number {which}");
                stored.message_id = format!("note-{which}@example.com");
                ordinary_mail_added(&mut archive, &stored, Some(&stops_mid_line), &[]);
            }

            let read = message_files::read_many_messages(&archive);

            assert_eq!(
                read.messages.len(),
                3,
                "the messages after the first were swallowed:\n{}",
                String::from_utf8_lossy(&archive)
            );
            // What the archive changed, and all of it: one line ending on the
            // end. Everything the sender wrote is still there and unaltered.
            let came_back = read.messages[0]
                .body_plain
                .as_deref()
                .or(read.messages[0].body_html.as_deref())
                .unwrap_or_default();
            let went_in = stops_mid_line
                .body_plain
                .as_deref()
                .or(stops_mid_line.body_html.as_deref())
                .unwrap_or_default();
            assert_eq!(came_back, format!("{went_in}\r\n"));
        }
    }

    #[test]
    fn test_a_body_whose_lines_end_the_other_way_is_not_given_a_different_ending_on_its_last() {
        // A message composed on this computer ends its lines with the newline
        // alone. Ending its last line both ways when every other line ends one
        // way is the kind of difference nobody sees until they compare two
        // exports of the same mailbox.
        let written_here = MessageBody {
            body_plain: Some("One\nTwo".to_string()),
            body_html: None,
        };
        let mut archive = Vec::new();

        ordinary_mail_added(&mut archive, &a_stored_message(), Some(&written_here), &[]);

        let read = message_files::read_many_messages(&archive);
        assert_eq!(read.messages[0].body_plain.as_deref(), Some("One\nTwo\n"));
    }

    #[test]
    fn test_a_body_that_already_ends_a_line_is_left_exactly_as_it_was() {
        // The common case, and the one that has to stay untouched: a body
        // gaining a blank line every time somebody exported and imported it
        // would grow one on each pass.
        let ends_properly = MessageBody {
            body_plain: Some("The engine weaves algebraic patterns.\r\n".to_string()),
            body_html: None,
        };
        let mut archive = Vec::new();

        ordinary_mail_added(&mut archive, &a_stored_message(), Some(&ends_properly), &[]);

        let read = message_files::read_many_messages(&archive);
        assert_eq!(read.messages[0].body_plain, ends_properly.body_plain);
    }

    #[test]
    fn test_what_puts_a_message_in_its_conversation_and_on_its_date_survives_the_trip() {
        // A message that comes back without its identifier, its chain or its
        // date is a message that opens on its own at the wrong end of the
        // folder. For anybody reading a list by ear, a conversation broken
        // into single messages is the difference between following a thread
        // and hearing forty unrelated subjects.
        // Every shape the chain comes in. A message that answers nothing, one
        // that answers a single message, and one deep in a long thread are
        // three different readings of that column, and only the middle one is
        // the shape somebody would think to write a test for.
        for chain in [
            None,
            Some("only@example.com".to_string()),
            Some("first@example.com second@example.com third@example.com".to_string()),
        ] {
            let mut stored = a_stored_message();
            stored.refs_header = chain;

            let back = there_and_back(&stored);

            assert_eq!(back.message_id, stored.message_id);
            assert_eq!(back.date, stored.date);
            assert_eq!(back.refs_header, stored.refs_header);
        }
    }

    #[test]
    fn test_a_date_written_somewhere_other_than_here_comes_back_at_the_same_moment() {
        // The date goes out as the older form a header holds and comes back as
        // the sortable one, and the two spell an offset differently. A trip
        // that dropped or flattened the offset would move a message by hours,
        // which for anybody but a reader in London is mail filed on the wrong
        // day with nothing to say so.
        for written_at in [
            "2026-07-20T10:00:00Z",
            "2026-07-20T12:00:00+02:00",
            "2026-07-20T04:00:00-06:00",
        ] {
            let mut stored = a_stored_message();
            stored.date = written_at.to_string();

            assert_eq!(there_and_back(&stored).date, written_at);
        }
    }

    /// The names an export gives a run of folders.
    fn names_for(folders: &[&str]) -> Vec<String> {
        let stored: Vec<String> = folders.iter().map(|path| path.to_string()).collect();
        where_each_folder_goes(&stored)
            .iter()
            .map(|folder| folder.named.clone())
            .collect()
    }

    #[test]
    fn test_each_folder_keeps_its_place_inside_the_file() {
        // The point of exporting a mailbox rather than a folder: somebody gets
        // back the shape they had, with a folder inside a folder still inside
        // it, rather than every message they own in one heap.
        assert_eq!(
            names_for(&["INBOX", "INBOX/Archive/2026", "[Gmail]/All Mail"]),
            vec!["INBOX", "INBOX/Archive/2026", "[Gmail]/All Mail"]
        );
    }

    /// Whether every folder in this run got a name of its own.
    fn all_different(named: &[String]) -> bool {
        let mut alone = named.to_vec();
        alone.sort();
        alone.dedup();
        alone.len() == named.len()
    }

    #[test]
    fn test_two_folders_whose_names_clean_to_one_do_not_become_one_folder() {
        // A colon, a question mark and a star are all characters a file name
        // cannot carry, and all three become the same underscore. So three
        // folders somebody can tell apart at a glance arrive at one name, and
        // written under it each folder's mail is written over by the next.
        // The export says it worked and two thirds of the mail is not there.
        //
        // The third one is a folder really called what the other two clean to,
        // which is the case that catches a rule that only compares the names
        // it has changed.
        let named = names_for(&["Q1:Q2", "Q1?Q2", "Q1_Q2"]);

        assert!(
            all_different(&named),
            "two folders share one name: {named:?}"
        );
    }

    #[test]
    fn test_two_folders_differing_only_in_their_capitals_do_not_become_one_file() {
        // A mail server is happy to hold "Work" and "work" as two mailboxes.
        // Windows is not: unpacked side by side they are one file, and one
        // folder's mail is written over the other's by whatever unpacks the
        // export, long after this program has said it worked. This is where
        // that has to be caught, because nothing later gets the chance.
        let named = names_for(&["Work", "work", "WORK"]);

        let lowered: Vec<String> = named.iter().map(|name| name.to_lowercase()).collect();
        assert!(
            all_different(&lowered),
            "two folders are one file once this is unpacked: {named:?}"
        );
    }

    #[test]
    fn test_a_folder_named_with_nothing_a_file_can_carry_is_still_named_as_a_folder() {
        // A mailbox name comes from the server, and a server can say anything.
        // A name that is only dots, or only characters that are not really
        // characters, cleans down to nothing, and the rule that cleans it then
        // gives back the name it gives an attachment nobody named. Somebody
        // opening the file would be told their mail folder is a file that came
        // with a message.
        // Each of these is unusable for a different reason, and each is the
        // only case that reaches its half of the rule: only dots, only
        // characters that are not really characters, only spaces, only a mark
        // that makes a name read backwards, and a mixture of spaces and dots
        // that is not caught by either check on its own.
        let named = names_for(&["...", "\u{1}\u{2}", "   ", "\u{202E}", " . "]);

        for name in &named {
            assert!(!name.trim().is_empty(), "a folder with no name at all");
            assert!(
                !name.to_lowercase().contains("attachment"),
                "a folder is called after a file somebody sent: {name}"
            );
        }
        assert!(
            all_different(&named),
            "three folders share a name: {named:?}"
        );
    }

    #[test]
    fn test_a_folder_really_called_after_an_attachment_keeps_its_own_name() {
        // The other side of the rule above. Standing in for an unusable name
        // must not rename a folder somebody really called that, which is what
        // a rule comparing against the stand-in rather than asking what was
        // there would do.
        assert_eq!(names_for(&["attachment"]), vec!["attachment"]);
    }

    #[test]
    fn test_a_separator_with_no_folder_named_before_it_does_not_invent_one() {
        // A path opening with a separator names the folder after it and not an
        // unnamed folder holding it. Standing in for the nothing in front
        // would put every folder in the export one level deeper than it is.
        assert_eq!(names_for(&["/INBOX"]), vec!["INBOX"]);
        assert_eq!(names_for(&["INBOX//Archive"]), vec!["INBOX/Archive"]);
    }

    #[test]
    fn test_this_programs_own_mark_on_a_folder_here_does_not_reach_the_file() {
        // A folder on this computer is stored under a reserved prefix opening
        // with a character no mailbox name carries. That character is how this
        // program tells its own folders from a server's, and it is nobody
        // else's business: in a file somebody opens in another mail program it
        // is a byte that shows as nothing, sorts before everything, and on
        // Windows cannot be part of a name at all.
        //
        // The path comes from the one place that knows which folders live
        // here, rather than being spelled out again, so a folder added there
        // is a folder this answers about.
        let here = local_sent(Protocol::Pop3)
            .expect("an account whose mail is collected over POP keeps its folders here");

        let named = names_for(&[&here]);

        assert!(
            !named[0].contains(LOCAL_PREFIX),
            "the reserved prefix reached the file: {:?}",
            named[0]
        );
        assert!(
            !named[0].chars().any(char::is_control),
            "a character no name can carry reached the file: {:?}",
            named[0]
        );
        // And the word after the mark stays, because that is what the folder
        // is called here and what somebody opening the file is looking for.
        assert_eq!(named[0], "Local/Sent");
    }

    /// The one folder a run of one laid out.
    fn one_folder(stored_at: &str) -> FolderInTheFile {
        where_each_folder_goes(&[stored_at.to_string()])
            .pop()
            .expect("one folder in gives one folder out")
    }

    #[test]
    fn test_a_folders_mail_is_named_the_way_an_archive_written_on_its_own_is() {
        // A file whose name says one thing over contents that are another is
        // how an archive comes to be opened as a single message, with
        // everything after the first stuck on the end of its body where nobody
        // looks. The ending comes from the export that writes one archive on
        // its own, so a folder inside this file is named the way a folder
        // written out by itself is, and the two cannot drift apart.
        let folder = one_folder("INBOX/Archive/2026");

        assert_eq!(folder.stored_at, "INBOX/Archive/2026");
        assert_eq!(
            folder.an_archive_of_its_mail(),
            format!(
                "INBOX/Archive/2026{}",
                WritingOut::AnArchive
                    .the_file_ends_with()
                    .expect("an archive names the ending its file should have")
            )
        );
    }

    #[test]
    fn test_a_folder_with_no_mail_in_it_is_kept_as_a_folder_rather_than_an_empty_archive() {
        // Two decisions at once, and both go the same way.
        //
        // It is in the file: an empty folder somebody has kept for years is
        // part of the shape this export exists to keep, and dropping it would
        // lose the folder with nothing said.
        //
        // And it is the folder itself rather than an archive holding no
        // messages, because an archive holding no messages is a file this
        // program's own import turns away as not mail. Written that way, a
        // folder would go into the export that nobody could bring back.
        let folder = one_folder("Receipts 2019");

        assert_eq!(folder.the_folder_and_nothing_in_it(), "Receipts 2019");
        assert_ne!(
            folder.the_folder_and_nothing_in_it(),
            folder.an_archive_of_its_mail(),
            "a folder with nothing in it would go into the file under the name \
             an archive of its mail would have"
        );
    }

    #[test]
    fn test_a_folder_whose_every_message_was_left_out_puts_nothing_in_the_buffer() {
        // A folder somebody has only ever scrolled past has every message's
        // columns and none of their text. Whatever writes the file starts a
        // folder's archive when the first message goes into it, so a folder
        // where that never happens has to leave the buffer as empty as it
        // started: anything at all in it would open an archive for a folder
        // with no mail in it, and an archive holding no messages is a file this
        // program's own import turns away as not mail.
        //
        // What the file then holds for that folder is measured end to end in
        // `presentation::wx_app::what_an_export_holds`, against a real store.
        let mut archive = Vec::new();
        let mut written = 0;
        for _ in 0..3 {
            if ordinary_mail_added(&mut archive, &a_stored_message(), None, &[]).was_written() {
                written += 1;
            }
        }

        assert_eq!(written, 0);
        assert!(archive.is_empty());
    }

    #[test]
    fn test_every_name_this_gives_a_folder_is_one_a_file_can_carry() {
        // The property everything downstream rests on, asked of all the shapes
        // above at once and of the ones nothing else covers. A mailbox name
        // comes from a server, and a server can say anything.
        let very_long = "x".repeat(400);
        let awkward = [
            "INBOX",
            "[Gmail]/All Mail",
            "Q1:Q2",
            "Q1?Q2",
            "..",
            "...",
            "NUL",
            "COM1/LPT9",
            "annexe\u{202E}cod",
            "trailing. ",
            very_long.as_str(),
            "\u{1}Local/Sent",
            "/",
            "//",
            "",
            "   ",
            r"a\b",
            "Sent Items/2019/Q1*",
            "\u{4f1a}\u{8b70}",
            "Work",
            "work",
        ];
        let stored: Vec<String> = awkward.iter().map(|path| path.to_string()).collect();

        let laid_out = where_each_folder_goes(&stored);

        assert_eq!(laid_out.len(), awkward.len(), "a folder was dropped");
        for folder in &laid_out {
            for part in folder.named.split(SEPARATES_FOLDERS) {
                assert!(!part.is_empty(), "a folder with no name: {folder:?}");
                assert!(
                    !part.chars().any(char::is_control),
                    "a character no name can carry: {folder:?}"
                );
                assert!(
                    !part.chars().any(
                        |letter| matches!(letter, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
                    ),
                    "a name that reads backwards to a person: {folder:?}"
                );
                assert!(
                    !part.contains(['<', '>', ':', '"', '/', '\\', '|', '?', '*']),
                    "a character Windows refuses: {folder:?}"
                );
                assert!(
                    !part.ends_with(['.', ' ']),
                    "an ending the filesystem quietly strips after every check: {folder:?}"
                );
                assert!(part.chars().count() <= 130, "too long to write: {folder:?}");
            }
        }
        // And no two of them are one file once the export is unpacked.
        let lowered: Vec<String> = laid_out
            .iter()
            .map(|folder| folder.named.to_lowercase())
            .collect();
        assert!(
            all_different(&lowered),
            "two folders share a name: {lowered:?}"
        );
    }

    #[test]
    fn test_everything_this_module_says_is_a_sentence_and_names_no_machinery() {
        // All of it is read aloud. A fragment with no stop on the end runs
        // into whatever is spoken next, and a sentence naming a mechanism
        // tells somebody about this program's insides instead of about their
        // mail.
        //
        // Gathered here rather than checked one at a time, so a sentence added
        // later is covered without anybody remembering to.
        let everything = [
            what_the_folder_export_did(&FoldersExported {
                folders: 3,
                messages: MessagesExported {
                    written: 4,
                    not_on_this_computer: 2,
                },
                files_not_on_this_computer: 0,
                signatures_that_could_not_be_kept: 0,
            }),
            what_the_folder_export_did(&FoldersExported {
                folders: 1,
                messages: MessagesExported {
                    written: 1,
                    not_on_this_computer: 1,
                },
                files_not_on_this_computer: 1,
                signatures_that_could_not_be_kept: 1,
            }),
            what_the_folder_export_did(&FoldersExported {
                folders: 1,
                messages: MessagesExported {
                    written: 2,
                    not_on_this_computer: 0,
                },
                files_not_on_this_computer: 5,
                signatures_that_could_not_be_kept: 4,
            }),
        ];

        for said in &everything {
            assert!(!said.trim().is_empty(), "something said nothing at all");
            assert!(said.ends_with('.'), "not a sentence: {said}");
            let lowered = said.to_lowercase();
            for machinery in [
                "imap", "pop3", "uid", "database", "cache", "sync", "parse", "header", "mbox",
                ".eml", "row", "archive", "path",
            ] {
                assert!(
                    !lowered.contains(machinery),
                    "this names {machinery}, which is a mechanism and not what happens: {said}"
                );
            }
        }
    }

    #[test]
    fn test_every_message_the_export_reaches_moves_exactly_one_count() {
        // What makes the counts worth saying out loud: they have to add up to
        // the number of messages the folder held. A message that moves no
        // count is one nobody is ever told about, and one that moves two
        // reports a mailbox as holding more than it does.
        let text = some_text();
        let held = [Some(&text), None, Some(&text), None, None];
        let mut archive = Vec::new();
        let mut counted = FoldersExported::default();

        for body in held {
            ordinary_mail_added(&mut archive, &a_stored_message(), body, &[])
                .counted_in(&mut counted);
        }

        assert_eq!(
            counted.messages.written + counted.messages.not_on_this_computer,
            held.len(),
            "the counts do not add up to the messages the folder held: {counted:?}"
        );
        assert_eq!(
            counted.messages,
            MessagesExported {
                written: 2,
                not_on_this_computer: 3,
            }
        );
    }

    #[test]
    fn test_a_file_this_computer_does_not_have_is_counted_rather_than_passed_over() {
        // The whole point of counting them. An archive that quietly leaves a
        // file out is a backup that looks complete and is not, and somebody
        // finds out a year later when they go looking for the invoice.
        let mut archive = Vec::new();
        let mut counted = FoldersExported::default();

        ordinary_mail_added(
            &mut archive,
            &a_stored_message(),
            Some(&some_text()),
            &[
                a_file_kept("invoice.pdf", Some(b"the invoice")),
                a_file_kept("photograph.jpg", None),
                a_file_kept("notes.txt", None),
            ],
        )
        .counted_in(&mut counted);

        assert_eq!(counted.messages.written, 1, "the message was left out");
        assert_eq!(
            counted.files_not_on_this_computer, 2,
            "the files this computer does not have were not counted"
        );
        let written = String::from_utf8_lossy(&archive);
        assert!(
            written.contains("invoice.pdf"),
            "the file this computer does have was left out:\n{written}"
        );
    }

    /// One signed message, as it came off the wire.
    ///
    /// Signed mail is `multipart/signed`: the words and the signature over
    /// them, in that order, under one boundary. Nothing here checks the
    /// signature, and it does not have to. What matters is that these exact
    /// bytes come out again, because a signature is a statement about bytes and
    /// any change at all makes a good one look like a bad one.
    fn as_a_signed_message_arrived() -> Vec<u8> {
        concat!(
            "From: Ada Lovelace <ada@example.com>\r\n",
            "To: Charles Babbage <charles@example.com>\r\n",
            "Subject: Notes on the engine\r\n",
            "MIME-Version: 1.0\r\n",
            "Content-Type: multipart/signed; protocol=\"application/pkcs7-signature\";\r\n",
            " micalg=sha-256; boundary=\"the-boundary\"\r\n",
            "\r\n",
            "--the-boundary\r\n",
            "Content-Type: text/plain\r\n",
            "\r\n",
            "The engine weaves algebraic patterns.\r\n",
            "--the-boundary\r\n",
            "Content-Type: application/pkcs7-signature; name=\"smime.p7s\"\r\n",
            "Content-Transfer-Encoding: base64\r\n",
            "\r\n",
            "MIIBogYJKoZIhvcNAQcCoIIBkzCCAY8CAQExDTALBglghkgBZQMEAgE=\r\n",
            "--the-boundary--\r\n",
        )
        .as_bytes()
        .to_vec()
    }

    #[test]
    fn test_a_signed_message_goes_into_the_archive_exactly_as_it_arrived() {
        // Rebuilt from the stored columns, a signed message comes back out as
        // ordinary mail carrying a loose signature file, and the signature is
        // then a statement about bytes nobody has. Export and import it again
        // and the verdict is gone, with nothing said at any point.
        //
        // The bytes it arrived as are kept for exactly this reason, so where
        // they are here they are what goes into the file.
        let arrived = as_a_signed_message_arrived();
        let mut archive = Vec::new();

        let what = added_to_the_archive(
            &mut archive,
            &a_stored_message(),
            Some(&some_text()),
            &[],
            &SignedOriginal::Kept(arrived.clone()),
        );

        assert_eq!(what, WENT_IN_WHOLE);
        let written = String::from_utf8_lossy(&archive).into_owned();
        assert!(
            written.contains(&String::from_utf8_lossy(&arrived).into_owned()),
            "the message was rebuilt rather than written as it arrived, so its \
             signature is now about bytes nobody has:\n{written}"
        );
    }

    #[test]
    fn test_a_signed_message_whose_bytes_were_not_kept_is_still_written_out() {
        // Only signed mail that arrived since the bytes started being kept has
        // any, and the store drops them under a budget. Leaving those messages
        // out of the export, or refusing the export over them, would lose the
        // message to save the signature.
        let mut archive = Vec::new();

        let what = added_to_the_archive(
            &mut archive,
            &a_stored_message(),
            Some(&some_text()),
            &[],
            &SignedOriginal::NotKept,
        );

        assert!(what.was_written(), "the message was left out altogether");
        assert!(
            !archive.is_empty(),
            "nothing was written for a message counted as written"
        );
    }

    #[test]
    fn test_a_signature_the_export_could_not_keep_is_counted_rather_than_passed_over() {
        // Same class of problem as an archive quietly leaving a file out. The
        // message is in the file and the proof of who wrote it is not, and
        // whoever kept the export finds out when they need it.
        let mut counted = FoldersExported::default();

        for arrived in [
            SignedOriginal::Kept(as_a_signed_message_arrived()),
            SignedOriginal::NotKept,
            SignedOriginal::NotSigned,
        ] {
            let mut archive = Vec::new();
            added_to_the_archive(
                &mut archive,
                &a_stored_message(),
                Some(&some_text()),
                &[],
                &arrived,
            )
            .counted_in(&mut counted);
        }

        assert_eq!(counted.messages.written, 3);
        assert_eq!(
            counted.signatures_that_could_not_be_kept, 1,
            "the signature that could not be kept was not counted, or one that \
             could was counted as lost"
        );
    }

    #[test]
    fn test_a_message_carrying_a_file_writes_the_file_itself_and_not_only_its_name() {
        // A name with nothing under it is exactly the empty-message failure the
        // count of messages exists to prevent, one level down.
        let file: Vec<u8> = vec![0x00, 0xff, b'P', b'D', b'F', 0x1a];
        let mut archive = Vec::new();

        ordinary_mail_added(
            &mut archive,
            &a_stored_message(),
            Some(&some_text()),
            &[a_file_kept("report.pdf", Some(&file))],
        );

        let read = message_files::read_many_messages(&archive);
        assert_eq!(read.messages.len(), 1, "the message did not read back");
        assert_eq!(
            read.messages[0].attachments.len(),
            1,
            "the message came back carrying nothing"
        );
        assert_eq!(
            read.messages[0].attachments[0].filename.as_deref(),
            Some("report.pdf")
        );
        // And the file itself under that name, not a name with nothing under
        // it. A file is written encoded, because a file is bytes and a line of
        // it could otherwise read as the boundary and end the part early.
        let as_written = base64::engine::general_purpose::STANDARD.encode(&file);
        assert!(
            String::from_utf8_lossy(&archive).contains(&as_written),
            "the name went in and the file did not"
        );
    }

    #[test]
    fn test_an_export_says_how_many_messages_came_out_and_how_many_folders_they_came_from() {
        // Somebody who has just exported a mailbox is deciding whether they
        // have all of it. The number of folders is what tells them: four
        // messages out of one folder and four out of thirty are the same count
        // and very different news.
        let said = what_the_folder_export_did(&FoldersExported {
            folders: 3,
            messages: MessagesExported {
                written: 4,
                not_on_this_computer: 0,
            },
            files_not_on_this_computer: 0,
            signatures_that_could_not_be_kept: 0,
        });

        assert_eq!(said, "Exported 4 messages from 3 folders");
    }

    #[test]
    fn test_messages_left_out_are_said_in_the_words_the_one_folder_export_already_uses() {
        // A folder shows every message it knows about and keeps the text of
        // only the ones somebody has opened, so an export of a whole mailbox
        // reaches a great many messages it has the columns of and nothing
        // else. Left unsaid, the count on its own reads as a complete export
        // that happened to be smaller than expected.
        //
        // The clause is taken from the one-folder export at the moment this
        // runs rather than copied out here. A copy would go stale without
        // anything noticing, and somebody would then hear one wording on the
        // status line and another in the log.
        for how_many in [1, 3] {
            let said = what_the_folder_export_did(&FoldersExported {
                folders: 2,
                messages: MessagesExported {
                    written: 4,
                    not_on_this_computer: how_many,
                },
                files_not_on_this_computer: 0,
                signatures_that_could_not_be_kept: 0,
            });

            let by_the_one_folder_export = what_the_mail_export_did(&MessagesExported {
                written: 0,
                not_on_this_computer: how_many,
            });
            let clause = by_the_one_folder_export
                .strip_prefix("No messages were exported. ")
                .expect("the one folder export opens with a count and then explains");
            assert_eq!(
                said,
                format!("Exported 4 messages from 2 folders. {clause}")
            );
        }
    }

    #[test]
    fn test_one_folder_and_one_message_are_said_in_the_singular() {
        // Read aloud, "Exported 1 messages from 1 folders" is the kind of
        // sentence that makes somebody stop trusting the ones that matter.
        assert_eq!(
            what_the_folder_export_did(&FoldersExported {
                folders: 1,
                messages: MessagesExported {
                    written: 1,
                    not_on_this_computer: 0,
                },
                files_not_on_this_computer: 0,
                signatures_that_could_not_be_kept: 0,
            }),
            "Exported 1 message from 1 folder"
        );
    }

    #[test]
    fn test_an_export_that_wrote_nothing_says_only_that() {
        // Nothing chosen, or every folder empty. There are no folders for a
        // phrase about folders to be about, and "from 0 folders" is something
        // nobody needs read to them.
        assert_eq!(
            what_the_folder_export_did(&FoldersExported::default()),
            message_files::what_the_export_did(0)
        );
    }

    #[test]
    fn test_a_stored_message_comes_back_with_what_the_sender_wrote() {
        // Nothing keeps the message as it arrived: what is stored is a row of
        // columns and the text. So an export has to build the message again
        // from those, and the name somebody wrote under is in the column as
        // text rather than as an address.
        let rebuilt = rebuilt_from_what_is_stored(&a_stored_message(), &some_text());

        assert_eq!(rebuilt.subject, "Notes on the engine");
        assert_eq!(rebuilt.from[0].address, "ada@example.com");
        assert_eq!(rebuilt.from[0].name.as_deref(), Some("Ada Lovelace"));
        assert_eq!(rebuilt.to[0].address, "charles@example.com");
        assert_eq!(
            rebuilt.body_plain.as_deref(),
            Some("The engine weaves algebraic patterns.")
        );
    }
}

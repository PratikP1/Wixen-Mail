//! Files chosen to go out with a message.
//!
//! The composer had an Attach File button from the beginning and nothing behind
//! it: no handler, no list, no column in the outbox, and no part in the message
//! SMTP built. A button that looks like it works and does nothing is the failure
//! this project's third guardrail names, and it is worse here than most places,
//! because somebody who cannot see the window has no way to notice that the file
//! they picked never arrived.
//!
//! What is in this module is the part that can be decided without a window: what
//! a chosen file is, what is said about it, and what the message may carry. The
//! picking, the list and the sending are elsewhere and use this.

use crate::common::{Error, Result};
use std::path::{Path, PathBuf};

/// The most a message may carry, in bytes.
///
/// Providers differ and most sit near this. Gmail and Outlook.com both refuse
/// past 25 MB, and the refusal arrives after the upload, as an SMTP error
/// somebody reads long after they pressed Send. Saying so before the file is
/// attached is the only version of this that helps.
///
/// This is the size on disk. What goes on the wire is base64, which is about a
/// third larger again, so the real ceiling is lower and [`over_the_limit`]
/// accounts for it.
pub const LIMIT_BYTES: u64 = 25 * 1024 * 1024;

/// How much base64 adds: four characters for every three bytes.
const ENCODED_OVER_RAW: f64 = 4.0 / 3.0;

/// A file picked to go with the message.
///
/// The bytes are not held. A message is composed over minutes and the file is
/// read when it is sent, so a picture edited in between goes out as the version
/// that existed at Send, which is the one somebody meant. It also keeps a
/// window that has been open all afternoon from holding a hundred megabytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chosen {
    /// Where it is now. Not sent anywhere: the recipient gets the name.
    pub path: PathBuf,
    /// What the recipient will see, which is the file name and nothing else.
    pub name: String,
    /// Its size when it was picked, for saying so and for the running total.
    pub bytes: u64,
}

impl Chosen {
    /// Read a file that has just been picked.
    ///
    /// Fails rather than attaching a name with nothing behind it. A file that
    /// cannot be read now will not read at Send either, and finding that out
    /// then means finding it out after the rest of the message has gone.
    pub fn at(path: &Path) -> Result<Self> {
        Self::looked_at(path).map_err(|why| Error::Other(why.about(path)))
    }

    /// The same read, answering why rather than saying it.
    ///
    /// The one place a path is turned into an attachment. [`Self::at`] is this
    /// with the reason written out as the sentence it has always given, and
    /// [`choose_all`] is this with the reasons grouped, so neither of them
    /// carries a rule of its own about what may go on a message.
    fn looked_at(path: &Path) -> std::result::Result<Self, NotAttached> {
        let data = std::fs::metadata(path).map_err(|e| NotAttached::Unreadable(e.to_string()))?;
        if data.is_dir() {
            return Err(NotAttached::Folder);
        }
        Ok(Self {
            path: path.to_path_buf(),
            name: crate::service::attachment_name::safe_file_name(
                &path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            ),
            bytes: data.len(),
        })
    }

    /// The one sentence a screen reader says when this row is reached.
    ///
    /// The same shape the reader uses for an attachment that arrived, because
    /// they are the same kind of thing and learning one list should teach the
    /// other.
    pub fn label(&self) -> String {
        format!(
            "{}, {}",
            self.name,
            crate::presentation::reader_text::human_size(self.bytes as usize)
        )
    }
}

/// The most files one announcement names before it starts counting them.
///
/// Guardrail 5: feedback has to be bounded. Six names is about ten seconds of
/// speech, which is as long as a routine confirmation should ever be, and past
/// that the total said at the end is the useful fact rather than the roll call.
const NAMED_ALOUD: usize = 6;

/// Why a path did not become an attachment.
///
/// A reason rather than a sentence, because a batch has to group them: five
/// files that would not read, each said in its own sentence, is five
/// interruptions for one thing that went wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
enum NotAttached {
    /// A folder has a size and a name, so everything downstream would work
    /// until the read at Send, which is after the rest of the message went.
    Folder,
    /// What the operating system said. Kept, because for one file it is the
    /// difference between a file that has moved and one this program is not
    /// allowed to open, and those have different answers.
    Unreadable(String),
}

impl NotAttached {
    /// The sentence for this path on its own.
    fn about(&self, path: &Path) -> String {
        match self {
            Self::Folder => format!(
                "{} is a folder, and a folder cannot be attached",
                path.display()
            ),
            Self::Unreadable(why) => format!("Could not read {}: {why}", path.display()),
        }
    }
}

/// What a handful of paths came to.
///
/// A batch is not all or nothing. Somebody who dropped six files and got none
/// back would have to work out for themselves which one was the problem, so
/// everything that reads goes on and everything that did not is said once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Batch {
    /// The files that go on the message, in the order the paths arrived.
    pub chosen: Vec<Chosen>,
    /// Everything that did not go on, in one announcement, or `None` when
    /// everything did.
    pub refused: Option<String>,
}

/// Read several paths at once, keeping whatever reads.
///
/// The one door every route in. A file picked, a file pasted and a file dropped
/// are the same file, so the picker, the paste key and the drop target all end
/// here and here ends at [`Chosen::looked_at`], which is where the refusals and
/// the name cleaning live. Anything building a [`Chosen`] of its own would be a
/// second set of rules for a stranger's path.
pub fn choose_all(paths: &[PathBuf]) -> Batch {
    let mut chosen = Vec::new();
    let mut folders = Vec::new();
    let mut unreadable = Vec::new();
    // The sentence a single refusal would have been given on its own, kept in
    // case it turns out to be the only one.
    let mut on_its_own = None;

    for path in paths {
        match Chosen::looked_at(path) {
            Ok(file) => chosen.push(file),
            Err(why) => {
                // Through `Error` rather than straight from `about`, because
                // the sentence a single refusal is given has to be the one it
                // was given before there were batches, and that one came out
                // of `Chosen::at` and was formatted as an error. It therefore
                // opens with the word "Error", which is poor wording for a
                // thing said out loud and is how every refusal in this program
                // is worded; changing it is a change to `common::Error` and
                // every announcement that goes through it, not to this.
                on_its_own = Some(Error::Other(why.about(path)).to_string());
                match why {
                    NotAttached::Folder => folders.push(refused_name(path)),
                    NotAttached::Unreadable(_) => unreadable.push(refused_name(path)),
                }
            }
        }
    }

    let refused = match (folders.len() + unreadable.len(), paths.len()) {
        (0, _) => None,
        // One path handed over is somebody picking a file, and it keeps the
        // sentence picking a file has always had: the whole path, and the
        // reason the operating system gave, which is the difference between a
        // file that has moved and one this program is not allowed to open.
        //
        // Keyed on how many paths arrived rather than on how many were
        // refused, and the two are not the same question. Three paths with one
        // folder among them is a batch: a whole path read out in the middle of
        // a list of the files that did go on is not a sentence anybody can
        // follow, and it opens with the word "Error" as well.
        (_, 1) => on_its_own,
        _ => Some(refusals(&folders, &unreadable)),
    };

    Batch { chosen, refused }
}

/// What to call a path that did not go on, in a list of several.
///
/// Through the same cleaner an attached name goes through. Nothing is written
/// anywhere with this, so the filesystem rules do not apply, but the
/// bidirectional overrides do: a name that reads backwards is read backwards
/// aloud too, and a refusal is exactly the moment somebody is being asked to
/// recognise a file by hearing its name.
fn refused_name(path: &Path) -> String {
    crate::service::attachment_name::safe_file_name(
        &path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string()),
    )
}

/// One announcement about everything in a batch that did not go on.
///
/// Grouped by the reason rather than listed by file, so six folders dropped
/// among the photos say "these are folders" once instead of six times.
fn refusals(folders: &[String], unreadable: &[String]) -> String {
    let mut said = Vec::new();
    match folders.len() {
        0 => {}
        1 => said.push(format!(
            "{} is a folder, and a folder cannot be attached",
            folders[0]
        )),
        _ => said.push(format!(
            "{} are folders, and a folder cannot be attached",
            named(folders)
        )),
    }
    match unreadable.len() {
        0 => {}
        1 => said.push(format!(
            "{} could not be read, so it is not attached",
            unreadable[0]
        )),
        _ => said.push(format!(
            "{} could not be read, so they are not attached",
            named(unreadable)
        )),
    }
    said.join(". ")
}

/// Several names, said as a list somebody can follow.
///
/// Bounded at [`NAMED_ALOUD`], after which it counts. The names past that point
/// are not information: somebody who dropped forty files knows they dropped
/// forty, and what they are waiting to hear is that forty went on.
fn named(names: &[String]) -> String {
    if names.len() > NAMED_ALOUD {
        let (said, rest) = names.split_at(NAMED_ALOUD);
        return format!("{} and {} others", said.join(", "), rest.len());
    }
    match names.split_last() {
        None => String::new(),
        Some((last, [])) => last.clone(),
        Some((last, first)) => format!("{} and {last}", first.join(", ")),
    }
}

/// One thing said after a batch, and whether it is trouble.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Announcement {
    /// The words, as they are said and as they are shown.
    pub words: String,
    /// Whether this interrupts and is put in front of somebody as well as
    /// said. A file that did not go on and a message that will not send are
    /// both cases where hearing it once, quietly, is not enough.
    pub trouble: bool,
}

/// Everything said after a batch of paths was handed over, in the order it is
/// said.
///
/// A list rather than a loop that speaks as it goes, because "one announcement
/// per batch" is a property of the list and a loop cannot have it: six files
/// each announcing themselves is six interruptions, and six over-the-limit
/// complaints for one message that is too big is worse than none.
///
/// `all` is every file on the message once this batch has gone on, which is
/// what the running total and the limit are about. Whether anything was
/// attached before this batch is read from it rather than passed in, so the two
/// cannot disagree.
pub fn what_to_say(batch: &Batch, all: &[Chosen]) -> Vec<Announcement> {
    let mut said = Vec::new();
    if let Some(words) = attached_sentence(&batch.chosen, all) {
        said.push(Announcement {
            words,
            trouble: false,
        });
    }
    if let Some(words) = batch.refused.clone() {
        said.push(Announcement {
            words,
            trouble: true,
        });
    }
    // Once, at the end, about the message rather than about a file. Asked
    // after every file instead, three files that between them go over would
    // complain three times and the third complaint would say what the first
    // one said.
    if let Some(words) = over_the_limit(all) {
        said.push(Announcement {
            words,
            trouble: true,
        });
    }
    said
}

/// What is said about the files that just went on, or `None` if none did.
///
/// The names, not only the count. A batch that said "3 attachments" and stopped
/// would lose the one thing somebody who cannot see the window is waiting for,
/// which is whether the files that went on are the files they meant.
fn attached_sentence(added: &[Chosen], all: &[Chosen]) -> Option<String> {
    let what = match added {
        [] => return None,
        // One file says exactly what it said before there were batches: the
        // name and the size, which is the whole of what somebody needs to know
        // it is the right file.
        [only] => format!("Attached {}", only.label()),
        several => format!(
            "Attached {}, {} in total",
            named(
                &several
                    .iter()
                    .map(|file| file.name.clone())
                    .collect::<Vec<_>>()
            ),
            crate::presentation::reader_text::human_size(total_bytes(several) as usize)
        ),
    };
    // How to take one off is said once, with the first file to go on, and then
    // not repeated. It belongs on the list, as the description of a control,
    // and a description set that way does not reach the accessibility tree for
    // a native list in this wxWidgets binding: what a screen reader reads there
    // is the line above it. So it is said here, where it is heard.
    //
    // "Nothing was attached before this batch" is read from the message rather
    // than passed in, so the two cannot disagree.
    match all.len() == added.len() {
        true => Some(format!(
            "{what}. Press Delete in the attachments list to take one off"
        )),
        false => Some(what),
    }
}

/// What is said when files are dropped on the message itself.
///
/// The message area is a WebView2 control, and Windows sends a drop to the
/// deepest window that has registered for one. WebView2 registers its own, so a
/// drop there never reaches the composer's, and no arrangement of wxWidgets
/// windows changes that. The page refuses the drop so the engine does not
/// navigate to the file and take a half-written message off the screen, and
/// then says this, because nothing happening and nothing being said is
/// indistinguishable from the feature not existing.
///
/// Trouble, so it is shown as well as said: somebody who dropped a file with a
/// mouse is not necessarily listening to a screen reader, and they are the ones
/// who just watched nothing happen.
///
/// Three sentences and no more: what happened, why, and what to do instead. The
/// last one names the routes that do work rather than only the one that does
/// not, because "that did not work" is not an answer somebody can act on.
pub fn dropped_on_the_message(count: usize) -> Announcement {
    // Zero is a bug rather than an attack: the page counts what it was handed
    // and should never hand over none. Naming a number nobody sent would be
    // worse than not naming one, so it takes the plural without a number.
    let what = match count {
        1 => "That file was not attached.".to_string(),
        0 => "Those files were not attached.".to_string(),
        many => format!("Those {many} files were not attached."),
    };
    Announcement {
        words: format!(
            "{what} The message area is a small browser inside this window, and it takes the \
             drop before the composer sees it. Drop files on the attachments line under the \
             message, or press Ctrl+V, or use Attach File."
        ),
        trouble: true,
    }
}

/// What the line under the message says.
///
/// Said out loud when it changes as well as shown, because somebody who cannot
/// see it has nothing else to tell them the file went on.
pub fn summary(chosen: &[Chosen]) -> String {
    match chosen.len() {
        0 => "No attachments".to_string(),
        1 => format!("1 attachment: {}", chosen[0].label()),
        many => format!(
            "{many} attachments, {} in total",
            crate::presentation::reader_text::human_size(total_bytes(chosen) as usize)
        ),
    }
}

/// Everything picked, added up.
pub fn total_bytes(chosen: &[Chosen]) -> u64 {
    chosen.iter().map(|file| file.bytes).sum()
}

/// Why this will not send, if it will not.
///
/// Checked when a file is added rather than at Send, so the answer arrives
/// while somebody still has the file dialog in mind.
pub fn over_the_limit(chosen: &[Chosen]) -> Option<String> {
    let raw = total_bytes(chosen);
    let encoded = (raw as f64 * ENCODED_OVER_RAW) as u64;
    if encoded <= LIMIT_BYTES {
        return None;
    }
    Some(format!(
        "These files come to {} once encoded, and most providers refuse anything past {}. \
         Take one off, or send a link instead.",
        crate::presentation::reader_text::human_size(encoded as usize),
        crate::presentation::reader_text::human_size(LIMIT_BYTES as usize),
    ))
}

/// The content type to put on the part.
///
/// From the extension, because that is all there is: the file is on this
/// computer and nothing has declared what it holds. Unknown means
/// `application/octet-stream`, which is the honest answer and tells the
/// receiving client to treat it as a file rather than guess.
pub fn content_type(name: &str) -> &'static str {
    let extension = name
        .rsplit_once('.')
        .map(|(_, ext)| ext.trim().to_ascii_lowercase())
        .unwrap_or_default();
    match extension.as_str() {
        "pdf" => "application/pdf",
        "txt" | "log" | "md" => "text/plain",
        "html" | "htm" => "text/html",
        "csv" => "text/csv",
        "ics" => A_CALENDAR_DOCUMENT,
        "eml" => "message/rfc822",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "zip" => "application/zip",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        _ => "application/octet-stream",
    }
}

/// A calendar document that is asking nothing.
///
/// The declaration every `.ics` file here has always carried, and the one a
/// document with no `METHOD` keeps. Named rather than written out twice
/// because [`content_type_of`] asks whether the name landed on this, and an
/// extension table and a comparison against it are two readings of one
/// question if the answer is spelled in both places.
const A_CALENDAR_DOCUMENT: &str = "text/calendar";

/// What a reply's calendar part says it is.
///
/// RFC 6047. The method is the part that matters and the charset is what makes
/// a name that is not written in English survive the journey.
///
/// `method=REPLY` is the whole of it: it is what tells a receiving client this
/// attachment is an answer rather than a calendar file somebody happened to
/// send, and without it the answer is shown as a file to open by hand and
/// never recorded against the meeting.
const AN_ANSWER: &str = "text/calendar; charset=utf-8; method=REPLY";

/// The content type to put on the part, from the name and from what is in it.
///
/// The name decides for everything that is not a calendar document, because
/// for an ordinary file the name is all there is and nothing has declared what
/// it holds. A calendar document is the one kind whose declaration has to
/// agree with what the file says about itself: RFC 6047 requires the `method`
/// parameter on the header and the `METHOD` property inside the document to be
/// the same, and a client meeting a mismatch is entitled to believe neither.
///
/// Worked out here rather than stored, because this is the last place the
/// document and the type it is declared under are in one hand. A queued
/// message carries its files by path and they are read at the moment of
/// sending, so a type settled any earlier is a type that can come to disagree
/// with the document it travels on.
///
/// # Why the answers are fixed strings
///
/// The `METHOD` line comes out of a file somebody else wrote. Every answer
/// here is a `&'static str` chosen from a closed set, so what a stranger wrote
/// selects an answer and never becomes one. Build this with `format!` and the
/// sender of a forwarded invitation controls text on a header line of mail
/// leaving somebody's own account, which is a header of their choosing on a
/// message signed with somebody else's name.
pub fn content_type_of(name: &str, bytes: &[u8]) -> &'static str {
    let from_the_name = content_type(name);
    if from_the_name != A_CALENDAR_DOCUMENT {
        return from_the_name;
    }
    // Bytes that are not text are not asked about. A charset says the bytes
    // really are that encoding, and claiming one this program has just failed
    // to decode is a claim it has no evidence for.
    let Ok(document) = std::str::from_utf8(bytes) else {
        return from_the_name;
    };
    match crate::application::invitations::what_it_asks(document) {
        crate::application::invitations::WhatItAsks::SomebodysAnswer => AN_ANSWER,
        // A document naming no method must not be given one, and the two that
        // are left keep today's answer until the plan that writes theirs.
        crate::application::invitations::WhatItAsks::Invitation
        | crate::application::invitations::WhatItAsks::Cancellation
        | crate::application::invitations::WhatItAsks::SomethingElse => from_the_name,
    }
}

/// A file read and ready to go on a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ready {
    /// What the recipient sees. Through the same cleaner a received name goes
    /// through, because a name with a path in it is a name that came from
    /// somewhere and is about to be written into a header.
    pub name: String,
    pub content_type: &'static str,
    pub bytes: Vec<u8>,
}

/// Read the files a queued message names.
///
/// At the moment of sending rather than when they were picked, so what goes out
/// is the version that exists now. The cost is that a file moved or deleted in
/// between cannot be sent, and this says which one and stops, rather than
/// sending a message that is missing the thing it was written about.
pub fn read_all(paths: &[PathBuf]) -> Result<Vec<Ready>> {
    paths
        .iter()
        .map(|path| {
            let bytes = std::fs::read(path).map_err(|e| {
                Error::Other(format!(
                    "{} could not be read, so the message was not sent: {e}",
                    path.display()
                ))
            })?;
            let name = crate::service::attachment_name::safe_file_name(
                &path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            );
            Ok(Ready {
                // The type is read from the cleaned name, not the name on
                // disk, because the cleaned one is what goes into the part's
                // header. A part named as a document and declared as a
                // program, or the reverse, is what makes a receiving client
                // open a file as something it is not.
                //
                // The cleaning cannot lose a type this knows: it keeps any
                // extension shorter than sixteen characters, and every
                // extension in the table is at most four.
                //
                // The bytes go in as well, because a calendar document is the
                // one kind whose declaration cannot be worked out from a name:
                // whether it is an invitation, an answer or neither is a line
                // inside the file, and no extension can say it. This is the
                // last place the document and the type it is declared under
                // are in one hand, so it is where the two are made to agree.
                content_type: content_type_of(&name, &bytes),
                name,
                bytes,
            })
        })
        .collect()
}

/// Write bytes down as a part, under the name the part is meant to carry.
///
/// The twin of [`read_all`], and here for the same reason it is: the queue
/// carries files by name, so whatever this program sends of its own making has
/// to become a file first, and the name that file lands on is the name the
/// recipient sees. Written where the reading is, so one module holds both ends
/// of that convention rather than the writer choosing a name the reader then
/// has to live with.
///
/// The folder is made if it is not there. The name is used as given: a caller
/// that needs two of these not to collide gives them different folders, so the
/// name can stay the one that says what the file is.
pub fn write_a_part(folder: &Path, name: &str, bytes: &[u8]) -> Result<PathBuf> {
    std::fs::create_dir_all(folder).map_err(|e| {
        Error::Other(format!(
            "{} could not be made, so there was nowhere to put the file: {e}",
            folder.display()
        ))
    })?;
    let at = folder.join(name);
    std::fs::write(&at, bytes)
        .map_err(|e| Error::Other(format!("{} could not be written: {e}", at.display())))?;
    Ok(at)
}

/// What the part is called for anybody whose client shows it as a file.
///
/// A client that understands the content type never shows a name at all. One
/// that does not shows this, so it says what the file is: "invite.ics", which
/// is what most programs write whatever the method, would tell somebody they
/// had been sent an invitation when they had been sent an answer.
pub const WHAT_THE_PART_IS_CALLED: &str = "reply.ics";

/// Where a reply to a meeting invitation is written down so the queue can send
/// it.
///
/// Under the cache directory rather than a temporary folder, because a queued
/// message names its files by path and they are read at the moment of sending:
/// a temporary folder swept in between would take the answer with it.
///
/// `unique` names the folder rather than the file. Two answers waiting at once
/// must not land on one file and have the second replace the first while the
/// first is still waiting to go, and a unique name in the file would buy that
/// by giving the recipient the unique name to look at. So the uniqueness sits
/// one level up and the file keeps the name that says what it is.
///
/// `unique` is not the moment the answer was written: a date written out here
/// is a date written in a second place, and this program keeps one writer for
/// those on purpose.
pub fn a_place_for_the_reply(cache_dir: &Path, unique: &str, document: &str) -> Result<PathBuf> {
    write_a_part(
        &cache_dir.join("answers").join(unique),
        WHAT_THE_PART_IS_CALLED,
        document.as_bytes(),
    )
}

/// The paths, joined so one text column can hold them.
///
/// The outbox is a table with one row per message, and a message can carry
/// several files. A newline separates them because it is the one character a
/// Windows path cannot contain, so nothing needs escaping and nothing can be
/// split in the wrong place.
///
/// Takes paths rather than picked files, because that is what the composer is
/// holding by the time a message is queued. It took picked files before, which
/// meant nothing could call it and the queue built the same string by hand: two
/// copies of one convention, either of which could change without the other.
pub fn joined(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The paths, back out of the column.
pub fn split(stored: &str) -> Vec<PathBuf> {
    stored
        .split('\n')
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sized(name: &str, bytes: u64) -> Chosen {
        Chosen {
            path: PathBuf::from(format!("C:\\files\\{name}")),
            name: name.to_string(),
            bytes,
        }
    }

    /// An invitation of the ordinary shape, as a calendar server writes one.
    ///
    /// A third copy of the same text, and the reason is worth writing down
    /// rather than leaving as an accident. The other two sit in the test
    /// modules of `answering.rs` and `invitations.rs` and are byte-identical.
    /// Sharing one would mean reaching into another file's test module, which
    /// nothing in this tree does: every cross-module fixture here is
    /// `super::tests::`, inside a single file. The alternative, a shared test
    /// module, couples three test modules so that a change made for one
    /// breaks the others.
    ///
    /// What matters is that the reply this file writes down is the one the
    /// program really builds. So the document is not written here: it comes
    /// out of `the_answer_to_send`, the way it does when somebody presses
    /// Accept, and only the invitation it answers is a fixture.
    fn an_invitation_that_arrived() -> String {
        "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\nMETHOD:REQUEST\r\n\
         BEGIN:VEVENT\r\nUID:m-1@example.com\r\nSEQUENCE:2\r\n\
         SUMMARY:Quarterly review\r\nLOCATION:Room 3\r\n\
         DTSTART:20260305T090000Z\r\nDTEND:20260305T100000Z\r\n\
         ORGANIZER;CN=Ada Lovelace:mailto:ada@example.com\r\n\
         ATTENDEE;CN=Sam;PARTSTAT=NEEDS-ACTION;RSVP=TRUE:mailto:sam@example.com\r\n\
         ATTENDEE;CN=Kit;PARTSTAT=NEEDS-ACTION:mailto:kit@example.com\r\n\
         END:VEVENT\r\nEND:VCALENDAR\r\n"
            .to_string()
    }

    /// The reply document somebody accepting that meeting really sends.
    ///
    /// Built through the two public steps the window uses, so what is written
    /// down and read back below is the program's own answer rather than a
    /// calendar document written to suit the test.
    fn the_answer_somebody_accepted_with() -> String {
        use crate::application::allowed::Allowed;
        use crate::application::answering::whether_it_can_be_answered;
        use crate::application::invitations::Answer;

        whether_it_can_be_answered(
            &an_invitation_that_arrived(),
            "sam@example.com",
            Allowed::EVERYTHING,
        )
        .expect("an invitation that can be answered")
        .the_answer_to_send(
            Answer::Accepted,
            "2026-03-04T08:15:00Z"
                .parse()
                .expect("a moment written the way this test wrote it"),
        )
        .expect("an answer to send")
        .calendar_document
    }

    #[test]
    fn test_nothing_attached_says_so() {
        assert_eq!(summary(&[]), "No attachments");
    }

    #[test]
    fn test_a_drop_on_the_message_says_where_a_drop_does_work() {
        // "Nothing happened" is not an answer somebody can act on, and it is
        // exactly what a dead zone gives them. So the sentence names the three
        // routes that do work, and names them by what somebody would look for
        // rather than by what the code calls them.
        let said = dropped_on_the_message(1);

        assert!(
            said.words.contains("Ctrl+V"),
            "the quickest route is not named: {}",
            said.words
        );
        assert!(
            said.words.contains("Attach File"),
            "the button is not named, so somebody who has never found it still has not: {}",
            said.words
        );
        assert!(
            said.words.contains("attachments line"),
            "the place a drop does land is not named, so this says what does not work and \
             leaves somebody to guess what does: {}",
            said.words
        );
        assert!(
            said.trouble,
            "the refusal is spoken and not shown, so the person who used a mouse to drop the \
             file and is not listening to a screen reader gets nothing at all"
        );
    }

    #[test]
    fn test_a_drop_on_the_message_says_why_nothing_happened() {
        // What happened, why, and what to do next: the same three things every
        // other refusal in this program owes somebody. The why matters here
        // more than usual, because without it the honest reading is that
        // dropping is broken rather than that it lands somewhere else.
        let said = dropped_on_the_message(1);

        assert!(
            said.words.starts_with("That file was not attached."),
            "what happened is not the first thing said: {}",
            said.words
        );
        assert!(
            said.words.contains("small browser"),
            "why it did not happen is not said: {}",
            said.words
        );
    }

    #[test]
    fn test_a_drop_of_several_says_how_many_did_not_go_on() {
        // Somebody who dragged four files and heard "that file was not
        // attached" would reasonably go looking for the other three.
        assert!(
            dropped_on_the_message(4)
                .words
                .starts_with("Those 4 files were not attached."),
            "{}",
            dropped_on_the_message(4).words
        );
    }

    #[test]
    fn test_a_drop_that_carried_nothing_countable_still_says_what_to_do() {
        // The page counts what it was handed and the count can be zero, which
        // is a bug rather than an attack. Naming a number nobody sent would be
        // worse than not naming one, and saying nothing at all would leave
        // somebody watching a drop do nothing.
        let said = dropped_on_the_message(0);

        assert!(
            said.words.starts_with("Those files were not attached."),
            "{}",
            said.words
        );
        assert!(said.words.contains("Ctrl+V"), "{}", said.words);
    }

    #[test]
    fn test_one_file_is_named_and_measured() {
        // The whole of what somebody needs to know it is the right file, in
        // the line they are already being read.
        assert_eq!(
            summary(&[sized("report.pdf", 245_760)]),
            "1 attachment: report.pdf, 240 KB"
        );
    }

    #[test]
    fn test_several_files_are_counted_and_added_up() {
        // Not every name: five names read out on every change is a paragraph
        // where a number was wanted, and the list beside it holds the names.
        assert_eq!(
            summary(&[sized("a.pdf", 1024), sized("b.png", 1024)]),
            "2 attachments, 2 KB in total"
        );
    }

    #[test]
    fn test_a_message_under_the_limit_is_not_complained_about() {
        assert_eq!(over_the_limit(&[sized("a.pdf", 1024 * 1024)]), None);
    }

    #[test]
    fn test_the_limit_counts_what_goes_on_the_wire() {
        // 20 MB on disk is about 27 MB encoded, which Gmail refuses. Checking
        // the size on disk would call this fine and let somebody find out from
        // a bounce an hour later.
        let twenty_megabytes = sized("film.mp4", 20 * 1024 * 1024);

        let complaint = over_the_limit(&[twenty_megabytes]).expect("20 MB should be too much");

        // The figure read out is the size on the wire, not the size on disk,
        // because that is the number the provider is measuring against.
        assert!(complaint.contains("26.7 MB"), "{complaint}");
        assert!(complaint.contains("25 MB"), "{complaint}");
        assert!(complaint.contains("send a link"), "{complaint}");
    }

    #[test]
    fn test_a_file_that_still_fits_once_encoded_is_left_alone() {
        // The other side of the same sum. 18 MB on disk is 24 MB encoded, which
        // fits, and complaining about it would send somebody looking for a file
        // to remove that did not need removing.
        assert_eq!(over_the_limit(&[sized("scan.tif", 18 * 1024 * 1024)]), None);
    }

    #[test]
    fn test_the_type_comes_from_the_extension() {
        // Every extension this knows, not a sample of them. Four of twenty were
        // checked before, so any of the other sixteen could have quietly become
        // "a file" and the recipient's client would have offered to save it
        // rather than open it.
        for (name, expected) in [
            ("report.pdf", "application/pdf"),
            ("Photo.JPG", "image/jpeg"),
            ("scan.jpeg", "image/jpeg"),
            ("notes.txt", "text/plain"),
            ("run.log", "text/plain"),
            ("readme.md", "text/plain"),
            ("page.html", "text/html"),
            ("page.htm", "text/html"),
            ("rows.csv", "text/csv"),
            ("meeting.ics", "text/calendar"),
            ("forwarded.eml", "message/rfc822"),
            ("shot.png", "image/png"),
            ("loop.gif", "image/gif"),
            ("logo.svg", "image/svg+xml"),
            ("photo.webp", "image/webp"),
            ("bundle.zip", "application/zip"),
            ("letter.doc", "application/msword"),
            (
                "letter.docx",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ),
            ("book.xls", "application/vnd.ms-excel"),
            (
                "sheet.xlsx",
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            ),
            ("deck.ppt", "application/vnd.ms-powerpoint"),
            (
                "deck.pptx",
                "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            ),
        ] {
            assert_eq!(content_type(name), expected, "for {name}");
        }
    }

    #[test]
    fn test_a_type_nobody_knows_is_admitted_to_rather_than_guessed() {
        // octet-stream tells the receiving client to treat it as a file. A
        // guess would tell it to open something as the wrong kind.
        assert_eq!(content_type("archive.qqq"), "application/octet-stream");
        assert_eq!(content_type("no-extension"), "application/octet-stream");
    }

    #[test]
    fn test_the_paths_the_composer_hands_over_survive_one_column() {
        // The paths, not the picked files. By the time a message is queued the
        // composer holds paths and nothing else, so a round trip that starts
        // from a picked file is a trip nothing ever makes.
        let picked = vec![
            PathBuf::from("C:\\files\\a.pdf"),
            PathBuf::from("C:\\my files\\b b.png"),
        ];

        let back = split(&joined(&picked));

        assert_eq!(back, picked);
    }

    #[test]
    fn test_an_empty_column_is_no_attachments_rather_than_one_with_no_name() {
        assert!(split("").is_empty());
        assert!(split("\n\n").is_empty());
    }

    #[test]
    fn test_a_folder_cannot_be_attached() {
        // A folder has a size and a name, so everything downstream would work
        // until the read at Send, which is after the rest of the message went.
        let folder = tempfile::tempdir().expect("temp dir");

        let refusal = Chosen::at(folder.path()).expect_err("a folder is not a file");

        assert!(format!("{refusal}").contains("folder"), "{refusal}");
    }

    #[test]
    fn test_a_file_that_is_not_there_is_refused_now_rather_than_at_send() {
        let missing = std::path::Path::new("C:\\nothing\\here\\at\\all.pdf");

        assert!(Chosen::at(missing).is_err());
    }

    #[test]
    fn test_the_file_is_read_at_the_moment_of_sending() {
        // Not when it was picked. A message is written over minutes, and the
        // version somebody meant to send is the one that exists at Send.
        let folder = tempfile::tempdir().expect("temp dir");
        let file = folder.path().join("notes.txt");
        std::fs::write(&file, b"first").expect("write");
        std::fs::write(&file, b"second, after some editing").expect("rewrite");

        let ready = read_all(&[file]).expect("a real file");

        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].name, "notes.txt");
        assert_eq!(ready[0].content_type, "text/plain");
        assert_eq!(ready[0].bytes, b"second, after some editing");
    }

    #[test]
    fn test_a_reply_arrives_declared_as_a_reply_so_the_organisers_client_records_it() {
        // The content type is what tells a receiving client that this
        // attachment is an answer rather than a calendar file somebody
        // happened to send. Without `method=REPLY` it is shown as a file to
        // open by hand, and the answer is never recorded against the meeting,
        // which is the whole point of sending it.
        //
        // That used to be asserted about a function nothing in the running
        // program called, so it passed for as long as the feature was broken.
        // Every hop this drives is one the answer really takes: the document
        // is built the way pressing Accept builds it, written down the way the
        // window writes it, and read back the way the send loop reads it.
        let document = the_answer_somebody_accepted_with();
        let folder = tempfile::tempdir().expect("temp dir");

        let written = a_place_for_the_reply(folder.path(), "one", &document)
            .expect("somewhere to put the answer");
        let ready = read_all(&[written]).expect("the answer, read back");

        assert_eq!(ready.len(), 1);
        assert_eq!(
            ready[0].content_type,
            "text/calendar; charset=utf-8; method=REPLY"
        );
        assert_eq!(ready[0].name, "reply.ics");
        assert_eq!(
            String::from_utf8(ready[0].bytes.clone()).expect("a calendar document is text"),
            document
        );
    }

    #[test]
    fn test_a_calendar_document_asking_nothing_is_declared_the_way_it_always_was() {
        // A published feed saved as a file and attached is a calendar document
        // that is not a question. Giving it a method would tell the recipient's
        // client to act on something nobody asked, so it keeps the declaration
        // an `.ics` file has always had here.
        let feed = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Example//EN\r\n\
                    BEGIN:VEVENT\r\nUID:f-1@example.com\r\nSUMMARY:Bank holiday\r\n\
                    DTSTART:20260601T000000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        assert_eq!(
            content_type_of("holidays.ics", feed.as_bytes()),
            "text/calendar"
        );
    }

    #[test]
    fn test_a_calendar_file_that_is_not_text_claims_no_charset_it_cannot_stand_behind() {
        // A charset says the bytes really are that encoding. These are not, so
        // saying `charset=utf-8` would be this program asserting something it
        // has just failed to verify, and a receiving client that believes it
        // shows the recipient nonsense rather than a file it could not read.
        let not_text = [0x00u8, 0xff, 0xfe, 0x80, 0x41];

        let declared = content_type_of("diary.ics", &not_text);

        assert_eq!(declared, "text/calendar");
        assert!(!declared.contains("charset"), "{declared}");
    }

    #[test]
    fn test_two_answers_waiting_at_once_do_not_land_on_one_file() {
        // The second replacing the first while the first is still waiting to
        // go would send one person's answer twice and the other's never.
        let document = the_answer_somebody_accepted_with();
        let folder = tempfile::tempdir().expect("temp dir");

        let first = a_place_for_the_reply(folder.path(), "one", &document).expect("the first");
        let second = a_place_for_the_reply(folder.path(), "two", &document).expect("the second");

        assert_ne!(first, second);
        let ready = read_all(&[first, second]).expect("both, read back");
        assert_eq!(ready.len(), 2);
        for part in &ready {
            assert_eq!(part.name, "reply.ics");
            assert_eq!(
                String::from_utf8(part.bytes.clone()).expect("a calendar document is text"),
                document
            );
        }
    }

    #[test]
    fn test_a_file_that_has_gone_stops_the_send_rather_than_thinning_it() {
        // Sending the message without the file it was written about is the
        // worst of the three outcomes, and the one nobody would notice.
        let gone = PathBuf::from("C:\\moved\\away\\report.pdf");

        let refusal = read_all(&[gone]).expect_err("a file that is not there");

        let said = format!("{refusal}");
        assert!(said.contains("report.pdf"), "{said}");
        assert!(said.contains("not sent"), "{said}");
    }

    #[test]
    fn test_the_type_always_describes_the_name_that_goes_out() {
        // The two are written onto the same part, so they have to agree. A
        // long name is shortened on the way out, and if the shortening dropped
        // the extension the file would arrive declared as no particular kind
        // and the recipient would be offered a save box instead of the
        // document.
        let folder = tempfile::tempdir().expect("temp dir");
        let long = folder.path().join(format!("{}.pdf", "x".repeat(200)));
        std::fs::write(&long, b"a report").expect("write");
        let odd = folder.path().join("annexe\u{202E}cod.txt");
        std::fs::write(&odd, b"notes").expect("write");

        let ready = read_all(&[long, odd]).expect("real files");

        for file in &ready {
            assert_eq!(
                content_type(&file.name),
                file.content_type,
                "the declared type disagrees with the declared name: {}",
                file.name
            );
        }
        assert_eq!(ready[0].content_type, "application/pdf");
        assert_eq!(ready[1].content_type, "text/plain");
    }

    #[test]
    fn test_cleaning_a_name_cannot_take_its_type_away() {
        // The one way the type could be lost: a name so long that shortening
        // it dropped the extension. It does not, and this is what says so.
        let cleaned =
            crate::service::attachment_name::safe_file_name(&format!("{}.pdf", "x".repeat(200)));

        assert_eq!(content_type(&cleaned), "application/pdf");
    }

    #[test]
    fn test_a_picked_file_is_named_and_measured() {
        let folder = tempfile::tempdir().expect("temp dir");
        let file = folder.path().join("report.pdf");
        std::fs::write(&file, vec![0u8; 2048]).expect("write");

        let chosen = Chosen::at(&file).expect("a real file");

        assert_eq!(chosen.name, "report.pdf");
        assert_eq!(chosen.bytes, 2048);
        assert_eq!(chosen.label(), "report.pdf, 2 KB");
    }

    // ── Several files at once ──────────────────────────────────────────────
    //
    // A trap the plan named and it is worth naming again here: `Chosen::at`
    // already refuses a folder and already refuses a file that will not read,
    // so a test handing over one bad path and asserting it is refused would be
    // green against any loop at all and would say nothing. What carries
    // information is the batch: that the others still go on, that the refusals
    // arrive as one announcement rather than one each, and that the running
    // total and the complaint about size are asked once at the end.

    /// A folder with `count` real files in it, named `file-0.txt` upwards.
    fn some_files(folder: &std::path::Path, count: usize, each: usize) -> Vec<PathBuf> {
        (0..count)
            .map(|n| {
                let path = folder.join(format!("file-{n}.txt"));
                std::fs::write(&path, vec![0u8; each]).expect("write");
                path
            })
            .collect()
    }

    #[test]
    fn test_several_files_go_on_in_the_order_the_paths_arrived() {
        let folder = tempfile::tempdir().expect("temp dir");
        let paths = some_files(folder.path(), 3, 10);

        let batch = choose_all(&paths);

        assert_eq!(
            batch
                .chosen
                .iter()
                .map(|file| file.name.as_str())
                .collect::<Vec<_>>(),
            ["file-0.txt", "file-1.txt", "file-2.txt"]
        );
        assert_eq!(batch.refused, None);
    }

    #[test]
    fn test_a_folder_among_them_is_refused_by_name_and_the_others_still_go_on() {
        // The half that matters is "and the others still go on". Somebody who
        // dropped three files and got none would have to find out for
        // themselves which one was the problem.
        let folder = tempfile::tempdir().expect("temp dir");
        let files = some_files(folder.path(), 2, 10);
        let inner = folder.path().join("holiday photos");
        std::fs::create_dir(&inner).expect("a folder among the files");
        let paths = vec![files[0].clone(), inner, files[1].clone()];

        let batch = choose_all(&paths);

        assert_eq!(
            batch
                .chosen
                .iter()
                .map(|file| file.name.as_str())
                .collect::<Vec<_>>(),
            ["file-0.txt", "file-1.txt"]
        );
        let refusal = batch.refused.expect("the folder to be refused");
        assert!(refusal.contains("holiday photos"), "{refusal}");
        assert!(refusal.contains("folder"), "{refusal}");
    }

    #[test]
    fn test_a_file_that_cannot_be_read_among_them_does_not_stop_the_others() {
        let folder = tempfile::tempdir().expect("temp dir");
        let files = some_files(folder.path(), 2, 10);
        let missing = folder.path().join("gone.pdf");
        let paths = vec![files[0].clone(), missing, files[1].clone()];

        let batch = choose_all(&paths);

        assert_eq!(batch.chosen.len(), 2);
        let refusal = batch.refused.expect("the missing file to be refused");
        assert!(refusal.contains("gone.pdf"), "{refusal}");
    }

    #[test]
    fn test_two_that_did_not_go_on_are_one_announcement_rather_than_two() {
        // Six unreadable files should not be six interruptions. The list is
        // one string for exactly that reason, and both names are in it.
        let folder = tempfile::tempdir().expect("temp dir");
        let inner = folder.path().join("music");
        std::fs::create_dir(&inner).expect("a folder");
        let paths = vec![inner, folder.path().join("gone.pdf")];

        let batch = choose_all(&paths);

        assert!(batch.chosen.is_empty());
        let refusal = batch.refused.expect("both to be refused");
        assert!(refusal.contains("music"), "{refusal}");
        assert!(refusal.contains("gone.pdf"), "{refusal}");
    }

    #[test]
    fn test_one_path_is_refused_in_the_same_words_it_was_refused_in_before() {
        // Picking one file has to behave exactly as it did before this, and
        // the refusal is the part most easily lost: the full path and the
        // reason the operating system gave are worth more than a bare name
        // when there is only one thing to say.
        let folder = tempfile::tempdir().expect("temp dir");
        let inner = folder.path().join("photos");
        std::fs::create_dir(&inner).expect("a folder");

        let batch = choose_all(std::slice::from_ref(&inner));

        let alone = format!(
            "{}",
            Chosen::at(&inner).expect_err("a folder is not a file")
        );
        assert_eq!(batch.refused, Some(alone));
    }

    #[test]
    fn test_what_a_batch_of_three_says_out_loud() {
        // The whole sentence, not the parts it contains. Everything else here
        // asks whether a name is in the announcement, which cannot see the
        // punctuation, the order, or a stray word between two of them, and
        // this is a sentence somebody hears rather than reads: a comma where
        // "and" belongs is heard.
        let folder = tempfile::tempdir().expect("temp dir");
        let paths = some_files(folder.path(), 3, 2048);

        let batch = choose_all(&paths);
        let said = what_to_say(&batch, &batch.chosen);

        assert_eq!(said.len(), 1, "{said:?}");
        assert_eq!(
            said[0].words,
            "Attached file-0.txt, file-1.txt and file-2.txt, 6 KB in total. \
             Press Delete in the attachments list to take one off"
        );
        assert!(!said[0].trouble);

        // And the same three with a folder in the middle, which is two
        // announcements: what went on, then what did not.
        let inner = folder.path().join("holiday photos");
        std::fs::create_dir(&inner).expect("a folder among the files");
        let mixed = choose_all(&[paths[0].clone(), inner, paths[1].clone()]);
        let said = what_to_say(&mixed, &mixed.chosen);

        assert_eq!(
            said.iter()
                .map(|words| (words.words.as_str(), words.trouble))
                .collect::<Vec<_>>(),
            [
                (
                    "Attached file-0.txt and file-1.txt, 4 KB in total. \
                     Press Delete in the attachments list to take one off",
                    false
                ),
                (
                    "holiday photos is a folder, and a folder cannot be attached",
                    true
                ),
            ]
        );
    }

    #[test]
    fn test_one_bad_path_among_several_is_named_rather_than_pathed() {
        // The other half of the test above, and it took writing the summary to
        // see that they are different questions. Handing over one path is
        // picking a file and keeps the sentence picking a file has always had,
        // whole path and all. Handing over three is a batch, and a batch says
        // names: the whole path of one file read out in the middle of a list of
        // the ones that did go on is not a sentence anybody can follow, and it
        // opens with the word "Error" because that is how `common::Error`
        // formats itself.
        let folder = tempfile::tempdir().expect("temp dir");
        let files = some_files(folder.path(), 2, 10);
        let inner = folder.path().join("holiday photos");
        std::fs::create_dir(&inner).expect("a folder among the files");

        let batch = choose_all(&[files[0].clone(), inner, files[1].clone()]);

        assert_eq!(
            batch.refused,
            Some("holiday photos is a folder, and a folder cannot be attached".to_string())
        );
    }

    #[test]
    fn test_a_batch_names_the_files_that_went_on_rather_than_only_counting_them() {
        // A batch that said only "3 attachments" would lose the one thing
        // somebody who cannot see the window needs: whether the files that
        // went on are the files they meant.
        let folder = tempfile::tempdir().expect("temp dir");
        let paths = some_files(folder.path(), 3, 1024);

        let batch = choose_all(&paths);
        let said = what_to_say(&batch, &batch.chosen);

        let words = said.first().expect("something said about a batch of three");
        assert!(words.words.contains("file-0.txt"), "{}", words.words);
        assert!(words.words.contains("file-1.txt"), "{}", words.words);
        assert!(words.words.contains("file-2.txt"), "{}", words.words);
        assert!(!words.trouble, "attaching three files is not trouble");
    }

    #[test]
    fn test_how_to_take_one_off_is_said_with_the_first_batch_and_not_again() {
        let folder = tempfile::tempdir().expect("temp dir");
        let paths = some_files(folder.path(), 2, 1024);

        let first = choose_all(&paths[..1]);
        let opening = what_to_say(&first, &first.chosen);
        assert!(
            opening
                .iter()
                .any(|said| said.words.contains("Press Delete")),
            "{opening:?}"
        );

        let second = choose_all(&paths[1..]);
        let mut everything = first.chosen.clone();
        everything.extend(second.chosen.clone());
        let later = what_to_say(&second, &everything);
        assert!(
            !later.iter().any(|said| said.words.contains("Press Delete")),
            "{later:?}"
        );
    }

    #[test]
    fn test_a_batch_says_one_thing_about_being_too_big_rather_than_one_a_file() {
        // The defect this is written against is a loop that asks
        // `over_the_limit` after every file: three files that between them go
        // over would complain three times, and the third complaint says the
        // same thing as the first.
        let over = LIMIT_BYTES; // Over once encoded, whatever the base64 does.
        let batch = Batch {
            chosen: vec![
                sized("one.bin", over / 2),
                sized("two.bin", over / 2),
                sized("three.bin", over / 2),
            ],
            refused: None,
        };

        let said = what_to_say(&batch, &batch.chosen);

        let complaints: Vec<_> = said
            .iter()
            .filter(|words| words.words.contains("providers refuse"))
            .collect();
        assert_eq!(complaints.len(), 1, "{said:?}");
        assert!(
            complaints[0].trouble,
            "a message that will not send is trouble"
        );
    }

    #[test]
    fn test_a_batch_too_long_to_sit_through_names_the_first_few_and_counts_the_rest() {
        // Guardrail 5. Ten names is twenty seconds of speech for a
        // confirmation, and a screen reader saying it is a screen reader that
        // cannot be used for twenty seconds.
        let folder = tempfile::tempdir().expect("temp dir");
        let paths = some_files(folder.path(), 10, 16);

        let batch = choose_all(&paths);
        let said = what_to_say(&batch, &batch.chosen);

        let words = &said.first().expect("something said").words;
        assert!(words.contains("file-5.txt"), "{words}");
        assert!(!words.contains("file-6.txt"), "{words}");
        assert!(words.contains("4 others"), "{words}");
    }

    #[test]
    fn test_nothing_going_on_says_only_what_did_not() {
        let folder = tempfile::tempdir().expect("temp dir");
        let inner = folder.path().join("photos");
        std::fs::create_dir(&inner).expect("a folder");

        let batch = choose_all(&[inner]);
        let said = what_to_say(&batch, &[]);

        assert_eq!(said.len(), 1, "{said:?}");
        assert!(said[0].words.contains("folder"), "{said:?}");
        assert!(said[0].trouble, "a file that did not go on is trouble");
    }

    #[test]
    fn test_a_dropped_name_written_backwards_reaches_the_recipient_forwards() {
        // T-04-27. A path handed over by a drop or a paste was chosen by
        // whoever did the dragging, and the name on it is written into
        // somebody else's mailbox. The override is the case that matters most
        // here: a synthesiser reading the reordered name aloud gives no hint
        // at all that it was reordered.
        //
        // This is the one of the register's two fixtures that can be a real
        // file on Windows. A path that walks out of its folder never reaches
        // the cleaner as a path, because `Chosen::at` asks for the last
        // component; and a file named for a device cannot be created at all,
        // so no drop can produce one. Both rules are tested where they live,
        // in service::attachment_name.
        let folder = tempfile::tempdir().expect("temp dir");
        let odd = folder.path().join("annexe\u{202E}cod.exe");
        std::fs::write(&odd, b"nothing much").expect("write");

        let batch = choose_all(&[odd]);

        assert_eq!(batch.chosen.len(), 1);
        assert_eq!(batch.chosen[0].name, "annexecod.exe");
    }
}

//! Bringing messages in from a file, and writing them out to one.
//!
//! [`crate::application::message_files`] reads and writes the files. This
//! decides what to do about what it finds: whether there is anywhere to put the
//! mail, where it goes, what to do about a message that is here already, and
//! what somebody is told at the end.
//!
//! Nothing here opens a file or talks to a server, and one thing here writes to
//! the database: [`file_one_imported_message`], which is the single place a
//! message read out of a file becomes a row in a folder. It lives here rather
//! than in the window because getting it wrong loses somebody's mail quietly,
//! and here it can be run against a real database in a test.
//!
//! Nothing here reaches a server at all, which is worth saying plainly: an
//! import writes to this computer and stops, so it is not one of the changes
//! [`crate::application::allowed`] gates.
//!
//! # The one question that can lose somebody's mail
//!
//! A sync lists what a folder holds at the server, compares it with what is
//! stored here, and removes the rows the server does not have. A message read
//! out of a file has never been on anybody's server. So on that comparison it
//! is a message the server no longer has, and the next check for mail takes
//! away the archive somebody imported an hour ago, along with its text, which
//! for these rows is the only copy there was.
//!
//! What stops it is the marker the database puts on a row this program filed
//! itself, the same one a copy of a sent message carries. A marked row is left
//! out of the list the sync compares against, so it is on neither side of the
//! comparison, and the statement that forgets a message refuses a marked row as
//! well. Both halves have to be got wrong for the mail to go.
//!
//! [`WrittenDownAs`] is that decision, and it is spelled so there is no way to
//! say anything else: both of its answers file the row here, and what differs
//! between them is only which end of the folder's numbering the message takes a
//! number from.
//!
//! The imported calendar answers the same question differently, by filing
//! events under the local account and a calendar of its own, which is what
//! [`crate::application::opening::WHERE_IMPORTED_THINGS_GO`] names. Mail cannot
//! copy that. Somebody importing an archive wants it in a folder they already
//! read, and an account has the folders it has.
//!
//! # What a large file costs
//!
//! A mailbox somebody has kept for twenty years is the largest file this
//! program opens. Nothing here holds one: the file is read a message at a time,
//! each message is filed and let go, and what this module keeps is a handful of
//! counts and the identifiers already in the folder.

use crate::application::message_files;
use crate::application::summing_up::SummingUp;
use crate::common::Result;
use crate::common::types::FolderType;

/// What to say when a file of mail is chosen and no account is.
///
/// A message is filed in a folder and a folder belongs to an account, so with
/// no account there is nowhere for anything to go. The contacts import already
/// says its own version of this, and it exists because the alternative was an
/// import that reported success into a corner of the database nothing reads.
pub const CHOOSE_AN_ACCOUNT_FIRST: &str =
    "Choose an account first. Imported mail is filed in one of that account's folders.";

/// What to say when an account is chosen and no folder is.
///
/// Mail that arrives has its folder chosen by the server it came from. Mail
/// read out of a file has nobody to choose one, and picking the inbox on
/// somebody's behalf files twenty years of their archive in among the mail they
/// have not read yet.
pub const CHOOSE_A_FOLDER_FIRST: &str = "Choose the folder to import into. Mail read from a file is filed in a folder, \
     the same as mail that arrives.";

// ── Where the mail goes ─────────────────────────────────────────────────────

/// How a message read out of a file is written into a folder.
///
/// Every answer files the row here, and there is deliberately no way to spell
/// anything else. That is the whole of what keeps imported mail from being
/// removed by the next sync, and the note at the top of this file is where the
/// reasoning is.
///
/// What is left to decide is which end of the folder's numbering the message
/// takes a number from, and that depends on whether a server numbers the folder
/// as well.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrittenDownAs {
    /// Filed here, numbered downward from the top of the range.
    ///
    /// For a folder a server also fills. Counting upward from the highest
    /// number in use would take the number that server is about to hand out,
    /// and then two things go wrong quietly at once: the sync sees that number
    /// as already held and never fetches the real message, and anything that
    /// did fetch it writes over the imported one.
    FiledHereCountingDownFromTheTop,
    /// Filed here, numbered upward from the highest number in use.
    ///
    /// For a folder that lives on this computer, which no server numbers, so
    /// there is nothing at the top of the range to reserve anything from.
    FiledHereCountingUp,
}

impl WrittenDownAs {
    /// How a message imported into this folder is written down.
    ///
    /// Which folders live on this computer is asked of the one place that knows,
    /// rather than answered again from the shape of the path, so a folder added
    /// there is a folder this answers about.
    pub fn for_folder(path: &str) -> Self {
        if crate::application::local_folders::is_local(path) {
            return Self::FiledHereCountingUp;
        }
        Self::FiledHereCountingDownFromTheTop
    }
}

/// What to say when the folder chosen is the one holding mail on its way out.
///
/// The Outbox is how somebody reaches the send queue, and what it lists comes
/// from that queue rather than from the messages filed in it. A message
/// imported there is written down, counted, and never seen again.
pub const NOT_INTO_THE_OUTBOX: &str = "The Outbox holds mail waiting to be sent, not mail to read. \
     Choose another folder to import into.";

/// The folder somebody chose to import into.
///
/// The path and the kind travel together because both are needed and neither
/// can be worked out from the other: a folder on this computer is known by its
/// path, and what a folder is for is a column of its own that a server names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FolderChosen<'a> {
    /// The path it is stored under.
    pub path: &'a str,
    /// What the folder is for.
    pub kind: FolderType,
}

/// How much of the file is read at once.
///
/// The difference is not a detail of the parsing. A mailbox somebody has kept
/// for twenty years is the largest file this program opens, and reading it into
/// a list of messages before writing any of them down holds all of it twice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadAs {
    /// One message, which is what a file saved out of a mail program holds.
    OneMessage,
    /// One message at a time, which is what an archive of a mailbox needs: each
    /// message is filed and let go before the next is read.
    OneAtATimeFromAnArchive,
}

/// What an import of one file will do, or the one sentence saying why it will
/// not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Importing {
    /// Nothing, and this is what to say.
    Refused(&'static str),
    /// This, and this is all of it.
    GoAhead(HowItLands),
}

/// Where the mail in a file is filed, and how each row is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HowItLands {
    /// How much of the file is read at once.
    pub read: ReadAs,
    /// The account whose folder holds the mail.
    pub account: String,
    /// The path of the folder within that account.
    pub folder: String,
    /// How each row is written down.
    pub written_down_as: WrittenDownAs,
}

/// What bringing in this file would do.
///
/// The choices are asked about in the order somebody makes them, so the
/// sentence names the first one still missing and leads them through it rather
/// than listing everything at once.
pub fn importing(
    bytes: &[u8],
    account: Option<&str>,
    folder: Option<FolderChosen<'_>>,
) -> Importing {
    use message_files::FileHolds;

    let Some(account) = account else {
        return Importing::Refused(CHOOSE_AN_ACCOUNT_FIRST);
    };
    let Some(folder) = folder else {
        return Importing::Refused(CHOOSE_A_FOLDER_FIRST);
    };
    if folder.kind == FolderType::Outbox {
        return Importing::Refused(NOT_INTO_THE_OUTBOX);
    }
    // From the bytes rather than the name, which is `message_files`' rule and
    // holds here for the same reason: mail arrives named `.txt` and with no
    // extension at all, and a picture arrives named `.eml`.
    let read = match message_files::what_the_file_holds(bytes) {
        FileHolds::NotMail => return Importing::Refused(message_files::NOT_A_MAIL_FILE),
        FileHolds::OneMessage => ReadAs::OneMessage,
        FileHolds::ManyMessages => ReadAs::OneAtATimeFromAnArchive,
    };
    Importing::GoAhead(HowItLands {
        read,
        account: account.to_string(),
        folder: folder.path.to_string(),
        written_down_as: WrittenDownAs::for_folder(folder.path),
    })
}

// ── One message at a time ───────────────────────────────────────────────────

/// What to do with one message an import has read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatToDoWithIt {
    /// Write it down. This folder holds nothing carrying its identifier.
    BringItIn,
    /// Leave the folder as it is. The same message is in it already.
    ItIsAlreadyHere,
    /// There was no message in this stretch of the file. Say so at the end and
    /// carry on with the next one.
    ItCouldNotBeRead,
}

impl WhatToDoWithIt {
    /// What to do with one answer from the reader.
    ///
    /// One stretch of a file that holds no message is ordinary in an archive
    /// somebody has kept for years, and it is a fact about that stretch and
    /// about nothing else. Treating it as a failure of the import would lose
    /// every message filed after it, which is the whole reason the reader
    /// answers once per message rather than once per file.
    pub fn for_one_read(
        read: &Result<message_files::MessageFromAFile>,
        already_here: &std::collections::HashSet<String>,
    ) -> Self {
        match read {
            Err(_) => Self::ItCouldNotBeRead,
            Ok(read) => what_to_do_with(read.message.message_id.as_deref(), already_here),
        }
    }
}

/// Whether a message read out of a file is one this folder already holds.
///
/// By its `Message-ID` and nothing else. That is the one thing about a message
/// meant to be unique, it is minted once by the program that composed the
/// message and carried unchanged by every program that touches it afterwards,
/// and it is what a re-saved draft is already recognised by here. Comparing
/// senders, subjects and dates instead would take two identical reminders sent
/// a week apart for one message and drop the second.
///
/// Asked against the identifiers the folder holds, read once before the import
/// starts, rather than one query for each message: a mailbox of forty thousand
/// would otherwise be forty thousand queries. Identifiers rather than messages,
/// so what is held is a few bytes each and not the mail itself.
///
/// A message with nothing to go on is brought in. Mail from before identifiers
/// were universal carries none, and this program's own database stores that as
/// an empty identifier rather than as nothing, so one such message in the
/// folder would otherwise make every one of them in the file look like that
/// same message. A duplicate is a nuisance somebody can see and delete; a
/// message quietly not imported is one they find out about when they go looking
/// for it.
pub fn what_to_do_with(
    identifier: Option<&str>,
    already_here: &std::collections::HashSet<String>,
) -> WhatToDoWithIt {
    match identifier.map(str::trim) {
        Some(identifier) if !identifier.is_empty() && already_here.contains(identifier) => {
            WhatToDoWithIt::ItIsAlreadyHere
        }
        _ => WhatToDoWithIt::BringItIn,
    }
}

/// Every message one file of mail holds, one at a time.
///
/// One at a time and never gathered into a list, which is the whole of why this
/// is a function rather than two lines at each place that imports. A mailbox
/// somebody has kept for twenty years is the largest file this program opens,
/// and a list of its messages holds all of it a second time as parsed text, on
/// top of the file itself, and a third time for a mailbox of signed mail. Taken
/// one at a time each message is filed and let go before the next is read.
///
/// A file holding one message and a file holding many are two different
/// readers, and each refuses what the other takes, so the caller says which
/// this is. [`importing`] and [`message_files::what_the_file_holds`] are what
/// answer that, from the bytes rather than from the file's name.
pub fn each_message_in(
    bytes: &[u8],
    read: ReadAs,
) -> Box<dyn Iterator<Item = Result<message_files::MessageFromAFile>> + '_> {
    match read {
        ReadAs::OneMessage => Box::new(std::iter::once(
            message_files::read_one_message_as_it_arrived(bytes),
        )),
        ReadAs::OneAtATimeFromAnArchive => Box::new(message_files::each_message_read_from(bytes)),
    }
}

// ── Writing one message into a folder ───────────────────────────────────────

/// Whether one message read out of a file reached the folder.
///
/// A different question from [`WhatToDoWithIt`], and about this computer rather
/// than about the file: a full disk and a database another program has locked
/// are the two ways a message somebody's file held perfectly well does not
/// arrive. Answered rather than logged, because trying again may well work and
/// nobody can decide to when nothing said so.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhetherItWasWrittenDown {
    /// It is in the folder.
    ItIsInTheFolder,
    /// It is not, and somebody is told how many were not.
    ItCouldNotBeSavedHere,
}

/// Write one message read out of a file into a folder on this computer.
///
/// The one place that files an imported message, so the single messages and
/// the messages inside an archive cannot come to be written down differently.
///
/// Three things go in and all three have to. The row, without which there is no
/// message. Its text, without which the message is in the list and opening it
/// asks a server that has never held it. And, where the message says it is
/// signed, the bytes it arrived as: a signature is arithmetic over exactly
/// those, so a signed message filed without them reads afterwards as though it
/// had never claimed a signature at all.
pub fn file_one_imported_message(
    cache: &crate::data::message_cache::MessageCache,
    read: &message_files::MessageFromAFile,
    folder_id: i64,
) -> WhetherItWasWrittenDown {
    // Which end of the folder's numbering to take from is asked of the folder,
    // which is the only thing that knows. This used to count up on the grounds
    // that an import writes into a folder on this computer, and that is not
    // true: the folder is whichever one somebody chose, and every folder an
    // IMAP account has except the Outbox is on the server. Counting up there
    // takes the number the server is about to issue, and the message it
    // belongs to is then written straight over the imported one, leaving the
    // imported text under somebody else's headers.
    let Ok(uid) = cache.next_uid_for_filing(folder_id) else {
        return WhetherItWasWrittenDown::ItCouldNotBeSavedHere;
    };
    let filed_at = chrono::Utc::now().to_rfc3339();
    let row = crate::application::filing::a_row_filed_here(
        &read.message,
        folder_id,
        uid,
        read.message.body_plain.as_ref().map_or(0, String::len),
        crate::application::filing::AlreadyRead::No,
        &filed_at,
    );
    let Ok(stored) = cache.file_message_here(&row) else {
        return WhetherItWasWrittenDown::ItCouldNotBeSavedHere;
    };
    // Logged rather than counted as a message that did not arrive, because the
    // row did: it is in the folder and the count is about messages that are
    // not. Counting it would send somebody to import the file again, and the
    // second import would recognise the row already there and file nothing.
    if let Err(e) = cache.save_message_body(
        stored,
        read.message.body_plain.as_deref(),
        read.message.body_html.as_deref(),
    ) {
        tracing::warn!("Could not store the text of an imported message: {e}");
    }
    // Nothing at all for ordinary mail, which is nearly all of it: the reader
    // above carries these only for a message that says it is signed, and the
    // cache asks the same question again before it writes anything.
    //
    // A failure here is logged and not fatal. It costs a verdict rather than
    // the message, and the reader says the signature could not be checked here
    // rather than saying it failed, which are opposite pieces of news.
    if let Some(raw) = &read.the_form_it_arrived_in
        && let Err(e) = cache.keep_signed_original(stored, raw)
    {
        tracing::warn!("Could not keep the form an imported signed message arrived in: {e}");
    }
    WhetherItWasWrittenDown::ItIsInTheFolder
}

// ── Saying what the import did ──────────────────────────────────────────────

/// What bringing a file of mail into a folder did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MessagesImported {
    /// Messages written into the folder.
    pub brought_in: usize,
    /// Messages the folder already held, left as they were.
    pub already_here: usize,
    /// Stretches of the file that held nothing a mail program recognises.
    ///
    /// Counted rather than passed over. One unreadable message in the middle of
    /// an archive somebody has kept for years is ordinary, and the reading goes
    /// on past it, so the only way anybody finds out is if it is said.
    pub could_not_be_read: usize,
    /// Messages read out of the file that could not be saved on this computer.
    ///
    /// Separate from the count above because it is a different thing to act on:
    /// there is nothing wrong with the file, and trying again may well work.
    pub not_written_down: usize,
    /// Whether the folder it was filed in is one the account's server also
    /// fills.
    ///
    /// Imported mail is never offered to a server, so in a folder like that it
    /// exists on this computer and nowhere else. Worth a sentence there and
    /// nowhere else: a folder that only ever lived here was never going to
    /// reach another device anyway.
    pub the_server_also_fills_this_folder: bool,
}

impl MessagesImported {
    /// Count what was decided about one message in the file.
    ///
    /// Exactly one count moves, which is what makes the total trustworthy: the
    /// three of them have to add up to the number of stretches the file held. A
    /// message that moved no count is one nobody is ever told about, and one
    /// that moved two reports a file as holding more than it did.
    ///
    /// Whether the writing then worked is a separate question and a separate
    /// count, because it is a separate thing to do about it.
    pub fn count_one(&mut self, what: WhatToDoWithIt) {
        match what {
            WhatToDoWithIt::BringItIn => self.brought_in += 1,
            WhatToDoWithIt::ItIsAlreadyHere => self.already_here += 1,
            WhatToDoWithIt::ItCouldNotBeRead => self.could_not_be_read += 1,
        }
    }

    /// Count what became of one message the import tried to write down.
    ///
    /// Nothing moves when it arrived, which is nearly always. The count and the
    /// sentence for it were both written before anything filled it in, so a
    /// message this program read perfectly well and then failed to save went
    /// missing while the closing count said everything had arrived.
    pub fn count_one_written(&mut self, whether: WhetherItWasWrittenDown) {
        if whether == WhetherItWasWrittenDown::ItCouldNotBeSavedHere {
            self.not_written_down += 1;
        }
    }
}

/// What an import did, in the words somebody hears.
///
/// The counts that are not zero are the ones worth saying. Each one that is
/// gets a whole sentence rather than another number, because each names
/// something somebody has to decide what to do about.
pub fn what_the_mail_import_did(read: &MessagesImported) -> String {
    let mut said = SummingUp::opening(match read.brought_in {
        0 => "No messages were imported".to_string(),
        1 => "Imported 1 message".to_string(),
        many => format!("Imported {many} messages"),
    });
    if read.already_here > 0 {
        // Two sentences written out rather than one built from parts. Four
        // words have to agree in number, and a sentence assembled from
        // fragments reads like one.
        said.sentence(match read.already_here {
            1 => "1 message was already in this folder and was left as it is".to_string(),
            many => {
                format!("{many} messages were already in this folder and were left as they are")
            }
        });
    }
    if read.could_not_be_read > 0 {
        // The same words the reader uses about the same fact, so somebody does
        // not meet one wording in the status line and another in the log. A
        // test holds the two together.
        said.sentence(match read.could_not_be_read {
            1 => "1 message in the file could not be read, because there was \
                  nothing in it a mail program recognises"
                .to_string(),
            many => format!(
                "{many} messages in the file could not be read, because there \
                 was nothing in them a mail program recognises"
            ),
        });
    }
    if read.not_written_down > 0 {
        said.sentence(match read.not_written_down {
            1 => "1 message was read from the file and could not be saved on this computer"
                .to_string(),
            many => format!(
                "{many} messages were read from the file and could not be saved \
                 on this computer"
            ),
        });
    }
    // Only when something really went in. Importing the same archive a second
    // time files nothing, and a sentence about where the imported mail stays
    // would be about no mail at all.
    if read.the_server_also_fills_this_folder && read.brought_in > 0 {
        said.sentence(
            "The imported mail stays on this computer, so it will not appear in \
             this folder on your other devices",
        );
    }
    said.spoken()
}

// ── Writing messages out ────────────────────────────────────────────────────

/// What to say when Export is asked for and no message is chosen.
///
/// The alternative is an empty file that another mail program then refuses to
/// open, with nothing at any point saying the selection was the problem.
pub const CHOOSE_THE_MESSAGES_TO_EXPORT: &str =
    "Choose the messages to export first. Select one message in the list, or several.";

/// What kind of file an export writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WritingOut {
    /// Nothing will be written, and this is what to say.
    Refused(&'static str),
    /// One message, in the file a mail program saves a single message in.
    OneMessage,
    /// Several messages one after another, in one archive.
    AnArchive,
}

impl WritingOut {
    /// What the file this writes should be named to end with.
    ///
    /// A name saying one thing over contents that are another is how an archive
    /// comes to be saved as a single message: the file opens, the first message
    /// is there, and everything after it is stuck on the end of that message's
    /// body where nobody looks.
    ///
    /// The two endings are the ones mail programs have used for years, so a
    /// file written here opens where somebody takes it.
    pub const fn the_file_ends_with(&self) -> Option<&'static str> {
        match self {
            Self::Refused(_) => None,
            Self::OneMessage => Some(".eml"),
            Self::AnArchive => Some(".mbox"),
        }
    }
}

/// What writing out this many messages produces.
///
/// The two files are different things rather than a preference. A file holding
/// a single message has no separator lines in it, so two messages written into
/// one read back as the first message's headers with the second stuck on the
/// end of its body. A mail program opens that without complaint, and the second
/// message is gone.
pub fn writing_out(how_many: usize) -> WritingOut {
    match how_many {
        0 => WritingOut::Refused(CHOOSE_THE_MESSAGES_TO_EXPORT),
        1 => WritingOut::OneMessage,
        _ => WritingOut::AnArchive,
    }
}

/// What writing a file of messages out did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MessagesExported {
    /// Messages written into the file.
    pub written: usize,
    /// Messages left out because their text is not stored on this computer.
    ///
    /// A folder lists every message it knows about and keeps the text of only
    /// the ones somebody has opened, so exporting a whole folder reaches
    /// messages this computer has the headers of and nothing else. Written out
    /// anyway they are headers with nothing under them, which looks like a
    /// successful export and is a file of empty messages.
    pub not_on_this_computer: usize,
}

/// What an export did, in the words somebody hears.
pub fn what_the_mail_export_did(written: &MessagesExported) -> String {
    // The opening comes from the writer rather than being written out again, so
    // one fact has one wording wherever somebody meets it.
    let mut said = SummingUp::opening(message_files::what_the_export_did(written.written));
    if written.not_on_this_computer > 0 {
        said.sentence(match written.not_on_this_computer {
            1 => "1 message was left out, because it has not been downloaded to \
                  this computer: open it once, then export again"
                .to_string(),
            many => format!(
                "{many} messages were left out, because they have not been \
                 downloaded to this computer: open each one once, then export again"
            ),
        });
    }
    said.spoken()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::types::Protocol;
    use message_files::MessagesRead;

    /// A folder somebody reads their mail in, which is nearly every folder.
    fn an_ordinary_folder(path: &str) -> FolderChosen<'_> {
        FolderChosen {
            path,
            kind: FolderType::Inbox,
        }
    }

    #[test]
    fn test_a_file_of_mail_with_no_account_chosen_says_which_choice_is_missing() {
        // A message belongs to a folder and a folder belongs to an account.
        // With no account there is nowhere to file anything, and the contacts
        // import learned what happens when that is not said: the count claims
        // things arrived and no list ever shows them.
        let asked = importing(one_message().as_bytes(), None, None);

        assert_eq!(asked, Importing::Refused(CHOOSE_AN_ACCOUNT_FIRST));
    }

    #[test]
    fn test_a_file_of_mail_with_no_folder_chosen_says_so_rather_than_guessing() {
        // Mail arriving has a folder chosen for it by the server. Mail read out
        // of a file has nobody to choose one, and guessing the inbox would put
        // twenty years of somebody's archive in among their new mail.
        let asked = importing(one_message().as_bytes(), Some("acct"), None);

        assert_eq!(asked, Importing::Refused(CHOOSE_A_FOLDER_FIRST));
    }

    #[test]
    fn test_a_file_that_holds_no_mail_is_turned_away_before_anything_is_written() {
        // An empty file and a picture picked by mistake. Both read as one empty
        // message if nothing asks first, so the import would report a message
        // brought in and file a row nobody can identify.
        //
        // The sentence is the one the reader below already says, rather than a
        // second one meaning the same thing, so a person meets the same words
        // wherever they meet the file.
        for not_mail in [
            &b""[..],
            b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR",
            b"name,address\r\nAda,ada@example.com\r\n",
        ] {
            assert_eq!(
                importing(not_mail, Some("acct"), Some(an_ordinary_folder("INBOX"))),
                Importing::Refused(message_files::NOT_A_MAIL_FILE),
                "{:?} was taken for mail",
                String::from_utf8_lossy(&not_mail[..not_mail.len().min(20)])
            );
        }
    }

    #[test]
    fn test_mail_imported_into_a_folder_a_server_fills_is_not_deleted_by_the_next_sync() {
        // The one question in this module that loses somebody's mail if it is
        // wrong. A sync lists what the server holds, compares it with what is
        // stored, and removes the rows the server does not have. A message read
        // out of a file was never on any server, so on that comparison it is a
        // message the server no longer has, and the next check for mail wipes
        // the archive somebody just imported.
        //
        // What stops it is the marker the database puts on a row this program
        // filed itself. `stored_uids` leaves marked rows out, so they are on
        // neither side of the comparison, and `forget_message` refuses them in
        // its own statement as well.
        let asked = importing(
            one_message().as_bytes(),
            Some("acct"),
            Some(an_ordinary_folder("INBOX")),
        );

        let Importing::GoAhead(lands) = asked else {
            panic!("an ordinary message into an ordinary folder was refused: {asked:?}");
        };
        assert_eq!(
            lands.written_down_as,
            WrittenDownAs::FiledHereCountingDownFromTheTop
        );
    }

    #[test]
    fn test_mail_imported_into_a_folder_on_this_computer_is_numbered_from_the_bottom() {
        // A POP account's folders are all here and no server numbers them, so
        // the top of the range has nothing to be reserved from and counting up
        // from the highest in use is right. Taking the reserved end there would
        // start every imported message at the largest number there is and count
        // down through mail somebody had filed themselves.
        //
        // The path comes from the one place that knows which folders live here,
        // rather than being spelled out again, so a folder added there is a
        // folder this answers about.
        let here = crate::application::local_folders::local_sent(Protocol::Pop3)
            .expect("an account whose mail is collected over POP keeps its folders here");

        let asked = importing(
            one_message().as_bytes(),
            Some("acct"),
            Some(an_ordinary_folder(&here)),
        );

        let Importing::GoAhead(lands) = asked else {
            panic!("an ordinary message into a folder on this computer was refused: {asked:?}");
        };
        assert_eq!(lands.written_down_as, WrittenDownAs::FiledHereCountingUp);
    }

    #[test]
    fn test_each_kind_of_file_is_read_by_the_reader_that_suits_it() {
        // Both directions. An archive read as a single message comes back as
        // the first message's headers with every later message stuck on the end
        // of its body, and it opens without complaint, so nobody finds out. A
        // single message read as an archive is a different parse again.
        let from_an_archive: Vec<String> =
            each_message_in(an_archive().as_bytes(), ReadAs::OneAtATimeFromAnArchive)
                .flatten()
                .map(|read| read.message.subject)
                .collect();
        let from_one_file: Vec<String> =
            each_message_in(one_message().as_bytes(), ReadAs::OneMessage)
                .flatten()
                .map(|read| read.message.subject)
                .collect();

        assert_eq!(from_an_archive, vec!["The first one", "The second one"]);
        assert_eq!(from_one_file, vec!["Notes on the engine"]);
    }

    #[test]
    fn test_a_file_that_is_not_mail_is_still_refused_by_the_reader_that_takes_it() {
        // The refusal has to survive being read one message at a time. A
        // picture read as a single message answers with an empty message unless
        // something says otherwise, and the import reports one message brought
        // in with nothing in it.
        let refused: Vec<String> =
            each_message_in(b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR", ReadAs::OneMessage)
                .filter_map(|read| read.err())
                .map(|why| why.to_string())
                .collect();

        assert_eq!(refused.len(), 1, "{refused:?}");
        assert!(
            refused[0].contains(message_files::NOT_A_MAIL_FILE),
            "{}",
            refused[0]
        );
    }

    #[test]
    fn test_an_archive_is_read_one_message_at_a_time_and_a_single_message_whole() {
        // A mailbox somebody has kept for twenty years is the largest file this
        // program opens. Reading it into a list of messages and then writing
        // that list down holds all of it twice, so an archive says here that it
        // is read a message at a time: each one is filed and let go.
        //
        // Both directions, because an answer that is right for archives and
        // wrong for single messages reads an ordinary saved message as the
        // first message of an archive, which is a different parse.
        let archive = importing(
            an_archive().as_bytes(),
            Some("acct"),
            Some(an_ordinary_folder("INBOX")),
        );
        let single = importing(
            one_message().as_bytes(),
            Some("acct"),
            Some(an_ordinary_folder("INBOX")),
        );

        let (Importing::GoAhead(archive), Importing::GoAhead(single)) = (&archive, &single) else {
            panic!("a file of mail was refused: {archive:?} {single:?}");
        };
        assert_eq!(archive.read, ReadAs::OneAtATimeFromAnArchive);
        assert_eq!(single.read, ReadAs::OneMessage);
    }

    #[test]
    fn test_mail_cannot_be_imported_into_the_folder_that_holds_what_is_waiting_to_be_sent() {
        // The Outbox is how somebody reaches the send queue, and what it lists
        // comes from the queue rather than from the messages filed in it. So a
        // message imported there is written down, counted, and never seen
        // again: the import says it worked and the folder stays as it was.
        let asked = importing(
            one_message().as_bytes(),
            Some("acct"),
            Some(FolderChosen {
                path: "\u{1}Local/Outbox",
                kind: FolderType::Outbox,
            }),
        );

        assert_eq!(asked, Importing::Refused(NOT_INTO_THE_OUTBOX));
    }

    /// The identifiers a folder already holds.
    fn already_holding(identifiers: &[&str]) -> std::collections::HashSet<String> {
        identifiers.iter().map(|held| held.to_string()).collect()
    }

    #[test]
    fn test_importing_the_same_archive_twice_does_not_leave_two_of_everything() {
        // Somebody imports a backup, cannot see whether it worked, and imports
        // it again. Without this every message in it is in the folder twice,
        // and for anybody reading the list by ear that is a folder they have to
        // check message by message to find out what they actually have.
        let already = already_holding(&["one@example.com"]);

        assert_eq!(
            what_to_do_with(Some("one@example.com"), &already),
            WhatToDoWithIt::ItIsAlreadyHere
        );
        assert_eq!(
            what_to_do_with(Some("two@example.com"), &already),
            WhatToDoWithIt::BringItIn
        );
    }

    #[test]
    fn test_a_message_carrying_no_identifier_is_brought_in_rather_than_taken_for_a_duplicate() {
        // Mail from before identifiers were universal carries none, and this
        // program's own database stores that as an empty identifier rather than
        // as nothing. So a folder holding one such message holds an empty
        // string, and matching on it would make every message in the file that
        // carries no identifier look like the same message: an archive of old
        // mail would import as one message and the rest would be reported as
        // already here.
        //
        // A duplicate is a nuisance somebody can see and delete. A message
        // silently not imported is one they only find out about when they go
        // looking for it, so the doubt is settled the other way.
        let already = already_holding(&["", "one@example.com"]);

        for nothing_to_go_on in [None, Some(""), Some("   ")] {
            assert_eq!(
                what_to_do_with(nothing_to_go_on, &already),
                WhatToDoWithIt::BringItIn,
                "{nothing_to_go_on:?} was taken for a message already here"
            );
        }
    }

    #[test]
    fn test_an_import_that_brought_nothing_in_and_lost_nothing_says_only_that() {
        // A file whose every message the folder already held. Nothing went
        // wrong and nothing is owed an explanation.
        assert_eq!(
            what_the_mail_import_did(&MessagesImported::default()),
            "No messages were imported"
        );
    }

    #[test]
    fn test_one_message_imported_is_said_in_the_singular() {
        // Read aloud, "Imported 1 messages" is the kind of sentence that makes
        // somebody stop trusting the ones that matter.
        assert_eq!(
            what_the_mail_import_did(&MessagesImported {
                brought_in: 1,
                ..MessagesImported::default()
            }),
            "Imported 1 message"
        );
        assert_eq!(
            what_the_mail_import_did(&MessagesImported {
                brought_in: 4,
                ..MessagesImported::default()
            }),
            "Imported 4 messages"
        );
    }

    #[test]
    fn test_messages_the_folder_already_held_are_counted_out_loud_rather_than_passed_over() {
        // Importing the same file twice otherwise says "Imported 0 messages",
        // which is what a broken import says as well. The difference between
        // "you already have all of this" and "none of this worked" is the whole
        // of what somebody needs to decide what to do next.
        let said = what_the_mail_import_did(&MessagesImported {
            brought_in: 5,
            already_here: 2,
            ..MessagesImported::default()
        });

        assert_eq!(
            said,
            "Imported 5 messages. 2 messages were already in this folder and were left as they are."
        );
    }

    #[test]
    fn test_one_message_already_here_is_said_in_the_singular() {
        // Four words have to agree in number in that sentence, which is why it
        // is written out twice rather than built from parts.
        let said = what_the_mail_import_did(&MessagesImported {
            brought_in: 2,
            already_here: 1,
            ..MessagesImported::default()
        });

        assert_eq!(
            said,
            "Imported 2 messages. 1 message was already in this folder and was left as it is."
        );
    }

    #[test]
    fn test_a_part_of_the_file_that_could_not_be_read_is_said_in_the_readers_own_words() {
        // An archive with half its messages unreadable and one with half as
        // many messages in it look exactly the same from the outside, and the
        // difference is whether somebody should go and look for the rest of
        // their mail.
        //
        // Checked against the sentence the reader below already says about the
        // same fact rather than against a copy written out here. Two modules
        // describing one thing in two different ways is how somebody comes to
        // hear one wording in the status line and another in the log, and a
        // copy of the words in this test would go stale without anything
        // noticing.
        for (how_many, opening) in [(1, "No messages were imported"), (3, "Imported 2 messages")] {
            let said = what_the_mail_import_did(&MessagesImported {
                brought_in: if how_many == 1 { 0 } else { 2 },
                could_not_be_read: how_many,
                ..MessagesImported::default()
            });

            let by_the_reader = message_files::what_the_import_did(&MessagesRead {
                messages: Vec::new(),
                could_not_be_read: how_many,
            });
            let clause = by_the_reader
                .strip_prefix("No messages were imported. ")
                .expect("the reader opens with a count and then explains");
            assert_eq!(said, format!("{opening}. {clause}"));
        }
    }

    #[test]
    fn test_a_message_that_could_not_be_saved_is_said_rather_than_left_to_the_log() {
        // A message this program read perfectly well and then failed to write
        // down: a full disk, a locked database. It is different from one that
        // could not be read, because there is nothing wrong with their file and
        // trying again may well work.
        let said = what_the_mail_import_did(&MessagesImported {
            brought_in: 2,
            not_written_down: 1,
            ..MessagesImported::default()
        });

        assert_eq!(
            said,
            "Imported 2 messages. 1 message was read from the file and could not \
             be saved on this computer."
        );
    }

    #[test]
    fn test_mail_imported_into_a_folder_the_server_fills_says_it_stays_on_this_computer() {
        // The counterpart of the sentence the contacts import says about cards
        // on their way to somebody's address book, and it points the other way:
        // nothing imported here is ever offered to the mail server. Somebody
        // who imports an old archive into their Inbox and then looks for it on
        // their phone is owed that before they go looking.
        let said = what_the_mail_import_did(&MessagesImported {
            brought_in: 40,
            the_server_also_fills_this_folder: true,
            ..MessagesImported::default()
        });

        assert_eq!(
            said,
            "Imported 40 messages. The imported mail stays on this computer, so it \
             will not appear in this folder on your other devices."
        );
    }

    #[test]
    fn test_mail_imported_into_a_folder_only_this_computer_has_says_nothing_about_devices() {
        // A POP account's folders are on this computer whatever anybody does,
        // so nothing about them was ever going to reach another device, and a
        // sentence saying so every time is what teaches people to stop
        // listening to this line.
        let said = what_the_mail_import_did(&MessagesImported {
            brought_in: 40,
            ..MessagesImported::default()
        });

        assert_eq!(said, "Imported 40 messages");
    }

    #[test]
    fn test_an_import_that_filed_nothing_says_nothing_about_where_the_mail_stays() {
        // Importing the same archive a second time files nothing, so there is
        // no imported mail for a sentence about where it stays to be about.
        let said = what_the_mail_import_did(&MessagesImported {
            already_here: 3,
            the_server_also_fills_this_folder: true,
            ..MessagesImported::default()
        });

        assert_eq!(
            said,
            "No messages were imported. 3 messages were already in this folder and \
             were left as they are."
        );
    }

    #[test]
    fn test_everything_that_can_be_said_at_once_is_still_one_run_of_sentences() {
        // The punctuation belongs to `summing_up` and this is where it is asked
        // for. Sentences pushed on to each other arrive as "computer.. 2
        // messages", which on screen is a typo and read aloud is a stutter
        // followed by a fragment: the sentence worth interrupting somebody for
        // is the one that comes out broken.
        let said = what_the_mail_import_did(&MessagesImported {
            brought_in: 5,
            already_here: 2,
            could_not_be_read: 2,
            not_written_down: 2,
            the_server_also_fills_this_folder: true,
        });

        assert_eq!(
            said,
            "Imported 5 messages. 2 messages were already in this folder and were \
             left as they are. 2 messages in the file could not be read, because \
             there was nothing in them a mail program recognises. 2 messages were \
             read from the file and could not be saved on this computer. The \
             imported mail stays on this computer, so it will not appear in this \
             folder on your other devices."
        );
    }

    #[test]
    fn test_exporting_with_nothing_chosen_says_so_rather_than_writing_an_empty_file() {
        // An empty file that another mail program then refuses to open, and
        // nothing at any point saying the selection was the problem.
        assert_eq!(
            writing_out(0),
            WritingOut::Refused(CHOOSE_THE_MESSAGES_TO_EXPORT)
        );
    }

    #[test]
    fn test_one_message_is_written_as_a_single_message_and_several_as_an_archive() {
        // The two are different files and not a preference. A file that holds
        // one message has no separator lines in it, so two messages written
        // into one read back as the first message's headers with the second
        // stuck on the end of its body: a mail program opens it without
        // complaint and the second message is gone.
        assert_eq!(writing_out(1), WritingOut::OneMessage);
        assert_eq!(writing_out(2), WritingOut::AnArchive);
        assert_eq!(writing_out(4000), WritingOut::AnArchive);
    }

    #[test]
    fn test_each_kind_of_export_names_the_ending_the_file_it_writes_should_have() {
        // A file whose name says one thing and whose contents are another is
        // the way an archive gets saved as a single message and opened as one
        // message with everything after the first stuck on its end.
        assert_eq!(writing_out(1).the_file_ends_with(), Some(".eml"));
        assert_eq!(writing_out(2).the_file_ends_with(), Some(".mbox"));
        assert_eq!(writing_out(0).the_file_ends_with(), None);
    }

    #[test]
    fn test_an_export_opens_with_the_same_count_the_writer_says() {
        // One wording for one fact. The writer below says this about messages
        // it wrote; this says it about an export somebody asked for, and the
        // two meeting in the same status line with different words is what a
        // copy of the sentence in each module leads to.
        for how_many in [0, 1, 7] {
            assert_eq!(
                what_the_mail_export_did(&MessagesExported {
                    written: how_many,
                    ..MessagesExported::default()
                }),
                message_files::what_the_export_did(how_many)
            );
        }
    }

    #[test]
    fn test_messages_whose_text_was_never_downloaded_are_said_rather_than_written_out_empty() {
        // A folder shows every message it knows about and keeps the text of
        // only the ones somebody has opened. Exported without asking, the rest
        // go into the file as headers with nothing under them, which looks like
        // a successful export and is a file of empty messages.
        let said = what_the_mail_export_did(&MessagesExported {
            written: 4,
            not_on_this_computer: 2,
        });

        assert_eq!(
            said,
            "Exported 4 messages. 2 messages were left out, because they have not \
             been downloaded to this computer: open each one once, then export again."
        );
    }

    #[test]
    fn test_one_message_left_out_of_an_export_is_said_in_the_singular() {
        let said = what_the_mail_export_did(&MessagesExported {
            written: 1,
            not_on_this_computer: 1,
        });

        assert_eq!(
            said,
            "Exported 1 message. 1 message was left out, because it has not been \
             downloaded to this computer: open it once, then export again."
        );
    }

    #[test]
    fn test_a_message_that_cannot_be_read_does_not_take_the_rest_of_the_archive_with_it() {
        // An archive somebody has been keeping for twenty years has a bad
        // stretch in it somewhere, and stopping there loses every message filed
        // after it. Run through the real reader rather than a list written out
        // here, because the thing being checked is what the two modules do
        // together: the reader hands back one answer per message and this has
        // to carry on past the ones that are not messages.
        let already = already_holding(&[]);
        let mut counted = MessagesImported::default();
        let mut brought_in = Vec::new();

        for read in message_files::each_message_read_from(
            an_archive_with_a_bad_stretch_in_the_middle().as_bytes(),
        ) {
            let what = WhatToDoWithIt::for_one_read(&read, &already);
            if let (WhatToDoWithIt::BringItIn, Ok(read)) = (what, &read) {
                brought_in.push(read.message.subject.clone());
            }
            counted.count_one(what);
        }

        assert_eq!(brought_in, vec!["The first one", "The second one"]);
        assert_eq!(
            counted,
            MessagesImported {
                brought_in: 2,
                could_not_be_read: 1,
                ..MessagesImported::default()
            }
        );
    }

    #[test]
    fn test_every_message_in_the_file_moves_exactly_one_count() {
        // What makes the counts trustworthy: they have to add up to the number
        // of messages the file held. A message that moves no count is one
        // nobody is told about, and one that moves two is a file that reports
        // more messages than it has.
        let already = already_holding(&["one@example.com"]);
        let mut counted = MessagesImported::default();

        for read in message_files::each_message_read_from(
            an_archive_with_a_bad_stretch_in_the_middle().as_bytes(),
        ) {
            counted.count_one(WhatToDoWithIt::for_one_read(&read, &already));
        }

        assert_eq!(
            counted.brought_in + counted.already_here + counted.could_not_be_read,
            3,
            "the counts do not add up to the three stretches in the file: {counted:?}"
        );
        assert_eq!(counted.already_here, 1);
    }

    /// An archive whose middle message is not a message.
    ///
    /// The separator is there and there is nothing behind it, which is what a
    /// file that was cut short or written over leaves.
    fn an_archive_with_a_bad_stretch_in_the_middle() -> &'static str {
        concat!(
            "From ada@example.com Mon Jul 20 10:00:00 2026\r\n",
            "From: Ada Lovelace <ada@example.com>\r\n",
            "Subject: The first one\r\n",
            "Message-ID: <one@example.com>\r\n",
            "\r\n",
            "One.\r\n",
            "\r\n",
            "From nobody Tue Jul 21 11:00:00 2026\r\n",
            "\r\n",
            "From charles@example.com Wed Jul 22 12:00:00 2026\r\n",
            "From: Charles Babbage <charles@example.com>\r\n",
            "Subject: The second one\r\n",
            "Message-ID: <two@example.com>\r\n",
            "\r\n",
            "Two.\r\n",
        )
    }

    #[test]
    fn test_everything_this_module_says_is_a_sentence_and_names_no_machinery() {
        // All of it is read aloud. A fragment with no stop on the end runs into
        // whatever is spoken next, and a sentence naming a mechanism tells
        // somebody about this program's insides instead of about their mail.
        //
        // Every sentence, gathered here rather than checked one at a time, so a
        // sentence added later is covered without anybody remembering to.
        let everything = [
            CHOOSE_AN_ACCOUNT_FIRST.to_string(),
            CHOOSE_A_FOLDER_FIRST.to_string(),
            NOT_INTO_THE_OUTBOX.to_string(),
            CHOOSE_THE_MESSAGES_TO_EXPORT.to_string(),
            message_files::NOT_A_MAIL_FILE.to_string(),
            what_the_mail_import_did(&MessagesImported {
                brought_in: 5,
                already_here: 2,
                could_not_be_read: 2,
                not_written_down: 2,
                the_server_also_fills_this_folder: true,
            }),
            what_the_mail_import_did(&MessagesImported {
                brought_in: 1,
                already_here: 1,
                could_not_be_read: 1,
                not_written_down: 1,
                the_server_also_fills_this_folder: true,
            }),
            what_the_mail_export_did(&MessagesExported {
                written: 4,
                not_on_this_computer: 2,
            }),
            what_the_mail_export_did(&MessagesExported {
                written: 1,
                not_on_this_computer: 1,
            }),
        ];

        for said in &everything {
            assert!(!said.trim().is_empty(), "something said nothing at all");
            assert!(said.ends_with('.'), "not a sentence: {said}");
            let lowered = said.to_lowercase();
            for machinery in [
                "imap", "pop3", "uid", "database", "cache", "sync", "parse", "header", "mbox",
                ".eml", "row",
            ] {
                assert!(
                    !lowered.contains(machinery),
                    "this names {machinery}, which is a mechanism and not what happens: {said}"
                );
            }
        }
    }

    /// One ordinary message, as a file saved from a mail program holds it.
    pub(super) fn one_message() -> &'static str {
        concat!(
            "From: Ada Lovelace <ada@example.com>\r\n",
            "To: Charles Babbage <charles@example.com>\r\n",
            "Subject: Notes on the engine\r\n",
            "Date: Mon, 20 Jul 2026 10:00:00 +0000\r\n",
            "Message-ID: <note-1@example.com>\r\n",
            "\r\n",
            "The engine weaves algebraic patterns.\r\n",
        )
    }

    /// Two messages in the format an archive of a whole mailbox uses.
    fn an_archive() -> &'static str {
        concat!(
            "From ada@example.com Mon Jul 20 10:00:00 2026\r\n",
            "From: Ada Lovelace <ada@example.com>\r\n",
            "Subject: The first one\r\n",
            "Message-ID: <one@example.com>\r\n",
            "\r\n",
            "One.\r\n",
            "\r\n",
            "From charles@example.com Tue Jul 21 11:00:00 2026\r\n",
            "From: Charles Babbage <charles@example.com>\r\n",
            "Subject: The second one\r\n",
            "Message-ID: <two@example.com>\r\n",
            "\r\n",
            "Two.\r\n",
        )
    }
}

/// Bringing a file of mail in and then opening what came in.
///
/// Separate from the tests above because these go through a real database
/// rather than values handed about. What the tests above prove is that the
/// decisions are right; what these prove is that a message read out of a file
/// really lands in a folder, and that opening it afterwards says what it says.
#[cfg(test)]
mod end_to_end {
    use super::tests::one_message;
    use super::*;
    use crate::application::checking_signatures::{SignatureCheck, for_message};
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{CachedFolder, MessageCache};

    /// An empty cache with one folder in it, the way an import finds one.
    fn a_cache() -> (TempHome<MessageCache>, i64) {
        let cache = TempHome::named("wixen_message_import_", |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache")
        });
        let folder_id = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acct".to_string(),
                name: "Imported".to_string(),
                path: "\u{1}Local/Imported".to_string(),
                folder_type: "Custom".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder");
        (cache, folder_id)
    }

    /// A folder of this account that the mail server fills and numbers.
    ///
    /// The helper above makes one that lives on this computer, and every test
    /// here used it, so the whole family of imports into a folder a server
    /// also fills had no test at all.
    fn a_folder_on_the_server(cache: &MessageCache) -> i64 {
        cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acct".to_string(),
                name: "Archive".to_string(),
                path: "Archive".to_string(),
                folder_type: "Custom".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder")
    }

    /// A message a sync has just brought down from the server.
    fn from_the_server(
        folder_id: i64,
        uid: u32,
        subject: &str,
    ) -> crate::data::message_cache::IncomingMessage {
        crate::data::message_cache::IncomingMessage {
            folder_id,
            uid,
            message_id: format!("<{uid}@example.com>"),
            subject: subject.to_string(),
            from_addr: "someone@example.com".to_string(),
            to_addr: "me@example.com".to_string(),
            cc: None,
            reply_to: None,
            date: "2026-07-26T10:00:00+00:00".to_string(),
            internal_date: None,
            size_bytes: Some(512),
            refs_header: None,
            read: false,
            starred: false,
            answered: false,
            draft: false,
            deleted: false,
            has_attachments: false,
            safety: crate::service::safety::Verdict::ordinary(),
            gmail_message_id: None,
            labels: None,
            receipt_to: None,
            list_unsubscribe: None,
            pop_uidl: None,
        }
    }

    #[test]
    fn test_a_message_imported_into_a_folder_the_server_fills_leaves_its_numbers_alone() {
        // Which end of a folder's numbering an imported message takes from is
        // worked out when the import is set up, written down as
        // `written_down_as`, and the filing never asked for it. So every
        // imported message counted up from the highest number in use, which in
        // a folder a server fills is the number that server is about to hand
        // out. The message that number really belongs to is then written on
        // top of the imported one, and the imported text stays behind under
        // somebody else's headers.
        let (cache, _on_this_computer) = a_cache();
        let archive = a_folder_on_the_server(&cache);
        for uid in 1..=10 {
            cache
                .upsert_message(&from_the_server(archive, uid, "From the server"))
                .expect("the folder fills up");
        }
        let read = message_files::read_one_message_as_it_arrived(one_message().as_bytes())
            .expect("an ordinary message");

        let written = file_one_imported_message(&cache, &read, archive);

        assert_eq!(written, WhetherItWasWrittenDown::ItIsInTheFolder);
        // The server hands out the number it was about to hand out.
        cache
            .upsert_message(&from_the_server(archive, 11, "The real eleventh"))
            .expect("the message arrives");

        let listed = cache
            .get_message_list(archive, "acct")
            .expect("the folder listing");
        assert_eq!(listed.len(), 12, "a message went missing: {listed:?}");
        assert!(
            listed
                .iter()
                .any(|row| row.subject == "Notes on the engine"),
            "the imported message was written over by the one the server sent"
        );
        assert!(
            listed.iter().any(|row| row.subject == "The real eleventh"),
            "the message the server sent never arrived"
        );
    }

    /// A file holding one message that really is signed with a certificate.
    ///
    /// Real OpenSSL output rather than something written here to look signed,
    /// because the thing being asked is whether a checker can still make the
    /// arithmetic come out over what was kept.
    fn a_signed_file() -> Vec<u8> {
        crate::service::signed_mail::for_tests::signed_beside()
    }

    /// What the reader would say about the signature of a message just filed.
    fn opening(cache: &MessageCache, row: i64) -> SignatureCheck {
        for_message(
            cache,
            row,
            "alice@example.com",
            "2026-08-28T00:00:00Z".parse().expect("a fixed moment"),
        )
    }

    /// The one row a file of one message leaves in a folder.
    fn the_row_in(cache: &MessageCache, folder_id: i64) -> i64 {
        let listed = cache
            .get_message_list(folder_id, "acct")
            .expect("the folder listing");
        assert_eq!(listed.len(), 1, "{listed:?}");
        listed[0].id
    }

    #[test]
    fn test_a_signed_message_brought_in_from_a_file_can_still_have_its_signature_checked() {
        // The gap this closes. Mail read out of a file arrives as the exact
        // bytes a signature was made over, which is the one thing a signature
        // can be checked against, and the import threw them away. The message
        // went into the folder and read as ordinary mail: not "signed, and not
        // checkable here", which is true, but "nothing said this was signed",
        // which is a different claim and a false one.
        let (cache, folder_id) = a_cache();
        let read = message_files::read_one_message_as_it_arrived(&a_signed_file())
            .expect("a signed message");

        let written = file_one_imported_message(&cache, &read, folder_id);

        assert_eq!(written, WhetherItWasWrittenDown::ItIsInTheFolder);
        let check = opening(&cache, the_row_in(&cache, folder_id));
        let SignatureCheck::Checked(report) = check else {
            panic!("an imported signed message says nothing about its signature: {check:?}");
        };
        assert_eq!(
            report.outcome,
            crate::service::signed_mail::SignatureOutcome::Matches,
            "{:?}",
            report.findings
        );
    }

    #[test]
    fn test_an_imported_signed_message_is_checked_every_time_it_is_opened() {
        // Not only the first time. The bytes are what makes that possible, and
        // an import that worked out the answer once and stored it would go on
        // saying a withdrawn certificate was sound.
        let (cache, folder_id) = a_cache();
        let read = message_files::read_one_message_as_it_arrived(&a_signed_file())
            .expect("a signed message");
        file_one_imported_message(&cache, &read, folder_id);
        let row = the_row_in(&cache, folder_id);

        assert_eq!(opening(&cache, row), opening(&cache, row));
    }

    #[test]
    fn test_an_ordinary_message_brought_in_from_a_file_still_says_nothing_about_signatures() {
        // Nearly all mail, and the bar has to stay off it. A line on every
        // imported message saying anything at all about signatures is a line
        // people learn to talk past, and then the one that matters is talked
        // past too. Nothing is stored for these either.
        let (cache, folder_id) = a_cache();
        let read = message_files::read_one_message_as_it_arrived(one_message().as_bytes())
            .expect("an ordinary message");

        file_one_imported_message(&cache, &read, folder_id);

        assert_eq!(
            opening(&cache, the_row_in(&cache, folder_id)),
            SignatureCheck::NotSigned
        );
        assert_eq!(cache.kept_signed_original_bytes().expect("the total"), 0);
    }

    #[test]
    fn test_a_message_that_could_not_be_saved_here_is_counted_rather_than_dropped() {
        // The import has a sentence for this and nothing ever said it. Writing
        // a row can fail on a full disk or a locked database, and the failure
        // was swallowed, so the file's messages went missing one at a time
        // while the closing count said everything had arrived.
        //
        // A folder that is not there is the same refusal the database gives for
        // any of those: the row names a folder it cannot be attached to.
        let (cache, _) = a_cache();
        let read = message_files::read_one_message_as_it_arrived(one_message().as_bytes())
            .expect("an ordinary message");
        let no_such_folder = 9999;

        let written = file_one_imported_message(&cache, &read, no_such_folder);

        assert_eq!(written, WhetherItWasWrittenDown::ItCouldNotBeSavedHere);
        let mut counted = MessagesImported::default();
        counted.count_one_written(written);
        assert_eq!(counted.not_written_down, 1);
    }

    #[test]
    fn test_a_message_that_arrived_is_counted_as_nothing_going_wrong() {
        // The other direction, so the test above cannot pass by everything
        // being nought.
        let (cache, folder_id) = a_cache();
        let read = message_files::read_one_message_as_it_arrived(one_message().as_bytes())
            .expect("an ordinary message");

        let mut counted = MessagesImported::default();
        counted.count_one_written(file_one_imported_message(&cache, &read, folder_id));

        assert_eq!(counted.not_written_down, 0);
    }

    #[test]
    fn test_an_imported_message_carries_its_text_and_survives_the_next_check_for_mail() {
        // Two things the import cannot get wrong, asked here because this is
        // the one place that writes the row. Without the text, opening the
        // message asks a server that has never held it. Without the marker the
        // database puts on a row this program filed itself, the next check for
        // mail sees a message the server does not have and takes it away, along
        // with the only copy of its text there was.
        let (cache, folder_id) = a_cache();
        let read = message_files::read_one_message_as_it_arrived(one_message().as_bytes())
            .expect("an ordinary message");

        file_one_imported_message(&cache, &read, folder_id);

        let row = the_row_in(&cache, folder_id);
        assert!(
            cache
                .get_message_body(row)
                .expect("the text")
                .and_then(|body| body.body_plain)
                .unwrap_or_default()
                .contains("algebraic"),
            "the message went into the folder with no text under it"
        );
        assert!(
            cache.was_filed_here(row).expect("the marker"),
            "the next check for mail would take this message away"
        );
    }
}

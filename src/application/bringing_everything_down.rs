//! What "everything" means for one account, decided from the cache and
//! nothing else.
//!
//! A download of every message of every kept folder, with its text, is the
//! longest run this program makes against somebody else's server, and every
//! decision in it is made here: which folder's next chunk of headers comes
//! next, which messages' text comes next, how big a chunk is, in what order,
//! and when there is nothing left to do. The runner (10-05) asks
//! [`what_to_do_next`], does the one thing it answers, and asks again.
//!
//! # Pure, and why that is the point
//!
//! No cache handle, no server, no window. The inputs are values the caller
//! reads out of the cache: how much of each folder is here, which messages
//! have no text here, how much text is kept. That is the whole of the
//! runner's state, and it means two things. A run that stopped anywhere, for
//! any reason, picks up where it was on the next ask, because the cache
//! already says what is missing and there is no state file to lose or to
//! get wrong. And every decision here is tested in milliseconds against
//! values, where the same decision made in `wx_app.rs` would cost fifty-seven
//! guard records to add a test to.
//!
//! # The order
//!
//! Headers before text for the whole account, because a folder somebody can
//! see the list of is worth more than the text of a folder they cannot. Among
//! folders, the one on screen first, then the inbox, then the rest in the
//! order the tree shows them. Newest first inside a folder, which
//! `mail_sync::uids_to_fetch` already does by taking the newest missing uids.
//! Text newest first too, which is the order the cache lists it in and the
//! order somebody opening mail meets it.
//!
//! # The bounds
//!
//! A chunk of headers is [`HEADERS_PER_CHUNK`]. A chunk of text is at most
//! [`TEXT_PER_CHUNK_MESSAGES`] messages or [`TEXT_PER_CHUNK_BYTES`],
//! whichever is met first, so a chunk of fifty newsletters and a chunk of
//! three messages with photos in them each ask a server for a bounded
//! amount. How much text stays on this computer is a [`TextBudget`], which
//! is a setting 10-03 turns into this value; `All` never ends a run on bytes.
//!
//! # What only a provider can settle
//!
//! Whether a provider tolerates a run of chunks, how many, and what it does
//! when it has had enough: ledger 11 and 72. Nothing here has met one. The
//! chunk sizes are decisions of 2026-09-17 and each says so on its constant.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::common::types::FolderType;
use crate::data::message_cache::bodies::MessageToFetch;

/// How many headers a chunk asks for.
///
/// `mail_sync::INITIAL_FETCH_LIMIT` by name rather than a second five hundred,
/// because the sync pages by that number and two numbers that happen to be
/// the same come apart the day one of them moves.
pub const HEADERS_PER_CHUNK: usize = super::mail_sync::INITIAL_FETCH_LIMIT;

/// How many messages' text a chunk asks for, at most.
///
/// Fifty is a decision of 2026-09-17, not a measurement: enough that a
/// mailbox of twelve thousand is two hundred and sixty asks rather than
/// twelve thousand, few enough that a chunk a provider refuses part-way
/// through has not cost much. A provider observed refusing after some other
/// number, or tolerating a larger one, would move it; nothing has observed a
/// provider doing either (ledger 11).
pub const TEXT_PER_CHUNK_MESSAGES: usize = 50;

/// How many bytes of text a chunk asks for, at most.
///
/// Sixteen mebibytes is a decision of 2026-09-17, not a measurement. It
/// bounds the chunk the other way: fifty messages with photos in them are
/// hundreds of megabytes, and a chunk that size held in memory while it is
/// parsed and stored is the one that stalls a laptop. A single message
/// larger than this still goes, on its own, because a bound that skipped it
/// would leave it missing forever. A provider observed doing something else
/// would move it; nothing has observed one.
pub const TEXT_PER_CHUNK_BYTES: u64 = 16 * 1024 * 1024;

/// How often the folder on screen is read again while the download brings
/// it.
///
/// Ten seconds, the figure the verdict on finding #36 named on 2026-10-14.
/// Until then every chunk of headers made the window read the whole open
/// folder again, which grows with the square of the folder: the verdict put
/// the tester's 12,872 messages at about 40 to 90 ms a chunk on the window's
/// thread, and a freeze somebody notices at roughly 50,000. A chunk lands
/// every few seconds, so once every ten seconds is a fraction of the reads,
/// and the list is never more than ten seconds behind what is here.
pub const HOW_OFTEN_THE_OPEN_FOLDER_IS_READ_AGAIN: Duration = Duration::from_secs(10);

/// When the folder on screen is read again while the download brings it.
///
/// A chunk landing marks its folder when it is the one on screen, and the
/// main timer asks [`ReadingTheOpenFolderAgain::the_folder_due`] about once
/// a second. The mark outlives the chunks, so the last chunk of a folder is
/// still shown, once its ten seconds have passed.
#[derive(Debug, Clone, Copy, Default)]
pub struct ReadingTheOpenFolderAgain {
    /// The folder on screen a chunk landed in since it was last read.
    marked: Option<i64>,
    /// When a chunk's arrival last had the folder read again.
    last_read_again_at: Option<Instant>,
}

impl ReadingTheOpenFolderAgain {
    /// A chunk of headers landed in `folder_id`.
    ///
    /// Marked only when it is the folder on screen: a chunk landing in a
    /// folder nobody is looking at would otherwise take the mark from the
    /// one somebody is, and that folder's last chunk would never be shown.
    pub fn a_chunk_landed(&mut self, folder_id: i64, on_screen: Option<i64>) {
        if on_screen == Some(folder_id) {
            self.marked = Some(folder_id);
        }
    }

    /// The folder to read again now, if any.
    ///
    /// Answering forgets the mark and remembers the moment, so the next
    /// answer waits for the next chunk and for the interval both.
    pub fn the_folder_due(&mut self, now: Instant) -> Option<i64> {
        let waited = self.last_read_again_at.is_none_or(|last| {
            now.saturating_duration_since(last) >= HOW_OFTEN_THE_OPEN_FOLDER_IS_READ_AGAIN
        });
        if !waited {
            return None;
        }
        let due = self.marked.take()?;
        self.last_read_again_at = Some(now);
        Some(due)
    }
}

/// How much of a folder is on this computer after a chunk landed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HowMuchIsHere {
    /// How many of the folder's messages this computer now holds.
    pub held: usize,
    /// How many the server says the folder holds.
    pub total_on_server: usize,
}

/// How much of a folder is here after a chunk, with the mail the chunk left
/// for the check not counted as missing.
///
/// The server's count takes in mail that arrived while the download ran, and
/// a chunk leaves that mail for the check, which runs the account's rules over
/// it and says it arrived (item 9 of the review of 2026-10-20). Counted as
/// missing, it kept a folder whose only gap was new mail unfinished: the next
/// chunk brought nothing, and [`stopped_coming_down`] said the server had
/// stopped sending a folder it had not.
///
/// Never below what is held: the server is asked its count before it lists
/// the folder, so a message arriving between the two is left for the check
/// without being in the count.
pub fn how_much_a_chunk_left_here(done: &super::mail_sync::FolderSync) -> HowMuchIsHere {
    HowMuchIsHere {
        held: done.held,
        total_on_server: done
            .total_on_server
            .saturating_sub(done.left_for_the_check)
            .max(done.held),
    }
}

/// Whether a folder has stopped coming down.
///
/// A chunk that brought nothing new is a server that has stopped handing
/// messages over and would otherwise be asked forever. Checked after the
/// completeness test rather than before it, because a folder whose last chunk
/// finished it also brings nothing new to the next one, and reporting that as
/// a server that stopped would turn every finished folder into a failed one.
/// The rule the whole-folder request was written with, moved here on
/// 2026-09-17 and kept when 10-05 retired that request with its module.
pub fn stopped_coming_down(held_before_the_last_chunk: Option<usize>, here: HowMuchIsHere) -> bool {
    here.held < here.total_on_server && held_before_the_last_chunk == Some(here.held)
}

/// One folder of an account, as the cache knows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderHere {
    /// The cache's row for the folder.
    pub folder_id: i64,
    /// The server's own path, which is what a fetch asks for.
    pub path: String,
    /// The folder's name as it is spoken.
    pub name: String,
    /// What the folder is for, which decides where it sits in the tree.
    pub kind: FolderType,
    /// How much of it is here.
    pub here: HowMuchIsHere,
    /// How much was here before the last chunk, if a chunk has been asked
    /// for in this run. What tells a folder that stopped coming down from
    /// one that has not been asked yet.
    pub held_before_the_last_chunk: Option<usize>,
}

/// Where one folder stands in a download of everything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhereAFolderStands {
    /// Fewer messages are here than the server says it holds, and the last
    /// chunk, if there was one, brought some of them.
    NeedsMoreHeaders,
    /// Every message the server says it holds is here.
    IsAllHere,
    /// The last chunk brought nothing new, so it is not asked again this run
    /// and is reported through [`HowTheRunEnded::ItStoppedComingDown`].
    StoppedComingDown,
}

/// Where a folder stands, from what the cache holds.
pub fn where_a_folder_stands(folder: &FolderHere) -> WhereAFolderStands {
    if folder.here.held >= folder.here.total_on_server {
        WhereAFolderStands::IsAllHere
    } else if stopped_coming_down(folder.held_before_the_last_chunk, folder.here) {
        WhereAFolderStands::StoppedComingDown
    } else {
        WhereAFolderStands::NeedsMoreHeaders
    }
}

/// Where a folder goes in the order everything comes down.
///
/// The folder on screen first, then the inbox, then the rest as the tree
/// shows them, which is [`crate::common::types::tree_position`]: what the
/// folder is for, then its name.
fn place_in_the_order(folder: &FolderHere, on_screen: Option<i64>) -> (u8, u8, String) {
    let not_on_screen = u8::from(on_screen != Some(folder.folder_id));
    let (kind, name) = crate::common::types::tree_position(folder.kind, &folder.name);
    (not_on_screen, kind, name)
}

/// The next chunk of text, bounded by count, by bytes, and by the budget.
///
/// At least one message whatever its size, because a bound that skipped a
/// large message would leave it missing forever; the budget is the one bound
/// that can refuse the first message, and then the run ends on it.
fn the_next_chunk_of_text(text: &TextStillMissing<'_>, budget: TextBudget) -> WhatToDoNext {
    let room_under_the_budget = match budget {
        TextBudget::All => u64::MAX,
        TextBudget::UpTo(bytes) => bytes.saturating_sub(text.kept_bytes),
    };
    let mut chunk = Vec::new();
    let mut bytes = 0u64;
    for message in text.messages.iter().take(TEXT_PER_CHUNK_MESSAGES) {
        let with_this_one = bytes.saturating_add(message.size_bytes);
        if with_this_one > room_under_the_budget {
            break;
        }
        if !chunk.is_empty() && with_this_one > TEXT_PER_CHUNK_BYTES {
            break;
        }
        chunk.push(message.clone());
        bytes = with_this_one;
    }
    if chunk.is_empty() {
        let would_need = text.messages.iter().fold(text.kept_bytes, |sum, message| {
            sum.saturating_add(message.size_bytes)
        });
        return WhatToDoNext::EverythingIsHere {
            why: Why::TextStoppedAtTheBudget {
                kept: text.kept_bytes,
                would_need,
            },
        };
    }
    WhatToDoNext::TheNextChunkOfText {
        messages: chunk,
        bytes,
    }
}

/// How much message text stays on this computer.
///
/// A plain value here, with no serde on it: `keeping_message_text` (10-03)
/// turns the setting into this. `All` never ends a run on bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextBudget {
    /// Every message's text, however much there is.
    All,
    /// Up to this many bytes of text, newest first.
    UpTo(u64),
}

/// The text an account is still missing, and how much it already keeps.
///
/// Both from the cache: the list is
/// `MessageCache::messages_with_no_text_here`, newest first with each row's
/// size beside its uid, and the bytes kept are what the budget is measured
/// against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextStillMissing<'a> {
    /// Every message with no text here, newest first.
    pub messages: &'a [MessageToFetch],
    /// How many bytes of message text this computer already keeps for the
    /// account.
    pub kept_bytes: u64,
}

/// The messages whose text one account's run has already asked for.
///
/// Nothing on this computer records a message whose text failed to arrive,
/// so without this the list of what has no text here hands the same message
/// back after every chunk, and one that always fails is asked for again with
/// no wait until a check forgets it, which can be six hours or never (finding
/// #11, 2026-10-14). Held by the runner for one account's run and dropped at
/// its end, so a message is asked for at most once a run, and the run carries
/// on past one that will not come. Forgotten at the end on purpose: a failure
/// that was passing, a connection closed while a folder opened, is asked
/// again by the next run, which the next check starts.
#[derive(Debug, Default)]
pub struct TextAskedThisRun(HashSet<i64>);

impl TextAskedThisRun {
    /// Remember that this chunk's text was asked for.
    pub fn remember(&mut self, chunk: &[MessageToFetch]) {
        self.0
            .extend(chunk.iter().map(|message| message.message_id));
    }

    /// What is missing that this run has not asked for yet, in the order it
    /// came.
    pub fn not_yet_asked(&self, missing: &[MessageToFetch]) -> Vec<MessageToFetch> {
        missing
            .iter()
            .filter(|message| !self.0.contains(&message.message_id))
            .cloned()
            .collect()
    }
}

/// Whether a chunk of text counts as work done for the account.
///
/// A run says what an account came to only when it did something, and a
/// chunk that fetched nothing did nothing: with each message asked once a
/// run, a run whose only text work asked again for messages that fail would
/// otherwise say the download is done at every check.
pub fn a_chunk_of_text_did_work(fetched: usize) -> bool {
    fetched > 0
}

/// Why a download of everything has nothing more to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Why {
    /// Every folder is here and so is every message's text.
    ItIsAllHere,
    /// Every folder is here, and reading is not allowed, so no text was asked
    /// for.
    ReadingIsOff,
    /// Every folder is here, and the text stopped at the budget: `kept` bytes
    /// are here and every message's text would need `would_need`.
    TextStoppedAtTheBudget { kept: u64, would_need: u64 },
}

/// The one thing a download of everything does next.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatToDoNext {
    /// Ask this folder for its next chunk of headers.
    TheNextChunkOfHeaders { folder_id: i64, path: String },
    /// Ask for the text of these messages, which come to this many bytes.
    TheNextChunkOfText {
        messages: Vec<MessageToFetch>,
        bytes: u64,
    },
    /// Nothing is missing that this run may fetch.
    EverythingIsHere { why: Why },
}

/// What a download of everything does next, from what the cache holds.
///
/// `on_screen` is the folder somebody is looking at, which goes first.
/// `reading_allowed` is the account's permission to fetch message text at
/// all; without it no text is asked for and the run says so.
pub fn what_to_do_next(
    folders: &[FolderHere],
    text: TextStillMissing<'_>,
    budget: TextBudget,
    on_screen: Option<i64>,
    reading_allowed: bool,
) -> WhatToDoNext {
    let next_folder = folders
        .iter()
        .filter(|folder| where_a_folder_stands(folder) == WhereAFolderStands::NeedsMoreHeaders)
        .min_by_key(|folder| place_in_the_order(folder, on_screen));
    if let Some(folder) = next_folder {
        return WhatToDoNext::TheNextChunkOfHeaders {
            folder_id: folder.folder_id,
            path: folder.path.clone(),
        };
    }
    if text.messages.is_empty() {
        return WhatToDoNext::EverythingIsHere {
            why: Why::ItIsAllHere,
        };
    }
    if !reading_allowed {
        return WhatToDoNext::EverythingIsHere {
            why: Why::ReadingIsOff,
        };
    }
    the_next_chunk_of_text(&text, budget)
}

/// How the download of one folder ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HowTheRunEnded {
    /// Every message the server says the folder holds is here.
    TheWholeFolderIsHere { held: usize },
    /// A chunk brought nothing new, so asking again would ask forever.
    ItStoppedComingDown { held: usize, total_on_server: usize },
    /// A chunk was refused, in words this program chose.
    AChunkFailed {
        held: usize,
        total_on_server: usize,
        because: String,
    },
}

/// What to say while a folder is coming down.
///
/// Both numbers, because they are different facts and the fraction is not the
/// useful part: five hundred of forty thousand and five hundred of six hundred
/// want different decisions from the person hearing them. The count is a noun
/// phrase at the end so no verb has to agree with it.
pub fn how_far_the_download_has_got(folder: &str, here: HowMuchIsHere) -> String {
    format!(
        "Downloading {folder}: {} of {}.",
        here.held,
        crate::service::caldav::how_many(here.total_on_server, "message")
    )
}

/// What to say when one folder's download has ended.
///
/// A count rather than a progress line that happens to be last. Somebody who
/// has been hearing "3500 of 12872" needs to be told this one is the end, and
/// a folder that stopped short is said to have stopped rather than reported
/// as downloaded: a folder that says "downloaded" and is not is a folder
/// somebody searches and gets a shorter answer from than they should.
pub fn what_the_folder_download_came_to(folder: &str, ended: &HowTheRunEnded) -> String {
    match ended {
        HowTheRunEnded::TheWholeFolderIsHere { held } => format!(
            "{folder} is downloaded: {} on this computer.",
            crate::service::caldav::how_many(*held, "message")
        ),
        HowTheRunEnded::ItStoppedComingDown {
            held,
            total_on_server,
        } => format!(
            "The mail server stopped sending {folder}. {held} of {total_on_server} are on this \
             computer. It will be asked again."
        ),
        HowTheRunEnded::AChunkFailed {
            held,
            total_on_server,
            because,
        } => format!(
            "Downloading {folder} stopped: {because} {held} of {total_on_server} are on this \
             computer."
        ),
    }
}

/// What the text pass of one run came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextDownload {
    /// Messages whose text arrived and is stored.
    pub fetched: usize,
    /// Messages that were asked for and did not arrive.
    pub could_not: usize,
    /// Set when the budget, not the folder, is what ended it.
    pub stopped_at_the_budget: Option<StoppedAtTheBudget>,
}

/// What a run that stopped at the budget leaves behind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoppedAtTheBudget {
    /// How many newer messages have their text here.
    pub kept_messages: usize,
    /// The budget somebody chose, in bytes.
    pub budget_bytes: u64,
    /// How many older messages will be fetched when they are opened.
    pub older_messages: usize,
}

/// What to say when the text pass has ended.
///
/// How many arrived and how many could not, said even when it is none,
/// because a run that gives only its successes reads as complete. When the
/// budget ended it, one more sentence says what is kept, which budget that is,
/// and what happens to the rest.
pub fn what_the_text_download_came_to(done: &TextDownload) -> String {
    let counts = format!(
        "{}, and {}.",
        super::mail_sync::arrived(done.fetched),
        super::mail_sync::did_not_arrive(done.could_not)
    );
    match done.stopped_at_the_budget {
        None => counts,
        Some(budget) => format!(
            "{counts} The text of {} newer {} is kept, which is the {} you chose; {} older {} \
             will be fetched when they are opened.",
            budget.kept_messages,
            messages(budget.kept_messages),
            crate::presentation::reader_text::human_size(budget.budget_bytes as usize),
            budget.older_messages,
            messages(budget.older_messages),
        ),
    }
}

/// The noun for a count of messages, so a clause with an adjective between
/// the number and the noun still agrees.
fn messages(count: usize) -> &'static str {
    if count == 1 { "message" } else { "messages" }
}

/// The one sentence said at the end of a run over a whole account.
///
/// Two counts, because they are two different facts: how many folders are
/// whole, and how many messages have their text here to read and to search.
pub fn what_a_whole_account_came_to(folders_done: usize, text_done: usize) -> String {
    let folders = crate::service::caldav::how_many(folders_done, "folder");
    let are = if folders_done == 1 { "is" } else { "are" };
    format!(
        "{folders} {are} downloaded, and the text of {} is on this computer.",
        crate::service::caldav::how_many(text_done, "message")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn here(held: usize, total_on_server: usize) -> HowMuchIsHere {
        HowMuchIsHere {
            held,
            total_on_server,
        }
    }

    fn a_folder(
        folder_id: i64,
        name: &str,
        kind: FolderType,
        held: usize,
        total: usize,
    ) -> FolderHere {
        FolderHere {
            folder_id,
            path: name.to_uppercase(),
            name: name.to_string(),
            kind,
            here: here(held, total),
            held_before_the_last_chunk: None,
        }
    }

    fn a_message(uid: u32, size_bytes: u64) -> MessageToFetch {
        MessageToFetch {
            message_id: i64::from(uid),
            folder_path: "INBOX".to_string(),
            uid,
            size_bytes,
        }
    }

    fn no_text_missing() -> TextStillMissing<'static> {
        TextStillMissing {
            messages: &[],
            kept_bytes: 0,
        }
    }

    const MIB: u64 = 1024 * 1024;

    #[test]
    fn test_a_folder_with_fewer_messages_here_than_on_the_server_is_asked_for_headers() {
        let folders = [a_folder(1, "Inbox", FolderType::Inbox, 500, 12_872)];

        let next = what_to_do_next(&folders, no_text_missing(), TextBudget::All, None, true);

        assert_eq!(
            next,
            WhatToDoNext::TheNextChunkOfHeaders {
                folder_id: 1,
                path: "INBOX".to_string()
            }
        );
    }

    #[test]
    fn test_headers_come_before_text_for_the_whole_account() {
        // A folder somebody can see the list of is worth more than the text
        // of one they cannot, so every folder's headers come first.
        let folders = [
            a_folder(1, "Inbox", FolderType::Inbox, 12_872, 12_872),
            a_folder(2, "Receipts", FolderType::Custom, 100, 900),
        ];
        let missing = [a_message(9, 4096)];
        let text = TextStillMissing {
            messages: &missing,
            kept_bytes: 0,
        };

        let next = what_to_do_next(&folders, text, TextBudget::All, None, true);

        assert!(
            matches!(
                next,
                WhatToDoNext::TheNextChunkOfHeaders { folder_id: 2, .. }
            ),
            "text was asked for while a folder still needed headers: {next:?}"
        );
    }

    #[test]
    fn test_the_folder_on_screen_comes_first_then_the_inbox_then_the_tree_order() {
        // Asked four times with the folders in a shuffled order, taking each
        // answered folder as done, so the whole order is read and not only
        // the first answer.
        let mut folders = vec![
            a_folder(4, "Sent", FolderType::Sent, 0, 10),
            a_folder(3, "Archive", FolderType::Archive, 0, 10),
            a_folder(1, "Inbox", FolderType::Inbox, 0, 10),
            a_folder(2, "Receipts", FolderType::Custom, 0, 10),
        ];
        let mut order = Vec::new();
        for _ in 0..4 {
            let WhatToDoNext::TheNextChunkOfHeaders { folder_id, .. } =
                what_to_do_next(&folders, no_text_missing(), TextBudget::All, Some(2), true)
            else {
                panic!("a folder still needing headers was not asked for");
            };
            order.push(folder_id);
            let done = folders
                .iter_mut()
                .find(|folder| folder.folder_id == folder_id)
                .expect("the folder it named");
            done.here = here(10, 10);
        }

        assert_eq!(
            order,
            vec![2, 1, 4, 3],
            "the order is not on screen, inbox, then the tree's order"
        );
    }

    #[test]
    fn test_the_rest_follow_the_tree_order_and_custom_folders_go_by_name() {
        let mut folders = vec![
            a_folder(5, "Travel", FolderType::Custom, 0, 10),
            a_folder(4, "Drafts", FolderType::Drafts, 0, 10),
            a_folder(3, "Receipts", FolderType::Custom, 0, 10),
            a_folder(2, "Trash", FolderType::Trash, 0, 10),
        ];
        let mut order = Vec::new();
        for _ in 0..4 {
            let WhatToDoNext::TheNextChunkOfHeaders { folder_id, .. } =
                what_to_do_next(&folders, no_text_missing(), TextBudget::All, None, true)
            else {
                panic!("a folder still needing headers was not asked for");
            };
            order.push(folder_id);
            let done = folders
                .iter_mut()
                .find(|folder| folder.folder_id == folder_id)
                .expect("the folder it named");
            done.here = here(10, 10);
        }

        assert_eq!(order, vec![4, 2, 3, 5]);
    }

    #[test]
    fn test_a_folder_that_is_all_here_is_never_asked_for_headers_again() {
        let folders = [a_folder(1, "Inbox", FolderType::Inbox, 12_872, 12_872)];

        let next = what_to_do_next(&folders, no_text_missing(), TextBudget::All, None, true);

        assert_eq!(
            next,
            WhatToDoNext::EverythingIsHere {
                why: Why::ItIsAllHere
            }
        );
        assert_eq!(
            where_a_folder_stands(&folders[0]),
            WhereAFolderStands::IsAllHere
        );
    }

    #[test]
    fn test_a_folder_whose_last_chunk_brought_nothing_new_is_asked_once_more_and_then_reported() {
        // The rule the whole-folder request was written with: a chunk that
        // brings nothing new is a server that has stopped handing messages
        // over, and asking again would ask forever. A folder that has not
        // been asked in this run is asked; one whose last chunk left the
        // count where it was is reported and left alone.
        let not_yet_asked = a_folder(1, "Inbox", FolderType::Inbox, 500, 40_000);
        let mut asked_once = not_yet_asked.clone();
        asked_once.held_before_the_last_chunk = Some(500);

        assert_eq!(
            where_a_folder_stands(&not_yet_asked),
            WhereAFolderStands::NeedsMoreHeaders
        );
        assert_eq!(
            where_a_folder_stands(&asked_once),
            WhereAFolderStands::StoppedComingDown
        );
        assert_eq!(
            what_to_do_next(
                std::slice::from_ref(&asked_once),
                no_text_missing(),
                TextBudget::All,
                None,
                true
            ),
            WhatToDoNext::EverythingIsHere {
                why: Why::ItIsAllHere
            },
            "a folder that stopped coming down was asked again"
        );
    }

    #[test]
    fn test_a_finished_folder_brings_nothing_new_and_is_not_a_server_that_stopped() {
        // Checked after the completeness test, because the chunk that
        // finishes a folder also brings nothing new to the one after it.
        assert!(!stopped_coming_down(Some(10), here(10, 10)));
        assert!(stopped_coming_down(Some(500), here(500, 40_000)));
        assert!(!stopped_coming_down(Some(500), here(1000, 40_000)));
        assert!(!stopped_coming_down(None, here(500, 40_000)));
    }

    /// The folder after a chunk that left it holding `held` of the server's
    /// `total`, with `left` of them left for the check, `before` held before
    /// the chunk.
    fn after_a_chunk(held: usize, total: usize, left: usize, before: usize) -> FolderHere {
        FolderHere {
            here: how_much_a_chunk_left_here(&crate::application::mail_sync::FolderSync {
                held,
                total_on_server: total,
                left_for_the_check: left,
                ..Default::default()
            }),
            held_before_the_last_chunk: Some(before),
            ..a_folder(1, "Inbox", FolderType::Inbox, 0, 0)
        }
    }

    #[test]
    fn test_a_folder_whose_only_gap_was_left_for_the_check_is_all_here() {
        // Item 9 of the review of 2026-10-20: the two that arrived while the
        // download ran are the check's, so the five here are the folder.
        assert_eq!(
            where_a_folder_stands(&after_a_chunk(5, 7, 2, 3)),
            WhereAFolderStands::IsAllHere
        );
    }

    #[test]
    fn test_a_chunk_that_left_new_mail_for_the_check_is_not_a_server_that_stopped() {
        // The chunk after the one that finished the folder brings nothing,
        // which was read as the server no longer sending it, and the person
        // heard that the mail server stopped sending the Inbox.
        let asked_again = after_a_chunk(5, 7, 2, 5);
        assert_ne!(
            where_a_folder_stands(&asked_again),
            WhereAFolderStands::StoppedComingDown
        );
        assert_eq!(
            where_a_folder_stands(&asked_again),
            WhereAFolderStands::IsAllHere
        );
    }

    #[test]
    fn test_a_folder_still_missing_older_mail_needs_more_headers() {
        assert_eq!(
            where_a_folder_stands(&after_a_chunk(3, 7, 2, 1)),
            WhereAFolderStands::NeedsMoreHeaders
        );
    }

    #[test]
    fn test_what_was_left_for_the_check_never_counts_a_folder_below_what_is_here() {
        // The count is taken before the folder is listed, so a message that
        // arrives between the two is left for the check and is not in the
        // count: five here of a count of six with two left is all here, not
        // four of six.
        let folder = after_a_chunk(5, 6, 2, 4);
        assert_eq!(folder.here, here(5, 5));
        assert_eq!(
            where_a_folder_stands(&folder),
            WhereAFolderStands::IsAllHere
        );
    }

    #[test]
    fn test_text_is_asked_for_newest_first_once_every_folder_is_here() {
        let folders = [a_folder(1, "Inbox", FolderType::Inbox, 3, 3)];
        let missing = [a_message(3, 100), a_message(2, 100), a_message(1, 100)];
        let text = TextStillMissing {
            messages: &missing,
            kept_bytes: 0,
        };

        let next = what_to_do_next(&folders, text, TextBudget::All, None, true);

        assert_eq!(
            next,
            WhatToDoNext::TheNextChunkOfText {
                messages: missing.to_vec(),
                bytes: 300
            }
        );
    }

    #[test]
    fn test_a_chunk_of_text_is_at_most_fifty_messages() {
        let folders = [a_folder(1, "Inbox", FolderType::Inbox, 3, 3)];
        let missing: Vec<MessageToFetch> = (1..=120).rev().map(|uid| a_message(uid, 10)).collect();
        let text = TextStillMissing {
            messages: &missing,
            kept_bytes: 0,
        };

        let WhatToDoNext::TheNextChunkOfText { messages, bytes } =
            what_to_do_next(&folders, text, TextBudget::All, None, true)
        else {
            panic!("no chunk of text was asked for");
        };

        assert_eq!(messages.len(), TEXT_PER_CHUNK_MESSAGES);
        assert_eq!(
            messages.first().map(|m| m.uid),
            Some(120),
            "not newest first"
        );
        assert_eq!(bytes, 500);
    }

    #[test]
    fn test_a_chunk_of_text_is_at_most_sixteen_mebibytes_whichever_bound_is_met_first() {
        let folders = [a_folder(1, "Inbox", FolderType::Inbox, 3, 3)];
        let missing = [
            a_message(3, 10 * MIB),
            a_message(2, 10 * MIB),
            a_message(1, MIB),
        ];
        let text = TextStillMissing {
            messages: &missing,
            kept_bytes: 0,
        };

        let next = what_to_do_next(&folders, text, TextBudget::All, None, true);

        assert_eq!(
            next,
            WhatToDoNext::TheNextChunkOfText {
                messages: vec![a_message(3, 10 * MIB)],
                bytes: 10 * MIB
            },
            "the byte bound did not end the chunk"
        );
    }

    #[test]
    fn test_one_message_larger_than_a_chunk_still_goes_on_its_own() {
        // A bound that skipped it would leave it missing forever.
        let folders = [a_folder(1, "Inbox", FolderType::Inbox, 1, 1)];
        let missing = [a_message(1, 40 * MIB)];
        let text = TextStillMissing {
            messages: &missing,
            kept_bytes: 0,
        };

        let next = what_to_do_next(&folders, text, TextBudget::All, None, true);

        assert_eq!(
            next,
            WhatToDoNext::TheNextChunkOfText {
                messages: vec![a_message(1, 40 * MIB)],
                bytes: 40 * MIB
            }
        );
    }

    #[test]
    fn test_the_budget_ends_a_text_run_and_says_what_it_would_have_needed() {
        let folders = [a_folder(1, "Inbox", FolderType::Inbox, 2, 2)];
        let missing = [a_message(2, 2 * MIB), a_message(1, 2 * MIB)];
        let text = TextStillMissing {
            messages: &missing,
            kept_bytes: 1023 * MIB,
        };

        let next = what_to_do_next(&folders, text, TextBudget::UpTo(1024 * MIB), None, true);

        assert_eq!(
            next,
            WhatToDoNext::EverythingIsHere {
                why: Why::TextStoppedAtTheBudget {
                    kept: 1023 * MIB,
                    would_need: 1027 * MIB
                }
            }
        );
    }

    #[test]
    fn test_a_chunk_of_text_shrinks_to_what_the_budget_still_allows() {
        // Kept plus the chunk stays inside the budget, so the last chunk
        // before the bound is as much as fits and not one message more.
        let folders = [a_folder(1, "Inbox", FolderType::Inbox, 3, 3)];
        let missing = [
            a_message(3, 2 * MIB),
            a_message(2, 2 * MIB),
            a_message(1, 2 * MIB),
        ];
        let text = TextStillMissing {
            messages: &missing,
            kept_bytes: 1021 * MIB,
        };

        let next = what_to_do_next(&folders, text, TextBudget::UpTo(1024 * MIB), None, true);

        assert_eq!(
            next,
            WhatToDoNext::TheNextChunkOfText {
                messages: vec![a_message(3, 2 * MIB)],
                bytes: 2 * MIB
            }
        );
    }

    #[test]
    fn test_all_never_ends_a_run_on_bytes() {
        let folders = [a_folder(1, "Inbox", FolderType::Inbox, 1, 1)];
        let missing = [a_message(1, 2 * MIB)];
        let text = TextStillMissing {
            messages: &missing,
            kept_bytes: u64::MAX / 2,
        };

        let next = what_to_do_next(&folders, text, TextBudget::All, None, true);

        assert!(
            matches!(next, WhatToDoNext::TheNextChunkOfText { .. }),
            "All ended a run on bytes: {next:?}"
        );
    }

    #[test]
    fn test_no_text_is_asked_for_when_reading_is_not_allowed_and_the_run_says_why() {
        let folders = [a_folder(1, "Inbox", FolderType::Inbox, 1, 1)];
        let missing = [a_message(1, 100)];
        let text = TextStillMissing {
            messages: &missing,
            kept_bytes: 0,
        };

        let next = what_to_do_next(&folders, text, TextBudget::All, None, false);

        assert_eq!(
            next,
            WhatToDoNext::EverythingIsHere {
                why: Why::ReadingIsOff
            }
        );
    }

    #[test]
    fn test_a_message_asked_for_in_this_run_is_not_asked_for_again() {
        let missing = [a_message(9, 100), a_message(8, 100), a_message(7, 100)];
        let mut asked = TextAskedThisRun::default();

        assert_eq!(
            asked.not_yet_asked(&missing),
            missing.to_vec(),
            "a run that has asked for nothing left something out"
        );

        asked.remember(&[a_message(9, 100), a_message(7, 100)]);

        assert_eq!(
            asked.not_yet_asked(&missing),
            vec![a_message(8, 100)],
            "a message this run already asked for was handed back to be asked again"
        );
    }

    #[test]
    fn test_a_run_with_only_failed_text_left_finds_everything_here() {
        // Two messages whose text never comes: asked once, still missing,
        // and the run is over rather than asking again with no wait.
        let folders = [a_folder(1, "Inbox", FolderType::Inbox, 2, 2)];
        let missing = [a_message(9, 100), a_message(7, 100)];
        let mut asked = TextAskedThisRun::default();
        asked.remember(&missing);
        let left = asked.not_yet_asked(&missing);

        let next = what_to_do_next(
            &folders,
            TextStillMissing {
                messages: &left,
                kept_bytes: 0,
            },
            TextBudget::All,
            None,
            true,
        );

        assert_eq!(
            next,
            WhatToDoNext::EverythingIsHere {
                why: Why::ItIsAllHere
            }
        );
    }

    #[test]
    fn test_a_chunk_that_fetched_nothing_is_not_work() {
        assert!(
            !a_chunk_of_text_did_work(0),
            "a chunk that fetched nothing counted as work, so the run would say the download is \
             done at every check"
        );
        assert!(a_chunk_of_text_did_work(1));
    }

    /// A pacing whose folder 4 has been marked as on screen.
    fn folder_4_marked() -> ReadingTheOpenFolderAgain {
        let mut reading = ReadingTheOpenFolderAgain::default();
        reading.a_chunk_landed(4, Some(4));
        reading
    }

    #[test]
    fn test_the_open_folder_is_read_again_at_most_every_ten_seconds() {
        let start = Instant::now();

        let mut never_read = folder_4_marked();
        assert_eq!(
            never_read.the_folder_due(start),
            Some(4),
            "a marked folder read again by no chunk before waited"
        );

        let mut read_four_seconds_ago = folder_4_marked();
        read_four_seconds_ago.the_folder_due(start);
        read_four_seconds_ago.a_chunk_landed(4, Some(4));
        assert_eq!(
            read_four_seconds_ago.the_folder_due(start + Duration::from_secs(4)),
            None,
            "the folder was read again four seconds after the last time"
        );
        assert_eq!(
            read_four_seconds_ago.the_folder_due(start + HOW_OFTEN_THE_OPEN_FOLDER_IS_READ_AGAIN),
            Some(4),
            "the folder was not read again once ten seconds had passed"
        );

        assert_eq!(
            ReadingTheOpenFolderAgain::default().the_folder_due(start),
            None,
            "a folder was read again with nothing marked"
        );
    }

    #[test]
    fn test_a_folder_marked_after_the_last_chunk_is_still_read_again() {
        // The last chunk lands two seconds after a re-read and nothing comes
        // after it, so it is the mark alone that has it shown, and it is
        // shown once.
        let start = Instant::now();
        let mut reading = folder_4_marked();
        assert_eq!(reading.the_folder_due(start), Some(4));

        reading.a_chunk_landed(4, Some(4));

        assert_eq!(reading.the_folder_due(start + Duration::from_secs(5)), None);
        assert_eq!(
            reading.the_folder_due(start + Duration::from_secs(11)),
            Some(4),
            "the last chunk of the folder was never shown"
        );
        assert_eq!(
            reading.the_folder_due(start + Duration::from_secs(30)),
            None,
            "the folder was read again with no chunk since"
        );
    }

    #[test]
    fn test_a_chunk_in_a_folder_not_on_screen_marks_nothing() {
        let mut reading = folder_4_marked();

        reading.a_chunk_landed(9, Some(4));

        assert_eq!(
            reading.the_folder_due(Instant::now()),
            Some(4),
            "a chunk in a folder nobody is looking at took the mark from the one on screen"
        );

        let mut nothing_on_screen = ReadingTheOpenFolderAgain::default();
        nothing_on_screen.a_chunk_landed(9, None);
        assert_eq!(nothing_on_screen.the_folder_due(Instant::now()), None);
    }

    #[test]
    fn test_the_progress_line_names_the_folder_and_gives_both_numbers() {
        let said = how_far_the_download_has_got("Inbox", here(3500, 12_872));

        assert_eq!(said, "Downloading Inbox: 3500 of 12872 messages.");
    }

    #[test]
    fn test_a_whole_folder_is_said_as_downloaded_with_its_count() {
        let said = what_the_folder_download_came_to(
            "Inbox",
            &HowTheRunEnded::TheWholeFolderIsHere { held: 12_872 },
        );

        assert_eq!(
            said,
            "Inbox is downloaded: 12872 messages on this computer."
        );
    }

    #[test]
    fn test_a_folder_that_stopped_coming_down_is_said_to_have_stopped_and_will_be_asked_again() {
        let said = what_the_folder_download_came_to(
            "Inbox",
            &HowTheRunEnded::ItStoppedComingDown {
                held: 3500,
                total_on_server: 12_872,
            },
        );

        assert_eq!(
            said,
            "The mail server stopped sending Inbox. 3500 of 12872 are on this computer. It will \
             be asked again."
        );
        assert!(
            !said.contains("downloaded"),
            "a folder that stopped short reads as finished: {said}"
        );
    }

    #[test]
    fn test_a_refused_chunk_is_said_with_its_reason_and_the_count() {
        let said = what_the_folder_download_came_to(
            "Inbox",
            &HowTheRunEnded::AChunkFailed {
                held: 3500,
                total_on_server: 12_872,
                because: "the mail server refused.".to_string(),
            },
        );

        assert_eq!(
            said,
            "Downloading Inbox stopped: the mail server refused. 3500 of 12872 are on this \
             computer."
        );
    }

    #[test]
    fn test_the_text_report_carries_both_numbers_even_when_none_failed() {
        let said = what_the_text_download_came_to(&TextDownload {
            fetched: 12,
            could_not: 0,
            stopped_at_the_budget: None,
        });

        assert_eq!(said, "The text of 12 messages arrived, and nothing failed.");
    }

    #[test]
    fn test_the_text_report_says_what_the_budget_kept_and_what_happens_to_the_rest() {
        let said = what_the_text_download_came_to(&TextDownload {
            fetched: 4000,
            could_not: 0,
            stopped_at_the_budget: Some(StoppedAtTheBudget {
                kept_messages: 4000,
                budget_bytes: 1024 * MIB,
                older_messages: 8872,
            }),
        });

        assert_eq!(
            said,
            "The text of 4000 messages arrived, and nothing failed. The text of 4000 newer \
             messages is kept, which is the 1 GB you chose; 8872 older messages will be \
             fetched when they are opened."
        );
    }

    #[test]
    fn test_the_whole_account_is_said_in_one_sentence_with_both_counts() {
        let said = what_a_whole_account_came_to(50, 12_872);

        assert_eq!(
            said,
            "50 folders are downloaded, and the text of 12872 messages is on this computer."
        );
        assert_eq!(
            what_a_whole_account_came_to(1, 1),
            "1 folder is downloaded, and the text of 1 message is on this computer."
        );
    }

    #[test]
    fn test_the_headers_chunk_is_the_syncs_own_page_by_name() {
        assert_eq!(
            HEADERS_PER_CHUNK,
            super::super::mail_sync::INITIAL_FETCH_LIMIT
        );
    }
}

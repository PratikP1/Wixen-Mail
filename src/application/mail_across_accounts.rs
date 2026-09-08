//! Copying a message from the server of one account to the server of another.
//!
//! Two accounts are two mail servers, and no IMAP command reaches across them.
//! `COPY` and `MOVE` name a mailbox on the connection they are sent down, so a
//! destination in another account cannot be reached by sending the same command
//! somewhere else: the folder path would be looked up on the wrong server, and
//! on a server that happened to have a folder of that name the message would
//! land in the wrong mailbox rather than failing.
//!
//! So a crossing is made of two conversations. The bytes are fetched from the
//! account the message is in, and appended to the account it is going to, over
//! that account's own held session carrying that account's own permission.
//!
//! # Why the order is the safeguard
//!
//! The append happens at the destination and the source is not touched at all.
//! A copy that fails leaves the message exactly where it was, which is why
//! copying ships before moving: the fetch, the append, the flags, the two
//! sessions and the permission gate can all be built and proved before anything
//! is asked to remove an original. `imap.rs`'s no-MOVE path already works this
//! way for a move inside one account, and its doc gives the same reason.
//!
//! # Why the decisions here are functions rather than branches
//!
//! Three of them, and each has a reading that compiles and is wrong. Whether a
//! destination crosses is two accounts compared, and a comparison that is
//! dropped leaves a `COPY` going to the source server naming a folder that is
//! not there. Which flags travel is a filter, and the wrong filter carries
//! `\Deleted` to a server whose next expunge acts on it. Which date the message
//! arrived is a format, and no date at all makes a five year old message read as
//! arriving today, at the top of somebody's folder.

use crate::common::Result;
use crate::data::message_cache::MessageCache;
use crate::data::message_cache::moves_in_flight::{AMoveLeftUnfinished, AMoveStarting};
use crate::service::protocols::imap::{ImapMessage, LetGo, flag};

/// Whether a chosen destination is at the same server the message is at.
///
/// Two named values rather than a `bool`, because both readings look the same
/// at the call site and only one of them opens a second session. "true means it
/// crosses" and "true means it is the same account" are the same token, and
/// nothing but a name says which one a caller wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Crossing {
    /// A folder on the same server, so the existing `COPY` does it in one
    /// command down the connection that is already open.
    TheSameAccount,
    /// A folder on another account's server, so the message has to be fetched
    /// from one and appended to the other.
    AnotherAccount,
}

/// Whether putting the message there means speaking to a second server.
///
/// The account identifiers, never the folder paths. A path is unique inside one
/// account and not across them, which is what
/// [`crate::application::destinations::FolderInAnAccount`] exists to say.
pub fn whether_it_crosses(the_message_is_in: &str, it_is_going_to: &str) -> Crossing {
    if the_message_is_in == it_is_going_to {
        Crossing::TheSameAccount
    } else {
        Crossing::AnotherAccount
    }
}

/// The flags that survive being carried to another server.
///
/// Four of the five standard ones travel: read, starred, answered and draft.
/// They mean the same thing at every provider, and they are what somebody would
/// notice missing. The names are `flag.rs`'s constants, so this file does not
/// spell any of them a second time.
///
/// `\Deleted` is dropped. A message arriving somewhere new already marked for
/// removal is a message the destination's next expunge takes, which is a copy
/// somebody asked for and then cannot find.
///
/// Every other keyword is dropped as well, and that is the less obvious half. A
/// provider invents keywords for its own features, and they mean nothing at
/// another provider. A strict server may refuse the whole `APPEND` over a single
/// keyword it does not recognise, which turns a keyword nobody would miss into a
/// copy that did not happen.
///
/// `None` when nothing survives, because `APPEND` with an empty flag list and
/// `APPEND` with no flag list are different commands and only one of them is
/// what this means.
pub fn the_flags_that_travel(on_the_message: &[String]) -> Option<String> {
    let travelling: Vec<&str> = on_the_message
        .iter()
        .filter_map(|named| {
            [flag::SEEN, flag::FLAGGED, flag::ANSWERED, flag::DRAFT]
                .into_iter()
                .find(|known| named.eq_ignore_ascii_case(known))
        })
        .collect();
    if travelling.is_empty() {
        return None;
    }
    Some(format!("({})", travelling.join(" ")))
}

/// The date the source server filed it, spelled the way `APPEND` takes one.
///
/// Without it the destination stamps the message with the moment it arrived, so
/// a five year old message filed into another account sorts to the top of that
/// account's folder and every later sort by date is wrong about it.
///
/// Formatted from a parsed date and never passed through as the text it arrived
/// as. `async-imap` puts this value straight into the command line with no
/// quoting and no checking, so carrying a server's answer across unparsed would
/// be putting a stranger's text inside a command this program sends. A date that
/// does not read comes back as `None`, which is the same as a server that said
/// nothing: worse than the right date, and better than a command nobody wrote.
pub fn when_it_arrived(rfc3339: Option<&str>) -> Option<String> {
    let parsed = chrono::DateTime::parse_from_rfc3339(rfc3339?).ok()?;
    Some(format!(
        "\"{}\"",
        parsed.format(crate::service::protocols::imap::INTERNAL_DATE_FORMAT)
    ))
}

/// The account the message is in, as a crossing needs to see it.
///
/// A trait rather than the session itself, so the two conversations can be held
/// with two loopback servers whose permission is set in the test. A session
/// built from an `Account` reads what this program may do at that account out of
/// the settings of whoever is running the suite, so an assertion about an append
/// landing would pass on one machine and fail on another. `mail_session.rs`'s
/// own test module says the same thing about itself and tests only the sign-in
/// for that reason.
///
/// Both methods are the shape [`crate::application::mail_controller`] already
/// has, so the implementation that runs in the program forwards and decides
/// nothing. Everything decided is in [`copy_it_across`], which both sides share.
pub(crate) trait TheAccountItIsIn {
    /// What the server says about these messages: their flags and their dates.
    async fn the_headers_of(&self, folder: &str, uids: &[u32]) -> Result<Vec<ImapMessage>>;
    /// One message exactly as it arrived, which is what an `APPEND` takes.
    async fn the_bytes_of(&self, folder: &str, uid: u32) -> Result<Vec<u8>>;
}

/// The account it is going to.
pub(crate) trait TheAccountItIsGoingTo {
    /// Put this message in that folder, filed as the source had it filed.
    async fn take_this_message(
        &self,
        into: &str,
        flags: Option<&str>,
        arrived: Option<&str>,
        raw: &[u8],
    ) -> Result<()>;

    /// Which messages in that folder carry this identifier.
    ///
    /// A read, and the only thing that can answer a question an `APPEND` left
    /// open. The reply to an `APPEND` says where the message landed only on a
    /// server with UIDPLUS, and a reply that never arrived says nothing at all.
    async fn which_messages_carry(&self, folder: &str, message_id: &str) -> Result<Vec<u32>>;
}

/// The account the message is leaving, as the last step of a move needs it.
///
/// Separate from [`TheAccountItIsIn`] rather than a third method on it, and the
/// separation is the safeguard rather than tidiness. [`copy_it_across`] takes
/// only the reading trait, which has no write in it at all, so no body it could
/// ever be given can send a change to the source. That is a stronger guarantee
/// than the assertion in the test that watches the source's transcript, and it
/// would be given up by widening the trait a copy is handed.
pub(crate) trait TheAccountItIsLeaving {
    /// Take the message off this account's server, and say what really
    /// happened to it.
    async fn take_it_off_the_server(&self, folder: &str, uid: u32) -> Result<LetGo>;
}

/// Whether the destination folder holds the message after all.
///
/// Three answers rather than two, because "it is not there" and "nobody could
/// find out" lead to different things being done and to different sentences.
/// Reading the second as the first is how a message gets removed from the only
/// server that still has it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhetherItLanded {
    /// A message carrying that identifier is there now and was not before.
    ItIsThere,
    /// The folder was asked and holds nothing new carrying that identifier.
    ItIsNotThere,
    /// The question could not be put, or its answer could not be trusted.
    ItCannotBeAsked(String),
}

/// What keeping a move needs from this program's own records.
///
/// Three things that travel together because none of them is any use without
/// the others: the store the bytes go in, the row that says which message they
/// are, and the account that has to be found again to finish the move.
#[derive(Clone, Copy)]
pub(crate) struct TheMoveAsThisProgramRecordsIt<'a> {
    /// Where the bytes are kept while the move is in the air, or `None` where
    /// this computer could not open that store at all.
    ///
    /// Optional because the keeping is a safeguard and not the move. Without
    /// it a crossing works exactly as it did before this existed: the append
    /// goes first, nothing is removed at the source until the destination has
    /// answered, and the source holds the message throughout. Refusing to move
    /// somebody's message because a second copy of it could not be written
    /// would be the safeguard making things worse than not having it.
    pub cache: Option<&'a MessageCache>,
    /// The message's own row in that cache.
    pub row: i64,
    /// The account the destination folder belongs to.
    ///
    /// Here rather than beside the folder path because it is no part of the
    /// conversation with either server. The append names the folder down the
    /// destination account's own connection, and the account identifier is
    /// what a later run of the program needs in order to find that connection
    /// again.
    pub to_account_id: &'a str,
}

/// Every way a move to a folder on another account can end.
///
/// Not a `Result`, for the reason [`crate::service::protocols::imap::Moved`] is
/// not one: once the message is at the destination, nothing that goes wrong
/// afterwards is a failure. It is a fact about where two copies now are, and
/// the caller decides what to do with the row in the list on the strength of
/// it. A bare failure left the list and the servers disagreeing.
///
/// The two endings that name an unanswered append are the ones this phase
/// exists for. An append whose answer never arrived is not a refusal and is not
/// a success: it is a question, and until it has been answered nothing may be
/// removed at the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MovedAcross {
    /// It is at the destination and gone from the source.
    ItArrivedAndTheSourceLetItGo,
    /// It is at the destination and still at the source, marked for removal.
    ItArrivedAndIsStillHereMarked(crate::service::protocols::imap::StillHere),
    /// It is at the destination and still at the source, not even marked,
    /// because the source would not take the mark. Both copies are real and a
    /// second try would make a third.
    ItArrivedAndTheSourceWouldNotLetGo(String),
    /// The destination answered and turned it down. Nothing was made there and
    /// nothing was touched at the source.
    TheDestinationRefusedIt(String),
    /// The destination never answered, and asking it afterwards found nothing
    /// new carrying the message's identifier. Nothing was removed.
    ItNeverArrivedSoNothingWasRemoved(String),
    /// The destination never answered and could not be asked. The message may
    /// be in one place or in two, and nothing was removed either way.
    ItIsNotKnownWhereItIs(String),
}

/// Copy one message from the account it is in to a folder on another account.
///
/// Reads, then reads, then writes, and the write is at the destination. Nothing
/// is said to the source server that changes anything: no `STORE`, no `COPY` and
/// no `EXPUNGE`. That is what makes a failure leave the message exactly where it
/// was, and it is asserted over the source server's own transcript rather than
/// read off this function.
pub(crate) async fn copy_it_across(
    the_account_it_is_in: &impl TheAccountItIsIn,
    from: &str,
    uid: u32,
    the_account_it_is_going_to: &impl TheAccountItIsGoingTo,
    into: &str,
) -> Result<()> {
    let (filed, raw) = the_message_itself(the_account_it_is_in, from, uid).await?;
    the_account_it_is_going_to
        .take_this_message(
            into,
            the_flags_that_travel(&filed.flags).as_deref(),
            when_it_arrived(filed.internal_date.as_deref()).as_deref(),
            &raw,
        )
        .await
}

/// What the source says about the message, and the message itself.
///
/// One place for the refusal, so a copy and a move cannot come to different
/// conclusions about a folder that named no message.
async fn the_message_itself(
    the_account_it_is_in: &impl TheAccountItIsIn,
    from: &str,
    uid: u32,
) -> Result<(ImapMessage, Vec<u8>)> {
    let headers = the_account_it_is_in.the_headers_of(from, &[uid]).await?;
    let Some(filed) = headers.into_iter().find(|message| message.uid == uid) else {
        // The server answered and named no message. Appending anyway would put
        // a copy at the destination unread and dated today, which is worse than
        // not copying it, because nothing afterwards says it happened that way.
        return Err(crate::common::Error::Protocol(format!(
            "The mail server said nothing about the message with UID {uid}"
        )));
    };
    let raw = the_account_it_is_in.the_bytes_of(from, uid).await?;
    Ok((filed, raw))
}

/// The identifier a destination can be asked about, if there is one.
///
/// A `Message-ID` is a header a stranger wrote, and two of the things it can be
/// are not questions. It can be absent, which real mail is: `mail_sync.rs:564`
/// keeps such a message with an empty identifier. And it can be blank. Neither
/// names a message, and a search for neither matches whatever the server
/// decides it matches, which is not a confirmation of anything.
///
/// Everything else is passed on as it arrived, quoted at the boundary by
/// [`crate::service::protocols::imap::ImapSession::uids_with_message_id`] so
/// that a value holding a space or a bracket cannot end the search key early.
/// Not narrowed further here: an identifier this program does not recognise the
/// shape of is still the identifier the message really carries, and refusing to
/// ask about it would turn an answerable question into an unanswerable one.
fn the_identifier_to_ask_about(header: Option<&str>) -> Option<&str> {
    let named = header?.trim();
    (!named.is_empty()).then_some(named)
}

/// Which failures mean the destination never answered.
///
/// The whole safeguard turns on this line, so it is one function with its
/// reason written down rather than a `matches!` inside a branch.
///
/// A server that says `NO` or `BAD` has answered, and the message did not land.
/// A connection that dropped, and a command that ran out of time, have not
/// answered at all: the message may be there and may not, and both arrive as
/// [`crate::common::Error::Network`]. A refusal by the permission gate has not
/// answered either, but nothing was sent, so it is also not ambiguous; it is a
/// [`crate::common::Error::Security`] and falls on the answered side, which is
/// right because a message that never left cannot have landed.
fn the_destination_never_answered(why: &crate::common::Error) -> bool {
    matches!(why, crate::common::Error::Network(_))
}

/// Why the destination cannot be asked, decided without speaking to it.
///
/// Two of the three ways [`whether_the_destination_has_it`] can come back
/// unanswerable are settled before any server is involved, and one place says
/// so rather than two. That matters because the second reader is the window
/// that meets an unfinished move on the next start: it has to know, before it
/// offers to finish anything, whether there is a question to put at all. Two
/// readings of the same rule would eventually offer somebody a choice that
/// cannot be carried out.
///
/// `None` means the question can be put. It says nothing about the answer.
fn why_the_question_cannot_be_put(
    message_id: Option<&str>,
    was_there_before: Option<&[u32]>,
) -> Option<String> {
    if message_id.is_none() {
        return Some(
            "the message carries no identifier of its own, so there is nothing to ask about"
                .to_string(),
        );
    }
    if was_there_before.is_none() {
        return Some(
            "the folder was not read before the message was sent, so a message there now \
             cannot be told from one that was there all along"
                .to_string(),
        );
    }
    None
}

/// Whether the destination folder holds the message after an append that was
/// never answered.
///
/// Its own function, and not inlined into the move, because `04.1-04` calls it
/// from a place where no move is in progress: after the program has been closed
/// part way through one and started again. A question reachable only from
/// inside the move is a question that plan would have to write a second time.
///
/// # Why it takes what was there before
///
/// A message carrying this identifier being in the folder is not the same fact
/// as this message having arrived. The identifier is a header a stranger wrote,
/// and the folder may already hold a message carrying it: an earlier copy of
/// this same message, or one somebody else put there. Reading either as an
/// arrival would remove the source copy of a message that never left, which is
/// the one failure this whole phase exists to prevent.
///
/// So the folder is asked before the message is sent as well as after, and only
/// something there now that was not there before is an arrival. Where nobody
/// looked beforehand the question cannot be settled at all, which is what `None`
/// means and why it is not the same as an empty list.
pub(crate) async fn whether_the_destination_has_it(
    the_account_it_is_going_to: &impl TheAccountItIsGoingTo,
    into: &str,
    message_id: Option<&str>,
    was_there_before: Option<&[u32]>,
) -> WhetherItLanded {
    if let Some(why) = why_the_question_cannot_be_put(message_id, was_there_before) {
        return WhetherItLanded::ItCannotBeAsked(why);
    }
    let (Some(message_id), Some(was_there_before)) = (message_id, was_there_before) else {
        // Unreachable: the check above answers `None` only when both are
        // present. Written as a refusal rather than an unwrap because this file
        // may not panic, and because the two readings can only ever disagree
        // here, where the disagreement is visible.
        return WhetherItLanded::ItCannotBeAsked(
            "the question could not be put together".to_string(),
        );
    };
    match the_account_it_is_going_to
        .which_messages_carry(into, message_id)
        .await
    {
        Ok(now) if now.iter().any(|uid| !was_there_before.contains(uid)) => {
            WhetherItLanded::ItIsThere
        }
        Ok(_) => WhetherItLanded::ItIsNotThere,
        Err(why) => WhetherItLanded::ItCannotBeAsked(why.to_string()),
    }
}

/// Move one message from the account it is in to a folder on another account.
///
/// Append first, remove last, and nothing at the source is touched until the
/// destination has answered that it holds the message. That order is the whole
/// safeguard: a failure anywhere before the removal leaves the message exactly
/// where it was, and a failure after it leaves two copies. Both are things
/// somebody can put right. A message removed from the only server that had it
/// is not.
///
/// An `Err` here means nothing changed at either server, which is the contract
/// [`crate::service::protocols::imap::ImapSession::move_message`] already has.
/// Everything that did change something comes back as a [`MovedAcross`] naming
/// where the message now is.
pub(crate) async fn move_it_across(
    the_account_it_is_in: &(impl TheAccountItIsIn + TheAccountItIsLeaving),
    from: &str,
    uid: u32,
    the_account_it_is_going_to: &impl TheAccountItIsGoingTo,
    into: &str,
    recording: TheMoveAsThisProgramRecordsIt<'_>,
) -> Result<MovedAcross> {
    the_crossing(
        the_account_it_is_in,
        from,
        uid,
        the_account_it_is_going_to,
        into,
        recording,
    )
    .await
}

/// The crossing itself, with the kept bytes written in the middle of it.
async fn the_crossing(
    the_account_it_is_in: &(impl TheAccountItIsIn + TheAccountItIsLeaving),
    from: &str,
    uid: u32,
    the_account_it_is_going_to: &impl TheAccountItIsGoingTo,
    into: &str,
    recording: TheMoveAsThisProgramRecordsIt<'_>,
) -> Result<MovedAcross> {
    let (filed, raw) = the_message_itself(the_account_it_is_in, from, uid).await?;
    let identifier = the_identifier_to_ask_about(filed.message_id.as_deref());

    // Asked before the message goes, because afterwards it is too late: a
    // folder holding one message with that identifier cannot say whether it is
    // the one just sent or one that was always there. A folder that cannot be
    // read now is not a failure and does not stop the move; it only means an
    // append whose answer never arrives cannot be settled, and the ending then
    // says so rather than guessing.
    let was_there_before = match identifier {
        Some(identifier) => the_account_it_is_going_to
            .which_messages_carry(into, identifier)
            .await
            .ok(),
        None => None,
    };

    let flags = the_flags_that_travel(&filed.flags);
    let arrived = when_it_arrived(filed.internal_date.as_deref());

    if let Err(why) = the_account_it_is_going_to
        .take_this_message(into, flags.as_deref(), arrived.as_deref(), &raw)
        .await
    {
        if !the_destination_never_answered(&why) {
            return Ok(MovedAcross::TheDestinationRefusedIt(why.to_string()));
        }
        match whether_the_destination_has_it(
            the_account_it_is_going_to,
            into,
            identifier,
            was_there_before.as_deref(),
        )
        .await
        {
            // It landed after all. The removal may go, and it goes for the same
            // reason it would have gone had the answer arrived.
            WhetherItLanded::ItIsThere => {}
            WhetherItLanded::ItIsNotThere => {
                return Ok(MovedAcross::ItNeverArrivedSoNothingWasRemoved(
                    why.to_string(),
                ));
            }
            WhetherItLanded::ItCannotBeAsked(unanswerable) => {
                return Ok(MovedAcross::ItIsNotKnownWhereItIs(format!(
                    "{why}, and {unanswerable}"
                )));
            }
        }
    }

    // The message is at the destination and the removal has not gone yet, so
    // this is the window worth keeping the bytes for.
    if let Some(cache) = recording.cache
        && let Err(e) = cache.keep_the_message_while_it_moves(&AMoveStarting {
            message_row_id: recording.row,
            to_account_id: recording.to_account_id,
            to_folder: into,
            flags: flags.as_deref(),
            arrived: arrived.as_deref(),
            was_there_before: was_there_before.as_deref(),
            raw: &raw,
        })
    {
        tracing::warn!("The message being moved could not be kept while it moves: {e}");
    }

    let ended = the_removal(the_account_it_is_in, from, uid).await;
    if let MovedAcross::ItArrivedAndTheSourceLetItGo = ended
        && let Some(cache) = recording.cache
        && let Err(e) = cache.the_move_is_over(recording.row)
    {
        tracing::warn!("The bytes kept for a finished move could not be let go of: {e}");
    }
    Ok(ended)
}

/// Ask the source to let the message go, and say what really happened.
///
/// One place, shared by the move and by a move finished on a later run,
/// because the two have to come to the same conclusion about the same three
/// answers. A resume with a removal of its own would be a second reading of
/// what `StillHereMarked` means.
async fn the_removal(
    the_account_it_is_leaving: &impl TheAccountItIsLeaving,
    from: &str,
    uid: u32,
) -> MovedAcross {
    match the_account_it_is_leaving
        .take_it_off_the_server(from, uid)
        .await
    {
        Ok(LetGo::ItIsGone) => MovedAcross::ItArrivedAndTheSourceLetItGo,
        Ok(LetGo::StillHereMarked(why)) => MovedAcross::ItArrivedAndIsStillHereMarked(why),
        Ok(LetGo::StillHereUnmarked(said)) => MovedAcross::ItArrivedAndTheSourceWouldNotLetGo(said),
        // The message is at the destination, so this is not a failure of the
        // move however it reads from here, and answering with `Err` would
        // tell the caller nothing had happened anywhere. Nothing was marked
        // either: everything that can go wrong before the mark goes wrong
        // before it is sent.
        Err(why) => MovedAcross::ItArrivedAndTheSourceWouldNotLetGo(why.to_string()),
    }
}

/// What to do about a move the program stopped part way through.
///
/// A decision over what the destination answered and nothing else, so it is
/// asked and tested without a window and without a server. The window meets it
/// already made, which is the rule `server_delete.rs` states for the row and
/// the sentence and it is the same reason: a decision taken inside a callback
/// can only be checked by opening a window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinishingIt {
    /// The destination has the message, so all that is left is the removal.
    TakeItOffTheSource,
    /// The destination does not have it, so the append goes again from the
    /// kept bytes, and only then is the source asked to let go.
    SendItAgainThenTakeItOff,
    /// Nothing can be decided, so nothing is sent to either server.
    NothingCanBeSent(String),
}

/// What a resumed move should do, given what the destination answered.
///
/// The whole of the safeguard is that this is asked at all. A row surviving a
/// restart means the program stopped somewhere between the write and the
/// clear, and the append may well have landed. Sending it again on the
/// strength of the row existing is how a second copy of somebody's mail is
/// made, and it is the one way this store can do harm that not having it would
/// not.
pub fn how_to_finish(landed: &WhetherItLanded) -> FinishingIt {
    match landed {
        WhetherItLanded::ItIsThere => FinishingIt::TakeItOffTheSource,
        WhetherItLanded::ItIsNotThere => FinishingIt::SendItAgainThenTakeItOff,
        WhetherItLanded::ItCannotBeAsked(why) => FinishingIt::NothingCanBeSent(why.clone()),
    }
}

/// Finish a move the program was stopped part way through.
///
/// Asks the destination first, always, and then does what [`how_to_finish`]
/// says. Nothing reaches either server before that question has been put.
///
/// Both accounts' permissions still apply and both are asked again, because
/// each command goes down its own account's held session and the answer may
/// have changed between the run that started the move and the run that
/// finishes it.
pub(crate) async fn finish_the_move(
    unfinished: &AMoveLeftUnfinished,
    the_account_it_is_going_to: &impl TheAccountItIsGoingTo,
    the_account_it_is_leaving: &impl TheAccountItIsLeaving,
) -> MovedAcross {
    match the_account_it_is_going_to
        .take_this_message(
            &unfinished.to_folder,
            unfinished.flags.as_deref(),
            unfinished.arrived.as_deref(),
            &unfinished.raw,
        )
        .await
    {
        Ok(()) => {
            the_removal(
                the_account_it_is_leaving,
                &unfinished.from_folder,
                unfinished.uid,
            )
            .await
        }
        Err(why) if the_destination_never_answered(&why) => {
            MovedAcross::ItIsNotKnownWhereItIs(why.to_string())
        }
        Err(why) => MovedAcross::TheDestinationRefusedIt(why.to_string()),
    }
}

/// What somebody is told about a move the program did not finish.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AboutAnUnfinishedMove {
    /// It can be finished if they say so. Two answers, and the words say what
    /// each of them does.
    Ask {
        /// What the window is called.
        title: String,
        /// What is said and read out.
        words: String,
    },
    /// Nothing can be done about it. Said rather than asked, and the kept bytes
    /// go, because a question whose only honest answer is "nothing" is not a
    /// question.
    JustSay(String),
}

/// The two accounts a move was between, as somebody hears them named.
///
/// `None` where the account is no longer set up on this computer, which is a
/// real case: a row can outlive the account it names. Offering to finish a move
/// to an account that has gone would be offering a command with nowhere to go.
#[derive(Debug, Clone, Copy)]
pub struct TheAccountsInvolved<'a> {
    /// What the account still holding the message is called.
    pub it_is_in: Option<&'a str>,
    /// What the account it was going to is called.
    pub it_was_going_to: Option<&'a str>,
}

/// What to say to somebody about a move that did not finish.
///
/// Written here rather than in the window for the reason above, and it decides
/// three things at once: whether there is anything to offer, what the sentence
/// says, and which of the two shapes it takes.
///
/// It is not an error and it must not read like one. Nothing was lost: the
/// message is still where it was, because a move is append then remove and the
/// removal is last. The words say that first.
pub fn what_to_say_about_an_unfinished_move(
    unfinished: &AMoveLeftUnfinished,
    accounts: TheAccountsInvolved<'_>,
) -> AboutAnUnfinishedMove {
    let subject = &unfinished.subject;
    let (Some(it_is_in), Some(it_was_going_to)) = (accounts.it_is_in, accounts.it_was_going_to)
    else {
        return AboutAnUnfinishedMove::JustSay(format!(
            "Moving {subject} did not finish, and one of the two accounts it was \
             between is no longer set up on this computer, so it cannot be \
             finished from here. Nothing was lost."
        ));
    };
    let where_it_is = format!("{} in {it_is_in}", unfinished.from_folder);
    let where_it_was_going = format!("{} in {it_was_going_to}", unfinished.to_folder);

    if let Some(why) = why_the_question_cannot_be_put(
        the_identifier_to_ask_about(Some(&unfinished.identifier)),
        unfinished.was_there_before.as_deref(),
    ) {
        return AboutAnUnfinishedMove::JustSay(format!(
            "Moving {subject} to {where_it_was_going} did not finish. The message \
             is still in {where_it_is} and nothing was lost. It cannot be \
             finished from here, because {why}. Look in {where_it_was_going} to \
             see whether a copy arrived there, and move it again if it did not."
        ));
    }

    AboutAnUnfinishedMove::Ask {
        title: "A move that did not finish".to_string(),
        words: format!(
            "Moving {subject} to {where_it_was_going} did not finish, because \
             this program closed part way through it. The message is still in \
             {where_it_is} and nothing was lost.\n\n\
             Finish the move now? {it_was_going_to} is asked first whether it \
             already has the message, and it is only sent again if it does not. \
             Answer No to leave the message where it is; you will not be asked \
             about it again."
        ),
    }
}

impl TheAccountItIsIn for crate::application::mail_controller::MailController {
    async fn the_headers_of(&self, folder: &str, uids: &[u32]) -> Result<Vec<ImapMessage>> {
        self.fetch_headers(folder, uids).await
    }

    async fn the_bytes_of(&self, folder: &str, uid: u32) -> Result<Vec<u8>> {
        self.fetch_message_body(folder, uid).await
    }
}

impl TheAccountItIsGoingTo for crate::application::mail_controller::MailController {
    async fn take_this_message(
        &self,
        into: &str,
        flags: Option<&str>,
        arrived: Option<&str>,
        raw: &[u8],
    ) -> Result<()> {
        // Sent once. The ordinary `append_message` signs in again and sends the
        // whole message a second time when the connection went, which for this
        // command is how one message becomes two.
        self.append_message_once(into, flags, arrived, raw).await
    }

    async fn which_messages_carry(&self, folder: &str, message_id: &str) -> Result<Vec<u32>> {
        self.uids_with_message_id(folder, message_id).await
    }
}

impl TheAccountItIsLeaving for crate::application::mail_controller::MailController {
    async fn take_it_off_the_server(&self, folder: &str, uid: u32) -> Result<LetGo> {
        self.take_this_one_off(folder, uid).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::answering::{Conversation, LONG_ENOUGH, Turn, conversing};
    use crate::common::temp_home::TempHome;
    use crate::service::protocols::imap::ImapSession;
    use crate::service::protocols::imap::against_a_server_that_answers::{
        a_server_that_can, a_server_that_refuses, reading_only_on, signed_in_to,
    };

    /// The UID every test here asks for.
    const THE_UID: u32 = 4;

    /// The message the source server hands over, byte for byte.
    const THE_MESSAGE: &str =
        "Subject: Lunch\r\nFrom: Ada <ada@example.com>\r\n\r\nOne o'clock?\r\n";

    /// The headers the source server answers a header fetch with.
    const THE_HEADERS: &str = "Subject: Lunch\r\nFrom: Ada <ada@example.com>\r\n\r\n";

    /// The `Message-ID` header the message carries, as it goes on the wire.
    ///
    /// A header a stranger wrote, which is the whole reason it is quoted before
    /// it becomes a search key rather than pasted into one.
    const THE_IDENTIFIER: &str = "<lunch.4@example.com>";

    /// The same identifier as this program holds it.
    ///
    /// The angle brackets are part of how the header is written and not part of
    /// the identifier, and the parser takes them off, so this is what reaches a
    /// search. It matches the header either way: `HEADER MESSAGE-ID` looks for
    /// the value inside the header's text rather than for the whole of it, which
    /// is how the draft-replacement path has always found its previous copy.
    const THE_IDENTIFIER_AS_IT_IS_HELD: &str = "lunch.4@example.com";

    /// How the server holding the message behaves.
    ///
    /// A struct rather than five positional arguments, because four of the five
    /// are strings and a test reading `("", "", "UIDPLUS", "")` says nothing
    /// about which is which.
    #[derive(Clone, Copy)]
    struct ASourceServer {
        /// The UID the one message it holds really has. Different from the one
        /// being asked about is a folder the message has been taken out of.
        uid: u32,
        flags: &'static str,
        /// Empty means the server names no date at all, which is a real answer
        /// and a different one from a date that cannot be read.
        internal_date: &'static str,
        /// What it advertises. `UIDPLUS` is what lets one message be removed on
        /// its own; without it the only expunge available takes every message
        /// in the mailbox flagged for removal, which is other people's mail.
        capabilities: &'static str,
        /// One command it turns down, matched against the whole line without
        /// case, so `"UID STORE"` refuses the mark and leaves the rest alone.
        refusing: &'static str,
        /// The `Message-ID` header the message carries, or empty for a message
        /// that arrived without one. Real mail does: `mail_sync.rs:564` keeps
        /// such a message with an empty identifier, and an empty identifier is
        /// not something a destination can be asked about.
        message_id: &'static str,
    }

    impl Default for ASourceServer {
        fn default() -> Self {
            Self {
                uid: THE_UID,
                flags: flag::SEEN,
                internal_date: "01-Aug-2026 10:00:00 +0000",
                capabilities: "UIDPLUS",
                refusing: "",
                message_id: THE_IDENTIFIER,
            }
        }
    }

    /// A mail server holding one message, filed with these flags on this date.
    ///
    /// `a_server_that_can` cannot stand in for this. It answers a `UID FETCH`
    /// with `OK` and no data at all, which reads to the client as a message
    /// that is not there, so neither the flags nor the bytes a crossing needs
    /// could come back from it.
    async fn a_server_holding_the_message(
        flags: &'static str,
        internal_date: &'static str,
    ) -> Conversation {
        a_source_server(ASourceServer {
            flags,
            internal_date,
            // What the copy tests have always had, so widening this fixture for
            // the move does not quietly give them an extension they were
            // written without.
            capabilities: "",
            ..ASourceServer::default()
        })
        .await
    }

    /// The same, for a server whose message is not the one being asked about.
    async fn a_server_naming_a_different_message() -> Conversation {
        a_source_server(ASourceServer {
            uid: THE_UID + 1,
            internal_date: "",
            capabilities: "",
            ..ASourceServer::default()
        })
        .await
    }

    async fn a_source_server(how: ASourceServer) -> Conversation {
        let ASourceServer {
            uid,
            flags,
            internal_date,
            capabilities,
            refusing,
            message_id,
        } = how;
        conversing("* OK loopback ready\r\n", move |line| {
            let tag = line.split_whitespace().next().unwrap_or("*").to_string();
            let said = line.to_uppercase();
            if !refusing.is_empty() && said.contains(&refusing.to_uppercase()) {
                return Turn::Say(format!("{tag} NO the server would not do it\r\n"));
            }
            if said.contains("CAPABILITY") {
                return Turn::Say(format!(
                    "* CAPABILITY IMAP4rev1 {capabilities}\r\n{tag} OK done\r\n"
                ));
            }
            if said.contains(" LOGIN") || said.contains(" AUTHENTICATE") {
                return Turn::Say(format!("{tag} OK signed in\r\n"));
            }
            if said.contains("SELECT") || said.contains("EXAMINE") {
                return Turn::Say(format!(
                    "* 1 EXISTS\r\n* 0 RECENT\r\n* OK [UIDVALIDITY 1] valid\r\n\
                     {tag} OK [READ-WRITE] open\r\n"
                ));
            }
            // Before the header fetch below, because both are `UID FETCH` and
            // only the order tells them apart.
            if said.contains("BODY.PEEK[]") {
                return Turn::Say(format!(
                    "* 1 FETCH (UID {THE_UID} BODY[] {{{}}}\r\n{THE_MESSAGE})\r\n{tag} OK done\r\n",
                    THE_MESSAGE.len()
                ));
            }
            if said.contains("UID FETCH") {
                let dated = if internal_date.is_empty() {
                    String::new()
                } else {
                    format!(" INTERNALDATE \"{internal_date}\"")
                };
                let headers = if message_id.is_empty() {
                    THE_HEADERS.to_string()
                } else {
                    format!("Message-ID: {message_id}\r\n{THE_HEADERS}")
                };
                return Turn::Say(format!(
                    "* 1 FETCH (UID {uid} FLAGS ({flags}) RFC822.SIZE 120{dated} \
                     BODY[HEADER.FIELDS (SUBJECT FROM MESSAGE-ID)] {{{}}}\r\n{headers})\r\n\
                     {tag} OK done\r\n",
                    headers.len()
                ));
            }
            if said.contains("UID STORE") || said.contains("UID EXPUNGE") {
                return Turn::Say(format!("{tag} OK done\r\n"));
            }
            if said.contains("LOGOUT") {
                return Turn::Say(format!("* BYE signing off\r\n{tag} OK done\r\n"));
            }
            // Refused rather than ignored, so a script that has fallen behind
            // the client fails in the moment instead of waiting out a timeout,
            // which reads as a slow machine.
            Turn::Say(format!("{tag} BAD unscripted\r\n"))
        })
        .await
    }

    /// One end of a crossing, behind the lock a shared session needs.
    ///
    /// The two traits take `&self` because the program holds each session in an
    /// `Arc`, and a session's own commands take `&mut self`. In the program the
    /// lock is inside `MailController`; here it is around the bare session.
    struct AnAccountAt(tokio::sync::Mutex<ImapSession>);

    impl TheAccountItIsIn for AnAccountAt {
        async fn the_headers_of(&self, folder: &str, uids: &[u32]) -> Result<Vec<ImapMessage>> {
            let mut session = self.0.lock().await;
            if session.selected_folder() != Some(folder) {
                session.select_folder(folder).await?;
            }
            session.fetch_headers(uids).await
        }

        async fn the_bytes_of(&self, folder: &str, uid: u32) -> Result<Vec<u8>> {
            let mut session = self.0.lock().await;
            if session.selected_folder() != Some(folder) {
                session.select_folder(folder).await?;
            }
            session.fetch_body(uid).await
        }
    }

    impl TheAccountItIsGoingTo for AnAccountAt {
        async fn take_this_message(
            &self,
            into: &str,
            flags: Option<&str>,
            arrived: Option<&str>,
            raw: &[u8],
        ) -> Result<()> {
            self.0
                .lock()
                .await
                .append_message(into, flags, arrived, raw)
                .await
        }

        async fn which_messages_carry(&self, folder: &str, message_id: &str) -> Result<Vec<u32>> {
            let mut session = self.0.lock().await;
            if session.selected_folder() != Some(folder) {
                session.select_folder(folder).await?;
            }
            session.uids_with_message_id(message_id).await
        }
    }

    impl TheAccountItIsLeaving for AnAccountAt {
        async fn take_it_off_the_server(&self, folder: &str, uid: u32) -> Result<LetGo> {
            let mut session = self.0.lock().await;
            if session.selected_folder() != Some(folder) {
                session.select_folder(folder).await?;
            }
            session.take_this_one_off(uid).await
        }
    }

    /// A source account signed in to that server, allowed to read.
    ///
    /// Reading and no more, which is what a copy across accounts needs from the
    /// account the message is in and is worth proving rather than assuming: a
    /// crossing that needed the source's write permission would be a crossing
    /// that could change the source.
    async fn the_account_it_is_in(server: &Conversation) -> AnAccountAt {
        AnAccountAt(tokio::sync::Mutex::new(reading_only_on(server).await))
    }

    /// A destination account signed in to that server, allowed to write.
    async fn the_account_it_is_going_to(server: &Conversation) -> AnAccountAt {
        AnAccountAt(tokio::sync::Mutex::new(signed_in_to(server).await))
    }

    /// A destination account this program may not change anything at.
    async fn a_destination_this_program_may_not_write_to(server: &Conversation) -> AnAccountAt {
        AnAccountAt(tokio::sync::Mutex::new(reading_only_on(server).await))
    }

    /// The account a message is moving out of, allowed to change things.
    ///
    /// A move needs the source's write permission and a copy does not, which is
    /// the difference between the two acts stated where it is used. The copy
    /// tests keep [`the_account_it_is_in`], which cannot write at all.
    async fn the_account_it_is_leaving(server: &Conversation) -> AnAccountAt {
        AnAccountAt(tokio::sync::Mutex::new(signed_in_to(server).await))
    }

    /// What the destination does with the append it is sent.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum TheAppend {
        /// It lands and the server says so.
        Lands,
        /// The server answers and turns it down.
        IsRefused,
        /// The answer never arrives. The message may be there or may not.
        IsNeverAnswered,
    }

    /// A destination that answers however the test says, without a server.
    ///
    /// Every other test here drives a real loopback server, and the three
    /// endings about an unanswered append cannot be. The only way to make a
    /// real connection stop answering is to close it, and a closed connection
    /// cannot then be asked what the folder holds. That question is the whole
    /// subject of those three tests, so what the destination says is scripted
    /// here instead and what it was asked is recorded for the assertions.
    struct ADestinationThat<'a> {
        the_append: TheAppend,
        /// What each successive search answers, oldest first. A move asks once
        /// before the append and once after, so a test gives two.
        searches: tokio::sync::Mutex<std::collections::VecDeque<Result<Vec<u32>>>>,
        /// Every command it was asked for, in order.
        asked: tokio::sync::Mutex<Vec<String>>,
        /// The source server, read at the instant the append arrives.
        ///
        /// This is the ordering witness and there is no other. Two transcripts
        /// are two separate lists with no shared clock, so the position of a
        /// line in one says nothing about whether it came before a line in the
        /// other, and an assertion comparing the two indices is arithmetic that
        /// cannot fail. Reading the source's transcript from inside the append
        /// is what really answers the question, because the answer is taken at
        /// the moment that matters instead of afterwards.
        watching: Option<&'a Conversation>,
        /// What the source had been told when the append arrived.
        the_source_had_been_told: tokio::sync::Mutex<Vec<String>>,
        /// This program's own store, read at the instant the append arrives.
        ///
        /// The same witness as the one above and for the same reason. Whether
        /// the bytes were kept before the append is a question about an
        /// instant, and asking it after the move has finished asks about a
        /// different one: by then the row has been taken away again, and a
        /// store that never held anything and a store that held it and let go
        /// look exactly alike.
        watching_the_store: Option<&'a AStore>,
        /// What the store was holding when the append arrived.
        the_store_was_holding: tokio::sync::Mutex<Vec<AMoveLeftUnfinished>>,
    }

    impl<'a> ADestinationThat<'a> {
        fn takes_the_append(the_append: TheAppend, searches: Vec<Result<Vec<u32>>>) -> Self {
            Self {
                the_append,
                searches: tokio::sync::Mutex::new(searches.into()),
                asked: tokio::sync::Mutex::new(Vec::new()),
                watching: None,
                the_source_had_been_told: tokio::sync::Mutex::new(Vec::new()),
                watching_the_store: None,
                the_store_was_holding: tokio::sync::Mutex::new(Vec::new()),
            }
        }

        fn watching_the_source(mut self, source: &'a Conversation) -> Self {
            self.watching = Some(source);
            self
        }

        fn watching_the_store(mut self, store: &'a AStore) -> Self {
            self.watching_the_store = Some(store);
            self
        }

        async fn everything_it_was_asked(&self) -> Vec<String> {
            self.asked.lock().await.clone()
        }

        async fn what_the_source_had_been_told(&self) -> Vec<String> {
            self.the_source_had_been_told.lock().await.clone()
        }

        async fn what_the_store_was_holding(&self) -> Vec<AMoveLeftUnfinished> {
            self.the_store_was_holding.lock().await.clone()
        }
    }

    impl TheAccountItIsGoingTo for ADestinationThat<'_> {
        async fn take_this_message(
            &self,
            into: &str,
            _flags: Option<&str>,
            _arrived: Option<&str>,
            _raw: &[u8],
        ) -> Result<()> {
            if let Some(source) = self.watching {
                *self.the_source_had_been_told.lock().await = source.transcript().await;
            }
            if let Some(store) = self.watching_the_store {
                *self.the_store_was_holding.lock().await = store.moves_it_is_holding();
            }
            self.asked.lock().await.push(format!("APPEND {into}"));
            match self.the_append {
                TheAppend::Lands => Ok(()),
                TheAppend::IsRefused => Err(crate::common::Error::Protocol(
                    "the server would not take it".to_string(),
                )),
                // The variant a dropped connection and a timeout both arrive
                // as, and the only one this code may read as "nobody knows".
                TheAppend::IsNeverAnswered => Err(crate::common::Error::Network(
                    "the connection to the mail server failed".to_string(),
                )),
            }
        }

        async fn which_messages_carry(&self, folder: &str, message_id: &str) -> Result<Vec<u32>> {
            self.asked
                .lock()
                .await
                .push(format!("SEARCH {folder} {message_id}"));
            self.searches
                .lock()
                .await
                .pop_front()
                .unwrap_or_else(|| Ok(Vec::new()))
        }
    }

    /// This program's own store, with the message being moved already in it.
    ///
    /// The real cache rather than a double, and that is the point. What these
    /// tests ask is whether a whole message really is on the disk at the moment
    /// the append goes out, and really is gone once the move has ended. A
    /// double answers that question about itself.
    struct AStore {
        cache: TempHome<MessageCache>,
        row: i64,
    }

    impl AStore {
        /// Every move it currently holds bytes for.
        fn moves_it_is_holding(&self) -> Vec<AMoveLeftUnfinished> {
            self.cache
                .moves_that_did_not_finish()
                .expect("the moves that did not finish")
        }

        /// How this program records the move under test.
        fn recording(&self) -> TheMoveAsThisProgramRecordsIt<'_> {
            TheMoveAsThisProgramRecordsIt {
                cache: Some(&self.cache),
                row: self.row,
                to_account_id: THE_DESTINATION_ACCOUNT,
            }
        }
    }

    /// The account the destination folder is at, as this program knows it.
    const THE_DESTINATION_ACCOUNT: &str = "acc-2";

    fn a_store() -> AStore {
        let cache = TempHome::named("wixen_move_across_", |dir| {
            let cache = MessageCache::new(dir.to_path_buf(), None).expect("a cache");
            cache
                .save_folder(&crate::data::message_cache::CachedFolder {
                    id: 0,
                    account_id: "acc-1".to_string(),
                    name: "INBOX".to_string(),
                    path: "INBOX".to_string(),
                    folder_type: "Inbox".to_string(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("a folder");
            cache
        });
        let row = cache
            .save_message(&crate::data::message_cache::CachedMessage {
                id: 0,
                uid: THE_UID,
                folder_id: 1,
                message_id: THE_IDENTIFIER_AS_IT_IS_HELD.to_string(),
                subject: "Lunch".to_string(),
                from_addr: "ada@example.com".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                date: "2026-08-28".to_string(),
                body_plain: None,
                body_html: None,
                read: false,
                starred: false,
                deleted: false,
                safety: crate::service::safety::Safety::Ordinary,
            })
            .expect("a message");
        AStore { cache, row }
    }

    /// The move as the program makes it, with a real store behind it.
    ///
    /// Every test here goes through this rather than through
    /// [`move_it_across`] directly, so no test can be written by accident
    /// against a crossing that keeps nothing.
    async fn a_move_across(
        kept_in: &AStore,
        the_account_it_is_in: &(impl TheAccountItIsIn + TheAccountItIsLeaving),
        from: &str,
        uid: u32,
        the_account_it_is_going_to: &impl TheAccountItIsGoingTo,
        into: &str,
    ) -> Result<MovedAcross> {
        move_it_across(
            the_account_it_is_in,
            from,
            uid,
            the_account_it_is_going_to,
            into,
            kept_in.recording(),
        )
        .await
    }

    /// Wait for one crossing, and fail with a sentence rather than a timeout.
    async fn waiting_for<T>(operation: impl std::future::Future<Output = T>) -> T {
        tokio::time::timeout(LONG_ENOUGH, operation)
            .await
            .expect("the crossing never finished")
    }

    /// Every line said to a server, joined so an assertion can print it.
    async fn everything_said_to(server: &Conversation) -> String {
        server.transcript().await.join("\n")
    }

    #[test]
    fn test_a_folder_in_the_same_account_and_one_in_another_are_not_the_same_answer() {
        // Asserted as a difference rather than as two answers, on purpose. A
        // function with two values is green on arrival against any stub,
        // because the stub answers something and that something satisfies a
        // test that only ever asks about one input. This one cannot pass
        // against a constant, whichever constant it is.
        let same = whether_it_crosses("account-a", "account-a");
        let other = whether_it_crosses("account-a", "account-b");

        assert_ne!(
            same, other,
            "a destination in another account gave the same answer as one in \
             the account the message is in, so nothing here consults the \
             account at all"
        );
        assert_eq!(same, Crossing::TheSameAccount);
        assert_eq!(other, Crossing::AnotherAccount);
    }

    #[test]
    fn test_the_read_state_the_star_and_the_answered_mark_travel() {
        let travelling = the_flags_that_travel(&[
            flag::SEEN.to_string(),
            flag::FLAGGED.to_string(),
            flag::ANSWERED.to_string(),
            flag::DRAFT.to_string(),
        ])
        .expect("four standard flags to travel");

        assert!(travelling.contains(flag::SEEN), "{travelling}");
        assert!(travelling.contains(flag::FLAGGED), "{travelling}");
        assert!(travelling.contains(flag::ANSWERED), "{travelling}");
        assert!(travelling.contains(flag::DRAFT), "{travelling}");
        assert!(
            travelling.starts_with('(') && travelling.ends_with(')'),
            "an APPEND flag list is parenthesised: {travelling}"
        );
    }

    #[test]
    fn test_a_message_marked_for_removal_does_not_arrive_marked_for_removal() {
        // The copy somebody asked for, taken by the destination's next expunge
        // before they ever see it.
        let travelling = the_flags_that_travel(&[
            flag::SEEN.to_string(),
            flag::DELETED.to_string(),
            "$Phishing".to_string(),
        ])
        .expect("the read state to travel");

        assert!(
            !travelling.contains(flag::DELETED),
            "a message marked for removal arrived marked for removal: {travelling}"
        );
        assert!(
            !travelling.contains("Phishing"),
            "a keyword one provider invented was sent to another: {travelling}"
        );
        assert!(travelling.contains(flag::SEEN), "{travelling}");
    }

    #[test]
    fn test_a_message_carrying_nothing_that_travels_carries_no_flag_list_at_all() {
        // Not an empty list. `APPEND folder () {n}` and `APPEND folder {n}` are
        // different commands, and a server may refuse the first.
        assert_eq!(the_flags_that_travel(&[]), None);
        assert_eq!(
            the_flags_that_travel(&[flag::DELETED.to_string(), "$Junk".to_string()]),
            None
        );
    }

    #[test]
    fn test_the_date_the_source_filed_it_is_the_date_the_append_names() {
        let named = when_it_arrived(Some("2021-03-04T09:15:00+00:00"))
            .expect("a date the source server named");

        assert!(named.contains("04-Mar-2021"), "{named}");
        assert!(named.contains("09:15:00"), "{named}");
        assert!(
            named.starts_with('"') && named.ends_with('"'),
            "an APPEND date is quoted, and this one goes into the command line \
             exactly as written: {named}"
        );
    }

    #[test]
    fn test_a_date_the_source_did_not_give_is_no_date_rather_than_a_made_up_one() {
        assert_eq!(when_it_arrived(None), None);
    }

    #[test]
    fn test_a_date_that_does_not_read_is_left_off_rather_than_passed_through() {
        // The library underneath puts this value into the command line with no
        // quoting and no checking, so anything that came from a server and was
        // not understood must not reach it.
        assert_eq!(when_it_arrived(Some("not a date")), None);
        assert_eq!(
            when_it_arrived(Some("\"} UID STORE 1 +FLAGS (\\Deleted)\r\n")),
            None
        );
    }

    #[tokio::test]
    async fn test_a_copy_across_accounts_says_nothing_to_the_source_that_changes_it() {
        // The whole safeguard in one assertion, and the source's own transcript
        // is the only thing that can see it. A crossing that quietly also
        // flagged or copied at the source would look identical from the
        // destination's side and from this function's return value.
        let source = a_server_holding_the_message(flag::SEEN, "01-Aug-2026 10:00:00 +0000").await;
        let destination = a_server_that_can("").await;

        waiting_for(copy_it_across(
            &the_account_it_is_in(&source).await,
            "INBOX",
            THE_UID,
            &the_account_it_is_going_to(&destination).await,
            "Archive",
        ))
        .await
        .expect("the copy to be made");

        let said = everything_said_to(&source).await;
        assert!(!said.to_uppercase().contains("STORE"), "{said}");
        assert!(!said.to_uppercase().contains("EXPUNGE"), "{said}");
        assert!(!said.to_uppercase().contains("COPY"), "{said}");
    }

    #[tokio::test]
    async fn test_the_destination_is_told_to_append_into_the_folder_that_was_chosen() {
        let source = a_server_holding_the_message(flag::SEEN, "01-Aug-2026 10:00:00 +0000").await;
        let destination = a_server_that_can("").await;

        waiting_for(copy_it_across(
            &the_account_it_is_in(&source).await,
            "INBOX",
            THE_UID,
            &the_account_it_is_going_to(&destination).await,
            "Archive",
        ))
        .await
        .expect("the copy to be made");

        let appends: Vec<String> = destination
            .transcript()
            .await
            .into_iter()
            .filter(|line| line.to_uppercase().contains("APPEND"))
            .collect();
        assert_eq!(appends.len(), 1, "{appends:?}");
        assert!(appends[0].contains("Archive"), "{appends:?}");
        assert!(appends[0].contains(flag::SEEN), "{appends:?}");
        assert!(appends[0].contains("01-Aug-2026"), "{appends:?}");
    }

    #[tokio::test]
    async fn test_the_message_the_destination_is_given_is_the_one_the_source_handed_over() {
        // The bytes, not a reading of them. The cache holds a parsed body and
        // an APPEND made from that would be a different message: header order,
        // folding, transfer encoding and every attachment change or vanish.
        let source = a_server_holding_the_message("", "01-Aug-2026 10:00:00 +0000").await;
        let destination = a_server_that_can("").await;

        waiting_for(copy_it_across(
            &the_account_it_is_in(&source).await,
            "INBOX",
            THE_UID,
            &the_account_it_is_going_to(&destination).await,
            "Archive",
        ))
        .await
        .expect("the copy to be made");

        let said = everything_said_to(&destination).await;
        assert!(
            said.contains("One o'clock?"),
            "the message the source handed over never reached the destination: {said}"
        );
    }

    #[tokio::test]
    async fn test_a_destination_that_refuses_the_append_leaves_the_message_where_it_was() {
        let source = a_server_holding_the_message(flag::SEEN, "01-Aug-2026 10:00:00 +0000").await;
        let destination = a_server_that_refuses("", "APPEND").await;

        let refused = waiting_for(copy_it_across(
            &the_account_it_is_in(&source).await,
            "INBOX",
            THE_UID,
            &the_account_it_is_going_to(&destination).await,
            "Archive",
        ))
        .await;

        assert!(
            refused.is_err(),
            "an append the destination refused came back as a copy that was made"
        );
        let said = everything_said_to(&source).await;
        assert!(!said.to_uppercase().contains("STORE"), "{said}");
        assert!(!said.to_uppercase().contains("EXPUNGE"), "{said}");
        assert!(!said.to_uppercase().contains("COPY"), "{said}");
    }

    #[tokio::test]
    async fn test_an_account_this_program_may_not_write_to_refuses_at_its_own_session() {
        // The refusal belongs to the destination account, so it happens at the
        // destination's session and nothing is sent to either server. A gate
        // read at the source would let a crossing into an account somebody had
        // marked read-only.
        let source = a_server_holding_the_message(flag::SEEN, "01-Aug-2026 10:00:00 +0000").await;
        let destination = a_server_that_can("").await;

        let refused = waiting_for(copy_it_across(
            &the_account_it_is_in(&source).await,
            "INBOX",
            THE_UID,
            &a_destination_this_program_may_not_write_to(&destination).await,
            "Archive",
        ))
        .await;

        let Err(why) = refused else {
            panic!("a message was appended to an account this program may not change");
        };
        assert!(
            why.to_string()
                .contains(crate::application::allowed::SETTINGS_SECTION),
            "the refusal does not name the setting that would allow it: {why}"
        );
        let said = everything_said_to(&destination).await;
        assert!(
            !said.to_uppercase().contains("APPEND"),
            "the append reached a server it was not allowed to reach: {said}"
        );
    }

    #[tokio::test]
    async fn test_a_source_that_says_nothing_about_the_message_is_not_a_copy_with_no_flags() {
        // The server answered and named a different message, which for this
        // UID is a message that is not there. Appending anyway would put a copy
        // at the destination unread and dated today, which is worse than not
        // copying it, because nothing afterwards says it happened that way.
        //
        // The fixture hands the bytes over and only withholds the headers, on
        // purpose. It used to be a server that answered neither, and against
        // that one this test passed however the code behaved: the body fetch
        // failed first, so a build with no check on the headers at all came
        // back as an error and looked right. Found by taking the red by hand
        // rather than by reading the test.
        let source = a_server_naming_a_different_message().await;
        let destination = a_server_that_can("").await;

        let refused = waiting_for(copy_it_across(
            &the_account_it_is_in(&source).await,
            "INBOX",
            THE_UID,
            &the_account_it_is_going_to(&destination).await,
            "Archive",
        ))
        .await;

        assert!(refused.is_err(), "{refused:?}");
        let said = everything_said_to(&destination).await;
        assert!(!said.to_uppercase().contains("APPEND"), "{said}");
    }

    // ── The move, which is the copy with one command on the end ─────────────

    /// Where in a transcript a line matching this first appears.
    fn first_line_holding(transcript: &[String], word: &str) -> Option<usize> {
        transcript
            .iter()
            .position(|line| line.to_uppercase().contains(word))
    }

    /// Where the source was first told to change something.
    ///
    /// `STORE` and `EXPUNGE` together, because which of the two comes first is
    /// the server's business and neither may come before the append.
    fn first_removal_line(transcript: &[String]) -> Option<usize> {
        [
            first_line_holding(transcript, "UID STORE"),
            first_line_holding(transcript, "EXPUNGE"),
        ]
        .into_iter()
        .flatten()
        .min()
    }

    #[tokio::test]
    async fn test_the_message_reaches_one_server_and_leaves_the_other() {
        // Both ends of the act, each read off the server that did it. The order
        // they happened in is the test below, which needs a witness this one
        // does not have.
        let source = a_source_server(ASourceServer::default()).await;
        let destination = a_server_that_can("UIDPLUS").await;

        let across = waiting_for(a_move_across(
            &a_store(),
            &the_account_it_is_leaving(&source).await,
            "INBOX",
            THE_UID,
            &the_account_it_is_going_to(&destination).await,
            "Archive",
        ))
        .await
        .expect("the move to be made");

        assert_eq!(across, MovedAcross::ItArrivedAndTheSourceLetItGo);

        let at_the_destination = destination.transcript().await;
        assert!(
            first_line_holding(&at_the_destination, "APPEND").is_some(),
            "nothing was appended: {at_the_destination:?}"
        );
        // The two commands as they go on the wire, spelled out. A substring
        // would pass against a removal aimed at another message, and the UID is
        // the whole of what says which message is being taken off.
        let at_the_source = source.transcript().await;
        assert!(
            source.was_told("UID STORE 4 +FLAGS (\\Deleted)").await,
            "the message was never marked for removal at the source: {at_the_source:?}"
        );
        assert!(
            source.was_told("UID EXPUNGE 4").await,
            "the message was never taken off the source: {at_the_source:?}"
        );
        assert!(
            first_line_holding(&at_the_source, "APPEND").is_none(),
            "the append was sent to the source: {at_the_source:?}"
        );
    }

    #[tokio::test]
    async fn test_nothing_is_said_to_the_source_that_changes_it_until_the_append_has_gone_out() {
        // The safeguard the whole phase exists for, asked at the one instant
        // that can answer it: what had the source been told when the
        // destination was handed the message? Anything at all in that snapshot
        // that changes the source means the removal went first, and a failure
        // between the two would then have left the message nowhere.
        //
        // Read from inside the destination rather than compared afterwards.
        // Two transcripts have no shared clock, so an assertion comparing a
        // line's position in one with a line's position in the other is
        // arithmetic that cannot fail whatever the code does.
        let source = a_source_server(ASourceServer::default()).await;
        let destination = ADestinationThat::takes_the_append(TheAppend::Lands, vec![Ok(vec![])])
            .watching_the_source(&source);

        let across = waiting_for(a_move_across(
            &a_store(),
            &the_account_it_is_leaving(&source).await,
            "INBOX",
            THE_UID,
            &destination,
            "Archive",
        ))
        .await
        .expect("the move to be made");

        assert_eq!(across, MovedAcross::ItArrivedAndTheSourceLetItGo);
        assert!(
            !destination.everything_it_was_asked().await.is_empty(),
            "the destination was never given the message, so the snapshot below \
             is of a source nothing had happened to yet"
        );

        let when_the_append_went_out = destination.what_the_source_had_been_told().await;
        assert!(
            first_removal_line(&when_the_append_went_out).is_none(),
            "the source had already been told to give the message up before the \
             destination was handed it: {when_the_append_went_out:?}"
        );
        // And it really was told afterwards, so the snapshot above is a
        // measurement of the order rather than of a move that never removed
        // anything.
        let at_the_source = source.transcript().await;
        assert!(
            first_removal_line(&at_the_source).is_some(),
            "nothing was ever removed at the source: {at_the_source:?}"
        );
    }

    #[tokio::test]
    async fn test_a_destination_that_refuses_the_append_leaves_the_source_untouched() {
        // The assertion this whole phase exists for, and it is stated by an
        // absence: no STORE and no EXPUNGE at all. An absence cannot fail
        // against a build that never gets that far, so the outcome is asserted
        // beside it: a refused append has to come back naming the refusal, not
        // as a bare failure and not as a move.
        let source = a_source_server(ASourceServer::default()).await;
        let destination = a_server_that_refuses("UIDPLUS", "APPEND").await;

        let across = waiting_for(a_move_across(
            &a_store(),
            &the_account_it_is_leaving(&source).await,
            "INBOX",
            THE_UID,
            &the_account_it_is_going_to(&destination).await,
            "Archive",
        ))
        .await
        .expect("a refused append is an ending, not a failure");

        assert!(
            matches!(across, MovedAcross::TheDestinationRefusedIt(_)),
            "{across:?}"
        );
        let said = everything_said_to(&source).await;
        assert!(!said.to_uppercase().contains("STORE"), "{said}");
        assert!(!said.to_uppercase().contains("EXPUNGE"), "{said}");
    }

    #[tokio::test]
    async fn test_a_source_that_will_not_mark_the_message_leaves_it_in_both_places() {
        // The message really is at the destination, so this is not a failure.
        // It is two copies, and the sentence has to say so, because trying
        // again would make a third and nothing anywhere removes duplicates.
        let source = a_source_server(ASourceServer {
            refusing: "UID STORE",
            ..ASourceServer::default()
        })
        .await;
        let destination = a_server_that_can("UIDPLUS").await;

        let across = waiting_for(a_move_across(
            &a_store(),
            &the_account_it_is_leaving(&source).await,
            "INBOX",
            THE_UID,
            &the_account_it_is_going_to(&destination).await,
            "Archive",
        ))
        .await
        .expect("a source that would not let go is an ending, not a failure");

        assert!(
            matches!(across, MovedAcross::ItArrivedAndTheSourceWouldNotLetGo(_)),
            "{across:?}"
        );
        assert!(
            everything_said_to(&destination)
                .await
                .to_uppercase()
                .contains("APPEND"),
            "the message never reached the destination, so this test is not \
             about what it is named after"
        );
    }

    #[tokio::test]
    async fn test_a_source_that_cannot_remove_one_message_says_it_is_here_and_marked() {
        // No UIDPLUS. The only expunge available would take every message in
        // the mailbox flagged for removal, including ones flagged from another
        // client, so the message is marked and left. It is at the destination
        // and it is here marked, which is a different fact from the one above
        // and syncs back as a row marked deleted.
        let source = a_source_server(ASourceServer {
            capabilities: "",
            ..ASourceServer::default()
        })
        .await;
        let destination = a_server_that_can("UIDPLUS").await;

        let across = waiting_for(a_move_across(
            &a_store(),
            &the_account_it_is_leaving(&source).await,
            "INBOX",
            THE_UID,
            &the_account_it_is_going_to(&destination).await,
            "Archive",
        ))
        .await
        .expect("the move to end");

        assert_eq!(
            across,
            MovedAcross::ItArrivedAndIsStillHereMarked(
                crate::service::protocols::imap::StillHere::TheServerCannotRemoveOneMessage
            ),
            "a server that cannot remove one message reported the message as gone"
        );
        let said = everything_said_to(&source).await;
        assert!(said.to_uppercase().contains("UID STORE"), "{said}");
        assert!(!said.to_uppercase().contains("EXPUNGE"), "{said}");
    }

    #[tokio::test]
    async fn test_a_refusal_is_an_answer_so_a_message_already_there_is_not_read_as_an_arrival() {
        // The destination said no, and the folder happens to already hold a
        // message carrying the same identifier: an earlier copy of this one, or
        // a message a stranger wrote that identifier onto. A refusal is an
        // answer, so nothing needs asking, and asking anyway would read that
        // older message as the one just sent and remove the only copy that
        // really exists.
        let source = a_source_server(ASourceServer::default()).await;
        let destination =
            ADestinationThat::takes_the_append(TheAppend::IsRefused, vec![Ok(vec![9])]);

        let across = waiting_for(a_move_across(
            &a_store(),
            &the_account_it_is_leaving(&source).await,
            "INBOX",
            THE_UID,
            &destination,
            "Archive",
        ))
        .await
        .expect("a refused append is an ending, not a failure");

        assert!(
            matches!(across, MovedAcross::TheDestinationRefusedIt(_)),
            "{across:?}"
        );
        let said = everything_said_to(&source).await;
        assert!(!said.to_uppercase().contains("STORE"), "{said}");
        assert!(!said.to_uppercase().contains("EXPUNGE"), "{said}");
    }

    #[tokio::test]
    async fn test_an_append_with_no_answer_that_is_found_afterwards_is_removed_at_the_source() {
        // The destination stopped answering and the message is there. Asking is
        // what turns "nobody knows" into "it landed", and only then may
        // anything be removed. The search answers nothing before the append and
        // one message after it, which is what an arrival looks like.
        let source = a_source_server(ASourceServer::default()).await;
        let destination = ADestinationThat::takes_the_append(
            TheAppend::IsNeverAnswered,
            vec![Ok(vec![]), Ok(vec![9])],
        );

        let across = waiting_for(a_move_across(
            &a_store(),
            &the_account_it_is_leaving(&source).await,
            "INBOX",
            THE_UID,
            &destination,
            "Archive",
        ))
        .await
        .expect("the move to end");

        assert_eq!(across, MovedAcross::ItArrivedAndTheSourceLetItGo);
        let asked = destination.everything_it_was_asked().await;
        assert!(
            asked
                .iter()
                .any(|line| line.starts_with("SEARCH")
                    && line.contains(THE_IDENTIFIER_AS_IT_IS_HELD)),
            "the destination was never asked whether it had the message: {asked:?}"
        );
        // Twice: once before the message went, so a message already there could
        // not be mistaken for it, and once after.
        assert_eq!(
            asked
                .iter()
                .filter(|line| line.starts_with("SEARCH"))
                .count(),
            2,
            "{asked:?}"
        );
        let said = everything_said_to(&source).await;
        assert!(said.to_uppercase().contains("UID STORE"), "{said}");
    }

    #[tokio::test]
    async fn test_an_append_with_no_answer_that_is_not_found_removes_nothing() {
        // Asked, and the folder holds nothing new carrying that identifier. The
        // message never arrived, so it is still exactly where it was and
        // nothing at the source may be touched.
        let source = a_source_server(ASourceServer::default()).await;
        let destination = ADestinationThat::takes_the_append(
            TheAppend::IsNeverAnswered,
            vec![Ok(vec![]), Ok(vec![])],
        );

        let across = waiting_for(a_move_across(
            &a_store(),
            &the_account_it_is_leaving(&source).await,
            "INBOX",
            THE_UID,
            &destination,
            "Archive",
        ))
        .await
        .expect("the move to end");

        assert!(
            matches!(across, MovedAcross::ItNeverArrivedSoNothingWasRemoved(_)),
            "{across:?}"
        );
        let said = everything_said_to(&source).await;
        assert!(!said.to_uppercase().contains("STORE"), "{said}");
        assert!(!said.to_uppercase().contains("EXPUNGE"), "{said}");
    }

    #[tokio::test]
    async fn test_an_append_with_no_answer_about_a_message_with_no_identifier_removes_nothing() {
        // A message that arrived with no `Message-ID` cannot be asked about,
        // and searching for an empty identifier is not a confirmation of
        // anything: it matches whatever the server decides it matches.
        // `mail_sync.rs:564` keeps such a message with an empty identifier, so
        // this is a real message rather than an invented one.
        let source = a_source_server(ASourceServer {
            message_id: "",
            ..ASourceServer::default()
        })
        .await;
        let destination = ADestinationThat::takes_the_append(TheAppend::IsNeverAnswered, vec![]);

        let across = waiting_for(a_move_across(
            &a_store(),
            &the_account_it_is_leaving(&source).await,
            "INBOX",
            THE_UID,
            &destination,
            "Archive",
        ))
        .await
        .expect("the move to end");

        assert!(
            matches!(across, MovedAcross::ItIsNotKnownWhereItIs(_)),
            "{across:?}"
        );
        let asked = destination.everything_it_was_asked().await;
        assert!(
            !asked.iter().any(|line| line.starts_with("SEARCH")),
            "an empty identifier was sent to a server as a search key: {asked:?}"
        );
        let said = everything_said_to(&source).await;
        assert!(!said.to_uppercase().contains("STORE"), "{said}");
        assert!(!said.to_uppercase().contains("EXPUNGE"), "{said}");
    }

    #[tokio::test]
    async fn test_a_message_that_was_already_there_is_not_read_as_the_one_just_sent() {
        // The destination stopped answering, and the folder holds a message
        // carrying the same identifier that it already held before the send.
        // That is not this message arriving. It is an earlier copy of it, or a
        // message somebody else wrote that identifier onto, and reading either
        // as an arrival removes the source copy of a message that never left.
        //
        // The identifier is a header a stranger wrote, so the question it can
        // answer is bounded on purpose: not "is something carrying this here",
        // which a stranger decides, but "is something carrying this here that
        // was not here a moment ago", which the sending decides.
        let source = a_source_server(ASourceServer::default()).await;
        let destination = ADestinationThat::takes_the_append(
            TheAppend::IsNeverAnswered,
            vec![Ok(vec![9]), Ok(vec![9])],
        );

        let across = waiting_for(a_move_across(
            &a_store(),
            &the_account_it_is_leaving(&source).await,
            "INBOX",
            THE_UID,
            &destination,
            "Archive",
        ))
        .await
        .expect("the move to end");

        assert!(
            matches!(across, MovedAcross::ItNeverArrivedSoNothingWasRemoved(_)),
            "a message that was in the folder before the send was counted as the \
             one that was sent: {across:?}"
        );
        let said = everything_said_to(&source).await;
        assert!(!said.to_uppercase().contains("STORE"), "{said}");
        assert!(!said.to_uppercase().contains("EXPUNGE"), "{said}");
    }

    #[tokio::test]
    async fn test_a_folder_that_could_not_be_read_beforehand_settles_nothing_afterwards() {
        // The folder could not be read before the message went, so what it
        // holds now cannot be compared with anything. That is not the same as
        // the folder having been empty, and treating it as empty would make any
        // message carrying that identifier an arrival.
        //
        // The failed read does not stop the move: the append is still sent, and
        // it is only an append whose answer never comes that this leaves
        // unsettled.
        let source = a_source_server(ASourceServer::default()).await;
        let destination = ADestinationThat::takes_the_append(
            TheAppend::IsNeverAnswered,
            vec![
                Err(crate::common::Error::Protocol(
                    "the folder could not be opened".to_string(),
                )),
                Ok(vec![9]),
            ],
        );

        let across = waiting_for(a_move_across(
            &a_store(),
            &the_account_it_is_leaving(&source).await,
            "INBOX",
            THE_UID,
            &destination,
            "Archive",
        ))
        .await
        .expect("the move to end");

        assert!(
            matches!(across, MovedAcross::ItIsNotKnownWhereItIs(_)),
            "{across:?}"
        );
        assert!(
            destination
                .everything_it_was_asked()
                .await
                .iter()
                .any(|line| line.starts_with("APPEND")),
            "a folder that could not be read beforehand stopped the message being sent"
        );
        let said = everything_said_to(&source).await;
        assert!(!said.to_uppercase().contains("STORE"), "{said}");
    }

    // ── The question on its own, which is what 04.1-04 will call ────────────

    #[tokio::test]
    async fn test_a_message_with_no_identifier_is_never_asked_about() {
        let destination = ADestinationThat::takes_the_append(TheAppend::Lands, vec![Ok(vec![9])]);

        for nothing_to_ask_about in [None, Some(""), Some("   ")] {
            let answer = whether_the_destination_has_it(
                &destination,
                "Archive",
                the_identifier_to_ask_about(nothing_to_ask_about),
                Some(&[]),
            )
            .await;

            assert!(
                matches!(answer, WhetherItLanded::ItCannotBeAsked(_)),
                "{nothing_to_ask_about:?} was treated as a question: {answer:?}"
            );
        }
        assert!(
            destination.everything_it_was_asked().await.is_empty(),
            "an identifier that names no message was sent to a server as a search key"
        );
    }

    #[tokio::test]
    async fn test_nothing_new_carrying_the_identifier_is_not_there_rather_than_unanswerable() {
        // Three answers rather than two, and this is the one that would be lost
        // if a failure and an absence were folded together. A folder that
        // answered and holds nothing new is a folder that really does not have
        // the message, and something can be said about it.
        let destination =
            ADestinationThat::takes_the_append(TheAppend::Lands, vec![Ok(vec![1, 2])]);

        let answer = whether_the_destination_has_it(
            &destination,
            "Archive",
            Some(THE_IDENTIFIER_AS_IT_IS_HELD),
            Some(&[1, 2]),
        )
        .await;

        assert_eq!(answer, WhetherItLanded::ItIsNotThere);
    }

    #[tokio::test]
    async fn test_something_carrying_the_identifier_that_was_not_there_before_is_an_arrival() {
        let destination =
            ADestinationThat::takes_the_append(TheAppend::Lands, vec![Ok(vec![1, 2, 7])]);

        let answer = whether_the_destination_has_it(
            &destination,
            "Archive",
            Some(THE_IDENTIFIER_AS_IT_IS_HELD),
            Some(&[1, 2]),
        )
        .await;

        assert_eq!(answer, WhetherItLanded::ItIsThere);
    }

    #[tokio::test]
    async fn test_a_folder_that_will_not_answer_cannot_be_asked() {
        let destination = ADestinationThat::takes_the_append(
            TheAppend::Lands,
            vec![Err(crate::common::Error::Protocol(
                "the server would not search".to_string(),
            ))],
        );

        let answer = whether_the_destination_has_it(
            &destination,
            "Archive",
            Some(THE_IDENTIFIER_AS_IT_IS_HELD),
            Some(&[]),
        )
        .await;

        let WhetherItLanded::ItCannotBeAsked(why) = answer else {
            panic!("a server that would not answer was read as one that said no: {answer:?}");
        };
        assert!(why.contains("would not search"), "{why}");
    }

    #[test]
    fn test_a_refusal_is_told_apart_from_an_answer_that_never_came() {
        // The one line the whole safeguard turns on. A server that said no has
        // answered and the message did not land; a connection that dropped has
        // not answered at all.
        use crate::common::Error;

        assert!(the_destination_never_answered(&Error::Network(
            "the connection to the mail server failed".to_string()
        )));
        for answered in [
            Error::Protocol("the server refused it".to_string()),
            Error::Security("Allow Changes is off".to_string()),
            Error::InPlainWords("something already worded".to_string()),
            Error::Other("something else".to_string()),
        ] {
            assert!(
                !the_destination_never_answered(&answered),
                "{answered:?} was read as an answer that never came, so a message the \
                 destination never took could be searched for and then removed at the source"
            );
        }
    }

    #[tokio::test]
    async fn test_a_move_that_crosses_never_sends_a_copy_or_a_move_to_the_source() {
        // `COPY` and `MOVE` name a mailbox on the connection they are sent
        // down. Sent at the source for a destination in another account they
        // would look up the folder path on the wrong server, and on a server
        // that happened to have a folder of that name the message would land in
        // the wrong mailbox rather than failing.
        let source = a_source_server(ASourceServer::default()).await;
        let destination = a_server_that_can("UIDPLUS").await;

        waiting_for(a_move_across(
            &a_store(),
            &the_account_it_is_leaving(&source).await,
            "INBOX",
            THE_UID,
            &the_account_it_is_going_to(&destination).await,
            "Archive",
        ))
        .await
        .expect("the move to be made");

        let said = everything_said_to(&source).await;
        assert!(!said.to_uppercase().contains("UID COPY"), "{said}");
        assert!(!said.to_uppercase().contains("UID MOVE"), "{said}");
    }

    // ── The bytes kept while the move is in the air ──────────────────────

    #[tokio::test]
    async fn test_the_message_is_already_on_the_disk_when_the_append_goes_out() {
        // The instant that decides whether any of this is worth having. If the
        // bytes are written after the append, then the whole window this store
        // exists for, between handing the message over and hearing back, is a
        // window it does not cover.
        //
        // Read from inside the append rather than checked afterwards, for the
        // reason the ordering witness beside it gives: by the time the move has
        // ended the row has been taken away again, and a store that never held
        // anything is indistinguishable from one that held it and let go.
        let store = a_store();
        let source = a_source_server(ASourceServer::default()).await;
        let destination = ADestinationThat::takes_the_append(TheAppend::Lands, vec![Ok(vec![])])
            .watching_the_store(&store);

        let across = waiting_for(a_move_across(
            &store,
            &the_account_it_is_leaving(&source).await,
            "INBOX",
            THE_UID,
            &destination,
            "Archive",
        ))
        .await
        .expect("the move to be made");

        assert_eq!(across, MovedAcross::ItArrivedAndTheSourceLetItGo);
        let held = destination.what_the_store_was_holding().await;
        assert_eq!(
            held.len(),
            1,
            "the message was not being kept when the append went out, so a \
             program stopped between the two would have nothing to resume from"
        );
        assert_eq!(
            held[0].raw,
            THE_MESSAGE.as_bytes(),
            "what was kept is not the message that was sent"
        );
        assert_eq!(held[0].to_folder, "Archive");
        assert_eq!(held[0].to_account_id, THE_DESTINATION_ACCOUNT);
        assert_eq!(
            held[0].was_there_before,
            Some(Vec::new()),
            "the folder was read before the message went and the answer was not \
             kept, so a resume could never settle whether the append landed"
        );
    }

    /// Which ending this is, with no wildcard arm.
    ///
    /// A seventh way for a crossing to end stops this compiling until somebody
    /// says what it is called, and the list below then has to gain it too.
    fn which_ending(across: &MovedAcross) -> &'static str {
        match across {
            MovedAcross::ItArrivedAndTheSourceLetItGo => "the source let it go",
            MovedAcross::ItArrivedAndIsStillHereMarked(_) => "still here, marked",
            MovedAcross::ItArrivedAndTheSourceWouldNotLetGo(_) => "still here, unmarked",
            MovedAcross::TheDestinationRefusedIt(_) => "the destination refused it",
            MovedAcross::ItNeverArrivedSoNothingWasRemoved(_) => "it never arrived",
            MovedAcross::ItIsNotKnownWhereItIs(_) => "nobody knows where it is",
        }
    }

    #[tokio::test]
    async fn test_no_way_a_crossed_move_can_end_leaves_the_bytes_behind() {
        // Every arm, not the happy one. An ending that forgets to let go
        // leaves a whole unencrypted message on somebody's disk with nothing
        // anywhere saying it is there, and on the next start they are asked
        // about a move that ended perfectly well.
        //
        // The endings are collected as they happen and the set is compared
        // against the whole of the type at the end, so an ending nothing here
        // reaches fails this rather than passing quietly.
        let mut reached: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();

        for scenario in 0..6 {
            let store = a_store();
            let source = a_source_server(match scenario {
                2 => ASourceServer {
                    refusing: "UID STORE",
                    ..ASourceServer::default()
                },
                1 => ASourceServer {
                    capabilities: "",
                    ..ASourceServer::default()
                },
                _ => ASourceServer::default(),
            })
            .await;

            let across = match scenario {
                0..=2 => {
                    let destination = a_server_that_can("UIDPLUS").await;
                    waiting_for(a_move_across(
                        &store,
                        &the_account_it_is_leaving(&source).await,
                        "INBOX",
                        THE_UID,
                        &the_account_it_is_going_to(&destination).await,
                        "Archive",
                    ))
                    .await
                }
                3 => {
                    let destination = a_server_that_refuses("UIDPLUS", "APPEND").await;
                    waiting_for(a_move_across(
                        &store,
                        &the_account_it_is_leaving(&source).await,
                        "INBOX",
                        THE_UID,
                        &the_account_it_is_going_to(&destination).await,
                        "Archive",
                    ))
                    .await
                }
                4 => {
                    let destination = ADestinationThat::takes_the_append(
                        TheAppend::IsNeverAnswered,
                        vec![Ok(vec![]), Ok(vec![])],
                    );
                    waiting_for(a_move_across(
                        &store,
                        &the_account_it_is_leaving(&source).await,
                        "INBOX",
                        THE_UID,
                        &destination,
                        "Archive",
                    ))
                    .await
                }
                _ => {
                    let destination = ADestinationThat::takes_the_append(
                        TheAppend::IsNeverAnswered,
                        vec![
                            Ok(vec![]),
                            Err(crate::common::Error::Protocol("no search here".to_string())),
                        ],
                    );
                    waiting_for(a_move_across(
                        &store,
                        &the_account_it_is_leaving(&source).await,
                        "INBOX",
                        THE_UID,
                        &destination,
                        "Archive",
                    ))
                    .await
                }
            }
            .expect("every one of these is an ending rather than a failure");

            reached.insert(which_ending(&across));
            assert!(
                store.moves_it_is_holding().is_empty(),
                "the bytes were still being kept after a move ended as \
                 {across:?}"
            );
        }

        let all: std::collections::BTreeSet<&str> = [
            "the source let it go",
            "still here, marked",
            "still here, unmarked",
            "the destination refused it",
            "it never arrived",
            "nobody knows where it is",
        ]
        .into_iter()
        .collect();
        assert_eq!(
            reached, all,
            "a way a crossed move can end was never reached here, so nothing \
             above says whether it lets go of the bytes"
        );
    }

    // ── Finishing a move the program was stopped part way through ────────

    /// A move left unfinished, as the store would hand one back.
    fn left_unfinished(
        identifier: &str,
        was_there_before: Option<Vec<u32>>,
    ) -> AMoveLeftUnfinished {
        AMoveLeftUnfinished {
            message_row_id: 1,
            subject: "Lunch".to_string(),
            identifier: identifier.to_string(),
            from_account_id: "acc-1".to_string(),
            from_folder: "INBOX".to_string(),
            uid: THE_UID,
            to_account_id: THE_DESTINATION_ACCOUNT.to_string(),
            to_folder: "Archive".to_string(),
            flags: Some("(\\Seen)".to_string()),
            arrived: Some("\"01-Aug-2026 10:00:00 +0000\"".to_string()),
            was_there_before,
            raw: THE_MESSAGE.as_bytes().to_vec(),
            started_at: "2026-09-08T09:00:00Z".to_string(),
        }
    }

    #[tokio::test]
    async fn test_a_resumed_move_asks_the_destination_before_it_sends_anything() {
        // The one way this store can do harm that not having it would not. A
        // row surviving a restart means the program stopped somewhere between
        // the write and the clear, and the append may well have landed.
        // Sending it again because a row exists is how somebody ends up with
        // two copies of their message and nothing saying why.
        let source = a_source_server(ASourceServer::default()).await;
        let destination = ADestinationThat::takes_the_append(TheAppend::Lands, vec![Ok(vec![9])]);

        waiting_for(finish_the_move(
            &left_unfinished(THE_IDENTIFIER_AS_IT_IS_HELD, Some(Vec::new())),
            &destination,
            &the_account_it_is_leaving(&source).await,
        ))
        .await;

        let asked = destination.everything_it_was_asked().await;
        let Some(first) = asked.first() else {
            panic!("the destination was never asked anything at all");
        };
        assert!(
            first.starts_with("SEARCH"),
            "the first thing a resumed move said to the destination was \
             {first:?}, so the message was sent again without asking whether it \
             was already there"
        );
    }

    #[tokio::test]
    async fn test_a_resumed_move_the_destination_already_has_is_only_removed() {
        // The append landed before the program stopped. Sending it again would
        // make the second copy; the only thing left to do is the removal.
        let source = a_source_server(ASourceServer::default()).await;
        let destination = ADestinationThat::takes_the_append(TheAppend::Lands, vec![Ok(vec![9])]);

        let across = waiting_for(finish_the_move(
            &left_unfinished(THE_IDENTIFIER_AS_IT_IS_HELD, Some(Vec::new())),
            &destination,
            &the_account_it_is_leaving(&source).await,
        ))
        .await;

        assert_eq!(across, MovedAcross::ItArrivedAndTheSourceLetItGo);
        let asked = destination.everything_it_was_asked().await;
        assert!(
            !asked.iter().any(|said| said.starts_with("APPEND")),
            "the message was sent again to a destination that already had it: \
             {asked:?}"
        );
        assert!(
            source.was_told("UID EXPUNGE 4").await,
            "the move was not finished at the source"
        );
    }

    #[tokio::test]
    async fn test_a_resumed_move_the_destination_does_not_have_is_sent_again_then_removed() {
        // The append never landed, so the kept bytes are what completes the
        // move without going back to the source for the message. This is the
        // case the whole store exists for.
        let source = a_source_server(ASourceServer::default()).await;
        let destination = ADestinationThat::takes_the_append(TheAppend::Lands, vec![Ok(vec![])]);

        let across = waiting_for(finish_the_move(
            &left_unfinished(THE_IDENTIFIER_AS_IT_IS_HELD, Some(Vec::new())),
            &destination,
            &the_account_it_is_leaving(&source).await,
        ))
        .await;

        assert_eq!(across, MovedAcross::ItArrivedAndTheSourceLetItGo);
        let asked = destination.everything_it_was_asked().await;
        assert_eq!(
            asked,
            vec![
                format!("SEARCH Archive {THE_IDENTIFIER_AS_IT_IS_HELD}"),
                "APPEND Archive".to_string(),
            ],
            "a resumed move that had to send the message again did not ask \
             first, or did not send it"
        );
        assert!(
            !everything_said_to(&source)
                .await
                .to_uppercase()
                .contains("BODY.PEEK[]"),
            "the message was fetched from the source again, so the kept bytes \
             bought nothing"
        );
    }

    #[tokio::test]
    async fn test_a_resumed_move_that_cannot_be_asked_about_sends_nothing_to_either_server() {
        // A message with no identifier of its own, which real mail is:
        // `mail_sync.rs:564` keeps such a message with an empty one. Nothing
        // can be asked, so nothing may be sent: appending would risk a second
        // copy and removing would risk the only one.
        let source = a_source_server(ASourceServer::default()).await;
        let destination = ADestinationThat::takes_the_append(TheAppend::Lands, vec![Ok(vec![9])]);

        let across = waiting_for(finish_the_move(
            &left_unfinished("", Some(Vec::new())),
            &destination,
            &the_account_it_is_leaving(&source).await,
        ))
        .await;

        assert!(
            matches!(across, MovedAcross::ItIsNotKnownWhereItIs(_)),
            "{across:?}"
        );
        assert!(
            destination.everything_it_was_asked().await.is_empty(),
            "something was said to the destination about a move nothing can \
             settle"
        );
        let said = everything_said_to(&source).await;
        assert!(!said.to_uppercase().contains("STORE"), "{said}");
        assert!(!said.to_uppercase().contains("EXPUNGE"), "{said}");
    }

    #[tokio::test]
    async fn test_a_resumed_move_whose_folder_nobody_read_settles_nothing() {
        // The second unanswerable case, and it is not the same as the first. A
        // folder nobody read before the message was sent cannot tell a message
        // that has just arrived from one that was always there, so a hit
        // proves nothing.
        let source = a_source_server(ASourceServer::default()).await;
        let destination = ADestinationThat::takes_the_append(TheAppend::Lands, vec![Ok(vec![9])]);

        let across = waiting_for(finish_the_move(
            &left_unfinished(THE_IDENTIFIER_AS_IT_IS_HELD, None),
            &destination,
            &the_account_it_is_leaving(&source).await,
        ))
        .await;

        assert!(
            matches!(across, MovedAcross::ItIsNotKnownWhereItIs(_)),
            "{across:?}"
        );
        assert!(destination.everything_it_was_asked().await.is_empty());
    }

    #[test]
    fn test_the_three_answers_lead_to_three_different_things_being_done() {
        // Two of them folded together is the failure this whole phase is
        // about. "It is not there" sends the message again; "nobody could find
        // out" must send nothing at all, and reading the second as the first
        // is how a message is removed from the only server that has it.
        assert_eq!(
            how_to_finish(&WhetherItLanded::ItIsThere),
            FinishingIt::TakeItOffTheSource
        );
        assert_eq!(
            how_to_finish(&WhetherItLanded::ItIsNotThere),
            FinishingIt::SendItAgainThenTakeItOff
        );
        assert_eq!(
            how_to_finish(&WhetherItLanded::ItCannotBeAsked("no search".to_string())),
            FinishingIt::NothingCanBeSent("no search".to_string())
        );
    }

    // ── What somebody is told about it ───────────────────────────────────

    /// Both accounts still set up, named as somebody hears them.
    fn both_accounts() -> TheAccountsInvolved<'static> {
        TheAccountsInvolved {
            it_is_in: Some("Work"),
            it_was_going_to: Some("Home"),
        }
    }

    #[test]
    fn test_the_offer_says_where_the_message_is_and_does_not_read_like_an_error() {
        // Nothing was lost and it must not sound as though something was. A
        // move is append then remove and the removal is last, so at every
        // point the program can stop the message is still at the account it
        // came from.
        let AboutAnUnfinishedMove::Ask { words, .. } = what_to_say_about_an_unfinished_move(
            &left_unfinished(THE_IDENTIFIER_AS_IT_IS_HELD, Some(Vec::new())),
            both_accounts(),
        ) else {
            panic!("a move that can be finished was not offered");
        };

        assert!(words.contains("Lunch"), "{words}");
        assert!(words.contains("INBOX in Work"), "{words}");
        assert!(words.contains("Archive in Home"), "{words}");
        assert!(
            words.contains("nothing was lost"),
            "the words do not say the message is safe: {words}"
        );
        for alarming in ["error", "failed", "lost the message", "went wrong"] {
            assert!(
                !words.to_lowercase().contains(alarming),
                "the words read like something went wrong ({alarming}): {words}"
            );
        }
    }

    #[test]
    fn test_a_move_nothing_can_settle_is_told_rather_than_offered() {
        // Offering to finish a move that cannot be finished is offering a
        // choice whose only honest outcome is nothing happening. The person is
        // told where the message is and where to look instead.
        let said = what_to_say_about_an_unfinished_move(
            &left_unfinished("", Some(Vec::new())),
            both_accounts(),
        );

        let AboutAnUnfinishedMove::JustSay(words) = said else {
            panic!("a move nothing can settle was offered as a choice: {said:?}");
        };
        assert!(words.contains("INBOX in Work"), "{words}");
        assert!(words.contains("Archive in Home"), "{words}");
        assert!(
            words.contains("nothing was lost"),
            "the words do not say the message is safe: {words}"
        );
    }

    #[test]
    fn test_a_move_to_an_account_that_has_gone_is_told_rather_than_offered() {
        // A row can outlive the account it names. Offering to finish the move
        // would be offering a command with nowhere to send it, and the answer
        // Yes would sign in to nothing.
        let said = what_to_say_about_an_unfinished_move(
            &left_unfinished(THE_IDENTIFIER_AS_IT_IS_HELD, Some(Vec::new())),
            TheAccountsInvolved {
                it_is_in: Some("Work"),
                it_was_going_to: None,
            },
        );

        assert!(
            matches!(said, AboutAnUnfinishedMove::JustSay(_)),
            "{said:?}"
        );
    }

    #[test]
    fn test_only_the_crossing_keeps_a_message_while_it_moves() {
        // The store is written from one place, so a move that stays inside one
        // account cannot write a row: the same-account path is a `COPY` down
        // one connection and never reaches this file at all.
        //
        // An exact list rather than an emptiness check, so a reading that has
        // quietly stopped matching anything fails here rather than reporting a
        // clean tree. That is the companion this project asks any
        // source-reading check to carry, and the reason is in
        // `how_it_arrived.rs`, where the version without one passed against a
        // scope that could not see the path that had gone wrong.
        assert_eq!(
            shipped_files_calling(".keep_the_message_while_it_moves("),
            vec!["src/application/mail_across_accounts.rs".to_string()]
        );
    }

    /// Every shipped source file that calls a named method.
    ///
    /// The whole of `src/`, and the shipped half of each file through
    /// [`crate::common::what_ships::what_ships`]: the test halves here call
    /// these directly on purpose, and a reading that saw them would report
    /// every one as a caller.
    fn shipped_files_calling(wanted: &str) -> Vec<String> {
        fn walk(dir: &std::path::Path, into: &mut Vec<std::path::PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, into);
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    into.push(path);
                }
            }
        }
        let mut files = Vec::new();
        walk(std::path::Path::new("src"), &mut files);
        files.sort();

        let mut named: Vec<String> = files
            .iter()
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .filter(|path| !path.starts_with("src/data/message_cache"))
            .filter(|path| {
                std::fs::read_to_string(path).is_ok_and(|source| {
                    crate::common::what_ships::what_ships(&source).contains(wanted)
                })
            })
            .collect();
        named.sort();
        named
    }
}

//! Bringing a POP mailbox down into the local Inbox.
//!
//! POP3 has no folders, no flags and no server-side state, so a sync here is
//! simpler than the IMAP one and more dangerous. Simpler because there is one
//! mailbox and one question: which of these have we not got yet. More dangerous
//! because the only way to keep a POP mailbox from filling is to delete from
//! it, and POP3's delete is permanent with no trash behind it.
//!
//! # What decides "not got yet"
//!
//! The UIDL, the identifier the server gives each message. Message numbers are
//! assigned per session and shift as messages are deleted, so a number from one
//! connection means a different message in the next. Anything keyed on numbers
//! downloads mail twice or skips it, and both look like the account working.
//!
//! # When mail is removed from the server
//!
//! Only when somebody asked for it, and only after the days they said. The
//! default is to leave everything, which costs them a mailbox that fills and
//! saves them the case where this computer is the only copy and it is gone.

use crate::application::summing_up::SummingUp;
use crate::common::Result;

/// What a sync of a POP mailbox did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PopSync {
    pub fetched: usize,
    /// How many were removed from the server, having been kept long enough.
    pub removed_from_server: usize,
    /// How many were old enough to go and were held back by Allow Changes.
    ///
    /// A removal the setting refuses is not a failed check. Everything asked
    /// for arrived; the only thing that did not happen is the clearing out,
    /// and that is what the setting is for. Counted so the status line can say
    /// so, because otherwise a mailbox quietly stops emptying and the only
    /// sign is that it fills up.
    pub waiting_on_the_setting: usize,
    /// How many are on the server in total.
    pub on_server: usize,
    /// What the rules did to the mail that just arrived.
    pub filtered: crate::application::mail_sync::Filtered,
    /// The rows this check wrote, oldest first.
    ///
    /// So anything that has to look at each new message can find them without
    /// reading the whole folder again. A check that downloaded nothing reports
    /// none, which is what keeps a message from being looked at twice.
    pub written: Vec<i64>,
}

/// What a check of a POP mailbox did, in the words the status line uses.
///
/// Named here rather than built where it is shown, for the reason the IMAP
/// summary is: a sentence assembled at the call site cannot be argued about in
/// a test, and the one this replaces was assembled inside a closure on a
/// background thread where nothing could reach it. What the rules did comes
/// from [`crate::application::mail_sync::say_what_the_rules_did`], so a reader
/// hears the same words whichever kind of account the mail came from.
///
/// Not yet called. The POP check's status line is still put together where it
/// is spoken, in the main window, which is a file another change owns as this
/// is written, so the last step is a single call there in place of that
/// assembly. Until it happens a POP reader hears what was downloaded and
/// nothing about their rules, and the wording lives in two places, which is
/// the state this function exists to end rather than one to keep.
pub fn what_the_pop_check_did(result: &PopSync) -> String {
    let mut said = SummingUp::opening(format!(
        "{} new, {} on the server",
        result.fetched, result.on_server
    ));
    if result.removed_from_server > 0 {
        // Said out loud, because it is mail leaving a server for good and the
        // only warning anybody gets that the policy is running.
        said.count(format!(
            "{} removed from the server",
            result.removed_from_server
        ));
    }
    if result.waiting_on_the_setting > 0 {
        // The other half, and it needs saying just as much. This account is
        // set to clear its server and the setting is holding that back, so
        // without a word here the mailbox quietly fills and the first sign is
        // the provider refusing new mail.
        said.sentence(crate::application::allowed::removals_waiting_here(
            result.waiting_on_the_setting,
        ));
    }
    crate::application::mail_sync::say_what_the_rules_did(&result.filtered, &mut said);
    said.spoken()
}

/// What the account said about clearing the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Housekeeping {
    /// Whether to leave downloaded mail where it is.
    pub leave_on_server: bool,
    /// Remove it this many days after downloading. Nought means never.
    pub remove_after_days: u32,
}

impl Housekeeping {
    /// What an account that has said nothing means.
    ///
    /// Leave everything. It is the answer that cannot lose somebody's only
    /// copy, and the cost of getting it wrong the other way is a full mailbox,
    /// which is recoverable.
    pub const CAUTIOUS: Self = Self {
        leave_on_server: true,
        remove_after_days: 0,
    };
}

/// What turning off "leave mail on the server" costs, for the account
/// settings screen to attach as that checkbox's accessible description.
///
/// Said here rather than only in the box's visible neighbourhood, because a
/// screen reader user tabbing onto the checkbox hears its name and its
/// checked state and stops there unless a description is attached to carry
/// the rest. POP's removal is worth the extra sentence: it has no Trash
/// behind it the way deleting mail on an IMAP account does, so once the days
/// below have passed there is nothing left to recover a copy from.
pub const SERVER_REMOVAL_IS_PERMANENT: &str = "Turning this off allows mail to be removed from the server for good, once the number \
     of days below has passed (0 there means never). POP has no Trash folder to recover \
     it from, unlike deleting mail on an IMAP account. Mail already downloaded to this \
     computer is unaffected.";

/// Which messages on the server have not been downloaded.
///
/// Compared by identifier, never by number. Returned in the order the server
/// listed them, which is oldest first, so an interrupted first sync has brought
/// down a run from the beginning rather than a scatter.
pub fn to_fetch<'a>(
    on_server: &'a [(u32, String)],
    already_have: &std::collections::HashSet<String>,
) -> Vec<&'a (u32, String)> {
    on_server
        .iter()
        .filter(|(_, uidl)| !already_have.contains(uidl))
        .collect()
}

/// Which downloaded messages may now be removed from the server.
///
/// Three things have to be true: somebody turned off leaving mail on the
/// server, they set a number of days, and that many days have passed since this
/// copy was downloaded. Any of the three missing means the message stays, which
/// is the answer that cannot lose it.
///
/// `downloaded` is what the cache holds, as an identifier and the day it
/// arrived. `today` is passed in rather than read, so this can be tested.
pub fn to_remove<'a>(
    on_server: &'a [(u32, String)],
    downloaded: &std::collections::HashMap<String, chrono::DateTime<chrono::Utc>>,
    housekeeping: Housekeeping,
    today: chrono::DateTime<chrono::Utc>,
) -> Vec<&'a (u32, String)> {
    if housekeeping.leave_on_server || housekeeping.remove_after_days == 0 {
        return Vec::new();
    }
    let keep_for = chrono::Duration::days(i64::from(housekeeping.remove_after_days));

    on_server
        .iter()
        .filter(|(_, uidl)| match downloaded.get(uidl) {
            // Only mail this computer actually has. Removing something never
            // downloaded would delete it having never been read.
            Some(when) => today.signed_duration_since(*when) >= keep_for,
            None => false,
        })
        .collect()
}

/// What a sync asks of a POP server.
///
/// Named for what it needs rather than for the protocol behind it. Four
/// methods, which is the whole of it. Saying it in a type is what lets the rest
/// of this module be tested: the order the work happens in, and in particular
/// that nothing is deleted until everything is written down, is the guarantee
/// the module is built on, and it had never been run in a test because running
/// it meant having a server and a mailbox to empty.
pub(crate) trait PopMailbox {
    /// Every message on the server, with its size and stable identifier.
    async fn list(&self) -> Result<Vec<crate::application::mail_controller::Pop3MessagePreview>>;

    /// One whole message, as it arrived.
    async fn retrieve(&self, id: u32) -> Result<Vec<u8>>;

    /// Mark a message to go. Nothing happens until the session ends politely.
    async fn mark_for_deletion(&self, id: u32) -> Result<()>;

    /// End the session, which is what commits the marks.
    async fn finish(&self) -> Result<()>;
}

impl PopMailbox for crate::application::mail_controller::MailController {
    async fn list(&self) -> Result<Vec<crate::application::mail_controller::Pop3MessagePreview>> {
        self.list_pop3_messages().await
    }

    async fn retrieve(&self, id: u32) -> Result<Vec<u8>> {
        self.fetch_pop3_message_body(id).await
    }

    async fn mark_for_deletion(&self, id: u32) -> Result<()> {
        self.delete_pop3_message(id).await
    }

    async fn finish(&self) -> Result<()> {
        self.finish_pop3().await
    }
}

/// Where downloaded mail is going.
///
/// The three together rather than three parameters in a row, because the
/// account and the folder are both identifiers and writing them the wrong way
/// round still compiles. Both are needed and they answer different questions:
/// the folder is where a message is filed, the account is the scope of "have we
/// had this one already", which is wider than one folder on purpose.
#[derive(Clone, Copy)]
pub(crate) struct Landing<'a> {
    pub cache: &'a crate::data::message_cache::MessageCache,
    pub account_id: &'a str,
    pub folder_id: i64,
}

/// Bring a POP mailbox into a folder, and clear the server if asked.
///
/// The order matters. Everything is downloaded and written before anything is
/// deleted, so an interruption leaves mail on the server rather than gone from
/// both places. POP3 helps here: DELE only marks, and nothing is committed
/// until the session ends politely, so a dropped connection undoes the lot.
///
/// `now` is passed in rather than read, for the same reason `to_remove` takes
/// it: it decides both which mail is old enough to leave the server and how a
/// message with no readable date of its own is dated, and neither can be tested
/// against a clock the test cannot set.
pub(crate) async fn sync<M: PopMailbox>(
    server: &M,
    into: &Landing<'_>,
    housekeeping: Housekeeping,
    in_junk_folder: bool,
    look_at_the_body: bool,
    now: chrono::DateTime<chrono::Utc>,
    filtering: Option<&crate::application::mail_sync::Filtering<'_>>,
) -> Result<PopSync> {
    let Landing {
        cache,
        account_id,
        folder_id,
    } = *into;
    let listing = server.list().await?;
    let on_server: Vec<(u32, String)> = listing
        .iter()
        .map(|message| (message.id, message.uidl.clone()))
        .collect();

    // The whole account, not just the inbox. A message somebody moved to the
    // trash or deleted has left the inbox and is still mail this computer has
    // had, and asking one folder brings it straight back on the next check.
    let already_have = cache.pop_uidls_for_account(account_id)?;
    let wanted = to_fetch(&on_server, &already_have);

    let mut written: Vec<i64> = Vec::new();
    for (id, uidl) in &wanted {
        let raw = server.retrieve(*id).await?;
        let parsed = crate::service::mime::parse(&raw)?;
        let uid = cache.next_local_uid(folder_id)?;
        let arrival = Arrival {
            raw: &raw,
            folder_id,
            uid,
            uidl,
            in_junk_folder,
            look_at_the_body,
            at: now,
        };
        let row = cache.upsert_message(&to_incoming(&parsed, &arrival))?;
        // The whole message is already here, so the body is stored now rather
        // than downloaded again on opening. POP has no way to ask for one
        // message twice once it has been removed from the server.
        cache.save_message_body(
            row,
            parsed.body_plain.as_deref(),
            parsed.body_html.as_deref(),
        )?;
        // And the bytes themselves, where this is signed mail, because a
        // signature can only ever be checked against exactly those. It matters
        // more here than anywhere: POP has no server to ask again, so a
        // signature nobody kept the bytes for is one nobody can ever check.
        // The call decides for itself whether there is anything to keep, and
        // for ordinary mail there is not.
        //
        // Logged and not fatal, the same as on the IMAP path. The message is
        // already written down by the time this runs, so a failure here costs a
        // verdict on one message and nothing else; ending the check on it would
        // say the mail had not arrived when it had, and would return before the
        // polite ending, which is the only thing that commits anything on a POP
        // server.
        if let Err(e) = cache.keep_signed_original(row, &raw) {
            tracing::warn!("Could not keep the form a signed message arrived in: {e}");
        }
        written.push(row);
    }

    // Widened for the same reason, and it matters differently: this is what the
    // removal policy counts from, so mail that leaves the inbox and loses its
    // time is mail that silently never leaves the server.
    let downloaded = cache.pop_download_times_for_account(account_id)?;
    let stale = to_remove(&on_server, &downloaded, housekeeping, now);
    let mut removed = 0;
    let mut waiting = 0;
    for (id, _) in &stale {
        match server.mark_for_deletion(*id).await {
            Ok(()) => removed += 1,
            // Held back by Allow Changes rather than refused by the server.
            // Nothing left this machine, the mail is still there, and the
            // check itself worked, so this is a wait and not a failure. The
            // same rule the calendar and the tasks syncs follow.
            Err(e) if crate::service::outward::was_refused_by_the_gate(&e) => waiting += 1,
            Err(e) => return Err(e),
        }
    }
    // Committed here, and reached either way. Until this runs, every DELE is a
    // mark the server throws away if the connection drops, and a session left
    // without it is a connection somebody's server holds open for nothing.
    server.finish().await?;

    // Rules, on what has just arrived and nothing else, the same rule the
    // IMAP path follows. Nothing ran them here at all: the Rules Manager has
    // no protocol gate, so somebody on POP could write rules, name them,
    // enable them, and never have one evaluated, while the changelog said
    // rules run on arriving mail with no exception written down.
    let mut filtered = match filtering {
        Some(rules) => crate::application::mail_sync::apply_rules(cache, rules, &written),
        None => crate::application::mail_sync::Filtered::default(),
    };
    // And the filing they asked for, which nothing here carried out. The
    // rules ran, the folder each message belonged in was worked out, and the
    // answer was dropped: blocking a sender wrote the rule down, moved
    // nothing, and said nothing about either half.
    let (filed, could_not) =
        file_where_the_rules_said(cache, account_id, folder_id, &filtered.to_move);
    filtered.changed += filed;
    for reason in &could_not {
        // Logged as well as said, the same as the IMAP path. A status line is
        // gone as soon as the next one replaces it, and the summary reads out
        // the first few and says how many others there are; this is where
        // those others are.
        //
        // Folder names and what went wrong, never a subject: a subject line is
        // close enough to the message to be held to the same rule as its body.
        tracing::warn!("A rule could not file a message: {reason}");
    }
    filtered.could_not_be_filed = could_not;

    Ok(PopSync {
        fetched: written.len(),
        filtered,
        removed_from_server: removed,
        waiting_on_the_setting: waiting,
        on_server: on_server.len(),
        written,
    })
}

/// Do the filing the rules asked for, and say how many really happened.
///
/// Nothing here reaches a server, and that is the whole difference from the
/// IMAP side. Mail collected over POP is on this computer once it has been
/// downloaded, and every folder a POP account has is on this computer too, so
/// filing is a row moving between two local folders and there is nobody to ask
/// first. That also settles which end of the new folder's numbering the row
/// takes a number from: the cache works that out from the folder itself, so
/// this cannot get it wrong and does not try.
///
/// One message at a time and one failure at a time. A rule that cannot be
/// carried out on one message is not a reason to stop filing the rest, and the
/// sentences it hands back are the IMAP path's own, so a reader hears the same
/// words whichever kind of account the mail came from.
fn file_where_the_rules_said(
    cache: &crate::data::message_cache::MessageCache,
    account_id: &str,
    from_id: i64,
    moves: &[crate::application::mail_sync::Moving],
) -> (usize, Vec<String>) {
    if moves.is_empty() {
        return (0, Vec::new());
    }
    // Turning the folder name a rule uses into a folder needs the account's
    // folder list. Without it nothing can be filed, and giving up quietly
    // would make that check read exactly like a check with no rules in it. One
    // sentence per message, as everywhere else here, so the count is right and
    // the summary folds the repeats into one.
    let every_one_of_them_dropped = || {
        vec![
            crate::application::mail_sync::THE_FOLDER_LIST_COULD_NOT_BE_READ.to_string();
            moves.len()
        ]
    };
    let Ok(folders) = cache.get_folders_for_account(account_id) else {
        return (0, every_one_of_them_dropped());
    };
    // The folder the mail landed in, so a sentence can say where a message
    // that was not filed actually is. It was written into a moment ago, so its
    // absence here means the list itself is not to be trusted.
    let Some(from) = folders.iter().find(|folder| folder.id == from_id) else {
        return (0, every_one_of_them_dropped());
    };

    let mut done = 0;
    let mut could_not = Vec::new();
    for moving in moves {
        let Some(into) =
            crate::application::mail_sync::the_folder_a_rule_names(&folders, &moving.into)
        else {
            could_not.push(crate::application::mail_sync::no_folder_of_that_name(
                &moving.into,
            ));
            continue;
        };
        // Already where the rule wants it, so there is nothing to do and
        // nothing to report. Rules run on whatever has just arrived, and a
        // rule that files a sender into a folder goes on matching that
        // sender's mail once it is in that folder.
        if into.id == from.id {
            continue;
        }
        match cache.move_message(moving.message_row, into.id) {
            Ok(()) => done += 1,
            Err(why) => could_not.push(crate::application::mail_sync::it_is_still_where_it_was(
                &into.name, &from.name, &why,
            )),
        }
    }
    (done, could_not)
}

/// One message as it came down, and where it is going.
///
/// A struct rather than more parameters: `to_incoming` was already taking two
/// numbers next to each other, which is where a folder and a message number get
/// written the wrong way round and still compile.
struct Arrival<'a> {
    /// The whole message, as the server sent it.
    raw: &'a [u8],
    folder_id: i64,
    uid: u32,
    /// The identifier the server gave it.
    uidl: &'a str,
    in_junk_folder: bool,
    /// Whether to read the message itself as well as its headers.
    ///
    /// Carried here rather than read from the settings file per message: this
    /// runs inside the download loop, and reading a file once per message on a
    /// first sync of a full mailbox is a cost nobody asked for.
    look_at_the_body: bool,
    /// When this computer got it.
    at: chrono::DateTime<chrono::Utc>,
}

/// Turn a downloaded message into the row the cache stores.
fn to_incoming(
    parsed: &crate::service::mime::ParsedMessage,
    arrival: &Arrival<'_>,
) -> crate::data::message_cache::IncomingMessage {
    let Arrival {
        raw,
        folder_id,
        uid,
        uidl,
        in_junk_folder,
        look_at_the_body,
        at,
    } = *arrival;
    let addresses = |list: &[crate::common::types::EmailAddress]| {
        list.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    };

    crate::data::message_cache::IncomingMessage {
        folder_id,
        uid,
        message_id: parsed.message_id.clone().unwrap_or_default(),
        subject: parsed.subject.clone(),
        from_addr: addresses(&parsed.from),
        to_addr: addresses(&parsed.to),
        cc: Some(addresses(&parsed.cc)).filter(|cc| !cc.is_empty()),
        reply_to: Some(addresses(&parsed.reply_to)).filter(|to| !to.is_empty()),
        // The sender's own date when it is readable. POP3 has no equivalent of
        // the receipt time a server keeps, so when it is not, the time this
        // computer got the message stands in: a blank sorts to the far end of a
        // newest-first list, which is an end its reader will not look at.
        date: parsed.date.clone().unwrap_or_else(|| at.to_rfc3339()),
        internal_date: None,
        // What this computer actually holds, rather than the figure the server
        // listed. The two agree on a sane server and the downloaded length is
        // the one that is true here whatever the server said.
        size_bytes: Some(i64::try_from(raw.len()).unwrap_or(i64::MAX)),
        refs_header: reference_chain(parsed),
        // POP3 has no flags. Everything downloaded is new, which is true: this
        // is the first time this computer has seen it.
        read: false,
        starred: false,
        answered: false,
        draft: false,
        deleted: false,
        has_attachments: !parsed.attachments.is_empty(),
        // Three sources, worst winning. The whole message is already in hand
        // here, so the third one costs nothing extra: an IMAP account has had
        // this reading since the body fetch was written and a POP account had
        // none of it, with nothing saying which account you were on.
        safety: crate::service::safety::from_headers(&String::from_utf8_lossy(header_block(raw)))
            .and(crate::service::safety::from_folder(in_junk_folder))
            .and(if look_at_the_body {
                crate::application::body_safety::from_body(
                    &addresses(&parsed.from),
                    &parsed.subject,
                    parsed.body_plain.as_deref(),
                    parsed.body_html.as_deref(),
                )
            } else {
                crate::service::safety::Verdict::ordinary()
            }),
        gmail_message_id: None,
        labels: None,
        receipt_to: parsed.receipt_to.clone(),
        list_unsubscribe: parsed.list_unsubscribe.clone(),
        pop_uidl: Some(uidl.to_string()),
    }
}

/// Everything a message says before its first blank line.
///
/// The part servers wrote, and the only part a verdict may be read from. The
/// reader below treats any line shaped like a name and a value as a field, so
/// handing it a whole message would let somebody quoting a header in what they
/// wrote decide how their own message is marked.
///
/// Both line endings are looked for, and the earlier one wins: the wire uses
/// the pair, and mail that has been through something that rewrote its endings
/// arrives with the single one.
fn header_block(raw: &[u8]) -> &[u8] {
    const WIRE_BREAK: &[u8] = b"\r\n\r\n";
    const BARE_BREAK: &[u8] = b"\n\n";

    let first_of = |break_bytes: &[u8]| {
        raw.windows(break_bytes.len())
            .position(|window| window == break_bytes)
    };

    match [first_of(WIRE_BREAK), first_of(BARE_BREAK)]
        .into_iter()
        .flatten()
        .min()
    {
        Some(end) => &raw[..end],
        // No blank line at all, so the whole of it is headers or nothing is.
        None => raw,
    }
}

/// The whole ancestry a message names, as one stored string.
fn reference_chain(parsed: &crate::service::mime::ParsedMessage) -> Option<String> {
    let mut chain: Vec<&str> = parsed.references.iter().map(String::as_str).collect();
    if let Some(parent) = parsed.in_reply_to.as_deref()
        && !chain.contains(&parent)
    {
        chain.push(parent);
    }
    if chain.is_empty() {
        return None;
    }
    Some(chain.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::common::types::EmailAddress;
    use crate::data::message_cache::{CachedFolder, MessageCache};
    use chrono::{Duration, Utc};
    use std::collections::{HashMap, HashSet};

    use crate::application::mail_controller::Pop3MessagePreview;

    /// What a sync asked of the server, in the order it asked.
    ///
    /// Kept because the order is the guarantee: mail leaves a POP server for
    /// good, so anything marked for deletion before its copy is written down
    /// here is mail somebody can lose, and only the order shows that.
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Asked {
        Listed,
        Retrieved(u32),
        MarkedForDeletion(u32),
        Finished,
    }

    /// A POP server that answers from a script rather than a socket.
    #[derive(Default)]
    struct Scripted {
        on_server: Vec<Pop3MessagePreview>,
        bodies: HashMap<u32, Vec<u8>>,
        /// Which message number the connection drops on, if any.
        fails_on: Option<u32>,
        /// Whether the Allow Changes setting holds every removal back.
        ///
        /// What a real POP session does when the account may only be read: the
        /// refusal is raised before anything is sent, so the server hears
        /// nothing and the mail is still there.
        removals_held_by_the_setting: bool,
        asked: std::cell::RefCell<Vec<Asked>>,
    }

    impl Scripted {
        /// A server holding these messages, in this order, with these bodies.
        fn holding(messages: &[(u32, &str, &[u8])]) -> Self {
            Self {
                on_server: messages
                    .iter()
                    .map(|(id, uidl, raw)| Pop3MessagePreview {
                        id: *id,
                        size: raw.len(),
                        uidl: (*uidl).to_string(),
                    })
                    .collect(),
                bodies: messages
                    .iter()
                    .map(|(id, _, raw)| (*id, (*raw).to_vec()))
                    .collect(),
                ..Default::default()
            }
        }

        fn journal(&self) -> Vec<Asked> {
            self.asked.borrow().clone()
        }
    }

    impl PopMailbox for Scripted {
        async fn list(&self) -> Result<Vec<Pop3MessagePreview>> {
            self.asked.borrow_mut().push(Asked::Listed);
            Ok(self.on_server.clone())
        }

        async fn retrieve(&self, id: u32) -> Result<Vec<u8>> {
            self.asked.borrow_mut().push(Asked::Retrieved(id));
            if self.fails_on == Some(id) {
                return Err(crate::common::Error::Protocol(
                    "the connection dropped".into(),
                ));
            }
            self.bodies
                .get(&id)
                .cloned()
                .ok_or_else(|| crate::common::Error::Protocol("no such message".into()))
        }

        async fn mark_for_deletion(&self, id: u32) -> Result<()> {
            if self.removals_held_by_the_setting {
                return Err(crate::common::Error::Security(
                    crate::service::outward::refusal("remove a message from the mail server"),
                ));
            }
            self.asked.borrow_mut().push(Asked::MarkedForDeletion(id));
            Ok(())
        }

        async fn finish(&self) -> Result<()> {
            self.asked.borrow_mut().push(Asked::Finished);
            Ok(())
        }
    }

    /// A message as it comes off the wire.
    fn raw_message(headers: &str, body: &str) -> Vec<u8> {
        format!("{headers}\r\n\r\n{body}").into_bytes()
    }

    /// An empty cache in its own directory, and the inbox inside it.
    fn a_cache() -> (TempHome<MessageCache>, i64) {
        let cache = TempHome::named("wixen_pop_", |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache")
        });
        let folder_id = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acct".into(),
                name: "Inbox".into(),
                path: "INBOX".into(),
                folder_type: "Inbox".into(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder");
        (cache, folder_id)
    }

    fn run<M: PopMailbox>(
        server: &M,
        cache: &MessageCache,
        folder_id: i64,
        housekeeping: Housekeeping,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<PopSync> {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
            .block_on(sync(
                server,
                &Landing {
                    cache,
                    account_id: "acct",
                    folder_id,
                },
                housekeeping,
                false,
                true,
                now,
                None,
            ))
    }

    #[test]
    fn test_mail_collected_over_pop_is_sorted_by_the_rules() {
        // Nothing ran the rules on POP mail at all. The Rules Manager has no
        // protocol gate, so somebody collecting mail this way could write a
        // rule, name it, switch it on, and never have it evaluated, while the
        // changelog said rules run on arriving mail with no exception written
        // down anywhere.
        let (cache, folder_id) = a_cache();
        let raw = raw_message(
            "From: news@example.com
Subject: Weekly roundup",
            "Body",
        );
        let server = Scripted::holding(&[(1, "aaa", &raw)]);

        let mut engine = crate::application::filters::FilterEngine::default();
        engine.load_from_persisted(&[crate::data::message_cache::MessageFilterRule {
            id: "r1".into(),
            account_id: "acct".into(),
            name: "Newsletters are read".into(),
            field: "from".into(),
            match_type: "contains".into(),
            pattern: "news@example.com".into(),
            case_sensitive: false,
            action_type: "mark_as_read".into(),
            action_value: None,
            enabled: true,
            created_at: String::new(),
        }]);
        let filtering = crate::application::mail_sync::Filtering {
            rules: &engine,
            allowed: crate::application::allowed::Allowed {
                mail: true,
                personal_information: true,
                reading: true,
            },
        };

        let outcome = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
            .block_on(sync(
                &server,
                &Landing {
                    cache: &cache,
                    account_id: "acct",
                    folder_id,
                },
                Housekeeping::CAUTIOUS,
                false,
                true,
                Utc::now(),
                Some(&filtering),
            ))
            .expect("the sync to finish");

        assert_eq!(outcome.fetched, 1);
        assert_eq!(
            outcome.filtered.changed, 1,
            "a rule that matches the arriving message did not run"
        );
    }

    /// Another of this account's folders. Every folder a POP account has lives
    /// on this computer, so they all carry the reserved prefix.
    fn a_folder_here(cache: &MessageCache, name: &str) -> i64 {
        cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acct".into(),
                name: name.into(),
                path: format!("{}/{name}", crate::application::local_folders::LOCAL_PREFIX),
                folder_type: "Custom".into(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder")
    }

    /// A rule as the Rules Manager stores one.
    fn a_rule(
        name: &str,
        pattern: &str,
        action: &str,
        value: Option<&str>,
    ) -> crate::data::message_cache::MessageFilterRule {
        crate::data::message_cache::MessageFilterRule {
            id: format!("rule-{name}"),
            account_id: "acct".into(),
            name: name.into(),
            field: "from".into(),
            match_type: "contains".into(),
            pattern: pattern.into(),
            case_sensitive: false,
            action_type: action.into(),
            action_value: value.map(str::to_string),
            enabled: true,
            created_at: String::new(),
        }
    }

    /// A check with these rules on, and everything they ask for allowed.
    fn run_with_rules<M: PopMailbox>(
        server: &M,
        cache: &MessageCache,
        folder_id: i64,
        rules: &[crate::data::message_cache::MessageFilterRule],
    ) -> Result<PopSync> {
        let mut engine = crate::application::filters::FilterEngine::default();
        engine.load_from_persisted(rules);
        let filtering = crate::application::mail_sync::Filtering {
            rules: &engine,
            allowed: crate::application::allowed::Allowed {
                mail: true,
                personal_information: true,
                reading: true,
            },
        };
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
            .block_on(sync(
                server,
                &Landing {
                    cache,
                    account_id: "acct",
                    folder_id,
                },
                Housekeeping::CAUTIOUS,
                false,
                true,
                Utc::now(),
                Some(&filtering),
            ))
    }

    #[test]
    fn test_a_rule_that_files_mail_into_a_folder_really_files_it() {
        // Blocking a sender writes exactly this rule and nothing else. Here
        // the rules ran, the filing was worked out, and the answer was thrown
        // away: the message stayed in the inbox, nothing was said, and the
        // rule read as one that was working.
        let (cache, inbox) = a_cache();
        let junk = a_folder_here(&cache, "Junk");
        let raw = raw_message("From: news@example.com\r\nSubject: Weekly roundup", "Body");

        let done = run_with_rules(
            &Scripted::holding(&[(1, "aaa", &raw)]),
            &cache,
            inbox,
            &[a_rule(
                "Blocked sender",
                "news@example.com",
                "move_to_folder",
                Some("Junk"),
            )],
        )
        .expect("the check runs");

        assert_eq!(
            cache
                .get_message_list(junk, "acct")
                .expect("the list")
                .len(),
            1,
            "the rule filed nothing"
        );
        assert!(
            cache
                .get_message_list(inbox, "acct")
                .expect("the list")
                .is_empty(),
            "the message is in the inbox as well, so it is in two places"
        );
        assert_eq!(done.filtered.changed, 1, "the filing was not counted");
        assert!(
            done.filtered.could_not_be_filed.is_empty(),
            "{:?}",
            done.filtered.could_not_be_filed
        );
    }

    #[test]
    fn test_a_rule_that_could_not_file_mail_says_so_once_however_many_it_matched() {
        // A rule naming a folder somebody has since renamed fails the same way
        // on every message it matches, so a check that brought down three of
        // them holds three copies of one sentence. Kept one per message, so
        // the count in front of them is right, and folded to a single reading
        // when it is said, which is the rule about feedback that does not
        // flood a reader.
        let (cache, inbox) = a_cache();
        let raw = raw_message("From: news@example.com\r\nSubject: Weekly roundup", "Body");
        let mailbox = [
            (1, "aaa", raw.as_slice()),
            (2, "bbb", raw.as_slice()),
            (3, "ccc", raw.as_slice()),
        ];

        let done = run_with_rules(
            &Scripted::holding(&mailbox),
            &cache,
            inbox,
            &[a_rule(
                "Old folder",
                "news@example.com",
                "move_to_folder",
                Some("Receipts"),
            )],
        )
        .expect("the check runs");

        assert_eq!(
            done.filtered.could_not_be_filed.len(),
            3,
            "one sentence per message is what the count is read from: {:?}",
            done.filtered.could_not_be_filed
        );
        assert_eq!(
            done.filtered.changed, 0,
            "filing that never happened was counted as done"
        );
        assert_eq!(
            cache
                .get_message_list(inbox, "acct")
                .expect("the list")
                .len(),
            3,
            "mail went missing over a rule that could not be carried out"
        );

        let said = what_the_pop_check_did(&done);
        assert!(
            said.contains("3 messages not filed as asked"),
            "the check did not say how many were left where they were: {said}"
        );
        assert_eq!(
            said.matches("Receipts").count(),
            1,
            "one broken rule was read out once per message it matched: {said}"
        );
    }

    #[test]
    fn test_a_check_says_how_much_the_rules_sorted() {
        // The other direction, so the test above cannot be satisfied by a
        // summary that says nothing at all. Filing that worked is worth a word
        // too: mail moving out of the inbox on its own is otherwise a folder
        // that quietly holds less than the reader expects.
        let (cache, inbox) = a_cache();
        a_folder_here(&cache, "Junk");
        let raw = raw_message("From: news@example.com\r\nSubject: Weekly roundup", "Body");

        let done = run_with_rules(
            &Scripted::holding(&[(1, "aaa", &raw)]),
            &cache,
            inbox,
            &[a_rule(
                "Blocked sender",
                "news@example.com",
                "move_to_folder",
                Some("Junk"),
            )],
        )
        .expect("the check runs");

        let said = what_the_pop_check_did(&done);
        assert!(
            said.contains("1 new, 1 on the server"),
            "the check stopped saying what it downloaded: {said}"
        );
        assert!(
            said.contains("1 sorted by your rules"),
            "the check said nothing about the filing it did: {said}"
        );
        assert!(
            !said.contains("not filed as asked"),
            "a check where everything worked reported a failure: {said}"
        );
    }

    fn server() -> Vec<(u32, String)> {
        vec![
            (1, "aaa".to_string()),
            (2, "bbb".to_string()),
            (3, "ccc".to_string()),
        ]
    }

    #[test]
    fn test_only_what_is_not_already_here_is_downloaded() {
        let have: HashSet<String> = ["aaa".to_string(), "ccc".to_string()].into_iter().collect();

        let listed = server();
        let wanted = to_fetch(&listed, &have);

        assert_eq!(wanted.len(), 1);
        assert_eq!(wanted[0].1, "bbb");
    }

    #[test]
    fn test_a_first_sync_downloads_everything() {
        assert_eq!(to_fetch(&server(), &HashSet::new()).len(), 3);
    }

    #[test]
    fn test_a_message_is_matched_by_identifier_rather_than_number() {
        // The whole reason UIDL exists. Numbers shift as messages are deleted,
        // so the same number means a different message in the next session, and
        // anything keyed on them downloads mail twice or skips it.
        let renumbered = vec![(1, "ccc".to_string()), (2, "ddd".to_string())];
        let have: HashSet<String> = ["ccc".to_string()].into_iter().collect();

        let wanted = to_fetch(&renumbered, &have);

        assert_eq!(wanted.len(), 1);
        assert_eq!(wanted[0].1, "ddd", "matched on the identifier, not on 1");
    }

    #[test]
    fn test_nothing_is_removed_when_mail_is_left_on_the_server() {
        // The default, and the answer that cannot lose somebody's only copy.
        let downloaded = downloaded_days_ago(&["aaa", "bbb", "ccc"], 400);

        assert!(to_remove(&server(), &downloaded, Housekeeping::CAUTIOUS, Utc::now()).is_empty());
    }

    #[test]
    fn test_nothing_is_removed_when_no_number_of_days_was_given() {
        // Turning off "leave on server" without saying when is not an
        // instruction to delete immediately.
        let downloaded = downloaded_days_ago(&["aaa", "bbb", "ccc"], 400);
        let no_days = Housekeeping {
            leave_on_server: false,
            remove_after_days: 0,
        };

        assert!(to_remove(&server(), &downloaded, no_days, Utc::now()).is_empty());
    }

    #[test]
    fn test_mail_kept_long_enough_is_removed() {
        let downloaded = downloaded_days_ago(&["aaa", "bbb", "ccc"], 30);
        let after_a_fortnight = Housekeeping {
            leave_on_server: false,
            remove_after_days: 14,
        };

        let listed = server();
        let going = to_remove(&listed, &downloaded, after_a_fortnight, Utc::now());

        assert_eq!(going.len(), 3);
    }

    #[test]
    fn test_mail_not_yet_old_enough_stays() {
        let downloaded = downloaded_days_ago(&["aaa", "bbb", "ccc"], 3);
        let after_a_fortnight = Housekeeping {
            leave_on_server: false,
            remove_after_days: 14,
        };

        assert!(to_remove(&server(), &downloaded, after_a_fortnight, Utc::now()).is_empty());
    }

    #[test]
    fn test_mail_this_computer_never_downloaded_is_never_removed() {
        // The dangerous one. Removing something never downloaded deletes it
        // having never been read, and POP3 has no trash to get it back from.
        let downloaded = downloaded_days_ago(&["aaa"], 400);
        let aggressive = Housekeeping {
            leave_on_server: false,
            remove_after_days: 1,
        };

        let listed = server();
        let going = to_remove(&listed, &downloaded, aggressive, Utc::now());

        assert_eq!(going.len(), 1);
        assert_eq!(going[0].1, "aaa", "only the one actually held");
    }

    /// An arrival into folder 7, for tests about what a row carries.
    fn arrival(raw: &[u8]) -> Arrival<'_> {
        Arrival {
            raw,
            folder_id: 7,
            uid: 1,
            uidl: "aaa",
            in_junk_folder: false,
            // Off in the shared helper, so a test about anything else is not
            // quietly also a test about the reading of the message. The tests
            // that are about it switch it on by name.
            look_at_the_body: false,
            at: one_oclock(),
        }
    }

    /// A downloaded message with nothing optional set on it.
    fn plain() -> crate::service::mime::ParsedMessage {
        crate::service::mime::ParsedMessage {
            subject: "Notes on the engine".to_string(),
            from: vec![EmailAddress::new(
                "ada@example.com".to_string(),
                Some("Ada Lovelace".to_string()),
            )],
            to: vec![EmailAddress::new("me@example.com".to_string(), None)],
            date: Some("2026-07-20T10:00:00+00:00".to_string()),
            message_id: Some("note-1@example.com".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_a_downloaded_message_with_no_copy_recipients_stores_nothing_rather_than_an_empty_one() {
        // An empty string in the column is not the same as no column. It reads
        // back as a recipient with no address, which a list announces as a
        // person nobody can see and a reply addresses to nowhere.
        let none = to_incoming(&plain(), &arrival(b""));

        assert_eq!(none.cc, None);
        assert_eq!(none.reply_to, None);

        let copied = crate::service::mime::ParsedMessage {
            cc: vec![EmailAddress::new("bob@example.com".to_string(), None)],
            reply_to: vec![EmailAddress::new("list@example.com".to_string(), None)],
            ..plain()
        };
        let stored = to_incoming(&copied, &arrival(b""));

        assert_eq!(stored.cc.as_deref(), Some("bob@example.com"));
        assert_eq!(stored.reply_to.as_deref(), Some("list@example.com"));
    }

    #[test]
    fn test_a_downloaded_message_says_whether_it_carries_an_attachment() {
        // Both ways round. A message announced as carrying a file that has
        // none wastes the reader's time; one carrying a file and saying
        // nothing hides it entirely.
        assert!(!to_incoming(&plain(), &arrival(b"")).has_attachments);

        let carrying = crate::service::mime::ParsedMessage {
            attachments: vec![crate::service::mime::AttachmentInfo {
                filename: Some("figures.pdf".to_string()),
                mime_type: "application/pdf".to_string(),
                size: 1024,
                description: crate::service::mime::WhatTheSenderSaid::Nothing,
                content_id: None,
            }],
            ..plain()
        };

        assert!(to_incoming(&carrying, &arrival(b"")).has_attachments);
    }

    #[test]
    fn test_a_downloaded_reply_keeps_the_whole_ancestry_it_names() {
        // Threading reads this column and nothing else. A chain that loses a
        // name makes the reply a conversation of one, sitting on its own away
        // from the exchange it belongs to.
        let reply = crate::service::mime::ParsedMessage {
            references: vec!["first@example.com".to_string()],
            in_reply_to: Some("second@example.com".to_string()),
            ..plain()
        };

        assert_eq!(
            reference_chain(&reply).as_deref(),
            Some("first@example.com second@example.com")
        );
    }

    #[test]
    fn test_a_downloaded_message_starting_a_conversation_names_nobody() {
        assert_eq!(reference_chain(&plain()), None);
    }

    #[test]
    fn test_a_parent_already_named_in_the_chain_is_not_repeated() {
        // Most senders write the parent in both headers. Writing it twice
        // would put the same name in the ancestry of every reply in a long
        // exchange, once per hop.
        let reply = crate::service::mime::ParsedMessage {
            references: vec!["first@example.com".to_string()],
            in_reply_to: Some("first@example.com".to_string()),
            ..plain()
        };

        assert_eq!(
            reference_chain(&reply).as_deref(),
            Some("first@example.com")
        );
    }

    /// A downloaded message pretending to come from somebody it does not.
    fn pretending() -> crate::service::mime::ParsedMessage {
        crate::service::mime::ParsedMessage {
            subject: "Urgent: your account is suspended".to_string(),
            from: vec![EmailAddress::new(
                "noreply@paypa1.example".to_string(),
                Some("Security".to_string()),
            )],
            body_plain: Some("Reply to this email with your details.".to_string()),
            body_html: Some(
                "<p>Visit <a href=\"http://192.0.2.7/collect\">https://yourbank.example</a></p>"
                    .to_string(),
            ),
            ..plain()
        }
    }

    /// The same arrival, with the reading of the message switched on or off.
    fn arrival_reading(raw: &[u8], look_at_the_body: bool) -> Arrival<'_> {
        Arrival {
            look_at_the_body,
            ..arrival(raw)
        }
    }

    #[test]
    fn test_a_downloaded_message_carrying_a_deceptive_link_is_marked_suspicious() {
        // Mail collected this way had no reading of its own contents at all.
        // The same message arriving on an IMAP account was marked and on a POP
        // account was silent, and nothing said which account you were on.
        let stored = to_incoming(&pretending(), &arrival_reading(b"", true));

        assert_eq!(
            stored.safety.level,
            crate::service::safety::Safety::Suspicious
        );
        assert!(!stored.safety.reasons.is_empty(), "marked with no reason");
    }

    #[test]
    fn test_an_ordinary_downloaded_message_is_still_left_alone() {
        // A mailbox where every row announces a warning is a mailbox where
        // nobody hears the one that mattered.
        let ordinary = crate::service::mime::ParsedMessage {
            body_plain: Some("The numbers are attached. See you Thursday.".to_string()),
            ..plain()
        };

        assert_eq!(
            to_incoming(&ordinary, &arrival_reading(b"", true))
                .safety
                .level,
            crate::service::safety::Safety::Ordinary
        );
    }

    #[test]
    fn test_the_reading_of_the_message_can_be_switched_off() {
        // With it off the verdict is exactly what the headers said and nothing
        // more, which is what this path did before.
        let stored = to_incoming(&pretending(), &arrival_reading(b"", false));

        assert_eq!(
            stored.safety.level,
            crate::service::safety::Safety::Ordinary
        );
        assert!(stored.safety.reasons.is_empty());
    }

    #[test]
    fn test_the_reading_of_the_message_does_not_overrule_the_provider() {
        // Two sources, worst winning, and both reasons kept. A message can be
        // both in a filter's junk pile and carrying a link that lies about
        // where it goes, and losing either half loses why it was marked.
        let headers = b"Subject: Your account is suspended\r\n\
X-Spam-Flag: YES\r\n\r\nbody";

        let stored = to_incoming(&pretending(), &arrival_reading(headers, true));

        assert_eq!(stored.safety.level, crate::service::safety::Safety::Spam);
        assert!(
            stored.safety.reasons.len() > 1,
            "one of the two sources lost its reason: {:?}",
            stored.safety.reasons
        );
    }

    #[test]
    fn test_the_formatted_part_of_a_downloaded_message_is_read() {
        // Which half carries the trouble is the sender's choice, so a field
        // that is built and never passed on is a whole class of message that
        // arrives with nothing said about it.
        let formatted_only = crate::service::mime::ParsedMessage {
            body_plain: None,
            ..pretending()
        };

        assert_eq!(
            to_incoming(&formatted_only, &arrival_reading(b"", true))
                .safety
                .level,
            crate::service::safety::Safety::Suspicious
        );
    }

    #[test]
    fn test_the_plain_part_of_a_downloaded_message_is_read() {
        let plain_only = crate::service::mime::ParsedMessage {
            body_plain: Some(
                "Reply to this email. Go to http://192.0.2.7/collect now.".to_string(),
            ),
            body_html: None,
            ..pretending()
        };

        assert_eq!(
            to_incoming(&plain_only, &arrival_reading(b"", true))
                .safety
                .level,
            crate::service::safety::Safety::Suspicious
        );
    }

    #[test]
    fn test_who_the_message_says_it_is_from_is_read() {
        // The sender is one of the four things the reading is given, and the
        // one whose absence is hardest to notice: the message still gets a
        // verdict, just never this one.
        let body = "Reply to this email with your password. \
                    It is urgent. Go to http://192.0.2.7/collect";
        let from_a_do_not_reply = crate::service::mime::ParsedMessage {
            subject: "Immediate action required".to_string(),
            from: vec![EmailAddress::new(
                "noreply@example.com".to_string(),
                Some("Support".to_string()),
            )],
            body_plain: Some(body.to_string()),
            body_html: None,
            ..plain()
        };
        let from_a_person = crate::service::mime::ParsedMessage {
            from: vec![EmailAddress::new(
                "ada@example.com".to_string(),
                Some("Ada Lovelace".to_string()),
            )],
            ..from_a_do_not_reply.clone()
        };

        let unsigned = to_incoming(&from_a_do_not_reply, &arrival_reading(b"", true));
        let signed = to_incoming(&from_a_person, &arrival_reading(b"", true));

        assert_ne!(
            unsigned.safety.reasons, signed.safety.reasons,
            "changing who it came from changed nothing, so the sender is not being read"
        );
    }

    #[test]
    fn test_the_subject_of_a_downloaded_message_is_read() {
        let pressure_in_the_subject = crate::service::mime::ParsedMessage {
            subject: "Urgent: wire transfer".to_string(),
            body_plain: Some("Go to http://192.0.2.7/collect".to_string()),
            body_html: None,
            ..plain()
        };
        let no_pressure = crate::service::mime::ParsedMessage {
            subject: "Notes".to_string(),
            ..pressure_in_the_subject.clone()
        };

        assert_ne!(
            to_incoming(&pressure_in_the_subject, &arrival_reading(b"", true))
                .safety
                .level,
            to_incoming(&no_pressure, &arrival_reading(b"", true))
                .safety
                .level,
            "changing the subject changed nothing, so the subject is not being read"
        );
    }

    #[test]
    fn test_a_message_with_no_body_stored_is_not_marked_for_having_none() {
        // A warning on every message that happens to be empty is a warning
        // nobody reads by the second one.
        let empty = crate::service::mime::ParsedMessage {
            body_plain: None,
            body_html: None,
            ..plain()
        };
        let blank = crate::service::mime::ParsedMessage {
            body_plain: Some(String::new()),
            body_html: Some(String::new()),
            ..plain()
        };

        for message in [empty, blank] {
            assert_eq!(
                to_incoming(&message, &arrival_reading(b"", true))
                    .safety
                    .level,
                crate::service::safety::Safety::Ordinary
            );
        }
    }

    #[test]
    fn test_a_message_repeating_itself_in_both_halves_reads_the_same_as_one() {
        // Some senders put identical text in both parts. Reading it twice must
        // not make one message look worse than the same message sent once.
        let text = "Reply to this email. http://192.0.2.7/collect";
        let once = crate::service::mime::ParsedMessage {
            body_plain: Some(text.to_string()),
            body_html: None,
            ..pretending()
        };
        let twice = crate::service::mime::ParsedMessage {
            body_html: Some(text.to_string()),
            ..once.clone()
        };

        assert_eq!(
            to_incoming(&once, &arrival_reading(b"", true)).safety,
            to_incoming(&twice, &arrival_reading(b"", true)).safety
        );
    }

    #[test]
    fn test_a_header_quoted_in_the_body_still_decides_nothing() {
        // The rule the header reader already holds, restated now that the body
        // is read too: what somebody wrote must not be able to set the verdict
        // a server gave, in either direction.
        let raw = raw_message(
            "Subject: What that message was\r\nFrom: colleague@example.com",
            "The one you forwarded had this on it:\r\nX-Spam-Flag: YES\r\nso I binned it.",
        );
        let quoting = crate::service::mime::ParsedMessage {
            body_plain: Some(
                "The one you forwarded had this on it:\r\nX-Spam-Flag: YES".to_string(),
            ),
            ..plain()
        };

        assert_eq!(
            to_incoming(&quoting, &arrival_reading(&raw, true))
                .safety
                .level,
            crate::service::safety::Safety::Ordinary
        );
    }

    #[test]
    fn test_a_check_says_which_rows_it_wrote() {
        // What the link check needs to reach POP mail at all. Without it there
        // is no way to say which of the folder's messages are the new ones, so
        // either every message is looked at again on every check or none is.
        let raw = raw_message("Subject: One\r\nFrom: ada@example.com", "Text.");
        let (cache, folder_id) = a_cache();

        let first = run(
            &Scripted::holding(&[(1, "aaa", &raw), (2, "bbb", &raw)]),
            &cache,
            folder_id,
            Housekeeping::CAUTIOUS,
            Utc::now(),
        )
        .expect("the first check runs");

        assert_eq!(first.written.len(), 2);
        for row in &first.written {
            assert!(
                cache.get_message(*row).expect("the lookup").is_some(),
                "a row was reported that is not there"
            );
        }
    }

    /// A message that really is signed with a certificate, as it comes off the
    /// wire.
    fn a_signed_message() -> Vec<u8> {
        crate::service::signed_mail::for_tests::signed_beside()
    }

    #[test]
    fn test_a_signed_message_collected_over_pop_can_still_have_its_signature_checked() {
        // POP matters more here than anywhere else. Once mail has been
        // collected there is no asking the server for it again, and the copy
        // this program keeps is the parsed text, which a signature cannot be
        // checked against. Without the bytes, a signature collected this way is
        // one nobody could ever check, and the message would read as mail that
        // never claimed one.
        //
        // Run through the whole check rather than by calling the cache, because
        // what is being asked is whether the running program does it.
        let raw = a_signed_message();
        let (cache, folder_id) = a_cache();

        let done = run(
            &Scripted::holding(&[(1, "aaa", &raw)]),
            &cache,
            folder_id,
            Housekeeping::CAUTIOUS,
            Utc::now(),
        )
        .expect("the check runs");

        assert_eq!(done.written.len(), 1);
        assert_eq!(
            cache.signed_original(done.written[0]).expect("read back"),
            crate::data::message_cache::signed_original::SignedOriginal::Kept(raw)
        );
    }

    #[test]
    fn test_a_signed_message_whose_bytes_could_not_be_kept_is_still_a_check_that_worked() {
        // Keeping the bytes costs a verdict on one message when it fails, and
        // nothing else: the message itself is written down before this runs.
        // Reported as a failure it reads as "your mail did not arrive", and it
        // returns before the polite ending, which is the only thing that
        // commits anything on a POP server. The same rule the IMAP path
        // already follows for the same call.
        let raw = a_signed_message();
        let (cache, folder_id) = a_cache();
        crate::data::message_cache::signed_original::for_tests::stop_it_keeping_signed_originals(
            &cache,
        )
        .expect("a cache that can no longer keep them");
        let server = Scripted::holding(&[(1, "aaa", &raw)]);

        let done = run(
            &server,
            &cache,
            folder_id,
            Housekeeping::CAUTIOUS,
            Utc::now(),
        )
        .expect("a verdict that could not be kept is not a failed mail check");

        assert_eq!(done.fetched, 1);
        let journal = server.journal();
        assert!(
            journal.contains(&Asked::Finished),
            "the session was dropped instead of ended politely: {journal:?}"
        );
        assert_eq!(
            cache
                .get_message_list(folder_id, "acct")
                .expect("the list")
                .len(),
            1,
            "the mail went missing over a verdict that could not be kept"
        );
    }

    #[test]
    fn test_a_check_that_downloaded_nothing_reports_no_rows() {
        // Mail already here contributes none, so nothing is looked at twice.
        let raw = raw_message("Subject: One\r\nFrom: ada@example.com", "Text.");
        let (cache, folder_id) = a_cache();
        let two = [(1, "aaa", raw.as_slice()), (2, "bbb", raw.as_slice())];
        run(
            &Scripted::holding(&two),
            &cache,
            folder_id,
            Housekeeping::CAUTIOUS,
            Utc::now(),
        )
        .expect("the first check runs");

        let second = run(
            &Scripted::holding(&two),
            &cache,
            folder_id,
            Housekeeping::CAUTIOUS,
            Utc::now(),
        )
        .expect("the second check runs");

        assert!(second.written.is_empty(), "{:?}", second.written);
    }

    /// Housekeeping that clears the server after a fortnight.
    const AFTER_A_FORTNIGHT: Housekeeping = Housekeeping {
        leave_on_server: false,
        remove_after_days: 14,
    };

    #[test]
    fn test_nothing_is_deleted_from_the_server_until_everything_is_written_down() {
        // The whole guarantee this module is built on. POP3 has no trash, so a
        // message deleted before its copy is written down is gone from both
        // places, and only the order the server is asked in shows that.
        let raw = raw_message("Subject: One\r\nFrom: ada@example.com", "Text.");
        let (cache, folder_id) = a_cache();
        let first_run = Scripted::holding(&[(1, "aaa", &raw)]);
        run(
            &first_run,
            &cache,
            folder_id,
            Housekeeping::CAUTIOUS,
            Utc::now(),
        )
        .expect("the first check runs");

        let server = Scripted::holding(&[(1, "aaa", &raw), (2, "bbb", &raw), (3, "ccc", &raw)]);
        run(
            &server,
            &cache,
            folder_id,
            AFTER_A_FORTNIGHT,
            Utc::now() + Duration::days(40),
        )
        .expect("the second check runs");

        let journal = server.journal();
        let last_write = journal
            .iter()
            .rposition(|asked| matches!(asked, Asked::Retrieved(_)))
            .expect("nothing was downloaded at all");
        let first_delete = journal
            .iter()
            .position(|asked| matches!(asked, Asked::MarkedForDeletion(_)))
            .expect("nothing was removed at all");
        assert!(
            last_write < first_delete,
            "a message was marked to go before the download was finished: {journal:?}"
        );
        assert_eq!(
            journal.last(),
            Some(&Asked::Finished),
            "the session ended somewhere other than the end: {journal:?}"
        );
    }

    #[test]
    fn test_a_download_that_fails_partway_leaves_the_server_untouched() {
        // The interruption the order exists for. A connection that drops in the
        // middle must leave every message where it is, so the next check finds
        // them rather than finding a mailbox somebody's only copy left.
        let raw = raw_message("Subject: One\r\nFrom: ada@example.com", "Text.");
        let (cache, folder_id) = a_cache();
        let first_run = Scripted::holding(&[(1, "aaa", &raw)]);
        run(
            &first_run,
            &cache,
            folder_id,
            Housekeeping::CAUTIOUS,
            Utc::now(),
        )
        .expect("the first check runs");

        let server = Scripted {
            fails_on: Some(2),
            ..Scripted::holding(&[(1, "aaa", &raw), (2, "bbb", &raw)])
        };
        let outcome = run(
            &server,
            &cache,
            folder_id,
            AFTER_A_FORTNIGHT,
            Utc::now() + Duration::days(40),
        );

        assert!(outcome.is_err(), "a dropped connection reported as a check");
        let journal = server.journal();
        assert!(
            !journal
                .iter()
                .any(|asked| matches!(asked, Asked::MarkedForDeletion(_))),
            "mail was marked to go after the download failed: {journal:?}"
        );
        assert!(
            !journal.contains(&Asked::Finished),
            "the session was ended politely, which is what commits deletions: {journal:?}"
        );
        assert_eq!(
            cache
                .get_message_list(folder_id, "acct")
                .expect("the list")
                .len(),
            1,
            "the message downloaded before the failure was lost"
        );
    }

    #[test]
    fn test_mail_kept_long_enough_is_removed_from_the_server_and_kept_here() {
        // Both halves. Removing it from the server is what somebody asked for;
        // keeping the copy here is why they can ask for it at all.
        let raw = raw_message("Subject: One\r\nFrom: ada@example.com", "Text.");
        let (cache, folder_id) = a_cache();
        let first_run = Scripted::holding(&[(1, "aaa", &raw)]);
        run(
            &first_run,
            &cache,
            folder_id,
            Housekeeping::CAUTIOUS,
            Utc::now(),
        )
        .expect("the first check runs");

        let server = Scripted::holding(&[(1, "aaa", &raw)]);
        let done = run(
            &server,
            &cache,
            folder_id,
            AFTER_A_FORTNIGHT,
            Utc::now() + Duration::days(40),
        )
        .expect("the second check runs");

        assert_eq!(done.removed_from_server, 1);
        assert_eq!(
            server.journal(),
            vec![Asked::Listed, Asked::MarkedForDeletion(1), Asked::Finished],
            "the message was downloaded a second time or left on the server"
        );
        assert_eq!(
            cache
                .get_message_list(folder_id, "acct")
                .expect("the list")
                .len(),
            1,
            "the copy on this computer went with the one on the server"
        );
    }

    #[test]
    fn test_a_removal_held_by_allow_changes_is_counted_as_waiting_rather_than_failing() {
        // Everything was downloaded and written down correctly. The only thing
        // that did not happen is the clearing out, and that was the setting
        // doing its job. Reported as a failure it reads as "your mail did not
        // arrive", and the session is dropped before the polite ending, which
        // is the only thing that commits anything on a POP server.
        let raw = raw_message("Subject: One\r\nFrom: ada@example.com", "Text.");
        let (cache, folder_id) = a_cache();
        run(
            &Scripted::holding(&[(1, "aaa", &raw)]),
            &cache,
            folder_id,
            Housekeeping::CAUTIOUS,
            Utc::now(),
        )
        .expect("the first check runs");

        let server = Scripted {
            removals_held_by_the_setting: true,
            ..Scripted::holding(&[(1, "aaa", &raw)])
        };
        let done = run(
            &server,
            &cache,
            folder_id,
            AFTER_A_FORTNIGHT,
            Utc::now() + Duration::days(40),
        )
        .expect("a held-back removal is not a failed mail check");

        let journal = server.journal();
        assert_eq!(done.waiting_on_the_setting, 1);
        assert_eq!(done.removed_from_server, 0);
        assert!(
            journal.contains(&Asked::Finished),
            "the session was dropped instead of ended politely: {journal:?}"
        );
        assert_eq!(
            cache
                .get_message_list(folder_id, "acct")
                .expect("the list")
                .len(),
            1,
            "the copy on this computer went missing over a removal that never happened"
        );
    }

    #[test]
    fn test_an_ordinary_removal_is_still_counted_as_a_removal() {
        // The other direction, so the test above cannot pass by everything
        // being nought. A check where nothing was held back has to say so.
        let raw = raw_message("Subject: One\r\nFrom: ada@example.com", "Text.");
        let (cache, folder_id) = a_cache();
        run(
            &Scripted::holding(&[(1, "aaa", &raw)]),
            &cache,
            folder_id,
            Housekeeping::CAUTIOUS,
            Utc::now(),
        )
        .expect("the first check runs");

        let done = run(
            &Scripted::holding(&[(1, "aaa", &raw)]),
            &cache,
            folder_id,
            AFTER_A_FORTNIGHT,
            Utc::now() + Duration::days(40),
        )
        .expect("the second check runs");

        assert_eq!(done.removed_from_server, 1);
        assert_eq!(done.waiting_on_the_setting, 0);
    }

    #[test]
    fn test_a_second_check_brings_down_only_what_is_new() {
        // What somebody is told, and what they wait for. Counting downloads
        // again on every check reports mail that did not arrive and downloads a
        // mailbox once per check.
        let raw = raw_message("Subject: One\r\nFrom: ada@example.com", "Text.");
        let (cache, folder_id) = a_cache();
        let two = [(1, "aaa", raw.as_slice()), (2, "bbb", raw.as_slice())];

        let first = run(
            &Scripted::holding(&two),
            &cache,
            folder_id,
            Housekeeping::CAUTIOUS,
            Utc::now(),
        )
        .expect("the first check runs");
        assert_eq!(first.fetched, 2);

        let three = [
            (1, "aaa", raw.as_slice()),
            (2, "bbb", raw.as_slice()),
            (3, "ccc", raw.as_slice()),
        ];
        let second = run(
            &Scripted::holding(&three),
            &cache,
            folder_id,
            Housekeeping::CAUTIOUS,
            Utc::now(),
        )
        .expect("the second check runs");

        assert_eq!(second.fetched, 1, "mail already here was downloaded again");
        assert_eq!(second.on_server, 3);
        assert_eq!(
            cache
                .get_message_list(folder_id, "acct")
                .expect("the list")
                .len(),
            3,
            "a message was stored twice or lost"
        );
    }

    #[test]
    fn test_a_sync_with_no_connection_says_so_rather_than_reporting_an_empty_mailbox() {
        // "0 new, 0 on the server" reads as a mailbox with no mail in it. A
        // reader who is told that stops checking, and the mail is still there.
        let (cache, folder_id) = a_cache();
        let controller = crate::application::mail_controller::MailController::new();

        let outcome = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
            .block_on(sync(
                &controller,
                &Landing {
                    cache: &cache,
                    account_id: "acct",
                    folder_id,
                },
                Housekeeping::CAUTIOUS,
                false,
                true,
                Utc::now(),
                None,
            ));

        assert!(outcome.is_err(), "a failed sync reported as a done one");
    }

    #[test]
    fn test_pop_commands_with_no_connection_say_so_rather_than_answering() {
        // What the real server does with nothing connected, since every test
        // above this one answers from a script. Three of the four refuse.
        // Ending the session is the exception and must succeed: it is called on
        // every tidying-up path, including ones that failed before connecting,
        // and an error there would report a check as broken that never started.
        let controller = crate::application::mail_controller::MailController::new();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("a runtime");

        runtime.block_on(async {
            assert!(
                controller.list().await.is_err(),
                "listed with no connection"
            );
            assert!(
                controller.retrieve(1).await.is_err(),
                "handed back a message with no connection"
            );
            assert!(
                controller.mark_for_deletion(1).await.is_err(),
                "marked mail to go with no connection"
            );
            assert!(
                controller.finish().await.is_ok(),
                "tidying up refused when there was nothing to tidy"
            );
        });
    }

    /// A POP3 server on the loopback interface that answers everything with
    /// `+OK` and says afterwards which commands it was given.
    ///
    /// Only the verbs are kept. One of the lines it reads is the PASS command,
    /// and a recording that held the whole line would put a password into a
    /// test failure.
    async fn scripted_pop_server() -> (u16, tokio::sync::oneshot::Receiver<Vec<String>>) {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a loopback port");
        let port = listener.local_addr().expect("the port it was given").port();
        let (heard, was_heard) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            let Ok((socket, _)) = listener.accept().await else {
                return;
            };
            let mut stream = tokio::io::BufReader::new(socket);
            // Every POP3 connection opens with a greeting, and the client reads
            // it before sending anything.
            if stream.get_mut().write_all(b"+OK ready\r\n").await.is_err() {
                return;
            }

            let mut verbs: Vec<String> = Vec::new();
            let mut line = String::new();
            loop {
                line.clear();
                match stream.read_line(&mut line).await {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {}
                }
                let verb = line
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_ascii_uppercase();
                let ending = verb == "QUIT";
                verbs.push(verb);
                if stream.get_mut().write_all(b"+OK\r\n").await.is_err() {
                    break;
                }
                if ending {
                    break;
                }
            }
            let _ = heard.send(verbs);
        });

        (port, was_heard)
    }

    #[test]
    fn test_ending_a_pop_session_sends_quit_and_leaves_nothing_connected() {
        // The one method of the four that has to reach a real server to mean
        // anything. POP3 has no other kind of delete: DELE marks and QUIT
        // commits, so every deletion this module ordered happens here or not at
        // all. The test above it only asks what ending an unopened session
        // does, and answering "fine" to that is also what doing nothing looks
        // like.
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("a runtime");

        runtime.block_on(async {
            let (port, was_heard) = scripted_pop_server().await;
            let controller = crate::application::mail_controller::MailController::new();
            controller
                .connect_pop3(
                    "127.0.0.1".to_string(),
                    port,
                    "someone".to_string(),
                    "the-loopback-server-accepts-anything".to_string(),
                    false,
                    "acct",
                )
                .await
                .expect("the loopback server signs anybody in");
            assert!(
                controller.is_pop3_connected().await,
                "nothing was connected, so ending it proves nothing"
            );

            controller.finish().await.expect("the session ends");

            assert!(
                !controller.is_pop3_connected().await,
                "the session is still open after being ended"
            );
            let verbs = tokio::time::timeout(std::time::Duration::from_secs(5), was_heard)
                .await
                .expect("the server said what it heard inside five seconds")
                .expect("the server task finished");
            assert!(verbs.contains(&"QUIT".to_string()), "{verbs:?}");
        });
    }

    #[test]
    fn test_a_downloaded_message_records_how_big_it_is() {
        // The size column is read by the list and sorted on. Left blank, every
        // message from a POP account shows nothing there and heaps together at
        // one end when somebody sorts by size.
        let raw = raw_message(
            "Subject: Notes\r\nFrom: ada@example.com\r\nDate: Mon, 20 Jul 2026 10:00:00 +0000",
            "Some text.",
        );
        let server = Scripted::holding(&[(1, "aaa", &raw)]);
        let (cache, folder_id) = a_cache();

        run(
            &server,
            &cache,
            folder_id,
            Housekeeping::CAUTIOUS,
            Utc::now(),
        )
        .expect("the sync runs");

        let rows = cache.get_message_list(folder_id, "acct").expect("the list");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].size_bytes, Some(raw.len() as i64));
    }

    /// What the cache stored about how safe the one message in a folder is.
    fn stored_verdict(cache: &MessageCache, folder_id: i64) -> crate::service::safety::Verdict {
        let row = cache
            .message_row_for_uid(folder_id, 1)
            .expect("the lookup")
            .expect("a stored message");
        cache.message_safety(row).expect("its verdict")
    }

    #[test]
    fn test_a_downloaded_message_carries_the_verdict_its_headers_already_have() {
        // The sending and receiving servers both wrote down what they made of
        // this message. Mail from other accounts is marked with what they said;
        // mail collected this way was recorded as ordinary whatever it carried,
        // so the one warning that costs nothing to read never reached anybody.
        let raw = raw_message(
            "Subject: Your account is suspended\r\nFrom: security@paypa1.example\r\n\
             Date: Mon, 20 Jul 2026 10:00:00 +0000\r\nX-Spam-Flag: YES",
            "Click here.",
        );
        let server = Scripted::holding(&[(1, "aaa", &raw)]);
        let (cache, folder_id) = a_cache();

        run(
            &server,
            &cache,
            folder_id,
            Housekeeping::CAUTIOUS,
            Utc::now(),
        )
        .expect("the sync runs");

        let verdict = stored_verdict(&cache, folder_id);
        assert_eq!(verdict.level, crate::service::safety::Safety::Spam);
        assert!(!verdict.reasons.is_empty(), "marked with no reason given");
    }

    #[test]
    fn test_a_body_that_repeats_a_header_is_not_read_as_one() {
        // Somebody quoting a header in their own message must not be able to
        // decide the verdict on it. Only what is above the first blank line was
        // written by a server.
        let raw = raw_message(
            "Subject: What that message was\r\nFrom: colleague@example.com\r\n\
             Date: Mon, 20 Jul 2026 10:00:00 +0000",
            "The one you forwarded had this on it:\r\nX-Spam-Flag: YES\r\nso I binned it.",
        );
        let server = Scripted::holding(&[(1, "aaa", &raw)]);
        let (cache, folder_id) = a_cache();

        run(
            &server,
            &cache,
            folder_id,
            Housekeeping::CAUTIOUS,
            Utc::now(),
        )
        .expect("the sync runs");

        assert_eq!(
            stored_verdict(&cache, folder_id).level,
            crate::service::safety::Safety::Ordinary
        );
    }

    /// A fixed moment, so a test can say what "when it arrived" was.
    fn one_oclock() -> chrono::DateTime<Utc> {
        chrono::DateTime::parse_from_rfc3339("2026-07-25T13:00:00+00:00")
            .expect("a readable time")
            .into()
    }

    #[test]
    fn test_a_message_with_no_usable_date_is_dated_when_it_arrived() {
        // The list is ordered newest first and nothing sorts lower than a blank
        // date, so an undated message went to the far end of the mailbox. By
        // ear that is a walk to the bottom rather than a glance.
        let raw = raw_message(
            "Subject: No date on this one\r\nFrom: ada@example.com",
            "Hello.",
        );
        let server = Scripted::holding(&[(1, "aaa", &raw)]);
        let (cache, folder_id) = a_cache();

        run(
            &server,
            &cache,
            folder_id,
            Housekeeping::CAUTIOUS,
            one_oclock(),
        )
        .expect("the sync runs");

        let rows = cache.get_message_list(folder_id, "acct").expect("the list");
        let stored = chrono::DateTime::parse_from_rfc3339(&rows[0].date)
            .unwrap_or_else(|_| panic!("stored an unreadable date: {:?}", rows[0].date));
        assert_eq!(stored, one_oclock());
    }

    #[test]
    fn test_a_message_that_carries_its_own_date_keeps_it() {
        // The other half of the trade. Dating everything on arrival would throw
        // away the one date the sender actually chose.
        let raw = raw_message(
            "Subject: Notes\r\nFrom: ada@example.com\r\nDate: Mon, 20 Jul 2026 10:00:00 +0000",
            "Some text.",
        );
        let server = Scripted::holding(&[(1, "aaa", &raw)]);
        let (cache, folder_id) = a_cache();

        run(
            &server,
            &cache,
            folder_id,
            Housekeeping::CAUTIOUS,
            one_oclock(),
        )
        .expect("the sync runs");

        let rows = cache.get_message_list(folder_id, "acct").expect("the list");
        let stored = chrono::DateTime::parse_from_rfc3339(&rows[0].date).expect("a readable date");
        assert_eq!(
            stored,
            chrono::DateTime::parse_from_rfc3339("2026-07-20T10:00:00+00:00")
                .expect("the sender's")
        );
    }

    #[test]
    fn test_a_message_moved_out_of_the_inbox_is_not_downloaded_again() {
        // What deleting POP mail means. Moving a message to the trash takes it
        // out of the inbox, and a check that only looks in the inbox concludes
        // it never arrived, so the next check puts it straight back and nobody
        // can delete anything.
        let raw = raw_message("Subject: One\r\nFrom: ada@example.com", "Text.");
        let (cache, folder_id) = a_cache();
        run(
            &Scripted::holding(&[(1, "aaa", &raw)]),
            &cache,
            folder_id,
            Housekeeping::CAUTIOUS,
            Utc::now(),
        )
        .expect("the first check runs");
        let trash = cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: "acct".into(),
                name: "Trash".into(),
                path: "\u{1}Local/Trash".into(),
                folder_type: "Trash".into(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a trash folder");
        let row = cache
            .message_row_for_uid(folder_id, 1)
            .expect("the lookup")
            .expect("the downloaded message");
        cache.move_message(row, trash).expect("the move");

        let again = run(
            &Scripted::holding(&[(1, "aaa", &raw)]),
            &cache,
            folder_id,
            Housekeeping::CAUTIOUS,
            Utc::now(),
        )
        .expect("the second check runs");

        assert_eq!(
            again.fetched, 0,
            "mail that was moved out of the inbox was downloaded again"
        );
    }

    fn downloaded_days_ago(
        uidls: &[&str],
        days: i64,
    ) -> HashMap<String, chrono::DateTime<chrono::Utc>> {
        let when = Utc::now() - Duration::days(days);
        uidls
            .iter()
            .map(|uidl| ((*uidl).to_string(), when))
            .collect()
    }

    #[test]
    fn test_the_server_removal_setting_says_it_is_permanent_and_unlike_imap() {
        // What the account settings screen attaches to "Leave mail on the
        // server after downloading it" as its consequence. A screen reader
        // user hears the name and the checked state and nothing else without
        // this, and POP's removal is the one setting here with no undo: no
        // Trash folder on the server, and no other device to recover a copy
        // from once the days below have run out.
        let lowered = SERVER_REMOVAL_IS_PERMANENT.to_lowercase();
        assert!(
            lowered.contains("imap"),
            "does not contrast with IMAP's own delete, which has a Trash: {SERVER_REMOVAL_IS_PERMANENT}"
        );
        assert!(
            lowered.contains("trash"),
            "does not say POP has no Trash to recover mail from: {SERVER_REMOVAL_IS_PERMANENT}"
        );
        assert!(
            ["gone", "for good", "permanent"]
                .iter()
                .any(|word| lowered.contains(word)),
            "does not say the removal is permanent: {SERVER_REMOVAL_IS_PERMANENT}"
        );
        assert!(
            SERVER_REMOVAL_IS_PERMANENT.ends_with('.'),
            "read aloud, this needs to end as a sentence: {SERVER_REMOVAL_IS_PERMANENT}"
        );
        assert!(
            !SERVER_REMOVAL_IS_PERMANENT.contains("  "),
            "a doubled space is read as a pause in the middle of a sentence: {SERVER_REMOVAL_IS_PERMANENT}"
        );
    }
}

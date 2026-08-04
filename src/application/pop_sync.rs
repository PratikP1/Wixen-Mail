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

use crate::common::Result;

/// What a sync of a POP mailbox did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PopSync {
    pub fetched: usize,
    /// How many were removed from the server, having been kept long enough.
    pub removed_from_server: usize,
    /// How many are on the server in total.
    pub on_server: usize,
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
    cache: &crate::data::message_cache::MessageCache,
    folder_id: i64,
    housekeeping: Housekeeping,
    in_junk_folder: bool,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<PopSync> {
    let listing = server.list().await?;
    let on_server: Vec<(u32, String)> = listing
        .iter()
        .map(|message| (message.id, message.uidl.clone()))
        .collect();

    let already_have = cache.pop_uidls(folder_id)?;
    let wanted = to_fetch(&on_server, &already_have);

    let mut fetched = 0usize;
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
        fetched += 1;
    }

    let downloaded = cache.pop_download_times(folder_id)?;
    let stale = to_remove(&on_server, &downloaded, housekeeping, now);
    for (id, _) in &stale {
        server.mark_for_deletion(*id).await?;
    }
    // Committed here. Until this runs, every DELE is a mark the server throws
    // away if the connection drops.
    server.finish().await?;

    Ok(PopSync {
        fetched,
        removed_from_server: stale.len(),
        on_server: on_server.len(),
    })
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
        safety: crate::service::safety::from_headers(&String::from_utf8_lossy(header_block(raw)))
            .and(crate::service::safety::from_folder(in_junk_folder)),
        gmail_message_id: None,
        labels: None,
        receipt_to: parsed.receipt_to.clone(),
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
    fn a_cache() -> (MessageCache, i64) {
        let dir = std::env::temp_dir().join(format!("wixen_pop_{}", uuid::Uuid::new_v4()));
        let cache = MessageCache::new(dir, None).expect("a cache");
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
            .block_on(sync(server, cache, folder_id, housekeeping, false, now))
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
                &cache,
                folder_id,
                Housekeeping::CAUTIOUS,
                false,
                Utc::now(),
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
}

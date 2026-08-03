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

/// Bring a POP mailbox into a folder, and clear the server if asked.
///
/// The order matters. Everything is downloaded and written before anything is
/// deleted, so an interruption leaves mail on the server rather than gone from
/// both places. POP3 helps here: DELE only marks, and nothing is committed
/// until the session ends politely, so a dropped connection undoes the lot.
pub async fn sync(
    controller: &crate::application::mail_controller::MailController,
    cache: &crate::data::message_cache::MessageCache,
    folder_id: i64,
    housekeeping: Housekeeping,
    in_junk_folder: bool,
) -> Result<PopSync> {
    let listing = controller.list_pop3_messages().await?;
    let on_server: Vec<(u32, String)> = listing
        .iter()
        .map(|message| (message.id, message.uidl.clone()))
        .collect();

    let already_have = cache.pop_uidls(folder_id)?;
    let wanted = to_fetch(&on_server, &already_have);

    let mut fetched = 0usize;
    for (id, uidl) in &wanted {
        let raw = controller.fetch_pop3_message_body(*id).await?;
        let parsed = crate::service::mime::parse(&raw)?;
        let uid = cache.next_local_uid(folder_id)?;
        let row =
            cache.upsert_message(&to_incoming(&parsed, folder_id, uid, uidl, in_junk_folder))?;
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
    let stale = to_remove(&on_server, &downloaded, housekeeping, chrono::Utc::now());
    for (id, _) in &stale {
        controller.delete_pop3_message(*id).await?;
    }
    // Committed here. Until this runs, every DELE is a mark the server throws
    // away if the connection drops.
    controller.finish_pop3().await?;

    Ok(PopSync {
        fetched,
        removed_from_server: stale.len(),
        on_server: on_server.len(),
    })
}

/// Turn a downloaded message into the row the cache stores.
fn to_incoming(
    parsed: &crate::service::mime::ParsedMessage,
    folder_id: i64,
    uid: u32,
    uidl: &str,
    in_junk_folder: bool,
) -> crate::data::message_cache::IncomingMessage {
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
        // POP3 has no INTERNALDATE, so the sender's own date is all there is.
        // Empty rather than invented: a message sorted to a date nobody chose
        // is a message its reader will not find.
        date: parsed.date.clone().unwrap_or_default(),
        internal_date: None,
        size_bytes: None,
        refs_header: reference_chain(parsed),
        // POP3 has no flags. Everything downloaded is new, which is true: this
        // is the first time this computer has seen it.
        read: false,
        starred: false,
        answered: false,
        draft: false,
        deleted: false,
        has_attachments: !parsed.attachments.is_empty(),
        safety: crate::service::safety::from_folder(in_junk_folder),
        gmail_message_id: None,
        labels: None,
        receipt_to: parsed.receipt_to.clone(),
        pop_uidl: Some(uidl.to_string()),
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
        let none = to_incoming(&plain(), 7, 1, "aaa", false);

        assert_eq!(none.cc, None);
        assert_eq!(none.reply_to, None);

        let copied = crate::service::mime::ParsedMessage {
            cc: vec![EmailAddress::new("bob@example.com".to_string(), None)],
            reply_to: vec![EmailAddress::new("list@example.com".to_string(), None)],
            ..plain()
        };
        let stored = to_incoming(&copied, 7, 1, "aaa", false);

        assert_eq!(stored.cc.as_deref(), Some("bob@example.com"));
        assert_eq!(stored.reply_to.as_deref(), Some("list@example.com"));
    }

    #[test]
    fn test_a_downloaded_message_says_whether_it_carries_an_attachment() {
        // Both ways round. A message announced as carrying a file that has
        // none wastes the reader's time; one carrying a file and saying
        // nothing hides it entirely.
        assert!(!to_incoming(&plain(), 7, 1, "aaa", false).has_attachments);

        let carrying = crate::service::mime::ParsedMessage {
            attachments: vec![crate::service::mime::AttachmentInfo {
                filename: Some("figures.pdf".to_string()),
                mime_type: "application/pdf".to_string(),
                size: 1024,
            }],
            ..plain()
        };

        assert!(to_incoming(&carrying, 7, 1, "aaa", false).has_attachments);
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

    #[test]
    fn test_a_sync_with_no_connection_says_so_rather_than_reporting_an_empty_mailbox() {
        // "0 new, 0 on the server" reads as a mailbox with no mail in it. A
        // reader who is told that stops checking, and the mail is still there.
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
            ));

        assert!(outcome.is_err(), "a failed sync reported as a done one");
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

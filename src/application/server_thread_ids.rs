//! Mail already stored gets the server's word for its conversation, once
//! per account (#88).
//!
//! A check lists a folder and stores what is new; a message already stored
//! is never fetched again. So on a Gmail account the messages stored before
//! `X-GM-THRID` was asked for would have stayed threaded by their headers,
//! and the fix would have shown only on new mail. This pass asks the server
//! for the one field over the stored numbers of each kept folder, at the
//! account's next check, and records that it ran.
//!
//! # When it runs, and what it costs
//!
//! In the check, after the moves that were waiting are replayed and before
//! the folder list is asked for, on the session the check has just opened.
//! One `UID FETCH <numbers> (UID X-GM-THRID)` per kept folder, in batches,
//! and no headers: a number a message rather than a second download. The
//! figure is measured on the scripted server by
//! [`tests::test_what_the_field_costs_per_message_on_the_wire`] and quoted
//! in the plan's summary.
//!
//! # Once, and how that is known
//!
//! A row in `work_done_once`, named for the account, written only when
//! every kept folder answered. A pass that could not finish is tried again
//! at the next check, and asks only for the rows still without a word, so
//! a folder that answered is not asked twice. An account whose server does
//! not name conversations is recorded as done at once, since there is
//! nothing to ask; the extension is read from what the server advertised at
//! sign-in, because a server that does not know the word refuses the whole
//! fetch.

use std::time::{Duration, Instant};

use crate::application::mail_sync::Mailbox;
use crate::application::thread_identity::the_servers_name;
use crate::common::Result;
use crate::data::message_cache::MessageCache;

/// The marker's prefix; the account's id follows it.
pub const SERVER_THREAD_IDS_FETCHED_FOR: &str = "server thread ids fetched for ";

/// The `work_done_once` name for one account's pass.
pub fn the_marker_for(account_id: &str) -> String {
    format!("{SERVER_THREAD_IDS_FETCHED_FOR}{account_id}")
}

/// What one run of the pass did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThePass {
    /// Recorded as done on an earlier check; the server was not asked.
    AlreadyDone,
    /// The server names no conversations; recorded as done, nothing asked.
    TheServerNamesNone,
    /// Every kept folder answered and the rows were named.
    Done {
        rows_named: usize,
        folders_asked: usize,
        took: Duration,
    },
}

/// Give the account's stored mail the server's word, once.
///
/// Answers `Err` when the server could not be asked about a folder; nothing
/// is recorded then and the next check tries again. The rows named before
/// the refusal keep their words.
pub(crate) async fn fetch_the_server_thread_ids_once<M: Mailbox>(
    server: &M,
    cache: &MessageCache,
    account_id: &str,
) -> Result<ThePass> {
    let marker = the_marker_for(account_id);
    if cache.has_this_been_done(&marker)? {
        return Ok(ThePass::AlreadyDone);
    }
    if !server.what_this_server_can_do().await.gmail {
        cache.record_this_as_done(&marker)?;
        return Ok(ThePass::TheServerNamesNone);
    }

    let started = Instant::now();
    let mut rows_named = 0usize;
    let mut folders_asked = 0usize;
    for folder in cache.rows_without_the_servers_word(account_id)? {
        let numbers: Vec<u32> = folder.rows.iter().map(|(_, uid)| *uid).collect();
        let answered = server.thread_ids_of(&folder.folder_path, &numbers).await?;
        folders_asked += 1;
        for (uid, word) in answered {
            let Some((row, _)) = folder.rows.iter().find(|(_, number)| *number == uid) else {
                continue;
            };
            cache.name_the_conversation_after_the_server(*row, &the_servers_name(word))?;
            rows_named += 1;
        }
    }
    cache.record_this_as_done(&marker)?;

    let took = started.elapsed();
    // A count, a count and a duration, never an identifier or a folder's
    // name: a folder is named by its owner and a conversation id is Gmail's.
    tracing::info!(
        "Gave {rows_named} stored messages the server's conversation id over {folders_asked} folders in {} ms",
        took.as_millis()
    );
    Ok(ThePass::Done {
        rows_named,
        folders_asked,
        took,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::mail_sync::tests::Scripted;
    use crate::common::answering::{Conversation, LONG_ENOUGH, Turn, conversing};
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{CachedFolder, IncomingMessage};
    use crate::service::protocols::imap::against_a_server_that_answers::reading_only_on;

    const THE_ACCOUNT: &str = "acct-gmail";

    fn a_cache() -> TempHome<MessageCache> {
        TempHome::new(|dir| MessageCache::new(dir.to_path_buf(), None).expect("a cache"))
    }

    fn a_folder(cache: &MessageCache, path: &str) -> i64 {
        cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: THE_ACCOUNT.to_string(),
                name: path.to_string(),
                path: path.to_string(),
                folder_type: "Custom".to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder")
    }

    /// A message stored before the field was asked for: no word.
    fn stored_earlier(cache: &MessageCache, folder_id: i64, uid: u32) -> i64 {
        cache
            .upsert_message(&IncomingMessage {
                folder_id,
                uid,
                message_id: format!("m{uid}@x"),
                subject: "Re: the figures".to_string(),
                from_addr: "ada@example.com".to_string(),
                to_addr: "me@example.com".to_string(),
                cc: None,
                reply_to: None,
                date: format!("2026-09-{uid:02}T09:00:00Z"),
                internal_date: None,
                size_bytes: Some(100),
                refs_header: None,
                read: false,
                starred: false,
                answered: false,
                draft: false,
                deleted: false,
                has_attachments: false,
                safety: crate::service::safety::Verdict::ordinary(),
                gmail_message_id: None,
                server_thread_id: None,
                labels: None,
                receipt_to: None,
                list_unsubscribe: None,
                pop_uidl: None,
            })
            .expect("stored")
    }

    /// A Gmail server that knows these messages' conversations.
    fn gmail_knowing(words: &[(u32, u64)]) -> Scripted {
        Scripted::a_gmail_knowing_the_conversations(words)
    }

    /// The conversation the store files a row under, as the listing reads it.
    fn the_word_of(cache: &MessageCache, row: i64) -> Option<String> {
        cache
            .get_folders_for_account(THE_ACCOUNT)
            .expect("the folders")
            .iter()
            .flat_map(|folder| {
                cache
                    .get_message_list_sorted(folder.id, THE_ACCOUNT, None, None)
                    .expect("the listing")
            })
            .find(|listed| listed.id == row)
            .expect("the row is listed")
            .thread_id
    }

    fn run(server: &Scripted, cache: &MessageCache) -> Result<ThePass> {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
            .block_on(fetch_the_server_thread_ids_once(server, cache, THE_ACCOUNT))
    }

    #[test]
    fn test_stored_rows_get_their_word_once_per_kept_folder_and_the_pass_is_recorded() {
        let cache = a_cache();
        let inbox = a_folder(&cache, "INBOX");
        let sent = a_folder(&cache, "[Gmail]/Sent Mail");
        let a = stored_earlier(&cache, inbox, 1);
        let b = stored_earlier(&cache, inbox, 2);
        let c = stored_earlier(&cache, sent, 5);
        let server = gmail_knowing(&[(1, 70), (2, 70), (5, 71)]);

        let did = run(&server, &cache).expect("the pass ran");

        assert!(
            matches!(
                did,
                ThePass::Done {
                    rows_named: 3,
                    folders_asked: 2,
                    ..
                }
            ),
            "{did:?}"
        );
        assert_eq!(the_word_of(&cache, a).as_deref(), Some("gm:70"));
        assert_eq!(the_word_of(&cache, b).as_deref(), Some("gm:70"));
        assert_eq!(the_word_of(&cache, c).as_deref(), Some("gm:71"));
        assert_eq!(
            *server.happened.borrow(),
            vec![
                "asked INBOX for the conversation of [1, 2]".to_string(),
                "asked [Gmail]/Sent Mail for the conversation of [5]".to_string(),
            ],
            "each kept folder asked once for its own rows"
        );
        assert!(
            cache
                .has_this_been_done(&the_marker_for(THE_ACCOUNT))
                .expect("asked")
        );
    }

    #[test]
    fn test_a_second_check_asks_the_server_nothing() {
        let cache = a_cache();
        let inbox = a_folder(&cache, "INBOX");
        stored_earlier(&cache, inbox, 1);
        let server = gmail_knowing(&[(1, 70)]);
        run(&server, &cache).expect("the first pass");
        server.happened.borrow_mut().clear();

        let again = run(&server, &cache).expect("the second pass");

        assert_eq!(again, ThePass::AlreadyDone);
        assert!(
            server.happened.borrow().is_empty(),
            "{:?}",
            server.happened.borrow()
        );
    }

    #[test]
    fn test_a_server_that_names_no_conversation_is_not_asked_and_the_pass_is_recorded() {
        let cache = a_cache();
        let inbox = a_folder(&cache, "INBOX");
        let a = stored_earlier(&cache, inbox, 1);
        let server = gmail_knowing(&[(1, 70)]).advertising(&["IMAP4rev1"]);

        let did = run(&server, &cache).expect("the pass ran");

        assert_eq!(did, ThePass::TheServerNamesNone);
        assert!(
            server.happened.borrow().is_empty(),
            "{:?}",
            server.happened.borrow()
        );
        assert_eq!(
            the_word_of(&cache, a).as_deref(),
            Some("m1@x"),
            "the header root stays"
        );
        assert!(
            cache
                .has_this_been_done(&the_marker_for(THE_ACCOUNT))
                .expect("asked"),
            "an account with nothing to ask is asked again at every check"
        );
    }

    #[test]
    fn test_a_refusal_records_nothing_and_the_next_check_asks_for_what_is_still_missing() {
        let cache = a_cache();
        let inbox = a_folder(&cache, "INBOX");
        let a = stored_earlier(&cache, inbox, 1);
        let refusing = gmail_knowing(&[(1, 70)]).refusing_the_conversations();

        assert!(
            run(&refusing, &cache).is_err(),
            "a refusal was read as a pass"
        );
        assert!(
            !cache
                .has_this_been_done(&the_marker_for(THE_ACCOUNT))
                .expect("asked"),
            "a pass that could not finish was recorded as done"
        );
        assert_eq!(the_word_of(&cache, a).as_deref(), Some("m1@x"));

        let answering = gmail_knowing(&[(1, 70)]);
        let did = run(&answering, &cache).expect("the next check's pass");
        assert!(
            matches!(did, ThePass::Done { rows_named: 1, .. }),
            "{did:?}"
        );
        assert_eq!(the_word_of(&cache, a).as_deref(), Some("gm:70"));
    }

    #[test]
    fn test_only_the_rows_still_without_a_word_are_asked_about() {
        // The first check named one row and was refused on the second
        // folder; the next check asks only for the other.
        let cache = a_cache();
        let inbox = a_folder(&cache, "INBOX");
        let archive = a_folder(&cache, "Archive");
        let a = stored_earlier(&cache, archive, 1);
        let b = stored_earlier(&cache, inbox, 2);
        cache
            .name_the_conversation_after_the_server(a, &the_servers_name(70))
            .expect("named on an earlier check");
        let server = gmail_knowing(&[(1, 70), (2, 71)]);

        let did = run(&server, &cache).expect("the pass ran");

        assert!(
            matches!(
                did,
                ThePass::Done {
                    rows_named: 1,
                    folders_asked: 1,
                    ..
                }
            ),
            "{did:?}"
        );
        assert_eq!(
            *server.happened.borrow(),
            vec!["asked INBOX for the conversation of [2]".to_string()]
        );
        assert_eq!(the_word_of(&cache, b).as_deref(), Some("gm:71"));
    }

    // ── The wire ───────────────────────────────────────────────────────────

    /// One message's conversation attribute as Gmail writes it in a FETCH
    /// reply: the name, a space, and a number of up to twenty digits.
    fn the_attribute_for(word: u64) -> String {
        format!("X-GM-THRID {word}")
    }

    /// A Gmail server on the loopback that answers a conversation fetch for
    /// these messages, and a header fetch with nothing, which is all the
    /// request's length needs.
    async fn a_gmail_on_the_loopback(words: &'static [(u32, u64)]) -> Conversation {
        conversing("* OK loopback ready\r\n", move |line| {
            let tag = line.split_whitespace().next().unwrap_or("*").to_string();
            let said = line.to_uppercase();
            let verb = said.split_whitespace().nth(1).unwrap_or_default();
            match verb {
                "CAPABILITY" => Turn::Say(format!(
                    "* CAPABILITY IMAP4rev1 X-GM-EXT-1\r\n{tag} OK done\r\n"
                )),
                "LOGIN" | "AUTHENTICATE" => Turn::Say(format!("{tag} OK signed in\r\n")),
                "ID" => Turn::Say(format!("* ID NIL\r\n{tag} OK done\r\n")),
                "SELECT" | "EXAMINE" => Turn::Say(format!(
                    "* {} EXISTS\r\n* 0 RECENT\r\n* OK [UIDVALIDITY 1] valid\r\n\
                     {tag} OK [READ-WRITE] open\r\n",
                    words.len()
                )),
                "UID" if said.contains("X-GM-THRID)") => {
                    let mut reply = String::new();
                    for (n, (uid, word)) in words.iter().enumerate() {
                        reply.push_str(&format!(
                            "* {} FETCH (UID {uid} {})\r\n",
                            n + 1,
                            the_attribute_for(*word)
                        ));
                    }
                    reply.push_str(&format!("{tag} OK done\r\n"));
                    Turn::Say(reply)
                }
                "UID" | "LOGOUT" | "NOOP" | "CLOSE" => Turn::Say(format!("{tag} OK done\r\n")),
                _ => Turn::Say(format!("{tag} BAD unscripted\r\n")),
            }
        })
        .await
    }

    #[tokio::test]
    async fn test_the_conversation_of_each_message_is_asked_for_by_number_and_read_back() {
        let server = a_gmail_on_the_loopback(&[(1, 1_278_455_344_230_334_865), (3, 42)]).await;
        let mut session = reading_only_on(&server).await;
        tokio::time::timeout(LONG_ENOUGH, session.select_folder("INBOX"))
            .await
            .expect("the folder to open in time")
            .expect("the folder to open");

        let named = tokio::time::timeout(LONG_ENOUGH, session.thread_ids_of(&[1, 3]))
            .await
            .expect("the answer in time")
            .expect("the answer");

        assert_eq!(named, vec![(1, 1_278_455_344_230_334_865), (3, 42)]);
        let asked = server.transcript().await;
        assert!(
            asked
                .iter()
                .any(|line| line.ends_with("UID FETCH 1,3 (UID X-GM-THRID)")),
            "{asked:#?}"
        );
    }

    #[test]
    fn test_what_the_field_costs_per_message_on_the_wire() {
        // The cost of asking, as bytes on the wire, from the shape of the
        // request and the reply rather than from an estimate. Printed so a
        // run with --nocapture quotes it, and held so a change to the query
        // moves it here rather than in a document.
        //
        // The header fetch: one word more in a request made once per batch
        // of messages, and one attribute more per message in the reply.
        let a_word = 1_278_455_344_230_334_865u64;
        let per_message_in_the_header_reply = the_attribute_for(a_word).len() + 1;
        // The once-only pass: a request per batch, and one line per message
        // in the reply, as the scripted server above writes it.
        let per_message_in_the_pass =
            format!("* 1 FETCH (UID 17753 {})\r\n", the_attribute_for(a_word)).len();
        let the_testers_mailbox = 17_753usize;

        println!(
            "X-GM-THRID costs {per_message_in_the_header_reply} bytes a message in the header \
             fetch's reply and {per_message_in_the_pass} bytes a message in the once-only \
             pass's reply, at a nineteen-digit id; over {the_testers_mailbox} messages the pass \
             is {} bytes, measured 2026-09-19 on the scripted server",
            per_message_in_the_pass * the_testers_mailbox
        );
        assert_eq!(per_message_in_the_header_reply, 31);
        assert_eq!(per_message_in_the_pass, 54);
        assert!(
            per_message_in_the_pass * the_testers_mailbox < 1_000_000,
            "over a megabyte"
        );
    }
}

//! What a rule did to arriving mail, and what the mail server is told of it.
//!
//! # The defect this is about
//!
//! A rule that marks arriving mail read, flags it or labels it wrote that here
//! and nowhere else. The next check read the message's flags back from the
//! server, which had never heard of the change, and undid it: on a server that
//! cannot say only what changed, every rule's mark lasted until the next check
//! (ledger 678). Pratik decided on 2026-09-29 that a rule's changes go to the
//! server through the gate and the queue the menu commands use.
//!
//! # Where the sending happens
//!
//! In the check that brought the message, on the session it already holds,
//! after the rules have run and before the rule's move and before the flags are
//! read back. Before the move, so a mark names the folder and the number the
//! server still has the message under; before the flag read, so the read finds
//! the server already holding what the rule did.
//!
//! This module decides; [`crate::application::mail_sync`] sends. Nothing here
//! names a connection.

use crate::application::filters::Outcome;
use crate::data::message_cache::{CachedMessage, Tag};

/// One change a rule made here that the mail server is to be told about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AChange {
    /// Read, or unread.
    Read(bool),
    /// Flagged, or not.
    Flagged(bool),
    /// A label put on, by the keyword it travels as.
    Label {
        keyword: String,
        tag_id: String,
        name: String,
    },
}

/// What the server is told about one arriving message, from what it arrived
/// as, what its rules settled on and the labels they put on here.
///
/// Only what changed: a mark the message arrived with is not sent again.
pub fn what_the_server_is_told(
    _arrived: &CachedMessage,
    _outcome: &Outcome,
    _labels_put_on: &[Tag],
) -> Vec<AChange> {
    Vec::new()
}

/// One message's changes waiting to be told to the server by the check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Telling {
    /// The row here, which a change put back is put back on.
    pub message_row: i64,
    /// What the server calls it in the folder it arrived in.
    pub uid: u32,
    pub changes: Vec<AChange>,
    /// Whether the rules counted it as sorted already, so a change put back
    /// can take it out of that count again.
    pub counted: bool,
}

/// What became of the changes the check told the server about.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Told {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::filters::FilterEngine;
    use crate::application::mail_sync::{
        Filtering, FolderSync, Mailbox, WhatThisSyncIsFor, sync_folder,
    };
    use crate::common::Result;
    use crate::common::types::FolderType;
    use crate::data::message_cache::{CachedFolder, MessageCache, MessageFilterRule};
    use crate::service::protocols::imap::abilities::Abilities;
    use crate::service::protocols::imap::flag::{FLAGGED, SEEN};
    use crate::service::protocols::imap::{
        FolderCounts, ImapFolder, ImapMessage, MailboxStatus, Moved,
    };
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    const THE_ACCOUNT: &str = "acct";

    /// One message as a server holds it in one folder.
    #[derive(Debug, Clone)]
    struct HeldThere {
        uid: u32,
        message_id: String,
        subject: String,
        flags: Vec<String>,
    }

    /// A mail server that keeps each message's flags and changes them when
    /// asked, and writes down every change it was asked for, in order.
    ///
    /// Of its own rather than `mail_sync`'s scripted server, because these
    /// cases are about what a check leaves at the server and what the next
    /// check reads back, and that server answers from a script that nothing
    /// a check sends can change.
    struct AServerThatKeepsFlags {
        folders: RefCell<BTreeMap<String, Vec<HeldThere>>>,
        /// `Some` for a server that can answer "what changed since".
        highest_modseq: Option<u64>,
        /// Every STORE and MOVE, in the order asked.
        log: RefCell<Vec<String>>,
    }

    impl AServerThatKeepsFlags {
        fn holding(folder: &str, messages: &[(u32, &str, &[&str])]) -> Self {
            let held = messages
                .iter()
                .map(|(uid, subject, flags)| HeldThere {
                    uid: *uid,
                    message_id: format!("{uid}.{folder}@example.com"),
                    subject: (*subject).to_string(),
                    flags: flags.iter().map(|flag| (*flag).to_string()).collect(),
                })
                .collect();
            Self {
                folders: RefCell::new(BTreeMap::from([(folder.to_string(), held)])),
                highest_modseq: None,
                log: RefCell::new(Vec::new()),
            }
        }

        fn answering_what_changed_since(self) -> Self {
            Self {
                highest_modseq: Some(5),
                ..self
            }
        }

        /// A message arriving after the first check.
        fn and_then_receiving(&self, folder: &str, uid: u32, subject: &str) {
            self.folders
                .borrow_mut()
                .entry(folder.to_string())
                .or_default()
                .push(HeldThere {
                    uid,
                    message_id: format!("{uid}.{folder}@example.com"),
                    subject: subject.to_string(),
                    flags: Vec::new(),
                });
        }

        fn the_log(&self) -> Vec<String> {
            self.log.borrow().clone()
        }

        fn in_folder(&self, folder: &str) -> Vec<HeldThere> {
            self.folders
                .borrow()
                .get(folder)
                .cloned()
                .unwrap_or_default()
        }
    }

    impl Mailbox for AServerThatKeepsFlags {
        async fn folder_counts(&self, folder: &str) -> Result<FolderCounts> {
            let held = self.in_folder(folder);
            let unread = held
                .iter()
                .filter(|message| !message.flags.iter().any(|flag| flag == SEEN))
                .count();
            Ok(FolderCounts {
                total: held.len() as u32,
                unread: unread as u32,
            })
        }

        async fn select_folder(&self, _folder: &str) -> Result<MailboxStatus> {
            Ok(MailboxStatus {
                uid_validity: Some(1),
                highest_modseq: self.highest_modseq,
                keeps_the_junk_mark: false,
            })
        }

        async fn what_this_server_can_do(&self) -> Abilities {
            Abilities::default()
        }

        async fn list_uids(&self, folder: &str) -> Result<Vec<u32>> {
            Ok(self.in_folder(folder).iter().map(|held| held.uid).collect())
        }

        async fn list_uids_above(&self, folder: &str, after: u32) -> Result<Vec<u32>> {
            Ok(self
                .list_uids(folder)
                .await?
                .into_iter()
                .filter(|uid| *uid >= after)
                .collect())
        }

        async fn fetch_headers(&self, folder: &str, uids: &[u32]) -> Result<Vec<ImapMessage>> {
            Ok(self
                .in_folder(folder)
                .into_iter()
                .filter(|held| uids.contains(&held.uid))
                .map(|held| ImapMessage {
                    uid: held.uid,
                    subject: held.subject,
                    message_id: Some(held.message_id),
                    flags: held.flags,
                    ..Default::default()
                })
                .collect())
        }

        async fn thread_ids_of(&self, _folder: &str, _uids: &[u32]) -> Result<Vec<(u32, u64)>> {
            Ok(Vec::new())
        }

        async fn move_message(&self, from: &str, uid: u32, into: &str) -> Result<Moved> {
            self.log
                .borrow_mut()
                .push(format!("MOVE {uid} in {from} into {into}"));
            let mut folders = self.folders.borrow_mut();
            let leaving = folders.get_mut(from).and_then(|held| {
                let at = held.iter().position(|message| message.uid == uid)?;
                Some(held.remove(at))
            });
            if let Some(mut message) = leaving {
                let arriving = folders.entry(into.to_string()).or_default();
                message.uid = arriving.iter().map(|held| held.uid).max().unwrap_or(0) + 1;
                arriving.push(message);
            }
            Ok(Moved::Moved)
        }

        async fn fetch_flags(
            &self,
            folder: &str,
            held: &[u32],
            changed_since: Option<u64>,
        ) -> Result<Vec<(u32, Vec<String>)>> {
            // Asked "what changed since", a server answers for the whole
            // mailbox, messages this same check just brought down included,
            // as `mail_sync`'s scripted server does.
            Ok(self
                .in_folder(folder)
                .into_iter()
                .filter(|message| changed_since.is_some() || held.contains(&message.uid))
                .map(|message| (message.uid, message.flags))
                .collect())
        }

        async fn fetch_message_body(&self, _folder: &str, _uid: u32) -> Result<Vec<u8>> {
            Ok(Vec::new())
        }
    }

    /// A cache holding the account's Inbox and a folder to file into, with
    /// the label Money, sent as a keyword, and the label Local, which has none.
    fn an_account() -> (tempfile::TempDir, MessageCache, i64) {
        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
        let inbox = a_folder(&cache, "Inbox", "INBOX", FolderType::Inbox);
        a_folder(&cache, "Invoices", "Invoices", FolderType::Custom);
        for (id, name, keyword) in [
            ("tag-money", "Money", Some("Money")),
            ("tag-local", "Local", None),
        ] {
            cache
                .create_tag(&Tag {
                    id: id.to_string(),
                    account_id: THE_ACCOUNT.to_string(),
                    name: name.to_string(),
                    color: "#009900".to_string(),
                    created_at: "2026-10-01T00:00:00Z".to_string(),
                    keyword: keyword.map(str::to_string),
                })
                .expect("the label made");
        }
        (dir, cache, inbox)
    }

    fn a_folder(cache: &MessageCache, name: &str, path: &str, kind: FolderType) -> i64 {
        cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: THE_ACCOUNT.to_string(),
                name: name.to_string(),
                path: path.to_string(),
                folder_type: kind.as_str().to_string(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("the folder")
    }

    /// The folder as the server lists it.
    fn the_inbox() -> ImapFolder {
        ImapFolder {
            name: "Inbox".into(),
            display_path: "INBOX".into(),
            path: "INBOX".into(),
            folder_type: FolderType::Inbox,
            selectable: true,
            holds_all_mail: false,
            subscribed: true,
            delimiter: None,
        }
    }

    /// Rules that each match a subject holding "Invoice", one per action and
    /// its value, in the order given.
    fn rules(actions: &[(&str, Option<&str>)]) -> FilterEngine {
        let written: Vec<MessageFilterRule> = actions
            .iter()
            .enumerate()
            .map(|(nth, (action, value))| MessageFilterRule {
                id: format!("r{nth}"),
                account_id: THE_ACCOUNT.into(),
                name: format!("Rule {nth}"),
                field: "subject".into(),
                match_type: "contains".into(),
                pattern: "Invoice".into(),
                case_sensitive: false,
                action_type: (*action).to_string(),
                action_value: value.map(str::to_string),
                enabled: true,
                plays_a_sound: false,
                created_at: "2026-10-01T00:00:00Z".into(),
            })
            .collect();
        let mut engine = FilterEngine::default();
        engine.load_from_persisted(&written);
        engine
    }

    /// One check of the Inbox, with the rules and the account's answer to
    /// whether mail may be changed.
    fn a_check(
        server: &AServerThatKeepsFlags,
        cache: &MessageCache,
        inbox: i64,
        engine: &FilterEngine,
        allowed: crate::application::allowed::Allowed,
    ) -> FolderSync {
        let filtering = Filtering {
            rules: engine,
            allowed,
        };
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
            .block_on(sync_folder(
                server,
                cache,
                &the_inbox(),
                inbox,
                50,
                Some(&filtering),
                WhatThisSyncIsFor::WhateverHasChanged,
            ))
            .expect("the check to finish")
    }

    fn allowed() -> crate::application::allowed::Allowed {
        crate::application::allowed::Allowed::EVERYTHING
    }

    /// The row here for a number in a folder, read back.
    fn the_row(cache: &MessageCache, folder: i64, uid: u32) -> CachedMessage {
        let row = cache
            .message_row_for_uid(folder, uid)
            .expect("the row looked up")
            .expect("the row is here");
        cache
            .get_message(row)
            .expect("the row read")
            .expect("the row is there")
    }

    fn the_labels_on(cache: &MessageCache, row: i64) -> Vec<String> {
        cache
            .get_tags_for_message(row)
            .expect("the labels read")
            .into_iter()
            .map(|tag| tag.name)
            .collect()
    }

    #[test]
    fn test_a_rule_that_marks_read_and_files_tells_the_server_the_mark_before_the_move() {
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])]);
        let engine = rules(&[("mark_as_read", None), ("move_to_folder", Some("Invoices"))]);

        a_check(&server, &cache, inbox, &engine, allowed());

        assert_eq!(
            server.the_log(),
            [
                format!("STORE +{SEEN} on 7 in INBOX"),
                "MOVE 7 in INBOX into Invoices".to_string(),
            ],
            "the mark did not reach the server before the move, from where the message was"
        );
    }

    #[test]
    fn test_a_rule_that_flags_and_labels_tells_the_server_the_flag_and_the_labels_keyword() {
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])]);
        let engine = rules(&[("star", None), ("add_tag", Some("Money"))]);

        a_check(&server, &cache, inbox, &engine, allowed());

        assert_eq!(
            server.the_log(),
            [
                format!("STORE +{FLAGGED} on 7 in INBOX"),
                "STORE +Money on 7 in INBOX".to_string(),
            ]
        );
        let row = the_row(&cache, inbox, 7);
        assert!(row.starred, "the flag is not on the row here");
        assert_eq!(the_labels_on(&cache, row.id), ["Money"]);
    }

    #[test]
    fn test_a_mark_the_message_arrived_with_is_not_sent_again() {
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[SEEN])]);
        let engine = rules(&[("mark_as_read", None)]);

        a_check(&server, &cache, inbox, &engine, allowed());

        assert!(
            server.the_log().is_empty(),
            "a mark the server already held was sent again: {:?}",
            server.the_log()
        );
        assert!(the_row(&cache, inbox, 7).read);
    }

    #[test]
    fn test_a_label_with_no_keyword_is_put_on_here_and_sent_nowhere() {
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])]);
        let engine = rules(&[("add_tag", Some("Local"))]);

        let done = a_check(&server, &cache, inbox, &engine, allowed());

        assert!(server.the_log().is_empty(), "{:?}", server.the_log());
        let row = the_row(&cache, inbox, 7);
        assert_eq!(the_labels_on(&cache, row.id), ["Local"]);
        assert_eq!(done.filtered.changed, 1);
    }

    #[test]
    fn test_the_next_check_leaves_a_rules_mark_the_server_now_holds() {
        // A server that cannot say only what changed answers the next check
        // with every message's flags, and until 2026-10-01 those flags were
        // the ones the message arrived with, so the rule's mark was undone.
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Invoice 4021", &[])]);
        let engine = rules(&[("mark_as_read", None)]);
        a_check(&server, &cache, inbox, &engine, allowed());

        let next = a_check(&server, &cache, inbox, &engine, allowed());

        assert!(
            the_row(&cache, inbox, 7).read,
            "the next check undid the rule's mark"
        );
        assert_eq!(next.flags_updated, 0, "the next check changed a row");
    }

    #[test]
    fn test_a_server_that_answers_what_changed_since_keeps_a_rules_mark_in_the_same_check() {
        // Asked what changed since, the server answers for the whole mailbox,
        // the message this check just brought down included, so a mark the
        // server never heard of was undone by the check that made it.
        let (_dir, cache, inbox) = an_account();
        let server = AServerThatKeepsFlags::holding("INBOX", &[(7, "Lunch", &[])])
            .answering_what_changed_since();
        let engine = rules(&[("mark_as_read", None)]);
        a_check(&server, &cache, inbox, &engine, allowed());
        server.and_then_receiving("INBOX", 8, "Invoice 4021");

        a_check(&server, &cache, inbox, &engine, allowed());

        assert!(
            the_row(&cache, inbox, 8).read,
            "the check that made the rule's mark undid it"
        );
    }

    #[test]
    fn test_what_the_server_is_told_is_only_what_the_rules_changed() {
        let arrived = |read: bool, starred: bool| CachedMessage {
            id: 1,
            uid: 7,
            folder_id: 1,
            message_id: "7.INBOX@example.com".to_string(),
            subject: "Invoice 4021".to_string(),
            from_addr: String::new(),
            to_addr: String::new(),
            cc: None,
            date: String::new(),
            body_plain: None,
            body_html: None,
            read,
            starred,
            deleted: false,
            safety: crate::service::safety::Safety::default(),
        };
        let label = |id: &str, name: &str, keyword: Option<&str>| Tag {
            id: id.to_string(),
            account_id: THE_ACCOUNT.to_string(),
            name: name.to_string(),
            color: String::new(),
            created_at: String::new(),
            keyword: keyword.map(str::to_string),
        };
        let marking = |read: Option<bool>, starred: Option<bool>| Outcome {
            read,
            starred,
            ..Outcome::default()
        };
        let rows: Vec<(CachedMessage, Outcome, Vec<Tag>, Vec<AChange>)> = vec![
            (
                arrived(false, false),
                marking(Some(true), None),
                vec![],
                vec![AChange::Read(true)],
            ),
            (
                arrived(true, false),
                marking(Some(true), None),
                vec![],
                vec![],
            ),
            (
                arrived(true, false),
                marking(Some(false), None),
                vec![],
                vec![AChange::Read(false)],
            ),
            (
                arrived(false, false),
                marking(None, Some(true)),
                vec![],
                vec![AChange::Flagged(true)],
            ),
            (
                arrived(false, true),
                marking(None, Some(true)),
                vec![],
                vec![],
            ),
            (
                arrived(false, true),
                marking(None, Some(false)),
                vec![],
                vec![AChange::Flagged(false)],
            ),
            (
                arrived(false, false),
                Outcome {
                    tags: vec!["Money".into(), "Local".into()],
                    ..Outcome::default()
                },
                vec![
                    label("tag-money", "Money", Some("Money")),
                    label("tag-local", "Local", None),
                ],
                vec![AChange::Label {
                    keyword: "Money".into(),
                    tag_id: "tag-money".into(),
                    name: "Money".into(),
                }],
            ),
            (
                arrived(false, false),
                Outcome {
                    read: Some(true),
                    delete: true,
                    ..Outcome::default()
                },
                vec![],
                vec![],
            ),
            (arrived(false, false), Outcome::default(), vec![], vec![]),
        ];
        for (message, outcome, labels, told) in rows {
            assert_eq!(
                what_the_server_is_told(&message, &outcome, &labels),
                told,
                "arrived read {} starred {}, outcome {outcome:?}",
                message.read,
                message.starred
            );
        }
    }
}

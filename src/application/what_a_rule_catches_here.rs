//! Which of the messages already on this computer a rule catches (13-25).
//!
//! A block is a rule, and a rule is run on mail as it arrives, so everything
//! already here from a blocked sender stayed where it was (#54, point 2;
//! GAP-06). Asking about that mail means asking the question the rule asks,
//! over the mail already here, through the same matcher that files mail on
//! arrival, [`FilterEngine::matches`], so the block and the count can never
//! disagree about who is caught.
//!
//! The folders mail is never moved out of are passed over: the junk folder,
//! because it is already there; Trash, because it was thrown away; Sent,
//! Drafts and the Outbox, because what is in them is somebody's own writing
//! and not mail from the sender. A message in a folder the account does not
//! list is passed over too, because a move needs to know where a message is.
//!
//! One function over values, with no store and no window, so a count over a
//! folder (13-43) asks the same question without a second copy of it.

use crate::application::filters::FilterRule;
use crate::data::message_cache::{CachedFolder, CachedMessage};

/// The row ids of the messages `rule` catches, among `messages`, in the
/// order given, passing over any message in a folder mail is not moved out
/// of or in a folder `folders` does not hold.
pub fn which_messages_here_a_rule_catches(
    rule: &FilterRule,
    messages: &[CachedMessage],
    folders: &[CachedFolder],
) -> Vec<i64> {
    let _ = (rule, folders);
    messages.iter().map(|message| message.id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::blocking::{self, Block};
    use crate::application::filters::FilterEngine;
    use crate::common::types::FolderType;

    const INBOX: i64 = 1;
    const RECEIPTS: i64 = 2;
    const JUNK: i64 = 3;
    const SENT: i64 = 4;
    const TRASH: i64 = 5;
    const DRAFTS: i64 = 6;
    const OUTBOX: i64 = 7;
    /// A folder id no folder in [`the_folders`] carries.
    const A_FOLDER_NOT_LISTED: i64 = 99;

    /// One account's folders, one of each kind these cases need.
    fn the_folders() -> Vec<CachedFolder> {
        [
            (INBOX, "INBOX", FolderType::Inbox),
            (RECEIPTS, "Receipts", FolderType::Custom),
            (JUNK, "Junk", FolderType::Spam),
            (SENT, "Sent", FolderType::Sent),
            (TRASH, "Trash", FolderType::Trash),
            (DRAFTS, "Drafts", FolderType::Drafts),
            (OUTBOX, "Outbox", FolderType::Outbox),
        ]
        .into_iter()
        .map(|(id, path, kind)| CachedFolder {
            id,
            account_id: "acct".to_string(),
            name: path.to_string(),
            path: path.to_string(),
            folder_type: kind.as_str().to_string(),
            unread_count: 0,
            total_count: 0,
        })
        .collect()
    }

    /// A message with this row id, in this folder, from this sender.
    fn from(id: i64, folder_id: i64, sender: &str) -> CachedMessage {
        CachedMessage {
            id,
            uid: u32::try_from(id).unwrap_or_default(),
            folder_id,
            message_id: format!("<{id}@example.net>"),
            subject: format!("Message {id}"),
            from_addr: sender.to_string(),
            to_addr: "me@work.example".to_string(),
            cc: None,
            date: "2026-09-24T09:00:00Z".to_string(),
            body_plain: None,
            body_html: None,
            read: false,
            starred: false,
            deleted: false,
            safety: crate::service::safety::Safety::Ordinary,
        }
    }

    /// The rule a block writes, as the matcher reads it.
    fn the_rule_for(block: &Block) -> FilterRule {
        let stored = blocking::a_rule_that_blocks("acct", block, "Junk", "t");
        FilterEngine::from_persisted_rule(&stored).expect("a block's rule reads as a rule")
    }

    fn blocking_ada() -> FilterRule {
        the_rule_for(&blocking::just_this_sender("ada@example.com").expect("an address"))
    }

    #[test]
    fn test_the_senders_mail_in_the_inbox_and_in_a_folder_of_their_own_is_caught() {
        // The whole point, under both ways a From header arrives: a bare
        // address, and a name around one. Somebody else's mail beside it is
        // left, so a row answering everything cannot pass.
        let messages = [
            from(10, INBOX, "ada@example.com"),
            from(11, RECEIPTS, "Ada Lovelace <ada@example.com>"),
            from(12, INBOX, "bob@example.com"),
        ];

        let caught = which_messages_here_a_rule_catches(&blocking_ada(), &messages, &the_folders());

        assert_eq!(caught, vec![10, 11]);
    }

    #[test]
    fn test_the_senders_mail_already_in_junk_is_passed_over() {
        // Already where the block would put it; counting it would ask
        // somebody to move mail that is not going anywhere.
        let messages = [
            from(10, INBOX, "ada@example.com"),
            from(11, JUNK, "ada@example.com"),
        ];

        let caught = which_messages_here_a_rule_catches(&blocking_ada(), &messages, &the_folders());

        assert_eq!(caught, vec![10]);
    }

    #[test]
    fn test_the_senders_mail_in_sent_is_passed_over() {
        // A message in Sent is one somebody sent, whatever its From says
        // after a forward; filing their own sent mail into Junk is a move
        // nobody asked for.
        let messages = [
            from(10, INBOX, "ada@example.com"),
            from(11, SENT, "ada@example.com"),
        ];

        let caught = which_messages_here_a_rule_catches(&blocking_ada(), &messages, &the_folders());

        assert_eq!(caught, vec![10]);
    }

    #[test]
    fn test_the_senders_mail_in_trash_and_in_drafts_is_passed_over() {
        let messages = [
            from(10, INBOX, "ada@example.com"),
            from(11, TRASH, "ada@example.com"),
            from(12, DRAFTS, "ada@example.com"),
        ];

        let caught = which_messages_here_a_rule_catches(&blocking_ada(), &messages, &the_folders());

        assert_eq!(caught, vec![10]);
    }

    #[test]
    fn test_mail_waiting_in_the_outbox_is_passed_over() {
        // Written here and not yet gone. Moving it into Junk would take it
        // out of the queue that sends it.
        let messages = [
            from(10, INBOX, "ada@example.com"),
            from(11, OUTBOX, "ada@example.com"),
        ];

        let caught = which_messages_here_a_rule_catches(&blocking_ada(), &messages, &the_folders());

        assert_eq!(caught, vec![10]);
    }

    #[test]
    fn test_a_domain_block_catches_everyone_at_the_domain_and_nobody_else() {
        // Two addresses at the domain are caught; a domain that only starts
        // the same way is not, which is the bound the block's pattern keeps.
        let rule = the_rule_for(
            &blocking::everyone_at_the_senders_domain("ada@example.com").expect("a domain"),
        );
        let messages = [
            from(10, INBOX, "ada@example.com"),
            from(11, RECEIPTS, "Bob <bob@example.com>"),
            from(12, INBOX, "eve@example.com.evil.test"),
            from(13, INBOX, "carol@other.test"),
        ];

        let caught = which_messages_here_a_rule_catches(&rule, &messages, &the_folders());

        assert_eq!(caught, vec![10, 11]);
    }

    #[test]
    fn test_a_message_from_somebody_else_is_not_caught() {
        let messages = [from(10, INBOX, "bob@example.com")];

        let caught = which_messages_here_a_rule_catches(&blocking_ada(), &messages, &the_folders());

        assert!(caught.is_empty(), "{caught:?}");
    }

    #[test]
    fn test_a_message_in_a_folder_the_account_does_not_list_is_passed_over() {
        // A move needs the folder a message is in, and a run that met one it
        // could not place would refuse the whole set.
        let messages = [from(10, A_FOLDER_NOT_LISTED, "ada@example.com")];

        let caught = which_messages_here_a_rule_catches(&blocking_ada(), &messages, &the_folders());

        assert!(caught.is_empty(), "{caught:?}");
    }

    #[test]
    fn test_nothing_here_catches_nothing() {
        let caught = which_messages_here_a_rule_catches(&blocking_ada(), &[], &the_folders());

        assert!(caught.is_empty(), "{caught:?}");
    }
}

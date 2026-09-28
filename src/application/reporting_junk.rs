//! Report as Junk: what a report does on each kind of account, and the one
//! sentence it says (#54, GAP-06's first `[D]` line).
//!
//! A report is a move into the account's junk folder, told to the provider
//! where the provider listens and said plainly where it does not. Three routes
//! and one gap, from RESEARCH-3:
//!
//! - An IMAP server whose folder keeps keywords is given `$Junk`, after
//!   `$NotJunk` is taken off, since RFC 9051 reads the two together as
//!   neither. Then the message moves.
//! - Gmail documents a move into Spam as a report (Gmail Help 1366858), so
//!   the move is the report and no keyword is sent.
//! - Microsoft offers no supported call a mail program may use: `reportMessage`
//!   is in the Graph beta alone and needs a permission this program does not
//!   ask for. The message moves to Junk Email and the sentence says Microsoft
//!   has not been told.
//! - POP has no junk folder at the server, so nothing is sent.
//!
//! Everything here that decides is a function over plain values. The one part
//! that talks to a server, [`mark_as_junk_at_the_server`], goes through the
//! gated flag write every other flag change uses and adds no gated method of
//! its own.

use crate::application::blocking::BlockedMailGoesTo;
use crate::common::Result;
use crate::data::account::Account;

/// The kinds of account a report treats differently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountKind {
    /// Collects its mail with POP: nothing at the server to report to.
    Pop,
    /// Gmail, where a move into Spam is the report.
    Gmail,
    /// Outlook.com or Microsoft 365, which offers no supported report.
    Microsoft,
    /// Any other IMAP server, told with the `$Junk` keyword where it keeps it.
    OtherImap,
}

impl AccountKind {
    /// Which kind this account is. Stub for the red.
    pub fn of(account: &Account) -> Self {
        let _ = account;
        AccountKind::OtherImap
    }
}

/// What a report does on one account, decided before anything is sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Report {
    /// Nothing leaves this computer, and this sentence says why.
    NothingSent(String),
    /// Mark the messages at the server where `mark` says to, then move them
    /// into `junk`, the account's junk folder by its path.
    MarkThenMove { junk: String, mark: bool },
}

/// What a report does on an account of this kind, with this junk folder,
/// under this answer from the write gate. Stub for the red.
pub fn what_a_report_does(
    kind: AccountKind,
    junk: BlockedMailGoesTo<'_>,
    may_change_mail: std::result::Result<(), String>,
) -> Report {
    let _ = (kind, junk, may_change_mail);
    Report::NothingSent(String::new())
}

/// What became of the junk mark at the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Marked {
    /// The folder keeps the keyword, and it was set.
    Kept,
    /// The folder keeps no such keyword, so nothing was sent.
    NotKept,
    /// Not asked, because the move is the report (Gmail).
    NotAsked,
    /// The server was asked and the mark could not be set, for this reason.
    Failed(String),
}

impl Marked {
    /// One account's answer from two of its folders. Stub for the red.
    pub fn and(self, other: Marked) -> Marked {
        let _ = other;
        self
    }
}

/// The one sentence a report says for one account. Stub for the red.
pub fn what_reporting_did(
    kind: AccountKind,
    how_many_went: usize,
    junk_name: &str,
    passed_over: usize,
    marked: &Marked,
) -> String {
    let _ = (kind, how_many_went, junk_name, passed_over, marked);
    String::new()
}

/// What marking a message as junk asks of a mail server.
///
/// Named for what it does rather than for the protocol, so the marking can be
/// held against a loopback server from this module's own tests, the way
/// [`crate::application::moves_waiting::ReplaysAMove`] is. Crate-private: the
/// seam the tests need, not something a caller should know about.
pub(crate) trait MarksMessages {
    /// Open the folder and say what the server said about it.
    async fn open(&self, folder: &str) -> Result<crate::service::protocols::imap::MailboxStatus>;
    /// Put a flag on or take it off, through the write gate.
    async fn set_flag(&self, folder: &str, uid: u32, flag: &str, on: bool) -> Result<()>;
}

impl MarksMessages for crate::application::mail_controller::MailController {
    async fn open(&self, folder: &str) -> Result<crate::service::protocols::imap::MailboxStatus> {
        self.select_folder(folder).await
    }

    async fn set_flag(&self, folder: &str, uid: u32, flag: &str, on: bool) -> Result<()> {
        crate::application::mail_controller::MailController::set_flag(self, folder, uid, flag, on)
            .await
    }
}

/// Mark these messages in this folder as junk at the account's server.
pub async fn mark_as_junk_at_the_server(
    controller: &crate::application::mail_controller::MailController,
    folder: &str,
    uids: &[u32],
) -> Result<Marked> {
    marking(controller, folder, uids).await
}

/// The marking over whatever answers as a server. Stub for the red: it asks
/// in the wrong order and whatever the folder keeps.
async fn marking<S: MarksMessages>(server: &S, folder: &str, uids: &[u32]) -> Result<Marked> {
    use crate::service::protocols::imap::flag::{JUNK, NOT_JUNK};
    server.open(folder).await?;
    for uid in uids {
        server.set_flag(folder, *uid, JUNK, true).await?;
        server.set_flag(folder, *uid, NOT_JUNK, false).await?;
    }
    Ok(Marked::Kept)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::answering::{Conversation, Turn, conversing};
    use crate::service::protocols::imap::ImapSession;

    fn an_account(protocol: &str, provider: Option<&str>) -> Account {
        let mut account = Account::new("Somebody".to_string(), "somebody@example.com".to_string());
        account.protocol = protocol.to_string();
        account.provider = provider.map(str::to_string);
        account
    }

    // ── Which kind of account ─────────────────────────────────────────────

    #[test]
    fn test_a_pop_account_is_its_own_kind_whatever_its_provider() {
        assert_eq!(AccountKind::of(&an_account("pop3", None)), AccountKind::Pop);
        assert_eq!(
            AccountKind::of(&an_account("pop3", Some("gmail"))),
            AccountKind::Pop,
            "a Gmail account collecting over POP has no Spam folder here to move into"
        );
    }

    #[test]
    fn test_gmail_and_microsoft_are_told_apart_by_provider() {
        assert_eq!(
            AccountKind::of(&an_account("imap", Some("gmail"))),
            AccountKind::Gmail
        );
        assert_eq!(
            AccountKind::of(&an_account("imap", Some("outlook"))),
            AccountKind::Microsoft
        );
        assert_eq!(
            AccountKind::of(&an_account("imap", Some("yahoo"))),
            AccountKind::OtherImap
        );
        assert_eq!(
            AccountKind::of(&an_account("imap", None)),
            AccountKind::OtherImap
        );
    }

    // ── What a report does ────────────────────────────────────────────────

    #[test]
    fn test_a_pop_account_reports_nothing_and_says_why() {
        let report = what_a_report_does(
            AccountKind::Pop,
            BlockedMailGoesTo::NoFoldersKnownYet,
            Ok(()),
        );
        assert_eq!(
            report,
            Report::NothingSent(
                "This account collects its mail with POP, which has no junk folder at the \
                 server, so nothing was reported."
                    .to_string()
            )
        );
    }

    #[test]
    fn test_an_account_whose_changes_are_off_is_refused_in_the_gates_words() {
        let gate = "Changes to your mail are off for this account.".to_string();
        let report = what_a_report_does(
            AccountKind::OtherImap,
            BlockedMailGoesTo::TheJunkFolder("Junk"),
            Err(gate.clone()),
        );
        assert_eq!(report, Report::NothingSent(gate));
    }

    #[test]
    fn test_an_account_with_no_junk_folder_reports_nothing() {
        let Report::NothingSent(said) = what_a_report_does(
            AccountKind::OtherImap,
            BlockedMailGoesTo::NoJunkFolderFound,
            Ok(()),
        ) else {
            panic!("an account with no junk folder was sent something");
        };
        assert!(said.starts_with("Nothing was reported."), "{said}");
        assert!(said.contains("junk"), "{said}");
    }

    #[test]
    fn test_an_account_that_has_not_learned_its_folders_reports_nothing() {
        let Report::NothingSent(said) = what_a_report_does(
            AccountKind::Microsoft,
            BlockedMailGoesTo::NoFoldersKnownYet,
            Ok(()),
        ) else {
            panic!("an account with no folders known was sent something");
        };
        assert!(said.starts_with("Nothing was reported."), "{said}");
        assert!(said.contains("Check for mail"), "{said}");
    }

    #[test]
    fn test_a_server_account_is_marked_then_moved() {
        for kind in [AccountKind::OtherImap, AccountKind::Microsoft] {
            assert_eq!(
                what_a_report_does(kind, BlockedMailGoesTo::TheJunkFolder("Junk"), Ok(())),
                Report::MarkThenMove {
                    junk: "Junk".to_string(),
                    mark: true
                },
                "{kind:?}"
            );
        }
    }

    #[test]
    fn test_gmail_is_reported_by_the_move_itself() {
        assert_eq!(
            what_a_report_does(
                AccountKind::Gmail,
                BlockedMailGoesTo::TheJunkFolder("[Gmail]/Spam"),
                Ok(())
            ),
            Report::MarkThenMove {
                junk: "[Gmail]/Spam".to_string(),
                mark: false
            }
        );
    }

    // ── What a report says ────────────────────────────────────────────────

    #[test]
    fn test_a_server_that_keeps_the_mark_says_reported_as_junk() {
        assert_eq!(
            what_reporting_did(AccountKind::OtherImap, 3, "Junk", 0, &Marked::Kept),
            "3 messages reported as junk and moved to Junk."
        );
    }

    #[test]
    fn test_gmail_says_the_move_tells_google() {
        assert_eq!(
            what_reporting_did(AccountKind::Gmail, 3, "Spam", 0, &Marked::NotAsked),
            "3 messages moved to Spam, which tells Google they are junk."
        );
    }

    #[test]
    fn test_a_server_that_keeps_no_mark_says_only_the_folder_says_so() {
        assert_eq!(
            what_reporting_did(AccountKind::OtherImap, 3, "Junk", 0, &Marked::NotKept),
            "3 messages moved to Junk. This server does not keep a junk mark, so only the \
             folder says they are junk."
        );
    }

    #[test]
    fn test_microsoft_says_it_has_not_been_told() {
        for marked in [Marked::Kept, Marked::NotKept] {
            assert_eq!(
                what_reporting_did(AccountKind::Microsoft, 3, "Junk Email", 0, &marked),
                "3 messages moved to Junk Email. Microsoft offers no supported way for a \
                 mail program to report junk, so Microsoft has not been told.",
                "{marked:?}"
            );
        }
    }

    #[test]
    fn test_messages_already_in_junk_are_counted_as_passed_over() {
        assert_eq!(
            what_reporting_did(AccountKind::OtherImap, 3, "Junk", 1, &Marked::Kept),
            "3 messages reported as junk and moved to Junk, and 1 already in Junk was \
             passed over."
        );
        assert_eq!(
            what_reporting_did(AccountKind::Gmail, 2, "Spam", 2, &Marked::NotAsked),
            "2 messages moved to Spam, which tells Google they are junk, and 2 already in \
             Spam were passed over."
        );
    }

    #[test]
    fn test_a_mark_that_failed_is_said_and_the_move_still_named() {
        assert_eq!(
            what_reporting_did(
                AccountKind::OtherImap,
                2,
                "Junk",
                0,
                &Marked::Failed("the server would not do it".to_string())
            ),
            "2 messages moved to Junk. The junk mark could not be set: the server would not \
             do it."
        );
    }

    #[test]
    fn test_one_message_is_said_in_the_singular() {
        assert_eq!(
            what_reporting_did(AccountKind::OtherImap, 1, "Junk", 0, &Marked::Kept),
            "1 message reported as junk and moved to Junk."
        );
        assert_eq!(
            what_reporting_did(AccountKind::Gmail, 1, "Spam", 0, &Marked::NotAsked),
            "1 message moved to Spam, which tells Google it is junk."
        );
        assert_eq!(
            what_reporting_did(AccountKind::OtherImap, 1, "Junk", 0, &Marked::NotKept),
            "1 message moved to Junk. This server does not keep a junk mark, so only the \
             folder says it is junk."
        );
    }

    #[test]
    fn test_a_report_where_every_message_was_already_junk_says_nothing_was_reported() {
        assert_eq!(
            what_reporting_did(AccountKind::OtherImap, 0, "Junk", 2, &Marked::NotAsked),
            "Nothing was reported: 2 already in Junk were passed over."
        );
    }

    #[test]
    fn test_one_accounts_answer_over_two_folders_is_its_worst() {
        assert_eq!(Marked::Kept.and(Marked::Kept), Marked::Kept);
        assert_eq!(Marked::Kept.and(Marked::NotKept), Marked::NotKept);
        assert_eq!(Marked::NotKept.and(Marked::Kept), Marked::NotKept);
        let failed = Marked::Failed("no".to_string());
        assert_eq!(Marked::Kept.and(failed.clone()), failed);
        assert_eq!(failed.clone().and(Marked::NotKept), failed);
    }

    // ── The mark at a server ──────────────────────────────────────────────

    /// A session the test opened and allowed to change things, as the
    /// controller's own tests hold one: signing in reads the account's
    /// setting out of the profile of whoever runs the suite.
    struct ASessionTheTestOpened(tokio::sync::Mutex<ImapSession>);

    impl MarksMessages for ASessionTheTestOpened {
        async fn open(
            &self,
            folder: &str,
        ) -> Result<crate::service::protocols::imap::MailboxStatus> {
            self.0.lock().await.select_folder(folder).await
        }

        async fn set_flag(&self, folder: &str, uid: u32, flag: &str, on: bool) -> Result<()> {
            let mut session = self.0.lock().await;
            if session.selected_folder() != Some(folder) {
                session.select_folder(folder).await?;
            }
            session.set_flag(uid, flag, on).await
        }
    }

    /// A server whose folders say this in PERMANENTFLAGS, or say nothing, and
    /// which answers a STORE holding `refusing` with NO.
    async fn a_server_keeping(
        permanent: Option<&'static str>,
        refusing: Option<&'static str>,
    ) -> Conversation {
        conversing("* OK loopback ready\r\n", move |line| {
            let tag = line.split_whitespace().next().unwrap_or("*").to_string();
            let said = line.to_uppercase();
            if let Some(refusing) = refusing
                && said.contains(&refusing.to_uppercase())
            {
                return Turn::Say(format!("{tag} NO this folder does not take that\r\n"));
            }
            match said.split_whitespace().nth(1).unwrap_or_default() {
                "CAPABILITY" => Turn::Say(format!("* CAPABILITY IMAP4rev1\r\n{tag} OK done\r\n")),
                "LOGIN" | "AUTHENTICATE" => Turn::Say(format!("{tag} OK signed in\r\n")),
                "ID" => Turn::Say(format!("* ID NIL\r\n{tag} OK done\r\n")),
                "SELECT" | "EXAMINE" => Turn::Say(format!(
                    "* FLAGS (\\Answered \\Flagged \\Deleted \\Seen \\Draft)\r\n{}* 2 EXISTS\r\n\
                     * OK [UIDVALIDITY 1] valid\r\n{tag} OK [READ-WRITE] open\r\n",
                    permanent
                        .map(|flags| format!("* OK [PERMANENTFLAGS ({flags})] kept\r\n"))
                        .unwrap_or_default()
                )),
                "UID" | "NOOP" | "LOGOUT" => Turn::Say(format!("{tag} OK done\r\n")),
                _ => Turn::Say(format!("{tag} BAD unscripted\r\n")),
            }
        })
        .await
    }

    async fn signed_in(server: &Conversation) -> ASessionTheTestOpened {
        ASessionTheTestOpened(tokio::sync::Mutex::new(
            crate::service::protocols::imap::against_a_server_that_answers::signed_in_to(server)
                .await,
        ))
    }

    #[tokio::test]
    async fn test_a_server_that_keeps_keywords_has_not_junk_taken_off_before_junk_goes_on() {
        let server = a_server_keeping(Some("\\Seen \\Deleted \\*"), None).await;
        let session = signed_in(&server).await;

        let marked = marking(&session, "INBOX", &[7, 9])
            .await
            .expect("the server took both");

        assert_eq!(marked, Marked::Kept);
        let transcript = server.transcript().await;
        for uid in [7, 9] {
            let off = server
                .when_told(&format!("UID STORE {uid} -FLAGS ($NotJunk)"))
                .await;
            let on = server
                .when_told(&format!("UID STORE {uid} +FLAGS ($Junk)"))
                .await;
            let (Some(off), Some(on)) = (off, on) else {
                panic!("message {uid} was not told both: {transcript:?}");
            };
            assert!(
                off < on,
                "$Junk went on message {uid} before $NotJunk came off, so for a moment it \
                 carried both, which RFC 9051 reads as neither: {transcript:?}"
            );
        }
    }

    #[tokio::test]
    async fn test_a_folder_that_names_the_junk_keyword_itself_is_marked() {
        let server = a_server_keeping(Some("\\Seen $Junk $NotJunk"), None).await;
        let session = signed_in(&server).await;

        let marked = marking(&session, "INBOX", &[4])
            .await
            .expect("the server took it");

        assert_eq!(marked, Marked::Kept);
        assert!(server.was_told("UID STORE 4 +FLAGS ($Junk)").await);
    }

    #[tokio::test]
    async fn test_a_folder_that_keeps_no_junk_keyword_is_sent_no_store() {
        for permanent in [Some("\\Seen \\Deleted"), None] {
            let server = a_server_keeping(permanent, None).await;
            let session = signed_in(&server).await;

            let marked = marking(&session, "INBOX", &[7])
                .await
                .expect("nothing was asked that could fail");

            assert_eq!(marked, Marked::NotKept, "{permanent:?}");
            assert!(
                server.was_told("SELECT").await,
                "the folder was never opened, so nobody asked what it keeps"
            );
            assert!(
                !server.was_told("STORE").await,
                "a keyword was sent to a folder that says it keeps none ({permanent:?}): {:?}",
                server.transcript().await
            );
        }
    }

    #[tokio::test]
    async fn test_a_refused_store_is_an_error_carrying_the_servers_words() {
        let server = a_server_keeping(Some("\\*"), Some("+FLAGS ($Junk)")).await;
        let session = signed_in(&server).await;

        let why = marking(&session, "INBOX", &[7])
            .await
            .expect_err("the server said no to the mark");

        assert!(
            why.to_string().contains("this folder does not take that"),
            "the server's own words were lost: {why}"
        );
    }
}

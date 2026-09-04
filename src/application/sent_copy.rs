//! Keeping a copy of a message that has gone out.
//!
//! A message that has been sent and filed nowhere is gone. There is no draft
//! left, the outbox row was removed the moment the server took it, and nothing
//! on this computer says it was ever written. So the copy is filed in three
//! steps, in this order:
//!
//! 1. On the server, into the account's Sent folder, so it is there on every
//!    device. That is what an IMAP account has always done.
//! 2. On this computer, when the server refuses it. Before this, a refusal was
//!    a line in a log and the message existed nowhere at all.
//! 3. On this computer as well as the server, when somebody asks for that.
//!
//! An account whose mail is collected over POP has no server folder to file
//! anything in, so step 1 does not apply and its copy is always the one here.
//!
//! # Why a copy filed here needs marking
//!
//! For an IMAP account the copy goes into the same Sent folder the sync fills.
//! That is the right place: sent mail belongs where the reader looks for sent
//! mail. It also means a row the server has never heard of sits in a folder the
//! sync reconciles against the server, and the sync's rule for such a row is to
//! delete it. [`crate::data::message_cache::MessageCache::file_message_here`]
//! marks it, and the marker is what the forget step checks.

use crate::data::account::Account;
use crate::data::message_cache::MessageCache;

/// Somewhere a copy of a sent message can be put that is not this computer.
///
/// One method, which is the whole of what filing a copy asks of a mail server.
/// Named for what it does rather than for IMAP, so the decisions around it can
/// be run in a test: what to file, where, what to do when it is refused and
/// what to say afterwards are all decisions, and none of them could be reached
/// before without a server.
pub(crate) trait FilesACopy {
    /// Put a copy of the message in the named folder. The string in the error
    /// is what the person is shown, so it says what the server said.
    async fn keep_a_copy(&self, folder: &str, raw: &[u8]) -> std::result::Result<(), String>;
}

/// What became of the copy of a message that has gone out.
///
/// Five answers rather than success and failure, because each is a different
/// thing for the person to hear and two of them are not failures at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kept {
    /// Filed in the account's Sent folder at the server.
    OnTheServer,
    /// Filed on this computer, which is where this account's sent mail lives.
    Here,
    /// Filed in both, because somebody asked for a copy here as well.
    OnTheServerAndHere,
    /// The server would not take it, so it was kept here instead.
    HereBecauseTheServerRefused(String),
    /// It could not be kept anywhere, and this is why.
    Nowhere(String),
}

/// What to say when an account has no Sent folder yet.
///
/// A brand new account that has never checked for mail has no folder list, so
/// there is nowhere to file a copy and nothing to name. It says what to do next
/// rather than only what went wrong.
pub const NO_SENT_FOLDER_YET: &str = "this account has not learned where its Sent folder is yet. Check for mail once, \
     and copies of what you send will be filed there from then on";

/// The label on the setting that keeps a copy here as well.
///
/// It says what happens rather than naming a folder or a protocol, and it
/// carries its own mnemonic. Kept here rather than written into the settings
/// screen so the words can be checked by a test. No test here, or anywhere,
/// can show that a screen reader read them out.
pub const KEEP_A_COPY_LABEL: &str =
    "&Keep a copy of sent mail on this computer, even when the server saves one";

/// The consequence of turning it on, which is what nobody can see coming.
///
/// Goes into the accessible description rather than the label, because it is
/// the answer to "and then what", not the name of the control.
pub const KEEP_A_COPY_CONSEQUENCE: &str = "Sent will then list each message twice: once saved here as you send it, and once from \
     the server on the next check for mail.";

impl Kept {
    /// One sentence saying where the copy is, for the status line.
    ///
    /// Never empty. This is read aloud, and a status line that goes blank is
    /// indistinguishable from one that never spoke.
    pub fn what_happened(&self) -> String {
        match self {
            Self::OnTheServer => "A copy was saved in Sent.".to_string(),
            Self::Here => "A copy was saved in Sent on this computer.".to_string(),
            Self::OnTheServerAndHere => {
                "A copy was saved in Sent, and another on this computer.".to_string()
            }
            Self::HereBecauseTheServerRefused(reason) => format!(
                "The server would not save a copy in Sent, so one was saved on this computer \
                 instead. The server said: {}",
                ended(reason)
            ),
            Self::Nowhere(reason) => format!(
                "The message was sent, but no copy of it could be saved anywhere: {}",
                ended(reason)
            ),
        }
    }

    /// Whether this is worth interrupting the person with.
    ///
    /// Three of the five are the ordinary end of sending a message and saying
    /// them every time is noise that buries the two that matter.
    pub fn needs_saying(&self) -> bool {
        matches!(
            self,
            Self::HereBecauseTheServerRefused(_) | Self::Nowhere(_)
        )
    }
}

/// A reason with a full stop after it.
///
/// A reason comes from a server or from the file system and ends however it
/// ends. Read aloud, a sentence running into the next one without a stop is
/// heard as one long sentence, so the stop is put here rather than hoped for.
fn ended(reason: &str) -> String {
    let reason = reason.trim_end();
    if reason.ends_with(['.', '!', '?']) {
        return reason.to_string();
    }
    format!("{reason}.")
}

/// Where a copy of this account's sent mail goes.
///
/// Worked out once, before any message is sent, because it is the same answer
/// for every message in the queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
    /// The account has no Sent folder yet, which is one that has never checked
    /// for mail. There is nowhere to put a copy and nothing to name.
    NotKnownYet,
    /// A folder on this computer, which is what an account collecting its mail
    /// over POP has. Nothing is offered to a server, and this is the only copy
    /// there will ever be.
    OnThisComputer(String),
    /// A folder at the account's own mail server.
    AtTheServer(String),
}

impl Destination {
    /// The folder at the account's own server, where there is one.
    ///
    /// One place answers "is there a server to offer a copy to". It is asked
    /// twice, by the signing in and by the offering, and two answers to it is a
    /// queue that signs in to say nothing, or one that files nothing having
    /// signed in.
    pub(crate) fn at_the_server(&self) -> Option<&str> {
        match self {
            Self::AtTheServer(folder) => Some(folder),
            _ => None,
        }
    }
}

/// Where this account's sent mail is filed.
pub fn destination(cache: &MessageCache, account: &Account) -> Destination {
    if let Some(here) = crate::application::local_folders::local_sent(account.protocol()) {
        return Destination::OnThisComputer(here);
    }
    let at_the_server = cache
        .get_folders_for_account(&account.id)
        .ok()
        .and_then(|folders| {
            folders.into_iter().find(|folder| {
                crate::common::types::FolderType::from_stored(&folder.folder_type)
                    == crate::common::types::FolderType::Sent
            })
        });
    match at_the_server {
        Some(folder) => Destination::AtTheServer(folder.path),
        None => Destination::NotKnownYet,
    }
}

/// What the account's mail server said about keeping the copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerSaid {
    /// There was no server to ask.
    Nothing,
    /// It has the copy.
    ItHasIt,
    /// It would not take it, and this is what it said.
    No(String),
}

/// Offer the copy to the account's mail server, where there is one.
///
/// The one step here that reaches a network, and the only one. It is split from
/// the writing below so that the connection this program holds to its own
/// database is never held across the wait for a server: that connection is not
/// safe to share between threads, and a task holding it across an await cannot
/// be handed to the runtime at all.
///
/// Called only after the message has actually been sent, and it can send
/// nothing itself. `keep_a_copy` goes through the same account and the same
/// permission check the send did.
pub(crate) async fn offer_to_the_server<F: FilesACopy>(
    server: &F,
    goes_to: &Destination,
    raw: &[u8],
) -> ServerSaid {
    let Some(folder) = goes_to.at_the_server() else {
        return ServerSaid::Nothing;
    };
    match server.keep_a_copy(folder, raw).await {
        Ok(()) => ServerSaid::ItHasIt,
        Err(refusal) => ServerSaid::No(refusal),
    }
}

/// The session this queue's copies go through.
///
/// Opened once for a queue rather than once per message: filing a copy used to
/// sign in again for every message sent, so a queue of fifty was fifty
/// sign-ins, which some providers turn down.
///
/// Three states because two of them are not failures. An account that keeps its
/// sent mail on this computer has nothing to sign in to, and one that could not
/// sign in still keeps every copy here and says why.
pub(crate) enum FilingSession {
    /// There is no server to offer anything to.
    NotNeeded,
    /// Signed in, and copies go through it.
    Open(crate::application::mail_controller::MailController),
    /// The account has a server and it could not be reached. This is what it
    /// said, and it is what the person is read out.
    CouldNotSignIn(String),
}

/// Sign in for this queue, if there is anywhere to sign in to.
pub(crate) async fn a_session_for(goes_to: &Destination, account: &Account) -> FilingSession {
    if goes_to.at_the_server().is_none() {
        return FilingSession::NotNeeded;
    }
    match crate::application::mail_session::a_session_at(account).await {
        Ok(session) => FilingSession::Open(session),
        Err(reason) => FilingSession::CouldNotSignIn(reason),
    }
}

/// Offer this copy through the queue's session.
///
/// A sign-in that failed is a refusal for every message in the queue, and a
/// refusal is what keeps the copy on this computer instead of losing it.
pub(crate) async fn offer_through(
    session: &FilingSession,
    goes_to: &Destination,
    raw: &[u8],
) -> ServerSaid {
    match session {
        FilingSession::NotNeeded => ServerSaid::Nothing,
        FilingSession::Open(session) => {
            offer_to_the_server(&ServerCopy { session }, goes_to, raw).await
        }
        FilingSession::CouldNotSignIn(reason) => ServerSaid::No(reason.clone()),
    }
}

impl FilingSession {
    /// Close it, if one was ever opened.
    ///
    /// A session left behind is a connection still held at the server after
    /// every send.
    pub(crate) async fn close(self) {
        if let Self::Open(session) = self {
            let _ = session.disconnect_imap().await;
        }
    }
}

/// Write whatever copy belongs on this computer, and say where the copy is.
///
/// Everything here touches this computer and nothing else.
///
/// `also_here` is the person's setting and is not a permission: the most it can
/// do is write one more row to the database.
pub fn file_the_copy(
    cache: &MessageCache,
    account: &Account,
    goes_to: &Destination,
    said: &ServerSaid,
    also_here: bool,
    raw: &[u8],
) -> Kept {
    let folder = match goes_to {
        Destination::NotKnownYet => return Kept::Nowhere(NO_SENT_FOLDER_YET.to_string()),
        Destination::OnThisComputer(folder) | Destination::AtTheServer(folder) => folder,
    };

    match said {
        // No server was asked, so the copy here is the only one there will be.
        ServerSaid::Nothing => match file_here(cache, account, folder, raw) {
            Ok(()) => Kept::Here,
            Err(reason) => Kept::Nowhere(reason),
        },
        ServerSaid::ItHasIt if !also_here => Kept::OnTheServer,
        ServerSaid::ItHasIt => match file_here(cache, account, folder, raw) {
            Ok(()) => Kept::OnTheServerAndHere,
            // The server has the message, so nothing has been lost and telling
            // somebody their mail is in danger would be wrong. The extra copy
            // they asked for is missing, which is worth a line in the log.
            Err(reason) => {
                tracing::warn!(
                    "The server saved a copy, but a second one could not be kept here: {reason}"
                );
                Kept::OnTheServer
            }
        },
        ServerSaid::No(refusal) => match file_here(cache, account, folder, raw) {
            Ok(()) => Kept::HereBecauseTheServerRefused(refusal.clone()),
            Err(reason) => Kept::Nowhere(format!(
                "the server refused it ({refusal}), and it could not be kept on this computer \
                 either ({reason})"
            )),
        },
    }
}

/// Put a copy of a sent message into a folder on this computer.
///
/// Marked read, because a message somebody wrote themselves is not unread mail
/// waiting to be dealt with and an unread count that climbs every time they
/// send something is a count nobody can use. Stored with its body, since for
/// this row there is nowhere to fetch one from later.
fn file_here(
    cache: &MessageCache,
    account: &Account,
    folder: &str,
    raw: &[u8],
) -> std::result::Result<(), String> {
    let parsed = crate::service::mime::parse(raw).map_err(|e| e.to_string())?;
    // Asked which account the folder's row is under rather than assuming this
    // one. Since D-18 the Sent folder on this computer is shared and its row is
    // under the reserved id, so looking it up under the account finds nothing
    // and the copy of a sent message is not filed at all.
    let owner = crate::application::local_folders::stored_under(folder, &account.id);
    let row = cache
        .get_folder(owner, folder)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("there is no {folder} folder for this account"))?;

    // A folder the server also fills needs a number the server will never
    // issue. See `next_reserved_uid`: taking the next one upward hides a real
    // message forever and lets a later sync write over this copy. Which end to
    // count from is asked of the one place that answers it, because a rule
    // filing a message and an import ask the same question and the three must
    // not differ.
    let uid = cache
        .next_uid_for_filing(row.id)
        .map_err(|e| e.to_string())?;

    // The moment it was sent, for a message with no usable Date header of its
    // own. An empty date sorts the copy to whichever end of Sent nobody looks
    // at, which is the same as losing it for anybody reading by ear.
    let filed_at = chrono::Utc::now().to_rfc3339();
    // Every column the two share is decided in one place, so a sent copy
    // and an imported message cannot come to differ about them. Read,
    // because the person whose Sent folder this lands in wrote it: filed
    // unread it puts a bold row and a count on a folder of their own mail.
    let message = crate::application::filing::a_row_filed_here(
        &parsed,
        row.id,
        uid,
        raw.len(),
        crate::application::filing::AlreadyRead::Yes,
        &filed_at,
    );

    let stored = cache
        .file_message_here(&message)
        .map_err(|e| e.to_string())?;
    cache
        .save_message_body(
            stored,
            parsed.body_plain.as_deref(),
            parsed.body_html.as_deref(),
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// The account's Sent folder at its own mail server.
///
/// The append and nothing else. It used to sign in and disconnect around every
/// message, which is where the sign-in went unmeasured: nothing could construct
/// this without a real account and a real server. The session comes from
/// [`a_session_for`] now, once for the whole queue.
///
/// Flagged `\Seen`, for the same reason the copy kept on this computer is
/// marked read: a message somebody wrote themselves is not unread mail waiting
/// to be dealt with, and an unread count that climbs every time they send
/// something is a count nobody can use.
pub(crate) struct ServerCopy<'a> {
    pub session: &'a crate::application::mail_controller::MailController,
}

impl FilesACopy for ServerCopy<'_> {
    async fn keep_a_copy(&self, folder: &str, raw: &[u8]) -> std::result::Result<(), String> {
        let already_read = format!("({})", crate::service::protocols::imap::flag::SEEN);
        self.session
            .append_message(folder, Some(&already_read), raw)
            .await
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::temp_home::TempHome;
    use crate::data::message_cache::{CachedFolder, MessageCache};

    /// A place to file a copy that always refuses.
    struct Refusing;

    impl FilesACopy for Refusing {
        async fn keep_a_copy(&self, _folder: &str, _raw: &[u8]) -> std::result::Result<(), String> {
            Err("the mailbox is over quota".to_string())
        }
    }

    /// A place to file a copy that takes it, and remembers what it was given.
    #[derive(Default)]
    struct Accepting {
        given: std::cell::RefCell<Vec<String>>,
    }

    impl FilesACopy for Accepting {
        async fn keep_a_copy(&self, folder: &str, _raw: &[u8]) -> std::result::Result<(), String> {
            self.given.borrow_mut().push(folder.to_string());
            Ok(())
        }
    }

    fn a_cache(sent: bool) -> (TempHome<MessageCache>, crate::data::account::Account) {
        let cache = TempHome::named("wixen_sent_", |dir| {
            MessageCache::new(dir.to_path_buf(), None).expect("a cache")
        });
        let mut account =
            crate::data::account::Account::new("Work".into(), "me@example.com".into());
        account.id = "acct".to_string();
        if sent {
            cache
                .save_folder(&CachedFolder {
                    id: 0,
                    account_id: "acct".into(),
                    name: "Sent".into(),
                    path: "Sent".into(),
                    folder_type: "Sent".into(),
                    unread_count: 0,
                    total_count: 0,
                })
                .expect("a folder");
        }
        (cache, account)
    }

    fn a_message() -> Vec<u8> {
        concat!(
            "From: me@example.com\r\n",
            "To: you@example.com\r\n",
            "Subject: The quarterly figures\r\n",
            "Date: Tue, 4 Aug 2026 10:00:00 +0000\r\n",
            "Message-ID: <sent-1@example.com>\r\n",
            "\r\n",
            "Here they are.\r\n",
        )
        .as_bytes()
        .to_vec()
    }

    /// The two steps the send loop runs, in the order it runs them.
    ///
    /// They are two because the connection to this program's own database
    /// cannot be held across the wait for a server. What the send loop composes
    /// is these two lines.
    fn keeping<F: FilesACopy>(
        server: &F,
        cache: &MessageCache,
        account: &Account,
        also_here: bool,
        raw: &[u8],
    ) -> Kept {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("a runtime")
            .block_on(async {
                let goes_to = destination(cache, account);
                let said = offer_to_the_server(server, &goes_to, raw).await;
                file_the_copy(cache, account, &goes_to, &said, also_here, raw)
            })
    }

    #[tokio::test]
    async fn test_a_send_that_could_not_sign_in_keeps_the_copy_here_and_says_why() {
        // The sign-in used to happen inside the one step that reaches a server,
        // where nothing could reach it. A queue whose account cannot sign in
        // still has to keep every copy on this computer and say so, because the
        // message has gone and this is the only record left of it.
        let (cache, account) = a_cache(true);
        let raw = a_message();
        let goes_to = Destination::AtTheServer("Sent".to_string());

        let said = offer_through(
            &FilingSession::CouldNotSignIn("the server could not be reached".to_string()),
            &goes_to,
            &raw,
        )
        .await;

        assert_eq!(
            said,
            ServerSaid::No("the server could not be reached".to_string())
        );
        let kept = file_the_copy(&cache, &account, &goes_to, &said, false, &raw);
        let Kept::HereBecauseTheServerRefused(reason) = &kept else {
            panic!("the copy was not kept here: {kept:?}");
        };
        assert!(reason.contains("could not be reached"), "{reason}");
        assert!(kept.needs_saying());

        let row = the_only_row(&cache);
        assert_eq!(row.subject, "The quarterly figures");
        let body = cache
            .get_message_body(row.id)
            .expect("read the body")
            .expect("a body was stored");
        assert!(
            body.body_plain
                .unwrap_or_default()
                .contains("Here they are"),
            "the copy was filed without its body"
        );
    }

    #[tokio::test]
    async fn test_no_session_is_opened_for_an_account_that_keeps_its_sent_mail_here() {
        // An account collecting its mail over POP has no server folder to file
        // anything in, so signing in to one would be a connection made to say
        // nothing. The same for an account that has not learnt where its Sent
        // folder is: there is nowhere to put a copy and nothing to name.
        let (_cache, account) = a_cache(false);
        let raw = a_message();

        // The folder a POP account really files its sent mail in, taken from
        // the one place that answers that, rather than spelled out again here.
        let here =
            crate::application::local_folders::local_sent(crate::common::types::Protocol::Pop3)
                .expect("an account that keeps its sent mail here has somewhere to keep it");

        for goes_to in [Destination::OnThisComputer(here), Destination::NotKnownYet] {
            let session = a_session_for(&goes_to, &account).await;
            assert!(
                matches!(session, FilingSession::NotNeeded),
                "a session was opened for {goes_to:?}"
            );
            assert_eq!(
                offer_through(&session, &goes_to, &raw).await,
                ServerSaid::Nothing
            );
            session.close().await;
        }
    }

    #[test]
    fn test_a_copy_the_server_refuses_is_kept_on_this_computer() {
        // The case the fallback exists for. Before this, a refused APPEND was a
        // warning in a log and the sent message existed nowhere at all.
        let (cache, account) = a_cache(true);
        let kept = keeping(&Refusing, &cache, &account, false, &a_message());

        assert!(
            matches!(kept, Kept::HereBecauseTheServerRefused(_)),
            "the copy was not kept here: {kept:?}"
        );

        let folder = cache
            .get_folder("acct", "Sent")
            .expect("read the folder")
            .expect("the Sent folder");
        let listed = cache
            .get_message_list(folder.id, "acct")
            .expect("list the folder");
        assert_eq!(listed.len(), 1, "the Sent folder holds {listed:?}");
        assert_eq!(listed[0].subject, "The quarterly figures");
        let body = cache
            .get_message_body(listed[0].id)
            .expect("read the body")
            .expect("a body was stored");
        assert!(
            body.body_plain
                .unwrap_or_default()
                .contains("Here they are"),
            "the copy was filed without its body"
        );
    }

    #[test]
    fn test_an_account_with_no_sent_folder_says_so_rather_than_going_quiet() {
        // The most likely of the five ways the copy used to be lost: somebody
        // adds an account and sends before the first check for mail, so there
        // is no folder list and no Sent folder in it. The old code turned that
        // into `Ok(())` and said nothing at all.
        let (cache, account) = a_cache(false);

        let kept = keeping(&Accepting::default(), &cache, &account, false, &a_message());

        let Kept::Nowhere(reason) = &kept else {
            panic!("a copy with nowhere to go was reported as {kept:?}");
        };
        assert!(
            reason.contains("Check for mail once"),
            "the reason does not say what to do next: {reason}"
        );
        assert!(
            kept.needs_saying(),
            "losing the only copy of a sent message passed over in silence"
        );
        assert!(!kept.what_happened().trim().is_empty());
    }

    #[test]
    fn test_every_answer_is_a_sentence_somebody_can_hear() {
        // What `what_happened` returns goes to the status line, which is read
        // aloud. A blank one is indistinguishable from one that never spoke.
        for answer in [
            Kept::OnTheServer,
            Kept::Here,
            Kept::OnTheServerAndHere,
            Kept::HereBecauseTheServerRefused("over quota".into()),
            Kept::Nowhere("no folder".into()),
        ] {
            let said = answer.what_happened();
            assert!(!said.trim().is_empty(), "{answer:?} says nothing");
            assert!(said.ends_with('.'), "{said:?} is not a sentence");
        }
        // The three ordinary endings are not worth interrupting anybody with,
        // and saying them every time buries the two that are.
        assert!(!Kept::OnTheServer.needs_saying());
        assert!(!Kept::Here.needs_saying());
        assert!(!Kept::OnTheServerAndHere.needs_saying());
    }

    #[test]
    fn test_a_reason_that_already_ends_in_a_full_stop_does_not_get_a_second_one() {
        // The same rule, and the same untested branch, as the draft path. The
        // routine that puts the stop on is written out once in each of the two
        // files, so a fix to one of them leaves the other stumbling, and the
        // only thing that would notice is a test in each.
        //
        // Neither can be reached by changing the code in small ways and seeing
        // what the suite notices: the early return is a call to `ends_with` on
        // a list of three characters, and the trim beside it takes no
        // arguments. A mutation run produces no mutant for either.
        for said_by_the_server in ["the mailbox is over quota.", "the mailbox is over quota. "] {
            let kept = Kept::HereBecauseTheServerRefused(said_by_the_server.to_string());

            let said = kept.what_happened();

            assert!(
                said.ends_with("the mailbox is over quota."),
                "the server's own words were punctuated again: {said}"
            );
        }
    }

    #[test]
    fn test_a_reply_kept_here_still_threads_with_what_it_answered() {
        // The chain the recipient's client threads on has to be the chain this
        // one stores, or a reply somebody sent is a message on its own in their
        // own Sent folder. The old local filing wrote nothing into this column.
        let (cache, account) = a_cache(true);
        let raw = concat!(
            "From: me@example.com\r\n",
            "To: you@example.com\r\n",
            "Subject: Re: Lunch\r\n",
            "Date: Tue, 4 Aug 2026 10:00:00 +0000\r\n",
            "Message-ID: <reply-1@example.com>\r\n",
            "In-Reply-To: <parent@example.com>\r\n",
            "References: <root@example.com> <parent@example.com>\r\n",
            "Reply-To: desk@example.com\r\n",
            "\r\n",
            "One o'clock.\r\n",
        )
        .as_bytes();

        keeping(&Refusing, &cache, &account, false, raw);

        let row = the_only_row(&cache);
        assert_eq!(
            row.refs_header.as_deref(),
            Some("root@example.com parent@example.com"),
            "the copy does not name what it answered"
        );
        // Where the sender asked replies to go. Dropping it sent a reply to
        // your own sent copy to your own address.
        assert_eq!(row.reply_to.as_deref(), Some("desk@example.com"));
        assert_eq!(row.message_id, "reply-1@example.com");
        assert!(
            row.read,
            "a message somebody wrote themselves is not unread"
        );
    }

    #[test]
    fn test_a_message_that_answers_only_one_other_still_names_it() {
        // In-Reply-To with no References at all, which is what a great many
        // clients send. Reading only References would store nothing.
        let (cache, account) = a_cache(true);
        let raw = with_headers(&["In-Reply-To: <parent@example.com>"]);

        keeping(&Refusing, &cache, &account, false, &raw);

        assert_eq!(
            the_only_row(&cache).refs_header.as_deref(),
            Some("parent@example.com")
        );
    }

    #[test]
    fn test_a_message_answering_nothing_stores_no_chain_rather_than_an_empty_one() {
        // An empty string here is not the same as nothing: it is a chain of no
        // ancestors, which reads back as a reply to a message with no name.
        let (cache, account) = a_cache(true);

        keeping(&Refusing, &cache, &account, false, &a_message());

        assert_eq!(the_only_row(&cache).refs_header, None);
        assert_eq!(the_only_row(&cache).reply_to, None);
    }

    #[test]
    fn test_a_parent_already_named_in_the_chain_is_not_named_twice() {
        // Some clients repeat the message answered at the end of References.
        let (cache, account) = a_cache(true);
        let raw = with_headers(&[
            "In-Reply-To: <parent@example.com>",
            "References: <root@example.com> <parent@example.com>",
        ]);

        keeping(&Refusing, &cache, &account, false, &raw);

        assert_eq!(
            the_only_row(&cache).refs_header.as_deref(),
            Some("root@example.com parent@example.com")
        );
    }

    #[test]
    fn test_a_copy_kept_here_is_sorted_where_the_reader_will_find_it() {
        // A blank date sorts the copy to whichever end of Sent nobody looks at,
        // which for somebody reading by ear is the same as losing it.
        let (cache, account) = a_cache(true);
        let no_date = concat!(
            "From: me@example.com\r\n",
            "To: you@example.com\r\n",
            "Subject: No date on this one\r\n",
            "\r\n",
            "Body.\r\n",
        )
        .as_bytes();

        keeping(&Refusing, &cache, &account, false, no_date);

        let row = the_only_row(&cache);
        assert!(!row.date.trim().is_empty(), "the copy has no date at all");
        assert!(
            chrono::DateTime::parse_from_rfc3339(&row.date).is_ok(),
            "{} is not a date the listing can sort on",
            row.date
        );
        // A message with no identifier of its own stores an empty one rather
        // than inventing one. Pinned because two such copies in one folder are
        // then indistinguishable to a lookup by identifier.
        assert_eq!(row.message_id, "");
    }

    #[test]
    fn test_a_receipt_the_sender_asked_for_is_not_turned_back_on_themselves() {
        // On an outgoing message that header is this sender asking their
        // recipient for a receipt. Carrying it onto their own copy would have
        // the client offer to tell them they had read their own mail.
        let (cache, account) = a_cache(true);
        let raw = with_headers(&["Disposition-Notification-To: me@example.com"]);

        keeping(&Refusing, &cache, &account, false, &raw);

        assert_eq!(the_only_row(&cache).receipt_to, None);
    }

    #[test]
    fn test_a_copy_kept_here_names_everybody_it_was_copied_to() {
        // Who else saw the message is part of what the sent copy is for, and
        // for somebody reading the list by ear it is the only way to find out.
        // Both halves, because an empty string here is not the same as
        // nothing: it reads back as a copy sent to a person with no address.
        let (cache, account) = a_cache(true);
        let raw = with_headers(&["Cc: her@example.com, him@example.com"]);

        keeping(&Refusing, &cache, &account, false, &raw);

        assert_eq!(
            the_only_row(&cache).cc.as_deref(),
            Some("her@example.com, him@example.com"),
            "the copy does not say who else it went to"
        );

        let (plain, plain_account) = a_cache(true);
        keeping(&Refusing, &plain, &plain_account, false, &a_message());
        assert_eq!(
            the_only_row(&plain).cc,
            None,
            "a message copied to nobody stored an empty list of people"
        );
    }

    #[test]
    fn test_a_copy_of_a_message_that_carried_a_file_says_it_carried_one() {
        // The flag the message list announces the row with. Getting it wrong
        // either way misleads: an attachment nobody is told about, or one
        // announced on a message that has none.
        let (cache, account) = a_cache(true);
        let with_a_file = concat!(
            "From: me@example.com\r\n",
            "To: you@example.com\r\n",
            "Subject: The quarterly figures\r\n",
            "Date: Tue, 4 Aug 2026 10:00:00 +0000\r\n",
            "Message-ID: <sent-2@example.com>\r\n",
            "Content-Type: multipart/mixed; boundary=\"b\"\r\n",
            "\r\n",
            "--b\r\n",
            "Content-Type: text/plain\r\n",
            "\r\n",
            "See attached.\r\n",
            "--b\r\n",
            "Content-Type: application/pdf; name=\"figures.pdf\"\r\n",
            "Content-Disposition: attachment; filename=\"figures.pdf\"\r\n",
            "\r\n",
            "%PDF-1.4 not really\r\n",
            "--b--\r\n",
        )
        .as_bytes();

        keeping(&Refusing, &cache, &account, false, with_a_file);

        assert!(
            the_only_row(&cache).has_attachments,
            "the copy of a message that carried a file says it carried none"
        );

        let (plain, plain_account) = a_cache(true);
        keeping(&Refusing, &plain, &plain_account, false, &a_message());
        assert!(
            !the_only_row(&plain).has_attachments,
            "a message with no attachment would be announced as carrying one"
        );
    }

    #[test]
    fn test_a_reply_to_a_message_this_program_sent_joins_its_conversation() {
        // The whole reason sent mail needed an identifier, measured end to
        // end: a message really sent, the copy really filed, and the reply
        // path's own function asked what a reply to it would carry. With no
        // identifier on the way out the copy's column is empty, that function
        // answers nothing, and replying to your own sent mail starts a new
        // conversation in the recipient's client.
        let (cache, account) = a_cache(true);
        let raw = a_message_this_program_really_sent();

        keeping(&Refusing, &cache, &account, false, &raw);

        let row = the_only_row(&cache);
        assert!(
            !row.message_id.is_empty(),
            "the copy in Sent has no identifier, so a reply to it starts a new conversation"
        );
        let reply =
            crate::application::threading::continuing(&row.message_id, row.refs_header.as_deref())
                .expect("a reply to a message with an identifier continues its conversation");
        assert!(reply.in_reply_to.contains(&row.message_id), "{reply:?}");
        assert!(reply.references.ends_with(&reply.in_reply_to), "{reply:?}");
    }

    /// The bytes of a message sent through the ordinary send path.
    ///
    /// A loopback server rather than a message typed out here, because what is
    /// being measured is what this program really writes on the way out. Bytes
    /// written by hand would carry whatever header the test remembered to put
    /// in them and could pass with the send path writing none.
    fn a_message_this_program_really_sent() -> Vec<u8> {
        use crate::service::protocols::smtp::against_a_server_that_answers::{
            an_smtp_server, pointed_at,
        };

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("a runtime")
            .block_on(async {
                let server = an_smtp_server().await;
                let client = crate::service::protocols::smtp::SmtpClient::allowed_to_send(
                    pointed_at(&server),
                )
                .expect("a client");
                let message = crate::service::protocols::smtp::Email::simple(
                    "me@example.com".to_string(),
                    "you@example.com".to_string(),
                    "The quarterly figures".to_string(),
                    "Here they are.".to_string(),
                );
                tokio::time::timeout(
                    crate::common::answering::LONG_ENOUGH,
                    client.send_email(
                        message,
                        &crate::service::protocols::MailAuth::Password("hunter2".to_string()),
                    ),
                )
                .await
                .expect("the server never finished the exchange")
                .expect("the loopback server takes anything")
            })
    }

    #[test]
    fn test_nothing_extra_is_kept_here_when_nobody_asked_for_it() {
        // The setting is off by default, and off means the Sent folder lists
        // each message once, from the server, exactly as it did before.
        let (cache, account) = a_cache(true);
        let server = Accepting::default();

        let kept = keeping(&server, &cache, &account, false, &a_message());

        assert_eq!(kept, Kept::OnTheServer);
        assert_eq!(*server.given.borrow(), vec!["Sent".to_string()]);
        assert!(
            rows(&cache).is_empty(),
            "a copy was written here that nobody asked for"
        );
    }

    #[test]
    fn test_keeping_a_copy_here_as_well_leaves_one_in_each_place() {
        // What the setting is for: the message is in Sent now, without waiting
        // for the next check for mail. The cost is that Sent lists it twice
        // once the server's own copy arrives, which the setting's own text says.
        let (cache, account) = a_cache(true);
        let server = Accepting::default();

        let kept = keeping(&server, &cache, &account, true, &a_message());

        assert_eq!(kept, Kept::OnTheServerAndHere);
        assert_eq!(*server.given.borrow(), vec!["Sent".to_string()]);
        let row = the_only_row(&cache);
        assert_eq!(row.subject, "The quarterly figures");
        assert!(cache.get_message_body(row.id).unwrap().is_some());
    }

    #[test]
    fn test_a_copy_kept_here_takes_a_number_the_server_will_never_issue() {
        // The part most likely to be got wrong later. A number one past the
        // highest is the number the server is about to hand out, and giving it
        // to a copy kept here hides a real message forever.
        let (cache, account) = a_cache(true);
        let folder = cache.get_folder("acct", "Sent").unwrap().unwrap();
        for uid in 1..=4 {
            cache
                .upsert_message(&crate::data::message_cache::IncomingMessage {
                    folder_id: folder.id,
                    uid,
                    ..from_the_server(folder.id, uid)
                })
                .unwrap();
        }

        keeping(&Refusing, &cache, &account, false, &a_message());

        let copy = rows(&cache)
            .into_iter()
            .find(|row| row.subject == "The quarterly figures")
            .expect("the copy");
        assert_eq!(copy.uid, u32::MAX);
    }

    #[test]
    fn test_an_account_whose_mail_is_collected_over_pop_files_its_copy_here() {
        // There is no server folder to offer it to, so the copy here is the
        // only one there will ever be, and it is numbered from the bottom
        // because no server numbers that folder.
        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
        let mut account =
            crate::data::account::Account::new("Home".into(), "me@example.com".into());
        account.id = "acct".to_string();
        account.protocol = crate::common::types::Protocol::Pop3.as_str().to_string();
        let path = crate::application::local_folders::local_sent(account.protocol())
            .expect("a POP account files sent mail here");
        // Under whoever owns it. Since D-18 the Sent folder on this computer is
        // shared, so its row is under the reserved id and a fixture writing it
        // under the account builds a row the application never writes.
        let owner = crate::application::local_folders::stored_under(&path, "acct").to_string();
        cache
            .save_folder(&CachedFolder {
                id: 0,
                account_id: owner.clone(),
                name: "Sent".into(),
                path: path.clone(),
                folder_type: "Sent".into(),
                unread_count: 0,
                total_count: 0,
            })
            .expect("a folder");
        let server = Accepting::default();

        let kept = keeping(&server, &cache, &account, false, &a_message());

        assert_eq!(kept, Kept::Here);
        assert!(
            server.given.borrow().is_empty(),
            "a POP account offered its sent mail to an IMAP server"
        );
        let folder = cache.get_folder(&owner, &path).unwrap().unwrap();
        let listed = cache.get_message_list(folder.id, &owner).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].uid, 1, "a folder no server numbers counts up");
    }

    #[test]
    fn test_a_copy_that_cannot_be_written_here_is_reported_rather_than_swallowed() {
        // A POP account whose folders have never been made: there is a path to
        // file the copy under and no folder behind it. The person is owed the
        // reason, because on this account there is no other copy anywhere.
        let dir = tempfile::tempdir().expect("a temporary folder");
        let cache = MessageCache::new(dir.path().to_path_buf(), None).expect("a cache");
        let mut account =
            crate::data::account::Account::new("Home".into(), "me@example.com".into());
        account.id = "acct".to_string();
        account.protocol = crate::common::types::Protocol::Pop3.as_str().to_string();

        let kept = keeping(&Refusing, &cache, &account, false, &a_message());

        let Kept::Nowhere(reason) = &kept else {
            panic!("a copy that went nowhere was reported as {kept:?}");
        };
        assert!(
            reason.contains("no") && reason.contains("folder"),
            "the reason does not say what was missing: {reason}"
        );
        assert!(kept.needs_saying());
    }

    #[test]
    fn test_the_setting_says_what_happens_rather_than_naming_a_mechanism() {
        // The label is what somebody hears when they arrow onto the box, and
        // the description is the part they cannot otherwise see coming: their
        // Sent folder will list everything twice.
        let spoken = crate::presentation::accessibility::names::name_from_label(KEEP_A_COPY_LABEL);
        assert_eq!(
            spoken,
            "Keep a copy of sent mail on this computer, even when the server saves one"
        );
        assert!(
            KEEP_A_COPY_LABEL.contains("&K"),
            "the box has no mnemonic, so it cannot be reached by keyboard from the tab"
        );
        assert!(
            KEEP_A_COPY_CONSEQUENCE.contains("twice"),
            "the description does not say what actually changes: {KEEP_A_COPY_CONSEQUENCE}"
        );
        for machinery in ["IMAP", "APPEND", "cache", "database", "uid"] {
            let said = format!("{KEEP_A_COPY_LABEL} {KEEP_A_COPY_CONSEQUENCE}");
            assert!(
                !said.to_lowercase().contains(&machinery.to_lowercase()),
                "the setting names {machinery}, which is a mechanism and not what happens"
            );
        }
    }

    #[test]
    fn test_the_one_thing_here_that_reaches_a_server_is_still_behind_the_gate() {
        // This unit adds no new outward write. It moves one: the command that
        // asks a mail server to keep a copy. Moving a write is exactly when its
        // gate gets left behind, so the two links in that chain are pinned
        // here rather than reasoned about.
        //
        // Read from the source, and that is now the weaker of two checks
        // rather than the only one. The command that keeps a copy is exercised
        // against a loopback server that records what it hears, so a session
        // which may not change anything is shown to say nothing rather than
        // merely to hold a line of code that asks. What this reads from the
        // source is the other half: that the asking is still in the function
        // at all, which no transcript can show.
        let imap = include_str!("../service/protocols/imap.rs");
        let append = imap
            .split_once("pub async fn append_message(")
            .expect("append_message is still the command that keeps a copy")
            .1;
        let body = append
            .split_once("\n    }")
            .expect("append_message has an end")
            .0;
        assert!(
            body.contains("self.may_i("),
            "the command that keeps a copy at a server no longer asks whether it may: {body}"
        );

        // The other link: nothing turns a session's permission on without
        // reading the account's own setting first.
        //
        // The production half of that file only, the way this test already
        // treats its own file below. The claim worth protecting is that no
        // place in the running program opens the gate on its own say-so.
        // Counting the test module as well would forbid ever measuring what
        // happens past an open gate, which is the only way any of these
        // commands can be exercised at all.
        //
        // Every occurrence, not the first. This used to insist there was
        // exactly one and then read the four lines above that one, so a second
        // opener added later would have been counted and never looked at,
        // which is worse than not counting at all. There are two: signing in
        // to a mail server, and signing in to a POP server.
        let controller = include_str!("mail_controller.rs")
            .split_once("#[cfg(test)]")
            .expect("the controller has tests")
            .0;
        let lines: Vec<&str> = controller.lines().collect();
        let opens_it: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.contains("allow_changes()"))
            .map(|(at, _)| at)
            .collect();
        assert_eq!(
            opens_it.len(),
            2,
            "the mail gate is opened somewhere other than the two sign-ins: {:?}",
            opens_it.iter().map(|at| lines[*at]).collect::<Vec<_>>()
        );
        for at in opens_it {
            let guard = lines[at.saturating_sub(4)..at].join(" ");
            assert!(
                guard.contains("allowed_for") && guard.contains(".mail"),
                "a session is allowed to change things without reading the account's \
                 own setting first: {guard}"
            );
        }

        // And there is one route to a mail server for the copies this program
        // files on somebody's behalf, in one place.
        //
        // It used to be here, and there was a second copy of the same six lines
        // in the draft path. Both are gone into one sign-in, so this module now
        // holds none. Counted in both directions: a module that grows its own
        // sign-in again is what this notices.
        let mine = include_str!("sent_copy.rs")
            .split_once("#[cfg(test)]")
            .expect("this module has tests")
            .0;
        assert_eq!(
            mine.matches(".connect_imap(").count(),
            0,
            "this module opens a session of its own again, and the one it should \
             be using is opened once for the whole queue"
        );

        // The one place has moved and there is still one of it. `mail_session`
        // used to build the connection itself; it now asks the controller,
        // which is what remembers the account so a connection that goes can be
        // made again. So the route is two named hops, and each has to be one:
        // one call asking to be signed in, and one call reaching a server.
        //
        // Both halves, because either alone can be satisfied by the other
        // moving. A file that stopped asking would pass a check that only
        // counted the far end, and a second dialler would pass one that only
        // counted the near end.
        let asking = include_str!("mail_session.rs")
            .split_once("#[cfg(test)]")
            .expect("the sign-in has tests")
            .0;
        assert_eq!(
            asking.matches(".sign_in_for(").count(),
            1,
            "the one sign-in the copies go through asks to be signed in by some \
             other number of routes than one"
        );
        let dialling = include_str!("mail_controller.rs")
            .split_once("#[cfg(test)]")
            .expect("the controller has tests")
            .0;
        assert_eq!(
            dialling.matches(".connect_imap(").count(),
            1,
            "the controller reaches a mail server by some other number of \
             routes than one"
        );
    }

    fn with_headers(extra: &[&str]) -> Vec<u8> {
        let mut raw = String::from(
            "From: me@example.com\r\nTo: you@example.com\r\nSubject: A message\r\n\
             Date: Tue, 4 Aug 2026 10:00:00 +0000\r\nMessage-ID: <one@example.com>\r\n",
        );
        for header in extra {
            raw.push_str(header);
            raw.push_str("\r\n");
        }
        raw.push_str("\r\nBody.\r\n");
        raw.into_bytes()
    }

    fn from_the_server(folder_id: i64, uid: u32) -> crate::data::message_cache::IncomingMessage {
        crate::data::message_cache::IncomingMessage {
            folder_id,
            uid,
            message_id: format!("<server-{uid}@example.com>"),
            subject: "From the server".to_string(),
            from_addr: "me@example.com".to_string(),
            to_addr: "you@example.com".to_string(),
            cc: None,
            reply_to: None,
            date: "2026-08-01T10:00:00+00:00".to_string(),
            internal_date: None,
            size_bytes: None,
            refs_header: None,
            read: true,
            starred: false,
            answered: false,
            draft: false,
            deleted: false,
            has_attachments: false,
            safety: crate::service::safety::Verdict::ordinary(),
            gmail_message_id: None,
            labels: None,
            receipt_to: None,
            pop_uidl: None,
        }
    }

    fn rows(cache: &MessageCache) -> Vec<crate::data::message_cache::MessageListRow> {
        let folder = cache
            .get_folder("acct", "Sent")
            .expect("read the folder")
            .expect("the Sent folder");
        cache
            .get_message_list(folder.id, "acct")
            .expect("list the folder")
    }

    fn the_only_row(cache: &MessageCache) -> crate::data::message_cache::MessageListRow {
        let mut listed = rows(cache);
        assert_eq!(listed.len(), 1, "the Sent folder holds {listed:?}");
        listed.remove(0)
    }
}

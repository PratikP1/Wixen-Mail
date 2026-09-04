//! Signing in to an account's own mail server, and holding the session open.
//!
//! One sign-in, for the things this program does at a server on somebody's
//! behalf. Two of them used to spell it out for themselves, six lines each, down
//! to the same sentence about an unusable port: the copy of a message that has
//! gone out, and the copy of a draft that is still being written. Two answers to
//! "how does this account sign in" that happened to agree. The main window did
//! it another twelve times, once per command, so marking one message read was a
//! whole TLS handshake, a CAPABILITY, a LOGIN and a SELECT.
//!
//! Kept apart from the steps that use it so that failing to reach the server at
//! all is one failure rather than several, and so nothing is dialled before
//! something has decided a connection is needed.
//!
//! # How long a session lives
//!
//! Four questions, because they are the ones somebody reading this will have,
//! and because this file used to answer all four with one sentence saying the
//! caller closes it.
//!
//! **When one opens.** The first piece of work for an account that has no
//! session opens one. Nothing opens a session in advance, so an account nobody
//! touches is never dialled.
//!
//! **When one closes.** When the account is removed, when the program closes,
//! and at no other time. A piece of work finishing does not close it: that is
//! the whole change. [`no_longer_signed_in_to`] is the first,
//! [`no_longer_signed_in_to_anything`] the second, and both send LOGOUT rather
//! than dropping the socket, so the session ends at the server as well as here.
//!
//! **What a piece of work that fails leaves behind.** A sign-in the server turns
//! down leaves nothing held, so the next piece of work signs in again rather
//! than finding an entry that was never filled. A piece of work that fails after
//! signing in leaves the session where it is, and that is deliberate: a server
//! refusing to open a folder, or refusing a flag on an account that may not
//! write, has answered, and a connection that answers is worth keeping. The case
//! that is not worth keeping is a connection that has gone, and it never reaches
//! here: [`crate::application::mail_controller::MailController`] signs in again
//! once when a command finds the connection gone, so a session that has died
//! replaces itself rather than being handed out dead.
//!
//! **What removing the account does.** Closes its session and forgets it.
//!
//! # What is held, and what is not held across a call to the server
//!
//! One session per account behind a lock of its own, and a map from account to
//! that lock behind a second. The map's lock is taken to find the account's
//! entry and released at once, so one account's slow sign-in never stops another
//! account being asked about. The account's own lock is held across its sign-in
//! on purpose, because that is what makes two pieces of work arriving together
//! sign in once rather than twice. Neither is held while the work runs.
//!
//! Inside the session one connection carries one command at a time, so
//! `MailController` takes turns on it. That is not a lock held too long, it is
//! the shape of IMAP.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use tokio::sync::Mutex;

use crate::application::mail_controller::MailController;
use crate::data::account::Account;

/// One account's held session, behind the lock that makes signing in for it
/// happen once.
type TheSessionForOneAccount = Arc<Mutex<Option<Arc<MailController>>>>;

/// Every account's held session, by the account's own id.
///
/// One for the program rather than one per window or per worker, because the
/// pieces of work that share a session run on different threads: each is a
/// `spawn_blocking` closure carrying the account and little else, and a session
/// threaded through all of them as an argument would be a session each in every
/// place that forgot to.
fn the_sessions_being_held() -> &'static Mutex<HashMap<String, TheSessionForOneAccount>> {
    static HELD: OnceLock<Mutex<HashMap<String, TheSessionForOneAccount>>> = OnceLock::new();
    HELD.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Sign in to the account's mail server.
///
/// A session, not the session: this opens a new one every time it is called.
/// [`the_session_at`] is what a piece of work in the main window wants; this is
/// the sign-in underneath it, and the two callers that deliberately want a
/// connection of their own.
///
/// The account's own id goes with it, so the session runs under the same
/// permission the rest of that account's writing does: an account that may not
/// change anything at its server gets a session that refuses each command
/// rather than one that carries it out.
pub(crate) async fn a_session_at(
    account: &Account,
) -> std::result::Result<crate::application::mail_controller::MailController, String> {
    let port = account
        .imap_port
        .trim()
        .parse::<u16>()
        .map_err(|_| format!("{} has no usable IMAP port", account.name))?;
    let auth = crate::application::mail_auth::for_account(account)
        .await
        .map_err(|e| e.to_string())?;

    let controller = crate::application::mail_controller::MailController::new();
    controller
        .connect_imap(
            account.imap_server.clone(),
            port,
            account.username.clone(),
            auth,
            account.imap_use_tls,
            &account.id,
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(controller)
}

/// The session this account is signed in with, signing in if there is not one.
///
/// The session, not a session. Two pieces of work one after the other get the
/// same one and the server sees one sign-in. The module note above says how long
/// it lives and what closes it.
pub(crate) async fn the_session_at(
    account: &Account,
) -> std::result::Result<Arc<MailController>, String> {
    // The map is let go before anything is dialled. Held across a sign-in it
    // would make every other account wait out a server that has taken the
    // connection and then gone quiet, which is two minutes.
    let held = {
        let mut sessions = the_sessions_being_held().lock().await;
        sessions.entry(account.id.clone()).or_default().clone()
    };
    // This one is held across the sign-in on purpose, and it is what makes two
    // pieces of work arriving together sign in once rather than twice. It
    // belongs to this account alone, so no other account waits on it, and it is
    // let go as soon as the session is handed back.
    let mut holding = held.lock().await;
    if let Some(session) = holding.as_ref() {
        return Ok(session.clone());
    }
    // A sign-in the server turns down writes nothing, so the entry stays empty
    // and the next piece of work signs in rather than waiting on a session that
    // was never made.
    let session = Arc::new(a_session_at(account).await?);
    *holding = Some(session.clone());
    Ok(session)
}

/// Close this account's session and forget it.
///
/// Forgotten as well as closed, so an account added later under the same id,
/// which is a different account, gets a session of its own.
pub(crate) async fn no_longer_signed_in_to(account_id: &str) {
    let held = the_sessions_being_held().lock().await.remove(account_id);
    let Some(held) = held else {
        return;
    };
    let ending = held.lock().await.take();
    if let Some(session) = ending
        && let Err(why) = session.disconnect_imap().await
    {
        // Logged rather than said out loud. Nobody asked for this: it happens
        // because an account was removed, and the removal itself worked.
        tracing::warn!("A removed account's session could not be closed: {why}");
    }
}

/// Close every account's session.
///
/// What the program does on its way out. One at a time rather than all at once,
/// because the caller bounds the whole of it and a bound spent on a server that
/// has stopped answering is spent whichever order they go in.
pub(crate) async fn no_longer_signed_in_to_anything() {
    let all: Vec<TheSessionForOneAccount> = {
        let mut sessions = the_sessions_being_held().lock().await;
        sessions.drain().map(|(_, held)| held).collect()
    };
    for held in all {
        let ending = held.lock().await.take();
        if let Some(session) = ending
            && let Err(why) = session.disconnect_imap().await
        {
            tracing::warn!("A session could not be closed on the way out: {why}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::answering::{Conversation, LONG_ENOUGH, Turn, conversing};
    use crate::service::protocols::imap::against_a_server_that_answers::a_server_that_can;

    /// Held for the whole of any test that holds a session.
    ///
    /// The held sessions live in one map for the whole program, and that map is
    /// the thing under test. Tests run at the same time in one process, so a
    /// test that closes every session would close one another test was part way
    /// through using, and a test counting sign-ins would count somebody else's.
    ///
    /// Serialised rather than given a map each, because a map per test is not
    /// the map the program uses and a test against it would prove nothing about
    /// the program.
    async fn one_test_at_a_time() -> tokio::sync::MutexGuard<'static, ()> {
        static ONE_AT_A_TIME: Mutex<()> = Mutex::const_new(());
        ONE_AT_A_TIME.lock().await
    }

    /// A mail server that only lets this one name in.
    ///
    /// The name has to be checked here rather than in the transcript, because
    /// the transcript takes the whole of a sign-in line off before recording
    /// it: it is printed when an assertion fails and a password does not go
    /// into something that gets printed. So the server refuses any other name
    /// and signing in at all is the measurement.
    async fn a_server_expecting(name: &'static str) -> Conversation {
        conversing("* OK loopback ready\r\n", move |line| {
            let tag = line.split_whitespace().next().unwrap_or("*").to_string();
            let said = line.to_uppercase();
            match said.split_whitespace().nth(1).unwrap_or_default() {
                "CAPABILITY" => Turn::Say(format!("* CAPABILITY IMAP4rev1\r\n{tag} OK done\r\n")),
                "LOGIN" if line.contains(name) => Turn::Say(format!("{tag} OK signed in\r\n")),
                "LOGIN" => Turn::Say(format!("{tag} NO that is not this account\r\n")),
                "LOGOUT" => Turn::Say(format!("* BYE signing off\r\n{tag} OK done\r\n")),
                _ => Turn::Say(format!("{tag} OK done\r\n")),
            }
        })
        .await
    }

    /// A mail server that drops the connection the first `opens` times it is
    /// asked to open a folder, and answers every one after that.
    ///
    /// What a provider does to a session that has been sitting idle, which is
    /// the case this whole reconnect exists for: nothing is said, the socket
    /// simply goes away, and the client finds out when it next speaks.
    async fn a_server_that_hangs_up_on_the_first_folder_opens(opens: usize) -> Conversation {
        let dropped = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        conversing("* OK loopback ready\r\n", move |line| {
            let tag = line.split_whitespace().next().unwrap_or("*").to_string();
            let said = line.to_uppercase();
            match said.split_whitespace().nth(1).unwrap_or_default() {
                "CAPABILITY" => Turn::Say(format!("* CAPABILITY IMAP4rev1\r\n{tag} OK done\r\n")),
                "LOGIN" | "AUTHENTICATE" => Turn::Say(format!("{tag} OK signed in\r\n")),
                "SELECT" | "EXAMINE" => {
                    let so_far = dropped.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if so_far < opens {
                        Turn::HangUp
                    } else {
                        Turn::Say(format!(
                            "* 0 EXISTS\r\n* 0 RECENT\r\n* OK [UIDVALIDITY 1] valid\r\n\
                             {tag} OK [READ-WRITE] open\r\n"
                        ))
                    }
                }
                "LOGOUT" => Turn::Say(format!("* BYE signing off\r\n{tag} OK done\r\n")),
                _ => Turn::Say(format!("{tag} OK done\r\n")),
            }
        })
        .await
    }

    /// The same server, dropping the connection every time a folder is opened.
    async fn a_server_that_hangs_up_on_every_folder_open() -> Conversation {
        a_server_that_hangs_up_on_the_first_folder_opens(usize::MAX).await
    }

    /// An account whose mail server is the loopback one this test is holding.
    ///
    /// The id is the test's own, because the held sessions are keyed by it and
    /// two tests sharing an id would share a session.
    fn an_account_at(server: &Conversation, id: &str) -> Account {
        let mut account = Account::new("Work".into(), "me@example.com".into());
        account.id = id.to_string();
        account.imap_server = server.server();
        account.imap_port = server.port().to_string();
        account.imap_use_tls = false;
        account.username = "someone".to_string();
        account.password = "hunter2".to_string();
        account.use_oauth = false;
        account
    }

    /// How many times the server was signed in to.
    ///
    /// Counted at the server rather than here. A counter this program kept would
    /// agree with itself: code that signs in twice and counts once passes an
    /// assertion about its own number and fails the only question that matters.
    async fn how_many_sign_ins(server: &Conversation) -> usize {
        server
            .transcript()
            .await
            .iter()
            .filter(|line| {
                let word = line
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or_default()
                    .to_uppercase();
                word == "LOGIN" || word == "AUTHENTICATE"
            })
            .count()
    }

    /// How many times the server was signed out of.
    async fn how_many_sign_offs(server: &Conversation) -> usize {
        server
            .transcript()
            .await
            .iter()
            .filter(|line| {
                line.split_whitespace()
                    .nth(1)
                    .unwrap_or_default()
                    .eq_ignore_ascii_case("LOGOUT")
            })
            .count()
    }

    // Nothing here asserts that a session may write. Whether it may is decided
    // at sign-in by reading the account's setting out of the profile of
    // whoever is running the suite, so an assertion about an append succeeding
    // through one of these would pass on one machine and fail on another. What
    // is measured is the sign-in itself: which server, under which name, and
    // that a bad port is named rather than dialled. Opening a folder is the
    // piece of work every test below uses for the same reason: it needs no
    // permission and every server answers it.

    #[tokio::test]
    async fn test_signing_in_for_a_save_reaches_the_account_s_own_server() {
        // Under the account's own login name, which on plenty of providers is
        // not its address. Signing in under the address instead is refused by
        // the server rather than noticed here, so a queue would fail on every
        // message with something that reads like a wrong password.
        let server = a_server_expecting("someone").await;
        let mut account = an_account_at(&server, "reaches-its-own-server");
        account.email = "not-the-login@example.com".to_string();

        let session = a_session_at(&account).await.expect("the server answered");

        assert!(session.is_connected().await, "nothing was connected");

        session.disconnect_imap().await.expect("the session to end");
        assert!(
            server.was_told("LOGOUT").await,
            "the session was dropped rather than closed, which leaves it open on \
             the server: {:?}",
            server.transcript().await
        );
    }

    #[tokio::test]
    async fn test_a_sign_in_the_server_turns_down_is_a_failure_rather_than_a_session() {
        // What proves the test above measures the name at all. The same server,
        // asked to let in a name it does not know.
        let server = a_server_expecting("someone").await;
        let mut account = an_account_at(&server, "turned-down");
        account.username = "somebody-else".to_string();

        let refused = a_session_at(&account).await;

        assert!(
            refused.is_err(),
            "a sign-in the server turned down came back as a session"
        );
    }

    #[tokio::test]
    async fn test_an_account_whose_port_cannot_be_read_is_named_rather_than_dialled() {
        // Named, because a queue that fails on every message with a bare number
        // gives nobody anything to fix.
        let server = a_server_that_can("").await;
        let mut account = an_account_at(&server, "no-usable-port");
        account.imap_port = "  ".to_string();

        let refused = a_session_at(&account).await;

        let Err(reason) = refused else {
            panic!("an account with no usable port signed in");
        };
        assert!(reason.contains("Work"), "{reason}");
        assert!(reason.contains("port"), "{reason}");
        assert!(
            server.transcript().await.is_empty(),
            "something was dialled before the port was read: {:?}",
            server.transcript().await
        );
    }

    #[tokio::test]
    async fn test_two_pieces_of_work_for_one_account_sign_in_once() {
        // The whole of SCALE-02 in one assertion, and it is the server that
        // answers it. Marking three messages read used to be three of these.
        let _one_at_a_time = one_test_at_a_time().await;
        let server = a_server_that_can("").await;
        let account = an_account_at(&server, "two-pieces-of-work");

        let first = the_session_at(&account).await.expect("the server answered");
        first
            .select_folder("INBOX")
            .await
            .expect("the folder to open");
        let second = the_session_at(&account).await.expect("the server answered");
        second
            .select_folder("INBOX")
            .await
            .expect("the folder to open");

        assert_eq!(
            how_many_sign_ins(&server).await,
            1,
            "the second piece of work signed in again instead of using the \
             session the first one opened: {:?}",
            server.transcript().await
        );

        no_longer_signed_in_to(&account.id).await;
    }

    #[tokio::test]
    async fn test_two_accounts_are_two_sessions_and_two_sign_ins() {
        // A session belongs to an account. Sharing one between two would send
        // one account's commands under the other's sign-in, which on a server
        // that allowed it at all would act on the wrong mailbox.
        let _one_at_a_time = one_test_at_a_time().await;
        let server = a_server_that_can("").await;
        let work = an_account_at(&server, "two-accounts-work");
        let home = an_account_at(&server, "two-accounts-home");

        let _first = the_session_at(&work).await.expect("the server answered");
        let _second = the_session_at(&home).await.expect("the server answered");

        assert_eq!(
            how_many_sign_ins(&server).await,
            2,
            "two accounts shared one session: {:?}",
            server.transcript().await
        );

        no_longer_signed_in_to(&work.id).await;
        no_longer_signed_in_to(&home.id).await;
    }

    #[tokio::test]
    async fn test_a_sign_in_the_server_turned_down_leaves_nothing_held() {
        // A failed sign-in that left an entry behind would be a session the
        // next piece of work waited on and never got.
        let _one_at_a_time = one_test_at_a_time().await;
        let server = a_server_expecting("someone").await;
        let mut wrong = an_account_at(&server, "nothing-held");
        wrong.username = "somebody-else".to_string();

        assert!(
            the_session_at(&wrong).await.is_err(),
            "a sign-in the server turned down came back as a session"
        );

        let right = an_account_at(&server, "nothing-held");
        let session = the_session_at(&right)
            .await
            .expect("the next piece of work to sign in for itself");
        assert!(
            session.is_connected().await,
            "the failed sign-in was held and handed out"
        );

        no_longer_signed_in_to(&right.id).await;
    }

    #[tokio::test]
    async fn test_closing_an_account_closes_the_session_it_was_holding() {
        // Said to the server, not only forgotten here. A session dropped rather
        // than closed stays open at the provider until it times it out, and it
        // counts against the account while it does.
        let _one_at_a_time = one_test_at_a_time().await;
        let server = a_server_that_can("").await;
        let account = an_account_at(&server, "closing-an-account");
        let session = the_session_at(&account).await.expect("the server answered");
        session
            .select_folder("INBOX")
            .await
            .expect("the folder to open");

        no_longer_signed_in_to(&account.id).await;

        assert_eq!(
            how_many_sign_offs(&server).await,
            1,
            "the account was removed and its session was left open at the \
             server: {:?}",
            server.transcript().await
        );

        // And the account is forgotten as well as closed, so a later piece of
        // work signs in rather than being handed the session that was ended.
        let _again = the_session_at(&account).await.expect("the server answered");
        assert_eq!(
            how_many_sign_ins(&server).await,
            2,
            "a closed session was handed out again: {:?}",
            server.transcript().await
        );

        no_longer_signed_in_to(&account.id).await;
    }

    #[tokio::test]
    async fn test_closing_everything_closes_the_session_of_every_account() {
        // What the program does on its way out.
        let _one_at_a_time = one_test_at_a_time().await;
        let server = a_server_that_can("").await;
        let work = an_account_at(&server, "closing-everything-work");
        let home = an_account_at(&server, "closing-everything-home");
        let _first = the_session_at(&work).await.expect("the server answered");
        let _second = the_session_at(&home).await.expect("the server answered");

        no_longer_signed_in_to_anything().await;

        assert_eq!(
            how_many_sign_offs(&server).await,
            2,
            "the program closed and left sessions open at the server: {:?}",
            server.transcript().await
        );
    }

    #[tokio::test]
    async fn test_one_account_s_sign_in_does_not_hold_up_another_account_s() {
        // The map from account to session is one lock. Held across a sign-in it
        // would make every other account wait out a server that has accepted
        // the connection and then gone quiet, which is two minutes.
        let _one_at_a_time = one_test_at_a_time().await;
        let silent = crate::common::answering::never_answering().await;
        let server = a_server_that_can("").await;
        let mut stalled = an_account_at(&server, "stalled");
        stalled.imap_server = silent.ip().to_string();
        stalled.imap_port = silent.port().to_string();
        let answering = an_account_at(&server, "not-stalled");

        let waiting = tokio::spawn(async move {
            let _ = the_session_at(&stalled).await;
        });
        tokio::task::yield_now().await;

        let got = tokio::time::timeout(LONG_ENOUGH, the_session_at(&answering)).await;

        assert!(
            got.is_ok(),
            "one account's sign-in was waiting on another account's"
        );
        waiting.abort();
        no_longer_signed_in_to(&answering.id).await;
        no_longer_signed_in_to("stalled").await;
    }

    #[tokio::test]
    async fn test_one_sign_in_is_one_connection() {
        // What says the connection count is a reading rather than a number this
        // test file agrees with. Nought before anything is dialled, one after a
        // sign-in, and the budget test below counts two: no constant answers
        // all three.
        let _one_at_a_time = one_test_at_a_time().await;
        let server = a_server_that_can("").await;
        let account = an_account_at(&server, "one-connection");

        assert_eq!(
            server.connections(),
            0,
            "something was dialled before anything asked for a session"
        );

        let _session = the_session_at(&account).await.expect("the server answered");

        assert_eq!(
            server.connections(),
            1,
            "one sign-in opened more than one connection"
        );

        no_longer_signed_in_to(&account.id).await;
    }

    #[tokio::test]
    async fn test_a_connection_the_server_dropped_is_signed_in_again_and_the_work_finishes() {
        // The connection goes away between one piece of work and the next, and
        // the caller does not have to know: it asked for a folder and gets one.
        let _one_at_a_time = one_test_at_a_time().await;
        let server = a_server_that_hangs_up_on_the_first_folder_opens(1).await;
        let account = an_account_at(&server, "dropped-once");

        let session = the_session_at(&account).await.expect("the server answered");
        session
            .select_folder("INBOX")
            .await
            .expect("the folder to open after the connection was made again");

        assert_eq!(
            how_many_sign_ins(&server).await,
            2,
            "the dropped connection was not signed in again: {:?}",
            server.transcript().await
        );

        no_longer_signed_in_to(&account.id).await;
    }

    #[tokio::test]
    async fn test_a_reconnect_that_fails_too_is_refused_in_plain_words() {
        // Nobody can act on "connection lost". The sentence says what happened,
        // that a second attempt was made, and what to do next.
        let _one_at_a_time = one_test_at_a_time().await;
        let server = a_server_that_hangs_up_on_every_folder_open().await;
        let account = an_account_at(&server, "dropped-always-words");

        let session = the_session_at(&account).await.expect("the server answered");
        let refused = session.select_folder("INBOX").await;

        let Err(why) = refused else {
            panic!("a server that hangs up on every folder open opened one");
        };
        let said = why.to_string();
        assert!(
            said.contains("connection to the mail server was lost"),
            "the refusal does not say what happened: {said}"
        );
        assert!(
            said.contains("tried once more"),
            "the refusal does not say a second attempt was made: {said}"
        );
        assert!(
            said.contains("try again"),
            "the refusal does not say what to do next: {said}"
        );
        assert!(
            !said.contains("Network error") && !said.contains("Protocol error"),
            "the refusal reaches somebody with the layer's name in front of it: {said}"
        );

        no_longer_signed_in_to(&account.id).await;
    }

    #[tokio::test]
    async fn test_a_server_that_drops_every_connection_is_tried_once_more_and_no_more() {
        // Once means once. A loop that kept trying against a server that keeps
        // dropping is what gets an account rate-limited, and Gmail punishes an
        // account that opens more than fifteen connections.
        let _one_at_a_time = one_test_at_a_time().await;
        let server = a_server_that_hangs_up_on_every_folder_open().await;
        let account = an_account_at(&server, "dropped-always-once");

        let session = the_session_at(&account).await.expect("the server answered");
        let _refused = session.select_folder("INBOX").await;

        assert_eq!(
            how_many_sign_ins(&server).await,
            2,
            "the first sign-in and exactly one more is what a dropped \
             connection costs: {:?}",
            server.transcript().await
        );
        assert_eq!(
            server.connections(),
            2,
            "a connection that keeps dropping was dialled more than twice"
        );

        no_longer_signed_in_to(&account.id).await;
    }
}

//! Signing in to an account's own mail server for a piece of work.
//!
//! One sign-in, for the two things this program files at a server on somebody's
//! behalf: the copy of a message that has gone out, and the copy of a draft
//! that is still being written. Both used to spell it out for themselves, six
//! lines each, down to the same sentence about an unusable port. Two answers to
//! "how does this account sign in" that happened to agree.
//!
//! Kept apart from the steps that use it so that failing to reach the server at
//! all is one failure rather than several, and so nothing is dialled before
//! something has decided a connection is needed. The caller closes it.

use crate::data::account::Account;

/// Sign in to the account's mail server.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::answering::{Conversation, Turn, conversing};
    use crate::service::protocols::imap::against_a_server_that_answers::a_server_that_can;

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

    /// An account whose mail server is the loopback one this test is holding.
    fn an_account_at(server: &crate::common::answering::Conversation) -> Account {
        let mut account = Account::new("Work".into(), "me@example.com".into());
        account.id = "acct".to_string();
        account.imap_server = server.server();
        account.imap_port = server.port().to_string();
        account.imap_use_tls = false;
        account.username = "someone".to_string();
        account.password = "hunter2".to_string();
        account.use_oauth = false;
        account
    }

    // Nothing here asserts that a session may write. Whether it may is decided
    // at sign-in by reading the account's setting out of the profile of
    // whoever is running the suite, so an assertion about an append succeeding
    // through one of these would pass on one machine and fail on another. What
    // is measured is the sign-in itself: which server, under which name, and
    // that a bad port is named rather than dialled.

    #[tokio::test]
    async fn test_signing_in_for_a_save_reaches_the_account_s_own_server() {
        // Under the account's own login name, which on plenty of providers is
        // not its address. Signing in under the address instead is refused by
        // the server rather than noticed here, so a queue would fail on every
        // message with something that reads like a wrong password.
        let server = a_server_expecting("someone").await;
        let mut account = an_account_at(&server);
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
        let mut account = an_account_at(&server);
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
        let mut account = an_account_at(&server);
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
}

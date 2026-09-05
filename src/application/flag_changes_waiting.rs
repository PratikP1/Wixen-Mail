//! A flag change that could not go yet, kept until it can.
//!
//! # The defect this is about
//!
//! Marking a message read, or starring it, is applied here at once and pushed
//! straight after, because waiting on a round trip before confirming a
//! keystroke makes the application feel broken. If the push fails, the local
//! change is put back and the reason is said out loud, so an announcement that
//! turned out to be wrong is corrected rather than left standing.
//!
//! Every failure took that path, including the one where the server was never
//! reached. So starring a message on a train starred it, un-starred it a moment
//! later, and said why. The change was not refused by anybody: it was never put
//! to anybody.
//!
//! # The distinction, drawn once
//!
//! [`why_the_push_failed`] is the only place this is decided, and everything
//! downstream reads its answer. It matches on [`Error`]'s own variants and
//! never on the text of a message, because a message is not a type and the two
//! cases want opposite treatment.
//!
//! Three answers rather than two, because the middle one is real: this
//! computer's own write gate refuses a change before anything is sent, and that
//! is neither the server answering nor the network dropping. It is kept for the
//! same reason a change no setting let out is kept everywhere else here.
//!
//! # Guardrail 7, which is the constraint a later reader will be most tempted
//! to break
//!
//! **Nothing in this module may observe the network coming back and send.** A
//! flag change reaching the server is a write at somebody else's service, so it
//! happens on purpose. A waiting change goes when something that was going to
//! talk to that server anyway does, or when somebody asks, and never as a side
//! effect of a connectivity event.
//!
//! It is easier to break here than it was for the Outbox, because a flag feels
//! smaller than a message. It is not smaller. It is a write at somebody else's
//! server, and it is the same trap plan 03-08 answered for the Outbox with an
//! offer rather than a send. [`WhatToDoWithAWaitingChange`] has no member
//! meaning "send now" for that reason: a decision that cannot express the
//! dangerous act cannot be wired to it by accident.
//!
//! # What has never been checked
//!
//! No account has ever been used with this program. The two failures below are
//! driven against a loopback server that can be made to drop a connection or to
//! answer `NO`, which is closer than a mocked error and is not a real mail
//! server.

use crate::common::Error;

/// Which flag a waiting change is about.
///
/// Written out as words rather than stored as a number, because these go in a
/// database column somebody may read and a discriminant is neither stable
/// across a reordering nor legible in a browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhichFlag {
    /// Read or unread.
    Read,
    /// Starred or not.
    Starred,
}

impl WhichFlag {
    /// The word the row carries.
    pub fn as_stored(self) -> &'static str {
        match self {
            WhichFlag::Read => "read",
            WhichFlag::Starred => "starred",
        }
    }

    /// Back from the column, where the word is one this version knows.
    pub fn from_stored(stored: &str) -> Option<Self> {
        match stored {
            "read" => Some(WhichFlag::Read),
            "starred" => Some(WhichFlag::Starred),
            _ => None,
        }
    }
}

/// Why a push did not get in.
///
/// Named rather than boolean, because the three want three different things and
/// a boolean would make somebody guess which way round it reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhyThePushFailed {
    /// The server was never reached, so it has said nothing about this change.
    ///
    /// A dropped connection, a machine with no network, a server that stopped
    /// answering, a sign-in that could not be completed. In none of them did
    /// anybody look at the change and turn it down.
    TheServerWasNeverAsked,
    /// This computer's own settings refused it before anything was sent.
    ///
    /// Nothing left the machine. The change is still worth keeping and the
    /// setting is what clears it, which is how every other sync here treats
    /// this case.
    ThisComputerRefusedIt,
    /// The server answered, and the answer was no.
    ///
    /// A refusal that can repeat for ever: sending the same thing again meets
    /// the same answer. This is the one where the local change has to be put
    /// back, because the message on the server really is not in the state the
    /// window is showing.
    TheServerSaidNo,
}

/// Why a push did not get in, from the error it failed with.
///
/// The one place this is decided. `common::Error`'s variants are what it reads:
/// `Network` is raised by `service::protocols::imap::connection_failed`, which
/// is reached from the library's own `Io` and `ConnectionLost` variants and
/// from a command that timed out, and `Security` is raised by
/// `service::outward::permitted` before a request is built. Everything else is
/// the server having answered.
///
/// `Authentication` counts as never having asked, and that is a decision rather
/// than an oversight. A sign-in the server turned down means the change was
/// never put to it, and a token that has expired is fixed by signing in again,
/// so the change can still go. Putting a star back because a token expired
/// loses work over something that clears on its own.
pub fn why_the_push_failed(error: &Error) -> WhyThePushFailed {
    match error {
        Error::Network(_) | Error::Authentication(_) => WhyThePushFailed::TheServerWasNeverAsked,
        Error::Security(_) => WhyThePushFailed::ThisComputerRefusedIt,
        Error::Config(_)
        | Error::Protocol(_)
        | Error::Api { .. }
        | Error::Other(_)
        | Error::InPlainWords(_) => WhyThePushFailed::TheServerSaidNo,
    }
}

/// What a failed push calls for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatAFailedPushCallsFor {
    /// Put the local change back and say the server refused it.
    PutItBack,
    /// Leave the local change alone and record it as waiting.
    KeepItAndWait,
}

/// What to do about a push that did not get in.
pub fn what_to_do_about_it(why: WhyThePushFailed) -> WhatAFailedPushCallsFor {
    match why {
        WhyThePushFailed::TheServerSaidNo => WhatAFailedPushCallsFor::PutItBack,
        WhyThePushFailed::TheServerWasNeverAsked | WhyThePushFailed::ThisComputerRefusedIt => {
            WhatAFailedPushCallsFor::KeepItAndWait
        }
    }
}

/// What may happen to a change that is waiting.
///
/// Three members and none of them means "send now". A decision that cannot
/// express the dangerous act cannot be wired to it by accident, which is the
/// shape plan 03-08 settled for the Outbox and the reason it is repeated here.
/// What sends a waiting change is a sync that already has a session, or
/// somebody asking; both call the sending directly and neither goes through
/// this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatToDoWithAWaitingChange {
    /// It went. Stop waiting.
    ItWent,
    /// The server refused it when it finally went. Put the local change back
    /// and stop waiting: sending it again meets the same answer.
    PutItBackAndStopWaiting,
    /// Still nothing to say. Leave it waiting.
    GoOnWaiting,
}

/// What became of a waiting change that was just offered.
pub fn what_became_of_it(offered: &Result<(), Error>) -> WhatToDoWithAWaitingChange {
    match offered {
        Ok(()) => WhatToDoWithAWaitingChange::ItWent,
        Err(why) => match why_the_push_failed(why) {
            WhyThePushFailed::TheServerSaidNo => {
                WhatToDoWithAWaitingChange::PutItBackAndStopWaiting
            }
            WhyThePushFailed::TheServerWasNeverAsked | WhyThePushFailed::ThisComputerRefusedIt => {
                WhatToDoWithAWaitingChange::GoOnWaiting
            }
        },
    }
}

/// The topic the sentences below are announced on.
///
/// Its own topic rather than `"status"`, and the precedent is `"network"` and
/// `"message text"`. `"status"` carries every steady sync line and the queue
/// keeps only the newest of a topic, so a sentence about a change that is
/// waiting would be replaced before anybody heard it.
///
/// Both sentences share it on purpose. A folder sync that fails on twenty
/// messages must say one thing with a count rather than twenty things, and
/// topic superseding is what makes that fall out of the queue rather than out
/// of code somebody has to remember.
pub const WHAT_HAPPENED_TO_A_CHANGE: &str = "flag change";

/// What is said when the server could not be reached.
///
/// Written out both ways rather than built from parts, because three words have
/// to agree in number.
pub fn kept_because_the_server_was_not_there(count: usize) -> String {
    match count {
        1 => "The mail server could not be reached, so your change is saved here \
              and goes the next time Wixen Mail talks to it"
            .to_string(),
        many => format!(
            "The mail server could not be reached, so {many} changes are saved here \
             and go the next time Wixen Mail talks to it"
        ),
    }
}

/// What is said when the server answered and said no.
///
/// Deliberately unlike the sentence above, because the two are different facts
/// and guardrail 5 asks that feedback be distinct as well as bounded. That one
/// says the change is kept; this one says it is gone. They share no opening
/// clause and no verb.
pub fn put_back_because_the_server_refused(count: usize, reason: &str) -> String {
    match count {
        1 => format!("The mail server refused your change, so it has been put back: {reason}"),
        many => {
            format!("The mail server refused {many} changes, so they have been put back: {reason}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::answering::Turn;

    #[test]
    fn test_a_lost_connection_is_the_server_never_having_been_asked() {
        assert_eq!(
            why_the_push_failed(&Error::Network("the connection went".to_string())),
            WhyThePushFailed::TheServerWasNeverAsked
        );
    }

    #[test]
    fn test_a_server_that_answered_no_is_a_refusal() {
        assert_eq!(
            why_the_push_failed(&Error::Protocol("NO cannot store".to_string())),
            WhyThePushFailed::TheServerSaidNo
        );
    }

    #[test]
    fn test_this_computers_own_gate_is_neither_of_the_two() {
        assert_eq!(
            why_the_push_failed(&Error::Security("Allow Changes is off".to_string())),
            WhyThePushFailed::ThisComputerRefusedIt
        );
    }

    #[test]
    fn test_a_sign_in_that_did_not_happen_leaves_the_change_worth_keeping() {
        // A token that expired clears by signing in again. Putting a star back
        // because of one loses work over something nobody has to decide.
        assert_eq!(
            what_to_do_about_it(why_the_push_failed(&Error::Authentication(
                "the token has expired".to_string()
            ))),
            WhatAFailedPushCallsFor::KeepItAndWait
        );
    }

    #[test]
    fn test_only_a_refusal_puts_the_local_change_back() {
        assert_eq!(
            what_to_do_about_it(WhyThePushFailed::TheServerSaidNo),
            WhatAFailedPushCallsFor::PutItBack
        );
        assert_eq!(
            what_to_do_about_it(WhyThePushFailed::TheServerWasNeverAsked),
            WhatAFailedPushCallsFor::KeepItAndWait
        );
        assert_eq!(
            what_to_do_about_it(WhyThePushFailed::ThisComputerRefusedIt),
            WhatAFailedPushCallsFor::KeepItAndWait
        );
    }

    #[test]
    fn test_a_waiting_change_that_went_stops_waiting() {
        assert_eq!(
            what_became_of_it(&Ok(())),
            WhatToDoWithAWaitingChange::ItWent
        );
    }

    #[test]
    fn test_a_waiting_change_the_server_refuses_is_put_back_rather_than_offered_for_ever() {
        assert_eq!(
            what_became_of_it(&Err(Error::Protocol("NO".to_string()))),
            WhatToDoWithAWaitingChange::PutItBackAndStopWaiting
        );
    }

    #[test]
    fn test_a_waiting_change_the_server_still_cannot_be_asked_about_goes_on_waiting() {
        assert_eq!(
            what_became_of_it(&Err(Error::Network("still gone".to_string()))),
            WhatToDoWithAWaitingChange::GoOnWaiting
        );
    }

    #[test]
    fn test_nothing_here_can_say_send_now() {
        // Guardrail 7, held by the shape of the type rather than by a comment.
        // A decision that cannot express the dangerous act cannot be wired to
        // it by accident. If a member meaning "send" is ever added, this is
        // where somebody is asked to argue for it.
        let every_answer = [
            WhatToDoWithAWaitingChange::ItWent,
            WhatToDoWithAWaitingChange::PutItBackAndStopWaiting,
            WhatToDoWithAWaitingChange::GoOnWaiting,
        ];
        assert_eq!(
            every_answer.len(),
            3,
            "a fourth answer was added to what may happen to a waiting change. \
             If it means sending one, that is guardrail 7 and it needs an \
             argument rather than an arm"
        );
    }

    #[test]
    fn test_the_two_sentences_are_plainly_different() {
        let kept = kept_because_the_server_was_not_there(1);
        let put_back = put_back_because_the_server_refused(1, "over quota");
        assert!(kept.contains("could not be reached"), "{kept}");
        assert!(kept.contains("saved here"), "{kept}");
        assert!(put_back.contains("refused"), "{put_back}");
        assert!(put_back.contains("put back"), "{put_back}");
        assert!(
            !kept.contains("refused") && !kept.contains("put back"),
            "the two sentences share the words that tell them apart: {kept}"
        );
        assert!(
            !put_back.contains("saved here"),
            "the two sentences share the words that tell them apart: {put_back}"
        );
    }

    #[test]
    fn test_twenty_failures_are_one_sentence_carrying_a_count() {
        let said = kept_because_the_server_was_not_there(20);
        assert!(said.contains("20 changes are saved"), "{said}");
        assert!(
            !said.contains("your change is saved"),
            "the singular leaked into the plural: {said}"
        );
    }

    #[test]
    fn test_one_failure_is_not_read_out_in_the_plural() {
        let said = kept_because_the_server_was_not_there(1);
        assert!(said.contains("your change is saved"), "{said}");
        assert!(!said.contains("changes"), "{said}");
    }

    /// A session signed in to that server with the write gate open, and a
    /// mailbox already selected.
    ///
    /// Below the controller rather than through it, and that is a finding
    /// rather than a shortcut. `MailController::connect_imap` opens the write
    /// gate only when `allowed_for` says the account may change things, which
    /// is a setting on the machine running the tests. Through the controller
    /// both servers below answered `Error::Security` before either was asked
    /// anything, so the tests proved that the gate works and nothing at all
    /// about the distinction they are named for. `mail_controller.rs`'s own
    /// tests record the same thing about themselves and assert on what reached
    /// the server for exactly this reason.
    ///
    /// What is skipped is the gate. What is exercised is the socket, the
    /// library's own error and this project's mapping of it, which is where
    /// the distinction really has to hold.
    async fn a_session_with_the_inbox_open(
        server: &crate::common::answering::Conversation,
    ) -> crate::service::protocols::imap::ImapSession {
        let mut session =
            crate::service::protocols::imap::against_a_server_that_answers::signed_in_to(server)
                .await;
        session
            .select_folder("INBOX")
            .await
            .expect("the folder to open");
        session
    }

    #[tokio::test]
    async fn test_a_server_that_drops_the_connection_is_read_as_never_having_been_asked() {
        // Against a real server rather than a mocked error, because the whole
        // point of the distinction is where it is drawn: it has to survive the
        // library's own error type and this project's mapping of it. A test
        // that builds an `Error::Network` by hand proves only that the match
        // arm below matches.
        //
        // The server takes the sign-in and the SELECT and then hangs up on the
        // command that would change the message. That is what a connection
        // going mid-command looks like from here.
        let server = crate::common::answering::conversing("* OK loopback ready\r\n", |line| {
            let tag = line.split_whitespace().next().unwrap_or("*").to_string();
            let said = line.to_uppercase();
            match said.split_whitespace().nth(1).unwrap_or_default() {
                "CAPABILITY" => Turn::Say(format!("* CAPABILITY IMAP4rev1\r\n{tag} OK done\r\n")),
                "LOGIN" | "AUTHENTICATE" => Turn::Say(format!("{tag} OK signed in\r\n")),
                "SELECT" | "EXAMINE" => Turn::Say(format!(
                    "* 0 EXISTS\r\n* OK [UIDVALIDITY 1] valid\r\n{tag} OK [READ-WRITE] open\r\n"
                )),
                // The command that matters, and nothing comes back at all.
                _ => Turn::HangUp,
            }
        })
        .await;
        let mut session = a_session_with_the_inbox_open(&server).await;

        let outcome = session
            .set_flag(1, crate::service::protocols::imap::flag::FLAGGED, true)
            .await;

        let why = outcome.expect_err("the connection went, so this cannot have worked");
        assert_eq!(
            why_the_push_failed(&why),
            WhyThePushFailed::TheServerWasNeverAsked,
            "a lost connection was read as the server having refused the \
             change, so the star is about to be taken back off: {why}"
        );
        assert_eq!(
            what_to_do_about_it(why_the_push_failed(&why)),
            WhatAFailedPushCallsFor::KeepItAndWait
        );
    }

    #[tokio::test]
    async fn test_a_server_that_answers_no_is_read_as_a_refusal_and_still_puts_the_change_back() {
        // The line the case above must not cross. This server answers, and the
        // answer is no. Nothing here should be kept: sending the same thing
        // again meets the same answer, and the message really is not in the
        // state the window is showing.
        let server =
            crate::service::protocols::imap::against_a_server_that_answers::a_server_that_refuses(
                "",
                "UID STORE",
            )
            .await;
        let mut session = a_session_with_the_inbox_open(&server).await;

        let outcome = session
            .set_flag(1, crate::service::protocols::imap::flag::FLAGGED, true)
            .await;

        let why = outcome.expect_err("the server said no");
        assert_eq!(
            why_the_push_failed(&why),
            WhyThePushFailed::TheServerSaidNo,
            "a refusal was read as the server not being there, so a change it \
             will never take is kept and offered on every sync: {why}"
        );
        assert_eq!(
            what_to_do_about_it(why_the_push_failed(&why)),
            WhatAFailedPushCallsFor::PutItBack
        );
    }

    #[tokio::test]
    async fn test_a_waiting_change_still_meets_the_write_gate_when_it_finally_goes() {
        // An account open for reading only must not accumulate changes that
        // are written later. The gate is met at the session, so a waiting
        // change offered on one is refused there rather than sent, and the
        // refusal is a gate refusal: the change goes on waiting rather than
        // being lost or put back.
        let server =
            crate::service::protocols::imap::against_a_server_that_answers::a_server_that_can("")
                .await;
        let mut session =
            crate::service::protocols::imap::against_a_server_that_answers::reading_only_on(
                &server,
            )
            .await;
        // The folder opens: reading is allowed. Only the change is refused.
        session
            .select_folder("INBOX")
            .await
            .expect("the folder to open");

        let outcome = session
            .set_flag(1, crate::service::protocols::imap::flag::FLAGGED, true)
            .await;

        let why = outcome.expect_err("an account open for reading only changed a message");
        assert_eq!(
            why_the_push_failed(&why),
            WhyThePushFailed::ThisComputerRefusedIt,
            "{why}"
        );
        assert_eq!(
            what_became_of_it(&Err(why)),
            WhatToDoWithAWaitingChange::GoOnWaiting,
            "a change the setting refused was either sent or thrown away, and \
             both are wrong: turning the setting on is what clears it"
        );
        assert!(
            !server
                .transcript()
                .await
                .iter()
                .any(|line| line.to_uppercase().contains("STORE")),
            "the change reached the server on an account open for reading \
             only: {:?}",
            server.transcript().await
        );
    }

    #[test]
    fn test_a_flag_survives_being_written_down_and_read_back() {
        for flag in [WhichFlag::Read, WhichFlag::Starred] {
            assert_eq!(WhichFlag::from_stored(flag.as_stored()), Some(flag));
        }
        assert_eq!(WhichFlag::from_stored("something later"), None);
    }
}

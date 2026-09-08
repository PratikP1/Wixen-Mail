//! Copying a message from the server of one account to the server of another.
//!
//! Two accounts are two mail servers, and no IMAP command reaches across them.
//! `COPY` and `MOVE` name a mailbox on the connection they are sent down, so a
//! destination in another account cannot be reached by sending the same command
//! somewhere else: the folder path would be looked up on the wrong server, and
//! on a server that happened to have a folder of that name the message would
//! land in the wrong mailbox rather than failing.
//!
//! So a crossing is made of two conversations. The bytes are fetched from the
//! account the message is in, and appended to the account it is going to, over
//! that account's own held session carrying that account's own permission.
//!
//! # Why the order is the safeguard
//!
//! The append happens at the destination and the source is not touched at all.
//! A copy that fails leaves the message exactly where it was, which is why
//! copying ships before moving: the fetch, the append, the flags, the two
//! sessions and the permission gate can all be built and proved before anything
//! is asked to remove an original. `imap.rs`'s no-MOVE path already works this
//! way for a move inside one account, and its doc gives the same reason.
//!
//! # Why the decisions here are functions rather than branches
//!
//! Three of them, and each has a reading that compiles and is wrong. Whether a
//! destination crosses is two accounts compared, and a comparison that is
//! dropped leaves a `COPY` going to the source server naming a folder that is
//! not there. Which flags travel is a filter, and the wrong filter carries
//! `\Deleted` to a server whose next expunge acts on it. Which date the message
//! arrived is a format, and no date at all makes a five year old message read as
//! arriving today, at the top of somebody's folder.

use crate::common::Result;
use crate::service::protocols::imap::{ImapMessage, flag};

/// Whether a chosen destination is at the same server the message is at.
///
/// Two named values rather than a `bool`, because both readings look the same
/// at the call site and only one of them opens a second session. "true means it
/// crosses" and "true means it is the same account" are the same token, and
/// nothing but a name says which one a caller wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Crossing {
    /// A folder on the same server, so the existing `COPY` does it in one
    /// command down the connection that is already open.
    TheSameAccount,
    /// A folder on another account's server, so the message has to be fetched
    /// from one and appended to the other.
    AnotherAccount,
}

/// Whether putting the message there means speaking to a second server.
///
/// The account identifiers, never the folder paths. A path is unique inside one
/// account and not across them, which is what
/// [`crate::application::destinations::FolderInAnAccount`] exists to say.
pub fn whether_it_crosses(the_message_is_in: &str, it_is_going_to: &str) -> Crossing {
    if the_message_is_in == it_is_going_to {
        Crossing::TheSameAccount
    } else {
        Crossing::AnotherAccount
    }
}

/// The flags that survive being carried to another server.
///
/// Four of the five standard ones travel: read, starred, answered and draft.
/// They mean the same thing at every provider, and they are what somebody would
/// notice missing. The names are `flag.rs`'s constants, so this file does not
/// spell any of them a second time.
///
/// `\Deleted` is dropped. A message arriving somewhere new already marked for
/// removal is a message the destination's next expunge takes, which is a copy
/// somebody asked for and then cannot find.
///
/// Every other keyword is dropped as well, and that is the less obvious half. A
/// provider invents keywords for its own features, and they mean nothing at
/// another provider. A strict server may refuse the whole `APPEND` over a single
/// keyword it does not recognise, which turns a keyword nobody would miss into a
/// copy that did not happen.
///
/// `None` when nothing survives, because `APPEND` with an empty flag list and
/// `APPEND` with no flag list are different commands and only one of them is
/// what this means.
pub fn the_flags_that_travel(on_the_message: &[String]) -> Option<String> {
    let travelling: Vec<&str> = on_the_message
        .iter()
        .filter_map(|named| {
            [flag::SEEN, flag::FLAGGED, flag::ANSWERED, flag::DRAFT]
                .into_iter()
                .find(|known| named.eq_ignore_ascii_case(known))
        })
        .collect();
    if travelling.is_empty() {
        return None;
    }
    Some(format!("({})", travelling.join(" ")))
}

/// The date the source server filed it, spelled the way `APPEND` takes one.
///
/// Without it the destination stamps the message with the moment it arrived, so
/// a five year old message filed into another account sorts to the top of that
/// account's folder and every later sort by date is wrong about it.
///
/// Formatted from a parsed date and never passed through as the text it arrived
/// as. `async-imap` puts this value straight into the command line with no
/// quoting and no checking, so carrying a server's answer across unparsed would
/// be putting a stranger's text inside a command this program sends. A date that
/// does not read comes back as `None`, which is the same as a server that said
/// nothing: worse than the right date, and better than a command nobody wrote.
pub fn when_it_arrived(rfc3339: Option<&str>) -> Option<String> {
    let parsed = chrono::DateTime::parse_from_rfc3339(rfc3339?).ok()?;
    Some(format!(
        "\"{}\"",
        parsed.format(crate::service::protocols::imap::INTERNAL_DATE_FORMAT)
    ))
}

/// The account the message is in, as a crossing needs to see it.
///
/// A trait rather than the session itself, so the two conversations can be held
/// with two loopback servers whose permission is set in the test. A session
/// built from an `Account` reads what this program may do at that account out of
/// the settings of whoever is running the suite, so an assertion about an append
/// landing would pass on one machine and fail on another. `mail_session.rs`'s
/// own test module says the same thing about itself and tests only the sign-in
/// for that reason.
///
/// Both methods are the shape [`crate::application::mail_controller`] already
/// has, so the implementation that runs in the program forwards and decides
/// nothing. Everything decided is in [`copy_it_across`], which both sides share.
pub(crate) trait TheAccountItIsIn {
    /// What the server says about these messages: their flags and their dates.
    async fn the_headers_of(&self, folder: &str, uids: &[u32]) -> Result<Vec<ImapMessage>>;
    /// One message exactly as it arrived, which is what an `APPEND` takes.
    async fn the_bytes_of(&self, folder: &str, uid: u32) -> Result<Vec<u8>>;
}

/// The account it is going to.
pub(crate) trait TheAccountItIsGoingTo {
    /// Put this message in that folder, filed as the source had it filed.
    async fn take_this_message(
        &self,
        into: &str,
        flags: Option<&str>,
        arrived: Option<&str>,
        raw: &[u8],
    ) -> Result<()>;
}

/// Copy one message from the account it is in to a folder on another account.
///
/// Reads, then reads, then writes, and the write is at the destination. Nothing
/// is said to the source server that changes anything: no `STORE`, no `COPY` and
/// no `EXPUNGE`. That is what makes a failure leave the message exactly where it
/// was, and it is asserted over the source server's own transcript rather than
/// read off this function.
pub(crate) async fn copy_it_across(
    the_account_it_is_in: &impl TheAccountItIsIn,
    from: &str,
    uid: u32,
    the_account_it_is_going_to: &impl TheAccountItIsGoingTo,
    into: &str,
) -> Result<()> {
    let headers = the_account_it_is_in.the_headers_of(from, &[uid]).await?;
    let Some(filed) = headers.into_iter().find(|message| message.uid == uid) else {
        // The server answered and named no message. Appending anyway would put
        // a copy at the destination unread and dated today, which is worse than
        // not copying it, because nothing afterwards says it happened that way.
        return Err(crate::common::Error::Protocol(format!(
            "The mail server said nothing about the message with UID {uid}"
        )));
    };
    let raw = the_account_it_is_in.the_bytes_of(from, uid).await?;
    the_account_it_is_going_to
        .take_this_message(
            into,
            the_flags_that_travel(&filed.flags).as_deref(),
            when_it_arrived(filed.internal_date.as_deref()).as_deref(),
            &raw,
        )
        .await
}

impl TheAccountItIsIn for crate::application::mail_controller::MailController {
    async fn the_headers_of(&self, folder: &str, uids: &[u32]) -> Result<Vec<ImapMessage>> {
        self.fetch_headers(folder, uids).await
    }

    async fn the_bytes_of(&self, folder: &str, uid: u32) -> Result<Vec<u8>> {
        self.fetch_message_body(folder, uid).await
    }
}

impl TheAccountItIsGoingTo for crate::application::mail_controller::MailController {
    async fn take_this_message(
        &self,
        into: &str,
        flags: Option<&str>,
        arrived: Option<&str>,
        raw: &[u8],
    ) -> Result<()> {
        self.append_message(into, flags, arrived, raw).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::answering::{Conversation, LONG_ENOUGH, Turn, conversing};
    use crate::service::protocols::imap::ImapSession;
    use crate::service::protocols::imap::against_a_server_that_answers::{
        a_server_that_can, a_server_that_refuses, reading_only_on, signed_in_to,
    };

    /// The UID every test here asks for.
    const THE_UID: u32 = 4;

    /// The message the source server hands over, byte for byte.
    const THE_MESSAGE: &str =
        "Subject: Lunch\r\nFrom: Ada <ada@example.com>\r\n\r\nOne o'clock?\r\n";

    /// The headers the source server answers a header fetch with.
    const THE_HEADERS: &str = "Subject: Lunch\r\nFrom: Ada <ada@example.com>\r\n\r\n";

    /// A mail server holding one message, filed with these flags on this date.
    ///
    /// `a_server_that_can` cannot stand in for this. It answers a `UID FETCH`
    /// with `OK` and no data at all, which reads to the client as a message
    /// that is not there, so neither the flags nor the bytes a crossing needs
    /// could come back from it.
    ///
    /// `internal_date` empty means the server names no date, which is a real
    /// answer and a different one from a date that cannot be read.
    async fn a_server_holding_the_message(
        flags: &'static str,
        internal_date: &'static str,
    ) -> Conversation {
        a_server_holding_the_message_numbered(THE_UID, flags, internal_date).await
    }

    /// The same, for a server whose message is not the one being asked about.
    async fn a_server_holding_the_message_numbered(
        uid: u32,
        flags: &'static str,
        internal_date: &'static str,
    ) -> Conversation {
        conversing("* OK loopback ready\r\n", move |line| {
            let tag = line.split_whitespace().next().unwrap_or("*").to_string();
            let said = line.to_uppercase();
            if said.contains("CAPABILITY") {
                return Turn::Say(format!("* CAPABILITY IMAP4rev1\r\n{tag} OK done\r\n"));
            }
            if said.contains(" LOGIN") || said.contains(" AUTHENTICATE") {
                return Turn::Say(format!("{tag} OK signed in\r\n"));
            }
            if said.contains("SELECT") || said.contains("EXAMINE") {
                return Turn::Say(format!(
                    "* 1 EXISTS\r\n* 0 RECENT\r\n* OK [UIDVALIDITY 1] valid\r\n\
                     {tag} OK [READ-WRITE] open\r\n"
                ));
            }
            // Before the header fetch below, because both are `UID FETCH` and
            // only the order tells them apart.
            if said.contains("BODY.PEEK[]") {
                return Turn::Say(format!(
                    "* 1 FETCH (UID {THE_UID} BODY[] {{{}}}\r\n{THE_MESSAGE})\r\n{tag} OK done\r\n",
                    THE_MESSAGE.len()
                ));
            }
            if said.contains("UID FETCH") {
                let dated = if internal_date.is_empty() {
                    String::new()
                } else {
                    format!(" INTERNALDATE \"{internal_date}\"")
                };
                return Turn::Say(format!(
                    "* 1 FETCH (UID {uid} FLAGS ({flags}) RFC822.SIZE 120{dated} \
                     BODY[HEADER.FIELDS (SUBJECT FROM)] {{{}}}\r\n{THE_HEADERS})\r\n\
                     {tag} OK done\r\n",
                    THE_HEADERS.len()
                ));
            }
            if said.contains("LOGOUT") {
                return Turn::Say(format!("* BYE signing off\r\n{tag} OK done\r\n"));
            }
            // Refused rather than ignored, so a script that has fallen behind
            // the client fails in the moment instead of waiting out a timeout,
            // which reads as a slow machine.
            Turn::Say(format!("{tag} BAD unscripted\r\n"))
        })
        .await
    }

    /// A mail server that hands the bytes over but names a different message.
    ///
    /// What a folder looks like when the message has been taken out from
    /// somewhere else between one command and the next.
    async fn a_server_naming_a_different_message() -> Conversation {
        a_server_holding_the_message_numbered(THE_UID + 1, flag::SEEN, "").await
    }

    /// One end of a crossing, behind the lock a shared session needs.
    ///
    /// The two traits take `&self` because the program holds each session in an
    /// `Arc`, and a session's own commands take `&mut self`. In the program the
    /// lock is inside `MailController`; here it is around the bare session.
    struct AnAccountAt(tokio::sync::Mutex<ImapSession>);

    impl TheAccountItIsIn for AnAccountAt {
        async fn the_headers_of(&self, folder: &str, uids: &[u32]) -> Result<Vec<ImapMessage>> {
            let mut session = self.0.lock().await;
            if session.selected_folder() != Some(folder) {
                session.select_folder(folder).await?;
            }
            session.fetch_headers(uids).await
        }

        async fn the_bytes_of(&self, folder: &str, uid: u32) -> Result<Vec<u8>> {
            let mut session = self.0.lock().await;
            if session.selected_folder() != Some(folder) {
                session.select_folder(folder).await?;
            }
            session.fetch_body(uid).await
        }
    }

    impl TheAccountItIsGoingTo for AnAccountAt {
        async fn take_this_message(
            &self,
            into: &str,
            flags: Option<&str>,
            arrived: Option<&str>,
            raw: &[u8],
        ) -> Result<()> {
            self.0
                .lock()
                .await
                .append_message(into, flags, arrived, raw)
                .await
        }
    }

    /// A source account signed in to that server, allowed to read.
    ///
    /// Reading and no more, which is what a copy across accounts needs from the
    /// account the message is in and is worth proving rather than assuming: a
    /// crossing that needed the source's write permission would be a crossing
    /// that could change the source.
    async fn the_account_it_is_in(server: &Conversation) -> AnAccountAt {
        AnAccountAt(tokio::sync::Mutex::new(reading_only_on(server).await))
    }

    /// A destination account signed in to that server, allowed to write.
    async fn the_account_it_is_going_to(server: &Conversation) -> AnAccountAt {
        AnAccountAt(tokio::sync::Mutex::new(signed_in_to(server).await))
    }

    /// A destination account this program may not change anything at.
    async fn a_destination_this_program_may_not_write_to(server: &Conversation) -> AnAccountAt {
        AnAccountAt(tokio::sync::Mutex::new(reading_only_on(server).await))
    }

    /// Wait for one crossing, and fail with a sentence rather than a timeout.
    async fn waiting_for<T>(operation: impl std::future::Future<Output = T>) -> T {
        tokio::time::timeout(LONG_ENOUGH, operation)
            .await
            .expect("the crossing never finished")
    }

    /// Every line said to a server, joined so an assertion can print it.
    async fn everything_said_to(server: &Conversation) -> String {
        server.transcript().await.join("\n")
    }

    #[test]
    fn test_a_folder_in_the_same_account_and_one_in_another_are_not_the_same_answer() {
        // Asserted as a difference rather than as two answers, on purpose. A
        // function with two values is green on arrival against any stub,
        // because the stub answers something and that something satisfies a
        // test that only ever asks about one input. This one cannot pass
        // against a constant, whichever constant it is.
        let same = whether_it_crosses("account-a", "account-a");
        let other = whether_it_crosses("account-a", "account-b");

        assert_ne!(
            same, other,
            "a destination in another account gave the same answer as one in \
             the account the message is in, so nothing here consults the \
             account at all"
        );
        assert_eq!(same, Crossing::TheSameAccount);
        assert_eq!(other, Crossing::AnotherAccount);
    }

    #[test]
    fn test_the_read_state_the_star_and_the_answered_mark_travel() {
        let travelling = the_flags_that_travel(&[
            flag::SEEN.to_string(),
            flag::FLAGGED.to_string(),
            flag::ANSWERED.to_string(),
            flag::DRAFT.to_string(),
        ])
        .expect("four standard flags to travel");

        assert!(travelling.contains(flag::SEEN), "{travelling}");
        assert!(travelling.contains(flag::FLAGGED), "{travelling}");
        assert!(travelling.contains(flag::ANSWERED), "{travelling}");
        assert!(travelling.contains(flag::DRAFT), "{travelling}");
        assert!(
            travelling.starts_with('(') && travelling.ends_with(')'),
            "an APPEND flag list is parenthesised: {travelling}"
        );
    }

    #[test]
    fn test_a_message_marked_for_removal_does_not_arrive_marked_for_removal() {
        // The copy somebody asked for, taken by the destination's next expunge
        // before they ever see it.
        let travelling = the_flags_that_travel(&[
            flag::SEEN.to_string(),
            flag::DELETED.to_string(),
            "$Phishing".to_string(),
        ])
        .expect("the read state to travel");

        assert!(
            !travelling.contains(flag::DELETED),
            "a message marked for removal arrived marked for removal: {travelling}"
        );
        assert!(
            !travelling.contains("Phishing"),
            "a keyword one provider invented was sent to another: {travelling}"
        );
        assert!(travelling.contains(flag::SEEN), "{travelling}");
    }

    #[test]
    fn test_a_message_carrying_nothing_that_travels_carries_no_flag_list_at_all() {
        // Not an empty list. `APPEND folder () {n}` and `APPEND folder {n}` are
        // different commands, and a server may refuse the first.
        assert_eq!(the_flags_that_travel(&[]), None);
        assert_eq!(
            the_flags_that_travel(&[flag::DELETED.to_string(), "$Junk".to_string()]),
            None
        );
    }

    #[test]
    fn test_the_date_the_source_filed_it_is_the_date_the_append_names() {
        let named = when_it_arrived(Some("2021-03-04T09:15:00+00:00"))
            .expect("a date the source server named");

        assert!(named.contains("04-Mar-2021"), "{named}");
        assert!(named.contains("09:15:00"), "{named}");
        assert!(
            named.starts_with('"') && named.ends_with('"'),
            "an APPEND date is quoted, and this one goes into the command line \
             exactly as written: {named}"
        );
    }

    #[test]
    fn test_a_date_the_source_did_not_give_is_no_date_rather_than_a_made_up_one() {
        assert_eq!(when_it_arrived(None), None);
    }

    #[test]
    fn test_a_date_that_does_not_read_is_left_off_rather_than_passed_through() {
        // The library underneath puts this value into the command line with no
        // quoting and no checking, so anything that came from a server and was
        // not understood must not reach it.
        assert_eq!(when_it_arrived(Some("not a date")), None);
        assert_eq!(
            when_it_arrived(Some("\"} UID STORE 1 +FLAGS (\\Deleted)\r\n")),
            None
        );
    }

    #[tokio::test]
    async fn test_a_copy_across_accounts_says_nothing_to_the_source_that_changes_it() {
        // The whole safeguard in one assertion, and the source's own transcript
        // is the only thing that can see it. A crossing that quietly also
        // flagged or copied at the source would look identical from the
        // destination's side and from this function's return value.
        let source = a_server_holding_the_message(flag::SEEN, "01-Aug-2026 10:00:00 +0000").await;
        let destination = a_server_that_can("").await;

        waiting_for(copy_it_across(
            &the_account_it_is_in(&source).await,
            "INBOX",
            THE_UID,
            &the_account_it_is_going_to(&destination).await,
            "Archive",
        ))
        .await
        .expect("the copy to be made");

        let said = everything_said_to(&source).await;
        assert!(!said.to_uppercase().contains("STORE"), "{said}");
        assert!(!said.to_uppercase().contains("EXPUNGE"), "{said}");
        assert!(!said.to_uppercase().contains("COPY"), "{said}");
    }

    #[tokio::test]
    async fn test_the_destination_is_told_to_append_into_the_folder_that_was_chosen() {
        let source = a_server_holding_the_message(flag::SEEN, "01-Aug-2026 10:00:00 +0000").await;
        let destination = a_server_that_can("").await;

        waiting_for(copy_it_across(
            &the_account_it_is_in(&source).await,
            "INBOX",
            THE_UID,
            &the_account_it_is_going_to(&destination).await,
            "Archive",
        ))
        .await
        .expect("the copy to be made");

        let appends: Vec<String> = destination
            .transcript()
            .await
            .into_iter()
            .filter(|line| line.to_uppercase().contains("APPEND"))
            .collect();
        assert_eq!(appends.len(), 1, "{appends:?}");
        assert!(appends[0].contains("Archive"), "{appends:?}");
        assert!(appends[0].contains(flag::SEEN), "{appends:?}");
        assert!(appends[0].contains("01-Aug-2026"), "{appends:?}");
    }

    #[tokio::test]
    async fn test_the_message_the_destination_is_given_is_the_one_the_source_handed_over() {
        // The bytes, not a reading of them. The cache holds a parsed body and
        // an APPEND made from that would be a different message: header order,
        // folding, transfer encoding and every attachment change or vanish.
        let source = a_server_holding_the_message("", "01-Aug-2026 10:00:00 +0000").await;
        let destination = a_server_that_can("").await;

        waiting_for(copy_it_across(
            &the_account_it_is_in(&source).await,
            "INBOX",
            THE_UID,
            &the_account_it_is_going_to(&destination).await,
            "Archive",
        ))
        .await
        .expect("the copy to be made");

        let said = everything_said_to(&destination).await;
        assert!(
            said.contains("One o'clock?"),
            "the message the source handed over never reached the destination: {said}"
        );
    }

    #[tokio::test]
    async fn test_a_destination_that_refuses_the_append_leaves_the_message_where_it_was() {
        let source = a_server_holding_the_message(flag::SEEN, "01-Aug-2026 10:00:00 +0000").await;
        let destination = a_server_that_refuses("", "APPEND").await;

        let refused = waiting_for(copy_it_across(
            &the_account_it_is_in(&source).await,
            "INBOX",
            THE_UID,
            &the_account_it_is_going_to(&destination).await,
            "Archive",
        ))
        .await;

        assert!(
            refused.is_err(),
            "an append the destination refused came back as a copy that was made"
        );
        let said = everything_said_to(&source).await;
        assert!(!said.to_uppercase().contains("STORE"), "{said}");
        assert!(!said.to_uppercase().contains("EXPUNGE"), "{said}");
        assert!(!said.to_uppercase().contains("COPY"), "{said}");
    }

    #[tokio::test]
    async fn test_an_account_this_program_may_not_write_to_refuses_at_its_own_session() {
        // The refusal belongs to the destination account, so it happens at the
        // destination's session and nothing is sent to either server. A gate
        // read at the source would let a crossing into an account somebody had
        // marked read-only.
        let source = a_server_holding_the_message(flag::SEEN, "01-Aug-2026 10:00:00 +0000").await;
        let destination = a_server_that_can("").await;

        let refused = waiting_for(copy_it_across(
            &the_account_it_is_in(&source).await,
            "INBOX",
            THE_UID,
            &a_destination_this_program_may_not_write_to(&destination).await,
            "Archive",
        ))
        .await;

        let Err(why) = refused else {
            panic!("a message was appended to an account this program may not change");
        };
        assert!(
            why.to_string()
                .contains(crate::application::allowed::SETTINGS_SECTION),
            "the refusal does not name the setting that would allow it: {why}"
        );
        let said = everything_said_to(&destination).await;
        assert!(
            !said.to_uppercase().contains("APPEND"),
            "the append reached a server it was not allowed to reach: {said}"
        );
    }

    #[tokio::test]
    async fn test_a_source_that_says_nothing_about_the_message_is_not_a_copy_with_no_flags() {
        // The server answered and named a different message, which for this
        // UID is a message that is not there. Appending anyway would put a copy
        // at the destination unread and dated today, which is worse than not
        // copying it, because nothing afterwards says it happened that way.
        //
        // The fixture hands the bytes over and only withholds the headers, on
        // purpose. It used to be a server that answered neither, and against
        // that one this test passed however the code behaved: the body fetch
        // failed first, so a build with no check on the headers at all came
        // back as an error and looked right. Found by taking the red by hand
        // rather than by reading the test.
        let source = a_server_naming_a_different_message().await;
        let destination = a_server_that_can("").await;

        let refused = waiting_for(copy_it_across(
            &the_account_it_is_in(&source).await,
            "INBOX",
            THE_UID,
            &the_account_it_is_going_to(&destination).await,
            "Archive",
        ))
        .await;

        assert!(refused.is_err(), "{refused:?}");
        let said = everything_said_to(&destination).await;
        assert!(!said.to_uppercase().contains("APPEND"), "{said}");
    }
}

//! Whether this application may change anything that is not on this computer.
//!
//! Reading somebody's mail into a local cache cannot hurt them. Deleting a
//! message, sending one, or removing a task at their provider can, and none of
//! those paths has ever run for real. That is a good reason to hold off, and
//! the answer is a gate rather than more care: a gate can be proved, and care
//! has to be remembered every time somebody adds a method.
//!
//! So this wraps the HTTP client. Reading is free. A request that changes
//! something at the other end has to say so, and in read-only mode is refused
//! before it reaches the network, naming what it was going to do.
//!
//! # Why this shape
//!
//! The four provider clients each held a bare `reqwest::Client`, so a new
//! `delete_thing` was one line and nothing anywhere could tell it apart from a
//! read. Holding this instead means the compiler finds every one: a call that
//! changes something does not type-check unless it goes through
//! [`Outward::changing`].
//!
//! That is the whole point. A checkbox somebody has to honour is a checkbox
//! somebody forgets on the method they add at half past five.

use crate::common::{Error, Result};
use reqwest::{Method, RequestBuilder};

/// The client every provider is talked to through.
#[derive(Debug, Clone)]
pub struct Outward {
    http: reqwest::Client,
    /// Whether a request that changes something is allowed out.
    may_change: bool,
}

impl Default for Outward {
    /// Read only.
    ///
    /// The safe one is the default deliberately. A client built without
    /// anybody thinking about it should be the one that cannot damage
    /// somebody's mail, so forgetting fails towards doing nothing rather than
    /// towards deleting something.
    fn default() -> Self {
        Self::read_only(reqwest::Client::new())
    }
}

/// A read whose verb is not `GET`.
///
/// WebDAV asks with `PROPFIND` and `REPORT`. Both only ever ask a calendar
/// server what it has. A closed set of verbs rather than a bare [`Method`],
/// because [`Outward::reading_with`] used to take any method with nothing
/// stopping a caller handing it one that changes something.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AskWith {
    Propfind,
    Report,
}

impl AskWith {
    /// Every variant, for tests that have to check them all.
    ///
    /// Adding a variant does not add it here on its own: the compiler forces
    /// [`Self::verb`] to say what the new variant asks for, because that
    /// match is exhaustive, but nothing forces the variant onto this array as
    /// well. A verb left off it is a verb this reading has stopped checking.
    #[cfg(test)]
    const EVERY: [Self; 2] = [Self::Propfind, Self::Report];

    /// The word this asks with, as a request method.
    ///
    /// Parsed from the word rather than built the way these two verbs used to
    /// be, with an inherent constructor and an `unwrap` on the end of it.
    /// Both words are fixed and known to parse, so the error path is never
    /// really taken; it exists so nothing here unwraps.
    fn verb(self) -> Result<Method> {
        let word = match self {
            Self::Propfind => "PROPFIND",
            Self::Report => "REPORT",
        };
        word.parse()
            .map_err(|e| Error::Protocol(format!("{word} is not a request verb: {e}")))
    }
}

impl Outward {
    /// A client that can read and cannot change anything.
    ///
    /// What an account is pointed at somebody's real mail with while any of
    /// this is still unproven. Everything that only reads works normally:
    /// signing in, listing folders, fetching messages, pulling tasks and
    /// contacts and the calendar down. Nothing can be removed or sent.
    pub fn read_only(http: reqwest::Client) -> Self {
        Self {
            http,
            may_change: false,
        }
    }

    /// A client that may change things at the other end.
    ///
    /// Named rather than defaulted, so that turning it on is a decision
    /// somebody made and can be found in the code that made it.
    pub fn may_change_things(http: reqwest::Client) -> Self {
        Self {
            http,
            may_change: true,
        }
    }

    /// Whether changes are allowed out.
    pub const fn may_change(&self) -> bool {
        self.may_change
    }

    /// A request that only reads.
    ///
    /// Always allowed. Reading into a local cache cannot damage anything at
    /// the other end, and the cache is this application's own.
    pub fn reading(&self, url: &str) -> RequestBuilder {
        self.http.get(url)
    }

    /// A read that is not a `GET`.
    ///
    /// WebDAV asks for things with `PROPFIND` and `REPORT`, which look like
    /// writes from the method alone and are not: both only ever ask a calendar
    /// server what it has. Takes [`AskWith`] rather than a bare [`Method`] so
    /// that a changing verb cannot be passed here at all: this used to take
    /// any method with nothing gating it, which is a read gate a caller could
    /// send `DELETE` through.
    pub fn reading_with(&self, ask: AskWith, url: &str) -> Result<RequestBuilder> {
        Ok(self.http.request(ask.verb()?, url))
    }

    /// A request that changes something at the other end.
    ///
    /// `doing` says what, in the words somebody would want to read in a log or
    /// hear from the status line: "delete a task", "send a message". It is
    /// used when the request is refused, so it has to name the act rather than
    /// the endpoint.
    ///
    /// Refused before the network in read-only mode. Before, so that a refusal
    /// cannot half-happen.
    pub fn changing(&self, method: Method, url: &str, doing: &str) -> Result<RequestBuilder> {
        if !self.may_change {
            return Err(Error::Security(refusal(doing)));
        }
        Ok(self.http.request(method, url))
    }
}

/// One value, ready to be dropped into a query string.
///
/// A marker, a page number or a timestamp is the provider's to choose, and this
/// application's job is to hand it back unchanged. Interpolated raw, a value
/// holding an `&` splits into two parameters and one holding a `+` arrives as a
/// space, which is how every first calendar sync sent a broken timestamp.
///
/// For a query value only. It writes a space as `+`, which is right after a `?`
/// and wrong in a path segment, where a space has to be `%20`. It is also wrong
/// for a whole URL a provider handed back: those carry their own separators and
/// are used as they came.
pub fn in_a_query(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

/// One value, ready to be dropped into a path segment.
///
/// A calendar's own identifier is the provider's to choose and this
/// application's job is to put it into an address unchanged. Two of them cannot
/// go in raw: a Google calendar identifier that begins with `#`, which the
/// holidays and contacts calendars do, truncates the address at the fragment, so
/// the request asks about a calendar nobody named; and an identifier holding a
/// space breaks the request line in two.
///
/// Separate from [`in_a_query`] rather than shared with it, because that one
/// writes a space as `+`, which is right after a `?` and is a literal plus
/// inside a path. Written out here rather than taken from a crate, because it is
/// a dozen lines and the alternative is a dependency for them.
pub fn in_a_path(value: &str) -> String {
    let mut written = String::with_capacity(value.len());
    for byte in value.bytes() {
        // The unreserved set from RFC 3986. Everything else is escaped, which
        // is safe even where it was not strictly required.
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                written.push(char::from(byte));
            }
            _ => written.push_str(&format!("%{byte:02X}")),
        }
    }
    written
}

/// What somebody is told when a change was refused.
///
/// Says what was refused and why, because "operation not permitted" sends
/// somebody looking for a broken account or a network fault. The setting is
/// named so it can be found.
///
/// Public because IMAP and SMTP keep their own gates: they are sessions rather
/// than HTTP clients, so they cannot go through [`Outward`]. Somebody hearing
/// a refusal should not be able to tell which of the three refused, and three
/// copies of a sentence drift the moment they are written.
pub fn refusal(doing: &str) -> String {
    format!(
        "Refused to {doing}: this account is open for reading only. \
         Turn on Allow Changes for it to send or delete anything."
    )
}

/// Whether a command that would change something may go ahead.
///
/// The single most important decision in the mail transports, and the one
/// nothing tested. Mutation testing replaced it with "yes" and the suite stayed
/// green, which means the gate that makes an alpha build unable to reorganise
/// somebody's real mailbox had no check behind it at all.
///
/// A free function rather than a method, so it can be asked the question
/// without a socket. A session that holds the answer cannot be built without
/// one, and a safety property that can only be tested against a live server is
/// a safety property that does not get tested.
///
/// Here rather than in one of the transports, because three of them ask it.
/// [`Outward`] answers it inline for the HTTP clients; the mail and the POP
/// sessions call this. A second copy of it in a second file is how a change to
/// the rule comes to reach one transport and not the others.
pub fn permitted(may_change: bool, doing: &str) -> Result<()> {
    if may_change {
        return Ok(());
    }
    Err(Error::Security(refusal(doing)))
}

/// Whether a change was refused by the setting rather than by the provider.
///
/// One answer to the question, here beside the refusal itself, because it is
/// asked by every sync that sends anything and two answers is how one of them
/// comes to be wrong. [`Outward::changing`] raises this before the request is
/// built, so nothing left the machine and the change is still worth keeping.
pub fn was_refused_by_the_gate(error: &crate::common::Error) -> bool {
    matches!(error, crate::common::Error::Security(_))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading_only() -> Outward {
        Outward::read_only(reqwest::Client::new())
    }

    fn allowed() -> Outward {
        Outward::may_change_things(reqwest::Client::new())
    }

    #[test]
    fn test_a_value_going_into_a_path_keeps_a_space_out_and_a_hash_in() {
        // A Google calendar's own identifier can begin with a hash: the
        // holidays and the contacts calendars both do. Left raw, everything
        // from the hash on is a fragment the server never sees, so the request
        // asks about a calendar nobody named. A space written the way a query
        // wants it, as a plus, is a literal plus inside a path segment.
        assert_eq!(in_a_path("a b#c"), "a%20b%23c");
        assert!(!in_a_path("a b").contains('+'));
        assert_eq!(
            in_a_path("team@group.calendar.google.com"),
            "team%40group.calendar.google.com"
        );
        // An ordinary identifier goes through unchanged, so a log line stays
        // readable and an address that worked before still works.
        assert_eq!(in_a_path("primary-1_2.3~4"), "primary-1_2.3~4");
    }

    #[test]
    fn test_reading_is_always_allowed() {
        // The point of the whole thing: a real account can be pointed at this
        // safely and everything that only looks at it still works.
        let _ = reading_only().reading("https://example.com/messages");
    }

    #[test]
    fn test_a_change_is_refused_when_the_account_is_open_for_reading_only() {
        let refused = reading_only().changing(
            Method::DELETE,
            "https://example.com/messages/1",
            "delete a message",
        );

        assert!(refused.is_err(), "a delete went out on a read-only account");
    }

    #[test]
    fn test_the_refusal_says_what_it_refused_and_how_to_allow_it() {
        // "Operation not permitted" sends somebody looking for a broken
        // account or a firewall. This has to name the act and the setting.
        let Err(said) = reading_only().changing(
            Method::DELETE,
            "https://example.com/tasks/1",
            "delete a task",
        ) else {
            panic!("it was allowed");
        };
        let said = said.to_string();

        assert!(said.contains("delete a task"), "{said}");
        assert!(said.contains("Allow Changes"), "{said}");
    }

    #[test]
    fn test_a_change_goes_out_when_the_account_allows_it() {
        assert!(
            allowed()
                .changing(Method::POST, "https://example.com/tasks", "add a task")
                .is_ok()
        );
    }

    #[test]
    fn test_which_mode_it_is_in_can_be_asked() {
        // The interface has to be able to say so, because somebody whose
        // deletion was refused needs to know it was the setting rather than
        // the server.
        assert!(!reading_only().may_change());
        assert!(allowed().may_change());
    }

    #[test]
    fn test_nothing_may_change_a_mailbox_until_it_is_allowed_to() {
        // The most important line in the mail transports, and mutation testing
        // found it untested: replaced with "yes", the whole suite stayed green.
        // That gate is what stops an alpha build reorganising somebody's real
        // mailbox, and it had nothing behind it.
        let refused = permitted(false, "change a message");

        assert!(refused.is_err(), "a change was allowed with changes off");
        assert!(
            matches!(refused, Err(Error::Security(_))),
            "a refusal came back as something other than a refusal"
        );
    }

    #[test]
    fn test_a_refusal_says_what_was_being_attempted() {
        // "Permission denied" sends somebody looking for a broken account.
        // Naming the act is what tells them it was a setting instead.
        let Err(Error::Security(said)) = permitted(false, "delete a message") else {
            panic!("a change was allowed with changes off");
        };

        assert!(said.contains("delete a message"), "{said}");
    }

    #[test]
    fn test_a_session_that_is_allowed_to_change_things_may() {
        // The other direction. A gate that refuses everything would be safe
        // and useless, and nothing checked this either.
        assert!(permitted(true, "change a message").is_ok());
    }

    #[test]
    fn test_refusing_happens_before_anything_leaves() {
        // A refusal that happened halfway would be worse than no gate at all:
        // the request would be out and the answer lost. `changing` returns the
        // builder rather than the response, so nothing is sent until the
        // caller has already been given the chance to be refused.
        let refused = reading_only().changing(Method::PUT, "https://example.invalid/x", "write");

        assert!(refused.is_err());
    }

    #[test]
    fn test_a_read_that_is_not_a_get_can_only_ask_for_something() {
        // Ties this type's verbs to the HTTP changing list at :1430 rather
        // than restating "these are safe" a second time in a second place: if
        // a verb here were ever a changing one, the two statements of "which
        // verbs change something" would disagree.
        //
        // What this cannot see: a variant left off `EVERY`. The compiler
        // forces a new variant to be given a verb, because `verb` is an
        // exhaustive match, but nothing forces the variant onto `EVERY`
        // itself, so a third verb added and forgotten here would pass.
        const CHANGES_SOMETHING: [&str; 4] = ["POST", "PUT", "PATCH", "DELETE"];
        for ask in AskWith::EVERY {
            let verb = ask.verb().expect("a fixed verb name to parse");
            assert!(
                verb == Method::from_bytes(b"PROPFIND").expect("PROPFIND parses")
                    || verb == Method::from_bytes(b"REPORT").expect("REPORT parses"),
                "{verb} is not one of the two verbs this type is for"
            );
            assert!(
                !CHANGES_SOMETHING.contains(&verb.as_str()),
                "{verb} changes something, and a read must never be able to carry it"
            );
        }
    }

    #[test]
    fn test_a_read_that_is_not_a_get_builds_its_verb_without_unwrapping() {
        // Proof the verb name really turns into a request, alongside the
        // source check that nothing here reaches for `Method::from_bytes`
        // with an `unwrap` on the end of it.
        assert!(
            reading_only()
                .reading_with(AskWith::Propfind, "https://example.com/cal")
                .is_ok()
        );
        assert!(
            reading_only()
                .reading_with(AskWith::Report, "https://example.com/cal")
                .is_ok()
        );
    }
}

/// Every place this application can change something that is not ours.
///
/// Written down so the list can be checked rather than remembered. Each of
/// these holds its own gate, because a `reqwest::Client`, an IMAP session and
/// an SMTP client have nothing in common to share one.
///
/// The test below reads the source and fails if a module grows a raw
/// `reqwest::Client` field again, which is how all four HTTP clients came to
/// have one apiece with nothing able to tell a read from a delete.
#[cfg(test)]
const GATED: [&str; 7] = [
    "src/service/tasks_api.rs",
    "src/service/google_api.rs",
    "src/service/microsoft_graph.rs",
    "src/service/caldav.rs",
    "src/service/protocols/imap.rs",
    "src/service/protocols/smtp.rs",
    "src/service/protocols/pop3.rs",
];

/// Every place that opens a connection of its own and only ever reads.
///
/// Nothing here can change anything at somebody's account, so none of it needs
/// a gate. It is written down anyway, because the reason POP3 went years with
/// no gate at all is that nothing anywhere held a list of the ways out of this
/// program: a module that only read was indistinguishable from a module nobody
/// had thought about. Moving one of these into the list above is what happens
/// when it grows a write.
#[cfg(test)]
const TALKS_BUT_ONLY_READS: [&str; 3] = [
    // Fetches a published calendar. GET, and nothing else.
    "src/service/ical_subscription.rs",
    // Asks Google whether a link is on its lists. POSTs, and they are
    // questions: nothing at anybody's account changes.
    "src/service/safebrowsing/client.rs",
    // Trades an authorisation code for a token at the provider's own endpoint.
    "src/service/oauth.rs",
];

/// Every client that can change something at a provider with an HTTP request:
/// Google, Microsoft, the task lists behind both, and a calendar server.
///
/// The HTTP half only. IMAP, SMTP and POP3 write to a mail server over their
/// own line protocols rather than HTTP, so nothing here reads their requests
/// at all; [`MAIL_TRANSPORTS`] below is the census for those three, and reads
/// a different shape of line for the same reason.
///
/// The two lists below account for every write in these files between them,
/// and nothing outside them. The calendar server client was missing from here
/// for as long as this list existed, so its three writes were on neither list
/// and outside every count, which is the same shape of hole this file was
/// written to close. Mail was the same hole again, wider: it was missing not
/// from a list but from this census's whole idea of what a write is, so a new
/// IMAP, SMTP or POP3 write could arrive with nothing reading its request and
/// nothing failing.
#[cfg(test)]
const CLIENTS: [&str; 4] = [
    "src/service/google_api.rs",
    "src/service/microsoft_graph.rs",
    "src/service/tasks_api.rs",
    "src/service/caldav.rs",
];

/// Every HTTP provider write whose request has been read off a socket by a
/// test.
///
/// Every write these four HTTP clients make, not every write this
/// application makes: [`MAIL_MEASURED_ON_THE_WIRE`] is the same record for
/// IMAP, SMTP and POP3.
///
/// The gate above answers whether a change may go out. It says nothing about
/// what goes out, and that is where this project has found its worst faults: a
/// sender's address carrying the login name, a password column written as an
/// empty string. Neither was visible from the layer that decided to send.
///
/// Contacts and tasks were called safe on decision-layer coverage that never
/// reached a server: several writes had no test calling them in either
/// direction, and the deletion of a contact was measured only by a test
/// asserting that it was refused. A refusal is not a measurement of a deletion.
///
/// A row is the client's file, the method, the file whose tests read the
/// request off a socket, and the request line those tests assert. Naming the
/// file is what makes this checkable. Before, a row said only that a write was
/// measured somewhere, and the check that verified it looked only in the
/// client's own file, so a write measured one layer up read as unmeasured and
/// sat on the other list for months while it was covered the whole time.
///
/// What a row records is that some test stood up a server and read that verb
/// and that address off the socket. It does not record that anything read what
/// the request carried, and it cannot: a body is asserted by whatever test
/// wants to, in whatever words. Two of these rows are for writes whose bodies
/// nothing anywhere reads.
///
/// Two things about particular rows, so nobody reads one row's evidence as
/// another's. A row naming a test file a layer above the client is measured
/// through the sync that calls it, which is why the row says where. And the
/// calendar server client writes a create and a replace with the same verb to
/// the same shape of address, so those two rows are told apart only by the
/// file that measures them, and inside those tests by one sending
/// `If-None-Match` and the other `If-Match`.
///
/// What this list cannot see:
///
/// - Whether the assertion behind a row is any good. The check reads that the
///   named file stands up a server and asserts that request line. A test that
///   asserts the line and nothing else keeps it green.
/// - Whether the test asserting that line is about that method at all. Two
///   methods can write the same verb to the same address.
/// - A write that reaches a server some way the reading below does not know.
///
/// What it can see is the thing that was missing: a write added to one of these
/// files and never measured fails here instead of being remembered.
#[cfg(test)]
const MEASURED_ON_THE_WIRE: [(&str, &str, &str, &str); 21] = [
    (
        "src/service/google_api.rs",
        "create_contact",
        "src/service/google_api.rs",
        "POST /people:createContact",
    ),
    (
        "src/service/google_api.rs",
        "update_contact",
        "src/service/google_api.rs",
        "PATCH /people/c1:updateContact?",
    ),
    (
        "src/service/google_api.rs",
        "delete_contact",
        "src/service/google_api.rs",
        "DELETE /people/c1:deleteContact",
    ),
    (
        "src/service/google_api.rs",
        "create_event",
        "src/application/calendar.rs",
        "POST /calendars/primary/events",
    ),
    (
        "src/service/google_api.rs",
        "update_event",
        "src/service/google_api.rs",
        "PATCH /calendars/primary/events/evt1",
    ),
    (
        "src/service/google_api.rs",
        "delete_event",
        "src/application/calendar.rs",
        "DELETE /calendars/primary/events/evt1",
    ),
    (
        "src/service/microsoft_graph.rs",
        "create_contact",
        "src/service/microsoft_graph.rs",
        "POST /me/contacts",
    ),
    (
        "src/service/microsoft_graph.rs",
        "update_contact",
        "src/service/microsoft_graph.rs",
        "PATCH /me/contacts/AAMkAGI2",
    ),
    (
        "src/service/microsoft_graph.rs",
        "delete_contact",
        "src/service/microsoft_graph.rs",
        "DELETE /me/contacts/AAMk%2FAGI2%2B3%3Fx",
    ),
    (
        "src/service/microsoft_graph.rs",
        "create_event",
        "src/application/calendar.rs",
        "POST /me/events",
    ),
    (
        "src/service/microsoft_graph.rs",
        "update_event",
        "src/application/calendar.rs",
        "PATCH /me/events/evt1",
    ),
    (
        "src/service/microsoft_graph.rs",
        "delete_event",
        "src/application/calendar.rs",
        "DELETE /me/events/evt1",
    ),
    (
        "src/service/tasks_api.rs",
        "google_create_task",
        "src/service/tasks_api.rs",
        "POST /lists/list-1/tasks",
    ),
    (
        "src/service/tasks_api.rs",
        "google_update_task",
        "src/service/tasks_api.rs",
        "PATCH /lists/list-1/tasks/t-9",
    ),
    (
        "src/service/tasks_api.rs",
        "google_delete_task",
        "src/service/tasks_api.rs",
        "DELETE /lists/list-1/tasks/t-9",
    ),
    (
        "src/service/tasks_api.rs",
        "ms_create_task",
        "src/service/tasks_api.rs",
        "POST /me/todo/lists/list-1/tasks",
    ),
    (
        "src/service/tasks_api.rs",
        "ms_update_task",
        "src/service/tasks_api.rs",
        "PATCH /me/todo/lists/list-1/tasks/t-9",
    ),
    (
        "src/service/tasks_api.rs",
        "ms_delete_task",
        "src/service/tasks_api.rs",
        "DELETE /me/todo/lists/list-1/tasks/t-9",
    ),
    (
        "src/service/caldav.rs",
        "create_event",
        "src/service/caldav.rs",
        "PUT /dav/sam/work/e-1.ics",
    ),
    (
        "src/service/caldav.rs",
        "update_event",
        "src/application/caldav_sync.rs",
        "PUT /cal/e-1.ics",
    ),
    (
        "src/service/caldav.rs",
        "delete_event",
        "src/application/caldav_sync.rs",
        "DELETE /cal/e-1.ics",
    ),
];

/// Every file that writes to a mail server over its own line protocol, and
/// the fewest gated writes it must have.
///
/// [`CLIENTS`] above is the HTTP half of this application; this is the other
/// half, and for as long as this file existed it was not written down at
/// all. IMAP, SMTP and POP3 do not build a `reqwest::Request`, so nothing
/// that reads a request line ever looked at them, and a new write to any of
/// the three could arrive with nothing reading what it sent and nothing
/// failing. Mail is the surface where that matters most.
///
/// The number is the floor [`writes_in`] must find in that file, the same job
/// the `>= 3` floor plays in [`test_every_provider_write_is_on_one_of_the_two_wire_lists`]:
/// a reading that has gone blind reports a short answer instead of a true one,
/// and the floor is what turns that into a failure rather than a quiet count.
#[cfg(test)]
const MAIL_TRANSPORTS: [(&str, usize); 3] = [
    ("src/service/protocols/imap.rs", 8),
    ("src/service/protocols/smtp.rs", 2),
    ("src/service/protocols/pop3.rs", 1),
];

/// Every mail write whose command has been read off a socket by a test.
///
/// The mail analogue of [`MEASURED_ON_THE_WIRE`], and it reads the same way:
/// the file, the method, the file whose tests read the command off a socket,
/// and the command line those tests assert. What it cannot see is the same
/// list, unchanged: whether the assertion is any good, and whether it is
/// really about this method.
///
/// Two pairs here are told apart only by an argument and nothing else, the
/// same shape [`MEASURED_ON_THE_WIRE`]'s doc already names for the calendar
/// server's create and replace. `remove_these` and `delete_message` both
/// expunge and differ only by which UID is in the command, 4 against 7;
/// `send_email` and `send_raw` both open an SMTP conversation and differ only
/// by which address the row names.
#[cfg(test)]
const MAIL_MEASURED_ON_THE_WIRE: [(&str, &str, &str, &str); 10] = [
    (
        "src/service/protocols/imap.rs",
        "set_subscribed",
        "src/service/protocols/imap.rs",
        "SUBSCRIBE \"Work\"",
    ),
    (
        "src/service/protocols/imap.rs",
        "set_flag",
        "src/service/protocols/imap.rs",
        "UID STORE 7 +FLAGS (\\Seen)",
    ),
    (
        "src/service/protocols/imap.rs",
        "copy_message",
        "src/service/protocols/imap.rs",
        "UID COPY 7 \"Archive\"",
    ),
    (
        "src/service/protocols/imap.rs",
        "move_message",
        "src/service/protocols/imap.rs",
        "UID MOVE 7 \"Archive\"",
    ),
    (
        "src/service/protocols/imap.rs",
        "remove_these",
        "src/service/protocols/imap.rs",
        "UID EXPUNGE 4",
    ),
    (
        "src/service/protocols/imap.rs",
        "append_message",
        "src/service/protocols/imap.rs",
        "APPEND \"Sent\" (\\Seen)",
    ),
    (
        "src/service/protocols/imap.rs",
        "delete_message",
        "src/service/protocols/imap.rs",
        "UID EXPUNGE 7",
    ),
    (
        "src/service/protocols/pop3.rs",
        "delete",
        "src/service/protocols/pop3.rs",
        "DELE 3",
    ),
    (
        "src/service/protocols/smtp.rs",
        "send_email",
        "src/service/protocols/smtp.rs",
        "RCPT TO:<quiet@example.com>",
    ),
    (
        "src/service/protocols/smtp.rs",
        "send_raw",
        "src/service/protocols/smtp.rs",
        "MAIL FROM:<me@example.com>",
    ),
];

/// Every mail method that asks the gate and changes nothing on a server.
///
/// Without this the mail census would demand a changing command for a method
/// whose whole job is to send a read. `uids_with_message_id` asks the gate
/// because it is half of replacing a saved draft, the half that has to run
/// even on an account open for reading only would refuse the half that
/// follows it, but what it sends is `UID SEARCH`, which asks a server what it
/// has and changes nothing there.
#[cfg(test)]
const ASKS_THE_GATE_AND_CHANGES_NOTHING: [(&str, &str, &str); 1] = [(
    "src/service/protocols/imap.rs",
    "uids_with_message_id",
    "gated as half of replacing a saved draft; sends UID SEARCH and changes nothing, so it \
     has no request to measure",
)];

/// Every command a mail write is allowed to start with.
///
/// The mail analogue of the POST/PUT/PATCH/DELETE list further down: what
/// stops a row in [`MAIL_MEASURED_ON_THE_WIRE`] being satisfied by a test that
/// only asserts `FETCH`, `SEARCH`, `LIST` or `STAT`, none of which changes
/// anything at a mail server.
#[cfg(test)]
const MAIL_COMMAND_THAT_CHANGES_SOMETHING: [&str; 12] = [
    "UID STORE",
    "UID COPY",
    "UID MOVE",
    "UID EXPUNGE",
    "APPEND",
    "SUBSCRIBE",
    "UNSUBSCRIBE",
    "CREATE",
    "RENAME",
    "DELE",
    "MAIL FROM",
    "RCPT TO",
];

/// Every provider write whose request nobody has read yet.
///
/// Empty today. Every write in every client above has had its verb and its
/// address read off a socket by some test, which was already true when this
/// list still named five of them: it was written once and never re-derived,
/// and it went on telling readers that the calendar writes were uncovered
/// while they were covered a layer up. A list that says something is uncovered
/// when it is covered is worse than no list, because the reverse mistake is
/// what lets a real gap sit unnoticed.
///
/// Kept rather than deleted. An entry goes back on it the moment a write is
/// added with nothing reading its request, and the check below is what forces
/// that: a write on neither list fails. Empty here is a statement about the
/// tree, not a list nobody has kept up.
#[cfg(test)]
const NOT_MEASURED_ON_THE_WIRE: [(&str, &str); 0] = [];

/// The gate itself, which holds the client the others are refused through.
///
/// On a list of its own because it is the one file that is supposed to hold a
/// bare client. Putting it among the gated ones would make the check below it
/// read its own client as an escape route.
#[cfg(test)]
const THE_GATE: [&str; 1] = ["src/service/outward.rs"];

/// Every place that names a way out and cannot use it to reach anybody's
/// account: the crate is there for something other than its transport.
///
/// The paths each one may name are written down beside it, so a mail-sending
/// library that stops being an address parser and starts being a transport
/// fails here rather than being waved through on the strength of this list.
#[cfg(test)]
const NAMES_A_WAY_OUT_AND_CANNOT_CONNECT: [(&str, &str, &[&str]); 3] = [
    // Formats an address for a header. No transport.
    (
        "src/application/draft_message.rs",
        "lettre",
        &["lettre::Address", "lettre::message::Mailbox"],
    ),
    // The same, for the identifier a sent message carries.
    (
        "src/application/message_id.rs",
        "lettre",
        &["lettre::Address"],
    ),
    // The message-structure types only. No session, no connection.
    (
        "src/service/protocols/imap/structure.rs",
        "async_imap",
        &["async_imap::imap_proto"],
    ),
];

/// Every module only a test build ever compiles, with the declaration that
/// makes that true.
///
/// A loopback listener is a way out of this program by every reading a check
/// can make of its text. What stops it being one is the attribute on the
/// module declaration two files away, so that declaration is named here and
/// checked rather than trusted.
#[cfg(test)]
const ONLY_A_TEST_BUILD_COMPILES_THIS: [(&str, &str, &str); 1] =
    [("src/common/answering.rs", "src/common/mod.rs", "answering")];

#[cfg(test)]
mod completeness {
    use super::{
        ASKS_THE_GATE_AND_CHANGES_NOTHING, CLIENTS, GATED, MAIL_COMMAND_THAT_CHANGES_SOMETHING,
        MAIL_MEASURED_ON_THE_WIRE, MAIL_TRANSPORTS, MEASURED_ON_THE_WIRE,
        NAMES_A_WAY_OUT_AND_CANNOT_CONNECT, NOT_MEASURED_ON_THE_WIRE,
        ONLY_A_TEST_BUILD_COMPILES_THIS, TALKS_BUT_ONLY_READS, THE_GATE,
    };
    use crate::common::what_ships::what_ships;

    #[test]
    fn test_no_transport_holds_a_client_that_cannot_be_gated() {
        // The gate is only worth having if nothing goes round it. A module
        // that holds a bare reqwest::Client can call .delete() on it and
        // nothing in the type system objects, which is exactly the state all
        // four provider clients were in.
        for path in GATED {
            let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
            assert!(
                !source.contains("http: reqwest::Client"),
                "{path} holds a raw client again, so its writes are not gated"
            );
        }
    }

    /// Every place that talks to a calendar server without going through a
    /// provider client of its own.
    ///
    /// Adding a calendar asks a server what it has, which changes nothing
    /// there, and asks it nothing else. If either of these ever holds a client
    /// or reaches for the changing method, a write path has grown in a screen
    /// whose whole reason for existing is that it only reads.
    const READS_ONLY: [&str; 2] = [
        "src/application/calendar_source.rs",
        "src/presentation/wx_add_calendar.rs",
    ];

    #[test]
    fn test_the_screen_that_adds_a_calendar_cannot_change_anything_at_a_server() {
        for path in READS_ONLY {
            let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
            for way_round in ["reqwest::Client", "may_change_things", ".changing("] {
                assert!(
                    !source.contains(way_round),
                    "{path} uses {way_round}, so it can change something at a server"
                );
            }
            // And it really is the client that refuses, rather than no client
            // at all: a file this test read as empty would pass every line
            // above and prove nothing.
            assert!(
                source.contains("CalDavClient::new()") || source.contains("wxdragon"),
                "{path} does not look like the file this check was written for"
            );
        }
    }

    #[test]
    fn test_every_gated_module_is_still_there() {
        // A list of paths rots. If one is renamed, the test above passes by
        // reading nothing, so the list is checked separately. All five lists,
        // because the census below decides whether a file is accounted for by
        // looking it up in them, and a stale entry there accounts for nothing.
        let parked = NAMES_A_WAY_OUT_AND_CANNOT_CONNECT
            .iter()
            .map(|(path, _, _)| path)
            .chain(
                ONLY_A_TEST_BUILD_COMPILES_THIS
                    .iter()
                    .map(|(path, _, _)| path),
            );
        for path in GATED
            .iter()
            .chain(TALKS_BUT_ONLY_READS.iter())
            .chain(THE_GATE.iter())
            .chain(parked)
        {
            assert!(
                std::path::Path::new(path).exists(),
                "{path} has moved, so the list is stale and the checks that read it prove nothing"
            );
        }
    }

    /// Every crate a module has to name to get a connection of its own out of
    /// this program, written as the root of the paths it appears under.
    ///
    /// This used to be four literal call sites: `reqwest::Client::new(` and
    /// three like it. Ordinary Rust walked straight past them. A module that
    /// wrote `use reqwest::Client;` and then `Client::new()`, which is how
    /// most of this tree is written, reached a server and matched none of the
    /// four. It was proved by writing such a module, watching the census pass,
    /// and only then replacing the reading; the test below holds that proof
    /// after the module was removed.
    ///
    /// Crate names rather than call sites, because an item from another crate
    /// cannot be used without a name that crate owns appearing in the file:
    /// either the crate's own name in a path, or the item's name in a `use`.
    /// Renaming at the use site does not help, because `as` is written after
    /// the original name.
    ///
    /// `std::net` and `tokio::net` are roots with two segments because their
    /// crates carry far more than sockets, and that is why the second list
    /// below exists. A grouped import writes neither of those two segments
    /// together: `use tokio::{io::AsyncWriteExt, net::TcpStream};` names
    /// `tokio`, and this list can no more treat that as a way out than it can
    /// treat `use std::{collections::HashMap, mem};` as one. Ordinary style,
    /// already written that way in this tree, and a module written that way
    /// with a real connect and a real write walked past this census twice. So
    /// where the crate name cannot answer, the socket's own name does.
    const A_WAY_OUT_OF_THIS_PROGRAM: [&str; 9] = [
        "reqwest",
        "lettre",
        "async_imap",
        "native_tls",
        "tokio_native_tls",
        "tiny_http",
        "oauth2",
        "tokio::net",
        "std::net",
    ];

    /// Every name a socket is written under when the crate's own name is not
    /// enough to tell one apart from anything else that crate carries.
    ///
    /// One of these in a file means either a `use` that brought it in or a
    /// path that named it, and either way something in that file is holding
    /// the end of a connection.
    ///
    /// An address is not a connection, so `SocketAddr` is not here: a module
    /// that parses one and hands it to a gated client is not a way out, and
    /// the test halves in this tree that build one would be read as though
    /// they were.
    const A_SOCKET_UNDER_ITS_OWN_NAME: [&str; 8] = [
        "TcpStream",
        "TcpListener",
        "TcpSocket",
        "UdpSocket",
        "UnixStream",
        "UnixListener",
        "ToSocketAddrs",
        "lookup_host",
    ];

    /// Which way out the shipped half of `source` names, if any.
    ///
    /// Whole words only, and that is load-bearing rather than tidy: without
    /// it `pub mod xoauth2;` reads as the authorisation crate and every
    /// mention of `tokio_native_tls` reads as `native_tls` as well.
    ///
    /// What this cannot see. Written as a list rather than a sentence about
    /// how hard it tries, because the two rounds before this one each closed
    /// one hole and left a doc saying no arrangement of imports could hide
    /// anything, which was not true either time:
    ///
    /// - A module handed a live connection, or a built client, by another
    ///   module. Where the handing on is a re-export, a type alias or a public
    ///   signature on one line, the check below refuses it outright. A
    ///   signature broken across several lines, or a value that travels some
    ///   other way, is invisible here.
    /// - A macro that writes the path. Nothing looks inside one.
    /// - A crate nobody has classified. That is the manifest sweep further
    ///   down, not this.
    /// - A call straight into a networking library through the foreign
    ///   function interface. Nothing in this tree makes one today: the
    ///   libraries its foreign blocks name are the Windows user interface,
    ///   system, automation and accessibility ones, and none of them carries a
    ///   socket. Nothing checks that, so it is a reading of the tree on the day
    ///   this was written rather than a rule.
    /// - Anything outside `src`. The build script and the scripts beside it
    ///   are not read at all.
    /// - This reads names and not the program, so a name in a string, or in a
    ///   comment that trails code on the same line, counts as a way out. That
    ///   direction is the safe one: it asks somebody to classify a file
    ///   rather than passing one over in silence.
    fn how_it_reaches_a_server(source: &str) -> Option<&'static str> {
        let ships = what_ships(source);
        let code: String = ships
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        A_WAY_OUT_OF_THIS_PROGRAM
            .iter()
            .chain(A_SOCKET_UNDER_ITS_OWN_NAME.iter())
            .find(|name| names_it_as_a_whole_word(&code, name))
            .copied()
    }

    fn names_it_as_a_whole_word(code: &str, root: &str) -> bool {
        let a_name_character = |letter: char| letter.is_alphanumeric() || letter == '_';
        code.match_indices(root).any(|(at, _)| {
            let before_is_clear = code[..at]
                .chars()
                .next_back()
                .is_none_or(|letter| !a_name_character(letter));
            let after_is_clear = root.ends_with(':')
                || code[at + root.len()..]
                    .chars()
                    .next()
                    .is_none_or(|letter| !a_name_character(letter));
            before_is_clear && after_is_clear
        })
    }

    /// A declaration with its visibility taken off, and whether that
    /// visibility lets another module reach it.
    ///
    /// `pub(crate)` and `pub(super)` count as reaching another module, because
    /// that is the only question either reading below is asking: can a file
    /// that names no way out get one from here. This tree writes a restricted
    /// visibility in over a hundred places, so a reading that knew only the
    /// word `pub` was blind to ordinary style, and it was: a plain
    /// `pub(crate) fn` handing a socket out walked past the refusal below, and
    /// in the reading of writes it was not recognised as a method at all, so
    /// its body was counted against whichever method sat above it.
    fn past_the_visibility(declaration: &str) -> (&str, bool) {
        if let Some(rest) = declaration.strip_prefix("pub ") {
            return (rest, true);
        }
        if let Some(restricted) = declaration.strip_prefix("pub(")
            && let Some((_, rest)) = restricted.split_once(") ")
        {
            return (rest, true);
        }
        (declaration, false)
    }

    #[test]
    fn test_every_module_that_talks_to_a_server_is_on_one_of_these_lists() {
        // The check that would have caught the POP hole. The gate list was
        // kept by hand and POP3 was never on it, so "mail changes are off" was
        // never true for a POP account and nothing anywhere could say so.
        //
        // The lists are the record; the tree is the answer. A new way out of
        // this program now fails here until somebody has said in writing
        // whether it can change anything at a person's account.
        let accounted_for: Vec<&str> = GATED
            .iter()
            .chain(TALKS_BUT_ONLY_READS.iter())
            .chain(THE_GATE.iter())
            .copied()
            .chain(
                NAMES_A_WAY_OUT_AND_CANNOT_CONNECT
                    .iter()
                    .map(|(path, _, _)| *path),
            )
            .chain(
                ONLY_A_TEST_BUILD_COMPILES_THIS
                    .iter()
                    .map(|(path, _, _)| *path),
            )
            .collect();

        for file in every_source_file() {
            let path = file.to_string_lossy().replace('\\', "/");
            let source = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{path}: {e}"));
            // The shipped half only. A test may open a loopback socket, and
            // several do: that is how the gate is measured at all.
            let Some(how) = how_it_reaches_a_server(&source) else {
                continue;
            };
            assert!(
                accounted_for.contains(&path.as_str()),
                "{path} names {how} and is on no list, so nobody has said whether it can \
                 change anything at somebody's account"
            );
        }
    }

    #[test]
    fn test_the_census_sees_a_client_built_by_its_short_name() {
        // Proving the measurement, over text written here rather than the tree.
        // What it cannot see is whether the census is pointed at the right files.
        // The experiment, kept after the module it was run against was
        // removed. Each of these was a real reading of the tree at the time
        // the census was replaced.
        let reaches = |source: &str| how_it_reaches_a_server(source);

        // The idiom the old census missed, which is the ordinary one.
        assert_eq!(
            reaches("use reqwest::Client;\nfn go() { let c = Client::new(); }"),
            Some("reqwest")
        );
        // The idiom it caught.
        assert_eq!(
            reaches("fn go() { reqwest::Client::new(); }"),
            Some("reqwest")
        );
        // A client only a test build sees does not count.
        assert_eq!(
            reaches("#[cfg(test)]\nmod tests {\n    fn go() { reqwest::Client::new(); }\n}"),
            None
        );
        // And a test-only helper does not hide what follows it, which is how
        // the old reading went blind to nine tenths of two large files.
        assert_eq!(
            reaches("#[cfg(test)]\nfn h() {}\nfn go() { reqwest::get(u); }"),
            Some("reqwest")
        );
        // A module that goes through the gate is quiet.
        assert_eq!(
            reaches("use crate::service::outward::Outward;\nfn go(o: &Outward) { o.reading(u); }"),
            None
        );
        // And so is a near miss.
        assert_eq!(reaches("pub mod xoauth2;"), None);
        assert_eq!(
            reaches("use tokio_native_tls::TlsConnector;"),
            Some("tokio_native_tls")
        );
    }

    #[test]
    fn test_the_census_sees_a_socket_brought_in_beside_something_else() {
        // Proving the measurement, over text written here rather than the tree.
        // What it cannot see is whether the census is pointed at the right files.
        // The idiom that walked past this census twice. A module that brings a
        // socket in alongside something else from the same crate writes
        // neither `tokio::net` nor `std::net`, so a reading of path roots
        // alone finds nothing in it. This tree already writes its imports that
        // way in several places, so it is ordinary style rather than an
        // evasion somebody would have to think of.
        //
        // No fixture here says `#[cfg(test)]`. The guard record on the reading
        // of which half of a file ships names one test, and a fixture with
        // that text in it would redden this one under the same break and leave
        // the record short.
        let reaches = |source: &str| how_it_reaches_a_server(source);

        assert_eq!(
            reaches(
                "use tokio::{io::AsyncWriteExt, net::TcpStream};\n\
                 async fn go(a: &str) { let _ = TcpStream::connect(a).await; }"
            ),
            Some("TcpStream")
        );
        assert_eq!(
            reaches(
                "use std::{io::Write, net::TcpStream};\n\
                 fn go(a: &str) { let _ = TcpStream::connect(a); }"
            ),
            Some("TcpStream")
        );
        // The inline path still answers with the root, so nothing that was
        // seen before has moved.
        assert_eq!(
            reaches("use tokio::net::TcpStream;\nasync fn go() {}"),
            Some("tokio::net")
        );
        // The crates whose names are everywhere for other reasons stay quiet
        // when they are there for another reason.
        assert_eq!(
            reaches(
                "use tokio::{time::sleep, sync::oneshot};\nasync fn go(d: u64) { sleep(d).await; }"
            ),
            None
        );
        assert_eq!(
            reaches("use std::{collections::HashMap, fmt::Write, mem};"),
            None
        );
    }

    #[test]
    fn test_nothing_parked_as_harmless_names_more_than_it_said_it_would() {
        // A file on that list is excused because of what it uses the crate
        // for. If it starts using the crate for something else, the excuse
        // stops holding and nobody would otherwise notice.
        for (path, crate_name, allowed) in NAMES_A_WAY_OUT_AND_CANNOT_CONNECT {
            let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
            let ships = what_ships(&source);
            let mentions = ships.match_indices(crate_name).count();
            assert!(
                mentions > 0,
                "{path} no longer names {crate_name}, so this entry excuses nothing and \
                 should go"
            );
            for (at, _) in ships.match_indices(crate_name) {
                let from_here = &ships[at..];
                assert!(
                    allowed
                        .iter()
                        .any(|path_it_may_name| from_here.starts_with(path_it_may_name)),
                    "{path} names {crate_name} in a way this list did not allow: {}",
                    from_here.lines().next().unwrap_or_default()
                );
            }
        }
    }

    #[test]
    fn test_nothing_parked_as_test_only_is_built_by_the_program() {
        // Checked rather than asserted from memory: the attribute that makes
        // this true lives in another file and nothing else would notice it
        // going.
        for (path, parent, name) in ONLY_A_TEST_BUILD_COMPILES_THIS {
            let declared = std::fs::read_to_string(parent)
                .unwrap_or_else(|e| panic!("{parent}: {e}"))
                .replace("\r\n", "\n");
            assert!(
                declared.contains(&format!("#[cfg(test)]\npub mod {name};")),
                "{path} is called test-only, but {parent} declares it without the attribute \
                 that makes that true, so a release build compiles a way out of this program"
            );
        }
    }

    /// Every dependency that can reach a server, by the name Cargo knows it by.
    ///
    /// The census reads path roots, which is a different granularity from a
    /// crate name and will drift from it if nothing holds them together. The
    /// test below is what holds them together, and it is also what makes a
    /// newly added networking crate fail until somebody has classified it,
    /// which is the one blind spot the census cannot close by reading harder.
    ///
    /// `std::net` is a root with no crate behind it, so it appears in the
    /// census list and not here.
    const A_CRATE_THAT_CAN_REACH_A_SERVER: [&str; 8] = [
        "lettre",
        "reqwest",
        "oauth2",
        "tiny_http",
        "async-imap",
        "tokio-native-tls",
        "native-tls",
        "tokio",
    ];

    /// Every other dependency. Written down rather than left implicit, so that
    /// adding one has to be a decision and cannot be an omission.
    const A_CRATE_THAT_CANNOT: [&str; 39] = [
        "uuid",
        "chrono",
        "chrono-tz",
        "serde",
        "serde_json",
        "toml",
        "tracing",
        "tracing-subscriber",
        "tracing-appender",
        "dirs",
        "async-channel",
        "futures",
        "mail-parser",
        "wxdragon",
        "rusqlite",
        "base64",
        "ammonia",
        "regex",
        "html-escape",
        "aes-gcm",
        "rand",
        "spellbook",
        "unicode-segmentation",
        "open",
        "keyring",
        "url",
        "icalendar",
        "quick-xml",
        "scraper",
        "ego-tree",
        "bitflags",
        "pdfpurr",
        "sha2",
        "pulldown-cmark",
        "windows",
        "winresource",
        "boa_engine",
        "tokio-test",
        "tempfile",
    ];

    /// Every dependency this project builds against, read from the manifest.
    ///
    /// Read rather than listed, for the reason every check in this file is
    /// read rather than listed: a written copy of a fact goes stale and stops
    /// being about anything.
    fn every_dependency() -> Vec<String> {
        let manifest = std::fs::read_to_string("Cargo.toml").expect("Cargo.toml");
        let mut names = Vec::new();
        let mut inside = false;
        for line in manifest.lines().map(str::trim) {
            if line.starts_with('[') {
                inside = line.ends_with("dependencies]");
                continue;
            }
            let Some((name, _)) = line.split_once(" =") else {
                continue;
            };
            let ordinary = !name.is_empty()
                && name.chars().all(|letter| {
                    letter.is_ascii_lowercase()
                        || letter.is_ascii_digit()
                        || matches!(letter, '_' | '-')
                });
            if inside && ordinary {
                names.push(name.to_string());
            }
        }
        assert!(
            names.len() > 30,
            "only {} dependencies were read, so the manifest is not being parsed",
            names.len()
        );
        names
    }

    #[test]
    fn test_every_dependency_has_been_told_apart_from_a_way_out() {
        for name in every_dependency() {
            let reaching = A_CRATE_THAT_CAN_REACH_A_SERVER.contains(&name.as_str());
            let not = A_CRATE_THAT_CANNOT.contains(&name.as_str());
            assert!(
                reaching != not,
                "{name} is a dependency and is on {}, so nobody has said whether a module \
                 using it can reach a server, and the census cannot see it",
                match reaching {
                    true => "both lists",
                    false => "neither list",
                }
            );
        }
    }

    #[test]
    fn test_neither_list_of_crates_has_gone_stale() {
        // The two lists are the artefact. What this cannot see is whether the
        // census built from them finds everything that reaches the network.
        // The other direction. A list naming a dependency that has gone is a
        // list nobody is reading, and it would let the check above pass by
        // classifying nothing.
        let real = every_dependency();
        for name in A_CRATE_THAT_CAN_REACH_A_SERVER
            .iter()
            .chain(A_CRATE_THAT_CANNOT.iter())
        {
            assert!(
                real.iter().any(|dependency| dependency == name),
                "{name} is on one of these lists and is not a dependency any more"
            );
        }
    }

    #[test]
    fn test_every_crate_that_can_reach_a_server_is_a_way_the_census_looks_for() {
        // Crate name and path root are two granularities of one fact. Held
        // together here so that classifying a crate as reaching, and then not
        // teaching the census its name, fails instead of going quiet.
        for name in A_CRATE_THAT_CAN_REACH_A_SERVER {
            let as_a_path = name.replace('-', "_");
            assert!(
                A_WAY_OUT_OF_THIS_PROGRAM
                    .iter()
                    .any(|root| root.starts_with(as_a_path.as_str())),
                "{name} can reach a server and the census looks for no path starting {as_a_path}, \
                 so a module using it is invisible"
            );
        }
    }

    #[test]
    fn test_no_module_hands_a_way_out_on_under_another_name() {
        // The door the name reading leaves open. One listed file could pass a
        // client or a live socket outward, and a second file would then reach
        // a server naming nothing this can see. Refused outright, rather than
        // chased.
        //
        // Both lists, because a re-exported socket type hides a module exactly
        // as well as a re-exported client does.
        //
        // What this cannot see: it reads one line at a time, so a public
        // signature broken across several lines hands a way out on and walks
        // past. That is the blind spot written on the reading itself, and it
        // is here rather than closed because a check that joined lines would
        // have to decide where a signature ends, and every reading of source
        // text that has had to decide that in this file has been wrong at
        // least once.
        //
        // The gate is skipped because handing out a request builder is what it
        // is for, and the test-only helpers are skipped because a release
        // build never compiles them.
        let skipped: Vec<&str> = THE_GATE
            .iter()
            .copied()
            .chain(
                ONLY_A_TEST_BUILD_COMPILES_THIS
                    .iter()
                    .map(|(path, _, _)| *path),
            )
            .collect();
        for file in every_source_file() {
            let path = file.to_string_lossy().replace('\\', "/");
            if skipped.contains(&path.as_str()) {
                continue;
            }
            let source = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{path}: {e}"));
            for line in what_ships(&source).lines() {
                let (declared, reaches_another_module) = past_the_visibility(line.trim_start());
                // A type alias counts whatever its visibility, because the
                // point of one is that the name it gives is used instead of
                // the name it stands for.
                let handing_on = declared.starts_with("type ")
                    || (reaches_another_module
                        && (declared.starts_with("use ")
                            || declared.starts_with("fn ")
                            || declared.starts_with("async fn ")
                            || declared.starts_with("const fn ")
                            // A field, which hands the thing itself on rather
                            // than a way of getting it.
                            || line.contains(':')));
                if !handing_on {
                    continue;
                }
                for name in A_WAY_OUT_OF_THIS_PROGRAM
                    .iter()
                    .chain(A_SOCKET_UNDER_ITS_OWN_NAME.iter())
                {
                    assert!(
                        !names_it_as_a_whole_word(line, name),
                        "{path} hands {name} on under another name, so a file that names \
                         nothing can reach a server through it: {}",
                        line.trim()
                    );
                }
            }
        }
    }

    #[test]
    fn test_nothing_builds_a_request_verb_out_of_bytes() {
        // `Method::from_bytes` is the only way a caller could hand
        // `reading_with` a verb of its own rather than one of the two
        // [`super::AskWith`] allows, and it is also where the `unwrap` used
        // to live that CLAUDE.md forbids outside tests. Banning the idiom
        // here is what keeps a second custom verb from being built the same
        // unchecked way somewhere else in the tree.
        for file in every_source_file() {
            let path = file.to_string_lossy().replace('\\', "/");
            let source = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{path}: {e}"));
            assert!(
                !what_ships(&source).contains("Method::from_bytes"),
                "{path} builds a request verb out of bytes, which is how the ungated \
                 unwrap this file exists to close got there in the first place"
            );
        }
    }

    /// How a method in a provider client reaches a server with a change.
    ///
    /// Three of these clients funnel their changes through a helper of their
    /// own that holds the gate. The fourth asks the gate directly, so the
    /// gate's own name is here as well, and that one catches any client
    /// written that way in future without anybody having to add its helper.
    ///
    /// A write that reaches a server some other way is the blind spot named in
    /// the doc comment on the list this serves.
    const SENDS_A_CHANGE: [&str; 6] = [
        "api_post",
        "api_patch",
        "api_delete",
        "self.send(",
        "self.delete(",
        ".changing(",
    ];

    /// How a method in a mail transport asks whether it may send.
    ///
    /// A marker list of its own rather than folded into [`SENDS_A_CHANGE`],
    /// even though both are read by the same [`writes_in`]. The four HTTP
    /// clients never call [`super::permitted`] today, so sharing one list
    /// would happen to be quiet, but the day one of them did, the HTTP census
    /// would start reading a mail marker and the mail census would start
    /// reading an HTTP one, and neither would be reading its own question.
    const SENDS_A_MAIL_CHANGE: [&str; 2] = ["self.may_i(", "outward::permitted("];

    /// A test's source text read as the bytes it puts on the wire.
    ///
    /// A row in [`MAIL_MEASURED_ON_THE_WIRE`] names a command as a server
    /// sees it. The test that asserts it is Rust source, so `\Seen` is
    /// written `\\Seen` and `"Work"` is written `\"Work\"`. Collapsing those
    /// is what lets a row be written the way it goes out rather than the way
    /// it is escaped in the file that asserts it.
    ///
    /// Order matters: the double backslash first, then the escaped quote, or
    /// a line escaping a quoted backslash would collapse wrong.
    fn as_it_goes_on_the_wire(tests: &str) -> String {
        tests.replace("\\\\", "\\").replace("\\\"", "\"")
    }

    /// Everything in a source file before its tests begin.
    fn the_production_half(source: &str) -> &str {
        source.split("mod tests {").next().unwrap_or_default()
    }

    /// Everything in a source file from its tests on.
    fn the_test_half(source: &str) -> &str {
        source
            .split_once("mod tests {")
            .map_or("", |(_, tests)| tests)
    }

    /// The method a line starts, and whether anything outside the file can
    /// call it, for a line sitting at the top level of an `impl` block.
    ///
    /// Every method rather than only the public ones, which is the whole point
    /// of it. A reading that only knew where public methods began ran each
    /// chunk on to the next one, so a private helper between two of them was
    /// read as part of the one above. That was harmless while the markers were
    /// the helpers' own names and stopped being harmless the moment the gate
    /// call became one: one client keeps its two sending helpers directly
    /// under a public read, and that read would have been reported as a write.
    ///
    /// A method another module can reach counts as public here, whether it
    /// says `pub` or narrows that to the crate. What a write is called from is
    /// not the question; whether anything outside this file can call it is.
    fn method_named(line: &str) -> Option<(&str, bool)> {
        let at_the_top_of_an_impl = line.strip_prefix("    ")?;
        if at_the_top_of_an_impl.starts_with(' ') {
            return None;
        }
        let (declaration, public) = past_the_visibility(at_the_top_of_an_impl);
        let declaration = declaration.strip_prefix("const ").unwrap_or(declaration);
        let declaration = declaration.strip_prefix("async ").unwrap_or(declaration);
        let named = declaration.strip_prefix("fn ")?;
        Some((
            named
                .split(|letter: char| !(letter.is_alphanumeric() || letter == '_'))
                .next()
                .unwrap_or_default(),
            public,
        ))
    }

    /// Every public method in one client whose body holds one of `markers`.
    ///
    /// Read off the tree rather than listed, so a write added next year has to
    /// be accounted for instead of remembered. Takes its markers as an
    /// argument rather than reading [`SENDS_A_CHANGE`] itself, so the HTTP
    /// census and the mail census below can each hand it the question that is
    /// really theirs instead of sharing one list between two different
    /// protocols.
    ///
    /// A chunk runs from one method's own line to the next method's, so a
    /// marker is counted against the method it is really in. The signature
    /// line counts too, which catches a method written on one line and costs
    /// nothing else: no public method in these clients is named after one of
    /// the markers.
    fn writes_in(production: &str, markers: &[&str]) -> Vec<String> {
        let keep = |inside: Option<(&str, bool)>, sends: bool, writes: &mut Vec<String>| {
            if let Some((name, public)) = inside
                && public
                && sends
            {
                writes.push(name.to_string());
            }
        };
        let mut writes = Vec::new();
        let mut inside: Option<(&str, bool)> = None;
        let mut sends = false;
        for line in production.lines() {
            if let Some(started) = method_named(line) {
                keep(inside, sends, &mut writes);
                inside = Some(started);
                sends = false;
            }
            sends |= markers.iter().any(|how| line.contains(how));
        }
        keep(inside, sends, &mut writes);
        writes
    }

    #[test]
    fn test_every_provider_write_is_on_one_of_the_two_wire_lists() {
        // The check that would have caught this round's gap. Six task writes
        // and one contact deletion sent changes to somebody's account with no
        // test anywhere reading what they sent, and nothing could say so,
        // because nothing held a list of the ways this program changes
        // something at a provider.
        let mut on_neither_list: Vec<String> = Vec::new();
        let mut found_altogether = 0;
        for path in CLIENTS {
            let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
            let production = the_production_half(&source);
            let writes = writes_in(production, &SENDS_A_CHANGE);
            // Before believing a short answer, the reading has to be able to
            // find anything at all. A file this read as empty, or one whose
            // methods it no longer recognises, would pass by looking at
            // nothing. Three is what the smallest of these clients has; the
            // total below is what stops three files reading as empty while the
            // fourth carries the check.
            assert!(
                writes.len() >= 3,
                "{path}: only {} writes were found, so the way a write is recognised has \
                 moved and this is looking at the wrong thing",
                writes.len()
            );
            found_altogether += writes.len();
            for write in writes {
                let accounted_for = MEASURED_ON_THE_WIRE
                    .iter()
                    .map(|(file, method, _, _)| (*file, *method))
                    .chain(NOT_MEASURED_ON_THE_WIRE.iter().copied())
                    .any(|(file, method)| file == path && method == write);
                if !accounted_for {
                    on_neither_list.push(format!("{path}: {write}"));
                }
            }
        }

        assert!(
            found_altogether >= 21,
            "only {found_altogether} writes were found across all of these clients, so the \
             way a write is recognised has moved and this is looking at the wrong thing"
        );

        assert!(
            on_neither_list.is_empty(),
            "these change something at somebody's account and are on neither list, so nobody \
             has said whether what they send has ever been read off a socket: {on_neither_list:?}"
        );
    }

    #[test]
    fn test_every_write_called_measured_had_its_request_read_off_a_socket() {
        // The other half. The list above only says a write is accounted for;
        // this says the account is true.
        //
        // It asks the file the row names and no other. That is narrower than
        // it sounds and was arrived at the hard way, twice. Written to look
        // only in the client's own file, it called five calendar writes
        // unmeasured for months while a sync one layer up was reading every
        // one of their requests off a socket. Written to look in any file that
        // stands up a server, it went green on writes nothing measured,
        // because a calendar server client two files away has methods of the
        // same names. Neither reading could be right, because the row did not
        // say where. Now it does.
        //
        // What this still cannot say is whether the assertion behind a row is
        // worth anything, or that it is about that method at all. Both are
        // written down where the list is.
        for (client, method, measured_in, request) in MEASURED_ON_THE_WIRE {
            let source =
                std::fs::read_to_string(client).unwrap_or_else(|e| panic!("{client}: {e}"));
            assert!(
                the_production_half(&source).contains(&format!("async fn {method}")),
                "{client}: {method} is called measured on the wire and is not in that file \
                 any more, so this row is about nothing"
            );

            let verb = request.split(' ').next().unwrap_or_default();
            assert!(
                ["POST", "PUT", "PATCH", "DELETE"].contains(&verb),
                "{client}: {method} is called measured by a request line that asks for \
                 something rather than changing it: {request}"
            );

            let holding = std::fs::read_to_string(measured_in)
                .unwrap_or_else(|e| panic!("{measured_in}: {e}"));
            let tests = the_test_half(&holding);
            assert!(
                tests.contains("answering"),
                "{measured_in}: nothing in its tests stands up a server, so no write can have \
                 been read off one there"
            );
            assert!(
                tests.contains(request),
                "{client}: {method} is called measured on the wire by {measured_in}, and no \
                 test there asserts {request}"
            );
        }
    }

    #[test]
    fn test_every_mailbox_write_is_on_the_mail_wire_list() {
        // The check the census never had. Four HTTP clients were covered and
        // no mail protocol was, so a new IMAP, SMTP or POP3 write could
        // arrive with nothing reading its request and nothing failing. POP3
        // shipped its one write ungated for the whole life of this project
        // behind exactly that gap.
        let mut on_neither_list: Vec<String> = Vec::new();
        for (path, floor) in MAIL_TRANSPORTS {
            let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
            let production = the_production_half(&source);
            let writes = writes_in(production, &SENDS_A_MAIL_CHANGE);
            assert!(
                writes.len() >= floor,
                "{path}: only {} writes ask the gate, so the way a write is recognised has \
                 moved and this is looking at the wrong thing",
                writes.len()
            );
            for write in writes {
                let accounted_for = MAIL_MEASURED_ON_THE_WIRE
                    .iter()
                    .map(|(file, method, _, _)| (*file, *method))
                    .any(|(file, method)| file == path && method == write)
                    || ASKS_THE_GATE_AND_CHANGES_NOTHING
                        .iter()
                        .any(|(file, method, _)| *file == path && *method == write);
                if !accounted_for {
                    on_neither_list.push(format!("{path}: {write}"));
                }
            }
        }

        assert!(
            on_neither_list.is_empty(),
            "these change something at a mail server and are on neither list, so nobody has \
             said whether what they send has ever been read off a socket: {on_neither_list:?}"
        );
    }

    #[test]
    fn test_every_mailbox_write_called_measured_had_its_command_read_off_a_socket() {
        // The other half, for mail: the list above only says a write is
        // accounted for; this says the account is true.
        for (client, method, measured_in, request) in MAIL_MEASURED_ON_THE_WIRE {
            let source =
                std::fs::read_to_string(client).unwrap_or_else(|e| panic!("{client}: {e}"));
            assert!(
                the_production_half(&source).contains(&format!("fn {method}")),
                "{client}: {method} is called measured on the wire and is not in that file \
                 any more, so this row is about nothing"
            );

            assert!(
                MAIL_COMMAND_THAT_CHANGES_SOMETHING
                    .iter()
                    .any(|opens| request.starts_with(opens)),
                "{client}: {method} is called measured by a command that does not open with \
                 something that changes a mailbox: {request}"
            );

            let holding = std::fs::read_to_string(measured_in)
                .unwrap_or_else(|e| panic!("{measured_in}: {e}"));
            let tests = as_it_goes_on_the_wire(the_test_half(&holding));
            assert!(
                tests.contains("answering"),
                "{measured_in}: nothing in its tests stands up a server, so no command can \
                 have been read off one there"
            );
            assert!(
                tests.contains(request),
                "{client}: {method} is called measured on the wire by {measured_in}, and no \
                 test there asserts {request} as it goes on the wire"
            );
        }
    }

    /// Every call that actually leaves a mail server changed, by the literal
    /// text it is written with, per file.
    ///
    /// The census above answers "was a write measured"; this answers "did a
    /// write ask the gate before it happened", and it does not depend on
    /// anybody remembering to call [`super::permitted`]. Most IMAP writes go
    /// through the library's own helpers (`self.session.subscribe`,
    /// `.uid_copy`, `.uid_mv`, `.append`), so the protocol keyword the census
    /// above reads is inside `async_imap` and not in this tree at all: a
    /// check that looked for `"UID COPY"` in imap.rs would find nothing and
    /// pass. What is bounded and checkable instead is the call into the
    /// session, because a command cannot reach an IMAP server without
    /// touching `self.session`, cannot reach a POP server without
    /// `self.command("DELE"`, and cannot reach an SMTP server without the
    /// lettre transport.
    const A_CALL_THAT_CHANGES_A_MAILBOX: [(&str, &[&str]); 3] = [
        (
            "src/service/protocols/imap.rs",
            &[
                "self.session.subscribe(",
                "self.session.unsubscribe(",
                "self.session.uid_copy(",
                "self.session.uid_mv(",
                "self.session.append(",
                "\"UID STORE",
                "\"UID EXPUNGE",
            ],
        ),
        ("src/service/protocols/pop3.rs", &["\"DELE\""]),
        (
            "src/service/protocols/smtp.rs",
            &["AsyncSmtpTransport", ".send(message)", ".send_raw("],
        ),
    ];

    /// A private helper that sends, and every method in the same file that
    /// calls it.
    ///
    /// [`test_nothing_sends_a_changing_command_without_asking_the_gate`]
    /// cannot see a write behind one more level of indirection on its own: a
    /// private helper's own chunk holds the marker, not its caller's, so the
    /// helper is named here with every real caller, and the check below holds
    /// each of those callers to being gated in its own right rather than
    /// trusting the exception blindly.
    ///
    /// Both entries were verified by reading every call site rather than
    /// assumed: `expunge_one` is reached from three places, not the two an
    /// earlier reading of this file named, because `remove_these` calls it as
    /// well as `move_message` and `delete_message`. All three ask the gate
    /// before they reach it. `transport` builds a connection and sends
    /// nothing itself; it is reached only from the two gated public methods
    /// that send.
    const NOT_GATED_AND_DOES_NOT_NEED_TO_BE: [(&str, &str, &[&str]); 2] = [
        (
            "src/service/protocols/imap.rs",
            "expunge_one",
            &["move_message", "remove_these", "delete_message"],
        ),
        (
            "src/service/protocols/smtp.rs",
            "transport",
            &["send_email", "send_raw"],
        ),
    ];

    /// Every method in `production`, in the order it appears, paired with its
    /// whole body as text.
    ///
    /// Every method rather than only the public ones that [`writes_in`]
    /// keeps: a private helper that sends has to be seen on its own, not
    /// merged into whichever public method happens to sit above it in the
    /// file, because [`NOT_GATED_AND_DOES_NOT_NEED_TO_BE`] has to be able to
    /// read that helper's chunk apart from its callers'.
    fn method_chunks(production: &str) -> Vec<(&str, String)> {
        let mut chunks = Vec::new();
        let mut inside: Option<&str> = None;
        let mut body = String::new();
        for line in production.lines() {
            if let Some((name, _)) = method_named(line) {
                if let Some(name) = inside.replace(name) {
                    chunks.push((name, std::mem::take(&mut body)));
                }
            }
            if inside.is_some() {
                body.push_str(line);
                body.push('\n');
            }
        }
        if let Some(name) = inside {
            chunks.push((name, body));
        }
        chunks
    }

    #[test]
    fn test_nothing_sends_a_changing_command_without_asking_the_gate() {
        // The guard that does not depend on anybody remembering to ask the
        // gate: it reads where a write actually leaves rather than where a
        // method happens to be named, which is the shape of hole POP3 shipped
        // in for the life of this project.
        for (path, markers) in A_CALL_THAT_CHANGES_A_MAILBOX {
            let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
            let production = the_production_half(&source);
            let chunks = method_chunks(production);
            assert!(
                !chunks.is_empty(),
                "{path}: no methods were found, so the way a method is recognised has moved \
                 and this is looking at the wrong thing"
            );
            for (name, body) in &chunks {
                let sends = markers.iter().any(|marker| body.contains(marker));
                if !sends {
                    continue;
                }
                let gated = SENDS_A_MAIL_CHANGE
                    .iter()
                    .any(|marker| body.contains(marker));
                if gated {
                    continue;
                }
                let Some((_, _, callers)) = NOT_GATED_AND_DOES_NOT_NEED_TO_BE
                    .iter()
                    .find(|(file, method, _)| *file == path && method == name)
                else {
                    panic!(
                        "{path}: {name} sends a change to a mail server and asks nothing \
                         before it does, and it is not the named caller of anything gated \
                         either"
                    );
                };
                for caller in *callers {
                    let (_, caller_body) = chunks
                        .iter()
                        .find(|(chunk_name, _)| chunk_name == caller)
                        .unwrap_or_else(|| {
                            panic!(
                                "{path}: {name} is excepted as called by {caller}, and {caller} \
                                 is not a method in this file any more"
                            )
                        });
                    assert!(
                        caller_body.contains(&format!(".{name}(")),
                        "{path}: {name} is excepted as called by {caller}, and {caller} does \
                         not call it"
                    );
                    assert!(
                        SENDS_A_MAIL_CHANGE
                            .iter()
                            .any(|marker| caller_body.contains(marker)),
                        "{path}: {name} is excepted because {caller} calls it and is gated, \
                         and {caller} does not ask the gate"
                    );
                }
            }
        }
    }

    /// Every Rust file under `src`, however deep.
    fn every_source_file() -> Vec<std::path::PathBuf> {
        let mut found = Vec::new();
        let mut looking = vec![std::path::PathBuf::from("src")];
        while let Some(dir) = looking.pop() {
            let entries = std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{dir:?}: {e}"));
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    looking.push(path);
                } else if path.extension().is_some_and(|kind| kind == "rs") {
                    found.push(path);
                }
            }
        }
        assert!(
            found.len() > 50,
            "only {} source files were found, so this walked the wrong tree",
            found.len()
        );
        found
    }
}

#[cfg(test)]
mod wiring {
    /// Every client that can change something, and the constructor that asks
    /// what the account allows before building it.
    ///
    /// A gate nothing calls is a gate nobody goes through. The whole point of
    /// this was that five of the six were built with the refusing constructor
    /// and nothing ever chose otherwise, so the application could not send,
    /// flag, delete or sync anything at all.
    const WIRED: [(&str, &str); 5] = [
        ("src/service/tasks_api.rs", "pub fn for_account"),
        ("src/service/google_api.rs", "pub fn for_account"),
        ("src/service/microsoft_graph.rs", "pub fn for_account"),
        ("src/service/caldav.rs", "pub fn for_account"),
        // These two are sessions rather than clients, so they take the
        // account at the point of connecting instead.
        (
            "src/application/mail_controller.rs",
            "allowed_for(account_id).mail",
        ),
    ];

    #[test]
    fn test_every_gated_client_has_something_that_can_turn_it_on() {
        for (path, wanted) in WIRED {
            let source = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
            assert!(
                source.contains(wanted),
                "{path} can refuse but nothing can allow it, so that feature does not work"
            );
        }
    }

    #[test]
    fn test_sending_asks_the_account_rather_than_always_refusing() {
        // The specific one that matters most. SmtpClient::new cannot send, so
        // a send path that only ever calls it is a mail client that cannot
        // post a message.
        //
        // What this cannot see: whether that call is on the path a send takes.
        // It asks only that the words appear somewhere in the controller. A
        // controller that builds a client which can send inside a branch
        // nothing reaches, and refuses everywhere else, keeps this green.
        let source =
            std::fs::read_to_string("src/application/mail_controller.rs").expect("the controller");

        assert!(
            source.contains("SmtpClient::allowed_to_send"),
            "nothing ever builds a client that can send"
        );
    }
}

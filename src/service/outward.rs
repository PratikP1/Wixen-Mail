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
    /// server what it has. Named separately rather than letting the gate guess
    /// from the verb, so the judgement is written down at the one place it
    /// applies instead of being a rule somebody has to know.
    pub fn reading_with(&self, method: Method, url: &str) -> RequestBuilder {
        self.http.request(method, url)
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

/// The gate itself, which holds the client the others are refused through.
///
/// On a list of its own because it is the one file that is supposed to hold a
/// bare client. Putting it among the gated ones would make the check below it
/// read its own client as an escape route.
#[cfg(test)]
const THE_GATE: [&str; 1] = ["src/service/outward.rs"];

#[cfg(test)]
mod completeness {
    use super::{GATED, TALKS_BUT_ONLY_READS, THE_GATE};

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
        // reading nothing, so the list is checked separately. All three lists,
        // because the census below decides whether a file is accounted for by
        // looking it up in them, and a stale entry there accounts for nothing.
        for path in GATED
            .iter()
            .chain(TALKS_BUT_ONLY_READS.iter())
            .chain(THE_GATE.iter())
        {
            assert!(
                std::path::Path::new(path).exists(),
                "{path} has moved, so the list is stale and the checks that read it prove nothing"
            );
        }
    }

    /// How a module gets a connection of its own out of this program.
    ///
    /// Four ways, and every one of them was found by reading the tree rather
    /// than by remembering. Anything that opens a socket or holds an HTTP
    /// client of its own matches one of these.
    const A_WAY_OUT: [&str; 4] = [
        "TcpStream::connect(",
        "reqwest::Client::new(",
        "reqwest::Client::builder(",
        "AsyncSmtpTransport",
    ];

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
            .collect();

        for file in every_source_file() {
            let path = file.to_string_lossy().replace('\\', "/");
            let source = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{path}: {e}"));
            // The production half only. A test may open a loopback socket, and
            // several do: that is how the gate is measured at all.
            let production = source.split("#[cfg(test)]").next().unwrap_or_default();
            let Some(how) = A_WAY_OUT.iter().find(|marker| production.contains(*marker)) else {
                continue;
            };
            assert!(
                accounted_for.contains(&path.as_str()),
                "{path} reaches a server with {how} and is on no list, so nobody has said \
                 whether it can change anything at somebody's account"
            );
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
        let source =
            std::fs::read_to_string("src/application/mail_controller.rs").expect("the controller");

        assert!(
            source.contains("SmtpClient::allowed_to_send"),
            "nothing ever builds a client that can send"
        );
    }
}

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
const GATED: [&str; 6] = [
    "src/service/tasks_api.rs",
    "src/service/google_api.rs",
    "src/service/microsoft_graph.rs",
    "src/service/caldav.rs",
    "src/service/protocols/imap.rs",
    "src/service/protocols/smtp.rs",
];

#[cfg(test)]
mod completeness {
    use super::GATED;

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

    #[test]
    fn test_every_gated_module_is_still_there() {
        // A list of paths rots. If one is renamed, the test above passes by
        // reading nothing, so the list is checked separately.
        for path in GATED {
            assert!(
                std::path::Path::new(path).exists(),
                "{path} has moved, so the gate list is stale and the test above proves nothing"
            );
        }
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

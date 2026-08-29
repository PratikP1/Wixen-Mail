//! Looking a name up in an organisation's directory.
//!
//! In a workplace, most of the people somebody writes to are in neither
//! their own contacts nor any message they have received. They are in the
//! organisation's directory, and this is how that is asked.
//!
//! # What somebody types is not a query
//!
//! The text goes into a query in a language with its own punctuation, and a
//! few characters in that language open and close clauses. Left alone, three
//! characters turn a search for one person into a search for the whole
//! organisation, or into a search for whichever entries hold an attribute the
//! person asking is not meant to read. Everything typed is escaped before it
//! goes anywhere near a query, and there are tests here for each character
//! that needs it.

use crate::common::error::redact_provider_message;
use crate::common::{Error, Result};
use crate::data::message_cache::{ContactEntry, EmailEntry};
use ldap3::{
    LdapConnAsync, LdapConnSettings, LdapError, LdapResult, Scope, SearchEntry, SearchOptions,
    ldap_escape,
};
use std::time::Duration;

/// The directory an account names, and how to reach it.
///
/// The password is deliberately not here. Secrets in this program live in the
/// operating system's credential store and never in a settings file or in the
/// database, so what is stored is the name to sign in as, and the password is
/// fetched and passed in at the moment of the search.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Directory {
    /// Where it is: `ldaps://directory.example.com`, or `ldap://host:389`
    /// where the directory offers no encrypted connection.
    pub url: String,
    /// The part of the directory to search under, as that directory names it.
    pub search_under: String,
    /// The name to sign in as, or nothing at all.
    ///
    /// Plenty of directories answer anybody who asks, and plenty refuse
    /// everybody who does not sign in. Both are ordinary, so this is an
    /// option rather than a required setting with a blank in it.
    pub sign_in_as: Option<String>,
}

/// How many people a search may bring back.
///
/// A search for one letter in a large organisation matches thousands, and a
/// list of thousands is no more use than no list at all: it takes minutes to
/// read through with a screen reader and the person being looked for is
/// somewhere in the middle of it. Past this, the answer is to ask for a
/// narrower search rather than to show the first however many and say nothing
/// about the rest.
pub const AT_MOST: usize = 50;

/// How long to wait for a directory to accept a connection.
///
/// A directory that is switched off, behind a firewall, or named wrongly in a
/// setting will not answer at all, and without this the search sits there
/// until the operating system gives up, which can be over a minute.
const BEFORE_GIVING_UP_ON_CONNECTING: Duration = Duration::from_secs(10);

/// How long to wait for an answer once connected.
///
/// The other half, and it is a separate limit because it bounds a different
/// failure: a directory that accepted the connection and then went quiet, or
/// one grinding through a search too broad for it. A connect timeout does
/// nothing about either.
const BEFORE_GIVING_UP_ON_AN_ANSWER: Duration = Duration::from_secs(20);

/// The longest one lookup can take before it gives up by itself.
///
/// Both limits together, because a search can spend the first waiting to be
/// connected and then the second waiting to be answered. Public because
/// anything waiting on a lookup has to be prepared to wait at least this long:
/// a window that gave up sooner would report a directory as silent while it
/// was still answering, and say so to somebody who has no other way to tell.
pub const AT_MOST_BEFORE_GIVING_UP: Duration = Duration::from_secs(
    BEFORE_GIVING_UP_ON_CONNECTING.as_secs() + BEFORE_GIVING_UP_ON_AN_ANSWER.as_secs(),
);

/// What to ask a directory to send back about each person.
///
/// Named rather than asking for everything. An entry in a workplace directory
/// can carry a photograph, a certificate and a manager's whole record, none of
/// which is wanted here, and asking for all of it makes every search slower
/// and drags more of somebody's personal information across the network than
/// the question needs.
/// Both spellings of the employer are here on purpose. `o` is the one the
/// original standard names and `company` is the one a Windows directory holds
/// it in, and asking for only the first left the company empty on every entry
/// from every Active Directory.
const WHAT_TO_ASK_ABOUT_EACH_PERSON: [&str; 12] = [
    "displayName",
    "cn",
    "givenName",
    "sn",
    "mail",
    "telephoneNumber",
    "mobile",
    "title",
    "department",
    "o",
    "company",
    "physicalDeliveryOfficeName",
];

/// Where a person's name might be written in a directory entry.
///
/// The five attributes that hold a name or an address in every directory
/// layout in common use: what the entry calls itself, its common name, the
/// two halves of a personal name, and the address itself, because people
/// search by the start of an address as often as by a name.
const WHERE_A_NAME_MIGHT_BE: [&str; 5] = ["displayName", "cn", "givenName", "sn", "mail"];

/// Whether there is anything to look for.
///
/// An empty search box would otherwise build a query of nothing but
/// wildcards, which asks a large organisation for the whole of itself.
fn nobody_typed_anything(typed: &str) -> bool {
    typed.trim().is_empty()
}

/// The query that looks for this name.
///
/// Wildcards around what was typed, so that three letters find the people
/// whose names hold them rather than only those whose names begin with them.
/// The wildcards are added here and never come from the typed text: anything
/// typed is escaped first, so an asterisk somebody types is an asterisk they
/// are looking for.
///
/// Narrowed to entries that have an address, because the question being asked
/// is who to write to. A meeting room, a printer and a distribution list with
/// no address of its own are all real directory entries and none of them is
/// an answer.
fn a_query_looking_for(typed: &str) -> String {
    let looking_for = ldap_escape(typed.trim());
    let clauses: String = WHERE_A_NAME_MIGHT_BE
        .iter()
        .map(|attribute| format!("({attribute}=*{looking_for}*)"))
        .collect();
    format!("(&(mail=*)(|{clauses}))")
}

/// What a contact found this way says about where it came from.
///
/// Not the name of a provider this program syncs with, on purpose: a
/// directory is read and never written, and nothing here is waiting to be
/// pushed anywhere.
pub const FROM_A_DIRECTORY: &str = "directory";

/// The label a directory address is filed under.
///
/// "Work", because a directory is somebody's employer's list of its own
/// people. Guessing anything finer from an attribute name would be inventing
/// a distinction the directory did not make.
const AT_WORK: &str = "Work";

/// One attribute of a directory entry, whatever case the server spelled it.
///
/// Attribute names in a directory have no case, and servers answer with
/// whichever spelling their schema holds: `mail`, `Mail` and `MAIL` are one
/// attribute. Reading only the spelling this code asked for loses the whole
/// entry from a server that writes it differently, and the entry still looks
/// like a real answer with nothing in it.
fn every_value_of<'a>(entry: &'a SearchEntry, attribute: &str) -> &'a [String] {
    entry
        .attrs
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(attribute))
        .map(|(_, values)| values.as_slice())
        .unwrap_or_default()
}

/// The first value of an attribute that has anything in it.
///
/// A directory may hold an attribute with an empty value, which is not the
/// same as holding the answer, and storing it would put an empty line in a
/// contact card.
fn one_value_of(entry: &SearchEntry, attribute: &str) -> Option<String> {
    every_value_of(entry, attribute)
        .iter()
        .map(|value| value.trim())
        .find(|value| !value.is_empty())
        .map(str::to_string)
}

/// The first of these attributes the directory filled in.
///
/// Directories disagree about which attribute holds a name, so the order here
/// is the order to prefer them in.
fn one_value_of_any(entry: &SearchEntry, attributes: &[&str]) -> Option<String> {
    attributes
        .iter()
        .find_map(|attribute| one_value_of(entry, attribute))
}

/// What to show this person as.
///
/// The parts are joined only when the directory gave them separately. A full
/// name is never split back into parts: "Grace Brewster Murray Hopper" splits
/// at the wrong space, and there is no rule that gets it right. Falling back
/// to the address is better than an entry showing as nothing, which cannot be
/// picked out of a list of results.
fn what_to_call_them(entry: &SearchEntry, address: &str) -> String {
    if let Some(name) = one_value_of_any(entry, &["displayName", "cn"]) {
        return name;
    }
    let parts: Vec<String> = ["givenName", "sn"]
        .iter()
        .filter_map(|attribute| one_value_of(entry, attribute))
        .collect();
    match parts.is_empty() {
        true => address.to_string(),
        false => parts.join(" "),
    }
}

/// Every address this entry holds, as the list a contact keeps.
///
/// All of them, because a person at work has more than one and the one a
/// message was about is often not the first. Same rule as everywhere else
/// contacts are built here: the main line and the list must not be able to
/// give two different answers, so the main line is the first of this list.
fn every_address_of(entry: &SearchEntry) -> Vec<String> {
    every_value_of(entry, "mail")
        .iter()
        .map(|address| address.trim())
        .filter(|address| !address.is_empty())
        .map(str::to_string)
        .collect()
}

/// Turn one directory entry into the contact shape this program already uses.
///
/// `None` when the entry has no address. A meeting room, a printer and a
/// building are all directory entries and none of them is an answer to "who
/// do I address this message to".
fn a_contact_from(entry: &SearchEntry, account_id: &str, found_at: &str) -> Option<ContactEntry> {
    let addresses = every_address_of(entry);
    let main_address = addresses.first()?.clone();
    let held_addresses: Vec<EmailEntry> = addresses
        .iter()
        .map(|address| EmailEntry {
            label: AT_WORK.to_string(),
            address: address.clone(),
            name: String::new(),
        })
        .collect();

    Some(ContactEntry {
        // The directory's own name for the entry. Nothing else about a
        // looked-up person is unique: two people share a name, and one person
        // has several addresses.
        id: entry.dn.clone(),
        account_id: account_id.to_string(),
        name: what_to_call_them(entry, &main_address),
        given_name: one_value_of(entry, "givenName"),
        family_name: one_value_of(entry, "sn"),
        email: main_address,
        phone: one_value_of_any(entry, &["telephoneNumber", "mobile"]),
        company: one_value_of_any(entry, &["o", "company"]),
        job_title: one_value_of(entry, "title"),
        website: None,
        address: one_value_of(entry, "physicalDeliveryOfficeName"),
        birthday: None,
        avatar_url: None,
        avatar_data_base64: None,
        source_provider: Some(FROM_A_DIRECTORY.to_string()),
        last_synced_at: None,
        vcard_raw: None,
        notes: None,
        favorite: false,
        created_at: found_at.to_string(),
        nickname: None,
        department: one_value_of(entry, "department"),
        relationship: None,
        emails_json: serde_json::to_string(&held_addresses).ok(),
        phones_json: None,
        addresses_json: None,
        custom_fields_json: None,
        // Nothing here was changed and nothing is owed to an address book. A
        // looked-up entry arriving marked as changed would be pushed at
        // somebody's real address book on the next sync, which is not what
        // looking a name up asked for.
        pending: false,
        known_to: Vec::new(),
    })
}

/// What to say when the account names no directory at all.
const NO_DIRECTORY_IS_SET_UP: &str = "This account does not name a directory to look people up in. Add one in the account's \
     settings: the address of the directory, and the part of it to search under.";

/// What to say when the search box is empty.
const NOTHING_TO_LOOK_FOR: &str =
    "Type at least part of a name to look for, and the directory will be searched for it.";

/// Somewhere a name can be looked up that is not this computer.
///
/// One step, which is the whole of what a lookup asks of a directory. Named
/// for what it does rather than for the protocol, so that everything deciding
/// around it, which is most of this file, can be run in a test: without this
/// seam, none of the five ways a search can fail could be reached without a
/// directory server to fail in each way on demand.
pub(crate) trait AsksADirectory {
    /// Ask for at most `at_most` entries matching `query`.
    ///
    /// The password is `None` when this directory signs nobody in, and it is
    /// never stored anywhere by anything below this point.
    async fn ask(
        &self,
        directory: &Directory,
        password: Option<&str>,
        query: &str,
        at_most: usize,
    ) -> std::result::Result<Vec<SearchEntry>, LdapError>;
}

/// Look a name up in the directory an account names.
pub async fn look_up(
    directory: Option<&Directory>,
    password: Option<&str>,
    typed: &str,
    account_id: &str,
) -> Result<Vec<ContactEntry>> {
    look_up_through(
        &TheDirectoryItself,
        directory,
        password,
        typed,
        account_id,
        &chrono::Utc::now().to_rfc3339(),
    )
    .await
}

/// The whole of the decision, with the network held at arm's length.
///
/// Everything that can be got wrong happens here: no directory set up, a
/// setting that is not an address, a sign-in with no password, an empty
/// search, a directory that does not answer, one that refuses, one that finds
/// nobody, and one that finds far too many.
async fn look_up_through<D: AsksADirectory>(
    asking: &D,
    directory: Option<&Directory>,
    password: Option<&str>,
    typed: &str,
    account_id: &str,
    found_at: &str,
) -> Result<Vec<ContactEntry>> {
    let Some(directory) = directory else {
        return Err(Error::Config(NO_DIRECTORY_IS_SET_UP.to_string()));
    };
    let named = where_this_directory_is(directory)?;
    if nobody_typed_anything(typed) {
        return Err(Error::Other(NOTHING_TO_LOOK_FOR.to_string()));
    }
    let signing_in = the_password_to_sign_in_with(directory, password, &named)?;

    // One more than will be shown, so that "exactly as many as the limit" and
    // "more than the limit" do not arrive looking the same.
    let found = asking
        .ask(
            directory,
            signing_in,
            &a_query_looking_for(typed),
            AT_MOST + 1,
        )
        .await
        .map_err(|failure| how_the_directory_failed(failure, &named, typed))?;

    if found.len() > AT_MOST {
        return Err(too_many_people_match(typed));
    }
    let people: Vec<ContactEntry> = found
        .iter()
        .filter_map(|entry| a_contact_from(entry, account_id, found_at))
        .collect();
    match people.is_empty() {
        // Empty because nothing matched, and empty because everything that
        // matched was a meeting room, are the same answer to the question
        // that was asked: there is nobody here to write to.
        true => Err(nobody_matches(typed)),
        false => Ok(people),
    }
}

/// The directory this account names, checked, and the name to call it by in a
/// message.
///
/// Checked here rather than left to the connection, because a setting that is
/// not an address is something somebody has to correct, and it should not be
/// reported the same way as a directory that is temporarily down.
///
/// Two schemes and no more. `ldaps` is a connection encrypted from the start
/// and is what a workplace directory offers; `ldap` is the plain one, still
/// what some internal directories run. The library also understands `ldapi`,
/// a socket file on this computer, which no account should be pointed at and
/// which does not exist on every platform this program has to run on.
fn where_this_directory_is(directory: &Directory) -> Result<String> {
    if directory.search_under.trim().is_empty() {
        return Err(Error::Config(
            "This account does not say which part of the directory to search under. Add it in \
             the account's settings: your organisation's directory administrator will know it."
                .to_string(),
        ));
    }
    let not_an_address = || {
        Error::Config(format!(
            "\"{}\" is not a directory address. It should start with ldaps:// for an encrypted \
             connection, or ldap:// where the directory offers none.",
            directory.url.trim()
        ))
    };
    let parsed = url::Url::parse(directory.url.trim()).map_err(|_| not_an_address())?;
    if !matches!(parsed.scheme(), "ldap" | "ldaps") {
        return Err(not_an_address());
    }
    parsed
        .host_str()
        .map(str::to_string)
        .ok_or_else(not_an_address)
}

/// The password to sign in with, or nothing, or a refusal.
///
/// A directory that signs nobody in is never sent a password, even when one
/// is on hand. A directory that does sign somebody in is never dialled
/// without one: a sign-in with a name and an empty password is an
/// unauthenticated sign-in, which many directory servers accept and then
/// treat as anonymous. That does not fail. It quietly succeeds as somebody
/// else, reading whatever that somebody is allowed to read, and every search
/// afterwards looks like a directory that holds less than it does.
fn the_password_to_sign_in_with<'a>(
    directory: &Directory,
    password: Option<&'a str>,
    named: &str,
) -> Result<Option<&'a str>> {
    let Some(sign_in_as) = &directory.sign_in_as else {
        return Ok(None);
    };
    match password.map(str::trim).filter(|held| !held.is_empty()) {
        Some(_) => Ok(password),
        None => Err(Error::Authentication(format!(
            "This account signs in to the directory at {named} as {sign_in_as}, and no password \
             for it has been saved. Add the password in the account's settings."
        ))),
    }
}

/// What to say when a search matched more people than can be shown.
fn too_many_people_match(typed: &str) -> Error {
    Error::Other(format!(
        "More than {AT_MOST} people in the directory match \"{}\", so none of them are shown. \
         Type more of the name to narrow the search down.",
        typed.trim()
    ))
}

/// What to say when a search matched nobody who can be written to.
fn nobody_matches(typed: &str) -> Error {
    Error::Other(format!(
        "Nobody in the directory matches \"{}\" and has an email address. Check the spelling, or \
         try a shorter part of the name.",
        typed.trim()
    ))
}

/// Whether a refused search is only the directory saying it holds nobody by
/// that name.
///
/// Every other refusal is something to act on: a setting to correct, a sign-in
/// to fix, a network to get on to, or a search to narrow. This one is not, and
/// it is the one that happens on the way to typing a name in full, so somebody
/// completing a recipient would hear it after every third letter.
///
/// Asked against the sentence rather than against a marker on the error,
/// because [`nobody_matches`] is the one place that sentence is written and
/// comparing with it cannot come apart from it.
pub fn means_only_that_nobody_matched(refusal: &Error, typed: &str) -> bool {
    refusal.to_string() == nobody_matches(typed).to_string()
}

/// Turn whatever went wrong into something worth reading.
///
/// The library's own errors split cleanly in two. One of them is the
/// directory answering with a result code, which is the directory saying
/// something about the request. Everything else is the connection: no route,
/// nothing listening, a certificate that would not check out, a stream that
/// ended. From where somebody is sitting those are all "it could not be
/// reached", and saying so and naming which directory is more use than the
/// library's own wording for any of them.
fn how_the_directory_failed(failure: LdapError, named: &str, typed: &str) -> Error {
    match failure {
        LdapError::Timeout { .. } => Error::Network(format!(
            "The directory at {named} took too long to answer, so the search was given up. Try \
             again, or type more of the name so there is less to search."
        )),
        LdapError::LdapResult { result } => what_the_directory_answered(result, named, typed),
        // A directory that answered, with something unreadable in it. Told
        // apart from one that never answered, because they are different
        // things to be told: this one is reachable and something in it is
        // shaped in a way this program cannot follow, so trying again will
        // not help and typing less of the name might.
        LdapError::Io { ref source } if source.kind() == std::io::ErrorKind::InvalidData => {
            Error::Protocol(format!(
                "The directory at {named} answered with an entry this could not read, so the \
                 search was stopped. Nothing was changed. Searching for a narrower name may \
                 avoid it."
            ))
        }
        could_not_connect => Error::Network(format!(
            "The directory at {named} did not answer. Check that you are connected to your \
             organisation's network, and that the directory address in this account's settings \
             is right. The directory said: {}",
            redact_provider_message(&could_not_connect.to_string())
        )),
    }
}

/// What a result code from a directory means for the person who searched.
///
/// The codes are RFC 4511's, appendix A.1. They are grouped by what the
/// person can do about them, which is the only grouping worth having: a
/// number and its standard name tell somebody nothing.
fn what_the_directory_answered(answered: LdapResult, named: &str, typed: &str) -> Error {
    const SEARCH_WAS_TOO_BROAD: [u32; 3] = [
        3,  // the directory's own time limit
        4,  // the directory's own size limit
        11, // the limit its administrator set
    ];
    const THE_SIGN_IN_WAS_NOT_ACCEPTED: [u32; 3] = [
        8,  // it wants a stronger sign-in than this
        48, // it does not accept this kind of sign-in
        49, // the name or the password is wrong
    ];
    const NOTHING_IS_THERE_TO_SEARCH: [u32; 2] = [
        32, // no such object: the part to search under does not exist
        34, // that is not a name a directory could hold
    ];

    if SEARCH_WAS_TOO_BROAD.contains(&answered.rc) {
        return too_many_people_match(typed);
    }
    if THE_SIGN_IN_WAS_NOT_ACCEPTED.contains(&answered.rc) {
        return Error::Authentication(format!(
            "The directory at {named} would not accept the sign-in. Check the name and password \
             for it in this account's settings."
        ));
    }
    if NOTHING_IS_THERE_TO_SEARCH.contains(&answered.rc) {
        return Error::Config(format!(
            "The directory at {named} has nothing at the place this account says to search \
             under. Check that setting: your organisation's directory administrator will know \
             what it should be."
        ));
    }
    if answered.rc == 50 {
        return Error::Authentication(format!(
            "The sign-in at {named} worked and it does not allow this account to search. Ask \
             whoever looks after the directory for permission to read it."
        ));
    }
    Error::Protocol(format!(
        "The directory at {named} refused the search: {}",
        redact_provider_message(&answered.to_string())
    ))
}

/// The directory itself, over the network.
///
/// Everything above this decides; this is the only part that dials. It is
/// also the only part no test here runs, which is why it is as small as it
/// can be made.
pub struct TheDirectoryItself;

impl AsksADirectory for TheDirectoryItself {
    async fn ask(
        &self,
        directory: &Directory,
        password: Option<&str>,
        query: &str,
        at_most: usize,
    ) -> std::result::Result<Vec<SearchEntry>, LdapError> {
        let settings = LdapConnSettings::new().set_conn_timeout(BEFORE_GIVING_UP_ON_CONNECTING);
        let (connection, mut asking) =
            LdapConnAsync::with_settings(settings, directory.url.trim()).await?;

        // The connection has to be driven by something or nothing sent on it
        // ever completes. It ends when the session is dropped.
        //
        // That puts a requirement on whoever calls this: the runtime it is
        // awaited on must have somewhere else to run this. On a runtime with
        // one thread and this call blocking it, the driver never runs, and
        // every search waits out the answer timeout and reports a directory
        // that went quiet. `presentation::finding_people` awaits it from a
        // blocking thread of a multi-threaded runtime, which is the shape that
        // works.
        tokio::spawn(async move {
            if let Err(ended) = connection.drive().await {
                tracing::warn!("The connection to the directory ended: {}", ended);
            }
        });
        asking.with_timeout(BEFORE_GIVING_UP_ON_AN_ANSWER);

        if let (Some(sign_in_as), Some(password)) = (&directory.sign_in_as, password) {
            asking.simple_bind(sign_in_as, password).await?.success()?;
        }

        // The directory's own limits as well as this program's. Asking the
        // server to stop early is what keeps a search for one letter from
        // being answered in full and then thrown away here, which costs the
        // network and the directory a great deal for nothing.
        asking.with_search_options(
            SearchOptions::new()
                .sizelimit(i32::try_from(at_most).unwrap_or(i32::MAX))
                .timelimit(
                    i32::try_from(BEFORE_GIVING_UP_ON_AN_ANSWER.as_secs()).unwrap_or(i32::MAX),
                ),
        );
        let (entries, _) = asking
            .search(
                directory.search_under.trim(),
                Scope::Subtree,
                query,
                WHAT_TO_ASK_ABOUT_EACH_PERSON,
            )
            .await?
            // `non_error` and not `success`, because result code 10 is a
            // referral, which is a directory saying "some of this lives
            // elsewhere" alongside the entries it does hold. Treating that as
            // a failure would turn a search that worked into an error on
            // every directory built as more than one server, which is most
            // large ones. The referrals themselves are not followed.
            .non_error()?;

        let _ = asking.unbind().await;
        // Upstream gap, caught rather than papered over: `construct` panics
        // rather than returning an error when an entry is not shaped the way
        // it expects, which its own documentation says plainly. A directory
        // is a server somebody else runs, so an entry this program cannot
        // read is a thing that will happen, and it must come out as a
        // sentence rather than as a search that vanishes.
        //
        // Caught here rather than left to the worker, because a worker that
        // stops carries no reason with it: the sentence somebody hears would
        // be nothing at all.
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            entries
                .into_iter()
                .map(SearchEntry::construct)
                .collect::<Vec<SearchEntry>>()
        }))
        .map_err(|_| LdapError::Io {
            // Carried as a kind rather than as wording, so the sentence is
            // chosen by the one place that writes sentences and this stays a
            // fact about what happened.
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "an entry the library could not read",
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every character that means something in a directory query, and so has
    /// to stop meaning it before somebody's typing is put into one.
    ///
    /// RFC 4515, section 3. The three that shape a query are the two
    /// parentheses, which open and close every clause, and the asterisk,
    /// which is the wildcard. The backslash is the escape itself, so leaving
    /// it alone would let somebody write their own escapes. The null byte
    /// ends a string in the C libraries a lot of directory servers are built
    /// on, so what follows one may never be looked at.
    const MEANS_SOMETHING_IN_A_QUERY: [char; 5] = ['*', '(', ')', '\\', '\0'];

    #[test]
    fn test_a_search_looks_for_the_name_in_the_usual_places() {
        let filter = a_query_looking_for("Lovelace");

        for attribute in ["displayName", "cn", "givenName", "sn", "mail"] {
            assert!(
                filter.contains(attribute),
                "a search did not look at {attribute}: {filter}"
            );
        }
        assert!(
            filter.contains("Lovelace"),
            "a search did not look for what was typed: {filter}"
        );
    }

    #[test]
    fn test_a_partial_name_matches_anywhere_in_the_name() {
        // Somebody types three letters and expects the people whose names
        // hold them, not only the people whose names start with them.
        let filter = a_query_looking_for("ove");

        assert!(filter.contains("*ove*"), "{filter}");
    }

    #[test]
    fn test_every_character_that_means_something_in_a_query_is_escaped() {
        // One name at a time, so a failure says which character got through,
        // and against the exact escape the standard gives rather than against
        // "it changed somehow". A backslash escapes to a backslash and two
        // digits, so "the character is gone" is not the property to check.
        let escaped_to = [
            ('*', "\\2a"),
            ('(', "\\28"),
            (')', "\\29"),
            ('\\', "\\5c"),
            ('\0', "\\00"),
        ];
        assert_eq!(
            escaped_to.len(),
            MEANS_SOMETHING_IN_A_QUERY.len(),
            "a character that means something in a query has no escape written down for it"
        );

        for (meaningful, escape) in escaped_to {
            let filter = a_query_looking_for(&format!("Ada{meaningful}Lovelace"));

            assert!(
                filter.contains(&format!("Ada{escape}Lovelace")),
                "{meaningful:?} did not become {escape}: {filter}"
            );
        }
    }

    #[test]
    fn test_a_query_never_has_more_clauses_than_it_was_built_with() {
        // The attack this escaping exists for. Left as it is, this text ends
        // the clause it lands in and starts one of its own, so a search for
        // one person becomes a search for every entry in the organisation,
        // or for the entries holding a password attribute.
        let ordinary = a_query_looking_for("Lovelace");
        let hostile = a_query_looking_for("*)(objectClass=*");

        assert_eq!(
            hostile.matches('(').count(),
            ordinary.matches('(').count(),
            "typed text opened a clause of its own: {hostile}"
        );
        assert_eq!(
            hostile.matches(')').count(),
            ordinary.matches(')').count(),
            "typed text closed a clause it did not open: {hostile}"
        );
        // And what is left is still a query. An escaping that mangled the
        // text into something no server could parse would pass the two counts
        // above and fail every search.
        assert!(
            ldap3::parse_filter(&hostile).is_ok(),
            "the escaped query is not a query any more: {hostile}"
        );
        assert!(ldap3::parse_filter(&ordinary).is_ok(), "{ordinary}");
    }

    #[test]
    fn test_a_wildcard_somebody_types_is_a_character_and_not_a_wildcard() {
        // Otherwise one asterisk is a search for the whole organisation.
        let filter = a_query_looking_for("*");

        assert!(
            !filter.contains("**"),
            "a typed asterisk stayed a wildcard: {filter}"
        );
    }

    #[test]
    fn test_a_query_only_looks_for_people_who_can_be_written_to() {
        // The point of the lookup is to address a message, so an entry with
        // no address is not an answer to the question that was asked.
        assert!(a_query_looking_for("Lovelace").contains("(mail=*)"));
    }

    // ── What comes back becomes a contact ───────────────────────────────

    fn an_entry(dn: &str, attributes: &[(&str, &[&str])]) -> SearchEntry {
        SearchEntry {
            dn: dn.to_string(),
            attrs: attributes
                .iter()
                .map(|(name, values)| {
                    (
                        name.to_string(),
                        values.iter().map(|v| v.to_string()).collect(),
                    )
                })
                .collect(),
            bin_attrs: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_a_directory_entry_becomes_the_contact_shape_this_program_uses() {
        let entry = an_entry(
            "uid=ada,ou=people,dc=example,dc=com",
            &[
                ("displayName", &["Ada Lovelace"]),
                ("givenName", &["Ada"]),
                ("sn", &["Lovelace"]),
                ("mail", &["ada@example.com"]),
                ("telephoneNumber", &["+44 20 7946 0000"]),
                ("title", &["Analyst"]),
                ("department", &["Computing"]),
                ("o", &["Example Company"]),
            ],
        );

        let contact = a_contact_from(&entry, "acct", "2026-08-24T00:00:00Z").expect("a contact");

        assert_eq!(contact.name, "Ada Lovelace");
        assert_eq!(contact.given_name.as_deref(), Some("Ada"));
        assert_eq!(contact.family_name.as_deref(), Some("Lovelace"));
        assert_eq!(contact.email, "ada@example.com");
        assert_eq!(contact.phone.as_deref(), Some("+44 20 7946 0000"));
        assert_eq!(contact.job_title.as_deref(), Some("Analyst"));
        assert_eq!(contact.department.as_deref(), Some("Computing"));
        assert_eq!(contact.company.as_deref(), Some("Example Company"));
        assert_eq!(contact.account_id, "acct");
    }

    #[test]
    fn test_a_directory_entry_is_not_an_address_book_this_program_syncs_with() {
        // Nothing here was changed, and no address book is waiting to be told
        // about it. A looked-up entry that arrived marked as changed would be
        // pushed at somebody's real address book on the next sync.
        let entry = an_entry("cn=ada", &[("mail", &["ada@example.com"])]);

        let contact = a_contact_from(&entry, "acct", "t").expect("a contact");

        assert!(!contact.pending);
        assert!(contact.known_to.is_empty());
        assert_eq!(contact.source_provider.as_deref(), Some(FROM_A_DIRECTORY));
    }

    #[test]
    fn test_two_entries_are_two_contacts_and_the_same_entry_is_the_same_one() {
        // The directory's own name for the entry is what tells them apart,
        // because nothing else about a looked-up person is unique.
        let ada = an_entry("uid=ada,dc=example", &[("mail", &["ada@example.com"])]);
        let bob = an_entry("uid=bob,dc=example", &[("mail", &["bob@example.com"])]);
        let identity = |entry: &SearchEntry| a_contact_from(entry, "acct", "t").expect("one").id;

        assert_ne!(identity(&ada), identity(&bob));
        assert_eq!(identity(&ada), identity(&ada));
    }

    #[test]
    fn test_an_entry_with_several_addresses_keeps_all_of_them() {
        // A person at work usually has more than one, and dropping the rest
        // loses the one the message was actually about.
        let entry = an_entry(
            "cn=ada",
            &[("mail", &["ada@example.com", "a.lovelace@example.com"])],
        );

        let contact = a_contact_from(&entry, "acct", "t").expect("a contact");

        assert_eq!(contact.email, "ada@example.com");
        let held = contact.emails_json.expect("the list of addresses");
        assert!(held.contains("ada@example.com"), "{held}");
        assert!(held.contains("a.lovelace@example.com"), "{held}");
    }

    #[test]
    fn test_a_name_is_taken_from_whichever_field_the_directory_filled_in() {
        // Directories disagree about which attribute holds the name, and an
        // entry showing an empty name is an entry nobody can pick out of a
        // list of results.
        let with_display = an_entry(
            "cn=ada",
            &[
                ("displayName", &["Ada Lovelace"]),
                ("cn", &["ada.lovelace"]),
                ("mail", &["ada@example.com"]),
            ],
        );
        let with_common_name = an_entry(
            "cn=ada",
            &[("cn", &["Ada Lovelace"]), ("mail", &["ada@example.com"])],
        );
        let with_parts = an_entry(
            "cn=ada",
            &[
                ("givenName", &["Ada"]),
                ("sn", &["Lovelace"]),
                ("mail", &["ada@example.com"]),
            ],
        );
        let with_nothing = an_entry("cn=ada", &[("mail", &["ada@example.com"])]);
        let named = |entry: &SearchEntry| a_contact_from(entry, "acct", "t").expect("one").name;

        assert_eq!(named(&with_display), "Ada Lovelace");
        assert_eq!(named(&with_common_name), "Ada Lovelace");
        assert_eq!(named(&with_parts), "Ada Lovelace");
        assert_eq!(
            named(&with_nothing),
            "ada@example.com",
            "an entry with no name at all showed as nothing"
        );
    }

    #[test]
    fn test_a_name_part_the_directory_did_not_give_is_not_guessed_at() {
        // Splitting a full name at a space sends "Grace Brewster Murray
        // Hopper" out with the wrong given name. The parts are recorded when
        // they were given and left empty when they were not.
        let entry = an_entry(
            "cn=hopper",
            &[
                ("displayName", &["Grace Brewster Murray Hopper"]),
                ("mail", &["grace@example.com"]),
            ],
        );

        let contact = a_contact_from(&entry, "acct", "t").expect("a contact");

        assert_eq!(contact.given_name, None);
        assert_eq!(contact.family_name, None);
    }

    #[test]
    fn test_attribute_names_are_read_however_the_directory_spells_them() {
        // Attribute names in a directory do not have a case, and servers
        // answer with whatever spelling their schema happens to hold. Reading
        // only the spelling this code asked for loses the whole entry.
        let entry = an_entry(
            "cn=ada",
            &[
                ("DisplayName", &["Ada Lovelace"]),
                ("MAIL", &["ada@example.com"]),
                ("TelephoneNumber", &["+44 20 7946 0000"]),
            ],
        );

        let contact = a_contact_from(&entry, "acct", "t").expect("a contact");

        assert_eq!(contact.name, "Ada Lovelace");
        assert_eq!(contact.email, "ada@example.com");
        assert_eq!(contact.phone.as_deref(), Some("+44 20 7946 0000"));
    }

    #[test]
    fn test_an_entry_with_no_address_is_not_somebody_to_write_to() {
        // A meeting room or a printer, which is a real directory entry and
        // not an answer to "who do I address this message to".
        let no_address = an_entry("cn=Meeting Room 3", &[("cn", &["Meeting Room 3"])]);
        let empty_address = an_entry("cn=Meeting Room 3", &[("mail", &["   "])]);

        assert!(a_contact_from(&no_address, "acct", "t").is_none());
        assert!(a_contact_from(&empty_address, "acct", "t").is_none());
    }

    #[test]
    fn test_searching_for_nothing_is_refused_rather_than_matching_everybody() {
        // A search built from an empty box is a wildcard on its own, which
        // asks a large organisation for all of it.
        for nothing in ["", "   ", "\t\n"] {
            assert!(
                nobody_typed_anything(nothing),
                "{nothing:?} was accepted as something to search for"
            );
        }
        assert!(!nobody_typed_anything("a"));
    }

    // ── Asking a directory, and every way that can go wrong ─────────────

    /// What the directory in this test does when it is asked.
    enum Answering {
        With(Vec<SearchEntry>),
        /// A result code, as the standard numbers them.
        RefusingWith(u32),
        /// Nothing answered at the other end.
        Unreachable,
        /// Something answered and never finished.
        TakingTooLong,
    }

    /// A directory that answers however the test needs, and writes down what
    /// it was asked.
    struct ADirectoryThat {
        does: Answering,
        was_asked: std::sync::Mutex<Vec<String>>,
        was_asked_for_at_most: std::sync::Mutex<Vec<usize>>,
        was_given_a_password: std::sync::Mutex<Vec<bool>>,
    }

    impl ADirectoryThat {
        fn does(answering: Answering) -> Self {
            Self {
                does: answering,
                was_asked: std::sync::Mutex::new(Vec::new()),
                was_asked_for_at_most: std::sync::Mutex::new(Vec::new()),
                was_given_a_password: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn answers_with(entries: Vec<SearchEntry>) -> Self {
            Self::does(Answering::With(entries))
        }

        fn the_query_it_was_asked(&self) -> String {
            self.was_asked
                .lock()
                .expect("what the directory was asked")
                .first()
                .cloned()
                .unwrap_or_default()
        }
    }

    impl AsksADirectory for ADirectoryThat {
        async fn ask(
            &self,
            _directory: &Directory,
            password: Option<&str>,
            query: &str,
            at_most: usize,
        ) -> std::result::Result<Vec<SearchEntry>, LdapError> {
            self.was_asked
                .lock()
                .expect("what the directory was asked")
                .push(query.to_string());
            self.was_asked_for_at_most
                .lock()
                .expect("how many were asked for")
                .push(at_most);
            self.was_given_a_password
                .lock()
                .expect("whether a password was sent")
                .push(password.is_some());

            match &self.does {
                Answering::With(entries) => Ok(entries.clone()),
                Answering::RefusingWith(code) => Err(LdapError::LdapResult {
                    result: ldap3::LdapResult {
                        rc: *code,
                        matched: String::new(),
                        text: "no".to_string(),
                        refs: Vec::new(),
                        ctrls: Vec::new(),
                    },
                }),
                Answering::Unreachable => Err(LdapError::Io {
                    source: std::io::Error::from(std::io::ErrorKind::ConnectionRefused),
                }),
                Answering::TakingTooLong => Err(LdapError::Timeout {
                    elapsed: tokio::time::timeout(
                        std::time::Duration::ZERO,
                        std::future::pending::<()>(),
                    )
                    .await
                    .expect_err("a timeout that has already run out"),
                }),
            }
        }
    }

    fn a_directory() -> Directory {
        Directory {
            url: "ldaps://directory.example.com".to_string(),
            search_under: "ou=people,dc=example,dc=com".to_string(),
            sign_in_as: None,
        }
    }

    fn somebody(name: &str, address: &str) -> SearchEntry {
        an_entry(
            &format!("cn={name}"),
            &[("displayName", &[name]), ("mail", &[address])],
        )
    }

    async fn looking_for(
        asking: &ADirectoryThat,
        directory: Option<&Directory>,
        typed: &str,
    ) -> Result<Vec<ContactEntry>> {
        look_up_through(asking, directory, None, typed, "acct", "t").await
    }

    #[tokio::test]
    async fn test_an_account_with_no_directory_says_so_rather_than_failing_oddly() {
        let asking = ADirectoryThat::answers_with(Vec::new());

        let refused = looking_for(&asking, None, "Lovelace")
            .await
            .expect_err("a refusal");

        assert!(
            matches!(refused, Error::Config(_)),
            "not a configuration problem: {refused:?}"
        );
        assert!(refused.to_string().contains("directory"), "{refused}");
        assert!(
            asking.the_query_it_was_asked().is_empty(),
            "a directory was asked something with no directory set up"
        );
    }

    #[tokio::test]
    async fn test_a_directory_that_does_not_answer_says_which_one() {
        let asking = ADirectoryThat::does(Answering::Unreachable);

        let refused = looking_for(&asking, Some(&a_directory()), "Lovelace")
            .await
            .expect_err("a refusal");

        assert!(
            matches!(refused, Error::Network(_)),
            "not a network problem: {refused:?}"
        );
        assert!(
            refused.to_string().contains("directory.example.com"),
            "the message did not say which directory: {refused}"
        );
    }

    #[tokio::test]
    async fn test_a_directory_that_takes_too_long_gives_up_and_says_so() {
        let asking = ADirectoryThat::does(Answering::TakingTooLong);

        let refused = looking_for(&asking, Some(&a_directory()), "Lovelace")
            .await
            .expect_err("a refusal");

        assert!(matches!(refused, Error::Network(_)), "{refused:?}");
        assert!(
            refused.to_string().contains("too long"),
            "the message did not say it gave up waiting: {refused}"
        );
    }

    #[tokio::test]
    async fn test_a_directory_that_refuses_the_sign_in_says_to_check_it() {
        // Result code 49 is what a directory answers when the name or the
        // password is wrong.
        let asking = ADirectoryThat::does(Answering::RefusingWith(49));

        let refused = looking_for(&asking, Some(&a_directory()), "Lovelace")
            .await
            .expect_err("a refusal");

        assert!(
            matches!(refused, Error::Authentication(_)),
            "a refused sign-in was not reported as one: {refused:?}"
        );
    }

    #[tokio::test]
    async fn test_every_way_a_directory_refuses_a_sign_in_is_read_as_one() {
        // 48 is "that kind of sign-in is not accepted", 8 is "not without a
        // stronger one, over an encrypted connection". Both are about the
        // sign-in and none of them is a network fault to be retried.
        for code in [8, 48, 49] {
            let asking = ADirectoryThat::does(Answering::RefusingWith(code));

            let refused = looking_for(&asking, Some(&a_directory()), "Lovelace")
                .await
                .expect_err("a refusal");

            assert!(
                matches!(refused, Error::Authentication(_)),
                "result code {code} was not read as a sign-in problem: {refused:?}"
            );
        }
    }

    #[tokio::test]
    async fn test_a_directory_that_finds_nobody_says_so_in_a_sentence() {
        let asking = ADirectoryThat::answers_with(Vec::new());

        let refused = looking_for(&asking, Some(&a_directory()), "Lovelace")
            .await
            .expect_err("a refusal");

        let said = refused.to_string();
        assert!(
            said.contains("Lovelace"),
            "the message did not say what was searched for: {said}"
        );
        assert!(said.contains("Nobody") || said.contains("nobody"), "{said}");
    }

    #[tokio::test]
    async fn test_a_search_that_matches_too_many_people_asks_for_a_narrower_one() {
        // A search for one letter in a large organisation matches thousands,
        // and a list of thousands is no more use than no list at all.
        let crowd: Vec<SearchEntry> = (0..=AT_MOST)
            .map(|n| somebody(&format!("Person {n}"), &format!("p{n}@example.com")))
            .collect();
        let asking = ADirectoryThat::answers_with(crowd);

        let refused = looking_for(&asking, Some(&a_directory()), "a")
            .await
            .expect_err("a refusal");

        let said = refused.to_string();
        assert!(
            said.contains(&AT_MOST.to_string()),
            "the message did not say how many is too many: {said}"
        );
        assert!(said.contains("more"), "{said}");
    }

    #[tokio::test]
    async fn test_a_directory_that_stopped_early_itself_gets_the_same_answer() {
        // Result code 4 is the directory saying it stopped at its own limit.
        // It is the same situation as finding too many here, and reporting it
        // as an unexplained protocol fault would send somebody looking for a
        // broken server instead of typing more letters.
        let asking = ADirectoryThat::does(Answering::RefusingWith(4));

        let refused = looking_for(&asking, Some(&a_directory()), "a")
            .await
            .expect_err("a refusal");

        assert!(
            refused.to_string().contains("more"),
            "a directory that stopped early did not ask for a narrower search: {refused}"
        );
    }

    #[tokio::test]
    async fn test_a_search_asks_for_one_more_than_it_will_show() {
        // Otherwise "exactly as many as the limit" and "more than the limit"
        // arrive looking the same, and the second one gets quietly cut down
        // to a list that says nothing about what is missing.
        let asking = ADirectoryThat::answers_with(vec![somebody("Ada", "ada@example.com")]);

        looking_for(&asking, Some(&a_directory()), "Ada")
            .await
            .expect("one person");

        assert_eq!(
            *asking
                .was_asked_for_at_most
                .lock()
                .expect("how many were asked for"),
            vec![AT_MOST + 1]
        );
    }

    #[tokio::test]
    async fn test_the_people_a_directory_finds_come_back_as_contacts() {
        let asking = ADirectoryThat::answers_with(vec![
            somebody("Ada Lovelace", "ada@example.com"),
            somebody("Adam Smith", "adam@example.com"),
        ]);

        let found = looking_for(&asking, Some(&a_directory()), "Ada")
            .await
            .expect("two people");

        assert_eq!(found.len(), 2);
        assert_eq!(found[0].name, "Ada Lovelace");
        assert_eq!(found[0].email, "ada@example.com");
    }

    #[tokio::test]
    async fn test_what_somebody_types_reaches_the_directory_escaped() {
        // The security test that is not about a function in isolation: what
        // actually leaves this program is the escaped query, not the typed
        // text with escaping applied somewhere else and forgotten here.
        let asking = ADirectoryThat::answers_with(vec![somebody("Ada", "ada@example.com")]);

        let _ = looking_for(&asking, Some(&a_directory()), "*)(objectClass=*").await;

        let asked = asking.the_query_it_was_asked();
        assert!(
            !asked.contains("objectClass=*)"),
            "typed text reached the directory as a clause of its own: {asked}"
        );
        assert!(asked.contains("\\2a\\29\\28"), "{asked}");
    }

    #[tokio::test]
    async fn test_nothing_is_asked_of_a_directory_when_nothing_was_typed() {
        let asking = ADirectoryThat::answers_with(Vec::new());

        let refused = looking_for(&asking, Some(&a_directory()), "   ")
            .await
            .expect_err("a refusal");

        assert!(
            asking.the_query_it_was_asked().is_empty(),
            "a directory was asked for everybody"
        );
        assert!(refused.to_string().contains("name"), "{refused}");
    }

    #[tokio::test]
    async fn test_a_directory_that_needs_no_sign_in_is_not_sent_a_password() {
        let asking = ADirectoryThat::answers_with(vec![somebody("Ada", "ada@example.com")]);

        look_up_through(
            &asking,
            Some(&a_directory()),
            Some("not-needed"),
            "Ada",
            "acct",
            "t",
        )
        .await
        .expect("one person");

        assert_eq!(
            *asking
                .was_given_a_password
                .lock()
                .expect("whether a password was sent"),
            vec![false],
            "a password was sent to a directory that signs nobody in"
        );
    }

    #[tokio::test]
    async fn test_a_directory_that_needs_a_sign_in_is_not_dialled_without_a_password() {
        // A sign-in with a name and an empty password is an unauthenticated
        // sign-in, which many directories accept and treat as anonymous. So
        // it does not fail: it quietly succeeds as somebody else, with
        // whatever that somebody is allowed to read.
        let asking = ADirectoryThat::answers_with(Vec::new());
        let needs_a_sign_in = Directory {
            sign_in_as: Some("cn=reader,dc=example,dc=com".to_string()),
            ..a_directory()
        };

        for no_password in [None, Some(""), Some("   ")] {
            let refused = look_up_through(
                &asking,
                Some(&needs_a_sign_in),
                no_password,
                "Ada",
                "acct",
                "t",
            )
            .await
            .expect_err("a refusal");

            assert!(
                matches!(refused, Error::Authentication(_)),
                "{no_password:?} was not refused as a missing password: {refused:?}"
            );
        }
        assert!(
            asking.the_query_it_was_asked().is_empty(),
            "a directory was dialled with no password to sign in with"
        );
    }

    #[tokio::test]
    async fn test_a_directory_named_by_something_that_is_not_a_directory_is_refused() {
        // Configuration, not something a person typed, so this is about a
        // setting somebody has to correct rather than a search to retry.
        let asking = ADirectoryThat::answers_with(Vec::new());

        for wrong in [
            "https://directory.example.com",
            "directory.example.com",
            "",
            "ldapi://var/run/ldapi",
        ] {
            let named = Directory {
                url: wrong.to_string(),
                ..a_directory()
            };

            let refused = look_up_through(&asking, Some(&named), None, "Ada", "acct", "t")
                .await
                .expect_err("a refusal");

            assert!(
                matches!(refused, Error::Config(_)),
                "{wrong:?} was not refused as a setting to correct: {refused:?}"
            );
        }
    }

    #[tokio::test]
    async fn test_a_directory_with_nowhere_to_search_is_refused() {
        let asking = ADirectoryThat::answers_with(Vec::new());
        let nowhere = Directory {
            search_under: "   ".to_string(),
            ..a_directory()
        };

        assert!(matches!(
            look_up_through(&asking, Some(&nowhere), None, "Ada", "acct", "t")
                .await
                .expect_err("a refusal"),
            Error::Config(_)
        ));
    }

    #[tokio::test]
    async fn test_a_directory_answering_something_else_entirely_is_still_reported() {
        // Anything the standard has a code for and this code has no better
        // sentence about. It must not vanish into an empty list of people.
        let asking = ADirectoryThat::does(Answering::RefusingWith(53));

        let refused = looking_for(&asking, Some(&a_directory()), "Ada")
            .await
            .expect_err("a refusal");

        assert!(matches!(refused, Error::Protocol(_)), "{refused:?}");
    }

    #[tokio::test]
    async fn test_entries_with_no_address_do_not_count_as_people_who_were_found() {
        // A directory that answers with meeting rooms found nobody to write
        // to, and saying "two results" and showing none is worse than saying
        // nobody matched.
        let rooms = vec![
            an_entry("cn=Room 1", &[("cn", &["Room 1"])]),
            an_entry("cn=Room 2", &[("cn", &["Room 2"])]),
        ];
        let asking = ADirectoryThat::answers_with(rooms);

        let refused = looking_for(&asking, Some(&a_directory()), "Room")
            .await
            .expect_err("a refusal");

        assert!(
            refused.to_string().to_lowercase().contains("nobody"),
            "{refused}"
        );
    }

    #[test]
    fn test_the_limit_on_how_many_come_back_is_a_useful_size() {
        // Big enough that an ordinary search is never cut short, small enough
        // that the list can be read through with a screen reader.
        assert!((10..=200).contains(&AT_MOST), "{AT_MOST}");
    }

    #[test]
    fn test_a_directory_is_given_a_time_limit_to_answer_within() {
        // A directory that never answers must not take the window with it.
        // Both limits are needed: one bounds getting a connection at all, and
        // the other bounds a search that connected and then went quiet.
        assert!(BEFORE_GIVING_UP_ON_CONNECTING <= std::time::Duration::from_secs(30));
        assert!(BEFORE_GIVING_UP_ON_AN_ANSWER <= std::time::Duration::from_secs(60));
    }
}

#[cfg(test)]
mod what_is_asked_for_and_what_is_read {
    use super::*;

    /// Every attribute name this file reads off an answer.
    ///
    /// Read out of the source rather than listed here, because a list written
    /// out by hand beside the one it is checking is the second copy that
    /// drifts. Every read in this file goes through one of three functions and
    /// each of them takes `entry` first, so the names are whatever is quoted
    /// between that and the closing bracket.
    fn every_attribute_this_reads(source: &str) -> Vec<String> {
        let before_the_tests = source.split("#[cfg(test)]").next().unwrap_or_default();
        let mut found = Vec::new();
        for after_entry in before_the_tests.split("entry, ").skip(1) {
            let Some(call) = after_entry.split(')').next() else {
                continue;
            };
            for quoted in call.split('"').skip(1).step_by(2) {
                found.push(quoted.to_string());
            }
        }
        found
    }

    #[test]
    fn test_every_attribute_this_reads_is_one_it_asked_the_directory_for() {
        // A directory sends back the attributes it was asked for and no
        // others, so reading one that was never asked for is a field that is
        // empty on every entry from every server, for ever, with nothing
        // saying why. `company` was read and never asked for, which is the
        // attribute Active Directory holds an employer in: every contact
        // looked up on a Windows directory had no company on it.
        let source = std::fs::read_to_string("src/service/directory.rs")
            .expect("this file, to read its own reads back");
        let read = every_attribute_this_reads(&source);

        assert!(
            !read.is_empty(),
            "no reads were found at all, so this check proves nothing"
        );
        for attribute in &read {
            assert!(
                WHAT_TO_ASK_ABOUT_EACH_PERSON.contains(&attribute.as_str()),
                "{attribute} is read off an answer and never asked for, so it is \
                 empty on every entry: {read:?}"
            );
        }
    }

    #[test]
    fn test_the_reading_of_this_file_can_see_a_read_that_was_never_asked_for() {
        // The check above says nothing unless it can fail. Two attributes,
        // one asked for and one not, told apart.
        let made_up = "\
            fn a_contact_from() {\n\
                one_value_of(entry, \"mail\");\n\
                one_value_of_any(entry, &[\"telephoneNumber\", \"neverAskedFor\"]);\n\
            }\n";

        let read = every_attribute_this_reads(made_up);

        assert!(read.contains(&"mail".to_string()), "{read:?}");
        assert!(read.contains(&"neverAskedFor".to_string()), "{read:?}");
        assert!(
            !WHAT_TO_ASK_ABOUT_EACH_PERSON.contains(&"neverAskedFor"),
            "the example has to be an attribute nothing asks for"
        );
    }
}

#[cfg(test)]
mod what_is_worth_saying_about_a_refusal {
    use super::*;

    #[test]
    fn test_a_directory_that_simply_holds_nobody_by_that_name_says_only_that() {
        // Told apart from every other refusal, because the two are acted on
        // differently. A directory with nobody by that name has nothing to add
        // when the contacts on this computer already matched somebody, and
        // saying so on every third keystroke is a flood.
        let nobody = nobody_matches("Lovelace");

        assert!(means_only_that_nobody_matched(&nobody, "Lovelace"));
    }

    #[test]
    fn test_everything_else_is_worth_hearing_about() {
        // Each of these is a thing somebody can do something about: a setting
        // to correct, a sign-in to fix, a network to get on to, or a search to
        // narrow. Staying quiet about any of them leaves a directory that is
        // answering nothing looking like an organisation with nobody in it.
        let worth_hearing = [
            too_many_people_match("Lovelace"),
            Error::Config("no directory is set up".to_string()),
            Error::Authentication("the sign-in was refused".to_string()),
            Error::Network("it did not answer".to_string()),
            Error::Protocol("it answered with something unreadable".to_string()),
        ];

        for refusal in worth_hearing {
            assert!(
                !means_only_that_nobody_matched(&refusal, "Lovelace"),
                "{refusal} was treated as nothing worth saying"
            );
        }
    }

    #[test]
    fn test_the_same_refusal_about_a_different_name_is_not_this_one() {
        // The sentence names what was searched for, so an answer left over
        // from an earlier search must not be read as the answer to this one.
        let nobody = nobody_matches("Lovelace");

        assert!(!means_only_that_nobody_matched(&nobody, "Babbage"));
    }
}

#[cfg(test)]
mod an_answer_this_cannot_read {
    use super::*;

    #[test]
    fn test_a_directory_that_answers_with_nonsense_says_so_rather_than_vanishing() {
        // The library this uses says in its own documentation that it panics
        // rather than erroring when an entry is not shaped the way it expects.
        // A directory is a server somebody else runs, so an entry this cannot
        // read is a thing that happens, and it has to come out as a sentence.
        // Without the guard the search goes down and takes its reason with it,
        // and what somebody hears is nothing at all.
        let unreadable = LdapError::Io {
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "an entry the library could not read",
            ),
        };

        let said = how_the_directory_failed(unreadable, "ldap.example.com", "Ada").to_string();

        assert!(said.contains("ldap.example.com"), "{said}");
        assert!(
            said.contains("could not read"),
            "the sentence does not say what went wrong: {said}"
        );
        // Not the sentence for a directory that never answered. This one did.
        assert!(
            !said.contains("did not answer"),
            "a directory that answered was reported as unreachable: {said}"
        );
    }

    #[test]
    fn test_a_directory_that_really_is_unreachable_still_says_that() {
        // The other half, so the arm above cannot be written in a way that
        // swallows every failure into one wording.
        let unreachable = LdapError::Io {
            source: std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused"),
        };

        let said = how_the_directory_failed(unreachable, "ldap.example.com", "Ada").to_string();

        assert!(said.contains("did not answer"), "{said}");
    }
}

//! Looking somebody up without stopping the window.
//!
//! One thread draws this application, reads the keyboard and talks to the
//! screen reader. Reading the address book is a database query and asking an
//! organisation's directory is a conversation over a network, and either done
//! on that thread is a window that has stopped: for somebody working by ear,
//! indistinguishable from a crash.
//!
//! So the work is handed to the same place every other slow job here goes, and
//! the answer comes back on a channel the compose window looks at on a timer.
//! What is decided is in [`crate::application::looking_people_up`], which can
//! be tested; what is here is the handing over.

use crate::application::looking_people_up as looking;
use crate::common::paths::AppPaths;
use crate::data::account::Account;
use crate::data::message_cache::MessageCache;
use crate::presentation::wx_compose::FindingPeople;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Build what the compose window uses to find people to write to.
///
/// Each request names the account of the entry chosen in the From list, so
/// nothing here turns a position in that list back into an account: an
/// account with other addresses holds several positions.
pub fn through(runtime: &Arc<Runtime>) -> FindingPeople {
    let (answers_go_to, answers) = async_channel::unbounded::<looking::WhoWasFound>();
    let runtime = runtime.clone();

    FindingPeople {
        answers,
        start: Box::new(move |asked| {
            let account_id = asked.from_account_id.clone();
            let answers_go_to = answers_go_to.clone();
            let handle = runtime.handle().clone();
            // `spawn_blocking` and not `spawn`: this opens a database, which
            // is a blocking read, and it belongs on a thread that is allowed
            // to block rather than on one running everything else.
            runtime.spawn_blocking(move || {
                let found = match account_id {
                    // No account, so there is no address book to read and no
                    // directory to ask. Answered rather than left unanswered,
                    // because a question with no answer is a window that says
                    // nothing.
                    None => looking::WhoWasFound {
                        search: asked.search,
                        name: asked.name,
                        everybody: Vec::new(),
                        trouble: None,
                    },
                    Some(account_id) => who_matches(&asked, &account_id, &handle),
                };
                handle.block_on(async {
                    let _ = answers_go_to.send(found).await;
                });
            });
        }),
    }
}

/// Everybody this name matches, in both places, and what went wrong.
///
/// Never given the typed name to log. A name being typed into a To line is a
/// person's name and belongs in no file: the sentences here name the account
/// and what failed, and the search itself is not written down anywhere.
fn who_matches(
    asked: &looking::LookFor,
    account_id: &str,
    handle: &tokio::runtime::Handle,
) -> looking::WhoWasFound {
    let mut trouble: Vec<String> = Vec::new();
    let address_book = the_address_book(&mut trouble);
    let from_your_contacts = address_book
        .as_ref()
        .map(|cache| the_contacts_here(cache, account_id, &asked.name, &mut trouble))
        .unwrap_or_default();
    let from_the_directory = the_organisation(account_id, &asked.name, handle, &mut trouble);
    let account = address_book
        .as_ref()
        .and_then(|cache| the_account(cache, account_id));
    let from_microsoft =
        the_people_microsoft_knows(account.as_ref(), &asked.name, handle, &mut trouble);

    looking::WhoWasFound {
        search: asked.search,
        name: asked.name.clone(),
        everybody: looking::everybody_found(from_your_contacts, from_the_directory, from_microsoft),
        trouble: match trouble.is_empty() {
            true => None,
            false => Some(trouble.join(" ")),
        },
    }
}

/// The address book on this computer, which also holds the accounts.
///
/// The worker opens the cache itself rather than carrying the window's, which
/// is how every other worker here reaches it: a database handle belongs to the
/// thread that opened it. Opened once for a search, for the contacts and for
/// the account Microsoft may be asked for.
fn the_address_book(trouble: &mut Vec<String>) -> Option<MessageCache> {
    let Some(dir) = AppPaths::resolve().ok().map(|paths| paths.cache_dir()) else {
        trouble.push("There is nowhere on this computer to keep contacts yet.".to_string());
        return None;
    };
    match MessageCache::new(dir, None) {
        Ok(cache) => Some(cache),
        Err(why) => {
            tracing::warn!("The contacts on this computer could not be opened: {why}");
            trouble.push("Your contacts on this computer could not be read.".to_string());
            None
        }
    }
}

/// The account a search is made from, as it is stored.
///
/// Nothing when it cannot be read, which asks Microsoft nothing: a search
/// that cannot tell which account it is for has no business telling any
/// provider a typed name.
fn the_account(cache: &MessageCache, account_id: &str) -> Option<Account> {
    match cache.load_accounts() {
        Ok(accounts) => accounts
            .into_iter()
            .find(|account| account.id == account_id),
        Err(why) => {
            tracing::warn!("The accounts could not be read, so Microsoft was not asked: {why}");
            None
        }
    }
}

/// The people in the address book on this computer who match.
fn the_contacts_here(
    cache: &MessageCache,
    account_id: &str,
    name: &str,
    trouble: &mut Vec<String>,
) -> Vec<looking::Somebody> {
    // One more than will be shown, so that "exactly as many as the limit" and
    // "more than the limit" do not arrive looking the same. The same reason
    // `service::directory` asks for one more than it shows.
    let asked_for = looking::AT_MOST_TO_READ_THROUGH + 1;
    let answered = match cache.search_contacts_for_account(account_id, name, asked_for) {
        Ok(answered) => answered,
        Err(why) => {
            tracing::warn!("Your contacts could not be searched: {why}");
            trouble.push("Your contacts on this computer could not be read.".to_string());
            return Vec::new();
        }
    };
    match looking::from_your_contacts(&answered) {
        Ok(found) => found,
        Err(too_many) => {
            trouble.push(too_many);
            Vec::new()
        }
    }
}

/// The people in this account's directory who match, if it names one.
///
/// No directory means nothing at all is sent anywhere, which is what a fresh
/// installation does and what every account that has not named one keeps
/// doing.
fn the_organisation(
    account_id: &str,
    name: &str,
    handle: &tokio::runtime::Handle,
    trouble: &mut Vec<String>,
) -> Vec<looking::Somebody> {
    let settings = match crate::data::config::ConfigManager::load_stored() {
        Ok(settings) => settings,
        Err(why) => {
            tracing::warn!("The settings could not be read, so no directory was asked: {why}");
            return Vec::new();
        }
    };
    let Some(directory) = settings.app_config().directory_for(account_id) else {
        return Vec::new();
    };

    let password = match the_password_to_offer(directory, account_id) {
        Ok(password) => password,
        Err(said) => {
            trouble.push(said);
            return Vec::new();
        }
    };
    let asked = handle.block_on(crate::service::directory::look_up(
        Some(directory),
        password.as_deref(),
        name,
        account_id,
    ));
    match asked {
        Ok(people) => people
            .iter()
            .filter_map(looking::Somebody::from_contact)
            .collect(),
        // A directory that simply holds nobody by that name has nothing to add
        // and nothing to say: on the way to typing a name in full it would
        // otherwise complain after every letter. Everything else is a thing
        // somebody can act on and is carried through to them.
        Err(why) => {
            if !crate::service::directory::means_only_that_nobody_matched(&why, name) {
                trouble.push(why.to_string());
            }
            Vec::new()
        }
    }
}

/// The people Microsoft finds for this account, if it is one Microsoft is
/// asked for.
///
/// Only an Outlook or Office 365 account signed in through the browser, which
/// [`looking::microsoft_is_asked_for`] decides; for every other account
/// nothing is asked and nothing is said. A sign-in that cannot ask, which is
/// every Microsoft account signed in before people search asked for its
/// permission, and a search Microsoft refuses each add one sentence to the
/// same trouble line the directory uses.
fn the_people_microsoft_knows(
    account: Option<&Account>,
    name: &str,
    handle: &tokio::runtime::Handle,
    trouble: &mut Vec<String>,
) -> Vec<looking::Somebody> {
    let Some(account) = account.filter(|account| looking::microsoft_is_asked_for(account)) else {
        return Vec::new();
    };
    let Some(token) = handle.block_on(crate::service::oauth::a_people_token_for(&account.id))
    else {
        trouble.push(looking::SIGN_IN_AGAIN_FOR_PEOPLE_SEARCH.to_string());
        return Vec::new();
    };
    let asked = handle.block_on(
        crate::service::microsoft_graph::MsGraphClient::new().people_matching(
            &token,
            name,
            looking::AT_MOST_TO_READ_THROUGH,
        ),
    );
    asked.unwrap_or_else(|why| {
        trouble.push(why.to_string());
        Vec::new()
    })
}

/// The password to offer the directory, or the sentence saying why none can
/// be.
///
/// Read only for a directory that signs somebody in: one that answers anybody
/// is never sent a password, so a credential store that will not answer is no
/// reason not to ask it. Whether the password may cross the network, and what
/// to say when none is saved, is `service::directory`'s to decide.
///
/// A store that will not give the password up stops the lookup. Asking
/// without it would be refused for a password nobody saved, which sends
/// somebody off to type it again when the store is what is wrong.
fn the_password_to_offer(
    directory: &crate::service::directory::Directory,
    account_id: &str,
) -> std::result::Result<Option<String>, String> {
    if directory.sign_in_as.is_none() {
        return Ok(None);
    }
    crate::service::directory::the_saved_password(account_id)
        .map_err(|why| format!("{why}. The directory was not asked."))
}

#[cfg(test)]
mod the_password_offered {
    use super::the_password_to_offer;
    use crate::service::directory::Directory;
    use crate::service::secret_store;

    fn a_directory(sign_in_as: Option<&str>) -> Directory {
        Directory {
            url: "ldaps://directory.example.com".to_string(),
            search_under: "dc=example,dc=com".to_string(),
            sign_in_as: sign_in_as.map(str::to_string),
        }
    }

    #[test]
    fn test_a_directory_that_signs_nobody_in_is_offered_no_password_and_the_store_is_not_asked() {
        // A store that will not answer is no reason to stop asking a
        // directory that never wanted a password.
        secret_store::refuse("the credential store is locked");
        let offered = the_password_to_offer(&a_directory(None), "acc-1");
        secret_store::allow();

        assert_eq!(offered, Ok(None));
    }

    #[test]
    fn test_the_password_saved_for_a_directory_is_the_one_offered() {
        secret_store::write("wixen-mail-directory", "acc-1", "hunter2")
            .expect("the directory password to be kept");

        let offered = the_password_to_offer(&a_directory(Some("cn=reader")), "acc-1");

        assert_eq!(offered, Ok(Some("hunter2".to_string())));
    }

    #[test]
    fn test_a_password_the_store_will_not_give_up_is_said_and_the_directory_not_asked() {
        // Looking up without it would be refused for a password nobody saved,
        // which sends somebody to type one again when the store is the
        // trouble.
        secret_store::refuse("the credential store is locked");
        let offered = the_password_to_offer(&a_directory(Some("cn=reader")), "acc-1");
        secret_store::allow();

        let said = offered.expect_err("a sentence rather than no password");
        assert!(said.contains("the credential store is locked"), "{said}");
        assert!(said.contains("not asked"), "{said}");
    }
}

#[cfg(test)]
mod asking_microsoft {
    use super::the_people_microsoft_knows;
    use crate::application::looking_people_up::SIGN_IN_AGAIN_FOR_PEOPLE_SEARCH;
    use crate::data::account::Account;

    fn an_account(id: &str, email: &str, provider: Option<&str>, oauth: bool) -> Account {
        let mut account = Account::new("Work".to_string(), email.to_string());
        account.id = id.to_string();
        account.provider = provider.map(str::to_string);
        account.use_oauth = oauth;
        account
    }

    /// What asking Microsoft for this account found, and what it said.
    fn asking_for(account: Option<&Account>) -> (usize, Vec<String>) {
        let runtime = tokio::runtime::Runtime::new().expect("a runtime");
        let mut trouble = Vec::new();
        let found = the_people_microsoft_knows(account, "ada", runtime.handle(), &mut trouble);
        (found.len(), trouble)
    }

    #[test]
    fn test_an_account_that_does_not_sign_in_to_microsoft_is_never_asked_about_people() {
        // None of these has a people token, so an account asked about would
        // come back with the sentence saying to sign in again. Silence is the
        // proof that nothing was asked.
        let never = [
            an_account("13-28-imap", "ada@example.com", None, false),
            an_account("13-28-gmail", "ada@gmail.com", None, true),
            an_account("13-28-outlook-password", "ada@outlook.com", None, false),
        ];

        for account in &never {
            assert_eq!(asking_for(Some(account)), (0, Vec::new()), "{account:?}");
        }
        assert_eq!(asking_for(None), (0, Vec::new()));
    }

    #[test]
    fn test_an_outlook_browser_sign_in_with_no_people_token_is_told_to_sign_in_again() {
        // The account signed in before people search asked for its
        // permission, which is every account on the day this arrives.
        let account = an_account("13-28-outlook-browser", "ada@outlook.com", None, true);

        assert_eq!(
            asking_for(Some(&account)),
            (0, vec![SIGN_IN_AGAIN_FOR_PEOPLE_SEARCH.to_string()])
        );
    }
}

#[cfg(test)]
mod it_is_reached {
    /// The one call that opens a compose window in the running application.
    ///
    /// Read out of the source, because nothing else can answer the question
    /// this file exists to answer: whether the window somebody types into is
    /// given anything to look people up with. Everything below could be built,
    /// tested and correct while the window was handed `None`, and this project
    /// has shipped exactly that shape more than once.
    fn how_the_window_is_opened() -> String {
        let app = std::fs::read_to_string("src/presentation/wx_app.rs").expect("the main window");
        let after = app
            .split("wx_compose::show_compose_dialog_full(")
            .nth(1)
            .expect("the call that opens a compose window");
        after.split(") {").next().unwrap_or_default().to_string()
    }

    #[test]
    fn test_the_compose_window_is_given_a_way_to_find_people() {
        let call = how_the_window_is_opened();

        assert!(
            call.contains("finding_people::through"),
            "the compose window is opened without anything to look people up with, so \
             typing a name finds nobody:\n{call}"
        );
        assert!(
            !call.contains("None,\n        saver") && !call.contains("None, saver"),
            "the compose window is handed nothing to look people up with:\n{call}"
        );
    }

    #[test]
    fn test_the_reading_of_that_call_can_see_what_is_in_it() {
        // The check above says nothing unless it is really reading the call.
        let call = how_the_window_is_opened();

        assert!(
            call.contains("signature"),
            "the call was not read at all, so the check above proves nothing:\n{call}"
        );
    }
}
